#![forbid(unsafe_code)]
//! Independent authored text-box contract selected by Snapshot 2026 §4.
//! Pinned Inline3 WD20241218 §6.1–6.3 supplies trim, edge, shorthand and
//! metadata; §5.2 supplies only the imported <text-edge> grammar (1490–1494).
//! Its normative edge table/index (2022–2031, 4641) and changelog (3811)
//! override the informative noninheritance note (2058). CSSWG editorial
//! correction 97441f74beb997a89accfc42832399311d40d79a / PR12765 supplies
//! trim-both for omitted trim; it does not admit the printed invalid `both`.
//! Generic external-consumer APIs express executable preimplementation RED.
//! No absent enum variant, production stub, parser-private API or common
//! property inventory is introduced. Typed model construction is tested with
//! its later functional implementation. Auto stays symbolic; font metrics,
//! line-fit-edge execution and trimming are outside this authored contract.

use surgeist_css::*;

#[path = "common/authored_property.rs"]
mod authored_property;
use authored_property::{ParserFront, assert_source, checked_components};

const FAMILY: [&str; 3] = ["text-box-trim", "text-box-edge", "text-box"];
const TRIMS: [&str; 4] = ["none", "trim-start", "trim-end", "trim-both"];
const SINGLE_EDGES: [&str; 3] = ["text", "ideographic", "ideographic-ink"];
const OVER: [&str; 5] = ["text", "ideographic", "ideographic-ink", "cap", "ex"];
const UNDER: [&str; 4] = ["text", "ideographic", "ideographic-ink", "alphabetic"];
const GLOBALS: [(&str, CssGlobalKeyword); 5] = [
    ("initial", CssGlobalKeyword::Initial),
    ("inherit", CssGlobalKeyword::Inherit),
    ("unset", CssGlobalKeyword::Unset),
    ("revert", CssGlobalKeyword::Revert),
    ("revert-layer", CssGlobalKeyword::RevertLayer),
];
const FRONTS: [ParserFront; 5] = [
    ParserFront::StyleAttribute,
    ParserFront::CheckedName,
    ParserFront::CheckedGrammar,
    ParserFront::TextName,
    ParserFront::TextGrammar,
];

#[track_caller]
fn property(name: &str) -> CssKnownProperty {
    // Existing callable admission, rather than an absent Rust symbol, exposes
    // the missing behavior. This runtime probe must be clean before lookup.
    let value = match name {
        "text-box-trim" => "none",
        "text-box-edge" => "auto",
        "text-box" => "normal",
        _ => panic!("selected text-box family"),
    };
    let css = format!("{name}:{value}");
    let report = parse_style_attribute(&css);
    assert!(report.is_clean(), "{css}: {:?}", report.diagnostics());
    let [declaration] = report.syntax().as_slice() else {
        panic!("{css}: one admitted declaration")
    };
    let property = declaration.known().unwrap().property();
    assert_eq!(property.canonical_name(), name);
    assert_eq!(CssKnownProperty::from_name(name), Some(property));
    assert_eq!(
        CssKnownProperty::from_name(&name.to_ascii_uppercase()),
        Some(property)
    );
    assert_eq!(
        CssPropertyGrammar::from_name(name),
        Some(property.grammar())
    );
    property
}

fn edges() -> Vec<String> {
    let mut values = vec!["auto".to_owned()];
    values.extend(SINGLE_EDGES.map(str::to_owned));
    for over in OVER {
        for under in UNDER {
            values.push(format!("{over} {under}"));
        }
    }
    values
}

fn ordinary_cases(name: &str) -> Vec<String> {
    match name {
        "text-box-trim" => TRIMS.map(str::to_owned).to_vec(),
        "text-box-edge" => edges(),
        "text-box" => {
            let mut values = vec!["normal".to_owned()];
            values.extend(TRIMS.map(str::to_owned));
            values.extend(edges());
            for trim in TRIMS {
                for edge in edges() {
                    values.push(format!("{trim} {edge}"));
                }
            }
            values
        }
        _ => panic!("selected grammar"),
    }
}

