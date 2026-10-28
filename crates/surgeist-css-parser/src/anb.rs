//! The `An+B` microsyntax of CSS Syntax Module Level 3 section 6.
//!
//! [`parse`] recognizes the `<a-n-plus-b>` type on a [`Stream`] by the rules of
//! section 6.2 and interprets what it recognized as the two integers
//! [`AnPlusB::a`] and [`AnPlusB::b`]; the type's [`Display`](std::fmt::Display)
//! is the serialization of section 9.1. The module gives the notation no
//! meaning of its own: which elements `An+B` selects is the business of the
//! grammar that uses it (Selectors).
//!
//! The specification defines `A` and `B` as integers without bound, and this
//! crate stores them as `i32`. A value outside `i32` is not saturated, because a
//! saturated value would be a different number: [`parse`] reports it as a typed
//! overflow ([`AnPlusBError::is_overflow`]) and the grammar decides what its
//! own rules say about such a number.

use std::fmt;

use crate::stream::StreamError;
use crate::token::{Number, NumberKind};
use crate::{Site, Stream, Token};

/// The step `A` and the offset `B` of an `An+B` notation (section 6).
///
/// The notation represents the `An+B`th elements of a list for every
/// non-negative integer `n`, counting from 1; the type only holds the two
/// integers. Its [`Display`](fmt::Display) is the serialization of section 9.1,
/// which writes the shortest equivalent form.
///
/// ```
/// use surgeist_css_parser::anb::AnPlusB;
///
/// let value = AnPlusB::new(2, 1);
/// assert_eq!((value.a(), value.b()), (2, 1));
/// assert_eq!(value.to_string(), "2n+1");
/// assert_eq!(AnPlusB::new(0, 5).to_string(), "5");
/// assert_eq!(AnPlusB::new(-1, 6).to_string(), "-n+6");
/// ```
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct AnPlusB {
    a: i32,
    b: i32,
}

impl AnPlusB {
    /// Creates the notation with step `a` and offset `b`.
    #[must_use]
    pub const fn new(a: i32, b: i32) -> Self {
        Self { a, b }
    }

    /// Returns `A`, the step.
    #[must_use]
    pub const fn a(self) -> i32 {
        self.a
    }

    /// Returns `B`, the offset.
    #[must_use]
    pub const fn b(self) -> i32 {
        self.b
    }
}

impl fmt::Display for AnPlusB {
    /// Serializes the notation by the algorithm of section 9.1.
    ///
    /// When `A` is zero the result is `B` alone. Otherwise it is `n` for an `A`
    /// of 1, `-n` for -1 and the integer `A` followed by `n` for any other, and
    /// then, for a `B` other than zero, `B` with a `+` before a positive one.
    /// A `B` of zero is left out, as the algorithm's fourth step names only the
    /// positive and the negative case.
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        if self.a == 0 {
            return write!(formatter, "{}", self.b);
        }
        match self.a {
            1 => formatter.write_str("n")?,
            -1 => formatter.write_str("-n")?,
            a => write!(formatter, "{a}n")?,
        }
        match self.b {
            0 => Ok(()),
            b if b > 0 => write!(formatter, "+{b}"),
            b => write!(formatter, "{b}"),
        }
    }
}

enum Kind<'i, P> {
    Stream(StreamError<'i, P>),
    Overflow(Site<'i, P>),
}

/// The failure to read an `<a-n-plus-b>`: the tokens do not match the type, a
/// stream failure, or a value that does not fit an `i32`.
///
/// ```
/// use surgeist_css_parser::lexer::Options;
/// use surgeist_css_parser::limits::{DepthExcess, Limits};
/// use surgeist_css_parser::{Document, anb};
///
/// let document: Document =
///     Document::parse("99999999999n", &Options::default(), Limits::default(), DepthExcess::Fail)?;
/// let error = anb::parse(&mut document.root()).unwrap_err();
/// assert!(error.is_overflow());
/// let document: Document =
///     Document::parse("3 n", &Options::default(), Limits::default(), DepthExcess::Fail)?;
/// // `3` is an `<integer>`, so it matches and the stray `n` is left for the caller.
/// let mut stream = document.root();
/// assert_eq!(anb::parse(&mut stream).unwrap().b(), 3);
/// assert!(stream.require_empty().is_err());
/// # Ok::<(), surgeist_css_parser::document::BuildError>(())
/// ```
pub struct AnPlusBError<'i, P = ()> {
    kind: Kind<'i, P>,
}

