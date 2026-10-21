//! Functional native evidence for the standard null-source ShowModal prefix.
//! HTML877d46585–46603 orders the guards, real beforetoggle, open hooks and assertion.
#![cfg(any(
    target_os = "macos",
    target_os = "linux",
    target_os = "windows",
    target_os = "android",
    target_os = "ios"
))]
use std::sync::atomic::AtomicBool;
use surgeist_dom::*;
use surgeist_html::{HtmlDialogAction, HtmlDialogOperation, HtmlDialogState};
use surgeist_runtime::*;

fn cancel() -> AtomicBool {
    AtomicBool::new(false)
}
fn source_limits() -> DomSourceLimits {
    DomSourceLimits::new(64_000_000, 4_000_000, 20_000_000, 128, 8192, 8192, 4)
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
fn fixture() -> (DomSession, DomHostedTopLevel, NodeId, NodeId) {
    fixture_with_profile(false)
}
fn fixture_with_profile(headless: bool) -> (DomSession, DomHostedTopLevel, NodeId, NodeId) {
    let dom = Dom::new(
        DomLimits::new(
            512,
            2_000_000,
            2048,
            2048,
            2048,
            64,
            8_000_000,
            surgeist_dom::DomNameLimits::new(16_777_216, 16_384, 4_000_000, 1_048_576).unwrap(),
        )
        .unwrap()
        .with_dispatch_custody(
            DomDispatchCustodyLimits::new(64, 4096, 32_000_000, 32_000_000, 8_000_000).unwrap(),
        ),
    )
    .unwrap();
    let mut session = DomSession::new(
        dom,
        DomSessionLimits {
            source_events: DomSourceEventRuntimeLimits {
                jobs: 8,
                storage_bytes: 32_000_000,
                work: 4_000_000,
            },
            style: DomStyleRuntimeLimits {
                roots: 8,
                completions: 32,
                reports: 32,
                storage_bytes: 32_000_000,
                work: 4_000_000,
                css_input: surgeist_css::CssUtf16InputLimits::default(),
            },
            element_tasks: DomElementTaskRuntimeLimits {
                tasks: 32,
                storage_bytes: 32_000_000,
                work: 4_000_000,
            },
            hosted: DomHostedRuntimeLimits {
                canonical: DomHostedLimits::new(
                    DomHostedRecordLimits::new(32, 32, 32, 32, 32, 32, 32),
                    32_000_000,
                    32_000_000,
                    4_000_000,
                ),
                tasks: 32,
                storage_bytes: 32_000_000,
                scratch_bytes: 32_000_000,
                work: 4_000_000,
            },
        },
    )
    .unwrap();
    let mut hosted = session.hosted_execution();
    let host = if headless {
        hosted
            .create_headless_host(&mut session.dom(), &cancel())
            .unwrap()
    } else {
        hosted.create_host(&mut session.dom(), &cancel()).unwrap()
    };
    let top = hosted
        .create_top_level(&mut session.dom(), host, &cancel())
        .unwrap();
    let snapshot = session.snapshot();
    let html = snapshot.node(top.document().node()).unwrap().children()[0];
    let body = snapshot.node(html).unwrap().children()[1];
    let mut batch = DomBatch::new();
    let dialog = batch.create_element_ns(top.document(), Namespace::html(), "dialog");
    batch.insert_before(&dialog, body, None);
    let dialog = session
        .dom()
        .apply(batch, &cancel())
        .unwrap()
        .created(&dialog)
        .unwrap()
        .unwrap();
    (session, top, body, dialog)
}
fn destroy_top(session: &mut DomSession, hosted: &mut HostedRuntime, top: DomHostedTopLevel) {
    let queued = hosted
        .destroy_top_level(&mut session.dom(), top.navigable(), &cancel())
        .unwrap();
    let mut last = DomHostedTaskProgress::Idle;
    for _ in 0..32 {
        last = hosted
            .run_next(&mut session.dom(), queued.traversable(), &cancel())
            .unwrap();
        if matches!(last, DomHostedTaskProgress::Complete { operation } if operation == queued.operation())
        {
            break;
        }
    }
    assert!(
        matches!(last, DomHostedTaskProgress::Complete { operation } if operation == queued.operation()),
        "actual top-level destruction did not complete within 32 queue advances: {last:?}"
    );
}
fn apply(session: &mut DomSession, edit: DomSourceEdit) -> DomSourceDriveProgress {
    try_apply(session, edit).unwrap()
}
#[expect(
    clippy::result_large_err,
    reason = "the helper preserves the actual root and original producer failure for exact driving refusal assertions"
)]
fn try_apply(
    session: &mut DomSession,
    edit: DomSourceEdit,
) -> Result<DomSourceDriveProgress, DomSourceDriveError> {
    session.source_execution().apply(
        &mut session.dom(),
        edit,
        source_limits(),
        &mut DomSourceServices {
            meta: None,
            execution_scope: None,
            style_policy: &mut |_| None,
            url: None,
        },
        &cancel(),
    )
}
fn complete(progress: DomSourceDriveProgress) -> DomSourceCompletion {
    match progress {
        DomSourceDriveProgress::Complete(done) => {
            assert!(!done.cancellation_requested());
            done
        }
        other => panic!("actual method/hooks must complete: {other:?}"),
    }
}
fn dispatcher<C>() -> DomEventDispatcher<C> {
    DomEventDispatcher::new(DomEventRegistryLimits {
        registrations: 16,
        pending_events: 8,
        nested_dispatches: 4,
    })
    .unwrap()
}
fn service<C>(
    session: &mut DomSession,
    request: &DomSourceEventRequest,
    registry: &mut DomEventDispatcher<C>,
    context: &mut C,
) -> DomEventDispatchReceipt {
    match session
        .source_execution()
        .service_event(
            &mut session.dom(),
            request,
            registry,
            context,
            DomSourceEventServiceLimits {
                source: source_limits(),
                dispatch: dispatch_limits(),
            },
            &cancel(),
        )
        .unwrap()
    {
        DomSourceEventServiceProgress::Serviced { receipt } => receipt,
        other => panic!("the actual beforetoggle dispatch must finish: {other:?}"),
    }
}
#[expect(
    clippy::result_large_err,
    reason = "the helper preserves the actual root and original producer failure for exact driving refusal assertions"
)]
fn resume(
    session: &mut DomSession,
    operation: DomSourceOperationId,
) -> Result<DomSourceDriveProgress, DomSourceDriveError> {
    session.source_execution().resume(
        &mut session.dom(),
        operation,
        source_limits(),
        &mut DomSourceServices {
            meta: None,
            execution_scope: None,
            style_policy: &mut |_| None,
            url: None,
        },
        &cancel(),
    )
}
fn start(session: &mut DomSession, subject: NodeId) -> DomSourceEventRequest {
    match apply(session, DomSourceEdit::show_modal_dialog(subject)) {
        DomSourceDriveProgress::Pending {
            request: DomSourceRequest::Event(request),
            reason: DomSourceUnavailable::Event,
        } => {
            assert_eq!(request.subject(), subject);
            assert!(request.cancelable());
            let toggle = request.toggle_event().unwrap();
            assert_eq!(toggle.old_state(), HtmlDialogState::Closed);
            assert_eq!(toggle.new_state(), HtmlDialogState::Open);
            assert_eq!(toggle.source(), None);
            request
        }
        other => panic!("standard ShowModal must reach actual beforetoggle: {other:?}"),
    }
}
fn assert_remembered_focus_wait(
    progress: DomSourceDriveProgress,
    operation: DomSourceOperationId,
    subject: NodeId,
) -> DomSourceDialogRequest {
    match progress {
        DomSourceDriveProgress::Pending {
            request: DomSourceRequest::Dialog(request),
            reason: DomSourceUnavailable::Dialog(HtmlDialogAction::RememberFocusedElement),
        } => {
            assert_eq!(request.operation(), operation);
            assert_eq!(request.subject(), subject);
            assert_eq!(request.action(), HtmlDialogAction::RememberFocusedElement);
            request
        }
        other => {
            panic!("the original host's unobserved remembered focus must remain pending: {other:?}")
        }
    }
}
fn assert_open_nonmodal(session: &DomSession, subject: NodeId, watched: bool) {
    let snapshot = session.snapshot();
    let node = snapshot.node(subject).unwrap();
    assert_eq!(node.attribute(&"open".into()), Some(&"".into()));
    let dialog = node.as_dialog().unwrap();
    assert!(!dialog.is_modal());
    assert_eq!(dialog.has_close_watcher(), watched);
    assert_eq!(
        node.as_element()
            .unwrap()
            .common_state()
            .html()
            .unwrap()
            .previously_focused_element(),
        None
    );
}
fn assert_open_modal(session: &DomSession, subject: NodeId) {
    let snapshot = session.snapshot();
    let node = snapshot.node(subject).unwrap();
    assert_eq!(node.attribute(&"open".into()), Some(&"".into()));
    let dialog = node.as_dialog().unwrap();
    assert!(dialog.is_modal());
    assert!(dialog.has_close_watcher());
    assert_eq!(
        node.as_element()
            .unwrap()
            .common_state()
            .html()
            .unwrap()
            .previously_focused_element(),
        None
    );
}
fn toggle_task(session: &DomSession, subject: NodeId, top: DomHostedTopLevel) -> DomElementTask {
    let tasks = session.element_task_execution();
    let task = (0..tasks
        .pending_tasks(DomElementTaskSourceKind::DomManipulation)
        .unwrap())
        .map(|index| {
            tasks
                .pending_task(DomElementTaskSourceKind::DomManipulation, index)
                .unwrap()
                .unwrap()
        })
        .find(|task| {
            task.dialog_toggle()
                .is_some_and(|toggle| toggle.id().target() == subject)
        })
        .unwrap();
    let toggle = task.dialog_toggle().unwrap();
    assert_eq!(toggle.id().target(), subject);
    assert_eq!(toggle.old_state(), HtmlDialogState::Closed);
    assert_eq!(toggle.new_state(), HtmlDialogState::Open);
    assert_eq!(toggle.source(), None);
    assert_eq!(task.queue_association().unwrap().global(), top.global());
    assert_eq!(task.queue_association().unwrap().document(), top.document());
    assert_eq!(
        session
            .snapshot()
            .node(subject)
            .unwrap()
            .as_dialog()
            .unwrap()
            .toggle_task_tracker()
            .unwrap()
            .task(),
        task.id()
    );
    task
}

