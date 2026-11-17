#![forbid(unsafe_code)]

use surgeist_animation as animation;
use surgeist_animation::CRATE_NAME;
use surgeist_animation::diagnostics::{NumericError, NumericInput};
use surgeist_animation::interpolation::{
    ColorInterpolationSpace, InterpolableColor, InterpolableValue, InterpolationOutcome,
    InterpolationPair, InterpolationProgress, PropertyKey,
};
use surgeist_animation::progress::{NormalizedProgress, UnitRatio};
use surgeist_animation::time::{AnimationDelay, AnimationDuration, ElapsedTime};

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
fn linear_easing_samples_through_public_modules() {
    let linear = animation::easing::LinearEasing::new(vec![
        animation::easing::LinearControlPoint::new(0.0, 0.0),
        animation::easing::LinearControlPoint::new(0.5, 0.25),
        animation::easing::LinearControlPoint::new(1.0, 1.0),
    ])
    .unwrap();
    let easing = animation::easing::Easing::linear_function(linear);
    let output = easing
        .evaluate_unrestricted(animation::progress::EasingInput::new(0.5).unwrap())
        .unwrap();

    assert_eq!(output.value(), 0.25);
    assert!(easing.as_linear_function().is_some());
}

#[test]
fn unrestricted_percentage_interpolation_samples_through_public_modules() {
    let pair = animation::interpolation::InterpolationPair::new(
        animation::interpolation::PropertyKey::new("width").unwrap(),
        animation::interpolation::InterpolableValue::percentage(
            animation::interpolation::InterpolablePercentage::new(-0.5).unwrap(),
        ),
        animation::interpolation::InterpolableValue::percentage(
            animation::interpolation::InterpolablePercentage::new(1.5).unwrap(),
        ),
    )
    .unwrap();

    assert_eq!(pair.property().as_str(), "width");
    let animation::interpolation::InterpolationOutcome::Value(
        animation::interpolation::InterpolableValue::Percentage(value),
    ) = pair.sample(animation::interpolation::InterpolationProgress::new(0.5).unwrap())
    else {
        panic!("expected percentage interpolation value");
    };

    assert_eq!(value.value(), 0.5);
}

#[test]
fn explicit_color_space_interpolation_samples_through_public_modules() {
    let from = animation::interpolation::InterpolableColor::from_straight_components(
        animation::interpolation::ColorInterpolationSpace::Oklab,
        0.5,
        0.0,
        0.0,
        animation::progress::UnitRatio::new(0.5).unwrap(),
    )
    .unwrap();
    let to = animation::interpolation::InterpolableColor::from_straight_components(
        animation::interpolation::ColorInterpolationSpace::Oklab,
        0.0,
        0.5,
        0.5,
        animation::progress::UnitRatio::new(1.0).unwrap(),
    )
    .unwrap();
    let pair = animation::interpolation::InterpolationPair::new(
        animation::interpolation::PropertyKey::new("color").unwrap(),
        animation::interpolation::InterpolableValue::color(from),
        animation::interpolation::InterpolableValue::color(to),
    )
    .unwrap();

    assert_eq!(pair.property().as_str(), "color");
    let animation::interpolation::InterpolationOutcome::Value(
        animation::interpolation::InterpolableValue::Color(color),
    ) = pair.sample(animation::interpolation::InterpolationProgress::new(0.5).unwrap())
    else {
        panic!("expected color interpolation value");
    };

    assert_eq!(
        color.space(),
        animation::interpolation::ColorInterpolationSpace::Oklab
    );
    assert_eq!(color.premultiplied_components(), (0.125, 0.25, 0.25));
    assert_eq!(color.alpha(), 0.75);
}

#[test]
fn delayed_transition_sample_uses_public_modules() {
    let track = public_transition(
        animation::interpolation::InterpolableValue::number(
            animation::interpolation::InterpolableNumber::new(0.0).unwrap(),
        ),
        animation::interpolation::InterpolableValue::number(
            animation::interpolation::InterpolableNumber::new(1.0).unwrap(),
        ),
        animation::time::AnimationDuration::from_secs(1.0).unwrap(),
        animation::time::AnimationDelay::from_secs(0.5).unwrap(),
        animation::easing::Easing::linear(),
    );

    let sample = track.sample(
        animation::time::ElapsedTime::from_secs(0.25).unwrap(),
        animation::timing::AnimationPlayState::Running,
    );

    assert_eq!(
        sample.classification(),
        animation::sample::SampleClassification::Filling
    );
    assert_eq!(
        sample.completion(),
        animation::sample::CompletionStatus::NotStarted
    );
    assert_eq!(
        sample.next_frame(),
        animation::sample::NextFrameHint::MayChange
    );
    assert_sampled_number(&sample, 0.0);
}

