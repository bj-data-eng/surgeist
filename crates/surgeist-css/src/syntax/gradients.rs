//! Shared numeric admission for the selected Images 4 angular stop/hint domain.
use super::CssAngularColorStopPosition;
use crate::{
    CssAngleLiteral, CssAnglePercentageCalculation, CssAngleValue, CssComponentValue,
    CssComponentValueRef, CssComponentValues, CssNumericConstructionError, CssSpecifiedPercentage,
    CssValueOrigin, CssValueTokenRef, CssZeroLiteral,
};

impl CssAngularColorStopPosition {
    /// Checks an ordinary angle, percentage, or exact number-zero token.
    pub fn try_from_component(
        component: CssComponentValue,
    ) -> Result<Self, CssNumericConstructionError> {
        match component.view() {
            CssComponentValueRef::Token(CssValueTokenRef::Percentage(_)) => {
                CssSpecifiedPercentage::try_from_component(component).map(Self::Percentage)
            }
            CssComponentValueRef::Token(CssValueTokenRef::Number(_)) => {
                CssZeroLiteral::try_from_component(component)
                    .map(Self::Zero)
                    .map_err(CssNumericConstructionError::component)
            }
            _ => CssAngleLiteral::try_from_component(component)
                .map(CssAngleValue::from_literal)
                .map(Self::Angle)
                .map_err(CssNumericConstructionError::component),
        }
    }

    /// Retains checked angular-percentage math without resolving its percentage basis.
    pub fn from_calculation(value: CssAnglePercentageCalculation) -> Self {
        Self::Calculation(value)
    }

    /// Borrows the original numeric occurrence, including its source coordinates.
    pub fn origin(&self) -> &CssValueOrigin {
        match self {
            Self::Angle(value) => value.origin(),
            Self::Percentage(value) => value.origin(),
            Self::Zero(value) => value.origin(),
            Self::Calculation(value) => value.origin(),
        }
    }

    pub(crate) fn from_parser_component(
        component: CssComponentValue,
        context: &crate::numeric::NumericInputContext<'_>,
    ) -> Result<Self, CssNumericConstructionError> {
        if matches!(component.view(), CssComponentValueRef::Token(_)) {
            return Self::try_from_component(component);
        }
        let values = CssComponentValues::try_new(vec![component])
            .map_err(CssNumericConstructionError::component)?;
        context
            .admit(values, crate::numeric::CalculationRoot::AnglePercentage)
            .map(CssAnglePercentageCalculation::from_expression)
            .map(Self::Calculation)
    }

    pub(crate) fn nesting_depth(&self) -> u32 {
        match self {
            Self::Angle(value) => value
                .calculation()
                .map_or(0, |v| v.components().nesting_depth()),
            Self::Percentage(value) => value
                .calculation()
                .map_or(0, |v| v.components().nesting_depth()),
            Self::Zero(_) => 0,
            Self::Calculation(value) => value.components().nesting_depth(),
        }
    }
}
