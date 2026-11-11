//! A table of attribute positions, so that a duplicate expanded name is found by probing the few
//! positions that share a hash instead of by comparing every attribute.
//!
//! The table is the shared [`HtmlNameIndex`] over slot storage that the consumer reserves and
//! accounts. It stores entries only: an entry names a stored attribute or a planned candidate and
//! keeps 32 bits of the name's hash, so that a probe reads a name only where those bits match.
//! The table never decides equality. A probe step hands the occupant back to the cursor that owns
//! it, which compares the two names through its own admitted comparison steps.

use super::QualifiedName;
use super::name_index::{
    HtmlNameIndex, HtmlNameIndexForeignProbe, HtmlNameIndexHasher, HtmlNameIndexInsertError,
    HtmlNameIndexProbe, HtmlNameIndexSlot, HtmlNameIndexSlots, HtmlNameIndexStep,
    HtmlNameIndexStorageError, HtmlNameIndexTable, HtmlNameIndexVacancy,
};

/// Slots a clearing action overwrites. Each action stays within one admission quantum.
pub(crate) const CLEAR_CHUNK: usize = 256;

/// The bits of an entry that hold the position of the attribute it names.
const POSITION_BITS: u32 = usize::BITS / 2;
const POSITION_MASK: usize = (1 << POSITION_BITS) - 1;
/// The bits of an entry that hold the part of the name's hash that tells names apart without
/// reading them. Above the position.
const TAG_MASK: usize = (1 << (usize::BITS - POSITION_BITS)) - 1;
/// The width of a tag, which fits `u32` on every target that has 64 bit positions and keeps the
/// phases of a cursor small.
const _: () = assert!(usize::BITS - POSITION_BITS <= 32);

/// The part of `hash` an entry keeps. A probe compares names only where it matches.
pub(crate) fn tag(hash: u64) -> u32 {
    (((hash >> 32) as usize) & TAG_MASK) as u32
}

/// The entry for the attribute at `position`, whose name hashes to a value with the given
/// [`tag`]. `None` when the position does not fit, which the caller treats as an index it cannot
/// use.
pub(crate) fn entry(tag: u32, position: usize) -> Option<usize> {
    // The all-ones position is left out so that no entry equals `usize::MAX`, which the shared
    // index cannot record.
    (position < POSITION_MASK).then_some((tag as usize & TAG_MASK) << POSITION_BITS | position)
}

/// The position an entry names, and its tag.
pub(crate) fn occupant(entry: usize) -> (usize, u32) {
    let position = entry & POSITION_MASK;
    let tag = ((entry >> POSITION_BITS) & TAG_MASK) as u32;
    (position, tag)
}

/// What the table currently describes.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum Binding {
    /// Nothing: the next use must rebuild the table.
    Unbound,
    /// The first `len` attributes of the element at ordinal `target` of the storage with serial
    /// `storage`, as they were at attribute revision `revision` of that storage, followed by
    /// `planned` entries for the attributes a merge plan means to append next, at positions
    /// `len` and above.
    Slots {
        storage: u64,
        target: usize,
        revision: u64,
        len: usize,
        planned: usize,
    },
}

/// A refusal of the shared index. The table is unchanged and the cursor that met it latches an
/// unusable-index failure.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct IndexFault;

impl From<HtmlNameIndexStorageError> for IndexFault {
    fn from(_: HtmlNameIndexStorageError) -> Self {
        Self
    }
}

impl From<HtmlNameIndexForeignProbe> for IndexFault {
    fn from(_: HtmlNameIndexForeignProbe) -> Self {
        Self
    }
}

impl From<HtmlNameIndexInsertError> for IndexFault {
    fn from(_: HtmlNameIndexInsertError) -> Self {
        Self
    }
}

