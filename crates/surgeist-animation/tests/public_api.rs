#![forbid(unsafe_code)]

use surgeist_animation as animation;
use surgeist_animation::{
    AnimationDelay, AnimationDuration, CRATE_NAME, ColorInterpolationSpace, ElapsedTime,
    InterpolableColor, InterpolableValue, InterpolationOutcome, InterpolationPair,
    InterpolationProgress, NormalizedProgress, NumericError, NumericInput, PropertyKey, UnitRatio,
};

#[test]
fn exposes_crate_identity() {
    assert_eq!(CRATE_NAME, "surgeist-animation");
}

#[test]
fn exposes_numeric_foundation_front_doors() {
    assert_eq!(AnimationDuration::from_secs(0.0).unwrap().as_secs(), 0.0);
    assert_eq!(AnimationDelay::from_secs(-0.1).unwrap().as_secs(), -0.1);
    assert_eq!(ElapsedTime::from_secs(1.0).unwrap().as_secs(), 1.0);
    assert_eq!(NormalizedProgress::new(1.0).unwrap().value(), 1.0);
    assert_eq!(UnitRatio::new(0.0).unwrap().value(), 0.0);
    assert_eq!(
        AnimationDuration::from_secs(f64::INFINITY),
        Err(NumericError::non_finite(NumericInput::Duration))
    );
}

#[test]
fn public_front_door_exposes_explicit_color_space_interpolation() {
    let from = InterpolableColor::from_straight_components(
        ColorInterpolationSpace::Oklab,
        0.5,
        0.25,
        0.0,
        UnitRatio::new(0.5).unwrap(),
    )
    .unwrap();
    let to = InterpolableColor::from_straight_components(
        ColorInterpolationSpace::Oklab,
        0.0,
        0.25,
        0.5,
        UnitRatio::new(1.0).unwrap(),
    )
    .unwrap();
    let pair = InterpolationPair::new(
        PropertyKey::new("color").unwrap(),
        InterpolableValue::color(from),
        InterpolableValue::color(to),
    )
    .unwrap();

    assert_eq!(pair.property().as_str(), "color");
    let InterpolationOutcome::Value(InterpolableValue::Color(color)) =
        pair.sample(InterpolationProgress::new(0.5).unwrap())
    else {
        panic!("expected public color interpolation sample");
    };

    assert_eq!(color.space(), ColorInterpolationSpace::Oklab);
    assert_eq!(color.premultiplied_components(), (0.125, 0.1875, 0.25));
    assert_eq!(color.alpha(), 0.75);
}

#[test]
fn linear_easing_samples_through_public_reexports() {
    let linear = animation::LinearEasing::new(vec![
        animation::LinearControlPoint::new(0.0, 0.0),
        animation::LinearControlPoint::new(0.5, 0.25),
        animation::LinearControlPoint::new(1.0, 1.0),
    ])
    .unwrap();
    let easing = animation::Easing::linear_function(linear);
    let output = easing
        .evaluate_unrestricted(animation::EasingInput::new(0.5).unwrap())
        .unwrap();

    assert_eq!(output.value(), 0.25);
    assert!(easing.as_linear_function().is_some());
}

#[test]
fn unrestricted_percentage_interpolation_samples_through_public_reexports() {
    let pair = animation::InterpolationPair::new(
        animation::PropertyKey::new("width").unwrap(),
        animation::InterpolableValue::percentage(
            animation::InterpolablePercentage::new(-0.5).unwrap(),
        ),
        animation::InterpolableValue::percentage(
            animation::InterpolablePercentage::new(1.5).unwrap(),
        ),
    )
    .unwrap();

    assert_eq!(pair.property().as_str(), "width");
    let animation::InterpolationOutcome::Value(animation::InterpolableValue::Percentage(value)) =
        pair.sample(animation::InterpolationProgress::new(0.5).unwrap())
    else {
        panic!("expected percentage interpolation value");
    };

    assert_eq!(value.value(), 0.5);
}

#[test]
fn explicit_color_space_interpolation_samples_through_public_reexports() {
    let from = animation::InterpolableColor::from_straight_components(
        animation::ColorInterpolationSpace::Oklab,
        0.5,
        0.0,
        0.0,
        animation::UnitRatio::new(0.5).unwrap(),
    )
    .unwrap();
    let to = animation::InterpolableColor::from_straight_components(
        animation::ColorInterpolationSpace::Oklab,
        0.0,
        0.5,
        0.5,
        animation::UnitRatio::new(1.0).unwrap(),
    )
    .unwrap();
    let pair = animation::InterpolationPair::new(
        animation::PropertyKey::new("color").unwrap(),
        animation::InterpolableValue::color(from),
        animation::InterpolableValue::color(to),
    )
    .unwrap();

    assert_eq!(pair.property().as_str(), "color");
    let animation::InterpolationOutcome::Value(animation::InterpolableValue::Color(color)) =
        pair.sample(animation::InterpolationProgress::new(0.5).unwrap())
    else {
        panic!("expected color interpolation value");
    };

    assert_eq!(color.space(), animation::ColorInterpolationSpace::Oklab);
    assert_eq!(color.premultiplied_components(), (0.125, 0.25, 0.25));
    assert_eq!(color.alpha(), 0.75);
}

