use crate::{measurement, parity_support, xml};
use serde_json::json;

fn compare_descendant_typography(parent_field: &str, parent_value: serde_json::Value) {
    let mut parent = json!({
        "display": "flex", "position": "absolute", "direction": "ltr",
        "alignItems": "start",
        "size": {"width": {"unit": "px", "value": 200}},
        "fontFamily": "ahem",
        "fontSize": {"unit": "px", "value": 10},
        "lineHeight": {"unit": "px", "value": 10}
    });
    parent[parent_field] = parent_value;
    let wire = json!({
        "layoutInput": "box", "tagName": "div", "useRounding": false,
        "gridAlignmentProfile": {"profile": 1, "owners": []},
        "viewport": {"rootContext": {
            "kind": "absolute-root-local", "containingWidth": 800, "containingHeight": 600,
            "containingDirection": "ltr", "hypotheticalDirection": "ltr",
            "hypotheticalDisplay": "flex", "hypotheticalAnchor": "inline-start",
            "hypotheticalInlineOffset": 0,
            "fallback": {
                "initialContainingBlock": {"width": 800, "height": 600},
                "nearestScrollport": "none",
                "containingLimits": {"width": "unbounded", "height": "unbounded"}
            }
        }},
        "style": parent,
        "unroundedLayout": {"x": 0, "y": 0, "width": 200, "height": 10},
        "worldRoundedLayout": {"x": 0, "y": 0, "width": 200, "height": 10},
        "children": [{
            "layoutInput": "box", "tagName": "div", "children": [],
            "style": {
                "display": "block", "direction": "ltr", "fontFamily": "ahem",
                "fontSize": {"unit": "px", "value": 10},
                "lineHeight": {"unit": "px", "value": 10}
            },
            "textContent": "HH",
            "unroundedLayout": {"x": 0, "y": 0, "width": 20, "height": 10},
            "worldRoundedLayout": {"x": 0, "y": 0, "width": 20, "height": 10}
        }]
    });
    let decoded = measurement::decode(
        &wire.to_string(),
        parent_field,
        "border_box_ltr",
        measurement::BoxObservationRequest::Geometry,
    )
    .expect("the source measurement and root context are supported");
    let measurement::DecodedMeasurement::Supported(measurement) = decoded else {
        panic!("the authored plain Flex input requires a supported measurement");
    };
    let generated = xml::generate_xml(parent_field, &measurement);
    let golden = parity_support::Golden::parse(&generated).expect("generated XML is readable");
    // Ahem at 10px gives each H a 10px advance and a 10px line.
    parity_support::assert_surgeist_matches(&golden)
        .unwrap_or_else(|error| panic!("{parent_field}: {error}\n{generated}"));
}

#[test]
fn captured_default_font_size_overrides_larger_parent_size() {
    compare_descendant_typography("fontSize", json!({"unit": "px", "value": 16}));
}

#[test]
fn captured_default_line_height_overrides_larger_parent_line_height() {
    compare_descendant_typography("lineHeight", json!({"unit": "px", "value": 24}));
}

#[test]
fn captured_default_font_family_overrides_parent_monospace() {
    compare_descendant_typography("fontFamily", json!("monospace"));
}
