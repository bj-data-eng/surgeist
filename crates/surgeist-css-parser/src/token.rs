//! The parts of a [`Token`](crate::Token): its kind, the type flag of a hash
//! token, and the number a numeric token carries.

/// The kinds of token that CSS Syntax Module Level 3 section 4 produces, plus
/// the comment extension of section 9.
///
/// The set is closed: it mirrors the normative token list, and a new edition
/// would be a deliberate breaking change. [`Token::kind`](crate::Token::kind)
/// returns it.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub enum TokenKind {
    /// An `<ident-token>`.
    Ident,
    /// A `<function-token>`.
    Function,
    /// An `<at-keyword-token>`.
    AtKeyword,
    /// A `<hash-token>`.
    Hash,
    /// A `<string-token>`.
    String,
    /// A `<bad-string-token>`.
    BadString,
    /// A `<url-token>`.
    Url,
    /// A `<bad-url-token>`.
    BadUrl,
    /// A `<delim-token>`.
    Delim,
    /// A `<number-token>`.
    Number,
    /// A `<percentage-token>`.
    Percentage,
    /// A `<dimension-token>`.
    Dimension,
    /// A `<unicode-range-token>`.
    UnicodeRange,
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
    /// A `<[-token>`.
    OpenSquare,
    /// A `<(-token>`.
    OpenParen,
    /// A `<{-token>`.
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

/// The type flag of a `<hash-token>` (section 4).
///
/// ```
/// use surgeist_css_parser::lexer::{Lexer, Options};
/// use surgeist_css_parser::token::HashKind;
/// use surgeist_css_parser::Token;
///
/// let mut lexer = Lexer::new("#a #1", &Options::default());
/// let Some(first) = lexer.next_lexeme() else { return };
/// // `a` would start an ident sequence, so the flag is "id".
/// assert_eq!(first.token(), Token::Hash { value: "a", kind: HashKind::Id });
/// ```
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub enum HashKind {
    /// The flag is "id": the value would start an ident sequence, so the hash
    /// is a valid ID selector.
    Id,
    /// The flag is "unrestricted", its default.
    Unrestricted,
}

/// The sign character of a numeric token (section 4).
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub enum Sign {
    /// U+002B PLUS SIGN.
    Plus,
    /// U+002D HYPHEN-MINUS.
    Minus,
}

/// The type flag of a `<number-token>` or `<dimension-token>` (section 4).
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub enum NumberKind {
    /// The flag is "integer", its default: the spelling has neither a fraction
    /// nor an exponent.
    Integer,
    /// The flag is "number": the spelling has a fraction or an exponent.
    Number,
}

/// The number of a numeric token: its exact spelling, sign and type flag.
///
/// The spelling is never converted when the token is made, so a consumer that
/// needs the digits exactly (a decimal, a custom property's original text) has
/// them, and one that needs a value asks for it with [`to_f64`](Self::to_f64)
/// or [`integer`](Self::integer).
///
/// ```
/// use surgeist_css_parser::lexer::{Lexer, Options};
/// use surgeist_css_parser::token::{NumberKind, Sign};
/// use surgeist_css_parser::Token;
///
/// let mut lexer = Lexer::new("-1.5e2 +7", &Options::default());
/// let Some(Token::Number(number)) = lexer.next_lexeme().map(|lexeme| lexeme.token()) else {
///     return;
/// };
/// assert_eq!(number.representation(), "-1.5e2");
/// assert_eq!(number.sign(), Some(Sign::Minus));
/// assert_eq!(number.kind(), NumberKind::Number);
/// assert_eq!(number.to_f64(), -150.0);
/// assert_eq!(number.integer(), None);
/// ```
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct Number<'i> {
    representation: &'i str,
    kind: NumberKind,
}

impl<'i> Number<'i> {
    pub(crate) const fn new(representation: &'i str, kind: NumberKind) -> Self {
        Self {
            representation,
            kind,
        }
    }

    /// Returns the exact source spelling, sign included and any unit or
    /// percent sign excluded.
    #[must_use]
    pub const fn representation(self) -> &'i str {
        self.representation
    }

    /// Returns the sign character the spelling starts with, if any.
    #[must_use]
    pub fn sign(self) -> Option<Sign> {
        match self.representation.as_bytes().first() {
            Some(b'+') => Some(Sign::Plus),
            Some(b'-') => Some(Sign::Minus),
            _ => None,
        }
    }

    /// Returns the type flag.
    #[must_use]
    pub const fn kind(self) -> NumberKind {
        self.kind
    }

    /// Returns the value rounded to the nearest `f64`.
    ///
    /// A spelling too large for `f64` gives infinity. A consumer that needs the
    /// digits exactly uses [`representation`](Self::representation).
    #[must_use]
    pub fn to_f64(self) -> f64 {
        self.representation.parse().unwrap_or(f64::NAN)
    }

    /// Returns the value of an integer spelling that fits an `i64`, and `None`
    /// for a number-flagged spelling or one too large.
    #[must_use]
    pub fn integer(self) -> Option<i64> {
        match self.kind {
            NumberKind::Integer => self.representation.parse().ok(),
            NumberKind::Number => None,
        }
    }
}
