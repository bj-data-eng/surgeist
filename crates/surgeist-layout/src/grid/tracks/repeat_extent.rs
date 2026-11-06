use core::cmp::Ordering;

// Finite binary64 values are integers in units of 2^-1074. Each magnitude
// needs 2098 bits, plus at most two usize factors for track and group counts
// and two bits for addition of track, gap, and bound terms.
const WORDS: usize = (2098 + 2 * usize::BITS as usize + 2).div_ceil(64);

pub(super) struct RepeatExtent {
    positive: [u64; WORDS],
    negative: [u64; WORDS],
}

impl Default for RepeatExtent {
    fn default() -> Self {
        Self {
            positive: [0; WORDS],
            negative: [0; WORDS],
        }
    }
}

impl RepeatExtent {
    pub(super) fn add(&mut self, value: f64, count: usize) {
        debug_assert!(value.is_finite());
        let bits = value.to_bits();
        let exponent = ((bits >> 52) & 0x7ff) as usize;
        let fraction = bits & ((1_u64 << 52) - 1);
        let mantissa = if exponent == 0 {
            fraction
        } else {
            fraction | (1_u64 << 52)
        };
        let product = u128::from(mantissa) * count as u128;
        let words = if value.is_sign_negative() {
            &mut self.negative
        } else {
            &mut self.positive
        };
        let shift = exponent.saturating_sub(1);
        let index = shift / 64;
        let offset = shift % 64;
        let low = product as u64;
        let high = (product >> 64) as u64;
        add_word(words, index, low << offset);
        if offset == 0 {
            add_word(words, index + 1, high);
        } else {
            add_word(words, index + 1, (high << offset) | (low >> (64 - offset)));
            add_word(words, index + 2, high >> (64 - offset));
        }
    }

    pub(super) fn negated(&self) -> Self {
        Self {
            positive: self.negative,
            negative: self.positive,
        }
    }

    pub(super) fn compare_scaled(&self, count: usize, bound: &Self) -> Ordering {
        let mut left_carry = 0;
        let mut right_carry = 0;
        let mut comparison = Ordering::Equal;
        // Move negative terms to the opposite side and propagate carries from
        // low to high. The most significant difference determines the result.
        for index in 0..WORDS {
            let left = u128::from(self.positive[index]) * count as u128
                + u128::from(bound.negative[index])
                + left_carry;
            let right = u128::from(self.negative[index]) * count as u128
                + u128::from(bound.positive[index])
                + right_carry;
            let word_comparison = (left as u64).cmp(&(right as u64));
            if word_comparison != Ordering::Equal {
                comparison = word_comparison;
            }
            left_carry = left >> 64;
            right_carry = right >> 64;
        }
        debug_assert_eq!((left_carry, right_carry), (0, 0));
        comparison
    }
}

fn add_word(words: &mut [u64; WORDS], mut index: usize, mut word: u64) {
    while word != 0 {
        let (sum, carry) = words[index].overflowing_add(word);
        words[index] = sum;
        word = u64::from(carry);
        index += 1;
    }
}
