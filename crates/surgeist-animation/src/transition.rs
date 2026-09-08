//! Normalized transition tracks.
//!
//! A transition track contains root-supplied endpoints, one validated duration,
//! one delay, and one easing function. Sampling uses effective elapsed time and
//! play state; this crate does not generate transitions from authored CSS.

use core::fmt;

use crate::{
    AnimationDelay, AnimationDuration, AnimationPlayState, Easing, ElapsedTime, InterpolableValue,
    InterpolationError, InterpolationPair, PropertyKey, SampledPropertyResult, TimingParameters,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct TransitionTrackId {
    value: u64,
}

#[derive(Debug, Clone, PartialEq)]
pub struct TransitionTrack {
    id: TransitionTrackId,
    pair: InterpolationPair,
    timing: TimingParameters,
}

#[derive(Debug, Clone, PartialEq)]
pub enum TransitionError {
    EquivalentEndpoints {
        property: PropertyKey,
        value: InterpolableValue,
    },
    Interpolation(Box<InterpolationError>),
}

impl fmt::Display for TransitionError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::EquivalentEndpoints { property, .. } => write!(
                f,
                "transition property {} endpoints must differ",
                property.as_str()
            ),
            Self::Interpolation(_) => f.write_str("transition interpolation failed"),
        }
    }
}

impl std::error::Error for TransitionError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::Interpolation(error) => Some(error.as_ref()),
            _ => None,
        }
    }
}

impl TransitionTrackId {
    pub const fn new(value: u64) -> Self {
        Self { value }
    }

    pub const fn value(self) -> u64 {
        self.value
    }
}

impl TransitionTrack {
    pub fn new(
        id: TransitionTrackId,
        property: PropertyKey,
        from: InterpolableValue,
        to: InterpolableValue,
        duration: AnimationDuration,
        delay: AnimationDelay,
        easing: Easing,
    ) -> Result<Self, TransitionError> {
        if from == to {
            return Err(TransitionError::EquivalentEndpoints {
                property,
                value: from,
            });
        }

        Ok(Self {
            id,
            pair: InterpolationPair::new(property, from, to)
                .map_err(|error| TransitionError::Interpolation(Box::new(error)))?,
            timing: TimingParameters::for_transition(delay, duration, easing),
        })
    }

    pub const fn id(&self) -> TransitionTrackId {
        self.id
    }

    pub fn property(&self) -> &PropertyKey {
        self.pair.property()
    }

    pub fn from(&self) -> &InterpolableValue {
        self.pair.from()
    }

    pub fn to(&self) -> &InterpolableValue {
        self.pair.to()
    }

    pub const fn duration(&self) -> AnimationDuration {
        self.timing.duration()
    }

    pub const fn delay(&self) -> AnimationDelay {
        self.timing.delay()
    }

    pub const fn easing(&self) -> &Easing {
        self.timing.easing()
    }

    pub fn sample(
        &self,
        elapsed: ElapsedTime,
        play_state: AnimationPlayState,
    ) -> SampledPropertyResult {
        SampledPropertyResult::from_timing_and_pair(
            self.timing.sample(elapsed, play_state),
            &self.pair,
        )
    }
}

#[cfg(test)]
mod tests {
    use super::{TransitionError, TransitionTrack, TransitionTrackId};
    use crate::{
        AnimationDelay, AnimationDuration, AnimationPlayState, CompletionStatus, Easing,
        ElapsedTime, InterpolableNumber, InterpolablePercentage, InterpolableValue,
        InterpolationError, LinearControlPoint, LinearEasing, NextFrameHint, PropertyKey,
        SampleClassification, SampleValue, SampledPropertyResult, TransformValue, ValueFamily,
    };

    #[test]
    fn transition_track_preserves_identity_property_values_and_timing_inputs() {
        let track = opacity_track(
            InterpolableNumber::new(0.0).unwrap(),
            InterpolableNumber::new(1.0).unwrap(),
            AnimationDuration::from_secs(2.0).unwrap(),
            AnimationDelay::from_secs(-0.25).unwrap(),
            Easing::keyword(crate::EasingKeyword::EaseIn),
        );

        assert_eq!(track.id(), TransitionTrackId::new(42));
        assert_eq!(track.property().as_str(), "opacity");
        assert_eq!(track.duration().as_secs(), 2.0);
        assert_eq!(track.delay().as_secs(), -0.25);
        assert!(track.easing().as_cubic_bezier().is_some());
        assert_eq!(track.from().family(), crate::ValueFamily::Number);
        assert_eq!(track.to().family(), crate::ValueFamily::Number);
    }

