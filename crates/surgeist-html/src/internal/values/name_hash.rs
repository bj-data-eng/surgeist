//! Perfect hashes over the UTF-16 code units of a closed set of ASCII names.
//!
//! `tools/generate_vocabulary.py` finds each table. A table maps every name of its set to its
//! own slot (hash and displace): the hash of the units selects a bucket, the bucket's
//! displacement pair moves the hash to a slot, and the slot holds the position of the one name
//! that can match. A name outside the set reaches some slot too, so the caller compares the
//! candidate's spelling with the units. A lookup reads the units once to hash them and once to
//! compare them, so it does work proportional to the longest name of the set at most, whatever
//! the length of the input.
//!
//! The functions are `const` and use no allocation, no panicking operation and no indexing.

/// The table of one name set.
#[derive(Clone, Copy, Debug)]
pub(crate) struct NameHash {
    seed: u64,
    longest: usize,
    displacements: &'static [(u32, u32)],
    slots: &'static [u16],
}

/// The element at `index`, or `None` beyond the end.
pub(crate) const fn at<T: Copy>(table: &[T], index: usize) -> Option<T> {
    match table.split_at_checked(index) {
        Some((_, rest)) => match rest.split_first() {
            Some((first, _)) => Some(*first),
            None => None,
        },
        None => None,
    }
}

/// Whether `units` spell exactly the ASCII `name`.
pub(crate) const fn units_spell(units: &[u16], name: &str) -> bool {
    let mut units = units;
    let mut bytes = name.as_bytes();
    if units.len() != bytes.len() {
        return false;
    }
    loop {
        match (units.split_first(), bytes.split_first()) {
            (None, None) => return true,
            (Some((&unit, rest_units)), Some((&byte, rest_bytes))) => {
                if unit != byte as u16 {
                    return false;
                }
                units = rest_units;
                bytes = rest_bytes;
            }
            _ => return false,
        }
    }
}

/// The 64-bit hash of `units` under `seed`: FNV-1a over the units, then a finalizing mix.
const fn hash_units(units: &[u16], seed: u64) -> u64 {
    let mut state = seed ^ 0xCBF2_9CE4_8422_2325;
    let mut rest = units;
    while let Some((&unit, tail)) = rest.split_first() {
        state = (state ^ unit as u64).wrapping_mul(0x0000_0100_0000_01B3);
        rest = tail;
    }
    state ^= state >> 30;
    state = state.wrapping_mul(0xBF58_476D_1CE4_E5B9);
    state ^= state >> 27;
    state = state.wrapping_mul(0x94D0_49BB_1331_11EB);
    state ^ (state >> 31)
}

impl NameHash {
    /// A table, as the generator emits it.
    pub(crate) const fn new(
        seed: u64,
        longest: usize,
        displacements: &'static [(u32, u32)],
        slots: &'static [u16],
    ) -> Self {
        Self {
            seed,
            longest,
            displacements,
            slots,
        }
    }

    /// The position in the name set of the only name that can equal `units`, or `None` if no name
    /// of the set can: the units are longer than every name, or the table has no such slot.
    /// The caller compares the spelling of the candidate.
    pub(crate) const fn candidate(&self, units: &[u16]) -> Option<usize> {
        if units.len() > self.longest {
            return None;
        }
        let state = hash_units(units, self.seed);
        let selector = state.rotate_left(21) as u32;
        let first = state as u32;
        let second = (state >> 32) as u32;
        let Some(bucket) = (selector as usize).checked_rem(self.displacements.len()) else {
            return None;
        };
        let Some((d1, d2)) = at(self.displacements, bucket) else {
            return None;
        };
        let displaced = d2.wrapping_add(first.wrapping_mul(d1)).wrapping_add(second);
        let Some(slot) = (displaced as usize).checked_rem(self.slots.len()) else {
            return None;
        };
        match at(self.slots, slot) {
            Some(position) => Some(position as usize),
            None => None,
        }
    }
}
