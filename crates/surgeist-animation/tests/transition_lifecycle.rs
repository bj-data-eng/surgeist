#![forbid(unsafe_code)]

use surgeist_animation::computed::{
    ComputedComponent, ComputedErrorKind, ComputedNumeric, ComputedValue,
};
use surgeist_animation::easing::{Easing, LinearControlPointInput, LinearEasing};
use surgeist_animation::interpolation::{
    InterpolableNumber, InterpolableValue, InterpolationOutcome, InterpolationPair,
    InterpolationProgress, PropertyKey,
};
use surgeist_animation::time::{AnimationDelay, AnimationDuration, OriginRelativeTime};
use surgeist_animation::transition::{
    TransitionChange, TransitionDecisionErrorKind, TransitionKey, TransitionOutcome,
    TransitionPropertyPolicy, TransitionSamplePhase, TransitionSnapshot, TransitionState,
    TransitionTiming, TransitionTrackId, Transitionability,
};

#[derive(Default)]
struct Policy {
    discrete_from: Option<f64>,
}

impl TransitionPropertyPolicy for Policy {
    fn transitionability(
        &self,
        property: &PropertyKey,
        from: &InterpolableValue,
        _: &InterpolableValue,
    ) -> Transitionability {
        if property.as_str() == "blocked" {
            return Transitionability::NotAnimatable;
        }
        if property.as_str() == "discrete" || self.discrete_from.is_some_and(|v| from == &number(v))
        {
            return Transitionability::Discrete;
        }
        Transitionability::Continuous
    }

    fn interpolate(
        &self,
        property: &PropertyKey,
        from: &InterpolableValue,
        to: &InterpolableValue,
        progress: InterpolationProgress,
    ) -> InterpolationOutcome {
        match InterpolationPair::new(property.clone(), from.clone(), to.clone()) {
            Ok(pair) => pair.sample(progress),
            Err(error) => InterpolationOutcome::Unsupported(error),
        }
    }
}

#[test]
fn creation_captures_identity_values_timing_and_reversal_defaults() {
    let state = start(1.0, 0.25, Easing::linear());
    let run = running(&state);
    assert_eq!(run.key(), &key());
    assert_eq!(run.id(), TransitionTrackId::new(1));
    assert_eq!(run.start_time(), time(0.25));
    assert_eq!(run.end_time(), time(1.25));
    assert_eq!(run.start_value(), &number(0.0));
    assert_eq!(run.end_value(), &number(100.0));
    assert_eq!(run.reversing_adjusted_start_value(), &number(0.0));
    assert_eq!(run.shortening_factor().value(), 1.0);
    assert_eq!(run.timing().duration().as_secs(), 1.0);
}

#[test]
fn each_creation_predicate_prevents_a_transition_independently() {
    let mut candidates = Vec::new();
    let mut ineligible = change(0.0, 100.0, 1.0, 0.0);
    ineligible.eligible = false;
    candidates.push(ineligible);
    candidates.push(change(0.0, 0.0, 1.0, 0.0));
    let mut unmatched = change(0.0, 100.0, 1.0, 0.0);
    unmatched.matching = None;
    candidates.push(unmatched);
    candidates.push(change(0.0, 100.0, 1.0, -1.0));
    candidates.push(change(0.0, 100.0, 1.0, -2.0));
    candidates.push(change(0.0, 100.0, 0.0, 0.0));
    for property in ["blocked", "discrete"] {
        let mut input = change(0.0, 100.0, 1.0, 0.0);
        input.key = TransitionKey::new(1, PropertyKey::new(property).unwrap());
        candidates.push(input);
    }
    for input in candidates {
        let update = TransitionSnapshot::default()
            .update(Some(time(0.0)), &[input], &Policy::default())
            .unwrap()
            .unwrap();
        assert!(update.state().records().is_empty());
        assert!(update.outcomes().is_empty());
    }
    let update = TransitionSnapshot::default()
        .update(
            Some(time(0.0)),
            &[change(0.0, 100.0, 1.0, 0.0)],
            &Policy {
                discrete_from: Some(0.0),
            },
        )
        .unwrap()
        .unwrap();
    assert!(update.state().records().is_empty());
}

