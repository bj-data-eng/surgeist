#![forbid(unsafe_code)]

//! Pure length consumers re-admit authored math in their own percentage context.
//! Values 4 numeric typing permits percentage units to cancel without a basis.

use surgeist_css::{
    CssCalcLength, CssCalcLengthTerm, CssComponentValues, CssLength,
    CssLengthPercentageCalculation, CssNonNegativeLength, CssNumericDimension, CssTransformLength,
    CssValueOrigin, parse_component_values, parse_style_attribute,
};

fn mixed_calculation(source: &str) -> (CssLength, CssComponentValues) {
    let components = parse_component_values(source).expect("checked numeric components");
    let calculation = CssLengthPercentageCalculation::try_from_components(components.clone())
        .expect("valid length-percentage expression");
    (
        CssLength::Calc(CssCalcLength::Typed(calculation)),
        components,
    )
}

fn assert_pure_tree_preserves_components(
    calculation: &CssLengthPercentageCalculation,
    expected: &CssComponentValues,
) {
    assert_eq!(calculation.components(), expected);
    assert_eq!(calculation.numeric_type().percent_hint(), None);
    assert_eq!(
        calculation
            .numeric_type()
            .exponent(CssNumericDimension::Length),
        1
    );
    assert_eq!(
        calculation
            .numeric_type()
            .exponent(CssNumericDimension::Percentage),
        0
    );
    let (CssValueOrigin::Parsed(actual), CssValueOrigin::Parsed(original)) = (
        calculation.components().items()[0].origin(),
        expected.items()[0].origin(),
    ) else {
        panic!("original parsed function origin must be retained");
    };
    assert!(actual.source().same_snapshot(original.source()));
}

#[test]
fn pure_constructor_readmits_percentage_cancellation_and_preserves_original_components() {
    let (value, components) = mixed_calculation("calc(10% / 10% * 1px)");
    let pure = CssTransformLength::try_new(value).expect("percentages cancel in a pure context");
    let CssLength::Calc(CssCalcLength::Typed(calculation)) = pure.value() else {
        panic!("expected retained exact calculation");
    };
    assert_pure_tree_preserves_components(calculation, &components);
}

#[test]
fn pure_constructor_readmits_typed_children_inside_a_public_sum() {
    let (value, components) = mixed_calculation("calc(10% / 10% * 1px)");
    let CssLength::Calc(child) = value else {
        panic!("expected calculation")
    };
    let sum = CssCalcLength::sum(CssCalcLengthTerm::add(child), []);
    let pure = CssTransformLength::try_new(CssLength::Calc(sum))
        .expect("a sum retains valid pure-length cancellation in its exact child");
    let CssLength::Calc(CssCalcLength::Sum(terms)) = pure.value() else {
        panic!("expected retained authored sum");
    };
    let CssCalcLength::Typed(calculation) = terms[0].value() else {
        panic!("expected retained exact child");
    };
    assert_pure_tree_preserves_components(calculation, &components);
}

#[test]
fn pure_consumers_reject_uncancelled_percentage_context_even_inside_a_sum() {
    for source in ["calc(1px + 10%)", "min(1px, 10%)", "calc(10%)"] {
        let (value, _) = mixed_calculation(source);
        assert!(
            CssTransformLength::try_new(value.clone()).is_none(),
            "{source}"
        );
        assert!(
            CssNonNegativeLength::try_new(value.clone()).is_none(),
            "{source}"
        );
        let CssLength::Calc(child) = value else {
            panic!("expected calculation")
        };
        let sum = CssCalcLength::sum(CssCalcLengthTerm::add(child), []);
        assert!(
            CssTransformLength::try_new(CssLength::Calc(sum)).is_none(),
            "{source}"
        );
    }
}

#[test]
fn parsed_border_shadow_and_transform_accept_pure_percentage_cancellation() {
    for declaration in [
        "border-width: calc(10% / 10% * 1px)",
        "border: calc(10% / 10% * 1px) solid red",
        "box-shadow: calc(10% / 10% * 1px) 0px",
        "filter: drop-shadow(calc(10% / 10% * 1px) 0px)",
        "transform: translateZ(calc(10% / 10% * 1px))",
        "transform: perspective(calc(10% / 10% * 1px))",
    ] {
        let report = parse_style_attribute(declaration);
        assert!(
            report.is_clean(),
            "{declaration}: {:?}",
            report.diagnostics()
        );
        assert_eq!(report.syntax().len(), 1, "{declaration}");
    }
}

#[test]
fn parsed_pure_consumers_reject_length_percentage_results() {
    for declaration in [
        "border-width: calc(1px + 10%)",
        "box-shadow: calc(1px + 10%) 0px",
        "filter: drop-shadow(calc(1px + 10%) 0px)",
        "transform: translateZ(calc(1px + 10%))",
        "transform: perspective(calc(1px + 10%))",
    ] {
        let report = parse_style_attribute(declaration);
        assert!(!report.is_clean(), "{declaration}");
        assert!(report.syntax().is_empty(), "{declaration}");
    }
}
