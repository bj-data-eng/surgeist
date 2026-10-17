//! Original aggregate task owner admission and source selection.
use super::*;

#[test]
fn exhausted_source_filter_work_can_be_raised_without_replacing_the_owner() {
    let (mut session, _, first, _) = fixture();
    open(&mut session, first);
    let mut runtime = session.element_task_execution();
    runtime.ingest(&mut session.dom(), &cancel()).unwrap();
    let owner = runtime.state().unwrap();
    let original = owner.borrow().tasks[0].value;
    let spent = owner.borrow().work;
    owner.borrow_mut().limits.work = spent;
    assert!(matches!(
        runtime.pending_tasks(DomElementTaskSourceKind::DomManipulation),
        Err(DomElementTaskRuntimeError::Admission)
    ));
    assert_eq!(owner.borrow().work, spent);
    assert_eq!(owner.borrow().tasks[0].value, original);
    runtime
        .set_limits(limits())
        .expect("a raised policy must recover the same exhausted owner");
    let before = owner.borrow().work;
    assert_eq!(
        runtime
            .pending_tasks(DomElementTaskSourceKind::DomManipulation)
            .unwrap(),
        1
    );
    assert_eq!(
        owner.borrow().work - before,
        1,
        "one actual filter row is charged"
    );
    assert_eq!(owner.borrow().tasks[0].value, original);
    assert!(Rc::ptr_eq(&runtime.state().unwrap(), &owner));
}

