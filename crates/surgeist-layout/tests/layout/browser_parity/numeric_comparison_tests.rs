//! Strict reader behavior over a real fixed-size Native tree.
use super::*;

fn fixture_xml() -> String {
    format!(r#"<test name="numeric-comparison" measurement-version="4" observation-frame="root-border" use-rounding="false">
<viewport root-context="block-flow-auto" parent-inline-extent="800px" parent-direction="ltr" fallback-icb-width="800px" fallback-icb-height="600px" fallback-nearest-scrollport="none" fallback-containing-width-limit="unbounded" fallback-containing-height-limit="unbounded"/>
<input><div display="block" position="relative" direction="ltr" width="100px" height="20px"><div display="block" width="20px" height="10px"/></div></input>
<expectations><node x="0" y="0" width="100" height="20"><node x="0" y="0" width="20" height="10"/></node></expectations>
{DIAGNOSTICS}</test>"#)
}

const DIAGNOSTICS: &str = r#"<diagnostics><world-rounded-layout node-path="0" x="0" y="0" width="100" height="20"/><world-rounded-layout node-path="0.0" x="0" y="0" width="20" height="10"/></diagnostics>"#;

fn fixture() -> Golden {
    Golden::parse(&fixture_xml()).expect("explicit source-bound fixed Native boxes parse")
}

fn legacy_fixture() -> Golden {
    let raw = fixture_xml()
        .replace(" measurement-version=\"4\"", "")
        .replace(" observation-frame=\"root-border\"", "")
        .replace(DIAGNOSTICS, "");
    Golden::parse(&raw).expect("historical source/expectations parse without issuing root authority")
}

#[test]
fn strict_numeric_comparison_keeps_first_error_and_exact_diagnostic() {
    let mut golden = fixture();
    assert_surgeist_matches(&golden).expect("real fixed-size Native control matches");
    golden.expectations.x = Some(1.0);
    let first = assert_surgeist_matches(&golden).unwrap_err();
    assert_eq!(first.to_string(), "numeric-comparison: x mismatch, expected 1, got 0");
    golden.expectations.width = Some(101.0);
    golden.expectations.children[0].height = Some(11.0);
    assert_eq!(assert_surgeist_matches(&golden).unwrap_err(), first,
        "later root and descendant mismatches do not change strict first-error behavior");
}

#[test]
fn strict_numeric_comparison_keeps_source_topology_fatal() {
    let mut golden = fixture();
    golden.expectations.children.push(golden.expectations.children[0].clone());
    assert_eq!(assert_surgeist_matches(&golden).unwrap_err().to_string(),
        "numeric-comparison: expected 2 children, got 1");
}

#[test]
fn strict_numeric_comparison_requires_actual_root_authority() {
    let golden = legacy_fixture();
    assert_eq!(assert_surgeist_matches(&golden).unwrap_err().to_string(),
        "MissingRootContextAuthority: historical viewport metadata cannot issue a Native sizing operation");
}
