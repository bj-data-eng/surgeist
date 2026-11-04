//! Open-addressed indexes over caller-owned item slices, and the lookups a
//! finished capture retains.
//!
//! A table stores only item positions. It holds at most one slot per two items,
//! rounded up to a power of two with a minimum of eight, so it never has more
//! than four slots for each admitted subject once it holds more than two. Growing
//! admits the item count against the adapter's subject limit, charges every slot
//! it fills and every item it moves, and only then allocates. A lookup charges one step for each
//! slot it examines. A key's probe seed is a fixed integer mix of the parts of
//! its identity that its store assigns deterministically (slot and generation),
//! never of the owner, which a process-wide counter assigns. Two snapshots whose
//! nodes have the same slots and generations therefore charge the same work,
//! whichever store and process holds them. Slots and generations follow a
//! store's edit history, so the guarantee is per snapshot, not per content.
//!
//! A finished capture retains a [`Lookup`]: the `NodeId` table that built it
//! and a dense table from each subject's ordinal to its entry. The ordinal
//! table has one slot per entry, admitted against the adapter's subject limit
//! and charged before it allocates, so a lookup by node or by subject is a
//! constant number of steps.

use std::{convert::Infallible, num::NonZeroUsize};

use surgeist_html_semantics::HtmlCaptureSubjectId;

use super::{Control, DomHtmlCaptureError, DomHtmlCaptureResource, Entry};
use crate::{DomRegistryId, NodeId};

/// A key an index probes from a fixed seed. Equality compares the whole key;
/// the seed only chooses where a probe starts, so it may leave parts of the
/// identity out, and must be the same for keys with the same slot and
/// generation in any store.
pub(super) trait ProbeKey: Copy + Eq {
    fn seed(self) -> u64;
}

/// The finalizer of splitmix64: a fixed, well-mixed integer function.
fn finalize(mut value: u64) -> u64 {
    value = (value ^ (value >> 30)).wrapping_mul(0xbf58_476d_1ce4_b9d9_u64);
    value = (value ^ (value >> 27)).wrapping_mul(0x94d0_49bb_1331_11eb_u64);
    value ^ (value >> 31)
}

/// Slot and generation only: the owner is a process-wide counter's value.
fn seed_of(slot: usize, generation: u64) -> u64 {
    finalize(finalize(slot as u64) ^ generation)
}

impl ProbeKey for NodeId {
    fn seed(self) -> u64 {
        seed_of(self.slot, self.generation)
    }
}
impl ProbeKey for DomRegistryId {
    fn seed(self) -> u64 {
        seed_of(self.slot, self.generation)
    }
}

/// An item an index can find by its key.
pub(super) trait Keyed {
    type Key: ProbeKey;
    fn key(&self) -> Self::Key;
}
impl Keyed for Entry {
    type Key = NodeId;
    fn key(&self) -> NodeId {
        self.node
    }
}

#[derive(Debug, Default)]
pub(super) struct Table {
    /// The position plus one of the item filling each slot.
    slots: Vec<Option<NonZeroUsize>>,
}

/// How a walk of one key's chain ended.
enum Probed {
    Found(usize),
    Absent,
    /// The walk left the table or visited every slot: not a state the
    /// half-full table reaches.
    Exhausted,
}

fn start<K: ProbeKey>(key: K, mask: usize) -> usize {
    (key.seed() & mask as u64) as usize
}

impl Table {
    /// The position of the item for `key`, if indexed.
    pub(super) fn get<T: Keyed>(
        &self,
        control: &mut Control<'_>,
        items: &[T],
        key: T::Key,
    ) -> Result<Option<usize>, DomHtmlCaptureError> {
        control.charge(1)?;
        match self.probe(items, key, || control.charge(1))? {
            Probed::Found(position) => Ok(Some(position)),
            Probed::Absent => Ok(None),
            Probed::Exhausted => Err(DomHtmlCaptureError::UnmappedSubject),
        }
    }

    /// The position of the item for `key`, if indexed, for a lookup no work
    /// bound charges: the public lookups of a finished capture.
    pub(super) fn find<T: Keyed>(&self, items: &[T], key: T::Key) -> Option<usize> {
        #[cfg(test)]
        super::lookup_work::visit();
        let step = || {
            #[cfg(test)]
            super::lookup_work::visit();
            Ok::<(), Infallible>(())
        };
        match self.probe(items, key, step) {
            Ok(Probed::Found(position)) => Some(position),
            Ok(Probed::Absent | Probed::Exhausted) | Err(_) => None,
        }
    }

