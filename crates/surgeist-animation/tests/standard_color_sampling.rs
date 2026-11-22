use surgeist_animation::composition::{AnimationTypeOperations, PropertyValueOperations};
use surgeist_animation::easing::{Easing, LinearControlPointInput, LinearEasing};
use surgeist_animation::interpolation::{
    InterpolableValue, InterpolationError, InterpolationOutcome, InterpolationPair,
    InterpolationProgress, PropertyKey,
};
use surgeist_animation::keyframe::{
    ComputedKeyframeInput, ComputedKeyframes, KeyframeAnimationId, KeyframeEndpointSource,
    KeyframeSampleInput, KeyframeSegment, KeyframeTarget, KeyframeTiming, KeyframeTrack,
};
use surgeist_animation::keyframe_input::CompositeOperation;
use surgeist_animation::progress::UnitRatio;
use surgeist_animation::sample::{
    CompletionStatus, NextFrameHint, SampleClassification, SampleValue, SampledPropertyResult,
};
use surgeist_animation::time::{
    AnimationDelay, AnimationDuration, ElapsedTime, OriginRelativeTime,
};
use surgeist_animation::timing::{
    AnimationDirection, AnimationPlayState, FillMode, IterationCount,
};
use surgeist_animation::transition::{
    TransitionChange, TransitionKey, TransitionPropertyPolicy, TransitionSamplePhase,
    TransitionSnapshot, TransitionState, TransitionTiming, TransitionTrack, TransitionTrackId,
    Transitionability,
};
use surgeist_color::{
    Color, ColorSpace, Finite, HueInterpolation, InterpolationComponent, InterpolationMethod,
    InterpolationSample, PredefinedColorSpace, UnitInterval,
};

const SRGB: ColorSpace = ColorSpace::Predefined(PredefinedColorSpace::Srgb);
const LINEAR_SRGB: ColorSpace = ColorSpace::Predefined(PredefinedColorSpace::SrgbLinear);

fn ratio(value: f64) -> UnitRatio {
    UnitRatio::new(value).unwrap()
}

fn time(seconds: f64) -> ElapsedTime {
    ElapsedTime::from_secs(seconds).unwrap()
}

fn progress_with_overshoot() -> Easing {
    linear_easing([(0.0, 0.0), (0.25, 2.0), (0.75, 0.5), (1.0, 1.0)])
}

fn linear_easing(points: [(f64, f64); 4]) -> Easing {
    Easing::LinearFunction(
        LinearEasing::new(
            points
                .into_iter()
                .map(|(input, output)| LinearControlPointInput { input, output })
                .collect(),
        )
        .unwrap(),
    )
}

fn color(space: ColorSpace, channels: [Option<f64>; 3], alpha: Option<f64>) -> Color {
    Color::new(
        space,
        channels.map(|channel| channel.map(|value| Finite::new(value).unwrap())),
        alpha.map(|value| UnitInterval::new(value).unwrap()),
    )
}

fn overflow_endpoints() -> [Color; 2] {
    [
        color(SRGB, [Some(f64::MAX), Some(0.0), Some(0.0)], Some(0.75)),
        color(SRGB, [Some(0.0); 3], Some(0.5)),
    ]
}

fn overflow_pair() -> InterpolationPair {
    let [first, second] = overflow_endpoints();
    InterpolationPair::resolved_colors(
        PropertyKey::new("color").unwrap(),
        first,
        second,
        InterpolationMethod::try_new(SRGB, None).unwrap(),
    )
}

fn transition() -> TransitionTrack {
    TransitionTrack::from_pair(
        TransitionTrackId::new(1),
        overflow_pair(),
        AnimationDuration::from_secs(1.0).unwrap(),
        AnimationDelay::from_secs(0.0).unwrap(),
        progress_with_overshoot(),
    )
    .unwrap()
}

fn keyframes() -> KeyframeTrack {
    let segment = KeyframeSegment::from_pair(
        overflow_pair(),
        ratio(0.0),
        ratio(1.0),
        progress_with_overshoot(),
    )
    .unwrap();
    KeyframeTrack::new(
        KeyframeAnimationId::new("overflow and recovery").unwrap(),
        KeyframeTiming::try_new(
            AnimationDelay::from_secs(0.0).unwrap(),
            AnimationDuration::from_secs(1.0).unwrap(),
            IterationCount::finite(1.0).unwrap(),
            AnimationDirection::Normal,
            FillMode::Both,
        )
        .unwrap(),
        vec![segment],
    )
    .unwrap()
}

