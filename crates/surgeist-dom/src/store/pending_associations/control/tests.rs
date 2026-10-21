//! Genuine original-entry reservation: both bootstrap paths, shared release.
use super::super::style_prefix::tests::{envelope, install, parse, setup};
use super::*;
fn flag() -> AtomicBool {
    AtomicBool::new(false)
}
#[test]
fn lazy_and_explicit_receipts_charge_metadata_once_and_share_exact_high_water() {
    for explicit in [false, true] {
        let (mut dom, env) = setup();
        let (_, id) = install(&mut dom, parse("<!doctype html><style>later</style>"), env);
        let record = dom.pending_associations.controlled_entry(id).unwrap();
        record.value().control.bind(id, envelope()).unwrap();
        let ledger = dom.custody_registry.ledger().clone();
        let base = ledger.total();
        let initial = record.value().control.bytes.load(Ordering::Relaxed);
        let meter = Meter::new(record.clone(), ledger.clone());
        meter.retain(64, 0, 0, &flag()).unwrap();
        let baseline = ledger.total();
        let admission = if explicit {
            Some(meter.admission(baseline, &flag()).unwrap())
        } else {
            None
        };
        let debit = ledger
            .retain(custody::Usage::new(17, 0), |total| match &admission {
                Some(a) => a.controlled(total, &flag()),
                None => meter.controlled(total, baseline, &flag()),
            })
            .unwrap();
        let mut graph = Vec::<u8>::new();
        graph.try_reserve_exact(17).unwrap();
        assert_eq!(graph.capacity(), 17);
        let shared = meter.admission(baseline, &flag()).unwrap();
        let metadata = size_of::<Charged>();
        assert_eq!(
            record.value().control.bytes.load(Ordering::Relaxed),
            initial + 64 + metadata + 17
        );
        assert_eq!(
            ledger.total(),
            base.checked_add(custody::Usage::new(64 + metadata + 17, 0))
                .unwrap()
        );
        // Repeated and decreased reports consume work, never duplicate debit.
        let work = record.value().control.spent_work();
        shared.controlled(ledger.total(), &flag()).unwrap();
        shared.controlled(baseline, &flag()).unwrap();
        assert_eq!(record.value().control.spent_work(), work + 2);
        assert_eq!(
            record.value().control.bytes.load(Ordering::Relaxed),
            initial + 64 + metadata + 17
        );
        drop(graph);
        drop(debit);
        drop(admission);
        drop(meter);
        assert_eq!(
            record.value().control.bytes.load(Ordering::Relaxed),
            initial + metadata + 17
        );
        assert_eq!(
            ledger.total(),
            base.checked_add(custody::Usage::new(metadata, 0)).unwrap()
        );
        drop(shared);
        assert_eq!(
            record.value().control.bytes.load(Ordering::Relaxed),
            initial
        );
        assert_eq!(ledger.total(), base);
    }
}
#[test]
fn one_short_growth_refuses_without_allocation_or_duplicate_bootstrap_and_releases_exactly() {
    let (mut dom, env) = setup();
    let (_, id) = install(&mut dom, parse("<!doctype html><style>later</style>"), env);
    let record = dom.pending_associations.controlled_entry(id).unwrap();
    let bound = 64 + size_of::<Charged>() + 16;
    record
        .value()
        .control
        .bind(
            id,
            DomParsedHtmlAssociationLimits::new(bound, 100, 1000, 10).unwrap(),
        )
        .unwrap();
    let ledger = dom.custody_registry.ledger().clone();
    let base = ledger.total();
    let meter = Meter::new(record.clone(), ledger.clone());
    meter.retain(64, 0, 0, &flag()).unwrap();
    let baseline = ledger.total();
    let result = ledger.retain(custody::Usage::new(17, 0), |total| {
        meter.controlled(total, baseline, &flag())
    });
    assert!(
        matches!(result,Err(DomError::ResourceLimit {resource:DomResource::ParsedAssociationProcessingRetainedBytes,limit}) if limit==bound)
    );
    assert_eq!(
        ledger.total(),
        base.checked_add(custody::Usage::new(64 + size_of::<Charged>(), 0))
            .unwrap()
    );
    assert_eq!(
        record.value().control.bytes.load(Ordering::Relaxed),
        64 + size_of::<Charged>()
    );
    drop(meter);
    assert_eq!(ledger.total(), base);
    assert_eq!(record.value().control.bytes.load(Ordering::Relaxed), 0);
}
#[test]
fn real_draft_and_formation_forks_use_one_original_positive_delta_receipt() {
    let (mut dom, env) = setup();
    let (_, id) = install(&mut dom, parse("<!doctype html><style>later</style>"), env);
    let record = dom.pending_associations.controlled_entry(id).unwrap();
    record.value().control.bind(id, envelope()).unwrap();
    let ledger = dom.custody_registry.ledger().clone();
    let baseline = ledger.total();
    let meter = Meter::new(record.clone(), ledger.clone());
    let receipt = meter.admission(baseline, &flag()).unwrap();
    let draft = Draft::new_parsed(&dom, Arc::new(BatchIdentity), &receipt, &flag()).unwrap();
    let formed = crate::store::source::formation::Formation::new(&draft, &flag())
        .unwrap()
        .unwrap();
    let own = record.value().control.bytes.load(Ordering::Relaxed);
    for _ in 0..2 {
        let fork = formed.fork();
        fork.retain_payload(
            crate::store::source::storage::Payload {
                bytes: 64,
                units: 32,
            },
            &flag(),
        )
        .unwrap();
        let mut value = Vec::<u16>::new();
        value.try_reserve_exact(32).unwrap();
        assert_eq!(
            record.value().control.bytes.load(Ordering::Relaxed),
            own + 64
        );
        assert_eq!(record.value().control.units.load(Ordering::Relaxed), 32);
        drop(value);
        drop(fork);
        // Actual graph/debit is gone, while the same original peak reservation
        // remains shared by real Draft/Formation authority until final release.
        assert_eq!(
            record.value().control.bytes.load(Ordering::Relaxed),
            own + 64
        );
    }
    drop(formed);
    drop(draft);
    drop(meter);
    assert_eq!(
        ledger.total(),
        baseline
            .checked_add(custody::Usage::new(size_of::<Charged>(), 0))
            .unwrap()
    );
    assert!(record.value().control.bytes.load(Ordering::Relaxed) > size_of::<Charged>());
    drop(receipt);
    assert_eq!(ledger.total(), baseline);
    assert_eq!(record.value().control.bytes.load(Ordering::Relaxed), 0);
    assert_eq!(record.value().control.units.load(Ordering::Relaxed), 0);
}
