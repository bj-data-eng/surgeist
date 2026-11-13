//! Ordered custom-element upgrade facts from committed native insertion steps.
use super::*;
use std::{collections::LinkedList, sync::Mutex};

/// One upgrade selected by an actual connected insertion step.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct DomCustomElementInsertion {
    pub element: NodeId,
    pub definition: crate::DomRegistryDefinitionId,
}

/// One published segment's insertion facts and their retained backing.
pub struct DomCustomElementInsertions {
    slot: Arc<Slot>,
    batch: Batch,
}

#[derive(Default)]
struct Batch {
    facts: Vec<DomCustomElementInsertion>,
    cursor: usize,
    _backing: Option<custody::Debit>,
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

impl DomCustomElementInsertions {
    pub fn operation(&self) -> crate::DomSourceOperationId {
        self.slot.operation
    }

    pub fn facts(&self) -> &[DomCustomElementInsertion] {
        &self.batch.facts[self.batch.cursor..]
    }

    /// Mark the front occurrence settled after its exact native/guest path.
    pub fn acknowledge_front(&mut self) {
        if self.batch.cursor < self.batch.facts.len() {
            self.batch.cursor += 1;
        }
    }
}

impl std::fmt::Debug for DomCustomElementInsertions {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("DomCustomElementInsertions")
            .field("operation", &self.operation())
            .field("facts", &self.facts())
            .finish_non_exhaustive()
    }
}

impl Drop for DomCustomElementInsertions {
    fn drop(&mut self) {
        let mut slot = self.slot.lock();
        *slot = if self.facts().is_empty() {
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
    ) -> Result<Option<DomCustomElementInsertions>, DomError> {
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
        Ok(Some(DomCustomElementInsertions {
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

pub(in crate::store) struct InsertionFacts {
    pub(in crate::store) entries: scratch::ScratchVec<(Key, crate::DomRegistryDefinitionId)>,
    pub(in crate::store) registration: Arc<crate::hosted::ScriptRegistration>,
}

pub(in crate::store) struct Publication {
    registration: Arc<crate::hosted::ScriptRegistration>,
    batches: LinkedList<Arc<Slot>>,
}

impl Publication {
    pub(in crate::store) fn publish(mut self) {
        if let Some(inbox) = self.registration.custom_element_insertions.get() {
            inbox.lock().append(&mut self.batches);
        }
    }
}

impl Draft {
    pub(in crate::store) fn custom_element_insertion_count(&self) -> usize {
        self.custom_element_insertions
            .as_ref()
            .map_or(0, |facts| facts.entries.len())
    }

    pub(in crate::store) fn captures_custom_element_insertions(&self) -> bool {
        self.source_limits.is_some()
            && self
                .script_registration
                .as_ref()
                .is_some_and(|registration| {
                    registration.is_active()
                        && registration.custom_element_insertions.get().is_some()
                })
    }

    pub(in crate::store) fn record_custom_element_insertion(
        &mut self,
        key: Key,
        cancel: &AtomicBool,
    ) -> Result<(), DomError> {
        let Some(registration) = self.script_registration.as_ref().filter(|registration| {
            registration.is_active() && registration.custom_element_insertions.get().is_some()
        }) else {
            return Ok(());
        };
        if self.source_limits.is_none() {
            return Ok(());
        }
        self.source_step(cancel)?;
        let Some(element) = self.node(key)?.element() else {
            return Ok(());
        };
        let element = element.language();
        if element.custom_element_state()
            != surgeist_html::elements::native::HtmlCustomElementState::Undefined
            || !self.shadow_connected(key, cancel)?
        {
            return Ok(());
        }
        let Some(definition) =
            self.lookup_element_definition(key, element.custom_element_registry(), cancel)?
        else {
            return Ok(());
        };
        let registration = registration.clone();
        let mut facts = self
            .custom_element_insertions
            .take()
            .unwrap_or_else(|| InsertionFacts {
                entries: self.effect_vec(),
                registration,
            });
        let reserve = self.select_reserve(&mut facts.entries, 1, cancel);
        if reserve.is_ok() {
            facts.entries.push((key, definition));
        }
        self.custom_element_insertions = Some(facts);
        reserve
    }

    pub(in crate::store) fn prepare_custom_element_insertions(
        &self,
        operation: crate::DomSourceOperationId,
        ids: &AllocationIds,
        cancel: &AtomicBool,
    ) -> Result<Option<Publication>, DomError> {
        let Some(pending) = self
            .custom_element_insertions
            .as_ref()
            .filter(|facts| !facts.entries.is_empty())
        else {
            return Ok(None);
        };
        let budget =
            formation::Formation::new(self, cancel)?.ok_or(DomError::InvalidSourceOperation)?;
        let mut facts = Vec::new();
        formation::reserve(&mut facts, pending.entries.len(), Some(&budget), cancel)?;
        for &(key, definition) in pending.entries.iter() {
            budget.step(cancel)?;
            let element = match key {
                Key::Existing(id) => id,
                Key::Created(index) => ids.get(index).ok_or(DomError::UnavailableCreation)?,
            };
            facts.push(DomCustomElementInsertion {
                element,
                definition,
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
            })),
            _backing: slot_backing.into_debit(),
        }));
        Ok(Some(Publication {
            registration: pending.registration.clone(),
            batches,
        }))
    }
}
