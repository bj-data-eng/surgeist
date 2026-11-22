#![forbid(unsafe_code)]

use surgeist_animation::computed::{
    ComputedComponent, ComputedDimension, ComputedNumeric, ComputedValue, DimensionKind,
};
use surgeist_animation::easing::Easing;
use surgeist_animation::interpolation::{
    DiscreteValue, InterpolableValue, InterpolationOutcome, InterpolationPair,
    InterpolationProgress, PropertyKey,
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

// Fonts 4, 2026-09-07 property tables. These are supplied effective computed
// values; Style resolves aliases and expands the font shorthand to longhands.
#[test]
fn computed_font_longhands_sample_fractional_values_through_all_public_tracks() {
    for (property, from, to, quarter, middle) in [
        (
            "font-weight",
            number(201.25),
            number(401.75),
            number(251.375),
            number(301.5),
        ),
        (
            "font-width",
            percentage(0.5),
            percentage(1.5),
            percentage(0.75),
            percentage(1.0),
        ),
        (
            "font-style",
            angle(0.0),
            angle(20.0),
            angle(5.0),
            angle(10.0),
        ),
        (
            "font-size",
            length(12.0),
            length(28.0),
            length(16.0),
            length(20.0),
        ),
        (
            "font-size-adjust",
            number(0.5),
            number(1.5),
            number(0.75),
            number(1.0),
        ),
    ] {
        let pair = pair(property, from.clone(), to.clone());
        assert_eq!(
            pair.sample(progress(0.25)),
            InterpolationOutcome::Value(quarter.clone())
        );
        assert_eq!(
            pair.sample(progress(0.5)),
            InterpolationOutcome::Value(middle.clone())
        );
        let keyframe = keyframe(property, from.clone(), to.clone());
        let transition = transition(property, from, to);
        for (elapsed, expected) in [(0.25, quarter), (0.5, middle)] {
            let samples = keyframe.sample(time(elapsed), AnimationPlayState::Running);
            assert_sample(samples[0].value(), &expected);
            assert_eq!(samples[0].classification(), SampleClassification::Active);
            let sample = transition.sample(time(elapsed), AnimationPlayState::Running);
            assert_sample(sample.value(), &expected);
        }
    }
}

#[test]
fn supplied_font_numbers_extrapolate_without_face_matching_or_final_range_clamping() {
    for (property, from, to, before, after) in [
        (
            "font-width",
            percentage(0.5),
            percentage(1.5),
            percentage(-0.5),
            percentage(2.5),
        ),
        (
            "font-size",
            length(12.0),
            length(28.0),
            length(-4.0),
            length(44.0),
        ),
        (
            "font-size-adjust",
            number(0.5),
            number(1.5),
            number(-0.5),
            number(2.5),
        ),
    ] {
        let pair = pair(property, from, to);
        assert_eq!(
            pair.sample(progress(-1.0)),
            InterpolationOutcome::Value(before)
        );
        assert_eq!(
            pair.sample(progress(2.0)),
            InterpolationOutcome::Value(after)
        );
    }
}

#[test]
fn font_style_italic_and_font_size_adjust_none_switch_whole_computed_values() {
    for (property, from, to) in [
        ("font-style", angle(0.0), computed_keyword("italic")),
        ("font-style", computed_keyword("italic"), angle(20.0)),
        ("font-size-adjust", computed_keyword("none"), number(0.75)),
    ] {
        assert_discrete(property, from, to);
    }
}

#[test]
fn every_discrete_font_family_selects_the_complete_endpoint_at_the_midpoint() {
    for (property, from, to) in [
        (
            "font-family",
            r#""First", serif"#,
            r#""Second", sans-serif"#,
        ),
        ("font-synthesis-weight", "auto", "none"),
        ("font-synthesis-style", "auto", "none"),
        ("font-synthesis-small-caps", "auto", "none"),
        ("font-synthesis-position", "auto", "none"),
        ("font-synthesis", "weight style", "small-caps position"),
        ("font-kerning", "normal", "none"),
        (
            "font-variant-ligatures",
            "common-ligatures",
            "no-common-ligatures discretionary-ligatures",
        ),
        ("font-variant-position", "sub", "super"),
        ("font-variant-caps", "small-caps", "all-small-caps"),
        (
            "font-variant-numeric",
            "lining-nums proportional-nums",
            "oldstyle-nums tabular-nums",
        ),
        (
            "font-variant-alternates",
            "historical-forms",
            "stylistic(custom)",
        ),
        (
            "font-variant-east-asian",
            "jis78 full-width",
            "jis04 proportional-width",
        ),
        (
            "font-variant",
            "small-caps oldstyle-nums",
            "all-small-caps lining-nums",
        ),
        (
            "font-feature-settings",
            r#""liga" 0, "kern" 1"#,
            r#""liga" 1, "kern" 0"#,
        ),
        ("font-language-override", "ENG", "TRK"),
        ("font-optical-sizing", "auto", "none"),
        ("font-variant-emoji", "text", "emoji"),
    ] {
        assert_discrete(property, keyword(from), keyword(to));
    }
}

fn assert_discrete(property: &str, from: InterpolableValue, to: InterpolableValue) {
    let pair = pair(property, from.clone(), to.clone());
    let keyframe = keyframe(property, from.clone(), to.clone());
    let transition = transition(property, from.clone(), to.clone());
    for (elapsed, expected) in [(0.5_f64.next_down(), from), (0.5, to)] {
        assert_eq!(
            pair.sample(progress(elapsed)),
            InterpolationOutcome::Value(expected.clone())
        );
        let samples = keyframe.sample(time(elapsed), AnimationPlayState::Running);
        assert_sample(samples[0].value(), &expected);
        assert_sample(
            transition
                .sample(time(elapsed), AnimationPlayState::Running)
                .value(),
            &expected,
        );
    }
}

fn numeric(value: ComputedNumeric) -> InterpolableValue {
    InterpolableValue::computed(ComputedValue::single(ComputedComponent::Numeric(value)))
}
fn number(value: f64) -> InterpolableValue {
    numeric(ComputedNumeric::number(value).unwrap())
}
fn percentage(value: f64) -> InterpolableValue {
    numeric(ComputedNumeric::percentage(value).unwrap())
}
fn dimension(kind: DimensionKind, value: f64) -> InterpolableValue {
    numeric(ComputedNumeric::dimension(
        ComputedDimension::new(kind, value).unwrap(),
    ))
}
fn length(value: f64) -> InterpolableValue {
    dimension(DimensionKind::Length, value)
}
fn angle(value: f64) -> InterpolableValue {
    dimension(DimensionKind::Angle, value)
}
fn keyword(value: &str) -> InterpolableValue {
    InterpolableValue::discrete(DiscreteValue::from_text(value).unwrap())
}
fn computed_keyword(value: &str) -> InterpolableValue {
    InterpolableValue::computed(ComputedValue::single(ComputedComponent::Discrete(
        DiscreteValue::from_text(value).unwrap(),
    )))
}
fn pair(property: &str, from: InterpolableValue, to: InterpolableValue) -> InterpolationPair {
    InterpolationPair::new(PropertyKey::from_text(property).unwrap(), from, to).unwrap()
}
fn keyframe(property: &str, from: InterpolableValue, to: InterpolableValue) -> KeyframeTrack {
    KeyframeTrack::new(
        KeyframeAnimationId::new("font").unwrap(),
        KeyframeTiming::try_new(
            AnimationDelay::from_secs(0.0).unwrap(),
            AnimationDuration::from_secs(1.0).unwrap(),
            IterationCount::finite(1.0).unwrap(),
            AnimationDirection::Normal,
            FillMode::Both,
        )
        .unwrap(),
        vec![
            KeyframeSegment::new(
                PropertyKey::from_text(property).unwrap(),
                UnitRatio::new(0.0).unwrap(),
                UnitRatio::new(1.0).unwrap(),
                from,
                to,
                Easing::linear(),
            )
            .unwrap(),
        ],
    )
    .unwrap()
}
fn transition(property: &str, from: InterpolableValue, to: InterpolableValue) -> TransitionTrack {
    TransitionTrack::new(
        TransitionTrackId::new(1),
        PropertyKey::from_text(property).unwrap(),
        from,
        to,
        AnimationDuration::from_secs(1.0).unwrap(),
        AnimationDelay::from_secs(0.0).unwrap(),
        Easing::linear(),
    )
    .unwrap()
}
fn progress(value: f64) -> InterpolationProgress {
    InterpolationProgress::new(value).unwrap()
}
fn time(value: f64) -> ElapsedTime {
    ElapsedTime::from_secs(value).unwrap()
}
fn assert_sample(sample: &SampleValue, expected: &InterpolableValue) {
    let SampleValue::Value(sample) = sample else {
        panic!("expected font sample")
    };
    assert_eq!(sample.value(), expected);
}
