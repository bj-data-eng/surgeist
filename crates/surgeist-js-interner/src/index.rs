//! Shared spelling hash and slot progression.

use crate::{InternerWork, Sym};

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

pub(super) fn hash_latin1(bytes: &[u8]) -> u64 {
    let mut hash = SEED;
    let (chunks, remainder) = bytes.as_chunks::<4>();
    for &[a, b, c, d] in chunks {
        hash = fold(hash, pack(a.into(), b.into(), c.into(), d.into()));
    }

    if !remainder.is_empty() {
        let mut tail = 0;
        for (index, &byte) in remainder.iter().enumerate() {
            tail |= u64::from(byte) << (index * 16);
        }
        hash = fold(hash, tail);
    }
    finish(hash, bytes.len())
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

pub(super) fn probe<E, F>(
    slots: &[Option<Sym>],
    hash: u64,
    admit: &mut F,
    mut matches: impl FnMut(Sym, &mut F) -> Result<bool, E>,
) -> Result<Probe, E>
where
    F: FnMut(InternerWork) -> Result<(), E>,
{
    if slots.is_empty() {
        return Ok(Probe::Vacant(0));
    }
    let mut slot = home(hash, slots.len());
    loop {
        admit(InternerWork::Probe)?;
        match slots[slot] {
            None => return Ok(Probe::Vacant(slot)),
            Some(symbol) if matches(symbol, admit)? => return Ok(Probe::Found(symbol)),
            Some(_) => slot = next(slot, slots.len()),
        }
    }
}

const fn place(slots: &mut [Option<Sym>], hash: u64, symbol: Sym) {
    let mut slot = home(hash, slots.len());
    while slots[slot].is_some() {
        slot = next(slot, slots.len());
    }
    slots[slot] = Some(symbol);
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
    use super::{Probe, hash_latin1, hash_utf8, hash_utf16, probe};
    use crate::{InternerWork, Sym};
    use alloc::vec::Vec;
    use core::convert::Infallible;

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
    fn latin1_hash_widens_high_bytes_and_partial_chunks() {
        let latin1 = [0, 0xff, 0xe9, b'a', b'z', 0x80, 0, 0xab, b'b'];
        let utf16 = [0, 0xff, 0xe9, 0x61, 0x7a, 0x80, 0, 0xab, 0x62];
        for len in 0..=latin1.len() {
            assert_eq!(hash_latin1(&latin1[..len]), hash_utf16(&utf16[..len]));
        }
    }

    #[test]
    fn shared_home_slots_check_each_candidate_through_wraparound() {
        let mut slots = [None; 8];
        let first = Sym::new(1).unwrap();
        let second = Sym::new(2).unwrap();
        slots[7] = Some(first);
        slots[0] = Some(second);
        let mut admit = |_| Ok::<_, Infallible>(());
        assert!(
            matches!(probe(&slots, 7, &mut admit, |symbol, _| Ok(symbol == second)), Ok(Probe::Found(symbol)) if symbol == second)
        );
        assert!(matches!(
            probe(&slots, 7, &mut admit, |_, _| Ok(false)),
            Ok(Probe::Vacant(1))
        ));
    }

    #[test]
    fn probe_refuses_before_wrapped_candidate_or_terminating_empty_slot() {
        let first = Sym::new(1).unwrap();
        let second = Sym::new(2).unwrap();
        let slots = [Some(second), None, None, Some(first)];
        for allowed in [1, 2] {
            let mut admitted = 0;
            let mut compared = Vec::new();
            let result = probe(
                &slots,
                3,
                &mut |work| {
                    assert!(matches!(work, InternerWork::Probe));
                    if admitted == allowed {
                        return Err("probe refused");
                    }
                    admitted += 1;
                    Ok(())
                },
                |symbol, _| {
                    compared.push(symbol);
                    Ok(false)
                },
            );
            assert!(matches!(result, Err("probe refused")));
            assert_eq!(admitted, allowed);
            assert_eq!(compared, [first, second][..allowed]);
        }
    }

    #[test]
    fn absent_table_needs_no_admission() {
        let result = probe(&[], 7, &mut |_| Err(()), |_, _| unreachable!());
        assert!(matches!(result, Ok(Probe::Vacant(0))));
    }
}
