#![forbid(unsafe_code)]
//! Composed authored Text value conventions through existing public APIs.
//!
//! Text4 WD20260814 §1.2 / Text3 CRD20260814 §1.2 import value syntax/types
//! and CSS-wide values; Values3/4 §2.2 requires whole-value globals and preserves
//! nonassociative grouping. Each witness below comes from its property grammar,
//! not from current serializer output. Existing owner suites retain exhaustive
//! permutations, typed constructors, initials and numeric precision/resource tests.
//! Shared Values4 §10.12 leaves calculation ranges and integer rounding deferred.
//! Product contracts retain strict original closure, residual-first reentry,
//! original occurrence/importance, replacement origins and intrinsic terminals.
//!
//! Runtime lookup avoids absent planned enum/borrowed variants. Six individually
//! addressable prior_owner_* groups isolate the older owner's original-closure
//! gap from the thirteen active properties.
//! It establishes no source-count, issue/status, acceptance or execution oracle.

use surgeist_css::*;

struct Witness {
    name: &'static str,
    authored: &'static str,
    canonical: &'static str,
    invalid: &'static str,
}

// One distinguishing witness per selected property. Full owner suites remain
// authoritative for exhaustive domains and construction, rather than duplicating
// their finite permutation machinery here.
const WITNESSES: &[Witness] = &[
    Witness {
        name: "text-transform",
        authored: "full-size-kana uppercase",
        canonical: "uppercase full-size-kana",
        invalid: "uppercase lowercase",
    },
    Witness {
        name: "word-space-transform",
        authored: "auto-phrase ideographic-space",
        canonical: "ideographic-space auto-phrase",
        invalid: "auto-phrase",
    },
    Witness {
        name: "white-space",
        authored: "discard-after discard-before nowrap preserve",
        canonical: "preserve nowrap discard-before discard-after",
        invalid: "discard-before nowrap discard-after",
    },
    Witness {
        name: "white-space-collapse",
        authored: "discard",
        canonical: "discard",
        invalid: "normal",
    },
    Witness {
        name: "white-space-trim",
        authored: "discard-inner discard-before",
        canonical: "discard-before discard-inner",
        invalid: "none discard-before",
    },
    Witness {
        name: "tab-size",
        authored: "calc(-1)",
        canonical: "calc(-1)",
        invalid: "-1e-999",
    },
    Witness {
        name: "text-wrap-mode",
        authored: "nowrap",
        canonical: "nowrap",
        invalid: "balance",
    },
    Witness {
        name: "wrap-inside",
        authored: "avoid",
        canonical: "avoid",
        invalid: "avoid-line",
    },
    Witness {
        name: "wrap-before",
        authored: "avoid-flex",
        canonical: "avoid-flex",
        invalid: "nowrap",
    },
    Witness {
        name: "wrap-after",
        authored: "line",
        canonical: "line",
        invalid: "anywhere",
    },
    Witness {
        name: "text-wrap-style",
        authored: "avoid-short-last-line",
        canonical: "avoid-short-last-line",
        invalid: "nowrap",
    },
    Witness {
        name: "text-wrap",
        authored: "balance nowrap",
        canonical: "nowrap balance",
        invalid: "wrap nowrap",
    },
    Witness {
        name: "word-break",
        authored: "auto-phrase",
        canonical: "auto-phrase",
        invalid: "anywhere",
    },
    Witness {
        name: "line-break",
        authored: "anywhere",
        canonical: "anywhere",
        invalid: "break-all",
    },
    Witness {
        name: "hyphens",
        authored: "manual",
        canonical: "manual",
        invalid: "normal",
    },
    Witness {
        name: "hyphenate-character",
        authored: "'inherit'",
        canonical: "\"inherit\"",
        invalid: "none",
    },
    Witness {
        name: "hyphenate-limit-zone",
        authored: "-2px",
        canonical: "-2px",
        invalid: "2",
    },
    Witness {
        name: "hyphenate-limit-chars",
        authored: "+0002 auto",
        canonical: "2 auto",
        invalid: "-1 auto",
    },
    Witness {
        name: "hyphenate-limit-lines",
        authored: "calc(1.5)",
        canonical: "calc(1.5)",
        invalid: "1.5",
    },
    Witness {
        name: "hyphenate-limit-last",
        authored: "spread",
        canonical: "spread",
        invalid: "manual",
    },
    Witness {
        name: "overflow-wrap",
        authored: "anywhere",
        canonical: "anywhere",
        invalid: "break-all",
    },
    Witness {
        name: "text-align",
        authored: "justify-all",
        canonical: "justify-all",
        invalid: "auto",
    },
    Witness {
        name: "text-align-all",
        authored: "\".\"",
        canonical: "\".\"",
        invalid: "\"ab\"",
    },
    Witness {
        name: "text-align-last",
        authored: "auto",
        canonical: "auto",
        invalid: "\".\"",
    },
    Witness {
        name: "text-justify",
        authored: "no-compress distribute",
        canonical: "distribute no-compress",
        invalid: "auto ruby",
    },
    Witness {
        name: "text-group-align",
        authored: "center",
        canonical: "center",
        invalid: "justify",
    },
    Witness {
        name: "word-spacing",
        authored: "-25%",
        canonical: "-25%",
        invalid: "auto",
    },
    Witness {
        name: "letter-spacing",
        authored: "calc(-2px - 3%)",
        canonical: "calc(-3% - 2px)",
        invalid: "auto",
    },
    Witness {
        name: "line-padding",
        authored: "-2px",
        canonical: "-2px",
        invalid: "2%",
    },
    Witness {
        name: "text-autospace",
        authored: "replace punctuation ideograph-alpha",
        canonical: "ideograph-alpha punctuation replace",
        invalid: "ideograph-alpha insert punctuation",
    },
    Witness {
        name: "text-spacing-trim",
        authored: "auto",
        canonical: "auto",
        invalid: "no-autospace",
    },
    Witness {
        name: "text-spacing",
        authored: "ideograph-numeric insert trim-start",
        canonical: "trim-start ideograph-numeric insert",
        invalid: "ideograph-alpha trim-start punctuation",
    },
    Witness {
        name: "text-indent",
        authored: "each-line -2px hanging",
        canonical: "-2px hanging each-line",
        invalid: "hanging",
    },
    Witness {
        name: "hanging-punctuation",
        authored: "last allow-end first",
        canonical: "first allow-end last",
        invalid: "force-end allow-end",
    },
];

