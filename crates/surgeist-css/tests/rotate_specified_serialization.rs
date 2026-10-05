#![forbid(unsafe_code)]
//! New rotate output API: Transforms2 §5/§5.1/§12.2, Values3 angles, and the
//! existing six-place numeric contract. No preimplementation callable output API
//! existed; these are functional tests alongside the new API, not regression RED.
use surgeist_css::*;

type Limits = CssSpecifiedValueSerializationLimits;
type Kind = CssSpecifiedValueSerializationErrorKind;

fn parsed(text: &str) -> CssRotate {
    let source = format!("rotate:{text}");
    let report = parse_style_attribute(&source);
    assert!(report.is_clean(), "{text}: {:?}", report.diagnostics());
    let CssKnownPropertyValueRef::Rotate(value) = report.syntax()[0]
        .known()
        .unwrap()
        .property_value()
        .unwrap()
    else {
        panic!("rotate value");
    };
    value.value().clone()
}
fn number(text: &str) -> CssSpecifiedNumber {
    CssSpecifiedNumber::try_from_component(CssComponentValue::try_number(text).unwrap()).unwrap()
}
fn ordinary(angle: &str, unit: CssAngleUnit, axis: Option<CssRotateAxis>) -> CssRotate {
    CssRotate::Value(CssRotateValues::new(
        CssAngleValue::from_literal(CssAngleLiteral::try_new(angle, unit).unwrap()),
        axis,
    ))
}
fn vector(angle: &str, axis: [&str; 3]) -> CssRotate {
    ordinary(
        angle,
        CssAngleUnit::Degrees,
        Some(CssRotateAxis::Vector(axis.map(number))),
    )
}
fn output(input: &str, expected: &str) {
    let value = parsed(input);
    assert_eq!(value.serialize_specified().unwrap(), expected, "{input}");
    // Returned CSS is accepted through the actual typed grammar again.
    parsed(expected);
}

#[test]
fn none_is_preserved_and_proved_identity_uses_zero_degrees() {
    for (input, expected) in [
        ("none", "none"),
        ("0deg", "0deg"),
        ("x -0turn", "0deg"),
        ("0 0 0 45deg", "0deg"),
        ("0 -0 0 calc(45deg + 15deg)", "0deg"),
        ("1 2 3 360deg", "0deg"),
        ("x -720deg", "0deg"),
        ("y 400grad", "0deg"),
        ("z -2turn", "0deg"),
        ("x calc(1turn + 360deg)", "0deg"),
    ] {
        output(input, expected);
    }
    output("0 0 0 calc(infinity * 1deg)", "0deg");
}

#[test]
fn directed_parallel_vectors_reduce_and_reverse_the_angle_for_negative_axes() {
    for (input, expected) in [
        ("2 0 0 30deg", "x 30deg"),
        ("0 2 0 30deg", "y 30deg"),
        ("0 0 2 30deg", "30deg"),
        ("-2 0 0 30deg", "x -30deg"),
        ("0 -2 0 30deg", "y -30deg"),
        ("0 0 -2 30deg", "-30deg"),
        ("-2 0 0 -30deg", "x 30deg"),
        ("0 -2 0 .25turn", "y -0.25turn"),
        ("0 0 -2 100grad", "-100grad"),
        ("0 0 -2 1rad", "-1rad"),
        ("calc(-1 - 1) 0 0 calc(15deg + 15deg)", "x calc(-30deg)"),
    ] {
        output(input, expected);
    }
}

#[test]
fn axis_order_is_canonical_and_keyword_z_is_omitted() {
    for (input, expected) in [
        ("30deg x", "x 30deg"),
        ("30deg y", "y 30deg"),
        ("30deg z", "30deg"),
        ("z 30deg", "30deg"),
        ("30deg 1 2 3", "1 2 3 30deg"),
        ("calc(10deg + 20deg) 1 2 3", "1 2 3 calc(30deg)"),
    ] {
        output(input, expected);
    }
}

#[test]
fn exact_literal_axis_classification_survives_huge_and_subrounding_coefficients() {
    for (axis, expected) in [
        (["1e400", "0", "0"], "x 30deg"),
        (["-1e400", "0", "0"], "x -30deg"),
        (["0", "1e-400", "0"], "y 30deg"),
        (["0", "0", "-1e-400"], "-30deg"),
        (
            ["1e9999999999999999999999999999999999999999", "0", "0"],
            "x 30deg",
        ),
        (
            ["0", "0", "1e-9999999999999999999999999999999999999999"],
            "30deg",
        ),
    ] {
        let value = vector("30", axis);
        assert_eq!(
            value
                .serialize_specified_with_limits(Limits::new(6, 7, expected.len()))
                .unwrap(),
            expected
        );
    }
}

