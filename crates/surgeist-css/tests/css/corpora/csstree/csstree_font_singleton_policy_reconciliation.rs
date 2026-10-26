//! Original corpus behavior under selected Fonts4 section 6.9.2/frozen WebKit.
//! The normative section 6.9.1 / 6.9.2 conflict remains unresolved.

use crate::css::support::descriptor_phases;
use descriptor_phases::*;

use serde_json::{Value, json};
use surgeist_css::{
    CssErrorCode, CssFontFeatureValuesItem, CssNamespaceContext, CssParser, CssRule,
    CssSourcePosition, CssSourceSpan,
};

use crate::css::support::csstree::adapters::{EntryPoint, Extractor};
use crate::css::support::csstree::reconciliation::{
    ObservationRoute, PredictedObservation, Reconciliation, RowPaths, catalog_module, text,
};

fn reconciliation() -> Reconciliation {
    Reconciliation::load("font-singleton-policy-reconciliation.json", RowPaths::Crate)
}

fn position(value: CssSourcePosition) -> Value {
    json!({"byte_offset": value.byte_offset().value(), "line": value.line().value(),
        "column": value.column().value()})
}

fn span(value: CssSourceSpan) -> Value {
    json!({"start": position(value.start()), "end": position(value.end())})
}

/// The retained typed definitions, presence, cleanliness, validation and
/// detailed descriptor diagnostics for the singleton row. The payload relation
/// of each diagnostic is reconciled once, through `expected_raw_diagnostics`.
fn typed_definitions_reconcile(row: &Value) -> Result<(), String> {
    let input = text(row, "input");
    let report = CssRule::parse(
        &CssParser::default(),
        input,
        &CssNamespaceContext::default(),
    );
    assert_eq!(report.syntax().is_some(), row["present"].as_bool().unwrap());
    assert_eq!(report.is_clean(), row["clean"].as_bool().unwrap());
    assert_eq!(
        report.clone().into_validation_result().is_ok(),
        row["validator_accepts"].as_bool().unwrap()
    );
    let Some(CssRule::FontFeatureValues(rule)) = report.syntax() else {
        panic!("original outer font-feature-values rule must survive")
    };
    let items: Vec<_> = rule
        .items()
        .iter()
        .map(|item| {
            let CssFontFeatureValuesItem::Block(block) = item else {
                panic!("original source contains only subsidiary blocks")
            };
            let definitions: Vec<_> = block
                .definitions()
                .iter()
                .map(|definition| {
                    let indexes: Vec<_> = definition
                        .ordinary_indexes()
                        .iter()
                        .map(|index| {
                            let origin = index.origin().expect("authored integer origin");
                            let range = origin.span();
                            let source = origin.source().as_str();
                            let authored = &source[range.start().byte_offset().value()
                                ..range.end().byte_offset().value()];
                            json!({"decimal": index.as_decimal_str(), "origin": {
                                "original_source": source, "span": span(range), "authored": authored
                            }})
                        })
                        .collect();
                    json!({"name": definition.name().as_str(),
                        "position": position(definition.position().expect("definition position")),
                        "indexes": indexes})
                })
                .collect();
            json!({"variant": "Block", "kind": format!("{:?}", block.kind()),
                "position": position(block.position().expect("block position")),
                "definitions": definitions})
        })
        .collect();
    let actual = json!({"variant": "CssRule::FontFeatureValues",
        "position": position(rule.position().expect("rule position")),
        "families": rule.families().iter().map(|family| family.as_str()).collect::<Vec<_>>(),
        "items": items});
    assert_eq!(actual, row["typed_retention"]);
    let diagnostics: Vec<_> = report
        .diagnostics()
        .iter()
        .map(|diagnostic| {
            let error = diagnostic.error();
            let detail = &error;
            assert_eq!(detail.code(), CssErrorCode::InvalidDescriptorValue, "expected descriptor grammar failure: {error:?}");
            let encountered = detail.encountered().expect("responsible authored identifier");
            let start = diagnostic.span().start().byte_offset().value();
            let end = diagnostic.span().end().byte_offset().value();
            json!({"error_code": format!("{:?}", error.code()),
                "error_byte_offset": error.position().byte_offset().value(),
                "error_position": position(error.position()), "recovery_byte_span": [start, end],
                "recovery_span": span(diagnostic.span()), "action": format!("{:?}", diagnostic.action()),
                "detail": {
                    "at_rule": detail.at_rule().unwrap().as_str(), "descriptor": detail.descriptor().unwrap().as_str(),
                    "expectation": detail.expectation().unwrap().as_str(),
                    "encountered": {"kind": format!("{:?}", encountered.kind()), "authored": encountered.authored()}
                }, "recovered_source": &input[start..end]})
        })
        .collect();
    let detailed = row["diagnostics"].as_array().unwrap();
    let raw = row["expected_raw_diagnostics"].as_array().unwrap();
    assert_eq!(
        detailed.len(),
        raw.len(),
        "one raw form per detailed diagnostic"
    );
    let expected: Vec<_> = detailed
        .iter()
        .zip(raw)
        .map(|(detailed, raw)| {
            let mut detailed = detailed.clone();
            let relation = detailed.as_object_mut().unwrap().remove("payload_relation");
            assert_eq!(relation.as_ref(), Some(&raw["payload_relation"]));
            detailed
        })
        .collect();
    assert_eq!(diagnostics, expected);
    Ok(())
}

#[test]
fn original_singleton_retains_all_typed_definitions_without_recovery() {
    let seen = reconciliation().reconcile_observations(
        ObservationRoute {
            entry_point: EntryPoint::Rule,
            extractor: Some(Extractor::AtRule),
        },
        |_| true,
        |row| PredictedObservation {
            syntax_count: 1,
            is_clean: row["clean"].as_bool().unwrap(),
            diagnostics: row["expected_raw_diagnostics"].as_array().unwrap().clone(),
        },
        Some(&typed_definitions_reconcile),
    );
    assert_eq!(seen, 1);
}

#[test]
fn original_singleton_registry_matches_independent_adopted_policy_class() {
    reconciliation().assert_registry_classes();
}

#[test]
fn original_singleton_identity_options_and_selected_publication_remain_bound() {
    let reconciliation = reconciliation();
    let data = reconciliation.data();
    let rows = reconciliation.rows();
    assert_eq!(rows.len(), 1);
    for source in data["normative_sources"].as_array().unwrap() {
        let module = catalog_module(&source["id"]);
        assert_eq!(module["publication"], source["publication"]);
        assert_eq!(module["authored_scope"], source["authored_scope"]);
        assert_eq!(module["selection"], source["selection"]);
        assert_eq!(
            module["publication"]["sha256"],
            data["source_policy"]["catalog_publication_sha256"]
        );
        assert_eq!(
            module["publication"]["url"],
            data["source_policy"]["publication_url"]
        );
    }
    assert_eq!(reconciliation.assert_original_bindings(rows, "atrule"), 1);
    let row = &rows[0];
    let source = reconciliation.row_json(row, "source_path");
    let neutral = reconciliation.row_json(row, "expectation_path");
    let original = neutral["cases"]
        .as_array()
        .unwrap()
        .iter()
        .find(|case| case["id"] == row["id"])
        .unwrap();
    assert_eq!(source[text(original, "label")]["source"], row["input"]);
}
