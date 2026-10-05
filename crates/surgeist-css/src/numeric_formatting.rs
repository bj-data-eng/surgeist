//! Canonical number text from exact authored coefficients or finite projected bits.
//!
//! CSSOM limits fractional text to six places. Its decimal tie direction is
//! unspecified; the selected frozen WebKit FIXED policy rounds ties away from
//! zero. Final ordinary sRGB scalars select Color 4 ties toward positive infinity
//! on the same finite-bit kernel; retained color calculation roles stay separate.

use crate::exact_decimal::{DecimalRounding, LexicalDecimal};
use crate::{
    CssSpecifiedValueSerializationError as Error, CssSpecifiedValueSerializationErrorKind,
};

type Result<T> = std::result::Result<T, Error>;
use CssSpecifiedValueSerializationErrorKind::{ByteLimit, CapacityOverflow};

/// Applies an exact decimal shift before planning the actual rounded output.
/// Input validity belongs to the checked component owner. Current unit policies
/// supply only shifts 0, -2 and -3. Scanning borrows the coefficient; allocation
/// is limited to final text, never an expanded input.
pub(crate) fn format_css_number(text: &str, shift: i128, limit: usize) -> Result<String> {
    format_css_number_with_sign(text, shift, false, limit).map(|number| number.text)
}

/// Formatting metadata comes from the decimal plan, before text is emitted.
pub(crate) struct FormattedNumber {
    pub(crate) text: String,
    pub(crate) is_zero: bool,
}

/// Slot-owned sign inversion preserves exact input and the shared rounding policy.
pub(crate) fn format_css_number_with_sign(
    text: &str,
    shift: i128,
    negate: bool,
    limit: usize,
) -> Result<FormattedNumber> {
    let value = LexicalDecimal::new(text);
    if value.len == 0 {
        return emit_with_metadata(value.digits(), 0, 0, false, limit);
    }
    let Some(exponent) = value.exponent else {
        // The input length and all callers' decimal shifts are bounded. An
        // exponent whose magnitude exceeds i128 dwarfs either adjustment.
        return if value.exponent_negative {
            emit_with_metadata(std::iter::empty(), 0, 0, false, limit)
        } else {
            Err(Error::new(ByteLimit))
        };
    };
    let Some(exponent) = exponent.checked_add(shift) else {
        return if shift < 0 {
            emit_with_metadata(std::iter::empty(), 0, 0, false, limit)
        } else {
            Err(Error::new(ByteLimit))
        };
    };
    if exponent >= -6 {
        return emit_with_metadata(
            value.digits(),
            value.len,
            exponent,
            value.negative != negate,
            limit,
        );
    }
    // The prefix before the six-place cutoff, including integer digits.
    // Since exponent < -6, this length is strictly smaller than value.len.
    let kept = (value.len as i128)
        .checked_add(exponent)
        .and_then(|point| point.checked_add(6))
        .ok_or_else(|| Error::new(ByteLimit))?;
    if kept < 0 {
        return emit_with_metadata(std::iter::empty(), 0, 0, false, limit);
    }
    let kept = usize::try_from(kept).map_err(|_| Error::new(CapacityOverflow))?;
    let mut last_nonzero = None;
    let mut last_non_nine = None;
    let mut digits = value.digits();
    for (index, digit) in digits.by_ref().take(kept).enumerate() {
        if digit != 0 {
            last_nonzero = Some(index);
        }
        if digit != 9 {
            last_non_nine = Some(index);
        }
    }
    let round_up = digits.next().expect("fraction beyond cutoff") >= 5;
    let (len, trailing, increment, one) = if round_up {
        if let Some(index) = last_non_nine {
            (index + 1, kept - index - 1, Some(index), false)
        } else {
            // A prefix of all nines (or no prefix) carries to one followed by
            // kept zeros. Strip those zeros before budgeting or allocation.
            (1, kept, None, true)
        }
    } else if let Some(index) = last_nonzero {
        (index + 1, kept - index - 1, None, false)
    } else {
        return emit_with_metadata(std::iter::empty(), 0, 0, false, limit);
    };
    let exponent = -6 + trailing as i128;
    let digits = value.digits().take(len).enumerate().map(|(index, digit)| {
        if one {
            1
        } else if increment == Some(index) {
            digit + 1
        } else {
            digit
        }
    });
    emit_with_metadata(digits, len, exponent, value.negative != negate, limit)
}

