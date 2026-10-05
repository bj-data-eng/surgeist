//! Canonical specified emission for the currently represented keyword domains.
//! This adds no grammar alternatives or computed-value behavior.
use crate::specified_rule_serialization::SpecifiedRuleWriter;
use crate::{
    CssBlendMode, CssImageRendering, CssIsolation, CssObjectFit, CssPointerEvents, CssResize,
    CssSpecifiedValueSerializationError, CssSpecifiedValueSerializationLimits,
    CssTextDecorationStyle, CssTextTransform, CssTextWrap, CssTransformBox, CssUserSelect,
    CssWhiteSpace, CssWordBreak,
};

type Result<T> = std::result::Result<T, CssSpecifiedValueSerializationError>;

macro_rules! keyword_provider {
    ($ty:ty, $value:ident => $text:expr) => {
        impl $ty {
            /// Emits the canonical spelling of the represented specified keyword.
            pub fn serialize_specified(&self) -> Result<String> {
                self.serialize_specified_with_limits(CssSpecifiedValueSerializationLimits::default())
            }

            /// Emits atomically, charging one input node, one projection node,
            /// and the final UTF-8 bytes. No contextual value is resolved.
            pub fn serialize_specified_with_limits(
                &self,
                limits: CssSpecifiedValueSerializationLimits,
            ) -> Result<String> {
                let mut writer = SpecifiedRuleWriter::new(limits);
                self.append_to_rule_writer(&mut writer)?;
                Ok(writer.css)
            }

            pub(crate) fn append_to_rule_writer(&self, writer: &mut SpecifiedRuleWriter) -> Result<()> {
                let $value = self;
                writer.keyword($text)
            }
        }
    };
}
macro_rules! enum_keywords {
    ($ty:ty, $($variant:ident => $text:literal),+ $(,)?) => {
        keyword_provider!($ty, value => match value { $(Self::$variant => $text),+ });
    };
}

enum_keywords!(CssTextWrap,
    Wrap => "wrap", NoWrap => "nowrap", Balance => "balance", Pretty => "pretty", Stable => "stable",
);
enum_keywords!(CssWhiteSpace,
    Normal => "normal", NoWrap => "nowrap", Pre => "pre", PreWrap => "pre-wrap",
    PreLine => "pre-line", BreakSpaces => "break-spaces",
);
enum_keywords!(CssWordBreak,
    Normal => "normal", BreakAll => "break-all", KeepAll => "keep-all", BreakWord => "break-word",
);
enum_keywords!(CssTextTransform,
    None => "none", Capitalize => "capitalize", Uppercase => "uppercase", Lowercase => "lowercase",
);
enum_keywords!(CssTextDecorationStyle,
    Solid => "solid", Double => "double", Dotted => "dotted", Dashed => "dashed", Wavy => "wavy",
);
enum_keywords!(CssImageRendering,
    Auto => "auto", CrispEdges => "crisp-edges", Pixelated => "pixelated",
);
enum_keywords!(CssObjectFit,
    Fill => "fill", Contain => "contain", Cover => "cover", None => "none", ScaleDown => "scale-down",
);
enum_keywords!(CssPointerEvents, Auto => "auto", None => "none");
enum_keywords!(CssUserSelect,
    Auto => "auto", Text => "text", None => "none", All => "all", Contain => "contain",
);
enum_keywords!(CssResize,
    None => "none", Both => "both", Horizontal => "horizontal", Vertical => "vertical",
);
enum_keywords!(CssIsolation, Auto => "auto", Isolate => "isolate");
keyword_provider!(CssTransformBox, value => value.edge().as_css_str());
keyword_provider!(CssBlendMode, value => value.as_css_str());

#[cfg(test)]
mod tests {
    use super::*;
    use crate::CssBoxEdgeKeyword;
    use crate::CssSpecifiedValueSerializationErrorKind as Kind;
    use CssSpecifiedValueSerializationLimits as Limits;

    fn compose(writer: &mut SpecifiedRuleWriter) -> Result<()> {
        writer.append("prefix ")?;
        CssTextWrap::Balance.append_to_rule_writer(writer)?;
        writer.append(" ")?;
        CssBlendMode::ColorBurn.append_to_rule_writer(writer)
    }

    #[test]
    fn siblings_share_consumed_nodes_and_final_bytes() {
        let expected = "prefix balance color-burn";
        let mut writer = SpecifiedRuleWriter::new(Limits::new(2, 2, expected.len()));
        compose(&mut writer).unwrap();
        assert_eq!(writer.css, expected);
        for (limits, kind) in [
            (Limits::new(1, 2, expected.len()), Kind::InputNodeLimit),
            (Limits::new(2, 1, expected.len()), Kind::ProjectionNodeLimit),
            (Limits::new(2, 2, expected.len() - 1), Kind::ByteLimit),
        ] {
            let mut writer = SpecifiedRuleWriter::new(limits);
            assert_eq!(compose(&mut writer).unwrap_err().kind(), kind);
        }
    }

    #[test]
    fn every_provider_suppresses_bytes_but_consumes_work_and_restores_modes() {
        type Emit = fn(&mut SpecifiedRuleWriter) -> Result<()>;
        let emitters: [Emit; 13] = [
            |w| CssTextWrap::Pretty.append_to_rule_writer(w),
            |w| CssWhiteSpace::BreakSpaces.append_to_rule_writer(w),
            |w| CssWordBreak::KeepAll.append_to_rule_writer(w),
            |w| CssTextTransform::Capitalize.append_to_rule_writer(w),
            |w| CssTextDecorationStyle::Wavy.append_to_rule_writer(w),
            |w| CssImageRendering::CrispEdges.append_to_rule_writer(w),
            |w| CssObjectFit::ScaleDown.append_to_rule_writer(w),
            |w| CssPointerEvents::None.append_to_rule_writer(w),
            |w| CssUserSelect::Contain.append_to_rule_writer(w),
            |w| CssResize::Horizontal.append_to_rule_writer(w),
            |w| CssIsolation::Isolate.append_to_rule_writer(w),
            |w| {
                CssTransformBox::try_new(CssBoxEdgeKeyword::StrokeBox)
                    .unwrap()
                    .append_to_rule_writer(w)
            },
            |w| CssBlendMode::Luminosity.append_to_rule_writer(w),
        ];
        for emit in emitters {
            let mut writer = SpecifiedRuleWriter::new(Limits::new(1, 1, 0));
            writer.without_output(emit).unwrap();
            assert!(writer.css.is_empty());
            assert!(!writer.context.output_suppressed());
            assert_eq!(emit(&mut writer).unwrap_err().kind(), Kind::InputNodeLimit);

            for (limits, kind) in [
                (Limits::new(0, 1, 1), Kind::InputNodeLimit),
                (Limits::new(1, 0, 1), Kind::ProjectionNodeLimit),
            ] {
                let mut writer = SpecifiedRuleWriter::new(limits);
                writer
                    .without_output(|writer| {
                        assert_eq!(writer.without_output(emit).unwrap_err().kind(), kind);
                        assert!(writer.context.output_suppressed());
                        writer.append("discarded")
                    })
                    .unwrap();
                assert!(!writer.context.output_suppressed());
                assert!(writer.css.is_empty());
                writer.append("x").unwrap();
                assert_eq!(writer.css, "x");
            }
        }
    }
}
