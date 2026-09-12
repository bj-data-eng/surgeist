//! Independently derived MQ5 and Syntax expectations for original CSSTree inputs.
#![forbid(unsafe_code)]

use serde_json::{Value, json};
use std::{collections::BTreeSet, fs, path::Path};
use surgeist_css::{
    CssErrorCode, CssMediaConditionKind, CssMediaQuery, CssRecoveryAction,
    CssUnknownMediaFeatureReason, CssUnknownMediaFeatureRef, parse_media_query,
};

#[path = "support/digest.rs"]
mod digest;

fn expectations() -> Value {
    serde_json::from_str(include_str!("csstree/media-query-reconciliation.json")).unwrap()
}

fn text<'a>(row: &'a Value, key: &str) -> &'a str {
    row[key].as_str().unwrap()
}

#[test]
fn original_media_queries_retain_audited_nodes_and_ordered_diagnostics() {
    let data = expectations();
    for row in data["rows"].as_array().unwrap() {
        let id = text(row, "id");
        let input = text(row, "input");
        let report = parse_media_query(input);
        assert_eq!(report.is_clean(), row["clean"].as_bool().unwrap(), "{id}");
        assert_eq!(
            report.clone().into_validation_result().is_ok(),
            row["clean"].as_bool().unwrap(),
            "{id}: clean validation contract"
        );
        match (text(row, "node"), report.syntax()) {
            ("Never", CssMediaQuery::Never(_)) => {}
            (expected, CssMediaQuery::Condition(condition)) => match (expected, condition.kind()) {
                ("Condition.UnknownFeature", CssMediaConditionKind::UnknownFeature(feature)) => {
                    assert_eq!(feature.name(), text(row, "feature_name"), "{id}");
                    let reason = match text(row, "feature_reason") {
                        "UnknownName" => CssUnknownMediaFeatureReason::UnknownName,
                        "InvalidValue" => CssUnknownMediaFeatureReason::InvalidValue,
                        other => panic!("{id}: unaudited reason {other}"),
                    };
                    assert_eq!(feature.reason(), reason, "{id}");
                    assert_eq!(feature.authored(), Some(input), "{id}");
                    assert_eq!(
                        matches!(feature.view(), CssUnknownMediaFeatureRef::Boolean),
                        row["boolean_feature"].as_bool().unwrap(),
                        "{id}: feature form"
                    );
                }
                ("Condition.GeneralEnclosed", CssMediaConditionKind::GeneralEnclosed(enclosed)) => {
                    assert_eq!(enclosed.authored(), Some(input), "{id}");
                }
                ("Condition.And", CssMediaConditionKind::And(operands)) => {
                    let expected = row["enclosed_operands"].as_array().unwrap();
                    assert_eq!(operands.conditions().len(), expected.len(), "{id}");
                    for (operand, authored) in operands.conditions().iter().zip(expected) {
                        let CssMediaConditionKind::GeneralEnclosed(enclosed) = operand.kind()
                        else {
                            panic!("{id}: expected opaque operand");
                        };
                        assert_eq!(enclosed.authored(), authored.as_str(), "{id}");
                    }
                }
                _ => panic!("{id}: unexpected condition {condition:?}"),
            },
            _ => panic!("{id}: unexpected query {:?}", report.syntax()),
        }
        let actual: Vec<_> = report.diagnostics().iter().map(|diagnostic| {
            let code = match diagnostic.error().code() {
                CssErrorCode::InvalidMediaQuery => "invalid_media_query",
                CssErrorCode::UnexpectedEnd => "unexpected_end",
                other => panic!("{id}: unexpected diagnostic {other:?}"),
            };
            let action = match diagnostic.action() {
                CssRecoveryAction::ReplaceMediaQueryWithNever => "replace_media_query_with_never",
                CssRecoveryAction::RetainWithImplicitClosure => "retain_with_implicit_closure",
                other => panic!("{id}: unexpected action {other:?}"),
            };
            json!({"code":code,"action":action,"byte_offset":diagnostic.error().position().byte_offset().value(),"span_start":diagnostic.span().start().byte_offset().value(),"span_end":diagnostic.span().end().byte_offset().value()})
        }).collect();
        let expected: Vec<_> = row["diagnostics"].as_array().unwrap().iter().map(|d| {
            json!({"code":d["code"],"action":d["action"],"byte_offset":d["byte_offset"],"span_start":d["span_start"],"span_end":d["span_end"]})
        }).collect();
        assert_eq!(actual, expected, "{id}: ordered diagnostics");
    }
}

#[test]
fn original_media_identity_bytes_options_and_digests_are_bound() {
    let data = expectations();
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("tests");
    let rows = data["rows"].as_array().unwrap();
    assert_eq!(rows.len(), 49);
    let mut ids = BTreeSet::new();
    for row in rows {
        let id = text(row, "id");
        assert!(ids.insert(id), "{id}");
        for (path, hash) in [
            ("source_path", "source_sha256"),
            ("expectation_path", "expectation_sha256"),
        ] {
            assert_eq!(
                digest::sha256_hex(&fs::read(root.join(text(row, path))).unwrap()),
                text(row, hash),
                "{id}"
            );
        }
        let neutral: Value =
            serde_json::from_slice(&fs::read(root.join(text(row, "expectation_path"))).unwrap())
                .unwrap();
        let case = neutral["cases"]
            .as_array()
            .unwrap()
            .iter()
            .find(|case| case["id"] == id)
            .unwrap();
        for key in ["input", "context"] {
            assert_eq!(case[key], row[key], "{id}: {key}");
        }
        assert_eq!(
            case.get("options").cloned().unwrap_or_else(|| json!({})),
            row["options"],
            "{id}: original options"
        );
        assert_eq!(case["status"], "active", "{id}");
    }
}

#[test]
fn registry_classes_match_independent_media_query_contracts() {
    let data = expectations();
    let registry: Value =
        serde_json::from_str(include_str!("csstree/expected-classes.json")).unwrap();
    let mut failures = Vec::new();
    for row in data["rows"].as_array().unwrap() {
        let id = text(row, "id");
        let record = registry["records"]
            .as_array()
            .unwrap()
            .iter()
            .find(|r| r["id"] == id)
            .unwrap();
        assert_eq!(
            record["expectation_sha256"], row["expectation_sha256"],
            "{id}"
        );
        if record["class"] != row["expected_class"] {
            failures.push(format!(
                "{id}: expected {}, got {}",
                row["expected_class"], record["class"]
            ));
        }
    }
    assert!(
        failures.is_empty(),
        "{} stale registry classes:\n{}",
        failures.len(),
        failures.join("\n")
    );
}
