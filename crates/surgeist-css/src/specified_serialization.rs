//! Bounded, context-independent specified-value text.

use crate::{CssOpacityScalarKind, CssOpacityValue};
use std::fmt;

/// Resource policy for specified-value projection and serialization.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct CssSpecifiedValueSerializationLimits {
    max_input_nodes: usize,
    max_projection_nodes: usize,
    max_css_bytes: usize,
}

impl CssSpecifiedValueSerializationLimits {
    /// Sets independent limits. Zero is a valid limit.
    #[must_use]
    pub const fn new(
        max_input_nodes: usize,
        max_projection_nodes: usize,
        max_css_bytes: usize,
    ) -> Self {
        Self {
            max_input_nodes,
            max_projection_nodes,
            max_css_bytes,
        }
    }
    #[must_use]
    pub const fn max_input_nodes(self) -> usize {
        self.max_input_nodes
    }
    #[must_use]
    pub const fn max_projection_nodes(self) -> usize {
        self.max_projection_nodes
    }
    #[must_use]
    pub const fn max_css_bytes(self) -> usize {
        self.max_css_bytes
    }
}

impl Default for CssSpecifiedValueSerializationLimits {
    fn default() -> Self {
        Self::new(65_536, 262_144, 1_048_576)
    }
}

/// The exhausted resource or unrepresentable capacity calculation.
#[non_exhaustive]
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum CssSpecifiedValueSerializationErrorKind {
    InputNodeLimit,
    ProjectionNodeLimit,
    ByteLimit,
    CapacityOverflow,
}

/// Atomic failure: no partial CSS is returned and the authored input is unchanged.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CssSpecifiedValueSerializationError {
    kind: CssSpecifiedValueSerializationErrorKind,
}

impl CssSpecifiedValueSerializationError {
    pub(crate) const fn new(kind: CssSpecifiedValueSerializationErrorKind) -> Self {
        Self { kind }
    }
    #[must_use]
    pub const fn kind(&self) -> CssSpecifiedValueSerializationErrorKind {
        self.kind
    }
}
impl fmt::Display for CssSpecifiedValueSerializationError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(match self.kind {
            CssSpecifiedValueSerializationErrorKind::InputNodeLimit => {
                "specified-value input node limit exceeded"
            }
            CssSpecifiedValueSerializationErrorKind::ProjectionNodeLimit => {
                "specified-value projection node limit exceeded"
            }
            CssSpecifiedValueSerializationErrorKind::ByteLimit => {
                "specified-value CSS byte limit exceeded"
            }
            CssSpecifiedValueSerializationErrorKind::CapacityOverflow => {
                "specified-value capacity calculation overflowed"
            }
        })
    }
}
impl std::error::Error for CssSpecifiedValueSerializationError {}

type Result<T> = std::result::Result<T, CssSpecifiedValueSerializationError>;
use CssSpecifiedValueSerializationErrorKind as Kind;

pub(crate) fn serialize_keyword_sequence(
    text: &str,
    limits: CssSpecifiedValueSerializationLimits,
) -> Result<String> {
    if limits.max_input_nodes() == 0 {
        return Err(CssSpecifiedValueSerializationError::new(
            Kind::InputNodeLimit,
        ));
    }
    if limits.max_projection_nodes() == 0 {
        return Err(CssSpecifiedValueSerializationError::new(
            Kind::ProjectionNodeLimit,
        ));
    }
    if text.len() > limits.max_css_bytes() {
        return Err(CssSpecifiedValueSerializationError::new(Kind::ByteLimit));
    }
    Ok(text.to_owned())
}

impl CssOpacityValue {
    /// Produces canonical specified opacity, without computed-value clamping.
    ///
    /// Ordinary scalar magnitudes are exact; context-independent math uses
    /// binary64 arithmetic. Expressions requiring external context stay symbolic.
    pub fn serialize_specified(&self) -> Result<String> {
        self.serialize_specified_with_limits(CssSpecifiedValueSerializationLimits::default())
    }

