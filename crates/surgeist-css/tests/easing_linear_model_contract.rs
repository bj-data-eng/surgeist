#![forbid(unsafe_code)]
//! Easing2 WD 2024-08-29 §§2.1.1–2.1.2 define authored cardinality and
//! unordered Number/contiguous Percentage-group grammar. These new API tests
//! preserve authored inputs before contextual creation or computed serialization.

use surgeist_css::*;

fn number(source: &str) -> CssSpecifiedNumber {
    CssSpecifiedNumber::try_from_calculation(
        CssNumberCalculation::try_from_components(parse_component_values(source).unwrap()).unwrap(),
    )
    .unwrap()
}

fn percentage(source: &str) -> CssSpecifiedPercentage {
    CssSpecifiedPercentage::try_from_calculation(
        CssPercentageCalculation::try_from_components(parse_component_values(source).unwrap())
            .unwrap(),
    )
    .unwrap()
}

fn parsed(source: &str) -> CssLinearEasing {
    let declaration = parse_property_value(
        CssPropertyNameRef::Known(CssKnownProperty::TransitionTimingFunction),
        parse_component_values(source).unwrap(),
        CssImportance::Normal,
    )
    .unwrap();
    let CssKnownPropertyValueRef::TransitionTimingFunction(value) =
        declaration.known().unwrap().property_value().unwrap()
    else {
        panic!("timing wrapper")
    };
    let [CssEasing::Linear(value)] = value.timing_functions().values() else {
        panic!("one linear function")
    };
    value.clone()
}

#[test]
fn checked_cardinality_preserves_omitted_single_and_paired_authored_inputs() {
    let omitted = CssLinearStop::try_new(number("-2"), vec![]).unwrap();
    let single = CssLinearStop::try_new(number("3"), vec![percentage("-20%")]).unwrap();
    let pair =
        CssLinearStop::try_new(number("0.5"), vec![percentage("150%"), percentage("20%")]).unwrap();
    assert_eq!(omitted.inputs().len(), 0);
    assert_eq!(single.inputs().len(), 1);
    assert_eq!(pair.inputs().len(), 2);
    assert!(
        CssLinearStop::try_new(
            number("1"),
            vec![percentage("0%"), percentage("50%"), percentage("100%")]
        )
        .is_none()
    );
    assert!(CssLinearEasing::try_new(vec![]).is_none());
    assert!(CssLinearEasing::try_new(vec![pair.clone()]).is_none());
    let value = CssLinearEasing::try_new(vec![omitted, single, pair]).unwrap();
    assert_eq!(value.stops().len(), 3);
    assert_eq!(
        value.serialize_specified().unwrap(),
        "linear(-2, 3 -20%, 0.5 150% 20%)"
    );
    assert_eq!(value, parsed("linear(-2, -20% 3, 150% 20% 0.5)"));
}

#[test]
fn checked_numeric_domains_and_original_recovered_math_are_rejected_before_composition() {
    for source in [
        "1px",
        "10%",
        "calc(1px)",
        "calc((1px + 1%) / 1px)",
        "calc(1",
    ] {
        assert!(
            CssNumberCalculation::try_from_components(parse_component_values(source).unwrap())
                .is_err(),
            "{source}"
        );
    }
    for source in ["1", "1px", "calc(1)", "calc(1%"] {
        assert!(
            CssPercentageCalculation::try_from_components(parse_component_values(source).unwrap())
                .is_err(),
            "{source}"
        );
    }
    assert!(
        CssSpecifiedNumber::try_from_component(CssComponentValue::try_token("1%").unwrap())
            .is_err()
    );
    assert!(
        CssSpecifiedPercentage::try_from_component(CssComponentValue::try_number("1").unwrap())
            .is_err()
    );
}

