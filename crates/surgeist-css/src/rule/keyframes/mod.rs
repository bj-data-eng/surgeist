//! `@keyframes` rules, selectors and matching.
//!
//! # Keyframes structure
//!
//! [`CssKeyframesRule`] and [`CssKeyframeBlock`] preserve source structure. Empty
//! rules and blocks remain present, while repeated selector blocks, equivalent
//! offsets across blocks, and repeated equivalent selectors within one
//! [`CssKeyframeSelectorList`] remain in authored order. Dropping an invalid
//! declaration leaves its valid empty parents; an invalid selector still drops
//! the smallest invalid block. The parser does not sort, merge, or deduplicate
//! keyframes.
//!
//! # Keyframe selectors
//!
//! `@keyframes` supports the complete selected authored production. Quoted names
//! admit empty and whitespace-only text. Literal percentage offsets use checked
//! binary64; percentage-valued calculations retain typed graphs and origins.
//! [`CssKeyframePercent::literal_value`](crate::rule::keyframes::CssKeyframePercent::literal_value) and [`CssKeyframePercent::calculation`](crate::rule::keyframes::CssKeyframePercent::calculation)
//! expose those states, while [`CssKeyframeSelector::offset`](crate::rule::keyframes::CssKeyframeSelector::offset) clones the authored
//! domain without evaluating it. Keyframe mode drops the eight prohibited Animation
//! declarations and retains animation-timing-function even at the final endpoint.
//! Computed range clamping, endpoint easing use and name lookup remain downstream.
//! Authored equality preserves quoted versus identifier form; numeric offsets compare by value.
//! This crate does not perform Grid
//! layout, cascade declarations, evaluate or interpolate keyframes, run
//! timelines, or lower either syntax family into sibling Surgeist crates.

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
