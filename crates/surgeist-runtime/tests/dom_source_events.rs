//! The original Session dispatches actual Source beforetoggle requests. Closed
//! native fixtures and the separate real hosted constructor test their admitted
//! boundaries without claiming completed modal or hosted Show services.
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
        dom_manipulation: DomManipulationRuntimeLimits {
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

fn pending_dialog_service(
    progress: DomSourceDriveProgress,
    subject: NodeId,
    expected: HtmlDialogAction,
) -> DomSourceOperationId {
    match progress {
        DomSourceDriveProgress::Pending {
            request: DomSourceRequest::Dialog(dialog),
            reason: DomSourceUnavailable::Dialog(action),
        } => {
            assert_eq!(dialog.subject(), subject);
            assert_eq!(dialog.action(), expected);
            assert_eq!(action, expected);
            dialog.operation()
        }
        other => panic!("the original Dialog service must remain pending: {other:?}"),
    }
}

fn show_publishes_open_and_retains_queued_task(
    mut session: DomSession,
    subject: NodeId,
    next: HtmlDialogAction,
    association: Option<(DomHostedGlobalId, DocumentId)>,
) {
    struct Context {
        owner: DomManipulationRuntime,
        beforetoggle_calls: usize,
        toggle_calls: usize,
    }
    impl DomManipulationEventContext for Context {
        fn dom_manipulation(&mut self) -> &mut DomManipulationRuntime {
            &mut self.owner
        }
    }
    let cancel = AtomicBool::new(false);
    let mut listeners = dispatcher();
    listeners
        .add_listener(
            &session.dom(),
            subject,
            DomEventType::BeforeToggle,
            DomEventCallback::new(move |dom, _, context: &mut Context, _| {
                context.beforetoggle_calls += 1;
                let mut batch = DomBatch::new();
                batch.set_attribute(subject, "data-beforetoggle", "kept");
                dom.apply(batch, &AtomicBool::new(false)).unwrap();
                Ok(())
            }),
            DomEventListenerOptions::default(),
        )
        .unwrap();
    listeners
        .add_listener(
            &session.dom(),
            subject,
            DomEventType::Toggle,
            DomEventCallback::new(|_, _, context: &mut Context, _| {
                context.toggle_calls += 1;
                Ok(())
            }),
            DomEventListenerOptions::default(),
        )
        .unwrap();
    let mut context = Context {
        owner: session.dom_manipulation_execution(),
        beforetoggle_calls: 0,
        toggle_calls: 0,
    };
    let request = request(&mut session, subject);
    assert_eq!(request.old_state(), HtmlDialogState::Closed);
    assert_eq!(request.new_state(), HtmlDialogState::Open);
    assert_eq!(request.source(), None);
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
    assert_eq!(receipt.callbacks(), 1);
    assert!(receipt.failures().is_empty());
    assert!(!receipt.default_prevented());
    drop(receipt);
    let before_addition = session.snapshot();
    assert_eq!(
        before_addition
            .node(subject)
            .unwrap()
            .attribute(&"open".into()),
        None
    );
    let progress = resume(&mut session, request.operation());
    let published = session.snapshot();
    // HTML877d Show step 6 adds the ordinary empty attribute. Its hook must
    // finish, or park at its own watcher, before Show can remember focus.
    assert_eq!(
        published.node(subject).unwrap().attribute(&"open".into()),
        Some(&"".into())
    );
    assert_eq!(
        pending_dialog_service(progress, subject, next),
        request.operation()
    );
    assert_ne!(published.stamp(), before_addition.stamp());
    assert_eq!(
        published
            .node(subject)
            .unwrap()
            .attribute(&"data-beforetoggle".into()),
        Some(&"kept".into())
    );
    assert_eq!(
        published.incomplete_source_operation(),
        Some(request.operation())
    );
    assert_eq!(context.owner.pending_tasks().unwrap(), 1);
    let task = context.owner.pending_task(0).unwrap().unwrap();
    let dialog = task.dialog().expect("the actual Show toggle task");
    assert_eq!(dialog.id().target(), subject);
    assert_eq!(dialog.old_state(), HtmlDialogState::Closed);
    assert_eq!(dialog.new_state(), HtmlDialogState::Open);
    assert_eq!(dialog.source(), None);
    assert_eq!(
        dialog
            .queue_association()
            .map(|queue| (queue.global(), queue.document())),
        association
    );
    let tracker = published
        .node(subject)
        .unwrap()
        .as_dialog()
        .unwrap()
        .toggle_task_tracker()
        .unwrap();
    assert_eq!(tracker.task(), dialog.id());
    assert_eq!(tracker.old_state(), HtmlDialogState::Closed);

    assert_eq!(
        pending_dialog_service(resume(&mut session, request.operation()), subject, next),
        request.operation()
    );
    assert_eq!(context.beforetoggle_calls, 1);
    assert_eq!(context.owner.pending_tasks().unwrap(), 1);
    assert_eq!(context.owner.pending_task(0).unwrap(), Some(task));
    let retried = session.snapshot();
    assert_eq!(retried.stamp(), published.stamp());
    assert_eq!(
        retried
            .node(subject)
            .unwrap()
            .as_dialog()
            .unwrap()
            .toggle_task_tracker(),
        Some(tracker)
    );
    assert_eq!(
        retried.node(subject).unwrap().attribute(&"open".into()),
        Some(&"".into())
    );
    assert!(matches!(
        DomManipulationRuntime::run_next(
            &mut session.dom(),
            &mut listeners,
            &mut context,
            DomManipulationRunLimits {
                dispatch: dispatch_limits(),
                source: source_limits()
            },
            &cancel,
        ),
        Err(DomManipulationRuntimeError::Dom(
            DomError::SourceOperationPending
        ))
    ));
    assert_eq!(context.toggle_calls, 0);
    assert_eq!(context.beforetoggle_calls, 1);
    assert_eq!(context.owner.pending_task(0).unwrap(), Some(task));
    assert_eq!(session.snapshot().stamp(), published.stamp());
    assert_eq!(
        before_addition
            .node(subject)
            .unwrap()
            .attribute(&"open".into()),
        None
    );
    // No focus value, watcher reply or completed Show is supplied by this test.
}

#[test]
fn native_show_publishes_empty_open_before_waiting_for_remembered_focus() {
    let (session, subject) = fixture(session_limits());
    assert_eq!(session.snapshot().relevant_global(subject).unwrap(), None);
    show_publishes_open_and_retains_queued_task(
        session,
        subject,
        HtmlDialogAction::RememberFocusedElement,
        None,
    );
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
fn uncanceled_resume_publishes_open_and_toggle_tracker_without_replaying_callback() {
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
            // the producer's next action is the still-unserviced remembered focus.
            assert_eq!(action, HtmlDialogAction::RememberFocusedElement);
        }
        other => panic!(
            "queued toggle must retain the original Show at its actual next service: {other:?}"
        ),
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
        Some(&"".into())
    );
}

