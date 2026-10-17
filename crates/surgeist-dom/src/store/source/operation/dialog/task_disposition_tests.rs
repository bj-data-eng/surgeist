//! Controlled qualified queued-task inputs at the Source cleanup owner. These
//! fixtures do not claim a Runtime queue or listener delivery. Real production
//! task qualification, executor scopes, receipts and canonical destruction run
//! here; Runtime tests independently cover the actual queue and callbacks.
use super::tests::{fixture, limits};
use super::*;
use crate::{
    DomDialogTask, DomDialogTaskInfo, DomDialogToggleTaskDisposition as D, DomElementTaskSource,
};

fn cancel() -> AtomicBool {
    AtomicBool::new(false)
}
fn queued(
    toggle: bool,
) -> (
    Dom,
    DomSourceEventExecutor,
    DomElementTaskSource,
    Option<DomDialogTask>,
    DocumentId,
    NodeId,
) {
    let (mut dom, executor, document, subject, _) = fixture();
    let source = dom.bind_element_task_source(&cancel()).unwrap();
    let id = source.task_id(1, subject).unwrap();
    let info = if toggle {
        let owner = DomBatch::new();
        let mut draft = Draft::new(&dom, owner.owner.clone(), &cancel()).unwrap();
        let key = draft.resolve(subject.into()).unwrap();
        let mut state = draft.dialog_state(key).unwrap().clone();
        state.toggle_tracker = Some(crate::DomDialogToggleTracker {
            task: id,
            old: surgeist_html::HtmlDialogState::Closed,
        });
        draft.element_mut(key).set_dialog_state(state);
        let prepared = draft.finish(&dom, &cancel()).unwrap();
        dom.publish(prepared, &cancel()).unwrap();
        DomDialogTaskInfo::Toggle(crate::DomDialogToggleTaskInfo {
            id,
            old: surgeist_html::HtmlDialogState::Closed,
            new: surgeist_html::HtmlDialogState::Open,
            association: None,
        })
    } else {
        DomDialogTaskInfo::Close(crate::DomDialogCloseTaskInfo {
            id,
            association: None,
        })
    };
    let task = Some(DomDialogTask {
        info,
        binding: Arc::downgrade(&source.binding),
    });
    (dom, executor, source, task, document, subject)
}
fn begin(
    dom: &mut Dom,
    source: &DomElementTaskSource,
    task: &mut Option<DomDialogTask>,
    policy: DomSourceLimits,
) -> DomSourceEventRequest {
    let mut lease = source
        .begin_dialog_task(dom, task, policy, &cancel())
        .unwrap();
    let DomSourceProgress::Request(DomSourceRequest::Event(event)) =
        lease.advance(&cancel()).unwrap()
    else {
        panic!("actual task event observation")
    };
    assert!(task.is_none());
    event
}
fn accept(
    dom: &mut Dom,
    source: &DomElementTaskSource,
    id: crate::DomElementTaskId,
    event: &DomSourceEventRequest,
    executor: &DomSourceEventExecutor,
    permit: crate::DomSourceEventDispatchPermit,
    policy: DomSourceLimits,
) {
    let mut permit = Some(permit);
    let mut publication = Some(
        executor
            .seal_completed(
                dom,
                &mut permit,
                DomSourceEventOutcome::NotCanceled,
                &cancel(),
            )
            .unwrap(),
    );
    assert!(permit.is_none());
    source
        .resume_dialog_task(dom, id, event.operation(), policy, &cancel())
        .unwrap()
        .accept_source_event(event, &mut publication, &cancel())
        .unwrap();
    assert!(publication.is_none());
}
fn complete(progress: DomSourceProgress) -> DomSourceCompletion {
    let DomSourceProgress::Complete(done) = progress else {
        panic!("completed original cleanup")
    };
    done
}

