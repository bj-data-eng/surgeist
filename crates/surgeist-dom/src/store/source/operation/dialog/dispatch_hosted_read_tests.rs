//! Actual affine Source-dispatch qualification, without a fake hosted owner.
use super::tests::{fixture, limits, request};
use super::*;

#[test]
fn dispatch_reads_require_the_current_active_scope_and_stop_when_it_drops() {
    let cancel = AtomicBool::new(false);
    let (mut dom, executor, document, subject, _) = fixture();
    let event = request(&mut dom, subject);
    let blocked = crate::DomHostedError::Dom(DomError::IncompleteSourceOperation);
    assert_eq!(
        dom.dispatch_relevant_global(document.node(), &cancel)
            .unwrap_err(),
        blocked
    );
    let permit = executor.begin_dispatch(&dom, &event, &cancel).unwrap();
    let scope = executor.enter_dispatch(&dom, &permit, &cancel).unwrap();
    assert_eq!(
        dom.dispatch_relevant_global(document.node(), &cancel)
            .unwrap(),
        None
    );
    assert!(
        !dom.dispatch_document_fully_active(document, &cancel)
            .unwrap()
    );
    assert!(
        dom.source_operation
            .as_ref()
            .unwrap()
            .observed_event_work()
            .unwrap()
            .work
            .load(Ordering::Relaxed)
            > 0
    );
    drop(scope);
    assert_eq!(
        dom.dispatch_document_fully_active(document, &cancel)
            .unwrap_err(),
        blocked
    );
    let scope = executor.enter_dispatch(&dom, &permit, &cancel).unwrap();
    assert_eq!(
        dom.dispatch_relevant_global(subject, &cancel).unwrap(),
        None
    );
    drop(scope);
    let mut permit = Some(permit);
    let _publication = executor
        .seal_completed(
            &dom,
            &mut permit,
            DomSourceEventOutcome::NotCanceled,
            &cancel,
        )
        .unwrap();
    assert_eq!(
        dom.dispatch_relevant_global(subject, &cancel).unwrap_err(),
        blocked
    );
}

#[test]
fn dispatch_reads_refuse_a_pending_child_and_recover_only_the_actual_observing_parent() {
    let cancel = AtomicBool::new(false);
    let (mut dom, executor, document, subject, child_subject) = fixture();
    let event = request(&mut dom, subject);
    let permit = executor.begin_dispatch(&dom, &event, &cancel).unwrap();
    let scope = executor.enter_dispatch(&dom, &permit, &cancel).unwrap();
    {
        let child = dom
            .begin_source_edit(DomSourceEdit::show_dialog(child_subject), limits(), &cancel)
            .unwrap();
        assert_eq!(
            child
                .dom
                .dispatch_relevant_global(document.node(), &cancel)
                .unwrap_err(),
            crate::DomHostedError::Dom(DomError::IncompleteSourceOperation)
        );
        assert_eq!(
            child.dom.source_parents.last().unwrap().id,
            event.operation()
        );
    }
    assert_eq!(
        dom.dispatch_relevant_global(subject, &cancel).unwrap(),
        None
    );
    drop(scope);
}

#[test]
fn expired_executor_foreign_owner_and_canceled_reads_do_not_supply_hosted_authority() {
    let cancel = AtomicBool::new(false);
    let (mut dom, executor, document, subject, _) = fixture();
    let event = request(&mut dom, subject);
    let permit = executor.begin_dispatch(&dom, &event, &cancel).unwrap();
    let scope = executor.enter_dispatch(&dom, &permit, &cancel).unwrap();
    let (other, _, _, _, _) = fixture();
    assert_eq!(
        other
            .dispatch_relevant_global(subject, &cancel)
            .unwrap_err(),
        crate::DomHostedError::Dom(DomError::ForeignOwner)
    );
    assert_eq!(
        dom.dispatch_document_fully_active(document, &AtomicBool::new(true))
            .unwrap_err(),
        crate::DomHostedError::Dom(DomError::Cancelled)
    );
    drop(executor);
    assert_eq!(
        dom.dispatch_relevant_global(subject, &cancel).unwrap_err(),
        crate::DomHostedError::Dom(DomError::InvalidSourceEvent)
    );
    drop(scope);
}

#[test]
fn complete_native_document_reads_keep_actual_closed_activity_and_identity_checks() {
    let cancel = AtomicBool::new(false);
    let (dom, _, document, subject, _) = fixture();
    assert_eq!(dom.relevant_global(subject).unwrap(), None);
    assert_eq!(
        dom.dispatch_relevant_global(subject, &cancel).unwrap(),
        None
    );
    assert!(
        !dom.snapshot()
            .document(document)
            .unwrap()
            .is_fully_active()
            .unwrap()
    );
    assert!(
        !dom.dispatch_document_fully_active(document, &cancel)
            .unwrap()
    );
    let (other, _, _, _, _) = fixture();
    assert_eq!(
        other
            .dispatch_document_fully_active(document, &cancel)
            .unwrap_err(),
        crate::DomHostedError::Dom(DomError::ForeignOwner)
    );
}
