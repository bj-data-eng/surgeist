use surgeist_animation::playback::{
    AnimationRecords, EffectFacts, PlaybackCommand, PlaybackContext, PlaybackCoordinates,
    PlaybackErrorKind, PlaybackRequest,
};
use surgeist_animation::time::{AnimationTime, PlaybackRate, TimelineTime};
use surgeist_animation::timeline::{
    AnimationSnapshot, TimelineId, TimelineMonotonicity, TimelineSnapshot,
};
use surgeist_animation::timing::{
    AnimationReplaceState, EffectPlayState, EffectTiming, EffectTimingOptions, FillMode,
};

fn timeline(id: u64, time: Option<f64>) -> TimelineSnapshot {
    TimelineSnapshot::new(
        TimelineId::new(id),
        time.map(|v| TimelineTime::from_millis(v).unwrap()),
        TimelineMonotonicity::Monotonic,
        None,
    )
}

fn record_context() -> PlaybackContext {
    PlaybackContext::new(vec![timeline(1, None), timeline(2, None)]).unwrap()
}

fn coordinates(
    start: Option<f64>,
    hold: Option<f64>,
    rate: f64,
    timeline: Option<TimelineSnapshot>,
) -> PlaybackCoordinates {
    PlaybackCoordinates::new(
        AnimationSnapshot::new(
            PlaybackRate::new(rate).unwrap(),
            start.map(|v| TimelineTime::from_millis(v).unwrap()),
            timeline,
        ),
        hold.map(|v| AnimationTime::from_millis(v).unwrap()),
    )
}

#[test]
fn creation_has_unresolved_times_unit_rate_and_unique_global_order() {
    let mut records = AnimationRecords::new();
    let a = records
        .create(None, None, &record_context())
        .unwrap()
        .animation()
        .clone();
    let b = records
        .create(Some(TimelineId::new(2)), None, &record_context())
        .unwrap()
        .animation()
        .clone();
    let c = records
        .create(Some(TimelineId::new(1)), None, &record_context())
        .unwrap()
        .animation()
        .clone();
    let snapshots = [
        records.snapshot(&a, None).unwrap(),
        records.snapshot(&b, Some(timeline(2, Some(10.0)))).unwrap(),
        records.snapshot(&c, Some(timeline(1, Some(10.0)))).unwrap(),
    ];
    assert!(snapshots[0].creation_order() < snapshots[1].creation_order());
    assert!(snapshots[1].creation_order() < snapshots[2].creation_order());
    for snapshot in snapshots {
        let coordinates = snapshot.coordinates();
        assert_eq!(coordinates.animation().start_time(), None);
        assert_eq!(coordinates.hold_time(), None);
        assert_eq!(coordinates.animation().playback_rate().value(), 1.0);
        assert_eq!(coordinates.current_time().unwrap(), None);
        assert_eq!(
            snapshot.observation().unwrap().play_state(),
            EffectPlayState::Idle
        );
    }
    assert_eq!(records.animations().collect::<Vec<_>>(), vec![a, b, c]);
}

#[test]
fn effect_transfer_detaches_previous_animation_atomically_and_noops_identical_assignment() {
    let mut records = AnimationRecords::new();
    let effect = records
        .register_effect(EffectFacts::from_timing(&EffectTiming::default()))
        .unwrap();
    let a = records
        .create(None, Some(&effect), &record_context())
        .unwrap()
        .animation()
        .clone();
    let b = records
        .create(None, None, &record_context())
        .unwrap()
        .animation()
        .clone();
    let captured = records.snapshot(&a, None).unwrap();
    assert_eq!(
        records
            .apply(
                &a,
                PlaybackCommand::AssignEffect(Some(effect.clone())),
                &record_context()
            )
            .unwrap(),
        Vec::<PlaybackRequest>::new()
    );
    assert_eq!(
        records
            .apply(
                &b,
                PlaybackCommand::AssignEffect(Some(effect.clone())),
                &record_context()
            )
            .unwrap(),
        Vec::<PlaybackRequest>::new()
    );
    assert_eq!(records.effect_owner(&effect).unwrap(), Some(b.clone()));
    assert_eq!(records.snapshot(&a, None).unwrap().effect(), None);
    assert_eq!(
        records.snapshot(&b, None).unwrap().effect().unwrap().id(),
        &effect
    );
    assert_eq!(captured.effect().unwrap().id(), &effect);
    assert_eq!(
        captured.creation_order(),
        records.snapshot(&a, None).unwrap().creation_order()
    );
    assert_eq!(
        records
            .apply(&b, PlaybackCommand::AssignEffect(None), &record_context())
            .unwrap(),
        Vec::<PlaybackRequest>::new()
    );
    assert_eq!(records.effect_owner(&effect).unwrap(), None);
}

