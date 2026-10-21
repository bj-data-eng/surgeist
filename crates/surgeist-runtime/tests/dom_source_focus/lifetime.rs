//! Public lifetime/composition evidence for the sealed complete-Host restriction.
use super::*;

fn body(snapshot: &DomSnapshot, document: DocumentId) -> NodeId {
    let html = snapshot.node(document.node()).unwrap().children()[0];
    snapshot.node(html).unwrap().children()[1]
}

fn add_dialog(session: &mut DomSession, document: DocumentId) -> NodeId {
    let parent = body(&session.snapshot(), document);
    let mut batch = DomBatch::new();
    let dialog = batch.create_element_ns(document, Namespace::html(), "dialog");
    batch.insert_before(&dialog, parent, None);
    session
        .dom()
        .apply(batch, &AtomicBool::new(false))
        .unwrap()
        .created(&dialog)
        .unwrap()
        .unwrap()
}

#[test]
fn retired_original_global_preserves_host_scope_but_refuses_new_toggle_queue() {
    let cancel = AtomicBool::new(false);
    let mut session = session();
    let (host, original, subject) = hosted_dialog(&mut session, true);
    let scope = session.source_execution_scope(host, &cancel).unwrap();
    let (_, destination, _) = hosted_dialog(&mut session, false);
    let destination_body = body(&session.snapshot(), destination.document());
    let mut batch = DomBatch::new();
    batch.insert_before(subject, destination_body, None);
    session.dom().apply(batch, &cancel).unwrap();
    let adopted = session.snapshot();
    assert_eq!(
        adopted.node(subject).unwrap().owner_document(),
        Some(destination.document())
    );
    assert_eq!(
        adopted.relevant_global(subject).unwrap(),
        Some(original.global())
    );

    // Teardown runs while Source is genuinely idle. It must not be pumped
    // behind an unfinished Show or through an event callback's entered loop.
    let mut hosted = session.hosted_execution();
    let queued = hosted
        .destroy_top_level(&mut session.dom(), original.navigable(), &cancel)
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
        "actual original-Global destruction must complete within 32 steps: {last:?}"
    );
    assert!(matches!(
        hosted.settings(&session.dom(), original.global(), &cancel),
        Err(DomHostedRuntimeError::Dom(DomHostedError::StaleIdentity))
    ));
    // The original Host remains live; the receiver's historical Global is not
    // an execution Global and cannot be silently rebound to the destination.
    session.source_execution_scope(host, &cancel).unwrap();
    let before = session.snapshot();
    assert!(
        !before
            .document(original.document())
            .unwrap()
            .is_fully_active()
            .unwrap()
    );
    assert!(
        before
            .document(destination.document())
            .unwrap()
            .is_fully_active()
            .unwrap()
    );
    assert_eq!(
        before.node(subject).unwrap().parent(),
        Some(destination_body)
    );
    assert_eq!(
        before.relevant_global(subject).unwrap(),
        Some(original.global())
    );
    let event = start(&mut session, subject, Some(&scope));
    let receipt = dispatch(&mut session, &event);
    assert_eq!(receipt.original_target(), subject);
    assert_eq!(receipt.event_type(), DomEventType::BeforeToggle);
    assert_eq!(receipt.callbacks(), 0);
    assert!(!receipt.default_prevented());
    assert!(receipt.failures().is_empty());
    drop(receipt);
    assert!(matches!(
        resume(&mut session, event.operation(), Some(&scope)),
        Err(surgeist_runtime::DomSourceDriveError::Source(
            DomSourceRuntimeError::ElementTasks(DomElementTaskRuntimeError::Dom(
                DomError::HostedSourceUnavailable
            ))
        ))
    ));
    let after = session.snapshot();
    assert_eq!(after.stamp(), before.stamp());
    assert_eq!(
        after.node(subject).unwrap().owner_document(),
        Some(destination.document())
    );
    assert_eq!(
        after.relevant_global(subject).unwrap(),
        Some(original.global())
    );
    assert_eq!(after.node(subject).unwrap().attribute(&"open".into()), None);
    assert!(
        !after
            .node(subject)
            .unwrap()
            .as_dialog()
            .unwrap()
            .has_toggle_task_tracker()
    );
    assert_eq!(
        session
            .element_task_execution()
            .pending_tasks(surgeist_dom::DomElementTaskSourceKind::DomManipulation)
            .unwrap(),
        0
    );
    assert_eq!(
        session.dom().pending_source_edit().unwrap().operation(),
        event.operation()
    );
    // QueueToggle's live-original-Global guard is reached before Remember.
    // This case deliberately provides no remembered-focus result after retirement.
}

