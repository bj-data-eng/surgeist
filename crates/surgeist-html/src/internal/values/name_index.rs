//! An insert-only index from names to entry numbers, in a caller-owned open-addressing table.
//!
//! The consumer owns the table's memory and admission; this module owns its rules, and the types
//! carry them. The slot count is a power of two. The index counts its entries and refuses one
//! beyond its capacity, which is at most half the slots, so a probe sequence always meets an empty
//! slot. A slot is opaque, so a consumer cannot forge an entry. The only way to write is to
//! insert at the [`HtmlNameIndexVacancy`] that a probe produces at the first empty slot of its
//! sequence, so a later lookup of the same name always meets the entry before any empty slot.
//! Nothing is removed. The hash key is random for each [`HtmlNameIndexHasher`], so no input can
//! choose names that collide and the expected probes per lookup are constant. The consumer
//! charges the hash of a name (one unit inspection per unit), each probe and each comparison.

use std::{
    error::Error,
    fmt,
    hash::{BuildHasher, RandomState},
    num::NonZeroUsize,
};

/// The slot count of a name index, a power of two, with the number of entries it may hold.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct HtmlNameIndexSlots {
    count: NonZeroUsize,
    capacity: usize,
}
impl HtmlNameIndexSlots {
    /// The least slot count that holds `max_entries` entries at most half full, at least two.
    /// A count beyond 2^(usize::BITS - 2) saturates at the largest power of two, which no table
    /// can reach: reserving it is the consumer's admission to refuse. The capacity is
    /// `max_entries`, or half the slots if that is fewer.
    pub fn for_entries(max_entries: usize) -> Self {
        let slots = max_entries
            .saturating_mul(2)
            .max(2)
            .checked_next_power_of_two()
            .unwrap_or(1 << (usize::BITS - 1));
        let count = NonZeroUsize::new(slots).unwrap_or(NonZeroUsize::MIN);
        Self {
            count,
            capacity: max_entries.min(count.get() / 2),
        }
    }
    /// The number of slots.
    pub fn get(self) -> usize {
        self.count.get()
    }
    /// The number of entries that the index accepts.
    pub fn capacity(self) -> usize {
        self.capacity
    }
}

/// The key that hashes names for one index. It is random for each instance.
#[derive(Clone, Debug)]
pub struct HtmlNameIndexHasher {
    state: RandomState,
}
impl Default for HtmlNameIndexHasher {
    fn default() -> Self {
        Self::new()
    }
}
impl HtmlNameIndexHasher {
    /// Creates a hasher with a random key.
    pub fn new() -> Self {
        Self {
            state: RandomState::new(),
        }
    }
    /// The keyed hash of the exact UTF-16 code units of `name`.
    pub fn hash(&self, name: &[u16]) -> u64 {
        self.state.hash_one(name)
    }
}

/// One slot of an index's storage: empty, or holding an entry that only the index can record.
/// The default is empty, so a consumer allocates storage as `HtmlNameIndexSlot::default()` values.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct HtmlNameIndexSlot(Option<NonZeroUsize>);
impl HtmlNameIndexSlot {
    /// Whether the slot holds no entry.
    pub fn is_empty(self) -> bool {
        self.0.is_none()
    }
}

/// The slots that a name visits, from its hash onward, each once, wrapping around. It keeps no
/// table and does no I/O, so a consumer that admits each probe as an action steps it between
/// actions.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct HtmlNameIndexProbe {
    slot: usize,
    mask: usize,
    remaining: usize,
}
impl HtmlNameIndexProbe {
    /// The probe sequence of a hash over a table of `slots` slots.
    pub fn new(hash: u64, slots: HtmlNameIndexSlots) -> Self {
        let mask = slots.get().saturating_sub(1);
        Self {
            slot: (hash as usize) & mask,
            mask,
            remaining: slots.get(),
        }
    }
    fn finish(&mut self) {
        self.remaining = 0;
    }
}
impl Iterator for HtmlNameIndexProbe {
    type Item = usize;
    /// The next slot, the first slot at the first call, or `None` once every slot has been
    /// visited.
    fn next(&mut self) -> Option<usize> {
        if self.remaining == 0 {
            return None;
        }
        let slot = self.slot;
        self.remaining -= 1;
        self.slot = slot.wrapping_add(1) & self.mask;
        Some(slot)
    }
}

