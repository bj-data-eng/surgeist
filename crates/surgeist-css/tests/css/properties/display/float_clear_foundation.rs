//! CSS2 REC (2011-06-07) §§9.5.1–9.5.2 plus Logical 1 WD (2025-12-04)
//! §2.2. Flow-relative values remain authored keywords until the containing
//! block's writing mode is available downstream.

use crate::css::support::property_admission::assert_declaration_rejected as rejected;
use crate::css::support::{
    checked_grammar_declaration as checked, important_declaration as declaration,
    selected_grammar as grammar,
};
use surgeist_css::*;

const NAMES: [&str; 2] = ["float", "clear"];

fn authored_value<'a>(source: &'a CssDeclaration, name: &str) -> &'a str {
    let known = source.known().expect("known property");
    assert_eq!(known.property(), grammar(name).target_property());
    match (name, known.property_value().unwrap()) {
        ("float", CssKnownPropertyValueRef::Float(value)) => value.as_css(),
        ("clear", CssKnownPropertyValueRef::Clear(value)) => value.as_css(),
        _ => panic!("{name} has its typed property wrapper"),
    }
}

fn accepted(name: &str, value: &str) {
    for source in [declaration(name, value), checked(name, value)] {
        assert_eq!(source.importance(), CssImportance::Important);
        assert_eq!(
            source
                .value_components()
                .serialize_tokens(Default::default())
                .unwrap()
                .as_css(),
            value
        );
        assert_eq!(authored_value(&source, name), value);
    }
}

fn one_value(source: &CssDeclaration) -> CssLonghandValue {
    let CssExpansion::Contributions(CssContributions::Longhands(values)) = source.expand().unwrap()
    else {
        panic!("one terminal contribution")
    };
    let [item] = values.items() else {
        panic!("one terminal contribution")
    };
    assert_eq!(item.property(), source.known().unwrap().property());
    assert!(item.source().same_occurrence(source));
    item.ordinary_value().unwrap().clone()
}

#[test]
fn both_names_keep_baseline_ids_and_report_complete_logical1_extension() {
    for name in NAMES {
        let property = grammar(name).target_property();
        assert_eq!(grammar(name).name(), name);
        assert_eq!(property.canonical_name(), name);
        assert_eq!(
            grammar(name).feature_id().as_str(),
            format!("baseline.property.{name}")
        );
        let support = property_support_metadata(name).expect("selected support metadata");
        assert_eq!(support.property(), property);
        assert_eq!(support.feature().status(), CssSupportStatus::Complete);
        assert_eq!(
            support.feature().source().id().as_str(),
            "I-LOGICAL1-20251204"
        );
        assert_eq!(
            support.feature().source().url(),
            Some("https://www.w3.org/TR/2025/WD-css-logical-1-20251204/")
        );
        assert_eq!(support.feature().production(), "#float-clear");
    }
    assert_ne!(
        grammar("float").target_property(),
        grammar("clear").target_property()
    );
}

#[test]
fn physical_and_flow_relative_keywords_parse_without_losing_authored_spelling() {
    for name in NAMES {
        for value in ["none", "left", "right"] {
            accepted(name, value);
        }
        for value in [
            "inline-start",
            "inline-end",
            "INLINE-START",
            "InLiNe-EnD",
            "inline\\2d start",
            "inline\\2d end",
        ] {
            accepted(name, value);
        }
    }
    accepted("clear", "both");
    for (value, token) in [
        ("both", Some("both")),
        ("start", Some("start")),
        ("end", Some("end")),
        ("center", Some("center")),
        ("inline", Some("inline")),
        ("block-start", Some("block-start")),
        ("inline-start left", Some("left")),
        ("initial left", Some("left")),
    ] {
        rejected("float", value, token);
    }
    for (value, token) in [
        ("start", Some("start")),
        ("end", Some("end")),
        ("center", Some("center")),
        ("inline", Some("inline")),
        ("block-end", Some("block-end")),
        ("inline-end right", Some("right")),
        ("initial both", Some("both")),
    ] {
        rejected("clear", value, token);
    }
}

