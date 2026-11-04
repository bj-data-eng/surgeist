//! Open-addressed `NodeId` indexes over caller-owned item slices.
//!
//! A table stores only item positions. It holds at most one slot per two items,
//! rounded up to a power of two with a minimum of eight, so it never has more
//! than four slots for each admitted subject once it holds more than two. Growing
//! admits the item count against the adapter's subject limit, charges every slot
//! it fills and every item it moves, and only then allocates. A lookup charges one step for each
//! slot it examines. The hash is fixed, so charged work is reproducible.

use std::{
    collections::hash_map::DefaultHasher,
    hash::{Hash, Hasher},
    num::NonZeroUsize,
};

use super::{Control, DomHtmlCaptureError, DomHtmlCaptureResource, Entry};
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
        let count = moved
            .checked_add(1)
            .ok_or(DomHtmlCaptureError::RepresentationExhausted)?;
        if count > control.limits.subjects {
            return Err(DomHtmlCaptureError::ResourceLimit {
                resource: DomHtmlCaptureResource::Subjects,
                limit: control.limits.subjects,
            });
        }
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

#[cfg(test)]
mod tests {
    use std::sync::atomic::AtomicBool;

    use super::*;
    use crate::{Dom, DomBatch, DomHtmlCaptureLimits, DomLimits, DomNameLimits, Namespace};

    struct Item(NodeId);
    impl Keyed for Item {
        fn node_id(&self) -> NodeId {
            self.0
        }
    }

    /// `count` distinct node identities from a real store.
    fn nodes(count: usize) -> Vec<NodeId> {
        let names = DomNameLimits::new(16_777_216, 16_384, 4_000_000, 1_048_576).unwrap();
        let limits =
            DomLimits::new(count + 8, 1 << 20, 4 * count + 64, 0, 0, 1, 65536, names).unwrap();
        let mut dom = Dom::new(limits).unwrap();
        let mut batch = DomBatch::new();
        let document = batch.create_document();
        for _ in 0..count {
            batch.create_element_ns(&document, Namespace::html(), "div");
        }
        dom.apply(batch, &AtomicBool::new(false)).unwrap();
        let ids: Vec<NodeId> = dom.snapshot().nodes().take(count).collect();
        assert_eq!(ids.len(), count);
        ids
    }

