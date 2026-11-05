use super::*;
use crate::measurement::{
    self, BoxObservationRequest, DecodedMeasurement, NodeKind, ValidatedMeasurement,
};
use serde_json::{Value, json};

fn leaf() -> Value {
    let geometry = json!({"x":0,"y":0,"width":20,"height":30,
        "clientWidth":20,"clientHeight":30,"scrollWidth":25,"scrollHeight":41});
    json!({"layoutInput":"box","tagName":"div",
        "style":{"display":"block","writingMode":"horizontal-tb","direction":"ltr",
            "overflowX":"visible","overflowY":"visible","scrollbarGutter":"auto","scrollbarWidth":15},
        "unroundedLayout":geometry,"smartRoundedLayout":geometry,
        "worldRoundedLayout":geometry,"naivelyRoundedLayout":geometry,"children":[]})
}

fn fixture() -> Value {
    let mut root = leaf();
    root["useRounding"] = json!(false);
    root["gridAlignmentProfile"] = json!({"profile":1,"owners":[]});
    root["style"]["position"] = json!("absolute");
    root["viewport"] = json!({"rootContext":{"kind":"absolute-root-local",
        "containingWidth":800,"containingHeight":600,"containingDirection":"ltr",
        "hypotheticalDirection":"ltr","hypotheticalDisplay":"block",
        "hypotheticalAnchor":"inline-start","hypotheticalInlineOffset":0,
        "fallback":{"initialContainingBlock":{"width":800,"height":600},
            "nearestScrollport":"none","containingLimits":{"width":"unbounded","height":"unbounded"}}}});
    root
}

fn decode(
    value: &Value,
    request: BoxObservationRequest,
) -> Result<DecodedMeasurement, MeasurementError> {
    measurement::decode(
        &value.to_string(),
        "element-sizes",
        "border_box_ltr",
        request,
    )
}

fn supported(value: &Value, request: BoxObservationRequest) -> ValidatedMeasurement {
    match decode(value, request).expect("valid selected element-size observation") {
        DecodedMeasurement::Supported(value) => value,
        DecodedMeasurement::Unsupported { reason } => {
            panic!("unexpected unsupported profile: {reason}")
        }
    }
}

fn row_sizes(measurement: &ValidatedMeasurement) -> Vec<(String, [u32; 4])> {
    measurement
        .element_scroll_sizes()
        .unwrap()
        .iter()
        .map(|row| {
            (
                row.node_path().into(),
                [
                    row.client_width(),
                    row.client_height(),
                    row.scroll_width(),
                    row.scroll_height(),
                ],
            )
        })
        .collect()
}

#[test]
fn observation_request_owns_canonical_input_bytes() {
    assert_eq!(BoxObservationRequest::Geometry.input_bytes(), None);
    assert_eq!(
        BoxObservationRequest::from_input_bytes(None),
        Ok(BoxObservationRequest::Geometry)
    );
    assert_eq!(
        BoxObservationRequest::from_input_bytes(
            BoxObservationRequest::ElementScrollSizes.input_bytes()
        ),
        Ok(BoxObservationRequest::ElementScrollSizes)
    );
    for invalid in [
        b"".as_slice(),
        b"{}",
        b"{\"schema_version\":1,\"element_scroll_sizes\":\"horizontal-zero-edge-divs\"}",
        b"{\"schema_version\":1,\"element_scroll_sizes\":\"other\"}\n",
        &[0xff],
    ] {
        assert!(BoxObservationRequest::from_input_bytes(Some(invalid)).is_err());
    }
    assert_eq!(
        BoxObservationRequest::ElementScrollSizes.input_bytes(),
        Some(
            b"{\"schema_version\":1,\"element_scroll_sizes\":\"horizontal-zero-edge-divs\"}\n"
                .as_slice()
        )
    );
}

#[test]
fn selected_tuple_is_coupled_and_independent_of_diagnostics_and_naive_clients() {
    for rounded in [false, true] {
        let mut wire = fixture();
        wire["useRounding"] = json!(rounded);
        for (field, sizes) in [
            ("unroundedLayout", [20, 30, 25, 41]),
            ("smartRoundedLayout", [21, 31, 26, 42]),
            ("worldRoundedLayout", [90, 91, 92, 93]),
            ("naivelyRoundedLayout", [80, 81, 82, 83]),
        ] {
            for (name, size) in ["clientWidth", "clientHeight", "scrollWidth", "scrollHeight"]
                .into_iter()
                .zip(sizes)
            {
                wire[field][name] = json!(size);
            }
        }
        let result = supported(&wire, BoxObservationRequest::ElementScrollSizes);
        assert_eq!(
            row_sizes(&result),
            vec![(
                "0".into(),
                if rounded {
                    [21, 31, 26, 42]
                } else {
                    [20, 30, 25, 41]
                }
            )]
        );
    }
}

