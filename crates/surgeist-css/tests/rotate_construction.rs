#![forbid(unsafe_code)]
//! Authored Transforms2 §5 angle/axis construction, identity and provenance.
//! These tests deliberately exercise no canonical rotate output or matrix execution.
use surgeist_css::*;

fn angle(text: &str) -> CssAngleValue {
    CssAngleValue::from_literal(CssAngleLiteral::try_new(text, CssAngleUnit::Degrees).unwrap())
}
fn number(text: &str) -> CssSpecifiedNumber {
    CssSpecifiedNumber::try_from_component(CssComponentValue::try_number(text).unwrap()).unwrap()
}
fn parsed(source: &str) -> CssRotate {
    let report = parse_style_attribute(source);
    assert!(report.is_clean(), "{source}: {:?}", report.diagnostics());
    let declaration = &report.syntax()[0];
    let CssKnownPropertyValueRef::Rotate(value) =
        declaration.known().unwrap().property_value().unwrap()
    else {
        panic!("ordinary rotate");
    };
    value.value().clone()
}
fn values(value: &CssRotate) -> &CssRotateValues {
    let CssRotate::Value(value) = value else {
        panic!("rotation distinct from none");
    };
    value
}
fn assert_origin(origin: &CssValueOrigin, start: usize, end: usize) {
    let CssValueOrigin::Parsed(origin) = origin else {
        panic!("retained parsed origin");
    };
    assert_eq!(origin.span().start().byte_offset().value(), start);
    assert_eq!(origin.span().end().byte_offset().value(), end);
}

#[test]
fn construction_keeps_none_identity_and_optional_keyword_axis_distinct() {
    let implicit = CssRotateValues::new(angle("0"), None);
    assert!(implicit.axis().is_none());
    assert!(implicit.keyword_axis_origin().is_none());
    assert_eq!(
        implicit
            .angle()
            .literal()
            .unwrap()
            .numeric()
            .representation(),
        "0"
    );
    assert_eq!(implicit.angle().origin(), &CssValueOrigin::Programmatic);
    assert_ne!(CssRotate::None, CssRotate::Value(implicit.clone()));
    for axis in [CssRotateAxis::X, CssRotateAxis::Y, CssRotateAxis::Z] {
        let explicit = CssRotateValues::new(angle("0"), Some(axis.clone()));
        assert_eq!(explicit.axis(), Some(&axis));
        assert_eq!(
            explicit.keyword_axis_origin(),
            Some(&CssValueOrigin::Programmatic)
        );
        assert_ne!(explicit, implicit);
    }
}

#[test]
fn vector_construction_preserves_exact_signed_zero_and_extreme_operands() {
    for components in [["0", "-0", "+0"], ["-1", "1e999", "1e-999"]] {
        let value = CssRotateValues::new(
            angle("45"),
            Some(CssRotateAxis::Vector(components.map(number))),
        );
        let Some(CssRotateAxis::Vector(actual)) = value.axis() else {
            panic!("vector");
        };
        assert!(value.keyword_axis_origin().is_none());
        for (actual, expected) in actual.iter().zip(components) {
            let component = actual.literal_component().unwrap();
            let CssComponentValueRef::Token(CssValueTokenRef::Number(number)) = component.view()
            else {
                panic!("number");
            };
            assert_eq!(number.representation(), expected);
            assert_eq!(actual.origin(), &CssValueOrigin::Programmatic);
        }
    }
}

#[test]
fn typed_math_construction_keeps_angle_and_number_domains_symbolic() {
    let angle_calculation = CssAngleCalculation::try_from_components(
        parse_component_values("calc(15deg + 15deg)").unwrap(),
    )
    .unwrap();
    let angle_value = CssAngleValue::try_from_calculation(angle_calculation.clone()).unwrap();
    let axis_calculation =
        CssNumberCalculation::try_from_components(parse_component_values("calc(1 + 1)").unwrap())
            .unwrap();
    let x = CssSpecifiedNumber::try_from_calculation(axis_calculation.clone()).unwrap();
    let value = CssRotateValues::new(
        angle_value,
        Some(CssRotateAxis::Vector([x, number("0"), number("0")])),
    );
    assert_eq!(value.angle().calculation(), Some(&angle_calculation));
    let Some(CssRotateAxis::Vector(axis)) = value.axis() else {
        panic!("vector");
    };
    assert_eq!(axis[0].calculation(), Some(&axis_calculation));
    assert!(value.angle().literal().is_none());
    assert!(axis[0].literal_component().is_none());
}

