//! Exact ordinary color token domains and conservative frozen-value proofs.
use crate::{
    CssAngleLiteral, CssAngleUnit, CssAuthoredColorComponent, CssAuthoredHue, CssComponentValue,
    CssComponentValueError, CssComponentValueErrorKind, CssComponentValueRef, CssFiniteNumber,
    CssNumericTokenRef, CssValueOrigin, CssValueTokenRef,
};

macro_rules! literal {
    ($name:ident, $variant:ident, $doc:literal) => {
        #[doc = $doc]
        #[derive(Clone, Debug, Eq, PartialEq)]
        pub struct $name {
            component: Box<CssComponentValue>,
        }
        impl $name {
            /// Checks the original token domain without floating-point conversion.
            pub fn try_from_component(
                component: CssComponentValue,
            ) -> Result<Self, CssComponentValueError> {
                if matches!(
                    component.view(),
                    CssComponentValueRef::Token(CssValueTokenRef::$variant(_))
                ) {
                    Ok(Self {
                        component: Box::new(component),
                    })
                } else {
                    Err(invalid(&component))
                }
            }
            /// Returns the original finite decimal coefficient spelling.
            pub fn numeric(&self) -> CssNumericTokenRef<'_> {
                let CssComponentValueRef::Token(CssValueTokenRef::$variant(value)) =
                    self.component.view()
                else {
                    unreachable!("checked color token domain")
                };
                value
            }
            pub const fn component(&self) -> &CssComponentValue {
                &self.component
            }
            pub const fn origin(&self) -> &CssValueOrigin {
                self.component.origin()
            }
        }
    };
}
literal!(
    CssColorNumberLiteral,
    Number,
    "An exact ordinary color number token, including original provenance."
);
literal!(
    CssColorPercentageLiteral,
    Percentage,
    "An exact ordinary color percentage token; its coefficient is unscaled."
);

/// An exact ordinary color angle token, without hue normalization.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CssColorAngleLiteral {
    component: Box<CssComponentValue>,
    unit: CssAngleUnit,
}
impl CssColorAngleLiteral {
    pub fn try_from_component(
        component: CssComponentValue,
    ) -> Result<Self, CssComponentValueError> {
        let CssComponentValueRef::Token(CssValueTokenRef::Dimension { unit, .. }) =
            component.view()
        else {
            return Err(invalid(&component));
        };
        let unit = match unit.to_ascii_lowercase().as_str() {
            "deg" => CssAngleUnit::Degrees,
            "grad" => CssAngleUnit::Gradians,
            "rad" => CssAngleUnit::Radians,
            "turn" => CssAngleUnit::Turns,
            _ => return Err(invalid(&component)),
        };
        Ok(Self {
            component: Box::new(component),
            unit,
        })
    }
    pub fn numeric(&self) -> CssNumericTokenRef<'_> {
        let CssComponentValueRef::Token(CssValueTokenRef::Dimension { number, .. }) =
            self.component.view()
        else {
            unreachable!("checked color angle")
        };
        number
    }
    pub const fn unit(&self) -> CssAngleUnit {
        self.unit
    }
    pub const fn component(&self) -> &CssComponentValue {
        &self.component
    }
    pub const fn origin(&self) -> &CssValueOrigin {
        self.component.origin()
    }
}
fn invalid(component: &CssComponentValue) -> CssComponentValueError {
    CssComponentValueError::new(
        CssComponentValueErrorKind::InvalidToken,
        component.origin().clone(),
    )
}

/// A checked color scalar construction failure.
#[non_exhaustive]
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum CssColorScalarErrorKind {
    InvalidComponent,
    OutOfRange,
}
/// A range or component failure retaining the actual supplied token origin.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CssColorScalarError {
    kind: CssColorScalarErrorKind,
    origin: CssValueOrigin,
    source: Option<Box<CssComponentValueError>>,
}
impl CssColorScalarError {
    pub const fn kind(&self) -> CssColorScalarErrorKind {
        self.kind
    }
    pub const fn origin(&self) -> &CssValueOrigin {
        &self.origin
    }
    pub(crate) fn out_of_range(origin: CssValueOrigin) -> Self {
        Self {
            kind: CssColorScalarErrorKind::OutOfRange,
            origin,
            source: None,
        }
    }
}
impl From<CssComponentValueError> for CssColorScalarError {
    fn from(source: CssComponentValueError) -> Self {
        Self {
            kind: CssColorScalarErrorKind::InvalidComponent,
            origin: source.origin().clone(),
            source: Some(Box::new(source)),
        }
    }
}
impl std::fmt::Display for CssColorScalarError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "invalid color scalar: {:?}", self.kind)
    }
}
impl std::error::Error for CssColorScalarError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        self.source.as_deref().map(|e| e as &dyn std::error::Error)
    }
}

