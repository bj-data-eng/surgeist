//! Canonical protocol tests at the explicit trusted-executor boundary. These
//! exercise actual owner mutation and affinity, not Runtime event delivery;
//! the consuming dispatcher integration supplies real listener evidence.
use super::*;
pub(super) fn limits() -> DomSourceLimits {
    DomSourceLimits::new(32_000_000, 1_000_000, 10_000_000, 64, 4096, 4096, 4)
}
pub(super) fn fixture() -> (Dom, DomSourceEventExecutor, DocumentId, NodeId, NodeId) {
    let cancel = AtomicBool::new(false);
    let mut dom = Dom::new(
        DomLimits::new(128, 100_000, 256, 128, 128, 32, 4_000_000)
            .unwrap()
            .with_dispatch_custody(
                crate::DomDispatchCustodyLimits::new(64, 4096, 16_000_000, 16_000_000, 4_000_000)
                    .unwrap(),
            ),
    )
    .unwrap();
    let mut batch = DomBatch::new();
    let document = batch.create_document();
    let first = batch.create_element_ns(&document, crate::Namespace::html(), "dialog");
    let second = batch.create_element_ns(&document, crate::Namespace::html(), "dialog");
    batch.insert_before(&first, &document, None);
    let commit = dom.apply(batch, &cancel).unwrap();
    let ids = (
        commit.document(&document).unwrap(),
        commit.created(&first).unwrap().unwrap(),
        commit.created(&second).unwrap().unwrap(),
    );
    let executor = dom.bind_source_event_executor(&cancel).unwrap();
    (dom, executor, ids.0, ids.1, ids.2)
}
pub(super) fn request(dom: &mut Dom, subject: NodeId) -> DomSourceEventRequest {
    let mut lease = dom
        .begin_source_edit(
            DomSourceEdit::show_dialog(subject),
            limits(),
            &AtomicBool::new(false),
        )
        .unwrap();
    let DomSourceProgress::Request(DomSourceRequest::Event(request)) =
        lease.advance(&AtomicBool::new(false)).unwrap()
    else {
        panic!("actual initial event boundary");
    };
    assert!(!lease.status().has_published());
    assert_eq!(
        request
            .toggle_event()
            .expect("ToggleEvent source payload")
            .old_state(),
        surgeist_html::HtmlDialogState::Closed
    );
    assert_eq!(
        request
            .toggle_event()
            .expect("ToggleEvent source payload")
            .new_state(),
        surgeist_html::HtmlDialogState::Open
    );
    assert!(request.cancelable());
    assert_eq!(
        request
            .toggle_event()
            .expect("ToggleEvent source payload")
            .source(),
        None
    );
    assert_eq!(lease.status().phase(), DomSourcePhase::EventObservation);
    request
}
pub(super) fn attest(
    dom: &mut Dom,
    executor: &DomSourceEventExecutor,
    request: &DomSourceEventRequest,
    outcome: DomSourceEventOutcome,
) {
    let cancel = AtomicBool::new(false);
    let permit = executor.begin_dispatch(dom, request, &cancel).unwrap();
    let scope = executor.enter_dispatch(dom, &permit, &cancel).unwrap();
    drop(scope);
    let mut permit = Some(permit);
    let mut publication = Some(
        executor
            .seal_completed(dom, &mut permit, outcome, &cancel)
            .unwrap(),
    );
    assert!(permit.is_none());
    dom.resume_source_edit(request.operation(), limits(), &cancel)
        .unwrap()
        .accept_source_event(request, &mut publication, &cancel)
        .unwrap();
    assert!(publication.is_none());
}

