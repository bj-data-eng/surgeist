//! Original stylesheet expectations from an independent pinned-publication audit.
//! Grammar retention, public recovery policy, and registry classification are
//! separate assertions. Unsettled diagnostic details are deliberately not inferred
//! from parser output or the captured corpus oracle.
#![forbid(unsafe_code)]

use std::{collections::BTreeSet, fs, path::Path};

use serde_json::{Value, json};
use surgeist_css::{
    CssColor, CssDeclarationList, CssImportance, CssKnownPropertyValueRef, CssLength, CssRgbaColor,
    CssRule, CssSelector, parse_sheet, validate_sheet,
};

#[path = "support/digest.rs"]
mod digest;

fn expectations() -> Value {
    serde_json::from_str(include_str!("csstree/stylesheet-reconciliation.json")).unwrap()
}

fn text<'a>(value: &'a Value, key: &str) -> &'a str {
    value[key].as_str().unwrap()
}

fn declarations(actual: &CssDeclarationList) -> Value {
    Value::Array(actual.iter().map(|declaration| {
        assert_eq!(declaration.importance(), CssImportance::Normal);
        match declaration.known().and_then(|known| known.property_value()) {
            Some(CssKnownPropertyValueRef::Color(value)) => {
                assert_eq!(value.i01_subset(), Some(&CssColor::Rgba(
                    CssRgbaColor::try_new(255, 0, 0, 1.0).unwrap()
                )));
                json!(["color", "red"])
            }
            Some(CssKnownPropertyValueRef::Width(value)) => {
                assert!(matches!(value.i01_subset(), Some(CssLength::Px(px)) if px.value() == 10.0));
                json!(["width", "10px"])
            }
            other => panic!("unexpected retained declaration: {other:?}"),
        }
    }).collect())
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
            let condition = media.query().serialize().unwrap().as_css().replace(' ', "");
            json!({"kind": "media", "condition": condition, "rules": rules(media.rules())})
        }
        CssRule::Page(page) => {
            assert!(page.selector().is_none());
            json!({"kind": "page", "selector": null, "declarations": declarations(page.declarations())})
        }
        other => panic!("unexpected retained rule: {other:?}"),
    }).collect())
}

fn actions_match(actual: &[Value], row: &Value) -> bool {
    let expected = row["expected"]["ordered_recovery_actions"]
        .as_array()
        .unwrap();
    if row["diagnostic_action_relation"] == "exact" {
        actual == expected
    } else {
        let mut remaining = actual.iter();
        expected
            .iter()
            .all(|action| remaining.any(|candidate| candidate == action))
    }
}

#[test]
fn original_stylesheets_retain_independently_expected_ordered_rules_and_values() {
    for row in expectations()["rows"].as_array().unwrap() {
        let report = parse_sheet(text(row, "input"));
        assert!(report.syntax().encoding().is_none(), "{}", row["id"]);
        assert_eq!(
            rules(report.syntax().rules()),
            row["expected"]["ordered_rules"],
            "{}",
            row["id"]
        );
        assert_eq!(
            report.syntax().rules().len(),
            row["expected"]["top_level_rule_count"].as_u64().unwrap() as usize,
            "{}",
            row["id"]
        );
    }
}

#[test]
fn original_stylesheet_recovery_and_validation_follow_independent_requirements() {
    let mut failures = Vec::new();
    for row in expectations()["rows"].as_array().unwrap() {
        let input = text(row, "input");
        let report = parse_sheet(input);
        let actions: Vec<_> = report
            .diagnostics()
            .iter()
            .map(|diagnostic| json!(format!("{:?}", diagnostic.action())))
            .collect();
        let expected = &row["expected"];
        if json!(report.is_clean()) != expected["is_clean"]
            || json!(validate_sheet(input).is_ok()) != expected["validation_accepts"]
            || !actions_match(&actions, row)
        {
            failures.push(format!(
                "{}: clean={}, actions={actions:?}",
                row["id"],
                report.is_clean()
            ));
        }
        if !expected["diagnostic_positions"].is_null() {
            let positions: Vec<_> = report
                .diagnostics()
                .iter()
                .map(|diagnostic| diagnostic.error().position().byte_offset().value())
                .collect();
            if json!(positions) != expected["diagnostic_positions"] {
                failures.push(format!("{}: positions={positions:?}", row["id"]));
            }
        }
    }
    assert!(failures.is_empty(), "{}", failures.join("\n"));
}

