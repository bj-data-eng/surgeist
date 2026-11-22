use surgeist_animation::playback::{
    AnimationId, AnimationRecords, CompletionState, EffectFacts, EffectId, EffectReadiness,
    PlaybackCommand as Command, PlaybackContext, PlaybackErrorKind, PlaybackEventKind,
    PlaybackOperation, PlaybackRequest as Request, ReadinessInput, ReadinessToken,
};
use surgeist_animation::time::{AnimationTime, PlaybackRate, TimelineTime};
use surgeist_animation::timeline::{TimelineId, TimelineMonotonicity, TimelineSnapshot};
use surgeist_animation::timing::EffectPlayState;

fn frame(now: Option<f64>, monotonic: bool) -> TimelineSnapshot {
    TimelineSnapshot::new(
        TimelineId::new(1),
        now.map(|v| TimelineTime::from_millis(v).unwrap()),
        if monotonic {
            TimelineMonotonicity::Monotonic
        } else {
            TimelineMonotonicity::NonMonotonic
        },
        None,
    )
}
fn context(now: Option<f64>) -> PlaybackContext {
    PlaybackContext::new(vec![frame(now, true)]).unwrap()
}
fn time(value: f64) -> AnimationTime {
    AnimationTime::from_millis(value).unwrap()
}
fn rate(value: f64) -> PlaybackRate {
    PlaybackRate::new(value).unwrap()
}
fn setup(end: f64) -> (AnimationRecords, AnimationId, EffectId) {
    let mut model = AnimationRecords::new();
    let effect = model
        .register_effect(EffectFacts::try_new(end).unwrap())
        .unwrap();
    let id = model
        .create(Some(TimelineId::new(1)), Some(&effect), &context(Some(0.0)))
        .unwrap()
        .animation()
        .clone();
    (model, id, effect)
}
fn ready(model: &mut AnimationRecords, id: &AnimationId, now: f64) -> Vec<Request> {
    let token = model
        .snapshot(id, Some(frame(Some(now), true)))
        .unwrap()
        .pending_task()
        .unwrap()
        .clone();
    model
        .complete_ready(
            &token,
            ReadinessInput::new(frame(Some(now), true), EffectReadiness::Ready),
            &context(Some(now)),
        )
        .unwrap()
}
fn running(end: f64) -> (AnimationRecords, AnimationId, EffectId) {
    let (mut model, id, effect) = setup(end);
    model
        .apply(
            &id,
            Command::Play { auto_rewind: true },
            &context(Some(0.0)),
        )
        .unwrap();
    ready(&mut model, &id, 0.0);
    (model, id, effect)
}
fn current(model: &AnimationRecords, id: &AnimationId, now: Option<f64>) -> Option<f64> {
    model
        .snapshot(id, Some(frame(now, true)))
        .unwrap()
        .observation()
        .unwrap()
        .current_time()
        .map(AnimationTime::as_millis)
}
fn pending(model: &AnimationRecords, id: &AnimationId, now: f64) -> ReadinessToken {
    model
        .snapshot(id, Some(frame(Some(now), true)))
        .unwrap()
        .pending_task()
        .unwrap()
        .clone()
}

#[test]
fn creation_has_distinct_resolved_ready_and_pending_finished_identities() {
    let (model, id, _) = setup(100.0);
    let snapshot = model.snapshot(&id, Some(frame(Some(0.0), true))).unwrap();
    assert_eq!(snapshot.ready().state(), CompletionState::Resolved);
    assert_eq!(snapshot.finished().state(), CompletionState::Pending);
    assert_ne!(snapshot.ready().id(), snapshot.finished().id());
}

#[test]
fn seek_adjusts_start_once_and_rejects_unresolved_seek_atomically() {
    let (mut model, id, _) = running(1000.0);
    model
        .apply(
            &id,
            Command::SetPlaybackRate(rate(2.0)),
            &context(Some(0.0)),
        )
        .unwrap();
    model
        .apply(&id, Command::Seek(Some(time(30.0))), &context(Some(100.0)))
        .unwrap();
    let snapshot = model.snapshot(&id, Some(frame(Some(100.0), true))).unwrap();
    assert_eq!(
        snapshot
            .coordinates()
            .animation()
            .start_time()
            .unwrap()
            .as_millis(),
        85.0
    );
    assert_eq!(current(&model, &id, Some(100.0)), Some(30.0));
    assert_eq!(
        model
            .apply(&id, Command::Seek(None), &context(Some(100.0)))
            .unwrap_err()
            .kind(),
        &PlaybackErrorKind::TypeError
    );
    assert_eq!(
        model.snapshot(&id, Some(frame(Some(100.0), true))).unwrap(),
        snapshot
    );
}

