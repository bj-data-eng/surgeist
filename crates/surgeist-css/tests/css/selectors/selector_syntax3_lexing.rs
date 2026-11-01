//! The tokens a selector is read as, following Syntax 3 (2026 CRD) §4.3.
//!
//! - §4.3.7 and §4.3.8: a backslash followed by a newline is not a valid escape,
//!   so it never continues an identifier or a class name; it is a delimiter
//!   token and a parse error at the backslash.
//! - §4.3.9: `-` followed by `\` and a newline does not start an ident sequence,
//!   so `-` is a delimiter token, and no selector starts with that delimiter.
//! - §4.3.1: `~=` is not a token; `~` and `=` are delimiter tokens. A selector
//!   that is rejected at such a pair names the first delimiter.
//! - Selectors 4: `<combinator>` includes `'|' '|'` and `<attr-matcher>` is
//!   `['~' | '|' | '^' | '$' | '*']? '='`. A pair of delimiters that makes one of
//!   those units is one unit, and when it is the unit a selector is rejected at,
//!   the diagnostic names its first delimiter, wherever the unit stands.
//! - Syntax 3 §6.2: `<an+b>` consumes a whole dimension token, so a dimension
//!   that is no `n` form is the unit a notation is rejected at, and a comment
//!   between two tokens of a notation belongs to neither.

use surgeist_css::CssParser;
use surgeist_css::diagnostic::CssErrorCode;
use surgeist_css::selector::{CssNamespaceContext, CssSelector};

/// The kind and spelling of the token the whole-input selector `source` is
/// rejected at, and the byte of the diagnostic.
fn rejection(source: &str) -> (String, String, usize) {
    let report = CssSelector::parse(
        &CssParser::default(),
        source,
        &CssNamespaceContext::default(),
    );
    let [diagnostic] = report.diagnostics() else {
        panic!("one rejection: {source:?}: {:?}", report.diagnostics());
    };
    let token = diagnostic
        .error()
        .encountered()
        .expect("the rejected token");
    (
        format!("{:?}", token.kind()),
        token.authored().to_owned(),
        diagnostic.error().position().byte_offset().value(),
    )
}

#[test]
fn a_hyphen_before_a_backslash_newline_is_a_delimiter_that_no_selector_starts_with() {
    for newline in ["\n", "\r\n", "\r", "\u{c}"] {
        let source = format!("-\\{newline} {{ a: b }}");
        let report = CssParser::default().parse_sheet(&source);
        let rejected: Vec<_> = report
            .diagnostics()
            .iter()
            .filter(|diagnostic| diagnostic.error().code() == CssErrorCode::InvalidSelector)
            .collect();
        assert_eq!(rejected.len(), 1, "{source:?}");
        let token = rejected[0]
            .error()
            .encountered()
            .expect("the rejected token");
        assert_eq!(format!("{:?}", token.kind()), "Delim", "{source:?}");
        assert_eq!(token.authored(), "-", "{source:?}");
        let escapes: Vec<_> = report
            .diagnostics()
            .iter()
            .filter(|diagnostic| diagnostic.error().code() == CssErrorCode::EscapeParseError)
            .map(|diagnostic| diagnostic.error().position().byte_offset().value())
            .collect();
        assert_eq!(escapes, [1], "{source:?}");
    }
}

#[test]
fn a_class_name_does_not_continue_past_a_backslash_newline() {
    let source = ".a-\\\n { x: y }";
    let report = CssParser::default().parse_sheet(source);
    assert!(
        report
            .diagnostics()
            .iter()
            .any(|diagnostic| diagnostic.error().code() == CssErrorCode::InvalidSelector),
        "{source:?}"
    );
}

#[test]
fn a_matcher_outside_an_attribute_selector_is_rejected_at_its_first_delimiter() {
    let source = "a ~= b { x: y }";
    let report = CssParser::default().parse_sheet(source);
    let rejected: Vec<_> = report
        .diagnostics()
        .iter()
        .filter(|diagnostic| diagnostic.error().code() == CssErrorCode::InvalidSelector)
        .collect();
    assert_eq!(rejected.len(), 1, "{source:?}");
    let token = rejected[0]
        .error()
        .encountered()
        .expect("the rejected token");
    assert_eq!(format!("{:?}", token.kind()), "Delim", "{source:?}");
    assert_eq!(token.authored(), "~", "{source:?}");
    assert_eq!(rejected[0].error().position().byte_offset().value(), 2);
}

