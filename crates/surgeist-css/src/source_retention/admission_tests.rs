use super::*;
use std::cell::Cell;

struct Reads<'a>(&'a Cell<usize>);
impl CssRetainedSources for Reads<'_> {
    fn visit_retained_sources(
        &self,
        _: &mut CssSourceRetention<'_>,
    ) -> Result<(), CssSourceRetentionError> {
        self.0.set(self.0.get() + 1);
        Ok(())
    }
}

#[test]
fn first_step_refusal_prevents_visitor_entry_and_keeps_typed_caller_error() {
    #[derive(Debug, Eq, PartialEq)]
    enum CallerError {
        OriginalAncestor,
    }
    let reads = Cell::new(0);
    let mut failure = None;
    let mut admit = |action| {
        if action == CssSourceRetentionAction::Step {
            failure = Some(CallerError::OriginalAncestor);
            false
        } else {
            true
        }
    };
    let mut collector = CssSourceRetention::with_admission(4096, 10, 10, &mut admit).unwrap();
    assert_eq!(
        collector.visit(&Reads(&reads)),
        Err(CssSourceRetentionError::caller_admission())
    );
    assert_eq!(reads.get(), 0);
    assert_eq!(collector.steps(), 0);
    drop(collector);
    assert_eq!(failure, Some(CallerError::OriginalAncestor));
}

#[test]
fn initial_metadata_refusal_does_not_form_a_usable_collector() {
    let mut observed = None;
    let mut admit = |action| {
        observed = Some(action);
        false
    };
    assert!(matches!(
        CssSourceRetention::with_admission(0, 0, 0, &mut admit),
        Err(error) if error.is_caller_admission()
    ));
    assert_eq!(
        observed,
        Some(CssSourceRetentionAction::Scratch {
            live_bytes: std::mem::size_of::<CssSourceRetention<'_>>()
        })
    );
}

struct Swallows;
impl CssRetainedSources for Swallows {
    fn visit_retained_sources(
        &self,
        sources: &mut CssSourceRetention<'_>,
    ) -> Result<(), CssSourceRetentionError> {
        let _ = sources.charge_step();
        Ok(())
    }
}

#[test]
fn swallowed_refusal_stays_terminal_without_later_callbacks_or_visitor_work() {
    let calls = Cell::new(0);
    let reads = Cell::new(0);
    let mut admit = |action| {
        calls.set(calls.get() + 1);
        action != CssSourceRetentionAction::Step || calls.get() < 3
    };
    let mut collector = CssSourceRetention::with_admission(4096, 10, 10, &mut admit).unwrap();
    assert_eq!(
        collector.visit(&Swallows),
        Err(CssSourceRetentionError::caller_admission())
    );
    assert_eq!(calls.get(), 3);
    assert_eq!(
        collector.visit(&Reads(&reads)),
        Err(CssSourceRetentionError::caller_admission())
    );
    assert_eq!(
        collector.charge_step(),
        Err(CssSourceRetentionError::caller_admission())
    );
    assert_eq!(
        collector.at_depth(0, |_| {
            reads.set(1);
            Ok(())
        }),
        Err(CssSourceRetentionError::caller_admission())
    );
    assert_eq!(
        collector.complete_payload_bytes_admitted(),
        Err(CssRetainedPayloadError::admission(
            CssSourceRetentionError::caller_admission()
        ))
    );
    assert_eq!(calls.get(), 3);
    assert_eq!(reads.get(), 0);
    // An explicitly metered read-only observer has a distinct admission budget.
    let observations = Cell::new(0);
    assert_eq!(
        collector.complete_payload_bytes_with(|| {
            observations.set(observations.get() + 1);
            Ok::<_, ()>(())
        }),
        Ok(0)
    );
    assert_eq!(observations.get(), 1);
}

#[test]
fn comparison_refusal_stops_before_duplicate_or_distinct_source_decision() {
    let first = CssSourceSnapshot::new("a");
    let distinct = CssSourceSnapshot::new("a");
    for candidate in [&first, &distinct] {
        let steps = Cell::new(0);
        let mut admit = |action| {
            if action == CssSourceRetentionAction::Step {
                steps.set(steps.get() + 1);
            }
            steps.get() != 3
        };
        let mut collector = CssSourceRetention::with_admission(4096, 10, 10, &mut admit).unwrap();
        collector.source(&first).unwrap();
        let usage = collector.usage();
        let prefix_payload = collector
            .complete_payload_bytes_with(|| Ok::<_, ()>(()))
            .unwrap();
        assert_eq!(
            collector.source(candidate),
            Err(CssSourceRetentionError::caller_admission())
        );
        assert_eq!(collector.sources.len(), 1);
        assert_eq!(collector.steps(), 2);
        assert_eq!(collector.usage(), usage);
        assert_eq!(
            collector.complete_payload_bytes_with(|| Ok::<_, ()>(())),
            Ok(prefix_payload)
        );
        assert_eq!(steps.get(), 3);
    }
}

