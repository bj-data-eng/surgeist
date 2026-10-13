//! Intrinsic numeric values from the pinned HTML common microsyntaxes.
//!
//! Author validation and user-agent recovery are distinct entry points. Parsing
//! stays crate-private so the controlled interpretation owner can apply its
//! request limits before invoking these algorithms.

use crate::DocumentString;
use std::{cmp::Ordering, error::Error, fmt, num::NonZeroU32};
pub(crate) mod natural_ratio;
pub(crate) mod range;
pub use natural_ratio::*;

fn integer_sum_capacity(left_digits: usize, right_digits: usize) -> Option<usize> {
    left_digits
        .max(right_digits)
        .checked_add(1)
        .filter(|&len| len <= isize::MAX as usize / size_of::<char>())
}

// Shared decimal kernels. Callers select ordinary or fallible storage; these
// functions perform only bounded digit/carry work on already valid ASCII digits.
fn product_length(input: &[u8], multiplier: u32, mut carry: u64) -> usize {
    let mut count = input.len();
    for &digit in input.iter().rev() {
        carry = (u64::from(digit - b'0') * u64::from(multiplier) + carry) / 10;
    }
    while carry != 0 {
        count += 1;
        carry /= 10;
    }
    count
}

fn write_product(input: &[u8], multiplier: u32, mut carry: u64, output: &mut Vec<u8>) {
    for &digit in input.iter().rev() {
        let product = u64::from(digit - b'0') * u64::from(multiplier) + carry;
        output.push(b'0' + (product % 10) as u8);
        carry = product / 10;
    }
    while carry != 0 {
        output.push(b'0' + (carry % 10) as u8);
        carry /= 10;
    }
    output.reverse();
}

fn sum_carry(left: &[u8], right: &[u8], mut carry: u8) -> u8 {
    let mut left = left.iter().rev();
    let mut right = right.iter().rev();
    loop {
        let a = left.next();
        let b = right.next();
        if a.is_none() && b.is_none() {
            return carry;
        }
        carry =
            (a.map_or(0, |&digit| digit - b'0') + b.map_or(0, |&digit| digit - b'0') + carry) / 10;
    }
}

fn write_sum(left: &[u8], right: &[u8], mut carry: u8, output: &mut Vec<u8>) {
    let mut left = left.iter().rev();
    let mut right = right.iter().rev();
    loop {
        let a = left.next();
        let b = right.next();
        if a.is_none() && b.is_none() && carry == 0 {
            break;
        }
        let sum = a.map_or(0, |&digit| digit - b'0') + b.map_or(0, |&digit| digit - b'0') + carry;
        output.push(b'0' + sum % 10);
        carry = sum / 10;
    }
    output.reverse();
}

fn write_fraction_sum(left: &[u8], right: &[u8], output: &mut [u8]) -> u8 {
    let mut carry = 0;
    for index in (0..output.len()).rev() {
        let sum = left.get(index).map_or(0, |&digit| digit - b'0')
            + right.get(index).map_or(0, |&digit| digit - b'0')
            + carry;
        output[index] = b'0' + sum % 10;
        carry = sum / 10;
    }
    carry
}

fn write_fraction_product(input: &[u8], multiplier: u32, output: &mut [u8]) -> u64 {
    let mut carry = 0;
    for (index, &digit) in input.iter().enumerate().rev() {
        let product = u64::from(digit - b'0') * u64::from(multiplier) + carry;
        output[index] = b'0' + (product % 10) as u8;
        carry = product / 10;
    }
    carry
}

pub(crate) fn remainder_digit(digit: u8, divisor: u64, remainder: u64) -> u64 {
    (remainder * 10 + u64::from(digit)) % divisor
}

/// Remainder over already validated ASCII decimal digits. Syntax and positive
/// year checks stay with the temporal recognizer; this performs one digit pass.
pub(crate) fn borrowed_decimal_remainder(input: &[u16], divisor: NonZeroU32) -> u32 {
    let divisor = u64::from(divisor.get());
    input.iter().fold(0, |remainder, &unit| {
        remainder_digit((unit - u16::from(b'0')) as u8, divisor, remainder)
    }) as u32
}

fn division_digit(digit: u8, divisor: u64, remainder: &mut u64) -> u8 {
    let partial = *remainder * 10 + u64::from(digit - b'0');
    *remainder = partial % divisor;
    (partial / divisor) as u8
}

fn write_quotient(input: &[u8], divisor: NonZeroU32, output: &mut Vec<u8>) -> u32 {
    let divisor = u64::from(divisor.get());
    let mut remainder = 0;
    for &digit in input {
        let digit = division_digit(digit, divisor, &mut remainder);
        if digit != 0 || !output.is_empty() {
            output.push(b'0' + digit);
        }
    }
    if output.is_empty() {
        output.push(b'0');
    }
    remainder as u32
}

fn ascii_digit_string(bytes: Vec<u8>) -> String {
    // Safe ownership transfer, with UTF-8 validation but no second allocation.
    String::from_utf8(bytes).expect("numeric owner writes only ASCII digits")
}

/// Observed cumulative capacity of every variable allocation made by one
/// concrete operation, including intermediates that have already been dropped.
/// The semantic session records this before interpreting either outcome.
#[derive(Debug)]
pub(crate) struct NumericMaterialized<T> {
    pub(crate) value: T,
    pub(crate) capacity_bytes: usize,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum NumericMaterializationError {
    Digits(HtmlDecimalDigitsError),
    Boundary(IntegerUnitStepError),
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct NumericMaterializationFailure {
    pub(crate) kind: NumericMaterializationError,
    pub(crate) capacity_bytes: usize,
}

fn materialization_boundary(
    error: IntegerUnitStepError,
    capacity_bytes: usize,
) -> NumericMaterializationFailure {
    NumericMaterializationFailure {
        kind: NumericMaterializationError::Boundary(error),
        capacity_bytes,
    }
}

fn materialization_digits(
    error: HtmlDecimalDigitsError,
    capacity_bytes: usize,
) -> NumericMaterializationFailure {
    NumericMaterializationFailure {
        kind: NumericMaterializationError::Digits(error),
        capacity_bytes,
    }
}

fn reserve_materialization(
    requested: usize,
    max_integer_digits: Option<usize>,
    max_capacity_bytes: usize,
    observed: &mut usize,
) -> Result<Vec<u8>, NumericMaterializationFailure> {
    if let Some(max_digits) = max_integer_digits {
        check_integer_digit_capacity(requested, max_digits, usize::MAX)
            .map_err(|e| materialization_boundary(e, *observed))?;
    } else if requested > isize::MAX as usize {
        return Err(materialization_boundary(
            IntegerUnitStepError::RepresentationExhausted,
            *observed,
        ));
    }
    let required = observed.checked_add(requested).ok_or_else(|| {
        materialization_boundary(IntegerUnitStepError::RepresentationExhausted, *observed)
    })?;
    if required > max_capacity_bytes {
        return Err(materialization_boundary(
            IntegerUnitStepError::RequestedCapacityLimitExceeded {
                limit: max_capacity_bytes,
                required,
            },
            *observed,
        ));
    }
    let mut bytes = Vec::new();
    let allocation = bytes.try_reserve_exact(requested);
    // Even an allocation failure retains any observed backing, without claiming
    // allocator internals or old immutable inputs as new variable storage.
    *observed = observed.checked_add(bytes.capacity()).ok_or_else(|| {
        materialization_boundary(IntegerUnitStepError::RepresentationExhausted, *observed)
    })?;
    allocation.map_err(|_| {
        materialization_boundary(
            IntegerUnitStepError::AllocationFailed { requested },
            *observed,
        )
    })?;
    check_integer_actual_capacity(required, *observed, max_capacity_bytes)
        .map_err(|e| materialization_boundary(e, *observed))?;
    Ok(bytes)
}

fn controlled_digit_string(
    input: &[u16],
    fractional: bool,
    max_integer_digits: usize,
    max_capacity_bytes: usize,
    observed: &mut usize,
) -> Result<String, NumericMaterializationFailure> {
    // An empty fraction is the exact integer case, not an invalid integer.
    if fractional && input.is_empty() {
        return Ok(String::new());
    }
    check_decimal_digits(input).map_err(|e| materialization_digits(e, *observed))?;
    let canonical = if fractional {
        let end = input
            .iter()
            .rposition(|&unit| unit != u16::from(b'0'))
            .map_or(0, |i| i + 1);
        &input[..end]
    } else {
        let first = input
            .iter()
            .position(|&unit| unit != u16::from(b'0'))
            .unwrap_or(input.len() - 1);
        &input[first..]
    };
    let mut bytes = reserve_materialization(
        canonical.len(),
        (!fractional).then_some(max_integer_digits),
        max_capacity_bytes,
        observed,
    )?;
    for &unit in canonical {
        bytes.push(unit as u8);
    }
    Ok(ascii_digit_string(bytes))
}

// These are loop-derived pre-execution envelopes, not instruction counts or
// cancellation quanta. Each checked sum includes the fixed branch/return work.
fn bounded_work(parts: &[usize]) -> Option<usize> {
    parts
        .iter()
        .try_fold(8usize, |sum, part| sum.checked_add(*part))
}

impl HtmlNonNegativeInteger {
    // Validation n, zero scan n, copy reads/writes n+n, UTF-8 n.
    pub(crate) fn from_borrowed_digits_work_bound(length: usize) -> Option<usize> {
        bounded_work(&[length, length, length, length, length])
    }
    pub(crate) fn try_from_borrowed_digits(
        input: &[u16],
        max_digits: usize,
        max_bytes: usize,
    ) -> Result<NumericMaterialized<Self>, NumericMaterializationFailure> {
        let mut observed = 0;
        let digits = controlled_digit_string(input, false, max_digits, max_bytes, &mut observed)?;
        Ok(NumericMaterialized {
            value: Self { digits },
            capacity_bytes: observed,
        })
    }
    // Existing fixed-width constant owner scans/outputs at most usize's decimal
    // width. On the supported 32/64-bit targets, four passes of 20 bound it.
    pub(crate) fn from_u32_work_bound() -> Option<usize> {
        bounded_work(&[20, 20, 20, 20])
    }
    pub(crate) fn try_from_u32(
        value: u32,
        max_digits: usize,
        max_bytes: usize,
    ) -> Result<NumericMaterialized<Self>, NumericMaterializationFailure> {
        let result = HtmlInteger::try_from_usize(value as usize, max_digits, max_bytes)
            .map_err(unit_step_materialization_failure)?;
        Ok(NumericMaterialized {
            value: result.value.magnitude,
            capacity_bytes: result.capacity_bytes,
        })
    }
    // Copy reads/writes n+n and UTF-8 n; native canonical input needs no zero scan.
    pub(crate) fn copy_work_bound(&self) -> Option<usize> {
        bounded_work(&[self.digits.len(), self.digits.len(), self.digits.len()])
    }
    pub(crate) fn try_copy(
        &self,
        max_digits: usize,
        max_bytes: usize,
    ) -> Result<NumericMaterialized<Self>, NumericMaterializationFailure> {
        let mut observed = 0;
        let mut bytes = reserve_materialization(
            self.digits.len(),
            Some(max_digits),
            max_bytes,
            &mut observed,
        )?;
        bytes.extend_from_slice(self.digits.as_bytes());
        Ok(NumericMaterialized {
            value: Self {
                digits: ascii_digit_string(bytes),
            },
            capacity_bytes: observed,
        })
    }
    // Division n input visits + n writes, UTF-8 n, canonical quotient length
    // prepass n. The prepass shares write_quotient's exact recurrence below.
    pub(crate) fn quotient_remainder_work_bound(&self) -> Option<usize> {
        let n = self.digits.len();
        bounded_work(&[n, n, n, n])
    }
    pub(crate) fn try_quotient_remainder(
        &self,
        divisor: NonZeroU32,
        max_digits: usize,
        max_bytes: usize,
    ) -> Result<NumericMaterialized<(Self, u32)>, NumericMaterializationFailure> {
        let mut remainder = 0u64;
        let mut length = 0;
        for &digit in self.digits.as_bytes() {
            let digit = division_digit(digit, u64::from(divisor.get()), &mut remainder);
            if digit != 0 || length != 0 {
                length += 1;
            }
        }
        let mut observed = 0;
        let mut bytes =
            reserve_materialization(length.max(1), Some(max_digits), max_bytes, &mut observed)?;
        let remainder = write_quotient(self.digits.as_bytes(), divisor, &mut bytes);
        Ok(NumericMaterialized {
            value: (
                Self {
                    digits: ascii_digit_string(bytes),
                },
                remainder,
            ),
            capacity_bytes: observed,
        })
    }
    // Pivot n, prefix-copy reads n, writes n; zero returns no allocation.
    pub(crate) fn decremented_work_bound(&self) -> Option<usize> {
        let n = self.digits.len();
        bounded_work(&[n, n, n])
    }
    pub(crate) fn try_decremented(
        &self,
        max_digits: usize,
        max_bytes: usize,
    ) -> Result<NumericMaterialized<Option<Self>>, NumericMaterializationFailure> {
        if self.is_zero() {
            return Ok(NumericMaterialized {
                value: None,
                capacity_bytes: 0,
            });
        }
        let digits = try_unit_step_digits(&self.digits, false, max_digits, max_bytes)
            .map_err(unit_step_materialization_failure)?;
        let capacity_bytes = digits.capacity();
        Ok(NumericMaterialized {
            value: Some(Self { digits }),
            capacity_bytes,
        })
    }
    // Length pass n+10; product reads/writes n+10 each; reversal reads
    // and writes at most n+10 each; UTF-8 validation n+10.
    pub(crate) fn multiplied_by_work_bound(&self) -> Option<usize> {
        let n = self.digits.len().checked_add(10)?;
        bounded_work(&[n, n, n, n, n, n])
    }
    pub(crate) fn try_multiplied_by(
        &self,
        multiplier: u32,
        max_digits: usize,
        max_bytes: usize,
    ) -> Result<NumericMaterialized<Self>, NumericMaterializationFailure> {
        if multiplier == 0 || self.is_zero() {
            return Self::try_from_u32(0, max_digits, max_bytes);
        }
        let mut observed = 0;
        let digits = controlled_product(
            self.digits.as_bytes(),
            multiplier,
            0,
            max_digits,
            max_bytes,
            &mut observed,
        )?;
        Ok(NumericMaterialized {
            value: Self { digits },
            capacity_bytes: observed,
        })
    }
    // Carry prepass and sum each read m+m. Output writes m+1; reversal
    // reads/writes m+1 each; UTF-8 validation m+1.
    pub(crate) fn added_to_work_bound(&self, other: &Self) -> Option<usize> {
        let m = self.digits.len().max(other.digits.len());
        let n = m.checked_add(1)?;
        bounded_work(&[m, m, m, m, n, n, n, n])
    }
    pub(crate) fn try_added_to(
        &self,
        other: &Self,
        max_digits: usize,
        max_bytes: usize,
    ) -> Result<NumericMaterialized<Self>, NumericMaterializationFailure> {
        let mut observed = 0;
        let digits = controlled_sum(
            self.digits.as_bytes(),
            other.digits.as_bytes(),
            0,
            max_digits,
            max_bytes,
            &mut observed,
        )?;
        Ok(NumericMaterialized {
            value: Self { digits },
            capacity_bytes: observed,
        })
    }
}

fn controlled_product(
    input: &[u8],
    multiplier: u32,
    carry: u64,
    max_digits: usize,
    max_bytes: usize,
    observed: &mut usize,
) -> Result<String, NumericMaterializationFailure> {
    input.len().checked_add(10).ok_or_else(|| {
        materialization_boundary(IntegerUnitStepError::RepresentationExhausted, *observed)
    })?;
    let required = product_length(input, multiplier, carry);
    let mut bytes = reserve_materialization(required, Some(max_digits), max_bytes, observed)?;
    write_product(input, multiplier, carry, &mut bytes);
    Ok(ascii_digit_string(bytes))
}

fn controlled_sum(
    left: &[u8],
    right: &[u8],
    carry: u8,
    max_digits: usize,
    max_bytes: usize,
    observed: &mut usize,
) -> Result<String, NumericMaterializationFailure> {
    let required = left
        .len()
        .max(right.len())
        .checked_add(usize::from(sum_carry(left, right, carry)))
        .ok_or_else(|| {
            materialization_boundary(IntegerUnitStepError::RepresentationExhausted, *observed)
        })?;
    let mut bytes = reserve_materialization(required, Some(max_digits), max_bytes, observed)?;
    write_sum(left, right, carry, &mut bytes);
    Ok(ascii_digit_string(bytes))
}

fn unit_step_shape(
    input: &str,
    grow: bool,
) -> Result<(usize, Option<usize>), IntegerUnitStepError> {
    let pivot = input
        .as_bytes()
        .iter()
        .rposition(|&digit| digit != if grow { b'9' } else { b'0' });
    let required = if grow && pivot.is_none() {
        input
            .len()
            .checked_add(1)
            .ok_or(IntegerUnitStepError::RepresentationExhausted)?
    } else if !grow && pivot == Some(0) && input.as_bytes()[0] == b'1' && input.len() > 1 {
        input.len() - 1
    } else {
        input.len()
    };
    Ok((required, pivot))
}

fn write_unit_step(input: &str, grow: bool, pivot: Option<usize>, digits: &mut String) {
    if grow {
        if let Some(pivot) = pivot {
            digits.push_str(&input[..pivot]);
            digits.push(char::from(input.as_bytes()[pivot] + 1));
            for _ in pivot + 1..input.len() {
                digits.push('0');
            }
        } else {
            digits.push('1');
            for _ in 0..input.len() {
                digits.push('0');
            }
        }
    } else {
        // The private magnitude invariant is canonical decimal, and zero
        // selected the growing branch above. A nonzero digit therefore exists.
        let pivot = pivot.expect("canonical nonzero magnitude has a nonzero digit");
        if pivot == 0 && input.as_bytes()[0] == b'1' && input.len() > 1 {
            for _ in 1..input.len() {
                digits.push('9');
            }
        } else {
            digits.push_str(&input[..pivot]);
            digits.push(char::from(input.as_bytes()[pivot] - 1));
            for _ in pivot + 1..input.len() {
                digits.push('9');
            }
        }
    }
}

fn try_unit_step_digits(
    input: &str,
    grow: bool,
    max_digits: usize,
    max_bytes: usize,
) -> Result<String, IntegerUnitStepError> {
    let (required, pivot) = unit_step_shape(input, grow)?;
    let mut digits = reserve_integer_digits(required, max_digits, max_bytes)?;
    write_unit_step(input, grow, pivot, &mut digits);
    Ok(digits)
}

fn unit_step_materialization_failure(error: IntegerUnitStepError) -> NumericMaterializationFailure {
    let capacity = match error {
        IntegerUnitStepError::ActualCapacityLimitExceeded { capacity_bytes, .. } => capacity_bytes,
        _ => 0,
    };
    materialization_boundary(error, capacity)
}

impl HtmlInteger {
    // Pivot scan n, prefix-copy reads n, writes n+1, fixed sign/zero handling.
    pub(crate) fn materialized_unit_step_work_bound(&self) -> Option<usize> {
        bounded_work(&[
            self.magnitude.digits.len(),
            self.magnitude.digits.len(),
            self.magnitude.digits.len().checked_add(1)?,
        ])
    }
    pub(crate) fn try_materialized_unit_step(
        &self,
        step: IntegerUnitStep,
        max_digits: usize,
        max_bytes: usize,
    ) -> Result<NumericMaterialized<Self>, NumericMaterializationFailure> {
        self.try_unit_step(step, max_digits, max_bytes)
            .map(|r| NumericMaterialized {
                value: r.value,
                capacity_bytes: r.capacity_bytes,
            })
            .map_err(unit_step_materialization_failure)
    }
}

impl HtmlNonNegativeDecimal {
    // Each part uses validation, canonicalization, copy reads/writes and UTF-8.
    pub(crate) fn from_borrowed_digits_work_bound(
        integer: usize,
        fraction: usize,
    ) -> Option<usize> {
        HtmlNonNegativeInteger::from_borrowed_digits_work_bound(integer)?.checked_add(
            HtmlNonNegativeInteger::from_borrowed_digits_work_bound(fraction)?,
        )
    }
    pub(crate) fn try_from_borrowed_digits(
        integer: &[u16],
        fraction: &[u16],
        max_digits: usize,
        max_bytes: usize,
    ) -> Result<NumericMaterialized<Self>, NumericMaterializationFailure> {
        let mut observed = 0;
        let integer =
            controlled_digit_string(integer, false, max_digits, max_bytes, &mut observed)?;
        let fraction =
            controlled_digit_string(fraction, true, max_digits, max_bytes, &mut observed)?;
        Ok(NumericMaterialized {
            value: Self {
                integer: HtmlNonNegativeInteger { digits: integer },
                fraction,
            },
            capacity_bytes: observed,
        })
    }
    pub(crate) fn copy_work_bound(&self) -> Option<usize> {
        let n = self.fraction.len();
        self.integer
            .copy_work_bound()?
            .checked_add(bounded_work(&[n, n, n])?)
    }
    pub(crate) fn try_copy(
        &self,
        max_digits: usize,
        max_bytes: usize,
    ) -> Result<NumericMaterialized<Self>, NumericMaterializationFailure> {
        let mut observed = 0;
        let mut integer = reserve_materialization(
            self.integer.digits.len(),
            Some(max_digits),
            max_bytes,
            &mut observed,
        )?;
        integer.extend_from_slice(self.integer.digits.as_bytes());
        let mut fraction =
            reserve_materialization(self.fraction.len(), None, max_bytes, &mut observed)?;
        fraction.extend_from_slice(self.fraction.as_bytes());
        Ok(NumericMaterialized {
            value: Self {
                integer: HtmlNonNegativeInteger {
                    digits: ascii_digit_string(integer),
                },
                fraction: ascii_digit_string(fraction),
            },
            capacity_bytes: observed,
        })
    }
    // Fraction: initialized writes f, product reads/writes f+f, trailing trim f,
    // UTF-8 f; integer: length/read/write/reversal-read/reversal-write/UTF-8.
    pub(crate) fn multiplied_by_work_bound(&self) -> Option<usize> {
        let f = self.fraction.len();
        self.integer
            .multiplied_by_work_bound()?
            .checked_add(bounded_work(&[f, f, f, f, f])?)
    }
    pub(crate) fn try_multiplied_by(
        &self,
        multiplier: u32,
        max_digits: usize,
        max_bytes: usize,
    ) -> Result<NumericMaterialized<Self>, NumericMaterializationFailure> {
        if multiplier == 0 || self.is_zero() {
            return HtmlNonNegativeInteger::try_from_u32(0, max_digits, max_bytes).map(|r| {
                NumericMaterialized {
                    value: r.value.into(),
                    capacity_bytes: r.capacity_bytes,
                }
            });
        }
        let mut observed = 0;
        let mut fraction =
            reserve_materialization(self.fraction.len(), None, max_bytes, &mut observed)?;
        fraction.resize(self.fraction.len(), b'0');
        let carry = write_fraction_product(self.fraction.as_bytes(), multiplier, &mut fraction);
        let integer = controlled_product(
            self.integer.digits.as_bytes(),
            multiplier,
            carry,
            max_digits,
            max_bytes,
            &mut observed,
        )?;
        Ok(NumericMaterialized {
            value: Self::from_ascii_fraction(HtmlNonNegativeInteger { digits: integer }, fraction),
            capacity_bytes: observed,
        })
    }
    // Fraction: initialization f, operand reads f+f, output f, trim f, UTF-8 f.
    // Integral carry is handled inside the shared sum, without a second sum.
    pub(crate) fn added_to_work_bound(&self, other: &Self) -> Option<usize> {
        let f = self.fraction.len().max(other.fraction.len());
        self.integer
            .added_to_work_bound(&other.integer)?
            .checked_add(bounded_work(&[f, f, f, f, f, f])?)
    }
    pub(crate) fn try_added_to(
        &self,
        other: &Self,
        max_digits: usize,
        max_bytes: usize,
    ) -> Result<NumericMaterialized<Self>, NumericMaterializationFailure> {
        let mut observed = 0;
        let scale = self.fraction.len().max(other.fraction.len());
        let mut fraction = reserve_materialization(scale, None, max_bytes, &mut observed)?;
        fraction.resize(scale, b'0');
        let carry = write_fraction_sum(
            self.fraction.as_bytes(),
            other.fraction.as_bytes(),
            &mut fraction,
        );
        let integer = controlled_sum(
            self.integer.digits.as_bytes(),
            other.integer.digits.as_bytes(),
            carry,
            max_digits,
            max_bytes,
            &mut observed,
        )?;
        Ok(NumericMaterialized {
            value: Self::from_ascii_fraction(HtmlNonNegativeInteger { digits: integer }, fraction),
            capacity_bytes: observed,
        })
    }
}

/// An exact HTML integer, without a machine-integer magnitude limit.
///
/// Leading zeroes and negative zero are normalized. Decimal magnitude digits
/// remain exact; bounded machine projections report range failure separately.
#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub struct HtmlInteger {
    negative: bool,
    magnitude: HtmlNonNegativeInteger,
}

/// The two exact arithmetic operations used by HTML's ordinal-value algorithm.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum IntegerUnitStep {
    Increment,
    Decrement,
}

