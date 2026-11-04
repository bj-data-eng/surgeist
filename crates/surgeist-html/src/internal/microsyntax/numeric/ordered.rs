//! Subtraction of exactly ordered magnitudes.
//!
//! A difference exists only between a larger and a smaller magnitude, so the operation takes the
//! pair that an exact comparison produced instead of two operands it would have to clamp. The
//! pair's fields are private to this module and only [`Order::of`] builds one, so a caller cannot
//! hand the subtraction an operand pair outside its domain.

use super::{HtmlNonNegativeDecimal, HtmlNonNegativeInteger, ascii_digits, trim_trailing_zeroes};
use std::cmp::Ordering;

/// The exact order of two magnitudes. A pair that differs carries the gap between its members.
pub(super) enum Order<'a, T> {
    /// The magnitudes are equal; their difference is zero.
    Equal,
    /// The first magnitude is the larger one.
    FirstLarger(Gap<'a, T>),
    /// The second magnitude is the larger one.
    SecondLarger(Gap<'a, T>),
}

/// A larger magnitude and a strictly smaller one.
pub(super) struct Gap<'a, T> {
    larger: &'a T,
    smaller: &'a T,
}

/// A magnitude with an exact order and an exact difference between a larger and a smaller one.
pub(super) trait Magnitude {
    fn order(&self, other: &Self) -> Ordering;
    /// `larger - smaller`, for `larger` strictly greater than `smaller`.
    fn gap(larger: &Self, smaller: &Self) -> Self;
}

impl<'a, T: Magnitude> Order<'a, T> {
    pub(super) fn of(first: &'a T, second: &'a T) -> Self {
        match first.order(second) {
            Ordering::Equal => Self::Equal,
            Ordering::Greater => Self::FirstLarger(Gap {
                larger: first,
                smaller: second,
            }),
            Ordering::Less => Self::SecondLarger(Gap {
                larger: second,
                smaller: first,
            }),
        }
    }
}

impl<T: Magnitude> Gap<'_, T> {
    /// The exact positive difference.
    pub(super) fn difference(&self) -> T {
        T::gap(self.larger, self.smaller)
    }
}

impl Magnitude for HtmlNonNegativeInteger {
    fn order(&self, other: &Self) -> Ordering {
        self.compare(other)
    }
    fn gap(larger: &Self, smaller: &Self) -> Self {
        Self {
            digits: difference_string(subtract_digits(
                larger.digits.as_bytes(),
                smaller.digits.as_bytes(),
                0,
            )),
        }
    }
}

impl Magnitude for HtmlNonNegativeDecimal {
    fn order(&self, other: &Self) -> Ordering {
        self.compare(other)
    }
    /// The fraction borrows from the integer part exactly when the smaller fraction exceeds the
    /// larger one, which the larger decimal's order covers.
    fn gap(larger: &Self, smaller: &Self) -> Self {
        let scale = larger.fraction.len().max(smaller.fraction.len());
        let mut fraction = vec![b'0'; scale];
        let mut borrow = 0;
        for (index, place) in fraction.iter_mut().enumerate().rev() {
            let left = larger
                .fraction
                .as_bytes()
                .get(index)
                .copied()
                .unwrap_or(b'0');
            let right = smaller
                .fraction
                .as_bytes()
                .get(index)
                .copied()
                .unwrap_or(b'0');
            let difference = i16::from(left - b'0') - i16::from(right - b'0') - i16::from(borrow);
            borrow = u8::from(difference < 0);
            *place = b'0' + difference.rem_euclid(10) as u8;
        }
        trim_trailing_zeroes(&mut fraction);
        Self {
            integer: HtmlNonNegativeInteger {
                digits: difference_string(subtract_digits(
                    larger.integer.digits.as_bytes(),
                    smaller.integer.digits.as_bytes(),
                    borrow,
                )),
            },
            fraction: fraction_string(fraction),
        }
    }
}

/// The string of a fraction buffer, with the buffer's capacity.
fn fraction_string(digits: Vec<u8>) -> String {
    ascii_digits(&digits, digits.capacity())
}

/// The string of a difference's integer digits, with exactly their length as capacity.
fn difference_string(digits: Vec<u8>) -> String {
    ascii_digits(&digits, digits.len())
}

