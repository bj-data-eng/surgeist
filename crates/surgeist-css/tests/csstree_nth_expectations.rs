#![forbid(unsafe_code)]
//! Declared Nth corpus rejection follows token-level An+B grammar and raw input.
//! https://www.w3.org/TR/2021/CRD-css-syntax-3-20211224/#anb-type
#[test]
fn nth_invalid_expectations_use_raw_fragment_rejection() {
    let expected = serde_json::json!({
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
    let actual: serde_json::Value =
        serde_json::from_str(include_str!("csstree/expected-classes.json")).unwrap();
    let records = actual["records"].as_array().unwrap();
    for id in [
        "selector/Nth.json#/error/0",
        "selector/Nth.json#/error/1",
        "selector/Nth.json#/error/10",
        "selector/Nth.json#/error/11",
        "selector/Nth.json#/error/12",
        "selector/Nth.json#/error/13",
        "selector/Nth.json#/error/14",
        "selector/Nth.json#/error/15",
        "selector/Nth.json#/error/16",
        "selector/Nth.json#/error/17",
        "selector/Nth.json#/error/18",
        "selector/Nth.json#/error/19",
        "selector/Nth.json#/error/2",
        "selector/Nth.json#/error/20",
        "selector/Nth.json#/error/21",
        "selector/Nth.json#/error/22",
        "selector/Nth.json#/error/23",
        "selector/Nth.json#/error/24",
        "selector/Nth.json#/error/25",
        "selector/Nth.json#/error/26",
        "selector/Nth.json#/error/27",
        "selector/Nth.json#/error/28",
        "selector/Nth.json#/error/29",
        "selector/Nth.json#/error/3",
        "selector/Nth.json#/error/30",
        "selector/Nth.json#/error/31",
        "selector/Nth.json#/error/32",
        "selector/Nth.json#/error/33",
        "selector/Nth.json#/error/34",
        "selector/Nth.json#/error/35",
        "selector/Nth.json#/error/36",
        "selector/Nth.json#/error/37",
        "selector/Nth.json#/error/38",
        "selector/Nth.json#/error/4",
        "selector/Nth.json#/error/5",
        "selector/Nth.json#/error/6",
        "selector/Nth.json#/error/7",
        "selector/Nth.json#/error/8",
        "selector/Nth.json#/error/9",
    ] {
        let matching: Vec<_> = records.iter().filter(|record| record["id"] == id).collect();
        assert_eq!(matching.len(), 1, "one class record for {id}");
        assert_eq!(matching[0]["class"], expected, "{id}");
    }
}
