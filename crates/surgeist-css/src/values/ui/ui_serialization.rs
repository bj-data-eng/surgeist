//! Canonical authored UI4 projection with retained optional fields and cumulative providers.
use crate::serialization::CssSerializationError;
use crate::serialization::specified_rule_serialization::SpecifiedRuleWriter;
use crate::values::ui::CssCaretColor;
use crate::values::ui::values::*;
type Result<T> = std::result::Result<T, CssSerializationError>;
macro_rules! keywords {
    ($ty:ty, {$($variant:ident => $text:literal),+}) => {
        impl crate::serialization::specified_rule_serialization::SpecifiedProvider for $ty {
            fn append_to_rule_writer(&self, writer: &mut SpecifiedRuleWriter) -> Result<()> {
                writer.keyword(match self { $(Self::$variant => $text),+ })
            }
        }
        crate::serialization::specified_rule_serialization::specified_serialization!($ty);
    };
}
keywords!(CssCaretAnimation, { Auto => "auto", Manual => "manual" });
keywords!(CssCaretShape, { Auto => "auto", Bar => "bar", Block => "block", Underscore => "underscore" });
keywords!(CssInteractivity, { Auto => "auto", Inert => "inert" });
keywords!(CssAppearance, { None => "none", Auto => "auto", Base => "base", BaseSelect => "base-select", Searchfield => "searchfield",
    Textarea => "textarea", Checkbox => "checkbox", Radio => "radio", Menulist => "menulist", Listbox => "listbox", Meter => "meter",
    ProgressBar => "progress-bar", Button => "button", Textfield => "textfield", MenulistButton => "menulist-button" });
crate::serialization::specified_rule_serialization::specified_serialization!(CssAccentColor);
crate::serialization::specified_rule_serialization::specified_serialization!(CssCaret);
crate::serialization::specified_rule_serialization::specified_serialization!(CssInterestDelayValue);
crate::serialization::specified_rule_serialization::specified_serialization!(CssNavigation);
impl crate::serialization::specified_rule_serialization::SpecifiedProvider for CssAccentColor {
    fn append_to_rule_writer(&self, writer: &mut SpecifiedRuleWriter) -> Result<()> {
        match self {
            Self::Auto => writer.keyword("auto"),
            Self::Color(value) => value.append_specified(&mut writer.context, &mut writer.css),
        }
    }
}
impl crate::serialization::specified_rule_serialization::SpecifiedProvider for CssCaret {
    fn append_to_rule_writer(&self, writer: &mut SpecifiedRuleWriter) -> Result<()> {
        writer.node()?;
        let mut started = false;
        let noninitial = self
            .color()
            .is_some_and(|value| !matches!(value, CssCaretColor::Auto))
            || self
                .animation()
                .is_some_and(|value| value != CssCaretAnimation::Auto)
            || self
                .shape()
                .is_some_and(|value| value != CssCaretShape::Auto);
        if let Some(value) = self.color() {
            writer.source_member(0, |writer| {
                if matches!(value, CssCaretColor::Auto) && (noninitial || started) {
                    writer.without_output(|writer| value.append_to_rule_writer(writer))
                } else {
                    writer.space_before_item(&mut started)?;
                    value.append_to_rule_writer(writer)
                }
            })?;
        }
        if let Some(value) = self.animation() {
            writer.source_member(1, |writer| {
                if value == CssCaretAnimation::Auto && (noninitial || started) {
                    writer.without_output(|writer| value.append_to_rule_writer(writer))
                } else {
                    writer.space_before_item(&mut started)?;
                    value.append_to_rule_writer(writer)
                }
            })?;
        }
        if let Some(value) = self.shape() {
            writer.source_member(2, |writer| {
                if value == CssCaretShape::Auto && (noninitial || started) {
                    writer.without_output(|writer| value.append_to_rule_writer(writer))
                } else {
                    writer.space_before_item(&mut started)?;
                    value.append_to_rule_writer(writer)
                }
            })?;
        }
        Ok(())
    }
}
impl crate::serialization::specified_rule_serialization::SpecifiedProvider
    for CssInterestDelayValue
{
    fn append_to_rule_writer(&self, writer: &mut SpecifiedRuleWriter) -> Result<()> {
        match self {
            Self::Normal => writer.keyword("normal"),
            Self::Time(time) => time.append_to_rule_writer(writer),
        }
    }
}
impl crate::serialization::specified_rule_serialization::SpecifiedProvider for CssNavigation {
    fn append_to_rule_writer(&self, writer: &mut SpecifiedRuleWriter) -> Result<()> {
        match self {
            Self::Auto => writer.keyword("auto"),
            Self::Id(value) => {
                writer.node()?;
                writer.node()?;
                writer.append("#")?;
                writer.append_identifier(value.id().as_str())?;
                if let Some(target) = value.target() {
                    writer.append(" ")?;
                    match target {
                        CssNavigationTarget::Current => writer.keyword("current"),
                        CssNavigationTarget::Root => writer.keyword("root"),
                        CssNavigationTarget::Name(value) => {
                            writer.node()?;
                            writer.append_string(value.as_str())
                        }
                        CssNavigationTarget::LegacyName(value) => {
                            writer.node()?;
                            writer.append_string(value.as_str())
                        }
                    }?;
                }
                Ok(())
            }
        }
    }
}

