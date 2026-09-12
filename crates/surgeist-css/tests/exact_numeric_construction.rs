//! Checked construction preserves exact tokens and bounded public operations.
use surgeist_css::{
    CssCalculationExpressionRef, CssComponentValue, CssComponentValues, CssNumberCalculation,
    CssNumericConstructionErrorKind, parse_component_values,
};

#[test]
fn adjacent_programmatic_tokens_do_not_turn_into_an_exponent() {
    let components = CssComponentValues::try_new(vec![
        CssComponentValue::try_number("1").unwrap(),
        CssComponentValue::try_ident("e2").unwrap(),
    ])
    .unwrap();
    let error = CssNumberCalculation::try_from_components(components).unwrap_err();
    assert_eq!(
        error.kind(),
        &CssNumericConstructionErrorKind::MultipleValues
    );
}

#[test]
fn empty_and_trailing_values_report_the_responsible_origin() {
    let empty = CssNumberCalculation::try_from_components(parse_component_values(" ").unwrap())
        .unwrap_err();
    assert_eq!(empty.kind(), &CssNumericConstructionErrorKind::EmptyValue);
    assert_eq!(empty.origin(), None);
    let values = parse_component_values("1 2").unwrap();
    let extra = values.items()[2].origin().clone();
    let error = CssNumberCalculation::try_from_components(values).unwrap_err();
    assert_eq!(
        error.kind(),
        &CssNumericConstructionErrorKind::MultipleValues
    );
    assert_eq!(error.origin(), Some(&extra));
}

#[test]
fn exact_numeric_depth_limit_fits_an_ordinary_stack() {
    const CHILD: &str = "SURGEIST_EXACT_NUMERIC_STACK_CHILD";
    const TEST: &str = "exact_numeric_depth_limit_fits_an_ordinary_stack";
    if std::env::var(CHILD).as_deref() == Ok(TEST) {
        let worker = std::thread::Builder::new()
            .stack_size(2 * 1024 * 1024)
            .spawn(|| {
                let source = format!("{}1{}", "calc(".repeat(256), ")".repeat(256));
                let calculation = CssNumberCalculation::try_from_components(
                    parse_component_values(&source).unwrap(),
                )
                .unwrap();
                let cloned = calculation.clone();
                assert_eq!(calculation, cloned);
                assert_eq!(calculation.serialize().unwrap().as_css(), source);
                let mut expression = calculation.expression();
                for _ in 0..256 {
                    let CssCalculationExpressionRef::NestedCalc(inner) = expression else {
                        panic!("expected retained calc")
                    };
                    expression = inner.operand();
                }
                let CssCalculationExpressionRef::Value(value) = expression else {
                    panic!("expected exact leaf")
                };
                assert_eq!(value.literal().representation(), "1");
                drop(cloned);
                drop(calculation);
            })
            .unwrap();
        if let Err(panic) = worker.join() {
            std::panic::resume_unwind(panic);
        }
        println!("numeric stack assertions and destruction completed");
        return;
    }
    let output = std::process::Command::new(std::env::current_exe().unwrap())
        .args(["--exact", TEST, "--nocapture", "--test-threads=1"])
        .env(CHILD, TEST)
        .env_remove("RUST_MIN_STACK")
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
    assert!(
        String::from_utf8_lossy(&output.stdout)
            .contains("numeric stack assertions and destruction completed")
    );
}

#[test]
fn property_bridge_retains_the_programmatic_root_and_original_parsed_children() {
    use surgeist_css::{
        CssImportance, CssKnownProperty, CssKnownPropertyValueRef, CssOpacityValue,
        CssPropertyNameRef, CssValueOrigin, parse_property_value,
    };
    let children = parse_component_values("1e99999 / 2").unwrap();
    let expected_leaf = children.items()[0].origin().clone();
    let values = CssComponentValues::try_new(vec![
        CssComponentValue::try_function("calc", children).unwrap(),
    ])
    .unwrap();
    let declaration = parse_property_value(
        CssPropertyNameRef::Known(CssKnownProperty::Opacity),
        values.clone(),
        CssImportance::Important,
    )
    .unwrap();
    let CssKnownPropertyValueRef::Opacity(opacity) =
        declaration.known().unwrap().property_value().unwrap()
    else {
        panic!("expected opacity")
    };
    let CssOpacityValue::Calculation(calculation) = opacity.value() else {
        panic!("expected retained calculation")
    };
    assert_eq!(calculation.components(), &values);
    assert_eq!(calculation.position(), None);
    assert_eq!(calculation.origin(), &CssValueOrigin::Programmatic);
    let CssCalculationExpressionRef::NestedCalc(root) = calculation.expression() else {
        panic!("expected calc")
    };
    let CssCalculationExpressionRef::Product(product) = root.operand() else {
        panic!("expected division")
    };
    let CssCalculationExpressionRef::Value(leaf) = product.factor(0).unwrap().expression() else {
        panic!("expected exact leaf")
    };
    assert_eq!(leaf.literal().representation(), "1e99999");
    let (CssValueOrigin::Parsed(actual), CssValueOrigin::Parsed(expected)) =
        (leaf.literal().origin(), &expected_leaf)
    else {
        panic!("expected original source")
    };
    assert!(actual.source().same_snapshot(expected.source()));
    assert_eq!(actual.span(), expected.span());
}

