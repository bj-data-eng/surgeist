#![forbid(unsafe_code)]
//! Syntax block-contents flushes the current declaration run when an at-rule is
//! consumed or a complete qualified-rule boundary is reached, even if that
//! candidate is later rejected. Failed declaration text alone has no such rule
//! boundary. These generic tests do not become ordinary style declarations.

use surgeist_css::{CssRecoveryAction, CssRule, CssSupportsTestItem, parse_sheet};

fn report_with_definition(source: &str) -> surgeist_css::CssParseReport<surgeist_css::CssSheet> {
    let report = parse_sheet(source);
    assert_eq!(report.syntax().rules().len(), 1, "{source}: {report:?}");
    assert!(matches!(
        report.syntax().rules()[0],
        CssRule::SupportsCondition(_)
    ));
    report
}

fn assert_split_runs(source: &str, action: CssRecoveryAction) {
    let report = report_with_definition(source);
    assert!(!report.is_clean(), "{source}");
    assert!(
        report
            .diagnostics()
            .iter()
            .any(|diagnostic| diagnostic.action() == action),
        "{source}: {:?}",
        report.diagnostics()
    );
    let [CssRule::SupportsCondition(rule)] = report.syntax().rules() else {
        unreachable!()
    };
    let [
        CssSupportsTestItem::Declarations(before),
        CssSupportsTestItem::Declarations(after),
    ] = rule.body().items()
    else {
        panic!(
            "rejected rule must separate declaration runs; got {} items",
            rule.body().items().len()
        );
    };
    let [before] = before.declarations() else {
        panic!("one before declaration");
    };
    let [after] = after.declarations() else {
        panic!("one after declaration");
    };
    assert_eq!(before.property(), "before");
    assert_eq!(after.property(), "after");
}

#[test]
fn rejected_statement_at_rule_separates_declaration_runs() {
    assert_split_runs(
        "@supports-condition --x{before:yes; @future url(a b); after:yes;}",
        CssRecoveryAction::DropAtRule,
    );
}

#[test]
fn rejected_block_at_rule_separates_runs_without_promoting_its_child() {
    assert_split_runs(
        "@supports-condition --x{before:yes; @future url(a b){child:yes} after:yes;}",
        CssRecoveryAction::DropAtRule,
    );
}

#[test]
fn rejected_complete_qualified_rule_separates_runs_without_promoting_its_child() {
    assert_split_runs(
        "@supports-condition --x{before:yes; a url(a b){child:yes} after:yes;}",
        CssRecoveryAction::DropQualifiedRule,
    );
}

#[test]
fn failed_declaration_without_rule_boundary_keeps_one_run() {
    for failed in ["broken;", "--:a{b:c}d;"] {
        let source = format!("@supports-condition --x{{before:yes; {failed} after:yes;}}");
        let report = report_with_definition(&source);
        assert!(!report.is_clean());
        assert!(
            report
                .diagnostics()
                .iter()
                .any(|diagnostic| diagnostic.action() == CssRecoveryAction::DropDeclaration)
        );
        let [CssRule::SupportsCondition(rule)] = report.syntax().rules() else {
            unreachable!()
        };
        let [CssSupportsTestItem::Declarations(run)] = rule.body().items() else {
            panic!("failed declaration does not split run: {:?}", rule.body());
        };
        let names: Vec<_> = run
            .declarations()
            .iter()
            .map(|declaration| declaration.property())
            .collect();
        assert_eq!(names, ["before", "after"]);
    }
}
