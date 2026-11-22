use surgeist_animation::easing::Easing;
use surgeist_animation::time::{
    AnimationDelay, AnimationDuration, AnimationTime, ElapsedTime, PlaybackRate,
};
use surgeist_animation::timeline::TimelineMonotonicity;
use surgeist_animation::timing::{
    AnimationDirection, AnimationPlayState, AnimationReplaceState, EffectAnimationState,
    EffectPlayState, EffectTiming, EffectTimingErrorKind, EffectTimingField, EffectTimingInput,
    EffectTimingOptions, FillMode, IterationCount, PhaseProgress, TimingOperation,
    TimingParameters, TimingPhase,
};

fn state(
    rate: f64,
    play: EffectPlayState,
    timeline: Option<TimelineMonotonicity>,
    replace: AnimationReplaceState,
) -> EffectAnimationState {
    EffectAnimationState::new(PlaybackRate::new(rate).unwrap(), play, timeline, replace)
}

fn input(time: Option<f64>, rate: f64) -> EffectTimingInput {
    EffectTimingInput::Associated {
        local_time: time.map(|time| AnimationTime::from_millis(time).unwrap()),
        animation: state(
            rate,
            EffectPlayState::Running,
            Some(TimelineMonotonicity::Monotonic),
            AnimationReplaceState::Active,
        ),
    }
}

fn make_effect(options: EffectTimingOptions) -> EffectTiming {
    EffectTiming::try_new(options).unwrap()
}

fn progress(sample: surgeist_animation::timing::EffectTimingSample) -> Option<f64> {
    sample.simple_progress().map(PhaseProgress::value)
}

#[test]
fn defaults_have_zero_duration_and_linear_easing() {
    let effect = EffectTiming::default();
    assert_eq!(effect.active_duration_millis(), 0.0);
    assert_eq!(effect.end_time_millis(), 0.0);
    let sample = effect.sample(input(Some(0.0), 1.0)).unwrap();
    assert_eq!(sample.phase(), TimingPhase::After);
    assert_eq!(sample.local_time().unwrap().as_millis(), 0.0);
    assert_eq!(sample.active_time_millis(), None);
    let linear = make_effect(EffectTimingOptions {
        duration_millis: 100.0,
        ..EffectTimingOptions::default()
    });
    assert_eq!(
        linear
            .sample(input(Some(25.0), 1.0))
            .unwrap()
            .eased_progress()
            .unwrap()
            .value(),
        0.25
    );
}

#[test]
fn unresolved_local_time_and_absent_animation_are_idle() {
    let effect = make_effect(EffectTimingOptions {
        duration_millis: 100.0,
        fill: FillMode::Both,
        ..EffectTimingOptions::default()
    });
    for input in [input(None, 1.0), EffectTimingInput::Unassociated] {
        let sample = effect.sample(input).unwrap();
        assert_eq!(sample.phase(), TimingPhase::Idle);
        assert_eq!(sample.local_time(), None);
        assert_eq!(sample.active_time_millis(), None);
        assert_eq!(sample.overall_progress(), None);
        assert_eq!(sample.current_iteration(), None);
        assert_eq!(sample.eased_progress(), None);
        assert!(!sample.is_in_effect());
        assert!(!sample.is_relevant());
    }
}

#[test]
fn exact_phase_boundaries_follow_associated_playback_rate() {
    let effect = make_effect(EffectTimingOptions {
        start_delay_millis: 100.0,
        duration_millis: 200.0,
        ..EffectTimingOptions::default()
    });
    assert_eq!(effect.before_active_boundary_millis(), 100.0);
    assert_eq!(effect.active_after_boundary_millis(), 300.0);
    for (time, rate, expected) in [
        (-1.0, 1.0, TimingPhase::Before),
        (100.0, 1.0, TimingPhase::Active),
        (100.0, 0.0, TimingPhase::Active),
        (100.0, -1.0, TimingPhase::Before),
        (300.0, 1.0, TimingPhase::After),
        (300.0, 0.0, TimingPhase::After),
        (300.0, -1.0, TimingPhase::Active),
        (301.0, -1.0, TimingPhase::After),
    ] {
        assert_eq!(
            effect.sample(input(Some(time), rate)).unwrap().phase(),
            expected,
            "time={time}, rate={rate}"
        );
    }
}

