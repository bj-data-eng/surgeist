//! Exact retained payloads remain owned; finite RGB output uses Color4 legacy commas.
#![forbid(unsafe_code)]

use stats_alloc::{INSTRUMENTED_SYSTEM, Region, StatsAlloc};
use std::alloc::System;
use surgeist_css::*;

#[global_allocator]
static GLOBAL: &StatsAlloc<System> = &INSTRUMENTED_SYSTEM;

fn color(declaration: &CssDeclaration) -> &CssColor {
    let CssKnownPropertyValueRef::Color(value) =
        declaration.known().unwrap().property_value().unwrap()
    else {
        panic!("owned color result");
    };
    value.value()
}

fn checked(values: CssComponentValues) -> CssDeclaration {
    parse_property_value(
        CssPropertyNameRef::Known(CssKnownProperty::Color),
        values,
        CssImportance::Normal,
    )
    .unwrap()
}

#[test]
fn constructed_result_retains_mixed_originals_after_inputs_and_another_result_drop() {
    let (first, second, expected) = {
        let source = String::from("calc(1 + max(2, min(3,4))) 5 6");
        let children = parse_component_values(&source).unwrap();
        let CssComponentValueRef::Function(calc) = children.items()[0].view() else {
            panic!("exact nested root");
        };
        let expected = calc.values().items()[0].origin().clone();
        let values = CssComponentValues::try_new(vec![
            CssComponentValue::try_function("rgb", children).unwrap(),
        ])
        .unwrap();
        (checked(values.clone()), checked(values), expected)
    };
    drop(first);
    let CssColorComponent::NumberCalculation(value) =
        &color(&second).rgb_value().unwrap().channels()[0]
    else {
        panic!("independently owned exact calculation");
    };
    assert_eq!(value.origin(), value.components().items()[0].origin());
    let CssValueOrigin::Parsed(expected) = expected else {
        panic!("actual child snapshot");
    };
    let CssComponentValueRef::Function(calc) = value.components().items()[0].view() else {
        panic!("retained calc graph");
    };
    let CssValueOrigin::Parsed(actual) = calc.values().items()[0].origin() else {
        panic!("original leaf");
    };
    assert!(actual.source().same_snapshot(expected.source()));
    assert_eq!(actual.span(), expected.span());
    assert_eq!(actual.span().start().byte_offset().value(), 5);
    assert_eq!(
        value.components().serialize().unwrap().as_css(),
        "calc(1 + max(2, min(3,4)))"
    );
    assert_eq!(color(&second).to_specified_css().unwrap(), "rgb(4, 5, 6)");
}

#[test]
fn profile_result_exposes_one_retained_calculation_graph_after_source_drop() {
    let result = {
        let text = String::from("calc(Cyan + max(Magenta, 1))");
        CssProfileColorExpression::try_from_components(parse_component_values(&text).unwrap())
            .unwrap()
    };
    let CssProfileColorExpressionRef::Calculation(calculation) = result.view() else {
        panic!("profile calculation");
    };
    assert_eq!(
        calculation
            .references()
            .iter()
            .map(|name| name.as_str())
            .collect::<Vec<_>>(),
        ["Cyan", "Magenta"]
    );
    assert!(std::ptr::eq(result.components(), calculation.components()));
    assert_eq!(
        result.components().serialize().unwrap().as_css(),
        "calc(Cyan + max(Magenta, 1))"
    );
}

#[test]
fn recovered_source_closures_are_retained_and_checked_property_admission_stays_strict() {
    let source = "color:rgb(calc(1 + 2) 3 4";
    let report = parse_style_attribute(source);
    assert_eq!(report.syntax().len(), 1);
    let value = color(&report.syntax()[0]);
    let CssComponentValueRef::Function(root) =
        report.syntax()[0].value_components().items()[0].view()
    else {
        panic!("retained source function");
    };
    assert!(matches!(
        root.closing_origin(),
        CssValueOrigin::ImplicitClosure { .. }
    ));
    assert_eq!(value.to_specified_css().unwrap(), "rgb(3, 3, 4)");
    let values = parse_component_values("rgb(calc(1 + 2) 3 4").unwrap();
    assert!(
        parse_property_value(
            CssPropertyNameRef::Known(CssKnownProperty::Color),
            values,
            CssImportance::Normal
        )
        .is_err()
    );
}

#[test]
fn report_completed_public_color_construction_allocation_separately_from_transport() {
    const TEST: &str =
        "report_completed_public_color_construction_allocation_separately_from_transport";
    const CHILD: &str = "SURGEIST_CSS_COMPONENT_BORROW_PUBLIC_ALLOCATION_CHILD";
    if std::env::var(CHILD).as_deref() != Ok(TEST) {
        let output = std::process::Command::new(std::env::current_exe().unwrap())
            .args(["--exact", TEST, "--nocapture"])
            .env(CHILD, TEST)
            .stdin(std::process::Stdio::null())
            .output()
            .unwrap();
        assert!(
            output.status.success(),
            "{}\n{}\n{}",
            output.status,
            String::from_utf8_lossy(&output.stdout),
            String::from_utf8_lossy(&output.stderr)
        );
        print!(
            "{}{}",
            String::from_utf8_lossy(&output.stdout),
            String::from_utf8_lossy(&output.stderr)
        );
        return;
    }
    let values = parse_component_values("rgb(calc(1 + max(2, min(3,4))) 5 6)").unwrap();
    let region = Region::new(GLOBAL);
    let result = checked(values);
    let stats = region.change();
    assert_eq!(color(&result).to_specified_css().unwrap(), "rgb(4, 5, 6)");
    eprintln!(
        "completed public nested RGB: allocations={}, reallocations={}, bytes_allocated={}, bytes_reallocated={}; includes required owned public result retention",
        stats.allocations, stats.reallocations, stats.bytes_allocated, stats.bytes_reallocated
    );
}

#[test]
fn relative_calculation_retains_required_authored_text_and_references_after_input_drop() {
    let result = {
        let text = String::from("calc(r + .000000000000001)");
        CssRelativeColorExpression::try_from_components(
            parse_component_values(&text).unwrap(),
            CssRelativeColorEnvironment::Rgb,
            CssRelativeColorResultDomain::NumberPercentage,
        )
        .unwrap()
    };
    let CssRelativeColorExpressionValue::Calculation(calculation) = result.value() else {
        panic!("relative exact calculation");
    };
    assert_eq!(
        calculation.authored().as_css(),
        "calc(r + .000000000000001)"
    );
    assert_eq!(calculation.references(), &[CssRelativeColorChannel::R]);
    assert_eq!(calculation.result_type(), CssCalculationType::Number);
    assert!(matches!(result.origin(), CssValueOrigin::Parsed(_)));
}
