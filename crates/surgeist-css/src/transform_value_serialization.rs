//! Bounded specified transform syntax, before contextual matrix execution.
//!
//! A transform list, individual value, origin and function each cost one input
//! and projection node. Origin also visits its planar position owner. Numeric
//! children retain their own costs, even when an exact ordinary default is
//! omitted. Punctuation costs bytes only. Rotation's synthesized components cost
//! projection nodes in its owner. Six-place formatting never selects omission.

use crate::{
    CssAngleOrZero, CssComponentValue, CssComponentValueRef, CssScale,
    CssSpecifiedLengthPercentage, CssSpecifiedPercentage, CssSpecifiedValueSerializationError,
    CssSpecifiedValueSerializationLimits, CssTransform, CssTransformFunction, CssTransformOrigin,
    CssTransformPerspective, CssTransformScaleComponent, CssTranslate, CssValueTokenRef,
    exact_decimal::LexicalDecimal,
    numeric::{NumericProjectionScale, capture_specified_scaled},
    specified_rule_serialization::SpecifiedRuleWriter,
};

type Result<T> = std::result::Result<T, CssSpecifiedValueSerializationError>;

macro_rules! specified_methods {
    () => {
        /// Emits canonical specified syntax from retained typed operands.
        /// Exact ordinary defaults may be omitted; authored fields and origins
        /// remain unchanged. Percentages and calculations retain their owning
        /// specified policy without resolving reference boxes or matrices.
        pub fn serialize_specified(&self) -> Result<String> {
            self.serialize_specified_with_limits(CssSpecifiedValueSerializationLimits::default())
        }

        /// Uses one cumulative input, projection and UTF-8 byte context.
        /// Numeric output retains six-place formatting. Failure returns no
        /// partial CSS and preserves the authored value. General rotate3d axes
        /// share the Rotate owner's approximation and representability policy.
        pub fn serialize_specified_with_limits(
            &self,
            limits: CssSpecifiedValueSerializationLimits,
        ) -> Result<String> {
            let mut writer = SpecifiedRuleWriter::new(limits);
            self.append_to_rule_writer(&mut writer)?;
            Ok(writer.css)
        }
    };
}

fn ordinary_zero(component: &CssComponentValue, allow_percentage: bool) -> bool {
    let number = match component.view() {
        CssComponentValueRef::Token(CssValueTokenRef::Number(number))
        | CssComponentValueRef::Token(CssValueTokenRef::Dimension { number, .. }) => number,
        CssComponentValueRef::Token(CssValueTokenRef::Percentage(number)) if allow_percentage => {
            number
        }
        _ => return false,
    };
    LexicalDecimal::new(number.representation()).len == 0
}

fn zero_translation(value: &CssSpecifiedLengthPercentage, allow_percentage: bool) -> bool {
    value
        .literal_component()
        .is_some_and(|value| ordinary_zero(value, allow_percentage))
}

fn ordinary_one(value: &crate::CssSpecifiedNumber) -> bool {
    let Some(component) = value.literal_component() else {
        return false;
    };
    let CssComponentValueRef::Token(CssValueTokenRef::Number(number)) = component.view() else {
        unreachable!("checked number literal")
    };
    LexicalDecimal::new(number.representation()).value_eq(&LexicalDecimal::new("1"))
}

fn ordinary_numbers_equal(
    left: &crate::CssSpecifiedNumber,
    right: &crate::CssSpecifiedNumber,
) -> bool {
    match (left.literal_component(), right.literal_component()) {
        (Some(left), Some(right)) => crate::specified_numeric::ordinary_literal_equal(left, right),
        _ => false,
    }
}

fn zero_angle(value: &CssAngleOrZero) -> bool {
    match value {
        CssAngleOrZero::Zero(_) => true,
        CssAngleOrZero::Angle(value) => value.literal().is_some_and(|literal| {
            LexicalDecimal::new(literal.numeric().representation()).len == 0
        }),
    }
}

impl CssTransformOrigin {
    specified_methods!();

