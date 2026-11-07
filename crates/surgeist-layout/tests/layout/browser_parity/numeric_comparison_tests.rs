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

#[test]
fn captured_flex_baselines_use_original_prepared_text() {
    const ORIGINAL: &str = "html/flex/fri07_collapsed_strut_single_line.html";
    const STYLESHEET: &str = "scripts/gentest/test_base_style.css";
    let source = include_bytes!("html/flex/fri07_collapsed_strut_single_line.html");
    let stylesheet = include_bytes!("scripts/gentest/test_base_style.css");
    let encoded_font =
        include_bytes!("../../../../surgeist-font/tests/fixtures/ahem-original/Ahem.woff2");
    let raw_report = include_str!("captured/flex-text-differences/xml/generation-reports/all.json");
    let report = capture_authority::unique_json(raw_report).unwrap();
    let inputs = &report["metadata"]["inputs"];
    assert_eq!(inputs[ORIGINAL], capture_authority::sha256(source));
    assert_eq!(inputs[STYLESHEET], capture_authority::sha256(stylesheet));
    assert_eq!(
        capture_authority::sha256(stylesheet),
        pinned_font::ROOT_CONTEXT_STYLESHEET_SHA256
    );
    assert_eq!(
        capture_authority::sha256(encoded_font),
        pinned_font::AHEM_STYLESHEET_WOFF2_SHA256
    );

    macro_rules! capture {
        ($variant:literal) => {
            (
                $variant,
                include_str!(concat!(
                    "captured/flex-text-differences/xml/flex/fri07_collapsed_strut_single_line__",
                    $variant,
                    ".xml"
                )),
            )
        };
    }
    let captures = [
        capture!("border_box_ltr"),
        capture!("border_box_rtl"),
        capture!("content_box_ltr"),
        capture!("content_box_rtl"),
    ];
    let artifacts = report["generated"].as_array().unwrap();
    assert_eq!(artifacts.len(), captures.len());

    let mut failures = Vec::new();
    for (variant, xml) in captures {
        let artifact = artifacts
            .iter()
            .find(|artifact| artifact["variant"] == variant)
            .expect("each box and direction variant is in the capture report");
        assert_eq!(artifact["source"], ORIGINAL);
        assert_eq!(artifact["source_sha256"], inputs[ORIGINAL]);
        assert_eq!(
            artifact["output_sha256"],
            capture_authority::sha256(xml.as_bytes())
        );

        let unbound = Golden::parse(xml).expect("original V4 capture parses");
        assert_eq!(
            collect_surgeist_numeric_mismatches(&unbound)
                .unwrap_err()
                .to_string(),
            "UnsupportedAnalyticBaselineTextProfile"
        );
        let golden = Golden::parse_with_flex_content_text(
            xml,
            raw_report,
            ORIGINAL,
            source,
            artifact["output"].as_str().unwrap(),
        )
        .expect("original V4 capture and font proof bind");
        assert_eq!(artifact["name"], golden.name);
        assert!(golden.has_element_scroll_sizes());
        let mut changed = golden.clone();
        changed.root.children[0].text = Some("different label".into());
        assert!(
            collect_surgeist_numeric_mismatches(&changed)
                .unwrap_err()
                .to_string()
                .contains("source inputs changed")
        );
        assert!(
            Golden::parse_with_flex_content_text(
                xml,
                raw_report,
                ORIGINAL,
                b"changed source",
                artifact["output"].as_str().unwrap(),
            )
            .is_err()
        );
        let labels = golden
            .root
            .children
            .iter()
            .map(|child| child.text.as_deref().unwrap())
            .collect::<Vec<_>>();
        assert_eq!(labels, ["base", "small strut", "peer", "large strut"]);

        let mut numeric = NumericComparisonSink::CollectAll {
            root: golden.name.clone(),
            fields: Vec::new(),
        };
        let rtl = variant.ends_with("_rtl");
        let result = compare_captured_with_batch(&golden, &mut numeric, |tree, batch| {
            assert_original_flex_packets(tree, batch, rtl);
            Ok(())
        });
        if let Err(error) = result {
            failures.push(format!("{variant}: {error}"));
            continue;
        }
        let NumericComparisonSink::CollectAll { fields, .. } = numeric else {
            unreachable!("comparison owns the exact collection");
        };
        assert_original_flex_observations(&fields, rtl);
    }
    assert!(failures.is_empty(), "{}", failures.join("\n"));
}