/// One canonical integer and the actual capacity of its sole digit allocation.
#[derive(Debug)]
pub(crate) struct IntegerUnitStepResult {
    pub(crate) value: HtmlInteger,
    pub(crate) capacity_bytes: usize,
}

/// Numeric allocation boundaries; the semantic session owns their admission.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum IntegerUnitStepError {
    RepresentationExhausted,
    DigitLimitExceeded {
        limit: usize,
        required: usize,
    },
    RequestedCapacityLimitExceeded {
        limit: usize,
        required: usize,
    },
    ActualCapacityLimitExceeded {
        limit: usize,
        requested: usize,
        capacity_bytes: usize,
    },
    AllocationFailed {
        requested: usize,
    },
}

fn reserve_integer_digits(
    required: usize,
    max_result_digits: usize,
    max_capacity_bytes: usize,
) -> Result<String, IntegerUnitStepError> {
    check_integer_digit_capacity(required, max_result_digits, max_capacity_bytes)?;
    let mut digits = String::new();
    digits
        .try_reserve_exact(required)
        .map_err(|_| IntegerUnitStepError::AllocationFailed {
            requested: required,
        })?;
    check_integer_actual_capacity(required, digits.capacity(), max_capacity_bytes)?;
    Ok(digits)
}

fn check_integer_digit_capacity(
    required: usize,
    max_result_digits: usize,
    max_capacity_bytes: usize,
) -> Result<(), IntegerUnitStepError> {
    if required > isize::MAX as usize {
        return Err(IntegerUnitStepError::RepresentationExhausted);
    }
    if required > max_result_digits {
        return Err(IntegerUnitStepError::DigitLimitExceeded {
            limit: max_result_digits,
            required,
        });
    }
    if required > max_capacity_bytes {
        return Err(IntegerUnitStepError::RequestedCapacityLimitExceeded {
            limit: max_capacity_bytes,
            required,
        });
    }
    Ok(())
}

fn check_integer_actual_capacity(
    requested: usize,
    capacity_bytes: usize,
    max_capacity_bytes: usize,
) -> Result<(), IntegerUnitStepError> {
    if capacity_bytes > max_capacity_bytes {
        Err(IntegerUnitStepError::ActualCapacityLimitExceeded {
            limit: max_capacity_bytes,
            requested,
            capacity_bytes,
        })
    } else {
        Ok(())
    }
}

impl HtmlInteger {
    /// Constructs an integer from an intrinsically valid magnitude and sign.
    /// A zero magnitude always produces zero, regardless of `negative`.
    pub fn from_magnitude(magnitude: HtmlNonNegativeInteger, negative: bool) -> Self {
        Self {
            negative: negative && !magnitude.is_zero(),
            magnitude,
        }
    }

    /// Exact signed ordering without a machine-integer projection.
    pub(crate) fn compare(&self, other: &Self) -> Ordering {
        match (self.negative, other.negative) {
            (true, false) => Ordering::Less,
            (false, true) => Ordering::Greater,
            (false, false) => self.magnitude.compare(&other.magnitude),
            (true, true) => self.magnitude.compare(&other.magnitude).reverse(),
        }
    }

    /// Reports whether the mathematical value is below zero.
    pub fn is_negative(&self) -> bool {
        self.negative
    }

    /// Returns the exact absolute value.
    pub fn magnitude(&self) -> &HtmlNonNegativeInteger {
        &self.magnitude
    }

    /// Exact ordinal increment/decrement using one fallible digit allocation.
    ///
    /// The caller precharges its bounded inspection/write envelope and owns
    /// cancellation/admission. This atomic operation inspects at most the input
    /// digit count and writes at most that count plus one; it has no internal
    /// cancellation poll or parser-quantum promise. The sign occupies no digit
    /// capacity. Logical digits and actual requested allocation are separate.
    pub(crate) fn try_unit_step(
        &self,
        step: IntegerUnitStep,
        max_result_digits: usize,
        max_capacity_bytes: usize,
    ) -> Result<IntegerUnitStepResult, IntegerUnitStepError> {
        let input = self.magnitude.decimal_digits();
        let zero = self.magnitude.is_zero();
        let grow = zero || (step == IntegerUnitStep::Increment) != self.negative;
        let negative = if zero {
            step == IntegerUnitStep::Decrement
        } else {
            self.negative
        };
        let digits = try_unit_step_digits(input, grow, max_result_digits, max_capacity_bytes)?;
        let capacity_bytes = digits.capacity();
        Ok(IntegerUnitStepResult {
            value: Self::from_magnitude(HtmlNonNegativeInteger { digits }, negative),
            capacity_bytes,
        })
    }

    /// Constructs the source's reversed-list count without infallible formatting.
    /// Both fixed-width scans and the write are bounded by `usize` decimal width.
    pub(crate) fn try_from_usize(
        value: usize,
        max_result_digits: usize,
        max_capacity_bytes: usize,
    ) -> Result<IntegerUnitStepResult, IntegerUnitStepError> {
        let mut divisor = 1_usize;
        let mut required = 1;
        while divisor <= value / 10 {
            divisor *= 10;
            required += 1;
        }
        let mut digits = reserve_integer_digits(required, max_result_digits, max_capacity_bytes)?;
        loop {
            digits.push(char::from(b'0' + ((value / divisor) % 10) as u8));
            divisor /= 10;
            if divisor == 0 {
                break;
            }
        }
        let capacity_bytes = digits.capacity();
        Ok(IntegerUnitStepResult {
            value: Self::from_magnitude(HtmlNonNegativeInteger { digits }, false),
            capacity_bytes,
        })
    }

    /// Returns normalized author syntax, with no leading zeroes or plus sign.
    pub fn as_document_string(&self) -> DocumentString {
        let prefix = if self.negative { "-" } else { "" };
        DocumentString::from(format!("{prefix}{}", self.magnitude.digits))
    }

    /// Projects the value into `i128`, reporting a range error without loss.
    pub fn to_i128(&self) -> Result<i128, HtmlNumericRangeError> {
        let magnitude = self.magnitude.to_u128()?;
        if self.negative {
            if magnitude == i128::MIN.unsigned_abs() {
                Ok(i128::MIN)
            } else {
                i128::try_from(magnitude)
                    .map(|value| -value)
                    .map_err(|_| HtmlNumericRangeError)
            }
        } else {
            i128::try_from(magnitude).map_err(|_| HtmlNumericRangeError)
        }
    }

    /// Projects the value into `i64`, reporting a range error without loss.
    pub fn to_i64(&self) -> Result<i64, HtmlNumericRangeError> {
        i64::try_from(self.to_i128()?).map_err(|_| HtmlNumericRangeError)
    }
}

impl From<i128> for HtmlInteger {
    fn from(value: i128) -> Self {
        Self::from_magnitude(value.unsigned_abs().into(), value.is_negative())
    }
}

/// An exact zero or positive HTML integer.
#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub struct HtmlNonNegativeInteger {
    digits: String,
}

impl HtmlNonNegativeInteger {
    fn compare(&self, other: &Self) -> Ordering {
        self.digits
            .len()
            .cmp(&other.digits.len())
            .then_with(|| self.digits.cmp(&other.digits))
    }

    // The caller proves self >= other + borrow. Digit subtraction never projects
    // the magnitude into a machine integer. Each output byte is one ASCII digit.
    fn subtract_with_borrow(&self, other: &Self, mut borrow: u8) -> Self {
        let mut right = other.digits.bytes().rev();
        let mut reversed = Vec::with_capacity(self.digits.len());
        for digit in self.digits.bytes().rev() {
            let difference = i16::from(digit - b'0')
                - i16::from(right.next().map_or(0, |digit| digit - b'0'))
                - i16::from(borrow);
            borrow = u8::from(difference < 0);
            reversed.push(b'0' + difference.rem_euclid(10) as u8);
        }
        debug_assert_eq!(borrow, 0);
        debug_assert!(right.next().is_none());
        while reversed.len() > 1 && reversed.last() == Some(&b'0') {
            reversed.pop();
        }
        Self {
            digits: reversed.into_iter().rev().map(char::from).collect(),
        }
    }

    /// Checks an exact sequence of one or more ASCII decimal digits.
    ///
    /// No sign, whitespace, trimming, or prefix recovery is accepted. There is
    /// no machine-integer magnitude limit; leading zeroes are normalized.
    pub fn from_decimal_digits(input: &DocumentString) -> Result<Self, HtmlDecimalDigitsError> {
        let units = input.code_units();
        check_decimal_digits(units)?;
        Ok(Self::from_digits(units))
    }

    fn from_digits(digits: &[u16]) -> Self {
        let first = digits.iter().position(|&unit| unit != b'0' as u16);
        let digits = first.map_or_else(
            || "0".to_owned(),
            |first| {
                digits[first..]
                    .iter()
                    .map(|&unit| char::from(unit as u8))
                    .collect()
            },
        );
        Self { digits }
    }

    /// Reports whether the mathematical value is zero.
    pub fn is_zero(&self) -> bool {
        self.digits == "0"
    }

    /// Returns canonical ASCII decimal digits, without leading zeroes.
    pub fn decimal_digits(&self) -> &str {
        &self.digits
    }

    /// Returns the exact remainder without projecting the whole value.
    pub fn remainder(&self, divisor: NonZeroU32) -> u32 {
        let divisor = u64::from(divisor.get());
        self.digits.bytes().fold(0_u64, |remainder, digit| {
            remainder_digit(digit - b'0', divisor, remainder)
        }) as u32
    }

    pub(crate) fn multiplied_by(&self, multiplier: u32) -> Self {
        if multiplier == 0 || self.is_zero() {
            return Self::from(0);
        }
        let capacity = self
            .digits
            .len()
            .checked_add(10)
            .filter(|&len| len <= isize::MAX as usize)
            .expect("integer product representation exceeds native capacity");
        let mut output = Vec::with_capacity(capacity);
        write_product(self.digits.as_bytes(), multiplier, 0, &mut output);
        Self {
            digits: ascii_digit_string(output),
        }
    }

    pub(crate) fn added_to(&self, other: &Self) -> Self {
        let capacity = integer_sum_capacity(self.digits.len(), other.digits.len())
            .expect("integer sum representation exceeds native capacity");
        let mut output = Vec::with_capacity(capacity);
        write_sum(
            self.digits.as_bytes(),
            other.digits.as_bytes(),
            0,
            &mut output,
        );
        Self {
            digits: ascii_digit_string(output),
        }
    }

    /// Subtracts one exactly, with no result below zero.
    pub(crate) fn decremented(&self) -> Option<Self> {
        if self.is_zero() {
            return None;
        }
        let (required, pivot) = unit_step_shape(&self.digits, false)
            .expect("integer decrement representation exceeds native capacity");
        let mut digits = String::with_capacity(required);
        write_unit_step(&self.digits, false, pivot, &mut digits);
        Some(Self { digits })
    }

    /// Divides an exact magnitude by a bounded nonzero integer.
    /// The remainder is strictly less than the divisor.
    pub(crate) fn quotient_remainder(&self, divisor: NonZeroU32) -> (Self, u32) {
        let mut output = Vec::with_capacity(self.digits.len());
        let remainder = write_quotient(self.digits.as_bytes(), divisor, &mut output);
        (
            Self {
                digits: ascii_digit_string(output),
            },
            remainder,
        )
    }

    /// Returns normalized ASCII decimal magnitude digits.
    pub fn as_document_string(&self) -> DocumentString {
        DocumentString::from(self.digits.as_str())
    }

    /// Projects the value into `u128`, reporting a range error without loss.
    pub fn to_u128(&self) -> Result<u128, HtmlNumericRangeError> {
        self.digits.parse().map_err(|_| HtmlNumericRangeError)
    }

    /// Projects the value into `u64`, reporting a range error without loss.
    pub fn to_u64(&self) -> Result<u64, HtmlNumericRangeError> {
        self.digits.parse().map_err(|_| HtmlNumericRangeError)
    }
}

impl From<u128> for HtmlNonNegativeInteger {
    fn from(value: u128) -> Self {
        Self {
            digits: value.to_string(),
        }
    }
}

impl TryFrom<HtmlInteger> for HtmlNonNegativeInteger {
    type Error = HtmlNegativeIntegerError;

    fn try_from(value: HtmlInteger) -> Result<Self, Self::Error> {
        if value.negative {
            Err(HtmlNegativeIntegerError)
        } else {
            Ok(value.magnitude)
        }
    }
}

/// An exact decimal-magnitude constructor received no digits or a non-digit.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum HtmlDecimalDigitsError {
    /// There were no code units.
    Empty,
    /// The first non-ASCII-digit code unit, with its zero-based UTF-16 offset.
    NonAsciiDigit {
        /// The zero-based UTF-16 code-unit offset.
        code_unit_offset: usize,
        /// The exact invalid code unit.
        code_unit: u16,
    },
}

impl fmt::Display for HtmlDecimalDigitsError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Empty => formatter.write_str("HTML integer magnitude has no decimal digits"),
            Self::NonAsciiDigit {
                code_unit_offset,
                code_unit,
            } => write!(
                formatter,
                "non-ASCII-digit U+{code_unit:04X} at UTF-16 code-unit offset {code_unit_offset}"
            ),
        }
    }
}

impl Error for HtmlDecimalDigitsError {}

/// An exact numeric value cannot fit the requested machine-integer range.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct HtmlNumericRangeError;

impl fmt::Display for HtmlNumericRangeError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str("HTML numeric value exceeds the requested integer range")
    }
}

impl Error for HtmlNumericRangeError {}

/// A negative integer cannot become a nonnegative integer.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct HtmlNegativeIntegerError;

impl fmt::Display for HtmlNegativeIntegerError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str("HTML integer is negative")
    }
}

impl Error for HtmlNegativeIntegerError {}

/// A finite IEEE 754 binary64 HTML numeric value, with zero normalized to +0.
///
/// The HTML recovery algorithm rounds the exact decimal value once to the
/// nearest binary64 value, breaking ties with an even significand. Overflow is
/// an error; underflow may produce zero. Integer values have a separate exact
/// representation and must not be routed through this type.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct HtmlFloatingPointNumber {
    value: f64,
}

impl HtmlFloatingPointNumber {
    /// Changes representation of the actual finite binary64 bits, without
    /// reparsing decimal serialization or introducing another rounding step.
    pub(crate) fn exact_decimal(self) -> HtmlSignedDecimal {
        let bits = self.value.to_bits();
        let exponent_field = ((bits >> 52) & 0x7ff) as i32;
        let fraction = bits & ((1_u64 << 52) - 1);
        let (significand, exponent) = if exponent_field == 0 {
            (fraction, -1074)
        } else {
            ((1_u64 << 52) | fraction, exponent_field - 1023 - 52)
        };
        if significand == 0 {
            return HtmlSignedDecimal::from(HtmlInteger::from(0));
        }
        let mut coefficient = HtmlNonNegativeInteger::from(u128::from(significand));
        let magnitude = if exponent >= 0 {
            for _ in 0..exponent {
                coefficient = coefficient.multiplied_by(2);
            }
            coefficient.into()
        } else {
            // The largest scale is 1074. n / 2^k = n * 5^k / 10^k exactly.
            let scale = (-exponent) as usize;
            for _ in 0..scale {
                coefficient = coefficient.multiplied_by(5);
            }
            let digits = coefficient.digits;
            let (integer, fraction) = if digits.len() > scale {
                let split = digits.len() - scale;
                (
                    HtmlNonNegativeInteger {
                        digits: digits[..split].to_owned(),
                    },
                    digits.as_bytes()[split..].to_vec(),
                )
            } else {
                let mut fraction = vec![b'0'; scale];
                fraction[scale - digits.len()..].copy_from_slice(digits.as_bytes());
                (HtmlNonNegativeInteger::from(0), fraction)
            };
            HtmlNonNegativeDecimal::from_ascii_fraction(integer, fraction)
        };
        HtmlSignedDecimal::from_magnitude(magnitude, bits >> 63 != 0)
    }

