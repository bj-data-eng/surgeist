#![forbid(unsafe_code)]

use surgeist_animation::composition::{
    AnimationClass, AnimationOrder, AnimationStackInput, AnimationType, AnimationTypeOperations,
    EffectStackEffect, EffectStackErrorKind, EffectStackSnapshot, EffectTargetId,
};
use surgeist_animation::computed::{ComputedComponent, ComputedRatio, ComputedValue};
use surgeist_animation::easing::Easing;
use surgeist_animation::interpolation::{InterpolableNumber, InterpolableValue, PropertyKey};
use surgeist_animation::keyframe::{
    ComputedKeyframeInput, ComputedKeyframes, KeyframeEndpointSource, KeyframeTarget,
};
use surgeist_animation::keyframe_input::CompositeOperation;
use surgeist_animation::playback::{AnimationRecords, EffectFacts, PlaybackContext};
use surgeist_animation::progress::UnitRatio;
use surgeist_animation::time::{AnimationTime, PlaybackRate, TimelineTime};
use surgeist_animation::timeline::{TimelineId, TimelineMonotonicity, TimelineSnapshot};
use surgeist_animation::timing::{
    AnimationReplaceState, EffectAnimationState, EffectPlayState, EffectTiming,
    EffectTimingOptions, FillMode,
};

fn key(name: &str) -> PropertyKey {
    PropertyKey::new(name).unwrap()
}
fn number(value: f64) -> InterpolableValue {
    InterpolableValue::number(InterpolableNumber::new(value).unwrap())
}
fn timing() -> EffectTiming {
    EffectTiming::try_new(EffectTimingOptions {
        duration_millis: 100.0,
        fill: FillMode::Both,
        ..Default::default()
    })
    .unwrap()
}
fn frames(
    values: &[(&str, InterpolableValue)],
    composite: Option<CompositeOperation>,
) -> ComputedKeyframes {
    ComputedKeyframes::new(
        [0.0, 1.0]
            .into_iter()
            .map(|offset| ComputedKeyframeInput {
                specified_offset: None,
                computed_offset: UnitRatio::new(offset).unwrap(),
                easing: Easing::linear(),
                composite,
                properties: values
                    .iter()
                    .map(|(name, value)| (key(name), value.clone()))
                    .collect(),
            })
            .collect(),
        KeyframeEndpointSource::Native,
    )
    .unwrap()
}
fn input(
    records: &mut AnimationRecords,
    values: &[(&str, InterpolableValue)],
    composite: CompositeOperation,
) -> AnimationStackInput {
    let timing = timing();
    let effect = records
        .register_effect(EffectFacts::from_timing(&timing))
        .unwrap();
    let timeline = TimelineSnapshot::new(
        TimelineId::new(1),
        Some(TimelineTime::from_millis(100.0).unwrap()),
        TimelineMonotonicity::Monotonic,
        None,
    );
    let context = PlaybackContext::new(vec![timeline]).unwrap();
    let created = records
        .create(Some(timeline.id()), Some(&effect), &context)
        .unwrap();
    AnimationStackInput {
        animation: created.animation().clone(),
        timeline: Some(timeline),
        local_time: Some(AnimationTime::from_millis(100.0).unwrap()),
        state: EffectAnimationState::new(
            PlaybackRate::new(1.0).unwrap(),
            EffectPlayState::Finished,
            Some(TimelineMonotonicity::Monotonic),
            AnimationReplaceState::Active,
        ),
        order: AnimationOrder::Defined {
            class: AnimationClass::new(0),
            inter_class: None,
            class_specific: None,
        },
        target: Some(EffectTargetId::new(1)),
        prescribed_by_markup: false,
        in_document: true,
        effect: Some(EffectStackEffect {
            timing,
            keyframes: frames(values, None),
            composite,
            not_animatable: vec![],
        }),
    }
}
fn state(input: &mut AnimationStackInput, play: EffectPlayState, replace: AnimationReplaceState) {
    input.state = EffectAnimationState::new(
        PlaybackRate::new(1.0).unwrap(),
        play,
        input.timeline.map(|t| t.monotonicity()),
        replace,
    );
}
fn sample(
    snapshot: &EffectStackSnapshot,
    property: &str,
    base: &InterpolableValue,
) -> Option<InterpolableValue> {
    snapshot
        .sample(
            EffectTargetId::new(1),
            &key(property),
            KeyframeTarget::Available(base),
            &AnimationTypeOperations::new(AnimationType::ByComputedValue),
        )
        .unwrap()
        .map(|result| result.value().clone())
}