impl<'i, P> AnPlusBError<'i, P> {
    /// Returns the boundary the failure is about.
    #[must_use]
    pub const fn at(&self) -> Site<'i, P> {
        match &self.kind {
            Kind::Stream(error) => error.at(),
            Kind::Overflow(at) => *at,
        }
    }

    /// Returns whether the notation matched but `A` or `B` is outside `i32`.
    #[must_use]
    pub const fn is_overflow(&self) -> bool {
        matches!(self.kind, Kind::Overflow(_))
    }

    /// Returns the stream failure, if the failure is one: a token that cannot
    /// start or continue the notation, the end of the stream, or a refusal.
    #[must_use]
    pub const fn stream_error(&self) -> Option<&StreamError<'i, P>> {
        match &self.kind {
            Kind::Stream(error) => Some(error),
            Kind::Overflow(_) => None,
        }
    }

    /// Takes the stream failure out, or gives the error back if it is an
    /// overflow.
    ///
    /// # Errors
    ///
    /// Returns `self` unchanged when the failure is an overflow.
    pub fn into_stream_error(self) -> Result<StreamError<'i, P>, Self> {
        match self.kind {
            Kind::Stream(error) => Ok(error),
            Kind::Overflow(_) => Err(self),
        }
    }

    /// Returns whether a token that does not fit the notation was found.
    #[must_use]
    pub const fn is_unexpected_token(&self) -> bool {
        matches!(&self.kind, Kind::Stream(error) if error.is_unexpected_token())
    }

    /// Returns the token that did not fit, if one did not.
    #[must_use]
    pub const fn unexpected_token(&self) -> Option<Token<'i>> {
        match &self.kind {
            Kind::Stream(error) => error.unexpected_token(),
            Kind::Overflow(_) => None,
        }
    }

    /// Returns whether the stream ended before the notation did.
    #[must_use]
    pub const fn is_end_of_stream(&self) -> bool {
        matches!(&self.kind, Kind::Stream(error) if error.is_end_of_stream())
    }

    /// Returns whether the meter refused an action; the host holds the reason.
    #[must_use]
    pub const fn is_refused(&self) -> bool {
        matches!(&self.kind, Kind::Stream(error) if error.is_refused())
    }
}

impl<'i, P> From<StreamError<'i, P>> for AnPlusBError<'i, P> {
    fn from(error: StreamError<'i, P>) -> Self {
        Self {
            kind: Kind::Stream(error),
        }
    }
}

impl<P> Clone for AnPlusBError<'_, P> {
    fn clone(&self) -> Self {
        Self {
            kind: match &self.kind {
                Kind::Stream(error) => Kind::Stream(error.clone()),
                Kind::Overflow(at) => Kind::Overflow(*at),
            },
        }
    }
}

impl<P> PartialEq for AnPlusBError<'_, P> {
    fn eq(&self, other: &Self) -> bool {
        match (&self.kind, &other.kind) {
            (Kind::Stream(left), Kind::Stream(right)) => left == right,
            (Kind::Overflow(left), Kind::Overflow(right)) => left == right,
            _ => false,
        }
    }
}

impl<P> fmt::Debug for AnPlusBError<'_, P> {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match &self.kind {
            Kind::Stream(error) => formatter.debug_tuple("Stream").field(error).finish(),
            Kind::Overflow(at) => formatter.debug_tuple("Overflow").field(at).finish(),
        }
    }
}

impl<P> fmt::Display for AnPlusBError<'_, P> {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match &self.kind {
            Kind::Stream(error) => fmt::Display::fmt(error, formatter),
            Kind::Overflow(_) => formatter.write_str("an An+B value does not fit an i32"),
        }
    }
}

impl<P> std::error::Error for AnPlusBError<'_, P> {}

