//! Actual Close input lifetime, canonical receipt/rebase and pre-effect admission.
//! Trusted completion is the canonical owning boundary, not Runtime dispatch evidence.
use super::tests::{attest, fixture, limits};
use super::*;
mod name_closing;

fn result(units: usize) -> DocumentString {
    let mut backing = Vec::new();
    backing.try_reserve_exact(units).unwrap();
    backing.resize(units, 0xd800);
    assert_eq!(backing.capacity(), units);
    DocumentString::from_utf16(backing)
}
fn close_observer(
    value: Option<DocumentString>,
) -> (Dom, DomSourceEventExecutor, NodeId, DomSourceEventRequest) {
    let (mut dom, executor, _, subject, _) = fixture();
    let mut setup = DomBatch::new();
    setup.set_attribute(subject, "open", "");
    dom.apply(setup, &AtomicBool::new(false)).unwrap();
    let event = {
        let mut lease = dom
            .begin_source_edit(
                DomSourceEdit::close_dialog(subject, value),
                limits(),
                &AtomicBool::new(false),
            )
            .unwrap();
        let DomSourceProgress::Request(DomSourceRequest::Event(event)) =
            lease.advance(&AtomicBool::new(false)).unwrap()
        else {
            panic!("real noncancelable Close beforetoggle required")
        };
        assert!(!event.cancelable());
        let payload = event.toggle_event().unwrap();
        assert_eq!(payload.old_state(), surgeist_html::HtmlDialogState::Open);
        assert_eq!(payload.new_state(), surgeist_html::HtmlDialogState::Closed);
        assert_eq!(payload.source(), None);
        event
    };
    (dom, executor, subject, event)
}
fn resource_limit(policy: &mut DomSourceLimits, resource: DomResource, amount: usize) {
    match resource {
        DomResource::SourceRetainedBytes => policy.retained_bytes = amount,
        DomResource::SourceRetainedUnits => policy.retained_units = amount,
        _ => unreachable!(),
    }
}
fn is_limit(error: DomError, resource: DomResource, amount: usize) {
    assert_eq!(
        error,
        DomError::ResourceLimit {
            resource,
            limit: amount
        }
    );
}
#[test]
fn close_observation_owns_exactly_one_original_result_backing_at_each_limit() {
    for units in [0, 17] {
        let value = result(units);
        let weak = Arc::downgrade(value.shared_code_units());
        let (mut dom, _, subject, event) = close_observer(Some(value));
        let occurrence_bytes = size_of::<Vec<u16>>() + units * size_of::<u16>();
        // Enumerate this original command occurrence independently. Its
        // immutable event request owns no cloned command result.
        let observation = dom
            .source_operation
            .as_ref()
            .unwrap()
            .observation()
            .unwrap();
        assert_eq!(observation.inputs.bytes, occurrence_bytes);
        assert_eq!(observation.inputs.units, units);
        let expected = dom
            .source_observation_total(storage::Payload::default(), &AtomicBool::new(false))
            .unwrap();
        for resource in [
            DomResource::SourceRetainedBytes,
            DomResource::SourceRetainedUnits,
        ] {
            let exact = match resource {
                DomResource::SourceRetainedBytes => expected.bytes,
                _ => expected.units,
            };
            if exact > 0 {
                let mut short = limits();
                resource_limit(&mut short, resource, exact - 1);
                match dom.resume_source_edit(event.operation(), short, &AtomicBool::new(false)) {
                    Err(error) => is_limit(error, resource, exact - 1),
                    Ok(_) => panic!("one-short original input was admitted"),
                }
            }
            let mut policy = limits();
            resource_limit(&mut policy, resource, exact);
            let lease = dom
                .resume_source_edit(event.operation(), policy, &AtomicBool::new(false))
                .unwrap();
            assert_eq!(lease.status().phase(), DomSourcePhase::EventObservation);
            assert_eq!(
                lease
                    .snapshot()
                    .node(subject)
                    .unwrap()
                    .as_dialog()
                    .unwrap()
                    .return_value(),
                &DocumentString::default()
            );
        }
        assert!(weak.upgrade().is_some());
        drop(dom);
        assert!(
            weak.upgrade().is_none(),
            "the command releases its actual input, not an event-payload copy"
        );
    }
}
fn advance_rebase(
    dom: &mut Dom,
    subject: NodeId,
    event: &DomSourceEventRequest,
    resource: DomResource,
    cap: usize,
) -> Result<DomSourceDialogRequest, DomError> {
    let mut policy = limits();
    resource_limit(&mut policy, resource, cap);
    let queue = {
        let mut lease =
            dom.resume_source_edit(event.operation(), policy, &AtomicBool::new(false))?;
        assert!(lease.dom.source_parents.is_empty());
        assert_eq!(event.work.bytes.load(Ordering::Relaxed), 0);
        assert_eq!(event.work.units.load(Ordering::Relaxed), 0);
        let progress = lease.advance(&AtomicBool::new(false))?;
        let DomSourceProgress::Request(DomSourceRequest::Dialog(queue)) = progress else {
            panic!("the authentic receipt must reach original QueueToggle")
        };
        assert!(matches!(
            queue.action(),
            HtmlDialogAction::QueueToggle { .. }
        ));
        queue
    };
    assert_eq!(queue.subject(), subject);
    Ok(queue)
}
#[test]
fn original_result_survives_same_owner_admission_refusal_and_transfers_once_on_rebase() {
    for units in [0, 23] {
        let value = result(units);
        let weak = Arc::downgrade(value.shared_code_units());
        let (mut dom, executor, subject, event) = close_observer(Some(value));
        attest(
            &mut dom,
            &executor,
            &event,
            DomSourceEventOutcome::NotCanceled,
        );
        refuse_original_observation(&mut dom, &event, units);
        let queue = advance_rebase(
            &mut dom,
            subject,
            &event,
            DomResource::SourceRetainedBytes,
            limits().retained_bytes,
        )
        .unwrap();
        assert_eq!(queue.operation(), event.operation());
        assert_eq!(queue.subject(), subject);
        assert_eq!(
            dom.snapshot()
                .node(subject)
                .unwrap()
                .attribute(&"open".into()),
            Some(&"".into())
        );
        assert!(weak.upgrade().is_some());
        let stamp = dom.snapshot().stamp();
        let repeat = advance_rebase(
            &mut dom,
            subject,
            &event,
            DomResource::SourceRetainedBytes,
            limits().retained_bytes,
        )
        .unwrap();
        assert_eq!(repeat.action(), queue.action());
        assert_eq!(dom.snapshot().stamp(), stamp);
        assert_original_result(&dom, &weak);
        drop(dom);
        assert!(weak.upgrade().is_none());
    }
}

