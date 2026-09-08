#![forbid(unsafe_code)]

use std::sync::Arc;

use surgeist_animation::{
    AnimationDelay, AnimationDirection, AnimationDuration, AnimationPlayState, CompletionStatus,
    Easing, ElapsedTime, FillMode, InterpolableNumber, InterpolableValue, IterationCount,
    KeyframeAnimationId, KeyframeError, KeyframeSegment, KeyframeTiming, KeyframeTrack,
    NextFrameHint, PropertyKey, SampleClassification, SampleValue, UnitRatio,
};

#[test]
fn one_property_with_one_two_or_eight_segments_selects_distinct_values() {
    let cases: [(u32, &[(f64, f64)]); 3] = [
        (1, &[(0.0, 10.0), (0.5, 20.0), (1.0, 30.0)]),
        (
            2,
            &[(0.25, 20.0), (0.5, 110.0), (0.75, 120.0), (1.0, 130.0)],
        ),
        (
            8,
            &[
                (0.0625, 20.0),
                (0.124_999, 29.999_84),
                (0.125, 110.0),
                (0.125_001, 110.000_16),
                (0.1875, 120.0),
                (0.875, 710.0),
                (0.9375, 720.0),
                (1.0, 730.0),
            ],
        ),
    ];

    for (count, samples) in cases {
        let segments: Vec<_> = (0..count)
            .map(|index| {
                let initial = 10.0 + 100.0 * f64::from(index);
                segment(
                    "opacity",
                    f64::from(index) / f64::from(count),
                    f64::from(index + 1) / f64::from(count),
                    initial,
                    initial + 20.0,
                )
            })
            .collect();
        let track = track_preserving_segments(segments);

        // Each segment rises by 20, with a jump of 80 at the next boundary.
        // Midpoints and the literal boundary expectations are calculated above.
        for &(elapsed, expected) in samples {
            assert_samples(&track, elapsed, &[("opacity", expected)]);
        }
    }
}

#[test]
fn contiguous_property_groups_preserve_input_order_and_following_boundaries() {
    let track = track_preserving_segments(vec![
        segment("width", 0.0, 0.5, 10.0, 30.0),
        segment("width", 0.5, 1.0, 110.0, 130.0),
        segment("opacity", 0.0, 1.0, 200.0, 220.0),
        segment("height", 0.0, 0.25, 300.0, 320.0),
        segment("height", 0.25, 1.0, 400.0, 460.0),
    ]);

    for (elapsed, width, opacity, height) in [
        (0.25, 20.0, 205.0, 400.0),
        (0.5, 110.0, 210.0, 420.0),
        (1.0, 130.0, 220.0, 460.0),
    ] {
        assert_samples(
            &track,
            elapsed,
            &[("width", width), ("opacity", opacity), ("height", height)],
        );
    }
}

#[test]
fn interrupted_property_runs_preserve_earlier_segments_and_first_appearance_order() {
    // A, A, B, A, C, B: both the initial run and later scattered entries matter.
    let track = track_preserving_segments(vec![
        segment("opacity", 0.0, 0.25, 10.0, 30.0),
        segment("opacity", 0.25, 0.5, 110.0, 130.0),
        segment("width", 0.0, 0.5, 200.0, 220.0),
        segment("opacity", 0.5, 1.0, 310.0, 350.0),
        segment("height", 0.0, 1.0, 400.0, 480.0),
        segment("width", 0.5, 1.0, 500.0, 560.0),
    ]);

    for (elapsed, opacity, width, height) in [
        (0.125, 20.0, 205.0, 410.0),
        (0.25, 110.0, 210.0, 420.0),
        (0.5, 310.0, 500.0, 440.0),
        (0.75, 330.0, 530.0, 460.0),
        (1.0, 350.0, 560.0, 480.0),
    ] {
        assert_samples(
            &track,
            elapsed,
            &[("opacity", opacity), ("width", width), ("height", height)],
        );
    }
}

