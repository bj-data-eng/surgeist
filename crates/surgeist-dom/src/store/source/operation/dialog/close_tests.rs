//! Actual Close input lifetime, canonical receipt/rebase and pre-effect admission.
//! Trusted completion is the canonical owning boundary, not Runtime dispatch evidence.
use super::tests::{attest, fixture, limits};
use super::*;

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
        let (base, _, _, _) = close_observer(None);
        let baseline = base
            .source_observation_total(storage::Payload::default(), &AtomicBool::new(false))
            .unwrap();
        let value = result(units);
        let weak = Arc::downgrade(value.shared_code_units());
        let (mut dom, _, subject, event) = close_observer(Some(value));
        let occurrence_bytes = size_of::<Vec<u16>>() + units * size_of::<u16>();
        // Equal-shape omitted-result owner supplies only the preexisting
        // baseline. The new input occurrence is independently sized here.
        let expected = storage::Payload {
            bytes: baseline.bytes + occurrence_bytes,
            units: baseline.units + units,
        };
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
#[derive(Debug)]
struct RebaseRefusal {
    error: DomError,
    expected_limit: usize,
}
fn rebase(
    value: Option<DocumentString>,
    resource: DomResource,
    cap: usize,
) -> Result<(Dom, NodeId, DomSourceDialogRequest), RebaseRefusal> {
    let (mut dom, executor, subject, event) = close_observer(value);
    attest(
        &mut dom,
        &executor,
        &event,
        DomSourceEventOutcome::NotCanceled,
    );
    rebase_accepted(dom, subject, event, resource, cap)
}
fn rebase_accepted(
    mut dom: Dom,
    subject: NodeId,
    event: DomSourceEventRequest,
    resource: DomResource,
    cap: usize,
) -> Result<(Dom, NodeId, DomSourceDialogRequest), RebaseRefusal> {
    let queue = advance_rebase(&mut dom, subject, &event, resource, cap)?;
    Ok((dom, subject, queue))
}
fn advance_rebase(
    dom: &mut Dom,
    subject: NodeId,
    event: &DomSourceEventRequest,
    resource: DomResource,
    cap: usize,
) -> Result<DomSourceDialogRequest, RebaseRefusal> {
    let mut policy = limits();
    resource_limit(&mut policy, resource, cap);
    let queue = {
        let mut lease = dom
            .resume_source_edit(event.operation(), policy, &AtomicBool::new(false))
            .map_err(|error| RebaseRefusal {
                error,
                expected_limit: cap,
            })?;
        // There is exactly one original observer, with no external event
        // payload still retained after trusted completion. Its original input
        // transfers into the new Draft once; only these control/header bodies
        // remain additional during formation. Derive their extent from layout,
        // independently of the producer's charged payload counter.
        assert!(lease.dom.source_parents.is_empty());
        assert_eq!(event.work.bytes.load(Ordering::Relaxed), 0);
        assert_eq!(event.work.units.load(Ordering::Relaxed), 0);
        let control_bytes = size_of::<Operation>()
            + size_of::<EventWork>()
            + size_of::<AtomicU64>()
            + 2 * size_of::<AtomicUsize>();
        let advance = lease.advance(&AtomicBool::new(false));
        let progress = advance.map_err(|error| {
            let old_observer_remains = lease
                .dom
                .source_operation
                .as_ref()
                .is_some_and(|operation| operation.observation().is_some());
            let expected_limit =
                if old_observer_remains && resource == DomResource::SourceRetainedBytes {
                    // Subtraction itself reports the raw policy on underflow;
                    // otherwise the new Draft receives this effective allowance.
                    cap.checked_sub(control_bytes).unwrap_or(cap)
                } else {
                    // Once the old observation is replaced, the Draft's limit is
                    // refreshed to the original policy. Fixed control has no UTF16.
                    cap
                };
            RebaseRefusal {
                error,
                expected_limit,
            }
        })?;
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
fn original_result_transfers_across_rebase_once_without_duplicate_debit_or_credit() {
    for resource in [
        DomResource::SourceRetainedBytes,
        DomResource::SourceRetainedUnits,
    ] {
        // Calibrate only omitted-input canonical formation at its real front;
        // never observe a result-bearing run's required total as the oracle.
        let mut low = 0;
        let mut high = match resource {
            DomResource::SourceRetainedBytes => limits().retained_bytes,
            _ => limits().retained_units,
        };
        while low < high {
            let middle = low + (high - low) / 2;
            match rebase(None, resource, middle) {
                Ok(_) => high = middle,
                Err(refusal) => {
                    is_limit(refusal.error, resource, refusal.expected_limit);
                    low = middle + 1;
                }
            }
        }
        for units in [0, 23] {
            let extra = match resource {
                DomResource::SourceRetainedBytes => {
                    size_of::<Vec<u16>>() + units * size_of::<u16>()
                }
                _ => units,
            };
            let exact = low + extra;
            match rebase(Some(result(units)), resource, exact - 1) {
                Err(refusal) => is_limit(refusal.error, resource, refusal.expected_limit),
                Ok(_) => panic!("one-short result rebase was admitted"),
            }
            let value = result(units);
            let weak = Arc::downgrade(value.shared_code_units());
            let (dom, subject, queue) = rebase(Some(value), resource, exact).unwrap();
            assert_eq!(queue.subject(), subject);
            assert_eq!(
                dom.snapshot()
                    .node(subject)
                    .unwrap()
                    .attribute(&"open".into()),
                Some(&"".into())
            );
            assert!(weak.upgrade().is_some());
            drop(dom);
            assert!(weak.upgrade().is_none());
        }
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
    for resource in [
        DomResource::SourceRetainedBytes,
        DomResource::SourceRetainedUnits,
    ] {
        // Only the equal-shape omitted-parent-input route calibrates the
        // canonical scaffold: actual child/callback publication and restored
        // parent stack capacity are included, but no result-bearing high-water.
        let attempt = |value, cap| {
            let (dom, subject, event) = parent_after_child(value);
            let operation = event.operation();
            rebase_accepted(dom, subject, event, resource, cap).map(|(dom, subject, queue)| {
                assert_eq!(queue.operation(), operation);
                assert_eq!(queue.subject(), subject);
                assert!(matches!(
                    queue.action(),
                    HtmlDialogAction::QueueToggle { .. }
                ));
                assert_eq!(dom.pending_source_edit().unwrap().operation(), operation);
                (dom, subject, queue)
            })
        };
        let mut low = 0;
        let mut high = match resource {
            DomResource::SourceRetainedBytes => limits().retained_bytes,
            _ => limits().retained_units,
        };
        while low < high {
            let middle = low + (high - low) / 2;
            match attempt(None, middle) {
                Ok(_) => high = middle,
                Err(refusal) => {
                    is_limit(refusal.error, resource, refusal.expected_limit);
                    low = middle + 1;
                }
            }
        }
        for units in [0, 17] {
            // One actual original input: Vec descriptor plus exact requested
            // UTF16 capacity, or string code units in their separate domain.
            let extra = match resource {
                DomResource::SourceRetainedBytes => {
                    size_of::<Vec<u16>>() + units * size_of::<u16>()
                }
                _ => units,
            };
            let exact = low + extra;
            let value = result(units);
            let weak = Arc::downgrade(value.shared_code_units());
            let (mut dom, subject, event) = parent_after_child(Some(value));
            match advance_rebase(&mut dom, subject, &event, resource, exact - 1) {
                Err(refusal) => is_limit(refusal.error, resource, refusal.expected_limit),
                Ok(_) => panic!("one-short original parent result rebase admitted"),
            }
            assert_eq!(
                dom.pending_source_edit().unwrap().operation(),
                event.operation()
            );
            let operation = dom.source_operation.as_ref().unwrap();
            let OperationRoot::Edit(
                [
                    Command::CloseDialog {
                        result: Some(retained),
                        ..
                    },
                ],
            ) = &operation.commands
            else {
                panic!("the original parent's result-bearing command remains")
            };
            assert!(std::sync::Weak::ptr_eq(
                &weak,
                &Arc::downgrade(retained.shared_code_units())
            ));
            match &operation.draft {
                OperationDraft::Observed(observation) => {
                    // Formation can refuse before the same accepted receipt
                    // has transferred into the original operation's Draft.
                    assert_eq!(
                        observation.accepted,
                        Some(DomSourceEventOutcome::NotCanceled)
                    );
                    assert!(Arc::ptr_eq(&observation.request.work, &event.work));
                    assert_eq!(
                        operation.status().phase(),
                        DomSourcePhase::EventReceiptAccepted
                    );
                }
                OperationDraft::Segment(draft) => {
                    // Successful rebase is retained even when the subsequent
                    // original admission/checkpoint refuses. This status is
                    // NOT evidence that QueueToggle was already reached.
                    assert!(operation.event_exposed);
                    assert_eq!(operation.status().phase(), DomSourcePhase::DialogService);
                    let frame = draft.dialog_method.as_ref().unwrap();
                    assert_eq!(draft.published_id(frame.subject), Some(subject));
                    assert_eq!(frame.cursor.action(), HtmlDialogAction::ReadOpen);
                    assert!(matches!(frame.invocation, Invocation::Unissued));
                }
            }
            assert!(weak.upgrade().is_some());
            let queue = advance_rebase(&mut dom, subject, &event, resource, exact).unwrap();
            assert_eq!(queue.operation(), event.operation());
            // Reacquiring and advancing the same original operation repeats
            // its actual Queue request, not beforetoggle or callback execution.
            let stamp = dom.snapshot().stamp();
            let repeat = advance_rebase(&mut dom, subject, &event, resource, exact).unwrap();
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
            assert!(
                weak.upgrade().is_some(),
                "same original input survives its real parent rebase"
            );
            drop(dom);
            assert!(weak.upgrade().is_none());
        }
    }
}
