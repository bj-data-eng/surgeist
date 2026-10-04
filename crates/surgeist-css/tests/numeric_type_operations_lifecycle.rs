#![forbid(unsafe_code)]
//! Independent type oracles from Values 4 WD 2024-03-12 §10.9 and
//! Typed OM WD 2024-03-21 §4.3.2. Percent hints describe type substitution;
//! they neither supply a numeric basis nor permit mixing Number and Percent.
//! https://www.w3.org/TR/2024/WD-css-values-4-20240312/#calc-type-checking
//! https://www.w3.org/TR/2024/WD-css-typed-om-1-20240321/#cssnumericvalue-match

use surgeist_css::*;

const AXES: [CssNumericDimension; 7] = [
    CssNumericDimension::Length,
    CssNumericDimension::Angle,
    CssNumericDimension::Time,
    CssNumericDimension::Frequency,
    CssNumericDimension::Resolution,
    CssNumericDimension::Flex,
    CssNumericDimension::Percentage,
];
const DIMENSION_UNITS: [(CssNumericDimension, &str); 6] = [
    (CssNumericDimension::Length, "px"),
    (CssNumericDimension::Angle, "deg"),
    (CssNumericDimension::Time, "s"),
    (CssNumericDimension::Frequency, "hz"),
    (CssNumericDimension::Resolution, "dppx"),
    (CssNumericDimension::Flex, "fr"),
];

fn components(text: &str) -> CssComponentValues {
    parse_component_values(text).unwrap()
}
fn assert_type(ty: CssNumericType, exponents: [i32; 7], hint: Option<CssNumericDimension>) {
    for (axis, expected) in AXES.into_iter().zip(exponents) {
        assert_eq!(ty.exponent(axis), expected, "{axis:?}: {ty:?}");
    }
    assert_eq!(ty.percent_hint(), hint);
}
fn declaration(name: &str, text: &str) -> CssDeclaration {
    let report = parse_style_attribute(&format!("{name}:{text}!important;height:1px"));
    assert!(
        report.is_clean(),
        "{name}:{text}: {:?}",
        report.diagnostics()
    );
    assert_eq!(report.syntax().len(), 2);
    assert_eq!(
        report.syntax()[1].known().unwrap().property(),
        CssKnownProperty::Height
    );
    report.syntax()[0].clone()
}
fn number(source: &CssDeclaration) -> &CssNumberCalculation {
    match source.known().unwrap().property_value().unwrap() {
        CssKnownPropertyValueRef::Opacity(value) => {
            let CssOpacityValue::NumberCalculation(calculation) = value.value() else {
                panic!("Number opacity calculation")
            };
            calculation
        }
        CssKnownPropertyValueRef::Color(value) => {
            let CssColorComponent::NumberCalculation(calculation) =
                &value.value().rgb_value().unwrap().channels()[0]
            else {
                panic!("Number RGB channel")
            };
            calculation
        }
        _ => panic!("percentage-admitting Number consumer"),
    }
}
fn value_for(name: &str, expression: &str) -> String {
    if name == "color" {
        format!("rgb({expression} 0 0)")
    } else {
        expression.to_owned()
    }
}
fn hinted_number(source: &CssDeclaration, dimension: CssNumericDimension) {
    let calculation = number(source);
    assert_eq!(calculation.result_type(), CssCalculationType::Number);
    assert_type(calculation.numeric_type(), [0; 7], Some(dimension));
    assert_eq!(
        calculation.expression().numeric_type(),
        calculation.numeric_type()
    );
}

#[test]
fn percentage_admitting_color_and_opacity_match_number_without_erasing_each_dimension_hint() {
    for (dimension, unit) in DIMENSION_UNITS {
        let expression = format!("calc((1{unit} + 1%) / 1{unit})");
        for name in ["color", "opacity"] {
            let text = value_for(name, &expression);
            let source = declaration(name, &text);
            hinted_number(&source, dimension);
            assert_eq!(
                source.value_components().serialize().unwrap().as_css(),
                text
            );
            assert_eq!(source.importance(), CssImportance::Important);
            assert!(validate_style_attribute(&format!("{name}:{text}")).is_ok());
        }
    }
}

