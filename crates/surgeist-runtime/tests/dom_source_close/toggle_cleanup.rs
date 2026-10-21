//! The cleanup result belongs to the actual accepted Toggle, not its current tree path.
use super::*;

#[derive(Clone, Copy, Debug)]
enum Change {
    Control,
    Detach,
    Adopt(NodeId),
    Retire,
}
struct Cleanup {
    owner: DomElementTaskRuntime,
    source: DomSourceRuntime,
    change: Change,
    calls: Vec<&'static str>,
    delivered: Vec<EventObservation>,
    historical: Option<DomSnapshot>,
}
impl DomElementTaskEventContext for Cleanup {
    fn element_tasks(&mut self) -> &mut DomElementTaskRuntime {
        &mut self.owner
    }
}

#[test]
fn accepted_toggle_reports_exact_retired_or_surviving_cleanup_after_later_once_listener() {
    for choice in 0..4 {
        let (mut session, top, body, subject) = fixture();
        let second = (choice == 2).then(|| {
            session
                .hosted_execution()
                .create_top_level(&mut session.dom(), top.host(), &cancel())
                .unwrap()
        });
        let change = match second {
            Some(second) => {
                let snapshot = session.snapshot();
                let html = snapshot.node(second.document().node()).unwrap().children()[0];
                Change::Adopt(snapshot.node(html).unwrap().children()[1])
            }
            None if choice == 1 => Change::Detach,
            None if choice == 3 => Change::Retire,
            None => Change::Control,
        };
        // Real Close produces the Toggle and removes its actual watcher before
        // any retirement callback. Its independent Close notification survives.
        open(&mut session, subject);
        let mut method = context(&session);
        let mut before = listeners(&mut session, subject, body);
        let request = start_close(&mut session, subject, None);
        finish_close(&mut session, &request, &mut before, &mut method);
        assert_eq!(method.before.len(), 1);
        let tasks = session.element_task_execution();
        let task = tasks
            .pending_task(DomElementTaskSourceKind::DomManipulation, 0)
            .unwrap()
            .unwrap();
        let toggle = task.dialog_toggle().unwrap();
        assert_eq!(toggle.old_state(), HtmlDialogState::Open);
        assert_eq!(toggle.new_state(), HtmlDialogState::Closed);
        assert_eq!(toggle.source(), None);
        assert_eq!(task.queue_association().unwrap().document(), top.document());
        assert_eq!(task.queue_association().unwrap().global(), top.global());
        let queued = session.snapshot();
        assert_eq!(
            queued
                .node(subject)
                .unwrap()
                .as_dialog()
                .unwrap()
                .toggle_task_tracker()
                .unwrap()
                .task(),
            task.id()
        );
        assert!(
            !queued
                .node(subject)
                .unwrap()
                .as_dialog()
                .unwrap()
                .has_close_watcher()
        );
        let mut context = Cleanup {
            owner: session.element_task_execution(),
            source: session.source_execution(),
            change,
            calls: Vec::new(),
            delivered: Vec::new(),
            historical: None,
        };
        let mut registry = dispatcher();
        registry
            .add_listener(
                &mut session.dom(),
                subject,
                DomEventType::Toggle,
                DomEventCallback::new(move |dom, _, context: &mut Cleanup, event| {
                    context.calls.push("first");
                    context.delivered.push(observe(event));
                    context.historical = Some(dom.snapshot());
                    let edit = match context.change {
                        Change::Detach => Some(DomSourceEdit::detach(subject)),
                        Change::Adopt(parent) => {
                            Some(DomSourceEdit::insert_before(subject, parent, None))
                        }
                        Change::Control | Change::Retire => None,
                    };
                    if let Some(edit) = edit {
                        let result = context
                            .source
                            .apply(
                                dom,
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
                            .unwrap();
                        assert!(matches!(result, DomSourceDriveProgress::Complete(_)));
                    }
                    if matches!(context.change, Change::Retire) {
                        let mut batch = DomBatch::new();
                        batch.detach(subject);
                        batch.destroy(subject);
                        dom.apply(batch, &cancel()).unwrap();
                        assert!(matches!(
                            dom.snapshot().node(subject),
                            Err(DomError::StaleNode)
                        ));
                    }
                    Ok(())
                }),
                DomEventListenerOptions {
                    capture: false,
                    once: true,
                },
            )
            .unwrap();
        let late = registry
            .add_listener(
                &mut session.dom(),
                subject,
                DomEventType::Toggle,
                DomEventCallback::new(move |dom, _, context: &mut Cleanup, event| {
                    context.calls.push("late");
                    context.delivered.push(observe(event));
                    let mut batch = DomBatch::new();
                    batch.set_attribute(body, "data-late-toggle", "ran");
                    dom.apply(batch, &cancel()).unwrap();
                    Ok(())
                }),
                DomEventListenerOptions {
                    capture: false,
                    once: true,
                },
            )
            .unwrap();
        let result = DomElementTaskRuntime::run_next(
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
        } = result
        else {
            panic!("accepted Toggle must finish cleanup: {result:?}")
        };
        assert_eq!(actual, task);
        assert_eq!(
            dialog_toggle_disposition,
            Some(if matches!(change, Change::Retire) {
                DomDialogToggleTaskDisposition::TargetRetiredDuringDispatch
            } else {
                DomDialogToggleTaskDisposition::TrackerCleared
            })
        );
        assert!(displaced_owner.is_none());
        assert_eq!(receipt.callbacks(), 2);
        assert!(receipt.failures().is_empty(), "{:?}", receipt.failures());
        assert_eq!(receipt.original_target(), subject);
        assert!(!receipt.default_prevented());
        assert_eq!(context.calls, ["first", "late"]);
        assert!(!registry.remove_listener(late));
        for event in &context.delivered {
            assert_eq!(event.kind, DomEventType::Toggle);
            assert_eq!(event.target, Some(subject));
            assert_eq!(event.current, Some(DomEventTarget::Node(subject)));
            let payload = event.payload.unwrap();
            assert_eq!(payload.old_state(), "open");
            assert_eq!(payload.new_state(), "closed");
            assert_eq!(payload.source(), None);
            assert!(!event.cancelable);
        }
        let snapshot = session.snapshot();
        assert_eq!(
            snapshot
                .node(body)
                .unwrap()
                .attribute(&"data-late-toggle".into()),
            Some(&"ran".into())
        );
        if matches!(change, Change::Retire) {
            assert!(matches!(snapshot.node(subject), Err(DomError::StaleNode)));
        } else {
            let node = snapshot.node(subject).unwrap();
            assert_eq!(node.as_dialog().unwrap().toggle_task_tracker(), None);
            if let Some(second) = second {
                assert_eq!(node.owner_document(), Some(second.document()));
            }
            if matches!(change, Change::Detach) {
                assert_eq!(node.parent(), None);
            }
        }
        assert_eq!(
            queued
                .node(subject)
                .unwrap()
                .as_dialog()
                .unwrap()
                .toggle_task_tracker()
                .unwrap()
                .task(),
            task.id()
        );
        assert!(context.historical.as_ref().unwrap().node(subject).is_ok());
        assert!(session.dom().pending_source_edit().is_none());
        assert_eq!(snapshot.incomplete_source_operation(), None);
        assert_eq!(tasks.pending_total_tasks().unwrap(), 1);
        // Retired Close is independently skipped; surviving Close delivers with
        // no Toggle disposition. Neither path recreates the already settled row.
        let result = DomElementTaskRuntime::run_next(
            &mut session.dom(),
            DomElementTaskSourceKind::UserInteraction,
            &mut registry,
            &mut context,
            run_limits(),
            &cancel(),
        )
        .unwrap();
        if matches!(change, Change::Retire) {
            assert!(matches!(
                result,
                DomElementTaskProgress::SkippedTargetDestroyed(_)
            ));
        } else {
            let DomElementTaskProgress::Complete {
                dialog_toggle_disposition,
                receipt,
                ..
            } = result
            else {
                panic!("Close receipt: {result:?}")
            };
            assert_eq!(dialog_toggle_disposition, None);
            assert_eq!(receipt.callbacks(), 0);
            assert!(receipt.failures().is_empty());
        }
        assert_eq!(tasks.pending_total_tasks().unwrap(), 0);
    }
}

#[test]
fn toggle_retired_before_dispatch_is_skipped_without_postdispatch_result_or_callbacks() {
    let (mut session, _, body, subject) = fixture();
    open(&mut session, subject);
    let mut method = context(&session);
    let mut before = listeners(&mut session, subject, body);
    let request = start_close(&mut session, subject, None);
    finish_close(&mut session, &request, &mut before, &mut method);
    let tasks = session.element_task_execution();
    let task = tasks
        .pending_task(DomElementTaskSourceKind::DomManipulation, 0)
        .unwrap()
        .unwrap();
    let mut context = Cleanup {
        owner: session.element_task_execution(),
        source: session.source_execution(),
        change: Change::Control,
        calls: Vec::new(),
        delivered: Vec::new(),
        historical: None,
    };
    let mut registry = dispatcher();
    let listener = registry
        .add_listener(
            &mut session.dom(),
            subject,
            DomEventType::Toggle,
            DomEventCallback::new(|_, _, context: &mut Cleanup, _| {
                context.calls.push("unexpected");
                Ok(())
            }),
            DomEventListenerOptions {
                capture: false,
                once: true,
            },
        )
        .unwrap();
    let mut batch = DomBatch::new();
    batch.detach(subject);
    batch.destroy(subject);
    session.dom().apply(batch, &cancel()).unwrap();
    let result = DomElementTaskRuntime::run_next(
        &mut session.dom(),
        DomElementTaskSourceKind::DomManipulation,
        &mut registry,
        &mut context,
        run_limits(),
        &cancel(),
    )
    .unwrap();
    assert!(
        matches!(result, DomElementTaskProgress::SkippedTargetDestroyed(actual) if actual == task)
    );
    assert!(context.calls.is_empty());
    assert!(registry.remove_listener(listener));
    assert!(session.dom().pending_source_edit().is_none());
    let result = DomElementTaskRuntime::run_next(
        &mut session.dom(),
        DomElementTaskSourceKind::UserInteraction,
        &mut registry,
        &mut context,
        run_limits(),
        &cancel(),
    )
    .unwrap();
    assert!(matches!(
        result,
        DomElementTaskProgress::SkippedTargetDestroyed(_)
    ));
    assert_eq!(tasks.pending_total_tasks().unwrap(), 0);
}
