//! Exact ordinary opacity scalars and lossless legacy-payload admission.

use crate::exact_decimal::exact_legacy_value;

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
