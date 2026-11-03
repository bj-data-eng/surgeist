//! A keyed open-addressed table of attribute positions, so that a duplicate expanded name is
//! found by probing the few positions that share a hash instead of by comparing every attribute.
//!
//! The table stores positions only. It never decides equality: a probe hands the occupant's
//! position back to the cursor that owns it, which compares the two names through its own
//! admitted comparison steps. The table is insert-only between rebuilds, at most half full, and
//! keyed by a random state per instance, so no input can choose names that collide.

use super::QualifiedName;
use std::{
    collections::hash_map::RandomState,
    error::Error,
    fmt,
    hash::BuildHasher,
    num::NonZeroUsize,
    sync::atomic::{AtomicU64, Ordering},
};

/// The identities issued to indexes: a vacancy is valid only for the index that found it.
static NEXT_IDENTITY: AtomicU64 = AtomicU64::new(1);

/// One slot of an open-addressed name table. A slot is empty or holds one entry, and nothing but
/// the table can put an entry in one, so a consumer reserves and accounts `Vec`s of them and
/// leaves their contents to the table.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct HtmlNameIndexSlot(Option<NonZeroUsize>);

/// The geometry of an open-addressed name table: a power-of-two slot count and the number of
/// entries that fit in it at most half full.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct HtmlNameIndexSlots {
    slots: NonZeroUsize,
    entries: usize,
}

/// The most slots a table has: a probe keeps its slot numbers in 32 bits so that a cursor's phase
/// stays small. A table of that size is 16 GiB of slots, which no consumer admits.
const MAX_SLOTS: usize = 1 << 31;

impl HtmlNameIndexSlots {
    /// The geometry for at most `max_entries` entries. A count too large to double saturates at
    /// the largest table, which no allocation can hold, so the consumer's reservation is the one
    /// to refuse.
    pub(crate) fn for_entries(max_entries: usize) -> Self {
        let slots = max_entries
            .max(1)
            .checked_mul(2)
            .and_then(usize::checked_next_power_of_two)
            .map_or(MAX_SLOTS, |slots| slots.min(MAX_SLOTS));
        let slots = NonZeroUsize::new(slots).unwrap_or(NonZeroUsize::MIN);
        Self {
            slots,
            entries: max_entries.min(slots.get() / 2),
        }
    }

    /// The largest geometry that fits `capacity` slots, if any.
    pub(crate) fn within(capacity: usize) -> Option<Self> {
        let capacity = capacity.min(MAX_SLOTS);
        let held = if capacity.is_power_of_two() {
            capacity
        } else {
            capacity.checked_next_power_of_two()? >> 1
        };
        NonZeroUsize::new(held).map(|slots| Self {
            slots,
            entries: held / 2,
        })
    }

    pub(crate) fn slots(self) -> usize {
        self.slots.get()
    }

    pub(crate) fn entries(self) -> usize {
        self.entries
    }
}

/// A per-instance random key, so that no input can choose names that collide.
#[derive(Clone, Debug)]
pub(crate) struct HtmlNameIndexHasher {
    state: RandomState,
}

impl HtmlNameIndexHasher {
    pub(crate) fn new() -> Self {
        Self {
            state: RandomState::new(),
        }
    }

    /// The hash of `units`.
    pub(crate) fn hash(&self, units: &[u16]) -> u64 {
        self.state.hash_one(units)
    }
}

/// The slots a name visits in a table, from the one its hash selects onward, wrapping around
/// once. It keeps no table and does no I/O. Only an index makes one, positioned at its first slot.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct HtmlNameIndexProbe {
    slot: u32,
    mask: u32,
    remaining: u32,
}

impl HtmlNameIndexProbe {
    fn new(hash: u64, slots: HtmlNameIndexSlots) -> Self {
        // The geometry never exceeds `MAX_SLOTS`, so these conversions hold.
        let mask = u32::try_from(slots.slots() - 1).unwrap_or(u32::MAX);
        Self {
            slot: (hash as u32) & mask,
            mask,
            remaining: mask.saturating_add(1),
        }
    }

