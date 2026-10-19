//! Functional evidence for standard nonmodal close(optional result), whose source is null.
//! HTML877d46464–68,46684–46705 and46727–46 supply the literal outcomes.
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
fn start_close(
    session: &mut DomSession,
    subject: NodeId,
    result: Option<DocumentString>,
) -> DomSourceEventRequest {
    match apply(session, DomSourceEdit::close_dialog(subject, result)) {
        DomSourceDriveProgress::Pending {
            request: DomSourceRequest::Event(request),
            reason: DomSourceUnavailable::Event,
        } => {
            assert_eq!(request.subject(), subject);
            assert!(!request.cancelable());
            let toggle = request
                .toggle_event()
                .expect("Close beforetoggle has the actual ToggleEvent data");
            assert_eq!(toggle.old_state(), HtmlDialogState::Open);
            assert_eq!(toggle.new_state(), HtmlDialogState::Closed);
            assert_eq!(toggle.source(), None);
            request
        }
        other => {
            panic!("open Close must first yield its actual noncancelable beforetoggle: {other:?}")
        }
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
#[derive(Debug)]
struct EventObservation {
    kind: DomEventType,
    target: Option<NodeId>,
    current: Option<DomEventTarget>,
    payload: Option<DomToggleEvent>,
    path: Vec<DomEventTarget>,
    bubbles: bool,
    cancelable: bool,
    prevented: bool,
}
fn observe(event: &NativeDomEvent) -> EventObservation {
    EventObservation {
        kind: event.event_type(),
        target: event.target(),
        current: event.current_target(),
        payload: event.toggle_event().copied(),
        path: event.composed_path().collect(),
        bubbles: event.bubbles(),
        cancelable: event.cancelable(),
        prevented: event.default_prevented(),
    }
}
struct Context {
    owner: DomElementTaskRuntime,
    before: Vec<EventObservation>,
    delivered: Vec<EventObservation>,
    edits: Vec<Result<DomCommit, DomError>>,
}
impl DomElementTaskEventContext for Context {
    fn element_tasks(&mut self) -> &mut DomElementTaskRuntime {
        &mut self.owner
    }
}
fn context(session: &DomSession) -> Context {
    Context {
        owner: session.element_task_execution(),
        before: Vec::new(),
        delivered: Vec::new(),
        edits: Vec::new(),
    }
}
fn listeners(
    session: &mut DomSession,
    subject: NodeId,
    body: NodeId,
) -> DomEventDispatcher<Context> {
    let mut registry = dispatcher();
    registry
        .add_listener(
            &mut session.dom(),
            subject,
            DomEventType::BeforeToggle,
            DomEventCallback::new(|_, _, context: &mut Context, event| {
                event.prevent_default();
                context.before.push(observe(event));
                Ok(())
            }),
            DomEventListenerOptions::default(),
        )
        .unwrap();
    for kind in [DomEventType::Close, DomEventType::Toggle] {
        registry
            .add_listener(
                &mut session.dom(),
                subject,
                kind,
                DomEventCallback::new(move |dom, _, context: &mut Context, event| {
                    context.delivered.push(observe(event));
                    let mut batch = DomBatch::new();
                    batch.set_attribute(
                        body,
                        if kind == DomEventType::Close {
                            "data-close"
                        } else {
                            "data-toggle"
                        },
                        "kept",
                    );
                    context.edits.push(dom.apply(batch, &cancel()));
                    Ok(())
                }),
                DomEventListenerOptions::default(),
            )
            .unwrap();
    }
    registry
}
fn finish_close(
    session: &mut DomSession,
    request: &DomSourceEventRequest,
    registry: &mut DomEventDispatcher<Context>,
    context: &mut Context,
) {
    let receipt = service(session, request, registry, context);
    assert_eq!(receipt.callbacks(), 1);
    assert!(receipt.failures().is_empty());
    assert!(!receipt.default_prevented());
    drop(receipt);
    let done = complete(resume(session, request.operation()).unwrap());
    assert_eq!(done.operation(), request.operation());
    drop(done);
}
fn run(
    session: &mut DomSession,
    source: DomElementTaskSourceKind,
    registry: &mut DomEventDispatcher<Context>,
    context: &mut Context,
) -> DomElementTask {
    match DomElementTaskRuntime::run_next(
        &mut session.dom(),
        source,
        registry,
        context,
        run_limits(),
        &cancel(),
    )
    .unwrap()
    {
        DomElementTaskProgress::Complete {
            task,
            receipt,
            dialog_toggle_disposition,
            displaced_owner,
        } => {
            assert_eq!(
                dialog_toggle_disposition,
                task.dialog_toggle()
                    .map(|_| DomDialogToggleTaskDisposition::TrackerCleared)
            );
            assert!(displaced_owner.is_none());
            assert_eq!(receipt.original_target(), task.id().target());
            assert_eq!(
                receipt.event_type(),
                if source == DomElementTaskSourceKind::UserInteraction {
                    DomEventType::Close
                } else {
                    DomEventType::Toggle
                }
            );
            assert_eq!(receipt.callbacks(), 1);
            assert!(receipt.failures().is_empty());
            assert!(!receipt.default_prevented());
            task
        }
        other => panic!("the selected actual task must finish delivery: {other:?}"),
    }
}
fn assert_before(event: &EventObservation, subject: NodeId) {
    assert_eq!(event.kind, DomEventType::BeforeToggle);
    assert_eq!(event.target, Some(subject));
    assert_eq!(event.current, Some(DomEventTarget::Node(subject)));
    assert!(!event.cancelable);
    assert!(!event.bubbles);
    assert!(!event.prevented);
    let toggle = event.payload.unwrap();
    assert_eq!(toggle.old_state(), "open");
    assert_eq!(toggle.new_state(), "closed");
    assert_eq!(toggle.source(), None);
}

#[test]
fn initially_closed_method_has_no_event_task_or_return_value_write() {
    let (mut session, _, body, subject) = fixture();
    let seeded = DocumentString::from("seeded");
    seed_return_value(&mut session, subject, &seeded);
    let mut context = context(&session);
    let _registry = listeners(&mut session, subject, body);
    let before = session.snapshot();
    for result in [
        None,
        Some(DocumentString::from("")),
        Some(DocumentString::from("ignored")),
    ] {
        drop(complete(apply(
            &mut session,
            DomSourceEdit::close_dialog(subject, result),
        )));
        assert_eq!(session.snapshot().stamp(), before.stamp());
        assert_eq!(
            session
                .snapshot()
                .node(subject)
                .unwrap()
                .as_dialog()
                .unwrap()
                .return_value(),
            &seeded
        );
        assert_eq!(context.owner.pending_total_tasks().unwrap(), 0);
        assert!(session.dom().pending_source_edit().is_none());
    }
    assert!(context.before.is_empty());
    assert!(context.delivered.is_empty());
    // Keep the actual observer owner live through the entire no-op sequence.
    assert_eq!(context.element_tasks().pending_total_tasks().unwrap(), 0);
}

#[test]
fn close_result_matrix_and_user_interaction_first_delivery_preserve_toggle_tracker() {
    let exact = DocumentString::from_utf16(vec![0xd800, 0, 0x0d, 0x0a, 0xdc00]);
    for result in [None, Some(DocumentString::from("")), Some(exact)] {
        let (mut session, top, body, subject) = fixture();
        let seeded = DocumentString::from("preserved if omitted");
        seed_return_value(&mut session, subject, &seeded);
        open(&mut session, subject);
        let before = session.snapshot();
        let mut context = context(&session);
        let mut registry = listeners(&mut session, subject, body);
        let expected = result.as_ref().unwrap_or(&seeded).clone();
        let request = start_close(&mut session, subject, result);
        finish_close(&mut session, &request, &mut registry, &mut context);
        assert_eq!(context.before.len(), 1);
        assert_before(&context.before[0], subject);
        let closed = session.snapshot();
        let dialog = closed.node(subject).unwrap().as_dialog().unwrap();
        assert_eq!(
            closed.node(subject).unwrap().attribute(&"open".into()),
            None
        );
        assert!(!dialog.has_close_watcher());
        assert!(!dialog.is_modal());
        assert_eq!(dialog.return_value(), &expected);
        assert_eq!(dialog.request_close_return_value(), None);
        assert_eq!(dialog.request_close_source(), None);
        assert_eq!(
            closed
                .node(subject)
                .unwrap()
                .as_element()
                .unwrap()
                .common_state()
                .html()
                .unwrap()
                .previously_focused_element(),
            None
        );
        assert!(session.dom().pending_source_edit().is_none());
        assert_eq!(closed.incomplete_source_operation(), None);
        assert_eq!(context.owner.pending_total_tasks().unwrap(), 2);
        assert_eq!(
            context
                .owner
                .pending_tasks(DomElementTaskSourceKind::DomManipulation)
                .unwrap(),
            1
        );
        assert_eq!(
            context
                .owner
                .pending_tasks(DomElementTaskSourceKind::UserInteraction)
                .unwrap(),
            1
        );
        let toggle = context
            .owner
            .pending_task(DomElementTaskSourceKind::DomManipulation, 0)
            .unwrap()
            .unwrap();
        let close = context
            .owner
            .pending_task(DomElementTaskSourceKind::UserInteraction, 0)
            .unwrap()
            .unwrap();
        assert!(matches!(toggle, DomElementTask::DialogToggle(_)));
        assert!(matches!(close, DomElementTask::DialogClose(_)));
        assert_eq!(toggle.source(), DomElementTaskSourceKind::DomManipulation);
        assert_eq!(close.source(), DomElementTaskSourceKind::UserInteraction);
        assert_ne!(toggle.id(), close.id());
        for task in [toggle, close] {
            assert_eq!(task.id().target(), subject);
            assert_eq!(task.queue_association().unwrap().global(), top.global());
            assert_eq!(task.queue_association().unwrap().document(), top.document());
        }
        assert_eq!(close.dialog_close().unwrap().id(), close.id());
        let payload = toggle.dialog_toggle().unwrap();
        assert_eq!(payload.old_state(), HtmlDialogState::Open);
        assert_eq!(payload.new_state(), HtmlDialogState::Closed);
        assert_eq!(payload.source(), None);
        let tracker = dialog.toggle_task_tracker().unwrap();
        assert_eq!(tracker.task(), toggle.id());

        // HTML specifies two sources, not a cross-source FIFO. Select the
        // later-queued plain Close first; it has no Toggle tracker cleanup.
        assert_eq!(
            run(
                &mut session,
                DomElementTaskSourceKind::UserInteraction,
                &mut registry,
                &mut context
            ),
            close
        );
        assert_eq!(context.delivered.len(), 1);
        let observed = &context.delivered[0];
        assert_eq!(observed.kind, DomEventType::Close);
        assert_eq!(observed.target, Some(subject));
        assert_eq!(observed.current, Some(DomEventTarget::Node(subject)));
        assert_eq!(observed.payload, None);
        assert!(!observed.cancelable);
        assert!(!observed.bubbles);
        assert_eq!(
            session
                .snapshot()
                .node(subject)
                .unwrap()
                .as_dialog()
                .unwrap()
                .toggle_task_tracker(),
            Some(tracker)
        );
        assert_eq!(
            context
                .owner
                .pending_tasks(DomElementTaskSourceKind::UserInteraction)
                .unwrap(),
            0
        );
        assert_eq!(
            context
                .owner
                .pending_tasks(DomElementTaskSourceKind::DomManipulation)
                .unwrap(),
            1
        );
        assert_eq!(
            session
                .snapshot()
                .node(body)
                .unwrap()
                .attribute(&"data-close".into()),
            Some(&"kept".into())
        );
        assert_eq!(
            run(
                &mut session,
                DomElementTaskSourceKind::DomManipulation,
                &mut registry,
                &mut context
            ),
            toggle
        );
        assert_eq!(context.delivered.len(), 2);
        let observed = &context.delivered[1];
        assert_eq!(observed.kind, DomEventType::Toggle);
        let event = observed.payload.unwrap();
        assert_eq!(event.old_state(), "open");
        assert_eq!(event.new_state(), "closed");
        assert_eq!(event.source(), None);
        let html = closed.node(top.document().node()).unwrap().children()[0];
        let expected_path = [
            DomEventTarget::Node(subject),
            DomEventTarget::Node(body),
            DomEventTarget::Node(html),
            DomEventTarget::Node(top.document().node()),
            DomEventTarget::Global(top.global()),
        ];
        for observed in &context.delivered {
            assert_eq!(observed.path, expected_path);
            assert_eq!(observed.target, Some(subject));
            assert_eq!(observed.current, Some(DomEventTarget::Node(subject)));
            assert!(!observed.cancelable);
            assert!(!observed.bubbles);
            assert!(!observed.prevented);
        }
        assert!(context.edits.iter().all(Result::is_ok));
        assert_eq!(
            session
                .snapshot()
                .node(body)
                .unwrap()
                .attribute(&"data-toggle".into()),
            Some(&"kept".into())
        );
        assert_eq!(
            session
                .snapshot()
                .node(subject)
                .unwrap()
                .as_dialog()
                .unwrap()
                .toggle_task_tracker(),
            None
        );
        assert_eq!(context.owner.pending_total_tasks().unwrap(), 0);
        assert!(session.dom().pending_source_edit().is_none());
        assert_eq!(
            closed
                .node(subject)
                .unwrap()
                .as_dialog()
                .unwrap()
                .toggle_task_tracker(),
            Some(tracker)
        );
        assert!(
            before
                .node(subject)
                .unwrap()
                .as_dialog()
                .unwrap()
                .has_close_watcher()
        );
        assert_eq!(
            before
                .node(subject)
                .unwrap()
                .as_dialog()
                .unwrap()
                .return_value(),
            &seeded
        );
        for source in [
            DomElementTaskSourceKind::UserInteraction,
            DomElementTaskSourceKind::DomManipulation,
        ] {
            assert!(matches!(
                DomElementTaskRuntime::run_next(
                    &mut session.dom(),
                    source,
                    &mut registry,
                    &mut context,
                    run_limits(),
                    &cancel()
                )
                .unwrap(),
                DomElementTaskProgress::Empty
            ));
        }
        assert_eq!(context.delivered.len(), 2);
    }
}

#[test]
fn reopening_between_closes_retains_each_plain_close_in_its_source_order() {
    let (mut session, _, body, subject) = fixture();
    let mut context = context(&session);
    let mut registry = listeners(&mut session, subject, body);
    open(&mut session, subject);
    let first = start_close(&mut session, subject, None);
    finish_close(&mut session, &first, &mut registry, &mut context);
    let first_snapshot = session.snapshot();
    let first_close = context
        .owner
        .pending_task(DomElementTaskSourceKind::UserInteraction, 0)
        .unwrap()
        .unwrap();
    let first_toggle = context
        .owner
        .pending_task(DomElementTaskSourceKind::DomManipulation, 0)
        .unwrap()
        .unwrap();
    open(&mut session, subject);
    let second = start_close(&mut session, subject, None);
    finish_close(&mut session, &second, &mut registry, &mut context);
    let second_close = context
        .owner
        .pending_task(DomElementTaskSourceKind::UserInteraction, 1)
        .unwrap()
        .unwrap();
    let second_toggle = context
        .owner
        .pending_task(DomElementTaskSourceKind::DomManipulation, 0)
        .unwrap()
        .unwrap();
    assert_ne!(first_close.id(), second_close.id());
    assert_ne!(first_toggle.id(), second_toggle.id());
    assert_eq!(
        context
            .owner
            .pending_tasks(DomElementTaskSourceKind::UserInteraction)
            .unwrap(),
        2
    );
    assert_eq!(
        context
            .owner
            .pending_tasks(DomElementTaskSourceKind::DomManipulation)
            .unwrap(),
        1
    );
    assert_eq!(context.owner.pending_total_tasks().unwrap(), 3);
    let tracker = session
        .snapshot()
        .node(subject)
        .unwrap()
        .as_dialog()
        .unwrap()
        .toggle_task_tracker()
        .unwrap();
    assert_eq!(tracker.task(), second_toggle.id());
    assert_eq!(
        first_snapshot
            .node(subject)
            .unwrap()
            .as_dialog()
            .unwrap()
            .toggle_task_tracker()
            .unwrap()
            .task(),
        first_toggle.id()
    );
    assert_eq!(
        run(
            &mut session,
            DomElementTaskSourceKind::UserInteraction,
            &mut registry,
            &mut context
        ),
        first_close
    );
    assert_eq!(
        run(
            &mut session,
            DomElementTaskSourceKind::UserInteraction,
            &mut registry,
            &mut context
        ),
        second_close
    );
    assert!(
        context
            .delivered
            .iter()
            .all(|event| event.kind == DomEventType::Close && event.payload.is_none())
    );
    assert_eq!(
        session
            .snapshot()
            .node(subject)
            .unwrap()
            .as_dialog()
            .unwrap()
            .toggle_task_tracker(),
        Some(tracker)
    );
    assert_eq!(
        run(
            &mut session,
            DomElementTaskSourceKind::DomManipulation,
            &mut registry,
            &mut context
        ),
        second_toggle
    );
    assert_eq!(context.before.len(), 2);
    assert_eq!(context.delivered.len(), 3);
    assert_eq!(context.owner.pending_total_tasks().unwrap(), 0);
}

#[derive(Clone, Copy)]
enum Mutation {
    RemoveOpen,
    Retire,
    FailedRetire,
    Detach,
    Adopt(NodeId),
}
struct MutationContext {
    source: DomSourceRuntime,
    mutation: Mutation,
    calls: Vec<&'static str>,
    nested: Vec<DomSourceCompletion>,
    destruction: Option<Result<DomCommit, DomError>>,
}
fn mutate_before_close(subject: NodeId) -> DomEventCallback<MutationContext> {
    DomEventCallback::new(move |dom, _, context: &mut MutationContext, event| {
        context.calls.push("first");
        event.prevent_default();
        let edit = match context.mutation {
            Mutation::RemoveOpen | Mutation::Retire => {
                Some(DomSourceEdit::remove_attribute(subject, "open"))
            }
            Mutation::Detach => Some(DomSourceEdit::detach(subject)),
            Mutation::Adopt(parent) => Some(DomSourceEdit::insert_before(subject, parent, None)),
            Mutation::FailedRetire => None,
        };
        if let Some(edit) = edit {
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
        match context.mutation {
            Mutation::Retire | Mutation::FailedRetire => {
                let mut batch = DomBatch::new();
                batch.detach(subject);
                batch.destroy(subject);
                context.destruction = Some(dom.apply(
                    batch,
                    &AtomicBool::new(matches!(context.mutation, Mutation::FailedRetire)),
                ));
            }
            _ => {}
        }
        Ok(())
    })
}
fn mutation_registry(
    session: &mut DomSession,
    subject: NodeId,
) -> DomEventDispatcher<MutationContext> {
    let mut registry = dispatcher();
    registry
        .add_listener(
            &mut session.dom(),
            subject,
            DomEventType::BeforeToggle,
            mutate_before_close(subject),
            DomEventListenerOptions::default(),
        )
        .unwrap();
    registry
}

#[test]
fn surviving_callback_open_removal_returns_without_outer_tasks_or_result_write() {
    let (mut session, _, _, subject) = fixture();
    let seed = DocumentString::from("keep actual returnValue");
    seed_return_value(&mut session, subject, &seed);
    open(&mut session, subject);
    let mut context = MutationContext {
        source: session.source_execution(),
        mutation: Mutation::RemoveOpen,
        calls: Vec::new(),
        nested: Vec::new(),
        destruction: None,
    };
    let mut registry = mutation_registry(&mut session, subject);
    let request = start_close(
        &mut session,
        subject,
        Some(DocumentString::from("must not write")),
    );
    let receipt = service(&mut session, &request, &mut registry, &mut context);
    assert_eq!(receipt.callbacks(), 1);
    assert!(!receipt.default_prevented());
    assert!(receipt.failures().is_empty());
    assert_eq!(context.nested.len(), 1);
    context.nested.clear();
    drop(complete(resume(&mut session, request.operation()).unwrap()));
    let snapshot = session.snapshot();
    let dialog = snapshot.node(subject).unwrap().as_dialog().unwrap();
    assert_eq!(dialog.return_value(), &seed);
    assert!(!dialog.has_close_watcher());
    assert!(!dialog.has_toggle_task_tracker());
    assert_eq!(
        snapshot.node(subject).unwrap().attribute(&"open".into()),
        None
    );
    assert_eq!(
        session
            .element_task_execution()
            .pending_total_tasks()
            .unwrap(),
        0
    );
    assert!(session.dom().pending_source_edit().is_none());
    assert_eq!(context.calls, ["first"]);
}

#[test]
fn started_close_finishes_late_once_listener_then_reports_exact_retired_read_open() {
    let (mut session, top, _, subject) = fixture();
    open(&mut session, subject);
    let before = session.snapshot();
    let mut context = MutationContext {
        source: session.source_execution(),
        mutation: Mutation::Retire,
        calls: Vec::new(),
        nested: Vec::new(),
        destruction: None,
    };
    let mut registry = mutation_registry(&mut session, subject);
    let late = registry
        .add_listener(
            &mut session.dom(),
            subject,
            DomEventType::BeforeToggle,
            DomEventCallback::new(move |_, _, context: &mut MutationContext, event| {
                context.calls.push("late");
                assert_eq!(event.target(), Some(subject));
                assert!(!event.cancelable());
                event.prevent_default();
                assert!(!event.default_prevented());
                Ok(())
            }),
            DomEventListenerOptions {
                capture: false,
                once: true,
            },
        )
        .unwrap();
    let request = start_close(
        &mut session,
        subject,
        Some(DocumentString::from("unperformed")),
    );
    let receipt = service(&mut session, &request, &mut registry, &mut context);
    assert_eq!(receipt.callbacks(), 2);
    assert!(receipt.failures().is_empty());
    assert!(!receipt.default_prevented());
    assert_eq!(context.calls, ["first", "late"]);
    assert!(context.destruction.as_ref().unwrap().is_ok());
    assert_eq!(context.nested.len(), 1);
    assert!(matches!(
        session.snapshot().node(subject),
        Err(DomError::StaleNode)
    ));
    assert!(!registry.remove_listener(late));
    context.nested.clear();
    drop(context.destruction.take());
    drop(receipt);
    assert!(
        matches!(resume(&mut session, request.operation()), Err(DomSourceRuntimeError::Dom(DomError::SourceReceiverRetired { operation, subject: actual, action: HtmlDialogAction::ReadOpen })) if operation == request.operation() && actual == subject)
    );
    assert_eq!(
        session
            .element_task_execution()
            .pending_total_tasks()
            .unwrap(),
        0
    );
    assert!(session.dom().pending_source_edit().is_none());
    assert!(
        before
            .node(subject)
            .unwrap()
            .as_dialog()
            .unwrap()
            .has_close_watcher()
    );
    assert_eq!(
        before.node(subject).unwrap().attribute(&"open".into()),
        Some(&"".into())
    );
    assert_eq!(context.calls, ["first", "late"]);
    let shutdown = session.try_shutdown();
    let mut session = match shutdown {
        DomShutdownOutcome::Pending {
            session,
            reason: DomShutdownReason::HostedLifecycle,
        } => session,
        other => panic!(
            "only actual hosted lifecycle remains after retired-receiver Source cleanup: {other:?}"
        ),
    };
    assert_eq!(session.snapshot().owner(), before.owner());
    // Shutdown priority has proved Source/event/task cleanup. Service the same
    // returned Session's remaining real top and Host before claiming completion.
    let mut hosted = session.hosted_execution();
    destroy_top(&mut session, &mut hosted, top);
    hosted
        .destroy_host(&mut session.dom(), top.host(), &cancel())
        .unwrap();
    let shutdown = session.try_shutdown();
    assert!(
        matches!(&shutdown, DomShutdownOutcome::Complete),
        "actual shutdown outcome after real remaining top/Host teardown: {shutdown:?}"
    );
}

#[test]
fn failed_retirement_detach_and_adoption_keep_close_on_the_live_receiver_path() {
    for selected in 0..3 {
        let (mut session, top, _, subject) = fixture();
        let destination = if selected == 2 {
            let second = session
                .hosted_execution()
                .create_top_level(&mut session.dom(), top.host(), &cancel())
                .unwrap();
            let snapshot = session.snapshot();
            let html = snapshot.node(second.document().node()).unwrap().children()[0];
            Some((
                second.document(),
                snapshot.node(html).unwrap().children()[1],
            ))
        } else {
            None
        };
        open(&mut session, subject);
        let mutation = match destination {
            Some((_, parent)) => Mutation::Adopt(parent),
            None if selected == 0 => Mutation::FailedRetire,
            None => Mutation::Detach,
        };
        let mut context = MutationContext {
            source: session.source_execution(),
            mutation,
            calls: Vec::new(),
            nested: Vec::new(),
            destruction: None,
        };
        let mut registry = mutation_registry(&mut session, subject);
        let result = DocumentString::from("tail reached");
        let request = start_close(&mut session, subject, Some(result.clone()));
        let receipt = service(&mut session, &request, &mut registry, &mut context);
        assert_eq!(receipt.callbacks(), 1);
        assert!(receipt.failures().is_empty());
        assert!(!receipt.default_prevented());
        if selected == 0 {
            assert!(matches!(
                context.destruction.as_ref().unwrap(),
                Err(DomError::Cancelled)
            ));
        } else {
            assert_eq!(context.nested.len(), 1);
        }
        context.nested.clear();
        drop(complete(resume(&mut session, request.operation()).unwrap()));
        let snapshot = session.snapshot();
        let dialog = snapshot.node(subject).unwrap().as_dialog().unwrap();
        assert_eq!(dialog.return_value(), &result);
        assert!(!dialog.has_close_watcher());
        assert_eq!(
            snapshot.node(subject).unwrap().attribute(&"open".into()),
            None
        );
        if let Some((document, parent)) = destination {
            assert_eq!(
                snapshot.node(subject).unwrap().owner_document(),
                Some(document)
            );
            assert_eq!(snapshot.node(subject).unwrap().parent(), Some(parent));
        } else if selected == 1 {
            assert_eq!(snapshot.node(subject).unwrap().parent(), None);
        }
        let tasks = session.element_task_execution();
        assert_eq!(tasks.pending_total_tasks().unwrap(), 2);
        for source in [
            DomElementTaskSourceKind::DomManipulation,
            DomElementTaskSourceKind::UserInteraction,
        ] {
            let task = tasks.pending_task(source, 0).unwrap().unwrap();
            assert_eq!(task.id().target(), subject);
            assert_eq!(task.queue_association().unwrap().global(), top.global());
            assert_eq!(task.queue_association().unwrap().document(), top.document());
        }
        assert!(session.dom().pending_source_edit().is_none());
        assert_eq!(context.calls, ["first"]);
    }
}

#[test]
fn completed_show_then_close_replaces_toggle_with_equal_states_but_keeps_plain_close() {
    struct Adoption {
        calls: usize,
        path: Vec<DomEventTarget>,
        result: Option<Result<DomCommit, DomError>>,
    }
    let (mut session, a, body_a, subject) = fixture_with_profile(true);
    let mut hosted = session.hosted_execution();
    let b = hosted
        .create_top_level(&mut session.dom(), a.host(), &cancel())
        .unwrap();
    let created = session.snapshot();
    let html_b = created.node(b.document().node()).unwrap().children()[0];
    let body_b = created.node(html_b).unwrap().children()[1];
    let sentinel = {
        let mut batch = DomBatch::new();
        let sentinel = batch.create_element_ns(b.document(), Namespace::html(), "dialog");
        batch.insert_before(&sentinel, body_b, None);
        session
            .dom()
            .apply(batch, &cancel())
            .unwrap()
            .created(&sentinel)
            .unwrap()
            .unwrap()
    };
    open(&mut session, sentinel);
    let watched = session.snapshot();
    let watcher_b = watched
        .node(sentinel)
        .unwrap()
        .as_dialog()
        .unwrap()
        .close_watcher()
        .unwrap();
    assert_eq!(watcher_b.global(), b.global());
    destroy_top(&mut session, &mut hosted, b);
    let retired = session.snapshot();
    assert!(
        !retired
            .document(b.document())
            .unwrap()
            .is_fully_active()
            .unwrap()
    );
    assert_eq!(
        retired
            .node(sentinel)
            .unwrap()
            .as_dialog()
            .unwrap()
            .close_watcher(),
        Some(watcher_b)
    );
    assert!(
        hosted
            .settings(&session.dom(), a.global(), &cancel())
            .is_ok()
    );
    assert!(matches!(
        hosted.settings(&session.dom(), b.global(), &cancel()),
        Err(DomHostedRuntimeError::Dom(DomHostedError::StaleIdentity))
    ));
    assert_eq!(
        retired.node(subject).unwrap().owner_document(),
        Some(a.document())
    );
    assert_eq!(
        session
            .element_task_execution()
            .pending_total_tasks()
            .unwrap(),
        0
    );

    // Use the accepted negative-focus Show route: dispatch begins in live A,
    // then one actual target callback adopts the closed receiver into inactive B.
    let show = match apply(&mut session, DomSourceEdit::show_dialog(subject)) {
        DomSourceDriveProgress::Pending {
            request: DomSourceRequest::Event(request),
            reason: DomSourceUnavailable::Event,
        } => request,
        other => panic!("actual Show beforetoggle is required: {other:?}"),
    };
    assert!(show.cancelable());
    let show_event = show.toggle_event().unwrap();
    assert_eq!(show_event.old_state(), HtmlDialogState::Closed);
    assert_eq!(show_event.new_state(), HtmlDialogState::Open);
    assert_eq!(show_event.source(), None);
    let mut show_registry = dispatcher();
    show_registry
        .add_listener(
            &mut session.dom(),
            subject,
            DomEventType::BeforeToggle,
            DomEventCallback::new(move |dom, _, adoption: &mut Adoption, event| {
                adoption.calls += 1;
                adoption.path = event.composed_path().collect();
                let mut batch = DomBatch::new();
                batch.insert_before(subject, body_b, None);
                adoption.result = Some(dom.apply(batch, &cancel()));
                Ok(())
            }),
            DomEventListenerOptions::default(),
        )
        .unwrap();
    let mut adoption = Adoption {
        calls: 0,
        path: Vec::new(),
        result: None,
    };
    let receipt = service(&mut session, &show, &mut show_registry, &mut adoption);
    assert_eq!(receipt.callbacks(), 1);
    assert!(receipt.failures().is_empty());
    assert!(!receipt.default_prevented());
    assert_eq!(adoption.calls, 1);
    assert_eq!(
        adoption.path.last(),
        Some(&DomEventTarget::Global(a.global()))
    );
    drop(adoption.result.take().unwrap().unwrap());
    drop(receipt);
    let adopted = session.snapshot();
    assert_eq!(
        adopted.node(subject).unwrap().owner_document(),
        Some(b.document())
    );
    assert_eq!(adopted.node(subject).unwrap().parent(), Some(body_b));
    assert_eq!(adopted.relevant_global(subject).unwrap(), Some(a.global()));
    let done = complete(resume(&mut session, show.operation()).unwrap());
    assert_eq!(done.operation(), show.operation());
    drop(done);
    let shown = session.snapshot();
    assert_eq!(
        shown.node(subject).unwrap().attribute(&"open".into()),
        Some(&"".into())
    );
    assert!(
        !shown
            .node(subject)
            .unwrap()
            .as_dialog()
            .unwrap()
            .has_close_watcher()
    );
    assert_eq!(
        shown
            .node(sentinel)
            .unwrap()
            .as_dialog()
            .unwrap()
            .close_watcher(),
        Some(watcher_b)
    );
    assert_eq!(
        shown
            .node(subject)
            .unwrap()
            .as_element()
            .unwrap()
            .common_state()
            .html()
            .unwrap()
            .previously_focused_element(),
        None
    );
    assert!(session.dom().pending_source_edit().is_none());
    let tasks = session.element_task_execution();
    assert_eq!(tasks.pending_total_tasks().unwrap(), 1);
    let show_task = tasks
        .pending_task(DomElementTaskSourceKind::DomManipulation, 0)
        .unwrap()
        .unwrap();
    let show_toggle = show_task.dialog_toggle().unwrap();
    assert_eq!(show_toggle.old_state(), HtmlDialogState::Closed);
    assert_eq!(show_toggle.new_state(), HtmlDialogState::Open);
    assert_eq!(show_toggle.source(), None);
    assert_eq!(show_task.queue_association().unwrap().global(), a.global());
    assert_eq!(
        show_task.queue_association().unwrap().document(),
        a.document()
    );
    assert_eq!(
        shown
            .node(subject)
            .unwrap()
            .as_dialog()
            .unwrap()
            .toggle_task_tracker()
            .unwrap()
            .task(),
        show_task.id()
    );

    // Ordinary Source insertion returns the open receiver to A and runs its
    // real watcher/open-list hooks. No queued notification is delivered yet.
    drop(complete(apply(
        &mut session,
        DomSourceEdit::insert_before(subject, body_a, None),
    )));
    let returned = session.snapshot();
    assert_eq!(
        returned.node(subject).unwrap().owner_document(),
        Some(a.document())
    );
    assert_eq!(returned.node(subject).unwrap().parent(), Some(body_a));
    assert_eq!(
        returned.node(subject).unwrap().attribute(&"open".into()),
        Some(&"".into())
    );
    assert_eq!(
        returned
            .node(subject)
            .unwrap()
            .as_dialog()
            .unwrap()
            .close_watcher()
            .unwrap()
            .global(),
        a.global()
    );
    assert_eq!(
        tasks
            .pending_task(DomElementTaskSourceKind::DomManipulation, 0)
            .unwrap(),
        Some(show_task)
    );
    let mut context = context(&session);
    let mut registry = listeners(&mut session, subject, body_a);
    let close = start_close(
        &mut session,
        subject,
        Some(DocumentString::from("closed after show")),
    );
    finish_close(&mut session, &close, &mut registry, &mut context);
    assert_eq!(context.before.len(), 1);
    assert_before(&context.before[0], subject);
    assert_eq!(adoption.calls, 1);
    let closed = session.snapshot();
    assert_eq!(
        closed.node(subject).unwrap().attribute(&"open".into()),
        None
    );
    assert!(
        !closed
            .node(subject)
            .unwrap()
            .as_dialog()
            .unwrap()
            .has_close_watcher()
    );
    assert_eq!(
        closed
            .node(subject)
            .unwrap()
            .as_dialog()
            .unwrap()
            .return_value(),
        &DocumentString::from("closed after show")
    );
    assert_eq!(
        tasks
            .pending_tasks(DomElementTaskSourceKind::DomManipulation)
            .unwrap(),
        1
    );
    assert_eq!(
        tasks
            .pending_tasks(DomElementTaskSourceKind::UserInteraction)
            .unwrap(),
        1
    );
    assert_eq!(tasks.pending_total_tasks().unwrap(), 2);
    let equal = tasks
        .pending_task(DomElementTaskSourceKind::DomManipulation, 0)
        .unwrap()
        .unwrap();
    let plain = tasks
        .pending_task(DomElementTaskSourceKind::UserInteraction, 0)
        .unwrap()
        .unwrap();
    assert_ne!(equal.id(), show_task.id());
    assert_ne!(plain.id(), equal.id());
    assert_ne!(plain.id(), show_task.id());
    let equal_toggle = equal.dialog_toggle().unwrap();
    assert_eq!(equal_toggle.old_state(), HtmlDialogState::Closed);
    assert_eq!(equal_toggle.new_state(), HtmlDialogState::Closed);
    assert_eq!(equal_toggle.source(), None);
    assert!(plain.dialog_close().is_some());
    for task in [equal, plain] {
        assert_eq!(task.id().target(), subject);
        assert_eq!(task.queue_association().unwrap().global(), a.global());
        assert_eq!(task.queue_association().unwrap().document(), a.document());
    }
    assert_eq!(
        closed
            .node(subject)
            .unwrap()
            .as_dialog()
            .unwrap()
            .toggle_task_tracker()
            .unwrap()
            .task(),
        equal.id()
    );

    // HTML877d46727–46 keeps first-old even when it equals latest-new. The
    // replacement still fires; the independently queued Close is not coalesced.
    assert_eq!(
        run(
            &mut session,
            DomElementTaskSourceKind::DomManipulation,
            &mut registry,
            &mut context
        ),
        equal
    );
    assert_eq!(context.delivered.len(), 1);
    assert_eq!(context.delivered[0].kind, DomEventType::Toggle);
    let event = context.delivered[0].payload.unwrap();
    assert_eq!(event.old_state(), "closed");
    assert_eq!(event.new_state(), "closed");
    assert_eq!(event.source(), None);
    assert_eq!(
        session
            .snapshot()
            .node(subject)
            .unwrap()
            .as_dialog()
            .unwrap()
            .toggle_task_tracker(),
        None
    );
    assert_eq!(
        tasks
            .pending_tasks(DomElementTaskSourceKind::UserInteraction)
            .unwrap(),
        1
    );
    assert_eq!(
        run(
            &mut session,
            DomElementTaskSourceKind::UserInteraction,
            &mut registry,
            &mut context
        ),
        plain
    );
    assert_eq!(context.delivered.len(), 2);
    assert_eq!(context.delivered[1].kind, DomEventType::Close);
    assert_eq!(context.delivered[1].payload, None);
    let html_a = closed.node(a.document().node()).unwrap().children()[0];
    let path = [
        DomEventTarget::Node(subject),
        DomEventTarget::Node(body_a),
        DomEventTarget::Node(html_a),
        DomEventTarget::Node(a.document().node()),
        DomEventTarget::Global(a.global()),
    ];
    for event in &context.delivered {
        assert_eq!(event.target, Some(subject));
        assert_eq!(event.current, Some(DomEventTarget::Node(subject)));
        assert_eq!(event.path, path);
        assert!(!event.bubbles);
        assert!(!event.cancelable);
    }
    assert!(context.edits.iter().all(Result::is_ok));
    assert_eq!(tasks.pending_total_tasks().unwrap(), 0);
    assert!(session.dom().pending_source_edit().is_none());
    drop(complete(apply(
        &mut session,
        DomSourceEdit::remove_attribute(sentinel, "open"),
    )));
    assert!(
        !session
            .snapshot()
            .node(sentinel)
            .unwrap()
            .as_dialog()
            .unwrap()
            .has_close_watcher()
    );
    assert_eq!(
        session
            .snapshot()
            .node(sentinel)
            .unwrap()
            .attribute(&"open".into()),
        None
    );
    assert_eq!(
        watched
            .node(sentinel)
            .unwrap()
            .as_dialog()
            .unwrap()
            .close_watcher(),
        Some(watcher_b)
    );
    assert_eq!(
        shown
            .node(subject)
            .unwrap()
            .as_dialog()
            .unwrap()
            .toggle_task_tracker()
            .unwrap()
            .task(),
        show_task.id()
    );
    assert_eq!(show_toggle.old_state(), HtmlDialogState::Closed);
    assert_eq!(show_toggle.new_state(), HtmlDialogState::Open);
    for source in [
        DomElementTaskSourceKind::DomManipulation,
        DomElementTaskSourceKind::UserInteraction,
    ] {
        assert!(matches!(
            DomElementTaskRuntime::run_next(
                &mut session.dom(),
                source,
                &mut registry,
                &mut context,
                run_limits(),
                &cancel()
            )
            .unwrap(),
            DomElementTaskProgress::Empty
        ));
    }
    assert_eq!(context.delivered.len(), 2);
    assert_eq!(adoption.calls, 1);
}

struct CloseRetirementContext {
    owner: DomElementTaskRuntime,
    started_retirement: bool,
    calls: Vec<&'static str>,
    delivered: Vec<EventObservation>,
    destruction: Option<Result<DomCommit, DomError>>,
}
impl DomElementTaskEventContext for CloseRetirementContext {
    fn element_tasks(&mut self) -> &mut DomElementTaskRuntime {
        &mut self.owner
    }
}

#[test]
fn queued_plain_close_retirement_before_or_during_dispatch_settles_original_custody() {
    for started_retirement in [false, true] {
        let (mut session, top, body, subject) = fixture();
        open(&mut session, subject);
        let mut method_context = context(&session);
        let mut method_listeners = listeners(&mut session, subject, body);
        let event = start_close(&mut session, subject, Some("closed result".into()));
        finish_close(
            &mut session,
            &event,
            &mut method_listeners,
            &mut method_context,
        );
        assert_eq!(method_context.before.len(), 1);
        assert_before(&method_context.before[0], subject);
        // Drain the independent Toggle source first. Only the genuine plain
        // Close notification remains when Session custody is observed below.
        run(
            &mut session,
            DomElementTaskSourceKind::DomManipulation,
            &mut method_listeners,
            &mut method_context,
        );
        assert_eq!(method_context.delivered.len(), 1);
        assert_eq!(method_context.delivered[0].kind, DomEventType::Toggle);
        assert!(method_context.edits.iter().all(Result::is_ok));
        let tasks = session.element_task_execution();
        let close = tasks
            .pending_task(DomElementTaskSourceKind::UserInteraction, 0)
            .unwrap()
            .unwrap();
        assert!(close.dialog_close().is_some());
        assert_eq!(close.id().target(), subject);
        assert_eq!(close.queue_association().unwrap().global(), top.global());
        assert_eq!(
            close.queue_association().unwrap().document(),
            top.document()
        );
        assert_eq!(tasks.pending_total_tasks().unwrap(), 1);
        let before = session.snapshot();
        let dialog = before.node(subject).unwrap().as_dialog().unwrap();
        assert!(!dialog.has_close_watcher());
        assert_eq!(dialog.toggle_task_tracker(), None);
        assert_eq!(
            dialog.return_value(),
            &DocumentString::from("closed result")
        );
        assert_eq!(
            before.node(subject).unwrap().attribute(&"open".into()),
            None
        );
        assert!(session.dom().pending_source_edit().is_none());
        let mut context = CloseRetirementContext {
            owner: session.element_task_execution(),
            started_retirement,
            calls: Vec::new(),
            delivered: Vec::new(),
            destruction: None,
        };
        let mut registry = dispatcher();
        registry
            .add_listener(
                &mut session.dom(),
                subject,
                DomEventType::Close,
                DomEventCallback::new(
                    move |dom, _, context: &mut CloseRetirementContext, event| {
                        context.calls.push("first");
                        context.delivered.push(observe(event));
                        assert!(context.started_retirement);
                        let mut batch = DomBatch::new();
                        batch.detach(subject);
                        batch.destroy(subject);
                        context.destruction = Some(dom.apply(batch, &cancel()));
                        Ok(())
                    },
                ),
                DomEventListenerOptions::default(),
            )
            .unwrap();
        let late = registry
            .add_listener(
                &mut session.dom(),
                subject,
                DomEventType::Close,
                DomEventCallback::new(move |_, _, context: &mut CloseRetirementContext, event| {
                    context.calls.push("late");
                    context.delivered.push(observe(event));
                    Ok(())
                }),
                DomEventListenerOptions {
                    capture: false,
                    once: true,
                },
            )
            .unwrap();
        if !started_retirement {
            let mut batch = DomBatch::new();
            batch.detach(subject);
            batch.destroy(subject);
            session.dom().apply(batch, &cancel()).unwrap();
            assert!(matches!(
                session.snapshot().node(subject),
                Err(DomError::StaleNode)
            ));
        }
        // The same Session must retain the undelivered UI task even when its
        // receiver is already retired; an execution permission owns no bank.
        let mut session = match session.try_shutdown() {
            DomShutdownOutcome::Pending { session, reason } => {
                assert_eq!(reason, DomShutdownReason::DomElementTask(close.id()));
                session
            }
            DomShutdownOutcome::Complete => panic!("the real queued Close still needs settlement"),
        };
        assert_eq!(session.snapshot().owner(), before.owner());
        assert_eq!(tasks.pending_total_tasks().unwrap(), 1);
        assert_eq!(
            tasks
                .pending_task(DomElementTaskSourceKind::UserInteraction, 0)
                .unwrap(),
            Some(close)
        );
        let progress = DomElementTaskRuntime::run_next(
            &mut session.dom(),
            DomElementTaskSourceKind::UserInteraction,
            &mut registry,
            &mut context,
            run_limits(),
            &cancel(),
        )
        .unwrap();
        if started_retirement {
            let DomElementTaskProgress::Complete {
                task,
                receipt,
                dialog_toggle_disposition,
                displaced_owner,
            } = progress
            else {
                panic!("started plain Close must complete its authentic receipt: {progress:?}");
            };
            assert_eq!(task, close);
            assert_eq!(dialog_toggle_disposition, None);
            assert!(displaced_owner.is_none());
            assert_eq!(receipt.original_target(), subject);
            assert_eq!(receipt.event_type(), DomEventType::Close);
            assert_eq!(receipt.callbacks(), 2);
            assert!(receipt.failures().is_empty());
            assert!(!receipt.default_prevented());
            assert_eq!(context.calls, ["first", "late"]);
            assert!(context.destruction.as_ref().unwrap().is_ok());
            assert!(!registry.remove_listener(late));
            for delivered in &context.delivered {
                assert_eq!(delivered.kind, DomEventType::Close);
                assert_eq!(delivered.target, Some(subject));
                assert_eq!(delivered.current, Some(DomEventTarget::Node(subject)));
                assert!(delivered.payload.is_none());
                assert!(!delivered.cancelable);
                assert!(!delivered.bubbles);
                assert!(!delivered.prevented);
            }
            drop(receipt);
        } else {
            assert!(
                matches!(progress, DomElementTaskProgress::SkippedTargetDestroyed(task) if task == close)
            );
            assert!(context.calls.is_empty());
            assert!(context.delivered.is_empty());
            assert!(context.destruction.is_none());
            assert!(registry.remove_listener(late));
        }
        drop(context.destruction.take());
        assert!(matches!(
            session.snapshot().node(subject),
            Err(DomError::StaleNode)
        ));
        assert!(session.dom().pending_source_edit().is_none());
        assert_eq!(session.snapshot().incomplete_source_operation(), None);
        assert_eq!(tasks.pending_total_tasks().unwrap(), 0);
        assert!(matches!(
            DomElementTaskRuntime::run_next(
                &mut session.dom(),
                DomElementTaskSourceKind::UserInteraction,
                &mut registry,
                &mut context,
                run_limits(),
                &cancel(),
            )
            .unwrap(),
            DomElementTaskProgress::Empty
        ));
        assert_eq!(context.calls.len(), if started_retirement { 2 } else { 0 });
        assert_eq!(
            before
                .node(subject)
                .unwrap()
                .as_dialog()
                .unwrap()
                .return_value(),
            &DocumentString::from("closed result")
        );
        let mut session = match session.try_shutdown() {
            DomShutdownOutcome::Pending {
                session,
                reason: DomShutdownReason::HostedLifecycle,
            } => session,
            other => panic!("notification Source/event/task custody must be settled: {other:?}"),
        };
        let mut hosted = session.hosted_execution();
        destroy_top(&mut session, &mut hosted, top);
        hosted
            .destroy_host(&mut session.dom(), top.host(), &cancel())
            .unwrap();
        let shutdown = session.try_shutdown();
        assert!(
            matches!(&shutdown, DomShutdownOutcome::Complete),
            "real teardown must release all custody: {shutdown:?}"
        );
        assert!(matches!(
            tasks.pending_total_tasks(),
            Err(DomElementTaskRuntimeError::OwnerDisposed)
        ));
        assert!(matches!(
            context.owner.pending_total_tasks(),
            Err(DomElementTaskRuntimeError::OwnerDisposed)
        ));
    }
}

#[path = "dom_source_close/toggle_reentry.rs"]
mod toggle_reentry;

#[path = "dom_source_close/toggle_cleanup.rs"]
mod toggle_cleanup;
