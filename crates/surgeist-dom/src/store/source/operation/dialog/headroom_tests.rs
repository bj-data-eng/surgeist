//! Original Source revision-credit tests. Trusted executor outcomes are Source
//! unit inputs, not evidence of a Runtime listener or native Window service.
use super::tests::{attest, fixture, limits, request};
use super::*;

#[test]
fn close_observation_retains_four_revisions_under_lower_phase_policy_and_cancel_releases_credit() {
    for phases in [3, 4] {
        let cancel = AtomicBool::new(false);
        let (mut dom, _executor, _, subject, _) = fixture();
        let mut setup = DomBatch::new();
        setup.set_attribute(subject, "open", "");
        dom.apply(setup, &cancel).unwrap();
        dom.revision = DomRevision(u64::MAX - 4);
        let before = dom.snapshot().stamp();
        let mut policy = limits();
        policy.phases = phases;
        let mut lease = dom
            .begin_source_edit(DomSourceEdit::close_dialog(subject, None), policy, &cancel)
            .unwrap();
        let DomSourceProgress::Request(DomSourceRequest::Event(event)) =
            lease.advance(&cancel).unwrap()
        else {
            panic!("actual Close beforetoggle");
        };
        assert!(!event.cancelable());
        assert_eq!(lease.status().phase(), DomSourcePhase::EventObservation);
        // Three phases may expose an event whose possible tail needs four
        // revisions. The caller may raise policy; canonical capacity cannot grow.
        assert_eq!(lease.dom.source_reserved_closing(&cancel).unwrap(), 4);
        lease.request_cancel();
        let DomSourceProgress::Complete(done) = lease.advance(&cancel).unwrap() else {
            panic!("genuine cancellation before dispatch");
        };
        assert!(done.cancellation_requested());
        assert_eq!(lease.dom.source_reserved_closing(&cancel).unwrap(), 0);
        assert_eq!(lease.snapshot().stamp(), before);
        assert_eq!(lease.snapshot().incomplete_source_operation(), None);
    }
}

#[test]
fn nested_show_observations_sum_original_credit_and_release_only_the_completed_child() {
    // Each callback can leave Toggle, open, watcher and final publications.
    // Seven revisions cannot expose two such callbacks; eight exactly can.
    for available in [7, 8] {
        let cancel = AtomicBool::new(false);
        let (mut dom, executor, _, first, second) = fixture();
        dom.revision = DomRevision(u64::MAX - available);
        let before = dom.snapshot().stamp();
        let parent = request(&mut dom, first);
        let permit = executor.begin_dispatch(&dom, &parent, &cancel).unwrap();
        let scope = executor.enter_dispatch(&dom, &permit, &cancel).unwrap();
        assert_eq!(dom.source_reserved_closing(&cancel).unwrap(), 4);
        let child = {
            let mut lease = dom
                .begin_source_edit(DomSourceEdit::show_dialog(second), limits(), &cancel)
                .unwrap();
            let result = lease.advance(&cancel);
            if available == 7 {
                assert!(matches!(result, Err(DomError::RevisionExhausted)));
                None
            } else {
                let DomSourceProgress::Request(DomSourceRequest::Event(event)) = result.unwrap()
                else {
                    panic!("actual child beforetoggle");
                };
                Some(event)
            }
        };
        if let Some(child) = child {
            assert_eq!(child.parent_operation(), Some(parent.operation()));
            assert_eq!(dom.source_reserved_closing(&cancel).unwrap(), 8);
            attest(&mut dom, &executor, &child, DomSourceEventOutcome::Canceled);
            assert!(matches!(
                dom.resume_source_edit(child.operation(), limits(), &cancel)
                    .unwrap()
                    .advance(&cancel)
                    .unwrap(),
                DomSourceProgress::Complete(_)
            ));
        }
        assert_eq!(dom.source_reserved_closing(&cancel).unwrap(), 4);
        assert_eq!(
            dom.pending_source_edit().unwrap().operation(),
            parent.operation()
        );
        assert_eq!(dom.snapshot().stamp(), before);
        drop(scope);
        let mut permit = Some(permit);
        let mut publication = Some(
            executor
                .seal_completed(&dom, &mut permit, DomSourceEventOutcome::Canceled, &cancel)
                .unwrap(),
        );
        let mut lease = dom
            .resume_source_edit(parent.operation(), limits(), &cancel)
            .unwrap();
        lease
            .accept_source_event(&parent, &mut publication, &cancel)
            .unwrap();
        assert!(matches!(
            lease.advance(&cancel).unwrap(),
            DomSourceProgress::Complete(_)
        ));
        assert_eq!(lease.dom.source_reserved_closing(&cancel).unwrap(), 0);
        assert_eq!(lease.snapshot().stamp(), before);
    }
}

