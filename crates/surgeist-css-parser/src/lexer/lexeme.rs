//! What the free-running lexer reports: lexemes, lexical errors and options.

use std::fmt;
use std::ops::Range;

use crate::{Offset, Token};

/// One token with the source interval it covers.
///
/// A lexeme borrows the lexer that produced it (a lending iterator): the
/// decoded text of an escaped name or string lives in the lexer's scratch space
/// until the next call to [`Lexer::next_lexeme`](super::Lexer::next_lexeme).
///
/// ```
/// use surgeist_css_parser::lexer::{Lexer, Options};
/// use surgeist_css_parser::{Offset, Token};
///
/// let mut lexer = Lexer::new("a /* x", &Options::default());
/// lexer.next_lexeme();
/// lexer.next_lexeme();
/// let comment = lexer.next_lexeme().unwrap();
/// assert_eq!(comment.token(), Token::Comment);
/// assert_eq!(comment.range(), Offset::new(2)..Offset::new(6));
/// // The comment ran to the end of the text instead of closing.
/// assert!(comment.is_unterminated());
/// ```
#[derive(Clone, Debug)]
pub struct Lexeme<'a> {
    pub(super) token: Token<'a>,
    pub(super) range: Range<Offset>,
    pub(super) unterminated: bool,
}

impl<'a> Lexeme<'a> {
    /// Returns the token.
    #[must_use]
    pub const fn token(&self) -> Token<'a> {
        self.token
    }

    /// Returns the interval of the lexed text that the token covers, as
    /// offsets of the text plus the [`origin`](Options::origin) of the options.
    ///
    /// The intervals of successive lexemes tile the text: the first starts at
    /// the origin and each starts where the one before it ended, comments
    /// included, so no byte belongs to two lexemes or to none.
    #[must_use]
    pub fn range(&self) -> Range<Offset> {
        self.range.clone()
    }

    /// Returns whether the token ended at the end of the text instead of at its
    /// own closing delimiter: a comment without `*/`, a string without its
    /// closing quote, and a url or bad url without its closing parenthesis.
    ///
    /// A bad string ends at a newline, which is not part of it, so it is not
    /// unterminated in this sense.
    #[must_use]
    pub const fn is_unterminated(&self) -> bool {
        self.unterminated
    }
}

/// How a [`Lexer`](super::Lexer) reads its text.
///
/// ```
/// use surgeist_css_parser::lexer::Options;
/// use surgeist_css_parser::Offset;
///
/// let options = Options::default().unicode_ranges(true).origin(Offset::new(40));
/// assert!(options.allows_unicode_ranges());
/// assert_eq!(options.origin_offset(), Offset::new(40));
/// assert!(!Options::default().allows_unicode_ranges());
/// ```
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct Options {
    unicode_ranges: bool,
    origin: usize,
}

impl Options {
    /// Sets whether the lexer produces [`Token::UnicodeRange`] where section
    /// 4.3.11 says the input would start a unicode-range. The default is
    /// `false`: only the `unicode-range` descriptor (section 5.5.6, step 8)
    /// allows them.
    #[must_use]
    pub const fn unicode_ranges(mut self, allowed: bool) -> Self {
        self.unicode_ranges = allowed;
        self
    }

    /// Sets the offset at which the lexed text begins in a larger text. Every
    /// reported offset is that far from the start of the lexed text, so a
    /// sub-range re-lexed on its own still reports positions in the text it came
    /// from. The default is zero.
    #[must_use]
    pub const fn origin(mut self, origin: Offset) -> Self {
        self.origin = origin.value();
        self
    }

    /// Returns whether unicode ranges are allowed.
    #[must_use]
    pub const fn allows_unicode_ranges(&self) -> bool {
        self.unicode_ranges
    }

    /// Returns the offset at which the lexed text begins.
    #[must_use]
    pub const fn origin_offset(&self) -> Offset {
        Offset::new(self.origin)
    }
}