/// Open-addressed positions of attribute names, in storage the consumer reserves and accounts.
///
/// The consumer reserves at least [`slots_required`](Self::slots_required) slots for the entries
/// it expects, adopts the allocation with [`new`](Self::new), and takes it back with
/// [`into_storage`](Self::into_storage) to release it. The table is only ever filled within the
/// capacity it was given; a cursor that finds the capacity too small falls back to comparing
/// every attribute, which stays correct but quadratic.
#[derive(Debug)]
pub struct HtmlAttributeNameIndex {
    hasher: HtmlNameIndexHasher,
    storage: Vec<HtmlNameIndexSlot>,
    index: Option<HtmlNameIndex>,
    binding: Binding,
}

impl HtmlAttributeNameIndex {
    /// Adopts `storage` as the table's slots, under the process's hash key. The table is
    /// empty until a cursor builds it, and its contents are replaced then.
    pub fn new(mut storage: Vec<HtmlNameIndexSlot>) -> Self {
        storage.clear();
        Self {
            hasher: HtmlNameIndexHasher::new(),
            storage,
            index: None,
            binding: Binding::Unbound,
        }
    }

    /// The number of slots, a power of two, that holds `entries` names at most half full. `None`
    /// when the count is not representable.
    pub fn slots_required(entries: usize) -> Option<usize> {
        let slots = HtmlNameIndexSlots::for_entries(entries);
        // A geometry that cannot hold the entries is the saturated one.
        (slots.capacity() >= entries).then_some(slots.get())
    }

    /// The slots the adopted storage can hold without reallocating.
    pub fn capacity(&self) -> usize {
        self.storage.capacity()
    }

    /// Returns the adopted storage, whatever it holds.
    pub fn into_storage(self) -> Vec<HtmlNameIndexSlot> {
        self.storage
    }

    /// The geometry for `entries`, within the adopted capacity: the largest the capacity holds,
    /// if it is at least what `entries` need. `None` when the capacity is too small, which is
    /// the caller's cue to compare every attribute instead.
    pub(crate) fn geometry_for(&self, entries: usize) -> Option<HtmlNameIndexSlots> {
        let required = Self::slots_required(entries)?;
        let capacity = self.storage.capacity();
        let held = if capacity.is_power_of_two() {
            capacity
        } else {
            capacity.checked_next_power_of_two()? >> 1
        };
        // `for_entries` of half the held slots is a geometry of exactly the held slots.
        let geometry = HtmlNameIndexSlots::for_entries(held / 2);
        (geometry.get() <= capacity && geometry.get() >= required).then_some(geometry)
    }

    pub(crate) fn binding(&self) -> Binding {
        self.binding
    }

    pub(crate) fn bind(&mut self, binding: Binding) {
        self.binding = binding;
    }

    /// Starts rebuilding with `geometry` (from [`geometry_for`](Self::geometry_for)): the table
    /// is emptied of meaning at once, under a new index, and refilled by
    /// [`clear_chunk`](Self::clear_chunk).
    pub(crate) fn begin_clear(&mut self, geometry: HtmlNameIndexSlots) {
        self.binding = Binding::Unbound;
        self.storage.clear();
        self.index = Some(HtmlNameIndex::new(geometry));
    }

    /// The slots a clearing is yet to overwrite.
    pub(crate) fn unwritten(&self) -> usize {
        self.index.as_ref().map_or(0, |index| {
            index.slots().get().saturating_sub(self.storage.len())
        })
    }

    /// Overwrites the next `count` slots with the empty value. The count never exceeds the
    /// unwritten slots, so the adopted capacity is never exceeded.
    pub(crate) fn clear_chunk(&mut self, count: usize) {
        let count = count.min(self.unwritten());
        self.storage
            .extend((0..count).map(|_| HtmlNameIndexSlot::default()));
    }

    /// The table size in slots, once cleared.
    pub(crate) fn size(&self) -> usize {
        self.index.as_ref().map_or(0, |index| index.slots().get())
    }

    /// The slots that binding the storage reads when a clearing action of `chunk` slots finishes
    /// the table: all of them, once. Zero for an action that leaves slots unwritten.
    pub(crate) fn scanned_by_clearing(&self, chunk: usize) -> usize {
        if self.index.is_some() && self.unwritten() <= chunk {
            self.size()
        } else {
            0
        }
    }