#[test]
fn active_show_modal_publishes_structural_phase_then_waits_for_unobserved_remembered_focus() {
    let (mut session, top, _, subject) = fixture();
    drop(complete(apply(
        &mut session,
        DomSourceEdit::set_attribute(subject, "popover", "manual"),
    )));
    assert!(
        session
            .snapshot()
            .node(subject)
            .unwrap()
            .as_element()
            .unwrap()
            .common_state()
            .html()
            .unwrap()
            .popover_is_hidden()
    );
    let mut registry = dispatcher::<usize>();
    registry
        .add_listener(
            &mut session.dom(),
            subject,
            DomEventType::BeforeToggle,
            DomEventCallback::new(|_, _, calls: &mut usize, event| {
                *calls += 1;
                assert!(event.cancelable());
                Ok(())
            }),
            DomEventListenerOptions::default(),
        )
        .unwrap();
    let request = start(&mut session, subject);
    let mut calls = 0;
    let receipt = service(&mut session, &request, &mut registry, &mut calls);
    assert!(!receipt.default_prevented());
    assert!(receipt.failures().is_empty());
    assert_eq!(receipt.callbacks(), 1);
    assert_eq!(
        session
            .snapshot()
            .node(subject)
            .unwrap()
            .attribute(&"open".into()),
        None
    );
    drop(receipt);
    let pending = assert_remembered_focus_wait(
        resume(&mut session, request.operation()).unwrap(),
        request.operation(),
        subject,
    );
    assert_open_modal(&session, subject);
    assert!(matches!(
        session
            .snapshot()
            .document(top.document())
            .unwrap()
            .top_layer(),
        Err(DomError::IncompleteSourceOperation)
    ));
    let watcher = session
        .snapshot()
        .node(subject)
        .unwrap()
        .as_dialog()
        .unwrap()
        .close_watcher()
        .unwrap();
    assert_eq!(watcher.global(), top.global());
    assert_eq!(watcher.window_document(), top.document());
    assert_eq!(watcher.subject(), subject);
    let task = toggle_task(&session, subject, top);
    assert_eq!(
        session
            .element_task_execution()
            .pending_total_tasks()
            .unwrap(),
        1
    );
    assert_eq!(
        session.snapshot().incomplete_source_operation(),
        Some(request.operation())
    );
    let stamp = session.snapshot().stamp();
    drop(session.source_execution());
    assert_eq!(
        assert_remembered_focus_wait(
            resume(&mut session, request.operation()).unwrap(),
            request.operation(),
            subject
        ),
        pending
    );
    assert_eq!(session.snapshot().stamp(), stamp);
    assert_eq!(toggle_task(&session, subject, top), task);
    assert_eq!(calls, 1);
    assert_open_modal(&session, subject);
}

