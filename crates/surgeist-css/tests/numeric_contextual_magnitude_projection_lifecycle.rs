#![forbid(unsafe_code)]
//! Values 4 WD 2024-03-12 §10.6 defines abs() on the resolved value.
//! For a nonnegative unit basis b, abs(a*b) = abs(a)*b, including b=0.
//! The em/rem definitions refer to computed font size; pinned Fonts 4 WD
//! 2026-09-07 allows font-size in [0, infinity]. Values 4 defines vw as a
//! percentage of viewport width. These bases need not be strictly positive.
//! Consequently sign(-2em) must remain symbolic: its result changes at b=0.
//! Values 4 §10.5 explicitly gives hypot(3em, 4em) = 5em; for these bases,
//! hypot(a*b, c*b) = hypot(a, c)*b likewise preserves an unknown zero basis.
//! Sources:
//! https://www.w3.org/TR/2024/WD-css-values-4-20240312/#exponent-funcs
//! https://www.w3.org/TR/2024/WD-css-values-4-20240312/#sign-funcs
//! https://www.w3.org/TR/2024/WD-css-values-4-20240312/#font-relative-lengths
//! https://www.w3.org/TR/2024/WD-css-values-4-20240312/#viewport-relative-lengths
//! https://www.w3.org/TR/2026/WD-css-fonts-4-20260907/#propdef-font-size
//! Frozen WebKit 73aa6c89e2cb77c46184a81aec944e4ab99d114d also permits
//! contextual numeric coefficients in simplify(Abs&) through magnitudeComparable.

use surgeist_css::*;

type Limits = CssSpecifiedValueSerializationLimits;
type Kind = CssSpecifiedValueSerializationErrorKind;

fn length(source: &str) -> CssSpecifiedLength {
    CssSpecifiedLength::try_from_calculation(
        CssLengthCalculation::try_from_components(parse_component_values(source).unwrap()).unwrap(),
    )
    .unwrap()
}

fn assert_length(source: &str, expected: &str) {
    let value = length(source);
    let before = value.clone();
    let calculation = value.calculation().unwrap();
    let ty = calculation.numeric_type();
    let components = calculation.components().clone();
    assert_eq!(components.serialize().unwrap().as_css(), source);
    let result = value.serialize_specified();
    assert_eq!(value, before);
    assert_eq!(value.origin(), before.origin());
    assert_eq!(value.calculation().unwrap().components(), &components);
    assert_eq!(value.calculation().unwrap().numeric_type(), ty);
    assert_eq!(result.unwrap(), expected, "{source}");
}

#[test]
fn contextual_abs_projects_positive_negative_and_zero_coefficients_in_the_same_unit() {
    for (source, expected) in [
        ("abs(-2em)", "calc(2em)"),
        ("abs(2em)", "calc(2em)"),
        ("abs(0em)", "calc(0em)"),
        ("abs(-2rem)", "calc(2rem)"),
        ("abs(2rem)", "calc(2rem)"),
        ("abs(0rem)", "calc(0rem)"),
        ("abs(-2vw)", "calc(2vw)"),
        ("abs(2vw)", "calc(2vw)"),
        ("abs(0vw)", "calc(0vw)"),
        ("abs(0 * -1em)", "calc(0em)"),
    ] {
        assert_length(source, expected);
    }
}

#[test]
fn nested_contextual_abs_projects_known_coefficients_without_resolving_the_basis() {
    for (source, expected) in [
        ("abs(-2 * 1em)", "calc(2em)"),
        ("abs(-1em - 1em)", "calc(2em)"),
        ("abs(abs(-2em))", "calc(2em)"),
        ("min(abs(-2em),3em)", "calc(2em)"),
        ("abs(1em - 1px)", "abs(1em - 1px)"),
    ] {
        assert_length(source, expected);
    }
}

#[test]
fn contextual_hypot_projects_same_unit_norms_including_zero_coefficients() {
    for (source, expected) in [
        ("hypot(3em,4em)", "calc(5em)"),
        ("hypot(-3em,4em)", "calc(5em)"),
        ("hypot(0em,0em)", "calc(0em)"),
        ("hypot(-3rem,-4rem)", "calc(5rem)"),
        ("hypot(3vw,4vw)", "calc(5vw)"),
        ("hypot(0vw,0vw)", "calc(0vw)"),
        ("hypot(3em)", "calc(3em)"),
        ("hypot(abs(-3em),4em)", "calc(5em)"),
        ("hypot(3em,4px)", "hypot(3em, 4px)"),
    ] {
        assert_length(source, expected);
    }
}

