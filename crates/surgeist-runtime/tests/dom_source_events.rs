//! The original Session dispatches actual Source beforetoggle requests. These
//! native no-global fixtures test the event bridge, not modal or hosted Show.
use std::sync::{
    Arc,
    atomic::{AtomicBool, Ordering},
};
use surgeist_dom::*;
use surgeist_html::{HtmlDialogAction, HtmlDialogState};
use surgeist_runtime::*;

fn source_limits() -> DomSourceLimits {
    DomSourceLimits::new(32_000_000, 1_000_000, 10_000_000, 128, 4096, 4096, 4)
}
fn dispatch_limits() -> DomEventDispatchLimits {
    DomEventDispatchLimits {
        path_items: 32,
        snapshot_listeners: 32,
        failure_records: 32,
        callbacks: 32,
        work: 100_000,
    }
}
fn service_limits() -> DomSourceEventServiceLimits {
    DomSourceEventServiceLimits {
        source: source_limits(),
        dispatch: dispatch_limits(),
    }
}
fn session_limits() -> DomSessionLimits {
    DomSessionLimits {
        source_events: DomSourceEventRuntimeLimits {
            jobs: 4,
            storage_bytes: 16_000_000,
            work: 1_000_000,
        },
        style: DomStyleRuntimeLimits {
            roots: 4,
            completions: 8,
            reports: 8,
            storage_bytes: 16_000_000,
            work: 1_000_000,
            css_input: surgeist_css::CssUtf16InputLimits::default(),
        },
        details: DomDetailsRuntimeLimits {
            tasks: 8,
            storage_bytes: 16_000_000,
            work: 1_000_000,
        },
        hosted: DomHostedRuntimeLimits {
            canonical: DomHostedLimits::new(
                DomHostedRecordLimits::new(8, 8, 8, 8, 8, 8, 8),
                16_000_000,
                16_000_000,
                1_000_000,
            ),
            tasks: 8,
            storage_bytes: 16_000_000,
            scratch_bytes: 16_000_000,
            work: 1_000_000,
        },
    }
}
fn fixture(limits: DomSessionLimits) -> (DomSession, NodeId) {
    let (session, first, _) = fixture_pair(limits);
    (session, first)
}
fn fixture_pair(limits: DomSessionLimits) -> (DomSession, NodeId, NodeId) {
    let cancel = AtomicBool::new(false);
    let mut dom = Dom::new(
        DomLimits::new(128, 100_000, 256, 128, 128, 32, 4_000_000)
            .unwrap()
            .with_dispatch_custody(
                DomDispatchCustodyLimits::new(64, 4096, 16_000_000, 16_000_000, 4_000_000).unwrap(),
            ),
    )
    .unwrap();
    let mut batch = DomBatch::new();
    let document = batch.create_document();
    let dialog = batch.create_element_ns(&document, Namespace::html(), "dialog");
    let second = batch.create_element_ns(&document, Namespace::html(), "dialog");
    batch.insert_before(&dialog, &document, None);
    let done = dom.apply(batch, &cancel).unwrap();
    let dialog = done.created(&dialog).unwrap().unwrap();
    let second = done.created(&second).unwrap().unwrap();
    (DomSession::new(dom, limits).unwrap(), dialog, second)
}
fn request(session: &mut DomSession, subject: NodeId) -> DomSourceEventRequest {
    let mut source = session.source_execution();
    let mut policy = |_: &DomStyleSource| None;
    let progress = source
        .apply(
            &mut session.dom(),
            DomSourceEdit::show_dialog(subject),
            source_limits(),
            &mut DomSourceServices {
                style_policy: &mut policy,
                url: None,
            },
            &AtomicBool::new(false),
        )
        .unwrap();
    match progress {
        DomSourceDriveProgress::Pending {
            request: DomSourceRequest::Event(request),
            reason: DomSourceUnavailable::Event,
        } => request,
        other => panic!("actual beforetoggle required: {other:?}"),
    }
}
fn resume(session: &mut DomSession, operation: DomSourceOperationId) -> DomSourceDriveProgress {
    let mut source = session.source_execution();
    let mut policy = |_: &DomStyleSource| None;
    source
        .resume(
            &mut session.dom(),
            operation,
            source_limits(),
            &mut DomSourceServices {
                style_policy: &mut policy,
                url: None,
            },
            &AtomicBool::new(false),
        )
        .unwrap()
}
fn dispatcher<C>() -> DomEventDispatcher<C> {
    DomEventDispatcher::new(DomEventRegistryLimits {
        registrations: 8,
        pending_events: 4,
        nested_dispatches: 4,
    })
    .unwrap()
}
fn serviced(progress: DomSourceEventServiceProgress) -> DomEventDispatchReceipt {
    match progress {
        DomSourceEventServiceProgress::Serviced { receipt } => receipt,
        other => panic!("actual completed receipt required: {other:?}"),
    }
}

