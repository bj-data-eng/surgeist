//! Values 4 §10.10 Sum step 8.2 replaces each same-unit set with ONE numeric
//! value. Each finite source-order accumulation step is itself a value that
//! must be supported, so Angle range conversion applies at every overflowing
//! step (#1297 review), and grouping no longer changes the result.
//! https://www.w3.org/TR/2024/WD-css-values-4-20240312/#calc-simplification
//! https://www.w3.org/TR/2024/WD-css-values-4-20240312/#numeric-types
//! Source-order accumulation retains the selected binary64 policy. The first
//! finite-input overflow converts to the supported multiple endpoint E; E minus
//! binary64(1e308), divided by binary64(1e308), rounds to 0.797693, flat or grouped.
//! Genuine infinite operands and NaN retain the §10.9.1 exception semantics.

use crate::css::support::failure_classes::SerializationFailure;
use crate::css::support::failure_classes::serialization_failure;
use crate::css::support::value_assertions::assert_calculated_number as assert_number;
use crate::css::support::value_assertions::calculated_number as number;
use surgeist_css::CssParser;
use surgeist_css::component::{CssComponentValue, CssComponentValues, CssValueOrigin};
use surgeist_css::numeric::{CssCalculation, CssNumber, CssNumeric};
use surgeist_css::serialization::CssSerializationLimits;

type Limits = CssSerializationLimits;
type Kind = SerializationFailure;

#[test]
fn flat_same_unit_angle_sum_converts_at_each_accumulation_step() {
    for (source, expected) in [
        (
            "calc((1e308deg + 1e308deg - 1e308deg) / 1e308deg)",
            "calc(0.797693)",
        ),
        (
            "calc((-1e308deg - 1e308deg + 1e308deg) / 1e308deg)",
            "calc(-0.797693)",
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
    let arguments = CssComponentValues::parse(
        &CssParser::default(),
        "(+01e308DEG + +01e308DEG - +01e308DEG) / +01e308DEG",
    )
    .unwrap();
    let root = CssComponentValue::try_function("CALC", arguments).unwrap();
    let components = CssComponentValues::try_new(vec![root], Default::default()).unwrap();
    let value = CssNumeric::<CssNumber>::try_from_calculation(
        CssCalculation::<CssNumber>::try_from_components(components.clone(), Default::default())
            .unwrap(),
    )
    .unwrap();
    let before = value.clone();
    assert!(matches!(value.origin(), CssValueOrigin::Programmatic));
    let result = value.serialize_specified(Default::default());
    assert_eq!(value, before);
    assert_eq!(value.origin(), before.origin());
    assert_eq!(value.calculation().unwrap().components(), &components);
    assert_eq!(result.unwrap(), "calc(0.797693)");
}

#[test]
fn merged_angle_replacement_obeys_exact_limits_and_atomic_failures() {
    for (source, expected) in [
        (
            "calc((1e308deg + 1e308deg - 1e308deg) / 1e308deg)",
            "calc(0.797693)",
        ),
        (
            "calc((-1e308deg - 1e308deg + 1e308deg) / 1e308deg)",
            "calc(-0.797693)",
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
            let error = value.serialize_specified(limits).unwrap_err();
            assert_eq!(serialization_failure(&error), kind, "{source}: {limits:?}");
            assert_eq!(value, before);
            assert_eq!(value.origin(), before.origin());
            assert_eq!(
                value.calculation().unwrap().components(),
                before.calculation().unwrap().components()
            );
        }
        let result = value.serialize_specified(Limits::new(8, 8, expected.len()));
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
    let expected = "calc(1.595386)";
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
        let error = value.serialize_specified(limits).unwrap_err();
        assert_eq!(serialization_failure(&error), kind, "{limits:?}");
        assert_eq!(value, before);
        assert_eq!(value.origin(), before.origin());
        assert_eq!(
            value.calculation().unwrap().components(),
            before.calculation().unwrap().components()
        );
    }
    let result = value.serialize_specified(Limits::new(18, 17, expected.len()));
    assert_eq!(value, before);
    assert_eq!(result.unwrap(), expected);
}
