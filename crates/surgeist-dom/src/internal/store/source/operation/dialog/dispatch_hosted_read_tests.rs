//! Actual affine Source-dispatch qualification, without a fake hosted owner.
use super::tests::{fixture, limits, request};
use super::*;
use crate::internal::hosted::DomHostedError;

#[test]
fn dispatch_reads_require_the_current_active_scope_and_stop_when_it_drops() {
    let cancel = AtomicBool::new(false);
    let (mut dom, executor, document, subject, _) = fixture();
    let event = request(&mut dom, subject);
    let blocked = crate::internal::hosted::DomHostedError::Dom(DomError::IncompleteSourceOperation);
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
        !dom.dispatch_document_fully_active_with::<DomHostedError>(document, &cancel, &mut || Ok(
            ()
        ))
        .unwrap()
    );
    assert!(
        dom.source_operation
            .as_ref()
            .unwrap()
            .callback_observation()
            .unwrap()
            .work
            .work
            .load(Ordering::Relaxed)
            > 0
    );
    drop(scope);
    assert_eq!(
        dom.dispatch_document_fully_active_with::<DomHostedError>(
            document,
            &cancel,
            &mut || Ok(())
        )
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
            crate::internal::hosted::DomHostedError::Dom(DomError::IncompleteSourceOperation)
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
        crate::internal::hosted::DomHostedError::Dom(DomError::ForeignOwner)
    );
    assert_eq!(
        dom.dispatch_document_fully_active_with::<DomHostedError>(
            document,
            &AtomicBool::new(true),
            &mut || Ok(())
        )
        .unwrap_err(),
        crate::internal::hosted::DomHostedError::Dom(DomError::Cancelled)
    );
    drop(executor);
    assert_eq!(
        dom.dispatch_relevant_global(subject, &cancel).unwrap_err(),
        crate::internal::hosted::DomHostedError::Dom(DomError::InvalidSourceEvent)
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
        !dom.dispatch_document_fully_active_with::<DomHostedError>(document, &cancel, &mut || Ok(
            ()
        ))
        .unwrap()
    );
    let (other, _, _, _, _) = fixture();
    assert_eq!(
        other
            .dispatch_document_fully_active_with::<DomHostedError>(
                document,
                &cancel,
                &mut || Ok(())
            )
            .unwrap_err(),
        crate::internal::hosted::DomHostedError::Dom(DomError::ForeignOwner)
    );
}

#[test]
fn never_hosted_activity_reads_charge_the_actual_source_record() {
    let cancel = AtomicBool::new(false);
    let (mut dom, executor, document, subject, _) = fixture();
    let event = request(&mut dom, subject);
    let permit = executor.begin_dispatch(&dom, &event, &cancel).unwrap();
    let scope = executor.enter_dispatch(&dom, &permit, &cancel).unwrap();
    let spent = &dom
        .source_operation
        .as_ref()
        .unwrap()
        .callback_observation()
        .unwrap()
        .work
        .work;
    let mut lookups = 0;
    dom.state
        .node_with::<DomError>(dom.owner, document.node(), &mut || {
            lookups += 1;
            Ok(())
        })
        .unwrap();
    assert!(lookups > 0);
    let before_qualification = spent.load(Ordering::Relaxed);
    let proof = dom.dispatch_hosted_read_work(&cancel).unwrap();
    assert!(proof.is_some());
    let qualification = spent.load(Ordering::Relaxed) - before_qualification;
    let before = spent.load(Ordering::Relaxed);
    let mut caller = 0;
    assert!(
        !dom.dispatch_document_fully_active_with::<DomHostedError>(document, &cancel, &mut || {
            caller += 1;
            Ok(())
        })
        .unwrap()
    );
    assert_eq!(caller, lookups);
    assert_eq!(
        spent.load(Ordering::Relaxed) - before,
        qualification + lookups
    );

    let before_refusal = spent.load(Ordering::Relaxed);
    let error = dom
        .dispatch_document_fully_active_with::<DomHostedError>(document, &cancel, &mut || {
            cancel.store(true, Ordering::Relaxed);
            Ok(())
        })
        .unwrap_err();
    assert_eq!(error, DomHostedError::Dom(DomError::Cancelled));
    assert_eq!(
        spent.load(Ordering::Relaxed) - before_refusal,
        qualification
    );
    drop(scope);
}
