#![forbid(unsafe_code)]
//! Pure color context is preserved when checked numeric values are composed.
//!
//! The selected Values 5 import admits tree-counting in container size queries.
//! Reusing their number type must not bypass the pure color expression context.

use surgeist_css::{
    CssColorComponent, CssColorConstructionError, CssColorNumberLiteral, CssColorSyntax,
    CssComponentValue, CssContainerCondition, CssContainerConditionKind, CssContainerFeatureQuery,
    CssContainerRatioRef, CssMediaRangeRef, CssNumberCalculation, CssRgbColor,
    parse_component_values,
};

fn container_number(expression: &str) -> CssNumberCalculation {
    let source = format!("(aspect-ratio: {expression})");
    let condition =
        CssContainerCondition::try_from_components(parse_component_values(&source).unwrap())
            .expect("tree-counting remains valid in its selected container context");
    let CssContainerConditionKind::Feature(CssContainerFeatureQuery::AspectRatio(range)) =
        condition.kind()
    else {
        panic!("expected an authored aspect-ratio feature: {source}");
    };
    let CssMediaRangeRef::Plain { value } = range.view() else {
        panic!("expected a plain ratio operand: {source}");
    };
    let CssContainerRatioRef::Numeric(ratio) = value.view() else {
        panic!("expected a typed numeric ratio: {source}");
    };
    ratio.numerator().clone()
}

fn rgb_with_calculation(
    calculation: CssNumberCalculation,
) -> Result<CssRgbColor, CssColorConstructionError> {
    let zero =
        CssColorNumberLiteral::try_from_component(CssComponentValue::try_number("0").unwrap())
            .unwrap();
    CssRgbColor::try_new(
        CssColorSyntax::Modern,
        [
            CssColorComponent::NumberCalculation(calculation),
            CssColorComponent::Number(zero.clone()),
            CssColorComponent::Number(zero),
        ],
        None,
    )
}

#[test]
fn rgb_constructor_rejects_container_tree_counting_calculations() {
    let mut accepted = Vec::new();
    for expression in [
        "sibling-count()",
        "sibling-index()",
        "calc(sibling-count() + 1)",
        "calc(sibling-index() + 1)",
        "max(1, sibling-count())",
        "max(1, sibling-index())",
    ] {
        let calculation = container_number(expression);
        assert!(
            CssNumberCalculation::try_from_components(parse_component_values(expression).unwrap())
                .is_err(),
            "the pure number boundary must retain its context restriction: {expression}"
        );
        if rgb_with_calculation(calculation).is_ok() {
            accepted.push(expression);
        }
    }
    assert!(
        accepted.is_empty(),
        "checked RGB admitted container-only calculations: {accepted:?}"
    );
}

#[test]
fn rgb_constructor_accepts_pure_number_calculations() {
    for expression in ["calc(1 + 2)", "max(1, 2)"] {
        let calculation =
            CssNumberCalculation::try_from_components(parse_component_values(expression).unwrap())
                .expect("a pure number calculation is valid outside container context");
        assert!(
            rgb_with_calculation(calculation).is_ok(),
            "pure calculation must remain valid in RGB: {expression}"
        );
    }
}