#[test]
fn folds_in_creation_order_with_evolving_underlying_and_no_double_composition() {
    let mut records = AnimationRecords::new();
    let low = input(
        &mut records,
        &[("x", number(10.0))],
        CompositeOperation::Replace,
    );
    let high = input(&mut records, &[("x", number(3.0))], CompositeOperation::Add);
    let highest = high.animation.clone();
    let snapshot = EffectStackSnapshot::new(&records, vec![high, low]).unwrap();
    let result = snapshot
        .sample(
            EffectTargetId::new(1),
            &key("x"),
            KeyframeTarget::Available(&number(2.0)),
            &AnimationTypeOperations::new(AnimationType::ByComputedValue),
        )
        .unwrap()
        .unwrap();
    assert_eq!(result.value(), &number(13.0));
    assert_eq!(result.animation(), &highest);
}

#[test]
fn keyframe_override_composes_before_interpolation_using_the_new_underlying() {
    let mut records = AnimationRecords::new();
    let low = input(
        &mut records,
        &[("x", number(10.0))],
        CompositeOperation::Replace,
    );
    let mut high = input(&mut records, &[], CompositeOperation::Add);
    high.local_time = Some(AnimationTime::from_millis(50.0).unwrap());
    state(
        &mut high,
        EffectPlayState::Running,
        AnimationReplaceState::Active,
    );
    high.effect.as_mut().unwrap().keyframes = ComputedKeyframes::new(
        vec![
            ComputedKeyframeInput {
                specified_offset: None,
                computed_offset: UnitRatio::new(0.0).unwrap(),
                easing: Easing::linear(),
                composite: Some(CompositeOperation::Replace),
                properties: vec![(key("x"), number(2.0))],
            },
            ComputedKeyframeInput {
                specified_offset: None,
                computed_offset: UnitRatio::new(1.0).unwrap(),
                easing: Easing::linear(),
                composite: None,
                properties: vec![(key("x"), number(4.0))],
            },
        ],
        KeyframeEndpointSource::Native,
    )
    .unwrap();
    let snapshot = EffectStackSnapshot::new(&records, vec![low, high]).unwrap();
    // Endpoints are 2 and (10 + 4), whose midpoint is 8.
    assert_eq!(sample(&snapshot, "x", &number(100.0)), Some(number(8.0)));
}

#[test]
fn absent_properties_do_not_claim_contribution_or_change_the_top_class() {
    let mut records = AnimationRecords::new();
    let low = input(
        &mut records,
        &[("x", number(10.0)), ("y", number(20.0))],
        CompositeOperation::Replace,
    );
    let low_id = low.animation.clone();
    let mut high = input(
        &mut records,
        &[("y", number(30.0))],
        CompositeOperation::Replace,
    );
    high.order = AnimationOrder::Defined {
        class: AnimationClass::new(9),
        inter_class: None,
        class_specific: None,
    };
    let snapshot = EffectStackSnapshot::new(&records, vec![high, low]).unwrap();
    let result = snapshot
        .sample(
            EffectTargetId::new(1),
            &key("x"),
            KeyframeTarget::Available(&number(10.0)),
            &AnimationTypeOperations::new(AnimationType::Discrete),
        )
        .unwrap()
        .unwrap();
    assert_eq!(result.animation(), &low_id);
    assert_eq!(result.class(), AnimationClass::new(0));
    assert_eq!(sample(&snapshot, "y", &number(0.0)), Some(number(30.0)));
    assert_eq!(sample(&snapshot, "absent", &number(0.0)), None);
}

