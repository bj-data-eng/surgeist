//! Serialization of tokens and component values (CSS Syntax Module Level 3
//! section 9).
//!
//! Section 9 has one requirement: serializing what was parsed and parsing the
//! result must give the same data structures again, except that consecutive
//! whitespace may collapse into one token. This module writes tokens so that
//! they meet it. That takes three things, and the module owns all of them:
//!
//! - the spelling of each token, in [`Token`'s `Display`](Token) and in the
//!   functions [`identifier`], [`string`] and [`url`], which escape what the
//!   tokenizer would otherwise read differently;
//! - the comment that must separate two tokens that would otherwise run
//!   together when read again, section 9's table, as [`needs_separator`]
//!   over the [`Adjacency`] of a token; and
//! - the [`Writer`] and [`list`] that apply both to a run of tokens or to the
//!   component values of a [`Document`].
//!
//! # Escaping
//!
//! The escaping algorithms are those of CSSOM 1 section 2.1 (serialize an
//! identifier, a string), because section 9 only says that the output must
//! round-trip. Where the two differ the round trip governs. An identifier code
//! point is written as itself, and every other code point is escaped, by the
//! predicate of the tokenizer ([`lexical::is_ident_code_point`]), so the
//! output re-tokenizes to the identifier it came from at every edition of that
//! predicate. Today that predicate admits every non-ASCII code point (the 2021
//! rule, see [`lexical`]); when it adopts the ranges of the pinned edition,
//! the non-ASCII code points outside them are written as escapes.
//!
//! # What is serialized
//!
//! A token is written from its value, never from the text it was lexed from:
//! the output is a normal form (the quotes of a string are `"`, an escape is a
//! hexadecimal one only when needed), except that a number keeps its
//! spelling. A [`Token::Whitespace`] is one space; a [`Token::Comment`], which
//! keeps no contents, is `/**/`. A bad string and a bad url keep no value, so
//! each is written as text that the tokenizer reads as one again.

use std::fmt::{self, Write};

use crate::document::{BlockKind, Component, ListId};
use crate::lexical;
use crate::token::HashKind;
use crate::{Document, Token};

// ------------------------------------------------------------------ escaping

fn hex_escape<W: Write + ?Sized>(point: char, out: &mut W) -> fmt::Result {
    write!(out, "\\{:x} ", u32::from(point))
}

/// Writes one code point of a name (the part of an identifier after its first
/// code points, the value of an unrestricted hash): an ident code point as
/// itself, NUL as U+FFFD, a control code point or a non-ASCII code point that
/// is not an ident code point as a hexadecimal escape, and anything else ASCII
/// behind a backslash.
fn name_point<W: Write + ?Sized>(point: char, out: &mut W) -> fmt::Result {
    if point == '\0' {
        out.write_char('\u{fffd}')
    } else if ('\u{1}'..='\u{1f}').contains(&point) || point == '\u{7f}' {
        hex_escape(point, out)
    } else if lexical::is_ident_code_point(point) {
        out.write_char(point)
    } else if point.is_ascii() {
        out.write_char('\\')?;
        out.write_char(point)
    } else {
        hex_escape(point, out)
    }
}

/// Writes the value of an unrestricted hash: every code point as in a name,
/// including a leading digit.
fn name<W: Write + ?Sized>(value: &str, out: &mut W) -> fmt::Result {
    value.chars().try_for_each(|point| name_point(point, out))
}

/// Writes `value` as an identifier (CSSOM 1 section 2.1, "serialize an
/// identifier", under the round-trip rule of Syntax 3 section 9).
///
/// Each code point is written as itself when it is an ident code point and
/// escaped otherwise. NUL is written as U+FFFD. A control code point (U+0001 to
/// U+001F, U+007F) is a hexadecimal escape with its terminating space, and so
/// is a digit at the start or directly after a leading hyphen-minus, where a
/// digit would not continue an identifier. A lone hyphen-minus is `\-`. An
/// ASCII code point that is not an ident code point is written behind a
/// backslash; a non-ASCII one, behind a hexadecimal escape.
///
/// # Errors
///
/// Fails only when `out` does.
///
/// ```
/// use surgeist_css_parser::serialize::identifier;
///
/// fn escaped(value: &str) -> String {
///     let mut out = String::new();
///     identifier(value, &mut out).unwrap();
///     out
/// }
/// assert_eq!(escaped("a-b_c"), "a-b_c");
/// assert_eq!(escaped("1a"), "\\31 a");
/// assert_eq!(escaped("-1"), "-\\31 ");
/// assert_eq!(escaped("-"), "\\-");
/// assert_eq!(escaped("a b"), "a\\ b");
/// assert_eq!(escaped("a\u{1}b"), "a\\1 b");
/// ```
pub fn identifier<W: Write + ?Sized>(value: &str, out: &mut W) -> fmt::Result {
    let lone_hyphen = value == "-";
    let leading_hyphen = value.starts_with('-');
    for (index, point) in value.chars().enumerate() {
        let digit_here_is_invalid =
            point.is_ascii_digit() && (index == 0 || (index == 1 && leading_hyphen));
        if digit_here_is_invalid {
            hex_escape(point, out)?;
        } else if index == 0 && lone_hyphen {
            out.write_str("\\-")?;
        } else {
            name_point(point, out)?;
        }
    }
    Ok(())
}