#[test]
fn canceled_receipt_is_actual_and_distinct_from_callback_failure_without_running_tail() {
    let cancel = AtomicBool::new(false);
    let (mut session, subject) = fixture(session_limits());
    let mut listeners = dispatcher();
    listeners
        .add_listener(
            &session.dom(),
            subject,
            DomEventType::BeforeToggle,
            DomEventCallback::new(|_, _, observed: &mut (usize, bool), event| {
                observed.0 += 1;
                observed.1 = event.cancelable();
                event.prevent_default();
                Err(DomEventCallbackError { code: 91 })
            }),
            DomEventListenerOptions {
                capture: false,
                once: false,
            },
        )
        .unwrap();
    let before = session.snapshot().stamp();
    let request = request(&mut session, subject);
    assert_eq!(request.old_state(), HtmlDialogState::Closed);
    assert_eq!(request.new_state(), HtmlDialogState::Open);
    assert_eq!(request.source(), None);
    assert_eq!(session.snapshot().stamp(), before);
    let mut observed = (0, false);
    let permission = session.source_execution();
    let receipt = serviced(
        permission
            .service_event(
                &mut session.dom(),
                &request,
                &mut listeners,
                &mut observed,
                service_limits(),
                &cancel,
            )
            .unwrap(),
    );
    drop(permission);
    assert!(receipt.default_prevented());
    assert_eq!(receipt.failures().len(), 1);
    assert_eq!(
        receipt.failures()[0].kind,
        DomEventListenerFailureKind::Reported(DomEventCallbackError { code: 91 })
    );
    assert_eq!(receipt.callbacks(), 1);
    assert_eq!(observed, (1, true));
    assert_eq!(receipt.original_target(), subject);
    assert_eq!(receipt.event_type(), DomEventType::BeforeToggle);
    assert_eq!(session.snapshot().stamp(), before);
    let DomSourceDriveProgress::Complete(done) = resume(&mut session, request.operation()) else {
        panic!("canceled beforetoggle returns normally");
    };
    assert!(!done.cancellation_requested());
    assert_eq!(
        session
            .snapshot()
            .node(subject)
            .unwrap()
            .attribute(&"open".into()),
        None
    );
    assert!(
        session
            .source_execution()
            .service_event(
                &mut session.dom(),
                &request,
                &mut listeners,
                &mut observed,
                service_limits(),
                &cancel
            )
            .is_err()
    );
    assert_eq!(observed, (1, true));
}

#[test]
fn uncanceled_resume_publishes_toggle_tracker_then_parks_at_add_open_without_replaying_callback() {
    let cancel = AtomicBool::new(false);
    let (mut session, subject) = fixture(session_limits());
    let mut listeners = dispatcher();
    listeners
        .add_listener(
            &session.dom(),
            subject,
            DomEventType::BeforeToggle,
            DomEventCallback::new(move |dom, _, calls: &mut usize, _| {
                *calls += 1;
                let mut batch = DomBatch::new();
                batch.set_attribute(subject, "data-callback", "kept");
                dom.apply(batch, &AtomicBool::new(false)).unwrap();
                Ok(())
            }),
            DomEventListenerOptions {
                capture: false,
                once: false,
            },
        )
        .unwrap();
    let request = request(&mut session, subject);
    // HTML877d Show queues closed -> open with a null source only after the
    // authentic uncanceled beforetoggle and its current open recheck.
    assert_eq!(request.old_state(), HtmlDialogState::Closed);
    assert_eq!(request.new_state(), HtmlDialogState::Open);
    assert_eq!(request.source(), None);
    let mut calls = 0;
    let receipt = serviced(
        session
            .source_execution()
            .service_event(
                &mut session.dom(),
                &request,
                &mut listeners,
                &mut calls,
                service_limits(),
                &cancel,
            )
            .unwrap(),
    );
    assert!(!receipt.default_prevented());
    let callback_snapshot = session.snapshot();
    let callback_stamp = callback_snapshot.stamp();
    assert!(
        !callback_snapshot
            .node(subject)
            .unwrap()
            .as_dialog()
            .unwrap()
            .has_toggle_task_tracker()
    );
    match resume(&mut session, request.operation()) {
        DomSourceDriveProgress::Pending {
            request: DomSourceRequest::Dialog(dialog),
            reason: DomSourceUnavailable::Dialog(action),
        } => {
            assert_eq!(dialog.operation(), request.operation());
            assert_eq!(dialog.subject(), subject);
            assert_eq!(dialog.action(), action);
            // QueueToggle stores the real task/tracker before acknowledgment;
            // the producer's next action is the still-unserviced AddOpen.
            assert_eq!(action, HtmlDialogAction::AddOpen);
        }
        other => panic!("queued toggle must retain the original Show at AddOpen: {other:?}"),
    }
    assert_eq!(calls, 1);
    let snapshot = session.snapshot();
    assert_ne!(snapshot.stamp(), callback_stamp);
    assert!(
        snapshot
            .node(subject)
            .unwrap()
            .as_dialog()
            .unwrap()
            .has_toggle_task_tracker()
    );
    assert_eq!(
        snapshot.incomplete_source_operation(),
        Some(request.operation())
    );
    assert_eq!(
        snapshot
            .node(subject)
            .unwrap()
            .attribute(&"data-callback".into()),
        Some(&"kept".into())
    );
    assert_eq!(
        snapshot.node(subject).unwrap().attribute(&"open".into()),
        None
    );
}

