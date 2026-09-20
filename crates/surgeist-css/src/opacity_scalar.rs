//! Exact ordinary opacity scalars and lossless legacy-payload admission.

use crate::{
    CssComponentValue, CssComponentValueError, CssComponentValueErrorKind, CssComponentValueRef,
    CssFiniteNumber, CssNumericTokenRef, CssOpacity, CssOpacityValue, CssValueOrigin,
    CssValueTokenRef,
};

/// The authored numeric token kind of an exact ordinary opacity scalar.
#[non_exhaustive]
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum CssOpacityScalarKind {
    Number,
    Percentage,
}

/// One exact finite decimal number or percentage token for authored opacity.
///
/// The lexical value remains authoritative even when a cached floating-point
/// approximation would overflow or underflow. Equality includes provenance.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CssOpacityScalar {
    component: Box<CssComponentValue>,
}

impl CssOpacityScalar {
    /// Checks the token kind without rounding or imposing a floating-point range.
    pub fn try_from_component(
        component: CssComponentValue,
    ) -> Result<Self, CssComponentValueError> {
        if matches!(
            component.view(),
            CssComponentValueRef::Token(
                CssValueTokenRef::Number(_) | CssValueTokenRef::Percentage(_)
            )
        ) {
            Ok(Self {
                component: Box::new(component),
            })
        } else {
            Err(CssComponentValueError::new(
                CssComponentValueErrorKind::InvalidToken,
                component.origin().clone(),
            ))
        }
    }

    /// Returns whether the authored scalar has a percentage suffix.
    #[must_use]
    pub fn kind(&self) -> CssOpacityScalarKind {
        match self.component.view() {
            CssComponentValueRef::Token(CssValueTokenRef::Number(_)) => {
                CssOpacityScalarKind::Number
            }
            CssComponentValueRef::Token(CssValueTokenRef::Percentage(_)) => {
                CssOpacityScalarKind::Percentage
            }
            _ => unreachable!("checked opacity scalar token"),
        }
    }

    /// Returns the complete numeric spelling, excluding a percentage suffix.
    #[must_use]
    pub fn numeric(&self) -> CssNumericTokenRef<'_> {
        match self.component.view() {
            CssComponentValueRef::Token(
                CssValueTokenRef::Number(value) | CssValueTokenRef::Percentage(value),
            ) => value,
            _ => unreachable!("checked opacity scalar token"),
        }
    }

    /// Borrows the original checked component without replacing its provenance.
    #[must_use]
    pub const fn component(&self) -> &CssComponentValue {
        &self.component
    }

    /// Returns the original parsed or explicitly programmatic token origin.
    #[must_use]
    pub const fn origin(&self) -> &CssValueOrigin {
        self.component.origin()
    }
}

pub(crate) fn admit_opacity_scalar(
    component: CssComponentValue,
) -> Result<CssOpacityValue, CssComponentValueError> {
    let scalar = CssOpacityScalar::try_from_component(component)?;
    if let Some(value) = exact_legacy_value(scalar.numeric().representation()) {
        let finite = CssFiniteNumber::try_new(value).expect("checked finite legacy value");
        return Ok(match scalar.kind() {
            CssOpacityScalarKind::Number => CssOpacity::try_new(value)
                .map(CssOpacityValue::Literal)
                .unwrap_or(CssOpacityValue::Number(finite)),
            CssOpacityScalarKind::Percentage => CssOpacityValue::Percentage(finite),
        });
    }
    Ok(CssOpacityValue::ExactScalar(scalar))
}