/// Writes `value` as a quoted string (CSSOM 1 section 2.1, "serialize a
/// string"): between U+0022 QUOTATION MARKs, with NUL as U+FFFD, a control
/// code point as a hexadecimal escape, and a quotation mark or a backslash
/// behind a backslash.
///
/// # Errors
///
/// Fails only when `out` does.
///
/// ```
/// use surgeist_css_parser::serialize::string;
///
/// let mut out = String::new();
/// string("a\"b\\c\nd", &mut out).unwrap();
/// assert_eq!(out, "\"a\\\"b\\\\c\\a d\"");
/// ```
pub fn string<W: Write + ?Sized>(value: &str, out: &mut W) -> fmt::Result {
    out.write_char('"')?;
    for point in value.chars() {
        match point {
            '\0' => out.write_char('\u{fffd}')?,
            '"' | '\\' => {
                out.write_char('\\')?;
                out.write_char(point)?;
            }
            point if ('\u{1}'..='\u{1f}').contains(&point) || point == '\u{7f}' => {
                hex_escape(point, out)?;
            }
            point => out.write_char(point)?,
        }
    }
    out.write_char('"')
}

/// Writes `value` as an unquoted url token: `url(`, the value, and `)`.
///
/// Syntax 3 section 9 requires the output to read as the `<url-token>` it came
/// from, so this is the unquoted form, with the code points that end or break
/// one escaped: whitespace, control code points and NUL as a hexadecimal escape
/// (NUL as U+FFFD), and a quotation mark, an apostrophe, a parenthesis and a
/// backslash behind a backslash. The quoted `url("...")` that CSSOM writes is a
/// function token and a string token, not a url token; write that with
/// [`Token::Function`], [`string`] and a closing parenthesis.
///
/// # Errors
///
/// Fails only when `out` does.
///
/// ```
/// use surgeist_css_parser::serialize::url;
///
/// let mut out = String::new();
/// url("a b(c)", &mut out).unwrap();
/// assert_eq!(out, "url(a\\20 b\\(c\\))");
/// ```
pub fn url<W: Write + ?Sized>(value: &str, out: &mut W) -> fmt::Result {
    out.write_str("url(")?;
    for point in value.chars() {
        match point {
            '\0' => out.write_char('\u{fffd}')?,
            '"' | '\'' | '(' | ')' | '\\' => {
                out.write_char('\\')?;
                out.write_char(point)?;
            }
            point
                if lexical::is_whitespace(point)
                    || lexical::is_non_printable(point)
                    || ('\u{1}'..='\u{1f}').contains(&point)
                    || point == '\u{7f}' =>
            {
                hex_escape(point, out)?;
            }
            point => out.write_char(point)?,
        }
    }
    out.write_char(')')
}

/// Writes the unit of a dimension: an identifier, with the first code point
/// escaped when the unit would read as the exponent of the number before it
/// (`e` or `E` followed by a digit, or by a hyphen-minus and a digit; section
/// 4.3.13 step 6 reads those as an exponent).
fn unit<W: Write + ?Sized>(value: &str, out: &mut W) -> fmt::Result {
    let mut points = value.chars();
    if let (Some(first @ ('e' | 'E')), Some(second)) = (points.next(), points.next()) {
        let exponent = second.is_ascii_digit()
            || (second == '-' && points.next().is_some_and(|third| third.is_ascii_digit()));
        if exponent {
            hex_escape(first, out)?;
            return name(&value[1..], out);
        }
    }
    identifier(value, out)
}

