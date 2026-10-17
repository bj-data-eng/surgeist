//! Actual original Show queue publication; none of these tests finishes its tail.
use super::*;
use crate::{DomSourceEventServiceLimits, DomSourceEventServiceProgress};
use surgeist_dom::{
    DomSourceDialogRequest, DomSourceEdit, DomSourceLimits, DomSourceProgress, DomSourceRequest,
};

fn source_limits() -> DomSourceLimits {
    DomSourceLimits::new(16_000_000, 16_000_000, 4_000_000, 64, 65_536, 65_536, 16)
}
fn queue_request(prior: bool) -> (DomSession, NodeId, DomSourceDialogRequest) {
    let (mut session, document, details, _) = fixture();
    if prior {
        open(&mut session, details);
        session
            .dom_manipulation_execution()
            .ingest(&mut session.dom(), &cancel())
            .unwrap();
    }
    let mut batch = DomBatch::new();
    let dialog = batch.create_element_ns(document, Namespace::html(), "dialog");
    let subject = apply(&mut session, batch)
        .created(&dialog)
        .unwrap()
        .unwrap();
    let source = session.source_execution();
    // This fixture really dispatches one Source event; its original eventless
    // profile is raised explicitly through the same owner, without replacement.
    source
        .raise_event_runtime_limits(
            &session.dom(),
            crate::DomSourceEventRuntimeLimits {
                jobs: 8,
                storage_bytes: 16_000_000,
                work: 1_000_000,
            },
            &cancel(),
        )
        .unwrap();
    let event = {
        let mut view = session.dom();
        let mut lease = view
            .begin_source_edit(
                DomSourceEdit::show_dialog(subject),
                source_limits(),
                &cancel(),
            )
            .unwrap();
        match lease.advance(&cancel()).unwrap() {
            DomSourceProgress::Request(DomSourceRequest::Event(request)) => request,
            other => panic!("actual initial Show event required: {other:?}"),
        }
    };
    let mut dispatcher = DomEventDispatcher::<()>::new(DomEventRegistryLimits {
        registrations: 0,
        pending_events: 1,
        nested_dispatches: 1,
    })
    .unwrap();
    match source
        .service_event(
            &mut session.dom(),
            &event,
            &mut dispatcher,
            &mut (),
            DomSourceEventServiceLimits {
                dispatch: dispatch_limits(),
                source: source_limits(),
            },
            &cancel(),
        )
        .unwrap()
    {
        DomSourceEventServiceProgress::Serviced { receipt } => {
            assert_eq!(receipt.callbacks(), 0);
            assert!(!receipt.default_prevented());
        }
        other => panic!("actual event acceptance required: {other:?}"),
    }
    let request = {
        let mut view = session.dom();
        let mut lease = view
            .resume_source_edit(event.operation(), source_limits(), &cancel())
            .unwrap();
        match lease.advance(&cancel()).unwrap() {
            DomSourceProgress::Request(DomSourceRequest::Dialog(request))
                if matches!(
                    request.action(),
                    surgeist_html::HtmlDialogAction::QueueToggle { .. }
                ) =>
            {
                request
            }
            other => panic!("actual original QueueToggle required: {other:?}"),
        }
    };
    (session, subject, request)
}
fn candidate_peak(prior: bool) -> usize {
    // A genuinely installed Details bank, when present, coexists with both
    // candidate banks, complete Intake header and final output backing.
    let old = usize::from(prior);
    let new = old + 1;
    size_of::<RefCell<ManipulationState>>()
        + size_of::<Cell<usize>>()
        + size_of::<surgeist_dom::DomManipulationTaskSource>()
        + surgeist_dom::DomManipulationTaskSource::binding_payload_bytes()
        + size_of::<Intake>()
        + old * (size_of::<Task>() + size_of::<Tracker>())
        + new * (size_of::<Plan>() + size_of::<Tracker>() + size_of::<Task>())
}
#[test]
fn queue_runtime_exact_candidate_peak_succeeds_and_one_short_has_no_effect() {
    for prior in [false, true] {
        for short in [true, false] {
            let (mut session, subject, request) = queue_request(prior);
            let mut runtime = session.dom_manipulation_execution();
            let owner = runtime.state().unwrap();
            let old = usize::from(prior);
            assert_eq!(owner.borrow().tasks.capacity(), old);
            assert_eq!(owner.borrow().trackers.capacity(), old);
            let original = owner.borrow().tasks.first().map(|task| task.value.id());
            runtime
                .set_limits(DomManipulationRuntimeLimits {
                    tasks: 1 + old,
                    storage_bytes: candidate_peak(prior) - usize::from(short),
                    work: 1_000_000,
                })
                .unwrap();
            let before = session.dom().snapshot();
            let mut view = session.dom();
            let mut lease = view
                .resume_source_edit(request.operation(), source_limits(), &cancel())
                .unwrap();
            let result = runtime.queue_dialog(&mut lease, &request, &cancel());
            if short {
                assert!(
                    matches!(result, Err(DomManipulationRuntimeError::Admission)),
                    "one-short must refuse original candidate: {result:?}"
                );
                assert_eq!(lease.snapshot().stamp(), before.stamp());
                assert!(
                    !lease
                        .snapshot()
                        .node(subject)
                        .unwrap()
                        .as_dialog()
                        .unwrap()
                        .has_toggle_task_tracker()
                );
                assert_eq!(owner.borrow().tasks.len(), old);
                assert_eq!(
                    owner.borrow().tasks.first().map(|task| task.value.id()),
                    original
                );
            } else {
                result.unwrap();
                let owner = owner.borrow();
                assert_eq!(owner.tasks.capacity(), old + 1);
                assert_eq!(owner.trackers.capacity(), old + 1);
                assert_eq!(owner.tasks.len(), old + 1);
                assert!(owner.tasks[old].publication.is_none());
                let snapshot = lease.snapshot();
                assert_eq!(
                    snapshot
                        .node(subject)
                        .unwrap()
                        .as_dialog()
                        .unwrap()
                        .toggle_task_tracker()
                        .unwrap()
                        .task(),
                    owner.tasks[old].value.id()
                );
                assert!(
                    matches!(lease.advance(&cancel()).unwrap(), DomSourceProgress::Request(DomSourceRequest::Dialog(ref next)) if next.action() == surgeist_html::HtmlDialogAction::AddOpen)
                );
            }
        }
    }
}