fn source(name: &str, text: &str) -> CssDeclaration {
    ParserFront::CheckedGrammar.parse(property(name), text)
}

fn contributions(source: &CssDeclaration) -> CssLonghandContributions {
    let CssExpansion::Contributions(CssContributions::Longhands(values)) =
        expand_declaration(source).unwrap()
    else {
        panic!("ordinary or CSS-wide complete longhands")
    };
    for item in values.items() {
        assert_source(item, source);
        assert!(item.replacement_components().is_none());
    }
    values
}

fn expected_members(name: &str) -> Vec<CssKnownProperty> {
    if name == "text-box" {
        vec![property("text-box-trim"), property("text-box-edge")]
    } else {
        vec![property(name)]
    }
}

fn assert_projection(declaration: &CssDeclaration, trim: &str, edge: &str) {
    let values = contributions(declaration);
    assert_eq!(values.items().len(), 2);
    for (item, name, text) in [
        (&values.items()[0], "text-box-trim", trim),
        (&values.items()[1], "text-box-edge", edge),
    ] {
        assert_eq!(item.property(), property(name));
        assert!(matches!(item.value(), CssContributionValueRef::Ordinary(_)));
        let expected = contributions(&source(name, text));
        assert_eq!(item.ordinary_value(), expected.items()[0].ordinary_value());
    }
}

fn assert_longhand_fronts(name: &str) {
    let p = property(name);
    for text in ordinary_cases(name) {
        for front in FRONTS {
            let declaration = front.valid(p, &text, &text);
            let values = contributions(&declaration);
            assert_eq!(values.items().len(), 1);
            assert_eq!(values.items()[0].property(), p);
            assert!(matches!(
                values.items()[0].value(),
                CssContributionValueRef::Ordinary(_)
            ));
        }
    }
}

#[test]
fn every_trim_choice_admits_through_five_fronts() {
    assert_longhand_fronts("text-box-trim");
}

#[test]
fn exact_edge_single_pair_alternatives_admit_through_five_fronts() {
    assert_longhand_fronts("text-box-edge");
}

#[test]
fn shorthand_preserves_authored_slot_omission_and_orders_whole_trim_edge_slots() {
    let p = property("text-box");
    for front in FRONTS {
        assert_projection(&front.valid(p, "normal", "normal"), "none", "auto");
        for trim in TRIMS {
            assert_projection(&front.valid(p, trim, trim), trim, "auto");
            for edge in edges() {
                let canonical = format!("{trim} {edge}");
                for input in [canonical.clone(), format!("{edge} {trim}")] {
                    assert_projection(&front.valid(p, &input, &canonical), trim, &edge);
                }
            }
        }
        for edge in edges() {
            assert_projection(&front.valid(p, &edge, &edge), "trim-both", &edge);
        }
    }
}

