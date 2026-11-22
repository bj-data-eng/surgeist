#![forbid(unsafe_code)]

use surgeist_animation::easing::{
    Easing, LinearControlPointInput, LinearEasing, StepPosition, Steps,
};
use surgeist_animation::interpolation::{InterpolableNumber, InterpolableValue, PropertyKey};
use surgeist_animation::keyframe::{
    KeyframeAnimationId, KeyframeSegment, KeyframeTiming, KeyframeTrack,
};
use surgeist_animation::progress::UnitRatio;
use surgeist_animation::sample::{SampleClassification, SampleValue, SampledPropertyResult};
use surgeist_animation::time::{AnimationDelay, AnimationDuration, ElapsedTime};
use surgeist_animation::timing::{
    AnimationDirection, AnimationPlayState, FillMode, IterationCount,
};

// CSS Animations 1 defines delay fill from the first iteration's starting
// keyframe, without applying its timing function. Non-unit easing endpoints
// make applying the function during the delay observable in both directions.
#[test]
fn delayed_forward_fill_holds_the_start_value_without_easing() {
    for direction in [AnimationDirection::Normal, AnimationDirection::Alternate] {
        let track = delayed_track(direction, FillMode::Backwards);
        let samples = sample(&track, 0.5);
        assert_eq!(samples[0].classification(), SampleClassification::Filling);
        assert_number(&samples[0], "opacity", 0.0);
    }
}

#[test]
fn delayed_reverse_fill_holds_the_end_value_without_easing() {
    for direction in [
        AnimationDirection::Reverse,
        AnimationDirection::AlternateReverse,
    ] {
        let track = delayed_track(direction, FillMode::Both);
        let samples = sample(&track, 0.5);
        assert_eq!(samples[0].classification(), SampleClassification::Filling);
        assert_number(&samples[0], "opacity", 100.0);
    }
}

#[test]
fn delay_without_backwards_fill_has_no_value() {
    let track = delayed_track(AnimationDirection::Normal, FillMode::Forwards);
    let samples = sample(&track, 0.5);
    assert!(matches!(samples[0].value(), SampleValue::None));
}

#[test]
fn delay_end_and_negative_delay_use_active_interval_easing() {
    let positive_delay = delayed_track(AnimationDirection::Normal, FillMode::Both);
    assert_number(&sample(&positive_delay, 1.0)[0], "opacity", 25.0);
    let negative_delay = track(
        timing(AnimationDirection::Normal, FillMode::Both, -0.5),
        vec![segment(
            "opacity",
            (0.0, 1.0),
            (0.0, 100.0),
            shifted_linear(),
        )],
    );
    assert_number(&sample(&negative_delay, 0.0)[0], "opacity", 50.0);
}

#[test]
fn each_property_uses_its_interval_easing_after_direction() {
    for (direction, elapsed) in [
        (AnimationDirection::Normal, 0.4375),
        (AnimationDirection::Reverse, 0.5625),
    ] {
        let track = track(
            timing(direction, FillMode::Both, 0.0),
            vec![
                segment("opacity", (0.0, 0.25), (10.0, 30.0), Easing::linear()),
                segment(
                    "opacity",
                    (0.25, 1.0),
                    (80.0, 140.0),
                    Easing::steps(Steps::new(2, StepPosition::JumpEnd).unwrap()),
                ),
                segment("scale", (0.0, 1.0), (0.0, 100.0), Easing::linear()),
            ],
        );
        let samples = sample(&track, elapsed);
        // Directed .4375 is local .25 in the opacity interval: steps holds 80.
        // The other property has no intermediate frame and remains at 43.75.
        assert_number(&samples[0], "opacity", 80.0);
        assert_number(&samples[1], "scale", 43.75);
    }
}

