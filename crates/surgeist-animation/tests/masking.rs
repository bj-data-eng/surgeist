#![forbid(unsafe_code)]

use surgeist_animation::computed::{
    ComputedComponent, ComputedDimension, ComputedNumeric, ComputedValue, DimensionKind,
};
use surgeist_animation::easing::Easing;
use surgeist_animation::interpolation::{DiscreteValue, InterpolableValue, PropertyKey};
use surgeist_animation::keyframe::{
    KeyframeAnimationId, KeyframeSegment, KeyframeTiming, KeyframeTrack,
};
use surgeist_animation::progress::UnitRatio;
use surgeist_animation::sample::{SampleValue, SampledPropertyResult};
use surgeist_animation::time::{AnimationDelay, AnimationDuration, ElapsedTime};
use surgeist_animation::timing::{
    AnimationDirection, AnimationPlayState, FillMode, IterationCount,
};
use surgeist_animation::transition::{TransitionTrack, TransitionTrackId};

// Masking 1's property tables select discrete animation even for the numeric
// mask-border values whose value definitions delegate to Backgrounds 3.
#[test]
fn discrete_masking_properties_switch_complete_values_at_the_midpoint() {
    let cases = [
        ("mask-image", "none, url(a.svg)", "url(b.svg)"),
        ("mask-mode", "alpha, match-source", "luminance"),
        ("mask-repeat", "repeat no-repeat, round", "space"),
        ("mask-clip", "content-box, no-clip", "stroke-box"),
        ("mask-origin", "padding-box, fill-box", "view-box"),
        ("mask-composite", "add, intersect", "exclude"),
        ("mask-border-source", "none", "url(border.svg)"),
        ("mask-border-mode", "alpha", "luminance"),
        ("mask-border-slice", "10 20 30 40 fill", "50 60 70 80"),
        (
            "mask-border-width",
            "2px 4px 6px 8px",
            "10px 12px 14px 16px",
        ),
        ("mask-border-outset", "1 2 3 4", "5 6 7 8"),
        ("mask-border-repeat", "stretch round", "repeat space"),
        ("clip-rule", "nonzero", "evenodd"),
        ("mask-type", "luminance", "alpha"),
    ];
    for (property, from, to) in cases {
        let from = keyword(from);
        let to = keyword(to);
        for p in [0.0, 0.25, 0.5_f64.next_down(), 0.5, 0.75] {
            assert_samples(
                property,
                from.clone(),
                to.clone(),
                p,
                if p < 0.5 { &from } else { &to },
            );
        }
    }
}

#[test]
fn legacy_clip_rectangles_interpolate_all_four_signed_offsets() {
    let from = rect([Some(0.0), Some(20.0), Some(30.0), Some(-10.0)]);
    let to = rect([Some(10.0), Some(40.0), Some(50.0), Some(-2.0)]);
    assert_samples(
        "clip",
        from,
        to,
        0.5,
        &rect([Some(5.0), Some(30.0), Some(40.0), Some(-6.0)]),
    );
}

#[test]
fn matching_auto_edges_stay_symbolic_while_other_clip_offsets_interpolate() {
    assert_samples(
        "clip",
        rect([None, Some(20.0), Some(30.0), Some(-10.0)]),
        rect([None, Some(40.0), Some(50.0), Some(-2.0)]),
        0.5,
        &rect([None, Some(30.0), Some(40.0), Some(-6.0)]),
    );
}

#[test]
fn a_clip_auto_conflict_switches_the_whole_rectangle_without_resolving_geometry() {
    let cases = [
        (
            rect([None, Some(20.0), Some(30.0), Some(-10.0)]),
            rect([Some(0.0), Some(40.0), Some(50.0), Some(-2.0)]),
        ),
        (
            InterpolableValue::computed(ComputedValue::single(token("auto"))),
            rect([None; 4]),
        ),
    ];
    for (from, to) in cases {
        assert_samples("clip", from.clone(), to.clone(), 0.5_f64.next_down(), &from);
        assert_samples("clip", from, to.clone(), 0.5, &to);
    }
}

fn token(value: &str) -> ComputedComponent {
    ComputedComponent::Discrete(DiscreteValue::new(value).unwrap())
}
fn px(value: f64) -> ComputedNumeric {
    ComputedNumeric::dimension(ComputedDimension::new(DimensionKind::Length, value).unwrap())
}
fn rect(edges: [Option<f64>; 4]) -> InterpolableValue {
    InterpolableValue::computed(
        ComputedValue::new(
            edges
                .into_iter()
                .map(|edge| {
                    edge.map_or_else(
                        || token("auto"),
                        |value| ComputedComponent::Numeric(px(value)),
                    )
                })
                .collect(),
        )
        .unwrap(),
    )
}
fn keyword(value: &str) -> InterpolableValue {
    InterpolableValue::discrete(DiscreteValue::new(value).unwrap())
}
fn assert_value(sample: &SampledPropertyResult, property: &str, expected: &InterpolableValue) {
    let SampleValue::Value(value) = sample.value() else {
        panic!("expected sampled value: {sample:?}")
    };
    assert_eq!(value.property().as_str(), property);
    assert_eq!(value.value(), expected);
}
fn tracks(
    property: &str,
    from: InterpolableValue,
    to: InterpolableValue,
    easing: Easing,
) -> (KeyframeTrack, TransitionTrack) {
    let property = PropertyKey::new(property).unwrap();
    let delay = AnimationDelay::from_secs(0.0).unwrap();
    let duration = AnimationDuration::from_secs(1.0).unwrap();
    let transition = TransitionTrack::new(
        TransitionTrackId::new(1),
        property.clone(),
        from.clone(),
        to.clone(),
        duration,
        delay,
        easing.clone(),
    )
    .unwrap();
    let segment = KeyframeSegment::new(
        property,
        UnitRatio::new(0.0).unwrap(),
        UnitRatio::new(1.0).unwrap(),
        from,
        to,
        easing,
    )
    .unwrap();
    let timing = KeyframeTiming::try_new(
        delay,
        duration,
        IterationCount::finite(1.0).unwrap(),
        AnimationDirection::Normal,
        FillMode::Both,
    )
    .unwrap();
    (
        KeyframeTrack::new(
            KeyframeAnimationId::new("mask").unwrap(),
            timing,
            vec![segment],
        )
        .unwrap(),
        transition,
    )
}
fn assert_samples(
    property: &str,
    from: InterpolableValue,
    to: InterpolableValue,
    p: f64,
    expected: &InterpolableValue,
) {
    let (track, transition) = tracks(property, from, to, Easing::linear());
    let time = ElapsedTime::from_secs(p).unwrap();
    let samples = track.sample(time, AnimationPlayState::Running);
    assert_value(&samples[0], property, expected);
    assert_value(
        &transition.sample(time, AnimationPlayState::Running),
        property,
        expected,
    );
}
