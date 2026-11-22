#![forbid(unsafe_code)]

use surgeist_animation::css_animation::{
    CssAnimationEligibility, CssAnimationErrorKind, CssAnimationInstance, CssAnimationList,
    CssAnimationName, CssAnimationSlot, CssAnimationStopReason, ResolvedCssAnimation,
};
use surgeist_animation::keyframe::KeyframeTiming;
use surgeist_animation::playback::{
    AnimationId, AnimationRecords, PlaybackContext, PlaybackCoordinates,
};
use surgeist_animation::time::{
    AnimationDelay, AnimationDuration, AnimationTime, PlaybackRate, TimelineTime,
};
use surgeist_animation::timeline::{
    AnimationSnapshot, TimelineId, TimelineMonotonicity, TimelineSnapshot,
};
use surgeist_animation::timing::{
    AnimationDirection, AnimationPlayState, FillMode, IterationCount, TimingPhase,
};

#[derive(Debug, Clone, PartialEq)]
struct Rule {
    identity: u32,
    revision: u32,
    values: Vec<f64>,
}
fn rule(identity: u32, revision: u32) -> Rule {
    Rule {
        identity,
        revision,
        values: vec![0.0, 100.0],
    }
}
fn timing(delay: f64, duration: f64, fill: FillMode) -> KeyframeTiming {
    KeyframeTiming::try_new(
        AnimationDelay::from_secs(delay).unwrap(),
        AnimationDuration::from_secs(duration).unwrap(),
        IterationCount::finite(1.0).unwrap(),
        AnimationDirection::Normal,
        fill,
    )
    .unwrap()
}
fn animation(name: &str, rule: Rule, delay: f64, duration: f64) -> ResolvedCssAnimation<Rule> {
    ResolvedCssAnimation::new(
        CssAnimationName::new(name),
        rule,
        timing(delay, duration, FillMode::Both),
        AnimationPlayState::Running,
    )
    .unwrap()
}
fn slot(name: &str, rule: Rule, delay: f64, duration: f64) -> CssAnimationSlot<Rule> {
    CssAnimationSlot::Matched(animation(name, rule, delay, duration))
}
fn list(slots: Vec<CssAnimationSlot<Rule>>) -> CssAnimationList<Rule> {
    CssAnimationList::new(slots, CssAnimationEligibility::Eligible).unwrap()
}
fn id(owner: &mut AnimationRecords) -> AnimationId {
    let created = owner
        .create(None, None, &PlaybackContext::default())
        .unwrap();
    created.animation().clone()
}
fn coordinates(current: Option<f64>, held: bool) -> PlaybackCoordinates {
    let timeline = TimelineSnapshot::new(
        TimelineId::new(1),
        current.map(|v| TimelineTime::from_millis(v).unwrap()),
        TimelineMonotonicity::Monotonic,
        None,
    );
    PlaybackCoordinates::new(
        AnimationSnapshot::new(
            PlaybackRate::new(1.0).unwrap(),
            if held {
                None
            } else {
                Some(TimelineTime::from_millis(0.0).unwrap())
            },
            Some(timeline),
        ),
        if held {
            current.map(|v| AnimationTime::from_millis(v).unwrap())
        } else {
            None
        },
    )
}
fn old(owner: &mut AnimationRecords, name: &str, current: f64) -> CssAnimationInstance<Rule> {
    CssAnimationInstance::new(
        id(owner),
        animation(name, rule(1, 1), 0.0, 10.0),
        coordinates(Some(current), false),
    )
    .unwrap()
}
fn progress(sample: surgeist_animation::timing::EffectTimingSample) -> Option<f64> {
    sample.directed_progress().map(|p| p.value())
}

