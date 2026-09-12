#![forbid(unsafe_code)]
//! Checked sum assembly preserves authored operands and applies ordinary numeric
//! typing. Root-only unitless-zero admission does not turn zero into a length.
use surgeist_css::{
    CssCalculationExpressionRef, CssCalculationSumOperator as Op, CssComponentValueError,
    CssComponentValueErrorKind, CssComponentValueLimits, CssComponentValueRef,
    CssLengthPercentageCalculation as Calculation, CssNumericConstructionError,
    CssNumericConstructionErrorKind, CssSerializedOrigin, CssValueOrigin, parse_component_values,
};

fn operand(source: &str) -> Calculation {
    Calculation::try_from_components(parse_component_values(source).unwrap()).unwrap()
}

fn resource(error: &CssNumericConstructionError, expected: CssComponentValueErrorKind) {
    assert_eq!(
        error.kind(),
        &CssNumericConstructionErrorKind::ResourceLimit
    );
    let component = std::error::Error::source(error)
        .and_then(|source| source.downcast_ref::<CssComponentValueError>())
        .expect("original component resource failure");
    assert_eq!(component.kind(), expected);
}

fn same_origin(actual: &CssValueOrigin, expected: &CssValueOrigin) {
    let (CssValueOrigin::Parsed(actual), CssValueOrigin::Parsed(expected)) = (actual, expected)
    else {
        panic!("original parsed token")
    };
    assert!(actual.source().same_snapshot(expected.source()));
    assert_eq!(actual.span(), expected.span());
}

#[test]
fn one_operand_keeps_its_signed_token_inside_an_explicit_calc_wrapper() {
    for (source, expected) in [
        ("-1px", "calc(-1px)"),
        ("+001.5PX", "calc(+001.5px)"),
        ("0px", "calc(0px)"),
        ("-2%", "calc(-2%)"),
    ] {
        let sum = Calculation::try_sum(operand(source), []).unwrap();
        assert_eq!(sum.serialize().unwrap().as_css(), expected);
        assert_eq!(sum.origin(), &CssValueOrigin::Programmatic);
        assert_eq!(sum.position(), None);
    }
}

#[test]
fn mixed_units_percentages_and_later_subtraction_remain_symbolic() {
    let sum = Calculation::try_sum(
        operand("-1em"),
        [(Op::Add, operand("2%")), (Op::Subtract, operand("3px"))],
    )
    .unwrap();
    assert_eq!(sum.serialize().unwrap().as_css(), "calc(-1em + 2% - 3px)");
    let nested = Calculation::try_sum(sum, [(Op::Add, operand("calc(4px / 2)"))]).unwrap();
    assert_eq!(
        nested.serialize().unwrap().as_css(),
        "calc(calc(-1em + 2% - 3px) + calc(4px / 2))"
    );
}

#[test]
fn identical_tokens_from_distinct_snapshots_preserve_each_operand_and_insert_programmatic_syntax() {
    let first = operand("+001.5PX");
    let second = operand("+001.5PX");
    let first_components = first.components().clone();
    let second_components = second.components().clone();
    let first_origin = first.origin().clone();
    let second_origin = second.origin().clone();
    let (CssValueOrigin::Parsed(a), CssValueOrigin::Parsed(b)) = (&first_origin, &second_origin)
    else {
        panic!("snapshots")
    };
    assert!(!a.source().same_snapshot(b.source()));
    let sum = Calculation::try_sum(first, [(Op::Subtract, second)]).unwrap();
    let CssComponentValueRef::Function(function) = sum.components().items()[0].view() else {
        panic!("outer calc")
    };
    let children = function.values().items();
    assert_eq!(&children[..1], first_components.items());
    assert_eq!(&children[4..], second_components.items());
    same_origin(children[0].origin(), &first_origin);
    same_origin(children[4].origin(), &second_origin);
    for child in &children[1..4] {
        assert_eq!(child.origin(), &CssValueOrigin::Programmatic);
    }
    assert_eq!(function.closing_origin(), &CssValueOrigin::Programmatic);
    let CssCalculationExpressionRef::NestedCalc(calc) = sum.expression() else {
        panic!("calc")
    };
    let CssCalculationExpressionRef::Sum(terms) = calc.operand() else {
        panic!("subtraction")
    };
    for (index, expected) in [(0, &first_origin), (1, &second_origin)] {
        let CssCalculationExpressionRef::Value(value) = terms.term(index).unwrap().expression()
        else {
            panic!("exact leaf")
        };
        assert_eq!(value.literal().representation(), "+001.5");
        assert_eq!(value.literal().unit(), Some("PX"));
        same_origin(value.literal().origin(), expected);
    }
    assert_eq!(
        terms.term(1).unwrap().operator_origin(),
        Some(&CssValueOrigin::Programmatic)
    );
    let serialized = sum.serialize().unwrap();
    assert_eq!(serialized.as_css(), "calc(+001.5px - +001.5px)");
    assert_eq!(
        sum.components().serialize().unwrap().as_css(),
        "calc(+001.5PX - +001.5PX)"
    );
    for (offset, expected) in [(5, &first_origin), (17, &second_origin)] {
        let Some(CssSerializedOrigin::Token(actual)) = serialized.origin_at(offset) else {
            panic!("original operand output")
        };
        same_origin(actual, expected);
    }
}