fn additional_headless_context_remembers_none(in_frame_child: bool) {
    let cancel = AtomicBool::new(false);
    let mut session = session();
    let (host, first, _) = hosted_dialog(&mut session, true);
    let scope = session.source_execution_scope(host, &cancel).unwrap();
    let second = session
        .hosted_execution()
        .create_top_level(&mut session.dom(), host, &cancel)
        .unwrap();
    assert_ne!(second.global(), first.global());
    assert_eq!(second.host(), host);
    let document = if in_frame_child {
        let (closed, frame) = {
            let mut batch = DomBatch::new();
            let closed = batch.create_html_document(second.document(), None);
            let frame = batch.create_element_ns(&closed, Namespace::html(), "frame");
            let done = session.dom().apply(batch, &cancel).unwrap();
            (
                done.document(&closed).unwrap(),
                done.created(&frame).unwrap().unwrap(),
            )
        };
        let parent = body(&session.snapshot(), second.document());
        let before = session.snapshot();
        assert_eq!(before.node(frame).unwrap().owner_document(), Some(closed));
        assert_eq!(
            before
                .node(frame)
                .unwrap()
                .as_element()
                .unwrap()
                .child_navigable(),
            Some(None)
        );
        let progress = session
            .source_execution()
            .apply(
                &mut session.dom(),
                DomSourceEdit::insert_before(frame, parent, None),
                limits(),
                &mut DomSourceServices {
                    meta: None,
                    execution_scope: Some(&scope),
                    style_policy: &mut |_| None,
                    url: None,
                },
                &cancel,
            )
            .unwrap();
        let DomSourceDriveProgress::Complete(done) = progress else {
            panic!("actual absent-src Frame creation must complete its insertion: {progress:?}");
        };
        assert!(!done.cancellation_requested());
        drop(done);
        let snapshot = session.snapshot();
        let child = snapshot
            .node(frame)
            .unwrap()
            .as_element()
            .unwrap()
            .child_navigable()
            .unwrap()
            .unwrap();
        let navigable = snapshot.hosted_navigable(child).unwrap();
        assert_eq!(navigable.parent(), Some(second.navigable()));
        assert_eq!(navigable.container(), Some(frame));
        assert_eq!(navigable.host(), host);
        let document = navigable.active_document().unwrap();
        assert_ne!(document, second.document());
        assert_eq!(
            snapshot.node(frame).unwrap().owner_document(),
            Some(second.document())
        );
        assert_eq!(
            before
                .node(frame)
                .unwrap()
                .as_element()
                .unwrap()
                .child_navigable(),
            Some(None)
        );
        document
    } else {
        second.document()
    };
    let subject = add_dialog(&mut session, document);
    let before = session.snapshot();
    assert!(
        before
            .document(document)
            .unwrap()
            .is_fully_active()
            .unwrap()
    );
    let global = before.relevant_global(subject).unwrap().unwrap();
    assert_ne!(global, first.global());
    let event = start(&mut session, subject, None);
    let receipt = dispatch(&mut session, &event);
    assert_eq!(receipt.callbacks(), 0);
    assert!(!receipt.default_prevented());
    assert!(receipt.failures().is_empty());
    let next = pending_focusing(
        resume(&mut session, event.operation(), None).unwrap(),
        HtmlDialogFocusingAction::RunFocusingSteps {
            control: surgeist_html::HtmlDialogFocusControl::Subject,
        },
    );
    assert_eq!(next.operation(), event.operation());
    assert_eq!(next.subject(), subject);
    let after = session.snapshot();
    let element = after.node(subject).unwrap().as_element().unwrap();
    assert_eq!(element.attribute(&"open".into()), Some(&"".into()));
    assert_eq!(
        element
            .common_state()
            .html()
            .unwrap()
            .previously_focused_element(),
        None
    );
    assert_eq!(
        after
            .node(subject)
            .unwrap()
            .as_dialog()
            .unwrap()
            .close_watcher()
            .unwrap()
            .global(),
        global
    );
    let task = session
        .element_task_execution()
        .pending_task(surgeist_dom::DomElementTaskSourceKind::DomManipulation, 0)
        .unwrap()
        .unwrap()
        .dialog_toggle()
        .unwrap();
    assert_eq!(task.queue_association().unwrap().global(), global);
    assert_eq!(task.queue_association().unwrap().document(), document);
    assert_eq!(
        before.node(subject).unwrap().attribute(&"open".into()),
        None
    );
    // Both future top-levels and child contexts inherit the same enforced Host
    // restriction; no new absence callback or Window focus default is installed.
}