#[test]
fn creation_with_an_owned_effect_transfers_ownership_without_reordering_previous_records() {
    let mut records = AnimationRecords::new();
    let effect = records
        .register_effect(EffectFacts::try_new(100.0).unwrap())
        .unwrap();
    let a = records
        .create(None, Some(&effect), &record_context())
        .unwrap()
        .animation()
        .clone();
    let a_order = records.snapshot(&a, None).unwrap().creation_order();
    let b = records
        .create(None, Some(&effect), &record_context())
        .unwrap()
        .animation()
        .clone();
    assert_eq!(records.snapshot(&a, None).unwrap().effect(), None);
    assert_eq!(records.effect_owner(&effect).unwrap(), Some(b.clone()));
    assert_eq!(
        records.snapshot(&a, None).unwrap().creation_order(),
        a_order
    );
    assert!(records.snapshot(&b, None).unwrap().creation_order() > a_order);
}

#[test]
fn foreign_handles_are_rejected_without_detaching_either_models_effect() {
    let mut first = AnimationRecords::new();
    let mut other = AnimationRecords::new();
    let effect = first
        .register_effect(EffectFacts::try_new(10.0).unwrap())
        .unwrap();
    let a = first
        .create(None, Some(&effect), &record_context())
        .unwrap()
        .animation()
        .clone();
    let foreign_effect = other
        .register_effect(EffectFacts::try_new(20.0).unwrap())
        .unwrap();
    let foreign_animation = other
        .create(None, Some(&foreign_effect), &record_context())
        .unwrap()
        .animation()
        .clone();
    assert_ne!(effect, foreign_effect);
    assert_ne!(a, foreign_animation);
    let original = first.snapshot(&a, None).unwrap();
    assert_eq!(
        first
            .apply(
                &a,
                PlaybackCommand::AssignEffect(Some(foreign_effect.clone())),
                &record_context()
            )
            .unwrap_err()
            .kind(),
        &PlaybackErrorKind::ForeignEffect
    );
    assert_eq!(
        first
            .apply(
                &foreign_animation,
                PlaybackCommand::AssignEffect(Some(effect.clone())),
                &record_context()
            )
            .unwrap_err()
            .kind(),
        &PlaybackErrorKind::ForeignAnimation
    );
    assert!(
        first
            .create(None, Some(&foreign_effect), &record_context())
            .is_err()
    );
    assert_eq!(first.snapshot(&a, None).unwrap(), original);
    assert_eq!(first.effect_owner(&effect).unwrap(), Some(a.clone()));
    assert_eq!(
        other.effect_owner(&foreign_effect).unwrap(),
        Some(foreign_animation)
    );
    assert_eq!(first.animations().collect::<Vec<_>>(), vec![a]);
}

