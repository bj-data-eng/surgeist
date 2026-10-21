//! A refused containing-bank replacement retains the acknowledged Close for retry.
use super::*;

const UNRELATED_ATTRIBUTES: usize = 256;
const NAME_BYTES: usize = 512 * 1024;

fn source_policy() -> DomSourceLimits {
    // These independent envelopes must not masquerade as name-local refusal.
    DomSourceLimits::new(256_000_000, 8_000_000, 100_000_000, 64, 4096, 4096, 4)
}

#[test]
fn acknowledged_close_retries_same_attribute_tail_after_raising_name_allowance() {
    let cancel = AtomicBool::new(false);
    let (mut dom, executor, _, subject, callback_subject) = fixture();
    // Select the valid finite policy before any Source observation. No policy
    // is lowered to measured live usage during dispatch or mandatory closing.
    // Disabling history prevents old setup arrays from becoming participants.
    let policy = DomLimits::new(
        128,
        1_000_000,
        256,
        128,
        128,
        0,
        0,
        crate::DomNameLimits::new(NAME_BYTES, 16_384, 100_000_000, 8_000_000).unwrap(),
    )
    .unwrap()
    .with_dispatch_custody(dom.limits().dispatch_custody());
    dom.set_limits(policy).unwrap();
    for index in 0..UNRELATED_ATTRIBUTES {
        // Separate ordinary edits avoid simultaneously staging all intermediate
        // growing banks. Only the final old bank belongs to the Close input.
        let mut setup = DomBatch::new();
        setup.set_attribute(subject, format!("data-kept-{index:03}"), "kept");
        dom.apply(setup, &cancel).unwrap();
    }
    let mut setup = DomBatch::new();
    setup.set_attribute(subject, "open", "");
    dom.apply(setup, &cancel).unwrap();
    assert_eq!(
        dom.snapshot()
            .node(subject)
            .unwrap()
            .as_element()
            .unwrap()
            .attributes()
            .len(),
        UNRELATED_ATTRIBUTES + 1
    );

    let task_source = dom.bind_element_task_source(&cancel).unwrap();
    // Actual fixed task-owner storage, admitted before dispatch. Both slots
    // remain owned through the final fence; no fabricated queue reply is used.
    let task_retention = task_source
        .retain_payload(
            &mut dom,
            size_of::<[Option<crate::DomDialogTask>; 2]>(),
            &cancel,
        )
        .unwrap();
    let mut tasks: [Option<crate::DomDialogTask>; 2] = [None, None];
    let event = {
        let mut lease = dom
            .begin_source_edit(
                DomSourceEdit::close_dialog(subject, None),
                source_policy(),
                &cancel,
            )
            .unwrap();
        let DomSourceProgress::Request(DomSourceRequest::Event(event)) =
            lease.advance(&cancel).unwrap()
        else {
            panic!("Close must expose its real beforetoggle event")
        };
        assert!(!event.cancelable());
        let toggle = event.toggle_event().unwrap();
        assert_eq!(toggle.old_state(), surgeist_html::HtmlDialogState::Open);
        assert_eq!(toggle.new_state(), surgeist_html::HtmlDialogState::Closed);
        event
    };
    let operation = event.operation();
    let permit = executor.begin_dispatch(&dom, &event, &cancel).unwrap();
    let scope = executor.enter_dispatch(&dom, &permit, &cancel).unwrap();
    // Preserve an ordinary callback publication independently of the tokens.
    // It touches a different small bank, not the large closing input bank.
    let mut callback = DomBatch::new();
    callback.set_attribute(callback_subject, "data-callback", "published");
    dom.apply(callback, &cancel).unwrap();
    let mut held = Vec::new();
    let mut binding_refusal = None;
    for index in 0..16_384 {
        let text = DocumentString::from(format!("callback-held-{index:05}"));
        match dom.bind_name(&text, &cancel) {
            Ok(token) => held.push(token),
            Err(error) => {
                binding_refusal = Some(error);
                break;
            }
        }
    }
    assert!(
        !held.is_empty(),
        "callback must retain successful public bindings"
    );
    assert_eq!(
        binding_refusal,
        Some(DomError::ResourceLimit {
            resource: DomResource::NameRetainedBytes,
            limit: NAME_BYTES,
        }),
        "the callback exhausts only the preselected name byte allowance"
    );
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
    assert!(permit.is_none());
    let (closing, work_before_closing) = {
        let mut lease = dom
            .resume_source_edit(operation, source_policy(), &cancel)
            .unwrap();
        lease
            .accept_source_event(&event, &mut publication, &cancel)
            .unwrap();
        assert!(publication.is_none());
        let DomSourceProgress::Request(DomSourceRequest::Dialog(queue)) =
            lease.advance(&cancel).unwrap()
        else {
            panic!("the authentic beforetoggle receipt must reach QueueToggle")
        };
        assert!(matches!(
            queue.action(),
            HtmlDialogAction::QueueToggle { .. }
        ));
        let plan = lease
            .dialog_task_queue_plan(&queue, &task_source, 1, &cancel)
            .unwrap();
        assert_eq!(plan.replacing(), None);
        let prepared = lease
            .prepare_dialog_task_queue(&queue, &task_source, plan, &cancel)
            .expect("the real tracker publication precedes mandatory RemoveOpen");
        let (task, queued) = prepared.publish();
        tasks[0] = Some(task); // fixed actual queue: no allocation or callback
        let mut queued = Some(queued);
        lease
            .accept_dialog_task_queue(&queue, &mut queued, &cancel)
            .unwrap();
        assert!(queued.is_none());
        let work = lease.status().spent_work();
        (lease.advance(&cancel), work)
    };

    // A replacement must at least own its concrete target and surviving array.
    // This independent lower bound excludes child-custody and occurrence costs;
    // it is not an observed admission maximum or an exact receipt oracle.
    let replacement_minimum =
        size_of::<crate::store::payload::Payload<crate::name::AttributeValues>>()
            + UNRELATED_ATTRIBUTES * size_of::<crate::DomAttribute>();
    eprintln!(
        "Close bindings={}, name_bytes={}, mandatory_array_minimum={replacement_minimum}",
        held.len(),
        dom.names.inspect_usage().bytes(),
    );
    assert!(
        matches!(
            closing,
            Err(DomError::ResourceLimit {
                resource: DomResource::NameRetainedBytes,
                limit: NAME_BYTES,
            })
        ),
        "the fresh containing bank must report its exact finite refusal: {closing:?}"
    );
    let status = dom.pending_source_edit().expect("mandatory tail retained");
    assert_eq!(status.operation(), operation);
    assert_eq!(status.phase(), DomSourcePhase::DialogService);
    assert!(status.has_published());
    assert!(status.spent_work() > work_before_closing);
    assert_eq!(
        dom.snapshot().incomplete_source_operation(),
        Some(operation)
    );
    let retained = dom.source_operation.as_ref().unwrap();
    assert!(retained.headroom.as_ref().unwrap().remaining_phases > 0);
    let Some(crate::store::method::Frame::Dialog(frame)) = retained.draft.method.as_ref() else {
        panic!("the original Close cursor must survive refusal")
    };
    assert_eq!(frame.cursor.action(), HtmlDialogAction::RemoveOpen);
    assert_eq!(
        dom.snapshot()
            .node(subject)
            .unwrap()
            .attribute(&"open".into()),
        Some(&"".into())
    );
    assert_eq!(tasks[0].as_ref().unwrap().info().id().target(), subject);
    assert!(tasks[1].is_none());
    let published_phases = status.published_phases();
    let spent_work = status.spent_work();

    // A fresh containing bank follows the existing raised-budget Source retry
    // contract. Fixed doubling is selected independently of measured receipts;
    // atom/index reservations and already published queue work remain intact.
    let raised = DomLimits::new(
        128,
        1_000_000,
        256,
        128,
        128,
        0,
        0,
        crate::DomNameLimits::new(NAME_BYTES * 2, 16_384, 100_000_000, 8_000_000).unwrap(),
    )
    .unwrap()
    .with_dispatch_custody(dom.limits().dispatch_custody());
    dom.set_limits(raised).unwrap();
    let after_raise = dom.pending_source_edit().unwrap();
    assert_eq!(after_raise.operation(), operation);
    assert!(after_raise.spent_work() >= spent_work);
    let retry_work = after_raise.spent_work();
    assert_eq!(after_raise.published_phases(), published_phases);
    {
        let mut lease = dom
            .resume_source_edit(operation, source_policy(), &cancel)
            .unwrap();
        assert!(lease.status().spent_work() >= retry_work);
        let DomSourceProgress::Request(DomSourceRequest::Dialog(close)) =
            lease.advance(&cancel).unwrap()
        else {
            panic!("original RemoveOpen must resume directly to QueueClose")
        };
        // Repeated beforetoggle or QueueToggle would be a protocol regression.
        assert_eq!(close.operation(), operation);
        assert_eq!(close.subject(), subject);
        assert_eq!(close.action(), HtmlDialogAction::QueueClose);
        assert_eq!(lease.status().published_phases(), published_phases);
        assert!(lease.status().spent_work() > retry_work);
        let plan = lease
            .dialog_task_queue_plan(&close, &task_source, 2, &cancel)
            .unwrap();
        assert_eq!(plan.replacing(), None);
        let prepared = lease
            .prepare_dialog_task_queue(&close, &task_source, plan, &cancel)
            .unwrap();
        let (task, queued) = prepared.publish();
        tasks[1] = Some(task);
        let mut queued = Some(queued);
        lease
            .accept_dialog_task_queue(&close, &mut queued, &cancel)
            .unwrap();
        assert!(queued.is_none());
        assert!(matches!(
            lease.advance(&cancel),
            Ok(DomSourceProgress::Complete(_))
        ));
    }
    for (index, token) in held.iter().enumerate() {
        assert_eq!(token.owner(), dom.owner());
        assert_eq!(
            token.text(),
            &DocumentString::from(format!("callback-held-{index:05}"))
        );
    }
    let snapshot = dom.snapshot();
    assert_eq!(
        snapshot
            .node(callback_subject)
            .unwrap()
            .attribute(&"data-callback".into()),
        Some(&"published".into())
    );
    for index in 0..UNRELATED_ATTRIBUTES {
        assert_eq!(
            snapshot
                .node(subject)
                .unwrap()
                .attribute(&format!("data-kept-{index:03}").into()),
            Some(&"kept".into())
        );
    }
    drop(tasks);
    drop(task_retention);
    assert!(dom.pending_source_edit().is_none());
    assert_eq!(snapshot.incomplete_source_operation(), None);
    assert_eq!(
        snapshot.node(subject).unwrap().attribute(&"open".into()),
        None
    );
}

