use crate::measurement::{self, DecodedMeasurement, MeasurementError, MeasurementErrorKind};
use serde_json::{Value, json};

fn decode_style(style: Value, unsupported: bool) -> Result<DecodedMeasurement, MeasurementError> {
    let mut node = json!({
        "layoutInput": "box", "gridAlignmentProfile":{"profile":1,"owners":[]},"useRounding": false,
        // Authored codec context, independent of this test's zero observation.
        "viewport": {"rootContext": {
            "kind": "block-flow-auto", "parentInlineExtent": 800, "parentDirection": "ltr",
            "fallback": {
                "initialContainingBlock": {"width": 800, "height": 600},
                "nearestScrollport": "none",
                "containingLimits": {"width": "unbounded", "height": "unbounded"}
            }
        }},
        "style": style, "unroundedLayout": {"x": 0, "y": 0, "width": 0, "height": 0},
        // Explicit zero-phase diagnostic for this authored Box codec fixture.
        "worldRoundedLayout": {"x": 0, "y": 0, "width": 0, "height": 0}, "children": []
    });
    if unsupported {
        node["unsupportedReason"] = json!("unsupported fixture");
    }
    measurement::decode(
        &node.to_string(),
        "style-contract",
        "border_box_ltr",
        measurement::BoxObservationRequest::Geometry,
    )
}

#[test]
fn unknown_style_discriminants_are_rejected_before_unsupported_classification() {
    for field in [
        "alignItems",
        "alignSelf",
        "justifyItems",
        "justifySelf",
        "alignContent",
        "justifyContent",
        "cssFloat",
        "clear",
        "textAlign",
        "verticalAlign",
    ] {
        for unsupported in [false, true] {
            let error =
                decode_style(json!({field: "definitely-invalid"}), unsupported).unwrap_err();
            assert_eq!(error.kind, MeasurementErrorKind::InvalidValue, "{field}");
            assert_eq!(error.node_path, "root");
            assert_eq!(error.field_path, format!("style.{field}"));
        }
    }
}

#[test]
fn structured_dimensions_and_tracks_require_property_valid_units() {
    for (style, field) in [
        (
            json!({"size": {"width": {"unit": "fraction", "value": 1}}}),
            "style.size.width",
        ),
        (
            json!({"maxSize": {"height": {"unit": "content"}}}),
            "style.maxSize.height",
        ),
        (
            json!({"padding": {"left": {"unit": "auto"}}}),
            "style.padding.left",
        ),
        (
            json!({"gap": {"row": {"unit": "max-content"}}}),
            "style.gap.row",
        ),
        (
            json!({"gridTemplateColumns": [{"kind": "function", "name": "minmax", "arguments": [{"kind": "scalar", "unit": "fraction", "value": 1}, {"kind": "scalar", "unit": "px", "value": 20}]}]}),
            "style.gridTemplateColumns[0].arguments[0]",
        ),
        (
            json!({"gridTemplateColumns": [{"kind": "function", "name": "fit-content", "arguments": [{"kind": "scalar", "unit": "fraction", "value": 1}]}]}),
            "style.gridTemplateColumns[0].arguments[0]",
        ),
    ] {
        for unsupported in [false, true] {
            let error = decode_style(style.clone(), unsupported).unwrap_err();
            assert_eq!(error.kind, MeasurementErrorKind::InvalidValue);
            assert_eq!(error.field_path, field);
        }
    }
}

fn assert_empty_explicit_template(
    empty_field: &str,
    empty_attribute: &str,
    sized_field: &str,
    sized_attribute: &str,
    flow: &str,
) {
    let input = json!({"style": {
        "display": "grid-lanes",
        "gridAutoFlow": {"direction": flow, "algorithm": "sparse"},
        empty_field: [{"kind": "scalar", "unit": "none"}],
        sized_field: [
            {"kind": "scalar", "unit": "px", "value": 30},
            {"kind": "scalar", "unit": "px", "value": 40}
        ],
    }});
    let attributes = measurement::style_attributes(&input.to_string()).unwrap();
    let attribute = |key| {
        attributes
            .iter()
            .find_map(|(name, value)| (*name == key).then_some(value.as_str()))
    };
    assert_eq!(attribute(empty_attribute), None);
    assert_eq!(attribute(sized_attribute), Some("30px 40px"));
    assert_eq!(attribute("grid-auto-flow"), Some(flow));
}