    /// Returns the finite binary64 value.
    pub fn value(&self) -> f64 {
        self.value
    }

    /// Returns deterministic valid HTML author syntax for this binary64 value.
    ///
    /// This is native Rust decimal serialization, not the ECMAScript
    /// `ToString` operation used by HTML's separate best-representation rule.
    pub fn as_document_string(&self) -> DocumentString {
        DocumentString::from(self.value.to_string())
    }
}

impl TryFrom<f64> for HtmlFloatingPointNumber {
    type Error = HtmlNonFiniteNumberError;

    fn try_from(value: f64) -> Result<Self, Self::Error> {
        if !value.is_finite() {
            return Err(HtmlNonFiniteNumberError);
        }
        Ok(Self {
            value: if value == 0.0 { 0.0 } else { value },
        })
    }
}

/// Infinity and NaN cannot become HTML floating-point values.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct HtmlNonFiniteNumberError;

impl fmt::Display for HtmlNonFiniteNumberError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str("HTML floating-point value is not finite")
    }
}

impl Error for HtmlNonFiniteNumberError {}

/// A list of intrinsically valid HTML floating-point values.
///
/// Attribute-specific cardinality and numeric-range constraints are applied by
/// the attribute owner, rather than guessed by this common microsyntax value.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct HtmlFloatingPointList {
    values: Vec<HtmlFloatingPointNumber>,
}

impl HtmlFloatingPointList {
    /// Constructs a list from finite, already checked numeric values.
    pub fn new(values: impl Into<Vec<HtmlFloatingPointNumber>>) -> Self {
        Self {
            values: values.into(),
        }
    }

    /// Returns the ordered numeric values.
    pub fn values(&self) -> &[HtmlFloatingPointNumber] {
        &self.values
    }

    /// Returns valid author syntax with commas and no whitespace.
    pub fn as_document_string(&self) -> DocumentString {
        DocumentString::from(
            self.values
                .iter()
                .map(|number| number.value.to_string())
                .collect::<Vec<_>>()
                .join(","),
        )
    }
}

/// An exact nonnegative decimal number used by HTML dimension and temporal
/// algorithms.
///
/// Unlike HTML floating-point recovery, these clauses specify no binary64
/// conversion, rounding, or overflow rule for their decimal quantities. This
/// representation retains the base-ten value exactly, independently of
/// machine-number limits. Temporal owners apply their own range and authored
/// precision rules to this shared numeric value.
#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub struct HtmlNonNegativeDecimal {
    integer: HtmlNonNegativeInteger,
    fraction: String,
}

impl HtmlNonNegativeDecimal {
    fn compare(&self, other: &Self) -> Ordering {
        let integer = self.integer.compare(&other.integer);
        if integer != Ordering::Equal {
            return integer;
        }
        for index in 0..self.fraction.len().max(other.fraction.len()) {
            let left = self.fraction.as_bytes().get(index).copied().unwrap_or(b'0');
            let right = other
                .fraction
                .as_bytes()
                .get(index)
                .copied()
                .unwrap_or(b'0');
            let order = left.cmp(&right);
            if order != Ordering::Equal {
                return order;
            }
        }
        Ordering::Equal
    }

    // Caller establishes the magnitude ordering before entering this operation.
    fn subtracted_by(&self, other: &Self) -> Self {
        let scale = self.fraction.len().max(other.fraction.len());
        let mut fraction = vec![b'0'; scale];
        let mut borrow = 0;
        for index in (0..scale).rev() {
            let left = self.fraction.as_bytes().get(index).copied().unwrap_or(b'0');
            let right = other
                .fraction
                .as_bytes()
                .get(index)
                .copied()
                .unwrap_or(b'0');
            let difference = i16::from(left - b'0') - i16::from(right - b'0') - i16::from(borrow);
            borrow = u8::from(difference < 0);
            fraction[index] = b'0' + difference.rem_euclid(10) as u8;
        }
        Self::from_ascii_fraction(
            self.integer.subtract_with_borrow(&other.integer, borrow),
            fraction,
        )
    }

    /// Constructs a decimal from an exact integer and one or more fractional
    /// ASCII digits. Signs, whitespace, and recovery syntax are rejected.
    ///
    /// Fractional leading zeroes are significant; trailing zeroes are removed.
    pub fn with_fraction_digits(
        integer: HtmlNonNegativeInteger,
        fraction: &DocumentString,
    ) -> Result<Self, HtmlDecimalDigitsError> {
        check_decimal_digits(fraction.code_units())?;
        Ok(Self::from_parts(integer, fraction.code_units()))
    }

    fn from_parts(integer: HtmlNonNegativeInteger, fraction: &[u16]) -> Self {
        let end = fraction
            .iter()
            .rposition(|&unit| unit != b'0' as u16)
            .map_or(0, |index| index + 1);
        Self {
            integer,
            fraction: fraction[..end]
                .iter()
                .map(|&unit| char::from(unit as u8))
                .collect(),
        }
    }

    /// Adds exact decimal quantities, preserving significant fractional zeroes.
    pub(crate) fn added_to(&self, other: &Self) -> Self {
        let mut fraction = vec![b'0'; self.fraction.len().max(other.fraction.len())];
        let carry = write_fraction_sum(
            self.fraction.as_bytes(),
            other.fraction.as_bytes(),
            &mut fraction,
        );
        let capacity = integer_sum_capacity(self.integer.digits.len(), other.integer.digits.len())
            .expect("decimal sum representation exceeds native capacity");
        let mut integer = Vec::with_capacity(capacity);
        write_sum(
            self.integer.digits.as_bytes(),
            other.integer.digits.as_bytes(),
            carry,
            &mut integer,
        );
        Self::from_ascii_fraction(
            HtmlNonNegativeInteger {
                digits: ascii_digit_string(integer),
            },
            fraction,
        )
    }

    /// Multiplies an exact decimal quantity by a bounded nonnegative integer.
    pub(crate) fn multiplied_by(&self, multiplier: u32) -> Self {
        if multiplier == 0 || self.is_zero() {
            return HtmlNonNegativeInteger::from(0).into();
        }
        let mut fraction = vec![b'0'; self.fraction.len()];
        let carry = write_fraction_product(self.fraction.as_bytes(), multiplier, &mut fraction);
        let capacity = self
            .integer
            .digits
            .len()
            .checked_add(10)
            .filter(|&len| len <= isize::MAX as usize)
            .expect("decimal product representation exceeds native capacity");
        let mut integer = Vec::with_capacity(capacity);
        write_product(
            self.integer.digits.as_bytes(),
            multiplier,
            carry,
            &mut integer,
        );
        Self::from_ascii_fraction(
            HtmlNonNegativeInteger {
                digits: ascii_digit_string(integer),
            },
            fraction,
        )
    }

    // Only the arithmetic above constructs these ASCII digit buffers. Leading
    // fractional zeroes retain scale; removing trailing zeroes is lossless.
    fn from_ascii_fraction(integer: HtmlNonNegativeInteger, mut fraction: Vec<u8>) -> Self {
        while fraction.last() == Some(&b'0') {
            fraction.pop();
        }
        Self {
            integer,
            fraction: ascii_digit_string(fraction),
        }
    }

    /// Returns the exact integer part.
    pub fn integer_part(&self) -> &HtmlNonNegativeInteger {
        &self.integer
    }

    /// Returns significant fractional digits, including any leading zeroes.
    /// An integer value has no fractional digits.
    pub fn fractional_digits(&self) -> Option<&str> {
        (!self.fraction.is_empty()).then_some(self.fraction.as_str())
    }

    /// Reports whether the exact value is zero, without underflow.
    pub fn is_zero(&self) -> bool {
        self.integer.is_zero() && self.fraction.is_empty()
    }

    /// Returns normalized decimal syntax without sign or exponent.
    pub fn as_document_string(&self) -> DocumentString {
        DocumentString::from(self.decimal_string())
    }

    fn decimal_string(&self) -> String {
        if self.fraction.is_empty() {
            self.integer.digits.clone()
        } else {
            self.integer
                .digits
                .len()
                .checked_add(1)
                .and_then(|len| len.checked_add(self.fraction.len()))
                .filter(|&len| len <= isize::MAX as usize)
                .expect("decimal representation exceeds native capacity");
            format!("{}.{}", self.integer.digits, self.fraction)
        }
    }
}

impl From<HtmlNonNegativeInteger> for HtmlNonNegativeDecimal {
    fn from(integer: HtmlNonNegativeInteger) -> Self {
        Self {
            integer,
            fraction: String::new(),
        }
    }
}

/// An exact signed decimal quantity with a canonical nonnegative magnitude.
///
/// Zero is never negative. This native value retains arbitrary integer digits
/// and significant fractional digits; it does not impose binary64 rounding.
/// Native arithmetic and serialization use ordinary allocations, without an
/// allocator-failure or cancellation guarantee.
#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub struct HtmlSignedDecimal {
    magnitude: HtmlNonNegativeDecimal,
    negative: bool,
}

impl HtmlSignedDecimal {
    pub(crate) fn compare(&self, other: &Self) -> Ordering {
        match (self.negative, other.negative) {
            (true, false) => Ordering::Less,
            (false, true) => Ordering::Greater,
            (false, false) => self.magnitude.compare(&other.magnitude),
            (true, true) => other.magnitude.compare(&self.magnitude),
        }
    }

    /// Constructs an exact quantity, normalizing either sign of zero to zero.
    pub fn from_magnitude(magnitude: HtmlNonNegativeDecimal, negative: bool) -> Self {
        Self {
            negative: negative && !magnitude.is_zero(),
            magnitude,
        }
    }

    /// Returns the exact absolute value.
    pub fn magnitude(&self) -> &HtmlNonNegativeDecimal {
        &self.magnitude
    }

    /// Reports whether the mathematical quantity is below zero.
    pub fn is_negative(&self) -> bool {
        self.negative
    }

    /// Returns canonical decimal syntax, without an exponent or negative zero.
    pub fn as_document_string(&self) -> DocumentString {
        let mut result = self.magnitude.decimal_string();
        if self.negative {
            result
                .len()
                .checked_add(1)
                .filter(|&len| len <= isize::MAX as usize)
                .expect("signed decimal representation exceeds native capacity");
            result.insert(0, '-');
        }
        result.into()
    }

    pub(crate) fn added_to(&self, other: &Self) -> Self {
        if self.negative == other.negative {
            return Self::from_magnitude(self.magnitude.added_to(&other.magnitude), self.negative);
        }
        match self.magnitude.compare(&other.magnitude) {
            Ordering::Less => Self::from_magnitude(
                other.magnitude.subtracted_by(&self.magnitude),
                other.negative,
            ),
            Ordering::Equal => Self::from(HtmlInteger::from(0)),
            Ordering::Greater => Self::from_magnitude(
                self.magnitude.subtracted_by(&other.magnitude),
                self.negative,
            ),
        }
    }

    pub(crate) fn multiplied_by(&self, multiplier: u32) -> Self {
        Self::from_magnitude(self.magnitude.multiplied_by(multiplier), self.negative)
    }

    pub(crate) fn integer_difference(
        left: HtmlNonNegativeInteger,
        right: HtmlNonNegativeInteger,
    ) -> Self {
        Self::from_magnitude(left.into(), false).added_to(&Self::from_magnitude(right.into(), true))
    }
}

impl From<HtmlInteger> for HtmlSignedDecimal {
    fn from(value: HtmlInteger) -> Self {
        Self::from_magnitude(value.magnitude.into(), value.negative)
    }
}

/// An exact HTML dimension with its intrinsic unit category.
///
/// Relative values belong to the dimension-list algorithm. The single-value
/// dimension algorithm returns only absolute lengths or percentages. These
/// categories do not resolve CSS units or downstream layout context.
#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub enum HtmlDimension {
    /// An absolute length.
    Absolute(HtmlNonNegativeDecimal),
    /// A percentage.
    Percentage(HtmlNonNegativeDecimal),
    /// A relative dimension-list quantity.
    Relative(HtmlNonNegativeDecimal),
}

impl HtmlDimension {
    /// Returns the exact nonnegative decimal quantity.
    pub fn value(&self) -> &HtmlNonNegativeDecimal {
        match self {
            Self::Absolute(value) | Self::Percentage(value) | Self::Relative(value) => value,
        }
    }

    /// Returns normalized dimension syntax, with `%` or `*` where applicable.
    pub fn as_document_string(&self) -> DocumentString {
        DocumentString::from(self.dimension_string())
    }

    fn dimension_string(&self) -> String {
        let suffix = match self {
            Self::Absolute(_) => "",
            Self::Percentage(_) => "%",
            Self::Relative(_) => "*",
        };
        format!("{}{suffix}", self.value().decimal_string())
    }
}

/// An ordered list of exact HTML dimensions.
#[derive(Clone, Debug, Default, Eq, Hash, PartialEq)]
pub struct HtmlDimensionList {
    values: Vec<HtmlDimension>,
}

impl HtmlDimensionList {
    /// Constructs a list from intrinsically valid dimensions.
    pub fn new(values: impl Into<Vec<HtmlDimension>>) -> Self {
        Self {
            values: values.into(),
        }
    }

    /// Returns the ordered dimensions.
    pub fn values(&self) -> &[HtmlDimension] {
        &self.values
    }

    /// Returns normalized comma-separated dimension syntax.
    pub fn as_document_string(&self) -> DocumentString {
        DocumentString::from(
            self.values
                .iter()
                .map(HtmlDimension::dimension_string)
                .collect::<Vec<_>>()
                .join(","),
        )
    }
}

/// Failures of the common numeric algorithms; request limits are separate.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum NumericParseError {
    InvalidSyntax,
    Negative,
    FloatingPointOverflow,
    ZeroDimension,
}

fn is_digit(unit: u16) -> bool {
    (b'0' as u16..=b'9' as u16).contains(&unit)
}

fn check_decimal_digits(units: &[u16]) -> Result<(), HtmlDecimalDigitsError> {
    if units.is_empty() {
        return Err(HtmlDecimalDigitsError::Empty);
    }
    if let Some(code_unit_offset) = units.iter().position(|&unit| !is_digit(unit)) {
        return Err(HtmlDecimalDigitsError::NonAsciiDigit {
            code_unit_offset,
            code_unit: units[code_unit_offset],
        });
    }
    Ok(())
}

fn is_ascii_whitespace(unit: u16) -> bool {
    matches!(unit, 0x09 | 0x0a | 0x0c | 0x0d | 0x20)
}

fn skip_whitespace(units: &[u16], position: &mut usize) {
    while units
        .get(*position)
        .is_some_and(|&u| is_ascii_whitespace(u))
    {
        *position += 1;
    }
}

fn collect_digits(units: &[u16], position: &mut usize) {
    while units.get(*position).is_some_and(|&u| is_digit(u)) {
        *position += 1;
    }
}

/// Author grammar: an optional minus followed by one or more ASCII digits.
pub(crate) fn validate_integer(input: &DocumentString) -> Result<HtmlInteger, NumericParseError> {
    integer(input.code_units(), false)
}

/// §2.3.4.1 recovery consumes only the initial signed digit sequence.
pub(crate) fn parse_integer(input: &DocumentString) -> Result<HtmlInteger, NumericParseError> {
    integer(input.code_units(), true)
}

/// A borrowed digit prefix, prior to canonicalization or numeric projection.
struct IntegerPrefix<'a> {
    negative: bool,
    digits: &'a [u16],
}

/// The shared integer grammar as one-unit progress. Callers own admission;
/// this scanner never reads an input provider or allocates digit storage.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct IntegerPrefixScanner {
    recover: bool,
    phase: IntegerPrefixPhase,
    negative: bool,
    position: usize,
    start: usize,
    end: usize,
}
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum IntegerPrefixPhase {
    Leading,
    Signed,
    Digits,
    Done,
    Invalid,
}
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct ScannedIntegerPrefix {
    pub(crate) negative: bool,
    pub(crate) start: usize,
    pub(crate) end: usize,
}
impl IntegerPrefixScanner {
    pub(crate) fn new(recover: bool) -> Self {
        Self {
            recover,
            phase: IntegerPrefixPhase::Leading,
            negative: false,
            position: 0,
            start: 0,
            end: 0,
        }
    }
    /// False means the recovery prefix is complete or syntax is already invalid.
    pub(crate) fn feed(&mut self, unit: u16) -> bool {
        match self.phase {
            IntegerPrefixPhase::Done | IntegerPrefixPhase::Invalid => return false,
            IntegerPrefixPhase::Leading if self.recover && is_ascii_whitespace(unit) => {
                self.position += 1;
                return true;
            }
            IntegerPrefixPhase::Leading => {
                if unit == b'-' as u16 || (self.recover && unit == b'+' as u16) {
                    self.negative = unit == b'-' as u16;
                    self.position += 1;
                    self.start = self.position;
                    self.phase = IntegerPrefixPhase::Signed;
                    return true;
                }
                self.start = self.position;
            }
            IntegerPrefixPhase::Signed | IntegerPrefixPhase::Digits => {}
        }
        if is_digit(unit) {
            self.position += 1;
            self.end = self.position;
            self.phase = IntegerPrefixPhase::Digits;
            true
        } else {
            self.phase = if self.phase == IntegerPrefixPhase::Digits && self.recover {
                IntegerPrefixPhase::Done
            } else {
                IntegerPrefixPhase::Invalid
            };
            false
        }
    }
    pub(crate) fn finish(self) -> Result<ScannedIntegerPrefix, NumericParseError> {
        if matches!(
            self.phase,
            IntegerPrefixPhase::Digits | IntegerPrefixPhase::Done
        ) {
            Ok(ScannedIntegerPrefix {
                negative: self.negative,
                start: self.start,
                end: self.end,
            })
        } else {
            Err(NumericParseError::InvalidSyntax)
        }
    }
}

impl ScannedIntegerPrefix {
    /// The same nonnegative recovery decision as the ordinary parser, with no
    /// allocation. The caller admits the original input and precharges at most
    /// one visit per prefix digit before this atomic sign/zero inspection.
    pub(crate) fn nonnegative_digits(self, units: &[u16]) -> Result<&[u16], NumericParseError> {
        let digits = &units[self.start..self.end];
        nonnegative_prefix_digits(self.negative, digits)
    }
}

fn nonnegative_prefix_digits(negative: bool, digits: &[u16]) -> Result<&[u16], NumericParseError> {
    if negative && digits.iter().any(|&unit| unit != b'0' as u16) {
        Err(NumericParseError::Negative)
    } else {
        Ok(digits)
    }
}

fn integer_prefix(units: &[u16], recover: bool) -> Result<IntegerPrefix<'_>, NumericParseError> {
    let mut scanner = IntegerPrefixScanner::new(recover);
    for &unit in units {
        if !scanner.feed(unit) {
            break;
        }
    }
    let prefix = scanner.finish()?;
    Ok(IntegerPrefix {
        negative: prefix.negative,
        digits: &units[prefix.start..prefix.end],
    })
}

fn integer(units: &[u16], recover: bool) -> Result<HtmlInteger, NumericParseError> {
    let prefix = integer_prefix(units, recover)?;
    Ok(HtmlInteger::from_magnitude(
        HtmlNonNegativeInteger::from_digits(prefix.digits),
        prefix.negative,
    ))
}

/// Recovery rejection and allocation admission are separate source outcomes.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum IntegerRecoveryError {
    Syntax(NumericParseError),
    Allocation(IntegerUnitStepError),
}