#[test]
fn delayed_transition_sample_uses_public_reexports() {
    let track = public_transition(
        animation::InterpolableValue::number(animation::InterpolableNumber::new(0.0).unwrap()),
        animation::InterpolableValue::number(animation::InterpolableNumber::new(1.0).unwrap()),
        animation::AnimationDuration::from_secs(1.0).unwrap(),
        animation::AnimationDelay::from_secs(0.5).unwrap(),
        animation::Easing::linear(),
    );

    let sample = track.sample(
        animation::ElapsedTime::from_secs(0.25).unwrap(),
        animation::AnimationPlayState::Running,
    );

    assert_eq!(
        sample.classification(),
        animation::SampleClassification::Filling
    );
    assert_eq!(sample.completion(), animation::CompletionStatus::NotStarted);
    assert_eq!(sample.next_frame(), animation::NextFrameHint::MayChange);
    assert_sampled_number(&sample, 0.0);
}

#[test]
fn zero_iteration_keyframe_sample_uses_public_reexports() {
    let timing = animation::KeyframeTiming::try_new(
        animation::AnimationDelay::from_secs(0.0).unwrap(),
        animation::AnimationDuration::from_secs(1.0).unwrap(),
        animation::IterationCount::finite(0.0).unwrap(),
        animation::AnimationDirection::Reverse,
        animation::FillMode::Forwards,
    )
    .unwrap();
    let segment = animation::KeyframeSegment::new(
        animation::PropertyKey::new("opacity").unwrap(),
        animation::UnitRatio::new(0.0).unwrap(),
        animation::UnitRatio::new(1.0).unwrap(),
        animation::InterpolableValue::number(animation::InterpolableNumber::new(0.0).unwrap()),
        animation::InterpolableValue::number(animation::InterpolableNumber::new(1.0).unwrap()),
        animation::Easing::linear(),
    )
    .unwrap();
    let track = animation::KeyframeTrack::new(
        animation::KeyframeAnimationId::new("fade").unwrap(),
        timing,
        vec![segment],
    )
    .unwrap();

    let samples = track.sample(
        animation::ElapsedTime::from_secs(0.0).unwrap(),
        animation::AnimationPlayState::Running,
    );

    assert_eq!(
        samples[0].classification(),
        animation::SampleClassification::Filling
    );
    assert_eq!(
        samples[0].completion(),
        animation::CompletionStatus::Finished
    );
    assert_eq!(
        samples[0].next_frame(),
        animation::NextFrameHint::StableUntilExternalInput
    );
    assert_sampled_number(&samples[0], 1.0);
}

#[test]
fn unsupported_sample_distinction_uses_public_reexports() {
    let track = animation::TransitionTrack::new(
        animation::TransitionTrackId::new(2),
        animation::PropertyKey::new("transform").unwrap(),
        animation::InterpolableValue::transform(animation::TransformValue::unsupported(
            "translate",
        )),
        animation::InterpolableValue::transform(animation::TransformValue::unsupported("scale")),
        animation::AnimationDuration::from_secs(1.0).unwrap(),
        animation::AnimationDelay::from_secs(0.0).unwrap(),
        animation::Easing::linear(),
    )
    .unwrap();

    let sample = track.sample(
        animation::ElapsedTime::from_secs(0.5).unwrap(),
        animation::AnimationPlayState::Running,
    );

    assert_eq!(
        sample.classification(),
        animation::SampleClassification::Unsupported
    );
    assert_eq!(
        sample.completion(),
        animation::CompletionStatus::Unsupported
    );
    assert_eq!(
        sample.next_frame(),
        animation::NextFrameHint::StableUntilExternalInput
    );
    assert!(matches!(
        sample.value(),
        animation::SampleValue::Unsupported(
            animation::InterpolationError::UnsupportedFamily { property, family: animation::ValueFamily::Transform, .. }
        ) if property.as_str() == "transform"
    ));
}