#[test]
fn percentage_hypot_distinguishes_raw_norms_from_unknown_signed_bases() {
    let source = "hypot(3%,4%)";
    let components = parse_component_values(source).unwrap();
    let raw = CssPercentageCalculation::try_from_components(components.clone()).unwrap();
    assert_eq!(raw.numeric_type().percent_hint(), None);
    let raw = CssSpecifiedPercentage::try_from_calculation(raw).unwrap();
    let raw_before = raw.clone();
    assert_eq!(raw.serialize_specified().unwrap(), "calc(5%)");
    assert_eq!(raw, raw_before);
    assert_eq!(raw.calculation().unwrap().components(), &components);

    let hinted = CssLengthPercentageCalculation::try_from_components(components.clone()).unwrap();
    assert_eq!(
        hinted.numeric_type().percent_hint(),
        Some(CssNumericDimension::Length)
    );
    let hinted = CssSpecifiedLengthPercentage::try_from_calculation(hinted).unwrap();
    let hinted_before = hinted.clone();
    assert_eq!(hinted.serialize_specified().unwrap(), "hypot(3%, 4%)");
    assert_eq!(hinted, hinted_before);
    assert_eq!(hinted.calculation().unwrap().components(), &components);
}

#[test]
fn sign_retains_unknown_zero_bases_and_mixed_unit_magnitudes() {
    for (source, expected) in [
        ("sign(-2em)", "sign(-2em)"),
        ("sign(2em)", "sign(2em)"),
        ("sign(-2rem)", "sign(-2rem)"),
        ("sign(-2vw)", "sign(-2vw)"),
        ("sign(1em - 1px)", "sign(1em - 1px)"),
        ("sign(abs(-2em))", "sign(2em)"),
        ("sign(-2px)", "calc(-1)"),
    ] {
        let value = CssSpecifiedNumber::try_from_calculation(
            CssNumberCalculation::try_from_components(parse_component_values(source).unwrap())
                .unwrap(),
        )
        .unwrap();
        let before = value.clone();
        let result = value.serialize_specified();
        assert_eq!(value, before);
        assert_eq!(value.origin(), before.origin());
        assert_eq!(
            value.calculation().unwrap().components(),
            before.calculation().unwrap().components()
        );
        assert_eq!(result.unwrap(), expected, "{source}");
    }
}

#[test]
fn percentage_abs_distinguishes_raw_coefficients_from_unknown_signed_bases() {
    let source = "abs(-2%)";
    let components = parse_component_values(source).unwrap();
    let raw = CssPercentageCalculation::try_from_components(components.clone()).unwrap();
    assert_eq!(raw.numeric_type().percent_hint(), None);
    let raw = CssSpecifiedPercentage::try_from_calculation(raw).unwrap();
    let raw_before = raw.clone();
    assert_eq!(raw.serialize_specified().unwrap(), "calc(2%)");
    assert_eq!(raw, raw_before);
    assert_eq!(raw.calculation().unwrap().components(), &components);

    let hinted = CssLengthPercentageCalculation::try_from_components(components.clone()).unwrap();
    assert_eq!(
        hinted.numeric_type().percent_hint(),
        Some(CssNumericDimension::Length)
    );
    let hinted = CssSpecifiedLengthPercentage::try_from_calculation(hinted).unwrap();
    let hinted_before = hinted.clone();
    assert_eq!(hinted.serialize_specified().unwrap(), source);
    assert_eq!(hinted, hinted_before);
    assert_eq!(hinted.calculation().unwrap().components(), &components);
}

#[test]
fn programmatic_abs_keeps_exact_authored_coefficient_and_origin() {
    let arguments =
        CssComponentValues::try_new(vec![CssComponentValue::try_token("-02.0000000EM").unwrap()])
            .unwrap();
    let component = CssComponentValue::try_function("ABS", arguments).unwrap();
    let components = CssComponentValues::try_new(vec![component]).unwrap();
    let value = CssSpecifiedLength::try_from_calculation(
        CssLengthCalculation::try_from_components(components.clone()).unwrap(),
    )
    .unwrap();
    let before = value.clone();
    assert!(matches!(value.origin(), CssValueOrigin::Programmatic));
    let result = value.serialize_specified();
    assert_eq!(value, before);
    assert_eq!(value.origin(), before.origin());
    assert_eq!(value.calculation().unwrap().components(), &components);
    assert_eq!(result.unwrap(), "calc(2em)");
}