#[test]
fn case_escapes_comments_and_programmatic_origins_preserve_meaning_and_pair_roles() {
    for (name, input, canonical) in [
        ("text-box-trim", r"\74 rim-both", "trim-both"),
        ("text-box-edge", r"\63 ap /**/ ALPHABETIC", "cap alphabetic"),
        ("text-box", r"EX/**/TEXT/**/\74 rim-end", "trim-end ex text"),
    ] {
        let p = property(name);
        for front in FRONTS {
            front.valid(p, input, canonical);
        }
        let words: Vec<_> = canonical.split_whitespace().collect();
        let mut items = Vec::new();
        for (i, word) in words.iter().enumerate() {
            if i > 0 {
                items.push(CssComponentValue::try_token(" ").unwrap());
            }
            items.push(CssComponentValue::try_ident(*word).unwrap());
        }
        let components = CssComponentValues::try_new(items).unwrap();
        for grammar in [false, true] {
            let declaration =
                checked_components(p, components.clone(), grammar, CssImportance::Important)
                    .unwrap();
            assert_eq!(declaration.value_components(), &components);
            assert!(
                declaration
                    .value_components()
                    .items()
                    .iter()
                    .all(|v| v.origin() == &CssValueOrigin::Programmatic)
            );
            assert_eq!(
                declaration.to_specified_css().unwrap(),
                format!("{name}: {canonical} !important;")
            );
            contributions(&declaration);
        }
    }
    let css = r"\74 ext-box:CAP ALPHABETIC TRIM-START!important";
    let report = parse_style_attribute(css);
    assert!(report.is_clean());
    assert_eq!(
        report.syntax()[0].to_specified_css().unwrap(),
        "text-box: trim-start cap alphabetic !important;"
    );
    assert_eq!(
        report.syntax()[0].parsed_name().unwrap().source().as_str(),
        css
    );
}

#[test]
fn forbidden_single_metrics_wrong_pair_roles_and_duplicate_slots_drop_atomically() {
    let trim = property("text-box-trim");
    for value in [
        "both",
        "start",
        "end",
        "auto",
        "normal",
        "trim-start trim-end",
        "none none",
    ] {
        authored_property::invalid(trim, value);
    }
    let edge = property("text-box-edge");
    for value in [
        "cap",
        "ex",
        "alphabetic",
        "leading",
        "auto text",
        "text auto",
        "cap cap",
        "ex ex",
        "alphabetic text",
        "text cap",
        "text ex",
        "text text text",
    ] {
        authored_property::invalid(edge, value);
    }
    for over in OVER {
        for forbidden_under in ["cap", "ex", "auto"] {
            authored_property::invalid(edge, &format!("{over} {forbidden_under}"));
        }
    }
    let shorthand = property("text-box");
    for value in [
        "both",
        "cap",
        "ex",
        "alphabetic",
        "normal auto",
        "none normal",
        "normal normal",
        "trim-start trim-end",
        "auto auto",
        "text text text",
        "cap trim-start alphabetic",
        "trim-both cap cap",
        "trim-start cap",
        "trim-end alphabetic",
    ] {
        authored_property::invalid(shorthand, value);
    }
    for name in FAMILY {
        for value in [
            "",
            "1",
            "1px",
            "25%",
            "calc(1 + 2)",
            "normal,auto",
            "[text]",
            "\"text\"",
            "initial auto",
            "auto inherit",
        ] {
            authored_property::invalid(property(name), value);
        }
    }
}

#[test]
fn trim_none_and_edge_auto_initials_remain_ordinary_with_only_edge_inheriting() {
    for (name, text, inherited) in [
        ("text-box-trim", "none", false),
        ("text-box-edge", "auto", true),
    ] {
        let p = property(name);
        let CssPropertyKindRef::Longhand(metadata) = p.metadata().unwrap().kind() else {
            panic!("ordinary longhand metadata")
        };
        assert_eq!(metadata.inherited_by_default(), inherited);
        assert_eq!(metadata.property().known_property(), p);
        let initial_value = metadata.initial_value();
        let CssInitialValueRef::Value(initial) = initial_value.view() else {
            panic!("intrinsic symbolic initial")
        };
        let values = contributions(&source(name, text));
        assert_eq!(values.items()[0].ordinary_value(), Some(initial));
    }
    let CssPropertyKindRef::Shorthand(metadata) = property("text-box").metadata().unwrap().kind()
    else {
        panic!("ordinary two-member shorthand")
    };
    assert_eq!(
        metadata
            .members()
            .iter()
            .map(|v| v.known_property())
            .collect::<Vec<_>>(),
        expected_members("text-box")
    );
    assert_eq!(metadata.settable_members(), metadata.members());
    assert!(metadata.reset_only_members().is_empty());
    assert!(!metadata.is_legacy());
}

