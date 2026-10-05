#![forbid(unsafe_code)]
//! Functional evidence for the intentional finite Text models and borrowed views.
//! Oracles: Text4 2026-08-14 property tables/special-form mapping; Values4 §2.2
//! preserves the contiguous quoted trim group. Selected product output visits
//! each emitted keyword once and retains authored absence/alias identity.

use surgeist_css::CssKnownProperty as P;
use surgeist_css::*;

const MODES: [(CssTextWrapMode, &str); 2] = [
    (CssTextWrapMode::Wrap, "wrap"),
    (CssTextWrapMode::NoWrap, "nowrap"),
];
const STYLES: [(CssTextWrapStyle, &str); 5] = [
    (CssTextWrapStyle::Auto, "auto"),
    (CssTextWrapStyle::Balance, "balance"),
    (CssTextWrapStyle::Stable, "stable"),
    (CssTextWrapStyle::Pretty, "pretty"),
    (
        CssTextWrapStyle::AvoidShortLastLine,
        "avoid-short-last-line",
    ),
];
const COLLAPSE: [(CssWhiteSpaceCollapse, &str); 6] = [
    (CssWhiteSpaceCollapse::Collapse, "collapse"),
    (CssWhiteSpaceCollapse::Discard, "discard"),
    (CssWhiteSpaceCollapse::Preserve, "preserve"),
    (CssWhiteSpaceCollapse::PreserveBreaks, "preserve-breaks"),
    (CssWhiteSpaceCollapse::PreserveSpaces, "preserve-spaces"),
    (CssWhiteSpaceCollapse::BreakSpaces, "break-spaces"),
];
const TRIM: [(bool, bool, bool, &str, usize); 8] = [
    (false, false, false, "none", 1),
    (true, false, false, "discard-before", 1),
    (false, true, false, "discard-after", 1),
    (false, false, true, "discard-inner", 1),
    (true, true, false, "discard-before discard-after", 2),
    (true, false, true, "discard-before discard-inner", 2),
    (false, true, true, "discard-after discard-inner", 2),
    (
        true,
        true,
        true,
        "discard-before discard-after discard-inner",
        3,
    ),
];
const SPACE: [P; 3] = [P::WhiteSpaceCollapse, P::TextWrapMode, P::WhiteSpaceTrim];

