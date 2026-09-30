#![forbid(unsafe_code)]

//! Selected basic shapes retain symbolic calculations in the sole authored graph.

use surgeist_css::*;

fn parsed_shape(shape: &str) -> CssBasicShape {
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
    let CssClipPath::BasicShape(shape) = value.value() else {
        panic!("typed basic shape: {shape}");
    };
    shape.clone()
}

#[test]
fn calculated_circle_radius_keeps_typed_shape() {
    let CssBasicShape::Circle(circle) = parsed_shape("circle(calc(1px + 2px))") else {
        panic!("circle");
    };
    assert!(
        matches!(circle.radius(), surgeist_css::CssCircleRadius::LengthPercentage(radius)
        if radius.calculation().is_some())
    );
}

#[test]
fn calculated_shape_position_keeps_typed_shape() {
    let CssBasicShape::Circle(circle) = parsed_shape("circle(1px at calc(1px + 2px) center)")
    else {
        panic!("circle");
    };
    assert!(circle.position().is_some());
}

#[test]
fn calculated_ellipse_keeps_typed_shape() {
    let CssBasicShape::Ellipse(ellipse) = parsed_shape("ellipse(calc(1px + 2px) 4px)") else {
        panic!("ellipse");
    };
    assert!(
        matches!(ellipse.radius(), surgeist_css::CssEllipseRadius::Radii(radii)
        if radii.horizontal().calculation().is_some())
    );
}

#[test]
fn calculated_inset_keeps_typed_shape() {
    let CssBasicShape::Inset(inset) = parsed_shape("inset(calc(1px + 2px))") else {
        panic!("inset");
    };
    assert!(matches!(inset.offsets().values(), [value] if value.calculation().is_some()));
}

#[test]
fn calculated_polygon_keeps_typed_shape() {
    let CssBasicShape::Polygon(polygon) = parsed_shape("polygon(calc(1px + 2px) 0px, 1px 1px)")
    else {
        panic!("polygon");
    };
    assert!(polygon.points().points()[0].x().calculation().is_some());
}

#[test]
fn calculated_blur_keeps_typed_filter_expression() {
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
    let surgeist_css::CssFilter::Functions(functions) = value.value() else {
        panic!("filter list");
    };
    assert!(
        matches!(functions.functions(), [surgeist_css::CssFilterFunction::Blur(blur)]
        if blur.length().calculation().is_some())
    );
}

#[test]
fn literal_shapes_and_percentage_circle_use_typed_graph() {
    for (shape, expected_kind) in [
        ("circle(1px)", "circle"),
        ("circle(1px at center)", "circle"),
        ("ellipse(1px 2px)", "ellipse"),
        ("inset(1px round 2px)", "inset"),
        ("polygon(0px 0px, 1px 1px)", "polygon"),
        ("circle(50% at center)", "circle"),
    ] {
        let actual = parsed_shape(shape);
        assert_eq!(
            match actual {
                CssBasicShape::Circle(_) => "circle",
                CssBasicShape::Ellipse(_) => "ellipse",
                CssBasicShape::Inset(_) => "inset",
                CssBasicShape::Polygon(_) => "polygon",
                _ => panic!("unsupported shape"),
            },
            expected_kind
        );
    }
}

#[test]
fn literal_blur_keeps_typed_filter_length() {
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
    let surgeist_css::CssFilter::Functions(functions) = value.value() else {
        panic!("filter list");
    };
    assert!(
        matches!(functions.functions(), [surgeist_css::CssFilterFunction::Blur(blur)]
        if exact_dimension(blur.length().literal_component(), "3", "px"))
    );
}

fn exact_dimension(
    component: Option<&surgeist_css::CssComponentValue>,
    representation: &str,
    expected_unit: &str,
) -> bool {
    matches!(component.map(surgeist_css::CssComponentValue::view), Some(surgeist_css::CssComponentValueRef::Token(surgeist_css::CssValueTokenRef::Dimension { number, unit })) if number.representation() == representation && unit == expected_unit)
}