const GLOBALS: [(&str, CssGlobalKeyword); 5] = [
    ("initial", CssGlobalKeyword::Initial),
    ("inherit", CssGlobalKeyword::Inherit),
    ("unset", CssGlobalKeyword::Unset),
    ("revert", CssGlobalKeyword::Revert),
    ("revert-layer", CssGlobalKeyword::RevertLayer),
];

fn property(name: &str) -> CssKnownProperty {
    CssKnownProperty::from_name(name)
        .unwrap_or_else(|| panic!("selected property unavailable: {name}"))
}

fn witness(name: &str) -> &'static Witness {
    WITNESSES
        .iter()
        .find(|w| w.name == name)
        .expect("independent witness")
}

fn members(name: &str) -> Vec<&str> {
    match name {
        "text-align" => vec!["text-align-all", "text-align-last"],
        "text-wrap" => vec!["text-wrap-mode", "text-wrap-style"],
        "white-space" => vec!["white-space-collapse", "text-wrap-mode", "white-space-trim"],
        "text-spacing" => vec!["text-spacing-trim", "text-autospace"],
        _ => vec![name],
    }
}

fn parsed(name: &str, value: &str) -> CssDeclaration {
    let css = format!("/*😀*/{name}:{value}!important");
    let report = parse_style_attribute(&css);
    assert!(report.is_clean(), "{css}: {:?}", report.diagnostics());
    assert!(validate_style_attribute(&css).is_ok());
    let [source] = report.syntax().as_slice() else {
        panic!("one authored occurrence")
    };
    assert_eq!(source.known().unwrap().grammar(), property(name).grammar());
    assert_eq!(source.importance(), CssImportance::Important);
    assert_eq!(source.parsed_name().unwrap().source().as_str(), css);
    source.clone()
}

fn checked_components(
    name: &str,
    components: CssComponentValues,
    grammar: bool,
) -> Result<CssDeclaration, CssPropertyValueParseError> {
    let p = property(name);
    if grammar {
        parse_property_value_for_grammar(p.grammar(), components, CssImportance::Important)
    } else {
        parse_property_value(
            CssPropertyNameRef::Known(p),
            components,
            CssImportance::Important,
        )
    }
}

fn checked(name: &str, value: &str, grammar: bool) -> CssDeclaration {
    let components = parse_component_values(value).unwrap();
    let before = components.clone();
    let source = checked_components(name, components.clone(), grammar).unwrap();
    assert_eq!(components, before);
    assert_eq!(source.value_components(), &components);
    assert_eq!(source.importance(), CssImportance::Important);
    assert!(source.position().is_none());
    assert!(source.parsed_name().is_none());
    assert!(source.parsed_value().is_none());
    source
}

