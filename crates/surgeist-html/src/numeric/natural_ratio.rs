//! Exact natural dimensions and private bounded decimal arithmetic.
//! The pinned HTML [dimension attributes](https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#dimension-attributes)
//! consumer uses resource-qualified intrinsic CSS-pixel quantities. Exact
//! rationals are this API's supported domain, not a universal measurement rule;
//! this owner introduces no geometry or resource loader.

use super::{
    HtmlNonNegativeInteger, IntegerUnitStepError, NumericMaterializationFailure,
    NumericMaterialized,
};
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
    pub fn retained_bytes(&self) -> Option<usize> {
        self.numerator
            .digits
            .capacity()
            .checked_add(self.denominator.digits.capacity())
    }

    /// Stored representation equality, deliberately not fraction reduction.
    pub fn same_representation(&self, other: &Self) -> bool {
        self.numerator == other.numerator && self.denominator == other.denominator
    }

    pub fn copy_work_bound(&self) -> Option<usize> {
        self.numerator
            .copy_work_bound()?
            .checked_add(self.denominator.copy_work_bound()?)
    }

    /// Copies both admitted canonical integers. Failure retains the cumulative
    /// actual capacity, including a numerator copied before denominator failure.
    pub fn try_copy(
        &self,
        max_digits: usize,
        max_bytes: usize,
    ) -> Result<NumericMaterialized<Self>, NumericMaterializationFailure> {
        let numerator = self.numerator.try_copy(max_digits, max_bytes)?;
        let denominator = self
            .denominator
            .try_copy(max_digits, max_bytes - numerator.capacity_bytes)
            .map_err(|mut error| {
                error.capacity_bytes += numerator.capacity_bytes;
                match &mut error.kind {
                    super::NumericMaterializationError::Boundary(
                        IntegerUnitStepError::RequestedCapacityLimitExceeded { limit, required },
                    ) => {
                        *limit = max_bytes;
                        *required += numerator.capacity_bytes;
                    }
                    super::NumericMaterializationError::Boundary(
                        IntegerUnitStepError::ActualCapacityLimitExceeded { limit, .. },
                    ) => {
                        *limit = max_bytes;
                    }
                    _ => {}
                }
                error
            })?;
        Ok(NumericMaterialized {
            capacity_bytes: numerator.capacity_bytes + denominator.capacity_bytes,
            value: Self {
                numerator: numerator.value,
                denominator: denominator.value,
            },
        })
    }

    pub fn integer_work_bound() -> Option<usize> {
        HtmlNonNegativeInteger::from_u32_work_bound()
    }

    /// Takes an already admitted integer and materializes only denominator one.
    /// Unlike ordinary `new`, zero does not allocate a discarded replacement.
    pub fn try_from_integer(
        numerator: HtmlNonNegativeInteger,
        max_digits: usize,
        max_bytes: usize,
    ) -> Result<NumericMaterialized<Self>, RationalIntegerMaterializationFailure> {
        let denominator = match HtmlNonNegativeInteger::try_from_u32(1, max_digits, max_bytes) {
            Ok(value) => value,
            Err(failure) => {
                return Err(RationalIntegerMaterializationFailure { failure, numerator });
            }
        };
        Ok(NumericMaterialized {
            capacity_bytes: denominator.capacity_bytes,
            value: Self {
                numerator,
                denominator: denominator.value,
            },
        })
    }
}

/// A refused rational assembly retains its exact original numerator and every
/// observed denominator-allocation byte. Readback does not copy digit storage;
/// consuming this error returns both owners for explicit cleanup or retry.
#[derive(Debug)]
pub struct RationalIntegerMaterializationFailure {
    failure: NumericMaterializationFailure,
    numerator: HtmlNonNegativeInteger,
}
impl RationalIntegerMaterializationFailure {
    pub fn numerator(&self) -> &HtmlNonNegativeInteger {
        &self.numerator
    }
    pub fn failure(&self) -> &NumericMaterializationFailure {
        &self.failure
    }
    pub fn capacity_bytes(&self) -> usize {
        self.failure.capacity_bytes()
    }
    pub fn kind(&self) -> super::NumericMaterializationError {
        self.failure.kind()
    }
    pub fn into_parts(self) -> (NumericMaterializationFailure, HtmlNonNegativeInteger) {
        (self.failure, self.numerator)
    }
}
impl fmt::Display for RationalIntegerMaterializationFailure {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::Display::fmt(&self.failure, f)
    }
}
impl Error for RationalIntegerMaterializationFailure {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        Some(&self.failure)
    }
}

