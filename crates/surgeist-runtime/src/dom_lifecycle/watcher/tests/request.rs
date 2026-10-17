//! Private custody evidence; all stimuli use real native Source and dispatch.
use super::*;
use crate::{
    DomEventCallback, DomEventListenerOptions, DomEventType, DomSourceEventServiceLimits,
    DomSourceEventServiceProgress, DomSourceRuntime,
};
struct Context {
    source: DomSourceRuntime,
    owner: Rc<RefCell<HostedState>>,
    original: Rc<RefCell<Watcher>>,
    nested: Vec<DomSourceCompletion>,
    old_id: DomCloseWatcherId,
    replacement: Option<DomCloseWatcherId>,
    global_callbacks: usize,
}
#[test]
fn removed_record_stays_in_original_invocation_and_resets_without_touching_replacement() {
    let (mut session, _, _, subject) = fixture();
    let cancel = AtomicBool::new(false);
    drop(apply(
        &mut session,
        DomSourceEdit::set_attribute(subject, "open", ""),
    ));
    let hosted = session.hosted_execution();
    let owner = hosted.state().unwrap();
    let original = owner.borrow().hosts[0].top_levels[0]
        .group
        .as_ref()
        .unwrap()
        .clusters[0]
        .agent
        .windows
        .rows[0]
        .groups[0]
        .members[0]
        .clone();
    let old_id = original.borrow().id;
    let mut context = Context {
        source: session.source_execution(),
        owner: owner.clone(),
        original: original.clone(),
        nested: Vec::new(),
        old_id,
        replacement: None,
        global_callbacks: 0,
    };
    let mut registry = dispatcher();
    registry.add_listener(&session.dom(), subject, DomEventType::Cancel, DomEventCallback::new(move |dom,listeners,context: &mut Context,event| {
        event.prevent_default();
        let published_parent = dom.snapshot().incomplete_source_operation();
        assert!(published_parent.is_some());
        let document = context.old_id.window_document();
        assert!(matches!(dom.raw().relevant_global(subject), Err(DomHostedError::Dom(DomError::IncompleteSourceOperation))));
        assert!(matches!(dom.raw().global_document(context.old_id.global()), Err(DomHostedError::Dom(DomError::IncompleteSourceOperation))));
        assert!(matches!(dom.snapshot().document(document).unwrap().is_fully_active(), Err(DomHostedError::Dom(DomError::IncompleteSourceOperation))));
        // Real callback registration uses the dispatch-only association front;
        // ordinary current-owner reads above remain strict during publication.
        listeners.add_listener(dom, context.old_id.global(), DomEventType::Cancel,
            DomEventCallback::new(|_,_,context: &mut Context,_| {context.global_callbacks += 1; Ok(())}),
            DomEventListenerOptions {capture: true, once: true}).unwrap();
        assert!(context.original.borrow().state.running_cancel_action());
        for edit in [DomSourceEdit::remove_attribute(subject, "open"), DomSourceEdit::set_attribute(subject, "open", "")] {
            let progress = context.source.apply(dom, edit, source_limits(), &mut DomSourceServices {execution_scope: None, style_policy: &mut |_| None, url: None}, &AtomicBool::new(false)).unwrap();
            let DomSourceDriveProgress::Complete(done) = progress else {panic!("actual Hook must finish: {progress:?}");};
            context.nested.push(done);
        }
        assert_eq!(dom.snapshot().incomplete_source_operation(), published_parent);
        let replacement = dom.snapshot().node(subject).unwrap().as_dialog().unwrap().close_watcher().unwrap();
        assert_ne!(replacement, context.old_id);
        context.replacement = Some(replacement);
        let state = context.owner.borrow();
        let window = &state.hosts[0].top_levels[0].group.as_ref().unwrap().clusters[0].agent.windows.rows[0];
        assert_eq!(window.groups.len(), 1);
        assert_eq!(window.groups[0].members.len(), 1);
        assert_eq!(window.groups[0].members[0].borrow().id, replacement);
        assert!(!Rc::ptr_eq(&window.groups[0].members[0], &context.original));
        assert!(window.custody.iter().any(|custody| matches!(custody,
            WatcherCustody::Invocation(invocation) if invocation.body.running_guard && Rc::ptr_eq(&invocation.body.record, &context.original))));
        assert!(context.original.borrow().state.running_cancel_action());
        Ok(())
    }), DomEventListenerOptions::default()).unwrap();
    let initial = session
        .source_execution()
        .apply(
            &mut session.dom(),
            DomSourceEdit::request_close_dialog(subject, Some("retained".into())),
            source_limits(),
            &mut DomSourceServices {
                execution_scope: None,
                style_policy: &mut |_| None,
                url: None,
            },
            &cancel,
        )
        .unwrap();
    let DomSourceDriveProgress::Pending {
        request: DomSourceRequest::Event(request),
        ..
    } = initial
    else {
        panic!("real Cancel required: {initial:?}");
    };
    assert!(original.borrow().state.running_cancel_action());
    let serviced = session
        .source_execution()
        .service_event(
            &mut session.dom(),
            &request,
            &mut registry,
            &mut context,
            DomSourceEventServiceLimits {
                source: source_limits(),
                dispatch: event_limits(),
            },
            &cancel,
        )
        .unwrap();
    let DomSourceEventServiceProgress::Serviced { receipt } = serviced else {
        panic!("real dispatch must finish: {serviced:?}");
    };
    assert!(receipt.default_prevented());
    assert!(receipt.failures().is_empty());
    assert_eq!(
        context.global_callbacks, 0,
        "the already passed Global capture phase does not replay"
    );
    assert!(original.borrow().state.running_cancel_action());
    context.nested.clear();
    let progress = session
        .source_execution()
        .resume(
            &mut session.dom(),
            request.operation(),
            source_limits(),
            &mut DomSourceServices {
                execution_scope: None,
                style_policy: &mut |_| None,
                url: None,
            },
            &cancel,
        )
        .unwrap();
    assert!(matches!(progress, DomSourceDriveProgress::Complete(_)));
    assert!(!original.borrow().state.running_cancel_action());
    let state = owner.borrow();
    let window = &state.hosts[0].top_levels[0]
        .group
        .as_ref()
        .unwrap()
        .clusters[0]
        .agent
        .windows
        .rows[0];
    assert!(window.custody.is_empty());
    assert_eq!(
        window.groups[0].members[0].borrow().id,
        context.replacement.unwrap()
    );
    assert!(
        !window.groups[0].members[0]
            .borrow()
            .state
            .running_cancel_action()
    );
    drop(state);
    drop(receipt);
    drop(progress);
    drop(context);
    drop(original);
    assert!(session.dom().pending_source_edit().is_none());
}

