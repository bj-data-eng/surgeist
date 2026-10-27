//! Animation, transition and easing values.

pub(crate) mod easing_serialization;
pub(crate) mod syntax;
pub(crate) mod timed_property_serialization;

#[doc(inline)]
pub use syntax::{
    CssAnimation, CssAnimationComponents, CssAnimationDirection, CssAnimationDirectionList,
    CssAnimationFillMode, CssAnimationFillModeList, CssAnimationIterationCount,
    CssAnimationIterationCountList, CssAnimationList, CssAnimationName, CssAnimationNameList,
    CssAnimationPlayState, CssAnimationPlayStateList, CssCubicBezier, CssCubicBezierX,
    CssDelayList, CssDurationList, CssEasing, CssEasingKeyword, CssEasingList, CssLinearEasing,
    CssLinearStop, CssStepPosition, CssSteps, CssTransition, CssTransitionList,
    CssTransitionProperty, CssTransitionPropertyList, CssTransitionPropertyName,
};
