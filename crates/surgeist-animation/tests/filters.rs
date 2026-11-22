#![forbid(unsafe_code)]

use std::sync::Arc;
use surgeist_animation::filter::image::{FilterImage, FilterImageSample, FilterImageSource};
use surgeist_animation::filter::{
    AmountFunction as A, FilterAmountInput, FilterErrorKind, FilterInput as F, FilterList,
};
use surgeist_animation::interpolation::{
    ColorInterpolationSpace as Space, InterpolableColor, InterpolableNumber,
    InterpolablePercentage, InterpolableValue, InterpolationOutcome, InterpolationPair,
    InterpolationProgress, PropertyKey,
};
use surgeist_animation::progress::UnitRatio;
use surgeist_animation::transform::TransformAngle;

fn list(inputs: Vec<F>) -> FilterList {
    FilterList::new(inputs).unwrap()
}
fn amount(function: A, value: f64) -> F {
    F::amount(
        function,
        FilterAmountInput::Number(InterpolableNumber::new(value).unwrap()),
    )
    .unwrap()
}
fn color(space: Space) -> InterpolableColor {
    InterpolableColor::from_straight_components(space, 1., 0., 0., UnitRatio::new(1.).unwrap())
        .unwrap()
}
fn at(a: &FilterList, b: &FilterList, p: f64) -> FilterList {
    a.interpolate(b, InterpolationProgress::new(p).unwrap())
        .unwrap()
}
fn scalar(value: &FilterList, index: usize) -> f64 {
    value.functions()[index].amount().unwrap().1
}

#[test]
fn each_amount_function_uses_its_interpolation_identity_not_omitted_defaults() {
    for (function, identity) in [
        (A::Brightness, 1.),
        (A::Contrast, 1.),
        (A::Grayscale, 0.),
        (A::Invert, 0.),
        (A::Opacity, 1.),
        (A::Saturate, 1.),
        (A::Sepia, 0.),
    ] {
        // All seven omitted authored arguments compute to one in Style.
        let computed_omitted = list(vec![amount(function, 1.)]);
        let midpoint = at(&list(vec![]), &computed_omitted, 0.5);
        assert_eq!(scalar(&midpoint, 0), (identity + 1.) / 2.);
        assert_eq!(scalar(&computed_omitted, 0), 1.);
    }
}

#[test]
fn percentage_amounts_keep_extended_computed_values_until_consumer_clamping() {
    let from = list(vec![
        F::amount(
            A::Grayscale,
            FilterAmountInput::Percentage(InterpolablePercentage::new(2.).unwrap()),
        )
        .unwrap(),
    ]);
    let to = list(vec![amount(A::Grayscale, 0.)]);
    assert_eq!(scalar(&at(&from, &to, 0.25), 0), 1.5);
    assert_eq!(scalar(&at(&from, &to, 0.75), 0), 0.5);
    let extrapolated = at(&from, &to, 1.5);
    assert_eq!(scalar(&extrapolated, 0), -1.);
    assert_eq!(at(&extrapolated, &extrapolated, 0.5), extrapolated);
    let a = list(vec![amount(A::Brightness, 0.5)]);
    let b = list(vec![amount(A::Brightness, 1.5)]);
    assert_eq!(scalar(&at(&a, &b, 0.5), 0), 1.);
}

#[test]
fn blur_and_hue_use_resolved_lengths_and_unwrapped_angles() {
    let a = list(vec![
        F::blur_pixels(2.).unwrap(),
        F::hue_rotate(TransformAngle::from_degrees(0.).unwrap()),
    ]);
    let b = list(vec![
        F::blur_pixels(6.).unwrap(),
        F::hue_rotate(TransformAngle::from_degrees(720.).unwrap()),
    ]);
    let result = at(&a, &b, 0.5);
    assert_eq!(result.functions()[0].blur_pixels(), Some(4.));
    assert_eq!(
        result.functions()[1].angle().unwrap().radians(),
        std::f64::consts::TAU
    );
    let padded = at(&list(vec![]), &b, 0.5);
    assert_eq!(padded.functions()[0].blur_pixels(), Some(3.));
}