/// Recognizes the `<a-n-plus-b>` type of section 6.2 at the start of `stream`,
/// after any whitespace, and interprets it.
///
/// The match is the longest the production allows: `2n + 1` is one notation,
/// and in `2n + x` the notation is `2n` and the rest is left in the stream for
/// the caller, as a grammar match leaves it. Whitespace is permitted between
/// the tokens of the production except between a `+` sign and the `n` it
/// precedes, where the footnote of the production forbids it; a comment is not a
/// token of the specification, so it does not separate them. Trailing whitespace
/// is not consumed.
///
/// A recognized notation whose `A` or `B` does not fit an `i32` is reported as
/// an overflow and not saturated.
///
/// On failure the stream is where it was.
///
/// # Errors
///
/// Fails with a [`stream error`](AnPlusBError::stream_error) when the tokens
/// do not match the production, with an [overflow](AnPlusBError::is_overflow)
/// when a value does not fit, and with `is_refused()` when the meter refuses.
///
/// ```
/// use surgeist_css_parser::lexer::Options;
/// use surgeist_css_parser::limits::{DepthExcess, Limits};
/// use surgeist_css_parser::{Document, anb};
///
/// let document: Document =
///     Document::parse("+3n - 2", &Options::default(), Limits::default(), DepthExcess::Fail)?;
/// let value = anb::parse(&mut document.root()).unwrap();
/// assert_eq!((value.a(), value.b()), (3, -2));
/// # Ok::<(), surgeist_css_parser::document::BuildError>(())
/// ```
pub fn parse<'i, P>(stream: &mut Stream<'i, P>) -> Result<AnPlusB, AnPlusBError<'i, P>> {
    let start = stream.mark();
    let result = recognize(stream);
    if result.is_err() {
        stream.restore(start);
    }
    result
}

/// The `n` part of a form, before its `B`.
#[derive(Clone, Copy)]
enum Tail {
    /// Nothing follows `n` in the token: `B` is optional.
    Open,
    /// The token ends in `n-`: a signless integer must follow.
    Dash,
    /// The token carries `B` as `-digits`: nothing follows.
    Done(i64),
}

fn overflow<'i, P>(at: Site<'i, P>) -> AnPlusBError<'i, P> {
    AnPlusBError {
        kind: Kind::Overflow(at),
    }
}

fn unexpected<'i, P>(at: Site<'i, P>, token: Token<'i>) -> AnPlusBError<'i, P> {
    StreamError::unexpected(at, token).into()
}

fn to_i32<'i, P>(value: i64, at: Site<'i, P>) -> Result<i32, AnPlusBError<'i, P>> {
    i32::try_from(value).map_err(|_| overflow(at))
}

/// The value of an integer-flagged number, or an overflow when the spelling is
/// too large even for an `i64`.
fn integer_value<'i, P>(number: Number<'i>, at: Site<'i, P>) -> Result<i64, AnPlusBError<'i, P>> {
    number.integer().ok_or_else(|| overflow(at))
}

/// Splits an `n`-form of an ident or dimension unit into the tail it spells:
/// `n` (open), `n-` (a dash), or `n-` followed by one or more digits. `text` is
/// the value after any leading `-`. `None` when it is none of them.
fn tail_of(text: &str) -> Option<Result<Tail, ()>> {
    let rest = text.strip_prefix('n').or_else(|| text.strip_prefix('N'))?;
    if rest.is_empty() {
        return Some(Ok(Tail::Open));
    }
    let digits = rest.strip_prefix('-')?;
    if digits.is_empty() {
        return Some(Ok(Tail::Dash));
    }
    if !digits.bytes().all(|byte| byte.is_ascii_digit()) {
        return None;
    }
    // `B` is the negation of the digits; too many digits for an i64 overflow.
    Some(
        digits
            .parse::<i64>()
            .map(|value| Tail::Done(-value))
            .map_err(|_| ()),
    )
}

