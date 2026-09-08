//! CSS animation and transition timing contracts for Surgeist.
//!
//! This crate owns executable animation sampling contracts: typed elapsed time,
//! timing phases, easing, interpolation, sampled values, and diagnostics. Keep
//! authored CSS declarations in `surgeist-style`, runtime clocks in
//! `surgeist-runtime`, and cross-crate lowering in the root `surgeist` facade.
//!
//! Callers pass effective [`ElapsedTime`] values that already account for the
//! runtime clock and track start time. Paused sampling is explicit through
//! [`AnimationPlayState::Paused`]; advancing wall-clock time alone does not make a
//! paused sample unstable. Root supplies normalized transition and keyframe
//! tracks, including computed endpoints, rectangular color spaces, and property
//! support policy.
//!
//! Transition sampling uses the public front door:
//!
//! ```
//! use surgeist_animation::{
//!     AnimationDelay, AnimationDuration, AnimationPlayState, Easing, ElapsedTime,
//!     InterpolableNumber, InterpolableValue, PropertyKey, SampleClassification, SampleValue,
//!     TransitionTrack, TransitionTrackId,
//! };
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
//! Keyframe sampling also uses root-supplied normalized segments and public
//! reexports only:
//!
//! ```
//! use surgeist_animation::{
//!     AnimationDelay, AnimationDirection, AnimationDuration, AnimationPlayState, Easing,
//!     ElapsedTime, FillMode, InterpolableNumber, InterpolableValue, IterationCount,
//!     KeyframeAnimationId, KeyframeSegment, KeyframeTiming, KeyframeTrack, PropertyKey,
//!     SampleClassification, SampleValue, UnitRatio,
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

pub mod diagnostics;
pub mod easing;
pub mod interpolation;
pub mod keyframe;
pub mod progress;
pub mod sample;
pub mod time;
pub mod timing;
pub mod transition;

pub use diagnostics::{NumericError, NumericErrorKind, NumericInput};
pub use easing::{
    CubicBezier, EasedProgress, Easing, EasingError, EasingKeyword, LinearControlPoint,
    LinearControlPointComponent, LinearEasing, StepPosition, Steps,
};
pub use interpolation::{
    ColorInterpolationSpace, CompositeValue, DiscreteValue, InterpolableColor, InterpolableNumber,
    InterpolablePercentage, InterpolableValue, InterpolationComponent, InterpolationError,
    InterpolationOutcome, InterpolationPair, InterpolationProgress, NonFiniteInterpolationResult,
    PropertyKey, TransformValue, ValueFamily,
};
pub use keyframe::{
    KeyframeAnimationId, KeyframeError, KeyframeSegment, KeyframeTiming, KeyframeTrack,
};
pub use progress::{EasingInput, NormalizedProgress, UnitRatio};
pub use sample::{
    CompletionStatus, NextFrameHint, SampleClassification, SampleValue, SampledProperty,
    SampledPropertyResult,
};
pub use time::{AnimationDelay, AnimationDuration, ElapsedTime};
pub use timing::{
    AnimationDirection, AnimationPlayState, CurrentIteration, FillMode, IterationCount,
    PhaseProgress, TimingError, TimingParameters, TimingPhase, TimingSample,
};
pub use transition::{TransitionError, TransitionTrack, TransitionTrackId};

/// Crate identity string used by smoke tests and API artifacts.
pub const CRATE_NAME: &str = "surgeist-animation";
