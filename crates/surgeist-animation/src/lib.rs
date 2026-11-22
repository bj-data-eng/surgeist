//! CSS animation and transition timing contracts for Surgeist.
//!
//! This crate owns executable animation sampling contracts: typed elapsed time,
//! timing phases, easing, interpolation, sampled values, and diagnostics. Keep
//! authored CSS syntax in `surgeist-css`, live authored editing in CSSOM,
//! runtime clocks in `surgeist-runtime`, and cross-crate lowering in the root
//! `surgeist` facade.
//!
//! [`timeline`] accepts signed, nullable model coordinates and immutable
//! conversion snapshots. Document timelines use a caller-supplied frame timestamp
//! with a fixed origin offset; they never read a clock or own document lifecycle.
//! Animation play control and effect sampling consume this model separately.
//! [`timing::EffectTiming`] samples optional signed model local time and derives
//! effect phases, fill, iteration progress and relevance from supplied animation
//! observations. Normalized CSS elapsed tracks use the same timing kernel.
//!
//! Callers pass effective [`time::ElapsedTime`] values that already account for
//! the runtime clock and track start time. Paused sampling is explicit through
//! [`timing::AnimationPlayState::Paused`]; advancing wall-clock time alone does not make a
//! paused sample unstable. Root supplies normalized transition and keyframe
//! tracks, including computed endpoints, rectangular color spaces, and property
//! support policy.
//!
//! Each public type has one path, through the module that owns it.
//!
//! | Module | Owns |
//! | --- | --- |
//! | [`time`] | Typed elapsed time, delay and duration. |
//! | [`timeline`] | Model time flow, fixed-origin document timing and conversions. |
//! | [`progress`] | Normalized progress, unit ratios and easing input. |
//! | [`easing`] | Cubic Bezier, step and linear easing functions. |
//! | [`timing`] | Timing parameters, phases and the timing sample. |
//! | [`interpolation`] | Interpolable values, property keys and pairs. |
//! | [`computed`] | Canonical computed atoms, symbolic math and property combination. |
//! | [`transform`] | Resolved 2D functions, list addition and affine interpolation. |
//! | [`sample`] | Sampled property results and next-frame hints. |
//! | [`transition`] | Transition tracks, lifecycle decisions, and current values. |
//! | [`keyframe`] | Keyframe tracks, the entry point for keyframe animations. |
//! | [`keyframe_input`] | Native typed keyframe normalization and computed offsets. |
//! | [`diagnostics`] | Numeric diagnostics shared by the typed constructors. |
//!
//! Transition sampling starts from [`transition::TransitionTrack`]:
//!
//! ```
//! use surgeist_animation::easing::Easing;
//! use surgeist_animation::interpolation::{InterpolableNumber, InterpolableValue, PropertyKey};
//! use surgeist_animation::sample::{SampleClassification, SampleValue};
//! use surgeist_animation::time::{AnimationDelay, AnimationDuration, ElapsedTime};
//! use surgeist_animation::timing::AnimationPlayState;
//! use surgeist_animation::transition::{TransitionTrack, TransitionTrackId};
//!
//! let track = TransitionTrack::new(
//!     TransitionTrackId::new(1),
//!     PropertyKey::new("opacity").unwrap(),
//!     InterpolableValue::number(InterpolableNumber::new(0.0).unwrap()),
//!     InterpolableValue::number(InterpolableNumber::new(1.0).unwrap()),
//!     AnimationDuration::from_secs(2.0).unwrap(),
//!     AnimationDelay::from_secs(0.0).unwrap(),
//!     Easing::linear(),
//! )
//! .unwrap();
//!
//! let sample = track.sample(
//!     ElapsedTime::from_secs(1.0).unwrap(),
//!     AnimationPlayState::Running,
//! );
//! assert_eq!(sample.classification(), SampleClassification::Active);
//! let SampleValue::Value(value) = sample.value() else {
//!     panic!("expected sampled transition value");
//! };
//! let InterpolableValue::Number(opacity) = value.value() else {
//!     panic!("expected opacity number");
//! };
//! assert_eq!(opacity.value(), 0.5);
//! ```
//!
//! Keyframe sampling starts from [`keyframe::KeyframeTrack`] and root-supplied
//! normalized segments:
//!
//! ```
//! use surgeist_animation::easing::Easing;
//! use surgeist_animation::interpolation::{InterpolableNumber, InterpolableValue, PropertyKey};
//! use surgeist_animation::keyframe::{
//!     KeyframeAnimationId, KeyframeSegment, KeyframeTiming, KeyframeTrack,
//! };
//! use surgeist_animation::progress::UnitRatio;
//! use surgeist_animation::sample::{SampleClassification, SampleValue};
//! use surgeist_animation::time::{AnimationDelay, AnimationDuration, ElapsedTime};
//! use surgeist_animation::timing::{
//!     AnimationDirection, AnimationPlayState, FillMode, IterationCount,
//! };
//!
//! let timing = KeyframeTiming::try_new(
//!     AnimationDelay::from_secs(0.0).unwrap(),
//!     AnimationDuration::from_secs(2.0).unwrap(),
//!     IterationCount::finite(1.0).unwrap(),
//!     AnimationDirection::Normal,
//!     FillMode::Both,
//! )
//! .unwrap();
//! let segment = KeyframeSegment::new(
//!     PropertyKey::new("opacity").unwrap(),
//!     UnitRatio::new(0.0).unwrap(),
//!     UnitRatio::new(1.0).unwrap(),
//!     InterpolableValue::number(InterpolableNumber::new(0.0).unwrap()),
//!     InterpolableValue::number(InterpolableNumber::new(1.0).unwrap()),
//!     Easing::linear(),
//! )
//! .unwrap();
//! let track = KeyframeTrack::new(
//!     KeyframeAnimationId::new("fade").unwrap(),
//!     timing,
//!     vec![segment],
//! )
//! .unwrap();
//!
//! let samples = track.sample(
//!     ElapsedTime::from_secs(1.0).unwrap(),
//!     AnimationPlayState::Running,
//! );
//! assert_eq!(samples[0].classification(), SampleClassification::Active);
//! let SampleValue::Value(value) = samples[0].value() else {
//!     panic!("expected sampled keyframe value");
//! };
//! let InterpolableValue::Number(opacity) = value.value() else {
//!     panic!("expected opacity number");
//! };
//! assert_eq!(opacity.value(), 0.5);
//! ```

#![forbid(unsafe_code)]

pub mod computed;
pub mod diagnostics;
pub mod easing;
pub mod interpolation;
pub mod keyframe;
pub mod keyframe_input;
pub mod progress;
pub mod sample;
pub mod time;
pub mod timeline;
pub mod timing;
pub mod transform;
pub mod transition;

/// Crate identity string used by smoke tests and API artifacts.
pub const CRATE_NAME: &str = "surgeist-animation";