#[test]
fn every_css_wide_keyword_expands_to_all_members_with_original_importance() {
    for name in FAMILY {
        let p = property(name);
        for (text, keyword) in GLOBALS {
            for front in FRONTS {
                let declaration = front.valid(p, text, text);
                assert_eq!(declaration.known().unwrap().global(), Some(keyword));
                assert!(declaration.known().unwrap().property_value().is_none());
                let values = contributions(&declaration);
                assert_eq!(
                    values
                        .items()
                        .iter()
                        .map(CssLonghandContribution::property)
                        .collect::<Vec<_>>(),
                    expected_members(name)
                );
                for item in values.items() {
                    assert!(
                        matches!(item.value(), CssContributionValueRef::Global(actual) if actual == keyword)
                    );
                }
            }
        }
    }
}

#[test]
fn var_env_whole_value_reentry_preserves_source_replacement_and_every_ordinary_expansion() {
    for name in FAMILY {
        let p = property(name);
        for pending in ["var(--box)", "env(box)"] {
            for front in FRONTS {
                let declaration = front.valid(p, pending, pending);
                let before = declaration.clone();
                let CssExpansion::Pending(handle) = expand_declaration(&declaration).unwrap()
                else {
                    panic!("one whole-value pending handle")
                };
                assert!(handle.source().same_occurrence(&declaration));
                for text in ordinary_cases(name) {
                    let replacement = parse_component_values(&text.to_ascii_uppercase()).unwrap();
                    let snapshot = replacement.clone();
                    let CssContributions::Longhands(values) =
                        handle.reenter(replacement.clone()).unwrap()
                    else {
                        panic!("complete replacement")
                    };
                    let expected = contributions(&source(name, &text));
                    assert_eq!(values.items().len(), expected.items().len());
                    for (item, expected) in values.items().iter().zip(expected.items()) {
                        assert_source(item, &declaration);
                        assert_eq!(item.property(), expected.property());
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
                                panic!("replacement parsed origin")
                            };
                            assert!(actual.source().same_snapshot(supplied.source()));
                            assert_eq!(actual.source().as_str(), text.to_ascii_uppercase());
                        }
                    }
                    assert_eq!(replacement, snapshot);
                }
                for (text, keyword) in GLOBALS {
                    let replacement = parse_component_values(text).unwrap();
                    let CssContributions::Longhands(values) =
                        handle.reenter(replacement.clone()).unwrap()
                    else {
                        panic!("complete global replacement")
                    };
                    assert_eq!(
                        values
                            .items()
                            .iter()
                            .map(CssLonghandContribution::property)
                            .collect::<Vec<_>>(),
                        expected_members(name)
                    );
                    for item in values.items() {
                        assert_source(item, &declaration);
                        assert!(
                            matches!(item.value(), CssContributionValueRef::Global(actual) if actual == keyword)
                        );
                        assert_eq!(item.replacement_components(), Some(&replacement));
                    }
                }
                assert_eq!(declaration, before);
            }
        }
    }
}

#[test]
fn invalid_replacements_residual_functions_and_annotations_fail_atomically_then_retry() {
    for name in FAMILY {
        for pending in ["var(--box)", "env(box)", "text var(--box)"] {
            let declaration = source(name, pending);
            let before = declaration.clone();
            let CssExpansion::Pending(handle) = expand_declaration(&declaration).unwrap() else {
                panic!("whole-value pending")
            };
            for text in [
                "var(--again)",
                "env(again)",
                "text var(--again)",
                r"\76 ar(--again)",
                "var(--again",
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
                "both",
                "cap",
                "auto auto",
                "initial text",
                "text!important",
                "text;color:red",
            ] {
                for _ in 0..2 {
                    let error = handle
                        .reenter(parse_component_values(text).unwrap())
                        .unwrap_err();
                    let CssExpansionErrorKind::InvalidReplacement(error) = error.kind() else {
                        panic!("strict complete grammar failure")
                    };
                    assert!(matches!(
                        error.kind(),
                        CssPropertyValueErrorKind::Grammar(_)
                    ));
                }
            }
            let valid = match name {
                "text-box-trim" => "trim-start",
                "text-box-edge" => "cap alphabetic",
                _ => "cap alphabetic trim-end",
            };
            assert!(
                handle
                    .reenter(parse_component_values(valid).unwrap())
                    .is_ok()
            );
            assert!(handle.source().same_occurrence(&declaration));
            assert_eq!(declaration, before);
        }
    }
}

