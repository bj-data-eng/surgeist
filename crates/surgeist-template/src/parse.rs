//! Diagnostics reported by template parsing.
//!
//! [`parse_template`](crate::parse_template) returns a [`ParseError`] for the
//! first strictness violation in the source text. The error carries a
//! [`ParseErrorKind`] describing the violation and the
//! [`SourceSpan`](crate::span::SourceSpan) of the offending source, so callers
//! can point at authored text. Parsing does not recover from malformed tags the
//! way a browser would.

#[doc(inline)]
pub use crate::error::{ParseError, ParseErrorKind};