    /// Projects and serializes with independent input, projection and byte limits.
    pub fn serialize_specified_with_limits(
        &self,
        limits: CssSpecifiedValueSerializationLimits,
    ) -> Result<String> {
        match self {
            Self::Calculation(value) => {
                return crate::numeric::project_specified(&value.expression, limits);
            }
            Self::PercentageCalculation(value) => {
                return crate::numeric::project_specified(&value.expression, limits);
            }
            _ => {}
        }
        if limits.max_input_nodes() == 0 {
            return Err(CssSpecifiedValueSerializationError::new(
                Kind::InputNodeLimit,
            ));
        }
        if limits.max_projection_nodes() == 0 {
            return Err(CssSpecifiedValueSerializationError::new(
                Kind::ProjectionNodeLimit,
            ));
        }
        match self {
            Self::Literal(value) => crate::opacity_scalar::serialize_binary32(
                value.value(),
                false,
                limits.max_css_bytes(),
            ),
            Self::Number(value) => crate::opacity_scalar::serialize_binary32(
                value.value(),
                false,
                limits.max_css_bytes(),
            ),
            Self::Percentage(value) => crate::opacity_scalar::serialize_binary32(
                value.value(),
                true,
                limits.max_css_bytes(),
            ),
            Self::ExactScalar(value) => format_lexical(
                value.numeric().representation(),
                value.kind() == CssOpacityScalarKind::Percentage,
                limits.max_css_bytes(),
            ),
            Self::Calculation(_) | Self::PercentageCalculation(_) => {
                unreachable!("calculation branches handled above")
            }
        }
    }
}

fn format_lexical(text: &str, percentage: bool, limit: usize) -> Result<String> {
    let negative = text.starts_with('-');
    let text = text.strip_prefix(['+', '-']).unwrap_or(text);
    let (mantissa, exponent) = text.split_once(['e', 'E']).unwrap_or((text, "0"));
    let first = mantissa.bytes().position(|c| matches!(c, b'1'..=b'9'));
    let Some(first) = first else {
        return format_digits(std::iter::empty(), 0, 0, false, limit);
    };
    let last = mantissa
        .bytes()
        .rposition(|c| matches!(c, b'1'..=b'9'))
        .expect("nonzero mantissa");
    let significant = &mantissa[first..=last];
    let len = significant.len() - usize::from(significant.contains('.'));
    let fractional = mantissa
        .find('.')
        .map_or(0, |point| mantissa.len() - point - 1);
    let trailing = mantissa[last + 1..].bytes().filter(|c| *c != b'.').count();
    // An exponent outside i128 cannot be canceled by a usize-sized source into
    // a usize-sized output. Parse without allocating exponent-sized storage.
    let exponent = exponent
        .parse::<i128>()
        .map_err(|_| CssSpecifiedValueSerializationError::new(Kind::ByteLimit))?
        .checked_sub(fractional as i128)
        .and_then(|e| e.checked_add(trailing as i128))
        .and_then(|e| e.checked_sub(if percentage { 2 } else { 0 }))
        .ok_or_else(|| CssSpecifiedValueSerializationError::new(Kind::ByteLimit))?;
    format_digits(
        significant.bytes().filter(|c| *c != b'.').map(|c| c - b'0'),
        len,
        exponent,
        negative,
        limit,
    )
}