fn implicit_origin(components: &CssComponentValues) -> CssValueOrigin {
    let serialized = components.serialize().unwrap();
    (0..serialized.as_css().len())
        .find_map(|offset| match serialized.origin_at(offset) {
            Some(CssSerializedOrigin::Token(origin @ CssValueOrigin::ImplicitClosure { .. })) => {
                Some(origin.clone())
            }
            _ => None,
        })
        .expect("original implicit closing origin")
}

fn assert_closure(error: &CssPropertyValueParseError, origin: &CssValueOrigin) {
    assert!(matches!(
        error.kind(),
        CssPropertyValueErrorKind::Grammar(ErrorKind::UnexpectedEnd(_))
    ));
    assert_eq!(
        error.origin(),
        &CssSerializedOrigin::End(Some(origin.clone()))
    );
}

#[test]
fn checked_original_and_replacement_closures_reject_with_exact_origin() {
    for name in FAMILY {
        let p = property(name);
        let ordinary = match name {
            "text-box-trim" => "trim-both",
            "text-box-edge" => "ex text",
            _ => "trim-start cap alphabetic",
        };
        for text in [
            format!("{ordinary}/*"),
            "initial/*".to_owned(),
            "var(--box".to_owned(),
            "env(box".to_owned(),
        ] {
            let components = parse_component_values(&text).unwrap();
            let before = components.clone();
            // Function closers expose their closing origin directly; comment
            // closers expose it through original-component serialization.
            let origin = components
                .items()
                .iter()
                .find_map(|v| match v.view() {
                    CssComponentValueRef::Function(v) => Some(v.closing_origin().clone()),
                    _ => None,
                })
                .unwrap_or_else(|| implicit_origin(&components));
            for grammar in [false, true] {
                assert_closure(
                    &checked_components(p, components.clone(), grammar, CssImportance::Normal)
                        .unwrap_err(),
                    &origin,
                );
            }
            assert_eq!(components, before);
        }
        let declaration = source(name, "var(--box)");
        let CssExpansion::Pending(handle) = expand_declaration(&declaration).unwrap() else {
            panic!("pending")
        };
        for text in [format!("{ordinary}/*"), "unset/*".to_owned()] {
            let replacement = parse_component_values(&text).unwrap();
            let origin = implicit_origin(&replacement);
            let error = handle.reenter(replacement).unwrap_err();
            let CssExpansionErrorKind::InvalidReplacement(error) = error.kind() else {
                panic!("strict replacement closure")
            };
            assert_closure(error, &origin);
        }
        assert!(
            handle
                .reenter(parse_component_values(&format!("{ordinary}/**/")).unwrap())
                .is_ok()
        );
    }
}

