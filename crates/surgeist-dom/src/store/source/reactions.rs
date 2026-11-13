//! Ordered custom-element reactions from committed native mutation steps.
use super::*;
use std::{collections::LinkedList, sync::Mutex};

/// One native reaction occurrence, in the order reached by the original edit.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct DomCustomElementReaction<'a> {
    pub element: NodeId,
    pub definition: crate::DomRegistryDefinitionId,
    pub kind: DomCustomElementReactionKind<'a>,
}

/// Payload captured by the native mutation algorithm.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[non_exhaustive]
pub enum DomCustomElementReactionKind<'a> {
    Upgrade,
    Connected,
    Disconnected,
    ConnectedMove,
    Adopted {
        old_document: crate::DocumentId,
        new_document: crate::DocumentId,
    },
    AttributeChanged {
        local_name: &'a [u16],
        old_value: Option<&'a [u16]>,
        new_value: Option<&'a [u16]>,
        namespace: Option<&'a [u16]>,
    },
}

#[derive(Clone, Debug)]
enum ReactionKind<D> {
    Upgrade,
    Connected,
    Disconnected,
    ConnectedMove,
    Adopted {
        old_document: D,
        new_document: D,
    },
    AttributeChanged {
        local_name: DocumentString,
        old_value: Option<DocumentString>,
        new_value: Option<DocumentString>,
        namespace: Option<DocumentString>,
    },
}

#[derive(Debug)]
struct Reaction {
    element: NodeId,
    definition: crate::DomRegistryDefinitionId,
    kind: ReactionKind<crate::DocumentId>,
}

impl Reaction {
    fn view(&self) -> DomCustomElementReaction<'_> {
        let kind = match &self.kind {
            ReactionKind::Upgrade => DomCustomElementReactionKind::Upgrade,
            ReactionKind::Connected => DomCustomElementReactionKind::Connected,
            ReactionKind::Disconnected => DomCustomElementReactionKind::Disconnected,
            ReactionKind::ConnectedMove => DomCustomElementReactionKind::ConnectedMove,
            ReactionKind::Adopted {
                old_document,
                new_document,
            } => DomCustomElementReactionKind::Adopted {
                old_document: *old_document,
                new_document: *new_document,
            },
            ReactionKind::AttributeChanged {
                local_name,
                old_value,
                new_value,
                namespace,
            } => DomCustomElementReactionKind::AttributeChanged {
                local_name: local_name.code_units(),
                old_value: old_value.as_ref().map(DocumentString::code_units),
                new_value: new_value.as_ref().map(DocumentString::code_units),
                namespace: namespace.as_ref().map(DocumentString::code_units),
            },
        };
        DomCustomElementReaction {
            element: self.element,
            definition: self.definition,
            kind,
        }
    }
}

/// One published segment's reaction facts and their retained backing.
pub struct DomCustomElementReactions {
    slot: Arc<Slot>,
    batch: Batch,
}

#[derive(Default)]
struct Batch {
    facts: Vec<Reaction>,
    cursor: usize,
    _backing: Option<custody::Debit>,
    _strings: Vec<custody::SharedDebit>,
}

enum SlotState {
    Ready(Batch),
    Loaned,
    Settled,
}

struct Slot {
    operation: crate::DomSourceOperationId,
    state: Mutex<SlotState>,
    _backing: Option<custody::Debit>,
}

impl Slot {
    fn lock(&self) -> std::sync::MutexGuard<'_, SlotState> {
        self.state
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
    }
}

impl DomCustomElementReactions {
    pub fn operation(&self) -> crate::DomSourceOperationId {
        self.slot.operation
    }

    pub fn facts(&self) -> impl ExactSizeIterator<Item = DomCustomElementReaction<'_>> {
        self.remaining().iter().map(Reaction::view)
    }

    /// Borrow the next occurrence without advancing the receipt.
    pub fn front(&self) -> Option<DomCustomElementReaction<'_>> {
        self.remaining().first().map(Reaction::view)
    }

    fn remaining(&self) -> &[Reaction] {
        &self.batch.facts[self.batch.cursor..]
    }

    /// Mark the front occurrence settled after its exact native/guest path.
    pub fn acknowledge_front(&mut self) {
        if self.batch.cursor < self.batch.facts.len() {
            self.batch.cursor += 1;
        }
    }
}

impl std::fmt::Debug for DomCustomElementReactions {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("DomCustomElementReactions")
            .field("operation", &self.operation())
            .field("facts", &self.remaining())
            .finish_non_exhaustive()
    }
}

