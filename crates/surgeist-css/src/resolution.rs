//! Exact nonnegative ordinary resolution and strict Resolution-root value admission.
use crate::{
    CssComponentValue, CssNumericConstructionError, CssNumericConstructionErrorKind,
    CssNumericTokenRef, CssResolutionUnit, CssSpecifiedValueSerializationError,
    CssSpecifiedValueSerializationErrorKind, CssSpecifiedValueSerializationLimits, CssValueOrigin,
    specified_serialization::SpecifiedSerializationContext,
};

type SerializationResult<T> = Result<T, CssSpecifiedValueSerializationError>;

const fn suffix(unit: CssResolutionUnit) -> &'static str {
    crate::numeric::units::suffix(crate::CssNumericUnit::Resolution(unit))
}
pub(crate) const fn exact_factor(unit: CssResolutionUnit) -> crate::exact_decimal::ExactFactor {
    match crate::numeric::units::exact_factor(crate::CssNumericUnit::Resolution(unit)) {
        Some(factor) => factor,
        None => panic!("Values 4 fixes an exact ratio for every resolution unit"),
    }
}

/// An exact nonnegative ordinary resolution, retaining coefficient, unit and provenance.
///
/// Media resolution has a separate signed authored domain. Specified serialization
/// converts this ordinary value to `dppx` without changing its authored storage.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CssResolutionLiteral {
    component: Box<CssComponentValue>,
    unit: CssResolutionUnit,
}
impl crate::dimension::DimensionRoot for CssResolutionLiteral {
    fn from_root(component: CssComponentValue) -> Result<Self, CssNumericConstructionError> {
        Self::try_from_component(component)
    }
    fn origin(&self) -> &CssValueOrigin {
        self.component.origin()
    }
    fn structural_eq(&self, other: &Self) -> bool {
        self.unit == other.unit
            && self
                .component
                .structural_eq_ignoring_origin(&other.component)
    }
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
        let unit = crate::dimension::decode_unit(&component)
            .map_err(CssNumericConstructionError::component)?;
        if crate::exact_decimal::LexicalDecimal::new(
            crate::dimension::dimension_number(&component).representation(),
        )
        .negative
        {
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
        crate::dimension::dimension_number(&self.component)
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

    /// Emits canonical `dppx` after exact unit conversion and six-place rounding.
    ///
    /// The CSSOM resolution rule selects dots per CSS pixel. Conversion uses
    /// exactly 96 dots per inch and 127/4800 dots per pixel per dot per centimeter.
    /// Shared number serialization rounds halfway values away from zero;
    /// signed zero emits zero. The original coefficient, unit and origin survive.
    pub fn serialize_specified(&self) -> SerializationResult<String> {
        self.serialize_specified_with_limits(CssSpecifiedValueSerializationLimits::default())
    }

    /// Uses one cumulative input, projection-work and generated-byte budget.
    ///
    /// Bytes include the suffix and are checked against actual rounded output.
    /// Exact conversion spends shared rational work under the projection limit.
    /// Failure returns no partial text and does not alter the authored value.
    pub fn serialize_specified_with_limits(
        &self,
        limits: CssSpecifiedValueSerializationLimits,
    ) -> SerializationResult<String> {
        let mut writer = crate::specified_rule_serialization::SpecifiedRuleWriter::new(limits);
        self.append_to_rule_writer(&mut writer)?;
        Ok(writer.css)
    }

    pub(crate) fn append_to_rule_writer(
        &self,
        writer: &mut crate::specified_rule_serialization::SpecifiedRuleWriter,
    ) -> std::result::Result<(), crate::CssSpecifiedValueSerializationError> {
        let context = &mut writer.context;
        let output = &mut writer.css;
        self.append_specified(context, output)?;
        Ok(())
    }

    pub(crate) fn append_specified(
        &self,
        context: &mut SpecifiedSerializationContext,
        output: &mut String,
    ) -> SerializationResult<()> {
        context.charge_input(1)?;
        context.charge_projection(1)?;
        if context.output_suppressed() {
            return Ok(());
        }
        let text = if self.unit() == CssResolutionUnit::Dppx {
            crate::specified_serialization::format_coefficient(
                self.numeric().representation(),
                0,
                "dppx",
                context.remaining_bytes(),
            )?
        } else {
            let number_bytes = context.remaining_bytes().checked_sub(4).ok_or_else(|| {
                CssSpecifiedValueSerializationError::new(
                    CssSpecifiedValueSerializationErrorKind::ByteLimit,
                )
            })?;
            let factor = exact_factor(self.unit());
            let number = crate::exact_decimal::ExactRational::format_generic_number(
                self.numeric().representation(),
                factor,
                number_bytes,
                context,
            )?;
            context.append(output, &number)?;
            return context.append(output, "dppx");
        };
        context.append(output, &text)
    }
}

crate::dimension::dimension_value_owner!(
    /// An authored ordinary resolution with a nonnegative range, retaining actual math symbolically.
    ///
    /// This does not replace the signed media Resolution-root domain or evaluate/clamp calculations.
    CssResolutionValue,
    CssResolutionLiteral,
    CssResolutionCalculation,
    {
        /// Emits an ordinary resolution in `dppx` or projects retained numeric math.
        ///
        /// Calculations use the existing shared finite/structural simplifier. Their
        /// original components and type remain unchanged; this does not resolve a
        /// media query or apply the ordinary literal's range to a calculated result.
        pub fn serialize_specified(&self) -> SerializationResult<String> {
            self.serialize_specified_with_limits(CssSpecifiedValueSerializationLimits::default())
        }

        /// Shares cumulative visits, numeric projection work and UTF-8 output bytes.
        /// Failure is atomic; no raw-component fallback or contextual evaluation occurs.
        pub fn serialize_specified_with_limits(
            &self,
            limits: CssSpecifiedValueSerializationLimits,
        ) -> SerializationResult<String> {
            let mut writer = crate::specified_rule_serialization::SpecifiedRuleWriter::new(limits);
            self.append_to_rule_writer(&mut writer)?;
            Ok(writer.css)
        }

        pub(crate) fn append_to_rule_writer(
            &self,
            writer: &mut crate::specified_rule_serialization::SpecifiedRuleWriter,
        ) -> std::result::Result<(), crate::CssSpecifiedValueSerializationError> {
            let context = &mut writer.context;
            let output = &mut writer.css;
            self.append_specified(context, output)?;
            Ok(())
        }

        pub(crate) fn append_specified(
            &self,
            context: &mut SpecifiedSerializationContext,
            output: &mut String,
        ) -> SerializationResult<()> {
            match &self.value {
                crate::dimension::DimensionValue::Literal(value) => {
                    value.append_specified(context, output)
                }
                crate::dimension::DimensionValue::Calculation(value) => {
                    crate::numeric::project_specified_into(&value.expression, context, output)
                        .map(|_| ())
                }
            }
        }
    }
);

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{CssComponentValueRef, CssResolutionCalculation, parse_component_values};
    use std::error::Error;