#[test]
fn original_credit_queries_preserve_cancellation_and_work_errors() {
    let cancel = AtomicBool::new(false);
    let (mut dom, _executor, _, subject, _) = fixture();
    let event = request(&mut dom, subject);
    let before = dom.snapshot().stamp();
    assert!(matches!(
        dom.source_reserved_closing(&AtomicBool::new(true)),
        Err(DomError::Cancelled)
    ));
    // Install a real caller policy whose work budget is precisely already spent.
    // It cannot admit even the first original-owner debit; no work is padded.
    let spent = event.work.work.load(Ordering::Relaxed);
    let mut policy = ResumeWorkPolicy::new(event.work.clone(), spent).unwrap();
    policy.commit();
    assert!(matches!(
        dom.source_reserved_closing(&cancel),
        Err(DomError::SourceWorkLimit { limit }) if limit == spent
    ));
    assert_eq!(dom.snapshot().stamp(), before);
    assert_eq!(
        dom.pending_source_edit().unwrap().operation(),
        event.operation()
    );
    // Restore the original policy using its production transition, then retain
    // the same exact event and credit through a lease drop and retry.
    let mut policy = ResumeWorkPolicy::new(event.work.clone(), limits().work).unwrap();
    policy.commit();
    assert_eq!(dom.source_reserved_closing(&cancel).unwrap(), 4);
    let mut lease = dom
        .resume_source_edit(event.operation(), limits(), &cancel)
        .unwrap();
    lease.request_cancel();
    assert!(matches!(
        lease.advance(&cancel).unwrap(),
        DomSourceProgress::Complete(_)
    ));
    assert_eq!(lease.snapshot().stamp(), before);
}