#[test]
fn explicit_template_none_columns_emits_no_column_tracks() {
    assert_empty_explicit_template(
        "gridTemplateColumns",
        "grid-template-columns",
        "gridTemplateRows",
        "grid-template-rows",
        "column",
    );
}

#[test]
fn explicit_template_none_rows_emits_no_row_tracks() {
    assert_empty_explicit_template(
        "gridTemplateRows",
        "grid-template-rows",
        "gridTemplateColumns",
        "grid-template-columns",
        "row",
    );
}

#[test]
fn explicit_template_none_refuses_mixed_nested_payload_and_auto_tracks() {
    let none = json!({"kind": "scalar", "unit": "none"});
    let sized = json!({"kind": "scalar", "unit": "px", "value": 30});
    for (field, tracks, suffix) in [
        ("gridTemplateColumns", json!([none, sized]), "[0]"),
        ("gridTemplateRows", json!([sized, none]), "[1]"),
        (
            "gridTemplateRows",
            json!([{"kind": "function", "name": "minmax", "arguments": [sized, none]}]),
            "[0].arguments[1]",
        ),
        (
            "gridTemplateColumns",
            json!([{"kind": "scalar", "unit": "none", "value": 0}]),
            "[0]",
        ),
        (
            "gridTemplateRows",
            json!([{"kind": "scalar", "unit": "none", "names": []}]),
            "[0]",
        ),
        ("gridAutoColumns", json!([none]), "[0]"),
        ("gridAutoRows", json!([none]), "[0]"),
    ] {
        for unsupported in [false, true] {
            let error = decode_style(json!({field: tracks}), unsupported).unwrap_err();
            assert_eq!(error.kind, MeasurementErrorKind::InvalidValue);
            assert_eq!(error.field_path, format!("style.{field}{suffix}"));
        }
    }
}

#[test]
fn nonnegative_style_extents_are_distinct_from_signed_offsets() {
    for (style, field) in [
        (
            json!({"padding": {"left": {"unit": "px", "value": -1}}}),
            "style.padding.left",
        ),
        (
            json!({"border": {"right": {"unit": "px", "value": -1}}}),
            "style.border.right",
        ),
        (
            json!({"gap": {"row": {"unit": "percent", "value": -0.1}}}),
            "style.gap.row",
        ),
        (
            json!({"fontSize": {"unit": "px", "value": -1}}),
            "style.fontSize",
        ),
        (
            json!({"lineHeight": {"unit": "px", "value": -1}}),
            "style.lineHeight",
        ),
        (
            json!({"flexBasis": {"unit": "px", "value": -1}}),
            "style.flexBasis",
        ),
        (
            json!({"size": {"width": {"unit": "px", "value": -1}}}),
            "style.size.width",
        ),
        (
            json!({"minSize": {"height": {"unit": "px", "value": -1}}}),
            "style.minSize.height",
        ),
        (
            json!({"maxSize": {"width": {"unit": "px", "value": -1}}}),
            "style.maxSize.width",
        ),
        (
            json!({"gridTemplateRows": [{"kind": "scalar", "unit": "percent", "value": -0.1}]}),
            "style.gridTemplateRows[0]",
        ),
    ] {
        let error = decode_style(style.clone(), false).unwrap_err();
        assert_eq!(error.kind, MeasurementErrorKind::InvalidValue);
        assert_eq!(error.field_path, field);
        assert!(matches!(
            decode_style(style, true).unwrap(),
            DecodedMeasurement::Unsupported { .. }
        ));
    }
    for style in [
        json!({"margin": {"left": {"unit": "px", "value": -1}}}),
        json!({"inset": {"top": {"unit": "percent", "value": -0.1}}}),
        json!({"logicalMargin": {"inlineStart": {"unit": "px", "value": -2}}}),
    ] {
        assert!(matches!(
            decode_style(style, false).unwrap(),
            DecodedMeasurement::Supported(_)
        ));
    }
}