/// An immutable checked operand: a native integer or canonical little-endian
/// decimal digits. A completed arithmetic cursor can lend its actual scratch
/// directly, without another validation pass.
#[derive(Clone, Copy, Debug)]
pub struct NaturalDigits<'a> {
    repr: NaturalDigitsRepr<'a>,
}
#[derive(Clone, Copy, Debug)]
enum NaturalDigitsRepr<'a> {
    Integer(&'a HtmlNonNegativeInteger),
    Little(&'a [u8]),
}
impl<'a> NaturalDigits<'a> {
    pub fn integer(value: &'a HtmlNonNegativeInteger) -> Self {
        Self {
            repr: NaturalDigitsRepr::Integer(value),
        }
    }
    /// Validate external scratch once. Zero is `[0]`; other operands have a
    /// nonzero last digit. No allocation or machine-integer projection occurs.
    pub fn try_little(value: &'a [u8]) -> Result<Self, NaturalArithmeticError> {
        if value.is_empty() || (value.len() > 1 && value.last() == Some(&0)) {
            return Err(NaturalArithmeticError::NonCanonical);
        }
        if let Some((index, &value)) = value.iter().enumerate().find(|(_, d)| **d > 9) {
            return Err(NaturalArithmeticError::InvalidDigit { index, value });
        }
        Ok(Self {
            repr: NaturalDigitsRepr::Little(value),
        })
    }
    /// Pre-execution envelope for the single external-digit validation pass.
    pub fn validation_work_bound(length: usize) -> Option<usize> {
        length.checked_add(8)
    }
    pub fn len(self) -> usize {
        match self.repr {
            NaturalDigitsRepr::Integer(v) => v.digits.len(),
            NaturalDigitsRepr::Little(v) => v.len(),
        }
    }
    /// Checked operands always contain at least their zero digit.
    pub const fn is_empty(self) -> bool {
        false
    }
    fn digit(self, i: usize) -> u8 {
        match self.repr {
            NaturalDigitsRepr::Integer(v) => v.digits.as_bytes()[v.digits.len() - 1 - i] - b'0',
            NaturalDigitsRepr::Little(v) => v[i],
        }
    }
}
/// Invalid external operands, unadmitted scratch, or premature completion.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum NaturalArithmeticError {
    NonCanonical,
    InvalidDigit { index: usize, value: u8 },
    ScratchNotEmpty,
    ScratchCapacity { required: usize, capacity: usize },
    RepresentationExhausted,
    NotReady,
    EmptySlot,
}
impl fmt::Display for NaturalArithmeticError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{self:?}")
    }
}
impl Error for NaturalArithmeticError {}
fn check_scratch(output: &Vec<u8>, required: usize) -> Result<(), NaturalArithmeticError> {
    if !output.is_empty() {
        return Err(NaturalArithmeticError::ScratchNotEmpty);
    }
    if output.capacity() < required {
        return Err(NaturalArithmeticError::ScratchCapacity {
            required,
            capacity: output.capacity(),
        });
    }
    Ok(())
}

