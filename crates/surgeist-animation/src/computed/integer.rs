//! Exact dyadic interpolation followed by CSS integer rounding.

use std::cmp::Ordering;

// Nonzero integral endpoints have a power of at least zero after removing
// trailing significand zeros. Finite binary64 progress has a power of at
// least -1074. The magnitude of each product is below 2^2048; a sum of
// three terms and the rounding offset fits below 2^2050.
const FRACTION_WORDS: usize = 18;
const WORDS: usize = 51;

#[derive(Default)]
struct Sum {
    positive: Magnitude,
    negative: Magnitude,
}

impl Sum {
    fn add_product(&mut self, integer: f64, factor: f64, subtract: bool) {
        let (integer_significand, integer_power) = significand(integer);
        let (factor_significand, factor_power) = significand(factor);
        if integer_significand == 0 || factor_significand == 0 {
            return;
        }
        let trailing = integer_significand.trailing_zeros();
        let product = u128::from(integer_significand >> trailing) * u128::from(factor_significand);
        let power = integer_power + trailing as i32 + factor_power;
        let shift = (power + (FRACTION_WORDS * 64) as i32) as usize;
        let negative = integer.is_sign_negative() ^ factor.is_sign_negative() ^ subtract;
        let magnitude = if negative {
            &mut self.negative
        } else {
            &mut self.positive
        };
        magnitude.add_shifted(product as u64, shift);
        magnitude.add_shifted((product >> 64) as u64, shift + 64);
    }

    fn rounded(mut self) -> f64 {
        // floor(n + 1/2) rounds ties toward positive infinity.
        self.positive.add_shifted(1, FRACTION_WORDS * 64 - 1);
        match self
            .positive
            .0
            .iter()
            .rev()
            .cmp(self.negative.0.iter().rev())
        {
            Ordering::Less => {
                self.negative.subtract(&self.positive);
                if self
                    .negative
                    .0
                    .iter()
                    .take(FRACTION_WORDS)
                    .any(|&word| word != 0)
                {
                    // floor(-x) is -ceil(x).
                    self.negative.add_shifted(1, FRACTION_WORDS * 64);
                }
                -self.negative.integer_as_f64()
            }
            Ordering::Equal => 0.0,
            Ordering::Greater => {
                self.positive.subtract(&self.negative);
                self.positive.integer_as_f64()
            }
        }
    }
}

struct Magnitude([u64; WORDS]);

impl Default for Magnitude {
    fn default() -> Self {
        Self([0; WORDS])
    }
}

impl Magnitude {
    fn add_shifted(&mut self, value: u64, shift: usize) {
        let word = shift / 64;
        let bit = shift % 64;
        self.add_word(word, value << bit);
        if bit != 0 {
            self.add_word(word + 1, value >> (64 - bit));
        }
    }

    fn add_word(&mut self, index: usize, mut value: u64) {
        for word in self.0.iter_mut().skip(index) {
            if value == 0 {
                break;
            }
            let (next, carry) = word.overflowing_add(value);
            *word = next;
            value = u64::from(carry);
        }
    }

    fn subtract(&mut self, smaller: &Self) {
        let mut borrow = false;
        for (word, &other) in self.0.iter_mut().zip(&smaller.0) {
            let (value, underflow) = word.borrowing_sub(other, borrow);
            *word = value;
            borrow = underflow;
        }
    }

    fn integer_as_f64(&self) -> f64 {
        let Some((index, &word)) = self
            .0
            .iter()
            .enumerate()
            .skip(FRACTION_WORDS)
            .rev()
            .find(|&(_, word)| *word != 0)
        else {
            return 0.0;
        };
        let highest = (index - FRACTION_WORDS) * 64 + (63 - word.leading_zeros()) as usize;
        if highest < 53 {
            return self.word(FRACTION_WORDS) as f64;
        }
        if highest > 1023 {
            return f64::INFINITY;
        }

        // Once the CSS integer has been determined, round it to binary64
        // with ties to even. Discarded fractional bits are irrelevant here.
        let shift = highest - 52;
        let word_index = FRACTION_WORDS + shift / 64;
        let offset = shift % 64;
        let mantissa = self.word(word_index) >> offset;
        let mantissa = if offset == 0 {
            mantissa
        } else {
            mantissa | (self.word(word_index + 1) << (64 - offset))
        };
        let guard_index = FRACTION_WORDS + (shift - 1) / 64;
        let guard = 1_u64 << ((shift - 1) % 64);
        let guard_word = self.word(guard_index);
        let sticky = guard_word & (guard - 1) != 0
            || self
                .0
                .iter()
                .skip(FRACTION_WORDS)
                .take(guard_index - FRACTION_WORDS)
                .any(|&word| word != 0);
        let round_up = guard_word & guard != 0 && (sticky || mantissa & 1 != 0);
        let exponent = ((highest as u64 + 1022) << 52) + mantissa;
        f64::from_bits(exponent + u64::from(round_up))
    }

    fn word(&self, index: usize) -> u64 {
        self.0.get(index).copied().unwrap_or(0)
    }
}

fn significand(value: f64) -> (u64, i32) {
    let bits = value.to_bits();
    let exponent = ((bits >> 52) & 0x7ff) as i32;
    let fraction = bits & ((1_u64 << 52) - 1);
    if exponent == 0 {
        (fraction, -1074)
    } else {
        (fraction | (1_u64 << 52), exponent - 1075)
    }
}

pub(super) fn interpolate(first: f64, second: f64, progress: f64) -> f64 {
    let mut sum = Sum::default();
    // Expand first + (second - first) * progress without rounding the
    // difference, either product, or the sum before integer rounding.
    sum.add_product(first, 1.0, false);
    sum.add_product(second, progress, false);
    sum.add_product(first, progress, true);
    sum.rounded()
}