// ------------------------------------------------------------------ a token

/// Writes the spelling of one token: no separator, no context.
pub(crate) fn write_token<W: Write + ?Sized>(token: &Token<'_>, out: &mut W) -> fmt::Result {
    match token {
        Token::Ident(value) => identifier(value, out),
        Token::Function(value) => {
            identifier(value, out)?;
            out.write_char('(')
        }
        Token::AtKeyword(value) => {
            out.write_char('@')?;
            identifier(value, out)
        }
        Token::Hash { value, kind } => {
            out.write_char('#')?;
            match kind {
                HashKind::Id => identifier(value, out),
                HashKind::Unrestricted => name(value, out),
            }
        }
        Token::String(value) => string(value, out),
        // The tokenizer reads `"` and a newline as a bad string and then a
        // whitespace token starting with that newline.
        Token::BadString => out.write_str("\"\n"),
        Token::Url(value) => url(value, out),
        // Whitespace and then anything but `)` makes a bad url (4.3.6).
        Token::BadUrl => out.write_str("url(a b)"),
        // 4.3.1: a backslash that starts no valid escape is a delim, and the
        // tokenizer only emits one before a newline (4.3.8).
        Token::Delim('\\') => out.write_str("\\\n"),
        Token::Delim(point) => out.write_char(*point),
        Token::Number(number) => out.write_str(number.representation()),
        Token::Percentage(number) => {
            out.write_str(number.representation())?;
            out.write_char('%')
        }
        Token::Dimension {
            number,
            unit: value,
        } => {
            out.write_str(number.representation())?;
            unit(value, out)
        }
        Token::UnicodeRange(range) => {
            if range.start() == range.end() {
                write!(out, "U+{:X}", range.start())
            } else {
                write!(out, "U+{:X}-{:X}", range.start(), range.end())
            }
        }
        Token::Whitespace => out.write_char(' '),
        Token::Comment => out.write_str("/**/"),
        Token::Colon => out.write_char(':'),
        Token::Semicolon => out.write_char(';'),
        Token::Comma => out.write_char(','),
        Token::OpenSquare => out.write_char('['),
        Token::OpenParen => out.write_char('('),
        Token::OpenCurly => out.write_char('{'),
        Token::CloseSquare => out.write_char(']'),
        Token::CloseParen => out.write_char(')'),
        Token::CloseCurly => out.write_char('}'),
        Token::Cdo => out.write_str("<!--"),
        Token::Cdc => out.write_str("-->"),
    }
}

impl fmt::Display for Token<'_> {
    /// Serializes the token on its own, as section 9 asks.
    ///
    /// The output is a normal form written from the token's value; see the
    /// [module documentation](crate::serialize). A `<delim-token>` of U+005C
    /// REVERSE SOLIDUS is a backslash and a newline, as the note of section 9
    /// requires. Two tokens written one after the other may need a comment
    /// between them; [`Writer`] adds it.
    ///
    /// ```
    /// use surgeist_css_parser::lexer::{Lexer, Options};
    ///
    /// // An escape that was not needed is not written.
    /// let mut lexer = Lexer::new("1\\70 x", &Options::default());
    /// let token = lexer.next_lexeme().unwrap().token();
    /// assert_eq!(token.to_string(), "1px");
    /// // The unit `e3` would read as the exponent of `1`, so its `e` stays escaped.
    /// let mut lexer = Lexer::new("1\\65 3", &Options::default());
    /// let token = lexer.next_lexeme().unwrap().token();
    /// assert_eq!(token.to_string(), "1\\65 3");
    /// ```
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write_token(self, formatter)
    }
}

// ----------------------------------------------------------------- adjacency

