//! The error of a [`TokenCursor`](super::TokenCursor) call.

use std::fmt;

/// A protocol violation by the host of a [`TokenCursor`](super::TokenCursor).
///
/// Every error leaves the cursor unchanged, so the pending request remains
/// valid. Ask what happened with the `is_*` predicates; the cause is private so
/// that a new one is not a breaking change.
///
/// ```
/// use surgeist_css_parser::input::Scalar;
/// use surgeist_css_parser::lexer::protocol::{TokenCursor, TokenRequest};
///
/// let mut cursor = TokenCursor::new(1, 0);
/// assert_eq!(cursor.request(), TokenRequest::ReadScalar { index: 0 });
/// // Section 3.3 turns a NULL into U+FFFD before the tokenizer sees it, so
/// // the cursor refuses one.
/// let nul = Scalar { value: '\0', start: 0, end: 1 };
/// let error = cursor.read_scalar(0, nul).unwrap_err();
/// assert!(error.is_unpreprocessed_scalar());
/// // The refusal changed nothing: the same scalar is still wanted.
/// assert_eq!(cursor.request(), TokenRequest::ReadScalar { index: 0 });
/// ```
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct ProtocolError {
    kind: ProtocolErrorKind,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum ProtocolErrorKind {
    WrongReply,
    WrongIndex,
    Boundary,
    Overflow,
    Complete,
    UnpreprocessedScalar,
}

impl ProtocolError {
    pub(crate) const WRONG_REPLY: Self = Self {
        kind: ProtocolErrorKind::WrongReply,
    };
    pub(crate) const WRONG_INDEX: Self = Self {
        kind: ProtocolErrorKind::WrongIndex,
    };
    pub(crate) const BOUNDARY: Self = Self {
        kind: ProtocolErrorKind::Boundary,
    };
    pub(crate) const OVERFLOW: Self = Self {
        kind: ProtocolErrorKind::Overflow,
    };
    pub(crate) const COMPLETE: Self = Self {
        kind: ProtocolErrorKind::Complete,
    };
    pub(crate) const UNPREPROCESSED_SCALAR: Self = Self {
        kind: ProtocolErrorKind::UnpreprocessedScalar,
    };

    kind_predicates! { kind;
        /// Whether the call does not answer the pending
        /// [`TokenRequest`](super::TokenRequest).
        is_wrong_reply => ProtocolErrorKind::WrongReply,
        /// Whether the slot index differs from the pending append's.
        is_wrong_index => ProtocolErrorKind::WrongIndex,
        /// Whether a scalar is empty or does not continue the previous scalar's
        /// interval, or a consumption exceeds the cached lookahead.
        is_boundary => ProtocolErrorKind::Boundary,
        /// Whether a position, slot or digit count does not fit `usize`.
        is_overflow => ProtocolErrorKind::Overflow,
        /// Whether the cursor already finished.
        is_complete => ProtocolErrorKind::Complete,
        /// Whether the scalar is U+0000, U+000D or U+000C, which section 3.3
        /// replaces before tokenizing, so a preprocessed stream never holds
        /// one.
        is_unpreprocessed_scalar => ProtocolErrorKind::UnpreprocessedScalar,
    }
}

impl fmt::Display for ProtocolError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(match self.kind {
            ProtocolErrorKind::WrongReply => "the call does not answer the pending token request",
            ProtocolErrorKind::WrongIndex => "the slot index is not the pending append's",
            ProtocolErrorKind::Boundary => {
                "the scalar is empty or does not continue the previous interval"
            }
            ProtocolErrorKind::Overflow => "a position, slot or digit count does not fit usize",
            ProtocolErrorKind::Complete => "the token cursor already finished",
            ProtocolErrorKind::UnpreprocessedScalar => {
                "the scalar is a NULL, carriage return or form feed that preprocessing replaces"
            }
        })
    }
}

impl std::error::Error for ProtocolError {}
