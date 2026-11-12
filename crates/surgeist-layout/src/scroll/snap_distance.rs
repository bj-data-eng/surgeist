use core::{any::TypeId, cmp::Ordering};

use crate::{
    LayoutScalar,
    scalar::{from_nonnegative_finite_rank, max_nonnegative_finite_rank},
};

// The unit is half the smallest binary64 subnormal. Four finite scalar
// terms need at most 2101 bits, including the carry above their top bit.
const WORDS: usize = 33;
const WORD_BITS: usize = u64::BITS as usize;
const UNIT_EXPONENT: usize = 1075;
const FRACTION_BITS: usize = f64::MANTISSA_DIGITS as usize - 1;

struct Magnitude([u64; WORDS]);

impl Magnitude {
    fn add(&mut self, significand: u64, shift: usize) {
        let word = shift / WORD_BITS;
        let bit = shift % WORD_BITS;
        self.add_word(word, significand << bit);
        if bit != 0 {
            self.add_word(word + 1, significand >> (WORD_BITS - bit));
        }
    }

    fn add_word(&mut self, mut index: usize, mut value: u64) {
        while value != 0 {
            let (sum, carry) = self.0[index].overflowing_add(value);
            self.0[index] = sum;
            value = u64::from(carry);
            index += 1;
        }
    }

    fn compare(&self, other: &Self) -> Ordering {
        self.0.iter().rev().cmp(other.0.iter().rev())
    }

    fn subtract(mut self, other: &Self) -> Self {
        let mut borrow = false;
        for (left, &right) in self.0.iter_mut().zip(&other.0) {
            let (difference, next_borrow) = left.borrowing_sub(right, borrow);
            *left = difference;
            borrow = next_borrow;
        }
        debug_assert!(!borrow);
        self
    }

    fn highest_bit(&self) -> Option<usize> {
        let index = self.0.iter().rposition(|&word| word != 0)?;
        Some(index * WORD_BITS + WORD_BITS - 1 - self.0[index].leading_zeros() as usize)
    }

    fn shifted_word(&self, shift: usize) -> u64 {
        let word = shift / WORD_BITS;
        let bit = shift % WORD_BITS;
        let low = self.0[word] >> bit;
        if bit == 0 {
            low
        } else {
            low | (self.0.get(word + 1).copied().unwrap_or(0) << (WORD_BITS - bit))
        }
    }

    fn has_bits_below(&self, shift: usize) -> bool {
        let word = shift / WORD_BITS;
        let bit = shift % WORD_BITS;
        self.0[..word].iter().any(|&part| part != 0) || self.0[word] & ((1_u64 << bit) - 1) != 0
    }
}

pub(super) enum SnapRounding {
    Nearest,
    Up,
    Down,
}

pub(super) struct SnapDistance {
    positive: Magnitude,
    negative: Magnitude,
}

impl SnapDistance {
    // Every input is finite. The optional half applies to the extent terms
    // of center alignment without rounding either extent in isolation.
    pub(super) fn sum<S: LayoutScalar>(terms: &[S], half_terms: &[S]) -> Self {
        debug_assert!(terms.len() + half_terms.len() <= 4);
        let mut distance = Self {
            positive: Magnitude([0; WORDS]),
            negative: Magnitude([0; WORDS]),
        };
        for (term, half) in terms
            .iter()
            .map(|&term| (term, false))
            .chain(half_terms.iter().map(|&term| (term, true)))
        {
            let term = term.to_f64();
            debug_assert!(term.is_finite());
            let bits = term.to_bits();
            let fraction = bits & ((1_u64 << FRACTION_BITS) - 1);
            let exponent = ((bits >> FRACTION_BITS) & 0x7ff) as usize;
            let (significand, shift) = if exponent == 0 {
                (fraction, usize::from(!half))
            } else {
                (
                    (1_u64 << FRACTION_BITS) | fraction,
                    exponent - usize::from(half),
                )
            };
            let magnitude = if term.is_sign_negative() {
                &mut distance.negative
            } else {
                &mut distance.positive
            };
            magnitude.add(significand, shift);
        }
        distance
    }

    pub(super) fn sign(&self) -> Ordering {
        self.positive.compare(&self.negative)
    }

    pub(super) fn rounded<S: LayoutScalar>(self, rounding: SnapRounding) -> S {
        let (negative, magnitude) = match self.sign() {
            Ordering::Less => (true, self.negative.subtract(&self.positive)),
            Ordering::Equal => return S::ZERO,
            Ordering::Greater => (false, self.positive.subtract(&self.negative)),
        };
        let highest = magnitude.highest_bit().expect("nonzero distance");
        let (fraction_bits, subnormal_shift) = if TypeId::of::<S>() == TypeId::of::<f32>() {
            (f32::MANTISSA_DIGITS as usize - 1, UNIT_EXPONENT - 149)
        } else {
            (FRACTION_BITS, UNIT_EXPONENT - 1074)
        };
        let shift = highest.saturating_sub(fraction_bits).max(subnormal_shift);
        let significand = magnitude.shifted_word(shift);
        let rounds_away = match rounding {
            SnapRounding::Nearest => {
                let guard = shift - 1;
                magnitude.shifted_word(guard) & 1 != 0
                    && (significand & 1 != 0 || magnitude.has_bits_below(guard))
            }
            SnapRounding::Up => !negative && magnitude.has_bits_below(shift),
            SnapRounding::Down => negative && magnitude.has_bits_below(shift),
        };
        // The significand fits exactly in binary64. Its power of two is also
        // representable, including at the subnormal boundary.
        let scale = if shift < UNIT_EXPONENT - 1022 {
            f64::from_bits(1 << (shift - 1))
        } else {
            f64::from_bits(((shift + 1023 - UNIT_EXPONENT) as u64) << FRACTION_BITS)
        };
        let value = S::from_f64((significand + u64::from(rounds_away)) as f64 * scale);
        let toward_zero = match rounding {
            SnapRounding::Nearest => false,
            SnapRounding::Up => negative,
            SnapRounding::Down => !negative,
        };
        let value = if toward_zero && !value.is_finite() {
            from_nonnegative_finite_rank(max_nonnegative_finite_rank::<S>())
        } else {
            value
        };
        if negative && value != S::ZERO {
            -value
        } else {
            value
        }
    }
}
