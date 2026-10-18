//! Original issuer calls: occupied and historical extent is administrative work.
use super::*;

const UNRELATED: usize = 256;
const PATH_ALLOWANCE: u64 = 64;
fn cancel() -> AtomicBool {
    AtomicBool::new(false)
}
fn issuer() -> DomHostedSource {
    DomHostedSource::new(
        DomHostedSourceId {
            owner: DomOwnerId::allocate().unwrap(),
            serial: 0,
        },
        DomHostedLimits::new(
            DomHostedRecordLimits::new(512, 512, 512, 512, 512, 512, 512),
            64_000_000,
            64_000_000,
            10_000_000,
        ),
        &cancel(),
    )
    .unwrap()
}
fn occupied(source: &mut DomHostedSource, count: usize) -> Vec<DomHostedOperationId> {
    let work = source.work(&cancel()).unwrap();
    (0..count)
        .map(|_| source.begin_operation(&work, 2, 0, &cancel()).unwrap())
        .collect()
}

#[test]
fn operation_append_selection_does_not_walk_unrelated_live_rows() {
    fn sample(count: usize) -> (u64, u64) {
        let mut source = issuer();
        let old = occupied(&mut source, count);
        let work = source.work(&cancel()).unwrap();
        let before = work.spent.load(Ordering::Relaxed);
        assert_eq!(source.next_operation_slot(&work, &cancel()).unwrap(), count);
        let preview = work.spent.load(Ordering::Relaxed) - before;
        let before = work.spent.load(Ordering::Relaxed);
        let next = source.begin_operation(&work, 2, 0, &cancel()).unwrap();
        let issued = work.spent.load(Ordering::Relaxed) - before;
        assert_eq!(next.source, source.id());
        assert_eq!(next.slot, count);
        assert_eq!(next.generation, 1);
        for id in old {
            source
                .observe()
                .validate(id.source, Role::Operation, id.slot, id.generation)
                .unwrap();
        }
        (preview, issued)
    }
    let small = sample(0);
    let large = sample(UNRELATED);
    assert!(
        large.0 <= small.0 + PATH_ALLOWANCE,
        "occupied-prefix preview: small={small:?}, large={large:?}"
    );
    assert!(
        large.1 <= small.1 + PATH_ALLOWANCE,
        "append issuance: small={small:?}, large={large:?}"
    );
}

#[test]
fn operation_lowest_reuse_does_not_recount_unrelated_historical_extent() {
    fn sample(count: usize) -> u64 {
        let mut source = issuer();
        let ids = occupied(&mut source, count);
        for id in &ids {
            source
                .retire(
                    Role::Operation,
                    id.source,
                    id.slot,
                    id.generation,
                    &cancel(),
                )
                .unwrap();
        }
        let work = source.work(&cancel()).unwrap();
        assert_eq!(source.next_operation_slot(&work, &cancel()).unwrap(), 0);
        let before = work.spent.load(Ordering::Relaxed);
        let next = source.begin_operation(&work, 2, 0, &cancel()).unwrap();
        let steps = work.spent.load(Ordering::Relaxed) - before;
        assert_eq!(next.slot, 0);
        assert_eq!(next.generation, 2);
        assert_eq!(
            source
                .observe()
                .validate(ids[0].source, Role::Operation, 0, ids[0].generation),
            Err(DomHostedError::StaleIdentity)
        );
        source
            .observe()
            .validate(next.source, Role::Operation, next.slot, next.generation)
            .unwrap();
        steps
    }
    let small = sample(1);
    let large = sample(UNRELATED);
    assert!(
        large <= small + PATH_ALLOWANCE,
        "one reused slot spent {large} steps across {UNRELATED} retired rows, versus {small}"
    );
}

#[test]
fn operation_terminal_generation_is_skipped_and_cancelled_issue_preserves_reservation() {
    let mut source = issuer();
    let ids = occupied(&mut source, 3);
    // Producer-owned terminal-state characterization, as in the existing Host
    // exhaustion test. This does not fabricate an external operation receipt.
    source.lock().unwrap().banks[Role::Operation.index()][ids[0].slot].generation = u64::MAX;
    source
        .retire(
            Role::Operation,
            ids[0].source,
            ids[0].slot,
            u64::MAX,
            &cancel(),
        )
        .unwrap();
    source
        .retire(
            Role::Operation,
            ids[2].source,
            ids[2].slot,
            ids[2].generation,
            &cancel(),
        )
        .unwrap();
    source
        .retire(
            Role::Operation,
            ids[1].source,
            ids[1].slot,
            ids[1].generation,
            &cancel(),
        )
        .unwrap();
    let work = source.work(&cancel()).unwrap();
    assert_eq!(source.next_operation_slot(&work, &cancel()).unwrap(), 1);
    let revision = source.revision().unwrap();
    let reserved = source.lock().unwrap().reserved_revisions;
    let cancelled = AtomicBool::new(true);
    assert!(matches!(
        source.begin_operation(&work, 2, 0, &cancelled),
        Err(DomHostedError::Dom(DomError::Cancelled))
    ));
    assert_eq!(source.revision().unwrap(), revision);
    assert_eq!(source.lock().unwrap().reserved_revisions, reserved);
    let next = source.begin_operation(&work, 2, 0, &cancel()).unwrap();
    assert_eq!(next.slot, 1);
    assert_eq!(next.generation, 2);
    assert_eq!(source.next_operation_slot(&work, &cancel()).unwrap(), 2);
}
