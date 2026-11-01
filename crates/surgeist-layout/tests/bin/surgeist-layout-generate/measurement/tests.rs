use super::*;
use serde_json::json;

fn box_fixture() -> serde_json::Value {
    json!({"layoutInput":"box","tagName":"div","useRounding":false,"viewport":{"rootContext":"root","width":{"unit":"max-content"},"height":{"unit":"max-content"}},"style":{},"unroundedLayout":{"x":0,"y":0,"width":20,"height":10},"children":[]})
}
fn decode_fixture(value: &serde_json::Value) -> Result<DecodedMeasurement, MeasurementError> {
    decode(&value.to_string(), "fixture/case", "border-box-ltr")
}
fn supported(value: &serde_json::Value) -> ValidatedMeasurement {
    match decode_fixture(value).expect("valid fixture") {
        DecodedMeasurement::Supported(value) => value,
        DecodedMeasurement::Unsupported { reason } => panic!("unexpected unsupported {reason}"),
    }
}
#[test]
fn malformed_geometry_returns_location_and_json_source() {
    let mut value = box_fixture();
    value["unroundedLayout"]["width"] = json!("invalid");
    let error = decode_fixture(&value).unwrap_err();
    assert_eq!(error.kind, MeasurementErrorKind::Decode);
    assert_eq!(error.case_id, "fixture/case");
    assert_eq!(error.variant, "border-box-ltr");
    assert_eq!(error.field_path, "unroundedLayout.width");
    assert!(std::error::Error::source(&error).is_some());
}
#[test]
fn unknown_and_duplicate_fields_are_rejected_before_unsupported_classification() {
    for raw in [
        r#"{"layoutInput":"unsupported","unsupportedReason":"known","unknown":0}"#,
        r#"{"layoutInput":"unsupported","unsupportedReason":"first","unsupportedReason":"second"}"#,
    ] {
        assert_eq!(
            decode(raw, "case", "variant").unwrap_err().kind,
            MeasurementErrorKind::Decode
        );
    }
    let raw = r#"{"layoutInput":"box","unsupportedReason":"known","style":{"size":{"width":{"unit":"px","unit":"auto","value":2}}}}"#;
    assert_eq!(
        decode(raw, "case", "variant").unwrap_err().kind,
        MeasurementErrorKind::Decode
    );
}
#[test]
fn unsupported_reason_precedence_does_not_require_supported_geometry() {
    let value = json!({"layoutInput":"box","unsupportedReason":"root","children":[{"layoutInput":"unsupported","unsupportedReason":"child"}]});
    assert!(
        matches!(decode_fixture(&value),Ok(DecodedMeasurement::Unsupported{reason}) if reason=="root")
    );
    let value = json!({"layoutInput":"box","children":[{"layoutInput":"box","children":[{"layoutInput":"unsupported","unsupportedReason":"first"}]},{"layoutInput":"unsupported","unsupportedReason":"second"}]});
    assert!(
        matches!(decode_fixture(&value),Ok(DecodedMeasurement::Unsupported{reason}) if reason=="first")
    );
}
#[test]
fn required_supported_fields_never_default() {
    for field in ["useRounding", "viewport", "children", "unroundedLayout"] {
        let mut value = box_fixture();
        value.as_object_mut().unwrap().remove(field);
        let error = decode_fixture(&value).unwrap_err();
        assert_eq!(error.kind, MeasurementErrorKind::MissingField);
        assert_eq!(error.field_path, field);
    }
}
#[test]
fn geometry_extents_and_f32_conversion_are_validated_before_formatting() {
    for (value, kind) in [
        (-1.0, MeasurementErrorKind::InvalidValue),
        (f64::from(f32::MAX) * 2.0, MeasurementErrorKind::OutOfRange),
    ] {
        let mut fixture = box_fixture();
        fixture["unroundedLayout"]["width"] = json!(value);
        assert_eq!(decode_fixture(&fixture).unwrap_err().kind, kind);
    }
    let mut fixture = box_fixture();
    fixture["unroundedLayout"]["width"] = json!(f64::from(f32::MAX));
    let measurement = supported(&fixture);
    assert!(
        crate::xml::generate_xml("representable", &measurement)
            .contains(&format!("width=\"{:.0}\"", f32::MAX))
    );
}
#[test]
fn dimension_units_values_and_track_arity_are_closed() {
    for dimension in [
        json!({"unit":"px"}),
        json!({"unit":"px","value":"3"}),
        json!({"unit":"auto","value":0}),
        json!({"unit":"em","value":2}),
        json!({"unit":"percent","value":f64::MAX}),
    ] {
        let mut value = box_fixture();
        value["style"]["size"] = json!({"width":dimension});
        assert!(decode_fixture(&value).is_err());
    }
    for arguments in [
        json!([]),
        json!([{"kind":"scalar","unit":"px","value":1}]),
        json!([{"kind":"scalar","unit":"px","value":1},{"kind":"scalar","unit":"px","value":2},{"kind":"scalar","unit":"px","value":3}]),
    ] {
        let mut value = box_fixture();
        value["style"]["gridTemplateColumns"] =
            json!([{"kind":"function","name":"minmax","arguments":arguments}]);
        let error = decode_fixture(&value).unwrap_err();
        assert_eq!(error.kind, MeasurementErrorKind::InvalidValue);
        assert_eq!(error.field_path, "style.gridTemplateColumns[0].arguments");
    }
}
#[test]
fn inline_metrics_are_coupled_and_zero_height_remains_valid() {
    let mut fixture = box_fixture();
    fixture["style"]["inlineMetrics"] = json!({"baseline":0,"lineHeight":0});
    assert!(decode_fixture(&fixture).is_ok());
    fixture["style"]["inlineMetrics"] = json!({"baseline":3,"lineHeight":2});
    let error = decode_fixture(&fixture).unwrap_err();
    assert_eq!(error.field_path, "style.inlineMetrics.lineHeight");
    fixture["style"]["inlineMetrics"] = json!({"baseline":2});
    assert_eq!(
        decode_fixture(&fixture).unwrap_err().kind,
        MeasurementErrorKind::Decode
    );
}
#[test]
fn old_metric_strings_and_flat_shape_endpoints_are_not_v1() {
    let mut value = box_fixture();
    value["style"]["inlineBaseline"] = json!("8px");
    assert_eq!(
        decode_fixture(&value).unwrap_err().kind,
        MeasurementErrorKind::Decode
    );
    let mut value = box_fixture();
    value["shapeBands"] =
        json!([{"bandMinimum":0,"bandMaximum":10,"intervalMinimum":0,"intervalMaximum":20}]);
    assert_eq!(
        decode_fixture(&value).unwrap_err().kind,
        MeasurementErrorKind::Decode
    );
}
#[test]
fn optional_shape_interval_and_explicit_empty_observations_are_distinct() {
    let mut value = box_fixture();
    value["shapeBands"] = json!([{"bandMinimum":0,"bandMaximum":10},{"bandMinimum":10,"bandMaximum":20,"interval":{"minimum":2,"maximum":8}}]);
    value["fragments"] = json!([]);
    let xml = crate::xml::generate_xml("shape", &supported(&value));
    assert!(xml.contains("<fragments/>"));
    assert!(xml.contains("<shape-band band-minimum=\"0\" band-maximum=\"10\"/>"));
    assert!(xml.contains("interval-minimum=\"2\" interval-maximum=\"8\""));
    value.as_object_mut().unwrap().remove("fragments");
    assert!(!crate::xml::generate_xml("absent", &supported(&value)).contains("<fragments"));
    value["fragments"] = serde_json::Value::Null;
    assert_eq!(
        decode_fixture(&value).unwrap_err().kind,
        MeasurementErrorKind::Decode
    );
}
#[test]
fn range_observations_cannot_mix_with_box_geometry() {
    let mut root = box_fixture();
    root["children"] = json!([{"layoutInput":"inline-text","inlineSegments":[{"id":u64::MAX,"inlineExtent":0,"inlineBaseline":0,"inlineLineHeight":0,"bidiLevel":126,"whitespaceEdge":"preserve","followingBreak":"prohibited"}],"rangeInks":[],"children":[]}]);
    let xml = crate::xml::generate_xml("empty-range", &supported(&root));
    assert!(xml.contains("<range-inks/>"));
    assert!(xml.contains(&u64::MAX.to_string()));
    root["children"][0]["unroundedLayout"] = json!({"x":0,"y":0,"width":0,"height":0});
    let error = decode_fixture(&root).unwrap_err();
    assert_eq!(error.kind, MeasurementErrorKind::ContradictoryFields);
    assert_eq!(error.node_path, "root.children[0]");
    assert_eq!(error.field_path, "rangeInks");
}
#[test]
fn boundary_metrics_and_end_payload_are_checked() {
    let mut root = box_fixture();
    root["children"] = json!([{"layoutInput":"inline-boundary","inlineBoundary":{"kind":"start","baseline":0,"lineHeight":0},"children":[]}]);
    assert_eq!(
        decode_fixture(&root).unwrap_err().field_path,
        "inlineBoundary.lineHeight"
    );
    root["children"][0]["inlineBoundary"] = json!({"kind":"start","baseline":2,"lineHeight":3});
    assert!(decode_fixture(&root).is_ok());
    root["children"][0]["inlineBoundary"]["kind"] = json!("end");
    assert_eq!(
        decode_fixture(&root).unwrap_err().kind,
        MeasurementErrorKind::ContradictoryFields
    );
}