impl Drop for DomCustomElementReactions {
    fn drop(&mut self) {
        let mut slot = self.slot.lock();
        *slot = if self.remaining().is_empty() {
            SlotState::Settled
        } else {
            SlotState::Ready(std::mem::take(&mut self.batch))
        };
    }
}

#[derive(Default)]
pub(crate) struct Inbox(Mutex<LinkedList<Arc<Slot>>>);

impl std::fmt::Debug for Inbox {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Inbox").finish_non_exhaustive()
    }
}

impl Inbox {
    fn lock(&self) -> std::sync::MutexGuard<'_, LinkedList<Arc<Slot>>> {
        self.0
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
    }

    pub(crate) fn take(
        &self,
        operation: crate::DomSourceOperationId,
        inspections: usize,
    ) -> Result<Option<DomCustomElementReactions>, DomError> {
        let mut queue = self.lock();
        if queue.len() > inspections {
            return Err(DomError::SourceOutputLimit);
        }
        let mut retained = LinkedList::new();
        let mut selected = None;
        while !queue.is_empty() {
            let tail = queue.split_off(1);
            let mut head = std::mem::replace(&mut *queue, tail);
            if let Some(slot) = head.front()
                && !matches!(*slot.lock(), SlotState::Settled)
            {
                if selected.is_none() && slot.operation == operation {
                    selected = Some(slot.clone());
                }
                retained.append(&mut head);
            }
        }
        *queue = retained;
        let Some(slot) = selected else {
            return Ok(None);
        };
        let mut state = slot.lock();
        let SlotState::Ready(_) = &*state else {
            return Ok(None);
        };
        let SlotState::Ready(batch) = std::mem::replace(&mut *state, SlotState::Loaned) else {
            return Err(DomError::InvalidSourceOperation);
        };
        Ok(Some(DomCustomElementReactions {
            slot: slot.clone(),
            batch,
        }))
    }

    pub(crate) fn segments(&self) -> usize {
        self.lock().len()
    }

    pub(crate) fn has_pending(&self) -> bool {
        self.lock()
            .iter()
            .any(|slot| !matches!(*slot.lock(), SlotState::Settled))
    }

    pub(crate) fn discard(&self) -> Result<usize, DomError> {
        let mut queue = self.lock();
        let count = queue
            .iter()
            .try_fold(0usize, |count, slot| match &*slot.lock() {
                SlotState::Loaned => Err(DomError::InvalidSourceOperation),
                SlotState::Settled => Ok(count),
                SlotState::Ready(batch) => count
                    .checked_add(batch.facts.len() - batch.cursor)
                    .ok_or(DomError::RepresentationExhausted),
            })?;
        queue.clear();
        Ok(count)
    }
}

pub(in crate::store) struct ReactionFacts {
    entries: scratch::ScratchVec<StagedReaction>,
    pub(in crate::store) registration: Arc<crate::hosted::ScriptRegistration>,
}

struct StagedReaction {
    element: Key,
    definition: crate::DomRegistryDefinitionId,
    kind: ReactionKind<Key>,
    strings: Option<custody::SharedDebit>,
}

impl ReactionFacts {
    pub(in crate::store) fn len(&self) -> usize {
        self.entries.len()
    }

    pub(in crate::store) fn truncate(&mut self, len: usize) {
        self.entries.truncate(len);
    }
}

fn retain_reaction_strings(
    kind: &ReactionKind<Key>,
    budget: &formation::Formation,
    cancel: &AtomicBool,
) -> Result<(), DomError> {
    let ReactionKind::AttributeChanged {
        local_name,
        old_value,
        new_value,
        namespace,
    } = kind
    else {
        return Ok(());
    };
    let mut payload = storage::Payload::default();
    for value in [
        Some(local_name),
        old_value.as_ref(),
        new_value.as_ref(),
        namespace.as_ref(),
    ]
    .into_iter()
    .flatten()
    {
        budget.step(cancel)?;
        payload.string(value)?;
    }
    budget.retain_payload(payload, cancel)
}

pub(in crate::store) struct Publication {
    registration: Arc<crate::hosted::ScriptRegistration>,
    batches: LinkedList<Arc<Slot>>,
}

impl Publication {
    pub(in crate::store) fn publish(mut self) {
        if let Some(inbox) = self.registration.custom_element_reactions.get() {
            inbox.lock().append(&mut self.batches);
        }
    }
}

impl Draft {
    pub(in crate::store) fn custom_element_reaction_count(&self) -> usize {
        self.custom_element_reactions
            .as_ref()
            .map_or(0, |facts| facts.entries.len())
    }

