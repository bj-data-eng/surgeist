#![forbid(unsafe_code)]

use surgeist_animation::{
    AnimationDelay, AnimationDirection, AnimationDuration, AnimationPlayState, CompletionStatus,
    DiscreteValue, Easing, ElapsedTime, FillMode, InterpolableNumber, InterpolableValue,
    InterpolationComponent, InterpolationError, IterationCount, KeyframeAnimationId, KeyframeError,
    KeyframeSegment, KeyframeTiming, KeyframeTrack, NextFrameHint, PropertyKey,
    SampleClassification, SampleValue, SampledPropertyResult, StepPosition, Steps, TransformValue,
    UnitRatio, ValueFamily,
};

#[test]
fn interleaved_segments_keep_input_order_and_sample_in_first_property_order() {
    let segments = interleaved_segments();
    let track = track(
        segments.clone(),
        timing(0.0, 1.0, AnimationDirection::Normal, FillMode::Both),
    );

    assert_eq!(track.segments(), segments.as_slice());
    for (elapsed, opacity, width) in [
        (0.2, 20.0, 200.0),
        (0.399_999, 29.999_95, 200.0),
        (0.4, 80.0, 200.0),
        (0.400_001, 80.000_1, 200.0),
        (0.7, 110.0, 400.0),
    ] {
        let samples = track.sample(time(elapsed), AnimationPlayState::Running);
        assert_eq!(samples.len(), 2);
        assert_number(&samples[0], "opacity", opacity, active());
        assert_number(&samples[1], "width", width, active());
    }

    let final_samples = track.sample(time(1.0), AnimationPlayState::Running);
    assert_eq!(final_samples.len(), 2);
    assert_number(&final_samples[0], "opacity", 140.0, finished_fill());
    assert_number(&final_samples[1], "width", 400.0, finished_fill());
}

#[test]
fn first_property_diagnostic_wins_over_later_property_error() {
    let error = KeyframeTrack::new(
        KeyframeAnimationId::new("invalid").unwrap(),
        timing(0.0, 1.0, AnimationDirection::Normal, FillMode::Both),
        vec![
            segment("first", 0.0, 0.4, 10.0, 30.0),
            segment("second", 0.2, 1.0, 80.0, 140.0),
            segment("first", 0.4, 0.8, 80.0, 140.0),
        ],
    )
    .unwrap_err();

    assert_eq!(
        error,
        KeyframeError::MissingFinalOffset {
            property: property("first"),
        }
    );
}

#[test]
fn earliest_gap_wins_over_a_later_duplicate_start() {
    let error = invalid_track(vec![
        segment("opacity", 0.0, 0.2, 0.0, 1.0),
        segment("opacity", 0.4, 0.8, 2.0, 3.0),
        segment("opacity", 0.0, 1.0, 4.0, 5.0),
    ]);

    assert_eq!(
        error,
        KeyframeError::NonContiguousSegment {
            property: property("opacity"),
            expected: ratio(0.2),
            actual: ratio(0.4),
        }
    );
}

#[test]
fn a_backward_start_distinguishes_duplicates_from_overlaps() {
    for start in [0.2, 0.3] {
        let error = invalid_track(vec![
            segment("opacity", 0.0, 0.2, 0.0, 1.0),
            segment("opacity", 0.2, 0.4, 1.0, 2.0),
            segment("opacity", 0.4, 0.6, 2.0, 3.0),
            segment("opacity", 0.6, 0.8, 3.0, 4.0),
            segment("opacity", start, 1.0, 4.0, 5.0),
        ]);

        let expected = if start == 0.2 {
            KeyframeError::DuplicateOffset {
                property: property("opacity"),
                offset: ratio(0.2),
            }
        } else {
            KeyframeError::OverlappingSegment {
                property: property("opacity"),
                previous_to: ratio(0.8),
                actual: ratio(0.3),
            }
        };
        assert_eq!(error, expected);
    }
}

#[test]
fn signed_zero_starts_are_equal_for_duplicate_diagnostics() {
    for (first_zero, repeated_zero) in [(0.0, -0.0), (-0.0, 0.0)] {
        let error = invalid_track(vec![
            segment("opacity", first_zero, 0.4, 10.0, 30.0),
            segment("opacity", 0.4, 0.8, 80.0, 120.0),
            segment("opacity", repeated_zero, 1.0, 80.0, 140.0),
        ]);

        assert_eq!(
            error,
            KeyframeError::DuplicateOffset {
                property: property("opacity"),
                offset: ratio(repeated_zero),
            }
        );
    }
}