#[test]
fn zero_iteration_keyframe_sample_uses_public_modules() {
    let timing = animation::keyframe::KeyframeTiming::try_new(
        animation::time::AnimationDelay::from_secs(0.0).unwrap(),
        animation::time::AnimationDuration::from_secs(1.0).unwrap(),
        animation::timing::IterationCount::finite(0.0).unwrap(),
        animation::timing::AnimationDirection::Reverse,
        animation::timing::FillMode::Forwards,
    )
    .unwrap();
    let segment = animation::keyframe::KeyframeSegment::new(
        animation::interpolation::PropertyKey::new("opacity").unwrap(),
        animation::progress::UnitRatio::new(0.0).unwrap(),
        animation::progress::UnitRatio::new(1.0).unwrap(),
        animation::interpolation::InterpolableValue::number(
            animation::interpolation::InterpolableNumber::new(0.0).unwrap(),
        ),
        animation::interpolation::InterpolableValue::number(
            animation::interpolation::InterpolableNumber::new(1.0).unwrap(),
        ),
        animation::easing::Easing::linear(),
    )
    .unwrap();
    let track = animation::keyframe::KeyframeTrack::new(
        animation::keyframe::KeyframeAnimationId::new("fade").unwrap(),
        timing,
        vec![segment],
    )
    .unwrap();

    let samples = track.sample(
        animation::time::ElapsedTime::from_secs(0.0).unwrap(),
        animation::timing::AnimationPlayState::Running,
    );

    assert_eq!(
        samples[0].classification(),
        animation::sample::SampleClassification::Filling
    );
    assert_eq!(
        samples[0].completion(),
        animation::sample::CompletionStatus::Finished
    );
    assert_eq!(
        samples[0].next_frame(),
        animation::sample::NextFrameHint::StableUntilExternalInput
    );
    assert_sampled_number(&samples[0], 1.0);
}

#[test]
fn unsupported_sample_distinction_uses_public_modules() {
    let track = animation::transition::TransitionTrack::new(
        animation::transition::TransitionTrackId::new(2),
        animation::interpolation::PropertyKey::new("transform").unwrap(),
        animation::interpolation::InterpolableValue::transform(
            animation::interpolation::TransformValue::unsupported("translate"),
        ),
        animation::interpolation::InterpolableValue::transform(
            animation::interpolation::TransformValue::unsupported("scale"),
        ),
        animation::time::AnimationDuration::from_secs(1.0).unwrap(),
        animation::time::AnimationDelay::from_secs(0.0).unwrap(),
        animation::easing::Easing::linear(),
    )
    .unwrap();

    let sample = track.sample(
        animation::time::ElapsedTime::from_secs(0.5).unwrap(),
        animation::timing::AnimationPlayState::Running,
    );

    assert_eq!(
        sample.classification(),
        animation::sample::SampleClassification::Unsupported
    );
    assert_eq!(
        sample.completion(),
        animation::sample::CompletionStatus::Unsupported
    );
    assert_eq!(
        sample.next_frame(),
        animation::sample::NextFrameHint::StableUntilExternalInput
    );
    assert!(matches!(
        sample.value(),
        animation::sample::SampleValue::Unsupported(
            animation::interpolation::InterpolationError::UnsupportedFamily { property, family: animation::interpolation::ValueFamily::Transform, .. }
        ) if property.as_str() == "transform"
    ));
}

#[test]
fn transient_arithmetic_failure_sample_distinction_uses_public_modules() {
    let track = public_transition(
        animation::interpolation::InterpolableValue::number(
            animation::interpolation::InterpolableNumber::new(f64::MAX).unwrap(),
        ),
        animation::interpolation::InterpolableValue::number(
            animation::interpolation::InterpolableNumber::new(-f64::MAX).unwrap(),
        ),
        animation::time::AnimationDuration::from_secs(1.0).unwrap(),
        animation::time::AnimationDelay::from_secs(0.0).unwrap(),
        animation::easing::Easing::linear(),
    );

    let failed = track.sample(
        animation::time::ElapsedTime::from_secs(0.5).unwrap(),
        animation::timing::AnimationPlayState::Running,
    );
    let recovered = track.sample(
        animation::time::ElapsedTime::from_secs(0.0).unwrap(),
        animation::timing::AnimationPlayState::Running,
    );

    assert_eq!(
        failed.classification(),
        animation::sample::SampleClassification::Failed
    );
    assert_eq!(
        failed.completion(),
        animation::sample::CompletionStatus::Running
    );
    assert_eq!(
        failed.next_frame(),
        animation::sample::NextFrameHint::MayChange
    );
    let animation::sample::SampleValue::Error(
        animation::interpolation::InterpolationError::NonFiniteInterpolationResult(result),
    ) = failed.value()
    else {
        panic!("expected sample-local arithmetic failure");
    };
    assert_eq!(result.property().as_str(), "opacity");
    assert_eq!(
        result.family(),
        animation::interpolation::ValueFamily::Number
    );
    assert!(!result.value().is_finite());

    assert_eq!(
        recovered.classification(),
        animation::sample::SampleClassification::Active
    );
    assert_eq!(
        recovered.completion(),
        animation::sample::CompletionStatus::Running
    );
    assert_eq!(
        recovered.next_frame(),
        animation::sample::NextFrameHint::MayChange
    );
    assert_sampled_number(&recovered, f64::MAX);
}

