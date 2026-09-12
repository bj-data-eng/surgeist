#![forbid(unsafe_code)]

//! Calculation-only compatibility retirement keeps current authored parsing and
//! literal I01 projections. It removes the duplicate frozen math grammar.

use surgeist_css::{CssKnownPropertyValueRef, parse_style_attribute};

fn assert_current_shape_without_old_projection(shape: &str) {
    let report = parse_style_attribute(&format!("clip-path: {shape}"));
    assert!(report.is_clean(), "{shape}: {:?}", report.diagnostics());
    let CssKnownPropertyValueRef::ClipPath(value) = report.syntax()[0]
        .known()
        .unwrap()
        .property_value()
        .unwrap()
    else {
        panic!("clip-path retains its property identity")
    };
    assert!(value.current().is_some(), "{shape}: current authored shape");
    assert!(
        value.i01_subset().is_none(),
        "{shape}: retired math projection"
    );
}

#[test]
fn calculated_circle_radius_keeps_current_shape_without_old_projection() {
    assert_current_shape_without_old_projection("circle(calc(1px + 2px))");
}

#[test]
fn calculated_shape_position_keeps_current_shape_without_old_projection() {
    assert_current_shape_without_old_projection("circle(1px at calc(1px + 2px) center)");
}

#[test]
fn calculated_ellipse_keeps_current_shape_without_old_projection() {
    assert_current_shape_without_old_projection("ellipse(calc(1px + 2px) 4px)");
}

#[test]
fn calculated_inset_keeps_current_shape_without_old_projection() {
    assert_current_shape_without_old_projection("inset(calc(1px + 2px))");
}

#[test]
fn calculated_polygon_keeps_current_shape_without_old_projection() {
    assert_current_shape_without_old_projection("polygon(calc(1px + 2px) 0px, 1px 1px)");
}

#[test]
fn calculated_blur_keeps_current_filter_without_old_projection() {
    let report = parse_style_attribute("filter: blur(calc(1px + 2px))");
    assert!(report.is_clean(), "{:?}", report.diagnostics());
    let CssKnownPropertyValueRef::Filter(value) = report.syntax()[0]
        .known()
        .unwrap()
        .property_value()
        .unwrap()
    else {
        panic!("filter retains its property identity")
    };
    assert!(value.i01_subset().is_none());
    assert!(matches!(
        value.current(),
        surgeist_css::CssFilterValue::Functions(_)
    ));
}

#[test]
fn literal_shapes_and_frozen_percentage_circle_keep_their_projections() {
    for (shape, has_current) in [
        ("circle(1px)", true),
        ("circle(1px at center)", true),
        ("ellipse(1px 2px)", true),
        ("inset(1px round 2px)", true),
        ("polygon(0px 0px, 1px 1px)", true),
        ("circle(50% at center)", false),
    ] {
        let report = parse_style_attribute(&format!("clip-path: {shape}"));
        assert!(report.is_clean(), "{shape}: {:?}", report.diagnostics());
        let CssKnownPropertyValueRef::ClipPath(value) = report.syntax()[0]
            .known()
            .unwrap()
            .property_value()
            .unwrap()
        else {
            panic!("clip-path retains its property identity")
        };
        assert!(value.i01_subset().is_some(), "{shape}: literal projection");
        assert_eq!(value.current().is_some(), has_current, "{shape}");
    }
}

#[test]
fn literal_blur_keeps_current_filter_and_old_projection() {
    let report = parse_style_attribute("filter: blur(3px)");
    assert!(report.is_clean(), "{:?}", report.diagnostics());
    let CssKnownPropertyValueRef::Filter(value) = report.syntax()[0]
        .known()
        .unwrap()
        .property_value()
        .unwrap()
    else {
        panic!("filter retains its property identity")
    };
    assert!(value.i01_subset().is_some());
}
