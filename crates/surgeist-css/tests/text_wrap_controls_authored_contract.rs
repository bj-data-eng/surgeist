#![forbid(unsafe_code)]
//! Existing-public-API authored wrapping control contract.
//! Text4 WD20260814 #propdef-wrap-inside (1602–1622), #propdef-wrap-before /
//! #propdef-wrap-after (1680–1736), #propdef-line-break (2486–2521) supply
//! independent finite keyword/initial/inheritance oracles. Text3 CRD20260814
//! agrees on line-break. Syntax3 recovery and existing strict checked contracts
//! distinguish browser implicit closure from original-component admission.
//! Adopted keyword price is1input/1projection; declaration/name add2. Output
//! punctuation/importance cost bytes only; whole-sheet work totals are not guessed.
//! No new typed models, wrapper symbols, borrowed variants, stubs or private
//! helpers are referenced. Exact new payload/primitive provider/suppression and
//! new typed API tests accompany functional implementation. Opaque initial/value
//! equality is exercised through existing generic public boundaries.
//! Nested breaking, flex/forced propagation, language and white-space execution
//! remain contextual. This group neither changes nor certifies #53 constituents.

use surgeist_css::CssKnownProperty as P;
use surgeist_css::*;

#[path = "common/authored_property.rs"]
mod authored_property;
use authored_property::assert_source;

const FAMILY: [&str; 4] = ["wrap-inside", "wrap-before", "wrap-after", "line-break"];
const INSIDE: [&str; 2] = ["auto", "avoid"];
const BOUNDARY: [&str; 6] = ["auto", "avoid", "avoid-line", "avoid-flex", "line", "flex"];
const LINE_BREAK: [&str; 5] = ["auto", "loose", "normal", "strict", "anywhere"];
const GLOBALS: [(&str, CssGlobalKeyword); 5] = [
    ("initial", CssGlobalKeyword::Initial),
    ("inherit", CssGlobalKeyword::Inherit),
    ("unset", CssGlobalKeyword::Unset),
    ("revert", CssGlobalKeyword::Revert),
    ("revert-layer", CssGlobalKeyword::RevertLayer),
];
fn keywords(name: &str) -> &'static [&'static str] {
    match name {
        "wrap-inside" => &INSIDE,
        "wrap-before" | "wrap-after" => &BOUNDARY,
        "line-break" => &LINE_BREAK,
        _ => panic!("selected finite grammar"),
    }
}
fn property(name: &str) -> P {
    P::from_name(name).unwrap_or_else(|| panic!("authored property unavailable: {name}"))
}
fn parsed(name: &str, value: &str) -> CssDeclaration {
    authored_property::parsed(property(name), value)
}
fn checked_components(
    name: &str,
    value: CssComponentValues,
    grammar: bool,
) -> Result<CssDeclaration, CssPropertyValueParseError> {
    authored_property::checked_components(property(name), value, grammar, CssImportance::Important)
}
fn checked(name: &str, value: &str, grammar: bool) -> CssDeclaration {
    authored_property::checked(property(name), value, grammar)
}
fn fronts(name: &str, text: &str) -> [CssDeclaration; 5] {
    authored_property::fronts(property(name), text)
}
fn completed(source: &CssDeclaration) -> CssLonghandContributions {
    let CssExpansion::Contributions(CssContributions::Longhands(values)) =
        expand_declaration(source).unwrap()
    else {
        panic!("complete longhand contribution")
    };
    assert_eq!(values.items().len(), 1);
    let item = &values.items()[0];
    assert_eq!(item.property(), source.known().unwrap().property());
    assert_source(item, source);
    values
}
fn accepted(name: &str, input: &str, canonical: &str) {
    for source in fronts(name, input) {
        let before = source.clone();
        assert_eq!(
            source.to_specified_css().unwrap(),
            format!("{name}: {canonical} !important;")
        );
        assert!(source.known().unwrap().property_value().is_some());
        assert!(source.known().unwrap().global().is_none());
        assert!(source.known().unwrap().substitution_dependent().is_none());
        let values = completed(&source);
        let item = &values.items()[0];
        assert!(matches!(item.value(), CssContributionValueRef::Ordinary(_)));
        assert_eq!(
            item.ordinary_value().unwrap().property().known_property(),
            property(name)
        );
        assert!(item.replacement_components().is_none());
        assert_eq!(source, before);
    }
    assert_eq!(
        checked(name, canonical, true).to_specified_css().unwrap(),
        format!("{name}: {canonical} !important;")
    );
}
fn invalid(name: &str, value: &str) {
    let p = property(name);
    let css = format!("color:red;{name}:{value};color:blue");
    let report = parse_style_attribute(&css);
    assert_eq!(report.syntax().len(), 2, "{css}");
    assert!(
        report
            .syntax()
            .iter()
            .all(|v| v.known().unwrap().property() == P::Color)
    );
    let [diagnostic] = report.diagnostics() else {
        panic!("one atomic grammar failure")
    };
    assert_eq!(diagnostic.action(), CssRecoveryAction::DropDeclaration);
    let ErrorKind::InvalidPropertyValue(detail) = diagnostic.error().kind() else {
        panic!("property-specific diagnostic")
    };
    assert_eq!(detail.property(), p);
    assert_eq!(
        validate_style_attribute(&css).unwrap_err().diagnostics(),
        report.diagnostics()
    );
    let components = parse_component_values(value).unwrap();
    let before = components.clone();
    for grammar in [false, true] {
        assert!(matches!(
            checked_components(name, components.clone(), grammar)
                .unwrap_err()
                .kind(),
            CssPropertyValueErrorKind::Grammar(_)
        ));
    }
    for report in [
        parse_property_value_text(value, CssPropertyNameRef::Known(p), CssImportance::Normal),
        parse_property_value_text_for_grammar(value, p.grammar(), CssImportance::Normal),
    ] {
        assert!(report.syntax().is_none());
        assert!(!report.is_clean());
    }
    assert_eq!(components, before);
}