#[test]
fn grouped_samples_retain_the_first_equal_property_allocation() {
    let first: Arc<str> = Arc::from("opacity");
    let later: Arc<str> = Arc::from("opacity");
    let first_weak = Arc::downgrade(&first);
    let later_weak = Arc::downgrade(&later);
    let track = construct(vec![
        segment_with_property(
            PropertyKey::from_shared(first).unwrap(),
            0.0,
            0.5,
            10.0,
            30.0,
        ),
        segment("width", 0.0, 1.0, 200.0, 220.0),
        segment_with_property(
            PropertyKey::from_shared(later).unwrap(),
            0.5,
            1.0,
            110.0,
            130.0,
        ),
    ])
    .unwrap();

    let samples = track.sample(
        ElapsedTime::from_secs(0.75).unwrap(),
        AnimationPlayState::Running,
    );
    assert_eq!(samples.len(), 2);
    let sample = &samples[0];
    assert_eq!(sample.classification(), SampleClassification::Active);
    assert_eq!(sample.completion(), CompletionStatus::Running);
    assert_eq!(sample.next_frame(), NextFrameHint::MayChange);
    let SampleValue::Value(value) = sample.value() else {
        panic!("expected numeric sample, got {sample:?}");
    };
    assert_eq!(value.property().as_str(), "opacity");
    let InterpolableValue::Number(actual) = value.value() else {
        panic!("expected numeric value, got {value:?}");
    };
    // The following segment's midpoint is (110 + 130) / 2 = 120.
    assert!((actual.value() - 120.0).abs() < 1e-9);

    drop(track);
    assert!(first_weak.upgrade().is_some());
    assert!(later_weak.upgrade().is_none());
    drop(samples);
    assert!(first_weak.upgrade().is_none());
    assert!(later_weak.upgrade().is_none());
}

#[test]
fn interleaved_clone_samples_on_another_thread_outlive_both_tracks() {
    for include_third_property in [false, true] {
        // A, A, B, A, B and A, A, B, A, C, B interrupt two property runs.
        // segment() independently allocates each repeated name.
        let mut segments = vec![
            segment("opacity", 0.0, 0.25, 10.0, 30.0),
            segment("opacity", 0.25, 0.5, 110.0, 130.0),
            segment("width", 0.0, 0.5, 200.0, 220.0),
            segment("opacity", 0.5, 1.0, 310.0, 350.0),
        ];
        if include_third_property {
            segments.push(segment("height", 0.0, 1.0, 400.0, 480.0));
        }
        segments.push(segment("width", 0.5, 1.0, 500.0, 560.0));
        let original_segments = segments.clone();
        let original = construct(segments).unwrap();
        let cloned = original.clone();
        drop(original);

        let samples = std::thread::spawn(move || {
            assert_eq!(cloned.segments(), original_segments.as_slice());
            drop(original_segments);
            let samples = cloned.sample(
                ElapsedTime::from_secs(0.75).unwrap(),
                AnimationPlayState::Running,
            );
            drop(cloned);
            samples
        })
        .join()
        .unwrap();

        // Later opacity and width segments are halfway through their ranges;
        // height is three quarters through its single segment.
        let expected: &[(&str, f64)] = if include_third_property {
            &[("opacity", 330.0), ("width", 530.0), ("height", 460.0)]
        } else {
            &[("opacity", 330.0), ("width", 530.0)]
        };
        assert_eq!(samples.len(), expected.len());
        for (sample, &(name, expected)) in samples.iter().zip(expected) {
            assert_eq!(sample.classification(), SampleClassification::Active);
            assert_eq!(sample.completion(), CompletionStatus::Running);
            assert_eq!(sample.next_frame(), NextFrameHint::MayChange);
            let SampleValue::Value(value) = sample.value() else {
                panic!("expected numeric sample, got {sample:?}");
            };
            assert_eq!(value.property().as_str(), name);
            let InterpolableValue::Number(actual) = value.value() else {
                panic!("expected numeric value, got {value:?}");
            };
            assert!(
                (actual.value() - expected).abs() < 1e-9,
                "{name}: expected {expected}, got {}",
                actual.value()
            );
        }
    }
}

#[test]
fn interrupted_first_property_errors_precede_later_property_errors() {
    let cases = [
        (
            0.25,
            KeyframeError::DuplicateOffset {
                property: property("opacity"),
                offset: ratio(0.25),
            },
        ),
        (
            0.375,
            KeyframeError::OverlappingSegment {
                property: property("opacity"),
                previous_to: ratio(0.5),
                actual: ratio(0.375),
            },
        ),
        (
            0.75,
            KeyframeError::NonContiguousSegment {
                property: property("opacity"),
                expected: ratio(0.5),
                actual: ratio(0.75),
            },
        ),
    ];

    for (later_start, expected) in cases {
        let error = construct(vec![
            segment("opacity", 0.0, 0.25, 10.0, 30.0),
            segment("opacity", 0.25, 0.5, 110.0, 130.0),
            segment("width", 0.25, 0.5, 200.0, 220.0),
            segment("opacity", later_start, 1.0, 310.0, 350.0),
            segment("height", 0.0, 1.0, 400.0, 480.0),
            segment("width", 0.5, 1.0, 500.0, 560.0),
        ])
        .unwrap_err();

        // Width's missing initial offset occurs earlier in the input, but the
        // first-appearing property's complete validation determines the error.
        assert_eq!(error, expected);
    }
}

