//! Canonical specified values from the selected Writing Modes publications.

use crate::{
    CssDirection, CssSerializationError, CssTextCombineUpright, CssTextOrientation, CssUnicodeBidi,
    CssWritingMode,
};

impl crate::specified_rule_serialization::SpecifiedProvider for CssDirection {
    fn append_to_rule_writer(
        &self,
        writer: &mut crate::specified_rule_serialization::SpecifiedRuleWriter,
    ) -> std::result::Result<(), crate::CssSerializationError> {
        let text = match self {
            Self::Ltr => "ltr",
            Self::Rtl => "rtl",
        };
        writer.keyword(text)
    }
}
crate::specified_rule_serialization::specified_serialization!(CssDirection);

impl crate::specified_rule_serialization::SpecifiedProvider for CssUnicodeBidi {
    fn append_to_rule_writer(
        &self,
        writer: &mut crate::specified_rule_serialization::SpecifiedRuleWriter,
    ) -> std::result::Result<(), crate::CssSerializationError> {
        let text = match self {
            Self::Normal => "normal",
            Self::Embed => "embed",
            Self::Isolate => "isolate",
            Self::BidiOverride => "bidi-override",
            Self::IsolateOverride => "isolate-override",
            Self::Plaintext => "plaintext",
        };
        writer.keyword(text)
    }
}
crate::specified_rule_serialization::specified_serialization!(CssUnicodeBidi);

impl crate::specified_rule_serialization::SpecifiedProvider for CssWritingMode {
    fn append_to_rule_writer(
        &self,
        writer: &mut crate::specified_rule_serialization::SpecifiedRuleWriter,
    ) -> std::result::Result<(), crate::CssSerializationError> {
        let text = match self {
            Self::HorizontalTb => "horizontal-tb",
            Self::VerticalRl => "vertical-rl",
            Self::VerticalLr => "vertical-lr",
            Self::SidewaysRl => "sideways-rl",
            Self::SidewaysLr => "sideways-lr",
        };
        writer.keyword(text)
    }
}
crate::specified_rule_serialization::specified_serialization!(CssWritingMode);

impl crate::specified_rule_serialization::SpecifiedProvider for CssTextOrientation {
    fn append_to_rule_writer(
        &self,
        writer: &mut crate::specified_rule_serialization::SpecifiedRuleWriter,
    ) -> std::result::Result<(), crate::CssSerializationError> {
        let text = match self {
            Self::Mixed => "mixed",
            Self::Upright => "upright",
            Self::Sideways => "sideways",
        };
        writer.keyword(text)
    }
}
crate::specified_rule_serialization::specified_serialization!(CssTextOrientation);

impl crate::specified_rule_serialization::SpecifiedProvider for CssTextCombineUpright {
    fn append_to_rule_writer(
        &self,
        writer: &mut crate::specified_rule_serialization::SpecifiedRuleWriter,
    ) -> Result<(), CssSerializationError> {
        let count = match self {
            Self::None => return writer.keyword("none"),
            Self::All => return writer.keyword("all"),
            Self::Digits(None) => return writer.keyword("digits"),
            Self::Digits(Some(count)) => count,
        };
        writer.keyword("digits ")?;
        if let Some(calculation) = count.calculation() {
            crate::numeric::project_specified_into(
                &calculation.expression,
                &mut writer.context,
                &mut writer.css,
            )?;
            Ok(())
        } else {
            let text = match count.literal() {
                Some(2) => "2",
                Some(3) => "3",
                Some(4) => "4",
                _ => unreachable!("checked literal count or calculation"),
            };
            writer.keyword(text)
        }
    }
}
crate::specified_rule_serialization::specified_serialization!(CssTextCombineUpright);

#[cfg(test)]
mod combine_cumulative_bridge_contract {
    use super::*;
    use crate::specified_rule_serialization::SpecifiedProvider as _;
    use crate::specified_rule_serialization::SpecifiedRuleWriter;
    use crate::{
        CssCalculation, CssInteger, CssSerializationLimits as Limits,
        CssSpecifiedValueSerializationErrorKind as Kind, CssTextCombineDigitCount, CssValueOrigin,
    };