/// Each step initializes one cell, multiplies one cell, writes one carry, or
/// trims one zero. Workspace allocation/admission is the consuming session's
/// responsibility. A cell accumulator is at most 9+81+9=99.
pub(crate) struct NaturalProductKernel<'a> {
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
impl<'a> NaturalProductKernel<'a> {
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

pub struct NaturalComparison<'a> {
    left: NaturalDigits<'a>,
    right: NaturalDigits<'a>,
    next: usize,
    result: Option<Ordering>,
}
impl<'a> NaturalComparison<'a> {
    pub fn new(left: NaturalDigits<'a>, right: NaturalDigits<'a>) -> Self {
        Self {
            left,
            right,
            next: left.len(),
            result: None,
        }
    }
    /// Reads at most one digit from each operand; terminal replies are stable.
    pub fn step(&mut self) -> Option<Ordering> {
        if let Some(result) = self.result {
            return Some(result);
        }
        let result = if self.left.len() != self.right.len() {
            Some(self.left.len().cmp(&self.right.len()))
        } else if self.next == 0 {
            Some(Ordering::Equal)
        } else {
            self.next -= 1;
            let cmp = self.left.digit(self.next).cmp(&self.right.digit(self.next));
            (cmp != Ordering::Equal).then_some(cmp)
        };
        self.result = result;
        result
    }
    pub fn result(&self) -> Option<Ordering> {
        self.result
    }
    /// Exact maximum scratch length for the absolute difference, available
    /// before comparison. Allocation and its admission belong to the caller.
    pub fn difference_capacity(&self) -> usize {
        self.left.len().max(self.right.len())
    }
    /// Uses this completed comparison's actual immutable operands; no second
    /// comparison or unchecked large-minus-small claim crosses the boundary.
    /// Premature transfer and scratch refusal leave the original slot intact.
    pub fn take_difference<'scratch>(
        slot: &mut Option<Self>,
        output: &'scratch mut Vec<u8>,
    ) -> Result<NaturalDifference<'a, 'scratch>, NaturalArithmeticError> {
        let comparison = slot.as_ref().ok_or(NaturalArithmeticError::EmptySlot)?;
        let ordering = comparison.result.ok_or(NaturalArithmeticError::NotReady)?;
        let (large, small) = if ordering == Ordering::Less {
            (comparison.right, comparison.left)
        } else {
            (comparison.left, comparison.right)
        };
        check_scratch(output, large.len())?;
        // All fallible checks precede the sole ownership transfer.
        let _completed = slot
            .take()
            .expect("shared preflight proved a completed comparison");
        Ok(NaturalDifference {
            kernel: NaturalDifferenceKernel::new(large, small),
            output,
            finished: false,
        })
    }
}
pub(crate) struct NaturalDifferenceKernel<'a> {
    large: NaturalDigits<'a>,
    small: NaturalDigits<'a>,
    next: usize,
    borrow: i16,
}
impl<'a> NaturalDifferenceKernel<'a> {
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

/// Bounded multiplication over checked operands and caller-reserved scratch.
/// Exclusive borrowing prevents workspace replacement between admitted steps.
pub struct NaturalProduct<'input, 'scratch> {
    kernel: NaturalProductKernel<'input>,
    output: &'scratch mut Vec<u8>,
    finished: bool,
}
impl<'input, 'scratch> NaturalProduct<'input, 'scratch> {
    pub fn required_capacity(left: NaturalDigits<'_>, right: NaturalDigits<'_>) -> Option<usize> {
        natural_product_capacity(left.len(), right.len())
    }
    pub fn new(
        left: NaturalDigits<'input>,
        right: NaturalDigits<'input>,
        output: &'scratch mut Vec<u8>,
    ) -> Result<Self, NaturalArithmeticError> {
        let capacity = Self::required_capacity(left, right)
            .ok_or(NaturalArithmeticError::RepresentationExhausted)?;
        check_scratch(output, capacity)?;
        Ok(Self {
            kernel: NaturalProductKernel::new(left, right),
            output,
            finished: false,
        })
    }
    /// One initialization, multiplication, carry write or trimming step.
    /// Admit fixed scalar work and at most one scratch-cell write beforehand.
    pub fn step(&mut self) -> bool {
        if !self.finished {
            self.finished = self.kernel.step(self.output);
        }
        self.finished
    }
    pub fn scratch(&self) -> &[u8] {
        self.output
    }
    pub fn is_finished(&self) -> bool {
        self.finished
    }
    /// Refusal preserves the exact cursor and its exclusive scratch borrow.
    /// Only a completed cursor is removed and lends its canonical result.
    pub fn take(
        slot: &mut Option<Self>,
    ) -> Result<NaturalDigits<'scratch>, NaturalArithmeticError> {
        let cursor = slot.as_ref().ok_or(NaturalArithmeticError::EmptySlot)?;
        if !cursor.finished {
            return Err(NaturalArithmeticError::NotReady);
        }
        let cursor = slot
            .take()
            .expect("shared preflight proved a completed cursor");
        Ok(NaturalDigits {
            repr: NaturalDigitsRepr::Little(cursor.output.as_slice()),
        })
    }
}
/// Absolute difference authorized by a completed immutable comparison.
/// Each step writes one subtraction digit or trims one high zero.
pub struct NaturalDifference<'input, 'scratch> {
    kernel: NaturalDifferenceKernel<'input>,
    output: &'scratch mut Vec<u8>,
    finished: bool,
}
impl<'scratch> NaturalDifference<'_, 'scratch> {
    pub fn step(&mut self) -> bool {
        if !self.finished {
            self.finished = self.kernel.step(self.output);
        }
        self.finished
    }
    pub fn scratch(&self) -> &[u8] {
        self.output
    }
    pub fn is_finished(&self) -> bool {
        self.finished
    }
    /// Refusal preserves the exact cursor and its exclusive scratch borrow.
    /// Only a completed cursor is removed and lends its canonical result.
    pub fn take(
        slot: &mut Option<Self>,
    ) -> Result<NaturalDigits<'scratch>, NaturalArithmeticError> {
        let cursor = slot.as_ref().ok_or(NaturalArithmeticError::EmptySlot)?;
        if !cursor.finished {
            return Err(NaturalArithmeticError::NotReady);
        }
        let cursor = slot
            .take()
            .expect("shared preflight proved a completed cursor");
        Ok(NaturalDigits {
            repr: NaturalDigitsRepr::Little(cursor.output.as_slice()),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::DocumentString;
    fn integer(v: &str) -> HtmlNonNegativeInteger {
        HtmlNonNegativeInteger::from_decimal_digits(&DocumentString::from(v)).unwrap()
    }
    #[test]
    fn controlled_rational_copy_preserves_arbitrary_unreduced_digits_and_capacity() {
        let r = HtmlNonNegativeRational::new(
            integer("340282366920938463463374607431768211456"),
            integer("10"),
        )
        .unwrap();
        let c = r.try_copy(100, 1000).unwrap();
        assert!(r.same_representation(&c.value));
        assert!(c.capacity_bytes >= 41);
        assert!(
            !HtmlNonNegativeRational::new(integer("2"), integer("4"))
                .unwrap()
                .same_representation(
                    &HtmlNonNegativeRational::new(integer("1"), integer("2")).unwrap()
                )
        );
    }
    #[test]
    fn denominator_failure_retains_successful_numerator_capacity_and_global_request() {
        let r = HtmlNonNegativeRational::new(integer("12345"), integer("67")).unwrap();
        let failure = r.try_copy(5, 5).unwrap_err();
        assert_eq!(failure.capacity_bytes, 5);
        assert_eq!(
            failure.kind,
            super::super::NumericMaterializationError::Boundary(
                IntegerUnitStepError::RequestedCapacityLimitExceeded {
                    limit: 5,
                    required: 7
                }
            )
        );
        let failure = r.try_copy(1, 100).unwrap_err();
        assert_eq!(failure.capacity_bytes, 0);
        assert_eq!(
            failure.kind,
            super::super::NumericMaterializationError::Boundary(
                IntegerUnitStepError::DigitLimitExceeded {
                    limit: 1,
                    required: 5
                }
            )
        );
    }
    #[test]
    fn controlled_integer_assembly_allocates_only_one_positive_denominator_even_for_zero() {
        for digits in ["0", "123456789012345678901234567890123456789"] {
            let r = HtmlNonNegativeRational::try_from_integer(integer(digits), 100, 1).unwrap();
            assert_eq!(r.capacity_bytes, 1);
            assert_eq!(r.value.numerator().decimal_digits(), digits);
            assert_eq!(r.value.denominator().decimal_digits(), "1");
        }
        let e = HtmlNonNegativeRational::try_from_integer(integer("0"), 1, 0).unwrap_err();
        assert_eq!(e.capacity_bytes(), 0);
        assert!(matches!(
            e.kind(),
            super::super::NumericMaterializationError::Boundary(
                IntegerUnitStepError::RequestedCapacityLimitExceeded { required: 1, .. }
            )
        ));
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
                NaturalProductKernel::new(NaturalDigits::integer(&a), NaturalDigits::integer(&b));
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
                NaturalComparison::new(NaturalDigits::integer(&a), NaturalDigits::integer(&b));
            let actual = loop {
                if let Some(v) = compare.step() {
                    break v;
                }
            };
            assert_ne!(actual, Ordering::Less);
            let mut cursor = NaturalDifferenceKernel::new(
                NaturalDigits::integer(&a),
                NaturalDigits::integer(&b),
            );
            let mut out = Vec::with_capacity(a.digits.len());
            while !cursor.step(&mut out) {}
            assert_eq!(text(&out), expected);
        }
    }
}