    pub(crate) fn append_to_rule_writer(&self, writer: &mut SpecifiedRuleWriter) -> Result<()> {
        writer.node()?;
        self.append_planar_to_rule_writer(writer)?;
        if let Some(z) = self.z() {
            writer.append(" ")?;
            z.append_to_rule_writer(writer)?;
        }
        Ok(())
    }
}

impl CssTranslate {
    specified_methods!();

    pub(crate) fn append_to_rule_writer(&self, writer: &mut SpecifiedRuleWriter) -> Result<()> {
        writer.node()?;
        let Self::Values(values) = self else {
            return writer.append("none");
        };
        values.x().append_to_rule_writer(writer)?;
        let retain_z = values.z().is_some_and(|z| {
            !z.literal_component()
                .is_some_and(|value| ordinary_zero(value, false))
        });
        if let Some(y) = values.y() {
            if retain_z || !zero_translation(y, false) {
                writer.append(" ")?;
                y.append_to_rule_writer(writer)?;
            } else {
                writer.without_output(|writer| y.append_to_rule_writer(writer))?;
            }
        }
        if let Some(z) = values.z() {
            if retain_z {
                writer.append(" ")?;
                z.append_to_rule_writer(writer)?;
            } else {
                writer.without_output(|writer| z.append_to_rule_writer(writer))?;
            }
        }
        Ok(())
    }
}

impl CssScale {
    specified_methods!();

    pub(crate) fn append_to_rule_writer(&self, writer: &mut SpecifiedRuleWriter) -> Result<()> {
        writer.node()?;
        let Self::Values(values) = self else {
            return writer.append("none");
        };
        let values = values.values();
        // Public construction already guarantees one to three literal numbers.
        let retain_z = values.get(2).is_some_and(|z| !ordinary_one(z));
        values[0].append_to_rule_writer(writer)?;
        if let Some(y) = values.get(1) {
            if retain_z || !ordinary_numbers_equal(&values[0], y) {
                writer.append(" ")?;
                y.append_to_rule_writer(writer)?;
            } else {
                writer.without_output(|writer| y.append_to_rule_writer(writer))?;
            }
        }
        if let Some(z) = values.get(2) {
            if retain_z {
                writer.append(" ")?;
                z.append_to_rule_writer(writer)?;
            } else {
                writer.without_output(|writer| z.append_to_rule_writer(writer))?;
            }
        }
        Ok(())
    }
}

impl CssTransform {
    specified_methods!();

    pub(crate) fn append_to_rule_writer(&self, writer: &mut SpecifiedRuleWriter) -> Result<()> {
        // The enum is the list's semantic carrier, without another node charge.
        writer.node()?;
        let Self::Functions(functions) = self else {
            return writer.append("none");
        };
        for (index, function) in functions.functions().iter().enumerate() {
            if index != 0 {
                writer.append(" ")?;
            }
            append_function(function, writer)?;
        }
        Ok(())
    }
}

