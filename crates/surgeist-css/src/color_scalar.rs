//! Exact ordinary color token domains.
use crate::{
    CssAngleLiteral, CssColorComponent, CssColorHue, CssComponentValue, CssComponentValueError,
    CssComponentValueErrorKind, CssComponentValueRef, CssNumericTokenRef, CssValueOrigin,
    CssValueTokenRef,
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
) -> Result<CssColorComponent, CssComponentValueError> {
    match value.view() {
        CssComponentValueRef::Token(CssValueTokenRef::Number(_)) => Ok(CssColorComponent::Number(
            CssColorNumberLiteral::try_from_component(value)?,
        )),
        CssComponentValueRef::Token(CssValueTokenRef::Percentage(_)) => Ok(
            CssColorComponent::Percentage(CssColorPercentageLiteral::try_from_component(value)?),
        ),
        _ => Err(invalid(&value)),
    }
}
pub(crate) fn hue(value: CssComponentValue) -> Result<CssColorHue, CssComponentValueError> {
    if matches!(
        value.view(),
        CssComponentValueRef::Token(CssValueTokenRef::Number(_))
    ) {
        return Ok(CssColorHue::Number(
            CssColorNumberLiteral::try_from_component(value)?,
        ));
    }
    Ok(CssColorHue::Angle(CssAngleLiteral::try_from_component(
        value,
    )?))
}

#[cfg(test)]
mod tests {
    use crate::specified_serialization::format_coefficient;
    #[test]
    fn exact_weight_range_uses_all_significant_digits() {
        for text in [
            "0",
            "-0e99999999999999999999999999999999999999",
            "100",
            "1e-99999999999999999999999999999999999999",
        ] {
            assert!(
                crate::exact_decimal::LexicalDecimal::new(text).in_percentage_range(),
                "{text}"
            );
        }
        for text in [
            "-1e-99999999999999999999999999999999999999",
            "1e99999999999999999999999999999999999999",
            "100.0000000000000000000001",
        ] {
            assert!(
                !crate::exact_decimal::LexicalDecimal::new(text).in_percentage_range(),
                "{text}"
            );
        }
        let equal = format!("1{}e-{}", "0".repeat(4096), 4094);
        assert!(crate::exact_decimal::LexicalDecimal::new(&equal).in_percentage_range());
        let above = format!("100.{}1", "0".repeat(4096));
        assert!(!crate::exact_decimal::LexicalDecimal::new(&above).in_percentage_range());
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
        assert_eq!(format_coefficient("1e-47", 0, "", 49).unwrap(), "0");
    }
}
