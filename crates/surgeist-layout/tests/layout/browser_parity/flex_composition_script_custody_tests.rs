const SCRIPT_CUSTODY_REPORT: &str = include_str!("flex_composition_script_custody_report.json");

fn script_custody_flex_compare_variant(index: usize) {
    use super::exact_differences::flex_composition_qualification::{Inputs, qualify};

    assert_eq!(
        sha256(SCRIPT_CUSTODY_REPORT.as_bytes()),
        "cd167f9ecd34d142423885080df4f839ccf94b81c5ba80eb70ea868770547024"
    );
    let inputs = Inputs::load(std::path::Path::new(env!("CARGO_MANIFEST_DIR"))).unwrap();
    let (variant, xml) = VARIANTS[index];
    let output = format!("xml/flex/fri07_flex_composition__{variant}.xml");
    let retained = Golden::parse_with_flex_composition(xml, REPORT, SOURCE, SOURCE_BYTES, &output)
        .expect("the retained environment remains admissible");
    let current = Golden::parse_with_flex_composition(
        xml,
        SCRIPT_CUSTODY_REPORT,
        SOURCE,
        SOURCE_BYTES,
        &output,
    )
    .expect("the script-custody environment preserves the qualified Flex capture");
    let mut retained_raw = Vec::new();
    let mut current_raw = Vec::new();
    let retained_counts = qualify(&retained, variant, &inputs, &mut retained_raw).unwrap();
    let current_counts = qualify(&current, variant, &inputs, &mut current_raw).unwrap();
    assert_eq!(current_counts, (9, usize::from(variant.ends_with("_rtl"))));
    assert_eq!(current_counts, retained_counts);
    assert_eq!(current_raw, retained_raw);
}

#[test]
fn script_custody_flex_border_box_ltr_preserves_complete_qualification() {
    script_custody_flex_compare_variant(0);
}

#[test]
fn script_custody_flex_border_box_rtl_preserves_complete_qualification() {
    script_custody_flex_compare_variant(1);
}

#[test]
fn script_custody_flex_content_box_ltr_preserves_complete_qualification() {
    script_custody_flex_compare_variant(2);
}

#[test]
fn script_custody_flex_content_box_rtl_preserves_complete_qualification() {
    script_custody_flex_compare_variant(3);
}

#[test]
fn script_custody_flex_environment_refuses_changed_or_crossed_identity_rows() {
    let old: serde_json::Value = serde_json::from_str(REPORT).unwrap();
    let new: serde_json::Value = serde_json::from_str(SCRIPT_CUSTODY_REPORT).unwrap();
    let (variant, xml) = VARIANTS[0];
    let output = format!("xml/flex/fri07_flex_composition__{variant}.xml");
    let refused = |report: &serde_json::Value| {
        assert!(
            Golden::parse_with_flex_composition(
                xml,
                &report.to_string(),
                SOURCE,
                SOURCE_BYTES,
                &output,
            )
            .is_err()
        );
    };
    for pointer in [
        "/metadata/contract_sha256",
        "/metadata/inputs/corpus.toml",
        "/metadata/inputs/scripts~1gentest~1test_helper.js",
    ] {
        for (base, other) in [(&new, &old), (&old, &new)] {
            let mut crossed = base.clone();
            *crossed.pointer_mut(pointer).unwrap() = other.pointer(pointer).unwrap().clone();
            refused(&crossed);
        }
    }
    for pointer in [
        "/metadata/contract_sha256",
        "/metadata/inputs/corpus.toml",
        "/metadata/inputs/scripts~1gentest~1test_helper.js",
        "/metadata/inputs/browser-owner.toml",
        "/metadata/browser_executable",
        "/metadata/browser_executable_sha256",
        "/metadata/browser_provenance",
        "/metadata/engine_version",
        "/metadata/generator",
    ] {
        let mut changed = new.clone();
        *changed.pointer_mut(pointer).unwrap() = serde_json::json!("0".repeat(64));
        refused(&changed);
    }
    let mut imported = new;
    imported["metadata"]["source_imports"] = serde_json::json!([{"source": "another-owner"}]);
    refused(&imported);
}