#[test]
fn transfer_releases_the_targets_old_effect_and_the_sources_animation() {
    let mut records = AnimationRecords::new();
    let left = records
        .register_effect(EffectFacts::try_new(10.0).unwrap())
        .unwrap();
    let right = records
        .register_effect(EffectFacts::try_new(20.0).unwrap())
        .unwrap();
    let a = records
        .create(None, Some(&left), &record_context())
        .unwrap()
        .animation()
        .clone();
    let b = records
        .create(None, Some(&right), &record_context())
        .unwrap()
        .animation()
        .clone();
    let captured = records.snapshot(&a, None).unwrap();
    records
        .apply(
            &a,
            PlaybackCommand::AssignEffect(Some(right.clone())),
            &record_context(),
        )
        .unwrap();
    assert_eq!(records.effect_owner(&left).unwrap(), None);
    assert_eq!(records.effect_owner(&right).unwrap(), Some(a.clone()));
    assert_eq!(records.snapshot(&b, None).unwrap().effect(), None);
    assert_eq!(
        records.snapshot(&a, None).unwrap().effect().unwrap().id(),
        &right
    );
    assert_eq!(captured.effect().unwrap().id(), &left);
}

#[test]
fn removed_handles_stay_stale_and_live_effects_require_explicit_detachment() {
    let mut records = AnimationRecords::new();
    let effect = records
        .register_effect(EffectFacts::try_new(10.0).unwrap())
        .unwrap();
    let a = records
        .create(None, Some(&effect), &record_context())
        .unwrap()
        .animation()
        .clone();
    assert_eq!(
        records.remove_effect(&effect).unwrap_err().kind(),
        &PlaybackErrorKind::EffectInUse
    );
    records.remove_animation(&a).unwrap();
    assert_eq!(
        records.snapshot(&a, None).unwrap_err().kind(),
        &PlaybackErrorKind::StaleAnimation
    );
    assert_eq!(records.effect_owner(&effect).unwrap(), None);
    records.remove_effect(&effect).unwrap();
    assert_eq!(
        records.effect_owner(&effect).unwrap_err().kind(),
        &PlaybackErrorKind::StaleEffect
    );
    let next_effect = records
        .register_effect(EffectFacts::try_new(20.0).unwrap())
        .unwrap();
    let b = records
        .create(None, Some(&next_effect), &record_context())
        .unwrap()
        .animation()
        .clone();
    assert_ne!(a, b);
    assert_ne!(effect, next_effect);
    assert!(
        records
            .apply(
                &b,
                PlaybackCommand::AssignEffect(Some(effect.clone())),
                &record_context()
            )
            .is_err()
    );
    assert_eq!(records.effect_owner(&next_effect).unwrap(), Some(b));
}

#[test]
fn supplied_timeline_distinguishes_missing_inactive_wrong_and_detached() {
    let mut records = AnimationRecords::new();
    let a = records
        .create(Some(TimelineId::new(1)), None, &record_context())
        .unwrap()
        .animation()
        .clone();
    assert_eq!(
        records.snapshot(&a, None).unwrap_err().kind(),
        &PlaybackErrorKind::MissingTimeline {
            expected: TimelineId::new(1)
        }
    );
    assert_eq!(
        records
            .snapshot(&a, Some(timeline(2, Some(10.0))))
            .unwrap_err()
            .kind(),
        &PlaybackErrorKind::TimelineMismatch {
            expected: TimelineId::new(1),
            actual: TimelineId::new(2)
        }
    );
    assert_eq!(
        records
            .snapshot(&a, Some(timeline(1, None)))
            .unwrap()
            .coordinates()
            .current_time()
            .unwrap(),
        None
    );
    assert_eq!(
        records
            .apply(
                &a,
                PlaybackCommand::AssignTimeline(Some(TimelineId::new(1))),
                &record_context()
            )
            .unwrap(),
        Vec::<PlaybackRequest>::new()
    );
    assert_eq!(
        records
            .apply(&a, PlaybackCommand::AssignTimeline(None), &record_context())
            .unwrap(),
        Vec::<PlaybackRequest>::new()
    );
    assert_eq!(
        records
            .snapshot(&a, Some(timeline(1, None)))
            .unwrap_err()
            .kind(),
        &PlaybackErrorKind::UnexpectedTimeline {
            actual: TimelineId::new(1)
        }
    );
    assert_eq!(records.snapshot(&a, None).unwrap().timeline(), None);
}

