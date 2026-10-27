//! Authored break controls from Fragmentation 3 CR (2018-12-04) §§3.1, 3.2, 3.4,
//! including the flow-relative page classifications of Logical 1 WD (2025-12-04) §3.

use crate::css::support::property_admission::assert_declaration_rejected as rejected;
use crate::css::support::{
    checked_grammar_declaration as checked, important_declaration as declaration,
    selected_grammar as grammar,
};
use surgeist_css::CssParser;
use surgeist_css::component::CssComponentValues;
use surgeist_css::conformance::{
    CssFeatureKind, CssSupportStatus, feature_metadata, property_support_metadata,
};
use surgeist_css::declaration::{CssDeclaration, CssGlobalKeyword, CssImportance};
use surgeist_css::diagnostic::{CssErrorCode, CssRecoveryAction};
use surgeist_css::normalize::CssNormalizedItem;
use surgeist_css::property::{
    CssContributionValueRef, CssContributions, CssExpansion, CssInitialValueRef, CssKnownProperty,
    CssLonghandValue, CssPropertyKindRef, CssPropertyNameRef,
};

const MODERN: [&str; 3] = ["break-before", "break-after", "break-inside"];
const LEGACY: [&str; 3] = ["page-break-before", "page-break-after", "page-break-inside"];
const BEFORE_AFTER: [&str; 12] = [
    "auto",
    "avoid",
    "avoid-page",
    "page",
    "left",
    "right",
    "recto",
    "verso",
    "avoid-column",
    "column",
    "avoid-region",
    "region",
];
const INSIDE: [&str; 5] = [
    "auto",
    "avoid",
    "avoid-page",
    "avoid-column",
    "avoid-region",
];
const LEGACY_BEFORE_AFTER: [&str; 7] =
    ["auto", "always", "avoid", "left", "right", "recto", "verso"];

