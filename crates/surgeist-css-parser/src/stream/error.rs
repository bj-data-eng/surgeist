//! The failure of a token stream to supply what was asked of it.

use std::fmt;

use super::Site;
use crate::Token;

#[derive(Clone, Copy, Debug, PartialEq)]
pub(super) enum StreamErrorKind<'i> {
    UnexpectedToken(Token<'i>),
    EndOfStream,
    BlockExpected,
    Refused,
}

/// A stream-level failure: the token stream could not supply what the caller
/// asked for.
///
/// Section 5 words these as a syntax error, or as "return nothing". Which of
/// them applies is a fact the error reports: [`unexpected_token`] names the
/// token that was there, [`is_end_of_stream`] says the `<eof-token>` was,
/// [`is_block_expected`] says a block was to be entered and none was
/// pending, and [`is_refused`] says the [`Meter`](crate::limits::Meter) would
/// not admit the observation. [`at`] is the boundary the failure is about.
///
/// A `StreamError` converts into a [`SyntaxError`](crate::SyntaxError) with
/// `?`, so a grammar that returns the latter can use the consuming methods of a
/// [`Stream`](crate::Stream) directly.
///
/// ```
/// use surgeist_css_parser::lexer::Options;
/// use surgeist_css_parser::limits::{DepthExcess, Limits};
/// use surgeist_css_parser::{Document, Token};
///
/// let document: Document =
///     Document::parse("1", &Options::default(), Limits::default(), DepthExcess::Fail)?;
/// let mut stream = document.root();
/// let error = stream.consume_ident().unwrap_err();
/// assert!(error.is_unexpected_token());
/// assert!(matches!(error.unexpected_token(), Some(Token::Number(_))));
/// let error = stream.consume_ident().unwrap_err();
/// assert!(error.is_end_of_stream());
/// # Ok::<(), surgeist_css_parser::document::BuildError>(())
/// ```
///
/// [`unexpected_token`]: Self::unexpected_token
/// [`is_end_of_stream`]: Self::is_end_of_stream
/// [`is_block_expected`]: Self::is_block_expected
/// [`is_refused`]: Self::is_refused
/// [`at`]: Self::at
pub struct StreamError<'i, P = ()> {
    at: Site<'i, P>,
    kind: StreamErrorKind<'i>,
}

impl<P> Clone for StreamError<'_, P> {
    fn clone(&self) -> Self {
        Self {
            at: self.at,
            kind: self.kind,
        }
    }
}

impl<P> PartialEq for StreamError<'_, P> {
    fn eq(&self, other: &Self) -> bool {
        self.at == other.at && self.kind == other.kind
    }
}

impl<P> fmt::Debug for StreamError<'_, P> {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("StreamError")
            .field("at", &self.at)
            .field("kind", &self.kind)
            .finish()
    }
}

impl<'i, P> StreamError<'i, P> {
    pub(crate) const fn unexpected(at: Site<'i, P>, token: Token<'i>) -> Self {
        Self {
            at,
            kind: StreamErrorKind::UnexpectedToken(token),
        }
    }

    pub(crate) const fn end_of_stream(at: Site<'i, P>) -> Self {
        Self {
            at,
            kind: StreamErrorKind::EndOfStream,
        }
    }

    pub(crate) const fn block_expected(at: Site<'i, P>) -> Self {
        Self {
            at,
            kind: StreamErrorKind::BlockExpected,
        }
    }

    pub(crate) const fn refused(at: Site<'i, P>) -> Self {
        Self {
            at,
            kind: StreamErrorKind::Refused,
        }
    }

    /// Returns the boundary the failure is about: where the stream was when it
    /// could not supply what was asked.
    #[must_use]
    pub const fn at(&self) -> Site<'i, P> {
        self.at
    }

    /// Returns the token that was there instead, if a token was.
    #[must_use]
    pub const fn unexpected_token(&self) -> Option<Token<'i>> {
        match self.kind {
            StreamErrorKind::UnexpectedToken(token) => Some(token),
            _ => None,
        }
    }

    kind_predicates! { kind;
        /// Whether a token other than the one asked for was next.
        is_unexpected_token => StreamErrorKind::UnexpectedToken(_),
        /// Whether the stream was empty where a token was required: the next
        /// token was the `<eof-token>`.
        is_end_of_stream => StreamErrorKind::EndOfStream,
        /// Whether a function or simple block was to be entered and none was
        /// pending.
        is_block_expected => StreamErrorKind::BlockExpected,
        /// Whether the [`Meter`](crate::limits::Meter) refused the action; the
        /// host holds the reason.
        is_refused => StreamErrorKind::Refused,
    }
}

impl<P> fmt::Display for StreamError<'_, P> {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self.kind {
            StreamErrorKind::UnexpectedToken(token) => {
                write!(formatter, "unexpected {:?} token", token.kind())
            }
            StreamErrorKind::EndOfStream => formatter.write_str("unexpected end of the stream"),
            StreamErrorKind::BlockExpected => {
                formatter.write_str("no function or block is waiting to be entered")
            }
            StreamErrorKind::Refused => formatter.write_str("the meter refused the action"),
        }
    }
}

impl<P> std::error::Error for StreamError<'_, P> {}