/// Normalized borrowed metadata for an already checked finite decimal token.
/// The coefficient remains in the original source regardless of its length.
pub(crate) struct LexicalDecimal<'a> {
    significant: &'a str,
    pub(crate) len: usize,
    pub(crate) exponent: Option<i128>,
    pub(crate) negative: bool,
    pub(crate) exponent_negative: bool,
}
impl<'a> LexicalDecimal<'a> {
    pub(crate) fn new(text: &'a str) -> Self {
        let negative = text.starts_with('-');
        let text = text.strip_prefix(['+', '-']).unwrap_or(text);
        let (mantissa, exponent_text) = text.split_once(['e', 'E']).unwrap_or((text, "0"));
        let Some(first) = mantissa.bytes().position(|c| matches!(c, b'1'..=b'9')) else {
            return Self {
                significant: "",
                len: 0,
                exponent: Some(0),
                negative: false,
                exponent_negative: false,
            };
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
        let exponent = exponent_text
            .parse::<i128>()
            .ok()
            .and_then(|e| e.checked_sub(fractional as i128))
            .and_then(|e| e.checked_add(trailing as i128));
        Self {
            significant,
            len,
            exponent,
            negative,
            exponent_negative: exponent_text.starts_with('-'),
        }
    }
    pub(crate) fn digits(&self) -> impl Iterator<Item = u8> + '_ {
        self.significant
            .bytes()
            .filter(|c| *c != b'.')
            .map(|c| c - b'0')
    }
    pub(crate) fn in_percentage_range(&self) -> bool {
        if self.len == 0 {
            return true;
        }
        if self.negative {
            return false;
        }
        let Some(exponent) = self.exponent else {
            return self.exponent_negative;
        };
        let point = exponent.saturating_add(self.len as i128);
        point < 3 || (point == 3 && self.len == 1 && self.digits().next() == Some(1))
    }
}

// A binary32 exact decimal needs at most 112 significant coefficient digits.
// This fixed bound is used only to prove membership in the legacy subset: a
// larger lexical coefficient remains valid in CssOpacityScalar.
const DIGITS: usize = 192;

#[derive(Debug, Eq, PartialEq)]
struct Decimal {
    digits: [u8; DIGITS],
    len: usize,
    exponent: i128,
    negative: bool,
}

impl Decimal {
    fn zero() -> Self {
        Self {
            digits: [0; DIGITS],
            len: 0,
            exponent: 0,
            negative: false,
        }
    }

    // Input is already a checked CSS numeric representation. Delay trailing
    // zeros so an arbitrarily long redundant suffix does not fill the buffer.
    fn lexical(text: &str) -> Option<Self> {
        let lexical = LexicalDecimal::new(text);
        if lexical.len > DIGITS {
            return None;
        }
        let mut value = Self::zero();
        value.len = lexical.len;
        for (target, digit) in value.digits.iter_mut().zip(lexical.digits()) {
            *target = digit;
        }
        value.exponent = lexical.exponent?;
        value.negative = lexical.negative;
        Some(value)
    }

    fn binary32(value: f32) -> Self {
        let bits = value.to_bits();
        let fraction = bits & 0x7f_ffff;
        let encoded_exponent = (bits >> 23) & 0xff;
        let (mut coefficient, exponent) = if encoded_exponent == 0 {
            (fraction, -149)
        } else {
            (fraction | (1 << 23), encoded_exponent as i32 - 150)
        };
        if coefficient == 0 {
            return Self::zero();
        }
        let mut reversed = [0u8; DIGITS];
        let mut len = 0;
        while coefficient != 0 {
            reversed[len] = (coefficient % 10) as u8;
            coefficient /= 10;
            len += 1;
        }
        let multiplier = if exponent < 0 { 5 } else { 2 };
        for _ in 0..exponent.unsigned_abs() {
            let mut carry = 0;
            for digit in &mut reversed[..len] {
                let product = *digit * multiplier + carry;
                *digit = product % 10;
                carry = product / 10;
            }
            if carry != 0 {
                reversed[len] = carry;
                len += 1;
            }
        }
        let trailing = reversed[..len]
            .iter()
            .take_while(|&&digit| digit == 0)
            .count();
        let mut result = Self::zero();
        result.len = len - trailing;
        for (target, source) in result.digits[..result.len]
            .iter_mut()
            .zip(reversed[trailing..len].iter().rev())
        {
            *target = *source;
        }
        result.exponent = i128::from(exponent.min(0)) + trailing as i128;
        result.negative = bits >> 31 != 0;
        result
    }
}