#[test]
fn inter_class_then_common_class_then_creation_order_are_applied() {
    let mut records = AnimationRecords::new();
    let mut first = input(
        &mut records,
        &[("x", number(1.0))],
        CompositeOperation::Replace,
    );
    let mut second = input(
        &mut records,
        &[("x", number(2.0))],
        CompositeOperation::Replace,
    );
    first.order = AnimationOrder::Defined {
        class: AnimationClass::new(3),
        inter_class: Some(2),
        class_specific: Some(0),
    };
    second.order = AnimationOrder::Defined {
        class: AnimationClass::new(4),
        inter_class: Some(1),
        class_specific: Some(999),
    };
    assert_eq!(
        sample(
            &EffectStackSnapshot::new(&records, vec![first.clone(), second.clone()]).unwrap(),
            "x",
            &number(0.0)
        ),
        Some(number(1.0))
    );
    second.order = AnimationOrder::Defined {
        class: AnimationClass::new(3),
        inter_class: Some(2),
        class_specific: Some(1),
    };
    assert_eq!(
        sample(
            &EffectStackSnapshot::new(&records, vec![second.clone(), first.clone()]).unwrap(),
            "x",
            &number(0.0)
        ),
        Some(number(2.0))
    );
    second.order = AnimationOrder::Defined {
        class: AnimationClass::new(4),
        inter_class: None,
        class_specific: None,
    };
    assert_eq!(
        sample(
            &EffectStackSnapshot::new(&records, vec![second, first]).unwrap(),
            "x",
            &number(0.0)
        ),
        Some(number(2.0))
    );
}

#[test]
fn contradictory_order_foreign_duplicate_and_inconsistent_observations_are_rejected() {
    let mut records = AnimationRecords::new();
    let mut a = input(&mut records, &[], CompositeOperation::Replace);
    let b = input(&mut records, &[], CompositeOperation::Replace);
    let mut c = input(&mut records, &[], CompositeOperation::Replace);
    // a < b < c by creation, but c < a by a common-class rule.
    a.order = AnimationOrder::Defined {
        class: AnimationClass::new(1),
        inter_class: None,
        class_specific: Some(1),
    };
    c.order = AnimationOrder::Defined {
        class: AnimationClass::new(1),
        inter_class: None,
        class_specific: Some(0),
    };
    assert_eq!(
        EffectStackSnapshot::new(&records, vec![a.clone(), b, c])
            .unwrap_err()
            .kind(),
        EffectStackErrorKind::InconsistentOrder
    );
    assert_eq!(
        EffectStackSnapshot::new(&records, vec![a.clone(), a.clone()])
            .unwrap_err()
            .kind(),
        EffectStackErrorKind::DuplicateAnimation
    );
    let other = AnimationRecords::new();
    assert_eq!(
        EffectStackSnapshot::new(&other, vec![a.clone()])
            .unwrap_err()
            .kind(),
        EffectStackErrorKind::Playback
    );
    a.state = EffectAnimationState::new(
        PlaybackRate::new(1.0).unwrap(),
        EffectPlayState::Finished,
        None,
        AnimationReplaceState::Active,
    );
    assert_eq!(
        EffectStackSnapshot::new(&records, vec![a])
            .unwrap_err()
            .kind(),
        EffectStackErrorKind::TimelineMismatch
    );
}

#[test]
fn undefined_orphan_idle_order_is_retained_but_never_enters_a_stack() {
    let mut records = AnimationRecords::new();
    let mut orphan = input(
        &mut records,
        &[("x", number(1.0))],
        CompositeOperation::Replace,
    );
    orphan.order = AnimationOrder::Undefined {
        class: AnimationClass::new(2),
    };
    assert_eq!(
        EffectStackSnapshot::new(&records, vec![orphan.clone()])
            .unwrap_err()
            .kind(),
        EffectStackErrorKind::UndefinedOrder
    );
    state(
        &mut orphan,
        EffectPlayState::Idle,
        AnimationReplaceState::Active,
    );
    orphan.local_time = None;
    let snapshot = EffectStackSnapshot::new(&records, vec![orphan]).unwrap();
    assert_eq!(snapshot.undefined_animations().count(), 1);
    assert_eq!(sample(&snapshot, "x", &number(0.0)), None);
}