#[test]
fn canceled_or_post_event_disconnected_show_modal_completes_without_open_or_tasks() {
    for disconnect in [false, true] {
        let (mut session, _, _, subject) = fixture();
        let mut registry = dispatcher::<usize>();
        registry
            .add_listener(
                &mut session.dom(),
                subject,
                DomEventType::BeforeToggle,
                DomEventCallback::new(move |dom, _, calls: &mut usize, event| {
                    *calls += 1;
                    if disconnect {
                        let mut batch = DomBatch::new();
                        batch.detach(subject);
                        dom.apply(batch, &cancel()).unwrap();
                    } else {
                        event.prevent_default();
                    }
                    Ok(())
                }),
                DomEventListenerOptions::default(),
            )
            .unwrap();
        let request = start(&mut session, subject);
        let mut calls = 0;
        let receipt = service(&mut session, &request, &mut registry, &mut calls);
        assert_eq!(receipt.default_prevented(), !disconnect);
        assert!(receipt.failures().is_empty());
        drop(receipt);
        let done = complete(resume(&mut session, request.operation()).unwrap());
        assert_eq!(done.operation(), request.operation());
        drop(done);
        let snapshot = session.snapshot();
        let node = snapshot.node(subject).unwrap();
        assert_eq!(node.attribute(&"open".into()), None);
        assert!(!node.as_dialog().unwrap().is_modal());
        assert!(!node.as_dialog().unwrap().has_close_watcher());
        assert_eq!(node.parent().is_none(), disconnect);
        assert_eq!(calls, 1);
        assert!(session.dom().pending_source_edit().is_none());
        assert_eq!(snapshot.incomplete_source_operation(), None);
        assert_eq!(
            session
                .element_task_execution()
                .pending_total_tasks()
                .unwrap(),
            0
        );
    }
}