#[test]
fn start_assignment_snapshots_old_rate_before_applying_pending_rate() {
    let (mut model, id, _) = running(1000.0);
    model
        .apply(
            &id,
            Command::UpdatePlaybackRate(rate(2.0)),
            &context(Some(10.0)),
        )
        .unwrap();
    assert_eq!(
        model
            .snapshot(&id, Some(frame(Some(10.0), true)))
            .unwrap()
            .pending_playback_rate(),
        Some(rate(2.0))
    );
    model
        .apply(&id, Command::SetStartTime(None), &context(Some(10.0)))
        .unwrap();
    let snapshot = model.snapshot(&id, Some(frame(Some(10.0), true))).unwrap();
    assert_eq!(snapshot.coordinates().hold_time(), Some(time(10.0)));
    assert_eq!(
        snapshot.coordinates().animation().playback_rate(),
        rate(2.0)
    );
    assert!(snapshot.pending_task().is_none());
    assert_eq!(snapshot.ready().state(), CompletionState::Resolved);
}

#[test]
fn inactive_start_assignment_clears_hold_before_previous_time_capture() {
    let (mut model, id, _) = setup(100.0);
    model
        .apply(&id, Command::Seek(Some(time(7.0))), &context(None))
        .unwrap();
    model
        .apply(
            &id,
            Command::SetStartTime(Some(TimelineTime::from_millis(10.0).unwrap())),
            &context(None),
        )
        .unwrap();
    assert_eq!(current(&model, &id, None), None);
    model
        .apply(&id, Command::SetStartTime(None), &context(None))
        .unwrap();
    assert_eq!(current(&model, &id, None), None);
}

#[test]
fn pending_task_replacement_reuses_ready_identity_and_rejects_stale_callbacks() {
    let (mut model, id, _) = setup(100.0);
    let effects = model
        .apply(
            &id,
            Command::Play { auto_rewind: true },
            &context(Some(0.0)),
        )
        .unwrap();
    assert!(
        effects
            .iter()
            .any(|effect| matches!(effect, Request::ScheduleReadiness(_)))
    );
    let original = model.snapshot(&id, Some(frame(Some(0.0), true))).unwrap();
    let first = original.pending_task().unwrap().clone();
    assert_eq!(original.ready().state(), CompletionState::Pending);
    model
        .apply(&id, Command::Pause, &context(Some(0.0)))
        .unwrap();
    let second = pending(&model, &id, 0.0);
    assert_ne!(first, second);
    assert_eq!(
        model
            .snapshot(&id, Some(frame(Some(0.0), true)))
            .unwrap()
            .ready()
            .id(),
        original.ready().id()
    );
    let before = model.snapshot(&id, Some(frame(Some(0.0), true))).unwrap();
    assert_eq!(
        model
            .complete_ready(
                &first,
                ReadinessInput::new(frame(Some(0.0), true), EffectReadiness::Ready),
                &context(Some(0.0))
            )
            .unwrap_err()
            .kind(),
        &PlaybackErrorKind::StaleReadiness
    );
    assert_eq!(
        model.snapshot(&id, Some(frame(Some(0.0), true))).unwrap(),
        before
    );
    model
        .apply(
            &id,
            Command::Play { auto_rewind: true },
            &context(Some(0.0)),
        )
        .unwrap();
    assert_ne!(pending(&model, &id, 0.0), second);
    ready(&mut model, &id, 10.0);
    assert_eq!(current(&model, &id, Some(20.0)), Some(10.0));
}

#[test]
fn readiness_requires_setup_and_active_matching_timeline_and_aligns_at_ready_time() {
    let (mut model, id, _) = setup(100.0);
    model
        .apply(
            &id,
            Command::Play { auto_rewind: true },
            &context(Some(0.0)),
        )
        .unwrap();
    let token = pending(&model, &id, 0.0);
    for input in [
        ReadinessInput::new(frame(Some(10.0), true), EffectReadiness::Pending),
        ReadinessInput::new(frame(None, true), EffectReadiness::Ready),
    ] {
        assert_eq!(
            model
                .complete_ready(&token, input, &context(Some(20.0)))
                .unwrap_err()
                .kind(),
            &PlaybackErrorKind::NotReady
        );
        assert_eq!(pending(&model, &id, 20.0), token);
    }
    model
        .complete_ready(
            &token,
            ReadinessInput::new(frame(Some(10.0), true), EffectReadiness::Ready),
            &context(Some(20.0)),
        )
        .unwrap();
    assert_eq!(current(&model, &id, Some(20.0)), Some(10.0));
}

#[test]
fn seek_completes_pending_pause_and_applies_staged_rate_synchronously() {
    let (mut model, id, _) = running(1000.0);
    model
        .apply(&id, Command::Pause, &context(Some(10.0)))
        .unwrap();
    let token = pending(&model, &id, 10.0);
    model
        .apply(
            &id,
            Command::UpdatePlaybackRate(rate(3.0)),
            &context(Some(10.0)),
        )
        .unwrap();
    let outcomes = model
        .apply(&id, Command::Seek(Some(time(40.0))), &context(Some(10.0)))
        .unwrap();
    let snapshot = model.snapshot(&id, Some(frame(Some(100.0), true))).unwrap();
    assert_eq!(
        snapshot.observation().unwrap().play_state(),
        EffectPlayState::Paused
    );
    assert_eq!(
        snapshot.observation().unwrap().current_time(),
        Some(time(40.0))
    );
    assert_eq!(
        snapshot.coordinates().animation().playback_rate(),
        rate(3.0)
    );
    assert!(
        outcomes
            .iter()
            .any(|v| matches!(v, Request::CancelReadiness(cancelled) if cancelled == &token))
    );
    assert_eq!(snapshot.ready().state(), CompletionState::Resolved);
}