fn assert_overflow_diagnostic(result: &SampledPropertyResult) {
    assert_eq!(result.completion(), CompletionStatus::Running);
    assert_eq!(result.next_frame(), NextFrameHint::MayChange);
    let SampleValue::Error(InterpolationError::Color(error)) = result.value() else {
        panic!(
            "expected a color interpolation error, got {:?}",
            result.value()
        );
    };
    let [first, second] = overflow_endpoints();
    assert_eq!(error.property().as_str(), "color");
    assert_eq!(error.progress().value(), 2.0);
    assert_resolved_endpoint(error.from(), first);
    assert_resolved_endpoint(error.to(), second);
    assert_eq!(
        error.error().component(),
        InterpolationComponent::Channel(0)
    );
}

fn assert_resolved_endpoint(value: &InterpolableValue, expected: Color) {
    let InterpolableValue::ResolvedColor(color) = value else {
        panic!("expected the resolved endpoint color, got {value:?}");
    };
    assert!(color.color().identical(expected));
}

fn sampled_color(result: &SampledPropertyResult) -> InterpolationSample {
    assert_eq!(result.classification(), SampleClassification::Active);
    let SampleValue::Value(property) = result.value() else {
        panic!("expected a sampled property, got {:?}", result.value());
    };
    assert_eq!(property.property().as_str(), "color");
    let InterpolableValue::ColorSample(sample) = property.value() else {
        panic!("expected a color sample, got {:?}", property.value());
    };
    *sample
}

fn channels(sample: InterpolationSample) -> [Option<f64>; 3] {
    sample.channels().map(|value| value.map(Finite::get))
}

fn assert_recovered_color(result: &SampledPropertyResult) {
    let sample = sampled_color(result);
    assert_eq!(sample.space(), SRGB);
    assert_eq!(sample.alpha().map(Finite::get), Some(0.625));
    let [red, green, blue] = channels(sample);
    // At p = 1/2, the red weight is 3/8. Straight red is (3/8) / (5/8) of MAX.
    assert!((red.unwrap() / f64::MAX - 0.6).abs() <= f64::EPSILON);
    assert_eq!([green, blue], [Some(0.0); 2]);
}

fn color_transition(
    first: Color,
    second: Color,
    method: InterpolationMethod,
    delay: f64,
    easing: Easing,
) -> TransitionTrack {
    TransitionTrack::from_pair(
        TransitionTrackId::new(1),
        InterpolationPair::resolved_colors(
            PropertyKey::new("color").unwrap(),
            first,
            second,
            method,
        ),
        AnimationDuration::from_secs(1.0).unwrap(),
        AnimationDelay::from_secs(delay).unwrap(),
        easing,
    )
    .unwrap()
}

fn midpoint(first: Color, second: Color, method: InterpolationMethod) -> InterpolationSample {
    let track = color_transition(first, second, method, 0.0, Easing::Linear);
    sampled_color(&track.sample(time(0.5), AnimationPlayState::Running))
}

#[test]
fn transition_reports_unrepresentable_straight_color_at_eased_progress_and_recovers() {
    let transition = transition();
    let failed = transition.sample(time(0.25), AnimationPlayState::Running);
    // At p = 2 the alpha is 1/4 and the premultiplied red is -3*MAX/4.
    // Its straight coordinate is -3*MAX, which cannot be published.
    assert_eq!(failed.classification(), SampleClassification::Failed);
    assert!(matches!(failed.value(), SampleValue::Error(_)));
    assert_overflow_diagnostic(&failed);

    let recovered = transition.sample(time(0.75), AnimationPlayState::Running);
    assert_eq!(recovered.classification(), SampleClassification::Active);
    assert_recovered_color(&recovered);
}

