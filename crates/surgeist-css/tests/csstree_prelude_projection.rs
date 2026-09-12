#![forbid(unsafe_code)]

//! CSS-owned prelude classification follows the explicit rule-name option.

#[test]
fn prelude_expected_classes_distinguish_generic_and_named_media_contexts() {
    let document: serde_json::Value =
        serde_json::from_str(include_str!("csstree/expected-classes.json")).unwrap();
    let records = document["records"].as_array().unwrap();
    let find = |id: &str| &records.iter().find(|record| record["id"] == id).unwrap()["class"];
    assert_eq!(
        find("atrulePrelude/index.json#/base test"),
        &serde_json::json!({
            "kind":"unsupported", "reason":"generic_fragment_without_truthful_supported_property_or_descriptor",
            "policy":{"kind":"panic_freedom_only"}
        })
    );
    assert_eq!(
        find("atrulePrelude/index.json#/should use custom parser"),
        &serde_json::json!({
            "kind":"clean", "retained_syntax":{"extractor":{"kind":"media_queries"},"predicate":{"relation":"nonempty"}}
        })
    );
}