#[test]
fn interleaved_track_can_be_shared_and_its_samples_moved_between_threads() {
    let track = track(
        interleaved_segments(),
        timing(0.0, 2.0, AnimationDirection::Alternate, FillMode::Both),
    );
    let samples = std::thread::scope(|scope| {
        scope
            .spawn(|| track.sample(time(1.3), AnimationPlayState::Running))
            .join()
            .unwrap()
    });
    drop(track);
    std::thread::spawn(move || {
        assert_eq!(samples.len(), 2);
        assert_number(&samples[0], "opacity", 110.0, active());
        assert_number(&samples[1], "width", 400.0, active());
    })
    .join()
    .unwrap();
}

#[test]
fn values_and_complete_diagnostics_outlive_the_originating_track() {
    let track = diagnostic_track();
    let samples = track.sample(time(0.5), AnimationPlayState::Running);
    let endpoint = track.sample(time(1.0), AnimationPlayState::Running);
    drop(track);

    assert_eq!(samples.len(), 3);
    assert_discrete(&samples[0], "display", "visible", active());
    assert_unsupported(&samples[1]);
    assert_arithmetic_failure(&samples[2]);
    assert_number(&endpoint[2], "overflow", -f64::MAX, finished_fill());
}

#[test]
fn reusable_samples_replace_larger_results_and_retain_sufficient_capacity() {
    let diagnostics = diagnostic_track();
    let interleaved = track(
        interleaved_segments(),
        timing(0.0, 1.0, AnimationDirection::Normal, FillMode::Both),
    );
    let mut output = Vec::with_capacity(8);
    let capacity = output.capacity();
    diagnostics.sample_into(time(0.5), AnimationPlayState::Running, &mut output);
    assert_eq!(output.len(), 3);
    assert_discrete(&output[0], "display", "visible", active());
    assert_unsupported(&output[1]);
    assert_arithmetic_failure(&output[2]);

    interleaved.sample_into(time(0.2), AnimationPlayState::Running, &mut output);
    assert_eq!(output.len(), 2);
    assert_eq!(output.capacity(), capacity);
    assert_number(&output[0], "opacity", 20.0, active());
    assert_number(&output[1], "width", 200.0, active());

    interleaved.sample_into(time(0.7), AnimationPlayState::Paused, &mut output);
    let paused = (
        SampleClassification::Active,
        CompletionStatus::Running,
        NextFrameHint::StableUntilExternalInput,
    );
    assert_eq!(output.len(), 2);
    assert_eq!(output.capacity(), capacity);
    assert_number(&output[0], "opacity", 110.0, paused);
    assert_number(&output[1], "width", 400.0, paused);
    assert_eq!(
        output,
        interleaved.sample(time(0.7), AnimationPlayState::Paused)
    );

    diagnostics.sample_into(time(0.5), AnimationPlayState::Running, &mut output);
    assert_eq!(output.len(), 3);
    assert_eq!(output.capacity(), capacity);
    assert_discrete(&output[0], "display", "visible", active());
    assert_unsupported(&output[1]);
    assert_arithmetic_failure(&output[2]);
}

#[test]
fn reusable_samples_replace_values_with_pending_and_finished_states() {
    let values = track(
        interleaved_segments(),
        timing(0.0, 1.0, AnimationDirection::Normal, FillMode::Both),
    );
    let no_fill = track(
        interleaved_segments(),
        timing(1.0, 1.0, AnimationDirection::Normal, FillMode::None),
    );
    let mut output = values.sample(time(0.2), AnimationPlayState::Running);
    assert_number(&output[0], "opacity", 20.0, active());
    assert_number(&output[1], "width", 200.0, active());
    let capacity = output.capacity();

    for (elapsed, state) in [
        (
            0.0,
            (
                SampleClassification::Pending,
                CompletionStatus::NotStarted,
                NextFrameHint::MayChange,
            ),
        ),
        (
            2.0,
            (
                SampleClassification::Finished,
                CompletionStatus::Finished,
                NextFrameHint::StableUntilExternalInput,
            ),
        ),
    ] {
        no_fill.sample_into(time(elapsed), AnimationPlayState::Running, &mut output);
        assert_eq!(output.len(), 2);
        assert_eq!(output.capacity(), capacity);
        for sample in &output {
            assert_state(sample, state);
            assert_eq!(sample.value(), &SampleValue::None);
        }
    }
}