#[test]
fn delay_and_exact_start_return_raw_value_but_retain_old_function_output() {
    let state = start(1.0, 1.0, shifted());
    let run = running(&state);
    for t in [0.0, 0.5, 1.0] {
        let sample = run.sample(Some(time(t)), &Policy::default()).unwrap();
        assert_eq!(sample.phase(), TransitionSamplePhase::Delay);
        assert_eq!(sample.value(), Some(&number(0.0)));
        assert_eq!(sample.easing_output().unwrap().value(), 0.25);
        assert_eq!(sample.key(), &key());
        assert_eq!(sample.id(), run.id());
        assert_eq!(sample.time(), Some(time(t)));
    }
    let sample = run.sample(Some(time(1.5)), &Policy::default()).unwrap();
    assert_eq!(sample.phase(), TransitionSamplePhase::Active);
    assert_eq!(sample.value(), Some(&number(62.5)));
    assert_eq!(sample.easing_output().unwrap().value(), 0.625);
    assert_eq!(
        run.sample(Some(time(2.0)), &Policy::default())
            .unwrap()
            .phase(),
        TransitionSamplePhase::Finished
    );
}

#[test]
fn eligible_marker_target_uses_normal_computed_opacity_sampling() {
    #[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
    enum Target {
        Element,
        Marker,
    }

    let opacity = PropertyKey::new("opacity").unwrap();
    let changes =
        [(Target::Element, 1), (Target::Marker, 2)].map(|(target, id)| TransitionChange {
            key: TransitionKey::new(target, opacity.clone()),
            before: number(0.25),
            after: number(0.75),
            // Style supplies target eligibility even though opacity is outside
            // the direct marker-box applicability list.
            eligible: true,
            matching: Some(timing(1.0, 0.0, Easing::linear())),
            new_id: TransitionTrackId::new(id),
        });
    let policy = Policy::default();
    let snapshot = TransitionSnapshot::default()
        .update(Some(time(0.0)), &changes, &policy)
        .unwrap()
        .unwrap();
    for change in &changes {
        let Some(TransitionState::Running(run)) = snapshot.state().get(&change.key) else {
            panic!("expected a running transition for each eligible target");
        };
        let sample = run.sample(Some(time(0.25)), &policy).unwrap();
        assert!(sample.key() == &change.key);
        assert_eq!(sample.value(), Some(&number(0.375)));
    }
}

#[test]
fn samples_are_stateless_for_supplied_times() {
    let state = start(1.0, 0.0, Easing::linear());
    let run = running(&state);
    assert_eq!(
        run.sample(Some(time(0.75)), &Policy::default())
            .unwrap()
            .value(),
        Some(&number(75.0))
    );
    assert_eq!(
        run.sample(Some(time(0.25)), &Policy::default())
            .unwrap()
            .value(),
        Some(&number(25.0))
    );
    assert_eq!(
        run.sample(Some(time(0.75)), &Policy::default())
            .unwrap()
            .value(),
        Some(&number(75.0))
    );
}

#[test]
fn all_completions_are_promoted_before_same_time_style_decisions() {
    let mut second = change(0.0, 100.0, 1.0, 0.0);
    second.key = TransitionKey::new(2, PropertyKey::new("opacity").unwrap());
    second.new_id = TransitionTrackId::new(2);
    let state = TransitionSnapshot::default()
        .update(
            Some(time(0.0)),
            &[change(0.0, 100.0, 1.0, 0.0), second],
            &Policy::default(),
        )
        .unwrap()
        .unwrap()
        .state()
        .clone();
    let update = state
        .update(
            Some(time(1.0)),
            &[change(50.0, 100.0, 1.0, 0.0)],
            &Policy::default(),
        )
        .unwrap()
        .unwrap();
    assert_eq!(update.completions().len(), 2);
    assert!(
        update
            .after_completion()
            .records()
            .iter()
            .all(|record| matches!(record, TransitionState::Completed(_)))
    );
    assert!(update.outcomes().is_empty());
    assert!(
        update
            .state()
            .records()
            .iter()
            .all(|record| matches!(record, TransitionState::Completed(_)))
    );
    assert!(
        update
            .state()
            .update(Some(time(2.0)), &[], &Policy::default())
            .unwrap()
            .unwrap()
            .completions()
            .is_empty()
    );
}

