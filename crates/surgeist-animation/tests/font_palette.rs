#![forbid(unsafe_code)]

use surgeist_animation::easing::{Easing, LinearControlPointInput, LinearEasing};
use surgeist_animation::interpolation::{
    InterpolableValue, InterpolationError, InterpolationOutcome, InterpolationPair,
    InterpolationProgress, PropertyKey, ValueFamily,
};
use surgeist_animation::keyframe::{
    KeyframeAnimationId, KeyframeSegment, KeyframeTiming, KeyframeTrack,
};
use surgeist_animation::palette::{PaletteNode, PaletteSource, PaletteValue};
use surgeist_animation::progress::UnitRatio;
use surgeist_animation::sample::{SampleClassification, SampleValue};
use surgeist_animation::time::{AnimationDelay, AnimationDuration, ElapsedTime};
use surgeist_animation::timing::{
    AnimationDirection, AnimationPlayState, FillMode, IterationCount,
};
use surgeist_animation::transition::{TransitionTrack, TransitionTrackId};

// Fonts 4 §9.1 and the official font-palette-interpolation.html expectations.
// Style retains the mapping from these opaque computed sources to light, dark,
// normal, named palettes or an already computed palette-mix expression.
#[test]
fn light_to_dark_returns_an_oklab_plan_at_strictly_interior_progress() {
    let light = PaletteSource::new();
    let dark = PaletteSource::new();
    let pair = pair(value(&light), value(&dark));
    for at in [0.3, 0.6] {
        let result = sampled(&pair, at);
        assert_eq!(
            result.nodes(),
            &[
                PaletteNode::Source(light.clone()),
                PaletteNode::Source(dark.clone()),
                PaletteNode::MixOklab {
                    first: 0,
                    second: 1,
                    progress: progress(at)
                },
            ]
        );
        assert_eq!(result.as_source(), None);
    }
    assert_eq!(pair.from().family(), ValueFamily::Palette);
}

#[test]
fn keyword_and_named_operands_are_exact_at_and_outside_the_palette_limits() {
    let named = PaletteSource::new();
    let normal = PaletteSource::new();
    let from = value(&named);
    let to = value(&normal);
    let pair = pair(from.clone(), to.clone());
    for at in [f64::MIN, -2.0, -0.25, -0.0, 0.0] {
        assert_eq!(sampled(&pair, at), from);
    }
    for at in [1.0, 1.5, f64::MAX] {
        assert_eq!(sampled(&pair, at), to);
    }
    assert_eq!(
        sampled(&pair, 0.5).nodes(),
        &[
            PaletteNode::Source(named),
            PaletteNode::Source(normal),
            PaletteNode::MixOklab {
                first: 0,
                second: 1,
                progress: progress(0.5)
            },
        ]
    );
}

#[test]
fn equivalent_supplied_operands_and_equal_sampled_plans_collapse_unchanged() {
    let normal = value(&PaletteSource::new());
    let other = value(&PaletteSource::new());
    let mixed = normal.interpolate(&other, progress(0.25)).unwrap();
    for original in [normal, mixed] {
        let pair = pair(original.clone(), original.clone());
        for at in [-2.0, 0.0, 0.3, 0.6, 1.0, 1.5] {
            assert_eq!(sampled(&pair, at), original);
        }
    }
}

#[test]
fn separately_created_sources_stay_distinct_while_clones_keep_computed_identity() {
    let first = PaletteSource::new();
    let second = PaletteSource::new();
    assert_eq!(first.clone(), first);
    assert_ne!(first, second);
    assert_eq!(value(&first).as_source(), Some(&first));
    assert_eq!(value(&second).as_source(), Some(&second));
}