#[test]
fn selected_tuple_never_borrows_a_missing_component_from_another_geometry() {
    for rounded in [false, true] {
        for field in ["clientWidth", "clientHeight", "scrollWidth", "scrollHeight"] {
            let mut wire = fixture();
            wire["useRounding"] = json!(rounded);
            let selected = if rounded {
                "smartRoundedLayout"
            } else {
                "unroundedLayout"
            };
            wire[selected].as_object_mut().unwrap().remove(field);
            let error = decode(&wire, BoxObservationRequest::ElementScrollSizes).unwrap_err();
            assert_eq!(
                error.kind,
                MeasurementErrorKind::MissingField,
                "{rounded}/{field}"
            );
            assert_eq!(error.field_path, format!("{selected}.{field}"));
            assert_eq!(error.node_path, "root");
        }
    }
}

#[test]
fn integer_domain_is_lossless_above_f32_precision_and_at_cssom_long_bound() {
    for sizes in [
        [0, 0, 0, 0],
        [16_777_216, 16_777_217, 16_777_218, 16_777_219],
        [i32::MAX as u32; 4],
    ] {
        let mut wire = fixture();
        for (field, value) in ["clientWidth", "clientHeight", "scrollWidth", "scrollHeight"]
            .into_iter()
            .zip(sizes)
        {
            wire["unroundedLayout"][field] = json!(value);
        }
        let result = supported(&wire, BoxObservationRequest::ElementScrollSizes);
        assert_eq!(row_sizes(&result), vec![("0".into(), sizes)]);
        let xml = crate::xml::generate_xml("integer-domain", &result);
        let document = roxmltree::Document::parse(&xml).unwrap();
        let table = document
            .root_element()
            .children()
            .find(|node| node.has_tag_name("element-scroll-sizes"))
            .unwrap();
        let row = table.first_element_child().unwrap();
        for (name, value) in [
            "client-width",
            "client-height",
            "scroll-width",
            "scroll-height",
        ]
        .into_iter()
        .zip(sizes)
        {
            assert_eq!(
                row.attribute(name).unwrap().parse::<u32>().unwrap(),
                value,
                "{name}"
            );
        }
    }
}

#[test]
fn fractional_negative_and_out_of_long_sizes_refuse_with_selected_field_context() {
    for field in ["clientWidth", "clientHeight", "scrollWidth", "scrollHeight"] {
        for (value, kind) in [
            (-1.0, MeasurementErrorKind::InvalidValue),
            (0.5, MeasurementErrorKind::InvalidValue),
            (2_147_483_648.0, MeasurementErrorKind::OutOfRange),
        ] {
            let mut wire = fixture();
            wire["unroundedLayout"][field] = json!(value);
            let error = decode(&wire, BoxObservationRequest::ElementScrollSizes).unwrap_err();
            assert_eq!(error.kind, kind, "{field}/{value}");
            assert_eq!(error.field_path, format!("unroundedLayout.{field}"));
            assert_eq!(error.case_id, "element-sizes");
            assert_eq!(error.variant, "border_box_ltr");
        }
    }
    let context = Context {
        case_id: "nonfinite",
        variant: "ltr",
        node_path: "root".into(),
    };
    for value in [f64::NAN, f64::INFINITY, f64::NEG_INFINITY] {
        assert_eq!(
            dom_size(Some(value), &context, "unroundedLayout.clientWidth")
                .unwrap_err()
                .kind,
            MeasurementErrorKind::InvalidValue
        );
    }
}

#[test]
fn null_unknown_and_duplicate_tuple_members_are_not_ignored() {
    let mut wire = fixture();
    wire["unroundedLayout"]["clientWidth"] = Value::Null;
    assert_eq!(
        decode(&wire, BoxObservationRequest::ElementScrollSizes)
            .unwrap_err()
            .kind,
        MeasurementErrorKind::Decode
    );
    wire = fixture();
    wire["unroundedLayout"]["unexpectedSize"] = json!(1);
    assert_eq!(
        decode(&wire, BoxObservationRequest::ElementScrollSizes)
            .unwrap_err()
            .kind,
        MeasurementErrorKind::Decode
    );
    let raw = fixture().to_string().replace(
        "\"clientWidth\":20",
        "\"clientWidth\":20,\"clientWidth\":21",
    );
    assert_eq!(
        measurement::decode(
            &raw,
            "duplicate",
            "ltr",
            BoxObservationRequest::ElementScrollSizes
        )
        .unwrap_err()
        .kind,
        MeasurementErrorKind::Decode
    );
}

