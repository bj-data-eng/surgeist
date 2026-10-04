#![forbid(unsafe_code)]
//! Values 4 WD 2024-03-12 §10.10.1 steps 4–5 distinguish comparable
//! magnitudes from coefficients whose percentage basis is still unknown:
//! https://www.w3.org/TR/2024/WD-css-values-4-20240312/#calc-simplification
//! A negative basis reverses percentage ordering; a zero basis makes all
//! finite percentage magnitudes zero. Specified serialization therefore keeps
//! hinted percentage comparisons, while same-unit sums may still combine.
//! Frozen WebKit 73aa6c89e2cb77c46184a81aec944e4ab99d114d independently
//! distinguishes these cases in CSSCalcTree+Simplification.cpp predicates
//! magnitudeComparable and fullyResolved. Same-unit contextual dimensions
//! remain comparable coefficients without resolving their actual magnitudes.

use surgeist_css::*;

type Limits = CssSpecifiedValueSerializationLimits;
type Kind = CssSpecifiedValueSerializationErrorKind;

fn length_percentage(source: &str) -> CssSpecifiedLengthPercentage {
    let components = parse_component_values(source).unwrap();
    let calculation = CssLengthPercentageCalculation::try_from_components(components).unwrap();
    CssSpecifiedLengthPercentage::try_from_calculation(calculation).unwrap()
}

fn assert_length_percentage(source: &str, expected: &str) {
    let value = length_percentage(source);
    let before = value.clone();
    let calculation = value.calculation().unwrap();
    let components = calculation.components().clone();
    let origin = value.origin().clone();
    assert_eq!(components.serialize().unwrap().as_css(), source);
    let result = value.serialize_specified();
    assert_eq!(value, before);
    assert_eq!(value.origin(), &origin);
    assert_eq!(value.calculation().unwrap().components(), &components);
    assert_eq!(result.unwrap(), expected, "{source}");
}

#[test]
fn hinted_whole_comparisons_keep_positive_negative_and_zero_coefficients_symbolic() {
    for (source, expected) in [
        ("min(1%,2%)", "min(1%, 2%)"),
        ("max(1%,2%)", "max(1%, 2%)"),
        ("min(-1%,-2%)", "min(-1%, -2%)"),
        ("max(-1%,-2%)", "max(-1%, -2%)"),
        ("min(0%,1%)", "min(0%, 1%)"),
        ("max(0%,-1%)", "max(0%, -1%)"),
        ("clamp(1%,2%,3%)", "clamp(1%, 2%, 3%)"),
        ("clamp(3%,2%,1%)", "clamp(3%, 2%, 1%)"),
        ("clamp(none,2%,3%)", "clamp(none, 2%, 3%)"),
        ("clamp(1%,2%,none)", "clamp(1%, 2%, none)"),
    ] {
        let value = length_percentage(source);
        assert_eq!(
            value.calculation().unwrap().numeric_type().percent_hint(),
            Some(CssNumericDimension::Length)
        );
        assert_length_percentage(source, expected);
    }
}

#[test]
fn partial_comparisons_keep_each_percentage_and_the_first_comparable_group_position() {
    for (source, expected) in [
        ("min(1%,2px,2%,1px)", "min(1%, 1px, 2%)"),
        ("max(1%,2px,2%,1px)", "max(1%, 2px, 2%)"),
        ("min(2px,1%,1px,2%)", "min(1px, 1%, 2%)"),
        ("max(2px,1%,1px,2%)", "max(2px, 1%, 2%)"),
        (
            "min(2em,1%,1em,2%,3px)",
            "min(1em, 1%, 2%, 3px)",
        ),
        (
            "max(1em,1%,2em,2%,3px)",
            "max(2em, 1%, 2%, 3px)",
        ),
    ] {
        assert_length_percentage(source, expected);
    }
}

#[test]
fn nested_comparisons_and_same_unit_sums_do_not_supply_a_percentage_basis() {
    for (source, expected) in [
        ("max(1px,min(1%,2%))", "max(1px, min(1%, 2%))"),
        ("min(1% + 2%,4%)", "min(3%, 4%)"),
        ("max(1% - 1%,2%)", "max(0%, 2%)"),
        ("clamp(1%,2% + 3%,6%)", "clamp(1%, 5%, 6%)"),
    ] {
        assert_length_percentage(source, expected);
    }
}

#[test]
fn raw_percentage_comparisons_still_fold_without_a_dimension_hint() {
    for (source, expected) in [
        ("min(1%,2%)", "calc(1%)"),
        ("max(1%,2%)", "calc(2%)"),
        ("min(-1%,-2%)", "calc(-2%)"),
        ("max(-1%,-2%)", "calc(-1%)"),
        ("clamp(3%,2%,1%)", "calc(3%)"),
        ("clamp(none,2%,3%)", "calc(2%)"),
        ("clamp(1%,2%,none)", "calc(2%)"),
    ] {
        let components = parse_component_values(source).unwrap();
        let calculation =
            CssPercentageCalculation::try_from_components(components.clone()).unwrap();
        assert_eq!(calculation.numeric_type().percent_hint(), None);
        let value = CssSpecifiedPercentage::try_from_calculation(calculation).unwrap();
        let before = value.clone();
        let result = value.serialize_specified();
        assert_eq!(value, before);
        assert_eq!(value.calculation().unwrap().components(), &components);
        assert_eq!(value.origin(), before.origin());
        assert_eq!(result.unwrap(), expected, "{source}");
    }
}

