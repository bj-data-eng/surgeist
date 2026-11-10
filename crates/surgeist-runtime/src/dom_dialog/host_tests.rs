//! Requirements: a selected host invocation retains actual event/Source custody,
//! releases all caller loans, and advances only after one authentic completion.
use super::*;
use crate::{
    DomEventCallback, DomEventCallbackError, DomEventHostError, DomEventHostInvocation,
    DomEventListenerFailureKind, DomEventListenerOptions, DomEventPhase, DomEventRegistryLimits,
    DomEventTarget, DomSession, DomSessionLimits, DomShutdownOutcome, DomShutdownReason,
    DomSourceDriveProgress, DomSourceRuntime, DomSourceServices, DomSourceUnavailable,
    DomStyleRuntimeLimits,
};
use std::cell::RefCell;
use surgeist_dom::{
    DomBatch, DomDispatchCustodyLimits, DomHostedLimits, DomHostedRecordLimits, DomLimits,
    DomSourceEdit, DomSourceRequest, Namespace, NodeId,
};

fn limits() -> DomSourceEventServiceLimits {
    DomSourceEventServiceLimits {
        source: DomSourceLimits::new(32_000_000, 1_000_000, 10_000_000, 128, 4096, 4096, 8),
        dispatch: DomEventDispatchLimits {
            path_items: 4,
            snapshot_listeners: 16,
            failure_records: 16,
            callbacks: 16,
            work: 100_000,
        },
    }
}
fn fixture() -> (DomSession, NodeId, NodeId, NodeId) {
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
    let dialog = batch.create_element_ns(&document, Namespace::html(), "dialog");
    let child = batch.create_element_ns(&document, Namespace::html(), "dialog");
    batch.insert_before(&parent, &document, None);
    batch.insert_before(&dialog, &parent, None);
    batch.insert_before(&child, &parent, None);
    let commit = dom.apply(batch, &cancel).unwrap();
    let document = commit.created(&document).unwrap().unwrap();
    let dialog = commit.created(&dialog).unwrap().unwrap();
    let child = commit.created(&child).unwrap().unwrap();
    let session = DomSession::new(
        dom,
        DomSessionLimits {
            source_events: DomSourceEventRuntimeLimits {
                jobs: 8,
                storage_bytes: 16_000_000,
                work: 1_000_000,
            },
            style: DomStyleRuntimeLimits {
                roots: 4,
                completions: 8,
                reports: 8,
                storage_bytes: 1_000_000,
                work: 100_000,
                css_input: surgeist_css::source::CssUtf16InputLimits::default(),
            },
            element_tasks: crate::DomElementTaskRuntimeLimits {
                tasks: 8,
                storage_bytes: 1_000_000,
                work: 100_000,
            },
            hosted: crate::DomHostedRuntimeLimits {
                canonical: DomHostedLimits::new(
                    DomHostedRecordLimits::new(8, 8, 8, 8, 8, 8, 8),
                    1_000_000,
                    1_000_000,
                    100_000,
                ),
                tasks: 8,
                storage_bytes: 1_000_000,
                scratch_bytes: 1_000_000,
                work: 100_000,
            },
        },
    )
    .unwrap();
    (session, document, dialog, child)
}
fn dispatcher() -> DomEventDispatcher<Vec<u64>> {
    DomEventDispatcher::new(DomEventRegistryLimits {
        registrations: 16,
        pending_events: 8,
        nested_dispatches: 8,
    })
    .unwrap()
}