#[test]
fn null_text_and_foreign_role_payload_are_rejected() {
    let mut root = box_fixture();
    root["textContent"] = serde_json::Value::Null;
    assert_eq!(
        decode_fixture(&root).unwrap_err().kind,
        MeasurementErrorKind::Decode
    );
    let reason = json!({"layoutInput":"unsupported","unsupportedReason":"known","style":{}});
    assert_eq!(
        decode_fixture(&reason).unwrap_err().kind,
        MeasurementErrorKind::ContradictoryFields
    );
    let mut root = box_fixture();
    root["children"] = json!([{"layoutInput":"inline-boundary","inlineBoundary":{"kind":"start"},"style":{},"children":[]}]);
    assert_eq!(
        decode_fixture(&root).unwrap_err().kind,
        MeasurementErrorKind::ContradictoryFields
    );
}

#[test]
fn unsupported_measurements_skip_metric_semantics_after_closed_wire_validation() {
    let fixture = json!({"layoutInput":"box","unsupportedReason":"outside layout","style":{"inlineMetrics":{"baseline":3,"lineHeight":2}},"unroundedLayout":{"width":-1}});
    assert!(
        matches!(decode_fixture(&fixture),Ok(DecodedMeasurement::Unsupported{reason}) if reason=="outside layout")
    );
}
#[test]
fn duplicate_shape_queries_are_rejected() {
    let mut fixture = box_fixture();
    fixture["shapeBands"] = json!([{"bandMinimum":0,"bandMaximum":10},{"bandMinimum":0,"bandMaximum":10,"interval":{"minimum":0,"maximum":5}}]);
    assert_eq!(
        decode_fixture(&fixture).unwrap_err().field_path,
        "shapeBands[1]"
    );
}