#[test]
fn pause_captures_old_rate_at_ready_time_before_applying_pending_rate() {
    let (mut model, id, _) = running(1000.0);
    model
        .apply(&id, Command::Pause, &context(Some(10.0)))
        .unwrap();
    model
        .apply(
            &id,
            Command::UpdatePlaybackRate(rate(2.0)),
            &context(Some(10.0)),
        )
        .unwrap();
    ready(&mut model, &id, 20.0);
    assert_eq!(current(&model, &id, Some(100.0)), Some(20.0));
    assert_eq!(
        model
            .snapshot(&id, Some(frame(Some(100.0), true)))
            .unwrap()
            .coordinates()
            .animation()
            .playback_rate(),
        rate(2.0)
    );
    assert!(
        model
            .apply(&id, Command::Pause, &context(Some(100.0)))
            .unwrap()
            .is_empty()
    );
}

#[test]
fn synchronous_and_seamless_rate_changes_follow_different_paths() {
    let (mut model, id, _) = running(1000.0);
    model
        .apply(
            &id,
            Command::SetPlaybackRate(rate(2.0)),
            &context(Some(10.0)),
        )
        .unwrap();
    assert_eq!(current(&model, &id, Some(10.0)), Some(10.0));
    assert_eq!(current(&model, &id, Some(20.0)), Some(30.0));
    model
        .apply(
            &id,
            Command::UpdatePlaybackRate(rate(0.0)),
            &context(Some(20.0)),
        )
        .unwrap();
    assert_eq!(
        model
            .snapshot(&id, Some(frame(Some(20.0), true)))
            .unwrap()
            .pending_playback_rate(),
        Some(rate(0.0))
    );
    ready(&mut model, &id, 30.0);
    let snapshot = model.snapshot(&id, Some(frame(Some(100.0), true))).unwrap();
    assert_eq!(
        snapshot.observation().unwrap().current_time(),
        Some(time(50.0))
    );
    assert_eq!(
        snapshot.observation().unwrap().play_state(),
        EffectPlayState::Running
    );
    assert!(snapshot.coordinates().animation().start_time().is_some());
}

#[test]
fn finite_nonmonotonic_rate_sign_change_flips_start_offset() {
    let (mut model, id, _) = setup(100.0);
    let ctx = PlaybackContext::new(vec![frame(Some(50.0), false)]).unwrap();
    model
        .apply(
            &id,
            Command::SetStartTime(Some(TimelineTime::from_millis(20.0).unwrap())),
            &ctx,
        )
        .unwrap();
    model
        .apply(&id, Command::SetPlaybackRate(rate(-1.0)), &ctx)
        .unwrap();
    let snapshot = model.snapshot(&id, Some(frame(Some(50.0), false))).unwrap();
    assert_eq!(
        snapshot
            .coordinates()
            .animation()
            .start_time()
            .unwrap()
            .as_millis(),
        80.0
    );
    assert_eq!(
        snapshot.observation().unwrap().current_time(),
        Some(time(30.0))
    );
}

#[test]
fn finish_clamps_normal_crossings_but_preserves_sought_overshoot() {
    let (mut model, id, _) = running(100.0);
    model
        .apply(&id, Command::Tick, &context(Some(150.0)))
        .unwrap();
    assert_eq!(current(&model, &id, Some(150.0)), Some(100.0));
    model
        .apply(&id, Command::Seek(Some(time(170.0))), &context(Some(150.0)))
        .unwrap();
    model
        .apply(&id, Command::Tick, &context(Some(200.0)))
        .unwrap();
    assert_eq!(current(&model, &id, Some(200.0)), Some(170.0));
}

#[test]
fn finish_notification_rechecks_live_state_and_uses_current_completion_identity() {
    let (mut model, id, effect) = running(100.0);
    model
        .apply(&id, Command::Tick, &context(Some(150.0)))
        .unwrap();
    let snapshot = model.snapshot(&id, Some(frame(Some(150.0), true))).unwrap();
    let token = snapshot.finish_notification().unwrap().clone();
    let completion = snapshot.finished().id().clone();
    model
        .update_effect(
            &effect,
            EffectFacts::try_new(300.0).unwrap(),
            &context(Some(150.0)),
        )
        .unwrap();
    assert!(
        model
            .process_finish(&token, &context(Some(150.0)))
            .unwrap()
            .is_empty()
    );
    assert_eq!(
        model
            .snapshot(&id, Some(frame(Some(150.0), true)))
            .unwrap()
            .finished()
            .id(),
        &completion
    );
    model
        .apply(&id, Command::Tick, &context(Some(350.0)))
        .unwrap();
    let token = model
        .snapshot(&id, Some(frame(Some(350.0), true)))
        .unwrap()
        .finish_notification()
        .unwrap()
        .clone();
    let outcomes = model.process_finish(&token, &context(Some(350.0))).unwrap();
    assert!(
        matches!(outcomes.first(), Some(Request::SettleCompletion { completion: settled, state: CompletionState::Resolved }) if settled == &completion)
    );
    assert!(
        matches!(outcomes.last(), Some(Request::QueueEvent(event)) if event.kind() == PlaybackEventKind::Finish)
    );
    model
        .apply(&id, Command::Seek(Some(time(0.0))), &context(Some(350.0)))
        .unwrap();
    assert_ne!(
        model
            .snapshot(&id, Some(frame(Some(350.0), true)))
            .unwrap()
            .finished()
            .id(),
        &completion
    );
}

