#![forbid(unsafe_code)]

use surgeist_animation::computed::{
    ComputedComponent, ComputedDimension, ComputedNumeric, ComputedValue, DimensionKind,
};
use surgeist_animation::easing::{Easing, LinearControlPointInput, LinearEasing};
use surgeist_animation::interpolation::{
    DiscreteValue, InterpolableValue, InterpolationError, InterpolationOutcome, InterpolationPair,
    InterpolationProgress, PropertyKey,
};
use surgeist_animation::keyframe::{
    KeyframeAnimationId, KeyframeSegment, KeyframeTiming, KeyframeTrack,
};
use surgeist_animation::masking::{ClipPathValue, ClipSource, ClipSourceErrorKind};
use surgeist_animation::progress::UnitRatio;
use surgeist_animation::sample::{SampleClassification, SampleValue, SampledPropertyResult};
use surgeist_animation::shape::{
    BasicShape, Circle, CommandPath, CommandShape, Endpoint, FillRule, LengthPercentage, Radius,
    ReferenceBox, ShapeCommand, ShapeComponent, ShapeValue,
};
use surgeist_animation::time::{AnimationDelay, AnimationDuration, ElapsedTime};
use surgeist_animation::timing::{
    AnimationDirection, AnimationPlayState, FillMode, IterationCount,
};
use surgeist_animation::transition::{TransitionTrack, TransitionTrackId};

#[test]
fn opaque_clip_sources_reject_blank_and_control_inputs_and_retain_exact_identity() {
    for value in ["", " ", "\u{2003}"] {
        assert_eq!(
            ClipSource::new(value).unwrap_err().kind(),
            ClipSourceErrorKind::Blank
        );
    }
    for value in ["a\nb", "a\u{0}b", "a\u{85}b"] {
        assert!(matches!(
            ClipSource::new(value).unwrap_err().kind(),
            ClipSourceErrorKind::ControlCharacter { .. }
        ));
    }
    // This is an opaque already resolved identity, not an authored URL parser.
    let source = ClipSource::new("https://example.test/a.svg#Clip").unwrap();
    assert_eq!(source.as_str(), "https://example.test/a.svg#Clip");
    assert_ne!(
        source,
        ClipSource::new("https://example.test/a.svg#clip").unwrap()
    );
    assert_eq!(
        ClipSource::new(" source identity ").unwrap().as_str(),
        " source identity "
    );
}

#[test]
fn clip_path_branch_conflicts_switch_complete_values_through_public_samplers() {
    let values = [
        ClipPathValue::None,
        ClipPathValue::Source(ClipSource::new("https://example.test/a.svg#clip").unwrap()),
        ClipPathValue::Source(ClipSource::new("https://example.test/b.svg#clip").unwrap()),
        ClipPathValue::Box(ReferenceBox::BorderBox),
        ClipPathValue::Box(ReferenceBox::FillBox),
        circle(10.0, ReferenceBox::BorderBox),
    ];
    for from in &values {
        for to in &values {
            if matches!(
                (from, to),
                (ClipPathValue::Shape(_), ClipPathValue::Shape(_))
            ) {
                continue;
            }
            for p in [0.25, 0.5_f64.next_down(), 0.5, 0.75] {
                let expected = clip(if p < 0.5 { from.clone() } else { to.clone() });
                let pair = clip_pair(from.clone(), to.clone());
                assert_eq!(
                    pair.sample(progress(p)),
                    InterpolationOutcome::Value(expected.clone())
                );
                if from != to {
                    assert_samples(
                        "clip-path",
                        clip(from.clone()),
                        clip(to.clone()),
                        p,
                        &expected,
                    );
                }
            }
        }
    }
}

#[test]
fn numeric_clip_shapes_delegate_interpolation_and_preserve_reference_identity() {
    assert_samples(
        "clip-path",
        clip(circle(10.0, ReferenceBox::BorderBox)),
        clip(circle(30.0, ReferenceBox::BorderBox)),
        0.5,
        &clip(circle(20.0, ReferenceBox::BorderBox)),
    );
    let from = circle(10.0, ReferenceBox::BorderBox);
    let to = circle(30.0, ReferenceBox::PaddingBox);
    let pair = clip_pair(from.clone(), to.clone());
    for p in [-1.0, 0.5_f64.next_down(), 0.5, 2.0] {
        assert_eq!(
            pair.sample(progress(p)),
            InterpolationOutcome::Value(clip(if p < 0.5 { from.clone() } else { to.clone() }))
        );
    }
}

#[test]
fn mixed_path_and_shape_clip_values_keep_shape_output_even_at_exact_endpoints() {
    let commands = |x| vec![ShapeCommand::Line(Endpoint::To(point(x, 20.0)))];
    let from = ClipPathValue::Shape(ShapeValue::new(
        ReferenceBox::BorderBox,
        BasicShape::Path(
            CommandPath::new(point(0.0, 0.0), commands(10.0), FillRule::Nonzero).unwrap(),
        ),
    ));
    let to = ClipPathValue::Shape(ShapeValue::new(
        ReferenceBox::BorderBox,
        BasicShape::Shape(
            CommandShape::new(point(0.0, 0.0), commands(30.0), FillRule::Nonzero).unwrap(),
        ),
    ));
    for p in [0.0, 0.5, 1.0] {
        let expected = clip(ClipPathValue::Shape(ShapeValue::new(
            ReferenceBox::BorderBox,
            BasicShape::Shape(
                CommandShape::new(
                    point(0.0, 0.0),
                    commands(10.0 + 20.0 * p),
                    FillRule::Nonzero,
                )
                .unwrap(),
            ),
        )));
        assert_eq!(
            clip_pair(from.clone(), to.clone()).sample(progress(p)),
            InterpolationOutcome::Value(expected.clone())
        );
        let (track, transition) = tracks(
            "clip-path",
            clip(from.clone()),
            clip(to.clone()),
            Easing::linear(),
        );
        let time = ElapsedTime::from_secs(p).unwrap();
        assert_value(
            &track.sample(time, AnimationPlayState::Running)[0],
            "clip-path",
            &expected,
        );
        // CSS transitions hold their supplied start value through the delay
        // boundary. Once interpolation starts, the shared shape rule applies.
        let transition_expected = if p == 0.0 {
            clip(from.clone())
        } else {
            expected
        };
        let transition_sample = transition.sample(time, AnimationPlayState::Running);
        if p == 1.0 {
            assert_eq!(
                transition_sample.classification(),
                SampleClassification::Finished
            );
            assert_eq!(transition_sample.value(), &SampleValue::None);
        } else {
            assert_value(&transition_sample, "clip-path", &transition_expected);
        }
    }
}