#[test]
fn browser_retains_recovered_numeric_math_but_public_linear_constructors_reject_it() {
    for source in ["linear(0, calc(1", "linear(0, 1 calc(20%"] {
        let report = parse_style_attribute(&format!("transition-timing-function:{source}"));
        assert!(!report.is_clean());
        let [declaration] = report.syntax().as_slice() else {
            panic!("browser retains recovered numeric graph")
        };
        let CssKnownPropertyValueRef::TransitionTimingFunction(value) =
            declaration.known().unwrap().property_value().unwrap()
        else {
            panic!("timing wrapper")
        };
        let [CssEasing::Linear(value)] = value.timing_functions().values() else {
            panic!("linear model")
        };
        let last = &value.stops()[1];
        assert!(CssLinearStop::try_new(last.output().clone(), last.inputs().to_vec()).is_none());
        assert!(CssLinearEasing::try_new(value.stops().to_vec()).is_none());
        assert!(
            parse_property_value(
                CssPropertyNameRef::Known(CssKnownProperty::TransitionTimingFunction),
                parse_component_values(source).unwrap(),
                CssImportance::Normal
            )
            .is_err()
        );
        assert_eq!(value.stops().len(), 2);
        assert!(value.serialize_specified().is_ok());
    }
}

#[test]
fn checked_composition_keeps_exact_original_tokens_math_and_origins() {
    let output = number("-9007199254740993");
    let first = percentage("+000150.0000000%");
    let second = percentage("calc(10% + 20%)");
    let stop = CssLinearStop::try_new(output.clone(), vec![first.clone(), second.clone()]).unwrap();
    let value = CssLinearEasing::try_new(vec![
        stop.clone(),
        CssLinearStop::try_new(number("sign(-2em)"), vec![]).unwrap(),
    ])
    .unwrap();
    assert_eq!(stop.output(), &output);
    assert_eq!(stop.output().origin(), output.origin());
    assert_eq!(
        stop.output().literal_component(),
        output.literal_component()
    );
    assert_eq!(stop.inputs(), &[first.clone(), second.clone()]);
    assert_eq!(stop.inputs()[0].origin(), first.origin());
    assert_eq!(
        stop.inputs()[1].calculation().unwrap().components(),
        second.calculation().unwrap().components()
    );
    assert_eq!(
        value.serialize_specified().unwrap(),
        "linear(-9007199254740993 150% calc(30%), sign(-2em))"
    );
    assert_eq!(value.stops()[0], stop);
}

#[test]
fn equality_ignores_origins_but_retains_stop_order_omissions_pairs_and_math_structure() {
    assert_eq!(
        parsed("linear(+000.0 20e0%, 1e0)"),
        parsed("linear(0 20%, 1)")
    );
    // Formatting rounds both coefficients to one spelling, but authored equality
    // compares exact values before projection.
    let precise = parsed("linear(0.0000001 20%, 1)");
    let zero = parsed("linear(0 20%, 1)");
    assert_eq!(
        precise.serialize_specified().unwrap(),
        zero.serialize_specified().unwrap()
    );
    assert_ne!(precise, zero);
    let value = parsed("linear(calc(1 + 2) calc(10% + 20%), 1)");
    assert_eq!(value, parsed("LINEAR(CALC(1 + 2) CALC(10% + 20%), 1)"));
    for different in [
        "linear(calc(2 + 1) calc(10% + 20%), 1)",
        "linear(3 30%, 1)",
        "linear(calc(1 + 2) calc(10% + 20%), 1 100%)",
        "linear(calc(1 + 2) calc(10% + 20%) 40%, 1)",
        "linear(1, calc(1 + 2) calc(10% + 20%))",
    ] {
        assert_ne!(value, parsed(different), "{different}");
    }
}

fn assert_limits(value: &CssLinearEasing, expected: &str, input: usize, projection: usize) {
    let before = value.clone();
    assert_eq!(
        value
            .serialize_specified_with_limits(CssSpecifiedValueSerializationLimits::new(
                input,
                projection,
                expected.len()
            ))
            .unwrap(),
        expected
    );
    for (limits, kind) in [
        (
            CssSpecifiedValueSerializationLimits::new(input - 1, projection, expected.len()),
            CssSpecifiedValueSerializationErrorKind::InputNodeLimit,
        ),
        (
            CssSpecifiedValueSerializationLimits::new(input, projection - 1, expected.len()),
            CssSpecifiedValueSerializationErrorKind::ProjectionNodeLimit,
        ),
        (
            CssSpecifiedValueSerializationLimits::new(input, projection, expected.len() - 1),
            CssSpecifiedValueSerializationErrorKind::ByteLimit,
        ),
    ] {
        assert_eq!(
            value
                .serialize_specified_with_limits(limits)
                .unwrap_err()
                .kind(),
            kind
        );
        assert_eq!(value, &before);
        assert_eq!(value.serialize_specified().unwrap(), expected);
    }
}