    fn control(cancellation: &AtomicBool, subjects: usize) -> Control<'_> {
        Control {
            limits: DomHtmlCaptureLimits::new(subjects, usize::MAX),
            work: 0,
            cancellation,
        }
    }

    /// Four distinct nodes that start probing at the same slot of an
    /// eight-slot table, found from the hash itself rather than assumed.
    fn colliding() -> [NodeId; 4] {
        let ids = nodes(96);
        for slot in 0..8 {
            let group: Vec<NodeId> = ids
                .iter()
                .copied()
                .filter(|node| start(*node, 7) == slot)
                .take(4)
                .collect();
            if let Ok(found) = <[NodeId; 4]>::try_from(group) {
                return found;
            }
        }
        panic!("96 nodes over 8 slots must put four in one");
    }

    #[test]
    fn colliding_keys_probe_forward_and_charge_each_slot_examined() {
        let [a, b, c, missing] = colliding();
        let items = [Item(a), Item(b), Item(c)];
        let cancel = AtomicBool::new(false);
        let mut control = control(&cancel, 8);
        let mut table = NodeTable::default();
        // The first insert fills eight slots and then examines one; the next
        // two examine the occupied slots of their chain and then a free one.
        table.insert(&mut control, &items[..1], 0).unwrap();
        assert_eq!(control.work, 8 + 1);
        table.insert(&mut control, &items[..2], 1).unwrap();
        assert_eq!(control.work, 9 + 2);
        table.insert(&mut control, &items, 2).unwrap();
        assert_eq!(control.work, 11 + 3);
        assert_eq!(table.slots.len(), 8);
        // A hit after two collisions: the lookup step, then one per mismatch.
        let before = control.work;
        assert_eq!(table.get(&mut control, &items, c).unwrap(), Some(2));
        assert_eq!(control.work - before, 1 + 2);
        assert_eq!(table.get(&mut control, &items, a).unwrap(), Some(0));
        assert_eq!(table.get(&mut control, &items, b).unwrap(), Some(1));
        // A miss that starts in the same chain walks all of it to the free slot.
        let before = control.work;
        assert_eq!(table.get(&mut control, &items, missing).unwrap(), None);
        assert_eq!(control.work - before, 1 + 3);
    }

    #[test]
    fn a_miss_in_an_empty_table_or_slot_costs_one_step() {
        let ids = nodes(8);
        let items = [Item(ids[0])];
        let cancel = AtomicBool::new(false);
        let mut control = control(&cancel, 8);
        let mut table = NodeTable::default();
        assert_eq!(table.get(&mut control, &items, ids[1]).unwrap(), None);
        assert_eq!(control.work, 1);
        table.insert(&mut control, &items, 0).unwrap();
        let before = control.work;
        let free = ids[1..]
            .iter()
            .copied()
            .find(|node| start(*node, 7) != start(ids[0], 7))
            .unwrap();
        assert_eq!(table.get(&mut control, &items, free).unwrap(), None);
        assert_eq!(control.work - before, 1);
    }

    #[test]
    fn growing_across_several_doublings_keeps_every_position_findable() {
        let ids = nodes(40);
        let items: Vec<Item> = ids.iter().copied().map(Item).collect();
        let cancel = AtomicBool::new(false);
        let mut control = control(&cancel, 40);
        let mut table = NodeTable::default();
        let mut sizes = Vec::new();
        for position in 0..items.len() {
            table
                .insert(&mut control, &items[..=position], position)
                .unwrap();
            if sizes.last() != Some(&table.slots.len()) {
                sizes.push(table.slots.len());
            }
            for (at, node) in ids[..=position].iter().enumerate() {
                assert_eq!(
                    table.get(&mut control, &items[..=position], *node).unwrap(),
                    Some(at)
                );
            }
        }
        // Eight slots hold four items; each doubling keeps load at most half.
        assert_eq!(sizes, [8, 16, 32, 64, 128]);
    }

    #[test]
    fn growth_beyond_the_subject_limit_is_refused_before_allocating() {
        let ids = nodes(8);
        let items: Vec<Item> = ids.iter().copied().map(Item).collect();
        let cancel = AtomicBool::new(false);
        let mut control = control(&cancel, 4);
        let mut table = NodeTable::default();
        for position in 0..4 {
            table
                .insert(&mut control, &items[..=position], position)
                .unwrap();
        }
        let slots = table.slots.len();
        let before = control.work;
        assert!(matches!(
            table.insert(&mut control, &items[..5], 4),
            Err(DomHtmlCaptureError::ResourceLimit {
                resource: DomHtmlCaptureResource::Subjects,
                limit: 4
            })
        ));
        assert_eq!(table.slots.len(), slots);
        assert_eq!(control.work, before);
    }

    #[test]
    fn growth_charges_its_fill_and_moved_items_and_observes_cancellation() {
        let ids = nodes(8);
        let items: Vec<Item> = ids.iter().copied().map(Item).collect();
        let cancel = AtomicBool::new(false);
        let mut control = control(&cancel, 8);
        let mut table = NodeTable::default();
        for position in 0..4 {
            table
                .insert(&mut control, &items[..=position], position)
                .unwrap();
        }
        // The fifth item doubles the table: sixteen slots filled and four moved,
        // then one placement of the four moved items each and the new item.
        let before = control.work;
        table.insert(&mut control, &items[..5], 4).unwrap();
        let charged = control.work - before;
        assert!(charged >= 16 + 4 + 5, "{charged}");
        cancel.store(true, std::sync::atomic::Ordering::Release);
        assert!(matches!(
            table.get(&mut control, &items[..5], ids[0]),
            Err(DomHtmlCaptureError::Cancelled)
        ));
    }
}
