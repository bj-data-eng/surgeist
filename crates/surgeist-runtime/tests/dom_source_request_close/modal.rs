//! HTML877d46708–23,46625–36,46684–705 through genuine completed modal setup.
use super::*;

fn layer(snapshot: &DomSnapshot, document: DocumentId, subject: NodeId, pending: bool) {
    let layer = snapshot.document(document).unwrap().top_layer().unwrap();
    assert_eq!(layer.raw_members(), [subject]);
    let expected_pending = if pending { vec![subject] } else { Vec::new() };
    assert_eq!(layer.pending_removals(), expected_pending);
    let expected = if pending {
        vec![DomTopLayerUaRule::UniversalOverlayNone]
    } else {
        vec![
            DomTopLayerUaRule::UniversalOverlayNone,
            DomTopLayerUaRule::ElementOverlayAuto(subject),
        ]
    };
    assert_eq!(layer.ua_rules().collect::<Vec<_>>(), expected);
}
fn shown_and_initial_toggle_delivered(
    session: &mut DomSession,
    top: DomHostedTopLevel,
    subject: NodeId,
) -> DomSnapshot {
    // These attributes are set after actual insertion. No candidate enrollment
    // or positive focus observation is supplied by this fixture.
    for name in ["inert", "autofocus"] {
        drop(complete(apply(
            session,
            DomSourceEdit::set_attribute(subject, name, ""),
        )));
    }
    let before = pending_event(
        apply(session, DomSourceEdit::show_modal_dialog(subject)),
        DomSourceEventKind::BeforeToggle,
    );
    assert!(before.cancelable());
    let payload = before.toggle_event().unwrap();
    assert_eq!(payload.old_state(), HtmlDialogState::Closed);
    assert_eq!(payload.new_state(), HtmlDialogState::Open);
    assert_eq!(payload.source(), None);
    let mut registry = dispatcher::<usize>();
    let once = registry
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
    let receipt = service(session, &before, &mut registry, &mut calls);
    assert_eq!(receipt.callbacks(), 1);
    assert!(!receipt.default_prevented());
    assert!(receipt.failures().is_empty());
    assert_eq!(calls, 1);
    assert!(!registry.remove_listener(once));
    drop(receipt);
    let done = complete(resume(session, before.operation()).unwrap());
    assert_eq!(done.operation(), before.operation());
    assert_eq!(done.dialog_toggle_disposition(), None);
    let shown = session.snapshot();
    let node = shown.node(subject).unwrap();
    assert_eq!(node.attribute(&"open".into()), Some(&"".into()));
    let dialog = node.as_dialog().unwrap();
    assert!(dialog.is_modal());
    assert_eq!(dialog.close_watcher().unwrap().global(), top.global());
    assert_eq!(dialog.close_watcher().unwrap().subject(), subject);
    assert_eq!(
        node.as_element()
            .unwrap()
            .common_state()
            .html()
            .unwrap()
            .previously_focused_element(),
        None
    );
    layer(&shown, top.document(), subject, false);
    let history = shown.document(top.document()).unwrap().autofocus().unwrap();
    assert!(history.candidates().is_empty());
    assert!(history.processed());
    let tasks = session.element_task_execution();
    assert_eq!(tasks.pending_total_tasks().unwrap(), 1);
    let task = tasks
        .pending_task(DomElementTaskSourceKind::DomManipulation, 0)
        .unwrap()
        .unwrap();
    assert_eq!(task.queue_association().unwrap().document(), top.document());
    assert_eq!(task.queue_association().unwrap().global(), top.global());
    assert_eq!(dialog.toggle_task_tracker().unwrap().task(), task.id());
    let mut context = QueueContext {
        tasks: session.element_task_execution(),
        delivered: Vec::new(),
    };
    let mut registry = dispatcher::<QueueContext>();
    let once = registry
        .add_listener(
            &mut session.dom(),
            subject,
            DomEventType::Toggle,
            DomEventCallback::new(|_, _, context: &mut QueueContext, event| {
                let payload = event.toggle_event().unwrap();
                assert_eq!(payload.old_state(), "closed");
                assert_eq!(payload.new_state(), "open");
                assert_eq!(payload.source(), None);
                context.delivered.push(DomEventType::Toggle);
                Ok(())
            }),
            DomEventListenerOptions {
                capture: false,
                once: true,
            },
        )
        .unwrap();
    let progress = DomElementTaskRuntime::run_next(
        &mut session.dom(),
        DomElementTaskSourceKind::DomManipulation,
        &mut registry,
        &mut context,
        run_limits(),
        &cancel(),
    )
    .unwrap();
    let DomElementTaskProgress::Complete {
        task: actual,
        receipt,
        dialog_toggle_disposition,
        displaced_owner,
    } = progress
    else {
        panic!("initial real Toggle must deliver: {progress:?}")
    };
    assert_eq!(actual, task);
    assert_eq!(
        dialog_toggle_disposition,
        Some(DomDialogToggleTaskDisposition::TrackerCleared)
    );
    assert!(displaced_owner.is_none());
    assert_eq!(receipt.callbacks(), 1);
    assert!(receipt.failures().is_empty(), "{:?}", receipt.failures());
    assert_eq!(context.delivered, [DomEventType::Toggle]);
    assert!(!registry.remove_listener(once));
    assert_eq!(tasks.pending_total_tasks().unwrap(), 0);
    assert!(session.dom().pending_source_edit().is_none());
    let ready = session.snapshot();
    assert_eq!(
        ready
            .node(subject)
            .unwrap()
            .as_dialog()
            .unwrap()
            .toggle_task_tracker(),
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
        task.id()
    );
    ready
}
fn deliver_closed_tasks(session: &mut DomSession, top: DomHostedTopLevel, subject: NodeId) {
    let mut context = QueueContext {
        tasks: session.element_task_execution(),
        delivered: Vec::new(),
    };
    assert_eq!(context.tasks.pending_total_tasks().unwrap(), 2);
    let toggle = context
        .tasks
        .pending_task(DomElementTaskSourceKind::DomManipulation, 0)
        .unwrap()
        .unwrap();
    let close = context
        .tasks
        .pending_task(DomElementTaskSourceKind::UserInteraction, 0)
        .unwrap()
        .unwrap();
    assert_ne!(toggle.id(), close.id());
    let payload = toggle.dialog_toggle().unwrap();
    assert_eq!(payload.old_state(), HtmlDialogState::Open);
    assert_eq!(payload.new_state(), HtmlDialogState::Closed);
    assert_eq!(payload.source(), None);
    assert!(close.dialog_close().is_some());
    for task in [toggle, close] {
        assert_eq!(task.queue_association().unwrap().document(), top.document());
        assert_eq!(task.queue_association().unwrap().global(), top.global());
    }
    let mut registry = dispatcher::<QueueContext>();
    for kind in [DomEventType::Toggle, DomEventType::Close] {
        registry
            .add_listener(
                &mut session.dom(),
                subject,
                kind,
                DomEventCallback::new(move |_, _, context: &mut QueueContext, event| {
                    assert_eq!(event.event_type(), kind);
                    assert!(!event.cancelable());
                    assert!(!event.bubbles());
                    if kind == DomEventType::Close {
                        assert!(event.toggle_event().is_none());
                    } else {
                        let payload = event.toggle_event().unwrap();
                        assert_eq!(payload.old_state(), "open");
                        assert_eq!(payload.new_state(), "closed");
                        assert_eq!(payload.source(), None);
                    }
                    context.delivered.push(kind);
                    Ok(())
                }),
                DomEventListenerOptions {
                    capture: false,
                    once: true,
                },
            )
            .unwrap();
    }
    // Independent sources are deliberately selected here; this asserts no
    // inter-source scheduling order beyond the caller's explicit selection.
    for expected in [close, toggle] {
        let result = DomElementTaskRuntime::run_next(
            &mut session.dom(),
            expected.source(),
            &mut registry,
            &mut context,
            run_limits(),
            &cancel(),
        )
        .unwrap();
        let DomElementTaskProgress::Complete {
            task,
            receipt,
            dialog_toggle_disposition,
            displaced_owner,
        } = result
        else {
            panic!("actual modal Close task must deliver: {result:?}")
        };
        assert_eq!(task, expected);
        assert_eq!(
            dialog_toggle_disposition,
            expected
                .dialog_toggle()
                .map(|_| DomDialogToggleTaskDisposition::TrackerCleared)
        );
        assert!(displaced_owner.is_none());
        assert_eq!(receipt.original_target(), subject);
        assert_eq!(receipt.callbacks(), 1);
        assert!(receipt.failures().is_empty(), "{:?}", receipt.failures());
        assert!(!receipt.default_prevented());
    }
    assert_eq!(
        context.delivered,
        [DomEventType::Close, DomEventType::Toggle]
    );
    assert_eq!(context.tasks.pending_total_tasks().unwrap(), 0);
    assert!(session.dom().pending_source_edit().is_none());
}
fn teardown(mut session: DomSession, top: DomHostedTopLevel) {
    let tasks = session.element_task_execution();
    assert_eq!(tasks.pending_total_tasks().unwrap(), 0);
    let mut hosted = session.hosted_execution();
    destroy_top(&mut session, &mut hosted, top);
    hosted
        .destroy_host(&mut session.dom(), top.host(), &cancel())
        .unwrap();
    assert!(matches!(
        session.try_shutdown(),
        DomShutdownOutcome::Complete
    ));
    assert!(matches!(
        tasks.pending_total_tasks(),
        Err(DomElementTaskRuntimeError::OwnerDisposed)
    ));
}