    #[test]
    fn resolution_children_share_consumed_input_projection_and_output_budgets() {
        let component = parse_component_values("-0dppx").unwrap().items()[0].clone();
        let value = CssResolutionLiteral::try_from_component(component).unwrap();
        let mut context =
            SpecifiedSerializationContext::new(CssSpecifiedValueSerializationLimits::new(2, 2, 10));
        let mut output = String::new();
        value.append_specified(&mut context, &mut output).unwrap();
        value.append_specified(&mut context, &mut output).unwrap();
        assert_eq!(output, "0dppx0dppx");
        assert_eq!(context.remaining_bytes(), 0);
        assert_eq!(
            value
                .append_specified(&mut context, &mut output)
                .unwrap_err()
                .kind(),
            CssSpecifiedValueSerializationErrorKind::InputNodeLimit,
        );
        for (projection, bytes, expected) in [
            (
                1,
                10,
                CssSpecifiedValueSerializationErrorKind::ProjectionNodeLimit,
            ),
            (2, 9, CssSpecifiedValueSerializationErrorKind::ByteLimit),
        ] {
            let mut context = SpecifiedSerializationContext::new(
                CssSpecifiedValueSerializationLimits::new(2, projection, bytes),
            );
            let mut output = String::new();
            value.append_specified(&mut context, &mut output).unwrap();
            assert_eq!(
                value
                    .append_specified(&mut context, &mut output)
                    .unwrap_err()
                    .kind(),
                expected,
            );
            assert_eq!(output, "0dppx");
        }
    }

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
