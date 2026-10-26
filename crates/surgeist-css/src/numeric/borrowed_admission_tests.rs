//! An isolated child keeps global allocation counters separate from other tests.

#![forbid(unsafe_code)]

use super::NumericInputContext;
use crate::{
    CssComponentValue, CssComponentValueRef, CssComponentValues, CssParsedOrigin,
    CssSourceSnapshot, CssValueOrigin, CssValueTokenRef, parse_component_values,
};
use stats_alloc::{INSTRUMENTED_SYSTEM, Region, StatsAlloc};
use std::alloc::System;

#[global_allocator]
static GLOBAL: &StatsAlloc<System> = &INSTRUMENTED_SYSTEM;

fn first_number(root: &CssComponentValue) -> &CssComponentValue {
    let CssComponentValueRef::Function(rgb) = root.view() else {
        panic!("RGB root");
    };
    let CssComponentValueRef::Function(calc) = rgb.values().items()[0].view() else {
        panic!("nested calc operand");
    };
    &calc.values().items()[0]
}

fn parsed_origin(component: &CssComponentValue) -> &CssParsedOrigin {
    let CssValueOrigin::Parsed(origin) = component.origin() else {
        panic!("original parsed occurrence");
    };
    origin
}

fn component_css(component: &CssComponentValue) -> String {
    CssComponentValues::try_new(vec![component.clone()], Default::default())
        .unwrap()
        .serialize_tokens(Default::default())
        .unwrap()
        .as_css()
        .to_owned()
}

#[test]
fn mapped_collection_lends_root_without_transport_allocation() {
    const TEST: &str = "numeric::borrowed_admission_tests::mapped_collection_lends_root_without_transport_allocation";
    const CHILD: &str = "SURGEIST_CSS_COMPONENT_BORROW_ALLOCATION_CHILD";
    const COMPLETED: &str = "mapped collection transport allocation checked";
    crate::test_support::run_in_isolated_child(TEST, CHILD, COMPLETED, check_mapped_collection);
}

fn check_mapped_collection() {
    const CHILDREN: &str = "calc(1 + max(2, min(3,4))) 5 6";
    const ROOT: &str = "rgb(calc(1 + max(2, min(3,4))) 5 6)";
    const SOURCE: &str = "rgb(calc(1 + max(2, min(3,4))) 5 6) blue";

    // The source route actually collects an owned graph. Its allocations are
    // legitimate source admission and are outside the mapped-transport region.
    let source = CssSourceSnapshot::new(SOURCE);
    let mut source_input = cssparser::ParserInput::new(SOURCE);
    let mut source_cursor = cssparser::Parser::new(&mut source_input);
    let source_root = NumericInputContext::Parsed(&source)
        .collect(&mut source_cursor)
        .unwrap();
    assert_eq!(source_cursor.expect_ident().unwrap().as_ref(), "blue");
    assert!(source_cursor.is_exhausted());
    let source_leaf = first_number(&source_root);
    let origin = parsed_origin(source_leaf);
    assert!(origin.source().same_snapshot(&source));
    assert_eq!(origin.span().start().byte_offset().value(), 9);
    assert_eq!(origin.span().end().byte_offset().value(), 10);

    // A programmatic root with actual parsed children exercises mixed origins.
    // All fixture construction, validation, serialization and parser setup stay
    // outside the measured call. Ordinary ASCII names need no escape decoding.
    let children = parse_component_values(CHILDREN).unwrap();
    let root = CssComponentValue::try_function("rgb", children).unwrap();
    let expected_leaf_origin = parsed_origin(first_number(&root)).clone();
    let values = CssComponentValues::try_new(
        vec![root, CssComponentValue::try_ident("blue").unwrap()],
        Default::default(),
    )
    .unwrap();
    let serialized = values.serialize_tokens(Default::default()).unwrap();
    let context = NumericInputContext::Components(&values, &serialized);
    let mut input = cssparser::ParserInput::new(serialized.as_css());
    let mut cursor = cssparser::Parser::new(&mut input);

    let region = Region::new(GLOBAL);
    let collected = context.collect(&mut cursor).unwrap();
    let allocations = region.change();

    // This coercion works for the existing owned component and for a borrowed
    // ownership carrier. The witness never requires a future production API.
    let observed: &CssComponentValue = &collected;
    assert_eq!(observed.origin(), &CssValueOrigin::Programmatic);
    assert_eq!(observed, &values.items()[0]);
    let leaf = first_number(observed);
    let CssComponentValueRef::Token(CssValueTokenRef::Number(number)) = leaf.view() else {
        panic!("exact number token");
    };
    assert_eq!(number.representation(), "1");
    let origin = parsed_origin(leaf);
    assert!(origin.source().same_snapshot(expected_leaf_origin.source()));
    assert_eq!(origin.source().as_str(), CHILDREN);
    assert_eq!(origin.span(), expected_leaf_origin.span());
    assert_eq!(origin.span().start().byte_offset().value(), 5);
    assert_eq!(origin.span().end().byte_offset().value(), 6);
    assert_eq!(cursor.expect_ident().unwrap().as_ref(), "blue");
    assert!(cursor.is_exhausted());
    assert_eq!(component_css(&source_root), ROOT);
    assert_eq!(component_css(observed), ROOT);

    let same_root = std::ptr::eq(observed, &values.items()[0]);
    eprintln!(
        "mapped nested component collection: allocations={}, reallocations={}, bytes_allocated={}, bytes_reallocated={}, original_root={same_root}; source cursor, mapped cursor, exact spelling and mixed snapshot origins passed",
        allocations.allocations,
        allocations.reallocations,
        allocations.bytes_allocated,
        allocations.bytes_reallocated,
    );
    assert_eq!(
        (
            allocations.allocations,
            allocations.reallocations,
            allocations.bytes_allocated,
            allocations.bytes_reallocated,
        ),
        (0, 0, 0, 0),
        "mapped collection must lend its checked subtree without allocating a transport copy",
    );
    assert!(
        same_root,
        "mapped collection must inspect the original node"
    );
}