#[test]
fn end_delay_truncates_active_interval_without_changing_active_duration() {
    let effect = make_effect(EffectTimingOptions {
        start_delay_millis: 100.0,
        end_delay_millis: -50.0,
        duration_millis: 200.0,
        fill: FillMode::Forwards,
        ..EffectTimingOptions::default()
    });
    assert_eq!(effect.active_duration_millis(), 200.0);
    assert_eq!(effect.end_time_millis(), 250.0);
    assert_eq!(effect.active_after_boundary_millis(), 250.0);
    let sample = effect.sample(input(Some(250.0), 1.0)).unwrap();
    assert_eq!(sample.phase(), TimingPhase::After);
    assert_eq!(sample.active_time_millis(), Some(150.0));
    assert_eq!(progress(sample), Some(0.75));
    assert_eq!(
        progress(effect.sample(input(Some(1000.0), 1.0)).unwrap()),
        Some(1.0)
    );
    let truncated = make_effect(EffectTimingOptions {
        start_delay_millis: 100.0,
        end_delay_millis: -400.0,
        duration_millis: 200.0,
        ..EffectTimingOptions::default()
    });
    assert_eq!(truncated.end_time_millis(), 0.0);
    assert_eq!(truncated.before_active_boundary_millis(), 0.0);
    assert_eq!(truncated.active_after_boundary_millis(), 0.0);
}

#[test]
fn fill_resolves_active_time_without_moving_phase_boundaries() {
    for fill in [
        FillMode::None,
        FillMode::Backwards,
        FillMode::Forwards,
        FillMode::Both,
    ] {
        let effect = make_effect(EffectTimingOptions {
            start_delay_millis: 50.0,
            duration_millis: 100.0,
            end_delay_millis: 40.0,
            fill,
            ..EffectTimingOptions::default()
        });
        assert_eq!(effect.end_time_millis(), 190.0);
        assert_eq!(effect.before_active_boundary_millis(), 50.0);
        assert_eq!(effect.active_after_boundary_millis(), 150.0);
        let before = effect.sample(input(Some(-20.0), 1.0)).unwrap();
        let active = effect.sample(input(Some(75.0), 1.0)).unwrap();
        let after = effect.sample(input(Some(160.0), 1.0)).unwrap();
        assert_eq!(
            before.active_time_millis(),
            matches!(fill, FillMode::Backwards | FillMode::Both).then_some(0.0)
        );
        assert_eq!(active.active_time_millis(), Some(25.0));
        assert_eq!(
            after.active_time_millis(),
            matches!(fill, FillMode::Forwards | FillMode::Both).then_some(100.0)
        );
    }
}

#[test]
fn negative_start_delay_and_negative_local_time_use_the_clipped_boundaries() {
    let effect = make_effect(EffectTimingOptions {
        start_delay_millis: -50.0,
        duration_millis: 100.0,
        fill: FillMode::Both,
        ..EffectTimingOptions::default()
    });
    assert_eq!(effect.before_active_boundary_millis(), 0.0);
    assert_eq!(effect.active_after_boundary_millis(), 50.0);
    let before = effect.sample(input(Some(-25.0), 1.0)).unwrap();
    assert_eq!(before.phase(), TimingPhase::Before);
    assert_eq!(before.active_time_millis(), Some(25.0));
    assert_eq!(progress(before), Some(0.25));
    assert_eq!(
        progress(effect.sample(input(Some(0.0), 1.0)).unwrap()),
        Some(0.5)
    );
}

#[test]
fn iteration_start_offsets_progress_without_extending_active_duration() {
    let effect = make_effect(EffectTimingOptions {
        duration_millis: 100.0,
        iterations: IterationCount::finite(2.5).unwrap(),
        iteration_start: 0.25,
        fill: FillMode::Both,
        ..EffectTimingOptions::default()
    });
    assert_eq!(effect.active_duration_millis(), 250.0);
    assert_eq!(effect.end_time_millis(), 250.0);
    for (time, overall, simple, current) in [
        (-1.0, 0.25, 0.25, 0.0),
        (75.0, 1.0, 0.0, 1.0),
        (250.0, 2.75, 0.75, 2.0),
    ] {
        let sample = effect.sample(input(Some(time), 1.0)).unwrap();
        assert_eq!(sample.overall_progress(), Some(overall));
        assert_eq!(progress(sample), Some(simple));
        assert_eq!(
            sample.current_iteration().unwrap().finite_index(),
            Some(current)
        );
    }
}