#[test]
fn reverse_play_traverses_the_same_asymmetric_easing_backwards() {
    let easing = Easing::linear_function(
        LinearEasing::new(vec![
            LinearControlPointInput {
                input: 0.0,
                output: 0.0,
            },
            LinearControlPointInput {
                input: 0.5,
                output: 0.25,
            },
            LinearControlPointInput {
                input: 1.0,
                output: 1.0,
            },
        ])
        .unwrap(),
    );
    let track = track(
        timing(AnimationDirection::Reverse, FillMode::Both, 0.0),
        vec![segment("opacity", (0.0, 1.0), (0.0, 100.0), easing)],
    );
    assert_number(&sample(&track, 0.25)[0], "opacity", 62.5);
    assert_number(&sample(&track, 0.75)[0], "opacity", 12.5);
}

#[test]
fn final_interval_easing_reaches_the_caller_supplied_endpoint() {
    // The caller has supplied a computed 100% endpoint for a property whose
    // final authored keyframe was at 50%. Only the 50% outgoing easing enters
    // this segment; a timing function on 100% has no outgoing interval.
    let track = track(
        timing(AnimationDirection::Normal, FillMode::Both, 0.0),
        vec![
            segment("opacity", (0.0, 0.5), (0.0, 50.0), Easing::linear()),
            segment("opacity", (0.5, 1.0), (50.0, 100.0), Easing::step_end()),
        ],
    );
    assert_number(&sample(&track, 0.75)[0], "opacity", 50.0);
    assert_number(&sample(&track, 1.0)[0], "opacity", 100.0);
}

fn shifted_linear() -> Easing {
    Easing::linear_function(
        LinearEasing::new(vec![
            LinearControlPointInput {
                input: 0.0,
                output: 0.25,
            },
            LinearControlPointInput {
                input: 1.0,
                output: 0.75,
            },
        ])
        .unwrap(),
    )
}

fn delayed_track(direction: AnimationDirection, fill: FillMode) -> KeyframeTrack {
    track(
        timing(direction, fill, 1.0),
        vec![segment(
            "opacity",
            (0.0, 1.0),
            (0.0, 100.0),
            shifted_linear(),
        )],
    )
}

fn timing(direction: AnimationDirection, fill: FillMode, delay: f64) -> KeyframeTiming {
    KeyframeTiming::try_new(
        AnimationDelay::from_secs(delay).unwrap(),
        AnimationDuration::from_secs(1.0).unwrap(),
        IterationCount::finite(1.0).unwrap(),
        direction,
        fill,
    )
    .unwrap()
}

fn segment(
    property: &str,
    offsets: (f64, f64),
    values: (f64, f64),
    easing: Easing,
) -> KeyframeSegment {
    KeyframeSegment::new(
        PropertyKey::new(property).unwrap(),
        UnitRatio::new(offsets.0).unwrap(),
        UnitRatio::new(offsets.1).unwrap(),
        InterpolableValue::number(InterpolableNumber::new(values.0).unwrap()),
        InterpolableValue::number(InterpolableNumber::new(values.1).unwrap()),
        easing,
    )
    .unwrap()
}

fn track(timing: KeyframeTiming, segments: Vec<KeyframeSegment>) -> KeyframeTrack {
    KeyframeTrack::new(KeyframeAnimationId::new("fade").unwrap(), timing, segments).unwrap()
}

fn sample(track: &KeyframeTrack, elapsed: f64) -> Vec<SampledPropertyResult> {
    track.sample(
        ElapsedTime::from_secs(elapsed).unwrap(),
        AnimationPlayState::Running,
    )
}

fn assert_number(sample: &SampledPropertyResult, property: &str, expected: f64) {
    let SampleValue::Value(value) = sample.value() else {
        panic!("expected sampled value, got {sample:?}");
    };
    assert_eq!(value.property().as_str(), property);
    let InterpolableValue::Number(number) = value.value() else {
        panic!("expected numeric value, got {value:?}");
    };
    assert_eq!(number.value(), expected);
}
