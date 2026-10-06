#![forbid(unsafe_code)]
//! Conditional Rules 3 method overload grammar, distinct from rule grammar.
use surgeist_css::*;

fn declaration(condition: &CssSupportsCondition) -> &CssSupportsDeclaration {
    let CssSupportsConditionKind::Declaration(value) = condition.kind() else {
        panic!("declaration condition")
    };
    value
}
fn original(origin: &CssValueOrigin, text: &str) {
    let CssValueOrigin::Parsed(origin) = origin else {
        panic!("original argument origin: {origin:?}")
    };
    assert_eq!(origin.source().as_str(), text);
}

#[test]
fn literal_property_arguments_reject_css_trivia_and_escape_processing() {
    for property in [
        " color",
        "color ",
        "/*x*/color",
        "color/**/",
        r"\63 olor",
        "co\nlor",
    ] {
        let error = parse_css_supports_declaration(property, "red").unwrap_err();
        assert!(
            matches!(
                error,
                CssSupportsConstructionError::InvalidDeclarationGrammar { .. }
            ),
            "{property:?}: {error:?}"
        );
        original(error.origin(), property);
    }
}
#[test]
fn two_argument_values_reject_root_importance_without_changing_rule_grammar() {
    for value in [
        "red!important",
        "red ! IMPORTANT",
        "red!/**/important",
        "var(--x)!important",
    ] {
        let error = parse_css_supports_declaration("color", value).unwrap_err();
        assert!(matches!(
            error,
            CssSupportsConstructionError::InvalidDeclarationGrammar { .. }
        ));
        original(error.origin(), value);
        let CssValueOrigin::Parsed(origin) = error.origin() else {
            unreachable!()
        };
        assert_eq!(
            origin.span().start().byte_offset().value(),
            value.find('!').unwrap()
        );
        let condition = format!("(color:{value})");
        let query = parse_css_supports_condition(&condition).unwrap();
        assert_eq!(declaration(&query).importance(), CssImportance::Important);
    }
}
#[test]
fn bare_one_argument_declarations_gain_parentheses_with_original_child_sources() {
    for (text, known) in [
        ("color:red", true),
        ("/*😀*/ COLOR: red", true),
        ("color:123", false),
        ("color:red!important", true),
    ] {
        let before = text.to_owned();
        let condition = parse_css_supports_condition(text).unwrap();
        let query = declaration(&condition);
        assert_eq!(query.known().is_some(), known);
        original(query.property_component().origin(), text);
        for value in query.value_components() {
            original(value.origin(), text);
        }
        assert_eq!(condition.serialize().unwrap().as_css(), format!("({text})"));
        let values = parse_component_values(text).unwrap();
        assert_eq!(query.components(), values.items());
        assert_eq!(text, before);
    }
}
#[test]
fn complete_conditions_remain_first_and_both_method_overloads_force_standards() {
    let text = "(color:red) and (width:1px)";
    let condition = parse_css_supports_condition(text).unwrap();
    assert!(matches!(condition.kind(), CssSupportsConditionKind::And(_)));
    assert_eq!(condition.serialize().unwrap().as_css(), text);
    for property in ["color", "COLOR"] {
        let value = parse_css_supports_declaration(property, "/*😀*/red").unwrap();
        assert!(value.known().is_some());
        assert_eq!(value.importance(), CssImportance::Normal);
        original(value.origin(), property);
        for component in value.value_components() {
            original(component.origin(), "/*😀*/red");
        }
    }
    for (name, value) in [("color", "123"), ("future-property", "future(1)")] {
        assert!(
            parse_css_supports_declaration(name, value)
                .unwrap()
                .known()
                .is_none()
        );
    }
    let quirk = CssParserContext::new(CssParserMode::Quirks);
    let rule = quirk
        .parse_supports_condition("(color:123)", &CssNamespaceContext::default())
        .unwrap();
    assert!(declaration(&rule).known().is_some());
    assert!(
        declaration(&parse_css_supports_condition("color:123").unwrap())
            .known()
            .is_none()
    );
    assert!(
        parse_css_supports_declaration("color", "123")
            .unwrap()
            .known()
            .is_none()
    );
    assert!(
        parse_css_supports_declaration("--Case", r#""!important""#)
            .unwrap()
            .known()
            .is_none()
    );
}
#[test]
fn method_retry_preserves_component_and_recovered_errors_and_valid_retry() {
    for text in ["(color:rgb(1 2 3)", "color:red/*"] {
        let error = parse_css_supports_condition(text).unwrap_err();
        assert!(matches!(
            error,
            CssSupportsConstructionError::RecoveredInput { .. }
        ));
        assert!(matches!(
            error.origin(),
            CssValueOrigin::ImplicitClosure { .. }
        ));
    }
    let error = parse_css_supports_condition("color:)").unwrap_err();
    assert!(matches!(error, CssSupportsConstructionError::Component(_)));
    let text = "(color:".to_owned() + &"f(".repeat(257) + "red" + &")".repeat(258);
    let error = parse_css_supports_condition(&text).unwrap_err();
    let CssSupportsConstructionError::Component(error) = error else {
        panic!("original component resource failure")
    };
    assert_eq!(error.kind(), CssComponentValueErrorKind::NestingLimit);
    assert!(parse_css_supports_condition("color:red").is_ok());
}
