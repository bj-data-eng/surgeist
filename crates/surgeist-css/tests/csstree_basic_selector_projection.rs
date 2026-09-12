//! Persisted CSS-owned basic-selector admission and raw rejection contracts.
//! The CSS-owned expected-class registry is a persisted corpus contract.
//! Authority: Selectors 4 sections 6.4 and 16, and the raw singleton RejectInput contract.
//! https://www.w3.org/TR/2026/WD-selectors-4-20260122/#attrnmsp
//! https://www.w3.org/TR/2026/WD-selectors-4-20260122/#grammar
use serde_json::Value;

const ADMISSION_CHANGES: [&str; 5] = [
    "selector/AttributeSelector.json#/attrib with namespace and dashmatch",
    "selector/AttributeSelector.json#/attrib with namespace, dashmatch and spaces",
    "selector/AttributeSelector.json#/attrib.2 with flags",
    "selector/AttributeSelector.json#/namespace",
    "selector/AttributeSelector.json#/namespace w~1o attrselector",
];
const EXISTING_REJECTIONS: [&str; 17] = [
    "selector/AttributeSelector.json#/error/0",
    "selector/AttributeSelector.json#/error/1",
    "selector/AttributeSelector.json#/error/10",
    "selector/AttributeSelector.json#/error/11",
    "selector/AttributeSelector.json#/error/12",
    "selector/AttributeSelector.json#/error/13",
    "selector/AttributeSelector.json#/error/14",
    "selector/AttributeSelector.json#/error/2",
    "selector/AttributeSelector.json#/error/3",
    "selector/AttributeSelector.json#/error/4",
    "selector/AttributeSelector.json#/error/5",
    "selector/AttributeSelector.json#/error/6",
    "selector/AttributeSelector.json#/error/7",
    "selector/AttributeSelector.json#/error/8",
    "selector/AttributeSelector.json#/error/9",
    "selector/TypeSelector.json#/error/0",
    "selector/TypeSelector.json#/error/1",
];

fn classes() -> Value {
    serde_json::from_str(include_str!("csstree/expected-classes.json")).unwrap()
}
fn class<'a>(document: &'a Value, id: &str) -> &'a Value {
    &document["records"]
        .as_array()
        .unwrap()
        .iter()
        .find(|record| record["id"] == id)
        .expect("pinned case exists")["class"]
}

#[test]
fn basic_selector_classes_reject_presence_flags_and_undeclared_prefixes() {
    let document = classes();
    for id in ADMISSION_CHANGES {
        let value = class(&document, id);
        assert_eq!(value["kind"], "strict_rejected", "{id}");
        assert_eq!(
            value["retained_syntax"],
            serde_json::json!({"extractor":{"kind":"selector"}, "predicate":{"relation":"empty"}}),
            "{id}"
        );
        let diagnostics = value["diagnostics"]
            .as_array()
            .expect("rejection requires diagnostics");
        assert!(!diagnostics.is_empty(), "{id}");
        assert!(
            diagnostics
                .iter()
                .any(|diagnostic| diagnostic["action"] == "reject_input"),
            "{id}"
        );
    }
}

#[test]
fn basic_selector_existing_rejections_use_raw_input_actions() {
    let document = classes();
    for id in EXISTING_REJECTIONS {
        let value = class(&document, id);
        assert_eq!(value["kind"], "strict_rejected", "{id}");
        assert_eq!(
            value["retained_syntax"],
            serde_json::json!({"extractor":{"kind":"selector"}, "predicate":{"relation":"empty"}}),
            "{id}"
        );
        let diagnostics = value["diagnostics"]
            .as_array()
            .expect("rejection requires diagnostics");
        assert!(!diagnostics.is_empty(), "{id}");
        assert!(
            diagnostics
                .iter()
                .any(|diagnostic| diagnostic["action"] == "reject_input"),
            "{id}"
        );
        assert!(
            diagnostics
                .iter()
                .all(|diagnostic| diagnostic["action"] != "drop_qualified_rule"),
            "{id}: raw input contains no accepted style rule to drop"
        );
    }
}
