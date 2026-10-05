//! Canonical specified emission for the currently represented keyword domains.
//! This adds no grammar alternatives or computed-value behavior.
use crate::specified_rule_serialization::SpecifiedRuleWriter;
use crate::{
    CssBlendMode, CssImageRendering, CssIsolation, CssLineBreak, CssObjectFit, CssPointerEvents,
    CssResize, CssSpecifiedValueSerializationError, CssSpecifiedValueSerializationLimits,
    CssTextDecorationStyle, CssTextTransform, CssTextTransformCase, CssTextWrap, CssTextWrapMode,
    CssTextWrapStyle, CssTransformBox, CssUserSelect, CssWhiteSpace, CssWhiteSpaceCollapse,
    CssWhiteSpaceKeyword, CssWhiteSpaceTrim, CssWordBreak, CssWordSpaceTransform, CssWrapBoundary,
    CssWrapInside,
};

type Result<T> = std::result::Result<T, CssSpecifiedValueSerializationError>;

macro_rules! specified_provider {
    ($ty:ty, $value:ident, $writer:ident => $body:expr) => {
        impl $ty {
            /// Emits canonical authored keywords without resolving contextual values.
            pub fn serialize_specified(&self) -> Result<String> {
                self.serialize_specified_with_limits(CssSpecifiedValueSerializationLimits::default())
            }

            /// Emits atomically, charging each emitted keyword once for input and
            /// projection work, plus the final UTF-8 bytes. Carriers add no work.
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
                let $writer = writer;
                $body
            }
        }
    };
}
macro_rules! keyword_provider {
    ($ty:ty, $value:ident => $text:expr) => {
        specified_provider!($ty, $value, writer => writer.keyword($text));
    };
}
macro_rules! enum_keywords {
    ($ty:ty, $($variant:ident => $text:literal),+ $(,)?) => {
        keyword_provider!($ty, value => match value { $(Self::$variant => $text),+ });
    };
}

enum_keywords!(CssTextWrapMode, Wrap => "wrap", NoWrap => "nowrap");
enum_keywords!(CssTextWrapStyle,
    Auto => "auto", Balance => "balance", Stable => "stable", Pretty => "pretty",
    AvoidShortLastLine => "avoid-short-last-line",
);
enum_keywords!(CssWhiteSpaceCollapse,
    Collapse => "collapse", Discard => "discard", Preserve => "preserve",
    PreserveBreaks => "preserve-breaks", PreserveSpaces => "preserve-spaces", BreakSpaces => "break-spaces",
);
specified_provider!(CssWhiteSpaceTrim, value, writer => {
    if value.is_none() {
        return writer.keyword("none");
    }
    let mut separated = false;
    for (present, text) in [
        (value.discard_before(), "discard-before"),
        (value.discard_after(), "discard-after"),
        (value.discard_inner(), "discard-inner"),
    ] {
        if present {
            if separated { writer.append(" ")?; }
            writer.keyword(text)?;
            separated = true;
        }
    }
    Ok(())
});
specified_provider!(CssTextWrap, value, writer => {
    if let Some(mode) = value.mode() { mode.append_to_rule_writer(writer)?; }
    if let Some(style) = value.style() {
        if value.mode().is_some() { writer.append(" ")?; }
        style.append_to_rule_writer(writer)?;
    }
    Ok(())
});
specified_provider!(CssWhiteSpace, value, writer => {
    if let Some(keyword) = value.keyword() {
        return writer.keyword(match keyword {
            CssWhiteSpaceKeyword::Normal => "normal",
            CssWhiteSpaceKeyword::Pre => "pre",
            CssWhiteSpaceKeyword::PreWrap => "pre-wrap",
            CssWhiteSpaceKeyword::PreLine => "pre-line",
        });
    }
    if let Some(collapse) = value.collapse() { collapse.append_to_rule_writer(writer)?; }
    if let Some(mode) = value.mode() {
        if value.collapse().is_some() { writer.append(" ")?; }
        mode.append_to_rule_writer(writer)?;
    }
    if let Some(trim) = value.trim() {
        if value.collapse().is_some() || value.mode().is_some() { writer.append(" ")?; }
        trim.append_to_rule_writer(writer)?;
    }
    Ok(())
});
enum_keywords!(CssWordBreak,
    Normal => "normal", BreakAll => "break-all", KeepAll => "keep-all", Manual => "manual",
    AutoPhrase => "auto-phrase", BreakWord => "break-word",
);