fn append_function(
    function: &CssTransformFunction,
    writer: &mut SpecifiedRuleWriter,
) -> Result<()> {
    writer.node()?;
    match function {
        CssTransformFunction::Matrix(value) => {
            writer.append("matrix(")?;
            append_numbers(value.components(), writer)?;
        }
        CssTransformFunction::Matrix3d(value) => {
            writer.append("matrix3d(")?;
            append_numbers(value.components(), writer)?;
        }
        CssTransformFunction::Perspective(value) => {
            writer.append("perspective(")?;
            match value {
                CssTransformPerspective::None => writer.keyword("none")?,
                CssTransformPerspective::Length(value) => value.append_to_rule_writer(writer)?,
            }
        }
        CssTransformFunction::Rotate(value) => append_angle("rotate(", value, writer)?,
        CssTransformFunction::RotateX(value) => append_angle("rotateX(", value, writer)?,
        CssTransformFunction::RotateY(value) => append_angle("rotateY(", value, writer)?,
        CssTransformFunction::RotateZ(value) => append_angle("rotateZ(", value, writer)?,
        CssTransformFunction::Rotate3d(value) => {
            writer.append("rotate3d(")?;
            value.append_rotation_arguments_to_rule_writer(writer)?;
        }
        CssTransformFunction::Scale(value) => {
            writer.append("scale(")?;
            value.x().append_to_rule_writer(writer)?;
            if let Some(y) = value.y() {
                if ordinary_numbers_equal(value.x(), y) {
                    writer.without_output(|writer| y.append_to_rule_writer(writer))?;
                } else {
                    writer.append(", ")?;
                    y.append_to_rule_writer(writer)?;
                }
            }
        }
        CssTransformFunction::Scale3d(value) => {
            writer.append("scale3d(")?;
            append_scale_component(value.x(), writer)?;
            writer.append(", ")?;
            append_scale_component(value.y(), writer)?;
            writer.append(", ")?;
            append_scale_component(value.z(), writer)?;
        }
        CssTransformFunction::ScaleX(value) => {
            writer.append("scaleX(")?;
            value.append_to_rule_writer(writer)?;
        }
        CssTransformFunction::ScaleY(value) => {
            writer.append("scaleY(")?;
            value.append_to_rule_writer(writer)?;
        }
        CssTransformFunction::ScaleZ(value) => {
            writer.append("scaleZ(")?;
            append_scale_component(value, writer)?;
        }
        CssTransformFunction::Skew(value) => {
            append_angle("skew(", value.x(), writer)?;
            if let Some(y) = value.y() {
                if zero_angle(y) {
                    writer.without_output(|writer| {
                        y.append_specified(&mut writer.context, &mut writer.css)
                    })?;
                } else {
                    writer.append(", ")?;
                    y.append_specified(&mut writer.context, &mut writer.css)?;
                }
            }
        }
        CssTransformFunction::SkewX(value) => append_angle("skewX(", value, writer)?,
        CssTransformFunction::SkewY(value) => append_angle("skewY(", value, writer)?,
        CssTransformFunction::Translate(value) => {
            writer.append("translate(")?;
            value.x().append_to_rule_writer(writer)?;
            if let Some(y) = value.y() {
                if zero_translation(y, true) {
                    writer.without_output(|writer| y.append_to_rule_writer(writer))?;
                } else {
                    writer.append(", ")?;
                    y.append_to_rule_writer(writer)?;
                }
            }
        }
        CssTransformFunction::Translate3d(value) => {
            writer.append("translate3d(")?;
            value.x().append_to_rule_writer(writer)?;
            writer.append(", ")?;
            value.y().append_to_rule_writer(writer)?;
            writer.append(", ")?;
            value.z().append_to_rule_writer(writer)?;
        }
        CssTransformFunction::TranslateX(value) => {
            writer.append("translateX(")?;
            value.append_to_rule_writer(writer)?;
        }
        CssTransformFunction::TranslateY(value) => {
            writer.append("translateY(")?;
            value.append_to_rule_writer(writer)?;
        }
        CssTransformFunction::TranslateZ(value) => {
            writer.append("translateZ(")?;
            value.append_to_rule_writer(writer)?;
        }
    }
    writer.append(")")
}

fn append_numbers(
    values: &[crate::CssSpecifiedNumber],
    writer: &mut SpecifiedRuleWriter,
) -> Result<()> {
    for (index, value) in values.iter().enumerate() {
        if index != 0 {
            writer.append(", ")?;
        }
        value.append_to_rule_writer(writer)?;
    }
    Ok(())
}

fn append_angle(
    opening: &str,
    value: &CssAngleOrZero,
    writer: &mut SpecifiedRuleWriter,
) -> Result<()> {
    writer.append(opening)?;
    value.append_specified(&mut writer.context, &mut writer.css)
}

fn append_scale_component(
    value: &CssTransformScaleComponent,
    writer: &mut SpecifiedRuleWriter,
) -> Result<()> {
    match value {
        CssTransformScaleComponent::Number(value) => value.append_to_rule_writer(writer),
        // This is already a Number result, with its own unresolved dimensional
        // hint. Scaling the whole expression by 1/100 would change its meaning.
        CssTransformScaleComponent::HintedNumberCalculation(value) => {
            value.append_specified(&mut writer.context, &mut writer.css)
        }
        CssTransformScaleComponent::Percentage(value) => append_percentage_as_number(value, writer),
    }
}