fn accepted(name: &str, value: &str) {
    for source in [declaration(name, value), checked(name, value)] {
        assert_eq!(source.known().unwrap().grammar(), grammar(name));
        assert_eq!(source.importance(), CssImportance::Important);
        assert_eq!(
            source
                .value_components()
                .serialize_tokens(Default::default())
                .unwrap()
                .as_css(),
            value
        );
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
fn modern_properties_and_distinct_legacy_grammars_have_pinned_identities() {
    let mut targets = Vec::new();
    for (modern, legacy) in MODERN.into_iter().zip(LEGACY) {
        let canonical = grammar(modern);
        let alias = grammar(legacy);
        let property = canonical.target_property();
        assert_eq!(canonical.name(), modern);
        assert_eq!(property.canonical_name(), modern);
        assert_eq!(
            canonical.feature_id().as_str(),
            format!("official.property.{modern}")
        );
        assert!(!targets.contains(&property));
        targets.push(property);
        assert_eq!(alias.name(), legacy);
        assert_ne!(alias, canonical);
        assert_eq!(alias.target_property(), property);
        assert_eq!(
            alias.feature_id().as_str(),
            format!("official.property.{legacy}")
        );
        assert_eq!(property.legacy_shorthands(), &[alias]);
        assert_eq!(CssKnownProperty::from_name(legacy), None);
        assert!(property_support_metadata(legacy).is_none());

        let support = property_support_metadata(modern).expect("canonical property metadata");
        assert_eq!(support.property(), property);
        assert_eq!(support.feature().status(), CssSupportStatus::Complete);
        assert_eq!(support.feature().source().id().as_str(), "S-BREAK3");
        assert_eq!(
            support.feature().source().url(),
            Some("https://www.w3.org/TR/2018/CR-css-break-3-20181204/")
        );
        assert_eq!(support.feature().production(), format!("#propdef-{modern}"));
        let alias_feature =
            feature_metadata(alias.feature_id().as_str()).expect("legacy grammar record");
        assert_eq!(alias_feature.kind(), CssFeatureKind::PropertyAlias);
        assert_eq!(alias_feature.status(), CssSupportStatus::Complete);
        assert_eq!(alias_feature.source().id().as_str(), "S-BREAK3");
        assert_eq!(alias_feature.production(), "#page-break-properties");
    }
    assert_eq!(targets.len(), 3);
    let logical = feature_metadata("official.value.page-break-logical-values")
        .expect("selected flow-relative page classifications");
    assert_eq!(logical.kind(), CssFeatureKind::Value);
    assert_eq!(logical.status(), CssSupportStatus::Complete);
    assert_eq!(logical.source().id().as_str(), "I-LOGICAL1-20251204");
    assert_eq!(
        logical.source().url(),
        Some("https://www.w3.org/TR/2025/WD-css-logical-1-20251204/")
    );
    assert_eq!(logical.production(), "#page");

    let glyph = grammar("glyph-orientation-vertical");
    assert_eq!(glyph.target_property().canonical_name(), "text-orientation");
    assert_eq!(glyph.name(), "glyph-orientation-vertical");
}

#[test]
fn modern_and_legacy_keyword_domains_are_distinct_and_recover_invalid_neighbors() {
    for name in ["break-before", "break-after"] {
        for value in BEFORE_AFTER {
            accepted(name, value);
        }
        for (value, token) in [
            ("always", Some("always")),
            ("none", Some("none")),
            ("auto page", Some("page")),
            ("page avoid", Some("avoid")),
        ] {
            rejected(name, value, token);
        }
    }
    for value in INSIDE {
        accepted("break-inside", value);
    }
    for (value, token) in [
        ("page", Some("page")),
        ("column", Some("column")),
        ("region", Some("region")),
        ("left", Some("left")),
        ("recto", Some("recto")),
        ("always", Some("always")),
        ("auto avoid", Some("avoid")),
    ] {
        rejected("break-inside", value, token);
    }
    for name in ["page-break-before", "page-break-after"] {
        for value in LEGACY_BEFORE_AFTER {
            accepted(name, value);
        }
        for (value, token) in [
            ("page", Some("page")),
            ("avoid-page", Some("avoid-page")),
            ("column", Some("column")),
            ("avoid-column", Some("avoid-column")),
            ("region", Some("region")),
            ("avoid-region", Some("avoid-region")),
            ("none", Some("none")),
            ("always page", Some("page")),
        ] {
            rejected(name, value, token);
        }
    }
    for value in ["auto", "avoid"] {
        accepted("page-break-inside", value);
    }
    for (value, token) in [
        ("always", Some("always")),
        ("page", Some("page")),
        ("avoid-page", Some("avoid-page")),
        ("left", Some("left")),
        ("recto", Some("recto")),
        ("avoid-column", Some("avoid-column")),
    ] {
        rejected("page-break-inside", value, token);
    }
    accepted("page-break-before", "RECTO");
    accepted("break-after", "VERSO");
}

#[test]
fn legacy_shorthands_map_one_terminal_with_auto_initials_and_no_reset_members() {
    for (modern, legacy) in MODERN.into_iter().zip(LEGACY) {
        let canonical_metadata = grammar(modern).metadata().unwrap();
        let CssPropertyKindRef::Longhand(longhand) = canonical_metadata.kind() else {
            panic!("{modern} is a terminal")
        };
        assert_eq!(
            longhand.property().known_property(),
            grammar(modern).target_property()
        );
        assert!(!longhand.inherited_by_default());
        let initial = longhand.initial_value();
        let CssInitialValueRef::Value(value) = initial.view() else {
            panic!("{modern} has a fixed auto initial")
        };
        assert_eq!(value, &one_value(&declaration(modern, "auto")));

        let alias_metadata = grammar(legacy).metadata().unwrap();
        let CssPropertyKindRef::Shorthand(shorthand) = alias_metadata.kind() else {
            panic!("{legacy} is a legacy shorthand")
        };
        assert!(shorthand.is_legacy());
        assert!(shorthand.reset_only_members().is_empty());
        assert_eq!(shorthand.settable_members().len(), 1);
        assert_eq!(
            shorthand.settable_members()[0].known_property(),
            grammar(modern).target_property()
        );
        let cases: &[(&str, &str)] = if modern == "break-inside" {
            &[("auto", "auto"), ("avoid", "avoid")]
        } else {
            &[
                ("auto", "auto"),
                ("always", "page"),
                ("avoid", "avoid"),
                ("left", "left"),
                ("right", "right"),
                ("recto", "recto"),
                ("verso", "verso"),
            ]
        };
        for (authored, mapped) in cases {
            for source in [declaration(legacy, authored), checked(legacy, authored)] {
                let known = source.known().unwrap();
                assert_eq!(known.grammar(), grammar(legacy));
                assert_eq!(known.property(), grammar(modern).target_property());
                let CssExpansion::Contributions(CssContributions::Longhands(values)) =
                    source.expand().unwrap()
                else {
                    panic!("{legacy}:{authored} maps to one modern terminal")
                };
                let [item] = values.items() else {
                    panic!("one mapped contribution")
                };
                assert_eq!(item.property(), grammar(modern).target_property());
                assert_eq!(
                    item.ordinary_value(),
                    Some(&one_value(&declaration(modern, mapped)))
                );
                assert!(item.source().same_occurrence(&source));
                assert_eq!(item.source().importance(), CssImportance::Important);
                assert!(item.replacement_components().is_none());
                assert_eq!(
                    source
                        .value_components()
                        .serialize_tokens(Default::default())
                        .unwrap()
                        .as_css(),
                    *authored
                );
            }
        }
    }
}

#[test]
fn css_wide_values_and_all_include_each_modern_target_through_legacy_grammars() {
    let all_source = declaration("all", "initial");
    let CssExpansion::Contributions(CssContributions::UniversalReset(reset)) =
        all_source.expand().unwrap()
    else {
        panic!("symbolic all reset")
    };
    for (modern, legacy) in MODERN.into_iter().zip(LEGACY) {
        assert!(!reset.excludes(CssPropertyNameRef::Known(grammar(modern).target_property())));
        for name in [modern, legacy] {
            for (text, keyword) in [
                ("initial", CssGlobalKeyword::Initial),
                ("inherit", CssGlobalKeyword::Inherit),
                ("unset", CssGlobalKeyword::Unset),
                ("revert", CssGlobalKeyword::Revert),
                ("revert-layer", CssGlobalKeyword::RevertLayer),
            ] {
                let source = declaration(name, text);
                checked(name, text);
                let CssExpansion::Contributions(CssContributions::Longhands(values)) =
                    source.expand().unwrap()
                else {
                    panic!("{name} has one symbolic global")
                };
                let [item] = values.items() else {
                    panic!("one symbolic global contribution")
                };
                assert_eq!(item.property(), grammar(modern).target_property());
                assert_eq!(item.value(), CssContributionValueRef::Global(keyword));
                assert!(item.source().same_occurrence(&source));
                assert_eq!(item.source().importance(), CssImportance::Important);
            }
        }
    }
}

#[test]
fn pending_reentry_rechecks_original_legacy_grammar_and_preserves_occurrence() {
    for (modern, legacy) in MODERN.into_iter().zip(LEGACY) {
        for name in [modern, legacy] {
            for pending in ["var(--break)", "env(--break)"] {
                let source = declaration(name, pending);
                let CssExpansion::Pending(handle) = source.expand().unwrap() else {
                    panic!("{name} retains substitution")
                };
                assert!(handle.source().same_occurrence(&source));
                let invalid_value = if name == modern { "always" } else { "page" };
                assert!(
                    handle
                        .reenter(
                            CssComponentValues::parse(&CssParser::default(), invalid_value)
                                .unwrap()
                        )
                        .unwrap_err()
                        .replacement_error()
                        .is_some()
                );
                assert!(
                    handle
                        .reenter(
                            CssComponentValues::parse(&CssParser::default(), "var(--again)")
                                .unwrap()
                        )
                        .unwrap_err()
                        .is_residual_substitution()
                );
                let replacement_text = if modern == "break-inside" {
                    "avoid"
                } else if name == modern {
                    "page"
                } else {
                    "always"
                };
                let mapped = if replacement_text == "always" {
                    "page"
                } else {
                    replacement_text
                };
                let replacement =
                    CssComponentValues::parse(&CssParser::default(), replacement_text).unwrap();
                let CssContributions::Longhands(values) =
                    handle.reenter(replacement.clone()).unwrap()
                else {
                    panic!("{name} reenters to one modern terminal")
                };
                let [item] = values.items() else {
                    panic!("one substituted contribution")
                };
                assert_eq!(item.property(), grammar(modern).target_property());
                assert_eq!(
                    item.ordinary_value(),
                    Some(&one_value(&declaration(modern, mapped)))
                );
                assert!(item.source().same_occurrence(&source));
                assert_eq!(item.source().importance(), CssImportance::Important);
                assert_eq!(item.replacement_components(), Some(&replacement));
            }
        }
    }
}

#[test]
fn normalization_retains_order_and_original_legacy_occurrences_after_recovery() {
    let report = CssParser::default().parse_sheet(".a{page-break-before:always;break-after:recto;page-break-inside:avoid;page-break-after:column;break-inside:avoid-region;page-break-after:verso}");
    let [diagnostic] = report.diagnostics() else {
        panic!(
            "one invalid legacy keyword diagnostic: {:?}",
            report.diagnostics()
        )
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
    assert_eq!(declarations.len(), 5);
    for (index, (authored, target, keyword)) in [
        ("page-break-before", "break-before", "always"),
        ("break-after", "break-after", "recto"),
        ("page-break-inside", "break-inside", "avoid"),
        ("break-inside", "break-inside", "avoid-region"),
        ("page-break-after", "break-after", "verso"),
    ]
    .into_iter()
    .enumerate()
    {
        let item = declarations[index];
        assert_eq!(item.order(), index);
        assert_eq!(item.source().known().unwrap().grammar(), grammar(authored));
        assert_eq!(
            item.source().known().unwrap().property(),
            grammar(target).target_property()
        );
        assert_eq!(
            item.source()
                .value_components()
                .serialize_tokens(Default::default())
                .unwrap()
                .as_css(),
            keyword
        );
        let CssExpansion::Contributions(CssContributions::Longhands(values)) = item.expansion()
        else {
            panic!("{authored} normalizes to a modern contribution")
        };
        let [contribution] = values.items() else {
            panic!("one normalized contribution")
        };
        assert_eq!(contribution.property(), grammar(target).target_property());
        assert!(contribution.source().same_occurrence(item.source()));
    }
}