    /// Walks the chain of `key`, calling `step` after each slot examined that
    /// holds another key.
    fn probe<T: Keyed, E>(
        &self,
        items: &[T],
        key: T::Key,
        mut step: impl FnMut() -> Result<(), E>,
    ) -> Result<Probed, E> {
        let Some(mask) = self.slots.len().checked_sub(1) else {
            return Ok(Probed::Absent);
        };
        let mut at = start(key, mask);
        for _ in 0..self.slots.len() {
            match self.slots.get(at) {
                None => return Ok(Probed::Exhausted),
                Some(None) => return Ok(Probed::Absent),
                Some(Some(filled)) => {
                    let position = filled.get() - 1;
                    if items.get(position).is_some_and(|item| item.key() == key) {
                        return Ok(Probed::Found(position));
                    }
                }
            }
            at = (at + 1) & mask;
            step()?;
        }
        Ok(Probed::Exhausted)
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
        let key = items
            .get(position)
            .ok_or(DomHtmlCaptureError::UnmappedSubject)?
            .key();
        let filled = position
            .checked_add(1)
            .and_then(NonZeroUsize::new)
            .ok_or(DomHtmlCaptureError::RepresentationExhausted)?;
        let mask = self
            .slots
            .len()
            .checked_sub(1)
            .ok_or(DomHtmlCaptureError::UnmappedSubject)?;
        let mut at = start(key, mask);
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
    table: Table,
}
impl<'a> Subjects<'a> {
    pub(super) fn new(
        entries: &'a [Entry],
        control: &mut Control<'_>,
    ) -> Result<Self, DomHtmlCaptureError> {
        Ok(Self {
            entries,
            table: Table::over(control, entries)?,
        })
    }

    /// The subjects of a table already built over exactly `entries`.
    pub(super) fn with_table(entries: &'a [Entry], table: Table) -> Self {
        Self { entries, table }
    }

    pub(super) fn entries(&self) -> &'a [Entry] {
        self.entries
    }

    /// The subject captured for `node`; a node outside the entries is unmapped.
    pub(super) fn subject(
        &self,
        control: &mut Control<'_>,
        node: NodeId,
    ) -> Result<HtmlCaptureSubjectId, DomHtmlCaptureError> {
        self.table
            .get(control, self.entries, node)?
            .and_then(|position| self.entries.get(position))
            .map(|entry| entry.subject.clone())
            .ok_or(DomHtmlCaptureError::UnmappedSubject)
    }

    /// The node table, once nothing borrows the entries through this value.
    pub(super) fn into_table(self) -> Table {
        self.table
    }
}

/// The entries of a finished list, with the node table over them once
/// something needs it. A caller that already holds the table supplies it.
pub(super) struct LazyNodes<'a> {
    entries: &'a [Entry],
    table: Option<Table>,
}
impl<'a> LazyNodes<'a> {
    pub(super) fn new(entries: &'a [Entry], table: Option<Table>) -> Self {
        Self { entries, table }
    }

    pub(super) fn entries(&self) -> &'a [Entry] {
        self.entries
    }

    /// The entry of `node`, if any. The first lookup builds the table, charged
    /// then; every later one costs a constant number of steps.
    pub(super) fn entry(
        &mut self,
        control: &mut Control<'_>,
        node: NodeId,
    ) -> Result<Option<&'a Entry>, DomHtmlCaptureError> {
        let entries = self.entries;
        let table = match &mut self.table {
            Some(table) => table,
            None => self.table.insert(Table::over(control, entries)?),
        };
        Ok(table
            .get(control, entries, node)?
            .and_then(|position| entries.get(position)))
    }

    /// Whether a table exists: supplied, or built by a lookup.
    #[cfg(test)]
    pub(super) fn built(&self) -> bool {
        self.table.is_some()
    }

    /// The table, built now if nothing supplied or needed one.
    pub(super) fn into_table(
        self,
        control: &mut Control<'_>,
    ) -> Result<Table, DomHtmlCaptureError> {
        match self.table {
            Some(table) => Ok(table),
            None => Table::over(control, self.entries),
        }
    }
}

