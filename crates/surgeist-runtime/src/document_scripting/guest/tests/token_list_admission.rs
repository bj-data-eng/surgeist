use super::*;
use surgeist_dom::{DomBatch, identity::NodeId};

fn fixture() -> (DocumentScriptHost, DocumentScriptPermit) {
    let cancel = Arc::new(AtomicBool::new(false));
    let (session, target) =
        crate::document_scripting::cross_realm_tests::hosted_session(&cancel, 0);
    let host = attach_public_nodes(session, target, cancel, 200_000, 2_000_000);
    let permit = host.permit().unwrap();
    (host, permit)
}

fn open_list(host: &mut DocumentScriptHost, permit: &DocumentScriptPermit) -> NodeId {
    let node = host
        .in_document(permit, |scope| {
            let element = scope.evaluate(&units(
                r"(() => {
                const element = document.createElement('div');
                element.setAttribute('class', 'one two one');
                globalThis.classProbe = element.classList;
                return element;
            })()",
            ))?;
            projected_node_id(scope, &element)
        })
        .unwrap();
    fund_guest_operation_after_setup(host);
    node
}

fn destroy_element(host: &mut DocumentScriptHost, node: NodeId) {
    host.with_native(|dom, _, _| {
        let mut batch = DomBatch::new();
        batch.destroy(node);
        dom.apply(batch, &AtomicBool::new(false)).unwrap();
    })
    .unwrap();
    assert!(host.snapshot().unwrap().node(node).is_err());
}