#[test]
fn admitted_collector_keeps_identity_visit_depth_and_payload_policies() {
    let first = CssSourceSnapshot::new("a");
    let distinct = CssSourceSnapshot::new("a");
    let mut admit = |_| true;
    let mut collector = CssSourceRetention::with_admission(4096, 5, 1, &mut admit).unwrap();
    collector.source(&first).unwrap(); // One source action.
    collector.source(&first).unwrap(); // Source action plus identity comparison.
    collector.source(&distinct).unwrap(); // Same two actions, distinct identity.
    assert_eq!(collector.steps(), 5);
    assert_eq!(collector.sources.len(), 2);
    assert_eq!(
        collector.usage().retained_bytes(),
        first.storage_usage().retained_bytes() + distinct.storage_usage().retained_bytes()
    );
    let payload = collector
        .complete_payload_bytes_with(|| Ok::<_, ()>(()))
        .unwrap();
    assert_eq!(collector.complete_payload_bytes_admitted(), Ok(payload));
    assert_eq!(collector.steps(), 5);
    assert_eq!(
        collector.source(&first),
        Err(CssSourceRetentionError::limit(
            crate::CssSourceRetentionResource::Visits
        ))
    );
    assert_eq!(collector.steps(), 5);
    let reads = Cell::new(0);
    let mut depth_admit = |_| true;
    let mut depth = CssSourceRetention::with_admission(4096, 10, 1, &mut depth_admit).unwrap();
    assert_eq!(
        depth.at_depth(1, |collector| collector.visit(&Reads(&reads))),
        Err(CssSourceRetentionError::limit(
            crate::CssSourceRetentionResource::Depth
        ))
    );
    depth.visit(&Reads(&reads)).unwrap();
    assert_eq!(reads.get(), 1);
    assert_eq!(depth.steps(), 2);
}

#[test]
fn stored_predicate_prepays_payload_observations_without_spending_traversal_steps() {
    let source = CssSourceSnapshot::new("a");
    let enabled = Cell::new(true);
    let calls = Cell::new(0);
    let mut admit = |action| {
        if action == CssSourceRetentionAction::Step {
            calls.set(calls.get() + 1);
        }
        enabled.get()
    };
    let mut collector = CssSourceRetention::with_admission(4096, 10, 10, &mut admit).unwrap();
    collector.source(&source).unwrap();
    let steps = collector.steps();
    let before = calls.get();
    collector.complete_payload_bytes_admitted().unwrap();
    assert_eq!(calls.get() - before, 3); // entry, source, payload.
    assert_eq!(collector.steps(), steps);
    enabled.set(false);
    assert_eq!(
        collector.complete_payload_bytes_admitted(),
        Err(CssRetainedPayloadError::admission(
            CssSourceRetentionError::caller_admission()
        ))
    );
    let refused = calls.get();
    assert_eq!(
        collector.complete_payload_bytes_admitted(),
        Err(CssRetainedPayloadError::admission(
            CssSourceRetentionError::caller_admission()
        ))
    );
    assert_eq!(calls.get(), refused);
}

#[test]
fn recursive_css_uses_the_same_tariff_and_stops_at_each_refused_action() {
    let report = parse_sheet(".a { --token: calc(1 + var(--other)); }");
    let mut standalone = CssSourceRetention::new(4096, 1000, 20);
    standalone.visit(report.syntax()).unwrap();
    let expected_steps = standalone.steps();
    let expected_usage = standalone.usage();
    let expected_payload = standalone
        .complete_payload_bytes_with(|| Ok::<_, ()>(()))
        .unwrap();
    for allowance in 0..=expected_steps {
        let steps = Cell::new(0);
        let mut admit = |action| {
            if action == CssSourceRetentionAction::Step {
                let reached = steps.get();
                steps.set(reached + 1);
                reached < allowance
            } else {
                true
            }
        };
        let mut collector = CssSourceRetention::with_admission(4096, 1000, 20, &mut admit).unwrap();
        let result = collector.visit(report.syntax());
        if allowance == expected_steps {
            assert_eq!(result, Ok(()));
            assert_eq!(collector.usage(), expected_usage);
            assert_eq!(
                collector.complete_payload_bytes_with(|| Ok::<_, ()>(())),
                Ok(expected_payload)
            );
            assert_eq!(steps.get(), expected_steps);
        } else {
            assert_eq!(result, Err(CssSourceRetentionError::caller_admission()));
            assert_eq!(steps.get(), allowance + 1);
        }
        assert_eq!(collector.steps(), allowance);
    }
}

#[test]
fn replacement_scratch_reports_the_live_old_new_and_fixed_metadata_union() {
    let first = CssSourceSnapshot::new("a");
    let second = CssSourceSnapshot::new("b");
    let mut reports = Vec::new();
    let mut admit = |action| {
        if let CssSourceRetentionAction::Scratch { live_bytes } = action {
            reports.push(live_bytes);
        }
        true
    };
    let mut collector = CssSourceRetention::with_admission(4096, 1000, 10, &mut admit).unwrap();
    collector.source(&first).unwrap();
    let first_capacity = collector.sources.capacity();
    for index in 1..first_capacity {
        collector
            .source(&CssSourceSnapshot::new(&index.to_string()))
            .unwrap();
    }
    collector.source(&second).unwrap();
    let second_capacity = collector.sources.capacity();
    drop(collector);
    let fixed = std::mem::size_of::<CssSourceRetention<'_>>();
    let temporary_header = std::mem::size_of::<Vec<CssSourceSnapshot>>();
    let slot = std::mem::size_of::<CssSourceSnapshot>();
    assert_eq!(reports.len(), 5);
    assert_eq!(reports[0], fixed);
    assert_eq!(reports[1], fixed + temporary_header + slot);
    assert_eq!(reports[2], fixed + temporary_header + first_capacity * slot);
    assert_eq!(
        reports[3],
        fixed + temporary_header + (first_capacity + first_capacity * 2) * slot
    );
    assert_eq!(
        reports[4],
        fixed + temporary_header + (first_capacity + second_capacity) * slot
    );
}
