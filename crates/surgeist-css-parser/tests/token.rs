//! The token types: `Token`, `TokenKind`, `Number` and `CodePointRange`.
//!
//! The expectations come from CSS Syntax Module Level 3 section 4 (pinned
//! edition) and section 9 for the comment extension: a number token has a
//! numeric value, an optional sign character and a type flag; a
//! `<unicode-range-token>` has a starting and an ending code point, and an end
//! before the start is an empty range. A number's value is what its spelling
//! says, computed by hand here.

use surgeist_css_parser::lexer::{Lexer, Options};
use surgeist_css_parser::token::{HashKind, NumberKind, Sign, TokenKind};
use surgeist_css_parser::unicode_range::CodePointRange;
use surgeist_css_parser::{Offset, Token};

/// What the lexer's number token for `text` says. `text` is one number and
/// nothing else, so the first lexeme is a number token and the only one.
fn number_of(text: &str) -> (String, Option<Sign>, NumberKind, f64, Option<i64>) {
    let mut lexer = Lexer::new(text, &Options::default());
    let Some(Token::Number(number)) = lexer.next_lexeme().map(|lexeme| lexeme.token()) else {
        panic!("{text:?} is not a number token");
    };
    let facts = (
        number.representation().to_owned(),
        number.sign(),
        number.kind(),
        number.to_f64(),
        number.integer(),
    );
    assert!(
        lexer.next_lexeme().is_none(),
        "{text:?} is more than a number"
    );
    facts
}

#[test]
fn a_number_keeps_its_exact_spelling_sign_and_type_flag() {
    // 4.3.13: the type flag is "integer" unless a fraction or an exponent was
    // consumed; the sign character is the "+" or "-" that was consumed.
    let cases: &[(&str, Option<Sign>, NumberKind)] = &[
        ("12", None, NumberKind::Integer),
        ("+12", Some(Sign::Plus), NumberKind::Integer),
        ("-12", Some(Sign::Minus), NumberKind::Integer),
        ("007", None, NumberKind::Integer),
        ("1.5", None, NumberKind::Number),
        (".5", None, NumberKind::Number),
        ("-.5", Some(Sign::Minus), NumberKind::Number),
        ("1e3", None, NumberKind::Number),
        ("1E+3", None, NumberKind::Number),
        ("+1.25e-2", Some(Sign::Plus), NumberKind::Number),
    ];
    for (text, sign, kind) in cases {
        let (representation, found_sign, found_kind, ..) = number_of(text);
        assert_eq!(representation, *text);
        assert_eq!(found_sign, *sign, "{text}");
        assert_eq!(found_kind, *kind, "{text}");
    }
}

#[test]
fn a_number_converts_to_the_value_its_spelling_states() {
    // Values worked out from the spelling: digits, fraction and power of ten.
    let cases: &[(&str, f64)] = &[
        ("0", 0.0),
        ("12", 12.0),
        ("-12", -12.0),
        ("+7", 7.0),
        ("1.5", 1.5),
        (".25", 0.25),
        ("-.5", -0.5),
        ("1e3", 1000.0),
        ("1E3", 1000.0),
        ("1.5e3", 1500.0),
        ("25e-2", 0.25),
        ("+1.25e+2", 125.0),
        ("5e0", 5.0),
        ("0.0", 0.0),
        ("1e-400", 0.0),
        ("1e400", f64::INFINITY),
        ("-1e400", f64::NEG_INFINITY),
    ];
    for (text, value) in cases {
        assert_eq!(number_of(text).3, *value, "{text}");
    }
    // The spelling of a negative zero keeps its sign.
    let zero = number_of("-0").3;
    assert_eq!(zero, 0.0);
    assert!(zero.is_sign_negative());
}

