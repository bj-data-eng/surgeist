//! Independent Web Animations Level 1 time-space and fixed-offset examples.

use surgeist_animation::time::{
    AnimationTime, OriginRelativeTime, PlaybackRate, TimeErrorKind, TimeInput, TimeOperation,
    TimelineTime,
};
use surgeist_animation::timeline::{
    AnimationSnapshot, AnimationTimeInput, DocumentClock, DocumentTimeline, TimelineId,
    TimelineMonotonicity, TimelineSnapshot, animation_time_to_origin_relative_time,
    animation_time_to_timeline_time, timeline_time_to_animation_time,
    timeline_time_to_origin_relative_time,
};

fn animation_time(value: f64) -> AnimationTime {
    AnimationTime::from_millis(value).unwrap()
}

fn timeline_time(value: f64) -> TimelineTime {
    TimelineTime::from_millis(value).unwrap()
}

fn origin_time(value: f64) -> OriginRelativeTime {
    OriginRelativeTime::from_millis(value).unwrap()
}

fn document(origin: f64, now: f64) -> DocumentTimeline {
    let mut timeline = DocumentTimeline::new(TimelineId::new(7), origin_time(origin));
    timeline
        .update(DocumentClock::Active(origin_time(now)))
        .unwrap();
    timeline
}

fn animation(
    timeline: Option<TimelineSnapshot>,
    rate: f64,
    start: Option<f64>,
) -> AnimationSnapshot {
    AnimationSnapshot::new(
        PlaybackRate::new(rate).unwrap(),
        start.map(timeline_time),
        timeline,
    )
}

#[test]
fn signed_coordinates_preserve_microsecond_input_without_clamping() {
    for value in [-10.5, -0.001, 0.0, 0.001, 2.5, f64::MAX] {
        assert_eq!(animation_time(value).as_millis(), value);
        assert_eq!(timeline_time(value).as_millis(), value);
        assert_eq!(origin_time(value).as_millis(), value);
    }
    assert_ne!(animation_time(0.001), animation_time(0.0));
    assert_ne!(timeline_time(0.001), timeline_time(0.0));
    assert_ne!(origin_time(0.001), origin_time(0.0));
    assert_eq!(PlaybackRate::new(-2.0).unwrap().value(), -2.0);
    assert_eq!(PlaybackRate::new(0.0).unwrap().value(), 0.0);
}

#[test]
fn coordinate_and_rate_construction_rejects_nonfinite_values() {
    for value in [f64::NAN, f64::INFINITY, f64::NEG_INFINITY] {
        assert_eq!(
            AnimationTime::from_millis(value).unwrap_err().kind(),
            TimeErrorKind::NonFinite {
                input: TimeInput::Animation
            }
        );
        assert_eq!(
            TimelineTime::from_millis(value).unwrap_err().kind(),
            TimeErrorKind::NonFinite {
                input: TimeInput::Timeline
            }
        );
        assert_eq!(
            OriginRelativeTime::from_millis(value).unwrap_err().kind(),
            TimeErrorKind::NonFinite {
                input: TimeInput::OriginRelative
            }
        );
        assert_eq!(
            PlaybackRate::new(value).unwrap_err().kind(),
            TimeErrorKind::NonFinite {
                input: TimeInput::PlaybackRate
            }
        );
    }
}

#[test]
fn document_time_is_unresolved_until_active_and_uses_a_fixed_offset() {
    let id = TimelineId::new(9);
    let mut timeline = DocumentTimeline::new(id, origin_time(40.0));
    for state in [DocumentClock::Unestablished, DocumentClock::Inactive] {
        let snapshot = timeline.update(state).unwrap();
        assert_eq!(snapshot.current_time(), None);
        assert_eq!(
            timeline_time_to_origin_relative_time(Some(timeline_time(30.0)), snapshot).unwrap(),
            None
        );
    }
    let snapshot = timeline
        .update(DocumentClock::Active(origin_time(100.0)))
        .unwrap();
    assert_eq!(snapshot.id(), id);
    assert_eq!(snapshot.current_time(), Some(timeline_time(60.0)));
    assert_eq!(snapshot.monotonicity(), TimelineMonotonicity::Monotonic);
    assert_eq!(
        timeline_time_to_origin_relative_time(snapshot.current_time(), snapshot).unwrap(),
        Some(origin_time(100.0))
    );
    assert_eq!(timeline.snapshot(), snapshot);
    // Reading a frame never advances its supplied timestamp.
    assert_eq!(timeline.snapshot(), snapshot);
}