/// The part of a token that decides whether a comment must separate it from
/// its neighbour (the headings of the table in section 9).
///
/// Only the kinds that appear as a row or a column heading of the table are
/// distinguished, plus [`UnicodeRange`](Self::UnicodeRange), which the table
/// does not have; every other token is [`Other`](Self::Other).
///
/// ```
/// use surgeist_css_parser::serialize::Adjacency;
/// use surgeist_css_parser::Token;
///
/// assert_eq!(Adjacency::from(&Token::Ident("a")), Adjacency::Ident);
/// assert_eq!(Adjacency::from(&Token::Delim('-')), Adjacency::Delim('-'));
/// assert_eq!(Adjacency::from(&Token::Comma), Adjacency::Other);
/// ```
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
#[non_exhaustive]
pub enum Adjacency {
    /// An `<ident-token>`.
    Ident,
    /// An `<at-keyword-token>`.
    AtKeyword,
    /// A `<hash-token>`.
    Hash,
    /// A `<dimension-token>`.
    Dimension,
    /// A `<number-token>`.
    Number,
    /// A `<delim-token>` with this code point.
    Delim(char),
    /// A `<function-token>`.
    Function,
    /// A `<url-token>`.
    Url,
    /// A `<bad-url-token>`.
    BadUrl,
    /// A `<percentage-token>`.
    Percentage,
    /// A `<CDC-token>`.
    Cdc,
    /// A `<(-token>`.
    OpenParen,
    /// A `<unicode-range-token>`. The table does not have it; it is read like
    /// an ident on both sides, because it starts with `U` and ends in hex digits
    /// or question marks that a following ident, number or `?` would extend.
    UnicodeRange,
    /// Any other token: it needs no comment on either side.
    Other,
}

impl From<&Token<'_>> for Adjacency {
    fn from(token: &Token<'_>) -> Self {
        match token {
            Token::Ident(_) => Self::Ident,
            Token::AtKeyword(_) => Self::AtKeyword,
            Token::Hash { .. } => Self::Hash,
            Token::Dimension { .. } => Self::Dimension,
            Token::Number(_) => Self::Number,
            Token::Delim(point) => Self::Delim(*point),
            Token::Function(_) => Self::Function,
            Token::Url(_) => Self::Url,
            Token::BadUrl => Self::BadUrl,
            Token::Percentage(_) => Self::Percentage,
            Token::Cdc => Self::Cdc,
            Token::OpenParen => Self::OpenParen,
            Token::UnicodeRange(_) => Self::UnicodeRange,
            Token::String(_)
            | Token::BadString
            | Token::Whitespace
            | Token::Comment
            | Token::Colon
            | Token::Semicolon
            | Token::Comma
            | Token::OpenSquare
            | Token::OpenCurly
            | Token::CloseSquare
            | Token::CloseParen
            | Token::CloseCurly
            | Token::Cdo => Self::Other,
        }
    }
}

/// Whether a comment must separate `previous` from `next` when they are
/// written one right after the other, for the output to read as the same two
/// tokens again: the table of section 9.
///
/// The rows of the table are the first token, the columns the second. An ident
/// runs into every name-like token that follows it, a number into the tokens
/// that would make it a dimension or a longer number, an at-keyword, hash,
/// dimension, `#` and `-` into the name-like tokens, `@` into the ones that
/// would make it an at-keyword, `.` and `+` into a number, and `/` into `*`.
/// The unicode-range token is read as an ident (see
/// [`Adjacency::UnicodeRange`]), and is also followed by a `?`, which
/// would make it a longer range.
///
/// ```
/// use surgeist_css_parser::serialize::{Adjacency, needs_separator};
///
/// assert!(needs_separator(Adjacency::Ident, Adjacency::Ident));
/// assert!(needs_separator(Adjacency::Number, Adjacency::Delim('%')));
/// assert!(needs_separator(Adjacency::Delim('/'), Adjacency::Delim('*')));
/// assert!(!needs_separator(Adjacency::Ident, Adjacency::Delim('*')));
/// assert!(!needs_separator(Adjacency::Number, Adjacency::Delim('-')));
/// ```
#[must_use]
pub const fn needs_separator(previous: Adjacency, next: Adjacency) -> bool {
    use Adjacency::{
        AtKeyword, BadUrl, Cdc, Delim, Dimension, Function, Hash, Ident, Number, OpenParen,
        Percentage, UnicodeRange, Url,
    };
    // The columns that run into a name: ident, function, url, bad url, `-`,
    // number, percentage, dimension and CDC; and the unicode-range token.
    let name_like = matches!(
        next,
        Ident
            | Function
            | Url
            | BadUrl
            | Delim('-')
            | Number
            | Percentage
            | Dimension
            | Cdc
            | UnicodeRange
    );
    match previous {
        Ident => name_like || matches!(next, OpenParen),
        AtKeyword | Hash | Dimension | Delim('#' | '-') => name_like,
        Number => {
            matches!(
                next,
                Ident
                    | Function
                    | Url
                    | BadUrl
                    | Number
                    | Percentage
                    | Dimension
                    | Cdc
                    | UnicodeRange
                    | Delim('%')
            )
        }
        Delim('@') => matches!(
            next,
            Ident | Function | Url | BadUrl | Delim('-') | Cdc | UnicodeRange
        ),
        Delim('.' | '+') => matches!(next, Number | Percentage | Dimension),
        Delim('/') => matches!(next, Delim('*')),
        UnicodeRange => {
            matches!(
                next,
                Ident | Function | Url | BadUrl | Number | Percentage | Dimension | Delim('?')
            )
        }
        _ => false,
    }
}

