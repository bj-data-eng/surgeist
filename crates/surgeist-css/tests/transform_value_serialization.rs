#![forbid(unsafe_code)]
//! Functional tests for four new represented-value APIs, with no artificial
//! preimplementation RED. Transforms 1 §§4.1/5/9 and Transforms 2 §§5.1/12
//! govern order and equivalent defaults. Values 4 §10.13 and the shared numeric
//! owner govern specified math; parser/model grammar gaps remain separate.

use surgeist_css::{
    CssSpecifiedValueSerializationErrorKind as K, CssSpecifiedValueSerializationLimits as L, *,
};

fn number(text: &str) -> CssSpecifiedNumber {
    CssSpecifiedNumber::try_from_component(CssComponentValue::try_number(text).unwrap()).unwrap()
}
fn lp(text: &str) -> CssSpecifiedLengthPercentage {
    CssSpecifiedLengthPercentage::try_from_component(CssComponentValue::try_token(text).unwrap())
        .unwrap()
}
fn length(text: &str) -> CssSpecifiedLength {
    CssSpecifiedLength::try_from_component(CssComponentValue::try_token(text).unwrap()).unwrap()
}
fn angle(text: &str, unit: CssAngleUnit) -> CssAngleOrZero {
    CssAngleOrZero::Angle(CssAngleValue::from_literal(
        CssAngleLiteral::try_new(text, unit).unwrap(),
    ))
}
fn declaration(text: &str) -> CssDeclaration {
    let report = parse_style_attribute(text);
    assert!(report.is_clean(), "{text}: {:?}", report.diagnostics());
    let [value] = report.syntax().as_slice() else {
        panic!("one declaration")
    };
    value.clone()
}
macro_rules! parsed {
    ($source:expr, TransformOrigin) => {
        parsed!($source, TransformOrigin, origin)
    };
    ($source:expr, $variant:ident) => {
        parsed!($source, $variant, value)
    };
    ($source:expr, $variant:ident, $accessor:ident) => {{
        let declaration = declaration($source);
        let CssKnownPropertyValueRef::$variant(value) =
            declaration.known().unwrap().property_value().unwrap()
        else {
            panic!("expected transform property")
        };
        value.$accessor().clone()
    }};
}
fn transform(function: CssTransformFunction) -> CssTransform {
    CssTransform::Functions(CssTransformFunctionList::try_new(vec![function]).unwrap())
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
        (L::new(0, projection, expected.len()), K::InputNodeLimit),
        (L::new(input, 0, expected.len()), K::ProjectionNodeLimit),
        (L::new(input, projection, 0), K::ByteLimit),
    ] {
        assert_eq!(serialize(limits).unwrap_err().kind(), kind);
    }
}
macro_rules! check {
    ($value:expr, $input:expr, $projection:expr, $expected:expr) => {{
        let value = $value;
        let expected: &str = $expected;
        assert_eq!(value.serialize_specified().unwrap(), expected);
        budget(expected, $input, $projection, |limits| {
            value.serialize_specified_with_limits(limits)
        });
    }};
}

#[test]
fn all_twenty_one_functions_use_typed_spelling_order_and_exact_literal_costs() {
    for (source, expected, nodes) in [
        ("MATRIX(1,0,0,1,2,-3)", "matrix(1, 0, 0, 1, 2, -3)", 8),
        (
            "matrix3d(1,0,0,0,0,1,0,0,0,0,1,0,2,3,4,1)",
            "matrix3d(1, 0, 0, 0, 0, 1, 0, 0, 0, 0, 1, 0, 2, 3, 4, 1)",
            18,
        ),
        ("perspective(.25px)", "perspective(0.25px)", 3),
        ("rotate(30deg)", "rotate(30deg)", 3),
        ("rotate3d(1,2,3,30deg)", "rotate3d(1, 2, 3, 30deg)", 6),
        ("rotatex(20grad)", "rotateX(20grad)", 3),
        ("rotatey(.5turn)", "rotateY(0.5turn)", 3),
        ("rotatez(1rad)", "rotateZ(1rad)", 3),
        ("scale(2,3)", "scale(2, 3)", 4),
        ("scale3d(50%,100%,150%)", "scale3d(0.5, 1, 1.5)", 5),
        ("scalex(-2)", "scaleX(-2)", 3),
        ("scaley(3)", "scaleY(3)", 3),
        ("scalez(25%)", "scaleZ(0.25)", 3),
        ("skew(10deg,20deg)", "skew(10deg, 20deg)", 4),
        ("skewx(0)", "skewX(0)", 3),
        ("skewy(-10deg)", "skewY(-10deg)", 3),
        ("translate(10%,20px)", "translate(10%, 20px)", 4),
        ("translate3d(1%,2%,3px)", "translate3d(1%, 2%, 3px)", 5),
        ("translatex(-2px)", "translateX(-2px)", 3),
        ("translatey(15%)", "translateY(15%)", 3),
        ("translatez(-3px)", "translateZ(-3px)", 3),
    ] {
        let value = parsed!(&format!("transform:{source}"), Transform);
        let before = value.clone();
        check!(&value, nodes, nodes, expected);
        assert_eq!(value, before);
        let reentered = parsed!(&format!("transform:{expected}"), Transform);
        assert_eq!(reentered.serialize_specified().unwrap(), expected);
        let CssTransform::Functions(original) = &value else {
            panic!("functions")
        };
        let CssTransform::Functions(round_trip) = reentered else {
            panic!("functions")
        };
        assert_eq!(
            original.functions()[0].kind(),
            round_trip.functions()[0].kind()
        );
    }
}