/// §2.3.4.1 recovery with one fallible canonical digit allocation.
///
/// The caller admits raw UTF-16 units, precharges a three-input-length envelope
/// plus fixed work, and owns cancellation. This atomic helper scans the prefix,
/// scans its leading zeroes, and writes at most the input length. It performs no
/// intermediate integer construction, formatting, or unchecked string clone.
pub(crate) fn try_parse_integer(
    input: &DocumentString,
    max_result_digits: usize,
    max_capacity_bytes: usize,
) -> Result<IntegerUnitStepResult, IntegerRecoveryError> {
    let prefix = integer_prefix(input.code_units(), true).map_err(IntegerRecoveryError::Syntax)?;
    // The scanner proves at least one ASCII digit. All-zero input retains its
    // last zero, so canonical zero also requires exactly one output digit.
    let first = prefix
        .digits
        .iter()
        .position(|&unit| unit != b'0' as u16)
        .unwrap_or(prefix.digits.len() - 1);
    let canonical = &prefix.digits[first..];
    let mut digits = reserve_integer_digits(canonical.len(), max_result_digits, max_capacity_bytes)
        .map_err(IntegerRecoveryError::Allocation)?;
    for &unit in canonical {
        digits.push(char::from(unit as u8));
    }
    let capacity_bytes = digits.capacity();
    Ok(IntegerUnitStepResult {
        value: HtmlInteger::from_magnitude(HtmlNonNegativeInteger { digits }, prefix.negative),
        capacity_bytes,
    })
}

/// Fixed signed-long projection of the shared integer recovery prefix.
/// `None` is a successfully parsed mathematical integer outside i32, not syntax
/// failure. No canonical digit allocation or arbitrary integer digit cap.
pub(crate) fn parse_integer_long(input: &DocumentString) -> Result<Option<i32>, NumericParseError> {
    let prefix = integer_prefix(input.code_units(), true)?;
    let limit = if prefix.negative {
        2_147_483_648_u32
    } else {
        2_147_483_647_u32
    };
    let mut magnitude = 0_u32;
    for &unit in prefix.digits {
        let Some(next) = magnitude
            .checked_mul(10)
            .and_then(|n| n.checked_add(u32::from(unit - b'0' as u16)))
        else {
            return Ok(None);
        };
        if next > limit {
            return Ok(None);
        }
        magnitude = next;
    }
    Ok(Some(if prefix.negative {
        if magnitude == 2_147_483_648 {
            i32::MIN
        } else {
            -(magnitude as i32)
        }
    } else {
        magnitude as i32
    }))
}

/// Input-bounded atomic operation envelope: prefix scan <= n+2 inspections,
/// projection <= n digit reads plus subtraction, multiply, add and range checks;
/// fixed sign, length, result and branch operations fit the remaining allowance.
/// Caller admits raw units and precharges/polls; no instruction/quantum claim.
pub(crate) fn integer_long_work_bound(input_units: usize) -> Option<usize> {
    input_units.checked_mul(8)?.checked_add(16)
}

/// §2.3.4.2 recovery projected to `min(exact_value, cap)` without allocation.
///
/// Negative nonzero input is rejected before projection, including when the cap
/// is zero. The caller admits and precharges raw input inspection; two scans of
/// its units suffice, and the fixed-width arithmetic cannot overflow.
pub(crate) fn parse_nonnegative_integer_capped(
    input: &DocumentString,
    cap: usize,
) -> Result<usize, NumericParseError> {
    let prefix = integer_prefix(input.code_units(), true)?;
    let digits = nonnegative_prefix_digits(prefix.negative, prefix.digits)?;
    let mut value = 0_usize;
    for &unit in digits {
        let digit = usize::from(unit - b'0' as u16);
        value = value
            .checked_mul(10)
            .and_then(|value| value.checked_add(digit))
            .map_or(cap, |value| value.min(cap));
        if value == cap {
            return Ok(cap);
        }
    }
    Ok(value)
}

/// Author grammar: one or more ASCII digits, without either sign.
pub(crate) fn validate_nonnegative_integer(
    input: &DocumentString,
) -> Result<HtmlNonNegativeInteger, NumericParseError> {
    let units = input.code_units();
    if units.is_empty() || !units.iter().copied().all(is_digit) {
        return Err(NumericParseError::InvalidSyntax);
    }
    Ok(HtmlNonNegativeInteger::from_digits(units))
}

/// §2.3.4.2 recovery rejects negative values, while accepting recovered -0.
pub(crate) fn parse_nonnegative_integer(
    input: &DocumentString,
) -> Result<HtmlNonNegativeInteger, NumericParseError> {
    let prefix = integer_prefix(input.code_units(), true)?;
    Ok(HtmlNonNegativeInteger::from_digits(
        nonnegative_prefix_digits(prefix.negative, prefix.digits)?,
    ))
}

/// Author grammar, followed by the binary64 conversion contract.
/// Grammatical overflow is distinct from invalid author syntax.
pub(crate) fn validate_floating_point_number(
    input: &DocumentString,
) -> Result<HtmlFloatingPointNumber, NumericParseError> {
    floating_point(input.code_units(), false)
}

/// §2.3.4.3 recovery, including ignored leading plus and incomplete exponents.
pub(crate) fn parse_floating_point_number(
    input: &DocumentString,
) -> Result<HtmlFloatingPointNumber, NumericParseError> {
    floating_point(input.code_units(), true)
}

/// The source's author grammar and user-agent recovery remain distinct modes.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum FloatingPointParseMode {
    Author,
    Recovery,
}

/// The conversion outcome and capacity of its already-dropped temporary token.
/// Numeric rejection after allocation must still be accounted by the caller.
#[derive(Debug)]
pub(crate) struct FloatingPointParseResult {
    pub(crate) value: Result<HtmlFloatingPointNumber, NumericParseError>,
    pub(crate) capacity_bytes: usize,
}

/// Allocation admission failures, distinct from source numeric rejection.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum FloatingPointAllocationError {
    RepresentationExhausted,
    RequestedCapacityLimitExceeded {
        limit: usize,
        required: usize,
    },
    ActualCapacityLimitExceeded {
        limit: usize,
        requested: usize,
        capacity_bytes: usize,
    },
    AllocationFailed {
        requested: usize,
    },
}

/// Mathematical decimal equality before HTML's distinct binary64 conversion.
///
/// Nonzero significands have no leading or trailing zeroes. The value is the
/// signed significand times ten to the exact integer exponent. Zero has one
/// canonical key, independently of its authored sign and exponent. This is
/// private author-constraint machinery, not another public numeric scalar.
#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub(crate) struct HtmlScientificDecimalKey {
    negative: bool,
    significand: HtmlNonNegativeInteger,
    exponent: HtmlInteger,
}

impl HtmlScientificDecimalKey {
    pub(crate) fn is_zero(&self) -> bool {
        self.significand.is_zero()
    }

    pub(crate) fn is_positive(&self) -> bool {
        !self.negative && !self.is_zero()
    }

    /// Canonical nonzero significands have no trailing zeroes, so a negative
    /// normalized exponent cannot describe an integer. Exact zero is integral.
    pub(crate) fn is_integer(&self) -> bool {
        self.is_zero() || !self.exponent.is_negative()
    }

    /// Compares mathematical decimals with storage bounded by authored digits.
    /// Decimal orders use one checked exact integer adjustment; significands
    /// are compared with virtual trailing zeroes, never exponent expansion.
    pub(crate) fn compare(&self, other: &Self) -> Result<Ordering, ScientificDecimalKeyError> {
        if self.is_zero() || other.is_zero() {
            return Ok(match (self.is_zero(), other.is_zero()) {
                (true, true) => Ordering::Equal,
                (true, false) => {
                    if other.negative {
                        Ordering::Greater
                    } else {
                        Ordering::Less
                    }
                }
                (false, true) => {
                    if self.negative {
                        Ordering::Less
                    } else {
                        Ordering::Greater
                    }
                }
                (false, false) => unreachable!(),
            });
        }
        if self.negative != other.negative {
            return Ok(if self.negative {
                Ordering::Less
            } else {
                Ordering::Greater
            });
        }
        let left = self.significand.decimal_digits();
        let right = other.significand.decimal_digits();
        let left_order = adjust_scientific_exponent(self.exponent.clone(), 0, left.len())?;
        let right_order = adjust_scientific_exponent(other.exponent.clone(), 0, right.len())?;
        let mut order = left_order.compare(&right_order);
        if order == Ordering::Equal {
            order = left
                .bytes()
                .chain(std::iter::repeat(b'0'))
                .take(left.len().max(right.len()))
                .cmp(
                    right
                        .bytes()
                        .chain(std::iter::repeat(b'0'))
                        .take(left.len().max(right.len())),
                );
        }
        Ok(if self.negative {
            order.reverse()
        } else {
            order
        })
    }
}

/// Native representation exhaustion is not a rejection of decimal grammar.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum ScientificDecimalKeyError {
    Syntax(NumericParseError),
    RepresentationExhausted,
}

/// Checks the same complete author grammar, without binary64 rounding.
///
/// This standalone producer uses ordinary native allocation. Checked native
/// representation arithmetic is fallible, but allocator exhaustion is not a
/// recoverable error here. Neither the key nor its raw-input cap establishes a
/// physical-memory ledger, work budget, or cancellation bound. Storage depends
/// on input digit lengths, never the mathematical exponent's magnitude.
pub(crate) fn strict_scientific_decimal_key(
    input: &DocumentString,
) -> Result<HtmlScientificDecimalKey, ScientificDecimalKeyError> {
    strict_scientific_decimal_key_units(input.code_units())
}

/// Borrowed token entry for common-list consumers; the scanner is shared with
/// scalar author parsing and no substring DocumentString allocation is needed.
pub(crate) fn strict_scientific_decimal_key_units(
    input: &[u16],
) -> Result<HtmlScientificDecimalKey, ScientificDecimalKeyError> {
    let prefix = floating_point_prefix(input, FloatingPointParseMode::Author)
        .map_err(ScientificDecimalKeyError::Syntax)?;
    let digit_count = prefix
        .integer_digits
        .len()
        .checked_add(prefix.fraction_digits.len())
        .ok_or(ScientificDecimalKeyError::RepresentationExhausted)?;
    let mut first = None;
    let mut last = 0;
    for (index, &unit) in prefix
        .integer_digits
        .iter()
        .chain(prefix.fraction_digits)
        .enumerate()
    {
        if unit != u16::from(b'0') {
            first.get_or_insert(index);
            last = index;
        }
    }
    let Some(first) = first else {
        // The whole grammar, including an arbitrarily long exponent, was
        // checked before this shortcut. Its value cannot change exact zero.
        return Ok(HtmlScientificDecimalKey {
            negative: false,
            significand: 0.into(),
            exponent: 0.into(),
        });
    };
    let required = last - first + 1;
    if required > isize::MAX as usize {
        return Err(ScientificDecimalKeyError::RepresentationExhausted);
    }
    let mut digits = String::with_capacity(required);
    for &unit in prefix
        .integer_digits
        .iter()
        .chain(prefix.fraction_digits)
        .skip(first)
        .take(required)
    {
        digits.push(char::from(unit as u8));
    }
    let exponent = prefix.exponent.map_or_else(
        || HtmlInteger::from(0),
        |exponent| {
            HtmlInteger::from_magnitude(
                HtmlNonNegativeInteger::from_digits(exponent.digits),
                exponent.negative,
            )
        },
    );
    let trailing_zeroes = digit_count - last - 1;
    let exponent =
        adjust_scientific_exponent(exponent, prefix.fraction_digits.len(), trailing_zeroes)?;
    Ok(HtmlScientificDecimalKey {
        negative: prefix.negative,
        significand: HtmlNonNegativeInteger { digits },
        exponent,
    })
}

fn adjust_scientific_exponent(
    exponent: HtmlInteger,
    fraction_digits: usize,
    trailing_zeroes: usize,
) -> Result<HtmlInteger, ScientificDecimalKeyError> {
    if fraction_digits == trailing_zeroes {
        return Ok(exponent);
    }
    let (negative, adjustment) = if fraction_digits > trailing_zeroes {
        (true, fraction_digits - trailing_zeroes)
    } else {
        (false, trailing_zeroes - fraction_digits)
    };
    // usize counts are bounded native metadata; only the authored exponent has
    // arbitrary magnitude. One exact adjustment suffices, without repeated ±1.
    let adjustment = HtmlNonNegativeInteger::from(adjustment as u128);
    if exponent.magnitude.is_zero() {
        return Ok(HtmlInteger::from_magnitude(adjustment, negative));
    }
    if exponent.negative == negative {
        integer_sum_capacity(
            exponent.magnitude.decimal_digits().len(),
            adjustment.decimal_digits().len(),
        )
        .ok_or(ScientificDecimalKeyError::RepresentationExhausted)?;
        return Ok(HtmlInteger::from_magnitude(
            exponent.magnitude.added_to(&adjustment),
            negative,
        ));
    }
    Ok(match exponent.magnitude.compare(&adjustment) {
        Ordering::Less => HtmlInteger::from_magnitude(
            adjustment.subtract_with_borrow(&exponent.magnitude, 0),
            negative,
        ),
        Ordering::Equal => HtmlInteger::from(0),
        Ordering::Greater => HtmlInteger::from_magnitude(
            exponent.magnitude.subtract_with_borrow(&adjustment, 0),
            exponent.negative,
        ),
    })
}

fn floating_point(
    units: &[u16],
    recover: bool,
) -> Result<HtmlFloatingPointNumber, NumericParseError> {
    let mode = if recover {
        FloatingPointParseMode::Recovery
    } else {
        FloatingPointParseMode::Author
    };
    let prefix = floating_point_prefix(units, mode)?;
    // Preserve all decimal digits until the single final binary64 conversion.
    // Intermediate floating accumulation can produce spurious overflow for a
    // long significand with a compensating exponent.
    let token: String = prefix
        .token
        .iter()
        .map(|&unit| char::from(unit as u8))
        .collect();
    convert_floating_point_token(&token)
}

/// Borrowed components recognized by the single strict/recovery grammar owner.
struct FloatingPointPrefix<'a> {
    token: &'a [u16],
    negative: bool,
    integer_digits: &'a [u16],
    fraction_digits: &'a [u16],
    exponent: Option<IntegerPrefix<'a>>,
}

/// One bounded grammar action per advance; no conversion or allocation.
/// Author mode recognizes the complete HTML floating-point grammar, independently
/// of whether its decimal value can be represented as a finite binary64 number.
#[derive(Clone, Copy, Debug)]
pub(crate) struct FloatingPointGrammarCursor<'a> {
    units: &'a [u16],
    mode: FloatingPointParseMode,
    step: FloatingGrammarStep,
    position: usize,
    start: usize,
    integer_start: usize,
    integer_end: usize,
    fraction_start: usize,
    fraction_end: usize,
    significand_end: usize,
    exponent_start: usize,
    negative: bool,
    exponent_negative: bool,
    has_exponent: bool,
    valid: bool,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum FloatingGrammarStep {
    Whitespace,
    Sign,
    Integer,
    Point,
    Fraction,
    Exponent,
    ExponentSign,
    ExponentDigits,
    Finish,
    Done,
}

impl<'a> FloatingPointGrammarCursor<'a> {
    pub(crate) fn new(units: &'a [u16], mode: FloatingPointParseMode) -> Self {
        Self {
            units,
            mode,
            step: FloatingGrammarStep::Whitespace,
            position: 0,
            start: 0,
            integer_start: 0,
            integer_end: 0,
            fraction_start: 0,
            fraction_end: 0,
            significand_end: 0,
            exponent_start: 0,
            negative: false,
            exponent_negative: false,
            has_exponent: false,
            valid: false,
        }
    }

    pub(crate) fn is_finished(&self) -> bool {
        self.step == FloatingGrammarStep::Done
    }

    pub(crate) fn is_valid(&self) -> bool {
        self.is_finished() && self.valid
    }

    pub(crate) fn inspects_unit(&self) -> bool {
        self.position < self.units.len()
            && match self.step {
                FloatingGrammarStep::Whitespace => self.mode == FloatingPointParseMode::Recovery,
                FloatingGrammarStep::Sign
                | FloatingGrammarStep::Integer
                | FloatingGrammarStep::Point
                | FloatingGrammarStep::Fraction
                | FloatingGrammarStep::Exponent
                | FloatingGrammarStep::ExponentSign
                | FloatingGrammarStep::ExponentDigits => true,
                FloatingGrammarStep::Finish | FloatingGrammarStep::Done => false,
            }
    }

    fn reject(&mut self) {
        self.valid = false;
        self.step = FloatingGrammarStep::Done;
    }

    pub(crate) fn advance(&mut self) {
        use FloatingGrammarStep::*;
        match self.step {
            Whitespace => {
                if self.mode == FloatingPointParseMode::Recovery
                    && self
                        .units
                        .get(self.position)
                        .is_some_and(|&u| is_ascii_whitespace(u))
                {
                    self.position += 1;
                } else {
                    self.start = self.position;
                    self.step = Sign;
                }
            }
            Sign => {
                let unit = self.units.get(self.position).copied();
                self.negative = unit == Some(b'-' as u16);
                if self.negative
                    || (self.mode == FloatingPointParseMode::Recovery && unit == Some(b'+' as u16))
                {
                    self.position += 1;
                }
                self.integer_start = self.position;
                self.step = Integer;
            }
            Integer => {
                if self.units.get(self.position).is_some_and(|&u| is_digit(u)) {
                    self.position += 1;
                } else {
                    self.integer_end = self.position;
                    self.fraction_start = self.position;
                    self.fraction_end = self.position;
                    self.step = Point;
                }
            }
            Point => {
                if self.units.get(self.position) == Some(&(b'.' as u16)) {
                    self.position += 1;
                    self.fraction_start = self.position;
                    self.step = Fraction;
                } else if self.integer_end == self.integer_start {
                    self.reject();
                } else {
                    self.significand_end = self.position;
                    self.step = Exponent;
                }
            }
            Fraction => {
                if self.units.get(self.position).is_some_and(|&u| is_digit(u)) {
                    self.position += 1;
                } else {
                    self.fraction_end = self.position;
                    if self.fraction_end == self.fraction_start
                        && (self.mode == FloatingPointParseMode::Author
                            || self.integer_end == self.integer_start)
                    {
                        self.reject();
                    } else {
                        self.significand_end = self.position;
                        self.step = Exponent;
                    }
                }
            }
            Exponent => {
                if matches!(self.units.get(self.position), Some(&0x45) | Some(&0x65)) {
                    self.position += 1;
                    self.step = ExponentSign;
                } else {
                    self.step = Finish;
                }
            }
            ExponentSign => {
                let unit = self.units.get(self.position).copied();
                self.exponent_negative = unit == Some(b'-' as u16);
                if matches!(unit, Some(0x2b) | Some(0x2d)) {
                    self.position += 1;
                }
                self.exponent_start = self.position;
                self.step = ExponentDigits;
            }
            ExponentDigits => {
                if self.units.get(self.position).is_some_and(|&u| is_digit(u)) {
                    self.position += 1;
                } else if self.position == self.exponent_start {
                    if self.mode == FloatingPointParseMode::Author {
                        self.reject();
                    } else {
                        self.position = self.significand_end;
                        self.step = Finish;
                    }
                } else {
                    self.has_exponent = true;
                    self.step = Finish;
                }
            }
            Finish => {
                self.valid = self.mode == FloatingPointParseMode::Recovery
                    || self.position == self.units.len();
                self.step = Done;
            }
            Done => {}
        }
    }

    /// Issued only by the completed shared grammar. The numeric executor owns
    /// offsets and signs; its adapter retains and qualifies the selected source.
    pub(crate) fn range_decimal_parts(
        &self,
    ) -> Result<range::RangeDecimalParts, NumericParseError> {
        if !self.is_valid() {
            return Err(NumericParseError::InvalidSyntax);
        }
        Ok(range::RangeDecimalParts {
            negative: self.negative,
            integer_start: self.integer_start,
            integer_end: self.integer_end,
            fraction_start: self.fraction_start,
            fraction_end: self.fraction_end,
            exponent_start: if self.has_exponent {
                self.exponent_start
            } else {
                0
            },
            exponent_end: if self.has_exponent { self.position } else { 0 },
            exponent_negative: self.exponent_negative,
        })
    }

    fn prefix(&self) -> Result<FloatingPointPrefix<'a>, NumericParseError> {
        if !self.is_valid() {
            return Err(NumericParseError::InvalidSyntax);
        }
        Ok(FloatingPointPrefix {
            token: &self.units[self.start..self.position],
            negative: self.negative,
            integer_digits: &self.units[self.integer_start..self.integer_end],
            fraction_digits: &self.units[self.fraction_start..self.fraction_end],
            exponent: self.has_exponent.then(|| IntegerPrefix {
                negative: self.exponent_negative,
                digits: &self.units[self.exponent_start..self.position],
            }),
        })
    }
}