#[test]
fn restored_document_uses_now_without_deducting_inactive_time() {
    let mut default = document(0.0, 100.0);
    let mut offset = document(40.0, 100.0);
    let first = offset.snapshot();
    for timeline in [&mut default, &mut offset] {
        assert_eq!(
            timeline
                .update(DocumentClock::Inactive)
                .unwrap()
                .current_time(),
            None
        );
        timeline
            .update(DocumentClock::Active(origin_time(250.0)))
            .unwrap();
    }
    assert_eq!(
        default.snapshot().current_time(),
        Some(timeline_time(250.0))
    );
    assert_eq!(offset.snapshot().current_time(), Some(timeline_time(210.0)));
    for timeline in [&default, &offset] {
        let snapshot = timeline.snapshot();
        assert_eq!(
            timeline_time_to_origin_relative_time(snapshot.current_time(), snapshot).unwrap(),
            Some(origin_time(250.0))
        );
    }
    assert_eq!(first.current_time(), Some(timeline_time(60.0)));
    assert_eq!(offset.snapshot().id(), first.id());
}

#[test]
fn negative_document_current_time_is_not_clamped_to_zero() {
    let snapshot = document(40.0, 10.0).snapshot();
    assert_eq!(snapshot.current_time(), Some(timeline_time(-30.0)));
    assert_eq!(
        timeline_time_to_origin_relative_time(Some(timeline_time(-50.0)), snapshot).unwrap(),
        Some(origin_time(-10.0))
    );
}

#[test]
fn decreasing_document_reports_are_rejected_atomically_even_after_inactivity() {
    let mut timeline = document(40.0, 100.0);
    assert_eq!(
        timeline
            .update(DocumentClock::Active(origin_time(100.0)))
            .unwrap(),
        timeline.snapshot()
    );
    let retained = timeline.snapshot();
    let error = timeline
        .update(DocumentClock::Active(origin_time(99.0)))
        .unwrap_err();
    assert_eq!(
        error.kind(),
        TimeErrorKind::DecreasingDocumentTime {
            previous: timeline_time(60.0),
            attempted: timeline_time(59.0)
        }
    );
    assert_eq!(timeline.snapshot(), retained);
    let inactive = timeline.update(DocumentClock::Inactive).unwrap();
    assert!(
        timeline
            .update(DocumentClock::Active(origin_time(99.0)))
            .is_err()
    );
    assert_eq!(timeline.snapshot(), inactive);
    assert_eq!(
        timeline
            .update(DocumentClock::Active(origin_time(101.0)))
            .unwrap()
            .current_time(),
        Some(timeline_time(61.0))
    );
}

#[test]
fn conversion_distinguishes_animation_timeline_and_origin_coordinates() {
    let timeline = document(40.0, 150.0).snapshot();
    let state = animation(Some(timeline), 2.0, Some(100.0));
    let input = AnimationTimeInput::Finite(animation_time(30.0));
    assert_eq!(
        animation_time_to_timeline_time(input, state).unwrap(),
        Some(timeline_time(115.0))
    );
    assert_eq!(
        animation_time_to_origin_relative_time(input, state).unwrap(),
        Some(origin_time(155.0))
    );
    // Timeline-coordinate cancellation/removal input skips animation conversion.
    assert_eq!(
        timeline_time_to_origin_relative_time(Some(timeline_time(30.0)), timeline).unwrap(),
        Some(origin_time(70.0))
    );
}