#[test]
fn synchronous_finish_cancels_queued_notification_and_snapshots_payload() {
    let (mut model, id, _) = running(100.0);
    model
        .apply(&id, Command::Tick, &context(Some(150.0)))
        .unwrap();
    let token = model
        .snapshot(&id, Some(frame(Some(150.0), true)))
        .unwrap()
        .finish_notification()
        .unwrap()
        .clone();
    let effects = model
        .apply(&id, Command::Finish, &context(Some(150.0)))
        .unwrap();
    assert!(
        matches!(effects.first(), Some(Request::CancelFinishNotification(cancelled)) if cancelled == &token)
    );
    let event = effects
        .iter()
        .find_map(|v| {
            if let Request::QueueEvent(event) = v {
                Some(event.clone())
            } else {
                None
            }
        })
        .unwrap();
    model
        .apply(&id, Command::Seek(Some(time(0.0))), &context(Some(150.0)))
        .unwrap();
    assert_eq!(event.current_time(), Some(time(100.0)));
    assert_eq!(
        event.timeline_time(),
        Some(TimelineTime::from_millis(150.0).unwrap())
    );
    assert_eq!(
        model
            .process_finish(&token, &context(Some(150.0)))
            .unwrap_err()
            .kind(),
        &PlaybackErrorKind::StaleFinishNotification
    );
}

#[test]
fn cancel_aborts_pending_completions_replaces_them_and_retains_effect() {
    let (mut model, id, effect) = setup(100.0);
    assert!(
        model
            .apply(&id, Command::Cancel, &context(Some(0.0)))
            .unwrap()
            .is_empty()
    );
    model
        .apply(
            &id,
            Command::Play { auto_rewind: true },
            &context(Some(0.0)),
        )
        .unwrap();
    model
        .apply(
            &id,
            Command::UpdatePlaybackRate(rate(2.0)),
            &context(Some(0.0)),
        )
        .unwrap();
    let before = model.snapshot(&id, Some(frame(Some(0.0), true))).unwrap();
    let effects = model.apply(&id, Command::Cancel, &context(None)).unwrap();
    let after = model.snapshot(&id, Some(frame(None, true))).unwrap();
    assert_eq!(
        after.observation().unwrap().play_state(),
        EffectPlayState::Idle
    );
    assert_eq!(after.coordinates().animation().playback_rate(), rate(2.0));
    assert_eq!(after.effect().unwrap().id(), &effect);
    assert_ne!(before.ready().id(), after.ready().id());
    assert_ne!(before.finished().id(), after.finished().id());
    assert_eq!(after.ready().state(), CompletionState::Resolved);
    assert_eq!(after.finished().state(), CompletionState::Pending);
    let aborts: Vec<_> = effects
        .iter()
        .filter_map(|v| {
            if let Request::SettleCompletion {
                completion,
                state: CompletionState::Aborted,
            } = v
            {
                Some(completion)
            } else {
                None
            }
        })
        .collect();
    assert_eq!(aborts, vec![before.ready().id(), before.finished().id()]);
    assert!(
        matches!(effects.last(), Some(Request::QueueEvent(event)) if event.kind() == PlaybackEventKind::Cancel && event.current_time().is_none() && event.timeline_time().is_none())
    );
}

#[test]
fn pending_finish_notification_survives_cancel_and_rechecks_new_live_finished_state() {
    let (mut model, id, _) = running(100.0);
    model
        .apply(&id, Command::Tick, &context(Some(150.0)))
        .unwrap();
    let old = model.snapshot(&id, Some(frame(Some(150.0), true))).unwrap();
    let token = old.finish_notification().unwrap().clone();
    model
        .apply(&id, Command::Cancel, &context(Some(150.0)))
        .unwrap();
    model
        .apply(
            &id,
            Command::SetStartTime(Some(TimelineTime::from_millis(0.0).unwrap())),
            &context(Some(150.0)),
        )
        .unwrap();
    let new = model.snapshot(&id, Some(frame(Some(150.0), true))).unwrap();
    assert_ne!(old.finished().id(), new.finished().id());
    assert_eq!(new.finish_notification(), Some(&token));
    let effects = model.process_finish(&token, &context(Some(150.0))).unwrap();
    assert!(
        matches!(effects.first(), Some(Request::SettleCompletion {completion, state: CompletionState::Resolved}) if completion == new.finished().id())
    );
}

