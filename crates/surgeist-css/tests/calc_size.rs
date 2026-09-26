#![forbid(unsafe_code)]

use surgeist_css::{
    CssCalcSize, CssCalcSizeBasisRef, CssCalculationExpressionRef, CssComponentValueErrorKind,
    CssComponentValueLimits, CssIntrinsicSizeKeyword, CssLengthPercentageCalculation,
    CssNumericConstructionErrorKind, CssSpecifiedValueSerializationErrorKind,
    CssSpecifiedValueSerializationLimits, parse_component_values,
};

fn checked(text: &str) -> CssCalcSize {
    let values = parse_component_values(text).unwrap();
    let [component] = values.items() else {
        panic!("one function: {text}")
    };
    CssCalcSize::try_from_component(component.clone())
        .unwrap_or_else(|error| panic!("{text}: {error:?}"))
}

fn rejected(text: &str) {
    let values = parse_component_values(text).unwrap();
    let [component] = values.items() else {
        panic!("one function: {text}")
    };
    assert!(
        CssCalcSize::try_from_component(component.clone()).is_err(),
        "{text}"
    );
}

#[test]
fn basis_and_symbolic_size_are_borrowed_checked_views() {
    let value = checked("calc-size(min-content, size + 1px)");
    assert!(matches!(
        value.basis(),
        CssCalcSizeBasisRef::Keyword(CssIntrinsicSizeKeyword::MinContent)
    ));
    let CssCalculationExpressionRef::Sum(sum) = value.calculation() else {
        panic!("sum")
    };
    assert!(matches!(
        sum.term(0).unwrap().expression(),
        CssCalculationExpressionRef::Size(_)
    ));
    assert_eq!(
        value.serialize_specified().unwrap(),
        "calc-size(min-content, 1px + size)"
    );

    let nested = checked("calc-size(calc-size(auto, size), size + 1px)");
    let CssCalcSizeBasisRef::Nested(child) = nested.basis() else {
        panic!("nested basis")
    };
    assert!(matches!(
        child.basis(),
        CssCalcSizeBasisRef::Keyword(CssIntrinsicSizeKeyword::Auto)
    ));
    assert_eq!(
        nested.serialize_specified().unwrap(),
        "calc-size(calc-size(auto, size), 1px + size)"
    );
}

#[test]
fn any_basis_never_admits_size_even_when_algebra_would_erase_it() {
    for text in [
        "calc-size(any, size)",
        "calc-size(any, size * 0)",
        "calc-size(any, 0)",
    ] {
        rejected(text);
    }
    for text in [
        "calc-size(any, 0px)",
        "calc-size(any, 0%)",
        "calc-size(any, -1px)",
    ] {
        checked(text);
    }
}

#[test]
fn basis_and_calculation_require_length_percentage_algebra() {
    for text in [
        "calc-size(0, size)",
        "calc-size(size, 1px)",
        "calc-size(none, 1px)",
        "calc-size(fit-content(1px), size)",
        "calc-size(min-content, 1)",
        "calc-size(any, 1px / 1px)",
        "calc-size(any, 1px * 1px)",
        "calc-size(min-content, calc-size(min-content, size))",
    ] {
        rejected(text);
    }
    checked("calc-size(0px, size + 1%)");
    checked("calc-size(0%, size + 1px)");
}

#[test]
fn origins_retain_authored_nested_basis_site() {
    let value = checked("calc-size(calc-size(auto, size), 1px)");
    let CssCalcSizeBasisRef::Nested(child) = value.basis() else {
        panic!("nested")
    };
    assert_eq!(value.basis_origin(), child.origin());
    assert_ne!(child.basis_origin(), child.origin());
}

#[test]
fn specified_serialization_uses_one_budget_across_nested_children() {
    let value = checked("calc-size(calc-size(min-content, size), size + 1px)");
    assert_eq!(
        value
            .serialize_specified_with_limits(CssSpecifiedValueSerializationLimits::new(3, 100, 100))
            .unwrap_err()
            .kind(),
        CssSpecifiedValueSerializationErrorKind::InputNodeLimit
    );
    assert_eq!(
        value
            .serialize_specified_with_limits(CssSpecifiedValueSerializationLimits::new(100, 2, 100))
            .unwrap_err()
            .kind(),
        CssSpecifiedValueSerializationErrorKind::ProjectionNodeLimit
    );
    assert_eq!(
        value
            .serialize_specified_with_limits(CssSpecifiedValueSerializationLimits::new(
                100, 100, 12
            ))
            .unwrap_err()
            .kind(),
        CssSpecifiedValueSerializationErrorKind::ByteLimit
    );
}