#[test]
fn exact_final_iteration_end_promotes_zero_remainder_only_at_active_duration() {
    let effect = make_effect(EffectTimingOptions {
        duration_millis: 100.0,
        iterations: IterationCount::finite(2.0).unwrap(),
        fill: FillMode::Both,
        ..EffectTimingOptions::default()
    });
    assert_eq!(
        progress(effect.sample(input(Some(100.0), 1.0)).unwrap()),
        Some(0.0)
    );
    for rate in [1.0, -1.0] {
        let sample = effect.sample(input(Some(200.0), rate)).unwrap();
        assert_eq!(progress(sample), Some(1.0));
        assert_eq!(
            sample.current_iteration().unwrap().finite_index(),
            Some(1.0)
        );
    }
    let empty = make_effect(EffectTimingOptions {
        duration_millis: 100.0,
        iterations: IterationCount::finite(0.0).unwrap(),
        fill: FillMode::Both,
        ..EffectTimingOptions::default()
    });
    assert_eq!(
        progress(empty.sample(input(Some(0.0), 1.0)).unwrap()),
        Some(0.0)
    );
}

#[test]
fn zero_factors_override_infinity_and_infinite_overall_uses_iteration_start() {
    let zero = make_effect(EffectTimingOptions {
        duration_millis: f64::INFINITY,
        iterations: IterationCount::finite(0.0).unwrap(),
        fill: FillMode::Both,
        ..EffectTimingOptions::default()
    });
    assert_eq!(zero.active_duration_millis(), 0.0);
    assert_eq!(
        progress(zero.sample(input(Some(0.0), 1.0)).unwrap()),
        Some(0.0)
    );
    let infinite = make_effect(EffectTimingOptions {
        duration_millis: 0.0,
        iterations: IterationCount::infinite(),
        iteration_start: 0.25,
        fill: FillMode::Both,
        direction: AnimationDirection::AlternateReverse,
        ..EffectTimingOptions::default()
    });
    assert_eq!(infinite.active_duration_millis(), 0.0);
    let sample = infinite.sample(input(Some(0.0), 1.0)).unwrap();
    assert_eq!(sample.overall_progress(), Some(f64::INFINITY));
    assert_eq!(progress(sample), Some(0.25));
    assert!(sample.current_iteration().unwrap().is_infinite());
    assert_eq!(sample.directed_progress().unwrap().value(), 0.25);
    let at_end = make_effect(EffectTimingOptions {
        duration_millis: 0.0,
        iterations: IterationCount::infinite(),
        fill: FillMode::Both,
        ..EffectTimingOptions::default()
    });
    assert_eq!(
        progress(at_end.sample(input(Some(0.0), 1.0)).unwrap()),
        Some(1.0)
    );
}

#[test]
fn directions_and_before_flag_follow_iteration_parity_and_effect_phase() {
    for (direction, first, second) in [
        (AnimationDirection::Normal, 0.25, 0.25),
        (AnimationDirection::Reverse, 0.75, 0.75),
        (AnimationDirection::Alternate, 0.25, 0.75),
        (AnimationDirection::AlternateReverse, 0.75, 0.25),
    ] {
        let effect = make_effect(EffectTimingOptions {
            duration_millis: 100.0,
            iterations: IterationCount::finite(2.0).unwrap(),
            direction,
            ..EffectTimingOptions::default()
        });
        assert_eq!(
            effect
                .sample(input(Some(25.0), 1.0))
                .unwrap()
                .directed_progress()
                .unwrap()
                .value(),
            first
        );
        assert_eq!(
            effect
                .sample(input(Some(125.0), 1.0))
                .unwrap()
                .directed_progress()
                .unwrap()
                .value(),
            second
        );
    }
    let before = make_effect(EffectTimingOptions {
        start_delay_millis: 100.0,
        duration_millis: 100.0,
        fill: FillMode::Both,
        easing: Easing::step_start(),
        ..EffectTimingOptions::default()
    })
    .sample(input(Some(0.0), 1.0))
    .unwrap();
    assert!(before.easing_before_flag());
    assert_eq!(before.eased_progress().unwrap().value(), 0.0);
    let after = make_effect(EffectTimingOptions {
        duration_millis: 100.0,
        fill: FillMode::Both,
        direction: AnimationDirection::Reverse,
        easing: Easing::step_end(),
        ..EffectTimingOptions::default()
    })
    .sample(input(Some(100.0), 1.0))
    .unwrap();
    assert!(after.easing_before_flag());
    assert_eq!(after.eased_progress().unwrap().value(), 0.0);
}

