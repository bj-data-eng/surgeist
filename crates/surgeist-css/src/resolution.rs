//! Exact nonnegative ordinary resolution and strict Resolution-root value admission.
use crate::{
    CssComponentValue, CssComponentValueError, CssComponentValueErrorKind, CssComponentValueRef,
    CssNumericConstructionError, CssNumericConstructionErrorKind, CssNumericTokenRef,
    CssResolutionCalculation, CssResolutionUnit, CssValueOrigin, CssValueTokenRef,
};

const fn suffix(unit: CssResolutionUnit) -> &'static str {
    match unit {
        CssResolutionUnit::Dpi => "dpi",
        CssResolutionUnit::Dpcm => "dpcm",
        CssResolutionUnit::Dppx => "dppx",
    }
}
fn invalid(component: &CssComponentValue) -> CssNumericConstructionError {
    CssNumericConstructionError::component(CssComponentValueError::new(
        CssComponentValueErrorKind::InvalidToken,
        component.origin().clone(),
    ))
}

/// An exact nonnegative ordinary resolution, retaining coefficient, unit and provenance.
///
/// Media resolution has a separate signed authored domain. Canonical resolution
/// specified emission awaits the CSS-owned exact conversion and rounding provider.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CssResolutionLiteral {
    component: Box<CssComponentValue>,
    unit: CssResolutionUnit,
}
impl CssResolutionLiteral {
    /// Constructs an exact ordinary coefficient, rejecting only negative nonzero values.
    pub fn try_new(
        number: &str,
        unit: CssResolutionUnit,
    ) -> Result<Self, CssNumericConstructionError> {
        Self::try_from_component(
            CssComponentValue::try_dimension(number, suffix(unit))
                .map_err(CssNumericConstructionError::component)?,
        )
    }
    /// Requires a resolution dimension; decoded x selects Dppx while raw spelling survives.
    pub fn try_from_component(
        component: CssComponentValue,
    ) -> Result<Self, CssNumericConstructionError> {
        let CssComponentValueRef::Token(CssValueTokenRef::Dimension { number, unit }) =
            component.view()
        else {
            return Err(invalid(&component));
        };
        let unit = match unit.to_ascii_lowercase().as_str() {
            "dpi" => CssResolutionUnit::Dpi,
            "dpcm" => CssResolutionUnit::Dpcm,
            "dppx" | "x" => CssResolutionUnit::Dppx,
            _ => return Err(invalid(&component)),
        };
        if crate::exact_decimal::LexicalDecimal::new(number.representation()).negative {
            return Err(CssNumericConstructionError::at_origin(
                CssNumericConstructionErrorKind::OutOfRange,
                component.origin().clone(),
            ));
        }
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
            unreachable!("checked resolution dimension")
        };
        number
    }
    /// Returns the selected decoded unit without coefficient conversion.
    pub const fn unit(&self) -> CssResolutionUnit {
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
enum ResolutionValue {
    Literal(CssResolutionLiteral),
    Calculation(CssResolutionCalculation),
}
/// An authored ordinary resolution with a nonnegative range, retaining actual math symbolically.
///
/// This does not replace the signed media Resolution-root domain or evaluate/clamp calculations.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CssResolutionValue {
    value: ResolutionValue,
}
impl CssResolutionValue {
    /// Retains an already checked nonnegative ordinary literal.
    pub fn from_literal(literal: CssResolutionLiteral) -> Self {
        Self {
            value: ResolutionValue::Literal(literal),
        }
    }
    /// Rejects original recovery first, checks an actual ordinary root's range and retains math.
    pub fn try_from_calculation(
        calculation: CssResolutionCalculation,
    ) -> Result<Self, CssNumericConstructionError> {
        if let Some(origin) = calculation.components().first_implicit_origin() {
            return Err(CssNumericConstructionError::at_origin(
                CssNumericConstructionErrorKind::RecoveredComponent,
                origin.clone(),
            ));
        }
        if let Some(component) = calculation.literal_root_component() {
            return CssResolutionLiteral::try_from_component(component.clone())
                .map(Self::from_literal);
        }
        Ok(Self {
            value: ResolutionValue::Calculation(calculation),
        })
    }
    /// Borrows the ordinary branch, when present.
    pub fn literal(&self) -> Option<&CssResolutionLiteral> {
        match &self.value {
            ResolutionValue::Literal(value) => Some(value),
            ResolutionValue::Calculation(_) => None,
        }
    }
    /// Borrows the original symbolic math branch, when present.
    pub fn calculation(&self) -> Option<&CssResolutionCalculation> {
        match &self.value {
            ResolutionValue::Calculation(value) => Some(value),
            ResolutionValue::Literal(_) => None,
        }
    }
    /// Borrows the selected branch's original provenance.
    pub fn origin(&self) -> &CssValueOrigin {
        match &self.value {
            ResolutionValue::Literal(value) => value.origin(),
            ResolutionValue::Calculation(value) => value.origin(),
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
        let left_text = "/* first 😀 */\ncalc(-1dppx";
        let right_text = "/* second snapshot */\ncalc(2dppx";
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
                .admit(values, crate::numeric::CalculationRoot::Resolution)
                .unwrap();
            let calculation = CssResolutionCalculation::from_expression(expression);
            let original = calculation.clone();
            let error = CssResolutionValue::try_from_calculation(calculation.clone()).unwrap_err();
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
        let text = "/* 😀 */\ncalc(min(-1dppx, 2dppx";
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
            .admit(components, crate::numeric::CalculationRoot::Resolution)
            .unwrap();
        let calculation = CssResolutionCalculation::from_expression(expression);
        let original = calculation.clone();
        // The existing authored writer materializes closings; strict admission
        // must inspect the original graph rather than this serialized output.
        assert_eq!(
            calculation.serialize().unwrap().as_css(),
            "calc(min(-1dppx, 2dppx))"
        );
        let error = CssResolutionValue::try_from_calculation(calculation.clone()).unwrap_err();
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
