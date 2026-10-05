#![forbid(unsafe_code)]
//! CSS Syntax 3 §§4.3.1 and 4.3.7 require escape parse errors while preserving
//! delimiter/replacement recovery. Literal coordinates below independently
//! specify original UTF-8 bytes, zero-based lines and UTF-16 columns.

use surgeist_css::{
    CssErrorCode, CssEscapeError, CssRecoveryAction, CssRecoveryDiagnostic, ErrorKind,
    parse_declaration, parse_sheet, parse_style_attribute, validate_style_attribute,
};

fn assert_escape(
    diagnostic: &CssRecoveryDiagnostic,
    expected: CssEscapeError,
    position: (usize, u32, u32),
    span: std::ops::Range<usize>,
) {
    assert_eq!(diagnostic.action(), CssRecoveryAction::RecoverEscape);
    assert_eq!(diagnostic.error().code(), CssErrorCode::EscapeParseError);
    let ErrorKind::EscapeParseError(actual) = diagnostic.error().kind() else {
        panic!("typed escape error: {:?}", diagnostic.error());
    };
    assert_eq!(*actual, expected);
    let actual = diagnostic.error().position();
    assert_eq!(actual.byte_offset().value(), position.0);
    assert_eq!(actual.line().value(), position.1);
    assert_eq!(actual.column().value(), position.2);
    assert_eq!(diagnostic.span().start().byte_offset().value(), span.start);
    assert_eq!(diagnostic.span().end().byte_offset().value(), span.end);
}

#[test]
fn newline_escape_diagnostic_owns_the_backslash_before_original_crlf() {
    let source = "--x:😀\\\r\n;--after:kept";
    let report = parse_style_attribute(source);
    assert_eq!(report.syntax().len(), 2);
    let [diagnostic] = report.diagnostics() else {
        panic!("one lexical event: {report:?}");
    };
    assert_escape(diagnostic, CssEscapeError::Newline, (8, 0, 6), 8..9);
    assert_eq!(diagnostic.span().start().column().value(), 6);
    assert_eq!(diagnostic.span().end().column().value(), 7);
    assert_eq!(&source[8..9], "\\");
    let failure = validate_style_attribute(source).unwrap_err();
    assert_eq!(failure.diagnostics(), report.diagnostics());
}

#[test]
fn escaped_eof_diagnostic_owns_the_final_backslash_and_original_eof_cursor() {
    let source = "--x:😀a\\";
    let report = parse_style_attribute(source);
    assert_eq!(report.syntax().len(), 1);
    let [diagnostic] = report.diagnostics() else {
        panic!("one EOF escape event: {report:?}");
    };
    assert_escape(diagnostic, CssEscapeError::EndOfInput, (10, 0, 8), 9..10);
    assert_eq!(diagnostic.span().start().column().value(), 7);
    assert_eq!(diagnostic.span().end().column().value(), 8);
    assert!(validate_style_attribute(source).is_err());
}

#[test]
fn multiple_escape_events_are_published_once_in_original_source_order() {
    let source = "--a:\\\n;--b:\\\r\n;--c:a\\";
    let report = parse_style_attribute(source);
    assert_eq!(report.syntax().len(), 3);
    let [first, second, third] = report.diagnostics() else {
        panic!("exactly three lexical events: {report:?}");
    };
    assert_escape(first, CssEscapeError::Newline, (4, 0, 4), 4..5);
    assert_escape(second, CssEscapeError::Newline, (11, 1, 5), 11..12);
    assert_escape(third, CssEscapeError::EndOfInput, (21, 2, 7), 20..21);
    assert!(validate_style_attribute(source).is_err());
}

#[test]
fn lexical_recovery_remains_observable_when_surrounding_grammar_drops_a_value() {
    let source = "width:\\\n;--after:kept";
    let report = parse_style_attribute(source);
    assert_eq!(report.syntax().len(), 1);
    assert!(
        report
            .diagnostics()
            .iter()
            .any(|diagnostic| diagnostic.action() == CssRecoveryAction::DropDeclaration)
    );
    let lexical = report
        .diagnostics()
        .iter()
        .filter(|diagnostic| diagnostic.action() == CssRecoveryAction::RecoverEscape)
        .collect::<Vec<_>>();
    let [diagnostic] = lexical.as_slice() else {
        panic!("one independently published lexical event");
    };
    assert_escape(diagnostic, CssEscapeError::Newline, (6, 0, 6), 6..7);
    assert!(
        report
            .diagnostics()
            .windows(2)
            .all(|pair| pair[0].error().position().byte_offset()
                <= pair[1].error().position().byte_offset())
    );
}

#[test]
fn recursive_sheet_and_fragment_parsing_publish_each_escape_once() {
    let source = ".a{--x:f(a\\\n)}";
    let report = parse_sheet(source);
    assert_eq!(report.syntax().rules().len(), 1);
    let [diagnostic] = report.diagnostics() else {
        panic!("one public-entry lexical event: {report:?}");
    };
    assert_escape(diagnostic, CssEscapeError::Newline, (10, 0, 10), 10..11);
    let fragment = parse_declaration("--x:a\\");
    assert!(fragment.syntax().is_some());
    let [diagnostic] = fragment.diagnostics() else {
        panic!("one fragment lexical event");
    };
    assert_escape(diagnostic, CssEscapeError::EndOfInput, (6, 0, 6), 5..6);
}

#[test]
fn strings_comments_and_url_payloads_do_not_publish_escape_events() {
    let clean = "--s:\"a\\\nb\";--c:/*\\\n*/kept;--u:url(a\\\\);--after:kept";
    let report = parse_style_attribute(clean);
    assert!(report.is_clean(), "{report:?}");
    assert_eq!(report.syntax().len(), 4);
    for source in [
        "--u:url(a\\\nb);--after:kept",
        "--x:/*\\",
        "--x:\"a\\",
        "--x:url(a\\",
    ] {
        let report = parse_style_attribute(source);
        assert!(
            !report.is_clean(),
            "the owning token already requires recovery: {source:?}"
        );
        assert_eq!(
            report.diagnostics().len(),
            1,
            "no extra escape diagnostic: {source:?}: {report:?}"
        );
        assert!(
            report
                .diagnostics()
                .iter()
                .all(|diagnostic| diagnostic.action() != CssRecoveryAction::RecoverEscape)
        );
    }
}

#[test]
fn escaped_backslash_pairs_and_hex_escape_whitespace_are_clean_controls() {
    for value in ["a\\\\", "a\\\\\n", "\\61\n", "#a\\\\", "@a\\\\", "1p\\\\"] {
        let source = format!("--x:{value};--after:kept");
        let report = parse_style_attribute(&source);
        assert!(report.is_clean(), "{source:?}: {report:?}");
        assert_eq!(report.syntax().len(), 2);
    }
    let source = "--x:a\\\\\\\n;--after:kept";
    let report = parse_style_attribute(source);
    let [diagnostic] = report.diagnostics() else {
        panic!("only the final unmatched backslash is invalid");
    };
    assert_escape(diagnostic, CssEscapeError::Newline, (7, 0, 7), 7..8);
}
