//! Original at-rule grammar expectations, independently authored from pinned
//! specifications and the public single-rule recovery contract. The numeric
//! diagnostic tuples characterize API behavior by static source derivation;
//! they are not coordinate requirements imposed by CSS publications.

use serde_json::{Value, json};
use surgeist_css::{CssParser, CssRule};

use crate::css::support::csstree::adapters::EntryPoint;
use crate::css::support::csstree::reconciliation::{
    ObservationRoute, PredictedObservation, Reconciliation, RowPaths, namespace_context, text,
};

fn reconciliation() -> Reconciliation {
    Reconciliation::load("at-rule-reconciliation.json", RowPaths::Tests)
}

/// The oracle's snake-case spelling of a recorded variant name, such as
/// `InvalidAtRuleBody` or `DropDescriptor`.
fn recorded_variant(value: &Value) -> String {
    let mut result = String::new();
    for character in value.as_str().unwrap().chars() {
        if character.is_ascii_uppercase() && !result.is_empty() {
            result.push('_');
        }
        result.push(character.to_ascii_lowercase());
    }
    result
}

/// One recorded diagnostic in the oracle's raw form. This file records no
/// payload relation; the harness still classifies every observed diagnostic.
fn predicted_diagnostic(recorded: &Value) -> Value {
    json!({
        "code": recorded_variant(&recorded["error_code"]),
        "action": recorded_variant(&recorded["action"]),
        "byte_offset": recorded["error_byte_offset"],
        "span_start": recorded["recovery_byte_span"][0],
        "span_end": recorded["recovery_byte_span"][1],
        "multiplicity": 1,
    })
}

/// Public single-rule presence for one row.
fn rule_presence_reconciles(row: &Value) -> Result<(), String> {
    let report = CssRule::parse(
        &CssParser::default(),
        text(row, "input"),
        &namespace_context(),
    );
    if report.syntax().is_some() == row["present"].as_bool().unwrap() {
        Ok(())
    } else {
        Err(format!("presence differs: {:?}", report.syntax()))
    }
}

fn assert_parser_group(present: bool, clean: bool, expected_count: usize) {
    // The fixture's registry entry selects its at-rule extractor; each row's
    // expected class has to name the same one.
    let seen = reconciliation().reconcile_observations(
        ObservationRoute {
            entry_point: EntryPoint::Rule,
            extractor: None,
        },
        |row| row["present"] == present && row["clean"] == clean,
        |row| PredictedObservation {
            syntax_count: u64::from(row["present"].as_bool().unwrap()),
            is_clean: row["clean"].as_bool().unwrap(),
            diagnostics: row["diagnostics"]
                .as_array()
                .unwrap()
                .iter()
                .map(predicted_diagnostic)
                .collect(),
        },
        Some(&rule_presence_reconciles),
    );
    assert_eq!(seen, expected_count, "selected original cases");
}

#[test]
fn clean_original_at_rules_retain_one_rule_without_diagnostics() {
    assert_parser_group(true, true, 55);
}

#[test]
fn invalid_original_outer_at_rules_reject_the_complete_input() {
    assert_parser_group(false, false, 66);
}

#[test]
fn original_inner_recovery_retains_outer_rule_and_exact_diagnostics() {
    assert_parser_group(true, false, 8);
}

#[test]
fn semantic_registry_matches_independent_original_at_rule_classes() {
    reconciliation().assert_registry_classes();
}

#[test]
fn original_identity_source_and_active_disposition_are_preserved() {
    let reconciliation = reconciliation();
    let data = reconciliation.data();
    let rows = reconciliation.rows();
    assert_eq!(rows.len(), 129);
    assert_eq!(
        reconciliation.assert_original_bindings(
            rows.iter().chain(std::iter::once(&data["unresolved"])),
            "atrule"
        ),
        130
    );
    // The unresolved singleton retains its original identity and active status;
    // no cleanliness or diagnostic expectation is invented for it.
    assert!(data["unresolved"].get("clean").is_none());
    assert!(data["unresolved"].get("diagnostics").is_none());
}

#[test]
fn unresolved_character_variant_original_retains_font_feature_values_outer_rule() {
    let reconciliation = reconciliation();
    let row = &reconciliation.data()["unresolved"];
    assert_eq!(row["present"], true);
    let report = CssRule::parse(
        &CssParser::default(),
        text(row, "input"),
        &namespace_context(),
    );
    assert!(
        matches!(
            report.syntax(),
            Some(surgeist_css::CssRule::FontFeatureValues(_))
        ),
        "{} must retain its independently valid outer family",
        text(row, "id"),
    );
    // Descriptor cardinality remains disputed: this control deliberately makes
    // no assertion about report cleanliness or its diagnostic sequence.
}
