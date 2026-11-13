//! Intrinsic numeric values from the pinned HTML common microsyntaxes.
//!
//! Author validation and user-agent recovery are distinct entry points. Parsing
//! exposes intentional bounded operations through `bounded`; the caller applies
//! request limits before invoking these algorithms.

use crate::internal::microsyntax::tokens::is_ascii_whitespace;
use crate::internal::values::DocumentString;
use ordered::Order;
use std::{cmp::Ordering, error::Error, fmt, num::NonZeroU32};
pub(crate) mod decimal_total;
pub(crate) mod natural_ratio;
mod ordered;
pub(crate) mod range;

// Digit visits made by the decimal kernels on this thread, for work-growth tests.
#[cfg(test)]
thread_local! {
    static DIGIT_STEPS: std::cell::Cell<u64> = const { std::cell::Cell::new(0) };
}
#[cfg(test)]
pub(crate) fn count_digit_steps(steps: usize) {
    DIGIT_STEPS.with(|total| total.set(total.get() + steps as u64));
}
/// Runs `operation` and returns its result with the digit visits it made.
#[cfg(test)]
pub(crate) fn digit_steps_during<R>(operation: impl FnOnce() -> R) -> (R, u64) {
    let before = DIGIT_STEPS.with(std::cell::Cell::get);
    let result = operation();
    (result, DIGIT_STEPS.with(std::cell::Cell::get) - before)
}

/// The capacity to reserve ahead for `len` digits and `extra` more. It is only a reservation hint,
/// since every writer grows its buffer as it pushes: it saturates at the largest length a digit
/// string can have, so no existing length can make it unrepresentable.
fn capacity_hint(len: usize, extra: usize) -> usize {
    len.saturating_add(extra)
        .min(isize::MAX as usize / size_of::<char>())
}

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
    #[cfg(test)]
    count_digit_steps(output.len());
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
    #[cfg(test)]
    count_digit_steps(output.len());
}

fn trim_trailing_zeroes(fraction: &mut Vec<u8>) {
    while fraction.last() == Some(&b'0') {
        fraction.pop();
    }
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
    #[cfg(test)]
    count_digit_steps(output.len());
    carry
}

fn write_fraction_product(input: &[u8], multiplier: u32, output: &mut [u8]) -> u64 {
    let mut carry = 0;
    for (index, &digit) in input.iter().enumerate().rev() {
        let product = u64::from(digit - b'0') * u64::from(multiplier) + carry;
        output[index] = b'0' + (product % 10) as u8;
        carry = product / 10;
    }
    #[cfg(test)]
    count_digit_steps(input.len());
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

/// Takes ownership of a metered digit buffer as a string without a second allocation. It scans the
/// bytes once for the first one that is not an ASCII digit and reports it, with its offset, as the
/// digits error the controlled operations already carry; a buffer of digits alone is valid UTF-8,
/// so the move into a string follows. Writers push only ASCII digits, so no valid operation
/// reaches the error.
fn ascii_digit_string(bytes: Vec<u8>) -> Result<String, HtmlDecimalDigitsError> {
    #[cfg(test)]
    count_digit_steps(bytes.len());
    let non_digit = |offset: usize, bytes: &[u8]| HtmlDecimalDigitsError::NonAsciiDigit {
        code_unit_offset: offset,
        code_unit: bytes.get(offset).map_or(0, |byte| u16::from(*byte)),
    };
    if let Some(offset) = bytes.iter().position(|byte| !byte.is_ascii_digit()) {
        return Err(non_digit(offset, &bytes));
    }
    String::from_utf8(bytes).map_err(|rejected| {
        let offset = rejected.utf8_error().valid_up_to();
        non_digit(offset, rejected.as_bytes())
    })
}

/// One decimal digit. Only the units digit of a number makes one, so a digit string never holds
/// anything else.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
struct Digit(u8);
impl Digit {
    /// The units digit of `value`.
    fn units(value: u64) -> Self {
        Self((value % 10) as u8)
    }
    fn is_zero(self) -> bool {
        self.0 == 0
    }
    fn character(self) -> char {
        char::from(b'0' + self.0)
    }
}

/// Which computed places an unmetered digit string keeps.
#[derive(Clone, Copy, Debug)]
enum Keep {
    /// From the most significant nonzero place down to the units place, or a lone zero when every
    /// place is zero: an integer.
    Integer,
    /// From the most significant place down to the least significant nonzero place, or nothing
    /// when every place is zero: the fractional digits, whose leading zeroes retain scale and
    /// whose trailing zeroes add nothing.
    Fraction,
}

/// The places a block of an unmetered digit string covers in one pass.
const DIGIT_BLOCK: usize = 256;

/// The digits of an unmetered operation, written into their string most significant first.
///
/// A carry or borrow runs from the least significant place upward, so `step` computes the digit of
/// one place and the state for the next, and the places are visited in that order. The string needs
/// the most significant digit first, so one pass finds the places to keep and records the state
/// before every block of [`DIGIT_BLOCK`] places, and a second pass recomputes the blocks from the
/// top down and writes each block's digits in reverse. The string is the only buffer of the
/// digits: nothing is copied into it, and the working storage is one state per block and one
/// block of digits. It starts with `capacity` bytes, or the digits it holds if they are more, and
/// the state after the last place comes back with it.
fn digit_string<S: Copy>(
    places: usize,
    keep: Keep,
    capacity: usize,
    start: S,
    step: impl Fn(S, usize) -> (Digit, S),
) -> (String, S) {
    let mut state = start;
    let mut checkpoints = Vec::new();
    let (mut lowest, mut highest) = (None, None);
    for place in 0..places {
        if place % DIGIT_BLOCK == 0 {
            checkpoints.push(state);
        }
        let (digit, next) = step(state, place);
        if !digit.is_zero() {
            lowest.get_or_insert(place);
            highest = Some(place);
        }
        state = next;
    }
    let range = match keep {
        Keep::Integer => Some((0, highest.unwrap_or(0))),
        Keep::Fraction => lowest.map(|lowest| (lowest, places.saturating_sub(1))),
    };
    let Some((low, high)) = range else {
        #[cfg(test)]
        count_digit_steps(places);
        return (String::with_capacity(capacity), state);
    };
    let kept = high.saturating_sub(low).saturating_add(1);
    let mut digits = String::with_capacity(capacity.max(kept));
    for (block, &checkpoint) in checkpoints.iter().enumerate().rev() {
        let first = block.saturating_mul(DIGIT_BLOCK);
        if first > high {
            continue;
        }
        let end = places.min(first.saturating_add(DIGIT_BLOCK));
        if end <= low {
            break;
        }
        let mut block_state = checkpoint;
        let mut block_digits = Vec::with_capacity(end.saturating_sub(first));
        for place in first..end {
            let (digit, next) = step(block_state, place);
            block_digits.push(digit);
            block_state = next;
        }
        for (offset, digit) in block_digits.iter().enumerate().rev() {
            if (low..=high).contains(&(first + offset)) {
                digits.push(digit.character());
            }
        }
    }
    #[cfg(test)]
    count_digit_steps(
        digits.len()
            + match keep {
                Keep::Integer => digits.len(),
                Keep::Fraction => places,
            },
    );
    (digits, state)
}

/// The value of the digit `place` positions above the units place of `digits`, written most
/// significant first, or zero above the first digit.
fn digit_at(digits: &[u8], place: usize) -> u64 {
    digits
        .len()
        .checked_sub(place.saturating_add(1))
        .and_then(|index| digits.get(index))
        .map_or(0, |&digit| u64::from(digit - b'0'))
}

/// The digits of `input * multiplier + carry`, as the integer a decimal's whole part holds.
fn product_string(input: &[u8], multiplier: u32, carry: u64, capacity: usize) -> String {
    // The carry past the input has at most as many digits as a u64.
    let places = input.len().saturating_add(20);
    digit_string(places, Keep::Integer, capacity, carry, |carry, place| {
        let product = digit_at(input, place) * u64::from(multiplier) + carry;
        (Digit::units(product), product / 10)
    })
    .0
}

/// The digits of `left + right + carry`.
fn sum_string(left: &[u8], right: &[u8], carry: u8, capacity: usize) -> String {
    let places = left.len().max(right.len()).saturating_add(1);
    digit_string(places, Keep::Integer, capacity, carry, |carry, place| {
        let sum = digit_at(left, place) + digit_at(right, place) + u64::from(carry);
        (Digit::units(sum), (sum / 10) as u8)
    })
    .0
}

/// The digits of the integer quotient of `input` by `divisor`, and the remainder.
fn quotient_string(input: &[u8], divisor: NonZeroU32) -> (String, u32) {
    let divisor = u64::from(divisor.get());
    let mut remainder = 0;
    let mut quotient = String::with_capacity(input.len());
    for &digit in input {
        let partial = remainder * 10 + u64::from(digit - b'0');
        remainder = partial % divisor;
        let digit = Digit::units(partial / divisor);
        if !digit.is_zero() || !quotient.is_empty() {
            quotient.push(digit.character());
        }
    }
    if quotient.is_empty() {
        quotient.push('0');
    }
    #[cfg(test)]
    count_digit_steps(quotient.len());
    (quotient, remainder as u32)
}

/// The fractional digits of `left + right`, kept at the larger scale with trailing zeroes
/// dropped, and the carry into the whole part.
fn fraction_sum_string(left: &[u8], right: &[u8]) -> (String, u8) {
    let scale = left.len().max(right.len());
    digit_string(scale, Keep::Fraction, scale, 0, |carry: u8, place| {
        let sum = fraction_digit(left, scale, place)
            + fraction_digit(right, scale, place)
            + u64::from(carry);
        (Digit::units(sum), (sum / 10) as u8)
    })
}

/// The fractional digits of `input * multiplier`, kept at the scale of `input` with trailing
/// zeroes dropped, and the carry into the whole part.
fn fraction_product_string(input: &[u8], multiplier: u32) -> (String, u64) {
    let scale = input.len();
    digit_string(scale, Keep::Fraction, scale, 0, |carry: u64, place| {
        let product = fraction_digit(input, scale, place) * u64::from(multiplier) + carry;
        (Digit::units(product), product / 10)
    })
}

/// The value of the fractional digit `place` positions above the last of `scale` places, or zero
/// where `digits` is shorter than the scale.
fn fraction_digit(digits: &[u8], scale: usize, place: usize) -> u64 {
    scale
        .checked_sub(place.saturating_add(1))
        .and_then(|index| digits.get(index))
        .map_or(0, |&digit| u64::from(digit - b'0'))
}

/// Observed cumulative capacity of every variable allocation made by one
/// concrete operation, including intermediates that have already been dropped.
/// The semantic session records this before interpreting either outcome.
#[derive(Debug)]
pub struct HtmlNumericMaterialized<T> {
    pub(crate) value: T,
    pub(crate) capacity_bytes: usize,
}

/// Why a controlled digit materialization failed: the digits were invalid, or the allocation hit a
/// boundary. The semantic session owns admission of both.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum HtmlNumericMaterializationError {
    /// The borrowed UTF-16 digits are empty or contain a code unit that is not an ASCII digit.
    Digits(HtmlDecimalDigitsError),
    /// The digit allocation exceeded a limit, could not be represented, or failed.
    Boundary(HtmlIntegerUnitStepError),
}

/// A failed materialization: its cause and the capacity of every allocation observed before it
/// failed, which the caller must still account for. Read it with `kind` and `capacity_bytes`.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct HtmlNumericMaterializationFailure {
    pub(crate) kind: HtmlNumericMaterializationError,
    pub(crate) capacity_bytes: usize,
}

fn materialization_boundary(
    error: HtmlIntegerUnitStepError,
    capacity_bytes: usize,
) -> HtmlNumericMaterializationFailure {
    HtmlNumericMaterializationFailure {
        kind: HtmlNumericMaterializationError::Boundary(error),
        capacity_bytes,
    }
}

fn materialization_digits(
    error: HtmlDecimalDigitsError,
    capacity_bytes: usize,
) -> HtmlNumericMaterializationFailure {
    HtmlNumericMaterializationFailure {
        kind: HtmlNumericMaterializationError::Digits(error),
        capacity_bytes,
    }
}

