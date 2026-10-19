#![forbid(unsafe_code)]
//! Values 4 WD 2024-03-12 §10.6 admits one calculation to sign(), whose
//! result is Number made consistent with its input. Unlike abs/comparison
//! results, the input need not itself be a simple CSS numeric domain.
//! §10.9 checks each function's result, allowing compound intermediate types.
//! https://www.w3.org/TR/2024/WD-css-values-4-20240312/#sign-funcs
//! https://www.w3.org/TR/2024/WD-css-values-4-20240312/#calc-type-checking
//! §10.8 requires at least 32 arguments for arbitrary-arity functions; an
//! explicit component budget is separate from that minimum grammar capacity.
//! https://www.w3.org/TR/2024/WD-css-values-4-20240312/#calc-syntax

use surgeist_css::*;

fn checked(source: &str) -> CssNumberCalculation {
    let components = parse_component_values(source).unwrap();
    let before = components.clone();
    let result = CssNumberCalculation::try_from_components(components.clone());
    assert_eq!(components, before);
    let calculation = result.unwrap_or_else(|error| panic!("{source}: {error:?}"));
    assert_eq!(calculation.result_type(), CssCalculationType::Number);
    assert_eq!(calculation.components(), &before);
    for dimension in [
        CssNumericDimension::Length,
        CssNumericDimension::Angle,
        CssNumericDimension::Time,
        CssNumericDimension::Frequency,
        CssNumericDimension::Resolution,
        CssNumericDimension::Flex,
        CssNumericDimension::Percentage,
    ] {
        assert_eq!(calculation.numeric_type().exponent(dimension), 0);
    }
    assert_eq!(calculation.numeric_type().percent_hint(), None);
    calculation
}

fn sign_argument(calculation: &CssNumberCalculation) -> CssCalculationExpressionRef<'_> {
    let CssCalculationExpressionRef::Function(function) = calculation.expression() else {
        panic!("sign function")
    };
    assert_eq!(function.function(), CssMathFunction::Sign);
    assert_eq!(function.len(), 1);
    function.argument(0).unwrap().unwrap()
}

#[test]
fn sign_accepts_squared_length_input_and_retains_the_product_operator_origin() {
    let calculation = checked("sign(1px * 1px)");
    let argument = sign_argument(&calculation);
    assert_eq!(
        argument
            .numeric_type()
            .exponent(CssNumericDimension::Length),
        2
    );
    let CssCalculationExpressionRef::Product(product) = argument else {
        panic!("compound product")
    };
    assert_eq!(product.len(), 2);
    let factor = product.factor(1).unwrap();
    assert_eq!(
        factor.operator(),
        Some(CssCalculationProductOperator::Multiply)
    );
    let CssValueOrigin::Parsed(origin) = factor.operator_origin().unwrap() else {
        panic!("parsed multiplication")
    };
    assert_eq!(origin.span().start().byte_offset().value(), 9);
    assert_eq!(origin.source().as_str(), "sign(1px * 1px)");
}

#[test]
fn sign_accepts_a_length_time_quotient_without_changing_its_argument_type() {
    let calculation = checked("sign(1px / 1s)");
    let argument = sign_argument(&calculation);
    assert_eq!(
        argument
            .numeric_type()
            .exponent(CssNumericDimension::Length),
        1
    );
    assert_eq!(
        argument.numeric_type().exponent(CssNumericDimension::Time),
        -1
    );
    let CssCalculationExpressionRef::Product(product) = argument else {
        panic!("compound quotient")
    };
    assert_eq!(
        product.factor(1).unwrap().operator(),
        Some(CssCalculationProductOperator::Divide)
    );
    assert!(matches!(
        product.factor(1).unwrap().operator_origin(),
        Some(CssValueOrigin::Parsed(_))
    ));
}

