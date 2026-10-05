#![forbid(unsafe_code)]
//! Independent authored contract: Text4 WD20260814 #propdef-text-justify,
//! #valdef-text-justify-distribute and #propdef-text-group-align; Values3/4
//! §2.2 and Text4 §1.2 supply grammar, CSS-wide exclusivity and metadata.
//! Base-before-modifier ordering and transparent keyword prices are adopted
//! Surgeist policies; the text-justify canonical-order table says n/a.
//! Existing public front doors only. New typed constructors/borrowed views,
//! direct primitive providers and private suppression tests accompany functional
//! implementation; computed distribute equivalence remains downstream.

use surgeist_css::CssKnownProperty as P;
use surgeist_css::*;

const GLOBALS: [(&str, CssGlobalKeyword); 5] = [
    ("initial", CssGlobalKeyword::Initial),
    ("inherit", CssGlobalKeyword::Inherit),
    ("unset", CssGlobalKeyword::Unset),
    ("revert", CssGlobalKeyword::Revert),
    ("revert-layer", CssGlobalKeyword::RevertLayer),
];
fn property(name: &str) -> P {
    P::from_name(name).unwrap_or_else(|| panic!("authored property unavailable: {name}"))
}
fn parsed(name: &str, value: &str) -> CssDeclaration {
    let css = format!("/*😀*/{}:{value}!important", name.to_ascii_uppercase());
    let report = parse_style_attribute(&css);
    assert!(report.is_clean(), "{css}: {:?}", report.diagnostics());
    assert!(validate_style_attribute(&css).is_ok());
    let [source] = report.syntax().as_slice() else {
        panic!("one occurrence")
    };
    assert_eq!(source.known().unwrap().property().canonical_name(), name);
    assert_eq!(source.importance(), CssImportance::Important);
    assert!(source.position().is_some());
    assert_eq!(source.parsed_name().unwrap().source().as_str(), css);
    assert_eq!(source.parsed_value().unwrap().source().as_str(), css);
    source.clone()
}
fn checked_components(
    name: &str,
    value: CssComponentValues,
    grammar: bool,
) -> Result<CssDeclaration, CssPropertyValueParseError> {
    let p = property(name);
    if grammar {
        parse_property_value_for_grammar(p.grammar(), value, CssImportance::Important)
    } else {
        parse_property_value(
            CssPropertyNameRef::Known(p),
            value,
            CssImportance::Important,
        )
    }
}
fn checked(name: &str, text: &str, grammar: bool) -> CssDeclaration {
    let components = parse_component_values(text).unwrap();
    let before = components.clone();
    let source = checked_components(name, components.clone(), grammar).unwrap();
    assert_eq!(components, before);
    assert_eq!(source.value_components(), &components);
    assert!(source.position().is_none());
    assert!(source.parsed_name().is_none());
    assert!(source.parsed_value().is_none());
    assert_eq!(source.importance(), CssImportance::Important);
    assert_eq!(source.known().unwrap().grammar(), property(name).grammar());
    source
}
fn text_front(name: &str, text: &str, grammar: bool) -> CssDeclaration {
    let p = property(name);
    let report = if grammar {
        parse_property_value_text_for_grammar(text, p.grammar(), CssImportance::Important)
    } else {
        parse_property_value_text(text, CssPropertyNameRef::Known(p), CssImportance::Important)
    };
    assert!(
        report.is_clean(),
        "{name}:{text}: {:?}",
        report.diagnostics()
    );
    let source = report.syntax().as_ref().unwrap().clone();
    assert_eq!(source.known().unwrap().grammar(), p.grammar());
    assert_eq!(source.importance(), CssImportance::Important);
    assert!(source.position().is_none());
    assert!(source.parsed_name().is_none());
    assert_eq!(source.parsed_value().unwrap().source().as_str(), text);
    source
}
fn fronts(name: &str, text: &str) -> [CssDeclaration; 5] {
    [
        parsed(name, text),
        checked(name, text, false),
        checked(name, text, true),
        text_front(name, text, false),
        text_front(name, text, true),
    ]
}
fn assert_source(item: &CssLonghandContribution, source: &CssDeclaration) {
    assert!(item.source().same_occurrence(source));
    assert_eq!(item.source().importance(), source.importance());
    assert_eq!(item.source().position(), source.position());
    assert_eq!(item.source().value_components(), source.value_components());
    assert_eq!(
        item.source().known().unwrap().grammar(),
        source.known().unwrap().grammar()
    );
}
fn contributions(source: &CssDeclaration) -> CssLonghandContributions {
    let CssExpansion::Contributions(CssContributions::Longhands(values)) =
        expand_declaration(source).unwrap()
    else {
        panic!("completed intrinsic contribution")
    };
    let name = source.known().unwrap().property().canonical_name();
    assert_eq!(values.items().len(), terminals(name).len());
    for (item, name) in values.items().iter().zip(terminals(name)) {
        assert_eq!(item.property(), property(name));
        assert_source(item, source);
    }
    values
}
fn canonical(source: &CssDeclaration, name: &str, text: &str) {
    assert_eq!(
        source.to_specified_css().unwrap(),
        format!("{name}: {text} !important;")
    );
}
fn accepted(name: &str, input: &str, expected: &str) {
    for source in fronts(name, input) {
        let before = source.clone();
        canonical(&source, name, expected);
        assert!(source.known().unwrap().property_value().is_some());
        assert!(source.known().unwrap().global().is_none());
        assert!(source.known().unwrap().substitution_dependent().is_none());
        for item in contributions(&source).items() {
            assert!(matches!(item.value(), CssContributionValueRef::Ordinary(_)));
            assert_eq!(
                item.ordinary_value().unwrap().property().known_property(),
                item.property()
            );
            assert!(item.replacement_components().is_none());
        }
        assert_eq!(source, before);
    }
    canonical(&checked(name, expected, true), name, expected);
}
fn invalid(name: &str, text: &str) {
    let p = property(name);
    let css = format!("color:red;{name}:{text};color:blue");
    let report = parse_style_attribute(&css);
    assert_eq!(report.syntax().len(), 2, "{css}");
    assert!(
        report
            .syntax()
            .iter()
            .all(|v| v.known().unwrap().property() == P::Color)
    );
    let [diagnostic] = report.diagnostics() else {
        panic!("one atomic grammar failure: {css}")
    };
    assert_eq!(diagnostic.action(), CssRecoveryAction::DropDeclaration);
    let ErrorKind::InvalidPropertyValue(detail) = diagnostic.error().kind() else {
        panic!("property-specific grammar error")
    };
    assert_eq!(detail.property(), p);
    assert_eq!(
        validate_style_attribute(&css).unwrap_err().diagnostics(),
        report.diagnostics()
    );
    let components = parse_component_values(text).unwrap();
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
        parse_property_value_text(text, CssPropertyNameRef::Known(p), CssImportance::Normal),
        parse_property_value_text_for_grammar(text, p.grammar(), CssImportance::Normal),
    ] {
        assert!(report.syntax().is_none());
        assert!(!report.is_clean());
    }
    assert_eq!(components, before);
}
fn permutations(words: &[&str]) -> Vec<String> {
    fn walk(words: &[&str], used: &mut Vec<bool>, prefix: &mut Vec<String>, out: &mut Vec<String>) {
        if prefix.len() == words.len() {
            out.push(prefix.join(" "));
            return;
        }
        for i in 0..words.len() {
            if !used[i] {
                used[i] = true;
                prefix.push(words[i].to_owned());
                walk(words, used, prefix, out);
                prefix.pop();
                used[i] = false;
            }
        }
    }
    let mut out = Vec::new();
    walk(
        words,
        &mut vec![false; words.len()],
        &mut Vec::new(),
        &mut out,
    );
    out
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
        .expect("public original implicit EOF origin")
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
fn css_wide_values_are_exclusive_and_expand_each_intrinsic_terminal_in_source_order() {
    for name in FAMILY {
        for (text, keyword) in GLOBALS {
            for source in fronts(name, text) {
                assert_eq!(source.known().unwrap().global(), Some(keyword));
                canonical(&source, name, text);
                for item in contributions(&source).items() {
                    assert!(
                        matches!(item.value(),CssContributionValueRef::Global(v) if v==keyword)
                    );
                    assert!(item.replacement_components().is_none());
                }
            }
            for ordinary in ordinary_cases(name) {
                invalid(name, &format!("{text} {ordinary}"));
                invalid(name, &format!("{ordinary} {text}"));
            }
            for (other, _) in GLOBALS {
                invalid(name, &format!("{text} {other}"));
            }
        }
    }
}
#[test]
fn programmatic_components_preserve_origin_identity_and_importance_at_both_checked_fronts() {
    for name in FAMILY {
        let text = ordinary_cases(name)[0];
        let value = if name == "line-padding" {
            CssComponentValue::try_number("0").unwrap()
        } else {
            CssComponentValue::try_ident(text).unwrap()
        };
        let components = CssComponentValues::try_new(vec![value]).unwrap();
        for grammar in [false, true] {
            let source = checked_components(name, components.clone(), grammar).unwrap();
            assert_eq!(source.value_components(), &components);
            assert_eq!(
                source.value_components().items()[0].origin(),
                &CssValueOrigin::Programmatic
            );
            assert!(source.parsed_value().is_none());
            assert_eq!(source.importance(), CssImportance::Important);
            canonical(&source, name, text);
            contributions(&source);
        }
    }
}
#[test]
fn pending_var_env_attr_reentry_preserves_original_occurrence_and_replacement_origins() {
    for name in FAMILY {
        for pending in ["var(--text)", "env(text)", "attr(data-text *)"] {
            for source in fronts(name, pending) {
                let before = source.clone();
                canonical(&source, name, pending);
                let CssExpansion::Pending(handle) = expand_declaration(&source).unwrap() else {
                    panic!("pending")
                };
                assert!(handle.source().same_occurrence(&source));
                for text in ordinary_cases(name) {
                    let replacement = parse_component_values(text).unwrap();
                    let CssContributions::Longhands(values) =
                        handle.reenter(replacement.clone()).unwrap()
                    else {
                        panic!("ordinary replacement")
                    };
                    assert_eq!(values.items().len(), terminals(name).len());
                    let direct = contributions(
                        &checked_components(name, replacement.clone(), true).unwrap(),
                    );
                    for ((item, expected), terminal) in values
                        .items()
                        .iter()
                        .zip(direct.items())
                        .zip(terminals(name))
                    {
                        assert_eq!(item.property(), property(terminal));
                        assert_source(item, &source);
                        assert_eq!(item.ordinary_value(), expected.ordinary_value());
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
                                panic!("parsed replacement")
                            };
                            assert!(actual.source().same_snapshot(supplied.source()));
                            assert_eq!(actual.source().as_str(), *text);
                        }
                    }
                }
                for (text, keyword) in GLOBALS {
                    let replacement = parse_component_values(text).unwrap();
                    let CssContributions::Longhands(values) =
                        handle.reenter(replacement.clone()).unwrap()
                    else {
                        panic!("global replacement")
                    };
                    assert_eq!(values.items().len(), terminals(name).len());
                    for (item, terminal) in values.items().iter().zip(terminals(name)) {
                        assert_eq!(item.property(), property(terminal));
                        assert_source(item, &source);
                        assert!(
                            matches!(item.value(),CssContributionValueRef::Global(v) if v==keyword)
                        );
                        assert_eq!(item.replacement_components(), Some(&replacement));
                    }
                }
                assert_eq!(source, before);
            }
        }
    }
}
#[test]
fn replacement_residual_priority_and_invalid_failures_leave_each_pending_handle_reusable() {
    for name in FAMILY {
        for pending in ["var(--text)", "env(text)", "attr(data-text *)"] {
            let source = checked(name, pending, false);
            let before = source.clone();
            let CssExpansion::Pending(handle) = expand_declaration(&source).unwrap() else {
                panic!("pending")
            };
            for text in [
                "var(--again)",
                "env(again)",
                "attr(data-again *)",
                r"\76 ar(--again)",
                "var(--again",
                "env(again",
                "attr(data-again *",
                "var(--again)/*",
                "unknown var(--again",
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
            for text in [
                "unknown",
                "initial inherit",
                "unknown!important",
                "unknown;color:red",
            ] {
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
            let replacement = parse_component_values(ordinary_cases(name)[0]).unwrap();
            let CssContributions::Longhands(values) = handle.reenter(replacement.clone()).unwrap()
            else {
                panic!("successful retry")
            };
            assert_eq!(values.items().len(), terminals(name).len());
            for item in values.items() {
                assert_source(item, &source);
                assert_eq!(item.replacement_components(), Some(&replacement));
            }
            assert!(handle.source().same_occurrence(&source));
            assert_eq!(source, before);
        }
    }
}
#[test]
fn both_checked_fronts_reject_original_eof_quote_comment_and_function_closures() {
    for name in FAMILY {
        let mut texts: Vec<String> = ordinary_cases(name)
            .iter()
            .map(|s| format!("/*😀*/{s}/*"))
            .collect();
        texts.extend(GLOBALS.map(|(s, _)| format!("/*😀*/{s}/*")));
        texts.extend(
            [
                "/*😀*/\"unterminated",
                "/*😀*/var(--text",
                "/*😀*/env(text",
                "/*😀*/attr(data-text *",
                "/*😀*/var(--text)/*",
                "/*😀*/env(text)/*",
                "/*😀*/attr(data-text *)/*",
            ]
            .map(str::to_owned),
        );
        if name == "line-padding" {
            texts.push("/*😀*/calc(1px".to_owned());
        }
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
fn strict_reentry_retains_original_eof_origins_and_complete_comment_controls_succeed() {
    for name in FAMILY {
        for pending in ["var(--text)", "env(text)", "attr(data-text *)"] {
            let source = checked(name, pending, true);
            let CssExpansion::Pending(handle) = expand_declaration(&source).unwrap() else {
                panic!("pending")
            };
            let mut texts: Vec<String> = ordinary_cases(name)
                .iter()
                .map(|s| format!("/*😀*/{s}/*"))
                .collect();
            texts.extend(GLOBALS.map(|(s, _)| format!("/*😀*/{s}/*")));
            texts.push("/*😀*/\"unterminated".to_owned());
            if name == "line-padding" {
                texts.push("/*😀*/calc(1px".to_owned());
            }
            for text in texts {
                let components = parse_component_values(&text).unwrap();
                let origin = implicit_origin(&components);
                for _ in 0..2 {
                    let error = handle.reenter(components.clone()).unwrap_err();
                    let CssExpansionErrorKind::InvalidReplacement(error) = error.kind() else {
                        panic!("strict original closure")
                    };
                    assert_closure(error, &origin, &text);
                }
            }
            for text in [ordinary_cases(name)[0], "initial", pending] {
                for grammar in [false, true] {
                    checked(name, &format!("{text}/**/"), grammar);
                }
            }
            assert!(
                handle
                    .reenter(
                        parse_component_values(&format!("{}/**/", ordinary_cases(name)[0]))
                            .unwrap()
                    )
                    .is_ok()
            );
        }
    }
}
#[test]
fn checked_annotations_fail_at_original_utf8_token_origin_while_authored_importance_is_retained() {
    for name in FAMILY {
        let value = ordinary_cases(name)[0];
        let text = format!("/*😀*/{value}!important");
        let components = parse_component_values(&text).unwrap();
        let serialized = components.serialize().unwrap();
        let origin = serialized
            .origin_at(serialized.as_css().find('!').unwrap())
            .unwrap();
        for grammar in [false, true] {
            let error = checked_components(name, components.clone(), grammar).unwrap_err();
            assert!(matches!(
                error.kind(),
                CssPropertyValueErrorKind::Grammar(_)
            ));
            assert_eq!(error.origin(), origin);
        }
        canonical(&parsed(name, value), name, value);
    }
}
#[test]
fn intrinsic_initials_and_inheritance_equal_authored_terminal_values_without_fabricated_sources() {
    for (name, text, inherited) in INITIALS {
        let p = property(name);
        assert_eq!(P::from_name(&name.to_ascii_uppercase()), Some(p));
        let CssPropertyKindRef::Longhand(meta) = p.metadata().unwrap().kind() else {
            panic!("intrinsic longhand")
        };
        assert_eq!(meta.property().known_property(), p);
        assert_eq!(meta.inherited_by_default(), inherited);
        let initial = meta.initial_value();
        let CssInitialValueRef::Value(value) = initial.view() else {
            panic!("ordinary initial")
        };
        assert_eq!(value.property().known_property(), p);
        let declaration = checked(name, text, true);
        let values = contributions(&declaration);
        assert_eq!(values.items()[0].ordinary_value(), Some(value));
    }
}
#[test]
fn normalization_counts_complete_terminals_and_pending_occurrences_cumulatively() {
    for name in FAMILY {
        let css = format!(
            "@media screen{{.a{{{name}:{}!important;{name}:var(--text);{name}:unset!important;color:red}}}}",
            ordinary_cases(name)[0]
        );
        let report = parse_sheet(&css);
        assert!(report.is_clean(), "{css}: {:?}", report.diagnostics());
        let before = report.clone();
        let count = terminals(name).len() * 2 + 2;
        let exact = CssNormalizationLimits::try_new(1, 2, 4, count).unwrap();
        let normalized = normalize_sheet_with_limits(report.syntax(), exact).unwrap();
        let declarations: Vec<_> = normalized
            .items()
            .iter()
            .filter_map(|v| match v {
                CssNormalizedItem::Declaration(v) => Some(v),
                _ => None,
            })
            .collect();
        assert_eq!(declarations.len(), 4);
        for (index, item) in declarations.iter().enumerate() {
            assert_eq!(item.order(), index);
            assert_eq!(
                item.source().known().unwrap().property(),
                if index == 3 { P::Color } else { property(name) }
            );
            assert_eq!(
                item.source().importance(),
                if index == 0 || index == 2 {
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
                    assert_eq!(index, 1);
                    assert!(handle.source().same_occurrence(item.source()));
                }
                CssExpansion::Contributions(CssContributions::Longhands(values)) => {
                    assert_eq!(
                        values.items().len(),
                        if index == 3 { 1 } else { terminals(name).len() }
                    );
                    for value in values.items() {
                        assert_source(value, item.source());
                        if index == 2 {
                            assert!(matches!(
                                value.value(),
                                CssContributionValueRef::Global(CssGlobalKeyword::Unset)
                            ));
                        } else {
                            assert!(matches!(
                                value.value(),
                                CssContributionValueRef::Ordinary(_)
                            ));
                        }
                    }
                }
                _ => panic!("intrinsic authored lifecycle"),
            }
        }
        for (limits, resource, limit) in [
            (
                CssNormalizationLimits::try_new(1, 2, 4, count - 1).unwrap(),
                CssNormalizationResource::Contributions,
                count - 1,
            ),
            (
                CssNormalizationLimits::try_new(1, 2, 3, count).unwrap(),
                CssNormalizationResource::Declarations,
                3,
            ),
        ] {
            for _ in 0..2 {
                let error = normalize_sheet_with_limits(report.syntax(), limits).unwrap_err();
                assert_eq!(
                    error.kind(),
                    &CssNormalizationErrorKind::LimitExceeded { resource, limit }
                );
                assert_eq!(error.declaration_order(), Some(3));
                assert!(
                    error
                        .declaration()
                        .unwrap()
                        .same_occurrence(declarations[3].source())
                );
            }
        }
        assert_eq!(report, before);
        assert!(normalize_sheet_with_limits(report.syntax(), exact).is_ok());
    }
}
#[test]
fn browser_eof_recovery_normalizes_original_occurrences_and_preserves_ordered_diagnostics() {
    for name in FAMILY {
        let mut texts = vec![
            format!("{}/*", ordinary_cases(name)[0]),
            "var(--text".to_owned(),
            "env(text".to_owned(),
            "attr(data-text *".to_owned(),
        ];
        texts.extend(GLOBALS.map(|(s, _)| format!("{s}/*")));
        for text in texts {
            let css = format!("/*😀*/.a{{{name}:{text}");
            let report = parse_sheet(&css);
            assert!(!report.is_clean());
            assert!(validate_sheet(&css).is_err());
            assert!(
                report
                    .diagnostics()
                    .iter()
                    .any(|d| d.action() == CssRecoveryAction::RetainWithImplicitClosure)
            );
            let before = report.clone();
            let count = if text.starts_with("var(")
                || text.starts_with("env(")
                || text.starts_with("attr(")
            {
                1
            } else {
                terminals(name).len()
            };
            let exact = CssNormalizationLimits::try_new(0, 1, 1, count).unwrap();
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
                panic!("one recovered occurrence")
            };
            assert_eq!(item.order(), 0);
            assert_eq!(item.source().known().unwrap().property(), property(name));
            assert_eq!(item.source().parsed_name().unwrap().source().as_str(), css);
            match item.expansion() {
                CssExpansion::Pending(handle) => {
                    assert!(handle.source().same_occurrence(item.source()))
                }
                CssExpansion::Contributions(CssContributions::Longhands(values)) => {
                    assert_eq!(values.items().len(), count);
                    for value in values.items() {
                        assert_source(value, item.source());
                    }
                }
                _ => panic!("retained intrinsic lifecycle"),
            }
            let error = normalize_report_with_limits(
                &report,
                CssNormalizationLimits::try_new(0, 1, 1, count - 1).unwrap(),
            )
            .unwrap_err();
            assert_eq!(
                error.kind(),
                &CssNormalizationErrorKind::LimitExceeded {
                    resource: CssNormalizationResource::Contributions,
                    limit: count - 1
                }
            );
            assert_eq!(error.declaration_order(), Some(0));
            assert!(error.declaration().unwrap().same_occurrence(item.source()));
            assert_eq!(report, before);
            assert!(normalize_report_with_limits(&report, exact).is_ok());
        }
    }
}
fn declaration_limits(name: &str, text: &str, nodes: usize) {
    use CssSpecifiedValueSerializationErrorKind as Kind;
    use CssSpecifiedValueSerializationLimits as Limits;
    for source in [parsed(name, text), checked(name, text, true)] {
        let expected = format!("{name}: {text} !important;");
        let before = source.clone();
        let exact = Limits::new(nodes + 2, nodes + 2, expected.len());
        assert_eq!(
            source.to_specified_css_with_limits(exact).unwrap(),
            expected
        );
        for (limits, kind) in [
            (
                Limits::new(nodes + 1, nodes + 2, expected.len()),
                Kind::InputNodeLimit,
            ),
            (
                Limits::new(nodes + 2, nodes + 1, expected.len()),
                Kind::ProjectionNodeLimit,
            ),
            (
                Limits::new(nodes + 2, nodes + 2, expected.len() - 1),
                Kind::ByteLimit,
            ),
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
#[test]
fn every_emitted_keyword_and_literal_charges_exact_declaration_work_and_utf8_bytes_atomically() {
    for name in FAMILY {
        for text in budget_cases(name) {
            declaration_limits(name, &text, text.split_whitespace().count());
        }
        for (text, _) in GLOBALS {
            declaration_limits(name, text, 1);
        }
    }
}
fn sheet_limits(input: &str, expected: &str, nodes: usize) {
    use CssSpecifiedValueSerializationErrorKind as Kind;
    use CssSpecifiedValueSerializationLimits as Limits;
    let report = parse_sheet(input);
    assert!(report.is_clean(), "{input}: {:?}", report.diagnostics());
    let sheet = report.syntax();
    let before = sheet.clone();
    let exact = Limits::new(nodes, nodes, expected.len());
    assert_eq!(sheet.to_specified_css_with_limits(exact).unwrap(), expected);
    for (limits, kind) in [
        (
            Limits::new(nodes - 1, nodes, expected.len()),
            Kind::InputNodeLimit,
        ),
        (
            Limits::new(nodes, nodes - 1, expected.len()),
            Kind::ProjectionNodeLimit,
        ),
        (
            Limits::new(nodes, nodes, expected.len() - 1),
            Kind::ByteLimit,
        ),
    ] {
        // Each rule fits separately; exhaustion only comes from cumulative use.
        assert!(
            sheet
                .rules()
                .iter()
                .all(|rule| rule.to_specified_css_with_limits(limits).is_ok())
        );
        for _ in 0..2 {
            let error = sheet.to_specified_css_with_limits(limits).unwrap_err();
            assert_eq!(
                error.kind(),
                CssSpecifiedRuleSerializationErrorKind::Resource(kind)
            );
            assert_eq!(error.rule_index(), Some(1));
            assert_eq!(sheet, &before);
        }
    }
    assert_eq!(sheet.to_specified_css_with_limits(exact).unwrap(), expected);
}

const FAMILY: [&str; 2] = ["text-justify", "text-group-align"];
const BASES: [&str; 6] = [
    "auto",
    "none",
    "inter-word",
    "inter-character",
    "ruby",
    "distribute",
];
const GROUP: [&str; 6] = ["none", "start", "end", "left", "right", "center"];
const INITIALS: [(&str, &str, bool); 2] = [
    ("text-justify", "auto", true),
    ("text-group-align", "none", false),
];
fn terminals(name: &str) -> &'static [&'static str] {
    match name {
        "text-justify" => &["text-justify"],
        "text-group-align" => &["text-group-align"],
        _ => panic!("family"),
    }
}
fn ordinary_cases(name: &str) -> &'static [&'static str] {
    match name {
        "text-justify" => &[
            "auto",
            "none",
            "inter-word",
            "inter-character",
            "ruby",
            "distribute",
            "no-compress",
            "ruby no-compress",
            "distribute no-compress",
        ],
        "text-group-align" => &GROUP,
        _ => panic!("family"),
    }
}
fn budget_cases(name: &str) -> Vec<String> {
    match name {
        "text-justify" => BASES
            .into_iter()
            .flat_map(|base| [base.to_owned(), format!("{base} no-compress")])
            .chain(["no-compress".to_owned()])
            .collect(),
        "text-group-align" => GROUP.into_iter().map(str::to_owned).collect(),
        _ => panic!("family"),
    }
}
#[test]
fn all_justify_bases_and_modifier_orders_preserve_base_omission_and_authored_distribute() {
    accepted("text-justify", "no-compress", "no-compress");
    for base in BASES {
        accepted("text-justify", base, base);
        let canonical = format!("{base} no-compress");
        for text in permutations(&[base, "no-compress"]) {
            accepted("text-justify", &text, &canonical);
        }
    }
    // The chosen specified phase retains distribute; computed inter-character
    // equivalence belongs to the downstream computation owner.
    let distribute = contributions(&checked("text-justify", "distribute", true));
    let inter_character = contributions(&checked("text-justify", "inter-character", true));
    assert_ne!(
        distribute.items()[0].ordinary_value(),
        inter_character.items()[0].ordinary_value()
    );
}
#[test]
fn group_alignment_admits_all_six_complete_keywords() {
    for text in GROUP {
        accepted("text-group-align", text, text);
    }
}
#[test]
fn duplicate_bases_modifiers_and_group_keywords_reject_with_atomic_sibling_recovery() {
    for name in FAMILY {
        for text in [
            "",
            "normal",
            "1",
            "-1px",
            "25%",
            "calc(1 + 2)",
            "min(1,2)",
            "[auto]",
            "\"auto\"",
            "auto,none",
        ] {
            invalid(name, text);
        }
    }
    for left in BASES {
        for right in BASES {
            invalid("text-justify", &format!("{left} {right}"));
            invalid("text-justify", &format!("{left} no-compress {right}"));
        }
        invalid("text-justify", &format!("no-compress {left} no-compress"));
    }
    invalid("text-justify", "no-compress no-compress");
    for left in GROUP {
        for right in GROUP {
            invalid("text-group-align", &format!("{left} {right}"));
        }
    }
    for text in ["auto", "ruby", "no-compress", "distribute"] {
        invalid("text-group-align", text);
    }
}
#[test]
fn escaped_case_and_comment_keywords_keep_original_sources_and_canonical_roles() {
    for (name, input, expected) in [
        (
            "text-justify",
            "NO-COMPRESS/**/INTER-WORD",
            "inter-word no-compress",
        ),
        (
            "text-justify",
            r"\64 istribute \6e o-compress",
            "distribute no-compress",
        ),
        ("text-justify", r"\6e o-compress", "no-compress"),
        ("text-group-align", r"\63 enter", "center"),
        ("text-group-align", "LEFT/**/", "left"),
    ] {
        accepted(name, input, expected);
    }
}
#[test]
fn each_single_terminal_isolated_from_group_alignment_and_existing_spacing_siblings() {
    let report = parse_style_attribute(
        "text-justify:no-compress!important;text-group-align:center;letter-spacing:2px;word-spacing:3px;color:red",
    );
    assert!(report.is_clean());
    let names = [
        "text-justify",
        "text-group-align",
        "letter-spacing",
        "word-spacing",
        "color",
    ];
    for (source, name) in report.syntax().iter().zip(names) {
        let CssExpansion::Contributions(CssContributions::Longhands(values)) =
            expand_declaration(source).unwrap()
        else {
            panic!("intrinsic single terminal")
        };
        assert_eq!(values.items().len(), 1);
        assert_eq!(values.items()[0].property(), property(name));
        assert_source(&values.items()[0], source);
    }
}
#[test]
fn sheet_siblings_share_exact_work_and_non_ascii_final_bytes() {
    // Published rule tariff: sheet 1; each class rule 4 structural nodes;
    // declaration/name 2; two and one explicit keyword leaves respectively.
    sheet_limits(
        ".é{text-justify:no-compress ruby!important}.b{text-group-align:center}",
        ".é { text-justify: ruby no-compress !important; }\n.b { text-group-align: center; }",
        16,
    );
}

#[test]
fn substitution_inside_authored_tokens_is_one_whole_pending_occurrence() {
    for name in FAMILY {
        for call in ["var(--text)", "env(text)", "attr(data-text *)"] {
            let input = format!("{} {call}", ordinary_cases(name)[0]);
            for source in fronts(name, &input) {
                let before = source.clone();
                assert!(source.known().unwrap().substitution_dependent().is_some());
                assert!(source.known().unwrap().property_value().is_none());
                canonical(&source, name, &input);
                let CssExpansion::Pending(handle) = expand_declaration(&source).unwrap() else {
                    panic!("whole-value pending")
                };
                assert!(handle.source().same_occurrence(&source));
                let replacement = parse_component_values(ordinary_cases(name)[0]).unwrap();
                let CssContributions::Longhands(values) =
                    handle.reenter(replacement.clone()).unwrap()
                else {
                    panic!("whole replacement")
                };
                assert_eq!(values.items().len(), terminals(name).len());
                for item in values.items() {
                    assert_source(item, &source);
                    assert_eq!(item.replacement_components(), Some(&replacement));
                }
                assert_eq!(source, before);
            }
        }
    }
}
#[test]
fn family_grammar_invalid_replacements_fail_twice_then_a_valid_replacement_succeeds() {
    for name in FAMILY {
        let source = checked(name, "var(--text)", true);
        let CssExpansion::Pending(handle) = expand_declaration(&source).unwrap() else {
            panic!("pending")
        };
        for text in invalid_replacements(name) {
            for _ in 0..2 {
                let error = handle
                    .reenter(parse_component_values(text).unwrap())
                    .unwrap_err();
                let CssExpansionErrorKind::InvalidReplacement(error) = error.kind() else {
                    panic!("family replacement error")
                };
                assert!(matches!(
                    error.kind(),
                    CssPropertyValueErrorKind::Grammar(_)
                ));
            }
        }
        let replacement = parse_component_values(ordinary_cases(name)[0]).unwrap();
        let CssContributions::Longhands(values) = handle.reenter(replacement.clone()).unwrap()
        else {
            panic!("successful reuse")
        };
        assert_eq!(values.items().len(), terminals(name).len());
        for item in values.items() {
            assert_source(item, &source);
            assert_eq!(item.replacement_components(), Some(&replacement));
        }
    }
}

fn invalid_replacements(name: &str) -> &'static [&'static str] {
    match name {
        "text-justify" => &[
            "auto inter-word",
            "ruby ruby",
            "no-compress no-compress",
            "none ruby",
            "distribute inter-character",
        ],
        "text-group-align" => &["none center", "left right", "auto", "ruby"],
        _ => panic!("family"),
    }
}
