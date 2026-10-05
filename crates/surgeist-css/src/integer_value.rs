//! Exact ordinary signed integers and specified integer/z-index serialization.

use std::cmp::Ordering;

use crate::{
    CssComponentValue, CssComponentValueError, CssComponentValueErrorKind, CssComponentValueRef,
    CssIntegerValue, CssNumericTokenKind, CssNumericTokenRef, CssSpecifiedValueSerializationError,
    CssSpecifiedValueSerializationErrorKind, CssSpecifiedValueSerializationLimits, CssValueOrigin,
    CssValueTokenRef, CssZIndexValue, specified_serialization::SpecifiedSerializationContext,
};

/// One checked lexical integer token, without a machine-integer magnitude bound.
///
/// Original sign, digits and provenance remain authoritative. Equality includes
/// spelling and provenance, rather than comparing only mathematical magnitude.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CssIntegerLiteral {
    component: Box<CssComponentValue>,
}

impl CssIntegerLiteral {
    /// Accepts only a number token with integer syntax (no point or exponent).
    pub fn try_from_component(
        component: CssComponentValue,
    ) -> Result<Self, CssComponentValueError> {
        if matches!(component.view(),
            CssComponentValueRef::Token(CssValueTokenRef::Number(number))
                if number.kind() == CssNumericTokenKind::Integer)
        {
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

    /// Constructs the canonical programmatic spelling of a machine integer.
    #[must_use]
    pub fn from_i32(value: i32) -> Self {
        Self::try_from_component(
            CssComponentValue::try_number(&value.to_string()).expect("valid integer spelling"),
        )
        .expect("checked integer token")
    }

    /// Whether this lexical integer is mathematically zero, regardless of sign.
    #[must_use]
    pub fn is_zero(&self) -> bool {
        self.numeric()
            .representation()
            .strip_prefix(['+', '-'])
            .unwrap_or(self.numeric().representation())
            .bytes()
            .all(|digit| digit == b'0')
    }

    /// Whether this lexical integer is mathematically negative; signed zero is not negative.
    #[must_use]
    pub fn is_negative(&self) -> bool {
        self.numeric().representation().starts_with('-') && !self.is_zero()
    }

    /// Compares mathematical integer values without changing authored identity.
    ///
    /// Explicit plus signs, leading zeroes and provenance do not affect this
    /// ordering. Negative values precede zero and positive values; signed zeroes
    /// compare equally. Equality remains sensitive to spelling and origin.
    #[must_use]
    pub fn compare_value(&self, other: &Self) -> Ordering {
        fn significant_digits(value: &CssIntegerLiteral) -> &str {
            let text = value.numeric().representation();
            text.strip_prefix(['+', '-'])
                .unwrap_or(text)
                .trim_start_matches('0')
        }
        let left = significant_digits(self);
        let right = significant_digits(other);
        let left_negative = self.numeric().representation().starts_with('-') && !left.is_empty();
        let right_negative = other.numeric().representation().starts_with('-') && !right.is_empty();
        match (left_negative, right_negative) {
            (true, false) => Ordering::Less,
            (false, true) => Ordering::Greater,
            _ => {
                let magnitude = left.len().cmp(&right.len()).then_with(|| left.cmp(right));
                if left_negative {
                    magnitude.reverse()
                } else {
                    magnitude
                }
            }
        }
    }

    /// Returns the original sign and digits without numeric approximation.
    #[must_use]
    pub fn numeric(&self) -> CssNumericTokenRef<'_> {
        match self.component.view() {
            CssComponentValueRef::Token(CssValueTokenRef::Number(number)) => number,
            _ => unreachable!("checked integer number token"),
        }
    }

    /// Borrows the original checked component, including its provenance.
    #[must_use]
    pub const fn component(&self) -> &CssComponentValue {
        &self.component
    }

    /// Returns the original parsed or explicitly programmatic token origin.
    #[must_use]
    pub const fn origin(&self) -> &CssValueOrigin {
        self.component.origin()
    }

    pub(crate) fn append_specified(
        &self,
        context: &mut SpecifiedSerializationContext,
        output: &mut String,
    ) -> Result<(), CssSpecifiedValueSerializationError> {
        context.charge_input(1)?;
        context.charge_projection(1)?;
        let text =
            serialize_integer_digits(self.numeric().representation(), context.remaining_bytes())?;
        context.append(output, &text)
    }
}

pub(crate) fn admit_integer_literal(
    component: CssComponentValue,
) -> Result<CssIntegerValue, CssComponentValueError> {
    let literal = CssIntegerLiteral::try_from_component(component)?;
    Ok(CssIntegerValue::Literal(literal))
}

// Input has already been checked as a lexical integer. Accumulating negatively
// admits i32::MIN without an overflowing positive intermediate. Leading zeroes
// never consume an artificial significant-digit allowance.
pub(crate) fn exact_i32(text: &str) -> Option<i32> {
    let negative = text.starts_with('-');
    let digits = text.strip_prefix(['+', '-']).unwrap_or(text);
    let mut value = 0_i32;
    for digit in digits.bytes() {
        value = value
            .checked_mul(10)?
            .checked_sub(i32::from(digit - b'0'))?;
    }
    if negative {
        Some(value)
    } else {
        value.checked_neg()
    }
}

impl CssIntegerValue {
    /// Produces canonical specified text without computed integer rounding.
    ///
    /// Ordinary literal magnitudes are exact. Calculation projection, including
    /// an explicitly constructed literal-only calculation root, uses the shared
    /// binary64 math policy while retaining its exact authored expression. Finite
    /// results emit at most six fractional places, nearest with ties away from
    /// zero, and preserve all exact integer digits of the binary64 result.
    pub fn serialize_specified(&self) -> Result<String, CssSpecifiedValueSerializationError> {
        self.serialize_specified_with_limits(CssSpecifiedValueSerializationLimits::default())
    }