#[test]
fn association_changes_reschedule_readiness_and_update_displaced_owner() {
    let (mut model, a, effect) = running(100.0);
    model
        .apply(&a, Command::Pause, &context(Some(20.0)))
        .unwrap();
    let old_token = pending(&model, &a, 20.0);
    let b = model
        .create(Some(TimelineId::new(1)), None, &context(Some(20.0)))
        .unwrap()
        .animation()
        .clone();
    let effects = model
        .apply(
            &b,
            Command::AssignEffect(Some(effect.clone())),
            &context(Some(20.0)),
        )
        .unwrap();
    assert_eq!(model.effect_owner(&effect).unwrap(), Some(b));
    assert_ne!(pending(&model, &a, 20.0), old_token);
    assert!(
        effects
            .iter()
            .any(|v| matches!(v, Request::CancelReadiness(token) if token == &old_token))
    );
}

#[test]
fn invalid_reverse_and_finish_preserve_state_and_staged_rates() {
    let (mut model, id, _) = setup(f64::INFINITY);
    model
        .apply(
            &id,
            Command::UpdatePlaybackRate(rate(-1.0)),
            &context(Some(0.0)),
        )
        .unwrap();
    let before = model.snapshot(&id, Some(frame(Some(0.0), true))).unwrap();
    assert_eq!(
        model
            .apply(
                &id,
                Command::Play { auto_rewind: true },
                &context(Some(0.0))
            )
            .unwrap_err()
            .kind(),
        &PlaybackErrorKind::InvalidState {
            operation: PlaybackOperation::Play
        }
    );
    assert_eq!(
        model.snapshot(&id, Some(frame(Some(0.0), true))).unwrap(),
        before
    );
    assert!(model.apply(&id, Command::Reverse, &context(None)).is_err());
    model
        .apply(
            &id,
            Command::SetPlaybackRate(rate(0.0)),
            &context(Some(0.0)),
        )
        .unwrap();
    let before = model.snapshot(&id, Some(frame(Some(0.0), true))).unwrap();
    assert_eq!(
        model
            .apply(&id, Command::Finish, &context(Some(0.0)))
            .unwrap_err()
            .kind(),
        &PlaybackErrorKind::InvalidState {
            operation: PlaybackOperation::Finish
        }
    );
    assert_eq!(
        model.snapshot(&id, Some(frame(Some(0.0), true))).unwrap(),
        before
    );
}

#[test]
fn numeric_failure_is_atomic_and_readiness_tokens_are_owner_checked() {
    let (mut model, id, _) = running(f64::INFINITY);
    model
        .apply(
            &id,
            Command::SetPlaybackRate(rate(2.0)),
            &context(Some(0.0)),
        )
        .unwrap();
    model
        .apply(
            &id,
            Command::UpdatePlaybackRate(rate(f64::MAX)),
            &context(Some(1.0)),
        )
        .unwrap();
    let token = pending(&model, &id, 1.0);
    let before = model.snapshot(&id, Some(frame(Some(1.0), true))).unwrap();
    assert!(
        model
            .complete_ready(
                &token,
                ReadinessInput::new(frame(Some(f64::MAX), true), EffectReadiness::Ready),
                &context(Some(f64::MAX))
            )
            .is_err()
    );
    assert_eq!(
        model.snapshot(&id, Some(frame(Some(1.0), true))).unwrap(),
        before
    );
    let (mut foreign, _, _) = setup(100.0);
    assert_eq!(
        foreign
            .complete_ready(
                &token,
                ReadinessInput::new(frame(Some(1.0), true), EffectReadiness::Ready),
                &context(Some(1.0))
            )
            .unwrap_err()
            .kind(),
        &PlaybackErrorKind::ForeignAnimation
    );
}

#[test]
fn reverse_rewinds_at_zero_then_runs_backwards_and_clamps_at_zero() {
    let (mut model, id, _) = running(100.0);
    model
        .apply(&id, Command::Reverse, &context(Some(0.0)))
        .unwrap();
    ready(&mut model, &id, 0.0);
    assert_eq!(current(&model, &id, Some(0.0)), Some(100.0));
    assert_eq!(current(&model, &id, Some(50.0)), Some(50.0));
    model
        .apply(&id, Command::Tick, &context(Some(150.0)))
        .unwrap();
    let snapshot = model.snapshot(&id, Some(frame(Some(150.0), true))).unwrap();
    assert_eq!(
        snapshot.observation().unwrap().current_time(),
        Some(time(0.0))
    );
    assert_eq!(
        snapshot.observation().unwrap().play_state(),
        EffectPlayState::Finished
    );
}