fn expanded(source: &CssDeclaration) -> CssLonghandContributions {
    let CssExpansion::Contributions(CssContributions::Longhands(values)) =
        expand_declaration(source).unwrap()
    else {
        panic!("completed intrinsic terminals")
    };
    assert_terminals(&values, source);
    values
}

fn assert_terminals(values: &CssLonghandContributions, source: &CssDeclaration) {
    let name = source.known().unwrap().property().canonical_name();
    assert_eq!(
        values
            .items()
            .iter()
            .map(|v| v.property().canonical_name())
            .collect::<Vec<_>>(),
        members(name)
    );
    for item in values.items() {
        assert!(item.source().same_occurrence(source));
        assert_eq!(item.source().importance(), source.importance());
        assert_eq!(item.source().value_components(), source.value_components());
        assert_eq!(item.source().parsed_name(), source.parsed_name());
        assert_eq!(item.source().parsed_value(), source.parsed_value());
    }
}

fn invalid(name: &str, value: &str) {
    let css = format!("color:red;{name}:{value};color:blue");
    let report = parse_style_attribute(&css);
    let [diagnostic] = report.diagnostics() else {
        panic!("one property grammar failure: {css}")
    };
    assert_eq!(diagnostic.action(), CssRecoveryAction::DropDeclaration);
    let ErrorKind::InvalidPropertyValue(detail) = diagnostic.error().kind() else {
        panic!("registered property grammar failure")
    };
    assert_eq!(detail.property(), property(name));
    assert_eq!(
        report
            .syntax()
            .iter()
            .map(|v| v.known().unwrap().property())
            .collect::<Vec<_>>(),
        [CssKnownProperty::Color, CssKnownProperty::Color]
    );
    assert_eq!(
        validate_style_attribute(&css).unwrap_err().diagnostics(),
        report.diagnostics()
    );
    let components = parse_component_values(value).unwrap();
    let before = components.clone();
    for grammar in [false, true] {
        assert!(
            matches!(
                checked_components(name, components.clone(), grammar)
                    .unwrap_err()
                    .kind(),
                CssPropertyValueErrorKind::Grammar(_)
            ),
            "{name}:{value}"
        );
    }
    assert_eq!(components, before);
}

#[test]
fn property_specific_witnesses_compose_providers_and_intrinsic_terminal_shapes() {
    for w in WITNESSES {
        for source in [
            parsed(w.name, w.authored),
            checked(w.name, w.authored, false),
            checked(w.name, w.authored, true),
        ] {
            let before = source.clone();
            assert_eq!(
                source.to_specified_css().unwrap(),
                format!("{}: {} !important;", w.name, w.canonical)
            );
            let values = expanded(&source);
            for item in values.items() {
                assert!(matches!(item.value(), CssContributionValueRef::Ordinary(_)));
                assert_eq!(
                    item.ordinary_value().unwrap().property().known_property(),
                    item.property()
                );
                assert!(item.replacement_components().is_none());
            }
            match property(w.name).metadata().unwrap().kind() {
                CssPropertyKindRef::Longhand(value) => {
                    assert_eq!(value.property().known_property(), property(w.name))
                }
                CssPropertyKindRef::Shorthand(value) => {
                    assert_eq!(
                        value
                            .settable_members()
                            .iter()
                            .map(|v| v.known_property().canonical_name())
                            .collect::<Vec<_>>(),
                        members(w.name)
                    );
                    assert!(value.reset_only_members().is_empty());
                }
                _ => panic!("ordinary longhand or bounded Text shorthand"),
            }
            assert_eq!(source, before);
        }
        invalid(w.name, w.invalid);
    }
}

