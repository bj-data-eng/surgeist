#![forbid(unsafe_code)]

use surgeist_animation::interpolation::{
    ColorInterpolationSpace, InterpolableColor, InterpolableValue, InterpolationOutcome,
    InterpolationPair, InterpolationProgress, PropertyKey,
};
use surgeist_animation::progress::UnitRatio;
use surgeist_animation::shadow::{
    ShadowErrorKind, ShadowGeometry, ShadowInput, ShadowKind, ShadowList,
};

fn color(space: ColorInterpolationSpace, red: f64, alpha: f64) -> InterpolableColor {
    InterpolableColor::from_straight_components(space, red, 0., 0., UnitRatio::new(alpha).unwrap())
        .unwrap()
}
fn input(kind: ShadowKind, geometry: [f64; 4], red: f64, inset: bool) -> ShadowInput {
    ShadowInput::new(
        kind,
        ShadowGeometry::new(geometry).unwrap(),
        color(ColorInterpolationSpace::Srgb, red, 1.),
        inset,
    )
    .unwrap()
}
fn list(kind: ShadowKind, entries: Vec<ShadowInput>) -> ShadowList {
    ShadowList::new(kind, entries).unwrap()
}
fn at(a: &ShadowList, b: &ShadowList, p: f64) -> ShadowList {
    a.interpolate(b, InterpolationProgress::new(p).unwrap())
        .unwrap()
}

#[test]
fn input_ranges_and_box_text_shapes_are_guarded_before_sampling() {
    for bad in [f64::NAN, f64::INFINITY, f64::NEG_INFINITY] {
        for i in 0..4 {
            let mut values = [0.; 4];
            values[i] = bad;
            assert_eq!(
                ShadowGeometry::new(values).unwrap_err().kind(),
                ShadowErrorKind::NonFinite
            );
        }
    }
    let c = color(ColorInterpolationSpace::Srgb, 1., 1.);
    let negative_blur = ShadowGeometry::new([0., 0., -1., 0.]).unwrap();
    assert_eq!(
        ShadowInput::new(ShadowKind::Box, negative_blur, c, false)
            .unwrap_err()
            .kind(),
        ShadowErrorKind::NegativeInput
    );
    let negative_spread = ShadowGeometry::new([0., 0., 0., -1.]).unwrap();
    assert!(ShadowInput::new(ShadowKind::Box, negative_spread, c, true).is_ok());
    assert_eq!(
        ShadowInput::new(ShadowKind::Text, negative_spread, c, true)
            .unwrap_err()
            .kind(),
        ShadowErrorKind::NegativeInput
    );
    assert_eq!(
        ShadowList::new(
            ShadowKind::Box,
            vec![input(ShadowKind::Text, [0.; 4], 1., true)]
        )
        .unwrap_err()
        .kind(),
        ShadowErrorKind::MismatchedKinds
    );
}

#[test]
fn empty_lists_pad_transparent_zero_geometry_with_the_matching_inset() {
    let a = list(ShadowKind::Text, vec![]);
    let b = list(
        ShadowKind::Text,
        vec![input(ShadowKind::Text, [8., -4., 6., 2.], 1., true)],
    );
    let result = at(&a, &b, 0.25);
    assert_eq!(result.shadows().len(), 1);
    let shadow = &result.shadows()[0];
    assert_eq!(shadow.geometry().components(), [2., -1., 1.5, 0.5]);
    assert!(shadow.inset());
    assert_eq!(shadow.color().premultiplied_components(), (0.25, 0., 0.));
    assert_eq!(shadow.color().alpha(), 0.25);
    assert_eq!(a.shadows().len(), 0);
}