#[test]
fn replacement_uses_union_coverage_and_persisted_covers_without_being_removed() {
    let mut records = AnimationRecords::new();
    let a = input(
        &mut records,
        &[("x", number(1.0)), ("y", number(2.0))],
        CompositeOperation::Replace,
    );
    let a_id = a.animation.clone();
    let b = input(&mut records, &[("x", number(3.0))], CompositeOperation::Add);
    let mut c = input(
        &mut records,
        &[("y", number(4.0))],
        CompositeOperation::Accumulate,
    );
    state(
        &mut c,
        EffectPlayState::Finished,
        AnimationReplaceState::Persisted,
    );
    let saved_time = a.local_time;
    let snapshot =
        EffectStackSnapshot::new(&records, vec![c.clone(), b.clone(), a.clone()]).unwrap();
    let decisions = snapshot.replacements().unwrap();
    assert_eq!(decisions.len(), 1);
    assert_eq!(decisions[0].animation(), &a_id);
    assert_eq!(decisions[0].current_time(), saved_time);
    assert_eq!(decisions[0].timeline().unwrap().id(), TimelineId::new(1));
    c.target = Some(EffectTargetId::new(2));
    assert!(
        EffectStackSnapshot::new(&records, vec![a.clone(), b, c])
            .unwrap()
            .replacements()
            .unwrap()
            .is_empty()
    );
    let mut persisted = AnimationReplaceState::Removed;
    persisted.persist();
    assert_eq!(persisted, AnimationReplaceState::Persisted);
}

#[test]
fn nonreplaceable_cover_and_removed_effects_are_excluded() {
    let mut records = AnimationRecords::new();
    let a = input(
        &mut records,
        &[("x", number(1.0))],
        CompositeOperation::Replace,
    );
    let mut b = input(
        &mut records,
        &[("x", number(2.0))],
        CompositeOperation::Replace,
    );
    for (play, replace, markup) in [
        (
            EffectPlayState::Running,
            AnimationReplaceState::Active,
            false,
        ),
        (
            EffectPlayState::Finished,
            AnimationReplaceState::Removed,
            false,
        ),
        (
            EffectPlayState::Finished,
            AnimationReplaceState::Active,
            true,
        ),
    ] {
        state(&mut b, play, replace);
        b.prescribed_by_markup = markup;
        assert!(
            EffectStackSnapshot::new(&records, vec![a.clone(), b.clone()])
                .unwrap()
                .replacements()
                .unwrap()
                .is_empty()
        );
    }
    state(
        &mut b,
        EffectPlayState::Finished,
        AnimationReplaceState::Removed,
    );
    assert_eq!(
        sample(
            &EffectStackSnapshot::new(&records, vec![a, b]).unwrap(),
            "x",
            &number(0.0)
        ),
        Some(number(1.0))
    );
}

#[test]
fn not_animatable_values_do_not_contribute_but_effect_timing_survives() {
    let mut records = AnimationRecords::new();
    let mut a = input(
        &mut records,
        &[("x", number(10.0))],
        CompositeOperation::Replace,
    );
    a.effect.as_mut().unwrap().not_animatable.push(key("x"));
    let snapshot = EffectStackSnapshot::new(&records, vec![a]).unwrap();
    assert_eq!(sample(&snapshot, "x", &number(0.0)), None);
    // Empty targeted-property coverage is vacuously complete, not a timing rejection.
    assert_eq!(snapshot.replacements().unwrap().len(), 1);
}