#[test]
fn original_operand_trivia_is_retained_even_when_numeric_output_normalizes_it() {
    let first = operand(" /*😀*/ 1px ");
    let original = first.components().clone();
    let sum = Calculation::try_sum(first, []).unwrap();
    let CssComponentValueRef::Function(function) = sum.components().items()[0].view() else {
        panic!("calc")
    };
    assert_eq!(function.values(), &original);
    assert_eq!(sum.serialize().unwrap().as_css(), "calc(1px)");
}

#[test]
fn root_only_unitless_zero_is_rechecked_as_an_arithmetic_operand() {
    for zero in ["0", "-0", "0e2"] {
        let error = Calculation::try_sum(operand(zero), []).unwrap_err();
        assert_eq!(
            error.kind(),
            &CssNumericConstructionErrorKind::RootDomainMismatch
        );
        for (first, rest) in [
            (operand(zero), operand("1px")),
            (operand("1px"), operand(zero)),
        ] {
            let error = Calculation::try_sum(first, [(Op::Add, rest)]).unwrap_err();
            assert_eq!(
                error.kind(),
                &CssNumericConstructionErrorKind::IncompatibleTypes
            );
        }
    }
    assert_eq!(
        Calculation::try_sum(operand("0px"), [(Op::Subtract, operand("1px"))])
            .unwrap()
            .serialize()
            .unwrap()
            .as_css(),
        "calc(0px - 1px)"
    );
}

#[test]
fn trusted_parsed_recovery_remains_in_the_child_while_strict_component_admission_rejects_it() {
    use surgeist_css::{
        CssCalcLength, CssFlowToleranceRef, CssKnownPropertyValueRef, CssLength,
        parse_style_attribute,
    };
    let report = parse_style_attribute("flow-tolerance:calc(1px");
    assert!(!report.is_clean());
    let CssKnownPropertyValueRef::FlowTolerance(value) = report.syntax()[0]
        .known()
        .unwrap()
        .property_value()
        .unwrap()
    else {
        panic!("flow tolerance")
    };
    let CssFlowToleranceRef::LengthPercentage(CssLength::Calc(CssCalcLength::Typed(child))) =
        value.value().as_ref()
    else {
        panic!("trusted recovered typed child")
    };
    let components = child.components().clone();
    let CssComponentValueRef::Function(function) = components.items()[0].view() else {
        panic!("calc")
    };
    let closure = function.closing_origin().clone();
    assert!(matches!(closure, CssValueOrigin::ImplicitClosure { .. }));
    let sum = Calculation::try_sum(child.clone(), [(Op::Add, operand("2px"))]).unwrap();
    assert_eq!(sum.serialize().unwrap().as_css(), "calc(calc(1px) + 2px)");
    let CssComponentValueRef::Function(outer) = sum.components().items()[0].view() else {
        panic!("outer")
    };
    assert_eq!(outer.values().items()[0], components.items()[0]);
    let CssComponentValueRef::Function(inner) = outer.values().items()[0].view() else {
        panic!("inner")
    };
    assert_eq!(inner.closing_origin(), &closure);
    assert_eq!(
        Calculation::try_from_components(sum.components().clone())
            .unwrap_err()
            .kind(),
        &CssNumericConstructionErrorKind::RecoveredComponent
    );
}

#[test]
fn inserted_wrapper_and_separators_are_included_in_exact_component_and_byte_limits() {
    // calc(1px + 2px): one function, two operands, and three separator tokens.
    let exact = CssComponentValueLimits::try_new(1, 6, 15).unwrap();
    let sum = Calculation::try_sum_with_limits(operand("1px"), [(Op::Add, operand("2px"))], exact)
        .unwrap();
    assert_eq!(sum.serialize().unwrap().as_css(), "calc(1px + 2px)");
    for (limits, kind) in [
        (
            CssComponentValueLimits::try_new(0, 6, 15).unwrap(),
            CssComponentValueErrorKind::NestingLimit,
        ),
        (
            CssComponentValueLimits::try_new(1, 5, 15).unwrap(),
            CssComponentValueErrorKind::ComponentLimit,
        ),
        (
            CssComponentValueLimits::try_new(1, 6, 14).unwrap(),
            CssComponentValueErrorKind::ByteLimit,
        ),
    ] {
        let error =
            Calculation::try_sum_with_limits(operand("1px"), [(Op::Add, operand("2px"))], limits)
                .unwrap_err();
        resource(&error, kind);
    }
    let error = Calculation::try_sum_with_limits(
        operand("1px"),
        [],
        CssComponentValueLimits::try_new(0, usize::MAX, usize::MAX).unwrap(),
    )
    .unwrap_err();
    resource(&error, CssComponentValueErrorKind::NestingLimit);
    assert_eq!(error.origin(), Some(&CssValueOrigin::Programmatic));
}

