//! An insert-only index from names to entry numbers, in a caller-owned open-addressing table.
//!
//! The consumer owns the table's memory and admission; this module owns its rules, and the types
//! carry them.
//!
//! - The slot count is a power of two, and the index counts its entries against a capacity of at
//!   most half the slots, so a probe sequence always meets an empty slot.
//! - A slot is opaque, so a consumer cannot forge an entry.
//! - An index has a unique identity and is neither `Clone` nor `Copy`, so its count cannot be
//!   duplicated. It binds to the storage it is first lent, which must be empty, and refuses any
//!   other storage afterwards. That detects accidental mismatch: storage of the wrong length,
//!   populated first storage and a different buffer. It is not a custody guarantee. A consumer
//!   that clears, swaps or replaces its own slots is outside it, because the consumer owns that
//!   memory.
//! - Probes come only from the index and carry its identity, and the only way to write is to
//!   insert at the [`HtmlNameIndexVacancy`] that a probe produces at the first empty slot of its
//!   sequence, which carries the identity too. Nothing is removed, so a later lookup of a name
//!   meets its entry before any empty slot.
//!
//! The hash key is random for each [`HtmlNameIndexHasher`], so no input can choose names that
//! collide and the expected probes per lookup are constant. The consumer charges the hash of a
//! name (one unit inspection per unit), each probe and each comparison.

use std::{
    error::Error,
    fmt,
    hash::{BuildHasher, RandomState},
    num::NonZeroUsize,
    sync::atomic::{AtomicU64, Ordering},
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
/// The default is empty, so a consumer fills its storage with `HtmlNameIndexSlot::default()`.
///
/// ```compile_fail,E0423
/// use surgeist_html::bounded::values::HtmlNameIndexSlot;
/// let forged = HtmlNameIndexSlot(None);
/// ```
#[derive(Debug, Default, Eq, PartialEq)]
pub struct HtmlNameIndexSlot(Option<NonZeroUsize>);
impl HtmlNameIndexSlot {
    /// Whether the slot holds no entry.
    pub fn is_empty(&self) -> bool {
        self.0.is_none()
    }
}

/// The slots that a name visits, from its hash onward, each once, wrapping around, for one index.
/// It keeps no table and does no I/O, so a consumer that admits each probe as an action keeps it in
/// its own state between actions. Only [`HtmlNameIndexTable::probe`] makes one.
///
/// ```compile_fail,E0624
/// use surgeist_html::bounded::values::{HtmlNameIndexProbe, HtmlNameIndexSlots};
/// let forged = HtmlNameIndexProbe::new(1, 0, HtmlNameIndexSlots::for_entries(1));
/// ```
#[derive(Debug, Eq, PartialEq)]
pub struct HtmlNameIndexProbe {
    index: u64,
    slot: usize,
    mask: usize,
    remaining: usize,
}
impl HtmlNameIndexProbe {
    fn new(index: u64, hash: u64, slots: HtmlNameIndexSlots) -> Self {
        let mask = slots.get().saturating_sub(1);
        Self {
            index,
            slot: (hash as usize) & mask,
            mask,
            remaining: slots.get(),
        }
    }
    fn next_slot(&mut self) -> Option<usize> {
        if self.remaining == 0 {
            return None;
        }
        let slot = self.slot;
        self.remaining -= 1;
        self.slot = slot.wrapping_add(1) & self.mask;
        Some(slot)
    }
    fn finish(&mut self) {
        self.remaining = 0;
    }
}

static NEXT_INDEX: AtomicU64 = AtomicU64::new(1);

/// The count of entries of one index, its slot count and its identity. A consumer keeps it beside
/// its slot storage for the life of the index and lends both to a [`HtmlNameIndexTable`] for each
/// lookup. It is not `Clone` or `Copy`: a copy would count from zero over populated storage.
///
/// ```compile_fail,E0599
/// use surgeist_html::bounded::values::{HtmlNameIndex, HtmlNameIndexSlots};
/// let index = HtmlNameIndex::new(HtmlNameIndexSlots::for_entries(4));
/// let copy = index.clone();
/// ```
///
/// ```compile_fail,E0382
/// use surgeist_html::bounded::values::{HtmlNameIndex, HtmlNameIndexSlots};
/// let index = HtmlNameIndex::new(HtmlNameIndexSlots::for_entries(4));
/// let moved = index;
/// let again = index;
/// ```
#[derive(Debug, Eq, PartialEq)]
pub struct HtmlNameIndex {
    id: u64,
    slots: HtmlNameIndexSlots,
    entries: usize,
    /// The address of the storage that the index first guarded.
    bound: Option<usize>,
}
impl HtmlNameIndex {
    /// An index with no entries and a unique identity.
    pub fn new(slots: HtmlNameIndexSlots) -> Self {
        Self {
            id: NEXT_INDEX.fetch_add(1, Ordering::Relaxed),
            slots,
            entries: 0,
            bound: None,
        }
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

/// Why slot storage cannot be lent to an index.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[non_exhaustive]
pub enum HtmlNameIndexStorageError {
    /// The storage does not have the index's slot count.
    Length {
        /// The slot count that the storage needed.
        expected: usize,
        /// The length of the storage that was supplied.
        actual: usize,
    },
    /// A new index was lent storage that already holds an entry, so it would count from zero.
    Occupied {
        /// The first occupied slot.
        slot: usize,
    },
    /// The index guards other storage than the one supplied.
    OtherStorage,
}
impl fmt::Display for HtmlNameIndexStorageError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Length { expected, actual } => write!(
                f,
                "name index storage has {actual} slots but the index needs {expected}"
            ),
            Self::Occupied { slot } => {
                write!(f, "name index storage is already occupied at slot {slot}")
            }
            Self::OtherStorage => f.write_str("the name index guards other storage"),
        }
    }
}
impl Error for HtmlNameIndexStorageError {}