#[test]
fn contextual_abs_replacement_obeys_exact_limits_and_atomic_failures() {
    let value = length("abs(-2em)");
    let before = value.clone();
    let expected = "calc(2em)";
    // Input: Abs and its numeric leaf. Projection: leaf and Abs replacement.
    for (limits, kind) in [
        (Limits::new(1, 2, expected.len()), Kind::InputNodeLimit),
        (Limits::new(2, 1, expected.len()), Kind::ProjectionNodeLimit),
        (Limits::new(2, 2, expected.len() - 1), Kind::ByteLimit),
        (Limits::new(0, 0, 0), Kind::InputNodeLimit),
        (Limits::new(2, 0, 0), Kind::ProjectionNodeLimit),
    ] {
        let error = value.serialize_specified_with_limits(limits).unwrap_err();
        assert_eq!(error.kind(), kind);
        assert_eq!(value, before);
        assert_eq!(value.origin(), before.origin());
        assert_eq!(
            value.calculation().unwrap().components(),
            before.calculation().unwrap().components()
        );
    }
    let result = value.serialize_specified_with_limits(Limits::new(2, 2, expected.len()));
    assert_eq!(value, before);
    assert_eq!(result.unwrap(), expected);
}

#[test]
fn contextual_hypot_replacement_obeys_exact_limits_and_atomic_failures() {
    let value = length("hypot(3em,4em)");
    let before = value.clone();
    let expected = "calc(5em)";
    // Input: Hypot and two numeric leaves. Projection: leaves and replacement.
    for (limits, kind) in [
        (Limits::new(2, 3, expected.len()), Kind::InputNodeLimit),
        (Limits::new(3, 2, expected.len()), Kind::ProjectionNodeLimit),
        (Limits::new(3, 3, expected.len() - 1), Kind::ByteLimit),
    ] {
        let error = value.serialize_specified_with_limits(limits).unwrap_err();
        assert_eq!(error.kind(), kind);
        assert_eq!(value, before);
        assert_eq!(value.origin(), before.origin());
        assert_eq!(
            value.calculation().unwrap().components(),
            before.calculation().unwrap().components()
        );
    }
    let result = value.serialize_specified_with_limits(Limits::new(3, 3, expected.len()));
    assert_eq!(value, before);
    assert_eq!(result.unwrap(), expected);
}

#[test]
fn existing_numeric_color_consumption_censors_only_after_nested_arithmetic() {
    for (source, expected) in [
        ("rgb(calc(NaN) 0 0)", "rgb(0, 0, 0)"),
        ("rgb(calc(0 * -1) 0 0)", "rgb(0, 0, 0)"),
        ("rgb(calc(1 / calc(0 * -1)) 0 0)", "rgb(0, 0, 0)"),
        ("rgb(0 0 0 / calc(NaN))", "rgba(0, 0, 0, 0)"),
        ("rgb(0 0 0 / calc(1 / calc(0 * -1)))", "rgba(0, 0, 0, 0)"),
        ("rgb(0 0 0 / calc(1 / (0 * 1)))", "rgb(0, 0, 0)"),
    ] {
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
            panic!("color")
        };
        let result = value.value().to_specified_css();
        assert_eq!(declaration, before);
        assert_eq!(declaration.value_components(), &components);
        assert_eq!(result.unwrap(), expected, "{source}");
    }
    // Generic specified-value serialization still retains the raw tree result.
    for (source, expected) in [
        ("calc(0 / 0)", "calc(NaN)"),
        ("calc(1 / calc(0 * -1))", "calc(-infinity)"),
    ] {
        let value = CssSpecifiedNumber::try_from_calculation(
            CssNumberCalculation::try_from_components(parse_component_values(source).unwrap())
                .unwrap(),
        )
        .unwrap();
        let before = value.clone();
        let result = value.serialize_specified();
        assert_eq!(value, before);
        assert_eq!(result.unwrap(), expected);
    }
}