#[test]
fn exact_full_turn_detection_borrows_unbounded_decimal_exponents() {
    for (coefficient, unit) in [
        ("360.000", CssAngleUnit::Degrees),
        (".4e3", CssAngleUnit::Gradians),
        ("1.000", CssAngleUnit::Turns),
        (
            "9e9999999999999999999999999999999999999999",
            CssAngleUnit::Degrees,
        ),
        (
            "1e9999999999999999999999999999999999999999",
            CssAngleUnit::Gradians,
        ),
        (
            "1e9999999999999999999999999999999999999999",
            CssAngleUnit::Turns,
        ),
    ] {
        assert_eq!(
            ordinary(coefficient, unit, None)
                .serialize_specified_with_limits(Limits::new(2, 3, 4))
                .unwrap(),
            "0deg"
        );
    }
    assert_eq!(
        ordinary(
            "1e9999999999999999999999999999999999999999",
            CssAngleUnit::Degrees,
            None
        )
        .serialize_specified_with_limits(Limits::new(2, 3, 4))
        .unwrap_err()
        .kind(),
        Kind::ByteLimit
    );
    output("360.000001deg", "360.000001deg");
    output("400.000001grad", "400.000001grad");
    output("1.000001turn", "1.000001turn");
    output("6.283185rad", "6.283185rad");
    output("-1e-400deg", "0deg");
    output("-1 0 0 -1e-400deg", "x 0deg");
}

#[test]
fn general_vector_rejects_only_proved_nonzero_components_that_round_to_zero() {
    for axis in [
        [".00000001", "1", "0"],
        ["1", "-.00000001", "0"],
        ["1e-400", "2e-400", "0"],
    ] {
        let value = vector("30", axis);
        assert_eq!(
            value.serialize_specified().unwrap_err().kind(),
            Kind::UnrepresentableValue
        );
    }
    for input in ["calc(.00000001) 1 0 30deg", "1 calc(-.00000001) 0 30deg"] {
        assert_eq!(
            parsed(input).serialize_specified().unwrap_err().kind(),
            Kind::UnrepresentableValue
        );
    }
    assert_eq!(
        parsed("calc(1 / 2097152) 1 0 30deg")
            .serialize_specified()
            .unwrap_err()
            .kind(),
        Kind::UnrepresentableValue
    );
    output("calc(1 / 1048576) 1 0 30deg", "calc(0.000001) 1 0 30deg");
    // The surviving components are still rounded independently, not normalized.
    output("1.23456789 2.34567891 0 30deg", "1.234568 2.345679 0 30deg");
    output(".0000005 1 0 30deg", "0.000001 1 0 30deg");
    output("-.0000005 1 0 30deg", "-0.000001 1 0 30deg");
    assert_eq!(
        vector("30", ["1e400", "1", "0"])
            .serialize_specified_with_limits(Limits::new(6, 6, 20))
            .unwrap_err()
            .kind(),
        Kind::ByteLimit
    );
}

#[test]
fn symbolic_and_nonfinite_axis_math_is_retained_without_parallel_axis_guesses() {
    output("calc(1em / 1px) 0 0 30deg", "calc(1em / 1px) 0 0 30deg");
    output("calc(infinity) 0 0 30deg", "calc(infinity) 0 0 30deg");
    output("calc(NaN) 0 0 30deg", "calc(NaN) 0 0 30deg");
    output("calc(1e400) 0 0 30deg", "calc(infinity) 0 0 30deg");
    output("calc(1 + 1) 0 0 30deg", "x 30deg");
    // Math retains the existing binary64 arithmetic contract, unlike ordinary
    // exact coefficients: this nonzero authored decimal underflows in projection.
    output("calc(1e-400) 0 0 30deg", "0deg");
}

#[test]
fn symbolic_angle_negation_preserves_the_typed_calculation() {
    output(
        "0 0 -1 calc(1deg * 1em / 1px)",
        "calc(-1 * (1deg * 1em / 1px))",
    );
    output("0 0 -1 calc(infinity * 1deg)", "calc(-infinity * 1deg)");
    output("x calc(1deg * 1em / 1px)", "x calc(1deg * 1em / 1px)");
}

#[test]
fn limits_charge_authored_children_and_derived_identity_or_sign_work() {
    let positive = vector("45", ["2", "0", "0"]);
    assert_eq!(
        positive
            .serialize_specified_with_limits(Limits::new(6, 6, 7))
            .unwrap(),
        "x 45deg"
    );
    for (limits, kind) in [
        (Limits::new(5, 6, 7), Kind::InputNodeLimit),
        (Limits::new(6, 5, 7), Kind::ProjectionNodeLimit),
        (Limits::new(6, 6, 6), Kind::ByteLimit),
    ] {
        assert_eq!(
            positive
                .serialize_specified_with_limits(limits)
                .unwrap_err()
                .kind(),
            kind
        );
    }
    let negative = vector("45", ["-2", "0", "0"]);
    assert_eq!(
        negative
            .serialize_specified_with_limits(Limits::new(6, 7, 8))
            .unwrap(),
        "x -45deg"
    );
    assert_eq!(
        negative
            .serialize_specified_with_limits(Limits::new(6, 6, 8))
            .unwrap_err()
            .kind(),
        Kind::ProjectionNodeLimit
    );
    assert_eq!(
        vector("45", ["0", "0", "0"])
            .serialize_specified_with_limits(Limits::new(6, 7, 4))
            .unwrap(),
        "0deg"
    );
    assert_eq!(
        CssRotate::None
            .serialize_specified_with_limits(Limits::new(1, 1, 4))
            .unwrap(),
        "none"
    );
}