#[test]
fn completed_source_body_custody_covers_host_event_and_index_observers_once() {
    for (index_first, owner_first) in [(false, false), (false, true), (true, false), (true, true)] {
        let (mut session, _, dialog, _) = fixture();
        let mut source = session.source_execution();
        let mut dispatcher = dispatcher();
        dispatcher
            .add_listener(
                &mut session.dom(),
                dialog,
                DomEventType::BeforeToggle,
                DomEventCallback::host(17),
                DomEventListenerOptions::default(),
            )
            .unwrap();
        let request = show(&mut source, &mut session, dialog);
        let mut calls = Vec::new();
        let mut invocation = pending(service(
            &source,
            &mut session,
            &request,
            &mut dispatcher,
            &mut calls,
        ));
        let event = invocation.event().clone();
        let (live, event_body, continuation_body) = event.inspect_fixed_body_storage();
        let counter_body = std::mem::size_of_val(live.as_ref());
        assert_eq!(live.get(), counter_body + event_body + continuation_body);
        event.prevent_default();
        invocation.complete(&dispatcher, Ok(())).unwrap();
        let DomSourceEventServiceProgress::Serviced { receipt } =
            service(&source, &mut session, &request, &mut dispatcher, &mut calls)
        else {
            panic!("actual host completion is accepted once")
        };
        assert_eq!(receipt.callbacks(), 1);
        assert!(receipt.default_prevented());
        assert!(calls.is_empty());
        drop(receipt);
        canceled_tail(&mut source, &mut session, &request);
        assert_eq!(event.phase(), DomEventPhase::None);
        assert_eq!(event.current_target(), None);
        assert_eq!(live.get(), counter_body + event_body + continuation_body);
        drop(invocation);
        assert_eq!(live.get(), counter_body + event_body + continuation_body);
        assert!(session.dom().pending_source_edit().is_none());
        let owner = if owner_first {
            drop(session);
            None
        } else {
            Some(session)
        };
        assert_eq!(live.get(), counter_body + event_body + continuation_body);
        if index_first {
            drop(dispatcher);
            assert_eq!(live.get(), counter_body + event_body + continuation_body);
            drop(event);
        } else {
            drop(event);
            assert_eq!(live.get(), counter_body + continuation_body);
            drop(dispatcher);
        }
        assert_eq!(
            live.get(),
            counter_body,
            "the probe still retains the real accounting target"
        );
        assert_eq!(Rc::strong_count(&live), if owner_first { 1 } else { 2 });
        drop(owner);
        assert_eq!(Rc::strong_count(&live), 1);
        assert_eq!(live.get(), counter_body);
        drop(live); // The final Rc frees its still-funded counter target.
    }
}