#[test]
fn request_close_called_boundary_reserves_nested_close_before_request_state_publication() {
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
    issuer
        .observe()
        .validate(
            issuer.id(),
            crate::hosted::Role::Global,
            global.slot,
            global.generation,
        )
        .unwrap();
    let original_source = {
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
    // Controlled canonical stored-watcher input for the Source preflight. The
    // original issuer token is real; no manager registration, Window activity,
    // request action or external service reply is fabricated by this fixture.
    let watcher = crate::DomCloseWatcherId {
        operation: original_source,
        command: 0,
        occurrence: 0,
        target: subject,
        global,
        window_document: document,
    };
    let b = DomBatch::new();
    let mut staged = Draft::new_source(&dom, b.owner.clone(), &b.commands, limits(), &cancel)
        .unwrap()
        .0;
    let key = staged.resolve(subject.into()).unwrap();
    staged
        .set_dialog_watcher(key, Some(watcher), &cancel)
        .unwrap();
    let prepared = staged.finish(&dom, &cancel).unwrap();
    dom.publish(prepared, &cancel).unwrap();
    assert_eq!(
        dom.hosted_revisions, 0,
        "no unfinished hosted constructor contaminates the boundary"
    );
    dom.revision = DomRevision(u64::MAX - 2);
    let before = dom.snapshot();
    assert!(
        !before
            .node(subject)
            .unwrap()
            .as_dialog()
            .unwrap()
            .force_close_watcher_enabled()
    );
    assert_eq!(
        before
            .node(subject)
            .unwrap()
            .as_dialog()
            .unwrap()
            .request_close_return_value(),
        None
    );
    let mut lease = dom
        .begin_source_edit(
            DomSourceEdit::request_close_dialog(subject, Some("new request".into())),
            limits(),
            &cancel,
        )
        .unwrap();
    // Explicit private caller-frame precondition: only the identity-free HTML
    // cursor is supplied after its initial guards. This does not assert native
    // guard evaluation, an active Window, manager registration or any completed
    // external service. The tested boundary starts at CaptureCloseWatcher.
    let mut caller =
        surgeist_html::HtmlDialogCursor::new(surgeist_html::HtmlDialogOperation::RequestClose);
    for reply in [
        surgeist_html::HtmlDialogReply::Open(true),
        surgeist_html::HtmlDialogReply::Connected(true),
        surgeist_html::HtmlDialogReply::FullyActive(true),
    ] {
        caller.reply(reply).unwrap();
    }
    assert_eq!(caller.action(), HtmlDialogAction::CaptureCloseWatcher);
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
        panic!("original requestClose input");
    };
    // The production Draft interpreter captures the actual stored watcher and
    // performs the actual private force/request-slot writes before the call.
    assert_eq!(
        operation.draft.dialog_method_boundary(
            key,
            crate::store::dialog::MethodInput::RequestClose(result.as_ref()),
            lease.dom.limits,
            &cancel
        ),
        Err(DomError::DialogProcessingUnavailable {
            action: HtmlDialogAction::RequestWatcherClose {
                require_history_action_activation: false
            }
        })
    );
    assert!(
        operation
            .draft
            .dialog_state(key)
            .unwrap()
            .force_watcher_enabled
    );
    assert_eq!(
        operation
            .draft
            .dialog_state(key)
            .unwrap()
            .request_return_value
            .as_ref(),
        Some(&DocumentString::from("new request"))
    );
    assert_eq!(lease.snapshot().stamp(), before.stamp());
    // HTML877d46708–46723 calls the original watcher, whose accepted close
    // action calls ordinary Close. Its possible canonical tail has separate
    // Toggle tracking, open-removal/watcher-stop, Close queue, and final reset
    // publications. Two remaining revisions cannot cover the initial request
    // publication plus that tail; refuse before force/request fields escape.
    let called = lease.begin_request_close(&mut operation, false, &cancel);
    lease.dom.source_operation = Some(operation);
    assert!(
        matches!(called, Err(DomError::RevisionExhausted)),
        "called request-state prepublication admission must refuse: {called:?}"
    );
    assert!(!lease.status().has_published());
    let after = lease.snapshot();
    assert_eq!(after.stamp(), before.stamp());
    assert_eq!(after.incomplete_source_operation(), None);
    let dialog = after.node(subject).unwrap().as_dialog().unwrap();
    assert!(!dialog.force_close_watcher_enabled());
    assert_eq!(dialog.request_close_return_value(), None);
    assert_eq!(dialog.request_close_source(), None);
    assert_eq!(dialog.close_watcher(), Some(watcher));
}

