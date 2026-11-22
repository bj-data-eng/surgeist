//! Exact range comparison for a channel whose wide result is near overflow.

use crate::wide::Wide;

// The smallest exponent of a normalized binary64 integer significand is
// -1074 - 52. The largest term multiplies two unrestricted finite values
// and a unit alpha. Six such terms need fewer than 2052 bits above zero.
const FIRST_BIT: i32 = 3 * (-1074 - 52);
const LIMBS: usize = 86;

/// Compares the unrounded weighted numerator with either signed finite limit.
/// A nonzero divisor supplies its sign. No divisor compares the numerator
/// itself, including the premultiplied coordinate at alpha zero.
pub(super) fn exceeds(
    first: (f64, f64),
    second: (f64, f64),
    progress: f64,
    divisor: Option<f64>,
) -> bool {
    let (x, a) = first;
    let (y, b) = second;
    [-1.0, 1.0].into_iter().any(|direction| {
        let mut sum = ExactSum::default();
        sum.product([direction * x, a, 1.0]);
        sum.product([-direction * x, a, progress]);
        sum.product([direction * y, b, progress]);
        match divisor {
            Some(sign) => {
                // Subtract MAX times the magnitude of the weighted alpha.
                let limit = f64::MAX.copysign(sign);
                sum.product([-limit, a, 1.0]);
                sum.product([limit, a, progress]);
                sum.product([-limit, b, progress]);
            }
            None => sum.product([-f64::MAX, 1.0, 1.0]),
        }
        sum.positive
            .iter()
            .rev()
            .cmp(sum.negative.iter().rev())
            .is_gt()
    })
}

struct ExactSum {
    positive: [u64; LIMBS],
    negative: [u64; LIMBS],
}

impl Default for ExactSum {
    fn default() -> Self {
        Self {
            positive: [0; LIMBS],
            negative: [0; LIMBS],
        }
    }
}

impl ExactSum {
    fn product(&mut self, factors: [f64; 3]) {
        if factors.contains(&0.0) {
            return;
        }
        let mut product = [1_u64, 0, 0];
        let mut exponent = 0;
        let mut negative = false;
        for factor in factors {
            let (significand, power) = Wide::from_f64(factor).unsigned_parts();
            exponent += power;
            negative ^= factor.is_sign_negative();
            let mut carry = 0;
            // Three 53-bit significands fit in the three 64-bit limbs.
            for word in &mut product {
                let multiplied = u128::from(*word) * u128::from(significand) + carry;
                *word = multiplied as u64;
                carry = multiplied >> 64;
            }
        }
        let total = if negative {
            &mut self.negative
        } else {
            &mut self.positive
        };
        let shift = (exponent - FIRST_BIT) as usize;
        let bits = shift % 64;
        for (offset, word) in product.into_iter().enumerate() {
            let index = shift / 64 + offset;
            add_word(total, index, word << bits);
            if bits != 0 {
                add_word(total, index + 1, word >> (64 - bits));
            }
        }
    }
}

fn add_word(total: &mut [u64; LIMBS], mut index: usize, mut word: u64) {
    while word != 0
        && let Some(limb) = total.get_mut(index)
    {
        let (updated, carry) = limb.overflowing_add(word);
        *limb = updated;
        word = u64::from(carry);
        index += 1;
    }
}