#[test]
fn queue_cancellation_precedes_effect_and_published_proof_survives_refused_acceptance() {
    let (mut session, subject, request) = queue_request(false);
    let mut runtime = session.dom_manipulation_execution();
    let owner = runtime.state().unwrap();
    let mut view = session.dom();
    let mut lease = view
        .resume_source_edit(request.operation(), source_limits(), &cancel())
        .unwrap();
    let before = lease.snapshot().stamp();
    let stopped = AtomicBool::new(true);
    assert!(matches!(
        runtime.queue_dialog(&mut lease, &request, &stopped),
        Err(DomManipulationRuntimeError::Dom(DomError::Cancelled))
    ));
    assert_eq!(lease.snapshot().stamp(), before);
    assert!(owner.borrow().tasks.is_empty());
    runtime
        .set_limits(DomManipulationRuntimeLimits {
            tasks: 0,
            storage_bytes: 65_536,
            work: 1_000_000,
        })
        .unwrap();
    assert!(matches!(
        runtime.queue_dialog(&mut lease, &request, &cancel()),
        Err(DomManipulationRuntimeError::Admission)
    ));
    assert_eq!(lease.snapshot().stamp(), before);
    assert!(owner.borrow().tasks.is_empty());
    runtime.set_limits(limits()).unwrap();
    // This is the same real production publication phase called by queue_dialog.
    let index = owner
        .borrow_mut()
        .publish_dialog_queue(&mut lease, &request, &cancel())
        .unwrap();
    let id = owner.borrow().tasks[index].value.id();
    let published = lease.snapshot().stamp();
    assert_ne!(published, before);
    assert_eq!(
        lease
            .snapshot()
            .node(subject)
            .unwrap()
            .as_dialog()
            .unwrap()
            .toggle_task_tracker()
            .unwrap()
            .task(),
        id
    );
    {
        let mut owner = owner.borrow_mut();
        assert!(matches!(
            lease.accept_dialog_toggle_queue(
                &request,
                &mut owner.tasks[index].publication,
                &stopped
            ),
            Err(DomError::Cancelled)
        ));
        assert!(owner.tasks[index].publication.is_some());
    }
    runtime
        .queue_dialog(&mut lease, &request, &cancel())
        .unwrap();
    assert_eq!(lease.snapshot().stamp(), published);
    assert_eq!(owner.borrow().tasks.len(), 1);
    assert_eq!(owner.borrow().tasks[0].value.id(), id);
    assert!(owner.borrow().tasks[0].publication.is_none());
    assert!(
        matches!(lease.advance(&cancel()).unwrap(), DomSourceProgress::Request(DomSourceRequest::Dialog(ref next)) if next.action() == surgeist_html::HtmlDialogAction::AddOpen)
    );
}