    #[test]
    fn transition_max_duration_uses_infallible_single_iteration_timing() {
        let track = opacity_track(
            InterpolableNumber::new(0.0).unwrap(),
            InterpolableNumber::new(1.0).unwrap(),
            AnimationDuration::from_secs(f64::MAX).unwrap(),
            AnimationDelay::from_secs(0.0).unwrap(),
            Easing::linear(),
        );

        let sample = track.sample(
            ElapsedTime::from_secs(f64::MAX / 2.0).unwrap(),
            AnimationPlayState::Running,
        );

        assert_eq!(sample.classification(), SampleClassification::Active);
        assert_sampled_number(&sample, "opacity", 0.5);
    }

    #[test]
    fn equivalent_transition_endpoints_are_rejected_as_noops() {
        let value = InterpolableValue::number(InterpolableNumber::new(1.0).unwrap());
        let error = TransitionTrack::new(
            TransitionTrackId::new(1),
            PropertyKey::new("opacity").unwrap(),
            value.clone(),
            value.clone(),
            AnimationDuration::from_secs(1.0).unwrap(),
            AnimationDelay::from_secs(0.0).unwrap(),
            Easing::linear(),
        )
        .unwrap_err();

        assert_eq!(
            error,
            TransitionError::EquivalentEndpoints {
                property: PropertyKey::new("opacity").unwrap(),
                value,
            }
        );
    }

    #[test]
    fn mismatched_transition_value_families_return_interpolation_error() {
        let error = TransitionTrack::new(
            TransitionTrackId::new(1),
            PropertyKey::new("opacity").unwrap(),
            InterpolableValue::number(InterpolableNumber::new(0.0).unwrap()),
            InterpolableValue::percentage(InterpolablePercentage::new(1.0).unwrap()),
            AnimationDuration::from_secs(1.0).unwrap(),
            AnimationDelay::from_secs(0.0).unwrap(),
            Easing::linear(),
        )
        .unwrap_err();

        assert!(matches!(
            error,
            TransitionError::Interpolation(error)
                if matches!(*error, InterpolationError::MismatchedFamilies { .. })
        ));
    }

    #[test]
    fn wrapped_transition_diagnostics_expose_interpolation_source() {
        let error = TransitionTrack::new(
            TransitionTrackId::new(1),
            PropertyKey::new("opacity").unwrap(),
            InterpolableValue::number(InterpolableNumber::new(0.0).unwrap()),
            InterpolableValue::percentage(InterpolablePercentage::new(1.0).unwrap()),
            AnimationDuration::from_secs(1.0).unwrap(),
            AnimationDelay::from_secs(0.0).unwrap(),
            Easing::linear(),
        )
        .unwrap_err();

        assert_eq!(error.to_string(), "transition interpolation failed");
        assert_eq!(
            std::error::Error::source(&error).unwrap().to_string(),
            "property opacity cannot interpolate number to percentage"
        );
    }

    #[test]
    fn positive_delay_samples_start_value_before_active_interval() {
        let track = opacity_track(
            InterpolableNumber::new(0.0).unwrap(),
            InterpolableNumber::new(1.0).unwrap(),
            AnimationDuration::from_secs(1.0).unwrap(),
            AnimationDelay::from_secs(0.5).unwrap(),
            Easing::linear(),
        );

        let sample = track.sample(
            ElapsedTime::from_secs(0.25).unwrap(),
            AnimationPlayState::Running,
        );

        assert_eq!(sample.classification(), SampleClassification::Filling);
        assert_sampled_number(&sample, "opacity", 0.0);
    }

    #[test]
    fn delayed_transition_running_sample_next_frame_may_change() {
        let track = opacity_track(
            InterpolableNumber::new(0.0).unwrap(),
            InterpolableNumber::new(1.0).unwrap(),
            AnimationDuration::from_secs(1.0).unwrap(),
            AnimationDelay::from_secs(0.5).unwrap(),
            Easing::linear(),
        );

        let sample = track.sample(
            ElapsedTime::from_secs(0.25).unwrap(),
            AnimationPlayState::Running,
        );

        assert_eq!(sample.classification(), SampleClassification::Filling);
        assert_eq!(sample.completion(), CompletionStatus::NotStarted);
        assert_eq!(sample.next_frame(), NextFrameHint::MayChange);
        assert_sampled_number(&sample, "opacity", 0.0);
    }