struct InactiveTarget {
    session: DomSession,
    a: DomHostedTopLevel,
    b: DomHostedTopLevel,
    body_a: NodeId,
    body_b: NodeId,
    subject: NodeId,
    sentinel: NodeId,
    sentinel_watcher: DomCloseWatcherId,
}
fn inactive_target() -> InactiveTarget {
    let (mut session, a, body_a, subject) = fixture_with_profile(true);
    let mut hosted = session.hosted_execution();
    let b = hosted
        .create_top_level(&mut session.dom(), a.host(), &cancel())
        .unwrap();
    let snapshot = session.snapshot();
    let html = snapshot.node(b.document().node()).unwrap().children()[0];
    let body_b = snapshot.node(html).unwrap().children()[1];
    let mut batch = DomBatch::new();
    let sentinel = batch.create_element_ns(b.document(), Namespace::html(), "dialog");
    batch.insert_before(&sentinel, body_b, None);
    let sentinel = session
        .dom()
        .apply(batch, &cancel())
        .unwrap()
        .created(&sentinel)
        .unwrap()
        .unwrap();
    drop(complete(apply(
        &mut session,
        DomSourceEdit::set_attribute(sentinel, "open", ""),
    )));
    let sentinel_watcher = session
        .snapshot()
        .node(sentinel)
        .unwrap()
        .as_dialog()
        .unwrap()
        .close_watcher()
        .unwrap();
    destroy_top(&mut session, &mut hosted, b);
    assert!(
        !session
            .snapshot()
            .document(b.document())
            .unwrap()
            .is_fully_active()
            .unwrap()
    );
    assert!(
        session
            .snapshot()
            .document(a.document())
            .unwrap()
            .is_fully_active()
            .unwrap()
    );
    assert!(matches!(
        hosted.settings(&session.dom(), b.global(), &cancel()),
        Err(DomHostedRuntimeError::Dom(DomHostedError::StaleIdentity))
    ));
    InactiveTarget {
        session,
        a,
        b,
        body_a,
        body_b,
        subject,
        sentinel,
        sentinel_watcher,
    }
}

