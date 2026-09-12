#![forbid(unsafe_code)]
//! Responsible-token positions follow Error::position and CssRecoveryDiagnostic::error.
//! Unknown function names are invalid under pinned Selectors 4 section 3.9:
//! https://www.w3.org/TR/2026/WD-selectors-4-20260122/#invalid
use surgeist_css::{
    CssNamespaceContext, CssPseudoClass, CssRecoveryAction, CssRecoveryDiagnostic, CssRule,
    CssSelector, CssTokenKind, ErrorKind, parse_selector, parse_sheet,
};

fn assert_origin(
    diagnostic: &CssRecoveryDiagnostic,
    position: (usize, u32, u32),
    span: (usize, usize),
    kind: CssTokenKind,
    spelling: &str,
) {
    let actual = diagnostic.error().position();
    assert_eq!(actual.byte_offset().value(), position.0);
    assert_eq!(actual.line().value(), position.1);
    assert_eq!(actual.column().value(), position.2);
    assert_eq!(diagnostic.span().start().byte_offset().value(), span.0);
    assert_eq!(diagnostic.span().end().byte_offset().value(), span.1);
    let ErrorKind::InvalidSelector(detail) = diagnostic.error().kind() else {
        panic!("selector error expected: {diagnostic:?}");
    };
    let token = detail.encountered().expect("responsible authored token");
    assert_eq!(token.kind(), kind);
    assert_eq!(token.authored(), spelling);
}

#[test]
fn unknown_functions_blame_function_token_in_raw_and_sheet_contexts() {
    for (raw, position, spelling) in [
        (":unknown()", (1, 0, 1), "unknown("),
        (":unknown(x)", (1, 0, 1), "unknown("),
        (":unknown( x)", (1, 0, 1), "unknown("),
        (r":un\6bnown(x)", (1, 0, 1), r"un\6bnown("),
        ("/*😀*/\r\n/*é*/:unknown(x)", (17, 1, 6), "unknown("),
        (":未知(x)", (1, 0, 1), "未知("),
    ] {
        let report = parse_selector(raw, &CssNamespaceContext::default());
        assert!(report.syntax().is_none(), "{raw}: {report:?}");
        let diagnostic = report
            .diagnostics()
            .iter()
            .find(|diagnostic| diagnostic.action() == CssRecoveryAction::RejectInput)
            .expect("public fragment rejection");
        assert_origin(
            diagnostic,
            position,
            (0, raw.len()),
            CssTokenKind::Function,
            spelling,
        );

        let rejected_rule = format!("{raw}{{}}");
        let sheet = format!("{rejected_rule}.after{{color:red}}");
        let report = parse_sheet(&sheet);
        let [CssRule::Style(after)] = report.syntax().rules() else {
            panic!("retain only following valid rule: {report:?}");
        };
        assert_eq!(
            after.selectors().selectors()[0].selector(),
            &CssSelector::Class("after".into())
        );
        assert_eq!(after.declarations().len(), 1);
        let diagnostic = report
            .diagnostics()
            .iter()
            .find(|diagnostic| diagnostic.action() == CssRecoveryAction::DropQualifiedRule)
            .expect("qualified-rule rejection");
        assert_origin(
            diagnostic,
            position,
            (raw.find(':').unwrap(), rejected_rule.len()),
            CssTokenKind::Function,
            spelling,
        );
    }
}

#[test]
fn forgiving_unknown_function_preserves_whole_member_span_and_valid_member() {
    let source = ":is(:unknown(x),.ok)";
    let report = parse_selector(source, &CssNamespaceContext::default());
    let Some(CssSelector::PseudoClass(CssPseudoClass::Is(members))) = report.syntax() else {
        panic!("retained forgiving selector: {report:?}");
    };
    assert_eq!(members.selectors(), [CssSelector::Class("ok".into())]);
    let diagnostic = report
        .diagnostics()
        .iter()
        .find(|diagnostic| diagnostic.action() == CssRecoveryAction::DropSelectorListItem)
        .expect("unknown member dropped");
    assert_origin(
        diagnostic,
        (5, 0, 5),
        (4, 15),
        CssTokenKind::Function,
        "unknown(",
    );
}

#[test]
fn known_function_argument_error_keeps_argument_origin() {
    let source = ":lang(123)";
    let report = parse_selector(source, &CssNamespaceContext::default());
    assert!(report.syntax().is_none());
    let diagnostic = report
        .diagnostics()
        .iter()
        .find(|diagnostic| diagnostic.action() == CssRecoveryAction::RejectInput)
        .expect("invalid language argument rejected");
    assert_origin(
        diagnostic,
        (6, 0, 6),
        (0, source.len()),
        CssTokenKind::Number,
        "123",
    );
    for source in [":lang(en)", ":not(.x)", ":is(.ok)"] {
        let report = parse_selector(source, &CssNamespaceContext::default());
        assert!(
            report.is_clean() && report.syntax().is_some(),
            "{source}: {report:?}"
        );
    }
}