#[test]
fn close_observation_preserves_its_remaining_publications_before_scoped_batch_revision() {
    let cancel = AtomicBool::new(false);
    let (mut dom, executor, _, subject, _) = fixture();
    let mut setup = DomBatch::new();
    setup.set_attribute(subject, "open", "");
    dom.apply(setup, &cancel).unwrap();
    assert_eq!(dom.hosted_revisions, 0);
    dom.revision = DomRevision(u64::MAX - 4);
    let before = dom.snapshot();
    let event = {
        let mut lease = dom
            .begin_source_edit(
                DomSourceEdit::close_dialog(subject, None),
                limits(),
                &cancel,
            )
            .unwrap();
        let DomSourceProgress::Request(DomSourceRequest::Event(event)) =
            lease.advance(&cancel).unwrap()
        else {
            panic!("actual Close beforetoggle observation");
        };
        assert!(!event.cancelable());
        assert!(!lease.status().has_published());
        event
    };
    let permit = executor.begin_dispatch(&dom, &event, &cancel).unwrap();
    let scope = executor.enter_dispatch(&dom, &permit, &cancel).unwrap();
    let mut callback = DomBatch::new();
    callback.set_attribute(
        subject,
        "data-actual-scoped-mutation",
        "must not consume the tail",
    );
    // This actual callback mutation keeps `open` and the Close path intact.
    // Pre-exposure reservation must also cover a watcher established/replaced
    // by subsequent callback work: Toggle tracking, possible watcher-stop,
    // Close-queue/open-removal, and final publication are four distinct stops.
    // All four fit before this mutation; consuming one revision would make
    // that permitted tail impossible. No watcher effect is fabricated here.
    let result = dom.apply(callback, &cancel);
    assert!(
        matches!(result, Err(DomError::RevisionExhausted)),
        "scoped mutation must preserve the original Close tail: {result:?}"
    );
    assert_eq!(dom.snapshot().stamp(), before.stamp());
    assert_eq!(
        dom.snapshot()
            .node(subject)
            .unwrap()
            .attribute(&"data-actual-scoped-mutation".into()),
        None
    );
    assert_eq!(
        dom.snapshot()
            .node(subject)
            .unwrap()
            .attribute(&"open".into()),
        Some(&DocumentString::from(""))
    );
    assert_eq!(
        dom.pending_source_edit().unwrap().operation(),
        event.operation()
    );
    assert_eq!(
        dom.pending_source_edit().unwrap().phase(),
        DomSourcePhase::EventObservation
    );
    drop(scope);
    // No completed dispatch or queue service is asserted to manufacture entry.
    drop(permit);
}
#[test]
fn initial_event_request_survives_permission_drop_without_tree_publication_and_prestart_cancel() {
    fn requires_send<T: Send>() {}
    requires_send::<Dom>();
    let (mut dom, _executor, _, first, _) = fixture();
    let before = dom.snapshot().stamp();
    let event = request(&mut dom, first);
    assert_eq!(dom.snapshot().stamp(), before);
    assert_eq!(dom.snapshot().incomplete_source_operation(), None);
    assert_eq!(
        dom.pending_source_edit().unwrap().operation(),
        event.operation()
    );
    let mut lease = dom
        .resume_source_edit(event.operation(), limits(), &AtomicBool::new(false))
        .unwrap();
    lease.request_cancel();
    let DomSourceProgress::Complete(done) = lease.advance(&AtomicBool::new(false)).unwrap() else {
        panic!("prestart cancel");
    };
    assert!(done.cancellation_requested());
    drop(lease);
    assert_eq!(dom.pending_source_edit(), None);
    assert_eq!(dom.snapshot().stamp(), before);
}
#[test]
fn uncanceled_event_stops_at_actual_queue_toggle_and_rebase_preserves_callback_state() {
    let cancel = AtomicBool::new(false);
    let (mut dom, executor, _, first, _) = fixture();
    let event = request(&mut dom, first);
    let permit = executor.begin_dispatch(&dom, &event, &cancel).unwrap();
    let scope = executor.enter_dispatch(&dom, &permit, &cancel).unwrap();
    let mut batch = DomBatch::new();
    batch.set_attribute(first, "data-actual-callback", "kept");
    dom.apply(batch, &cancel).unwrap();
    let callback_stamp = dom.snapshot().stamp();
    drop(scope);
    let mut permit = Some(permit);
    let mut publication = Some(
        executor
            .seal_completed(
                &dom,
                &mut permit,
                DomSourceEventOutcome::NotCanceled,
                &cancel,
            )
            .unwrap(),
    );
    let mut lease = dom
        .resume_source_edit(event.operation(), limits(), &cancel)
        .unwrap();
    lease
        .accept_source_event(&event, &mut publication, &cancel)
        .unwrap();
    let DomSourceProgress::Request(DomSourceRequest::Dialog(service)) =
        lease.advance(&cancel).unwrap()
    else {
        panic!("unfulfilled real tail");
    };
    assert_eq!(service.subject(), first);
    assert!(matches!(
        service.action(),
        HtmlDialogAction::QueueToggle {
            old: surgeist_html::HtmlDialogState::Closed,
            new: surgeist_html::HtmlDialogState::Open,
            ..
        }
    ));
    assert_eq!(
        lease
            .snapshot()
            .node(first)
            .unwrap()
            .attribute(&"data-actual-callback".into()),
        Some(&"kept".into())
    );
    assert_eq!(lease.snapshot().stamp(), callback_stamp);
    assert_eq!(
        lease
            .snapshot()
            .node(first)
            .unwrap()
            .attribute(&"open".into()),
        None
    );
}
#[test]
fn scoped_reentry_closes_on_unwind_and_recursive_show_retains_original_parent() {
    let cancel = AtomicBool::new(false);
    let (mut dom, executor, _, first, second) = fixture();
    let outer = request(&mut dom, first);
    let permit = executor.begin_dispatch(&dom, &outer, &cancel).unwrap();
    let panic = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
        let _scope = executor.enter_dispatch(&dom, &permit, &cancel).unwrap();
        panic!("actual caller unwind");
    }));
    assert!(panic.is_err());
    assert_eq!(dom.source_dispatch_blocker(), Some(outer.operation()));
    let scope = executor.enter_dispatch(&dom, &permit, &cancel).unwrap();
    let nested = request(&mut dom, second);
    assert_eq!(nested.parent_operation(), Some(outer.operation()));
    assert_eq!(dom.source_dispatch_blocker(), Some(nested.operation()));
    assert!(matches!(
        dom.resume_source_edit(outer.operation(), limits(), &cancel),
        Err(DomError::InvalidSourceOperation)
    ));
    attest(
        &mut dom,
        &executor,
        &nested,
        DomSourceEventOutcome::Canceled,
    );
    let DomSourceProgress::Complete(_) = dom
        .resume_source_edit(nested.operation(), limits(), &cancel)
        .unwrap()
        .advance(&cancel)
        .unwrap()
    else {
        panic!("nested canceled");
    };
    assert_eq!(
        dom.pending_source_edit().unwrap().operation(),
        outer.operation()
    );
    assert_eq!(dom.source_dispatch_blocker(), None);
    drop(scope);
    let mut permit = Some(permit);
    let mut publication = Some(
        executor
            .seal_completed(&dom, &mut permit, DomSourceEventOutcome::Canceled, &cancel)
            .unwrap(),
    );
    let mut lease = dom
        .resume_source_edit(outer.operation(), limits(), &cancel)
        .unwrap();
    lease
        .accept_source_event(&outer, &mut publication, &cancel)
        .unwrap();
    assert!(matches!(
        lease.advance(&cancel).unwrap(),
        DomSourceProgress::Complete(_)
    ));
}
#[test]
fn actual_retirement_is_read_only_after_uncanceled_event_but_canceled_event_returns_first() {
    let cancel = AtomicBool::new(false);
    for canceled in [true, false] {
        let (mut dom, executor, _, first, _) = fixture();
        let event = request(&mut dom, first);
        let permit = executor.begin_dispatch(&dom, &event, &cancel).unwrap();
        let scope = executor.enter_dispatch(&dom, &permit, &cancel).unwrap();
        let mut batch = DomBatch::new();
        batch.detach(first);
        batch.destroy(first);
        dom.apply(batch, &cancel).unwrap();
        drop(scope);
        let mut permit = Some(permit);
        let mut publication = Some(
            executor
                .seal_completed(
                    &dom,
                    &mut permit,
                    if canceled {
                        DomSourceEventOutcome::Canceled
                    } else {
                        DomSourceEventOutcome::NotCanceled
                    },
                    &cancel,
                )
                .unwrap(),
        );
        let mut lease = dom
            .resume_source_edit(event.operation(), limits(), &cancel)
            .unwrap();
        lease
            .accept_source_event(&event, &mut publication, &cancel)
            .unwrap();
        if canceled {
            assert!(matches!(
                lease.advance(&cancel).unwrap(),
                DomSourceProgress::Complete(_)
            ));
        } else {
            assert_eq!(
                lease.advance(&cancel).unwrap_err(),
                DomError::SourceReceiverRetired {
                    operation: event.operation(),
                    subject: first,
                    action: HtmlDialogAction::ReadOpen
                }
            );
        }
        drop(lease);
        assert_eq!(dom.pending_source_edit(), None);
        assert!(dom.snapshot().node(first).is_err());
    }
}

