#![forbid(unsafe_code)]
//! Shapes 1 CRD 2025-06-12 §3.3 specifies grammar order and punctuation;
//! computed defaults and geometry belong to separate phases.
//! Numeric expectations follow Values 4 WD 2024-03-12 §§5, 10.4, 10.5,
//! 10.9–10.13 and the documented binary64/source-order projection policy.
//! Ordinary coefficients use the shared six-place, ties-away CSSOM policy.
//! This is coverage of existing integration, without a claimed behavioral RED.

use surgeist_css::{
    CssSpecifiedValueSerializationErrorKind as K, CssSpecifiedValueSerializationLimits as L, *,
};

fn declaration(css: &str) -> CssDeclaration {
    let report = parse_style_attribute(&format!("clip-path:{css}!important"));
    assert!(report.is_clean(), "{css}: {:?}", report.diagnostics());
    report.syntax()[0].clone()
}

fn clip(declaration: &CssDeclaration) -> &CssClipPath {
    let CssKnownPropertyValueRef::ClipPath(value) =
        declaration.known().unwrap().property_value().unwrap()
    else {
        panic!("typed clip-path")
    };
    value.value()
}

fn basic(value: &CssClipPath) -> &CssBasicShape {
    let CssClipPath::BasicShape(value) = value else {
        panic!("basic shape")
    };
    value.shape()
}

fn assert_projection(authored: &str, expected: &str) {
    let source = declaration(authored);
    let before = source.clone();
    let components = source.value_components().clone();
    let origins: Vec<_> = components
        .items()
        .iter()
        .map(|v| v.origin().clone())
        .collect();
    assert_eq!(clip(&source).serialize_specified().unwrap(), expected);
    assert_eq!(
        basic(clip(&source)).serialize_specified().unwrap(),
        expected
    );
    let checked = parse_property_value(
        CssPropertyNameRef::Known(CssKnownProperty::ClipPath),
        components.clone(),
        CssImportance::Important,
    )
    .unwrap();
    assert_eq!(clip(&checked).serialize_specified().unwrap(), expected);
    assert_eq!(source, before);
    assert_eq!(source.value_components(), &components);
    for (component, origin) in source.value_components().items().iter().zip(origins) {
        assert_eq!(component.origin(), &origin);
    }
}

#[test]
fn all_eight_authored_functions_use_the_shared_numeric_or_retained_payload_contract() {
    for (authored, expected) in [
        ("circle(calc(1in + 1in))", "circle(calc(192px))"),
        (
            "ellipse(abs(-2em) hypot(3em,4em))",
            "ellipse(calc(2em) calc(5em))",
        ),
        (
            "inset(calc(1e16px - 1e16px + 1px) round calc(1px / 3))",
            "inset(calc(1px) round calc(0.333333px))",
        ),
        (
            "polygon(round calc(1in / 2), calc(1px + 2%) calc(3px * 2))",
            "polygon(round calc(48px), calc(2% + 1px) calc(6px))",
        ),
        (
            "rect(calc(1in) auto calc(2px + 3px) 0 round hypot(3em,4em))",
            "rect(calc(96px) auto calc(5px) 0 round calc(5em))",
        ),
        (
            "xywh(calc(1px + 2%) 0 calc(1px - 2px) abs(-2em))",
            "xywh(calc(2% + 1px) 0 calc(-1px) calc(2em))",
        ),
        (
            "shape(from abs(-2em) 0px, arc by calc(1in) 2% of hypot(3em,4em) rotate calc(0.25turn + 100grad))",
            "shape(from calc(2em) 0px, arc by calc(96px) 2% of calc(5em) rotate calc(180deg))",
        ),
        (
            "path(evenodd,'M01.00,0 L1e2 -0 Z')",
            "path(evenodd, \"M01.00,0 L1e2 -0 Z\")",
        ),
    ] {
        assert_projection(authored, expected);
    }
}