    /// The following slot, or `None`, leaving the probe unchanged, once every slot has been
    /// visited.
    pub(crate) fn next_slot(&mut self) -> Option<usize> {
        if self.remaining <= 1 {
            return None;
        }
        self.remaining -= 1;
        self.slot = self.slot.wrapping_add(1) & self.mask;
        Some(self.slot as usize)
    }

    /// The slot the probe is at.
    pub(crate) fn slot(&self) -> usize {
        self.slot as usize
    }
}

/// The first empty slot of a probe sequence, with the identity of the index that found it. Only a
/// table step makes one, and an insertion is the only way to write a slot. It borrows nothing, so
/// a cursor may keep it across its actions, and it is spent by the insertion.
#[derive(Debug, Eq, PartialEq)]
pub(crate) struct HtmlNameIndexVacancy {
    identity: u64,
    slot: usize,
}

/// What a table step found in the probe's slot.
#[derive(Debug, Eq, PartialEq)]
pub(crate) enum HtmlNameIndexStep {
    /// The slot holds this entry.
    Occupied(usize),
    /// The slot is empty: no entry the probe has not already visited can have the name.
    Vacant(HtmlNameIndexVacancy),
}

/// Why a table step or insertion was refused. The table is unchanged.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[non_exhaustive]
pub(crate) enum HtmlNameIndexError {
    /// The slot is outside the table, which is not cleared to its full size.
    Slot,
    /// The table already holds as many entries as it was sized for.
    Full,
    /// The vacancy is no longer empty.
    Taken,
    /// The vacancy was found by another index.
    Foreign,
    /// The entry cannot be stored.
    Entry,
}

impl fmt::Display for HtmlNameIndexError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(match self {
            Self::Slot => "name index slot is outside the table",
            Self::Full => "name index holds as many entries as it was sized for",
            Self::Taken => "name index slot is no longer empty",
            Self::Foreign => "name index vacancy belongs to another index",
            Self::Entry => "name index cannot store the entry",
        })
    }
}

impl Error for HtmlNameIndexError {}

/// Set in an entry that names a planned candidate, not a stored attribute.
const CANDIDATE: usize = 1 << (usize::BITS - 1);
/// The bits of an entry that hold the position of the attribute or candidate it names.
const POSITION_BITS: u32 = usize::BITS / 2 - 1;
const POSITION_MASK: usize = (1 << POSITION_BITS) - 1;
/// The bits of an entry that hold the part of the name's hash that tells names apart without
/// reading them. Between the position and the candidate marker.
const TAG_MASK: usize = (1 << (usize::BITS - 1 - POSITION_BITS)) - 1;
/// The width of a tag, which fits `u32` on every target that has 64 bit positions and keeps the
/// phases of a cursor small.
const _: () = assert!(usize::BITS - 1 - POSITION_BITS <= 32);

/// What an entry names.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum Occupant {
    /// The stored attribute at this position.
    Attribute(usize),
    /// The planned candidate with this index.
    Candidate(usize),
}

/// The part of `hash` an entry keeps. A probe compares names only where it matches.
pub(crate) fn tag(hash: u64) -> u32 {
    (((hash >> 32) as usize) & TAG_MASK) as u32
}

/// The entry for `occupant`, whose name hashes to a value with the given [`tag`]. `None` when
/// the position does not fit, which the caller treats as an index it cannot use.
pub(crate) fn entry(tag: u32, occupant: Occupant) -> Option<usize> {
    let (position, marker) = match occupant {
        Occupant::Attribute(position) => (position, 0),
        Occupant::Candidate(position) => (position, CANDIDATE),
    };
    // The all-ones position is left out so that no entry equals `usize::MAX`, which a slot
    // cannot hold.
    (position < POSITION_MASK)
        .then_some(marker | (tag as usize & TAG_MASK) << POSITION_BITS | position)
}