#[test]
fn transient_arithmetic_failure_sample_distinction_uses_public_reexports() {
    let track = public_transition(
        animation::InterpolableValue::number(animation::InterpolableNumber::new(f64::MAX).unwrap()),
        animation::InterpolableValue::number(
            animation::InterpolableNumber::new(-f64::MAX).unwrap(),
        ),
        animation::AnimationDuration::from_secs(1.0).unwrap(),
        animation::AnimationDelay::from_secs(0.0).unwrap(),
        animation::Easing::linear(),
    );

    let failed = track.sample(
        animation::ElapsedTime::from_secs(0.5).unwrap(),
        animation::AnimationPlayState::Running,
    );
    let recovered = track.sample(
        animation::ElapsedTime::from_secs(0.0).unwrap(),
        animation::AnimationPlayState::Running,
    );

    assert_eq!(
        failed.classification(),
        animation::SampleClassification::Failed
    );
    assert_eq!(failed.completion(), animation::CompletionStatus::Running);
    assert_eq!(failed.next_frame(), animation::NextFrameHint::MayChange);
    let animation::SampleValue::Error(animation::InterpolationError::NonFiniteInterpolationResult(
        result,
    )) = failed.value()
    else {
        panic!("expected sample-local arithmetic failure");
    };
    assert_eq!(result.property().as_str(), "opacity");
    assert_eq!(result.family(), animation::ValueFamily::Number);
    assert!(!result.value().is_finite());

    assert_eq!(
        recovered.classification(),
        animation::SampleClassification::Active
    );
    assert_eq!(recovered.completion(), animation::CompletionStatus::Running);
    assert_eq!(recovered.next_frame(), animation::NextFrameHint::MayChange);
    assert_sampled_number(&recovered, f64::MAX);
}

#[test]
fn transition_sampling_uses_public_front_doors() {
    let track = animation::TransitionTrack::new(
        animation::TransitionTrackId::new(3),
        animation::PropertyKey::new("opacity").unwrap(),
        animation::InterpolableValue::number(animation::InterpolableNumber::new(0.0).unwrap()),
        animation::InterpolableValue::number(animation::InterpolableNumber::new(1.0).unwrap()),
        animation::AnimationDuration::from_secs(1.0).unwrap(),
        animation::AnimationDelay::from_secs(0.0).unwrap(),
        animation::Easing::linear(),
    )
    .unwrap();

    let sample = track.sample(
        animation::ElapsedTime::from_secs(0.5).unwrap(),
        animation::AnimationPlayState::Running,
    );

    assert_eq!(
        sample.classification(),
        animation::SampleClassification::Active
    );
    assert_active_sample(&sample, "opacity", animation::ValueFamily::Number, 0.5);
}

#[test]
fn transition_percentage_sampling_uses_public_front_doors() {
    let track = animation::TransitionTrack::new(
        animation::TransitionTrackId::new(4),
        animation::PropertyKey::new("width").unwrap(),
        animation::InterpolableValue::percentage(
            animation::InterpolablePercentage::new(-0.5).unwrap(),
        ),
        animation::InterpolableValue::percentage(
            animation::InterpolablePercentage::new(1.5).unwrap(),
        ),
        animation::AnimationDuration::from_secs(2.0).unwrap(),
        animation::AnimationDelay::from_secs(0.0).unwrap(),
        animation::Easing::linear(),
    )
    .unwrap();

    let sample = track.sample(
        animation::ElapsedTime::from_secs(1.0).unwrap(),
        animation::AnimationPlayState::Running,
    );

    assert_eq!(
        sample.classification(),
        animation::SampleClassification::Active
    );
    assert_active_sample(&sample, "width", animation::ValueFamily::Percentage, 0.5);
}

#[test]
fn keyframe_sampling_uses_public_front_doors() {
    let timing = animation::KeyframeTiming::try_new(
        animation::AnimationDelay::from_secs(0.0).unwrap(),
        animation::AnimationDuration::from_secs(1.0).unwrap(),
        animation::IterationCount::finite(1.0).unwrap(),
        animation::AnimationDirection::Normal,
        animation::FillMode::Both,
    )
    .unwrap();
    let segment = animation::KeyframeSegment::new(
        animation::PropertyKey::new("opacity").unwrap(),
        animation::UnitRatio::new(0.0).unwrap(),
        animation::UnitRatio::new(1.0).unwrap(),
        animation::InterpolableValue::number(animation::InterpolableNumber::new(0.0).unwrap()),
        animation::InterpolableValue::number(animation::InterpolableNumber::new(1.0).unwrap()),
        animation::Easing::linear(),
    )
    .unwrap();
    let track = animation::KeyframeTrack::new(
        animation::KeyframeAnimationId::new("fade").unwrap(),
        timing,
        vec![segment],
    )
    .unwrap();

    let samples = track.sample(
        animation::ElapsedTime::from_secs(0.5).unwrap(),
        animation::AnimationPlayState::Running,
    );

    assert_eq!(
        samples[0].classification(),
        animation::SampleClassification::Active
    );
    assert_active_sample(&samples[0], "opacity", animation::ValueFamily::Number, 0.5);
}

