//! Contract for the corpus's persisted raw-fragment extractor identities.
//! Admission and diagnostics are independent of this adapter projection.

#[test]
fn raw_fragment_expectations_name_their_direct_extractors() {
    let document: serde_json::Value =
        serde_json::from_str(include_str!("csstree/expected-classes.json"))
            .expect("expected-class artifact must be JSON");
    let mut counts = [0; 3];
    let mut mismatches = Vec::new();
    for record in document["records"].as_array().expect("records array") {
        let id = record["id"].as_str().expect("fixture identity");
        let (index, expected) =
            if id.starts_with("selector/") && !id.starts_with("selector/Combinator.json#") {
                (0, "selector")
            } else if id.starts_with("selectorList/") {
                (1, "selector_list")
            } else if id.starts_with("mediaQuery/") {
                (2, "media_query")
            } else {
                continue;
            };
        counts[index] += 1;
        let class = &record["class"];
        let retained = if class["kind"] == "unsupported" {
            &class["policy"]["retained_syntax"]
        } else {
            &class["retained_syntax"]
        };
        if retained["extractor"]["kind"] != expected {
            mismatches.push(format!("{id}: expected {expected}"));
        }
    }
    assert_eq!(counts, [300, 10, 49], "pinned raw-fragment inventory");
    assert!(mismatches.is_empty(), "{}", mismatches.join("\n"));
}
