#![forbid(unsafe_code)]
//! Selectors 4 (2026-01-22) section 16.1 checks <any-value>? before forgiving
//! selector-member filtering. Syntax 3 (2021-12-24) sections 5.3.1, 5.4.8 and
//! 8.2 retain an unmatched ')' inside an unclosed '{'; nested bad tokens are
//! excluded from any-value. A missing EOF closure alone is recoverable.
//! https://www.w3.org/TR/2026/WD-selectors-4-20260122/#parse-as-a-forgiving-selector-list
//! https://www.w3.org/TR/2021/CRD-css-syntax-3-20211224/#any-value
use surgeist_css::{
    CssErrorCode, CssNamespaceContext, CssPseudoClass, CssRecoveryAction, CssSelector,
    CssTokenKind, ErrorKind, parse_selector,
};

fn assert_rejected(source: &str) {
    let report = parse_selector(source, &CssNamespaceContext::default());
    assert!(
        report.syntax().is_none(),
        "invalid whole selector retained: {source}: {report:?}"
    );
    let rejection = report
        .diagnostics()
        .iter()
        .find(|diagnostic| diagnostic.action() == CssRecoveryAction::RejectInput)
        .expect("complete selector rejection");
    assert_eq!(rejection.error().code(), CssErrorCode::InvalidSelector);
    assert_eq!(rejection.span().start().byte_offset().value(), 0);
    assert_eq!(rejection.span().end().byte_offset().value(), source.len());
    assert!(
        !report
            .diagnostics()
            .iter()
            .any(|diagnostic| diagnostic.action() == CssRecoveryAction::RetainWithImplicitClosure),
        "rejected syntax cannot claim retained EOF closures"
    );
    assert!(report.clone().into_validation_result().is_err());
}

#[test]
fn is_rejects_unmatched_parenthesis_inside_unclosed_curly_block() {
    assert_rejected(":is(.a{)");
}

#[test]
fn is_rejects_invalid_envelope_even_with_a_valid_first_member() {
    assert_rejected(":is(.a,.b{)");
}

#[test]
fn where_rejects_unmatched_parenthesis_inside_unclosed_curly_block() {
    assert_rejected(":where(.a{)");
}

#[test]
fn where_rejects_invalid_envelope_even_with_a_valid_first_member() {
    assert_rejected(":where(.a,.b{)");
}

fn retained_members(selector: &Option<CssSelector>) -> &[CssSelector] {
    match selector {
        Some(CssSelector::PseudoClass(CssPseudoClass::Is(list) | CssPseudoClass::Where(list))) => {
            list.selectors()
        }
        other => panic!("retained forgiving pseudo: {other:?}"),
    }
}

#[test]
fn lexical_validity_does_not_prevent_forgiving_removal_of_invalid_selector_members() {
    for pseudo in ["is", "where"] {
        let source = format!(":{pseudo}(:unknown(x),.ok)");
        let report = parse_selector(&source, &CssNamespaceContext::default());
        assert_eq!(
            retained_members(report.syntax()),
            [CssSelector::Class("ok".into())]
        );
        assert!(
            report
                .diagnostics()
                .iter()
                .any(|diagnostic| diagnostic.action() == CssRecoveryAction::DropSelectorListItem)
        );
        assert!(
            !report
                .diagnostics()
                .iter()
                .any(|diagnostic| diagnostic.action() == CssRecoveryAction::RejectInput)
        );
        assert!(
            report.into_validation_result().is_err(),
            "strict validation rejects recovered reports"
        );
    }
}