#[test]
fn parsed_and_constructed_functions_reenter_with_the_same_semantic_fields() {
    for (value, expected) in [
        (
            transform(CssTransformFunction::Matrix(CssTransformMatrix::new([
                number("1"),
                number("0"),
                number("0"),
                number("1"),
                number("2"),
                number("3"),
            ]))),
            "matrix(1, 0, 0, 1, 2, 3)",
        ),
        (
            transform(CssTransformFunction::Translate3d(
                CssTransformTranslate3d::new(lp("10%"), lp("-2px"), length("3px")),
            )),
            "translate3d(10%, -2px, 3px)",
        ),
        (
            transform(CssTransformFunction::Skew(CssTransformSkew::new(
                angle("10", CssAngleUnit::Degrees),
                Some(angle("20", CssAngleUnit::Degrees)),
            ))),
            "skew(10deg, 20deg)",
        ),
        (
            transform(CssTransformFunction::Scale3d(CssTransformScale3d::new(
                CssTransformScaleComponent::Number(number("2")),
                CssTransformScaleComponent::Number(number("3")),
                CssTransformScaleComponent::Number(number("4")),
            ))),
            "scale3d(2, 3, 4)",
        ),
    ] {
        assert_eq!(value.serialize_specified().unwrap(), expected);
        assert_eq!(value, parsed!(&format!("transform:{expected}"), Transform));
    }
}

#[test]
fn transform_optional_arguments_are_classified_before_rounding_and_still_visited() {
    for (source, expected) in [
        ("translate(10px, 0px)", "translate(10px)"),
        ("translate(10px, 0%)", "translate(10px)"),
        ("translate(10px, 1e-999px)", "translate(10px, 0px)"),
        ("scale(2, 2.000)", "scale(2)"),
        ("scale(2, 2.0000004)", "scale(2, 2)"),
        ("skew(10deg, -0grad)", "skew(10deg)"),
        ("skew(10deg, 1e-999deg)", "skew(10deg, 0deg)"),
    ] {
        check!(
            parsed!(&format!("transform:{source}"), Transform),
            4,
            4,
            expected
        );
    }
}

#[test]
fn individual_translate_omits_only_ordinary_trailing_zero_lengths() {
    check!(CssTranslate::None, 1, 1, "none");
    for (source, expected, reentered_css, nodes) in [
        ("10px", "10px", "10px", 2),
        ("10px 0px", "10px", "10px", 3),
        ("10px 0px -0em", "10px", "10px", 4),
        ("10px 2px 0px", "10px 2px", "10px 2px", 4),
        ("10px 0px 3px", "10px 0px 3px", "10px 0px 3px", 4),
        ("10px 0%", "10px 0%", "10px 0%", 3),
        ("10px 0% 0px", "10px 0%", "10px 0%", 4),
        ("10px 1e-999px", "10px 0px", "10px", 3),
        ("10px 0px 1e-999px", "10px 0px 0px", "10px", 4),
    ] {
        let value = parsed!(&format!("translate:{source}"), Translate);
        let before = value.clone();
        check!(&value, nodes, nodes, expected);
        assert_eq!(value, before);
        assert_eq!(
            parsed!(&format!("translate:{expected}"), Translate)
                .serialize_specified()
                .unwrap(),
            reentered_css
        );
    }
    let value = CssTranslate::Values(
        CssTranslateValues::try_new(lp("25%"), Some(lp("-2px")), Some(length("3px"))).unwrap(),
    );
    assert_eq!(value, parsed!("translate:25% -2px 3px", Translate));
    assert_eq!(value.serialize_specified().unwrap(), "25% -2px 3px");
}

