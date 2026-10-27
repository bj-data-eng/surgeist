//! Shared value shapes, identifiers, URLs and box-edge keywords.

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