#[test]
fn effect_states_follow_supplied_animation_facts_and_replace_state() {
    let effect = make_effect(EffectTimingOptions {
        start_delay_millis: 100.0,
        duration_millis: 100.0,
        ..EffectTimingOptions::default()
    });
    let observed = |time: Option<f64>, rate, play, timeline, replace| {
        effect
            .sample(EffectTimingInput::Associated {
                local_time: time.map(|v| AnimationTime::from_millis(v).unwrap()),
                animation: state(rate, play, timeline, replace),
            })
            .unwrap()
    };
    let paused = observed(
        Some(150.0),
        1.0,
        EffectPlayState::Paused,
        Some(TimelineMonotonicity::Monotonic),
        AnimationReplaceState::Active,
    );
    assert!(paused.is_in_play());
    assert!(paused.is_current());
    assert!(paused.is_in_effect());
    assert!(paused.is_relevant());
    let finished = observed(
        Some(150.0),
        1.0,
        EffectPlayState::Finished,
        Some(TimelineMonotonicity::Monotonic),
        AnimationReplaceState::Active,
    );
    assert!(!finished.is_in_play());
    assert!(!finished.is_current());
    assert!(finished.is_in_effect());
    assert!(finished.is_relevant());
    for (time, rate) in [(50.0, 1.0), (250.0, -1.0)] {
        let sample = observed(
            Some(time),
            rate,
            EffectPlayState::Running,
            None,
            AnimationReplaceState::Active,
        );
        assert!(sample.is_current());
        assert!(!sample.is_in_effect());
        assert!(sample.is_relevant());
    }
    for (time, play, timeline, expected) in [
        (
            None,
            EffectPlayState::Running,
            Some(TimelineMonotonicity::NonMonotonic),
            true,
        ),
        (
            None,
            EffectPlayState::Idle,
            Some(TimelineMonotonicity::NonMonotonic),
            false,
        ),
        (
            None,
            EffectPlayState::Running,
            Some(TimelineMonotonicity::Monotonic),
            false,
        ),
        (None, EffectPlayState::Running, None, false),
    ] {
        assert_eq!(
            observed(time, 0.0, play, timeline, AnimationReplaceState::Active).is_current(),
            expected
        );
    }
    for replace in [
        AnimationReplaceState::Active,
        AnimationReplaceState::Persisted,
        AnimationReplaceState::Removed,
    ] {
        assert_eq!(
            observed(Some(150.0), 1.0, EffectPlayState::Running, None, replace).is_relevant(),
            replace != AnimationReplaceState::Removed
        );
    }
}

#[test]
fn filled_effect_can_remain_relevant_without_being_current() {
    let effect = make_effect(EffectTimingOptions {
        duration_millis: 100.0,
        fill: FillMode::Forwards,
        ..EffectTimingOptions::default()
    });
    let sample = effect
        .sample(EffectTimingInput::Associated {
            local_time: Some(AnimationTime::from_millis(200.0).unwrap()),
            animation: state(
                1.0,
                EffectPlayState::Finished,
                Some(TimelineMonotonicity::Monotonic),
                AnimationReplaceState::Active,
            ),
        })
        .unwrap();
    assert!(!sample.is_current());
    assert!(!sample.is_in_play());
    assert!(sample.is_in_effect());
    assert!(sample.is_relevant());
}

