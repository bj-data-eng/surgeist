//! Strict reader behavior over a real fixed-size Native tree.
use super::*;

fn fixture_xml() -> String {
    format!(
        r#"<test name="numeric-comparison" measurement-version="4" observation-frame="root-border" use-rounding="false">
<viewport root-context="block-flow-auto" parent-inline-extent="800px" parent-direction="ltr" fallback-icb-width="800px" fallback-icb-height="600px" fallback-nearest-scrollport="none" fallback-containing-width-limit="unbounded" fallback-containing-height-limit="unbounded"/>
<input><div grid-alignment-profile="{{&quot;profile&quot;:1,&quot;owners&quot;:[]}}" display="block" position="relative" direction="ltr" width="100px" height="20px"><div display="block" width="20px" height="10px"/></div></input>
<expectations><node x="0" y="0" width="100" height="20"><node x="0" y="0" width="20" height="10"/></node></expectations>
{DIAGNOSTICS}</test>"#
    )
}

const DIAGNOSTICS: &str = r#"<diagnostics><world-rounded-layout node-path="0" x="0" y="0" width="100" height="20"/><world-rounded-layout node-path="0.0" x="0" y="0" width="20" height="10"/></diagnostics>"#;

fn fixture() -> Golden {
    Golden::parse(&fixture_xml()).expect("explicit source-bound fixed Native boxes parse")
}

fn legacy_fixture() -> Golden {
    let raw = fixture_xml()
        .replace(" measurement-version=\"4\"", " measurement-version=\"2\"")
        .replace(
            r#" grid-alignment-profile="{&quot;profile&quot;:1,&quot;owners&quot;:[]}""#,
            "",
        )
        .replace(" observation-frame=\"root-border\"", "")
        .replace(DIAGNOSTICS, "");
    let document = roxmltree::Document::parse(&raw).unwrap();
    assert_eq!(
        document.root_element().attribute("measurement-version"),
        Some("2")
    );
    assert!(
        !document
            .descendants()
            .any(|node| node.attribute("grid-alignment-profile").is_some())
    );
    Golden::parse(&raw)
        .expect("historical V2 source/expectations parse without issuing root authority")
}

#[test]
fn strict_numeric_comparison_keeps_first_error_and_exact_diagnostic() {
    let mut golden = fixture();
    assert_surgeist_matches(&golden).expect("real fixed-size Native control matches");
    golden.expectations.x = Some(1.0);
    let first = assert_surgeist_matches(&golden).unwrap_err();
    assert_eq!(
        first.to_string(),
        "numeric-comparison: x mismatch, expected 1, got 0"
    );
    golden.expectations.width = Some(101.0);
    golden.expectations.children[0].height = Some(11.0);
    assert_eq!(
        assert_surgeist_matches(&golden).unwrap_err(),
        first,
        "later root and descendant mismatches do not change strict first-error behavior"
    );
}

#[test]
fn strict_numeric_comparison_keeps_source_topology_fatal() {
    let mut golden = fixture();
    golden
        .expectations
        .children
        .push(golden.expectations.children[0].clone());
    assert_eq!(
        assert_surgeist_matches(&golden).unwrap_err().to_string(),
        "numeric-comparison: expected 2 children, got 1"
    );
}

#[test]
fn strict_numeric_comparison_requires_actual_root_authority() {
    let golden = legacy_fixture();
    assert_eq!(
        assert_surgeist_matches(&golden).unwrap_err().to_string(),
        "MissingRootContextAuthority: historical viewport metadata cannot issue a Native sizing operation"
    );
}