#[test]
fn source_cancellation_after_force_publication_still_finishes_authentic_cancel_and_final_reset() {
    let (mut session, _, _, subject) = fixture();
    drop(apply(
        &mut session,
        DomSourceEdit::set_attribute(subject, "open", ""),
    ));
    let request = start_request(&mut session, subject, Some("published request".into()));
    assert!(
        session
            .snapshot()
            .node(subject)
            .unwrap()
            .as_dialog()
            .unwrap()
            .force_close_watcher_enabled()
    );
    {
        let mut dom = session.dom();
        let mut lease = dom
            .raw_mut()
            .resume_source_edit(request.operation(), source_limits(), &cancel())
            .unwrap();
        lease.request_cancel();
    }
    let mut registry = dispatcher::<usize>();
    registry
        .add_listener(
            &session.dom(),
            subject,
            DomEventType::Cancel,
            DomEventCallback::new(|_, _, calls: &mut usize, event| {
                *calls += 1;
                event.prevent_default();
                Ok(())
            }),
            DomEventListenerOptions::default(),
        )
        .unwrap();
    let mut calls = 0;
    let receipt = service(&mut session, &request, &mut registry, &mut calls);
    assert_eq!(receipt.callbacks(), 1);
    assert!(receipt.default_prevented());
    let DomSourceDriveProgress::Complete(done) = resume(&mut session, request.operation()).unwrap()
    else {
        panic!("cancellation retains the real mandatory tail through completion");
    };
    assert!(done.cancellation_requested());
    assert_eq!(calls, 1);
    assert_surviving_reset(&session, subject, Some(&"published request".into()));
    assert!(session.dom().pending_source_edit().is_none());
    let next = start_request(&mut session, subject, None);
    assert_eq!(
        service(&mut session, &next, &mut registry, &mut calls).callbacks(),
        1
    );
    drop(complete(resume(&mut session, next.operation()).unwrap()));
    assert_eq!(calls, 2);
}