#[test]
fn single_segments_require_initial_and_final_coverage_in_that_order() {
    for (start, end, expected) in [
        (
            0.25,
            1.0,
            KeyframeError::MissingInitialOffset {
                property: property("opacity"),
            },
        ),
        (
            0.0,
            0.75,
            KeyframeError::MissingFinalOffset {
                property: property("opacity"),
            },
        ),
        (
            0.25,
            0.75,
            KeyframeError::MissingInitialOffset {
                property: property("opacity"),
            },
        ),
    ] {
        assert_eq!(
            construct(vec![segment("opacity", start, end, 10.0, 30.0)]).unwrap_err(),
            expected
        );
    }
}

#[test]
fn signed_zero_is_initial_coverage_and_a_duplicate_even_after_an_interruption() {
    let track = track_preserving_segments(vec![segment("opacity", -0.0, 1.0, 10.0, 30.0)]);
    assert_samples(&track, 0.0, &[("opacity", 10.0)]);

    for (first_zero, repeated_zero) in [(-0.0, 0.0), (0.0, -0.0)] {
        for interrupted in [false, true] {
            let mut segments = vec![
                segment("opacity", first_zero, 0.25, 10.0, 30.0),
                segment("opacity", 0.25, 0.5, 110.0, 130.0),
            ];
            if interrupted {
                segments.push(segment("width", 0.0, 1.0, 200.0, 220.0));
            }
            segments.push(segment("opacity", repeated_zero, 1.0, 310.0, 350.0));

            assert_eq!(
                construct(segments).unwrap_err(),
                KeyframeError::DuplicateOffset {
                    property: property("opacity"),
                    offset: ratio(repeated_zero),
                }
            );
        }
    }
}

fn construct(segments: Vec<KeyframeSegment>) -> Result<KeyframeTrack, KeyframeError> {
    let timing = KeyframeTiming::try_new(
        AnimationDelay::from_secs(0.0).unwrap(),
        AnimationDuration::from_secs(1.0).unwrap(),
        IterationCount::finite(1.0).unwrap(),
        AnimationDirection::Normal,
        FillMode::Both,
    )
    .unwrap();
    KeyframeTrack::new(
        KeyframeAnimationId::new("construction").unwrap(),
        timing,
        segments,
    )
}

fn track_preserving_segments(segments: Vec<KeyframeSegment>) -> KeyframeTrack {
    let original = segments.clone();
    let track = construct(segments).unwrap();
    assert_eq!(track.segments(), original.as_slice());
    track
}

fn segment(name: &str, start: f64, end: f64, from: f64, to: f64) -> KeyframeSegment {
    segment_with_property(property(name), start, end, from, to)
}

fn segment_with_property(
    property: PropertyKey,
    start: f64,
    end: f64,
    from: f64,
    to: f64,
) -> KeyframeSegment {
    KeyframeSegment::new(
        property,
        ratio(start),
        ratio(end),
        InterpolableValue::number(InterpolableNumber::new(from).unwrap()),
        InterpolableValue::number(InterpolableNumber::new(to).unwrap()),
        Easing::linear(),
    )
    .unwrap()
}

fn property(name: &str) -> PropertyKey {
    // Repeated names are constructed from distinct owned inputs, never cloned
    // keys: grouping must use property content rather than shared identity.
    PropertyKey::new(name.to_owned()).unwrap()
}

fn ratio(value: f64) -> UnitRatio {
    UnitRatio::new(value).unwrap()
}

fn assert_samples(track: &KeyframeTrack, elapsed: f64, expected: &[(&str, f64)]) {
    let samples = track.sample(
        ElapsedTime::from_secs(elapsed).unwrap(),
        AnimationPlayState::Running,
    );
    assert_eq!(samples.len(), expected.len());
    let state = if elapsed == 1.0 {
        (
            SampleClassification::Filling,
            CompletionStatus::Finished,
            NextFrameHint::StableUntilExternalInput,
        )
    } else {
        (
            SampleClassification::Active,
            CompletionStatus::Running,
            NextFrameHint::MayChange,
        )
    };

    for (sample, &(name, expected)) in samples.iter().zip(expected) {
        assert_eq!(
            (
                sample.classification(),
                sample.completion(),
                sample.next_frame()
            ),
            state
        );
        let SampleValue::Value(value) = sample.value() else {
            panic!("expected numeric sample, got {sample:?}");
        };
        assert_eq!(value.property().as_str(), name);
        let InterpolableValue::Number(actual) = value.value() else {
            panic!("expected numeric value, got {value:?}");
        };
        assert!(
            (actual.value() - expected).abs() < 1e-9,
            "{name} at {elapsed}: expected {expected}, got {}",
            actual.value()
        );
    }
}