    #[test]
    fn active_transition_samples_interpolated_value() {
        let track = opacity_track(
            InterpolableNumber::new(0.0).unwrap(),
            InterpolableNumber::new(10.0).unwrap(),
            AnimationDuration::from_secs(2.0).unwrap(),
            AnimationDelay::from_secs(0.0).unwrap(),
            Easing::linear(),
        );

        let sample = track.sample(
            ElapsedTime::from_secs(0.5).unwrap(),
            AnimationPlayState::Running,
        );

        assert_eq!(sample.classification(), SampleClassification::Active);
        assert_eq!(sample.completion(), CompletionStatus::Running);
        assert_eq!(sample.next_frame(), NextFrameHint::MayChange);
        assert_sampled_number(&sample, "opacity", 2.5);
    }

    #[test]
    fn active_transition_arithmetic_failure_is_failed_running_and_may_change() {
        let track = opacity_track(
            InterpolableNumber::new(f64::MAX).unwrap(),
            InterpolableNumber::new(-f64::MAX).unwrap(),
            AnimationDuration::from_secs(1.0).unwrap(),
            AnimationDelay::from_secs(0.0).unwrap(),
            Easing::linear(),
        );

        let sample = track.sample(
            ElapsedTime::from_secs(0.5).unwrap(),
            AnimationPlayState::Running,
        );

        assert_eq!(sample.classification(), SampleClassification::Failed);
        assert_eq!(sample.completion(), CompletionStatus::Running);
        assert_eq!(sample.next_frame(), NextFrameHint::MayChange);
        assert_arithmetic_error(&sample, "opacity", ValueFamily::Number);
    }

    #[test]
    fn delayed_running_arithmetic_failure_next_frame_may_change_and_later_recovers() {
        let track = opacity_track(
            InterpolableNumber::new(f64::MAX).unwrap(),
            InterpolableNumber::new(-f64::MAX).unwrap(),
            AnimationDuration::from_secs(1.0).unwrap(),
            AnimationDelay::from_secs(1.0).unwrap(),
            before_fill_failure_then_start_endpoint_easing(),
        );

        let delayed = track.sample(
            ElapsedTime::from_secs(0.5).unwrap(),
            AnimationPlayState::Running,
        );
        let recovered = track.sample(
            ElapsedTime::from_secs(1.0).unwrap(),
            AnimationPlayState::Running,
        );

        assert_eq!(delayed.classification(), SampleClassification::Failed);
        assert_eq!(delayed.completion(), CompletionStatus::NotStarted);
        assert_eq!(delayed.next_frame(), NextFrameHint::MayChange);
        assert_arithmetic_error(&delayed, "opacity", ValueFamily::Number);
        assert_eq!(recovered.classification(), SampleClassification::Active);
        assert_eq!(recovered.completion(), CompletionStatus::Running);
        assert_eq!(recovered.next_frame(), NextFrameHint::MayChange);
        assert_sampled_number(&recovered, "opacity", f64::MAX);
    }

    #[test]
    fn transition_percentage_sample_uses_public_front_door() {
        let track = percentage_track(
            InterpolablePercentage::new(-0.5).unwrap(),
            InterpolablePercentage::new(1.5).unwrap(),
            AnimationDuration::from_secs(2.0).unwrap(),
            AnimationDelay::from_secs(0.0).unwrap(),
            Easing::linear(),
        );

        let sample = track.sample(
            ElapsedTime::from_secs(1.0).unwrap(),
            AnimationPlayState::Running,
        );

        assert_eq!(sample.classification(), SampleClassification::Active);
        assert_sampled_percentage(&sample, "width", 0.5);
    }

    #[test]
    fn transition_timing_function_controls_sampled_progress() {
        let track = opacity_track(
            InterpolableNumber::new(0.0).unwrap(),
            InterpolableNumber::new(1.0).unwrap(),
            AnimationDuration::from_secs(1.0).unwrap(),
            AnimationDelay::from_secs(0.0).unwrap(),
            Easing::step_start(),
        );

        let sample = track.sample(
            ElapsedTime::from_secs(0.25).unwrap(),
            AnimationPlayState::Running,
        );

        assert_sampled_number(&sample, "opacity", 1.0);
    }