#[test]
fn negative_conversion_inputs_and_rates_keep_their_signs() {
    let timeline = document(40.0, 100.0).snapshot();
    let state = animation(Some(timeline), -2.0, Some(-100.0));
    let input = AnimationTimeInput::Finite(animation_time(-30.0));
    assert_eq!(
        animation_time_to_timeline_time(input, state).unwrap(),
        Some(timeline_time(-85.0))
    );
    assert_eq!(
        animation_time_to_origin_relative_time(input, state).unwrap(),
        Some(origin_time(-45.0))
    );
}

#[test]
fn unresolved_conversion_outcomes_do_not_become_input_or_zero() {
    let active = document(40.0, 100.0).snapshot();
    let inactive = DocumentTimeline::new(TimelineId::new(7), origin_time(40.0)).snapshot();
    let no_conversion = TimelineSnapshot::new(
        TimelineId::new(8),
        Some(timeline_time(60.0)),
        TimelineMonotonicity::NonMonotonic,
        None,
    );
    let input = AnimationTimeInput::Finite(animation_time(30.0));
    for state in [
        animation(Some(active), 0.0, Some(100.0)),
        animation(Some(active), 2.0, None),
    ] {
        assert_eq!(animation_time_to_timeline_time(input, state).unwrap(), None);
        assert_eq!(
            animation_time_to_origin_relative_time(input, state).unwrap(),
            None
        );
    }
    for state in [
        animation(None, 2.0, Some(100.0)),
        animation(Some(inactive), 2.0, Some(100.0)),
        animation(Some(no_conversion), 2.0, Some(100.0)),
    ] {
        assert_eq!(
            animation_time_to_timeline_time(input, state).unwrap(),
            Some(timeline_time(115.0))
        );
        assert_eq!(
            animation_time_to_origin_relative_time(input, state).unwrap(),
            None
        );
    }
    for input in [AnimationTimeInput::Unresolved, AnimationTimeInput::Infinite] {
        let state = animation(Some(active), 2.0, Some(100.0));
        assert_eq!(animation_time_to_timeline_time(input, state).unwrap(), None);
        assert_eq!(
            animation_time_to_origin_relative_time(input, state).unwrap(),
            None
        );
    }
    assert_eq!(
        timeline_time_to_origin_relative_time(None, active).unwrap(),
        None
    );
    let zero = animation(Some(document(0.0, 0.0).snapshot()), 1.0, Some(0.0));
    assert_eq!(
        animation_time_to_origin_relative_time(
            AnimationTimeInput::Finite(animation_time(0.0)),
            zero
        )
        .unwrap(),
        Some(origin_time(0.0))
    );
}

#[test]
fn snapshots_keep_association_and_conversion_state_after_later_changes() {
    let mut timeline = document(40.0, 150.0);
    let first = animation(Some(timeline.snapshot()), 2.0, Some(100.0));
    let second = animation(Some(timeline.snapshot()), -1.0, Some(200.0));
    let input = AnimationTimeInput::Finite(animation_time(30.0));
    timeline.update(DocumentClock::Inactive).unwrap();
    let later = animation(Some(timeline.snapshot()), 5.0, Some(0.0));
    assert_eq!(
        animation_time_to_origin_relative_time(input, first).unwrap(),
        Some(origin_time(155.0))
    );
    assert_eq!(
        animation_time_to_origin_relative_time(input, second).unwrap(),
        Some(origin_time(210.0))
    );
    assert_eq!(
        animation_time_to_origin_relative_time(input, later).unwrap(),
        None
    );
    assert_eq!(first.timeline().unwrap().id(), TimelineId::new(7));
}

#[test]
fn time_flow_uses_supplied_coordinates_without_frame_history() {
    let state = animation(Some(document(40.0, 150.0).snapshot()), -2.0, Some(100.0));
    assert_eq!(
        timeline_time_to_animation_time(Some(timeline_time(110.0)), state).unwrap(),
        Some(animation_time(-20.0))
    );
    assert_eq!(timeline_time_to_animation_time(None, state).unwrap(), None);
    assert_eq!(
        timeline_time_to_animation_time(Some(timeline_time(110.0)), animation(None, 1.0, None))
            .unwrap(),
        None
    );
    assert_eq!(
        timeline_time_to_animation_time(
            Some(timeline_time(110.0)),
            animation(None, 0.0, Some(100.0))
        )
        .unwrap(),
        Some(animation_time(0.0))
    );
    // Seeking samples pure input coordinates, even when the timeline is monotonic.
    for (input, expected) in [
        (110.0, -20.0),
        (90.0, 20.0),
        (110.0, -20.0),
        (1000.0, -1800.0),
    ] {
        assert_eq!(
            timeline_time_to_animation_time(Some(timeline_time(input)), state).unwrap(),
            Some(animation_time(expected))
        );
    }
}

