//! Exact authored frequency dimensions and strict Frequency-root admission.
use crate::{
    CssComponentValue, CssComponentValueError, CssComponentValueErrorKind, CssComponentValueRef,
    CssFrequencyCalculation, CssFrequencyUnit, CssNumericConstructionError,
    CssNumericConstructionErrorKind, CssNumericTokenRef, CssValueOrigin, CssValueTokenRef,
};

pub(crate) const fn suffix(unit: CssFrequencyUnit) -> &'static str {
    match unit {
        CssFrequencyUnit::Hertz => "hz",
        CssFrequencyUnit::Kilohertz => "khz",
    }
}
fn invalid(component: &CssComponentValue) -> CssComponentValueError {
    CssComponentValueError::new(
        CssComponentValueErrorKind::InvalidToken,
        component.origin().clone(),
    )
}

/// An exact signed ordinary frequency, retaining coefficient, selected unit and provenance.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CssFrequencyLiteral {
    component: Box<CssComponentValue>,
    unit: CssFrequencyUnit,
}
impl CssFrequencyLiteral {
    /// Constructs an exact coefficient with the selected frequency unit.
    pub fn try_new(number: &str, unit: CssFrequencyUnit) -> Result<Self, CssComponentValueError> {
        Self::try_from_component(CssComponentValue::try_dimension(number, suffix(unit))?)
    }
    /// Requires one dimension token with a decoded Hz or kHz unit.
    pub fn try_from_component(
        component: CssComponentValue,
    ) -> Result<Self, CssComponentValueError> {
        let CssComponentValueRef::Token(CssValueTokenRef::Dimension { unit, .. }) =
            component.view()
        else {
            return Err(invalid(&component));
        };
        let unit = match unit.to_ascii_lowercase().as_str() {
            "hz" => CssFrequencyUnit::Hertz,
            "khz" => CssFrequencyUnit::Kilohertz,
            _ => return Err(invalid(&component)),
        };
        Ok(Self {
            component: Box::new(component),
            unit,
        })
    }
    /// Borrows the exact authored numeric token.
    pub fn numeric(&self) -> CssNumericTokenRef<'_> {
        let CssComponentValueRef::Token(CssValueTokenRef::Dimension { number, .. }) =
            self.component.view()
        else {
            unreachable!("checked frequency dimension")
        };
        number
    }
    /// Returns the selected decoded unit without coefficient conversion.
    pub const fn unit(&self) -> CssFrequencyUnit {
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
}