    #[test]
    fn keywords_omission_and_literal_counts_share_prefix_and_sibling_budgets() {
        for (value, expected, nodes) in [
            (CssTextCombineUpright::None, "none", 1),
            (CssTextCombineUpright::All, "all", 1),
            (CssTextCombineUpright::Digits(None), "digits", 1),
            (
                CssTextCombineUpright::Digits(Some(
                    CssTextCombineDigitCount::try_literal(2).unwrap(),
                )),
                "digits 2",
                2,
            ),
            (
                CssTextCombineUpright::Digits(Some(
                    CssTextCombineDigitCount::try_literal(3).unwrap(),
                )),
                "digits 3",
                2,
            ),
            (
                CssTextCombineUpright::Digits(Some(
                    CssTextCombineDigitCount::try_literal(4).unwrap(),
                )),
                "digits 4",
                2,
            ),
        ] {
            assert_eq!(
                value.serialize_specified(Default::default()).unwrap(),
                expected
            );
            let mut writer = SpecifiedRuleWriter::new(Limits::new(
                2 * nodes + 1,
                2 * nodes + 1,
                2 * expected.len() + 2,
            ));
            writer.context.charge_input(1).unwrap();
            writer.context.charge_projection(1).unwrap();
            writer.append("!").unwrap();
            value.append_to_rule_writer(&mut writer).unwrap();
            writer.append(";").unwrap();
            value.append_to_rule_writer(&mut writer).unwrap();
            assert_eq!(writer.css, format!("!{expected};{expected}"));
            assert_eq!(
                writer.context.charge_input(1).unwrap_err().kind(),
                Kind::InputNodeLimit
            );
            assert_eq!(
                writer.context.charge_projection(1).unwrap_err().kind(),
                Kind::ProjectionNodeLimit
            );
            assert_eq!(writer.append("x").unwrap_err().kind(), Kind::ByteLimit);
            let mut suppressed = SpecifiedRuleWriter::new(Limits::new(nodes, nodes, 0));
            suppressed
                .without_output(|writer| value.append_to_rule_writer(writer))
                .unwrap();
            assert!(suppressed.css.is_empty());
            assert!(!suppressed.context.output_suppressed());
            assert_eq!(
                suppressed.context.charge_input(1).unwrap_err().kind(),
                Kind::InputNodeLimit
            );
            assert_eq!(
                suppressed.context.charge_projection(1).unwrap_err().kind(),
                Kind::ProjectionNodeLimit
            );
        }
    }

    #[test]
    fn programmatic_literal_calculation_keeps_calc_category() {
        let count =
            CssTextCombineDigitCount::from_calculation(CssCalculation::<CssInteger>::literal(1));
        let value = CssTextCombineUpright::Digits(Some(count));
        let mut writer = SpecifiedRuleWriter::new(Limits::default());
        writer.append("!").unwrap();
        value.append_to_rule_writer(&mut writer).unwrap();
        assert_eq!(writer.css, "!digits calc(1)");
        assert_eq!(
            value.serialize_specified(Default::default()).unwrap(),
            "digits calc(1)"
        );
        let CssTextCombineUpright::Digits(Some(count)) = value else {
            panic!("explicit count");
        };
        assert!(count.literal().is_none());
        assert!(matches!(
            count.calculation().unwrap().origin(),
            CssValueOrigin::Programmatic
        ));
    }

    #[test]
    fn suppressed_symbolic_calculation_preserves_parsed_origin_without_byte_work() {
        let text = "digits calc(1px / 1em)";
        let report = crate::CssParser::default()
            .parse_block_contents(format!("/*😀*/text-combine-upright:{text}"));
        assert!(report.is_clean(), "{:?}", report.diagnostics());
        let crate::CssKnownPropertyValueRef::TextCombineUpright(wrapper) = report.syntax()[0]
            .known()
            .unwrap()
            .property_value()
            .unwrap()
        else {
            panic!("combine");
        };
        let value = wrapper.value();
        let before = value.clone();
        let mut writer = SpecifiedRuleWriter::new(Limits::new(100, 100, 1));
        writer.append("x").unwrap();
        writer
            .without_output(|writer| value.append_to_rule_writer(writer))
            .unwrap();
        assert_eq!(writer.css, "x");
        assert!(!writer.context.output_suppressed());
        assert_eq!(*value, before);
        assert_eq!(value.serialize_specified(Default::default()).unwrap(), text);
        let CssTextCombineUpright::Digits(Some(count)) = value else {
            panic!("explicit count");
        };
        let CssValueOrigin::Parsed(origin) = count.calculation().unwrap().origin() else {
            panic!("parsed origin");
        };
        let start = origin.source().as_str().find("calc(").unwrap();
        assert_eq!(origin.span().start().byte_offset().value(), start);
        assert_eq!(
            origin.span().end().byte_offset().value(),
            start + "calc(".len()
        );
    }

    #[test]
    fn suppressed_count_failure_restores_nested_mode_and_public_error_order() {
        let value =
            CssTextCombineUpright::Digits(Some(CssTextCombineDigitCount::try_literal(2).unwrap()));
        let mut writer = SpecifiedRuleWriter::new(Limits::new(1, 100, 1));
        let error = writer
            .without_output(|writer| {
                let error = writer
                    .without_output(|writer| value.append_to_rule_writer(writer))
                    .unwrap_err();
                assert!(writer.context.output_suppressed());
                Err::<(), _>(error)
            })
            .unwrap_err();
        assert_eq!(error.kind(), Kind::InputNodeLimit);
        assert!(!writer.context.output_suppressed());
        writer.append("x").unwrap();
        assert_eq!(writer.css, "x");
        assert_eq!(
            value
                .serialize_specified(Limits::new(1, 1, 6))
                .unwrap_err()
                .kind(),
            Kind::ByteLimit
        );
    }
}