#[test]
fn same_unit_contextual_dimensions_remain_comparable_and_unbounded_clamp_keeps_its_value() {
    for (source, expected) in [
        ("min(1em,2em)", "calc(1em)"),
        ("max(1em,2em)", "calc(2em)"),
        ("clamp(3em,2em,1em)", "calc(3em)"),
        ("min(1vw,2vw)", "calc(1vw)"),
        ("max(1vw,2vw)", "calc(2vw)"),
        ("clamp(none,2%,none)", "calc(2%)"),
    ] {
        assert_length_percentage(source, expected);
    }
}

#[test]
fn programmatic_comparison_keeps_exact_coefficients_and_programmatic_origin() {
    let arguments = CssComponentValues::try_new(vec![
        CssComponentValue::try_token("+01.0000000%").unwrap(),
        CssComponentValue::try_token(",").unwrap(),
        CssComponentValue::try_token("2e0%").unwrap(),
    ])
    .unwrap();
    let root = CssComponentValue::try_function("MAX", arguments).unwrap();
    let components = CssComponentValues::try_new(vec![root]).unwrap();
    let calculation =
        CssLengthPercentageCalculation::try_from_components(components.clone()).unwrap();
    let value = CssSpecifiedLengthPercentage::try_from_calculation(calculation).unwrap();
    let before = value.clone();
    assert!(matches!(value.origin(), CssValueOrigin::Programmatic));
    let result = value.serialize_specified();
    assert_eq!(value, before);
    assert_eq!(value.calculation().unwrap().components(), &components);
    assert_eq!(value.origin(), before.origin());
    assert_eq!(result.unwrap(), "max(1%, 2%)");
}

fn assert_numeric_limits(source: &str, expected: &str, inputs: usize, projections: usize) {
    let value = length_percentage(source);
    let before = value.clone();
    for (limits, kind) in [
        (
            Limits::new(inputs - 1, projections, expected.len()),
            Kind::InputNodeLimit,
        ),
        (
            Limits::new(inputs, projections - 1, expected.len()),
            Kind::ProjectionNodeLimit,
        ),
        (
            Limits::new(inputs, projections, expected.len() - 1),
            Kind::ByteLimit,
        ),
        (Limits::new(0, 0, 0), Kind::InputNodeLimit),
        (Limits::new(inputs, 0, 0), Kind::ProjectionNodeLimit),
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
    let result = value.serialize_specified_with_limits(Limits::new(
        inputs,
        projections,
        expected.len(),
    ));
    assert_eq!(value, before);
    assert_eq!(result.unwrap(), expected);
}

#[test]
fn whole_hinted_comparisons_share_exact_input_projection_and_output_limits() {
    // Each comparison visits its function and numeric leaves, and emits one
    // projected function plus those leaves. Omitted clamp bounds are not nodes.
    assert_numeric_limits("min(1%,2%)", "min(1%, 2%)", 3, 3);
    assert_numeric_limits("max(1%,2%)", "max(1%, 2%)", 3, 3);
    assert_numeric_limits("clamp(1%,2%,3%)", "clamp(1%, 2%, 3%)", 4, 4);
    assert_numeric_limits("clamp(none,2%,3%)", "clamp(none, 2%, 3%)", 3, 3);
}

#[test]
fn partial_numeric_group_charges_its_replacement_without_merging_percentage_siblings() {
    // Function plus four leaves: five inputs. Four projected leaves, one
    // combined px replacement, and the retained comparison: six projections.
    assert_numeric_limits("min(1%,2px,2%,1px)", "min(1%, 1px, 2%)", 5, 6);
}

#[test]
fn sibling_hinted_comparisons_keep_one_cumulative_enclosing_budget() {
    let shape = CssInsetShape::new(
        CssInsetShapeOffsets::try_new(vec![
            length_percentage("min(1%,2%)"),
            length_percentage("max(3%,4%)"),
        ])
        .unwrap(),
        None,
    );
    let before = shape.clone();
    let expected = "inset(min(1%, 2%) max(3%, 4%))";
    // Shape and offset-list aggregates plus two three-node comparisons.
    for (limits, kind) in [
        (Limits::new(7, 8, expected.len()), Kind::InputNodeLimit),
        (Limits::new(8, 7, expected.len()), Kind::ProjectionNodeLimit),
        (Limits::new(8, 8, expected.len() - 1), Kind::ByteLimit),
    ] {
        let error = shape.serialize_specified_with_limits(limits).unwrap_err();
        assert_eq!(error.kind(), kind, "{limits:?}");
        assert_eq!(shape, before);
        for (value, original) in shape.offsets().values().iter().zip(before.offsets().values()) {
            assert_eq!(value.origin(), original.origin());
            assert_eq!(
                value.calculation().unwrap().components(),
                original.calculation().unwrap().components()
            );
        }
    }
    let result = shape.serialize_specified_with_limits(Limits::new(8, 8, expected.len()));
    assert_eq!(shape, before);
    assert_eq!(result.unwrap(), expected);
}