/// A place where the text breaks a rule of section 4.3 that the specification
/// calls a parse error and that no token records.
///
/// Tokenizing never fails, so these are events beside the tokens, in the order
/// the lexer met them; a consumer that does not care ignores them. Each
/// corresponds to one "this is a parse error" sentence of the pinned edition,
/// and to nothing else: a bad url that arises because whitespace was followed
/// by something other than `)` is a `<bad-url-token>` but, as section 4.3.6
/// words it, not a parse error, so it has no event.
///
/// ```
/// use surgeist_css_parser::lexer::{Lexer, Options};
///
/// let mut lexer = Lexer::new("'a\nb", &Options::default());
/// while lexer.next_lexeme().is_some() {}
/// let errors = lexer.errors();
/// assert_eq!(errors.len(), 1);
/// // A newline ended the string: section 4.3.5's bad-string parse error.
/// assert!(errors[0].is_bad_string());
/// ```
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct LexicalError {
    kind: LexicalErrorKind,
    range: Range<Offset>,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(super) enum LexicalErrorKind {
    UnterminatedComment,
    UnterminatedString,
    BadString,
    UnterminatedUrl,
    BadUrl,
    EscapedNewline,
    EscapedEof,
}

impl LexicalError {
    pub(super) const fn new(kind: LexicalErrorKind, range: Range<Offset>) -> Self {
        Self { kind, range }
    }

    /// Returns the interval the error is about: the whole comment, string or
    /// url token for the first five kinds, the backslash itself for the last
    /// two.
    #[must_use]
    pub fn range(&self) -> Range<Offset> {
        self.range.clone()
    }

    kind_predicates! { kind;
        /// Whether a comment reached the end of the text without `*/`
        /// (section 4.3.2).
        is_unterminated_comment => LexicalErrorKind::UnterminatedComment,
        /// Whether a string reached the end of the text without its closing
        /// quote (section 4.3.5).
        is_unterminated_string => LexicalErrorKind::UnterminatedString,
        /// Whether a newline ended a string, making a `<bad-string-token>`
        /// (section 4.3.5).
        is_bad_string => LexicalErrorKind::BadString,
        /// Whether a url reached the end of the text without its closing
        /// parenthesis (section 4.3.6).
        is_unterminated_url => LexicalErrorKind::UnterminatedUrl,
        /// Whether a quote, an opening parenthesis, a non-printable code point
        /// or a backslash that starts no valid escape made a url a
        /// `<bad-url-token>` (section 4.3.6).
        is_bad_url => LexicalErrorKind::BadUrl,
        /// Whether a backslash outside a string was followed by a newline, so
        /// it starts no valid escape and is a `<delim-token>` (section 4.3.1).
        is_escaped_newline => LexicalErrorKind::EscapedNewline,
        /// Whether a backslash was the last code point of the text, so the
        /// escape it starts has no code point and stands for U+FFFD (section
        /// 4.3.7).
        is_escaped_eof => LexicalErrorKind::EscapedEof,
    }
}

impl fmt::Display for LexicalError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        let what = match self.kind {
            LexicalErrorKind::UnterminatedComment => "a comment is missing its closing \"*/\"",
            LexicalErrorKind::UnterminatedString => "a string is missing its closing quote",
            LexicalErrorKind::BadString => "a newline ended a string",
            LexicalErrorKind::UnterminatedUrl => "a url is missing its closing parenthesis",
            LexicalErrorKind::BadUrl => "a url holds a code point it cannot",
            LexicalErrorKind::EscapedNewline => "a backslash before a newline starts no escape",
            LexicalErrorKind::EscapedEof => "a backslash ends the text, so its escape is empty",
        };
        write!(
            formatter,
            "{what} (bytes {} to {})",
            self.range.start.value(),
            self.range.end.value()
        )
    }
}

impl std::error::Error for LexicalError {}