#[test]
fn unsupported_roles_require_reasons_and_cannot_hide_foreign_inline_payloads() {
    let mut fixture = box_fixture();
    fixture["layoutInput"] = json!("unsupported");
    let error = decode_fixture(&fixture).unwrap_err();
    assert_eq!(error.kind, MeasurementErrorKind::MissingField);
    assert_eq!(error.field_path, "unsupportedReason");
    for role in ["inline-text", "inline-boundary"] {
        let value = json!({"layoutInput":role,"unsupportedReason":"hidden"});
        let error = decode_fixture(&value).unwrap_err();
        assert_eq!(error.kind, MeasurementErrorKind::ContradictoryFields);
        assert_eq!(error.field_path, "unsupportedReason");
    }
    let fixture = json!({"layoutInput":"box","unsupportedReason":"hidden","inlineSegments":[]});
    let error = decode_fixture(&fixture).unwrap_err();
    assert_eq!(error.kind, MeasurementErrorKind::ContradictoryFields);
    assert_eq!(error.field_path, "layoutInput");
}

#[test]
fn unsupported_ancestors_do_not_hide_conflicting_inline_observation_forms() {
    let value = json!({"layoutInput":"box","unsupportedReason":"outside layout","children":[{"layoutInput":"inline-text","children":[],"rangeInks":[],"unroundedLayout":{"width":-1}}]});
    let error = decode_fixture(&value).unwrap_err();
    assert_eq!(error.kind, MeasurementErrorKind::ContradictoryFields);
    assert_eq!(error.node_path, "root.children[0]");
    assert_eq!(error.field_path, "rangeInks");
}

#[test]
fn root_atomic_participation_requires_a_parent_input_slot() {
    let mut value = box_fixture();
    value["atomicInlineParticipation"] = json!({"bidiLevel":0,"followingBreak":"prohibited"});
    let error = decode_fixture(&value).unwrap_err();
    assert_eq!(error.kind, MeasurementErrorKind::ContradictoryFields);
    assert_eq!(error.node_path, "root");
    assert_eq!(error.field_path, "atomicInlineParticipation");
}

#[test]
fn scrolling_uses_the_root_selected_geometry_observation() {
    let mut value = box_fixture();
    value["useRounding"] = json!(true);
    value["style"]["overflowX"] = json!("auto");
    value["unroundedLayout"] =
        json!({"x":0,"y":0,"width":20,"height":10,"scrollWidth":30,"scrollHeight":10});
    value["smartRoundedLayout"] =
        json!({"x":0,"y":0,"width":20,"height":10,"scrollWidth":50,"scrollHeight":10});
    value["naivelyRoundedLayout"] = json!({"clientWidth":20,"clientHeight":10});
    assert!(
        crate::xml::generate_xml("rounded", &supported(&value)).contains("scroll_width=\"30\"")
    );
    value["useRounding"] = json!(false);
    assert!(
        crate::xml::generate_xml("unrounded", &supported(&value)).contains("scroll_width=\"10\"")
    );
}