#[test]
fn keyframe_percentage_sampling_uses_public_front_doors() {
    let timing = animation::KeyframeTiming::try_new(
        animation::AnimationDelay::from_secs(0.0).unwrap(),
        animation::AnimationDuration::from_secs(1.0).unwrap(),
        animation::IterationCount::finite(1.0).unwrap(),
        animation::AnimationDirection::Normal,
        animation::FillMode::Both,
    )
    .unwrap();
    let segment = animation::KeyframeSegment::new(
        animation::PropertyKey::new("width").unwrap(),
        animation::UnitRatio::new(0.0).unwrap(),
        animation::UnitRatio::new(1.0).unwrap(),
        animation::InterpolableValue::percentage(
            animation::InterpolablePercentage::new(-0.5).unwrap(),
        ),
        animation::InterpolableValue::percentage(
            animation::InterpolablePercentage::new(1.5).unwrap(),
        ),
        animation::Easing::linear(),
    )
    .unwrap();
    let track = animation::KeyframeTrack::new(
        animation::KeyframeAnimationId::new("grow").unwrap(),
        timing,
        vec![segment],
    )
    .unwrap();

    let samples = track.sample(
        animation::ElapsedTime::from_secs(0.5).unwrap(),
        animation::AnimationPlayState::Running,
    );

    assert_eq!(
        samples[0].classification(),
        animation::SampleClassification::Active
    );
    assert_active_sample(
        &samples[0],
        "width",
        animation::ValueFamily::Percentage,
        0.5,
    );
}

#[test]
fn public_api_shape_uses_front_door_only() {
    let timing = animation::TimingParameters::try_new(
        animation::AnimationDelay::from_secs(0.0).unwrap(),
        animation::AnimationDuration::from_secs(1.0).unwrap(),
        animation::IterationCount::finite(1.0).unwrap(),
        animation::AnimationDirection::Normal,
        animation::FillMode::Both,
        animation::Easing::linear(),
    )
    .unwrap();
    let pair = animation::InterpolationPair::new(
        animation::PropertyKey::new("opacity").unwrap(),
        animation::InterpolableValue::number(animation::InterpolableNumber::new(0.0).unwrap()),
        animation::InterpolableValue::number(animation::InterpolableNumber::new(1.0).unwrap()),
    )
    .unwrap();
    let sample = timing.sample(
        animation::ElapsedTime::from_secs(0.5).unwrap(),
        animation::AnimationPlayState::Running,
    );
    let output = animation::SampledPropertyResult::from_timing_and_pair(sample, &pair);

    assert_eq!(
        output.classification(),
        animation::SampleClassification::Active
    );
    assert_active_sample(&output, "opacity", animation::ValueFamily::Number, 0.5);
}

fn public_transition(
    from: animation::InterpolableValue,
    to: animation::InterpolableValue,
    duration: animation::AnimationDuration,
    delay: animation::AnimationDelay,
    easing: animation::Easing,
) -> animation::TransitionTrack {
    animation::TransitionTrack::new(
        animation::TransitionTrackId::new(1),
        animation::PropertyKey::new("opacity").unwrap(),
        from,
        to,
        duration,
        delay,
        easing,
    )
    .unwrap()
}

fn assert_sampled_number(sample: &animation::SampledPropertyResult, expected: f64) {
    let animation::SampleValue::Value(sampled) = sample.value() else {
        panic!("expected sampled value, got {sample:?}");
    };
    assert_eq!(sampled.property().as_str(), "opacity");
    let animation::InterpolableValue::Number(number) = sampled.value() else {
        panic!("expected number value, got {:?}", sampled.value());
    };
    assert_eq!(number.value(), expected);
}

fn assert_active_sample(
    sample: &animation::SampledPropertyResult,
    property: &str,
    family: animation::ValueFamily,
    expected: f64,
) {
    assert_eq!(
        sample.classification(),
        animation::SampleClassification::Active
    );
    assert_eq!(sample.completion(), animation::CompletionStatus::Running);
    assert_eq!(sample.next_frame(), animation::NextFrameHint::MayChange);
    let animation::SampleValue::Value(value) = sample.value() else {
        panic!("expected value, got {sample:?}")
    };
    assert_eq!(value.property().as_str(), property);
    assert_eq!(value.value().family(), family);
    let actual = match value.value() {
        animation::InterpolableValue::Number(value) => value.value(),
        animation::InterpolableValue::Percentage(value) => value.value(),
        value => panic!("expected scalar, got {value:?}"),
    };
    assert_eq!(actual, expected);
}

#[test]
fn rejected_elapsed_time_exposes_numeric_diagnostic() {
    let error = ElapsedTime::from_secs(-1.0).unwrap_err();
    assert_eq!(
        error,
        NumericError::negative(NumericInput::ElapsedTime, -1.0)
    );
    assert_eq!(
        error.to_string(),
        "elapsed time must be non-negative, got -1"
    );
    assert!(std::error::Error::source(&error).is_none());
}
