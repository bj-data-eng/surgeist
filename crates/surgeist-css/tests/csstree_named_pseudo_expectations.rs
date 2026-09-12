//! Declared named-pseudo admission and raw-fragment rejection dispositions.
//! https://www.w3.org/TR/2026/WD-selectors-4-20260122/#pseudo-element-syntax
//! https://www.w3.org/TR/2026/WD-selectors-4-20260122/#invalid
#[test]
fn named_pseudo_expectations_use_defined_grammar_and_fragment_rejection() {
    let actual: serde_json::Value =
        serde_json::from_str(include_str!("csstree/expected-classes.json")).unwrap();
    let clean = serde_json::json!({
        "kind": "clean",
        "retained_syntax": {
            "extractor": {"kind": "selector"},
            "predicate": {"relation": "nonempty"}
        }
    });
    let rejected = serde_json::json!({
        "kind": "strict_rejected",
        "retained_syntax": {
            "extractor": {"kind": "selector"},
            "predicate": {"relation": "empty"}
        },
        "diagnostics": [{
            "code": "invalid_selector",
            "action": "reject_input",
            "payload_relation": "intersects"
        }]
    });
    let records = actual["records"].as_array().unwrap();
    for (id, expected) in [
        (
            "selector/PseudoClassSelector.json#/pseudo element :first-letter",
            &clean,
        ),
        (
            "selector/PseudoClassSelector.json#/pseudo element :first-line",
            &clean,
        ),
        ("selector/PseudoClassSelector.json#/pseudoc.0", &rejected),
        ("selector/PseudoClassSelector.json#/pseudoc.1", &rejected),
        (
            "selector/PseudoClassSelector.json#/unknown function pseudo class",
            &rejected,
        ),
        (
            "selector/PseudoClassSelector.json#/unknown function pseudo class with nested blocks",
            &rejected,
        ),
        (
            "selector/PseudoClassSelector.json#/unknown function pseudo class with nested blocks (unbalanced parenthesis)",
            &rejected,
        ),
        (
            "selector/PseudoClassSelector.json#/unknown function pseudo class with spaces and comments",
            &rejected,
        ),
        (
            "selector/PseudoElementSelector.json#/pseudo element #0",
            &rejected,
        ),
        (
            "selector/PseudoElementSelector.json#/pseudo element #1",
            &rejected,
        ),
        (
            "selector/PseudoElementSelector.json#/functional pseudo element",
            &rejected,
        ),
    ] {
        let matching: Vec<_> = records.iter().filter(|record| record["id"] == id).collect();
        assert_eq!(matching.len(), 1, "expected one record for {id}");
        assert_eq!(&matching[0]["class"], expected, "{id}");
    }
}