#[test]
fn reusable_samples_apply_local_steps_after_segment_selection() {
    let mut segments = interleaved_segments();
    segments[2] = KeyframeSegment::new(
        property("opacity"),
        ratio(0.4),
        ratio(1.0),
        number(80.0),
        number(140.0),
        Easing::steps(Steps::new(2, StepPosition::JumpEnd).unwrap()),
    )
    .unwrap();
    let track = track(
        segments,
        timing(0.0, 1.0, AnimationDirection::Normal, FillMode::Both),
    );
    let mut output = Vec::new();
    track.sample_into(time(0.55), AnimationPlayState::Running, &mut output);

    // (0.55 - 0.4) / (1 - 0.4) = 0.25; two-step jump-end maps this to zero.
    assert_eq!(output.len(), 2);
    assert_number(&output[0], "opacity", 80.0, active());
    assert_number(&output[1], "width", 400.0, active());
}

#[test]
fn reusable_samples_follow_alternate_direction_and_endpoint_fill() {
    let track = track(
        interleaved_segments(),
        timing(0.0, 2.0, AnimationDirection::Alternate, FillMode::Both),
    );
    let mut output = Vec::new();
    track.sample_into(time(1.3), AnimationPlayState::Running, &mut output);
    assert_eq!(output.len(), 2);
    assert_number(&output[0], "opacity", 110.0, active());
    assert_number(&output[1], "width", 400.0, active());

    track.sample_into(time(2.0), AnimationPlayState::Running, &mut output);
    assert_eq!(output.len(), 2);
    assert_number(&output[0], "opacity", 10.0, finished_fill());
    assert_number(&output[1], "width", 200.0, finished_fill());
}

#[test]
fn reusable_diagnostics_stay_owned_after_replacement_and_track_drop() {
    let track = diagnostic_track();
    let mut output = Vec::new();
    track.sample_into(time(0.5), AnimationPlayState::Running, &mut output);
    let retained = output.clone();
    track.sample_into(time(1.0), AnimationPlayState::Running, &mut output);
    drop(track);

    assert_eq!(retained.len(), 3);
    assert_discrete(&retained[0], "display", "visible", active());
    assert_unsupported(&retained[1]);
    assert_arithmetic_failure(&retained[2]);
    assert_eq!(output.len(), 3);
    assert_discrete(&output[0], "display", "visible", finished_fill());
    assert_unsupported(&output[1]);
    assert_number(&output[2], "overflow", -f64::MAX, finished_fill());
}

fn interleaved_segments() -> Vec<KeyframeSegment> {
    vec![
        segment("opacity", 0.0, 0.4, 10.0, 30.0),
        segment("width", 0.0, 0.5, 200.0, 200.0),
        segment("opacity", 0.4, 1.0, 80.0, 140.0),
        segment("width", 0.5, 1.0, 400.0, 400.0),
    ]
}

fn diagnostic_track() -> KeyframeTrack {
    let display = KeyframeSegment::new(
        property("display"),
        ratio(0.0),
        ratio(1.0),
        InterpolableValue::discrete(DiscreteValue::new("hidden").unwrap()),
        InterpolableValue::discrete(DiscreteValue::new("visible").unwrap()),
        Easing::linear(),
    )
    .unwrap();
    let transform = KeyframeSegment::new(
        property("transform"),
        ratio(0.0),
        ratio(1.0),
        InterpolableValue::transform(TransformValue::unsupported("translate")),
        InterpolableValue::transform(TransformValue::unsupported("scale")),
        Easing::linear(),
    )
    .unwrap();
    track(
        vec![
            display,
            transform,
            segment("overflow", 0.0, 1.0, f64::MAX, -f64::MAX),
        ],
        timing(0.0, 1.0, AnimationDirection::Normal, FillMode::Both),
    )
}

fn invalid_track(segments: Vec<KeyframeSegment>) -> KeyframeError {
    KeyframeTrack::new(
        KeyframeAnimationId::new("invalid").unwrap(),
        timing(0.0, 1.0, AnimationDirection::Normal, FillMode::Both),
        segments,
    )
    .unwrap_err()
}

fn track(segments: Vec<KeyframeSegment>, timing: KeyframeTiming) -> KeyframeTrack {
    KeyframeTrack::new(
        KeyframeAnimationId::new("motion").unwrap(),
        timing,
        segments,
    )
    .unwrap()
}