/// Rounds the actual finite binary value to millionths, without a float round
/// or a shortest-decimal intermediate. Fixed stack storage covers every finite
/// binary64 integer; only the final bounded text is allocated.
pub(crate) fn format_projected_number(value: f64, limit: usize) -> Result<String> {
    format_finite_number(value, limit, DecimalRounding::AwayFromZero)
}

/// Final ordinary sRGB text uses nearest millionths with exact ties toward
/// positive infinity. The supplied value is the original post-scale outcome or
/// the owning color algorithm's final binary64 channel, never decimal text.
pub(crate) fn format_ordinary_color_number(value: f64, limit: usize) -> Result<String> {
    format_finite_number(value, limit, DecimalRounding::TowardPositiveInfinity)
}

fn format_finite_number(value: f64, limit: usize, rounding: DecimalRounding) -> Result<String> {
    assert!(value.is_finite(), "checked finite projected scalar");
    let bits = value.to_bits();
    let negative = bits >> 63 != 0;
    let encoded_exponent = ((bits >> 52) & 0x7ff) as i32;
    let fraction = bits & ((1_u64 << 52) - 1);
    let (significand, exponent) = if encoded_exponent == 0 {
        (fraction, -1074)
    } else {
        ((1_u64 << 52) | fraction, encoded_exponent - 1075)
    };
    if significand == 0 {
        return emit(std::iter::empty(), 0, 0, false, limit);
    }
    let mut digits = [0_u8; 309];
    let mut len = 0;
    let decimal_exponent;
    if exponent < 0 {
        let mut rounded = rounded_millionths(significand, exponent, negative, rounding);
        if rounded == 0 {
            return emit(std::iter::empty(), 0, 0, false, limit);
        }
        let mut trailing = 0;
        while rounded.is_multiple_of(10) {
            trailing += 1;
            rounded /= 10;
        }
        decimal_exponent = -6 + trailing;
        while rounded != 0 {
            digits[len] = (rounded % 10) as u8;
            len += 1;
            rounded /= 10;
        }
    } else {
        let mut integer = significand;
        while integer != 0 {
            digits[len] = (integer % 10) as u8;
            len += 1;
            integer /= 10;
        }
        for _ in 0..exponent {
            let mut carry = 0;
            for digit in &mut digits[..len] {
                let doubled = *digit * 2 + carry;
                *digit = doubled % 10;
                carry = doubled / 10;
            }
            if carry != 0 {
                // (2^53-1)*2^971 has at most 309 decimal digits.
                digits[len] = carry;
                len += 1;
            }
        }
        decimal_exponent = 0;
    }
    emit(
        digits[..len].iter().rev().copied(),
        len,
        decimal_exponent,
        negative,
        limit,
    )
}

/// Tests the actual finite binary coefficient using the formatter's millionths
/// kernel; this is emitted-zero metadata, not authored-value classification.
pub(crate) fn projected_number_rounds_to_zero(value: f64) -> bool {
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
    rounded_millionths(
        significand,
        exponent,
        bits >> 63 != 0,
        DecimalRounding::AwayFromZero,
    ) == 0
}