    /// Lends the storage to the index. The first lend of cleared storage checks that it is
    /// empty, which reads every slot once.
    fn table(&mut self) -> Result<HtmlNameIndexTable<'_>, IndexFault> {
        let index = self.index.as_mut().ok_or(IndexFault)?;
        Ok(HtmlNameIndexTable::new(index, &mut self.storage)?)
    }

    /// Binds the cleared storage to its index.
    pub(crate) fn bind_storage(&mut self) -> Result<(), IndexFault> {
        self.table().map(drop)
    }

    /// The hash of the exact expanded name: the namespace URI units if it has one, combined with
    /// the local name units. The prefix is excluded, as it is from name identity.
    pub(crate) fn hash(&self, name: &QualifiedName) -> u64 {
        let local = self.hasher.hash(name.local_name().code_units());
        match name.namespace().uri() {
            None => local,
            Some(uri) => (local.rotate_left(29) ^ 0x9e37_79b9_7f4a_7c15)
                .wrapping_add(self.hasher.hash(uri.code_units())),
        }
    }

    /// The hash of a name that has no namespace, from its local name units alone. It equals
    /// [`hash`](Self::hash) of the same name.
    pub(crate) fn hash_local(&self, local: &[u16]) -> u64 {
        self.hasher.hash(local)
    }

    /// The probe of a name with `hash` over the cleared table.
    pub(crate) fn probe(&mut self, hash: u64) -> Result<HtmlNameIndexProbe, IndexFault> {
        if self.unwritten() != 0 {
            return Err(IndexFault);
        }
        Ok(self.table()?.probe(hash))
    }

    /// Moves `probe` to its next slot and reports what it holds. `None` once every slot was
    /// visited.
    pub(crate) fn advance(
        &mut self,
        probe: &mut HtmlNameIndexProbe,
    ) -> Result<Option<HtmlNameIndexStep>, IndexFault> {
        Ok(self.table()?.advance(probe)?)
    }

    /// Records `entry` at the empty slot a probe found.
    pub(crate) fn insert(
        &mut self,
        vacancy: HtmlNameIndexVacancy,
        entry: usize,
    ) -> Result<(), IndexFault> {
        Ok(self.table()?.insert(vacancy, entry)?)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::internal::values::{DocumentString, Namespace};

    fn name(text: &str) -> QualifiedName {
        QualifiedName::new(Namespace::none(), None, DocumentString::from(text))
    }

    fn cleared(slots: usize) -> HtmlAttributeNameIndex {
        let mut index = HtmlAttributeNameIndex::new(Vec::with_capacity(slots));
        let geometry = index.geometry_for(slots / 2).expect("a geometry");
        index.begin_clear(geometry);
        while index.unwritten() != 0 {
            index.clear_chunk(CLEAR_CHUNK);
        }
        index.bind_storage().expect("cleared storage binds");
        index
    }

    fn vacancy_of(index: &mut HtmlAttributeNameIndex, text: &str) -> HtmlNameIndexVacancy {
        let hash = index.hash(&name(text));
        let mut probe = index.probe(hash).expect("cleared");
        loop {
            match index.advance(&mut probe).expect("own probe") {
                Some(HtmlNameIndexStep::Vacant(vacancy)) => return vacancy,
                Some(HtmlNameIndexStep::Occupied(_)) => {}
                None => panic!("a half full table has a vacancy"),
            }
        }
    }

    #[test]
    fn slots_required_is_the_power_of_two_at_least_twice_the_entries() {
        let cases = [
            (0, 2),
            (1, 2),
            (2, 4),
            (3, 8),
            (4, 8),
            (5, 16),
            (1000, 2048),
        ];
        for (entries, slots) in cases {
            assert_eq!(HtmlAttributeNameIndex::slots_required(entries), Some(slots));
        }
        assert_eq!(HtmlAttributeNameIndex::slots_required(usize::MAX), None);
        assert_eq!(
            HtmlAttributeNameIndex::slots_required(1 << (usize::BITS - 2)),
            Some(1 << (usize::BITS - 1))
        );
        assert_eq!(
            HtmlAttributeNameIndex::slots_required((1 << (usize::BITS - 2)) + 1),
            None
        );
    }

    #[test]
    fn a_table_never_outgrows_its_adopted_capacity() {
        let mut index = HtmlAttributeNameIndex::new(Vec::with_capacity(16));
        let capacity = index.capacity();
        assert!(capacity >= 16);
        let geometry = index
            .geometry_for(4)
            .expect("sixteen slots hold four names");
        assert!(geometry.get().is_power_of_two() && geometry.get() <= capacity);
        index.begin_clear(geometry);
        while index.unwritten() != 0 {
            index.clear_chunk(5);
        }
        assert_eq!(index.capacity(), capacity, "clearing does not reallocate");
        assert!(
            index.geometry_for(capacity).is_none(),
            "more entries than half the slots"
        );
    }

    #[test]
    fn a_probe_before_the_table_is_cleared_is_refused() {
        let mut index = HtmlAttributeNameIndex::new(Vec::with_capacity(8));
        assert_eq!(index.probe(0), Err(IndexFault));
        let geometry = index.geometry_for(2).expect("a geometry");
        index.begin_clear(geometry);
        assert_eq!(index.probe(0), Err(IndexFault), "still unwritten");
    }

    #[test]
    fn equal_expanded_names_hash_alike_and_prefixes_do_not_matter() {
        let index = HtmlAttributeNameIndex::new(Vec::new());
        let plain = name("x");
        let prefixed = QualifiedName::new(
            Namespace::none(),
            Some(DocumentString::from("p")),
            DocumentString::from("x"),
        );
        assert_eq!(index.hash(&plain), index.hash(&prefixed));
        assert_eq!(index.hash(&plain), index.hash_local(&[b'x'.into()]));
    }

    #[test]
    fn entries_keep_their_position_and_tag_and_never_equal_the_largest_value() {
        for hash in [0_u64, 1, u64::MAX, 0xdead_beef_cafe_f00d] {
            for position in [0, 77, POSITION_MASK - 1] {
                let made = entry(tag(hash), position).expect("fits");
                assert_ne!(made, usize::MAX);
                assert_eq!(occupant(made), (position, tag(hash)));
            }
        }
        assert_eq!(entry(tag(u64::MAX), POSITION_MASK), None);
        assert_eq!(entry(0, usize::MAX), None);
    }

    #[test]
    fn a_vacancy_kept_between_actions_fills_exactly_its_slot_once() {
        let mut index = cleared(8);
        let vacancy = vacancy_of(&mut index, "a");
        let again = vacancy_of(&mut index, "a");
        assert_eq!(index.insert(vacancy, 5), Ok(()));
        assert_eq!(
            index.insert(again, 6),
            Err(IndexFault),
            "the slot is no longer empty"
        );
        let hash = index.hash(&name("a"));
        let mut probe = index.probe(hash).expect("cleared");
        assert_eq!(
            index.advance(&mut probe),
            Ok(Some(HtmlNameIndexStep::Occupied(5)))
        );
    }

    #[test]
    fn an_insertion_past_the_entries_the_table_was_sized_for_is_refused_without_a_write() {
        let mut index = cleared(4);
        let mut stored = 0;
        for text in ["a", "b", "c", "d", "e", "f"] {
            let vacancy = vacancy_of(&mut index, text);
            if index.insert(vacancy, stored).is_ok() {
                stored += 1;
            }
        }
        assert_eq!(stored, 2, "four slots hold two entries");
        assert_eq!(
            index.storage.iter().filter(|slot| !slot.is_empty()).count(),
            2
        );
    }

    #[test]
    fn a_vacancy_of_another_table_is_refused() {
        let mut one = cleared(8);
        let mut other = cleared(8);
        let vacancy = vacancy_of(&mut other, "a");
        assert_eq!(one.insert(vacancy, 0), Err(IndexFault));
    }
}