#[test]
fn property_specific_accumulation_keeps_the_effect_operand_second() {
    let mut records = AnimationRecords::new();
    let ratio = |a, b| {
        InterpolableValue::computed(ComputedValue::single(ComputedComponent::Ratio(
            ComputedRatio::new(a, b).unwrap(),
        )))
    };
    let a = input(
        &mut records,
        &[("x", ratio(1.0, 2.0))],
        CompositeOperation::Replace,
    );
    let b = input(
        &mut records,
        &[("x", ratio(3.0, 4.0))],
        CompositeOperation::Accumulate,
    );
    assert_eq!(
        sample(
            &EffectStackSnapshot::new(&records, vec![a, b]).unwrap(),
            "x",
            &ratio(9.0, 1.0)
        ),
        Some(ratio(3.0, 4.0))
    );
}

#[test]
fn transform_addition_keeps_underlying_before_effect_value() {
    use surgeist_animation::interpolation::TransformValue;
    use surgeist_animation::transform::{
        CssPixels, ResolvedTransformList, ScaleFactor, TransformFunction,
    };
    let transform = |function| {
        InterpolableValue::transform(TransformValue::resolved(ResolvedTransformList::new(vec![
            function,
        ])))
    };
    let translate = TransformFunction::TranslateX(CssPixels::new(10.0).unwrap());
    let scale = TransformFunction::ScaleX(ScaleFactor::new(2.0).unwrap());
    let mut records = AnimationRecords::new();
    let a = input(
        &mut records,
        &[("x", transform(translate))],
        CompositeOperation::Replace,
    );
    let b = input(
        &mut records,
        &[("x", transform(scale))],
        CompositeOperation::Add,
    );
    let value = sample(
        &EffectStackSnapshot::new(&records, vec![a, b]).unwrap(),
        "x",
        &number(0.0),
    )
    .unwrap();
    let InterpolableValue::Transform(value) = value else {
        panic!("transform required");
    };
    let value = value.resolved_list().unwrap();
    assert_eq!(value.functions(), &[translate, scale]);
    assert_eq!(
        value.to_matrix().unwrap().coefficients(),
        [2.0, 0.0, 0.0, 1.0, 10.0, 0.0]
    );
}

#[test]
fn property_operation_failure_keeps_animation_and_keyframe_diagnostics() {
    use surgeist_animation::composition::PropertyOperationErrorKind;
    use surgeist_animation::keyframe::KeyframeSampleStage;
    let mut records = AnimationRecords::new();
    let a = input(
        &mut records,
        &[("x", number(f64::MAX))],
        CompositeOperation::Replace,
    );
    let b = input(
        &mut records,
        &[("x", number(f64::MAX))],
        CompositeOperation::Add,
    );
    let b_id = b.animation.clone();
    let snapshot = EffectStackSnapshot::new(&records, vec![a, b]).unwrap();
    let error = snapshot
        .sample(
            EffectTargetId::new(1),
            &key("x"),
            KeyframeTarget::Available(&number(0.0)),
            &AnimationTypeOperations::new(AnimationType::ByComputedValue),
        )
        .unwrap_err();
    assert_eq!(error.animation(), &b_id);
    assert_eq!(
        error.keyframe_error().stage(),
        KeyframeSampleStage::Composition
    );
    assert!(matches!(
        error.keyframe_error().operation_error().unwrap().kind(),
        PropertyOperationErrorKind::Interpolation(_)
    ));
}

#[test]
fn snapshot_survives_registration_retirement_and_retains_captured_times() {
    let mut records = AnimationRecords::new();
    let a = input(
        &mut records,
        &[("x", number(1.0))],
        CompositeOperation::Replace,
    );
    let b = input(
        &mut records,
        &[("x", number(2.0))],
        CompositeOperation::Replace,
    );
    let a_id = a.animation.clone();
    let b_id = b.animation.clone();
    let snapshot = EffectStackSnapshot::new(&records, vec![a, b]).unwrap();
    records.remove_animation(&a_id).unwrap();
    records.remove_animation(&b_id).unwrap();
    assert_eq!(sample(&snapshot, "x", &number(0.0)), Some(number(2.0)));
    let decisions = snapshot.replacements().unwrap();
    assert_eq!(decisions[0].animation(), &a_id);
    assert_eq!(decisions[0].current_time().unwrap().as_millis(), 100.0);
    assert_eq!(
        decisions[0]
            .timeline()
            .unwrap()
            .current_time()
            .unwrap()
            .as_millis(),
        100.0
    );
}

