#![forbid(unsafe_code)]

//! A percentage keeps its basis while scalar arithmetic distributes over a sum.
//! CSS Values 4 specifies simplification without resolving an unknown percentage basis:
//! https://www.w3.org/TR/2024/WD-css-values-4-20240312/#calc-simplification

use surgeist_css::{
    CssLengthPercentageCalculation, CssNumericDimension, CssSpecifiedNonNegativeLengthPercentage,
    parse_component_values,
};

fn specified(source: &str) -> String {
    let calculation = CssLengthPercentageCalculation::try_from_components(
        parse_component_values(source).unwrap(),
    )
    .unwrap();
    assert_eq!(
        calculation.numeric_type().percent_hint(),
        Some(CssNumericDimension::Length),
        "{source} must retain its percentage basis"
    );
    CssSpecifiedNonNegativeLengthPercentage::try_from_calculation(calculation)
        .unwrap()
        .serialize_specified()
        .unwrap()
}

#[test]
fn scalar_multiplication_preserves_distinct_length_and_percentage_terms() {
    assert_eq!(specified("calc((1px + 2%) * 3)"), "calc(6% + 3px)");
    assert_eq!(specified("calc((1em + 2%) * 3)"), "calc(6% + 3em)");
}

#[test]
fn scalar_division_preserves_distinct_length_and_percentage_terms() {
    assert_eq!(specified("calc((1px + 2%) / 2)"), "calc(1% + 0.5px)");
}

#[test]
fn percentage_only_sum_remains_percentage_after_scalar_multiplication() {
    assert_eq!(specified("calc((1% + 2%) * 3)"), "calc(9%)");
}

#[test]
fn same_unit_length_sum_can_collapse_before_scalar_multiplication() {
    let calculation = CssLengthPercentageCalculation::try_from_components(
        parse_component_values("calc((1px + 2px) * 3)").unwrap(),
    )
    .unwrap();
    assert_eq!(
        CssSpecifiedNonNegativeLengthPercentage::try_from_calculation(calculation)
            .unwrap()
            .serialize_specified()
            .unwrap(),
        "calc(9px)"
    );
}

#[test]
fn compatible_absolute_units_can_collapse_before_scalar_multiplication() {
    let calculation = CssLengthPercentageCalculation::try_from_components(
        parse_component_values("calc((1in + 2px) * 3)").unwrap(),
    )
    .unwrap();
    assert_eq!(
        CssSpecifiedNonNegativeLengthPercentage::try_from_calculation(calculation)
            .unwrap()
            .serialize_specified()
            .unwrap(),
        "calc(294px)"
    );
}