struct HeldObservations {
    owner: DomManipulationRuntime,
    event: Option<NativeDomEvent>,
    path: Option<crate::DomEventComposedPath>,
}
impl DomManipulationEventContext for HeldObservations {
    fn dom_manipulation(&mut self) -> &mut DomManipulationRuntime {
        &mut self.owner
    }
}
#[test]
fn actual_details_observations_outlive_row_and_receipt_output_transfers_its_backing() {
    let (mut session, _, target, _) = fixture();
    open(&mut session, target);
    let mut owner = session.dom_manipulation_execution();
    owner.ingest(&mut session.dom(), &cancel()).unwrap();
    let state = owner.state().unwrap();
    assert_eq!(state.borrow().tasks.capacity(), 1);
    assert_eq!(state.borrow().trackers.capacity(), 1);
    let mut context = HeldObservations {
        owner,
        event: None,
        path: None,
    };
    let mut dispatcher = DomEventDispatcher::new(DomEventRegistryLimits {
        registrations: 1,
        pending_events: 1,
        nested_dispatches: 1,
    })
    .unwrap();
    dispatcher
        .add_listener(
            &session.dom(),
            target,
            DomEventType::Toggle,
            DomEventCallback::new(|_, _, context: &mut HeldObservations, event| {
                context.event = Some(event.clone());
                context.path = Some(event.composed_path());
                Err(DomEventCallbackError { code: 91 })
            }),
            DomEventListenerOptions {
                capture: false,
                once: false,
            },
        )
        .unwrap();
    let receipt = match DomManipulationRuntime::run_next(
        &mut session.dom(),
        &mut dispatcher,
        &mut context,
        crate::DomManipulationRunLimits {
            dispatch: dispatch_limits(),
            source: source_limits(),
        },
        &cancel(),
    )
    .unwrap()
    {
        DomManipulationProgress::Complete { receipt, .. } => receipt,
        other => panic!("actual Details delivery required: {other:?}"),
    };
    assert_eq!(receipt.callbacks(), 1);
    assert_eq!(receipt.failures().len(), 1);
    assert!(matches!(
        receipt.failures()[0].kind,
        crate::DomEventListenerFailureKind::Reported(DomEventCallbackError { code: 91 })
    ));
    assert_eq!(context.owner.pending_tasks().unwrap(), 0);
    let (event_bytes, path_bytes) = crate::dom_event::manipulation_observation_layout(
        context.event.as_ref().unwrap(),
        context.path.as_ref().unwrap(),
    );
    assert!(event_bytes > 0 && path_bytes > 0);
    assert_eq!(state.borrow().payload_bytes.get(), event_bytes + path_bytes);
    // Spare bank capacity is still physically owned. Failure backing already
    // belongs to this caller's real returned receipt and is absent here.
    let installed = candidate_peak(false) - size_of::<Intake>() - size_of::<Plan>();
    let exact = installed + event_bytes + path_bytes;
    assert!(matches!(
        context.owner.set_limits(DomManipulationRuntimeLimits {
            storage_bytes: exact - 1,
            ..limits()
        }),
        Err(DomManipulationRuntimeError::Admission)
    ));
    context
        .owner
        .set_limits(DomManipulationRuntimeLimits {
            storage_bytes: exact,
            ..limits()
        })
        .unwrap();
    drop(context.event.take());
    assert_eq!(state.borrow().payload_bytes.get(), path_bytes);
    context
        .owner
        .set_limits(DomManipulationRuntimeLimits {
            storage_bytes: installed + path_bytes,
            ..limits()
        })
        .unwrap();
    drop(context.path.take());
    assert_eq!(state.borrow().payload_bytes.get(), 0);
    context
        .owner
        .set_limits(DomManipulationRuntimeLimits {
            storage_bytes: installed,
            ..limits()
        })
        .unwrap();
    assert_eq!(receipt.failures().len(), 1);
    assert!(matches!(
        context.owner.set_limits(DomManipulationRuntimeLimits {
            storage_bytes: installed - 1,
            ..limits()
        }),
        Err(DomManipulationRuntimeError::Admission)
    ));
}