fn floating_point_prefix(
    units: &[u16],
    mode: FloatingPointParseMode,
) -> Result<FloatingPointPrefix<'_>, NumericParseError> {
    let mut cursor = FloatingPointGrammarCursor::new(units, mode);
    while !cursor.is_finished() {
        cursor.advance();
    }
    cursor.prefix()
}

fn convert_floating_point_token(token: &str) -> Result<HtmlFloatingPointNumber, NumericParseError> {
    let value = token
        .parse::<f64>()
        .map_err(|_| NumericParseError::InvalidSyntax)?;
    HtmlFloatingPointNumber::try_from(value).map_err(|_| NumericParseError::FloatingPointOverflow)
}

fn check_floating_point_requested_capacity(
    required: usize,
    max_capacity_bytes: usize,
) -> Result<(), FloatingPointAllocationError> {
    if required > isize::MAX as usize {
        return Err(FloatingPointAllocationError::RepresentationExhausted);
    }
    if required > max_capacity_bytes {
        return Err(
            FloatingPointAllocationError::RequestedCapacityLimitExceeded {
                limit: max_capacity_bytes,
                required,
            },
        );
    }
    Ok(())
}

fn check_floating_point_actual_capacity(
    requested: usize,
    capacity_bytes: usize,
    max_capacity_bytes: usize,
) -> Result<(), FloatingPointAllocationError> {
    if capacity_bytes > max_capacity_bytes {
        Err(FloatingPointAllocationError::ActualCapacityLimitExceeded {
            limit: max_capacity_bytes,
            requested,
            capacity_bytes,
        })
    } else {
        Ok(())
    }
}

/// Converts the same source-selected ASCII prefix using one fallible reserve.
///
/// The semantic session admits the raw input and owns work/cancellation. Prefix
/// inspection and writing are input-length bounded; native binary64 conversion
/// is an atomic provider operation, without an internal cancellation poll or a
/// parser-quantum guarantee. This returns actual temporary String capacity even
/// when numeric conversion rejects the input; pre-allocation syntax rejection
/// reports zero capacity. No raw copy, report, or second grammar is constructed.
pub(crate) fn try_floating_point_number(
    input: &DocumentString,
    mode: FloatingPointParseMode,
    max_capacity_bytes: usize,
) -> Result<FloatingPointParseResult, FloatingPointAllocationError> {
    let prefix = match floating_point_prefix(input.code_units(), mode) {
        Ok(prefix) => prefix,
        Err(error) => {
            return Ok(FloatingPointParseResult {
                value: Err(error),
                capacity_bytes: 0,
            });
        }
    };
    let required = prefix.token.len();
    check_floating_point_requested_capacity(required, max_capacity_bytes)?;
    let mut token = String::new();
    token.try_reserve_exact(required).map_err(|_| {
        FloatingPointAllocationError::AllocationFailed {
            requested: required,
        }
    })?;
    check_floating_point_actual_capacity(required, token.capacity(), max_capacity_bytes)?;
    for &unit in prefix.token {
        token.push(char::from(unit as u8));
    }
    Ok(FloatingPointParseResult {
        value: convert_floating_point_token(&token),
        capacity_bytes: token.capacity(),
    })
}

/// Author list grammar is comma-separated author numbers, without whitespace.
pub(crate) fn validate_floating_point_list(
    input: &DocumentString,
) -> Result<HtmlFloatingPointList, NumericParseError> {
    if input.is_empty() {
        return Ok(HtmlFloatingPointList::default());
    }
    let mut values = Vec::new();
    let mut overflow = false;
    for units in input.code_units().split(|&unit| unit == b',' as u16) {
        match floating_point(units, false) {
            Ok(value) => values.push(value),
            Err(NumericParseError::FloatingPointOverflow) => overflow = true,
            Err(error) => return Err(error),
        }
    }
    // Overflow describes a representational failure only when the complete
    // list satisfies author grammar. A later invalid token takes precedence.
    if overflow {
        Err(NumericParseError::FloatingPointOverflow)
    } else {
        Ok(HtmlFloatingPointList::new(values))
    }
}

/// §2.3.4.6 recovery preserves token order and substitutes zero on failure.
pub(crate) fn parse_floating_point_list(input: &DocumentString) -> HtmlFloatingPointList {
    let units = input.code_units();
    let mut position = 0;
    let mut values = Vec::new();
    let is_delimiter = |unit| is_ascii_whitespace(unit) || matches!(unit, 0x2c | 0x3b);
    while units.get(position).is_some_and(|&u| is_delimiter(u)) {
        position += 1;
    }
    while position < units.len() {
        while units
            .get(position)
            .is_some_and(|&u| !is_delimiter(u) && !is_digit(u) && !matches!(u, 0x2e | 0x2d))
        {
            position += 1;
        }
        let start = position;
        while units.get(position).is_some_and(|&u| !is_delimiter(u)) {
            position += 1;
        }
        values.push(
            floating_point(&units[start..position], true)
                .unwrap_or(HtmlFloatingPointNumber { value: 0.0 }),
        );
        while units.get(position).is_some_and(|&u| is_delimiter(u)) {
            position += 1;
        }
    }
    HtmlFloatingPointList::new(values)
}

/// §2.3.4.4 digit-led dimension recovery, without an invented author grammar.
pub(crate) fn parse_dimension(input: &DocumentString) -> Result<HtmlDimension, NumericParseError> {
    let units = input.code_units();
    let mut position = 0;
    skip_whitespace(units, &mut position);
    let start = position;
    collect_digits(units, &mut position);
    if start == position {
        return Err(NumericParseError::InvalidSyntax);
    }
    let integer = HtmlNonNegativeInteger::from_digits(&units[start..position]);
    let mut fraction = &[][..];
    if units.get(position) == Some(&(b'.' as u16)) {
        position += 1;
        let start = position;
        collect_digits(units, &mut position);
        fraction = &units[start..position];
    }
    let value = HtmlNonNegativeDecimal::from_parts(integer, fraction);
    if units.get(position) == Some(&(b'%' as u16)) {
        Ok(HtmlDimension::Percentage(value))
    } else {
        Ok(HtmlDimension::Absolute(value))
    }
}

/// §2.3.4.5 rejects exact zero, including arbitrarily small nonzero decimals.
pub(crate) fn parse_nonzero_dimension(
    input: &DocumentString,
) -> Result<HtmlDimension, NumericParseError> {
    let dimension = parse_dimension(input)?;
    if dimension.value().is_zero() {
        Err(NumericParseError::ZeroDimension)
    } else {
        Ok(dimension)
    }
}

/// §2.3.4.7 with the selected #1133 decimal-cursor correction.
///
/// [Retained cursor witness](../../../references/html-dimension-list-cursor--webkit-73aa6c89e2cb-blink-7984f9d11800.md).
///
/// Advance once after recognizing `.` before collecting whitespace/digits.
/// All other pinned steps remain unchanged, including Infra comma trimming,
/// exact decimals, empty-token relative zero, and prefix unit categorization.
pub(crate) fn parse_dimension_list(input: &DocumentString) -> HtmlDimensionList {
    let units = input.code_units();
    let units = units.strip_suffix(&[b',' as u16]).unwrap_or(units);
    let mut position = 0;
    let mut values = Vec::new();
    // Infra split-on-commas does not emit a token once the cursor reaches end.
    while position < units.len() {
        let start = position;
        while units.get(position).is_some_and(|&unit| unit != b',' as u16) {
            position += 1;
        }
        let mut token = &units[start..position];
        while token.first().is_some_and(|&unit| is_ascii_whitespace(unit)) {
            token = &token[1..];
        }
        while token.last().is_some_and(|&unit| is_ascii_whitespace(unit)) {
            token = &token[..token.len() - 1];
        }
        values.push(dimension_list_token(token));
        if position < units.len() {
            position += 1;
        }
    }
    HtmlDimensionList::new(values)
}