#[test]
fn grouped_relative_compound_arguments_remain_symbolic_and_preserve_authored_graphs() {
    // An em basis can be zero. Neither the product nor quotient establishes
    // whether the resolved sign is +1, -1 or signed zero without that context.
    for (source, expected) in [
        ("sign((1em * 1px))", "sign(1em * 1px)"),
        ("sign(1em / 1s)", "sign(1em / 1s)"),
        ("sign((1em * 1px) / 1s)", "sign(1em * 1px / 1s)"),
    ] {
        let calculation = checked(source);
        let components = calculation.components().clone();
        let origin = calculation.origin().clone();
        let value = CssSpecifiedNumber::try_from_calculation(calculation).unwrap();
        let before = value.clone();
        assert_eq!(value.serialize_specified().unwrap(), expected);
        assert_eq!(value, before);
        assert_eq!(value.origin(), &origin);
        assert_eq!(value.calculation().unwrap().components(), &components);
    }
}

#[test]
fn authored_opacity_admits_number_result_sign_with_compound_inputs() {
    for expression in ["sign(1px * 1px)", "sign(1px / 1s)", "sign((1em * 1px))"] {
        let css = format!("opacity:{expression}!important");
        let report = parse_style_attribute(&css);
        assert!(report.is_clean(), "{css}: {:?}", report.diagnostics());
        assert_eq!(validate_style_attribute(&css), Ok(report.syntax().clone()));
        let [declaration] = report.syntax().as_slice() else {
            panic!("one opacity")
        };
        let CssKnownPropertyValueRef::Opacity(value) =
            declaration.known().unwrap().property_value().unwrap()
        else {
            panic!("opacity")
        };
        let CssOpacityValue::NumberCalculation(calculation) = value.value() else {
            panic!("number calculation")
        };
        assert_eq!(calculation.result_type(), CssCalculationType::Number);
        let components = declaration.value_components().clone();
        let constructed = parse_property_value(
            CssPropertyNameRef::Known(CssKnownProperty::Opacity),
            components.clone(),
            CssImportance::Important,
        )
        .unwrap();
        assert_eq!(constructed.value_components(), &components);
        assert_eq!(constructed.importance(), CssImportance::Important);
        let sheet = format!("a{{{css}}}");
        let report = parse_sheet(&sheet);
        assert!(report.is_clean(), "{sheet}: {:?}", report.diagnostics());
    }
}

#[test]
fn programmatic_sign_preserves_exact_relative_compound_input_and_origins() {
    let components = CssComponentValues::try_new(vec![
        CssComponentValue::try_function(
            "SIGN",
            CssComponentValues::try_new(vec![
                CssComponentValue::try_dimension("+01.00", "EM").unwrap(),
                CssComponentValue::try_token("*").unwrap(),
                CssComponentValue::try_dimension("02.000", "px").unwrap(),
            ])
            .unwrap(),
        )
        .unwrap(),
    ])
    .unwrap();
    let calculation = CssNumberCalculation::try_from_components(components.clone()).unwrap();
    assert_eq!(calculation.origin(), &CssValueOrigin::Programmatic);
    assert_eq!(
        sign_argument(&calculation)
            .numeric_type()
            .exponent(CssNumericDimension::Length),
        2
    );
    let value = CssSpecifiedNumber::try_from_calculation(calculation).unwrap();
    let before = value.clone();
    assert_eq!(value.serialize_specified().unwrap(), "sign(1em * 2px)");
    assert_eq!(value, before);
    assert_eq!(value.origin(), &CssValueOrigin::Programmatic);
    assert_eq!(value.calculation().unwrap().components(), &components);
}

#[test]
fn bare_calc_abs_and_comparison_compound_results_remain_invalid() {
    for source in [
        "calc(1px * 1px)",
        "abs(1px * 1px)",
        "min(1px * 1px,2px * 2px)",
        "max(1px / 1s,2px / 2s)",
        "sign(calc(1px * 1px))",
        "sign(abs(1px * 1px))",
    ] {
        let components = parse_component_values(source).unwrap();
        let before = components.clone();
        let error = CssNumberCalculation::try_from_components(components.clone()).unwrap_err();
        assert_eq!(
            error.kind(),
            &CssNumericConstructionErrorKind::InvalidArgumentType,
            "{source}"
        );
        assert_eq!(components, before);
        let css = format!("opacity:{source};color:red");
        let report = parse_style_attribute(&css);
        assert_eq!(report.diagnostics().len(), 1, "{css}");
        assert_eq!(report.syntax().len(), 1);
        assert_eq!(
            report.syntax()[0].known().unwrap().property(),
            CssKnownProperty::Color
        );
    }
}

