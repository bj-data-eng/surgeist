use surgeist_animation::easing::{Easing, LinearControlPointInput, LinearEasing};
use surgeist_animation::interpolation::{
    ColorInterpolationSpace, InterpolableColor, InterpolableValue, PropertyKey,
};
use surgeist_animation::keyframe::{
    KeyframeAnimationId, KeyframeSegment, KeyframeTiming, KeyframeTrack,
};
use surgeist_animation::progress::UnitRatio;
use surgeist_animation::sample::{SampleClassification, SampleValue};
use surgeist_animation::time::{AnimationDelay, AnimationDuration, ElapsedTime};
use surgeist_animation::timing::{
    AnimationDirection, AnimationPlayState, FillMode, IterationCount,
};
use surgeist_animation::transition::{TransitionTrack, TransitionTrackId};

fn ratio(value: f64) -> UnitRatio {
    UnitRatio::new(value).unwrap()
}

fn time(seconds: f64) -> ElapsedTime {
    ElapsedTime::from_secs(seconds).unwrap()
}

fn progress_with_overshoot() -> Easing {
    Easing::LinearFunction(
        LinearEasing::new(
            [(0.0, 0.0), (0.25, 2.0), (0.75, 0.5), (1.0, 1.0)]
                .into_iter()
                .map(|(input, output)| LinearControlPointInput { input, output })
                .collect(),
        )
        .unwrap(),
    )
}

// The legacy explicit-space value can supply these sRGB endpoint coordinates
// to both samplers. The sample contract below concerns the straight color.
fn endpoint(red: f64, alpha: f64) -> InterpolableValue {
    InterpolableValue::color(
        InterpolableColor::from_straight_components(
            ColorInterpolationSpace::Srgb,
            red,
            0.0,
            0.0,
            ratio(alpha),
        )
        .unwrap(),
    )
}

fn transition() -> TransitionTrack {
    TransitionTrack::new(
        TransitionTrackId::new(1),
        PropertyKey::new("color").unwrap(),
        endpoint(f64::MAX, 0.75),
        endpoint(0.0, 0.5),
        AnimationDuration::from_secs(1.0).unwrap(),
        AnimationDelay::from_secs(0.0).unwrap(),
        progress_with_overshoot(),
    )
    .unwrap()
}

fn keyframes() -> KeyframeTrack {
    let segment = KeyframeSegment::new(
        PropertyKey::new("color").unwrap(),
        ratio(0.0),
        ratio(1.0),
        endpoint(f64::MAX, 0.75),
        endpoint(0.0, 0.5),
        progress_with_overshoot(),
    )
    .unwrap();
    KeyframeTrack::new(
        KeyframeAnimationId::new("overflow and recovery").unwrap(),
        KeyframeTiming::try_new(
            AnimationDelay::from_secs(0.0).unwrap(),
            AnimationDuration::from_secs(1.0).unwrap(),
            IterationCount::finite(1.0).unwrap(),
            AnimationDirection::Normal,
            FillMode::Both,
        )
        .unwrap(),
        vec![segment],
    )
    .unwrap()
}

#[test]
fn transition_reports_unrepresentable_straight_color_at_eased_progress_and_recovers() {
    let transition = transition();
    let failed = transition.sample(time(0.25), AnimationPlayState::Running);
    // At p = 2 the alpha is 1/4 and the premultiplied red is -3*MAX/4.
    // Its straight coordinate is -3*MAX, which cannot be published.
    assert_eq!(failed.classification(), SampleClassification::Failed);
    assert!(matches!(failed.value(), SampleValue::Error(_)));

    let recovered = transition.sample(time(0.75), AnimationPlayState::Running);
    assert_eq!(recovered.classification(), SampleClassification::Active);
}

#[test]
fn keyframes_report_unrepresentable_straight_color_at_eased_progress_and_recovers() {
    let track = keyframes();
    let failed = track.sample(time(0.25), AnimationPlayState::Running);
    assert_eq!(failed.len(), 1);
    assert_eq!(failed[0].classification(), SampleClassification::Failed);
    assert!(matches!(failed[0].value(), SampleValue::Error(_)));

    let recovered = track.sample(time(0.75), AnimationPlayState::Running);
    assert_eq!(recovered.len(), 1);
    assert_eq!(recovered[0].classification(), SampleClassification::Active);
}