#[test]
fn ordinary_batch_reaches_the_same_event_boundary_and_rolls_back_earlier_private_writes() {
    let cancel = AtomicBool::new(false);
    let (mut dom, _executor, document, first, _) = fixture();
    let before = dom.snapshot().stamp();
    let mut batch = DomBatch::new();
    batch.set_attribute(first, "data-before-show", "private");
    batch.show_dialog(first);
    assert!(
        matches!(dom.apply(batch, &cancel), Err(DomError::NeedsEventfulOperation { subject: Some(actual), action: HtmlDialogAction::FireBeforeToggle { .. } }) if actual == first)
    );
    assert_eq!(dom.snapshot().stamp(), before);
    assert_eq!(
        dom.snapshot()
            .node(first)
            .unwrap()
            .attribute(&"data-before-show".into()),
        None
    );
    assert!(dom.pending_source_edit().is_none());
    let mut batch = DomBatch::new();
    batch.show_dialog(document.node());
    assert_eq!(
        dom.apply(batch, &cancel).unwrap_err(),
        DomError::DialogInapplicable
    );
    assert_eq!(dom.snapshot().stamp(), before);
}

#[test]
fn policy_only_raises_reject_foreign_completed_and_lowering_requests_without_changing_policy() {
    let cancel = AtomicBool::new(false);
    let (mut dom, executor, _, first, second) = fixture();
    let event = request(&mut dom, first);
    let (mut foreign, _foreign_executor, _, foreign_first, _) = fixture();
    let foreign_request = request(&mut foreign, foreign_first);
    let stamp = dom.snapshot().stamp();
    let original = dom
        .source_operation
        .as_ref()
        .unwrap()
        .observation()
        .unwrap()
        .limits;
    assert_eq!(
        dom.raise_source_event_limits(&foreign_request, limits(), &cancel),
        Err(DomError::InvalidSourceEvent)
    );
    for field in 0..7 {
        let mut lower = original;
        match field {
            0 => lower.retained_bytes -= 1,
            1 => lower.retained_units -= 1,
            2 => lower.work -= 1,
            3 => lower.phases -= 1,
            4 => lower.outputs -= 1,
            5 => lower.identities -= 1,
            6 => lower.nested_operations -= 1,
            _ => unreachable!(),
        }
        assert_eq!(
            dom.raise_source_event_limits(&event, lower, &cancel),
            Err(DomError::SourceLimitReduction)
        );
        assert_eq!(
            dom.source_operation
                .as_ref()
                .unwrap()
                .observation()
                .unwrap()
                .limits,
            original
        );
        assert_eq!(event.work.work_limit.load(Ordering::Relaxed), original.work);
        assert_eq!(dom.snapshot().stamp(), stamp);
        assert_eq!(
            dom.pending_source_edit().unwrap().operation(),
            event.operation()
        );
    }
    let spent = dom.pending_source_edit().unwrap().spent_work();
    let mut below_spent = original;
    below_spent.work = spent - 1;
    assert_eq!(
        dom.raise_source_event_limits(&event, below_spent, &cancel),
        Err(DomError::SourceWorkLimit {
            limit: below_spent.work
        })
    );
    let mut no_close = original;
    no_close.phases = 0;
    assert_eq!(
        dom.raise_source_event_limits(&event, no_close, &cancel),
        Err(DomError::SourcePublicationLimit)
    );
    let mut no_owner = original;
    no_owner.nested_operations = 0;
    assert_eq!(
        dom.raise_source_event_limits(&event, no_owner, &cancel),
        Err(DomError::SourceNestedOperationLimit { limit: 0 })
    );
    let mut raised = original;
    raised.work += 1000;
    dom.raise_source_event_limits(&event, raised, &cancel)
        .unwrap();
    assert!(dom.pending_source_edit().unwrap().spent_work() > spent);
    assert_eq!(event.work.work_limit.load(Ordering::Relaxed), raised.work);
    assert_eq!(dom.snapshot().stamp(), stamp);
    attest(&mut dom, &executor, &event, DomSourceEventOutcome::Canceled);
    {
        let mut lease = dom
            .resume_source_edit(event.operation(), raised, &cancel)
            .unwrap();
        assert!(matches!(
            lease.advance(&cancel).unwrap(),
            DomSourceProgress::Complete(_)
        ));
    }
    let next = request(&mut dom, second);
    let before = dom.pending_source_edit().unwrap();
    assert_eq!(
        dom.raise_source_event_limits(&event, raised, &cancel),
        Err(DomError::InvalidSourceEvent)
    );
    assert_eq!(dom.pending_source_edit().unwrap(), before);
    assert_eq!(
        dom.pending_source_edit().unwrap().operation(),
        next.operation()
    );
}