#[test]
fn replacement_requires_in_effect_monotonic_and_document_candidate_facts() {
    let mut records = AnimationRecords::new();
    let original = input(
        &mut records,
        &[("x", number(1.0))],
        CompositeOperation::Replace,
    );
    let cover = input(
        &mut records,
        &[("x", number(2.0))],
        CompositeOperation::Replace,
    );
    let cases = [
        (false, true, true, AnimationReplaceState::Active),
        (true, false, true, AnimationReplaceState::Active),
        (true, true, false, AnimationReplaceState::Active),
        (true, true, true, AnimationReplaceState::Persisted),
    ];
    for (in_document, monotonic, fill, replace) in cases {
        let mut candidate = original.clone();
        candidate.in_document = in_document;
        if !monotonic {
            candidate.timeline = Some(TimelineSnapshot::new(
                TimelineId::new(1),
                Some(TimelineTime::from_millis(100.0).unwrap()),
                TimelineMonotonicity::NonMonotonic,
                None,
            ));
        }
        state(&mut candidate, EffectPlayState::Finished, replace);
        if !fill {
            candidate.effect.as_mut().unwrap().timing =
                EffectTiming::try_new(EffectTimingOptions {
                    duration_millis: 100.0,
                    ..Default::default()
                })
                .unwrap();
        }
        let mut covering = cover.clone();
        covering.in_document = in_document;
        covering.timeline = candidate.timeline;
        state(
            &mut covering,
            EffectPlayState::Finished,
            AnimationReplaceState::Active,
        );
        assert!(
            EffectStackSnapshot::new(&records, vec![candidate, covering])
                .unwrap()
                .replacements()
                .unwrap()
                .is_empty()
        );
    }
}

#[test]
fn construction_rejects_mismatched_effects_and_invalid_property_exclusions() {
    let mut records = AnimationRecords::new();
    let original = input(
        &mut records,
        &[("x", number(1.0))],
        CompositeOperation::Replace,
    );
    let mut changed = original.clone();
    changed.effect = None;
    assert_eq!(
        EffectStackSnapshot::new(&records, vec![changed])
            .unwrap_err()
            .kind(),
        EffectStackErrorKind::EffectMismatch
    );
    let mut changed = original.clone();
    changed.effect.as_mut().unwrap().timing = EffectTiming::default();
    assert_eq!(
        EffectStackSnapshot::new(&records, vec![changed])
            .unwrap_err()
            .kind(),
        EffectStackErrorKind::EffectMismatch
    );
    let mut changed = original;
    changed
        .effect
        .as_mut()
        .unwrap()
        .not_animatable
        .push(key("unknown"));
    assert_eq!(
        EffectStackSnapshot::new(&records, vec![changed])
            .unwrap_err()
            .kind(),
        EffectStackErrorKind::PropertyPolicy
    );
}

#[test]
fn one_update_rejects_conflicting_timeline_and_target_observations() {
    let mut records = AnimationRecords::new();
    let first = input(
        &mut records,
        &[("x", number(1.0))],
        CompositeOperation::Replace,
    );
    let mut second = input(
        &mut records,
        &[("x", number(2.0))],
        CompositeOperation::Replace,
    );
    second.timeline = Some(TimelineSnapshot::new(
        TimelineId::new(1),
        Some(TimelineTime::from_millis(200.0).unwrap()),
        TimelineMonotonicity::Monotonic,
        None,
    ));
    assert_eq!(
        EffectStackSnapshot::new(&records, vec![first.clone(), second.clone()])
            .unwrap_err()
            .kind(),
        EffectStackErrorKind::TimelineMismatch
    );
    second.timeline = first.timeline;
    second.in_document = false;
    assert_eq!(
        EffectStackSnapshot::new(&records, vec![first, second])
            .unwrap_err()
            .kind(),
        EffectStackErrorKind::TargetMismatch
    );
}