    #[test]
    fn transition_samples_owned_linear_function_through_public_front_door() {
        let linear = LinearEasing::new(vec![
            LinearControlPoint::new(0.0, 0.0),
            LinearControlPoint::new(1.0, 0.5),
        ])
        .unwrap();
        let track = opacity_track(
            InterpolableNumber::new(0.0).unwrap(),
            InterpolableNumber::new(1.0).unwrap(),
            AnimationDuration::from_secs(1.0).unwrap(),
            AnimationDelay::from_secs(0.0).unwrap(),
            Easing::linear_function(linear),
        );

        assert!(track.easing().as_linear_function().is_some());
        let sample = track.sample(
            ElapsedTime::from_secs(0.5).unwrap(),
            AnimationPlayState::Running,
        );

        assert_sampled_number(&sample, "opacity", 0.25);
    }

    #[test]
    fn completed_transition_finishes_without_after_fill_value() {
        let track = opacity_track(
            InterpolableNumber::new(0.0).unwrap(),
            InterpolableNumber::new(1.0).unwrap(),
            AnimationDuration::from_secs(1.0).unwrap(),
            AnimationDelay::from_secs(0.0).unwrap(),
            Easing::linear(),
        );

        let sample = track.sample(
            ElapsedTime::from_secs(1.0).unwrap(),
            AnimationPlayState::Running,
        );

        assert_eq!(sample.classification(), SampleClassification::Finished);
        assert_eq!(sample.completion(), CompletionStatus::Finished);
        assert_eq!(sample.value(), &SampleValue::None);
    }

    #[test]
    fn paused_transition_reports_stable_active_sample() {
        let track = opacity_track(
            InterpolableNumber::new(0.0).unwrap(),
            InterpolableNumber::new(1.0).unwrap(),
            AnimationDuration::from_secs(1.0).unwrap(),
            AnimationDelay::from_secs(0.0).unwrap(),
            Easing::linear(),
        );

        let sample = track.sample(
            ElapsedTime::from_secs(0.5).unwrap(),
            AnimationPlayState::Paused,
        );

        assert_eq!(sample.classification(), SampleClassification::Active);
        assert_eq!(sample.next_frame(), NextFrameHint::StableUntilExternalInput);
        assert_sampled_number(&sample, "opacity", 0.5);
    }

    #[test]
    fn zero_duration_transition_finishes_at_delay_boundary_without_value() {
        let track = opacity_track(
            InterpolableNumber::new(0.0).unwrap(),
            InterpolableNumber::new(1.0).unwrap(),
            AnimationDuration::from_secs(0.0).unwrap(),
            AnimationDelay::from_secs(0.0).unwrap(),
            Easing::linear(),
        );

        let sample = track.sample(
            ElapsedTime::from_secs(0.0).unwrap(),
            AnimationPlayState::Running,
        );

        assert_eq!(sample.classification(), SampleClassification::Finished);
        assert_eq!(sample.value(), &SampleValue::None);
    }

    #[test]
    fn negative_delay_can_start_transition_partway_through() {
        let track = opacity_track(
            InterpolableNumber::new(0.0).unwrap(),
            InterpolableNumber::new(10.0).unwrap(),
            AnimationDuration::from_secs(2.0).unwrap(),
            AnimationDelay::from_secs(-0.5).unwrap(),
            Easing::linear(),
        );

        let sample = track.sample(
            ElapsedTime::from_secs(0.0).unwrap(),
            AnimationPlayState::Running,
        );

        assert_eq!(sample.classification(), SampleClassification::Active);
        assert_sampled_number(&sample, "opacity", 2.5);
    }

    #[test]
    fn negative_delay_can_skip_completed_transition() {
        let track = opacity_track(
            InterpolableNumber::new(0.0).unwrap(),
            InterpolableNumber::new(1.0).unwrap(),
            AnimationDuration::from_secs(1.0).unwrap(),
            AnimationDelay::from_secs(-2.0).unwrap(),
            Easing::linear(),
        );

        let sample = track.sample(
            ElapsedTime::from_secs(0.0).unwrap(),
            AnimationPlayState::Running,
        );

        assert_eq!(sample.classification(), SampleClassification::Finished);
        assert_eq!(sample.value(), &SampleValue::None);
    }

