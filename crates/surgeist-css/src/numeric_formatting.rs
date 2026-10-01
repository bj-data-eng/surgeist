//! Canonical ordinary number text from checked exact authored coefficients.
//!
//! CSSOM limits fractional text to six places. Its decimal tie direction is
//! unspecified; the selected frozen WebKit FIXED policy rounds ties away from
//! zero. This owner does not format calculation or color projection results.

use crate::exact_decimal::LexicalDecimal;
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
    let value = LexicalDecimal::new(text);
    if value.len == 0 {
        return emit(value.digits(), 0, 0, false, limit);
    }
    let Some(exponent) = value.exponent else {
        // The input length and all callers' decimal shifts are bounded. An
        // exponent whose magnitude exceeds i128 dwarfs either adjustment.
        return if value.exponent_negative {
            emit(std::iter::empty(), 0, 0, false, limit)
        } else {
            Err(Error::new(ByteLimit))
        };
    };
    let Some(exponent) = exponent.checked_add(shift) else {
        return if shift < 0 {
            emit(std::iter::empty(), 0, 0, false, limit)
        } else {
            Err(Error::new(ByteLimit))
        };
    };
    if exponent >= -6 {
        return emit(value.digits(), value.len, exponent, value.negative, limit);
    }
    // The prefix before the six-place cutoff, including integer digits.
    // Since exponent < -6, this length is strictly smaller than value.len.
    let kept = (value.len as i128)
        .checked_add(exponent)
        .and_then(|point| point.checked_add(6))
        .ok_or_else(|| Error::new(ByteLimit))?;
    if kept < 0 {
        return emit(std::iter::empty(), 0, 0, false, limit);
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
        return emit(std::iter::empty(), 0, 0, false, limit);
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
    emit(digits, len, exponent, value.negative, limit)
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
