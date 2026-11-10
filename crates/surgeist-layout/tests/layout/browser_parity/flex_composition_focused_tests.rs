const FOCUSED_ENVIRONMENT_REPORT: &str = include_str!("flex_composition_focused_report.json");

#[test]
fn focused_flex_environment_preserves_all_four_complete_policy_comparisons() {
    use super::exact_differences::flex_composition_qualification::{Inputs, qualify};
    assert_eq!(
        sha256(FOCUSED_ENVIRONMENT_REPORT.as_bytes()),
        "60437234e3d3d380bda1cb208409d23ebfa690a3add9d1db9b21c8f94ebc5b82"
    );
    let inputs = Inputs::load(std::path::Path::new(env!("CARGO_MANIFEST_DIR"))).unwrap();
    for (variant, xml) in VARIANTS {
        let output = format!("xml/flex/fri07_flex_composition__{variant}.xml");
        let old = Golden::parse_with_flex_composition(xml, REPORT, SOURCE, SOURCE_BYTES, &output)
            .unwrap();
        let new = Golden::parse_with_flex_composition(
            xml,
            FOCUSED_ENVIRONMENT_REPORT,
            SOURCE,
            SOURCE_BYTES,
            &output,
        )
        .unwrap();
        let (mut a, mut b) = (Vec::new(), Vec::new());
        let old_counts = qualify(&old, variant, &inputs, &mut a).unwrap();
        let new_counts = qualify(&new, variant, &inputs, &mut b).unwrap();
        assert_eq!(new_counts, (9, usize::from(variant.ends_with("_rtl"))));
        assert_eq!(old_counts, new_counts);
        assert_eq!(a, b);
    }
}

#[test]
fn focused_flex_environment_refuses_missing_wrong_or_crossed_declaration_keys() {
    let original: serde_json::Value = serde_json::from_str(FOCUSED_ENVIRONMENT_REPORT).unwrap();
    let old: serde_json::Value = serde_json::from_str(REPORT).unwrap();
    let (variant, xml) = VARIANTS[0];
    let output = format!("xml/flex/fri07_flex_composition__{variant}.xml");
    let refused = |value: &serde_json::Value| {
        assert!(
            Golden::parse_with_flex_composition(
                xml,
                &value.to_string(),
                SOURCE,
                SOURCE_BYTES,
                &output
            )
            .is_err()
        )
    };
    for (path, _) in super::exact_differences::flex_composition_qualification::input_pins() {
        let mut changed = original.clone();
        assert!(changed["metadata"]["inputs"].get(path).is_some(), "{path}");
        changed["metadata"]["inputs"][path] = serde_json::json!("0".repeat(64));
        refused(&changed);
        changed["metadata"]["inputs"].as_object_mut().unwrap().remove(path);
        refused(&changed);
    }
    for replacement in [
        serde_json::Value::Null,
        serde_json::json!("bad"),
        serde_json::json!("0".repeat(64)),
    ] {
        let mut changed = original.clone();
        changed["metadata"]["inputs"]["focused-declaration.json"] = replacement;
        refused(&changed);
    }
    let mut missing = original.clone();
    missing["metadata"]["inputs"]
        .as_object_mut()
        .unwrap()
        .remove("focused-declaration.json");
    refused(&missing);
    let mut crossed = missing;
    crossed["metadata"]["inputs"]["corpus.toml"] = old["metadata"]["inputs"]["corpus.toml"].clone();
    refused(&crossed);
    let mut mixed = original.clone();
    mixed["metadata"]["inputs"]["corpus.toml"] = old["metadata"]["inputs"]["corpus.toml"].clone();
    refused(&mixed);
    for key in [
        "generator",
        "contract_sha256",
        "browser_executable_sha256",
        "browser_provenance",
    ] {
        let mut changed = original.clone();
        changed["metadata"][key] = serde_json::json!("0".repeat(64));
        refused(&changed);
    }
    let mut crossed_contract = original;
    crossed_contract["metadata"]["contract_sha256"] = old["metadata"]["contract_sha256"].clone();
    refused(&crossed_contract);
}
