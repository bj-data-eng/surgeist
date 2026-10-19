#![forbid(unsafe_code)]
//! Shared substitution diagnostics retain genuine missing-argument boundaries.
use surgeist_css::*;

#[test]
fn missing_custom_variable_name_points_to_closing_or_input_end() {
    let name = CssCustomPropertyName::try_new("--x").unwrap();
    for text in ["var()", "var( )", "var(/**/)", "var(", "var( ", "var(/**/"] {
        let report = parse_property_value_text(
            text,
            CssPropertyNameRef::Custom(&name),
            CssImportance::Normal,
        );
        assert!(report.syntax().is_none(), "{text}");
        let diagnostic = report
            .diagnostics()
            .iter()
            .find(|diagnostic| diagnostic.action() == CssRecoveryAction::RejectInput)
            .unwrap();
        assert_eq!(
            diagnostic.error().code(),
            CssErrorCode::UnexpectedEnd,
            "{text}"
        );
        assert_eq!(
            diagnostic.error().position().byte_offset().value(),
            text.find(')').unwrap_or(text.len()),
            "{text}"
        );
    }
}

#[test]
fn checked_missing_variable_name_keeps_original_closing_origin() {
    let name = CssCustomPropertyName::try_new("--x").unwrap();
    for text in ["var()", "var( )", "var(/**/)", "var(", "var( ", "var(/**/"] {
        let values = parse_component_values(text).unwrap();
        let CssComponentValueRef::Function(function) = values.items()[0].view() else {
            panic!("var function")
        };
        let expected = CssSerializedOrigin::Token(function.closing_origin().clone());
        let error = parse_property_value(
            CssPropertyNameRef::Custom(&name),
            values,
            CssImportance::Normal,
        )
        .unwrap_err();
        assert!(
            matches!(
                error.kind(),
                CssPropertyValueErrorKind::Grammar(ErrorKind::UnexpectedEnd(_))
            ),
            "{text}: {error:?}"
        );
        assert_eq!(error.origin(), &expected, "{text}");
    }
    for argument in [" ", "/**/"] {
        let values = CssComponentValues::try_new(vec![
            CssComponentValue::try_function("var", parse_component_values(argument).unwrap())
                .unwrap(),
        ])
        .unwrap();
        let error = parse_property_value(
            CssPropertyNameRef::Custom(&name),
            values,
            CssImportance::Normal,
        )
        .unwrap_err();
        assert_eq!(
            error.origin(),
            &CssSerializedOrigin::Token(CssValueOrigin::Programmatic)
        );
    }
}

#[test]
fn invalid_known_variable_name_preserves_the_responsible_identifier() {
    let text = "display:var(x)";
    let report = parse_style_attribute(text);
    let error = report.diagnostics()[0].error();
    assert_eq!(error.position().byte_offset().value(), 12);
    let ErrorKind::InvalidPropertyValue(detail) = error.kind() else {
        panic!("property context")
    };
    let token = detail.encountered().unwrap();
    assert_eq!(token.kind(), CssTokenKind::Ident);
    assert_eq!(token.authored(), "x");

    let components = parse_component_values("var(x)").unwrap();
    let CssComponentValueRef::Function(function) = components.items()[0].view() else {
        panic!("var")
    };
    let expected = CssSerializedOrigin::Token(function.values().items()[0].origin().clone());
    let error = parse_property_value(
        CssPropertyNameRef::Known(CssKnownProperty::Display),
        components,
        CssImportance::Normal,
    )
    .unwrap_err();
    assert_eq!(error.origin(), &expected);
}

#[test]
fn nested_custom_bad_string_preserves_lexical_category_and_neighbor_recovery() {
    let report = parse_style_attribute("--x:f(\"x\n);display:block");
    assert_eq!(report.diagnostics().len(), 1);
    let diagnostic = &report.diagnostics()[0];
    assert_eq!(diagnostic.action(), CssRecoveryAction::DropDeclaration);
    assert_eq!(diagnostic.error().code(), CssErrorCode::UnexpectedToken);
    assert_eq!(diagnostic.error().position().byte_offset().value(), 6);
    let ErrorKind::UnexpectedToken(detail) = diagnostic.error().kind() else {
        panic!("lexical category")
    };
    assert_eq!(detail.encountered().kind(), CssTokenKind::BadString);
    assert_eq!(report.syntax().len(), 1);
    assert_eq!(
        report.syntax()[0].known().unwrap().property(),
        CssKnownProperty::Display
    );
}

#[test]
fn nonempty_custom_variable_mismatches_keep_their_existing_categories() {
    let name = CssCustomPropertyName::try_new("--x").unwrap();
    for (text, code, offset) in [
        ("var(x)", CssErrorCode::InvalidQualifiedRule, 4),
        ("var(1)", CssErrorCode::UnexpectedToken, 4),
        ("var(--x bad)", CssErrorCode::UnexpectedToken, 8),
        ("var(--x,;)", CssErrorCode::UnexpectedToken, 8),
    ] {
        let report = parse_property_value_text(
            text,
            CssPropertyNameRef::Custom(&name),
            CssImportance::Normal,
        );
        let diagnostic = report
            .diagnostics()
            .iter()
            .find(|diagnostic| diagnostic.action() == CssRecoveryAction::RejectInput)
            .unwrap();
        assert_eq!(diagnostic.error().code(), code, "{text}");
        assert_eq!(
            diagnostic.error().position().byte_offset().value(),
            offset,
            "{text}"
        );
    }
}