#[test]
fn nested_and_reversed_samples_preserve_each_mix_in_postorder() {
    let light = PaletteSource::new();
    let dark = PaletteSource::new();
    let named = PaletteSource::new();
    let first = value(&light)
        .interpolate(&value(&dark), progress(0.25))
        .unwrap();
    let nested = first.interpolate(&value(&named), progress(0.5)).unwrap();
    assert_eq!(
        nested.nodes(),
        &[
            PaletteNode::Source(light),
            PaletteNode::Source(dark),
            PaletteNode::MixOklab {
                first: 0,
                second: 1,
                progress: progress(0.25)
            },
            PaletteNode::Source(named.clone()),
            PaletteNode::MixOklab {
                first: 2,
                second: 3,
                progress: progress(0.5)
            },
        ]
    );
    let reversed = value(&named).interpolate(&first, progress(0.5)).unwrap();
    let PaletteNode::MixOklab {
        first,
        second,
        progress: fraction,
    } = reversed.nodes().last().unwrap()
    else {
        panic!("expected reversed palette mix")
    };
    assert_eq!(
        reversed.nodes().get(*first),
        Some(&PaletteNode::Source(named))
    );
    assert!(matches!(
        reversed.nodes().get(*second),
        Some(PaletteNode::MixOklab { .. })
    ));
    assert_eq!(fraction.value(), 0.5);
    assert_eq!(
        nested.interpolate(&reversed, progress(0.0)).unwrap(),
        nested
    );
    assert_eq!(
        nested.interpolate(&reversed, progress(1.0)).unwrap(),
        reversed
    );
}

#[test]
fn repeated_branching_samples_share_subplans_without_exponential_expansion() {
    let first_source = PaletteSource::new();
    let second_source = PaletteSource::new();
    let mut first = value(&first_source);
    let mut second = value(&second_source);
    for generation in 1..=64 {
        let next_first = first.interpolate(&second, progress(0.25)).unwrap();
        let next_second = second.interpolate(&first, progress(0.5)).unwrap();
        // Both descendants refer to the same earlier subgraphs. The exported
        // plan owns each structural operation once, rather than expanding a tree.
        assert_eq!(next_first.nodes().len(), 2 * generation + 1);
        assert_eq!(next_second.nodes().len(), 2 * generation + 1);
        assert_topology(&next_first);
        assert_topology(&next_second);
        first = next_first;
        second = next_second;
    }
    assert_eq!(first.interpolate(&first, progress(0.5)).unwrap(), first);
    assert_eq!(first.add(&second), second);
}

#[test]
fn deeply_nested_plans_support_sampling_equality_debug_and_drop_on_a_small_stack() {
    std::thread::Builder::new()
        .stack_size(64 * 1024)
        .spawn(|| {
            let light = value(&PaletteSource::new());
            let dark = value(&PaletteSource::new());
            let mut result = light.clone();
            for _ in 0..2048 {
                result = result.interpolate(&dark, progress(0.5)).unwrap();
            }
            assert_eq!(result.nodes().len(), 2050);
            assert_topology(&result);
            assert_eq!(result.clone(), result);
            assert_eq!(result.interpolate(&result, progress(0.5)).unwrap(), result);
            assert_eq!(result.interpolate(&dark, progress(-0.5)).unwrap(), result);
            let rendered = format!("{result:?}");
            assert!(!rendered.is_empty());
            drop(result);
        })
        .unwrap()
        .join()
        .unwrap();
}

#[test]
fn palette_addition_returns_the_effect_value_including_an_existing_mix() {
    let underlying = value(&PaletteSource::new());
    let effect_start = value(&PaletteSource::new());
    let effect_end = value(&PaletteSource::new());
    let effect = effect_start
        .interpolate(&effect_end, progress(0.5))
        .unwrap();
    assert_eq!(underlying.add(&effect), effect);
    assert_eq!(effect.add(&underlying), underlying);
}

