#![forbid(unsafe_code)]
//! Pinned Selectors 4 (2026-01-22), sections 7.1 and 7.2:
//! https://www.w3.org/TR/2026/WD-selectors-4-20260122/#the-dir-pseudo
//! https://www.w3.org/TR/2026/WD-selectors-4-20260122/#the-lang-pseudo
//! Direction takes one identifier; language takes a nonempty comma-separated
//! list of identifiers or strings. These tests preserve authored syntax without
//! evaluating directionality, language matching, or document inheritance.
use surgeist_css::{
    CssErrorCode, CssNamespaceContext, CssPseudoClass, CssRecoveryAction, CssSelector,
    parse_selector,
};

fn assert_clean_pseudo(source: &str) {
    let report = parse_selector(source, &CssNamespaceContext::default());
    assert!(report.is_clean(), "{source}: {report:?}");
    assert!(
        matches!(report.syntax(), Some(CssSelector::PseudoClass(_))),
        "{source}: {report:?}"
    );
    assert!(report.into_validation_result().is_ok(), "{source}");
}
fn assert_rejected(source: &str) {
    let report = parse_selector(source, &CssNamespaceContext::default());
    assert!(report.syntax().is_none(), "{source}: {report:?}");
    let [diagnostic] = report.diagnostics() else {
        panic!("one complete rejection: {source}: {report:?}")
    };
    assert_eq!(
        diagnostic.error().code(),
        CssErrorCode::InvalidSelector,
        "{source}"
    );
    assert_eq!(
        diagnostic.action(),
        CssRecoveryAction::RejectInput,
        "{source}"
    );
    assert_eq!(diagnostic.span().start().byte_offset().value(), 0);
    assert_eq!(diagnostic.span().end().byte_offset().value(), source.len());
    assert!(report.into_validation_result().is_err(), "{source}");
}

#[test]
fn original_direction_identifiers_are_retained_cleanly() {
    for source in [":dir(ltr)", ":dir(  ltr /* test */ )", ":DiR(ltr)"] {
        assert_clean_pseudo(source);
    }
}
#[test]
fn direction_preserves_valid_identifier_syntax_without_evaluating_matching() {
    // Other identifier values are syntactically valid, even though they do not
    // match an element under the currently defined directionality semantics.
    for source in [
        ":dir(rtl)",
        ":dir(LTR)",
        ":dir(future)",
        r":dir(\6c tr)",
        r":d\69 r(rtl)",
    ] {
        assert_clean_pseudo(source);
    }
}
#[test]
fn direction_rejects_original_and_additional_invalid_parameter_shapes() {
    for source in [
        ":dir(1)",
        ":dir(foo var)",
        ":dir()",
        ":dir('ltr')",
        ":dir(ltr,rtl)",
        ":dir(var(--direction))",
    ] {
        assert_rejected(source);
    }
}
#[test]
fn original_identifier_language_controls_preserve_decoded_payload() {
    for (source, expected) in [
        (":lang(en)", "en"),
        (":lang(  en /* test */ )", "en"),
        (":LanG(en-US)", "en-US"),
        (r":lang(\65 n)", "en"),
    ] {
        let report = parse_selector(source, &CssNamespaceContext::default());
        assert!(report.is_clean(), "{source}: {report:?}");
        let Some(CssSelector::PseudoClass(CssPseudoClass::Lang(range))) = report.syntax() else {
            panic!("language pseudo: {source}: {report:?}")
        };
        assert_eq!(range.ranges()[0].as_str(), expected);
    }
}
#[test]
fn original_quoted_language_range_is_retained_cleanly() {
    assert_clean_pseudo(":lang('en')");
}
#[test]
fn original_mixed_language_list_is_retained_cleanly() {
    assert_clean_pseudo(":lang( 'en', de-DE ,  \"es\" )");
}
#[test]
fn escaped_strings_and_comma_separated_language_ranges_are_valid() {
    for source in [
        r":lang('\65 n')",
        r":l\61 ng(en)",
        ":lang(en,fr)",
        ":lang( en /**/, 'fr' )",
        ":lang('*')",
        ":lang(\"*-Latn\")",
    ] {
        assert_clean_pseudo(source);
    }
}
#[test]
fn empty_string_and_escaped_wildcard_are_language_range_syntax() {
    // Section 7.2 explicitly defines :lang("") for untagged language and
    // gives the escaped wildcard identifier as an alternative to a string.
    // Here only syntax retention is asserted; matching remains unresolved.
    for source in [r#":lang("")"#, r":lang(\*-Latn)"] {
        assert_clean_pseudo(source);
    }
}

#[test]
fn language_rejects_original_invalid_members_and_missing_commas() {
    // The upstream "not an error" label on :lang(en en) does not override the
    // independently selected comma-list grammar.
    for source in [
        ":lang(en en)",
        ":lang(1)",
        ":lang(var(--test)) {}",
        ":lang()",
        ":lang(,en)",
        ":lang(en,)",
        ":lang(en,,fr)",
        ":lang(*)",
        ":lang('en' 'fr')",
        ":lang(var(--test))",
    ] {
        assert_rejected(source);
    }
}
#[test]
fn invalid_language_token_coordinates_refer_to_original_unicode_source() {
    // The argument is a number token, whose original start is responsible for
    // rejection. Byte offsets and UTF-16 columns are different after the emoji.
    for source in [":lang(1)", "/*😀*/:lang(1)"] {
        assert_rejected(source);
        let report = parse_selector(source, &CssNamespaceContext::default());
        let position = report.diagnostics()[0].error().position();
        let offset = source.find('1').unwrap();
        assert_eq!(position.byte_offset().value(), offset);
        assert_eq!(position.line().value(), 0);
        assert_eq!(
            position.column().value(),
            u32::try_from(source[..offset].encode_utf16().count()).unwrap()
        );
    }
}