#[test]
fn structural_equality_ignores_source_origins_but_retains_axis_and_operand_identity() {
    let first = parsed("rotate:x 30deg");
    let second = parsed("/*😀*/rotate:30deg x");
    assert_eq!(first, second);
    assert_ne!(
        values(&first).angle().origin(),
        values(&second).angle().origin()
    );
    assert_ne!(
        values(&first).keyword_axis_origin(),
        values(&second).keyword_axis_origin()
    );
    assert_ne!(first, parsed("rotate:y 30deg"));
    assert_ne!(first, parsed("rotate:x 31deg"));
    let vector = parsed("rotate:1 2 3 30deg");
    assert_eq!(vector, parsed("/*😀*/rotate:30deg 1 2 3"));
    assert_ne!(vector, parsed("rotate:1 2 4 30deg"));
}

#[test]
fn parsed_keyword_and_numeric_operands_keep_original_utf8_spans() {
    let source = "/*😀*/rotate:X 30deg";
    let rotate = parsed(source);
    let value = values(&rotate);
    let keyword = source.find('X').unwrap();
    let coefficient = source.find("30deg").unwrap();
    assert_origin(value.keyword_axis_origin().unwrap(), keyword, keyword + 1);
    assert_origin(value.angle().origin(), coefficient, coefficient + 5);
    let source = "/*😀*/rotate:30deg -1 2 3";
    let rotate = parsed(source);
    let value = values(&rotate);
    let Some(CssRotateAxis::Vector(axis)) = value.axis() else {
        panic!("vector");
    };
    for (operand, token) in axis.iter().zip(["-1", "2", "3"]) {
        let start = source.rfind(token).unwrap();
        assert_origin(operand.origin(), start, start + token.len());
    }
    assert!(value.keyword_axis_origin().is_none());
}

#[test]
fn checked_property_reentry_preserves_each_supplied_operand_origin() {
    let components = parse_component_values("1 2 3 calc(15deg + 15deg)").unwrap();
    let constructed = parse_property_value_for_grammar(
        CssKnownProperty::Rotate.grammar(),
        components.clone(),
        CssImportance::Important,
    )
    .unwrap();
    let CssKnownPropertyValueRef::Rotate(wrapper) =
        constructed.known().unwrap().property_value().unwrap()
    else {
        panic!("rotate");
    };
    let value = values(wrapper.value());
    let significant = components
        .items()
        .iter()
        .filter(|item| {
            !matches!(
                item.view(),
                CssComponentValueRef::Token(CssValueTokenRef::Whitespace(_))
            )
        })
        .collect::<Vec<_>>();
    let Some(CssRotateAxis::Vector(axis)) = value.axis() else {
        panic!("vector");
    };
    for (operand, component) in axis.iter().zip(&significant[..3]) {
        assert_eq!(operand.origin(), component.origin());
    }
    assert_eq!(value.angle().origin(), significant[3].origin());
    assert_eq!(constructed.value_components(), &components);
}

