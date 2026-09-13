//! Independently specified original declaration-list grammar and source-traced
//! diagnostic contracts. Expected values never come from the runtime or registry.
#![forbid(unsafe_code)]

use serde_json::{Value, json};
use std::{collections::BTreeSet, fs, path::Path};
use surgeist_css::{
    CssColor, CssDeclarationList, CssKnownPropertyValueRef, parse_style_attribute,
    validate_style_attribute,
};

#[path = "support/digest.rs"]
mod digest;

fn expectations() -> Value {
    serde_json::from_str(include_str!("csstree/declaration-list-reconciliation.json")).unwrap()
}

fn text<'a>(row: &'a Value, key: &str) -> &'a str {
    row[key].as_str().unwrap()
}

fn snake_name(value: impl std::fmt::Debug) -> String {
    let mut result = String::new();
    for character in format!("{value:?}").chars() {
        if character.is_ascii_uppercase() && !result.is_empty() {
            result.push('_');
        }
        result.push(character.to_ascii_lowercase());
    }
    result
}

fn declarations(list: &CssDeclarationList) -> Value {
    let declarations: Vec<_> = list
        .iter()
        .map(|declaration| {
            let Some(CssKnownPropertyValueRef::Color(value)) =
                declaration.known().and_then(|known| known.property_value())
            else {
                panic!("unexpected retained declaration: {declaration:?}");
            };
            let Some(CssColor::Rgba(color)) = value.i01_subset() else {
                panic!("expected independently specified RGBA color");
            };
            json!({"property": "color", "typed_value": {"kind": "rgba", "red": color.red(),
            "green": color.green(), "blue": color.blue(), "alpha": color.alpha()},
            "importance": snake_name(declaration.importance())})
        })
        .collect();
    json!(declarations)
}

#[test]
fn original_declaration_lists_preserve_raw_grammar_typed_values_and_exact_recovery() {
    let mut failures = Vec::new();
    for row in expectations()["rows"].as_array().unwrap() {
        let input = text(row, "input");
        let report = parse_style_attribute(input);
        let diagnostics: Vec<_> = report
            .diagnostics()
            .iter()
            .map(|diagnostic| {
                let byte_offset = diagnostic.error().position().byte_offset().value();
                let span_start = diagnostic.span().start().byte_offset().value();
                let span_end = diagnostic.span().end().byte_offset().value();
                let relation = if byte_offset < input.len() {
                    "intersects"
                } else if span_start == input.len() && span_end == input.len() {
                    "ends_at"
                } else {
                    "recovery_ends_at"
                };
                json!({"code": snake_name(diagnostic.error().code()),
                "action": snake_name(diagnostic.action()), "byte_offset": byte_offset,
                "span_start": span_start, "span_end": span_end,
                "multiplicity": 1, "payload_relation": relation})
            })
            .collect();
        let actual = json!({"declarations": declarations(report.syntax()),
            "declaration_count": report.syntax().len(), "clean": report.is_clean(),
            "validation_accepts": validate_style_attribute(input).is_ok(), "diagnostics": diagnostics});
        let expected = json!({"declarations": row["expected"]["ordered_declarations"],
            "declaration_count": row["expected"]["declaration_count"], "clean": row["expected"]["is_clean"],
            "validation_accepts": row["expected"]["validation_accepts"], "diagnostics": row["expected_raw_diagnostics"]});
        if actual != expected {
            failures.push(format!(
                "{}\nexpected: {expected}\nactual: {actual}",
                row["id"]
            ));
        }
    }
    assert!(failures.is_empty(), "{}", failures.join("\n"));
}

#[test]
fn declaration_list_registry_matches_independent_complete_original_classes() {
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
        if record["class"] != row["expected_class"] {
            failures.push(format!(
                "{}\nexpected: {}\nactual: {}",
                row["id"], row["expected_class"], record["class"]
            ));
        }
    }
    assert!(
        failures.is_empty(),
        "{} full registry classes differ:\n{}",
        failures.len(),
        failures.join("\n")
    );
}

#[test]
fn original_declaration_list_identity_options_and_selected_publications_remain_bound() {
    let data = expectations();
    let rows = data["rows"].as_array().unwrap();
    assert_eq!(rows.len(), 19);
    let root = Path::new(env!("CARGO_MANIFEST_DIR"));
    let catalog: Value =
        serde_json::from_slice(&fs::read(root.join("specs/catalog.json")).unwrap()).unwrap();
    // Keep the original whole-catalog digest as provenance; unrelated catalog
    // additions do not alter these selected normative editions.
    for source in data["normative_sources"].as_array().unwrap() {
        let module = catalog["modules"]
            .as_array()
            .unwrap()
            .iter()
            .find(|module| module["id"] == source["id"])
            .unwrap();
        assert_eq!(
            module["publication"], source["publication"],
            "{}",
            source["id"]
        );
    }
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
            digest::sha256_hex(&serde_json::to_vec(&row["effective_options"]).unwrap()),
            text(row, "options_sha256"),
            "{id}"
        );
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
        let original = neutral["cases"]
            .as_array()
            .unwrap()
            .iter()
            .find(|case| case["id"] == id)
            .unwrap();
        assert_eq!(original, &row["original_expectation"], "{id}");
        assert_eq!(original["input"], row["input"], "{id}");
        assert_eq!(original["context"], "declarationList", "{id}");
        assert_eq!(original["status"], "active", "{id}");
        assert_eq!(
            original.get("options").is_some(),
            row["options_present"].as_bool().unwrap(),
            "{id}"
        );
        assert_eq!(original["options"], row["options"], "{id}");
        assert_eq!(
            original
                .get("options")
                .cloned()
                .unwrap_or_else(|| json!({})),
            row["effective_options"],
            "{id}"
        );
    }
}