/// The canonical digits of `larger - smaller - borrow`, most significant first.
///
/// Canonical digits have no leading zeroes, so the larger magnitude is at least as long as the
/// smaller one and its digits cover the smaller one's. Only a [`Gap`] reaches this function: the
/// larger magnitude exceeds the smaller one, and with a fraction's borrow it still covers
/// `smaller + borrow`, so the subtraction leaves no borrow past the most significant digit.
fn subtract_digits(larger: &[u8], smaller: &[u8], mut borrow: u8) -> Vec<u8> {
    let mut right = smaller.iter().rev();
    let mut reversed = Vec::with_capacity(larger.len());
    for digit in larger.iter().rev() {
        let difference = i16::from(digit - b'0')
            - i16::from(right.next().map_or(0, |digit| digit - b'0'))
            - i16::from(borrow);
        borrow = u8::from(difference < 0);
        reversed.push(b'0' + difference.rem_euclid(10) as u8);
    }
    while reversed.len() > 1 && reversed.last() == Some(&b'0') {
        reversed.pop();
    }
    reversed.reverse();
    reversed
}

#[cfg(test)]
mod tests {
    use super::*;

    fn integer(value: u128) -> HtmlNonNegativeInteger {
        HtmlNonNegativeInteger::from(value)
    }

    /// The decimal `whole.fraction`, with no fractional digits when `fraction` is empty.
    fn decimal(whole: u128, fraction: &str) -> HtmlNonNegativeDecimal {
        if fraction.is_empty() {
            return integer(whole).into();
        }
        HtmlNonNegativeDecimal::with_fraction_digits(
            integer(whole),
            &crate::internal::values::DocumentString::from(fraction),
        )
        .unwrap()
    }

    /// Thousandths in `whole.fraction`, for fractions of at most three digits.
    fn milli(whole: u128, fraction: &str) -> u128 {
        let padded = format!("{fraction:0<3}");
        whole * 1000 + padded.parse::<u128>().unwrap()
    }

    /// Independent oracle: ordinary machine arithmetic on values small enough to fit it.
    #[test]
    fn integer_gaps_match_machine_subtraction() {
        let samples = [
            0_u128,
            1,
            5,
            9,
            10,
            11,
            99,
            100,
            101,
            999,
            1000,
            123_456_789,
            123_456_790,
            u128::MAX,
        ];
        for &left in &samples {
            for &right in &samples {
                let (left_integer, right_integer) = (integer(left), integer(right));
                match Order::of(&left_integer, &right_integer) {
                    Order::Equal => assert_eq!(left, right, "{left} {right}"),
                    Order::FirstLarger(gap) => {
                        assert!(left > right, "{left} {right}");
                        assert_eq!(
                            gap.difference().decimal_digits(),
                            (left - right).to_string(),
                            "{left} - {right}"
                        );
                    }
                    Order::SecondLarger(gap) => {
                        assert!(right > left, "{left} {right}");
                        assert_eq!(
                            gap.difference().decimal_digits(),
                            (right - left).to_string(),
                            "{right} - {left}"
                        );
                    }
                }
            }
        }
    }

    /// Independent oracle: decimals of at most three fractional digits as integer thousandths.
    /// This reaches the borrow from the fraction into the integer part, and a borrow through
    /// zeroes.
    #[test]
    fn decimal_gaps_match_machine_subtraction_of_thousandths() {
        let samples = [
            (0, ""),
            (0, "5"),
            (1, ""),
            (1, "05"),
            (1, "5"),
            (2, "001"),
            (9, "999"),
            (10, ""),
            (10, "001"),
            (100, "5"),
            (101, ""),
        ];
        for &(left_whole, left_fraction) in &samples {
            for &(right_whole, right_fraction) in &samples {
                let (left, right) = (
                    decimal(left_whole, left_fraction),
                    decimal(right_whole, right_fraction),
                );
                let (left_milli, right_milli) = (
                    milli(left_whole, left_fraction),
                    milli(right_whole, right_fraction),
                );
                let (gap, larger, smaller) = match Order::of(&left, &right) {
                    Order::Equal => {
                        assert_eq!(left_milli, right_milli);
                        continue;
                    }
                    Order::FirstLarger(gap) => (gap, left_milli, right_milli),
                    Order::SecondLarger(gap) => (gap, right_milli, left_milli),
                };
                assert!(larger > smaller);
                let difference = gap.difference();
                assert!(!difference.fraction.ends_with('0'), "canonical fraction");
                assert_eq!(
                    milli(
                        difference.integer.decimal_digits().parse().unwrap(),
                        &difference.fraction
                    ),
                    larger - smaller,
                    "{left_whole}.{left_fraction} and {right_whole}.{right_fraction}"
                );
            }
        }
    }
}