fn rounded_millionths(
    significand: u64,
    exponent: i32,
    negative: bool,
    rounding: DecimalRounding,
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

fn emit_with_metadata(
    digits: impl Iterator<Item = u8>,
    len: usize,
    exponent: i128,
    negative: bool,
    limit: usize,
) -> Result<FormattedNumber> {
    let is_zero = len == 0;
    emit(digits, len, exponent, negative, limit).map(|text| FormattedNumber { text, is_zero })
}

/// Emits a normalized finite decimal whose fractional part is already rounded.
/// Count the final representation before reserving any storage.
fn emit(
    digits: impl Iterator<Item = u8>,
    len: usize,
    exponent: i128,
    negative: bool,
    limit: usize,
) -> Result<String> {
    if len == 0 {
        if limit == 0 {
            return Err(Error::new(ByteLimit));
        }
        let mut output = String::new();
        output
            .try_reserve(1)
            .map_err(|_| Error::new(CapacityOverflow))?;
        output.push('0');
        return Ok(output);
    }
    let point = (len as i128)
        .checked_add(exponent)
        .ok_or_else(|| Error::new(ByteLimit))?;
    let length = if point <= 0 {
        (len as i128)
            .checked_add(2)
            .and_then(|n| n.checked_sub(point))
    } else {
        point
            .max(len as i128)
            .checked_add(i128::from(point < len as i128))
    }
    .and_then(|n| n.checked_add(i128::from(negative)))
    .ok_or_else(|| Error::new(ByteLimit))?;
    if length > limit as i128 {
        return Err(Error::new(ByteLimit));
    }
    let length = usize::try_from(length).map_err(|_| Error::new(CapacityOverflow))?;
    if length > isize::MAX as usize {
        return Err(Error::new(CapacityOverflow));
    }
    let mut output = String::new();
    output
        .try_reserve(length)
        .map_err(|_| Error::new(CapacityOverflow))?;
    if negative {
        output.push('-');
    }
    if point <= 0 {
        output.push_str("0.");
        output.extend(std::iter::repeat_n('0', (-point) as usize));
    }
    for (index, digit) in digits.enumerate() {
        if point > 0 && index as i128 == point {
            output.push('.');
        }
        output.push(char::from(b'0' + digit));
    }
    if point > len as i128 {
        output.extend(std::iter::repeat_n('0', (point - len as i128) as usize));
    }
    debug_assert_eq!(output.len(), length);
    Ok(output)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn projected_dyadic_rounding_uses_bits_and_actual_output_budgets() {
        for (value, expected) in [
            (1.0 / 3.0, "0.333333"),
            (-1.0 / 128.0, "-0.007813"),
            (f64::from_bits(0x3f7fffffffffffff), "0.007812"),
            (f64::from_bits(0x3f80000000000000), "0.007813"),
            (f64::from_bits(0x3f80000000000001), "0.007813"),
            (5e-7, "0"),
            (5.000000000000001e-7, "0.000001"),
            (-0.0000004, "0"),
            (0.9999996, "1"),
            (-0.9999996, "-1"),
            (f64::from_bits(1), "0"),
            (f64::MIN_POSITIVE, "0"),
            (-0.0, "0"),
            (f64::from_bits(0x43ab_c16d_674e_c801), "1000000000000000128"),
        ] {
            assert_eq!(
                format_projected_number(value, expected.len()).unwrap(),
                expected
            );
            assert_eq!(
                format_projected_number(value, expected.len() - 1)
                    .unwrap_err()
                    .kind(),
                ByteLimit
            );
        }
        // The shortest spelling 5e-7 is above its actual binary approximation.
        assert_eq!(format_css_number("5e-7", 0, 8).unwrap(), "0.000001");
    }

    #[test]
    fn ordinary_color_rounding_uses_positive_ties_and_exact_final_bounds() {
        for (value, expected) in [
            (f64::from_bits(0x3f7fffffffffffff), "0.007812"),
            (f64::from_bits(0x3f80000000000000), "0.007813"),
            (f64::from_bits(0x3f80000000000001), "0.007813"),
            (-f64::from_bits(0x3f7fffffffffffff), "-0.007812"),
            (-f64::from_bits(0x3f80000000000000), "-0.007812"),
            (-f64::from_bits(0x3f80000000000001), "-0.007813"),
            (f64::from_bits(0x3ea0c6f7a0b5ed8d), "0"),
            (f64::from_bits(0x3ea0c6f7a0b5ed8e), "0.000001"),
            (-f64::from_bits(0x3ea0c6f7a0b5ed8d), "0"),
            (-f64::from_bits(0x3ea0c6f7a0b5ed8e), "-0.000001"),
            (0.9999996, "1"),
            (-0.9999996, "-1"),
            (f64::from_bits(1), "0"),
            (-f64::from_bits(1), "0"),
            (f64::MIN_POSITIVE, "0"),
            (-f64::MIN_POSITIVE, "0"),
            (0.0, "0"),
            (-0.0, "0"),
            (f64::from_bits(0x43ab_c16d_674e_c801), "1000000000000000128"),
        ] {
            assert_eq!(
                format_ordinary_color_number(value, expected.len()).unwrap(),
                expected
            );
            assert_eq!(
                format_ordinary_color_number(value, expected.len() - 1)
                    .unwrap_err()
                    .kind(),
                ByteLimit
            );
        }
    }

    #[test]
    fn maximum_projected_integer_is_exact_and_preflighted() {
        // (2^53 - 1) * 2^971, independently expanded integer oracle.
        const INTEGER: &str = concat!(
            "179769313486231570814527423731704356798070567525844996598917476803157260",
            "780028538760589558632766878171540458953514382464234321326889464182768467",
            "546703537516986049910576551282076245490090389328944075868508455133942304",
            "583236903222948165808559332123348274797826204144723168738177180919299881",
            "250404026184124858368"
        );
        for formatter in [format_projected_number, format_ordinary_color_number] {
            assert_eq!(formatter(f64::MAX, 309).unwrap(), INTEGER);
            assert_eq!(formatter(-f64::MAX, 310).unwrap(), format!("-{INTEGER}"));
            assert_eq!(formatter(f64::MAX, 308).unwrap_err().kind(), ByteLimit);
            assert_eq!(formatter(-f64::MAX, 309).unwrap_err().kind(), ByteLimit);
        }
    }

    #[test]
    fn exponent_extremes_and_zero_do_not_require_expanded_intermediates() {
        for number in [
            "1e-170141183460469231731687303715884105728",
            "1e-170141183460469231731687303715884105729",
            "-1e-99999999999999999999999999999999999999999999",
            "-0e999999999999999999999999999999999999999999999",
        ] {
            for shift in [0, -2, -3] {
                assert_eq!(format_css_number(number, shift, 1).unwrap(), "0");
                assert_eq!(
                    format_css_number(number, shift, 0).unwrap_err().kind(),
                    ByteLimit
                );
            }
        }
        for number in [
            "1e170141183460469231731687303715884105727",
            "1e170141183460469231731687303715884105728",
            "1e999999999999999999999999999999999999999999999",
        ] {
            for shift in [0, -2, -3] {
                assert_eq!(
                    format_css_number(number, shift, 10).unwrap_err().kind(),
                    ByteLimit
                );
            }
        }
    }

    #[test]
    fn discarded_tails_and_carries_budget_only_final_digits() {
        for (number, expected) in [
            ("0.9999994", "0.999999"),
            ("0.9999995", "1"),
            ("-0.9999995", "-1"),
            ("0.00009995", "0.0001"),
            ("0.0000005", "0.000001"),
            ("999.9999995", "1000"),
            ("1.200000499999", "1.2"),
            ("1.2000005", "1.200001"),
        ] {
            assert_eq!(
                format_css_number(number, 0, expected.len()).unwrap(),
                expected
            );
            assert_eq!(
                format_css_number(number, 0, expected.len() - 1)
                    .unwrap_err()
                    .kind(),
                ByteLimit
            );
        }
        let number = format!("{}.9999995", "9".repeat(1000));
        let expected = format!("1{}", "0".repeat(1000));
        assert_eq!(format_css_number(&number, 0, 1001).unwrap(), expected);
        assert_eq!(
            format_css_number(&number, 0, 1000).unwrap_err().kind(),
            ByteLimit
        );
        let tiny = format!("0.{}5", "0".repeat(100_000));
        assert_eq!(format_css_number(&tiny, 0, 1).unwrap(), "0");
    }
}