#[test]
fn dispatcher_drop_abandons_pending_host_and_closes_the_original_source_scope() {
    let (mut session, document, dialog, _) = fixture();
    let mut source = session.source_execution();
    let mut owner = dispatcher();
    owner
        .add_listener(
            &mut session.dom(),
            dialog,
            DomEventType::BeforeToggle,
            DomEventCallback::host(41),
            DomEventListenerOptions {
                capture: false,
                once: true,
            },
        )
        .unwrap();
    owner
        .add_listener(
            &mut session.dom(),
            dialog,
            DomEventType::BeforeToggle,
            DomEventCallback::new(|_, _, calls: &mut Vec<u64>, _| {
                calls.push(99);
                Ok(())
            }),
            DomEventListenerOptions::default(),
        )
        .unwrap();
    let request = show(&mut source, &mut session, dialog);
    let mut calls = Vec::new();
    let mut invocation = pending(service(
        &source,
        &mut session,
        &request,
        &mut owner,
        &mut calls,
    ));
    let event = invocation.event().clone();
    let path = event.composed_path();
    let parent = session.snapshot().node(dialog).unwrap().parent().unwrap();
    // The pending host enables nested Source work before owner loss.
    assert!(matches!(
        source
            .apply(
                &mut session.dom(),
                DomSourceEdit::set_attribute(dialog, "data-before-owner-drop", "kept"),
                limits().source,
                &mut services(&mut |_| None),
                &AtomicBool::new(false)
            )
            .unwrap(),
        DomSourceDriveProgress::Complete(_)
    ));
    drop(owner);
    assert_eq!(event.current_target(), None);
    assert_eq!(event.phase(), DomEventPhase::None);
    assert_eq!(event.composed_path().count(), 0);
    assert_eq!(
        path.collect::<Vec<_>>(),
        [dialog.into(), parent.into(), document.into()]
    );
    assert!(
        source
            .apply(
                &mut session.dom(),
                DomSourceEdit::set_attribute(dialog, "data-after-owner-drop", "forbidden"),
                limits().source,
                &mut services(&mut |_| None),
                &AtomicBool::new(false)
            )
            .is_err()
    );
    let snapshot = session.snapshot();
    assert_eq!(
        snapshot
            .node(dialog)
            .unwrap()
            .attribute(&"data-before-owner-drop".into()),
        Some(&"kept".into())
    );
    assert!(
        snapshot
            .node(dialog)
            .unwrap()
            .attribute(&"data-after-owner-drop".into())
            .is_none()
    );
    assert!(
        snapshot
            .node(dialog)
            .unwrap()
            .attribute(&"open".into())
            .is_none()
    );
    assert!(session.dom().pending_source_edit().is_some());
    assert!(calls.is_empty());
    // A different live registry cannot revive the old token or its Source tail.
    let (mut fresh, _, target, _) = fixture();
    let mut fresh_source = fresh.source_execution();
    let mut replacement = dispatcher();
    replacement
        .add_listener(
            &mut fresh.dom(),
            target,
            DomEventType::BeforeToggle,
            DomEventCallback::new(|_, _, calls: &mut Vec<u64>, event| {
                calls.push(7);
                event.prevent_default();
                Ok(())
            }),
            DomEventListenerOptions::default(),
        )
        .unwrap();
    assert!(matches!(
        invocation.complete(&replacement, Ok(())),
        Err(DomEventDispatchError::Abandoned)
    ));
    let fresh_request = show(&mut fresh_source, &mut fresh, target);
    let receipt = match service(
        &fresh_source,
        &mut fresh,
        &fresh_request,
        &mut replacement,
        &mut calls,
    ) {
        DomSourceEventServiceProgress::Serviced { receipt } => receipt,
        other => panic!("fresh authentic event: {other:?}"),
    };
    assert_eq!(receipt.callbacks(), 1);
    assert_eq!(calls, [7]);
    assert!(receipt.default_prevented());
    assert!(receipt.failures().is_empty());
    assert!(matches!(
        source.service_event(
            &mut session.dom(),
            &request,
            &mut replacement,
            &mut calls,
            limits(),
            &AtomicBool::new(false)
        ),
        Err(crate::DomSourceRuntimeError::SourceEvent(
            DomSourceEventRuntimeError::Dispatch(DomEventDispatchError::Dom(
                DomError::ForeignOwner
            ))
        ))
    ));
}
fn services(
    policy: &mut dyn FnMut(
        &surgeist_dom::DomStyleSource,
    ) -> Option<surgeist_dom::DomStyleInlineCspObservation>,
) -> DomSourceServices<'_> {
    DomSourceServices {
        meta: None,
        execution_scope: None,
        style_policy: policy,
        url: None,
    }
}
fn show(
    source: &mut DomSourceRuntime,
    session: &mut DomSession,
    target: NodeId,
) -> DomSourceEventRequest {
    let progress = source
        .apply(
            &mut session.dom(),
            DomSourceEdit::show_dialog(target),
            limits().source,
            &mut services(&mut |_| None),
            &AtomicBool::new(false),
        )
        .unwrap();
    let DomSourceDriveProgress::Pending {
        request: DomSourceRequest::Event(request),
        reason: DomSourceUnavailable::Event,
    } = progress
    else {
        panic!("actual Show reaches its original beforetoggle")
    };
    request
}
fn pending(progress: DomSourceEventServiceProgress) -> DomEventHostInvocation {
    let DomSourceEventServiceProgress::Retained {
        dispatch: DomEventDispatchOutcome::HostInvocation(invocation),
    } = progress
    else {
        panic!("host listener is selected: {progress:?}")
    };
    invocation
}
fn service(
    source: &DomSourceRuntime,
    session: &mut DomSession,
    request: &DomSourceEventRequest,
    dispatcher: &mut DomEventDispatcher<Vec<u64>>,
    calls: &mut Vec<u64>,
) -> DomSourceEventServiceProgress {
    source
        .service_event(
            &mut session.dom(),
            request,
            dispatcher,
            calls,
            limits(),
            &AtomicBool::new(false),
        )
        .unwrap()
}
fn canceled_tail(
    source: &mut DomSourceRuntime,
    session: &mut DomSession,
    request: &DomSourceEventRequest,
) {
    assert!(matches!(
        source
            .resume(
                &mut session.dom(),
                request.operation(),
                limits().source,
                &mut services(&mut |_| None),
                &AtomicBool::new(false),
            )
            .unwrap(),
        DomSourceDriveProgress::Complete(_)
    ));
    assert!(
        session
            .snapshot()
            .node(request.subject())
            .unwrap()
            .attribute(&"open".into())
            .is_none()
    );
}

