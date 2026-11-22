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
use surgeist_animation::playback::{
    AnimationRecords, EffectFacts, PlaybackCommand, PlaybackContext,
};
use surgeist_animation::progress::UnitRatio;
use surgeist_animation::time::TimelineTime;
use surgeist_animation::timeline::{TimelineId, TimelineMonotonicity, TimelineSnapshot};
use surgeist_animation::timing::{
    AnimationReplaceState, EffectPlayState, EffectTiming, EffectTimingOptions, FillMode,
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
    let animation = created.animation().clone();
    records
        .apply(
            &animation,
            PlaybackCommand::SetStartTime(Some(TimelineTime::from_millis(0.0).unwrap())),
            &context,
        )
        .unwrap();
    let setup = records
        .snapshot(&animation, Some(timeline))
        .unwrap()
        .effect()
        .unwrap()
        .clone();
    AnimationStackInput {
        animation,
        timeline: Some(timeline),
        replace_state: AnimationReplaceState::Active,
        order: AnimationOrder::Defined {
            class: AnimationClass::new(0),
            inter_class: None,
            class_specific: None,
        },
        target: Some(EffectTargetId::new(1)),
        prescribed_by_markup: false,
        in_document: true,
        effect: Some(EffectStackEffect::new(
            setup,
            timing,
            frames(values, None),
            composite,
            vec![],
        )),
    }
}
fn state(
    records: &mut AnimationRecords,
    input: &mut AnimationStackInput,
    play: EffectPlayState,
    replace: AnimationReplaceState,
) {
    let context = PlaybackContext::new(input.timeline.into_iter().collect()).unwrap();
    let command = match play {
        EffectPlayState::Idle => PlaybackCommand::Cancel,
        EffectPlayState::Finished => {
            PlaybackCommand::SetStartTime(Some(TimelineTime::from_millis(0.0).unwrap()))
        }
        EffectPlayState::Running => {
            PlaybackCommand::SetStartTime(Some(TimelineTime::from_millis(50.0).unwrap()))
        }
        EffectPlayState::Paused => PlaybackCommand::Pause,
    };
    records.apply(&input.animation, command, &context).unwrap();
    input.replace_state = replace;
    assert_eq!(
        records
            .snapshot(&input.animation, input.timeline)
            .unwrap()
            .observation()
            .unwrap()
            .play_state(),
        play
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

fn grid_value(first_length: f64, second_length: f64, second_percentage: f64) -> InterpolableValue {
    use surgeist_animation::computed::{
        ComputedDimension, ComputedNumeric, DimensionKind, DimensionPercentage,
    };
    use surgeist_animation::grid::{
        GridValue, LineNames, TemplateTrackList, TrackBreadth, TrackSection, TrackSize,
    };
    let size = |length, percentage| {
        TrackSize::breadth(
            TrackBreadth::length_percentage(ComputedNumeric::mix(
                DimensionPercentage::new(
                    ComputedDimension::new(DimensionKind::Length, length).unwrap(),
                    percentage,
                )
                .unwrap(),
            ))
            .unwrap(),
        )
    };
    InterpolableValue::grid(GridValue::Template(
        TemplateTrackList::new(
            LineNames::new(vec![]),
            vec![
                (
                    TrackSection::Single(size(first_length, 0.0)),
                    LineNames::new(vec![]),
                ),
                (
                    TrackSection::Single(size(second_length, second_percentage)),
                    LineNames::new(vec![]),
                ),
            ],
        )
        .unwrap(),
    ))
}

#[test]
fn grid_default_adapter_adds_computed_tracks() {
    use surgeist_animation::composition::PropertyValueOperations;
    let operations = AnimationTypeOperations::new(AnimationType::ByComputedValue);
    assert_eq!(
        operations.add(
            &key("grid-template-columns"),
            &grid_value(40.0, 0.0, 0.125),
            &grid_value(80.0, 60.0, 0.0)
        ),
        Ok(grid_value(120.0, 60.0, 0.125))
    );
}

#[test]
fn grid_default_adapter_accumulates_computed_tracks() {
    use surgeist_animation::composition::PropertyValueOperations;
    let operations = AnimationTypeOperations::new(AnimationType::ByComputedValue);
    assert_eq!(
        operations.accumulate(
            &key("grid-template-columns"),
            &grid_value(40.0, 0.0, 0.125),
            &grid_value(80.0, 60.0, 0.0)
        ),
        Ok(grid_value(120.0, 60.0, 0.125))
    );
}

fn assert_grid_stack_composes(operation: CompositeOperation) {
    let mut records = AnimationRecords::new();
    let low = input(
        &mut records,
        &[("grid-template-columns", grid_value(40.0, 0.0, 0.125))],
        CompositeOperation::Replace,
    );
    let high = input(
        &mut records,
        &[("grid-template-columns", grid_value(80.0, 60.0, 0.0))],
        operation,
    );
    let high_id = high.animation.clone();
    let snapshot = EffectStackSnapshot::new(&records, vec![high, low]).unwrap();
    let base = grid_value(1000.0, 20.0, 0.5);
    let result = snapshot
        .sample(
            EffectTargetId::new(1),
            &key("grid-template-columns"),
            KeyframeTarget::Available(&base),
            &AnimationTypeOperations::new(AnimationType::ByComputedValue),
        )
        .expect("Grid composition must reach its numerical owner")
        .unwrap();
    assert_eq!(result.animation(), &high_id);
    // The lower Replace supplies 40px/12.5%; the top effect adds 80px/60px once.
    assert_eq!(result.value(), &grid_value(120.0, 60.0, 0.125));
    assert_eq!(base, grid_value(1000.0, 20.0, 0.5));
}

#[test]
fn grid_default_stack_adds_tracks_against_evolving_underlying() {
    assert_grid_stack_composes(CompositeOperation::Add);
}

#[test]
fn grid_default_stack_accumulates_tracks_against_evolving_underlying() {
    assert_grid_stack_composes(CompositeOperation::Accumulate);
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
    state(
        &mut records,
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
    a.timeline = Some(TimelineSnapshot::new(
        TimelineId::new(2),
        None,
        TimelineMonotonicity::Monotonic,
        None,
    ));
    assert_eq!(
        EffectStackSnapshot::new(&records, vec![a])
            .unwrap_err()
            .kind(),
        EffectStackErrorKind::Playback
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
        &mut records,
        &mut orphan,
        EffectPlayState::Idle,
        AnimationReplaceState::Active,
    );
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
        &mut records,
        &mut c,
        EffectPlayState::Finished,
        AnimationReplaceState::Persisted,
    );
    let saved_time = records
        .snapshot(&a.animation, a.timeline)
        .unwrap()
        .observation()
        .unwrap()
        .current_time();
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
        state(&mut records, &mut b, play, replace);
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
        &mut records,
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
    let timeline = a.timeline;
    let snapshot = EffectStackSnapshot::new(&records, vec![a, b]).unwrap();
    let context = PlaybackContext::new(timeline.into_iter().collect()).unwrap();
    records
        .apply(&a_id, PlaybackCommand::Cancel, &context)
        .unwrap();
    records
        .apply(&b_id, PlaybackCommand::Cancel, &context)
        .unwrap();
    for id in [&a_id, &b_id] {
        let snapshot = records.snapshot(id, timeline).unwrap();
        if let Some(token) = snapshot.finish_notification() {
            records.process_finish(token, &context).unwrap();
        }
    }
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
        state(
            &mut records,
            &mut candidate,
            EffectPlayState::Finished,
            replace,
        );
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
            &mut records,
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

fn color(
    space: surgeist_animation::interpolation::ColorInterpolationSpace,
    value: f64,
) -> InterpolableValue {
    use surgeist_animation::interpolation::InterpolableColor;
    InterpolableValue::color(
        InterpolableColor::from_straight_components(
            space,
            value,
            value * 0.5,
            -value,
            UnitRatio::new(1.0).unwrap(),
        )
        .unwrap(),
    )
}

#[test]
fn color_nonadditive_addition_returns_the_effect_operand() {
    use surgeist_animation::composition::PropertyValueOperations;
    use surgeist_animation::interpolation::ColorInterpolationSpace::{Oklab, Srgb};
    let operations = AnimationTypeOperations::new(AnimationType::ByComputedValue);
    // Selected Values 4: a type without addition selects B. No color-space
    // conversion, clipping, or component sum is part of this operation.
    for (underlying, effect) in [
        (color(Srgb, 0.25), color(Srgb, 0.75)),
        (color(Srgb, f64::MAX), color(Oklab, -f64::MAX)),
    ] {
        assert_eq!(
            operations.add(&key("color"), &underlying, &effect),
            Ok(effect.clone())
        );
        assert_eq!(
            operations.add(&key("color"), &effect, &underlying),
            Ok(underlying)
        );
    }
}

#[test]
fn color_nonadditive_accumulation_uses_the_same_effect_operand_fallback() {
    use surgeist_animation::composition::PropertyValueOperations;
    use surgeist_animation::interpolation::ColorInterpolationSpace::{Oklab, Srgb};
    let operations = AnimationTypeOperations::new(AnimationType::ByComputedValue);
    // Values 4's unspecified accumulation delegates to addition, so B survives.
    let underlying = color(Srgb, f64::MAX);
    let effect = color(Oklab, -f64::MAX);
    assert_eq!(
        operations.accumulate(&key("color"), &underlying, &effect),
        Ok(effect.clone())
    );
    assert_eq!(
        operations.accumulate(&key("color"), &effect, &underlying),
        Ok(underlying)
    );
}

fn assert_color_stack_fallback(operation: CompositeOperation) {
    use surgeist_animation::interpolation::ColorInterpolationSpace::{Oklab, Srgb};
    let mut records = AnimationRecords::new();
    let underlying = color(Srgb, 0.25);
    let effect = color(Oklab, 0.75);
    let lower = input(
        &mut records,
        &[("color", underlying)],
        CompositeOperation::Replace,
    );
    let mut higher = input(&mut records, &[("color", effect.clone())], operation);
    // Exercise composition of both interval endpoints before real interpolation.
    state(
        &mut records,
        &mut higher,
        EffectPlayState::Running,
        AnimationReplaceState::Active,
    );
    let higher_id = higher.animation.clone();
    let snapshot = EffectStackSnapshot::new(&records, vec![higher, lower]).unwrap();
    let sampled = snapshot.sample(
        EffectTargetId::new(1),
        &key("color"),
        KeyframeTarget::Available(&color(Srgb, 0.0)),
        &AnimationTypeOperations::new(AnimationType::ByComputedValue),
    );
    let sampled = sampled
        .expect("nonadditive color composition returns a value")
        .unwrap();
    assert_eq!(sampled.value(), &effect);
    assert_eq!(sampled.animation(), &higher_id);
}

#[test]
fn color_nonadditive_addition_samples_through_the_effect_stack() {
    assert_color_stack_fallback(CompositeOperation::Add);
}

#[test]
fn color_nonadditive_accumulation_samples_through_the_effect_stack() {
    assert_color_stack_fallback(CompositeOperation::Accumulate);
}

#[test]
fn integration_cancelled_animation_cannot_contribute_stale_finished_data() {
    use surgeist_animation::playback::PlaybackCommand;
    let mut records = AnimationRecords::new();
    let stale = input(
        &mut records,
        &[("x", number(7.0))],
        CompositeOperation::Replace,
    );
    let context = PlaybackContext::new(vec![stale.timeline.unwrap()]).unwrap();
    records
        .apply(
            &stale.animation,
            PlaybackCommand::SetStartTime(Some(TimelineTime::from_millis(0.0).unwrap())),
            &context,
        )
        .unwrap();
    assert_eq!(
        records
            .snapshot(&stale.animation, stale.timeline)
            .unwrap()
            .observation()
            .unwrap()
            .play_state(),
        EffectPlayState::Finished
    );
    records
        .apply(&stale.animation, PlaybackCommand::Cancel, &context)
        .unwrap();
    assert_eq!(
        records
            .snapshot(&stale.animation, stale.timeline)
            .unwrap()
            .observation()
            .unwrap()
            .play_state(),
        EffectPlayState::Idle
    );
    let snapshot = EffectStackSnapshot::new(&records, vec![stale]).unwrap();
    assert_eq!(sample(&snapshot, "x", &number(0.0)), None);
    assert!(snapshot.replacements().unwrap().is_empty());
}

#[test]
fn integration_same_duration_reassociation_rejects_old_computed_frames() {
    use surgeist_animation::playback::PlaybackCommand;
    let mut records = AnimationRecords::new();
    let stale = input(
        &mut records,
        &[("x", number(7.0))],
        CompositeOperation::Replace,
    );
    let context = PlaybackContext::new(vec![stale.timeline.unwrap()]).unwrap();
    let another = records
        .register_effect(EffectFacts::from_timing(&timing()))
        .unwrap();
    records
        .apply(
            &stale.animation,
            PlaybackCommand::AssignEffect(Some(another)),
            &context,
        )
        .unwrap();
    assert_eq!(
        EffectStackSnapshot::new(&records, vec![stale])
            .unwrap_err()
            .kind(),
        EffectStackErrorKind::EffectMismatch
    );
}

#[test]
fn integration_same_duration_setup_revision_rejects_old_computed_frames() {
    let mut records = AnimationRecords::new();
    let stale = input(
        &mut records,
        &[("x", number(7.0))],
        CompositeOperation::Replace,
    );
    let context = PlaybackContext::new(vec![stale.timeline.unwrap()]).unwrap();
    let setup = records
        .snapshot(&stale.animation, stale.timeline)
        .unwrap()
        .effect()
        .unwrap()
        .clone();
    records
        .update_effect(setup.id(), setup.facts(), &context)
        .unwrap();
    assert_eq!(
        EffectStackSnapshot::new(&records, vec![stale])
            .unwrap_err()
            .kind(),
        EffectStackErrorKind::EffectMismatch
    );
}

fn integration_scale(value: f64) -> InterpolableValue {
    use surgeist_animation::interpolation::TransformValue;
    use surgeist_animation::transform::{ResolvedTransformList, ScaleFactor, TransformFunction};
    InterpolableValue::transform(TransformValue::resolved(ResolvedTransformList::new(vec![
        TransformFunction::ScaleX(ScaleFactor::new(value).unwrap()),
    ])))
}

fn integration_shadow_items(values: &[(f64, f64)]) -> InterpolableValue {
    use surgeist_animation::interpolation::{ColorInterpolationSpace, InterpolableColor};
    use surgeist_animation::shadow::{ShadowGeometry, ShadowInput, ShadowKind, ShadowList};
    let color = InterpolableColor::from_straight_components(
        ColorInterpolationSpace::Srgb,
        1.0,
        0.0,
        0.0,
        UnitRatio::new(1.0).unwrap(),
    )
    .unwrap();
    InterpolableValue::shadow(
        ShadowList::new(
            ShadowKind::Box,
            values
                .iter()
                .map(|&(x, blur)| {
                    ShadowInput::new(
                        ShadowKind::Box,
                        ShadowGeometry::new([x, 0.0, blur, 0.0]).unwrap(),
                        color,
                        false,
                    )
                    .unwrap()
                })
                .collect(),
        )
        .unwrap(),
    )
}

#[test]
fn integration_default_operations_use_transform_accumulation() {
    use surgeist_animation::composition::PropertyValueOperations;
    let operations = AnimationTypeOperations::new(AnimationType::ByComputedValue);
    assert_eq!(
        operations.accumulate(
            &key("transform"),
            &integration_scale(2.0),
            &integration_scale(3.0)
        ),
        Ok(integration_scale(4.0))
    );
}

#[test]
fn integration_default_operations_use_shadow_addition() {
    use surgeist_animation::composition::PropertyValueOperations;
    let operations = AnimationTypeOperations::new(AnimationType::ByComputedValue);
    assert_eq!(
        operations.add(
            &key("box-shadow"),
            &integration_shadow_items(&[(1.0, 2.0)]),
            &integration_shadow_items(&[(3.0, 4.0)])
        ),
        Ok(integration_shadow_items(&[(1.0, 2.0), (3.0, 4.0)]))
    );
}

#[test]
fn integration_default_operations_use_shadow_accumulation() {
    use surgeist_animation::composition::PropertyValueOperations;
    let operations = AnimationTypeOperations::new(AnimationType::ByComputedValue);
    assert_eq!(
        operations.accumulate(
            &key("box-shadow"),
            &integration_shadow_items(&[(1.0, 2.0)]),
            &integration_shadow_items(&[(3.0, 4.0)])
        ),
        Ok(integration_shadow_items(&[(4.0, 6.0)]))
    );
}

fn filter_value(inputs: Vec<surgeist_animation::filter::FilterInput>) -> InterpolableValue {
    InterpolableValue::filter(surgeist_animation::filter::FilterList::new(inputs).unwrap())
}

#[test]
fn filter_default_addition_preserves_underlying_first_order() {
    use surgeist_animation::composition::compose_value;
    use surgeist_animation::filter::{AmountFunction, FilterAmountInput, FilterInput};
    let blur = FilterInput::blur_pixels(2.).unwrap();
    let brightness = FilterInput::amount(
        AmountFunction::Brightness,
        FilterAmountInput::Number(InterpolableNumber::new(3.).unwrap()),
    )
    .unwrap();
    let underlying = filter_value(vec![blur.clone()]);
    let effect = filter_value(vec![brightness.clone()]);
    let operations = AnimationTypeOperations::new(AnimationType::ByComputedValue);
    assert_eq!(
        compose_value(
            &key("filter"),
            CompositeOperation::Add,
            &underlying,
            &effect,
            &operations
        ),
        Ok(filter_value(vec![blur, brightness]))
    );
}

#[test]
fn filter_default_accumulation_selects_the_complete_effect_including_urls() {
    use surgeist_animation::composition::compose_value;
    use surgeist_animation::filter::FilterInput;
    let underlying = filter_value(vec![FilterInput::blur_pixels(2.).unwrap()]);
    let effect = filter_value(vec![
        FilterInput::url_reference(std::sync::Arc::from("url(#effect)")).unwrap(),
    ]);
    let operations = AnimationTypeOperations::new(AnimationType::ByComputedValue);
    assert_eq!(
        compose_value(
            &key("filter"),
            CompositeOperation::Accumulate,
            &underlying,
            &effect,
            &operations
        ),
        Ok(effect.clone())
    );
    assert_eq!(
        compose_value(
            &key("filter"),
            CompositeOperation::Accumulate,
            &effect,
            &underlying,
            &operations
        ),
        Ok(underlying)
    );
}

fn filter_stack(operation: CompositeOperation) -> (InterpolableValue, InterpolableValue) {
    use surgeist_animation::filter::{AmountFunction, FilterAmountInput, FilterInput};
    let blur = FilterInput::blur_pixels(2.).unwrap();
    let brightness = FilterInput::amount(
        AmountFunction::Brightness,
        FilterAmountInput::Number(InterpolableNumber::new(3.).unwrap()),
    )
    .unwrap();
    let underlying = filter_value(vec![blur.clone()]);
    let effect = filter_value(vec![brightness.clone()]);
    let expected = if operation == CompositeOperation::Add {
        filter_value(vec![blur, brightness])
    } else {
        effect.clone()
    };
    let mut records = AnimationRecords::new();
    let low = input(
        &mut records,
        &[("filter", underlying.clone())],
        CompositeOperation::Replace,
    );
    let mut high = input(&mut records, &[("filter", effect)], operation);
    // The active midpoint must compose both endpoints using the real default
    // operations before filter interpolation, rather than returning an endpoint.
    state(
        &mut records,
        &mut high,
        EffectPlayState::Running,
        AnimationReplaceState::Active,
    );
    let snapshot = EffectStackSnapshot::new(&records, vec![high, low]).unwrap();
    (sample(&snapshot, "filter", &underlying).unwrap(), expected)
}

#[test]
fn filter_default_addition_flows_through_the_active_effect_stack() {
    let (actual, expected) = filter_stack(CompositeOperation::Add);
    assert_eq!(actual, expected);
}

#[test]
fn filter_default_accumulation_flows_through_the_active_effect_stack() {
    let (actual, expected) = filter_stack(CompositeOperation::Accumulate);
    assert_eq!(actual, expected);
}

#[test]
fn integration_landed_list_operations_fold_through_canonical_stack_samples() {
    for (underlying, effect, operation, expected) in [
        (
            integration_scale(2.0),
            integration_scale(3.0),
            CompositeOperation::Accumulate,
            integration_scale(4.0),
        ),
        (
            integration_shadow_items(&[(1.0, 2.0)]),
            integration_shadow_items(&[(3.0, 4.0)]),
            CompositeOperation::Add,
            integration_shadow_items(&[(1.0, 2.0), (3.0, 4.0)]),
        ),
        (
            integration_shadow_items(&[(1.0, 2.0)]),
            integration_shadow_items(&[(3.0, 4.0)]),
            CompositeOperation::Accumulate,
            integration_shadow_items(&[(4.0, 6.0)]),
        ),
    ] {
        let mut records = AnimationRecords::new();
        let lower = input(
            &mut records,
            &[("x", underlying.clone())],
            CompositeOperation::Replace,
        );
        let mut higher = input(&mut records, &[("x", effect)], operation);
        state(
            &mut records,
            &mut higher,
            EffectPlayState::Running,
            AnimationReplaceState::Active,
        );
        let snapshot = EffectStackSnapshot::new(&records, vec![higher, lower]).unwrap();
        assert_eq!(sample(&snapshot, "x", &underlying), Some(expected));
    }
}

#[test]
fn integration_list_failures_preserve_their_owning_typed_diagnostics() {
    use surgeist_animation::composition::{PropertyOperationErrorKind, PropertyValueOperations};
    use surgeist_animation::shadow::ShadowErrorKind;
    use surgeist_animation::transform::TransformErrorKind;
    let operations = AnimationTypeOperations::new(AnimationType::ByComputedValue);
    let error = operations
        .accumulate(
            &key("transform"),
            &integration_scale(f64::MAX),
            &integration_scale(f64::MAX),
        )
        .unwrap_err();
    let PropertyOperationErrorKind::Transform(error) = error.kind() else {
        panic!("transform diagnostic required");
    };
    assert_eq!(error.kind(), TransformErrorKind::NonFinite);
    let shadow = integration_shadow_items(&[(f64::MAX, 0.0)]);
    let error = operations
        .accumulate(&key("box-shadow"), &shadow, &shadow)
        .unwrap_err();
    let PropertyOperationErrorKind::Shadow(error) = error.kind() else {
        panic!("shadow diagnostic required");
    };
    assert_eq!(error.kind(), ShadowErrorKind::NonFinite);
    assert_eq!(error.shadow_index(), Some(0));
}

#[test]
fn palette_nonadditive_addition_returns_the_effect_operand() {
    use surgeist_animation::composition::PropertyValueOperations;
    use surgeist_animation::palette::{PaletteSource, PaletteValue};
    let underlying = InterpolableValue::palette(PaletteValue::source(PaletteSource::new()));
    let effect = InterpolableValue::palette(PaletteValue::source(PaletteSource::new()));
    let operations = AnimationTypeOperations::new(AnimationType::ByComputedValue);
    // Font-palette is nonadditive. The selected Values 4 operation retains B.
    assert_eq!(
        operations.add(&key("font-palette"), &underlying, &effect),
        Ok(effect.clone())
    );
    assert_eq!(
        operations.add(&key("font-palette"), &effect, &underlying),
        Ok(underlying)
    );
}

#[test]
fn palette_unspecified_accumulation_uses_the_nonadditive_effect_operand() {
    use surgeist_animation::composition::PropertyValueOperations;
    use surgeist_animation::palette::{PaletteSource, PaletteValue};
    let underlying = InterpolableValue::palette(PaletteValue::source(PaletteSource::new()));
    let effect = InterpolableValue::palette(PaletteValue::source(PaletteSource::new()));
    let operations = AnimationTypeOperations::new(AnimationType::ByComputedValue);
    // Values 4 delegates unspecified accumulation to the addition operation.
    assert_eq!(
        operations.accumulate(&key("font-palette"), &underlying, &effect),
        Ok(effect.clone())
    );
    assert_eq!(
        operations.accumulate(&key("font-palette"), &effect, &underlying),
        Ok(underlying)
    );
}

#[test]
fn palette_default_addition_samples_through_the_effect_stack() {
    assert_palette_stack_fallback(CompositeOperation::Add);
}

#[test]
fn palette_default_accumulation_samples_through_the_effect_stack() {
    assert_palette_stack_fallback(CompositeOperation::Accumulate);
}

fn assert_palette_stack_fallback(operation: CompositeOperation) {
    use surgeist_animation::interpolation::InterpolationProgress;
    use surgeist_animation::palette::{PaletteNode, PaletteSource, PaletteValue};
    let mut records = AnimationRecords::new();
    let normal = InterpolableValue::palette(PaletteValue::source(PaletteSource::new()));
    let light = PaletteSource::new();
    let dark = PaletteSource::new();
    let lower = input(
        &mut records,
        &[("font-palette", normal.clone())],
        CompositeOperation::Replace,
    );
    let mut higher = input(&mut records, &[], operation);
    higher.effect.as_mut().unwrap().keyframes = ComputedKeyframes::new(
        [light.clone(), dark.clone()]
            .into_iter()
            .enumerate()
            .map(|(index, source)| ComputedKeyframeInput {
                specified_offset: None,
                computed_offset: UnitRatio::new(index as f64).unwrap(),
                easing: Easing::linear(),
                composite: None,
                properties: vec![(
                    key("font-palette"),
                    InterpolableValue::palette(PaletteValue::source(source)),
                )],
            })
            .collect(),
        KeyframeEndpointSource::Native,
    )
    .unwrap();
    state(
        &mut records,
        &mut higher,
        EffectPlayState::Running,
        AnimationReplaceState::Active,
    );
    let higher_id = higher.animation.clone();
    let snapshot = EffectStackSnapshot::new(&records, vec![higher, lower]).unwrap();
    let result = snapshot
        .sample(
            EffectTargetId::new(1),
            &key("font-palette"),
            KeyframeTarget::Available(&normal),
            &AnimationTypeOperations::new(AnimationType::ByComputedValue),
        )
        .expect("nonadditive palette composition returns a value")
        .unwrap();
    assert_eq!(result.animation(), &higher_id);
    let InterpolableValue::Palette(value) = result.value() else {
        panic!("expected palette mix")
    };
    assert_eq!(
        value.nodes(),
        &[
            PaletteNode::Source(light),
            PaletteNode::Source(dark),
            PaletteNode::MixOklab {
                first: 0,
                second: 1,
                progress: InterpolationProgress::new(0.5).unwrap()
            },
        ]
    );
}