#[test]
fn callback_adoption_into_genuinely_inactive_document_keeps_open_and_queue_then_fails_assertion() {
    let InactiveTarget {
        mut session,
        a,
        b,
        body_a,
        body_b,
        subject,
        sentinel,
        sentinel_watcher,
    } = inactive_target();
    let unrelated = seed_unrelated_close(&mut session, a, body_a);
    let mut registry = dispatcher::<usize>();
    registry
        .add_listener(
            &mut session.dom(),
            subject,
            DomEventType::BeforeToggle,
            DomEventCallback::new(move |dom, _, calls: &mut usize, event| {
                *calls += 1;
                assert_eq!(
                    event.composed_path().last(),
                    Some(DomEventTarget::Global(a.global()))
                );
                let mut batch = DomBatch::new();
                batch.insert_before(subject, body_b, None);
                dom.apply(batch, &cancel()).unwrap();
                Ok(())
            }),
            DomEventListenerOptions::default(),
        )
        .unwrap();
    let request = start(&mut session, subject);
    let mut calls = 0;
    let receipt = service(&mut session, &request, &mut registry, &mut calls);
    assert!(!receipt.default_prevented());
    assert!(receipt.failures().is_empty());
    drop(receipt);
    let terminal = resume(&mut session, request.operation());
    let failure = match terminal {
        Err(surgeist_runtime::DomSourceDriveError::Source(DomSourceRuntimeError::Dom(
            DomError::SourceInvariantFailed { operation, failure },
        ))) => {
            assert_eq!(operation, request.operation());
            failure
        }
        other => panic!(
            "proven missing watcher must terminate with its native invariant failure: {other:?}"
        ),
    };
    assert_eq!(failure.dialog_operation(), HtmlDialogOperation::ShowModal);
    assert_eq!(failure.subject(), subject);
    assert_eq!(
        failure.action(),
        HtmlDialogAction::AssertCloseWatcherPresent
    );
    assert_eq!(
        failure.reason(),
        DomSourceInvariantReason::MissingDialogCloseWatcher
    );
    // Failure names the published observation, before the terminal release may
    // publish a later revision. The callback itself had not added open.
    assert_eq!(failure.basis().owner(), session.snapshot().owner());
    assert_open_nonmodal(&session, subject, false);
    assert_eq!(
        session.snapshot().node(subject).unwrap().owner_document(),
        Some(b.document())
    );
    assert_eq!(
        session.snapshot().node(subject).unwrap().parent(),
        Some(body_b)
    );
    assert_eq!(
        session
            .snapshot()
            .node(sentinel)
            .unwrap()
            .as_dialog()
            .unwrap()
            .close_watcher(),
        Some(sentinel_watcher)
    );
    let task = toggle_task(&session, subject, a);
    assert_eq!(
        session
            .element_task_execution()
            .pending_total_tasks()
            .unwrap(),
        3
    );
    assert_eq!(
        session
            .element_task_execution()
            .pending_task(DomElementTaskSourceKind::UserInteraction, 0)
            .unwrap(),
        Some(unrelated)
    );
    assert!(session.dom().pending_source_edit().is_none());
    assert_eq!(session.snapshot().incomplete_source_operation(), None);
    assert_eq!(calls, 1);
    let replay = resume(&mut session, request.operation());
    assert!(
        matches!(
            replay,
            Err(surgeist_runtime::DomSourceDriveError::Source(
                DomSourceRuntimeError::Dom(DomError::InvalidSourceOperation)
            ))
        ),
        "failed method must not retry its assertion: {replay:?}"
    );
    assert_eq!(toggle_task(&session, subject, a), task);
    let next = try_apply(&mut session, DomSourceEdit::show_modal_dialog(subject));
    assert!(
        matches!(
            next,
            Err(surgeist_runtime::DomSourceDriveError::Source(
                DomSourceRuntimeError::Dom(DomError::DialogInvalidState)
            ))
        ),
        "new invocation observes retained open/nonmodal state: {next:?}"
    );
    assert_eq!(calls, 1);
    assert_eq!(toggle_task(&session, subject, a), task);
    // A later independent Source edit may establish a watcher, but cannot
    // revive the terminal failed method or consume its independent task.
    drop(complete(apply(
        &mut session,
        DomSourceEdit::insert_before(subject, body_a, None),
    )));
    assert_open_nonmodal(&session, subject, true);
    assert_eq!(toggle_task(&session, subject, a), task);
    assert_eq!(
        session
            .element_task_execution()
            .pending_task(DomElementTaskSourceKind::UserInteraction, 0)
            .unwrap(),
        Some(unrelated)
    );
}