#[test]
fn owned_host_invocation_releases_session_loan_and_retains_real_nested_source_authority() {
    struct Owned {
        session: DomSession,
        dispatcher: DomEventDispatcher<Vec<u64>>,
        calls: Vec<u64>,
    }
    let (mut session, document, dialog, child) = fixture();
    let mut source = session.source_execution();
    let mut dispatcher = dispatcher();
    let once = dispatcher
        .add_listener(
            &mut session.dom(),
            dialog,
            DomEventType::BeforeToggle,
            DomEventCallback::host(7),
            DomEventListenerOptions {
                once: true,
                capture: false,
            },
        )
        .unwrap();
    dispatcher
        .add_listener(
            &mut session.dom(),
            child,
            DomEventType::BeforeToggle,
            DomEventCallback::host(8),
            DomEventListenerOptions::default(),
        )
        .unwrap();
    dispatcher
        .add_listener(
            &mut session.dom(),
            dialog,
            DomEventType::BeforeToggle,
            DomEventCallback::new(|_, _, calls: &mut Vec<u64>, event| {
                calls.push(9);
                assert!(event.default_prevented());
                Ok(())
            }),
            DomEventListenerOptions::default(),
        )
        .unwrap();
    let request = show(&mut source, &mut session, dialog);
    let owner = RefCell::new(Owned {
        session,
        dispatcher,
        calls: Vec::new(),
    });
    let mut invocation = {
        let mut loan = owner.borrow_mut();
        let Owned {
            session,
            dispatcher,
            calls,
        } = &mut *loan;
        pending(service(&source, session, &request, dispatcher, calls))
    };
    let event = invocation.event().clone();
    assert_eq!(invocation.key(), 7);
    assert_eq!(invocation.registration(), once);
    assert_eq!(event.target(), Some(dialog));
    assert_eq!(event.current_target(), Some(dialog.into()));
    assert_eq!(event.phase(), DomEventPhase::AtTarget);
    assert_eq!(
        event.composed_path().collect::<Vec<_>>().last(),
        Some(&DomEventTarget::Node(document))
    );
    let nested = {
        // This real second RefCell loan is impossible under the old borrowed
        // callback front. Source operations use fresh, short canonical borrows.
        let mut loan = owner.borrow_mut();
        assert!(loan.calls.is_empty());
        assert!(!loan.dispatcher.remove_listener(once));
        assert!(matches!(
            source
                .apply(
                    &mut loan.session.dom(),
                    DomSourceEdit::set_attribute(dialog, "data-host", "nested"),
                    limits().source,
                    &mut services(&mut |_| None),
                    &AtomicBool::new(false),
                )
                .unwrap(),
            DomSourceDriveProgress::Complete(_)
        ));
        assert_eq!(
            loan.session
                .snapshot()
                .node(dialog)
                .unwrap()
                .attribute(&"data-host".into()),
            Some(&"nested".into())
        );
        show(&mut source, &mut loan.session, child)
    };
    {
        let mut loan = owner.borrow_mut();
        let Owned {
            session,
            dispatcher,
            calls,
        } = &mut *loan;
        assert!(
            source
                .service_event(
                    &mut session.dom(),
                    &request,
                    dispatcher,
                    calls,
                    limits(),
                    &AtomicBool::new(false)
                )
                .is_err()
        );
        assert!(
            source
                .resume(
                    &mut session.dom(),
                    request.operation(),
                    limits().source,
                    &mut services(&mut |_| None),
                    &AtomicBool::new(false)
                )
                .is_err()
        );
        assert!(
            calls.is_empty(),
            "neither the parent listener nor its tail overtakes its child"
        );
    }
    let mut nested_invocation = {
        let mut loan = owner.borrow_mut();
        let Owned {
            session,
            dispatcher,
            calls,
        } = &mut *loan;
        pending(service(&source, session, &nested, dispatcher, calls))
    };
    assert_eq!(nested_invocation.key(), 8);
    assert_eq!(event.current_target(), Some(dialog.into()));
    assert_eq!(
        nested_invocation.event().current_target(),
        Some(child.into())
    );
    nested_invocation.event().prevent_default();
    {
        let mut loan = owner.borrow_mut();
        nested_invocation
            .complete(&loan.dispatcher, Ok(()))
            .unwrap();
        let Owned {
            session,
            dispatcher,
            calls,
        } = &mut *loan;
        assert!(matches!(
            service(&source, session, &nested, dispatcher, calls),
            DomSourceEventServiceProgress::Serviced { .. }
        ));
        canceled_tail(&mut source, session, &nested);
        assert!(calls.is_empty());
        assert_eq!(event.current_target(), Some(dialog.into()));
    }
    event.prevent_default();
    {
        let mut loan = owner.borrow_mut();
        invocation.complete(&loan.dispatcher, Ok(())).unwrap();
        assert_eq!(event.current_target(), None);
        assert_eq!(event.phase(), DomEventPhase::None);
        assert_eq!(event.composed_path().count(), 0);
        assert!(
            loan.calls.is_empty(),
            "completion does not run later listeners"
        );
        let Owned {
            session,
            dispatcher,
            calls,
        } = &mut *loan;
        let DomSourceEventServiceProgress::Serviced { receipt } =
            service(&source, session, &request, dispatcher, calls)
        else {
            panic!("real event completes")
        };
        assert_eq!(receipt.callbacks(), 2);
        assert!(receipt.default_prevented());
        assert!(receipt.failures().is_empty());
        assert_eq!(calls, &[9]);
        canceled_tail(&mut source, session, &request);
    }
}