#[test]
fn keyframe_and_transition_samples_return_palette_plans_and_clamp_easing_overshoot() {
    let light = value(&PaletteSource::new());
    let dark = value(&PaletteSource::new());
    let easing = Easing::linear_function(
        LinearEasing::new(vec![
            LinearControlPointInput {
                input: 0.0,
                output: -1.0,
            },
            LinearControlPointInput {
                input: 1.0,
                output: 2.0,
            },
        ])
        .unwrap(),
    );
    let keyframe = keyframe(light.clone(), dark.clone(), easing.clone());
    let transition = transition(light.clone(), dark.clone(), easing);
    let middle = light.interpolate(&dark, progress(0.5)).unwrap();
    for (at, expected) in [
        (0.0, light.clone()),
        (0.25, light),
        (0.5, middle),
        (0.75, dark.clone()),
        (1.0, dark),
    ] {
        let samples = keyframe.sample(time(at), AnimationPlayState::Running);
        assert_sample(samples[0].value(), &expected);
        let sample = transition.sample(time(at), AnimationPlayState::Running);
        if at == 1.0 {
            // CSS transition tracks have backwards fill only. After the active
            // interval the caller publishes its supplied after-change style.
            assert!(matches!(sample.value(), SampleValue::None));
        } else {
            assert_sample(sample.value(), &expected);
        }
        if at == 0.5 {
            assert_eq!(sample.classification(), SampleClassification::Active);
        }
    }
}

#[test]
fn a_palette_cannot_be_paired_with_an_unrelated_value_family() {
    let from = InterpolableValue::palette(value(&PaletteSource::new()));
    let to = InterpolableValue::computed(surgeist_animation::computed::ComputedValue::single(
        surgeist_animation::computed::ComputedComponent::Numeric(
            surgeist_animation::computed::ComputedNumeric::number(1.0).unwrap(),
        ),
    ));
    assert!(
        matches!(InterpolationPair::new(property(), from.clone(), to.clone()),
        Err(InterpolationError::MismatchedFamilies { from: ValueFamily::Palette, to: ValueFamily::Computed, from_value, to_value, .. })
            if from_value == from && to_value == to)
    );
}

#[test]
fn native_running_transition_reversal_samples_the_previous_symbolic_palette() {
    use surgeist_animation::time::OriginRelativeTime;
    use surgeist_animation::transition::{
        TransitionChange, TransitionKey, TransitionPropertyPolicy, TransitionSnapshot,
        TransitionState, TransitionTiming, Transitionability,
    };
    struct PalettePolicy;
    impl TransitionPropertyPolicy for PalettePolicy {
        fn transitionability(
            &self,
            _: &PropertyKey,
            _: &InterpolableValue,
            _: &InterpolableValue,
        ) -> Transitionability {
            Transitionability::Continuous
        }
        fn interpolate(
            &self,
            property: &PropertyKey,
            from: &InterpolableValue,
            to: &InterpolableValue,
            at: InterpolationProgress,
        ) -> InterpolationOutcome {
            match InterpolationPair::new(property.clone(), from.clone(), to.clone()) {
                Ok(pair) => pair.sample(at),
                Err(error) => InterpolationOutcome::Unsupported(error),
            }
        }
    }
    let light = PaletteSource::new();
    let dark = PaletteSource::new();
    let from = InterpolableValue::palette(value(&light));
    let to = InterpolableValue::palette(value(&dark));
    let key = TransitionKey::new(1_u64, property());
    let matching = Some(TransitionTiming::new(
        AnimationDuration::from_secs(1.0).unwrap(),
        AnimationDelay::from_secs(0.0).unwrap(),
        Easing::linear(),
    ));
    let start = TransitionSnapshot::default()
        .update(
            Some(OriginRelativeTime::from_millis(0.0).unwrap()),
            &[TransitionChange {
                key: key.clone(),
                before: from.clone(),
                after: to.clone(),
                eligible: true,
                matching: matching.clone(),
                new_id: TransitionTrackId::new(1),
            }],
            &PalettePolicy,
        )
        .unwrap()
        .unwrap();
    let Some(TransitionState::Running(run)) = start.state().get(&key) else {
        panic!("expected running palette transition")
    };
    let middle = run
        .sample(
            Some(OriginRelativeTime::from_millis(500.0).unwrap()),
            &PalettePolicy,
        )
        .unwrap();
    let Some(InterpolableValue::Palette(middle_value)) = middle.value() else {
        panic!("expected intermediate palette plan")
    };
    assert_eq!(
        middle_value.nodes(),
        &[
            PaletteNode::Source(light.clone()),
            PaletteNode::Source(dark.clone()),
            PaletteNode::MixOklab {
                first: 0,
                second: 1,
                progress: progress(0.5)
            },
        ]
    );
    let reverse = start
        .state()
        .update(
            Some(OriginRelativeTime::from_millis(500.0).unwrap()),
            &[TransitionChange {
                key: key.clone(),
                before: middle.value().unwrap().clone(),
                after: from,
                eligible: true,
                matching,
                new_id: TransitionTrackId::new(2),
            }],
            &PalettePolicy,
        )
        .unwrap()
        .unwrap();
    let Some(TransitionState::Running(run)) = reverse.state().get(&key) else {
        panic!("expected reversed palette transition")
    };
    let result = run
        .sample(
            Some(OriginRelativeTime::from_millis(750.0).unwrap()),
            &PalettePolicy,
        )
        .unwrap();
    let Some(InterpolableValue::Palette(value)) = result.value() else {
        panic!("expected nested palette plan")
    };
    assert_eq!(
        value.nodes(),
        &[
            PaletteNode::Source(light),
            PaletteNode::Source(dark),
            PaletteNode::MixOklab {
                first: 0,
                second: 1,
                progress: progress(0.5)
            },
            PaletteNode::MixOklab {
                first: 2,
                second: 0,
                progress: progress(0.5)
            },
        ]
    );
}