#[test]
fn retained_dispatch_survives_permission_replacement_and_never_repeats_listeners() {
    let cancel = AtomicBool::new(false);
    let (mut session, subject) = fixture(session_limits());
    let mut listeners = dispatcher();
    for number in [1, 2] {
        listeners
            .add_listener(
                &session.dom(),
                subject,
                DomEventType::BeforeToggle,
                DomEventCallback::new(move |_, _, calls: &mut Vec<usize>, event| {
                    calls.push(number);
                    event.prevent_default();
                    Ok(())
                }),
                DomEventListenerOptions {
                    capture: false,
                    once: false,
                },
            )
            .unwrap();
    }
    let request = request(&mut session, subject);
    let mut calls = Vec::new();
    let permission = session.source_execution();
    let limits = DomSourceEventServiceLimits {
        dispatch: DomEventDispatchLimits {
            callbacks: 1,
            ..dispatch_limits()
        },
        ..service_limits()
    };
    assert!(matches!(
        permission
            .service_event(
                &mut session.dom(),
                &request,
                &mut listeners,
                &mut calls,
                limits,
                &cancel
            )
            .unwrap(),
        DomSourceEventServiceProgress::Retained { .. }
    ));
    assert_eq!(calls, vec![1]);
    drop(permission);
    let receipt = serviced(
        session
            .source_execution()
            .service_event(
                &mut session.dom(),
                &request,
                &mut listeners,
                &mut calls,
                service_limits(),
                &cancel,
            )
            .unwrap(),
    );
    assert_eq!(calls, vec![1, 2]);
    assert_eq!(receipt.callbacks(), 2);
    assert!(receipt.default_prevented());
    assert!(matches!(
        resume(&mut session, request.operation()),
        DomSourceDriveProgress::Complete(_)
    ));
}

#[test]
fn actual_completed_receipt_survives_canceled_acceptance_without_listener_replay() {
    let cancel = Arc::new(AtomicBool::new(false));
    let (mut session, subject) = fixture(session_limits());
    let mut listeners = dispatcher();
    listeners
        .add_listener(
            &session.dom(),
            subject,
            DomEventType::BeforeToggle,
            DomEventCallback::new(|_, _, context: &mut (usize, Arc<AtomicBool>), event| {
                context.0 += 1;
                event.prevent_default();
                context.1.store(true, Ordering::Relaxed);
                Ok(())
            }),
            DomEventListenerOptions {
                capture: false,
                once: false,
            },
        )
        .unwrap();
    let request = request(&mut session, subject);
    let mut context = (0, cancel.clone());
    let permission = session.source_execution();
    assert!(matches!(
        permission.service_event(
            &mut session.dom(),
            &request,
            &mut listeners,
            &mut context,
            service_limits(),
            &cancel
        ),
        Err(DomSourceRuntimeError::SourceEvent(
            DomSourceEventRuntimeError::Dom(DomError::Cancelled)
        ))
    ));
    assert_eq!(context.0, 1);
    drop(permission);
    cancel.store(false, Ordering::Relaxed);
    let receipt = serviced(
        session
            .source_execution()
            .service_event(
                &mut session.dom(),
                &request,
                &mut listeners,
                &mut context,
                service_limits(),
                &cancel,
            )
            .unwrap(),
    );
    assert_eq!(context.0, 1);
    assert_eq!(receipt.callbacks(), 1);
    assert!(matches!(
        resume(&mut session, request.operation()),
        DomSourceDriveProgress::Complete(_)
    ));
}

#[test]
fn explicit_job_zero_refuses_before_dispatch_and_constructor_work_refusal_returns_unbound_dom() {
    let mut limits = session_limits();
    limits.source_events.jobs = 0;
    let (mut session, subject) = fixture(limits);
    let request = request(&mut session, subject);
    let mut listeners = dispatcher::<()>();
    assert!(matches!(
        session.source_execution().service_event(
            &mut session.dom(),
            &request,
            &mut listeners,
            &mut (),
            service_limits(),
            &AtomicBool::new(false)
        ),
        Err(DomSourceRuntimeError::SourceEvent(
            DomSourceEventRuntimeError::Admission(DomSourceEventResource::Jobs)
        ))
    ));
    let dom = Dom::new(DomLimits::new(32, 4096, 64, 0, 0, 8, 100_000).unwrap()).unwrap();
    let before = dom.snapshot().stamp();
    limits.source_events.work = 0;
    let rejected = DomSession::new(dom, limits).unwrap_err();
    assert!(matches!(
        rejected.error(),
        DomSessionCreateFailure::SourceEvents(DomSourceEventRuntimeError::Admission(
            DomSourceEventResource::Work
        ))
    ));
    let dom = rejected.into_dom();
    assert_eq!(dom.snapshot().stamp(), before);
    assert!(!dom.has_source_event_executor());
}

