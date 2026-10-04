#![forbid(unsafe_code)]
//! Values 4 §10.10 Sum step 8.2 replaces each same-unit set with ONE numeric
//! value. Supported Angle range conversion belongs to that replacement, not
//! each binary64 accumulator prefix. Grouped Sums materialize separately.
//! https://www.w3.org/TR/2024/WD-css-values-4-20240312/#calc-simplification
//! https://www.w3.org/TR/2024/WD-css-values-4-20240312/#numeric-types
//! Source-order accumulation retains the selected binary64 policy. The flat
//! finite-input overflow converts once to the supported multiple endpoint E;
//! E/binary64(1e308) rounds to 1.797693. A separately materialized E followed by
//! subtraction of binary64(1e308) instead rounds to a quotient of 0.797693.
//! Genuine infinite operands and NaN retain the §10.9.1 exception semantics.

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
fn flat_same_unit_angle_sum_converts_only_its_merged_replacement() {
    for (source, expected) in [
        (
            "calc((1e308deg + 1e308deg - 1e308deg) / 1e308deg)",
            "calc(1.797693)",
        ),
        (
            "calc((-1e308deg - 1e308deg + 1e308deg) / 1e308deg)",
            "calc(-1.797693)",
        ),
    ] {
        assert_number(source, expected);
    }
}

#[test]
fn grouped_angle_replacements_and_finite_cancellation_remain_distinct() {
    for (source, expected) in [
        (
            "calc(((1e308deg + 1e308deg) - 1e308deg) / 1e308deg)",
            "calc(0.797693)",
        ),
        (
            "calc(((-1e308deg - 1e308deg) + 1e308deg) / 1e308deg)",
            "calc(-0.797693)",
        ),
        (
            "calc((1e308deg - 1e308deg + 1e308deg) / 1e308deg)",
            "calc(1)",
        ),
        (
            "calc((-1e308deg + 1e308deg - 1e308deg) / 1e308deg)",
            "calc(-1)",
        ),
        ("calc((1e308deg - 1e308deg) / 1e308deg)", "calc(0)"),
    ] {
        assert_number(source, expected);
    }
}

#[test]
fn merged_angle_sum_keeps_genuine_infinity_and_nan_exceptions() {
    for (source, expected) in [
        (
            "calc((infinity * 1deg + 1e308deg - 1e308deg) / 1e308deg)",
            "calc(infinity)",
        ),
        (
            "calc((-infinity * 1deg - 1e308deg + 1e308deg) / 1e308deg)",
            "calc(-infinity)",
        ),
        (
            "calc((infinity * 1deg - infinity * 1deg + 1e308deg) / 1e308deg)",
            "calc(NaN)",
        ),
        (
            "calc((NaN * 1deg + 1e308deg - 1e308deg) / 1e308deg)",
            "calc(NaN)",
        ),
    ] {
        assert_number(source, expected);
    }
}

#[test]
fn programmatic_angle_sum_keeps_exact_coefficients_and_origin() {
    let arguments =
        parse_component_values("(+01e308DEG + +01e308DEG - +01e308DEG) / +01e308DEG").unwrap();
    let root = CssComponentValue::try_function("CALC", arguments).unwrap();
    let components = CssComponentValues::try_new(vec![root]).unwrap();
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
    assert_eq!(result.unwrap(), "calc(1.797693)");
}

#[test]
fn merged_angle_replacement_obeys_exact_limits_and_atomic_failures() {
    for (source, expected) in [
        (
            "calc((1e308deg + 1e308deg - 1e308deg) / 1e308deg)",
            "calc(1.797693)",
        ),
        (
            "calc((-1e308deg - 1e308deg + 1e308deg) / 1e308deg)",
            "calc(-1.797693)",
        ),
    ] {
        let value = number(source);
        let before = value.clone();
        // Input: Calc/Product/Group/Sum and four leaves. Projection: four
        // leaves, one subtract Negate, merged Sum scalar, Invert and Product.
        for (limits, kind) in [
            (Limits::new(7, 8, expected.len()), Kind::InputNodeLimit),
            (Limits::new(8, 7, expected.len()), Kind::ProjectionNodeLimit),
            (Limits::new(8, 8, expected.len() - 1), Kind::ByteLimit),
        ] {
            let error = value.serialize_specified_with_limits(limits).unwrap_err();
            assert_eq!(error.kind(), kind, "{source}: {limits:?}");
            assert_eq!(value, before);
            assert_eq!(value.origin(), before.origin());
            assert_eq!(
                value.calculation().unwrap().components(),
                before.calculation().unwrap().components()
            );
        }
        let result = value.serialize_specified_with_limits(Limits::new(8, 8, expected.len()));
        assert_eq!(value, before);
        assert_eq!(result.unwrap(), expected);
    }
}

#[test]
fn sibling_angle_replacements_share_one_cumulative_budget() {
    let value = number(
        "calc(calc((1e308deg + 1e308deg - 1e308deg) / 1e308deg) + calc((1e308deg + 1e308deg - 1e308deg) / 1e308deg))",
    );
    let before = value.clone();
    let expected = "calc(3.595386)";
    // Two eight-input/eight-projection children; outer Calc/Sum costs two
    // inputs and one combined Number replacement projection.
    for (limits, kind) in [
        (Limits::new(17, 17, expected.len()), Kind::InputNodeLimit),
        (
            Limits::new(18, 16, expected.len()),
            Kind::ProjectionNodeLimit,
        ),
        (Limits::new(18, 17, expected.len() - 1), Kind::ByteLimit),
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
    let result = value.serialize_specified_with_limits(Limits::new(18, 17, expected.len()));
    assert_eq!(value, before);
    assert_eq!(result.unwrap(), expected);
}