#[test]
fn equal_host_keys_have_distinct_identity_and_foreign_duplicate_completion_cannot_redirect() {
    let (mut session, _, dialog, _) = fixture();
    let mut source = session.source_execution();
    let mut dispatcher = dispatcher();
    let callback = DomEventCallback::host(7);
    let first = dispatcher
        .add_listener(
            &mut session.dom(),
            dialog,
            DomEventType::BeforeToggle,
            callback.clone(),
            DomEventListenerOptions::default(),
        )
        .unwrap();
    assert_eq!(
        dispatcher
            .add_listener(
                &mut session.dom(),
                dialog,
                DomEventType::BeforeToggle,
                callback,
                DomEventListenerOptions::default()
            )
            .unwrap(),
        first
    );
    let second = dispatcher
        .add_listener(
            &mut session.dom(),
            dialog,
            DomEventType::BeforeToggle,
            DomEventCallback::host(7),
            DomEventListenerOptions::default(),
        )
        .unwrap();
    assert_ne!(first, second);
    dispatcher
        .add_listener(
            &mut session.dom(),
            dialog,
            DomEventType::BeforeToggle,
            DomEventCallback::new(|_, _, calls: &mut Vec<u64>, _| {
                calls.push(3);
                Ok(())
            }),
            DomEventListenerOptions::default(),
        )
        .unwrap();
    let request = show(&mut source, &mut session, dialog);
    let mut calls = Vec::new();
    let mut one = pending(service(
        &source,
        &mut session,
        &request,
        &mut dispatcher,
        &mut calls,
    ));
    let foreign: DomEventDispatcher<Vec<u64>> = DomEventDispatcher::new(DomEventRegistryLimits {
        registrations: 16,
        pending_events: 8,
        nested_dispatches: 8,
    })
    .unwrap();
    assert!(matches!(
        one.complete(&foreign, Ok(())),
        Err(DomEventDispatchError::WrongContinuation)
    ));
    assert_eq!(one.event().current_target(), Some(dialog.into()));
    assert!(
        dispatcher.remove_listener(first),
        "self-removal cannot invalidate selected invocation"
    );
    let reused = dispatcher
        .add_listener(
            &mut session.dom(),
            dialog,
            DomEventType::BeforeToggle,
            DomEventCallback::host(99),
            DomEventListenerOptions::default(),
        )
        .unwrap();
    assert_ne!(reused, first);
    one.complete(&dispatcher, Err(DomEventCallbackError { code: 41 }))
        .unwrap();
    let mut two = pending(service(
        &source,
        &mut session,
        &request,
        &mut dispatcher,
        &mut calls,
    ));
    assert_eq!(two.registration(), second);
    assert!(matches!(
        one.complete(&dispatcher, Ok(())),
        Err(DomEventDispatchError::AlreadyCompleted)
    ));
    assert_eq!(two.event().current_target(), Some(dialog.into()));
    two.event().prevent_default();
    two.complete(&dispatcher, Ok(())).unwrap();
    let DomSourceEventServiceProgress::Serviced { receipt } =
        service(&source, &mut session, &request, &mut dispatcher, &mut calls)
    else {
        panic!("later standard listener runs after guest throw")
    };
    assert_eq!(calls, [3]);
    assert_eq!(receipt.callbacks(), 3);
    assert_eq!(receipt.failures().len(), 1);
    assert_eq!(receipt.failures()[0].registration, first);
    assert_eq!(
        receipt.failures()[0].kind,
        DomEventListenerFailureKind::Reported(DomEventCallbackError { code: 41 })
    );
    canceled_tail(&mut source, &mut session, &request);
}