#[test]
fn unresolved_rule_waits_and_later_valid_empty_rule_starts_from_zero() {
    let missing = list(vec![CssAnimationSlot::Unmatched(CssAnimationName::new(
        "a",
    ))]);
    let plan = missing.reconcile(&[]).unwrap();
    assert!(plan.updates().is_empty());
    let empty = Rule {
        identity: 1,
        revision: 2,
        values: vec![],
    };
    let resolved = list(vec![slot("a", empty.clone(), 1.0, 2.0)]);
    let plan = resolved.reconcile(&[]).unwrap();
    assert_eq!(plan.updates().len(), 1);
    let start = &plan.updates()[0];
    assert!(start.previous().is_none());
    assert_eq!(start.new_index(), 0);
    assert_eq!(start.animation().rule(), &empty);
    assert_eq!(start.after().local_time().unwrap().as_millis(), 0.0);
    assert_eq!(start.after().phase(), TimingPhase::Before);
    assert_eq!(start.play_state_change(), Some(AnimationPlayState::Running));
}

#[test]
fn edited_and_replaced_matching_rules_keep_identity_and_coordinates() {
    let mut owner = AnimationRecords::new();
    let previous = vec![old(&mut owner, "a", 2500.0)];
    for next_rule in [
        rule(1, 2),
        rule(77, 1),
        Rule {
            identity: 5,
            revision: 1,
            values: vec![],
        },
    ] {
        let next = list(vec![slot("a", next_rule.clone(), 0.0, 10.0)]);
        let plan = next.reconcile(&previous).unwrap();
        assert!(plan.stops().is_empty());
        let update = &plan.updates()[0];
        assert_eq!(update.previous().unwrap().id(), previous[0].id());
        assert_eq!(
            update.previous().unwrap().coordinates(),
            previous[0].coordinates()
        );
        assert_eq!(update.animation().rule(), &next_rule);
        assert_eq!(update.before(), Some(previous[0].sample()));
        assert_eq!(progress(update.after()), Some(0.25));
        assert_eq!(update.play_state_change(), None);
    }
}

#[test]
fn repeated_names_match_last_to_first_and_consume_each_old_instance_once() {
    let mut owner = AnimationRecords::new();
    let previous = vec![
        old(&mut owner, "a", 1000.0),
        old(&mut owner, "b", 2000.0),
        old(&mut owner, "a", 3000.0),
    ];
    let next = list(vec![
        slot("a", rule(1, 2), 0.0, 20.0),
        slot("a", rule(1, 2), 0.0, 30.0),
        slot("b", rule(2, 1), 0.0, 40.0),
        slot("a", rule(1, 2), 0.0, 50.0),
    ]);
    let plan = next.reconcile(&previous).unwrap();
    assert_eq!(
        plan.updates()
            .iter()
            .map(|u| u.new_index())
            .collect::<Vec<_>>(),
        vec![0, 1, 2, 3]
    );
    assert!(plan.updates()[0].previous().is_none());
    for (new, old) in [(1, 0), (2, 1), (3, 2)] {
        assert_eq!(
            plan.updates()[new].previous().unwrap().id(),
            previous[old].id()
        );
        assert_eq!(plan.updates()[new].old_index(), Some(old));
    }
    assert_eq!(progress(plan.updates()[1].after()), Some(1.0 / 30.0));
    assert_eq!(progress(plan.updates()[3].after()), Some(3.0 / 50.0));
    assert!(plan.stops().is_empty());
}

#[test]
fn a_to_a_a_keeps_the_existing_animation_in_the_second_position() {
    let mut owner = AnimationRecords::new();
    let previous = vec![old(&mut owner, "a", 4000.0)];
    let next = list(vec![
        slot("a", rule(1, 1), 0.0, 10.0),
        slot("a", rule(1, 1), 0.0, 10.0),
    ]);
    let plan = next.reconcile(&previous).unwrap();
    assert!(plan.updates()[0].previous().is_none());
    assert_eq!(plan.updates()[1].previous().unwrap().id(), previous[0].id());
    assert_eq!(progress(plan.updates()[0].after()), Some(0.0));
    assert_eq!(progress(plan.updates()[1].after()), Some(0.4));
}