fn assert_original_result(dom: &Dom, weak: &std::sync::Weak<Vec<u16>>) {
    let op = dom.source_operation.as_ref().unwrap();
    let OperationRoot::Edit(
        [
            Command::CloseDialog {
                result: Some(value),
                ..
            },
        ],
    ) = &op.commands
    else {
        panic!("one original result-bearing command");
    };
    assert!(std::sync::Weak::ptr_eq(
        weak,
        &Arc::downgrade(value.shared_code_units())
    ));
}

fn refuse_original_observation(dom: &mut Dom, event: &DomSourceEventRequest, units: usize) {
    let observation = dom
        .source_operation
        .as_ref()
        .unwrap()
        .observation()
        .unwrap();
    assert_eq!(
        observation.inputs.bytes,
        size_of::<Vec<u16>>() + units * size_of::<u16>()
    );
    assert_eq!(observation.inputs.units, units);
    assert_eq!(
        observation.accepted,
        Some(DomSourceEventOutcome::NotCanceled)
    );
    // This exact retained operation supplies both boundaries. No other owner's
    // registry layout or allocation history is used to predict its price.
    let current = dom
        .source_observation_total(storage::Payload::default(), &AtomicBool::new(false))
        .unwrap();
    for resource in [
        DomResource::SourceRetainedBytes,
        DomResource::SourceRetainedUnits,
    ] {
        let exact = if resource == DomResource::SourceRetainedBytes {
            current.bytes
        } else {
            current.units
        };
        if exact == 0 {
            continue;
        }
        let mut short = limits();
        resource_limit(&mut short, resource, exact - 1);
        match dom.resume_source_edit(event.operation(), short, &AtomicBool::new(false)) {
            Err(error) => is_limit(error, resource, exact - 1),
            Ok(_) => panic!("one-short original observation was admitted"),
        }
        let mut policy = limits();
        resource_limit(&mut policy, resource, exact);
        let lease = dom
            .resume_source_edit(event.operation(), policy, &AtomicBool::new(false))
            .unwrap();
        assert_eq!(lease.status().phase(), DomSourcePhase::EventReceiptAccepted);
    }
}

