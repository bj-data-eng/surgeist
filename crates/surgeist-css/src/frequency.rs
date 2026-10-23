//! Exact authored frequency dimensions and strict Frequency-root admission.
use crate::{
    CssComponentValue, CssFrequencyCalculation, CssFrequencyUnit, CssNumericConstructionError,
};

pub(crate) const fn suffix(unit: CssFrequencyUnit) -> &'static str {
    crate::numeric::units::suffix(crate::CssNumericUnit::Frequency(unit))
}

crate::dimension::dimension_literal_owner!(
    /// An exact signed ordinary frequency, retaining coefficient, selected unit and provenance.
    ///
    /// # Specified serialization
    ///
    /// Emits ordinary frequencies with the selected lowercase hz or khz unit. This authored phase
    /// follows the selected primitive serialization policy; CSSOM's specified/computed phase
    /// question remains open. Ordinary text rounds to six fractional places, nearest with ties away
    /// from zero; retained coefficients stay exact. This literal operation performs no calculation
    /// projection. Ordinary units are retained lowercased with at most six fractional places,
    /// nearest with ties away from zero and no scientific fallback. CSSOM's specified/computed
    /// frequency phase question stays open. This literal operation performs no calculation
    /// projection;
    CssFrequencyLiteral,
    CssFrequencyUnit,
    suffix
);

crate::dimension::dimension_value_owner!(
    /// An authored frequency dimension or retained Frequency-root math expression.
    ///
    /// Checked conversion requires original graph closure; no bare number is admitted.
    ///
    /// # Specified serialization
    ///
    /// Emits ordinary frequencies with the selected lowercase hz or khz unit. This authored phase
    /// follows the selected primitive serialization policy; CSSOM's specified/computed phase
    /// question remains open. Ordinary text rounds to six fractional places, nearest with ties away
    /// from zero; retained coefficients stay exact. Finite math text rounds the actual binary64
    /// value to six places; arithmetic precision and range remain unfinished. Ordinary units are
    /// retained lowercased with at most six fractional places, nearest with ties away from zero and
    /// no scientific fallback. CSSOM's specified/computed frequency phase question stays open.
    /// Actual math retains its canonical units and simplification. Finite number text follows the
    /// six-place policy; arithmetic precision and range remain unfinished.
    CssFrequencyValue,
    CssFrequencyLiteral,
    CssFrequencyCalculation
);
impl CssFrequencyValue {
    pub(crate) fn from_parser_component(
        component: CssComponentValue,
        context: &crate::numeric::NumericInputContext<'_>,
    ) -> Result<Self, CssNumericConstructionError> {
        crate::dimension::DimensionValue::from_parser_component(
            component,
            context,
            crate::numeric::CalculationRoot::Frequency,
            false,
            CssFrequencyCalculation::from_expression,
        )
        .map(|value| Self { value })
    }
    pub(crate) fn ensure_closed(&self) -> Result<(), CssNumericConstructionError> {
        self.value.ensure_closed()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        CssComponentValueRef, CssNumericConstructionErrorKind, CssValueOrigin,
        parse_component_values,
    };
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
            let arguments = crate::CssComponentValues::try_new(
                vec![
                    first,
                    crate::CssComponentValue::try_token(",").unwrap(),
                    second,
                ],
                Default::default(),
            )
            .unwrap();
            let outer = crate::CssComponentValue::try_function("min", arguments).unwrap();
            let values =
                crate::CssComponentValues::try_new(vec![outer], Default::default()).unwrap();
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
            calculation
                .serialize_tokens(Default::default())
                .unwrap()
                .as_css(),
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