#[test]
fn invalid_options_and_finite_numeric_limits_are_typed() {
    for (field, options) in [
        (
            EffectTimingField::StartDelay,
            EffectTimingOptions {
                start_delay_millis: f64::INFINITY,
                ..EffectTimingOptions::default()
            },
        ),
        (
            EffectTimingField::EndDelay,
            EffectTimingOptions {
                end_delay_millis: f64::NAN,
                ..EffectTimingOptions::default()
            },
        ),
        (
            EffectTimingField::Duration,
            EffectTimingOptions {
                duration_millis: -1.0,
                ..EffectTimingOptions::default()
            },
        ),
        (
            EffectTimingField::Duration,
            EffectTimingOptions {
                duration_millis: f64::NEG_INFINITY,
                ..EffectTimingOptions::default()
            },
        ),
        (
            EffectTimingField::IterationStart,
            EffectTimingOptions {
                iteration_start: -0.1,
                ..EffectTimingOptions::default()
            },
        ),
        (
            EffectTimingField::IterationStart,
            EffectTimingOptions {
                iteration_start: f64::INFINITY,
                ..EffectTimingOptions::default()
            },
        ),
    ] {
        assert!(
            matches!(EffectTiming::try_new(options), Err(error) if matches!(error.kind(), EffectTimingErrorKind::InvalidOption { field: actual, .. } if actual == field))
        );
    }
    assert!(matches!(
        EffectTiming::try_new(EffectTimingOptions {
            duration_millis: f64::MAX,
            iterations: IterationCount::finite(2.0).unwrap(),
            ..EffectTimingOptions::default()
        }),
        Err(error) if matches!(error.kind(), EffectTimingErrorKind::Unrepresentable {
            operation: TimingOperation::ActiveDuration
        })
    ));
    assert!(matches!(
        EffectTiming::try_new(EffectTimingOptions {
            start_delay_millis: f64::MAX,
            duration_millis: f64::MAX,
            ..EffectTimingOptions::default()
        }),
        Err(error) if matches!(error.kind(), EffectTimingErrorKind::Unrepresentable {
            operation: TimingOperation::EndTime
        })
    ));
    let overflow = make_effect(EffectTimingOptions {
        duration_millis: f64::MIN_POSITIVE,
        iterations: IterationCount::infinite(),
        ..EffectTimingOptions::default()
    });
    assert!(matches!(
        overflow.sample(input(Some(f64::MAX), 1.0)),
        Err(error) if matches!(error.kind(), EffectTimingErrorKind::Unrepresentable {
            operation: TimingOperation::OverallProgress
        })
    ));
}

#[test]
fn model_and_css_elapsed_boundaries_share_phase_and_progress() {
    for direction in [
        AnimationDirection::Normal,
        AnimationDirection::Reverse,
        AnimationDirection::Alternate,
        AnimationDirection::AlternateReverse,
    ] {
        for fill in [FillMode::None, FillMode::Both] {
            let count = IterationCount::finite(2.5).unwrap();
            let model = make_effect(EffectTimingOptions {
                start_delay_millis: -50.0,
                duration_millis: 100.0,
                iterations: count,
                direction,
                fill,
                ..EffectTimingOptions::default()
            });
            let css = TimingParameters::try_new(
                AnimationDelay::from_secs(-0.05).unwrap(),
                AnimationDuration::from_secs(0.1).unwrap(),
                count,
                direction,
                fill,
                Easing::linear(),
            )
            .unwrap();
            for millis in [0.0, 25.0, 50.0, 125.0, 200.0, 250.0] {
                let model = model.sample(input(Some(millis), 1.0)).unwrap();
                let css = css.sample(
                    ElapsedTime::from_secs(millis / 1000.0).unwrap(),
                    AnimationPlayState::Running,
                );
                assert_eq!(model.phase(), css.phase());
                assert_eq!(model.current_iteration(), css.current_iteration());
                match (model.directed_progress(), css.directed_progress()) {
                    (Some(a), Some(b)) => assert!((a.value() - b.value()).abs() < 1.0e-12),
                    (None, None) => {}
                    values => panic!("different progress availability: {values:?}"),
                }
            }
        }
    }
}

#[test]
fn arbitrary_seeks_and_explicit_edits_do_not_accumulate_frame_history() {
    let effect = make_effect(EffectTimingOptions {
        duration_millis: 100.0,
        iterations: IterationCount::finite(2.0).unwrap(),
        iteration_start: 0.25,
        fill: FillMode::Both,
        ..EffectTimingOptions::default()
    });
    let expected = effect.sample(input(Some(125.0), 1.0)).unwrap();
    assert_eq!(progress(expected), Some(0.5));
    for time in [175.0, -50.0, 25.0, 1000.0, 125.0] {
        let _ = effect.sample(input(Some(time), 1.0)).unwrap();
        assert_eq!(effect.sample(input(Some(125.0), 1.0)).unwrap(), expected);
    }
    // A different iteration duration explicitly changes progress at the same time.
    let edited = make_effect(EffectTimingOptions {
        duration_millis: 200.0,
        ..effect.options().clone()
    });
    assert_eq!(
        progress(edited.sample(input(Some(125.0), 1.0)).unwrap()),
        Some(0.875)
    );
    assert_eq!(effect.sample(input(Some(125.0), 1.0)).unwrap(), expected);
}

