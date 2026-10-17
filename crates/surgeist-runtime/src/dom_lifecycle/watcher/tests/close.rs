//! Close retains its result and published Toggle across real watcher refusal.
use super::*;
use crate::{
    DomEventCallback, DomEventListenerOptions, DomEventType, DomSourceEventServiceLimits,
    DomSourceEventServiceProgress,
};

fn close_at_removal(
    cap: u64,
    attempt_removal: bool,
) -> (
    DomSession,
    NodeId,
    DomCloseWatcherId,
    DomSourceDialogRequest,
    u64,
) {
    let (mut session, _, _, subject) = fixture();
    let cancel = AtomicBool::new(false);
    let original = DocumentString::from("original result");
    apply(
        &mut session,
        DomSourceEdit::set_dialog_return_value(subject, original),
    );
    apply(
        &mut session,
        DomSourceEdit::set_attribute(subject, "open", ""),
    );
    let watcher = session
        .snapshot()
        .node(subject)
        .unwrap()
        .as_dialog()
        .unwrap()
        .close_watcher()
        .unwrap();
    let source = session.source_execution();
    let event = {
        let mut dom = session.dom();
        let mut lease = dom
            .begin_source_edit(
                DomSourceEdit::close_dialog(
                    subject,
                    Some(DocumentString::from_utf16(vec![0xd800, 0, 0xdc00])),
                ),
                work_limit(cap),
                &cancel,
            )
            .unwrap();
        let DomSourceProgress::Request(DomSourceRequest::Event(event)) =
            lease.advance(&cancel).unwrap()
        else {
            panic!("actual Close beforetoggle");
        };
        assert!(!event.cancelable());
        event
    };
    let mut listeners = dispatcher();
    listeners
        .add_listener(
            &session.dom(),
            subject,
            DomEventType::BeforeToggle,
            DomEventCallback::new(|_, _, calls: &mut usize, event| {
                *calls += 1;
                event.prevent_default();
                assert!(!event.default_prevented());
                Ok(())
            }),
            DomEventListenerOptions::default(),
        )
        .unwrap();
    let mut calls = 0;
    let outcome = source
        .service_event(
            &mut session.dom(),
            &event,
            &mut listeners,
            &mut calls,
            DomSourceEventServiceLimits {
                source: work_limit(cap),
                dispatch: event_limits(),
            },
            &cancel,
        )
        .unwrap();
    let DomSourceEventServiceProgress::Serviced { receipt } = outcome else {
        panic!("actual noncancelable beforetoggle must complete");
    };
    assert_eq!(receipt.callbacks(), 1);
    assert!(receipt.failures().is_empty());
    assert!(!receipt.default_prevented());
    assert_eq!(calls, 1);
    drop(receipt);
    let runtime = session.element_task_execution();
    let hosted = session.hosted_execution();
    let (request, spent) = {
        let mut dom = session.dom();
        let mut lease = dom
            .resume_source_edit(event.operation(), work_limit(cap), &cancel)
            .unwrap();
        let toggle = pending(&mut lease);
        assert_eq!(
            toggle.action(),
            HtmlDialogAction::QueueToggle {
                old: surgeist_html::HtmlDialogState::Open,
                new: surgeist_html::HtmlDialogState::Closed,
                source: surgeist_html::HtmlDialogEventSource::OperationSource,
            }
        );
        runtime.queue_dialog(&mut lease, &toggle, &cancel).unwrap();
        let request = pending(&mut lease);
        assert_eq!(request.action(), HtmlDialogAction::DestroyCloseWatcher);
        assert_eq!(request.operation(), event.operation());
        assert_eq!(lease.status().operation(), event.operation());
        let spent = lease.status().spent_work();
        if attempt_removal {
            let before = lease.snapshot();
            let toggle = runtime
                .pending_task(DomElementTaskSourceKind::DomManipulation, 0)
                .unwrap()
                .unwrap();
            let result = hosted.install_dialog_watcher(&mut lease, &request, &cancel);
            match result {
                Err(DomHostedRuntimeError::Dom(DomHostedError::Dom(
                    DomError::SourceWorkLimit { limit },
                ))) => assert_eq!(limit, cap),
                Err(error) => panic!("wrong Close watcher service refusal: {error:?}"),
                Ok(_) => panic!("one-short admitted actual Close watcher removal"),
            }
            assert_eq!(lease.status().spent_work(), cap);
            assert_eq!(lease.status().operation(), request.operation());
            assert_eq!(lease.snapshot().stamp(), before.stamp());
            assert_eq!(
                runtime
                    .pending_task(DomElementTaskSourceKind::DomManipulation, 0)
                    .unwrap(),
                Some(toggle)
            );
        }
        (request, spent)
    };
    assert_eq!(calls, 1);
    (session, subject, watcher, request, spent)
}