#[test]
fn function_stops_and_numeric_children_share_exact_cumulative_budgets() {
    // Function 1 + stops 2 + output leaves 2 + input leaves 2 = 7.
    assert_limits(
        &parsed("linear(0, 1 20% 40%)"),
        "linear(0, 1 20% 40%)",
        7,
        7,
    );
    // Function/stops 3, two calc Sum graphs visit 4 each, final output 1.
    // Each sum projects three scalar nodes, giving 3 + 3 + 3 + 1 = 10.
    assert_limits(
        &parsed("linear(calc(1 + 2) calc(10% + 20%), 1)"),
        "linear(calc(3) calc(30%), 1)",
        12,
        10,
    );
    // List 1 + first function/stops/scalars 5 + second function/stops/scalars 7.
    let expected = "linear(0, 1), linear(0, 1 20% 40%)";
    let list = CssEasingList::try_new(vec![
        CssEasing::Linear(parsed("linear(0, 1)")),
        CssEasing::Linear(parsed("linear(0, 1 20% 40%)")),
    ])
    .unwrap();
    assert_eq!(
        list.serialize_specified_with_limits(CssSpecifiedValueSerializationLimits::new(
            13,
            13,
            expected.len()
        ))
        .unwrap(),
        expected
    );
    assert_eq!(
        list.serialize_specified_with_limits(CssSpecifiedValueSerializationLimits::new(
            12,
            13,
            expected.len()
        ))
        .unwrap_err()
        .kind(),
        CssSpecifiedValueSerializationErrorKind::InputNodeLimit
    );
    assert_eq!(
        list.serialize_specified_with_limits(CssSpecifiedValueSerializationLimits::new(
            13,
            12,
            expected.len()
        ))
        .unwrap_err()
        .kind(),
        CssSpecifiedValueSerializationErrorKind::ProjectionNodeLimit
    );
    assert_eq!(list.serialize_specified().unwrap(), expected);
    let stop = CssLinearStop::try_new(number("-2"), vec![percentage("150%")]).unwrap();
    assert_eq!(
        stop.serialize_specified_with_limits(CssSpecifiedValueSerializationLimits::new(3, 3, 7))
            .unwrap(),
        "-2 150%"
    );
    assert_eq!(
        stop.serialize_specified_with_limits(CssSpecifiedValueSerializationLimits::new(2, 3, 7))
            .unwrap_err()
            .kind(),
        CssSpecifiedValueSerializationErrorKind::InputNodeLimit
    );
    assert_eq!(
        stop.serialize_specified_with_limits(CssSpecifiedValueSerializationLimits::new(3, 2, 7))
            .unwrap_err()
            .kind(),
        CssSpecifiedValueSerializationErrorKind::ProjectionNodeLimit
    );
}

#[test]
fn shorthand_inverse_reconstructs_both_timing_consumers_without_fixup() {
    for (source, property, expected) in [
        (
            "transition:opacity 1s linear(0 80% 20%, 1)",
            CssKnownProperty::Transition,
            "opacity 1s linear(0 80% 20%, 1)",
        ),
        (
            "animation:fade 1s linear(0 80% 20%, 1)",
            CssKnownProperty::Animation,
            "1s linear(0 80% 20%, 1) fade",
        ),
    ] {
        let report = parse_style_attribute(source);
        assert!(report.is_clean());
        let block = CssSpecifiedDeclarationBlock::try_from_declarations(report.syntax()).unwrap();
        assert_eq!(
            block
                .property_value(CssPropertyNameRef::Known(property))
                .unwrap()
                .as_deref(),
            Some(expected)
        );
    }
}