#[test]
fn callback_open_is_observed_by_separate_resume_without_queueing_toggle() {
    let cancel = AtomicBool::new(false);
    let (mut session, subject) = fixture(session_limits());
    let mut listeners = dispatcher();
    listeners
        .add_listener(
            &session.dom(),
            subject,
            DomEventType::BeforeToggle,
            DomEventCallback::new(move |dom, _, source: &mut DomSourceRuntime, _| {
                let mut policy = |_: &DomStyleSource| None;
                assert!(matches!(
                    source
                        .apply(
                            dom,
                            DomSourceEdit::set_attribute(subject, "open", ""),
                            source_limits(),
                            &mut DomSourceServices {
                                style_policy: &mut policy,
                                url: None
                            },
                            &AtomicBool::new(false)
                        )
                        .unwrap(),
                    DomSourceDriveProgress::Complete(_)
                ));
                Ok(())
            }),
            DomEventListenerOptions {
                capture: false,
                once: false,
            },
        )
        .unwrap();
    let request = request(&mut session, subject);
    let mut callback_permission = session.source_execution();
    let receipt = serviced(
        session
            .source_execution()
            .service_event(
                &mut session.dom(),
                &request,
                &mut listeners,
                &mut callback_permission,
                service_limits(),
                &cancel,
            )
            .unwrap(),
    );
    assert!(!receipt.default_prevented());
    assert_eq!(receipt.callbacks(), 1);
    assert!(matches!(
        resume(&mut session, request.operation()),
        DomSourceDriveProgress::Complete(_)
    ));
    assert_eq!(
        session
            .snapshot()
            .node(subject)
            .unwrap()
            .attribute(&"open".into()),
        Some(&"".into())
    );
}

struct Nested {
    permission: Option<DomSourceRuntime>,
    child: Option<DomSourceEventRequest>,
    log: Vec<&'static str>,
}
#[test]
fn pending_nested_request_parks_original_dispatch_and_resumes_without_replay() {
    let cancel = AtomicBool::new(false);
    let (mut session, first, second) = fixture_pair(session_limits());
    let mut listeners = dispatcher();
    listeners
        .add_listener(
            &session.dom(),
            first,
            DomEventType::BeforeToggle,
            DomEventCallback::new(move |dom, _, context: &mut Nested, event| {
                context.log.push("outer");
                event.prevent_default();
                let mut source = context.permission.take().unwrap();
                let mut policy = |_: &DomStyleSource| None;
                let progress = source
                    .apply(
                        dom,
                        DomSourceEdit::show_dialog(second),
                        source_limits(),
                        &mut DomSourceServices {
                            style_policy: &mut policy,
                            url: None,
                        },
                        &AtomicBool::new(false),
                    )
                    .unwrap();
                let DomSourceDriveProgress::Pending {
                    request: DomSourceRequest::Event(child),
                    reason: DomSourceUnavailable::Event,
                } = progress
                else {
                    panic!("actual nested event");
                };
                context.child = Some(child);
                context.permission = Some(source);
                Ok(())
            }),
            DomEventListenerOptions {
                capture: false,
                once: false,
            },
        )
        .unwrap();
    listeners
        .add_listener(
            &session.dom(),
            second,
            DomEventType::BeforeToggle,
            DomEventCallback::new(|_, _, context: &mut Nested, event| {
                context.log.push("child");
                event.prevent_default();
                Ok(())
            }),
            DomEventListenerOptions {
                capture: false,
                once: false,
            },
        )
        .unwrap();
    let outer = request(&mut session, first);
    let mut context = Nested {
        permission: Some(session.source_execution()),
        child: None,
        log: Vec::new(),
    };
    assert!(matches!(
        session
            .source_execution()
            .service_event(
                &mut session.dom(),
                &outer,
                &mut listeners,
                &mut context,
                service_limits(),
                &cancel
            )
            .unwrap(),
        DomSourceEventServiceProgress::Retained {
            dispatch: DomEventDispatchOutcome::RetainedSourceOperation(_)
        }
    ));
    assert_eq!(context.log, vec!["outer"]);
    let child = context.child.take().unwrap();
    assert_eq!(child.parent_operation(), Some(outer.operation()));
    serviced(
        session
            .source_execution()
            .service_event(
                &mut session.dom(),
                &child,
                &mut listeners,
                &mut context,
                service_limits(),
                &cancel,
            )
            .unwrap(),
    );
    assert!(matches!(
        resume(&mut session, child.operation()),
        DomSourceDriveProgress::Complete(_)
    ));
    let receipt = serviced(
        session
            .source_execution()
            .service_event(
                &mut session.dom(),
                &outer,
                &mut listeners,
                &mut context,
                service_limits(),
                &cancel,
            )
            .unwrap(),
    );
    assert_eq!(receipt.callbacks(), 1);
    assert_eq!(context.log, vec!["outer", "child"]);
    assert!(matches!(
        resume(&mut session, outer.operation()),
        DomSourceDriveProgress::Complete(_)
    ));
}

struct ReceiverRetirement {
    cancel_event: bool,
    calls: usize,
    destruction: Option<Result<(), DomError>>,
}
fn receiver_destroyer(subject: NodeId) -> DomEventCallback<ReceiverRetirement> {
    DomEventCallback::new(move |dom, _, context: &mut ReceiverRetirement, event| {
        context.calls += 1;
        let mut batch = DomBatch::new();
        batch.detach(subject);
        batch.destroy(subject);
        context.destruction = Some(dom.apply(batch, &AtomicBool::new(false)).map(|_| ()));
        if context.cancel_event {
            event.prevent_default();
        }
        Ok(())
    })
}