// -------------------------------------------------------------------- writer

/// Writes a run of tokens so that they read as the same tokens again.
///
/// Each [`token`](Self::token) is written by [`Token`'s `Display`](Token), with
/// the comment of [`needs_separator`] before it when the token before it
/// requires one. A [`Token::Whitespace`] is one space however many are written
/// in a row, and a comment resets what the writer knows of the token before
/// it, because a comment separates its neighbours.
///
/// Two more runs of tokens need a comment than the table says, and the writer
/// separates them, because the round trip is the requirement and the table is
/// a note on how to meet it:
///
/// - a `<` delim and a `!` delim followed by something that starts with two
///   hyphen-minuses (an ident or a function whose name does, or a CDC) would read as a
///   `<CDO-token>`, which section 4.3.1 reads from `<!--`;
/// - the ident `--` followed by a `>` delim would read as a `<CDC-token>`
///   (`-->`), which section 4.3.1 reads before an ident. An ident that is
///   longer, `--x` for one, ends before the `>` and needs nothing.
///
/// ```
/// use surgeist_css_parser::lexer::Options;
/// use surgeist_css_parser::limits::{DepthExcess, Limits};
/// use surgeist_css_parser::serialize::Writer;
/// use surgeist_css_parser::{Document, Token};
///
/// // `1`, a comment and `px` are two tokens; written without the comment they
/// // would be the one dimension `1px`.
/// let document: Document =
///     Document::parse("1/**/px", &Options::default(), Limits::default(), DepthExcess::Fail)?;
/// let mut writer = Writer::new(String::new());
/// let mut stream = document.root();
/// while let Ok(token) = stream.consume_token() {
///     writer.token(token).unwrap();
/// }
/// assert_eq!(writer.into_inner(), "1/**/px");
/// # Ok::<(), surgeist_css_parser::document::BuildError>(())
/// ```
#[derive(Debug)]
pub struct Writer<W> {
    out: W,
    previous: Option<Adjacency>,
    before_previous: Option<Adjacency>,
    /// Whether the token written last is the ident `--`.
    double_hyphen: bool,
    whitespace: bool,
}

impl<W: Write> Writer<W> {
    /// Creates a writer that writes to `out`.
    #[must_use]
    pub const fn new(out: W) -> Self {
        Self {
            out,
            previous: None,
            before_previous: None,
            double_hyphen: false,
            whitespace: false,
        }
    }

    /// Returns the output.
    #[must_use]
    pub fn into_inner(self) -> W {
        self.out
    }

    fn separate(&mut self) {
        self.previous = None;
        self.before_previous = None;
        self.double_hyphen = false;
    }

    /// Writes `token`, with a comment first when the token written before it
    /// requires one.
    ///
    /// # Errors
    ///
    /// Fails only when the output does.
    pub fn token(&mut self, token: Token<'_>) -> fmt::Result {
        match token {
            Token::Whitespace => {
                if !self.whitespace {
                    self.out.write_char(' ')?;
                }
                self.whitespace = true;
                self.separate();
                return Ok(());
            }
            Token::Comment => {
                write_token(&token, &mut self.out)?;
                self.whitespace = false;
                self.separate();
                return Ok(());
            }
            _ => {}
        }
        let next = Adjacency::from(&token);
        let starts_cdc_or_hyphens = match token {
            Token::Ident(value) | Token::Function(value) => value.starts_with("--"),
            Token::Cdc => true,
            _ => false,
        };
        let cdo = starts_cdc_or_hyphens
            && self.before_previous == Some(Adjacency::Delim('<'))
            && self.previous == Some(Adjacency::Delim('!'));
        let cdc = self.double_hyphen && token == Token::Delim('>');
        let separated = cdo
            || cdc
            || self
                .previous
                .is_some_and(|previous| needs_separator(previous, next));
        if separated {
            self.out.write_str("/**/")?;
            self.before_previous = None;
        } else {
            self.before_previous = self.previous;
        }
        write_token(&token, &mut self.out)?;
        self.previous = Some(next);
        self.double_hyphen = token == Token::Ident("--");
        self.whitespace = false;
        Ok(())
    }