#[test]
fn keyframes_report_unrepresentable_straight_color_at_eased_progress_and_recovers() {
    let track = keyframes();
    let failed = track.sample(time(0.25), AnimationPlayState::Running);
    assert_eq!(failed.len(), 1);
    assert_eq!(failed[0].classification(), SampleClassification::Failed);
    assert!(matches!(failed[0].value(), SampleValue::Error(_)));
    assert_overflow_diagnostic(&failed[0]);

    let recovered = track.sample(time(0.75), AnimationPlayState::Running);
    assert_eq!(recovered.len(), 1);
    assert_eq!(recovered[0].classification(), SampleClassification::Active);
    assert_recovered_color(&recovered[0]);
}

#[test]
fn running_css_transition_preserves_the_color_cause_and_recovers() {
    struct ColorPolicy(InterpolationMethod);
    impl TransitionPropertyPolicy for ColorPolicy {
        fn transitionability(
            &self,
            _: &PropertyKey,
            from: &InterpolableValue,
            to: &InterpolableValue,
        ) -> Transitionability {
            if matches!(
                (from, to),
                (
                    InterpolableValue::ResolvedColor(_),
                    InterpolableValue::ResolvedColor(_)
                )
            ) {
                Transitionability::Continuous
            } else {
                Transitionability::Discrete
            }
        }

        fn interpolate(
            &self,
            property: &PropertyKey,
            from: &InterpolableValue,
            to: &InterpolableValue,
            progress: InterpolationProgress,
        ) -> InterpolationOutcome {
            if let (InterpolableValue::ResolvedColor(from), InterpolableValue::ResolvedColor(to)) =
                (from, to)
            {
                return InterpolationPair::resolved_colors(
                    property.clone(),
                    from.color(),
                    to.color(),
                    self.0,
                )
                .sample(progress);
            }
            InterpolationOutcome::Unsupported(InterpolationError::unsupported_family(
                property.clone(),
                from.clone(),
                to.clone(),
                "resolved colors are required",
            ))
        }
    }

    let [first, second] = overflow_endpoints();
    let key = TransitionKey::new(1, PropertyKey::new("color").unwrap());
    let policy = ColorPolicy(InterpolationMethod::try_new(SRGB, None).unwrap());
    let change = TransitionChange {
        key: key.clone(),
        before: InterpolableValue::resolved_color(first),
        after: InterpolableValue::resolved_color(second),
        eligible: true,
        matching: Some(TransitionTiming::new(
            AnimationDuration::from_secs(1.0).unwrap(),
            AnimationDelay::from_secs(0.0).unwrap(),
            progress_with_overshoot(),
        )),
        new_id: TransitionTrackId::new(1),
    };
    let at = |time| Some(OriginRelativeTime::from_millis(time).unwrap());
    let update = TransitionSnapshot::default()
        .update(at(0.0), &[change], &policy)
        .unwrap()
        .unwrap();
    let Some(TransitionState::Running(run)) = update.state().get(&key) else {
        panic!("expected a running CSS transition");
    };
    let failure = run.sample(at(250.0), &policy).unwrap_err();
    let Some(InterpolationError::Color(cause)) = failure.interpolation_error() else {
        panic!("expected a typed color cause, got {failure}");
    };
    assert_eq!(cause.property(), key.property());
    assert_eq!(cause.progress().value(), 2.0);
    assert_eq!(
        cause.error().component(),
        InterpolationComponent::Channel(0)
    );

    let later = run.sample(at(750.0), &policy).unwrap();
    assert_eq!(later.phase(), TransitionSamplePhase::Active);
    let Some(InterpolableValue::ColorSample(color)) = later.value() else {
        panic!("expected a checked color after recovery");
    };
    assert_eq!(color.alpha().map(Finite::get), Some(0.625));
    assert!((channels(*color)[0].unwrap() / f64::MAX - 0.6).abs() <= f64::EPSILON);
}