#[test]
fn exact_halfway_literals_round_through_radii_positions_and_arc_rotation() {
    for (authored, expected) in [
        (
            "circle(0.0000005em at -0.0000005px 1.2345675%)",
            "circle(0.000001em at -0.000001px 1.234568%)",
        ),
        (
            "ellipse(1.2345675px 0.0000005% at 0.000000499999px -0.000000499999px)",
            "ellipse(1.234568px 0.000001% at 0px 0px)",
        ),
        (
            "inset(-0.0000005px round 0.0000005em / 1.2345675%)",
            "inset(-0.000001px round 0.000001em / 1.234568%)",
        ),
        (
            "polygon(round -0.0000005px, 0.0000005% -1.2345675px)",
            "polygon(round -0.000001px, 0.000001% -1.234568px)",
        ),
        (
            "rect(-0.0000005px auto 1.2345675% 0)",
            "rect(-0.000001px auto 1.234568% 0)",
        ),
        (
            "xywh(-0.0000005px 0.0000005% 1.2345675em 0)",
            "xywh(-0.000001px 0.000001% 1.234568em 0)",
        ),
        (
            "shape(from 0px 0px, arc by 1px 2% of 3px rotate -0.0000005turn)",
            "shape(from 0px 0px, arc by 1px 2% of 3px rotate -0.000001turn)",
        ),
    ] {
        assert_projection(authored, expected);
    }
}

#[test]
fn context_dependent_comparisons_and_glyph_metrics_remain_symbolic() {
    for (authored, expected) in [
        ("circle(min(1%,2%))", "circle(min(1%, 2%))"),
        (
            "ellipse(abs(-2ex) hypot(3em,4rem))",
            "ellipse(abs(-2ex) hypot(3em, 4rem))",
        ),
        (
            "inset(calc(sign(-2em) * 1px) min(1px,2%))",
            "inset(calc(1px * sign(-2em)) min(1px, 2%))",
        ),
    ] {
        assert_projection(authored, expected);
    }
}

#[test]
fn finite_angle_range_conversion_reaches_arc_rotation_without_geometry_resolution() {
    for expression in [
        "cos(1e400deg)",
        "cos(1e308turn)",
        "cos(1e308rad)",
        "cos(hypot(1.3e308deg,1.3e308deg))",
        "cos(round(up,1.5e308deg,1e308deg))",
        "cos(1e308deg * 2 * 0)",
    ] {
        assert_projection(
            &format!("shape(from 0px 0px, arc by 0px 0px of 1px rotate calc({expression} * 1deg))"),
            "shape(from 0px 0px, arc by 0px 0px of 1px rotate calc(1deg))",
        );
    }
    assert_projection(
        "shape(from 0px 0px, arc by 0px 0px of 1px rotate calc(1.8e308grad / 1e308deg * 1deg))",
        "shape(from 0px 0px, arc by 0px 0px of 1px rotate calc(1.62deg))",
    );
}

#[test]
fn specified_nonfinite_math_is_not_top_level_numeric_censorship_or_radius_clamping() {
    for expression in ["cos(infinity * 1deg)", "cos(1deg / 0)"] {
        assert_projection(
            &format!("shape(from 0px 0px, arc by 0px 0px of 1px rotate calc({expression} * 1deg))"),
            "shape(from 0px 0px, arc by 0px 0px of 1px rotate calc(NaN * 1deg))",
        );
    }
    assert_projection("circle(calc(1px / 0))", "circle(calc(infinity * 1px))");
    assert_projection("circle(calc(1px - 2px))", "circle(calc(-1px))");
    assert_projection(
        "xywh(0 0 calc(1px - 2px) calc(0px / 0))",
        "xywh(0 0 calc(-1px) calc(NaN * 1px))",
    );
}

fn budget(
    expected: &str,
    input: usize,
    projection: usize,
    serialize: impl Fn(L) -> Result<String, CssSpecifiedValueSerializationError>,
) {
    assert_eq!(
        serialize(L::new(input, projection, expected.len())).unwrap(),
        expected
    );
    for (limits, kind) in [
        (
            L::new(input - 1, projection, expected.len()),
            K::InputNodeLimit,
        ),
        (
            L::new(input, projection - 1, expected.len()),
            K::ProjectionNodeLimit,
        ),
        (L::new(input, projection, expected.len() - 1), K::ByteLimit),
    ] {
        assert_eq!(serialize(limits).unwrap_err().kind(), kind);
    }
    assert_eq!(serialize(L::default()).unwrap(), expected);
}