/// The probe was made by another index.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct HtmlNameIndexForeignProbe;
impl fmt::Display for HtmlNameIndexForeignProbe {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("the probe was made by another name index")
    }
}
impl Error for HtmlNameIndexForeignProbe {}

/// The first empty slot of a probe sequence, the only place a new entry may be recorded. It is
/// produced by [`HtmlNameIndexTable::advance`] and consumed by [`HtmlNameIndexTable::insert`], and
/// a consumer may keep it between the two, because it borrows nothing. It is not `Clone`.
///
/// ```compile_fail,E0451
/// use surgeist_html::bounded::values::HtmlNameIndexVacancy;
/// let forged = HtmlNameIndexVacancy { index: 1, slot: 0 };
/// ```
#[derive(Debug, Eq, PartialEq)]
pub struct HtmlNameIndexVacancy {
    index: u64,
    slot: usize,
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
    /// The vacancy was produced by another index.
    ForeignVacancy,
    /// The slot of the vacancy is no longer empty.
    SlotTaken,
    /// The index already holds as many entries as its capacity.
    CapacityReached,
    /// The entry number cannot be recorded.
    EntryNumber,
}
impl fmt::Display for HtmlNameIndexInsertError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(match self {
            Self::ForeignVacancy => "the vacancy was produced by another name index",
            Self::SlotTaken => "the slot of the vacancy is no longer empty",
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
    /// The first empty slot is no longer empty.
    SlotTaken,
    /// The vacancy was produced by another index.
    ForeignVacancy,
}
impl<E> From<HtmlNameIndexInsertError> for HtmlNameIndexError<E> {
    fn from(error: HtmlNameIndexInsertError) -> Self {
        match error {
            HtmlNameIndexInsertError::CapacityReached => Self::CapacityReached,
            HtmlNameIndexInsertError::EntryNumber => Self::EntryNumber,
            HtmlNameIndexInsertError::SlotTaken => Self::SlotTaken,
            HtmlNameIndexInsertError::ForeignVacancy => Self::ForeignVacancy,
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
            Self::ForeignVacancy => HtmlNameIndexInsertError::ForeignVacancy.fmt(f),
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

/// An index lent the storage it guards, for a lookup.
#[derive(Debug)]
pub struct HtmlNameIndexTable<'a> {
    index: &'a mut HtmlNameIndex,
    storage: &'a mut [HtmlNameIndexSlot],
}
impl<'a> HtmlNameIndexTable<'a> {
    /// Lends `storage`, which must hold exactly the index's slot count. The first storage that an
    /// index is lent must be empty and is the only storage that it accepts afterwards (the first
    /// lend scans it once, which is proportional to the fill that the consumer already charged).
    pub fn new(
        index: &'a mut HtmlNameIndex,
        storage: &'a mut [HtmlNameIndexSlot],
    ) -> Result<Self, HtmlNameIndexStorageError> {
        if storage.len() != index.slots.get() {
            return Err(HtmlNameIndexStorageError::Length {
                expected: index.slots.get(),
                actual: storage.len(),
            });
        }
        let address = storage.as_ptr() as usize;
        match index.bound {
            Some(bound) if bound != address => return Err(HtmlNameIndexStorageError::OtherStorage),
            Some(_) => {}
            None => {
                if let Some(slot) = storage.iter().position(|slot| !slot.is_empty()) {
                    return Err(HtmlNameIndexStorageError::Occupied { slot });
                }
                index.bound = Some(address);
            }
        }
        Ok(Self { index, storage })
    }
    /// The probe sequence of a hash, which only this index accepts.
    pub fn probe(&self, hash: u64) -> HtmlNameIndexProbe {
        HtmlNameIndexProbe::new(self.index.id, hash, self.index.slots)
    }
    /// Moves `probe` to its next slot and reports it: an occupied slot with its entry number, or
    /// the first empty slot, which ends the probe. `None` once every slot was occupied.
    pub fn advance(
        &self,
        probe: &mut HtmlNameIndexProbe,
    ) -> Result<Option<HtmlNameIndexStep>, HtmlNameIndexForeignProbe> {
        if probe.index != self.index.id {
            return Err(HtmlNameIndexForeignProbe);
        }
        let Some(slot) = probe.next_slot() else {
            return Ok(None);
        };
        let Some(content) = self.storage.get(slot) else {
            return Ok(None);
        };
        Ok(Some(match content.0 {
            Some(recorded) => HtmlNameIndexStep::Occupied(recorded.get() - 1),
            None => {
                probe.finish();
                HtmlNameIndexStep::Vacant(HtmlNameIndexVacancy {
                    index: self.index.id,
                    slot,
                })
            }
        }))
    }
    /// Records `entry` (the number that the new name will have) at `vacancy`, the only way to
    /// write. The vacancy must be this index's, its slot must still be empty, and the index must
    /// be under its capacity.
    pub fn insert(
        &mut self,
        vacancy: HtmlNameIndexVacancy,
        entry: usize,
    ) -> Result<(), HtmlNameIndexInsertError> {
        if vacancy.index != self.index.id {
            return Err(HtmlNameIndexInsertError::ForeignVacancy);
        }
        let recorded = entry
            .checked_add(1)
            .and_then(NonZeroUsize::new)
            .ok_or(HtmlNameIndexInsertError::EntryNumber)?;
        if self.index.entries >= self.index.slots.capacity() {
            return Err(HtmlNameIndexInsertError::CapacityReached);
        }
        let target = self
            .storage
            .get_mut(vacancy.slot)
            .filter(|target| target.is_empty())
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
            // The probe is this index's own, so it is never foreign.
            match self.advance(&mut probe) {
                Ok(None) | Err(HtmlNameIndexForeignProbe) => {
                    return Err(HtmlNameIndexError::NoEmptySlot);
                }
                Ok(Some(HtmlNameIndexStep::Occupied(existing))) => {
                    if same_name(context, existing).map_err(HtmlNameIndexError::Consumer)? {
                        return Ok(HtmlNameIndexLookup::Found(existing));
                    }
                }
                Ok(Some(HtmlNameIndexStep::Vacant(vacancy))) => {
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

    /// Charge totals depend on probe counts, which depend on the hash key. A process uses one
    /// key, so the same input charges the same work every time within it, and a caller that
    /// measures a parse's usage can repeat that parse under exactly those limits.
    #[test]
    fn every_hasher_in_a_process_hashes_a_name_alike() {
        let name: Vec<u16> = "data-name".encode_utf16().collect();
        let first = HtmlNameIndexHasher::new().hash(&name);
        for _ in 0..8 {
            assert_eq!(HtmlNameIndexHasher::new().hash(&name), first);
            assert_eq!(HtmlNameIndexHasher::default().hash(&name), first);
        }
        let from_another_thread =
            std::thread::spawn(move || HtmlNameIndexHasher::new().hash(&name))
                .join()
                .ok();
        assert_eq!(from_another_thread, Some(first));
    }

    fn storage(slots: HtmlNameIndexSlots) -> Vec<HtmlNameIndexSlot> {
        (0..slots.get())
            .map(|_| HtmlNameIndexSlot::default())
            .collect()
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
                let mut probe = HtmlNameIndexProbe::new(7, hash, slots);
                let first = probe.next_slot().unwrap();
                assert_eq!(first, (hash as usize) & (slots.get() - 1));
                let mut visited = vec![first];
                visited.extend(std::iter::from_fn(|| probe.next_slot()));
                assert_eq!(probe.next_slot(), None);
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
    fn a_full_table_has_no_empty_slot_to_offer_and_a_probe_ends_at_the_first_empty_one() {
        let slots = HtmlNameIndexSlots::for_entries(1);
        let mut index = HtmlNameIndex::new(slots);
        let mut entries = storage(slots);
        let table = HtmlNameIndexTable::new(&mut index, &mut entries).unwrap();
        let mut probe = table.probe(1);
        assert!(matches!(
            table.advance(&mut probe),
            Ok(Some(HtmlNameIndexStep::Vacant(_)))
        ));
        assert_eq!(table.advance(&mut probe), Ok(None));
        // Storage that is full, which a sized index never reaches, reports no empty slot.
        let mut index = HtmlNameIndex::new(slots);
        let mut full: Vec<HtmlNameIndexSlot> = (0..slots.get())
            .map(|_| HtmlNameIndexSlot(NonZeroUsize::new(1)))
            .collect();
        index.bound = Some(full.as_ptr() as usize);
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
            HtmlNameIndexError::ForeignVacancy,
        ] {
            assert!(!error.to_string().is_empty() && error.source().is_none());
        }
        for error in [
            HtmlNameIndexStorageError::Length {
                expected: 4,
                actual: 3,
            },
            HtmlNameIndexStorageError::Occupied { slot: 1 },
            HtmlNameIndexStorageError::OtherStorage,
        ] {
            assert!(!error.to_string().is_empty());
        }
        assert!(!HtmlNameIndexInsertError::SlotTaken.to_string().is_empty());
        assert!(!HtmlNameIndexForeignProbe.to_string().is_empty());
    }
}
