//! The tokens the preludes of `@keyframes` and `@layer` and the descriptors of
//! `@page` are read as, following Syntax 3 (2026 CRD) §4.3.
//!
//! - §4.3.7 and §4.3.8: a backslash followed by a newline is not a valid escape,
//!   so it never continues the token before it; it is a delimiter token and a
//!   parse error at the backslash.
//! - §4.3.9: `-` followed by `\` and a newline does not start an ident
//!   sequence, so `-` is a delimiter token; `#-` is a hash token holding `-`
//!   (a name code point), `1-` is a number followed by that delimiter, and `@-`
//!   is a delimiter followed by it (an at-keyword needs an ident sequence).

use surgeist_css::CssParser;
use surgeist_css::diagnostic::CssErrorCode;

/// The first diagnostic of `source` that is not the escape error, as its code,
/// the kind and spelling of the token it names, and its byte; and the byte of
/// the escape error.
fn rejection(source: &str) -> (CssErrorCode, String, String, usize, usize) {
    let report = CssParser::default().parse_sheet(source);
    let diagnostics = report.diagnostics();
    let escapes: Vec<_> = diagnostics
        .iter()
        .filter(|diagnostic| diagnostic.error().code() == CssErrorCode::EscapeParseError)
        .map(|diagnostic| diagnostic.error().position().byte_offset().value())
        .collect();
    let [escape] = escapes.as_slice() else {
        panic!("one escape error: {source:?}: {diagnostics:?}");
    };
    let rejected: Vec<_> = diagnostics
        .iter()
        .filter(|diagnostic| diagnostic.error().code() != CssErrorCode::EscapeParseError)
        .collect();
    let [rejected] = rejected.as_slice() else {
        panic!("one rejection: {source:?}: {diagnostics:?}");
    };
    let error = rejected.error();
    let token = error.encountered().expect("the rejected token");
    (
        error.code(),
        format!("{:?}", token.kind()),
        token.authored().to_owned(),
        error.position().byte_offset().value(),
        *escape,
    )
}

#[test]
fn a_backslash_before_a_newline_never_continues_the_token_before_it_in_a_keyframe_selector() {
    for (source, code, kind, authored, byte, escape) in [
        (
            "@keyframes k { -\\\n { color: red } }",
            CssErrorCode::UnexpectedToken,
            "Delim",
            "-",
            15,
            16,
        ),
        (
            "@keyframes k { from, -\\\n { color: red } }",
            CssErrorCode::UnexpectedToken,
            "Delim",
            "-",
            21,
            22,
        ),
        (
            "@keyframes k { #-\\\n { color: red } }",
            CssErrorCode::UnexpectedToken,
            "Hash",
            "#-",
            15,
            17,
        ),
        (
            "@keyframes k { @-\\\n { color: red } }",
            CssErrorCode::UnexpectedToken,
            "Delim",
            "@",
            15,
            17,
        ),
        (
            "@keyframes k { 1-\\\n { color: red } }",
            CssErrorCode::InvalidQualifiedRule,
            "Number",
            "1",
            15,
            17,
        ),
    ] {
        let (found, found_kind, spelling, offset, escape_at) = rejection(source);
        assert_eq!(found, code, "{source:?}");
        assert_eq!(found_kind, kind, "{source:?}");
        assert_eq!(spelling, authored, "{source:?}");
        assert_eq!(offset, byte, "{source:?}");
        assert_eq!(escape_at, escape, "{source:?}");
    }
}

#[test]
fn a_backslash_before_a_newline_never_continues_the_token_before_it_in_a_layer_prelude() {
    for (source, kind, authored, byte, escape) in [
        ("@layer -\\\n;", "Delim", "-", 7, 8),
        ("@layer a, -\\\n;", "Delim", "-", 10, 11),
        ("@layer a.-\\\n;", "Delim", "-", 9, 10),
        ("@layer #-\\\n;", "Hash", "#-", 7, 9),
        ("@layer 1-\\\n;", "Number", "1", 7, 9),
    ] {
        let (code, found_kind, spelling, offset, escape_at) = rejection(source);
        assert_eq!(code, CssErrorCode::InvalidAtRulePrelude, "{source:?}");
        assert_eq!(found_kind, kind, "{source:?}");
        assert_eq!(spelling, authored, "{source:?}");
        assert_eq!(offset, byte, "{source:?}");
        assert_eq!(escape_at, escape, "{source:?}");
    }
}