fn seed_unrelated_close(
    session: &mut DomSession,
    top: DomHostedTopLevel,
    body: NodeId,
) -> DomElementTask {
    let mut batch = DomBatch::new();
    let other = batch.create_element_ns(top.document(), Namespace::html(), "dialog");
    batch.insert_before(&other, body, None);
    let other = session
        .dom()
        .apply(batch, &cancel())
        .unwrap()
        .created(&other)
        .unwrap()
        .unwrap();
    drop(complete(apply(
        session,
        DomSourceEdit::set_attribute(other, "open", ""),
    )));
    let event = match apply(session, DomSourceEdit::close_dialog(other, None)) {
        DomSourceDriveProgress::Pending {
            request: DomSourceRequest::Event(event),
            ..
        } => event,
        other => panic!("unrelated real Close must dispatch before queueing: {other:?}"),
    };
    let mut registry = dispatcher::<()>();
    drop(service(session, &event, &mut registry, &mut ()));
    drop(complete(resume(session, event.operation()).unwrap()));
    let task = session
        .element_task_execution()
        .pending_task(DomElementTaskSourceKind::UserInteraction, 0)
        .unwrap()
        .unwrap();
    assert!(task.dialog_close().is_some());
    assert_eq!(task.queue_association().unwrap().document(), top.document());
    assert_eq!(
        session
            .element_task_execution()
            .pending_total_tasks()
            .unwrap(),
        2
    );
    task
}