#[test]
fn zero_length_ready_settlement_precedes_finished_settlement_and_event() {
    let (mut model, id, _) = setup(0.0);
    let requests = model
        .apply(
            &id,
            Command::Play { auto_rewind: true },
            &context(Some(0.0)),
        )
        .unwrap();
    let ready_position = requests
        .iter()
        .position(|v| matches!(v, Request::ScheduleReadiness(_)))
        .unwrap();
    let finish_position = requests
        .iter()
        .position(|v| matches!(v, Request::ScheduleFinishNotification(_)))
        .unwrap();
    assert!(ready_position < finish_position);
    let before = model.snapshot(&id, Some(frame(Some(0.0), true))).unwrap();
    let ready_effects = ready(&mut model, &id, 0.0);
    assert!(
        matches!(ready_effects.first(), Some(Request::SettleCompletion {completion, state:CompletionState::Resolved}) if completion == before.ready().id())
    );
    let finish_effects = model
        .process_finish(before.finish_notification().unwrap(), &context(Some(0.0)))
        .unwrap();
    assert!(
        matches!(finish_effects.first(), Some(Request::SettleCompletion {completion, state:CompletionState::Resolved}) if completion == before.finished().id())
    );
    assert!(
        matches!(finish_effects.last(), Some(Request::QueueEvent(event)) if event.current_time() == Some(time(0.0)))
    );
}

#[test]
fn invalid_reverse_restores_preexisting_staged_rate_and_readiness_identity() {
    let (mut model, id, _) = running(f64::INFINITY);
    model
        .apply(
            &id,
            Command::UpdatePlaybackRate(rate(2.0)),
            &context(Some(0.0)),
        )
        .unwrap();
    let before = model.snapshot(&id, Some(frame(Some(0.0), true))).unwrap();
    assert!(
        model
            .apply(&id, Command::Reverse, &context(Some(0.0)))
            .is_err()
    );
    assert_eq!(
        model.snapshot(&id, Some(frame(Some(0.0), true))).unwrap(),
        before
    );
}

#[test]
fn pause_unresolved_reverse_uses_effect_end_and_rejects_infinity() {
    let (mut model, id, _) = setup(100.0);
    model
        .apply(
            &id,
            Command::SetPlaybackRate(rate(-1.0)),
            &context(Some(0.0)),
        )
        .unwrap();
    model
        .apply(&id, Command::Pause, &context(Some(0.0)))
        .unwrap();
    ready(&mut model, &id, 10.0);
    assert_eq!(current(&model, &id, Some(50.0)), Some(100.0));
    let (mut infinite, id, _) = setup(f64::INFINITY);
    infinite
        .apply(
            &id,
            Command::SetPlaybackRate(rate(-1.0)),
            &context(Some(0.0)),
        )
        .unwrap();
    let before = infinite
        .snapshot(&id, Some(frame(Some(0.0), true)))
        .unwrap();
    assert_eq!(
        infinite
            .apply(&id, Command::Pause, &context(Some(0.0)))
            .unwrap_err()
            .kind(),
        &PlaybackErrorKind::InvalidState {
            operation: PlaybackOperation::Pause
        }
    );
    assert_eq!(
        infinite
            .snapshot(&id, Some(frame(Some(0.0), true)))
            .unwrap(),
        before
    );
}

#[test]
fn finished_seamless_rate_change_uses_unconstrained_time_and_handles_inactivity() {
    let (mut model, id, _) = running(100.0);
    model
        .apply(&id, Command::Tick, &context(Some(150.0)))
        .unwrap();
    model
        .apply(
            &id,
            Command::UpdatePlaybackRate(rate(-1.0)),
            &context(Some(150.0)),
        )
        .unwrap();
    assert_eq!(current(&model, &id, Some(150.0)), Some(150.0));
    assert_eq!(current(&model, &id, Some(200.0)), Some(100.0));
    let (mut inactive, id, _) = running(100.0);
    inactive
        .apply(&id, Command::Tick, &context(Some(150.0)))
        .unwrap();
    inactive
        .apply(&id, Command::UpdatePlaybackRate(rate(2.0)), &context(None))
        .unwrap();
    let snapshot = inactive.snapshot(&id, Some(frame(None, true))).unwrap();
    assert_eq!(snapshot.coordinates().animation().start_time(), None);
    assert_eq!(
        snapshot.observation().unwrap().play_state(),
        EffectPlayState::Paused
    );
    assert_eq!(
        snapshot.observation().unwrap().current_time(),
        Some(time(100.0))
    );
}

#[test]
fn effect_setup_revision_and_timeline_reassignment_invalidate_readiness_only() {
    let (mut model, id, effect) = setup(100.0);
    model
        .apply(
            &id,
            Command::Play { auto_rewind: true },
            &context(Some(0.0)),
        )
        .unwrap();
    let before = model.snapshot(&id, Some(frame(Some(0.0), true))).unwrap();
    let old_token = before.pending_task().unwrap().clone();
    model
        .update_effect(
            &effect,
            EffectFacts::try_new(100.0).unwrap(),
            &context(Some(0.0)),
        )
        .unwrap();
    let after = model.snapshot(&id, Some(frame(Some(0.0), true))).unwrap();
    assert_ne!(after.pending_task(), Some(&old_token));
    assert_ne!(
        after.effect().unwrap().revision(),
        before.effect().unwrap().revision()
    );
    assert_eq!(after.ready().id(), before.ready().id());
    let old_token = after.pending_task().unwrap().clone();
    let new_timeline = TimelineSnapshot::new(
        TimelineId::new(2),
        Some(TimelineTime::from_millis(10.0).unwrap()),
        TimelineMonotonicity::Monotonic,
        None,
    );
    let ctx = PlaybackContext::new(vec![new_timeline]).unwrap();
    model
        .apply(&id, Command::AssignTimeline(Some(TimelineId::new(2))), &ctx)
        .unwrap();
    let after = model.snapshot(&id, Some(new_timeline)).unwrap();
    assert_eq!(after.ready().id(), before.ready().id());
    assert_ne!(after.pending_task(), Some(&old_token));
    assert_eq!(
        after.pending_task().unwrap().timeline(),
        Some(TimelineId::new(2))
    );
}

