//! Normalized style sheets and their limits and errors.

pub(crate) mod normalization;

#[doc(inline)]
pub use normalization::{
    CssNormalizationError, CssNormalizationLimits, CssNormalizationResource,
    CssNormalizedDeclaration, CssNormalizedItem, CssNormalizedReport, CssNormalizedSelector,
    CssNormalizedSheet, CssRuleContext, CssRuleContextKindRef, CssSelectorBinding,
    CssSelectorContext,
};