#[test]
fn canceled_modal_request_keeps_layer_then_uncanceled_request_closes_with_real_pending_removal_tasks()
 {
    let (mut session, top, _, subject) = fixture_with_profile(true);
    let seed = DocumentString::from("seed modal result");
    seed_return_value(&mut session, subject, &seed);
    let shown = shown_and_initial_toggle_delivered(&mut session, top, subject);
    let original = shown
        .node(subject)
        .unwrap()
        .as_dialog()
        .unwrap()
        .close_watcher();
    let mut registry = dispatcher::<usize>();
    registry
        .add_listener(
            &mut session.dom(),
            subject,
            DomEventType::Cancel,
            DomEventCallback::new(|_, _, calls: &mut usize, event| {
                *calls += 1;
                assert!(event.cancelable());
                assert!(event.toggle_event().is_none());
                if *calls == 1 {
                    event.prevent_default();
                }
                Ok(())
            }),
            DomEventListenerOptions::default(),
        )
        .unwrap();
    let canceled_result = DocumentString::from("canceled modal request");
    let request = start_request(&mut session, subject, Some(canceled_result.clone()));
    let mut calls = 0;
    let receipt = service(&mut session, &request, &mut registry, &mut calls);
    assert_eq!(receipt.callbacks(), 1);
    assert!(receipt.default_prevented());
    assert!(receipt.failures().is_empty());
    drop(receipt);
    drop(complete(resume(&mut session, request.operation()).unwrap()));
    let canceled = session.snapshot();
    let state = canceled.node(subject).unwrap().as_dialog().unwrap();
    assert!(state.is_modal());
    assert_eq!(
        canceled.node(subject).unwrap().attribute(&"open".into()),
        Some(&"".into())
    );
    assert_eq!(state.close_watcher(), original);
    assert_eq!(state.return_value(), &seed);
    layer(&canceled, top.document(), subject, false);
    assert_surviving_reset(&session, subject, Some(&canceled_result));
    assert_eq!(
        session
            .element_task_execution()
            .pending_total_tasks()
            .unwrap(),
        0
    );
    // Starting and delivering another actual Cancel proves the original native
    // running-cancel guard was reset after the prevented result.
    let accepted = DocumentString::from("accepted modal result");
    let request = start_request(&mut session, subject, Some(accepted.clone()));
    let receipt = service(&mut session, &request, &mut registry, &mut calls);
    assert_eq!(receipt.callbacks(), 1);
    assert!(!receipt.default_prevented());
    assert!(receipt.failures().is_empty());
    drop(receipt);
    finish_nested_close(&mut session, request.operation(), &mut registry, &mut calls);
    assert_eq!(calls, 2);
    let closed = session.snapshot();
    let state = closed.node(subject).unwrap().as_dialog().unwrap();
    assert!(!state.is_modal());
    assert_eq!(
        closed.node(subject).unwrap().attribute(&"open".into()),
        None
    );
    assert!(!state.has_close_watcher());
    assert_eq!(state.return_value(), &accepted);
    assert_surviving_reset(&session, subject, None);
    layer(&closed, top.document(), subject, true);
    layer(&shown, top.document(), subject, false);
    layer(&canceled, top.document(), subject, false);
    assert!(shown.node(subject).unwrap().as_dialog().unwrap().is_modal());
    assert_eq!(
        canceled
            .node(subject)
            .unwrap()
            .as_dialog()
            .unwrap()
            .return_value(),
        &seed
    );
    let history = closed
        .document(top.document())
        .unwrap()
        .autofocus()
        .unwrap();
    assert!(history.candidates().is_empty());
    assert!(history.processed());
    deliver_closed_tasks(&mut session, top, subject);
    teardown(session, top);
}

