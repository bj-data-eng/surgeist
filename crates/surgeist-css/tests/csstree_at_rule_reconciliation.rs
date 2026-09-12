//! Original at-rule grammar expectations, independently authored from pinned
//! specifications and the public single-rule recovery contract. The numeric
//! diagnostic tuples characterize API behavior by static source derivation;
//! they are not coordinate requirements imposed by CSS publications.
#![forbid(unsafe_code)]

use std::{collections::BTreeSet, fs, path::Path};

use serde_json::{Value, json};
use surgeist_css::{
    CssErrorCode, CssNamespaceContext, CssNamespaceName, CssNamespacePrefix, CssRecoveryAction,
    parse_rule,
};

#[path = "support/digest.rs"]
mod digest;

fn expectations() -> Value {
    serde_json::from_str(include_str!("csstree/at-rule-reconciliation.json")).unwrap()
}

fn text<'a>(value: &'a Value, key: &str) -> &'a str {
    value[key].as_str().unwrap()
}

fn code_name(code: CssErrorCode) -> &'static str {
    match code {
        CssErrorCode::InvalidAtRuleBody => "InvalidAtRuleBody",
        CssErrorCode::InvalidAtRulePrelude => "InvalidAtRulePrelude",
        CssErrorCode::InvalidMediaQuery => "InvalidMediaQuery",
        CssErrorCode::UnexpectedEnd => "UnexpectedEnd",
        CssErrorCode::UnexpectedToken => "UnexpectedToken",
        CssErrorCode::UnknownAtRule => "UnknownAtRule",
        CssErrorCode::UnknownDescriptor => "UnknownDescriptor",
        other => panic!("unexpected diagnostic code: {other:?}"),
    }
}

fn action_name(action: CssRecoveryAction) -> &'static str {
    match action {
        CssRecoveryAction::DropDescriptor => "DropDescriptor",
        CssRecoveryAction::RejectInput => "RejectInput",
        CssRecoveryAction::ReplaceMediaQueryWithNever => "ReplaceMediaQueryWithNever",
        CssRecoveryAction::RetainWithImplicitClosure => "RetainWithImplicitClosure",
        other => panic!("unexpected recovery action: {other:?}"),
    }
}

fn assert_parser_group(present: bool, clean: bool, expected_count: usize) {
    let data = expectations();
    let context = CssNamespaceContext::from_bindings([(
        Some(CssNamespacePrefix::try_new("ns").unwrap()),
        CssNamespaceName::new("surgeist-corpus-probe"),
    )]);
    let mut seen = 0;
    let mut failures = Vec::new();
    for row in data["rows"]
        .as_array()
        .unwrap()
        .iter()
        .filter(|row| row["present"] == present && row["clean"] == clean)
    {
        seen += 1;
        let report = parse_rule(text(row, "input"), &context);
        let diagnostics: Vec<_> = report
            .diagnostics()
            .iter()
            .map(|diagnostic| {
                json!({
                    "error_code": code_name(diagnostic.error().code()),
                    "error_byte_offset": diagnostic.error().position().byte_offset().value(),
                    "recovery_byte_span": [diagnostic.span().start().byte_offset().value(),
                        diagnostic.span().end().byte_offset().value()],
                    "action": action_name(diagnostic.action()),
                })
            })
            .collect();
        let actual = json!({"present": report.syntax().is_some(), "clean": report.is_clean(),
            "diagnostics": diagnostics});
        let expected = json!({"present": row["present"], "clean": row["clean"],
            "diagnostics": row["diagnostics"]});
        if actual != expected {
            failures.push(format!(
                "{}\nexpected: {expected}\nactual: {actual}",
                text(row, "id")
            ));
        }
    }
    assert_eq!(seen, expected_count, "selected original cases");
    assert!(
        failures.is_empty(),
        "{} of {seen} parser cases differ:\n{}",
        failures.len(),
        failures.join("\n")
    );
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
    let data = expectations();
    let registry: Value =
        serde_json::from_str(include_str!("csstree/expected-classes.json")).unwrap();
    let records = registry["records"].as_array().unwrap();
    let mut failures = Vec::new();
    for row in data["rows"].as_array().unwrap() {
        let id = text(row, "id");
        let record = records.iter().find(|record| record["id"] == id).unwrap();
        assert_eq!(
            record["expectation_sha256"], row["expectation_sha256"],
            "{id}"
        );
        if record["class"] != row["expected_class"] {
            failures.push(format!(
                "{id}\nexpected: {}\nactual: {}",
                row["expected_class"], record["class"]
            ));
        }
    }
    assert!(
        failures.is_empty(),
        "{} of 129 registry classes differ:\n{}",
        failures.len(),
        failures.join("\n")
    );
}

#[test]
fn original_identity_source_and_active_disposition_are_preserved() {
    let data = expectations();
    let rows = data["rows"].as_array().unwrap();
    assert_eq!(rows.len(), 129);
    let mut ids = BTreeSet::new();
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("tests");
    for row in rows.iter().chain(std::iter::once(&data["unresolved"])) {
        let id = text(row, "id");
        assert!(ids.insert(id), "duplicate original {id}");
        let source_bytes = fs::read(root.join(text(row, "source_path"))).unwrap();
        assert_eq!(
            digest::sha256_hex(&source_bytes),
            text(row, "source_sha256"),
            "{id}"
        );
        let neutral_bytes = fs::read(root.join(text(row, "expectation_path"))).unwrap();
        assert_eq!(
            digest::sha256_hex(&neutral_bytes),
            text(row, "expectation_sha256"),
            "{id}"
        );
        let neutral: Value = serde_json::from_slice(&neutral_bytes).unwrap();
        let case = neutral["cases"]
            .as_array()
            .unwrap()
            .iter()
            .find(|case| case["id"] == id)
            .unwrap();
        assert_eq!(case["input"], row["input"], "{id}");
        assert_eq!(case["status"], row["status"], "{id}");
        assert_eq!(case["context"], "atrule", "{id}");
    }
    assert_eq!(ids.len(), 130);
    // The unresolved singleton retains its original identity and active status;
    // no cleanliness or diagnostic expectation is invented for it.
    assert!(data["unresolved"].get("clean").is_none());
    assert!(data["unresolved"].get("diagnostics").is_none());
}

#[test]
fn unresolved_character_variant_original_retains_font_feature_values_outer_rule() {
    let data = expectations();
    let row = &data["unresolved"];
    assert_eq!(row["present"], true);
    let context = CssNamespaceContext::from_bindings([(
        Some(CssNamespacePrefix::try_new("ns").unwrap()),
        CssNamespaceName::new("surgeist-corpus-probe"),
    )]);
    let report = parse_rule(text(row, "input"), &context);
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