#[test]
fn stylesheet_registry_satisfies_independently_settled_class_and_recovery_constraints() {
    let registry: Value =
        serde_json::from_str(include_str!("csstree/expected-classes.json")).unwrap();
    let mut failures = Vec::new();
    for row in expectations()["rows"].as_array().unwrap() {
        let record = registry["records"]
            .as_array()
            .unwrap()
            .iter()
            .find(|record| record["id"] == row["id"])
            .unwrap();
        assert_eq!(record["expectation_sha256"], row["expectation_sha256"]);
        let class = &record["class"];
        let count = row["expected"]["top_level_rule_count"].as_u64().unwrap();
        let predicate = &class["retained_syntax"]["predicate"];
        let count_matches = match predicate["relation"].as_str() {
            Some("exact") => predicate["value"] == count,
            Some("empty") => count == 0,
            Some("nonempty") => count > 0,
            _ => false,
        };
        let actions: Vec<_> = class["diagnostics"]
            .as_array()
            .map(|diagnostics| {
                diagnostics
                    .iter()
                    .map(|diagnostic| {
                        // Registry serde names are snake_case; public action names in the
                        // independent audit follow the Rust variants.
                        let action: String = text(diagnostic, "action")
                            .split('_')
                            .map(|word| {
                                let mut chars = word.chars();
                                let first = chars.next().unwrap().to_ascii_uppercase();
                                format!("{first}{}", chars.as_str())
                            })
                            .collect();
                        json!(action)
                    })
                    .collect()
            })
            .unwrap_or_default();
        if class["kind"] != row["registry_kind"]
            || !count_matches
            || class["retained_syntax"]["extractor"] != json!({"kind": "sheet_rules"})
            || !actions_match(&actions, row)
        {
            failures.push(format!(
                "{}: expected kind={}, count={count}, actions={}; actual={class}",
                row["id"], row["registry_kind"], row["expected"]["ordered_recovery_actions"]
            ));
        }
    }
    assert!(
        failures.is_empty(),
        "{} registry constraints differ:\n{}",
        failures.len(),
        failures.join("\n")
    );
}

#[test]
fn original_stylesheet_identity_input_options_and_source_hashes_are_preserved() {
    let data = expectations();
    let rows = data["rows"].as_array().unwrap();
    assert_eq!(rows.len(), 76);
    let root = Path::new(env!("CARGO_MANIFEST_DIR"));
    assert_eq!(
        digest::sha256_hex(&fs::read(root.join("specs/catalog.json")).unwrap()),
        text(&data, "catalog_sha256")
    );
    let mut ids = BTreeSet::new();
    for row in rows {
        let id = text(row, "id");
        assert!(ids.insert(id), "duplicate original {id}");
        assert_eq!(
            digest::sha256_hex(text(row, "input").as_bytes()),
            text(row, "input_sha256"),
            "{id}"
        );
        assert_eq!(
            digest::sha256_hex(&serde_json::to_vec(&row["options"]).unwrap()),
            text(row, "options_sha256"),
            "{id}"
        );
        let source = fs::read(root.join(text(row, "source_path"))).unwrap();
        assert_eq!(
            digest::sha256_hex(&source),
            text(row, "source_sha256"),
            "{id}"
        );
        let neutral = fs::read(root.join(text(row, "expectation_path"))).unwrap();
        assert_eq!(
            digest::sha256_hex(&neutral),
            text(row, "expectation_sha256"),
            "{id}"
        );
        let neutral: Value = serde_json::from_slice(&neutral).unwrap();
        let original = neutral["cases"]
            .as_array()
            .unwrap()
            .iter()
            .find(|case| case["id"] == id)
            .unwrap();
        assert_eq!(original, &row["original_expectation"], "{id}");
        assert_eq!(original["context"], "stylesheet", "{id}");
        assert_eq!(original["status"], "active", "{id}");
        assert_eq!(original["input"], row["input"], "{id}");
        assert_eq!(
            original
                .get("options")
                .cloned()
                .unwrap_or_else(|| json!({})),
            row["options"],
            "{id}"
        );
    }
}