/// What a non-empty entry names, and its tag.
pub(crate) fn occupant(entry: usize) -> (Occupant, u32) {
    let position = entry & POSITION_MASK;
    let tag = ((entry >> POSITION_BITS) & TAG_MASK) as u32;
    if entry & CANDIDATE != 0 {
        (Occupant::Candidate(position), tag)
    } else {
        (Occupant::Attribute(position), tag)
    }
}
/// Slots a clearing action overwrites. Each action stays within one admission quantum.
pub(crate) const CLEAR_CHUNK: usize = 256;

/// What the table currently describes.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum Binding {
    /// Nothing: the next use must rebuild the table.
    Unbound,
    /// The first `len` attributes of the element at ordinal `target`, backed by the allocation at
    /// address `backing`.
    Slots {
        target: usize,
        backing: usize,
        len: usize,
    },
    /// The attributes of the element at ordinal `target` and the candidates planned so far.
    Merge { target: usize },
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
    identity: u64,
    hasher: HtmlNameIndexHasher,
    slots: Vec<HtmlNameIndexSlot>,
    geometry: Option<HtmlNameIndexSlots>,
    count: usize,
    binding: Binding,
}

impl HtmlAttributeNameIndex {
    /// Adopts `storage` as the table's slots, with a random key for this instance. The table is
    /// empty until a cursor builds it, and its contents are replaced then.
    pub fn new(mut storage: Vec<HtmlNameIndexSlot>) -> Self {
        storage.clear();
        Self {
            identity: NEXT_IDENTITY.fetch_add(1, Ordering::Relaxed),
            hasher: HtmlNameIndexHasher::new(),
            slots: storage,
            geometry: None,
            count: 0,
            binding: Binding::Unbound,
        }
    }

    /// The number of slots, a power of two, that holds `entries` names at most half full. `None`
    /// when the count is not representable.
    pub fn slots_required(entries: usize) -> Option<usize> {
        let geometry = HtmlNameIndexSlots::for_entries(entries);
        // A geometry that cannot hold the entries is the saturated one.
        (geometry.entries() >= entries).then_some(geometry.slots())
    }

    /// The slots the adopted storage can hold without reallocating.
    pub fn capacity(&self) -> usize {
        self.slots.capacity()
    }

    /// Returns the adopted storage, whatever it holds.
    pub fn into_storage(self) -> Vec<HtmlNameIndexSlot> {
        self.slots
    }

    /// The geometry for `entries`, within the adopted capacity: the largest the capacity holds,
    /// if it is at least what `entries` need. `None` when the capacity is too small, which is
    /// the caller's cue to compare every attribute instead.
    pub(crate) fn geometry_for(&self, entries: usize) -> Option<HtmlNameIndexSlots> {
        let required = Self::slots_required(entries)?;
        let held = HtmlNameIndexSlots::within(self.slots.capacity())?;
        (held.slots() >= required).then_some(held)
    }

    pub(crate) fn binding(&self) -> Binding {
        self.binding
    }

    pub(crate) fn bind(&mut self, binding: Binding) {
        self.binding = binding;
    }

    /// Starts rebuilding with `geometry` (from [`geometry_for`](Self::geometry_for)): the table
    /// is emptied of meaning at once and refilled by [`clear_chunk`](Self::clear_chunk).
    pub(crate) fn begin_clear(&mut self, geometry: HtmlNameIndexSlots) {
        self.binding = Binding::Unbound;
        self.slots.clear();
        self.count = 0;
        self.geometry = Some(geometry);
    }

    /// The slots a clearing is yet to overwrite.
    pub(crate) fn unwritten(&self) -> usize {
        self.geometry.map_or(0, |geometry| {
            geometry.slots().saturating_sub(self.slots.len())
        })
    }

    /// Overwrites the next `count` slots with the empty value. The count never exceeds the
    /// unwritten slots, so the adopted capacity is never exceeded.
    pub(crate) fn clear_chunk(&mut self, count: usize) {
        let count = count.min(self.unwritten());
        self.slots
            .extend(std::iter::repeat_n(HtmlNameIndexSlot::default(), count));
    }