#[test]
fn unequal_lists_append_blanks_in_order_without_repetition() {
    let a = list(
        ShadowKind::Box,
        vec![
            input(ShadowKind::Box, [2., 0., 0., 0.], 1., false),
            input(ShadowKind::Box, [4., 0., 0., 0.], 1., true),
        ],
    );
    let b = list(
        ShadowKind::Box,
        vec![
            input(ShadowKind::Box, [6., 0., 0., 0.], 1., false),
            input(ShadowKind::Box, [8., 0., 0., 0.], 1., true),
            input(ShadowKind::Box, [10., 0., 0., 0.], 1., true),
        ],
    );
    let result = at(&a, &b, 0.5);
    assert_eq!(
        result
            .shadows()
            .iter()
            .map(|s| s.geometry().components()[0])
            .collect::<Vec<_>>(),
        [4., 6., 5.]
    );
    assert_eq!(result.shadows()[2].color().alpha(), 0.5);
    assert!(result.shadows()[2].inset());
    assert_eq!(at(&a, &b, 0.).shadows().len(), 3);
}

#[test]
fn any_inset_mismatch_switches_the_entire_original_list_at_midpoint() {
    let a = list(
        ShadowKind::Box,
        vec![
            input(ShadowKind::Box, [2., 0., 0., 0.], 1., false),
            input(ShadowKind::Box, [4., 0., 0., 0.], 1., true),
        ],
    );
    let b = list(
        ShadowKind::Box,
        vec![
            input(ShadowKind::Box, [6., 0., 0., 0.], 1., false),
            input(ShadowKind::Box, [8., 0., 0., 0.], 1., false),
            input(ShadowKind::Box, [10., 0., 0., 0.], 1., false),
        ],
    );
    assert_eq!(at(&a, &b, 0.5_f64.next_down()), a);
    assert_eq!(at(&a, &b, 0.5), b);
    assert_eq!(a.accumulated_with(&b).unwrap(), b);
}

#[test]
fn addition_concatenates_underlying_first_without_inset_matching() {
    let a = list(
        ShadowKind::Box,
        vec![input(ShadowKind::Box, [2., 0., 0., 0.], 1., true)],
    );
    let b = list(
        ShadowKind::Box,
        vec![input(ShadowKind::Box, [6., 0., 0., 0.], 1., false)],
    );
    let result = a.added_to(&b).unwrap();
    assert_eq!(result.shadows(), &[a.shadows()[0], b.shadows()[0]]);
    assert_ne!(result, b.added_to(&a).unwrap());
}

#[test]
fn accumulation_adds_geometry_but_selects_the_effect_color() {
    let a = list(
        ShadowKind::Box,
        vec![
            input(ShadowKind::Box, [2., 4., 6., -1.], 0.25, false),
            input(ShadowKind::Box, [10., 0., 0., 0.], 1., true),
        ],
    );
    let b = list(
        ShadowKind::Box,
        vec![input(ShadowKind::Box, [3., -2., 1., 2.], 0.75, false)],
    );
    let result = a.accumulated_with(&b).unwrap();
    assert_eq!(
        result.shadows()[0].geometry().components(),
        [5., 2., 7., 1.]
    );
    assert_eq!(result.shadows()[0].color(), b.shadows()[0].color());
    assert_eq!(
        result.shadows()[1].geometry().components(),
        [10., 0., 0., 0.]
    );
    assert_eq!(result.shadows()[1].color().alpha(), 0.);
    assert!(result.shadows()[1].inset());
}

#[test]
fn extrapolated_geometry_and_alpha_can_be_reused_without_authored_clamping() {
    let a = list(ShadowKind::Text, vec![]);
    let b = list(
        ShadowKind::Text,
        vec![input(ShadowKind::Text, [8., 0., 6., 2.], 1., false)],
    );
    let result = at(&a, &b, -0.5);
    assert_eq!(
        result.shadows()[0].geometry().components(),
        [-4., 0., -3., -1.]
    );
    assert_eq!(result.shadows()[0].color().alpha(), -0.5);
    assert_eq!(at(&result, &result, 0.5), result);
}