#[test]
fn hinted_number_construction_preserves_original_components_and_unresolved_percentage_leaf() {
    let expression = "calc((1px + 1%) / 1px)";
    for (name, property) in [
        ("color", CssKnownProperty::Color),
        ("opacity", CssKnownProperty::Opacity),
    ] {
        let text = value_for(name, expression);
        let original = components(&text);
        let source = parse_property_value(
            CssPropertyNameRef::Known(property),
            original.clone(),
            CssImportance::Important,
        )
        .unwrap_or_else(|error| panic!("checked {name}:{text}: {error:?}"));
        assert!(source.position().is_none());
        assert_eq!(source.value_components(), &original);
        hinted_number(&source, CssNumericDimension::Length);
        let calculation = number(&source);
        let CssCalculationExpressionRef::NestedCalc(calc) = calculation.expression() else {
            panic!("outer calc")
        };
        let CssCalculationExpressionRef::Product(product) = calc.operand() else {
            panic!("division")
        };
        let CssCalculationExpressionRef::Group(group) = product.factor(0).unwrap().expression()
        else {
            panic!("numerator group")
        };
        let CssCalculationExpressionRef::Sum(sum) = group.operand() else {
            panic!("mixed numerator")
        };
        assert_type(
            sum.term(0).unwrap().expression().numeric_type(),
            [1, 0, 0, 0, 0, 0, 0],
            None,
        );
        let CssCalculationExpressionRef::Value(CssCalculationValueRef::Percentage(percent)) =
            sum.term(1).unwrap().expression()
        else {
            panic!("unresolved authored percentage")
        };
        assert_eq!(percent.representation(), "1");
        // Applying the inferred hint to the sum does not rewrite its authored leaf.
        assert_type(percent.numeric_type(), [0, 0, 0, 0, 0, 0, 1], None);
        let serialized = calculation.serialize().unwrap();
        let reconstructed = declaration(name, &value_for(name, serialized.as_css()));
        hinted_number(&reconstructed, CssNumericDimension::Length);
        assert_eq!(source.value_components(), &original);
    }
}

#[test]
fn hinted_number_substitution_reentry_keeps_the_consumer_permission_and_source_occurrence() {
    for (name, property) in [
        ("color", CssKnownProperty::Color),
        ("opacity", CssKnownProperty::Opacity),
    ] {
        let source = declaration(name, "var(--numeric)");
        let CssExpansion::Pending(handle) = expand_declaration(&source).unwrap() else {
            panic!("pending substitution")
        };
        let text = value_for(name, "calc((1px + 1%) / 1px)");
        let replacement = components(&text);
        let checked = parse_property_value(
            CssPropertyNameRef::Known(property),
            replacement.clone(),
            CssImportance::Important,
        )
        .unwrap();
        hinted_number(&checked, CssNumericDimension::Length);
        let CssExpansion::Contributions(CssContributions::Longhands(expected)) =
            expand_declaration(&checked).unwrap()
        else {
            panic!("checked terminal")
        };
        let CssContributions::Longhands(actual) = handle.reenter(replacement.clone()).unwrap()
        else {
            panic!("replacement terminal")
        };
        let [actual] = actual.items() else {
            panic!("one replacement")
        };
        assert_eq!(actual.value(), expected.items()[0].value());
        assert_eq!(actual.property(), property);
        assert!(actual.source().same_occurrence(&source));
        assert_eq!(actual.source().importance(), CssImportance::Important);
        assert_eq!(actual.replacement_components(), Some(&replacement));
        for invalid in [
            value_for(name, "calc(1 + 1%)"),
            value_for(name, "calc(1px + 1s)"),
        ] {
            let error = handle.reenter(components(&invalid)).unwrap_err();
            assert!(matches!(
                error.kind(),
                CssExpansionErrorKind::InvalidReplacement(_)
            ));
        }
        assert!(handle.reenter(replacement).is_ok());
    }
}