fn assert_original_flex_packets(
    tree: &TestTree,
    batch: &layout::CompletedLayoutBatch<usize>,
    rtl: bool,
) {
    use layout::{Direction, Point, Size, SourceIndex};
    let flow = layout::FlowAxes::new(
        layout::WritingMode::HorizontalTb,
        if rtl { Direction::Rtl } else { Direction::Ltr },
    );
    let children = &tree.nodes[0].children;
    assert_eq!(children.len(), 4);
    assert_eq!(tree.nodes.len(), 13);
    for entries in [
        super::observation::phase(batch, false),
        super::observation::phase(batch, true),
    ] {
        assert_eq!(entries.0.node_layouts().len(), 13);
        let packet =
            |node| super::observation::observe(entries, node, tree.expected_publication_kind(node));
        assert_original_flex_box(packet(0), 0, 0.0, Size::new(180.0, 60.0), 0.0, 180.0, flow);
        for (index, &source) in children.iter().enumerate() {
            let [wrapper] = tree.nodes[source].children[..] else {
                panic!("each label has one anonymous item");
            };
            let [text] = tree.nodes[wrapper].children[..] else {
                panic!("each anonymous item has one text leaf");
            };
            let (x, height) = match (index, rtl) {
                (0, false) => (0.0, 20.0),
                (0, true) => (150.0, 20.0),
                (2, false) => (30.0, 25.0),
                (2, true) => (120.0, 25.0),
                _ => {
                    assert!(packet(source).matches_numeric_fields(
                        &layout::NodeOutput::with_source_index(SourceIndex::new(index))
                    ));
                    for node in [wrapper, text] {
                        assert!(packet(node).matches_numeric_fields(&layout::NodeOutput::new()));
                    }
                    continue;
                }
            };
            // The 40px min-content word overflows the 30px inner Flex
            // container from inline-start, which is the right edge in RTL.
            let anonymous_x = if rtl { -10.0 } else { 0.0 };
            assert_original_flex_box(
                packet(source),
                index,
                x,
                Size::new(30.0, height),
                anonymous_x,
                40.0,
                flow,
            );
            assert_original_flex_box(
                packet(wrapper),
                0,
                anonymous_x,
                Size::new(40.0, height),
                0.0,
                40.0,
                flow,
            );
            assert!(packet(text).matches_numeric_fields(&layout::NodeOutput {
                size: Size::new(40.0, 10.0),
                content_size: Size::new(40.0, 10.0),
                ..layout::NodeOutput::new()
            }));
        }
    }

    let [first, second] = batch.placed_text() else {
        panic!("only the two visible labels have placed text");
    };
    let label_range = surgeist_text::Range::new(0, 4);
    for (placed, source) in [(first, children[0]), (second, children[2])] {
        let wrapper = tree.nodes[source].children[0];
        let text = tree.nodes[wrapper].children[0];
        assert_eq!(placed.container(), wrapper);
        assert_eq!(placed.nodes(), &[text]);
        let lines = placed.layout().lines();
        assert_eq!(lines.len(), 1);
        assert_eq!(lines[0].range(), label_range);
        // The face's ascent is 800 of 1000 units at 10px with zero leading.
        assert_eq!(lines[0].baseline(), 8.0);
        for fragments in [
            batch.unrounded_inline_fragments(),
            batch.final_inline_fragments(),
        ] {
            assert_eq!(fragments.len(), 8);
            let mut fragments = fragments
                .iter()
                .filter(|entry| entry.owner() == &layout::InlineFragmentOwner::Node(text))
                .map(|entry| {
                    let fragment = entry.fragment();
                    let range = fragment.finalized_fragment().view().local_range().unwrap();
                    (range, fragment)
                })
                .collect::<Vec<_>>();
            // Both visible labels are one LTR word, including in the RTL
            // paragraph. Each character covers one unit and advances 10px.
            fragments.sort_by_key(|(range, _)| range.start);
            assert_eq!(fragments.len(), label_range.len());
            for (index, (range, fragment)) in fragments.into_iter().enumerate() {
                assert_eq!(range, surgeist_text::Range::new(index, index + 1));
                assert_eq!(fragment.line_index(), 0);
                assert_eq!(fragment.baseline().y, 8.0);
                assert_eq!(
                    fragment.rect(),
                    Some(
                        layout::ScrollRect::try_new(
                            Point::new(index as Scalar * 10.0, 0.0),
                            Size::new(10.0, 10.0),
                        )
                        .unwrap()
                    )
                );
            }
        }
    }
}