#[test]
fn a_target_change_at_completion_starts_an_ordinary_run_not_a_reversal() {
    let state = start(1.0, 0.0, Easing::linear());
    let update = state
        .update(
            Some(time(1.0)),
            &[change(100.0, 0.0, 2.0, 0.0)],
            &Policy::default(),
        )
        .unwrap()
        .unwrap();
    assert_eq!(update.completions().len(), 1);
    assert_eq!(running(update.state()).shortening_factor().value(), 1.0);
    assert_eq!(running(update.state()).start_value(), &number(100.0));
    assert_eq!(running(update.state()).end_time(), time(3.0));
    assert!(matches!(
        update.outcomes(),
        [
            TransitionOutcome::CompletedRemoved(_),
            TransitionOutcome::Started(_)
        ]
    ));
}

#[test]
fn completed_mismatches_are_removed_even_when_no_transition_can_start() {
    let state = start(1.0, 0.0, Easing::linear());
    let completed = state
        .update(Some(time(1.0)), &[], &Policy::default())
        .unwrap()
        .unwrap()
        .state()
        .clone();
    let update = completed
        .update(
            Some(time(2.0)),
            &[change(100.0, 50.0, 0.0, 0.0)],
            &Policy::default(),
        )
        .unwrap()
        .unwrap();
    assert!(update.state().records().is_empty());
    assert!(matches!(
        update.outcomes(),
        [TransitionOutcome::CompletedRemoved(_)]
    ));
}

#[test]
fn unchanged_target_preserves_original_timing_but_property_removal_cancels() {
    let state = start(2.0, 0.25, Easing::linear());
    let mut input = change(25.0, 100.0, 0.0, -1.0);
    input.matching = Some(timing(0.0, -1.0, Easing::step_end()));
    let update = state
        .update(Some(time(0.75)), &[input], &Policy::default())
        .unwrap()
        .unwrap();
    assert_eq!(update.state(), &state);
    assert!(update.outcomes().is_empty());
    let mut input = change(25.0, 100.0, 1.0, 0.0);
    input.matching = None;
    let update = state
        .update(Some(time(0.75)), &[input], &Policy::default())
        .unwrap()
        .unwrap();
    assert!(update.state().records().is_empty());
    assert!(matches!(
        update.outcomes(),
        [TransitionOutcome::Cancelled(_)]
    ));
}

#[test]
fn running_current_equality_pair_incompatibility_and_nonpositive_duration_cancel() {
    let state = start(1.0, 0.0, Easing::linear());
    for (input, policy) in [
        (change(90.0, 50.0, 1.0, 0.0), Policy::default()),
        (
            change(90.0, 200.0, 1.0, 0.0),
            Policy {
                discrete_from: Some(50.0),
            },
        ),
        (change(90.0, 200.0, 1.0, -1.0), Policy::default()),
    ] {
        let update = state
            .update(Some(time(0.5)), &[input], &policy)
            .unwrap()
            .unwrap();
        assert!(update.state().records().is_empty());
        assert!(matches!(
            update.outcomes(),
            [TransitionOutcome::Cancelled(_)]
        ));
    }
}