    pub(in crate::store) fn captures_custom_element_reactions(&self) -> bool {
        self.source_limits.is_some()
            && self
                .script_registration
                .as_ref()
                .is_some_and(|registration| {
                    registration.is_active()
                        && registration.custom_element_reactions.get().is_some()
                })
    }

    pub(in crate::store) fn record_custom_element_insertion(
        &mut self,
        key: Key,
        cancel: &AtomicBool,
    ) -> Result<(), DomError> {
        if !self.captures_custom_element_reactions() {
            return Ok(());
        }
        self.source_step(cancel)?;
        let Some(element) = self.node(key)?.element() else {
            return Ok(());
        };
        let element = element.language();
        if !self.shadow_connected(key, cancel)? {
            return Ok(());
        }
        if element.custom_element_state()
            == surgeist_html::elements::native::HtmlCustomElementState::Custom
            && let crate::DomElementDefinition::Definition(definition) =
                element.custom_element_definition()
        {
            return self.record_custom_element_reaction(
                key,
                definition,
                ReactionKind::Connected,
                cancel,
            );
        }
        use surgeist_html::elements::native::HtmlCustomElementState;
        // Inserting a receiver during its upgrade can put it on a nested
        // reaction queue. The upgrade itself is then a no-op, but its
        // already queued callbacks can run before the constructor returns.
        if !matches!(
            element.custom_element_state(),
            HtmlCustomElementState::Undefined | HtmlCustomElementState::Precustomized
        ) {
            return Ok(());
        }
        let Some(definition) =
            self.lookup_element_definition(key, element.custom_element_registry(), cancel)?
        else {
            return Ok(());
        };
        self.record_custom_element_reaction(key, definition, ReactionKind::Upgrade, cancel)
    }

    pub(in crate::store) fn record_custom_element_removal(
        &mut self,
        key: Key,
        old_parent: Key,
        cancel: &AtomicBool,
    ) -> Result<(), DomError> {
        if !self.captures_custom_element_reactions() {
            return Ok(());
        }
        self.source_step(cancel)?;
        let Some(element) = self.node(key)?.element().map(|element| element.language()) else {
            return Ok(());
        };
        if element.custom_element_state()
            == surgeist_html::elements::native::HtmlCustomElementState::Custom
            && let crate::DomElementDefinition::Definition(definition) =
                element.custom_element_definition()
            && self.shadow_connected(old_parent, cancel)?
        {
            self.record_custom_element_reaction(
                key,
                definition,
                ReactionKind::Disconnected,
                cancel,
            )?;
        }
        Ok(())
    }

    pub(in crate::store) fn record_custom_element_move(
        &mut self,
        key: Key,
        cancel: &AtomicBool,
    ) -> Result<(), DomError> {
        if !self.captures_custom_element_reactions() {
            return Ok(());
        }
        self.source_step(cancel)?;
        let Some(element) = self.node(key)?.element().map(|element| element.language()) else {
            return Ok(());
        };
        if element.custom_element_state()
            == surgeist_html::elements::native::HtmlCustomElementState::Custom
            && let crate::DomElementDefinition::Definition(definition) =
                element.custom_element_definition()
            && self.shadow_connected(key, cancel)?
        {
            self.record_custom_element_reaction(
                key,
                definition,
                ReactionKind::ConnectedMove,
                cancel,
            )?;
        }
        Ok(())
    }

    pub(in crate::store) fn record_custom_element_attribute(
        &mut self,
        key: Key,
        name: &crate::DomQualifiedName,
        (old_value, new_value): (Option<&DocumentString>, Option<&DocumentString>),
        cancel: &AtomicBool,
    ) -> Result<(), DomError> {
        if !self.captures_custom_element_reactions() {
            return Ok(());
        }
        self.source_step(cancel)?;
        let Some(element) = self.node(key)?.element().map(|element| element.language()) else {
            return Ok(());
        };
        if element.custom_element_state()
            != surgeist_html::elements::native::HtmlCustomElementState::Custom
        {
            return Ok(());
        }
        let crate::DomElementDefinition::Definition(definition) =
            element.custom_element_definition()
        else {
            return Ok(());
        };
        self.record_custom_element_reaction(
            key,
            definition,
            ReactionKind::AttributeChanged {
                local_name: name.local_name().clone(),
                old_value: old_value.cloned(),
                new_value: new_value.cloned(),
                namespace: name.namespace_uri().cloned(),
            },
            cancel,
        )
    }