#[test]
fn canceled_beforetoggle_completes_normally_after_actual_receiver_destruction() {
    let cancel = AtomicBool::new(false);
    let (mut session, subject) = fixture(session_limits());
    let mut listeners = dispatcher();
    listeners
        .add_listener(
            &session.dom(),
            subject,
            DomEventType::BeforeToggle,
            receiver_destroyer(subject),
            DomEventListenerOptions {
                capture: false,
                once: false,
            },
        )
        .unwrap();
    let request = request(&mut session, subject);
    let mut context = ReceiverRetirement {
        cancel_event: true,
        calls: 0,
        destruction: None,
    };
    let receipt = serviced(
        session
            .source_execution()
            .service_event(
                &mut session.dom(),
                &request,
                &mut listeners,
                &mut context,
                service_limits(),
                &cancel,
            )
            .unwrap(),
    );
    assert_eq!(context.destruction, Some(Ok(())));
    assert_eq!(context.calls, 1);
    assert!(session.snapshot().node(subject).is_err());
    assert_eq!(receipt.callbacks(), 1);
    assert!(receipt.default_prevented());
    assert!(receipt.failures().is_empty());
    let DomSourceDriveProgress::Complete(done) = resume(&mut session, request.operation()) else {
        panic!("canceled beforetoggle returns before the retired receiver read");
    };
    assert_eq!(done.operation(), request.operation());
    assert!(!done.cancellation_requested());
    assert!(matches!(
        session.try_shutdown(),
        DomShutdownOutcome::Complete
    ));
}

#[test]
fn uncanceled_beforetoggle_reports_precise_retired_receiver_after_actual_receipt_acceptance() {
    let cancel = AtomicBool::new(false);
    let (mut session, subject) = fixture(session_limits());
    let mut listeners = dispatcher();
    listeners
        .add_listener(
            &session.dom(),
            subject,
            DomEventType::BeforeToggle,
            receiver_destroyer(subject),
            DomEventListenerOptions {
                capture: false,
                once: false,
            },
        )
        .unwrap();
    let request = request(&mut session, subject);
    let mut context = ReceiverRetirement {
        cancel_event: false,
        calls: 0,
        destruction: None,
    };
    let receipt = serviced(
        session
            .source_execution()
            .service_event(
                &mut session.dom(),
                &request,
                &mut listeners,
                &mut context,
                service_limits(),
                &cancel,
            )
            .unwrap(),
    );
    assert_eq!(context.destruction, Some(Ok(())));
    assert_eq!(context.calls, 1);
    assert!(session.snapshot().node(subject).is_err());
    assert_eq!(receipt.callbacks(), 1);
    assert!(!receipt.default_prevented());
    assert!(receipt.failures().is_empty());
    let mut source = session.source_execution();
    let mut policy = |_: &DomStyleSource| None;
    let error = source
        .resume(
            &mut session.dom(),
            request.operation(),
            source_limits(),
            &mut DomSourceServices {
                style_policy: &mut policy,
                url: None,
            },
            &cancel,
        )
        .unwrap_err();
    assert!(
        matches!(error, DomSourceRuntimeError::Dom(DomError::SourceReceiverRetired {
        operation, subject: retired, action: HtmlDialogAction::ReadOpen,
    }) if operation == request.operation() && retired == subject)
    );
    // The public whole-owner completion check proves neither the Source tail
    // nor its actual event job remains parked after the terminal retirement.
    assert!(matches!(
        session.try_shutdown(),
        DomShutdownOutcome::Complete
    ));
}

struct AncestorRecovery {
    permission: DomSourceRuntime,
    child_progress: Option<Result<DomSourceDriveProgress, DomSourceRuntimeError>>,
    calls: Vec<&'static str>,
    edits: usize,
    refusal: Option<DomError>,
    refused_edit_unchanged: Option<bool>,
}