#[test]
fn restart_uses_the_old_current_value_and_the_new_timing() {
    let state = start(1.0, 0.0, Easing::linear());
    let update = state
        .update(
            Some(time(0.5)),
            &[change(90.0, 200.0, 2.0, 0.1)],
            &Policy::default(),
        )
        .unwrap()
        .unwrap();
    let run = running(update.state());
    assert_eq!(run.start_value(), &number(50.0));
    assert_eq!(run.end_value(), &number(200.0));
    assert_eq!(run.reversing_adjusted_start_value(), &number(50.0));
    assert_eq!(run.start_time(), time(0.6));
    assert_eq!(run.end_time(), time(2.6));
    assert_eq!(run.shortening_factor().value(), 1.0);
    assert!(matches!(
        update.outcomes(),
        [
            TransitionOutcome::Cancelled(_),
            TransitionOutcome::Started(_)
        ]
    ));
}

#[test]
fn repeated_reversals_use_independent_shortening_factors() {
    let initial = start(1.0, 0.0, Easing::linear());
    let first = initial
        .update(
            Some(time(0.25)),
            &[change(25.0, 0.0, 1.0, 0.0)],
            &Policy::default(),
        )
        .unwrap()
        .unwrap();
    let run = running(first.state());
    assert_eq!(run.start_value(), &number(25.0));
    assert_eq!(run.reversing_adjusted_start_value(), &number(100.0));
    assert_eq!(run.shortening_factor().value(), 0.25);
    assert_eq!(run.end_time(), time(0.5));
    let second = first
        .state()
        .update(
            Some(time(0.375)),
            &[change(12.5, 100.0, 1.0, 0.0)],
            &Policy::default(),
        )
        .unwrap()
        .unwrap();
    let run = running(second.state());
    assert_eq!(run.start_value(), &number(12.5));
    assert_eq!(run.reversing_adjusted_start_value(), &number(0.0));
    assert_eq!(run.shortening_factor().value(), 0.875);
    assert_eq!(run.end_time(), time(1.25));
}

#[test]
fn reversal_during_delay_uses_the_last_equal_input_of_linear_easing() {
    let initial = start(1.0, 0.0, Easing::linear());
    let policy = Policy::default();
    let mut reverse = change(50.0, 0.0, 1.0, 1.0);
    let easing = Easing::linear_function(
        LinearEasing::new(vec![
            LinearControlPointInput {
                input: 0.0,
                output: 0.25,
            },
            LinearControlPointInput {
                input: 0.0,
                output: 0.75,
            },
            LinearControlPointInput {
                input: 1.0,
                output: 1.0,
            },
        ])
        .unwrap(),
    );
    reverse.matching = Some(timing(1.0, 1.0, easing));
    let first = initial
        .update(Some(time(0.5)), &[reverse], &policy)
        .unwrap()
        .unwrap();
    let first_run = running(first.state());
    assert_eq!(first_run.shortening_factor().value(), 0.5);
    let sample = first_run.sample(Some(time(0.75)), &policy).unwrap();
    assert_eq!(sample.value(), Some(&number(50.0)));

    let mut reverse_again = change(50.0, 100.0, 1.0, 0.0);
    reverse_again.new_id = TransitionTrackId::new(3);
    let second = first
        .state()
        .update(Some(time(0.75)), &[reverse_again], &policy)
        .unwrap()
        .unwrap();
    let second_run = running(second.state());
    // The last duplicate zero input has output .75. With old factor .5 the
    // new shortening factor is .75*.5 + (1-.5) = .875.
    assert_eq!(
        (
            sample.easing_output().unwrap().value(),
            second_run.shortening_factor().value()
        ),
        (0.75, 0.875)
    );
    assert_eq!(second_run.end_time(), time(1.625));
}

#[test]
fn step_start_uses_the_bottom_of_its_initial_step_only_during_delay() {
    let state = start(1.0, 1.0, Easing::step_start());
    for (elapsed, output) in [(0.5, 0.0), (1.0, 1.0)] {
        let sample = running(&state)
            .sample(Some(time(elapsed)), &Policy::default())
            .unwrap();
        assert_eq!(sample.value(), Some(&number(0.0)));
        assert_eq!(sample.easing_output().unwrap().value(), output);
    }
}

