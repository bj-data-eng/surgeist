//! Dependency-free finite numerical formatting and shared tie policy.
//!
//! Exact authored coefficients and CSS specified-value policy stay with their
//! owners. This module consumes actual finite binary64 bits only, proven by
//! [`Finite`].

pub(crate) mod bounded;
mod controlled;
use crate::Finite;
use bounded::{FiniteFormatCursor, FormatReply, FormatRequest};
pub use controlled::{
    FiniteFormatter, FormatAction, FormatCompute, FormatDigitReceipt, FormatEmit, FormatReadDigit,
    FormatStep, FormatSummary, FormatWriteDigit,
};

/// A refused finite formatting output or allocation.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[non_exhaustive]
pub enum FormatError {
    ByteLimit,
    CapacityOverflow,
}

impl std::fmt::Display for FormatError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::ByteLimit => f.write_str("finite formatted output exceeds the byte limit"),
            Self::CapacityOverflow => f.write_str("finite formatting storage capacity exceeded"),
        }
    }
}
impl std::error::Error for FormatError {}

/// Generic specified coefficients use away-from-zero ties; exact color
/// rationals and final ordinary sRGB scalars use ties toward positive infinity.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum RoundingMode {
    TowardPositiveInfinity,
    AwayFromZero,
}
impl RoundingMode {
    /// Resolves a comparison with the halfway point for the supplied sign.
    pub fn rounds_up(self, ordering: std::cmp::Ordering, negative: bool) -> bool {
        ordering.is_gt() || (ordering.is_eq() && (matches!(self, Self::AwayFromZero) || !negative))
    }
}

/// Formats actual finite binary64 bits to at most six fractional decimal places.
pub fn format_finite(
    value: Finite,
    rounding: RoundingMode,
    byte_limit: usize,
) -> Result<String, FormatError> {
    let mut cursor = FiniteFormatCursor::new(value, rounding);
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
            if length > byte_limit {
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
#[must_use]
pub fn rounds_to_zero(value: Finite, rounding: RoundingMode) -> bool {
    let bits = value.get().to_bits();
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

#[cfg(test)]
mod tests {
    use super::*;

    fn finite(value: f64) -> Finite {
        Finite::new(value).expect("finite formatter fixture")
    }

    #[test]
    fn native_output_preserves_ties_zero_and_exact_byte_admission() {
        for (value, away, positive) in [
            (-0.0, "0", "0"),
            (f64::from_bits(1), "0", "0"),
            (-f64::MIN_POSITIVE, "0", "0"),
            (1.0 / 3.0, "0.333333", "0.333333"),
            (1.25, "1.25", "1.25"),
            (-1.0 / 128.0, "-0.007813", "-0.007812"),
            (
                f64::from_bits(0x3f7f_ffff_ffff_ffff),
                "0.007812",
                "0.007812",
            ),
            (
                f64::from_bits(0x3f80_0000_0000_0001),
                "0.007813",
                "0.007813",
            ),
            (f64::from_bits(0x3ea0_c6f7_a0b5_ed8d), "0", "0"),
            (
                f64::from_bits(0x3ea0_c6f7_a0b5_ed8e),
                "0.000001",
                "0.000001",
            ),
            (0.9999996, "1", "1"),
            (-0.9999996, "-1", "-1"),
            (
                f64::from_bits(0x43ab_c16d_674e_c801),
                "1000000000000000128",
                "1000000000000000128",
            ),
        ] {
            for (rounding, expected) in [
                (RoundingMode::AwayFromZero, away),
                (RoundingMode::TowardPositiveInfinity, positive),
            ] {
                assert_eq!(
                    format_finite(finite(value), rounding, expected.len()).unwrap(),
                    expected
                );
                assert_eq!(
                    format_finite(finite(value), rounding, expected.len() - 1),
                    Err(FormatError::ByteLimit)
                );
                assert_eq!(rounds_to_zero(finite(value), rounding), expected == "0");
            }
        }
    }

    #[test]
    fn maximum_native_integer_has_exact_text_and_final_byte_preflight() {
        // (2^53 - 1) * 2^971, independently expanded integer oracle.
        const INTEGER: &str = concat!(
            "179769313486231570814527423731704356798070567525844996598917476803157260",
            "780028538760589558632766878171540458953514382464234321326889464182768467",
            "546703537516986049910576551282076245490090389328944075868508455133942304",
            "583236903222948165808559332123348274797826204144723168738177180919299881",
            "250404026184124858368"
        );
        for rounding in [
            RoundingMode::AwayFromZero,
            RoundingMode::TowardPositiveInfinity,
        ] {
            assert_eq!(
                format_finite(finite(f64::MAX), rounding, 309).unwrap(),
                INTEGER
            );
            assert_eq!(
                format_finite(finite(-f64::MAX), rounding, 310).unwrap(),
                format!("-{INTEGER}")
            );
            assert_eq!(
                format_finite(finite(f64::MAX), rounding, 308),
                Err(FormatError::ByteLimit)
            );
            assert_eq!(
                format_finite(finite(-f64::MAX), rounding, 309),
                Err(FormatError::ByteLimit)
            );
        }
    }
}
