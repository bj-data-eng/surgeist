//! Actual Source dispatch backing outlives its producer only while retained.
use super::*;
use crate::dom_event::manipulation_observation_layout;
use surgeist_dom::{DomResource, DomSourceEdit, DomSourceProgress};

fn actual_source_owner() -> (Dom, SourceEventOwner, [NodeId; 2]) {
    let cancel = AtomicBool::new(false);
    let mut dom = Dom::new(
        DomLimits::new(
            128,
            100_000,
            256,
            128,
            128,
            32,
            4_000_000,
            surgeist_dom::DomNameLimits::new(16_777_216, 16_384, 4_000_000, 1_048_576).unwrap(),
        )
        .unwrap()
        .with_dispatch_custody(
            DomDispatchCustodyLimits::new(64, 4096, 16_000_000, 16_000_000, 4_000_000).unwrap(),
        ),
    )
    .unwrap();
    let mut batch = DomBatch::new();
    let document = batch.create_document();
    let parent = batch.create_element_ns(&document, Namespace::html(), "div");
    let first = batch.create_element_ns(&document, Namespace::html(), "dialog");
    let second = batch.create_element_ns(&document, Namespace::html(), "dialog");
    batch.insert_before(&parent, &document, None);
    batch.insert_before(&first, &parent, None);
    batch.insert_before(&second, &parent, None);
    let commit = dom.apply(batch, &cancel).unwrap();
    let targets = [first, second].map(|ticket| commit.created(&ticket).unwrap().unwrap());
    let executor = dom.bind_source_event_executor(&cancel).unwrap();
    let policy = DomSourceEventRuntimeLimits {
        jobs: 2,
        storage_bytes: 16_000_000,
        work: 1_000_000,
    };
    SourceEventOwner::admit_initial(policy).unwrap();
    let storage = crate::dom_event::EventStorage::session(
        &mut DomEventDom::borrow(&mut dom),
        crate::DomEventStorageLimits {
            storage_bytes: 16_000_000,
            work: 1_000_000,
        },
        &cancel,
    )
    .unwrap();
    (
        dom,
        SourceEventOwner::new(targets[0].owner(), executor, policy, storage),
        targets,
    )
}

fn source_request(dom: &mut Dom, target: NodeId, cancel: &AtomicBool) -> DomSourceEventRequest {
    let mut lease = dom
        .begin_source_edit(DomSourceEdit::show_dialog(target), limits().source, cancel)
        .unwrap();
    let DomSourceProgress::Request(DomSourceRequest::Event(request)) =
        lease.advance(cancel).unwrap()
    else {
        panic!("actual Show beforetoggle request")
    };
    request
}

fn source_headroom(dom: &Dom, request: &DomSourceEventRequest, cancel: &AtomicBool) -> usize {
    let limit = limits().source.retained_bytes();
    let mut admitted = 0;
    let mut refused = limit + 1;
    while refused - admitted > 1 {
        let probe = admitted + (refused - admitted) / 2;
        match dom.retain_source_event_payload(request, probe, 0, cancel) {
            Ok(retained) => {
                drop(retained);
                admitted = probe;
            }
            Err(DomError::ResourceLimit {
                resource: DomResource::SourceRetainedBytes,
                limit: actual,
            }) if actual == limit => refused = probe,
            other => panic!("the original Source byte allowance decides the cut: {other:?}"),
        }
    }
    admitted
}