impl Table {
    /// A table over every item of `items`, in order.
    pub(super) fn over<T: Keyed>(
        control: &mut Control<'_>,
        items: &[T],
    ) -> Result<Self, DomHtmlCaptureError> {
        let mut table = Self::default();
        for position in 0..items.len() {
            table.insert(control, items, position)?;
        }
        Ok(table)
    }
}

/// The lookups a finished capture retains over its entries, in both
/// directions. It is owned by the capture and released with it.
#[derive(Debug)]
pub(super) struct Lookup {
    nodes: Table,
    /// The entry position plus one of the subject at each capture ordinal.
    subjects: Vec<Option<NonZeroUsize>>,
}
impl Lookup {
    /// Keeps `nodes`, which indexes `entries`, and indexes their subjects by
    /// ordinal. The ordinal table has one slot per entry; its extent is
    /// admitted against the adapter's subject limit and its fill and
    /// placements are charged before they run. An entry whose subject lies
    /// outside the table, or shares an ordinal with an earlier one, is not a
    /// finished capture's entry list.
    pub(super) fn new(
        nodes: Table,
        entries: &[Entry],
        control: &mut Control<'_>,
    ) -> Result<Self, DomHtmlCaptureError> {
        let count = entries.len();
        // Admit the extent, then charge the fill, before anything allocates.
        control.charge(1)?;
        if count > control.limits.subjects {
            return Err(DomHtmlCaptureError::ResourceLimit {
                resource: DomHtmlCaptureResource::Subjects,
                limit: control.limits.subjects,
            });
        }
        count
            .checked_mul(std::mem::size_of::<Option<NonZeroUsize>>())
            .filter(|bytes| *bytes <= isize::MAX as usize)
            .ok_or(DomHtmlCaptureError::RepresentationExhausted)?;
        control.charge(count)?;
        let mut subjects: Vec<Option<NonZeroUsize>> = Vec::new();
        subjects
            .try_reserve_exact(count)
            .map_err(|_| DomHtmlCaptureError::AllocationFailed)?;
        subjects.resize(count, None);
        for (position, entry) in entries.iter().enumerate() {
            control.charge(1)?;
            let slot = subjects
                .get_mut(entry.subject.ordinal())
                .ok_or(DomHtmlCaptureError::UnmappedSubject)?;
            if slot.is_some() {
                return Err(DomHtmlCaptureError::UnmappedSubject);
            }
            *slot = Some(
                position
                    .checked_add(1)
                    .and_then(NonZeroUsize::new)
                    .ok_or(DomHtmlCaptureError::RepresentationExhausted)?,
            );
        }
        Ok(Self { nodes, subjects })
    }

    /// The entry of `node` among `entries`, which this lookup was built over.
    pub(super) fn entry_of_node<'a>(
        &self,
        entries: &'a [Entry],
        node: NodeId,
    ) -> Option<&'a Entry> {
        self.nodes
            .find(entries, node)
            .and_then(|position| entries.get(position))
    }

    /// The entry of `subject` among `entries`, which this lookup was built
    /// over. The caller has checked that the subject belongs to this capture;
    /// an ordinal outside the table, and an entry holding another subject
    /// (an ordinal shared with a subject of another capture), find nothing.
    pub(super) fn entry_of_subject<'a>(
        &self,
        entries: &'a [Entry],
        subject: &HtmlCaptureSubjectId,
    ) -> Option<&'a Entry> {
        #[cfg(test)]
        super::lookup_work::visit();
        let position = self.subjects.get(subject.ordinal()).copied().flatten()?;
        let entry = entries.get(position.get() - 1)?;
        (entry.subject == *subject).then_some(entry)
    }
}

#[cfg(test)]
mod tests {
    use std::sync::atomic::AtomicBool;

    use super::*;
    use crate::{Dom, DomBatch, DomHtmlCaptureLimits, DomLimits, DomNameLimits, Namespace};