    pub(in crate::store) fn record_custom_element_adoption(
        &mut self,
        key: Key,
        old_document: Key,
        new_document: Key,
        cancel: &AtomicBool,
    ) -> Result<(), DomError> {
        if !self.captures_custom_element_reactions() || old_document == new_document {
            return Ok(());
        }
        self.source_step(cancel)?;
        let Some(element) = self.node(key)?.element().map(|element| element.language()) else {
            return Ok(());
        };
        if element.custom_element_state()
            == surgeist_html::elements::native::HtmlCustomElementState::Custom
            && let crate::DomElementDefinition::Definition(definition) =
                element.custom_element_definition()
        {
            self.record_custom_element_reaction(
                key,
                definition,
                ReactionKind::Adopted {
                    old_document,
                    new_document,
                },
                cancel,
            )?;
        }
        Ok(())
    }

    fn record_custom_element_reaction(
        &mut self,
        key: Key,
        definition: crate::DomRegistryDefinitionId,
        kind: ReactionKind<Key>,
        cancel: &AtomicBool,
    ) -> Result<(), DomError> {
        let registration = self
            .script_registration
            .clone()
            .ok_or(DomError::HostedSourceUnavailable)?;
        let budget =
            formation::Formation::new(self, cancel)?.ok_or(DomError::InvalidSourceOperation)?;
        retain_reaction_strings(&kind, &budget, cancel)?;
        let strings = budget.fork();
        let retained = budget
            .into_debit()
            .map(|mut debit| debit.share(|next| strings.admit_custody_usage(next, cancel)))
            .transpose()?;
        let mut facts = self
            .custom_element_reactions
            .take()
            .unwrap_or_else(|| ReactionFacts {
                entries: self.effect_vec(),
                registration,
            });
        let reserve = self.select_reserve(&mut facts.entries, 1, cancel);
        if reserve.is_ok() {
            facts.entries.push(StagedReaction {
                element: key,
                definition,
                kind,
                strings: retained,
            });
        }
        self.custom_element_reactions = Some(facts);
        reserve
    }

    pub(in crate::store) fn prepare_custom_element_reactions(
        &self,
        operation: crate::DomSourceOperationId,
        ids: &AllocationIds,
        cancel: &AtomicBool,
    ) -> Result<Option<Publication>, DomError> {
        let Some(pending) = self
            .custom_element_reactions
            .as_ref()
            .filter(|facts| !facts.entries.is_empty())
        else {
            return Ok(None);
        };
        let budget =
            formation::Formation::new(self, cancel)?.ok_or(DomError::InvalidSourceOperation)?;
        let mut facts = Vec::new();
        let mut strings = Vec::new();
        formation::reserve(&mut facts, pending.entries.len(), Some(&budget), cancel)?;
        let resolve = |key| match key {
            Key::Existing(id) => Ok(id),
            Key::Created(index) => ids.get(index).ok_or(DomError::UnavailableCreation),
        };
        for entry in pending.entries.iter() {
            budget.step(cancel)?;
            let element = resolve(entry.element)?;
            if let Some(debit) = &entry.strings {
                formation::reserve(&mut strings, 1, Some(&budget), cancel)?;
                strings.push(debit.clone());
            }
            facts.push(Reaction {
                element,
                definition: entry.definition,
                kind: match &entry.kind {
                    ReactionKind::Upgrade => ReactionKind::Upgrade,
                    ReactionKind::Connected => ReactionKind::Connected,
                    ReactionKind::Disconnected => ReactionKind::Disconnected,
                    ReactionKind::ConnectedMove => ReactionKind::ConnectedMove,
                    ReactionKind::Adopted {
                        old_document,
                        new_document,
                    } => ReactionKind::Adopted {
                        old_document: crate::DocumentId(resolve(*old_document)?),
                        new_document: crate::DocumentId(resolve(*new_document)?),
                    },
                    ReactionKind::AttributeChanged {
                        local_name,
                        old_value,
                        new_value,
                        namespace,
                    } => ReactionKind::AttributeChanged {
                        local_name: local_name.clone(),
                        old_value: old_value.clone(),
                        new_value: new_value.clone(),
                        namespace: namespace.clone(),
                    },
                },
            });
        }
        let slot_backing = budget.fork();
        slot_backing.retain_bytes(size_of::<Slot>() + 4 * size_of::<usize>(), cancel)?;
        let mut batches = LinkedList::new();
        batches.push_back(formation::allocated_arc(Slot {
            operation,
            state: Mutex::new(SlotState::Ready(Batch {
                facts,
                cursor: 0,
                _backing: budget.into_debit(),
                _strings: strings,
            })),
            _backing: slot_backing.into_debit(),
        }));
        Ok(Some(Publication {
            registration: pending.registration.clone(),
            batches,
        }))
    }
}