#[test]
fn imported_numeric_domains_keep_signed_lengths_but_distinguish_literal_ranges_and_math() {
    for (name, input, canonical) in [
        ("word-spacing", "-1e-999%", "0%"),
        ("letter-spacing", "-2px", "-2px"),
        ("text-indent", "-25%", "-25%"),
        ("hyphenate-limit-zone", "-25%", "-25%"),
        ("line-padding", "calc(-2px)", "calc(-2px)"),
        ("tab-size", "0", "0"),
        ("tab-size", "0px", "0px"),
        ("tab-size", "calc(-1px)", "calc(-1px)"),
        ("hyphenate-limit-lines", "-0", "0"),
        ("hyphenate-limit-lines", "calc(-1)", "calc(-1)"),
        ("hyphenate-limit-chars", "calc(1.5) auto", "calc(1.5) auto"),
    ] {
        let source = checked(name, input, true);
        let before = source.value_components().clone();
        assert_eq!(
            source.to_specified_css().unwrap(),
            format!("{name}: {canonical} !important;")
        );
        assert_eq!(source.value_components(), &before);
    }
    for (name, value) in [
        ("tab-size", "-1e-999px"),
        ("tab-size", "1%"),
        ("tab-size", "calc(0 + 1px)"),
        ("hyphenate-limit-chars", "1e2"),
        ("hyphenate-limit-lines", "-1"),
        ("hyphenate-limit-lines", "1e2"),
        ("line-padding", "calc(1px + 1%)"),
    ] {
        invalid(name, value);
    }
}

#[test]
fn imported_strings_keep_property_specific_empty_and_grapheme_constraints() {
    assert_eq!(
        checked("hyphenate-character", "\"\"", true)
            .to_specified_css()
            .unwrap(),
        "hyphenate-character: \"\" !important;"
    );
    let alignment = checked("text-align-all", "\"\\65\\301 \"", true);
    assert_eq!(
        alignment.to_specified_css().unwrap(),
        "text-align-all: \"é\" !important;"
    );
    invalid("text-align-all", "\"\"");
    invalid("text-align-last", "\"é\"");
}

#[test]
fn all_globals_are_whole_values_even_when_property_tables_omit_them() {
    for w in WITNESSES {
        for (text, keyword) in GLOBALS {
            for source in [
                parsed(w.name, text),
                checked(w.name, text, false),
                checked(w.name, text, true),
            ] {
                assert_eq!(source.known().unwrap().global(), Some(keyword));
                assert_eq!(
                    source.to_specified_css().unwrap(),
                    format!("{}: {text} !important;", w.name)
                );
                for item in expanded(&source).items() {
                    assert!(
                        matches!(item.value(), CssContributionValueRef::Global(v) if v == keyword)
                    );
                    assert!(item.replacement_components().is_none());
                }
            }
            invalid(w.name, &format!("{text} {}", w.authored));
            invalid(w.name, &format!("{} {text}", w.authored));
        }
    }
    // A quoted CSS-wide spelling remains an ordinary string in its own domain.
    assert!(
        parsed("hyphenate-character", "'inherit'")
            .known()
            .unwrap()
            .property_value()
            .is_some()
    );
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
        .expect("retained implicit closure")
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
        panic!("original EOF origin")
    };
    assert_eq!(opening.source().as_str(), text);
    assert!(opening.source().same_snapshot(at.source()));
    assert_eq!(at.span().start().byte_offset().value(), text.len());
    assert_eq!(at.span().end().byte_offset().value(), text.len());
}

fn strict_original(name: &str) {
    let w = witness(name);
    let mut values = vec![
        format!("{}/*", w.authored),
        "var(--text)/*".into(),
        "var(--text".into(),
    ];
    values.extend(GLOBALS.map(|(text, _)| format!("{text}/*")));
    for text in values {
        let components = parse_component_values(&text).unwrap();
        let before = components.clone();
        let origin = implicit_origin(&components);
        for grammar in [false, true] {
            let error = checked_components(name, components.clone(), grammar).unwrap_err();
            assert_closure(&error, &origin, &text);
        }
        assert_eq!(components, before);
    }
    for text in [
        format!("{}/**/", w.authored),
        "initial/**/".into(),
        "var(--text)/**/".into(),
    ] {
        checked(name, &text, false);
        checked(name, &text, true);
    }
}

#[test]
fn prior_owner_text_align_rejects_original_closure_at_both_checked_fronts() {
    strict_original("text-align");
}
#[test]
fn prior_owner_text_align_all_rejects_original_closure_at_both_checked_fronts() {
    strict_original("text-align-all");
}
#[test]
fn prior_owner_text_align_last_rejects_original_closure_at_both_checked_fronts() {
    strict_original("text-align-last");
}
#[test]
fn prior_owner_overflow_wrap_rejects_original_closure_at_both_checked_fronts() {
    strict_original("overflow-wrap");
}
#[test]
fn prior_owner_word_spacing_rejects_original_closure_at_both_checked_fronts() {
    strict_original("word-spacing");
}
#[test]
fn prior_owner_letter_spacing_rejects_original_closure_at_both_checked_fronts() {
    strict_original("letter-spacing");
}

