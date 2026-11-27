//! The measurement of one payload and the ledger of measurements against the
//! collector that visits the same payloads, which is the independent figure of
//! every test: its steps, the deepest nesting it reaches and the bytes it retains.
use super::{CssSourceBounds, CssSourceDelta, CssSourceLedger, CssSourceMeasurement};
use crate::CssParser;
use crate::source::source_retention::{
    Admission, CssRetainedSources, CssSourceRetention, CssSourceRetentionAction,
    CssSourceRetentionError, CssSourceRetentionResource,
};
use crate::source::{CssSourceSnapshot, CssUtf16InputLimits};
use std::sync::Arc;

fn utf8(index: usize) -> CssSourceSnapshot {
    CssParser::default()
        .parse_sheet(format!(".s{index} {{ color: red }}"))
        .syntax()
        .parse_source()
        .unwrap()
        .source()
        .clone()
}

/// A snapshot of `original`; two of them share one UTF-16 backing and are
/// distinct snapshots.
fn utf16(original: &Arc<Vec<u16>>) -> CssSourceSnapshot {
    CssParser::default()
        .parse_sheet_utf16(original.clone(), CssUtf16InputLimits::default(), || {
            Ok::<_, ()>(())
        })
        .unwrap()
        .syntax()
        .parse_source()
        .unwrap()
        .source()
        .clone()
}

fn original(index: usize) -> Arc<Vec<u16>> {
    Arc::new(
        format!(".o{index} {{ margin: 1px }}")
            .encode_utf16()
            .collect(),
    )
}

/// Four UTF-8 snapshots, then three snapshots of each of three originals.
fn pool() -> Vec<CssSourceSnapshot> {
    let mut pool = (0..4).map(utf8).collect::<Vec<_>>();
    for index in 0..3 {
        let backing = original(index);
        pool.extend((0..3).map(|_| utf16(&backing)));
    }
    pool
}

/// A payload that charges `extra` steps, records its snapshots and nests
/// `depth` owners below itself, the innermost recording the first snapshot again.
struct Payload<'a> {
    snapshots: Vec<&'a CssSourceSnapshot>,
    extra: usize,
    depth: usize,
}
impl CssRetainedSources for Payload<'_> {
    fn visit_retained_sources(
        &self,
        sources: &mut CssSourceRetention<'_>,
    ) -> Result<(), CssSourceRetentionError> {
        for _ in 0..self.extra {
            sources.charge_step()?;
        }
        for snapshot in &self.snapshots {
            sources.source(snapshot)?;
        }
        if self.depth > 0 {
            sources.visit(&Payload {
                snapshots: self.snapshots.iter().take(1).copied().collect(),
                extra: 0,
                depth: self.depth - 1,
            })?;
        }
        Ok(())
    }
}

fn payload<'a>(pool: &'a [CssSourceSnapshot], indices: &[usize], depth: usize) -> Payload<'a> {
    Payload {
        snapshots: indices.iter().map(|index| &pool[*index]).collect(),
        extra: indices.len() % 3,
        depth,
    }
}

const BIG: CssSourceBounds = CssSourceBounds::new(usize::MAX, usize::MAX, 64);

fn measure(payload: &Payload<'_>) -> CssSourceMeasurement {
    CssSourceMeasurement::of(BIG, 0, |sources| sources.visit(payload)).unwrap()
}