fn parsed(p: P, value: &str) -> CssDeclaration {
    let css = format!("/*😀*/{}:{value}!important", p.canonical_name());
    let report = parse_style_attribute(&css);
    assert!(report.is_clean(), "{css}: {:?}", report.diagnostics());
    let [source] = report.syntax().as_slice() else {
        panic!("one occurrence")
    };
    source.clone()
}
fn checked(p: P, value: &str, grammar: bool) -> CssDeclaration {
    let components = parse_component_values(value).unwrap();
    let result = if grammar {
        parse_property_value_for_grammar(p.grammar(), components.clone(), CssImportance::Important)
    } else {
        parse_property_value(
            CssPropertyNameRef::Known(p),
            components.clone(),
            CssImportance::Important,
        )
    }
    .unwrap();
    assert_eq!(result.value_components(), &components);
    assert!(result.position().is_none());
    result
}
fn fronts(p: P, value: &str) -> [CssDeclaration; 3] {
    [
        parsed(p, value),
        checked(p, value, false),
        checked(p, value, true),
    ]
}
fn wrap(source: &CssDeclaration) -> &CssTextWrap {
    let CssKnownPropertyValueRef::TextWrap(value) =
        source.known().unwrap().property_value().unwrap()
    else {
        panic!("wrap model")
    };
    value.value()
}
fn space(source: &CssDeclaration) -> &CssWhiteSpace {
    let CssKnownPropertyValueRef::WhiteSpace(value) =
        source.known().unwrap().property_value().unwrap()
    else {
        panic!("space model")
    };
    value.value()
}
fn completed(source: &CssDeclaration) -> CssLonghandContributions {
    let CssExpansion::Contributions(CssContributions::Longhands(values)) =
        expand_declaration(source).unwrap()
    else {
        panic!("ordinary terminals")
    };
    values
}
fn assert_source(
    values: &CssLonghandContributions,
    source: &CssDeclaration,
    replacement: Option<&CssComponentValues>,
) {
    for item in values.items() {
        assert!(item.source().same_occurrence(source));
        assert_eq!(
            item.source().known().unwrap().grammar(),
            source.known().unwrap().grammar()
        );
        assert_eq!(item.source().value_components(), source.value_components());
        assert_eq!(item.source().importance(), source.importance());
        assert_eq!(item.source().position(), source.position());
        assert_eq!(item.replacement_components(), replacement);
    }
}
fn assert_wrap(values: &CssLonghandContributions, mode: CssTextWrapMode, style: CssTextWrapStyle) {
    let [mode_item, style_item] = values.items() else {
        panic!("two wrap terminals")
    };
    assert_eq!(mode_item.property(), P::TextWrapMode);
    assert_eq!(style_item.property(), P::TextWrapStyle);
    let CssLonghandValueRef::TextWrapMode(actual) = mode_item.ordinary_value().unwrap().view()
    else {
        panic!("mode view")
    };
    assert_eq!(*actual, mode);
    let CssLonghandValueRef::TextWrapStyle(actual) = style_item.ordinary_value().unwrap().view()
    else {
        panic!("style view")
    };
    assert_eq!(*actual, style);
}
fn assert_space(
    values: &CssLonghandContributions,
    collapse: CssWhiteSpaceCollapse,
    mode: CssTextWrapMode,
    trim: CssWhiteSpaceTrim,
) {
    assert_eq!(
        values
            .items()
            .iter()
            .map(CssLonghandContribution::property)
            .collect::<Vec<_>>(),
        SPACE
    );
    let CssLonghandValueRef::WhiteSpaceCollapse(actual) =
        values.items()[0].ordinary_value().unwrap().view()
    else {
        panic!("collapse view")
    };
    assert_eq!(*actual, collapse);
    let CssLonghandValueRef::TextWrapMode(actual) =
        values.items()[1].ordinary_value().unwrap().view()
    else {
        panic!("mode view")
    };
    assert_eq!(*actual, mode);
    let CssLonghandValueRef::WhiteSpaceTrim(actual) =
        values.items()[2].ordinary_value().unwrap().view()
    else {
        panic!("trim view")
    };
    assert_eq!(*actual, trim);
}
fn provider(
    expected: &str,
    work: usize,
    emit: impl Fn(
        CssSpecifiedValueSerializationLimits,
    ) -> Result<String, CssSpecifiedValueSerializationError>,
) {
    use CssSpecifiedValueSerializationErrorKind as Kind;
    use CssSpecifiedValueSerializationLimits as Limits;
    let exact = Limits::new(work, work, expected.len());
    assert_eq!(emit(exact).unwrap(), expected);
    for (limits, kind) in [
        (
            Limits::new(work - 1, work, expected.len()),
            Kind::InputNodeLimit,
        ),
        (
            Limits::new(work, work - 1, expected.len()),
            Kind::ProjectionNodeLimit,
        ),
        (Limits::new(work, work, expected.len() - 1), Kind::ByteLimit),
    ] {
        for _ in 0..2 {
            assert_eq!(emit(limits).unwrap_err().kind(), kind);
        }
        assert_eq!(emit(exact).unwrap(), expected);
    }
}

