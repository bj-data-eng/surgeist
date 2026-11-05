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

// These captures are authored V4 inputs. The labels remain original DIV
// content, not empty boxes or prepared Text; font-family alone does not supply
// the selected line/font baseline consumed by parallel row baseline alignment.
fn analytic_flex_capture(
    name: &str,
    alignment: &str,
    items: &str,
    observations: &str,
    diagnostics: &str,
) -> Golden {
    Golden::parse(&format!(
        r#"<test name="{name}" measurement-version="4" observation-frame="root-border" use-rounding="false">
<viewport root-context="block-flow-auto" parent-inline-extent="800px" parent-direction="ltr" fallback-icb-width="800px" fallback-icb-height="600px" fallback-nearest-scrollport="none" fallback-containing-width-limit="unbounded" fallback-containing-height-limit="unbounded"/>
<input><div grid-alignment-profile="{{&quot;profile&quot;:1,&quot;owners&quot;:[]}}" source-tag="div" display="flex" position="relative" direction="ltr" writing-mode="horizontal-tb" font-family="ahem" font-size="10px" line-height="10px" width="180px" height="50px" {alignment}>{items}</div></input>
<expectations><node x="0" y="0" width="180" height="50">{observations}</node></expectations>
<diagnostics><world-rounded-layout node-path="0" x="0" y="0" width="180" height="50"/>{diagnostics}</diagnostics>
</test>"#
    ))
    .expect("authored V4 Flex source with actual Htb root context parses")
}

fn analytic_flex_label_items(first_style: &str, second_style: &str) -> String {
    format!(
        r#"<text source-tag="div" display="flex" direction="ltr" writing-mode="horizontal-tb" flex-shrink="0" width="30px" height="20px" {first_style}>base</text>
<text source-tag="div" display="flex" direction="ltr" writing-mode="horizontal-tb" flex-shrink="0" width="30px" height="25px" {second_style}>peer</text>"#
    )
}

#[test]
fn numeric_collection_refuses_opaque_content_inherited_row_baseline_without_line_authority() {
    let golden = analytic_flex_capture(
        "opaque-flex-content-baseline",
        r#"flex-direction="row" align-items="baseline""#,
        &analytic_flex_label_items("", ""),
        r#"<node x="0" y="0" width="30" height="20"/><node x="30" y="0" width="30" height="25"/>"#,
        r#"<world-rounded-layout node-path="0.0" x="0" y="0" width="30" height="20"/><world-rounded-layout node-path="0.1" x="30" y="0" width="30" height="25"/>"#,
    );
    // Equal source typography does not authenticate a Native content baseline.
    // In particular, the 20px/25px box edges cannot replace those line facts.
    assert_eq!(
        collect_surgeist_numeric_mismatches(&golden)
            .unwrap_err()
            .to_string(),
        "UnsupportedAnalyticBaselineTextProfile"
    );
    assert_eq!(
        assert_surgeist_matches(&golden).unwrap_err().to_string(),
        "UnsupportedAnalyticBaselineTextProfile"
    );
}

#[test]
fn numeric_collection_refuses_missing_content_baseline_before_earlier_root_mismatch() {
    let mut golden = analytic_flex_capture(
        "opaque-flex-baseline-after-root-mismatch",
        r#"flex-direction="row" align-items="baseline""#,
        &analytic_flex_label_items("", ""),
        r#"<node x="0" y="0" width="30" height="20"/><node x="30" y="0" width="30" height="25"/>"#,
        r#"<world-rounded-layout node-path="0.0" x="0" y="0" width="30" height="20"/><world-rounded-layout node-path="0.1" x="30" y="0" width="30" height="25"/>"#,
    );
    golden.expectations.x = Some(1.0);
    assert_eq!(
        collect_surgeist_numeric_mismatches(&golden)
            .unwrap_err()
            .to_string(),
        "UnsupportedAnalyticBaselineTextProfile",
        "a partial root mismatch set cannot qualify missing descendant line authority"
    );
}