#[test]
fn unsupported_classification_does_not_bypass_structural_alternatives() {
    let cases = [
        (
            json!({"layoutInput":"box","unsupportedReason":"known","children":[{"layoutInput":"inline-boundary","inlineBoundary":{"kind":"end","baseline":0,"lineHeight":1},"children":[]}]}),
            "inlineBoundary.kind",
        ),
        (
            json!({"layoutInput":"box","unsupportedReason":"known","viewport":{"rootContext":"root","width":{"unit":"max-content"},"height":{"unit":"max-content"},"hostInlineSize":20}}),
            "viewport.rootContext",
        ),
        (
            json!({"layoutInput":"box","unsupportedReason":"known","children":[{"layoutInput":"inline-text","inlineSegments":[{"id":0,"inlineExtent":1,"inlineBaseline":0,"inlineLineHeight":1,"bidiLevel":0,"whitespaceEdge":"preserve","followingBreak":"prohibited","replacementInlineExtent":1}],"rangeInks":[],"children":[]}]}),
            "inlineSegments[0].replacementInlineExtent",
        ),
    ];
    for (value, field) in cases {
        let error = decode_fixture(&value).unwrap_err();
        assert_eq!(error.kind, MeasurementErrorKind::ContradictoryFields);
        assert_eq!(error.field_path, field);
    }
    let mut root = box_fixture();
    root["unsupportedReason"] = json!("known");
    root["viewport"] = json!({"rootContext":"flex-item","width":{"unit":"max-content"},"height":{"unit":"max-content"},"parentWritingMode":"horizontal-tb","parentDirection":"bogus","hostInlineSize":-1});
    let error = decode_fixture(&root).unwrap_err();
    assert_eq!(error.kind, MeasurementErrorKind::InvalidValue);
    assert_eq!(error.field_path, "viewport.parentDirection");
}

#[test]
fn block_br_used_extents_keep_width_height_order_before_insets() {
    let mut root = box_fixture();
    root["children"] = json!([{"layoutInput":"box","tagName":"br","style":{"display":"block","size":{"height":{"unit":"px","value":1}},"inset":{"left":{"unit":"px","value":3}}},"unroundedLayout":{"x":0,"y":0,"width":20,"height":10},"children":[]}]);
    let xml = crate::xml::generate_xml("block-br", &supported(&root));
    assert!(
        xml.contains(
            "<div source-tag=\"br\" display=\"block\" width=\"20px\" height=\"10px\" left=\"3px\"/>"
        ),
        "{xml}"
    );
}

#[test]
fn invalid_xml_characters_are_rejected_before_serialization() {
    for field in ["textContent", "tagName"] {
        let mut value = box_fixture();
        value[field] = json!("invalid\u{0001}text");
        let error = decode_fixture(&value).unwrap_err();
        assert_eq!(error.kind, MeasurementErrorKind::InvalidValue);
        assert_eq!(error.field_path, field);
    }
}
#[test]
fn cdata_terminators_in_text_are_escaped_without_changing_text() {
    let mut value = box_fixture();
    value["textContent"] = json!("before ]]> after");
    let xml = crate::xml::generate_xml("text", &supported(&value));
    let document = roxmltree::Document::parse(&xml).expect("generated XML must be well formed");
    let text = document
        .descendants()
        .find(|node| node.has_tag_name("text"))
        .unwrap()
        .text()
        .unwrap()
        .trim();
    assert_eq!(text, "before ]]> after");
}

#[test]
fn viewport_constraints_use_available_space_alternatives() {
    for dimension in [
        json!({"unit":"auto"}),
        json!({"unit":"percent","value":0.5}),
        json!({"unit":"fraction","value":1}),
    ] {
        let mut value = box_fixture();
        value["viewport"]["width"] = dimension;
        value["unsupportedReason"] = json!("known");
        let error = decode_fixture(&value).unwrap_err();
        assert_eq!(error.kind, MeasurementErrorKind::InvalidValue);
        assert_eq!(error.field_path, "viewport.width");
    }
    let mut value = box_fixture();
    value["viewport"]["width"] = json!({"unit":"px","value":-1});
    assert_eq!(
        decode_fixture(&value).unwrap_err().field_path,
        "viewport.width"
    );
    value["unsupportedReason"] = json!("known");
    assert!(matches!(
        decode_fixture(&value),
        Ok(DecodedMeasurement::Unsupported { .. })
    ));
}

#[test]
fn resolved_bidi_126_is_admitted_but_127_refuses_at_the_actual_measurement_field() {
    let mut root = box_fixture();
    root["children"] = json!([{"layoutInput":"inline-text","inlineSegments":[{"id":1,"inlineExtent":1,"inlineBaseline":0,"inlineLineHeight":1,"bidiLevel":126,"whitespaceEdge":"preserve","followingBreak":"prohibited"}],"rangeInks":[],"children":[]}]);
    let xml = crate::xml::generate_xml("resolved-level", &supported(&root));
    assert!(xml.contains("bidi-level=\"126\""));
    root["children"][0]["inlineSegments"][0]["bidiLevel"] = json!(127);
    let error = decode_fixture(&root).unwrap_err();
    assert_eq!(error.kind, MeasurementErrorKind::OutOfRange);
    assert_eq!(error.field_path, "inlineSegments[0].bidiLevel");
}

