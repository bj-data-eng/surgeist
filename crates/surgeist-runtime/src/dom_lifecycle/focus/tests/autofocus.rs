//! The actual insertion allow-focus occurrence uses the original receipt slot.
use super::*;

struct Insertion {
    session: DomSession,
    request: DomSourceAutofocusInsertionRequest,
    document: DocumentId,
    global: DomHostedGlobalId,
    host: DomHostedHostId,
    subject: NodeId,
}
fn parked_insertion() -> Insertion {
    let (mut session, host, _) = fixture();
    let cancel = AtomicBool::new(false);
    let mut hosted = session.hosted_execution();
    let top = hosted
        .create_top_level(&mut session.dom(), host, &cancel)
        .unwrap();
    let snapshot = session.snapshot();
    let html = snapshot.node(top.document().node()).unwrap().children()[0];
    let body = snapshot.node(html).unwrap().children()[1];
    let mut batch = DomBatch::new();
    let detached = batch.create_html_document(top.document(), None);
    let iframe = batch.create_element_ns(&detached, Namespace::html(), "iframe");
    batch.set_attribute(&iframe, "allow", "focus-without-user-activation 'none'");
    let iframe = session
        .dom()
        .apply(batch, &cancel)
        .unwrap()
        .created(&iframe)
        .unwrap()
        .unwrap();
    let inserted = session
        .source_execution()
        .apply(
            &mut session.dom(),
            DomSourceEdit::insert_before(iframe, body, None),
            source_limits(10_000_000),
            &mut DomSourceServices {
                execution_scope: None,
                style_policy: &mut |_| None,
                url: None,
            },
            &cancel,
        )
        .unwrap();
    assert!(matches!(inserted, DomSourceDriveProgress::Complete(_)));
    let snapshot = session.snapshot();
    let child = snapshot
        .node(iframe)
        .unwrap()
        .as_element()
        .unwrap()
        .child_navigable()
        .unwrap()
        .unwrap();
    let document = snapshot
        .hosted_navigable(child)
        .unwrap()
        .active_document()
        .unwrap();
    let html = snapshot.node(document.node()).unwrap().children()[0];
    let body = snapshot.node(html).unwrap().children()[1];
    let mut batch = DomBatch::new();
    let subject = batch.create_element_ns(document, Namespace::html(), "input");
    batch.set_attribute(&subject, "autofocus", "");
    let subject = session
        .dom()
        .apply(batch, &cancel)
        .unwrap()
        .created(&subject)
        .unwrap()
        .unwrap();
    let global = session
        .snapshot()
        .relevant_global(subject)
        .unwrap()
        .unwrap();
    let request = {
        let mut dom = session.dom();
        let mut lease = dom
            .raw_mut()
            .begin_source_edit(
                DomSourceEdit::insert_before(subject, body, None),
                source_limits(10_000_000),
                &cancel,
            )
            .unwrap();
        let DomSourceProgress::Request(DomSourceRequest::AutofocusInsertion(request)) =
            lease.advance(&cancel).unwrap()
        else {
            panic!("actual insertion allow-focus request required")
        };
        assert_eq!(
            request.action(),
            surgeist_html::HtmlAutofocusInsertionAction::ReadAllowFocus
        );
        assert_eq!(request.subject(), subject);
        assert_eq!(
            lease.status().phase(),
            DomSourcePhase::AutofocusInsertionService
        );
        request
    };
    Insertion {
        session,
        request,
        document,
        global,
        host,
        subject,
    }
}
fn retained_insertion() -> Insertion {
    let mut case = parked_insertion();
    let request = DomFocusRequest::AllowAutofocusInsertion(case.request.clone());
    let (cap, _, target) = source_focus_boundary(&mut case.session, &request, 20, false);
    assert_eq!(
        target,
        DomFocusTarget::DeniedDocumentWindow {
            document: case.document,
            global: case.global,
            host: case.host
        }
    );
    let hosted = case.session.hosted_execution();
    let owner = hosted.state().unwrap();
    let before = owner.borrow().work;
    {
        let mut dom = case.session.dom();
        let mut lease = dom
            .raw_mut()
            .resume_source_edit(
                case.request.operation(),
                source_limits(cap),
                &AtomicBool::new(false),
            )
            .unwrap();
        assert!(
            matches!(hosted.service_focus(&mut lease, &DomFocusRequest::AllowAutofocusInsertion(case.request.clone()), &AtomicBool::new(false)),
            Err(DomHostedRuntimeError::Dom(DomHostedError::Dom(DomError::SourceWorkLimit {limit}))) if limit == cap)
        );
    }
    // Host/composition/cluster3 + two Windows2 + two Global/settings pairs4
    // give nine retained visits. This fixture has no watcher or invocation.
    // Child Global5 + child Window5 + actual observation1 = twenty visits.
    assert_eq!(owner.borrow().work - before, 9 + 5 + 5 + 1);
    assert_eq!(
        owner.borrow().focus.pending.as_ref().unwrap().request(),
        &DomFocusRequest::AllowAutofocusInsertion(case.request.clone())
    );
    case
}
#[test]
fn actual_insertion_denial_receipt_survives_refusal_cancel_foreign_source_and_permission_drop() {
    let mut first = retained_insertion();
    let mut second = retained_insertion();
    let a = first.session.hosted_execution().state().unwrap();
    let b = second.session.hosted_execution().state().unwrap();
    let mut foreign = a.borrow_mut().focus.pending.take();
    let cancel = AtomicBool::new(false);
    {
        let mut dom = second.session.dom();
        let mut lease = dom
            .raw_mut()
            .resume_source_edit(
                second.request.operation(),
                source_limits(10_000_000),
                &cancel,
            )
            .unwrap();
        assert!(matches!(
            lease.accept_focus(
                &DomFocusRequest::AllowAutofocusInsertion(second.request.clone()),
                &b.borrow().source,
                &mut foreign,
                &cancel
            ),
            Err(DomError::InvalidSourceOperation)
        ));
        assert_eq!(
            foreign.as_ref().unwrap().request(),
            &DomFocusRequest::AllowAutofocusInsertion(first.request.clone())
        );
    }
    a.borrow_mut().focus.pending = foreign;
    for case in [&mut first, &mut second] {
        let old = case.session.hosted_execution();
        let owner = old.state().unwrap();
        drop(old);
        let replacement = case.session.hosted_execution();
        let before = owner.borrow().work;
        {
            let mut dom = case.session.dom();
            let mut lease = dom
                .raw_mut()
                .resume_source_edit(case.request.operation(), source_limits(10_000_000), &cancel)
                .unwrap();
            assert!(matches!(
                replacement.service_focus(
                    &mut lease,
                    &DomFocusRequest::AllowAutofocusInsertion(case.request.clone()),
                    &AtomicBool::new(true)
                ),
                Err(DomHostedRuntimeError::Cancelled
                    | DomHostedRuntimeError::Dom(DomHostedError::Dom(DomError::Cancelled)))
            ));
            assert_eq!(
                owner.borrow().focus.pending.as_ref().unwrap().request(),
                &DomFocusRequest::AllowAutofocusInsertion(case.request.clone())
            );
            assert!(
                replacement
                    .service_focus(
                        &mut lease,
                        &DomFocusRequest::AllowAutofocusInsertion(case.request.clone()),
                        &cancel
                    )
                    .unwrap()
            );
            assert_eq!(
                owner.borrow().work,
                before,
                "the retained insertion observation must not replay Window reads"
            );
            assert!(owner.borrow().focus.pending.is_none());
            assert!(matches!(
                lease.prepare_focus(
                    &DomFocusRequest::AllowAutofocusInsertion(case.request.clone()),
                    &owner.borrow().source,
                    &cancel
                ),
                Err(DomError::InvalidSourceOperation)
            ));
            assert!(matches!(
                lease.advance(&cancel).unwrap(),
                DomSourceProgress::Complete(_)
            ));
        }
        assert!(case.session.dom().pending_source_edit().is_none());
        let snapshot = case.session.snapshot();
        assert!(snapshot.node(case.subject).unwrap().parent().is_some());
        let view = snapshot
            .document(case.document)
            .unwrap()
            .autofocus()
            .unwrap();
        assert!(view.candidates().is_empty());
        assert!(!view.processed());
        assert_eq!(case.session.dom().usage().autofocus_candidates(), 0);
        assert_eq!(
            case.session
                .element_task_execution()
                .pending_total_tasks()
                .unwrap(),
            0
        );
    }
}