#[test]
fn replacement_permission_raises_exhausted_ancestor_without_replaying_retained_child() {
    const OUTER_CAP: u64 = 100_000;
    let cancel = AtomicBool::new(false);
    let (mut session, first, second) = fixture_pair(session_limits());
    let mut listeners = dispatcher();
    listeners
        .add_listener(
            &session.dom(),
            first,
            DomEventType::BeforeToggle,
            DomEventCallback::new(move |dom, _, context: &mut AncestorRecovery, event| {
                context.calls.push("outer");
                event.prevent_default();
                let mut policy = |_: &DomStyleSource| None;
                context.child_progress = Some(context.permission.apply(
                    dom,
                    DomSourceEdit::show_dialog(second),
                    source_limits(),
                    &mut DomSourceServices {
                        style_policy: &mut policy,
                        url: None,
                    },
                    &AtomicBool::new(false),
                ));
                Ok(())
            }),
            DomEventListenerOptions {
                capture: false,
                once: false,
            },
        )
        .unwrap();
    listeners
        .add_listener(
            &session.dom(),
            second,
            DomEventType::BeforeToggle,
            DomEventCallback::new(move |dom, _, context: &mut AncestorRecovery, event| {
                context.calls.push("child");
                event.prevent_default();
                // Every iteration requests a real distinct ordinary edit. The
                // finite sequence exceeds the parent's allowance independently
                // of private per-operation accounting or record layout.
                for index in 0..512 {
                    let mut batch = DomBatch::new();
                    batch.set_attribute(
                        second,
                        "data-parent-work",
                        format!("{index:04}{}", "x".repeat(256)),
                    );
                    let before = dom.snapshot().stamp();
                    match dom.apply(batch, &AtomicBool::new(false)) {
                        Ok(_) => context.edits += 1,
                        Err(error) => {
                            context.refused_edit_unchanged = Some(dom.snapshot().stamp() == before);
                            context.refusal = Some(error);
                            break;
                        }
                    }
                }
                Ok(())
            }),
            DomEventListenerOptions {
                capture: false,
                once: false,
            },
        )
        .unwrap();
    let outer = request(&mut session, first);
    let mut context = AncestorRecovery {
        permission: session.source_execution(),
        child_progress: None,
        calls: Vec::new(),
        edits: 0,
        refusal: None,
        refused_edit_unchanged: None,
    };
    let ordinary = source_limits();
    let tight = DomSourceLimits::new(
        ordinary.retained_bytes(),
        ordinary.retained_string_code_units(),
        OUTER_CAP,
        ordinary.phase_publications(),
        ordinary.output_records(),
        ordinary.published_native_identities(),
        ordinary.nested_operations(),
    );
    let permission = session.source_execution();
    let outer_progress = permission
        .service_event(
            &mut session.dom(),
            &outer,
            &mut listeners,
            &mut context,
            DomSourceEventServiceLimits {
                source: tight,
                ..service_limits()
            },
            &cancel,
        )
        .unwrap();
    assert!(
        matches!(
            outer_progress,
            DomSourceEventServiceProgress::Retained {
                dispatch: DomEventDispatchOutcome::RetainedSourceOperation(_)
            }
        ),
        "actual outer {outer_progress:?}; child formation {:?}",
        context.child_progress
    );
    let child_progress = context
        .child_progress
        .take()
        .expect("actual callback formation attempt")
        .expect("nested setup must fit the outer allowance");
    let DomSourceDriveProgress::Pending {
        request: DomSourceRequest::Event(child),
        reason: DomSourceUnavailable::Event,
    } = child_progress
    else {
        panic!("actual child event required: {child_progress:?}");
    };
    assert_eq!(child.parent_operation(), Some(outer.operation()));
    assert!(matches!(
        permission.service_event(
            &mut session.dom(),
            &child,
            &mut listeners,
            &mut context,
            service_limits(),
            &cancel
        ),
        Err(DomSourceRuntimeError::SourceEvent(
            DomSourceEventRuntimeError::Dom(DomError::SourceWorkLimit { limit: OUTER_CAP })
        ))
    ));
    assert_eq!(context.calls, vec!["outer", "child"]);
    assert!(context.edits > 0);
    assert_eq!(
        context.refusal,
        Some(DomError::SourceWorkLimit { limit: OUTER_CAP })
    );
    assert_eq!(context.refused_edit_unchanged, Some(true));
    let before = session.snapshot().stamp();
    let child_status = session.dom().pending_source_edit().unwrap();
    assert_eq!(child_status.operation(), child.operation());
    drop(permission);
    let replacement = session.source_execution();
    replacement
        .raise_event_limits(&mut session.dom(), &outer, source_limits(), &cancel)
        .unwrap();
    assert_eq!(session.snapshot().stamp(), before);
    assert_eq!(session.dom().pending_source_edit(), Some(child_status));
    let child_receipt = serviced(
        replacement
            .service_event(
                &mut session.dom(),
                &child,
                &mut listeners,
                &mut context,
                service_limits(),
                &cancel,
            )
            .unwrap(),
    );
    assert_eq!(child_receipt.callbacks(), 1);
    assert!(child_receipt.default_prevented());
    assert!(child_receipt.failures().is_empty());
    assert!(matches!(
        resume(&mut session, child.operation()),
        DomSourceDriveProgress::Complete(_)
    ));
    let outer_receipt = serviced(
        replacement
            .service_event(
                &mut session.dom(),
                &outer,
                &mut listeners,
                &mut context,
                service_limits(),
                &cancel,
            )
            .unwrap(),
    );
    assert_eq!(outer_receipt.callbacks(), 1);
    assert_eq!(context.calls, vec!["outer", "child"]);
    assert!(matches!(
        resume(&mut session, outer.operation()),
        DomSourceDriveProgress::Complete(_)
    ));
    assert!(matches!(
        session.try_shutdown(),
        DomShutdownOutcome::Complete
    ));
}