#[test]
fn started_beforetoggle_finishes_late_once_after_actual_receiver_retirement() {
    for prevent in [false, true] {
        let (mut session, top, _, subject) = fixture();
        let mut registry = dispatcher::<Vec<&'static str>>();
        registry
            .add_listener(
                &mut session.dom(),
                subject,
                DomEventType::BeforeToggle,
                DomEventCallback::new(move |dom, _, calls: &mut Vec<&'static str>, event| {
                    calls.push("retire");
                    if prevent {
                        event.prevent_default();
                    }
                    let marker = dom.snapshot().incomplete_source_operation();
                    let mut batch = DomBatch::new();
                    batch.detach(subject);
                    batch.destroy(subject);
                    dom.apply(batch, &cancel()).unwrap();
                    assert_eq!(dom.snapshot().incomplete_source_operation(), marker);
                    // This started event precedes QueueToggle and the first
                    // ShowModal prefix publication; retirement must preserve
                    // the actual marker, without inventing a published parent.
                    Ok(())
                }),
                DomEventListenerOptions::default(),
            )
            .unwrap();
        let late = registry
            .add_listener(
                &mut session.dom(),
                subject,
                DomEventType::BeforeToggle,
                DomEventCallback::new(move |_, _, calls: &mut Vec<&'static str>, event| {
                    calls.push("late");
                    assert_eq!(event.target(), Some(subject));
                    assert_eq!(event.default_prevented(), prevent);
                    Ok(())
                }),
                DomEventListenerOptions {
                    capture: false,
                    once: true,
                },
            )
            .unwrap();
        let request = start(&mut session, subject);
        let mut calls = Vec::new();
        let receipt = service(&mut session, &request, &mut registry, &mut calls);
        assert_eq!(receipt.callbacks(), 2);
        assert_eq!(receipt.default_prevented(), prevent);
        assert!(receipt.failures().is_empty());
        assert_eq!(calls, ["retire", "late"]);
        assert!(!registry.remove_listener(late));
        assert!(matches!(
            session.snapshot().node(subject),
            Err(DomError::StaleNode)
        ));
        drop(receipt);
        let terminal = resume(&mut session, request.operation());
        if prevent {
            drop(complete(terminal.unwrap()));
        } else {
            assert!(
                matches!(&terminal,Err(surgeist_runtime::DomSourceDriveError::Source(DomSourceRuntimeError::Dom(DomError::SourceReceiverRetired {operation,subject:actual,action:HtmlDialogAction::ReadOpen}))) if *operation==request.operation() && *actual==subject),
                "required exact retired receiver read: {terminal:?}"
            );
        }
        assert!(session.dom().pending_source_edit().is_none());
        assert_eq!(session.snapshot().incomplete_source_operation(), None);
        assert_eq!(
            session
                .element_task_execution()
                .pending_total_tasks()
                .unwrap(),
            0
        );
        assert_eq!(calls, ["retire", "late"]);
        let mut hosted = session.hosted_execution();
        destroy_top(&mut session, &mut hosted, top);
        hosted
            .destroy_host(&mut session.dom(), top.host(), &cancel())
            .unwrap();
        assert!(matches!(
            session.try_shutdown(),
            DomShutdownOutcome::Complete
        ));
    }
}