/// The count of entries of one index, and its slot count. A consumer keeps it beside its slot
/// storage for the life of the index and lends both to a [`HtmlNameIndexTable`] for each lookup.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct HtmlNameIndex {
    slots: HtmlNameIndexSlots,
    entries: usize,
}
impl HtmlNameIndex {
    /// An index with no entries.
    pub fn new(slots: HtmlNameIndexSlots) -> Self {
        Self { slots, entries: 0 }
    }
    /// The slot count and capacity.
    pub fn slots(&self) -> HtmlNameIndexSlots {
        self.slots
    }
    /// The number of entries recorded.
    pub fn len(&self) -> usize {
        self.entries
    }
    /// Whether no entry is recorded.
    pub fn is_empty(&self) -> bool {
        self.entries == 0
    }
}

/// The slot storage does not have the index's slot count.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct HtmlNameIndexLengthError {
    expected: usize,
    actual: usize,
}
impl HtmlNameIndexLengthError {
    /// The slot count that the storage needed.
    pub fn expected(&self) -> usize {
        self.expected
    }
    /// The length of the storage that was supplied.
    pub fn actual(&self) -> usize {
        self.actual
    }
}
impl fmt::Display for HtmlNameIndexLengthError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "name index storage has {} slots but the index needs {}",
            self.actual, self.expected
        )
    }
}
impl Error for HtmlNameIndexLengthError {}

/// The first empty slot of a probe sequence, the only place a new entry may be recorded. It is
/// produced by [`HtmlNameIndexTable::advance`] and consumed by [`HtmlNameIndexTable::insert`].
#[derive(Debug, Eq, PartialEq)]
pub struct HtmlNameIndexVacancy {
    slot: usize,
    slots: usize,
}
/// What a probe found at its next slot.
#[derive(Debug, Eq, PartialEq)]
pub enum HtmlNameIndexStep {
    /// An occupied slot, with its entry number. The consumer decides whether the entry's name is
    /// the sought one and, if not, advances again.
    Occupied(usize),
    /// The first empty slot of the sequence, which ends the probe.
    Vacant(HtmlNameIndexVacancy),
}
/// The result of a lookup.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum HtmlNameIndexLookup {
    /// The earlier entry with an equal name; nothing was inserted.
    Found(usize),
    /// No entry has the name; the new entry was recorded in the first empty slot.
    Inserted,
}
/// Why an entry was not recorded. The index is unchanged.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[non_exhaustive]
pub enum HtmlNameIndexInsertError {
    /// The vacancy belongs to another index, or its slot is no longer empty.
    SlotTaken,
    /// The index already holds as many entries as its capacity.
    CapacityReached,
    /// The entry number cannot be recorded.
    EntryNumber,
}
impl fmt::Display for HtmlNameIndexInsertError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(match self {
            Self::SlotTaken => "the vacancy does not name an empty slot of this name index",
            Self::CapacityReached => "the name index holds as many entries as its capacity",
            Self::EntryNumber => "the entry number cannot be recorded in the name index",
        })
    }
}
impl Error for HtmlNameIndexInsertError {}
/// Why a lookup stopped. In every case the index is unchanged.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[non_exhaustive]
pub enum HtmlNameIndexError<E> {
    /// A charge or comparison callback refused.
    Consumer(E),
    /// Every slot of the probe sequence is occupied by another name.
    NoEmptySlot,
    /// The index already holds as many entries as its capacity.
    CapacityReached,
    /// The entry number cannot be recorded.
    EntryNumber,
    /// The first empty slot could not be written; it was not empty or not this index's.
    SlotTaken,
}
impl<E> From<HtmlNameIndexInsertError> for HtmlNameIndexError<E> {
    fn from(error: HtmlNameIndexInsertError) -> Self {
        match error {
            HtmlNameIndexInsertError::CapacityReached => Self::CapacityReached,
            HtmlNameIndexInsertError::EntryNumber => Self::EntryNumber,
            HtmlNameIndexInsertError::SlotTaken => Self::SlotTaken,
        }
    }
}
impl<E: fmt::Display> fmt::Display for HtmlNameIndexError<E> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Consumer(error) => write!(f, "name index lookup refused: {error}"),
            Self::NoEmptySlot => f.write_str("every slot of the name index probe is occupied"),
            Self::CapacityReached => HtmlNameIndexInsertError::CapacityReached.fmt(f),
            Self::EntryNumber => HtmlNameIndexInsertError::EntryNumber.fmt(f),
            Self::SlotTaken => HtmlNameIndexInsertError::SlotTaken.fmt(f),
        }
    }
}
impl<E: Error + 'static> Error for HtmlNameIndexError<E> {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::Consumer(error) => Some(error),
            _ => None,
        }
    }
}