#[cfg(any(
    target_os = "macos",
    target_os = "linux",
    target_os = "windows",
    target_os = "android",
    target_os = "ios"
))]
#[test]
fn hosted_uncanceled_resume_publishes_open_and_toggle_tracker_then_parks_at_watcher() {
    let cancel = AtomicBool::new(false);
    // The same public constructor used by native_hosted_lifecycle establishes
    // the actual Global/settings/agent/window loop; no native Document is
    // converted to hosted state by a fixture flag.
    let dom = Dom::new(
        DomLimits::new(128, 100_000, 256, 128, 128, 32, 4_000_000)
            .unwrap()
            .with_dispatch_custody(
                DomDispatchCustodyLimits::new(64, 4096, 16_000_000, 16_000_000, 4_000_000).unwrap(),
            ),
    )
    .unwrap();
    let mut session = DomSession::new(dom, session_limits()).unwrap();
    let mut hosted = session.hosted_execution();
    let top = {
        let mut dom = session.dom();
        let host = hosted.create_host(&mut dom, &cancel).unwrap();
        let top = hosted.create_top_level(&mut dom, host, &cancel).unwrap();
        assert_eq!(top.host(), host);
        top
    };
    let constructed = session.snapshot();
    assert!(
        constructed
            .document(top.document())
            .unwrap()
            .is_fully_active()
            .unwrap()
    );
    assert_eq!(
        constructed.global_document(top.global()).unwrap(),
        top.document()
    );
    let settings = hosted
        .settings(&session.dom(), top.global(), &cancel)
        .unwrap();
    assert_eq!(settings.document().id(), top.document());
    assert!(settings.environment_ready());
    let html = constructed.node(top.document().node()).unwrap().children()[0];
    let children = constructed.node(html).unwrap().children();
    assert_eq!(children.len(), 2);
    let body = children[1];
    let subject = {
        let mut batch = DomBatch::new();
        let dialog = batch.create_element_ns(top.document(), Namespace::html(), "dialog");
        batch.insert_before(&dialog, body, None);
        session
            .dom()
            .apply(batch, &cancel)
            .unwrap()
            .created(&dialog)
            .unwrap()
            .unwrap()
    };
    assert_eq!(
        session.snapshot().relevant_global(subject).unwrap(),
        Some(top.global())
    );
    let mut listeners = dispatcher();
    listeners
        .add_listener(
            &session.dom(),
            subject,
            DomEventType::BeforeToggle,
            DomEventCallback::new(
                move |dom, _, observed: &mut (usize, Vec<DomEventTarget>), event| {
                    observed.0 += 1;
                    observed.1 = event.composed_path().collect();
                    let mut batch = DomBatch::new();
                    batch.set_attribute(subject, "data-hosted-callback", "kept");
                    dom.apply(batch, &AtomicBool::new(false)).unwrap();
                    Ok(())
                },
            ),
            DomEventListenerOptions {
                capture: false,
                once: false,
            },
        )
        .unwrap();
    let request = request(&mut session, subject);
    // Pinned HTML877d Show step 5 queues closed -> open with a null source;
    // this remains distinct from future nonnull-source/Close coverage.
    assert_eq!(request.old_state(), HtmlDialogState::Closed);
    assert_eq!(request.new_state(), HtmlDialogState::Open);
    assert_eq!(request.source(), None);
    let mut observed = (0, Vec::new());
    let receipt = serviced(
        session
            .source_execution()
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
    assert_eq!(receipt.callbacks(), 1);
    assert!(receipt.failures().is_empty());
    assert!(!receipt.default_prevented());
    assert_eq!(observed.0, 1);
    assert_eq!(
        observed.1,
        [
            DomEventTarget::Node(subject),
            DomEventTarget::Node(body),
            DomEventTarget::Node(html),
            DomEventTarget::Node(top.document().node()),
            DomEventTarget::Global(top.global()),
        ]
    );
    let callback_snapshot = session.snapshot();
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
            assert_eq!(dialog.action(), HtmlDialogAction::EstablishCloseWatcher);
            assert_eq!(action, HtmlDialogAction::EstablishCloseWatcher);
        }
        other => {
            panic!(
                "hosted queue publication must retain its original Show at its actual next service: {other:?}"
            )
        }
    }
    let snapshot = session.snapshot();
    assert_ne!(snapshot.stamp(), callback_snapshot.stamp());
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
        snapshot.node(subject).unwrap().owner_document(),
        Some(top.document())
    );
    assert_eq!(
        snapshot.global_document(top.global()).unwrap(),
        top.document()
    );
    assert_eq!(
        snapshot
            .node(subject)
            .unwrap()
            .attribute(&"data-hosted-callback".into()),
        Some(&"kept".into())
    );
    assert_eq!(
        snapshot.node(subject).unwrap().attribute(&"open".into()),
        Some(&"".into())
    );
    assert_eq!(observed.0, 1);
    // This pending synchronous Source tail is not permission to pump the
    // queued event, acknowledge the watcher, or claim completed Show.
}

