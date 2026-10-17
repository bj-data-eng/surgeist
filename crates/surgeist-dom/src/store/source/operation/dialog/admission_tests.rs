//! Admission through the actual trusted-executor protocol. These are canonical
//! owner tests; trusted completion attestations do not prove Runtime delivery.
use super::tests::{fixture, limits, request};
use super::*;

pub(super) fn update_limits(
    dom: &mut Dom,
    request: &DomSourceEventRequest,
    limits: DomSourceLimits,
) {
    drop(
        dom.resume_source_edit(request.operation(), limits, &AtomicBool::new(false))
            .unwrap(),
    );
}

fn published_state_is_unchanged(dom: &Dom, request: &DomSourceEventRequest) {
    let status = dom.pending_source_edit().unwrap();
    assert_eq!(status.operation(), request.operation());
    assert!(!status.has_published());
    assert_eq!(status.published_phases(), 0);
}

#[test]
fn callback_batches_obey_ancestor_bytes_and_units_without_publishing_a_refused_edit() {
    let cancel = AtomicBool::new(false);
    for resource in [
        DomResource::SourceRetainedBytes,
        DomResource::SourceRetainedUnits,
    ] {
        let (mut dom, executor, _, first, _) = fixture();
        let event = request(&mut dom, first);
        let permit = executor.begin_dispatch(&dom, &event, &cancel).unwrap();
        let scope = executor.enter_dispatch(&dom, &permit, &cancel).unwrap();
        // The actual current owner extent is admitted, with no headroom for a
        // callback's new Draft/strings. No record-layout estimate is the oracle.
        let extent = dom
            .source_observation_total(storage::Payload::default(), &cancel)
            .unwrap();
        let mut tight = limits();
        match resource {
            DomResource::SourceRetainedBytes => tight.retained_bytes = extent.bytes,
            DomResource::SourceRetainedUnits => tight.retained_units = extent.units,
            _ => unreachable!(),
        }
        update_limits(&mut dom, &event, tight);
        let before = dom.snapshot().stamp();
        let work = dom.pending_source_edit().unwrap().spent_work();
        let mut batch = DomBatch::new();
        batch.set_attribute(first, "data-callback", "retained value");
        assert!(matches!(
            dom.apply(batch, &cancel),
            Err(DomError::ResourceLimit { resource: actual, .. }) if actual == resource
        ));
        assert_eq!(dom.snapshot().stamp(), before);
        assert_eq!(
            dom.snapshot()
                .node(first)
                .unwrap()
                .attribute(&"data-callback".into()),
            None
        );
        assert!(dom.pending_source_edit().unwrap().spent_work() > work);
        published_state_is_unchanged(&dom, &event);

        update_limits(&mut dom, &event, limits());
        let work = dom.pending_source_edit().unwrap().spent_work();
        let mut batch = DomBatch::new();
        batch.set_attribute(first, "data-callback", "retained value");
        dom.apply(batch, &cancel).unwrap();
        assert_eq!(
            dom.snapshot()
                .node(first)
                .unwrap()
                .attribute(&"data-callback".into()),
            Some(&"retained value".into())
        );
        assert!(dom.pending_source_edit().unwrap().spent_work() > work);
        // Callback commits own their publications. The observing method has
        // neither published its phase nor produced the callback's outputs.
        published_state_is_unchanged(&dom, &event);
        drop(scope);
    }
}

