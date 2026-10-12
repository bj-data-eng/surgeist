//! Exact natural dimensions and private bounded decimal arithmetic.
//! The pinned HTML [dimension attributes](https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#dimension-attributes)
//! consumer uses resource-qualified intrinsic CSS-pixel quantities. Exact
//! rationals are this API's supported domain, not a universal measurement rule;
//! this owner introduces no geometry or resource loader.

use super::HtmlNonNegativeInteger;
use std::{cmp::Ordering, error::Error, fmt};

/// An exact nonnegative rational. Construction and cloning use ordinary
/// allocation outside semantic-session admission. Positive fractions need not
/// be reduced; structural equality is deliberately not mathematical equality.
#[derive(Clone, Debug)]
pub struct HtmlNonNegativeRational {
    numerator: HtmlNonNegativeInteger,
    denominator: HtmlNonNegativeInteger,
}
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum HtmlRationalError {
    ZeroDenominator,
}
impl fmt::Display for HtmlRationalError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{self:?}")
    }
}
impl Error for HtmlRationalError {}
impl HtmlNonNegativeRational {
    pub fn new(
        numerator: HtmlNonNegativeInteger,
        denominator: HtmlNonNegativeInteger,
    ) -> Result<Self, HtmlRationalError> {
        if denominator.is_zero() {
            return Err(HtmlRationalError::ZeroDenominator);
        }
        let denominator = if numerator.is_zero() {
            1u128.into()
        } else {
            denominator
        };
        Ok(Self {
            numerator,
            denominator,
        })
    }
    pub fn numerator(&self) -> &HtmlNonNegativeInteger {
        &self.numerator
    }
    pub fn denominator(&self) -> &HtmlNonNegativeInteger {
        &self.denominator
    }
    pub(crate) fn retained_bytes(&self) -> Option<usize> {
        self.numerator
            .digits
            .capacity()
            .checked_add(self.denominator.digits.capacity())
    }
}

/// Borrowed native ASCII or admitted little-endian numeric digits. Neither
/// representation projects a magnitude into a machine integer.
#[derive(Clone, Copy)]
pub(crate) enum NaturalDigits<'a> {
    Integer(&'a HtmlNonNegativeInteger),
    Little(&'a [u8]),
}
impl NaturalDigits<'_> {
    pub(crate) fn len(self) -> usize {
        match self {
            Self::Integer(v) => v.digits.len(),
            Self::Little(v) => v.len(),
        }
    }
    fn digit(self, i: usize) -> u8 {
        match self {
            Self::Integer(v) => v.digits.as_bytes()[v.digits.len() - 1 - i] - b'0',
            Self::Little(v) => v[i],
        }
    }
}

/// Each step initializes one cell, multiplies one cell, writes one carry, or
/// trims one zero. Workspace allocation/admission is the consuming session's
/// responsibility. A cell accumulator is at most 9+81+9=99.
pub(crate) struct NaturalProduct<'a> {
    left: NaturalDigits<'a>,
    right: NaturalDigits<'a>,
    initialized: usize,
    row: usize,
    column: usize,
    carry: u8,
}
fn natural_product_capacity(left: usize, right: usize) -> Option<usize> {
    left.checked_add(right)
        .filter(|&length| length <= isize::MAX as usize)
}
impl<'a> NaturalProduct<'a> {
    pub(crate) fn new(left: NaturalDigits<'a>, right: NaturalDigits<'a>) -> Self {
        Self {
            left,
            right,
            initialized: 0,
            row: 0,
            column: 0,
            carry: 0,
        }
    }
    pub(crate) fn capacity(&self) -> Option<usize> {
        natural_product_capacity(self.left.len(), self.right.len())
    }
    pub(crate) fn step(&mut self, output: &mut Vec<u8>) -> bool {
        let capacity = self.capacity().expect("caller checked product length");
        if self.initialized < capacity {
            output.push(0);
            self.initialized += 1;
            return false;
        }
        if self.row < self.left.len() {
            if self.column < self.right.len() {
                let index = self.row + self.column;
                let n = output[index]
                    + self.left.digit(self.row) * self.right.digit(self.column)
                    + self.carry;
                output[index] = n % 10;
                self.carry = n / 10;
                self.column += 1;
            } else {
                output[self.row + self.right.len()] = self.carry;
                self.carry = 0;
                self.column = 0;
                self.row += 1;
            }
            return false;
        }
        if output.len() > 1 && output.last() == Some(&0) {
            output.pop();
            false
        } else {
            true
        }
    }
}