#[test]
fn nested_close_inputs_and_callback_input_remain_distinct_until_their_real_owner_returns() {
    let cancel = AtomicBool::new(false);
    let (mut dom, executor, _, first, second) = fixture();
    let mut setup = DomBatch::new();
    setup.set_attribute(first, "open", "");
    setup.set_attribute(second, "open", "");
    dom.apply(setup, &cancel).unwrap();
    let outer_value = result(17);
    let outer_weak = Arc::downgrade(outer_value.shared_code_units());
    let outer = {
        let mut lease = dom
            .begin_source_edit(
                DomSourceEdit::close_dialog(first, Some(outer_value)),
                limits(),
                &cancel,
            )
            .unwrap();
        let DomSourceProgress::Request(DomSourceRequest::Event(event)) =
            lease.advance(&cancel).unwrap()
        else {
            panic!("outer Close beforetoggle")
        };
        event
    };
    let outer_permit = executor.begin_dispatch(&dom, &outer, &cancel).unwrap();
    let outer_scope = executor
        .enter_dispatch(&dom, &outer_permit, &cancel)
        .unwrap();
    let child_value = result(23);
    let child_weak = Arc::downgrade(child_value.shared_code_units());
    let child = {
        let mut lease = dom
            .begin_source_edit(
                DomSourceEdit::close_dialog(second, Some(child_value)),
                limits(),
                &cancel,
            )
            .unwrap();
        let DomSourceProgress::Request(DomSourceRequest::Event(event)) =
            lease.advance(&cancel).unwrap()
        else {
            panic!("child Close beforetoggle")
        };
        assert_eq!(lease.status().parent_operation(), Some(outer.operation()));
        event
    };
    let child_permit = executor.begin_dispatch(&dom, &child, &cancel).unwrap();
    let child_scope = executor
        .enter_dispatch(&dom, &child_permit, &cancel)
        .unwrap();
    let callback_value = DocumentString::from("a distinct callback value");
    let callback_weak = Arc::downgrade(callback_value.shared_code_units());
    assert!(!std::sync::Weak::ptr_eq(&outer_weak, &child_weak));
    assert!(!std::sync::Weak::ptr_eq(&outer_weak, &callback_weak));
    let mut callback = DomBatch::new();
    callback.set_attribute(second, "data-callback", callback_value);
    callback.remove_attribute(second, "open");
    dom.apply(callback, &cancel).unwrap();
    assert!(outer_weak.upgrade().is_some());
    assert!(child_weak.upgrade().is_some());
    assert!(callback_weak.upgrade().is_some());
    drop(child_scope);
    let mut permit = Some(child_permit);
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
    {
        let mut lease = dom
            .resume_source_edit(child.operation(), limits(), &cancel)
            .unwrap();
        lease
            .accept_source_event(&child, &mut publication, &cancel)
            .unwrap();
        assert!(matches!(
            lease.advance(&cancel).unwrap(),
            DomSourceProgress::Complete(_)
        ));
    }
    assert_eq!(
        dom.pending_source_edit().unwrap().operation(),
        outer.operation()
    );
    assert!(
        child_weak.upgrade().is_none(),
        "surviving callback-open removal returns without writing its unused result"
    );
    assert!(outer_weak.upgrade().is_some());
    assert!(
        callback_weak.upgrade().is_some(),
        "the actual callback commit owns its distinct value"
    );
    drop(outer_scope);
    drop(dom);
    assert!(outer_weak.upgrade().is_none());
    assert!(callback_weak.upgrade().is_none());
}

