//! Sampled property output, completion, and next-frame hints.
//!
//! Samples distinguish active values, fill values, pending/no-value states,
//! permanent unsupported interpolation, and recoverable sample-local arithmetic
//! failure. Hints are derived from timing phase plus play state.

use crate::{
    AnimationPlayState, InterpolableValue, InterpolationError, InterpolationOutcome,
    InterpolationPair, InterpolationProgress, PropertyKey, TimingPhase, TimingSample,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SampleClassification {
    Idle,
    Pending,
    Active,
    Filling,
    Finished,
    Failed,
    Unsupported,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CompletionStatus {
    NotStarted,
    Running,
    Finished,
    Unsupported,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NextFrameHint {
    MayChange,
    StableUntilExternalInput,
}

#[derive(Debug, Clone, PartialEq)]
pub struct SampledProperty {
    property: PropertyKey,
    value: InterpolableValue,
}

#[derive(Debug, Clone, PartialEq)]
pub enum SampleValue {
    Value(SampledProperty),
    Error(InterpolationError),
    Unsupported(InterpolationError),
    None,
}

#[derive(Debug, Clone, PartialEq)]
pub struct SampledPropertyResult {
    classification: SampleClassification,
    completion: CompletionStatus,
    next_frame: NextFrameHint,
    value: SampleValue,
}

impl SampledProperty {
    pub const fn new(property: PropertyKey, value: InterpolableValue) -> Self {
        Self { property, value }
    }

    pub const fn property(&self) -> &PropertyKey {
        &self.property
    }

    pub const fn value(&self) -> &InterpolableValue {
        &self.value
    }
}

impl SampledPropertyResult {
    #[inline]
    pub fn from_timing_and_pair(timing: TimingSample, pair: &InterpolationPair) -> Self {
        let Some(eased_progress) = timing.eased_progress() else {
            return Self::without_progress(timing);
        };

        let progress = match InterpolationProgress::from_eased(eased_progress) {
            Ok(progress) => progress,
            Err(error) => return Self::unsupported(error),
        };

        Self::from_pair_outcome(timing, pair.property(), Some(pair.sample(progress)))
    }

    // Pair sampling benefits from an outlined result boundary; keyframe loops
    // inline the same implementation while producing each property's output.
    #[inline(never)]
    fn from_pair_outcome(
        timing: TimingSample,
        property: &PropertyKey,
        outcome: Option<InterpolationOutcome>,
    ) -> Self {
        Self::from_timing_and_interpolation_outcome(timing, property, outcome)
    }

    #[inline]
    pub(crate) fn from_timing_and_interpolation_outcome(
        timing: TimingSample,
        property: &PropertyKey,
        outcome: Option<InterpolationOutcome>,
    ) -> Self {
        let Some(outcome) = outcome else {
            return Self::without_progress(timing);
        };

        match outcome {
            InterpolationOutcome::Value(value) => {
                Self::with_value(timing, SampledProperty::new(property.clone(), value))
            }
            InterpolationOutcome::ArithmeticFailure(result) => {
                Self::arithmetic_failure(timing, result)
            }
            InterpolationOutcome::Unsupported(error) => Self::unsupported(error),
        }
    }

    pub const fn classification(&self) -> SampleClassification {
        self.classification
    }

    pub const fn completion(&self) -> CompletionStatus {
        self.completion
    }

    pub const fn next_frame(&self) -> NextFrameHint {
        self.next_frame
    }

    pub const fn value(&self) -> &SampleValue {
        &self.value
    }

    const fn without_progress(timing: TimingSample) -> Self {
        match timing.phase() {
            TimingPhase::Idle => Self {
                classification: SampleClassification::Idle,
                completion: CompletionStatus::NotStarted,
                next_frame: NextFrameHint::StableUntilExternalInput,
                value: SampleValue::None,
            },
            TimingPhase::Before => Self {
                classification: SampleClassification::Pending,
                completion: CompletionStatus::NotStarted,
                next_frame: next_frame_for_timing(timing),
                value: SampleValue::None,
            },
            TimingPhase::Active => Self {
                classification: SampleClassification::Active,
                completion: CompletionStatus::Running,
                next_frame: next_frame_for_timing(timing),
                value: SampleValue::None,
            },
            TimingPhase::After => Self {
                classification: SampleClassification::Finished,
                completion: CompletionStatus::Finished,
                next_frame: NextFrameHint::StableUntilExternalInput,
                value: SampleValue::None,
            },
        }
    }

    fn with_value(timing: TimingSample, value: SampledProperty) -> Self {
        match timing.phase() {
            TimingPhase::Idle => Self::without_progress(timing),
            TimingPhase::Before => Self {
                classification: SampleClassification::Filling,
                completion: CompletionStatus::NotStarted,
                next_frame: next_frame_for_timing(timing),
                value: SampleValue::Value(value),
            },
            TimingPhase::Active => Self {
                classification: SampleClassification::Active,
                completion: CompletionStatus::Running,
                next_frame: next_frame_for_timing(timing),
                value: SampleValue::Value(value),
            },
            TimingPhase::After => Self {
                classification: SampleClassification::Filling,
                completion: CompletionStatus::Finished,
                next_frame: NextFrameHint::StableUntilExternalInput,
                value: SampleValue::Value(value),
            },
        }
    }

    fn arithmetic_failure(
        timing: TimingSample,
        result: crate::NonFiniteInterpolationResult,
    ) -> Self {
        Self {
            classification: SampleClassification::Failed,
            completion: failed_completion(timing.phase()),
            next_frame: next_frame_for_timing(timing),
            value: SampleValue::Error(InterpolationError::NonFiniteInterpolationResult(result)),
        }
    }

    const fn unsupported(error: InterpolationError) -> Self {
        Self {
            classification: SampleClassification::Unsupported,
            completion: CompletionStatus::Unsupported,
            next_frame: NextFrameHint::StableUntilExternalInput,
            value: SampleValue::Unsupported(error),
        }
    }
}

const fn failed_completion(phase: TimingPhase) -> CompletionStatus {
    match phase {
        TimingPhase::Idle | TimingPhase::Before => CompletionStatus::NotStarted,
        TimingPhase::Active => CompletionStatus::Running,
        TimingPhase::After => CompletionStatus::Finished,
    }
}

const fn next_frame_for_timing(timing: TimingSample) -> NextFrameHint {
    match (timing.phase(), timing.play_state()) {
        (TimingPhase::Before | TimingPhase::Active, AnimationPlayState::Running) => {
            NextFrameHint::MayChange
        }
        _ => NextFrameHint::StableUntilExternalInput,
    }
}

#[cfg(test)]
mod tests {
    use super::{
        CompletionStatus, NextFrameHint, SampleClassification, SampleValue, SampledPropertyResult,
    };
    use crate::{
        AnimationDelay, AnimationDirection, AnimationDuration, AnimationPlayState, DiscreteValue,
        Easing, ElapsedTime, FillMode, InterpolableNumber, InterpolableValue, InterpolationError,
        InterpolationOutcome, InterpolationPair, IterationCount, LinearControlPoint, LinearEasing,
        PropertyKey, TimingParameters, TimingSample, TransformValue, ValueFamily,
    };

    #[test]
    fn idle_sample_classifies_without_value_or_next_frame() {
        let result = SampledPropertyResult::from_timing_and_pair(
            TimingSample::idle(AnimationPlayState::Running),
            &number_pair(),
        );

        assert_eq!(result.classification(), SampleClassification::Idle);
        assert_eq!(result.completion(), CompletionStatus::NotStarted);
        assert_eq!(result.next_frame(), NextFrameHint::StableUntilExternalInput);
        assert_eq!(result.value(), &SampleValue::None);
    }

    #[test]
    fn before_without_fill_is_pending() {
        let timing = timing(FillMode::None).sample(
            ElapsedTime::from_secs(0.5).unwrap(),
            AnimationPlayState::Running,
        );
        let result = SampledPropertyResult::from_timing_and_pair(timing, &number_pair());

        assert_eq!(result.classification(), SampleClassification::Pending);
        assert_eq!(result.completion(), CompletionStatus::NotStarted);
        assert_eq!(result.value(), &SampleValue::None);
    }

    #[test]
    fn running_pending_before_sample_next_frame_may_change() {
        let timing = timing(FillMode::None).sample(
            ElapsedTime::from_secs(0.5).unwrap(),
            AnimationPlayState::Running,
        );
        let result = SampledPropertyResult::from_timing_and_pair(timing, &number_pair());

        assert_eq!(result.classification(), SampleClassification::Pending);
        assert_eq!(result.completion(), CompletionStatus::NotStarted);
        assert_eq!(result.next_frame(), NextFrameHint::MayChange);
        assert_eq!(result.value(), &SampleValue::None);
    }

    #[test]
    fn paused_pending_before_sample_next_frame_is_stable() {
        let timing = timing(FillMode::None).sample(
            ElapsedTime::from_secs(0.5).unwrap(),
            AnimationPlayState::Paused,
        );
        let result = SampledPropertyResult::from_timing_and_pair(timing, &number_pair());

        assert_eq!(result.classification(), SampleClassification::Pending);
        assert_eq!(result.completion(), CompletionStatus::NotStarted);
        assert_eq!(result.next_frame(), NextFrameHint::StableUntilExternalInput);
        assert_eq!(result.value(), &SampleValue::None);
    }

    #[test]
    fn running_backwards_fill_sample_next_frame_may_change() {
        let timing = timing(FillMode::Backwards).sample(
            ElapsedTime::from_secs(0.5).unwrap(),
            AnimationPlayState::Running,
        );
        let result = SampledPropertyResult::from_timing_and_pair(timing, &number_pair());

        assert_eq!(result.classification(), SampleClassification::Filling);
        assert_eq!(result.completion(), CompletionStatus::NotStarted);
        assert_eq!(result.next_frame(), NextFrameHint::MayChange);
        assert!(matches!(result.value(), SampleValue::Value(_)));
    }

    #[test]
    fn paused_backwards_fill_sample_next_frame_is_stable() {
        let timing = timing(FillMode::Backwards).sample(
            ElapsedTime::from_secs(0.5).unwrap(),
            AnimationPlayState::Paused,
        );
        let result = SampledPropertyResult::from_timing_and_pair(timing, &number_pair());

        assert_eq!(result.classification(), SampleClassification::Filling);
        assert_eq!(result.completion(), CompletionStatus::NotStarted);
        assert_eq!(result.next_frame(), NextFrameHint::StableUntilExternalInput);
        assert!(matches!(result.value(), SampleValue::Value(_)));
    }

    #[test]
    fn active_running_sample_produces_value_and_may_change() {
        let timing = timing(FillMode::None).sample(
            ElapsedTime::from_secs(1.5).unwrap(),
            AnimationPlayState::Running,
        );
        let result = SampledPropertyResult::from_timing_and_pair(timing, &number_pair());

        assert_eq!(result.classification(), SampleClassification::Active);
        assert_eq!(result.completion(), CompletionStatus::Running);
        assert_eq!(result.next_frame(), NextFrameHint::MayChange);
        assert!(matches!(result.value(), SampleValue::Value(_)));
    }

    #[test]
    fn paused_active_sample_is_stable_until_external_input() {
        let timing = timing(FillMode::None).sample(
            ElapsedTime::from_secs(1.5).unwrap(),
            AnimationPlayState::Paused,
        );
        let result = SampledPropertyResult::from_timing_and_pair(timing, &number_pair());

        assert_eq!(result.classification(), SampleClassification::Active);
        assert_eq!(result.next_frame(), NextFrameHint::StableUntilExternalInput);
    }

    #[test]
    fn positive_duration_infinite_running_sample_may_change() {
        let timing = TimingParameters::try_new(
            AnimationDelay::from_secs(0.0).unwrap(),
            AnimationDuration::from_secs(1.0).unwrap(),
            IterationCount::infinite(),
            AnimationDirection::Normal,
            FillMode::None,
            Easing::linear(),
        )
        .unwrap()
        .sample(
            ElapsedTime::from_secs(10.25).unwrap(),
            AnimationPlayState::Running,
        );
        let result = SampledPropertyResult::from_timing_and_pair(timing, &number_pair());

        assert_eq!(result.classification(), SampleClassification::Active);
        assert_eq!(result.completion(), CompletionStatus::Running);
        assert_eq!(result.next_frame(), NextFrameHint::MayChange);
        assert!(matches!(result.value(), SampleValue::Value(_)));
    }

    #[test]
    fn before_and_after_fill_classify_as_filling() {
        let before = timing(FillMode::Both).sample(
            ElapsedTime::from_secs(0.5).unwrap(),
            AnimationPlayState::Running,
        );
        let after = timing(FillMode::Both).sample(
            ElapsedTime::from_secs(2.0).unwrap(),
            AnimationPlayState::Running,
        );

        assert_eq!(
            SampledPropertyResult::from_timing_and_pair(before, &number_pair()).classification(),
            SampleClassification::Filling
        );
        assert_eq!(
            SampledPropertyResult::from_timing_and_pair(after, &number_pair()).classification(),
            SampleClassification::Filling
        );
    }

    #[test]
    fn after_without_fill_is_finished_without_value() {
        let timing = timing(FillMode::None).sample(
            ElapsedTime::from_secs(2.0).unwrap(),
            AnimationPlayState::Running,
        );
        let result = SampledPropertyResult::from_timing_and_pair(timing, &number_pair());

        assert_eq!(result.classification(), SampleClassification::Finished);
        assert_eq!(result.completion(), CompletionStatus::Finished);
        assert_eq!(result.value(), &SampleValue::None);
    }

    #[test]
    fn unsupported_interpolation_classifies_as_unsupported() {
        let timing = timing(FillMode::None).sample(
            ElapsedTime::from_secs(1.5).unwrap(),
            AnimationPlayState::Running,
        );
        let result = SampledPropertyResult::from_timing_and_pair(timing, &transform_pair());

        assert_eq!(result.classification(), SampleClassification::Unsupported);
        assert_eq!(result.completion(), CompletionStatus::Unsupported);
        assert_eq!(result.next_frame(), NextFrameHint::StableUntilExternalInput);
        assert!(matches!(result.value(), SampleValue::Unsupported(_)));
    }

    #[test]
    fn running_before_interpolation_arithmetic_failure_is_failed_not_started_and_may_change() {
        let timing = TimingParameters::try_new(
            AnimationDelay::from_secs(1.0).unwrap(),
            AnimationDuration::from_secs(1.0).unwrap(),
            IterationCount::finite(1.0).unwrap(),
            AnimationDirection::Normal,
            FillMode::Backwards,
            half_progress_easing(),
        )
        .unwrap()
        .sample(
            ElapsedTime::from_secs(0.5).unwrap(),
            AnimationPlayState::Running,
        );
        let result = SampledPropertyResult::from_timing_and_pair(timing, &overflow_pair());

        assert_eq!(result.classification(), SampleClassification::Failed);
        assert_eq!(result.completion(), CompletionStatus::NotStarted);
        assert_eq!(result.next_frame(), NextFrameHint::MayChange);
        assert_arithmetic_error(result.value(), "opacity", ValueFamily::Number);
    }

    #[test]
    fn running_active_interpolation_arithmetic_failure_is_failed_running_and_may_change() {
        let timing = timing(FillMode::None).sample(
            ElapsedTime::from_secs(1.5).unwrap(),
            AnimationPlayState::Running,
        );
        let result = SampledPropertyResult::from_timing_and_pair(timing, &overflow_pair());

        assert_eq!(result.classification(), SampleClassification::Failed);
        assert_eq!(result.completion(), CompletionStatus::Running);
        assert_eq!(result.next_frame(), NextFrameHint::MayChange);
        assert_arithmetic_error(result.value(), "opacity", ValueFamily::Number);
    }

    #[test]
    fn paused_interpolation_arithmetic_failure_is_running_but_stable() {
        let timing = timing(FillMode::None).sample(
            ElapsedTime::from_secs(1.5).unwrap(),
            AnimationPlayState::Paused,
        );
        let result = SampledPropertyResult::from_timing_and_pair(timing, &overflow_pair());

        assert_eq!(result.classification(), SampleClassification::Failed);
        assert_eq!(result.completion(), CompletionStatus::Running);
        assert_eq!(result.next_frame(), NextFrameHint::StableUntilExternalInput);
        assert_arithmetic_error(result.value(), "opacity", ValueFamily::Number);
    }

    #[test]
    fn paused_arithmetic_failure_next_frame_is_stable_without_becoming_unsupported() {
        let timing = TimingParameters::try_new(
            AnimationDelay::from_secs(1.0).unwrap(),
            AnimationDuration::from_secs(1.0).unwrap(),
            IterationCount::finite(1.0).unwrap(),
            AnimationDirection::Normal,
            FillMode::Backwards,
            half_progress_easing(),
        )
        .unwrap()
        .sample(
            ElapsedTime::from_secs(0.5).unwrap(),
            AnimationPlayState::Paused,
        );
        let result = SampledPropertyResult::from_timing_and_pair(timing, &overflow_pair());

        assert_eq!(result.classification(), SampleClassification::Failed);
        assert_eq!(result.completion(), CompletionStatus::NotStarted);
        assert_eq!(result.next_frame(), NextFrameHint::StableUntilExternalInput);
        assert_arithmetic_error(result.value(), "opacity", ValueFamily::Number);
    }

    #[test]
    fn after_phase_interpolation_arithmetic_failure_is_finished_and_stable() {
        let timing = TimingParameters::try_new(
            AnimationDelay::from_secs(0.0).unwrap(),
            AnimationDuration::from_secs(1.0).unwrap(),
            IterationCount::finite(2.5).unwrap(),
            AnimationDirection::Normal,
            FillMode::Forwards,
            Easing::linear(),
        )
        .unwrap()
        .sample(
            ElapsedTime::from_secs(2.5).unwrap(),
            AnimationPlayState::Running,
        );
        let result = SampledPropertyResult::from_timing_and_pair(timing, &overflow_pair());

        assert_eq!(result.classification(), SampleClassification::Failed);
        assert_eq!(result.completion(), CompletionStatus::Finished);
        assert_eq!(result.next_frame(), NextFrameHint::StableUntilExternalInput);
        assert_arithmetic_error(result.value(), "opacity", ValueFamily::Number);
    }

    #[test]
    fn finished_and_after_fill_samples_next_frame_remain_stable() {
        let finished_timing = timing(FillMode::None).sample(
            ElapsedTime::from_secs(2.0).unwrap(),
            AnimationPlayState::Running,
        );
        let after_fill_timing = timing(FillMode::Forwards).sample(
            ElapsedTime::from_secs(2.0).unwrap(),
            AnimationPlayState::Running,
        );

        let finished = SampledPropertyResult::from_timing_and_pair(finished_timing, &number_pair());
        let after_fill =
            SampledPropertyResult::from_timing_and_pair(after_fill_timing, &number_pair());

        assert_eq!(finished.classification(), SampleClassification::Finished);
        assert_eq!(finished.completion(), CompletionStatus::Finished);
        assert_eq!(
            finished.next_frame(),
            NextFrameHint::StableUntilExternalInput
        );
        assert_eq!(finished.value(), &SampleValue::None);
        assert_eq!(after_fill.classification(), SampleClassification::Filling);
        assert_eq!(after_fill.completion(), CompletionStatus::Finished);
        assert_eq!(
            after_fill.next_frame(),
            NextFrameHint::StableUntilExternalInput
        );
        assert!(matches!(after_fill.value(), SampleValue::Value(_)));
    }

    #[test]
    fn precomputed_active_value_uses_timing_classification() {
        let timing = timing(FillMode::None).sample(
            ElapsedTime::from_secs(1.5).unwrap(),
            AnimationPlayState::Running,
        );
        let result = SampledPropertyResult::from_timing_and_interpolation_outcome(
            timing,
            &PropertyKey::new("opacity").unwrap(),
            Some(InterpolationOutcome::Value(InterpolableValue::number(
                InterpolableNumber::new(0.5).unwrap(),
            ))),
        );

        assert_eq!(result.classification(), SampleClassification::Active);
        assert_eq!(result.next_frame(), NextFrameHint::MayChange);
        assert!(matches!(result.value(), SampleValue::Value(_)));
    }

    #[test]
    fn precomputed_none_uses_no_progress_timing_classification() {
        let timing = timing(FillMode::None).sample(
            ElapsedTime::from_secs(0.5).unwrap(),
            AnimationPlayState::Running,
        );
        let result = SampledPropertyResult::from_timing_and_interpolation_outcome(
            timing,
            &PropertyKey::new("opacity").unwrap(),
            None,
        );

        assert_eq!(result.classification(), SampleClassification::Pending);
        assert_eq!(result.value(), &SampleValue::None);
    }

    #[test]
    fn precomputed_unsupported_outcome_preserves_error_classification() {
        let timing = timing(FillMode::None).sample(
            ElapsedTime::from_secs(1.5).unwrap(),
            AnimationPlayState::Running,
        );
        let error = InterpolationError::UnsupportedFamily {
            property: PropertyKey::new("filter").unwrap(),
            family: ValueFamily::Discrete,
            from: InterpolableValue::discrete(DiscreteValue::new("a").unwrap()),
            to: InterpolableValue::discrete(DiscreteValue::new("b").unwrap()),
            reason: "property support policy rejected interpolation",
        };
        let result = SampledPropertyResult::from_timing_and_interpolation_outcome(
            timing,
            &PropertyKey::new("filter").unwrap(),
            Some(InterpolationOutcome::Unsupported(error)),
        );

        assert_eq!(result.classification(), SampleClassification::Unsupported);
        assert_eq!(result.completion(), CompletionStatus::Unsupported);
        assert!(matches!(result.value(), SampleValue::Unsupported(_)));
    }

    fn timing(fill_mode: FillMode) -> TimingParameters {
        TimingParameters::try_new(
            AnimationDelay::from_secs(1.0).unwrap(),
            AnimationDuration::from_secs(1.0).unwrap(),
            IterationCount::finite(1.0).unwrap(),
            AnimationDirection::Normal,
            fill_mode,
            Easing::linear(),
        )
        .unwrap()
    }

    fn number_pair() -> InterpolationPair {
        InterpolationPair::new(
            PropertyKey::new("opacity").unwrap(),
            InterpolableValue::number(InterpolableNumber::new(0.0).unwrap()),
            InterpolableValue::number(InterpolableNumber::new(1.0).unwrap()),
        )
        .unwrap()
    }

    fn transform_pair() -> InterpolationPair {
        InterpolationPair::new(
            PropertyKey::new("transform").unwrap(),
            InterpolableValue::transform(TransformValue::unsupported("translate")),
            InterpolableValue::transform(TransformValue::unsupported("scale")),
        )
        .unwrap()
    }

    fn overflow_pair() -> InterpolationPair {
        InterpolationPair::new(
            PropertyKey::new("opacity").unwrap(),
            InterpolableValue::number(InterpolableNumber::new(f64::MAX).unwrap()),
            InterpolableValue::number(InterpolableNumber::new(-f64::MAX).unwrap()),
        )
        .unwrap()
    }

    fn half_progress_easing() -> Easing {
        Easing::linear_function(
            LinearEasing::new(vec![
                LinearControlPoint::new(0.0, 0.5),
                LinearControlPoint::new(1.0, 0.5),
            ])
            .unwrap(),
        )
    }

    fn assert_arithmetic_error(value: &SampleValue, property: &str, family: ValueFamily) {
        let SampleValue::Error(InterpolationError::NonFiniteInterpolationResult(result)) = value
        else {
            panic!("expected arithmetic error, got {value:?}");
        };

        assert_eq!(result.property().as_str(), property);
        assert_eq!(result.family(), family);
        assert_eq!(result.from().family(), family);
        assert_eq!(result.to().family(), family);
        assert!(!result.value().is_finite());
    }
}
