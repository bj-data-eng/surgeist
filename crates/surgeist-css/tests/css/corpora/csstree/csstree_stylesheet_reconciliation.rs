//! Original stylesheet expectations from an independent pinned-publication audit.
//! Grammar retention, public recovery policy, and registry classification are
//! separate assertions. Exact diagnostic predictions come from a second static
//! source-contract audit, never parser output or the captured corpus oracle.

use std::collections::BTreeSet;

use serde_json::{Value, json};
use surgeist_css::{
    CssDeclarationList, CssImportance, CssKnownPropertyValueRef, CssRule, CssSelector, parse_sheet,
    validate_sheet,
};

use crate::css::support::csstree::adapters::{EntryPoint, Extractor};
use crate::css::support::csstree::reconciliation::{
    ObservationRoute, PredictedObservation, Reconciliation, RowPaths, catalog_module, text,
};

fn reconciliation() -> Reconciliation {
    Reconciliation::load("stylesheet-reconciliation.json", RowPaths::Crate)
}

fn declarations(actual: &CssDeclarationList) -> Value {
    Value::Array(
        actual
            .iter()
            .map(|declaration| {
                assert_eq!(declaration.importance(), CssImportance::Normal);
                match declaration.known().and_then(|known| known.property_value()) {
                    Some(CssKnownPropertyValueRef::Color(value)) => {
                        assert_eq!(value.value().named().expect("named color").name(), "red");
                        json!(["color", "red"])
                    }
                    Some(CssKnownPropertyValueRef::Width(value)) => {
                        assert_eq!(
                            value
                                .value()
                                .serialize_specified(Default::default())
                                .unwrap(),
                            "10px"
                        );
                        json!(["width", "10px"])
                    }
                    other => panic!("unexpected retained declaration: {other:?}"),
                }
            })
            .collect(),
    )
}

fn selector(actual: &CssSelector) -> String {
    match actual {
        CssSelector::Tag(name) => name.clone(),
        CssSelector::Class(name) => format!(".{name}"),
        CssSelector::Compound(compound) => {
            assert!(
                compound
                    .type_selector()
                    .is_some_and(|name| name.is_universal())
            );
            assert!(compound.ids().is_empty());
            assert!(compound.classes().is_empty());
            assert!(compound.attributes().is_empty());
            assert!(compound.pseudo_classes().is_empty());
            assert!(compound.pseudo_elements().is_none());
            assert_eq!(compound.nesting_selectors(), 0);
            assert!(!compound.has_scope_anchor());
            "*".into()
        }
        other => panic!("unexpected retained selector: {other:?}"),
    }
}

fn rules(actual: &[CssRule]) -> Value {
    Value::Array(actual.iter().map(|rule| match rule {
        CssRule::Style(style) => {
            assert_eq!(style.selectors().selectors().len(), 1);
            json!({"kind": "style", "selector": selector(style.selectors().selectors()[0].selector()),
                "declarations": declarations(style.declarations()), "nested_rules": rules(style.rules())})
        }
        CssRule::Media(media) => {
            // Whitespace is immaterial for the two simple query forms in this
            // fixture; serialization still exposes feature names and values.
            let condition = media.query().serialize_tokens(Default::default()).unwrap().as_css().replace(' ', "");
            json!({"kind": "media", "condition": condition, "rules": rules(media.rules())})
        }
        CssRule::Page(page) => {
            assert!(page.selectors().is_empty());
            json!({"kind": "page", "selector": null, "declarations": declarations(page.declarations().properties())})
        }
        other => panic!("unexpected retained rule: {other:?}"),
    }).collect())
}

/// Public retained rules, their typed values and strict validation for one row.
fn typed_rules_reconcile(row: &Value) -> Result<(), String> {
    let input = text(row, "input");
    let report = parse_sheet(input);
    let actual = json!({"ordered_rules": rules(report.syntax().rules()),
        "validation_accepts": validate_sheet(input).is_ok()});
    let predicted = json!({"ordered_rules": row["expected"]["ordered_rules"],
        "validation_accepts": row["expected"]["validation_accepts"]});
    if actual == predicted {
        Ok(())
    } else {
        Err(format!("expected {predicted}, actual {actual}"))
    }
}

#[test]
fn original_stylesheets_retain_rules_values_and_follow_independent_recovery() {
    let seen = reconciliation().reconcile_observations(
        ObservationRoute {
            entry_point: EntryPoint::Sheet,
            extractor: Some(Extractor::SheetRules),
        },
        |_| true,
        |row| PredictedObservation {
            syntax_count: row["expected"]["top_level_rule_count"].as_u64().unwrap(),
            is_clean: row["expected"]["is_clean"].as_bool().unwrap(),
            diagnostics: row["expected_raw_diagnostics"].as_array().unwrap().clone(),
        },
        Some(&typed_rules_reconcile),
    );
    assert_eq!(seen, 76);
}

#[test]
fn stylesheet_registry_matches_independent_complete_original_classes() {
    reconciliation().assert_registry_classes();
}

#[test]
fn original_stylesheet_identity_input_options_and_source_hashes_are_preserved() {
    let reconciliation = reconciliation();
    let data = reconciliation.data();
    let rows = reconciliation.rows();
    assert_eq!(rows.len(), 76);
    // Preserve the audit's historical whole-catalog hash as provenance, while
    // checking its actual publication dependencies against the current catalog.
    // An unrelated required-definition import does not change these sources.
    let audited_publications = data["audited_publications"].as_array().unwrap();
    let referenced_urls: BTreeSet<_> = rows
        .iter()
        .flat_map(|row| row["normative_sources"].as_array().unwrap())
        .map(|url| url.as_str().unwrap().split('#').next().unwrap())
        .collect();
    let audited_urls: BTreeSet<_> = audited_publications
        .iter()
        .map(|audited| text(&audited["publication"], "url"))
        .collect();
    assert_eq!(audited_urls, referenced_urls);
    assert_eq!(audited_publications.len(), referenced_urls.len());
    for audited in audited_publications {
        assert_eq!(
            catalog_module(&audited["module_id"])["publication"],
            audited["publication"]
        );
    }
    assert_eq!(
        reconciliation.assert_original_bindings(rows, "stylesheet"),
        76
    );
}
