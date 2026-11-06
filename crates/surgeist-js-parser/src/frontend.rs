//! Work admitted before frontend operations.

use std::fmt;

use crate::source::ReadUnit;

/// Work about to be performed by the frontend.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum FrontendWork {
    /// One decoder input attempt.
    Read(ReadUnit),
    /// Consume one decoded point and retain its UTF-16 representation together.
    Consume {
        /// Width of the decoded point's UTF-16 representation.
        retained_units: u8,
    },
    /// One bounded lexer lookahead request, including cache hits and EOF probes.
    Lookahead,
    /// One lexical production dispatch, including comments and continuations.
    LexicalDispatch,
    /// One bounded buffered-token probe, including cache hits and EOF.
    TokenProbe,
    /// One buffered-token advance request, including requests at EOF.
    TokenAdvance,
    /// One grammar production attempt, including an inline repeated item.
    GrammarDispatch,
    /// One AST inspection during an early-error walk.
    EarlyErrorVisit,
    /// One logical label-table lookup, including scoped registration and cleanup when needed.
    LabelLookup,
}

/// The frontend budget dimension that refused work.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum FrontendLimit {
    /// Maximum admitted decoder input attempts.
    Read(u64),
    /// Maximum admitted decoded code points.
    Consumption(u64),
    /// Maximum admitted retained UTF-16 units.
    Retention(u64),
    /// Maximum admitted lexer lookahead requests.
    Lookahead(u64),
    /// Maximum admitted lexical production dispatches.
    LexicalDispatch(u64),
    /// Maximum admitted buffered-token probes.
    TokenProbe(u64),
    /// Maximum admitted buffered-token advance requests.
    TokenAdvance(u64),
    /// Maximum admitted grammar production attempts.
    GrammarDispatch(u64),
    /// Maximum admitted early-error AST inspections.
    EarlyErrorVisit(u64),
    /// Maximum admitted logical label-table lookups.
    LabelLookup(u64),
}

impl fmt::Display for FrontendLimit {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        let (limit, units) = match self {
            Self::Read(limit) => (limit, "source reads"),
            Self::Consumption(limit) => (limit, "consumed source code points"),
            Self::Retention(limit) => (limit, "retained UTF-16 source units"),
            Self::Lookahead(limit) => (limit, "lexer lookahead probes"),
            Self::LexicalDispatch(limit) => (limit, "lexer dispatches"),
            Self::TokenProbe(limit) => (limit, "buffered-token probes"),
            Self::TokenAdvance(limit) => (limit, "buffered-token advance requests"),
            Self::GrammarDispatch(limit) => (limit, "grammar dispatches"),
            Self::EarlyErrorVisit(limit) => (limit, "early-error AST visits"),
            Self::LabelLookup(limit) => (limit, "label lookups"),
        };
        write!(
            formatter,
            "reached the operation's limit of {limit} {units}"
        )
    }
}

impl std::error::Error for FrontendLimit {}

/// The caller's admission policy for frontend work.
pub type FrontendControl<'control> =
    dyn FnMut(FrontendWork) -> Result<(), FrontendLimit> + Send + Sync + 'control;

pub(crate) struct Control<'control>(pub(crate) Option<&'control mut FrontendControl<'control>>);

impl Control<'_> {
    #[inline]
    pub(crate) fn admit(&mut self, work: FrontendWork) -> Result<(), FrontendLimit> {
        self.0.as_deref_mut().map_or(Ok(()), |admit| admit(work))
    }
}

impl fmt::Debug for Control<'_> {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_tuple("Control")
            .field(&self.0.is_some())
            .finish()
    }
}