enum_keywords!(CssWrapInside, Auto => "auto", Avoid => "avoid");
enum_keywords!(CssWrapBoundary,
    Auto => "auto", Avoid => "avoid", AvoidLine => "avoid-line",
    AvoidFlex => "avoid-flex", Line => "line", Flex => "flex",
);
enum_keywords!(CssLineBreak,
    Auto => "auto", Loose => "loose", Normal => "normal", Strict => "strict", Anywhere => "anywhere",
);
specified_provider!(CssTextTransform, value, writer => {
    let set = match value {
        Self::None => return writer.keyword("none"),
        Self::MathAuto => return writer.keyword("math-auto"),
        Self::Transforms(set) => set,
    };
    let mut separated = false;
    if let Some(case) = set.case() {
        writer.keyword(match case {
            CssTextTransformCase::Capitalize => "capitalize",
            CssTextTransformCase::Uppercase => "uppercase",
            CssTextTransformCase::Lowercase => "lowercase",
        })?;
        separated = true;
    }
    for (present, text) in [(set.full_width(), "full-width"), (set.full_size_kana(), "full-size-kana")] {
        if present {
            if separated { writer.append(" ")?; }
            writer.keyword(text)?;
            separated = true;
        }
    }
    Ok(())
});
specified_provider!(CssWordSpaceTransform, value, writer => {
    let (base, auto_phrase) = match value {
        Self::None => return writer.keyword("none"),
        Self::Space { auto_phrase } => ("space", *auto_phrase),
        Self::IdeographicSpace { auto_phrase } => ("ideographic-space", *auto_phrase),
    };
    writer.keyword(base)?;
    if auto_phrase { writer.append(" ")?; writer.keyword("auto-phrase")?; }
    Ok(())
});

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
        CssTextWrap::try_new(None, Some(CssTextWrapStyle::Balance))
            .unwrap()
            .append_to_rule_writer(writer)?;
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
            |w| {
                CssTextWrap::try_new(None, Some(CssTextWrapStyle::Pretty))
                    .unwrap()
                    .append_to_rule_writer(w)
            },
            |w| {
                CssWhiteSpace::try_new(Some(CssWhiteSpaceCollapse::BreakSpaces), None, None)
                    .unwrap()
                    .append_to_rule_writer(w)
            },
            |w| CssWordBreak::KeepAll.append_to_rule_writer(w),
            |w| {
                CssTextTransform::Transforms(
                    crate::CssTextTransformSet::try_new(
                        Some(CssTextTransformCase::Capitalize),
                        false,
                        false,
                    )
                    .unwrap(),
                )
                .append_to_rule_writer(w)
            },
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

    #[test]
    fn text_constituents_and_transparent_carriers_charge_all_suppressed_keywords() {
        type Emit = fn(&mut SpecifiedRuleWriter) -> Result<()>;
        let cases: [(Emit, usize, &str); 7] = [
            (
                |w| CssTextWrapMode::NoWrap.append_to_rule_writer(w),
                1,
                "nowrap",
            ),
            (
                |w| CssTextWrapStyle::AvoidShortLastLine.append_to_rule_writer(w),
                1,
                "avoid-short-last-line",
            ),
            (
                |w| CssWhiteSpaceCollapse::Discard.append_to_rule_writer(w),
                1,
                "discard",
            ),
            (
                |w| CssWhiteSpaceTrim::none().append_to_rule_writer(w),
                1,
                "none",
            ),
            (
                |w| CssWhiteSpaceTrim::new(true, true, true).append_to_rule_writer(w),
                3,
                "discard-before discard-after discard-inner",
            ),
            (
                |w| {
                    CssTextWrap::try_new(
                        Some(CssTextWrapMode::NoWrap),
                        Some(CssTextWrapStyle::Balance),
                    )
                    .unwrap()
                    .append_to_rule_writer(w)
                },
                2,
                "nowrap balance",
            ),
            (
                |w| {
                    CssWhiteSpace::try_new(
                        Some(CssWhiteSpaceCollapse::Preserve),
                        Some(CssTextWrapMode::NoWrap),
                        Some(CssWhiteSpaceTrim::new(true, true, true)),
                    )
                    .unwrap()
                    .append_to_rule_writer(w)
                },
                5,
                "preserve nowrap discard-before discard-after discard-inner",
            ),
        ];
        for (emit, work, expected) in cases {
            let mut writer = SpecifiedRuleWriter::new(Limits::new(work, work, expected.len()));
            emit(&mut writer).unwrap();
            assert_eq!(writer.css, expected);
            let mut writer = SpecifiedRuleWriter::new(Limits::new(work, work, 0));
            writer
                .without_output(|writer| writer.without_output(emit))
                .unwrap();
            assert!(writer.css.is_empty());
            assert!(!writer.context.output_suppressed());
            assert_eq!(emit(&mut writer).unwrap_err().kind(), Kind::InputNodeLimit);
            for (limits, kind) in [
                (Limits::new(work - 1, work, 1), Kind::InputNodeLimit),
                (Limits::new(work, work - 1, 1), Kind::ProjectionNodeLimit),
            ] {
                let mut writer = SpecifiedRuleWriter::new(limits);
                writer
                    .without_output(|writer| {
                        assert_eq!(writer.without_output(emit).unwrap_err().kind(), kind);
                        assert!(writer.context.output_suppressed());
                        writer.append("hidden")
                    })
                    .unwrap();
                assert!(writer.css.is_empty());
                assert!(!writer.context.output_suppressed());
                writer.append("x").unwrap();
                assert_eq!(writer.css, "x");
            }
        }
    }

    #[test]
    fn typography_keyword_providers_share_suppressed_work_and_consumed_siblings() {
        type Emit = fn(&mut SpecifiedRuleWriter) -> Result<()>;
        let cases: [(Emit, usize, &str); 9] = [
            (
                |w| CssTextTransform::None.append_to_rule_writer(w),
                1,
                "none",
            ),
            (
                |w| CssTextTransform::MathAuto.append_to_rule_writer(w),
                1,
                "math-auto",
            ),
            (
                |w| {
                    CssTextTransform::Transforms(
                        crate::CssTextTransformSet::try_new(None, true, true).unwrap(),
                    )
                    .append_to_rule_writer(w)
                },
                2,
                "full-width full-size-kana",
            ),
            (
                |w| {
                    CssTextTransform::Transforms(
                        crate::CssTextTransformSet::try_new(
                            Some(CssTextTransformCase::Uppercase),
                            true,
                            true,
                        )
                        .unwrap(),
                    )
                    .append_to_rule_writer(w)
                },
                3,
                "uppercase full-width full-size-kana",
            ),
            (
                |w| CssWrapInside::Avoid.append_to_rule_writer(w),
                1,
                "avoid",
            ),
            (
                |w| CssWrapBoundary::AvoidFlex.append_to_rule_writer(w),
                1,
                "avoid-flex",
            ),
            (
                |w| CssLineBreak::Anywhere.append_to_rule_writer(w),
                1,
                "anywhere",
            ),
            (
                |w| CssWordSpaceTransform::None.append_to_rule_writer(w),
                1,
                "none",
            ),
            (
                |w| {
                    CssWordSpaceTransform::IdeographicSpace { auto_phrase: true }
                        .append_to_rule_writer(w)
                },
                2,
                "ideographic-space auto-phrase",
            ),
        ];
        for (emit, work, expected) in cases {
            let css = format!("prefix {expected};{expected}");
            let mut writer = SpecifiedRuleWriter::new(Limits::new(work * 2, work * 2, css.len()));
            writer.append("prefix ").unwrap();
            emit(&mut writer).unwrap();
            writer.append(";").unwrap();
            emit(&mut writer).unwrap();
            assert_eq!(writer.css, css);
            for (limits, kind) in [
                (
                    Limits::new(work * 2 - 1, work * 2, css.len()),
                    Kind::InputNodeLimit,
                ),
                (
                    Limits::new(work * 2, work * 2 - 1, css.len()),
                    Kind::ProjectionNodeLimit,
                ),
                (
                    Limits::new(work * 2, work * 2, css.len() - 1),
                    Kind::ByteLimit,
                ),
            ] {
                let mut writer = SpecifiedRuleWriter::new(limits);
                writer.append("prefix ").unwrap();
                emit(&mut writer).unwrap();
                writer.append(";").unwrap();
                assert_eq!(emit(&mut writer).unwrap_err().kind(), kind);
            }
            let mut writer = SpecifiedRuleWriter::new(Limits::new(work, work, 0));
            writer.without_output(|w| w.without_output(emit)).unwrap();
            assert!(writer.css.is_empty());
            assert!(!writer.context.output_suppressed());
            assert_eq!(emit(&mut writer).unwrap_err().kind(), Kind::InputNodeLimit);
            for (limits, kind) in [
                (Limits::new(work - 1, work, 1), Kind::InputNodeLimit),
                (Limits::new(work, work - 1, 1), Kind::ProjectionNodeLimit),
            ] {
                let mut writer = SpecifiedRuleWriter::new(limits);
                writer
                    .without_output(|w| {
                        assert_eq!(w.without_output(emit).unwrap_err().kind(), kind);
                        assert!(w.context.output_suppressed());
                        w.append("hidden")
                    })
                    .unwrap();
                assert!(writer.css.is_empty());
                assert!(!writer.context.output_suppressed());
                writer.append("x").unwrap();
                assert_eq!(writer.css, "x");
            }
        }
    }
}