#[test]
fn close_result_and_queued_toggle_survive_watcher_work_refusal_then_same_operation_finishes() {
    // Calibrate only the genuine event/QueueToggle/RemoveOpen prefix. No
    // watcher service has run. Leave less than the independently counted27
    // Runtime/preparation steps; actual Source payload admission is additional.
    let scaffold = close_at_removal(source_limits().work(), false).4;
    let cap = scaffold + ONE_MEMBER_REMOVAL_STEPS - 1;
    let (mut session, subject, id, request, spent) = close_at_removal(cap, true);
    assert_eq!(spent, scaffold);
    let cancel = AtomicBool::new(false);
    let hosted = session.hosted_execution();
    let owner = hosted.state().unwrap();
    let tasks = session.element_task_execution();
    let toggle = tasks
        .pending_task(DomElementTaskSourceKind::DomManipulation, 0)
        .unwrap()
        .unwrap();
    assert!(toggle.dialog_toggle().is_some());
    assert_eq!(toggle.id().target(), subject);
    assert_eq!(tasks.pending_total_tasks().unwrap(), 1);
    let before = session.snapshot();
    assert_eq!(
        before.node(subject).unwrap().attribute(&"open".into()),
        None
    );
    let dialog = before.node(subject).unwrap().as_dialog().unwrap();
    assert_eq!(
        dialog.return_value(),
        &DocumentString::from("original result")
    );
    assert_eq!(dialog.close_watcher(), Some(id));
    assert_eq!(dialog.toggle_task_tracker().unwrap().task(), toggle.id());
    let status = session.dom().pending_source_edit().unwrap();
    assert_eq!(status.operation(), request.operation());
    assert_eq!(status.spent_work(), cap);
    assert_eq!(counts(&hosted), (1, 1, 1));
    {
        let state = owner.borrow();
        let watcher = &state.hosts[0].top_levels[0]
            .group
            .as_ref()
            .unwrap()
            .clusters[0]
            .agent
            .windows
            .rows[0]
            .groups[0]
            .members[0];
        let watcher = watcher.borrow();
        assert_eq!(watcher.id, id);
        assert_eq!(watcher.phase, WatcherPhase::Registered);
        assert!(watcher.publication.is_none());
    }
    assert_eq!(
        tasks
            .pending_task(DomElementTaskSourceKind::DomManipulation, 0)
            .unwrap(),
        Some(toggle)
    );
    assert_eq!(
        tasks
            .pending_tasks(DomElementTaskSourceKind::UserInteraction)
            .unwrap(),
        0
    );
    let original_record = {
        let state = owner.borrow();
        Rc::downgrade(
            &state.hosts[0].top_levels[0]
                .group
                .as_ref()
                .unwrap()
                .clusters[0]
                .agent
                .windows
                .rows[0]
                .groups[0]
                .members[0],
        )
    };
    // Reacquire the original owner and raise the same cumulative Source policy;
    // no callback, QueueToggle publication or result input is recreated.
    let replacement = session.hosted_execution();
    {
        let mut dom = session.dom();
        let mut lease = dom
            .resume_source_edit(request.operation(), source_limits(), &cancel)
            .unwrap();
        let location = replacement
            .install_dialog_watcher(&mut lease, &request, &cancel)
            .unwrap()
            .unwrap();
        assert_eq!(location.get(&mut owner.borrow_mut()).id, id);
        replacement
            .accept_watcher(&mut lease, &request, &location, &cancel)
            .unwrap();
        let settlement = replacement
            .prepare_watcher_settlement(&lease, &cancel)
            .unwrap();
        let close = pending(&mut lease);
        assert_eq!(close.action(), HtmlDialogAction::QueueClose);
        assert_eq!(close.operation(), request.operation());
        settlement.finish(&lease.snapshot());
        // Physical membership is already gone; only the original pending
        // HookRemoval custody survives until canonical slot publication.
        assert_eq!(counts(&replacement), (1, 0, 0));
        {
            let state = owner.borrow();
            let window = &state.hosts[0].top_levels[0]
                .group
                .as_ref()
                .unwrap()
                .clusters[0]
                .agent
                .windows
                .rows[0];
            assert_eq!(window.custody.len(), 1);
            let WatcherCustody::HookRemoval(record) = &window.custody[0] else {
                panic!("real Hook removal must retain the original record");
            };
            assert_eq!(record.borrow().id, id);
            assert!(Rc::ptr_eq(record, &location.record));
            assert!(Rc::ptr_eq(record, &original_record.upgrade().unwrap()));
        }
        assert_eq!(location.get(&mut owner.borrow_mut()).id, id);
        assert_eq!(
            location.get(&mut owner.borrow_mut()).phase,
            WatcherPhase::Removing
        );
        assert!(location.get(&mut owner.borrow_mut()).publication.is_none());
        let current = lease.snapshot();
        let dialog = current.node(subject).unwrap().as_dialog().unwrap();
        assert_eq!(dialog.close_watcher(), Some(id));
        assert_eq!(
            dialog.return_value(),
            &DocumentString::from("original result")
        );
        assert_eq!(dialog.toggle_task_tracker().unwrap().task(), toggle.id());
        drop(location);
        assert!(
            original_record.upgrade().is_some(),
            "actual Window custody retains the original without the installer handle"
        );
        tasks.queue_dialog(&mut lease, &close, &cancel).unwrap();
        // Match the next real driver iteration after the queue publication:
        // preadmit settlement, advance the same method, then observe its state.
        let settlement = replacement
            .prepare_watcher_settlement(&lease, &cancel)
            .unwrap();
        assert!(matches!(
            lease.advance(&cancel).unwrap(),
            DomSourceProgress::Complete(done) if done.operation() == request.operation()
        ));
        settlement.finish(&lease.snapshot());
        assert_eq!(counts(&replacement), (1, 0, 0));
        assert!(
            owner.borrow().hosts[0].top_levels[0]
                .group
                .as_ref()
                .unwrap()
                .clusters[0]
                .agent
                .windows
                .rows[0]
                .custody
                .is_empty()
        );
        assert!(
            original_record.upgrade().is_none(),
            "real final publication releases the last original owner"
        );
        let current = lease.snapshot();
        let dialog = current.node(subject).unwrap().as_dialog().unwrap();
        assert_eq!(dialog.close_watcher(), None);
        assert_eq!(
            dialog.return_value(),
            &DocumentString::from_utf16(vec![0xd800, 0, 0xdc00])
        );
    }
    assert!(session.dom().pending_source_edit().is_none());
    assert_eq!(tasks.pending_total_tasks().unwrap(), 2);
    assert_eq!(
        tasks
            .pending_task(DomElementTaskSourceKind::DomManipulation, 0)
            .unwrap(),
        Some(toggle)
    );
    let close = tasks
        .pending_task(DomElementTaskSourceKind::UserInteraction, 0)
        .unwrap()
        .unwrap();
    assert!(close.dialog_close().is_some());
    assert_ne!(close.id(), toggle.id());
    assert_eq!(close.queue_association(), toggle.queue_association());
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
        toggle.id()
    );
    assert_eq!(
        before
            .node(subject)
            .unwrap()
            .as_dialog()
            .unwrap()
            .return_value(),
        &DocumentString::from("original result")
    );
}
