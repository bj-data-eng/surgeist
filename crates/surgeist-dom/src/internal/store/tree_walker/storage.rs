//! Canonical TreeWalker records, custody and reversible node retirement.

use super::*;
use crate::internal::tree_walker::{DomTreeWalkerFilter, DomTreeWalkerId};

struct FilterControl {
    active: AtomicBool,
    identity: custody::MaterializationId,
}

#[derive(Clone)]
pub(super) struct Callback(Arc<FilterControl>);

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

    pub(super) fn is_active(&self) -> bool {
        self.0.active.load(Ordering::Acquire)
    }
}

/// Keeps the original admitted callback body alive without new backing.
pub(super) struct Activation {
    callback: Callback,
    _custody: bank::ObjectCustody,
}

impl Drop for Activation {
    fn drop(&mut self) {
        self.callback.0.active.store(false, Ordering::Release);
    }
}

pub(super) fn activate_tree_walker(dom: &Dom, id: DomTreeWalkerId) -> Result<Activation, DomError> {
    let cursor = dom.state.tree_walkers.cursor(dom.owner, id)?;
    let callback = cursor.callback.clone().ok_or(DomError::StaleTreeWalker)?;
    let slot = dom
        .state
        .tree_walkers
        .slots
        .controlled_record(&dom.state_custody.tree_walkers.slots, id.slot)
        .ok_or(DomError::StaleTreeWalker)?;
    let custody = slot
        .dependency::<bank::ObjectCustody>()
        .ok_or(DomError::StaleTreeWalker)?;
    callback
        .0
        .active
        .compare_exchange(false, true, Ordering::AcqRel, Ordering::Acquire)
        .map_err(|_| DomError::TreeWalkerFilterActive)?;
    Ok(Activation {
        callback,
        _custody: (*custody).clone(),
    })
}

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
struct Links {
    previous: Option<usize>,
    next: Option<usize>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(super) struct Cursor {
    pub(super) root: NodeId,
    pub(super) current: NodeId,
    pub(super) mask: u32,
    pub(super) callback: Option<Callback>,
    root_links: Links,
    current_links: Links,
}

#[derive(Clone, Copy)]
enum Relation {
    Root,
    Current,
}

impl Cursor {
    fn links(&self, relation: Relation) -> Links {
        match relation {
            Relation::Root => self.root_links,
            Relation::Current => self.current_links,
        }
    }

    fn links_mut(&mut self, relation: Relation) -> &mut Links {
        match relation {
            Relation::Root => &mut self.root_links,
            Relation::Current => &mut self.current_links,
        }
    }