fn append_percentage_as_number(
    value: &CssSpecifiedPercentage,
    writer: &mut SpecifiedRuleWriter,
) -> Result<()> {
    if let Some(component) = value.literal_component() {
        writer.node()?;
        if writer.context.output_suppressed() {
            return Ok(());
        }
        let CssComponentValueRef::Token(CssValueTokenRef::Percentage(number)) = component.view()
        else {
            unreachable!("checked percentage literal")
        };
        // Transforms 2 §12: scale percentages serialize as their Number factor.
        let text = crate::numeric_formatting::format_css_number(
            number.representation(),
            -2,
            writer.context.remaining_bytes(),
        )?;
        writer.append(&text)
    } else {
        let (text, _) = capture_specified_scaled(
            &value
                .calculation()
                .expect("checked percentage calculation")
                .expression,
            NumericProjectionScale::PercentageToNumber {
                numerator: 1,
                denominator: 100,
            },
            &mut writer.context,
        )?;
        writer.append(&text)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{CssKnownPropertyValueRef, CssSpecifiedValueSerializationErrorKind as K};
    type L = CssSpecifiedValueSerializationLimits;

    fn transform(source: &str) -> CssTransform {
        let report = crate::parse_style_attribute(&format!("transform:{source}"));
        assert!(report.is_clean(), "{:?}", report.diagnostics());
        let CssKnownPropertyValueRef::Transform(value) = report.syntax()[0]
            .known()
            .unwrap()
            .property_value()
            .unwrap()
        else {
            panic!("transform")
        };
        value.value().clone()
    }

    fn shared(
        css: &str,
        input: usize,
        projection: usize,
        append: impl Fn(&mut SpecifiedRuleWriter) -> Result<()>,
    ) {
        let expected = format!("p:{css};{css}");
        let mut writer =
            SpecifiedRuleWriter::new(L::new(input * 2, projection * 2, expected.len()));
        writer.append("p:").unwrap();
        append(&mut writer).unwrap();
        writer.append(";").unwrap();
        append(&mut writer).unwrap();
        assert_eq!(writer.css, expected);
        for (limits, kind) in [
            (
                L::new(input * 2 - 1, projection * 2, expected.len()),
                K::InputNodeLimit,
            ),
            (
                L::new(input * 2, projection * 2 - 1, expected.len()),
                K::ProjectionNodeLimit,
            ),
            (
                L::new(input * 2, projection * 2, expected.len() - 1),
                K::ByteLimit,
            ),
        ] {
            let mut writer = SpecifiedRuleWriter::new(limits);
            writer.append("p:").unwrap();
            append(&mut writer).unwrap();
            writer.append(";").unwrap();
            assert_eq!(append(&mut writer).unwrap_err().kind(), kind);
        }
        let mut writer = SpecifiedRuleWriter::new(L::new(input * 2, projection * 2, css.len() + 2));
        writer.append("p:").unwrap();
        writer.without_output(|writer| append(writer)).unwrap();
        append(&mut writer).unwrap();
        assert_eq!(writer.css, format!("p:{css}"));
        assert_eq!(
            writer.context.charge_input(1).unwrap_err().kind(),
            K::InputNodeLimit
        );
        assert_eq!(
            writer.context.charge_projection(1).unwrap_err().kind(),
            K::ProjectionNodeLimit
        );

        // Both nested suppression levels restore the enclosing output mode,
        // including a child failure after partial cumulative work.
        for (limits, kind) in [
            (L::new(input - 1, projection, 1), K::InputNodeLimit),
            (L::new(input, projection - 1, 1), K::ProjectionNodeLimit),
        ] {
            let mut writer = SpecifiedRuleWriter::new(limits);
            assert_eq!(
                writer
                    .without_output(|writer| writer.without_output(|writer| append(writer)))
                    .unwrap_err()
                    .kind(),
                kind
            );
            writer.append("x").unwrap();
            assert_eq!(writer.css, "x");
        }
    }

    #[test]
    fn every_provider_borrows_prefix_sibling_budgets_and_restores_suppression() {
        let value = transform("scale(2, 2) translate(1px, 0px)");
        shared("scale(2) translate(1px)", 7, 7, |writer| {
            value.append_to_rule_writer(writer)
        });
        let translate = CssTranslate::Values(
            crate::CssTranslateValues::try_new(
                crate::CssSpecifiedLengthPercentage::try_from_component(
                    crate::CssComponentValue::try_token("1px").unwrap(),
                )
                .unwrap(),
                Some(
                    crate::CssSpecifiedLengthPercentage::try_from_component(
                        crate::CssComponentValue::try_token("0px").unwrap(),
                    )
                    .unwrap(),
                ),
                Some(
                    crate::CssSpecifiedLength::try_from_component(
                        crate::CssComponentValue::try_token("0px").unwrap(),
                    )
                    .unwrap(),
                ),
            )
            .unwrap(),
        );
        shared("1px", 4, 4, |writer| {
            translate.append_to_rule_writer(writer)
        });
        let scale = CssScale::Values(
            crate::CssScaleValues::try_new(
                ["2", "2", "1"]
                    .map(|text| {
                        crate::CssSpecifiedNumber::try_from_component(
                            crate::CssComponentValue::try_number(text).unwrap(),
                        )
                        .unwrap()
                    })
                    .to_vec(),
            )
            .unwrap(),
        );
        shared("2", 4, 4, |writer| scale.append_to_rule_writer(writer));
        let origin = CssTransformOrigin::try_new(
            crate::CssPhysicalPosition::try_new(
                crate::CssHorizontalPosition::Left,
                crate::CssVerticalPosition::Top,
            )
            .unwrap(),
            None,
        )
        .unwrap();
        shared("left top", 4, 4, |writer| {
            origin.append_to_rule_writer(writer)
        });
        shared("none", 1, 1, |writer| {
            CssTransform::None.append_to_rule_writer(writer)
        });
        shared("none", 1, 1, |writer| {
            CssTranslate::None.append_to_rule_writer(writer)
        });
        shared("none", 1, 1, |writer| {
            CssScale::None.append_to_rule_writer(writer)
        });
    }

    #[test]
    fn math_conversion_and_rotation_replacements_visit_under_suppression() {
        let value = transform("translateX(calc(1px + 2px))");
        shared("translateX(calc(3px))", 6, 5, |writer| {
            value.append_to_rule_writer(writer)
        });
        let value = transform("scaleZ(50%)");
        shared("scaleZ(0.5)", 3, 3, |writer| {
            value.append_to_rule_writer(writer)
        });
        let value = transform("scaleZ(calc(50% + 50%))");
        // Four checked math input nodes, six projection nodes including the
        // percentage divisor, inverse and resolved product; two aggregates.
        shared("scaleZ(calc(1))", 6, 8, |writer| {
            value.append_to_rule_writer(writer)
        });
        let value = transform("rotate3d(0, 0, 0, 30deg)");
        shared("rotate3d(0, 0, 1, 0deg)", 6, 10, |writer| {
            value.append_to_rule_writer(writer)
        });
        let value = transform("rotate3d(-2, 0, 0, 30deg)");
        shared("rotate3d(1, 0, 0, -30deg)", 6, 10, |writer| {
            value.append_to_rule_writer(writer)
        });
    }

    #[test]
    fn suppressed_calculations_preserve_arithmetic_failure_and_restore_output() {
        let value = transform("scaleZ(calc(50% + 50%))");
        let mut writer = SpecifiedRuleWriter::new(L::new(usize::MAX, usize::MAX, 1));
        writer.context.charge_projection(usize::MAX - 3).unwrap();
        assert_eq!(
            writer
                .without_output(|writer| value.append_to_rule_writer(writer))
                .unwrap_err()
                .kind(),
            K::CapacityOverflow
        );
        writer.append("x").unwrap();
        assert_eq!(writer.css, "x");
    }
}