#[test]
fn loss_of_last_rule_stops_and_reappearance_requires_a_fresh_instance() {
    let mut owner = AnimationRecords::new();
    let previous = vec![old(&mut owner, "a", 4000.0)];
    let missing = list(vec![CssAnimationSlot::Unmatched(CssAnimationName::new(
        "a",
    ))]);
    let stopped = missing.reconcile(&previous).unwrap();
    assert!(stopped.updates().is_empty());
    assert_eq!(stopped.stops()[0].instance().id(), previous[0].id());
    assert_eq!(
        stopped.stops()[0].reason(),
        CssAnimationStopReason::MissingRule
    );
    assert_eq!(
        stopped.stops()[0].sample().active_time_millis(),
        Some(4000.0)
    );
    let restored = list(vec![slot("a", rule(9, 1), 0.0, 10.0)]);
    let restarted = restored.reconcile(&[]).unwrap();
    assert!(restarted.updates()[0].previous().is_none());
    assert_eq!(progress(restarted.updates()[0].after()), Some(0.0));
    // Runtime allocates a fresh opaque identity when publishing the start.
    let fresh = id(&mut owner);
    assert_ne!(&fresh, previous[0].id());
}

#[test]
fn removed_names_stop_in_old_order_and_none_preserves_new_slot_positions() {
    let mut owner = AnimationRecords::new();
    let previous = vec![
        old(&mut owner, "a", 1000.0),
        old(&mut owner, "b", 2000.0),
        old(&mut owner, "a", 3000.0),
    ];
    let next = list(vec![
        CssAnimationSlot::None,
        slot("a", rule(1, 1), 0.0, 10.0),
    ]);
    let plan = next.reconcile(&previous).unwrap();
    assert_eq!(plan.updates()[0].new_index(), 1);
    assert_eq!(plan.updates()[0].old_index(), Some(2));
    assert_eq!(
        plan.stops()
            .iter()
            .map(|s| s.old_index())
            .collect::<Vec<_>>(),
        vec![0, 1]
    );
    assert!(
        plan.stops()
            .iter()
            .all(|s| s.reason() == CssAnimationStopReason::NameRemoved)
    );
}

#[test]
fn display_and_selected_media_policy_stop_all_and_restoration_starts_again() {
    let mut owner = AnimationRecords::new();
    let previous = vec![old(&mut owner, "a", 2000.0)];
    for eligibility in [
        CssAnimationEligibility::ElementDisplayNone,
        CssAnimationEligibility::AncestorDisplayNone,
        CssAnimationEligibility::IgnoredMedia,
    ] {
        let suppressed =
            CssAnimationList::new(vec![slot("a", rule(1, 1), 0.0, 10.0)], eligibility).unwrap();
        let plan = suppressed.reconcile(&previous).unwrap();
        assert!(plan.updates().is_empty());
        assert_eq!(
            plan.stops()[0].reason(),
            CssAnimationStopReason::Ineligible(eligibility)
        );
    }
    let restored = list(vec![slot("a", rule(1, 1), 0.0, 10.0)]);
    assert!(
        restored.reconcile(&[]).unwrap().updates()[0]
            .previous()
            .is_none()
    );
}

#[test]
fn timing_edits_apply_at_the_same_local_time_and_can_finish_or_return_to_delay() {
    let mut owner = AnimationRecords::new();
    let previous = vec![old(&mut owner, "a", 5000.0)];
    for (delay, duration, phase, expected) in [
        (-1.0, 10.0, TimingPhase::Active, 0.6),
        (0.0, 2.0, TimingPhase::After, 1.0),
        (8.0, 10.0, TimingPhase::Before, 0.0),
    ] {
        let next = list(vec![slot("a", rule(1, 2), delay, duration)]);
        let plan = next.reconcile(&previous).unwrap();
        let update = &plan.updates()[0];
        assert_eq!(progress(update.before().unwrap()), Some(0.5));
        assert_eq!(update.after().local_time().unwrap().as_millis(), 5000.0);
        assert_eq!(update.after().phase(), phase);
        assert_eq!(progress(update.after()), Some(expected));
        assert_eq!(update.previous().unwrap().id(), previous[0].id());
    }
}

