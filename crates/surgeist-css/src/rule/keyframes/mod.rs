//! `@keyframes` rules, selectors and matching.

pub(crate) mod keyframe_matching;
pub(crate) mod keyframe_serialization;
pub(crate) mod page_keyframe_serialization;
pub(crate) mod parsed;
pub(crate) mod syntax;

#[doc(inline)]
pub use keyframe_matching::{
    CssKeyframeSelectorComparisonError, CssKeyframeSelectorComparisonStage,
    CssKeyframeSelectorMatcher,
};
#[doc(inline)]
pub use keyframe_serialization::{
    CssKeyframeRuleView, CssKeyframeRuleViewError, CssKeyframesRuleView,
};
#[doc(inline)]
pub use parsed::{CssParsedKeyframeRule, CssParsedKeyframeSelectors};
#[doc(inline)]
pub use syntax::{
    CssKeyframeBlock, CssKeyframeDeclaration, CssKeyframeDeclarationList, CssKeyframePercent,
    CssKeyframeSelector, CssKeyframeSelectorList, CssKeyframesIdent, CssKeyframesName,
    CssKeyframesRule, CssKeyframesString,
};