#[test]
fn movable_external_payload_drop_and_formed_growth_conserve_the_original_live_debit() {
    let cancel = AtomicBool::new(false);
    let (mut dom, executor, _, first, _) = fixture();
    let event = request(&mut dom, first);
    let permit = executor.begin_dispatch(&dom, &event, &cancel).unwrap();
    let scope = executor.enter_dispatch(&dom, &permit, &cancel).unwrap();
    let requested = 32;
    let mut external_credit = dom
        .retain_source_event_payload(
            &event,
            size_of::<Vec<u16>>() + requested * size_of::<u16>(),
            requested,
            &cancel,
        )
        .unwrap();
    let mut external = Vec::<u16>::new();
    external.try_reserve_exact(requested).unwrap();
    let extra = external.capacity() - requested;
    if extra != 0 {
        external_credit
            .absorb(
                dom.retain_source_event_payload(&event, extra * size_of::<u16>(), extra, &cancel)
                    .unwrap(),
            )
            .unwrap();
    }
    let formation = dom.source_observation_formation(&cancel).unwrap().unwrap();
    let barrier = Arc::new(std::sync::Barrier::new(2));
    let start = barrier.clone();
    let mut local = Vec::<u16>::new();
    std::thread::scope(|threads| {
        let release = threads.spawn(move || {
            start.wait();
            drop(external);
            drop(external_credit);
        });
        barrier.wait();
        formation
            .retain_bytes(size_of::<Vec<u16>>(), &cancel)
            .unwrap();
        formation::reserve_utf16(&mut local, 16, Some(&formation), &cancel).unwrap();
        release.join().unwrap();
    });
    let owned = formation.owned_payload();
    assert_eq!(
        owned.bytes,
        size_of::<Vec<u16>>() + local.capacity() * size_of::<u16>()
    );
    assert_eq!(owned.units, local.capacity());
    assert_eq!(event.work.bytes.load(Ordering::Relaxed), owned.bytes);
    assert_eq!(event.work.units.load(Ordering::Relaxed), owned.units);
    drop(local);
    drop(formation);
    assert_eq!(event.work.bytes.load(Ordering::Relaxed), 0);
    assert_eq!(event.work.units.load(Ordering::Relaxed), 0);
    drop(scope);
}

