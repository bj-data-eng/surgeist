#![forbid(unsafe_code)]
//! Pure length consumers re-admit authored math in their own percentage context.
//! Values 4 numeric typing permits percentage units to cancel without a basis.
use surgeist_css::*;

fn mixed_calculation(source: &str) -> (CssLengthPercentageCalculation, CssComponentValues) {
    let components = parse_component_values(source).unwrap();
    let calculation =
        CssLengthPercentageCalculation::try_from_components(components.clone()).unwrap();
    (calculation, components)
}
fn pure(
    calculation: CssLengthPercentageCalculation,
) -> Result<CssSpecifiedLength, CssNumericConstructionError> {
    CssSpecifiedLength::try_from_calculation(CssLengthCalculation::try_from_components(
        calculation.components().clone(),
    )?)
}
fn assert_pure_tree_preserves_components(
    calculation: &CssLengthCalculation,
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
        panic!("retained parsed origin")
    };
    assert!(actual.source().same_snapshot(original.source()));
    assert_eq!(actual.span(), original.span());
}
#[test]
fn pure_constructor_readmits_percentage_cancellation_and_preserves_original_components() {
    let (value, components) = mixed_calculation("calc(10% / 10% * 1px)");
    let value = pure(value).expect("percentages cancel in pure context");
    assert_pure_tree_preserves_components(value.calculation().unwrap(), &components);
}
#[test]
fn pure_constructor_readmits_typed_children_inside_a_public_sum() {
    let (child, components) = mixed_calculation("calc(10% / 10% * 1px)");
    let sum = CssLengthPercentageCalculation::try_sum(child, []).unwrap();
    let value = pure(sum).expect("checked sum retains pure cancellation");
    let calculation = value.calculation().unwrap();
    assert_eq!(calculation.origin(), &CssValueOrigin::Programmatic);
    assert_eq!(calculation.position(), None);
    assert_eq!(
        calculation.serialize().unwrap().as_css(),
        "calc(calc(10% / 10% * 1px))"
    );
    let CssComponentValueRef::Function(outer) = calculation.components().items()[0].view() else {
        panic!("outer calc")
    };
    assert_eq!(outer.values(), &components);
    assert_eq!(calculation.numeric_type().percent_hint(), None);
    assert_eq!(
        calculation
            .numeric_type()
            .exponent(CssNumericDimension::Length),
        1
    );
    let (CssValueOrigin::Parsed(actual), CssValueOrigin::Parsed(original)) = (
        outer.values().items()[0].origin(),
        components.items()[0].origin(),
    ) else {
        panic!("original child")
    };
    assert!(actual.source().same_snapshot(original.source()));
    assert_eq!(actual.span(), original.span());
}
#[test]
fn pure_consumers_reject_uncancelled_percentage_context_even_inside_a_sum() {
    for source in ["calc(1px + 10%)", "min(1px, 10%)", "calc(10%)"] {
        let (value, _) = mixed_calculation(source);
        assert!(pure(value.clone()).is_err(), "{source}");
        assert!(
            CssLengthCalculation::try_from_components(value.components().clone()).is_err(),
            "{source}"
        );
        let sum = CssLengthPercentageCalculation::try_sum(value, []).unwrap();
        assert!(pure(sum).is_err(), "{source}");
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
