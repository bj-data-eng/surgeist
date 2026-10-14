//! Dependency-free finite numerical formatting and shared tie policy.
//!
//! Exact authored coefficients and CSS specified-value policy stay with their
//! owners. This module consumes actual finite binary64 bits only.

pub(crate) mod bounded;
use bounded::{FiniteFormatCursor, FormatReply, FormatRequest};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum FormatError {
    ByteLimit,
    CapacityOverflow,
}

/// Generic specified coefficients use away-from-zero ties; exact color
/// rationals and final ordinary sRGB scalars use ties toward positive infinity.
#[derive(Clone, Copy)]
pub(crate) enum RoundingMode {
    TowardPositiveInfinity,
    AwayFromZero,
}
impl RoundingMode {
    pub(crate) fn rounds_up(self, ordering: std::cmp::Ordering, negative: bool) -> bool {
        ordering.is_gt() || (ordering.is_eq() && (matches!(self, Self::AwayFromZero) || !negative))
    }
}

pub(crate) fn format(
    value: f64,
    limit: usize,
    rounding: RoundingMode,
) -> Result<String, FormatError> {
    assert!(value.is_finite(), "checked finite projected scalar");
    let mut cursor = FiniteFormatCursor::new(value, rounding).expect("checked finite scalar");
    // This ordinary owner services the SAME digit protocol as the controlled
    // used serializer. It owns real scratch and reserves only final output;
    // no preview number or expanded fractional spelling is materialized.
    let mut scratch = Vec::new();
    let mut output = String::new();
    let mut output_ready = false;
    loop {
        if let Some(length) = cursor.planned_length()
            && !output_ready
        {
            if length > limit {
                return Err(FormatError::ByteLimit);
            }
            output
                .try_reserve(length)
                .map_err(|_| FormatError::CapacityOverflow)?;
            output_ready = true;
        }
        let reply = match cursor.next_request() {
            FormatRequest::Compute(kernel) => FormatReply::Computed(kernel),
            FormatRequest::ReadDigit { index } => FormatReply::Digit {
                index,
                digit: scratch[index],
            },
            FormatRequest::AppendDigit { index, digit } => {
                assert_eq!(index, scratch.len(), "actual next scratch index");
                scratch
                    .try_reserve(1)
                    .map_err(|_| FormatError::CapacityOverflow)?;
                scratch.push(digit);
                FormatReply::Appended { index, digit }
            }
            FormatRequest::ReplaceDigit { index, digit } => {
                scratch[index] = digit;
                FormatReply::Replaced { index, digit }
            }
            FormatRequest::EmitByte { unit } => {
                debug_assert!(output_ready);
                output.push(char::from(unit));
                FormatReply::Emitted { unit }
            }
            FormatRequest::Done(summary) => {
                debug_assert_eq!(summary.emitted, output.len());
                return Ok(output);
            }
        };
        cursor
            .complete(reply)
            .expect("actual ordinary owner acknowledgement");
    }
}

/// Tests the actual finite binary coefficient using the formatter's millionths
/// kernel; this is emitted-zero metadata, not authored-value classification.
pub(crate) fn rounds_to_zero(value: f64, rounding: RoundingMode) -> bool {
    debug_assert!(value.is_finite());
    let bits = value.to_bits();
    let encoded_exponent = ((bits >> 52) & 0x7ff) as i32;
    let fraction = bits & ((1_u64 << 52) - 1);
    let (significand, exponent) = if encoded_exponent == 0 {
        (fraction, -1074)
    } else {
        ((1_u64 << 52) | fraction, encoded_exponent - 1075)
    };
    if significand == 0 {
        return true;
    }
    if exponent >= 0 {
        return false;
    }
    rounded_millionths(significand, exponent, bits >> 63 != 0, rounding) == 0
}

fn rounded_millionths(
    significand: u64,
    exponent: i32,
    negative: bool,
    rounding: RoundingMode,
) -> u128 {
    debug_assert!(exponent < 0);
    let numerator = u128::from(significand) * 1_000_000;
    let shift = exponent.unsigned_abs();
    if shift >= 128 {
        return 0;
    }
    let denominator = 1_u128 << shift;
    numerator / denominator
        + u128::from(
            rounding.rounds_up((numerator % denominator).cmp(&(denominator / 2)), negative),
        )
}