#[cfg(any(
    target_os = "macos",
    target_os = "linux",
    target_os = "windows",
    target_os = "android",
    target_os = "ios"
))]
fn hosted_queue_fixture() -> (DomSession, DomHostedTopLevel, NodeId) {
    let cancel = AtomicBool::new(false);
    let dom = Dom::new(
        DomLimits::new(128, 100_000, 256, 128, 128, 32, 4_000_000)
            .unwrap()
            .with_dispatch_custody(
                DomDispatchCustodyLimits::new(64, 4096, 16_000_000, 16_000_000, 4_000_000).unwrap(),
            )
            .with_details_toggle_intentions(8)
            .unwrap(),
    )
    .unwrap();
    let mut session = DomSession::new(dom, session_limits()).unwrap();
    let mut hosted = session.hosted_execution();
    let top = {
        let mut dom = session.dom();
        let host = hosted.create_host(&mut dom, &cancel).unwrap();
        hosted.create_top_level(&mut dom, host, &cancel).unwrap()
    };
    let snapshot = session.snapshot();
    assert!(
        snapshot
            .document(top.document())
            .unwrap()
            .is_fully_active()
            .unwrap()
    );
    assert_eq!(
        snapshot.global_document(top.global()).unwrap(),
        top.document()
    );
    let html = snapshot.node(top.document().node()).unwrap().children()[0];
    let children = snapshot.node(html).unwrap().children();
    assert_eq!(children.len(), 2);
    (session, top, children[1])
}

#[cfg(any(
    target_os = "macos",
    target_os = "linux",
    target_os = "windows",
    target_os = "android",
    target_os = "ios"
))]
#[test]
fn hosted_show_publishes_empty_open_before_waiting_for_close_watcher() {
    let cancel = AtomicBool::new(false);
    let (mut session, top, body) = hosted_queue_fixture();
    let subject = {
        let mut batch = DomBatch::new();
        let dialog = batch.create_element_ns(top.document(), Namespace::html(), "dialog");
        batch.insert_before(&dialog, body, None);
        session
            .dom()
            .apply(batch, &cancel)
            .unwrap()
            .created(&dialog)
            .unwrap()
            .unwrap()
    };
    assert_eq!(
        session.snapshot().node(subject).unwrap().parent(),
        Some(body)
    );
    assert_eq!(
        session.snapshot().relevant_global(subject).unwrap(),
        Some(top.global())
    );
    // HTML877d's open-change guard passes for this genuinely connected,
    // fully active Document; setup reaches the watcher before Show step 7.
    show_publishes_open_and_retains_queued_task(
        session,
        subject,
        HtmlDialogAction::EstablishCloseWatcher,
        Some((top.global(), top.document())),
    );
}

#[cfg(any(
    target_os = "macos",
    target_os = "linux",
    target_os = "windows",
    target_os = "android",
    target_os = "ios"
))]
#[test]
fn hosted_source_open_attribute_publishes_hook_prefix_before_waiting_for_close_watcher() {
    let cancel = AtomicBool::new(false);
    let (mut session, top, body) = hosted_queue_fixture();
    let subject = {
        let mut batch = DomBatch::new();
        let dialog = batch.create_element_ns(top.document(), Namespace::html(), "dialog");
        batch.insert_before(&dialog, body, None);
        session
            .dom()
            .apply(batch, &cancel)
            .unwrap()
            .created(&dialog)
            .unwrap()
            .unwrap()
    };
    let original = session.snapshot();
    assert_eq!(
        original.node(subject).unwrap().attribute(&"open".into()),
        None
    );
    assert_eq!(
        original.relevant_global(subject).unwrap(),
        Some(top.global())
    );
    assert_eq!(original.node(subject).unwrap().parent(), Some(body));
    let progress = {
        let mut policy = |_: &DomStyleSource| None;
        session
            .source_execution()
            .apply(
                &mut session.dom(),
                DomSourceEdit::set_attribute(subject, "open", ""),
                source_limits(),
                &mut DomSourceServices {
                    style_policy: &mut policy,
                    url: None,
                },
                &cancel,
            )
            .unwrap()
    };
    // The ordinary attribute request owns the nested hook independently of
    // Show. HTML877d setup adds the open-list entry before its watcher wait.
    let operation =
        pending_dialog_service(progress, subject, HtmlDialogAction::EstablishCloseWatcher);
    let published = session.snapshot();
    assert_ne!(published.stamp(), original.stamp());
    assert_eq!(
        published.node(subject).unwrap().attribute(&"open".into()),
        Some(&"".into())
    );
    assert_eq!(published.incomplete_source_operation(), Some(operation));
    assert!(
        !published
            .node(subject)
            .unwrap()
            .as_dialog()
            .unwrap()
            .has_toggle_task_tracker()
    );
    assert_eq!(
        session
            .dom_manipulation_execution()
            .pending_tasks()
            .unwrap(),
        0
    );
    assert_eq!(
        pending_dialog_service(
            resume(&mut session, operation),
            subject,
            HtmlDialogAction::EstablishCloseWatcher
        ),
        operation
    );
    let retried = session.snapshot();
    assert_eq!(retried.stamp(), published.stamp());
    assert_eq!(
        retried.node(subject).unwrap().attribute(&"open".into()),
        Some(&"".into())
    );
    assert_eq!(retried.incomplete_source_operation(), Some(operation));
    assert_eq!(
        session
            .dom_manipulation_execution()
            .pending_tasks()
            .unwrap(),
        0
    );
    assert_eq!(
        original.node(subject).unwrap().attribute(&"open".into()),
        None
    );
    // No beforetoggle or toggle queue belongs to direct attribute addition,
    // and no watcher completion is fabricated here.
}