pub(crate) fn format_digits(
    digits: impl Iterator<Item = u8>,
    len: usize,
    exponent: i128,
    negative: bool,
    limit: usize,
) -> Result<String> {
    if len == 0 {
        return if limit == 0 {
            Err(CssSpecifiedValueSerializationError::new(Kind::ByteLimit))
        } else {
            Ok("0".into())
        };
    }
    let point = (len as i128)
        .checked_add(exponent)
        .ok_or_else(|| CssSpecifiedValueSerializationError::new(Kind::ByteLimit))?;
    let length = if point <= 0 {
        (len as i128)
            .checked_add(2)
            .and_then(|n| n.checked_sub(point))
    } else {
        Some(point.max(len as i128) + i128::from(point < len as i128))
    }
    .and_then(|n| n.checked_add(i128::from(negative)))
    .ok_or_else(|| CssSpecifiedValueSerializationError::new(Kind::ByteLimit))?;
    if length > limit as i128 {
        return Err(CssSpecifiedValueSerializationError::new(Kind::ByteLimit));
    }
    let length = usize::try_from(length)
        .map_err(|_| CssSpecifiedValueSerializationError::new(Kind::CapacityOverflow))?;
    if length > isize::MAX as usize {
        return Err(CssSpecifiedValueSerializationError::new(
            Kind::CapacityOverflow,
        ));
    }
    let mut output = String::with_capacity(length);
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

// Rust's shortest round-trip spelling supplies the nearest decimal candidate.
// At an exact decimal midpoint choose the numerically greater candidate, even
// when the host formatter's last-digit tie rule chose the other neighbor.
pub(crate) fn format_binary64(value: f64) -> String {
    debug_assert!(value.is_finite());
    if value == 0.0 {
        return "0".into();
    }
    let negative = value.is_sign_negative();
    let shortest = value.abs().to_string();
    let (mantissa, exponent) = shortest.split_once(['e', 'E']).unwrap_or((&shortest, "0"));
    let fractional = mantissa
        .find('.')
        .map_or(0, |point| mantissa.len() - point - 1);
    let mut exponent =
        exponent.parse::<i32>().expect("binary64 decimal exponent") - fractional as i32;
    let significant = mantissa.trim_end_matches('0');
    let trailing = mantissa.len() - significant.len();
    // Trimming a whole-number suffix is balanced by an exponent shift; a
    // decimal point itself is skipped, never counted as a coefficient digit.
    exponent += trailing as i32;
    let mut coefficient = significant
        .bytes()
        .filter(|c| *c != b'.')
        .fold(0u64, |n, c| n * 10 + u64::from(c - b'0'));
    let next = if negative {
        coefficient - 1
    } else {
        coefficient + 1
    };
    let midpoint = (coefficient + next) * 5;
    if binary64_equals_decimal(value.abs(), midpoint, exponent - 1) {
        coefficient = next;
    }
    while coefficient.is_multiple_of(10) {
        coefficient /= 10;
        exponent += 1;
    }
    let coefficient = coefficient.to_string();
    format_digits(
        coefficient.bytes().map(|c| c - b'0'),
        coefficient.len(),
        i128::from(exponent),
        negative,
        usize::MAX,
    )
    .expect("finite binary64 text has bounded length")
}

fn binary64_equals_decimal(value: f64, mut coefficient: u64, mut exponent: i32) -> bool {
    if coefficient == 0 {
        return false;
    }
    while coefficient.is_multiple_of(10) {
        coefficient /= 10;
        exponent += 1;
    }
    let bits = value.to_bits();
    let encoded = ((bits >> 52) & 0x7ff) as i32;
    let fraction = bits & ((1u64 << 52) - 1);
    let (mut binary_coefficient, binary_exponent) = if encoded == 0 {
        (fraction, -1074)
    } else {
        (fraction | (1u64 << 52), encoded - 1075)
    };
    let mut digits = [0u8; 1100];
    let mut len = 0;
    while binary_coefficient != 0 {
        digits[len] = (binary_coefficient % 10) as u8;
        binary_coefficient /= 10;
        len += 1;
    }
    let multiplier = if binary_exponent < 0 { 5 } else { 2 };
    for _ in 0..binary_exponent.unsigned_abs() {
        let mut carry = 0;
        for digit in &mut digits[..len] {
            let product = *digit * multiplier + carry;
            *digit = product % 10;
            carry = product / 10;
        }
        if carry != 0 {
            digits[len] = carry;
            len += 1;
        }
    }
    let trailing = digits[..len].iter().take_while(|&&d| d == 0).count();
    if binary_exponent.min(0) + trailing as i32 != exponent {
        return false;
    }
    let decimal = coefficient.to_string();
    decimal.len() == len - trailing
        && decimal
            .bytes()
            .map(|c| c - b'0')
            .eq(digits[trailing..len].iter().rev().copied())
}

#[cfg(test)]
mod tests {
    use super::{binary64_equals_decimal, format_binary64};

    #[test]
    fn binary64_text_uses_fixed_notation_and_preserves_retained_value() {
        for (value, expected) in [
            (0.0, "0"),
            (-0.0, "0"),
            (0.1, "0.1"),
            (1.0 / 3.0, "0.3333333333333333"),
            (-1.0 / 3.0, "-0.3333333333333333"),
            (1e20, "100000000000000000000"),
            (1e-7, "0.0000001"),
        ] {
            assert_eq!(format_binary64(value), expected);
        }
        for bits in [
            1,
            2,
            3,
            0x0010_0000_0000_0000,
            0x3fef_ffff_ffff_ffff,
            0x3ff0_0000_0000_0001,
            0x7fef_ffff_ffff_ffff,
        ] {
            for value in [f64::from_bits(bits), -f64::from_bits(bits)] {
                let text = format_binary64(value);
                assert!(!text.contains(['e', 'E']));
                assert_eq!(text.parse::<f64>().unwrap().to_bits(), value.to_bits());
            }
        }
    }

    #[test]
    fn decimal_midpoint_comparison_uses_exact_binary_value() {
        assert!(binary64_equals_decimal(0.125, 125, -3));
        assert!(binary64_equals_decimal(1.5, 150, -2));
        assert!(binary64_equals_decimal(
            9007199254740992.0,
            9007199254740992,
            0
        ));
        assert!(!binary64_equals_decimal(0.1, 1, -1));
        assert!(!binary64_equals_decimal(f64::from_bits(1), 5, -324));
    }
}
