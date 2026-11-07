//! Shared spelling hash and slot progression.

use crate::Sym;

const SEED: u64 = 0xcbf2_9ce4_8422_2325;
const MULTIPLIER: u64 = 0x100_0000_01b3;

const fn fold(hash: u64, word: u64) -> u64 {
    (hash ^ word).wrapping_mul(MULTIPLIER)
}

const fn finish(hash: u64, units: usize) -> u64 {
    // Mix the high bits into the low bits before the slot mask is applied.
    let hash = fold(hash, units as u64);
    let hash = (hash ^ (hash >> 33)).wrapping_mul(0xff51_afd7_ed55_8ccd);
    let hash = (hash ^ (hash >> 33)).wrapping_mul(0xc4ce_b9fe_1a85_ec53);
    hash ^ (hash >> 33)
}

const fn pack(a: u16, b: u16, c: u16, d: u16) -> u64 {
    a as u64 | ((b as u64) << 16) | ((c as u64) << 32) | ((d as u64) << 48)
}

pub(super) const fn hash_utf16(units: &[u16]) -> u64 {
    let mut hash = SEED;
    let mut index = 0;
    while units.len() - index >= 4 {
        hash = fold(
            hash,
            pack(
                units[index],
                units[index + 1],
                units[index + 2],
                units[index + 3],
            ),
        );
        index += 4;
    }

    let mut tail = 0;
    let mut shift = 0;
    while index < units.len() {
        tail |= (units[index] as u64) << shift;
        shift += 16;
        index += 1;
    }
    if shift != 0 {
        hash = fold(hash, tail);
    }
    finish(hash, units.len())
}

pub(super) fn hash_utf8(text: &str) -> u64 {
    let mut hash = SEED;
    let (chunks, remainder) = text.as_bytes().as_chunks::<4>();
    let mut units = 0;
    for &[a, b, c, d] in chunks {
        if (a | b | c | d) >= 0x80 {
            break;
        }
        hash = fold(hash, pack(a.into(), b.into(), c.into(), d.into()));
        units += 4;
    }

    if units == text.len() - remainder.len() {
        let tail = match *remainder {
            [] => return finish(hash, units),
            [a] if a < 0x80 => pack(a.into(), 0, 0, 0),
            [a, b] if (a | b) < 0x80 => pack(a.into(), b.into(), 0, 0),
            [a, b, c] if (a | b | c) < 0x80 => pack(a.into(), b.into(), c.into(), 0),
            _ => return finish_utf8(hash, units, &text[units..]),
        };
        return finish(fold(hash, tail), text.len());
    }

    // Each consumed chunk is ASCII, so the remaining slice starts at a
    // character boundary and the consumed byte and unit counts agree.
    finish_utf8(hash, units, &text[units..])
}

fn finish_utf8(mut hash: u64, mut units: usize, suffix: &str) -> u64 {
    let mut tail = 0;
    let mut shift = 0;
    for unit in suffix.encode_utf16() {
        tail |= u64::from(unit) << shift;
        shift += 16;
        units += 1;
        if shift == 64 {
            hash = fold(hash, tail);
            tail = 0;
            shift = 0;
        }
    }
    if shift != 0 {
        hash = fold(hash, tail);
    }
    finish(hash, units)
}

const fn home(hash: u64, capacity: usize) -> usize {
    hash as usize & (capacity - 1)
}

const fn next(slot: usize, capacity: usize) -> usize {
    (slot + 1) & (capacity - 1)
}

pub(super) enum Probe {
    Found(Sym),
    Vacant(usize),
}

pub(super) fn probe(
    slots: &[Option<Sym>],
    hash: u64,
    mut matches: impl FnMut(Sym) -> bool,
) -> Probe {
    if slots.is_empty() {
        return Probe::Vacant(0);
    }
    let mut slot = home(hash, slots.len());
    loop {
        match slots[slot] {
            None => return Probe::Vacant(slot),
            Some(symbol) if matches(symbol) => return Probe::Found(symbol),
            Some(_) => slot = next(slot, slots.len()),
        }
    }
}

pub(super) const fn place(slots: &mut [Option<Sym>], hash: u64, symbol: Sym) -> usize {
    let mut slot = home(hash, slots.len());
    while slots[slot].is_some() {
        slot = next(slot, slots.len());
    }
    slots[slot] = Some(symbol);
    slot
}

pub(super) struct CommonIndex<const N: usize, const CAPACITY: usize> {
    pub(super) slots: [Option<Sym>; CAPACITY],
    pub(super) hashes: [u64; N],
}

impl<const N: usize, const CAPACITY: usize> CommonIndex<N, CAPACITY> {
    pub(super) const fn new(spellings: &[&[u16]]) -> Self {
        assert!(spellings.len() == N && CAPACITY.is_power_of_two() && CAPACITY > N);
        let mut slots = [None; CAPACITY];
        let mut hashes = [0; N];
        let mut index = 0;
        while index < N {
            let hash = hash_utf16(spellings[index]);
            let symbol = Sym::new(index + 1).expect("common symbols are nonzero");
            hashes[index] = hash;
            place(&mut slots, hash, symbol);
            index += 1;
        }
        Self { slots, hashes }
    }
}

#[cfg(test)]
mod tests {
    use super::{Probe, hash_utf8, hash_utf16, place, probe};
    use crate::Sym;
    use alloc::vec::Vec;

    #[test]
    fn packed_hash_agrees_across_encodings() {
        for spelling in [
            "",
            "a\0",
            "aaaa",
            "aaaé",
            "aaaaé",
            "aaaa𐐀",
            "a𐐀baaaa",
            "abcdefgé𐐀",
        ] {
            let utf16: Vec<_> = spelling.encode_utf16().collect();
            assert_eq!(hash_utf8(spelling), hash_utf16(&utf16), "{spelling:?}");
        }
    }

    #[test]
    fn shared_home_slots_check_each_candidate_through_wraparound() {
        let mut slots = [None; 8];
        let first = Sym::new(1).unwrap();
        let second = Sym::new(2).unwrap();
        place(&mut slots, 7, first);
        place(&mut slots, 7, second);
        assert!(
            matches!(probe(&slots, 7, |symbol| symbol == second), Probe::Found(symbol) if symbol == second)
        );
        assert!(matches!(probe(&slots, 7, |_| false), Probe::Vacant(1)));
    }
}
