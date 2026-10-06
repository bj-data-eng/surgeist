//! Canonical specified text values, before indentation, alignment or painting.
//!
//! Indent and decoration aggregates cost one input and projection node. Each
//! present indent flag costs one of each. A decoration-line aggregate costs one
//! plus each keyword, including `none`. Vertical-align and thickness keywords
//! cost one node; numeric variants delegate without charging their Rust carrier.
//! Numeric/color children keep their owning costs. Punctuation costs only bytes.

use crate::{
    CssSpecifiedValueSerializationError, CssSpecifiedValueSerializationErrorKind,
    CssSpecifiedValueSerializationLimits, CssTabSize, CssTextDecoration, CssTextDecorationLine,
    CssTextDecorationLineComponent, CssTextDecorationThickness, CssTextIndent, CssVerticalAlign,
    specified_rule_serialization::SpecifiedRuleWriter,
};

type Result<T> = std::result::Result<T, CssSpecifiedValueSerializationError>;

macro_rules! specified_methods {
    () => {
        /// Emits canonical specified syntax from retained semantic fields.
        /// Optional values remain omitted; explicit initials remain explicit.
        /// This does not resolve percentages or floor actual decoration thickness.
        pub fn serialize_specified(&self) -> Result<String> {
            self.serialize_specified_with_limits(CssSpecifiedValueSerializationLimits::default())
        }

        /// Emits atomically under cumulative input, projection and CSS byte limits.
        /// Children share one context; failure returns no partial CSS and leaves
        /// the original exact coefficients, units, field order and origins intact.
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

impl CssTabSize {
    specified_methods!();
    pub(crate) fn append_to_rule_writer(&self, writer: &mut SpecifiedRuleWriter) -> Result<()> {
        match self {
            Self::Number(value) => value.append_to_rule_writer(writer),
            Self::Length(value) => value.append_to_rule_writer(writer),
        }
    }
}

impl CssTextIndent {
    specified_methods!();

    pub(crate) fn append_to_rule_writer(&self, writer: &mut SpecifiedRuleWriter) -> Result<()> {
        writer.node()?;
        self.length().append_to_rule_writer(writer)?;
        if self.hanging() {
            writer.append(" ")?;
            writer.keyword("hanging")?;
        }
        if self.each_line() {
            writer.append(" ")?;
            writer.keyword("each-line")?;
        }
        Ok(())
    }
}

impl CssVerticalAlign {
    specified_methods!();

    pub(crate) fn append_to_rule_writer(&self, writer: &mut SpecifiedRuleWriter) -> Result<()> {
        let keyword = match self {
            Self::Baseline => "baseline",
            Self::Sub => "sub",
            Self::Super => "super",
            Self::TextTop => "text-top",
            Self::TextBottom => "text-bottom",
            Self::Middle => "middle",
            Self::Top => "top",
            Self::Bottom => "bottom",
            Self::Length(value) => return value.append_to_rule_writer(writer),
        };
        writer.keyword(keyword)
    }
}

fn unrepresentable() -> CssSpecifiedValueSerializationError {
    CssSpecifiedValueSerializationError::new(
        CssSpecifiedValueSerializationErrorKind::UnrepresentableValue,
    )
}

impl CssTextDecorationLine {
    specified_methods!();

    pub(crate) fn append_to_rule_writer(&self, writer: &mut SpecifiedRuleWriter) -> Result<()> {
        writer.node()?;
        if self.is_none() {
            if !self.components().is_empty() {
                return Err(unrepresentable());
            }
            return writer.keyword("none");
        }
        if let Some(error) = self.error_kind() {
            if !self.components().is_empty() {
                return Err(unrepresentable());
            }
            return writer.keyword(match error {
                crate::CssTextDecorationError::SpellingError => "spelling-error",
                crate::CssTextDecorationError::GrammarError => "grammar-error",
            });
        }
        // The closed keyword grammar needs only four flags, independent of
        // authored order. Visit the original children once, then emit in grammar
        // order. Invalid internal states fail instead of losing or echoing data.
        let mut present = [false; 4];
        for component in self.components() {
            writer.context.charge_input(1)?;
            let index = match component {
                CssTextDecorationLineComponent::Underline => 0,
                CssTextDecorationLineComponent::Overline => 1,
                CssTextDecorationLineComponent::LineThrough => 2,
                CssTextDecorationLineComponent::Blink => 3,
            };
            if std::mem::replace(&mut present[index], true) {
                return Err(unrepresentable());
            }
        }
        let mut started = false;
        for (present, keyword) in
            present
                .into_iter()
                .zip(["underline", "overline", "line-through", "blink"])
        {
            if present {
                separator(writer, &mut started)?;
                writer.context.charge_projection(1)?;
                writer.append(keyword)?;
            }
        }
        if !started {
            return Err(unrepresentable());
        }
        Ok(())
    }
}

impl CssTextDecorationThickness {
    specified_methods!();

    pub(crate) fn append_to_rule_writer(&self, writer: &mut SpecifiedRuleWriter) -> Result<()> {
        match self {
            Self::Auto => writer.keyword("auto"),
            Self::FromFont => writer.keyword("from-font"),
            Self::Length(value) => value.append_to_rule_writer(writer),
        }
    }
}

fn separator(writer: &mut SpecifiedRuleWriter, started: &mut bool) -> Result<()> {
    if *started {
        writer.append(" ")?;
    }
    *started = true;
    Ok(())
}

impl CssTextDecoration {
    specified_methods!();

    pub(crate) fn append_to_rule_writer(&self, writer: &mut SpecifiedRuleWriter) -> Result<()> {
        writer.node()?;
        let mut started = false;
        // Selected Text Decoration 4 §2.6: line || thickness || style || color.
        if let Some(line) = self.line() {
            writer.source_property(crate::CssKnownProperty::TextDecorationLine, |writer| {
                separator(writer, &mut started)?;
                line.append_to_rule_writer(writer)
            })?;
        }
        if let Some(thickness) = self.thickness() {
            writer.source_property(crate::CssKnownProperty::TextDecorationThickness, |writer| {
                separator(writer, &mut started)?;
                thickness.append_to_rule_writer(writer)
            })?;
        }
        if let Some(style) = self.style() {
            writer.source_property(crate::CssKnownProperty::TextDecorationStyle, |writer| {
                separator(writer, &mut started)?;
                style.append_to_rule_writer(writer)
            })?;
        }
        if let Some(color) = self.color() {
            writer.source_property(crate::CssKnownProperty::TextDecorationColor, |writer| {
                separator(writer, &mut started)?;
                color.append_specified(&mut writer.context, &mut writer.css)
            })?;
        }
        if !started {
            return Err(unrepresentable());
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{CssColor, CssSpecifiedLengthPercentage, CssTextDecorationStyle};
    use CssSpecifiedValueSerializationErrorKind as K;
    use CssSpecifiedValueSerializationLimits as L;

    fn length() -> CssSpecifiedLengthPercentage {
        CssSpecifiedLengthPercentage::try_from_component(
            crate::CssComponentValue::try_token("-1px").unwrap(),
        )
        .unwrap()
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
        let mut writer = SpecifiedRuleWriter::new(L::new(input * 2, projection * 2, css.len()));
        writer.without_output(&append).unwrap();
        assert!(writer.css.is_empty());
        append(&mut writer).unwrap();
        assert_eq!(writer.css, css);
        for (limits, kind) in [
            (L::new(input - 1, projection, 1), K::InputNodeLimit),
            (L::new(input, projection - 1, 1), K::ProjectionNodeLimit),
        ] {
            let mut writer = SpecifiedRuleWriter::new(limits);
            writer
                .without_output(|writer| {
                    assert_eq!(writer.without_output(&append).unwrap_err().kind(), kind);
                    assert!(writer.context.output_suppressed());
                    Ok(())
                })
                .unwrap();
            assert!(!writer.context.output_suppressed());
            writer.append("x").unwrap();
            assert_eq!(writer.css, "x");
        }
    }

    #[test]
    fn every_text_provider_shares_prefix_sibling_and_suppressed_work_budgets() {
        let value = CssTextIndent::new(length(), true, true);
        shared("-1px hanging each-line", 4, 4, |writer| {
            value.append_to_rule_writer(writer)
        });
        let value = CssVerticalAlign::Baseline;
        shared("baseline", 1, 1, |writer| {
            value.append_to_rule_writer(writer)
        });
        let value = CssVerticalAlign::Length(length());
        shared("-1px", 1, 1, |writer| value.append_to_rule_writer(writer));
        let value = CssTextDecorationLine::try_new(vec![
            CssTextDecorationLineComponent::Overline,
            CssTextDecorationLineComponent::Underline,
        ])
        .unwrap();
        shared("underline overline", 3, 3, |writer| {
            value.append_to_rule_writer(writer)
        });
        let value = CssTextDecorationLine::none();
        shared("none", 2, 2, |writer| value.append_to_rule_writer(writer));
        let value = CssTextDecorationThickness::Auto;
        shared("auto", 1, 1, |writer| value.append_to_rule_writer(writer));
        let value = CssTextDecorationThickness::Length(length());
        shared("-1px", 1, 1, |writer| value.append_to_rule_writer(writer));
        let value = CssTextDecoration::try_new(
            Some(
                CssTextDecorationLine::try_new(vec![
                    CssTextDecorationLineComponent::Underline,
                    CssTextDecorationLineComponent::Overline,
                ])
                .unwrap(),
            ),
            Some(CssColor::current_color()),
            Some(CssTextDecorationStyle::Wavy),
            Some(CssTextDecorationThickness::Length(length())),
        )
        .unwrap();
        shared(
            "underline overline -1px wavy currentcolor",
            7,
            7,
            |writer| value.append_to_rule_writer(writer),
        );
    }

    #[test]
    fn suppressed_thickness_and_indent_math_still_project_every_numeric_child() {
        let number = CssSpecifiedLengthPercentage::try_from_calculation(
            crate::CssLengthPercentageCalculation::try_from_components(
                crate::parse_component_values("calc(1px + 2px)").unwrap(),
            )
            .unwrap(),
        )
        .unwrap();
        // Calc + Sum + two scalar leaves: four input nodes; the two leaves and
        // their combined scalar: three projections. Carrier enums add no node.
        let value = CssTextDecorationThickness::Length(number.clone());
        shared("calc(3px)", 4, 3, |writer| {
            value.append_to_rule_writer(writer)
        });
        let value = CssTextIndent::new(number, true, true);
        shared("calc(3px) hanging each-line", 7, 6, |writer| {
            value.append_to_rule_writer(writer)
        });
    }

    #[test]
    fn invalid_internal_line_or_empty_decoration_fails_instead_of_losing_fields() {
        for line in [
            CssTextDecorationLine::new(vec![]),
            CssTextDecorationLine::new(vec![CssTextDecorationLineComponent::Underline; 2]),
        ] {
            assert_eq!(
                line.serialize_specified().unwrap_err().kind(),
                K::UnrepresentableValue
            );
            let mut writer = SpecifiedRuleWriter::new(L::new(100, 100, 1));
            writer
                .without_output(|writer| {
                    assert_eq!(
                        line.append_to_rule_writer(writer).unwrap_err().kind(),
                        K::UnrepresentableValue
                    );
                    Ok(())
                })
                .unwrap();
            writer.append("x").unwrap();
            assert_eq!(writer.css, "x");
        }
        let value = CssTextDecoration::new(None, None, None, None);
        assert_eq!(
            value.serialize_specified().unwrap_err().kind(),
            K::UnrepresentableValue
        );
    }

    #[test]
    fn cumulative_overflow_is_typed_and_suppressed_errors_restore_output_mode() {
        let value = CssTextIndent::new(length(), false, false);
        let mut writer = SpecifiedRuleWriter::new(L::new(usize::MAX, usize::MAX, 1));
        writer.context.charge_input(usize::MAX).unwrap();
        let error = writer
            .without_output(|writer| value.append_to_rule_writer(writer))
            .unwrap_err();
        assert_eq!(error.kind(), K::CapacityOverflow);
        assert!(!writer.context.output_suppressed());
        writer.append("x").unwrap();
        assert_eq!(writer.css, "x");
    }

    #[test]
    fn tab_size_numeric_carriers_share_existing_work_and_suppression_without_aggregate_cost() {
        let number = CssTabSize::Number(
            crate::CssSpecifiedNonNegativeNumber::try_from_component(
                crate::CssComponentValue::try_number("8").unwrap(),
            )
            .unwrap(),
        );
        shared("8", 1, 1, |w| number.append_to_rule_writer(w));
        let length = CssTabSize::Length(
            crate::CssSpecifiedNonNegativeLength::try_from_component(
                crate::CssComponentValue::try_dimension("0", "px").unwrap(),
            )
            .unwrap(),
        );
        shared("0px", 1, 1, |w| length.append_to_rule_writer(w));
        let math = CssTabSize::Length(
            crate::CssSpecifiedNonNegativeLength::try_from_calculation(
                crate::CssLengthCalculation::try_from_components(
                    crate::parse_component_values("calc(1px + 2em)").unwrap(),
                )
                .unwrap(),
            )
            .unwrap(),
        );
        shared("calc(2em + 1px)", 4, 5, |w| math.append_to_rule_writer(w));
    }
}