#[test]
fn callback_work_refusal_retains_the_original_meter_for_a_raised_limit_retry() {
    let cancel = AtomicBool::new(false);
    let (mut dom, executor, _, first, _) = fixture();
    let event = request(&mut dom, first);
    let permit = executor.begin_dispatch(&dom, &event, &cancel).unwrap();
    let scope = executor.enter_dispatch(&dom, &permit, &cancel).unwrap();
    let mut tight = limits();
    tight.work = dom.pending_source_edit().unwrap().spent_work() + 4096;
    update_limits(&mut dom, &event, tight);
    let before = dom.snapshot().stamp();
    let name = "a".repeat(10_000);
    let mut batch = DomBatch::new();
    batch.set_attribute(first, name.clone(), "value");
    assert_eq!(
        dom.apply(batch, &cancel).unwrap_err(),
        DomError::SourceWorkLimit { limit: tight.work }
    );
    let spent = dom.pending_source_edit().unwrap().spent_work();
    assert_eq!(spent, tight.work);
    assert_eq!(dom.snapshot().stamp(), before);
    assert_eq!(
        dom.snapshot()
            .node(first)
            .unwrap()
            .attribute(&name.clone().into()),
        None
    );
    published_state_is_unchanged(&dom, &event);

    update_limits(&mut dom, &event, limits());
    assert!(dom.pending_source_edit().unwrap().spent_work() >= spent);
    let mut batch = DomBatch::new();
    batch.set_attribute(first, name.clone(), "value");
    dom.apply(batch, &cancel).unwrap();
    assert_eq!(
        dom.snapshot().node(first).unwrap().attribute(&name.into()),
        Some(&"value".into())
    );
    assert!(dom.pending_source_edit().unwrap().spent_work() > spent);
    published_state_is_unchanged(&dom, &event);
    drop(scope);
}