#[test]
fn recovered_angle_closure_is_reported_and_cannot_enter_checked_property_admission() {
    let source = "rotate:x calc(15deg + 15deg";
    let report = parse_style_attribute(source);
    assert!(!report.is_clean(), "implicit closing delimiter is recovery");
    let CssKnownPropertyValueRef::Rotate(wrapper) = report.syntax()[0]
        .known()
        .unwrap()
        .property_value()
        .unwrap()
    else {
        panic!("recovered authored rotation");
    };
    assert!(values(wrapper.value()).angle().calculation().is_some());
    let components = parse_component_values("x calc(15deg + 15deg").unwrap();
    assert!(
        parse_property_value(
            CssPropertyNameRef::Known(CssKnownProperty::Rotate),
            components.clone(),
            CssImportance::Normal
        )
        .is_err()
    );
    assert!(
        parse_property_value_for_grammar(
            CssKnownProperty::Rotate.grammar(),
            components,
            CssImportance::Normal
        )
        .is_err()
    );
    let recovered = CssAngleCalculation::try_from_components(
        parse_component_values("calc(15deg + 15deg").unwrap(),
    );
    assert!(
        recovered.is_err(),
        "checked Angle-root construction rejects implicit closure"
    );
}

fn assert_incompatible_angle_diagnostic(value: &str) {
    let source = format!("/*😀*/rotate:{value};color:red");
    let report = parse_style_attribute(&source);
    assert!(
        !report.is_clean(),
        "incompatible angle/length sum: {source}"
    );
    let [retained] = report.syntax().as_slice() else {
        panic!(
            "only the invalid rotation is dropped: {source}: {:?}",
            report.syntax()
        );
    };
    assert_eq!(
        retained.known().unwrap().property(),
        CssKnownProperty::Color
    );
    let [diagnostic] = report.diagnostics() else {
        panic!(
            "one intrinsic numeric diagnostic: {source}: {:?}",
            report.diagnostics()
        );
    };
    assert_eq!(diagnostic.action(), CssRecoveryAction::DropDeclaration);
    let ErrorKind::InvalidPropertyValue(detail) = diagnostic.error().kind() else {
        panic!(
            "typed rotate grammar rejection: {source}: {:?}",
            diagnostic.error()
        );
    };
    assert_eq!(detail.property(), CssKnownProperty::Rotate);
    let encountered = detail
        .encountered()
        .expect("the incompatible right-hand numeric operand");
    assert_eq!(encountered.kind(), CssTokenKind::Dimension, "{source}");
    assert_eq!(encountered.authored(), "1px", "{source}");
    let offending_start = source.find("1px").unwrap();
    assert_eq!(
        diagnostic.error().position().byte_offset().value(),
        offending_start,
        "{source}"
    );
    assert_eq!(diagnostic.error().position().line().value(), 0, "{source}");
    assert_eq!(
        diagnostic.error().position().column().value(),
        source[..offending_start].encode_utf16().count() as u32,
        "{source}"
    );
}

#[test]
fn checked_angle_type_error_identifies_the_incompatible_right_hand_dimension() {
    // Values4 mathematical addition requires compatible types. This existing
    // numeric boundary establishes the specific operand origin to preserve,
    // independently of the rotate parser's speculative alternatives.
    let source = "/*😀*/calc(15deg + 1px)";
    let error = CssAngleCalculation::try_from_components(parse_component_values(source).unwrap())
        .unwrap_err();
    assert_eq!(
        error.kind(),
        &CssNumericConstructionErrorKind::IncompatibleTypes
    );
    let start = source.find("1px").unwrap();
    assert_origin(
        error.origin().expect("incompatible operand origin"),
        start,
        start + 3,
    );
}

#[test]
fn angle_first_numeric_rejection_preserves_the_incompatible_operand_diagnostic() {
    assert_incompatible_angle_diagnostic("calc(15deg + 1px)");
}

#[test]
fn keyword_axis_before_or_after_invalid_angle_preserves_the_numeric_operand_diagnostic() {
    for value in ["x calc(15deg + 1px)", "calc(15deg + 1px) x"] {
        assert_incompatible_angle_diagnostic(value);
    }
}

#[test]
fn vector_axis_before_or_after_invalid_angle_preserves_the_numeric_operand_diagnostic() {
    for value in ["1 2 3 calc(15deg + 1px)", "calc(15deg + 1px) 1 2 3"] {
        assert_incompatible_angle_diagnostic(value);
    }
}