pub(crate) fn component(
    value: CssComponentValue,
) -> Result<CssAuthoredColorComponent, CssComponentValueError> {
    match value.view() {
        CssComponentValueRef::Token(CssValueTokenRef::Number(_)) => {
            let value = CssColorNumberLiteral::try_from_component(value)?;
            Ok(
                match crate::opacity_scalar::exact_legacy_value(value.numeric().representation()) {
                    Some(n) => CssAuthoredColorComponent::Number(
                        CssFiniteNumber::try_new(n).expect("proved finite"),
                    ),
                    None => CssAuthoredColorComponent::ExactNumber(value),
                },
            )
        }
        CssComponentValueRef::Token(CssValueTokenRef::Percentage(_)) => {
            let value = CssColorPercentageLiteral::try_from_component(value)?;
            Ok(
                match crate::opacity_scalar::exact_legacy_value(value.numeric().representation()) {
                    Some(n) => CssAuthoredColorComponent::Percentage(
                        CssFiniteNumber::try_new(n).expect("proved finite"),
                    ),
                    None => CssAuthoredColorComponent::ExactPercentage(value),
                },
            )
        }
        _ => Err(invalid(&value)),
    }
}
pub(crate) fn hue(value: CssComponentValue) -> Result<CssAuthoredHue, CssComponentValueError> {
    if matches!(
        value.view(),
        CssComponentValueRef::Token(CssValueTokenRef::Number(_))
    ) {
        return component(value).map(|v| match v {
            CssAuthoredColorComponent::Number(n) => CssAuthoredHue::Number(n),
            CssAuthoredColorComponent::ExactNumber(n) => CssAuthoredHue::ExactNumber(n),
            _ => unreachable!("number admission"),
        });
    }
    let value = CssColorAngleLiteral::try_from_component(value)?;
    Ok(
        match crate::opacity_scalar::exact_legacy_value(value.numeric().representation()) {
            Some(n) => CssAuthoredHue::Angle(
                CssAngleLiteral::try_new(n, value.unit()).expect("proved finite angle"),
            ),
            None => CssAuthoredHue::ExactAngle(value),
        },
    )
}

/// Formats a checked color coefficient without imposing opacity's ratio scale.
/// Suffix bytes belong to this operation's budget, not a later unchecked append.
#[cfg_attr(not(test), allow(dead_code))] // Used by the subsequent shared color serializer.
pub(crate) fn format_coefficient(
    text: &str,
    shift: i128,
    suffix: &str,
    limit: usize,
) -> Result<String, crate::CssSpecifiedValueSerializationError> {
    let remaining = limit.checked_sub(suffix.len()).ok_or_else(|| {
        crate::CssSpecifiedValueSerializationError::new(
            crate::CssSpecifiedValueSerializationErrorKind::ByteLimit,
        )
    })?;
    let mut result = crate::specified_serialization::format_lexical_shift(text, shift, remaining)?;
    result.push_str(suffix);
    Ok(result)
}