#[test]
fn missing_eof_closure_retains_a_lexically_valid_forgiving_selector() {
    for source in [":is(.a", ":where(.a", ":is(.a,.b", ":where(.a,.b"] {
        let report = parse_selector(source, &CssNamespaceContext::default());
        let expected = if source.contains(',') {
            vec![
                CssSelector::Class("a".into()),
                CssSelector::Class("b".into()),
            ]
        } else {
            vec![CssSelector::Class("a".into())]
        };
        assert_eq!(retained_members(report.syntax()), expected);
        let [diagnostic] = report.diagnostics() else {
            panic!("one retained EOF closure: {report:?}")
        };
        assert_eq!(
            diagnostic.action(),
            CssRecoveryAction::RetainWithImplicitClosure
        );
        assert_eq!(
            diagnostic.error().position().byte_offset().value(),
            source.len()
        );
        assert!(report.into_validation_result().is_err());
    }
}

#[test]
fn not_and_has_do_not_forgive_invalid_selector_members() {
    for source in [
        ":not(.ok,:unknown(x))",
        ":has(.ok,:unknown(x))",
        ":not(.a,.b{)",
        ":has(.a,.b{)",
    ] {
        assert_rejected(source);
    }
    for source in [":not(.ok)", ":has(>.ok)", ":is(.ok)", ":where(.ok)"] {
        let report = parse_selector(source, &CssNamespaceContext::default());
        assert!(report.is_clean(), "{source}: {report:?}");
        assert!(report.into_validation_result().unwrap().is_some());
    }
}

#[test]
fn bad_tokens_are_not_forgiven_and_the_responsible_unicode_source_position_is_retained() {
    for (body, offending, kind) in [
        (".ok,url(a b)", "url(", CssTokenKind::BadUrl),
        (".ok,\"bad\n", "\"bad", CssTokenKind::BadString),
        (".ok,]", "]", CssTokenKind::CloseSquareBracket),
    ] {
        let source = format!("/*😀*/\r\n/*é*/:is({body})");
        let report = parse_selector(&source, &CssNamespaceContext::default());
        assert!(report.syntax().is_none(), "{source}: {report:?}");
        let diagnostic = report
            .diagnostics()
            .iter()
            .find(|diagnostic| diagnostic.action() == CssRecoveryAction::RejectInput)
            .expect("whole selector rejection");
        assert_eq!(diagnostic.error().code(), CssErrorCode::InvalidSelector);
        let ErrorKind::InvalidSelector(detail) = diagnostic.error().kind() else {
            panic!("selector diagnostic")
        };
        assert_eq!(detail.encountered().unwrap().kind(), kind);
        let expected_offset = source.find(offending).unwrap();
        assert_eq!(
            diagnostic.error().position().byte_offset().value(),
            expected_offset
        );
        assert_eq!(diagnostic.error().position().line().value(), 1);
        let line_start = source.find('\n').unwrap() + 1;
        assert_eq!(
            diagnostic.error().position().column().value(),
            u32::try_from(source[line_start..expected_offset].encode_utf16().count()).unwrap()
        );
        assert!(report.into_validation_result().is_err());
    }
}

#[test]
fn an_outer_forgiving_list_cannot_swallow_a_lexically_invalid_inner_envelope() {
    for source in [":is(.ok,:where(.a{))", ":where(.ok,:is(url(a b)))"] {
        assert_rejected(source);
    }
}

#[test]
fn a_lexically_valid_nested_invalid_selector_still_allows_the_outer_valid_sibling() {
    // :not() is unforgiving, so its invalid :unknown() member invalidates that
    // whole outer-list member. The lexical envelope itself remains well formed.
    for source in [
        ":is(.ok,:not(:unknown(x)))",
        ":where(.ok,:not(:unknown(x)))",
    ] {
        let report = parse_selector(source, &CssNamespaceContext::default());
        assert_eq!(
            retained_members(report.syntax()),
            [CssSelector::Class("ok".into())]
        );
        assert!(
            report
                .diagnostics()
                .iter()
                .any(|diagnostic| diagnostic.action() == CssRecoveryAction::DropSelectorListItem)
        );
        assert!(!report.diagnostics().iter().any(|diagnostic| matches!(
            diagnostic.action(),
            CssRecoveryAction::RejectInput | CssRecoveryAction::RetainWithImplicitClosure
        )));
    }
}