#[test]
fn finite_end_time_recovers_from_a_cancelled_overflowing_sum() {
    let effect = make_effect(EffectTimingOptions {
        start_delay_millis: f64::MAX,
        duration_millis: f64::MAX,
        end_delay_millis: -f64::MAX,
        fill: FillMode::Both,
        ..EffectTimingOptions::default()
    });
    assert_eq!(effect.end_time_millis(), f64::MAX);
    let sample = effect.sample(input(Some(f64::MAX), 1.0)).unwrap();
    assert_eq!(sample.phase(), TimingPhase::After);
    assert_eq!(sample.active_time_millis(), Some(0.0));
    assert_eq!(progress(sample), Some(0.0));
}

#[test]
fn numerical_limit_preserves_existing_css_no_value_classification() {
    use surgeist_animation::interpolation::{InterpolableNumber, InterpolableValue, PropertyKey};
    use surgeist_animation::keyframe::{
        KeyframeAnimationId, KeyframeSegment, KeyframeTiming, KeyframeTrack,
    };
    use surgeist_animation::progress::UnitRatio;
    use surgeist_animation::sample::{
        CompletionStatus, NextFrameHint, SampleClassification, SampleValue,
    };
    let timing = KeyframeTiming::try_new(
        AnimationDelay::from_secs(0.0).unwrap(),
        AnimationDuration::from_secs(f64::MIN_POSITIVE).unwrap(),
        IterationCount::infinite(),
        AnimationDirection::Normal,
        FillMode::None,
    )
    .unwrap();
    let segment = KeyframeSegment::new(
        PropertyKey::new("opacity").unwrap(),
        UnitRatio::new(0.0).unwrap(),
        UnitRatio::new(1.0).unwrap(),
        InterpolableValue::number(InterpolableNumber::new(0.0).unwrap()),
        InterpolableValue::number(InterpolableNumber::new(1.0).unwrap()),
        Easing::linear(),
    )
    .unwrap();
    let track = KeyframeTrack::new(
        KeyframeAnimationId::new("finite-limit").unwrap(),
        timing,
        vec![segment],
    )
    .unwrap();
    let samples = track.sample(
        ElapsedTime::from_secs(f64::MAX).unwrap(),
        AnimationPlayState::Running,
    );
    assert_eq!(samples.len(), 1);
    assert_eq!(samples[0].classification(), SampleClassification::Active);
    assert_eq!(samples[0].completion(), CompletionStatus::Running);
    assert_eq!(samples[0].next_frame(), NextFrameHint::MayChange);
    assert!(matches!(samples[0].value(), SampleValue::None));
    let recovered = track.sample(
        ElapsedTime::from_secs(0.0).unwrap(),
        AnimationPlayState::Running,
    );
    assert_eq!(recovered[0].classification(), SampleClassification::Active);
    assert!(matches!(recovered[0].value(), SampleValue::Value(_)));
}

#[test]
fn model_forward_fill_preserves_the_final_iteration() {
    let timing = make_effect(EffectTimingOptions {
        duration_millis: 0.1,
        iterations: IterationCount::finite(3.0).unwrap(),
        fill: FillMode::Forwards,
        ..EffectTimingOptions::default()
    });
    let fill = timing.sample(input(Some(1.0), 1.0)).unwrap();
    assert_eq!(fill.phase(), TimingPhase::After);
    assert_eq!(progress(fill), Some(1.0));
    assert_eq!(
        fill.current_iteration()
            .and_then(|index| index.finite_index()),
        Some(2.0)
    );
}

#[test]
fn effect_timing_rejects_incoherent_play_state_and_local_time() {
    let timing = EffectTiming::default();
    for (play, local_time) in [
        (
            EffectPlayState::Idle,
            Some(AnimationTime::from_millis(10.0).unwrap()),
        ),
        (EffectPlayState::Finished, None),
    ] {
        let result = timing.sample(EffectTimingInput::Associated {
            local_time,
            animation: state(1.0, play, None, AnimationReplaceState::Active),
        });
        let diagnostic = result.unwrap_err();
        assert_eq!(
            diagnostic.kind(),
            EffectTimingErrorKind::InconsistentLocalTime {
                play_state: play,
                local_time,
            }
        );
    }
}
