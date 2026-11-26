use crate::source::{
    CssRetainedPayloadError, CssRetainedSources, CssSourceRetention, CssSourceRetentionAction,
    CssSourceRetentionError, CssSourceSnapshot,
};
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
fn source_step_refusal_stops_before_duplicate_or_distinct_source_decision() {
    let first = CssSourceSnapshot::new("a");
    let distinct = CssSourceSnapshot::new("a");
    for candidate in [&first, &distinct] {
        let steps = Cell::new(0);
        let mut admit = |action| {
            if action == CssSourceRetentionAction::Step {
                steps.set(steps.get() + 1);
            }
            steps.get() != 2
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
        assert_eq!(collector.bank.len(), 1);
        assert_eq!(collector.steps(), 1);
        assert_eq!(collector.usage(), usage);
        assert_eq!(
            collector.complete_payload_bytes_with(|| Ok::<_, ()>(())),
            Ok(prefix_payload)
        );
        assert_eq!(steps.get(), 2);
    }
}

#[test]
fn admitted_collector_keeps_identity_visit_depth_and_payload_policies() {
    let first = CssSourceSnapshot::new("a");
    let distinct = CssSourceSnapshot::new("a");
    let mut admit = |_| true;
    let mut collector = CssSourceRetention::with_admission(4096, 3, 1, &mut admit).unwrap();
    collector.source(&first).unwrap(); // One source action.
    collector.source(&first).unwrap(); // One source action, recorded identity.
    collector.source(&distinct).unwrap(); // One source action, distinct identity.
    assert_eq!(collector.steps(), 3);
    assert_eq!(collector.bank.len(), 2);
    assert_eq!(
        collector.usage().retained_bytes(),
        first.storage_usage().retained_bytes() + distinct.storage_usage().retained_bytes()
    );
    let payload = collector
        .complete_payload_bytes_with(|| Ok::<_, ()>(()))
        .unwrap();
    assert_eq!(collector.complete_payload_bytes_admitted(), Ok(payload));
    assert_eq!(collector.steps(), 3);
    assert_eq!(
        collector.source(&first),
        Err(CssSourceRetentionError::limit(
            crate::source::CssSourceRetentionResource::Visits
        ))
    );
    assert_eq!(collector.steps(), 3);
    let reads = Cell::new(0);
    let mut depth_admit = |_| true;
    let mut depth = CssSourceRetention::with_admission(4096, 10, 1, &mut depth_admit).unwrap();
    assert_eq!(
        depth.at_depth(1, |collector| collector.visit(&Reads(&reads))),
        Err(CssSourceRetentionError::limit(
            crate::source::CssSourceRetentionResource::Depth
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
    let report = crate::CssParser::default().parse_sheet(".a { --token: calc(1 + var(--other)); }");
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

/// The length of an identity index for a bank of `capacity` snapshots: none
/// for an empty bank, otherwise the smallest power of two that is at least
/// twice the capacity.
fn index_slots(capacity: usize) -> usize {
    if capacity == 0 {
        0
    } else {
        (capacity * 2).next_power_of_two()
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
    let first_capacity = collector.bank.capacity();
    for index in 1..first_capacity {
        collector
            .source(&CssSourceSnapshot::new(&index.to_string()))
            .unwrap();
    }
    collector.source(&second).unwrap();
    let second_capacity = collector.bank.capacity();
    drop(collector);
    let fixed = std::mem::size_of::<CssSourceRetention<'_>>();
    let bank_header = std::mem::size_of::<Vec<CssSourceSnapshot>>();
    let index_header = std::mem::size_of::<Vec<usize>>();
    let slot = std::mem::size_of::<CssSourceSnapshot>();
    let word = std::mem::size_of::<usize>();
    // Snapshots without an original backing need the bank and the snapshot index
    // only. Each growth step requests, then receives, the new bank and index
    // beside the old ones, with the temporary header of each.
    let headers = bank_header + index_header;
    let grown =
        |old: usize, new: usize| (old + new) * slot + (index_slots(old) + index_slots(new)) * word;
    assert_eq!(reports.len(), 5);
    assert_eq!(reports[0], fixed);
    assert_eq!(reports[1], fixed + headers + grown(0, 1));
    assert_eq!(reports[2], reports[1]);
    assert_eq!(
        reports[3],
        fixed + headers + grown(first_capacity, first_capacity * 2)
    );
    assert_eq!(
        reports[4],
        fixed + headers + grown(first_capacity, second_capacity)
    );
}

fn utf16_source(original: &std::sync::Arc<Vec<u16>>) -> CssSourceSnapshot {
    crate::CssParser::default()
        .parse_sheet_utf16(
            original.clone(),
            crate::source::CssUtf16InputLimits::default(),
            || Ok::<_, ()>(()),
        )
        .unwrap()
        .syntax()
        .parse_source()
        .unwrap()
        .source()
        .clone()
}

#[test]
fn the_original_backing_index_is_allocated_and_reported_with_the_first_original() {
    let word = std::mem::size_of::<usize>();
    let slot = std::mem::size_of::<CssSourceSnapshot>();
    let fixed = std::mem::size_of::<CssSourceRetention<'_>>();
    let bank_header = std::mem::size_of::<Vec<CssSourceSnapshot>>();
    let index_header = std::mem::size_of::<Vec<usize>>();
    let original = std::sync::Arc::new(vec![97_u16]);
    // Three snapshots of UTF-8 text, then one that has an original backing.
    let plain = (0..3)
        .map(|index| CssSourceSnapshot::new(&index.to_string()))
        .collect::<Vec<_>>();
    let backed = utf16_source(&original);
    let mut reports = Vec::new();
    let mut admit = |action| {
        if let CssSourceRetentionAction::Scratch { live_bytes } = action {
            reports.push(live_bytes);
        }
        true
    };
    let mut collector =
        CssSourceRetention::with_admission(usize::MAX, 1000, 10, &mut admit).unwrap();
    for source in &plain {
        collector.source(source).unwrap();
    }
    // The bank holds three snapshots in room for four; its tables have eight slots.
    assert_eq!(collector.bank.capacity(), 4);
    let before = collector.bank.capacity();
    collector.source(&backed).unwrap();
    assert_eq!(collector.bank.capacity(), before);
    drop(collector);
    // Four growth steps happened for the first three snapshots (1, 2 and 4 snapshots of
    // room, each requested and then actual), and one for the fourth snapshot, which
    // allocates only the original index, sized for the room of four.
    assert_eq!(reports.len(), 1 + 2 * 3 + 2);
    let held = 4 * slot + 8 * word;
    let original_index = 8 * word;
    assert_eq!(
        reports[7],
        fixed + held + original_index + index_header,
        "requested"
    );
    assert_eq!(reports[8], reports[7], "actual");
    assert_eq!(
        reports[6],
        fixed + (2 + 4) * slot + (4 + 8) * word + bank_header + index_header
    );
}

#[test]
fn an_original_backing_with_the_first_snapshot_replaces_bank_and_both_indexes() {
    let word = std::mem::size_of::<usize>();
    let slot = std::mem::size_of::<CssSourceSnapshot>();
    let fixed = std::mem::size_of::<CssSourceRetention<'_>>();
    let headers =
        std::mem::size_of::<Vec<CssSourceSnapshot>>() + 2 * std::mem::size_of::<Vec<usize>>();
    let backed = utf16_source(&std::sync::Arc::new(vec![97_u16]));
    let mut reports = Vec::new();
    let mut admit = |action| {
        if let CssSourceRetentionAction::Scratch { live_bytes } = action {
            reports.push(live_bytes);
        }
        true
    };
    let mut collector =
        CssSourceRetention::with_admission(usize::MAX, 1000, 10, &mut admit).unwrap();
    collector.source(&backed).unwrap();
    drop(collector);
    // Room for one snapshot: two slots in each of the two indexes.
    let expected = fixed + slot + 4 * word + headers;
    assert_eq!(reports, [fixed, expected, expected]);
}
