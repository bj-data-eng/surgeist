#![forbid(unsafe_code)]
//! Conditional 5 §8 selects modern block contents, including nested at-rule
//! termination at a real parent close. Checked output may supply a semicolon.

use surgeist_css::{
    CssNamedSupportsConstructionError, CssRecoveryAction, CssRule, CssSerializedOrigin,
    CssSupportsAtRuleTest, CssSupportsConditionRule, CssSupportsTestBody, CssSupportsTestItem,
    CssValueOrigin, parse_component_values, parse_sheet, validate_sheet,
};

fn statement(body: &CssSupportsTestBody) -> &CssSupportsAtRuleTest {
    let [CssSupportsTestItem::AtRule(statement)] = body.items() else {
        panic!("one retained generic at-rule test")
    };
    assert_eq!(statement.name(), "future");
    assert!(statement.prelude().items().is_empty());
    assert!(statement.body().is_none());
    statement
}

fn generated_semicolon(body: &CssSupportsTestBody) {
    let output = body.serialize().unwrap();
    assert_eq!(output.as_css(), "@future;");
    assert!(matches!(
        output.origin_at(7),
        Some(CssSerializedOrigin::Token(CssValueOrigin::Programmatic))
    ));
}

#[test]
fn explicit_named_body_close_and_checked_components_allow_an_unterminated_symbolic_statement() {
    // Pinned Syntax block supplement §§5.5.2/5.5.5 uses nested=true.
    let source = "/*🦊*/\r\n@supports-condition --x{@future}";
    let report = parse_sheet(source);
    assert!(report.is_clean(), "{:?}", report.diagnostics());
    let [CssRule::SupportsCondition(definition)] = report.syntax().rules() else {
        panic!("one named definition")
    };
    let keyword = statement(definition.body());
    let CssValueOrigin::Parsed(origin) = keyword.origin() else {
        panic!("original parsed at-keyword")
    };
    let start = source.find("@future").unwrap();
    assert_eq!(origin.source().as_str(), source);
    assert_eq!(origin.span().start().byte_offset().value(), start);
    assert_eq!(origin.span().end().byte_offset().value(), start + 7);
    assert!(definition.body().recovery_origin().is_none());
    generated_semicolon(definition.body());
    assert_eq!(validate_sheet(source).unwrap(), *report.syntax());

    let input = parse_component_values("@future").unwrap();
    let input_origin = input.items()[0].origin().clone();
    let body = CssSupportsTestBody::try_from_components(input).unwrap();
    let keyword = statement(&body);
    assert_eq!(keyword.origin(), &input_origin);
    let (CssValueOrigin::Parsed(before), CssValueOrigin::Parsed(after)) =
        (&input_origin, keyword.origin())
    else {
        panic!("checked promotion preserves the supplied parsed keyword")
    };
    assert!(before.source().same_snapshot(after.source()));
    assert!(body.recovery_origin().is_none());
    generated_semicolon(&body);
}

#[test]
fn actual_source_eof_retains_the_statement_and_only_the_real_named_body_closure() {
    let source = "/*🦊*/\r\n@supports-condition --x{@future";
    let report = parse_sheet(source);
    let [CssRule::SupportsCondition(definition)] = report.syntax().rules() else {
        panic!("EOF-recovered named definition")
    };
    statement(definition.body());
    generated_semicolon(definition.body());
    let [diagnostic] = report.diagnostics() else {
        panic!(
            "only the missing outer brace reports recovery: {:?}",
            report.diagnostics()
        )
    };
    assert_eq!(
        diagnostic.action(),
        CssRecoveryAction::RetainWithImplicitClosure
    );
    let recovery = definition.body().recovery_origin().unwrap();
    let CssValueOrigin::ImplicitClosure { opening, at } = recovery else {
        panic!("actual missing brace remains the recovery origin")
    };
    assert_eq!(
        opening.span().start().byte_offset().value(),
        source.find('{').unwrap()
    );
    assert_eq!(at.span().start().byte_offset().value(), source.len());
    assert!(opening.source().same_snapshot(at.source()));
    assert_eq!(at.source().as_str(), source);
    assert_eq!(
        validate_sheet(source).unwrap_err().diagnostics(),
        report.diagnostics()
    );
    assert!(matches!(
        CssSupportsConditionRule::try_new(definition.name().clone(), definition.body().clone()),
        Err(CssNamedSupportsConstructionError::RecoveredInput { origin }) if &origin == recovery
    ));
}
