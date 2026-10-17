//! Native standard requestClose: real cancel dispatch, original watcher custody,
//! nested modal/nonmodal Close and literal final force reset (HTML877d46708–46723).
#![cfg(any(
    target_os = "macos",
    target_os = "linux",
    target_os = "windows",
    target_os = "android",
    target_os = "ios"
))]
use std::sync::atomic::AtomicBool;
use surgeist_dom::*;
use surgeist_html::{HtmlDialogAction, HtmlDialogState};
use surgeist_runtime::*;

#[path = "dom_source_request_close/modal.rs"]
mod modal;

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
fn run_limits() -> DomElementTaskRunLimits {
    DomElementTaskRunLimits {
        source: source_limits(),
        dispatch: dispatch_limits(),
    }
}
fn fixture() -> (DomSession, DomHostedTopLevel, NodeId, NodeId) {
    fixture_with_profile(false)
}
fn fixture_with_profile(headless: bool) -> (DomSession, DomHostedTopLevel, NodeId, NodeId) {
    let dom = Dom::new(
        DomLimits::new(512, 2_000_000, 2048, 2048, 2048, 64, 8_000_000)
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
    session
        .source_execution()
        .apply(
            &mut session.dom(),
            edit,
            source_limits(),
            &mut DomSourceServices {
                execution_scope: None,
                style_policy: &mut |_| None,
                url: None,
            },
            &cancel(),
        )
        .unwrap()
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
fn open(session: &mut DomSession, subject: NodeId) {
    drop(complete(apply(
        session,
        DomSourceEdit::set_attribute(subject, "open", ""),
    )));
    let snapshot = session.snapshot();
    assert_eq!(
        snapshot.node(subject).unwrap().attribute(&"open".into()),
        Some(&"".into())
    );
    assert!(
        snapshot
            .node(subject)
            .unwrap()
            .as_dialog()
            .unwrap()
            .has_close_watcher()
    );
}
fn seed_return_value(session: &mut DomSession, subject: NodeId, value: &DocumentString) {
    drop(complete(apply(
        session,
        DomSourceEdit::set_dialog_return_value(subject, value.clone()),
    )));
}
fn pending_event(
    progress: DomSourceDriveProgress,
    kind: DomSourceEventKind,
) -> DomSourceEventRequest {
    match progress {
        DomSourceDriveProgress::Pending {
            request: DomSourceRequest::Event(request),
            reason: DomSourceUnavailable::Event,
        } => {
            assert_eq!(request.kind(), kind);
            request
        }
        other => panic!("expected original {kind:?} event, got {other:?}"),
    }
}
fn start_request(
    session: &mut DomSession,
    subject: NodeId,
    result: Option<DocumentString>,
) -> DomSourceEventRequest {
    let request = pending_event(
        apply(
            session,
            DomSourceEdit::request_close_dialog(subject, result),
        ),
        DomSourceEventKind::Cancel,
    );
    assert!(request.cancelable());
    assert!(request.toggle_event().is_none());
    request
}
fn resume(
    session: &mut DomSession,
    operation: DomSourceOperationId,
) -> Result<DomSourceDriveProgress, DomSourceRuntimeError> {
    session.source_execution().resume(
        &mut session.dom(),
        operation,
        source_limits(),
        &mut DomSourceServices {
            execution_scope: None,
            style_policy: &mut |_| None,
            url: None,
        },
        &cancel(),
    )
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
        other => panic!("original dispatch must finish: {other:?}"),
    }
}
fn finish_nested_close<C>(
    session: &mut DomSession,
    operation: DomSourceOperationId,
    registry: &mut DomEventDispatcher<C>,
    context: &mut C,
) {
    let before = pending_event(
        resume(session, operation).unwrap(),
        DomSourceEventKind::BeforeToggle,
    );
    assert!(!before.cancelable());
    let payload = before.toggle_event().unwrap();
    assert_eq!(payload.old_state(), HtmlDialogState::Open);
    assert_eq!(payload.new_state(), HtmlDialogState::Closed);
    let receipt = service(session, &before, registry, context);
    assert!(!receipt.default_prevented());
    assert!(receipt.failures().is_empty());
    drop(complete(resume(session, operation).unwrap()));
}
fn assert_surviving_reset(session: &DomSession, subject: NodeId, result: Option<&DocumentString>) {
    let snapshot = session.snapshot();
    assert_eq!(snapshot.incomplete_source_operation(), None);
    let state = snapshot.node(subject).unwrap().as_dialog().unwrap();
    assert!(!state.force_close_watcher_enabled());
    assert_eq!(state.request_close_return_value(), result);
    assert_eq!(state.request_close_source(), None);
}
#[test]
fn closed_and_detached_receivers_return_before_request_state_or_dispatch() {
    let (mut session, _, _, subject) = fixture();
    let result = DocumentString::from("unreached");
    drop(complete(apply(
        &mut session,
        DomSourceEdit::request_close_dialog(subject, Some(result.clone())),
    )));
    assert_surviving_reset(&session, subject, None);
    open(&mut session, subject);
    drop(complete(apply(
        &mut session,
        DomSourceEdit::detach(subject),
    )));
    drop(complete(apply(
        &mut session,
        DomSourceEdit::request_close_dialog(subject, Some(result)),
    )));
    assert_surviving_reset(&session, subject, None);
    assert!(session.dom().pending_source_edit().is_none());
}
#[test]
fn authentic_uncanceled_request_closes_with_omitted_empty_and_supplied_results_and_real_queues() {
    for result in [
        None,
        Some(DocumentString::from("")),
        Some(DocumentString::from("actual result")),
    ] {
        let (mut session, _, _, subject) = fixture();
        let seed = DocumentString::from("seed");
        seed_return_value(&mut session, subject, &seed);
        open(&mut session, subject);
        let mut registry = dispatcher::<()>();
        registry
            .add_listener(
                &session.dom(),
                subject,
                DomEventType::Cancel,
                DomEventCallback::new(|_, _, _, event| {
                    assert_eq!(event.event_type(), DomEventType::Cancel);
                    assert!(event.cancelable());
                    assert!(event.toggle_event().is_none());
                    assert!(!event.bubbles());
                    assert!(!event.composed());
                    Ok(())
                }),
                DomEventListenerOptions::default(),
            )
            .unwrap();
        let request = start_request(&mut session, subject, result.clone());
        let receipt = service(&mut session, &request, &mut registry, &mut ());
        assert_eq!(receipt.callbacks(), 1);
        assert!(!receipt.default_prevented());
        assert!(receipt.failures().is_empty());
        finish_nested_close(&mut session, request.operation(), &mut registry, &mut ());
        let snapshot = session.snapshot();
        let state = snapshot.node(subject).unwrap().as_dialog().unwrap();
        assert_eq!(state.return_value(), result.as_ref().unwrap_or(&seed));
        assert!(!state.has_close_watcher());
        assert_eq!(
            snapshot.node(subject).unwrap().attribute(&"open".into()),
            None
        );
        assert_surviving_reset(&session, subject, None);
        assert_eq!(
            session
                .element_task_execution()
                .pending_total_tasks()
                .unwrap(),
            2
        );
        assert!(session.dom().pending_source_edit().is_none());
    }
}
#[test]
fn canceled_cancel_keeps_open_and_real_original_guard_is_reset_for_the_next_request() {
    let (mut session, _, _, subject) = fixture();
    open(&mut session, subject);
    let original = session
        .snapshot()
        .node(subject)
        .unwrap()
        .as_dialog()
        .unwrap()
        .close_watcher();
    let mut registry = dispatcher::<usize>();
    registry
        .add_listener(
            &session.dom(),
            subject,
            DomEventType::Cancel,
            DomEventCallback::new(|_, _, calls: &mut usize, event| {
                *calls += 1;
                if *calls == 1 {
                    event.prevent_default();
                }
                Ok(())
            }),
            DomEventListenerOptions::default(),
        )
        .unwrap();
    let result = DocumentString::from("canceled request");
    let request = start_request(&mut session, subject, Some(result.clone()));
    let mut calls = 0;
    let receipt = service(&mut session, &request, &mut registry, &mut calls);
    assert!(receipt.default_prevented());
    drop(complete(resume(&mut session, request.operation()).unwrap()));
    let snapshot = session.snapshot();
    assert_eq!(
        snapshot.node(subject).unwrap().attribute(&"open".into()),
        Some(&"".into())
    );
    assert_eq!(
        snapshot
            .node(subject)
            .unwrap()
            .as_dialog()
            .unwrap()
            .close_watcher(),
        original
    );
    assert_eq!(
        session
            .element_task_execution()
            .pending_total_tasks()
            .unwrap(),
        0
    );
    assert_surviving_reset(&session, subject, Some(&result));
    let next = start_request(&mut session, subject, None);
    assert!(!service(&mut session, &next, &mut registry, &mut calls).default_prevented());
    finish_nested_close(&mut session, next.operation(), &mut registry, &mut calls);
    assert_eq!(calls, 2);
}
#[derive(Clone, Copy)]
enum Mutation {
    Reenter,
    RemoveReopen,
    Remove,
    Detach,
    Adopt(NodeId),
    FailedRetire,
    Retire,
}
struct MutationContext {
    source: DomSourceRuntime,
    mutation: Mutation,
    calls: Vec<&'static str>,
    nested: Vec<DomSourceCompletion>,
    destruction: Option<Result<DomCommit, DomError>>,
    prevent: bool,
    latest: DocumentString,
}
fn mutation_context(session: &DomSession, mutation: Mutation, prevent: bool) -> MutationContext {
    MutationContext {
        source: session.source_execution(),
        mutation,
        calls: Vec::new(),
        nested: Vec::new(),
        destruction: None,
        prevent,
        latest: "latest request".into(),
    }
}
fn nested(context: &mut MutationContext, dom: &mut DomEventDom<'_>, edit: DomSourceEdit) {
    context.nested.push(complete(
        context
            .source
            .apply(
                dom,
                edit,
                source_limits(),
                &mut DomSourceServices {
                    execution_scope: None,
                    style_policy: &mut |_| None,
                    url: None,
                },
                &cancel(),
            )
            .unwrap(),
    ));
}
fn mutation_registry(
    session: &mut DomSession,
    subject: NodeId,
) -> DomEventDispatcher<MutationContext> {
    let mut registry = dispatcher();
    registry
        .add_listener(
            &session.dom(),
            subject,
            DomEventType::Cancel,
            DomEventCallback::new(move |dom, _, context: &mut MutationContext, event| {
                context.calls.push("first");
                if context.prevent {
                    event.prevent_default();
                }
                let parent = dom.snapshot().incomplete_source_operation();
                assert!(
                    parent.is_some(),
                    "request fields were published before actual Cancel"
                );
                let document = dom
                    .snapshot()
                    .node(subject)
                    .unwrap()
                    .owner_document()
                    .unwrap();
                match context.mutation {
                    Mutation::Reenter => {
                        let latest = context.latest.clone();
                        nested(
                            context,
                            dom,
                            DomSourceEdit::request_close_dialog(subject, Some(latest)),
                        );
                    }
                    Mutation::RemoveReopen => {
                        nested(
                            context,
                            dom,
                            DomSourceEdit::remove_attribute(subject, "open"),
                        );
                        nested(
                            context,
                            dom,
                            DomSourceEdit::set_attribute(subject, "open", ""),
                        );
                    }
                    Mutation::Remove | Mutation::Retire => nested(
                        context,
                        dom,
                        DomSourceEdit::remove_attribute(subject, "open"),
                    ),
                    Mutation::Detach => nested(context, dom, DomSourceEdit::detach(subject)),
                    Mutation::Adopt(parent) => nested(
                        context,
                        dom,
                        DomSourceEdit::insert_before(subject, parent, None),
                    ),
                    Mutation::FailedRetire => {}
                }
                assert_eq!(
                    dom.snapshot().incomplete_source_operation(),
                    parent,
                    "completed child Source work preserves its actual published parent marker"
                );
                assert!(matches!(
                    dom.snapshot().html_capture(
                        document,
                        None,
                        surgeist_html_semantics::HtmlCaptureLimits::new(
                            128, 512, 512, 100_000, 128, 32, 32
                        ),
                        DomHtmlCaptureLimits::new(128, 1_000_000),
                        &cancel()
                    ),
                    Err(DomHtmlCaptureError::Source(
                        DomError::IncompleteSourceOperation
                    ))
                ));
                if matches!(context.mutation, Mutation::Retire | Mutation::FailedRetire) {
                    let mut batch = DomBatch::new();
                    batch.detach(subject);
                    batch.destroy(subject);
                    context.destruction = Some(dom.apply(
                        batch,
                        &AtomicBool::new(matches!(context.mutation, Mutation::FailedRetire)),
                    ));
                    if context.destruction.as_ref().unwrap().is_ok() {
                        let after = dom.snapshot();
                        assert_eq!(
                            after.incomplete_source_operation(),
                            parent,
                            "ordinary destruction preserves its actual published Source parent"
                        );
                        assert!(matches!(
                            after.document(document).unwrap().is_fully_active(),
                            Err(DomHostedError::Dom(DomError::IncompleteSourceOperation))
                        ));
                    }
                }
                Ok(())
            }),
            DomEventListenerOptions::default(),
        )
        .unwrap();
    registry
}
#[test]
fn reentrant_request_retains_latest_fields_and_literal_false_blocks_outer_auto_or_none_close() {
    for closedby in [None, Some("none")] {
        let (mut session, _, _, subject) = fixture();
        if let Some(closedby) = closedby {
            drop(complete(apply(
                &mut session,
                DomSourceEdit::set_attribute(subject, "closedby", closedby),
            )));
        }
        open(&mut session, subject);
        let mut context = mutation_context(&session, Mutation::Reenter, false);
        let mut registry = mutation_registry(&mut session, subject);
        let request = start_request(&mut session, subject, Some("outer".into()));
        let receipt = service(&mut session, &request, &mut registry, &mut context);
        assert_eq!(receipt.callbacks(), 1);
        assert!(receipt.failures().is_empty());
        assert!(!receipt.default_prevented());
        drop(complete(resume(&mut session, request.operation()).unwrap()));
        assert_eq!(context.calls, ["first"]);
        assert_eq!(context.nested.len(), 1);
        assert_surviving_reset(&session, subject, Some(&context.latest));
        assert_eq!(
            session
                .snapshot()
                .node(subject)
                .unwrap()
                .attribute(&"open".into()),
            Some(&"".into())
        );
        assert_eq!(
            session
                .element_task_execution()
                .pending_total_tasks()
                .unwrap(),
            0
        );
    }
}
#[test]
fn current_request_fields_are_captured_when_fresh_enabled_check_allows_nested_close() {
    let (mut session, _, _, subject) = fixture();
    drop(complete(apply(
        &mut session,
        DomSourceEdit::set_attribute(subject, "closedby", "any"),
    )));
    open(&mut session, subject);
    let mut context = mutation_context(&session, Mutation::Reenter, false);
    let mut registry = mutation_registry(&mut session, subject);
    let request = start_request(&mut session, subject, Some("outer".into()));
    let receipt = service(&mut session, &request, &mut registry, &mut context);
    assert!(receipt.failures().is_empty());
    finish_nested_close(
        &mut session,
        request.operation(),
        &mut registry,
        &mut context,
    );
    assert_eq!(
        session
            .snapshot()
            .node(subject)
            .unwrap()
            .as_dialog()
            .unwrap()
            .return_value(),
        &context.latest
    );
    assert_surviving_reset(&session, subject, None);
}
#[test]
fn membership_removal_and_reopening_preserve_original_reset_without_removing_replacement() {
    for prevent in [true, false] {
        let (mut session, _, _, subject) = fixture();
        open(&mut session, subject);
        let original = session
            .snapshot()
            .node(subject)
            .unwrap()
            .as_dialog()
            .unwrap()
            .close_watcher()
            .unwrap();
        let mut context = mutation_context(&session, Mutation::RemoveReopen, prevent);
        let mut registry = mutation_registry(&mut session, subject);
        let result = DocumentString::from("outer retained");
        let request = start_request(&mut session, subject, Some(result.clone()));
        let receipt = service(&mut session, &request, &mut registry, &mut context);
        assert_eq!(receipt.default_prevented(), prevent);
        assert!(receipt.failures().is_empty());
        drop(complete(resume(&mut session, request.operation()).unwrap()));
        let replacement = session
            .snapshot()
            .node(subject)
            .unwrap()
            .as_dialog()
            .unwrap()
            .close_watcher()
            .unwrap();
        assert_ne!(original, replacement);
        assert_surviving_reset(&session, subject, Some(&result));
        assert_eq!(
            session
                .snapshot()
                .node(subject)
                .unwrap()
                .attribute(&"open".into()),
            Some(&"".into())
        );
        registry = dispatcher();
        let next = start_request(&mut session, subject, None);
        let receipt = service(&mut session, &next, &mut registry, &mut context);
        assert!(!receipt.default_prevented());
        finish_nested_close(&mut session, next.operation(), &mut registry, &mut context);
        assert!(
            !session
                .snapshot()
                .node(subject)
                .unwrap()
                .as_dialog()
                .unwrap()
                .has_close_watcher()
        );
    }
}
#[test]
fn cancel_retirement_finishes_started_path_and_once_before_native_final_reset_failure() {
    for prevent in [true, false] {
        let (mut session, _, _, subject) = fixture();
        open(&mut session, subject);
        let mut context = mutation_context(&session, Mutation::Retire, prevent);
        let mut registry = mutation_registry(&mut session, subject);
        let late = registry
            .add_listener(
                &session.dom(),
                subject,
                DomEventType::Cancel,
                DomEventCallback::new(move |_, _, context: &mut MutationContext, event| {
                    context.calls.push("late");
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
        let request = start_request(&mut session, subject, Some("retired request".into()));
        let receipt = service(&mut session, &request, &mut registry, &mut context);
        assert_eq!(receipt.callbacks(), 2);
        assert_eq!(receipt.default_prevented(), prevent);
        assert!(receipt.failures().is_empty());
        assert_eq!(context.calls, ["first", "late"]);
        assert!(context.destruction.as_ref().unwrap().is_ok());
        assert!(!registry.remove_listener(late));
        assert!(matches!(
            session.snapshot().node(subject),
            Err(DomError::StaleNode)
        ));
        context.nested.clear();
        drop(context.destruction.take());
        drop(receipt);
        let terminal = resume(&mut session, request.operation());
        assert!(
            matches!(&terminal, Err(DomSourceRuntimeError::Dom(DomError::SourceReceiverRetired {
            operation, subject: actual, action: HtmlDialogAction::SetForceCloseWatcherEnabled(false)
        })) if *operation == request.operation() && *actual == subject),
            "actual retired Cancel terminal: {terminal:?}"
        );
        assert!(session.dom().pending_source_edit().is_none());
        assert_eq!(
            session
                .element_task_execution()
                .pending_total_tasks()
                .unwrap(),
            0
        );
    }
}
#[test]
fn ordinary_removal_detach_adoption_and_failed_retirement_preserve_live_final_reset() {
    for case in 0..4 {
        let (mut session, _, _, subject) = fixture();
        // Adoption stays in this canonical owner; a second real top-level
        // supplies its independently associated Document.
        let mut hosted = session.hosted_execution();
        let host = hosted.create_host(&mut session.dom(), &cancel()).unwrap();
        let target = hosted
            .create_top_level(&mut session.dom(), host, &cancel())
            .unwrap();
        let snapshot = session.snapshot();
        let html = snapshot.node(target.document().node()).unwrap().children()[0];
        let body = snapshot.node(html).unwrap().children()[1];
        open(&mut session, subject);
        let mutation = match case {
            0 => Mutation::Remove,
            1 => Mutation::Detach,
            2 => Mutation::Adopt(body),
            _ => Mutation::FailedRetire,
        };
        let mut context = mutation_context(&session, mutation, true);
        let mut registry = mutation_registry(&mut session, subject);
        let result = DocumentString::from("live receiver");
        let request = start_request(&mut session, subject, Some(result.clone()));
        let receipt = service(&mut session, &request, &mut registry, &mut context);
        assert!(receipt.default_prevented());
        assert!(receipt.failures().is_empty());
        drop(complete(resume(&mut session, request.operation()).unwrap()));
        assert_surviving_reset(&session, subject, Some(&result));
        if case == 3 {
            assert!(matches!(
                context.destruction,
                Some(Err(DomError::Cancelled))
            ));
        }
        assert!(session.dom().pending_source_edit().is_none());
    }
}
#[test]
fn cancel_dispatch_suspension_drop_and_canceled_attempt_preserve_once_and_original_receipt() {
    let (mut session, _, _, subject) = fixture();
    open(&mut session, subject);
    let mut registry = dispatcher::<Vec<&'static str>>();
    let once = registry
        .add_listener(
            &session.dom(),
            subject,
            DomEventType::Cancel,
            DomEventCallback::new(|_, _, calls: &mut Vec<&'static str>, event| {
                calls.push("first");
                event.prevent_default();
                Ok(())
            }),
            DomEventListenerOptions {
                capture: false,
                once: true,
            },
        )
        .unwrap();
    registry
        .add_listener(
            &session.dom(),
            subject,
            DomEventType::Cancel,
            DomEventCallback::new(|_, _, calls: &mut Vec<&'static str>, _| {
                calls.push("late");
                Ok(())
            }),
            DomEventListenerOptions::default(),
        )
        .unwrap();
    let request = start_request(&mut session, subject, Some("retained dispatch".into()));
    let mut calls = Vec::new();
    let mut one = dispatch_limits();
    one.callbacks = 1;
    let retained = session
        .source_execution()
        .service_event(
            &mut session.dom(),
            &request,
            &mut registry,
            &mut calls,
            DomSourceEventServiceLimits {
                source: source_limits(),
                dispatch: one,
            },
            &cancel(),
        )
        .unwrap();
    assert!(matches!(
        retained,
        DomSourceEventServiceProgress::Retained { .. }
    ));
    assert_eq!(calls, ["first"]);
    assert!(!registry.remove_listener(once));
    drop(retained);
    let canceled = session.source_execution().service_event(
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
    assert!(matches!(
        canceled,
        Err(DomSourceRuntimeError::SourceEvent(
            DomSourceEventRuntimeError::Dom(DomError::Cancelled)
        ))
    ));
    assert_eq!(calls, ["first"]);
    let receipt = service(&mut session, &request, &mut registry, &mut calls);
    assert_eq!(receipt.callbacks(), 2);
    assert!(receipt.default_prevented());
    assert!(receipt.failures().is_empty());
    assert_eq!(calls, ["first", "late"]);
    drop(complete(resume(&mut session, request.operation()).unwrap()));
    assert_surviving_reset(&session, subject, Some(&"retained dispatch".into()));
    assert!(session.dom().pending_source_edit().is_none());
    assert_eq!(
        session
            .element_task_execution()
            .pending_total_tasks()
            .unwrap(),
        0
    );
}
#[test]
fn nested_close_retirement_preserves_its_stopped_action_and_settles_original_watcher() {
    let (mut session, top, _, subject) = fixture();
    open(&mut session, subject);
    let mut context = mutation_context(&session, Mutation::Retire, false);
    let mut registry = dispatcher::<MutationContext>();
    registry
        .add_listener(
            &session.dom(),
            subject,
            DomEventType::BeforeToggle,
            DomEventCallback::new(move |dom, _, context: &mut MutationContext, event| {
                context.calls.push("nested first");
                assert!(!event.cancelable());
                let parent = dom.snapshot().incomplete_source_operation();
                assert!(parent.is_some());
                let document = dom
                    .snapshot()
                    .node(subject)
                    .unwrap()
                    .owner_document()
                    .unwrap();
                nested(
                    context,
                    dom,
                    DomSourceEdit::remove_attribute(subject, "open"),
                );
                let mut batch = DomBatch::new();
                batch.detach(subject);
                batch.destroy(subject);
                context.destruction = Some(dom.apply(batch, &cancel()));
                assert!(context.destruction.as_ref().unwrap().is_ok());
                let after = dom.snapshot();
                assert_eq!(
                    after.incomplete_source_operation(),
                    parent,
                    "ordinary nested Close destruction preserves the actual published Source parent"
                );
                assert!(matches!(
                    after.document(document).unwrap().is_fully_active(),
                    Err(DomHostedError::Dom(DomError::IncompleteSourceOperation))
                ));
                Ok(())
            }),
            DomEventListenerOptions::default(),
        )
        .unwrap();
    let late = registry
        .add_listener(
            &session.dom(),
            subject,
            DomEventType::BeforeToggle,
            DomEventCallback::new(move |_, _, context: &mut MutationContext, event| {
                context.calls.push("nested late");
                assert_eq!(event.target(), Some(subject));
                Ok(())
            }),
            DomEventListenerOptions {
                capture: false,
                once: true,
            },
        )
        .unwrap();
    let request = start_request(
        &mut session,
        subject,
        Some("unperformed nested result".into()),
    );
    let cancel_receipt = service(&mut session, &request, &mut registry, &mut context);
    assert!(!cancel_receipt.default_prevented());
    let before = pending_event(
        resume(&mut session, request.operation()).unwrap(),
        DomSourceEventKind::BeforeToggle,
    );
    assert_ne!(before.identity(), request.identity());
    let receipt = service(&mut session, &before, &mut registry, &mut context);
    assert_eq!(receipt.callbacks(), 2);
    assert!(receipt.failures().is_empty());
    assert!(!receipt.default_prevented());
    assert!(!registry.remove_listener(late));
    assert!(context.destruction.as_ref().unwrap().is_ok());
    assert_eq!(context.calls, ["nested first", "nested late"]);
    context.nested.clear();
    drop(context.destruction.take());
    drop(cancel_receipt);
    drop(receipt);
    let terminal = resume(&mut session, request.operation());
    assert!(
        matches!(&terminal, Err(DomSourceRuntimeError::Dom(DomError::SourceReceiverRetired {
        operation, subject: actual, action: HtmlDialogAction::ReadOpen
    })) if *operation == request.operation() && *actual == subject),
        "actual nested Close terminal: {terminal:?}"
    );
    assert!(session.dom().pending_source_edit().is_none());
    assert_eq!(
        session
            .element_task_execution()
            .pending_total_tasks()
            .unwrap(),
        0
    );
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
struct InactivityContext {
    hosted: HostedRuntime,
    top: DomHostedTopLevel,
    refused: bool,
}
#[test]
fn attempted_window_retirement_during_cancel_refuses_and_preserves_actual_live_reset() {
    let (mut session, top, _, subject) = fixture();
    open(&mut session, subject);
    let mut context = InactivityContext {
        hosted: session.hosted_execution(),
        top,
        refused: false,
    };
    let mut registry = dispatcher::<InactivityContext>();
    registry.add_listener(&session.dom(), subject, DomEventType::Cancel, DomEventCallback::new(|dom,_,context: &mut InactivityContext,event| {
        event.prevent_default();
        let retirement = context.hosted.destroy_top_level(dom, context.top.navigable(), &cancel());
        assert!(matches!(retirement, Err(DomHostedRuntimeError::Dom(DomHostedError::Dom(DomError::SourceOperationPending)))), "ordinary hosted destruction cannot start during Source dispatch: {retirement:?}");
        context.refused = true;
        Ok(())
    }), DomEventListenerOptions::default()).unwrap();
    let result = DocumentString::from("surviving active request");
    let request = start_request(&mut session, subject, Some(result.clone()));
    let receipt = service(&mut session, &request, &mut registry, &mut context);
    assert!(receipt.default_prevented());
    assert!(receipt.failures().is_empty());
    assert!(context.refused);
    drop(complete(resume(&mut session, request.operation()).unwrap()));
    assert_surviving_reset(&session, subject, Some(&result));
    assert!(
        session
            .snapshot()
            .document(top.document())
            .unwrap()
            .is_fully_active()
            .unwrap()
    );
    assert_eq!(
        session
            .snapshot()
            .node(subject)
            .unwrap()
            .attribute(&"open".into()),
        Some(&"".into())
    );
    assert_eq!(
        session
            .element_task_execution()
            .pending_total_tasks()
            .unwrap(),
        0
    );
    // Actual retirement occurs only after the Source method has completed.
    destroy_top(&mut session, &mut context.hosted, top);
    assert!(
        !session
            .snapshot()
            .document(top.document())
            .unwrap()
            .is_fully_active()
            .unwrap()
    );
    drop(complete(apply(
        &mut session,
        DomSourceEdit::request_close_dialog(subject, Some("unreached".into())),
    )));
    assert_surviving_reset(&session, subject, Some(&result));
    drop(complete(apply(
        &mut session,
        DomSourceEdit::remove_attribute(subject, "open"),
    )));
    context
        .hosted
        .destroy_host(&mut session.dom(), top.host(), &cancel())
        .unwrap();
    drop(receipt);
    assert!(matches!(
        session.try_shutdown(),
        DomShutdownOutcome::Complete
    ));
}
#[test]
fn adopted_dialog_uses_genuinely_retired_original_window_before_request_entry() {
    let (mut session, original, _, subject) = fixture();
    let mut hosted = session.hosted_execution();
    let target_host = hosted.create_host(&mut session.dom(), &cancel()).unwrap();
    let target = hosted
        .create_top_level(&mut session.dom(), target_host, &cancel())
        .unwrap();
    let snapshot = session.snapshot();
    let html = snapshot.node(target.document().node()).unwrap().children()[0];
    let body = snapshot.node(html).unwrap().children()[1];
    open(&mut session, subject);
    drop(complete(apply(
        &mut session,
        DomSourceEdit::insert_before(subject, body, None),
    )));
    let watcher = session
        .snapshot()
        .node(subject)
        .unwrap()
        .as_dialog()
        .unwrap()
        .close_watcher()
        .unwrap();
    assert_eq!(watcher.global(), original.global());
    assert_eq!(watcher.window_document(), original.document());
    destroy_top(&mut session, &mut hosted, original);
    let snapshot = session.snapshot();
    assert!(
        !snapshot
            .document(original.document())
            .unwrap()
            .is_fully_active()
            .unwrap()
    );
    assert!(
        snapshot
            .document(target.document())
            .unwrap()
            .is_fully_active()
            .unwrap()
    );
    assert_eq!(
        snapshot.node(subject).unwrap().owner_document(),
        Some(target.document())
    );
    let result = DocumentString::from("request into inactive original Window");
    // The current node-Document is active, so outer request state is written.
    // The actual original watcher Window is inactive, so no Cancel is fired.
    drop(complete(apply(
        &mut session,
        DomSourceEdit::request_close_dialog(subject, Some(result.clone())),
    )));
    assert_surviving_reset(&session, subject, Some(&result));
    assert_eq!(
        session
            .snapshot()
            .node(subject)
            .unwrap()
            .as_dialog()
            .unwrap()
            .close_watcher(),
        Some(watcher)
    );
    assert_eq!(
        session
            .snapshot()
            .node(subject)
            .unwrap()
            .attribute(&"open".into()),
        Some(&"".into())
    );
    assert_eq!(
        session
            .element_task_execution()
            .pending_total_tasks()
            .unwrap(),
        0
    );
    drop(complete(apply(
        &mut session,
        DomSourceEdit::remove_attribute(subject, "open"),
    )));
    hosted
        .destroy_host(&mut session.dom(), original.host(), &cancel())
        .unwrap();
    destroy_top(&mut session, &mut hosted, target);
    hosted
        .destroy_host(&mut session.dom(), target.host(), &cancel())
        .unwrap();
    assert!(matches!(
        session.try_shutdown(),
        DomShutdownOutcome::Complete
    ));
}
struct QueueContext {
    tasks: DomElementTaskRuntime,
    delivered: Vec<DomEventType>,
}
impl DomElementTaskEventContext for QueueContext {
    fn element_tasks(&mut self) -> &mut DomElementTaskRuntime {
        &mut self.tasks
    }
}
#[test]
fn successful_request_dispatches_real_close_and_toggle_tasks_through_their_actual_sources() {
    let (mut session, _, _, subject) = fixture();
    open(&mut session, subject);
    let mut context = QueueContext {
        tasks: session.element_task_execution(),
        delivered: Vec::new(),
    };
    let mut registry = dispatcher::<QueueContext>();
    for kind in [DomEventType::Close, DomEventType::Toggle] {
        registry
            .add_listener(
                &session.dom(),
                subject,
                kind,
                DomEventCallback::new(move |_, _, context: &mut QueueContext, event| {
                    assert_eq!(event.event_type(), kind);
                    assert!(!event.cancelable());
                    if kind == DomEventType::Close {
                        assert!(event.toggle_event().is_none());
                    } else {
                        let payload = event.toggle_event().unwrap();
                        assert_eq!(payload.old_state(), "open");
                        assert_eq!(payload.new_state(), "closed");
                    }
                    context.delivered.push(kind);
                    Ok(())
                }),
                DomEventListenerOptions::default(),
            )
            .unwrap();
    }
    let request = start_request(&mut session, subject, Some("real queue result".into()));
    assert!(!service(&mut session, &request, &mut registry, &mut context).default_prevented());
    finish_nested_close(
        &mut session,
        request.operation(),
        &mut registry,
        &mut context,
    );
    let close = context
        .tasks
        .pending_task(DomElementTaskSourceKind::UserInteraction, 0)
        .unwrap()
        .unwrap();
    let toggle = context
        .tasks
        .pending_task(DomElementTaskSourceKind::DomManipulation, 0)
        .unwrap()
        .unwrap();
    assert_ne!(close.id(), toggle.id());
    assert_eq!(close.queue_association(), toggle.queue_association());
    for source in [
        DomElementTaskSourceKind::UserInteraction,
        DomElementTaskSourceKind::DomManipulation,
    ] {
        let progress = DomElementTaskRuntime::run_next(
            &mut session.dom(),
            source,
            &mut registry,
            &mut context,
            run_limits(),
            &cancel(),
        )
        .unwrap();
        let DomElementTaskProgress::Complete {
            receipt,
            displaced_owner,
            ..
        } = progress
        else {
            panic!("real queued event must dispatch: {progress:?}");
        };
        assert!(displaced_owner.is_none());
        assert_eq!(receipt.callbacks(), 1);
        assert!(receipt.failures().is_empty());
    }
    assert_eq!(
        context.delivered,
        [DomEventType::Close, DomEventType::Toggle]
    );
    assert_eq!(context.tasks.pending_total_tasks().unwrap(), 0);
    assert!(session.dom().pending_source_edit().is_none());
}
struct TerminalChildContext {
    invoking: Option<DomSourceRuntime>,
    editing: DomSourceRuntime,
    parent: NodeId,
    child: NodeId,
    child_invocations: usize,
    child_operation: Option<DomSourceOperationId>,
    calls: Vec<&'static str>,
}
#[test]
fn terminal_child_request_inside_parent_cancel_preserves_original_parent_and_no_replay() {
    let (mut session, top, body, parent) = fixture();
    let mut batch = DomBatch::new();
    let child = batch.create_element_ns(top.document(), Namespace::html(), "dialog");
    batch.insert_before(&child, body, None);
    let child = session
        .dom()
        .apply(batch, &cancel())
        .unwrap()
        .created(&child)
        .unwrap()
        .unwrap();
    open(&mut session, parent);
    open(&mut session, child);
    let parent_watcher = session
        .snapshot()
        .node(parent)
        .unwrap()
        .as_dialog()
        .unwrap()
        .close_watcher()
        .unwrap();
    let mut context = TerminalChildContext {
        invoking: Some(session.source_execution()),
        editing: session.source_execution(),
        parent,
        child,
        child_invocations: 0,
        child_operation: None,
        calls: Vec::new(),
    };
    let mut registry = dispatcher::<TerminalChildContext>();
    registry
        .add_listener(
            &session.dom(),
            child,
            DomEventType::Cancel,
            DomEventCallback::new(|dom, _, context: &mut TerminalChildContext, event| {
                context.calls.push("child first");
                event.prevent_default();
                let child_marker = dom.snapshot().incomplete_source_operation();
                assert!(child_marker.is_some());
                let document = dom
                    .snapshot()
                    .node(context.child)
                    .unwrap()
                    .owner_document()
                    .unwrap();
                drop(complete(
                    context
                        .editing
                        .apply(
                            dom,
                            DomSourceEdit::remove_attribute(context.child, "open"),
                            source_limits(),
                            &mut DomSourceServices {
                                execution_scope: None,
                                style_policy: &mut |_| None,
                                url: None,
                            },
                            &cancel(),
                        )
                        .unwrap(),
                ));
                let mut batch = DomBatch::new();
                batch.detach(context.child);
                batch.destroy(context.child);
                dom.apply(batch, &cancel()).unwrap();
                assert_eq!(dom.snapshot().incomplete_source_operation(), child_marker);
                assert!(matches!(
                    dom.snapshot().document(document).unwrap().is_fully_active(),
                    Err(DomHostedError::Dom(DomError::IncompleteSourceOperation))
                ));
                Ok(())
            }),
            DomEventListenerOptions::default(),
        )
        .unwrap();
    let child_late = registry
        .add_listener(
            &session.dom(),
            child,
            DomEventType::Cancel,
            DomEventCallback::new(|_, _, context: &mut TerminalChildContext, event| {
                context.calls.push("child late");
                assert_eq!(event.target(), Some(context.child));
                assert!(event.default_prevented());
                Ok(())
            }),
            DomEventListenerOptions {
                capture: false,
                once: true,
            },
        )
        .unwrap();
    registry.add_listener(&session.dom(), parent, DomEventType::Cancel, DomEventCallback::new(|dom,listeners,context: &mut TerminalChildContext,event| {
        context.calls.push("parent first");
        event.prevent_default();
        if context.child_invocations != 0 { return Ok(()); }
        context.child_invocations += 1;
        let parent_marker = dom.snapshot().incomplete_source_operation().expect("real parent request prefix is published");
        let mut invoking = context.invoking.take().expect("original Session permission");
        let child_progress = invoking.apply(dom, DomSourceEdit::request_close_dialog(context.child, Some("child request".into())), source_limits(), &mut DomSourceServices {
            execution_scope: None, style_policy: &mut |_| None, url: None,
        }, &cancel()).unwrap();
        let child_request = pending_event(child_progress, DomSourceEventKind::Cancel);
        assert_ne!(child_request.operation(), parent_marker);
        context.child_operation = Some(child_request.operation());
        let actual = invoking.service_event(dom, &child_request, listeners, context,
            DomSourceEventServiceLimits {source: source_limits(), dispatch: dispatch_limits()}, &cancel()).unwrap();
        let DomSourceEventServiceProgress::Serviced {receipt} = actual else {panic!("real child Cancel must finish: {actual:?}");};
        assert_eq!(receipt.callbacks(), 2);
        assert!(receipt.default_prevented());
        assert!(receipt.failures().is_empty());
        drop(receipt);
        let terminal = invoking.resume(dom, child_request.operation(), source_limits(), &mut DomSourceServices {
            execution_scope: None, style_policy: &mut |_| None, url: None,
        }, &cancel());
        assert!(matches!(&terminal, Err(DomSourceRuntimeError::Dom(DomError::SourceReceiverRetired {
            operation, subject, action: HtmlDialogAction::SetForceCloseWatcherEnabled(false)
        })) if *operation == child_request.operation() && *subject == context.child), "actual child terminal: {terminal:?}");
        let actual_parent = dom.pending_source_edit().expect("actual observing parent remains");
        assert_eq!(actual_parent.operation(), parent_marker);
        assert_eq!(actual_parent.cleanup_cause(), None, "terminal child failure cannot become the parent's cleanup cause");
        assert_eq!(actual_parent.phase(), DomSourcePhase::EventObservation);
        assert_eq!(dom.snapshot().incomplete_source_operation(), Some(parent_marker));
        let snapshot = dom.snapshot();
        let parent_state = snapshot.node(context.parent).unwrap().as_dialog().unwrap();
        assert!(parent_state.force_close_watcher_enabled());
        assert_eq!(parent_state.request_close_return_value(), Some(&DocumentString::from("parent request")));
        assert_eq!(parent_state.request_close_source(), None);
        context.invoking = Some(invoking);
        Ok(())
    }), DomEventListenerOptions::default()).unwrap();
    let parent_late = registry
        .add_listener(
            &session.dom(),
            parent,
            DomEventType::Cancel,
            DomEventCallback::new(|_, _, context: &mut TerminalChildContext, event| {
                context.calls.push("parent late");
                assert_eq!(event.target(), Some(context.parent));
                assert!(event.default_prevented());
                Ok(())
            }),
            DomEventListenerOptions {
                capture: false,
                once: true,
            },
        )
        .unwrap();
    let parent_result = DocumentString::from("parent request");
    let request = start_request(&mut session, parent, Some(parent_result.clone()));
    let receipt = service(&mut session, &request, &mut registry, &mut context);
    assert_eq!(receipt.callbacks(), 2);
    assert!(
        receipt.failures().is_empty(),
        "actual outer receipt: {receipt:?}"
    );
    assert!(receipt.default_prevented());
    assert!(!registry.remove_listener(child_late));
    assert!(!registry.remove_listener(parent_late));
    assert!(context.child_operation.is_some());
    assert_ne!(context.child_operation, Some(request.operation()));
    assert_eq!(
        context.calls,
        ["parent first", "child first", "child late", "parent late"]
    );
    drop(receipt);
    let terminal = resume(&mut session, request.operation());
    let done = complete(terminal.unwrap());
    assert_eq!(done.operation(), request.operation());
    drop(done);
    assert_surviving_reset(&session, parent, Some(&parent_result));
    assert_eq!(
        session
            .snapshot()
            .node(parent)
            .unwrap()
            .as_dialog()
            .unwrap()
            .close_watcher(),
        Some(parent_watcher)
    );
    assert!(matches!(
        session.snapshot().node(child),
        Err(DomError::StaleNode)
    ));
    assert!(session.dom().pending_source_edit().is_none());
    assert_eq!(
        session
            .element_task_execution()
            .pending_total_tasks()
            .unwrap(),
        0
    );
    let next = start_request(&mut session, parent, None);
    let receipt = service(&mut session, &next, &mut registry, &mut context);
    assert_eq!(receipt.callbacks(), 1);
    assert!(receipt.failures().is_empty());
    assert!(receipt.default_prevented());
    drop(receipt);
    drop(complete(resume(&mut session, next.operation()).unwrap()));
    assert_eq!(context.child_invocations, 1);
    assert_eq!(
        context.calls,
        [
            "parent first",
            "child first",
            "child late",
            "parent late",
            "parent first"
        ]
    );
    assert_surviving_reset(&session, parent, None);
    drop(complete(apply(
        &mut session,
        DomSourceEdit::remove_attribute(parent, "open"),
    )));
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