#[test]
fn host_propagation_controls_stop_later_invocations_and_preserve_canceled_source_return() {
    for immediate in [false, true] {
        let (mut session, document, dialog, _) = fixture();
        let mut source = session.source_execution();
        let mut dispatcher = dispatcher();
        dispatcher
            .add_listener(
                &mut session.dom(),
                document,
                DomEventType::BeforeToggle,
                DomEventCallback::host(1),
                DomEventListenerOptions {
                    capture: true,
                    once: false,
                },
            )
            .unwrap();
        dispatcher
            .add_listener(
                &mut session.dom(),
                document,
                DomEventType::BeforeToggle,
                DomEventCallback::new(|_, _, calls: &mut Vec<u64>, _| {
                    calls.push(2);
                    Ok(())
                }),
                DomEventListenerOptions {
                    capture: true,
                    once: false,
                },
            )
            .unwrap();
        dispatcher
            .add_listener(
                &mut session.dom(),
                dialog,
                DomEventType::BeforeToggle,
                DomEventCallback::new(|_, _, calls: &mut Vec<u64>, _| {
                    calls.push(3);
                    Ok(())
                }),
                DomEventListenerOptions::default(),
            )
            .unwrap();
        let request = show(&mut source, &mut session, dialog);
        let mut calls = Vec::new();
        let mut invocation = pending(service(
            &source,
            &mut session,
            &request,
            &mut dispatcher,
            &mut calls,
        ));
        assert_eq!(invocation.event().phase(), DomEventPhase::Capturing);
        if immediate {
            invocation.event().stop_immediate_propagation();
        } else {
            invocation.event().stop_propagation();
        }
        invocation.event().prevent_default();
        invocation.complete(&dispatcher, Ok(())).unwrap();
        let DomSourceEventServiceProgress::Serviced { receipt } =
            service(&source, &mut session, &request, &mut dispatcher, &mut calls)
        else {
            panic!("stopped original event completes")
        };
        assert_eq!(receipt.callbacks(), if immediate { 1 } else { 2 });
        assert_eq!(calls, if immediate { vec![] } else { vec![2] });
        canceled_tail(&mut source, &mut session, &request);
    }
}

#[test]
fn cancellation_preserves_reported_guest_result_and_host_refusal_never_becomes_receipt() {
    for refuse in [false, true] {
        let (mut session, _, dialog, _) = fixture();
        let mut source = session.source_execution();
        let mut dispatcher = dispatcher();
        dispatcher
            .add_listener(
                &mut session.dom(),
                dialog,
                DomEventType::BeforeToggle,
                DomEventCallback::host(5),
                DomEventListenerOptions::default(),
            )
            .unwrap();
        dispatcher
            .add_listener(
                &mut session.dom(),
                dialog,
                DomEventType::BeforeToggle,
                DomEventCallback::new(|_, _, calls: &mut Vec<u64>, _| {
                    calls.push(6);
                    Ok(())
                }),
                DomEventListenerOptions::default(),
            )
            .unwrap();
        let request = show(&mut source, &mut session, dialog);
        let mut calls = Vec::new();
        let mut invocation = pending(service(
            &source,
            &mut session,
            &request,
            &mut dispatcher,
            &mut calls,
        ));
        invocation.event().prevent_default();
        if refuse {
            invocation
                .refuse(&dispatcher, DomEventHostError { code: 77 })
                .unwrap();
            for _ in 0..2 {
                assert!(matches!(
                    source.service_event(
                        &mut session.dom(),
                        &request,
                        &mut dispatcher,
                        &mut calls,
                        limits(),
                        &AtomicBool::new(false)
                    ),
                    Err(crate::DomSourceRuntimeError::SourceEvent(
                        DomSourceEventRuntimeError::Dispatch(DomEventDispatchError::HostRefused(
                            DomEventHostError { code: 77 }
                        ))
                    ))
                ));
            }
            assert!(matches!(
                invocation.complete(&dispatcher, Ok(())),
                Err(DomEventDispatchError::AlreadyCompleted)
            ));
        } else {
            invocation
                .complete(&dispatcher, Err(DomEventCallbackError { code: 88 }))
                .unwrap();
            assert!(
                source
                    .service_event(
                        &mut session.dom(),
                        &request,
                        &mut dispatcher,
                        &mut calls,
                        limits(),
                        &AtomicBool::new(true)
                    )
                    .is_err()
            );
            assert!(calls.is_empty());
            let DomSourceEventServiceProgress::Serviced { receipt } =
                service(&source, &mut session, &request, &mut dispatcher, &mut calls)
            else {
                panic!("same reported result survives cancellation")
            };
            assert_eq!(receipt.failures().len(), 1);
            assert_eq!(
                receipt.failures()[0].kind,
                DomEventListenerFailureKind::Reported(DomEventCallbackError { code: 88 })
            );
            canceled_tail(&mut source, &mut session, &request);
            assert_eq!(calls, [6]);
        }
        if refuse {
            assert!(calls.is_empty());
        }
    }
}