#[test]
fn legacy_details_transfer_refuses_during_observation_and_preserves_the_actual_intention() {
    let cancel = AtomicBool::new(false);
    let (mut dom, executor, document, first, _) = fixture();
    dom.set_limits(dom.limits().with_details_toggle_intentions(4).unwrap())
        .unwrap();
    let mut setup = DomBatch::new();
    let details = setup.create_element_ns(document, crate::Namespace::html(), "details");
    setup.insert_before(&details, first, None);
    setup.set_attribute(&details, "open", "");
    let commit = dom.apply(setup, &cancel).unwrap();
    let details = commit.created(&details).unwrap().unwrap();
    let original = dom.snapshot();
    assert_eq!(dom.pending_details_toggle_intentions(), 1);
    let event = request(&mut dom, first);
    let permit = executor.begin_dispatch(&dom, &event, &cancel).unwrap();
    let scope = executor.enter_dispatch(&dom, &permit, &cancel).unwrap();
    let before = dom.snapshot().stamp();
    assert!(matches!(
        dom.take_details_toggle_intentions(4, &cancel),
        Err(DomError::SourceOperationPending)
    ));
    assert_eq!(dom.snapshot().stamp(), before);
    assert_eq!(dom.pending_details_toggle_intentions(), 1);
    assert!(!dom.has_transferred_details_toggle_intentions());
    assert_eq!(
        dom.snapshot().details_toggle_intentions().unwrap(),
        original.details_toggle_intentions().unwrap()
    );
    drop(scope);
    let mut permit = Some(permit);
    let mut publication = Some(
        executor
            .seal_completed(&dom, &mut permit, DomSourceEventOutcome::Canceled, &cancel)
            .unwrap(),
    );
    {
        let mut lease = dom
            .resume_source_edit(event.operation(), limits(), &cancel)
            .unwrap();
        lease
            .accept_source_event(&event, &mut publication, &cancel)
            .unwrap();
        assert!(matches!(
            lease.advance(&cancel).unwrap(),
            DomSourceProgress::Complete(_)
        ));
    }
    let transfer = dom.take_details_toggle_intentions(4, &cancel).unwrap();
    let records: Vec<_> = transfer.into_iter().collect();
    assert_eq!(records.len(), 1);
    assert_eq!(records[0].target(), details);
    assert_eq!(
        (records[0].old_state(), records[0].new_state()),
        (
            surgeist_html::HtmlDetailsState::Closed,
            surgeist_html::HtmlDetailsState::Open
        )
    );
    assert_eq!(dom.pending_details_toggle_intentions(), 0);
}