pub(crate) fn channel_matches(
    current: &CssAuthoredColorComponent,
    candidate: Option<f32>,
    percentage_numerator: u32,
    percentage_denominator: u32,
) -> bool {
    match current {
        CssAuthoredColorComponent::None => candidate.is_none(),
        CssAuthoredColorComponent::Number(value) => candidate == Some(value.value()),
        CssAuthoredColorComponent::Percentage(value) => candidate.is_some_and(|candidate| {
            crate::opacity_scalar::binary32_scaled_eq(
                value.value(),
                percentage_numerator,
                percentage_denominator,
                candidate,
            )
        }),
        CssAuthoredColorComponent::ExactNumber(_)
        | CssAuthoredColorComponent::ExactPercentage(_)
        | CssAuthoredColorComponent::NumberCalculation(_)
        | CssAuthoredColorComponent::PercentageCalculation(_) => false,
    }
}
pub(crate) fn alpha_matches(
    current: Option<&CssAuthoredColorComponent>,
    candidate: Option<f32>,
) -> bool {
    current.map_or(candidate == Some(1.0), |current| {
        channel_matches(current, candidate, 1, 100)
    })
}
pub(crate) fn hue_matches(current: &CssAuthoredHue, candidate: Option<f32>) -> bool {
    match current {
        CssAuthoredHue::None => candidate.is_none(),
        CssAuthoredHue::Number(value) => candidate == Some(value.value()),
        CssAuthoredHue::Angle(value) => candidate.is_some_and(|candidate| match value.unit() {
            CssAngleUnit::Degrees => candidate == value.value(),
            CssAngleUnit::Gradians => {
                crate::opacity_scalar::binary32_scaled_eq(value.value(), 9, 10, candidate)
            }
            CssAngleUnit::Turns => {
                crate::opacity_scalar::binary32_scaled_eq(value.value(), 360, 1, candidate)
            }
            CssAngleUnit::Radians => value.value() == 0.0 && candidate == 0.0,
        }),
        CssAuthoredHue::ExactNumber(_)
        | CssAuthoredHue::ExactAngle(_)
        | CssAuthoredHue::NumberCalculation(_)
        | CssAuthoredHue::AngleCalculation(_) => false,
    }
}
pub(crate) fn relative_matches(
    current: &crate::CssTypedRelativeColorExpression,
    candidate: &crate::CssColorComponentExpression,
) -> bool {
    use crate::CssRelativeColorExpressionValue as Value;
    if let Value::Calculation(value) = current.value() {
        return value.authored().as_css() == candidate.authored().as_css();
    }
    let Ok(token) = CssComponentValue::try_token(candidate.authored().as_css()) else {
        return false;
    };
    match (current.value(), token.view()) {
        (Value::None, CssComponentValueRef::Token(CssValueTokenRef::Ident(name))) => {
            name.eq_ignore_ascii_case("none")
        }
        (Value::Number(value), CssComponentValueRef::Token(CssValueTokenRef::Number(number)))
        | (
            Value::Percentage(value),
            CssComponentValueRef::Token(CssValueTokenRef::Percentage(number)),
        ) => {
            crate::opacity_scalar::exact_legacy_value(number.representation())
                == Some(value.value())
        }
        (
            Value::Angle(value),
            CssComponentValueRef::Token(CssValueTokenRef::Dimension { number, unit }),
        ) => {
            let expected = match value.unit() {
                CssAngleUnit::Degrees => "deg",
                CssAngleUnit::Gradians => "grad",
                CssAngleUnit::Radians => "rad",
                CssAngleUnit::Turns => "turn",
            };
            unit.eq_ignore_ascii_case(expected)
                && crate::opacity_scalar::exact_legacy_value(number.representation())
                    == Some(value.value())
        }
        (Value::Channel(channel), CssComponentValueRef::Token(CssValueTokenRef::Ident(name))) => {
            use crate::CssRelativeColorChannel::*;
            let expected = match channel {
                R => "r",
                G => "g",
                B => "b",
                H => "h",
                S => "s",
                L => "l",
                W => "w",
                A => "a",
                C => "c",
                X => "x",
                Y => "y",
                Z => "z",
                Alpha => "alpha",
            };
            name.eq_ignore_ascii_case(expected)
        }
        _ => false,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn exact_weight_range_uses_all_significant_digits() {
        for text in [
            "0",
            "-0e99999999999999999999999999999999999999",
            "100",
            "1e-99999999999999999999999999999999999999",
        ] {
            assert!(
                crate::opacity_scalar::LexicalDecimal::new(text).in_percentage_range(),
                "{text}"
            );
        }
        for text in [
            "-1e-99999999999999999999999999999999999999",
            "1e99999999999999999999999999999999999999",
            "100.0000000000000000000001",
        ] {
            assert!(
                !crate::opacity_scalar::LexicalDecimal::new(text).in_percentage_range(),
                "{text}"
            );
        }
        let equal = format!("1{}e-{}", "0".repeat(4096), 4094);
        assert!(crate::opacity_scalar::LexicalDecimal::new(&equal).in_percentage_range());
        let above = format!("100.{}1", "0".repeat(4096));
        assert!(!crate::opacity_scalar::LexicalDecimal::new(&above).in_percentage_range());
    }
    #[test]
    fn coefficient_formatting_has_explicit_scale_and_suffix_budget() {
        assert_eq!(format_coefficient("0.1", 0, "%", 4).unwrap(), "0.1%");
        assert_eq!(format_coefficient("0.1", -2, "", 5).unwrap(), "0.001");
        assert_eq!(
            format_coefficient("-0e999999999999999999999999999", 0, "%", 2).unwrap(),
            "0%"
        );
        assert!(format_coefficient("0.1", 0, "%", 3).is_err());
        assert!(format_coefficient("1e999999999999999999999999999", 0, "", 100).is_err());
        assert_eq!(
            format_coefficient("1e-47", 0, "", 49).unwrap(),
            format!("0.{}1", "0".repeat(46))
        );
    }
    #[test]
    fn frozen_ratio_proofs_do_not_round_or_underflow() {
        use crate::opacity_scalar::binary32_scaled_eq as equal;
        assert!(equal(25.0, 1, 100, 0.25));
        assert!(equal(100.0, 1, 1, 100.0));
        assert!(equal(20.0, 5, 4, 25.0));
        assert!(!equal(100.0, 1, 250, 0.4));
        assert!(!equal(f32::from_bits(1), 1, 100, 0.0));
        let coefficient = f32::from_bits(0x4248_0001);
        assert!(!equal(coefficient, 1, 100, coefficient / 100.0));
        assert!(equal(-0.0, 1, 100, 0.0));
    }
}