#[test]
fn native_close_queue_preparation_reserves_the_actual_remaining_phases_and_outputs() {
    let cancel = AtomicBool::new(false);
    for (phases, outputs, expected) in [
        (2, 4096, Some(DomError::SourcePublicationLimit)),
        (3, 3, Some(DomError::SourceOutputLimit)),
        (3, 4, None),
    ] {
        let (mut dom, executor, _, first, _) = fixture();
        let source = dom.bind_element_task_source(&cancel).unwrap();
        let mut setup = DomBatch::new();
        setup.set_attribute(first, "open", "");
        dom.apply(setup, &cancel).unwrap();
        let event = {
            let mut lease = dom
                .begin_source_edit(DomSourceEdit::close_dialog(first, None), limits(), &cancel)
                .unwrap();
            let DomSourceProgress::Request(DomSourceRequest::Event(event)) =
                lease.advance(&cancel).unwrap()
            else {
                panic!("real Close beforetoggle")
            };
            event
        };
        attest(
            &mut dom,
            &executor,
            &event,
            DomSourceEventOutcome::NotCanceled,
        );
        let mut policy = limits();
        policy.phases = phases;
        policy.outputs = outputs;
        let stamp = dom.snapshot().stamp();
        let mut lease = dom
            .resume_source_edit(event.operation(), policy, &cancel)
            .unwrap();
        let DomSourceProgress::Request(DomSourceRequest::Dialog(queue)) =
            lease.advance(&cancel).unwrap()
        else {
            panic!("real Close QueueToggle")
        };
        let plan = lease
            .dialog_task_queue_plan(&queue, &source, 1, &cancel)
            .unwrap();
        {
            let candidate = lease.prepare_dialog_task_queue(&queue, &source, plan, &cancel);
            if let Some(expected) = expected {
                match candidate {
                    Err(actual) => assert_eq!(actual, expected),
                    Ok(_) => panic!("one-short Close tail admitted"),
                }
            } else {
                drop(candidate.unwrap());
            }
        }
        assert_eq!(lease.snapshot().stamp(), stamp);
        assert!(
            !lease
                .snapshot()
                .node(first)
                .unwrap()
                .as_dialog()
                .unwrap()
                .has_toggle_task_tracker()
        );
    }
}

