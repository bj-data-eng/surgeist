//! Independent Selectors 4 logical-pseudo expectations for the 45 original
//! CSSTree records. The fixture cites pinned grammar and diagnostic source
//! derivations; neither the current parser nor registry supplies its oracle.
#![forbid(unsafe_code)]

use serde_json::{Value, json};
use std::{collections::BTreeSet, fs, path::Path};
use surgeist_css::{
    CssErrorCode, CssNamespaceContext, CssNamespaceName, CssNamespacePrefix, CssPseudoClass,
    CssRecoveryAction, CssSelector, parse_selector,
};
#[path = "support/digest.rs"]
mod digest;
fn expectations() -> Value {
    serde_json::from_str(include_str!("csstree/logical-selector-reconciliation.json")).unwrap()
}
fn text<'a>(row: &'a Value, key: &str) -> &'a str {
    row[key].as_str().unwrap()
}

fn assert_group(present: bool, clean: bool, count: usize) {
    let data = expectations();
    let context = CssNamespaceContext::from_bindings([(
        Some(CssNamespacePrefix::try_new("ns").unwrap()),
        CssNamespaceName::new("surgeist-corpus-probe"),
    )]);
    let mut seen = 0;
    for row in data["rows"]
        .as_array()
        .unwrap()
        .iter()
        .filter(|r| r["present"] == present && r["clean"] == clean)
    {
        seen += 1;
        let id = text(row, "id");
        let report = parse_selector(text(row, "input"), &context);
        assert_eq!(report.syntax().is_some(), present, "{id}");
        assert_eq!(report.is_clean(), clean, "{id}");
        let actual: Vec<_> = report.diagnostics().iter().map(|d| {
            assert_eq!(d.error().code(), CssErrorCode::InvalidSelector, "{id}");
            let action = match d.action() { CssRecoveryAction::RejectInput => "reject_input", CssRecoveryAction::DropSelectorListItem => "drop_selector_list_item", other => panic!("{id}: unexpected {other:?}") };
            json!({"code":"invalid_selector", "action":action,"byte_offset":d.error().position().byte_offset().value(),"span_start":d.span().start().byte_offset().value(),"span_end":d.span().end().byte_offset().value()})
        }).collect();
        let expected: Vec<_> = row["diagnostics"].as_array().unwrap().iter().map(|d|json!({"code":d["code"],"action":d["action"],"byte_offset":d["byte_offset"],"span_start":d["span_start"],"span_end":d["span_end"]})).collect();
        assert_eq!(actual, expected, "{id}");
        if let Some(members) = row["expected_inner_members_when_recovered"].as_array() {
            let Some(CssSelector::PseudoClass(
                CssPseudoClass::Is(list) | CssPseudoClass::Where(list),
            )) = report.syntax()
            else {
                panic!("{id}: retained forgiving pseudo")
            };
            let expected: Vec<_> = members
                .iter()
                .map(|v| CssSelector::Class(v.as_str().unwrap().strip_prefix('.').unwrap().into()))
                .collect();
            assert_eq!(list.selectors(), expected, "{id}: actual surviving members");
        }
    }
    assert_eq!(seen, count);
}
#[test]
fn clean_original_logical_selectors() {
    assert_group(true, true, 17);
}
#[test]
fn recovered_original_logical_selectors_retain_meaningful_members() {
    assert_group(true, false, 10);
}
#[test]
fn invalid_original_logical_selectors_reject_whole_input() {
    assert_group(false, false, 18);
}

#[test]
fn original_identity_bytes_options_and_source_digests_are_bound() {
    let data = expectations();
    let rows = data["rows"].as_array().unwrap();
    assert_eq!(rows.len(), 45);
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("tests");
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
            .find(|c| c["id"] == id)
            .unwrap();
        for key in ["input", "context"] {
            assert_eq!(case[key], row[key], "{id}: {key}");
        }
        assert_eq!(
            case.get("options").cloned().unwrap_or_else(|| json!({})),
            row["options"],
            "{id}: options"
        );
        assert_eq!(case["status"], "active", "{id}");
    }
}
#[test]
fn registry_classes_match_independent_logical_selector_contracts() {
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