/// An index lent its slot storage for a lookup.
#[derive(Debug)]
pub struct HtmlNameIndexTable<'a> {
    index: &'a mut HtmlNameIndex,
    storage: &'a mut [HtmlNameIndexSlot],
}
impl<'a> HtmlNameIndexTable<'a> {
    /// Lends `storage`, which must hold exactly the index's slot count.
    pub fn new(
        index: &'a mut HtmlNameIndex,
        storage: &'a mut [HtmlNameIndexSlot],
    ) -> Result<Self, HtmlNameIndexLengthError> {
        if storage.len() != index.slots.get() {
            return Err(HtmlNameIndexLengthError {
                expected: index.slots.get(),
                actual: storage.len(),
            });
        }
        Ok(Self { index, storage })
    }
    /// The probe sequence of a hash.
    pub fn probe(&self, hash: u64) -> HtmlNameIndexProbe {
        HtmlNameIndexProbe::new(hash, self.index.slots)
    }
    /// Moves `probe` to its next slot and reports it: an occupied slot with its entry number, or
    /// the first empty slot, which ends the probe. `None` once every slot was occupied.
    pub fn advance(&self, probe: &mut HtmlNameIndexProbe) -> Option<HtmlNameIndexStep> {
        let slot = probe.next()?;
        match self.storage.get(slot)?.0 {
            Some(recorded) => Some(HtmlNameIndexStep::Occupied(recorded.get() - 1)),
            None => {
                probe.finish();
                Some(HtmlNameIndexStep::Vacant(HtmlNameIndexVacancy {
                    slot,
                    slots: self.storage.len(),
                }))
            }
        }
    }
    /// Records `entry` (the number that the new name will have) at `vacancy`, the only way to
    /// write. The slot is checked to be empty again, and the entry count to be under capacity.
    pub fn insert(
        &mut self,
        vacancy: HtmlNameIndexVacancy,
        entry: usize,
    ) -> Result<(), HtmlNameIndexInsertError> {
        let recorded = entry
            .checked_add(1)
            .and_then(NonZeroUsize::new)
            .ok_or(HtmlNameIndexInsertError::EntryNumber)?;
        if vacancy.slots != self.storage.len() {
            return Err(HtmlNameIndexInsertError::SlotTaken);
        }
        if self.index.entries >= self.index.slots.capacity() {
            return Err(HtmlNameIndexInsertError::CapacityReached);
        }
        let target = self
            .storage
            .get_mut(vacancy.slot)
            .filter(|target| target.0.is_none())
            .ok_or(HtmlNameIndexInsertError::SlotTaken)?;
        target.0 = Some(recorded);
        self.index.entries += 1;
        Ok(())
    }
    /// Finds the entry whose name equals the sought one, or records `entry` (the number that the
    /// new name will have) in the first empty slot of the hash's probe sequence: a loop over
    /// [`probe`](Self::probe), [`advance`](Self::advance) and [`insert`](Self::insert).
    ///
    /// `step` is called once before each slot is examined, so the consumer charges the probe;
    /// `same_name` is called with the entry of each occupied slot and reports whether its name
    /// equals the sought one, charging its own comparison. Both receive `context` first, so they
    /// can charge one meter.
    pub fn find_or_insert<C, E>(
        &mut self,
        hash: u64,
        entry: usize,
        context: &mut C,
        mut step: impl FnMut(&mut C) -> Result<(), E>,
        mut same_name: impl FnMut(&mut C, usize) -> Result<bool, E>,
    ) -> Result<HtmlNameIndexLookup, HtmlNameIndexError<E>> {
        let mut probe = self.probe(hash);
        loop {
            step(context).map_err(HtmlNameIndexError::Consumer)?;
            match self.advance(&mut probe) {
                None => return Err(HtmlNameIndexError::NoEmptySlot),
                Some(HtmlNameIndexStep::Occupied(existing)) => {
                    if same_name(context, existing).map_err(HtmlNameIndexError::Consumer)? {
                        return Ok(HtmlNameIndexLookup::Found(existing));
                    }
                }
                Some(HtmlNameIndexStep::Vacant(vacancy)) => {
                    self.insert(vacancy, entry)?;
                    return Ok(HtmlNameIndexLookup::Inserted);
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn storage(slots: HtmlNameIndexSlots) -> Vec<HtmlNameIndexSlot> {
        vec![HtmlNameIndexSlot::default(); slots.get()]
    }
    fn lookup(
        hasher: &HtmlNameIndexHasher,
        table: &mut HtmlNameIndexTable<'_>,
        names: &[Vec<u16>],
        name: &[u16],
    ) -> Result<HtmlNameIndexLookup, HtmlNameIndexError<()>> {
        table.find_or_insert(
            hasher.hash(name),
            names.len(),
            &mut (),
            |_| Ok(()),
            |_, entry| Ok(names.get(entry).is_some_and(|n| n == name)),
        )
    }
    #[test]
    fn slots_are_powers_of_two_that_hold_the_entries_at_most_half_full_and_saturate() {
        assert_eq!(
            [0, 1, 2, 3, 4, 5, 255, 256, 257].map(|n| HtmlNameIndexSlots::for_entries(n).get()),
            [2, 2, 4, 8, 8, 16, 512, 512, 1024]
        );
        for entries in 0..300 {
            let slots = HtmlNameIndexSlots::for_entries(entries);
            assert!(slots.get().is_power_of_two() && 2 * slots.capacity() <= slots.get());
            assert_eq!(slots.capacity(), entries);
        }
        for huge in [usize::MAX / 4 + 1, usize::MAX] {
            let slots = HtmlNameIndexSlots::for_entries(huge);
            assert_eq!(slots.get(), 1 << (usize::BITS - 1));
            assert_eq!(slots.capacity(), 1 << (usize::BITS - 2));
        }
    }
    #[test]
    fn a_probe_visits_every_slot_once_starting_at_its_hash_and_then_ends() {
        for entries in [0_usize, 1, 3, 10] {
            let slots = HtmlNameIndexSlots::for_entries(entries);
            for hash in [0_u64, 1, 12345, u64::MAX] {
                let mut probe = HtmlNameIndexProbe::new(hash, slots);
                let first = probe.next().unwrap();
                assert_eq!(first, (hash as usize) & (slots.get() - 1));
                let mut visited: Vec<usize> = std::iter::once(first).chain(&mut probe).collect();
                assert_eq!(probe.next(), None);
                visited.sort_unstable();
                assert_eq!(visited, (0..slots.get()).collect::<Vec<_>>());
            }
        }
    }
    #[test]
    fn find_or_insert_matches_a_set_of_names_and_records_each_new_name_once() {
        let hasher = HtmlNameIndexHasher::new();
        let mut state = 0x9e37_79b9_7f4a_7c15_u64;
        let mut next = move |bound: u64| {
            state ^= state << 13;
            state ^= state >> 7;
            state ^= state << 17;
            state % bound
        };
        for _ in 0..200 {
            let count = next(60) as usize;
            let slots = HtmlNameIndexSlots::for_entries(count);
            let mut index = HtmlNameIndex::new(slots);
            let mut entries = storage(slots);
            let mut table = HtmlNameIndexTable::new(&mut index, &mut entries).unwrap();
            let mut names: Vec<Vec<u16>> = Vec::new();
            for _ in 0..count {
                let name: Vec<u16> = (0..1 + next(3)).map(|_| 97 + next(3) as u16).collect();
                let expected = names.iter().position(|n| *n == name);
                match (
                    expected,
                    lookup(&hasher, &mut table, &names, &name).unwrap(),
                ) {
                    (Some(entry), HtmlNameIndexLookup::Found(got)) => assert_eq!(entry, got),
                    (None, HtmlNameIndexLookup::Inserted) => names.push(name),
                    other => panic!("{other:?}"),
                }
            }
            assert_eq!(index.len(), names.len());
            assert_eq!(
                entries.iter().filter(|slot| !slot.is_empty()).count(),
                names.len()
            );
        }
    }
    #[test]
    fn wrong_storage_a_foreign_vacancy_and_the_capacity_leave_the_index_unchanged() {
        let hasher = HtmlNameIndexHasher::new();
        let slots = HtmlNameIndexSlots::for_entries(2);
        assert_eq!((slots.get(), slots.capacity()), (4, 2));
        let mut index = HtmlNameIndex::new(slots);
        let mut short = vec![HtmlNameIndexSlot::default(); 3];
        let error = HtmlNameIndexTable::new(&mut index, &mut short).unwrap_err();
        assert_eq!((error.expected(), error.actual()), (4, 3));
        // Two entries fill the capacity while two slots stay empty: a third name is refused,
        // and an existing one is still found.
        let mut entries = storage(slots);
        let mut table = HtmlNameIndexTable::new(&mut index, &mut entries).unwrap();
        let mut names = vec![];
        for name in [[97_u16], [98]] {
            assert_eq!(
                lookup(&hasher, &mut table, &names, &name),
                Ok(HtmlNameIndexLookup::Inserted)
            );
            names.push(name.to_vec());
        }
        assert_eq!(
            lookup(&hasher, &mut table, &names, &[99]),
            Err(HtmlNameIndexError::CapacityReached)
        );
        assert_eq!(
            lookup(&hasher, &mut table, &names, &[98]),
            Ok(HtmlNameIndexLookup::Found(1))
        );
        assert_eq!(index.len(), 2);
        // A vacancy from a table with another slot count is not this table's.
        let other_slots = HtmlNameIndexSlots::for_entries(8);
        let mut other_index = HtmlNameIndex::new(other_slots);
        let mut other_entries = storage(other_slots);
        let other = HtmlNameIndexTable::new(&mut other_index, &mut other_entries).unwrap();
        let mut probe = other.probe(0);
        let Some(HtmlNameIndexStep::Vacant(foreign)) = other.advance(&mut probe) else {
            panic!("an empty table is vacant at once");
        };
        let mut fresh_index = HtmlNameIndex::new(slots);
        let mut fresh = storage(slots);
        let mut table = HtmlNameIndexTable::new(&mut fresh_index, &mut fresh).unwrap();
        assert_eq!(
            table.insert(foreign, 0),
            Err(HtmlNameIndexInsertError::SlotTaken)
        );
        assert_eq!(fresh_index.len(), 0);
    }
    #[test]
    fn a_probe_ends_at_the_first_empty_slot_and_a_full_table_has_none() {
        let slots = HtmlNameIndexSlots::for_entries(1);
        let mut index = HtmlNameIndex::new(slots);
        let mut entries = storage(slots);
        let table = HtmlNameIndexTable::new(&mut index, &mut entries).unwrap();
        let mut probe = table.probe(1);
        assert!(matches!(
            table.advance(&mut probe),
            Some(HtmlNameIndexStep::Vacant(_))
        ));
        assert_eq!(table.advance(&mut probe), None);
        // Storage that is full, which a sized index never reaches, reports no empty slot.
        let mut index = HtmlNameIndex::new(slots);
        let mut full = vec![HtmlNameIndexSlot(NonZeroUsize::new(1)); slots.get()];
        let mut table = HtmlNameIndexTable::new(&mut index, &mut full).unwrap();
        assert_eq!(
            table.find_or_insert(3, 5, &mut (), |_| Ok::<_, ()>(()), |_, _| Ok(false)),
            Err(HtmlNameIndexError::NoEmptySlot)
        );
    }
    #[test]
    fn a_refused_charge_stops_the_lookup_and_an_unrepresentable_entry_is_refused() {
        let hasher = HtmlNameIndexHasher::new();
        let slots = HtmlNameIndexSlots::for_entries(1);
        let mut index = HtmlNameIndex::new(slots);
        let mut entries = storage(slots);
        let mut table = HtmlNameIndexTable::new(&mut index, &mut entries).unwrap();
        let mut calls = 0;
        let refused = table.find_or_insert(
            hasher.hash(&[97]),
            0,
            &mut calls,
            |calls| {
                *calls += 1;
                Err("stop")
            },
            |_, _| Ok(false),
        );
        assert_eq!(refused, Err(HtmlNameIndexError::Consumer("stop")));
        assert_eq!(calls, 1);
        assert_eq!(
            table.find_or_insert::<(), ()>(0, usize::MAX, &mut (), |_| Ok(()), |_, _| Ok(false)),
            Err(HtmlNameIndexError::EntryNumber)
        );
        assert_eq!(index.len(), 0);
        assert!(entries.iter().all(|slot| slot.is_empty()));
    }
    #[test]
    fn errors_describe_themselves_and_expose_the_consumer_error_as_their_source() {
        #[derive(Debug)]
        struct Refused;
        impl fmt::Display for Refused {
            fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
                f.write_str("refused")
            }
        }
        impl Error for Refused {}
        let consumer = HtmlNameIndexError::Consumer(Refused);
        assert_eq!(consumer.to_string(), "name index lookup refused: refused");
        assert!(consumer.source().unwrap().is::<Refused>());
        for error in [
            HtmlNameIndexError::<Refused>::NoEmptySlot,
            HtmlNameIndexError::CapacityReached,
            HtmlNameIndexError::EntryNumber,
            HtmlNameIndexError::SlotTaken,
        ] {
            assert!(!error.to_string().is_empty() && error.source().is_none());
        }
        assert!(
            !HtmlNameIndexLengthError {
                expected: 4,
                actual: 3
            }
            .to_string()
            .is_empty()
        );
        assert!(!HtmlNameIndexInsertError::SlotTaken.to_string().is_empty());
    }
}