fn source_limits() -> surgeist_dom::DomSourceLimits {
    surgeist_dom::DomSourceLimits::new(16_000_000, 1_000_000, 4_000_000, 64, 4096, 4096, 4)
}
fn close_queue(
    final_notification: bool,
) -> (DomSession, NodeId, surgeist_dom::DomSourceDialogRequest) {
    use surgeist_dom::{DomSourceEdit, DomSourceProgress, DomSourceRequest};
    let (mut session, document, _, _) = fixture();
    let mut batch = DomBatch::new();
    let dialog = batch.create_element_ns(document, Namespace::html(), "dialog");
    batch.set_attribute(&dialog, "open", "");
    let subject = apply(&mut session, batch)
        .created(&dialog)
        .unwrap()
        .unwrap();
    let source = session.source_execution();
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
        let mut dom = session.dom();
        let mut lease = dom
            .begin_source_edit(
                DomSourceEdit::close_dialog(subject, None),
                source_limits(),
                &cancel(),
            )
            .unwrap();
        let DomSourceProgress::Request(DomSourceRequest::Event(event)) =
            lease.advance(&cancel()).unwrap()
        else {
            panic!("actual Close beforetoggle")
        };
        event
    };
    let mut dispatcher = DomEventDispatcher::<()>::new(DomEventRegistryLimits {
        registrations: 0,
        pending_events: 1,
        nested_dispatches: 1,
    })
    .unwrap();
    assert!(matches!(
        source
            .service_event(
                &mut session.dom(),
                &event,
                &mut dispatcher,
                &mut (),
                crate::DomSourceEventServiceLimits {
                    source: source_limits(),
                    dispatch: dispatch_limits()
                },
                &cancel()
            )
            .unwrap(),
        crate::DomSourceEventServiceProgress::Serviced { .. }
    ));
    let runtime = session.element_task_execution();
    let queue = {
        let mut dom = session.dom();
        let mut lease = dom
            .resume_source_edit(event.operation(), source_limits(), &cancel())
            .unwrap();
        let DomSourceProgress::Request(DomSourceRequest::Dialog(toggle)) =
            lease.advance(&cancel()).unwrap()
        else {
            panic!("actual Close Toggle queue")
        };
        if final_notification {
            runtime
                .queue_dialog(&mut lease, &toggle, &cancel())
                .unwrap();
            let DomSourceProgress::Request(DomSourceRequest::Dialog(close)) =
                lease.advance(&cancel()).unwrap()
            else {
                panic!("mandatory actual final QueueClose")
            };
            assert_eq!(close.action(), surgeist_html::HtmlDialogAction::QueueClose);
            close
        } else {
            toggle
        }
    };
    (session, subject, queue)
}
#[test]
fn close_reserves_two_original_task_serials_before_its_first_queue_effect() {
    for remaining in [1_u64, 2] {
        let (mut session, subject, request) = close_queue(false);
        let runtime = session.element_task_execution();
        let owner = runtime.state().unwrap();
        owner.borrow_mut().next = u64::MAX - remaining;
        let before = session.snapshot().stamp();
        let mut dom = session.dom();
        let mut lease = dom
            .resume_source_edit(request.operation(), source_limits(), &cancel())
            .unwrap();
        let outcome = runtime.queue_dialog(&mut lease, &request, &cancel());
        if remaining == 1 {
            assert!(matches!(
                outcome,
                Err(DomElementTaskRuntimeError::IdentityExhausted)
            ));
            assert_eq!(lease.snapshot().stamp(), before);
            assert!(owner.borrow().tasks.is_empty());
            assert!(
                !lease
                    .snapshot()
                    .node(subject)
                    .unwrap()
                    .as_dialog()
                    .unwrap()
                    .has_toggle_task_tracker()
            );
        } else {
            outcome.unwrap();
            let surgeist_dom::DomSourceProgress::Request(surgeist_dom::DomSourceRequest::Dialog(
                close,
            )) = lease.advance(&cancel()).unwrap()
            else {
                panic!("actual final Close queue")
            };
            runtime.queue_dialog(&mut lease, &close, &cancel()).unwrap();
            assert!(matches!(
                lease.advance(&cancel()).unwrap(),
                surgeist_dom::DomSourceProgress::Complete(_)
            ));
            let state = owner.borrow();
            assert_eq!(state.next, u64::MAX);
            assert_eq!(state.tasks.len(), 2);
            assert_eq!(
                state.tasks[0].value.source(),
                DomElementTaskSourceKind::DomManipulation
            );
            assert_eq!(
                state.tasks[1].value.source(),
                DomElementTaskSourceKind::UserInteraction
            );
            assert_ne!(state.tasks[0].value.id(), state.tasks[1].value.id());
        }
    }
}
fn final_candidate_peak() -> usize {
    // One installed Toggle row and its retained tracker-bank capacity coexist
    // with two-entry plan/output banks, a one-entry tracker bank and their
    // real Intake owner. Dialog tasks do not append a Details tracker record.
    size_of::<RefCell<ElementTaskState>>()
        + size_of::<Cell<usize>>()
        + size_of::<surgeist_dom::DomElementTaskSource>()
        + surgeist_dom::DomElementTaskSource::binding_payload_bytes()
        + size_of::<Intake>()
        + size_of::<Task>()
        + size_of::<Tracker>()
        + 2 * (size_of::<Plan>() + size_of::<Task>())
        + size_of::<Tracker>()
}
#[test]
fn final_close_queue_exact_overlap_refusal_and_affine_retry_preserve_toggle_tracker() {
    for short in [true, false] {
        let (mut session, subject, request) = close_queue(true);
        let mut runtime = session.element_task_execution();
        let owner = runtime.state().unwrap();
        assert_eq!(owner.borrow().tasks.len(), 1);
        assert_eq!(owner.borrow().tasks.capacity(), 1);
        assert_eq!(owner.borrow().trackers.len(), 0);
        assert_eq!(owner.borrow().trackers.capacity(), 1);
        // plan() adds one actual queue row: plan/output request two slots,
        // while zero Details trackers plus its one additional slot request one.
        assert_eq!(owner.borrow().tasks.len() + 1, 2);
        assert_eq!(owner.borrow().trackers.len() + 1, 1);
        let toggle = owner.borrow().tasks[0].value;
        runtime
            .set_limits(DomElementTaskRuntimeLimits {
                tasks: 2,
                storage_bytes: final_candidate_peak() - usize::from(short),
                work: 1_000_000,
            })
            .unwrap();
        let before = session.snapshot().stamp();
        let published = {
            let mut dom = session.dom();
            let mut lease = dom
                .resume_source_edit(request.operation(), source_limits(), &cancel())
                .unwrap();
            assert!(matches!(
                runtime.queue_dialog(&mut lease, &request, &AtomicBool::new(true)),
                Err(DomElementTaskRuntimeError::Dom(DomError::Cancelled))
            ));
            let result = owner
                .borrow_mut()
                .publish_dialog_queue(&mut lease, &request, &cancel());
            if short {
                match result {
                    Err(DomElementTaskRuntimeError::Admission) => {}
                    other => panic!("one-short actual queue candidate: {other:?}"),
                }
                assert_eq!(lease.snapshot().stamp(), before);
                assert_eq!(owner.borrow().tasks.len(), 1);
                runtime
                    .set_limits(DomElementTaskRuntimeLimits {
                        tasks: 2,
                        storage_bytes: final_candidate_peak(),
                        work: 1_000_000,
                    })
                    .unwrap();
            } else {
                assert_eq!(result.unwrap(), 1);
            }
            if short {
                assert_eq!(
                    owner
                        .borrow_mut()
                        .publish_dialog_queue(&mut lease, &request, &cancel())
                        .unwrap(),
                    1
                );
            }
            let stamp = lease.snapshot().stamp();
            assert_ne!(stamp, before);
            let mut state = owner.borrow_mut();
            assert!(matches!(
                lease.accept_dialog_task_queue(
                    &request,
                    &mut state.tasks[1].publication,
                    &AtomicBool::new(true)
                ),
                Err(DomError::Cancelled)
            ));
            assert!(state.tasks[1].publication.is_some());
            stamp
        }; // Drop/reacquire the exact Source lease and weak Runtime permission.
        let replacement = session.element_task_execution();
        {
            let mut dom = session.dom();
            let mut lease = dom
                .resume_source_edit(request.operation(), source_limits(), &cancel())
                .unwrap();
            replacement
                .queue_dialog(&mut lease, &request, &cancel())
                .unwrap();
            assert_eq!(lease.snapshot().stamp(), published);
            assert_eq!(owner.borrow().tasks.len(), 2);
            assert!(owner.borrow().tasks[1].publication.is_none());
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
                toggle.id()
            );
            assert!(matches!(
                lease.advance(&cancel()).unwrap(),
                surgeist_dom::DomSourceProgress::Complete(_)
            ));
        }
        // The exact policy above covers queue formation and its affine
        // acceptance, not the separate event/path/listener allocation below.
        // Raise ordinary delivery policy on this SAME owner after proving the
        // original Source completed with both queue identities preserved.
        runtime.set_limits(limits()).unwrap();
        assert!(Rc::ptr_eq(&runtime.state().unwrap(), &owner));
        assert_eq!(owner.borrow().tasks[0].value, toggle);
        let mut context = context(&mut session);
        let mut dispatcher = registry();
        let close = owner.borrow().tasks[1].value;
        assert!(close.dialog_close().is_some());
        let outcome = DomElementTaskRuntime::run_next(
            &mut session.dom(),
            DomElementTaskSourceKind::UserInteraction,
            &mut dispatcher,
            &mut context,
            DomElementTaskRunLimits {
                dispatch: dispatch_limits(),
                source: source_limits(),
            },
            &cancel(),
        )
        .unwrap();
        let DomElementTaskProgress::Complete { task, receipt, .. } = outcome else {
            panic!("actual plain Close delivery")
        };
        assert_eq!(task, close);
        assert_eq!(receipt.event_type(), DomEventType::Close);
        assert_eq!(owner.borrow().tasks.len(), 1);
        assert_eq!(owner.borrow().tasks[0].value, toggle);
        assert_eq!(
            session
                .snapshot()
                .node(subject)
                .unwrap()
                .as_dialog()
                .unwrap()
                .toggle_task_tracker()
                .unwrap()
                .task(),
            toggle.id()
        );
        let outcome = DomElementTaskRuntime::run_next(
            &mut session.dom(),
            DomElementTaskSourceKind::DomManipulation,
            &mut dispatcher,
            &mut context,
            DomElementTaskRunLimits {
                dispatch: dispatch_limits(),
                source: source_limits(),
            },
            &cancel(),
        )
        .unwrap();
        assert!(matches!(outcome, DomElementTaskProgress::Complete { .. }));
        assert_eq!(runtime.pending_total_tasks().unwrap(), 0);
        assert!(
            !session
                .snapshot()
                .node(subject)
                .unwrap()
                .as_dialog()
                .unwrap()
                .has_toggle_task_tracker()
        );
    }
}
