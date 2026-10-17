//! Actual current live-Document focus-policy qualification and receipt custody.
use super::*;
use crate::{DomEventCallback, DomEventListenerOptions, DomEventType};

struct LiveCase {
    session: DomSession,
    request: DomSourceDialogFocusingRequest,
    host: DomHostedHostId,
    document: DocumentId,
    global: DomHostedGlobalId,
    subject: NodeId,
    calls: usize,
}
fn parked_live(denied: bool) -> LiveCase {
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
    if denied {
        batch.set_attribute(&iframe, "allow", "focus-without-user-activation 'none'");
    }
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
    let subject = batch.create_element_ns(document, Namespace::html(), "dialog");
    batch.insert_before(&subject, body, None);
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
    let event = match session
        .source_execution()
        .apply(
            &mut session.dom(),
            DomSourceEdit::show_modal_dialog(subject),
            source_limits(10_000_000),
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
        other => panic!("actual ShowModal must dispatch: {other:?}"),
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
            DomEventType::BeforeToggle,
            DomEventCallback::new(|_, _, calls: &mut usize, _| {
                *calls += 1;
                Ok(())
            }),
            DomEventListenerOptions {
                capture: false,
                once: true,
            },
        )
        .unwrap();
    let mut calls = 0;
    let receipt = session
        .source_execution()
        .service_event(
            &mut session.dom(),
            &event,
            &mut registry,
            &mut calls,
            DomSourceEventServiceLimits {
                source: source_limits(10_000_000),
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
        .unwrap();
    match receipt {
        DomSourceEventServiceProgress::Serviced { receipt } => {
            assert_eq!(receipt.callbacks(), 1);
            assert!(receipt.failures().is_empty());
            assert!(!receipt.default_prevented());
        }
        other => panic!("actual receipt required: {other:?}"),
    }
    let tasks = session.element_task_execution();
    let focusing = {
        let mut dom = session.dom();
        let mut lease = dom
            .raw_mut()
            .resume_source_edit(event.operation(), source_limits(10_000_000), &cancel)
            .unwrap();
        let queue = request(
            &mut lease,
            HtmlDialogAction::QueueToggle {
                old: HtmlDialogState::Closed,
                new: HtmlDialogState::Open,
                source: HtmlDialogEventSource::Null,
            },
        );
        tasks.queue_dialog(&mut lease, &queue, &cancel).unwrap();
        let watcher = request(&mut lease, HtmlDialogAction::EstablishCloseWatcher);
        assert!(
            hosted
                .service_dialog_watcher(&mut lease, &watcher, &cancel)
                .unwrap()
        );
        let settlement = hosted.prepare_watcher_settlement(&lease, &cancel).unwrap();
        let remember = request(&mut lease, HtmlDialogAction::RememberFocusedElement);
        settlement.finish(&lease.snapshot());
        assert!(
            hosted
                .service_dialog_focus(
                    &mut lease,
                    &DomDialogFocusRequest::Remember(remember),
                    &cancel
                )
                .unwrap()
        );
        {
            let owner = hosted.state().unwrap();
            let before = owner.borrow().work;
            let retained = owner.borrow().source.retained_storage_bytes();
            let next = focusing_request(
                &mut lease,
                if denied {
                    surgeist_html::HtmlDialogFocusingAction::ReadAllowFocus
                } else {
                    surgeist_html::HtmlDialogFocusingAction::FindFocusDelegate
                },
            );
            assert_eq!(
                owner.borrow().work,
                before,
                "canonical focusing progress performs no Runtime Window observation"
            );
            assert_eq!(owner.borrow().source.retained_storage_bytes(), retained);
            assert!(owner.borrow().focus.pending.is_none());
            next
        }
    };
    LiveCase {
        session,
        request: focusing,
        host,
        document,
        global,
        subject,
        calls,
    }
}

#[test]
fn enabled_permission_advances_locally_to_delegate_wait_without_runtime_result_or_header() {
    let mut case = parked_live(false);
    let cancel = AtomicBool::new(false);
    let hosted = case.session.hosted_execution();
    let owner = hosted.state().unwrap();
    let work = owner.borrow().work;
    let retained = owner.borrow().source.retained_storage_bytes();
    let stamp = case.session.snapshot().stamp();
    let task = case
        .session
        .element_task_execution()
        .pending_task(DomElementTaskSourceKind::DomManipulation, 0)
        .unwrap();
    for _ in 0..2 {
        let mut dom = case.session.dom();
        let mut lease = dom
            .raw_mut()
            .resume_source_edit(case.request.operation(), source_limits(10_000_000), &cancel)
            .unwrap();
        assert_eq!(
            focusing_request(
                &mut lease,
                surgeist_html::HtmlDialogFocusingAction::FindFocusDelegate
            ),
            case.request
        );
        // A real yielded delegate request cannot masquerade as the allow-focus
        // child occurrence, even though its outer Dialog method is unchanged.
        assert!(matches!(
            lease.prepare_dialog_focus(
                &DomDialogFocusRequest::Allow(case.request.clone()),
                &owner.borrow().source,
                &cancel
            ),
            Err(DomError::InvalidSourceOperation)
        ));
        assert_eq!(lease.status().cleanup_cause(), None);
        assert_eq!(lease.snapshot().stamp(), stamp);
        assert_eq!(owner.borrow().work, work);
        assert_eq!(owner.borrow().source.retained_storage_bytes(), retained);
        assert!(owner.borrow().focus.pending.is_none());
    }
    assert_eq!(case.calls, 1);
    assert_eq!(
        case.session
            .element_task_execution()
            .pending_task(DomElementTaskSourceKind::DomManipulation, 0)
            .unwrap(),
        task
    );
    assert_eq!(
        case.session
            .snapshot()
            .node(case.subject)
            .unwrap()
            .owner_document(),
        Some(case.document)
    );
}

fn denied_producer_scaffold() -> (u64, u64, u64) {
    let mut case = parked_live(true);
    let hosted = case.session.hosted_execution();
    let owner = hosted.state().unwrap();
    let state = owner.borrow();
    let cancel = AtomicBool::new(false);
    let mut dom = case.session.dom();
    let mut lease = dom
        .raw_mut()
        .resume_source_edit(case.request.operation(), source_limits(10_000_000), &cancel)
        .unwrap();
    let scaffold = lease.status().spent_work();
    let preparation = {
        let prepared = lease
            .prepare_dialog_focus(
                &DomDialogFocusRequest::Allow(case.request.clone()),
                &state.source,
                &cancel,
            )
            .unwrap()
            .unwrap();
        assert_eq!(
            prepared.target(),
            DomDialogFocusTarget::DeniedDocumentWindow {
                document: case.document,
                global: case.global,
                host: case.host
            }
        );
        lease.status().spent_work() - scaffold
    };
    let before = lease.status().spent_work();
    let payload = lease
        .retain_hosted_runtime_payload(&state.source, size_of::<FocusFormation>(), &cancel)
        .unwrap();
    let retention = lease.status().spent_work() - before;
    drop(payload);
    (scaffold, preparation, retention)
}
#[test]
fn live_denial_retains_actual_receipt_across_source_refusal_cancel_drop_and_retry() {
    let (scaffold, preparation, retention) = denied_producer_scaffold();
    // One Host/composition/cluster3, two Windows2, actual child watcher group
    // and original record2, two Global/settings pairs4 =11 retention visits.
    // Exact child Global lookup5 and original child Window lookup5 precede the
    // actual observation1. This literal22 excludes canonical producer work.
    const RUNTIME_VISITS: u64 = 11 + 5 + 5 + 1;
    let cap = scaffold + preparation + retention + RUNTIME_VISITS;
    let mut case = parked_live(true);
    let cancel = AtomicBool::new(false);
    let hosted = case.session.hosted_execution();
    let owner = hosted.state().unwrap();
    let stamp = case.session.snapshot().stamp();
    let task = case
        .session
        .element_task_execution()
        .pending_task(DomElementTaskSourceKind::DomManipulation, 0)
        .unwrap();
    {
        let state = owner.borrow();
        let agent = &state.hosts[0].top_levels[0]
            .group
            .as_ref()
            .unwrap()
            .clusters[0]
            .agent;
        assert_eq!(state.hosts.len(), 1);
        assert_eq!(state.hosts[0].top_levels.len(), 1);
        assert_eq!(agent.globals.len(), 2);
        assert_eq!(agent.windows.rows.len(), 2);
    }
    {
        let before = owner.borrow().work;
        let mut dom = case.session.dom();
        let mut lease = dom
            .raw_mut()
            .resume_source_edit(case.request.operation(), source_limits(cap), &cancel)
            .unwrap();
        assert_eq!(lease.status().spent_work(), scaffold);
        assert!(
            matches!(hosted.service_dialog_focus(&mut lease,&DomDialogFocusRequest::Allow(case.request.clone()),&cancel),Err(DomHostedRuntimeError::Dom(DomHostedError::Dom(DomError::SourceWorkLimit{limit}))) if limit==cap)
        );
        assert_eq!(owner.borrow().work - before, RUNTIME_VISITS);
        assert_eq!(lease.snapshot().stamp(), stamp);
        assert_eq!(
            owner.borrow().focus.pending.as_ref().unwrap().request(),
            &DomDialogFocusRequest::Allow(case.request.clone())
        );
    }
    drop(hosted);
    let replacement = case.session.hosted_execution();
    {
        let mut dom = case.session.dom();
        let mut lease = dom
            .raw_mut()
            .resume_source_edit(case.request.operation(), source_limits(10_000_000), &cancel)
            .unwrap();
        assert!(matches!(
            replacement.service_dialog_focus(
                &mut lease,
                &DomDialogFocusRequest::Allow(case.request.clone()),
                &AtomicBool::new(true)
            ),
            Err(DomHostedRuntimeError::Cancelled
                | DomHostedRuntimeError::Dom(DomHostedError::Dom(DomError::Cancelled)))
        ));
        assert_eq!(
            owner.borrow().focus.pending.as_ref().unwrap().request(),
            &DomDialogFocusRequest::Allow(case.request.clone())
        );
        let before = owner.borrow().work;
        assert!(
            replacement
                .service_dialog_focus(
                    &mut lease,
                    &DomDialogFocusRequest::Allow(case.request.clone()),
                    &cancel
                )
                .unwrap()
        );
        assert_eq!(
            owner.borrow().work,
            before,
            "retained actual receipt accepts without Window/activation observation replay"
        );
        assert!(owner.borrow().focus.pending.is_none());
        // The actual receipt moved this exact child allow occurrence. Its old
        // request cannot be acknowledged again while the same outer call still
        // owns the now-terminal child cursor.
        assert!(matches!(
            lease.prepare_dialog_focus(
                &DomDialogFocusRequest::Allow(case.request.clone()),
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
    assert_eq!(case.calls, 1);
    assert!(case.session.dom().pending_source_edit().is_none());
    assert_eq!(
        case.session
            .element_task_execution()
            .pending_task(DomElementTaskSourceKind::DomManipulation, 0)
            .unwrap(),
        task
    );
}

fn retained_denied_child() -> LiveCase {
    let (scaffold, preparation, retention) = denied_producer_scaffold();
    // Same independently derived native visits as the exact live-denial case:
    // retained11 + actual child Global5 + original child Window5 + observation1.
    let cap = scaffold + preparation + retention + 22;
    let mut case = parked_live(true);
    let hosted = case.session.hosted_execution();
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
            matches!(hosted.service_dialog_focus(&mut lease, &DomDialogFocusRequest::Allow(case.request.clone()), &AtomicBool::new(false)),
            Err(DomHostedRuntimeError::Dom(DomHostedError::Dom(DomError::SourceWorkLimit {limit}))) if limit == cap)
        );
    }
    assert_eq!(
        hosted
            .state()
            .unwrap()
            .borrow()
            .focus
            .pending
            .as_ref()
            .unwrap()
            .request(),
        &DomDialogFocusRequest::Allow(case.request.clone())
    );
    case
}

#[test]
fn actual_child_allow_receipt_cannot_cross_original_source_and_accepts_once_without_resampling() {
    let mut first = retained_denied_child();
    let mut second = retained_denied_child();
    let first_owner = first.session.hosted_execution().state().unwrap();
    let second_owner = second.session.hosted_execution().state().unwrap();
    let cancel = AtomicBool::new(false);
    let mut foreign = first_owner.borrow_mut().focus.pending.take();
    let original_second = DomDialogFocusRequest::Allow(second.request.clone());
    let basis = second.session.snapshot().stamp();
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
            lease.accept_dialog_focus(
                &original_second,
                &second_owner.borrow().source,
                &mut foreign,
                &cancel
            ),
            Err(DomError::InvalidSourceOperation)
        ));
        assert_eq!(
            foreign.as_ref().unwrap().request(),
            &DomDialogFocusRequest::Allow(first.request.clone())
        );
        assert_eq!(lease.snapshot().stamp(), basis);
        assert_eq!(
            second_owner
                .borrow()
                .focus
                .pending
                .as_ref()
                .unwrap()
                .request(),
            &original_second
        );
    }
    first_owner.borrow_mut().focus.pending = foreign;
    for case in [&mut first, &mut second] {
        let hosted = case.session.hosted_execution();
        let owner = hosted.state().unwrap();
        let before = owner.borrow().work;
        {
            let mut dom = case.session.dom();
            let mut lease = dom
                .raw_mut()
                .resume_source_edit(case.request.operation(), source_limits(10_000_000), &cancel)
                .unwrap();
            assert!(
                hosted
                    .service_dialog_focus(
                        &mut lease,
                        &DomDialogFocusRequest::Allow(case.request.clone()),
                        &cancel
                    )
                    .unwrap()
            );
            assert!(owner.borrow().focus.pending.is_none());
            assert_eq!(owner.borrow().work, before);
            assert!(matches!(
                lease.advance(&cancel).unwrap(),
                DomSourceProgress::Complete(_)
            ));
        }
        assert!(case.session.dom().pending_source_edit().is_none());
        assert_eq!(case.calls, 1);
        assert_eq!(
            case.session
                .element_task_execution()
                .pending_total_tasks()
                .unwrap(),
            1
        );
    }
}

#[test]
fn real_modal_close_reserves_its_post_watcher_pending_removal_output_before_queue() {
    let mut case = parked_live(true);
    let cancel = AtomicBool::new(false);
    let hosted = case.session.hosted_execution();
    {
        let mut dom = case.session.dom();
        let mut lease = dom
            .raw_mut()
            .resume_source_edit(case.request.operation(), source_limits(10_000_000), &cancel)
            .unwrap();
        assert!(
            hosted
                .service_dialog_focus(
                    &mut lease,
                    &DomDialogFocusRequest::Allow(case.request.clone()),
                    &cancel
                )
                .unwrap()
        );
        assert!(matches!(
            lease.advance(&cancel).unwrap(),
            DomSourceProgress::Complete(_)
        ));
    }
    let shown = case.session.snapshot();
    let dialog = shown.node(case.subject).unwrap().as_dialog().unwrap();
    let original_watcher = dialog.close_watcher().expect("real modal watcher");
    assert!(dialog.is_modal());
    let layer = shown.document(case.document).unwrap().top_layer().unwrap();
    assert_eq!(layer.raw_members(), [case.subject]);
    assert!(layer.pending_removals().is_empty());
    let tasks = case.session.element_task_execution();
    let original_task = tasks
        .pending_task(DomElementTaskSourceKind::DomManipulation, 0)
        .unwrap();
    let bounded = |outputs| {
        let ordinary = source_limits(10_000_000);
        DomSourceLimits::new(
            ordinary.retained_bytes(),
            ordinary.retained_string_code_units(),
            ordinary.work(),
            ordinary.phase_publications(),
            outputs,
            ordinary.published_native_identities(),
            ordinary.nested_operations(),
        )
    };
    // QueueToggle publishes tracker + Source start (2). The actual remaining
    // rows are open attribute, open-list removal, watcher/modal state, the
    // distinct post-watcher Document pending-removal change, and Source close
    // (5). No changed return-value row or other receiver contributes here.
    const EXACT_OUTPUTS: usize = 2 + 5;
    let before = match case
        .session
        .source_execution()
        .apply(
            &mut case.session.dom(),
            DomSourceEdit::close_dialog(case.subject, None),
            bounded(EXACT_OUTPUTS - 1),
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
        other => panic!("real Close beforetoggle required: {other:?}"),
    };
    assert!(!before.cancelable());
    let mut registry = DomEventDispatcher::new(DomEventRegistryLimits {
        registrations: 1,
        pending_events: 1,
        nested_dispatches: 1,
    })
    .unwrap();
    registry
        .add_listener(
            &case.session.dom(),
            case.subject,
            DomEventType::BeforeToggle,
            DomEventCallback::new(|_, _, calls: &mut usize, event| {
                assert!(!event.cancelable());
                *calls += 1;
                Ok(())
            }),
            DomEventListenerOptions {
                capture: false,
                once: true,
            },
        )
        .unwrap();
    match case
        .session
        .source_execution()
        .service_event(
            &mut case.session.dom(),
            &before,
            &mut registry,
            &mut case.calls,
            DomSourceEventServiceLimits {
                source: bounded(EXACT_OUTPUTS - 1),
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
        other => panic!("actual Close receipt required: {other:?}"),
    }
    let stamp = case.session.snapshot().stamp();
    let queue = {
        let mut dom = case.session.dom();
        let mut lease = dom
            .raw_mut()
            .resume_source_edit(before.operation(), bounded(EXACT_OUTPUTS - 1), &cancel)
            .unwrap();
        let queue = request(
            &mut lease,
            HtmlDialogAction::QueueToggle {
                old: HtmlDialogState::Open,
                new: HtmlDialogState::Closed,
                source: HtmlDialogEventSource::OperationSource,
            },
        );
        assert!(matches!(
            tasks.queue_dialog(&mut lease, &queue, &cancel),
            Err(crate::DomElementTaskRuntimeError::Dom(
                DomError::SourceOutputLimit
            ))
        ));
        assert_eq!(lease.snapshot().stamp(), stamp);
        assert_eq!(request(&mut lease, queue.action()), queue);
        let current = lease.snapshot();
        let dialog = current.node(case.subject).unwrap().as_dialog().unwrap();
        assert!(dialog.is_modal());
        assert_eq!(dialog.close_watcher(), Some(original_watcher));
        assert_eq!(
            current
                .node(case.subject)
                .unwrap()
                .attribute(&"open".into()),
            Some(&"".into())
        );
        queue
    };
    assert_eq!(tasks.pending_total_tasks().unwrap(), 1);
    assert_eq!(
        tasks
            .pending_task(DomElementTaskSourceKind::DomManipulation, 0)
            .unwrap(),
        original_task
    );
    {
        let mut dom = case.session.dom();
        let mut lease = dom
            .raw_mut()
            .resume_source_edit(before.operation(), bounded(EXACT_OUTPUTS), &cancel)
            .unwrap();
        tasks.queue_dialog(&mut lease, &queue, &cancel).unwrap();
    }
    assert!(matches!(
        case.session
            .source_execution()
            .resume(
                &mut case.session.dom(),
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
    assert_eq!(case.calls, 2);
    assert!(case.session.dom().pending_source_edit().is_none());
    let closed = case.session.snapshot();
    let dialog = closed.node(case.subject).unwrap().as_dialog().unwrap();
    assert!(!dialog.is_modal());
    assert!(!dialog.has_close_watcher());
    let layer = closed.document(case.document).unwrap().top_layer().unwrap();
    assert_eq!(layer.raw_members(), [case.subject]);
    assert_eq!(layer.pending_removals(), [case.subject]);
    assert_eq!(
        layer.ua_rules().collect::<Vec<_>>(),
        [DomTopLayerUaRule::UniversalOverlayNone]
    );
    assert_eq!(tasks.pending_total_tasks().unwrap(), 2);
    assert!(
        tasks
            .pending_task(DomElementTaskSourceKind::UserInteraction, 0)
            .unwrap()
            .unwrap()
            .dialog_close()
            .is_some()
    );
}