    /// Writes `text` as it is, as the next token: it counts as a token that
    /// needs no comment on either side.
    fn verbatim(&mut self, text: &str) -> fmt::Result {
        self.out.write_str(text)?;
        self.previous = Some(Adjacency::Other);
        self.before_previous = None;
        self.double_hyphen = false;
        self.whitespace = false;
        Ok(())
    }
}

// ---------------------------------------------------------------------- list

const fn block_tokens(kind: BlockKind) -> (Token<'static>, Token<'static>) {
    match kind {
        BlockKind::Paren => (Token::OpenParen, Token::CloseParen),
        BlockKind::Square => (Token::OpenSquare, Token::CloseSquare),
        BlockKind::Curly => (Token::OpenCurly, Token::CloseCurly),
    }
}

/// Serializes the component values of `list` in `document` (section 9): every
/// token in order, each function and simple block with its contents and its
/// closing token, and the comment of [`needs_separator`] wherever two tokens
/// need one.
///
/// A group the input ended instead of closing is written with its closing
/// token, because the specification's data structure is the same for both.
/// A comment the document kept is written as `/**/`, and one the document did
/// not keep is written only where the table requires it, so the output reads as
/// the same component values once comments, which section 9 says have no effect
/// on parsing, are left out.
///
/// A `{}`-block that the build denied for nesting too deep
/// ([`DepthExcess::DenyBlock`](crate::limits::DepthExcess::DenyBlock)) kept no
/// contents; it is written as the source text it covered, which only a
/// document built from text can have denied.
///
/// The walk keeps its own stack, so a deep tree cannot overflow the thread's.
///
/// # Errors
///
/// Fails only when `out` does.
///
/// # Panics
///
/// Panics if the document holds a denied block that has no source text, which no
/// document built by this crate does.
///
/// ```
/// use surgeist_css_parser::lexer::Options;
/// use surgeist_css_parser::limits::{DepthExcess, Limits};
/// use surgeist_css_parser::{Document, serialize};
///
/// let document: Document =
///     Document::parse("a { b : f( 1px ) } ", &Options::default(), Limits::default(), DepthExcess::Fail)?;
/// let mut out = String::new();
/// serialize::list(&document, document.root_list(), &mut out).unwrap();
/// assert_eq!(out, "a { b : f( 1px ) } ");
///
/// // A block denied for nesting too deep is written as the text it was.
/// let limits = Limits::new(1, usize::MAX, usize::MAX);
/// let document: Document =
///     Document::parse("{{a}}", &Options::default(), limits, DepthExcess::DenyBlock)?;
/// let mut out = String::new();
/// serialize::list(&document, document.root_list(), &mut out).unwrap();
/// assert_eq!(out, "{{a}}");
/// # Ok::<(), surgeist_css_parser::document::BuildError>(())
/// ```
pub fn list<P, W: Write>(document: &Document<'_, P>, list: ListId, out: W) -> fmt::Result {
    struct Frame {
        list: ListId,
        next: usize,
        closer: Option<Token<'static>>,
    }
    let mut writer = Writer::new(out);
    let mut frames = vec![Frame {
        list,
        next: 0,
        closer: None,
    }];
    while let Some(frame) = frames.last_mut() {
        let Some(&node) = document.list(frame.list).get(frame.next) else {
            let closer = frame.closer;
            frames.pop();
            if let Some(closer) = closer {
                writer.token(closer)?;
            }
            continue;
        };
        frame.next += 1;
        match document.component(node) {
            Component::Token { token, .. } => writer.token(token)?,
            Component::Function { name, children, .. } => {
                writer.token(Token::Function(name))?;
                frames.push(Frame {
                    list: children,
                    next: 0,
                    closer: Some(Token::CloseParen),
                });
            }
            Component::Block { kind, children, .. } => {
                let (open, close) = block_tokens(kind);
                writer.token(open)?;
                frames.push(Frame {
                    list: children,
                    next: 0,
                    closer: Some(close),
                });
            }
            Component::Denied { range, .. } => {
                let text = range
                    .and_then(|range| document.slice(range.start, range.end))
                    .expect("a denied block was lexed from source text, so its text is there");
                writer.verbatim(text)?;
            }
        }
    }
    Ok(())
}