#[test]
fn finished_instances_are_retained_and_dynamic_fill_updates_the_contribution() {
    let mut owner = AnimationRecords::new();
    let previous = vec![old(&mut owner, "a", 15000.0)];
    for (fill, in_effect) in [
        (FillMode::None, false),
        (FillMode::Backwards, false),
        (FillMode::Forwards, true),
        (FillMode::Both, true),
    ] {
        let next_animation = ResolvedCssAnimation::new(
            CssAnimationName::new("a"),
            rule(1, 1),
            timing(0.0, 10.0, fill),
            AnimationPlayState::Running,
        )
        .unwrap();
        let next = list(vec![CssAnimationSlot::Matched(next_animation)]);
        let plan = next.reconcile(&previous).unwrap();
        assert!(plan.stops().is_empty());
        assert!(plan.updates()[0].previous().is_some());
        assert_eq!(plan.updates()[0].after().phase(), TimingPhase::After);
        assert_eq!(plan.updates()[0].after().is_in_effect(), in_effect);
    }
    let extended = list(vec![slot("a", rule(1, 1), 0.0, 20.0)]);
    assert_eq!(
        progress(extended.reconcile(&previous).unwrap().updates()[0].after()),
        Some(0.75)
    );
}

#[test]
fn paused_and_unresolved_coordinates_survive_updates_without_a_new_playback_machine() {
    let mut owner = AnimationRecords::new();
    for current in [Some(2500.0), None] {
        let mut before = animation("a", rule(1, 1), 0.0, 10.0);
        before = ResolvedCssAnimation::new(
            before.name().clone(),
            before.rule().clone(),
            before.timing(),
            AnimationPlayState::Paused,
        )
        .unwrap();
        let previous = vec![
            CssAnimationInstance::new(id(&mut owner), before, coordinates(current, true)).unwrap(),
        ];
        let next = list(vec![slot("a", rule(2, 1), 0.0, 20.0)]);
        let plan = next.reconcile(&previous).unwrap();
        let update = &plan.updates()[0];
        assert_eq!(
            update.previous().unwrap().coordinates(),
            previous[0].coordinates()
        );
        assert_eq!(
            update.after().local_time(),
            current.map(|v| AnimationTime::from_millis(v).unwrap())
        );
        assert_eq!(progress(update.after()), current.map(|_| 0.125));
        assert_eq!(
            update.play_state_change(),
            Some(AnimationPlayState::Running)
        );
    }
}

#[test]
fn initially_paused_negative_delay_and_zero_duration_have_real_timing_samples() {
    let paused = ResolvedCssAnimation::new(
        CssAnimationName::new("a"),
        rule(1, 1),
        timing(-2.0, 10.0, FillMode::Both),
        AnimationPlayState::Paused,
    )
    .unwrap();
    let next = list(vec![
        CssAnimationSlot::Matched(paused),
        slot("b", rule(2, 1), 0.0, 0.0),
    ]);
    let plan = next.reconcile(&[]).unwrap();
    assert_eq!(progress(plan.updates()[0].after()), Some(0.2));
    assert_eq!(
        plan.updates()[0].play_state_change(),
        Some(AnimationPlayState::Paused)
    );
    assert_eq!(plan.updates()[1].after().phase(), TimingPhase::After);
    assert_eq!(progress(plan.updates()[1].after()), Some(1.0));
}