#[test]
fn read_only_component_check_skips_input_graph_retention() {
    const TEST: &str =
        "numeric::borrowed_admission_tests::read_only_component_check_skips_input_graph_retention";
    const CHILD: &str = "SURGEIST_CSS_READ_ONLY_CALCULATION_CHECK_CHILD";
    const COMPLETED: &str = "read-only calculation retention checked";
    crate::test_support::run_in_isolated_child(TEST, CHILD, COMPLETED, check_read_only_retention);
}

fn check_read_only_retention() {
    // Input construction and retention assertions remain outside the measured
    // call. The calculation needs integer-domain validation, not a public result.
    let values = [128, 65_536].map(|length| {
        parse_component_values(&format!("calc(1 /*{}*/ + 2)", "x".repeat(length))).unwrap()
    });
    for contextual in [false, true] {
        let stats = values.each_ref().map(|values| {
            let region = Region::new(GLOBAL);
            let result = if contextual {
                NumericInputContext::Checked
                    .check_component(&values.items()[0], super::CalculationRoot::Integer)
            } else {
                super::check_component(&values.items()[0], super::CalculationRoot::Integer)
            };
            let stats = region.change();
            result.expect("a sum of integer literals is an integer-domain calculation");
            stats
        });
        let copied = |stats: stats_alloc::Stats| {
            stats.bytes_allocated + stats.bytes_reallocated.max(0) as usize
        };
        let growth = copied(stats[1]).saturating_sub(copied(stats[0]));
        assert!(
            growth <= 1024,
            "read-only check retained comment payload: contextual={contextual}, growth={growth}, stats={stats:?}"
        );
    }
    let invalid = parse_component_values("calc(1px + 2px)").unwrap();
    let error = super::check_component(&invalid.items()[0], super::CalculationRoot::Integer)
        .expect_err("a length sum cannot enter the integer domain");
    assert_eq!(
        error.kind(),
        &crate::CssNumericConstructionErrorKind::RootDomainMismatch
    );
}