#[test]
fn canceled_and_callback_refused_service_retain_actual_dispatch_then_retry_without_replay() {
    let (mut session, top, _, subject) = fixture();
    let mut registry = dispatcher::<usize>();
    registry
        .add_listener(
            &mut session.dom(),
            subject,
            DomEventType::BeforeToggle,
            DomEventCallback::new(|_, _, calls: &mut usize, _| {
                *calls += 1;
                Ok(())
            }),
            DomEventListenerOptions {
                capture: false,
                once: true,
            },
        )
        .unwrap();
    let request = start(&mut session, subject);
    let mut calls = 0;
    let source = session.source_execution();
    let (mut other, _, _, _) = fixture();
    let wrong = source.service_event(
        &mut other.dom(),
        &request,
        &mut registry,
        &mut calls,
        DomSourceEventServiceLimits {
            source: source_limits(),
            dispatch: dispatch_limits(),
        },
        &cancel(),
    );
    assert!(wrong.is_err());
    assert_eq!(calls, 0);
    let canceled = source.service_event(
        &mut session.dom(),
        &request,
        &mut registry,
        &mut calls,
        DomSourceEventServiceLimits {
            source: source_limits(),
            dispatch: dispatch_limits(),
        },
        &AtomicBool::new(true),
    );
    assert!(canceled.is_err());
    assert_eq!(calls, 0);
    let mut bounded = dispatch_limits();
    bounded.callbacks = 0;
    let retained = source
        .service_event(
            &mut session.dom(),
            &request,
            &mut registry,
            &mut calls,
            DomSourceEventServiceLimits {
                source: source_limits(),
                dispatch: bounded,
            },
            &cancel(),
        )
        .unwrap();
    assert!(
        matches!(retained, DomSourceEventServiceProgress::Retained { .. }),
        "actual dispatcher must retain its cursor: {retained:?}"
    );
    assert_eq!(calls, 0);
    drop(source);
    let receipt = service(&mut session, &request, &mut registry, &mut calls);
    assert_eq!(receipt.callbacks(), 1);
    assert!(receipt.failures().is_empty());
    drop(receipt);
    assert_eq!(calls, 1);
    // An already accepted event is not a second dispatch authorization.
    let replay = session.source_execution().service_event(
        &mut session.dom(),
        &request,
        &mut registry,
        &mut calls,
        DomSourceEventServiceLimits {
            source: source_limits(),
            dispatch: dispatch_limits(),
        },
        &cancel(),
    );
    assert!(replay.is_err());
    assert_eq!(calls, 1);
    let stamp = session.snapshot().stamp();
    let canceled = session.source_execution().resume(
        &mut session.dom(),
        request.operation(),
        source_limits(),
        &mut DomSourceServices {
            meta: None,
            execution_scope: None,
            style_policy: &mut |_| None,
            url: None,
        },
        &AtomicBool::new(true),
    );
    assert!(canceled.is_err());
    assert_eq!(session.snapshot().stamp(), stamp);
    let tight = DomSourceLimits::new(64_000_000, 4_000_000, 0, 128, 8192, 8192, 4);
    let refused = session.source_execution().resume(
        &mut session.dom(),
        request.operation(),
        tight,
        &mut DomSourceServices {
            meta: None,
            execution_scope: None,
            style_policy: &mut |_| None,
            url: None,
        },
        &cancel(),
    );
    assert!(refused.is_err());
    assert_eq!(calls, 1);
    assert_remembered_focus_wait(
        resume(&mut session, request.operation()).unwrap(),
        request.operation(),
        subject,
    );
    assert_open_modal(&session, subject);
    toggle_task(&session, subject, top);
    assert_eq!(
        session
            .element_task_execution()
            .pending_total_tasks()
            .unwrap(),
        1
    );
    assert_eq!(calls, 1);
}

#[test]
fn initial_open_inactive_and_disconnected_guards_do_not_dispatch_or_queue() {
    for case in 0..3 {
        let (mut session, top, _, subject) = fixture();
        match case {
            0 => drop(complete(apply(
                &mut session,
                DomSourceEdit::set_attribute(subject, "open", ""),
            ))),
            1 => {
                let mut hosted = session.hosted_execution();
                destroy_top(&mut session, &mut hosted, top);
            }
            _ => {
                let mut batch = DomBatch::new();
                batch.detach(subject);
                session.dom().apply(batch, &cancel()).unwrap();
            }
        }
        let result = try_apply(&mut session, DomSourceEdit::show_modal_dialog(subject));
        assert!(
            matches!(
                result,
                Err(surgeist_runtime::DomSourceDriveError::Source(
                    DomSourceRuntimeError::Dom(DomError::DialogInvalidState)
                ))
            ),
            "literal initial ShowModal guard must reject: {result:?}"
        );
        assert!(session.dom().pending_source_edit().is_none());
        assert_eq!(
            session
                .element_task_execution()
                .pending_total_tasks()
                .unwrap(),
            0
        );
        assert!(
            !session
                .snapshot()
                .node(subject)
                .unwrap()
                .as_dialog()
                .unwrap()
                .is_modal()
        );
    }
}
