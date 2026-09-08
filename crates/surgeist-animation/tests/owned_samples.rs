#![forbid(unsafe_code)]

use surgeist_animation::{
    AnimationDelay, AnimationDuration, AnimationPlayState, CompletionStatus, DiscreteValue, Easing,
    ElapsedTime, InterpolableNumber, InterpolableValue, InterpolationComponent, InterpolationError,
    NextFrameHint, PropertyKey, SampleClassification, SampleValue, SampledPropertyResult,
    TransformValue, TransitionTrack, TransitionTrackId, ValueFamily,
};

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
    let SampleValue::Unsupported(InterpolationError::UnsupportedFamily {
        property,
        family,
        from,
        to,
        reason,
    }) = retained.value()
    else {
        panic!("expected unsupported family")
    };
    assert_eq!(property.as_str(), "transform");
    assert_eq!(*family, ValueFamily::Transform);
    assert_eq!(*reason, "transform interpolation deferred");
    let (InterpolableValue::Transform(from), InterpolableValue::Transform(to)) = (from, to) else {
        panic!("expected transform endpoints")
    };
    assert_eq!(from.kind(), "translate");
    assert_eq!(to.kind(), "scale");
}

#[test]
fn arithmetic_diagnostic_survives_recovery_and_track_destruction() {
    let (failure, recovery) = {
        let track = transition("opacity", number(f64::MAX), number(-f64::MAX));
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
    assert_eq!(error.from(), &number(f64::MAX));
    assert_eq!(error.to(), &number(-f64::MAX));
    assert_eq!(error.progress().value(), 0.5);
    assert_eq!(error.component(), InterpolationComponent::Number);
    assert_eq!(error.value(), f64::NEG_INFINITY);
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
    assert_eq!(value.value(), &number(f64::MAX));
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