fn parked_source_dispatch_releases_dynamic_custody(withdraw_dispatcher: bool) {
    let cancel = AtomicBool::new(false);
    let (mut dom, owner, [target, _]) = actual_source_owner();
    let runtime = owner.execution();
    let mut registry = dispatcher();
    for callback in [
        DomEventCallback::new(|_, _, calls: &mut Vec<u64>, _| {
            calls.push(1);
            Err(DomEventCallbackError { code: 91 })
        }),
        DomEventCallback::host(17),
        DomEventCallback::new(|_, _, calls: &mut Vec<u64>, _| {
            calls.push(2);
            Ok(())
        }),
    ] {
        registry
            .add_listener(
                &mut DomEventDom::borrow(&mut dom),
                target,
                DomEventType::BEFORE_TOGGLE,
                callback,
                DomEventListenerOptions::default(),
            )
            .unwrap();
    }
    let request = source_request(&mut dom, target, &cancel);
    let mut calls = Vec::new();
    let mut invocation = Some(pending(
        runtime
            .service_event(
                &mut DomEventDom::borrow(&mut dom),
                &request,
                &mut registry,
                &mut calls,
                limits(),
                &cancel,
            )
            .unwrap(),
    ));
    assert_eq!(calls, [1]);
    assert_eq!(dom.source_dispatch_blocker(), None);
    let event = invocation.as_ref().unwrap().event().clone();
    assert_eq!(event.current_target(), Some(target.into()));
    let (event_body, path) = manipulation_observation_layout(&event, &event.composed_path());
    let (snapshot, failures, association) = event.inspect_retained_dispatch_banks();
    let dynamic = path + snapshot + failures;
    let fixed = size_of::<RefCell<State>>()
        + DomSourceEventExecutor::binding_payload_bytes()
        + size_of::<std::cell::Cell<usize>>()
        + size_of::<RefCell<Job>>()
        + event_body
        + event.inspect_continuation_body_bytes();
    assert!(path > 0 && snapshot > 0 && failures > 0 && association > 0);
    assert!(owner.state.borrow_mut().bytes(&cancel).unwrap() >= fixed + dynamic + association);
    let before = source_headroom(&dom, &request, &cancel);
    let canonical_before = dom.dispatch_custody_retained_bytes(&cancel).unwrap();
    let stamp = dom.snapshot().stamp();

    let mut registry = Some(registry);
    let registry_release = if withdraw_dispatcher {
        let dispatcher = registry.as_ref().unwrap();
        let bytes =
            dispatcher.inspect_pending_index_bytes() + dispatcher.inspect_registration_bytes();
        drop(registry.take());
        bytes
    } else {
        drop(invocation.take());
        0
    };
    assert_eq!(dom.source_dispatch_blocker(), Some(request.operation()));
    assert_eq!(owner.pending().unwrap(), Some(request.identity()));
    assert_eq!(event.current_target(), None);
    assert_eq!(event.phase(), DomEventPhase::None);
    assert_eq!(event.composed_path().count(), 0);
    let runtime_after = owner.state.borrow_mut().bytes(&cancel).unwrap();
    let (canonical_after, after) = {
        // Compare the two Source cuts with a real guard for the same original
        // request on both sides. The probe enters no callback and publishes nothing.
        let state = owner.state.borrow();
        let job = state.jobs.as_ref().unwrap().borrow();
        let Phase::Dispatching {
            permit: Some(permit),
        } = &job.phase
        else {
            panic!("the pending original job still owns its permit")
        };
        let _scope = state
            .executor
            .enter_dispatch(&dom, permit, &cancel)
            .unwrap();
        (
            dom.dispatch_custody_retained_bytes(&cancel).unwrap(),
            source_headroom(&dom, &request, &cancel),
        )
    };
    assert_eq!(dom.source_dispatch_blocker(), Some(request.operation()));
    assert!(canonical_after < canonical_before);
    let source_release = after - before - (canonical_before - canonical_after) - registry_release;

    let mut registry = registry.unwrap_or_else(dispatcher);
    if let Some(invocation) = invocation.as_mut() {
        assert!(matches!(
            invocation.complete(&registry, Ok(())),
            Err(DomEventDispatchError::Abandoned)
        ));
    }
    assert!(matches!(
        runtime.service_event(
            &mut DomEventDom::borrow(&mut dom),
            &request,
            &mut registry,
            &mut calls,
            limits(),
            &cancel,
        ),
        Err(DomSourceEventRuntimeError::Dispatch(
            DomEventDispatchError::Abandoned
        ))
    ));
    assert_eq!(calls, [1]);
    assert_eq!(owner.pending().unwrap(), Some(request.identity()));
    assert_eq!(dom.snapshot().stamp(), stamp);
    assert!(
        dom.snapshot()
            .node(target)
            .unwrap()
            .attribute(&"open".into())
            .is_none()
    );
    let runtime_retry = owner.state.borrow_mut().bytes(&cancel).unwrap();
    assert_eq!(
        (runtime_after, runtime_retry, source_release),
        (fixed, fixed, dynamic),
        "the original job and fixed bodies survive while parked backing releases immediately"
    );
}

