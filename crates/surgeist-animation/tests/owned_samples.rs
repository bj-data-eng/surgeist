#![forbid(unsafe_code)]

use surgeist_animation::easing::Easing;
use surgeist_animation::interpolation::{
    DiscreteValue, InterpolableNumber, InterpolableValue, InterpolationComponent,
    InterpolationError, PropertyKey, TransformValue, ValueFamily,
};
use surgeist_animation::sample::{
    CompletionStatus, NextFrameHint, SampleClassification, SampleValue, SampledPropertyResult,
};
use surgeist_animation::time::{AnimationDelay, AnimationDuration, ElapsedTime};
use surgeist_animation::timing::AnimationPlayState;
use surgeist_animation::transition::{TransitionTrack, TransitionTrackId};

#[test]
fn concurrent_discrete_samples_remain_owned_after_track_destruction() {
    let samples = {
        let track = transition(
            "visibility",
            InterpolableValue::discrete(DiscreteValue::new("hidden").unwrap()),
            InterpolableValue::discrete(DiscreteValue::new("visible").unwrap()),
        );
        std::thread::scope(|scope| {
            let first = scope.spawn(|| sample(&track, 0.25));
            let second = scope.spawn(|| sample(&track, 0.75));
            [first.join().unwrap(), second.join().unwrap()]
        })
    };

    for (sample, expected) in samples.iter().zip(["hidden", "visible"]) {
        assert_state(
            sample,
            SampleClassification::Active,
            CompletionStatus::Running,
            NextFrameHint::MayChange,
        );
        let SampleValue::Value(property) = sample.value() else {
            panic!("expected value")
        };
        assert_eq!(property.property().as_str(), "visibility");
        let InterpolableValue::Discrete(value) = property.value() else {
            panic!("expected discrete")
        };
        assert_eq!(value.token(), expected);
    }
}

#[test]
fn unsupported_sample_retains_both_endpoints_after_track_destruction() {
    let retained = {
        let track = transition(
            "transform",
            InterpolableValue::transform(TransformValue::unsupported("translate")),
            InterpolableValue::transform(TransformValue::unsupported("scale")),
        );
        sample(&track, 0.5)
    };
    assert_state(
        &retained,
        SampleClassification::Unsupported,
        CompletionStatus::Unsupported,
        NextFrameHint::StableUntilExternalInput,
    );
    let SampleValue::Unsupported(InterpolationError::UnsupportedFamily(error)) = retained.value()
    else {
        panic!("expected unsupported family")
    };
    assert_eq!(error.property().as_str(), "transform");
    assert_eq!(error.family(), ValueFamily::Transform);
    assert_eq!(error.reason(), "transform interpolation deferred");
    let (InterpolableValue::Transform(from), InterpolableValue::Transform(to)) =
        (error.from(), error.to())
    else {
        panic!("expected transform endpoints")
    };
    assert_eq!(from.kind(), "translate");
    assert_eq!(to.kind(), "scale");
}

#[test]
fn arithmetic_diagnostic_survives_recovery_and_track_destruction() {
    let (failure, recovery) = {
        let track = TransitionTrack::new(
            TransitionTrackId::new(1),
            PropertyKey::new("opacity").unwrap(),
            number(0.0),
            number(f64::MAX),
            AnimationDuration::from_secs(1.0).unwrap(),
            AnimationDelay::from_secs(0.0).unwrap(),
            overflow_easing(),
        )
        .unwrap();
        (sample(&track, 0.5), sample(&track, 0.0))
    };
    assert_state(
        &failure,
        SampleClassification::Failed,
        CompletionStatus::Running,
        NextFrameHint::MayChange,
    );
    let SampleValue::Error(InterpolationError::NonFiniteInterpolationResult(error)) =
        failure.value()
    else {
        panic!("expected arithmetic diagnostic")
    };
    assert_eq!(error.property().as_str(), "opacity");
    assert_eq!(error.family(), ValueFamily::Number);
    assert_eq!(error.from(), &number(0.0));
    assert_eq!(error.to(), &number(f64::MAX));
    assert_eq!(error.progress().value(), 2.0);
    assert_eq!(error.component(), InterpolationComponent::Number);
    assert_eq!(error.value(), f64::INFINITY);
    assert_state(
        &recovery,
        SampleClassification::Active,
        CompletionStatus::Running,
        NextFrameHint::MayChange,
    );
    let SampleValue::Value(value) = recovery.value() else {
        panic!("expected recovery")
    };
    assert_eq!(value.property().as_str(), "opacity");
    assert_eq!(value.value(), &number(0.0));
}

fn number(value: f64) -> InterpolableValue {
    InterpolableValue::number(InterpolableNumber::new(value).unwrap())
}

fn transition(property: &str, from: InterpolableValue, to: InterpolableValue) -> TransitionTrack {
    TransitionTrack::new(
        TransitionTrackId::new(1),
        PropertyKey::new(property).unwrap(),
        from,
        to,
        AnimationDuration::from_secs(1.0).unwrap(),
        AnimationDelay::from_secs(0.0).unwrap(),
        Easing::linear(),
    )
    .unwrap()
}

fn sample(track: &TransitionTrack, elapsed: f64) -> SampledPropertyResult {
    track.sample(
        ElapsedTime::from_secs(elapsed).unwrap(),
        AnimationPlayState::Running,
    )
}

fn assert_state(
    sample: &SampledPropertyResult,
    classification: SampleClassification,
    completion: CompletionStatus,
    hint: NextFrameHint,
) {
    assert_eq!(sample.classification(), classification);
    assert_eq!(sample.completion(), completion);
    assert_eq!(sample.next_frame(), hint);
}

fn overflow_easing() -> Easing {
    use surgeist_animation::easing::{LinearControlPointInput, LinearEasing};
    Easing::linear_function(
        LinearEasing::new(vec![
            LinearControlPointInput {
                input: 0.0,
                output: 0.0,
            },
            LinearControlPointInput {
                input: 0.5,
                output: 2.0,
            },
            LinearControlPointInput {
                input: 1.0,
                output: 1.0,
            },
        ])
        .unwrap(),
    )
}