#[test]
fn pure_number_roots_reject_inferred_hints_even_after_dimension_exponents_cancel() {
    for (_, unit) in DIMENSION_UNITS {
        let source = format!("calc((1{unit} + 1%) / 1{unit})");
        let error = CssNumberCalculation::try_from_components(components(&source)).unwrap_err();
        assert_eq!(
            error.kind(),
            &CssNumericConstructionErrorKind::RootDomainMismatch
        );
        assert!(error.origin().is_some());
    }
}

#[test]
fn ordinary_numeric_axes_and_math_constants_have_exact_non_evaluated_types() {
    macro_rules! axis {
        ($root:ident, $text:literal, $powers:expr) => {
            let value = $root::try_from_components(components($text)).unwrap();
            assert_type(value.numeric_type(), $powers, None);
            assert_type(value.expression().numeric_type(), $powers, None);
        };
    }
    axis!(CssNumberCalculation, "1e99999", [0; 7]);
    axis!(CssIntegerCalculation, "9007199254740993", [0; 7]);
    axis!(CssLengthCalculation, "1px", [1, 0, 0, 0, 0, 0, 0]);
    axis!(CssAngleCalculation, "1deg", [0, 1, 0, 0, 0, 0, 0]);
    axis!(CssTimeCalculation, "1s", [0, 0, 1, 0, 0, 0, 0]);
    axis!(CssFrequencyCalculation, "1hz", [0, 0, 0, 1, 0, 0, 0]);
    axis!(CssResolutionCalculation, "1dppx", [0, 0, 0, 0, 1, 0, 0]);
    axis!(CssFlexCalculation, "1fr", [0, 0, 0, 0, 0, 1, 0]);
    axis!(CssPercentageCalculation, "1%", [0, 0, 0, 0, 0, 0, 1]);
    for (text, expected) in [
        ("e", CssNumericConstant::E),
        ("pi", CssNumericConstant::Pi),
        ("infinity", CssNumericConstant::Infinity),
        ("-infinity", CssNumericConstant::NegativeInfinity),
        ("NaN", CssNumericConstant::NaN),
    ] {
        let value = CssNumberCalculation::try_from_components(components(&format!("calc({text})")))
            .unwrap();
        assert_type(value.numeric_type(), [0; 7], None);
        let CssCalculationExpressionRef::NestedCalc(calc) = value.expression() else {
            panic!("constant calc")
        };
        let CssCalculationExpressionRef::Constant(constant) = calc.operand() else {
            panic!("symbolic constant")
        };
        assert_eq!(constant.value(), expected);
        assert_type(constant.numeric_type(), [0; 7], None);
    }
    assert!(CssNumberCalculation::try_from_components(components("calc(custom-name)")).is_err());
}

#[test]
fn products_retain_positive_negative_and_mixed_intermediate_powers_before_cancellation() {
    for (text, powers) in [
        ("calc((1px * 2px) / 1px / 1px)", [2, 0, 0, 0, 0, 0, 0]),
        ("calc((1 / 1px) * 2px)", [-1, 0, 0, 0, 0, 0, 0]),
        ("calc((1px * 2s) / 1px / 1s)", [1, 0, 1, 0, 0, 0, 0]),
    ] {
        let value = CssNumberCalculation::try_from_components(components(text)).unwrap();
        assert_type(value.numeric_type(), [0; 7], None);
        let CssCalculationExpressionRef::NestedCalc(calc) = value.expression() else {
            panic!("outer calc")
        };
        let CssCalculationExpressionRef::Product(product) = calc.operand() else {
            panic!("product")
        };
        let CssCalculationExpressionRef::Group(group) = product.factor(0).unwrap().expression()
        else {
            panic!("intermediate group")
        };
        assert_type(group.operand().numeric_type(), powers, None);
    }
    for (_, unit) in DIMENSION_UNITS {
        let text = format!("calc(1{unit} / 2{unit})");
        let value = CssNumberCalculation::try_from_components(components(&text)).unwrap();
        assert_type(value.numeric_type(), [0; 7], None);
    }
    let value = CssNumberCalculation::try_from_components(components("calc(1% / 2%)")).unwrap();
    assert_type(value.numeric_type(), [0; 7], None);
}