#[test]
fn annotation_errors_reference_original_bang_and_authored_importance_stays_separate() {
    for name in FAMILY {
        let p = property(name);
        let ordinary = match name {
            "text-box-trim" => "none",
            "text-box-edge" => "auto",
            _ => "normal",
        };
        let text = format!("/*😀*/{ordinary}!important");
        let components = parse_component_values(&text).unwrap();
        let serialized = components.serialize().unwrap();
        let origin = serialized
            .origin_at(serialized.as_css().find('!').unwrap())
            .unwrap();
        for grammar in [false, true] {
            let error =
                checked_components(p, components.clone(), grammar, CssImportance::Important)
                    .unwrap_err();
            assert!(matches!(
                error.kind(),
                CssPropertyValueErrorKind::Grammar(_)
            ));
            assert_eq!(error.origin(), origin);
        }
        for report in [
            parse_property_value_text(
                &text,
                CssPropertyNameRef::Known(p),
                CssImportance::Important,
            ),
            parse_property_value_text_for_grammar(&text, p.grammar(), CssImportance::Important),
        ] {
            assert!(report.syntax().is_none());
            assert!(!report.is_clean());
        }
        ParserFront::StyleAttribute.valid(p, ordinary, ordinary);
    }
}

#[test]
fn normalized_order_contexts_pending_and_importance_charge_exact_cumulative_members() {
    let report = parse_sheet(
        "@media screen{.a{text-box:cap alphabetic!important;text-box-trim:trim-end;text-box-edge:ex text;text-box:var(--box);text-box:unset!important;color:red}}",
    );
    assert!(report.is_clean(), "{:?}", report.diagnostics());
    let before = report.clone();
    // Four ordinary contributions +two shorthand global contributions +color,
    // and one pending handle =8 contributions across6 declarations.
    let exact = CssNormalizationLimits::try_new(1, 2, 6, 8).unwrap();
    let normalized = normalize_sheet_with_limits(report.syntax(), exact).unwrap();
    let declarations: Vec<_> = normalized
        .items()
        .iter()
        .filter_map(|v| match v {
            CssNormalizedItem::Declaration(v) => Some(v),
            _ => None,
        })
        .collect();
    assert_eq!(declarations.len(), 6);
    let names = [
        "text-box",
        "text-box-trim",
        "text-box-edge",
        "text-box",
        "text-box",
        "color",
    ];
    for (i, item) in declarations.iter().enumerate() {
        assert_eq!(item.order(), i);
        assert_eq!(
            item.source().known().unwrap().property().canonical_name(),
            names[i]
        );
        assert_eq!(
            item.source().importance(),
            if i == 0 || i == 4 {
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
        if i > 0 {
            assert!(!item.source().same_occurrence(declarations[i - 1].source()));
        }
        match item.expansion() {
            CssExpansion::Pending(handle) => {
                assert_eq!(i, 3);
                assert!(handle.source().same_occurrence(item.source()));
            }
            CssExpansion::Contributions(CssContributions::Longhands(values)) => {
                assert_eq!(values.items().len(), if i == 0 || i == 4 { 2 } else { 1 });
                for value in values.items() {
                    assert_source(value, item.source());
                }
            }
            _ => panic!("intrinsic expansion"),
        }
    }
    for (limits, resource, limit, failing_order) in [
        (
            CssNormalizationLimits::try_new(1, 2, 6, 7).unwrap(),
            CssNormalizationResource::Contributions,
            7,
            5,
        ),
        (
            CssNormalizationLimits::try_new(1, 2, 5, 8).unwrap(),
            CssNormalizationResource::Declarations,
            5,
            5,
        ),
    ] {
        for _ in 0..2 {
            let error = normalize_sheet_with_limits(report.syntax(), limits).unwrap_err();
            assert_eq!(
                error.kind(),
                &CssNormalizationErrorKind::LimitExceeded { resource, limit }
            );
            assert_eq!(error.declaration_order(), Some(failing_order));
            assert!(
                error
                    .declaration()
                    .unwrap()
                    .same_occurrence(declarations[failing_order].source())
            );
        }
    }
    assert_eq!(report, before);
    assert!(normalize_sheet_with_limits(report.syntax(), exact).is_ok());
}

#[test]
fn browser_eof_recovery_keeps_original_declaration_and_diagnostics_during_normalization() {
    for (name, text, count) in [
        ("text-box-trim", "trim-start/*", 1),
        ("text-box-edge", "cap alphabetic/*", 1),
        ("text-box", "trim-end ex text/*", 2),
        ("text-box", "var(--box", 1),
    ] {
        let css = format!(".a{{{name}:{text}");
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
            panic!("retained recovered declaration")
        };
        assert_eq!(item.order(), 0);
        assert_eq!(
            item.source().known().unwrap().property().canonical_name(),
            name
        );
        assert_eq!(item.source().parsed_name().unwrap().source().as_str(), css);
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
        assert!(error.declaration().unwrap().same_occurrence(item.source()));
        assert_eq!(report, before);
        assert!(normalize_report_with_limits(&report, exact).is_ok());
    }
}

#[test]
fn semantic_keyword_budgets_and_canonical_bytes_fail_atomically_then_retry() {
    use CssSpecifiedValueSerializationErrorKind as Kind;
    use CssSpecifiedValueSerializationLimits as Limits;
    for name in FAMILY {
        let p = property(name);
        let mut cases = ordinary_cases(name);
        cases.extend(GLOBALS.map(|(text, _)| text.to_owned()));
        for text in cases {
            for front in [ParserFront::StyleAttribute, ParserFront::CheckedGrammar] {
                let declaration = front.parse(p, &text);
                let before = declaration.clone();
                let expected = format!("{name}: {text} !important;");
                // Existing specified-output tariff: one semantic node per
                // emitted keyword, plus declaration/name; no guessed work.
                let nodes = text.split_whitespace().count() + 2;
                let exact = Limits::new(nodes, nodes, expected.len());
                assert_eq!(
                    declaration.to_specified_css_with_limits(exact).unwrap(),
                    expected
                );
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
                    for _ in 0..2 {
                        assert_eq!(
                            declaration
                                .to_specified_css_with_limits(limits)
                                .unwrap_err()
                                .kind(),
                            kind
                        );
                        assert_eq!(declaration, before);
                    }
                }
                assert_eq!(
                    declaration.to_specified_css_with_limits(exact).unwrap(),
                    expected
                );
            }
        }
    }
}