#[test]
fn client_larger_than_scroll_is_a_coupled_contradiction_on_either_axis() {
    for (client, scroll) in [
        ("clientWidth", "scrollWidth"),
        ("clientHeight", "scrollHeight"),
    ] {
        let mut wire = fixture();
        wire["unroundedLayout"][client] = json!(50);
        let error = decode(&wire, BoxObservationRequest::ElementScrollSizes).unwrap_err();
        assert_eq!(error.kind, MeasurementErrorKind::ContradictoryFields);
        assert_eq!(error.field_path, format!("unroundedLayout.{scroll}"));
    }
}

#[test]
fn complete_original_preorder_includes_opaque_div_text_and_keeps_child_paths() {
    let mut wire = fixture();
    let mut nested = leaf();
    let mut text = leaf();
    text["textContent"] = json!("HHHH");
    nested["children"] = json!([text]);
    wire["children"] = json!([leaf(), nested]);
    let result = supported(&wire, BoxObservationRequest::ElementScrollSizes);
    assert_eq!(
        row_sizes(&result)
            .iter()
            .map(|(path, _)| path.as_str())
            .collect::<Vec<_>>(),
        ["0", "0.0", "0.1", "0.1.0"]
    );
    assert!(matches!(
        result.root().children()[1].children()[0].kind(),
        NodeKind::Text
    ));
    let xml = crate::xml::generate_xml("source-preorder", &result);
    let document = roxmltree::Document::parse(&xml).unwrap();
    let table = document
        .root_element()
        .children()
        .find(|node| node.has_tag_name("element-scroll-sizes"))
        .unwrap();
    assert_eq!(
        table.attribute("profile"),
        Some("horizontal-zero-edge-divs")
    );
    let rows = table
        .children()
        .filter(roxmltree::Node::is_element)
        .collect::<Vec<_>>();
    assert_eq!(
        rows.iter()
            .map(|row| row.attribute("node-path").unwrap())
            .collect::<Vec<_>>(),
        ["0", "0.0", "0.1", "0.1.0"]
    );
    assert!(rows.iter().all(|row| row.tag_name().name() == "element"
        && row.attributes().len() == 5
        && row.first_element_child().is_none()));
}

#[test]
fn unsupported_source_child_is_not_filtered_or_renumbered_into_a_partial_census() {
    let mut wire = fixture();
    wire["children"] = json!([leaf(), {"layoutInput":"inline-boundary","inlineBoundary":{"kind":"start"},"children":[]},leaf()]);
    match decode(&wire, BoxObservationRequest::ElementScrollSizes).unwrap() {
        DecodedMeasurement::Unsupported { reason } => {
            assert!(reason.starts_with("root.children[1]:"), "{reason}")
        }
        DecodedMeasurement::Supported(_) => {
            panic!("unsupported original occurrence must refuse the complete profile")
        }
    }
}

