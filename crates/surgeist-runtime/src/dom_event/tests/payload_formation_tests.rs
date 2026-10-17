//! Actual frozen-path and failure-replacement admission under changed retry policy.
use super::*;
use std::mem::{size_of, size_of_val};

fn retained_failure() -> (
    Ordinary,
    DomEventDispatcher<Vec<usize>>,
    NativeDomEventOwner,
    Vec<usize>,
) {
    let mut fixture = ordinary();
    let mut dispatcher = registry();
    for index in 0..4 {
        let code = u64::try_from(index).unwrap();
        dispatcher
            .add_listener(
                &DomEventDom::borrow(&mut fixture.dom),
                fixture.target,
                DomEventType::Load,
                DomEventCallback::new(move |_, _, calls: &mut Vec<usize>, _| {
                    calls.push(index);
                    Err(DomEventCallbackError { code })
                }),
                DomEventListenerOptions::default(),
            )
            .unwrap();
    }
    let event = NativeDomEvent::style_load(fixture.target);
    let mut calls = Vec::new();
    assert!(matches!(
        dispatcher
            .dispatch(
                &mut fixture.dom,
                &mut calls,
                &event,
                DomEventDispatchLimits {
                    path_items: 12,
                    snapshot_listeners: 4,
                    failure_records: 1,
                    callbacks: 1,
                    work: 100_000,
                },
            )
            .unwrap(),
        DomEventDispatchOutcome::Retained(DomEventResource::Callbacks)
    ));
    assert_eq!(calls, [0]);
    (fixture, dispatcher, event, calls)
}

fn retry() -> DomEventDispatchLimits {
    DomEventDispatchLimits {
        path_items: 0,
        snapshot_listeners: 1,
        failure_records: 4,
        callbacks: 4,
        work: 100_000,
    }
}

#[test]
fn lowered_path_policy_keeps_actual_frozen_storage_and_admits_failure_bank_overlap() {
    let (mut fixture, mut dispatcher, event, mut calls) = retained_failure();
    let (expected_source, scope_bytes) = {
        let continuation = event._custody.borrow();
        let dispatch = continuation.as_ref().unwrap();
        assert_eq!(dispatch.path.capacity(), 12);
        assert_eq!(dispatch.snapshot.as_ref().unwrap().capacity(), 4);
        assert_eq!(dispatch.failures.capacity(), 1);
        assert_eq!(dispatch.failures.len(), 1);
        // These are separate simultaneously owned occurrences: event/control
        // bodies, frozen shared path and backing, retained snapshot backing,
        // old failure backing, and the prepared replacement owner/backing.
        // Vec fields already inline in Dispatch/EventPath are not added again.
        let old_live = size_of_val(event.event.0.as_ref())
            + size_of_val(event._custody.as_ref())
            + size_of_val(dispatch.path.as_ref())
            + dispatch.path.capacity() * size_of::<PathItem>()
            + dispatch.snapshot.as_ref().unwrap().capacity() * size_of::<DomEventRegistrationId>()
            + dispatch.failures.capacity() * size_of::<DomEventListenerFailure>();
        let replacement =
            size_of::<Vec<DomEventListenerFailure>>() + 4 * size_of::<DomEventListenerFailure>();
        (
            old_live + replacement,
            dispatch.association_custody.retained_bytes(),
        )
    };
    let formation = event.dispatch_formation(retry()).unwrap();
    assert_eq!(formation.source, expected_source);
    assert_eq!(formation.full, expected_source + scope_bytes);
    let receipt = complete(
        dispatcher
            .resume(&mut fixture.dom, &mut calls, &event, retry())
            .unwrap(),
    );
    assert_eq!(calls, [0, 1, 2, 3]);
    assert_eq!(receipt.callbacks(), 4);
    assert_eq!(receipt.failures().len(), 4);
    assert_eq!(receipt.failures.capacity(), 4);
    for (index, failure) in receipt.failures().iter().enumerate() {
        assert_eq!(
            failure.kind,
            DomEventListenerFailureKind::Reported(DomEventCallbackError {
                code: u64::try_from(index).unwrap(),
            })
        );
    }
}

#[test]
fn failure_copy_work_refusal_preserves_original_banks_and_cursor_for_no_replay_retry() {
    let (mut fixture, mut dispatcher, event, mut calls) = retained_failure();
    let (path, snapshot, failures, cursor) = {
        let continuation = event._custody.borrow();
        let dispatch = continuation.as_ref().unwrap();
        (
            dispatch.path.clone(),
            dispatch.snapshot.as_ref().unwrap().as_ptr(),
            dispatch.failures.as_ptr(),
            (dispatch.invoke, dispatch.listener, dispatch.callbacks),
        )
    };
    let stamp = fixture.dom.snapshot().stamp();
    // Zero work refuses before allocation; one permits allocation but refuses
    // the first retained failure copy. Neither may replace the original bank.
    for work in [0, 1] {
        assert!(matches!(
            dispatcher.resume(
                &mut fixture.dom,
                &mut calls,
                &event,
                DomEventDispatchLimits { work, ..retry() },
            ),
            Err(DomEventDispatchError::Admission(DomEventResource::Work))
        ));
        assert_eq!(calls, [0]);
        assert_eq!(fixture.dom.snapshot().stamp(), stamp);
        assert_eq!(event.0.status.get(), Status::Retained);
        assert_eq!(dispatcher.owner.pending.get(), 1);
        let continuation = event._custody.borrow();
        let dispatch = continuation.as_ref().unwrap();
        assert!(Rc::ptr_eq(&path, &dispatch.path));
        assert_eq!(dispatch.snapshot.as_ref().unwrap().as_ptr(), snapshot);
        assert_eq!(dispatch.failures.as_ptr(), failures);
        assert_eq!(dispatch.failures.capacity(), 1);
        assert_eq!(dispatch.failures.len(), 1);
        assert_eq!(
            (dispatch.invoke, dispatch.listener, dispatch.callbacks),
            cursor
        );
    }
    let receipt = complete(
        dispatcher
            .resume(&mut fixture.dom, &mut calls, &event, retry())
            .unwrap(),
    );
    assert_eq!(calls, [0, 1, 2, 3]);
    assert_eq!(receipt.callbacks(), 4);
    assert_eq!(receipt.failures().len(), 4);
    assert_eq!(dispatcher.owner.pending.get(), 0);
    assert!(event._custody.borrow().is_none());
    assert_eq!(event.0.status.get(), Status::Complete);
}