#[test]
fn an_integer_is_available_only_for_the_integer_flag_and_when_it_fits() {
    let cases: &[(&str, Option<i64>)] = &[
        ("0", Some(0)),
        ("42", Some(42)),
        ("+42", Some(42)),
        ("-42", Some(-42)),
        ("007", Some(7)),
        ("9223372036854775807", Some(i64::MAX)),
        ("-9223372036854775808", Some(i64::MIN)),
        ("9223372036854775808", None),
        ("-9223372036854775809", None),
        ("1.0", None),
        ("1e3", None),
        (".5", None),
    ];
    for (text, expected) in cases {
        assert_eq!(number_of(text).4, *expected, "{text}");
    }
}

#[test]
fn every_token_reports_its_kind_and_only_whitespace_and_comments_are_trivia() {
    let mut lexer = Lexer::new("1", &Options::default());
    let Some(Token::Number(number)) = lexer.next_lexeme().map(|lexeme| lexeme.token()) else {
        panic!("1 is a number token");
    };
    let range = CodePointRange::new(0x26, 0x26);
    let all: [(Token<'_>, TokenKind); 26] = [
        (Token::Ident("a"), TokenKind::Ident),
        (Token::Function("f"), TokenKind::Function),
        (Token::AtKeyword("media"), TokenKind::AtKeyword),
        (
            Token::Hash {
                value: "h",
                kind: HashKind::Id,
            },
            TokenKind::Hash,
        ),
        (Token::String("s"), TokenKind::String),
        (Token::BadString, TokenKind::BadString),
        (Token::Url("u"), TokenKind::Url),
        (Token::BadUrl, TokenKind::BadUrl),
        (Token::Delim('!'), TokenKind::Delim),
        (Token::Number(number), TokenKind::Number),
        (Token::Percentage(number), TokenKind::Percentage),
        (
            Token::Dimension { number, unit: "px" },
            TokenKind::Dimension,
        ),
        (Token::UnicodeRange(range), TokenKind::UnicodeRange),
        (Token::Whitespace, TokenKind::Whitespace),
        (Token::Comment, TokenKind::Comment),
        (Token::Colon, TokenKind::Colon),
        (Token::Semicolon, TokenKind::Semicolon),
        (Token::Comma, TokenKind::Comma),
        (Token::OpenSquare, TokenKind::OpenSquare),
        (Token::OpenParen, TokenKind::OpenParen),
        (Token::OpenCurly, TokenKind::OpenCurly),
        (Token::CloseSquare, TokenKind::CloseSquare),
        (Token::CloseParen, TokenKind::CloseParen),
        (Token::CloseCurly, TokenKind::CloseCurly),
        (Token::Cdo, TokenKind::Cdo),
        (Token::Cdc, TokenKind::Cdc),
    ];
    for (token, kind) in all {
        assert_eq!(token.kind(), kind, "{token:?}");
        assert_eq!(
            token.is_trivia(),
            matches!(kind, TokenKind::Whitespace | TokenKind::Comment),
            "{token:?}"
        );
    }
}

#[test]
fn a_unicode_range_holds_its_bounds_and_an_end_before_the_start_is_empty() {
    // 4: "If the ending code point is before the starting code point, it
    // represents an empty range." A bound is what up to six hexadecimal digits
    // spell, so it may exceed the maximum allowed code point.
    let range = CodePointRange::new(0x400, 0x4ff);
    assert_eq!((range.start(), range.end()), (0x400, 0x4ff));
    assert!(!range.is_empty());
    assert!(!CodePointRange::new(0x26, 0x26).is_empty());
    assert!(CodePointRange::new(0x20, 0x1f).is_empty());
    assert!(!CodePointRange::new(0x110000, 0xffffff).is_empty());
}

#[test]
fn options_are_a_value_with_a_default_that_allows_nothing_and_rebases_nothing() {
    let options = Options::default();
    assert!(!options.allows_unicode_ranges());
    assert_eq!(options.origin_offset(), Offset::new(0));
    let changed = options.unicode_ranges(true).origin(Offset::new(7));
    assert!(changed.allows_unicode_ranges());
    assert_eq!(changed.origin_offset(), Offset::new(7));
    // The builders take the value and leave the original as it was.
    assert!(!options.allows_unicode_ranges());
    assert_ne!(changed, options);
}