fn prepared_fixture() -> serde_json::Value {
    let style = json!({"fontSize":10.0,"lineHeight":12.0,"whitespace":"collapse","wrap":"word","letterSpacing":0.0,"wordSpacing":0.0,"color":[0.0,0.0,0.0,1.0],"locale":"en"});
    let mut fixture = box_fixture();
    fixture["layoutReadyInlineRoot"] = json!(true);
    fixture["preparedParagraph"] = json!({"font":{"id":"ahem-ttf-f0a92cd0","sha256":"f0a92cd0cc45735591c9b5b1fa8aecd5194e8dc518895ca22af94a46c23550dc","faceIndex":0,"family":"SurgeistParityAhem","style":"normal","weight":400,"stretch":"normal","features":"normal","variations":"normal","synthesis":"none","loaded":true},"direction":"ltr","writingMode":"horizontal-tb","baseStyle":style,"origins":[{"child":0,"units":[88],"start":0,"end":1,"style":style}],"connections":[],"atomics":[],"controls":[],"order":[{"kind":"origin","index":0,"affinity":"before"}]});
    fixture["children"] = json!([{"layoutInput":"prepared-text","preparedOccurrence":0,"rangeFragments":[{"origin":0,"line":0,"physicalStartEdge":"left","start":123.0,"advance":999.0}],"children":[]}]);
    fixture
}

fn prepared_control_fixture() -> serde_json::Value {
    let mut fixture = prepared_fixture();
    let text = fixture["children"][0].clone();
    fixture["preparedParagraph"]["origins"][0]["child"] = json!(1);
    fixture["preparedParagraph"]["controls"] = json!([{"child":0,"at":0,"kind":"forced-break"}]);
    fixture["preparedParagraph"]["order"] = json!([
        {"kind":"control","index":0,"affinity":"before"},
        {"kind":"origin","index":0,"affinity":"before"}
    ]);
    fixture["children"] = json!([
        {"layoutInput":"box","tagName":"br","style":{"display":"inline"},
         "lineControlParticipation":{"kind":"forced-break"},
         "unroundedLayout":{"x":0,"y":0,"width":0,"height":12},"children":[]},
        text
    ]);
    fixture
}

#[test]
fn prepared_controls_accept_actual_break_and_transparent_roles() {
    let forced = supported(&prepared_control_fixture());
    assert!(
        crate::xml::generate_xml("actual-break", &forced).contains("line-control=\"forced-break\"")
    );

    let mut transparent = prepared_control_fixture();
    transparent["preparedParagraph"]["controls"] = json!([
        {"child":0,"at":0,"kind":"inline-boundary-start"},
        {"child":2,"at":1,"kind":"inline-boundary-end"}
    ]);
    transparent["preparedParagraph"]["order"] = json!([
        {"kind":"control","index":0,"affinity":"before"},
        {"kind":"origin","index":0,"affinity":"before"},
        {"kind":"control","index":1,"affinity":"before"}
    ]);
    transparent["children"][0] = json!({"layoutInput":"inline-boundary",
        "inlineBoundary":{"kind":"start"},"children":[]});
    transparent["children"].as_array_mut().unwrap().push(json!({
        "layoutInput":"inline-boundary","inlineBoundary":{"kind":"end"},"children":[]
    }));
    let xml = crate::xml::generate_xml("actual-transparent", &supported(&transparent));
    assert!(xml.contains("<inline-boundary kind=\"start\""));
    assert!(xml.contains("<inline-boundary kind=\"end\""));
}

#[test]
fn prepared_control_capture_rejects_mismatched_actual_child_before_xml() {
    let mut ordinary_box = prepared_control_fixture();
    // The capture still claims a forced break at the same authentic child and
    // source anchor, but that child is an ordinary box, not an inline BR.
    ordinary_box["children"][0] = json!({"layoutInput":"box","tagName":"span",
        "style":{"display":"inline-block"},
        "unroundedLayout":{"x":0,"y":0,"width":10,"height":12},"children":[]});
    let mut wrong_boundary = prepared_control_fixture();
    wrong_boundary["preparedParagraph"]["controls"][0]["kind"] = json!("inline-boundary-start");
    wrong_boundary["children"][0] = json!({"layoutInput":"inline-boundary",
        "inlineBoundary":{"kind":"end"},"children":[]});
    for fixture in [ordinary_box, wrong_boundary] {
        let error = decode_fixture(&fixture)
            .expect_err("a captured control must match its actual child before XML publication");
        assert_eq!(error.kind, MeasurementErrorKind::InvalidValue);
        assert_eq!(error.node_path, "root");
        assert_eq!(error.field_path, "preparedParagraph.controls");
    }
}

fn prepared_boundary_fixture(kinds: &[&str]) -> serde_json::Value {
    let mut fixture = prepared_fixture();
    fixture["preparedParagraph"]["origins"] = json!([]);
    fixture["preparedParagraph"]["controls"] = serde_json::Value::Array(
        kinds.iter().enumerate().map(|(child, kind)| {
            json!({"child":child,"at":0,"kind":format!("inline-boundary-{kind}")})
        }).collect(),
    );
    fixture["preparedParagraph"]["order"] = serde_json::Value::Array(
        (0..kinds.len())
            .map(|index| json!({"kind":"control","index":index,"affinity":"before"}))
            .collect(),
    );
    fixture["children"] = serde_json::Value::Array(kinds.iter().map(|kind| {
        json!({"layoutInput":"inline-boundary","inlineBoundary":{"kind":kind},"children":[]})
    }).collect());
    fixture
}