#[test]
fn compatible_percentage_hints_propagate_through_addition_multiplication_and_division() {
    for text in [
        "calc((1px + 1%) + 2px)",
        "calc(2 * (1px + 1%))",
        "calc((1px + 1%) / 2)",
        "calc((1px + 1%) / (2px + 2%) * 1px)",
        "calc(1% / 2% * 1px)",
    ] {
        let value = CssLengthPercentageCalculation::try_from_components(components(text)).unwrap();
        assert_type(
            value.numeric_type(),
            [1, 0, 0, 0, 0, 0, 0],
            Some(CssNumericDimension::Length),
        );
        assert_eq!(value.components().serialize().unwrap().as_css(), text);
    }
    let pure =
        CssLengthCalculation::try_from_components(components("calc(1% / 2% * 1px)")).unwrap();
    assert_type(pure.numeric_type(), [1, 0, 0, 0, 0, 0, 0], None);
}

#[test]
fn incompatible_dimensions_and_conflicting_hints_fail_before_zero_simplification() {
    for text in [
        "calc(1px + 1s)",
        "calc(0 * 1px + 1s)",
        "calc(1px - 1px + 1s)",
        "calc((1px + 1%) * (1s + 1%))",
        "calc((1px + 1%) / (1s + 1%))",
        "calc((1px + 1%) + (1s + 1%))",
    ] {
        let error = CssNumberCalculation::try_from_components(components(text)).unwrap_err();
        assert_eq!(
            error.kind(),
            &CssNumericConstructionErrorKind::IncompatibleTypes,
            "{text}"
        );
        assert!(error.origin().is_some());
    }
    for text in [
        "calc(1px * 1px)",
        "calc(1 / 1px)",
        "calc(calc(1px * 1px) / 1px / 1px)",
    ] {
        assert!(
            CssNumberCalculation::try_from_components(components(text)).is_err(),
            "{text}"
        );
    }
}

#[test]
fn integer_number_phase_and_unitless_zero_controls_do_not_promote_or_mix_domains() {
    let integer = CssIntegerCalculation::try_from_components(components("calc(1.5)")).unwrap();
    assert!(integer.requires_rounding());
    assert_type(integer.numeric_type(), [0; 7], None);
    assert!(CssIntegerCalculation::try_from_components(components("1.5")).is_err());
    assert!(CssLengthCalculation::try_from_components(components("0")).is_ok());
    for text in ["calc(0)", "calc(0 + 1px)"] {
        assert!(CssLengthCalculation::try_from_components(components(text)).is_err());
    }
    for name in ["color", "opacity"] {
        let text = value_for(name, "calc(.25 + 25%)");
        let report = parse_style_attribute(&format!("{name}:{text};height:1px"));
        assert_eq!(report.syntax().len(), 1);
        assert_eq!(
            report.syntax()[0].known().unwrap().property(),
            CssKnownProperty::Height
        );
        assert_eq!(report.diagnostics().len(), 1);
        assert_eq!(
            report.diagnostics()[0].action(),
            CssRecoveryAction::DropDeclaration
        );
        assert_eq!(
            validate_style_attribute(&format!("{name}:{text};height:1px"))
                .unwrap_err()
                .diagnostics(),
            report.diagnostics()
        );
    }
}