#[test]
fn late_adapter_work_refusal_recovers_same_receipt_through_qualified_monotonic_policy() {
    let cancel = AtomicBool::new(false);
    let mut limits = session_limits();
    let raised = limits.source_events;
    limits.source_events.work = 5;
    let (mut session, subject) = fixture(limits);
    let (mut foreign, _) = fixture(session_limits());
    let mut listeners = dispatcher();
    listeners
        .add_listener(
            &session.dom(),
            subject,
            DomEventType::BeforeToggle,
            DomEventCallback::new(|_, _, calls: &mut usize, event| {
                *calls += 1;
                event.prevent_default();
                Ok(())
            }),
            DomEventListenerOptions {
                capture: false,
                once: false,
            },
        )
        .unwrap();
    let request = request(&mut session, subject);
    let before = session.snapshot().stamp();
    let permission = session.source_execution();
    let mut calls = 0;
    assert!(matches!(
        permission.service_event(
            &mut session.dom(),
            &request,
            &mut listeners,
            &mut calls,
            service_limits(),
            &cancel
        ),
        Err(DomSourceRuntimeError::SourceEvent(
            DomSourceEventRuntimeError::Admission(DomSourceEventResource::Work)
        ))
    ));
    assert_eq!(
        calls, 1,
        "the actual listener ran before late adapter refusal"
    );
    let status = session.dom().pending_source_edit();
    assert_eq!(session.snapshot().stamp(), before);
    assert!(matches!(
        permission.raise_event_runtime_limits(&foreign.dom(), raised, &cancel),
        Err(DomSourceRuntimeError::SourceEvent(
            DomSourceEventRuntimeError::ForeignOwner
        ))
    ));
    assert!(matches!(
        permission.raise_event_runtime_limits(&session.dom(), raised, &AtomicBool::new(true)),
        Err(DomSourceRuntimeError::SourceEvent(
            DomSourceEventRuntimeError::Dom(DomError::Cancelled)
        ))
    ));
    for _ in 0..2 {
        assert!(matches!(
            permission.raise_event_runtime_limits(
                &session.dom(),
                DomSourceEventRuntimeLimits {
                    jobs: raised.jobs - 1,
                    ..raised
                },
                &cancel
            ),
            Err(DomSourceRuntimeError::SourceEvent(
                DomSourceEventRuntimeError::LimitReduction(DomSourceEventResource::Jobs)
            ))
        ));
    }
    assert!(matches!(
        permission.raise_event_runtime_limits(
            &session.dom(),
            DomSourceEventRuntimeLimits {
                storage_bytes: raised.storage_bytes - 1,
                ..raised
            },
            &cancel
        ),
        Err(DomSourceRuntimeError::SourceEvent(
            DomSourceEventRuntimeError::LimitReduction(DomSourceEventResource::StorageBytes)
        ))
    ));
    assert!(matches!(
        permission.raise_event_runtime_limits(
            &session.dom(),
            DomSourceEventRuntimeLimits { work: 6, ..raised },
            &cancel,
        ),
        Err(DomSourceRuntimeError::SourceEvent(
            DomSourceEventRuntimeError::Admission(DomSourceEventResource::Work)
        ))
    ));
    assert_eq!(session.dom().pending_source_edit(), status);
    assert_eq!(session.snapshot().stamp(), before);
    assert_eq!(calls, 1);
    drop(permission);
    let replacement = session.source_execution();
    replacement
        .raise_event_runtime_limits(&session.dom(), raised, &cancel)
        .unwrap();
    let receipt = serviced(
        replacement
            .service_event(
                &mut session.dom(),
                &request,
                &mut listeners,
                &mut calls,
                service_limits(),
                &cancel,
            )
            .unwrap(),
    );
    assert_eq!(calls, 1);
    assert_eq!(receipt.callbacks(), 1);
    assert!(receipt.default_prevented());
    assert!(receipt.failures().is_empty());
    let DomSourceDriveProgress::Complete(done) = resume(&mut session, request.operation()) else {
        panic!("actual canceled event completes");
    };
    assert!(!done.cancellation_requested());
    assert_eq!(session.snapshot().stamp(), before);
    assert!(matches!(
        session.try_shutdown(),
        DomShutdownOutcome::Complete
    ));
    assert!(matches!(
        replacement.raise_event_runtime_limits(&foreign.dom(), raised, &cancel),
        Err(DomSourceRuntimeError::SourceEvent(
            DomSourceEventRuntimeError::ExpiredOwner
        ))
    ));
    // A lower work policy is independently refused while its proposed cap can
    // still pay the constant validation; no exhausted-budget ambiguity.
    assert!(matches!(
        foreign.source_execution().raise_event_runtime_limits(
            &foreign.dom(),
            DomSourceEventRuntimeLimits {
                work: raised.work - 1,
                ..raised
            },
            &cancel
        ),
        Err(DomSourceRuntimeError::SourceEvent(
            DomSourceEventRuntimeError::LimitReduction(DomSourceEventResource::Work)
        ))
    ));
}

