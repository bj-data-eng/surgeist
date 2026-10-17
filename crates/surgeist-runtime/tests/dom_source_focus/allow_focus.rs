//! Negative allow-focus uses the current Document's retained original Window.
use super::*;
use surgeist_html::HtmlDialogState;

#[test]
fn callback_adoption_into_retired_document_completes_show_after_negative_allow_focus() {
    struct Context {
        calls: usize,
        path: Vec<DomEventTarget>,
        adoption: Option<Result<DomCommit, DomError>>,
    }
    let cancel = AtomicBool::new(false);
    let mut session = session();
    let (host, a, subject) = hosted_dialog(&mut session, true);
    let mut hosted = session.hosted_execution();
    let b = hosted
        .create_top_level(&mut session.dom(), host, &cancel)
        .unwrap();
    assert_eq!(b.host(), host);
    assert_ne!(a.global(), b.global());
    let initial = session.snapshot();
    let html_b = initial.node(b.document().node()).unwrap().children()[0];
    let body_b = initial.node(html_b).unwrap().children()[1];
    let sentinel = {
        let mut batch = DomBatch::new();
        let dialog = batch.create_element_ns(b.document(), Namespace::html(), "dialog");
        batch.insert_before(&dialog, body_b, None);
        session
            .dom()
            .apply(batch, &cancel)
            .unwrap()
            .created(&dialog)
            .unwrap()
            .unwrap()
    };

    // Ordinary open addition establishes a real B-associated watcher without
    // queuing a toggle or running Show/focusing. It retains B's actual Window.
    let opened = session
        .source_execution()
        .apply(
            &mut session.dom(),
            DomSourceEdit::set_attribute(sentinel, "open", ""),
            limits(),
            &mut DomSourceServices {
                execution_scope: None,
                style_policy: &mut |_| None,
                url: None,
            },
            &cancel,
        )
        .unwrap();
    let DomSourceDriveProgress::Complete(done) = opened else {
        panic!("the unrelated B watcher must genuinely be established: {opened:?}");
    };
    assert!(!done.cancellation_requested());
    drop(done);
    let watched = session.snapshot();
    let watcher_b = watched
        .node(sentinel)
        .unwrap()
        .as_dialog()
        .unwrap()
        .close_watcher()
        .unwrap();
    assert_eq!(watcher_b.global(), b.global());
    assert_eq!(watcher_b.subject(), sentinel);
    assert_eq!(
        session
            .dom_manipulation_execution()
            .pending_tasks()
            .unwrap(),
        0
    );
    assert!(session.dom().pending_source_edit().is_none());

    // Retire B while Source is idle, waiting for this exact actual operation.
    // The unrelated watcher keeps the original Window, not its execution Global.
    let queued = hosted
        .destroy_top_level(&mut session.dom(), b.navigable(), &cancel)
        .unwrap();
    let mut last = DomHostedTaskProgress::Idle;
    for _ in 0..32 {
        last = hosted
            .run_next(&mut session.dom(), queued.traversable(), &cancel)
            .unwrap();
        if matches!(last, DomHostedTaskProgress::Complete { operation } if operation == queued.operation())
        {
            break;
        }
    }
    assert!(
        matches!(last, DomHostedTaskProgress::Complete { operation } if operation == queued.operation()),
        "actual B retirement must complete within 32 queue advances: {last:?}"
    );
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
    assert!(hosted.settings(&session.dom(), a.global(), &cancel).is_ok());
    assert!(matches!(
        hosted.settings(&session.dom(), b.global(), &cancel),
        Err(DomHostedRuntimeError::Dom(DomHostedError::StaleIdentity))
    ));
    assert_eq!(
        retired.node(subject).unwrap().owner_document(),
        Some(a.document())
    );
    assert_eq!(retired.relevant_global(subject).unwrap(), Some(a.global()));

    // Dispatch starts in live A. Only the single genuine target callback moves
    // the closed receiver into B; no fresh dispatch is started in retired B.
    let event = start(&mut session, subject, None);
    let mut registry = DomEventDispatcher::new(DomEventRegistryLimits {
        registrations: 4,
        pending_events: 4,
        nested_dispatches: 4,
    })
    .unwrap();
    registry
        .add_listener(
            &session.dom(),
            subject,
            DomEventType::BeforeToggle,
            DomEventCallback::new(move |dom, _, context: &mut Context, event| {
                context.calls += 1;
                context.path = event.composed_path().collect();
                let mut batch = DomBatch::new();
                batch.insert_before(subject, body_b, None);
                context.adoption = Some(dom.apply(batch, &AtomicBool::new(false)));
                Ok(())
            }),
            DomEventListenerOptions {
                capture: false,
                once: false,
            },
        )
        .unwrap();
    let mut context = Context {
        calls: 0,
        path: Vec::new(),
        adoption: None,
    };
    let serviced = session
        .source_execution()
        .service_event(
            &mut session.dom(),
            &event,
            &mut registry,
            &mut context,
            DomSourceEventServiceLimits {
                source: limits(),
                dispatch: DomEventDispatchLimits {
                    path_items: 32,
                    snapshot_listeners: 32,
                    failure_records: 32,
                    callbacks: 32,
                    work: 100_000,
                },
            },
            &cancel,
        )
        .unwrap();
    let DomSourceEventServiceProgress::Serviced { receipt } = serviced else {
        panic!("the original live-A beforetoggle must finish its real dispatch: {serviced:?}");
    };
    assert_eq!(receipt.callbacks(), 1);
    assert!(!receipt.default_prevented());
    assert!(receipt.failures().is_empty());
    assert_eq!(context.calls, 1);
    assert_eq!(context.path.first(), Some(&DomEventTarget::Node(subject)));
    assert_eq!(
        context.path.last(),
        Some(&DomEventTarget::Global(a.global()))
    );
    drop(
        context
            .adoption
            .take()
            .expect("the callback performed adoption")
            .expect("actual insertion into inactive B must succeed"),
    );
    drop(receipt);
    let adopted = session.snapshot();
    assert_eq!(
        adopted.node(subject).unwrap().owner_document(),
        Some(b.document())
    );
    assert_eq!(adopted.node(subject).unwrap().parent(), Some(body_b));
    assert_eq!(adopted.relevant_global(subject).unwrap(), Some(a.global()));
    assert_eq!(
        adopted.node(subject).unwrap().attribute(&"open".into()),
        None
    );

    let progress = resume(&mut session, event.operation(), None).unwrap();
    let after = session.snapshot();
    assert_eq!(context.calls, 1);
    assert_eq!(
        after.node(subject).unwrap().owner_document(),
        Some(b.document())
    );
    assert_eq!(after.relevant_global(subject).unwrap(), Some(a.global()));
    // Activity/settings queries require a completed Source. The captured
    // environment remains directly observable while this Show tail is pending.
    // Actual null BC alone takes the feature-permission early denial.
    let current_environment = after
        .document(b.document())
        .unwrap()
        .metadata()
        .document_environment();
    assert!(current_environment.has_ever_had_navigable());
    assert_eq!(
        current_environment.browsing_context(),
        surgeist_html::HtmlBrowsingContextPresence::Absent
    );
    assert_eq!(
        after
            .node(sentinel)
            .unwrap()
            .as_dialog()
            .unwrap()
            .close_watcher(),
        Some(watcher_b)
    );
    let receiver = after.node(subject).unwrap().as_dialog().unwrap();
    assert!(!receiver.has_close_watcher());
    assert_eq!(
        after.node(subject).unwrap().attribute(&"open".into()),
        Some(&"".into())
    );
    assert_eq!(
        after
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
    assert_eq!(
        session
            .dom_manipulation_execution()
            .pending_tasks()
            .unwrap(),
        1
    );
    let task = session
        .dom_manipulation_execution()
        .pending_task(0)
        .unwrap()
        .unwrap();
    let toggle = task.dialog().unwrap();
    assert_eq!(toggle.id().target(), subject);
    assert_eq!(toggle.old_state(), HtmlDialogState::Closed);
    assert_eq!(toggle.new_state(), HtmlDialogState::Open);
    assert_eq!(toggle.source(), None);
    assert_eq!(toggle.queue_association().unwrap().global(), a.global());
    assert_eq!(toggle.queue_association().unwrap().document(), a.document());
    assert_eq!(receiver.toggle_task_tracker().unwrap().task(), toggle.id());

    // HTML877d46762–66/60636–41: focusing reads THEN-current Document B.
    // B denies the feature (24292–99), and its actual retained native Window
    // has never been activated (59232–38/59261–66). Negative allow-focus
    // returns before focus-delegate selection or any focusing side effect.
    let DomSourceDriveProgress::Complete(done) = progress else {
        panic!("negative allow-focus for current inactive B must complete Show: {progress:?}");
    };
    assert_eq!(done.operation(), event.operation());
    assert!(!done.cancellation_requested());
    drop(done);
    assert!(session.dom().pending_source_edit().is_none());
    assert_eq!(after.incomplete_source_operation(), None);
    assert!(
        !after
            .document(b.document())
            .unwrap()
            .is_fully_active()
            .unwrap()
    );
    assert!(hosted.settings(&session.dom(), a.global(), &cancel).is_ok());
    assert!(matches!(
        hosted.settings(&session.dom(), b.global(), &cancel),
        Err(DomHostedRuntimeError::Dom(DomHostedError::StaleIdentity))
    ));

    // Cleanup uses the same retained Window after successful Show. It does
    // not pump the A-associated toggle into retired B or revive B's Global.
    let cleanup = session
        .source_execution()
        .apply(
            &mut session.dom(),
            DomSourceEdit::remove_attribute(sentinel, "open"),
            limits(),
            &mut DomSourceServices {
                execution_scope: None,
                style_policy: &mut |_| None,
                url: None,
            },
            &cancel,
        )
        .unwrap();
    let DomSourceDriveProgress::Complete(done) = cleanup else {
        panic!("the unrelated watcher must be destroyed through real cleanup: {cleanup:?}");
    };
    assert!(!done.cancellation_requested());
    drop(done);
    let cleaned = session.snapshot();
    assert!(
        !cleaned
            .node(sentinel)
            .unwrap()
            .as_dialog()
            .unwrap()
            .has_close_watcher()
    );
    assert_eq!(
        cleaned.node(sentinel).unwrap().attribute(&"open".into()),
        None
    );
    assert_eq!(
        cleaned.node(subject).unwrap().attribute(&"open".into()),
        Some(&"".into())
    );
    assert_eq!(
        session
            .dom_manipulation_execution()
            .pending_tasks()
            .unwrap(),
        1
    );
    assert_eq!(
        session
            .dom_manipulation_execution()
            .pending_task(0)
            .unwrap(),
        Some(task)
    );
    assert_eq!(context.calls, 1);
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
        retired
            .node(sentinel)
            .unwrap()
            .as_dialog()
            .unwrap()
            .close_watcher(),
        Some(watcher_b)
    );
    assert_eq!(
        retired.node(subject).unwrap().owner_document(),
        Some(a.document())
    );
    assert_eq!(
        retired.node(subject).unwrap().attribute(&"open".into()),
        None
    );

    // Move the already open receiver through the original Source hooks before
    // delivering its original task; retired B is never a dispatch destination.
    assert!(session.dom().pending_source_edit().is_none());
    let before_return = session.snapshot();
    let original_task = session
        .dom_manipulation_execution()
        .pending_task(0)
        .unwrap()
        .unwrap();
    assert_eq!(original_task, task);
    let original_tracker = before_return
        .node(subject)
        .unwrap()
        .as_dialog()
        .unwrap()
        .toggle_task_tracker()
        .unwrap();
    assert_eq!(original_tracker.task(), original_task.id());
    let html_a = before_return.node(a.document().node()).unwrap().children()[0];
    let body_a = before_return.node(html_a).unwrap().children()[1];
    let moved = session
        .source_execution()
        .apply(
            &mut session.dom(),
            DomSourceEdit::insert_before(subject, body_a, None),
            limits(),
            &mut DomSourceServices {
                execution_scope: None,
                style_policy: &mut |_| None,
                url: None,
            },
            &cancel,
        )
        .unwrap();
    let DomSourceDriveProgress::Complete(done) = moved else {
        panic!(
            "return to live A must finish ordinary open removal/adoption/insertion hooks: {moved:?}"
        );
    };
    assert!(!done.cancellation_requested());
    drop(done);
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
    let returned_dialog = returned.node(subject).unwrap().as_dialog().unwrap();
    let watcher_a = returned_dialog
        .close_watcher()
        .expect("active insertion establishes its real watcher");
    assert_eq!(watcher_a.global(), a.global());
    assert_eq!(watcher_a.subject(), subject);
    assert_eq!(
        returned_dialog.toggle_task_tracker(),
        Some(original_tracker)
    );
    assert_eq!(
        session
            .dom_manipulation_execution()
            .pending_tasks()
            .unwrap(),
        1
    );
    assert_eq!(
        session
            .dom_manipulation_execution()
            .pending_task(0)
            .unwrap(),
        Some(original_task)
    );
    assert_eq!(
        original_task.dialog().unwrap().old_state(),
        HtmlDialogState::Closed
    );
    assert_eq!(
        original_task.dialog().unwrap().new_state(),
        HtmlDialogState::Open
    );
    assert_eq!(original_task.dialog().unwrap().source(), None);
    assert_eq!(
        original_task.queue_association().unwrap().global(),
        a.global()
    );
    assert_eq!(
        original_task.queue_association().unwrap().document(),
        a.document()
    );
    assert!(session.dom().pending_source_edit().is_none());

    struct Delivery {
        owner: DomManipulationRuntime,
        calls: usize,
        target: Option<NodeId>,
        current: Option<DomEventTarget>,
        payload: Option<DomToggleEvent>,
        path: Vec<DomEventTarget>,
        bubbles: Option<bool>,
        edit: Option<Result<DomCommit, DomError>>,
    }
    impl DomManipulationEventContext for Delivery {
        fn dom_manipulation(&mut self) -> &mut DomManipulationRuntime {
            &mut self.owner
        }
    }
    let mut delivery = Delivery {
        owner: session.dom_manipulation_execution(),
        calls: 0,
        target: None,
        current: None,
        payload: None,
        path: Vec::new(),
        bubbles: None,
        edit: None,
    };
    let mut registry = DomEventDispatcher::new(DomEventRegistryLimits {
        registrations: 4,
        pending_events: 4,
        nested_dispatches: 4,
    })
    .unwrap();
    registry
        .add_listener(
            &session.dom(),
            subject,
            DomEventType::Toggle,
            DomEventCallback::new(move |dom, _, delivery: &mut Delivery, event| {
                delivery.calls += 1;
                delivery.target = event.target();
                delivery.current = event.current_target();
                delivery.payload = event.toggle_event().copied();
                delivery.path = event.composed_path().collect();
                delivery.bubbles = Some(event.bubbles());
                let mut batch = DomBatch::new();
                batch.set_attribute(body_a, "data-toggle-delivery", "kept");
                delivery.edit = Some(dom.apply(batch, &AtomicBool::new(false)));
                Ok(())
            }),
            DomEventListenerOptions {
                capture: false,
                once: false,
            },
        )
        .unwrap();
    let run_limits = DomManipulationRunLimits {
        dispatch: DomEventDispatchLimits {
            path_items: 32,
            snapshot_listeners: 32,
            failure_records: 32,
            callbacks: 32,
            work: 100_000,
        },
        source: limits(),
    };
    let delivered = DomManipulationRuntime::run_next(
        &mut session.dom(),
        &mut registry,
        &mut delivery,
        run_limits,
        &cancel,
    )
    .unwrap();
    let DomManipulationProgress::Complete {
        task: delivered_task,
        receipt,
        displaced_owner,
    } = delivered
    else {
        panic!(
            "the original Dialog toggle must really dispatch and clear its tracker: {delivered:?}"
        );
    };
    assert_eq!(delivered_task, original_task);
    assert!(displaced_owner.is_none());
    assert_eq!(receipt.original_target(), subject);
    assert_eq!(receipt.event_type(), DomEventType::Toggle);
    assert_eq!(receipt.callbacks(), 1);
    assert!(receipt.failures().is_empty());
    assert!(!receipt.default_prevented());
    assert_eq!(delivery.calls, 1);
    assert_eq!(delivery.target, Some(subject));
    assert_eq!(delivery.current, Some(DomEventTarget::Node(subject)));
    let payload = delivery.payload.unwrap();
    assert_eq!(payload.old_state(), "closed");
    assert_eq!(payload.new_state(), "open");
    assert_eq!(payload.source(), None);
    assert_eq!(delivery.bubbles, Some(false));
    assert_eq!(
        delivery.path,
        [
            DomEventTarget::Node(subject),
            DomEventTarget::Node(body_a),
            DomEventTarget::Node(html_a),
            DomEventTarget::Node(a.document().node()),
            DomEventTarget::Global(a.global()),
        ]
    );
    drop(
        delivery
            .edit
            .take()
            .expect("Toggle callback performed its unrelated edit")
            .expect("ordinary callback edit succeeds"),
    );
    let completed = session.snapshot();
    assert_eq!(
        completed
            .node(body_a)
            .unwrap()
            .attribute(&"data-toggle-delivery".into()),
        Some(&"kept".into())
    );
    assert_eq!(
        completed
            .node(subject)
            .unwrap()
            .as_dialog()
            .unwrap()
            .toggle_task_tracker(),
        None
    );
    assert_eq!(
        completed
            .node(subject)
            .unwrap()
            .as_dialog()
            .unwrap()
            .close_watcher(),
        Some(watcher_a)
    );
    assert_eq!(delivery.owner.pending_tasks().unwrap(), 0);
    assert!(session.dom().pending_source_edit().is_none());
    assert_eq!(completed.incomplete_source_operation(), None);
    assert_eq!(
        before_return.node(subject).unwrap().owner_document(),
        Some(b.document())
    );
    assert_eq!(
        before_return
            .node(subject)
            .unwrap()
            .as_dialog()
            .unwrap()
            .toggle_task_tracker(),
        Some(original_tracker)
    );
    assert_eq!(
        returned
            .node(subject)
            .unwrap()
            .as_dialog()
            .unwrap()
            .toggle_task_tracker(),
        Some(original_tracker)
    );
    assert_eq!(
        returned
            .node(body_a)
            .unwrap()
            .attribute(&"data-toggle-delivery".into()),
        None
    );
    assert_eq!(
        original_task.dialog().unwrap().old_state(),
        HtmlDialogState::Closed
    );
    assert_eq!(
        original_task.dialog().unwrap().new_state(),
        HtmlDialogState::Open
    );
    assert_eq!(
        original_task.queue_association().unwrap().global(),
        a.global()
    );
    assert_eq!(
        original_task.queue_association().unwrap().document(),
        a.document()
    );
    assert!(matches!(
        DomManipulationRuntime::run_next(
            &mut session.dom(),
            &mut registry,
            &mut delivery,
            run_limits,
            &cancel,
        )
        .unwrap(),
        DomManipulationProgress::Empty
    ));
    assert_eq!(delivery.calls, 1);
    assert_eq!(session.snapshot().stamp(), completed.stamp());
}