#[test]
fn source_dispatcher_drop_releases_parked_backing_with_original_job_alive() {
    parked_source_dispatch_releases_dynamic_custody(true);
}

#[test]
fn source_host_token_drop_releases_parked_backing_with_original_job_alive() {
    parked_source_dispatch_releases_dynamic_custody(false);
}

#[test]
fn escaped_source_path_retains_original_debit_after_job_and_event_disposal() {
    let cancel = AtomicBool::new(false);
    let (mut dom, owner, [target, next]) = actual_source_owner();
    let runtime = owner.execution();
    let mut registry = dispatcher();
    registry
        .add_listener(
            &mut DomEventDom::borrow(&mut dom),
            target,
            DomEventType::BEFORE_TOGGLE,
            DomEventCallback::host(17),
            DomEventListenerOptions::default(),
        )
        .unwrap();
    let request = source_request(&mut dom, target, &cancel);
    let mut calls = Vec::new();
    let mut invocation = pending(
        runtime
            .service_event(
                &mut DomEventDom::borrow(&mut dom),
                &request,
                &mut registry,
                &mut calls,
                limits(),
                &cancel,
            )
            .unwrap(),
    );
    let event = invocation.event().clone();
    let path = event.composed_path();
    let second_path = event.composed_path();
    let (_, bytes) = manipulation_observation_layout(&event, &path);
    assert!(bytes > 0);
    let parent = dom.snapshot().node(target).unwrap().parent().unwrap();
    let document = dom.snapshot().node(parent).unwrap().parent().unwrap();
    event.prevent_default();
    invocation.complete(&registry, Ok(())).unwrap();
    let DomSourceEventServiceProgress::Serviced { receipt } = runtime
        .service_event(
            &mut DomEventDom::borrow(&mut dom),
            &request,
            &mut registry,
            &mut calls,
            limits(),
            &cancel,
        )
        .unwrap()
    else {
        panic!("the actual host result completes")
    };
    assert_eq!(receipt.callbacks(), 1);
    assert!(receipt.default_prevented());
    drop(receipt);
    assert_eq!(owner.pending().unwrap(), None);
    let mut tail = dom
        .resume_source_edit(request.operation(), limits().source, &cancel)
        .unwrap();
    assert!(matches!(
        tail.advance(&cancel).unwrap(),
        DomSourceProgress::Complete(_)
    ));
    drop(tail);
    drop(invocation);
    drop(event);
    drop(registry);
    assert!(
        dom.snapshot()
            .node(target)
            .unwrap()
            .attribute(&"open".into())
            .is_none()
    );

    let request = source_request(&mut dom, next, &cancel);
    let held = source_headroom(&dom, &request, &cancel);
    let live = owner.state.borrow_mut().bytes(&cancel).unwrap();
    drop(second_path);
    assert_eq!(source_headroom(&dom, &request, &cancel), held);
    assert_eq!(owner.state.borrow_mut().bytes(&cancel).unwrap(), live);
    assert_eq!(
        path.collect::<Vec<_>>(),
        [target.into(), parent.into(), document.into()]
    );
    let released = source_headroom(&dom, &request, &cancel);
    let remaining = owner.state.borrow_mut().bytes(&cancel).unwrap();
    assert_eq!(
        remaining,
        size_of::<RefCell<State>>()
            + DomSourceEventExecutor::binding_payload_bytes()
            + size_of::<std::cell::Cell<usize>>()
    );
    assert_eq!((live - remaining, released - held), (bytes, bytes));
}