#[test]
fn omitted_huge_children_still_consume_work_without_axis_text_allocations() {
    assert_eq!(
        parsed("0 0 0 calc(15deg + 15deg)")
            .serialize_specified_with_limits(Limits::new(9, 9, 4))
            .unwrap(),
        "0deg"
    );
    for (limits, kind) in [
        (Limits::new(8, 9, 4), Kind::InputNodeLimit),
        (Limits::new(9, 8, 4), Kind::ProjectionNodeLimit),
    ] {
        assert_eq!(
            parsed("0 0 0 calc(15deg + 15deg)")
                .serialize_specified_with_limits(limits)
                .unwrap_err()
                .kind(),
            kind
        );
    }
    assert_eq!(
        vector("1e400", ["0", "0", "0"])
            .serialize_specified_with_limits(Limits::new(6, 7, 4))
            .unwrap(),
        "0deg"
    );
    assert_eq!(
        parsed("z 45deg")
            .serialize_specified_with_limits(Limits::new(3, 3, 5))
            .unwrap(),
        "45deg"
    );
    assert_eq!(
        parsed("z 45deg")
            .serialize_specified_with_limits(Limits::new(2, 3, 5))
            .unwrap_err()
            .kind(),
        Kind::InputNodeLimit
    );
}

#[test]
fn checked_number_and_angle_math_share_canonical_output_with_parser_admission() {
    let axis = CssSpecifiedNumber::try_from_calculation(
        CssNumberCalculation::try_from_components(parse_component_values("calc(-1 - 1)").unwrap())
            .unwrap(),
    )
    .unwrap();
    let angle = CssAngleValue::try_from_calculation(
        CssAngleCalculation::try_from_components(
            parse_component_values("calc(15deg + 15deg)").unwrap(),
        )
        .unwrap(),
    )
    .unwrap();
    let angle_origin = angle.origin().clone();
    let axis_origin = axis.origin().clone();
    let value = CssRotate::Value(CssRotateValues::new(
        angle,
        Some(CssRotateAxis::Vector([axis, number("0"), number("0")])),
    ));
    assert_eq!(value.serialize_specified().unwrap(), "x calc(-30deg)");
    assert_eq!(
        parsed("calc(-1 - 1) 0 0 calc(15deg + 15deg)")
            .serialize_specified()
            .unwrap(),
        "x calc(-30deg)"
    );
    let CssRotate::Value(values) = &value else {
        panic!("value");
    };
    assert_eq!(values.angle().origin(), &angle_origin);
    let Some(CssRotateAxis::Vector(axis)) = values.axis() else {
        panic!("vector");
    };
    assert_eq!(axis[0].origin(), &axis_origin);
}

#[test]
fn success_and_failure_preserve_non_ascii_source_origins_and_operands() {
    let source = "/*😀*/rotate:0 0 -2 calc(15deg + 15deg)";
    let report = parse_style_attribute(source);
    assert!(report.is_clean());
    let CssKnownPropertyValueRef::Rotate(value) = report.syntax()[0]
        .known()
        .unwrap()
        .property_value()
        .unwrap()
    else {
        panic!("rotate");
    };
    let value = value.value();
    let before = format!("{value:?}");
    let CssRotate::Value(values) = value else {
        panic!("value");
    };
    let origin = values.angle().origin().clone();
    let CssValueOrigin::Parsed(parsed_origin) = &origin else {
        panic!("parsed origin");
    };
    let opener_start = source.find("calc(").unwrap();
    // The retained function origin covers its opener, not the complete body.
    assert_eq!(
        parsed_origin.span().start().byte_offset().value(),
        opener_start
    );
    assert_eq!(
        parsed_origin.span().end().byte_offset().value(),
        opener_start + "calc(".len()
    );
    assert_eq!(value.serialize_specified().unwrap(), "calc(-30deg)");
    assert_eq!(
        value
            .serialize_specified_with_limits(Limits::new(1, 1, 1))
            .unwrap_err()
            .kind(),
        Kind::InputNodeLimit
    );
    assert_eq!(values.angle().origin(), &origin);
    assert_eq!(format!("{value:?}"), before);
}
