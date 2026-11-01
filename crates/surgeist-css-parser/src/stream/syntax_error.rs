//! The failure of a grammar.

use std::fmt;

use super::{Site, StreamError};
use crate::Token;

enum Kind<'i, E, P> {
    Stream(StreamError<'i, P>),
    Custom { at: Site<'i, P>, error: Box<E> },
    UnknownAtRule { at: Site<'i, P>, name: &'i str },
    InvalidAtRuleBody { at: Site<'i, P> },
    InvalidQualifiedRule { at: Site<'i, P> },
    DeniedBlock { at: Site<'i, P> },
}

/// A grammar failure: a stream-level failure, a rejection by the rule
/// protocol, or the grammar's own error `E`.
///
/// This is the specification's "syntax error" (section 5.4), carried with
/// where it happened. A grammar returns it from every function that consumes a
/// [`Stream`](crate::Stream); `E` is whatever the grammar says went wrong, and
/// the crate gives it no meaning. The error is not an enum: it reports its
/// facts through `is_*` predicates and accessors, and the grammar's own error
/// is boxed, so the error stays small however large `E` is.
///
/// A [`StreamError`] converts into a `SyntaxError` with `?`.
///
/// ```
/// use surgeist_css_parser::lexer::Options;
/// use surgeist_css_parser::limits::{DepthExcess, Limits};
/// use surgeist_css_parser::{Document, Stream, SyntaxError};
///
/// #[derive(Debug, PartialEq)]
/// struct NotBold;
///
/// fn bold<'i>(stream: &mut Stream<'i>) -> Result<(), SyntaxError<'i, NotBold>> {
///     stream.consume_keyword("bold")?;
///     Ok(())
/// }
///
/// let document: Document =
///     Document::parse("bold normal", &Options::default(), Limits::default(), 0, DepthExcess::Fail)?;
/// let mut stream = document.root();
/// assert!(bold(&mut stream).is_ok());
/// let error = bold(&mut stream).unwrap_err();
/// assert!(error.is_unexpected_token());
/// let custom: SyntaxError<'_, NotBold> = SyntaxError::custom(stream.site(), NotBold);
/// assert_eq!(custom.custom_value(), Some(&NotBold));
/// # Ok::<(), surgeist_css_parser::document::BuildError>(())
/// ```
pub struct SyntaxError<'i, E, P = ()> {
    kind: Kind<'i, E, P>,
}

impl<'i, E, P> SyntaxError<'i, E, P> {
    /// Creates the error a grammar reports for itself: `error`, at `at`.
    #[must_use]
    pub fn custom(at: Site<'i, P>, error: E) -> Self {
        Self {
            kind: Kind::Custom {
                at,
                error: Box::new(error),
            },
        }
    }

    /// Creates the rejection of an at-rule whose name `name` the grammar does
    /// not know (section 5.5.2, an invalid rule error).
    #[must_use]
    pub const fn unknown_at_rule_error(at: Site<'i, P>, name: &'i str) -> Self {
        Self {
            kind: Kind::UnknownAtRule { at, name },
        }
    }

    /// Creates the rejection of an at-rule whose body the grammar does not
    /// accept (section 5.5.2).
    #[must_use]
    pub const fn invalid_at_rule_body_error(at: Site<'i, P>) -> Self {
        Self {
            kind: Kind::InvalidAtRuleBody { at },
        }
    }

    /// Creates the rejection of a qualified rule the grammar does not accept
    /// (section 5.5.3, an invalid rule error).
    #[must_use]
    pub const fn invalid_qualified_rule_error(at: Site<'i, P>) -> Self {
        Self {
            kind: Kind::InvalidQualifiedRule { at },
        }
    }

    /// Creates the failure of a rule whose `{}`-block the build denied for
    /// nesting too deep: the block has no contents to give the grammar.
    pub(crate) const fn denied_block_error(at: Site<'i, P>) -> Self {
        Self {
            kind: Kind::DeniedBlock { at },
        }
    }

    /// Returns the boundary the failure is about.
    #[must_use]
    pub const fn at(&self) -> Site<'i, P> {
        match &self.kind {
            Kind::Stream(error) => error.at(),
            Kind::Custom { at, .. }
            | Kind::UnknownAtRule { at, .. }
            | Kind::InvalidAtRuleBody { at }
            | Kind::InvalidQualifiedRule { at }
            | Kind::DeniedBlock { at } => *at,
        }
    }

    kind_predicates! { kind;
        /// Whether the grammar reported its own error; see
        /// [`custom_value`](Self::custom_value).
        is_custom => Kind::Custom { .. },
        /// Whether the token stream failed; see
        /// [`stream_error`](Self::stream_error).
        is_stream_error => Kind::Stream(_),
        /// Whether the rule protocol rejected an at-rule for its name.
        is_unknown_at_rule => Kind::UnknownAtRule { .. },
        /// Whether the rule protocol rejected an at-rule for its body.
        is_invalid_at_rule_body => Kind::InvalidAtRuleBody { .. },
        /// Whether the rule protocol rejected a qualified rule.
        is_invalid_qualified_rule => Kind::InvalidQualifiedRule { .. },
        /// Whether a rule driver could not give the grammar a rule because its
        /// `{}`-block nested deeper than the limit of the build that denied it
        /// (see [`DepthExcess::DenyBlock`](crate::limits::DepthExcess::DenyBlock)).
        is_denied_block => Kind::DeniedBlock { .. },
    }

    /// Returns the grammar's own error, if the failure is one.
    #[must_use]
    pub fn custom_value(&self) -> Option<&E> {
        match &self.kind {
            Kind::Custom { error, .. } => Some(error.as_ref()),
            _ => None,
        }
    }

    /// Takes the grammar's own error out, or gives the failure back if it is
    /// another kind.
    ///
    /// # Errors
    ///
    /// Returns `self` unchanged when the failure is not the grammar's own.
    pub fn into_custom(self) -> Result<E, Self> {
        match self.kind {
            Kind::Custom { error, .. } => Ok(*error),
            kind => Err(Self { kind }),
        }
    }

    /// Returns the stream-level failure, if the failure is one.
    #[must_use]
    pub const fn stream_error(&self) -> Option<&StreamError<'i, P>> {
        match &self.kind {
            Kind::Stream(error) => Some(error),
            _ => None,
        }
    }

    /// Returns whether a token other than the one asked for was next.
    #[must_use]
    pub const fn is_unexpected_token(&self) -> bool {
        matches!(&self.kind, Kind::Stream(error) if error.is_unexpected_token())
    }

    /// Returns the token that was there instead, if a token was.
    #[must_use]
    pub const fn unexpected_token(&self) -> Option<Token<'i>> {
        match &self.kind {
            Kind::Stream(error) => error.unexpected_token(),
            _ => None,
        }
    }

    /// Returns whether the stream was empty where a token was required.
    #[must_use]
    pub const fn is_end_of_stream(&self) -> bool {
        matches!(&self.kind, Kind::Stream(error) if error.is_end_of_stream())
    }

    /// Returns whether the meter refused the action; the host holds the
    /// reason.
    #[must_use]
    pub const fn is_refused(&self) -> bool {
        matches!(&self.kind, Kind::Stream(error) if error.is_refused())
    }

    /// Returns whether a block was to be entered and none was pending.
    #[must_use]
    pub const fn is_block_expected(&self) -> bool {
        matches!(&self.kind, Kind::Stream(error) if error.is_block_expected())
    }

    /// Returns the name of the at-rule the rule protocol rejected for its name,
    /// if that is the failure.
    #[must_use]
    pub const fn unknown_at_rule(&self) -> Option<&'i str> {
        match &self.kind {
            Kind::UnknownAtRule { name, .. } => Some(name),
            _ => None,
        }
    }
}

