//! Easing2 WD 2024-08-29 §§2.1.1–2.1.2 define authored cardinality and
//! unordered Number/contiguous Percentage-group grammar. These new API tests
//! preserve authored inputs before contextual creation or computed serialization.

use crate::css::support::failure_classes::NumericFailure::{
    RecoveredComponent, RootDomainMismatch,
};
use crate::css::support::property_admission::{
    assert_checked_closure_rejected, assert_component_rejected, assert_implicit_closure,
    assert_numeric_rejected,
};
use crate::css::support::serialization_limits::LimitLadder;
use surgeist_css::*;

fn number(source: &str) -> CssNumeric<CssNumber> {
    CssNumeric::<CssNumber>::try_from_calculation(
        CssCalculation::<CssNumber>::try_from_components(
            CssComponentValues::parse(&CssParser::default(), source).unwrap(),
            Default::default(),
        )
        .unwrap(),
    )
    .unwrap()
}

fn percentage(source: &str) -> CssNumeric<CssPercentage> {
    CssNumeric::<CssPercentage>::try_from_calculation(
        CssCalculation::<CssPercentage>::try_from_components(
            CssComponentValues::parse(&CssParser::default(), source).unwrap(),
            Default::default(),
        )
        .unwrap(),
    )
    .unwrap()
}

fn parsed(source: &str) -> CssLinearEasing {
    let declaration = CssDeclaration::try_from_components(
        &CssParser::default(),
        CssPropertyTarget::Property(CssPropertyNameRef::Known(
            CssKnownProperty::TransitionTimingFunction,
        )),
        CssComponentValues::parse(&CssParser::default(), source).unwrap(),
        CssImportance::Normal,
    )
    .unwrap();
    let CssKnownPropertyValueRef::TransitionTimingFunction(value) =
        declaration.known().unwrap().property_value().unwrap()
    else {
        panic!("timing wrapper")
    };
    let [CssEasing::Linear(value)] = value.value().values() else {
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
        value.serialize_specified(Default::default()).unwrap(),
        "linear(-2, 3 -20%, 0.5 150% 20%)"
    );
    assert_eq!(value, parsed("linear(-2, -20% 3, 150% 20% 0.5)"));
}

#[test]
fn checked_numeric_domains_and_original_recovered_math_are_rejected_before_composition() {
    for (source, kind) in [
        ("1px", RootDomainMismatch),
        ("10%", RootDomainMismatch),
        ("calc(1px)", RootDomainMismatch),
        ("calc((1px + 1%) / 1px)", RootDomainMismatch),
        ("calc(1", RecoveredComponent),
    ] {
        assert_numeric_rejected(
            CssCalculation::<CssNumber>::try_from_components(
                CssComponentValues::parse(&CssParser::default(), source).unwrap(),
                Default::default(),
            ),
            kind,
        );
    }
    for (source, kind) in [
        ("1", RootDomainMismatch),
        ("1px", RootDomainMismatch),
        ("calc(1)", RootDomainMismatch),
        ("calc(1%", RecoveredComponent),
    ] {
        assert_numeric_rejected(
            CssCalculation::<CssPercentage>::try_from_components(
                CssComponentValues::parse(&CssParser::default(), source).unwrap(),
                Default::default(),
            ),
            kind,
        );
    }
    assert_component_rejected(
        CssNumeric::<CssNumber>::try_from_component,
        CssComponentValue::try_token("1%").unwrap(),
        RootDomainMismatch,
    );
    assert_component_rejected(
        CssNumeric::<CssPercentage>::try_from_component,
        CssComponentValue::try_number("1").unwrap(),
        RootDomainMismatch,
    );
}

#[test]
fn browser_retains_recovered_numeric_math_but_public_linear_constructors_reject_it() {
    for source in ["linear(0, calc(1", "linear(0, 1 calc(20%"] {
        let css = format!("transition-timing-function:{source}");
        let report = CssParser::default().parse_block_contents(&css);
        assert_implicit_closure(&css, &report);
        let [declaration] = report.syntax().as_slice() else {
            panic!("browser retains recovered numeric graph")
        };
        let CssKnownPropertyValueRef::TransitionTimingFunction(value) =
            declaration.known().unwrap().property_value().unwrap()
        else {
            panic!("timing wrapper")
        };
        let [CssEasing::Linear(value)] = value.value().values() else {
            panic!("linear model")
        };
        let last = &value.stops()[1];
        assert!(CssLinearStop::try_new(last.output().clone(), last.inputs().to_vec()).is_none());
        assert!(CssLinearEasing::try_new(value.stops().to_vec()).is_none());
        assert_checked_closure_rejected(CssDeclaration::try_from_components(
            &CssParser::default(),
            CssPropertyTarget::Property(CssPropertyNameRef::Known(
                CssKnownProperty::TransitionTimingFunction,
            )),
            CssComponentValues::parse(&CssParser::default(), source).unwrap(),
            CssImportance::Normal,
        ));
        assert_eq!(value.stops().len(), 2);
        assert!(value.serialize_specified(Default::default()).is_ok());
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
        stop.output().literal().map(|literal| literal.component()),
        output.literal().map(|literal| literal.component())
    );
    assert_eq!(stop.inputs(), &[first.clone(), second.clone()]);
    assert_eq!(stop.inputs()[0].origin(), first.origin());
    assert_eq!(
        stop.inputs()[1].calculation().unwrap().components(),
        second.calculation().unwrap().components()
    );
    assert_eq!(
        value.serialize_specified(Default::default()).unwrap(),
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
        precise.serialize_specified(Default::default()).unwrap(),
        zero.serialize_specified(Default::default()).unwrap()
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
    LimitLadder::new(expected, input, projection).assert_unchanged_with(
        value,
        CssLinearEasing::serialize_specified,
        |_, _, _| {
            assert_eq!(
                value.serialize_specified(Default::default()).unwrap(),
                expected
            )
        },
    );
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
        list.serialize_specified(CssSerializationLimits::new(13, 13, expected.len()))
            .unwrap(),
        expected
    );
    assert!(
        list.serialize_specified(CssSerializationLimits::new(12, 13, expected.len()))
            .unwrap_err()
            .is_input_node_limit()
    );
    assert!(
        list.serialize_specified(CssSerializationLimits::new(13, 12, expected.len()))
            .unwrap_err()
            .is_projection_node_limit()
    );
    assert_eq!(
        list.serialize_specified(Default::default()).unwrap(),
        expected
    );
    let stop = CssLinearStop::try_new(number("-2"), vec![percentage("150%")]).unwrap();
    assert_eq!(
        stop.serialize_specified(CssSerializationLimits::new(3, 3, 7))
            .unwrap(),
        "-2 150%"
    );
    assert!(
        stop.serialize_specified(CssSerializationLimits::new(2, 3, 7))
            .unwrap_err()
            .is_input_node_limit()
    );
    assert!(
        stop.serialize_specified(CssSerializationLimits::new(3, 2, 7))
            .unwrap_err()
            .is_projection_node_limit()
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
        let report = CssParser::default().parse_block_contents(source);
        assert!(report.is_clean());
        let block = CssSpecifiedDeclarationBlock::try_from_declarations(
            report.syntax(),
            Default::default(),
        )
        .unwrap();
        assert_eq!(
            block
                .property_value(CssPropertyNameRef::Known(property), Default::default())
                .unwrap()
                .as_deref(),
            Some(expected)
        );
    }
}
