//! HTML877d46727–46: an executing Toggle's cleanup clears the current tracker.
use super::*;

fn show(session: &mut DomSession, subject: NodeId) {
    let event = match apply(session, DomSourceEdit::show_dialog(subject)) {
        DomSourceDriveProgress::Pending {
            request: DomSourceRequest::Event(event),
            reason: DomSourceUnavailable::Event,
        } => event,
        other => panic!("actual Show beforetoggle required: {other:?}"),
    };
    assert!(event.cancelable());
    let payload = event.toggle_event().unwrap();
    assert_eq!(payload.old_state(), HtmlDialogState::Closed);
    assert_eq!(payload.new_state(), HtmlDialogState::Open);
    assert_eq!(payload.source(), None);
    let mut registry = dispatcher();
    registry
        .add_listener(
            &mut session.dom(),
            subject,
            DomEventType::BeforeToggle,
            DomEventCallback::new(|_, _, calls: &mut usize, event| {
                assert!(event.cancelable());
                *calls += 1;
                Ok(())
            }),
            DomEventListenerOptions {
                capture: false,
                once: true,
            },
        )
        .unwrap();
    let mut calls = 0;
    let receipt = service(session, &event, &mut registry, &mut calls);
    assert_eq!(calls, 1);
    assert_eq!(receipt.callbacks(), 1);
    assert!(receipt.failures().is_empty());
    assert!(!receipt.default_prevented());
    let done = complete(resume(session, event.operation()).unwrap());
    assert_eq!(done.operation(), event.operation());
    assert!(session.dom().pending_source_edit().is_none());
}

