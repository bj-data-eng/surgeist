//! Shared value shapes, identifiers, URLs and box-edge keywords.
//!
//! Shapes shared across value families: [`CssAutoOr`], the four-side ([`CssFourSides`]) and
//! side-pair ([`CssSidePair`]) containers, identifiers ([`CssIdent`], [`CssCustomIdent`]), URLs with
//! their request modifiers, and the box-edge keywords.
//!
//! # Custom identifiers
//!
//! [`CssCustomIdent`] owns decoded custom-identifier admission. Timing names refine
//! that checked value through [`CssKeyframesIdent`](crate::rule::keyframes::CssKeyframesIdent) (shared by keyframe definitions
//! and animation names) and [`CssTransitionPropertyName`](crate::values::animation::CssTransitionPropertyName). [`CssGridLineNames`](crate::values::layout::CssGridLineNames)
//! contains checked [`CssGridLineName`](crate::values::layout::CssGridLineName) values with the Grid exclusions. Parsing
//! and public construction both reject mixed `None` transition lists.

pub(crate) mod common_values;
pub(crate) mod syntax;
pub(crate) mod url;
pub(crate) mod url_serialization;

#[doc(inline)]
pub use common_values::{CssAutoOr, CssBoxSideKind, CssFourSides, CssSidePair};
#[doc(inline)]
pub use syntax::{CssBoxEdgeKeyword, CssCustomIdent, CssIdent};
#[doc(inline)]
pub use url::{
    CssRequestUrlModifierRef, CssUrl, CssUrlCrossOrigin, CssUrlFunction, CssUrlModifier,
    CssUrlModifierFunction, CssUrlReferrerPolicy,
};
