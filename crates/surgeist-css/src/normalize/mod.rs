//! Normalized style sheets and their limits and errors.
//!
//! # Immutable stylesheet normalization
//!
//! `CssSheet::normalize` preserves an ordered stream of rule occurrences and
//! grouped declaration contributions. `CssParseReport::normalize` also retains every
//! original recovery diagnostic. Shared rule and selector contexts preserve
//! nesting, empty rules, conditions, imports, layers, scopes, and complete
//! nonstyle payloads without multiplying selectors or selecting cascade winners.
//! The current property-expansion slice remains explicit: valid declarations
//! outside it fail atomically with [`CssNormalizationError`].
//!
//! ```
//! use surgeist_css::normalize::CssNormalizedItem;
//! use surgeist_css::CssParser;
//!
//! let report = CssParser::default().parse_sheet(".a, #b { margin-block: 1px; & .child { padding-block: 2px } }");
//! let normalized = report.normalize(Default::default()).expect("supported expansion");
//! assert!(normalized.is_clean());
//! assert_eq!(normalized.syntax().items().len(), 4);
//! let [CssNormalizedItem::Rule(_), CssNormalizedItem::Declaration(parent),
//!      CssNormalizedItem::Rule(_), CssNormalizedItem::Declaration(child)] =
//!     normalized.syntax().items() else { unreachable!() };
//! assert!(child.selector_context().parent().unwrap()
//!     .same_context(parent.selector_context()));
//! ```

pub(crate) mod normalization;

#[doc(inline)]
pub use normalization::{
    CssNormalizationError, CssNormalizationLimits, CssNormalizationResource,
    CssNormalizedDeclaration, CssNormalizedItem, CssNormalizedReport, CssNormalizedSelector,
    CssNormalizedSheet, CssRuleContext, CssRuleContextKindRef, CssSelectorBinding,
    CssSelectorContext,
};
