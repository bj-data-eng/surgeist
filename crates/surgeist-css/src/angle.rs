//! Exact authored ordinary angles and the grammar-specific number-zero exception.
use crate::{
    CssAngleCalculation, CssAngleUnit, CssComponentValue, CssComponentValueError,
    CssComponentValueErrorKind, CssComponentValueRef, CssComponentValues,
    CssNumericConstructionError, CssNumericConstructionErrorKind, CssNumericTokenRef,
    CssValueOrigin, CssValueTokenRef,
};

pub(crate) const fn suffix(unit: CssAngleUnit) -> &'static str {
    match unit {
        CssAngleUnit::Degrees => "deg",
        CssAngleUnit::Gradians => "grad",
        CssAngleUnit::Radians => "rad",
        CssAngleUnit::Turns => "turn",
    }
}
fn invalid(component: &CssComponentValue) -> CssComponentValueError {
    CssComponentValueError::new(
        CssComponentValueErrorKind::InvalidToken,
        component.origin().clone(),
    )
}

/// An exact ordinary angle dimension, retaining spelling, unit and provenance.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CssAngleLiteral {
    component: Box<CssComponentValue>,
    unit: CssAngleUnit,
}
impl CssAngleLiteral {
    /// Constructs an exact coefficient with a canonical angle unit.
    pub fn try_new(number: &str, unit: CssAngleUnit) -> Result<Self, CssComponentValueError> {
        Self::try_from_component(CssComponentValue::try_dimension(number, suffix(unit))?)
    }
    /// Requires one dimension token with a decoded deg, grad, rad or turn unit.
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
    /// Returns the exact authored numeric token.
    pub fn numeric(&self) -> CssNumericTokenRef<'_> {
        let CssComponentValueRef::Token(CssValueTokenRef::Dimension { number, .. }) =
            self.component.view()
        else {
            unreachable!("checked angle dimension")
        };
        number
    }
    /// Returns the decoded angle unit without converting the coefficient.
    pub const fn unit(&self) -> CssAngleUnit {
        self.unit
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
        self.unit == other.unit
            && self
                .component
                .structural_eq_ignoring_origin(&other.component)
    }
    pub(crate) fn is_default_gradient_direction(&self) -> bool {
        let decimal = crate::exact_decimal::LexicalDecimal::new(self.numeric().representation());
        if decimal.negative {
            return false;
        }
        let (digits, exponent): (&[u8], i128) = match self.unit {
            CssAngleUnit::Degrees => (&[1, 8], 1),
            CssAngleUnit::Gradians => (&[2], 2),
            CssAngleUnit::Turns => (&[5], -1),
            CssAngleUnit::Radians => return false,
        };
        decimal.exponent == Some(exponent) && decimal.digits().eq(digits.iter().copied())
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
enum AngleValue {
    Literal(CssAngleLiteral),
    Calculation(CssAngleCalculation),
}
/// A strict authored angle dimension or Angle-root calculation; never a bare number.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CssAngleValue {
    value: AngleValue,
}
#[derive(Clone, Copy)]
enum Admission {
    Strict,
    RecoveringSyntax,
}
impl CssAngleValue {
    /// Retains an already checked ordinary angle literal.
    pub fn from_literal(literal: CssAngleLiteral) -> Self {
        Self {
            value: AngleValue::Literal(literal),
        }
    }
    /// Rejects recovered graph closure, normalizes an ordinary dimension root, and retains math roots.
    pub fn try_from_calculation(
        calculation: CssAngleCalculation,
    ) -> Result<Self, CssNumericConstructionError> {
        Self::from_calculation(calculation, Admission::Strict)
    }
    /// Borrows the ordinary dimension branch, when present.
    pub fn literal(&self) -> Option<&CssAngleLiteral> {
        match &self.value {
            AngleValue::Literal(value) => Some(value),
            AngleValue::Calculation(_) => None,
        }
    }
    /// Borrows the preserved Angle-root math branch, when present.
    pub fn calculation(&self) -> Option<&CssAngleCalculation> {
        match &self.value {
            AngleValue::Calculation(value) => Some(value),
            AngleValue::Literal(_) => None,
        }
    }
    /// Borrows the selected branch's original provenance.
    pub fn origin(&self) -> &CssValueOrigin {
        match &self.value {
            AngleValue::Literal(value) => value.origin(),
            AngleValue::Calculation(value) => value.origin(),
        }
    }
    fn from_calculation(
        calculation: CssAngleCalculation,
        admission: Admission,
    ) -> Result<Self, CssNumericConstructionError> {
        if matches!(admission, Admission::Strict)
            && let Some(origin) = calculation.components().first_implicit_origin()
        {
            return Err(CssNumericConstructionError::at_origin(
                CssNumericConstructionErrorKind::RecoveredComponent,
                origin.clone(),
            ));
        }
        if let Some(component) = calculation.literal_root_component() {
            return CssAngleLiteral::try_from_component(component.clone())
                .map(Self::from_literal)
                .map_err(CssNumericConstructionError::component);
        }
        Ok(Self {
            value: AngleValue::Calculation(calculation),
        })
    }
    pub(crate) fn from_parser_component(
        component: CssComponentValue,
        context: &crate::numeric::NumericInputContext<'_>,
    ) -> Result<Self, CssNumericConstructionError> {
        if matches!(component.view(), CssComponentValueRef::Token(_)) {
            return CssAngleLiteral::try_from_component(component)
                .map(Self::from_literal)
                .map_err(CssNumericConstructionError::component);
        }
        let values = CssComponentValues::try_new(vec![component])
            .map_err(CssNumericConstructionError::component)?;
        let admission = match context {
            crate::numeric::NumericInputContext::Parsed(_) => Admission::RecoveringSyntax,
            crate::numeric::NumericInputContext::Components(..) => Admission::Strict,
        };
        if matches!(admission, Admission::Strict)
            && let Some(origin) = values.first_implicit_origin()
        {
            return Err(CssNumericConstructionError::at_origin(
                CssNumericConstructionErrorKind::RecoveredComponent,
                origin.clone(),
            ));
        }
        let expression = context.admit(values, crate::numeric::CalculationRoot::Angle)?;
        Self::from_calculation(CssAngleCalculation::from_expression(expression), admission)
    }
    pub(crate) fn structural_eq(&self, other: &Self) -> bool {
        match (&self.value, &other.value) {
            (AngleValue::Literal(left), AngleValue::Literal(right)) => left.structural_eq(right),
            (AngleValue::Calculation(left), AngleValue::Calculation(right)) => {
                left.structural_eq(right)
            }
            _ => false,
        }
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
