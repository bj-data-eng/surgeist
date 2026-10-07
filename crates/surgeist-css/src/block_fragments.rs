//! Source-produced envelopes for genuine CSS block fragments.

use crate::CssParsedOrigin;

/// An admitted grammar-specific body and the origin of its genuine curly block.
///
/// The origin includes the opening brace and explicit closing brace, or ends at
/// original EOF for an implicit closure. Surrounding trivia remains in the shared
/// source snapshot but lies outside this span. Parsing reports own recovery
/// diagnostics; the presence of a fragment does not imply a clean report.
#[derive(Clone, Debug, PartialEq)]
pub struct CssBlockFragment<T> {
    body: T,
    origin: CssParsedOrigin,
}

impl<T> CssBlockFragment<T> {
    pub(crate) fn from_parsed(body: T, origin: CssParsedOrigin) -> Self {
        Self { body, origin }
    }

    /// Borrows the body retained by its grammar and recovery owner.
    #[must_use]
    pub const fn body(&self) -> &T {
        &self.body
    }

    /// Borrows the original-source range of the consumed curly block.
    #[must_use]
    pub const fn origin(&self) -> &CssParsedOrigin {
        &self.origin
    }
}
