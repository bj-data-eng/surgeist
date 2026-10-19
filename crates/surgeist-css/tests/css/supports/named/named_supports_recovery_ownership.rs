//! Surgeist #644 selects lowest retained recovery ownership and a body-local
//! first summary. Historical parser diagnostics keep their original order.

use surgeist_css::{
    CssRecoveryAction, CssRule, CssSupportsConditionRule, CssSupportsTestBody, CssSupportsTestItem,
    CssValueOrigin, parse_component_values, parse_sheet,
};

fn definition(rules: &[CssRule]) -> &CssSupportsConditionRule {
    let [CssRule::SupportsCondition(rule)] = rules else {
        panic!("one retained named definition: {rules:?}")
    };
    rule
}

fn parsed_offset(origin: &CssValueOrigin, source: &str) -> usize {
    let CssValueOrigin::Parsed(parsed) = origin else {
        panic!("original parsed fault: {origin:?}")
    };
    assert_eq!(parsed.source().as_str(), source);
    parsed.span().start().byte_offset().value()
}

fn assert_own_missing_close(origin: &CssValueOrigin, source: &str) {
    let CssValueOrigin::ImplicitClosure { opening, at } = origin else {
        panic!("body-local missing close must precede child recovery: {origin:?}")
    };
    assert_eq!(opening.source().as_str(), source);
    assert!(opening.source().same_snapshot(at.source()));
    assert_eq!(
        opening.span().start().byte_offset().value(),
        source.find('{').unwrap()
    );
    assert_eq!(at.span().start().byte_offset().value(), source.len());
}

#[test]
fn later_body_local_fault_precedes_earlier_retained_child_fault_in_summary() {
    let source = "@supports-condition --x { & { bad:url(a b); } bad:url(a b); }";
    let report = parse_sheet(source);
    let [first, second] = report.diagnostics() else {
        panic!(
            "two independently owned discarded declarations: {:?}",
            report.diagnostics()
        )
    };
    assert_eq!(first.action(), CssRecoveryAction::DropDeclaration);
    assert_eq!(second.action(), CssRecoveryAction::DropDeclaration);
    assert_eq!(
        first.error().position().byte_offset().value(),
        source.find("url(a b)").unwrap()
    );
    assert_eq!(
        second.error().position().byte_offset().value(),
        source.rfind("url(a b)").unwrap()
    );
    let body = definition(report.syntax().rules()).body();
    let [CssSupportsTestItem::QualifiedRule(child)] = body.items() else {
        panic!("retained qualified child with both invalid declarations dropped: {body:?}")
    };
    assert!(child.body().items().is_empty());
    assert_eq!(
        parsed_offset(child.body().recovery_origin().unwrap(), source),
        source.find("url(a b)").unwrap()
    );
    assert_eq!(
        parsed_offset(body.recovery_origin().unwrap(), source),
        source.rfind("url(a b)").unwrap()
    );
}

#[test]
fn own_missing_body_close_precedes_retained_nested_body_fault_in_summary() {
    let source = "@supports-condition --x { & { bad:url(a b); }";
    let report = parse_sheet(source);
    assert!(
        report
            .diagnostics()
            .iter()
            .any(|diagnostic| diagnostic.action() == CssRecoveryAction::DropDeclaration)
    );
    assert!(
        report
            .diagnostics()
            .iter()
            .any(|diagnostic| diagnostic.action() == CssRecoveryAction::RetainWithImplicitClosure)
    );
    let body = definition(report.syntax().rules()).body();
    let [CssSupportsTestItem::QualifiedRule(child)] = body.items() else {
        panic!("retained closed nested candidate: {body:?}")
    };
    assert_eq!(
        parsed_offset(child.body().recovery_origin().unwrap(), source),
        source.find("url(a b)").unwrap()
    );
    assert_own_missing_close(body.recovery_origin().unwrap(), source);
}

#[test]
fn own_missing_body_close_precedes_retained_declaration_value_closure_in_summary() {
    let source = "@supports-condition --x { color:fn(a";
    let report = parse_sheet(source);
    assert!(!report.is_clean());
    let body = definition(report.syntax().rules()).body();
    let [CssSupportsTestItem::Declarations(run)] = body.items() else {
        panic!("retained declaration with implicit value closure: {body:?}")
    };
    assert_eq!(run.declarations()[0].property(), "color");
    assert_own_missing_close(body.recovery_origin().unwrap(), source);
}

#[test]
fn nested_only_recovery_summary_uses_first_child_without_reordering_history() {
    let source = "@supports-condition --x { & { bad:url(a b); } @future { bad:url(c d); } }";
    let report = parse_sheet(source);
    assert_eq!(report.diagnostics().len(), 2);
    let body = definition(report.syntax().rules()).body();
    let [
        CssSupportsTestItem::QualifiedRule(first),
        CssSupportsTestItem::AtRule(second),
    ] = body.items()
    else {
        panic!("two retained children: {body:?}")
    };
    assert_eq!(body.recovery_origin(), first.body().recovery_origin());
    assert_eq!(
        parsed_offset(body.recovery_origin().unwrap(), source),
        source.find("url(a b)").unwrap()
    );
    assert_eq!(
        parsed_offset(second.body().unwrap().recovery_origin().unwrap(), source),
        source.find("url(c d)").unwrap()
    );
    assert_eq!(
        report.diagnostics()[1]
            .error()
            .position()
            .byte_offset()
            .value(),
        source.find("url(c d)").unwrap()
    );
}

#[test]
fn empty_semicolons_are_ignored_recursively_by_existing_partition() {
    let contents = ";color:red;; & { ; } @future { ;; inner:yes;; } ;";
    let body = CssSupportsTestBody::try_from_components(parse_component_values(contents).unwrap())
        .unwrap();
    assert!(body.recovery_origin().is_none());
    let [
        CssSupportsTestItem::Declarations(run),
        CssSupportsTestItem::QualifiedRule(qualified),
        CssSupportsTestItem::AtRule(at),
    ] = body.items()
    else {
        panic!("three ordered candidates, no semicolon children: {body:?}")
    };
    assert_eq!(run.declarations().len(), 1);
    assert_eq!(run.declarations()[0].property(), "color");
    assert!(qualified.body().items().is_empty());
    let [CssSupportsTestItem::Declarations(inner)] = at.body().unwrap().items() else {
        panic!("nested declaration with surrounding empty semicolons")
    };
    assert_eq!(inner.declarations().len(), 1);
    assert_eq!(inner.declarations()[0].property(), "inner");
    assert_eq!(
        body.to_specified_css().unwrap(),
        "color: red; & { } @future  { inner: yes; }"
    );
    let report = parse_sheet(
        "@supports-condition --x { ;color:red;; & { ; } @future { ;; inner:yes;; } ; }",
    );
    assert!(report.is_clean(), "{:?}", report.diagnostics());
    assert_eq!(
        definition(report.syntax().rules())
            .to_specified_css()
            .unwrap(),
        "@supports-condition --x { color: red; & { } @future  { inner: yes; } }"
    );
}