struct Reentry {
    owner: DomElementTaskRuntime,
    source: DomSourceRuntime,
    before: DomEventDispatcher<usize>,
    before_calls: usize,
    parent: Option<DomSourceOperationId>,
    closed: Option<DomSnapshot>,
    replacement: Option<DomElementTask>,
    plain_close: Option<DomElementTask>,
    callbacks: Vec<&'static str>,
    delivered: Vec<EventObservation>,
    edits: Vec<Result<DomCommit, DomError>>,
    later_calls: usize,
}
impl DomElementTaskEventContext for Reentry {
    fn element_tasks(&mut self) -> &mut DomElementTaskRuntime {
        &mut self.owner
    }
}
fn nested_close(dom: &mut DomEventDom<'_>, context: &mut Reentry, subject: NodeId) {
    let parent = dom
        .pending_source_edit()
        .expect("the executing Toggle owns its actual Source root")
        .operation();
    context.parent = Some(parent);
    let request = match context
        .source
        .apply(
            dom,
            DomSourceEdit::close_dialog(subject, Some("closed in first Toggle".into())),
            source_limits(),
            &mut DomSourceServices {
                meta: None,
                execution_scope: None,
                style_policy: &mut |_| None,
                url: None,
            },
            &cancel(),
        )
        .unwrap()
    {
        DomSourceDriveProgress::Pending {
            request: DomSourceRequest::Event(event),
            reason: DomSourceUnavailable::Event,
        } => event,
        other => panic!("real nested Close beforetoggle required: {other:?}"),
    };
    assert_ne!(request.operation(), parent);
    assert_eq!(request.parent_operation(), Some(parent));
    assert!(!request.cancelable());
    let payload = request.toggle_event().unwrap();
    assert_eq!(payload.old_state(), HtmlDialogState::Open);
    assert_eq!(payload.new_state(), HtmlDialogState::Closed);
    assert_eq!(payload.source(), None);
    match context
        .source
        .service_event(
            dom,
            &request,
            &mut context.before,
            &mut context.before_calls,
            DomSourceEventServiceLimits {
                source: source_limits(),
                dispatch: dispatch_limits(),
            },
            &cancel(),
        )
        .unwrap()
    {
        DomSourceEventServiceProgress::Serviced { receipt } => {
            assert_eq!(receipt.callbacks(), 1);
            assert!(receipt.failures().is_empty());
            assert!(!receipt.default_prevented());
        }
        other => panic!("nested Close must finish its authentic receipt: {other:?}"),
    }
    let done = complete(
        context
            .source
            .resume(
                dom,
                request.operation(),
                source_limits(),
                &mut DomSourceServices {
                    meta: None,
                    execution_scope: None,
                    style_policy: &mut |_| None,
                    url: None,
                },
                &cancel(),
            )
            .unwrap(),
    );
    assert_eq!(done.operation(), request.operation());
    assert_eq!(dom.pending_source_edit().unwrap().operation(), parent);
    let closed = dom.snapshot();
    let dialog = closed.node(subject).unwrap().as_dialog().unwrap();
    assert_eq!(
        closed.node(subject).unwrap().attribute(&"open".into()),
        None
    );
    assert!(!dialog.has_close_watcher());
    assert_eq!(
        dialog.return_value(),
        &DocumentString::from("closed in first Toggle")
    );
    // The executing first task remains retained until its actual receipt and
    // cleanup. Only the not-started replacement is newly queued in this source.
    assert_eq!(
        context
            .owner
            .pending_tasks(DomElementTaskSourceKind::DomManipulation)
            .unwrap(),
        2
    );
    let replacement = context
        .owner
        .pending_task(DomElementTaskSourceKind::DomManipulation, 1)
        .unwrap()
        .unwrap();
    let plain = context
        .owner
        .pending_task(DomElementTaskSourceKind::UserInteraction, 0)
        .unwrap()
        .unwrap();
    assert_eq!(
        dialog.toggle_task_tracker().unwrap().task(),
        replacement.id()
    );
    let payload = replacement.dialog_toggle().unwrap();
    assert_eq!(payload.old_state(), HtmlDialogState::Closed);
    assert_eq!(payload.new_state(), HtmlDialogState::Closed);
    assert_eq!(payload.source(), None);
    assert!(plain.dialog_close().is_some());
    context.closed = Some(closed);
    context.replacement = Some(replacement);
    context.plain_close = Some(plain);
}
fn check_delivery(progress: DomElementTaskProgress, expected: DomElementTask, callbacks: usize) {
    match progress {
        DomElementTaskProgress::Complete {
            task,
            receipt,
            dialog_toggle_disposition,
            displaced_owner,
        } => {
            assert_eq!(task, expected);
            assert_eq!(
                dialog_toggle_disposition,
                expected
                    .dialog_toggle()
                    .map(|_| DomDialogToggleTaskDisposition::TrackerCleared)
            );
            assert!(displaced_owner.is_none());
            assert_eq!(receipt.original_target(), expected.id().target());
            assert_eq!(
                receipt.event_type(),
                if expected.dialog_close().is_some() {
                    DomEventType::Close
                } else {
                    DomEventType::Toggle
                }
            );
            assert_eq!(receipt.callbacks(), callbacks);
            assert!(
                receipt.failures().is_empty(),
                "actual callback failures: {:?}",
                receipt.failures()
            );
            assert!(!receipt.default_prevented());
        }
        other => panic!("actual selected task must finish its receipt and cleanup: {other:?}"),
    }
}
fn deliver(
    session: &mut DomSession,
    source: DomElementTaskSourceKind,
    registry: &mut DomEventDispatcher<Reentry>,
    context: &mut Reentry,
    expected: DomElementTask,
    callbacks: usize,
) {
    let progress = DomElementTaskRuntime::run_next(
        &mut session.dom(),
        source,
        registry,
        context,
        run_limits(),
        &cancel(),
    )
    .unwrap();
    check_delivery(progress, expected, callbacks);
}
fn exercise(retain: bool) {
    let (mut session, top, body, subject) = fixture_with_profile(true);
    // Actual connected insertion is already finished; setting autofocus here
    // does not enroll a candidate or invoke insertion-time autofocus processing.
    for name in ["inert", "autofocus"] {
        drop(complete(apply(
            &mut session,
            DomSourceEdit::set_attribute(subject, name, ""),
        )));
    }
    let ready = session.snapshot();
    show(&mut session, subject);
    let shown = session.snapshot();
    let first_watcher = shown
        .node(subject)
        .unwrap()
        .as_dialog()
        .unwrap()
        .close_watcher()
        .unwrap();
    let tasks = session.element_task_execution();
    let first = tasks
        .pending_task(DomElementTaskSourceKind::DomManipulation, 0)
        .unwrap()
        .unwrap();
    let first_payload = first.dialog_toggle().unwrap();
    assert_eq!(first_payload.old_state(), HtmlDialogState::Closed);
    assert_eq!(first_payload.new_state(), HtmlDialogState::Open);
    assert_eq!(first_payload.source(), None);
    assert_eq!(
        shown
            .node(subject)
            .unwrap()
            .as_dialog()
            .unwrap()
            .toggle_task_tracker()
            .unwrap()
            .task(),
        first.id()
    );
    let mut before = dispatcher();
    let before_listener = before
        .add_listener(
            &mut session.dom(),
            subject,
            DomEventType::BeforeToggle,
            DomEventCallback::new(|_, _, calls: &mut usize, event| {
                *calls += 1;
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
    let mut context = Reentry {
        owner: session.element_task_execution(),
        source: session.source_execution(),
        before,
        before_calls: 0,
        parent: None,
        closed: None,
        replacement: None,
        plain_close: None,
        callbacks: Vec::new(),
        delivered: Vec::new(),
        edits: Vec::new(),
        later_calls: 0,
    };
    let mut registry = dispatcher();
    let first_listener = registry
        .add_listener(
            &mut session.dom(),
            subject,
            DomEventType::Toggle,
            DomEventCallback::new(move |dom, _, context: &mut Reentry, event| {
                context.callbacks.push("first");
                context.delivered.push(observe(event));
                nested_close(dom, context, subject);
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
            &mut session.dom(),
            subject,
            DomEventType::Toggle,
            DomEventCallback::new(move |dom, _, context: &mut Reentry, event| {
                context.callbacks.push("later");
                context.delivered.push(observe(event));
                context.later_calls += 1;
                let mut batch = DomBatch::new();
                batch.set_attribute(
                    body,
                    "data-toggle-callback",
                    context.later_calls.to_string(),
                );
                context.edits.push(dom.apply(batch, &cancel()));
                Ok(())
            }),
            DomEventListenerOptions::default(),
        )
        .unwrap();
    registry
        .add_listener(
            &mut session.dom(),
            subject,
            DomEventType::Close,
            DomEventCallback::new(move |dom, _, context: &mut Reentry, event| {
                context.callbacks.push("close");
                context.delivered.push(observe(event));
                let mut batch = DomBatch::new();
                batch.set_attribute(body, "data-plain-close", "seen");
                context.edits.push(dom.apply(batch, &cancel()));
                Ok(())
            }),
            DomEventListenerOptions {
                capture: false,
                once: true,
            },
        )
        .unwrap();
    let mut limits = run_limits();
    if retain {
        limits.dispatch.callbacks = 1;
    }
    let progress = DomElementTaskRuntime::run_next(
        &mut session.dom(),
        DomElementTaskSourceKind::DomManipulation,
        &mut registry,
        &mut context,
        limits,
        &cancel(),
    )
    .unwrap();
    if retain {
        assert!(
            matches!(progress, DomElementTaskProgress::Retained { task, dispatch:DomEventDispatchOutcome::Retained(DomEventResource::Callbacks), displaced_owner:None } if task==first)
        );
        assert_eq!(context.callbacks, ["first"]);
        assert_eq!(context.before_calls, 1);
        assert_eq!(context.later_calls, 0);
        let replacement = context.replacement.unwrap();
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
            replacement.id()
        );
        assert_eq!(
            tasks
                .pending_task(DomElementTaskSourceKind::DomManipulation, 0)
                .unwrap(),
            Some(first)
        );
        assert_eq!(
            session.dom().pending_source_edit().unwrap().operation(),
            context.parent.unwrap()
        );
        assert!(!registry.remove_listener(first_listener));
        context.owner = session.element_task_execution();
        deliver(
            &mut session,
            DomElementTaskSourceKind::DomManipulation,
            &mut registry,
            &mut context,
            first,
            2,
        );
    } else {
        check_delivery(progress, first, 2);
    }
    assert_eq!(context.callbacks, ["first", "later"]);
    assert_eq!(context.before_calls, 1);
    assert_eq!(context.later_calls, 1);
    assert!(!context.before.remove_listener(before_listener));
    assert!(!registry.remove_listener(first_listener));
    assert!(context.edits.iter().all(Result::is_ok));
    let replacement = context.replacement.unwrap();
    let plain = context.plain_close.unwrap();
    for task in [first, replacement, plain] {
        assert_eq!(task.id().target(), subject);
        assert_eq!(task.queue_association().unwrap().document(), top.document());
        assert_eq!(task.queue_association().unwrap().global(), top.global());
    }
    assert_ne!(first.id(), replacement.id());
    assert_ne!(replacement.id(), plain.id());
    let closed = session.snapshot();
    assert_eq!(
        closed
            .node(subject)
            .unwrap()
            .as_dialog()
            .unwrap()
            .toggle_task_tracker(),
        None
    );
    assert_eq!(
        tasks
            .pending_task(DomElementTaskSourceKind::DomManipulation, 0)
            .unwrap(),
        Some(replacement)
    );
    assert_eq!(
        tasks
            .pending_task(DomElementTaskSourceKind::UserInteraction, 0)
            .unwrap(),
        Some(plain)
    );
    assert_eq!(tasks.pending_total_tasks().unwrap(), 2);
    assert!(session.dom().pending_source_edit().is_none());
    // The completed first Toggle cleared the newer tracker unconditionally,
    // but not the replacement task. A later Show must therefore append a third
    // task rather than coalesce away that now-untracked replacement.
    show(&mut session, subject);
    let reopened = session.snapshot();
    let next = tasks
        .pending_task(DomElementTaskSourceKind::DomManipulation, 1)
        .unwrap()
        .unwrap();
    assert_ne!(next.id(), first.id());
    assert_ne!(next.id(), replacement.id());
    assert_eq!(
        tasks
            .pending_task(DomElementTaskSourceKind::DomManipulation, 0)
            .unwrap(),
        Some(replacement)
    );
    assert_eq!(
        reopened
            .node(subject)
            .unwrap()
            .as_dialog()
            .unwrap()
            .toggle_task_tracker()
            .unwrap()
            .task(),
        next.id()
    );
    assert_ne!(
        reopened
            .node(subject)
            .unwrap()
            .as_dialog()
            .unwrap()
            .close_watcher()
            .unwrap(),
        first_watcher
    );
    let payload = next.dialog_toggle().unwrap();
    assert_eq!(payload.old_state(), HtmlDialogState::Closed);
    assert_eq!(payload.new_state(), HtmlDialogState::Open);
    assert_eq!(payload.source(), None);
    assert_eq!(next.queue_association(), first.queue_association());
    // Sources are selected independently. Only the DOM-manipulation order
    // replacement→next is required; no cross-source event order is prescribed.
    if retain {
        deliver(
            &mut session,
            DomElementTaskSourceKind::UserInteraction,
            &mut registry,
            &mut context,
            plain,
            1,
        );
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
            next.id(),
            "the independent plain Close has no Toggle tracker cleanup"
        );
    }
    deliver(
        &mut session,
        DomElementTaskSourceKind::DomManipulation,
        &mut registry,
        &mut context,
        replacement,
        1,
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
    assert_eq!(
        tasks
            .pending_task(DomElementTaskSourceKind::DomManipulation, 0)
            .unwrap(),
        Some(next)
    );
    deliver(
        &mut session,
        DomElementTaskSourceKind::DomManipulation,
        &mut registry,
        &mut context,
        next,
        1,
    );
    if !retain {
        deliver(
            &mut session,
            DomElementTaskSourceKind::UserInteraction,
            &mut registry,
            &mut context,
            plain,
            1,
        );
    }
    assert_eq!(context.before_calls, 1);
    assert_eq!(context.later_calls, 3);
    assert_eq!(
        context
            .callbacks
            .iter()
            .filter(|label| **label == "first")
            .count(),
        1
    );
    assert_eq!(
        context
            .callbacks
            .iter()
            .filter(|label| **label == "close")
            .count(),
        1
    );
    assert!(context.edits.iter().all(Result::is_ok));
    let toggles: Vec<_> = context
        .delivered
        .iter()
        .filter(|event| event.kind == DomEventType::Toggle)
        .collect();
    let html = ready.node(top.document().node()).unwrap().children()[0];
    let expected_path = [
        DomEventTarget::Node(subject),
        DomEventTarget::Node(body),
        DomEventTarget::Node(html),
        DomEventTarget::Node(top.document().node()),
        DomEventTarget::Global(top.global()),
    ];
    // First and later callbacks observe the same initial event. Later-only
    // delivery then preserves each task's literal queued payload, including
    // the equal-state replacement that is still required to fire.
    assert_eq!(toggles.len(), 4);
    for (event, old, new) in [
        (toggles[0], "closed", "open"),
        (toggles[1], "closed", "open"),
        (toggles[2], "closed", "closed"),
        (toggles[3], "closed", "open"),
    ] {
        assert_eq!(event.target, Some(subject));
        assert_eq!(event.current, Some(DomEventTarget::Node(subject)));
        assert!(!event.bubbles);
        assert!(!event.cancelable);
        assert!(!event.prevented);
        let payload = event.payload.unwrap();
        assert_eq!(payload.old_state(), old);
        assert_eq!(payload.new_state(), new);
        assert_eq!(payload.source(), None);
        assert_eq!(event.path, expected_path);
    }
    let close: Vec<_> = context
        .delivered
        .iter()
        .filter(|event| event.kind == DomEventType::Close)
        .collect();
    assert_eq!(close.len(), 1);
    assert_eq!(close[0].payload, None);
    assert_eq!(close[0].target, Some(subject));
    assert!(!close[0].cancelable);
    assert!(!close[0].bubbles);
    assert_eq!(close[0].current, Some(DomEventTarget::Node(subject)));
    assert!(!close[0].prevented);
    assert_eq!(close[0].path, expected_path);
    let final_state = session.snapshot();
    assert_eq!(
        final_state
            .node(body)
            .unwrap()
            .attribute(&"data-toggle-callback".into()),
        Some(&"3".into())
    );
    assert_eq!(
        final_state
            .node(body)
            .unwrap()
            .attribute(&"data-plain-close".into()),
        Some(&"seen".into())
    );
    assert_eq!(
        final_state.node(subject).unwrap().attribute(&"open".into()),
        Some(&"".into())
    );
    assert_eq!(
        final_state
            .node(subject)
            .unwrap()
            .as_dialog()
            .unwrap()
            .return_value(),
        &DocumentString::from("closed in first Toggle")
    );
    assert_eq!(
        final_state
            .node(subject)
            .unwrap()
            .as_dialog()
            .unwrap()
            .toggle_task_tracker(),
        None
    );
    assert!(session.dom().pending_source_edit().is_none());
    assert_eq!(final_state.incomplete_source_operation(), None);
    assert_eq!(tasks.pending_total_tasks().unwrap(), 0);
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
    let nested = context.closed.as_ref().unwrap();
    // The live Observed task controller was checked through pending_source_edit
    // inside the callback. It has published no canonical prefix, so the child
    // completion's coherent historical snapshot has no incomplete marker.
    assert_eq!(nested.incomplete_source_operation(), None);
    assert_eq!(
        nested
            .node(subject)
            .unwrap()
            .as_dialog()
            .unwrap()
            .toggle_task_tracker()
            .unwrap()
            .task(),
        replacement.id()
    );
    assert_eq!(
        nested.node(subject).unwrap().attribute(&"open".into()),
        None
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
        first.id()
    );
    assert_eq!(
        shown
            .node(subject)
            .unwrap()
            .as_dialog()
            .unwrap()
            .close_watcher(),
        Some(first_watcher)
    );
    assert_eq!(ready.node(subject).unwrap().attribute(&"open".into()), None);
}

#[test]
fn toggle_callback_close_leaves_equal_state_replacement_untracked_and_next_show_distinct() {
    exercise(false);
}

#[test]
fn retained_toggle_callback_limit_resumes_without_replaying_nested_close() {
    exercise(true);
}