// The Source companion consumes actual task-owner buffers, rather than lowering
// a policy to an observed ledger total or calibrating an admission threshold.
const SOURCE_ATTRIBUTES: usize = 1024;
const SOURCE_BYTES: usize = 8 * 1024 * 1024;
const SOURCE_BANK_MINIMUM: usize = size_of::<
    crate::store::payload::Payload<crate::name::AttributeValues>,
>() + SOURCE_ATTRIBUTES * size_of::<crate::DomAttribute>();
const TASK_SLAB_BYTES: usize = SOURCE_BANK_MINIMUM / 2;

struct TaskSlab {
    bytes: Box<[u8; TASK_SLAB_BYTES]>,
    _retained: crate::DomElementTaskRetention,
}

fn finite_source_policy(bytes: usize) -> DomSourceLimits {
    DomSourceLimits::new(bytes, 8_000_000, 100_000_000, 64, 4096, 4096, 4)
}

#[test]
fn acknowledged_close_retries_attribute_formation_after_source_storage_refusal() {
    let cancel = AtomicBool::new(false);
    let (mut dom, executor, _, subject, callback_subject) = fixture();
    let names = crate::DomNameLimits::new(16_777_216, 16_384, 100_000_000, 8_000_000).unwrap();
    let owner_policy = DomLimits::new(128, 1_000_000, 256, 128, 128, 0, 0, names)
        .unwrap()
        .with_dispatch_custody(dom.limits().dispatch_custody());
    dom.set_limits(owner_policy).unwrap();
    for index in 0..SOURCE_ATTRIBUTES {
        let mut setup = DomBatch::new();
        setup.set_attribute(subject, format!("data-source-kept-{index:04}"), "kept");
        dom.apply(setup, &cancel).unwrap();
    }
    let mut setup = DomBatch::new();
    setup.set_attribute(subject, "open", "");
    dom.apply(setup, &cancel).unwrap();

    {
        let snapshot = dom.snapshot();
        let element = snapshot.node(subject).unwrap().as_element().unwrap();
        let open = element
            .attributes()
            .iter()
            .find(|attribute| attribute.name().local_name() == &DocumentString::from("open"))
            .unwrap();
        assert_eq!(open.name().namespace_uri(), None);
        assert_eq!(open.name().prefix(), None);
        assert_eq!(open.name().local_name().len(), 4);
        assert!(open.value().is_empty());
        assert_eq!(element.nonce(), Some(&DocumentString::default()));
    }

    // A fresh Source segment first allocates a checkpoint, one mark and one
    // method inverse. The independent empty deque measures its actual requested
    // one-row capacity, including the large inline Undo variants. The mark is
    // bounded conservatively by its larger owning JournalMark handle. Four
    // short-string occurrences cover the command, normalization and retained
    // removal-name/empty-value locals before attribute_input. No node record
    // materializes until publish_element, after that array has been formed.
    let mut raw_inverse = std::collections::VecDeque::<crate::store::transaction::Undo>::new();
    raw_inverse.try_reserve_exact(1).unwrap();
    let prefix_bound = size_of::<crate::store::source::checkpoint::Checkpoint>()
        + size_of::<crate::store::transaction::JournalMark>()
        + raw_inverse.capacity() * size_of::<crate::store::transaction::Undo>()
        + 4 * (size_of::<Vec<u16>>() + 4 * size_of::<u16>());
    assert!(
        prefix_bound + size_of::<crate::store::payload::Payload<crate::name::AttributeValues>>()
            <= TASK_SLAB_BYTES,
        "raw checkpoint/cursor/string prefix must fit the independently sized slab"
    );
    drop(raw_inverse);

    let task_source = dom.bind_element_task_source(&cancel).unwrap();
    let slots = SOURCE_BYTES / TASK_SLAB_BYTES + 1;
    let mut outer_retention = task_source
        .retain_payload(
            &mut dom,
            size_of::<Vec<TaskSlab>>()
                + slots * size_of::<TaskSlab>()
                + size_of::<[Option<crate::DomDialogTask>; 2]>(),
            &cancel,
        )
        .unwrap();
    let mut slabs = Vec::<TaskSlab>::new();
    slabs.try_reserve_exact(slots).unwrap();
    let excess = (slabs.capacity() - slots) * size_of::<TaskSlab>();
    if excess != 0 {
        outer_retention
            .absorb(
                task_source
                    .retain_payload(&mut dom, excess, &cancel)
                    .unwrap(),
            )
            .unwrap();
    }
    let mut tasks: [Option<crate::DomDialogTask>; 2] = [None, None];
    let policy = finite_source_policy(SOURCE_BYTES);
    let event = {
        let mut lease = dom
            .begin_source_edit(DomSourceEdit::close_dialog(subject, None), policy, &cancel)
            .unwrap();
        let DomSourceProgress::Request(DomSourceRequest::Event(event)) =
            lease.advance(&cancel).unwrap()
        else {
            panic!("actual Close beforetoggle")
        };
        assert!(!event.cancelable());
        event
    };
    let operation = event.operation();
    let permit = executor.begin_dispatch(&dom, &event, &cancel).unwrap();
    let scope = executor.enter_dispatch(&dom, &permit, &cancel).unwrap();
    let held = (0..8)
        .map(|index| {
            dom.bind_name(&format!("source-callback-held-{index}").into(), &cancel)
                .unwrap()
        })
        .collect::<Vec<_>>();
    let mut callback = DomBatch::new();
    callback.set_attribute(callback_subject, "data-callback", "source-published");
    dom.apply(callback, &cancel).unwrap();
    drop(scope);
    let mut permit = Some(permit);
    let mut event_publication = Some(
        executor
            .seal_completed(
                &dom,
                &mut permit,
                DomSourceEventOutcome::NotCanceled,
                &cancel,
            )
            .unwrap(),
    );
    assert!(permit.is_none());
    let (refusal, work_before) = {
        let mut lease = dom.resume_source_edit(operation, policy, &cancel).unwrap();
        lease
            .accept_source_event(&event, &mut event_publication, &cancel)
            .unwrap();
        assert!(event_publication.is_none());
        let DomSourceProgress::Request(DomSourceRequest::Dialog(toggle)) =
            lease.advance(&cancel).unwrap()
        else {
            panic!("authenticated event reaches actual QueueToggle")
        };
        assert!(matches!(
            toggle.action(),
            HtmlDialogAction::QueueToggle { .. }
        ));
        let plan = lease
            .dialog_task_queue_plan(&toggle, &task_source, 1, &cancel)
            .unwrap();
        assert_eq!(plan.replacing(), None);
        let prepared = lease
            .prepare_dialog_task_queue(&toggle, &task_source, plan, &cancel)
            .unwrap();
        let (task, publication) = prepared.publish();
        tasks[0] = Some(task);
        let mut publication = Some(publication);
        lease
            .accept_dialog_task_queue(&toggle, &mut publication, &cancel)
            .unwrap();
        assert!(publication.is_none());

        let mut saturated = false;
        for _ in 0..slots {
            match lease.retain_element_task_payload(&task_source, TASK_SLAB_BYTES, &cancel) {
                Ok(retained) => slabs.push(TaskSlab {
                    bytes: Box::new([0x5a; TASK_SLAB_BYTES]),
                    _retained: retained,
                }),
                Err(error) => {
                    assert_eq!(
                        error,
                        DomError::ResourceLimit {
                            resource: DomResource::SourceRetainedBytes,
                            limit: SOURCE_BYTES,
                        }
                    );
                    saturated = true;
                    break;
                }
            }
        }
        assert!(
            saturated,
            "actual task-owner storage must reach the fixed Source bound"
        );
        // retain_payload only clones its existing binding handle and increments
        // the declared byte extent; it allocates no additional metadata. Its
        // exact failed Source admission therefore leaves less than one slab.
        // Release one actual buffer, data before its original permission. Free
        // room is now >=slab and <2*slab <= the mandatory new bank minimum.
        let released = slabs.pop().expect("at least one actual buffer admitted");
        drop(released);
        // Notify the existing owner admission front of the reduced actual task
        // extent, refreshing the retained segment's cached external envelope.
        drop(
            lease
                .retain_element_task_payload(&task_source, 0, &cancel)
                .unwrap(),
        );
        let work = lease.status().spent_work();
        (lease.advance(&cancel), work)
    };
    assert!(
        matches!(
            refusal,
            Err(DomError::ResourceLimit {
                resource: DomResource::SourceRetainedBytes,
                limit: SOURCE_BYTES,
            })
        ),
        "mandatory array formation reports the original finite Source bound: {refusal:?}"
    );
    let status = dom.pending_source_edit().unwrap();
    assert_eq!(status.operation(), operation);
    assert_eq!(status.phase(), DomSourcePhase::DialogService);
    assert!(status.has_published());
    assert!(status.spent_work() > work_before);
    assert_eq!(
        dom.snapshot().incomplete_source_operation(),
        Some(operation)
    );
    let retained = dom.source_operation.as_ref().unwrap();
    let Some(crate::store::method::Frame::Dialog(frame)) = retained.draft.method.as_ref() else {
        panic!("the exact acknowledged tail survives")
    };
    assert_eq!(frame.cursor.action(), HtmlDialogAction::RemoveOpen);
    assert!(retained.headroom.as_ref().unwrap().remaining_phases > 0);
    let original_work = retained.draft.source_work.source_arc().clone();
    let phases = status.published_phases();
    let spent = status.spent_work();
    assert_eq!(dom.limits().names(), names);
    assert_eq!(
        dom.snapshot()
            .node(subject)
            .unwrap()
            .attribute(&"open".into()),
        Some(&"".into())
    );
    assert!(tasks[0].is_some());
    assert!(tasks[1].is_none());
    for slab in &slabs {
        assert_eq!(slab.bytes.first(), Some(&0x5a));
        assert_eq!(slab.bytes.last(), Some(&0x5a));
    }

    // Raise only Source storage by an independently chosen fixed doubling.
    // All successful task buffers, name tokens and canonical effects survive.
    {
        let mut lease = dom
            .resume_source_edit(operation, finite_source_policy(SOURCE_BYTES * 2), &cancel)
            .unwrap();
        assert!(lease.status().spent_work() >= spent);
        assert_eq!(lease.status().published_phases(), phases);
        assert!(Arc::ptr_eq(
            &original_work,
            lease
                .dom
                .source_operation
                .as_ref()
                .unwrap()
                .draft
                .source_work
                .source_arc(),
        ));
        let DomSourceProgress::Request(DomSourceRequest::Dialog(close)) =
            lease.advance(&cancel).unwrap()
        else {
            panic!("retry must continue RemoveOpen directly to QueueClose")
        };
        assert_eq!(close.operation(), operation);
        assert_eq!(close.subject(), subject);
        assert_eq!(close.action(), HtmlDialogAction::QueueClose);
        assert_eq!(lease.status().published_phases(), phases);
        let plan = lease
            .dialog_task_queue_plan(&close, &task_source, 2, &cancel)
            .unwrap();
        assert_eq!(plan.replacing(), None);
        let prepared = lease
            .prepare_dialog_task_queue(&close, &task_source, plan, &cancel)
            .unwrap();
        let (task, publication) = prepared.publish();
        tasks[1] = Some(task);
        let mut publication = Some(publication);
        lease
            .accept_dialog_task_queue(&close, &mut publication, &cancel)
            .unwrap();
        assert!(publication.is_none());
        assert!(matches!(
            lease.advance(&cancel),
            Ok(DomSourceProgress::Complete(_))
        ));
    }
    assert!(original_work.load(Ordering::Relaxed) > spent);
    assert!(dom.pending_source_edit().is_none());
    assert_eq!(dom.limits().names(), names);
    let snapshot = dom.snapshot();
    assert_eq!(snapshot.incomplete_source_operation(), None);
    assert_eq!(
        snapshot.node(subject).unwrap().attribute(&"open".into()),
        None
    );
    assert_eq!(
        snapshot
            .node(callback_subject)
            .unwrap()
            .attribute(&"data-callback".into()),
        Some(&"source-published".into())
    );
    for index in 0..SOURCE_ATTRIBUTES {
        assert_eq!(
            snapshot
                .node(subject)
                .unwrap()
                .attribute(&format!("data-source-kept-{index:04}").into()),
            Some(&"kept".into())
        );
    }
    for (index, token) in held.iter().enumerate() {
        assert_eq!(
            token.text(),
            &format!("source-callback-held-{index}").into()
        );
        assert!(
            token
                .same_identity(&dom.bind_name(token.text(), &cancel).unwrap())
                .unwrap()
        );
    }
    drop(slabs); // actual buffers before their individual retained permissions
    drop(tasks);
    drop(outer_retention);
}
