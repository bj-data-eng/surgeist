//! Greek uppercase support adapted from ICU4X 2.2.0 `greek_to_me`.
//! Copyright Unicode, Inc. Licensed under Unicode-3.0; see
//! `licenses/icu4x/LICENSE` in the crate root. Context is immutable input scalars,
//! including opaque non-scalars; every emission remains owned by its input row.

#[path = "greek_data.rs"]
mod data;

#[derive(Clone, Copy, Default)]
pub(super) struct Diacritics {
    pub(super) accented: bool,
    pub(super) dialytika: bool,
    pub(super) ypogegrammeni: bool,
}

#[derive(Clone, Copy)]
pub(super) enum Letter {
    Vowel(char, Diacritics),
    Consonant { rho: bool },
}

pub(super) fn letter(cp: u32) -> Option<Letter> {
    let packed = if (0x370..=0x3ff).contains(&cp) {
        *data::DATA_370.get(usize::try_from(cp - 0x370).ok()?)?
    } else if (0x1f00..0x1fff).contains(&cp) {
        *data::DATA_1F00.get(usize::try_from(cp - 0x1f00).ok()?)?
    } else {
        data::match_extras(char::from_u32(cp)?)?
    };
    if packed == 0 {
        return None;
    }
    if packed & 0x80 != 0 {
        return Some(Letter::Consonant {
            rho: packed == 0x81,
        });
    }
    let vowel = match packed & 0xf {
        1 => 'Α',
        2 => 'Ε',
        3 => 'Η',
        4 => 'Ι',
        5 => 'Ο',
        6 => 'Υ',
        7 => 'Ω',
        8 => 'ϒ',
        _ => return None,
    };
    Some(Letter::Vowel(
        vowel,
        Diacritics {
            accented: packed & 0x40 != 0,
            dialytika: packed & 0x20 != 0,
            ypogegrammeni: packed & 0x10 != 0,
        },
    ))
}

pub(super) fn accent(cp: u32) -> bool {
    matches!(cp, 0x300 | 0x301 | 0x342 | 0x302 | 0x303 | 0x311)
}

fn breathing(cp: u32) -> bool {
    matches!(cp, 0x304 | 0x306 | 0x313 | 0x314 | 0x343)
}

pub(super) fn removed_mark(cp: u32) -> bool {
    accent(cp) || breathing(cp) || matches!(cp, 0x308 | 0x344)
}

pub(super) fn preceded_by_letter(before: &[u32]) -> bool {
    before
        .iter()
        .rev()
        .copied()
        .find(|&cp| !removed_mark(cp) && cp != 0x345)
        .is_some_and(|cp| letter(cp).is_some())
}

pub(super) fn following(mut d: Diacritics, after: &[u32]) -> Diacritics {
    for &cp in after {
        if accent(cp) {
            d.accented = true;
        } else if cp == 0x344 {
            d.accented = true;
            d.dialytika = true;
        } else if cp == 0x308 {
            d.dialytika = true;
        } else if cp == 0x345 {
            d.ypogegrammeni = true;
        } else if !breathing(cp) {
            break;
        }
    }
    d
}

pub(super) fn preceding(before: &[u32]) -> Option<(Diacritics, Diacritics)> {
    let mut combining = Diacritics::default();
    for &cp in before.iter().rev() {
        if accent(cp) {
            combining.accented = true;
        } else if cp == 0x344 {
            combining.accented = true;
            combining.dialytika = true;
        } else if cp == 0x308 {
            combining.dialytika = true;
        } else if !breathing(cp) {
            return match letter(cp)? {
                Letter::Vowel(_, precomposed) => Some((precomposed, combining)),
                Letter::Consonant { .. } => None,
            };
        }
    }
    None
}