#[test]
fn abandonment_and_session_drop_close_active_observation_without_successful_source_tail() {
    for abandon in [false, true] {
        let (mut session, document, dialog, _) = fixture();
        let mut source = session.source_execution();
        let mut dispatcher = DomEventDispatcher::new(DomEventRegistryLimits {
            registrations: 16,
            pending_events: 1,
            nested_dispatches: 1,
        })
        .unwrap();
        dispatcher
            .add_listener(
                &mut session.dom(),
                dialog,
                DomEventType::BeforeToggle,
                DomEventCallback::host(1),
                DomEventListenerOptions::default(),
            )
            .unwrap();
        let request = show(&mut source, &mut session, dialog);
        let mut calls = Vec::new();
        let mut invocation = pending(service(
            &source,
            &mut session,
            &request,
            &mut dispatcher,
            &mut calls,
        ));
        let event = invocation.event().clone();
        let historical_path = event.composed_path();
        let parent = session.snapshot().node(dialog).unwrap().parent().unwrap();
        let expected_path = vec![dialog.into(), parent.into(), document.into()];
        let (mut next, _, target, _) = fixture();
        let mut next_source = next.source_execution();
        let next_request = show(&mut next_source, &mut next, target);
        assert!(
            matches!(
                next_source.service_event(
                    &mut next.dom(),
                    &next_request,
                    &mut dispatcher,
                    &mut calls,
                    limits(),
                    &AtomicBool::new(false)
                ),
                Err(crate::DomSourceRuntimeError::SourceEvent(
                    DomSourceEventRuntimeError::Dispatch(DomEventDispatchError::Dom(
                        DomError::ForeignOwner
                    ))
                ))
            ),
            "a bound dispatcher cannot execute against a different canonical owner"
        );
        assert_eq!(event.current_target(), Some(dialog.into()));
        let DomShutdownOutcome::Pending {
            mut session,
            reason,
        } = session.try_shutdown()
        else {
            panic!("an uncompleted host invocation prevents graceful shutdown")
        };
        assert_eq!(reason, DomShutdownReason::SourceTail(request.operation()));
        if abandon {
            drop(invocation);
            assert_eq!(event.current_target(), None);
            assert!(
                source
                    .service_event(
                        &mut session.dom(),
                        &request,
                        &mut dispatcher,
                        &mut calls,
                        limits(),
                        &AtomicBool::new(false)
                    )
                    .is_err()
            );
            assert!(session.dom().pending_source_edit().is_some());
            assert!(
                source
                    .apply(
                        &mut session.dom(),
                        DomSourceEdit::set_attribute(dialog, "data-after-abandon", "forbidden"),
                        limits().source,
                        &mut services(&mut |_| None),
                        &AtomicBool::new(false)
                    )
                    .is_err()
            );
            assert!(
                session
                    .snapshot()
                    .node(dialog)
                    .unwrap()
                    .attribute(&"data-after-abandon".into())
                    .is_none(),
                "abandonment closed the authentic Source reentry scope"
            );
            assert!(
                session
                    .snapshot()
                    .node(dialog)
                    .unwrap()
                    .attribute(&"open".into())
                    .is_none()
            );
        } else {
            drop(session);
            assert!(matches!(
                invocation.complete(&dispatcher, Ok(())),
                Err(DomEventDispatchError::Abandoned)
            ));
        }
        assert_eq!(event.current_target(), None);
        assert_eq!(event.phase(), DomEventPhase::None);
        assert_eq!(event.composed_path().count(), 0);
        assert!(calls.is_empty());
        // A different canonical owner requires its own fresh registry. The old
        // issuer remains bound even after its event or Session is abandoned.
        let mut next_dispatcher = DomEventDispatcher::new(DomEventRegistryLimits {
            registrations: 16,
            pending_events: 1,
            nested_dispatches: 1,
        })
        .unwrap();
        assert!(matches!(
            service(
                &next_source,
                &mut next,
                &next_request,
                &mut next_dispatcher,
                &mut calls
            ),
            DomSourceEventServiceProgress::Serviced { .. }
        ));
        assert_eq!(
            historical_path.collect::<Vec<_>>(),
            expected_path,
            "an independently held frozen path stays readable after terminal cleanup and reservation reuse"
        );
    }
}