fn reserve_materialization(
    requested: usize,
    max_integer_digits: Option<usize>,
    max_capacity_bytes: usize,
    observed: &mut usize,
) -> Result<Vec<u8>, HtmlNumericMaterializationFailure> {
    if let Some(max_digits) = max_integer_digits {
        check_integer_digit_capacity(requested, max_digits, usize::MAX)
            .map_err(|e| materialization_boundary(e, *observed))?;
    } else if requested > isize::MAX as usize {
        return Err(materialization_boundary(
            HtmlIntegerUnitStepError::RepresentationExhausted,
            *observed,
        ));
    }
    let required = observed.checked_add(requested).ok_or_else(|| {
        materialization_boundary(HtmlIntegerUnitStepError::RepresentationExhausted, *observed)
    })?;
    if required > max_capacity_bytes {
        return Err(materialization_boundary(
            HtmlIntegerUnitStepError::RequestedCapacityLimitExceeded {
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
        materialization_boundary(HtmlIntegerUnitStepError::RepresentationExhausted, *observed)
    })?;
    allocation.map_err(|_| {
        materialization_boundary(
            HtmlIntegerUnitStepError::AllocationFailed { requested },
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
) -> Result<String, HtmlNumericMaterializationFailure> {
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
    ascii_digit_string(bytes).map_err(|e| materialization_digits(e, *observed))
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
    /// The upper bound on the digit-visiting steps the matching `try_from_borrowed_digits`
    /// performs, for the caller to precharge before it runs. It is a loop-derived envelope, not an
    /// instruction count or a cancellation quantum. `None` means the bound itself is not
    /// representable in `usize`. The argument is the digit count `length`.
    pub fn from_borrowed_digits_work_bound(length: usize) -> Option<usize> {
        bounded_work(&[length, length, length, length, length])
    }
    /// Builds the canonical integer from borrowed UTF-16 digits, dropping leading zeroes (zero
    /// stays `0`). The digits must be nonempty ASCII digits. The result has at most `max_digits`
    /// digits, and the total observed allocation stays within `max_bytes`. Failure reports the
    /// cause and the capacity observed so far.
    pub fn try_from_borrowed_digits(
        input: &[u16],
        max_digits: usize,
        max_bytes: usize,
    ) -> Result<HtmlNumericMaterialized<Self>, HtmlNumericMaterializationFailure> {
        let mut observed = 0;
        let digits = controlled_digit_string(input, false, max_digits, max_bytes, &mut observed)?;
        Ok(HtmlNumericMaterialized {
            value: Self { digits },
            capacity_bytes: observed,
        })
    }
    // Existing fixed-width constant owner scans/outputs at most usize's decimal
    // width. On the supported 32/64-bit targets, four passes of 20 bound it.
    /// The upper bound on the digit-visiting steps the matching `try_from_u32` performs, for the
    /// caller to precharge before it runs. It is a loop-derived envelope, not an instruction count
    /// or a cancellation quantum. `None` means the bound itself is not representable in `usize`. It
    /// is fixed, because a `u32` has a bounded decimal width.
    pub fn from_u32_work_bound() -> Option<usize> {
        bounded_work(&[20, 20, 20, 20])
    }
    /// Builds the integer for `value` with the same digit and byte limits as
    /// `try_from_borrowed_digits`.
    pub fn try_from_u32(
        value: u32,
        max_digits: usize,
        max_bytes: usize,
    ) -> Result<HtmlNumericMaterialized<Self>, HtmlNumericMaterializationFailure> {
        let result = HtmlInteger::try_from_usize(value as usize, max_digits, max_bytes)
            .map_err(unit_step_materialization_failure)?;
        Ok(HtmlNumericMaterialized {
            value: result.value.magnitude,
            capacity_bytes: result.capacity_bytes,
        })
    }
    // Copy reads/writes n+n and UTF-8 n; native canonical input needs no zero scan.
    /// The upper bound on the digit-visiting steps the matching `try_copy` performs, for the caller
    /// to precharge before it runs. It is a loop-derived envelope, not an instruction count or a
    /// cancellation quantum. `None` means the bound itself is not representable in `usize`.
    pub fn copy_work_bound(&self) -> Option<usize> {
        bounded_work(&[self.digits.len(), self.digits.len(), self.digits.len()])
    }
    /// Copies the integer into one fresh allocation under the digit and byte limits; the copy is
    /// equal to the original.
    pub fn try_copy(
        &self,
        max_digits: usize,
        max_bytes: usize,
    ) -> Result<HtmlNumericMaterialized<Self>, HtmlNumericMaterializationFailure> {
        let mut observed = 0;
        let mut bytes = reserve_materialization(
            self.digits.len(),
            Some(max_digits),
            max_bytes,
            &mut observed,
        )?;
        bytes.extend_from_slice(self.digits.as_bytes());
        let digits = ascii_digit_string(bytes).map_err(|e| materialization_digits(e, observed))?;
        Ok(HtmlNumericMaterialized {
            value: Self { digits },
            capacity_bytes: observed,
        })
    }
    // Division n input visits + n writes, UTF-8 n, canonical quotient length
    // prepass n. The prepass shares write_quotient's exact recurrence below.
    /// The upper bound on the digit-visiting steps the matching `try_quotient_remainder` performs,
    /// for the caller to precharge before it runs. It is a loop-derived envelope, not an
    /// instruction count or a cancellation quantum. `None` means the bound itself is not
    /// representable in `usize`.
    pub fn quotient_remainder_work_bound(&self) -> Option<usize> {
        let n = self.digits.len();
        bounded_work(&[n, n, n, n])
    }
    /// Divides by `divisor`, returning the canonical quotient and the remainder, which is less than
    /// `divisor`. The quotient allocation obeys `max_digits` and `max_bytes`.
    pub fn try_quotient_remainder(
        &self,
        divisor: NonZeroU32,
        max_digits: usize,
        max_bytes: usize,
    ) -> Result<HtmlNumericMaterialized<(Self, u32)>, HtmlNumericMaterializationFailure> {
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
        let digits = ascii_digit_string(bytes).map_err(|e| materialization_digits(e, observed))?;
        Ok(HtmlNumericMaterialized {
            value: (Self { digits }, remainder),
            capacity_bytes: observed,
        })
    }
    // Pivot n, prefix-copy reads n, writes n; zero returns no allocation.
    /// The upper bound on the digit-visiting steps the matching `try_decremented` performs, for the
    /// caller to precharge before it runs. It is a loop-derived envelope, not an instruction count
    /// or a cancellation quantum. `None` means the bound itself is not representable in `usize`.
    pub fn decremented_work_bound(&self) -> Option<usize> {
        let n = self.digits.len();
        bounded_work(&[n, n, n])
    }
    /// Subtracts one. Zero has no predecessor, so it returns `None` without allocating; otherwise
    /// the result is the canonical predecessor under the digit and byte limits.
    pub fn try_decremented(
        &self,
        max_digits: usize,
        max_bytes: usize,
    ) -> Result<HtmlNumericMaterialized<Option<Self>>, HtmlNumericMaterializationFailure> {
        let Some(nonzero) = NonZeroDigits::new(&self.digits) else {
            return Ok(HtmlNumericMaterialized {
                value: None,
                capacity_bytes: 0,
            });
        };
        let digits = try_unit_step_digits(UnitStep::Predecessor(nonzero), max_digits, max_bytes)
            .map_err(unit_step_materialization_failure)?;
        let capacity_bytes = digits.capacity();
        Ok(HtmlNumericMaterialized {
            value: Some(Self { digits }),
            capacity_bytes,
        })
    }
    // Length pass n+10; product reads/writes n+10 each; reversal reads
    // and writes at most n+10 each; UTF-8 validation n+10.
    /// The upper bound on the digit-visiting steps the matching `try_multiplied_by` performs, for
    /// the caller to precharge before it runs. It is a loop-derived envelope, not an instruction
    /// count or a cancellation quantum. `None` means the bound itself is not representable in
    /// `usize`.
    pub fn multiplied_by_work_bound(&self) -> Option<usize> {
        let n = self.digits.len().checked_add(10)?;
        bounded_work(&[n, n, n, n, n, n])
    }
    /// Multiplies by `multiplier`. A zero multiplier or a zero value yields zero. The canonical
    /// product obeys `max_digits` and `max_bytes`.
    pub fn try_multiplied_by(
        &self,
        multiplier: u32,
        max_digits: usize,
        max_bytes: usize,
    ) -> Result<HtmlNumericMaterialized<Self>, HtmlNumericMaterializationFailure> {
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
        Ok(HtmlNumericMaterialized {
            value: Self { digits },
            capacity_bytes: observed,
        })
    }
    // Carry prepass and sum each read m+m. Output writes m+1; reversal
    // reads/writes m+1 each; UTF-8 validation m+1.
    /// The upper bound on the digit-visiting steps the matching `try_added_to` performs, for the
    /// caller to precharge before it runs. It is a loop-derived envelope, not an instruction count
    /// or a cancellation quantum. The bound is `8 * m + 12`, where `m` is the digit count of the
    /// longer operand. `None` means that value overflows `usize`.
    pub fn added_to_work_bound(&self, other: &Self) -> Option<usize> {
        let m = self.digits.len().max(other.digits.len());
        let n = m.checked_add(1)?;
        bounded_work(&[m, m, m, m, n, n, n, n])
    }
    /// Adds `other`, producing the canonical sum under `max_digits` and `max_bytes`; the carry may
    /// add one digit.
    pub fn try_added_to(
        &self,
        other: &Self,
        max_digits: usize,
        max_bytes: usize,
    ) -> Result<HtmlNumericMaterialized<Self>, HtmlNumericMaterializationFailure> {
        let mut observed = 0;
        let digits = controlled_sum(
            self.digits.as_bytes(),
            other.digits.as_bytes(),
            0,
            max_digits,
            max_bytes,
            &mut observed,
        )?;
        Ok(HtmlNumericMaterialized {
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
) -> Result<String, HtmlNumericMaterializationFailure> {
    input.len().checked_add(10).ok_or_else(|| {
        materialization_boundary(HtmlIntegerUnitStepError::RepresentationExhausted, *observed)
    })?;
    let required = product_length(input, multiplier, carry);
    let mut bytes = reserve_materialization(required, Some(max_digits), max_bytes, observed)?;
    write_product(input, multiplier, carry, &mut bytes);
    ascii_digit_string(bytes).map_err(|e| materialization_digits(e, *observed))
}

fn controlled_sum(
    left: &[u8],
    right: &[u8],
    carry: u8,
    max_digits: usize,
    max_bytes: usize,
    observed: &mut usize,
) -> Result<String, HtmlNumericMaterializationFailure> {
    let required = left
        .len()
        .max(right.len())
        .checked_add(usize::from(sum_carry(left, right, carry)))
        .ok_or_else(|| {
            materialization_boundary(HtmlIntegerUnitStepError::RepresentationExhausted, *observed)
        })?;
    let mut bytes = reserve_materialization(required, Some(max_digits), max_bytes, observed)?;
    write_sum(left, right, carry, &mut bytes);
    ascii_digit_string(bytes).map_err(|e| materialization_digits(e, *observed))
}

/// A nonzero magnitude's canonical digits and the position of its last nonzero digit. Only a
/// nonzero magnitude has a predecessor, so the pivot of a predecessor is the one this type holds.
#[derive(Clone, Copy)]
struct NonZeroDigits<'a> {
    digits: &'a str,
    pivot: usize,
}
impl<'a> NonZeroDigits<'a> {
    /// The digits with their pivot, or `None` for zero.
    fn new(digits: &'a str) -> Option<Self> {
        let pivot = digits.as_bytes().iter().rposition(|&digit| digit != b'0')?;
        Some(Self { digits, pivot })
    }
    /// The length of the predecessor: it keeps this length except when a lone leading one drops
    /// out. The length never grows, so no size can be unrepresentable.
    fn predecessor_length(self) -> usize {
        if self.pivot == 0 && self.digits.as_bytes().first() == Some(&b'1') && self.digits.len() > 1
        {
            self.digits.len() - 1
        } else {
            self.digits.len()
        }
    }
}

/// The unit step to write: the successor of any magnitude, or the predecessor of a nonzero one.
/// Each carries the pivot its scan found, so writing it scans nothing again.
#[derive(Clone, Copy)]
enum UnitStep<'a> {
    /// The digits, and the position of their last digit that is not 9 (none for all nines).
    Successor(&'a str, Option<usize>),
    Predecessor(NonZeroDigits<'a>),
}
impl<'a> UnitStep<'a> {
    fn successor(digits: &'a str) -> Self {
        Self::Successor(
            digits,
            digits.as_bytes().iter().rposition(|&digit| digit != b'9'),
        )
    }
    /// The length of the result, or the error where a successor's length is not representable.
    fn result_length(self) -> Result<usize, HtmlIntegerUnitStepError> {
        match self {
            Self::Successor(digits, None) => digits
                .len()
                .checked_add(1)
                .ok_or(HtmlIntegerUnitStepError::RepresentationExhausted),
            Self::Successor(digits, Some(_)) => Ok(digits.len()),
            Self::Predecessor(digits) => Ok(digits.predecessor_length()),
        }
    }
}

fn write_unit_step(step: UnitStep<'_>, digits: &mut String) {
    match step {
        UnitStep::Successor(input, Some(pivot)) => {
            digits.push_str(&input[..pivot]);
            digits.push(char::from(input.as_bytes()[pivot] + 1));
            for _ in pivot + 1..input.len() {
                digits.push('0');
            }
        }
        UnitStep::Successor(input, None) => {
            digits.push('1');
            for _ in 0..input.len() {
                digits.push('0');
            }
        }
        UnitStep::Predecessor(NonZeroDigits {
            digits: input,
            pivot,
        }) => {
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
}

fn try_unit_step_digits(
    step: UnitStep<'_>,
    max_digits: usize,
    max_bytes: usize,
) -> Result<String, HtmlIntegerUnitStepError> {
    let required = step.result_length()?;
    let mut digits = reserve_integer_digits(required, max_digits, max_bytes)?;
    write_unit_step(step, &mut digits);
    Ok(digits)
}

fn unit_step_materialization_failure(
    error: HtmlIntegerUnitStepError,
) -> HtmlNumericMaterializationFailure {
    let capacity = match error {
        HtmlIntegerUnitStepError::ActualCapacityLimitExceeded { capacity_bytes, .. } => {
            capacity_bytes
        }
        _ => 0,
    };
    materialization_boundary(error, capacity)
}

impl HtmlInteger {
    // Pivot scan n, prefix-copy reads n, writes n+1, fixed sign/zero handling.
    /// The upper bound on the digit-visiting steps the matching `try_materialized_unit_step`
    /// performs, for the caller to precharge before it runs. It is a loop-derived envelope, not an
    /// instruction count or a cancellation quantum. `None` means the bound itself is not
    /// representable in `usize`.
    pub fn materialized_unit_step_work_bound(&self) -> Option<usize> {
        bounded_work(&[
            self.magnitude.digits.len(),
            self.magnitude.digits.len(),
            self.magnitude.digits.len().checked_add(1)?,
        ])
    }
    /// Increments or decrements the integer by one as `step` selects and reports the observed
    /// allocation capacity. Its digit and byte limits and exact result are those of
    /// `try_unit_step`; only the failure shape differs, since it uses
    /// `HtmlNumericMaterializationFailure`.
    pub fn try_materialized_unit_step(
        &self,
        step: HtmlIntegerUnitStep,
        max_digits: usize,
        max_bytes: usize,
    ) -> Result<HtmlNumericMaterialized<Self>, HtmlNumericMaterializationFailure> {
        self.try_unit_step(step, max_digits, max_bytes)
            .map(|r| HtmlNumericMaterialized {
                value: r.value,
                capacity_bytes: r.capacity_bytes,
            })
            .map_err(unit_step_materialization_failure)
    }
}

impl HtmlNonNegativeDecimal {
    // Each part uses validation, canonicalization, copy reads/writes and UTF-8.
    /// The upper bound on the digit-visiting steps the matching `try_from_borrowed_digits`
    /// performs, for the caller to precharge before it runs. It is a loop-derived envelope, not an
    /// instruction count or a cancellation quantum. `None` means the bound itself is not
    /// representable in `usize`. The arguments are the integer-part and fraction digit counts.
    pub fn from_borrowed_digits_work_bound(integer: usize, fraction: usize) -> Option<usize> {
        HtmlNonNegativeInteger::from_borrowed_digits_work_bound(integer)?.checked_add(
            HtmlNonNegativeInteger::from_borrowed_digits_work_bound(fraction)?,
        )
    }
    /// Builds the canonical decimal from borrowed UTF-16 digits of the integer part and the
    /// fraction. The integer part must be nonempty; the fraction may be empty, which is the exact
    /// integer case, and trailing zeroes of the fraction are dropped. Only the integer part counts
    /// against `max_digits`; `max_bytes` bounds the total observed allocation.
    pub fn try_from_borrowed_digits(
        integer: &[u16],
        fraction: &[u16],
        max_digits: usize,
        max_bytes: usize,
    ) -> Result<HtmlNumericMaterialized<Self>, HtmlNumericMaterializationFailure> {
        let mut observed = 0;
        let integer =
            controlled_digit_string(integer, false, max_digits, max_bytes, &mut observed)?;
        let fraction =
            controlled_digit_string(fraction, true, max_digits, max_bytes, &mut observed)?;
        Ok(HtmlNumericMaterialized {
            value: Self {
                integer: HtmlNonNegativeInteger { digits: integer },
                fraction,
            },
            capacity_bytes: observed,
        })
    }
    /// The upper bound on the digit-visiting steps the matching `try_copy` performs, for the caller
    /// to precharge before it runs. It is a loop-derived envelope, not an instruction count or a
    /// cancellation quantum. `None` means the bound itself is not representable in `usize`.
    pub fn copy_work_bound(&self) -> Option<usize> {
        let n = self.fraction.len();
        self.integer
            .copy_work_bound()?
            .checked_add(bounded_work(&[n, n, n])?)
    }
    /// Copies the decimal into fresh allocations; the copy is equal to the original. `max_digits`
    /// limits the integer part and `max_bytes` the total observed capacity.
    pub fn try_copy(
        &self,
        max_digits: usize,
        max_bytes: usize,
    ) -> Result<HtmlNumericMaterialized<Self>, HtmlNumericMaterializationFailure> {
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
        let integer =
            ascii_digit_string(integer).map_err(|e| materialization_digits(e, observed))?;
        let fraction =
            ascii_digit_string(fraction).map_err(|e| materialization_digits(e, observed))?;
        Ok(HtmlNumericMaterialized {
            value: Self {
                integer: HtmlNonNegativeInteger { digits: integer },
                fraction,
            },
            capacity_bytes: observed,
        })
    }
    // Fraction: initialized writes f, product reads/writes f+f, trailing trim f,
    // UTF-8 f; integer: length/read/write/reversal-read/reversal-write/UTF-8.
    /// The upper bound on the digit-visiting steps the matching `try_multiplied_by` performs, for
    /// the caller to precharge before it runs. It is a loop-derived envelope, not an instruction
    /// count or a cancellation quantum. `None` means the bound itself is not representable in
    /// `usize`.
    pub fn multiplied_by_work_bound(&self) -> Option<usize> {
        let f = self.fraction.len();
        self.integer
            .multiplied_by_work_bound()?
            .checked_add(bounded_work(&[f, f, f, f, f])?)
    }
    /// Multiplies by `multiplier`, carrying from the fraction into the integer part, with the
    /// canonical result under the limits of `try_from_borrowed_digits`. A zero multiplier or a zero
    /// value yields zero.
    pub fn try_multiplied_by(
        &self,
        multiplier: u32,
        max_digits: usize,
        max_bytes: usize,
    ) -> Result<HtmlNumericMaterialized<Self>, HtmlNumericMaterializationFailure> {
        if multiplier == 0 || self.is_zero() {
            return HtmlNonNegativeInteger::try_from_u32(0, max_digits, max_bytes).map(|r| {
                HtmlNumericMaterialized {
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
        let value =
            Self::from_metered_fraction(HtmlNonNegativeInteger { digits: integer }, fraction)
                .map_err(|e| materialization_digits(e, observed))?;
        Ok(HtmlNumericMaterialized {
            value,
            capacity_bytes: observed,
        })
    }
    // Fraction: initialization f, operand reads f+f, output f, trim f, UTF-8 f.
    // Integral carry is handled inside the shared sum, without a second sum.
    /// The upper bound on the digit-visiting steps the matching `try_added_to` performs, for the
    /// caller to precharge before it runs. It is a loop-derived envelope, not an instruction count
    /// or a cancellation quantum. `None` means the bound itself is not representable in `usize`.
    pub fn added_to_work_bound(&self, other: &Self) -> Option<usize> {
        let f = self.fraction.len().max(other.fraction.len());
        self.integer
            .added_to_work_bound(&other.integer)?
            .checked_add(bounded_work(&[f, f, f, f, f, f])?)
    }
    /// Adds `other` exactly, with the canonical result under the limits of
    /// `try_from_borrowed_digits`.
    pub fn try_added_to(
        &self,
        other: &Self,
        max_digits: usize,
        max_bytes: usize,
    ) -> Result<HtmlNumericMaterialized<Self>, HtmlNumericMaterializationFailure> {
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
        let value =
            Self::from_metered_fraction(HtmlNonNegativeInteger { digits: integer }, fraction)
                .map_err(|e| materialization_digits(e, observed))?;
        Ok(HtmlNumericMaterialized {
            value,
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
pub enum HtmlIntegerUnitStep {
    /// Adds one to the integer.
    Increment,
    /// Subtracts one from the integer. The result of decrementing zero is `-1`.
    Decrement,
}

/// One canonical integer and the actual capacity of its sole digit allocation.
#[derive(Debug)]
pub struct HtmlIntegerUnitStepResult {
    pub(crate) value: HtmlInteger,
    pub(crate) capacity_bytes: usize,
}

/// Numeric allocation boundaries; the semantic session owns their admission.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum HtmlIntegerUnitStepError {
    /// The digit count or a capacity sum overflows `usize` or exceeds `isize::MAX`, so no
    /// allocation could represent it.
    RepresentationExhausted,
    /// The result needs more digits than the caller's digit limit allows.
    DigitLimitExceeded {
        /// The caller's digit limit.
        limit: usize,
        /// The digits the result needs.
        required: usize,
    },
    /// The bytes the result needs exceed the caller's byte limit before allocation.
    RequestedCapacityLimitExceeded {
        /// The caller's byte limit.
        limit: usize,
        /// The bytes the operation needs in total, counting earlier allocations of the same operation.
        required: usize,
    },
    /// The allocator granted more capacity than the caller's byte limit although the request fit.
    ActualCapacityLimitExceeded {
        /// The caller's byte limit.
        limit: usize,
        /// The bytes requested so far, counting earlier allocations of the same operation when it
        /// materializes several buffers.
        requested: usize,
        /// The capacity in bytes the allocator has granted so far, counting earlier allocations
        /// of the same operation.
        capacity_bytes: usize,
    },
    /// The allocator could not provide the requested bytes.
    AllocationFailed {
        /// The bytes that were requested from the allocator.
        requested: usize,
    },
}

fn reserve_integer_digits(
    required: usize,
    max_result_digits: usize,
    max_capacity_bytes: usize,
) -> Result<String, HtmlIntegerUnitStepError> {
    check_integer_digit_capacity(required, max_result_digits, max_capacity_bytes)?;
    let mut digits = String::new();
    digits
        .try_reserve_exact(required)
        .map_err(|_| HtmlIntegerUnitStepError::AllocationFailed {
            requested: required,
        })?;
    check_integer_actual_capacity(required, digits.capacity(), max_capacity_bytes)?;
    Ok(digits)
}

fn check_integer_digit_capacity(
    required: usize,
    max_result_digits: usize,
    max_capacity_bytes: usize,
) -> Result<(), HtmlIntegerUnitStepError> {
    if required > isize::MAX as usize {
        return Err(HtmlIntegerUnitStepError::RepresentationExhausted);
    }
    if required > max_result_digits {
        return Err(HtmlIntegerUnitStepError::DigitLimitExceeded {
            limit: max_result_digits,
            required,
        });
    }
    if required > max_capacity_bytes {
        return Err(HtmlIntegerUnitStepError::RequestedCapacityLimitExceeded {
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
) -> Result<(), HtmlIntegerUnitStepError> {
    if capacity_bytes > max_capacity_bytes {
        Err(HtmlIntegerUnitStepError::ActualCapacityLimitExceeded {
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
    pub fn try_unit_step(
        &self,
        step: HtmlIntegerUnitStep,
        max_result_digits: usize,
        max_capacity_bytes: usize,
    ) -> Result<HtmlIntegerUnitStepResult, HtmlIntegerUnitStepError> {
        let input = self.magnitude.decimal_digits();
        let nonzero = NonZeroDigits::new(input);
        let (unit_step, negative) = match nonzero {
            // Zero grows to the successor, whichever way it steps.
            None => (
                UnitStep::successor(input),
                step == HtmlIntegerUnitStep::Decrement,
            ),
            Some(_) if (step == HtmlIntegerUnitStep::Increment) != self.negative => {
                (UnitStep::successor(input), self.negative)
            }
            Some(nonzero) => (UnitStep::Predecessor(nonzero), self.negative),
        };
        let digits = try_unit_step_digits(unit_step, max_result_digits, max_capacity_bytes)?;
        let capacity_bytes = digits.capacity();
        Ok(HtmlIntegerUnitStepResult {
            value: Self::from_magnitude(HtmlNonNegativeInteger { digits }, negative),
            capacity_bytes,
        })
    }

    /// Constructs the source's reversed-list count without infallible formatting.
    /// Both fixed-width scans and the write are bounded by `usize` decimal width.
    pub fn try_from_usize(
        value: usize,
        max_result_digits: usize,
        max_capacity_bytes: usize,
    ) -> Result<HtmlIntegerUnitStepResult, HtmlIntegerUnitStepError> {
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
        Ok(HtmlIntegerUnitStepResult {
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

/// An exact integer above zero, which therefore has a predecessor. Zero is refused at
/// construction, so the predecessor is total.
#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub(crate) struct HtmlPositiveInteger {
    magnitude: HtmlNonNegativeInteger,
    /// The position of the magnitude's last nonzero digit, which only a positive magnitude has.
    pivot: usize,
}
impl HtmlPositiveInteger {
    /// The positive integer, or the zero it was given back.
    pub(crate) fn new(magnitude: HtmlNonNegativeInteger) -> Result<Self, HtmlNonNegativeInteger> {
        match NonZeroDigits::new(&magnitude.digits) {
            Some(NonZeroDigits { pivot, .. }) => Ok(Self { magnitude, pivot }),
            None => Err(magnitude),
        }
    }
    pub(crate) fn magnitude(&self) -> &HtmlNonNegativeInteger {
        &self.magnitude
    }
    /// The exact integer below this one, written straight into its string.
    pub(crate) fn predecessor(&self) -> HtmlNonNegativeInteger {
        let nonzero = NonZeroDigits {
            digits: &self.magnitude.digits,
            pivot: self.pivot,
        };
        let mut digits = String::with_capacity(nonzero.predecessor_length());
        write_unit_step(UnitStep::Predecessor(nonzero), &mut digits);
        HtmlNonNegativeInteger { digits }
    }
}

impl HtmlNonNegativeInteger {
    fn compare(&self, other: &Self) -> Ordering {
        self.digits
            .len()
            .cmp(&other.digits.len())
            .then_with(|| self.digits.cmp(&other.digits))
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

    /// An integer from digits that recognition already checked to be ASCII decimal digits.
    pub(crate) fn from_recognized_digits(digits: &[u16]) -> Self {
        Self::from_digits(digits)
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

    /// Returns the actual owned decimal buffer capacity in bytes, in constant
    /// work without allocation. This includes spare capacity, excludes this
    /// value's inline header and allocator bookkeeping, and does not measure
    /// unrelated or shared allocations.
    pub fn retained_buffer_bytes(&self) -> usize {
        self.digits.capacity()
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
        Self {
            digits: product_string(
                self.digits.as_bytes(),
                multiplier,
                0,
                capacity_hint(self.digits.len(), 10),
            ),
        }
    }

    pub(crate) fn added_to(&self, other: &Self) -> Self {
        Self {
            digits: sum_string(
                self.digits.as_bytes(),
                other.digits.as_bytes(),
                0,
                capacity_hint(self.digits.len().max(other.digits.len()), 1),
            ),
        }
    }

    /// Subtracts one exactly, with no result below zero.
    pub(crate) fn decremented(&self) -> Option<Self> {
        let nonzero = NonZeroDigits::new(&self.digits)?;
        let mut digits = String::with_capacity(nonzero.predecessor_length());
        write_unit_step(UnitStep::Predecessor(nonzero), &mut digits);
        Some(Self { digits })
    }

    /// Divides an exact magnitude by a bounded nonzero integer.
    /// The remainder is strictly less than the divisor.
    pub(crate) fn quotient_remainder(&self, divisor: NonZeroU32) -> (Self, u32) {
        let (digits, remainder) = quotient_string(self.digits.as_bytes(), divisor);
        (Self { digits }, remainder)
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
#[non_exhaustive]
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
    /// Positive zero.
    pub(crate) const ZERO: Self = Self { value: 0.0 };
    /// One.
    pub(crate) const ONE: Self = Self { value: 1.0 };
    /// One hundred.
    pub(crate) const ONE_HUNDRED: Self = Self { value: 100.0 };

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
            // Multiply by the largest power of two that fits a u32 multiplier, 2^31, per pass.
            let mut remaining = exponent.unsigned_abs();
            while remaining > 0 {
                let step = remaining.min(31);
                coefficient = coefficient.multiplied_by(1_u32 << step);
                remaining -= step;
            }
            coefficient.into()
        } else {
            // The largest scale is 1074. n / 2^k = n * 5^k / 10^k exactly.
            let scale = (-exponent) as usize;
            // Multiply by the largest power of five that fits a u32 multiplier, 5^13, per pass.
            let mut remaining = scale;
            while remaining > 0 {
                let step = remaining.min(13);
                coefficient = coefficient.multiplied_by(5_u32.pow(step as u32));
                remaining -= step;
            }
            let digits = coefficient.digits;
            // The fraction keeps the scale in its capacity and drops its trailing zeroes.
            let mut fraction = String::with_capacity(scale);
            let integer = if digits.len() > scale {
                let split = digits.len() - scale;
                fraction.push_str(digits[split..].trim_end_matches('0'));
                HtmlNonNegativeInteger {
                    digits: digits[..split].to_owned(),
                }
            } else {
                fraction.extend(std::iter::repeat_n('0', scale - digits.len()));
                fraction.push_str(digits.trim_end_matches('0'));
                HtmlNonNegativeInteger::from(0)
            };
            HtmlNonNegativeDecimal { integer, fraction }
        };
        HtmlSignedDecimal::from_magnitude(magnitude, bits >> 63 != 0)
    }

    /// Returns the finite binary64 value.
    pub fn value(&self) -> f64 {
        self.value
    }

    /// Returns this value as plain decimal digits in valid HTML author syntax.
    ///
    /// This is Rust's shortest round-trip decimal expansion, which never uses an
    /// exponent: `1e21` is a 1 followed by twenty-one zeros, and the smallest
    /// subnormal has over three hundred fraction digits. The output has no
    /// work bound beyond the value's own expansion and always parses back to
    /// the same value. Native attribute construction uses this spelling: the
    /// input `min`, `max` and `step` attributes, native `progress` values and
    /// `srcset` densities.
    ///
    /// It is not the ECMAScript `ToString` operation that HTML's value
    /// sanitization algorithm calls "best representation". That serialization
    /// is `bounded::HtmlBestFiniteNumberCursor`, which the parser and DOM input
    /// range sanitization use over caller-owned storage; the two spellings
    /// differ for large and small magnitudes (`1e21`).
    pub fn to_plain_decimal_string(&self) -> DocumentString {
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

    /// Returns valid author syntax with commas and no whitespace, spelling
    /// each value as [`HtmlFloatingPointNumber::to_plain_decimal_string`] does.
    pub fn to_plain_decimal_string(&self) -> DocumentString {
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

    pub(crate) fn from_parts(integer: HtmlNonNegativeInteger, fraction: &[u16]) -> Self {
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
        let (fraction, carry) =
            fraction_sum_string(self.fraction.as_bytes(), other.fraction.as_bytes());
        let digits = sum_string(
            self.integer.digits.as_bytes(),
            other.integer.digits.as_bytes(),
            carry,
            capacity_hint(self.integer.digits.len().max(other.integer.digits.len()), 1),
        );
        Self {
            integer: HtmlNonNegativeInteger { digits },
            fraction,
        }
    }

    /// Multiplies an exact decimal quantity by a bounded nonnegative integer.
    pub(crate) fn multiplied_by(&self, multiplier: u32) -> Self {
        if multiplier == 0 || self.is_zero() {
            return HtmlNonNegativeInteger::from(0).into();
        }
        let (fraction, carry) = fraction_product_string(self.fraction.as_bytes(), multiplier);
        let digits = product_string(
            self.integer.digits.as_bytes(),
            multiplier,
            carry,
            capacity_hint(self.integer.digits.len(), 10),
        );
        Self {
            integer: HtmlNonNegativeInteger { digits },
            fraction,
        }
    }

    // Leading fractional zeroes retain scale; removing trailing zeroes is lossless.
    /// A decimal from a fraction buffer that a metered operation reserved.
    fn from_metered_fraction(
        integer: HtmlNonNegativeInteger,
        mut fraction: Vec<u8>,
    ) -> Result<Self, HtmlDecimalDigitsError> {
        trim_trailing_zeroes(&mut fraction);
        Ok(Self {
            integer,
            fraction: ascii_digit_string(fraction)?,
        })
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
            let mut text = String::with_capacity(capacity_hint(
                self.integer.digits.len(),
                self.fraction.len().saturating_add(1),
            ));
            text.push_str(&self.integer.digits);
            text.push('.');
            text.push_str(&self.fraction);
            text
        }
    }
}

/// An exact sum of nonnegative decimal quantities, each scaled by a bounded factor.
///
/// The sum keeps one unnormalized column per decimal place, so adding a quantity visits only
/// that quantity's own digits, whatever the digits of the running total. Carries are resolved
/// when the sum is read, and also, to keep every column below `u64::MAX`, after at most
/// `MAX_PENDING` additions. Native storage uses ordinary allocations.
#[derive(Debug, Default)]
pub(crate) struct HtmlDecimalColumns {
    /// Integer places, least significant first.
    integer: Vec<u64>,
    /// Fractional places, tenths first.
    fraction: Vec<u64>,
    /// Additions since the columns were last normalized.
    pending: u64,
}

impl HtmlDecimalColumns {
    /// One addition raises a column by at most `9 * u32::MAX`, below 2^36; with this many
    /// additions and the carry into the next place a column stays below 2^64.
    const MAX_PENDING: u64 = 1 << 27;

    /// Adds `integer.fraction * factor`, validating both digit runs before changing the sum.
    /// The integer digits must be nonempty; the fraction digits may be empty.
    pub(crate) fn add_scaled(
        &mut self,
        integer: &[u16],
        fraction: &[u16],
        factor: u32,
    ) -> Result<(), HtmlDecimalDigitsError> {
        check_decimal_digits(integer)?;
        if !fraction.is_empty() {
            check_decimal_digits(fraction)?;
        }
        if self.pending == Self::MAX_PENDING {
            self.normalize();
        }
        if self.integer.len() < integer.len() {
            self.integer.resize(integer.len(), 0);
        }
        if self.fraction.len() < fraction.len() {
            self.fraction.resize(fraction.len(), 0);
        }
        let factor = u64::from(factor);
        for (column, &unit) in self.integer.iter_mut().zip(integer.iter().rev()) {
            *column += u64::from(unit.wrapping_sub(u16::from(b'0'))) * factor;
        }
        for (column, &unit) in self.fraction.iter_mut().zip(fraction) {
            *column += u64::from(unit.wrapping_sub(u16::from(b'0'))) * factor;
        }
        self.pending += 1;
        #[cfg(test)]
        count_digit_steps(2 * (integer.len() + fraction.len()));
        Ok(())
    }

    /// Resolves every carry, leaving each column a single digit.
    fn normalize(&mut self) {
        let mut carry = 0_u64;
        for column in self
            .fraction
            .iter_mut()
            .rev()
            .chain(self.integer.iter_mut())
        {
            let total = *column + carry;
            *column = total % 10;
            carry = total / 10;
        }
        while carry != 0 {
            self.integer.push(carry % 10);
            carry /= 10;
        }
        self.pending = 0;
        #[cfg(test)]
        count_digit_steps(self.fraction.len() + self.integer.len());
    }

    /// The canonical exact value: no leading integer zeroes, no trailing fractional zeroes.
    pub(crate) fn finish(mut self) -> HtmlNonNegativeDecimal {
        self.normalize();
        let integer_len = self
            .integer
            .iter()
            .rposition(|&d| d != 0)
            .map_or(0, |i| i + 1);
        let fraction_len = self
            .fraction
            .iter()
            .rposition(|&d| d != 0)
            .map_or(0, |i| i + 1);
        let digit = |&place: &u64| char::from(b'0'.wrapping_add(place as u8));
        let mut digits = String::with_capacity(integer_len.max(1));
        if integer_len == 0 {
            digits.push('0');
        } else {
            digits.extend(self.integer.iter().take(integer_len).rev().map(digit));
        }
        let fraction: String = self.fraction.iter().take(fraction_len).map(digit).collect();
        #[cfg(test)]
        count_digit_steps(integer_len + fraction_len);
        HtmlNonNegativeDecimal {
            integer: HtmlNonNegativeInteger { digits },
            fraction,
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
            result.insert(0, '-');
        }
        result.into()
    }

    pub(crate) fn added_to(&self, other: &Self) -> Self {
        if self.negative == other.negative {
            return Self::from_magnitude(self.magnitude.added_to(&other.magnitude), self.negative);
        }
        match Order::of(&self.magnitude, &other.magnitude) {
            Order::SecondLarger(gap) => Self::from_magnitude(gap.difference(), other.negative),
            Order::Equal => Self::from(HtmlInteger::from(0)),
            Order::FirstLarger(gap) => Self::from_magnitude(gap.difference(), self.negative),
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
#[non_exhaustive]
pub enum HtmlNumericParseError {
    /// The value does not match the HTML grammar the entry point implements: author mode rejects
    /// what recovery would skip or truncate, and a value with no digits is always invalid (HTML877d
    /// #numbers).
    InvalidSyntax,
    /// A minus sign precedes a nonzero value where a non-negative integer is required (HTML877d
    /// #non-negative-integers). `-0` is accepted as zero.
    Negative,
    /// The value is grammatical but too large to convert to a finite binary64 number (HTML877d
    /// #floating-point-numbers), which differs from invalid syntax.
    FloatingPointOverflow,
    /// A percentage, absolute or relative length is zero where a nonzero one is required (HTML877d
    /// #nonzero-percentages-and-lengths).
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
pub(crate) fn validate_integer(
    input: &DocumentString,
) -> Result<HtmlInteger, HtmlNumericParseError> {
    integer(input.code_units(), false)
}

/// §2.3.4.1 recovery consumes only the initial signed digit sequence.
pub(crate) fn parse_integer(input: &DocumentString) -> Result<HtmlInteger, HtmlNumericParseError> {
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
pub struct HtmlIntegerPrefixScanner {
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
/// The result of scanning an integer prefix: its sign and the range of its digits within the units
/// that were fed. The range indexes the caller's input, so reading the digits back needs that same
/// input.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct HtmlScannedIntegerPrefix {
    pub(crate) negative: bool,
    pub(crate) start: usize,
    pub(crate) end: usize,
}
impl HtmlIntegerPrefixScanner {
    /// Starts a scan. With `recover` false it follows the author grammar: an optional `-`, then
    /// ASCII digits to the end of input. With `recover` true it follows the recovery algorithm of
    /// HTML877d #signed-integers: leading ASCII whitespace is skipped, `+` or `-` may precede the
    /// digits, and the prefix of digits ends at the first other unit.
    pub fn new(recover: bool) -> Self {
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
    pub fn feed(&mut self, unit: u16) -> bool {
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
    /// Ends the scan after the last unit has been fed. It returns the sign and digit range when at
    /// least one digit was accepted and the grammar was not violated, and `InvalidSyntax`
    /// otherwise.
    pub fn finish(self) -> Result<HtmlScannedIntegerPrefix, HtmlNumericParseError> {
        if matches!(
            self.phase,
            IntegerPrefixPhase::Digits | IntegerPrefixPhase::Done
        ) {
            Ok(HtmlScannedIntegerPrefix {
                negative: self.negative,
                start: self.start,
                end: self.end,
            })
        } else {
            Err(HtmlNumericParseError::InvalidSyntax)
        }
    }
}

fn nonnegative_prefix_digits(
    negative: bool,
    digits: &[u16],
) -> Result<&[u16], HtmlNumericParseError> {
    if negative && digits.iter().any(|&unit| unit != b'0' as u16) {
        Err(HtmlNumericParseError::Negative)
    } else {
        Ok(digits)
    }
}

fn integer_prefix(
    units: &[u16],
    recover: bool,
) -> Result<IntegerPrefix<'_>, HtmlNumericParseError> {
    let mut scanner = HtmlIntegerPrefixScanner::new(recover);
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

fn integer(units: &[u16], recover: bool) -> Result<HtmlInteger, HtmlNumericParseError> {
    let prefix = integer_prefix(units, recover)?;
    Ok(HtmlInteger::from_magnitude(
        HtmlNonNegativeInteger::from_digits(prefix.digits),
        prefix.negative,
    ))
}

/// Recovery rejection and allocation admission are separate source outcomes.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum HtmlIntegerRecoveryError {
    /// The input is not an acceptable integer prefix; no allocation was attempted.
    Syntax(HtmlNumericParseError),
    /// The input is syntactically acceptable but the canonical digit allocation was refused or
    /// failed.
    Allocation(HtmlIntegerUnitStepError),
}

/// §2.3.4.1 recovery with one fallible canonical digit allocation.
///
/// The caller admits raw UTF-16 units, precharges a three-input-length envelope
/// plus fixed work, and owns cancellation. This atomic helper scans the prefix,
/// scans its leading zeroes, and writes at most the input length. It performs no
/// intermediate integer construction, formatting, or unchecked string clone.
pub fn try_parse_integer(
    input: &DocumentString,
    max_result_digits: usize,
    max_capacity_bytes: usize,
) -> Result<HtmlIntegerUnitStepResult, HtmlIntegerRecoveryError> {
    let prefix = integer_prefix(input.code_units(), true)?;
    // The scanner proves at least one ASCII digit. All-zero input retains its
    // last zero, so canonical zero also requires exactly one output digit.
    let first = prefix
        .digits
        .iter()
        .position(|&unit| unit != b'0' as u16)
        .unwrap_or(prefix.digits.len() - 1);
    let canonical = &prefix.digits[first..];
    let mut digits =
        reserve_integer_digits(canonical.len(), max_result_digits, max_capacity_bytes)?;
    for &unit in canonical {
        digits.push(char::from(unit as u8));
    }
    let capacity_bytes = digits.capacity();
    Ok(HtmlIntegerUnitStepResult {
        value: HtmlInteger::from_magnitude(HtmlNonNegativeInteger { digits }, prefix.negative),
        capacity_bytes,
    })
}

/// Fixed signed-long projection of the shared integer recovery prefix.
/// `None` is a successfully parsed mathematical integer outside i32, not syntax
/// failure. No canonical digit allocation or arbitrary integer digit cap.
pub fn parse_integer_long(input: &DocumentString) -> Result<Option<i32>, HtmlNumericParseError> {
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
pub fn integer_long_work_bound(input_units: usize) -> Option<usize> {
    input_units.checked_mul(8)?.checked_add(16)
}

/// §2.3.4.2 recovery projected to `min(exact_value, cap)` without allocation.
///
/// Negative nonzero input is rejected before projection, including when the cap
/// is zero. The caller admits and precharges raw input inspection. For n input
/// units the operation makes at most n prefix-scan steps, at most n - 1 steps
/// checking a negative prefix for nonzero digits, and at most n - 1 projection
/// steps, so 3n steps suffice; the fixed-width arithmetic cannot overflow.
pub fn parse_nonnegative_integer_capped(
    input: &DocumentString,
    cap: usize,
) -> Result<usize, HtmlNumericParseError> {
    parse_nonnegative_integer_capped_steps(input.code_units(), cap, &mut || {})
}

/// The capped recovery with each unit of work reported to `step` just before
/// it is performed: one per unit fed to the prefix scanner, one per digit of a
/// negative prefix checked for a nonzero digit, and one per digit projected.
pub(crate) fn parse_nonnegative_integer_capped_steps(
    units: &[u16],
    cap: usize,
    step: &mut impl FnMut(),
) -> Result<usize, HtmlNumericParseError> {
    match try_parse_nonnegative_integer_capped_steps(units, cap, &mut || {
        step();
        Ok::<_, std::convert::Infallible>(())
    }) {
        Ok(result) => result,
        Err(never) => match never {},
    }
}

/// Caller refusal is distinct from an ordinary grammar rejection. Each unit
/// remains unread until its corresponding input/check/projection step admits.
pub(crate) fn try_parse_nonnegative_integer_capped_steps<E>(
    units: &[u16],
    cap: usize,
    step: &mut impl FnMut() -> Result<(), E>,
) -> Result<Result<usize, HtmlNumericParseError>, E> {
    let mut scanner = HtmlIntegerPrefixScanner::new(true);
    for unit in units {
        step()?;
        if !scanner.feed(*unit) {
            break;
        }
    }
    let prefix = match scanner.finish() {
        Ok(prefix) => prefix,
        Err(error) => return Ok(Err(error)),
    };
    let digits = &units[prefix.start..prefix.end];
    if prefix.negative {
        for unit in digits {
            step()?;
            if *unit != b'0' as u16 {
                return Ok(Err(HtmlNumericParseError::Negative));
            }
        }
    }
    let mut value = 0_usize;
    for unit in digits {
        step()?;
        let digit = usize::from(*unit - b'0' as u16);
        value = value
            .checked_mul(10)
            .and_then(|value| value.checked_add(digit))
            .map_or(cap, |value| value.min(cap));
        if value == cap {
            return Ok(Ok(cap));
        }
    }
    Ok(Ok(value))
}

/// Author grammar: one or more ASCII digits, without either sign.
pub(crate) fn validate_nonnegative_integer(
    input: &DocumentString,
) -> Result<HtmlNonNegativeInteger, HtmlNumericParseError> {
    let units = input.code_units();
    if units.is_empty() || !units.iter().copied().all(is_digit) {
        return Err(HtmlNumericParseError::InvalidSyntax);
    }
    Ok(HtmlNonNegativeInteger::from_digits(units))
}

/// §2.3.4.2 recovery rejects negative values, while accepting recovered -0.
pub(crate) fn parse_nonnegative_integer(
    input: &DocumentString,
) -> Result<HtmlNonNegativeInteger, HtmlNumericParseError> {
    let prefix = integer_prefix(input.code_units(), true)?;
    Ok(HtmlNonNegativeInteger::from_digits(
        nonnegative_prefix_digits(prefix.negative, prefix.digits)?,
    ))
}

/// Author grammar, followed by the binary64 conversion contract.
/// Grammatical overflow is distinct from invalid author syntax.
pub(crate) fn validate_floating_point_number(
    input: &DocumentString,
) -> Result<HtmlFloatingPointNumber, HtmlNumericParseError> {
    floating_point(input.code_units(), false)
}

/// §2.3.4.3 recovery, including ignored leading plus and incomplete exponents.
pub(crate) fn parse_floating_point_number(
    input: &DocumentString,
) -> Result<HtmlFloatingPointNumber, HtmlNumericParseError> {
    floating_point(input.code_units(), true)
}

/// The source's author grammar and user-agent recovery remain distinct modes.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum HtmlFloatingPointParseMode {
    /// Follows the author grammar: no leading whitespace, no `+`, and no trailing garbage.
    Author,
    /// Follows the user-agent recovery rules of HTML877d #floating-point-numbers: leading
    /// whitespace and `+` are skipped and an incomplete exponent or trailing text ends the number.
    Recovery,
}

/// The conversion outcome and capacity of its already-dropped temporary token.
/// Numeric rejection after allocation must still be accounted by the caller.
#[derive(Debug)]
pub struct HtmlFloatingPointParseResult {
    pub(crate) value: Result<HtmlFloatingPointNumber, HtmlNumericParseError>,
    pub(crate) capacity_bytes: usize,
}

/// Allocation admission failures, distinct from source numeric rejection.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum HtmlFloatingPointAllocationError {
    /// The token length overflows or exceeds `isize::MAX`, so no allocation could represent it.
    RepresentationExhausted,
    /// The bytes the token needs exceed the caller's byte limit before allocation.
    RequestedCapacityLimitExceeded {
        /// The caller's byte limit.
        limit: usize,
        /// The bytes the operation needs in total, counting earlier allocations of the same operation.
        required: usize,
    },
    /// The allocator granted more capacity than the caller's byte limit although the request fit.
    ActualCapacityLimitExceeded {
        /// The caller's byte limit.
        limit: usize,
        /// The bytes that were requested from the allocator.
        requested: usize,
        /// The capacity in bytes the allocator actually granted.
        capacity_bytes: usize,
    },
    /// The allocator could not provide the requested bytes.
    AllocationFailed {
        /// The bytes that were requested from the allocator.
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
    /// The canonical key of the mathematical value one.
    pub(crate) fn one() -> Self {
        Self {
            negative: false,
            significand: HtmlNonNegativeInteger {
                digits: String::from("1"),
            },
            exponent: HtmlInteger::from(0),
        }
    }

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
        match (self.is_zero(), other.is_zero()) {
            (true, true) => return Ok(Ordering::Equal),
            (true, false) => {
                return Ok(if other.negative {
                    Ordering::Greater
                } else {
                    Ordering::Less
                });
            }
            (false, true) => {
                return Ok(if self.negative {
                    Ordering::Less
                } else {
                    Ordering::Greater
                });
            }
            (false, false) => {}
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
    Syntax(HtmlNumericParseError),
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
    let prefix = floating_point_prefix(input, HtmlFloatingPointParseMode::Author)
        .map_err(ScientificDecimalKeyError::Syntax)?;
    scientific_decimal_key(&prefix)
}

/// The source's finite binary64 rounding of a checked decimal.
#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) enum HtmlBinary64Rounding {
    /// The decimal rounds to this finite value; underflow rounds to zero.
    Finite(HtmlFloatingPointNumber),
    /// The decimal is too large for a finite binary64 value.
    Overflow,
}

/// A decimal that passed the complete author grammar, with its exact key and its binary64
/// rounding. Both come from one scan of the same checked token.
#[derive(Clone, Debug)]
pub(crate) struct HtmlStrictDecimal {
    pub(crate) key: HtmlScientificDecimalKey,
    pub(crate) rounding: HtmlBinary64Rounding,
}

/// Checks the complete author grammar once, then derives the exact key and the binary64
/// rounding from the same checked token.
pub(crate) fn strict_decimal_units(
    input: &[u16],
) -> Result<HtmlStrictDecimal, ScientificDecimalKeyError> {
    let prefix = floating_point_prefix(input, HtmlFloatingPointParseMode::Author)
        .map_err(ScientificDecimalKeyError::Syntax)?;
    let key = scientific_decimal_key(&prefix)?;
    let token: String = prefix
        .token
        .iter()
        .map(|&unit| char::from(unit as u8))
        .collect();
    let rounding = match convert_floating_point_token(&token) {
        Ok(value) => HtmlBinary64Rounding::Finite(value),
        Err(HtmlNumericParseError::FloatingPointOverflow) => HtmlBinary64Rounding::Overflow,
        Err(error) => return Err(ScientificDecimalKeyError::Syntax(error)),
    };
    Ok(HtmlStrictDecimal { key, rounding })
}

fn scientific_decimal_key(
    prefix: &FloatingPointPrefix<'_>,
) -> Result<HtmlScientificDecimalKey, ScientificDecimalKeyError> {
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
    let exponent = prefix.exponent.as_ref().map_or_else(
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
    Ok(match Order::of(&exponent.magnitude, &adjustment) {
        Order::SecondLarger(gap) => HtmlInteger::from_magnitude(gap.difference(), negative),
        Order::Equal => HtmlInteger::from(0),
        Order::FirstLarger(gap) => HtmlInteger::from_magnitude(gap.difference(), exponent.negative),
    })
}

fn floating_point(
    units: &[u16],
    recover: bool,
) -> Result<HtmlFloatingPointNumber, HtmlNumericParseError> {
    let mode = if recover {
        HtmlFloatingPointParseMode::Recovery
    } else {
        HtmlFloatingPointParseMode::Author
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
pub struct HtmlFloatingPointGrammarCursor<'a> {
    units: &'a [u16],
    mode: HtmlFloatingPointParseMode,
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

impl<'a> HtmlFloatingPointGrammarCursor<'a> {
    /// Creates a cursor at the start of `units`, the exact UTF-16 code units of the value, to be
    /// read under `mode`. It does no work.
    pub fn new(units: &'a [u16], mode: HtmlFloatingPointParseMode) -> Self {
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

    /// Whether the grammar run is over, either accepting or rejecting the input.
    pub fn is_finished(&self) -> bool {
        self.step == FloatingGrammarStep::Done
    }

    /// Whether the run is over and accepted the input. It is false while the run is in progress and
    /// after a rejection.
    pub fn is_valid(&self) -> bool {
        self.is_finished() && self.valid
    }

    /// Whether the next `advance` reads a source unit. A driver charges the read only when this is
    /// true; some actions only change state.
    pub fn inspects_unit(&self) -> bool {
        self.position < self.units.len()
            && match self.step {
                FloatingGrammarStep::Whitespace => {
                    self.mode == HtmlFloatingPointParseMode::Recovery
                }
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

    /// Performs one fixed grammar action: it reads at most one source unit and never allocates or
    /// converts. Call it until `is_finished`; calling it afterward has no effect.
    pub fn advance(&mut self) {
        use FloatingGrammarStep::*;
        match self.step {
            Whitespace => {
                if self.mode == HtmlFloatingPointParseMode::Recovery
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
                    || (self.mode == HtmlFloatingPointParseMode::Recovery
                        && unit == Some(b'+' as u16))
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
                        && (self.mode == HtmlFloatingPointParseMode::Author
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
                    if self.mode == HtmlFloatingPointParseMode::Author {
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
                self.valid = self.mode == HtmlFloatingPointParseMode::Recovery
                    || self.position == self.units.len();
                self.step = Done;
            }
            Done => {}
        }
    }

    /// Issued only by the completed shared grammar. The numeric executor owns
    /// offsets and signs; its adapter retains and qualifies the selected source.
    pub fn range_decimal_parts(
        &self,
    ) -> Result<range::HtmlRangeDecimalParts, HtmlNumericParseError> {
        if !self.is_valid() {
            return Err(HtmlNumericParseError::InvalidSyntax);
        }
        Ok(range::HtmlRangeDecimalParts {
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

    fn prefix(&self) -> Result<FloatingPointPrefix<'a>, HtmlNumericParseError> {
        if !self.is_valid() {
            return Err(HtmlNumericParseError::InvalidSyntax);
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
    mode: HtmlFloatingPointParseMode,
) -> Result<FloatingPointPrefix<'_>, HtmlNumericParseError> {
    let mut cursor = HtmlFloatingPointGrammarCursor::new(units, mode);
    while !cursor.is_finished() {
        cursor.advance();
    }
    cursor.prefix()
}

fn convert_floating_point_token(
    token: &str,
) -> Result<HtmlFloatingPointNumber, HtmlNumericParseError> {
    let value = token
        .parse::<f64>()
        .map_err(|_| HtmlNumericParseError::InvalidSyntax)?;
    HtmlFloatingPointNumber::try_from(value)
        .map_err(|_| HtmlNumericParseError::FloatingPointOverflow)
}

fn check_floating_point_requested_capacity(
    required: usize,
    max_capacity_bytes: usize,
) -> Result<(), HtmlFloatingPointAllocationError> {
    if required > isize::MAX as usize {
        return Err(HtmlFloatingPointAllocationError::RepresentationExhausted);
    }
    if required > max_capacity_bytes {
        return Err(
            HtmlFloatingPointAllocationError::RequestedCapacityLimitExceeded {
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
) -> Result<(), HtmlFloatingPointAllocationError> {
    if capacity_bytes > max_capacity_bytes {
        Err(
            HtmlFloatingPointAllocationError::ActualCapacityLimitExceeded {
                limit: max_capacity_bytes,
                requested,
                capacity_bytes,
            },
        )
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
pub fn try_floating_point_number(
    input: &DocumentString,
    mode: HtmlFloatingPointParseMode,
    max_capacity_bytes: usize,
) -> Result<HtmlFloatingPointParseResult, HtmlFloatingPointAllocationError> {
    let prefix = match floating_point_prefix(input.code_units(), mode) {
        Ok(prefix) => prefix,
        Err(error) => {
            return Ok(HtmlFloatingPointParseResult {
                value: Err(error),
                capacity_bytes: 0,
            });
        }
    };
    let required = prefix.token.len();
    check_floating_point_requested_capacity(required, max_capacity_bytes)?;
    let mut token = String::new();
    token.try_reserve_exact(required).map_err(|_| {
        HtmlFloatingPointAllocationError::AllocationFailed {
            requested: required,
        }
    })?;
    check_floating_point_actual_capacity(required, token.capacity(), max_capacity_bytes)?;
    for &unit in prefix.token {
        token.push(char::from(unit as u8));
    }
    Ok(HtmlFloatingPointParseResult {
        value: convert_floating_point_token(&token),
        capacity_bytes: token.capacity(),
    })
}

/// Author list grammar is comma-separated author numbers, without whitespace.
pub(crate) fn validate_floating_point_list(
    input: &DocumentString,
) -> Result<HtmlFloatingPointList, HtmlNumericParseError> {
    if input.is_empty() {
        return Ok(HtmlFloatingPointList::default());
    }
    let mut values = Vec::new();
    let mut overflow = false;
    for units in input.code_units().split(|&unit| unit == b',' as u16) {
        match floating_point(units, false) {
            Ok(value) => values.push(value),
            Err(HtmlNumericParseError::FloatingPointOverflow) => overflow = true,
            Err(error) => return Err(error),
        }
    }
    // Overflow describes a representational failure only when the complete
    // list satisfies author grammar. A later invalid token takes precedence.
    if overflow {
        Err(HtmlNumericParseError::FloatingPointOverflow)
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
pub(crate) fn parse_dimension(
    input: &DocumentString,
) -> Result<HtmlDimension, HtmlNumericParseError> {
    let units = input.code_units();
    let mut position = 0;
    skip_whitespace(units, &mut position);
    let start = position;
    collect_digits(units, &mut position);
    if start == position {
        return Err(HtmlNumericParseError::InvalidSyntax);
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
) -> Result<HtmlDimension, HtmlNumericParseError> {
    let dimension = parse_dimension(input)?;
    if dimension.value().is_zero() {
        Err(HtmlNumericParseError::ZeroDimension)
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

// Public readback never permits callers to create canonical digit storage.
impl<T> HtmlNumericMaterialized<T> {
    /// The materialized value, which can only have been built by the owner's canonical
    /// constructors.
    pub fn value(&self) -> &T {
        &self.value
    }
    /// The cumulative capacity in bytes of every variable allocation the operation made, including
    /// intermediates that were dropped.
    pub const fn capacity_bytes(&self) -> usize {
        self.capacity_bytes
    }
    /// Splits the result into the value and the cumulative capacity in bytes.
    pub fn into_parts(self) -> (T, usize) {
        (self.value, self.capacity_bytes)
    }
}
impl HtmlNumericMaterializationFailure {
    /// The cause of the failure.
    pub const fn kind(&self) -> HtmlNumericMaterializationError {
        self.kind
    }
    /// The cumulative capacity in bytes of every allocation observed before the failure, which the
    /// caller must still account for.
    pub const fn capacity_bytes(&self) -> usize {
        self.capacity_bytes
    }
}
impl HtmlIntegerUnitStepResult {
    /// The canonical result of the step.
    pub fn value(&self) -> &HtmlInteger {
        &self.value
    }
    /// The capacity in bytes of the sole digit allocation the step made.
    pub const fn capacity_bytes(&self) -> usize {
        self.capacity_bytes
    }
    /// Splits the result into the integer and its allocation capacity in bytes.
    pub fn into_parts(self) -> (HtmlInteger, usize) {
        (self.value, self.capacity_bytes)
    }
}
impl HtmlFloatingPointParseResult {
    /// The conversion outcome: the number, or the numeric rejection. A rejection can follow an
    /// allocation, whose capacity is reported separately.
    pub fn value(&self) -> &Result<HtmlFloatingPointNumber, HtmlNumericParseError> {
        &self.value
    }
    /// The capacity in bytes of the temporary token the conversion allocated, or zero when syntax
    /// was rejected before allocating.
    pub const fn capacity_bytes(&self) -> usize {
        self.capacity_bytes
    }
    /// Splits the result into the conversion outcome and the token capacity in bytes.
    pub fn into_parts(
        self,
    ) -> (
        Result<HtmlFloatingPointNumber, HtmlNumericParseError>,
        usize,
    ) {
        (self.value, self.capacity_bytes)
    }
}
impl HtmlScannedIntegerPrefix {
    /// Whether a `-` sign preceded the digits. A negative zero prefix is still reported negative
    /// here; the value types normalize it.
    pub const fn is_negative(&self) -> bool {
        self.negative
    }
    /// The offset of the first digit, in UTF-16 code units from the start of the fed input.
    pub const fn start(&self) -> usize {
        self.start
    }
    /// The offset one past the last digit, in UTF-16 code units from the start of the fed input.
    pub const fn end(&self) -> usize {
        self.end
    }
    /// Checks a separately supplied input before returning its nonnegative prefix.
    /// Admit at most two visits per prefix unit, plus fixed range/sign work.
    pub fn try_nonnegative_digits(self, units: &[u16]) -> Result<&[u16], HtmlNumericParseError> {
        let digits = units
            .get(self.start..self.end)
            .ok_or(HtmlNumericParseError::InvalidSyntax)?;
        check_decimal_digits(digits).map_err(|_| HtmlNumericParseError::InvalidSyntax)?;
        nonnegative_prefix_digits(self.negative, digits)
    }
}

impl fmt::Display for HtmlNumericParseError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(match self {
            Self::InvalidSyntax => "value does not match the HTML numeric syntax",
            Self::Negative => "value is negative where a nonnegative value is required",
            Self::FloatingPointOverflow => "value overflows the finite binary64 range",
            Self::ZeroDimension => "dimension is zero where a nonzero value is required",
        })
    }
}
impl Error for HtmlNumericParseError {}

impl fmt::Display for HtmlIntegerUnitStepError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::RepresentationExhausted => {
                f.write_str("integer digit count cannot be represented on this platform")
            }
            Self::DigitLimitExceeded { limit, required } => write!(
                f,
                "integer needs {required} digits, exceeding the limit of {limit}"
            ),
            Self::RequestedCapacityLimitExceeded { limit, required } => write!(
                f,
                "integer needs {required} bytes of digit storage, exceeding the limit of {limit}"
            ),
            Self::ActualCapacityLimitExceeded {
                limit,
                requested,
                capacity_bytes,
            } => write!(
                f,
                "allocating {requested} bytes of digit storage produced {capacity_bytes} bytes, exceeding the limit of {limit}"
            ),
            Self::AllocationFailed { requested } => {
                write!(f, "allocating {requested} bytes of digit storage failed")
            }
        }
    }
}
impl Error for HtmlIntegerUnitStepError {}

impl fmt::Display for HtmlIntegerRecoveryError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(match self {
            Self::Syntax(_) => "integer recovery rejected the input",
            Self::Allocation(_) => "integer recovery could not store its digits",
        })
    }
}
impl From<HtmlNumericParseError> for HtmlIntegerRecoveryError {
    fn from(error: HtmlNumericParseError) -> Self {
        Self::Syntax(error)
    }
}
impl From<HtmlIntegerUnitStepError> for HtmlIntegerRecoveryError {
    fn from(error: HtmlIntegerUnitStepError) -> Self {
        Self::Allocation(error)
    }
}
impl Error for HtmlIntegerRecoveryError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::Syntax(error) => Some(error),
            Self::Allocation(error) => Some(error),
        }
    }
}

impl fmt::Display for HtmlFloatingPointAllocationError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::RepresentationExhausted => {
                f.write_str("floating-point token length cannot be represented on this platform")
            }
            Self::RequestedCapacityLimitExceeded { limit, required } => write!(
                f,
                "floating-point token needs {required} bytes of storage, exceeding the limit of {limit}"
            ),
            Self::ActualCapacityLimitExceeded {
                limit,
                requested,
                capacity_bytes,
            } => write!(
                f,
                "allocating {requested} bytes of floating-point token storage produced {capacity_bytes} bytes, exceeding the limit of {limit}"
            ),
            Self::AllocationFailed { requested } => write!(
                f,
                "allocating {requested} bytes of floating-point token storage failed"
            ),
        }
    }
}
impl Error for HtmlFloatingPointAllocationError {}

impl fmt::Display for HtmlNumericMaterializationError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(match self {
            Self::Digits(_) => "numeric digits are not a valid decimal magnitude",
            Self::Boundary(_) => "numeric materialization reached an allocation boundary",
        })
    }
}
impl Error for HtmlNumericMaterializationError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::Digits(error) => Some(error),
            Self::Boundary(error) => Some(error),
        }
    }
}

impl fmt::Display for HtmlNumericMaterializationFailure {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "numeric materialization failed after allocating {} bytes",
            self.capacity_bytes
        )
    }
}
impl Error for HtmlNumericMaterializationFailure {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        Some(&self.kind)
    }
}

#[cfg(test)]
mod tests {
    /// The column sums must equal the exact decimal arithmetic that they replace: each
    /// quantity multiplied by its factor and added to a running total.
    #[test]
    fn decimal_columns_equal_exact_scaled_addition_across_carries_and_normalization() {
        let mut state = 0x2545_f491_4f6c_dd1d_u64;
        let mut next = move |bound: u64| {
            state ^= state << 13;
            state ^= state >> 7;
            state ^= state << 17;
            state % bound
        };
        for round in 0..300 {
            let mut columns = HtmlDecimalColumns::default();
            let mut exact = HtmlNonNegativeDecimal::from(HtmlNonNegativeInteger::from(0_u128));
            for step in 0..(1 + next(12)) {
                let alphabet: &[u8] = if next(2) == 0 { b"09" } else { b"0123456789" };
                let mut run = |max: u64| -> Vec<u16> {
                    (0..next(max))
                        .map(|_| u16::from(alphabet[next(alphabet.len() as u64) as usize]))
                        .collect()
                };
                let mut integer = run(40);
                if integer.is_empty() {
                    integer.push(u16::from(b'0'));
                }
                let fraction = run(40);
                let factor = [1, 60, 3600, 86400, 604800, u32::MAX][next(6) as usize];
                columns.add_scaled(&integer, &fraction, factor).unwrap();
                let whole = HtmlNonNegativeInteger::from_decimal_digits(
                    &DocumentString::from_utf16(integer),
                )
                .unwrap();
                let quantity = if fraction.is_empty() {
                    HtmlNonNegativeDecimal::from(whole)
                } else {
                    HtmlNonNegativeDecimal::with_fraction_digits(
                        whole,
                        &DocumentString::from_utf16(fraction),
                    )
                    .unwrap()
                };
                exact = exact.added_to(&quantity.multiplied_by(factor));
                if (round + step) % 3 == 0 {
                    columns.normalize();
                }
            }
            assert_eq!(columns.finish(), exact, "round {round}");
        }
    }

    #[test]
    fn decimal_columns_reject_non_digits_without_changing_the_sum() {
        let mut columns = HtmlDecimalColumns::default();
        columns.add_scaled(&[u16::from(b'7')], &[], 2).unwrap();
        assert_eq!(
            columns.add_scaled(&[u16::from(b'1')], &[u16::from(b'x')], 1),
            Err(HtmlDecimalDigitsError::NonAsciiDigit {
                code_unit_offset: 0,
                code_unit: u16::from(b'x')
            })
        );
        assert_eq!(
            columns.add_scaled(&[], &[], 1),
            Err(HtmlDecimalDigitsError::Empty)
        );
        assert_eq!(
            columns.finish().as_document_string().to_utf8().unwrap(),
            "14"
        );
    }

    /// A strictly checked decimal rounds exactly as the author-mode number conversion does and
    /// carries the key of the strict key producer: both read one checked token.
    #[test]
    fn a_strict_decimal_carries_its_key_and_its_binary64_rounding() {
        for input in [
            "0",
            "1",
            "1.5",
            "0.25",
            "-3",
            "1e3",
            "1E-3",
            ".5",
            "000.0100",
            "123456789.987654321",
            "1e308",
            "1.7976931348623157e308",
            "4.9e-324",
            "1e-400",
        ] {
            let units = text(input);
            let decimal = strict_decimal_units(units.code_units()).unwrap();
            assert_eq!(
                decimal.key,
                strict_scientific_decimal_key(&units).unwrap(),
                "{input}: key"
            );
            match validate_floating_point_number(&units) {
                Ok(value) => assert_eq!(
                    decimal.rounding,
                    HtmlBinary64Rounding::Finite(value),
                    "{input}: rounding"
                ),
                Err(error) => panic!("{input} is finite: {error:?}"),
            }
        }
        // Too large for a finite value, yet a grammatical decimal with an exact key.
        let large = text("1e999");
        assert_eq!(
            validate_floating_point_number(&large),
            Err(HtmlNumericParseError::FloatingPointOverflow)
        );
        let decimal = strict_decimal_units(large.code_units()).unwrap();
        assert_eq!(decimal.rounding, HtmlBinary64Rounding::Overflow);
        assert_eq!(decimal.key, strict_scientific_decimal_key(&large).unwrap());
        for invalid in ["", "+1", " 1", "1 ", "1e", "e1", ".", "2.", "--1", "1x"] {
            assert!(
                matches!(
                    strict_decimal_units(text(invalid).code_units()),
                    Err(ScientificDecimalKeyError::Syntax(_))
                ),
                "{invalid:?}"
            );
        }
    }

    fn long_integer(text: &str) -> HtmlNonNegativeInteger {
        HtmlNonNegativeInteger::from_decimal_digits(&DocumentString::from(text)).unwrap()
    }

    /// The digits of an unmetered operation written most significant first match the metered
    /// operation, which is a separate implementation, across the block boundaries of the two-pass
    /// writer and through runs of nines that carry the whole way.
    #[test]
    fn unmetered_integer_arithmetic_matches_the_metered_operations_across_blocks() {
        let unlimited = (usize::MAX, usize::MAX);
        let operands = [
            "9".repeat(DIGIT_BLOCK - 1),
            "9".repeat(DIGIT_BLOCK),
            "9".repeat(DIGIT_BLOCK + 1),
            "9".repeat(3 * DIGIT_BLOCK + 7),
            format!("1{}", "0".repeat(2 * DIGIT_BLOCK)),
            format!("{}{}", "1".repeat(DIGIT_BLOCK), "9".repeat(DIGIT_BLOCK)),
            "123456789".repeat(100),
            "7".to_owned(),
            "1".to_owned(),
        ];
        for left in &operands {
            let left = long_integer(left);
            for multiplier in [1_u32, 2, 7, 10, 99_999, u32::MAX] {
                let metered = left
                    .try_multiplied_by(multiplier, unlimited.0, unlimited.1)
                    .unwrap();
                assert_eq!(left.multiplied_by(multiplier), metered.value);
            }
            for right in &operands {
                let right = long_integer(right);
                let metered = left.try_added_to(&right, unlimited.0, unlimited.1).unwrap();
                assert_eq!(left.added_to(&right), metered.value);
            }
            for divisor in [1_u32, 3, 7, 10, 999_999_937] {
                let divisor = NonZeroU32::new(divisor).unwrap();
                let metered = left
                    .try_quotient_remainder(divisor, unlimited.0, unlimited.1)
                    .unwrap();
                assert_eq!(left.quotient_remainder(divisor), metered.value);
            }
        }
    }

    /// The same for decimals, whose fraction carries into the whole part and keeps its scale.
    #[test]
    fn unmetered_decimal_arithmetic_matches_the_metered_operations_across_blocks() {
        let unlimited = (usize::MAX, usize::MAX);
        let decimal = |whole: &str, fraction: &str| {
            if fraction.is_empty() {
                return long_integer(whole).into();
            }
            HtmlNonNegativeDecimal::with_fraction_digits(
                long_integer(whole),
                &DocumentString::from(fraction),
            )
            .unwrap()
        };
        let operands = [
            decimal("0", &"9".repeat(DIGIT_BLOCK + 3)),
            decimal(&"9".repeat(DIGIT_BLOCK + 1), &"9".repeat(2 * DIGIT_BLOCK)),
            decimal("1", &format!("{}5", "0".repeat(DIGIT_BLOCK + 10))),
            decimal("12", "5"),
            decimal("0", "05"),
            decimal("42", ""),
        ];
        for left in &operands {
            for multiplier in [1_u32, 2, 3, 99_999, u32::MAX] {
                let metered = left
                    .try_multiplied_by(multiplier, unlimited.0, unlimited.1)
                    .unwrap();
                assert_eq!(left.multiplied_by(multiplier), metered.value);
            }
            for right in &operands {
                let metered = left.try_added_to(right, unlimited.0, unlimited.1).unwrap();
                assert_eq!(left.added_to(right), metered.value);
            }
        }
    }

    /// The writer keeps the places its kind keeps and returns the carry out of the last place.
    #[test]
    fn a_digit_string_keeps_integer_and_fraction_places_and_returns_the_final_state() {
        // 999 + 1 across three places: the carry runs through the nines and out.
        let (digits, carry) = digit_string(4, Keep::Integer, 0, 1_u8, |carry, place| {
            let sum = if place < 3 { 9 } else { 0 } + u64::from(carry);
            (Digit::units(sum), (sum / 10) as u8)
        });
        assert_eq!((digits.as_str(), carry), ("1000", 0));
        // Every place zero: an integer is a lone zero, a fraction is empty.
        let zeros = |_: (), _| (Digit::units(0), ());
        assert_eq!(digit_string(5, Keep::Integer, 0, (), zeros).0, "0");
        assert_eq!(digit_string(5, Keep::Fraction, 0, (), zeros).0, "");
        // Fractional places keep leading zeroes and drop trailing ones: 0.0500 -> "05".
        let fraction = digit_string(4, Keep::Fraction, 4, (), |(), place| {
            (Digit::units(if place == 2 { 5 } else { 0 }), ())
        });
        assert_eq!(fraction.0, "05");
        assert!(fraction.0.capacity() >= 4);
    }

    /// The predecessor of a positive integer against machine subtraction, and against the digits
    /// of a power of ten beyond any machine integer, whose predecessor is all nines.
    #[test]
    fn positive_predecessors_match_machine_subtraction() {
        let samples = [
            1_u128,
            2,
            9,
            10,
            11,
            99,
            100,
            101,
            999,
            1000,
            1001,
            10_u128.pow(30),
            10_u128.pow(30) + 1,
            u128::MAX,
        ];
        for value in samples {
            let positive = HtmlPositiveInteger::new(HtmlNonNegativeInteger::from(value)).unwrap();
            let predecessor = positive.predecessor();
            assert_eq!(
                predecessor.decimal_digits(),
                (value - 1).to_string(),
                "{value} - 1"
            );
            assert_eq!(
                predecessor.retained_buffer_bytes(),
                predecessor.digits.len()
            );
        }
        let big = HtmlNonNegativeInteger::from_decimal_digits(&DocumentString::from(format!(
            "1{}",
            "0".repeat(60)
        )))
        .unwrap();
        let predecessor = HtmlPositiveInteger::new(big).unwrap().predecessor();
        assert_eq!(predecessor.decimal_digits(), "9".repeat(60));
        let zero = HtmlNonNegativeInteger::from(0_u128);
        assert_eq!(HtmlPositiveInteger::new(zero.clone()), Err(zero));
    }

    /// 2^-1074 = 5^1074 / 10^1074 (binary64 subnormal minimum): 5^1074 has 751
    /// digits and ends in 5, so the expansion has exactly 1074 fractional digits.
    /// Multiplying by 5 once per power costs about 1074 * 400 digit visits plus a
    /// validation pass per call; chunks of 5^13 must cost an order of magnitude less.
    #[test]
    fn exact_decimal_of_the_smallest_subnormal_is_computed_in_chunked_passes() {
        let number = HtmlFloatingPointNumber::try_from(5e-324).unwrap();
        let (decimal, steps) = digit_steps_during(|| number.exact_decimal());
        let text = decimal.as_document_string().to_utf8().unwrap();
        let fraction = text.strip_prefix("0.").unwrap();
        assert_eq!(fraction.len(), 1074);
        assert!(fraction.starts_with(&format!(
            "{}494065645841246544176568792868221372365059802",
            "0".repeat(323)
        )));
        assert!(fraction.ends_with("19718265533447265625"));
        assert!(steps < 120_000, "{steps} digit visits");
    }

    #[test]
    fn retained_integer_bytes_observe_real_spare_buffer_without_changing_value_identity() {
        use std::hash::{Hash, Hasher};
        let mut digits = String::with_capacity(128);
        digits.push_str("123");
        let capacity = digits.capacity();
        let pointer = digits.as_ptr();
        assert!(capacity >= 128);
        let value = super::HtmlNonNegativeInteger { digits };
        let canonical = super::HtmlNonNegativeInteger::from(123);
        let hash = |value: &super::HtmlNonNegativeInteger| {
            let mut hasher = std::collections::hash_map::DefaultHasher::new();
            value.hash(&mut hasher);
            hasher.finish()
        };
        for _ in 0..3 {
            assert_eq!(value.retained_buffer_bytes(), capacity);
            assert!(value.retained_buffer_bytes() > value.decimal_digits().len());
            assert_eq!(value.decimal_digits(), "123");
            assert_eq!(value.decimal_digits().as_ptr(), pointer);
            assert_eq!(value, canonical);
            assert_eq!(hash(&value), hash(&canonical));
        }
    }

    #[test]
    fn retained_integer_bytes_include_arithmetic_spare_capacity_after_canonicalization() {
        let value = super::HtmlNonNegativeInteger::from_decimal_digits(&"000123".into())
            .unwrap()
            .multiplied_by(1);
        assert_eq!(value.decimal_digits(), "123");
        // The real owning arithmetic reserves input digits plus the u32 carry
        // envelope, then retains that allocation after canonical output.
        assert!(value.retained_buffer_bytes() >= 13);
        assert_eq!(value.retained_buffer_bytes(), value.digits.capacity());
        assert_eq!(value.to_u128(), Ok(123));
    }

    #[test]
    fn integer_unit_scanner_keeps_author_and_recovery_prefixes_distinct() {
        use super::{HtmlIntegerPrefixScanner, HtmlNumericParseError, HtmlScannedIntegerPrefix};
        for (raw, recover, expected) in [
            (" \t+0012suffix", true, Ok((false, 3, 7))),
            ("-000tail", true, Ok((true, 1, 4))),
            ("12tail", false, Err(HtmlNumericParseError::InvalidSyntax)),
            ("+12", false, Err(HtmlNumericParseError::InvalidSyntax)),
            ("-12", false, Ok((true, 1, 3))),
            ("-", true, Err(HtmlNumericParseError::InvalidSyntax)),
            (" \t", true, Err(HtmlNumericParseError::InvalidSyntax)),
        ] {
            let mut scanner = HtmlIntegerPrefixScanner::new(recover);
            for unit in raw.encode_utf16() {
                if !scanner.feed(unit) {
                    break;
                }
            }
            assert_eq!(
                scanner.finish(),
                expected.map(|(negative, start, end)| HtmlScannedIntegerPrefix {
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
                Err(HtmlNumericParseError::InvalidSyntax)
            );
        }
        assert_eq!(
            parse_integer_long(&DocumentString::from_utf16(vec![0xd800, 49])),
            Err(HtmlNumericParseError::InvalidSyntax)
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
            HtmlNumericMaterializationError::Digits(HtmlDecimalDigitsError::NonAsciiDigit {
                code_unit_offset: 1,
                code_unit: 0xd800
            })
        );
        assert_eq!(invalid.capacity_bytes, 0);
        assert_eq!(
            HtmlNonNegativeInteger::try_from_borrowed_digits(&[], 0, 0)
                .unwrap_err()
                .kind,
            HtmlNumericMaterializationError::Digits(HtmlDecimalDigitsError::Empty)
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
            .try_materialized_unit_step(HtmlIntegerUnitStep::Decrement, 1, 1)
            .unwrap();
        assert_eq!(previous.value.as_document_string(), text("-1"));
        assert_eq!(
            previous
                .value
                .try_materialized_unit_step(HtmlIntegerUnitStep::Increment, 1, 1)
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
            HtmlNumericMaterializationFailure {
                kind: HtmlNumericMaterializationError::Boundary(
                    HtmlIntegerUnitStepError::DigitLimitExceeded {
                        limit: 3,
                        required: 4
                    }
                ),
                capacity_bytes: 0
            }
        );
        assert_eq!(
            value.try_multiplied_by(10, 4, 3).unwrap_err(),
            HtmlNumericMaterializationFailure {
                kind: HtmlNumericMaterializationError::Boundary(
                    HtmlIntegerUnitStepError::RequestedCapacityLimitExceeded {
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
            HtmlNumericMaterializationError::Boundary(
                HtmlIntegerUnitStepError::DigitLimitExceeded {
                    limit: 0,
                    required: 1
                }
            )
        );
        let copied = value.try_copy(3, 100).unwrap();
        let exact = value.try_copy(3, copied.capacity_bytes).unwrap();
        assert_eq!(exact.value.decimal_digits(), "999");
        let short = value.try_copy(3, copied.capacity_bytes - 1).unwrap_err();
        assert!(matches!(
            short.kind,
            HtmlNumericMaterializationError::Boundary(
                HtmlIntegerUnitStepError::RequestedCapacityLimitExceeded { .. }
                    | HtmlIntegerUnitStepError::ActualCapacityLimitExceeded { .. }
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
            HtmlNumericMaterializationError::Digits(HtmlDecimalDigitsError::NonAsciiDigit {
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
            HtmlNumericMaterializationError::Boundary(
                HtmlIntegerUnitStepError::RequestedCapacityLimitExceeded {
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
                HtmlNumericMaterializationError::Boundary(
                    HtmlIntegerUnitStepError::DigitLimitExceeded {
                        limit: 1,
                        required: 2
                    }
                )
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
            Err(HtmlIntegerUnitStepError::ActualCapacityLimitExceeded {
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
            HtmlNumericMaterializationError::Boundary(
                HtmlIntegerUnitStepError::RepresentationExhausted
            )
        );
        let requested = reserve_materialization(3, Some(3), 9, &mut observed).unwrap_err();
        assert_eq!(requested.capacity_bytes, 7);
        assert_eq!(
            requested.kind,
            HtmlNumericMaterializationError::Boundary(
                HtmlIntegerUnitStepError::RequestedCapacityLimitExceeded {
                    limit: 9,
                    required: 10
                }
            )
        );
        let digit = reserve_materialization(3, Some(2), 100, &mut observed).unwrap_err();
        assert_eq!(digit.capacity_bytes, 7);
        assert_eq!(
            digit.kind,
            HtmlNumericMaterializationError::Boundary(
                HtmlIntegerUnitStepError::DigitLimitExceeded {
                    limit: 2,
                    required: 3
                }
            )
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
                    HtmlNumericParseError::InvalidSyntax
                ))
            );
        }
    }

    #[test]
    fn scientific_equality_is_mathematical_instead_of_binary64_equality() {
        let one = strict_scientific_decimal_key(&text("1")).unwrap();
        assert_eq!(HtmlScientificDecimalKey::one(), one);
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
            Err(HtmlNumericParseError::FloatingPointOverflow)
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
                    HtmlNumericParseError::InvalidSyntax
                ))
            );
            assert_eq!(
                validate_floating_point_number(&text(input)),
                Err(HtmlNumericParseError::InvalidSyntax)
            );
            assert_eq!(
                parse_floating_point_number(&text(input)).unwrap().value(),
                recovered
            );
            let bounded = try_floating_point_number(
                &text(input),
                HtmlFloatingPointParseMode::Recovery,
                input.len(),
            )
            .unwrap();
            assert_eq!(bounded.value.unwrap().value(), recovered);
        }
        for input in ["", ".", "NaN", "Infinity", "1e--1", "1\u{a0}"] {
            assert_eq!(
                strict_scientific_decimal_key(&text(input)),
                Err(ScientificDecimalKeyError::Syntax(
                    HtmlNumericParseError::InvalidSyntax
                ))
            );
        }
        for units in [vec![0xd800, 49], vec![49, 0xdc00], vec![49, 101, 0xd800]] {
            let input = DocumentString::from_utf16(units);
            assert_eq!(
                strict_scientific_decimal_key(&input),
                Err(ScientificDecimalKeyError::Syntax(
                    HtmlNumericParseError::InvalidSyntax
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

    /// A digit buffer is a string of ASCII digits, and the first other byte is reported with its offset.
    #[test]
    fn digit_buffers_report_the_first_byte_that_is_not_an_ascii_digit() {
        assert_eq!(
            ascii_digit_string(b"0123456789".to_vec()).as_deref(),
            Ok("0123456789")
        );
        assert_eq!(ascii_digit_string(Vec::new()).as_deref(), Ok(""));
        for (bytes, offset, unit) in [
            (&b"12a4"[..], 2, u16::from(b'a')),
            (&b" 1"[..], 0, u16::from(b' ')),
            (&b"12-"[..], 2, u16::from(b'-')),
            (&b"/"[..], 0, u16::from(b'/')),
            (&b":"[..], 0, u16::from(b':')),
            (&b"12\x804"[..], 2, 0x80),
            (&b"1\xff"[..], 1, 0xff),
        ] {
            assert_eq!(
                ascii_digit_string(bytes.to_vec()),
                Err(HtmlDecimalDigitsError::NonAsciiDigit {
                    code_unit_offset: offset,
                    code_unit: unit,
                }),
                "{bytes:?}"
            );
        }
    }

    /// The predecessor shape of a decimal string keeps its length except when a lone leading one
    /// drops out, and the pivot is the last digit that is not zero.
    #[test]
    fn predecessor_shape_matches_the_decremented_digits() {
        for value in [1_u128, 2, 9, 10, 11, 100, 101, 1000, 1999, 120_000] {
            let digits = value.to_string();
            let nonzero = NonZeroDigits::new(&digits).unwrap();
            let predecessor = (value - 1).to_string();
            assert_eq!(nonzero.predecessor_length(), predecessor.len(), "{value}");
            assert_eq!(
                Some(nonzero.pivot),
                digits.as_bytes().iter().rposition(|&digit| digit != b'0'),
                "{value}"
            );
            assert_eq!(
                HtmlNonNegativeInteger::from(value)
                    .decremented()
                    .map(|integer| integer.decimal_digits().to_owned()),
                Some(predecessor),
                "{value}"
            );
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
                (HtmlIntegerUnitStep::Increment, incremented),
                (HtmlIntegerUnitStep::Decrement, decremented),
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
                .try_unit_step(HtmlIntegerUnitStep::Increment, 41, 41)
                .unwrap()
                .value
                .as_document_string(),
            text("10000000000000000000000000000000000000000")
        );
        let minus_one = exact_signed("-1")
            .try_unit_step(HtmlIntegerUnitStep::Increment, 1, 1)
            .unwrap();
        assert!(!minus_one.value.is_negative());
        assert!(minus_one.value.magnitude().is_zero());
    }

    #[test]
    fn fallible_unit_step_checks_exact_result_digits_and_capacity_before_writing() {
        let value = exact_signed("999");
        assert_eq!(
            value
                .try_unit_step(HtmlIntegerUnitStep::Increment, 3, 100)
                .unwrap_err(),
            HtmlIntegerUnitStepError::DigitLimitExceeded {
                limit: 3,
                required: 4,
            }
        );
        assert_eq!(
            value
                .try_unit_step(HtmlIntegerUnitStep::Increment, 4, 3)
                .unwrap_err(),
            HtmlIntegerUnitStepError::RequestedCapacityLimitExceeded {
                limit: 3,
                required: 4,
            }
        );
        assert_eq!(
            value
                .try_unit_step(HtmlIntegerUnitStep::Increment, 4, 4)
                .unwrap()
                .value
                .as_document_string(),
            text("1000")
        );
        assert_eq!(
            exact_signed("1000")
                .try_unit_step(HtmlIntegerUnitStep::Decrement, 3, 3)
                .unwrap()
                .value
                .as_document_string(),
            text("999")
        );
        assert_eq!(
            exact_signed("1")
                .try_unit_step(HtmlIntegerUnitStep::Decrement, 0, 1)
                .unwrap_err(),
            HtmlIntegerUnitStepError::DigitLimitExceeded {
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
            HtmlIntegerUnitStepError::DigitLimitExceeded {
                limit: 1,
                required: 2,
            }
        );
        assert_eq!(
            HtmlInteger::try_from_usize(0, 1, 0).unwrap_err(),
            HtmlIntegerUnitStepError::RequestedCapacityLimitExceeded {
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
            Err(HtmlIntegerUnitStepError::ActualCapacityLimitExceeded {
                limit: 3,
                requested: 3,
                capacity_bytes: 4,
            })
        );
        assert_eq!(
            check_integer_digit_capacity(isize::MAX as usize + 1, usize::MAX, usize::MAX),
            Err(HtmlIntegerUnitStepError::RepresentationExhausted)
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
                HtmlIntegerRecoveryError::Syntax(HtmlNumericParseError::InvalidSyntax)
            );
        }
        let raw = text(" -000123suffix");
        assert_eq!(
            try_parse_integer(&raw, 3, 3).unwrap().value.to_i64(),
            Ok(-123)
        );
        assert_eq!(
            try_parse_integer(&raw, 2, 3).unwrap_err(),
            HtmlIntegerRecoveryError::Allocation(HtmlIntegerUnitStepError::DigitLimitExceeded {
                limit: 2,
                required: 3,
            })
        );
        assert_eq!(
            try_parse_integer(&raw, 3, 2).unwrap_err(),
            HtmlIntegerRecoveryError::Allocation(
                HtmlIntegerUnitStepError::RequestedCapacityLimitExceeded {
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
            HtmlIntegerRecoveryError::Allocation(HtmlIntegerUnitStepError::DigitLimitExceeded {
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
                    Err(HtmlNumericParseError::Negative)
                );
            }
        }
        for input in ["", "+", " -", "- 0", "x1", "\u{a0}1"] {
            assert_eq!(
                parse_nonnegative_integer_capped(&text(input), 0),
                Err(HtmlNumericParseError::InvalidSyntax)
            );
        }
        let leading_surrogate = DocumentString::from_utf16(vec![0xd800, b'1' as u16]);
        assert_eq!(
            parse_nonnegative_integer_capped(&leading_surrogate, 1000),
            Err(HtmlNumericParseError::InvalidSyntax)
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
                Err(HtmlNumericParseError::InvalidSyntax)
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
            Err(HtmlNumericParseError::InvalidSyntax)
        );
        assert_eq!(
            parse_integer(&DocumentString::from_utf16(vec![49, 0xd800]))
                .unwrap()
                .to_i64(),
            Ok(1)
        );
        assert_eq!(
            validate_integer(&DocumentString::from_utf16(vec![49, 0xd800])),
            Err(HtmlNumericParseError::InvalidSyntax)
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
            ("-001", Err(HtmlNumericParseError::Negative)),
            ("+", Err(HtmlNumericParseError::InvalidSyntax)),
            (
                "340282366920938463463374607431768211456z",
                Ok("340282366920938463463374607431768211456"),
            ),
        ] {
            let input = text(raw);
            let mut scanner = HtmlIntegerPrefixScanner::new(true);
            for &unit in input.code_units() {
                if !scanner.feed(unit) {
                    break;
                }
            }
            let actual = scanner
                .finish()
                .and_then(|prefix| prefix.try_nonnegative_digits(input.code_units()));
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
            Err(HtmlNumericParseError::InvalidSyntax)
        );
        assert_eq!(
            parse_nonnegative_integer(&text("-1")),
            Err(HtmlNumericParseError::Negative)
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
                try_floating_point_number(&raw, HtmlFloatingPointParseMode::Author, 0).unwrap();
            assert_eq!(author.value, Err(HtmlNumericParseError::InvalidSyntax));
            assert_eq!(author.capacity_bytes, 0);
            let recovery =
                try_floating_point_number(&raw, HtmlFloatingPointParseMode::Recovery, token_bytes)
                    .unwrap();
            assert_eq!(recovery.value.unwrap().value(), expected);
            assert_eq!(recovery.capacity_bytes, token_bytes);
        }
        for (input, expected) in [(".5", 0.5), ("-.5", -0.5), ("1e+2", 100.0), ("1E-2", 0.01)] {
            for mode in [
                HtmlFloatingPointParseMode::Author,
                HtmlFloatingPointParseMode::Recovery,
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
                HtmlFloatingPointParseMode::Author,
                HtmlFloatingPointParseMode::Recovery,
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
            try_floating_point_number(&input, HtmlFloatingPointParseMode::Recovery, input.len())
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
                try_floating_point_number(&raw, HtmlFloatingPointParseMode::Recovery, raw.len())
                    .unwrap();
            assert_eq!(
                parsed.value,
                Err(HtmlNumericParseError::FloatingPointOverflow)
            );
            assert_eq!(parsed.capacity_bytes, raw.len());
        }
        for input in [
            "0e999999999999999999999999999999999999999999999999999999999999",
            "-1e-999999999999999999999999999999999999999999999999999999999999",
        ] {
            let raw = text(input);
            let parsed =
                try_floating_point_number(&raw, HtmlFloatingPointParseMode::Recovery, raw.len())
                    .unwrap();
            assert_eq!(parsed.value.unwrap().value().to_bits(), 0);
            assert_eq!(parsed.capacity_bytes, raw.len());
        }
        let raw = text("1e999tail");
        let author =
            try_floating_point_number(&raw, HtmlFloatingPointParseMode::Author, 0).unwrap();
        assert_eq!(author.value, Err(HtmlNumericParseError::InvalidSyntax));
        assert_eq!(author.capacity_bytes, 0);
        let recovered =
            try_floating_point_number(&raw, HtmlFloatingPointParseMode::Recovery, 5).unwrap();
        assert_eq!(
            recovered.value,
            Err(HtmlNumericParseError::FloatingPointOverflow)
        );
        assert_eq!(recovered.capacity_bytes, 5);
    }

    #[test]
    fn controlled_float_rejection_and_utf16_suffixes_allocate_only_the_selected_token() {
        for input in ["", " \t", ".", "-.", "+", "NaN", "Infinity", "١", "\u{a0}1"] {
            for mode in [
                HtmlFloatingPointParseMode::Author,
                HtmlFloatingPointParseMode::Recovery,
            ] {
                let parsed = try_floating_point_number(&text(input), mode, 0).unwrap();
                assert_eq!(parsed.value, Err(HtmlNumericParseError::InvalidSyntax));
                assert_eq!(parsed.capacity_bytes, 0);
            }
        }
        let suffix = DocumentString::from_utf16(vec![b'1' as u16, 0xd800, b'2' as u16]);
        let parsed =
            try_floating_point_number(&suffix, HtmlFloatingPointParseMode::Recovery, 1).unwrap();
        assert_eq!(parsed.value.unwrap().value(), 1.0);
        assert_eq!(parsed.capacity_bytes, 1);
        assert_eq!(suffix.code_units(), &[b'1' as u16, 0xd800, b'2' as u16]);
        let author =
            try_floating_point_number(&suffix, HtmlFloatingPointParseMode::Author, 0).unwrap();
        assert_eq!(author.value, Err(HtmlNumericParseError::InvalidSyntax));
        assert_eq!(author.capacity_bytes, 0);
        let leading = DocumentString::from_utf16(vec![0xdc00, b'1' as u16]);
        assert_eq!(
            try_floating_point_number(&leading, HtmlFloatingPointParseMode::Recovery, 0)
                .unwrap()
                .value,
            Err(HtmlNumericParseError::InvalidSyntax)
        );
    }

    #[test]
    fn controlled_float_reservation_obeys_requested_observed_and_representation_limits() {
        let input = text("  -1.5tail");
        assert_eq!(
            try_floating_point_number(&input, HtmlFloatingPointParseMode::Recovery, 3).unwrap_err(),
            HtmlFloatingPointAllocationError::RequestedCapacityLimitExceeded {
                limit: 3,
                required: 4
            }
        );
        let parsed =
            try_floating_point_number(&input, HtmlFloatingPointParseMode::Recovery, 4).unwrap();
        assert_eq!(parsed.value.unwrap().value(), -1.5);
        assert_eq!(parsed.capacity_bytes, 4);
        // Exercise the real observed-capacity boundary with independent literal
        // observations, without a fake allocator or forced out-of-memory claim.
        assert_eq!(check_floating_point_actual_capacity(3, 3, 3), Ok(()));
        assert_eq!(check_floating_point_actual_capacity(3, 4, 4), Ok(()));
        assert_eq!(
            check_floating_point_actual_capacity(3, 4, 3),
            Err(
                HtmlFloatingPointAllocationError::ActualCapacityLimitExceeded {
                    limit: 3,
                    requested: 3,
                    capacity_bytes: 4
                }
            )
        );
        assert_eq!(
            check_floating_point_requested_capacity(isize::MAX as usize + 1, usize::MAX),
            Err(HtmlFloatingPointAllocationError::RepresentationExhausted)
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
                Err(HtmlNumericParseError::InvalidSyntax)
            );
            assert_eq!(
                parse_floating_point_number(&text(input)).unwrap().value(),
                expected
            );
        }
        for input in ["", ".", "-.", "NaN", "Infinity", "\u{a0}1"] {
            assert_eq!(
                parse_floating_point_number(&text(input)),
                Err(HtmlNumericParseError::InvalidSyntax)
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
            Err(HtmlNumericParseError::FloatingPointOverflow)
        );
        assert_eq!(
            validate_floating_point_number(&text("1e9999")),
            Err(HtmlNumericParseError::FloatingPointOverflow)
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
    fn checked_native_floats_reject_nonfinite_values_and_serialize_plain_decimal_syntax() {
        for value in [f64::NAN, f64::INFINITY, f64::NEG_INFINITY] {
            assert_eq!(
                HtmlFloatingPointNumber::try_from(value),
                Err(HtmlNonFiniteNumberError)
            );
        }
        for value in [-0.0, -1.25, f64::MAX, f64::MIN_POSITIVE, f64::from_bits(1)] {
            let number = HtmlFloatingPointNumber::try_from(value).unwrap();
            assert_eq!(
                validate_floating_point_number(&number.to_plain_decimal_string()),
                Ok(number)
            );
        }
        assert_eq!(
            HtmlFloatingPointNumber::try_from(-0.0)
                .unwrap()
                .to_plain_decimal_string(),
            text("0")
        );
        // Plain decimal expansion never switches to an exponent, unlike the
        // ECMAScript best representation, which spells 1e21 as 1e+21.
        assert_eq!(
            HtmlFloatingPointNumber::try_from(1e21)
                .unwrap()
                .to_plain_decimal_string(),
            text("1000000000000000000000")
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
                Err(HtmlNumericParseError::InvalidSyntax)
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
        assert_eq!(recovered.to_plain_decimal_string(), text("2,0,0,-3,0"));
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
                Err(HtmlNumericParseError::InvalidSyntax)
            );
        }
        assert_eq!(
            validate_floating_point_list(&text("1e999,1.5")),
            Err(HtmlNumericParseError::FloatingPointOverflow)
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
                Err(HtmlNumericParseError::InvalidSyntax)
            );
        }
    }

    #[test]
    fn nonzero_dimension_checks_exact_zero_without_binary64_underflow() {
        for input in ["0", "0.000%", "000.0tail"] {
            assert_eq!(
                parse_nonzero_dimension(&text(input)),
                Err(HtmlNumericParseError::ZeroDimension)
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
    /// Advances a grammar cursor to its terminal step. `HANG_GUARD` is only a
    /// generous fixed ceiling so a non-terminating regression fails with a clear
    /// message instead of hanging; it is not a cost contract, and the grammar
    /// declares no total action count.
    fn finish_grammar(cursor: &mut HtmlFloatingPointGrammarCursor<'_>) {
        const HANG_GUARD: usize = 100_000;
        for _ in 0..HANG_GUARD {
            if cursor.is_finished() {
                return;
            }
            cursor.advance();
        }
        assert!(
            cursor.is_finished(),
            "grammar cursor did not finish within the hang guard"
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
                HtmlFloatingPointGrammarCursor::new(&units, HtmlFloatingPointParseMode::Recovery);
            finish_grammar(&mut cursor);
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
            assert_eq!(
                floating_point_prefix(&units, HtmlFloatingPointParseMode::Author).err(),
                Some(HtmlNumericParseError::InvalidSyntax)
            );
        }
        let units = text("1e999");
        let mut cursor = HtmlFloatingPointGrammarCursor::new(
            units.code_units(),
            HtmlFloatingPointParseMode::Author,
        );
        finish_grammar(&mut cursor);
        assert!(cursor.is_valid());
        assert_eq!(cursor.prefix().unwrap().token, units.code_units());
        assert_eq!(
            validate_floating_point_number(&units),
            Err(HtmlNumericParseError::FloatingPointOverflow)
        );
    }
}