#[test]
fn reversal_shortens_only_negative_delays_and_preserves_used_timing() {
    let initial = start(1.0, 0.0, Easing::linear());
    for (delay, start, end) in [(-0.4, 0.3, 1.3), (0.4, 0.9, 1.9)] {
        let update = initial
            .update(
                Some(time(0.5)),
                &[change(50.0, 0.0, 2.0, delay)],
                &Policy::default(),
            )
            .unwrap()
            .unwrap();
        let run = running(update.state());
        assert_eq!(run.start_time(), time(start));
        assert_eq!(run.end_time(), time(end));
        assert_eq!(run.duration().as_secs(), 1.0);
        assert_eq!(run.timing().duration().as_secs(), 2.0);
        assert_eq!(run.timing().delay().as_secs(), delay);
        assert_eq!(
            run.sample(Some(time(0.5)), &Policy::default())
                .unwrap()
                .value(),
            Some(&number(if delay < 0.0 { 40.0 } else { 50.0 }))
        );
    }
}

#[test]
fn reversal_uses_absolute_old_output_then_clamps_without_clamping_values() {
    for (output, expected_value, expected_factor) in [(-0.5, -50.0, 0.5), (1.5, 150.0, 1.0)] {
        let easing = Easing::linear_function(
            LinearEasing::new(vec![
                LinearControlPointInput { input: 0.0, output },
                LinearControlPointInput { input: 1.0, output },
            ])
            .unwrap(),
        );
        let state = start(1.0, 0.0, easing);
        let update = state
            .update(
                Some(time(0.5)),
                &[change(0.0, 0.0, 1.0, 0.0)],
                &Policy::default(),
            )
            .unwrap()
            .unwrap();
        let run = running(update.state());
        assert_eq!(run.start_value(), &number(expected_value));
        assert_eq!(run.shortening_factor().value(), expected_factor);
    }
}

#[test]
fn zero_duration_positive_delay_completes_at_its_start_without_division() {
    let state = start(0.0, 1.0, shifted());
    assert_eq!(
        running(&state)
            .sample(Some(time(0.5)), &Policy::default())
            .unwrap()
            .value(),
        Some(&number(0.0))
    );
    assert_eq!(
        running(&state)
            .sample(Some(time(1.0)), &Policy::default())
            .unwrap()
            .phase(),
        TransitionSamplePhase::Finished
    );
    let update = state
        .update(Some(time(1.0)), &[], &Policy::default())
        .unwrap()
        .unwrap();
    assert_eq!(update.completions().len(), 1);
    assert!(matches!(
        update.state().get(&key()),
        Some(TransitionState::Completed(_))
    ));
}

#[test]
fn negative_delay_and_signed_absolute_coordinates_are_preserved() {
    let state = TransitionSnapshot::default()
        .update(
            Some(time(-2.0)),
            &[change(0.0, 100.0, 2.0, -0.5)],
            &Policy::default(),
        )
        .unwrap()
        .unwrap()
        .state()
        .clone();
    let run = running(&state);
    assert_eq!(run.start_time(), time(-2.5));
    assert_eq!(run.end_time(), time(-0.5));
    assert_eq!(
        run.sample(Some(time(-2.0)), &Policy::default())
            .unwrap()
            .value(),
        Some(&number(25.0))
    );
}

#[test]
fn document_removal_returns_cancellation_and_completed_removal_only_for_matches() {
    let state = start(1.0, 0.0, Easing::linear());
    let cancelled = state.remove_targets(&[1]);
    assert!(cancelled.state().records().is_empty());
    assert!(matches!(
        cancelled.outcomes(),
        [TransitionOutcome::Cancelled(_)]
    ));
    assert_eq!(state.remove_targets(&[2]).state(), &state);
    let completed = state
        .update(Some(time(1.0)), &[], &Policy::default())
        .unwrap()
        .unwrap();
    let removed = completed.state().remove_targets(&[1]);
    assert!(matches!(
        removed.outcomes(),
        [TransitionOutcome::CompletedRemoved(_)]
    ));
    assert!(removed.state().records().is_empty());
}