#[cfg(any(
    target_os = "macos",
    target_os = "linux",
    target_os = "windows",
    target_os = "android",
    target_os = "ios"
))]
#[test]
fn hosted_details_intake_precedes_dialog_queue_and_unfinished_show_blocks_all_delivery() {
    struct Context {
        owner: DomManipulationRuntime,
        beforetoggle_calls: usize,
        toggle_calls: usize,
        callback_edits: Vec<Result<DomCommit, DomError>>,
    }
    impl DomManipulationEventContext for Context {
        fn dom_manipulation(&mut self) -> &mut DomManipulationRuntime {
            &mut self.owner
        }
    }
    let cancel = AtomicBool::new(false);
    let (mut session, top, body) = hosted_queue_fixture();
    let (a, b, subject) = {
        let mut batch = DomBatch::new();
        let a = batch.create_element_ns(top.document(), Namespace::html(), "details");
        let b = batch.create_element_ns(top.document(), Namespace::html(), "details");
        let dialog = batch.create_element_ns(top.document(), Namespace::html(), "dialog");
        for node in [&a, &b, &dialog] {
            batch.insert_before(node, body, None);
        }
        let commit = session.dom().apply(batch, &cancel).unwrap();
        (
            commit.created(&a).unwrap().unwrap(),
            commit.created(&b).unwrap().unwrap(),
            commit.created(&dialog).unwrap().unwrap(),
        )
    };
    let mut context = Context {
        owner: session.dom_manipulation_execution(),
        beforetoggle_calls: 0,
        toggle_calls: 0,
        callback_edits: Vec::new(),
    };
    let mut batch = DomBatch::new();
    batch.set_attribute(a, "open", "");
    session.dom().apply(batch, &cancel).unwrap();
    assert_eq!(
        context.owner.ingest(&mut session.dom(), &cancel).unwrap(),
        1
    );
    let original_a = context.owner.pending_task(0).unwrap().unwrap();
    assert!(matches!(original_a, DomManipulationTask::Details(_)));
    assert_eq!(original_a.id().target(), a);

    let mut listeners = dispatcher();
    listeners
        .add_listener(
            &session.dom(),
            subject,
            DomEventType::BeforeToggle,
            DomEventCallback::new(move |dom, _, context: &mut Context, _| {
                context.beforetoggle_calls += 1;
                let mut batch = DomBatch::new();
                batch.set_attribute(b, "open", "");
                batch.set_attribute(subject, "data-callback", "kept");
                context
                    .callback_edits
                    .push(dom.apply(batch, &AtomicBool::new(false)));
                Ok(())
            }),
            DomEventListenerOptions::default(),
        )
        .unwrap();
    for target in [a, b, subject] {
        listeners
            .add_listener(
                &session.dom(),
                target,
                DomEventType::Toggle,
                DomEventCallback::new(|_, _, context: &mut Context, _| {
                    context.toggle_calls += 1;
                    Ok(())
                }),
                DomEventListenerOptions::default(),
            )
            .unwrap();
    }
    let request = request(&mut session, subject);
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
    assert_eq!(receipt.callbacks(), 1);
    assert!(receipt.failures().is_empty());
    assert!(!receipt.default_prevented());
    assert_eq!(context.beforetoggle_calls, 1);
    assert_eq!(context.callback_edits.len(), 1);
    assert!(context.callback_edits[0].is_ok());
    let callback_snapshot = session.snapshot();
    match resume(&mut session, request.operation()) {
        DomSourceDriveProgress::Pending {
            request: DomSourceRequest::Dialog(dialog),
            reason: DomSourceUnavailable::Dialog(action),
        } => {
            assert_eq!(dialog.operation(), request.operation());
            assert_eq!(dialog.subject(), subject);
            assert_eq!(dialog.action(), HtmlDialogAction::EstablishCloseWatcher);
            assert_eq!(action, HtmlDialogAction::EstablishCloseWatcher);
        }
        other => panic!("actual queueing must leave Show at its actual next service: {other:?}"),
    }
    assert_eq!(context.owner.pending_tasks().unwrap(), 3);
    let pending = (0..3)
        .map(|index| context.owner.pending_task(index).unwrap().unwrap())
        .collect::<Vec<_>>();
    // Both notifications were queued before Dialog's task. B's raw intention
    // arose during the authentic callback and must join A in this same bank.
    assert_eq!(
        pending
            .iter()
            .map(|task| task.id().target())
            .collect::<Vec<_>>(),
        [a, b, subject]
    );
    assert_eq!(pending[0], original_a);
    for task in &pending[..2] {
        let details = task.details().expect("the preceding actual Details task");
        assert_eq!(details.event().old_state().as_str(), "closed");
        assert_eq!(details.event().new_state().as_str(), "open");
    }
    let dialog_task = pending[2].dialog().expect("the actual Dialog task");
    assert_eq!(dialog_task.old_state(), HtmlDialogState::Closed);
    assert_eq!(dialog_task.new_state(), HtmlDialogState::Open);
    assert_eq!(dialog_task.source(), None);
    for task in &pending {
        let association = task
            .queue_association()
            .expect("the actual hosted queue association");
        assert_eq!(association.global(), top.global());
        assert_eq!(association.document(), top.document());
    }
    let queued_snapshot = session.snapshot();
    let tracker = queued_snapshot
        .node(subject)
        .unwrap()
        .as_dialog()
        .unwrap()
        .toggle_task_tracker()
        .unwrap();
    assert_eq!(tracker.task(), dialog_task.id());
    assert_eq!(tracker.old_state(), HtmlDialogState::Closed);
    assert_ne!(queued_snapshot.stamp(), callback_snapshot.stamp());
    assert_eq!(
        queued_snapshot
            .node(subject)
            .unwrap()
            .attribute(&"open".into()),
        Some(&"".into())
    );
    assert_eq!(
        queued_snapshot
            .node(subject)
            .unwrap()
            .attribute(&"data-callback".into()),
        Some(&"kept".into())
    );
    assert!(matches!(
        DomManipulationRuntime::run_next(
            &mut session.dom(),
            &mut listeners,
            &mut context,
            DomManipulationRunLimits {
                dispatch: dispatch_limits(),
                source: source_limits()
            },
            &cancel,
        ),
        Err(DomManipulationRuntimeError::Dom(
            DomError::SourceOperationPending
        ))
    ));
    assert_eq!(context.toggle_calls, 0);
    assert_eq!(context.beforetoggle_calls, 1);
    assert_eq!(session.snapshot().stamp(), queued_snapshot.stamp());
    assert_eq!(
        session.snapshot().incomplete_source_operation(),
        Some(request.operation())
    );
    assert_eq!(context.owner.pending_tasks().unwrap(), 3);
    assert_eq!(
        (0..3)
            .map(|index| context.owner.pending_task(index).unwrap().unwrap())
            .collect::<Vec<_>>(),
        pending
    );
}