fn cancel() -> AtomicBool {
    AtomicBool::new(false)
}
fn start_request(
    session: &mut DomSession,
    subject: NodeId,
    result: Option<DocumentString>,
) -> DomSourceEventRequest {
    let progress = session
        .source_execution()
        .apply(
            &mut session.dom(),
            DomSourceEdit::request_close_dialog(subject, result),
            source_limits(),
            &mut DomSourceServices {
                execution_scope: None,
                style_policy: &mut |_| None,
                url: None,
            },
            &cancel(),
        )
        .unwrap();
    let DomSourceDriveProgress::Pending {
        request: DomSourceRequest::Event(request),
        ..
    } = progress
    else {
        panic!("actual Cancel required: {progress:?}");
    };
    assert_eq!(request.kind(), DomSourceEventKind::Cancel);
    request
}
fn resume(
    session: &mut DomSession,
    operation: DomSourceOperationId,
) -> Result<DomSourceDriveProgress, crate::DomSourceRuntimeError> {
    session.source_execution().resume(
        &mut session.dom(),
        operation,
        source_limits(),
        &mut DomSourceServices {
            execution_scope: None,
            style_policy: &mut |_| None,
            url: None,
        },
        &cancel(),
    )
}
fn service<C>(
    session: &mut DomSession,
    request: &DomSourceEventRequest,
    registry: &mut crate::DomEventDispatcher<C>,
    context: &mut C,
) -> crate::DomEventDispatchReceipt {
    match session
        .source_execution()
        .service_event(
            &mut session.dom(),
            request,
            registry,
            context,
            DomSourceEventServiceLimits {
                source: source_limits(),
                dispatch: event_limits(),
            },
            &cancel(),
        )
        .unwrap()
    {
        DomSourceEventServiceProgress::Serviced { receipt } => receipt,
        other => panic!("real Cancel must finish: {other:?}"),
    }
}
fn assert_surviving_reset(session: &DomSession, subject: NodeId, result: Option<&DocumentString>) {
    let snapshot = session.snapshot();
    let dialog = snapshot.node(subject).unwrap().as_dialog().unwrap();
    assert!(!dialog.force_close_watcher_enabled());
    assert_eq!(dialog.request_close_return_value(), result);
    assert_eq!(dialog.request_close_source(), None);
}
fn complete(progress: DomSourceDriveProgress) -> DomSourceCompletion {
    let DomSourceDriveProgress::Complete(done) = progress else {
        panic!("real method must finish: {progress:?}");
    };
    assert!(!done.cancellation_requested());
    done
}
#[test]
fn history_consumption_prepays_both_actual_window_walks_and_write_before_effect() {
    for short in [true, false] {
        let (mut session, top, _, subject) = fixture();
        drop(apply(
            &mut session,
            DomSourceEdit::set_attribute(subject, "open", ""),
        ));
        let hosted = session.hosted_execution();
        let mut registry = dispatcher::<()>();
        registry
            .add_listener(
                &session.dom(),
                subject,
                DomEventType::Cancel,
                DomEventCallback::new(|_, _, _, event| {
                    event.prevent_default();
                    Ok(())
                }),
                DomEventListenerOptions::default(),
            )
            .unwrap();
        let request = start_request(&mut session, subject, None);
        let receipt = service(&mut session, &request, &mut registry, &mut ());
        assert!(receipt.default_prevented());
        let owner = hosted.state().unwrap();
        let mut dom = session.dom();
        let mut lease = dom
            .raw_mut()
            .resume_source_edit(request.operation(), source_limits(), &cancel())
            .unwrap();
        let DomSourceProgress::Request(DomSourceRequest::CloseWatcher(reset)) =
            lease.advance(&cancel()).unwrap()
        else {
            panic!("real canceled Cancel requires original guard reset");
        };
        assert_eq!(
            reset.step(),
            DomSourceCloseWatcherStep::Action(
                HtmlCloseWatcherRequestAction::SetRunningCancelAction(false)
            )
        );
        hosted
            .service_close_watcher_request(&mut lease, &reset, &cancel())
            .unwrap();
        let DomSourceProgress::Request(DomSourceRequest::CloseWatcher(consume)) =
            lease.advance(&cancel()).unwrap()
        else {
            panic!("authentic canceled branch must reach history consumption");
        };
        assert_eq!(
            consume.step(),
            DomSourceCloseWatcherStep::Action(
                HtmlCloseWatcherRequestAction::ConsumeHistoryActionActivation
            )
        );
        {
            let state = owner.borrow();
            assert_eq!(state.hosts.len(), 1);
            assert_eq!(state.hosts[0].top_levels.len(), 1);
            let group = state.hosts[0].top_levels[0].group.as_ref().unwrap();
            assert_eq!(group.clusters.len(), 1);
            let agent = &group.clusters[0].agent;
            assert_eq!(agent.globals.len(), 1);
            assert_eq!(agent.windows.rows.len(), 1);
            let window = &agent.windows.rows[0];
            assert_eq!(window.id, top.global());
            assert_eq!(window.groups.len(), 1);
            assert_eq!(window.groups[0].members.len(), 1);
            assert_eq!(window.custody.len(), 1);
        }
        // Source owns the canonical closure. In this actual one-top/one-Window
        // fixture the Runtime work is independently derived: original Window
        // lookup4; retained-bank visits12 (host,composition,cluster,Window,
        // two Settings strings,group,memberbody,custody,dedupgroup,dedupmember,invocationbody);
        // closure validation lookup4; actual assignment lookup4 + write1;
        // invocation/member lookup3. Total28, with no allocation/growth.
        const REQUIRED: u64 = 4 + 12 + 4 + 4 + 1 + 3;
        let (before, cap) = {
            let mut state = owner.borrow_mut();
            let before = state.work;
            let cap = before + REQUIRED - u64::from(short);
            state.limits.work = cap;
            (before, cap)
        };
        let result = hosted.service_close_watcher_request(&mut lease, &consume, &cancel());
        if short {
            assert!(
                matches!(result, Err(DomHostedRuntimeError::Admission)),
                "one-short must refuse before the actual history effect; got {result:?}"
            );
            assert_eq!(owner.borrow().work, cap);
            let DomSourceProgress::Request(DomSourceRequest::CloseWatcher(same)) =
                lease.advance(&cancel()).unwrap()
            else {
                panic!("unperformed consumption must remain pending");
            };
            assert_eq!(same, consume);
        } else {
            result.unwrap();
            assert_eq!(owner.borrow().work - before, REQUIRED);
            let DomSourceProgress::Request(DomSourceRequest::CloseWatcher(settle)) =
                lease.advance(&cancel()).unwrap()
            else {
                panic!("real consumption receipt must reach invocation handoff");
            };
            assert_eq!(settle.step(), DomSourceCloseWatcherStep::SettleInvocation);
        }
    }
}
fn insert_initial_child(
    session: &mut DomSession,
    associated: DocumentId,
    parent: NodeId,
    local: &str,
) -> (NodeId, DomHostedGlobalId, DocumentId) {
    // The established native contract initializes Frame/Iframe in an actual
    // closed Document, then real Source connection creates its hosted child.
    let container = {
        let mut batch = DomBatch::new();
        let closed = batch.create_html_document(associated, None);
        let container = batch.create_element_ns(&closed, Namespace::html(), local);
        session
            .dom()
            .apply(batch, &cancel())
            .unwrap()
            .created(&container)
            .unwrap()
            .unwrap()
    };
    drop(apply(
        session,
        DomSourceEdit::insert_before(container, parent, None),
    ));
    let snapshot = session.snapshot();
    let navigable = snapshot
        .node(container)
        .unwrap()
        .as_element()
        .unwrap()
        .child_navigable()
        .unwrap()
        .unwrap();
    let child = snapshot.hosted_navigable(navigable).unwrap();
    (container, child.global(), child.active_document().unwrap())
}
#[test]
fn canceled_request_history_capture_uses_shadow_tree_order_and_actual_child_documents() {
    let (mut session, top, body, subject) = fixture();
    let shadow = {
        let mut batch = DomBatch::new();
        let host = batch.create_element_ns(top.document(), Namespace::html(), "div");
        let shadow = batch.attach_shadow(&host, DomShadowRootInit::new(HtmlShadowRootMode::Open));
        batch.insert_before(&host, body, None);
        session
            .dom()
            .apply(batch, &cancel())
            .unwrap()
            .created(&shadow)
            .unwrap()
            .unwrap()
    };
    // Allocate this later DOM sibling's actual Global first. A registry/bank
    // iteration would therefore produce the wrong independently expected order.
    let (_, ordinary_global, _) = insert_initial_child(&mut session, top.document(), body, "frame");
    let (_, shadow_global, shadow_document) =
        insert_initial_child(&mut session, top.document(), shadow, "iframe");
    let child_body = {
        let snapshot = session.snapshot();
        let html = snapshot.node(shadow_document.node()).unwrap().children()[0];
        snapshot.node(html).unwrap().children()[1]
    };
    let (_, nested_global, _) =
        insert_initial_child(&mut session, shadow_document, child_body, "iframe");
    let (removed_container, removed_global, _) =
        insert_initial_child(&mut session, top.document(), body, "iframe");
    drop(apply(
        &mut session,
        DomSourceEdit::detach(removed_container),
    ));
    assert_eq!(
        session
            .snapshot()
            .node(removed_container)
            .unwrap()
            .as_element()
            .unwrap()
            .child_navigable(),
        Some(None)
    );
    let mut hosted = session.hosted_execution();
    let unrelated = hosted
        .create_top_level(&mut session.dom(), top.host(), &cancel())
        .unwrap();
    drop(apply(
        &mut session,
        DomSourceEdit::set_attribute(subject, "open", ""),
    ));
    let mut registry = dispatcher::<()>();
    registry
        .add_listener(
            &session.dom(),
            subject,
            DomEventType::Cancel,
            DomEventCallback::new(|_, _, _, event| {
                event.prevent_default();
                Ok(())
            }),
            DomEventListenerOptions::default(),
        )
        .unwrap();
    let request = start_request(&mut session, subject, None);
    let receipt = service(&mut session, &request, &mut registry, &mut ());
    assert!(receipt.default_prevented());
    assert!(receipt.failures().is_empty());
    let owner = hosted.state().unwrap();
    let mut dom = session.dom();
    let mut lease = dom
        .raw_mut()
        .resume_source_edit(request.operation(), source_limits(), &cancel())
        .unwrap();
    let DomSourceProgress::Request(DomSourceRequest::CloseWatcher(reset)) =
        lease.advance(&cancel()).unwrap()
    else {
        panic!("authentic canceled Cancel requires reset");
    };
    assert_eq!(
        reset.step(),
        DomSourceCloseWatcherStep::Action(HtmlCloseWatcherRequestAction::SetRunningCancelAction(
            false
        ))
    );
    hosted
        .service_close_watcher_request(&mut lease, &reset, &cancel())
        .unwrap();
    let DomSourceProgress::Request(DomSourceRequest::CloseWatcher(consume)) =
        lease.advance(&cancel()).unwrap()
    else {
        panic!("actual canceled branch requires consumption");
    };
    assert_eq!(
        consume.step(),
        DomSourceCloseWatcherStep::Action(
            HtmlCloseWatcherRequestAction::ConsumeHistoryActionActivation
        )
    );
    let capture = lease
        .close_watcher_history_windows(&consume, &owner.borrow().source, &cancel())
        .unwrap();
    assert_eq!(capture.window_navigable(), Some(top.navigable()));
    assert_eq!(
        capture.windows(),
        &[top.global(), shadow_global, nested_global, ordinary_global]
    );
    assert!(!capture.windows().contains(&removed_global));
    assert!(!capture.windows().contains(&unrelated.global()));
    drop(capture);
    // The capture alone attests no effect. Execute the actual native timestamp
    // assignment and its Source receipt before completing this method.
    hosted
        .service_close_watcher_request(&mut lease, &consume, &cancel())
        .unwrap();
    let DomSourceProgress::Request(DomSourceRequest::CloseWatcher(settle)) =
        lease.advance(&cancel()).unwrap()
    else {
        panic!("real history effect must hand off invocation");
    };
    assert_eq!(settle.step(), DomSourceCloseWatcherStep::SettleInvocation);
    hosted
        .service_close_watcher_request(&mut lease, &settle, &cancel())
        .unwrap();
    assert!(matches!(
        lease.advance(&cancel()).unwrap(),
        DomSourceProgress::Complete(_)
    ));
    assert!(
        owner
            .borrow()
            .hosts
            .iter()
            .flat_map(|host| &host.top_levels)
            .filter_map(|composition| composition.group.as_ref())
            .flat_map(|group| &group.clusters)
            .flat_map(|cluster| &cluster.agent.windows.rows)
            .all(|window| !window
                .custody
                .iter()
                .any(|custody| matches!(custody, WatcherCustody::Invocation(_))))
    );
}
struct RetirementContext {
    editing: DomSourceRuntime,
    calls: Vec<&'static str>,
}
#[test]
fn retired_nested_close_cleanup_parks_at_settle_and_source_tail_without_replay() {
    let (mut session, top, _, subject) = fixture();
    drop(apply(
        &mut session,
        DomSourceEdit::set_attribute(subject, "open", ""),
    ));
    let hosted = session.hosted_execution();
    let owner = hosted.state().unwrap();
    let original = {
        let state = owner.borrow();
        Rc::downgrade(
            &state.hosts[0].top_levels[0]
                .group
                .as_ref()
                .unwrap()
                .clusters[0]
                .agent
                .windows
                .rows[0]
                .groups[0]
                .members[0],
        )
    };
    let watcher = original.upgrade().unwrap().borrow().id;
    let mut context = RetirementContext {
        editing: session.source_execution(),
        calls: Vec::new(),
    };
    let mut registry = dispatcher::<RetirementContext>();
    registry
        .add_listener(
            &session.dom(),
            subject,
            DomEventType::BeforeToggle,
            DomEventCallback::new(move |dom, _, context: &mut RetirementContext, event| {
                context.calls.push("first");
                assert!(!event.cancelable());
                let marker = dom.snapshot().incomplete_source_operation();
                drop(complete(
                    context
                        .editing
                        .apply(
                            dom,
                            DomSourceEdit::remove_attribute(subject, "open"),
                            source_limits(),
                            &mut DomSourceServices {
                                execution_scope: None,
                                style_policy: &mut |_| None,
                                url: None,
                            },
                            &cancel(),
                        )
                        .unwrap(),
                ));
                let mut batch = DomBatch::new();
                batch.detach(subject);
                batch.destroy(subject);
                dom.apply(batch, &cancel()).unwrap();
                assert_eq!(dom.snapshot().incomplete_source_operation(), marker);
                Ok(())
            }),
            DomEventListenerOptions::default(),
        )
        .unwrap();
    let late = registry
        .add_listener(
            &session.dom(),
            subject,
            DomEventType::BeforeToggle,
            DomEventCallback::new(move |_, _, context: &mut RetirementContext, event| {
                context.calls.push("late");
                assert_eq!(event.target(), Some(subject));
                Ok(())
            }),
            DomEventListenerOptions {
                capture: false,
                once: true,
            },
        )
        .unwrap();
    let cancel_request = start_request(&mut session, subject, Some("unperformed result".into()));
    let receipt = service(&mut session, &cancel_request, &mut registry, &mut context);
    assert!(!receipt.default_prevented());
    assert!(receipt.failures().is_empty());
    drop(receipt);
    let DomSourceDriveProgress::Pending {
        request: DomSourceRequest::Event(before),
        ..
    } = resume(&mut session, cancel_request.operation()).unwrap()
    else {
        panic!("real nested Close must fire beforetoggle");
    };
    let receipt = service(&mut session, &before, &mut registry, &mut context);
    assert_eq!(receipt.callbacks(), 2);
    assert!(receipt.failures().is_empty());
    assert!(!receipt.default_prevented());
    assert!(!registry.remove_listener(late));
    drop(receipt);
    assert_eq!(context.calls, ["first", "late"]);
    assert!(
        !original
            .upgrade()
            .unwrap()
            .borrow()
            .state
            .running_cancel_action()
    );
    let cause = DomSourceCleanupCause::ReceiverRetired {
        subject,
        action: DomSourceStoppedAction::Dialog(HtmlDialogAction::ReadOpen),
    };
    let settle = {
        let mut dom = session.dom();
        let mut lease = dom
            .raw_mut()
            .resume_source_edit(before.operation(), source_limits(), &cancel())
            .unwrap();
        let DomSourceProgress::Request(DomSourceRequest::CloseWatcher(settle)) =
            lease.advance(&cancel()).unwrap()
        else {
            panic!("real nested ReadOpen failure must retain actual cleanup");
        };
        assert_eq!(settle.step(), DomSourceCloseWatcherStep::SettleInvocation);
        assert_eq!(settle.watcher(), watcher);
        assert_eq!(settle.cleanup_cause(), Some(cause));
        assert_eq!(lease.status().phase(), DomSourcePhase::PendingCleanup);
        assert_eq!(lease.status().cleanup_cause(), Some(cause));
        assert_eq!(
            lease.snapshot().incomplete_source_operation(),
            Some(before.operation())
        );
        let stamp = lease.snapshot().stamp();
        {
            let mut state = owner.borrow_mut();
            state.limits.work = state.work;
        }
        assert!(matches!(
            hosted.service_close_watcher_request(&mut lease, &settle, &cancel()),
            Err(DomHostedRuntimeError::Admission)
        ));
        assert_eq!(lease.snapshot().stamp(), stamp);
        assert_eq!(lease.status().cleanup_cause(), Some(cause));
        let record = original
            .upgrade()
            .expect("refused cleanup retains the exact original record");
        assert_eq!(record.borrow().id, watcher);
        assert!(!record.borrow().state.running_cancel_action());
        let state = owner.borrow();
        let window = &state.hosts[0].top_levels[0]
            .group
            .as_ref()
            .unwrap()
            .clusters[0]
            .agent
            .windows
            .rows[0];
        assert!(window.groups.is_empty());
        assert_eq!(window.custody.len(), 1);
        assert!(
            matches!(&window.custody[0], WatcherCustody::Invocation(invocation)
            if invocation.body.binding == settle.invocation() && !invocation.body.running_guard && Rc::ptr_eq(&invocation.body.record, &record))
        );
        drop(state);
        drop(record);
        let DomSourceProgress::Request(DomSourceRequest::CloseWatcher(same)) =
            lease.advance(&cancel()).unwrap()
        else {
            panic!("refused real SettleInvocation remains pending");
        };
        assert_eq!(same, settle);
        settle
    };
    // Reacquire weak permissions and raise the actual existing Runtime policy.
    // No dispatch, watcher reset or membership action is replayed.
    let mut replacement = session.hosted_execution();
    replacement
        .set_limits(&mut session.dom(), session_limits().hosted, &cancel())
        .unwrap();
    {
        let mut dom = session.dom();
        let mut lease = dom
            .raw_mut()
            .resume_source_edit(settle.operation(), source_limits(), &cancel())
            .unwrap();
        replacement
            .service_close_watcher_request(&mut lease, &settle, &cancel())
            .unwrap();
        assert_eq!(lease.status().cleanup_cause(), Some(cause));
        assert_eq!(
            lease.snapshot().incomplete_source_operation(),
            Some(settle.operation())
        );
    }
    assert!(
        original.upgrade().is_none(),
        "authentic native settlement releases its last original record"
    );
    assert!(
        owner.borrow().hosts[0].top_levels[0]
            .group
            .as_ref()
            .unwrap()
            .clusters[0]
            .agent
            .windows
            .rows[0]
            .custody
            .is_empty()
    );
    // Calibrate only the canonical resume producer's real, callback-free
    // admission prelude. The next call receives exactly that prelude's work;
    // its actual finalization must then refuse before Source publication.
    let resume_work = {
        let before = session.dom().pending_source_edit().unwrap().spent_work();
        let mut dom = session.dom();
        let lease = dom
            .raw_mut()
            .resume_source_edit(settle.operation(), source_limits(), &cancel())
            .unwrap();
        lease.status().spent_work() - before
    };
    let spent = session.dom().pending_source_edit().unwrap().spent_work();
    let cap = spent.checked_add(resume_work).unwrap();
    let stamp = session.snapshot().stamp();
    {
        let mut dom = session.dom();
        let mut lease = dom
            .raw_mut()
            .resume_source_edit(settle.operation(), work_limit(cap), &cancel())
            .unwrap();
        assert_eq!(lease.status().spent_work(), cap);
        assert!(
            matches!(lease.advance(&cancel()), Err(DomError::SourceWorkLimit {limit}) if limit == cap)
        );
        assert_eq!(lease.snapshot().stamp(), stamp);
        assert_eq!(
            lease.snapshot().incomplete_source_operation(),
            Some(settle.operation())
        );
        assert_eq!(lease.status().phase(), DomSourcePhase::PendingCleanup);
        assert_eq!(lease.status().cleanup_cause(), Some(cause));
    }
    assert_eq!(context.calls, ["first", "late"]);
    assert!(original.upgrade().is_none());
    let terminal = {
        let mut dom = session.dom();
        let mut lease = dom
            .raw_mut()
            .resume_source_edit(settle.operation(), source_limits(), &cancel())
            .unwrap();
        lease.advance(&cancel())
    };
    assert!(
        matches!(&terminal, Err(DomError::SourceReceiverRetired {operation, subject: actual, action: HtmlDialogAction::ReadOpen})
        if *operation == settle.operation() && *actual == subject),
        "actual retired Source terminal: {terminal:?}"
    );
    assert!(session.dom().pending_source_edit().is_none());
    assert_eq!(session.snapshot().incomplete_source_operation(), None);
    assert_eq!(context.calls, ["first", "late"]);
    assert_eq!(
        session
            .element_task_execution()
            .pending_total_tasks()
            .unwrap(),
        0
    );
    let queued = replacement
        .destroy_top_level(&mut session.dom(), top.navigable(), &cancel())
        .unwrap();
    let mut completed = false;
    for _ in 0..32 {
        if matches!(replacement.run_next(&mut session.dom(), queued.traversable(), &cancel()).unwrap(), DomHostedTaskProgress::Complete {operation} if operation == queued.operation())
        {
            completed = true;
            break;
        }
    }
    assert!(completed);
    replacement
        .destroy_host(&mut session.dom(), top.host(), &cancel())
        .unwrap();
    assert!(matches!(
        session.try_shutdown(),
        DomShutdownOutcome::Complete
    ));
}
fn first_native_request() -> (
    DomSession,
    HostedRuntime,
    NodeId,
    DomSourceCloseWatcherRequest,
) {
    let (mut session, _, _, subject) = fixture();
    drop(apply(
        &mut session,
        DomSourceEdit::set_attribute(subject, "open", ""),
    ));
    let request = {
        let mut dom = session.dom();
        let mut lease = dom
            .raw_mut()
            .begin_source_edit(
                DomSourceEdit::request_close_dialog(subject, None),
                source_limits(),
                &cancel(),
            )
            .unwrap();
        let DomSourceProgress::Request(DomSourceRequest::CloseWatcher(request)) =
            lease.advance(&cancel()).unwrap()
        else {
            panic!("actual request prefix must reach original watcher");
        };
        assert_eq!(
            request.step(),
            DomSourceCloseWatcherStep::Action(HtmlCloseWatcherRequestAction::ReadActive)
        );
        request
    };
    let hosted = session.hosted_execution();
    (session, hosted, subject, request)
}
fn finish_canceled_native_request(
    session: &mut DomSession,
    subject: NodeId,
    operation: DomSourceOperationId,
) {
    let progress = resume(session, operation).unwrap();
    let DomSourceDriveProgress::Pending {
        request: DomSourceRequest::Event(request),
        ..
    } = progress
    else {
        panic!("real watcher must dispatch actual Cancel: {progress:?}");
    };
    let mut registry = dispatcher::<()>();
    registry
        .add_listener(
            &session.dom(),
            subject,
            DomEventType::Cancel,
            DomEventCallback::new(|_, _, _, event| {
                event.prevent_default();
                Ok(())
            }),
            DomEventListenerOptions::default(),
        )
        .unwrap();
    let receipt = service(session, &request, &mut registry, &mut ());
    assert!(receipt.default_prevented());
    assert!(receipt.failures().is_empty());
    drop(receipt);
    drop(complete(resume(session, operation).unwrap()));
    assert!(session.dom().pending_source_edit().is_none());
}
#[test]
fn invocation_box_body_storage_and_scratch_one_short_preserve_request_until_real_admission() {
    for scratch_short in [false, true] {
        let (mut session, hosted, subject, request) = first_native_request();
        let owner = hosted.state().unwrap();
        let baseline = owner.borrow_mut().retained_bytes(&cancel()).unwrap();
        let bank = size_of::<WatcherCustody>();
        let body = invocation_body_bytes();
        {
            let mut state = owner.borrow_mut();
            state.limits.storage_bytes = baseline + bank + body - usize::from(!scratch_short);
            state.limits.scratch_bytes = body - usize::from(scratch_short);
        }
        let basis = session.snapshot().stamp();
        {
            let mut dom = session.dom();
            let mut lease = dom
                .raw_mut()
                .resume_source_edit(request.operation(), source_limits(), &cancel())
                .unwrap();
            assert!(matches!(
                hosted.service_close_watcher_request(&mut lease, &request, &cancel()),
                Err(DomHostedRuntimeError::Admission)
            ));
            assert_eq!(lease.snapshot().stamp(), basis);
            let state = owner.borrow();
            let window = &state.hosts[0].top_levels[0]
                .group
                .as_ref()
                .unwrap()
                .clusters[0]
                .agent
                .windows
                .rows[0];
            assert!(window.custody.is_empty());
            assert_eq!(window.custody.capacity(), 1);
            assert_eq!(window.groups[0].members[0].borrow().id, request.watcher());
            assert!(
                !window.groups[0].members[0]
                    .borrow()
                    .state
                    .running_cancel_action()
            );
            drop(state);
            let DomSourceProgress::Request(DomSourceRequest::CloseWatcher(same)) =
                lease.advance(&cancel()).unwrap()
            else {
                panic!("unformed body cannot consume the actual request");
            };
            assert_eq!(same, request);
        }
        {
            let mut state = owner.borrow_mut();
            state.limits.storage_bytes = baseline + bank + body;
            state.limits.scratch_bytes = body;
        }
        {
            let mut dom = session.dom();
            let mut lease = dom
                .raw_mut()
                .resume_source_edit(request.operation(), source_limits(), &cancel())
                .unwrap();
            hosted
                .service_close_watcher_request(&mut lease, &request, &cancel())
                .unwrap();
            let state = owner.borrow();
            let window = &state.hosts[0].top_levels[0]
                .group
                .as_ref()
                .unwrap()
                .clusters[0]
                .agent
                .windows
                .rows[0];
            assert_eq!(window.custody.len(), 1);
            let WatcherCustody::Invocation(invocation) = &window.custody[0] else {
                panic!("admitted body must retain original invocation");
            };
            assert_eq!(invocation.body.binding, request.invocation());
            assert_eq!(invocation.body.record.borrow().id, request.watcher());
        }
        assert_eq!(
            owner.borrow_mut().retained_bytes(&cancel()).unwrap(),
            baseline + bank + body
        );
        // Subsequent actual history capture has its own Source retention. Restore
        // this fixture's real policy before driving that independent native tail.
        owner.borrow_mut().limits = session_limits().hosted;
        finish_canceled_native_request(&mut session, subject, request.operation());
        assert_eq!(
            owner.borrow_mut().retained_bytes(&cancel()).unwrap(),
            baseline + bank,
            "settlement releases the actual Box while charged spare bank capacity remains"
        );
        assert!(
            owner.borrow().hosts[0].top_levels[0]
                .group
                .as_ref()
                .unwrap()
                .clusters[0]
                .agent
                .windows
                .rows[0]
                .custody
                .is_empty()
        );
    }
}
#[test]
fn invocation_box_body_formation_work_is_admitted_before_allocation_and_link() {
    for short in [true, false] {
        let (mut session, hosted, subject, request) = first_native_request();
        let owner = hosted.state().unwrap();
        // Before the first body exists: Window lookup4 + retained scans8
        // (Host/composition/cluster/Window/group/memberbody/twoSettings) +
        // membership lookup2 + empty custody-bank growth1 =15. Its actual
        // body formation/link requires one further owned step before Box::new.
        const FORMATION_PREFIX: u64 = 4 + 8 + 2 + 1 + 1;
        let before = owner.borrow().work;
        {
            let mut state = owner.borrow_mut();
            state.limits.work = before + FORMATION_PREFIX - u64::from(short);
        }
        let basis = session.snapshot().stamp();
        {
            let mut dom = session.dom();
            let mut lease = dom
                .raw_mut()
                .resume_source_edit(request.operation(), source_limits(), &cancel())
                .unwrap();
            // Exact formation still has no budget for the following ReadActive
            // traversal. Its actual body must survive that later refusal.
            assert!(matches!(
                hosted.service_close_watcher_request(&mut lease, &request, &cancel()),
                Err(DomHostedRuntimeError::Admission)
            ));
            assert_eq!(lease.snapshot().stamp(), basis);
            assert_eq!(
                owner.borrow().work - before,
                FORMATION_PREFIX - u64::from(short)
            );
            let state = owner.borrow();
            let window = &state.hosts[0].top_levels[0]
                .group
                .as_ref()
                .unwrap()
                .clusters[0]
                .agent
                .windows
                .rows[0];
            assert_eq!(window.custody.len(), usize::from(!short));
            assert!(
                !window.groups[0].members[0]
                    .borrow()
                    .state
                    .running_cancel_action()
            );
            if !short {
                let WatcherCustody::Invocation(invocation) = &window.custody[0] else {
                    panic!("actual admitted body owns custody");
                };
                assert_eq!(invocation.body.binding, request.invocation());
                assert_eq!(invocation.body.record.borrow().id, request.watcher());
                assert!(invocation.body.pending.is_none());
            }
        }
        owner.borrow_mut().limits.work = session_limits().hosted.work;
        {
            let mut dom = session.dom();
            let mut lease = dom
                .raw_mut()
                .resume_source_edit(request.operation(), source_limits(), &cancel())
                .unwrap();
            hosted
                .service_close_watcher_request(&mut lease, &request, &cancel())
                .unwrap();
            assert_eq!(
                owner.borrow().hosts[0].top_levels[0]
                    .group
                    .as_ref()
                    .unwrap()
                    .clusters[0]
                    .agent
                    .windows
                    .rows[0]
                    .custody
                    .len(),
                1
            );
        }
        finish_canceled_native_request(&mut session, subject, request.operation());
        assert!(
            owner.borrow().hosts[0].top_levels[0]
                .group
                .as_ref()
                .unwrap()
                .clusters[0]
                .agent
                .windows
                .rows[0]
                .custody
                .is_empty()
        );
    }
}