#[cfg(test)]
pub(crate) mod provider_tests {
    use super::*;
    use crate::property::CssKnownPropertyValueRef;
    use crate::serialization::CssSerializationLimits as L;
    use crate::serialization::specified_rule_serialization::SpecifiedProvider as _;
    use crate::serialization::specified_serialization::CssSpecifiedValueSerializationErrorKind as K;
    use crate::values::common::CssSidePair;

    pub(crate) fn shared(
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
    fn ui_providers_share_sibling_work_and_preserve_suppression_on_failure() {
        shared("base-select", 1, 1, |writer| {
            CssAppearance::BaseSelect.append_to_rule_writer(writer)
        });
        let accent = CssAccentColor::Color(Box::new(crate::color::CssColor::transparent()));
        shared("transparent", 1, 1, |writer| {
            accent.append_to_rule_writer(writer)
        });
        for (animation, shape, expected) in [
            (
                CssCaretAnimation::Manual,
                CssCaretShape::Block,
                "manual block",
            ),
            (CssCaretAnimation::Auto, CssCaretShape::Auto, "auto"),
        ] {
            let value =
                CssCaret::try_new(Some(CssCaretColor::Auto), Some(animation), Some(shape)).unwrap();
            shared(expected, 4, 4, |writer| value.append_to_rule_writer(writer));
            assert!(value.color().is_some());
            assert!(value.animation().is_some());
            assert!(value.shape().is_some());
        }
        let value = CssSidePair::<CssInterestDelayValue>::try_new(
            CssInterestDelayValue::Normal,
            Some(CssInterestDelayValue::Normal),
        )
        .unwrap();
        shared("normal", 3, 3, |writer| value.append_to_rule_writer(writer));
        assert!(value.authored_second().is_some());
        let value = CssNavigation::Id(Box::new(
            CssNavigationReference::try_new(
                CssNavigationId::try_new("Next").unwrap(),
                Some(CssNavigationTarget::Root),
            )
            .unwrap(),
        ));
        shared("#Next root", 3, 3, |writer| {
            value.append_to_rule_writer(writer)
        });
        assert_eq!(value.reference().unwrap().id().as_str(), "Next");
    }

    #[test]
    fn retained_equal_signed_times_and_math_consume_both_providers() {
        for (text, expected, input, projection) in [
            ("-1s -1000ms", "-1s", 3, 3),
            ("calc(1s + 2s) calc(1s + 2s)", "calc(3s)", 9, 7),
            ("0.0000001s 0.0000002s", "0s 0s", 3, 3),
        ] {
            let report =
                crate::CssParser::default().parse_block_contents(format!("interest-delay:{text}"));
            assert!(report.is_clean(), "{:?}", report.diagnostics());
            let CssKnownPropertyValueRef::InterestDelay(value) = report.syntax()[0]
                .known()
                .unwrap()
                .property_value()
                .unwrap()
            else {
                panic!("interest-delay")
            };
            let value = value.value();
            shared(expected, input, projection, |writer| {
                value.append_to_rule_writer(writer)
            });
            assert!(value.authored_second().is_some());
        }
    }
}