#[test]
fn reentrant_modal_request_literal_force_reset_closes_auto_but_preserves_none_with_latest_fields() {
    for closedby in [None, Some("auto"), Some("none")] {
        let (mut session, top, _, subject) = fixture_with_profile(true);
        if let Some(value) = closedby {
            drop(complete(apply(
                &mut session,
                DomSourceEdit::set_attribute(subject, "closedby", value),
            )));
        }
        let seed = DocumentString::from("seed before reentry");
        seed_return_value(&mut session, subject, &seed);
        let shown = shown_and_initial_toggle_delivered(&mut session, top, subject);
        let original = shown
            .node(subject)
            .unwrap()
            .as_dialog()
            .unwrap()
            .close_watcher();
        let mut context = mutation_context(&session, Mutation::Reenter, false);
        let mut registry = mutation_registry(&mut session, subject);
        let request = start_request(&mut session, subject, Some("outer modal request".into()));
        let receipt = service(&mut session, &request, &mut registry, &mut context);
        assert_eq!(receipt.callbacks(), 1);
        assert!(!receipt.default_prevented());
        assert!(receipt.failures().is_empty(), "{:?}", receipt.failures());
        drop(receipt);
        assert_eq!(context.calls, ["first"]);
        assert_eq!(context.nested.len(), 1);
        assert_ne!(context.nested[0].operation(), request.operation());
        // The same original watcher is already running Cancel, so the genuine
        // inner request returns without another Cancel but leaves its latest
        // request fields and literal force=false. Modal Auto computes to
        // CloseRequest; explicit None remains disabled on the fresh outer read.
        let inner = session.snapshot();
        let state = inner.node(subject).unwrap().as_dialog().unwrap();
        assert!(state.is_modal());
        assert!(!state.force_close_watcher_enabled());
        assert_eq!(state.request_close_return_value(), Some(&context.latest));
        assert_eq!(state.request_close_source(), None);
        assert_eq!(state.close_watcher(), original);
        assert_eq!(state.return_value(), &seed);
        if closedby == Some("none") {
            drop(complete(resume(&mut session, request.operation()).unwrap()));
            assert_surviving_reset(&session, subject, Some(&context.latest));
            let kept = session.snapshot();
            let state = kept.node(subject).unwrap().as_dialog().unwrap();
            assert!(state.is_modal());
            assert_eq!(state.close_watcher(), original);
            assert_eq!(state.return_value(), &seed);
            assert_eq!(
                kept.node(subject).unwrap().attribute(&"open".into()),
                Some(&"".into())
            );
            layer(&kept, top.document(), subject, false);
            assert_eq!(
                session
                    .element_task_execution()
                    .pending_total_tasks()
                    .unwrap(),
                0
            );
            // A later independent standard request forces the same watcher
            // enabled and actually dispatches Cancel, proving guard reset.
            let mut fresh = dispatcher::<()>();
            let cleanup = start_request(
                &mut session,
                subject,
                Some("later independent request".into()),
            );
            let receipt = service(&mut session, &cleanup, &mut fresh, &mut ());
            assert_eq!(receipt.callbacks(), 0);
            assert!(!receipt.default_prevented());
            assert!(receipt.failures().is_empty());
            drop(receipt);
            finish_nested_close(&mut session, cleanup.operation(), &mut fresh, &mut ());
            assert!(kept.node(subject).unwrap().as_dialog().unwrap().is_modal());
            assert_eq!(
                kept.node(subject)
                    .unwrap()
                    .as_dialog()
                    .unwrap()
                    .request_close_return_value(),
                Some(&context.latest)
            );
        } else {
            finish_nested_close(
                &mut session,
                request.operation(),
                &mut registry,
                &mut context,
            );
            assert_surviving_reset(&session, subject, None);
            let closed = session.snapshot();
            let state = closed.node(subject).unwrap().as_dialog().unwrap();
            assert!(!state.is_modal());
            assert!(!state.has_close_watcher());
            assert_eq!(state.return_value(), &context.latest);
            assert_eq!(
                closed.node(subject).unwrap().attribute(&"open".into()),
                None
            );
        }
        assert_eq!(
            context.calls,
            ["first"],
            "samewatcher inner request cannot redispatch Cancel"
        );
        assert_eq!(context.nested.len(), 1);
        let closed = session.snapshot();
        layer(&closed, top.document(), subject, true);
        assert_surviving_reset(&session, subject, None);
        layer(&shown, top.document(), subject, false);
        assert!(shown.node(subject).unwrap().as_dialog().unwrap().is_modal());
        deliver_closed_tasks(&mut session, top, subject);
        teardown(session, top);
    }
}
