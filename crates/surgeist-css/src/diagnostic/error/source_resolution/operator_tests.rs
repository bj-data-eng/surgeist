//! Literal source-token diagnostics required by Syntax3 delimiter fallback.
//! Next and previous observers share actual authored one-character boundaries.
use crate::diagnostic::CssTokenKind;
use crate::diagnostic::error::source_resolution::next_authored_token_at;
use crate::diagnostic::error::token_navigation::{LexedText, TokenContext};
use crate::source::CssSourcePosition;

#[test]
fn next_operator_observation_uses_each_authored_delimiter() {
    for (prefix, source) in [
        ('~', "~="),
        ('|', "|="),
        ('^', "^="),
        ('$', "$="),
        ('*', "*="),
    ] {
        for (offset, expected) in [(0, prefix.to_string()), (1, "=".to_owned())] {
            let (start, summary) = next_authored_token_at(
                source,
                CssSourcePosition::from_byte_offset_in(source, offset),
            )
            .unwrap();
            assert_eq!(start, offset);
            assert_eq!(summary.kind(), CssTokenKind::Delim);
            assert_eq!(summary.authored(), expected);
        }
    }
}

#[test]
fn previous_operator_observation_uses_actual_last_delimiter() {
    for source in ["~=", "|=", "^=", "$=", "*="] {
        let tokens = LexedText::new(source);
        let (start, summary) = tokens.previous_token(2).unwrap();
        assert_eq!(start, 1);
        assert_eq!(summary.kind(), CssTokenKind::Delim);
        assert_eq!(summary.authored(), "=");
        let (start, summary) = tokens.previous_token(1).unwrap();
        assert_eq!(start, 0);
        assert_eq!(summary.kind(), CssTokenKind::Delim);
        assert_eq!(summary.authored(), &source[..1]);
    }
}

#[test]
fn diagnostic_token_walkers_preserve_whitespace_and_comment_controls() {
    let source = " \t/**/~ ";
    let (start, summary) =
        next_authored_token_at(source, CssSourcePosition::from_byte_offset_in(source, 0)).unwrap();
    assert_eq!(start, 6);
    assert_eq!(summary.kind(), CssTokenKind::Delim);
    assert_eq!(summary.authored(), "~");
    let source = "x/**/ !";
    let (start, summary) = LexedText::new(source).previous_token(source.len()).unwrap();
    assert_eq!(start, 6);
    assert_eq!(summary.kind(), CssTokenKind::Delim);
    assert_eq!(summary.authored(), "!");
}

/// Syntax 3 section 4.3: punctuation inside a string, a comment or a URL is
/// part of that token, so it bounds nothing. The last token before a byte is
/// the token the lexer read there, whatever such punctuation precedes it.
#[test]
fn punctuation_inside_other_tokens_bounds_no_token() {
    for (source, offset, kind, authored) in [
        // A `;`, `:`, `,`, `(` or `[` in a string or a comment, before the byte.
        ("'a;b' c", 7, CssTokenKind::Ident, "c"),
        ("\"a:b\" c", 7, CssTokenKind::Ident, "c"),
        ("'a,b' c", 7, CssTokenKind::Ident, "c"),
        ("'a(b' c", 7, CssTokenKind::Ident, "c"),
        ("'a[b' c", 7, CssTokenKind::Ident, "c"),
        ("/* ; */ c", 9, CssTokenKind::Ident, "c"),
        ("/* : , ( [ */ c", 15, CssTokenKind::Ident, "c"),
        ("url(a;b) c", 10, CssTokenKind::Ident, "c"),
        // The string or the comment is the last token before the byte when
        // nothing follows it.
        ("a 'x;y'", 7, CssTokenKind::String, "'x;y'"),
        ("a url(x:y)", 10, CssTokenKind::Url, "url(x:y)"),
        // A closing token is the last one of a group that ended before the byte.
        ("f(a;b) ", 7, CssTokenKind::CloseParenthesis, ")"),
        ("a[b,c] ", 7, CssTokenKind::CloseSquareBracket, "]"),
        // The opening of the group the byte is in, when nothing precedes it there.
        ("f(", 2, CssTokenKind::Function, "f("),
        ("a (", 3, CssTokenKind::ParenthesisBlock, "("),
    ] {
        let (_, summary) = LexedText::new(source)
            .previous_token(offset)
            .unwrap_or_else(|| panic!("{source:?} at {offset}"));
        assert_eq!(summary.kind(), kind, "{source:?} at {offset}");
        assert_eq!(summary.authored(), authored, "{source:?} at {offset}");
    }
}

/// Syntax 3 section 5.4.10: whitespace after a comma is a member of the list of
/// its own, which holds no token; a comma is the last token otherwise.
#[test]
fn a_comma_that_whitespace_follows_ends_no_member() {
    assert!(LexedText::new("a, ").previous_token(3).is_none());
    assert!(LexedText::new("a,/**/ b").previous_token(2).is_none());
    let (start, summary) = LexedText::new("a,b").previous_token(2).unwrap();
    assert_eq!((start, summary.authored()), (1, ","));
}

/// The at-keyword before a byte is a token of the tree, not an `@` byte inside
/// a string or a comment.
#[test]
fn the_previous_at_keyword_is_a_token_and_no_at_sign_byte() {
    for source in ["@media '@x' ;", "@media /* @x */ ;", "@media url(@x) ;"] {
        let (start, name) = LexedText::new(source)
            .previous_at_keyword(source.len() - 1)
            .unwrap_or_else(|| panic!("{source:?}"));
        assert_eq!((start, name.as_str()), (0, "media"), "{source:?}");
    }
    assert!(LexedText::new("a '@x' ").previous_at_keyword(7).is_none());
}
