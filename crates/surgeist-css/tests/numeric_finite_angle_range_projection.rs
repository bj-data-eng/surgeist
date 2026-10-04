#![forbid(unsafe_code)]
//! Values 4 WD 2024-03-12 §5 requires an angle exceeding the supported
//! implementation range to convert to the nearest supported multiple of 360deg.
//! Every such finite multiple has cosine 1 and sine 0; these tests do not select
//! an implementation-specific endpoint or prescribe its conversion algorithm.
//! https://www.w3.org/TR/2024/WD-css-values-4-20240312/#numeric-types
//! §10.4 separately requires trig of genuine infinity to produce NaN, and
//! §10.13 preserves NaN in specified calculation serialization.
//! https://www.w3.org/TR/2024/WD-css-values-4-20240312/#trig-infinities
//! https://www.w3.org/TR/2024/WD-css-values-4-20240312/#calc-serialize
//! The finite 1e308 coefficient fits binary64; its turn/rad conversion to degrees
//! exceeds that range. The finite authored 1e400deg case instead exceeds the
//! coefficient range. Neither is an authored infinity or division by zero.
//! Target-context computed/used clamping is a different rule under §10.12.

use surgeist_css::*;

type Limits = CssSpecifiedValueSerializationLimits;
type Kind = CssSpecifiedValueSerializationErrorKind;

fn number(source: &str) -> CssSpecifiedNumber {
    CssSpecifiedNumber::try_from_calculation(
        CssNumberCalculation::try_from_components(parse_component_values(source).unwrap()).unwrap(),
    )
    .unwrap()
}

fn assert_number(source: &str, expected: &str) {
    let value = number(source);
    let before = value.clone();
    let calculation = value.calculation().unwrap();
    let components = calculation.components().clone();
    let ty = calculation.numeric_type();
    assert_eq!(components.serialize().unwrap().as_css(), source);
    assert!(matches!(value.origin(), CssValueOrigin::Parsed(_)));
    let result = value.serialize_specified();
    assert_eq!(value, before);
    assert_eq!(value.origin(), before.origin());
    assert_eq!(value.calculation().unwrap().components(), &components);
    assert_eq!(value.calculation().unwrap().numeric_type(), ty);
    assert_eq!(result.unwrap(), expected, "{source}");
}

#[test]
fn finite_turn_and_radian_conversion_overflow_has_cosine_one() {
    for source in [
        "cos(1e308turn)",
        "cos(-1e308turn)",
        "cos(1e308rad)",
        "cos(-1e308rad)",
    ] {
        assert_number(source, "calc(1)");
    }
}

#[test]
fn finite_turn_and_radian_conversion_overflow_has_sine_zero() {
    for source in [
        "sin(1e308turn)",
        "sin(-1e308turn)",
        "sin(1e308rad)",
        "sin(-1e308rad)",
    ] {
        assert_number(source, "calc(0)");
    }
}

#[test]
fn finite_authored_degree_coefficient_overflow_uses_the_angle_range_rule() {
    for (source, expected) in [
        ("cos(1e400deg)", "calc(1)"),
        ("cos(-1e400deg)", "calc(1)"),
        ("sin(1e400deg)", "calc(0)"),
        ("sin(-1e400deg)", "calc(0)"),
    ] {
        assert_number(source, expected);
    }
}

#[test]
fn genuine_authored_and_division_generated_infinities_still_have_nan_cosine() {
    for source in [
        "cos(infinity * 1deg)",
        "cos(-infinity * 1deg)",
        "cos(calc(1 / 0 * 1deg))",
        "cos(calc(-1 / 0 * 1deg))",
        "cos(infinity)",
        "cos(-infinity)",
    ] {
        assert_number(source, "calc(NaN)");
    }
}