#[test]
fn ordinary_numeric_admission_does_not_gain_size_or_calc_size() {
    for text in [
        "calc(size + 1px)",
        "calc(calc-size(min-content, size) + 1px)",
    ] {
        assert!(
            CssLengthPercentageCalculation::try_from_components(
                parse_component_values(text).unwrap()
            )
            .is_err(),
            "{text}"
        );
    }
}

#[test]
fn entire_component_graph_is_bounded_before_nested_admission() {
    let source = "calc-size(calc-size(min-content, size), size + 1px)";
    let values = parse_component_values(source).unwrap();
    let component = values.items()[0].clone();
    for limits in [
        CssComponentValueLimits::try_new(256, values.component_count() - 1, 1_000).unwrap(),
        CssComponentValueLimits::try_new(values.nesting_depth() - 1, 1_000, 1_000).unwrap(),
        CssComponentValueLimits::try_new(256, 1_000, source.len() - 1).unwrap(),
    ] {
        let error =
            CssCalcSize::try_from_component_with_limits(component.clone(), limits).unwrap_err();
        assert_eq!(
            error.kind(),
            &CssNumericConstructionErrorKind::ResourceLimit
        );
    }
}

#[test]
fn deep_checked_graph_clones_and_projects_within_the_hard_ceiling() {
    let source = format!(
        "calc-size({}1px{}, 1px)",
        "calc(".repeat(255),
        ")".repeat(255)
    );
    let values = parse_component_values(&source).unwrap();
    assert_eq!(values.nesting_depth(), 256);
    let original = checked(&source);
    let copied = original.clone();
    assert_eq!(copied.serialize_specified().unwrap(), "calc-size(1px, 1px)");
    assert_eq!(original, copied);

    let over_limit = format!(
        "calc-size({}1px{}, 1px)",
        "calc(".repeat(256),
        ")".repeat(256)
    );
    assert_eq!(
        parse_component_values(&over_limit).unwrap_err().kind(),
        CssComponentValueErrorKind::NestingLimit
    );
}

#[test]
fn nested_calc_size_bases_reach_the_hard_ceiling_without_recursion_overflow() {
    let mut source = "calc-size(min-content, size)".to_owned();
    for _ in 0..255 {
        source = format!("calc-size({source}, size)");
    }
    let values = parse_component_values(&source).unwrap();
    assert_eq!(values.nesting_depth(), 256);
    let value = checked(&source);
    let cloned = value.clone();
    let mut visited = 0;
    let mut current = &cloned;
    while let CssCalcSizeBasisRef::Nested(child) = current.basis() {
        visited += 1;
        current = child;
    }
    assert_eq!(visited, 255);
    assert!(matches!(
        current.basis(),
        CssCalcSizeBasisRef::Keyword(CssIntrinsicSizeKeyword::MinContent)
    ));
    assert_eq!(cloned.serialize_specified().unwrap(), source);

    let over_limit = format!("calc-size({source}, size)");
    assert_eq!(
        parse_component_values(&over_limit).unwrap_err().kind(),
        CssComponentValueErrorKind::NestingLimit
    );
}

#[test]
fn calc_size_equality_ignores_origins_but_keeps_exact_checked_structure() {
    let value = checked("calc-size(min-content, size + 1px)");
    let shifted = parse_component_values("  calc-size(min-content, size + 1px)").unwrap();
    let component = shifted
        .items()
        .iter()
        .find(|component| {
            matches!(
                component.view(),
                surgeist_css::CssComponentValueRef::Function(_)
            )
        })
        .unwrap()
        .clone();
    let shifted = CssCalcSize::try_from_component(component).unwrap();
    assert_ne!(value.origin(), shifted.origin());
    assert_eq!(value, shifted);
    assert_eq!(value, checked("calc-size(MIN-CONTENT, size + 1px)"));

    for different in [
        "calc-size(max-content, size + 1px)",
        "calc-size(min-content, size - 1px)",
        "calc-size(min-content, size + 2px)",
        "calc-size(min-content, (size + 1px))",
        "calc-size(calc-size(min-content, size), size + 1px)",
    ] {
        assert_ne!(value, checked(different), "{different}");
    }
}
