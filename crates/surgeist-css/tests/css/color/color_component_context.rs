//! Pure color context is preserved when checked numeric values are composed.
//!
//! The selected Values 5 import admits tree-counting in container size queries.
//! Reusing their number type must not bypass the pure color expression context.

use surgeist_color::PredefinedColorSpace;
use surgeist_css::{
    CssColorComponent, CssColorConstructionError, CssColorHue, CssColorNumberLiteral,
    CssColorProfileName, CssColorSyntax, CssComponentValue, CssContainerCondition,
    CssContainerConditionKind, CssContainerFeatureQuery, CssContainerRatioRef, CssCustomColor,
    CssDeviceCmykColor, CssHslColor, CssHwbColor, CssLabColor, CssLchColor, CssMediaRangeRef,
    CssNumberCalculation, CssNumericConstructionErrorKind, CssPredefinedColor, CssRgbColor,
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
        assert_eq!(
            CssNumberCalculation::try_from_components(parse_component_values(expression).unwrap())
                .unwrap_err()
                .kind(),
            &CssNumericConstructionErrorKind::UnknownFunction,
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
        // Container admission alone does not make a pure calculation contextual.
        let calculation = container_number(expression);
        let origin = calculation.origin().clone();
        let rgb = rgb_with_calculation(calculation)
            .expect("pure calculations may be reused without their container context");
        let CssColorComponent::NumberCalculation(retained) = &rgb.channels()[0] else {
            panic!("checked calculation remains authored");
        };
        assert_eq!(retained.origin(), &origin);
    }
}

#[test]
fn color_component_and_hue_boundaries_reject_container_only_calculations() {
    let calculation = container_number("calc(max(1, sibling-index()) + sibling-count())");
    let component = || CssColorComponent::NumberCalculation(calculation.clone());
    let hue = || CssColorHue::NumberCalculation(calculation.clone());
    let zero = || {
        CssColorComponent::Number(
            CssColorNumberLiteral::try_from_component(CssComponentValue::try_number("0").unwrap())
                .unwrap(),
        )
    };
    let zero_hue = || {
        CssColorHue::Number(
            CssColorNumberLiteral::try_from_component(CssComponentValue::try_number("0").unwrap())
                .unwrap(),
        )
    };
    let mut accepted = Vec::new();
    let mut record = |family, accepted_value| {
        if accepted_value {
            accepted.push(family);
        }
    };
    record(
        "RGB alpha",
        CssRgbColor::try_new(
            CssColorSyntax::Modern,
            [zero(), zero(), zero()],
            Some(component()),
        )
        .is_ok(),
    );
    record(
        "HSL hue",
        CssHslColor::try_new(CssColorSyntax::Modern, hue(), zero(), zero(), None).is_ok(),
    );
    record(
        "HWB alpha",
        CssHwbColor::try_new(zero_hue(), zero(), zero(), Some(component())).is_ok(),
    );
    record(
        "Lab channel",
        CssLabColor::try_new(zero(), component(), zero(), None).is_ok(),
    );
    record(
        "LCH hue",
        CssLchColor::try_new(zero(), zero(), hue(), None).is_ok(),
    );
    record(
        "predefined channel",
        CssPredefinedColor::try_new(
            PredefinedColorSpace::Srgb,
            [component(), zero(), zero()],
            None,
        )
        .is_ok(),
    );
    record(
        "custom alpha",
        CssCustomColor::try_new(
            CssColorProfileName::try_new("--profile").unwrap(),
            vec![zero()],
            Some(component()),
        )
        .is_ok(),
    );
    record(
        "device CMYK channel",
        CssDeviceCmykColor::try_new(
            CssColorSyntax::Legacy,
            [component(), zero(), zero(), zero()],
            None,
        )
        .is_ok(),
    );
    assert!(
        accepted.is_empty(),
        "color boundaries admitted a container-only expression: {accepted:?}"
    );
}