fn timing(
    delay: f64,
    iterations: f64,
    direction: AnimationDirection,
    fill: FillMode,
) -> KeyframeTiming {
    KeyframeTiming::try_new(
        AnimationDelay::from_secs(delay).unwrap(),
        AnimationDuration::from_secs(1.0).unwrap(),
        IterationCount::finite(iterations).unwrap(),
        direction,
        fill,
    )
    .unwrap()
}

fn segment(property_name: &str, start: f64, end: f64, from: f64, to: f64) -> KeyframeSegment {
    KeyframeSegment::new(
        property(property_name),
        ratio(start),
        ratio(end),
        number(from),
        number(to),
        Easing::linear(),
    )
    .unwrap()
}

fn property(name: &str) -> PropertyKey {
    PropertyKey::new(name).unwrap()
}

fn ratio(value: f64) -> UnitRatio {
    UnitRatio::new(value).unwrap()
}

fn number(value: f64) -> InterpolableValue {
    InterpolableValue::number(InterpolableNumber::new(value).unwrap())
}

fn time(value: f64) -> ElapsedTime {
    ElapsedTime::from_secs(value).unwrap()
}

type SampleState = (SampleClassification, CompletionStatus, NextFrameHint);

fn active() -> SampleState {
    (
        SampleClassification::Active,
        CompletionStatus::Running,
        NextFrameHint::MayChange,
    )
}

fn finished_fill() -> SampleState {
    (
        SampleClassification::Filling,
        CompletionStatus::Finished,
        NextFrameHint::StableUntilExternalInput,
    )
}

fn assert_state(sample: &SampledPropertyResult, expected: SampleState) {
    assert_eq!(
        (
            sample.classification(),
            sample.completion(),
            sample.next_frame()
        ),
        expected
    );
}

fn assert_number(
    sample: &SampledPropertyResult,
    property: &str,
    expected: f64,
    state: SampleState,
) {
    assert_state(sample, state);
    let SampleValue::Value(value) = sample.value() else {
        panic!("expected numeric sample, got {sample:?}");
    };
    assert_eq!(value.property().as_str(), property);
    let InterpolableValue::Number(actual) = value.value() else {
        panic!("expected number, got {value:?}");
    };
    let scale = expected.abs().max(1.0);
    assert!((actual.value() / scale - expected / scale).abs() < 1e-12);
}

fn assert_discrete(
    sample: &SampledPropertyResult,
    property: &str,
    token: &str,
    state: SampleState,
) {
    assert_state(sample, state);
    let SampleValue::Value(value) = sample.value() else {
        panic!("expected discrete sample, got {sample:?}");
    };
    assert_eq!(value.property().as_str(), property);
    let InterpolableValue::Discrete(actual) = value.value() else {
        panic!("expected discrete value, got {value:?}");
    };
    assert_eq!(actual.token(), token);
}

fn assert_unsupported(sample: &SampledPropertyResult) {
    assert_state(
        sample,
        (
            SampleClassification::Unsupported,
            CompletionStatus::Unsupported,
            NextFrameHint::StableUntilExternalInput,
        ),
    );
    let SampleValue::Unsupported(InterpolationError::UnsupportedFamily {
        property,
        family,
        from,
        to,
        reason,
    }) = sample.value()
    else {
        panic!("expected unsupported transform diagnostic, got {sample:?}");
    };
    assert_eq!(property.as_str(), "transform");
    assert_eq!(*family, ValueFamily::Transform);
    assert_eq!(
        from,
        &InterpolableValue::transform(TransformValue::unsupported("translate"))
    );
    assert_eq!(
        to,
        &InterpolableValue::transform(TransformValue::unsupported("scale"))
    );
    assert_eq!(*reason, "transform interpolation deferred");
}

fn assert_arithmetic_failure(sample: &SampledPropertyResult) {
    assert_state(
        sample,
        (
            SampleClassification::Failed,
            CompletionStatus::Running,
            NextFrameHint::MayChange,
        ),
    );
    let SampleValue::Error(InterpolationError::NonFiniteInterpolationResult(diagnostic)) =
        sample.value()
    else {
        panic!("expected arithmetic diagnostic, got {sample:?}");
    };
    assert_eq!(diagnostic.property().as_str(), "overflow");
    assert_eq!(diagnostic.family(), ValueFamily::Number);
    assert_eq!(diagnostic.from(), &number(f64::MAX));
    assert_eq!(diagnostic.to(), &number(-f64::MAX));
    assert_eq!(diagnostic.progress().value(), 0.5);
    assert_eq!(diagnostic.component(), InterpolationComponent::Number);
    assert_eq!(diagnostic.value(), f64::NEG_INFINITY);
}