#[test]
fn snapshots_reject_duplicate_keys_but_allow_distinct_owners_and_longhands() {
    let state = start(1.0, 0.0, Easing::linear());
    let record = state.records()[0].clone();
    let error = TransitionSnapshot::new(vec![record.clone(), record]).unwrap_err();
    assert_eq!(error.kind(), TransitionDecisionErrorKind::DuplicateKey);
    let completed = state
        .update(Some(time(1.0)), &[], &Policy::default())
        .unwrap()
        .unwrap();
    let error = TransitionSnapshot::new(vec![
        state.records()[0].clone(),
        completed.state().records()[0].clone(),
    ])
    .unwrap_err();
    assert_eq!(error.kind(), TransitionDecisionErrorKind::DuplicateKey);
    let input = change(0.0, 100.0, 1.0, 0.0);
    let error = state
        .update(Some(time(0.5)), &[input.clone(), input], &Policy::default())
        .unwrap_err();
    assert_eq!(error.kind(), TransitionDecisionErrorKind::DuplicateChange);
    let mut other_owner = change(0.0, 100.0, 1.0, 0.0);
    other_owner.key = TransitionKey::new(2, PropertyKey::new("opacity").unwrap());
    let mut other_property = change(0.0, 100.0, 1.0, 0.0);
    other_property.key = TransitionKey::new(1, PropertyKey::new("width").unwrap());
    let update = state
        .update(
            Some(time(0.5)),
            &[other_owner, other_property],
            &Policy::default(),
        )
        .unwrap()
        .unwrap();
    assert_eq!(update.state().records().len(), 3);
}

#[test]
fn unresolved_time_and_late_errors_publish_no_partial_update() {
    let state = start(1.0, 0.0, Easing::linear());
    assert!(
        state
            .update(None, &[change(50.0, 0.0, 1.0, 0.0)], &Policy::default())
            .unwrap()
            .is_none()
    );
    assert_eq!(
        running(&state)
            .sample(None, &Policy::default())
            .unwrap()
            .phase(),
        TransitionSamplePhase::Unresolved
    );
    let mut overflow = change(0.0, 100.0, f64::MAX, 0.0);
    overflow.key = TransitionKey::new(2, PropertyKey::new("width").unwrap());
    let error = state
        .update(
            Some(time(1.0)),
            &[change(100.0, 0.0, 1.0, 0.0), overflow],
            &Policy::default(),
        )
        .unwrap_err();
    assert_eq!(
        error.kind(),
        TransitionDecisionErrorKind::UnrepresentableTime
    );
    assert!(matches!(
        state.get(&key()),
        Some(TransitionState::Running(_))
    ));
    assert_eq!(running(&state).end_time(), time(1.0));
}

#[test]
fn arithmetic_sampling_failure_does_not_cancel_or_replace_the_old_run() {
    let mut input = change(0.0, f64::MAX, 1.0, 0.0);
    input.matching = Some(timing(1.0, 0.0, overshooting()));
    let state = TransitionSnapshot::default()
        .update(
            Some(time(0.0)),
            std::slice::from_ref(&input),
            &Policy::default(),
        )
        .unwrap()
        .unwrap()
        .state()
        .clone();
    assert_eq!(
        running(&state)
            .sample(Some(time(0.0)), &Policy::default())
            .unwrap()
            .value(),
        Some(&number(0.0))
    );
    assert_eq!(
        running(&state)
            .sample(Some(time(0.5)), &Policy::default())
            .unwrap_err()
            .kind(),
        TransitionDecisionErrorKind::Interpolation
    );
    input.after = number(-1.0);
    assert!(
        state
            .update(Some(time(0.5)), &[input], &Policy::default())
            .is_err()
    );
    assert_eq!(running(&state).end_value(), &number(f64::MAX));
}