fn assert_original_flex_box(
    output: super::observation::Observation<Scalar>,
    source_index: usize,
    x: Scalar,
    size: layout::Size<Scalar>,
    overflow_x: Scalar,
    overflow_width: Scalar,
    flow: layout::FlowAxes,
) {
    use layout::{Edges, Point, Size};
    let layout::NodeOutput {
        named_baselines,
        source_index: actual_index,
        location,
        size: actual_size,
        content_size,
        scroll_geometry,
        border,
        padding,
        margin,
    } = output
        .box_output
        .expect("original flex item produces one box");
    assert_eq!(named_baselines, layout::NamedBaselinesOf::unknown());
    assert_eq!(actual_index, layout::SourceIndex::new(source_index));
    assert_eq!(location, Point::new(x, 0.0));
    assert_eq!(actual_size, size);
    let overflow_size = Size::new(overflow_width, size.height);
    assert_eq!(content_size, overflow_size);
    assert_eq!([border, padding, margin], [Edges::ZERO; 3]);

    let scroll = scroll_geometry.expect("visible boxes publish performed scroll geometry");
    let box_rect = layout::ScrollRect::try_new(Point::ZERO, size).unwrap();
    for rect in [
        scroll.border_box(),
        scroll.padding_box(),
        scroll.content_box(),
        scroll.scrollport(),
        scroll.optimal_viewing_region(),
        scroll.target().border_box(),
    ] {
        assert_eq!(rect, box_rect);
    }
    assert_eq!(
        scroll.scrollable_overflow(),
        layout::ScrollRect::try_new(Point::new(overflow_x, 0.0), overflow_size).unwrap()
    );
    assert_eq!(scroll.flow_axes(), flow);
    assert_eq!(scroll.target().flow_axes(), flow);
    assert_eq!(scroll.used_overflow_x(), layout::Overflow::Visible);
    assert_eq!(scroll.used_overflow_y(), layout::Overflow::Visible);
    assert_eq!(scroll.overflow_clip().x(), None);
    assert_eq!(scroll.overflow_clip().y(), None);
    for interval in [scroll.physical_range().x(), scroll.physical_range().y()] {
        assert_eq!((interval.minimum(), interval.maximum()), (0.0, 0.0));
    }
    let gutters = scroll.gutters();
    assert_eq!(
        [
            gutters.top(),
            gutters.right(),
            gutters.bottom(),
            gutters.left()
        ],
        [None; 4]
    );
    assert_eq!(scroll.scrollbar_size(), Size::ZERO);
    assert_eq!(scroll.resolved_scroll_padding(), Edges::ZERO);
    assert_eq!(scroll.scroll_snap_type(), layout::ScrollSnapType::None);
    let target = scroll.target();
    let margin = target.scroll_margin();
    assert_eq!(
        [margin.top(), margin.right(), margin.bottom(), margin.left()],
        [0.0; 4]
    );
    assert_eq!(target.snap_align(), layout::ScrollSnapAlign::default());
    assert_eq!(target.snap_stop(), layout::ScrollSnapStop::Normal);
}

fn assert_original_flex_observations(observed: &[exact_differences::ComparedField], rtl: bool) {
    use ElementScrollSizeField as Element;
    use NumericComparisonField as Geometry;
    use exact_differences::ComparedField;

    // The root is 180px wide and its first line supplies the 60px collapse
    // strut. The second round retains only the two 30px visible items.
    // Each four-letter label advances 40px in the original 10px Ahem face.
    let (browser_x, native_x) = if rtl {
        (
            [0.0, 150.0, 110.0, 80.0, 30.0],
            [0.0, 150.0, 0.0, 120.0, 0.0],
        )
    } else {
        ([0.0, 0.0, 30.0, 70.0, 100.0], [0.0, 0.0, 0.0, 30.0, 0.0])
    };
    let browser_width = [180.0, 30.0, 40.0, 30.0, 50.0];
    let browser_height = [60.0, 20.0, 40.0, 25.0, 60.0];
    let native_width = [180.0, 30.0, 0.0, 30.0, 0.0];
    let native_height = [60.0, 20.0, 0.0, 25.0, 0.0];
    let browser_sizes = [
        [180, 60, 180, 60],
        [30, 20, 40, 20],
        [40, 40, 50, 40],
        [30, 25, 40, 25],
        [50, 60, 50, 60],
    ];
    let native_sizes = [
        [180, 60, 180, 60],
        [30, 20, 40, 20],
        [0, 0, 0, 0],
        [30, 25, 40, 25],
        [0, 0, 0, 0],
    ];
    let mut expected = Vec::new();
    for (index, path) in ["/", "/0", "/1", "/2", "/3"].into_iter().enumerate() {
        for (field, browser, native) in [
            (Geometry::X, browser_x[index], native_x[index]),
            (Geometry::Y, 0.0, 0.0),
            (Geometry::Width, browser_width[index], native_width[index]),
            (
                Geometry::Height,
                browser_height[index],
                native_height[index],
            ),
        ] {
            expected.push(
                ComparedField::layout(path.to_owned(), field, browser, Some(native)).unwrap(),
            );
        }
        for (field, (browser, native)) in [
            Element::ClientWidth,
            Element::ClientHeight,
            Element::ScrollWidth,
            Element::ScrollHeight,
        ]
        .into_iter()
        .zip(browser_sizes[index].into_iter().zip(native_sizes[index]))
        {
            expected.push(ComparedField::element(
                path.to_owned(),
                field,
                browser,
                native,
            ));
        }
    }
    assert_eq!(observed, expected);
}