#[test]
fn clip_shape_extrapolation_retains_finite_negative_radii_and_owned_inputs() {
    let from = circle(10.0, ReferenceBox::BorderBox);
    let to = circle(30.0, ReferenceBox::BorderBox);
    let pair = clip_pair(from.clone(), to.clone());
    let retained = pair.sample(progress(-1.0));
    assert_eq!(
        retained,
        InterpolationOutcome::Value(clip(circle(-10.0, ReferenceBox::BorderBox)))
    );
    assert_eq!(
        pair.sample(progress(1.5)),
        InterpolationOutcome::Value(clip(circle(40.0, ReferenceBox::BorderBox)))
    );
    assert_eq!(pair.from(), &clip(from));
    assert_eq!(pair.to(), &clip(to));
    assert_eq!(
        retained,
        InterpolationOutcome::Value(clip(circle(-10.0, ReferenceBox::BorderBox)))
    );
}

#[test]
fn clip_shape_failures_retain_typed_diagnostics_and_complete_endpoints_then_recover() {
    let from = circle(0.0, ReferenceBox::BorderBox);
    let to = circle(f64::MAX, ReferenceBox::BorderBox);
    let pair = clip_pair(from.clone(), to.clone());
    let InterpolationOutcome::ShapeFailure(error) = pair.sample(progress(2.0)) else {
        panic!("expected shape failure")
    };
    assert_eq!(error.property().as_str(), "clip-path");
    assert_eq!(error.progress().value(), 2.0);
    assert_eq!(error.error().component(), Some(ShapeComponent::Radius));
    assert_eq!(error.from(), &clip(from.clone()));
    assert_eq!(error.to(), &clip(to.clone()));
    assert!(std::error::Error::source(&InterpolationError::Shape(error)).is_some());
    assert_eq!(
        pair.sample(progress(0.0)),
        InterpolationOutcome::Value(clip(from))
    );
    assert_eq!(
        pair.sample(progress(1.0)),
        InterpolationOutcome::Value(clip(to))
    );
}

#[test]
fn clip_path_branch_fallback_precedes_unrelated_shape_arithmetic() {
    let from = circle(f64::MAX, ReferenceBox::BorderBox);
    assert_eq!(
        clip_pair(from, ClipPathValue::None).sample(progress(2.0)),
        InterpolationOutcome::Value(clip(ClipPathValue::None))
    );
}

#[test]
fn keyframe_and_transition_clip_failures_remain_sample_local_and_recover() {
    let from = clip(circle(0.0, ReferenceBox::BorderBox));
    let to = clip(circle(f64::MAX, ReferenceBox::BorderBox));
    let easing = Easing::linear_function(
        LinearEasing::new(vec![
            LinearControlPointInput {
                input: 0.0,
                output: 0.0,
            },
            LinearControlPointInput {
                input: 1.0,
                output: 2.0,
            },
        ])
        .unwrap(),
    );
    let (track, transition) = tracks("clip-path", from.clone(), to.clone(), easing);
    let time = ElapsedTime::from_secs(0.75).unwrap();
    let sampled = track.sample(time, AnimationPlayState::Running);
    for sample in [
        &sampled[0],
        &transition.sample(time, AnimationPlayState::Running),
    ] {
        assert_eq!(sample.classification(), SampleClassification::Failed);
        let SampleValue::Error(InterpolationError::Shape(error)) = sample.value() else {
            panic!("expected typed shape error")
        };
        assert_eq!(error.progress().value(), 1.5);
        assert_eq!(error.from(), &from);
        assert_eq!(error.to(), &to);
    }
    let zero = ElapsedTime::from_secs(0.0).unwrap();
    assert_value(
        &track.sample(zero, AnimationPlayState::Running)[0],
        "clip-path",
        &from,
    );
    assert_value(
        &transition.sample(zero, AnimationPlayState::Running),
        "clip-path",
        &from,
    );
}

fn point(x: f64, y: f64) -> surgeist_animation::computed::ComputedPosition {
    surgeist_animation::computed::ComputedPosition::new(px(x), px(y)).unwrap()
}
fn circle(radius: f64, reference: ReferenceBox) -> ClipPathValue {
    ClipPathValue::Shape(ShapeValue::new(
        reference,
        BasicShape::Circle(Circle::new(
            Radius::Numeric(LengthPercentage::new(px(radius)).unwrap()),
            point(0.0, 0.0),
        )),
    ))
}
fn clip(value: ClipPathValue) -> InterpolableValue {
    InterpolableValue::clip_path(value)
}
fn progress(value: f64) -> InterpolationProgress {
    InterpolationProgress::new(value).unwrap()
}
fn clip_pair(from: ClipPathValue, to: ClipPathValue) -> InterpolationPair {
    InterpolationPair::new(PropertyKey::new("clip-path").unwrap(), clip(from), clip(to)).unwrap()
}

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