#[test]
fn singleton_drop_shadow_interpolates_sigma_and_explicit_premultiplied_color() {
    let b = list(vec![
        F::drop_shadow_pixels([8., -4.], 6., color(Space::Srgb)).unwrap(),
    ]);
    let result = at(&list(vec![]), &b, 0.25);
    let shadow = result.functions()[0].drop_shadow().unwrap();
    assert_eq!(shadow.offsets_pixels(), [2., -1.]);
    assert_eq!(shadow.standard_deviation_pixels(), 1.5);
    assert_eq!(shadow.color().premultiplied_components(), (0.25, 0., 0.));
    assert_eq!(shadow.color().alpha(), 0.25);
    let negative = at(&list(vec![]), &b, -0.5);
    assert_eq!(
        negative.functions()[0]
            .drop_shadow()
            .unwrap()
            .standard_deviation_pixels(),
        -3.
    );
    assert_eq!(at(&negative, &negative, 0.5), negative);
}

#[test]
fn unequal_lists_pad_only_the_missing_tail_with_matching_identities() {
    let a = list(vec![F::blur_pixels(2.).unwrap()]);
    let b = list(vec![F::blur_pixels(6.).unwrap(), amount(A::Brightness, 3.)]);
    let result = at(&a, &b, 0.5);
    assert_eq!(result.functions().len(), 2);
    assert_eq!(result.functions()[0].blur_pixels(), Some(4.));
    assert_eq!(scalar(&result, 1), 2.);
    assert_eq!(at(&a, &b, 0.).functions().len(), 2);
    assert_eq!(a.functions().len(), 1);
}

#[test]
fn any_url_or_kind_mismatch_switches_complete_originals_before_arithmetic() {
    let from = list(vec![
        F::blur_pixels(f64::MAX).unwrap(),
        F::url_reference(Arc::from("url(#a)")).unwrap(),
    ]);
    let to = list(vec![
        F::blur_pixels(0.).unwrap(),
        F::url_reference(Arc::from("url(#b)")).unwrap(),
    ]);
    assert_eq!(at(&from, &to, -2.), from);
    assert_eq!(at(&from, &to, 0.5), to);
    let a = list(vec![
        F::blur_pixels(f64::MAX).unwrap(),
        amount(A::Contrast, 2.),
    ]);
    let b = list(vec![F::blur_pixels(0.).unwrap(), amount(A::Brightness, 3.)]);
    assert_eq!(at(&a, &b, -2.), a);
    assert_eq!(at(&a, &b, 0.5), b);
    assert_eq!(at(&list(vec![]), &from, 0.25), list(vec![]));
}

#[test]
fn addition_concatenates_and_accumulation_selects_the_whole_effect() {
    let a = list(vec![F::blur_pixels(2.).unwrap()]);
    let b = list(vec![
        amount(A::Sepia, 0.5),
        F::url_reference(Arc::from("url(#x)")).unwrap(),
    ]);
    let result = a.added_to(&b).unwrap();
    assert_eq!(
        result.functions(),
        &[
            a.functions()[0].clone(),
            b.functions()[0].clone(),
            b.functions()[1].clone()
        ]
    );
    assert_ne!(result, b.added_to(&a).unwrap());
    assert_eq!(a.accumulated_with(&b), b);
}

#[test]
fn normalized_input_guards_reject_nonfinite_negative_and_empty_references() {
    for bad in [f64::NAN, f64::INFINITY, f64::NEG_INFINITY] {
        assert_eq!(
            F::blur_pixels(bad).unwrap_err().kind(),
            FilterErrorKind::NonFinite
        );
        assert!(F::drop_shadow_pixels([bad, 0.], 0., color(Space::Srgb)).is_err());
        assert!(F::drop_shadow_pixels([0., 0.], bad, color(Space::Srgb)).is_err());
    }
    assert_eq!(
        F::blur_pixels(-1.).unwrap_err().kind(),
        FilterErrorKind::NegativeInput
    );
    assert_eq!(
        F::drop_shadow_pixels([0., 0.], -1., color(Space::Srgb))
            .unwrap_err()
            .kind(),
        FilterErrorKind::NegativeInput
    );
    assert_eq!(
        F::amount(
            A::Opacity,
            FilterAmountInput::Number(InterpolableNumber::new(-1.).unwrap())
        )
        .unwrap_err()
        .kind(),
        FilterErrorKind::NegativeInput
    );
    assert_eq!(
        F::url_reference(Arc::from(" \t")).unwrap_err().kind(),
        FilterErrorKind::EmptyReference
    );
}

