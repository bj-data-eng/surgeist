use super::*;
use surgeist_dom::{DomBatch, identity::NodeId};

fn fixture() -> (DocumentScriptHost, DocumentScriptPermit) {
    let cancel = Arc::new(AtomicBool::new(false));
    let (session, top) = crate::document_scripting::cross_realm_tests::hosted_session(&cancel, 0);
    let host = attach_public_nodes(session, top, cancel, 200_000, 2_000_000);
    let permit = host.permit().unwrap();
    (host, permit)
}

fn open_dataset(host: &mut DocumentScriptHost, permit: &DocumentScriptPermit) -> NodeId {
    let node = host
        .in_document(permit, |scope| {
            let element = scope.evaluate(&units(
                r"(() => {
                    const element = document.createElement('div');
                    element.setAttributeNS('urn:other', 'p:data-value', 'original');
                    element.setAttribute('data-value', 'later');
                    globalThis.datasetProbe = element.dataset;
                    return element;
                })()",
            ))?;
            projected_node_id(scope, &element)
        })
        .unwrap();
    fund_guest_operation_after_setup(host);
    node
}

pub(in crate::document_scripting) fn dataset_read_admits_exact_native_observation() {
    let read = |scope: &mut Scope<'_>| assert_guest(scope, "datasetProbe.value === 'original'");
    let (mut host, permit) = fixture();
    open_dataset(&mut host, &permit);
    let events = host.guest_events().unwrap();
    let initial = (events.budget.work().unwrap(), events.budget.used());
    let (result, peak) = events
        .budget
        .measure_peak(|| host.in_document(&permit, read));
    result.unwrap();
    let work = events.budget.work().unwrap() - initial.0;
    let bytes = peak - initial.1;
    assert!(work > 0 && bytes > 0);
    assert_eq!(events.budget.used(), initial.1);
    drop(events);
    host.stop().into_session().unwrap();

    for (work_remaining, storage_remaining) in [(work, bytes), (work - 1, bytes), (work, bytes - 1)]
    {
        let (mut host, permit) = fixture();
        open_dataset(&mut host, &permit);
        let events = host.guest_events().unwrap();
        let reserved = events.limits.storage_bytes - events.budget.used() - storage_remaining;
        let fill = events.reserve(&host.native, reserved).unwrap();
        let stamp = host.snapshot().unwrap().stamp();
        let used = events.budget.used();
        events.budget.set_work_remaining(work_remaining);
        let result = host.in_document(&permit, read);
        if (work_remaining, storage_remaining) == (work, bytes) {
            result.unwrap();
        } else {
            let Err(DocumentScriptError::Script(Error::HostFailure(failure))) = result else {
                panic!("one-short native dataset read must refuse: {result:?}");
            };
            assert!(matches!(
                (
                    work_remaining < work,
                    failure.cause::<DocumentGuestEventError>()
                ),
                (true, Some(DocumentGuestEventError::WorkLimit))
                    | (
                        false,
                        Some(DocumentGuestEventError::Document(
                            DocumentScriptError::Native(DocumentScriptNativeError::StorageLimit)
                        ))
                    )
            ));
        }
        assert_eq!(host.snapshot().unwrap().stamp(), stamp);
        assert_eq!(events.budget.used(), used);
        drop(fill);
        let budget = events.budget.clone();
        drop(events);
        host.stop().into_session().unwrap();
        assert_eq!(budget.used(), 0);
    }
}

pub(in crate::document_scripting) fn dataset_releases_retired_element_and_map() {
    let (mut host, permit) = fixture();
    let destroy = |host: &mut DocumentScriptHost, node| {
        host.with_native(|dom, _, _| {
            let mut batch = DomBatch::new();
            batch.destroy(node);
            dom.apply(batch, &AtomicBool::new(false)).unwrap();
        })
        .unwrap();
        assert!(host.snapshot().unwrap().node(node).is_err());
    };
    let collect = |host: &mut DocumentScriptHost| {
        host.checkpoint().unwrap();
        host.engine
            .as_mut()
            .unwrap()
            .process
            .execution()
            .memory_pressure();
        host.in_document(&permit, |_| Ok(())).unwrap();
    };
    // Keep the native slot bank identical in the control and the measured map.
    let control = host
        .in_document(&permit, |scope| {
            let element = scope.evaluate(&units("document.createElement('div')"))?;
            projected_node_id(scope, &element)
        })
        .unwrap();
    destroy(&mut host, control);
    collect(&mut host);
    let budget = host.guest_events().unwrap().budget.clone();
    let baseline = budget.used();
    let element = open_dataset(&mut host, &permit);
    assert!(budget.used() > baseline);
    collect(&mut host);
    host.in_document(&permit, |scope| {
        assert_guest(scope, "datasetProbe.value === 'original'")
    })
    .unwrap();
    destroy(&mut host, element);
    host.in_document(&permit, |scope| {
        scope
            .evaluate(&units("delete globalThis.datasetProbe"))
            .map(drop)
    })
    .unwrap();
    for _ in 0..8 {
        collect(&mut host);
        if budget.used() == baseline {
            break;
        }
    }
    assert_eq!(budget.used(), baseline);
    host.stop().into_session().unwrap();
    assert_eq!(budget.used(), 0);
}