#[derive(Clone, Debug, Eq, PartialEq)]
enum FrequencyValue {
    Literal(CssFrequencyLiteral),
    Calculation(CssFrequencyCalculation),
}
/// An authored frequency dimension or retained Frequency-root math expression.
///
/// Checked conversion requires original graph closure; no bare number is admitted.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CssFrequencyValue {
    value: FrequencyValue,
}
impl CssFrequencyValue {
    /// Retains an already checked ordinary literal.
    pub fn from_literal(literal: CssFrequencyLiteral) -> Self {
        Self {
            value: FrequencyValue::Literal(literal),
        }
    }
    /// Rejects original recovery before normalizing an actual ordinary root; retains math.
    pub fn try_from_calculation(
        calculation: CssFrequencyCalculation,
    ) -> Result<Self, CssNumericConstructionError> {
        if let Some(origin) = calculation.components().first_implicit_origin() {
            return Err(CssNumericConstructionError::at_origin(
                CssNumericConstructionErrorKind::RecoveredComponent,
                origin.clone(),
            ));
        }
        Self::from_parser_calculation(calculation)
    }
    fn from_parser_calculation(
        calculation: CssFrequencyCalculation,
    ) -> Result<Self, CssNumericConstructionError> {
        if let Some(component) = calculation.literal_root_component() {
            return CssFrequencyLiteral::try_from_component(component.clone())
                .map(Self::from_literal)
                .map_err(CssNumericConstructionError::component);
        }
        Ok(Self {
            value: FrequencyValue::Calculation(calculation),
        })
    }
    pub(crate) fn from_parser_component(
        component: CssComponentValue,
        context: &crate::numeric::NumericInputContext<'_>,
    ) -> Result<Self, CssNumericConstructionError> {
        if matches!(component.view(), CssComponentValueRef::Token(_)) {
            return CssFrequencyLiteral::try_from_component(component)
                .map(Self::from_literal)
                .map_err(CssNumericConstructionError::component);
        }
        let values = crate::CssComponentValues::try_new(vec![component])
            .map_err(CssNumericConstructionError::component)?;
        let expression = context.admit(values, crate::numeric::CalculationRoot::Frequency)?;
        Self::from_parser_calculation(CssFrequencyCalculation::from_expression(expression))
    }
    pub(crate) fn ensure_closed(&self) -> Result<(), CssNumericConstructionError> {
        if let Some(value) = self.calculation()
            && let Some(origin) = value.components().first_implicit_origin()
        {
            return Err(CssNumericConstructionError::at_origin(
                CssNumericConstructionErrorKind::RecoveredComponent,
                origin.clone(),
            ));
        }
        Ok(())
    }
    /// Borrows the ordinary branch, when present.
    pub fn literal(&self) -> Option<&CssFrequencyLiteral> {
        match &self.value {
            FrequencyValue::Literal(value) => Some(value),
            FrequencyValue::Calculation(_) => None,
        }
    }
    /// Borrows the original math branch, when present.
    pub fn calculation(&self) -> Option<&CssFrequencyCalculation> {
        match &self.value {
            FrequencyValue::Calculation(value) => Some(value),
            FrequencyValue::Literal(_) => None,
        }
    }
    /// Borrows the selected branch's original provenance.
    pub fn origin(&self) -> &CssValueOrigin {
        match &self.value {
            FrequencyValue::Literal(value) => value.origin(),
            FrequencyValue::Calculation(value) => value.origin(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{CssComponentValueRef, parse_component_values};
    use std::error::Error;

    #[test]
    fn independently_recovered_children_report_the_first_original_source_closure() {
        let left_text = "/* first 😀 */\ncalc(-1hz";
        let right_text = "/* second snapshot */\ncalc(2hz";
        let left = parse_component_values(left_text)
            .unwrap()
            .items()
            .last()
            .unwrap()
            .clone();
        let right = parse_component_values(right_text)
            .unwrap()
            .items()
            .last()
            .unwrap()
            .clone();
        let CssComponentValueRef::Function(left_function) = left.view() else {
            unreachable!()
        };
        let CssComponentValueRef::Function(right_function) = right.view() else {
            unreachable!()
        };
        let left_closure = left_function.closing_origin().clone();
        let right_closure = right_function.closing_origin().clone();
        assert_ne!(left_closure, right_closure);
        let source = left.parsed_origin().unwrap().source().clone();
        for reverse in [false, true] {
            let (first, second, expected) = if reverse {
                (right.clone(), left.clone(), &right_closure)
            } else {
                (left.clone(), right.clone(), &left_closure)
            };
            let arguments = crate::CssComponentValues::try_new(vec![
                first,
                crate::CssComponentValue::try_token(",").unwrap(),
                second,
            ])
            .unwrap();
            let outer = crate::CssComponentValue::try_function("min", arguments).unwrap();
            let values = crate::CssComponentValues::try_new(vec![outer]).unwrap();
            let expression = crate::numeric::NumericInputContext::Parsed(&source)
                .admit(values, crate::numeric::CalculationRoot::Frequency)
                .unwrap();
            let calculation = CssFrequencyCalculation::from_expression(expression);
            let original = calculation.clone();
            let error = CssFrequencyValue::try_from_calculation(calculation.clone()).unwrap_err();
            assert_eq!(
                error.kind(),
                &CssNumericConstructionErrorKind::RecoveredComponent
            );
            assert_eq!(error.origin(), Some(expected));
            assert!(error.source().is_none());
            assert_eq!(error.path(), None);
            assert_eq!(calculation, original);
        }
    }

    #[test]
    fn recovered_original_closure_precedes_value_normalization_and_range_checks() {
        let text = "/* 😀 */\ncalc(min(-1hz, 2hz";
        let components = parse_component_values(text).unwrap();
        let component = components
            .items()
            .iter()
            .find(|component| matches!(component.view(), CssComponentValueRef::Function(_)))
            .unwrap();
        let CssComponentValueRef::Function(function) = component.view() else {
            unreachable!()
        };
        let first_closure = function.closing_origin().clone();
        assert!(matches!(
            first_closure,
            CssValueOrigin::ImplicitClosure { .. }
        ));
        let source = component.parsed_origin().unwrap().source().clone();
        let expression = crate::numeric::NumericInputContext::Parsed(&source)
            .admit(components, crate::numeric::CalculationRoot::Frequency)
            .unwrap();
        let calculation = CssFrequencyCalculation::from_expression(expression);
        let original = calculation.clone();
        // The existing authored writer materializes closings; strict admission
        // must inspect the original graph rather than this serialized output.
        assert_eq!(
            calculation.serialize().unwrap().as_css(),
            "calc(min(-1hz, 2hz))"
        );
        let error = CssFrequencyValue::try_from_calculation(calculation.clone()).unwrap_err();
        assert_eq!(
            error.kind(),
            &CssNumericConstructionErrorKind::RecoveredComponent
        );
        assert_eq!(error.origin(), Some(&first_closure));
        assert!(error.source().is_none());
        assert_eq!(error.path(), None);
        assert_eq!(calculation, original);
    }
}
