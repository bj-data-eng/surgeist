//! The canonical cursor bank and its ordinary-tree removal hook.

mod operation;
mod traversal;
pub use operation::DomNodeIteratorFilterRequest;

use super::*;
use crate::{DomNodeIterator, DomNodeIteratorFilter, DomNodeIteratorId};
use traversal::{OrdinaryTree, Pointer};

struct FilterControl {
    invocation: std::sync::atomic::AtomicU64,
    identity: custody::MaterializationId,
}

#[derive(Clone)]
struct Callback(Arc<FilterControl>);

impl std::fmt::Debug for Callback {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("Callback")
    }
}

impl PartialEq for Callback {
    fn eq(&self, other: &Self) -> bool {
        Arc::ptr_eq(&self.0, &other.0)
    }
}

impl Eq for Callback {}

impl Callback {
    fn usage() -> custody::Usage {
        custody::Usage::new(size_of::<FilterControl>(), 0)
    }

    fn retain(
        &self,
        registry: &bank::CustodyRegistry,
        admit: impl FnMut(custody::Usage) -> Result<(), DomError>,
    ) -> Result<bank::ObjectCustody, DomError> {
        registry.adopt_object(self.0.identity, Self::usage(), admit)
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
struct Candidate {
    invocation: u64,
    pointer: Pointer<Key>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
struct Cursor {
    root: NodeId,
    reference: Pointer<Key>,
    candidate: Option<Candidate>,
    mask: u32,
    callback: Option<Callback>,
    previous: Option<usize>,
    next: Option<usize>,
}

impl Cursor {
    fn live_candidate(&self) -> Option<Candidate> {
        self.candidate.filter(|candidate| {
            self.callback.as_ref().is_some_and(|callback| {
                callback.0.invocation.load(Ordering::Acquire) == candidate.invocation
            })
        })
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
struct IteratorSlot {
    generation: u64,
    cursor: Option<Cursor>,
}

impl IteratorSlot {
    fn describe(
        &self,
        registry: &bank::CustodyRegistry,
        admit: &mut impl FnMut(custody::Usage) -> Result<(), DomError>,
    ) -> Result<bank::RecordDescription, DomError> {
        let usage = custody::Usage::new(size_of::<Self>(), 0);
        let Some(callback) = self
            .cursor
            .as_ref()
            .and_then(|cursor| cursor.callback.as_ref())
        else {
            return Ok(usage.into());
        };
        let object = callback.retain(registry, &mut *admit)?;
        let dependency = bank::Dependency::new(object, registry.ledger(), admit)?;
        Ok(bank::RecordDescription::with_dependency(usage, dependency))
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
struct Root {
    node: NodeId,
    head: usize,
}

/// Roots are keyed by exact node slot and qualified by generation. Cursor
/// links are private slot indices and never cross from one root into another.
#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub(super) struct Iterators {
    slots: bank::Bank<IteratorSlot>,
    roots: bank::Bank<Root>,
    live: usize,
}

#[derive(Clone, Default)]
pub(super) struct IteratorCustody {
    slots: bank::BankCustody<IteratorSlot>,
    roots: bank::BankCustody<Root>,
}

#[derive(Clone, Default)]
pub(super) struct DraftIterators {
    pub(super) value: Iterators,
    pub(super) custody: IteratorCustody,
    dirty: bank::Bank<bool>,
    dirty_custody: bank::BankCustody<bool>,
}

impl Iterators {
    pub(super) fn retained_slots(&self) -> usize {
        self.slots.len()
    }
    pub(super) fn live_slots(&self) -> usize {
        self.live
    }

    #[cfg(test)]
    pub(super) fn count_live_records(&self) -> usize {
        self.slots
            .values()
            .filter(|slot| slot.cursor.is_some())
            .count()
    }

    fn head(&self, root: NodeId) -> Option<usize> {
        self.roots
            .get(root.slot)
            .filter(|row| row.node == root)
            .map(|row| row.head)
    }

    fn slot(&self, index: usize) -> Result<&IteratorSlot, DomError> {
        self.slots.get(index).ok_or(DomError::StaleNodeIterator)
    }

    fn cursor(&self, owner: DomOwnerId, id: DomNodeIteratorId) -> Result<&Cursor, DomError> {
        if id.owner != owner {
            return Err(DomError::ForeignOwner);
        }
        let slot = self.slot(id.slot)?;
        slot.cursor
            .as_ref()
            .filter(|_| slot.generation == id.generation)
            .ok_or(DomError::StaleNodeIterator)
    }

    pub(super) fn adopt_custody(
        &self,
        registry: &bank::CustodyRegistry,
        admit: &mut impl FnMut(custody::Usage) -> Result<(), DomError>,
        step: &mut impl FnMut() -> Result<(), DomError>,
    ) -> Result<IteratorCustody, DomError> {
        let slots = {
            let admit = std::cell::RefCell::new(&mut *admit);
            self.slots.adopt_controlled(
                registry,
                |slot| {
                    step()?;
                    slot.describe(registry, &mut |next| (*admit.borrow_mut())(next))
                },
                |next| (*admit.borrow_mut())(next),
            )?
        };
        Ok(IteratorCustody {
            slots,
            roots: self.roots.adopt_controlled(
                registry,
                |_| {
                    step()?;
                    Ok(custody::Usage::new(size_of::<Root>(), 0))
                },
                admit,
            )?,
        })
    }
}

impl DraftIterators {
    pub(super) fn new(state: &State, custody: &transaction::StateCustody) -> Self {
        Self {
            value: state.node_iterators.clone(),
            custody: custody.node_iterators.clone(),
            ..Self::default()
        }
    }

    fn write_slot(
        &mut self,
        index: usize,
        slot: IteratorSlot,
        draft: &Draft,
        cancel: &AtomicBool,
    ) -> Result<(), DomError> {
        set_slot(
            &mut self.value.slots,
            &mut self.custody.slots,
            index,
            slot,
            draft,
            cancel,
        )?;
        if self.dirty.get(index).is_none() {
            set(
                &mut self.dirty,
                &mut self.dirty_custody,
                index,
                true,
                false,
                draft,
                cancel,
            )?;
        }
        Ok(())
    }

    fn write_root(
        &mut self,
        root: NodeId,
        head: Option<usize>,
        draft: &Draft,
        cancel: &AtomicBool,
    ) -> Result<(), DomError> {
        if let Some(head) = head {
            set(
                &mut self.value.roots,
                &mut self.custody.roots,
                root.slot,
                Root { node: root, head },
                false,
                draft,
                cancel,
            )
        } else {
            let (roots, custody, removed) = self.value.roots.remove_controlled(
                &self.custody.roots,
                root.slot,
                draft.custody_registry(),
                |usage| draft.admit_custody_usage(usage, cancel),
            )?;
            self.value.roots = roots;
            self.custody.roots = custody;
            drop(removed);
            Ok(())
        }
    }
}

fn set<T>(
    values: &mut bank::Bank<T>,
    custody: &mut bank::BankCustody<T>,
    index: usize,
    value: T,
    reusable: bool,
    draft: &Draft,
    cancel: &AtomicBool,
) -> Result<(), DomError> {
    let usage = custody::Usage::new(size_of::<T>(), 0);
    let registry = draft.custody_registry();
    let admit = |next| draft.admit_custody_usage(next, cancel);
    let record = bank::ControlledRecord::new(value, usage, registry, admit)?;
    set_record(values, custody, index, record, reusable, draft, cancel)
}

fn set_slot(
    values: &mut bank::Bank<IteratorSlot>,
    custody: &mut bank::BankCustody<IteratorSlot>,
    index: usize,
    slot: IteratorSlot,
    draft: &Draft,
    cancel: &AtomicBool,
) -> Result<(), DomError> {
    let registry = draft.custody_registry();
    let mut admit = |next| draft.admit_custody_usage(next, cancel);
    let reusable = slot.cursor.is_none() && slot.generation < u64::MAX;
    let description = slot.describe(registry, &mut admit)?;
    let record = bank::ControlledRecord::new(slot, description, registry, admit)?;
    set_record(values, custody, index, record, reusable, draft, cancel)
}

fn set_record<T>(
    values: &mut bank::Bank<T>,
    custody: &mut bank::BankCustody<T>,
    index: usize,
    record: bank::ControlledRecord<T>,
    reusable: bool,
    draft: &Draft,
    cancel: &AtomicBool,
) -> Result<(), DomError> {
    let totals = bank::Totals {
        bytes: size_of::<T>(),
        units: 0,
        occurrences: 1,
    };
    (*values, *custody) = values.set_controlled(
        custody,
        index,
        record,
        reusable,
        totals,
        draft.custody_registry(),
        |next| draft.admit_custody_usage(next, cancel),
    )?;
    Ok(())
}

pub(super) struct IteratorUndo(DraftIterators);

impl IteratorUndo {
    pub(super) fn restore(self, draft: &mut Draft) {
        draft.node_iterators = self.0;
    }
}

impl Draft {
    fn edit_node_iterators<T>(
        &mut self,
        cancel: &AtomicBool,
        edit: impl FnOnce(&mut DraftIterators, &Self) -> Result<T, DomError>,
    ) -> Result<T, DomError> {
        let mut candidate = self.node_iterators.clone();
        let result = edit(&mut candidate, self)?;
        let retain_undo = self.prepare_undo(cancel)?;
        let previous = std::mem::replace(&mut self.node_iterators, candidate);
        if retain_undo {
            self.journal
                .push_prepared(transaction::Undo::NodeIterators(IteratorUndo(previous)));
        }
        Ok(result)
    }

    fn create_node_iterator(
        &mut self,
        root: NodeId,
        mask: u32,
        filter: DomNodeIteratorFilter,
        cancel: &AtomicBool,
    ) -> Result<DomNodeIteratorId, DomError> {
        self.source_step(cancel)?;
        self.node(self.live_key(Key::Existing(root)))?;
        self.edit_node_iterators(cancel, |iterators, draft| {
            let (index, generation) = match iterators.value.slots.lowest_vacant() {
                Some(index) => (
                    index,
                    iterators
                        .value
                        .slot(index)?
                        .generation
                        .checked_add(1)
                        .ok_or(DomError::RepresentationExhausted)?,
                ),
                None => {
                    let index = iterators.value.retained_slots();
                    let count = index
                        .checked_add(1)
                        .ok_or(DomError::RepresentationExhausted)?;
                    admit(
                        count,
                        draft.dom_limits.node_iterators,
                        DomResource::NodeIterators,
                    )?;
                    (index, 1)
                }
            };
            let (callback, _callback_custody) = match filter {
                DomNodeIteratorFilter::None => (None, None),
                DomNodeIteratorFilter::Callback => {
                    let identity = custody::MaterializationId::fresh()?;
                    let retained = draft.custody_registry().adopt_object(
                        identity,
                        Callback::usage(),
                        |usage| draft.admit_custody_usage(usage, cancel),
                    )?;
                    let callback = Callback(Arc::new(FilterControl {
                        invocation: std::sync::atomic::AtomicU64::new(0),
                        identity,
                    }));
                    (Some(callback), Some(retained))
                }
            };
            let next = iterators.value.head(root);
            if let Some(next) = next {
                let mut slot = iterators.value.slot(next)?.clone();
                slot.cursor
                    .as_mut()
                    .ok_or(DomError::StaleNodeIterator)?
                    .previous = Some(index);
                iterators.write_slot(next, slot, draft, cancel)?;
            }
            iterators.write_root(root, Some(index), draft, cancel)?;
            let cursor = Cursor {
                root,
                reference: Pointer {
                    node: Key::Existing(root),
                    before: true,
                },
                candidate: None,
                mask,
                callback,
                previous: None,
                next,
            };
            iterators.write_slot(
                index,
                IteratorSlot {
                    generation,
                    cursor: Some(cursor),
                },
                draft,
                cancel,
            )?;
            iterators.value.live = iterators
                .value
                .live
                .checked_add(1)
                .ok_or(DomError::RepresentationExhausted)?;
            Ok(DomNodeIteratorId {
                owner: draft.store,
                slot: index,
                generation,
            })
        })
    }

    fn write_node_iterator(
        &mut self,
        id: DomNodeIteratorId,
        cursor: Cursor,
        cancel: &AtomicBool,
    ) -> Result<(), DomError> {
        self.node_iterators.value.cursor(self.store, id)?;
        self.edit_node_iterators(cancel, |iterators, draft| {
            iterators.write_slot(
                id.slot,
                IteratorSlot {
                    generation: id.generation,
                    cursor: Some(cursor),
                },
                draft,
                cancel,
            )
        })
    }

    fn retire_iterator_slot(&mut self, index: usize, cancel: &AtomicBool) -> Result<(), DomError> {
        self.edit_node_iterators(cancel, |iterators, draft| {
            let slot = iterators.value.slot(index)?.clone();
            let cursor = slot.cursor.ok_or(DomError::StaleNodeIterator)?;
            if let Some(previous) = cursor.previous {
                let mut previous_slot = iterators.value.slot(previous)?.clone();
                previous_slot
                    .cursor
                    .as_mut()
                    .ok_or(DomError::StaleNodeIterator)?
                    .next = cursor.next;
                iterators.write_slot(previous, previous_slot, draft, cancel)?;
            } else {
                iterators.write_root(cursor.root, cursor.next, draft, cancel)?;
            }
            if let Some(next) = cursor.next {
                let mut next_slot = iterators.value.slot(next)?.clone();
                next_slot
                    .cursor
                    .as_mut()
                    .ok_or(DomError::StaleNodeIterator)?
                    .previous = cursor.previous;
                iterators.write_slot(next, next_slot, draft, cancel)?;
            }
            iterators.write_slot(
                index,
                IteratorSlot {
                    generation: slot.generation,
                    cursor: None,
                },
                draft,
                cancel,
            )?;
            iterators.value.live = iterators
                .value
                .live
                .checked_sub(1)
                .ok_or(DomError::RepresentationExhausted)?;
            Ok(())
        })
    }

    pub(super) fn retire_node_iterator_root(
        &mut self,
        root: Key,
        cancel: &AtomicBool,
    ) -> Result<(), DomError> {
        let Some(root) = self.published_id(root) else {
            return Ok(());
        };
        while let Some(head) = self.node_iterators.value.head(root) {
            self.source_step(cancel)?;
            self.retire_iterator_slot(head, cancel)?;
        }
        Ok(())
    }

    pub(super) fn node_iterator_pre_remove(
        &mut self,
        removed: Key,
        parent: Key,
        cancel: &AtomicBool,
    ) -> Result<(), DomError> {
        if self.node_iterators.value.live == 0 {
            return Ok(());
        }
        let removed = self.live_key(removed);
        let mut ancestor = Some(self.live_key(parent));
        while let Some(root) = ancestor {
            self.source_step(cancel)?;
            if let Some(root_id) = self.published_id(root) {
                let mut participant = self.node_iterators.value.head(root_id);
                while let Some(index) = participant {
                    self.source_step(cancel)?;
                    let slot = self.node_iterators.value.slot(index)?;
                    let mut cursor = slot.cursor.clone().ok_or(DomError::StaleNodeIterator)?;
                    let id = DomNodeIteratorId {
                        owner: self.store,
                        slot: index,
                        generation: slot.generation,
                    };
                    participant = cursor.next;
                    let tree = DraftTree {
                        draft: self,
                        cancel,
                    };
                    let reference = tree.adjust(root, removed, cursor.reference)?;
                    let candidate = cursor
                        .live_candidate()
                        .map(|mut candidate| {
                            candidate.pointer = tree.adjust(root, removed, candidate.pointer)?;
                            Ok::<_, DomError>(candidate)
                        })
                        .transpose()?;
                    if reference != cursor.reference || candidate != cursor.candidate {
                        cursor.reference = reference;
                        cursor.candidate = candidate;
                        self.write_node_iterator(id, cursor, cancel)?;
                    }
                }
            }
            ancestor = self.node(root)?.parent();
        }
        Ok(())
    }
}

pub(super) struct FinishedIterators {
    pub(super) value: Iterators,
    pub(super) custody: IteratorCustody,
    pub(super) state_changed: bool,
}

impl Draft {
    pub(super) fn finish_node_iterators(
        &self,
        identify: impl Fn(Key) -> Option<NodeId>,
        cancel: &AtomicBool,
    ) -> Result<FinishedIterators, DomError> {
        let mut value = self.node_iterators.value.clone();
        let mut custody = self.node_iterators.custody.clone();
        let mut state_changed = false;
        for (index, _) in self.node_iterators.dirty.iter() {
            self.source_step(cancel)?;
            let slot = value.slot(index)?;
            if let Some(cursor) = &slot.cursor {
                let mut mapped = cursor.clone();
                let map = |pointer: &mut Pointer<Key>| -> Result<(), DomError> {
                    pointer.node = Key::Existing(
                        identify(pointer.node)
                            .ok_or(DomError::NodeIteratorPublicationUnavailable)?,
                    );
                    Ok(())
                };
                map(&mut mapped.reference)?;
                mapped.candidate = mapped.live_candidate();
                if let Some(candidate) = &mut mapped.candidate {
                    map(&mut candidate.pointer)?;
                }
                if mapped != *cursor {
                    let slot = IteratorSlot {
                        generation: slot.generation,
                        cursor: Some(mapped),
                    };
                    set_slot(
                        &mut value.slots,
                        &mut custody.slots,
                        index,
                        slot,
                        self,
                        cancel,
                    )?;
                }
            }
            state_changed |= value.slots.get(index) != self.base.node_iterators.slots.get(index);
        }
        Ok(FinishedIterators {
            value,
            custody,
            state_changed,
        })
    }

    pub(super) fn node_iterator_changes(
        &self,
        old: &State,
        new: &State,
        changes: &mut Vec<DomChange>,
        budget: Option<&source::formation::Formation>,
        cancel: &AtomicBool,
    ) -> Result<(), DomError> {
        for (index, _) in self.node_iterators.dirty.iter() {
            self.source_step(cancel)?;
            append_changes(self.store, index, old, new, |change| {
                source::formation::reserve(changes, 1, budget, cancel)?;
                changes.push(change);
                Ok(())
            })?;
        }
        Ok(())
    }
}

fn append_changes(
    owner: DomOwnerId,
    slot: usize,
    old: &State,
    new: &State,
    mut append: impl FnMut(DomChange) -> Result<(), DomError>,
) -> Result<(), DomError> {
    let row = |iterators: &Iterators| {
        iterators.slots.get(slot).and_then(|record| {
            let cursor = record.cursor.as_ref()?;
            let id = DomNodeIteratorId {
                owner,
                slot,
                generation: record.generation,
            };
            Some((id, cursor.reference))
        })
    };
    let before = row(&old.node_iterators);
    let after = row(&new.node_iterators);
    if let (Some((old_id, old_reference)), Some((id, reference))) = (before, after)
        && old_id == id
    {
        if old_reference != reference {
            append(DomChange::NodeIteratorReference(id))?;
        }
        return Ok(());
    }
    if let Some((id, _)) = before {
        append(DomChange::NodeIteratorRetired(id))?;
    }
    if let Some((id, _)) = after {
        append(DomChange::NodeIteratorCreated(id))?;
    }
    Ok(())
}

#[cfg(test)]
pub(super) fn full_changes(
    owner: DomOwnerId,
    old: &State,
    new: &State,
    changes: &mut Vec<DomChange>,
    cancel: &AtomicBool,
) -> Result<(), DomError> {
    let extent = old
        .node_iterators
        .retained_slots()
        .max(new.node_iterators.retained_slots());
    for index in 0..extent {
        check_cancel(cancel)?;
        append_changes(owner, index, old, new, |change| {
            reserve(changes, 1)?;
            changes.push(change);
            Ok(())
        })?;
    }
    Ok(())
}

struct DraftTree<'a> {
    draft: &'a Draft,
    cancel: &'a AtomicBool,
}

impl DraftTree<'_> {
    fn adjust(
        &self,
        root: Key,
        removed: Key,
        pointer: Pointer<Key>,
    ) -> Result<Pointer<Key>, DomError> {
        let live = self.draft.live_key(pointer.node);
        let adjusted = traversal::adjust(
            self,
            root,
            removed,
            Pointer {
                node: live,
                ..pointer
            },
        )?;
        if adjusted.node == live {
            Ok(Pointer {
                node: pointer.node,
                ..adjusted
            })
        } else {
            Ok(adjusted)
        }
    }
}

impl OrdinaryTree<Key> for DraftTree<'_> {
    fn position(&self, node: Key) -> Result<Option<(Key, usize)>, DomError> {
        self.draft.source_step(self.cancel)?;
        Ok(self
            .draft
            .slot_index
            .position(node)
            .map(|position| (position.parent, position.ordinal)))
    }

    fn child(&self, node: Key, ordinal: usize) -> Result<Option<Key>, DomError> {
        self.draft.source_step(self.cancel)?;
        Ok(self.draft.node(node)?.children().get(ordinal))
    }

    fn last_child(&self, node: Key) -> Result<Option<Key>, DomError> {
        self.draft.source_step(self.cancel)?;
        let children = self.draft.node(node)?.children();
        Ok(children
            .len()
            .checked_sub(1)
            .and_then(|last| children.get(last)))
    }
}

impl Dom {
    pub fn node_iterator(&self, id: DomNodeIteratorId) -> Result<DomNodeIterator, DomError> {
        let cursor = self.state.node_iterators.cursor(self.owner, id)?;
        let Key::Existing(reference) = cursor.reference.node else {
            return Err(DomError::StaleNodeIterator);
        };
        Ok(DomNodeIterator {
            id,
            root: cursor.root,
            reference,
            before: cursor.reference.before,
            mask: cursor.mask,
            filter: if cursor.callback.is_some() {
                DomNodeIteratorFilter::Callback
            } else {
                DomNodeIteratorFilter::None
            },
        })
    }
}
