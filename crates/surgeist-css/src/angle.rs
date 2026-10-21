//! Exact authored ordinary angles and the grammar-specific number-zero exception.
use crate::{
    CssAngleCalculation, CssAngleUnit, CssComponentValue, CssComponentValueError,
    CssComponentValueErrorKind, CssComponentValueRef, CssNumericConstructionError,
    CssNumericTokenRef, CssValueOrigin, CssValueTokenRef,
};

pub(crate) const fn suffix(unit: CssAngleUnit) -> &'static str {
    crate::numeric::units::suffix(crate::CssNumericUnit::Angle(unit))
}
fn invalid(component: &CssComponentValue) -> CssComponentValueError {
    CssComponentValueError::new(
        CssComponentValueErrorKind::InvalidToken,
        component.origin().clone(),
    )
}

crate::dimension::dimension_literal_owner!(
    /// An exact ordinary angle dimension, retaining spelling, unit and provenance.
    CssAngleLiteral,
    CssAngleUnit,
    suffix
);
impl CssAngleLiteral {
    pub(crate) fn is_default_gradient_direction(&self) -> bool {
        let decimal = crate::exact_decimal::LexicalDecimal::new(self.numeric().representation());
        let (digits, exponent) = match self.unit() {
            CssAngleUnit::Degrees => ("18", 1),
            CssAngleUnit::Gradians => ("2", 2),
            CssAngleUnit::Turns => ("5", -1),
            CssAngleUnit::Radians => return false,
        };
        decimal.is_positive(digits, exponent)
    }
}

crate::dimension::dimension_value_owner!(
    /// A strict authored angle dimension or Angle-root calculation; never a bare number.
    CssAngleValue,
    CssAngleLiteral,
    CssAngleCalculation
);
impl CssAngleValue {
    pub(crate) fn from_parser_component(
        component: CssComponentValue,
        context: &crate::numeric::NumericInputContext<'_>,
    ) -> Result<Self, CssNumericConstructionError> {
        crate::dimension::DimensionValue::from_parser_component(
            component,
            context,
            crate::numeric::CalculationRoot::Angle,
            true,
            CssAngleCalculation::from_expression,
        )
        .map(|value| Self { value })
    }
    pub(crate) fn structural_eq(&self, other: &Self) -> bool {
        self.value.structural_eq(&other.value)
    }
}

/// One exact Number token with zero coefficient, retaining signed spelling and provenance.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CssZeroLiteral {
    component: Box<CssComponentValue>,
}
impl CssZeroLiteral {
    /// Requires an exact Number token whose coefficient is zero.
    pub fn try_from_component(
        component: CssComponentValue,
    ) -> Result<Self, CssComponentValueError> {
        if let CssComponentValueRef::Token(CssValueTokenRef::Number(number)) = component.view()
            && crate::exact_decimal::LexicalDecimal::new(number.representation()).len == 0
        {
            return Ok(Self {
                component: Box::new(component),
            });
        }
        Err(invalid(&component))
    }
    /// Borrows the original signed or exponent-bearing zero spelling.
    pub fn numeric(&self) -> CssNumericTokenRef<'_> {
        let CssComponentValueRef::Token(CssValueTokenRef::Number(number)) = self.component.view()
        else {
            unreachable!("checked zero number")
        };
        number
    }
    /// Borrows the original checked component.
    pub const fn component(&self) -> &CssComponentValue {
        &self.component
    }
    /// Borrows the original component provenance.
    pub const fn origin(&self) -> &CssValueOrigin {
        self.component.origin()
    }
    pub(crate) fn structural_eq(&self, other: &Self) -> bool {
        self.component
            .structural_eq_ignoring_origin(&other.component)
    }
}

/// A checked angle or the literal number-zero exception selected by a consumer grammar.
#[derive(Clone, Debug, Eq, PartialEq)]
#[non_exhaustive]
pub enum CssAngleOrZero {
    Angle(CssAngleValue),
    Zero(CssZeroLiteral),
}
impl CssAngleOrZero {
    /// Borrows the selected branch's original provenance.
    pub fn origin(&self) -> &CssValueOrigin {
        match self {
            Self::Angle(value) => value.origin(),
            Self::Zero(value) => value.origin(),
        }
    }
    pub(crate) fn from_parser_component(
        component: CssComponentValue,
        context: &crate::numeric::NumericInputContext<'_>,
    ) -> Result<Self, CssNumericConstructionError> {
        if matches!(
            component.view(),
            CssComponentValueRef::Token(CssValueTokenRef::Number(_))
        ) {
            return CssZeroLiteral::try_from_component(component)
                .map(Self::Zero)
                .map_err(CssNumericConstructionError::component);
        }
        CssAngleValue::from_parser_component(component, context).map(Self::Angle)
    }
    pub(crate) fn structural_eq(&self, other: &Self) -> bool {
        match (self, other) {
            (Self::Angle(left), Self::Angle(right)) => left.structural_eq(right),
            (Self::Zero(left), Self::Zero(right)) => left.structural_eq(right),
            _ => false,
        }
    }
}