struct ByteOverlap {
    source: DomSourceRuntime,
    child: Option<Result<DomSourceDriveProgress, DomSourceRuntimeError>>,
    log: Vec<(&'static str, usize)>,
    children: usize,
}

#[test]
fn tight_ancestor_bytes_cover_live_nested_nodes_and_allow_repeated_child_removal() {
    const TIGHT_BYTES: usize = 700_000;
    let cancel = AtomicBool::new(false);
    let ordinary = source_limits();
    let tight = DomSourceLimits::new(
        TIGHT_BYTES,
        ordinary.retained_string_code_units(),
        ordinary.work(),
        ordinary.phase_publications(),
        ordinary.output_records(),
        ordinary.published_native_identities(),
        ordinary.nested_operations(),
    );
    let event_limits = DomEventDispatchLimits {
        path_items: 4096,
        snapshot_listeners: 4,
        failure_records: 4,
        ..dispatch_limits()
    };
    let selected = DomSourceEventServiceLimits {
        source: tight,
        dispatch: event_limits,
    };
    let mut limits = session_limits();
    limits.source_events.jobs = 2;
    // Independent public control: the same fixed Source profile admits one
    // actual job and its entire reservation, rather than failing setup alone.
    let (mut control, control_subject) = fixture(limits);
    let mut control_listeners = dispatcher();
    control_listeners
        .add_listener(
            &control.dom(),
            control_subject,
            DomEventType::BeforeToggle,
            DomEventCallback::new(|_, _, calls: &mut usize, event| {
                *calls += 1;
                event.prevent_default();
                Ok(())
            }),
            DomEventListenerOptions {
                capture: false,
                once: false,
            },
        )
        .unwrap();
    let control_request = request(&mut control, control_subject);
    let mut control_calls = 0;
    let receipt = serviced(
        control
            .source_execution()
            .service_event(
                &mut control.dom(),
                &control_request,
                &mut control_listeners,
                &mut control_calls,
                selected,
                &cancel,
            )
            .unwrap(),
    );
    assert_eq!(control_calls, 1);
    assert_eq!(receipt.callbacks(), 1);
    assert!(matches!(
        resume(&mut control, control_request.operation()),
        DomSourceDriveProgress::Complete(_)
    ));
    assert!(matches!(
        control.try_shutdown(),
        DomShutdownOutcome::Complete
    ));

    let (mut session, first, second) = fixture_pair(limits);
    let mut listeners = dispatcher();
    for number in 1..=2 {
        listeners
            .add_listener(
                &session.dom(),
                first,
                DomEventType::BeforeToggle,
                DomEventCallback::new(move |dom, _, context: &mut ByteOverlap, event| {
                    context.log.push(("outer", number));
                    event.prevent_default();
                    let mut policy = |_: &DomStyleSource| None;
                    context.child = Some(context.source.apply(
                        dom,
                        DomSourceEdit::show_dialog(second),
                        source_limits(),
                        &mut DomSourceServices {
                            style_policy: &mut policy,
                            url: None,
                        },
                        &AtomicBool::new(false),
                    ));
                    Ok(())
                }),
                DomEventListenerOptions {
                    capture: false,
                    once: false,
                },
            )
            .unwrap();
    }
    listeners
        .add_listener(
            &session.dom(),
            second,
            DomEventType::BeforeToggle,
            DomEventCallback::new(|_, _, context: &mut ByteOverlap, event| {
                context.children += 1;
                context.log.push(("child", context.children));
                event.prevent_default();
                Ok(())
            }),
            DomEventListenerOptions {
                capture: false,
                once: false,
            },
        )
        .unwrap();
    let outer = request(&mut session, first);
    let mut context = ByteOverlap {
        source: session.source_execution(),
        child: None,
        log: Vec::new(),
        children: 0,
    };
    let permission = session.source_execution();
    for number in 1..=2 {
        let progress = permission
            .service_event(
                &mut session.dom(),
                &outer,
                &mut listeners,
                &mut context,
                selected,
                &cancel,
            )
            .unwrap();
        assert!(
            matches!(
                progress,
                DomSourceEventServiceProgress::Retained {
                    dispatch: DomEventDispatchOutcome::RetainedSourceOperation(_)
                }
            ),
            "actual outer {progress:?}; child formation {:?}",
            context.child
        );
        let child_progress = context
            .child
            .take()
            .expect("actual formation attempt")
            .expect("child observation fits before job reservation");
        let DomSourceDriveProgress::Pending {
            request: DomSourceRequest::Event(child),
            reason: DomSourceUnavailable::Event,
        } = child_progress
        else {
            panic!("actual nested child: {child_progress:?}");
        };
        assert_eq!(child.parent_operation(), Some(outer.operation()));
        let before = session.snapshot().stamp();
        assert!(matches!(
            permission.service_event(
                &mut session.dom(),
                &child,
                &mut listeners,
                &mut context,
                DomSourceEventServiceLimits {
                    source: source_limits(),
                    dispatch: event_limits
                },
                &cancel
            ),
            Err(DomSourceRuntimeError::SourceEvent(
                DomSourceEventRuntimeError::Dom(DomError::ResourceLimit {
                    resource: DomResource::SourceRetainedBytes,
                    limit: TIGHT_BYTES
                })
            ))
        ));
        assert_eq!(
            context.children,
            number - 1,
            "refused formation must not enter the child listener"
        );
        assert_eq!(session.snapshot().stamp(), before);
        assert_eq!(
            session.dom().pending_source_edit().unwrap().operation(),
            child.operation()
        );
        permission
            .raise_event_limits(&mut session.dom(), &outer, source_limits(), &cancel)
            .unwrap();
        let receipt = serviced(
            permission
                .service_event(
                    &mut session.dom(),
                    &child,
                    &mut listeners,
                    &mut context,
                    DomSourceEventServiceLimits {
                        source: source_limits(),
                        dispatch: event_limits,
                    },
                    &cancel,
                )
                .unwrap(),
        );
        assert_eq!(receipt.callbacks(), 1);
        assert!(receipt.default_prevented());
        assert!(matches!(
            resume(&mut session, child.operation()),
            DomSourceDriveProgress::Complete(_)
        ));
        // Next iteration lowers the original parent to the same tight profile.
        // It must fit after the child node and reservation physically release;
        // the two-job policy must also admit the next genuine child.
    }
    let receipt = serviced(
        permission
            .service_event(
                &mut session.dom(),
                &outer,
                &mut listeners,
                &mut context,
                selected,
                &cancel,
            )
            .unwrap(),
    );
    assert_eq!(receipt.callbacks(), 2);
    assert_eq!(
        context.log,
        vec![("outer", 1), ("child", 1), ("outer", 2), ("child", 2)]
    );
    assert!(matches!(
        resume(&mut session, outer.operation()),
        DomSourceDriveProgress::Complete(_)
    ));
    assert!(matches!(
        session.try_shutdown(),
        DomShutdownOutcome::Complete
    ));
}
