//! Actual original Show queue publication; none of these tests finishes its tail.
use super::*;
use crate::{DomSourceEventServiceLimits, DomSourceEventServiceProgress};
use surgeist_dom::{
    DomSourceDialogRequest, DomSourceEdit, DomSourceLimits, DomSourceProgress, DomSourceRequest,
};

mod element_started;

fn source_limits() -> DomSourceLimits {
    DomSourceLimits::new(16_000_000, 16_000_000, 4_000_000, 64, 65_536, 65_536, 16)
}
fn queue_request(prior: bool) -> (DomSession, NodeId, DomSourceDialogRequest) {
    let (mut session, document, details, _) = fixture();
    if prior {
        open(&mut session, details);
        session
            .element_task_execution()
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
                    surgeist_html::elements::dialog::HtmlDialogAction::QueueToggle { .. }
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
    // An installed Details bank, when present, coexists with both
    // candidate banks, complete Intake header and final output backing.
    let old = usize::from(prior);
    let new = old + 1;
    size_of::<RefCell<ElementTaskState>>()
        + size_of::<TaskPayloadCounterTarget>()
        + size_of::<surgeist_dom::DomElementTaskSource>()
        + surgeist_dom::DomElementTaskSource::binding_payload_bytes()
        + size_of::<Intake>()
        + old * (size_of::<Task>() + size_of::<Tracker>())
        + new * (size_of::<Plan>() + size_of::<Tracker>() + size_of::<Task>())
}
#[test]
fn queue_runtime_exact_candidate_peak_succeeds_and_one_short_has_no_effect() {
    for prior in [false, true] {
        for short in [true, false] {
            let (mut session, subject, request) = queue_request(prior);
            let mut runtime = session.element_task_execution();
            let owner = runtime.state().unwrap();
            let old = usize::from(prior);
            assert_eq!(owner.borrow().tasks.capacity(), old);
            assert_eq!(owner.borrow().trackers.capacity(), old);
            let original = owner.borrow().tasks.first().map(|task| task.value.id());
            runtime
                .set_limits(DomElementTaskRuntimeLimits {
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
                    matches!(result, Err(DomElementTaskRuntimeError::Admission)),
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
                    matches!(lease.advance(&cancel()).unwrap(), DomSourceProgress::Request(DomSourceRequest::Dialog(ref next)) if next.action() == surgeist_html::elements::dialog::HtmlDialogAction::RememberFocusedElement)
                );
                assert_eq!(
                    lease
                        .snapshot()
                        .node(subject)
                        .unwrap()
                        .attribute(&"open".into()),
                    Some(&"".into())
                );
            }
        }
    }
}

#[test]
fn queue_cancellation_precedes_effect_and_published_proof_survives_refused_acceptance() {
    let (mut session, subject, request) = queue_request(false);
    let mut runtime = session.element_task_execution();
    let owner = runtime.state().unwrap();
    let mut view = session.dom();
    let mut lease = view
        .resume_source_edit(request.operation(), source_limits(), &cancel())
        .unwrap();
    let before = lease.snapshot().stamp();
    let stopped = AtomicBool::new(true);
    assert!(matches!(
        runtime.queue_dialog(&mut lease, &request, &stopped),
        Err(DomElementTaskRuntimeError::Dom(DomError::Cancelled))
    ));
    assert_eq!(lease.snapshot().stamp(), before);
    assert!(owner.borrow().tasks.is_empty());
    runtime
        .set_limits(DomElementTaskRuntimeLimits {
            tasks: 0,
            storage_bytes: 65_536,
            work: 1_000_000,
        })
        .unwrap();
    assert!(matches!(
        runtime.queue_dialog(&mut lease, &request, &cancel()),
        Err(DomElementTaskRuntimeError::Admission)
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
            lease.accept_dialog_task_queue(&request, &mut owner.tasks[index].publication, &stopped),
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
        matches!(lease.advance(&cancel()).unwrap(), DomSourceProgress::Request(DomSourceRequest::Dialog(ref next)) if next.action() == surgeist_html::elements::dialog::HtmlDialogAction::RememberFocusedElement)
    );
    assert_eq!(
        lease
            .snapshot()
            .node(subject)
            .unwrap()
            .attribute(&"open".into()),
        Some(&"".into())
    );
}

struct HeldObservations {
    owner: DomElementTaskRuntime,
    event: Option<NativeDomEvent>,
    path: Option<crate::DomEventComposedPath>,
}
impl DomElementTaskEventContext for HeldObservations {
    fn element_tasks(&mut self) -> &mut DomElementTaskRuntime {
        &mut self.owner
    }
}
#[test]
fn actual_details_observations_outlive_row_and_receipt_output_transfers_its_backing() {
    let (mut session, _, target, _) = fixture();
    open(&mut session, target);
    let mut owner = session.element_task_execution();
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
            &mut session.dom(),
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
    let receipt = match DomElementTaskRuntime::run_next(
        &mut session.dom(),
        surgeist_dom::DomElementTaskSourceKind::DomManipulation,
        &mut dispatcher,
        &mut context,
        crate::DomElementTaskRunLimits {
            dispatch: dispatch_limits(),
            source: source_limits(),
        },
        &cancel(),
    )
    .unwrap()
    {
        DomElementTaskProgress::Complete { receipt, .. } => receipt,
        other => panic!("actual Details delivery required: {other:?}"),
    };
    assert_eq!(receipt.callbacks(), 1);
    assert_eq!(receipt.failures().len(), 1);
    assert!(matches!(
        receipt.failures()[0].kind,
        crate::DomEventListenerFailureKind::Reported(DomEventCallbackError { code: 91 })
    ));
    assert_eq!(
        context
            .owner
            .pending_tasks(surgeist_dom::DomElementTaskSourceKind::DomManipulation)
            .unwrap(),
        0
    );
    let (event_bytes, path_bytes) = crate::dom_event::manipulation_observation_layout(
        context.event.as_ref().unwrap(),
        context.path.as_ref().unwrap(),
    );
    assert!(event_bytes > 0 && path_bytes > 0);
    let (continuation_bytes, _) = dispatcher.inspect_retired_continuation_storage();
    assert!(continuation_bytes > 0);
    let counter_body = std::mem::size_of_val(state.borrow().payload_bytes.as_ref());
    assert_eq!(
        state.borrow().payload_bytes.get(),
        counter_body + event_bytes + path_bytes + continuation_bytes
    );
    // Spare bank capacity is still physically owned. Failure backing already
    // belongs to this caller's real returned receipt and is absent here.
    let installed = candidate_peak(false) - size_of::<Intake>() - size_of::<Plan>();
    let exact = installed + event_bytes + path_bytes + continuation_bytes;
    assert!(matches!(
        context.owner.set_limits(DomElementTaskRuntimeLimits {
            storage_bytes: exact - 1,
            ..limits()
        }),
        Err(DomElementTaskRuntimeError::Admission)
    ));
    context
        .owner
        .set_limits(DomElementTaskRuntimeLimits {
            storage_bytes: exact,
            ..limits()
        })
        .unwrap();
    drop(context.event.take());
    assert_eq!(
        state.borrow().payload_bytes.get(),
        counter_body + path_bytes + continuation_bytes
    );
    context
        .owner
        .set_limits(DomElementTaskRuntimeLimits {
            storage_bytes: installed + path_bytes + continuation_bytes,
            ..limits()
        })
        .unwrap();
    drop(context.path.take());
    assert_eq!(
        state.borrow().payload_bytes.get(),
        counter_body + continuation_bytes
    );
    drop(dispatcher);
    assert_eq!(state.borrow().payload_bytes.get(), counter_body);
    context
        .owner
        .set_limits(DomElementTaskRuntimeLimits {
            storage_bytes: installed,
            ..limits()
        })
        .unwrap();
    assert_eq!(receipt.failures().len(), 1);
    assert!(matches!(
        context.owner.set_limits(DomElementTaskRuntimeLimits {
            storage_bytes: installed - 1,
            ..limits()
        }),
        Err(DomElementTaskRuntimeError::Admission)
    ));
}

struct DispatchDropObservation {
    held: HeldObservations,
    calls: Vec<&'static str>,
    running: Option<[(usize, usize, usize); 2]>,
}

impl DomElementTaskEventContext for DispatchDropObservation {
    fn element_tasks(&mut self) -> &mut DomElementTaskRuntime {
        &mut self.held.owner
    }
}

fn details_dispatch_funding(state: &RefCell<ElementTaskState>, dom: &Dom) -> (usize, usize, usize) {
    let state = state.borrow();
    (
        state.payload_bytes.get(),
        state.tasks[0]
            .event
            .as_ref()
            .unwrap()
            .inspect_element_task_source_bytes(),
        dom.dispatch_custody_retained_bytes(&cancel()).unwrap(),
    )
}

fn details_source_admission_cut(dom: &mut Dom, target: NodeId) -> usize {
    let mut refused = 0;
    let mut admitted = source_limits().retained_bytes();
    while admitted - refused > 1 {
        let bytes = refused + (admitted - refused) / 2;
        let limits = DomSourceLimits::new(bytes, 16_000_000, 4_000_000, 64, 65_536, 65_536, 16);
        match dom.begin_source_edit(
            DomSourceEdit::set_attribute(target, "data-probe", ""),
            limits,
            &cancel(),
        ) {
            Ok(lease) => {
                // No cursor step, publication, or effect escapes this probe.
                drop(lease);
                admitted = bytes;
            }
            Err(DomError::ResourceLimit {
                resource: surgeist_dom::DomResource::SourceRetainedBytes,
                limit,
            }) if limit == bytes => refused = bytes,
            Err(error) => panic!("the Source byte allowance decides the cut: {error:?}"),
        }
    }
    admitted
}

fn details_drop_registry() -> DomEventDispatcher<DispatchDropObservation> {
    DomEventDispatcher::new(DomEventRegistryLimits {
        registrations: 3,
        pending_events: 1,
        nested_dispatches: 1,
    })
    .unwrap()
}

struct DetailsDispatchDropFixture {
    session: DomSession,
    document: DocumentId,
    target: NodeId,
    context: DispatchDropObservation,
    dispatcher: DomEventDispatcher<DispatchDropObservation>,
}

fn details_dispatch_drop_fixture(drop_while_running: bool) -> DetailsDispatchDropFixture {
    let (mut session, document, target, _) = fixture();
    open(&mut session, target);
    let mut owner = session.element_task_execution();
    owner.ingest(&mut session.dom(), &cancel()).unwrap();
    let context = DispatchDropObservation {
        held: HeldObservations {
            owner,
            event: None,
            path: None,
        },
        calls: Vec::new(),
        running: None,
    };
    let mut dispatcher = details_drop_registry();
    dispatcher
        .add_listener(
            &mut session.dom(),
            target,
            DomEventType::Toggle,
            DomEventCallback::new(
                move |dom, listeners, context: &mut DispatchDropObservation, event| {
                    context.calls.push("prefix");
                    context.held.event = Some(event.clone());
                    context.held.path = Some(event.composed_path());
                    if drop_while_running {
                        let state = context.held.owner.state().unwrap();
                        let (runtime, _, canonical) = details_dispatch_funding(&state, dom.raw());
                        let registry_bytes = listeners.inspect_pending_index_bytes();
                        let before = (
                            runtime,
                            details_source_admission_cut(dom.raw_mut(), target) - registry_bytes,
                            canonical,
                        );
                        drop(std::mem::replace(listeners, details_drop_registry()));
                        let (runtime, _, canonical) = details_dispatch_funding(&state, dom.raw());
                        let after = (
                            runtime,
                            details_source_admission_cut(dom.raw_mut(), target),
                            canonical,
                        );
                        context.running = Some([before, after]);
                    }
                    Err(DomEventCallbackError { code: 91 })
                },
            ),
            DomEventListenerOptions::default(),
        )
        .unwrap();
    if !drop_while_running {
        dispatcher
            .add_listener(
                &mut session.dom(),
                target,
                DomEventType::Toggle,
                DomEventCallback::host(17),
                DomEventListenerOptions::default(),
            )
            .unwrap();
    }
    add_details_suffix(&mut session, target, &mut dispatcher);
    DetailsDispatchDropFixture {
        session,
        document,
        target,
        context,
        dispatcher,
    }
}

fn add_details_suffix(
    session: &mut DomSession,
    target: NodeId,
    dispatcher: &mut DomEventDispatcher<DispatchDropObservation>,
) {
    dispatcher
        .add_listener(
            &mut session.dom(),
            target,
            DomEventType::Toggle,
            DomEventCallback::new(|_, _, context: &mut DispatchDropObservation, _| {
                context.calls.push("suffix");
                Ok(())
            }),
            DomEventListenerOptions::default(),
        )
        .unwrap();
}

fn details_run_limits() -> crate::DomElementTaskRunLimits {
    crate::DomElementTaskRunLimits {
        dispatch: dispatch_limits(),
        source: source_limits(),
    }
}

fn details_observation_bytes(held: &HeldObservations) -> (usize, usize) {
    let event = held.event.as_ref().unwrap();
    let path = held.path.as_ref().unwrap();
    let (event_bytes, path_bytes) = crate::dom_event::manipulation_observation_layout(event, path);
    assert_eq!(path.inspect_element_task_source_bytes(), path_bytes);
    (
        event_bytes + event.inspect_continuation_body_bytes(),
        path_bytes,
    )
}

fn assert_abandoned_details_row(
    state: &RefCell<ElementTaskState>,
    task: surgeist_dom::DomElementTaskId,
    held: &mut HeldObservations,
    expected_path: [NodeId; 3],
) {
    let state = state.borrow();
    assert_eq!(state.tasks.len(), 1);
    assert_eq!(state.tasks[0].value.id(), task);
    assert!(state.tasks[0].receipt.is_none());
    let event = held.event.as_ref().unwrap();
    assert_eq!(event.current_target(), None);
    assert_eq!(event.phase(), DomEventPhase::None);
    assert!(event.composed_path().next().is_none());
    let toggle = event.toggle_event().unwrap();
    assert_eq!(toggle.old_state(), "closed");
    assert_eq!(toggle.new_state(), "open");
    assert_eq!(
        held.path.as_mut().unwrap().by_ref().collect::<Vec<_>>(),
        expected_path.map(surgeist_dom::DomEventTarget::Node),
    );
}

#[test]
fn details_dispatcher_drop_releases_retained_dispatch_with_live_task_owner() {
    let DetailsDispatchDropFixture {
        mut session,
        document,
        target,
        mut context,
        mut dispatcher,
    } = details_dispatch_drop_fixture(false);
    let snapshot = session.dom().snapshot();
    let stamp = snapshot.stamp();
    let parent = snapshot.node(target).unwrap().parent().unwrap();
    let source = surgeist_dom::DomElementTaskSourceKind::DomManipulation;
    let original = context.held.owner.pending_task(source, 0).unwrap().unwrap();
    let progress = DomElementTaskRuntime::run_next(
        &mut session.dom(),
        source,
        &mut dispatcher,
        &mut context,
        details_run_limits(),
        &cancel(),
    )
    .unwrap();
    assert!(matches!(
        progress,
        DomElementTaskProgress::Retained {
            task,
            dispatch: DomEventDispatchOutcome::RetainedHostUnavailable,
            displaced_owner: None,
        } if task.id() == original.id()
    ));
    assert_eq!(context.calls, ["prefix"]);

    let state = context.held.owner.state().unwrap();
    let counter = std::mem::size_of_val(state.borrow().payload_bytes.as_ref());
    let (fixed, path) = details_observation_bytes(&context.held);
    let (snapshot, failures, association) = context
        .held
        .event
        .as_ref()
        .unwrap()
        .inspect_retained_dispatch_banks();
    assert!(snapshot > 0 && failures > 0 && association > 0);
    let retained = details_dispatch_funding(&state, session.dom().raw());
    assert_eq!(
        retained.0,
        counter + fixed + path + snapshot + failures + association
    );
    assert_eq!(retained.1, fixed + path + snapshot + failures);

    drop(dispatcher);
    let abandoned = details_dispatch_funding(&state, session.dom().raw());
    assert!(
        abandoned.2 < retained.2,
        "the canonical dispatch scope was released"
    );
    assert_abandoned_details_row(
        &state,
        original.id(),
        &mut context.held,
        [target, parent, document.node()],
    );
    let installed = candidate_peak(false) - size_of::<Intake>() - size_of::<Plan>();
    let exact = installed + fixed + path;
    let short = context.held.owner.set_limits(DomElementTaskRuntimeLimits {
        storage_bytes: exact - 1,
        ..limits()
    });
    let admitted = context.held.owner.set_limits(DomElementTaskRuntimeLimits {
        storage_bytes: exact,
        ..limits()
    });
    assert!(matches!(short, Err(DomElementTaskRuntimeError::Admission)));
    assert_eq!(
        (abandoned.0, abandoned.1, admitted.is_ok()),
        (counter + fixed + path, fixed, true),
        "dispatcher drop releases the snapshot, failure and association projection before retry"
    );
    assert_eq!(
        context
            .held
            .path
            .as_ref()
            .unwrap()
            .inspect_element_task_source_bytes(),
        path
    );
    drop(context.held.path.take());
    assert_eq!(state.borrow().payload_bytes.get(), counter + fixed);
    context.held.owner.set_limits(limits()).unwrap();

    let mut replacement = details_drop_registry();
    add_details_suffix(&mut session, target, &mut replacement);
    let retry = DomElementTaskRuntime::run_next(
        &mut session.dom(),
        source,
        &mut replacement,
        &mut context,
        details_run_limits(),
        &cancel(),
    );
    assert!(matches!(
        retry,
        Err(DomElementTaskRuntimeError::Dispatch(
            DomEventDispatchError::Abandoned
        ))
    ));
    assert_eq!(context.calls, ["prefix"]);
    assert_eq!(
        context
            .held
            .owner
            .pending_task(source, 0)
            .unwrap()
            .unwrap()
            .id(),
        original.id()
    );
    assert!(state.borrow().tasks[0].receipt.is_none());
    assert_eq!(session.dom().snapshot().stamp(), stamp);
}

#[test]
fn details_dispatcher_drop_keeps_running_dispatch_funded_until_callback_returns() {
    let DetailsDispatchDropFixture {
        mut session,
        document,
        target,
        mut context,
        mut dispatcher,
    } = details_dispatch_drop_fixture(true);
    let parent = session
        .dom()
        .snapshot()
        .node(target)
        .unwrap()
        .parent()
        .unwrap();
    let source = surgeist_dom::DomElementTaskSourceKind::DomManipulation;
    let original = context.held.owner.pending_task(source, 0).unwrap().unwrap();
    let result = DomElementTaskRuntime::run_next(
        &mut session.dom(),
        source,
        &mut dispatcher,
        &mut context,
        details_run_limits(),
        &cancel(),
    );
    assert!(matches!(
        result,
        Err(DomElementTaskRuntimeError::Dispatch(
            DomEventDispatchError::Abandoned
        ))
    ));
    let state = context.held.owner.state().unwrap();
    let counter = std::mem::size_of_val(state.borrow().payload_bytes.as_ref());
    let (fixed, path) = details_observation_bytes(&context.held);
    let [before, after] = context.running.unwrap();
    assert_eq!(
        after, before,
        "the callback still owns the stack dispatch and canonical scope"
    );
    assert!(before.0 > counter + fixed + path);
    let settled = details_dispatch_funding(&state, session.dom().raw());
    assert_eq!((settled.0, settled.1), (counter + fixed + path, fixed));
    assert!(
        settled.2 < after.2,
        "the actual stack scope releases after return"
    );
    let source_after = details_source_admission_cut(session.dom().raw_mut(), target);
    assert!(
        after.1 - source_after > after.2 - settled.2,
        "return releases the original task Source funding as well as the canonical scope"
    );
    assert_eq!(context.calls, ["prefix"]);
    assert_abandoned_details_row(
        &state,
        original.id(),
        &mut context.held,
        [target, parent, document.node()],
    );
    drop(context.held.path.take());
    assert_eq!(state.borrow().payload_bytes.get(), counter + fixed);
}