#[test]
fn canonical_limit_update_refuses_before_scanning_an_exhausted_event_observer() {
    let cancel = AtomicBool::new(false);
    let (mut dom, executor, _, first, _) = fixture();
    let event = request(&mut dom, first);
    let permit = executor.begin_dispatch(&dom, &event, &cancel).unwrap();
    let scope = executor.enter_dispatch(&dom, &permit, &cancel).unwrap();
    let mut tight = limits();
    tight.work = dom.pending_source_edit().unwrap().spent_work() + 4096;
    drop(
        dom.resume_source_edit(event.operation(), tight, &cancel)
            .unwrap(),
    );
    let mut exhausted = false;
    for _ in 0..=4096 {
        match dom.source_event_step(&event, &cancel) {
            Ok(()) => {}
            Err(error) => {
                assert_eq!(error, DomError::SourceWorkLimit { limit: tight.work });
                exhausted = true;
                break;
            }
        }
    }
    assert!(exhausted, "actual original Work must reach its finite cap");
    let pending = dom.pending_source_edit().unwrap();
    assert_eq!(pending.spent_work(), tight.work);
    let stamp = dom.snapshot().stamp();
    let original = dom.limits();
    let mut lower = original;
    lower.nodes = 1;
    // The live tree already exceeds this proposed bound. Observation reentry
    // must refuse before even the failed lower-policy admission traverses it.
    assert_eq!(dom.set_limits(lower), Err(DomError::SourceOperationPending));
    assert_eq!(dom.limits(), original);
    assert_eq!(dom.pending_source_edit().unwrap(), pending);
    assert_eq!(dom.snapshot().stamp(), stamp);
    assert_eq!(dom.source_dispatch_blocker(), None);

    dom.raise_source_event_limits(&event, limits(), &cancel)
        .unwrap();
    drop(scope);
    let mut permit = Some(permit);
    let mut publication = Some(
        executor
            .seal_completed(&dom, &mut permit, DomSourceEventOutcome::Canceled, &cancel)
            .unwrap(),
    );
    let mut lease = dom
        .resume_source_edit(event.operation(), limits(), &cancel)
        .unwrap();
    lease
        .accept_source_event(&event, &mut publication, &cancel)
        .unwrap();
    assert!(publication.is_none());
    assert!(matches!(
        lease.advance(&cancel).unwrap(),
        DomSourceProgress::Complete(_)
    ));
}

#[test]
fn observation_resume_preserves_reserved_close_before_and_after_receipt_acceptance() {
    let cancel = AtomicBool::new(false);
    let (mut dom, executor, _, first, _) = fixture();
    let event = request(&mut dom, first);
    let stamp = dom.snapshot().stamp();
    let original = limits();
    let mut no_close = original;
    no_close.phases = 0;
    for accepted in [false, true] {
        if accepted {
            attest(&mut dom, &executor, &event, DomSourceEventOutcome::Canceled);
        }
        assert!(matches!(
            dom.resume_source_edit(event.operation(), no_close, &cancel),
            Err(DomError::SourcePublicationLimit)
        ));
        let observation = dom
            .source_operation
            .as_ref()
            .unwrap()
            .observation()
            .unwrap();
        assert_eq!(observation.limits, original);
        assert_eq!(observation.request.identity(), event.identity());
        assert!(Arc::ptr_eq(&observation.request.work, &event.work));
        assert_eq!(
            observation.accepted,
            accepted.then_some(DomSourceEventOutcome::Canceled)
        );
        assert_eq!(event.work.work_limit.load(Ordering::Relaxed), original.work);
        let status = dom.pending_source_edit().unwrap();
        assert_eq!(status.operation(), event.operation());
        assert_eq!(status.published_phases(), 0);
        assert!(!status.has_published());
        assert_eq!(
            status.phase(),
            if accepted {
                DomSourcePhase::EventReceiptAccepted
            } else {
                DomSourcePhase::EventObservation
            }
        );
        assert_eq!(dom.snapshot().stamp(), stamp);
    }
    {
        let mut lease = dom
            .resume_source_edit(event.operation(), original, &cancel)
            .unwrap();
        assert!(matches!(
            lease.advance(&cancel).unwrap(),
            DomSourceProgress::Complete(_)
        ));
    }
    assert!(dom.pending_source_edit().is_none());
    assert_eq!(dom.snapshot().stamp(), stamp);
}

