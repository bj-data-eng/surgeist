use super::*;
use surgeist_dom::{
    DomBatch,
    edit::DomChange,
    limits::DomResource,
    range::{DomActiveRangeState, DomRangeId},
};

fn range_host(source_work: u64) -> (DocumentScriptHost, DocumentScriptPermit, DomRangeId) {
    let cancellation = Arc::new(AtomicBool::new(false));
    let (mut session, target) =
        super::super::super::cross_realm_tests::hosted_session_with_dom_limits(
            &cancellation,
            0,
            0,
            |limits| limits.with_ranges(2).unwrap(),
        );
    let mut batch = DomBatch::new();
    let independent = batch.create_range(target.document());
    let receipt = session.dom().apply(batch, &cancellation).unwrap();
    let independent = receipt.created_range(&independent).unwrap().unwrap();
    let host = attach_public_nodes(session, target, cancellation, 40_000, source_work);
    let permit = host.permit().unwrap();
    (host, permit, independent)
}

fn collect(host: &mut DocumentScriptHost) {
    host.engine
        .as_mut()
        .unwrap()
        .process
        .execution()
        .memory_pressure();
    host.checkpoint().unwrap();
}

pub(in crate::document_scripting) fn live_range_cleanup_keeps_native_associations() {
    let (mut host, permit, independent) = range_host(200_000);
    let canonical = permit.document();
    let before = host.snapshot().unwrap();
    let baseline = host.guest_events().unwrap().budget.used();
    host.in_document(&permit, |scope| {
        assert_guest(scope, "(globalThis.rangeCycle = document.createRange(), rangeCycle.back = [rangeCycle], rangeCycle.startContainer === document)")
    }).unwrap();
    let first = host
        .with_native(|dom, _, _| {
            assert_eq!(dom.usage().ranges(), 2);
            let reported = dom.raw().changes_since(before.stamp()).unwrap();
            let [DomChange::RangeCreated(first)] = reported.changes() else {
                panic!("the factory must publish exactly one new canonical Range");
            };
            let mut batch = DomBatch::new();
            batch.set_active_range(canonical, Some((*first).into()));
            dom.apply(batch, &AtomicBool::new(false)).unwrap();
            *first
        })
        .unwrap();
    host.in_document(&permit, |scope| {
        assert_guest(scope, "delete globalThis.rangeCycle")
    })
    .unwrap();
    collect(&mut host);
    assert_eq!(host.guest_events().unwrap().budget.used(), baseline);
    let active = host.snapshot().unwrap();
    assert_eq!(
        active.document(canonical).unwrap().active_range(),
        DomActiveRangeState::Known(Some(first))
    );
    assert_eq!(
        active.range(first).unwrap().start().container(),
        canonical.node()
    );

    host.with_native(|dom, _, _| {
        let mut batch = DomBatch::new();
        batch.set_active_range(canonical, None);
        dom.apply(batch, &AtomicBool::new(false)).unwrap();
    })
    .unwrap();
    let reaped = host.snapshot().unwrap();
    assert!(matches!(reaped.range(first), Err(DomError::StaleRange)));
    assert!(reaped.range(independent).is_ok());
    assert!(active.range(first).is_ok());

    host.in_document(&permit, |scope| {
        assert_guest(
            scope,
            "(globalThis.heldRange = document.createRange(), heldRange.collapsed)",
        )
    })
    .unwrap();
    host.with_native(|dom, _, _| assert_eq!(dom.usage().ranges(), 2))
        .unwrap();
    let mut session = host.stop().into_session().unwrap();
    assert_eq!(session.dom().usage().ranges(), 1);
    assert!(session.snapshot().range(independent).is_ok());
}

pub(in crate::document_scripting) fn refused_range_creation_releases_prepared_projection() {
    let (mut host, permit, independent) = range_host(0);
    let stamp = host.snapshot().unwrap().stamp();
    let events = host.guest_events().unwrap();
    let baseline = events.budget.used();
    let result = host.in_document(&permit, |scope| {
        scope.evaluate(&units("document.createRange()"))?;
        Ok(())
    });
    let Err(DocumentScriptError::Script(Error::HostFailure(failure))) = result else {
        panic!("the zero-work native Source must refuse Range creation: {result:?}");
    };
    assert!(matches!(
        failure.cause::<DocumentGuestEventError>(),
        Some(DocumentGuestEventError::Document(
            DocumentScriptError::Native(DocumentScriptNativeError::Dom(
                DomError::SourceWorkLimit { limit: 0 }
            ))
        ))
    ));
    drop(failure);
    assert_eq!(events.budget.used(), baseline);
    assert_eq!(host.snapshot().unwrap().stamp(), stamp);
    assert!(host.snapshot().unwrap().range(independent).is_ok());
    host.with_native(|dom, _, _| {
        assert_eq!(dom.usage().ranges(), 1);
        assert!(dom.pending_source_edit().is_none());
        let mut batch = DomBatch::new();
        let available = batch.create_range(permit.document());
        let receipt = dom.apply(batch, &AtomicBool::new(false)).unwrap();
        assert_ne!(
            receipt.created_range(&available).unwrap().unwrap(),
            independent
        );
        let mut extra = DomBatch::new();
        extra.create_range(permit.document());
        assert!(matches!(
            dom.apply(extra, &AtomicBool::new(false)),
            Err(DomError::ResourceLimit {
                resource: DomResource::Ranges,
                ..
            })
        ));
    })
    .unwrap();
    drop(events);
    host.stop().into_session().unwrap();
}