#[test]
fn unavailable_property_interpolation_is_typed_and_does_not_hide_delay_values() {
    use surgeist_animation::interpolation::{InterpolationError, TransformValue};
    let from = InterpolableValue::transform(TransformValue::unsupported("translate"));
    let to = InterpolableValue::transform(TransformValue::unsupported("scale"));
    let mut input = change(0.0, 100.0, 1.0, 1.0);
    input.before = from.clone();
    input.after = to;
    let state = TransitionSnapshot::default()
        .update(Some(time(0.0)), &[input], &Policy::default())
        .unwrap()
        .unwrap()
        .state()
        .clone();
    assert_eq!(
        running(&state)
            .sample(Some(time(0.5)), &Policy::default())
            .unwrap()
            .value(),
        Some(&from)
    );
    let error = running(&state)
        .sample(Some(time(1.5)), &Policy::default())
        .unwrap_err();
    assert!(matches!(
        error.interpolation_error(),
        Some(InterpolationError::UnsupportedFamily { .. })
    ));
}

#[test]
fn computed_lifecycle_samples_preserve_arithmetic_diagnostics_and_recover() {
    let computed = |value| {
        InterpolableValue::computed(ComputedValue::single(ComputedComponent::Numeric(
            ComputedNumeric::number(value).unwrap(),
        )))
    };
    let mut input = change(0.0, 100.0, 1.0, 0.0);
    input.before = computed(0.0);
    input.after = computed(f64::MAX);
    input.matching = Some(timing(1.0, 0.0, overshooting()));
    let policy = Policy::default();
    let update = TransitionSnapshot::default()
        .update(Some(time(0.0)), &[input], &policy)
        .unwrap()
        .unwrap();
    let run = running(update.state());

    let error = run.sample(Some(time(0.5)), &policy).unwrap_err();
    assert_eq!(error.kind(), TransitionDecisionErrorKind::Interpolation);
    assert!(matches!(
        error.interpolation_error(),
        Some(surgeist_animation::interpolation::InterpolationError::Computed(diagnostic))
            if diagnostic.property().as_str() == "opacity"
                && diagnostic.error().kind() == ComputedErrorKind::NonFiniteResult
    ));
    let recovered = run.sample(Some(time(0.75)), &policy).unwrap();
    assert_eq!(recovered.value(), Some(&computed(f64::MAX)));
}

#[test]
fn completed_target_property_removal_does_not_create_another_transition() {
    let state = start(1.0, 0.0, Easing::linear());
    let mut input = change(50.0, 100.0, 1.0, 0.0);
    input.matching = None;
    let update = state
        .update(Some(time(1.0)), &[input], &Policy::default())
        .unwrap()
        .unwrap();
    assert_eq!(update.completions().len(), 1);
    assert!(matches!(
        update.outcomes(),
        [TransitionOutcome::CompletedRemoved(_)]
    ));
    assert!(update.state().records().is_empty());
}

#[test]
fn unchanged_target_does_not_require_an_unavailable_current_sample() {
    let input = change(f64::MAX, -f64::MAX, 1.0, 0.0);
    let state = TransitionSnapshot::default()
        .update(
            Some(time(0.0)),
            std::slice::from_ref(&input),
            &Policy::default(),
        )
        .unwrap()
        .unwrap()
        .state()
        .clone();
    let mut next = input;
    next.matching = Some(timing(0.0, -1.0, Easing::step_start()));
    let update = state
        .update(Some(time(0.5)), &[next], &Policy::default())
        .unwrap()
        .unwrap();
    assert_eq!(update.state(), &state);
    assert!(update.outcomes().is_empty());
}

#[test]
fn wide_signed_intervals_sample_without_intermediate_subtraction_overflow() {
    let mut input = change(0.0, 100.0, 1.0, 0.0);
    input.matching = Some(timing((f64::MAX / 1000.0) * 1.5, 0.0, Easing::linear()));
    let state = TransitionSnapshot::default()
        .update(
            Some(OriginRelativeTime::from_millis(-f64::MAX).unwrap()),
            &[input],
            &Policy::default(),
        )
        .unwrap()
        .unwrap()
        .state()
        .clone();
    let sample = running(&state)
        .sample(
            Some(OriginRelativeTime::from_millis(-f64::MAX / 4.0).unwrap()),
            &Policy::default(),
        )
        .unwrap();
    let Some(InterpolableValue::Number(value)) = sample.value() else {
        panic!("expected numeric sample")
    };
    assert!((value.value() - 50.0).abs() < 1e-12);
}