#[test]
fn individual_scale_uses_exact_literal_equality_and_one_before_shortening() {
    check!(CssScale::None, 1, 1, "none");
    for (source, expected, nodes) in [
        ("1", "1", 2),
        ("2 2", "2", 3),
        ("2 2.0 1e0", "2", 4),
        ("2 3 1", "2 3", 4),
        ("2 2 4", "2 2 4", 4),
        ("2 2.0000004 1", "2 2", 4),
        ("2 2 1.0000004", "2 2 1", 4),
        ("1e-999 2e-999", "0 0", 3),
    ] {
        let value = parsed!(&format!("scale:{source}"), Scale);
        let before = value.clone();
        check!(&value, nodes, nodes, expected);
        assert_eq!(value, before);
    }
    let value = CssScale::Values(
        CssScaleValues::try_new(vec![number("2"), number("3"), number("4")]).unwrap(),
    );
    assert_eq!(value, parsed!("scale:2 3 4", Scale));
    assert_eq!(value.serialize_specified().unwrap(), "2 3 4");
}

#[test]
fn transform_origin_reuses_physical_axes_and_retains_optional_z_and_provenance() {
    for (source, expected, nodes) in [
        ("top left", "left top", 4),
        ("top", "center top", 4),
        ("left top 0px", "left top 0px", 5),
        ("20% 30% -2px", "20% 30% -2px", 7),
        ("1e-999px bottom", "0px bottom", 5),
    ] {
        let value = parsed!(&format!("transform-origin:{source}"), TransformOrigin);
        let before = value.clone();
        check!(&value, nodes, nodes, expected);
        assert_eq!(value, before);
        let reentered = parsed!(&format!("transform-origin:{expected}"), TransformOrigin);
        assert_eq!(reentered.serialize_specified().unwrap(), expected);
    }
    let value = CssTransformOrigin::try_new(
        CssPhysicalPosition::try_new(CssHorizontalPosition::Left, CssVerticalPosition::Top)
            .unwrap(),
        Some(length("0px")),
    )
    .unwrap();
    assert_eq!(
        value,
        parsed!("transform-origin:left top 0px", TransformOrigin)
    );
    assert!(matches!(
        value.z().unwrap().origin(),
        CssValueOrigin::Programmatic
    ));
    let parsed = parsed!("/*😀*/ transform-origin:left top -2px", TransformOrigin);
    let CssValueOrigin::Parsed(origin) = parsed.z().unwrap().origin() else {
        panic!("parsed z origin")
    };
    assert_eq!(
        origin.source().as_str(),
        "/*😀*/ transform-origin:left top -2px"
    );
    assert_eq!(parsed.serialize_specified().unwrap(), "left top -2px");
}

#[test]
fn rotate3d_reuses_directed_parallel_and_identity_policy_with_derived_costs() {
    for (source, expected, input, projection) in [
        ("rotate3d(2, 0, 0, 30deg)", "rotate3d(1, 0, 0, 30deg)", 6, 9),
        (
            "rotate3d(-2, 0, 0, 30deg)",
            "rotate3d(1, 0, 0, -30deg)",
            6,
            10,
        ),
        (
            "rotate3d(0, -2, 0, 30deg)",
            "rotate3d(0, 1, 0, -30deg)",
            6,
            10,
        ),
        (
            "rotate3d(0, 0, 1e-999, 30deg)",
            "rotate3d(0, 0, 1, 30deg)",
            6,
            9,
        ),
        ("rotate3d(0, 0, 0, 30deg)", "rotate3d(0, 0, 1, 0deg)", 6, 10),
        (
            "rotate3d(1, 2, 3, 360deg)",
            "rotate3d(0, 0, 1, 0deg)",
            6,
            10,
        ),
        (
            "rotate3d(1, 2, 3, 400grad)",
            "rotate3d(0, 0, 1, 0deg)",
            6,
            10,
        ),
        (
            "rotate3d(1, 2, 3, -1turn)",
            "rotate3d(0, 0, 1, 0deg)",
            6,
            10,
        ),
        ("rotate3d(1, 2, 3, 0)", "rotate3d(0, 0, 1, 0deg)", 6, 10),
    ] {
        let value = parsed!(&format!("transform:{source}"), Transform);
        let before = value.clone();
        check!(&value, input, projection, expected);
        assert_eq!(value, before);
        assert_eq!(
            parsed!(&format!("transform:{expected}"), Transform)
                .serialize_specified()
                .unwrap(),
            expected
        );
    }
}