#[test]
fn read_only_component_check_preserves_context_closure_policy() {
    let source = CssSourceSnapshot::new("calc(1 + 2");
    let mut input = cssparser::ParserInput::new(source.as_str());
    let mut cursor = cssparser::Parser::new(&mut input);
    let parsed = NumericInputContext::Parsed(&source);
    let component = parsed.collect(&mut cursor).unwrap();
    parsed
        .check_component(&component, super::CalculationRoot::Integer)
        .expect("source mathematics admits its recovered closing boundary");
    NumericInputContext::QuirkyLengths(&parsed)
        .check_component(&component, super::CalculationRoot::Integer)
        .expect("length quirks preserve the source closure policy");
    for error in [
        super::check_component(&component, super::CalculationRoot::Integer).unwrap_err(),
        NumericInputContext::Checked
            .check_component(&component, super::CalculationRoot::Integer)
            .unwrap_err(),
    ] {
        assert_eq!(
            error.kind(),
            &crate::CssNumericConstructionErrorKind::RecoveredComponent
        );
    }
    let values =
        CssComponentValues::try_new(vec![component.into_owned()], Default::default()).unwrap();
    let serialized = values.serialize_tokens(Default::default()).unwrap();
    let error = NumericInputContext::Components(&values, &serialized)
        .check_component(&values.items()[0], super::CalculationRoot::Integer)
        .unwrap_err();
    assert_eq!(
        error.kind(),
        &crate::CssNumericConstructionErrorKind::RecoveredComponent
    );
}

#[test]
fn numerical_attempt_moves_owned_input_and_returns_it_for_failed_retry() {
    let context = NumericInputContext::Checked;
    let values = parse_component_values("calc(1fr + 2fr)").unwrap();
    let original = values.items().as_ptr();
    let expression = context
        .admit_or_return_input(
            super::CalculationComponents::Owned(values),
            super::CalculationRoot::Flex,
        )
        .unwrap();
    assert_eq!(
        expression.components.as_ref().unwrap().items().as_ptr(),
        original
    );
    assert_eq!(
        crate::CssCalculation::<crate::CssFlex>::from_expression(expression)
            .serialize_tokens(Default::default())
            .unwrap()
            .as_css(),
        "calc(1fr + 2fr)"
    );

    let values = parse_component_values("calc(1px + 2px)").unwrap();
    let original = values.items().as_ptr();
    let failure = context
        .admit_or_return_input(
            super::CalculationComponents::Owned(values),
            super::CalculationRoot::Flex,
        )
        .unwrap_err();
    assert_eq!(
        failure.error.kind(),
        &crate::CssNumericConstructionErrorKind::RootDomainMismatch
    );
    assert_eq!(failure.error.path(), Some([0].as_slice()));
    let super::CalculationComponents::Owned(ref returned) = failure.values else {
        panic!("the original owner is returned on failure");
    };
    assert_eq!(returned.items().as_ptr(), original);
    let expression = context
        .admit_or_return_input(failure.values, super::CalculationRoot::LengthPercentage)
        .unwrap();
    assert_eq!(
        expression.components.as_ref().unwrap().items().as_ptr(),
        original
    );
    assert_eq!(
        crate::CssCalculation::<crate::CssLengthPercentage>::from_expression(expression)
            .serialize_tokens(Default::default())
            .unwrap()
            .as_css(),
        "calc(1px + 2px)"
    );
}

#[test]
fn numerical_attempt_keeps_borrowed_failure_identity_and_owns_completed_output() {
    let context = NumericInputContext::Checked;
    let values = parse_component_values("calc(1px + 2px)").unwrap();
    let failure = context
        .admit_or_return_input(
            super::CalculationComponents::Borrowed(values.items()),
            super::CalculationRoot::Flex,
        )
        .unwrap_err();
    assert_eq!(
        failure.error.kind(),
        &crate::CssNumericConstructionErrorKind::RootDomainMismatch
    );
    let super::CalculationComponents::Borrowed(returned) = failure.values else {
        panic!("borrowed failure keeps the same actual components");
    };
    assert!(std::ptr::eq(returned, values.items()));
    let expression = context
        .admit_or_return_input(
            super::CalculationComponents::Borrowed(returned),
            super::CalculationRoot::LengthPercentage,
        )
        .unwrap();
    let output = expression.components.as_ref().unwrap();
    assert_ne!(output.items().as_ptr(), values.items().as_ptr());
    assert!(
        parsed_origin(&output.items()[0])
            .source()
            .same_snapshot(parsed_origin(&values.items()[0]).source())
    );
    drop(values);
    assert_eq!(
        crate::CssCalculation::<crate::CssLengthPercentage>::from_expression(expression)
            .serialize_tokens(Default::default())
            .unwrap()
            .as_css(),
        "calc(1px + 2px)"
    );
}