    #[test]
    fn unsupported_transition_interpolation_reports_unsupported_sample() {
        let track = TransitionTrack::new(
            TransitionTrackId::new(5),
            PropertyKey::new("transform").unwrap(),
            InterpolableValue::transform(TransformValue::unsupported("translate")),
            InterpolableValue::transform(TransformValue::unsupported("scale")),
            AnimationDuration::from_secs(1.0).unwrap(),
            AnimationDelay::from_secs(0.0).unwrap(),
            Easing::linear(),
        )
        .unwrap();

        let sample = track.sample(
            ElapsedTime::from_secs(0.5).unwrap(),
            AnimationPlayState::Running,
        );

        assert_eq!(sample.classification(), SampleClassification::Unsupported);
        assert_eq!(sample.completion(), CompletionStatus::Unsupported);
        assert_eq!(sample.next_frame(), NextFrameHint::StableUntilExternalInput);
        assert!(matches!(sample.value(), SampleValue::Unsupported(_)));
    }

    #[test]
    fn replacement_tracks_sample_supplied_endpoints_without_runtime_state() {
        let original = opacity_track(
            InterpolableNumber::new(0.0).unwrap(),
            InterpolableNumber::new(1.0).unwrap(),
            AnimationDuration::from_secs(1.0).unwrap(),
            AnimationDelay::from_secs(0.0).unwrap(),
            Easing::linear(),
        );
        let replacement = TransitionTrack::new(
            TransitionTrackId::new(99),
            PropertyKey::new("opacity").unwrap(),
            InterpolableValue::number(InterpolableNumber::new(0.5).unwrap()),
            InterpolableValue::number(InterpolableNumber::new(1.0).unwrap()),
            AnimationDuration::from_secs(1.0).unwrap(),
            AnimationDelay::from_secs(0.0).unwrap(),
            Easing::linear(),
        )
        .unwrap();

        assert_ne!(original.id(), replacement.id());
        let sample = replacement.sample(
            ElapsedTime::from_secs(0.5).unwrap(),
            AnimationPlayState::Running,
        );
        assert_sampled_number(&sample, "opacity", 0.75);
    }

    fn opacity_track(
        from: InterpolableNumber,
        to: InterpolableNumber,
        duration: AnimationDuration,
        delay: AnimationDelay,
        easing: Easing,
    ) -> TransitionTrack {
        TransitionTrack::new(
            TransitionTrackId::new(42),
            PropertyKey::new("opacity").unwrap(),
            InterpolableValue::number(from),
            InterpolableValue::number(to),
            duration,
            delay,
            easing,
        )
        .unwrap()
    }

    fn before_fill_failure_then_start_endpoint_easing() -> Easing {
        Easing::linear_function(
            LinearEasing::new(vec![
                LinearControlPoint::new(0.0, 0.5),
                LinearControlPoint::new(0.0, 0.0),
                LinearControlPoint::new(1.0, 1.0),
            ])
            .unwrap(),
        )
    }

    fn percentage_track(
        from: InterpolablePercentage,
        to: InterpolablePercentage,
        duration: AnimationDuration,
        delay: AnimationDelay,
        easing: Easing,
    ) -> TransitionTrack {
        TransitionTrack::new(
            TransitionTrackId::new(43),
            PropertyKey::new("width").unwrap(),
            InterpolableValue::percentage(from),
            InterpolableValue::percentage(to),
            duration,
            delay,
            easing,
        )
        .unwrap()
    }

    fn assert_sampled_number(sample: &SampledPropertyResult, property: &str, expected: f64) {
        let SampleValue::Value(sampled) = sample.value() else {
            panic!("expected sampled value, got {sample:?}");
        };
        assert_eq!(sampled.property().as_str(), property);
        let InterpolableValue::Number(number) = sampled.value() else {
            panic!("expected number value, got {:?}", sampled.value());
        };
        assert_eq!(number.value(), expected);
    }

    fn assert_sampled_percentage(sample: &SampledPropertyResult, property: &str, expected: f64) {
        let SampleValue::Value(sampled) = sample.value() else {
            panic!("expected sampled value, got {sample:?}");
        };
        assert_eq!(sampled.property().as_str(), property);
        let InterpolableValue::Percentage(percentage) = sampled.value() else {
            panic!("expected percentage value, got {:?}", sampled.value());
        };
        assert_eq!(percentage.value(), expected);
    }

    fn assert_arithmetic_error(
        sample: &SampledPropertyResult,
        property: &str,
        family: ValueFamily,
    ) {
        let SampleValue::Error(InterpolationError::NonFiniteInterpolationResult(result)) =
            sample.value()
        else {
            panic!("expected arithmetic error, got {sample:?}");
        };

        assert_eq!(result.property().as_str(), property);
        assert_eq!(result.family(), family);
        assert!(!result.value().is_finite());
    }
}
