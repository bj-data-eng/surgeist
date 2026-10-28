//! The token of CSS Syntax Module Level 3.

use crate::token::{HashKind, Number, TokenKind};
use crate::unicode_range::CodePointRange;

/// A token of CSS Syntax Module Level 3 section 4, with its decoded value.
///
/// Every text a token holds is a `&'i str`: it borrows the source when the
/// decoded text equals the spelling, and the decode pool of whoever produced the
/// token otherwise, so a `Token` is `Copy` and a consumer never decodes twice.
///
/// The set is closed. It mirrors the normative token list of the pinned
/// edition, with two differences that the specification itself allows or
/// implies:
///
/// - [`Comment`](Token::Comment) is not a Syntax 3 token. Section 9 says the
///   tokenizer produces none and that an implementation may preserve comments
///   and their location provided they have no effect on parsing. `Comment` is
///   that documented extension, and [`is_trivia`](Token::is_trivia) is true of it.
/// - The five attribute-selector operators `~=`, `|=`, `^=`, `$=` and `*=` are
///   not tokens: `~`, `|`, `^`, `$` and `*` are [`Delim`](Token::Delim) tokens
///   and `=` is another.
///
/// A `<bad-string-token>` and a `<bad-url-token>` carry no value, because the
/// specification gives them none; their text is the source range.
///
/// ```
/// use surgeist_css_parser::lexer::{Lexer, Options};
/// use surgeist_css_parser::Token;
///
/// // The lexer lends each token until its next call, so a token is compared
/// // or copied out before the next one is asked for.
/// let mut lexer = Lexer::new("a:b", &Options::default());
/// assert_eq!(lexer.next_lexeme().map(|lexeme| lexeme.token()), Some(Token::Ident("a")));
/// assert_eq!(lexer.next_lexeme().map(|lexeme| lexeme.token()), Some(Token::Colon));
/// assert_eq!(lexer.next_lexeme().map(|lexeme| lexeme.token()), Some(Token::Ident("b")));
/// assert!(lexer.next_lexeme().is_none());
/// ```
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Token<'i> {
    /// An `<ident-token>` with its decoded value.
    Ident(&'i str),
    /// A `<function-token>` with its decoded name, without the parenthesis.
    Function(&'i str),
    /// An `<at-keyword-token>` with its decoded name, without the `@`.
    AtKeyword(&'i str),
    /// A `<hash-token>` with its decoded value, without the `#`, and its type
    /// flag.
    Hash {
        /// The decoded value.
        value: &'i str,
        /// The type flag.
        kind: HashKind,
    },
    /// A `<string-token>` with its decoded value, without the quotes.
    String(&'i str),
    /// A `<bad-string-token>`.
    BadString,
    /// A `<url-token>` with its decoded value.
    Url(&'i str),
    /// A `<bad-url-token>`.
    BadUrl,
    /// A `<delim-token>` with its code point.
    Delim(char),
    /// A `<number-token>`.
    Number(Number<'i>),
    /// A `<percentage-token>`; its number excludes the percent sign.
    Percentage(Number<'i>),
    /// A `<dimension-token>` with its number and decoded unit.
    Dimension {
        /// The number.
        number: Number<'i>,
        /// The decoded unit.
        unit: &'i str,
    },
    /// A `<unicode-range-token>`, produced only when the lexer is told unicode
    /// ranges are allowed (section 4.3.1).
    UnicodeRange(CodePointRange),
    /// A `<whitespace-token>`.
    Whitespace,
    /// A comment, the one extension to the token list.
    Comment,
    /// A `<colon-token>`.
    Colon,
    /// A `<semicolon-token>`.
    Semicolon,
    /// A `<comma-token>`.
    Comma,
    /// A `<[-token>`, the opening of a simple block.
    OpenSquare,
    /// A `<(-token>`, the opening of a simple block.
    OpenParen,
    /// A `<{-token>`, the opening of a simple block.
    OpenCurly,
    /// A `<]-token>`.
    CloseSquare,
    /// A `<)-token>`.
    CloseParen,
    /// A `<}-token>`.
    CloseCurly,
    /// A `<CDO-token>`.
    Cdo,
    /// A `<CDC-token>`.
    Cdc,
}

impl Token<'_> {
    /// Returns the kind of the token.
    #[must_use]
    pub const fn kind(&self) -> TokenKind {
        match self {
            Self::Ident(_) => TokenKind::Ident,
            Self::Function(_) => TokenKind::Function,
            Self::AtKeyword(_) => TokenKind::AtKeyword,
            Self::Hash { .. } => TokenKind::Hash,
            Self::String(_) => TokenKind::String,
            Self::BadString => TokenKind::BadString,
            Self::Url(_) => TokenKind::Url,
            Self::BadUrl => TokenKind::BadUrl,
            Self::Delim(_) => TokenKind::Delim,
            Self::Number(_) => TokenKind::Number,
            Self::Percentage(_) => TokenKind::Percentage,
            Self::Dimension { .. } => TokenKind::Dimension,
            Self::UnicodeRange(_) => TokenKind::UnicodeRange,
            Self::Whitespace => TokenKind::Whitespace,
            Self::Comment => TokenKind::Comment,
            Self::Colon => TokenKind::Colon,
            Self::Semicolon => TokenKind::Semicolon,
            Self::Comma => TokenKind::Comma,
            Self::OpenSquare => TokenKind::OpenSquare,
            Self::OpenParen => TokenKind::OpenParen,
            Self::OpenCurly => TokenKind::OpenCurly,
            Self::CloseSquare => TokenKind::CloseSquare,
            Self::CloseParen => TokenKind::CloseParen,
            Self::CloseCurly => TokenKind::CloseCurly,
            Self::Cdo => TokenKind::Cdo,
            Self::Cdc => TokenKind::Cdc,
        }
    }

    /// Returns whether the token is whitespace or a comment, which a parser
    /// skips between the tokens that matter.
    #[must_use]
    pub const fn is_trivia(&self) -> bool {
        matches!(self, Self::Whitespace | Self::Comment)
    }
}