fn dimension_list_token(units: &[u16]) -> HtmlDimension {
    if units.is_empty() {
        return HtmlDimension::Relative(HtmlNonNegativeInteger::from(0).into());
    }
    let mut position = 0;
    collect_digits(units, &mut position);
    let integer = HtmlNonNegativeInteger::from_digits(&units[..position]);
    let mut fraction = Vec::new();
    if units.get(position) == Some(&(b'.' as u16)) {
        position += 1; // Selected correction to the pinned step 5.7.
        while let Some(&unit) = units.get(position) {
            if is_digit(unit) {
                fraction.push(unit);
            } else if !is_ascii_whitespace(unit) {
                break;
            }
            position += 1;
        }
    }
    skip_whitespace(units, &mut position);
    let value = HtmlNonNegativeDecimal::from_parts(integer, &fraction);
    match units.get(position) {
        Some(0x25) => HtmlDimension::Percentage(value),
        Some(0x2a) => HtmlDimension::Relative(value),
        _ => HtmlDimension::Absolute(value),
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn integer_unit_scanner_keeps_author_and_recovery_prefixes_distinct() {
        use super::{IntegerPrefixScanner, NumericParseError, ScannedIntegerPrefix};
        for (raw, recover, expected) in [
            (" \t+0012suffix", true, Ok((false, 3, 7))),
            ("-000tail", true, Ok((true, 1, 4))),
            ("12tail", false, Err(NumericParseError::InvalidSyntax)),
            ("+12", false, Err(NumericParseError::InvalidSyntax)),
            ("-12", false, Ok((true, 1, 3))),
            ("-", true, Err(NumericParseError::InvalidSyntax)),
            (" \t", true, Err(NumericParseError::InvalidSyntax)),
        ] {
            let mut scanner = IntegerPrefixScanner::new(recover);
            for unit in raw.encode_utf16() {
                if !scanner.feed(unit) {
                    break;
                }
            }
            assert_eq!(
                scanner.finish(),
                expected.map(|(negative, start, end)| ScannedIntegerPrefix {
                    negative,
                    start,
                    end,
                }),
                "{raw:?} recovery={recover}",
            );
        }
    }

    #[test]
    fn signed_long_projection_literal_boundaries_and_recovery() {
        for (raw, expected) in [
            ("2147483647", Some(i32::MAX)),
            ("-2147483648", Some(i32::MIN)),
            ("2147483648", None),
            ("-2147483649", None),
            ("999999999999999999999999999999999999999", None),
            ("-0", Some(0)),
            ("+0", Some(0)),
            (" \t\n\u{000c}\r+12suffix", Some(12)),
            ("-42.5", Some(-42)),
            ("0000000000000000000002147483647", Some(i32::MAX)),
            ("-0000000000000000000002147483648", Some(i32::MIN)),
        ] {
            assert_eq!(
                parse_integer_long(&DocumentString::from(raw)),
                Ok(expected),
                "{raw:?}"
            );
        }
        for raw in ["", "-", "+", "--1", "\u{00a0}1", "x12"] {
            assert_eq!(
                parse_integer_long(&DocumentString::from(raw)),
                Err(NumericParseError::InvalidSyntax)
            );
        }
        assert_eq!(
            parse_integer_long(&DocumentString::from_utf16(vec![0xd800, 49])),
            Err(NumericParseError::InvalidSyntax)
        );
        assert_eq!(
            parse_integer_long(&DocumentString::from_utf16(vec![49, 0xd800])),
            Ok(Some(1))
        );
    }
    #[test]
    fn signed_long_projection_work_bound_checks_representation() {
        assert_eq!(integer_long_work_bound(0), Some(16));
        assert_eq!(integer_long_work_bound(5), Some(56));
        assert_eq!(integer_long_work_bound(usize::MAX), None);
    }

    use super::*;

    fn digits(input: &str) -> Vec<u16> {
        input.encode_utf16().collect()
    }

    #[test]
    fn materialized_borrowed_digits_are_canonical_without_bounded_year_projection() {
        for (input, expected) in [
            ("0000", "0"),
            ("0001", "1"),
            (
                "340282366920938463463374607431768211456",
                "340282366920938463463374607431768211456",
            ),
        ] {
            let value = HtmlNonNegativeInteger::try_from_borrowed_digits(
                &digits(input),
                expected.len(),
                100,
            )
            .unwrap();
            assert_eq!(value.value.decimal_digits(), expected);
            assert!(value.capacity_bytes >= expected.len());
            let copied = value.value.try_copy(expected.len(), 100).unwrap();
            assert_eq!(copied.value.decimal_digits(), expected);
            assert!(copied.capacity_bytes >= expected.len());
        }
        let invalid =
            HtmlNonNegativeInteger::try_from_borrowed_digits(&[49, 0xd800], 2, 2).unwrap_err();
        assert_eq!(
            invalid.kind,
            NumericMaterializationError::Digits(HtmlDecimalDigitsError::NonAsciiDigit {
                code_unit_offset: 1,
                code_unit: 0xd800
            })
        );
        assert_eq!(invalid.capacity_bytes, 0);
        assert_eq!(
            HtmlNonNegativeInteger::try_from_borrowed_digits(&[], 0, 0)
                .unwrap_err()
                .kind,
            NumericMaterializationError::Digits(HtmlDecimalDigitsError::Empty)
        );
    }

    #[test]
    fn materialized_fixed_constants_decrement_and_signed_day_steps_cover_zero_and_carry() {
        for (number, expected) in [
            (0, "0"),
            (400, "400"),
            (146097, "146097"),
            (u32::MAX, "4294967295"),
        ] {
            let value = HtmlNonNegativeInteger::try_from_u32(number, expected.len(), 100).unwrap();
            assert_eq!(value.value.decimal_digits(), expected);
            assert!(value.capacity_bytes >= expected.len());
        }
        let zero = HtmlNonNegativeInteger::from(0);
        let none = zero.try_decremented(0, 0).unwrap();
        assert_eq!(none.value, None);
        assert_eq!(none.capacity_bytes, 0);
        for (input, expected) in [
            ("1", "0"),
            ("1000", "999"),
            (
                "340282366920938463463374607431768211456",
                "340282366920938463463374607431768211455",
            ),
        ] {
            let input = HtmlNonNegativeInteger::from_decimal_digits(&text(input)).unwrap();
            assert_eq!(
                input
                    .try_decremented(expected.len(), 100)
                    .unwrap()
                    .value
                    .unwrap()
                    .decimal_digits(),
                expected
            );
        }
        let day = HtmlInteger::from_magnitude(zero, false);
        let previous = day
            .try_materialized_unit_step(IntegerUnitStep::Decrement, 1, 1)
            .unwrap();
        assert_eq!(previous.value.as_document_string(), text("-1"));
        assert_eq!(
            previous
                .value
                .try_materialized_unit_step(IntegerUnitStep::Increment, 1, 1)
                .unwrap()
                .value
                .as_document_string(),
            text("0")
        );
    }

    #[test]
    fn materialized_cycle_arithmetic_has_literal_unbounded_quotient_and_product() {
        let large = HtmlNonNegativeInteger::from_decimal_digits(&text(
            "340282366920938463463374607431768211456",
        ))
        .unwrap();
        let divided = large
            .try_quotient_remainder(NonZeroU32::new(400).unwrap(), 36, 100)
            .unwrap();
        assert_eq!(
            divided.value.0.decimal_digits(),
            "850705917302346158658436518579420528"
        );
        assert_eq!(divided.value.1, 256);
        assert_eq!(
            borrowed_decimal_remainder(
                &digits("000340282366920938463463374607431768211456"),
                NonZeroU32::new(400).unwrap()
            ),
            256
        );
        let cycle = HtmlNonNegativeInteger::from(400);
        assert_eq!(
            cycle
                .try_multiplied_by(146097, 8, 8)
                .unwrap()
                .value
                .decimal_digits(),
            "58438800"
        );
        let carry = HtmlNonNegativeInteger::from(999);
        assert_eq!(
            carry
                .try_added_to(&HtmlNonNegativeInteger::from(1), 4, 4)
                .unwrap()
                .value
                .decimal_digits(),
            "1000"
        );
        assert_eq!(
            carry
                .try_multiplied_by(u32::MAX, 13, 13)
                .unwrap()
                .value
                .decimal_digits(),
            "4290672327705"
        );
        assert_eq!(
            large
                .try_multiplied_by(0, 1, 1)
                .unwrap()
                .value
                .decimal_digits(),
            "0"
        );
        assert_eq!(
            HtmlNonNegativeInteger::from(399)
                .try_quotient_remainder(NonZeroU32::new(400).unwrap(), 1, 1)
                .unwrap()
                .value,
            (HtmlNonNegativeInteger::from(0), 399)
        );
    }

    #[test]
    fn materialized_integer_digit_and_requested_byte_caps_apply_to_actual_result() {
        let value = HtmlNonNegativeInteger::from(999);
        assert_eq!(
            value
                .try_added_to(&HtmlNonNegativeInteger::from(1), 3, 100)
                .unwrap_err(),
            NumericMaterializationFailure {
                kind: NumericMaterializationError::Boundary(
                    IntegerUnitStepError::DigitLimitExceeded {
                        limit: 3,
                        required: 4
                    }
                ),
                capacity_bytes: 0
            }
        );
        assert_eq!(
            value.try_multiplied_by(10, 4, 3).unwrap_err(),
            NumericMaterializationFailure {
                kind: NumericMaterializationError::Boundary(
                    IntegerUnitStepError::RequestedCapacityLimitExceeded {
                        limit: 3,
                        required: 4
                    }
                ),
                capacity_bytes: 0
            }
        );
        assert_eq!(
            HtmlNonNegativeInteger::try_from_u32(0, 0, 1)
                .unwrap_err()
                .kind,
            NumericMaterializationError::Boundary(IntegerUnitStepError::DigitLimitExceeded {
                limit: 0,
                required: 1
            })
        );
        let copied = value.try_copy(3, 100).unwrap();
        let exact = value.try_copy(3, copied.capacity_bytes).unwrap();
        assert_eq!(exact.value.decimal_digits(), "999");
        let short = value.try_copy(3, copied.capacity_bytes - 1).unwrap_err();
        assert!(matches!(
            short.kind,
            NumericMaterializationError::Boundary(
                IntegerUnitStepError::RequestedCapacityLimitExceeded { .. }
                    | IntegerUnitStepError::ActualCapacityLimitExceeded { .. }
            )
        ));
    }

    #[test]
    fn materialized_decimal_borrowed_fraction_and_copies_preserve_exact_places() {
        let value = HtmlNonNegativeDecimal::try_from_borrowed_digits(
            &digits("00012"),
            &digits("0001234500"),
            2,
            100,
        )
        .unwrap();
        assert_eq!(value.value.as_document_string(), text("12.00012345"));
        assert!(value.capacity_bytes >= 10);
        let copied = value.value.try_copy(2, 100).unwrap();
        assert_eq!(copied.value.as_document_string(), text("12.00012345"));
        assert!(copied.capacity_bytes >= 10);
        let integer =
            HtmlNonNegativeDecimal::try_from_borrowed_digits(&digits("001"), &[], 1, 1).unwrap();
        assert_eq!(integer.value.as_document_string(), text("1"));
        assert_eq!(integer.capacity_bytes, 1);
        let zero = HtmlNonNegativeDecimal::try_from_borrowed_digits(
            &digits("000"),
            &digits("000000"),
            1,
            1,
        )
        .unwrap();
        assert_eq!(zero.value.as_document_string(), text("0"));
        assert_eq!(zero.capacity_bytes, 1);
    }

    #[test]
    fn materialized_decimal_scale_and_add_keep_fractional_carry_and_zero_capacity_history() {
        let tiny = decimal("0", "0001");
        assert_eq!(
            tiny.try_multiplied_by(100, 1, 100)
                .unwrap()
                .value
                .as_document_string(),
            text("0.01")
        );
        let integral = tiny.try_multiplied_by(10000, 1, 100).unwrap();
        assert_eq!(integral.value.as_document_string(), text("1"));
        assert!(integral.capacity_bytes >= 5); // discarded four-place buffer + integer
        let carried = decimal("0", "9999")
            .try_added_to(&decimal("0", "0001"), 1, 100)
            .unwrap();
        assert_eq!(carried.value.as_document_string(), text("1"));
        assert!(carried.capacity_bytes >= 5);
        assert_eq!(
            decimal("0", "25")
                .try_multiplied_by(u32::MAX, 10, 100)
                .unwrap()
                .value
                .as_document_string(),
            text("1073741823.75")
        );
        assert_eq!(
            decimal("0", "0125")
                .try_added_to(&decimal("0", "5"), 1, 100)
                .unwrap()
                .value
                .as_document_string(),
            text("0.5125")
        );
        let zero = tiny.try_multiplied_by(0, 1, 1).unwrap();
        assert_eq!(zero.value.as_document_string(), text("0"));
        assert_eq!(zero.capacity_bytes, 1);
    }

    #[test]
    fn materialized_decimal_late_failures_retain_every_earlier_observed_buffer() {
        let syntax =
            HtmlNonNegativeDecimal::try_from_borrowed_digits(&digits("12"), &[0xdfff], 2, 100)
                .unwrap_err();
        assert_eq!(
            syntax.kind,
            NumericMaterializationError::Digits(HtmlDecimalDigitsError::NonAsciiDigit {
                code_unit_offset: 0,
                code_unit: 0xdfff
            })
        );
        assert!(syntax.capacity_bytes >= 2);
        let requested =
            HtmlNonNegativeDecimal::try_from_borrowed_digits(&digits("1"), &digits("123"), 1, 1)
                .unwrap_err();
        assert!(requested.capacity_bytes >= 1);
        assert_eq!(
            requested.kind,
            NumericMaterializationError::Boundary(
                IntegerUnitStepError::RequestedCapacityLimitExceeded {
                    limit: 1,
                    required: requested.capacity_bytes + 3
                }
            )
        );
        let copy = decimal("1", "123").try_copy(1, 1).unwrap_err();
        assert!(copy.capacity_bytes >= 1);
        for failure in [
            decimal("9", "9999")
                .try_multiplied_by(2, 1, 100)
                .unwrap_err(),
            decimal("9", "9999")
                .try_added_to(&decimal("0", "0001"), 1, 100)
                .unwrap_err(),
        ] {
            assert_eq!(
                failure.kind,
                NumericMaterializationError::Boundary(IntegerUnitStepError::DigitLimitExceeded {
                    limit: 1,
                    required: 2
                })
            );
            assert!(failure.capacity_bytes >= 4);
        }
    }

    #[test]
    fn materialization_reservation_checks_observed_overshoot_and_late_representation() {
        // This exercises the actual capacity guard independently of an allocator
        // whose try_reserve_exact may or may not overshoot on this host.
        assert_eq!(
            check_integer_actual_capacity(3, 4, 3),
            Err(IntegerUnitStepError::ActualCapacityLimitExceeded {
                limit: 3,
                requested: 3,
                capacity_bytes: 4
            })
        );
        let mut observed = 7;
        let representation =
            reserve_materialization(usize::MAX, None, usize::MAX, &mut observed).unwrap_err();
        assert_eq!(representation.capacity_bytes, 7);
        assert_eq!(
            representation.kind,
            NumericMaterializationError::Boundary(IntegerUnitStepError::RepresentationExhausted)
        );
        let requested = reserve_materialization(3, Some(3), 9, &mut observed).unwrap_err();
        assert_eq!(requested.capacity_bytes, 7);
        assert_eq!(
            requested.kind,
            NumericMaterializationError::Boundary(
                IntegerUnitStepError::RequestedCapacityLimitExceeded {
                    limit: 9,
                    required: 10
                }
            )
        );
        let digit = reserve_materialization(3, Some(2), 100, &mut observed).unwrap_err();
        assert_eq!(digit.capacity_bytes, 7);
        assert_eq!(
            digit.kind,
            NumericMaterializationError::Boundary(IntegerUnitStepError::DigitLimitExceeded {
                limit: 2,
                required: 3
            })
        );
    }

    #[test]
    fn materialized_work_envelopes_are_checked_before_unrepresentable_operations() {
        assert_eq!(
            HtmlNonNegativeInteger::from_borrowed_digits_work_bound(usize::MAX),
            None
        );
        assert_eq!(
            HtmlNonNegativeDecimal::from_borrowed_digits_work_bound(1, usize::MAX),
            None
        );
        assert!(HtmlNonNegativeInteger::from_u32_work_bound().unwrap() >= 10);
        let integer = HtmlNonNegativeInteger::from(999);
        assert!(integer.decremented_work_bound().unwrap() >= 6);
        assert!(integer.quotient_remainder_work_bound().unwrap() >= 12);
        assert!(integer.multiplied_by_work_bound().unwrap() >= 12);
        assert!(integer.added_to_work_bound(&integer).unwrap() >= 24);
        let decimal = decimal("9", "9999");
        assert!(decimal.copy_work_bound().unwrap() >= 10);
        assert!(decimal.multiplied_by_work_bound().unwrap() >= 20);
        assert!(decimal.added_to_work_bound(&decimal).unwrap() >= 24);
    }

    #[test]
    fn scientific_keys_normalize_decimal_places_and_exponent_signs_exactly() {
        // §2.3.4.3 defines the mathematical significand times ten to the
        // exponent, independently of the later binary64 conversion.
        for (input, negative, significand, exponent) in [
            ("1", false, "1", "0"),
            ("1.0", false, "1", "0"),
            ("10e-1", false, "1", "0"),
            (".0100", false, "1", "-2"),
            ("00012.0300E+00002", false, "1203", "0"),
            ("-1200e-0001", true, "12", "1"),
            (".001e2", false, "1", "-1"),
            ("1000e-2", false, "1", "1"),
            ("0.1e1", false, "1", "0"),
            ("10e-0", false, "1", "1"),
        ] {
            let key = strict_scientific_decimal_key(&text(input)).unwrap();
            assert_eq!(key.negative, negative);
            assert_eq!(key.significand.decimal_digits(), significand);
            assert_eq!(key.exponent.as_document_string(), text(exponent));
            assert!(!key.is_zero());
            assert_eq!(key.is_positive(), !negative);
        }
    }

    #[test]
    fn scientific_zero_discards_sign_and_exponent_only_after_full_validation() {
        let zero = strict_scientific_decimal_key(&text("0")).unwrap();
        for input in ["-0", "-000.000E-999999999999999999999999999999", "0e+0"] {
            let key = strict_scientific_decimal_key(&text(input)).unwrap();
            assert_eq!(key, zero);
            assert!(key.is_zero());
            assert!(!key.is_positive());
            assert!(!key.negative);
            assert_eq!(key.exponent.as_document_string(), text("0"));
        }
        for input in ["0e", "-0e+", "0e999garbage", "0.", "+0", "0 "] {
            assert_eq!(
                strict_scientific_decimal_key(&text(input)),
                Err(ScientificDecimalKeyError::Syntax(
                    NumericParseError::InvalidSyntax
                ))
            );
        }
    }

    #[test]
    fn scientific_equality_is_mathematical_instead_of_binary64_equality() {
        let one = strict_scientific_decimal_key(&text("1")).unwrap();
        for input in ["1.0", "10e-1", "0001000E-0003", ".01e2"] {
            assert_eq!(strict_scientific_decimal_key(&text(input)).unwrap(), one);
        }
        let distinct = strict_scientific_decimal_key(&text("1.00000000000000001")).unwrap();
        assert_ne!(distinct, one);
        assert_eq!(
            validate_floating_point_number(&text("1.00000000000000001")),
            validate_floating_point_number(&text("1"))
        );
        for input in ["1e-999", "-1e-999"] {
            let key = strict_scientific_decimal_key(&text(input)).unwrap();
            assert!(!key.is_zero());
            assert_eq!(key.is_positive(), input == "1e-999");
            assert_eq!(
                validate_floating_point_number(&text(input))
                    .unwrap()
                    .value(),
                0.0
            );
        }
        let overflow = strict_scientific_decimal_key(&text("1e999")).unwrap();
        assert!(overflow.is_positive());
        assert_eq!(
            validate_floating_point_number(&text("1e999")),
            Err(NumericParseError::FloatingPointOverflow)
        );
    }

    #[test]
    fn scientific_exponents_carry_borrow_and_remain_unbounded_without_expansion() {
        for (input, exponent) in [
            (
                "10e9999999999999999999999999999999999999999",
                "10000000000000000000000000000000000000000",
            ),
            (
                ".1e-9999999999999999999999999999999999999999",
                "-10000000000000000000000000000000000000000",
            ),
            (
                ".1e10000000000000000000000000000000000000000",
                "9999999999999999999999999999999999999999",
            ),
            (
                "10e-10000000000000000000000000000000000000000",
                "-9999999999999999999999999999999999999999",
            ),
        ] {
            let key = strict_scientific_decimal_key(&text(input)).unwrap();
            assert_eq!(key.significand.decimal_digits(), "1");
            assert_eq!(key.exponent.as_document_string(), text(exponent));
            assert!(key.is_positive());
        }
    }

    #[test]
    fn scientific_strict_rejection_and_float_recovery_share_the_same_scanner() {
        for (input, recovered) in [
            ("+1", 1.0),
            ("1.", 1.0),
            ("1e+", 1.0),
            ("1E-", 1.0),
            (" 1", 1.0),
            ("1tail", 1.0),
        ] {
            assert_eq!(
                strict_scientific_decimal_key(&text(input)),
                Err(ScientificDecimalKeyError::Syntax(
                    NumericParseError::InvalidSyntax
                ))
            );
            assert_eq!(
                validate_floating_point_number(&text(input)),
                Err(NumericParseError::InvalidSyntax)
            );
            assert_eq!(
                parse_floating_point_number(&text(input)).unwrap().value(),
                recovered
            );
            let bounded = try_floating_point_number(
                &text(input),
                FloatingPointParseMode::Recovery,
                input.len(),
            )
            .unwrap();
            assert_eq!(bounded.value.unwrap().value(), recovered);
        }
        for input in ["", ".", "NaN", "Infinity", "1e--1", "1\u{a0}"] {
            assert_eq!(
                strict_scientific_decimal_key(&text(input)),
                Err(ScientificDecimalKeyError::Syntax(
                    NumericParseError::InvalidSyntax
                ))
            );
        }
        for units in [vec![0xd800, 49], vec![49, 0xdc00], vec![49, 101, 0xd800]] {
            let input = DocumentString::from_utf16(units);
            assert_eq!(
                strict_scientific_decimal_key(&input),
                Err(ScientificDecimalKeyError::Syntax(
                    NumericParseError::InvalidSyntax
                ))
            );
        }
    }

    #[test]
    fn scientific_adjustment_checks_the_real_integer_sum_representation_bound() {
        let limit = isize::MAX as usize / size_of::<char>();
        assert_eq!(integer_sum_capacity(limit - 1, 1), Some(limit));
        assert_eq!(integer_sum_capacity(limit, 1), None);
        assert_eq!(integer_sum_capacity(usize::MAX, 1), None);
        for (exponent, fraction, trailing, expected) in [
            ("-2", 0, 3, "1"),
            ("2", 3, 0, "-1"),
            ("-2", 0, 2, "0"),
            ("2", 2, 0, "0"),
            ("0", 0, 0, "0"),
        ] {
            let adjusted =
                adjust_scientific_exponent(exact_signed(exponent), fraction, trailing).unwrap();
            assert_eq!(adjusted.as_document_string(), text(expected));
        }
    }

    #[test]
    fn signed_comparison_preserves_sign_fraction_and_unbounded_magnitude() {
        for (left, right, expected) in [
            (
                signed_decimal(0, "0", true),
                signed_decimal(0, "000", false),
                Ordering::Equal,
            ),
            (
                signed_decimal(0, "001", true),
                signed_decimal(0, "0", false),
                Ordering::Less,
            ),
            (
                signed_decimal(1, "0", false),
                signed_decimal(1, "0", true),
                Ordering::Greater,
            ),
            (
                signed_decimal(2, "1", true),
                signed_decimal(2, "09", true),
                Ordering::Less,
            ),
            (
                signed_decimal(2, "100", false),
                signed_decimal(2, "1", false),
                Ordering::Equal,
            ),
            (
                signed_decimal(0, "000000001", false),
                signed_decimal(0, "0000000001", false),
                Ordering::Greater,
            ),
        ] {
            assert_eq!(left.compare(&right), expected);
            assert_eq!(right.compare(&left), expected.reverse());
        }
        let large = HtmlSignedDecimal::from(HtmlInteger::from_magnitude(
            HtmlNonNegativeInteger::from_decimal_digits(
                &"340282366920938463463374607431768211456".into(),
            )
            .unwrap(),
            false,
        ));
        let smaller = HtmlSignedDecimal::from(HtmlInteger::from_magnitude(u128::MAX.into(), false));
        assert_eq!(large.compare(&smaller), Ordering::Greater);
        let negative = HtmlSignedDecimal::from_magnitude(large.magnitude().clone(), true);
        let less_negative = HtmlSignedDecimal::from_magnitude(smaller.magnitude().clone(), true);
        assert_eq!(negative.compare(&less_negative), Ordering::Less);
    }

    fn signed_decimal(integer: u128, fraction: &str, negative: bool) -> HtmlSignedDecimal {
        HtmlSignedDecimal::from_magnitude(
            HtmlNonNegativeDecimal::with_fraction_digits(integer.into(), &fraction.into()).unwrap(),
            negative,
        )
    }

    #[test]
    fn signed_decimal_addition_handles_borrow_sign_and_cancellation_exactly() {
        for (left, right, expected) in [
            (
                signed_decimal(1000, "0", true),
                signed_decimal(999, "999999", false),
                "-0.000001",
            ),
            (
                signed_decimal(999, "999999", false),
                signed_decimal(1000, "0", true),
                "-0.000001",
            ),
            (
                signed_decimal(0, "001", false),
                signed_decimal(0, "0001", true),
                "0.0009",
            ),
            (
                signed_decimal(0, "001", true),
                signed_decimal(0, "0001", false),
                "-0.0009",
            ),
            (
                signed_decimal(0, "99", false),
                signed_decimal(0, "01", false),
                "1",
            ),
            (
                signed_decimal(0, "99", true),
                signed_decimal(0, "01", true),
                "-1",
            ),
            (
                signed_decimal(123, "456", false),
                signed_decimal(123, "456", true),
                "0",
            ),
        ] {
            assert_eq!(
                left.added_to(&right).as_document_string(),
                DocumentString::from(expected)
            );
        }
    }

    #[test]
    fn exact_magnitude_order_uses_fractional_zero_padding_and_integer_width() {
        assert_eq!(
            signed_decimal(1, "1", false)
                .magnitude
                .compare(&signed_decimal(1, "099999", false).magnitude),
            Ordering::Greater
        );
        assert_eq!(
            signed_decimal(9, "999", false)
                .magnitude
                .compare(&signed_decimal(10, "0", false).magnitude),
            Ordering::Less
        );
        assert_eq!(
            signed_decimal(1, "1000", false)
                .magnitude
                .compare(&signed_decimal(1, "1", false).magnitude),
            Ordering::Equal
        );
    }

    #[test]
    fn exact_integer_difference_never_projects_an_arbitrary_magnitude() {
        let huge = HtmlNonNegativeInteger::from_decimal_digits(
            &"10000000000000000000000000000000000000000".into(),
        )
        .unwrap();
        assert_eq!(
            HtmlSignedDecimal::integer_difference(huge.clone(), 1_u128.into()).as_document_string(),
            DocumentString::from("9999999999999999999999999999999999999999")
        );
        assert_eq!(
            HtmlSignedDecimal::integer_difference(1_u128.into(), huge).as_document_string(),
            DocumentString::from("-9999999999999999999999999999999999999999")
        );
    }

    #[test]
    fn finite_binary64_representation_preserves_sign_zero_and_exponent_branches() {
        for (value, expected) in [
            (0.0, "0"),
            (-0.0, "0"),
            (-2.125, "-2.125"),
            (0.5, "0.5"),
            (9007199254740992.0, "9007199254740992"),
        ] {
            let number = HtmlFloatingPointNumber::try_from(value).unwrap();
            assert_eq!(
                number.exact_decimal().as_document_string(),
                DocumentString::from(expected)
            );
        }
    }

    fn text(input: &str) -> DocumentString {
        input.into()
    }

    fn exact_signed(input: &str) -> HtmlInteger {
        let negative = input.starts_with('-');
        let magnitude = HtmlNonNegativeInteger::from_decimal_digits(&text(
            input.strip_prefix('-').unwrap_or(input),
        ))
        .unwrap();
        HtmlInteger::from_magnitude(magnitude, negative)
    }

    #[test]
    fn fallible_unit_steps_cross_zero_carry_borrow_and_unbounded_magnitudes() {
        // Independent exact decimal expectations for ordinal-value step 7,
        // HTML 877d6146487f `ordinal-value`; no machine projection is involved.
        for (input, incremented, decremented) in [
            ("0", "1", "-1"),
            ("1", "2", "0"),
            ("-1", "0", "-2"),
            ("9", "10", "8"),
            ("-9", "-8", "-10"),
            ("10", "11", "9"),
            ("-10", "-9", "-11"),
            ("99", "100", "98"),
            ("1000", "1001", "999"),
            ("-1000", "-999", "-1001"),
            (
                "340282366920938463463374607431768211456",
                "340282366920938463463374607431768211457",
                "340282366920938463463374607431768211455",
            ),
            (
                "-340282366920938463463374607431768211456",
                "-340282366920938463463374607431768211455",
                "-340282366920938463463374607431768211457",
            ),
        ] {
            let value = exact_signed(input);
            for (step, expected) in [
                (IntegerUnitStep::Increment, incremented),
                (IntegerUnitStep::Decrement, decremented),
            ] {
                let stepped = value.try_unit_step(step, 100, 100).unwrap();
                assert_eq!(stepped.value.as_document_string(), text(expected));
                assert!(stepped.capacity_bytes >= stepped.value.magnitude().decimal_digits().len());
                assert!(stepped.capacity_bytes <= 100);
                assert_eq!(value.as_document_string(), text(input));
            }
        }
        let nines = exact_signed("9999999999999999999999999999999999999999");
        assert_eq!(
            nines
                .try_unit_step(IntegerUnitStep::Increment, 41, 41)
                .unwrap()
                .value
                .as_document_string(),
            text("10000000000000000000000000000000000000000")
        );
        let minus_one = exact_signed("-1")
            .try_unit_step(IntegerUnitStep::Increment, 1, 1)
            .unwrap();
        assert!(!minus_one.value.is_negative());
        assert!(minus_one.value.magnitude().is_zero());
    }

    #[test]
    fn fallible_unit_step_checks_exact_result_digits_and_capacity_before_writing() {
        let value = exact_signed("999");
        assert_eq!(
            value
                .try_unit_step(IntegerUnitStep::Increment, 3, 100)
                .unwrap_err(),
            IntegerUnitStepError::DigitLimitExceeded {
                limit: 3,
                required: 4,
            }
        );
        assert_eq!(
            value
                .try_unit_step(IntegerUnitStep::Increment, 4, 3)
                .unwrap_err(),
            IntegerUnitStepError::RequestedCapacityLimitExceeded {
                limit: 3,
                required: 4,
            }
        );
        assert_eq!(
            value
                .try_unit_step(IntegerUnitStep::Increment, 4, 4)
                .unwrap()
                .value
                .as_document_string(),
            text("1000")
        );
        assert_eq!(
            exact_signed("1000")
                .try_unit_step(IntegerUnitStep::Decrement, 3, 3)
                .unwrap()
                .value
                .as_document_string(),
            text("999")
        );
        assert_eq!(
            exact_signed("1")
                .try_unit_step(IntegerUnitStep::Decrement, 0, 1)
                .unwrap_err(),
            IntegerUnitStepError::DigitLimitExceeded {
                limit: 0,
                required: 1,
            }
        );
        assert_eq!(value.as_document_string(), text("999"));
    }

    #[test]
    fn native_reversed_list_counts_use_the_same_fallible_allocation() {
        for (count, expected) in [(0, "0"), (1, "1"), (9, "9"), (10, "10"), (1000, "1000")] {
            let result = HtmlInteger::try_from_usize(count, 100, 100).unwrap();
            assert_eq!(result.value.as_document_string(), text(expected));
            assert!(!result.value.is_negative());
            assert!(result.capacity_bytes >= expected.len());
        }
        let expected = match usize::BITS {
            16 => "65535",
            32 => "4294967295",
            64 => "18446744073709551615",
            _ => panic!("test expectation requires an independently specified pointer width"),
        };
        assert_eq!(
            HtmlInteger::try_from_usize(usize::MAX, expected.len(), expected.len())
                .unwrap()
                .value
                .as_document_string(),
            text(expected)
        );
        assert_eq!(
            HtmlInteger::try_from_usize(10, 1, 2).unwrap_err(),
            IntegerUnitStepError::DigitLimitExceeded {
                limit: 1,
                required: 2,
            }
        );
        assert_eq!(
            HtmlInteger::try_from_usize(0, 1, 0).unwrap_err(),
            IntegerUnitStepError::RequestedCapacityLimitExceeded {
                limit: 0,
                required: 1,
            }
        );
    }

    #[test]
    fn integer_reserve_boundaries_reject_representation_and_observed_overshoot() {
        // Exercise the real private post-reserve boundary with independently
        // supplied observed capacities, without replacing the allocator/provider.
        assert_eq!(check_integer_actual_capacity(3, 3, 3), Ok(()));
        assert_eq!(check_integer_actual_capacity(3, 4, 4), Ok(()));
        assert_eq!(
            check_integer_actual_capacity(3, 4, 3),
            Err(IntegerUnitStepError::ActualCapacityLimitExceeded {
                limit: 3,
                requested: 3,
                capacity_bytes: 4,
            })
        );
        assert_eq!(
            check_integer_digit_capacity(isize::MAX as usize + 1, usize::MAX, usize::MAX),
            Err(IntegerUnitStepError::RepresentationExhausted)
        );
        assert_eq!(
            check_integer_digit_capacity(isize::MAX as usize, usize::MAX, usize::MAX),
            Ok(())
        );
    }

    #[test]
    fn fallible_recovery_preserves_signed_prefixes_and_exact_utf16_boundaries() {
        // HTML 877d §2.3.4.1 permits ASCII whitespace/plus and ignores the
        // suffix after the first digit run; no author-conformance claim follows.
        for (input, expected) in [
            ("0", "0"),
            ("\t\n\u{c}\r +00012suffix", "12"),
            ("-000", "0"),
            (" -0010.2", "-10"),
            (
                "340282366920938463463374607431768211456tail",
                "340282366920938463463374607431768211456",
            ),
            (
                "-9999999999999999999999999999999999999999!",
                "-9999999999999999999999999999999999999999",
            ),
        ] {
            let raw = text(input);
            let recovered = try_parse_integer(&raw, 100, 100).unwrap();
            assert_eq!(recovered.value.as_document_string(), text(expected));
            assert_eq!(
                recovered.capacity_bytes,
                recovered.value.magnitude.digits.capacity()
            );
            assert!(recovered.capacity_bytes >= recovered.value.magnitude.decimal_digits().len());
            assert_eq!(recovered.value, parse_integer(&raw).unwrap());
        }
        let raw = DocumentString::from_utf16(vec![b'1' as u16, 0xd800, b'2' as u16]);
        assert_eq!(try_parse_integer(&raw, 1, 1).unwrap().value.to_i64(), Ok(1));
        assert_eq!(raw.code_units(), &[b'1' as u16, 0xd800, b'2' as u16]);
    }

    #[test]
    fn fallible_recovery_distinguishes_rejection_from_canonical_allocation_limits() {
        for input in ["", " \t", "+", "-", "+ 1", "--1", "\u{a0}1", "١", "x1"] {
            assert_eq!(
                try_parse_integer(&text(input), 0, 0).unwrap_err(),
                IntegerRecoveryError::Syntax(NumericParseError::InvalidSyntax)
            );
        }
        let raw = text(" -000123suffix");
        assert_eq!(
            try_parse_integer(&raw, 3, 3).unwrap().value.to_i64(),
            Ok(-123)
        );
        assert_eq!(
            try_parse_integer(&raw, 2, 3).unwrap_err(),
            IntegerRecoveryError::Allocation(IntegerUnitStepError::DigitLimitExceeded {
                limit: 2,
                required: 3,
            })
        );
        assert_eq!(
            try_parse_integer(&raw, 3, 2).unwrap_err(),
            IntegerRecoveryError::Allocation(
                IntegerUnitStepError::RequestedCapacityLimitExceeded {
                    limit: 2,
                    required: 3,
                }
            )
        );
        assert_eq!(
            try_parse_integer(&text("-00000"), 1, 1)
                .unwrap()
                .value
                .to_i64(),
            Ok(0)
        );
        assert_eq!(
            try_parse_integer(&text("-00000"), 0, 1).unwrap_err(),
            IntegerRecoveryError::Allocation(IntegerUnitStepError::DigitLimitExceeded {
                limit: 0,
                required: 1,
            })
        );
    }

    #[test]
    fn capped_nonnegative_recovery_clamps_exact_values_without_machine_overflow() {
        // Source-specific span defaults remain with the table owner. The numeric
        // contract is just §2.3.4.2 recovery followed by a bounded projection.
        for (input, cap, expected) in [
            ("0", 1000, 0),
            ("0001", 1000, 1),
            ("  +0999tail", 1000, 999),
            ("1000", 1000, 1000),
            ("1001", 1000, 1000),
            ("65535", 65534, 65534),
            ("-000tail", 0, 0),
            ("42", 0, 0),
            (
                "99999999999999999999999999999999999999999999999999",
                65534,
                65534,
            ),
            (
                "340282366920938463463374607431768211456",
                usize::MAX,
                usize::MAX,
            ),
            ("-000", usize::MAX, 0),
        ] {
            assert_eq!(
                parse_nonnegative_integer_capped(&text(input), cap),
                Ok(expected)
            );
        }
        let maximum = match usize::BITS {
            16 => "65535",
            32 => "4294967295",
            64 => "18446744073709551615",
            _ => panic!("test expectation requires an independently specified pointer width"),
        };
        assert_eq!(
            parse_nonnegative_integer_capped(&text(maximum), usize::MAX),
            Ok(usize::MAX)
        );
    }

    #[test]
    fn capped_nonnegative_recovery_rejects_negative_nonzero_before_zero_projection() {
        for input in [
            "-1",
            "-0001tail",
            "-99999999999999999999999999999999999999999",
        ] {
            for cap in [0, 1, 1000, usize::MAX] {
                assert_eq!(
                    parse_nonnegative_integer_capped(&text(input), cap),
                    Err(NumericParseError::Negative)
                );
            }
        }
        for input in ["", "+", " -", "- 0", "x1", "\u{a0}1"] {
            assert_eq!(
                parse_nonnegative_integer_capped(&text(input), 0),
                Err(NumericParseError::InvalidSyntax)
            );
        }
        let leading_surrogate = DocumentString::from_utf16(vec![0xd800, b'1' as u16]);
        assert_eq!(
            parse_nonnegative_integer_capped(&leading_surrogate, 1000),
            Err(NumericParseError::InvalidSyntax)
        );
        let suffix_surrogate = DocumentString::from_utf16(vec![b'7' as u16, 0xdc00]);
        assert_eq!(
            parse_nonnegative_integer_capped(&suffix_surrogate, 1000),
            Ok(7)
        );
    }

    #[test]
    fn integer_author_syntax_rejects_recovery_only_spellings() {
        for input in ["", "+1", " 1", "1 ", "1suffix", "--1", "١"] {
            assert_eq!(
                validate_integer(&text(input)),
                Err(NumericParseError::InvalidSyntax)
            );
        }
        assert_eq!(validate_integer(&text("-0012")).unwrap().to_i64(), Ok(-12));
        assert_eq!(
            validate_integer(&text("-000")).unwrap(),
            HtmlInteger::from(0)
        );
    }

    #[test]
    fn integer_recovery_preserves_unbounded_magnitude_and_consumes_a_prefix() {
        let input = text("\t+340282366920938463463374607431768211456tail");
        let integer = parse_integer(&input).unwrap();
        assert_eq!(
            integer.as_document_string(),
            text("340282366920938463463374607431768211456")
        );
        assert_eq!(integer.to_i128(), Err(HtmlNumericRangeError));
        assert_eq!(integer.magnitude().to_u128(), Err(HtmlNumericRangeError));
        assert_eq!(
            parse_integer(&text("\u{a0}1")),
            Err(NumericParseError::InvalidSyntax)
        );
        assert_eq!(
            parse_integer(&DocumentString::from_utf16(vec![49, 0xd800]))
                .unwrap()
                .to_i64(),
            Ok(1)
        );
        assert_eq!(
            validate_integer(&DocumentString::from_utf16(vec![49, 0xd800])),
            Err(NumericParseError::InvalidSyntax)
        );
    }

    #[test]
    fn native_integer_construction_and_bounded_projection_keep_extreme_values() {
        for value in [i128::MIN, -1, 0, 1, i128::MAX] {
            assert_eq!(HtmlInteger::from(value).to_i128(), Ok(value));
        }
        let unsigned = HtmlNonNegativeInteger::from(u128::MAX);
        assert_eq!(unsigned.to_u128(), Ok(u128::MAX));
        assert_eq!(
            HtmlInteger::from_magnitude(unsigned, false).to_i128(),
            Err(HtmlNumericRangeError)
        );
        assert_eq!(HtmlInteger::from(i64::MIN as i128).to_i64(), Ok(i64::MIN));
        assert_eq!(
            HtmlInteger::from(i64::MAX as i128 + 1).to_i64(),
            Err(HtmlNumericRangeError)
        );
    }

    #[test]
    fn exact_decimal_constructor_and_arithmetic_preserve_unbounded_values() {
        let magnitude = HtmlNonNegativeInteger::from_decimal_digits(&text(
            "000340282366920938463463374607431768211456",
        ))
        .unwrap();
        assert_eq!(
            magnitude.decimal_digits(),
            "340282366920938463463374607431768211456"
        );
        assert_eq!(magnitude.to_u128(), Err(HtmlNumericRangeError));
        assert_eq!(magnitude.remainder(NonZeroU32::new(400).unwrap()), 256);
        assert_eq!(
            magnitude.multiplied_by(2).decimal_digits(),
            "680564733841876926926749214863536422912"
        );
        assert_eq!(magnitude.added_to(&magnitude), magnitude.multiplied_by(2));
        assert_eq!(
            HtmlNonNegativeInteger::from(999)
                .added_to(&HtmlNonNegativeInteger::from(1))
                .decimal_digits(),
            "1000"
        );
        assert_eq!(magnitude.multiplied_by(0).decimal_digits(), "0");
        assert_eq!(
            HtmlNonNegativeInteger::from(999)
                .multiplied_by(u32::MAX)
                .decimal_digits(),
            "4290672327705"
        );
        assert_eq!(
            HtmlNonNegativeInteger::from(0)
                .added_to(&HtmlNonNegativeInteger::from(0))
                .decimal_digits(),
            "0"
        );
        assert_eq!(
            HtmlNonNegativeInteger::from_decimal_digits(&text("")),
            Err(HtmlDecimalDigitsError::Empty)
        );
        assert_eq!(
            HtmlNonNegativeInteger::from_decimal_digits(&text("+1")),
            Err(HtmlDecimalDigitsError::NonAsciiDigit {
                code_unit_offset: 0,
                code_unit: 43
            })
        );
        assert_eq!(
            HtmlNonNegativeInteger::from_decimal_digits(&DocumentString::from_utf16(vec![
                49, 0xd800
            ])),
            Err(HtmlDecimalDigitsError::NonAsciiDigit {
                code_unit_offset: 1,
                code_unit: 0xd800
            })
        );
        let signed = HtmlInteger::from_magnitude(magnitude, true);
        assert!(signed.is_negative());
        assert_eq!(
            signed.as_document_string(),
            text("-340282366920938463463374607431768211456")
        );
    }

    #[test]
    fn borrowed_nonnegative_prefix_preserves_zero_sign_and_digit_suffix_boundaries() {
        for (raw, expected) in [
            (" +12tail", Ok("12")),
            (" -000tail", Ok("000")),
            ("-001", Err(NumericParseError::Negative)),
            ("+", Err(NumericParseError::InvalidSyntax)),
            (
                "340282366920938463463374607431768211456z",
                Ok("340282366920938463463374607431768211456"),
            ),
        ] {
            let input = text(raw);
            let mut scanner = IntegerPrefixScanner::new(true);
            for &unit in input.code_units() {
                if !scanner.feed(unit) {
                    break;
                }
            }
            let actual = scanner
                .finish()
                .and_then(|prefix| prefix.nonnegative_digits(input.code_units()));
            assert_eq!(
                actual,
                expected
                    .as_ref()
                    .map(|s| s.encode_utf16().collect::<Vec<_>>())
                    .as_deref()
                    .map_err(|e| **e)
            );
        }
    }
    #[test]
    fn nonnegative_recovery_accepts_negative_zero_but_author_grammar_does_not() {
        assert!(
            parse_nonnegative_integer(&text(" -000tail"))
                .unwrap()
                .is_zero()
        );
        assert_eq!(
            validate_nonnegative_integer(&text("-0")),
            Err(NumericParseError::InvalidSyntax)
        );
        assert_eq!(
            parse_nonnegative_integer(&text("-1")),
            Err(NumericParseError::Negative)
        );
        assert_eq!(
            validate_nonnegative_integer(&text("00042"))
                .unwrap()
                .to_u64(),
            Ok(42)
        );
        assert_eq!(
            HtmlNonNegativeInteger::try_from(HtmlInteger::from(-1)),
            Err(HtmlNegativeIntegerError)
        );
    }

    #[test]
    fn controlled_float_author_and_recovery_preserve_distinct_prefix_rules() {
        for (input, expected, token_bytes) in [
            (" +1tail", 1.0, 2),
            ("1.", 1.0, 2),
            ("1.e2", 100.0, 4),
            ("1e", 1.0, 1),
            ("1e+", 1.0, 1),
            ("1e-x", 1.0, 1),
            ("1.2suffix", 1.2, 3),
        ] {
            let raw = text(input);
            let author =
                try_floating_point_number(&raw, FloatingPointParseMode::Author, 0).unwrap();
            assert_eq!(author.value, Err(NumericParseError::InvalidSyntax));
            assert_eq!(author.capacity_bytes, 0);
            let recovery =
                try_floating_point_number(&raw, FloatingPointParseMode::Recovery, token_bytes)
                    .unwrap();
            assert_eq!(recovery.value.unwrap().value(), expected);
            assert_eq!(recovery.capacity_bytes, token_bytes);
        }
        for (input, expected) in [(".5", 0.5), ("-.5", -0.5), ("1e+2", 100.0), ("1E-2", 0.01)] {
            for mode in [
                FloatingPointParseMode::Author,
                FloatingPointParseMode::Recovery,
            ] {
                let parsed = try_floating_point_number(&text(input), mode, 100).unwrap();
                assert_eq!(parsed.value.unwrap().value(), expected);
            }
        }
    }

    #[test]
    fn controlled_float_conversion_rounds_once_including_subnormal_and_zero_values() {
        // HTML §2.3.4.3 conversion chooses nearest binary64 with ties to even,
        // rejects infinity, and excludes negative zero from its result set.
        for (input, expected_bits) in [
            (
                "1.00000000000000011102230246251565404236316680908203125",
                0x3ff0000000000000,
            ),
            (
                "1.00000000000000033306690738754696212708950042724609375",
                0x3ff0000000000002,
            ),
            ("0.1", 0x3fb999999999999a),
            ("5e-324", 1),
            ("-5e-324", 0x8000000000000001),
            ("2e-324", 0),
            ("-2e-324", 0),
            ("-0", 0),
            ("1.7976931348623158e308", 0x7fefffffffffffff),
        ] {
            for mode in [
                FloatingPointParseMode::Author,
                FloatingPointParseMode::Recovery,
            ] {
                let parsed = try_floating_point_number(&text(input), mode, 100).unwrap();
                assert_eq!(parsed.value.unwrap().value().to_bits(), expected_bits);
                assert!(parsed.capacity_bytes >= input.len());
            }
        }
        // Decimal identity is independently 10^400 * 10^-400 = 1. Retain all
        // significand digits; no intermediate f64 accumulation is permitted.
        let input = text(&format!("1{}e-400", "0".repeat(400)));
        let parsed =
            try_floating_point_number(&input, FloatingPointParseMode::Recovery, input.len())
                .unwrap();
        assert_eq!(parsed.value.unwrap().value(), 1.0);
        assert_eq!(parsed.capacity_bytes, input.len());
    }

    #[test]
    fn controlled_float_overflow_and_huge_exponents_retain_temporary_capacity() {
        for input in [
            "1e9999",
            "-1e9999",
            "1.7976931348623159e308",
            "1e999999999999999999999999999999999999999999999999999999999999",
        ] {
            let raw = text(input);
            let parsed =
                try_floating_point_number(&raw, FloatingPointParseMode::Recovery, raw.len())
                    .unwrap();
            assert_eq!(parsed.value, Err(NumericParseError::FloatingPointOverflow));
            assert_eq!(parsed.capacity_bytes, raw.len());
        }
        for input in [
            "0e999999999999999999999999999999999999999999999999999999999999",
            "-1e-999999999999999999999999999999999999999999999999999999999999",
        ] {
            let raw = text(input);
            let parsed =
                try_floating_point_number(&raw, FloatingPointParseMode::Recovery, raw.len())
                    .unwrap();
            assert_eq!(parsed.value.unwrap().value().to_bits(), 0);
            assert_eq!(parsed.capacity_bytes, raw.len());
        }
        let raw = text("1e999tail");
        let author = try_floating_point_number(&raw, FloatingPointParseMode::Author, 0).unwrap();
        assert_eq!(author.value, Err(NumericParseError::InvalidSyntax));
        assert_eq!(author.capacity_bytes, 0);
        let recovered =
            try_floating_point_number(&raw, FloatingPointParseMode::Recovery, 5).unwrap();
        assert_eq!(
            recovered.value,
            Err(NumericParseError::FloatingPointOverflow)
        );
        assert_eq!(recovered.capacity_bytes, 5);
    }

    #[test]
    fn controlled_float_rejection_and_utf16_suffixes_allocate_only_the_selected_token() {
        for input in ["", " \t", ".", "-.", "+", "NaN", "Infinity", "١", "\u{a0}1"] {
            for mode in [
                FloatingPointParseMode::Author,
                FloatingPointParseMode::Recovery,
            ] {
                let parsed = try_floating_point_number(&text(input), mode, 0).unwrap();
                assert_eq!(parsed.value, Err(NumericParseError::InvalidSyntax));
                assert_eq!(parsed.capacity_bytes, 0);
            }
        }
        let suffix = DocumentString::from_utf16(vec![b'1' as u16, 0xd800, b'2' as u16]);
        let parsed =
            try_floating_point_number(&suffix, FloatingPointParseMode::Recovery, 1).unwrap();
        assert_eq!(parsed.value.unwrap().value(), 1.0);
        assert_eq!(parsed.capacity_bytes, 1);
        assert_eq!(suffix.code_units(), &[b'1' as u16, 0xd800, b'2' as u16]);
        let author = try_floating_point_number(&suffix, FloatingPointParseMode::Author, 0).unwrap();
        assert_eq!(author.value, Err(NumericParseError::InvalidSyntax));
        assert_eq!(author.capacity_bytes, 0);
        let leading = DocumentString::from_utf16(vec![0xdc00, b'1' as u16]);
        assert_eq!(
            try_floating_point_number(&leading, FloatingPointParseMode::Recovery, 0)
                .unwrap()
                .value,
            Err(NumericParseError::InvalidSyntax)
        );
    }

    #[test]
    fn controlled_float_reservation_obeys_requested_observed_and_representation_limits() {
        let input = text("  -1.5tail");
        assert_eq!(
            try_floating_point_number(&input, FloatingPointParseMode::Recovery, 3).unwrap_err(),
            FloatingPointAllocationError::RequestedCapacityLimitExceeded {
                limit: 3,
                required: 4
            }
        );
        let parsed =
            try_floating_point_number(&input, FloatingPointParseMode::Recovery, 4).unwrap();
        assert_eq!(parsed.value.unwrap().value(), -1.5);
        assert_eq!(parsed.capacity_bytes, 4);
        // Exercise the real observed-capacity boundary with independent literal
        // observations, without a fake allocator or forced out-of-memory claim.
        assert_eq!(check_floating_point_actual_capacity(3, 3, 3), Ok(()));
        assert_eq!(check_floating_point_actual_capacity(3, 4, 4), Ok(()));
        assert_eq!(
            check_floating_point_actual_capacity(3, 4, 3),
            Err(FloatingPointAllocationError::ActualCapacityLimitExceeded {
                limit: 3,
                requested: 3,
                capacity_bytes: 4
            })
        );
        assert_eq!(
            check_floating_point_requested_capacity(isize::MAX as usize + 1, usize::MAX),
            Err(FloatingPointAllocationError::RepresentationExhausted)
        );
        assert_eq!(
            check_floating_point_requested_capacity(isize::MAX as usize, usize::MAX),
            Ok(())
        );
    }

    #[test]
    fn floating_author_grammar_and_recovery_observe_distinct_rules() {
        for (input, expected) in [(".5", 0.5), ("-.5", -0.5), ("1e+2", 100.0), ("1E-2", 0.01)] {
            assert_eq!(
                validate_floating_point_number(&text(input))
                    .unwrap()
                    .value(),
                expected
            );
        }
        for (input, expected) in [
            (" +1tail", 1.0),
            ("1.", 1.0),
            ("1.e2", 100.0),
            ("1e", 1.0),
            ("1e+", 1.0),
            ("1e-x", 1.0),
            ("1.2suffix", 1.2),
        ] {
            assert_eq!(
                validate_floating_point_number(&text(input)),
                Err(NumericParseError::InvalidSyntax)
            );
            assert_eq!(
                parse_floating_point_number(&text(input)).unwrap().value(),
                expected
            );
        }
        for input in ["", ".", "-.", "NaN", "Infinity", "\u{a0}1"] {
            assert_eq!(
                parse_floating_point_number(&text(input)),
                Err(NumericParseError::InvalidSyntax)
            );
        }
    }

    #[test]
    fn floating_conversion_rounds_once_and_distinguishes_overflow() {
        let halfway_even = text("1.00000000000000011102230246251565404236316680908203125");
        assert_eq!(
            parse_floating_point_number(&halfway_even).unwrap().value(),
            1.0
        );
        let halfway_odd = text("1.00000000000000033306690738754696212708950042724609375");
        assert_eq!(
            parse_floating_point_number(&halfway_odd)
                .unwrap()
                .value()
                .to_bits(),
            1.0f64.to_bits() + 2
        );
        assert_eq!(
            parse_floating_point_number(&text("1.7976931348623158e308"))
                .unwrap()
                .value(),
            f64::MAX
        );
        assert_eq!(
            parse_floating_point_number(&text("1.7976931348623159e308")),
            Err(NumericParseError::FloatingPointOverflow)
        );
        assert_eq!(
            validate_floating_point_number(&text("1e9999")),
            Err(NumericParseError::FloatingPointOverflow)
        );
        let long = text(&format!("1{}e-400", "0".repeat(400)));
        assert_eq!(parse_floating_point_number(&long).unwrap().value(), 1.0);
        for input in ["-0", "-1e-9999", "0e9999"] {
            assert_eq!(
                parse_floating_point_number(&text(input))
                    .unwrap()
                    .value()
                    .to_bits(),
                0.0f64.to_bits()
            );
        }
    }

    #[test]
    fn checked_native_floats_reject_nonfinite_values_and_serialize_author_syntax() {
        for value in [f64::NAN, f64::INFINITY, f64::NEG_INFINITY] {
            assert_eq!(
                HtmlFloatingPointNumber::try_from(value),
                Err(HtmlNonFiniteNumberError)
            );
        }
        for value in [-0.0, -1.25, f64::MAX, f64::MIN_POSITIVE, f64::from_bits(1)] {
            let number = HtmlFloatingPointNumber::try_from(value).unwrap();
            assert_eq!(
                validate_floating_point_number(&number.as_document_string()),
                Ok(number)
            );
        }
        assert_eq!(
            HtmlFloatingPointNumber::try_from(-0.0)
                .unwrap()
                .as_document_string(),
            text("0")
        );
    }

    #[test]
    fn floating_lists_validate_author_delimiters_and_recover_garbage_as_specified() {
        assert_eq!(
            validate_floating_point_list(&text("1,-.5,2e2"))
                .unwrap()
                .values()
                .iter()
                .map(|n| n.value())
                .collect::<Vec<_>>(),
            vec![1.0, -0.5, 200.0]
        );
        for input in ["1, 2", "1;2", "1,", ",1", "1,,2", "+1"] {
            assert_eq!(
                validate_floating_point_list(&text(input)),
                Err(NumericParseError::InvalidSyntax)
            );
        }
        assert!(
            validate_floating_point_list(&text(""))
                .unwrap()
                .values()
                .is_empty()
        );
        let recovered = parse_floating_point_list(&text(" ,;garbage+2;bad;.;-3tail;1e9999;"));
        assert_eq!(
            recovered
                .values()
                .iter()
                .map(|n| n.value())
                .collect::<Vec<_>>(),
            vec![2.0, 0.0, 0.0, -3.0, 0.0]
        );
        assert_eq!(recovered.as_document_string(), text("2,0,0,-3,0"));
        assert_eq!(
            parse_floating_point_list(&text("garbage")).values()[0].value(),
            0.0
        );
        assert!(parse_floating_point_list(&text(" ,; ")).values().is_empty());
    }

    #[test]
    fn author_list_overflow_does_not_hide_later_invalid_syntax() {
        for input in ["1e999,nope", "1e999,", "1e999, 1", "1e999,+1"] {
            assert_eq!(
                validate_floating_point_list(&text(input)),
                Err(NumericParseError::InvalidSyntax)
            );
        }
        assert_eq!(
            validate_floating_point_list(&text("1e999,1.5")),
            Err(NumericParseError::FloatingPointOverflow)
        );
    }

    #[test]
    fn native_dimension_decimals_preserve_fraction_zeroes_and_exact_magnitude() {
        let integer = HtmlNonNegativeInteger::from_decimal_digits(&text(
            "340282366920938463463374607431768211456",
        ))
        .unwrap();
        let decimal =
            HtmlNonNegativeDecimal::with_fraction_digits(integer, &text("0001200")).unwrap();
        assert_eq!(decimal.fractional_digits(), Some("00012"));
        assert_eq!(
            decimal.as_document_string(),
            text("340282366920938463463374607431768211456.00012")
        );
        let percentage = HtmlDimension::Percentage(decimal);
        assert_eq!(
            percentage.as_document_string(),
            text("340282366920938463463374607431768211456.00012%")
        );
        assert_eq!(
            HtmlNonNegativeDecimal::with_fraction_digits(
                HtmlNonNegativeInteger::from(1),
                &text("")
            ),
            Err(HtmlDecimalDigitsError::Empty)
        );
        assert!(
            HtmlNonNegativeDecimal::with_fraction_digits(
                HtmlNonNegativeInteger::from(1),
                &text(" 5")
            )
            .is_err()
        );
        let zero = HtmlNonNegativeDecimal::with_fraction_digits(
            HtmlNonNegativeInteger::from(0),
            &text("000"),
        )
        .unwrap();
        assert!(zero.is_zero());
        assert_eq!(zero.fractional_digits(), None);
        assert_eq!(zero.integer_part().decimal_digits(), "0");
    }

    #[test]
    fn exact_integer_decrement_borrows_and_stops_at_zero() {
        assert_eq!(HtmlNonNegativeInteger::from(0).decremented(), None);
        for (input, expected) in [
            ("1", "0"),
            ("10", "9"),
            ("1000", "999"),
            ("1010", "1009"),
            (
                "340282366920938463463374607431768211456",
                "340282366920938463463374607431768211455",
            ),
        ] {
            let value = HtmlNonNegativeInteger::from_decimal_digits(&text(input)).unwrap();
            assert_eq!(value.decremented().unwrap().decimal_digits(), expected);
            assert_eq!(value.decimal_digits(), input);
        }
    }

    #[test]
    fn exact_integer_division_handles_large_magnitudes_and_boundary_divisors() {
        // Independent base-ten expectations: 2^128 / 400; and the geometric
        // identity (2^32)^4 = (2^32 - 1)(1 + 2^32 + 2^64 + 2^96) + 1.
        for (input, divisor, quotient, remainder) in [
            ("0", 400, "0", 0),
            ("399", 400, "0", 399),
            ("400", 400, "1", 0),
            ("401", 400, "1", 1),
            ("4294967294", u32::MAX, "0", u32::MAX - 1),
            ("4294967295", u32::MAX, "1", 0),
            ("4294967296", u32::MAX, "1", 1),
            (
                "340282366920938463463374607431768211456",
                400,
                "850705917302346158658436518579420528",
                256,
            ),
            (
                "340282366920938463463374607431768211456",
                u32::MAX,
                "79228162532711081671548469249",
                1,
            ),
            (
                "340282366920938463463374607431768211456",
                1,
                "340282366920938463463374607431768211456",
                0,
            ),
        ] {
            let value = HtmlNonNegativeInteger::from_decimal_digits(&text(input)).unwrap();
            let (actual_quotient, actual_remainder) =
                value.quotient_remainder(NonZeroU32::new(divisor).unwrap());
            assert_eq!(actual_quotient.decimal_digits(), quotient);
            assert_eq!(actual_remainder, remainder);
            assert!(actual_remainder < divisor);
            assert_eq!(
                actual_quotient
                    .multiplied_by(divisor)
                    .added_to(&HtmlNonNegativeInteger::from(u128::from(actual_remainder))),
                value
            );
        }
    }

    #[test]
    fn bounded_integer_division_matches_machine_arithmetic_without_projection_loss() {
        for value in [0, 1, 399, 400, 401, u64::MAX as u128, u128::MAX] {
            for divisor in [1, 2, 400, u32::MAX - 1, u32::MAX] {
                let (quotient, remainder) = HtmlNonNegativeInteger::from(value)
                    .quotient_remainder(NonZeroU32::new(divisor).unwrap());
                assert_eq!(quotient.to_u128(), Ok(value / u128::from(divisor)));
                assert_eq!(u128::from(remainder), value % u128::from(divisor));
            }
        }
    }

    fn decimal(integer: &str, fraction: &str) -> HtmlNonNegativeDecimal {
        HtmlNonNegativeDecimal::with_fraction_digits(
            HtmlNonNegativeInteger::from_decimal_digits(&text(integer)).unwrap(),
            &text(fraction),
        )
        .unwrap()
    }

    #[test]
    fn exact_decimal_addition_aligns_places_and_carries_into_huge_integers() {
        for (left_integer, left_fraction, right_integer, right_fraction, expected) in [
            ("0", "999", "0", "001", "1"),
            ("0", "0125", "0", "5", "0.5125"),
            ("0", "5", "0", "05", "0.55"),
            ("0", "00000001", "0", "00000009", "0.0000001"),
            ("9", "75", "0", "25", "10"),
            (
                "340282366920938463463374607431768211456",
                "9999",
                "0",
                "0001",
                "340282366920938463463374607431768211457",
            ),
        ] {
            let left = decimal(left_integer, left_fraction);
            let right = decimal(right_integer, right_fraction);
            let sum = left.added_to(&right);
            assert_eq!(sum.as_document_string(), text(expected));
            assert_eq!(right.added_to(&left), sum);
        }
        let integer = HtmlNonNegativeDecimal::from(HtmlNonNegativeInteger::from(400));
        assert_eq!(integer.added_to(&integer).as_document_string(), text("800"));
        assert_eq!(
            integer.added_to(&decimal("0", "025")).as_document_string(),
            text("400.025")
        );
    }

    #[test]
    fn exact_decimal_multiplication_preserves_fractional_places_and_bounded_carries() {
        for (integer, fraction, multiplier, expected) in [
            ("0", "25", u32::MAX, "1073741823.75"),
            ("9", "75", 4, "39"),
            ("0", "0001", 100, "0.01"),
            ("0", "0001", 10_000, "1"),
            (
                "340282366920938463463374607431768211456",
                "75",
                4,
                "1361129467683753853853498429727072845827",
            ),
        ] {
            assert_eq!(
                decimal(integer, fraction)
                    .multiplied_by(multiplier)
                    .as_document_string(),
                text(expected)
            );
        }
        let integer = HtmlNonNegativeDecimal::from(HtmlNonNegativeInteger::from(400));
        assert_eq!(
            integer.multiplied_by(400).as_document_string(),
            text("160000")
        );
        assert_eq!(integer.multiplied_by(1), integer);
    }

    #[test]
    fn decimal_arithmetic_normalizes_zero_and_equivalent_trailing_zeroes() {
        let zero = HtmlNonNegativeDecimal::from(HtmlNonNegativeInteger::from(0));
        let fractional_zero = decimal("000", "0000");
        assert_eq!(fractional_zero, zero);
        assert_eq!(zero.added_to(&fractional_zero), zero);
        assert_eq!(fractional_zero.multiplied_by(u32::MAX), zero);
        let value = decimal("001", "0025000");
        assert_eq!(value, decimal("1", "0025"));
        assert_eq!(value.added_to(&zero), value);
        assert_eq!(value.multiplied_by(1), value);
        assert_eq!(value.multiplied_by(0), zero);
        assert_eq!(value.added_to(&value).as_document_string(), text("2.005"));
    }

    #[test]
    fn single_dimension_recovery_uses_digit_prefix_and_immediate_percentage_marker() {
        for (input, expected) in [
            (" 12.50%tail", "12.5%"),
            ("12.50 %", "12.5"),
            ("1.%", "1%"),
            ("1. %", "1"),
            ("3*", "3"),
            ("2e3%", "2"),
            ("7junk%", "7"),
        ] {
            assert_eq!(
                parse_dimension(&text(input)).unwrap().as_document_string(),
                text(expected)
            );
        }
        for input in ["", " ", ".5%", "+1", "-0", "\u{a0}1", "١"] {
            assert_eq!(
                parse_dimension(&text(input)),
                Err(NumericParseError::InvalidSyntax)
            );
        }
    }

    #[test]
    fn nonzero_dimension_checks_exact_zero_without_binary64_underflow() {
        for input in ["0", "0.000%", "000.0tail"] {
            assert_eq!(
                parse_nonzero_dimension(&text(input)),
                Err(NumericParseError::ZeroDimension)
            );
        }
        let tiny = format!("0.{}1%", "0".repeat(400));
        let value = parse_nonzero_dimension(&text(&tiny)).unwrap();
        assert_eq!(value.as_document_string(), text(&tiny));
        let huge = "340282366920938463463374607431768211456.125%";
        assert_eq!(
            parse_nonzero_dimension(&text(huge))
                .unwrap()
                .as_document_string(),
            text(huge)
        );
    }

    #[test]
    fn dimension_list_selected_decimal_cursor_correction_exposes_fraction_and_unit() {
        // Independent expectations selected in #1133:
        // ../../../references/html-dimension-list-cursor--webkit-73aa6c89e2cb-blink-7984f9d11800.md
        for (input, expected) in [
            ("1.5%", "1.5%"),
            ("1.5*", "1.5*"),
            ("1.5", "1.5"),
            (".5%", "0.5%"),
            ("1.%", "1%"),
            ("1. 5 0%", "1.5%"),
            ("2", "2"),
            ("3%", "3%"),
            ("4*", "4*"),
        ] {
            let list = parse_dimension_list(&text(input));
            assert_eq!(list.values().len(), 1);
            assert_eq!(list.as_document_string(), text(expected));
        }
        assert!(matches!(
            parse_dimension_list(&text("1.5%")).values()[0],
            HtmlDimension::Percentage(_)
        ));
        assert!(matches!(
            parse_dimension_list(&text("1.5*")).values()[0],
            HtmlDimension::Relative(_)
        ));
        assert!(matches!(
            parse_dimension_list(&text("1.5")).values()[0],
            HtmlDimension::Absolute(_)
        ));
    }

    #[test]
    fn dimension_list_preserves_infra_comma_trimming_and_pinned_empty_token_rules() {
        for input in ["", ","] {
            assert!(parse_dimension_list(&text(input)).values().is_empty());
        }
        for (input, expected) in [
            (" , ", "0*,0*"),
            (",,", "0*"),
            ("1,,2,,", "1,0*,2"),
            (" 1. 5 0% , .5* , 2,", "1.5%,0.5*,2"),
            ("junk, * , % , 1tail%", "0,0*,0%,1"),
            ("\u{a0}1%", "0"),
        ] {
            assert_eq!(
                parse_dimension_list(&text(input)).as_document_string(),
                text(expected)
            );
        }
        let huge = "340282366920938463463374607431768211456.0001200*";
        assert_eq!(
            parse_dimension_list(&text(huge)).as_document_string(),
            text("340282366920938463463374607431768211456.00012*")
        );
    }
    #[test]
    fn scientific_integer_constraint_is_exact_before_float_rounding() {
        for input in ["0", "-0e999", "1.0", "100e-2", "1e999", "-42"] {
            assert!(
                strict_scientific_decimal_key(&text(input))
                    .unwrap()
                    .is_integer(),
                "{input}"
            );
        }
        for input in ["1.00000000000000001", "1e-999", "100e-3", "-.01"] {
            assert!(
                !strict_scientific_decimal_key(&text(input))
                    .unwrap()
                    .is_integer(),
                "{input}"
            );
        }
    }

    #[test]
    fn scientific_order_handles_sign_zero_virtual_digits_and_unbounded_exponents() {
        for (a, b, expected) in [
            ("-0", "0e999", Ordering::Equal),
            ("-1", "0", Ordering::Less),
            ("0", "-1", Ordering::Greater),
            ("0", "1", Ordering::Less),
            ("1", "0", Ordering::Greater),
            ("1", "1.00000000000000001", Ordering::Less),
            ("1.2", "1.20", Ordering::Equal),
            ("12e-1", "1.19", Ordering::Greater),
            ("-12e-1", "-1.19", Ordering::Less),
            ("999", "1e3", Ordering::Less),
            (
                "9e999999999999999999999999",
                "1e1000000000000000000000000",
                Ordering::Less,
            ),
            ("9007199254740992", "9007199254740993", Ordering::Less),
        ] {
            let left = strict_scientific_decimal_key(&text(a)).unwrap();
            let right = strict_scientific_decimal_key(&text(b)).unwrap();
            assert_eq!(left.compare(&right), Ok(expected), "{a} vs {b}");
            assert_eq!(right.compare(&left), Ok(expected.reverse()));
            assert_eq!(
                strict_scientific_decimal_key_units(text(a).code_units()).unwrap(),
                left
            );
        }
    }

    #[test]
    fn signed_integer_order_uses_arbitrary_magnitudes() {
        let magnitude = HtmlNonNegativeInteger::from_decimal_digits(&text(
            "340282366920938463463374607431768211456",
        ))
        .unwrap();
        let positive = HtmlInteger::from_magnitude(magnitude.clone(), false);
        let negative = HtmlInteger::from_magnitude(magnitude, true);
        assert_eq!(
            negative.compare(&HtmlInteger::from(i128::MIN)),
            Ordering::Less
        );
        assert_eq!(
            positive.compare(&HtmlInteger::from(i128::MAX)),
            Ordering::Greater
        );
        assert_eq!(negative.compare(&positive), Ordering::Less);
        assert_eq!(positive.compare(&positive), Ordering::Equal);
        assert_eq!(
            HtmlInteger::from(-2).compare(&HtmlInteger::from(-1)),
            Ordering::Less
        );
    }
    #[test]
    fn bounded_floating_grammar_keeps_author_and_recovery_components_and_conversion_distinct() {
        for (raw, token, integer, fraction, exponent) in [
            (" \t+001.20e-03tail", "+001.20e-03", "001", "20", Some("03")),
            ("1.e+oops", "1.", "1", "", None),
            ("-.5e+", "-.5", "", "5", None),
        ] {
            let units = raw.encode_utf16().collect::<Vec<_>>();
            let mut cursor =
                FloatingPointGrammarCursor::new(&units, FloatingPointParseMode::Recovery);
            let mut actions = 0;
            while !cursor.is_finished() {
                cursor.advance();
                actions += 1;
                assert!(actions <= units.len() * 4 + 12);
            }
            let prefix = cursor.prefix().unwrap();
            assert_eq!(prefix.token, token.encode_utf16().collect::<Vec<_>>());
            assert_eq!(
                prefix.integer_digits,
                integer.encode_utf16().collect::<Vec<_>>()
            );
            assert_eq!(
                prefix.fraction_digits,
                fraction.encode_utf16().collect::<Vec<_>>()
            );
            assert_eq!(
                prefix.exponent.map(|e| e.digits.to_vec()),
                exponent.map(|e| e.encode_utf16().collect::<Vec<_>>())
            );
            assert!(matches!(
                floating_point_prefix(&units, FloatingPointParseMode::Author),
                Err(NumericParseError::InvalidSyntax)
            ));
        }
        let units = text("1e999");
        let mut cursor =
            FloatingPointGrammarCursor::new(units.code_units(), FloatingPointParseMode::Author);
        while !cursor.is_finished() {
            cursor.advance();
        }
        assert!(cursor.is_valid());
        assert_eq!(cursor.prefix().unwrap().token, units.code_units());
        assert_eq!(
            validate_floating_point_number(&units),
            Err(NumericParseError::FloatingPointOverflow)
        );
    }
}
