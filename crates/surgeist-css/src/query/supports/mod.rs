//! Supports conditions and named supports test bodies.
//!
//! # Supports rules
//!
//! Supports rules retain declaration tests, boolean grouping, complete Selectors 3 plus the
//! selected existing selector extensions as the typed `selector()` subset, and balanced
//! general-enclosed fallback syntax. Unselected Selectors 4 pseudo-classes and
//! pseudo-elements, and syntax outside the named extension rows remain outside the typed subset.
//! These nodes describe authored tests; the crate never evaluates whether a condition matches.
//! `font-tech()` retains one shared [`CssFontTechHint`](crate::rule::font_face::CssFontTechHint), `font-format()` retains
//! the distinct keyword or string [`CssFontFormat`](crate::rule::font_face::CssFontFormat), and `at-rule()` retains a
//! genuine at-keyword argument through [`CssSupportsAtRule`]. Unknown names and
//! `@charset` remain authored predicates. Balanced invalid arguments use the
//! general-enclosed fallback; lexical and resource failures remain terminal.
//!
//! ```
//! use surgeist_css::query::CssConditionKind;
//! use surgeist_css::rule::CssRule;
//! use surgeist_css::query::supports::CssSupportsLeaf;
//! use surgeist_css::CssParser;
//!
//! let report = CssParser::default().parse_sheet(concat!(
//!     "@supports selector(.card > .item:hover) {}",
//!     "@supports future-layout(mode) {}",
//! ));
//! assert!(report.is_clean());
//! let [CssRule::Supports(selector), CssRule::Supports(fallback)] =
//!     report.syntax().rules()
//! else {
//!     panic!("expected supports rules");
//! };
//! assert!(matches!(
//!     selector.condition().kind(),
//!     CssConditionKind::Leaf(CssSupportsLeaf::Selector(_))
//! ));
//! assert!(matches!(
//!     fallback.condition().kind(),
//!     CssConditionKind::Leaf(CssSupportsLeaf::GeneralEnclosed(value))
//!         if value.authored() == Some("future-layout(mode)")
//! ));
//! ```

pub(crate) mod construction;
pub(crate) mod named_supports;
pub(crate) mod named_supports_composition;
pub(crate) mod named_supports_serialization;
pub(crate) mod syntax;

#[doc(inline)]
pub use construction::{CssSupportsAtRule, CssSupportsConstructionError};
#[doc(inline)]
pub use named_supports::{
    CssSupportsAtRuleTest, CssSupportsConditionName, CssSupportsQualifiedRuleTest,
    CssSupportsTestBody, CssSupportsTestChild, CssSupportsTestChildRef, CssSupportsTestDeclaration,
    CssSupportsTestDeclarations, CssSupportsTestItem,
};
#[doc(inline)]
pub use syntax::{CssSupportsDeclaration, CssSupportsLeaf};
