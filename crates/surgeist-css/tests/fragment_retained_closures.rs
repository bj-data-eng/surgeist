#![forbid(unsafe_code)]

//! Syntax 3 closes open functions at actual EOF, but Selectors 4 forgiving
//! recovery discards malformed members. Retention diagnostics describe only
//! functions surviving that recovery, not all lexically open functions.

use surgeist_css::{
    CssNamespaceContext, CssPseudoClass, CssRecoveryAction, CssRecoveryDiagnostic, CssSelector,
    parse_selector, parse_selector_list,
};

fn assert_retained_closures(
    diagnostics: &[CssRecoveryDiagnostic],
    dropped_start: usize,
    eof: usize,
    count: usize,
) {
    let dropped: Vec<_> = diagnostics
        .iter()
        .filter(|d| d.action() == CssRecoveryAction::DropSelectorListItem)
        .collect();
    assert_eq!(dropped.len(), 1);
    assert_eq!(
        dropped[0].span().start().byte_offset().value(),
        dropped_start
    );
    assert_eq!(dropped[0].span().end().byte_offset().value(), eof);
    assert_eq!(
        dropped[0].error().position().byte_offset().value(),
        dropped_start
    );
    let closures: Vec<_> = diagnostics
        .iter()
        .filter(|d| d.action() == CssRecoveryAction::RetainWithImplicitClosure)
        .collect();
    assert_eq!(
        closures.len(),
        count,
        "discarded functions cannot claim retention"
    );
    assert_eq!(diagnostics.len(), 1 + count);
    for closure in closures {
        assert_eq!(closure.error().position().byte_offset().value(), eof);
        assert_eq!(closure.span().start().byte_offset().value(), eof);
        assert_eq!(closure.span().end().byte_offset().value(), eof);
    }
}

#[test]
fn discarded_unclosed_function_does_not_claim_a_retained_closure() {
    let report = parse_selector(":is(???f(", &CssNamespaceContext::default());
    let Some(CssSelector::PseudoClass(CssPseudoClass::Is(members))) = report.syntax() else {
        panic!("retained is selector");
    };
    assert!(members.selectors().is_empty());
    assert_retained_closures(report.diagnostics(), 4, 9, 1);
}

#[test]
fn nested_retained_functions_keep_their_own_closures_after_member_recovery() {
    let report = parse_selector(":is(:where(.kept,???f(", &CssNamespaceContext::default());
    let Some(CssSelector::PseudoClass(CssPseudoClass::Is(outer))) = report.syntax() else {
        panic!("retained is selector");
    };
    let [CssSelector::PseudoClass(CssPseudoClass::Where(inner))] = outer.selectors() else {
        panic!("retained where selector");
    };
    assert_eq!(inner.selectors(), [CssSelector::Class("kept".into())]);
    assert_retained_closures(report.diagnostics(), 17, 22, 2);
}

#[test]
fn selector_list_preserves_siblings_and_only_surviving_function_closures() {
    let report = parse_selector_list(".first,:where(.kept,???f(", &CssNamespaceContext::default());
    let members = report.syntax().as_ref().unwrap().selectors();
    assert_eq!(members.len(), 2);
    assert_eq!(members[0].selector(), &CssSelector::Class("first".into()));
    let CssSelector::PseudoClass(CssPseudoClass::Where(inner)) = members[1].selector() else {
        panic!("retained where selector");
    };
    assert_eq!(inner.selectors(), [CssSelector::Class("kept".into())]);
    assert_retained_closures(report.diagnostics(), 20, 25, 1);
}

#[test]
fn accepted_unclosed_function_without_discarded_members_keeps_its_closure() {
    let report = parse_selector(":is(.kept", &CssNamespaceContext::default());
    assert!(report.syntax().is_some());
    let [closure] = report.diagnostics() else {
        panic!("one retained closure");
    };
    assert_eq!(
        closure.action(),
        CssRecoveryAction::RetainWithImplicitClosure
    );
    assert_eq!(closure.error().position().byte_offset().value(), 9);
}
