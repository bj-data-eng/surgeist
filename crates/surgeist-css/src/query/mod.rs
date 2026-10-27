//! The generic condition model and `@when` conditions.
//!
//! [`CssCondition`] is the one boolean condition model, generic over the leaf type of each
//! grammar ([`CssConditionLeaf`]): media, supports, container, style, scroll-state and `@when`.
//! [`CssConditionKind`] exposes the authored shape: `not`, `and`, `or`, an explicit parenthesized
//! condition, or a leaf. The grammar-specific leaves live in the submodules.
//!
//! # Authored when and else groups
//!
//! [`CssCondition<CssWhenLeaf>`](crate::query::CssCondition) owns the adopted symbolic grammar: homogeneous boolean
//! terms, explicit groups, single media-feature and supports-declaration leaves,
//! and valid opaque enclosures. `CssCondition::<CssWhenLeaf>::parse` returns checked syntax
//! without evaluation. [`CssWhenRule`](crate::rule::CssWhenRule) and [`CssElseRule`](crate::rule::CssElseRule) retain ordinary children;
//! their scoped counterparts retain the separate scoped grammar. Parsed Else
//! chains require original-token adjacency after complete recovery reconstruction.
//! Checked enclosing assembly instead validates actual supplied sibling order.
//! Normalization retains immutable branch headers and nearest style contexts.
//! Shared specified output composes the graph under cumulative limits; ordinary
//! literal CSSOM returns [`CssRuleCssomKind::When`](crate::serialization::CssRuleCssomKind::When) or [`CssRuleCssomKind::Else`](crate::serialization::CssRuleCssomKind::Else)
//! as source-undefined. The current Scope literal front remains format-unavailable.
//! See the [adopted authored profile](https://github.com/bj-data-eng/surgeist/issues/647).

pub(crate) mod condition;
pub mod container;
pub mod media;
pub mod supports;
pub(crate) mod syntax;
pub(crate) mod when;

#[doc(inline)]
pub use condition::{CssCondition, CssConditionKind, CssConditionLeaf, CssConditionList};
#[doc(inline)]
pub use syntax::{CssGeneralEnclosed, CssQueryComparison, CssQueryLength, CssRangeFeature};
#[doc(inline)]
pub use when::{
    CssConditionConstructionError, CssWhenLeaf, CssWhenMediaFeature, CssWhenMediaFeatureKind,
};