#[test]
fn reverse_keyframe_delay_holds_the_raw_directed_endpoint() {
    let first = color(SRGB, [Some(0.5); 3], Some(1.0));
    let second = color(SRGB, [None, Some(0.25), None], Some(0.5));
    let segment = KeyframeSegment::from_pair(
        InterpolationPair::resolved_colors(
            PropertyKey::new("color").unwrap(),
            first,
            second,
            InterpolationMethod::try_new(LINEAR_SRGB, None).unwrap(),
        ),
        ratio(0.0),
        ratio(1.0),
        progress_with_overshoot(),
    )
    .unwrap();
    let track = KeyframeTrack::new(
        KeyframeAnimationId::new("reverse color delay").unwrap(),
        KeyframeTiming::try_new(
            AnimationDelay::from_secs(0.5).unwrap(),
            AnimationDuration::from_secs(1.0).unwrap(),
            IterationCount::finite(1.0).unwrap(),
            AnimationDirection::Reverse,
            FillMode::Both,
        )
        .unwrap(),
        vec![segment],
    )
    .unwrap();
    let results = track.sample(time(0.25), AnimationPlayState::Running);
    assert_eq!(results[0].classification(), SampleClassification::Filling);
    let SampleValue::Value(held) = results[0].value() else {
        panic!("expected the filled color");
    };
    assert_resolved_endpoint(held.value(), second);
}

#[test]
fn missing_components_borrow_the_other_endpoint_and_zero_components_interpolate() {
    let second = color(SRGB, [Some(0.5), None, Some(1.0)], None);
    let method = InterpolationMethod::try_new(SRGB, None).unwrap();
    let missing = midpoint(color(SRGB, [None, None, Some(0.0)], None), second, method);
    let zero = midpoint(
        color(SRGB, [Some(0.0), None, Some(0.0)], None),
        second,
        method,
    );

    // CSS Color 4 §13.3 carries a lone missing component across the pair.
    assert_eq!(channels(missing), [Some(0.5), None, Some(0.5)]);
    assert_eq!(channels(zero), [Some(0.25), None, Some(0.5)]);
    assert_eq!(missing.alpha(), None);
    assert_eq!(zero.alpha(), None);
}

#[test]
fn transparent_alpha_and_missing_alpha_give_different_weights() {
    let second = color(SRGB, [Some(0.0), None, Some(1.0)], Some(0.5));
    let method = InterpolationMethod::try_new(SRGB, None).unwrap();
    let transparent = midpoint(
        color(SRGB, [Some(1.0), None, Some(0.0)], Some(0.0)),
        second,
        method,
    );
    let missing = midpoint(
        color(SRGB, [Some(1.0), None, Some(0.0)], None),
        second,
        method,
    );

    // CSS Color 4 §§13.3–13.4. The explicit zero gives red no weight.
    // The missing alpha borrows 1/2, giving red and blue equal weight.
    assert_eq!(transparent.alpha().map(Finite::get), Some(0.25));
    assert_eq!(channels(transparent), [Some(0.0), None, Some(1.0)]);
    assert_eq!(missing.alpha().map(Finite::get), Some(0.5));
    assert_eq!(channels(missing), [Some(0.5), None, Some(0.5)]);
}

#[test]
fn hue_interpolation_uses_the_selected_arc_across_zero_degrees() {
    let space = ColorSpace::Oklch;
    let first = color(space, [Some(0.5), Some(0.125), Some(350.0)], Some(0.25));
    let second = color(space, [Some(0.5), Some(0.125), Some(10.0)], Some(0.75));
    // CSS Color 4 §13.5. The 20° increasing arc crosses zero. The 340°
    // decreasing arc crosses 180°. Differing alphas leave the arc weights equal.
    for (arc, expected_hue) in [
        (HueInterpolation::Shorter, 0.0),
        (HueInterpolation::Longer, 180.0),
        (HueInterpolation::Increasing, 0.0),
        (HueInterpolation::Decreasing, 180.0),
    ] {
        let sample = midpoint(
            first,
            second,
            InterpolationMethod::try_new(space, Some(arc)).unwrap(),
        );
        assert_eq!(sample.space(), space);
        assert_eq!(
            channels(sample),
            [Some(0.5), Some(0.125), Some(expected_hue)],
            "{arc:?}"
        );
        assert_eq!(sample.alpha().map(Finite::get), Some(0.5));
    }
}