#[test]
fn normal_pair_failure_retains_complete_endpoints_and_recovers_at_later_progress() {
    let a = list(vec![F::blur_pixels(0.).unwrap()]);
    let b = list(vec![F::blur_pixels(f64::MAX).unwrap()]);
    let from = InterpolableValue::filter(a);
    let to = InterpolableValue::filter(b);
    let pair = InterpolationPair::new(
        PropertyKey::new("filter").unwrap(),
        from.clone(),
        to.clone(),
    )
    .unwrap();
    let InterpolationOutcome::FilterFailure(failure) =
        pair.sample(InterpolationProgress::new(2.).unwrap())
    else {
        panic!("expected filter failure")
    };
    assert_eq!(failure.from(), &from);
    assert_eq!(failure.to(), &to);
    assert_eq!(failure.error().function_index(), Some(0));
    assert_eq!(failure.error().kind(), FilterErrorKind::NonFinite);
    assert_eq!(failure.progress().value(), 2.);
    assert_eq!(
        pair.sample(InterpolationProgress::new(1.).unwrap()),
        InterpolationOutcome::Value(to)
    );
}

#[test]
fn native_filter_images_specialize_only_cloned_verified_source_context() {
    let source = FilterImageSource::new();
    let a = FilterImage::filtered(
        "complete-a",
        source.clone(),
        list(vec![F::blur_pixels(2.).unwrap()]),
    );
    let b = FilterImage::filtered(
        "complete-b",
        source.clone(),
        list(vec![F::blur_pixels(6.).unwrap()]),
    );
    let result = a
        .sample(
            &b,
            InterpolationProgress::new(0.5).unwrap(),
            |_, _, _| -> Result<(), ()> { panic!("compatible specialization must not delegate") },
        )
        .unwrap();
    let FilterImageSample::Filtered {
        source: actual,
        filters,
    } = result
    else {
        panic!("expected filtered image")
    };
    assert_eq!(actual, source);
    assert_eq!(filters.functions()[0].blur_pixels(), Some(4.));
    let changed = FilterImage::filtered("changed-source", FilterImageSource::new(), list(vec![]));
    let fallback = a
        .sample(
            &changed,
            InterpolationProgress::new(0.25).unwrap(),
            |from, to, p| {
                assert_eq!(
                    (*from, *to, p.value()),
                    ("complete-a", "changed-source", 0.25)
                );
                Ok::<_, ()>("generic-image")
            },
        )
        .unwrap();
    assert!(matches!(
        fallback,
        FilterImageSample::Generic("generic-image")
    ));
}

#[test]
fn native_filter_image_fallback_executes_for_generic_url_and_failed_specialization() {
    let source = FilterImageSource::new();
    let a = FilterImage::filtered(
        "complete-a",
        source.clone(),
        list(vec![F::blur_pixels(0.).unwrap()]),
    );
    let endpoints = [
        FilterImage::generic("generic"),
        FilterImage::filtered(
            "url",
            source.clone(),
            list(vec![F::url_reference(Arc::from("url(#x)")).unwrap()]),
        ),
        FilterImage::filtered(
            "overflow",
            source,
            list(vec![F::blur_pixels(f64::MAX).unwrap()]),
        ),
    ];
    for to in endpoints {
        let error = a
            .sample(
                &to,
                InterpolationProgress::new(2.).unwrap(),
                |from, end, p| {
                    assert_eq!(*from, "complete-a");
                    assert_eq!(end, to.original());
                    assert_eq!(p.value(), 2.);
                    Err::<(), _>("generic-failure")
                },
            )
            .unwrap_err();
        assert_eq!(error, "generic-failure");
    }
}

fn tracks(
    from: FilterList,
    to: FilterList,
    easing: surgeist_animation::easing::Easing,
) -> (
    surgeist_animation::transition::TransitionTrack,
    surgeist_animation::keyframe::KeyframeTrack,
) {
    use surgeist_animation::keyframe::{
        KeyframeAnimationId, KeyframeSegment, KeyframeTiming, KeyframeTrack,
    };
    use surgeist_animation::time::{AnimationDelay, AnimationDuration};
    use surgeist_animation::timing::{AnimationDirection, FillMode, IterationCount};
    use surgeist_animation::transition::{TransitionTrack, TransitionTrackId};
    let from = InterpolableValue::filter(from);
    let to = InterpolableValue::filter(to);
    let property = PropertyKey::new("filter").unwrap();
    let delay = AnimationDelay::from_secs(0.).unwrap();
    let duration = AnimationDuration::from_secs(1.).unwrap();
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
    let timing = KeyframeTiming::try_new(
        delay,
        duration,
        IterationCount::finite(1.).unwrap(),
        AnimationDirection::Normal,
        FillMode::Both,
    )
    .unwrap();
    let segment = KeyframeSegment::new(
        property,
        UnitRatio::new(0.).unwrap(),
        UnitRatio::new(1.).unwrap(),
        from,
        to,
        easing,
    )
    .unwrap();
    let frames = KeyframeTrack::new(
        KeyframeAnimationId::new("filters").unwrap(),
        timing,
        vec![segment],
    )
    .unwrap();
    (transition, frames)
}

