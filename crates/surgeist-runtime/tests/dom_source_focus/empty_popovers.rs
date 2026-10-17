//! Show's empty popover prelude uses the actual current Document before focusing.
use super::*;
use surgeist_html::HtmlDialogState;

fn empty_popover_prelude_waits_for_dialog_focusing(
    session: &mut DomSession,
    subject: NodeId,
    scope: Option<&DomSourceExecutionScope>,
    association: Option<(DomHostedGlobalId, DocumentId)>,
) {
    let before = session.snapshot();
    let document = before.node(subject).unwrap().owner_document().unwrap();
    let event = start(session, subject, scope);
    let payload = event
        .toggle_event()
        .expect("actual Show ToggleEvent payload");
    assert_eq!(payload.old_state(), HtmlDialogState::Closed);
    assert_eq!(payload.new_state(), HtmlDialogState::Open);
    assert_eq!(payload.source(), None);
    let receipt = dispatch(session, &event);
    assert_eq!(receipt.callbacks(), 0);
    assert!(!receipt.default_prevented());
    assert!(receipt.failures().is_empty());

    // HTML877d Show steps 8–11 capture current nodeDocument, find an ancestor
    // with isPopover=false, and hide through a null endpoint with false/true
    // flags before focusing. These actual new Documents have constructor-owned
    // Known empty raw top layers, not unavailable imported lists. Auto and Hint
    // filtering therefore yields no candidates or hide iterations. DOM exposes
    // no public raw-layer getter; headlessness supplies only the focus absence.
    let next = pending_focusing(
        resume(session, event.operation(), scope).unwrap(),
        if association.is_some() {
            HtmlDialogFocusingAction::FindFocusDelegate
        } else {
            HtmlDialogFocusingAction::ReadAllowFocus
        },
    );
    assert_eq!(next.operation(), event.operation());
    assert_eq!(next.subject(), subject);
    let after = session.snapshot();
    assert_ne!(after.stamp(), before.stamp());
    assert_eq!(
        after.node(subject).unwrap().owner_document(),
        Some(document)
    );
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
    assert_eq!(after.incomplete_source_operation(), Some(event.operation()));
    let status = session.dom().pending_source_edit().unwrap();
    assert_eq!(status.operation(), event.operation());
    assert_eq!(status.phase(), DomSourcePhase::DialogService);
    assert_eq!(
        session
            .element_task_execution()
            .pending_tasks(surgeist_dom::DomElementTaskSourceKind::DomManipulation)
            .unwrap(),
        1
    );
    let task = session
        .element_task_execution()
        .pending_task(surgeist_dom::DomElementTaskSourceKind::DomManipulation, 0)
        .unwrap()
        .unwrap();
    let dialog_task = task
        .dialog_toggle()
        .expect("the actual original Show toggle task");
    assert_eq!(dialog_task.id().target(), subject);
    assert_eq!(dialog_task.old_state(), HtmlDialogState::Closed);
    assert_eq!(dialog_task.new_state(), HtmlDialogState::Open);
    assert_eq!(dialog_task.source(), None);
    assert_eq!(
        dialog_task
            .queue_association()
            .map(|queue| (queue.global(), queue.document())),
        association
    );
    let dialog = after.node(subject).unwrap().as_dialog().unwrap();
    let tracker = dialog.toggle_task_tracker().unwrap();
    assert_eq!(tracker.task(), dialog_task.id());
    assert_eq!(tracker.old_state(), HtmlDialogState::Closed);
    match association {
        Some((global, associated_document)) => {
            assert_eq!(associated_document, document);
            assert_eq!(after.relevant_global(subject).unwrap(), Some(global));
            assert_eq!(dialog.close_watcher().unwrap().global(), global);
        }
        None => {
            assert_eq!(after.relevant_global(subject).unwrap(), None);
            assert!(!dialog.has_close_watcher());
        }
    }
    assert_eq!(
        before.node(subject).unwrap().attribute(&"open".into()),
        None
    );
    assert!(
        !before
            .node(subject)
            .unwrap()
            .as_dialog()
            .unwrap()
            .has_toggle_task_tracker()
    );
    assert!(
        !before
            .node(subject)
            .unwrap()
            .as_dialog()
            .unwrap()
            .has_close_watcher()
    );

    let retry = pending_focusing(
        resume(session, event.operation(), None).unwrap(),
        if association.is_some() {
            HtmlDialogFocusingAction::FindFocusDelegate
        } else {
            HtmlDialogFocusingAction::ReadAllowFocus
        },
    );
    assert_eq!(retry.operation(), event.operation());
    assert_eq!(retry.subject(), subject);
    assert_eq!(session.snapshot().stamp(), after.stamp());
    assert_eq!(
        session
            .element_task_execution()
            .pending_tasks(surgeist_dom::DomElementTaskSourceKind::DomManipulation)
            .unwrap(),
        1
    );
    assert_eq!(
        session
            .element_task_execution()
            .pending_task(surgeist_dom::DomElementTaskSourceKind::DomManipulation, 0)
            .unwrap(),
        Some(task)
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
    // Focusing remains unacknowledged. No queued event is pumped through this
    // unfinished synchronous tail, and no focus movement or Show completion is claimed.
}

#[test]
fn native_headless_show_with_known_empty_top_layer_waits_for_dialog_focusing() {
    let mut session = session();
    let subject = native_dialog(&mut session);
    assert_eq!(session.snapshot().relevant_global(subject).unwrap(), None);
    let host = session
        .hosted_execution()
        .create_headless_host(&mut session.dom(), &AtomicBool::new(false))
        .unwrap();
    let scope = session
        .source_execution_scope(host, &AtomicBool::new(false))
        .unwrap();
    empty_popover_prelude_waits_for_dialog_focusing(&mut session, subject, Some(&scope), None);
}

#[test]
fn active_hosted_headless_show_with_known_empty_top_layer_waits_for_dialog_focusing() {
    let mut session = session();
    let (_, top, subject) = hosted_dialog(&mut session, true);
    assert!(
        session
            .snapshot()
            .document(top.document())
            .unwrap()
            .is_fully_active()
            .unwrap()
    );
    assert_eq!(
        session.snapshot().relevant_global(subject).unwrap(),
        Some(top.global())
    );
    empty_popover_prelude_waits_for_dialog_focusing(
        &mut session,
        subject,
        None,
        Some((top.global(), top.document())),
    );
}