#[test]
fn same_origin_autofocus_tail_output_is_preadmitted_before_actual_toggle_queue_effect() {
    for short in [true, false] {
        let (mut session, host, _) = fixture();
        let cancel = AtomicBool::new(false);
        let mut hosted = session.hosted_execution();
        let top = hosted
            .create_top_level(&mut session.dom(), host, &cancel)
            .unwrap();
        let snapshot = session.snapshot();
        let html = snapshot.node(top.document().node()).unwrap().children()[0];
        let body = snapshot.node(html).unwrap().children()[1];
        let mut batch = DomBatch::new();
        let subject = batch.create_element_ns(top.document(), Namespace::html(), "dialog");
        batch.insert_before(&subject, body, None);
        let subject = session
            .dom()
            .apply(batch, &cancel)
            .unwrap()
            .created(&subject)
            .unwrap()
            .unwrap();
        // Attribute changes occur after real insertion, so no autofocus
        // insertion history is claimed by this empty-before-queue fixture.
        for name in ["inert", "autofocus"] {
            let done = session
                .source_execution()
                .apply(
                    &mut session.dom(),
                    DomSourceEdit::set_attribute(subject, name, ""),
                    source_limits(10_000_000),
                    &mut DomSourceServices {
                        execution_scope: None,
                        style_policy: &mut |_| None,
                        url: None,
                    },
                    &cancel,
                )
                .unwrap();
            assert!(matches!(done, DomSourceDriveProgress::Complete(_)));
        }
        let ready = session.snapshot();
        let initial = ready.document(top.document()).unwrap().autofocus().unwrap();
        assert!(initial.candidates().is_empty());
        assert!(!initial.processed());
        // Queue tracker + Source marker2; AddOpen attribute + open-list2;
        // watcher ElementState1; modal ElementState + raw-layer Document2;
        // changed top-Document autofocus flag1; final Source marker1 = nine.
        // Emptying the already-empty list contributes no second autofocus row.
        // Remembering None preserves the fresh already-None slot, adding no row.
        // Five original publication phases suffice: the new tail joins final
        // publication and does not add a service or phase allowance.
        const EXACT_OUTPUTS: usize = 2 + 2 + 1 + 2 + 1 + 1;
        let bounded = |outputs| {
            let ordinary = source_limits(10_000_000);
            DomSourceLimits::new(
                ordinary.retained_bytes(),
                ordinary.retained_string_code_units(),
                ordinary.work(),
                5,
                outputs,
                ordinary.published_native_identities(),
                ordinary.nested_operations(),
            )
        };
        let policy = bounded(EXACT_OUTPUTS - usize::from(short));
        let before = match session
            .source_execution()
            .apply(
                &mut session.dom(),
                DomSourceEdit::show_modal_dialog(subject),
                policy,
                &mut DomSourceServices {
                    execution_scope: None,
                    style_policy: &mut |_| None,
                    url: None,
                },
                &cancel,
            )
            .unwrap()
        {
            DomSourceDriveProgress::Pending {
                request: DomSourceRequest::Event(event),
                ..
            } => event,
            other => panic!("actual beforetoggle must precede QueueToggle preadmission: {other:?}"),
        };
        let mut registry = DomEventDispatcher::new(DomEventRegistryLimits {
            registrations: 1,
            pending_events: 1,
            nested_dispatches: 1,
        })
        .unwrap();
        registry
            .add_listener(
                &session.dom(),
                subject,
                crate::DomEventType::BeforeToggle,
                crate::DomEventCallback::new(|_, _, calls: &mut usize, event| {
                    assert!(event.cancelable());
                    *calls += 1;
                    Ok(())
                }),
                crate::DomEventListenerOptions {
                    capture: false,
                    once: true,
                },
            )
            .unwrap();
        let mut calls = 0;
        match session
            .source_execution()
            .service_event(
                &mut session.dom(),
                &before,
                &mut registry,
                &mut calls,
                DomSourceEventServiceLimits {
                    source: policy,
                    dispatch: DomEventDispatchLimits {
                        path_items: 16,
                        snapshot_listeners: 1,
                        failure_records: 1,
                        callbacks: 1,
                        work: 100_000,
                    },
                },
                &cancel,
            )
            .unwrap()
        {
            DomSourceEventServiceProgress::Serviced { receipt } => {
                assert_eq!(receipt.callbacks(), 1);
                assert!(receipt.failures().is_empty());
                assert!(!receipt.default_prevented());
            }
            other => panic!("real beforetoggle receipt required: {other:?}"),
        }
        assert_eq!(calls, 1);
        let tasks = session.element_task_execution();
        let stamp = session.snapshot().stamp();
        let queue = {
            let mut dom = session.dom();
            let mut lease = dom
                .raw_mut()
                .resume_source_edit(before.operation(), policy, &cancel)
                .unwrap();
            let queue = request(
                &mut lease,
                HtmlDialogAction::QueueToggle {
                    old: HtmlDialogState::Closed,
                    new: HtmlDialogState::Open,
                    source: HtmlDialogEventSource::Null,
                },
            );
            if short {
                assert!(matches!(
                    tasks.queue_dialog(&mut lease, &queue, &cancel),
                    Err(crate::DomElementTaskRuntimeError::Dom(
                        DomError::SourceOutputLimit
                    ))
                ));
                assert_eq!(lease.snapshot().stamp(), stamp);
                assert_eq!(request(&mut lease, queue.action()), queue);
                let current = lease.snapshot();
                let dialog = current.node(subject).unwrap().as_dialog().unwrap();
                assert!(!dialog.is_modal());
                assert!(!dialog.has_close_watcher());
                assert_eq!(
                    current.node(subject).unwrap().attribute(&"open".into()),
                    None
                );
                assert_eq!(tasks.pending_total_tasks().unwrap(), 0);
                assert!(hosted.state().unwrap().borrow().focus.pending.is_none());
            }
            queue
        };
        {
            let mut dom = session.dom();
            let mut lease = dom
                .raw_mut()
                .resume_source_edit(before.operation(), bounded(EXACT_OUTPUTS), &cancel)
                .unwrap();
            tasks.queue_dialog(&mut lease, &queue, &cancel).unwrap();
        }
        assert!(matches!(
            session
                .source_execution()
                .resume(
                    &mut session.dom(),
                    before.operation(),
                    bounded(EXACT_OUTPUTS),
                    &mut DomSourceServices {
                        execution_scope: None,
                        style_policy: &mut |_| None,
                        url: None
                    },
                    &cancel
                )
                .unwrap(),
            DomSourceDriveProgress::Complete(_)
        ));
        let complete = session.snapshot();
        assert_eq!(calls, 1);
        assert!(session.dom().pending_source_edit().is_none());
        assert_eq!(tasks.pending_total_tasks().unwrap(), 1);
        let final_state = complete
            .document(top.document())
            .unwrap()
            .autofocus()
            .unwrap();
        assert!(final_state.candidates().is_empty());
        assert!(final_state.processed());
        assert!(!initial.processed());
        let dialog = complete.node(subject).unwrap().as_dialog().unwrap();
        assert!(dialog.is_modal());
        assert_eq!(dialog.close_watcher().unwrap().global(), top.global());
        let task = tasks
            .pending_task(DomElementTaskSourceKind::DomManipulation, 0)
            .unwrap()
            .unwrap();
        assert_eq!(task.queue_association().unwrap().document(), top.document());
        assert_eq!(dialog.toggle_task_tracker().unwrap().task(), task.id());
        assert_eq!(
            complete
                .document(top.document())
                .unwrap()
                .top_layer()
                .unwrap()
                .raw_members(),
            [subject]
        );
    }
}
