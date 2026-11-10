use super::{
    Golden, TestTree, capture_authority::sha256, compute_actual_batch, flex_composition, layout,
    root_request,
};
use layout::LayoutTree as _;

const SOURCE: &str = "html/flex/fri07_flex_composition.html";
const SOURCE_BYTES: &[u8] = include_bytes!("html/flex/fri07_flex_composition.html");
const REPORT: &str =
    include_str!("captured/flex-composition-differences/xml/generation-reports/all.json");
const VARIANTS: [(&str, &str); 4] = [
    (
        "border_box_ltr",
        include_str!(
            "captured/flex-composition-differences/xml/flex/fri07_flex_composition__border_box_ltr.xml"
        ),
    ),
    (
        "border_box_rtl",
        include_str!(
            "captured/flex-composition-differences/xml/flex/fri07_flex_composition__border_box_rtl.xml"
        ),
    ),
    (
        "content_box_ltr",
        include_str!(
            "captured/flex-composition-differences/xml/flex/fri07_flex_composition__content_box_ltr.xml"
        ),
    ),
    (
        "content_box_rtl",
        include_str!(
            "captured/flex-composition-differences/xml/flex/fri07_flex_composition__content_box_rtl.xml"
        ),
    ),
];

#[test]
fn geometry_replay_refuses_a_replaced_image_witness_without_capture_authority() {
    let error = Golden::parse(VARIANTS[1].1)
        .expect_err("geometry replay must refuse unbound image evidence");
    assert!(
        error
            .to_string()
            .contains("Flex composition image requires capture authority"),
        "{error}"
    );
}

#[test]
fn replaced_image_capture_refuses_changed_source_output_or_natural_dimensions() {
    let (variant, xml) = VARIANTS[0];
    let output = format!("xml/flex/fri07_flex_composition__{variant}.xml");
    Golden::parse_with_flex_composition(xml, REPORT, SOURCE, SOURCE_BYTES, &output).unwrap();
    assert!(
        Golden::parse_with_flex_composition(xml, REPORT, SOURCE, b"different source", &output)
            .unwrap_err()
            .to_string()
            .contains("original source changed")
    );
    let altered = xml.replace("natural-width=\"80\"", "natural-width=\"40\"");
    assert!(
        Golden::parse_with_flex_composition(&altered, REPORT, SOURCE, SOURCE_BYTES, &output)
            .unwrap_err()
            .to_string()
            .contains("output binding changed")
    );
    let forged = REPORT.replace(&sha256(xml.as_bytes()), &sha256(altered.as_bytes()));
    assert!(
        Golden::parse_with_flex_composition(&altered, &forged, SOURCE, SOURCE_BYTES, &output)
            .unwrap_err()
            .to_string()
            .contains("incompatible root or natural dimensions")
    );
}

#[test]
fn reissued_capture_cannot_change_source_projection() {
    let (variant, xml) = VARIANTS[0];
    let output = format!("xml/flex/fri07_flex_composition__{variant}.xml");
    for (original, replacement) in [
        ("order=\"-1\"", "order=\"99\""),
        (
            "hypothetical-inline-offset=\"0px\"",
            "hypothetical-inline-offset=\"1px\"",
        ),
    ] {
        let altered = xml.replacen(original, replacement, 1);
        assert_ne!(altered, xml);
        let forged = REPORT.replace(&sha256(xml.as_bytes()), &sha256(altered.as_bytes()));
        assert!(
            Golden::parse_with_flex_composition(&altered, &forged, SOURCE, SOURCE_BYTES, &output)
                .expect_err("reissued report cannot change the independently retained input")
                .to_string()
                .contains("normalized source projection changed")
        );
    }
}

#[test]
fn visible_sideways_scroller_reserves_its_logical_end_gutter_with_or_without_collapse() {
    for (variant, xml) in VARIANTS {
        let output = format!("xml/flex/fri07_flex_composition__{variant}.xml");
        let golden =
            Golden::parse_with_flex_composition(xml, REPORT, SOURCE, SOURCE_BYTES, &output)
                .unwrap();
        for disable_collapse in [false, true] {
            let mut tree =
                TestTree::from_golden(&golden.root, Some(&golden.viewport.root_context)).unwrap();
            flex_composition::lower(
                &mut tree,
                &golden,
                golden.flex_composition_image.as_ref().unwrap(),
            )
            .unwrap();
            let [_, image, collapsed, scroller] = tree.nodes[0].children[..] else {
                panic!("original Flex root must retain its four source children");
            };
            let child = tree.nodes[scroller].children[0];
            let hidden_descendant = tree.nodes[collapsed].children[0];
            if disable_collapse {
                for node in [collapsed, hidden_descendant] {
                    let mut input = tree.nodes[node].layout_input.as_box().unwrap().clone();
                    input.flex_item_collapse = layout::FlexItemCollapse::Normal;
                    tree.nodes[node].layout_input = layout::LayoutInput::box_input(input);
                }
            }

            let image_input = tree.nodes[image].layout_input.as_box().unwrap();
            assert!(image_input.item_is_replaced);
            assert_eq!(image_input.aspect_ratio.unwrap().get(), 2.0);
            assert!(tree.has_leaf_measurement(image));

            let batch = compute_actual_batch(
                &tree,
                &golden.name,
                root_request(&golden.viewport.root_context).unwrap(),
            )
            .unwrap();
            let rtl = variant.ends_with("_rtl");
            let content_top = if rtl { 15.0 } else { 0.0 };
            for phase in [
                super::observation::phase(&batch, false),
                super::observation::phase(&batch, true),
            ] {
                let packet =
                    |id| super::observation::observe(phase, id, tree.expected_publication_kind(id));
                let collapsed_size = if disable_collapse {
                    layout::Size::new(100.0, 30.0)
                } else {
                    layout::Size::new(0.0, 0.0)
                };
                assert_eq!(packet(collapsed).size, collapsed_size);
                // Authored 40×80 remains the layout size even with the natural ratio 2.
                assert_eq!(packet(image).size, layout::Size::new(40.0, 80.0));

                let visible = packet(scroller);
                assert_eq!(visible.size, layout::Size::new(70.0, 60.0));
                let scroll = visible.scroll_geometry.unwrap();
                let content_origin = layout::Point::new(0.0, content_top);
                let client_size = layout::Size::new(70.0, 45.0);
                assert_eq!(scroll.content_box().origin(), content_origin);
                assert_eq!(scroll.content_box().size(), client_size);
                assert_eq!(scroll.scrollport().origin(), content_origin);
                assert_eq!(scroll.scrollport().size(), client_size);
                assert_eq!(scroll.scrollbar_size(), layout::Size::new(0.0, 15.0));
                let gutters = scroll.gutters();
                let gutter = if rtl { gutters.top() } else { gutters.bottom() }.unwrap();
                assert_eq!(
                    gutter.origin(),
                    layout::Point::new(0.0, if rtl { 0.0 } else { 45.0 })
                );
                assert_eq!(gutter.size(), layout::Size::new(70.0, 15.0));
                assert!(gutters.left().is_none() && gutters.right().is_none());
                assert!(if rtl {
                    gutters.bottom().is_none()
                } else {
                    gutters.top().is_none()
                });
                let ranges = scroll.physical_range();
                assert_eq!((ranges.x().minimum(), ranges.x().maximum()), (-50.0, 0.0));
                assert_eq!((ranges.y().minimum(), ranges.y().maximum()), (0.0, 0.0));
                let packet = packet(child);
                assert_eq!(packet.location, layout::Point::new(-50.0, content_top));
                assert_eq!(packet.size, layout::Size::new(120.0, 45.0));
            }
        }
    }
}