#[test]
fn sign_keeps_one_argument_and_existing_simple_signed_zero_semantics() {
    for source in ["sign()", "sign(1,2)", "sign(none)"] {
        assert!(
            CssNumberCalculation::try_from_components(parse_component_values(source).unwrap())
                .is_err()
        );
    }
    for (source, expected) in [
        ("sign(-2px)", "calc(-1)"),
        ("sign(0)", "calc(0)"),
        // Literal -0 is unsigned positive zero (Values 4 §10.9.1);
        // only arithmetic can manufacture negative zero.
        ("calc(1 / sign(-0))", "calc(infinity)"),
        ("calc(1 / sign(-1 * 0))", "calc(-infinity)"),
    ] {
        let value = CssSpecifiedNumber::try_from_calculation(checked(source)).unwrap();
        assert_eq!(value.serialize_specified().unwrap(), expected);
    }
}

#[test]
fn arbitrary_arity_functions_accept_31_32_and_33_arguments_with_independent_results() {
    // 32 is a required minimum, not a maximum. Thirty-three remains valid
    // when the caller has not selected a tighter aggregate resource budget.
    for count in [31, 32, 33] {
        for (function, arguments, expected) in [
            (
                "min",
                (1..=count).map(|n| n.to_string()).collect::<Vec<_>>(),
                "calc(1)".to_owned(),
            ),
            (
                "max",
                (1..=count).map(|n| n.to_string()).collect::<Vec<_>>(),
                format!("calc({count})"),
            ),
            (
                "hypot",
                (0..count)
                    .map(|i| {
                        if i == 0 {
                            "3"
                        } else if i == 1 {
                            "4"
                        } else {
                            "0"
                        }
                        .to_owned()
                    })
                    .collect::<Vec<_>>(),
                "calc(5)".to_owned(),
            ),
        ] {
            let source = format!("{function}({})", arguments.join(","));
            let calculation = checked(&source);
            let CssCalculationExpressionRef::Function(root) = calculation.expression() else {
                panic!("variadic function")
            };
            assert_eq!(root.len(), count);
            let value = CssSpecifiedNumber::try_from_calculation(calculation).unwrap();
            assert_eq!(value.serialize_specified().unwrap(), expected);
        }
    }
}

#[test]
fn a_32_argument_calculation_obeys_explicit_aggregate_limits_atomically() {
    let source = format!("min({})", ["1"; 32].join(","));
    let components = parse_component_values(&source).unwrap();
    let before = components.clone();
    // One function +32 numeric tokens +31 commas =64 components, depth one.
    // Canonical grammar text is 5 +32 +31*2 =99 bytes, independently of
    // the 68-byte original spelling and the shorter projected calc(1).
    let exact = CssComponentValueLimits::try_new(1, 64, 99).unwrap();
    let calculation =
        CssNumberCalculation::try_from_components_with_limits(components.clone(), exact).unwrap();
    assert_eq!(calculation.components(), &components);
    for limits in [
        CssComponentValueLimits::try_new(0, 64, 99).unwrap(),
        CssComponentValueLimits::try_new(1, 63, 99).unwrap(),
        CssComponentValueLimits::try_new(1, 64, 98).unwrap(),
    ] {
        let error =
            CssNumberCalculation::try_from_components_with_limits(components.clone(), limits)
                .unwrap_err();
        assert_eq!(
            error.kind(),
            &CssNumericConstructionErrorKind::ResourceLimit
        );
        assert_eq!(components, before);
        assert!(error.origin().is_some());
    }
    let larger = parse_component_values(&format!("min({})", ["1"; 33].join(","))).unwrap();
    let larger_before = larger.clone();
    assert_eq!(
        CssNumberCalculation::try_from_components_with_limits(larger.clone(), exact)
            .unwrap_err()
            .kind(),
        &CssNumericConstructionErrorKind::ResourceLimit
    );
    assert_eq!(larger, larger_before);
    assert_eq!(
        CssSpecifiedNumber::try_from_calculation(calculation)
            .unwrap()
            .serialize_specified()
            .unwrap(),
        "calc(1)"
    );
}