#[test]
fn closed_source_profile_refuses_other_roles_axes_overflow_edges_and_reserved_gutters() {
    let mut cases = Vec::new();
    let mut missing_tag = leaf();
    missing_tag.as_object_mut().unwrap().remove("tagName");
    cases.push(missing_tag);
    for tag in ["span", "br", "img"] {
        let mut node = leaf();
        node["tagName"] = json!(tag);
        cases.push(node);
    }
    for display in [
        "inline",
        "none",
        "contents",
        "table",
        "table-row",
        "table-cell",
    ] {
        let mut node = leaf();
        node["style"]["display"] = json!(display);
        cases.push(node);
    }
    for mode in ["vertical-lr", "vertical-rl", "sideways-lr", "sideways-rl"] {
        let mut node = leaf();
        node["style"]["writingMode"] = json!(mode);
        cases.push(node);
    }
    for field in [
        "writingMode",
        "direction",
        "overflowX",
        "overflowY",
        "scrollbarGutter",
    ] {
        let mut node = leaf();
        node["style"].as_object_mut().unwrap().remove(field);
        cases.push(node);
    }
    for overflow in ["auto", "scroll"] {
        for axis in ["overflowX", "overflowY"] {
            let mut node = leaf();
            node["style"][axis] = json!(overflow);
            cases.push(node);
        }
    }
    for gutter in ["stable", "stable both-edges"] {
        let mut node = leaf();
        node["style"]["scrollbarGutter"] = json!(gutter);
        cases.push(node);
    }
    for edges in ["padding", "border"] {
        for edge in ["top", "right", "bottom", "left"] {
            let mut node = leaf();
            node["style"][edges] = json!({});
            node["style"][edges][edge] = json!({"unit":"px","value":1});
            cases.push(node);
        }
    }
    let mut anonymous = leaf();
    anonymous["layoutReadyAnonymousGridTextWrapper"] = json!(true);
    cases.push(anonymous);
    cases.push(json!({"layoutInput":"prepared-text","preparedOccurrence":0,"children":[]}));
    cases.push(json!({"layoutInput":"inline-text","children":[]}));
    for (index, node) in cases.into_iter().enumerate() {
        let mut wire = fixture();
        wire["children"] = json!([node]);
        assert!(
            matches!(
                decode(&wire, BoxObservationRequest::ElementScrollSizes).unwrap(),
                DecodedMeasurement::Unsupported { .. }
            ),
            "profile case {index}"
        );
    }
}

#[test]
fn absolute_root_local_authority_is_required_even_with_all_four_sizes() {
    let mut wire = fixture();
    wire["style"]["position"] = json!("relative");
    wire["viewport"] = json!({"rootContext":{"kind":"block-flow-auto","parentInlineExtent":800,
        "parentDirection":"ltr","fallback":{"initialContainingBlock":{"width":800,"height":600},
        "nearestScrollport":"none","containingLimits":{"width":"unbounded","height":"unbounded"}}}});
    assert!(matches!(
        decode(&wire, BoxObservationRequest::ElementScrollSizes).unwrap(),
        DecodedMeasurement::Unsupported { .. }
    ));
    assert!(decode(&wire, BoxObservationRequest::Geometry).is_ok());
}

#[test]
fn zero_edges_and_nonzero_host_scrollbar_thickness_do_not_fabricate_a_gutter() {
    for overflow in ["visible", "clip", "hidden"] {
        for direction in ["ltr", "rtl"] {
            let mut wire = fixture();
            wire["style"]["direction"] = json!(direction);
            wire["viewport"]["rootContext"]["hypotheticalDirection"] = json!(direction);
            wire["style"]["overflowX"] = json!(overflow);
            wire["style"]["overflowY"] = json!(overflow);
            for edges in ["border", "padding"] {
                wire["style"][edges] = json!({"top":{"unit":"px","value":0},"right":{"unit":"px","value":0},
                    "bottom":{"unit":"px","value":0},"left":{"unit":"px","value":0}});
            }
            assert_eq!(
                row_sizes(&supported(&wire, BoxObservationRequest::ElementScrollSizes)),
                vec![("0".into(), [20, 30, 25, 41])]
            );
        }
    }
}

#[test]
fn explicit_sizes_append_only_the_sideband_and_geometry_mode_keeps_legacy_tuple_policy() {
    let mut wire = fixture();
    wire["children"] = json!([leaf()]);
    let geometry = supported(&wire, BoxObservationRequest::Geometry);
    assert!(geometry.element_scroll_sizes().is_none());
    let historical = crate::xml::generate_xml("unchanged", &geometry);
    let observed = crate::xml::generate_xml(
        "unchanged",
        &supported(&wire, BoxObservationRequest::ElementScrollSizes),
    );
    let document = roxmltree::Document::parse(&observed).unwrap();
    let siblings = document
        .root_element()
        .children()
        .filter(roxmltree::Node::is_element)
        .collect::<Vec<_>>();
    assert_eq!(
        siblings
            .iter()
            .map(|node| node.tag_name().name())
            .collect::<Vec<_>>(),
        [
            "viewport",
            "input",
            "expectations",
            "element-scroll-sizes",
            "diagnostics"
        ]
    );
    let table = siblings[3];
    let sideband_line_start = observed[..table.range().start].rfind('\n').unwrap() + 1;
    let sideband_line_end =
        observed[table.range().end..].find('\n').unwrap() + table.range().end + 1;
    assert_eq!(
        format!(
            "{}{}",
            &observed[..sideband_line_start],
            &observed[sideband_line_end..]
        ),
        historical
    );
    let doc = roxmltree::Document::parse(&historical).unwrap();
    let expectations = doc
        .root_element()
        .children()
        .find(|node| node.has_tag_name("expectations"))
        .unwrap();
    assert_eq!(
        &historical[expectations.range()],
        "<expectations>\n    <node x=\"0\" y=\"0\" width=\"20\" height=\"30\">\n      <node x=\"0\" y=\"0\" width=\"20\" height=\"30\"/>\n    </node>\n  </expectations>"
    );
    wire["unroundedLayout"]["clientWidth"] = json!(0.25);
    assert!(decode(&wire, BoxObservationRequest::Geometry).is_ok());
    assert_eq!(
        decode(&wire, BoxObservationRequest::ElementScrollSizes)
            .unwrap_err()
            .kind,
        MeasurementErrorKind::InvalidValue
    );
}