#[test]
fn color_spaces_and_list_kinds_are_checked_without_contextual_conversion() {
    let a = list(
        ShadowKind::Box,
        vec![input(ShadowKind::Box, [0.; 4], 1., false)],
    );
    let b = list(
        ShadowKind::Box,
        vec![
            ShadowInput::new(
                ShadowKind::Box,
                ShadowGeometry::new([0.; 4]).unwrap(),
                color(ColorInterpolationSpace::Oklab, 1., 1.),
                false,
            )
            .unwrap(),
        ],
    );
    assert_eq!(
        a.interpolate(&b, InterpolationProgress::new(0.5).unwrap())
            .unwrap_err()
            .kind(),
        ShadowErrorKind::MismatchedColorSpaces
    );
    assert_eq!(
        a.accumulated_with(&b).unwrap().shadows()[0].color(),
        b.shadows()[0].color()
    );
    let text = list(ShadowKind::Text, vec![]);
    assert_eq!(
        a.added_to(&text).unwrap_err().kind(),
        ShadowErrorKind::MismatchedKinds
    );
}

#[test]
fn the_normal_sampler_reports_whole_value_failures_and_later_recovers() {
    let a = list(
        ShadowKind::Box,
        vec![input(ShadowKind::Box, [0.; 4], 1., false)],
    );
    let b = list(
        ShadowKind::Box,
        vec![input(ShadowKind::Box, [f64::MAX, 0., 0., 0.], 1., false)],
    );
    let pair = InterpolationPair::new(
        PropertyKey::new("box-shadow").unwrap(),
        InterpolableValue::shadow(a),
        InterpolableValue::shadow(b.clone()),
    )
    .unwrap();
    let InterpolationOutcome::ShadowFailure(failure) =
        pair.sample(InterpolationProgress::new(2.).unwrap())
    else {
        panic!("expected whole shadow failure")
    };
    assert_eq!(failure.error().kind(), ShadowErrorKind::NonFinite);
    assert_eq!(failure.error().shadow_index(), Some(0));
    assert_eq!(failure.property().as_str(), "box-shadow");
    assert_eq!(failure.progress().value(), 2.);
    assert_eq!(
        pair.sample(InterpolationProgress::new(1.).unwrap()),
        InterpolationOutcome::Value(InterpolableValue::shadow(b))
    );
}

#[test]
fn keyframes_and_transitions_publish_the_same_premultiplied_shadow() {
    use surgeist_animation::easing::Easing;
    use surgeist_animation::keyframe::{
        KeyframeAnimationId, KeyframeSegment, KeyframeTiming, KeyframeTrack,
    };
    use surgeist_animation::sample::{SampleClassification, SampleValue};
    use surgeist_animation::time::{AnimationDelay, AnimationDuration, ElapsedTime};
    use surgeist_animation::timing::{
        AnimationDirection, AnimationPlayState, FillMode, IterationCount,
    };
    use surgeist_animation::transition::{TransitionTrack, TransitionTrackId};
    let from = InterpolableValue::shadow(list(ShadowKind::Text, vec![]));
    let to = InterpolableValue::shadow(list(
        ShadowKind::Text,
        vec![input(ShadowKind::Text, [8., -4., 6., 2.], 1., true)],
    ));
    let property = PropertyKey::new("text-shadow").unwrap();
    let delay = AnimationDelay::from_secs(0.).unwrap();
    let duration = AnimationDuration::from_secs(1.).unwrap();
    let transition = TransitionTrack::new(
        TransitionTrackId::new(1),
        property.clone(),
        from.clone(),
        to.clone(),
        duration,
        delay,
        Easing::linear(),
    )
    .unwrap();
    let timing = KeyframeTiming::try_new(
        delay,
        duration,
        IterationCount::finite(1.).unwrap(),
        AnimationDirection::Normal,
        FillMode::None,
    )
    .unwrap();
    let segment = KeyframeSegment::new(
        property,
        UnitRatio::new(0.).unwrap(),
        UnitRatio::new(1.).unwrap(),
        from,
        to,
        Easing::linear(),
    )
    .unwrap();
    let track = KeyframeTrack::new(
        KeyframeAnimationId::new("shadow").unwrap(),
        timing,
        vec![segment],
    )
    .unwrap();
    let elapsed = ElapsedTime::from_secs(0.25).unwrap();
    let samples = track.sample(elapsed, AnimationPlayState::Running);
    for sample in [
        &transition.sample(elapsed, AnimationPlayState::Running),
        &samples[0],
    ] {
        assert_eq!(sample.classification(), SampleClassification::Active);
        let SampleValue::Value(property) = sample.value() else {
            panic!("expected sampled property")
        };
        assert_eq!(property.property().as_str(), "text-shadow");
        let InterpolableValue::Shadow(shadows) = property.value() else {
            panic!("expected shadow value")
        };
        assert_eq!(
            shadows.shadows()[0].geometry().components(),
            [2., -1., 1.5, 0.5]
        );
        assert_eq!(
            shadows.shadows()[0].color().premultiplied_components(),
            (0.25, 0., 0.)
        );
        assert_eq!(shadows.shadows()[0].color().alpha(), 0.25);
    }
}

