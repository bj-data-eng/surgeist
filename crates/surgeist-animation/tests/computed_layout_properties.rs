use surgeist_animation::computed::{
    ComputedComponent, ComputedDimension, ComputedNumeric, ComputedValue, DimensionKind,
    DimensionPercentage,
};
use surgeist_animation::easing::Easing;
use surgeist_animation::interpolation::{
    DiscreteValue, InterpolableNumber, InterpolableValue, PropertyKey,
};
use surgeist_animation::keyframe::{
    KeyframeAnimationId, KeyframeSegment, KeyframeTiming, KeyframeTrack,
};
use surgeist_animation::progress::UnitRatio;
use surgeist_animation::sample::{SampleClassification, SampleValue, SampledProperty};
use surgeist_animation::time::{AnimationDelay, AnimationDuration, ElapsedTime};
use surgeist_animation::timing::{
    AnimationDirection, AnimationPlayState, FillMode, IterationCount,
};

#[test]
fn flex_longhands_sample_factors_and_mixed_basis_independently() {
    // Style has already expanded flex into its three longhands.
    let track = track([
        ("flex-grow", computed_number(1.0), computed_number(5.0)),
        ("flex-shrink", number(6.0), number(2.0)),
        ("flex-basis", length(20.0), percentage(0.4)),
    ]);

    assert_frame(
        &track,
        0.25,
        [
            ("flex-grow", computed_number(2.0)),
            ("flex-shrink", number(5.0)),
            ("flex-basis", length_percentage(15.0, 0.1)),
        ],
    );
    assert_frame(
        &track,
        0.5,
        [
            ("flex-grow", computed_number(3.0)),
            ("flex-shrink", number(4.0)),
            ("flex-basis", length_percentage(10.0, 0.2)),
        ],
    );
}

#[test]
fn flex_basis_keyword_and_flow_keywords_switch_at_the_midpoint() {
    let track = track([
        ("flex-basis", computed_keyword("content"), length(12.0)),
        ("flex-direction", keyword("row"), keyword("column-reverse")),
        ("flex-wrap", keyword("nowrap"), keyword("wrap-reverse")),
    ]);

    assert_frame(
        &track,
        0.5_f64.next_down(),
        [
            ("flex-basis", computed_keyword("content")),
            ("flex-direction", keyword("row")),
            ("flex-wrap", keyword("nowrap")),
        ],
    );
    assert_frame(
        &track,
        0.5,
        [
            ("flex-basis", length(12.0)),
            ("flex-direction", keyword("column-reverse")),
            ("flex-wrap", keyword("wrap-reverse")),
        ],
    );
}

#[test]
fn alignment_content_self_and_items_longhands_switch_as_computed_keywords() {
    // Each place shorthand contributes its paired align and justify longhands.
    let endpoints = [
        ("align-content", "space-between", "space-around"),
        ("justify-content", "flex-start", "space-evenly"),
        ("align-self", "start", "self-end"),
        ("justify-self", "center", "stretch"),
        ("align-items", "baseline", "stretch"),
        ("justify-items", "start", "end"),
    ];
    let track = track(endpoints.map(|(property, from, to)| (property, keyword(from), keyword(to))));

    assert_frame(
        &track,
        0.5_f64.next_down(),
        endpoints.map(|(property, from, _)| (property, keyword(from))),
    );
    assert_frame(
        &track,
        0.5,
        endpoints.map(|(property, _, to)| (property, keyword(to))),
    );
}

#[test]
fn gap_samples_keep_unresolved_percentages_and_switch_normal_discretely() {
    let track = track([
        ("row-gap", computed_keyword("normal"), length(12.0)),
        ("column-gap", length(8.0), percentage(0.25)),
    ]);

    assert_frame(
        &track,
        0.25,
        [
            ("row-gap", computed_keyword("normal")),
            ("column-gap", length_percentage(6.0, 0.0625)),
        ],
    );
    assert_frame(
        &track,
        0.5,
        [
            ("row-gap", length(12.0)),
            ("column-gap", length_percentage(4.0, 0.125)),
        ],
    );
}

fn track<const N: usize>(
    endpoints: [(&str, InterpolableValue, InterpolableValue); N],
) -> KeyframeTrack {
    let timing = KeyframeTiming::try_new(
        AnimationDelay::from_secs(0.0).unwrap(),
        AnimationDuration::from_secs(1.0).unwrap(),
        IterationCount::finite(1.0).unwrap(),
        AnimationDirection::Normal,
        FillMode::None,
    )
    .unwrap();
    let segments = endpoints
        .into_iter()
        .map(|(property, from, to)| {
            KeyframeSegment::new(
                PropertyKey::new(property).unwrap(),
                UnitRatio::new(0.0).unwrap(),
                UnitRatio::new(1.0).unwrap(),
                from,
                to,
                Easing::linear(),
            )
            .unwrap()
        })
        .collect();
    KeyframeTrack::new(
        KeyframeAnimationId::new("computed-layout-properties").unwrap(),
        timing,
        segments,
    )
    .unwrap()
}

fn assert_frame<const N: usize>(
    track: &KeyframeTrack,
    elapsed: f64,
    expected: [(&str, InterpolableValue); N],
) {
    let frame = track.sample(
        ElapsedTime::from_secs(elapsed).unwrap(),
        AnimationPlayState::Running,
    );
    assert_eq!(frame.len(), expected.len());
    for (sample, (property, value)) in frame.iter().zip(expected) {
        assert_eq!(sample.classification(), SampleClassification::Active);
        assert_eq!(
            sample.value(),
            &SampleValue::Value(SampledProperty::new(
                PropertyKey::from_text(property).unwrap(),
                value
            )),
            "{property} at {elapsed}"
        );
    }
}

fn number(value: f64) -> InterpolableValue {
    InterpolableValue::number(InterpolableNumber::new(value).unwrap())
}

fn computed_number(value: f64) -> InterpolableValue {
    numeric(ComputedNumeric::number(value).unwrap())
}

fn length(value: f64) -> InterpolableValue {
    numeric(ComputedNumeric::dimension(
        ComputedDimension::new(DimensionKind::Length, value).unwrap(),
    ))
}

fn percentage(ratio: f64) -> InterpolableValue {
    numeric(ComputedNumeric::percentage(ratio).unwrap())
}

fn length_percentage(pixels: f64, ratio: f64) -> InterpolableValue {
    numeric(ComputedNumeric::mix(
        DimensionPercentage::new(
            ComputedDimension::new(DimensionKind::Length, pixels).unwrap(),
            ratio,
        )
        .unwrap(),
    ))
}

fn numeric(value: ComputedNumeric) -> InterpolableValue {
    InterpolableValue::computed(ComputedValue::single(ComputedComponent::Numeric(value)))
}

fn keyword(value: &str) -> InterpolableValue {
    InterpolableValue::discrete(DiscreteValue::from_text(value).unwrap())
}

fn computed_keyword(value: &str) -> InterpolableValue {
    InterpolableValue::computed(ComputedValue::single(ComputedComponent::Discrete(
        DiscreteValue::from_text(value).unwrap(),
    )))
}