#[cfg(any(
    target_os = "macos",
    target_os = "linux",
    target_os = "windows",
    target_os = "android",
    target_os = "ios"
))]
#[test]
fn hosted_dialog_queue_uses_original_relevant_global_after_beforetoggle_adoption() {
    struct Context {
        calls: usize,
        edits: Vec<Result<DomCommit, DomError>>,
        path: Vec<DomEventTarget>,
    }
    let cancel = AtomicBool::new(false);
    let (mut session, first, first_body) = hosted_queue_fixture();
    let second = session
        .hosted_execution()
        .create_top_level(&mut session.dom(), first.host(), &cancel)
        .unwrap();
    let initial = session.snapshot();
    let html = initial.node(second.document().node()).unwrap().children()[0];
    let children = initial.node(html).unwrap().children();
    assert_eq!(children.len(), 2);
    let second_body = children[1];
    assert!(
        initial
            .document(second.document())
            .unwrap()
            .is_fully_active()
            .unwrap()
    );
    assert_ne!(first.global(), second.global());
    let subject = {
        let mut batch = DomBatch::new();
        let dialog = batch.create_element_ns(first.document(), Namespace::html(), "dialog");
        batch.insert_before(&dialog, first_body, None);
        session
            .dom()
            .apply(batch, &cancel)
            .unwrap()
            .created(&dialog)
            .unwrap()
            .unwrap()
    };
    let original = session.snapshot();
    assert_eq!(
        original.node(subject).unwrap().owner_document(),
        Some(first.document())
    );
    assert_eq!(
        original.relevant_global(subject).unwrap(),
        Some(first.global())
    );
    let mut listeners = dispatcher();
    listeners
        .add_listener(
            &session.dom(),
            subject,
            DomEventType::BeforeToggle,
            DomEventCallback::new(move |dom, _, context: &mut Context, event| {
                context.calls += 1;
                context.path = event.composed_path().collect();
                let mut batch = DomBatch::new();
                batch.adopt(subject, second.document());
                batch.insert_before(subject, second_body, None);
                context
                    .edits
                    .push(dom.apply(batch, &AtomicBool::new(false)));
                Ok(())
            }),
            DomEventListenerOptions::default(),
        )
        .unwrap();
    let request = request(&mut session, subject);
    let mut context = Context {
        calls: 0,
        edits: Vec::new(),
        path: Vec::new(),
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
    assert_eq!(receipt.callbacks(), 1);
    assert!(receipt.failures().is_empty());
    assert!(!receipt.default_prevented());
    assert_eq!(context.calls, 1);
    assert_eq!(context.edits.len(), 1);
    assert!(context.edits[0].is_ok());
    assert_eq!(
        context.path.last(),
        Some(&DomEventTarget::Global(first.global()))
    );
    let adopted = session.snapshot();
    assert_eq!(
        adopted.node(subject).unwrap().owner_document(),
        Some(second.document())
    );
    assert_eq!(adopted.node(subject).unwrap().parent(), Some(second_body));
    match resume(&mut session, request.operation()) {
        DomSourceDriveProgress::Pending {
            request: DomSourceRequest::Dialog(dialog),
            reason: DomSourceUnavailable::Dialog(action),
        } => {
            assert_eq!(dialog.operation(), request.operation());
            assert_eq!(dialog.subject(), subject);
            assert_eq!(dialog.action(), HtmlDialogAction::EstablishCloseWatcher);
            assert_eq!(action, HtmlDialogAction::EstablishCloseWatcher);
        }
        other => {
            panic!("adopted Dialog queueing must retain Show at its actual next service: {other:?}")
        }
    }
    let owner = session.dom_manipulation_execution();
    assert_eq!(owner.pending_tasks().unwrap(), 1);
    let task = owner
        .pending_task(0)
        .unwrap()
        .unwrap()
        .dialog()
        .expect("the actual queued Dialog");
    assert_eq!(task.id().target(), subject);
    assert_eq!(task.old_state(), HtmlDialogState::Closed);
    assert_eq!(task.new_state(), HtmlDialogState::Open);
    assert_eq!(task.source(), None);
    let association = task
        .queue_association()
        .expect("the original hosted queue association");
    assert_eq!(association.global(), first.global());
    assert_eq!(association.document(), first.document());
    assert_ne!(association.document(), second.document());
    let queued = session.snapshot();
    assert_eq!(
        queued.node(subject).unwrap().owner_document(),
        Some(second.document())
    );
    assert_eq!(queued.node(subject).unwrap().parent(), Some(second_body));
    assert_eq!(
        queued.node(subject).unwrap().attribute(&"open".into()),
        Some(&"".into())
    );
    let tracker = queued
        .node(subject)
        .unwrap()
        .as_dialog()
        .unwrap()
        .toggle_task_tracker()
        .unwrap();
    assert_eq!(tracker.task(), task.id());
    assert_eq!(tracker.old_state(), HtmlDialogState::Closed);
    assert_eq!(
        queued.incomplete_source_operation(),
        Some(request.operation())
    );
    assert_eq!(context.calls, 1);
    assert_eq!(
        original.node(subject).unwrap().owner_document(),
        Some(first.document())
    );
    // Adoption happened before queue capture. This does not claim dispatch or
    // the still-unreachable queue-then-adopt execution path behind the unfinished Show tail.
}

#[cfg(any(
    target_os = "macos",
    target_os = "linux",
    target_os = "windows",
    target_os = "android",
    target_os = "ios"
))]
#[test]
fn canceled_child_intention_joins_parent_queue_while_grandparent_observation_survives_limit_raise()
{
    struct Context {
        source: DomSourceRuntime,
        requests: Vec<DomSourceEventRequest>,
        calls: Vec<&'static str>,
        edits: Vec<Result<DomCommit, DomError>>,
    }
    let cancel = AtomicBool::new(false);
    let (mut session, top, body) = hosted_queue_fixture();
    let (grandparent, parent, child, details) = {
        let mut batch = DomBatch::new();
        let grandparent = batch.create_element_ns(top.document(), Namespace::html(), "dialog");
        let parent = batch.create_element_ns(top.document(), Namespace::html(), "dialog");
        let child = batch.create_element_ns(top.document(), Namespace::html(), "dialog");
        let details = batch.create_element_ns(top.document(), Namespace::html(), "details");
        for node in [&grandparent, &parent, &child, &details] {
            batch.insert_before(node, body, None);
        }
        let commit = session.dom().apply(batch, &cancel).unwrap();
        (
            commit.created(&grandparent).unwrap().unwrap(),
            commit.created(&parent).unwrap().unwrap(),
            commit.created(&child).unwrap().unwrap(),
            commit.created(&details).unwrap().unwrap(),
        )
    };
    let mut listeners = dispatcher();
    for (subject, next, label) in [(grandparent, parent, "G"), (parent, child, "P")] {
        listeners
            .add_listener(
                &session.dom(),
                subject,
                DomEventType::BeforeToggle,
                DomEventCallback::new(move |dom, _, context: &mut Context, _| {
                    context.calls.push(label);
                    let mut policy = |_: &DomStyleSource| None;
                    let progress = context
                        .source
                        .apply(
                            dom,
                            DomSourceEdit::show_dialog(next),
                            source_limits(),
                            &mut DomSourceServices {
                                style_policy: &mut policy,
                                url: None,
                            },
                            &AtomicBool::new(false),
                        )
                        .unwrap();
                    let DomSourceDriveProgress::Pending {
                        request: DomSourceRequest::Event(request),
                        reason: DomSourceUnavailable::Event,
                    } = progress
                    else {
                        panic!(
                            "the real nested Show must issue its own beforetoggle: {progress:?}"
                        );
                    };
                    context.requests.push(request);
                    Ok(())
                }),
                DomEventListenerOptions::default(),
            )
            .unwrap();
    }
    listeners
        .add_listener(
            &session.dom(),
            child,
            DomEventType::BeforeToggle,
            DomEventCallback::new(move |dom, _, context: &mut Context, event| {
                context.calls.push("C");
                let mut batch = DomBatch::new();
                batch.set_attribute(details, "open", "");
                context
                    .edits
                    .push(dom.apply(batch, &AtomicBool::new(false)));
                event.prevent_default();
                Ok(())
            }),
            DomEventListenerOptions::default(),
        )
        .unwrap();
    let grandparent_request = request(&mut session, grandparent);
    let mut context = Context {
        source: session.source_execution(),
        requests: Vec::new(),
        calls: Vec::new(),
        edits: Vec::new(),
    };
    let permission = session.source_execution();
    let grandparent_progress = permission
        .service_event(
            &mut session.dom(),
            &grandparent_request,
            &mut listeners,
            &mut context,
            service_limits(),
            &cancel,
        )
        .unwrap();
    let parent_request = context.requests.pop().expect("G's genuine child request");
    assert_eq!(parent_request.subject(), parent);
    assert_eq!(
        parent_request.parent_operation(),
        Some(grandparent_request.operation())
    );
    assert!(matches!(
        grandparent_progress,
        DomSourceEventServiceProgress::Retained {
            dispatch: DomEventDispatchOutcome::RetainedSourceOperation(operation)
        } if operation == parent_request.operation()
    ));
    let parent_progress = permission
        .service_event(
            &mut session.dom(),
            &parent_request,
            &mut listeners,
            &mut context,
            service_limits(),
            &cancel,
        )
        .unwrap();
    let child_request = context.requests.pop().expect("P's genuine child request");
    assert_eq!(child_request.subject(), child);
    assert_eq!(
        child_request.parent_operation(),
        Some(parent_request.operation())
    );
    assert!(matches!(
        parent_progress,
        DomSourceEventServiceProgress::Retained {
            dispatch: DomEventDispatchOutcome::RetainedSourceOperation(operation)
        } if operation == child_request.operation()
    ));
    let child_receipt = serviced(
        permission
            .service_event(
                &mut session.dom(),
                &child_request,
                &mut listeners,
                &mut context,
                service_limits(),
                &cancel,
            )
            .unwrap(),
    );
    assert_eq!(child_receipt.callbacks(), 1);
    assert!(child_receipt.failures().is_empty());
    assert!(child_receipt.default_prevented());
    assert_eq!(context.calls, ["G", "P", "C"]);
    assert_eq!(context.edits.len(), 1);
    assert!(context.edits[0].is_ok());
    context.edits.clear();
    let child_operation = child_request.operation();
    let DomSourceDriveProgress::Complete(child_done) = resume(&mut session, child_operation) else {
        panic!("authentically canceled C must reach its producer's normal Complete");
    };
    assert_eq!(child_done.operation(), child_operation);
    assert!(!child_done.cancellation_requested());
    drop(child_done);
    drop(child_receipt);
    drop(child_request);
    let resumed_parent = session.dom().pending_source_edit().unwrap();
    assert_eq!(resumed_parent.operation(), parent_request.operation());
    assert_eq!(
        resumed_parent.parent_operation(),
        Some(grandparent_request.operation())
    );
    let owner = session.dom_manipulation_execution();
    // C produced an intention, not an executable task or a forged transfer.
    // Its actual operation and observations have closed before queue intake.
    assert_eq!(owner.pending_tasks().unwrap(), 0);
    assert_eq!(
        session
            .snapshot()
            .node(details)
            .unwrap()
            .attribute(&"open".into()),
        Some(&"".into())
    );
    let parent_receipt = serviced(
        permission
            .service_event(
                &mut session.dom(),
                &parent_request,
                &mut listeners,
                &mut context,
                service_limits(),
                &cancel,
            )
            .unwrap(),
    );
    assert_eq!(parent_receipt.callbacks(), 1);
    assert!(parent_receipt.failures().is_empty());
    assert!(!parent_receipt.default_prevented());
    drop(parent_receipt);
    match resume(&mut session, parent_request.operation()) {
        DomSourceDriveProgress::Pending {
            request: DomSourceRequest::Dialog(dialog),
            reason: DomSourceUnavailable::Dialog(action),
        } => {
            assert_eq!(dialog.operation(), parent_request.operation());
            assert_eq!(dialog.subject(), parent);
            assert_eq!(dialog.action(), HtmlDialogAction::EstablishCloseWatcher);
            assert_eq!(action, HtmlDialogAction::EstablishCloseWatcher);
        }
        other => panic!(
            "P's authentic queue publication must stop at its actual next service: {other:?}"
        ),
    }
    assert_eq!(owner.pending_tasks().unwrap(), 2);
    let pending = [
        owner.pending_task(0).unwrap().unwrap(),
        owner.pending_task(1).unwrap().unwrap(),
    ];
    let details_task = pending[0]
        .details()
        .expect("C's real Details intention joined first");
    assert_eq!(details_task.id().target(), details);
    assert_eq!(details_task.event().old_state().as_str(), "closed");
    assert_eq!(details_task.event().new_state().as_str(), "open");
    let dialog_task = pending[1]
        .dialog()
        .expect("P's real Dialog task appended second");
    assert_eq!(dialog_task.id().target(), parent);
    assert_eq!(dialog_task.old_state(), HtmlDialogState::Closed);
    assert_eq!(dialog_task.new_state(), HtmlDialogState::Open);
    assert_eq!(dialog_task.source(), None);
    for task in pending {
        let association = task
            .queue_association()
            .expect("actual original hosted association");
        assert_eq!(association.global(), top.global());
        assert_eq!(association.document(), top.document());
    }
    let queued = session.snapshot();
    let tracker = queued
        .node(parent)
        .unwrap()
        .as_dialog()
        .unwrap()
        .toggle_task_tracker()
        .unwrap();
    assert_eq!(tracker.task(), dialog_task.id());
    assert_eq!(tracker.old_state(), HtmlDialogState::Closed);
    assert_eq!(
        queued.node(parent).unwrap().attribute(&"open".into()),
        Some(&"".into())
    );
    assert!(
        !queued
            .node(grandparent)
            .unwrap()
            .as_dialog()
            .unwrap()
            .has_toggle_task_tracker()
    );
    assert!(
        !queued
            .node(child)
            .unwrap()
            .as_dialog()
            .unwrap()
            .has_toggle_task_tracker()
    );
    let parent_status = session.dom().pending_source_edit().unwrap();
    assert_eq!(parent_status.operation(), parent_request.operation());
    assert_eq!(
        parent_status.parent_operation(),
        Some(grandparent_request.operation())
    );
    assert_eq!(parent_status.phase(), DomSourcePhase::DialogService);
    assert_eq!(context.calls, ["G", "P", "C"]);
    assert!(context.requests.is_empty());

    // This is a genuine monotonic readmission of the original lower observer.
    // It proves coexistence/retention, not a measured or one-short byte bound.
    let original_limits = source_limits();
    assert_eq!(original_limits.retained_bytes(), 32_000_000);
    let raised = DomSourceLimits::new(
        64_000_000,
        original_limits.retained_string_code_units(),
        original_limits.work(),
        original_limits.phase_publications(),
        original_limits.output_records(),
        original_limits.published_native_identities(),
        original_limits.nested_operations(),
    );
    permission
        .raise_event_limits(&mut session.dom(), &grandparent_request, raised, &cancel)
        .unwrap();
    assert_eq!(session.snapshot().stamp(), queued.stamp());
    let after_raise = session.dom().pending_source_edit().unwrap();
    assert_eq!(after_raise.operation(), parent_status.operation());
    assert_eq!(
        after_raise.parent_operation(),
        parent_status.parent_operation()
    );
    assert_eq!(after_raise.phase(), DomSourcePhase::DialogService);
    assert_eq!(owner.pending_tasks().unwrap(), 2);
    assert_eq!(
        [
            owner.pending_task(0).unwrap().unwrap(),
            owner.pending_task(1).unwrap().unwrap()
        ],
        pending
    );
    assert_eq!(
        session
            .snapshot()
            .node(parent)
            .unwrap()
            .as_dialog()
            .unwrap()
            .toggle_task_tracker(),
        Some(tracker)
    );
    assert_eq!(context.calls, ["G", "P", "C"]);
    // P remains unfinished and G remains its original retained observer. No
    // ordinary intake, task pump, queue-only root or fabricated Show completion
    // was used; the task bank was created only after C actually completed.
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

#[cfg(any(
    target_os = "macos",
    target_os = "linux",
    target_os = "windows",
    target_os = "android",
    target_os = "ios"
))]
#[test]
fn native_installation_dialog_hook_uses_its_actual_first_published_identity_on_retry() {
    let cancel = AtomicBool::new(false);
    let (mut session, top, _) = hosted_queue_fixture();
    let before = session.snapshot();
    let html = before.node(top.document().node()).unwrap().children()[0];
    let body = before.node(html).unwrap().children()[1];
    let prior_children = before.node(body).unwrap().children().len();
    let mut builder = surgeist_html::HtmlFragment::builder(
        surgeist_html::HtmlConstructionLimits::new(16, 16, 16, 4096, 16),
    )
    .unwrap();
    builder
        .append_element(
            &builder.root(),
            surgeist_html::QualifiedName::new(surgeist_html::Namespace::html(), None, "dialog"),
            surgeist_html::OrderedAttributes::new([surgeist_html::Attribute::new(
                surgeist_html::QualifiedName::new(surgeist_html::Namespace::none(), None, "open"),
                "",
            )])
            .unwrap(),
        )
        .unwrap();
    let fragment = builder.finish();
    let target = DomNativeHtmlInstallTarget::new(
        before.document(top.document()).unwrap().stamp(),
        body,
        None,
    );
    let mut permission = session.source_execution();
    let mut policy = |_: &DomStyleSource| None;
    let progress = permission
        .install_native_html(
            &mut session.dom(),
            target,
            &fragment,
            DomSourceNativeHtmlInstallLimits::new(
                DomNativeHtmlInstallLimits::new(16, 1_000_000, 16_000_000),
                source_limits(),
            ),
            &mut DomSourceServices {
                style_policy: &mut policy,
                url: None,
            },
            &cancel,
        )
        .unwrap();
    let DomSourceDriveProgress::Pending {
        request: DomSourceRequest::Dialog(request),
        reason: DomSourceUnavailable::Dialog(HtmlDialogAction::EstablishCloseWatcher),
    } = progress
    else {
        panic!("the inserted active native Dialog must retain its actual watcher hook");
    };
    let subject = request.subject();
    assert_eq!(request.action(), HtmlDialogAction::EstablishCloseWatcher);
    assert!(before.node(subject).is_err());
    let published = session.snapshot();
    assert_eq!(published.node(subject).unwrap().parent(), Some(body));
    assert_eq!(
        published.node(subject).unwrap().owner_document(),
        Some(top.document())
    );
    assert_eq!(
        published.node(subject).unwrap().attribute(&"open".into()),
        Some(&"".into())
    );
    assert_eq!(
        published.node(body).unwrap().children().len(),
        prior_children + 1
    );
    assert_eq!(
        published.incomplete_source_operation(),
        Some(request.operation())
    );
    assert_eq!(
        session.dom().pending_source_edit().unwrap().phase(),
        DomSourcePhase::DialogService
    );
    assert_eq!(
        session
            .dom_manipulation_execution()
            .pending_tasks()
            .unwrap(),
        0
    );
    assert_eq!(
        pending_dialog_service(
            resume(&mut session, request.operation()),
            subject,
            HtmlDialogAction::EstablishCloseWatcher
        ),
        request.operation()
    );
    let retried = session.snapshot();
    assert_eq!(retried.stamp(), published.stamp());
    assert_eq!(
        retried.node(body).unwrap().children(),
        published.node(body).unwrap().children()
    );
    assert_eq!(
        retried.node(subject).unwrap().attribute(&"open".into()),
        Some(&"".into())
    );
    // This authentic created-key publication proves neither installation
    // completion nor a watcher reply; the original named root remains parked.
}
