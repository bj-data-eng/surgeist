//! Independently specified original remaining pseudo grammar and source-traced
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
    serde_json::from_str(include_str!("csstree/remaining-pseudo-reconciliation.json")).unwrap()
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

fn compound_argument(argument: &surgeist_css::CssCompoundSelectorArgument) -> Value {
    let compound = argument.compound();
    assert!(compound.ids().is_empty());
    assert!(compound.attributes().is_empty());
    assert!(compound.pseudo_classes().is_empty());
    assert!(compound.pseudo_elements().is_none());
    assert!(!compound.has_scope_anchor());
    assert_eq!(compound.nesting_selectors(), 0);
    let mut members = Vec::new();
    if let Some(name) = compound.type_selector() {
        members.push(json!({"type_selector": name.local_name().unwrap(),
            "namespace": format!("{:?}", name.namespace())}));
    }
    members.extend(compound.classes().iter().map(|name| json!({"class": name})));
    json!(members)
}

fn typed_meaning(selector: &CssSelector) -> Value {
    match selector {
        CssSelector::PseudoClass(CssPseudoClass::Dir(direction)) => {
            json!({"pseudo_class": "Dir", "identifier": direction.as_str()})
        }
        CssSelector::PseudoClass(CssPseudoClass::Lang(languages)) => {
            let ranges: Vec<_> = languages.ranges().iter().map(|range| {
                json!({"token_kind": snake_name(range.kind()), "value": range.as_str()})
            }).collect();
            json!({"pseudo_class": "Lang", "ranges": ranges})
        }
        CssSelector::PseudoClass(CssPseudoClass::Has(list)) => {
            let selectors: Vec<_> = list
                .selectors()
                .iter()
                .map(|relative| {
                    let CssSelector::Class(name) = relative.selector() else {
                        panic!("expected original single-class relative selector: {relative:?}");
                    };
                    json!({"leading_combinator": format!("{:?}", relative.combinator()),
                    "compound": [{"class": name}]})
                })
                .collect();
            json!({"pseudo_class": "Has", "relative_selectors": selectors})
        }
        CssSelector::PseudoClass(CssPseudoClass::Host) => {
            json!({"pseudo_class": "host", "functional": false, "compound_argument": null})
        }
        CssSelector::PseudoClass(CssPseudoClass::HostFunction(argument)) => {
            json!({"pseudo_class": "host", "functional": true, "compound_argument": compound_argument(argument)})
        }
        CssSelector::PseudoClass(CssPseudoClass::HostContext(argument)) => {
            json!({"pseudo_class": "host-context", "functional": true, "compound_argument": compound_argument(argument)})
        }
        CssSelector::Compound(compound) => {
            assert!(compound.type_selector().is_none());
            assert!(compound.ids().is_empty());
            assert!(compound.classes().is_empty());
            assert!(compound.attributes().is_empty());
            assert!(compound.pseudo_classes().is_empty());
            assert!(!compound.has_scope_anchor());
            assert_eq!(compound.nesting_selectors(), 0);
            let [
                surgeist_css::CssPseudoElementSegment::PseudoElement(
                    surgeist_css::CssPseudoElement::Slotted(argument),
                ),
            ] = compound.pseudo_elements().unwrap().segments()
            else {
                panic!("expected sole slotted pseudo-element");
            };
            json!({"pseudo_element": "slotted", "functional": true, "compound_argument": compound_argument(argument)})
        }
        other => panic!("unexpected retained original pseudo: {other:?}"),
    }
}

#[test]
fn original_remaining_pseudos_preserve_typed_meaning_and_exact_recovery() {
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
fn remaining_registry_matches_independent_complete_original_classes() {
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
fn original_remaining_identity_options_and_selected_publication_remain_bound() {
    let data = expectations();
    let rows = data["rows"].as_array().unwrap();
    assert_eq!(rows.len(), 75);
    let root = Path::new(env!("CARGO_MANIFEST_DIR"));
    let catalog: Value =
        serde_json::from_slice(&fs::read(root.join("specs/catalog.json")).unwrap()).unwrap();
    let publication = &catalog["modules"]
        .as_array()
        .unwrap()
        .iter()
        .find(|module| module["id"] == "selectors-4")
        .unwrap()["publication"];
    // The saved whole-catalog digest is historical provenance. Bind this test
    // to its actual normative edition, allowing unrelated catalog maintenance.
    for key in ["url", "sha256", "bytes", "date", "kind"] {
        assert_eq!(
            publication[key], data["diagnostic_audits"]["nonshadow"]["normative_source"][key],
            "selected publication {key}"
        );
    }
    for (name, source) in data["diagnostic_audits"]["shadow"]["normative_sources"]
        .as_object()
        .unwrap()
    {
        let actual = if name == "selectors" {
            publication
        } else {
            catalog["normative_definitions"]["sources"]
                .as_array()
                .unwrap()
                .iter()
                .find(|actual| actual["id"] == source["id"])
                .unwrap()
        };
        for key in ["url", "sha256", "bytes", "date", "kind", "revision", "path"] {
            if let Some(value) = source.get(key) {
                assert_eq!(&actual[key], value, "{name} source {key}");
            }
        }
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