#[test]
fn surviving_toggle_returns_tracker_cleared_only_after_original_segment_publication() {
    let (mut dom, executor, source, mut task, _, subject) = queued(true);
    let id = task.as_ref().unwrap().info().id();
    let before = dom.snapshot();
    let event = begin(&mut dom, &source, &mut task, limits());
    let wrong = source.task_id(2, subject).unwrap();
    assert!(matches!(
        source.resume_dialog_task(&mut dom, wrong, event.operation(), limits(), &cancel()),
        Err(DomError::InvalidSourceOperation)
    ));
    let permit = executor.begin_dispatch(&dom, &event, &cancel()).unwrap();
    drop(executor.enter_dispatch(&dom, &permit, &cancel()).unwrap());
    accept(&mut dom, &source, id, &event, &executor, permit, limits());
    let done = complete(
        source
            .resume_dialog_task(&mut dom, id, event.operation(), limits(), &cancel())
            .unwrap()
            .advance(&cancel())
            .unwrap(),
    );
    assert_eq!(done.operation(), event.operation());
    assert_eq!(done.dialog_toggle_disposition(), Some(D::TrackerCleared));
    assert!(!done.cancellation_requested());
    assert_eq!(done.output(), None);
    assert_eq!(
        done.commit().changes().changes(),
        &[DomChange::ElementState(subject)]
    );
    assert!(
        dom.snapshot()
            .node(subject)
            .unwrap()
            .as_dialog()
            .unwrap()
            .toggle_task_tracker()
            .is_none()
    );
    assert!(
        before
            .node(subject)
            .unwrap()
            .as_dialog()
            .unwrap()
            .toggle_task_tracker()
            .is_some()
    );
    assert!(dom.pending_source_edit().is_none());
}

#[test]
fn started_toggle_exact_retirement_returns_explicit_no_write_disposition_after_cancel_retry() {
    let (mut dom, executor, source, mut task, document, subject) = queued(true);
    let id = task.as_ref().unwrap().info().id();
    let original = dom.snapshot();
    let event = begin(&mut dom, &source, &mut task, limits());
    let permit = executor.begin_dispatch(&dom, &event, &cancel()).unwrap();
    let scope = executor.enter_dispatch(&dom, &permit, &cancel()).unwrap();
    let mut batch = DomBatch::new();
    batch.detach(subject);
    batch.destroy(subject);
    let replacement = batch.create_element_ns(document, crate::Namespace::html(), "dialog");
    let callback = dom.apply(batch, &cancel()).unwrap();
    let replacement = callback.created(&replacement).unwrap().unwrap();
    assert_ne!(replacement, subject);
    let stamp = dom.snapshot().stamp();
    drop(scope);
    accept(&mut dom, &source, id, &event, &executor, permit, limits());
    {
        let mut lease = source
            .resume_dialog_task(&mut dom, id, event.operation(), limits(), &cancel())
            .unwrap();
        assert!(matches!(
            lease.advance(&AtomicBool::new(true)),
            Err(DomError::Cancelled)
        ));
    }
    assert_eq!(
        dom.pending_source_edit().unwrap().operation(),
        event.operation()
    );
    assert_eq!(dom.snapshot().stamp(), stamp);
    let done = complete(
        source
            .resume_dialog_task(&mut dom, id, event.operation(), limits(), &cancel())
            .unwrap()
            .advance(&cancel())
            .unwrap(),
    );
    assert_eq!(
        done.dialog_toggle_disposition(),
        Some(D::TargetRetiredDuringDispatch)
    );
    assert_eq!(done.output(), None);
    assert!(done.commit().changes().changes().is_empty());
    assert_eq!(dom.snapshot().stamp(), stamp);
    assert!(
        dom.snapshot()
            .node(replacement)
            .unwrap()
            .as_dialog()
            .unwrap()
            .toggle_task_tracker()
            .is_none()
    );
    assert!(
        original
            .node(subject)
            .unwrap()
            .as_dialog()
            .unwrap()
            .toggle_task_tracker()
            .is_some()
    );
    assert!(dom.pending_source_edit().is_none());
}

#[test]
fn post_dispatch_output_refusal_retains_cleared_segment_and_original_task_until_retry() {
    let (mut dom, executor, source, mut task, _, subject) = queued(true);
    let id = task.as_ref().unwrap().info().id();
    let before = dom.snapshot();
    let mut tight = limits();
    tight.outputs = 0;
    let event = begin(&mut dom, &source, &mut task, tight);
    let permit = executor.begin_dispatch(&dom, &event, &cancel()).unwrap();
    drop(executor.enter_dispatch(&dom, &permit, &cancel()).unwrap());
    accept(&mut dom, &source, id, &event, &executor, permit, tight);
    {
        let mut lease = source
            .resume_dialog_task(&mut dom, id, event.operation(), tight, &cancel())
            .unwrap();
        assert!(matches!(
            lease.advance(&cancel()),
            Err(DomError::SourceOutputLimit)
        ));
    }
    assert_eq!(dom.snapshot().stamp(), before.stamp());
    assert!(
        dom.snapshot()
            .node(subject)
            .unwrap()
            .as_dialog()
            .unwrap()
            .toggle_task_tracker()
            .is_some()
    );
    let original = dom.source_operation.as_ref().unwrap();
    assert_eq!(original.id, event.operation());
    assert!(matches!(
        original.commands,
        OperationRoot::DialogTask(DomDialogTaskInfo::Toggle(_))
    ));
    assert!(matches!(original.draft, OperationDraft::Segment(_)));
    let mut exact = limits();
    exact.outputs = 1;
    let done = complete(
        source
            .resume_dialog_task(&mut dom, id, event.operation(), exact, &cancel())
            .unwrap()
            .advance(&cancel())
            .unwrap(),
    );
    assert_eq!(done.dialog_toggle_disposition(), Some(D::TrackerCleared));
    assert_eq!(
        done.commit().changes().changes(),
        &[DomChange::ElementState(subject)]
    );
    assert!(dom.pending_source_edit().is_none());
}