#[test]
fn both_are_noninherited_terminals_with_none_initials_and_one_ordinary_contribution() {
    for name in NAMES {
        let property = grammar(name).target_property();
        let CssPropertyKindRef::Longhand(longhand) = grammar(name).metadata().unwrap().kind()
        else {
            panic!("{name} is a terminal longhand")
        };
        assert_eq!(longhand.property().known_property(), property);
        assert!(!longhand.inherited_by_default());
        let initial = longhand.initial_value();
        assert_eq!(initial.property().known_property(), property);
        let CssInitialValueRef::Value(value) = initial.view() else {
            panic!("{name} has a fixed initial")
        };
        assert_eq!(value, &one_value(&declaration(name, "none")));

        for keyword in ["left", "right", "inline-start", "inline-end"] {
            for source in [declaration(name, keyword), checked(name, keyword)] {
                let CssExpansion::Contributions(CssContributions::Longhands(values)) =
                    source.expand().unwrap()
                else {
                    panic!("{name}:{keyword} has one ordinary contribution")
                };
                let [item] = values.items() else {
                    panic!("one ordinary contribution")
                };
                assert_eq!(item.property(), property);
                assert_eq!(
                    item.ordinary_value().unwrap().property().known_property(),
                    property
                );
                assert!(item.source().same_occurrence(&source));
                assert_eq!(item.source().importance(), CssImportance::Important);
                assert!(item.replacement_components().is_none());
                assert_eq!(authored_value(&source, name), keyword);
            }
            assert_eq!(
                one_value(&declaration(name, keyword)),
                one_value(&checked(name, keyword)),
                "parsed and checked {name}:{keyword} agree"
            );
        }
        assert_ne!(
            one_value(&declaration(name, "inline-start")),
            one_value(&declaration(name, "left")),
            "logical keyword stays symbolic"
        );
        assert_ne!(
            one_value(&declaration(name, "inline-start")),
            one_value(&declaration(name, "inline-end")),
            "logical start and end stay distinct"
        );
    }
}

#[test]
fn pending_reentry_checks_full_keyword_grammar_and_preserves_original_occurrence() {
    for name in NAMES {
        for pending in ["var(--side)", "env(--side)"] {
            let source = declaration(name, pending);
            let CssExpansion::Pending(handle) = source.expand().unwrap() else {
                panic!("{name} keeps substitution pending")
            };
            assert!(handle.source().same_occurrence(&source));
            for invalid in ["start", "center", "inline-start left"] {
                assert!(
                    handle
                        .reenter(CssComponentValues::parse(&CssParser::default(), invalid).unwrap())
                        .unwrap_err()
                        .replacement_error()
                        .is_some(),
                    "{name}:{invalid}"
                );
            }
            if name == "float" {
                assert!(
                    handle
                        .reenter(CssComponentValues::parse(&CssParser::default(), "both").unwrap())
                        .unwrap_err()
                        .replacement_error()
                        .is_some()
                );
            }
            assert!(
                handle
                    .reenter(
                        CssComponentValues::parse(&CssParser::default(), "var(--again)").unwrap()
                    )
                    .unwrap_err()
                    .is_residual_substitution()
            );
            let replacement =
                CssComponentValues::parse(&CssParser::default(), "inline-end").unwrap();
            let CssContributions::Longhands(values) = handle.reenter(replacement.clone()).unwrap()
            else {
                panic!("{name} reenters to one longhand")
            };
            let [item] = values.items() else {
                panic!("one substituted contribution")
            };
            assert_eq!(item.property(), grammar(name).target_property());
            assert!(item.ordinary_value().is_some());
            assert!(item.source().same_occurrence(&source));
            assert_eq!(item.source().importance(), CssImportance::Important);
            assert_eq!(item.replacement_components(), Some(&replacement));
            let CssContributions::Longhands(globals) = handle
                .reenter(CssComponentValues::parse(&CssParser::default(), "inherit").unwrap())
                .unwrap()
            else {
                panic!("{name} reenters to a global keyword")
            };
            assert_eq!(
                globals.items()[0].value(),
                CssContributionValueRef::Global(CssGlobalKeyword::Inherit)
            );
            assert!(globals.items()[0].source().same_occurrence(&source));
        }
    }
}

#[test]
fn normalization_preserves_order_and_authored_logical_keywords_after_recovery() {
    let report = CssParser::default()
        .parse_sheet(".a{float:inline-start;clear:left;float:center;clear:INLINE-END;float:right}");
    let [diagnostic] = report.diagnostics() else {
        panic!("one invalid float diagnostic: {:?}", report.diagnostics())
    };
    assert_eq!(diagnostic.action(), CssRecoveryAction::DropDeclaration);
    assert_ne!(diagnostic.error().code(), CssErrorCode::UnknownProperty);
    let normalized = report.syntax().normalize(Default::default()).unwrap();
    let declarations = normalized
        .items()
        .iter()
        .filter_map(|item| match item {
            CssNormalizedItem::Declaration(value) => Some(value),
            _ => None,
        })
        .collect::<Vec<_>>();
    assert_eq!(declarations.len(), 4);
    for (index, (name, value)) in [
        ("float", "inline-start"),
        ("clear", "left"),
        ("clear", "INLINE-END"),
        ("float", "right"),
    ]
    .into_iter()
    .enumerate()
    {
        let item = declarations[index];
        assert_eq!(item.order(), index);
        assert_eq!(
            item.source().known().unwrap().property(),
            grammar(name).target_property()
        );
        assert_eq!(authored_value(item.source(), name), value);
        assert!(matches!(
            item.expansion(),
            CssExpansion::Contributions(CssContributions::Longhands(_))
        ));
    }
}