#[test]
fn called_request_close_admits_exact_five_revisions_and_phases_before_its_first_publication() {
    for (available, phases, expected) in [(4, 64, "revision"), (5, 4, "phase"), (5, 5, "exact")] {
        let cancel = AtomicBool::new(false);
        let (mut dom, _executor, document, subject, _) = fixture();
        let mut issuer = dom
            .register_hosted_source(
                crate::DomHostedLimits::new(
                    crate::DomHostedRecordLimits::new(8, 8, 8, 8, 8, 8, 8),
                    8_000_000,
                    8_000_000,
                    1_000_000,
                ),
                &cancel,
            )
            .unwrap();
        let global = issuer
            .issue_global_for_canonical_source_fixture(&cancel)
            .unwrap();
        let original = {
            let mut lease = dom
                .begin_source_edit(
                    DomSourceEdit::set_attribute(subject, "open", ""),
                    limits(),
                    &cancel,
                )
                .unwrap();
            let id = lease.operation();
            assert!(matches!(
                lease.advance(&cancel).unwrap(),
                DomSourceProgress::Complete(_)
            ));
            id
        };
        // Same explicit stored-watcher/caller-frame unit domain as the RED:
        // real issuer and canonical input, no native Window or manager reply.
        let watcher = crate::DomCloseWatcherId {
            operation: original,
            command: 0,
            occurrence: 0,
            target: subject,
            global,
            window_document: document,
        };
        let batch = DomBatch::new();
        let mut draft = Draft::new_source(
            &dom,
            batch.owner.clone(),
            &batch.commands,
            limits(),
            &cancel,
        )
        .unwrap()
        .0;
        let subject_key = draft.resolve(subject.into()).unwrap();
        draft
            .set_dialog_watcher(subject_key, Some(watcher), &cancel)
            .unwrap();
        let prepared = draft.finish(&dom, &cancel).unwrap();
        dom.publish(prepared, &cancel).unwrap();
        assert_eq!(dom.hosted_revisions, 0);
        dom.revision = DomRevision(u64::MAX - available);
        let before = dom.snapshot();
        let mut policy = limits();
        policy.phases = phases;
        let mut lease = dom
            .begin_source_edit(
                DomSourceEdit::request_close_dialog(subject, Some("request result".into())),
                policy,
                &cancel,
            )
            .unwrap();
        let mut caller =
            surgeist_html::HtmlDialogCursor::new(surgeist_html::HtmlDialogOperation::RequestClose);
        for reply in [
            HtmlDialogReply::Open(true),
            HtmlDialogReply::Connected(true),
            HtmlDialogReply::FullyActive(true),
        ] {
            caller.reply(reply).unwrap();
        }
        let mut operation = lease.dom.source_operation.take().unwrap();
        let key = operation.draft.resolve(subject.into()).unwrap();
        operation.draft.method = Some(crate::store::method::Frame::Dialog(
            crate::store::dialog::MethodFrame {
                subject: key,
                cursor: caller,
                state: crate::store::dialog::MethodState::Ready,
                invocation: Invocation::Unissued,
                locals: crate::store::dialog::MethodLocals::RequestClose { watcher: None },
                focusing: None,
            },
        ));
        let Command::RequestCloseDialog { result, .. } = &operation.commands[0] else {
            panic!("original method input");
        };
        assert!(matches!(
            operation.draft.dialog_method_boundary(
                key,
                crate::store::dialog::MethodInput::RequestClose(result.as_ref()),
                lease.dom.limits,
                &cancel,
            ),
            Err(DomError::DialogProcessingUnavailable {
                action: HtmlDialogAction::RequestWatcherClose { .. }
            })
        ));
        // Independent five stops: request slots, Toggle, watcher/open removal,
        // Close queue, final marker/reset. No sixth epilogue is required.
        let result = lease.begin_request_close(&mut operation, false, &cancel);
        lease.dom.source_operation = Some(operation);
        match expected {
            "revision" => assert!(matches!(result, Err(DomError::RevisionExhausted))),
            "phase" => assert!(matches!(result, Err(DomError::SourcePublicationLimit))),
            "exact" => {
                result.unwrap();
                assert_eq!(lease.status().published_phases(), 1);
                let op = lease.dom.source_operation.as_ref().unwrap();
                assert_eq!(op.headroom.as_ref().unwrap().remaining_phases, 4);
                assert!(op.request_close.is_some());
                assert_eq!(lease.snapshot().stamp().revision().get(), u64::MAX - 4);
                let snapshot = lease.snapshot();
                let dialog = snapshot.node(subject).unwrap().as_dialog().unwrap();
                assert!(dialog.force_close_watcher_enabled());
                assert_eq!(
                    dialog.request_close_return_value(),
                    Some(&DocumentString::from("request result"))
                );
                continue;
            }
            _ => unreachable!(),
        }
        assert_eq!(lease.snapshot().stamp(), before.stamp());
        assert!(!lease.status().has_published());
        let snapshot = lease.snapshot();
        let dialog = snapshot.node(subject).unwrap().as_dialog().unwrap();
        assert!(!dialog.force_close_watcher_enabled());
        assert_eq!(dialog.request_close_return_value(), None);
        assert_eq!(dialog.close_watcher(), Some(watcher));
    }
}