#[test]
fn retired_select_skip_requires_idle_even_inside_actual_beforetoggle_dispatch_reentry() {
    let cancel = AtomicBool::new(false);
    let (mut dom, executor, document, dialog, _) = fixture();
    dom.set_limits(
        dom.limits
            .with_select_processing(16, 4_000_000, 10_000_000)
            .unwrap(),
    )
    .unwrap();

    let mut setup = DomBatch::new();
    let select = setup.create_element_ns(document, crate::Namespace::html(), "select");
    let option = setup.create_element_ns(document, crate::Namespace::html(), "option");
    let other = setup.create_element_ns(document, crate::Namespace::html(), "option");
    setup.insert_before(&option, &select, None);
    setup.insert_before(&other, &select, None);
    let created = dom.apply(setup, &cancel).unwrap();
    let select = created.created(&select).unwrap().unwrap();
    let option = created.created(&option).unwrap().unwrap();
    let mut removal = DomBatch::new();
    removal.detach(option);
    dom.apply(removal, &cancel).unwrap();
    let checkpoint = crate::DomNativeCheckpointLimits::new(
        4_000_000,
        10_000_000,
        crate::DomSlotQueryLimits::new(128, 128).unwrap(),
    )
    .unwrap();
    let crate::DomNativeCheckpoint::Select(issued) =
        dom.native_checkpoint_step(checkpoint, &cancel).unwrap()
    else {
        panic!("real issued Select intention")
    };
    assert_eq!(issued.select(), select);
    let historical = issued.source().clone();
    let mut retirement = DomBatch::new();
    retirement.destroy(select);
    let retired = dom.apply(retirement, &cancel).unwrap();
    assert!(retired.changes().changes().iter().any(|change| matches!(
        change,
        DomChange::SelectTaskCancelled {
            operation, select: target,
            reason: crate::DomSelectTaskCancellationReason::TargetDestroyed,
        } if *operation == issued.operation() && *target == select
    )));
    assert_eq!(issued.authorization.status(), 2);
    assert_eq!(dom.pending_native_tasks(), 0);

    let before = dom.snapshot();
    let event = request(&mut dom, dialog);
    let permit = executor.begin_dispatch(&dom, &event, &cancel).unwrap();
    let scope = executor.enter_dispatch(&dom, &permit, &cancel).unwrap();
    // Positive control: this exact real dispatcher scope admits a callback's
    // ordinary atomic entry. The empty callback publishes no fabricated effect.
    dom.apply(DomBatch::new(), &cancel).unwrap();
    assert_eq!(
        dom.pending_source_edit().unwrap().operation(),
        event.operation()
    );
    assert!(!dom.pending_source_edit().unwrap().has_published());
    assert_eq!(dom.snapshot().stamp(), before.stamp());

    // require_source_entry alone succeeds here. A retirement report instead
    // requires source-idle and must not consume its once-only authority.
    assert!(matches!(
        dom.begin_select_completion(
            &issued,
            checkpoint,
            DomSourceLimits::new(0, 0, 0, 0, 0, 0, 0),
            &cancel,
        ),
        Err(DomError::SourceOperationPending)
    ));
    assert!(matches!(
        dom.complete_select_task(&issued, checkpoint, &cancel),
        Err(DomError::SourceOperationPending)
    ));
    assert_eq!(issued.authorization.status(), 2);
    assert_eq!(dom.pending_native_tasks(), 0);
    assert_eq!(
        dom.pending_source_edit().unwrap().operation(),
        event.operation()
    );
    assert_eq!(dom.snapshot().stamp(), before.stamp());

    // Finish the actual trusted-executor protocol after leaving dispatch.
    // Canceling Show's genuine beforetoggle returns before any open/focus/task
    // effect; no synthetic receipt, flag setter or live service is substituted.
    drop(scope);
    let mut permit = Some(permit);
    let mut publication = Some(
        executor
            .seal_completed(&dom, &mut permit, DomSourceEventOutcome::Canceled, &cancel)
            .unwrap(),
    );
    assert!(permit.is_none());
    {
        let mut lease = dom
            .resume_source_edit(event.operation(), limits(), &cancel)
            .unwrap();
        lease
            .accept_source_event(&event, &mut publication, &cancel)
            .unwrap();
        assert!(publication.is_none());
        assert!(matches!(
            lease.advance(&cancel).unwrap(),
            DomSourceProgress::Complete(_)
        ));
    }
    assert!(dom.pending_source_edit().is_none());
    assert_eq!(issued.authorization.status(), 2);
    assert_eq!(dom.snapshot().stamp(), before.stamp());
    assert!(
        dom.snapshot()
            .node(dialog)
            .unwrap()
            .attribute(&"open".into())
            .is_none()
    );

    let completion = match dom
        .begin_select_completion(
            &issued,
            checkpoint,
            DomSourceLimits::new(0, 0, 0, 0, 0, 0, 0),
            &cancel,
        )
        .unwrap()
    {
        crate::DomSelectCompletionStart::Skipped(completion) => completion,
        crate::DomSelectCompletionStart::Source(_) => panic!("retirement is not Source execution"),
    };
    assert_eq!(
        completion.outcome(),
        crate::DomSelectTaskOutcome::SkippedTargetDestroyed
    );
    assert!(completion.changes().is_none());
    assert_eq!(issued.authorization.status(), 1);
    assert!(dom.pending_source_edit().is_none());
    assert_eq!(dom.pending_native_tasks(), 0);
    assert_eq!(dom.snapshot().stamp(), before.stamp());
    assert!(historical.node(select).is_ok());
    assert!(matches!(
        dom.snapshot().node(select),
        Err(DomError::StaleNode)
    ));
    assert!(matches!(
        dom.begin_select_completion(&issued, checkpoint, limits(), &cancel),
        Err(DomError::InvalidNativeTask)
    ));
}