#[test]
fn finite_nonmonotonic_timelines_accept_decreasing_external_reports() {
    let first = TimelineSnapshot::new(
        TimelineId::new(8),
        Some(timeline_time(100.0)),
        TimelineMonotonicity::NonMonotonic,
        None,
    );
    let second = TimelineSnapshot::new(
        first.id(),
        Some(timeline_time(-20.0)),
        TimelineMonotonicity::NonMonotonic,
        None,
    );
    assert_eq!(
        timeline_time_to_animation_time(
            first.current_time(),
            animation(Some(first), 1.0, Some(0.0))
        )
        .unwrap(),
        Some(animation_time(100.0))
    );
    assert_eq!(
        timeline_time_to_animation_time(
            second.current_time(),
            animation(Some(second), 1.0, Some(0.0))
        )
        .unwrap(),
        Some(animation_time(-20.0))
    );
    assert_eq!(
        timeline_time_to_origin_relative_time(second.current_time(), second).unwrap(),
        None
    );
}

#[test]
fn unrepresentable_arithmetic_is_a_typed_error_and_preserves_document_state() {
    let mut timeline = DocumentTimeline::new(TimelineId::new(1), origin_time(-f64::MAX));
    let retained = timeline.snapshot();
    assert_eq!(
        timeline
            .update(DocumentClock::Active(origin_time(f64::MAX)))
            .unwrap_err()
            .kind(),
        TimeErrorKind::Unrepresentable {
            operation: TimeOperation::DocumentCurrentTime
        }
    );
    assert_eq!(timeline.snapshot(), retained);
    let snapshot = document(f64::MAX, f64::MAX).snapshot();
    assert_eq!(
        timeline_time_to_origin_relative_time(Some(timeline_time(f64::MAX)), snapshot)
            .unwrap_err()
            .kind(),
        TimeErrorKind::Unrepresentable {
            operation: TimeOperation::TimelineToOrigin
        }
    );
    let state = animation(Some(snapshot), 0.5, Some(0.0));
    assert_eq!(
        animation_time_to_timeline_time(
            AnimationTimeInput::Finite(animation_time(f64::MAX)),
            state
        )
        .unwrap_err()
        .kind(),
        TimeErrorKind::Unrepresentable {
            operation: TimeOperation::AnimationToTimeline
        }
    );
    let state = animation(Some(document(0.0, f64::MAX).snapshot()), 2.0, Some(0.0));
    assert_eq!(
        timeline_time_to_animation_time(Some(timeline_time(f64::MAX)), state)
            .unwrap_err()
            .kind(),
        TimeErrorKind::Unrepresentable {
            operation: TimeOperation::TimelineToAnimation
        }
    );
}

#[test]
fn representable_final_results_survive_overflowing_intermediates() {
    let state = animation(None, 0.5, Some(-f64::MAX));
    assert_eq!(
        animation_time_to_timeline_time(
            AnimationTimeInput::Finite(animation_time(f64::MAX)),
            state
        )
        .unwrap(),
        Some(timeline_time(f64::MAX))
    );
    let reverse = animation(None, 0.25, Some(-f64::MAX));
    assert_eq!(
        timeline_time_to_animation_time(Some(timeline_time(f64::MAX)), reverse).unwrap(),
        Some(animation_time(f64::MAX * 0.5))
    );
    let negative = animation(None, -0.5, Some(f64::MAX));
    assert_eq!(
        animation_time_to_timeline_time(
            AnimationTimeInput::Finite(animation_time(f64::MAX)),
            negative
        )
        .unwrap(),
        Some(timeline_time(-f64::MAX))
    );
}