#[test]
fn names_preserve_string_ident_equivalence_empty_space_and_case() {
    let mut owner = AnimationRecords::new();
    let previous = vec![
        old(&mut owner, "", 1000.0),
        old(&mut owner, " ", 2000.0),
        old(&mut owner, "A", 3000.0),
    ];
    let next = list(vec![
        slot("a", rule(1, 1), 0.0, 10.0),
        slot(" ", rule(2, 1), 0.0, 10.0),
        slot("", rule(3, 1), 0.0, 10.0),
    ]);
    let plan = next.reconcile(&previous).unwrap();
    assert!(plan.updates()[0].previous().is_none());
    assert_eq!(plan.updates()[1].old_index(), Some(1));
    assert_eq!(plan.updates()[2].old_index(), Some(0));
    assert_eq!(plan.stops()[0].old_index(), 2);
    assert_eq!(
        CssAnimationName::new(String::from("ident")),
        CssAnimationName::new("ident")
    );
    assert_ne!(CssAnimationName::new("none"), CssAnimationName::new("None"));
}

#[test]
fn none_keyword_is_distinct_from_a_matched_string_name_none() {
    let next = list(vec![
        CssAnimationSlot::None,
        slot("none", rule(1, 1), 0.0, 1.0),
    ]);
    let plan = next.reconcile(&[]).unwrap();
    assert_eq!(plan.updates().len(), 1);
    assert_eq!(plan.updates()[0].new_index(), 1);
    assert_eq!(plan.updates()[0].animation().name().as_str(), "none");
}

#[test]
fn duplicate_instance_ids_and_conflicting_match_presence_are_rejected_atomically() {
    let mut owner = AnimationRecords::new();
    let instance = old(&mut owner, "a", 1000.0);
    let previous = vec![instance.clone(), instance];
    let next = list(vec![slot("a", rule(1, 1), 0.0, 10.0)]);
    let error = next.reconcile(&previous).unwrap_err();
    assert!(matches!(
        error.kind(),
        CssAnimationErrorKind::DuplicateAnimation { first: 0 }
    ));
    assert_eq!(error.old_index(), Some(1));
    let error = CssAnimationList::new(
        vec![
            slot("a", rule(1, 1), 0.0, 10.0),
            CssAnimationSlot::Unmatched(CssAnimationName::new("a")),
        ],
        CssAnimationEligibility::Eligible,
    )
    .unwrap_err();
    assert!(matches!(
        error.kind(),
        CssAnimationErrorKind::InconsistentRuleMatch { first: 0 }
    ));
    assert_eq!(error.new_index(), Some(1));
}

#[test]
fn timing_direction_iterations_and_play_state_updates_reuse_the_same_coordinate() {
    let mut owner = AnimationRecords::new();
    let previous = vec![old(&mut owner, "a", 6250.0)];
    let changed = KeyframeTiming::try_new(
        AnimationDelay::from_secs(0.0).unwrap(),
        AnimationDuration::from_secs(5.0).unwrap(),
        IterationCount::finite(3.0).unwrap(),
        AnimationDirection::AlternateReverse,
        FillMode::Both,
    )
    .unwrap();
    let paused = ResolvedCssAnimation::new(
        CssAnimationName::new("a"),
        rule(3, 1),
        changed,
        AnimationPlayState::Paused,
    )
    .unwrap();
    let next = list(vec![CssAnimationSlot::Matched(paused)]);
    let plan = next.reconcile(&previous).unwrap();
    let update = &plan.updates()[0];
    assert_eq!(
        update.after().current_iteration().unwrap().finite_index(),
        Some(1.0)
    );
    assert_eq!(progress(update.after()), Some(0.25));
    assert_eq!(update.play_state_change(), Some(AnimationPlayState::Paused));
    assert_eq!(update.after().local_time().unwrap().as_millis(), 6250.0);
}

