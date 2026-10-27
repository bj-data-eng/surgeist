//! Conditional Rules 3 method overload grammar, distinct from rule grammar.
use crate::css::support::authored_syntax::supports_declaration as declaration;
use surgeist_css::component::{CssComponentValues, CssValueOrigin};
use surgeist_css::declaration::CssImportance;
use surgeist_css::query::supports::{CssSupportsDeclaration, CssSupportsLeaf};
use surgeist_css::query::{CssCondition, CssConditionKind};
use surgeist_css::selector::CssNamespaceContext;
use surgeist_css::{CssParser, CssParserContext, CssParserMode};

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
        let error =
            CssSupportsDeclaration::parse(&CssParser::default(), property, "red").unwrap_err();
        assert!(
            error.is_invalid_declaration_grammar(),
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
        let error =
            CssSupportsDeclaration::parse(&CssParser::default(), "color", value).unwrap_err();
        assert!(error.is_invalid_declaration_grammar());
        original(error.origin(), value);
        let CssValueOrigin::Parsed(origin) = error.origin() else {
            unreachable!()
        };
        assert_eq!(
            origin.span().start().byte_offset().value(),
            value.find('!').unwrap()
        );
        let condition = format!("(color:{value})");
        let query = CssCondition::<CssSupportsLeaf>::parse_condition_text(
            &CssParser::default(),
            &condition,
        )
        .unwrap();
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
        let condition =
            CssCondition::<CssSupportsLeaf>::parse_condition_text(&CssParser::default(), text)
                .unwrap();
        let query = declaration(&condition);
        assert_eq!(query.known().is_some(), known);
        original(query.property_component().origin(), text);
        for value in query.value_components() {
            original(value.origin(), text);
        }
        assert_eq!(
            condition
                .serialize_tokens(Default::default())
                .unwrap()
                .as_css(),
            format!("({text})")
        );
        let values = CssComponentValues::parse(&CssParser::default(), text).unwrap();
        assert_eq!(query.components(), values.items());
        assert_eq!(text, before);
    }
}
#[test]
fn complete_conditions_remain_first_and_both_method_overloads_force_standards() {
    let text = "(color:red) and (width:1px)";
    let condition =
        CssCondition::<CssSupportsLeaf>::parse_condition_text(&CssParser::default(), text).unwrap();
    assert!(matches!(condition.kind(), CssConditionKind::And(_)));
    assert_eq!(
        condition
            .serialize_tokens(Default::default())
            .unwrap()
            .as_css(),
        text
    );
    for property in ["color", "COLOR"] {
        let value =
            CssSupportsDeclaration::parse(&CssParser::default(), property, "/*😀*/red").unwrap();
        assert!(value.known().is_some());
        assert_eq!(value.importance(), CssImportance::Normal);
        original(value.origin(), property);
        for component in value.value_components() {
            original(component.origin(), "/*😀*/red");
        }
    }
    for (name, value) in [("color", "123"), ("future-property", "future(1)")] {
        assert!(
            CssSupportsDeclaration::parse(&CssParser::default(), name, value)
                .unwrap()
                .known()
                .is_none()
        );
    }
    let quirk = CssParserContext::new(CssParserMode::Quirks);
    let rule = CssCondition::<CssSupportsLeaf>::parse(
        &CssParser::new(quirk, Default::default()),
        "(color:123)",
        &CssNamespaceContext::default(),
    )
    .unwrap();
    assert!(declaration(&rule).known().is_some());
    assert!(
        declaration(
            &CssCondition::<CssSupportsLeaf>::parse_condition_text(
                &CssParser::default(),
                "color:123"
            )
            .unwrap()
        )
        .known()
        .is_none()
    );
    assert!(
        CssSupportsDeclaration::parse(&CssParser::default(), "color", "123")
            .unwrap()
            .known()
            .is_none()
    );
    assert!(
        CssSupportsDeclaration::parse(&CssParser::default(), "--Case", r#""!important""#)
            .unwrap()
            .known()
            .is_none()
    );
}
#[test]
fn method_retry_preserves_component_and_recovered_errors_and_valid_retry() {
    for text in ["(color:rgb(1 2 3)", "color:red/*"] {
        let error =
            CssCondition::<CssSupportsLeaf>::parse_condition_text(&CssParser::default(), text)
                .unwrap_err();
        assert!(error.is_recovered_input());
        assert!(matches!(
            error.origin(),
            CssValueOrigin::ImplicitClosure { .. }
        ));
    }
    let error =
        CssCondition::<CssSupportsLeaf>::parse_condition_text(&CssParser::default(), "color:)")
            .unwrap_err();
    assert!(error.component_error().is_some());
    let text = "(color:".to_owned() + &"f(".repeat(257) + "red" + &")".repeat(258);
    let error = CssCondition::<CssSupportsLeaf>::parse_condition_text(&CssParser::default(), &text)
        .unwrap_err();
    let Some(error) = error.component_error() else {
        panic!("original component resource failure")
    };
    assert!(error.is_nesting_limit(), "{error:?}");
    assert!(
        declaration(
            &CssCondition::<CssSupportsLeaf>::parse_condition_text(
                &CssParser::default(),
                "color:red"
            )
            .unwrap()
        )
        .known()
        .is_some()
    );
}