#[test]
fn a_pair_of_delimiters_that_makes_one_selector_unit_is_named_by_its_first_delimiter() {
    for (source, spelling, byte) in [
        // `||` and a matcher: the bars are named, the first one.
        (".b||='x'", "|", 2),
        ("&||=$a", "|", 1),
        (":not( .b&)||=", "|", 10),
        ("a:not(.c||=)", "|", 8),
        // `*=` after a namespace prefix is one unit; the star is named.
        ("i|*=*", "*", 2),
        ("i|*=#c+|]", "*", 2),
        // An attribute matcher where a compound begins, or follows one.
        ("a *= b", "*", 2),
        ("a ~= b", "~", 2),
        ("a|=b", "|", 1),
        (":nth-child(~=)", "~", 13),
        (":nth-child(|=)", "|", 13),
        (":nth-child(*=)", "*", 13),
        (":nth-child(^=)", "^", 13),
        (":nth-child($=)", "$", 13),
        (":nth-child(~=x)", "~", 13),
    ] {
        let (kind, authored, offset) = rejection(source);
        assert_eq!(kind, "Delim", "{source:?}");
        assert_eq!(authored, spelling, "{source:?}");
        assert_eq!(offset, byte, "{source:?}");
    }
}

#[test]
fn a_dimension_that_is_no_notation_is_the_unit_an_nth_argument_is_rejected_at() {
    for (source, kind, authored, byte) in [
        ("a:nth-child(2x)", "Dimension", "2x", 14),
        (":nth-child(2nn of a, b)", "Dimension", "2nn", 14),
    ] {
        let (found, spelling, offset) = rejection(source);
        assert_eq!(
            (found.as_str(), spelling.as_str(), offset),
            (kind, authored, byte),
            "{source:?}"
        );
    }
}

#[test]
fn a_comment_inside_a_notation_belongs_to_neither_token_around_it() {
    for (source, kind, authored, byte) in [
        // The token after the plus sign is named where it ends, as without the comment.
        (":nth-child(+/**/1)", "Number", "1", 17),
        (":nth-child(+/**/x)", "Ident", "x", 17),
        (":nth-child(+/**/-n)", "Ident", "-n", 18),
        (":nth-child(+/**/ )", "Whitespace", " ", 17),
        (":nth-child(+/**/)", "CloseParenthesis", ")", 16),
        // An integer too large for `i32` is named as the largest one.
        (":nth-child(+/**/2147483648)", "Number", "2147483647", 26),
        (":nth-child(+ 2n)", "Whitespace", " ", 13),
    ] {
        let (found, spelling, offset) = rejection(source);
        assert_eq!(
            (found.as_str(), spelling.as_str(), offset),
            (kind, authored, byte),
            "{source:?}"
        );
    }
}

#[test]
fn a_matcher_after_the_sign_of_a_notation_is_named_by_its_first_delimiter_after_its_unit() {
    for name in [
        "nth-child",
        "nth-last-child",
        "nth-of-type",
        "nth-last-of-type",
    ] {
        let prefix = format!(":{name}(").len();
        for operator in ["~=", "|=", "^=", "$=", "*="] {
            // `n+` and `n -`: the sign is a delimiter after the bare `n`; `n-` is
            // one ident, which a signless integer must follow.
            for (notation, signed) in [("n+", 2), ("n-", 2), ("n -", 3), ("n + ", 4)] {
                let source = format!(":{name}({notation}{operator}1)");
                let (kind, authored, offset) = rejection(&source);
                assert_eq!(kind, "Delim", "{source:?}");
                assert_eq!(authored, &operator[..1], "{source:?}");
                // The failure is where the unit ends.
                assert_eq!(offset, prefix + signed + 2, "{source:?}");
            }
        }
    }
    // The same in a list that `of` continues.
    let (kind, authored, offset) = rejection(":nth-child(n+|=1 of a)");
    assert_eq!(
        (kind.as_str(), authored.as_str(), offset),
        ("Delim", "|", 15)
    );
    let (kind, authored, offset) = rejection(":nth-child(n+|=|*1)");
    assert_eq!(
        (kind.as_str(), authored.as_str(), offset),
        ("Delim", "|", 15)
    );
}
