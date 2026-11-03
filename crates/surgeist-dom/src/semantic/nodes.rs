//! Open-addressed `NodeId` indexes over caller-owned item slices.
//!
//! A table stores only item positions. It holds at most one slot per two items,
//! rounded up to a power of two, so it never needs more than four slots for
//! each admitted subject. Growing admits the exact slot count against the
//! caller's subject-limited item count, charges every slot it fills and every
//! item it moves, and only then allocates. A lookup charges one step for each
//! slot it examines. The hash is fixed, so charged work is reproducible.

use std::{
    collections::hash_map::DefaultHasher,
    hash::{Hash, Hasher},
    num::NonZeroUsize,
};

use super::{Control, DomHtmlCaptureError, Entry};
use crate::NodeId;

/// An item an index can find by its node.
pub(super) trait Keyed {
    fn node_id(&self) -> NodeId;
}
impl Keyed for Entry {
    fn node_id(&self) -> NodeId {
        self.node
    }
}

#[derive(Default)]
pub(super) struct NodeTable {
    /// The position plus one of the item filling each slot.
    slots: Vec<Option<NonZeroUsize>>,
}

fn start(node: NodeId, mask: usize) -> usize {
    let mut hasher = DefaultHasher::new();
    node.hash(&mut hasher);
    (hasher.finish() & mask as u64) as usize
}

impl NodeTable {
    /// The position of the item for `node`, if indexed.
    pub(super) fn get<T: Keyed>(
        &self,
        control: &mut Control<'_>,
        items: &[T],
        node: NodeId,
    ) -> Result<Option<usize>, DomHtmlCaptureError> {
        control.charge(1)?;
        let Some(mask) = self.slots.len().checked_sub(1) else {
            return Ok(None);
        };
        let mut at = start(node, mask);
        for _ in 0..self.slots.len() {
            match self.slots.get(at) {
                None => return Err(DomHtmlCaptureError::UnmappedSubject),
                Some(None) => return Ok(None),
                Some(Some(filled)) => {
                    let position = filled.get() - 1;
                    if items
                        .get(position)
                        .is_some_and(|item| item.node_id() == node)
                    {
                        return Ok(Some(position));
                    }
                }
            }
            at = (at + 1) & mask;
            control.charge(1)?;
        }
        Err(DomHtmlCaptureError::UnmappedSubject)
    }

    /// Indexes `items[position]`, which must not be indexed yet. The earlier
    /// items are the ones a growth moves.
    pub(super) fn insert<T: Keyed>(
        &mut self,
        control: &mut Control<'_>,
        items: &[T],
        position: usize,
    ) -> Result<(), DomHtmlCaptureError> {
        let needed = position
            .checked_add(1)
            .ok_or(DomHtmlCaptureError::RepresentationExhausted)?;
        let wanted = needed
            .checked_mul(2)
            .ok_or(DomHtmlCaptureError::RepresentationExhausted)?;
        if wanted > self.slots.len() {
            self.grow(control, items, position, wanted)?;
        }
        self.place(control, items, position)
    }

    fn grow<T: Keyed>(
        &mut self,
        control: &mut Control<'_>,
        items: &[T],
        moved: usize,
        wanted: usize,
    ) -> Result<(), DomHtmlCaptureError> {
        let target = wanted
            .checked_next_power_of_two()
            .ok_or(DomHtmlCaptureError::RepresentationExhausted)?
            .max(8);
        target
            .checked_mul(std::mem::size_of::<Option<NonZeroUsize>>())
            .filter(|bytes| *bytes <= isize::MAX as usize)
            .ok_or(DomHtmlCaptureError::RepresentationExhausted)?;
        // Filling the new slots and moving every earlier item, before allocating.
        control.charge(
            target
                .checked_add(moved)
                .ok_or(DomHtmlCaptureError::RepresentationExhausted)?,
        )?;
        let mut slots = Vec::new();
        slots
            .try_reserve_exact(target)
            .map_err(|_| DomHtmlCaptureError::AllocationFailed)?;
        slots.resize(target, None);
        self.slots = slots;
        for position in 0..moved {
            self.place(control, items, position)?;
        }
        Ok(())
    }

    /// Fills the first free slot after the start of `items[position]`.
    fn place<T: Keyed>(
        &mut self,
        control: &mut Control<'_>,
        items: &[T],
        position: usize,
    ) -> Result<(), DomHtmlCaptureError> {
        let node = items
            .get(position)
            .ok_or(DomHtmlCaptureError::UnmappedSubject)?
            .node_id();
        let filled = position
            .checked_add(1)
            .and_then(NonZeroUsize::new)
            .ok_or(DomHtmlCaptureError::RepresentationExhausted)?;
        let mask = self
            .slots
            .len()
            .checked_sub(1)
            .ok_or(DomHtmlCaptureError::UnmappedSubject)?;
        let mut at = start(node, mask);
        for _ in 0..self.slots.len() {
            control.charge(1)?;
            let slot = self
                .slots
                .get_mut(at)
                .ok_or(DomHtmlCaptureError::UnmappedSubject)?;
            if slot.is_none() {
                *slot = Some(filled);
                return Ok(());
            }
            at = (at + 1) & mask;
        }
        Err(DomHtmlCaptureError::UnmappedSubject)
    }
}

/// The captured subject of each node of a finished entry list.
pub(super) struct Subjects<'a> {
    entries: &'a [Entry],
    table: NodeTable,
}
impl<'a> Subjects<'a> {
    pub(super) fn new(
        entries: &'a [Entry],
        control: &mut Control<'_>,
    ) -> Result<Self, DomHtmlCaptureError> {
        let mut table = NodeTable::default();
        for position in 0..entries.len() {
            table.insert(control, entries, position)?;
        }
        Ok(Self { entries, table })
    }

    pub(super) fn entries(&self) -> &'a [Entry] {
        self.entries
    }

    /// The subject captured for `node`; a node outside the entries is unmapped.
    pub(super) fn subject(
        &self,
        control: &mut Control<'_>,
        node: NodeId,
    ) -> Result<surgeist_html_semantics::HtmlCaptureSubjectId, DomHtmlCaptureError> {
        self.table
            .get(control, self.entries, node)?
            .and_then(|position| self.entries.get(position))
            .map(|entry| entry.subject.clone())
            .ok_or(DomHtmlCaptureError::UnmappedSubject)
    }
}