#[test]
fn unresolved_running_and_signed_coordinates_are_preserved_in_retained_samples() {
    let mut owner = AnimationRecords::new();
    for current in [None, Some(-250.0)] {
        let previous = vec![
            CssAnimationInstance::new(
                id(&mut owner),
                animation("a", rule(1, 1), 0.0, 10.0),
                coordinates(current, false),
            )
            .unwrap(),
        ];
        let next = list(vec![slot("a", rule(2, 1), 0.0, 10.0)]);
        let plan = next.reconcile(&previous).unwrap();
        let update = &plan.updates()[0];
        assert!(update.previous().is_some());
        assert_eq!(
            update.after().local_time(),
            current.map(|v| AnimationTime::from_millis(v).unwrap())
        );
        assert_eq!(
            update.after().phase(),
            if current.is_some() {
                TimingPhase::Before
            } else {
                TimingPhase::Idle
            }
        );
        assert_eq!(update.play_state_change(), None);
    }
}

#[test]
fn a_later_sampling_failure_returns_no_partial_plan_and_preserves_snapshots() {
    let mut owner = AnimationRecords::new();
    let previous = vec![old(&mut owner, "a", f64::MAX), old(&mut owner, "b", 500.0)];
    let before = previous.clone();
    let tiny_unbounded = KeyframeTiming::try_new(
        AnimationDelay::from_secs(0.0).unwrap(),
        AnimationDuration::from_secs(f64::from_bits(1)).unwrap(),
        IterationCount::infinite(),
        AnimationDirection::Normal,
        FillMode::Both,
    )
    .unwrap();
    let next = list(vec![
        CssAnimationSlot::Matched(
            ResolvedCssAnimation::new(
                CssAnimationName::new("a"),
                rule(1, 1),
                tiny_unbounded,
                AnimationPlayState::Running,
            )
            .unwrap(),
        ),
        slot("b", rule(2, 1), 0.0, 1.0),
    ]);
    // The backwards scan can prepare b before a rejects its unrepresentable progress.
    let error = next.reconcile(&previous).unwrap_err();
    assert!(matches!(error.kind(), CssAnimationErrorKind::Timing(_)));
    assert_eq!(error.new_index(), Some(0));
    assert_eq!(previous, before);
    let valid = list(vec![
        slot("a", rule(1, 2), 0.0, 10.0),
        slot("b", rule(2, 2), 0.0, 1.0),
    ]);
    assert_eq!(valid.reconcile(&previous).unwrap().updates().len(), 2);
}

#[test]
fn finite_timing_conversion_and_coordinate_overflow_return_typed_errors() {
    let huge = ResolvedCssAnimation::new(
        CssAnimationName::new("a"),
        rule(1, 1),
        timing(0.0, f64::MAX, FillMode::Both),
        AnimationPlayState::Running,
    )
    .unwrap_err();
    assert!(matches!(
        huge.kind(),
        CssAnimationErrorKind::UnrepresentableMilliseconds
    ));
    let mut owner = AnimationRecords::new();
    let timeline = TimelineSnapshot::new(
        TimelineId::new(1),
        Some(TimelineTime::from_millis(f64::MAX).unwrap()),
        TimelineMonotonicity::Monotonic,
        None,
    );
    let bad = PlaybackCoordinates::new(
        AnimationSnapshot::new(
            PlaybackRate::new(2.0).unwrap(),
            Some(TimelineTime::from_millis(0.0).unwrap()),
            Some(timeline),
        ),
        None,
    );
    let error =
        CssAnimationInstance::new(id(&mut owner), animation("a", rule(1, 1), 0.0, 10.0), bad)
            .unwrap_err();
    assert!(matches!(error.kind(), CssAnimationErrorKind::Playback(_)));
    // Held time bypasses overflowing timeline arithmetic, as its owner specifies.
    let held = PlaybackCoordinates::new(
        bad.animation(),
        Some(AnimationTime::from_millis(2500.0).unwrap()),
    );
    assert_eq!(
        progress(
            CssAnimationInstance::new(id(&mut owner), animation("a", rule(1, 1), 0.0, 10.0), held)
                .unwrap()
                .sample()
        ),
        Some(0.25)
    );
}