#[test]
fn cancellation_before_dispatch_has_no_cleanup_disposition_or_tracker_clear() {
    let (mut dom, _executor, source, mut task, _, subject) = queued(true);
    let id = task.as_ref().unwrap().info().id();
    let before = dom.snapshot();
    let event = begin(&mut dom, &source, &mut task, limits());
    let mut lease = source
        .resume_dialog_task(&mut dom, id, event.operation(), limits(), &cancel())
        .unwrap();
    lease.request_cancel();
    let done = complete(lease.advance(&cancel()).unwrap());
    drop(lease);
    assert!(done.cancellation_requested());
    assert_eq!(done.dialog_toggle_disposition(), None);
    assert_eq!(dom.snapshot().stamp(), before.stamp());
    assert!(
        dom.snapshot()
            .node(subject)
            .unwrap()
            .as_dialog()
            .unwrap()
            .toggle_task_tracker()
            .is_some()
    );
    assert!(dom.pending_source_edit().is_none());
}

#[test]
fn cancellation_after_dispatch_preserves_actual_tracker_cleanup_disposition() {
    let (mut dom, executor, source, mut task, _, _) = queued(true);
    let id = task.as_ref().unwrap().info().id();
    let event = begin(&mut dom, &source, &mut task, limits());
    let permit = executor.begin_dispatch(&dom, &event, &cancel()).unwrap();
    drop(executor.enter_dispatch(&dom, &permit, &cancel()).unwrap());
    accept(&mut dom, &source, id, &event, &executor, permit, limits());
    let mut lease = source
        .resume_dialog_task(&mut dom, id, event.operation(), limits(), &cancel())
        .unwrap();
    lease.request_cancel();
    let done = complete(lease.advance(&cancel()).unwrap());
    assert!(done.cancellation_requested());
    assert_eq!(done.dialog_toggle_disposition(), Some(D::TrackerCleared));
}

#[test]
fn close_and_ordinary_creation_and_method_noop_do_not_report_toggle_cleanup() {
    let (mut dom, executor, source, mut task, document, subject) = queued(false);
    let id = task.as_ref().unwrap().info().id();
    let before = dom.snapshot().stamp();
    let event = begin(&mut dom, &source, &mut task, limits());
    let permit = executor.begin_dispatch(&dom, &event, &cancel()).unwrap();
    drop(executor.enter_dispatch(&dom, &permit, &cancel()).unwrap());
    accept(&mut dom, &source, id, &event, &executor, permit, limits());
    let done = complete(
        source
            .resume_dialog_task(&mut dom, id, event.operation(), limits(), &cancel())
            .unwrap()
            .advance(&cancel())
            .unwrap(),
    );
    assert_eq!(done.dialog_toggle_disposition(), None);
    assert_eq!(dom.snapshot().stamp(), before);
    let done = complete(
        dom.begin_source_edit(
            DomSourceEdit::create_text(document.node(), "created"),
            limits(),
            &cancel(),
        )
        .unwrap()
        .advance(&cancel())
        .unwrap(),
    );
    assert!(matches!(
        done.output(),
        Some(crate::DomSourceOutput::Node(_))
    ));
    assert_eq!(done.dialog_toggle_disposition(), None);
    let event = tests::request(&mut dom, subject);
    tests::attest(&mut dom, &executor, &event, DomSourceEventOutcome::Canceled);
    let done = complete(
        dom.resume_source_edit(event.operation(), limits(), &cancel())
            .unwrap()
            .advance(&cancel())
            .unwrap(),
    );
    assert_eq!(done.dialog_toggle_disposition(), None);
    assert!(!done.cancellation_requested());
}