#[test]
fn nested_source_and_recursive_show_charge_ancestors_and_obey_the_parent_depth_limit() {
    let cancel = AtomicBool::new(false);
    let (mut dom, executor, _, first, second) = fixture();
    let outer = request(&mut dom, first);
    let outer_permit = executor.begin_dispatch(&dom, &outer, &cancel).unwrap();
    let outer_scope = executor
        .enter_dispatch(&dom, &outer_permit, &cancel)
        .unwrap();
    let work = dom.pending_source_edit().unwrap().spent_work();
    {
        let mut child = dom
            .begin_source_edit(
                DomSourceEdit::set_attribute(second, "data-child", "kept"),
                limits(),
                &cancel,
            )
            .unwrap();
        assert!(matches!(
            child.advance(&cancel).unwrap(),
            DomSourceProgress::Complete(_)
        ));
    }
    assert_eq!(
        dom.snapshot()
            .node(second)
            .unwrap()
            .attribute(&"data-child".into()),
        Some(&"kept".into())
    );
    assert!(dom.pending_source_edit().unwrap().spent_work() > work);
    published_state_is_unchanged(&dom, &outer);

    let mut depth_one = limits();
    depth_one.nested_operations = 1;
    update_limits(&mut dom, &outer, depth_one);
    let before = dom.snapshot().stamp();
    let work = dom.pending_source_edit().unwrap().spent_work();
    assert!(matches!(
        dom.begin_source_edit(DomSourceEdit::show_dialog(second), limits(), &cancel),
        Err(DomError::SourceNestedOperationLimit { limit: 1 })
    ));
    assert_eq!(dom.snapshot().stamp(), before);
    assert!(dom.pending_source_edit().unwrap().spent_work() > work);
    published_state_is_unchanged(&dom, &outer);

    let mut depth_two = limits();
    depth_two.nested_operations = 2;
    update_limits(&mut dom, &outer, depth_two);
    let outer_work = outer.work.work.load(Ordering::Relaxed);
    let nested = request(&mut dom, second);
    assert_eq!(nested.parent_operation(), Some(outer.operation()));
    assert!(outer.work.work.load(Ordering::Relaxed) > outer_work);
    assert_eq!(
        dom.pending_source_edit().unwrap().phase(),
        DomSourcePhase::EventObservation
    );
    let nested_permit = executor.begin_dispatch(&dom, &nested, &cancel).unwrap();
    let nested_scope = executor
        .enter_dispatch(&dom, &nested_permit, &cancel)
        .unwrap();
    let before = dom.snapshot().stamp();
    let outer_work = outer.work.work.load(Ordering::Relaxed);
    assert!(matches!(
        dom.begin_source_edit(DomSourceEdit::show_dialog(first), limits(), &cancel),
        Err(DomError::SourceNestedOperationLimit { limit: 2 })
    ));
    assert_eq!(dom.snapshot().stamp(), before);
    assert!(outer.work.work.load(Ordering::Relaxed) > outer_work);
    assert_eq!(
        dom.pending_source_edit().unwrap().operation(),
        nested.operation()
    );
    drop(nested_scope);
    let mut nested_permit = Some(nested_permit);
    let mut publication = Some(
        executor
            .seal_completed(
                &dom,
                &mut nested_permit,
                DomSourceEventOutcome::Canceled,
                &cancel,
            )
            .unwrap(),
    );
    {
        let mut lease = dom
            .resume_source_edit(nested.operation(), limits(), &cancel)
            .unwrap();
        lease
            .accept_source_event(&nested, &mut publication, &cancel)
            .unwrap();
        assert!(matches!(
            lease.advance(&cancel).unwrap(),
            DomSourceProgress::Complete(_)
        ));
    }
    assert!(publication.is_none());
    published_state_is_unchanged(&dom, &outer);
    drop(outer_scope);
    let mut outer_permit = Some(outer_permit);
    let mut publication = Some(
        executor
            .seal_completed(
                &dom,
                &mut outer_permit,
                DomSourceEventOutcome::Canceled,
                &cancel,
            )
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
fn callback_and_child_outputs_do_not_consume_the_observing_methods_own_counters() {
    let cancel = AtomicBool::new(false);
    let (mut dom, executor, document, first, _) = fixture();
    let event = request(&mut dom, first);
    let mut observing = limits();
    observing.outputs = 0;
    observing.identities = 0;
    observing.phases = 1;
    update_limits(&mut dom, &event, observing);
    let permit = executor.begin_dispatch(&dom, &event, &cancel).unwrap();
    let scope = executor.enter_dispatch(&dom, &permit, &cancel).unwrap();
    let mut batch = DomBatch::new();
    let callback_text = batch.create_text(document, "callback output");
    let commit = dom.apply(batch, &cancel).unwrap();
    let callback_text = commit.created(&callback_text).unwrap().unwrap();
    assert_eq!(
        dom.snapshot().node(callback_text).unwrap().data(),
        Some(&"callback output".into())
    );
    published_state_is_unchanged(&dom, &event);

    let mut child_limits = limits();
    child_limits.outputs = 1;
    child_limits.identities = 1;
    child_limits.phases = 1;
    let output = {
        let mut child = dom
            .begin_source_edit(
                DomSourceEdit::create_text(document.node(), "child output"),
                child_limits,
                &cancel,
            )
            .unwrap();
        let DomSourceProgress::Complete(done) = child.advance(&cancel).unwrap() else {
            panic!("the child owns and completes its own single text output");
        };
        done.output().unwrap()
    };
    let DomSourceOutput::Node(child_text) = output else {
        panic!("the actual child text constructor returns its own Node output");
    };
    assert_ne!(child_text, callback_text);
    assert_eq!(
        dom.snapshot().node(child_text).unwrap().data(),
        Some(&"child output".into())
    );
    published_state_is_unchanged(&dom, &event);
    drop(scope);
}

#[test]
fn public_dispatch_custody_registration_uses_the_actual_affine_scope_and_ancestor_budget() {
    let cancel = AtomicBool::new(false);
    let (mut dom, executor, document, first, second) = fixture();
    let event = request(&mut dom, first);
    assert!(matches!(
        dom.register_dispatch_custody(&[first, second], &cancel),
        Err(DomError::SourceOperationPending)
    ));
    let permit = executor.begin_dispatch(&dom, &event, &cancel).unwrap();
    let scope = executor.enter_dispatch(&dom, &permit, &cancel).unwrap();
    let before = dom.snapshot().stamp();
    let work = dom.pending_source_edit().unwrap().spent_work();
    let custody = dom
        .register_dispatch_custody(&[first, second], &cancel)
        .unwrap();
    assert_eq!(custody.targets(), 2);
    assert_eq!(
        dom.dispatch_custody_association(&custody, 0, first)
            .unwrap(),
        crate::DomDispatchAssociation::Current(document)
    );
    assert_eq!(
        dom.dispatch_custody_association(&custody, 1, second)
            .unwrap(),
        crate::DomDispatchAssociation::Current(document)
    );
    assert!(dom.pending_source_edit().unwrap().spent_work() > work);
    assert_eq!(dom.snapshot().stamp(), before);
    published_state_is_unchanged(&dom, &event);
    drop(custody);

    let extent = dom
        .source_observation_total(storage::Payload::default(), &cancel)
        .unwrap();
    let mut tight = limits();
    tight.retained_bytes = extent.bytes;
    update_limits(&mut dom, &event, tight);
    let work = dom.pending_source_edit().unwrap().spent_work();
    assert!(matches!(
        dom.register_dispatch_custody(&[first, second], &cancel),
        Err(DomError::ResourceLimit {
            resource: DomResource::SourceRetainedBytes,
            ..
        })
    ));
    assert!(dom.pending_source_edit().unwrap().spent_work() > work);
    assert_eq!(dom.snapshot().stamp(), before);
    update_limits(&mut dom, &event, limits());
    let custody = dom
        .register_dispatch_custody(&[first, second], &cancel)
        .unwrap();
    assert_eq!(
        dom.dispatch_custody_association(&custody, 0, first)
            .unwrap(),
        crate::DomDispatchAssociation::Current(document)
    );
    drop(custody);
    drop(scope);
}

#[test]
fn rejected_receipt_and_accepted_receipt_survive_one_short_limits_and_rebase_refusal() {
    let cancel = AtomicBool::new(false);
    let (mut dom, executor, _, first, _) = fixture();
    let event = request(&mut dom, first);
    let permit = executor.begin_dispatch(&dom, &event, &cancel).unwrap();
    let scope = executor.enter_dispatch(&dom, &permit, &cancel).unwrap();
    let mut batch = DomBatch::new();
    batch.set_attribute(first, "data-callback", "preserved");
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
    let candidate = publication.as_ref().unwrap().work.clone();
    {
        let mut lease = dom
            .resume_source_edit(event.operation(), limits(), &cancel)
            .unwrap();
        assert_eq!(
            lease.accept_source_event(&event, &mut publication, &AtomicBool::new(true)),
            Err(DomError::Cancelled)
        );
        assert_eq!(lease.status().phase(), DomSourcePhase::EventObservation);
    }
    assert!(Arc::ptr_eq(&publication.as_ref().unwrap().work, &candidate));
    let extent = dom
        .source_observation_total(storage::Payload::default(), &cancel)
        .unwrap();
    let mut one_short = limits();
    one_short.retained_bytes = extent.bytes.checked_sub(1).unwrap();
    assert!(
        matches!(dom.resume_source_edit(event.operation(), one_short, &cancel), Err(DomError::ResourceLimit { resource: DomResource::SourceRetainedBytes, limit }) if limit == one_short.retained_bytes)
    );
    assert!(Arc::ptr_eq(&publication.as_ref().unwrap().work, &candidate));
    assert_eq!(
        dom.pending_source_edit().unwrap().phase(),
        DomSourcePhase::EventObservation
    );
    assert_eq!(dom.snapshot().stamp(), callback_stamp);

    // The exact current extent admits the receipt. It deliberately gives no
    // extra space for the new canonical Draft required by an uncanceled reply.
    let mut exact = limits();
    exact.retained_bytes = extent.bytes;
    {
        let mut lease = dom
            .resume_source_edit(event.operation(), exact, &cancel)
            .unwrap();
        lease
            .accept_source_event(&event, &mut publication, &cancel)
            .unwrap();
        assert!(publication.is_none());
        assert_eq!(lease.status().phase(), DomSourcePhase::EventReceiptAccepted);
        assert!(matches!(
            lease.advance(&cancel),
            Err(DomError::ResourceLimit {
                resource: DomResource::SourceRetainedBytes,
                ..
            })
        ));
        assert_eq!(lease.status().phase(), DomSourcePhase::EventReceiptAccepted);
        assert_eq!(lease.snapshot().stamp(), callback_stamp);
    }
    assert!(matches!(
        executor.begin_dispatch(&dom, &event, &cancel),
        Err(DomError::InvalidSourceEvent)
    ));
    let mut lease = dom
        .resume_source_edit(event.operation(), limits(), &cancel)
        .unwrap();
    let DomSourceProgress::Request(DomSourceRequest::Dialog(service)) =
        lease.advance(&cancel).unwrap()
    else {
        panic!("the accepted uncanceled reply must reach its real unfulfilled QueueToggle");
    };
    assert_eq!(service.subject(), first);
    assert!(matches!(
        service.action(),
        HtmlDialogAction::QueueToggle { .. }
    ));
    assert_eq!(
        lease
            .snapshot()
            .node(first)
            .unwrap()
            .attribute(&"data-callback".into()),
        Some(&"preserved".into())
    );
    assert_eq!(lease.snapshot().stamp(), callback_stamp);
}

#[test]
fn zero_closing_phase_allowance_refuses_show_before_event_exposure() {
    let cancel = AtomicBool::new(false);
    let (mut dom, _executor, _, first, _) = fixture();
    let before = dom.snapshot().stamp();
    let mut zero = limits();
    zero.phases = 0;
    let result = match dom.begin_source_edit(DomSourceEdit::show_dialog(first), zero, &cancel) {
        Ok(mut lease) => lease.advance(&cancel),
        Err(error) => Err(error),
    };
    assert!(matches!(result, Err(DomError::SourcePublicationLimit)));
    assert_eq!(dom.snapshot().stamp(), before);
    assert_eq!(
        dom.snapshot()
            .node(first)
            .unwrap()
            .attribute(&"open".into()),
        None
    );
}

#[test]
fn retained_child_recovers_after_two_exhausted_ancestors_are_raised_sequentially() {
    fn event_with_limits(
        dom: &mut Dom,
        subject: NodeId,
        policy: DomSourceLimits,
        cancel: &AtomicBool,
    ) -> DomSourceEventRequest {
        let mut lease = dom
            .begin_source_edit(DomSourceEdit::show_dialog(subject), policy, cancel)
            .unwrap();
        let DomSourceProgress::Request(DomSourceRequest::Event(event)) =
            lease.advance(cancel).unwrap()
        else {
            panic!("the real recursive Show must reach its original event boundary");
        };
        event
    }

    let cancel = AtomicBool::new(false);
    let (mut dom, executor, _, first, second) = fixture();
    let mut ancestor_policy = limits();
    ancestor_policy.work = 20_000;
    let outer = event_with_limits(&mut dom, first, ancestor_policy, &cancel);
    let outer_permit = executor.begin_dispatch(&dom, &outer, &cancel).unwrap();
    let outer_scope = executor
        .enter_dispatch(&dom, &outer_permit, &cancel)
        .unwrap();
    let middle = event_with_limits(&mut dom, second, ancestor_policy, &cancel);
    assert_eq!(middle.parent_operation(), Some(outer.operation()));
    let middle_permit = executor.begin_dispatch(&dom, &middle, &cancel).unwrap();
    let middle_scope = executor
        .enter_dispatch(&dom, &middle_permit, &cancel)
        .unwrap();
    let mut child_policy = limits();
    child_policy.work = 1_000_000;
    let child = event_with_limits(&mut dom, first, child_policy, &cancel);
    assert_eq!(child.parent_operation(), Some(middle.operation()));
    let child_permit = executor.begin_dispatch(&dom, &child, &cancel).unwrap();
    let child_scope = executor
        .enter_dispatch(&dom, &child_permit, &cancel)
        .unwrap();
    drop(child_scope);
    let mut child_permit = Some(child_permit);
    let mut publication = Some(
        executor
            .seal_completed(
                &dom,
                &mut child_permit,
                DomSourceEventOutcome::Canceled,
                &cancel,
            )
            .unwrap(),
    );
    let candidate = publication.as_ref().unwrap().work.clone();
    let stamp = dom.snapshot().stamp();

    // Charge through the actual owner's request front, including failed prefix
    // work. No meter is assigned or reconstructed to manufacture exhaustion.
    let maximum_attempts = ancestor_policy.work.checked_mul(2).unwrap() + 1;
    for _ in 0..maximum_attempts {
        match dom.source_event_step(&child, &cancel) {
            Ok(()) | Err(DomError::SourceWorkLimit { .. }) => {}
            Err(error) => panic!("actual child accounting failed unexpectedly: {error:?}"),
        }
        if outer.work.work.load(Ordering::Relaxed) == ancestor_policy.work
            && middle.work.work.load(Ordering::Relaxed) == ancestor_policy.work
        {
            break;
        }
    }
    assert_eq!(
        outer.work.work.load(Ordering::Relaxed),
        ancestor_policy.work
    );
    assert_eq!(
        middle.work.work.load(Ordering::Relaxed),
        ancestor_policy.work
    );
    let child_spent = child.work.work.load(Ordering::Relaxed);
    assert!(matches!(
        dom.resume_source_edit(child.operation(), child_policy, &cancel),
        Err(DomError::SourceWorkLimit { .. })
    ));
    assert!(Arc::ptr_eq(&publication.as_ref().unwrap().work, &candidate));
    published_state_is_unchanged(&dom, &child);
    assert_eq!(dom.snapshot().stamp(), stamp);

    let mut raised = ancestor_policy;
    raised.work = 1_000_000;
    dom.raise_source_event_limits(&outer, raised, &cancel)
        .unwrap();
    assert!(outer.work.work.load(Ordering::Relaxed) >= ancestor_policy.work);
    assert!(middle.work.work.load(Ordering::Relaxed) >= ancestor_policy.work);
    assert!(child.work.work.load(Ordering::Relaxed) >= child_spent);
    assert!(Arc::ptr_eq(&publication.as_ref().unwrap().work, &candidate));
    published_state_is_unchanged(&dom, &child);
    assert_eq!(dom.snapshot().stamp(), stamp);
    // Raising only the outer owner cannot bypass the still-exhausted middle.
    assert!(matches!(
        dom.resume_source_edit(child.operation(), child_policy, &cancel),
        Err(DomError::SourceWorkLimit { .. })
    ));
    dom.raise_source_event_limits(&middle, raised, &cancel)
        .unwrap();
    assert!(outer.work.work.load(Ordering::Relaxed) >= ancestor_policy.work);
    assert!(middle.work.work.load(Ordering::Relaxed) >= ancestor_policy.work);
    assert!(child.work.work.load(Ordering::Relaxed) >= child_spent);
    assert!(Arc::ptr_eq(&publication.as_ref().unwrap().work, &candidate));
    published_state_is_unchanged(&dom, &child);
    assert_eq!(dom.snapshot().stamp(), stamp);

    {
        let mut lease = dom
            .resume_source_edit(child.operation(), child_policy, &cancel)
            .unwrap();
        lease
            .accept_source_event(&child, &mut publication, &cancel)
            .unwrap();
        assert!(publication.is_none());
        assert!(matches!(
            lease.advance(&cancel).unwrap(),
            DomSourceProgress::Complete(_)
        ));
    }
    assert!(child.work.work.load(Ordering::Relaxed) >= child_spent);
    published_state_is_unchanged(&dom, &middle);
    drop(middle_scope);
    let mut middle_permit = Some(middle_permit);
    let mut publication = Some(
        executor
            .seal_completed(
                &dom,
                &mut middle_permit,
                DomSourceEventOutcome::Canceled,
                &cancel,
            )
            .unwrap(),
    );
    {
        let mut lease = dom
            .resume_source_edit(middle.operation(), raised, &cancel)
            .unwrap();
        lease
            .accept_source_event(&middle, &mut publication, &cancel)
            .unwrap();
        assert!(matches!(
            lease.advance(&cancel).unwrap(),
            DomSourceProgress::Complete(_)
        ));
    }
    published_state_is_unchanged(&dom, &outer);
    drop(outer_scope);
    let mut outer_permit = Some(outer_permit);
    let mut publication = Some(
        executor
            .seal_completed(
                &dom,
                &mut outer_permit,
                DomSourceEventOutcome::Canceled,
                &cancel,
            )
            .unwrap(),
    );
    {
        let mut lease = dom
            .resume_source_edit(outer.operation(), raised, &cancel)
            .unwrap();
        lease
            .accept_source_event(&outer, &mut publication, &cancel)
            .unwrap();
        assert!(matches!(
            lease.advance(&cancel).unwrap(),
            DomSourceProgress::Complete(_)
        ));
    }
    assert_eq!(dom.pending_source_edit(), None);
    assert_eq!(dom.snapshot().stamp(), stamp);
}