/// The collector that visits `payloads` in order.
fn collect(payloads: &[&Payload<'_>]) -> CssSourceRetention<'static> {
    let mut collector = CssSourceRetention::with_shared_original_utf16(usize::MAX, usize::MAX, 64);
    for payload in payloads {
        collector.visit(*payload).unwrap();
    }
    collector
}

#[test]
fn a_measurement_reports_the_steps_nesting_and_snapshots_the_collector_counts() {
    let pool = pool();
    for (indices, depth) in [
        (vec![0usize], 0usize),
        (vec![0, 1, 2], 2),
        (vec![4, 5, 6], 1),
        (vec![4, 7, 8, 0, 4], 3),
        (vec![], 4),
    ] {
        let payload = payload(&pool, &indices, depth);
        let collector = collect(&[&payload]);
        let measurement = measure(&payload);
        assert_eq!(measurement.steps(), collector.steps(), "{indices:?}");
        assert_eq!(measurement.depth(), collector.deepest(), "{indices:?}");
        assert_eq!(measurement.sources().len(), collector.bank.len());
        let mut ledger = CssSourceLedger::new();
        ledger.add(&measurement).unwrap();
        assert_eq!(
            ledger.bytes(),
            collector.usage().retained_bytes(),
            "{indices:?}"
        );
    }
}

#[test]
fn a_measurement_at_a_base_depth_reports_the_nesting_below_it() {
    let pool = pool();
    let payload = payload(&pool, &[0, 1], 2);
    let at_base = CssSourceMeasurement::of(BIG, 5, |sources| sources.visit(&payload)).unwrap();
    assert_eq!(at_base.depth(), measure(&payload).depth());
    let bounded = CssSourceBounds::new(usize::MAX, usize::MAX, 7);
    let refused = CssSourceMeasurement::of(bounded, 5, |sources| sources.visit(&payload));
    assert_eq!(
        refused.unwrap_err().exhausted_limit(),
        Some(CssSourceRetentionResource::Depth)
    );
    let allowed = CssSourceBounds::new(usize::MAX, usize::MAX, 8);
    assert!(CssSourceMeasurement::of(allowed, 5, |sources| sources.visit(&payload)).is_ok());
}

#[test]
fn a_measurement_within_a_step_budget_is_refused_when_the_payload_needs_more() {
    let pool = pool();
    let payload = payload(&pool, &[0, 1, 2, 3], 1);
    let steps = measure(&payload).steps();
    for budget in 0..=steps {
        let bounds = CssSourceBounds::new(usize::MAX, budget, 64);
        let measured = CssSourceMeasurement::of(bounds, 0, |sources| sources.visit(&payload));
        if budget < steps {
            assert_eq!(
                measured.unwrap_err().exhausted_limit(),
                Some(CssSourceRetentionResource::Visits),
                "budget {budget} of {steps}"
            );
        } else {
            assert_eq!(measured.unwrap().steps(), steps);
        }
    }
}

/// A deterministic order of additions and removals: `true` adds the payload
/// at that index, `false` removes it (and only when it is added).
fn schedules() -> Vec<Vec<(usize, bool)>> {
    vec![
        vec![
            (0, true),
            (1, true),
            (2, true),
            (3, true),
            (0, false),
            (2, false),
            (1, false),
            (3, false),
        ],
        vec![
            (3, true),
            (2, true),
            (1, true),
            (0, true),
            (3, false),
            (1, false),
            (0, false),
            (2, false),
        ],
        vec![
            (1, true),
            (3, true),
            (1, false),
            (0, true),
            (2, true),
            (3, false),
            (1, true),
            (0, false),
            (2, false),
            (1, false),
        ],
    ]
}

#[test]
fn the_ledger_counts_what_the_collector_counts_in_any_order_of_additions_and_removals() {
    let pool = pool();
    let payloads = [
        payload(&pool, &[0, 4, 5], 1),
        payload(&pool, &[5, 6, 7, 1], 0),
        payload(&pool, &[7, 8, 9, 10], 2),
        payload(&pool, &[2, 10, 11, 4, 0], 1),
    ];
    let measurements = payloads.iter().map(measure).collect::<Vec<_>>();
    for schedule in schedules() {
        let mut ledger = CssSourceLedger::new();
        let mut added = [false; 4];
        for (index, add) in schedule {
            if add && !added[index] {
                ledger.add(&measurements[index]).unwrap();
                added[index] = true;
            } else if !add && added[index] {
                ledger.remove(&measurements[index]);
                added[index] = false;
            }
            let present = payloads
                .iter()
                .zip(added)
                .filter_map(|(payload, added)| added.then_some(payload))
                .collect::<Vec<_>>();
            let collector = collect(&present);
            assert_eq!(
                ledger.bytes(),
                collector.usage().retained_bytes(),
                "{added:?}"
            );
            assert_eq!(ledger.len(), collector.bank.len(), "{added:?}");
            assert_eq!(ledger.is_empty(), present.is_empty());
        }
        assert!(ledger.is_empty());
        assert_eq!(ledger.bytes(), 0);
    }
}

#[test]
fn a_payload_added_twice_leaves_the_ledger_when_removed_twice() {
    let pool = pool();
    let payload = payload(&pool, &[4, 5, 0], 0);
    let measurement = measure(&payload);
    let mut ledger = CssSourceLedger::new();
    ledger.add(&measurement).unwrap();
    let once = ledger.bytes();
    ledger.add(&measurement).unwrap();
    assert_eq!((ledger.bytes(), ledger.len()), (once, 3));
    ledger.remove(&measurement);
    assert_eq!((ledger.bytes(), ledger.len()), (once, 3));
    ledger.remove(&measurement);
    assert_eq!((ledger.bytes(), ledger.len()), (0, 0));
    // A payload that was never added changes nothing.
    ledger.remove(&measurement);
    assert_eq!((ledger.bytes(), ledger.len()), (0, 0));
}

#[test]
fn an_addition_that_overflows_the_bytes_leaves_the_ledger_unchanged() {
    let pool = pool();
    let mut measurement = of_sources(&[&pool[0], &pool[1]]);
    measurement.sources[1].bytes = usize::MAX;
    let mut ledger = CssSourceLedger::new();
    // The first snapshot is held when the second overflows the sum.
    ledger.add(&of_sources(&[&pool[2]])).unwrap();
    let before = (ledger.bytes(), ledger.len());
    let error = ledger.add(&measurement).unwrap_err();
    assert_eq!(
        error.exhausted_limit(),
        Some(CssSourceRetentionResource::RetainedBytes)
    );
    assert_eq!((ledger.bytes(), ledger.len()), before);
}

#[test]
fn a_delta_of_a_payload_removed_and_added_again_changes_nothing() {
    let pool = pool();
    let payload = payload(&pool, &[4, 5, 0], 0);
    let measurement = measure(&payload);
    let mut delta = CssSourceDelta::new();
    delta.remove(&measurement).unwrap();
    delta.add(&measurement).unwrap();
    assert!(delta.is_empty());
    let mut ledger = CssSourceLedger::new();
    ledger.add(&measurement).unwrap();
    let held = (ledger.bytes(), ledger.len());
    ledger.apply(&delta).unwrap();
    assert_eq!((ledger.bytes(), ledger.len()), held);
}

#[test]
fn a_delta_applied_gives_the_ledger_the_collector_counts_for_the_payloads_that_remain() {
    let pool = pool();
    let payloads = [
        payload(&pool, &[0, 4, 5], 1),
        payload(&pool, &[5, 6, 7, 1], 0),
        payload(&pool, &[7, 8, 9, 10], 2),
        payload(&pool, &[2, 10, 11, 4, 0], 1),
    ];
    let measurements = payloads.iter().map(measure).collect::<Vec<_>>();
    // Start with the first two, replace the second by the third and fourth.
    let mut ledger = CssSourceLedger::new();
    ledger.add(&measurements[0]).unwrap();
    ledger.add(&measurements[1]).unwrap();
    let mut delta = CssSourceDelta::new();
    delta.remove(&measurements[1]).unwrap();
    delta.add(&measurements[2]).unwrap();
    delta.add(&measurements[3]).unwrap();
    assert!(!delta.is_empty());
    ledger.apply(&delta).unwrap();
    let collector = collect(&[&payloads[0], &payloads[2], &payloads[3]]);
    assert_eq!(ledger.bytes(), collector.usage().retained_bytes());
    assert_eq!(ledger.len(), collector.bank.len());
    ledger.assert_consistent();
    // Inverted, the delta restores the first two payloads.
    delta.invert();
    ledger.apply(&delta).unwrap();
    let collector = collect(&[&payloads[0], &payloads[1]]);
    assert_eq!(ledger.bytes(), collector.usage().retained_bytes());
    assert_eq!(ledger.len(), collector.bank.len());
    ledger.assert_consistent();
}

#[test]
fn merged_deltas_make_the_change_of_both() {
    let pool = pool();
    let (first, second) = (
        measure(&payload(&pool, &[0, 4, 5], 1)),
        measure(&payload(&pool, &[5, 6, 7], 0)),
    );
    let mut ledger = CssSourceLedger::new();
    ledger.add(&first).unwrap();
    // One delta removes the first payload, the other adds it back and adds the second.
    let (mut removal, mut addition) = (CssSourceDelta::new(), CssSourceDelta::new());
    removal.remove(&first).unwrap();
    addition.add(&first).unwrap();
    addition.add(&second).unwrap();
    removal.merge(&addition).unwrap();
    ledger.apply(&removal).unwrap();
    let mut alone = CssSourceLedger::new();
    alone.add(&first).unwrap();
    alone.add(&second).unwrap();
    assert_eq!((ledger.bytes(), ledger.len()), (alone.bytes(), alone.len()));
    // A delta merged with its own inverse changes nothing.
    let mut cancelled = CssSourceDelta::new();
    cancelled.add(&second).unwrap();
    let mut inverse = CssSourceDelta::new();
    inverse.remove(&second).unwrap();
    cancelled.merge(&inverse).unwrap();
    assert!(cancelled.is_empty());
}

#[test]
fn a_delta_of_a_ledger_adds_each_distinct_snapshot_once() {
    let pool = pool();
    let payload = payload(&pool, &[4, 5, 0], 0);
    let measurement = measure(&payload);
    let mut inner = CssSourceLedger::new();
    inner.add(&measurement).unwrap();
    inner.add(&measurement).unwrap();
    let mut outer = CssSourceLedger::new();
    let mut delta = CssSourceDelta::new();
    delta.add_ledger(&inner).unwrap();
    delta.add_ledger(&inner).unwrap();
    outer.apply(&delta).unwrap();
    assert_eq!((outer.bytes(), outer.len()), (inner.bytes(), inner.len()));
    // Two references were added: one removal leaves them held.
    let mut once = CssSourceDelta::new();
    once.remove_ledger(&inner).unwrap();
    outer.apply(&once).unwrap();
    assert_eq!((outer.bytes(), outer.len()), (inner.bytes(), inner.len()));
    outer.apply(&once).unwrap();
    assert_eq!((outer.bytes(), outer.len()), (0, 0));
}

/// The measurement of a payload that records `snapshots` in turn.
fn of_sources(snapshots: &[&CssSourceSnapshot]) -> CssSourceMeasurement {
    CssSourceMeasurement::of(BIG, 0, |sources| {
        for snapshot in snapshots {
            sources.source(snapshot)?;
        }
        Ok(())
    })
    .unwrap()
}

#[test]
fn recording_the_nesting_adds_nothing_to_the_size_of_the_collector() {
    // The deepest nesting is kept in the padding after the collector's flags, so
    // the live bytes it reports for its own metadata are the same as without it.
    assert_eq!(
        std::mem::size_of::<Admission<'static>>(),
        std::mem::size_of::<Option<&mut dyn FnMut(CssSourceRetentionAction) -> bool>>()
            + std::mem::size_of::<usize>()
    );
}
