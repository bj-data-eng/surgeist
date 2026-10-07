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

    pub(crate) fn into_parts(self) -> (T, CssParsedOrigin) {
        (self.body, self.origin)
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

/// An original-source ordinary inner rule list retained by its parsing owner.
///
/// Recovery diagnostics belong to the parsing report. This list supplies no
/// stylesheet metadata or enclosing rule name and has no public assembly path.
#[derive(Clone, Debug, PartialEq)]
pub struct CssRuleList {
    rules: Vec<crate::CssRule>,
}

impl CssRuleList {
    pub(crate) fn from_parsed(rules: Vec<crate::CssRule>) -> Self {
        Self { rules }
    }

    pub(crate) fn rules_mut(&mut self) -> &mut Vec<crate::CssRule> {
        &mut self.rules
    }

    /// Borrows admitted rules in original source order.
    #[must_use]
    pub fn rules(&self) -> &[crate::CssRule] {
        &self.rules
    }
}

/// Actual enclosing style ancestry for an authored scoped body.
///
/// An enclosing style rule supplies declaration and parent-selector permission,
/// including through scopes. This does not bind or invent a parent selector.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum CssStyleAncestor {
    /// No enclosing style rule supplies style ancestry.
    Absent,
    /// An enclosing style rule supplies style ancestry.
    Present,
}