#[test]
fn grid_area_matrices_require_equal_width_rectangular_named_regions() {
    for areas in [
        json!([["a", "a"], ["a", null]]),
        json!([["a"], ["b", "b"]]),
        json!([["auto"]]),
        json!([["bad/name"]]),
        json!([]),
    ] {
        for unsupported in [false, true] {
            let error = decode_style(json!({"gridTemplateAreas": areas}), unsupported).unwrap_err();
            assert_eq!(error.kind, MeasurementErrorKind::InvalidValue);
            assert!(error.field_path.starts_with("style.gridTemplateAreas"));
        }
    }
    let valid = json!({"gridTemplateAreas": [["head", "head"], ["nav", null]]});
    assert!(matches!(
        decode_style(valid, false).unwrap(),
        DecodedMeasurement::Supported(_)
    ));
}

#[test]
fn supported_alignment_and_symbolic_sizing_keep_their_spelling() {
    for (field, value) in [
        ("alignItems", "first baseline"),
        ("alignSelf", "last baseline"),
        ("justifySelf", "safe end"),
        ("alignContent", "space-evenly"),
        ("justifyContent", "unsafe end"),
    ] {
        assert!(matches!(
            decode_style(json!({field: value}), false).unwrap(),
            DecodedMeasurement::Supported(_)
        ));
    }
    let style = json!({"size": {"width": {"unit": "sizing", "value": "min(calc(70% - 8px), 160px)"}}, "gridTemplateColumns": [{"kind": "function", "name": "minmax", "arguments": [{"kind": "scalar", "unit": "px", "value": 0}, {"kind": "scalar", "unit": "fraction", "value": 1}]}]});
    let DecodedMeasurement::Supported(value) = decode_style(style, false).unwrap() else {
        panic!("supported style");
    };
    let xml = crate::xml::generate_xml("style-contract", &value);
    assert!(xml.contains("min(calc(70% - 8px), 160px)"));
    assert!(xml.contains("minmax(0px,1fr)"));
}

#[test]
fn xml_forbidden_style_characters_report_the_original_field() {
    for (style, field) in [
        (
            json!({"gridColumnStart": {"kind": "named-line", "name": "before\u{1}after"}}),
            "style.gridColumnStart.name",
        ),
        (
            json!({"gridTemplateRows": [{"kind": "line-names", "names": ["before\u{1}after"]}]}),
            "style.gridTemplateRows[0].names[0]",
        ),
        (
            json!({"gridTemplateColumns": [{"kind": "subgrid", "lineNames": [["before\u{1}after"]]}]}),
            "style.gridTemplateColumns[0].lineNames[0][0]",
        ),
        (
            json!({"size": {"width": {"unit": "sizing", "value": "calc(10px\u{1})"}}}),
            "style.size.width.value",
        ),
        (
            json!({"margin": {"left": {"unit": "calc", "value": "calc(10px\u{1})"}}}),
            "style.margin.left.value",
        ),
        (
            json!({"gridTemplateColumns": [{"kind": "scalar", "unit": "sizing", "value": "calc(10px\u{1})"}]}),
            "style.gridTemplateColumns[0].value",
        ),
        (
            json!({"scrollSnapType": "x\u{1} mandatory"}),
            "style.scrollSnapType",
        ),
        (json!({"order": "before\u{ffff}after"}), "style.order"),
    ] {
        let error = decode_style(style.clone(), false).unwrap_err();
        assert_eq!(error.kind, MeasurementErrorKind::InvalidValue);
        assert_eq!(error.node_path, "root");
        assert_eq!(error.field_path, field);
        assert!(matches!(
            decode_style(style, true).unwrap(),
            DecodedMeasurement::Unsupported { .. }
        ));
    }
}

#[test]
fn opaque_style_strings_preserve_xml_legal_characters() {
    let style = json!({"size": {"width": {"unit": "sizing", "value": "calc(10px\t +\n 1px\r)"}}, "gridColumnStart": {"kind": "named-line", "name": "边界😀"}});
    let DecodedMeasurement::Supported(value) = decode_style(style, false).unwrap() else {
        panic!("supported style");
    };
    let xml = crate::xml::generate_xml("style-contract", &value);
    assert!(xml.contains("calc(10px\t +\n 1px\r)"));
    assert!(xml.contains("边界😀"));
}
