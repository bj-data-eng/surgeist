//! Owning empty-storage/affine-credit proofs; no external service completion.
use super::*;
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
            DomHostedRecordLimits::new(16, 16, 16, 16, 16, 16, 16),
            16_000_000,
            16_000_000,
            1_000_000,
        ),
        &cancel(),
    )
    .unwrap()
}
fn used(source: &DomHostedSource) -> usize {
    source.retained_storage_bytes()
}
fn limit(source: &mut DomHostedSource, bytes: usize) {
    let old = source.limits();
    source
        .set_limits(
            DomHostedLimits::new(old.records(), bytes, old.scratch_bytes(), old.work()),
            &cancel(),
        )
        .unwrap();
}
#[test]
fn owned_body_credit_and_raw_empty_storage_have_exact_retained_boundary() {
    let mut source = issuer();
    let work = source.work(&cancel()).unwrap();
    let rows = Operations::default();
    let before = used(&source);
    let raw = {
        let credit = source.retain(size_of::<Operation>(), &cancel()).unwrap();
        let prepared = rows
            .prepare(
                0,
                &work,
                &source,
                0,
                None,
                BodyCredit::Owned(credit),
                Projection::default(),
                &cancel(),
            )
            .unwrap();
        let raw = prepared.inspect_new_storage_bytes();
        assert_eq!(used(&source), before + raw);
        drop(prepared);
        raw
    };
    assert_eq!(used(&source), before);
    limit(&mut source, before + raw - 1);
    let credit = source.retain(size_of::<Operation>(), &cancel()).unwrap();
    let spent = work.spent.load(Ordering::Relaxed);
    assert!(matches!(
        rows.prepare(
            0,
            &work,
            &source,
            0,
            None,
            BodyCredit::Owned(credit),
            Projection::default(),
            &cancel()
        ),
        Err(DomHostedError::Limit {
            resource: DomHostedResource::RetainedBytes,
            ..
        })
    ));
    assert_eq!(used(&source), before);
    assert!(rows.rows.is_empty());
    assert!(work.spent.load(Ordering::Relaxed) > spent);
    limit(&mut source, before + raw);
    let credit = source.retain(size_of::<Operation>(), &cancel()).unwrap();
    let prepared = rows
        .prepare(
            0,
            &work,
            &source,
            0,
            None,
            BodyCredit::Owned(credit),
            Projection::default(),
            &cancel(),
        )
        .unwrap();
    assert_eq!(prepared.inspect_new_storage_bytes(), raw);
    assert_eq!(used(&source), before + raw);
    drop(prepared);
    assert_eq!(used(&source), before);
}
#[test]
fn borrowed_body_credit_survives_refusal_cancel_drop_and_same_original_retry() {
    let mut source = issuer();
    let work = source.work(&cancel()).unwrap();
    let rows = Operations::default();
    let before = used(&source);
    let credit = source.retain(size_of::<Operation>(), &cancel()).unwrap();
    let original = used(&source);
    let extra = {
        let prepared = rows
            .prepare(
                0,
                &work,
                &source,
                0,
                None,
                BodyCredit::Borrowed(&credit),
                Projection::default(),
                &cancel(),
            )
            .unwrap();
        let extra = prepared.inspect_new_storage_bytes() - size_of::<Operation>();
        assert_eq!(used(&source), original + extra);
        drop(prepared);
        extra
    };
    assert_eq!(used(&source), original);
    limit(&mut source, original + extra - 1);
    assert!(matches!(
        rows.prepare(
            0,
            &work,
            &source,
            0,
            None,
            BodyCredit::Borrowed(&credit),
            Projection::default(),
            &cancel()
        ),
        Err(DomHostedError::Limit {
            resource: DomHostedResource::RetainedBytes,
            ..
        })
    ));
    assert_eq!(used(&source), original);
    let cancelled = AtomicBool::new(true);
    assert!(matches!(
        rows.prepare(
            0,
            &work,
            &source,
            0,
            None,
            BodyCredit::Borrowed(&credit),
            Projection::default(),
            &cancelled
        ),
        Err(DomHostedError::Dom(DomError::Cancelled))
    ));
    assert_eq!(used(&source), original);
    limit(&mut source, original + extra);
    let prepared = rows
        .prepare(
            0,
            &work,
            &source,
            0,
            None,
            BodyCredit::Borrowed(&credit),
            Projection::default(),
            &cancel(),
        )
        .unwrap();
    assert_eq!(used(&source), original + extra);
    drop(prepared);
    assert_eq!(used(&source), original);
    drop(credit);
    assert_eq!(used(&source), before);
}
#[test]
fn foreign_or_insufficient_original_credit_never_allocates_a_destination() {
    let source = issuer();
    let foreign = issuer();
    let work = source.work(&cancel()).unwrap();
    let rows = Operations::default();
    let before = used(&source);
    let foreign_before = used(&foreign);
    let credit = foreign.retain(size_of::<Operation>(), &cancel()).unwrap();
    assert!(matches!(
        rows.prepare(
            0,
            &work,
            &source,
            0,
            None,
            BodyCredit::Owned(credit),
            Projection::default(),
            &cancel()
        ),
        Err(DomHostedError::ForeignSource)
    ));
    assert_eq!(used(&foreign), foreign_before);
    assert_eq!(used(&source), before);
    let credit = source
        .retain(size_of::<Operation>() - 1, &cancel())
        .unwrap();
    assert!(matches!(
        rows.prepare(
            0,
            &work,
            &source,
            0,
            None,
            BodyCredit::Owned(credit),
            Projection::default(),
            &cancel()
        ),
        Err(DomHostedError::RepresentationExhausted)
    ));
    assert_eq!(used(&source), before);
    assert!(rows.rows.is_empty());
}
#[test]
fn actual_dom_and_owning_controller_remain_send() {
    fn send<T: Send>() {}
    send::<Dom>();
    send::<Operations>();
}