#[test]
fn a_supported_child_depth_cannot_silently_exceed_the_aggregate_wrapper_depth() {
    let source = format!("{}1px{}", "calc(".repeat(256), ")".repeat(256));
    let child = operand(&source);
    let error = Calculation::try_sum(child, []).unwrap_err();
    resource(&error, CssComponentValueErrorKind::NestingLimit);
    let source = format!("{}1px{}", "calc(".repeat(255), ")".repeat(255));
    let sum = Calculation::try_sum(operand(&source), []).unwrap();
    assert_eq!(sum.serialize().unwrap().as_css(), format!("calc({source})"));
}

#[test]
fn finite_component_limits_stop_consuming_a_finite_iterator_at_the_first_overflow() {
    use std::cell::Cell;
    let consumed = Cell::new(0);
    let rest = std::iter::repeat_with(|| {
        consumed.set(consumed.get() + 1);
        (Op::Add, operand("2px"))
    })
    .take(100);
    // First operand + outer wrapper use two components. One additional operand
    // would add four, exceeding this budget before any further operand is read.
    let error = Calculation::try_sum_with_limits(
        operand("1px"),
        rest,
        CssComponentValueLimits::try_new(256, 2, usize::MAX).unwrap(),
    )
    .unwrap_err();
    resource(&error, CssComponentValueErrorKind::ComponentLimit);
    assert_eq!(consumed.get(), 1);
}

#[test]
fn a_large_iterator_upper_bound_is_not_treated_as_an_actual_allocation_requirement() {
    struct LooseUpperBound(std::option::IntoIter<(Op, Calculation)>);
    impl Iterator for LooseUpperBound {
        type Item = (Op, Calculation);
        fn next(&mut self) -> Option<Self::Item> {
            self.0.next()
        }
        fn size_hint(&self) -> (usize, Option<usize>) {
            (0, Some(usize::MAX))
        }
    }
    // This upper bound is truthful but loose; there is exactly one operand.
    let rest = LooseUpperBound(Some((Op::Add, operand("2px"))).into_iter());
    let sum = Calculation::try_sum(operand("1px"), rest).unwrap();
    assert_eq!(sum.serialize().unwrap().as_css(), "calc(1px + 2px)");
}

#[test]
fn the_complete_wrapper_byte_budget_is_checked_before_consuming_another_operand() {
    use std::cell::Cell;
    let consumed = Cell::new(0);
    let rest = std::iter::once_with(|| {
        consumed.set(consumed.get() + 1);
        (Op::Add, operand("2px"))
    });
    // Five bytes for calc(, three for 1px, and one closing parenthesis.
    let tight = CssComponentValueLimits::try_new(256, usize::MAX, 8).unwrap();
    let error = Calculation::try_sum_with_limits(operand("1px"), rest, tight).unwrap_err();
    resource(&error, CssComponentValueErrorKind::ByteLimit);
    assert_eq!(consumed.get(), 0);
    let exact = CssComponentValueLimits::try_new(256, usize::MAX, 9).unwrap();
    let sum = Calculation::try_sum_with_limits(operand("1px"), [], exact).unwrap();
    assert_eq!(sum.serialize().unwrap().as_css(), "calc(1px)");
}

#[test]
fn normalized_operator_spacing_is_budgeted_before_consuming_another_operand() {
    use std::cell::Cell;
    let consumed = Cell::new(0);
    let rest = std::iter::once_with(|| {
        consumed.set(consumed.get() + 1);
        (Op::Add, operand("2px"))
    });
    // Original assembled components need 17 bytes; numeric serialization inserts
    // two spaces around the product operator and therefore needs 19 bytes.
    let tight = CssComponentValueLimits::try_new(256, usize::MAX, 18).unwrap();
    let error = Calculation::try_sum_with_limits(operand("calc(1px*2)"), rest, tight).unwrap_err();
    resource(&error, CssComponentValueErrorKind::ByteLimit);
    assert_eq!(consumed.get(), 0);
    let exact = CssComponentValueLimits::try_new(256, usize::MAX, 19).unwrap();
    let sum = Calculation::try_sum_with_limits(operand("calc(1px*2)"), [], exact).unwrap();
    assert_eq!(sum.serialize().unwrap().as_css(), "calc(calc(1px * 2))");
    assert_eq!(
        sum.components().serialize().unwrap().as_css(),
        "calc(calc(1px*2))"
    );
}
