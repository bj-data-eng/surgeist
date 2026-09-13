//! Independently specified original single-selector grammar and source-traced
//! diagnostic contracts. Expected values never come from the runtime or registry.
#![forbid(unsafe_code)]

use serde_json::{Value, json};
use std::{collections::BTreeSet, fs, path::Path};
use surgeist_css::{
    CssNamespaceContext, CssNamespaceName, CssNamespacePrefix, CssPseudoClass, CssSelector,
    parse_selector,
};

#[path = "support/digest.rs"]
mod digest;

fn expectations() -> Value {
    serde_json::from_str(include_str!(
        "csstree/selector-original-reconciliation.json"
    ))
    .unwrap()
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

fn pseudo_meaning(pseudo: &CssPseudoClass) -> Value {
    match pseudo {
        CssPseudoClass::Not(list) => {
            let selectors: Vec<_> = list.selectors().iter().map(typed_meaning).collect();
            json!({"pseudo_class": "Not", "selectors": selectors})
        }
        CssPseudoClass::Lang(list) => {
            let ranges: Vec<_> = list
                .ranges()
                .iter()
                .map(|range| {
                    json!({
                        "token_kind": snake_name(range.kind()), "value": range.as_str()
                    })
                })
                .collect();
            json!({"pseudo_class": "Lang", "ranges": ranges})
        }
        other => panic!("unexpected original pseudo: {other:?}"),
    }
}

fn compound_meaning(compound: &surgeist_css::CssCompoundSelector) -> Value {
    assert!(compound.pseudo_elements().is_none());
    assert!(!compound.has_scope_anchor());
    assert_eq!(compound.nesting_selectors(), 0);
    let mut items = Vec::new();
    if let Some(name) = compound.type_selector() {
        items.push(
            json!({"type_selector": {"local_name": name.local_name().unwrap_or("*"),
            "namespace": format!("{:?}", name.namespace())}}),
        );
    }
    items.extend(compound.ids().iter().map(|id| json!({"id": id})));
    items.extend(
        compound
            .classes()
            .iter()
            .map(|class| json!({"class": class})),
    );
    items.extend(compound.attributes().iter().map(|attribute| {
        let (matcher, value) = match attribute.matcher() {
            surgeist_css::CssAttributeMatcher::Exists => ("Exists", None),
            surgeist_css::CssAttributeMatcher::Equals(value) => ("Equals", Some(value.as_str())),
            other => panic!("unexpected original attribute matcher: {other:?}"),
        };
        json!({"attribute": {"local_name": attribute.name().as_str(),
            "namespace": format!("{:?}", attribute.namespace()), "matcher": matcher,
            "value": value, "case_sensitivity": format!("{:?}", attribute.case_sensitivity())}})
    }));
    items.extend(compound.pseudo_classes().iter().map(pseudo_meaning));
    json!({"kind": "compound", "items": items})
}

fn typed_meaning(selector: &CssSelector) -> Value {
    // Normalize only the public compatibility variants. All retained semantic
    // fields are checked, regardless of the parser's equivalent enum projection.
    match selector {
        CssSelector::Tag(name) => json!({"kind": "compound", "items": [
            {"type_selector": {"local_name": name, "namespace": "Any"}}]}),
        CssSelector::Class(name) => json!({"kind": "compound", "items": [{"class": name}]}),
        CssSelector::Key(name) => json!({"kind": "compound", "items": [{"id": name}]}),
        CssSelector::PseudoClass(pseudo) => {
            json!({"kind": "compound", "items": [pseudo_meaning(pseudo)]})
        }
        CssSelector::Compound(compound) => compound_meaning(compound),
        CssSelector::Complex(complex) => {
            let rest: Vec<_> = complex.rest().iter().map(|part| json!({
                "combinator": format!("{:?}", part.combinator()), "selector": compound_meaning(part.selector())
            })).collect();
            json!({"kind": "complex", "first": compound_meaning(complex.first()), "rest": rest})
        }
        other => panic!("unexpected original selector: {other:?}"),
    }
}

#[test]
fn original_selectors_preserve_typed_meaning_and_exact_recovery() {
    let context = CssNamespaceContext::from_bindings([(
        Some(CssNamespacePrefix::try_new("ns").unwrap()),
        CssNamespaceName::new("surgeist-corpus-probe"),
    )]);
    let mut failures = Vec::new();
    for row in expectations()["rows"].as_array().unwrap() {
        let input = text(row, "input");
        let report = parse_selector(input, &context);
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
        let actual = json!({"present": report.syntax().is_some(), "clean": report.is_clean(),
            "diagnostics": diagnostics});
        let expected = json!({"present": row["expected"]["semantic_acceptance"],
            "clean": row["expected"]["is_clean"], "diagnostics": row["expected_raw_diagnostics"]});
        if actual != expected {
            failures.push(format!(
                "{}\nexpected: {expected}\nactual: {actual}",
                row["id"]
            ));
        }
        if let Some(selector) = report.syntax() {
            let actual = typed_meaning(selector);
            if actual != row["expected"]["typed_expectation"] {
                failures.push(format!(
                    "{}: expected meaning {}, actual {actual}",
                    row["id"], row["expected"]["typed_expectation"]
                ));
            }
        }
    }
    assert!(failures.is_empty(), "{}", failures.join("\n"));
}

#[test]
fn original_selector_registry_matches_independent_complete_original_classes() {
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
fn original_selector_identity_options_and_selected_publication_remain_bound() {
    let data = expectations();
    let rows = data["rows"].as_array().unwrap();
    assert_eq!(rows.len(), 14);
    let root = Path::new(env!("CARGO_MANIFEST_DIR"));
    let catalog: Value =
        serde_json::from_slice(&fs::read(root.join("specs/catalog.json")).unwrap()).unwrap();
    // Historical whole-catalog hashes remain provenance. Bind the selected
    // publications without coupling to unrelated normative-definition additions.
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
        for key in ["input", "context", "status"] {
            assert_eq!(original[key], row[key], "{id}: {key}");
        }
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