#[test]
fn hold_wins_over_start_inactive_and_missing_timeline_and_overflowing_arithmetic() {
    for timeline in [
        None,
        Some(timeline(1, None)),
        Some(timeline(1, Some(f64::MAX))),
    ] {
        let value = coordinates(Some(-f64::MAX), Some(-7.0), 2.0, timeline);
        assert_eq!(value.current_time().unwrap().unwrap().as_millis(), -7.0);
    }
}

#[test]
fn coordinate_evaluation_preserves_sign_zero_and_recoverable_large_intermediates() {
    for (start, rate, now, expected) in [
        (20.0, 2.0, 100.0, 160.0),
        (20.0, -2.0, 100.0, -160.0),
        (20.0, 0.0, 100.0, 0.0),
        (-f64::MAX, 0.25, f64::MAX, f64::MAX * 0.5),
    ] {
        assert_eq!(
            coordinates(Some(start), None, rate, Some(timeline(1, Some(now))))
                .current_time()
                .unwrap()
                .unwrap()
                .as_millis(),
            expected
        );
    }
    let error = coordinates(Some(0.0), None, 2.0, Some(timeline(1, Some(f64::MAX))))
        .current_time()
        .unwrap_err();
    assert!(matches!(error.kind(), PlaybackErrorKind::Arithmetic(source)
    if source.kind() == surgeist_animation::time::TimeErrorKind::Unrepresentable {
        operation: surgeist_animation::time::TimeOperation::TimelineToAnimation,
    }));
    assert_eq!(
        coordinates(None, None, 1.0, Some(timeline(1, Some(100.0))))
            .current_time()
            .unwrap(),
        None
    );
    assert_eq!(
        coordinates(Some(0.0), None, 1.0, None)
            .current_time()
            .unwrap(),
        None
    );
}

#[test]
fn effect_facts_validate_end_and_accept_source_infinity() {
    for end in [-1.0, f64::NAN, f64::NEG_INFINITY] {
        assert!(EffectFacts::try_new(end).is_err());
    }
    assert_eq!(
        EffectFacts::try_new(f64::INFINITY)
            .unwrap()
            .end_time_millis(),
        f64::INFINITY
    );
    let timing = EffectTiming::try_new(EffectTimingOptions {
        duration_millis: 100.0,
        start_delay_millis: 20.0,
        ..EffectTimingOptions::default()
    })
    .unwrap();
    assert_eq!(EffectFacts::from_timing(&timing).end_time_millis(), 120.0);
}

#[test]
fn observations_derive_coherent_play_state_and_feed_effect_sampling() {
    let effect = EffectTiming::try_new(EffectTimingOptions {
        duration_millis: 100.0,
        fill: FillMode::Both,
        ..EffectTimingOptions::default()
    })
    .unwrap();
    let facts = EffectFacts::from_timing(&effect);
    for (start, hold, rate, now, expected) in [
        (None, None, 1.0, None, EffectPlayState::Idle),
        (None, Some(150.0), 1.0, Some(10.0), EffectPlayState::Paused),
        (Some(0.0), None, 1.0, Some(50.0), EffectPlayState::Running),
        (Some(0.0), None, 1.0, Some(150.0), EffectPlayState::Finished),
        (Some(0.0), None, -1.0, Some(10.0), EffectPlayState::Finished),
        (Some(0.0), None, 0.0, Some(10.0), EffectPlayState::Running),
        (Some(0.0), None, 1.0, None, EffectPlayState::Running),
    ] {
        let observation = coordinates(start, hold, rate, Some(timeline(1, now)))
            .observe(Some(facts), AnimationReplaceState::Active)
            .unwrap();
        assert_eq!(observation.play_state(), expected);
        assert!(effect.sample(observation.effect_input()).is_ok());
    }
    let no_effect = coordinates(Some(0.0), None, 1.0, Some(timeline(1, Some(10.0))))
        .observe(None, AnimationReplaceState::Active)
        .unwrap();
    assert_eq!(no_effect.play_state(), EffectPlayState::Finished);
    assert_eq!(
        effect
            .sample(no_effect.effect_input())
            .unwrap()
            .local_time(),
        None
    );
}