    /// The table size in slots, once cleared.
    pub(crate) fn size(&self) -> usize {
        self.geometry.map_or(0, HtmlNameIndexSlots::slots)
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

    /// The probe of a name with `hash` over the cleared table, at its first slot.
    pub(crate) fn probe(&self, hash: u64) -> Option<HtmlNameIndexProbe> {
        let geometry = self.geometry.filter(|_| self.unwritten() == 0)?;
        Some(HtmlNameIndexProbe::new(hash, geometry))
    }

    /// What the probe's slot holds.
    pub(crate) fn step(
        &self,
        probe: &HtmlNameIndexProbe,
    ) -> Result<HtmlNameIndexStep, HtmlNameIndexError> {
        match self.slots.get(probe.slot()) {
            None => Err(HtmlNameIndexError::Slot),
            Some(HtmlNameIndexSlot(Some(held))) => Ok(HtmlNameIndexStep::Occupied(held.get() - 1)),
            Some(HtmlNameIndexSlot(None)) => Ok(HtmlNameIndexStep::Vacant(HtmlNameIndexVacancy {
                identity: self.identity,
                slot: probe.slot(),
            })),
        }
    }

    /// Stores `entry` in the empty slot a step found.
    pub(crate) fn insert(
        &mut self,
        vacancy: HtmlNameIndexVacancy,
        entry: usize,
    ) -> Result<(), HtmlNameIndexError> {
        if vacancy.identity != self.identity {
            return Err(HtmlNameIndexError::Foreign);
        }
        let capacity = self.geometry.map_or(0, HtmlNameIndexSlots::entries);
        if self.count >= capacity {
            return Err(HtmlNameIndexError::Full);
        }
        let stored = entry
            .checked_add(1)
            .and_then(NonZeroUsize::new)
            .ok_or(HtmlNameIndexError::Entry)?;
        match self.slots.get_mut(vacancy.slot) {
            None => Err(HtmlNameIndexError::Slot),
            Some(HtmlNameIndexSlot(Some(_))) => Err(HtmlNameIndexError::Taken),
            Some(held) => {
                *held = HtmlNameIndexSlot(Some(stored));
                self.count += 1;
                Ok(())
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::internal::values::{DocumentString, Namespace};

    fn name(text: &str) -> QualifiedName {
        QualifiedName::new(Namespace::none(), None, DocumentString::from(text))
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
            HtmlAttributeNameIndex::slots_required(MAX_SLOTS / 2),
            Some(MAX_SLOTS)
        );
        assert_eq!(
            HtmlAttributeNameIndex::slots_required(MAX_SLOTS / 2 + 1),
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
        assert!(geometry.slots().is_power_of_two() && geometry.slots() <= capacity);
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

    fn cleared(slots: usize) -> HtmlAttributeNameIndex {
        let mut index = HtmlAttributeNameIndex::new(Vec::with_capacity(slots));
        index.begin_clear(HtmlNameIndexSlots::within(slots).expect("a size"));
        index.clear_chunk(slots);
        index
    }

    #[test]
    fn a_probe_visits_every_slot_once_from_its_first_before_it_ends() {
        let index = cleared(8);
        let mut probe = index.probe(index.hash(&name("a"))).expect("cleared");
        let mut seen = [false; 8];
        seen[probe.slot()] = true;
        while let Some(slot) = probe.next_slot() {
            assert!(!seen[slot]);
            seen[slot] = true;
            assert_eq!(probe.slot(), slot);
        }
        assert!(seen.iter().all(|v| *v));
        assert_eq!(probe.next_slot(), None, "a finished probe stays finished");
    }

    #[test]
    fn slot_counts_saturate_and_fit_within_a_capacity() {
        assert_eq!(
            HtmlNameIndexSlots::for_entries(usize::MAX).slots(),
            MAX_SLOTS
        );
        assert_eq!(
            HtmlNameIndexSlots::for_entries(usize::MAX).entries(),
            MAX_SLOTS / 2
        );
        assert_eq!(HtmlNameIndexSlots::for_entries(0).slots(), 2);
        assert_eq!(HtmlNameIndexSlots::for_entries(5).entries(), 5);
        assert_eq!(HtmlNameIndexSlots::within(0), None);
        let within =
            |capacity| HtmlNameIndexSlots::within(capacity).map(|s| (s.slots(), s.entries()));
        assert_eq!(within(1), Some((1, 0)));
        assert_eq!(within(100), Some((64, 32)));
        assert_eq!(within(128), Some((128, 64)));
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
    fn entries_keep_their_occupant_and_tag_and_never_equal_the_largest_value() {
        for hash in [0_u64, 1, u64::MAX, 0xdead_beef_cafe_f00d] {
            for occupant_in in [
                Occupant::Attribute(0),
                Occupant::Attribute(77),
                Occupant::Candidate(0),
                Occupant::Candidate(POSITION_MASK - 1),
            ] {
                let made = entry(tag(hash), occupant_in).expect("fits");
                assert_ne!(made, usize::MAX);
                assert_eq!(occupant(made), (occupant_in, tag(hash)));
            }
        }
        assert_eq!(
            entry(tag(u64::MAX), Occupant::Attribute(POSITION_MASK)),
            None
        );
        assert_eq!(entry(0, Occupant::Candidate(usize::MAX)), None);
    }

    #[test]
    fn a_step_finds_the_first_vacancy_and_an_insertion_fills_exactly_it() {
        let mut index = cleared(8);
        let mut probe = index.probe(index.hash(&name("a"))).expect("cleared");
        let HtmlNameIndexStep::Vacant(vacancy) = index.step(&probe).expect("in range") else {
            panic!("an empty table has no occupant")
        };
        // A second index's vacancy is refused, and so is a vacancy whose slot has been filled.
        let other = cleared(8);
        let HtmlNameIndexStep::Vacant(foreign) = other.step(&probe).expect("in range") else {
            panic!("an empty table has no occupant")
        };
        assert_eq!(index.insert(foreign, 0), Err(HtmlNameIndexError::Foreign));
        let HtmlNameIndexStep::Vacant(duplicate) = index.step(&probe).expect("in range") else {
            panic!("still empty")
        };
        assert_eq!(index.insert(vacancy, 0), Ok(()));
        assert_eq!(
            index.insert(duplicate, 1),
            Err(HtmlNameIndexError::Taken),
            "the slot is no longer empty"
        );
        assert_eq!(index.step(&probe), Ok(HtmlNameIndexStep::Occupied(0)));
        assert!(probe.next_slot().is_some());
        assert!(matches!(
            index.step(&probe),
            Ok(HtmlNameIndexStep::Vacant(_))
        ));
    }

    #[test]
    fn an_insertion_past_the_entries_the_table_was_sized_for_is_refused_without_a_write() {
        let mut index = cleared(4);
        let mut stored = 0;
        for text in ["a", "b", "c", "d", "e", "f"] {
            let mut probe = index.probe(index.hash(&name(text))).expect("cleared");
            let vacancy = loop {
                match index.step(&probe).expect("in range") {
                    HtmlNameIndexStep::Vacant(vacancy) => break vacancy,
                    HtmlNameIndexStep::Occupied(_) => {
                        probe.next_slot().expect("a half full table has a vacancy");
                    }
                }
            };
            match index.insert(vacancy, stored) {
                Ok(()) => stored += 1,
                Err(error) => assert_eq!(error, HtmlNameIndexError::Full),
            }
        }
        assert_eq!(stored, 2, "four slots hold two entries");
        assert_eq!(
            index.slots.iter().filter(|slot| slot.0.is_some()).count(),
            2
        );
    }

    #[test]
    fn errors_describe_themselves() {
        for error in [
            HtmlNameIndexError::Slot,
            HtmlNameIndexError::Full,
            HtmlNameIndexError::Taken,
            HtmlNameIndexError::Foreign,
            HtmlNameIndexError::Entry,
        ] {
            assert!(!error.to_string().is_empty());
            let _: &dyn Error = &error;
        }
    }
}