#[test]
fn compatible_extreme_geometry_uses_the_existing_checked_scalar_kernel() {
    let a = list(
        ShadowKind::Box,
        vec![input(ShadowKind::Box, [-f64::MAX, 0., 0., 0.], 1., false)],
    );
    let b = list(
        ShadowKind::Box,
        vec![input(ShadowKind::Box, [f64::MAX, 0., 0., 0.], 1., false)],
    );
    assert_eq!(
        at(&a, &b, 0.5).shadows()[0].geometry().components(),
        [0.; 4]
    );
    assert_eq!(
        b.accumulated_with(&b).unwrap_err().kind(),
        ShadowErrorKind::NonFinite
    );
}

#[test]
fn transition_overflow_publishes_a_typed_error_without_partial_shadows() {
    use surgeist_animation::easing::{CubicBezier, Easing};
    use surgeist_animation::interpolation::InterpolationError;
    use surgeist_animation::sample::{SampleClassification, SampleValue};
    use surgeist_animation::time::{AnimationDelay, AnimationDuration, ElapsedTime};
    use surgeist_animation::timing::AnimationPlayState;
    use surgeist_animation::transition::{TransitionTrack, TransitionTrackId};
    let a = list(
        ShadowKind::Box,
        vec![input(ShadowKind::Box, [0.; 4], 1., false)],
    );
    let b = list(
        ShadowKind::Box,
        vec![input(ShadowKind::Box, [f64::MAX, 0., 0., 0.], 1., false)],
    );
    let track = TransitionTrack::new(
        TransitionTrackId::new(2),
        PropertyKey::new("box-shadow").unwrap(),
        InterpolableValue::shadow(a),
        InterpolableValue::shadow(b),
        AnimationDuration::from_secs(1.).unwrap(),
        AnimationDelay::from_secs(0.).unwrap(),
        Easing::cubic_bezier(CubicBezier::new(0., 2., 1., 2.).unwrap()),
    )
    .unwrap();
    let sample = track.sample(
        ElapsedTime::from_secs(0.5).unwrap(),
        AnimationPlayState::Running,
    );
    assert_eq!(sample.classification(), SampleClassification::Failed);
    let SampleValue::Error(InterpolationError::Shadow(failure)) = sample.value() else {
        panic!("expected a typed shadow failure")
    };
    assert_eq!(failure.error().shadow_index(), Some(0));
    assert_eq!(failure.error().kind(), ShadowErrorKind::NonFinite);
    assert!(failure.progress().value() > 1.);
    assert_eq!(failure.property().as_str(), "box-shadow");
}