impl<'i, E, P> From<StreamError<'i, P>> for SyntaxError<'i, E, P> {
    fn from(error: StreamError<'i, P>) -> Self {
        Self {
            kind: Kind::Stream(error),
        }
    }
}

impl<E: fmt::Debug, P> fmt::Debug for SyntaxError<'_, E, P> {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match &self.kind {
            Kind::Stream(error) => formatter.debug_tuple("Stream").field(error).finish(),
            Kind::Custom { at, error } => formatter
                .debug_struct("Custom")
                .field("at", at)
                .field("error", error)
                .finish(),
            Kind::UnknownAtRule { at, name } => formatter
                .debug_struct("UnknownAtRule")
                .field("at", at)
                .field("name", name)
                .finish(),
            Kind::InvalidAtRuleBody { at } => formatter
                .debug_struct("InvalidAtRuleBody")
                .field("at", at)
                .finish(),
            Kind::InvalidQualifiedRule { at } => formatter
                .debug_struct("InvalidQualifiedRule")
                .field("at", at)
                .finish(),
            Kind::DeniedBlock { at } => formatter
                .debug_struct("DeniedBlock")
                .field("at", at)
                .finish(),
        }
    }
}

impl<E: fmt::Display, P> fmt::Display for SyntaxError<'_, E, P> {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match &self.kind {
            Kind::Stream(error) => fmt::Display::fmt(error, formatter),
            Kind::Custom { error, .. } => fmt::Display::fmt(error, formatter),
            Kind::UnknownAtRule { name, .. } => write!(formatter, "unknown at-rule @{name}"),
            Kind::InvalidAtRuleBody { .. } => formatter.write_str("invalid at-rule body"),
            Kind::InvalidQualifiedRule { .. } => formatter.write_str("invalid qualified rule"),
            Kind::DeniedBlock { .. } => {
                formatter.write_str("a block nested deeper than the limit was denied")
            }
        }
    }
}

impl<E: fmt::Debug + fmt::Display, P> std::error::Error for SyntaxError<'_, E, P> {}