#[test]
fn transition_sampling_uses_public_front_doors() {
    let track = animation::transition::TransitionTrack::new(
        animation::transition::TransitionTrackId::new(3),
        animation::interpolation::PropertyKey::new("opacity").unwrap(),
        animation::interpolation::InterpolableValue::number(
            animation::interpolation::InterpolableNumber::new(0.0).unwrap(),
        ),
        animation::interpolation::InterpolableValue::number(
            animation::interpolation::InterpolableNumber::new(1.0).unwrap(),
        ),
        animation::time::AnimationDuration::from_secs(1.0).unwrap(),
        animation::time::AnimationDelay::from_secs(0.0).unwrap(),
        animation::easing::Easing::linear(),
    )
    .unwrap();

    let sample = track.sample(
        animation::time::ElapsedTime::from_secs(0.5).unwrap(),
        animation::timing::AnimationPlayState::Running,
    );

    assert_eq!(
        sample.classification(),
        animation::sample::SampleClassification::Active
    );
    assert_active_sample(
        &sample,
        "opacity",
        animation::interpolation::ValueFamily::Number,
        0.5,
    );
}

#[test]
fn transition_percentage_sampling_uses_public_front_doors() {
    let track = animation::transition::TransitionTrack::new(
        animation::transition::TransitionTrackId::new(4),
        animation::interpolation::PropertyKey::new("width").unwrap(),
        animation::interpolation::InterpolableValue::percentage(
            animation::interpolation::InterpolablePercentage::new(-0.5).unwrap(),
        ),
        animation::interpolation::InterpolableValue::percentage(
            animation::interpolation::InterpolablePercentage::new(1.5).unwrap(),
        ),
        animation::time::AnimationDuration::from_secs(2.0).unwrap(),
        animation::time::AnimationDelay::from_secs(0.0).unwrap(),
        animation::easing::Easing::linear(),
    )
    .unwrap();

    let sample = track.sample(
        animation::time::ElapsedTime::from_secs(1.0).unwrap(),
        animation::timing::AnimationPlayState::Running,
    );

    assert_eq!(
        sample.classification(),
        animation::sample::SampleClassification::Active
    );
    assert_active_sample(
        &sample,
        "width",
        animation::interpolation::ValueFamily::Percentage,
        0.5,
    );
}

#[test]
fn keyframe_sampling_uses_public_front_doors() {
    let timing = animation::keyframe::KeyframeTiming::try_new(
        animation::time::AnimationDelay::from_secs(0.0).unwrap(),
        animation::time::AnimationDuration::from_secs(1.0).unwrap(),
        animation::timing::IterationCount::finite(1.0).unwrap(),
        animation::timing::AnimationDirection::Normal,
        animation::timing::FillMode::Both,
    )
    .unwrap();
    let segment = animation::keyframe::KeyframeSegment::new(
        animation::interpolation::PropertyKey::new("opacity").unwrap(),
        animation::progress::UnitRatio::new(0.0).unwrap(),
        animation::progress::UnitRatio::new(1.0).unwrap(),
        animation::interpolation::InterpolableValue::number(
            animation::interpolation::InterpolableNumber::new(0.0).unwrap(),
        ),
        animation::interpolation::InterpolableValue::number(
            animation::interpolation::InterpolableNumber::new(1.0).unwrap(),
        ),
        animation::easing::Easing::linear(),
    )
    .unwrap();
    let track = animation::keyframe::KeyframeTrack::new(
        animation::keyframe::KeyframeAnimationId::new("fade").unwrap(),
        timing,
        vec![segment],
    )
    .unwrap();

    let samples = track.sample(
        animation::time::ElapsedTime::from_secs(0.5).unwrap(),
        animation::timing::AnimationPlayState::Running,
    );

    assert_eq!(
        samples[0].classification(),
        animation::sample::SampleClassification::Active
    );
    assert_active_sample(
        &samples[0],
        "opacity",
        animation::interpolation::ValueFamily::Number,
        0.5,
    );
}