#[test]
fn constructors_reject_only_empty_aggregates_and_preserve_authored_presence() {
    assert!(CssTextWrap::try_new(None, None).is_none());
    assert!(CssWhiteSpace::try_new(None, None, None).is_none());
    let style_only = CssTextWrap::try_new(None, Some(CssTextWrapStyle::Auto)).unwrap();
    let mode_only = CssTextWrap::try_new(Some(CssTextWrapMode::Wrap), None).unwrap();
    assert_eq!(style_only.mode(), None);
    assert_eq!(style_only.style(), Some(CssTextWrapStyle::Auto));
    assert_eq!(mode_only.style(), None);
    assert_ne!(style_only, mode_only);
    assert_eq!(style_only.serialize_specified().unwrap(), "auto");
    assert_eq!(mode_only.serialize_specified().unwrap(), "wrap");
    for (before, after, inner, expected, _) in TRIM {
        let trim = CssWhiteSpaceTrim::new(before, after, inner);
        assert_eq!(trim.discard_before(), before);
        assert_eq!(trim.discard_after(), after);
        assert_eq!(trim.discard_inner(), inner);
        assert_eq!(trim.is_none(), !before && !after && !inner);
        assert_eq!(trim.serialize_specified().unwrap(), expected);
        let value = CssWhiteSpace::try_new(None, None, Some(trim)).unwrap();
        assert_eq!(value.keyword(), None);
        assert_eq!(value.collapse(), None);
        assert_eq!(value.mode(), None);
        assert_eq!(value.trim(), Some(trim));
        assert_eq!(value.serialize_specified().unwrap(), expected);
    }
    assert_eq!(
        CssWhiteSpaceTrim::none(),
        CssWhiteSpaceTrim::new(false, false, false)
    );
}
#[test]
fn every_mode_style_pair_has_programmatic_parsed_checked_and_typed_projection_parity() {
    for (mode, mode_css) in MODES {
        let mode_only = CssTextWrap::try_new(Some(mode), None).unwrap();
        for source in fronts(P::TextWrap, mode_css) {
            assert_eq!(wrap(&source), &mode_only);
            assert_wrap(&completed(&source), mode, CssTextWrapStyle::Auto);
        }
        for (style, style_css) in STYLES {
            let value = CssTextWrap::try_new(Some(mode), Some(style)).unwrap();
            let expected = format!("{mode_css} {style_css}");
            provider(&expected, 2, |limits| {
                value.serialize_specified_with_limits(limits)
            });
            for source in fronts(P::TextWrap, &format!("{style_css} {mode_css}")) {
                assert_eq!(wrap(&source), &value);
                let contributions = completed(&source);
                assert_wrap(&contributions, mode, style);
                assert_source(&contributions, &source, None);
            }
        }
    }
    for (style, css) in STYLES {
        let value = CssTextWrap::try_new(None, Some(style)).unwrap();
        for source in fronts(P::TextWrap, css) {
            assert_eq!(wrap(&source), &value);
            assert_wrap(&completed(&source), CssTextWrapMode::Wrap, style);
        }
    }
}
#[test]
fn special_white_space_forms_retain_alias_identity_and_exact_typed_projection() {
    for (keyword, css, collapse, mode) in [
        (
            CssWhiteSpaceKeyword::Normal,
            "normal",
            CssWhiteSpaceCollapse::Collapse,
            CssTextWrapMode::Wrap,
        ),
        (
            CssWhiteSpaceKeyword::Pre,
            "pre",
            CssWhiteSpaceCollapse::Preserve,
            CssTextWrapMode::NoWrap,
        ),
        (
            CssWhiteSpaceKeyword::PreWrap,
            "pre-wrap",
            CssWhiteSpaceCollapse::Preserve,
            CssTextWrapMode::Wrap,
        ),
        (
            CssWhiteSpaceKeyword::PreLine,
            "pre-line",
            CssWhiteSpaceCollapse::PreserveBreaks,
            CssTextWrapMode::Wrap,
        ),
    ] {
        let value = CssWhiteSpace::from_keyword(keyword);
        assert_eq!(value.keyword(), Some(keyword));
        assert_eq!(
            (value.collapse(), value.mode(), value.trim()),
            (None, None, None)
        );
        provider(css, 1, |limits| {
            value.serialize_specified_with_limits(limits)
        });
        for source in fronts(P::WhiteSpace, css) {
            assert_eq!(space(&source), &value);
            let contributions = completed(&source);
            assert_space(&contributions, collapse, mode, CssWhiteSpaceTrim::none());
            assert_source(&contributions, &source, None);
        }
        let components =
            CssWhiteSpace::try_new(Some(collapse), Some(mode), Some(CssWhiteSpaceTrim::none()))
                .unwrap();
        assert_ne!(components, value);
        assert_eq!(components.keyword(), None);
        assert_ne!(components.serialize_specified().unwrap(), css);
    }
}
#[test]
fn all_nonempty_component_presence_states_preserve_omission_and_defaults_for_every_trim_set() {
    for presence in 1u8..8 {
        for (before, after, inner, trim_css, trim_work) in TRIM {
            let collapse = (presence & 1 != 0).then_some(CssWhiteSpaceCollapse::Preserve);
            let mode = (presence & 2 != 0).then_some(CssTextWrapMode::NoWrap);
            let trim = (presence & 4 != 0).then_some(CssWhiteSpaceTrim::new(before, after, inner));
            let value = CssWhiteSpace::try_new(collapse, mode, trim).unwrap();
            let mut parts = Vec::new();
            if collapse.is_some() {
                parts.push("preserve");
            }
            if mode.is_some() {
                parts.push("nowrap");
            }
            if trim.is_some() {
                parts.push(trim_css);
            }
            let expected = parts.join(" ");
            let work = usize::from(collapse.is_some())
                + usize::from(mode.is_some())
                + if trim.is_some() { trim_work } else { 0 };
            provider(&expected, work, |limits| {
                value.serialize_specified_with_limits(limits)
            });
            for source in fronts(P::WhiteSpace, &expected) {
                assert_eq!(space(&source), &value);
                assert_eq!(
                    (
                        space(&source).collapse(),
                        space(&source).mode(),
                        space(&source).trim()
                    ),
                    (collapse, mode, trim)
                );
                let contributions = completed(&source);
                assert_space(
                    &contributions,
                    collapse.unwrap_or(CssWhiteSpaceCollapse::Collapse),
                    mode.unwrap_or(CssTextWrapMode::Wrap),
                    trim.unwrap_or(CssWhiteSpaceTrim::none()),
                );
                assert_source(&contributions, &source, None);
            }
        }
    }
}
#[test]
fn all_new_constituent_primitive_states_have_exact_keyword_work_and_bytes() {
    for (value, css) in MODES {
        provider(css, 1, |limits| {
            value.serialize_specified_with_limits(limits)
        });
    }
    for (value, css) in STYLES {
        provider(css, 1, |limits| {
            value.serialize_specified_with_limits(limits)
        });
    }
    for (value, css) in COLLAPSE {
        provider(css, 1, |limits| {
            value.serialize_specified_with_limits(limits)
        });
    }
    for (before, after, inner, css, work) in TRIM {
        let value = CssWhiteSpaceTrim::new(before, after, inner);
        provider(css, work, |limits| {
            value.serialize_specified_with_limits(limits)
        });
    }
}
#[test]
fn five_borrowed_terminal_views_expose_exact_direct_values_and_intrinsic_initials() {
    for (value, css) in MODES {
        for source in fronts(P::TextWrapMode, css) {
            let contributions = completed(&source);
            let CssLonghandValueRef::TextWrapMode(actual) =
                contributions.items()[0].ordinary_value().unwrap().view()
            else {
                panic!("direct mode")
            };
            assert_eq!(*actual, value);
            assert_source(&contributions, &source, None);
        }
    }
    for (value, css) in STYLES {
        for source in fronts(P::TextWrapStyle, css) {
            let contributions = completed(&source);
            let CssLonghandValueRef::TextWrapStyle(actual) =
                contributions.items()[0].ordinary_value().unwrap().view()
            else {
                panic!("direct style")
            };
            assert_eq!(*actual, value);
        }
    }
    for (value, css) in COLLAPSE {
        for source in fronts(P::WhiteSpaceCollapse, css) {
            let contributions = completed(&source);
            let CssLonghandValueRef::WhiteSpaceCollapse(actual) =
                contributions.items()[0].ordinary_value().unwrap().view()
            else {
                panic!("direct collapse")
            };
            assert_eq!(*actual, value);
        }
    }
    for (before, after, inner, css, _) in TRIM {
        for source in fronts(P::WhiteSpaceTrim, css) {
            let contributions = completed(&source);
            let CssLonghandValueRef::WhiteSpaceTrim(actual) =
                contributions.items()[0].ordinary_value().unwrap().view()
            else {
                panic!("direct trim")
            };
            assert_eq!(*actual, CssWhiteSpaceTrim::new(before, after, inner));
        }
    }
    for (value, css) in [
        (CssWordBreak::Normal, "normal"),
        (CssWordBreak::BreakAll, "break-all"),
        (CssWordBreak::KeepAll, "keep-all"),
        (CssWordBreak::Manual, "manual"),
        (CssWordBreak::AutoPhrase, "auto-phrase"),
        (CssWordBreak::BreakWord, "break-word"),
    ] {
        for source in fronts(P::WordBreak, css) {
            let contributions = completed(&source);
            assert_eq!(contributions.items().len(), 1);
            let CssLonghandValueRef::WordBreak(actual) =
                contributions.items()[0].ordinary_value().unwrap().view()
            else {
                panic!("direct word-break")
            };
            assert_eq!(*actual, value);
        }
    }
    for p in [
        P::TextWrapMode,
        P::TextWrapStyle,
        P::WhiteSpaceCollapse,
        P::WhiteSpaceTrim,
        P::WordBreak,
    ] {
        let CssPropertyKindRef::Longhand(metadata) = p.metadata().unwrap().kind() else {
            panic!("longhand metadata")
        };
        assert_eq!(metadata.inherited_by_default(), p != P::WhiteSpaceTrim);
        let initial = metadata.initial_value();
        let CssInitialValueRef::Value(initial) = initial.view() else {
            panic!("finite initial")
        };
        match (p, initial.view()) {
            (P::TextWrapMode, CssLonghandValueRef::TextWrapMode(value)) => {
                assert_eq!(*value, CssTextWrapMode::Wrap)
            }
            (P::TextWrapStyle, CssLonghandValueRef::TextWrapStyle(value)) => {
                assert_eq!(*value, CssTextWrapStyle::Auto)
            }
            (P::WhiteSpaceCollapse, CssLonghandValueRef::WhiteSpaceCollapse(value)) => {
                assert_eq!(*value, CssWhiteSpaceCollapse::Collapse)
            }
            (P::WhiteSpaceTrim, CssLonghandValueRef::WhiteSpaceTrim(value)) => {
                assert_eq!(*value, CssWhiteSpaceTrim::none())
            }
            (P::WordBreak, CssLonghandValueRef::WordBreak(value)) => {
                assert_eq!(*value, CssWordBreak::Normal)
            }
            _ => panic!("initial identity and borrowed role"),
        }
    }
}
#[test]
fn original_case_escapes_and_snapshots_survive_typed_projection() {
    for (p, css) in [
        (P::TextWrap, r"b\61 lance NOWRAP"),
        (P::WhiteSpace, "DISCARD-AFTER PRESERVE NOWRAP"),
        (P::WordBreak, "MANUAL"),
        (P::TextWrapMode, "NOWRAP"),
        (P::TextWrapStyle, "PRETTY"),
        (P::WhiteSpaceCollapse, "DISCARD"),
        (P::WhiteSpaceTrim, "DISCARD-INNER"),
    ] {
        let source = parsed(p, css);
        let prefix = format!("/*😀*/{}:", p.canonical_name());
        let original = format!("{prefix}{css}!important");
        let mut snapshot = None;
        for component in source.value_components().items() {
            let CssValueOrigin::Parsed(origin) = component.origin() else {
                panic!("original parsed child")
            };
            assert_eq!(origin.source().as_str(), original);
            assert!(origin.span().start().byte_offset().value() >= prefix.len());
            assert!(origin.span().end().byte_offset().value() <= prefix.len() + css.len());
            if let Some(snapshot) = snapshot {
                assert!(origin.source().same_snapshot(snapshot));
            } else {
                snapshot = Some(origin.source());
            }
        }
        let values = completed(&source);
        assert_source(&values, &source, None);
        for item in values.items() {
            for (actual, expected) in item
                .source()
                .value_components()
                .items()
                .iter()
                .zip(source.value_components().items())
            {
                assert_eq!(actual.origin(), expected.origin());
            }
        }
    }
}
#[test]
fn programmatic_keyword_components_reenter_without_fabricated_parsed_origins() {
    let replacement = CssComponentValues::try_new(
        ["discard-after", "nowrap", "preserve"]
            .into_iter()
            .map(|value| CssComponentValue::try_ident(value).unwrap())
            .collect(),
    )
    .unwrap();
    assert!(
        replacement
            .items()
            .iter()
            .all(|value| matches!(value.origin(), CssValueOrigin::Programmatic))
    );
    for grammar in [false, true] {
        let declaration = if grammar {
            parse_property_value_for_grammar(
                P::WhiteSpace.grammar(),
                replacement.clone(),
                CssImportance::Important,
            )
        } else {
            parse_property_value(
                CssPropertyNameRef::Known(P::WhiteSpace),
                replacement.clone(),
                CssImportance::Important,
            )
        }
        .unwrap();
        assert!(declaration.position().is_none());
        assert_eq!(
            space(&declaration),
            &CssWhiteSpace::try_new(
                Some(CssWhiteSpaceCollapse::Preserve),
                Some(CssTextWrapMode::NoWrap),
                Some(CssWhiteSpaceTrim::new(false, true, false))
            )
            .unwrap()
        );
        assert_eq!(declaration.value_components(), &replacement);
    }
    let source = parsed(P::WhiteSpace, "var(--space)");
    let CssExpansion::Pending(handle) = expand_declaration(&source).unwrap() else {
        panic!("pending")
    };
    let CssContributions::Longhands(values) = handle.reenter(replacement.clone()).unwrap() else {
        panic!("programmatic replacement")
    };
    assert_space(
        &values,
        CssWhiteSpaceCollapse::Preserve,
        CssTextWrapMode::NoWrap,
        CssWhiteSpaceTrim::new(false, true, false),
    );
    assert_source(&values, &source, Some(&replacement));
    assert!(
        values
            .items()
            .iter()
            .flat_map(|item| item.replacement_components().unwrap().items())
            .all(|value| matches!(value.origin(), CssValueOrigin::Programmatic))
    );
}
#[test]
fn contiguous_trim_groups_permute_but_cannot_interleave_other_roles() {
    for css in [
        "discard-inner discard-after discard-before nowrap preserve",
        "nowrap discard-before discard-inner discard-after preserve",
        "preserve nowrap discard-after discard-before discard-inner",
    ] {
        for source in fronts(P::WhiteSpace, css) {
            assert_space(
                &completed(&source),
                CssWhiteSpaceCollapse::Preserve,
                CssTextWrapMode::NoWrap,
                CssWhiteSpaceTrim::new(true, true, true),
            );
            assert_eq!(
                space(&source).serialize_specified().unwrap(),
                "preserve nowrap discard-before discard-after discard-inner"
            );
        }
    }
    for css in [
        "discard-before nowrap discard-after",
        "discard-inner preserve discard-after",
        "discard-before preserve nowrap discard-inner",
        "none nowrap discard-before",
    ] {
        let report = parse_style_attribute(&format!("white-space:{css}"));
        assert!(report.syntax().is_empty());
        assert!(!report.is_clean());
        for result in [
            parse_property_value(
                CssPropertyNameRef::Known(P::WhiteSpace),
                parse_component_values(css).unwrap(),
                CssImportance::Normal,
            ),
            parse_property_value_for_grammar(
                P::WhiteSpace.grammar(),
                parse_component_values(css).unwrap(),
                CssImportance::Normal,
            ),
        ] {
            assert!(result.is_err());
        }
    }
}
#[test]
fn new_constituent_browser_recovery_retains_eof_and_strict_checked_rejection() {
    for (p, css) in [
        (P::TextWrapMode, "nowrap"),
        (P::TextWrapStyle, "auto"),
        (P::WhiteSpaceCollapse, "discard"),
        (P::WhiteSpaceTrim, "discard-before discard-after"),
    ] {
        let text = format!("{}:{css}/*", p.canonical_name());
        let report = parse_style_attribute(&text);
        assert!(!report.is_clean());
        let [source] = report.syntax().as_slice() else {
            panic!("retained browser occurrence")
        };
        let before = source.clone();
        let serialized = source.value_components().serialize().unwrap();
        let origin = (0..serialized.as_css().len())
            .find_map(|offset| match serialized.origin_at(offset) {
                Some(CssSerializedOrigin::Token(
                    value @ CssValueOrigin::ImplicitClosure { .. },
                )) => Some(value.clone()),
                _ => None,
            })
            .unwrap();
        let CssValueOrigin::ImplicitClosure { at, .. } = &origin else {
            unreachable!()
        };
        assert_eq!(at.source().as_str(), text);
        assert_eq!(at.span().start().byte_offset().value(), text.len());
        assert_eq!(at.span().start(), at.span().end());
        let values = completed(source);
        assert_source(&values, source, None);
        for result in [
            parse_property_value(
                CssPropertyNameRef::Known(p),
                source.value_components().clone(),
                CssImportance::Normal,
            ),
            parse_property_value_for_grammar(
                p.grammar(),
                source.value_components().clone(),
                CssImportance::Normal,
            ),
        ] {
            let error = result.unwrap_err();
            assert!(matches!(
                error.kind(),
                CssPropertyValueErrorKind::Grammar(ErrorKind::UnexpectedEnd(_))
            ));
            assert_eq!(
                error.origin(),
                &CssSerializedOrigin::End(Some(origin.clone()))
            );
        }
        assert_eq!(source, &before);

        let report = parse_sheet(&format!(".a{{{}:{css}/*", p.canonical_name()));
        assert!(!report.is_clean());
        let before = report.clone();
        let exact = CssNormalizationLimits::try_new(0, 1, 1, 1).unwrap();
        let normalized = normalize_report_with_limits(&report, exact).unwrap();
        assert_eq!(normalized.diagnostics(), report.diagnostics());
        let items: Vec<_> = normalized
            .syntax()
            .items()
            .iter()
            .filter_map(|item| match item {
                CssNormalizedItem::Declaration(value) => Some(value),
                _ => None,
            })
            .collect();
        let [item] = items.as_slice() else {
            panic!("one recovered normalized occurrence")
        };
        let CssExpansion::Contributions(CssContributions::Longhands(values)) = item.expansion()
        else {
            panic!("one recovered terminal")
        };
        assert_eq!(values.items().len(), 1);
        assert_eq!(values.items()[0].property(), p);
        assert_source(values, item.source(), None);
        for _ in 0..2 {
            let error = normalize_report_with_limits(
                &report,
                CssNormalizationLimits::try_new(0, 1, 1, 0).unwrap(),
            )
            .unwrap_err();
            assert_eq!(
                error.kind(),
                &CssNormalizationErrorKind::LimitExceeded {
                    resource: CssNormalizationResource::Contributions,
                    limit: 0
                }
            );
            assert!(error.declaration().unwrap().same_occurrence(item.source()));
        }
        assert_eq!(report, before);
        assert!(normalize_report_with_limits(&report, exact).is_ok());
    }
}
#[test]
fn typed_pending_retries_keep_original_occurrence_and_replacement_snapshots() {
    for (p, css, bad) in [
        (P::TextWrap, "nowrap balance", "wrap nowrap"),
        (
            P::WhiteSpace,
            "discard-after preserve nowrap",
            "discard-before nowrap discard-after",
        ),
    ] {
        let source = parsed(p, "var(--text)");
        let before = source.clone();
        let CssExpansion::Pending(handle) = expand_declaration(&source).unwrap() else {
            panic!("pending")
        };
        let replacement = parse_component_values(css).unwrap();
        for invalid in [bad.to_string(), format!("{css}/*"), "initial/*".into()] {
            for _ in 0..2 {
                assert!(matches!(
                    handle
                        .reenter(parse_component_values(&invalid).unwrap())
                        .unwrap_err()
                        .kind(),
                    CssExpansionErrorKind::InvalidReplacement(_)
                ));
            }
            let CssContributions::Longhands(values) = handle.reenter(replacement.clone()).unwrap()
            else {
                panic!("retry")
            };
            if p == P::TextWrap {
                assert_wrap(&values, CssTextWrapMode::NoWrap, CssTextWrapStyle::Balance);
            } else {
                assert_space(
                    &values,
                    CssWhiteSpaceCollapse::Preserve,
                    CssTextWrapMode::NoWrap,
                    CssWhiteSpaceTrim::new(false, true, false),
                );
            }
        }
        assert_eq!(
            handle
                .reenter(parse_component_values("env(again)").unwrap())
                .unwrap_err()
                .kind(),
            &CssExpansionErrorKind::ResidualSubstitution
        );
        for _ in 0..2 {
            let CssContributions::Longhands(values) = handle.reenter(replacement.clone()).unwrap()
            else {
                panic!("final retry")
            };
            assert_source(&values, &source, Some(&replacement));
            for item in values.items() {
                for (actual, original) in item
                    .replacement_components()
                    .unwrap()
                    .items()
                    .iter()
                    .zip(replacement.items())
                {
                    let (CssValueOrigin::Parsed(actual), CssValueOrigin::Parsed(original)) =
                        (actual.origin(), original.origin())
                    else {
                        panic!("retained replacement origin")
                    };
                    assert!(actual.source().same_snapshot(original.source()));
                    assert_eq!(actual.span(), original.span());
                }
            }
        }
        assert_eq!(source, before);
    }
}
#[test]
fn seven_property_normalization_shares_context_and_atomic_eleven_member_budget() {
    let report = parse_sheet(
        "@media screen{.a{text-wrap-mode:nowrap!important;text-wrap-style:auto;white-space-collapse:discard;white-space-trim:none;text-wrap:stable;white-space:pre-line;word-break:auto-phrase;white-space:var(--space)}}",
    );
    assert!(report.is_clean());
    let before = report.clone();
    let exact = CssNormalizationLimits::try_new(1, 2, 8, 11).unwrap();
    let normalized = normalize_sheet_with_limits(report.syntax(), exact).unwrap();
    let items: Vec<_> = normalized
        .items()
        .iter()
        .filter_map(|item| match item {
            CssNormalizedItem::Declaration(value) => Some(value),
            _ => None,
        })
        .collect();
    assert_eq!(items.len(), 8);
    for (index, (item, p)) in items
        .iter()
        .zip([
            P::TextWrapMode,
            P::TextWrapStyle,
            P::WhiteSpaceCollapse,
            P::WhiteSpaceTrim,
            P::TextWrap,
            P::WhiteSpace,
            P::WordBreak,
            P::WhiteSpace,
        ])
        .enumerate()
    {
        assert_eq!(item.order(), index);
        assert_eq!(item.source().known().unwrap().property(), p);
        assert!(item.rule_context().same_context(items[0].rule_context()));
        assert!(
            item.selector_context()
                .same_context(items[0].selector_context())
        );
        assert_eq!(
            item.source().importance(),
            if index == 0 {
                CssImportance::Important
            } else {
                CssImportance::Normal
            }
        );
        if index == 7 {
            let CssExpansion::Pending(handle) = item.expansion() else {
                panic!("one unresolved occurrence")
            };
            assert!(handle.source().same_occurrence(item.source()));
        } else {
            let CssExpansion::Contributions(CssContributions::Longhands(values)) = item.expansion()
            else {
                panic!("typed ordinary terminals")
            };
            assert_source(values, item.source(), None);
            if index == 4 {
                assert_wrap(values, CssTextWrapMode::Wrap, CssTextWrapStyle::Stable);
            }
            if index == 5 {
                assert_space(
                    values,
                    CssWhiteSpaceCollapse::PreserveBreaks,
                    CssTextWrapMode::Wrap,
                    CssWhiteSpaceTrim::none(),
                );
            }
        }
    }
    for (limits, resource, limit) in [
        (
            CssNormalizationLimits::try_new(1, 2, 8, 10).unwrap(),
            CssNormalizationResource::Contributions,
            10,
        ),
        (
            CssNormalizationLimits::try_new(1, 2, 7, 11).unwrap(),
            CssNormalizationResource::Declarations,
            7,
        ),
    ] {
        for _ in 0..2 {
            let error = normalize_sheet_with_limits(report.syntax(), limits).unwrap_err();
            assert_eq!(
                error.kind(),
                &CssNormalizationErrorKind::LimitExceeded { resource, limit }
            );
            assert_eq!(error.declaration_order(), Some(7));
            assert!(
                error
                    .declaration()
                    .unwrap()
                    .same_occurrence(items[7].source())
            );
        }
    }
    assert_eq!(report, before);
    assert!(normalize_sheet_with_limits(report.syntax(), exact).is_ok());
}
#[test]
fn new_trim_declarations_and_sibling_rules_share_exact_provider_budgets() {
    use CssSpecifiedValueSerializationErrorKind as Kind;
    use CssSpecifiedValueSerializationLimits as Limits;
    let source = parsed(P::WhiteSpaceTrim, "discard-after discard-before");
    let before = source.clone();
    let expected = "white-space-trim: discard-before discard-after !important;";
    assert_eq!(
        source
            .to_specified_css_with_limits(Limits::new(4, 4, expected.len()))
            .unwrap(),
        expected
    );
    for (limits, kind) in [
        (Limits::new(3, 4, expected.len()), Kind::InputNodeLimit),
        (Limits::new(4, 3, expected.len()), Kind::ProjectionNodeLimit),
        (Limits::new(4, 4, expected.len() - 1), Kind::ByteLimit),
    ] {
        for _ in 0..2 {
            assert_eq!(
                source
                    .to_specified_css_with_limits(limits)
                    .unwrap_err()
                    .kind(),
                kind
            );
        }
        assert_eq!(source, before);
    }
    assert_eq!(source.to_specified_css().unwrap(), expected);
    let report = parse_sheet(
        ".a{white-space-trim:discard-after discard-before}.b{text-wrap-style:avoid-short-last-line}",
    );
    assert!(report.is_clean());
    let before = report.clone();
    let expected = ".a { white-space-trim: discard-before discard-after; }\n.b { text-wrap-style: avoid-short-last-line; }";
    let exact = Limits::new(usize::MAX, usize::MAX, expected.len());
    let short = Limits::new(usize::MAX, usize::MAX, expected.len() - 1);
    assert_eq!(
        report.syntax().to_specified_css_with_limits(exact).unwrap(),
        expected
    );
    assert!(
        report
            .syntax()
            .rules()
            .iter()
            .all(|rule| rule.to_specified_css_with_limits(short).is_ok())
    );
    for _ in 0..2 {
        let error = report
            .syntax()
            .to_specified_css_with_limits(short)
            .unwrap_err();
        assert_eq!(
            error.kind(),
            CssSpecifiedRuleSerializationErrorKind::Resource(Kind::ByteLimit)
        );
        assert_eq!(error.rule_index(), Some(1));
        assert_eq!(report, before);
    }
    assert_eq!(
        report.syntax().to_specified_css_with_limits(exact).unwrap(),
        expected
    );
}