#[test]
fn prepared_controls_accept_nested_boundaries_in_actual_occurrence_order() {
    let mut fixture = prepared_boundary_fixture(&["start", "start", "end", "end"]);
    // Capture-array storage order is not the authoritative actual child order.
    fixture["preparedParagraph"]["controls"]
        .as_array_mut()
        .unwrap()
        .reverse();
    for (position, occurrence) in fixture["preparedParagraph"]["order"]
        .as_array_mut()
        .unwrap()
        .iter_mut()
        .enumerate()
    {
        occurrence["index"] = json!(3 - position);
    }
    let xml = crate::xml::generate_xml("nested-transparent", &supported(&fixture));
    assert_eq!(xml.matches("<inline-boundary kind=\"start\"").count(), 2);
    assert_eq!(xml.matches("<inline-boundary kind=\"end\"").count(), 2);
}

#[test]
fn prepared_controls_reject_unbalanced_or_misordered_actual_boundaries() {
    for kinds in [
        &["start"][..],
        &["end"][..],
        &["end", "start"][..],
        &["start", "start", "end"][..],
    ] {
        // Every leaf matches its captured role and source anchor. Only the
        // complete actual occurrence sequence violates transparent topology.
        let error = decode_fixture(&prepared_boundary_fixture(kinds))
            .expect_err("actual transparent boundaries must balance in occurrence order");
        assert_eq!(error.kind, MeasurementErrorKind::InvalidValue);
        assert_eq!(error.node_path, "root");
        assert_eq!(error.field_path, "preparedParagraph.controls");
    }
}

fn prepared_atomic_fixture() -> serde_json::Value {
    let mut fixture = prepared_control_fixture();
    fixture["preparedParagraph"]["controls"] = json!([]);
    fixture["preparedParagraph"]["atomics"] = json!([{
        "child":0,"at":0,"kind":"in-flow","width":10.0,"height":12.0,
        "precedingWrap":"word","followingWrap":"word","ancestorWrap":"word"
    }]);
    fixture["preparedParagraph"]["order"][0]["kind"] = json!("atomic");
    fixture["children"][0] = json!({"layoutInput":"box","tagName":"span",
        "style":{"display":"inline-block","size":{
            "width":{"unit":"px","value":10},"height":{"unit":"px","value":12}}},
        "unroundedLayout":{"x":0,"y":0,"width":10,"height":12},"children":[]});
    fixture
}

#[test]
fn prepared_atomic_capture_accepts_actual_box_role() {
    let measurement = supported(&prepared_atomic_fixture());
    let xml = crate::xml::generate_xml("actual-atomic", &measurement);
    assert!(xml.contains("source-tag=\"span\" display=\"inline-block\""));
    assert!(xml.contains("width=\"10px\" height=\"12px\""));
}

#[test]
fn prepared_atomic_break_sequence_retains_source_order_in_each_writing_mode() {
    for writing_mode in ["horizontal-tb", "vertical-lr", "vertical-rl"] {
        for direction in ["ltr", "rtl"] {
            let mut fixture = prepared_atomic_fixture();
            fixture["style"]["writingMode"] = json!(writing_mode);
            fixture["style"]["direction"] = json!(direction);
            let paragraph = &mut fixture["preparedParagraph"];
            paragraph["writingMode"] = json!(writing_mode);
            paragraph["direction"] = json!(direction);
            paragraph["origins"] = json!([]);
            paragraph["controls"] = json!([{"child":1,"at":0,"kind":"forced-break"}]);
            let mut following = paragraph["atomics"][0].clone();
            following["child"] = json!(2);
            paragraph["atomics"].as_array_mut().unwrap().push(following);
            paragraph["order"] = json!([
                {"kind":"atomic","index":0,"affinity":"before"},
                {"kind":"control","index":0,"affinity":"before"},
                {"kind":"atomic","index":1,"affinity":"before"}
            ]);
            let atomic = fixture["children"][0].clone();
            fixture["children"] =
                json!([atomic, prepared_control_fixture()["children"][0], atomic]);
            let measurement = supported(&fixture);
            let captured: serde_json::Value =
                serde_json::from_str(measurement.root.prepared_paragraph().unwrap()).unwrap();
            assert_eq!(captured, fixture["preparedParagraph"]);
            assert!(captured["origins"].as_array().unwrap().is_empty());
            assert_eq!(measurement.root.children().len(), 3);
        }
    }
}

#[test]
fn prepared_capture_rejects_a_different_actual_container_frame() {
    for (field, different) in [("writingMode", "vertical-lr"), ("direction", "rtl")] {
        let mut fixture = prepared_atomic_fixture();
        fixture["style"][field] = json!(different);
        assert!(
            decode_fixture(&fixture).is_err(),
            "a captured paragraph must not borrow a different container {field}"
        );
    }
}