#[test]
fn a_backslash_before_a_newline_never_continues_the_token_before_it_in_a_page_descriptor() {
    for (source, kind, authored, byte, escape) in [
        ("@page { size: -\\\n }", "Delim", "-", 14, 15),
        ("@page { size: a4 -\\\n }", "Delim", "-", 17, 18),
        ("@page { size: #-\\\n }", "Hash", "#-", 14, 16),
        ("@page { size: 1-\\\n }", "Number", "1", 14, 16),
        ("@page { size: @-\\\n }", "Delim", "@", 14, 16),
        ("@page { marks: crop -\\\n }", "Delim", "-", 20, 21),
        ("@page { bleed: 1px -\\\n }", "Delim", "-", 19, 20),
        ("@page { page-orientation: -\\\n }", "Delim", "-", 26, 27),
    ] {
        let (code, found_kind, spelling, offset, escape_at) = rejection(source);
        assert_eq!(code, CssErrorCode::InvalidDescriptorValue, "{source:?}");
        assert_eq!(found_kind, kind, "{source:?}");
        assert_eq!(spelling, authored, "{source:?}");
        assert_eq!(offset, byte, "{source:?}");
        assert_eq!(escape_at, escape, "{source:?}");
    }
}

#[test]
fn a_backslash_before_a_newline_never_continues_the_token_before_it_in_a_page_selector() {
    for (source, kind, authored, byte, escape) in [
        ("@page -\\\n { size: a4 }", "Delim", "-", 6, 7),
        ("@page a, -\\\n { size: a4 }", "Delim", "-", 9, 10),
        ("@page :-\\\n { size: a4 }", "Delim", "-", 7, 8),
        // The ident `a-` is the name; the backslash after it is the delimiter.
        ("@page a-\\\n { size: a4 }", "Delim", "\\", 8, 8),
        // The ident `first-` is no page pseudo-class.
        ("@page :first-\\\n { size: a4 }", "Ident", "first-", 7, 13),
        ("@page #-\\\n { size: a4 }", "Hash", "#-", 6, 8),
        ("@page 1-\\\n { size: a4 }", "Number", "1", 6, 8),
        ("@page @-\\\n { size: a4 }", "Delim", "@", 6, 8),
    ] {
        let (code, found_kind, spelling, offset, escape_at) = rejection(source);
        assert_eq!(code, CssErrorCode::InvalidAtRulePrelude, "{source:?}");
        assert_eq!(found_kind, kind, "{source:?}");
        assert_eq!(spelling, authored, "{source:?}");
        assert_eq!(offset, byte, "{source:?}");
        assert_eq!(escape_at, escape, "{source:?}");
    }
}

#[test]
fn a_backslash_before_a_newline_never_continues_the_token_before_it_in_a_keyframes_name() {
    for (source, kind, authored, byte, escape) in [
        (
            "@keyframes -\\\n { to { color: red } }",
            "Delim",
            "-",
            11,
            12,
        ),
        (
            "@keyframes #-\\\n { to { color: red } }",
            "Hash",
            "#-",
            11,
            13,
        ),
        (
            "@keyframes 1-\\\n { to { color: red } }",
            "Number",
            "1",
            11,
            13,
        ),
        (
            "@keyframes @-\\\n { to { color: red } }",
            "Delim",
            "@",
            11,
            13,
        ),
        // The ident `k-` is the name; the backslash after it is the delimiter.
        (
            "@keyframes k-\\\n { to { color: red } }",
            "Delim",
            "\\",
            13,
            13,
        ),
        // A string is the name; the hyphen after it is another token.
        (
            "@keyframes \"k\"-\\\n { to { color: red } }",
            "Delim",
            "-",
            14,
            15,
        ),
    ] {
        let (code, found_kind, spelling, offset, escape_at) = rejection(source);
        assert_eq!(code, CssErrorCode::InvalidAtRulePrelude, "{source:?}");
        assert_eq!(found_kind, kind, "{source:?}");
        assert_eq!(spelling, authored, "{source:?}");
        assert_eq!(offset, byte, "{source:?}");
        assert_eq!(escape_at, escape, "{source:?}");
    }
}
