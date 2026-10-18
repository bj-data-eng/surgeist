//! Canonical registry-bank admission, without hosted execution or service claims.

use super::*;
use crate::hosted::{DomHostedLimits, DomHostedRecordLimits, DomHostedSourceId};
use std::sync::atomic::Ordering;

#[test]
fn registry_append_prices_shared_rows_once_and_preserves_old_on_one_short_refusal() {
    let cancel = AtomicBool::new(false);
    let owner = DomOwnerId::allocate().unwrap();
    let limits = DomHostedLimits::new(
        DomHostedRecordLimits::new(0, 0, 0, 0, 0, 0, 64),
        64_000_000,
        64_000_000,
        10_000_000,
    );
    let mut source =
        DomHostedSource::new(DomHostedSourceId { owner, serial: 0 }, limits, &cancel).unwrap();
    let work = source.work(&cancel).unwrap();
    let mut old = None;
    for _ in 0..8 {
        old = Some(
            Bank::form(old.as_deref(), owner, &source, &work, None, &cancel)
                .unwrap()
                .0,
        );
    }
    let old = old.unwrap();
    let first_capture = old.records[0].capture.identity().clone();
    let used = || source.limits().retained_bytes() - source.remaining_bytes().unwrap();
    let baseline = used();
    let (next, id) = Bank::form(Some(&old), owner, &source, &work, None, &cancel).unwrap();
    let mut actual = std::collections::BTreeMap::new();
    for graph in [&old, &next] {
        for (id, usage) in graph.inspect_materializations().unwrap() {
            if let Some(previous) = actual.insert(id, usage) {
                assert_eq!(previous, usage);
            }
        }
    }
    let union: usize = actual.values().map(|usage| usage.bytes()).sum();
    let old_extent: usize = old
        .inspect_materializations()
        .unwrap()
        .iter()
        .map(|(_, usage)| usage.bytes())
        .sum();
    let additional = union - old_extent;
    assert_eq!(used(), baseline + additional);
    assert_eq!(id.slot, 8);
    assert_eq!(next.records[0].capture.identity(), &first_capture);
    assert!(old.get(id).is_err());
    drop(next);
    assert_eq!(used(), baseline);
    let spent = work.spent.load(Ordering::Relaxed);
    source
        .set_limits(
            DomHostedLimits::new(
                limits.records(),
                baseline + additional - 1,
                limits.scratch_bytes(),
                limits.work(),
            ),
            &cancel,
        )
        .unwrap();
    assert!(matches!(
        Bank::form(Some(&old), owner, &source, &work, None, &cancel),
        Err(DomHostedError::Limit {
            resource: DomHostedResource::RetainedBytes,
            ..
        }) | Err(DomHostedError::Dom(DomError::ResourceLimit { .. }))
    ));
    assert_eq!(
        source.limits().retained_bytes() - source.remaining_bytes().unwrap(),
        baseline
    );
    assert_eq!(old.records.len(), 8);
    assert_eq!(old.records[0].capture.identity(), &first_capture);
    assert!(work.spent.load(Ordering::Relaxed) > spent);
    source.set_limits(limits, &cancel).unwrap();
    let (recovered, id) = Bank::form(Some(&old), owner, &source, &work, None, &cancel).unwrap();
    assert_eq!(id.slot, 8);
    assert!(recovered.get(id).is_ok());
}

#[test]
fn registry_outer_bank_append_work_is_bounded() {
    let cancel = AtomicBool::new(false);
    let owner = DomOwnerId::allocate().unwrap();
    let source = DomHostedSource::new(
        DomHostedSourceId { owner, serial: 0 },
        DomHostedLimits::new(
            DomHostedRecordLimits::new(64, 64, 64, 64, 64, 64, 64),
            64_000_000,
            64_000_000,
            10_000_000,
        ),
        &cancel,
    )
    .unwrap();
    let work = source.work(&cancel).unwrap();
    let (mut bank, first) = Bank::form(None, owner, &source, &work, None, &cancel).unwrap();
    let first_capture = bank.get(first).unwrap().capture.identity().clone();
    let before = work.spent.load(Ordering::Relaxed);
    let (small, _) = Bank::form(Some(&bank), owner, &source, &work, None, &cancel).unwrap();
    let small_steps = work.spent.load(Ordering::Relaxed) - before;
    drop(small);
    for _ in 1..48 {
        bank = Bank::form(Some(&bank), owner, &source, &work, None, &cancel)
            .unwrap()
            .0;
    }
    let before = work.spent.load(Ordering::Relaxed);
    let (large, id) = Bank::form(Some(&bank), owner, &source, &work, None, &cancel).unwrap();
    let large_steps = work.spent.load(Ordering::Relaxed) - before;
    assert_eq!(id.slot, 48);
    assert_eq!(large.get(first).unwrap().capture.identity(), &first_capture);
    assert!(bank.get(id).is_err());
    assert!(
        large_steps <= small_steps + 24,
        "one registry append spent {large_steps} steps after 48 unrelated records, versus {small_steps} after one"
    );
}