#[test]
fn vertical_prepared_capture_preserves_only_collapsing_formatting_source() {
    for mode in ["vertical-lr", "vertical-rl"] {
        let mut fixture = prepared_fixture();
        fixture["style"]["writingMode"] = json!(mode);
        fixture["preparedParagraph"]["writingMode"] = json!(mode);
        fixture["preparedParagraph"]["baseStyle"]["whitespace"] = json!("collapse");
        fixture["preparedParagraph"]["origins"][0]["style"]["whitespace"] = json!("collapse");
        fixture["preparedParagraph"]["origins"][0]["units"] = json!([10, 32, 32, 13]);
        fixture["preparedParagraph"]["origins"][0]["end"] = json!(4);
        fixture["children"][0]["rangeFragments"] = json!([]);
        let measurement = supported(&fixture);
        let captured: serde_json::Value =
            serde_json::from_str(measurement.root.prepared_paragraph().unwrap()).unwrap();
        assert_eq!(captured["origins"][0]["units"], json!([10, 32, 32, 13]));
        let mut visible = fixture.clone();
        visible["preparedParagraph"]["origins"][0]["units"] = json!([88, 32, 32, 13]);
        assert!(decode_fixture(&visible).is_err());
        fixture["preparedParagraph"]["origins"][0]["style"]["whitespace"] = json!("preserve");
        assert!(decode_fixture(&fixture).is_err());
    }
}

fn br_band_fixture(rounding: bool, participation: bool) -> serde_json::Value {
    let mut fixture = box_fixture();
    fixture["useRounding"] = json!(rounding);
    let mut child = json!({
        "layoutInput":"box","tagName":"br","style":{"display":"inline"},
        "browserBrObservation":"font-band",
        "unroundedLayout":{"x":1.125,"y":5.25,"width":0.0,"height":20.125},
        "smartRoundedLayout":{"x":1.0,"y":5.0,"width":0.0,"height":20.0},
        "children":[]
    });
    if participation {
        child["lineControlParticipation"] = json!({"kind":"forced-break"});
    }
    fixture["children"] = json!([child]);
    if rounding {
        fixture["smartRoundedLayout"] = fixture["unroundedLayout"].clone();
    }
    fixture
}

#[test]
fn br_font_band_request_preserves_selected_rectangle_independently_of_input_role() {
    for rounding in [false, true] {
        for participation in [false, true] {
            let fixture = br_band_fixture(rounding, participation);
            let xml = crate::xml::generate_xml("measured-br-band", &supported(&fixture));
            let parsed = roxmltree::Document::parse(&xml).unwrap();
            let band = parsed
                .descendants()
                .find(|node| node.has_tag_name("browser-br-font-band"))
                .unwrap();
            let expected = if rounding {
                [("x", "1"), ("y", "5"), ("width", "0"), ("height", "20")]
            } else {
                [
                    ("x", "1.125"),
                    ("y", "5.25"),
                    ("width", "0"),
                    ("height", "20.125"),
                ]
            };
            for (name, value) in expected {
                assert_eq!(band.attribute(name), Some(value));
            }
            assert!(
                parsed
                    .descendants()
                    .any(|node| node.has_tag_name("browser-control"))
            );
            let mut relation_only = fixture;
            relation_only["children"][0]
                .as_object_mut()
                .unwrap()
                .remove("browserBrObservation");
            let xml = crate::xml::generate_xml("no-band-demand", &supported(&relation_only));
            let parsed = roxmltree::Document::parse(&xml).unwrap();
            assert!(
                !parsed
                    .descendants()
                    .any(|node| node.has_tag_name("browser-br-font-band"))
            );
        }
    }
}

#[test]
fn br_font_band_request_rejects_false_roles_unknown_modes_and_lost_geometry() {
    for (field, value) in [
        ("tagName", json!("span")),
        ("browserBrObservation", json!("ink")),
        ("browserBrObservation", json!(null)),
        ("textContent", json!("not-a-BR-leaf")),
        ("fragments", json!([])),
        ("rangeFragments", json!([])),
    ] {
        let mut fixture = br_band_fixture(false, false);
        fixture["children"][0][field] = value;
        assert!(decode_fixture(&fixture).is_err(), "invalid {field}");
    }
    for display in ["none", "block"] {
        let mut fixture = br_band_fixture(false, false);
        fixture["children"][0]["style"]["display"] = json!(display);
        assert!(decode_fixture(&fixture).is_err());
    }
    let mut fixture = br_band_fixture(false, false);
    fixture["children"][0]
        .as_object_mut()
        .unwrap()
        .remove("unroundedLayout");
    assert!(decode_fixture(&fixture).is_err());
    let root_br = br_band_fixture(false, false)["children"][0].clone();
    assert!(decode_fixture(&root_br).is_err());
}