#[test]
fn sibling_numeric_functions_and_position_literals_keep_one_atomic_composed_budget() {
    let source =
        declaration("ellipse(abs(-2em) hypot(3em,4em) at -0.0000005px 0.0000005%) border-box");
    let before = source.clone();
    let expected = "ellipse(calc(2em) calc(5em) at -0.000001px 0.000001%)";
    // Ellipse and radii pair: two; abs and hypot: two plus three;
    // position aggregate and axes: three; two exact literal offsets: two.
    budget(expected, 12, 12, |limits| {
        basic(clip(&source)).serialize_specified_with_limits(limits)
    });
    // Composition and explicit reference box each add one cumulative node.
    budget(&format!("{expected} border-box"), 14, 14, |limits| {
        clip(&source).serialize_specified_with_limits(limits)
    });
    assert_eq!(source, before);
    assert_eq!(source.value_components(), before.value_components());
}

#[test]
fn finite_angle_projection_spends_only_existing_command_and_math_nodes() {
    let source =
        declaration("shape(from 0px 0px, arc by 0px 0px of 1px rotate calc(cos(1e400deg) * 1deg))");
    let before = source.clone();
    let expected = "shape(from 0px 0px, arc by 0px 0px of 1px rotate calc(1deg))";
    // Initial shape/from/position/axes/leaves/command list: eight. Arc,
    // endpoint, pair, two leaves, radii marker/leaf and rotation marker: eight.
    // Calc/Product/Cos/two leaves: five inputs, four projected nodes.
    budget(expected, 21, 20, |limits| {
        basic(clip(&source)).serialize_specified_with_limits(limits)
    });
    budget(expected, 22, 21, |limits| {
        clip(&source).serialize_specified_with_limits(limits)
    });
    assert_eq!(source, before);
    let CssBasicShape::Shape(shape) = basic(clip(&source)) else {
        panic!("shape function")
    };
    let CssShapeCommand::Arc(arc) = &shape.commands().commands()[0] else {
        panic!("arc")
    };
    let angle = arc.rotation().unwrap();
    assert!(matches!(angle.origin(), CssValueOrigin::Parsed(_)));
    assert_eq!(
        angle
            .calculation()
            .unwrap()
            .components()
            .serialize()
            .unwrap()
            .as_css(),
        "calc(cos(1e400deg) * 1deg)"
    );
}

#[test]
fn programmatic_shape_construction_preserves_exact_math_graph_and_origin() {
    let components = CssComponentValues::try_new(vec![
        CssComponentValue::try_function(
            "HYPOT",
            CssComponentValues::try_new(vec![
                CssComponentValue::try_dimension("+03.00", "EM").unwrap(),
                CssComponentValue::try_token(",").unwrap(),
                CssComponentValue::try_dimension("04.000", "em").unwrap(),
            ])
            .unwrap(),
        )
        .unwrap(),
    ])
    .unwrap();
    let calculation =
        CssLengthPercentageCalculation::try_from_components(components.clone()).unwrap();
    let ty = calculation.numeric_type();
    let radius =
        CssSpecifiedNonNegativeLengthPercentage::try_from_calculation(calculation).unwrap();
    let shape = CssCircleShape::new(CssCircleRadius::LengthPercentage(radius.clone()), None);
    let before = shape.clone();
    budget("circle(calc(5em))", 4, 4, |limits| {
        shape.serialize_specified_with_limits(limits)
    });
    assert_eq!(shape, before);
    assert_eq!(radius.origin(), &CssValueOrigin::Programmatic);
    assert_eq!(radius.calculation().unwrap().components(), &components);
    assert_eq!(radius.calculation().unwrap().numeric_type(), ty);
    let CssCircleRadius::LengthPercentage(retained) = shape.radius() else {
        panic!("numeric radius")
    };
    assert_eq!(retained.origin(), radius.origin());
    assert_eq!(retained.calculation().unwrap().components(), &components);
}