// This owns a real parent/child dispatch stack. The child callback publishes
// its distinct attribute input and removes open; its authentic receipt then
// returns without consuming its unused result. Only then can the original
// parent's actual receipt be accepted and its original method be rebased.
fn parent_after_child(value: Option<DocumentString>) -> (Dom, NodeId, DomSourceEventRequest) {
    parent_after_child_observe(value, |_| {})
}
fn parent_after_child_observe(
    value: Option<DocumentString>,
    mut observe: impl FnMut(&mut Dom),
) -> (Dom, NodeId, DomSourceEventRequest) {
    let cancel = AtomicBool::new(false);
    let (mut dom, executor, _, first, second) = fixture();
    let mut setup = DomBatch::new();
    setup.set_attribute(first, "open", "");
    setup.set_attribute(second, "open", "");
    dom.apply(setup, &cancel).unwrap();
    let outer_weak = value
        .as_ref()
        .map(|value| Arc::downgrade(value.shared_code_units()));
    let outer = {
        let mut lease = dom
            .begin_source_edit(DomSourceEdit::close_dialog(first, value), limits(), &cancel)
            .unwrap();
        let DomSourceProgress::Request(DomSourceRequest::Event(event)) =
            lease.advance(&cancel).unwrap()
        else {
            panic!("outer Close beforetoggle")
        };
        event
    };
    let outer_permit = executor.begin_dispatch(&dom, &outer, &cancel).unwrap();
    let outer_scope = executor
        .enter_dispatch(&dom, &outer_permit, &cancel)
        .unwrap();
    observe(&mut dom);
    let child_value = result(23);
    let child_weak = Arc::downgrade(child_value.shared_code_units());
    let child = {
        let mut lease = dom
            .begin_source_edit(
                DomSourceEdit::close_dialog(second, Some(child_value)),
                limits(),
                &cancel,
            )
            .unwrap();
        let DomSourceProgress::Request(DomSourceRequest::Event(event)) =
            lease.advance(&cancel).unwrap()
        else {
            panic!("child Close beforetoggle")
        };
        assert_eq!(lease.status().parent_operation(), Some(outer.operation()));
        event
    };
    let child_permit = executor.begin_dispatch(&dom, &child, &cancel).unwrap();
    let child_scope = executor
        .enter_dispatch(&dom, &child_permit, &cancel)
        .unwrap();
    observe(&mut dom);
    let callback_value = DocumentString::from("a distinct callback value");
    let callback_weak = Arc::downgrade(callback_value.shared_code_units());
    if let Some(outer_weak) = &outer_weak {
        assert!(!std::sync::Weak::ptr_eq(outer_weak, &child_weak));
        assert!(!std::sync::Weak::ptr_eq(outer_weak, &callback_weak));
    }
    let mut callback = DomBatch::new();
    callback.set_attribute(second, "data-callback", callback_value);
    callback.remove_attribute(second, "open");
    dom.apply(callback, &cancel).unwrap();
    if let Some(outer_weak) = &outer_weak {
        assert!(outer_weak.upgrade().is_some());
    }
    assert!(child_weak.upgrade().is_some());
    assert!(callback_weak.upgrade().is_some());
    drop(child_scope);
    let mut permit = Some(child_permit);
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
    {
        let mut lease = dom
            .resume_source_edit(child.operation(), limits(), &cancel)
            .unwrap();
        lease
            .accept_source_event(&child, &mut publication, &cancel)
            .unwrap();
        assert!(matches!(
            lease.advance(&cancel).unwrap(),
            DomSourceProgress::Complete(_)
        ));
    }
    observe(&mut dom);
    assert_eq!(
        dom.pending_source_edit().unwrap().operation(),
        outer.operation()
    );
    assert!(
        child_weak.upgrade().is_none(),
        "surviving callback-open removal returns without writing its unused result"
    );
    if let Some(outer_weak) = &outer_weak {
        assert!(outer_weak.upgrade().is_some());
    }
    assert!(
        callback_weak.upgrade().is_some(),
        "the actual callback commit owns its distinct value"
    );
    drop(outer_scope);
    let mut permit = Some(outer_permit);
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
    dom.resume_source_edit(outer.operation(), limits(), &cancel)
        .unwrap()
        .accept_source_event(&outer, &mut publication, &cancel)
        .unwrap();
    assert!(publication.is_none());
    assert_eq!(
        dom.pending_source_edit().unwrap().operation(),
        outer.operation()
    );
    assert_eq!(
        dom.pending_source_edit().unwrap().phase(),
        DomSourcePhase::EventReceiptAccepted
    );
    assert!(child_weak.upgrade().is_none());
    assert!(callback_weak.upgrade().is_some());
    (dom, first, outer)
}
#[test]
fn parent_result_rebases_once_after_actual_child_and_distinct_callback_overlap() {
    for units in [0, 17] {
        let value = result(units);
        let weak = Arc::downgrade(value.shared_code_units());
        let (mut dom, subject, event) = parent_after_child(Some(value));
        refuse_original_observation(&mut dom, &event, units);
        assert_original_result(&dom, &weak);
        let queue = advance_rebase(
            &mut dom,
            subject,
            &event,
            DomResource::SourceRetainedBytes,
            limits().retained_bytes,
        )
        .unwrap();
        assert_eq!(queue.operation(), event.operation());
        assert_eq!(queue.subject(), subject);
        let stamp = dom.snapshot().stamp();
        let repeat = advance_rebase(
            &mut dom,
            subject,
            &event,
            DomResource::SourceRetainedBytes,
            limits().retained_bytes,
        )
        .unwrap();
        assert_eq!(repeat.operation(), queue.operation());
        assert_eq!(repeat.action(), queue.action());
        assert_eq!(dom.snapshot().stamp(), stamp);
        assert_eq!(
            dom.snapshot()
                .node(subject)
                .unwrap()
                .attribute(&"open".into()),
            Some(&"".into())
        );
        assert_eq!(
            dom.snapshot()
                .node(subject)
                .unwrap()
                .as_dialog()
                .unwrap()
                .return_value(),
            &DocumentString::default()
        );
        assert_original_result(&dom, &weak);
        assert!(weak.upgrade().is_some());
        drop(dom);
        assert!(weak.upgrade().is_none());
    }
}

