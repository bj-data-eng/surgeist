//! Helpers shared by the lexer tests: owned renderings of lexemes.
//!
//! The renderings are a notation for writing expectations by hand from the
//! specification; they carry no behavior of their own.

#![allow(dead_code)]

pub mod digest;
pub mod generate;
pub mod grammar;
pub mod tree;

use std::ops::Range;

use surgeist_css_parser::lexer::{Lexer, Options};
use surgeist_css_parser::token::{HashKind, Number, NumberKind};
use surgeist_css_parser::{Offset, Token};

fn flag(number: Number<'_>) -> &'static str {
    match number.kind() {
        NumberKind::Integer => "int",
        NumberKind::Number => "num",
    }
}

/// Quotes `text` like `Debug` does, except that code points above ASCII are
/// written as themselves, so an expectation can name them directly.
fn quote(text: &str) -> String {
    let mut quoted = String::from("\"");
    for point in text.chars() {
        match point {
            '"' => quoted.push_str("\\\""),
            '\\' => quoted.push_str("\\\\"),
            '\n' => quoted.push_str("\\n"),
            '\r' => quoted.push_str("\\r"),
            '\t' => quoted.push_str("\\t"),
            point if point.is_ascii_control() => {
                quoted.push_str(&format!("\\u{{{:x}}}", u32::from(point)));
            }
            point => quoted.push(point),
        }
    }
    quoted.push('"');
    quoted
}

/// Renders a token as a short Debug-like string, for example `Ident("a")`.
pub fn render(token: Token<'_>) -> String {
    match token {
        Token::Ident(value) => format!("Ident({})", quote(value)),
        Token::Function(value) => format!("Function({})", quote(value)),
        Token::AtKeyword(value) => format!("AtKeyword({})", quote(value)),
        Token::Hash { value, kind } => format!(
            "Hash({},{})",
            quote(value),
            match kind {
                HashKind::Id => "id",
                HashKind::Unrestricted => "unrestricted",
            }
        ),
        Token::String(value) => format!("String({})", quote(value)),
        Token::BadString => "BadString".into(),
        Token::Url(value) => format!("Url({})", quote(value)),
        Token::BadUrl => "BadUrl".into(),
        Token::Delim(value) => format!("Delim({value:?})"),
        Token::Number(number) => {
            format!(
                "Number({},{})",
                quote(number.representation()),
                flag(number)
            )
        }
        Token::Percentage(number) => {
            format!(
                "Percentage({},{})",
                quote(number.representation()),
                flag(number)
            )
        }
        Token::Dimension { number, unit } => format!(
            "Dimension({},{},{})",
            quote(number.representation()),
            flag(number),
            quote(unit)
        ),
        Token::UnicodeRange(range) => {
            format!("UnicodeRange({:#x},{:#x})", range.start(), range.end())
        }
        Token::Whitespace => "Whitespace".into(),
        Token::Comment => "Comment".into(),
        Token::Colon => "Colon".into(),
        Token::Semicolon => "Semicolon".into(),
        Token::Comma => "Comma".into(),
        Token::OpenSquare => "OpenSquare".into(),
        Token::OpenParen => "OpenParen".into(),
        Token::OpenCurly => "OpenCurly".into(),
        Token::CloseSquare => "CloseSquare".into(),
        Token::CloseParen => "CloseParen".into(),
        Token::CloseCurly => "CloseCurly".into(),
        Token::Cdo => "Cdo".into(),
        Token::Cdc => "Cdc".into(),
    }
}

/// One lexeme, owned.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Item {
    pub token: String,
    pub range: Range<usize>,
    pub unterminated: bool,
}

/// An error event, owned: the predicate that holds and the range.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Event {
    pub kind: &'static str,
    pub range: Range<usize>,
}

fn offsets(range: Range<Offset>) -> Range<usize> {
    range.start.value()..range.end.value()
}

/// The name of the one predicate that holds for `error`; panics unless exactly
/// one does.
pub fn kind_of(error: &surgeist_css_parser::lexer::LexicalError) -> &'static str {
    let held: Vec<&'static str> = [
        (error.is_unterminated_comment(), "unterminated_comment"),
        (error.is_unterminated_string(), "unterminated_string"),
        (error.is_bad_string(), "bad_string"),
        (error.is_unterminated_url(), "unterminated_url"),
        (error.is_bad_url(), "bad_url"),
        (error.is_escaped_newline(), "escaped_newline"),
        (error.is_escaped_eof(), "escaped_eof"),
    ]
    .into_iter()
    .filter_map(|(holds, name)| holds.then_some(name))
    .collect();
    assert_eq!(held.len(), 1, "exactly one predicate holds for {error:?}");
    held[0]
}

/// Lexes `text` to the end with `options`.
pub fn lex_with(text: &str, options: &Options) -> (Vec<Item>, Vec<Event>) {
    let mut lexer = Lexer::new(text, options);
    let mut items = Vec::new();
    while let Some(lexeme) = lexer.next_lexeme() {
        items.push(Item {
            token: render(lexeme.token()),
            range: offsets(lexeme.range()),
            unterminated: lexeme.is_unterminated(),
        });
    }
    let events = lexer
        .errors()
        .iter()
        .map(|error| Event {
            kind: kind_of(error),
            range: offsets(error.range()),
        })
        .collect();
    (items, events)
}

pub fn lex_all(text: &str) -> (Vec<Item>, Vec<Event>) {
    lex_with(text, &Options::default())
}

/// The rendered tokens of `text`.
pub fn tokens(text: &str) -> Vec<String> {
    lex_all(text).0.into_iter().map(|item| item.token).collect()
}

/// The rendered tokens of `text` with unicode ranges allowed.
pub fn tokens_with_ranges(text: &str) -> Vec<String> {
    lex_with(text, &Options::default().unicode_ranges(true))
        .0
        .into_iter()
        .map(|item| item.token)
        .collect()
}

/// The events of `text`.
pub fn events(text: &str) -> Vec<Event> {
    lex_all(text).1
}

pub fn event(kind: &'static str, range: Range<usize>) -> Event {
    Event { kind, range }
}

/// Asserts that the lexemes of `text` tile it: the first starts at zero, each
/// starts where the one before it ended, and the last ends at the end.
pub fn assert_tiles(text: &str) {
    let (items, _) = lex_all(text);
    let mut at = 0;
    for item in &items {
        assert_eq!(
            item.range.start, at,
            "gap or overlap in {text:?}: {items:?}"
        );
        assert!(
            item.range.end > item.range.start,
            "empty lexeme in {text:?}"
        );
        at = item.range.end;
    }
    assert_eq!(at, text.len(), "{text:?} not covered: {items:?}");
}
