#![forbid(unsafe_code)]

//! Existing-API runtime RED, independent of the new fragment consumer.
//!
//! Selectors 4 (2026-01-22), section 3.9, rejects an unknown pseudo name:
//! https://www.w3.org/TR/2026/WD-selectors-4-20260122/#invalid
//! `Error::position` and `CssRecoveryDiagnostic::error` identify the first
//! responsible authored token. For these identifiers that is the start of
//! the name, after the colon(s), not the cursor after consuming the name.
//! Rule and forgiving-member recovery units retain their full original spans.

use surgeist_css::{
    CssErrorCode, CssPseudoClass, CssRecoveryAction, CssRecoveryDiagnostic, CssRule, CssSelector,
    CssSourcePosition, CssTokenKind, ErrorKind, parse_sheet, validate_sheet,
};

fn assert_position(actual: CssSourcePosition, expected: (usize, u32, u32)) {
    assert_eq!(
        actual.byte_offset().value(),
        expected.0,
        "responsible UTF-8 byte"
    );
    assert_eq!(actual.line().value(), expected.1, "zero-based line");
    assert_eq!(
        actual.column().value(),
        expected.2,
        "zero-based UTF-16 column"
    );
}

fn assert_named_error(
    diagnostic: &CssRecoveryDiagnostic,
    action: CssRecoveryAction,
    position: (usize, u32, u32),
    span: (usize, usize),
    spelling: &str,
) {
    assert_eq!(diagnostic.error().code(), CssErrorCode::InvalidSelector);
    assert_eq!(diagnostic.action(), action);
    assert_eq!(diagnostic.span().start().byte_offset().value(), span.0);
    assert_eq!(diagnostic.span().end().byte_offset().value(), span.1);
    let ErrorKind::InvalidSelector(detail) = diagnostic.error().kind() else {
        panic!("typed selector details");
    };
    assert_eq!(
        detail.production().unwrap().as_str(),
        "baseline.selector.complex"
    );
    // Assert the diagnosed responsible-position defect before its resulting
    // wrong-token spelling, so RED identifies the actual ownership error.
    assert_position(diagnostic.error().position(), position);
    let token = detail.encountered().expect("responsible pseudo name");
    assert_eq!(token.kind(), CssTokenKind::Ident);
    assert_eq!(token.authored(), spelling);
}

fn assert_rejected_rule(
    source: &str,
    position: (usize, u32, u32),
    span: (usize, usize),
    spelling: &str,
) {
    let report = parse_sheet(source);
    let [CssRule::Style(after)] = report.syntax().rules() else {
        panic!("reject the invalid rule and retain the following valid rule");
    };
    assert_eq!(
        after.selectors().selectors()[0].selector(),
        &CssSelector::Class("after".into())
    );
    assert_eq!(after.declarations().len(), 1);
    let [diagnostic] = report.diagnostics() else {
        panic!("one unknown pseudo name: {:?}", report.diagnostics());
    };
    let failure = validate_sheet(source).expect_err("recovered syntax is not clean validation");
    assert_eq!(failure.diagnostics(), report.diagnostics());
    assert_named_error(
        diagnostic,
        CssRecoveryAction::DropQualifiedRule,
        position,
        span,
        spelling,
    );
}

#[test]
fn unknown_named_pseudo_class_identifies_name_start_and_preserves_following_rule() {
    assert_rejected_rule(":test{}.after{color:red}", (1, 0, 1), (0, 7), "test");
}

#[test]
fn unknown_named_pseudo_element_identifies_name_start_and_preserves_following_rule() {
    assert_rejected_rule("::test{}.after{color:red}", (2, 0, 2), (0, 8), "test");
}

#[test]
fn escaped_unknown_pseudo_name_retains_exact_authored_identifier() {
    assert_rejected_rule(r":t\65st{}.after{color:red}", (1, 0, 1), (0, 9), r"t\65st");
}

#[test]
fn unknown_pseudo_name_positions_respect_unicode_comments_and_crlf() {
    assert_rejected_rule(
        "/*😀*/\r\n/*é*/:test{}.after{color:red}",
        (17, 1, 6),
        (16, 23),
        "test",
    );
}

#[test]
fn forgiving_unknown_named_pseudo_reports_name_without_blaming_following_comma() {
    let source = ":is(:test,.ok){color:red}.after{color:blue}";
    let report = parse_sheet(source);
    let [CssRule::Style(first), CssRule::Style(after)] = report.syntax().rules() else {
        panic!("retain both the recovered selector rule and its following rule");
    };
    let CssSelector::PseudoClass(CssPseudoClass::Is(members)) =
        first.selectors().selectors()[0].selector()
    else {
        panic!("typed forgiving selector list");
    };
    assert_eq!(members.selectors(), [CssSelector::Class("ok".into())]);
    assert_eq!(first.declarations().len(), 1);
    assert_eq!(
        after.selectors().selectors()[0].selector(),
        &CssSelector::Class("after".into())
    );
    let [diagnostic] = report.diagnostics() else {
        panic!("only the unknown member is discarded")
    };
    assert_eq!(
        validate_sheet(source).unwrap_err().diagnostics(),
        report.diagnostics()
    );
    assert_named_error(
        diagnostic,
        CssRecoveryAction::DropSelectorListItem,
        (5, 0, 5),
        (4, 9),
        "test",
    );
}

#[test]
fn known_named_and_escaped_pseudo_controls_stay_clean() {
    for source in [":hover{}", "::before{}", r":h\6fver{}", ":is(:hover,.ok){}"] {
        let report = parse_sheet(source);
        assert!(report.is_clean(), "{source}: {:?}", report.diagnostics());
        assert!(matches!(report.syntax().rules(), [CssRule::Style(_)]));
        assert_eq!(validate_sheet(source).unwrap(), *report.syntax());
    }
}