#[test]
fn prepared_atomic_capture_rejects_actual_break_control_before_xml() {
    let mut fixture = prepared_atomic_fixture();
    // Wire Box is also the envelope role of a real BR. The captured atomic
    // must not borrow that independently declared forced-break control.
    fixture["children"][0] = prepared_control_fixture()["children"][0].clone();
    let error = decode_fixture(&fixture)
        .expect_err("a captured atomic must match an actual box rather than a break control");
    assert_eq!(error.kind, MeasurementErrorKind::InvalidValue);
    assert_eq!(error.node_path, "root");
    assert_eq!(error.field_path, "preparedParagraph.atomics");
}

#[test]
fn prepared_atomic_capture_rejects_inline_br_without_optional_control_marker() {
    let mut fixture = prepared_atomic_fixture();
    let mut br = prepared_control_fixture()["children"][0].clone();
    br.as_object_mut()
        .unwrap()
        .remove("lineControlParticipation");
    fixture["children"][0] = br;
    let error = decode_fixture(&fixture)
        .expect_err("an actual inline BR remains a break without the optional wire marker");
    assert_eq!(error.kind, MeasurementErrorKind::InvalidValue);
    assert_eq!(error.node_path, "root");
    assert_eq!(error.field_path, "preparedParagraph.atomics");
}

#[test]
fn prepared_atomic_capture_accepts_corresponding_in_flow_and_absolute_roles() {
    for (kind, display, position) in [
        ("in-flow", "inline-block", "relative"),
        ("in-flow", "inline-grid", "relative"),
        ("in-flow", "inline-grid-lanes", "relative"),
        ("out-of-flow", "inline-block", "absolute"),
    ] {
        let mut fixture = prepared_atomic_fixture();
        fixture["preparedParagraph"]["atomics"][0]["kind"] = json!(kind);
        fixture["children"][0]["style"]["display"] = json!(display);
        fixture["children"][0]["style"]["position"] = json!(position);
        fixture["children"][0]["style"]["cssFloat"] = json!("none");
        let measurement = supported(&fixture);
        assert!(measurement.root.prepared_paragraph().is_some());
    }
}

#[test]
fn prepared_atomic_capture_rejects_mismatched_display_position_and_float_roles() {
    for (kind, display, position, float) in [
        ("in-flow", "block", "relative", "none"),
        ("in-flow", "inline-block", "absolute", "none"),
        ("in-flow", "inline-block", "relative", "left"),
        ("in-flow", "none", "relative", "none"),
        ("out-of-flow", "inline-block", "relative", "none"),
        ("out-of-flow", "none", "absolute", "none"),
    ] {
        let mut fixture = prepared_atomic_fixture();
        fixture["preparedParagraph"]["atomics"][0]["kind"] = json!(kind);
        fixture["children"][0]["style"]["display"] = json!(display);
        fixture["children"][0]["style"]["position"] = json!(position);
        fixture["children"][0]["style"]["cssFloat"] = json!(float);
        let error = decode_fixture(&fixture)
            .expect_err("captured atomic flow role must match the actual Layout box inputs");
        assert_eq!(error.kind, MeasurementErrorKind::InvalidValue);
        assert_eq!(error.node_path, "root");
        assert_eq!(error.field_path, "preparedParagraph.atomics");
    }
}

#[test]
fn prepared_source_and_range_rectangles_remain_distinct_closed_channels() {
    let fixture = prepared_fixture();
    let measurement = supported(&fixture);
    let xml = crate::xml::generate_xml("actual-source", &measurement);
    assert!(xml.contains("measurement-version=\"2\""));
    assert!(xml.contains("<prepared-paragraph capture="));
    assert!(xml.contains("<range-fragment origin=\"0\""));
    assert!(xml.contains("advance=\"999\""));
    assert!(!xml.contains("inline-extent="));
    // Observation changes cannot enter the serialized source capture.
    let mut changed = fixture.clone();
    changed["children"][0]["rangeFragments"][0]["advance"] = json!(1.0);
    assert_eq!(
        supported(&changed).root.prepared_paragraph(),
        measurement.root.prepared_paragraph()
    );
}

#[test]
fn prepared_source_rejects_uncovered_units_false_font_and_observation_as_input() {
    for fixture in [
        {
            let mut v = prepared_fixture();
            v["preparedParagraph"]["origins"][0]["units"] = json!([0x05d0]);
            v
        },
        {
            let mut v = prepared_fixture();
            v["preparedParagraph"]["font"]["loaded"] = json!(false);
            v
        },
        {
            let mut v = prepared_fixture();
            v["preparedParagraph"]["origins"][0]["width"] = json!(10);
            v
        },
        {
            let mut v = prepared_fixture();
            v["children"][0]["preparedOccurrence"] = json!(1);
            v
        },
        {
            let mut v = prepared_fixture();
            v["preparedParagraph"]["order"][0]["index"] = json!(1);
            v
        },
    ] {
        assert!(decode_fixture(&fixture).is_err());
    }
}
