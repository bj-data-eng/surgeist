//! An insert-only index from names to entry numbers, in a caller-owned open-addressing table.
//!
//! The consumer owns the table's memory and admission; this module owns its rules. The slot
//! count is a power of two, the table is kept at most half full by sizing it for the most
//! entries it will hold, an entry goes into the first empty slot of its probe sequence, and
//! nothing is removed. The hash key is random for each [`HtmlNameIndexHasher`], so no input can
//! choose names that collide and the expected probes per lookup are constant. The consumer
//! charges the hash of a name (one unit inspection per unit), each probe and each comparison.

use std::{
    hash::{BuildHasher, RandomState},
    num::NonZeroUsize,
};

/// The slot count of a name index: a power of two.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct HtmlNameIndexSlots(NonZeroUsize);
impl HtmlNameIndexSlots {
    /// The least slot count that holds `max_entries` entries at most half full, at least two.
    /// A count beyond 2^(usize::BITS - 2) saturates at the largest power of two, which no table
    /// can reach: reserving it is the consumer's admission to refuse, and a lookup in a table
    /// that is too full reports [`HtmlNameIndexError::Full`] instead of looping.
    pub fn for_entries(max_entries: usize) -> Self {
        let slots = max_entries
            .saturating_mul(2)
            .max(2)
            .checked_next_power_of_two()
            .unwrap_or(1 << (usize::BITS - 1));
        Self(NonZeroUsize::new(slots).unwrap_or(NonZeroUsize::MIN))
    }
    /// The number of slots.
    pub fn get(self) -> usize {
        self.0.get()
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

/// The slots that a name visits after its first, wrapping around once. It keeps no table and does
/// no I/O, so a consumer that admits each probe as its own action steps it between actions.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct HtmlNameIndexProbe {
    slot: usize,
    mask: usize,
    remaining: usize,
}
impl HtmlNameIndexProbe {
    /// The first slot of a hash, and the probe that yields the slots after it.
    pub fn start(hash: u64, slots: HtmlNameIndexSlots) -> (usize, Self) {
        let mask = slots.get().saturating_sub(1);
        let first = (hash as usize) & mask;
        (
            first,
            Self {
                slot: first,
                mask,
                remaining: slots.get().saturating_sub(1),
            },
        )
    }
}
impl Iterator for HtmlNameIndexProbe {
    type Item = usize;
    /// The following slot, or `None` once every slot has been visited.
    fn next(&mut self) -> Option<usize> {
        if self.remaining == 0 {
            return None;
        }
        self.remaining -= 1;
        self.slot = self.slot.wrapping_add(1) & self.mask;
        Some(self.slot)
    }
}

/// A table whose length is its slot count. An empty slot is `None`; an occupied one holds its
/// entry number plus one.
#[derive(Debug)]
pub struct HtmlNameIndexTable<'a> {
    slots: HtmlNameIndexSlots,
    entries: &'a mut [Option<NonZeroUsize>],
}
/// The table slice does not have the slot count.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct HtmlNameIndexLengthError {
    expected: usize,
    actual: usize,
}
impl HtmlNameIndexLengthError {
    /// The slot count that the table needed.
    pub fn expected(&self) -> usize {
        self.expected
    }
    /// The length of the slice that was supplied.
    pub fn actual(&self) -> usize {
        self.actual
    }
}
/// The result of a lookup.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum HtmlNameIndexLookup {
    /// The earlier entry with an equal name; nothing was inserted.
    Found(usize),
    /// No entry has the name; the new entry was recorded in the first empty slot.
    Inserted,
}
/// Why a lookup stopped. In either case the table is unchanged.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum HtmlNameIndexError<E> {
    /// A charge or comparison callback refused.
    Consumer(E),
    /// Every slot is occupied by another name, or the entry number cannot be recorded: the table
    /// was not sized for its entries.
    Full,
}
impl<'a> HtmlNameIndexTable<'a> {
    /// Wraps `entries`, which must hold exactly `slots` empty-or-occupied slots.
    pub fn new(
        slots: HtmlNameIndexSlots,
        entries: &'a mut [Option<NonZeroUsize>],
    ) -> Result<Self, HtmlNameIndexLengthError> {
        if entries.len() != slots.get() {
            return Err(HtmlNameIndexLengthError {
                expected: slots.get(),
                actual: entries.len(),
            });
        }
        Ok(Self { slots, entries })
    }
    /// The slot count.
    pub fn slots(&self) -> HtmlNameIndexSlots {
        self.slots
    }
    /// The entry number in `slot`, or `None` when it is empty or out of range.
    pub fn entry(&self, slot: usize) -> Option<usize> {
        self.entries
            .get(slot)
            .copied()
            .flatten()
            .map(|recorded| recorded.get() - 1)
    }
    /// Records `entry` in `slot`, which must be empty. Returns whether it did. This is the
    /// stepwise primitive: a consumer that admits each probe as an action reads
    /// [`entry`](Self::entry) at the slots of an [`HtmlNameIndexProbe`], compares names, and
    /// inserts at the first empty one.
    pub fn insert(&mut self, slot: usize, entry: usize) -> bool {
        let Some(recorded) = entry.checked_add(1).and_then(NonZeroUsize::new) else {
            return false;
        };
        match self.entries.get_mut(slot) {
            Some(target @ None) => {
                *target = Some(recorded);
                true
            }
            _ => false,
        }
    }
    /// Finds the entry whose name equals the sought one, or records `entry` (the number that the
    /// new name will have) in the first empty slot of the hash's probe sequence.
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
        if entry.checked_add(1).is_none() {
            return Err(HtmlNameIndexError::Full);
        }
        let (mut slot, mut probe) = HtmlNameIndexProbe::start(hash, self.slots);
        loop {
            step(context).map_err(HtmlNameIndexError::Consumer)?;
            match self.entry(slot) {
                None => {
                    return if self.insert(slot, entry) {
                        Ok(HtmlNameIndexLookup::Inserted)
                    } else {
                        Err(HtmlNameIndexError::Full)
                    };
                }
                Some(existing) => {
                    if same_name(context, existing).map_err(HtmlNameIndexError::Consumer)? {
                        return Ok(HtmlNameIndexLookup::Found(existing));
                    }
                }
            }
            slot = probe.next().ok_or(HtmlNameIndexError::Full)?;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

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
            let slots = HtmlNameIndexSlots::for_entries(entries).get();
            assert!(slots.is_power_of_two() && 2 * entries <= slots);
        }
        for huge in [usize::MAX / 4 + 1, usize::MAX] {
            assert_eq!(
                HtmlNameIndexSlots::for_entries(huge).get(),
                1 << (usize::BITS - 1)
            );
        }
    }
    #[test]
    fn a_probe_visits_every_slot_once_and_then_ends() {
        for entries in [0_usize, 1, 3, 10] {
            let slots = HtmlNameIndexSlots::for_entries(entries);
            for hash in [0_u64, 1, 12345, u64::MAX] {
                let (first, probe) = HtmlNameIndexProbe::start(hash, slots);
                let mut visited: Vec<usize> = std::iter::once(first).chain(probe).collect();
                visited.sort_unstable();
                assert_eq!(visited, (0..slots.get()).collect::<Vec<_>>());
            }
        }
        let (_, mut probe) = HtmlNameIndexProbe::start(7, HtmlNameIndexSlots::for_entries(0));
        assert_eq!(probe.by_ref().count(), 1);
        assert_eq!(probe.next(), None);
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
            let mut entries = vec![None; slots.get()];
            let mut table = HtmlNameIndexTable::new(slots, &mut entries).unwrap();
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
            assert_eq!(entries.iter().flatten().count(), names.len());
        }
    }
    #[test]
    fn a_table_of_the_wrong_length_a_full_table_and_a_refusal_leave_the_table_unchanged() {
        let hasher = HtmlNameIndexHasher::new();
        let slots = HtmlNameIndexSlots::for_entries(0);
        assert_eq!(slots.get(), 2);
        let mut short = vec![None; 1];
        let error = HtmlNameIndexTable::new(slots, &mut short).unwrap_err();
        assert_eq!((error.expected(), error.actual()), (2, 1));
        // Two slots hold two distinct names; a third, different name finds no empty slot.
        let mut entries = vec![None; 2];
        let mut table = HtmlNameIndexTable::new(slots, &mut entries).unwrap();
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
            Err(HtmlNameIndexError::Full)
        );
        assert!(!table.insert(0, 5) && !table.insert(9, 5));
        assert!(entries.iter().all(Option::is_some));
        // A refused charge stops the lookup before the table changes.
        let mut entries = vec![None; 2];
        let mut table = HtmlNameIndexTable::new(slots, &mut entries).unwrap();
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
        assert_eq!(entries, [None, None]);
        // An entry number that cannot be recorded is refused.
        let mut entries = vec![None; 2];
        let mut table = HtmlNameIndexTable::new(slots, &mut entries).unwrap();
        assert_eq!(
            table.find_or_insert::<(), ()>(0, usize::MAX, &mut (), |_| Ok(()), |_, _| Ok(false)),
            Err(HtmlNameIndexError::Full)
        );
    }
}