#[test]
fn one_short_callback_or_failure_admission_preserves_once_until_exact_host_selection() {
    for resource in [
        crate::DomEventResource::Callbacks,
        crate::DomEventResource::FailureRecords,
    ] {
        let (mut session, _, dialog, _) = fixture();
        let mut source = session.source_execution();
        let mut dispatcher = dispatcher();
        let once = dispatcher
            .add_listener(
                &mut session.dom(),
                dialog,
                DomEventType::BeforeToggle,
                DomEventCallback::host(3),
                DomEventListenerOptions {
                    capture: false,
                    once: true,
                },
            )
            .unwrap();
        let request = show(&mut source, &mut session, dialog);
        let mut calls = Vec::new();
        let mut exact = limits();
        exact.dispatch.path_items = 3;
        exact.dispatch.snapshot_listeners = 1;
        exact.dispatch.callbacks = 1;
        exact.dispatch.failure_records = 1;
        let mut short = exact;
        match resource {
            crate::DomEventResource::Callbacks => short.dispatch.callbacks = 0,
            crate::DomEventResource::FailureRecords => short.dispatch.failure_records = 0,
            _ => unreachable!(),
        }
        assert!(
            matches!(source.service_event(&mut session.dom(), &request, &mut dispatcher,
            &mut calls, short, &AtomicBool::new(false)).unwrap(),
            DomSourceEventServiceProgress::Retained { dispatch: DomEventDispatchOutcome::Retained(actual) }
            if actual == resource)
        );
        assert!(calls.is_empty());
        let mut invocation = pending(
            source
                .service_event(
                    &mut session.dom(),
                    &request,
                    &mut dispatcher,
                    &mut calls,
                    exact,
                    &AtomicBool::new(false),
                )
                .unwrap(),
        );
        assert_eq!(invocation.registration(), once);
        assert!(
            !dispatcher.remove_listener(once),
            "once removal occurs only at actual admitted selection"
        );
        invocation.event().prevent_default();
        invocation
            .complete(&dispatcher, Err(DomEventCallbackError { code: 9 }))
            .unwrap();
        let DomSourceEventServiceProgress::Serviced { receipt } = source
            .service_event(
                &mut session.dom(),
                &request,
                &mut dispatcher,
                &mut calls,
                exact,
                &AtomicBool::new(false),
            )
            .unwrap()
        else {
            panic!("exact callback and failure capacity fits actual host delivery")
        };
        assert_eq!(receipt.callbacks(), 1);
        assert_eq!(receipt.failures().len(), 1);
        canceled_tail(&mut source, &mut session, &request);
    }
}

#[test]
fn uncanceled_host_delivery_defers_show_publication_until_original_source_tail_is_resumed() {
    let (mut session, _, dialog, _) = fixture();
    let mut source = session.source_execution();
    let mut dispatcher = dispatcher();
    dispatcher
        .add_listener(
            &mut session.dom(),
            dialog,
            DomEventType::BeforeToggle,
            DomEventCallback::host(1),
            DomEventListenerOptions::default(),
        )
        .unwrap();
    let request = show(&mut source, &mut session, dialog);
    let mut calls = Vec::new();
    let mut invocation = pending(service(
        &source,
        &mut session,
        &request,
        &mut dispatcher,
        &mut calls,
    ));
    assert!(
        session
            .snapshot()
            .node(dialog)
            .unwrap()
            .attribute(&"open".into())
            .is_none()
    );
    invocation.complete(&dispatcher, Ok(())).unwrap();
    let DomSourceEventServiceProgress::Serviced { receipt } =
        service(&source, &mut session, &request, &mut dispatcher, &mut calls)
    else {
        panic!("original beforetoggle really completed")
    };
    assert!(!receipt.default_prevented());
    assert!(
        session
            .snapshot()
            .node(dialog)
            .unwrap()
            .attribute(&"open".into())
            .is_none(),
        "neither host completion nor event servicing runs the Show tail"
    );
    let progress = source
        .resume(
            &mut session.dom(),
            request.operation(),
            limits().source,
            &mut services(&mut |_| None),
            &AtomicBool::new(false),
        )
        .unwrap();
    assert!(matches!(
        progress,
        DomSourceDriveProgress::Pending {
            request: DomSourceRequest::Dialog(_),
            reason: DomSourceUnavailable::Dialog(
                surgeist_html::elements::dialog::HtmlDialogAction::RememberFocusedElement
            ),
        }
    ));
    assert_eq!(
        session
            .snapshot()
            .node(dialog)
            .unwrap()
            .attribute(&"open".into()),
        Some(&"".into())
    );
    assert!(
        session.dom().pending_source_edit().is_some(),
        "actual unavailable focus service retains its mandatory tail"
    );
}