pub(crate) fn serialize_binary32(
    value: f32,
    percentage: bool,
    limit: usize,
) -> Result<String, crate::CssSpecifiedValueSerializationError> {
    let value = Decimal::binary32(value);
    crate::specified_serialization::format_digits(
        value.digits[..value.len].iter().copied(),
        value.len,
        value.exponent - if percentage { 2 } else { 0 },
        value.negative,
        limit,
    )
}

// Exact equality left * numerator / denominator == right, for finite binary32
// operands and small fixed color-space bases. No floating-point arithmetic.
pub(crate) fn binary32_scaled_eq(left: f32, numerator: u32, denominator: u32, right: f32) -> bool {
    fn multiply(mut value: Decimal, factor: u32) -> Option<Decimal> {
        if value.len == 0 || factor == 0 {
            return Some(Decimal::zero());
        }
        let mut carry = 0_u64;
        for digit in value.digits[..value.len].iter_mut().rev() {
            let product = u64::from(*digit) * u64::from(factor) + carry;
            *digit = (product % 10) as u8;
            carry = product / 10;
        }
        while carry != 0 {
            if value.len == DIGITS {
                return None;
            }
            value.digits.copy_within(..value.len, 1);
            value.digits[0] = (carry % 10) as u8;
            value.len += 1;
            carry /= 10;
        }
        while value.digits[value.len - 1] == 0 {
            value.len -= 1;
            value.exponent += 1;
        }
        Some(value)
    }
    left.is_finite()
        && right.is_finite()
        && denominator != 0
        && multiply(Decimal::binary32(left), numerator)
            .zip(multiply(Decimal::binary32(right), denominator))
            .is_some_and(|(a, b)| a == b)
}

// Shared exact-fidelity proof for ordinary numeric consumers. The input must be
// a checked CSS numeric representation, without a unit or percentage suffix.
pub(crate) fn exact_legacy_value(text: &str) -> Option<f32> {
    let decimal = Decimal::lexical(text)?;
    if decimal.len == 0 {
        return Some(0.0);
    }
    let candidate = text.parse::<f32>().ok()?;
    (candidate.is_finite() && decimal == Decimal::binary32(candidate)).then_some(candidate)
}

#[cfg(test)]
mod tests {
    use super::exact_legacy_value;

    #[test]
    fn equality_uses_exact_decimal_value_instead_of_rounded_candidate() {
        for (text, expected) in [
            (".5", 0.5),
            ("-1.50", -1.5),
            ("150", 150.0),
            ("000.50000e+0", 0.5),
        ] {
            assert_eq!(exact_legacy_value(text), Some(expected));
        }
        for text in [".1", "1e100", "-1e100", "1e-47", "-1e-47", "16777217"] {
            assert_eq!(exact_legacy_value(text), None, "{text}");
        }
        assert_eq!(
            exact_legacy_value("0.100000001490116119384765625"),
            Some(0.1)
        );
        assert_eq!(
            exact_legacy_value("340282346638528859811704183484516925440"),
            Some(f32::MAX)
        );
        assert_eq!(
            exact_legacy_value(
                "0.00000000000000000000000000000000000000000000140129846432481707092372958328991613128026194187651577175706828388979108268586060148663818836212158203125"
            ),
            Some(f32::from_bits(1))
        );
    }

    #[test]
    fn redundant_digits_and_large_exponents_do_not_require_large_numeric_storage() {
        assert_eq!(
            exact_legacy_value("-0e9999999999999999999999999999999999999999999"),
            Some(0.0)
        );
        assert_eq!(
            exact_legacy_value("1e9999999999999999999999999999999999999999999"),
            None
        );
        assert_eq!(
            exact_legacy_value(&format!("0.5{}", "0".repeat(4096))),
            Some(0.5)
        );
        assert_eq!(
            exact_legacy_value(&format!("1{}e-4096", "0".repeat(4096))),
            Some(1.0)
        );
    }
}