    fn node(&self, relation: Relation) -> NodeId {
        match relation {
            Relation::Root => self.root,
            Relation::Current => self.current,
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
struct WalkerSlot {
    generation: u64,
    cursor: Option<Cursor>,
}

impl WalkerSlot {
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
struct Head {
    node: NodeId,
    slot: usize,
}

/// Both node indexes qualify their slot keys by the exact node generation.
#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub(in crate::internal::store) struct Walkers {
    slots: bank::Bank<WalkerSlot>,
    roots: bank::Bank<Head>,
    currents: bank::Bank<Head>,
    live: usize,
}

#[derive(Clone, Default)]
pub(in crate::internal::store) struct WalkerCustody {
    slots: bank::BankCustody<WalkerSlot>,
    roots: bank::BankCustody<Head>,
    currents: bank::BankCustody<Head>,
}

#[derive(Clone, Default)]
pub(in crate::internal::store) struct DraftWalkers {
    pub(in crate::internal::store) value: Walkers,
    pub(in crate::internal::store) custody: WalkerCustody,
    dirty: bank::Bank<bool>,
    dirty_custody: bank::BankCustody<bool>,
}

impl Walkers {
    pub(in crate::internal::store) fn retained_slots(&self) -> usize {
        self.slots.len()
    }

    pub(in crate::internal::store) fn live_slots(&self) -> usize {
        self.live
    }

    #[cfg(test)]
    pub(in crate::internal::store) fn count_live_records(&self) -> usize {
        self.slots
            .values()
            .filter(|slot| slot.cursor.is_some())
            .count()
    }

    fn head(&self, node: NodeId, relation: Relation) -> Option<usize> {
        let heads = match relation {
            Relation::Root => &self.roots,
            Relation::Current => &self.currents,
        };
        heads
            .get(node.slot)
            .filter(|head| head.node == node)
            .map(|head| head.slot)
    }

    fn slot(&self, index: usize) -> Result<&WalkerSlot, DomError> {
        self.slots.get(index).ok_or(DomError::StaleTreeWalker)
    }

    pub(super) fn cursor(
        &self,
        owner: DomOwnerId,
        id: DomTreeWalkerId,
    ) -> Result<&Cursor, DomError> {
        if id.owner != owner {
            return Err(DomError::ForeignOwner);
        }
        let slot = self.slot(id.slot)?;
        slot.cursor
            .as_ref()
            .filter(|_| slot.generation == id.generation)
            .ok_or(DomError::StaleTreeWalker)
    }

    pub(in crate::internal::store) fn adopt_custody(
        &self,
        registry: &bank::CustodyRegistry,
        admit: &mut impl FnMut(custody::Usage) -> Result<(), DomError>,
        step: &mut impl FnMut() -> Result<(), DomError>,
    ) -> Result<WalkerCustody, DomError> {
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
        let mut adopt_heads = |heads: &bank::Bank<Head>| {
            heads.adopt_controlled(
                registry,
                |_| {
                    step()?;
                    Ok(custody::Usage::new(size_of::<Head>(), 0))
                },
                &mut *admit,
            )
        };
        Ok(WalkerCustody {
            slots,
            roots: adopt_heads(&self.roots)?,
            currents: adopt_heads(&self.currents)?,
        })
    }
}

impl DraftWalkers {
    pub(in crate::internal::store) fn new(
        state: &State,
        custody: &transaction::StateCustody,
    ) -> Self {
        Self {
            value: state.tree_walkers.clone(),
            custody: custody.tree_walkers.clone(),
            ..Self::default()
        }
    }

    fn write_slot(
        &mut self,
        index: usize,
        slot: WalkerSlot,
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
                draft,
                cancel,
            )?;
        }
        Ok(())
    }

    fn write_head(
        &mut self,
        node: NodeId,
        head: Option<usize>,
        relation: Relation,
        draft: &Draft,
        cancel: &AtomicBool,
    ) -> Result<(), DomError> {
        let (heads, custody) = match relation {
            Relation::Root => (&mut self.value.roots, &mut self.custody.roots),
            Relation::Current => (&mut self.value.currents, &mut self.custody.currents),
        };
        if let Some(slot) = head {
            set(
                heads,
                custody,
                node.slot,
                Head { node, slot },
                draft,
                cancel,
            )
        } else {
            let (next, next_custody, removed) =
                heads.remove_controlled(custody, node.slot, draft.custody_registry(), |usage| {
                    draft.admit_custody_usage(usage, cancel)
                })?;
            *heads = next;
            *custody = next_custody;
            drop(removed);
            Ok(())
        }
    }

    fn link(
        &mut self,
        index: usize,
        relation: Relation,
        draft: &Draft,
        cancel: &AtomicBool,
    ) -> Result<(), DomError> {
        let mut slot = self.value.slot(index)?.clone();
        let cursor = slot.cursor.as_mut().ok_or(DomError::StaleTreeWalker)?;
        let node = cursor.node(relation);
        let next = self.value.head(node, relation);
        *cursor.links_mut(relation) = Links {
            previous: None,
            next,
        };
        if let Some(next) = next {
            let mut following = self.value.slot(next)?.clone();
            following
                .cursor
                .as_mut()
                .ok_or(DomError::StaleTreeWalker)?
                .links_mut(relation)
                .previous = Some(index);
            self.write_slot(next, following, draft, cancel)?;
        }
        self.write_head(node, Some(index), relation, draft, cancel)?;
        self.write_slot(index, slot, draft, cancel)
    }

    fn unlink(
        &mut self,
        index: usize,
        relation: Relation,
        draft: &Draft,
        cancel: &AtomicBool,
    ) -> Result<(), DomError> {
        let mut slot = self.value.slot(index)?.clone();
        let cursor = slot.cursor.as_mut().ok_or(DomError::StaleTreeWalker)?;
        let node = cursor.node(relation);
        let links = cursor.links(relation);
        *cursor.links_mut(relation) = Links::default();
        if let Some(previous) = links.previous {
            let mut preceding = self.value.slot(previous)?.clone();
            preceding
                .cursor
                .as_mut()
                .ok_or(DomError::StaleTreeWalker)?
                .links_mut(relation)
                .next = links.next;
            self.write_slot(previous, preceding, draft, cancel)?;
        } else {
            self.write_head(node, links.next, relation, draft, cancel)?;
        }
        if let Some(next) = links.next {
            let mut following = self.value.slot(next)?.clone();
            following
                .cursor
                .as_mut()
                .ok_or(DomError::StaleTreeWalker)?
                .links_mut(relation)
                .previous = links.previous;
            self.write_slot(next, following, draft, cancel)?;
        }
        self.write_slot(index, slot, draft, cancel)
    }
}

fn set<T>(
    values: &mut bank::Bank<T>,
    custody: &mut bank::BankCustody<T>,
    index: usize,
    value: T,
    draft: &Draft,
    cancel: &AtomicBool,
) -> Result<(), DomError> {
    let usage = custody::Usage::new(size_of::<T>(), 0);
    let record = bank::ControlledRecord::new(value, usage, draft.custody_registry(), |next| {
        draft.admit_custody_usage(next, cancel)
    })?;
    set_record(values, custody, index, record, false, draft, cancel)
}

fn set_slot(
    values: &mut bank::Bank<WalkerSlot>,
    custody: &mut bank::BankCustody<WalkerSlot>,
    index: usize,
    slot: WalkerSlot,
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

pub(in crate::internal::store) struct WalkerUndo(DraftWalkers);

impl WalkerUndo {
    pub(in crate::internal::store) fn restore(self, draft: &mut Draft) {
        draft.tree_walkers = self.0;
    }
}

impl Draft {
    fn edit_tree_walkers<T>(
        &mut self,
        cancel: &AtomicBool,
        edit: impl FnOnce(&mut DraftWalkers, &Self) -> Result<T, DomError>,
    ) -> Result<T, DomError> {
        let mut candidate = self.tree_walkers.clone();
        let result = edit(&mut candidate, self)?;
        let retain_undo = self.prepare_undo(cancel)?;
        let previous = std::mem::replace(&mut self.tree_walkers, candidate);
        if retain_undo {
            self.journal
                .push_prepared(transaction::Undo::TreeWalkers(WalkerUndo(previous)));
        }
        Ok(result)
    }

    pub(in crate::internal::store) fn create_tree_walker(
        &mut self,
        root: NodeId,
        mask: u32,
        filter: DomTreeWalkerFilter,
        cancel: &AtomicBool,
    ) -> Result<DomTreeWalkerId, DomError> {
        self.source_step(cancel)?;
        self.node(self.live_key(Key::Existing(root)))?;
        self.edit_tree_walkers(cancel, |walkers, draft| {
            let (index, generation) = match walkers.value.slots.lowest_vacant() {
                Some(index) => (
                    index,
                    walkers
                        .value
                        .slot(index)?
                        .generation
                        .checked_add(1)
                        .ok_or(DomError::RepresentationExhausted)?,
                ),
                None => {
                    let index = walkers.value.retained_slots();
                    let count = index
                        .checked_add(1)
                        .ok_or(DomError::RepresentationExhausted)?;
                    admit(
                        count,
                        draft.dom_limits.tree_walkers,
                        DomResource::TreeWalkers,
                    )?;
                    (index, 1)
                }
            };
            let (callback, _callback_custody) = match filter {
                DomTreeWalkerFilter::None => (None, None),
                DomTreeWalkerFilter::Callback => {
                    let identity = custody::MaterializationId::fresh()?;
                    let retained = draft.custody_registry().adopt_object(
                        identity,
                        Callback::usage(),
                        |usage| draft.admit_custody_usage(usage, cancel),
                    )?;
                    (
                        Some(Callback(Arc::new(FilterControl {
                            active: AtomicBool::new(false),
                            identity,
                        }))),
                        Some(retained),
                    )
                }
            };
            walkers.write_slot(
                index,
                WalkerSlot {
                    generation,
                    cursor: Some(Cursor {
                        root,
                        current: root,
                        mask,
                        callback,
                        root_links: Links::default(),
                        current_links: Links::default(),
                    }),
                },
                draft,
                cancel,
            )?;
            walkers.link(index, Relation::Root, draft, cancel)?;
            walkers.link(index, Relation::Current, draft, cancel)?;
            walkers.value.live = walkers
                .value
                .live
                .checked_add(1)
                .ok_or(DomError::RepresentationExhausted)?;
            Ok(DomTreeWalkerId {
                owner: draft.store,
                slot: index,
                generation,
            })
        })
    }

    pub(in crate::internal::store) fn set_tree_walker_current(
        &mut self,
        id: DomTreeWalkerId,
        node: NodeId,
        cancel: &AtomicBool,
    ) -> Result<(), DomError> {
        self.source_step(cancel)?;
        let cursor = self.tree_walkers.value.cursor(self.store, id)?;
        self.node(self.live_key(Key::Existing(node)))?;
        if cursor.current == node {
            return Ok(());
        }
        self.edit_tree_walkers(cancel, |walkers, draft| {
            walkers.unlink(id.slot, Relation::Current, draft, cancel)?;
            let mut slot = walkers.value.slot(id.slot)?.clone();
            slot.cursor
                .as_mut()
                .ok_or(DomError::StaleTreeWalker)?
                .current = node;
            walkers.write_slot(id.slot, slot, draft, cancel)?;
            walkers.link(id.slot, Relation::Current, draft, cancel)
        })
    }

    pub(in crate::internal::store) fn retire_tree_walker(
        &mut self,
        id: DomTreeWalkerId,
        cancel: &AtomicBool,
    ) -> Result<(), DomError> {
        self.source_step(cancel)?;
        self.tree_walkers.value.cursor(self.store, id)?;
        self.retire_tree_walker_slot(id.slot, cancel)
    }

    fn retire_tree_walker_slot(
        &mut self,
        index: usize,
        cancel: &AtomicBool,
    ) -> Result<(), DomError> {
        self.edit_tree_walkers(cancel, |walkers, draft| {
            walkers.unlink(index, Relation::Root, draft, cancel)?;
            walkers.unlink(index, Relation::Current, draft, cancel)?;
            let generation = walkers.value.slot(index)?.generation;
            walkers.write_slot(
                index,
                WalkerSlot {
                    generation,
                    cursor: None,
                },
                draft,
                cancel,
            )?;
            walkers.value.live = walkers
                .value
                .live
                .checked_sub(1)
                .ok_or(DomError::RepresentationExhausted)?;
            Ok(())
        })
    }

    pub(in crate::internal::store) fn retire_tree_walker_target(
        &mut self,
        node: Key,
        cancel: &AtomicBool,
    ) -> Result<(), DomError> {
        let Some(node) = self.published_id(node) else {
            return Ok(());
        };
        for relation in [Relation::Root, Relation::Current] {
            while let Some(head) = self.tree_walkers.value.head(node, relation) {
                self.source_step(cancel)?;
                self.retire_tree_walker_slot(head, cancel)?;
            }
        }
        Ok(())
    }
}

pub(in crate::internal::store) struct FinishedWalkers {
    pub(in crate::internal::store) value: Walkers,
    pub(in crate::internal::store) custody: WalkerCustody,
    pub(in crate::internal::store) state_changed: bool,
}

impl Draft {
    pub(in crate::internal::store) fn finish_tree_walkers(
        &self,
        cancel: &AtomicBool,
    ) -> Result<FinishedWalkers, DomError> {
        let mut state_changed = false;
        for (index, _) in self.tree_walkers.dirty.iter() {
            self.source_step(cancel)?;
            state_changed |=
                self.tree_walkers.value.slots.get(index) != self.base.tree_walkers.slots.get(index);
        }
        Ok(FinishedWalkers {
            value: self.tree_walkers.value.clone(),
            custody: self.tree_walkers.custody.clone(),
            state_changed,
        })
    }

    pub(in crate::internal::store) fn tree_walker_changes(
        &self,
        old: &State,
        new: &State,
        changes: &mut Vec<DomChange>,
        budget: Option<&source::formation::Formation>,
        cancel: &AtomicBool,
    ) -> Result<(), DomError> {
        for (index, _) in self.tree_walkers.dirty.iter() {
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
    let row = |walkers: &Walkers| {
        walkers.slots.get(slot).and_then(|record| {
            let cursor = record.cursor.as_ref()?;
            Some((
                DomTreeWalkerId {
                    owner,
                    slot,
                    generation: record.generation,
                },
                cursor.current,
            ))
        })
    };
    let before = row(&old.tree_walkers);
    let after = row(&new.tree_walkers);
    if let (Some((old_id, old_current)), Some((id, current))) = (before, after)
        && old_id == id
    {
        if old_current != current {
            append(DomChange::TreeWalkerCurrent(id))?;
        }
        return Ok(());
    }
    if let Some((id, _)) = before {
        append(DomChange::TreeWalkerRetired(id))?;
    }
    if let Some((id, _)) = after {
        append(DomChange::TreeWalkerCreated(id))?;
    }
    Ok(())
}

#[cfg(test)]
pub(in crate::internal::store) fn full_changes(
    owner: DomOwnerId,
    old: &State,
    new: &State,
    changes: &mut Vec<DomChange>,
    cancel: &AtomicBool,
) -> Result<(), DomError> {
    let extent = old
        .tree_walkers
        .retained_slots()
        .max(new.tree_walkers.retained_slots());
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn source_rollback_restores_watches_and_retries_the_same_unpublished_identity() {
        let cancel = AtomicBool::new(false);
        let limits = DomLimits::new(
            8,
            4096,
            64,
            0,
            0,
            16,
            1_000_000,
            crate::internal::name_limits::DomNameLimits::new(
                16_777_216, 16_384, 4_000_000, 1_048_576,
            )
            .unwrap(),
        )
        .unwrap()
        .with_tree_walkers(2);
        let mut dom = Dom::new(limits).unwrap();
        let mut setup = DomBatch::new();
        let document = setup.create_document();
        let root = setup.create_text(&document, "root");
        let current = setup.create_text(&document, "outside root");
        let commit = dom.apply(setup, &cancel).unwrap();
        let root = commit.created(&root).unwrap().unwrap();
        let current = commit.created(&current).unwrap().unwrap();
        let id = dom
            .create_tree_walker(root, u32::MAX, DomTreeWalkerFilter::None, &cancel)
            .unwrap();
        dom.set_tree_walker_current(id, current, &cancel).unwrap();
        let commands = DomBatch::new();
        let policy = crate::internal::source::DomSourceLimits::new(
            16_000_000, 100_000, 10_000_000, 128, 4096, 4096, 4,
        );
        let mut draft = Draft::new_source(
            &dom,
            commands.owner.clone(),
            &commands.commands,
            policy,
            &cancel,
        )
        .unwrap()
        .0;
        let checkpoint = draft.source_checkpoint(0, 0, &cancel).unwrap();
        let created = draft
            .create_tree_walker(root, u32::MAX, DomTreeWalkerFilter::None, &cancel)
            .unwrap();
        draft.set_tree_walker_current(id, root, &cancel).unwrap();
        draft
            .retire_tree_walker_target(Key::Existing(root), &cancel)
            .unwrap();
        assert_eq!(draft.tree_walkers.value.live_slots(), 0);
        let spent = draft.source_work.load(Ordering::Relaxed);
        cancel.store(true, Ordering::Relaxed);
        draft.source_limits.as_mut().unwrap().work = spent;
        checkpoint.restore(&mut draft);
        assert_eq!(draft.source_work.load(Ordering::Relaxed), spent);
        assert_eq!(
            draft
                .tree_walkers
                .value
                .cursor(dom.owner, id)
                .unwrap()
                .current,
            current
        );
        assert_eq!(draft.tree_walkers.value.live_slots(), 1);
        assert_eq!(
            draft.tree_walkers.value.cursor(dom.owner, created),
            Err(DomError::StaleTreeWalker)
        );
        assert_eq!(
            draft.tree_walkers.value.head(root, Relation::Root),
            Some(id.slot)
        );
        assert_eq!(
            draft.tree_walkers.value.head(current, Relation::Current),
            Some(id.slot)
        );
        cancel.store(false, Ordering::Relaxed);
        draft.source_limits.as_mut().unwrap().work = 10_000_000;
        let retry = draft
            .create_tree_walker(root, u32::MAX, DomTreeWalkerFilter::None, &cancel)
            .unwrap();
        assert_eq!(retry, created);
        draft
            .retire_tree_walker_target(Key::Existing(current), &cancel)
            .unwrap();
        assert_eq!(
            draft.tree_walkers.value.cursor(dom.owner, id),
            Err(DomError::StaleTreeWalker)
        );
        assert_eq!(
            draft
                .tree_walkers
                .value
                .cursor(dom.owner, retry)
                .unwrap()
                .current,
            root
        );
        let prepared = draft.finish(&dom, &cancel).unwrap();
        dom.publish(prepared, &cancel).unwrap();
        assert_eq!(dom.tree_walker(id), Err(DomError::StaleTreeWalker));
        assert_eq!(dom.tree_walker(retry).unwrap().current_node(), root);
    }
}
