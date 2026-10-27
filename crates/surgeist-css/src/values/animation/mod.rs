//! Animation, transition and easing values.
//!
//! # Transitions and animations
//!
//! These are authored syntax values. This crate does not multiply transform matrices,
//! interpolate or evaluate easing, render shadows or filters, resolve URLs, compute
//! shape geometry, perform layout or painting, or lower values into sibling crates.
//! Basic shapes and `clip-path` expose complete authored grammar support; this does
//! not establish contextual geometry or exact calculation projection. The selected
//! transition property family has intrinsic metadata and four ordered shorthand
//! contributions, preserving one list entry per authored item. [`CssLonghandValueRef`](crate::property::CssLonghandValueRef)
//! exposes property, duration, timing-function and delay lists, including explicit
//! child origins and programmatic omission defaults. The selected Animation family
//! likewise has intrinsic metadata and eight ordered contributions: duration,
//! timing-function, delay, iteration-count, direction, fill-mode, play-state and
//! name. Its borrowed variants retain the existing typed lists and one entry per
//! authored item, using schema initials for omissions. Both selected authored
//! families have Complete support metadata; contextual execution belongs downstream.
//! Checked whole-property construction and pending reentry reject original implicit
//! closures, while reused parser-produced typed children retain recovery origins.
//! Support for a typed function does not promote an unselected production.
//!
//! # Authored timing domains
//!
//! Time literals retain exact coefficients and authored units; durations are non-negative and
//! delays are signed. Well-typed calculations remain authored for later range processing.
//! Ordinary specified emission converts exactly to seconds before rounding text to at most six
//! fractional places, nearest with ties away from zero under the selected frozen WebKit policy.
//! Authored coefficients, units and origins remain exact. Finite calculation text rounds the
//! existing projected binary64 result with the same policy. Exact calculation arithmetic and
//! range processing remain unfinished.
//!
//! ```
//! use surgeist_css::property::CssKnownPropertyValueRef;
//! use surgeist_css::numeric::{CssNonNegative, CssNumeric, CssNumericLiteral, CssTime, CssTimeUnit};
//! use surgeist_css::CssParser;
//! let negative = CssNumericLiteral::<CssTime>::try_new("-1", CssTimeUnit::Seconds).unwrap();
//! assert!(CssNumeric::<CssTime, CssNonNegative>::try_from_literal(negative).is_err());
//! let report = CssParser::default().parse_block_contents("transition-duration: calc(-1s + 2s); transition-delay: -250ms");
//! assert!(report.is_clean());
//! let CssKnownPropertyValueRef::TransitionDuration(duration) = report.syntax()[0]
//!     .known().unwrap().property_value().unwrap() else { panic!("duration"); };
//! assert!(duration.value().values()[0].calculation().is_some());
//! let CssKnownPropertyValueRef::TransitionDelay(delay) = report.syntax()[1]
//!     .known().unwrap().property_value().unwrap() else { panic!("delay"); };
//! let delay = &delay.value().values()[0];
//! let literal = delay.literal().unwrap();
//! assert_eq!(literal.numeric().representation(), "-250");
//! assert_eq!(literal.unit(), CssTimeUnit::Milliseconds);
//! assert_eq!(delay.serialize_specified(Default::default()).unwrap(), "-0.25s");
//! ```
//!
//! This crate owns authored timing syntax only; timeline evaluation and
//! cross-crate lowering remain downstream responsibilities.

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
