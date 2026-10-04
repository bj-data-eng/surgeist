//! Exact authored ordinary time dimensions and duration range admission.
use crate::{
    CssComponentValue, CssComponentValueError, CssComponentValueErrorKind, CssComponentValueRef,
    CssComponentValues, CssNumericConstructionError, CssNumericConstructionErrorKind,
    CssNumericTokenRef, CssTimeCalculation, CssTimeUnit, CssValueOrigin, CssValueTokenRef,
};

pub(crate) const fn suffix(unit: CssTimeUnit) -> &'static str {
    match unit {
        CssTimeUnit::Seconds => "s",
        CssTimeUnit::Milliseconds => "ms",
    }
}
fn invalid(component: &CssComponentValue) -> CssComponentValueError {
    CssComponentValueError::new(
        CssComponentValueErrorKind::InvalidToken,
        component.origin().clone(),
    )
}

/// An exact ordinary time dimension, retaining spelling, unit and provenance.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CssTimeLiteral {
    component: Box<CssComponentValue>,
    unit: CssTimeUnit,
}
impl CssTimeLiteral {
    /// Constructs an exact coefficient with the selected standard time unit.
    pub fn try_new(number: &str, unit: CssTimeUnit) -> Result<Self, CssComponentValueError> {
        Self::try_from_component(CssComponentValue::try_dimension(number, suffix(unit))?)
    }
    /// Requires one dimension token with a decoded s or ms unit.
    pub fn try_from_component(
        component: CssComponentValue,
    ) -> Result<Self, CssComponentValueError> {
        let CssComponentValueRef::Token(CssValueTokenRef::Dimension { unit, .. }) =
            component.view()
        else {
            return Err(invalid(&component));
        };
        let unit = match unit.to_ascii_lowercase().as_str() {
            "s" => CssTimeUnit::Seconds,
            "ms" => CssTimeUnit::Milliseconds,
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
            unreachable!("checked time dimension")
        };
        number
    }
    /// Returns the decoded time unit without converting the coefficient.
    pub const fn unit(&self) -> CssTimeUnit {
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
    /// Exact ordinary duration equivalence for specified shorthand omission.
    /// Coefficients and origins remain authored; only standard s/ms conversion
    /// participates, without floating-point rounding or exponent expansion.
    fn equivalent(&self, other: &Self) -> bool {
        fn seconds(literal: &CssTimeLiteral) -> crate::exact_decimal::LexicalDecimal<'_> {
            crate::exact_decimal::LexicalDecimal::new(literal.numeric().representation()).shifted(
                match literal.unit() {
                    CssTimeUnit::Seconds => 0,
                    CssTimeUnit::Milliseconds => -3,
                },
            )
        }
        seconds(self).value_eq(&seconds(other))
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
enum TimeValue {
    Literal(CssTimeLiteral),
    Calculation(CssTimeCalculation),
}
/// An authored time dimension or Time-root calculation; never a bare number.
///
/// Checked construction requires closure; parsing can retain recovered syntax with diagnostics.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CssTimeValue {
    value: TimeValue,
}
#[derive(Clone, Copy)]
enum Admission {
    Strict,
    RecoveringSyntax,
}
impl CssTimeValue {
    /// Retains an already checked ordinary time literal.
    pub fn from_literal(literal: CssTimeLiteral) -> Self {
        Self {
            value: TimeValue::Literal(literal),
        }
    }
    /// Rejects recovered graph closure, normalizes an ordinary dimension root, and retains math roots.
    pub fn try_from_calculation(
        calculation: CssTimeCalculation,
    ) -> Result<Self, CssNumericConstructionError> {
        Self::from_calculation(calculation, Admission::Strict)
    }
    /// Borrows the ordinary dimension branch, when present.
    pub fn literal(&self) -> Option<&CssTimeLiteral> {
        match &self.value {
            TimeValue::Literal(value) => Some(value),
            TimeValue::Calculation(_) => None,
        }
    }
    /// Borrows the preserved Time-root math branch, when present.
    pub fn calculation(&self) -> Option<&CssTimeCalculation> {
        match &self.value {
            TimeValue::Calculation(value) => Some(value),
            TimeValue::Literal(_) => None,
        }
    }
    /// Borrows the selected branch's original provenance.
    pub fn origin(&self) -> &CssValueOrigin {
        match &self.value {
            TimeValue::Literal(value) => value.origin(),
            TimeValue::Calculation(value) => value.origin(),
        }
    }
    fn from_calculation(
        calculation: CssTimeCalculation,
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
            return CssTimeLiteral::try_from_component(component.clone())
                .map(Self::from_literal)
                .map_err(CssNumericConstructionError::component);
        }
        Ok(Self {
            value: TimeValue::Calculation(calculation),
        })
    }
    pub(crate) fn from_parser_component(
        component: CssComponentValue,
        context: &crate::numeric::NumericInputContext<'_>,
    ) -> Result<Self, CssNumericConstructionError> {
        if matches!(component.view(), CssComponentValueRef::Token(_)) {
            return CssTimeLiteral::try_from_component(component)
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
        let expression = context.admit(values, crate::numeric::CalculationRoot::Time)?;
        Self::from_calculation(CssTimeCalculation::from_expression(expression), admission)
    }
    pub(crate) fn first_implicit_origin(&self) -> Option<&CssValueOrigin> {
        self.calculation()
            .and_then(|value| value.components().first_implicit_origin())
    }
    pub(crate) fn ensure_closed(&self) -> Result<(), CssNumericConstructionError> {
        if let Some(origin) = self.first_implicit_origin() {
            return Err(CssNumericConstructionError::at_origin(
                CssNumericConstructionErrorKind::RecoveredComponent,
                origin.clone(),
            ));
        }
        Ok(())
    }
    pub(crate) fn structural_eq(&self, other: &Self) -> bool {
        match (&self.value, &other.value) {
            (TimeValue::Literal(left), TimeValue::Literal(right)) => left.structural_eq(right),
            (TimeValue::Calculation(left), TimeValue::Calculation(right)) => {
                left.structural_eq(right)
            }
            _ => false,
        }
    }
}

/// An authored duration with a nonnegative ordinary literal or a retained time calculation.
#[derive(Clone, Debug)]
pub struct CssDuration {
    time: CssTimeValue,
}
impl CssDuration {
    /// Rejects recovered input and genuinely negative ordinary literals; calculations retain their authored range.
    pub fn try_new(time: CssTimeValue) -> Result<Self, CssNumericConstructionError> {
        time.ensure_closed()?;
        Self::from_parser_value(time)
    }
    pub(crate) fn from_parser_value(
        time: CssTimeValue,
    ) -> Result<Self, CssNumericConstructionError> {
        if let Some(literal) = time.literal()
            && crate::exact_decimal::LexicalDecimal::new(literal.numeric().representation())
                .negative
        {
            return Err(CssNumericConstructionError::at_origin(
                CssNumericConstructionErrorKind::OutOfRange,
                literal.origin().clone(),
            ));
        }
        Ok(Self { time })
    }
    /// Borrows the retained time value without resolving calculations.
    pub const fn time(&self) -> &CssTimeValue {
        &self.time
    }
    /// Borrows the original selected branch provenance.
    pub fn origin(&self) -> &CssValueOrigin {
        self.time.origin()
    }
    pub(crate) fn specified_value_eq(&self, other: &Self) -> bool {
        match (self.time.literal(), other.time.literal()) {
            (Some(left), Some(right)) => left.equivalent(right),
            _ => self.time.structural_eq(&other.time),
        }
    }
    pub(crate) fn is_exact_ordinary_zero(&self) -> bool {
        self.time.literal().is_some_and(|literal| {
            crate::exact_decimal::LexicalDecimal::new(literal.numeric().representation()).len == 0
        })
    }
}
impl PartialEq for CssDuration {
    fn eq(&self, other: &Self) -> bool {
        self.time.structural_eq(&other.time)
    }
}
impl Eq for CssDuration {}
