#![forbid(unsafe_code)]
//! Values 4 WD 2024-03-12 §10.10.1 combines same-unit Sum children.
//! Frozen WebKit 73aa6c89e2cb77c46184a81aec944e4ab99d114d
//! CSSCalcTree+Simplification.cpp iterates children from first to last and
//! accumulates each later coefficient into the first same-unit child.
//! These binary64 expectations follow independently from the spacing of two
//! around 10^16: adding one to either signed 10^16 rounds back to that value,
//! while cancellation before adding one leaves exactly one.

use surgeist_css::*;

fn number(source: &str) -> CssSpecifiedNumber {
    CssSpecifiedNumber::try_from_calculation(
        CssNumberCalculation::try_from_components(parse_component_values(source).unwrap()).unwrap(),
    )
    .unwrap()
}

fn length(source: &str) -> CssSpecifiedLength {
    CssSpecifiedLength::try_from_calculation(
        CssLengthCalculation::try_from_components(parse_component_values(source).unwrap()).unwrap(),
    )
    .unwrap()
}

#[test]
fn numeric_cancellation_uses_source_order_for_all_three_term_permutations() {
    for (source, expected) in [
        ("calc(1e16 + -1e16 + 1)", "calc(1)"),
        ("calc(-1e16 + 1e16 + 1)", "calc(1)"),
        ("calc(1e16 + 1 + -1e16)", "calc(0)"),
        ("calc(-1e16 + 1 + 1e16)", "calc(0)"),
        ("calc(1 + 1e16 + -1e16)", "calc(0)"),
        ("calc(1 + -1e16 + 1e16)", "calc(0)"),
        ("calc(1e16 - 1e16 + 1)", "calc(1)"),
    ] {
        let value = number(source);
        let before = value.clone();
        let calculation = value.calculation().unwrap();
        let components = calculation.components().clone();
        let ty = calculation.numeric_type();
        assert_eq!(components.serialize().unwrap().as_css(), source);
        let result = value.serialize_specified();
        assert_eq!(value, before);
        assert_eq!(value.origin(), before.origin());
        assert_eq!(value.calculation().unwrap().components(), &components);
        assert_eq!(value.calculation().unwrap().numeric_type(), ty);
        assert_eq!(result.unwrap(), expected, "{source}");
    }
}

#[test]
fn absolute_and_contextual_same_unit_coefficients_use_the_same_order() {
    for (source, expected) in [
        ("calc(1e16px - 1e16px + 1px)", "calc(1px)"),
        ("calc(1px + 1e16px - 1e16px)", "calc(0px)"),
        ("calc(1e16em - 1e16em + 1em)", "calc(1em)"),
        ("calc(1em + 1e16em - 1e16em)", "calc(0em)"),
        ("calc(1e16vw - 1e16vw + 1vw)", "calc(1vw)"),
        ("calc(1vw + 1e16vw - 1e16vw)", "calc(0vw)"),
        ("calc(1e16px + (-1e16px + 1px))", "calc(0px)"),
        ("calc((1e16px - 1e16px) + 1px)", "calc(1px)"),
    ] {
        let value = length(source);
        let before = value.clone();
        let ty = value.calculation().unwrap().numeric_type();
        let result = value.serialize_specified();
        assert_eq!(value, before);
        assert_eq!(value.origin(), before.origin());
        assert_eq!(
            value.calculation().unwrap().components(),
            before.calculation().unwrap().components()
        );
        assert_eq!(value.calculation().unwrap().numeric_type(), ty);
        assert_eq!(result.unwrap(), expected, "{source}");
    }
}

#[test]
fn programmatic_sum_keeps_exact_authored_tokens_and_origin() {
    let arguments = CssComponentValues::try_new(
        ["+01e16", " ", "-", " ", "1e16", " ", "+", " ", "1.0000000"]
            .into_iter()
            .map(|token| CssComponentValue::try_token(token).unwrap())
            .collect(),
    )
    .unwrap();
    let root = CssComponentValue::try_function("CALC", arguments).unwrap();
    let components = CssComponentValues::try_new(vec![root]).unwrap();
    let calculation = CssNumberCalculation::try_from_components(components.clone()).unwrap();
    let value = CssSpecifiedNumber::try_from_calculation(calculation).unwrap();
    let before = value.clone();
    assert!(matches!(value.origin(), CssValueOrigin::Programmatic));
    let result = value.serialize_specified();
    assert_eq!(value, before);
    assert_eq!(value.origin(), before.origin());
    assert_eq!(value.calculation().unwrap().components(), &components);
    assert_eq!(result.unwrap(), "calc(1)");
}

#[test]
fn ordered_sum_replacement_has_exact_resource_limits_and_atomic_failures() {
    let value = number("calc(1e16 - 1e16 + 1)");
    let before = value.clone();
    let expected = "calc(1)";
    // Input: calc, Sum, three leaves. Projection: three leaves, Negate
    // replacement for subtraction, and one combined scalar replacement.
    for (limits, kind) in [
        (
            CssSpecifiedValueSerializationLimits::new(4, 5, expected.len()),
            CssSpecifiedValueSerializationErrorKind::InputNodeLimit,
        ),
        (
            CssSpecifiedValueSerializationLimits::new(5, 4, expected.len()),
            CssSpecifiedValueSerializationErrorKind::ProjectionNodeLimit,
        ),
        (
            CssSpecifiedValueSerializationLimits::new(5, 5, expected.len() - 1),
            CssSpecifiedValueSerializationErrorKind::ByteLimit,
        ),
    ] {
        let error = value.serialize_specified_with_limits(limits).unwrap_err();
        assert_eq!(error.kind(), kind);
        assert_eq!(value, before);
        assert_eq!(value.origin(), before.origin());
    }
    let result = value.serialize_specified_with_limits(CssSpecifiedValueSerializationLimits::new(
        5,
        5,
        expected.len(),
    ));
    assert_eq!(value, before);
    assert_eq!(
        value.calculation().unwrap().components(),
        before.calculation().unwrap().components()
    );
    assert_eq!(result.unwrap(), expected);
}