    /// Projects atomically with the shared input, projection and output budgets.
    /// Ordinary integers charge one input and one projection node.
    pub fn serialize_specified_with_limits(
        &self,
        limits: CssSpecifiedValueSerializationLimits,
    ) -> Result<String, CssSpecifiedValueSerializationError> {
        let mut context = SpecifiedSerializationContext::new(limits);
        let mut output = String::new();
        self.append_specified(&mut context, &mut output)?;
        Ok(output)
    }

    /// Appends one integer to a caller's cumulative specified-CSS budget.
    pub(crate) fn append_specified(
        &self,
        context: &mut SpecifiedSerializationContext,
        output: &mut String,
    ) -> Result<(), CssSpecifiedValueSerializationError> {
        if let Self::Calculation(calculation) = self {
            crate::numeric::project_specified_into(&calculation.expression, context, output)?;
            return Ok(());
        }
        match self {
            Self::Literal(literal) => literal.append_specified(context, output),
            Self::Calculation(_) => unreachable!("calculation handled above"),
        }
    }
}

impl CssZIndexValue {
    /// Produces canonical specified `auto` or integer text, retaining authored identity.
    ///
    /// Integer literals retain exact magnitude. Math uses the shared specified
    /// projection without computed integer rounding or contextual stacking policy.
    pub fn serialize_specified(&self) -> Result<String, CssSpecifiedValueSerializationError> {
        self.serialize_specified_with_limits(CssSpecifiedValueSerializationLimits::default())
    }

    /// Serializes atomically under shared input, projection and UTF-8 byte budgets.
    /// `auto` charges one input and one projection node. Integer values use the
    /// integer writer's accounting without an additional wrapper charge.
    pub fn serialize_specified_with_limits(
        &self,
        limits: CssSpecifiedValueSerializationLimits,
    ) -> Result<String, CssSpecifiedValueSerializationError> {
        let mut context = SpecifiedSerializationContext::new(limits);
        let mut output = String::new();
        self.append_specified(&mut context, &mut output)?;
        Ok(output)
    }