#[test]
fn original_closure_policy_composes_across_every_selected_text_property() {
    for w in WITNESSES {
        strict_original(w.name);
    }
}

#[test]
fn browser_recovery_retains_original_occurrences_and_diagnostics_without_cleaning_source() {
    for w in WITNESSES {
        for text in [
            format!("{}/*", w.authored),
            "initial/*".into(),
            "var(--text)/*".into(),
        ] {
            let css = format!(".a{{{}:{text}", w.name);
            let report = parse_sheet(&css);
            assert!(!report.is_clean());
            assert!(
                report
                    .diagnostics()
                    .iter()
                    .any(|d| d.action() == CssRecoveryAction::RetainWithImplicitClosure)
            );
            assert!(validate_sheet(&css).is_err());
            let before = report.clone();
            let normalized = normalize_report(&report).unwrap();
            assert_eq!(normalized.diagnostics(), report.diagnostics());
            let declarations: Vec<_> = normalized
                .syntax()
                .items()
                .iter()
                .filter_map(|item| match item {
                    CssNormalizedItem::Declaration(v) => Some(v),
                    _ => None,
                })
                .collect();
            let [item] = declarations.as_slice() else {
                panic!("retained recovered occurrence")
            };
            assert_eq!(item.source().known().unwrap().property(), property(w.name));
            assert_eq!(item.source().parsed_name().unwrap().source().as_str(), css);
            match item.expansion() {
                CssExpansion::Contributions(CssContributions::Longhands(values)) => {
                    assert_terminals(values, item.source())
                }
                CssExpansion::Pending(handle) => {
                    assert!(handle.source().same_occurrence(item.source()))
                }
                _ => panic!("Text terminals or whole pending value"),
            }
            assert_eq!(report, before);
        }
    }
}

#[test]
fn pending_reentry_keeps_source_importance_and_replacement_origins_with_residual_first_retry() {
    for w in WITNESSES {
        for text in ["var(--text)", "env(text)", "attr(data-text *)"] {
            let source = parsed(w.name, text);
            let before = source.clone();
            let CssExpansion::Pending(handle) = expand_declaration(&source).unwrap() else {
                panic!("whole pending occurrence")
            };
            for residual in [
                "var(--again",
                "calc(var(--again",
                "env(again)",
                "attr(data-again *)",
            ] {
                for _ in 0..2 {
                    assert!(matches!(
                        handle
                            .reenter(parse_component_values(residual).unwrap())
                            .unwrap_err()
                            .kind(),
                        CssExpansionErrorKind::ResidualSubstitution
                    ));
                }
            }
            for _ in 0..2 {
                let error = handle
                    .reenter(parse_component_values(w.invalid).unwrap())
                    .unwrap_err();
                assert!(matches!(
                    error.kind(),
                    CssExpansionErrorKind::InvalidReplacement(_)
                ));
            }
            let recovered = format!("{}/*", w.authored);
            let components = parse_component_values(&recovered).unwrap();
            let origin = implicit_origin(&components);
            let error = handle.reenter(components).unwrap_err();
            let CssExpansionErrorKind::InvalidReplacement(error) = error.kind() else {
                panic!("original replacement closure")
            };
            assert_closure(error, &origin, &recovered);
            let replacement = parse_component_values(w.authored).unwrap();
            let replacement_before = replacement.clone();
            let CssContributions::Longhands(values) = handle.reenter(replacement.clone()).unwrap()
            else {
                panic!("successful retry")
            };
            assert_terminals(&values, &source);
            for item in values.items() {
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
                        panic!("replacement source snapshot")
                    };
                    assert!(actual.source().same_snapshot(supplied.source()));
                    assert_eq!(actual.source().as_str(), w.authored);
                }
            }
            for (global, keyword) in GLOBALS {
                let replacement = parse_component_values(global).unwrap();
                let CssContributions::Longhands(values) =
                    handle.reenter(replacement.clone()).unwrap()
                else {
                    panic!("global retry")
                };
                assert_terminals(&values, &source);
                for item in values.items() {
                    assert!(
                        matches!(item.value(), CssContributionValueRef::Global(v) if v == keyword)
                    );
                    assert_eq!(item.replacement_components(), Some(&replacement));
                }
            }
            assert_eq!(replacement, replacement_before);
            assert!(handle.source().same_occurrence(&source));
            assert_eq!(source, before);
        }
    }
}