#[test]
fn rotate3d_general_vectors_approximate_survivors_and_fail_on_component_loss_atomically() {
    check!(
        parsed!("transform:rotate3d(1, 1.0000004, 0, 30deg)", Transform),
        6,
        6,
        "rotate3d(1, 1, 0, 30deg)"
    );
    for source in [
        "rotate3d(1e-999, 1, 0, 30deg)",
        "rotate3d(1, -.00000001, 0, 30deg)",
        "rotate3d(calc(1 / 2097152), 1, 0, 30deg)",
    ] {
        let value = parsed!(
            &format!("transform:translateX(2px) {source} scale(2)"),
            Transform
        );
        let before = value.clone();
        assert_eq!(
            value.serialize_specified().unwrap_err().kind(),
            K::UnrepresentableValue
        );
        assert_eq!(value, before);
    }
    assert_eq!(
        parsed!("transform:rotate3d(1, 2, 3, 30deg)", Transform)
            .serialize_specified()
            .unwrap(),
        "rotate3d(1, 2, 3, 30deg)"
    );
}

#[test]
fn rotate3d_calculations_share_projection_and_identity_bypasses_discarded_axis_formatting() {
    check!(
        parsed!(
            "transform:rotate3d(calc(1 + 1), 0, 0, calc(15deg + 15deg))",
            Transform
        ),
        12,
        13,
        "rotate3d(1, 0, 0, calc(30deg))"
    );
    check!(
        parsed!(
            "transform:rotate3d(1, 2, 3, calc(180deg + 180deg))",
            Transform
        ),
        9,
        12,
        "rotate3d(0, 0, 1, 0deg)"
    );
    check!(
        parsed!("transform:rotate3d(.00000001, 1, 0, 0deg)", Transform),
        6,
        10,
        "rotate3d(0, 0, 1, 0deg)"
    );
}

#[test]
fn specified_calculations_remain_wrapped_symbolic_and_unclamped() {
    for (source, expected, input, projection) in [
        (
            "transform:translateX(calc(1px + 2px))",
            "translateX(calc(3px))",
            6,
            5,
        ),
        ("transform:scaleX(calc(1 + 2))", "scaleX(calc(3))", 6, 5),
        (
            "transform:perspective(calc(-1px + -2px))",
            "perspective(calc(-3px))",
            6,
            5,
        ),
    ] {
        check!(parsed!(source, Transform), input, projection, expected);
    }
    for (source, expected) in [
        (
            "transform:translateX(calc(1px + 2em))",
            "translateX(calc(2em + 1px))",
        ),
        (
            "transform:translate(10px, calc(0px))",
            "translate(10px, calc(0px))",
        ),
        (
            "transform:skew(10deg, calc(0deg))",
            "skew(10deg, calc(0deg))",
        ),
        (
            "transform:scale3d(calc((1px + 1%) / 1px), 1, 1)",
            "scale3d(calc((1% + 1px) / 1px), 1, 1)",
        ),
        ("transform:scaleZ(calc(50% + 50%))", "scaleZ(calc(1))"),
        (
            "transform:rotate(calc(infinity * 1deg))",
            "rotate(calc(infinity * 1deg))",
        ),
    ] {
        let value = parsed!(source, Transform);
        let before = value.clone();
        assert_eq!(value.serialize_specified().unwrap(), expected);
        assert_eq!(value, before);
        assert_eq!(
            parsed!(&format!("transform:{expected}"), Transform)
                .serialize_specified()
                .unwrap(),
            expected
        );
    }
    let translate = parsed!("translate:10px calc(0px) calc(0px)", Translate);
    assert_eq!(
        translate.serialize_specified().unwrap(),
        "10px calc(0px) calc(0px)"
    );
    let origin = parsed!(
        "transform-origin:calc(1px + 2em) center calc(0px)",
        TransformOrigin
    );
    assert_eq!(
        origin.serialize_specified().unwrap(),
        "calc(2em + 1px) center calc(0px)"
    );
}

