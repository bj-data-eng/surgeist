//! Exact ordinary signed integers and specified integer-value serialization.

use crate::{
    CssComponentValue, CssComponentValueError, CssComponentValueErrorKind, CssComponentValueRef,
    CssIntegerValue, CssNumericTokenKind, CssNumericTokenRef, CssSpecifiedValueSerializationError,
    CssSpecifiedValueSerializationErrorKind, CssSpecifiedValueSerializationLimits, CssValueOrigin,
    CssValueTokenRef,
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
}

pub(crate) fn admit_integer_literal(
    component: CssComponentValue,
) -> Result<CssIntegerValue, CssComponentValueError> {
    let literal = CssIntegerLiteral::try_from_component(component)?;
    Ok(match exact_i32(literal.numeric().representation()) {
        Some(value) => CssIntegerValue::Literal(value),
        None => CssIntegerValue::ExactLiteral(literal),
    })
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
    /// binary64 math policy while retaining its exact authored expression.
    pub fn serialize_specified(&self) -> Result<String, CssSpecifiedValueSerializationError> {
        self.serialize_specified_with_limits(CssSpecifiedValueSerializationLimits::default())
    }

    /// Projects atomically with the shared input, projection and output budgets.
    /// Ordinary integers charge one input and one projection node.
    pub fn serialize_specified_with_limits(
        &self,
        limits: CssSpecifiedValueSerializationLimits,
    ) -> Result<String, CssSpecifiedValueSerializationError> {
        if let Self::Calculation(calculation) = self {
            return crate::numeric::project_specified(&calculation.expression, limits);
        }
        use CssSpecifiedValueSerializationErrorKind as Kind;
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
            Self::Literal(value) => {
                // The entire i32 decimal representation fits on the stack.
                let mut buffer = [b'0'; 11];
                let mut start = buffer.len();
                let mut magnitude = value.unsigned_abs();
                loop {
                    start -= 1;
                    buffer[start] = b'0' + (magnitude % 10) as u8;
                    magnitude /= 10;
                    if magnitude == 0 {
                        break;
                    }
                }
                if *value < 0 {
                    start -= 1;
                    buffer[start] = b'-';
                }
                serialize_integer_digits(
                    std::str::from_utf8(&buffer[start..]).expect("ASCII integer digits"),
                    limits.max_css_bytes(),
                )
            }
            Self::ExactLiteral(literal) => {
                serialize_integer_digits(literal.numeric().representation(), limits.max_css_bytes())
            }
            Self::Calculation(_) => unreachable!("calculation handled above"),
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
