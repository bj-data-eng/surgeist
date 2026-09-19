//! Original corpus behavior under the documented provisional Fonts4 policy.
//! The normative section 6.9.1 / 6.9.2 conflict remains unresolved.
#![forbid(unsafe_code)]

use serde_json::{Value, json};
use std::{fs, path::Path};
use surgeist_css::{
    CssFontFeatureValuesItem, CssNamespaceContext, CssRule, CssSourcePosition, CssSourceSpan,
    ErrorKind, parse_rule,
};

#[path = "support/digest.rs"]
mod digest;

fn expectations() -> Value {
    serde_json::from_str(include_str!(
        "csstree/font-singleton-policy-reconciliation.json"
    ))
    .unwrap()
}

fn text<'a>(row: &'a Value, key: &str) -> &'a str {
    row[key].as_str().unwrap()
}

fn position(value: CssSourcePosition) -> Value {
    json!({"byte_offset": value.byte_offset().value(), "line": value.line().value(),
        "column": value.column().value()})
}

fn span(value: CssSourceSpan) -> Value {
    json!({"start": position(value.start()), "end": position(value.end())})
}

#[test]
fn original_singleton_retains_typed_blocks_and_exact_policy_recovery() {
    let data = expectations();
    let row = &data["rows"][0];
    let input = text(row, "input");
    let report = parse_rule(input, &CssNamespaceContext::default());
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
                        .indexes()
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
            let ErrorKind::InvalidDescriptorValue(detail) = error.kind() else {
                panic!("expected descriptor grammar failure: {error:?}")
            };
            let encountered = detail.encountered().expect("responsible authored identifier");
            let start = diagnostic.span().start().byte_offset().value();
            let end = diagnostic.span().end().byte_offset().value();
            assert!(error.position().byte_offset().value() < input.len());
            assert!(start < input.len() && end > 0);
            json!({"error_code": format!("{:?}", error.code()),
                "error_byte_offset": error.position().byte_offset().value(),
                "error_position": position(error.position()), "recovery_byte_span": [start, end],
                "recovery_span": span(diagnostic.span()), "action": format!("{:?}", diagnostic.action()),
                "payload_relation": "intersects", "detail": {
                    "at_rule": detail.at_rule().as_str(), "descriptor": detail.descriptor().as_str(),
                    "expectation": detail.expectation().as_str(),
                    "encountered": {"kind": format!("{:?}", encountered.kind()), "authored": encountered.authored()}
                }, "recovered_source": &input[start..end]})
        })
        .collect();
    assert_eq!(diagnostics, *row["diagnostics"].as_array().unwrap());
}

#[test]
fn original_singleton_registry_matches_independent_adopted_policy_class() {
    let data = expectations();
    let row = &data["rows"][0];
    let registry: Value =
        serde_json::from_str(include_str!("csstree/expected-classes.json")).unwrap();
    let records: Vec<_> = registry["records"]
        .as_array()
        .unwrap()
        .iter()
        .filter(|record| record["id"] == row["id"])
        .collect();
    assert_eq!(records.len(), 1);
    assert_eq!(records[0]["expectation_sha256"], row["expectation_sha256"]);
    assert_eq!(records[0]["class"], row["expected_class"]);
}

#[test]
fn original_singleton_identity_options_and_selected_publication_remain_bound() {
    let data = expectations();
    let rows = data["rows"].as_array().unwrap();
    assert_eq!(rows.len(), 1);
    let row = &rows[0];
    let root = Path::new(env!("CARGO_MANIFEST_DIR"));
    let catalog: Value =
        serde_json::from_slice(&fs::read(root.join("specs/catalog.json")).unwrap()).unwrap();
    for source in data["normative_sources"].as_array().unwrap() {
        let module = catalog["modules"]
            .as_array()
            .unwrap()
            .iter()
            .find(|module| module["id"] == source["id"])
            .unwrap();
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
    assert_eq!(
        digest::sha256_hex(text(row, "input").as_bytes()),
        text(row, "input_sha256")
    );
    assert_eq!(
        digest::sha256_hex(&serde_json::to_vec(&row["effective_options"]).unwrap()),
        text(row, "options_sha256")
    );
    for (path, hash) in [
        ("source_path", "source_sha256"),
        ("expectation_path", "expectation_sha256"),
    ] {
        assert_eq!(
            digest::sha256_hex(&fs::read(root.join(text(row, path))).unwrap()),
            text(row, hash)
        );
    }
    let neutral: Value =
        serde_json::from_slice(&fs::read(root.join(text(row, "expectation_path"))).unwrap())
            .unwrap();
    let originals: Vec<_> = neutral["cases"]
        .as_array()
        .unwrap()
        .iter()
        .filter(|case| case["id"] == row["id"])
        .collect();
    assert_eq!(originals.len(), 1);
    let original = originals[0];
    assert_eq!(original, &row["original_expectation"]);
    for key in ["input", "context", "status"] {
        assert_eq!(original[key], row[key]);
    }
    assert_eq!(
        original.get("options").is_some(),
        row["options_present"].as_bool().unwrap()
    );
    assert_eq!(original["options"], row["options"]);
    assert_eq!(
        original
            .get("options")
            .cloned()
            .unwrap_or_else(|| json!({})),
        row["effective_options"]
    );
    let source: Value =
        serde_json::from_slice(&fs::read(root.join(text(row, "source_path"))).unwrap()).unwrap();
    assert_eq!(source[text(original, "label")]["source"], row["input"]);
}