#[test]
fn every_selected_keyword_has_canonical_identity_and_one_terminal_through_five_front_doors() {
    for name in FAMILY {
        // Parsed admission precedes runtime lookup, so absent grammar is a real
        // callable-boundary failure without naming absent compile-time symbols.
        let source = parsed(name, "auto");
        assert_eq!(source.known().unwrap().property(), property(name));
        assert_eq!(
            P::from_name(&name.to_ascii_uppercase()),
            Some(property(name))
        );
        assert_eq!(
            CssPropertyGrammar::from_name(name),
            Some(property(name).grammar())
        );
        for keyword in keywords(name) {
            accepted(name, keyword, keyword);
        }
    }
}
#[test]
fn case_escaped_keywords_and_comments_keep_original_components_and_canonical_spelling() {
    for (name, input, canonical) in [
        ("wrap-inside", r"\61 void", "avoid"),
        ("wrap-before", r"\61 void-flex", "avoid-flex"),
        ("wrap-after", r"\6c ine", "line"),
        ("line-break", r"\61 nywhere", "anywhere"),
    ] {
        accepted(name, input, canonical);
        accepted(
            name,
            &format!("/**/{} /**/", canonical.to_ascii_uppercase()),
            canonical,
        );
    }
    let report = parse_style_attribute(r"\77 rap-before:AvOiD-LiNe!important");
    assert!(report.is_clean());
    assert_eq!(
        report.syntax()[0].known().unwrap().property(),
        property("wrap-before")
    );
    assert_eq!(
        report.syntax()[0].to_specified_css().unwrap(),
        "wrap-before: avoid-line !important;"
    );
}
#[test]
fn duplicate_and_multiple_choices_reject_atomically_and_keep_color_siblings() {
    for name in FAMILY {
        for left in keywords(name) {
            for right in keywords(name) {
                invalid(name, &format!("{left} {right}"));
            }
        }
        for value in [
            "",
            "none",
            "nowrap",
            "wrap",
            "pretty",
            "balance",
            "1",
            "-1px",
            "25%",
            "calc(1 + 2)",
            "min(1,2)",
            "auto,auto",
            "[auto]",
            "\"auto\"",
            "initial auto",
            "auto inherit",
        ] {
            invalid(name, value);
        }
    }
    for value in [
        "avoid-line",
        "avoid-flex",
        "line",
        "flex",
        "loose",
        "normal",
        "strict",
        "anywhere",
    ] {
        invalid("wrap-inside", value);
    }
    for name in ["wrap-before", "wrap-after"] {
        for value in ["loose", "normal", "strict", "anywhere"] {
            invalid(name, value);
        }
    }
    for value in ["avoid", "avoid-line", "avoid-flex", "line", "flex"] {
        invalid("line-break", value);
    }
}
#[test]
fn programmatic_keyword_components_preserve_origins_identity_and_importance() {
    for name in FAMILY {
        for keyword in keywords(name) {
            let components =
                CssComponentValues::try_new(vec![CssComponentValue::try_ident(*keyword).unwrap()])
                    .unwrap();
            let before = components.clone();
            for grammar in [false, true] {
                let source = checked_components(name, components.clone(), grammar).unwrap();
                assert_eq!(source.value_components(), &components);
                assert_eq!(
                    source.value_components().items()[0].origin(),
                    &CssValueOrigin::Programmatic
                );
                assert_eq!(source.importance(), CssImportance::Important);
                assert!(source.position().is_none());
                assert!(source.parsed_name().is_none());
                assert!(source.parsed_value().is_none());
                assert_eq!(
                    source.to_specified_css().unwrap(),
                    format!("{name}: {keyword} !important;")
                );
                completed(&source);
            }
            assert_eq!(components, before);
        }
    }
}
#[test]
fn auto_initials_are_ordinary_longhands_and_only_line_break_inherits() {
    for name in FAMILY {
        let p = property(name);
        let CssPropertyKindRef::Longhand(metadata) = p.metadata().unwrap().kind() else {
            panic!("one longhand, no shorthand/reset members")
        };
        assert_eq!(metadata.property().known_property(), p);
        assert_eq!(metadata.inherited_by_default(), name == "line-break");
        let initial = metadata.initial_value();
        assert_eq!(initial.property().known_property(), p);
        let CssInitialValueRef::Value(initial) = initial.view() else {
            panic!("ordinary auto initial")
        };
        let components =
            CssComponentValues::try_new(vec![CssComponentValue::try_ident("auto").unwrap()])
                .unwrap();
        let auto = checked_components(name, components, false).unwrap();
        assert_eq!(
            auto.to_specified_css().unwrap(),
            format!("{name}: auto !important;")
        );
        let values = completed(&auto);
        assert_eq!(values.items()[0].ordinary_value(), Some(initial));
    }
}
#[test]
fn all_css_wide_values_are_symbolic_single_terminals_preserving_occurrence_and_importance() {
    for name in FAMILY {
        for (text, keyword) in GLOBALS {
            for source in fronts(name, text) {
                assert_eq!(source.known().unwrap().global(), Some(keyword));
                assert!(source.known().unwrap().property_value().is_none());
                assert_eq!(
                    source.to_specified_css().unwrap(),
                    format!("{name}: {text} !important;")
                );
                let values = completed(&source);
                let item = &values.items()[0];
                assert!(matches!(item.value(),CssContributionValueRef::Global(v) if v == keyword));
                assert!(item.replacement_components().is_none());
            }
        }
    }
}
#[test]
fn var_env_attr_reentry_admits_every_keyword_and_keeps_original_and_replacement_snapshots() {
    for name in FAMILY {
        for pending in ["var(--wrap)", "env(wrap)", "attr(data-wrap *)"] {
            for source in fronts(name, pending) {
                let before = source.clone();
                let CssExpansion::Pending(handle) = expand_declaration(&source).unwrap() else {
                    panic!("pending handle")
                };
                assert!(handle.source().same_occurrence(&source));
                assert_eq!(handle.source().importance(), CssImportance::Important);
                assert_eq!(
                    source.to_specified_css().unwrap(),
                    format!("{name}: {pending} !important;")
                );
                for text in keywords(name) {
                    let replacement = parse_component_values(&text.to_ascii_uppercase()).unwrap();
                    let snapshot = replacement.clone();
                    let CssContributions::Longhands(values) =
                        handle.reenter(replacement.clone()).unwrap()
                    else {
                        panic!("ordinary replacement")
                    };
                    assert_eq!(values.items().len(), 1);
                    let item = &values.items()[0];
                    assert_eq!(item.property(), property(name));
                    assert_source(item, &source);
                    assert!(matches!(item.value(), CssContributionValueRef::Ordinary(_)));
                    assert_eq!(item.replacement_components(), Some(&replacement));
                    for (actual, supplied) in item
                        .replacement_components()
                        .unwrap()
                        .items()
                        .iter()
                        .zip(replacement.items())
                    {
                        assert_eq!(actual.origin(), supplied.origin());
                        let (CssValueOrigin::Parsed(actual), CssValueOrigin::Parsed(supplied)) =
                            (actual.origin(), supplied.origin())
                        else {
                            panic!("replacement origin")
                        };
                        assert!(actual.source().same_snapshot(supplied.source()));
                        assert_eq!(actual.source().as_str(), text.to_ascii_uppercase());
                    }
                    let ordinary = checked(name, text, true);
                    assert_eq!(
                        ordinary.to_specified_css().unwrap(),
                        format!("{name}: {text} !important;")
                    );
                    let ordinary_values = completed(&ordinary);
                    assert_eq!(
                        item.ordinary_value(),
                        ordinary_values.items()[0].ordinary_value()
                    );
                    assert_eq!(replacement, snapshot);
                }
                for (text, keyword) in GLOBALS {
                    let replacement = parse_component_values(text).unwrap();
                    let CssContributions::Longhands(values) =
                        handle.reenter(replacement.clone()).unwrap()
                    else {
                        panic!("global replacement")
                    };
                    assert_eq!(values.items().len(), 1);
                    let item = &values.items()[0];
                    assert_source(item, &source);
                    assert!(
                        matches!(item.value(),CssContributionValueRef::Global(v) if v == keyword)
                    );
                    assert_eq!(item.replacement_components(), Some(&replacement));
                }
                assert_eq!(source, before);
            }
        }
    }
}
#[test]
fn invalid_or_residual_replacements_fail_atomically_and_the_same_handle_can_retry() {
    for name in FAMILY {
        for pending in ["var(--wrap)", "env(wrap)", "attr(data-wrap *)"] {
            let source = checked(name, pending, true);
            let before = source.clone();
            let CssExpansion::Pending(handle) = expand_declaration(&source).unwrap() else {
                panic!("pending")
            };
            for text in [
                "var(--again)",
                "env(again)",
                "attr(data-again *)",
                "auto var(--again)",
                r"\76 ar(--again)",
            ] {
                for _ in 0..2 {
                    assert!(matches!(
                        handle
                            .reenter(parse_component_values(text).unwrap())
                            .unwrap_err()
                            .kind(),
                        CssExpansionErrorKind::ResidualSubstitution
                    ));
                }
            }
            for text in ["auto auto", "none", "auto!important", "auto;color:red"] {
                for _ in 0..2 {
                    let error = handle
                        .reenter(parse_component_values(text).unwrap())
                        .unwrap_err();
                    let CssExpansionErrorKind::InvalidReplacement(error) = error.kind() else {
                        panic!("strict replacement")
                    };
                    assert!(matches!(
                        error.kind(),
                        CssPropertyValueErrorKind::Grammar(_)
                    ));
                }
            }
            let replacement = parse_component_values("auto").unwrap();
            let CssContributions::Longhands(values) = handle.reenter(replacement.clone()).unwrap()
            else {
                panic!("successful reusable retry")
            };
            assert_eq!(values.items().len(), 1);
            assert_source(&values.items()[0], &source);
            assert_eq!(
                values.items()[0].replacement_components(),
                Some(&replacement)
            );
            assert!(handle.source().same_occurrence(&source));
            assert_eq!(source, before);
        }
    }
}
fn implicit_origin(components: &CssComponentValues) -> CssValueOrigin {
    for value in components.items() {
        let closing = match value.view() {
            CssComponentValueRef::Function(v) => Some(v.closing_origin()),
            CssComponentValueRef::Block(v) => Some(v.closing_origin()),
            _ => None,
        };
        if let Some(origin @ CssValueOrigin::ImplicitClosure { .. }) = closing {
            return origin.clone();
        }
    }
    let serialized = components.serialize().unwrap();
    (0..serialized.as_css().len())
        .find_map(|offset| match serialized.origin_at(offset) {
            Some(CssSerializedOrigin::Token(origin @ CssValueOrigin::ImplicitClosure { .. })) => {
                Some(origin.clone())
            }
            _ => None,
        })
        .expect("public original implicit closing origin")
}
fn assert_closure(error: &CssPropertyValueParseError, origin: &CssValueOrigin, text: &str) {
    assert!(matches!(
        error.kind(),
        CssPropertyValueErrorKind::Grammar(ErrorKind::UnexpectedEnd(_))
    ));
    assert_eq!(
        error.origin(),
        &CssSerializedOrigin::End(Some(origin.clone()))
    );
    let CssValueOrigin::ImplicitClosure { opening, at } = origin else {
        panic!("implicit EOF")
    };
    assert_eq!(opening.source().as_str(), text);
    assert!(opening.source().same_snapshot(at.source()));
    assert_eq!(at.span().start().byte_offset().value(), text.len());
    assert_eq!(at.span().end().byte_offset().value(), text.len());
}
#[test]
fn both_checked_fronts_reject_original_comment_and_pending_function_closures() {
    for name in FAMILY {
        let mut texts: Vec<String> = keywords(name)
            .iter()
            .map(|text| format!("{text}/*"))
            .collect();
        texts.extend(GLOBALS.map(|(text, _)| format!("{text}/*")));
        texts.extend(
            [
                "var(--wrap)/*",
                "env(wrap)/*",
                "attr(data-wrap *)/*",
                "var(--wrap",
                "env(wrap",
                "attr(data-wrap *",
            ]
            .map(str::to_owned),
        );
        for text in texts {
            let components = parse_component_values(&text).unwrap();
            let before = components.clone();
            let origin = implicit_origin(&components);
            for grammar in [false, true] {
                assert_closure(
                    &checked_components(name, components.clone(), grammar).unwrap_err(),
                    &origin,
                    &text,
                );
            }
            assert_eq!(components, before);
        }
    }
}
#[test]
fn replacement_closure_is_strict_and_residual_priority_and_complete_comment_controls_hold() {
    for name in FAMILY {
        for pending in ["var(--wrap)", "env(wrap)", "attr(data-wrap *)"] {
            let source = checked(name, pending, false);
            let CssExpansion::Pending(handle) = expand_declaration(&source).unwrap() else {
                panic!("pending")
            };
            let mut texts: Vec<String> = keywords(name)
                .iter()
                .map(|text| format!("{text}/*"))
                .collect();
            texts.extend(GLOBALS.map(|(text, _)| format!("{text}/*")));
            for text in texts {
                let components = parse_component_values(&text).unwrap();
                let origin = implicit_origin(&components);
                for _ in 0..2 {
                    let error = handle.reenter(components.clone()).unwrap_err();
                    let CssExpansionErrorKind::InvalidReplacement(error) = error.kind() else {
                        panic!("original replacement closure")
                    };
                    assert_closure(error, &origin, &text);
                }
            }
            for text in [
                "var(--again",
                "env(again",
                "attr(data-again *",
                "var(--again)/*",
            ] {
                assert!(matches!(
                    handle
                        .reenter(parse_component_values(text).unwrap())
                        .unwrap_err()
                        .kind(),
                    CssExpansionErrorKind::ResidualSubstitution
                ));
            }
            for text in [
                "auto/**/",
                "initial/**/",
                "var(--wrap)/**/",
                "env(wrap)/**/",
                "attr(data-wrap *)/**/",
            ] {
                checked(name, text, false);
                checked(name, text, true);
            }
            assert!(
                handle
                    .reenter(parse_component_values("auto/**/").unwrap())
                    .is_ok()
            );
        }
    }
}
#[test]
fn checked_value_annotations_map_to_original_tokens_and_authored_importance_is_valid() {
    let text = "/*😀*/auto!important";
    let components = parse_component_values(text).unwrap();
    let serialized = components.serialize().unwrap();
    let origin = serialized
        .origin_at(serialized.as_css().find('!').unwrap())
        .unwrap();
    for name in FAMILY {
        for grammar in [false, true] {
            let error = checked_components(name, components.clone(), grammar).unwrap_err();
            assert!(matches!(
                error.kind(),
                CssPropertyValueErrorKind::Grammar(_)
            ));
            assert_eq!(error.origin(), origin);
        }
        for report in [
            parse_property_value_text(
                text,
                CssPropertyNameRef::Known(property(name)),
                CssImportance::Important,
            ),
            parse_property_value_text_for_grammar(
                text,
                property(name).grammar(),
                CssImportance::Important,
            ),
        ] {
            assert!(report.syntax().is_none());
            assert!(!report.is_clean());
        }
        accepted(name, "auto", "auto");
    }
}
#[test]
fn browser_eof_recovery_retains_diagnostics_and_normalized_original_occurrences() {
    for name in FAMILY {
        let mut texts: Vec<String> = ["auto/*", "var(--wrap", "env(wrap", "attr(data-wrap *"]
            .map(str::to_owned)
            .to_vec();
        texts.extend(GLOBALS.map(|(text, _)| format!("{text}/*")));
        for text in texts {
            let css = format!(".a{{{name}:{text}");
            let report = parse_sheet(&css);
            assert!(!report.is_clean());
            assert!(validate_sheet(&css).is_err());
            assert!(
                report
                    .diagnostics()
                    .iter()
                    .any(|v| v.action() == CssRecoveryAction::RetainWithImplicitClosure)
            );
            let before = report.clone();
            let exact = CssNormalizationLimits::try_new(0, 1, 1, 1).unwrap();
            let normalized = normalize_report_with_limits(&report, exact).unwrap();
            assert_eq!(normalized.diagnostics(), report.diagnostics());
            let declarations: Vec<_> = normalized
                .syntax()
                .items()
                .iter()
                .filter_map(|v| match v {
                    CssNormalizedItem::Declaration(v) => Some(v),
                    _ => None,
                })
                .collect();
            let [item] = declarations.as_slice() else {
                panic!("retained recovered declaration")
            };
            assert_eq!(item.order(), 0);
            assert_eq!(item.source().known().unwrap().property(), property(name));
            assert_eq!(item.source().parsed_name().unwrap().source().as_str(), css);
            match item.expansion() {
                CssExpansion::Pending(handle) => {
                    assert!(handle.source().same_occurrence(item.source()))
                }
                CssExpansion::Contributions(CssContributions::Longhands(values)) => {
                    assert_eq!(values.items().len(), 1);
                    assert_source(&values.items()[0], item.source());
                }
                _ => panic!("retained intrinsic lifecycle"),
            }
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
            assert_eq!(error.declaration_order(), Some(0));
            assert!(error.declaration().unwrap().same_occurrence(item.source()));
            assert_eq!(report, before);
            assert!(normalize_report_with_limits(&report, exact).is_ok());
        }
    }
}
#[test]
fn normalization_preserves_order_contexts_importance_and_exact_cumulative_terminal_counts() {
    let report = parse_sheet(
        "@media screen{.a{wrap-inside:avoid!important;wrap-before:line;wrap-after:flex;line-break:anywhere;wrap-before:var(--before);wrap-after:unset!important;line-break:normal;wrap-inside:auto;color:red}}",
    );
    assert!(report.is_clean());
    let before = report.clone();
    let exact = CssNormalizationLimits::try_new(1, 2, 9, 9).unwrap();
    let normalized = normalize_sheet_with_limits(report.syntax(), exact).unwrap();
    let declarations: Vec<_> = normalized
        .items()
        .iter()
        .filter_map(|v| match v {
            CssNormalizedItem::Declaration(v) => Some(v),
            _ => None,
        })
        .collect();
    assert_eq!(declarations.len(), 9);
    let names = [
        "wrap-inside",
        "wrap-before",
        "wrap-after",
        "line-break",
        "wrap-before",
        "wrap-after",
        "line-break",
        "wrap-inside",
        "color",
    ];
    for (index, item) in declarations.iter().enumerate() {
        assert_eq!(item.order(), index);
        assert_eq!(
            item.source().known().unwrap().property().canonical_name(),
            names[index]
        );
        assert_eq!(
            item.source().importance(),
            if index == 0 || index == 5 {
                CssImportance::Important
            } else {
                CssImportance::Normal
            }
        );
        assert!(
            item.selector_context()
                .same_context(declarations[0].selector_context())
        );
        assert!(
            item.rule_context()
                .same_context(declarations[0].rule_context())
        );
        if index > 0 {
            assert!(
                !item
                    .source()
                    .same_occurrence(declarations[index - 1].source())
            );
        }
        match item.expansion() {
            CssExpansion::Pending(handle) => {
                assert_eq!(index, 4);
                assert!(handle.source().same_occurrence(item.source()));
            }
            CssExpansion::Contributions(CssContributions::Longhands(values)) => {
                assert_eq!(values.items().len(), 1);
                assert_source(&values.items()[0], item.source());
                assert_eq!(
                    values.items()[0].property(),
                    item.source().known().unwrap().property()
                );
                if index == 5 {
                    assert!(matches!(
                        values.items()[0].value(),
                        CssContributionValueRef::Global(CssGlobalKeyword::Unset)
                    ));
                } else {
                    assert!(matches!(
                        values.items()[0].value(),
                        CssContributionValueRef::Ordinary(_)
                    ));
                }
            }
            _ => panic!("intrinsic single terminal/handle"),
        }
    }
    for (limits, resource) in [
        (
            CssNormalizationLimits::try_new(1, 2, 9, 8).unwrap(),
            CssNormalizationResource::Contributions,
        ),
        (
            CssNormalizationLimits::try_new(1, 2, 8, 9).unwrap(),
            CssNormalizationResource::Declarations,
        ),
    ] {
        for _ in 0..2 {
            let error = normalize_sheet_with_limits(report.syntax(), limits).unwrap_err();
            assert_eq!(
                error.kind(),
                &CssNormalizationErrorKind::LimitExceeded { resource, limit: 8 }
            );
            assert_eq!(error.declaration_order(), Some(8));
            assert!(
                error
                    .declaration()
                    .unwrap()
                    .same_occurrence(declarations[8].source())
            );
        }
    }
    assert_eq!(report, before);
    assert!(normalize_sheet_with_limits(report.syntax(), exact).is_ok());
}
#[test]
fn canonical_keyword_declarations_have_exact_node_byte_budgets_and_atomic_reusable_failures() {
    use CssSpecifiedValueSerializationErrorKind as Kind;
    use CssSpecifiedValueSerializationLimits as Limits;
    for name in FAMILY {
        let values: Vec<_> = keywords(name)
            .iter()
            .copied()
            .chain(GLOBALS.map(|(text, _)| text))
            .collect();
        for text in values {
            for source in [parsed(name, text), checked(name, text, true)] {
                let before = source.clone();
                let expected = format!("{name}: {text} !important;");
                let exact = Limits::new(3, 3, expected.len());
                assert_eq!(
                    source.to_specified_css_with_limits(exact).unwrap(),
                    expected
                );
                for (limits, kind) in [
                    (Limits::new(2, 3, expected.len()), Kind::InputNodeLimit),
                    (Limits::new(3, 2, expected.len()), Kind::ProjectionNodeLimit),
                    (Limits::new(3, 3, expected.len() - 1), Kind::ByteLimit),
                ] {
                    for _ in 0..2 {
                        assert_eq!(
                            source
                                .to_specified_css_with_limits(limits)
                                .unwrap_err()
                                .kind(),
                            kind
                        );
                        assert_eq!(source, before);
                    }
                }
                assert_eq!(
                    source.to_specified_css_with_limits(exact).unwrap(),
                    expected
                );
            }
        }
    }
}
#[test]
fn sheet_siblings_share_exact_final_bytes_and_failed_output_leaves_the_input_reusable() {
    use CssSpecifiedValueSerializationLimits as Limits;
    let report = parse_sheet(
        ".a{wrap-inside:avoid!important}.b{wrap-before:line}.c{wrap-after:flex}.d{line-break:anywhere}",
    );
    assert!(report.is_clean());
    let sheet = report.syntax();
    let before = sheet.clone();
    let expected = ".a { wrap-inside: avoid !important; }\n.b { wrap-before: line; }\n.c { wrap-after: flex; }\n.d { line-break: anywhere; }";
    let exact = Limits::new(usize::MAX, usize::MAX, expected.len());
    let short = Limits::new(usize::MAX, usize::MAX, expected.len() - 1);
    assert_eq!(sheet.to_specified_css_with_limits(exact).unwrap(), expected);
    assert!(
        sheet
            .rules()
            .iter()
            .all(|rule| rule.to_specified_css_with_limits(short).is_ok())
    );
    for _ in 0..2 {
        let error = sheet.to_specified_css_with_limits(short).unwrap_err();
        assert_eq!(
            error.kind(),
            CssSpecifiedRuleSerializationErrorKind::Resource(
                CssSpecifiedValueSerializationErrorKind::ByteLimit
            )
        );
        assert_eq!(error.rule_index(), Some(3));
        assert_eq!(sheet, &before);
    }
    assert_eq!(sheet.to_specified_css_with_limits(exact).unwrap(), expected);
}
#[test]
fn existing_flow_flex_grid_item_and_color_controls_execute_independently_of_wrap_lookup() {
    for (name, input, expected, count) in [
        (
            "flow-tolerance",
            "calc(-2px - 3%)",
            "flow-tolerance: calc(-3% - 2px);",
            1,
        ),
        (
            "flex-flow",
            "wrap row-reverse",
            "flex-flow: row-reverse wrap;",
            2,
        ),
        ("grid-template", "none", "grid-template: none;", 3),
        ("item-flow", "row", "item-flow: row auto normal normal;", 4),
        ("color", "red", "color: red;", 1),
    ] {
        let report = parse_style_attribute(&format!("{name}:{input}"));
        assert!(report.is_clean());
        for source in report.syntax().iter() {
            assert_eq!(source.to_specified_css().unwrap(), expected);
            let CssExpansion::Contributions(CssContributions::Longhands(values)) =
                expand_declaration(source).unwrap()
            else {
                panic!("supported independent control")
            };
            assert_eq!(values.items().len(), count);
            for item in values.items() {
                assert_source(item, source);
            }
        }
    }
}