#[test]
fn composed_normalization_and_sheet_bytes_preserve_all_occurrences_and_member_boundaries() {
    let input = format!(
        ".text{{{}}}",
        WITNESSES
            .iter()
            .map(|w| format!("{}:{}!important;", w.name, w.authored))
            .collect::<String>()
    );
    let expected = format!(
        ".text {{ {} }}",
        WITNESSES
            .iter()
            .map(|w| format!("{}: {} !important;", w.name, w.canonical))
            .collect::<Vec<_>>()
            .join(" ")
    );
    let report = parse_sheet(&input);
    assert!(report.is_clean());
    let before = report.clone();
    let terminal_budget = WITNESSES.iter().map(|w| members(w.name).len()).sum();
    let exact = CssNormalizationLimits::try_new(0, 1, WITNESSES.len(), terminal_budget).unwrap();
    let normalized = normalize_report_with_limits(&report, exact).unwrap();
    let declarations: Vec<_> = normalized
        .syntax()
        .items()
        .iter()
        .filter_map(|item| match item {
            CssNormalizedItem::Declaration(v) => Some(v),
            _ => None,
        })
        .collect();
    assert_eq!(declarations.len(), WITNESSES.len());
    for (index, (item, w)) in declarations.iter().zip(WITNESSES).enumerate() {
        assert_eq!(item.order(), index);
        assert_eq!(item.source().known().unwrap().property(), property(w.name));
        assert_eq!(item.source().importance(), CssImportance::Important);
        assert_eq!(
            item.source().parsed_name().unwrap().source().as_str(),
            input
        );
        assert!(
            item.rule_context()
                .same_context(declarations[0].rule_context())
        );
        assert!(
            item.selector_context()
                .same_context(declarations[0].selector_context())
        );
        if index > 0 {
            assert!(
                !item
                    .source()
                    .same_occurrence(declarations[index - 1].source())
            );
        }
        let CssExpansion::Contributions(CssContributions::Longhands(values)) = item.expansion()
        else {
            panic!("ordinary member boundary")
        };
        assert_terminals(values, item.source());
    }
    let short =
        CssNormalizationLimits::try_new(0, 1, WITNESSES.len(), terminal_budget - 1).unwrap();
    let error = normalize_report_with_limits(&report, short).unwrap_err();
    assert_eq!(
        error.kind(),
        &CssNormalizationErrorKind::LimitExceeded {
            resource: CssNormalizationResource::Contributions,
            limit: terminal_budget - 1
        }
    );
    assert_eq!(error.declaration_order(), Some(WITNESSES.len() - 1));
    assert!(
        error
            .declaration()
            .unwrap()
            .same_occurrence(declarations.last().unwrap().source())
    );
    let byte_exact =
        CssSpecifiedValueSerializationLimits::new(usize::MAX, usize::MAX, expected.len());
    let byte_short =
        CssSpecifiedValueSerializationLimits::new(usize::MAX, usize::MAX, expected.len() - 1);
    assert_eq!(
        report
            .syntax()
            .to_specified_css_with_limits(byte_exact)
            .unwrap(),
        expected
    );
    for _ in 0..2 {
        assert_eq!(
            report
                .syntax()
                .to_specified_css_with_limits(byte_short)
                .unwrap_err()
                .kind(),
            CssSpecifiedRuleSerializationErrorKind::Resource(
                CssSpecifiedValueSerializationErrorKind::ByteLimit
            )
        );
        assert_eq!(report, before);
    }
    assert!(normalize_report_with_limits(&report, exact).is_ok());
    assert_eq!(
        report
            .syntax()
            .to_specified_css_with_limits(byte_exact)
            .unwrap(),
        expected
    );
}

#[test]
fn prior_owner_complete_components_preserve_existing_value_and_source_contracts() {
    for name in [
        "text-align",
        "text-align-all",
        "text-align-last",
        "overflow-wrap",
        "word-spacing",
        "letter-spacing",
    ] {
        let w = witness(name);
        for text in [w.authored.to_string(), format!("{}/**/", w.authored)] {
            for grammar in [false, true] {
                let source = checked(name, &text, grammar);
                assert_eq!(
                    source.to_specified_css().unwrap(),
                    format!("{name}: {} !important;", w.canonical)
                );
                expanded(&source);
            }
        }
        for text in ["initial/**/", "var(--text)/**/"] {
            for grammar in [false, true] {
                checked(name, text, grammar);
            }
        }
    }
}