    struct Item(NodeId);
    impl Keyed for Item {
        type Key = NodeId;
        fn key(&self) -> NodeId {
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
        let mut table = Table::default();
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
        let mut table = Table::default();
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
        let mut table = Table::default();
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
        let mut table = Table::default();
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
        let mut table = Table::default();
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

    /// A finished capture of `count` Documents, and one entry per Document
    /// pairing it with a distinct real node.
    fn capture_entries(count: usize) -> (surgeist_html_semantics::HtmlCapture, Vec<Entry>) {
        use surgeist_html_semantics::{
            HtmlCaptureBuilder, HtmlCaptureLimits, HtmlCaptureRegistryInput, HtmlHostFactsRevision,
            HtmlScriptingStatus,
        };
        let ids = nodes(count);
        let mut builder = HtmlCaptureBuilder::new(
            HtmlCaptureLimits::new(count + 1, 8, 0, 4096, count + 1, 0, 0, 1 << 24),
            HtmlHostFactsRevision::fresh(),
        );
        let handles: Vec<_> = ids
            .iter()
            .map(|_| {
                builder
                    .add_document(
                        surgeist_html::construction::HtmlDocumentMode::NoQuirks,
                        HtmlScriptingStatus::Disabled,
                        HtmlCaptureRegistryInput::Null,
                    )
                    .unwrap()
            })
            .collect();
        let capture = builder.finish().unwrap();
        let entries = ids
            .into_iter()
            .zip(&handles)
            .map(|(node, handle)| Entry {
                node,
                subject: capture.subject_id(handle).unwrap(),
            })
            .collect();
        (capture, entries)
    }

    fn lookup(entries: &[Entry], subjects: usize) -> Lookup {
        let cancel = AtomicBool::new(false);
        let mut control = control(&cancel, subjects);
        let table = Table::over(&mut control, entries).unwrap();
        Lookup::new(table, entries, &mut control).unwrap()
    }

    #[test]
    fn each_entry_is_found_by_its_node_and_by_its_subject() {
        let (_capture, entries) = capture_entries(20);
        let lookup = lookup(&entries, 20);
        for entry in &entries {
            let by_node = lookup.entry_of_node(&entries, entry.node).unwrap();
            let by_subject = lookup.entry_of_subject(&entries, &entry.subject).unwrap();
            assert!(std::ptr::eq(by_node, entry) && std::ptr::eq(by_subject, entry));
        }
        let missing = nodes(24)[23];
        assert!(lookup.entry_of_node(&entries, missing).is_none());
    }

    /// A second capture's entries for the same nodes: every subject has the
    /// ordinal of the same node's subject in the first capture.
    fn entries_for_the_same_nodes(
        first: &[Entry],
    ) -> (surgeist_html_semantics::HtmlCapture, Vec<Entry>) {
        let (capture, mut entries) = capture_entries(first.len() + 4);
        entries.truncate(first.len());
        for (entry, original) in entries.iter_mut().zip(first) {
            entry.node = original.node;
        }
        (capture, entries)
    }

    #[test]
    fn a_subject_of_another_capture_with_an_equal_ordinal_finds_nothing() {
        let (_capture, entries) = capture_entries(20);
        let lookup = lookup(&entries, 20);
        let (_other, foreign) = entries_for_the_same_nodes(&entries);
        for (entry, other) in entries.iter().zip(&foreign) {
            assert_eq!(entry.subject.ordinal(), other.subject.ordinal());
            assert_ne!(entry.subject, other.subject);
            assert!(lookup.entry_of_subject(&entries, &other.subject).is_none());
        }
    }

    #[test]
    fn a_subject_ordinal_beyond_the_entries_finds_nothing() {
        let (_capture, entries) = capture_entries(20);
        let lookup = lookup(&entries, 20);
        // The second capture holds four more subjects than the first has entries.
        let (_other, longer) = capture_entries(24);
        for beyond in &longer[20..] {
            assert!(beyond.subject.ordinal() >= entries.len());
            assert!(lookup.entry_of_subject(&entries, &beyond.subject).is_none());
        }
    }

    #[test]
    fn the_ordinal_table_refuses_entries_that_are_not_a_finished_capture_list() {
        let (_capture, entries) = capture_entries(8);
        let cancel = AtomicBool::new(false);
        let build = |entries: &[Entry]| {
            let mut control = control(&cancel, 8);
            let table = Table::over(&mut control, entries).unwrap();
            Lookup::new(table, entries, &mut control).map(|_| ())
        };
        assert!(build(&entries).is_ok());
        // A subject listed twice shares its ordinal with itself.
        let doubled = [
            Entry {
                node: entries[0].node,
                subject: entries[0].subject.clone(),
            },
            Entry {
                node: entries[1].node,
                subject: entries[0].subject.clone(),
            },
        ];
        assert!(matches!(
            build(&doubled),
            Err(DomHtmlCaptureError::UnmappedSubject)
        ));
        // A subject whose ordinal lies beyond the listed entries.
        assert!(matches!(
            build(&entries[4..]),
            Err(DomHtmlCaptureError::UnmappedSubject)
        ));
    }

    #[test]
    fn the_ordinal_table_is_admitted_and_charged_before_it_allocates() {
        let (_capture, entries) = capture_entries(8);
        let cancel = AtomicBool::new(false);
        let table = {
            let mut control = control(&cancel, 8);
            Table::over(&mut control, &entries).unwrap()
        };
        // Over the subject limit: refused before any charge beyond the first.
        let mut small = control(&cancel, 7);
        assert!(matches!(
            Lookup::new(table, &entries, &mut small),
            Err(DomHtmlCaptureError::ResourceLimit {
                resource: DomHtmlCaptureResource::Subjects,
                limit: 7
            })
        ));
        assert_eq!(small.work, 1);
        // Exactly admitted: one step to admit, one per slot to fill, and one
        // per entry to place.
        let table = {
            let mut control = control(&cancel, 8);
            Table::over(&mut control, &entries).unwrap()
        };
        let mut exact = control(&cancel, 8);
        Lookup::new(table, &entries, &mut exact).unwrap();
        assert_eq!(exact.work, 1 + 8 + 8);
        // One step short of that fails on the work bound.
        let table = {
            let mut control = control(&cancel, 8);
            Table::over(&mut control, &entries).unwrap()
        };
        let mut short = control(&cancel, 8);
        short.limits = DomHtmlCaptureLimits::new(8, 1 + 8 + 8 - 1);
        assert!(matches!(
            Lookup::new(table, &entries, &mut short),
            Err(DomHtmlCaptureError::ResourceLimit {
                resource: DomHtmlCaptureResource::Work,
                ..
            })
        ));
    }

    #[test]
    fn duplicate_keys_resolve_to_the_first_position() {
        let ids = nodes(3);
        let items = [Item(ids[0]), Item(ids[0]), Item(ids[1])];
        let cancel = AtomicBool::new(false);
        let mut control = control(&cancel, 8);
        let table = Table::over(&mut control, &items).unwrap();
        assert_eq!(table.get(&mut control, &items, ids[0]).unwrap(), Some(0));
        assert_eq!(table.get(&mut control, &items, ids[1]).unwrap(), Some(2));
        assert_eq!(table.get(&mut control, &items, ids[2]).unwrap(), None);
    }

    #[test]
    fn lazy_nodes_build_on_the_first_lookup_only_and_supplied_tables_never_build() {
        let (_capture, entries) = capture_entries(12);
        let cancel = AtomicBool::new(false);
        let cost = |nodes: &[NodeId]| {
            let mut measure = control(&cancel, 12);
            let table = Table::over(&mut measure, &entries).unwrap();
            let build = measure.work;
            let mut lookups = Vec::new();
            for node in nodes {
                let before = measure.work;
                table.get(&mut measure, &entries, *node).unwrap();
                lookups.push(measure.work - before);
            }
            (build, lookups)
        };
        let wanted = [entries[3].node, entries[9].node];
        let (build, lookups) = cost(&wanted);

        let mut lazy = LazyNodes::new(&entries, None);
        assert!(!lazy.built());
        let mut control = control(&cancel, 12);
        assert!(lazy.entry(&mut control, wanted[0]).unwrap().is_some());
        assert!(lazy.built());
        assert_eq!(control.work, build + lookups[0]);
        let before = control.work;
        assert!(lazy.entry(&mut control, wanted[1]).unwrap().is_some());
        assert_eq!(control.work - before, lookups[1]);
        let before = control.work;
        lazy.into_table(&mut control).unwrap();
        assert_eq!(control.work, before);

        let (table, mut supplied_control) = {
            let mut c = self::control(&cancel, 12);
            (
                Table::over(&mut c, &entries).unwrap(),
                self::control(&cancel, 12),
            )
        };
        let mut supplied = LazyNodes::new(&entries, Some(table));
        assert!(supplied.built());
        supplied.entry(&mut supplied_control, wanted[0]).unwrap();
        assert_eq!(supplied_control.work, lookups[0]);
        // Without any lookup, finishing builds the table once and charges it.
        let mut idle_control = self::control(&cancel, 12);
        LazyNodes::new(&entries, None)
            .into_table(&mut idle_control)
            .unwrap();
        assert_eq!(idle_control.work, build);
    }
}