#[test]
fn numeric_collection_refuses_opaque_content_with_explicit_row_baseline_alignment() {
    let golden = analytic_flex_capture(
        "opaque-flex-explicit-baseline",
        r#"flex-direction="row" align-items="flex-start""#,
        &analytic_flex_label_items(r#"align-self="baseline""#, r#"align-self="baseline""#),
        r#"<node x="0" y="0" width="30" height="20"/><node x="30" y="0" width="30" height="25"/>"#,
        r#"<world-rounded-layout node-path="0.0" x="0" y="0" width="30" height="20"/><world-rounded-layout node-path="0.1" x="30" y="0" width="30" height="25"/>"#,
    );
    assert_eq!(
        collect_surgeist_numeric_mismatches(&golden)
            .unwrap_err()
            .to_string(),
        "UnsupportedAnalyticBaselineTextProfile"
    );
}

#[test]
fn numeric_collection_refuses_opaque_row_baseline_even_with_main_axis_auto_margin() {
    let golden = analytic_flex_capture(
        "opaque-flex-baseline-main-auto-margin",
        r#"flex-direction="row" align-items="baseline""#,
        &analytic_flex_label_items(r#"margin-left="auto""#, r#"align-self="flex-start""#),
        r#"<node x="120" y="0" width="30" height="20"/><node x="150" y="0" width="30" height="25"/>"#,
        r#"<world-rounded-layout node-path="0.0" x="120" y="0" width="30" height="20"/><world-rounded-layout node-path="0.1" x="150" y="0" width="30" height="25"/>"#,
    );
    assert_eq!(
        collect_surgeist_numeric_mismatches(&golden)
            .unwrap_err()
            .to_string(),
        "UnsupportedAnalyticBaselineTextProfile"
    );
}

#[test]
fn numeric_collection_keeps_opaque_content_when_align_self_overrides_baseline() {
    let golden = analytic_flex_capture(
        "opaque-flex-start-override",
        r#"flex-direction="row" align-items="baseline""#,
        &analytic_flex_label_items(r#"align-self="flex-start""#, r#"align-self="flex-start""#),
        r#"<node x="0" y="0" width="30" height="20"/><node x="30" y="0" width="30" height="25"/>"#,
        r#"<world-rounded-layout node-path="0.0" x="0" y="0" width="30" height="20"/><world-rounded-layout node-path="0.1" x="30" y="0" width="30" height="25"/>"#,
    );
    assert_surgeist_matches(&golden).expect("explicit start alignment uses fixed box geometry");
    assert!(
        collect_surgeist_numeric_mismatches(&golden)
            .unwrap()
            .is_empty()
    );
}

#[test]
fn numeric_collection_keeps_opaque_content_when_cross_auto_margins_take_precedence() {
    let golden = analytic_flex_capture(
        "opaque-flex-cross-auto-margin",
        r#"flex-direction="row" align-items="baseline""#,
        &analytic_flex_label_items(r#"margin-top="auto""#, r#"margin-top="auto""#),
        r#"<node x="0" y="30" width="30" height="20"/><node x="30" y="25" width="30" height="25"/>"#,
        r#"<world-rounded-layout node-path="0.0" x="0" y="30" width="30" height="20"/><world-rounded-layout node-path="0.1" x="30" y="25" width="30" height="25"/>"#,
    );
    assert_surgeist_matches(&golden).expect("cross auto margins consume the 30px/25px free space");
    assert!(
        collect_surgeist_numeric_mismatches(&golden)
            .unwrap()
            .is_empty()
    );
}

#[test]
fn numeric_collection_keeps_opaque_content_when_baseline_axis_cannot_align_column_cross_axis() {
    let golden = analytic_flex_capture(
        "opaque-flex-column-axis-fallback",
        r#"flex-direction="column" align-items="baseline""#,
        &analytic_flex_label_items("", ""),
        r#"<node x="0" y="0" width="30" height="20"/><node x="0" y="20" width="30" height="25"/>"#,
        r#"<world-rounded-layout node-path="0.0" x="0" y="0" width="30" height="20"/><world-rounded-layout node-path="0.1" x="0" y="20" width="30" height="25"/>"#,
    );
    assert_surgeist_matches(&golden)
        .expect("Htb block baseline is physical y; column cross alignment is physical x");
    assert!(
        collect_surgeist_numeric_mismatches(&golden)
            .unwrap()
            .is_empty()
    );
}

#[test]
fn numeric_collection_keeps_empty_boxes_in_row_baseline_alignment() {
    let golden = analytic_flex_capture(
        "empty-flex-box-baseline-synthesis",
        r#"flex-direction="row" align-items="baseline""#,
        r#"<div source-tag="div" display="flex" direction="ltr" writing-mode="horizontal-tb" flex-shrink="0" width="30px" height="20px"/>
<div source-tag="div" display="flex" direction="ltr" writing-mode="horizontal-tb" flex-shrink="0" width="30px" height="25px"/>"#,
        r#"<node x="0" y="5" width="30" height="20"/><node x="30" y="0" width="30" height="25"/>"#,
        r#"<world-rounded-layout node-path="0.0" x="0" y="5" width="30" height="20"/><world-rounded-layout node-path="0.1" x="30" y="0" width="30" height="25"/>"#,
    );
    // This independent source is actually empty: aligning its bottom edges is
    // legitimate synthesis, with 25 - 20 = 5px translation for the first box.
    assert_surgeist_matches(&golden).expect("empty boxes retain baseline synthesis");
    assert!(
        collect_surgeist_numeric_mismatches(&golden)
            .unwrap()
            .is_empty()
    );
}

#[test]
fn numeric_collection_keeps_opaque_absolute_boxes_outside_flex_baseline_participation() {
    let golden = analytic_flex_capture(
        "opaque-absolute-flex-content",
        r#"flex-direction="row" align-items="baseline""#,
        &analytic_flex_label_items(
            r#"position="absolute" left="0px" top="0px""#,
            r#"position="absolute" left="40px" top="0px""#,
        ),
        r#"<node x="0" y="0" width="30" height="20"/><node x="40" y="0" width="30" height="25"/>"#,
        r#"<world-rounded-layout node-path="0.0" x="0" y="0" width="30" height="20"/><world-rounded-layout node-path="0.1" x="40" y="0" width="30" height="25"/>"#,
    );
    assert_surgeist_matches(&golden).unwrap();
    assert!(
        collect_surgeist_numeric_mismatches(&golden)
            .unwrap()
            .is_empty()
    );
}

#[test]
fn numeric_collection_does_not_require_baselines_from_a_nonparticipating_hidden_subtree() {
    let golden = analytic_flex_capture(
        "hidden-opaque-flex-content",
        r#"flex-direction="row" align-items="baseline""#,
        r#"<div source-tag="div" display="none"><div source-tag="div" display="flex" align-items="baseline"><text source-tag="div" display="flex" width="30px" height="20px">base</text></div></div>"#,
        r#"<node x="0" y="0" width="0" height="0"><node x="0" y="0" width="0" height="0"><node x="0" y="0" width="0" height="0"/></node></node>"#,
        r#"<world-rounded-layout node-path="0.0" x="0" y="0" width="0" height="0"/><world-rounded-layout node-path="0.0.0" x="0" y="0" width="0" height="0"/><world-rounded-layout node-path="0.0.0.0" x="0" y="0" width="0" height="0"/>"#,
    );
    assert_surgeist_matches(&golden).unwrap();
    assert!(
        collect_surgeist_numeric_mismatches(&golden)
            .unwrap()
            .is_empty()
    );
}
