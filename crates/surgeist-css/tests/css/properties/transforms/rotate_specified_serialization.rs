//! New rotate output API: Transforms2 §5/§5.1/§12.2, Values3 angles, and the
//! existing six-place numeric contract. No preimplementation callable output API
//! existed; these are functional tests alongside the new API, not regression RED.
//! #1298 revises #554's identity folding: Transforms2 §5's computed value is an
//! angle with an axis, so full turns and zero axes keep their authored form.
use crate::css::support::failure_classes::SerializationFailure;
use crate::css::support::failure_classes::serialization_failure;
use surgeist_css::CssParser;
use surgeist_css::component::{CssComponentValue, CssComponentValues, CssValueOrigin};
use surgeist_css::declaration::CssDeclaration;
use surgeist_css::numeric::{
    CssAngle, CssAngleUnit, CssCalculation, CssNumber, CssNumeric, CssNumericLiteral,
};
use surgeist_css::property::CssKnownPropertyValueRef;
use surgeist_css::serialization::CssSerializationLimits;
use surgeist_css::values::transform::{CssRotate, CssRotateAxis, CssRotateValues};

type Limits = CssSerializationLimits;
type Kind = SerializationFailure;

fn parsed(text: &str) -> CssRotate {
    let source = format!("rotate:{text}");
    let report = CssParser::default().parse_block_contents(&source);
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
fn number(text: &str) -> CssNumeric<CssNumber> {
    CssNumeric::<CssNumber>::try_from_component(CssComponentValue::try_number(text).unwrap())
        .unwrap()
}
fn ordinary(angle: &str, unit: CssAngleUnit, axis: Option<CssRotateAxis>) -> CssRotate {
    CssRotate::Value(CssRotateValues::new(
        CssNumeric::<CssAngle>::try_from_literal(
            CssNumericLiteral::<CssAngle>::try_new(angle, unit).unwrap(),
        )
        .unwrap(),
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
    assert_eq!(
        value.serialize_specified(Default::default()).unwrap(),
        expected,
        "{input}"
    );
    // Returned CSS is accepted through the actual typed grammar again.
    parsed(expected);
}

#[test]
fn none_full_turns_zero_angles_and_zero_axes_keep_their_authored_form() {
    // Transforms 2 §5: the computed value is "an <angle> with an axis", not a
    // normalized orientation, and a [0,0,0] axis only makes the rotation not
    // apply. Specified serialization therefore folds neither (#1298).
    for (input, expected) in [
        ("none", "none"),
        ("0deg", "0deg"),
        ("x 0deg", "x 0deg"),
        ("x -0turn", "x 0turn"),
        ("360deg", "360deg"),
        ("calc(360deg)", "calc(360deg)"),
        ("calc(720deg)", "calc(720deg)"),
        ("calc(400deg)", "calc(400deg)"),
        ("0 0 0 45deg", "0 0 0 45deg"),
        ("0 0 0 400grad", "0 0 0 400grad"),
        ("0 -0 0 calc(45deg + 15deg)", "0 0 0 calc(60deg)"),
        ("1 2 3 360deg", "1 2 3 360deg"),
        ("x -720deg", "x -720deg"),
        ("y 400grad", "y 400grad"),
        ("z -2turn", "-2turn"),
        ("x calc(1turn + 360deg)", "x calc(720deg)"),
        ("0 0 0 calc(infinity * 1deg)", "0 0 0 calc(infinity * 1deg)"),
    ] {
        output(input, expected);
    }
}

#[test]
fn large_projected_full_turn_magnitudes_are_not_folded() {
    // binary64(1e300) is an exact multiple of 360; arithmetic keeps that
    // projected magnitude, and the single leaf keeps its exact coefficient.
    output("calc(1e300deg * 1)", &format!("calc({:.0}deg)", 1e300_f64));
    output("calc(1e300deg)", &format!("calc(1{}deg)", "0".repeat(300)));
    output("1e300deg", &format!("1{}deg", "0".repeat(300)));
}

#[test]
fn wpt_rotate_parsing_valid_specified_values() {
    // css/css-transforms/parsing/rotate-parsing-valid.html (WebKit import
    // 73aa6c89e2cb), test_valid_value(property, input, expected = input).
    for (input, expected) in [
        ("none", "none"),
        ("0deg", "0deg"),
        ("100 200 300 400grad", "100 200 300 400grad"),
        ("400grad 100 200 300", "100 200 300 400grad"),
        ("0 0 0 400grad", "0 0 0 400grad"),
        ("x 400grad", "x 400grad"),
        ("400grad x", "x 400grad"),
        ("0.5 0 0 400grad", "x 400grad"),
        ("1 0 0 400grad", "x 400grad"),
        ("y 400grad", "y 400grad"),
        ("400grad y", "y 400grad"),
        ("0 0.5 0 400grad", "y 400grad"),
        ("0 1 0 400grad", "y 400grad"),
        ("-0.5 0 0 400grad", "x -400grad"),
        ("-1 0 0 400grad", "x -400grad"),
        ("0 -0.5 0 400grad", "y -400grad"),
        ("0 -1 0 400grad", "y -400grad"),
        ("400grad", "400grad"),
        ("400grad z", "400grad"),
        ("0 0 0.5 400grad", "400grad"),
        ("0 0 1 400grad", "400grad"),
        ("0 0 -0.5 400grad", "-400grad"),
        ("0 0 -1 400grad", "-400grad"),
    ] {
        output(input, expected);
    }
}

#[test]
fn property_and_transform_function_paths_agree_on_equivalent_rotations() {
    fn declaration(source: &str) -> String {
        let report = CssDeclaration::parse(&CssParser::default(), source);
        assert!(report.is_clean(), "{source}: {:?}", report.diagnostics());
        report
            .syntax()
            .as_ref()
            .unwrap()
            .serialize_specified(Default::default())
            .unwrap()
    }
    for angle in [
        "360deg",
        "-720deg",
        "400grad",
        "2turn",
        "calc(360deg)",
        "calc(720deg)",
        "calc(1turn + 360deg)",
        "calc(1e300deg)",
        "calc(1e300deg * 1)",
    ] {
        let property = declaration(&format!("rotate:{angle}"));
        let function = declaration(&format!("transform:rotate({angle})"));
        let property = property
            .strip_prefix("rotate: ")
            .and_then(|rest| rest.strip_suffix(';'))
            .unwrap();
        assert_eq!(
            function,
            format!("transform: rotate({property});"),
            "{angle}"
        );
    }
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
                .serialize_specified(Limits::new(6, 7, expected.len()))
                .unwrap(),
            expected
        );
    }
}

#[test]
fn exact_full_turn_literals_keep_their_coefficient_and_unit() {
    for (coefficient, unit, expected) in [
        ("360.000", CssAngleUnit::Degrees, "360deg"),
        (".4e3", CssAngleUnit::Gradians, "400grad"),
        ("1.000", CssAngleUnit::Turns, "1turn"),
    ] {
        output_value(ordinary(coefficient, unit, None), expected);
    }
    // Unbounded exponents are no longer classified as identities; their
    // ordinary output cannot be expanded within the byte limit.
    for (coefficient, unit) in [
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
        assert!(
            ordinary(coefficient, unit, None)
                .serialize_specified(Limits::new(2, 3, 4))
                .unwrap_err()
                .is_byte_limit()
        );
    }
    output("360.000001deg", "360.000001deg");
    output("400.000001grad", "400.000001grad");
    output("1.000001turn", "1.000001turn");
    output("6.283185rad", "6.283185rad");
    output("-1e-400deg", "0deg");
    output("-1 0 0 -1e-400deg", "x 0deg");
}

fn output_value(value: CssRotate, expected: &str) {
    assert_eq!(
        value.serialize_specified(Default::default()).unwrap(),
        expected
    );
    parsed(expected);
}

#[test]
fn general_vector_rejects_only_proved_nonzero_components_that_round_to_zero() {
    for axis in [
        [".00000001", "1", "0"],
        ["1", "-.00000001", "0"],
        ["1e-400", "2e-400", "0"],
    ] {
        let value = vector("30", axis);
        assert!(
            value
                .serialize_specified(Default::default())
                .unwrap_err()
                .is_unrepresentable_value()
        );
    }
    for input in ["calc(.00000001) 1 0 30deg", "1 calc(-.00000001) 0 30deg"] {
        assert!(
            parsed(input)
                .serialize_specified(Default::default())
                .unwrap_err()
                .is_unrepresentable_value()
        );
    }
    assert!(
        parsed("calc(1 / 2097152) 1 0 30deg")
            .serialize_specified(Default::default())
            .unwrap_err()
            .is_unrepresentable_value()
    );
    output("calc(1 / 1048576) 1 0 30deg", "calc(0.000001) 1 0 30deg");
    // The surviving components are still rounded independently, not normalized.
    output("1.23456789 2.34567891 0 30deg", "1.234568 2.345679 0 30deg");
    output(".0000005 1 0 30deg", "0.000001 1 0 30deg");
    output("-.0000005 1 0 30deg", "-0.000001 1 0 30deg");
    assert!(
        vector("30", ["1e400", "1", "0"])
            .serialize_specified(Limits::new(6, 6, 20))
            .unwrap_err()
            .is_byte_limit()
    );
}

#[test]
fn symbolic_and_nonfinite_axis_math_is_retained_without_parallel_axis_guesses() {
    output("calc(1em / 1px) 0 0 30deg", "calc(1em / 1px) 0 0 30deg");
    output("calc(infinity) 0 0 30deg", "calc(infinity) 0 0 30deg");
    output("calc(NaN) 0 0 30deg", "calc(NaN) 0 0 30deg");
    // A finite authored coefficient keeps its exact text and converts to the
    // closest finite binary64 value for classification (#1297).
    output("calc(1e400) 0 0 30deg", "x 30deg");
    output(
        "calc(1e400) 1 0 30deg",
        &format!("calc(1{}) 1 0 30deg", "0".repeat(400)),
    );
    output("calc(1 + 1) 0 0 30deg", "x 30deg");
    // An operation-free leaf is classified and emitted from its exact authored
    // coefficient, as the literal is (#1297); arithmetic keeps binary64, where
    // this nonzero decimal underflows to an unnormalizable zero axis.
    output("calc(1e-400) 0 0 30deg", "x 30deg");
    output("calc(1e-400 * 1) 0 0 30deg", "calc(0) 0 0 30deg");
    output("calc(5e-7) 1 0 30deg", "calc(0.000001) 1 0 30deg");
    assert!(
        parsed("calc(5e-7 * 1) 1 0 30deg")
            .serialize_specified(Default::default())
            .unwrap_err()
            .is_unrepresentable_value()
    );
    assert!(
        parsed("calc(4e-7) 1 0 30deg")
            .serialize_specified(Default::default())
            .unwrap_err()
            .is_unrepresentable_value()
    );
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
fn limits_charge_authored_children_and_derived_sign_work() {
    let positive = vector("45", ["2", "0", "0"]);
    assert_eq!(
        positive.serialize_specified(Limits::new(6, 6, 7)).unwrap(),
        "x 45deg"
    );
    for (limits, kind) in [
        (Limits::new(5, 6, 7), Kind::InputNodeLimit),
        (Limits::new(6, 5, 7), Kind::ProjectionNodeLimit),
        (Limits::new(6, 6, 6), Kind::ByteLimit),
    ] {
        assert_eq!(
            serialization_failure(&positive.serialize_specified(limits).unwrap_err()),
            kind
        );
    }
    let negative = vector("45", ["-2", "0", "0"]);
    assert_eq!(
        negative.serialize_specified(Limits::new(6, 7, 8)).unwrap(),
        "x -45deg"
    );
    assert!(
        negative
            .serialize_specified(Limits::new(6, 6, 8))
            .unwrap_err()
            .is_projection_node_limit()
    );
    // A zero axis emits its authored children; no identity replacement is derived.
    let zero_axis = vector("45", ["0", "0", "0"]);
    assert_eq!(
        zero_axis
            .serialize_specified(Limits::new(6, 6, 11))
            .unwrap(),
        "0 0 0 45deg"
    );
    for (limits, kind) in [
        (Limits::new(5, 6, 11), Kind::InputNodeLimit),
        (Limits::new(6, 5, 11), Kind::ProjectionNodeLimit),
        (Limits::new(6, 6, 10), Kind::ByteLimit),
    ] {
        assert_eq!(
            serialization_failure(&zero_axis.serialize_specified(limits).unwrap_err()),
            kind
        );
    }
    assert_eq!(
        CssRotate::None
            .serialize_specified(Limits::new(1, 1, 4))
            .unwrap(),
        "none"
    );
}

#[test]
fn zero_axis_children_are_emitted_with_their_ordinary_work_and_bytes() {
    // Rotate, axis, three axis leaves, calc, sum and two angle leaves: 9 input
    // nodes. Five literal projections and three angle scalar nodes: 8.
    let value = parsed("0 0 0 calc(15deg + 15deg)");
    assert_eq!(
        value.serialize_specified(Limits::new(9, 8, 17)).unwrap(),
        "0 0 0 calc(30deg)"
    );
    for (limits, kind) in [
        (Limits::new(8, 8, 17), Kind::InputNodeLimit),
        (Limits::new(9, 7, 17), Kind::ProjectionNodeLimit),
        (Limits::new(9, 8, 16), Kind::ByteLimit),
    ] {
        assert_eq!(
            serialization_failure(&value.serialize_specified(limits).unwrap_err()),
            kind
        );
    }
    let huge = format!("0 0 0 1{}deg", "0".repeat(400));
    assert_eq!(
        vector("1e400", ["0", "0", "0"])
            .serialize_specified(Limits::new(6, 6, huge.len()))
            .unwrap(),
        huge
    );
    assert!(
        vector("1e400", ["0", "0", "0"])
            .serialize_specified(Limits::new(6, 6, huge.len() - 1))
            .unwrap_err()
            .is_byte_limit()
    );
    assert_eq!(
        parsed("z 45deg")
            .serialize_specified(Limits::new(3, 3, 5))
            .unwrap(),
        "45deg"
    );
    assert!(
        parsed("z 45deg")
            .serialize_specified(Limits::new(2, 3, 5))
            .unwrap_err()
            .is_input_node_limit()
    );
}

#[test]
fn checked_number_and_angle_math_share_canonical_output_with_parser_admission() {
    let axis = CssNumeric::<CssNumber>::try_from_calculation(
        CssCalculation::<CssNumber>::try_from_components(
            CssComponentValues::parse(&CssParser::default(), "calc(-1 - 1)").unwrap(),
            Default::default(),
        )
        .unwrap(),
    )
    .unwrap();
    let angle = CssNumeric::<CssAngle>::try_from_calculation(
        CssCalculation::<CssAngle>::try_from_components(
            CssComponentValues::parse(&CssParser::default(), "calc(15deg + 15deg)").unwrap(),
            Default::default(),
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
    assert_eq!(
        value.serialize_specified(Default::default()).unwrap(),
        "x calc(-30deg)"
    );
    assert_eq!(
        parsed("calc(-1 - 1) 0 0 calc(15deg + 15deg)")
            .serialize_specified(Default::default())
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
    let report = CssParser::default().parse_block_contents(source);
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
    assert_eq!(
        value.serialize_specified(Default::default()).unwrap(),
        "calc(-30deg)"
    );
    assert!(
        value
            .serialize_specified(Limits::new(1, 1, 1))
            .unwrap_err()
            .is_input_node_limit()
    );
    assert_eq!(values.angle().origin(), &origin);
    assert_eq!(format!("{value:?}"), before);
}

#[test]
fn rotate3d_classifies_operation_free_axis_leaves_from_their_exact_coefficients() {
    // Transforms 2 §12.2 normalizes the direction vector [x, y, z]: the exact
    // authored 1e-400 is a positive x-only vector, hence the unit x axis, and
    // 5e-7 is a nonzero component emitted with its exact six-place text (#1297).
    fn declaration(source: &str) -> String {
        let report = CssDeclaration::parse(&CssParser::default(), source);
        assert!(report.is_clean(), "{source}: {:?}", report.diagnostics());
        report
            .syntax()
            .as_ref()
            .unwrap()
            .serialize_specified(Default::default())
            .unwrap()
    }
    for (input, expected) in [
        (
            "rotate3d(calc(1e-400), 0, 0, 30deg)",
            "rotate3d(1, 0, 0, 30deg)",
        ),
        (
            "rotate3d(calc(5e-7), 1, 0, 30deg)",
            "rotate3d(calc(0.000001), 1, 0, 30deg)",
        ),
    ] {
        assert_eq!(
            declaration(&format!("transform:{input}")),
            format!("transform: {expected};"),
            "{input}"
        );
    }
}

#[test]
fn rotate3d_and_rotate_keep_the_same_authored_zero_axes_and_full_turns() {
    // Transforms 1 §4.1 serializes a function as its individual grammar, and
    // Transforms 2 §12.2 keeps a function whose direction vector cannot be
    // normalized; §5.1 gives `rotate` no folding clause either (#1298).
    // Equivalent rotations therefore keep their zero axis and full-turn angle
    // on both paths; only the axis spelling differs by grammar.
    fn function(source: &str) -> String {
        let report = CssDeclaration::parse(&CssParser::default(), format!("transform:{source}"));
        assert!(report.is_clean(), "{source}: {:?}", report.diagnostics());
        report
            .syntax()
            .as_ref()
            .unwrap()
            .serialize_specified(Default::default())
            .unwrap()
    }
    for (property, function_source) in [
        ("0 0 0 400grad", "rotate3d(0, 0, 0, 400grad)"),
        ("0 0 0 30deg", "rotate3d(0, 0, 0, 30deg)"),
        ("1 2 3 360deg", "rotate3d(1, 2, 3, 360deg)"),
        ("1 2 3 calc(720deg)", "rotate3d(1, 2, 3, calc(720deg))"),
        ("x 400grad", "rotate3d(1, 0, 0, 400grad)"),
        ("y -720deg", "rotate3d(0, 1, 0, -720deg)"),
        ("360deg", "rotate3d(0, 0, 1, 360deg)"),
    ] {
        output(property, property);
        assert_eq!(
            function(function_source),
            format!("transform: {function_source};"),
            "{function_source}"
        );
    }
}