#[test]
fn extended_easing_preserves_negative_and_greater_than_unit_alpha() {
    let track = color_transition(
        color(SRGB, [Some(1.0), None, Some(0.0)], Some(0.0)),
        color(SRGB, [Some(0.0), None, Some(1.0)], Some(0.75)),
        InterpolationMethod::try_new(SRGB, None).unwrap(),
        0.0,
        linear_easing([(0.0, 0.0), (0.25, -0.5), (0.75, 2.0), (1.0, 1.0)]),
    );

    // Only blue has nonzero premultiplied weight, so dividing by the
    // extrapolated alpha restores blue to 1 on both sides of the interval.
    for (elapsed, expected_alpha) in [(0.25, -0.375), (0.75, 1.5)] {
        let sample = sampled_color(&track.sample(time(elapsed), AnimationPlayState::Running));
        assert_eq!(sample.alpha().map(Finite::get), Some(expected_alpha));
        assert_eq!(channels(sample), [Some(0.0), None, Some(1.0)]);
    }
}

#[test]
fn transition_holds_the_raw_start_then_converts_to_the_selected_space() {
    let first = color(SRGB, [Some(0.02584), None, None], None);
    let second = color(LINEAR_SRGB, [Some(0.0), Some(0.5), None], Some(1.0));
    let track = color_transition(
        first,
        second,
        InterpolationMethod::try_new(LINEAR_SRGB, None).unwrap(),
        1.0,
        Easing::Linear,
    );

    for (elapsed, classification) in [
        (0.5, SampleClassification::Filling),
        (1.0, SampleClassification::Active),
    ] {
        let held = track.sample(time(elapsed), AnimationPlayState::Running);
        assert_eq!(held.classification(), classification);
        let SampleValue::Value(property) = held.value() else {
            panic!("expected a held property, got {:?}", held.value());
        };
        assert_resolved_endpoint(property.value(), first);
    }

    let sample = sampled_color(&track.sample(time(1.5), AnimationPlayState::Running));
    assert_eq!(sample.space(), LINEAR_SRGB);
    let [red, green, blue] = channels(sample);
    // CSS Color 4 §10.2 linearizes the encoded red to 0.02584 / 12.92.
    assert!((red.unwrap() - 0.001).abs() < 1e-12);
    assert_eq!([green, blue], [Some(0.5), None]);
    assert_eq!(sample.alpha().map(Finite::get), Some(1.0));
}

#[test]
fn computed_keyframe_samples_can_reenter_a_color_pair_with_a_selected_method() {
    let property = PropertyKey::new("color").unwrap();
    let start = InterpolableValue::resolved_color(color(SRGB, [Some(0.0); 3], Some(0.0)));
    let end = InterpolableValue::resolved_color(color(SRGB, [Some(1.0); 3], Some(1.0)));
    let frames = ComputedKeyframes::new(
        [(0.0, start.clone()), (1.0, end.clone())]
            .into_iter()
            .map(|(offset, value)| ComputedKeyframeInput {
                specified_offset: Some(ratio(offset)),
                computed_offset: ratio(offset),
                easing: Easing::Linear,
                composite: None,
                properties: vec![(property.clone(), value)],
            })
            .collect(),
        KeyframeEndpointSource::CssAssembled,
    )
    .unwrap();
    let operations = AnimationTypeOperations::for_color(
        InterpolationMethod::try_new(LINEAR_SRGB, None).unwrap(),
    );
    let progress = InterpolationProgress::new(2.0).unwrap();
    let value = frames
        .sample(
            &property,
            KeyframeSampleInput {
                target: KeyframeTarget::Available(&start),
                effect_composite: CompositeOperation::Replace,
            },
            Some(progress),
            &operations,
        )
        .unwrap()
        .unwrap();
    let InterpolableValue::ColorSample(sample) = &value else {
        panic!("expected a color sample, got {value:?}");
    };
    assert_eq!(sample.space(), LINEAR_SRGB);
    assert_eq!(sample.alpha().map(Finite::get), Some(2.0));
    assert_eq!(channels(*sample), [Some(1.0); 3]);

    assert!(matches!(
        InterpolationPair::new(property.clone(), value.clone(), end.clone()),
        Err(InterpolationError::UnsupportedFamily(_))
    ));
    let next = operations
        .interpolate(
            &property,
            &value,
            &end,
            InterpolationProgress::new(0.5).unwrap(),
        )
        .unwrap();
    let InterpolableValue::ColorSample(next) = next else {
        panic!("expected a re-interpolated color sample, got {next:?}");
    };
    assert_eq!(next.space(), LINEAR_SRGB);
    assert_eq!(next.alpha().map(Finite::get), Some(1.5));
    assert_eq!(channels(next), [Some(1.0); 3]);
}