#[test]
fn percentage_scale_conversion_is_exact_before_rounding_and_keeps_authored_units() {
    for (source, expected) in [
        ("scaleZ(.00005%)", "scaleZ(0.000001)"),
        ("scaleZ(-.00005%)", "scaleZ(-0.000001)"),
        ("scaleZ(1e-999%)", "scaleZ(0)"),
        ("scaleZ(150%)", "scaleZ(1.5)"),
    ] {
        let value = parsed!(&format!("transform:{source}"), Transform);
        let before = value.clone();
        check!(&value, 3, 3, expected);
        assert_eq!(value, before);
    }
}

#[test]
fn rounded_output_and_atomic_byte_failure_preserve_exact_operands_and_origins() {
    let source = "/*😀*/\r\nscale:2 2.0000004 1!important";
    let value = parsed!(source, Scale);
    let before = value.clone();
    assert_eq!(value.serialize_specified().unwrap(), "2 2");
    assert_eq!(
        value
            .serialize_specified_with_limits(L::new(4, 4, 2))
            .unwrap_err()
            .kind(),
        K::ByteLimit
    );
    assert_eq!(value, before);
    let CssScale::Values(values) = &value else {
        panic!("scale values")
    };
    let operand = &values.values()[1];
    let CssComponentValueRef::Token(CssValueTokenRef::Number(token)) =
        operand.literal_component().unwrap().view()
    else {
        panic!("number token")
    };
    assert_eq!(token.representation(), "2.0000004");
    let CssValueOrigin::Parsed(origin) = operand.origin() else {
        panic!("parsed origin")
    };
    assert_eq!(origin.source().as_str(), source);
    assert_eq!(origin.span().start().line().value(), 1);
    assert_eq!(origin.span().start().column().value(), 8);

    let value = transform(CssTransformFunction::TranslateX(lp("1e999px")));
    let before = value.clone();
    assert_eq!(
        value
            .serialize_specified_with_limits(L::new(3, 3, 64))
            .unwrap_err()
            .kind(),
        K::ByteLimit
    );
    assert_eq!(value, before);
    let CssTransform::Functions(values) = value else {
        panic!("functions")
    };
    let CssTransformFunction::TranslateX(value) = &values.functions()[0] else {
        panic!("translate x")
    };
    assert!(matches!(value.origin(), CssValueOrigin::Programmatic));
    let CssComponentValueRef::Token(CssValueTokenRef::Dimension { number, unit }) =
        value.literal_component().unwrap().view()
    else {
        panic!("dimension")
    };
    assert_eq!(number.representation(), "1e999");
    assert_eq!(unit, "px");
}

#[test]
fn list_order_large_incremental_limits_and_late_failure_preserve_input() {
    check!(CssTransform::None, 1, 1, "none");
    let source = "transform:translateX(1px) scale(2, 2) rotate(30deg)";
    let value = parsed!(source, Transform);
    check!(&value, 8, 8, "translateX(1px) scale(2) rotate(30deg)");
    let before = value.clone();
    assert_eq!(
        value
            .serialize_specified_with_limits(L::new(8, 8, 20))
            .unwrap_err()
            .kind(),
        K::ByteLimit
    );
    assert_eq!(value, before);
    let function = CssTransformFunction::TranslateX(lp("1px"));
    let value =
        CssTransform::Functions(CssTransformFunctionList::try_new(vec![function; 10_000]).unwrap());
    let before = value.clone();
    assert_eq!(
        value
            .serialize_specified_with_limits(L::new(5, 20_001, 1_000_000))
            .unwrap_err()
            .kind(),
        K::InputNodeLimit
    );
    assert_eq!(
        value
            .serialize_specified_with_limits(L::new(20_001, 5, 1_000_000))
            .unwrap_err()
            .kind(),
        K::ProjectionNodeLimit
    );
    assert_eq!(
        value
            .serialize_specified_with_limits(L::new(20_001, 20_001, 20))
            .unwrap_err()
            .kind(),
        K::ByteLimit
    );
    assert_eq!(value, before);
}