#[test]
fn bound_replaced_image_reaches_native_geometry_comparison_in_all_variants() {
    for (variant, xml) in VARIANTS {
        let output = format!("xml/flex/fri07_flex_composition__{variant}.xml");
        let golden =
            Golden::parse_with_flex_composition(xml, REPORT, SOURCE, SOURCE_BYTES, &output)
                .unwrap();
        let error = super::assert_surgeist_matches(&golden).unwrap_err();
        let difference = if variant.ends_with("_rtl") {
            "/0: y mismatch, expected 134, got 113"
        } else {
            "/0: y mismatch, expected 6, got 27"
        };
        assert!(error.to_string().contains(difference), "{variant}: {error}");
    }
}

#[test]
fn complete_comparison_keeps_absent_native_scroll_spans_and_visits_later_boxes() {
    use super::{
        NumericComparisonSink, compare_captured_with_batch, exact_differences::ComparedField,
    };
    use serde_json::json;

    for (variant, xml) in VARIANTS {
        let output = format!("xml/flex/fri07_flex_composition__{variant}.xml");
        let golden =
            Golden::parse_with_flex_composition(xml, REPORT, SOURCE, SOURCE_BYTES, &output)
                .unwrap();
        let mut sink = NumericComparisonSink::CollectAll {
            root: golden.name.clone(),
            fields: Vec::new(),
        };
        compare_captured_with_batch(&golden, &mut sink, |_, _| Ok(()))
            .unwrap_or_else(|error| panic!("{variant}: {error}"));
        let NumericComparisonSink::CollectAll { fields, .. } = sink else {
            unreachable!()
        };
        assert_eq!(fields.len(), 34, "{variant}");

        for (path, field, browser, native) in [
            ("/2", "scroll-width", 40, None),
            ("/2", "scroll-height", 0, None),
            ("/2/0", "width", 140, Some(0)),
            ("/3", "scroll-width", 50, Some(50)),
            (
                "/3/0",
                "y",
                0,
                Some(if variant.ends_with("_rtl") { 15 } else { 0 }),
            ),
        ] {
            let expected = serde_json::from_value::<ComparedField>(json!({
                "domain": "layout", "path": path, "field": field,
                "browser": browser, "native": native,
            }))
            .unwrap();
            assert!(fields.contains(&expected), "{variant}: {path} {field}");
        }
    }
}

#[test]
fn sideways_flex_binding_refuses_changed_browser_environment() {
    let (variant, xml) = VARIANTS[0];
    let output = format!("xml/flex/fri07_flex_composition__{variant}.xml");
    let original: serde_json::Value = serde_json::from_str(REPORT).unwrap();
    for pointer in [
        "/metadata/browser_executable",
        "/metadata/browser_executable_sha256",
        "/metadata/browser_provenance",
        "/metadata/engine_version",
        "/metadata/contract_sha256",
        "/metadata/inputs/browser-owner.toml",
    ] {
        let mut changed = original.clone();
        *changed.pointer_mut(pointer).unwrap() = serde_json::json!("a".repeat(64));
        let error = Golden::parse_with_flex_composition(
            xml,
            &changed.to_string(),
            SOURCE,
            SOURCE_BYTES,
            &output,
        )
        .expect_err("capture environment must match the retained control");
        assert!(
            error.to_string().contains("capture environment changed"),
            "{pointer}: {error}"
        );
    }

    let mut rebuilt = original;
    rebuilt["metadata"]["host_executable_sha256"] = serde_json::json!("b".repeat(64));
    Golden::parse_with_flex_composition(xml, &rebuilt.to_string(), SOURCE, SOURCE_BYTES, &output)
        .expect("a rebuilt qualifier can replay the same browser environment");
}

include!("flex_composition_script_custody_tests.rs");