pub(in crate::document_scripting) fn class_token_read_admits_exact_work_and_temporary_backing() {
    let read = |scope: &mut Scope<'_>| assert_guest(scope, "classProbe.item(1) === 'two'");
    let (mut host, permit) = fixture();
    open_list(&mut host, &permit);
    let events = host.guest_events().unwrap();
    let before = (events.budget.work().unwrap(), events.budget.used());
    let (observed, peak) = events
        .budget
        .measure_peak(|| host.in_document(&permit, read));
    observed.unwrap();
    let work = events.budget.work().unwrap() - before.0;
    let bytes = peak - before.1;
    assert!(work > 0 && bytes > 0);
    assert_eq!(events.budget.used(), before.1);
    drop(events);
    host.stop().into_session().unwrap();

    for (available_work, available_bytes) in [(work, bytes), (work - 1, bytes), (work, bytes - 1)] {
        let (mut host, permit) = fixture();
        open_list(&mut host, &permit);
        let events = host.guest_events().unwrap();
        let fill_bytes = events.limits.storage_bytes - events.budget.used() - available_bytes;
        let fill = events.reserve(&host.native, fill_bytes).unwrap();
        let before = (host.snapshot().unwrap().stamp(), events.budget.used());
        events.budget.set_work_remaining(available_work);
        let observed = host.in_document(&permit, read);
        assert_eq!(host.snapshot().unwrap().stamp(), before.0);
        assert_eq!(events.budget.used(), before.1);
        if (available_work, available_bytes) == (work, bytes) {
            observed.unwrap();
        } else {
            let Err(DocumentScriptError::Script(Error::HostFailure(failure))) = observed else {
                panic!("one-short token observation must refuse: {observed:?}");
            };
            assert!(matches!(
                (
                    available_work < work,
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
        drop(fill);
        let budget = events.budget.clone();
        drop(events);
        host.stop().into_session().unwrap();
        assert_eq!(budget.used(), 0);
    }
}

pub(in crate::document_scripting) fn retained_class_list_keeps_then_releases_its_traced_element() {
    let (mut host, permit) = fixture();
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
    // The native registry permanently allocates its first slot bank. Reuse a
    // retired slot so the control and measured projection have the same bank.
    let control = host
        .in_document(&permit, |scope| {
            let element = scope.evaluate(&units("document.createElement('div')"))?;
            projected_node_id(scope, &element)
        })
        .unwrap();
    destroy_element(&mut host, control);
    collect(&mut host);
    let budget = host.guest_events().unwrap().budget.clone();
    let initial = budget.used();
    let element = open_list(&mut host, &permit);
    assert!(budget.used() > initial);

    collect(&mut host);
    host.in_document(&permit, |scope| {
        assert_guest(
            scope,
            "classProbe.toggle('new') === true && classProbe.value === 'one two new'",
        )
    })
    .unwrap();

    // A live native Element pins its canonical wrapper and SameObject view.
    // Removing that pin allows the guest collector to reclaim the pair.
    destroy_element(&mut host, element);
    host.in_document(&permit, |scope| {
        scope
            .evaluate(&units("delete globalThis.classProbe"))
            .map(drop)
    })
    .unwrap();

    for _ in 0..8 {
        collect(&mut host);
        if budget.used() == initial {
            break;
        }
    }
    assert_eq!(budget.used(), initial);
    host.stop().into_session().unwrap();
    assert_eq!(budget.used(), 0);
}

pub(in crate::document_scripting) fn cached_class_list_authenticates_the_current_element() {
    let (mut host, permit) = fixture();
    let element = host
        .in_document(&permit, |scope| {
            let node = scope.evaluate(&units(
                r"(() => {
                const node = globalThis.retiredOwner = document.createElement('div');
                globalThis.retainedList = node.classList;
                return node;
            })()",
            ))?;
            assert_guest(scope, "retiredOwner.classList === retainedList")?;
            projected_node_id(scope, &node)
        })
        .unwrap();
    destroy_element(&mut host, element);
    let result = host.in_document(&permit, |scope| {
        scope.evaluate(&units("retiredOwner.classList")).map(drop)
    });
    let Err(DocumentScriptError::Script(Error::HostFailure(failure))) = result else {
        panic!("a retired Element must refuse its cached classList read: {result:?}");
    };
    assert!(matches!(
        failure.cause::<DocumentGuestEventError>(),
        Some(DocumentGuestEventError::Document(
            DocumentScriptError::Native(DocumentScriptNativeError::Dom(DomError::StaleNode))
        ))
    ));
}

pub(in crate::document_scripting) fn token_edit_preserves_the_original_source_refusal() {
    use crate::internal::dom_source::{DomSourceDriveError, DomSourceRuntimeError};

    let cancel = Arc::new(AtomicBool::new(false));
    let (mut session, target) =
        crate::document_scripting::cross_realm_tests::hosted_session(&cancel, 0);
    let mut batch = DomBatch::new();
    let element = batch.create_element(target.document(), "div");
    batch.set_attribute(&element, "class", "one  one");
    let element = session
        .dom()
        .apply(batch, &cancel)
        .unwrap()
        .created(&element)
        .unwrap()
        .unwrap();
    let mut host = attach_public_nodes(session, target, cancel, 200_000, 0);
    let permit = host.permit().unwrap();
    expose_public_node_for_source_test(&mut host, &permit, element, "tokenSourceOwner");
    host.in_document(&permit, |scope| {
        assert_guest(
            scope,
            "(globalThis.sourceTokens = tokenSourceOwner.classList).value === 'one  one'",
        )
    })
    .unwrap();
    let before = host.snapshot().unwrap();
    let backing = host.guest_events().unwrap().budget.used();
    let result = host.in_document(&permit, |scope| {
        scope
            .evaluate(&units("sourceTokens.toggle('two')"))
            .map(drop)
    });
    let Err(DocumentScriptError::Script(Error::HostFailure(failure))) = result else {
        panic!("the native Source work limit must refuse the token write: {result:?}");
    };
    assert!(matches!(
        failure.cause::<DomSourceDriveError>(),
        Some(DomSourceDriveError::Source(DomSourceRuntimeError::Dom(
            DomError::SourceWorkLimit { limit: 0 }
        )))
    ));
    let after = host.snapshot().unwrap();
    assert_eq!(after.stamp(), before.stamp());
    assert_eq!(
        after.node(element).unwrap().attribute(&"class".into()),
        Some(&"one  one".into()),
    );
    assert_eq!(host.guest_events().unwrap().budget.used(), backing);
    host.with_native(|dom, _, _| assert!(dom.pending_source_edit().is_none()))
        .unwrap();
}
