#![forbid(unsafe_code)]
//! Values 4 §5 distinguishes finite values outside the supported numeric range
//! from §10.9.1's defined infinities. An unrepresentable finite angle converts
//! to the nearest supported multiple of 360deg; sin/tan of that angle is zero
//! and cos is one. Division by zero and specified function exceptions instead
//! produce genuine infinity, whose trigonometric result remains NaN (§10.4.1).
//! https://www.w3.org/TR/2024/WD-css-values-4-20240312/#numeric-types
//! https://www.w3.org/TR/2024/WD-css-values-4-20240312/#calc-ieee
//! https://www.w3.org/TR/2024/WD-css-values-4-20240312/#trig-infinities
//! https://www.w3.org/TR/2024/WD-css-values-4-20240312/#round-infinities
//! These are representation conversions during math projection, distinct from
//! target-context computed/used range clamping under §10.12.

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
    let result = value.serialize_specified();
    assert_eq!(value, before);
    assert_eq!(value.origin(), before.origin());
    assert_eq!(value.calculation().unwrap().components(), &components);
    assert_eq!(value.calculation().unwrap().numeric_type(), ty);
    assert_eq!(result.unwrap(), expected, "{source}");
}

#[test]
fn overflowing_grad_coefficients_can_have_supported_canonical_degree_values() {
    // The exact factor 9/10 brings these finite coefficients back into the
    // canonical degree range. Coefficient parse overflow must not clamp them.
    for (source, expected) in [
        ("calc(1.8e308grad / 1e308deg)", "calc(1.62)"),
        ("calc(-1.8e308grad / 1e308deg)", "calc(-1.62)"),
        ("calc(1.9e308grad / 1e308deg)", "calc(1.71)"),
        ("calc(1.99e308grad / 1e308deg)", "calc(1.791)"),
    ] {
        assert_number(source, expected);
    }
}

#[test]
fn grad_conversion_clamps_only_when_the_canonical_angle_exceeds_the_range() {
    for (source, expected) in [
        ("cos(2e308grad)", "calc(1)"),
        ("cos(-2e308grad)", "calc(1)"),
        ("sin(2e308grad)", "calc(0)"),
        ("tan(-2e308grad)", "calc(0)"),
    ] {
        assert_number(source, expected);
    }
}

#[test]
fn representable_angles_near_the_maximum_are_not_capped_to_the_multiple_endpoint() {
    // At the maximum binary64 exponent, one ULP is exactly 2^971. The largest
    // finite multiple of 360 is MAX - 31 ULPs: (2^53-1) mod 45 == 31.
    // Inputs are round-tripping spellings of these independently chosen values.
    // Expected differences are k*2^971 / binary64(1e293), rounded for CSS text.
    let endpoint = f64::from_bits(f64::MAX.to_bits() - 31).to_string();
    for (bits, expected) in [
        (f64::MAX.to_bits() - 32, "calc(-0.199584)"),
        (f64::MAX.to_bits() - 30, "calc(0.199584)"),
        (f64::MAX.to_bits(), "calc(6.187105)"),
    ] {
        let coefficient = f64::from_bits(bits).to_string();
        assert_number(
            &format!("calc(({coefficient}deg - {endpoint}deg) / 1e293deg)"),
            expected,
        );
    }
}

#[test]
fn finite_pure_angle_sum_overflow_converts_before_trigonometric_evaluation() {
    for (source, expected) in [
        ("cos(1e308deg + 1e308deg)", "calc(1)"),
        ("cos(-1e308deg - 1e308deg)", "calc(1)"),
        ("sin(1e308deg + 1e308deg)", "calc(0)"),
        ("tan(-1e308deg - 1e308deg)", "calc(0)"),
    ] {
        assert_number(source, expected);
    }
}

#[test]
fn finite_pure_angle_product_overflow_converts_before_trigonometric_evaluation() {
    for (source, expected) in [
        ("cos(2 * 1e308deg)", "calc(1)"),
        ("cos(-2 * 1e308deg)", "calc(1)"),
        ("sin(2 * 1e308deg)", "calc(0)"),
        ("tan(-2 * 1e308deg)", "calc(0)"),
    ] {
        assert_number(source, expected);
    }
}

#[test]
fn division_and_function_exception_infinities_keep_their_math_meaning() {
    for source in [
        "cos(1deg / 0)",
        "cos(1deg / (0 * -1))",
        "cos(pow(0,-1) * 1deg)",
        "cos(round(up,1deg,infinity * 1deg))",
        "cos(round(down,-1deg,infinity * 1deg))",
        "cos(hypot(infinity * 1deg,1deg))",
        "cos(infinity * 1deg + 1e308deg)",
        "cos(-infinity * 1deg - 1e308deg)",
    ] {
        assert_number(source, "calc(NaN)");
    }
}

#[test]
fn finite_angle_arithmetic_replacements_obey_exact_limits_and_atomic_failures() {
    let value = number("cos(2 * 1e308deg)");
    let before = value.clone();
    let expected = "calc(1)";
    // Inputs: Cos, Product and two leaves. Projections: both leaves, the
    // Product replacement and Cos replacement. Range conversion adds no node.
    for (limits, kind) in [
        (Limits::new(3, 4, expected.len()), Kind::InputNodeLimit),
        (Limits::new(4, 3, expected.len()), Kind::ProjectionNodeLimit),
        (Limits::new(4, 4, expected.len() - 1), Kind::ByteLimit),
    ] {
        let error = value.serialize_specified_with_limits(limits).unwrap_err();
        assert_eq!(error.kind(), kind, "{limits:?}");
        assert_eq!(value, before);
        assert_eq!(value.origin(), before.origin());
        assert_eq!(
            value.calculation().unwrap().components(),
            before.calculation().unwrap().components()
        );
    }
    let result = value.serialize_specified_with_limits(Limits::new(4, 4, expected.len()));
    assert_eq!(value, before);
    assert_eq!(result.unwrap(), expected);
}