fn recognize<'i, P>(stream: &mut Stream<'i, P>) -> Result<AnPlusB, AnPlusBError<'i, P>> {
    stream.discard_whitespace();
    let at = stream.site();
    let token = stream.consume_token()?;
    match token {
        Token::Number(number) if number.kind() == NumberKind::Integer => {
            // <integer>: A is 0, B is the value.
            let value = integer_value(number, at)?;
            Ok(AnPlusB::new(0, to_i32(value, at)?))
        }
        Token::Dimension { number, unit } if number.kind() == NumberKind::Integer => {
            let Some(tail) = tail_of(unit) else {
                return Err(unexpected(at, token));
            };
            let a = to_i32(integer_value(number, at)?, at)?;
            finish(stream, at, a, tail)
        }
        Token::Delim('+') => {
            // `'+'? n ...`: the ident must follow with nothing between.
            let next_at = stream.site();
            let next = stream.consume_token()?;
            let Token::Ident(name) = next else {
                return Err(unexpected(next_at, next));
            };
            let Some(tail) = tail_of(name) else {
                return Err(unexpected(next_at, next));
            };
            finish(stream, at, 1, tail)
        }
        Token::Ident(name) => {
            if name.eq_ignore_ascii_case("odd") {
                return Ok(AnPlusB::new(2, 1));
            }
            if name.eq_ignore_ascii_case("even") {
                return Ok(AnPlusB::new(2, 0));
            }
            if let Some(rest) = name.strip_prefix('-') {
                // `-n`, `-n-`, `<dashndashdigit-ident>`.
                return match tail_of(rest) {
                    Some(tail) => finish(stream, at, -1, tail),
                    None => Err(unexpected(at, token)),
                };
            }
            match tail_of(name) {
                Some(tail) => finish(stream, at, 1, tail),
                None => Err(unexpected(at, token)),
            }
        }
        other => Err(unexpected(at, other)),
    }
}

/// Completes a form whose `A` is known and whose first token ended with `tail`.
fn finish<'i, P>(
    stream: &mut Stream<'i, P>,
    at: Site<'i, P>,
    a: i32,
    tail: Result<Tail, ()>,
) -> Result<AnPlusB, AnPlusBError<'i, P>> {
    match tail.map_err(|()| overflow(at))? {
        Tail::Done(b) => Ok(AnPlusB::new(a, to_i32(b, at)?)),
        Tail::Dash => {
            // `n-` <signless-integer>: B is the negation of the integer.
            stream.discard_whitespace();
            let number_at = stream.site();
            let token = stream.consume_token()?;
            match token {
                Token::Number(number)
                    if number.kind() == NumberKind::Integer && number.sign().is_none() =>
                {
                    let value = integer_value(number, number_at)?;
                    Ok(AnPlusB::new(a, to_i32(-value, number_at)?))
                }
                other => Err(unexpected(number_at, other)),
            }
        }
        Tail::Open => open_tail(stream, a),
    }
}

/// After an `n`, reads the optional `B`: a signed integer, or a sign and a
/// signless integer. When neither follows, `B` is zero and the stream is left
/// right after the `n`.
fn open_tail<'i, P>(stream: &mut Stream<'i, P>, a: i32) -> Result<AnPlusB, AnPlusBError<'i, P>> {
    let mark = stream.mark();
    stream.discard_whitespace();
    let at = stream.site();
    let first = match stream.consume_token() {
        Ok(token) => token,
        Err(error) if error.is_refused() => return Err(error.into()),
        Err(_) => {
            stream.restore(mark);
            return Ok(AnPlusB::new(a, 0));
        }
    };
    match first {
        // <signed-integer>: a number with a sign character.
        Token::Number(number)
            if number.kind() == NumberKind::Integer && number.sign().is_some() =>
        {
            let value = integer_value(number, at)?;
            Ok(AnPlusB::new(a, to_i32(value, at)?))
        }
        Token::Delim(sign @ ('+' | '-')) => {
            // ['+' | '-'] <signless-integer>
            stream.discard_whitespace();
            let number_at = stream.site();
            match stream.consume_token() {
                Ok(Token::Number(number))
                    if number.kind() == NumberKind::Integer && number.sign().is_none() =>
                {
                    let value = integer_value(number, number_at)?;
                    let value = if sign == '-' { -value } else { value };
                    Ok(AnPlusB::new(a, to_i32(value, number_at)?))
                }
                Err(error) if error.is_refused() => Err(error.into()),
                Ok(_) | Err(_) => {
                    stream.restore(mark);
                    Ok(AnPlusB::new(a, 0))
                }
            }
        }
        _ => {
            stream.restore(mark);
            Ok(AnPlusB::new(a, 0))
        }
    }
}