#[test]
fn numeric_collection_retains_later_fields_and_descendants_after_same_first_error() {
    let mut golden = fixture();
    golden.expectations.x = Some(1.0);
    let strict_first = assert_surgeist_matches(&golden).unwrap_err();
    let first = collect_surgeist_numeric_mismatches(&golden).unwrap();
    assert_eq!(
        first
            .iter()
            .map(|m| {
                let NumericMismatch::Layout(m) = m else {
                    panic!("box-only fixture must retain a layout-number mismatch");
                };
                (m.path(), m.field(), m.browser_value(), m.native_value())
            })
            .collect::<Vec<_>>(),
        [("numeric-comparison", NumericComparisonField::X, 1.0, 0.0)]
    );
    golden.expectations.width = Some(101.0);
    golden.expectations.scroll_size = Some(Size::new(5.0, 6.0));
    golden.expectations.children[0].height = Some(11.0);
    assert_eq!(assert_surgeist_matches(&golden).unwrap_err(), strict_first);
    let complete = collect_surgeist_numeric_mismatches(&golden).unwrap();
    assert_eq!(
        complete
            .iter()
            .map(|m| {
                let NumericMismatch::Layout(m) = m else {
                    panic!("box-only fixture must retain a layout-number mismatch");
                };
                (m.path(), m.field(), m.browser_value(), m.native_value())
            })
            .collect::<Vec<_>>(),
        [
            ("numeric-comparison", NumericComparisonField::X, 1.0, 0.0),
            (
                "numeric-comparison",
                NumericComparisonField::Width,
                101.0,
                100.0
            ),
            (
                "numeric-comparison",
                NumericComparisonField::ScrollWidth,
                5.0,
                0.0
            ),
            (
                "numeric-comparison",
                NumericComparisonField::ScrollHeight,
                6.0,
                0.0
            ),
            (
                "numeric-comparison/0",
                NumericComparisonField::Height,
                11.0,
                10.0
            ),
        ]
    );
    assert_ne!(
        complete, first,
        "the same first diagnostic cannot hide later changed observations"
    );
}

#[test]
fn numeric_collection_keeps_topology_and_admission_fatal_after_numeric_mismatch() {
    let mut golden = fixture();
    golden.expectations.x = Some(1.0);
    golden
        .expectations
        .children
        .push(golden.expectations.children[0].clone());
    assert_eq!(
        collect_surgeist_numeric_mismatches(&golden)
            .unwrap_err()
            .to_string(),
        "numeric-comparison: expected 2 children, got 1"
    );
    let mut legacy = legacy_fixture();
    legacy.expectations.x = Some(1.0);
    assert_eq!(
        collect_surgeist_numeric_mismatches(&legacy).unwrap_err(),
        assert_surgeist_matches(&legacy).unwrap_err()
    );
}

#[test]
fn numeric_collection_keeps_other_observation_categories_fatal() {
    let mut fragments = fixture();
    fragments.expectations.x = Some(1.0);
    fragments.expectations.children[0].fragments = Some(Vec::new());
    assert_eq!(
        collect_surgeist_numeric_mismatches(&fragments)
            .unwrap_err()
            .to_string(),
        "numeric-comparison/0: numeric collection supports only box geometry and scroll observations"
    );
    let mut mixed_category = fixture();
    mixed_category.expectations.x = Some(1.0);
    mixed_category.expectations.children[0].range_rectangles = Some(Vec::new());
    assert_eq!(
        collect_surgeist_numeric_mismatches(&mixed_category)
            .unwrap_err()
            .to_string(),
        "numeric-comparison/0: Range rectangle must not include node geometry"
    );
}

#[test]
fn numeric_collection_uses_existing_tolerance_and_refuses_nonfinite_values() {
    let mut matching = fixture();
    matching.expectations.x = Some(0.05);
    matching.expectations.width = Some(100.05);
    matching.expectations.children[0].height = Some(10.05);
    assert_surgeist_matches(&matching).unwrap();
    assert!(
        collect_surgeist_numeric_mismatches(&matching)
            .unwrap()
            .is_empty()
    );
    matching.expectations.x = Some(1.0);
    matching.expectations.children[0].width = Some(Scalar::NAN);
    let error = collect_surgeist_numeric_mismatches(&matching).unwrap_err();
    assert!(
        error.to_string().contains(
            "numeric-comparison/0: width numeric collection requires finite browser/native values"
        ),
        "non-finite observations cannot become finite mismatch records: {error}"
    );
}

#[test]
fn numeric_collection_shares_strict_root_border_frame_under_rtl_parent() {
    let raw = fixture_xml()
        .replace(r#"parent-direction="ltr""#, r#"parent-direction="rtl""#)
        .replace(r#"node-path="0" x="0""#, r#"node-path="0" x="700""#);
    let mut golden = Golden::parse(&raw).unwrap();
    assert_surgeist_matches(&golden).unwrap();
    golden.expectations.x = Some(1.0);
    assert_eq!(
        assert_surgeist_matches(&golden).unwrap_err().to_string(),
        "numeric-comparison: x mismatch, expected 1, got 0"
    );
    let mismatches = collect_surgeist_numeric_mismatches(&golden).unwrap();
    assert_eq!(
        mismatches
            .iter()
            .map(|m| {
                let NumericMismatch::Layout(m) = m else {
                    panic!("box-only fixture must retain a layout-number mismatch");
                };
                (m.path(), m.field(), m.browser_value(), m.native_value())
            })
            .collect::<Vec<_>>(),
        [("numeric-comparison", NumericComparisonField::X, 1.0, 0.0)]
    );
}