pub(crate) struct NaturalComparison<'a> {
    left: NaturalDigits<'a>,
    right: NaturalDigits<'a>,
    next: usize,
}
impl<'a> NaturalComparison<'a> {
    pub(crate) fn new(left: NaturalDigits<'a>, right: NaturalDigits<'a>) -> Self {
        Self {
            left,
            right,
            next: left.len(),
        }
    }
    pub(crate) fn step(&mut self) -> Option<Ordering> {
        if self.left.len() != self.right.len() {
            return Some(self.left.len().cmp(&self.right.len()));
        }
        if self.next == 0 {
            return Some(Ordering::Equal);
        }
        self.next -= 1;
        let cmp = self.left.digit(self.next).cmp(&self.right.digit(self.next));
        (cmp != Ordering::Equal).then_some(cmp)
    }
}
pub(crate) struct NaturalDifference<'a> {
    large: NaturalDigits<'a>,
    small: NaturalDigits<'a>,
    next: usize,
    borrow: i16,
}
impl<'a> NaturalDifference<'a> {
    /// The consumer's admitted exact comparison proves large >= small.
    pub(crate) fn new(large: NaturalDigits<'a>, small: NaturalDigits<'a>) -> Self {
        Self {
            large,
            small,
            next: 0,
            borrow: 0,
        }
    }
    pub(crate) fn step(&mut self, output: &mut Vec<u8>) -> bool {
        if self.next < self.large.len() {
            let n = i16::from(self.large.digit(self.next))
                - if self.next < self.small.len() {
                    i16::from(self.small.digit(self.next))
                } else {
                    0
                }
                - self.borrow;
            self.borrow = i16::from(n < 0);
            output.push(n.rem_euclid(10) as u8);
            self.next += 1;
            return false;
        }
        debug_assert_eq!(self.borrow, 0);
        if output.len() > 1 && output.last() == Some(&0) {
            output.pop();
            false
        } else {
            true
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::DocumentString;
    fn integer(v: &str) -> HtmlNonNegativeInteger {
        HtmlNonNegativeInteger::from_decimal_digits(&DocumentString::from(v)).unwrap()
    }
    fn text(v: &[u8]) -> String {
        v.iter().rev().map(|v| char::from(b'0' + v)).collect()
    }
    #[test]
    fn workspace_length_preflight_checks_native_representation_without_claiming_allocator_failure()
    {
        let max = isize::MAX as usize;
        assert_eq!(natural_product_capacity(max - 1, 1), Some(max));
        assert_eq!(natural_product_capacity(max, 1), None);
        assert_eq!(natural_product_capacity(usize::MAX, 1), None);
        assert_eq!(natural_product_capacity(3, 4), Some(7));
    }
    #[test]
    fn arbitrary_products_have_literal_carry_and_length_oracles() {
        for (a, b, expected) in [
            ("0", "98765", "0"),
            ("99", "99", "9801"),
            ("9999999999", "9999999999", "99999999980000000001"),
            ("123456789", "987654321", "121932631112635269"),
            (
                "100000000000000000000",
                "100000000000000000000",
                "10000000000000000000000000000000000000000",
            ),
        ] {
            let (a, b) = (integer(a), integer(b));
            let mut cursor =
                NaturalProduct::new(NaturalDigits::Integer(&a), NaturalDigits::Integer(&b));
            let mut out = Vec::with_capacity(cursor.capacity().unwrap());
            let mut steps = 0;
            while !cursor.step(&mut out) {
                steps += 1;
                assert!(steps < 10000);
            }
            assert_eq!(text(&out), expected);
        }
    }
    #[test]
    fn comparisons_and_subtractions_preserve_all_digits_and_borrow_chains() {
        for (a, b, expected) in [
            ("100000000000000000000", "1", "99999999999999999999"),
            ("12345", "12345", "0"),
            ("123456789012345678901", "123456789012345678900", "1"),
        ] {
            let (a, b) = (integer(a), integer(b));
            let mut compare =
                NaturalComparison::new(NaturalDigits::Integer(&a), NaturalDigits::Integer(&b));
            let actual = loop {
                if let Some(v) = compare.step() {
                    break v;
                }
            };
            assert_ne!(actual, Ordering::Less);
            let mut cursor =
                NaturalDifference::new(NaturalDigits::Integer(&a), NaturalDigits::Integer(&b));
            let mut out = Vec::with_capacity(a.digits.len());
            while !cursor.step(&mut out) {}
            assert_eq!(text(&out), expected);
        }
    }
}
