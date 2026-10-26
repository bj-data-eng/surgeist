use std::error::Error as _;

use super::*;
use crate::{CssValueOrigin, parse_component_values};

fn last_component(text: &str) -> CssComponentValue {
    parse_component_values(text)
        .unwrap()
        .items()
        .last()
        .unwrap()
        .clone()
}

fn closing_origin(component: &CssComponentValue) -> CssValueOrigin {
    let CssComponentValueRef::Function(function) = component.view() else {
        unreachable!("math function")
    };
    function.closing_origin().clone()
}

/// Math whose two children were recovered in independent source snapshots.
fn independently_recovered<T: CssNumericRoot>(
    left: &str,
    right: &str,
    reverse: bool,
) -> (CssCalculation<T>, CssValueOrigin) {
    let left = last_component(left);
    let right = last_component(right);
    let (left_closure, right_closure) = (closing_origin(&left), closing_origin(&right));
    assert_ne!(left_closure, right_closure);
    let source = left.parsed_origin().unwrap().source().clone();
    let (first, second, expected) = if reverse {
        (right, left, right_closure)
    } else {
        (left, right, left_closure)
    };
    let arguments = CssComponentValues::try_new(
        vec![first, CssComponentValue::try_token(",").unwrap(), second],
        Default::default(),
    )
    .unwrap();
    let outer = CssComponentValue::try_function("min", arguments).unwrap();
    let values = CssComponentValues::try_new(vec![outer], Default::default()).unwrap();
    let expression = NumericInputContext::Parsed(&source)
        .admit(values, root_kind::<T>().calculation_root())
        .unwrap();
    (CssCalculation::from_expression(expression), expected)
}

/// Math recovered by one original implicit closure, before any range check.
fn recovered<T: CssNumericRoot>(
    text: &str,
    serialized: &str,
) -> (CssCalculation<T>, CssValueOrigin) {
    let components = parse_component_values(text).unwrap();
    let component = components
        .items()
        .iter()
        .find(|component| matches!(component.view(), CssComponentValueRef::Function(_)))
        .unwrap();
    let first_closure = closing_origin(component);
    assert!(matches!(
        first_closure,
        CssValueOrigin::ImplicitClosure { .. }
    ));
    let source = component.parsed_origin().unwrap().source().clone();
    let expression = NumericInputContext::Parsed(&source)
        .admit(components, root_kind::<T>().calculation_root())
        .unwrap();
    let calculation = CssCalculation::from_expression(expression);
    // The authored writer materializes closings; checked admission must
    // inspect the original graph rather than this serialized output.
    assert_eq!(
        calculation
            .serialize_tokens(Default::default())
            .unwrap()
            .as_css(),
        serialized
    );
    (calculation, first_closure)
}

fn assert_recovered_at<T: CssNumericRoot, R: CssRange<T>>(
    calculation: CssCalculation<T>,
    expected: &CssValueOrigin,
) {
    let original = calculation.clone();
    let error = CssNumeric::<T, R>::try_from_calculation(calculation.clone()).unwrap_err();
    assert!(error.is_recovered_component(), "{error:?}");
    assert_eq!(error.origin(), Some(expected));
    assert!(error.source().is_none());
    assert_eq!(error.path(), None);
    assert_eq!(calculation, original);
}

#[test]
fn independently_recovered_children_report_the_first_original_source_closure() {
    for reverse in [false, true] {
        let (calculation, expected) = independently_recovered::<CssFrequency>(
            "/* first 😀 */\ncalc(-1hz",
            "/* second snapshot */\ncalc(2hz",
            reverse,
        );
        assert_recovered_at::<CssFrequency, CssAnyRange>(calculation, &expected);
        let (calculation, expected) = independently_recovered::<CssResolution>(
            "/* first 😀 */\ncalc(-1dppx",
            "/* second snapshot */\ncalc(2dppx",
            reverse,
        );
        assert_recovered_at::<CssResolution, CssNonNegative>(calculation, &expected);
    }
}

#[test]
fn recovered_original_closure_precedes_value_normalization_and_range_checks() {
    let (calculation, closure) =
        recovered::<CssFrequency>("/* 😀 */\ncalc(min(-1hz, 2hz", "calc(min(-1hz, 2hz))");
    assert_recovered_at::<CssFrequency, CssAnyRange>(calculation, &closure);
    let (calculation, closure) = recovered::<CssResolution>(
        "/* 😀 */\ncalc(min(-1dppx, 2dppx",
        "calc(min(-1dppx, 2dppx))",
    );
    assert_recovered_at::<CssResolution, CssNonNegative>(calculation, &closure);
}

#[test]
fn resolution_children_share_consumed_input_projection_and_output_budgets() {
    use crate::CssSerializationLimits;
    use CssSpecifiedValueSerializationErrorKind as Kind;
    let component = parse_component_values("-0dppx").unwrap().items()[0].clone();
    let value = CssNumericLiteral::<CssResolution>::try_from_component(component).unwrap();
    let mut context = SpecifiedSerializationContext::new(CssSerializationLimits::new(2, 2, 10));
    let mut output = String::new();
    value.append_specified(&mut context, &mut output).unwrap();
    value.append_specified(&mut context, &mut output).unwrap();
    assert_eq!(output, "0dppx0dppx");
    assert_eq!(context.remaining_bytes(), 0);
    assert_eq!(
        value
            .append_specified(&mut context, &mut output)
            .unwrap_err()
            .kind(),
        Kind::InputNodeLimit,
    );
    for (projection, bytes, expected) in
        [(1, 10, Kind::ProjectionNodeLimit), (2, 9, Kind::ByteLimit)]
    {
        let mut context =
            SpecifiedSerializationContext::new(CssSerializationLimits::new(2, projection, bytes));
        let mut output = String::new();
        value.append_specified(&mut context, &mut output).unwrap();
        assert_eq!(
            value
                .append_specified(&mut context, &mut output)
                .unwrap_err()
                .kind(),
            expected,
        );
        assert_eq!(output, "0dppx");
    }
}