#[test]
fn supports_math_keeps_full_stylesheet_coordinates_after_unicode_prefix() {
    use surgeist_css::{
        CssKnownPropertyValueRef, CssOpacityValue, CssRule, CssSupportsConditionKind,
        CssValueOrigin, parse_sheet,
    };
    let source = "/*λ*/\n@supports (opacity: calc(1e99999 / 2)) {}";
    let report = parse_sheet(source);
    assert!(report.is_clean(), "{:?}", report.diagnostics());
    let CssRule::Supports(rule) = &report.syntax().rules()[0] else {
        panic!("expected supports")
    };
    let CssSupportsConditionKind::Declaration(declaration) = rule.condition().kind() else {
        panic!("expected declaration")
    };
    let CssKnownPropertyValueRef::Opacity(opacity) =
        declaration.known().unwrap().property_value().unwrap()
    else {
        panic!("expected opacity")
    };
    let CssOpacityValue::Calculation(calculation) = opacity.value() else {
        panic!("expected math")
    };
    let CssValueOrigin::Parsed(origin) = calculation.origin() else {
        panic!("expected parsed origin")
    };
    assert_eq!(origin.source().as_str(), source);
    assert_eq!(
        origin.span().start().byte_offset().value(),
        source.find("calc(").unwrap()
    );
    let CssCalculationExpressionRef::NestedCalc(root) = calculation.expression() else {
        panic!("expected calc")
    };
    let CssCalculationExpressionRef::Product(product) = root.operand() else {
        panic!("expected quotient")
    };
    let CssCalculationExpressionRef::Value(leaf) = product.factor(0).unwrap().expression() else {
        panic!("expected exact leaf")
    };
    let CssValueOrigin::Parsed(leaf_origin) = leaf.literal().origin() else {
        panic!("expected parsed leaf")
    };
    assert!(leaf_origin.source().same_snapshot(origin.source()));
    assert_eq!(
        leaf_origin.span().start().byte_offset().value(),
        source.find("1e99999").unwrap()
    );
}

#[test]
fn textual_numeric_eof_recovery_retains_the_value_but_checked_construction_rejects_it() {
    use surgeist_css::{
        CssCalcLength, CssErrorCode, CssKnownPropertyValueRef, CssLength, CssLengthCalculation,
        CssRecoveryAction, parse_style_attribute,
    };
    let report = parse_style_attribute("width:calc(1px");
    assert_eq!(report.syntax().len(), 1, "{:?}", report.diagnostics());
    assert!(!report.is_clean());
    let [diagnostic] = report.diagnostics() else {
        panic!("expected one missing-close diagnostic")
    };
    assert_eq!(diagnostic.error().code(), CssErrorCode::UnexpectedEnd);
    assert_eq!(
        diagnostic.action(),
        CssRecoveryAction::RetainWithImplicitClosure
    );
    assert_eq!(
        diagnostic.span().start().byte_offset().value(),
        "width:calc(1px".len()
    );
    assert_eq!(diagnostic.span().start(), diagnostic.span().end());
    assert!(surgeist_css::validate_style_attribute("width:calc(1px").is_err());
    let CssKnownPropertyValueRef::Width(width) = report.syntax()[0]
        .known()
        .unwrap()
        .property_value()
        .unwrap()
    else {
        panic!("expected width")
    };
    let CssLength::Calc(CssCalcLength::Typed(calculation)) = width.i01_subset().unwrap() else {
        panic!("expected exact recovered calculation")
    };
    assert_eq!(calculation.serialize().unwrap().as_css(), "calc(1px)");
    let error =
        CssLengthCalculation::try_from_components(calculation.components().clone()).unwrap_err();
    assert_eq!(
        error.kind(),
        &CssNumericConstructionErrorKind::RecoveredComponent
    );
}

#[test]
fn incompatible_sum_reports_the_original_right_operand() {
    use surgeist_css::{
        CssImportance, CssKnownProperty, CssLengthCalculation, CssPropertyNameRef,
        CssSerializedOrigin, parse_property_value,
    };
    let children = parse_component_values("1px + 1s").unwrap();
    let right = children.items()[4].origin().clone();
    let values = CssComponentValues::try_new(vec![
        CssComponentValue::try_function("calc", children).unwrap(),
    ])
    .unwrap();
    let direct = CssLengthCalculation::try_from_components(values.clone()).unwrap_err();
    assert_eq!(
        direct.kind(),
        &CssNumericConstructionErrorKind::IncompatibleTypes
    );
    assert_eq!(direct.origin(), Some(&right));
    let property = parse_property_value(
        CssPropertyNameRef::Known(CssKnownProperty::Width),
        values,
        CssImportance::Normal,
    )
    .unwrap_err();
    assert_eq!(property.origin(), &CssSerializedOrigin::Token(right));
}

#[test]
fn empty_function_opening_counts_toward_the_nesting_limit_before_grammar() {
    use surgeist_css::CssComponentValueLimits;
    let values = parse_component_values("min()").unwrap();
    assert_eq!(values.nesting_depth(), 1);
    let origin = values.items()[0].origin().clone();
    let limits = CssComponentValueLimits::try_new(0, 8, 64).unwrap();
    let error = CssNumberCalculation::try_from_components_with_limits(values, limits).unwrap_err();
    assert_eq!(
        error.kind(),
        &CssNumericConstructionErrorKind::ResourceLimit
    );
    assert_eq!(error.origin(), Some(&origin));
}