#[test]
fn programmatic_finite_angle_projection_preserves_exact_lexeme_and_origin() {
    let leaf = CssComponentValue::try_dimension("+01e308", "TURN").unwrap();
    let CssComponentValueRef::Token(CssValueTokenRef::Dimension { number, unit }) = leaf.view()
    else {
        panic!("checked dimension");
    };
    assert_eq!(number.representation(), "+01e308");
    assert_eq!(unit, "TURN");
    let arguments = CssComponentValues::try_new(vec![leaf]).unwrap();
    let component = CssComponentValue::try_function("COS", arguments).unwrap();
    let components = CssComponentValues::try_new(vec![component]).unwrap();
    let value = CssSpecifiedNumber::try_from_calculation(
        CssNumberCalculation::try_from_components(components.clone()).unwrap(),
    )
    .unwrap();
    let before = value.clone();
    assert!(matches!(value.origin(), CssValueOrigin::Programmatic));
    let result = value.serialize_specified();
    assert_eq!(value, before);
    assert_eq!(value.origin(), before.origin());
    assert_eq!(value.calculation().unwrap().components(), &components);
    assert_eq!(result.unwrap(), "calc(1)");
}

#[test]
fn ordinary_rgb_keeps_red_after_finite_angle_range_conversion() {
    for (source, expected) in [
        ("rgb(calc(255 * cos(1e308turn)) 0 0)", "rgb(255, 0, 0)"),
        ("rgb(calc(255 * cos(-1e308rad)) 0 0)", "rgb(255, 0, 0)"),
        ("rgb(calc(255 * cos(1e400deg)) 0 0)", "rgb(255, 0, 0)"),
    ] {
        assert_color(source, expected);
    }
}

#[test]
fn ordinary_rgb_censors_genuine_infinite_angle_cosine_to_zero() {
    for source in [
        "rgb(calc(255 * cos(infinity * 1deg)) 0 0)",
        "rgb(calc(255 * cos(1 / 0 * 1deg)) 0 0)",
    ] {
        assert_color(source, "rgb(0, 0, 0)");
    }
}

fn assert_color(source: &str, expected: &str) {
    let components = parse_component_values(source).unwrap();
    let declaration = parse_property_value(
        CssPropertyNameRef::Known(CssKnownProperty::Color),
        components.clone(),
        CssImportance::Normal,
    )
    .unwrap();
    let before = declaration.clone();
    let CssKnownPropertyValueRef::Color(value) =
        declaration.known().unwrap().property_value().unwrap()
    else {
        panic!("checked color property");
    };
    let result = value.value().to_specified_css();
    assert_eq!(declaration, before);
    assert_eq!(declaration.value_components(), &components);
    assert_eq!(result.unwrap(), expected, "{source}");
}

#[test]
fn finite_angle_conversion_obeys_typed_limits_without_mutating_authored_values() {
    let value = number("cos(1e308turn)");
    let before = value.clone();
    for (limits, expected) in [
        (Limits::new(0, 100, 100), Kind::InputNodeLimit),
        (Limits::new(100, 0, 100), Kind::ProjectionNodeLimit),
        (Limits::new(100, 100, 0), Kind::ByteLimit),
    ] {
        let error = value.serialize_specified_with_limits(limits).unwrap_err();
        assert_eq!(error.kind(), expected);
        assert_eq!(value, before);
        assert_eq!(value.origin(), before.origin());
        assert_eq!(
            value.calculation().unwrap().components(),
            before.calculation().unwrap().components()
        );
    }
    let result = value.serialize_specified_with_limits(Limits::new(100, 100, 100));
    assert_eq!(value, before);
    assert_eq!(result.unwrap(), "calc(1)");
}

#[test]
fn multiple_finite_angle_conversions_share_cumulative_traversal_limits() {
    let value = number("calc(cos(1e308turn) + cos(-1e308rad))");
    let before = value.clone();
    for (limits, expected) in [
        (Limits::new(2, 100, 100), Kind::InputNodeLimit),
        (Limits::new(100, 2, 100), Kind::ProjectionNodeLimit),
    ] {
        let error = value.serialize_specified_with_limits(limits).unwrap_err();
        assert_eq!(error.kind(), expected);
        assert_eq!(value, before);
        assert_eq!(value.origin(), before.origin());
    }
    let result = value.serialize_specified_with_limits(Limits::new(100, 100, 100));
    assert_eq!(value, before);
    assert_eq!(result.unwrap(), "calc(2)");
}