    pub(crate) fn append_specified(
        &self,
        context: &mut SpecifiedSerializationContext,
        output: &mut String,
    ) -> Result<(), CssSpecifiedValueSerializationError> {
        match self {
            Self::Auto => {
                context.charge_input(1)?;
                context.charge_projection(1)?;
                context.append(output, "auto")
            }
            Self::Integer(value) => value.append_specified(context, output),
        }
    }
}

fn serialize_integer_digits(
    text: &str,
    byte_limit: usize,
) -> Result<String, CssSpecifiedValueSerializationError> {
    use CssSpecifiedValueSerializationErrorKind as Kind;
    let negative = text.starts_with('-');
    let digits = text
        .strip_prefix(['+', '-'])
        .unwrap_or(text)
        .trim_start_matches('0');
    let (digits, negative) = if digits.is_empty() {
        ("0", false)
    } else {
        (digits, negative)
    };
    let length = digits
        .len()
        .checked_add(usize::from(negative))
        .ok_or_else(|| CssSpecifiedValueSerializationError::new(Kind::CapacityOverflow))?;
    if length > byte_limit {
        return Err(CssSpecifiedValueSerializationError::new(Kind::ByteLimit));
    }
    let mut output = String::with_capacity(length);
    if negative {
        output.push('-');
    }
    output.push_str(digits);
    Ok(output)
}

#[cfg(test)]
mod z_index_serialization_tests {
    use super::*;
    use crate::CssIntegerCalculation;
    use CssSpecifiedValueSerializationErrorKind as Kind;

    #[test]
    fn z_index_siblings_share_input_projection_and_utf8_byte_budgets() {
        let first = CssZIndexValue::Auto;
        let second =
            CssZIndexValue::Integer(CssIntegerValue::Literal(CssIntegerLiteral::from_i32(-2)));
        let before = (first.clone(), second.clone());
        // The separator is a UTF-8 accounting probe in the shared private writer,
        // not a proposed CSS property grammar. "autoé-2" occupies eight bytes.
        for (limits, failure) in [
            (CssSpecifiedValueSerializationLimits::new(2, 2, 8), None),
            (
                CssSpecifiedValueSerializationLimits::new(1, 2, 8),
                Some(Kind::InputNodeLimit),
            ),
            (
                CssSpecifiedValueSerializationLimits::new(2, 1, 8),
                Some(Kind::ProjectionNodeLimit),
            ),
            (
                CssSpecifiedValueSerializationLimits::new(2, 2, 7),
                Some(Kind::ByteLimit),
            ),
        ] {
            let mut context = SpecifiedSerializationContext::new(limits);
            let mut output = String::new();
            first.append_specified(&mut context, &mut output).unwrap();
            context.append(&mut output, "é").unwrap();
            let result = second.append_specified(&mut context, &mut output);
            if let Some(kind) = failure {
                assert_eq!(result.unwrap_err().kind(), kind);
                assert_eq!(output, "autoé");
            } else {
                result.unwrap();
                assert_eq!(output, "autoé-2");
            }
            assert_eq!((&first, &second), (&before.0, &before.1));
        }
    }

    #[test]
    fn z_index_math_delegates_into_the_existing_cumulative_context() {
        let first = CssZIndexValue::Auto;
        let second = CssZIndexValue::Integer(CssIntegerValue::Calculation(
            CssIntegerCalculation::literal(2),
        ));
        let before = second.clone();
        for (limits, failure) in [
            (
                CssSpecifiedValueSerializationLimits::new(100, 100, 12),
                None,
            ),
            (
                CssSpecifiedValueSerializationLimits::new(1, 100, 12),
                Some(Kind::InputNodeLimit),
            ),
            (
                CssSpecifiedValueSerializationLimits::new(100, 1, 12),
                Some(Kind::ProjectionNodeLimit),
            ),
            (
                CssSpecifiedValueSerializationLimits::new(100, 100, 11),
                Some(Kind::ByteLimit),
            ),
        ] {
            let mut context = SpecifiedSerializationContext::new(limits);
            let mut output = String::new();
            first.append_specified(&mut context, &mut output).unwrap();
            context.append(&mut output, " ").unwrap();
            let result = second.append_specified(&mut context, &mut output);
            if let Some(kind) = failure {
                assert_eq!(result.unwrap_err().kind(), kind);
                // Internal math appends incrementally. Public serialization
                // discards this scratch output on error; it does not promise
                // rollback of a caller's private buffer.
                assert!(output.starts_with("auto "));
                assert!(output.len() <= limits.max_css_bytes());
                assert_eq!(
                    context.remaining_bytes(),
                    limits.max_css_bytes() - output.len()
                );
            } else {
                result.unwrap();
                assert_eq!(output, "auto calc(2)");
            }
            assert_eq!(second, before);
        }
    }
}