#[test]
fn sheet_siblings_share_final_byte_limit_and_failed_output_keeps_input_reusable() {
    use CssSpecifiedValueSerializationLimits as Limits;
    let report = parse_sheet(
        ".a{text-box-trim:trim-start!important}.b{text-box-edge:cap alphabetic}.c{text-box:ex text trim-end}",
    );
    assert!(report.is_clean(), "{:?}", report.diagnostics());
    let sheet = report.syntax();
    let before = sheet.clone();
    let expected = ".a { text-box-trim: trim-start !important; }\n.b { text-box-edge: cap alphabetic; }\n.c { text-box: trim-end ex text; }";
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
        assert_eq!(error.rule_index(), Some(2));
        assert_eq!(sheet, &before);
    }
    assert_eq!(sheet.to_specified_css_with_limits(exact).unwrap(), expected);
}

#[test]
fn existing_shared_keyword_global_pending_and_shorthand_controls_execute() {
    for (name, text, expected, count) in [
        ("text-wrap-mode", "nowrap", "nowrap", 1),
        ("flex-flow", "wrap row-reverse", "row-reverse wrap", 2),
        ("color", "initial", "initial", 1),
    ] {
        let p = CssKnownProperty::from_name(name).unwrap();
        let declaration = ParserFront::CheckedGrammar.valid(p, text, expected);
        assert_eq!(contributions(&declaration).items().len(), count);
    }
    let p = CssKnownProperty::Color;
    let declaration = ParserFront::CheckedGrammar.parse(p, "var(--c)");
    let CssExpansion::Pending(handle) = expand_declaration(&declaration).unwrap() else {
        panic!("existing shared pending control")
    };
    assert!(
        handle
            .reenter(parse_component_values("red").unwrap())
            .is_ok()
    );
}