#[test]
fn unrepresentable_positive_timing_is_rejected_instead_of_collapsing_to_zero() {
    let empty = TransitionSnapshot::default();
    let input = change(0.0, 100.0, 1.0, 0.0);
    let error = empty
        .update(
            Some(OriginRelativeTime::from_millis(f64::MAX).unwrap()),
            &[input],
            &Policy::default(),
        )
        .unwrap_err();
    assert_eq!(
        error.kind(),
        TransitionDecisionErrorKind::UnrepresentableTime
    );
    assert!(empty.records().is_empty());

    let tiny = f64::MIN_POSITIVE;
    let easing = Easing::linear_function(
        LinearEasing::new(vec![
            LinearControlPointInput {
                input: 0.0,
                output: tiny,
            },
            LinearControlPointInput {
                input: 1.0,
                output: tiny,
            },
        ])
        .unwrap(),
    );
    let state = start(1.0, 0.0, easing);
    let error = state
        .update(
            Some(time(0.5)),
            &[change(0.0, 0.0, tiny, 0.0)],
            &Policy::default(),
        )
        .unwrap_err();
    assert_eq!(
        error.kind(),
        TransitionDecisionErrorKind::UnrepresentableTime
    );
    assert_eq!(running(&state).shortening_factor().value(), 1.0);
}

fn number(value: f64) -> InterpolableValue {
    InterpolableValue::number(InterpolableNumber::new(value).unwrap())
}
fn time(secs: f64) -> OriginRelativeTime {
    OriginRelativeTime::from_millis(secs * 1000.0).unwrap()
}
fn key() -> TransitionKey<u64> {
    TransitionKey::new(1, PropertyKey::new("opacity").unwrap())
}
fn timing(duration: f64, delay: f64, easing: Easing) -> TransitionTiming {
    TransitionTiming::new(
        AnimationDuration::from_secs(duration).unwrap(),
        AnimationDelay::from_secs(delay).unwrap(),
        easing,
    )
}
fn change(before: f64, after: f64, duration: f64, delay: f64) -> TransitionChange<u64> {
    TransitionChange {
        key: key(),
        before: number(before),
        after: number(after),
        eligible: true,
        matching: Some(timing(duration, delay, Easing::linear())),
        new_id: TransitionTrackId::new(2),
    }
}
fn start(duration: f64, delay: f64, easing: Easing) -> TransitionSnapshot<u64> {
    let mut input = change(0.0, 100.0, duration, delay);
    input.new_id = TransitionTrackId::new(1);
    input.matching = Some(timing(duration, delay, easing));
    TransitionSnapshot::default()
        .update(Some(time(0.0)), &[input], &Policy::default())
        .unwrap()
        .unwrap()
        .state()
        .clone()
}
fn running(
    state: &TransitionSnapshot<u64>,
) -> &surgeist_animation::transition::RunningTransition<u64> {
    let Some(TransitionState::Running(run)) = state.get(&key()) else {
        panic!("expected running transition")
    };
    run
}
fn shifted() -> Easing {
    Easing::linear_function(
        LinearEasing::new(vec![
            LinearControlPointInput {
                input: 0.0,
                output: 0.25,
            },
            LinearControlPointInput {
                input: 1.0,
                output: 1.0,
            },
        ])
        .unwrap(),
    )
}

fn overshooting() -> Easing {
    Easing::linear_function(
        LinearEasing::new(vec![
            LinearControlPointInput {
                input: 0.0,
                output: 0.0,
            },
            LinearControlPointInput {
                input: 0.5,
                output: 2.0,
            },
            LinearControlPointInput {
                input: 0.75,
                output: 1.0,
            },
            LinearControlPointInput {
                input: 1.0,
                output: 1.0,
            },
        ])
        .unwrap(),
    )
}