fn assert_topology(value: &PaletteValue) {
    for (index, node) in value.nodes().iter().enumerate() {
        if let PaletteNode::MixOklab {
            first,
            second,
            progress,
        } = node
        {
            assert!(*first < index && *second < index);
            assert!(progress.value() > 0.0 && progress.value() < 1.0);
        }
    }
}
fn value(source: &PaletteSource) -> PaletteValue {
    PaletteValue::source(source.clone())
}
fn property() -> PropertyKey {
    PropertyKey::from_text("font-palette").unwrap()
}
fn progress(value: f64) -> InterpolationProgress {
    InterpolationProgress::new(value).unwrap()
}
fn time(value: f64) -> ElapsedTime {
    ElapsedTime::from_secs(value).unwrap()
}
fn pair(from: PaletteValue, to: PaletteValue) -> InterpolationPair {
    InterpolationPair::new(
        property(),
        InterpolableValue::palette(from),
        InterpolableValue::palette(to),
    )
    .unwrap()
}
fn sampled(pair: &InterpolationPair, at: f64) -> PaletteValue {
    let InterpolationOutcome::Value(InterpolableValue::Palette(value)) = pair.sample(progress(at))
    else {
        panic!("expected native palette sample")
    };
    value
}
fn keyframe(from: PaletteValue, to: PaletteValue, easing: Easing) -> KeyframeTrack {
    KeyframeTrack::new(
        KeyframeAnimationId::new("palette").unwrap(),
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
                property(),
                UnitRatio::new(0.0).unwrap(),
                UnitRatio::new(1.0).unwrap(),
                InterpolableValue::palette(from),
                InterpolableValue::palette(to),
                easing,
            )
            .unwrap(),
        ],
    )
    .unwrap()
}
fn transition(from: PaletteValue, to: PaletteValue, easing: Easing) -> TransitionTrack {
    TransitionTrack::new(
        TransitionTrackId::new(1),
        property(),
        InterpolableValue::palette(from),
        InterpolableValue::palette(to),
        AnimationDuration::from_secs(1.0).unwrap(),
        AnimationDelay::from_secs(0.0).unwrap(),
        easing,
    )
    .unwrap()
}
fn assert_sample(sample: &SampleValue, expected: &PaletteValue) {
    let SampleValue::Value(sample) = sample else {
        panic!("expected native palette sample, got {sample:?}")
    };
    assert_eq!(
        sample.value(),
        &InterpolableValue::palette(expected.clone())
    );
}