#[test]
fn identical_association_does_not_require_new_facts_or_replace_pending_tokens() {
    let (mut model, id, effect) = setup(100.0);
    model
        .apply(
            &id,
            Command::Play { auto_rewind: true },
            &context(Some(0.0)),
        )
        .unwrap();
    let before = model.snapshot(&id, Some(frame(Some(0.0), true))).unwrap();
    assert!(
        model
            .apply(
                &id,
                Command::AssignTimeline(Some(TimelineId::new(1))),
                &PlaybackContext::default()
            )
            .unwrap()
            .is_empty()
    );
    assert!(
        model
            .apply(
                &id,
                Command::AssignEffect(Some(effect)),
                &PlaybackContext::default()
            )
            .unwrap()
            .is_empty()
    );
    assert_eq!(
        model.snapshot(&id, Some(frame(Some(0.0), true))).unwrap(),
        before
    );
}

#[test]
fn failed_transfer_preserves_both_active_owners_and_their_pending_work() {
    let (mut model, a, effect) = running(100.0);
    model
        .apply(&a, Command::Pause, &context(Some(20.0)))
        .unwrap();
    let second_timeline = TimelineSnapshot::new(
        TimelineId::new(2),
        Some(TimelineTime::from_millis(20.0).unwrap()),
        TimelineMonotonicity::Monotonic,
        None,
    );
    let both = PlaybackContext::new(vec![frame(Some(20.0), true), second_timeline]).unwrap();
    let b = model
        .create(Some(TimelineId::new(2)), None, &both)
        .unwrap()
        .animation()
        .clone();
    let before_a = model.snapshot(&a, Some(frame(Some(20.0), true))).unwrap();
    let before_b = model.snapshot(&b, Some(second_timeline)).unwrap();
    let only_second = PlaybackContext::new(vec![second_timeline]).unwrap();
    assert!(
        model
            .apply(
                &b,
                Command::AssignEffect(Some(effect.clone())),
                &only_second
            )
            .is_err()
    );
    assert_eq!(model.effect_owner(&effect).unwrap(), Some(a.clone()));
    assert_eq!(
        model.snapshot(&a, Some(frame(Some(20.0), true))).unwrap(),
        before_a
    );
    assert_eq!(model.snapshot(&b, Some(second_timeline)).unwrap(), before_b);
}

#[test]
fn finish_and_cancel_keep_distinct_time_spaces_and_apply_origin_once() {
    let (mut model, id, _) = setup(30.0);
    let timeline = TimelineSnapshot::new(
        TimelineId::new(1),
        Some(TimelineTime::from_millis(115.0).unwrap()),
        TimelineMonotonicity::Monotonic,
        Some(surgeist_animation::time::OriginRelativeTime::from_millis(40.0).unwrap()),
    );
    let ctx = PlaybackContext::new(vec![timeline]).unwrap();
    model
        .apply(&id, Command::SetPlaybackRate(rate(2.0)), &ctx)
        .unwrap();
    model
        .apply(
            &id,
            Command::SetStartTime(Some(TimelineTime::from_millis(100.0).unwrap())),
            &ctx,
        )
        .unwrap();
    let token = model
        .snapshot(&id, Some(timeline))
        .unwrap()
        .finish_notification()
        .unwrap()
        .clone();
    let effects = model.process_finish(&token, &ctx).unwrap();
    let finish = effects
        .into_iter()
        .find_map(|v| {
            if let Request::QueueEvent(event) = v {
                Some(event)
            } else {
                None
            }
        })
        .unwrap();
    assert_eq!(finish.current_time(), Some(time(30.0)));
    assert_eq!(finish.timeline_time().unwrap().as_millis(), 115.0);
    assert_eq!(finish.scheduled_time().unwrap().as_millis(), 155.0);
    let old_ready = model.snapshot(&id, Some(timeline)).unwrap().ready().clone();
    let effects = model.apply(&id, Command::Cancel, &ctx).unwrap();
    assert!(!effects.iter().any(|v| matches!(
        v,
        Request::SettleCompletion {
            state: CompletionState::Aborted,
            ..
        }
    )));
    assert_eq!(
        model.snapshot(&id, Some(timeline)).unwrap().ready(),
        &old_ready
    );
    let cancel = effects
        .into_iter()
        .find_map(|v| {
            if let Request::QueueEvent(event) = v {
                Some(event)
            } else {
                None
            }
        })
        .unwrap();
    assert_eq!(cancel.current_time(), None);
    assert_eq!(cancel.timeline_time().unwrap().as_millis(), 115.0);
    assert_eq!(cancel.scheduled_time().unwrap().as_millis(), 155.0);
    assert_eq!(
        finish
            .animation_snapshot()
            .start_time()
            .unwrap()
            .as_millis(),
        100.0
    );
    assert_eq!(finish.animation_snapshot().playback_rate(), rate(2.0));
}

