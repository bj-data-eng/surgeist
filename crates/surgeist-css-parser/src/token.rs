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

    /// Returns the number that `representation` spells, with the type flag
    /// section 4 gives it, or `None` when `representation` is not the spelling
    /// of a number: an optional sign, digits with an optional fraction (or a
    /// fraction alone), and an optional exponent, and nothing else.
    ///
    /// A host that holds numbers it already checked, as the typed components of
    /// a value, makes the tokens of a [`TreeBuilder`](crate::document::TreeBuilder)
    /// this way.
    ///
    /// ```
    /// use surgeist_css_parser::token::{Number, NumberKind};
    ///
    /// let number = Number::parse("-1.5e2").unwrap();
    /// assert_eq!(number.representation(), "-1.5e2");
    /// assert_eq!(number.kind(), NumberKind::Number);
    /// assert_eq!(Number::parse("+7").unwrap().kind(), NumberKind::Integer);
    /// assert_eq!(Number::parse(".5").unwrap().kind(), NumberKind::Number);
    /// assert!(Number::parse("1.").is_none());
    /// assert!(Number::parse("1px").is_none());
    /// assert!(Number::parse("").is_none());
    /// ```
    #[must_use]
    pub fn parse(representation: &'i str) -> Option<Self> {
        let bytes = representation.as_bytes();
        let digits = |from: usize| {
            bytes[from..]
                .iter()
                .take_while(|byte| byte.is_ascii_digit())
                .count()
        };
        let mut at = usize::from(matches!(bytes.first(), Some(b'+' | b'-')));
        let whole = digits(at);
        at += whole;
        let mut kind = NumberKind::Integer;
        if bytes.get(at) == Some(&b'.') {
            let fraction = digits(at + 1);
            if fraction == 0 {
                return None;
            }
            at += 1 + fraction;
            kind = NumberKind::Number;
        } else if whole == 0 {
            return None;
        }
        if matches!(bytes.get(at), Some(b'e' | b'E')) {
            let signed = usize::from(matches!(bytes.get(at + 1), Some(b'+' | b'-')));
            let exponent = digits(at + 1 + signed);
            if exponent == 0 {
                return None;
            }
            at += 1 + signed + exponent;
            kind = NumberKind::Number;
        }
        (at == bytes.len()).then_some(Self::new(representation, kind))
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