#[test]
fn semantic_dom_clients_do_not_replace_the_legacy_naive_range_bridge() {
    let mut wire = fixture();
    wire["style"]["overflowX"] = json!("hidden");
    wire["style"]["overflowY"] = json!("hidden");
    wire["naivelyRoundedLayout"]["clientWidth"] = json!(5);
    wire["naivelyRoundedLayout"]["clientHeight"] = json!(6);
    let result = supported(&wire, BoxObservationRequest::ElementScrollSizes);
    assert_eq!(row_sizes(&result), vec![("0".into(), [20, 30, 25, 41])]);
    let xml = crate::xml::generate_xml("legacy-range", &result);
    let document = roxmltree::Document::parse(&xml).unwrap();
    let expected = document
        .root_element()
        .children()
        .find(|node| node.has_tag_name("expectations"))
        .unwrap()
        .first_element_child()
        .unwrap();
    assert_eq!(expected.attribute("scroll_width"), Some("20"));
    assert_eq!(expected.attribute("scroll_height"), Some("35"));
}

#[test]
fn source_display_profile_matches_the_readers_non_inline_box_front_door() {
    for display in ["block", "flex", "grid", "grid-lanes"] {
        let mut wire = fixture();
        let mut child = leaf();
        child["style"]["display"] = json!(display);
        wire["children"] = json!([child]);
        assert_eq!(
            row_sizes(&supported(&wire, BoxObservationRequest::ElementScrollSizes))
                .iter()
                .map(|(path, _)| path.as_str())
                .collect::<Vec<_>>(),
            ["0", "0.0"],
            "admitted display {display}",
        );
    }
    for display in [
        "inline-block",
        "inline-flex",
        "inline-grid",
        "inline-grid-lanes",
        "flow-root",
        "list-item",
    ] {
        let mut wire = fixture();
        let mut child = leaf();
        child["style"]["display"] = json!(display);
        wire["children"] = json!([child]);
        match decode(&wire, BoxObservationRequest::ElementScrollSizes).unwrap() {
            DecodedMeasurement::Unsupported { reason } => {
                assert!(
                    reason.starts_with("root.children[0]:"),
                    "{display}: {reason}"
                );
                assert!(
                    reason.contains("non-inline ordinary element box"),
                    "{display}: {reason}"
                );
            }
            DecodedMeasurement::Supported(_) => {
                panic!("display {display} exceeds the requested profile")
            }
        }
        // This closed observation profile does not narrow ordinary wire decoding.
        assert!(matches!(
            decode(&wire, BoxObservationRequest::Geometry).unwrap(),
            DecodedMeasurement::Supported(_)
        ));
    }
}

#[test]
fn inline_root_and_atomic_participation_cannot_enter_the_ordinary_source_census() {
    for (field, value) in [
        ("layoutReadyInlineRoot", json!(true)),
        (
            "atomicInlineParticipation",
            json!({"bidiLevel":0,"followingBreak":"prohibited"}),
        ),
    ] {
        let mut wire = fixture();
        let mut child = leaf();
        child[field] = value;
        wire["children"] = json!([child]);
        match decode(&wire, BoxObservationRequest::ElementScrollSizes).unwrap() {
            DecodedMeasurement::Unsupported { reason } => {
                assert!(reason.starts_with("root.children[0]:"), "{field}: {reason}");
                assert!(
                    reason.contains("synthetic, prepared and control"),
                    "{field}: {reason}"
                );
            }
            DecodedMeasurement::Supported(_) => {
                panic!("{field} exceeds the ordinary source census")
            }
        }
        assert!(matches!(
            decode(&wire, BoxObservationRequest::Geometry).unwrap(),
            DecodedMeasurement::Supported(_)
        ));
    }
}
