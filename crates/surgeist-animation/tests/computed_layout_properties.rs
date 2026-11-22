use surgeist_animation::computed::{
    ComputedComponent, ComputedDimension, ComputedInteger, ComputedNumeric, ComputedValue,
    DimensionKind, DimensionPercentage,
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

#[test]
fn order_uses_computed_integer_rounding() {
    let track = track([("order", integer(-3.0), integer(0.0))]);

    assert_frame(&track, 0.25, [("order", integer(-2.0))]);
    assert_frame(&track, 0.5, [("order", integer(-1.0))]);
}

#[test]
fn visibility_uses_the_discrete_type_for_visible_hidden_and_collapse() {
    for (from, to) in [("visible", "collapse"), ("hidden", "visible")] {
        let track = track([("visibility", keyword(from), keyword(to))]);

        assert_frame(&track, 0.5_f64.next_down(), [("visibility", keyword(from))]);
        assert_frame(&track, 0.5, [("visibility", keyword(to))]);
    }
}

#[test]
fn position_switches_while_inset_longhands_use_their_computed_types() {
    let track = track([
        ("position", keyword("absolute"), keyword("sticky")),
        ("top", length(16.0), percentage(0.5)),
        ("right", computed_keyword("auto"), length(8.0)),
        ("bottom", percentage(0.125), percentage(0.375)),
        ("left", length(-2.0), length(6.0)),
    ]);

    assert_frame(
        &track,
        0.25,
        [
            ("position", keyword("absolute")),
            ("top", length_percentage(12.0, 0.125)),
            ("right", computed_keyword("auto")),
            ("bottom", percentage(0.1875)),
            ("left", length(0.0)),
        ],
    );
    assert_frame(
        &track,
        0.5,
        [
            ("position", keyword("sticky")),
            ("top", length_percentage(8.0, 0.25)),
            ("right", length(8.0)),
            ("bottom", percentage(0.25)),
            ("left", length(2.0)),
        ],
    );
}

#[test]
fn expanded_logical_inset_longhands_keep_their_own_values() {
    let track = track([
        ("inset-block-start", length(16.0), percentage(0.5)),
        ("inset-block-end", length(8.0), computed_keyword("auto")),
        ("inset-inline-start", length(-16.0), length(-8.0)),
        ("inset-inline-end", percentage(0.125), percentage(0.375)),
    ]);

    assert_frame(
        &track,
        0.5,
        [
            ("inset-block-start", length_percentage(8.0, 0.25)),
            ("inset-block-end", computed_keyword("auto")),
            ("inset-inline-start", length(-12.0)),
            ("inset-inline-end", percentage(0.25)),
        ],
    );
}

#[test]
fn break_and_decoration_keywords_switch_discretely() {
    let endpoints = [
        ("break-before", "auto", "page"),
        ("break-after", "avoid-page", "column"),
        ("break-inside", "auto", "avoid"),
        ("box-decoration-break", "slice", "clone"),
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
fn widows_and_orphans_use_computed_integer_interpolation() {
    let track = track([
        ("widows", integer(2.0), integer(6.0)),
        ("orphans", integer(3.0), integer(4.0)),
    ]);

    assert_frame(
        &track,
        0.25,
        [("widows", integer(3.0)), ("orphans", integer(3.0))],
    );
    assert_frame(
        &track,
        0.5,
        [("widows", integer(4.0)), ("orphans", integer(4.0))],
    );
}

#[test]
fn generated_content_and_bookmark_payloads_are_discrete() {
    let endpoints = [
        ("content", "none", r#""Heading""#),
        ("quotes", "none", r#""«" "»""#),
        ("string-set", "none", r#"chapter "Heading""#),
        ("bookmark-label", "content(text)", r#""Overview""#),
        ("bookmark-state", "open", "closed"),
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
fn bookmark_levels_interpolate_integers_and_switch_none_discretely() {
    let numbered = track([("bookmark-level", integer(1.0), integer(4.0))]);
    assert_frame(&numbered, 0.25, [("bookmark-level", integer(2.0))]);
    assert_frame(&numbered, 0.5, [("bookmark-level", integer(3.0))]);

    let from_none = track([("bookmark-level", computed_keyword("none"), integer(4.0))]);
    assert_frame(
        &from_none,
        0.5_f64.next_down(),
        [("bookmark-level", computed_keyword("none"))],
    );
    assert_frame(&from_none, 0.5, [("bookmark-level", integer(4.0))]);
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

fn integer(value: f64) -> InterpolableValue {
    numeric(ComputedNumeric::integer(
        ComputedInteger::new(value).unwrap(),
    ))
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