#[test]
fn cancel_does_not_compute_unneeded_overflowing_animation_time() {
    let (mut model, id, _) = running(f64::INFINITY);
    model
        .apply(
            &id,
            Command::SetPlaybackRate(rate(2.0)),
            &context(Some(0.0)),
        )
        .unwrap();
    model
        .apply(&id, Command::Cancel, &context(Some(f64::MAX)))
        .unwrap();
    assert_eq!(current(&model, &id, Some(f64::MAX)), None);
}

#[test]
fn nonmonotonic_timeline_can_leave_finished_and_resume_inside_its_range() {
    let (mut model, id, _) = setup(100.0);
    let ctx = |now| PlaybackContext::new(vec![frame(Some(now), false)]).unwrap();
    model
        .apply(
            &id,
            Command::SetStartTime(Some(TimelineTime::from_millis(0.0).unwrap())),
            &ctx(0.0),
        )
        .unwrap();
    model.apply(&id, Command::Tick, &ctx(150.0)).unwrap();
    let token = model
        .snapshot(&id, Some(frame(Some(150.0), false)))
        .unwrap()
        .finish_notification()
        .unwrap()
        .clone();
    model.process_finish(&token, &ctx(150.0)).unwrap();
    model.apply(&id, Command::Tick, &ctx(50.0)).unwrap();
    let snapshot = model.snapshot(&id, Some(frame(Some(50.0), false))).unwrap();
    assert_eq!(
        snapshot.observation().unwrap().current_time(),
        Some(time(50.0))
    );
    assert_eq!(
        snapshot.observation().unwrap().play_state(),
        EffectPlayState::Running
    );
    assert_eq!(snapshot.finished().state(), CompletionState::Pending);
}

#[test]
fn readiness_rejects_wrong_identity_and_future_monotonic_time_without_consuming_token() {
    let (mut model, id, _) = setup(100.0);
    model
        .apply(
            &id,
            Command::Play { auto_rewind: true },
            &context(Some(0.0)),
        )
        .unwrap();
    let token = pending(&model, &id, 0.0);
    let other = TimelineSnapshot::new(
        TimelineId::new(2),
        Some(TimelineTime::from_millis(10.0).unwrap()),
        TimelineMonotonicity::Monotonic,
        None,
    );
    assert_eq!(
        model
            .complete_ready(
                &token,
                ReadinessInput::new(other, EffectReadiness::Ready),
                &context(Some(20.0))
            )
            .unwrap_err()
            .kind(),
        &PlaybackErrorKind::TimelineMismatch {
            expected: TimelineId::new(1),
            actual: TimelineId::new(2)
        }
    );
    assert_eq!(
        model
            .complete_ready(
                &token,
                ReadinessInput::new(frame(Some(30.0), true), EffectReadiness::Ready),
                &context(Some(20.0))
            )
            .unwrap_err()
            .kind(),
        &PlaybackErrorKind::InvalidReadiness
    );
    assert_eq!(pending(&model, &id, 20.0), token);
}

#[test]
fn pinned_nonmonotonic_redundant_play_cancels_work_without_settling_reused_ready() {
    let (mut model, id, _) = setup(100.0);
    let timeline = frame(Some(10.0), false);
    let ctx = PlaybackContext::new(vec![timeline]).unwrap();
    model
        .apply(&id, Command::Play { auto_rewind: true }, &ctx)
        .unwrap();
    let before = model.snapshot(&id, Some(timeline)).unwrap();
    let token = before.pending_task().unwrap().clone();
    // Selected WA1 steps9–10 cancel the task before the redundant-play return.
    // No Level2 auto-aligned-start readiness repair is imported by this model.
    let requests = model
        .apply(&id, Command::Play { auto_rewind: true }, &ctx)
        .unwrap();
    let after = model.snapshot(&id, Some(timeline)).unwrap();
    assert_eq!(requests, vec![Request::CancelReadiness(token.clone())]);
    assert_eq!(after.pending_task(), None);
    assert_eq!(after.ready(), before.ready());
    assert_eq!(after.ready().state(), CompletionState::Pending);
    assert_eq!(
        after.observation().unwrap().play_state(),
        EffectPlayState::Running
    );
    assert_eq!(
        model
            .complete_ready(
                &token,
                ReadinessInput::new(timeline, EffectReadiness::Ready),
                &ctx
            )
            .unwrap_err()
            .kind(),
        &PlaybackErrorKind::StaleReadiness
    );
}
