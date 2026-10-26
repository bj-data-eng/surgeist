//! Selected basic shapes retain symbolic calculations in the sole authored graph.

use crate::css::support::value_assertions::exact_dimension;
use surgeist_css::*;

fn parsed_shape(shape: &str) -> CssBasicShape {
    let report = CssParser::default().parse_block_contents(format!("clip-path: {shape}"));
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
    shape.shape().clone()
}

/// The authored text a retained calculation keeps, without simplification.
macro_rules! calculation_text {
    ($value:expr) => {
        $value
            .calculation()
            .expect("retained symbolic calculation")
            .serialize_tokens(Default::default())
            .unwrap()
            .as_css()
            .to_owned()
    };
}

// Each calculation is retained as the exact authored expression, and each
// literal operand as its authored dimension token: Shapes 1 (CRD 2025-06-12)
// §3.1 circle(), ellipse(), inset() and polygon() take <length-percentage>
// operands, and Filter Effects 1 blur() a <length>, so a calc() stays in the
// typed operand it was written for (Filter Effects 1, WD 2018-12-18, §6.1).
// Keeping `calc(1px + 2px)` unsimplified is the crate's authored-retention
// representation contract for retained calculations; it is not the Values 4
// §10.13 specified serialization, which would simplify it to `calc(3px)`.

#[test]
fn calculated_circle_radius_keeps_typed_shape() {
    let CssBasicShape::Circle(circle) = parsed_shape("circle(calc(1px + 2px))") else {
        panic!("circle");
    };
    let surgeist_css::CssCircleRadius::LengthPercentage(radius) = circle.radius() else {
        panic!("length-percentage radius");
    };
    assert_eq!(calculation_text!(radius), "calc(1px + 2px)");
    assert!(circle.position().is_none());
}

#[test]
fn calculated_shape_position_keeps_typed_shape() {
    let CssBasicShape::Circle(circle) = parsed_shape("circle(1px at calc(1px + 2px) center)")
    else {
        panic!("circle");
    };
    let surgeist_css::CssCircleRadius::LengthPercentage(radius) = circle.radius() else {
        panic!("length-percentage radius");
    };
    assert!(exact_dimension(
        radius.literal().map(|literal| literal.component()),
        "1",
        "px"
    ));
    let CssPositionRef::Cartesian(position) = circle.position().expect("position").view() else {
        panic!("Cartesian position");
    };
    let CssHorizontalPosition::Offset(x) = position.horizontal() else {
        panic!("horizontal offset: {:?}", position.horizontal());
    };
    assert_eq!(calculation_text!(x), "calc(1px + 2px)");
    assert!(matches!(position.vertical(), CssVerticalPosition::Center));
}

#[test]
fn calculated_ellipse_keeps_typed_shape() {
    let CssBasicShape::Ellipse(ellipse) = parsed_shape("ellipse(calc(1px + 2px) 4px)") else {
        panic!("ellipse");
    };
    let radii = ellipse.radii().expect("explicit radii");
    let CssEllipseRadius::LengthPercentage(horizontal) = radii.horizontal() else {
        panic!("horizontal length-percentage radius");
    };
    assert_eq!(calculation_text!(horizontal), "calc(1px + 2px)");
    let CssEllipseRadius::LengthPercentage(vertical) = radii.vertical() else {
        panic!("vertical length-percentage radius");
    };
    assert!(exact_dimension(
        vertical.literal().map(|literal| literal.component()),
        "4",
        "px"
    ));
    assert!(ellipse.position().is_none());
}

#[test]
fn calculated_inset_keeps_typed_shape() {
    let CssBasicShape::Inset(inset) = parsed_shape("inset(calc(1px + 2px))") else {
        panic!("inset");
    };
    let [offset] = inset.offsets().values() else {
        panic!("one authored inset offset");
    };
    assert_eq!(calculation_text!(offset), "calc(1px + 2px)");
    assert!(inset.round().is_none());
}

#[test]
fn calculated_polygon_keeps_typed_shape() {
    let CssBasicShape::Polygon(polygon) = parsed_shape("polygon(calc(1px + 2px) 0px, 1px 1px)")
    else {
        panic!("polygon");
    };
    let [first, second] = polygon.points().points() else {
        panic!("two authored polygon points");
    };
    assert_eq!(calculation_text!(first.x()), "calc(1px + 2px)");
    assert!(exact_dimension(
        first.y().literal().map(|literal| literal.component()),
        "0",
        "px"
    ));
    assert!(exact_dimension(
        second.x().literal().map(|literal| literal.component()),
        "1",
        "px"
    ));
    assert!(exact_dimension(
        second.y().literal().map(|literal| literal.component()),
        "1",
        "px"
    ));
}

#[test]
fn calculated_blur_keeps_typed_filter_expression() {
    let report = CssParser::default().parse_block_contents("filter: blur(calc(1px + 2px))");
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
    let [surgeist_css::CssFilterFunction::Blur(blur)] = functions.functions() else {
        panic!("one blur function");
    };
    assert_eq!(calculation_text!(blur.length()), "calc(1px + 2px)");
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
    let report = CssParser::default().parse_block_contents("filter: blur(3px)");
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
        if exact_dimension(blur.length().literal().map(|literal| literal.component()), "3", "px"))
    );
}