#[test]
fn normal_transition_and_keyframe_paths_sample_the_same_complete_list() {
    use surgeist_animation::sample::{SampleClassification, SampleValue};
    use surgeist_animation::time::ElapsedTime;
    use surgeist_animation::timing::AnimationPlayState;
    let a = list(vec![F::blur_pixels(2.).unwrap()]);
    let b = list(vec![F::blur_pixels(6.).unwrap(), amount(A::Brightness, 3.)]);
    let (transition, frames) = tracks(a, b, surgeist_animation::easing::Easing::linear());
    let elapsed = ElapsedTime::from_secs(0.5).unwrap();
    let frame = frames.sample(elapsed, AnimationPlayState::Running);
    for sample in [
        &transition.sample(elapsed, AnimationPlayState::Running),
        &frame[0],
    ] {
        assert_eq!(sample.classification(), SampleClassification::Active);
        let SampleValue::Value(property) = sample.value() else {
            panic!("expected complete sampled value")
        };
        assert_eq!(property.property().as_str(), "filter");
        let InterpolableValue::Filter(value) = property.value() else {
            panic!("expected filter list")
        };
        assert_eq!(value.functions()[0].blur_pixels(), Some(4.));
        assert_eq!(scalar(value, 1), 2.);
    }
}

#[test]
fn eased_overflow_publishes_no_partial_list_and_later_samples_recover() {
    use surgeist_animation::easing::{CubicBezier, Easing};
    use surgeist_animation::interpolation::InterpolationError;
    use surgeist_animation::sample::{SampleClassification, SampleValue};
    use surgeist_animation::time::ElapsedTime;
    use surgeist_animation::timing::AnimationPlayState;
    let a = list(vec![F::blur_pixels(0.).unwrap(), amount(A::Brightness, 1.)]);
    let b = list(vec![
        F::blur_pixels(f64::MAX).unwrap(),
        amount(A::Brightness, 2.),
    ]);
    let (transition, frames) = tracks(
        a,
        b,
        Easing::cubic_bezier(CubicBezier::new(0., 3., 1., -1.).unwrap()),
    );
    // At curve parameter 1/4, x=5/32 and y=73/64. At x=1/2,
    // y=7/8, so both ordinary samplers recover during their active interval.
    let elapsed = ElapsedTime::from_secs(0.15625).unwrap();
    let frame = frames.sample(elapsed, AnimationPlayState::Running);
    for sample in [
        &transition.sample(elapsed, AnimationPlayState::Running),
        &frame[0],
    ] {
        assert_eq!(sample.classification(), SampleClassification::Failed);
        let SampleValue::Error(InterpolationError::Filter(failure)) = sample.value() else {
            panic!("expected typed failure")
        };
        assert_eq!(failure.error().function_index(), Some(0));
        assert_eq!(failure.property().as_str(), "filter");
        assert!(failure.progress().value() > 1.);
    }
    assert!(matches!(
        transition
            .sample(
                ElapsedTime::from_secs(0.5).unwrap(),
                AnimationPlayState::Running
            )
            .value(),
        SampleValue::Value(_)
    ));
    assert!(matches!(
        frames.sample(
            ElapsedTime::from_secs(0.5).unwrap(),
            AnimationPlayState::Running
        )[0]
        .value(),
        SampleValue::Value(_)
    ));
}

#[test]
fn drop_shadow_space_mismatch_and_extended_input_alpha_are_typed() {
    let a = list(vec![
        F::drop_shadow_pixels([0., 0.], 0., color(Space::Srgb)).unwrap(),
    ]);
    let b = list(vec![
        F::drop_shadow_pixels([0., 0.], 0., color(Space::Oklab)).unwrap(),
    ]);
    assert_eq!(
        a.interpolate(&b, InterpolationProgress::new(0.5).unwrap())
            .unwrap_err()
            .kind(),
        FilterErrorKind::MismatchedColorSpaces
    );
    let extended = at(&list(vec![]), &a, 2.);
    let c = extended.functions()[0].drop_shadow().unwrap().color();
    assert_eq!(
        F::drop_shadow_pixels([0., 0.], 0., c).unwrap_err().kind(),
        FilterErrorKind::InvalidInputAlpha
    );
    assert_eq!(
        F::amount(
            A::Grayscale,
            FilterAmountInput::Percentage(InterpolablePercentage::new(-0.1).unwrap())
        )
        .unwrap_err()
        .kind(),
        FilterErrorKind::NegativeInput
    );
}