#[test]
fn measured_external_envelope_follows_nested_source_parent_restore_and_drop() {
    if !scale_metrics::ENABLED {
        return;
    }
    let measurement = scale_metrics::Measurement::begin().unwrap();
    let mut stage = 0;
    let (dom, _, _) = parent_after_child_observe(Some(result(17)), |dom| {
        let external = scale_metrics::source_external_observation().unwrap();
        assert_eq!(external.units(), if stage == 1 { 17 + 23 } else { 17 });
        if stage == 0 {
            let before = external;
            let stack_capacity = dom.source_parents.capacity();
            let subject = dom
                .source_operation
                .as_ref()
                .unwrap()
                .observation()
                .unwrap()
                .request
                .subject();
            let child = dom
                .begin_source_edit(
                    DomSourceEdit::set_attribute(subject, "data-unpublished", "child"),
                    limits(),
                    &AtomicBool::new(false),
                )
                .unwrap();
            assert!(
                scale_metrics::source_external_observation()
                    .unwrap()
                    .units()
                    > before.units()
            );
            drop(child);
            let restored = scale_metrics::source_external_observation().unwrap();
            assert_eq!(restored.units(), before.units());
            assert_eq!(
                restored.bytes(),
                before.bytes()
                    + (dom.source_parents.capacity() - stack_capacity)
                        * size_of::<Box<Operation>>()
            );
        }
        if stage == 1 {
            let subject = dom
                .source_operation
                .as_ref()
                .unwrap()
                .observation()
                .unwrap()
                .request
                .subject();
            let mut callback = DomBatch::new();
            callback.set_attribute(subject, "data-measurement", "callback");
            dom.apply(callback, &AtomicBool::new(false)).unwrap();
            let restored = scale_metrics::source_external_observation().unwrap();
            assert_eq!(restored.units(), 40);
            // Revision history is already controlled by the same ledger; the
            // callback does not add another external history envelope.
            assert_eq!(restored.bytes(), external.bytes());
        }
        stage += 1;
    });
    assert_eq!(stage, 3);
    assert_eq!(
        scale_metrics::source_external_observation()
            .unwrap()
            .units(),
        17
    );
    drop(dom);
    assert_eq!(
        scale_metrics::source_external_observation().unwrap(),
        custody::Usage::default()
    );
    let counters = measurement.finish(scale_metrics::Hooks::default()).unwrap();
    assert_eq!(counters.source_live_units, 0);
}