#[test]
fn keyframe_percentage_sampling_uses_public_front_doors() {
    let timing = animation::keyframe::KeyframeTiming::try_new(
        animation::time::AnimationDelay::from_secs(0.0).unwrap(),
        animation::time::AnimationDuration::from_secs(1.0).unwrap(),
        animation::timing::IterationCount::finite(1.0).unwrap(),
        animation::timing::AnimationDirection::Normal,
        animation::timing::FillMode::Both,
    )
    .unwrap();
    let segment = animation::keyframe::KeyframeSegment::new(
        animation::interpolation::PropertyKey::new("width").unwrap(),
        animation::progress::UnitRatio::new(0.0).unwrap(),
        animation::progress::UnitRatio::new(1.0).unwrap(),
        animation::interpolation::InterpolableValue::percentage(
            animation::interpolation::InterpolablePercentage::new(-0.5).unwrap(),
        ),
        animation::interpolation::InterpolableValue::percentage(
            animation::interpolation::InterpolablePercentage::new(1.5).unwrap(),
        ),
        animation::easing::Easing::linear(),
    )
    .unwrap();
    let track = animation::keyframe::KeyframeTrack::new(
        animation::keyframe::KeyframeAnimationId::new("grow").unwrap(),
        timing,
        vec![segment],
    )
    .unwrap();

    let samples = track.sample(
        animation::time::ElapsedTime::from_secs(0.5).unwrap(),
        animation::timing::AnimationPlayState::Running,
    );

    assert_eq!(
        samples[0].classification(),
        animation::sample::SampleClassification::Active
    );
    assert_active_sample(
        &samples[0],
        "width",
        animation::interpolation::ValueFamily::Percentage,
        0.5,
    );
}

#[test]
fn public_api_shape_uses_front_door_only() {
    let timing = animation::timing::TimingParameters::try_new(
        animation::time::AnimationDelay::from_secs(0.0).unwrap(),
        animation::time::AnimationDuration::from_secs(1.0).unwrap(),
        animation::timing::IterationCount::finite(1.0).unwrap(),
        animation::timing::AnimationDirection::Normal,
        animation::timing::FillMode::Both,
        animation::easing::Easing::linear(),
    )
    .unwrap();
    let pair = animation::interpolation::InterpolationPair::new(
        animation::interpolation::PropertyKey::new("opacity").unwrap(),
        animation::interpolation::InterpolableValue::number(
            animation::interpolation::InterpolableNumber::new(0.0).unwrap(),
        ),
        animation::interpolation::InterpolableValue::number(
            animation::interpolation::InterpolableNumber::new(1.0).unwrap(),
        ),
    )
    .unwrap();
    let sample = timing.sample(
        animation::time::ElapsedTime::from_secs(0.5).unwrap(),
        animation::timing::AnimationPlayState::Running,
    );
    let output = animation::sample::SampledPropertyResult::from_timing_and_pair(sample, &pair);

    assert_eq!(
        output.classification(),
        animation::sample::SampleClassification::Active
    );
    assert_active_sample(
        &output,
        "opacity",
        animation::interpolation::ValueFamily::Number,
        0.5,
    );
}

fn public_transition(
    from: animation::interpolation::InterpolableValue,
    to: animation::interpolation::InterpolableValue,
    duration: animation::time::AnimationDuration,
    delay: animation::time::AnimationDelay,
    easing: animation::easing::Easing,
) -> animation::transition::TransitionTrack {
    animation::transition::TransitionTrack::new(
        animation::transition::TransitionTrackId::new(1),
        animation::interpolation::PropertyKey::new("opacity").unwrap(),
        from,
        to,
        duration,
        delay,
        easing,
    )
    .unwrap()
}

fn assert_sampled_number(sample: &animation::sample::SampledPropertyResult, expected: f64) {
    let animation::sample::SampleValue::Value(sampled) = sample.value() else {
        panic!("expected sampled value, got {sample:?}");
    };
    assert_eq!(sampled.property().as_str(), "opacity");
    let animation::interpolation::InterpolableValue::Number(number) = sampled.value() else {
        panic!("expected number value, got {:?}", sampled.value());
    };
    assert_eq!(number.value(), expected);
}

fn assert_active_sample(
    sample: &animation::sample::SampledPropertyResult,
    property: &str,
    family: animation::interpolation::ValueFamily,
    expected: f64,
) {
    assert_eq!(
        sample.classification(),
        animation::sample::SampleClassification::Active
    );
    assert_eq!(
        sample.completion(),
        animation::sample::CompletionStatus::Running
    );
    assert_eq!(
        sample.next_frame(),
        animation::sample::NextFrameHint::MayChange
    );
    let animation::sample::SampleValue::Value(value) = sample.value() else {
        panic!("expected value, got {sample:?}")
    };
    assert_eq!(value.property().as_str(), property);
    assert_eq!(value.value().family(), family);
    let actual = match value.value() {
        animation::interpolation::InterpolableValue::Number(value) => value.value(),
        animation::interpolation::InterpolableValue::Percentage(value) => value.value(),
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