#[test]
fn additional_top_level_under_original_headless_host_remembers_authentic_none() {
    additional_headless_context_remembers_none(false);
}

#[test]
fn actual_frame_child_under_headless_host_remembers_authentic_none() {
    additional_headless_context_remembers_none(true);
}

#[test]
fn matching_hosted_scope_survives_conflicting_resume_without_replacement() {
    let cancel = AtomicBool::new(false);
    let mut session = session();
    let (host, top, subject) = hosted_dialog(&mut session, true);
    let scope = session.source_execution_scope(host, &cancel).unwrap();
    let other = session
        .hosted_execution()
        .create_host(&mut session.dom(), &cancel)
        .unwrap();
    let conflict = session.source_execution_scope(other, &cancel).unwrap();
    let event = start(&mut session, subject, Some(&scope));
    let receipt = dispatch(&mut session, &event);
    assert_eq!(receipt.callbacks(), 0);
    assert!(receipt.failures().is_empty());
    let before = session.snapshot();
    assert!(matches!(
        resume(&mut session, event.operation(), Some(&conflict)),
        Err(surgeist_runtime::DomSourceDriveError::Source(
            DomSourceRuntimeError::Hosted(DomHostedRuntimeError::Dom(DomHostedError::Dom(
                DomError::InvalidSourceOperation
            )))
        ))
    ));
    assert_eq!(session.snapshot().stamp(), before.stamp());
    assert_eq!(
        session.dom().pending_source_edit().unwrap().operation(),
        event.operation()
    );
    assert_eq!(
        session
            .element_task_execution()
            .pending_tasks(surgeist_dom::DomElementTaskSourceKind::DomManipulation)
            .unwrap(),
        0
    );
    pending_focusing(
        resume(&mut session, event.operation(), Some(&scope)).unwrap(),
        HtmlDialogFocusingAction::RunFocusingSteps {
            control: surgeist_html::HtmlDialogFocusControl::Subject,
        },
    );
    let accepted = session.snapshot();
    let task = session
        .element_task_execution()
        .pending_task(surgeist_dom::DomElementTaskSourceKind::DomManipulation, 0)
        .unwrap()
        .unwrap();
    assert_eq!(task.queue_association().unwrap().global(), top.global());
    assert_eq!(task.queue_association().unwrap().document(), top.document());
    assert!(resume(&mut session, event.operation(), Some(&conflict)).is_err());
    pending_focusing(
        resume(&mut session, event.operation(), None).unwrap(),
        HtmlDialogFocusingAction::RunFocusingSteps {
            control: surgeist_html::HtmlDialogFocusControl::Subject,
        },
    );
    assert_eq!(session.snapshot().stamp(), accepted.stamp());
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
}

#[test]
fn canceled_resume_preserves_original_headless_scope_for_uncanceled_retry() {
    let cancel = AtomicBool::new(false);
    let mut session = session();
    let (host, _, subject) = hosted_dialog(&mut session, true);
    let scope = session.source_execution_scope(host, &cancel).unwrap();
    let event = start(&mut session, subject, Some(&scope));
    let receipt = dispatch(&mut session, &event);
    assert_eq!(receipt.callbacks(), 0);
    assert!(!receipt.default_prevented());
    assert!(receipt.failures().is_empty());
    let before = session.snapshot();
    let refused = session.source_execution().resume(
        &mut session.dom(),
        event.operation(),
        limits(),
        &mut DomSourceServices {
            meta: None,
            execution_scope: Some(&scope),
            style_policy: &mut |_| None,
            url: None,
        },
        &AtomicBool::new(true),
    );
    assert!(matches!(
        refused,
        Err(surgeist_runtime::DomSourceDriveError::Source(
            DomSourceRuntimeError::Dom(DomError::Cancelled)
        ))
    ));
    assert_eq!(session.snapshot().stamp(), before.stamp());
    assert_eq!(
        session.dom().pending_source_edit().unwrap().operation(),
        event.operation()
    );
    assert_eq!(
        session
            .element_task_execution()
            .pending_tasks(surgeist_dom::DomElementTaskSourceKind::DomManipulation)
            .unwrap(),
        0
    );
    pending_focusing(
        resume(&mut session, event.operation(), None).unwrap(),
        HtmlDialogFocusingAction::RunFocusingSteps {
            control: surgeist_html::HtmlDialogFocusControl::Subject,
        },
    );
    let after = session.snapshot();
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
            .element_task_execution()
            .pending_tasks(surgeist_dom::DomElementTaskSourceKind::DomManipulation)
            .unwrap(),
        1
    );
}
