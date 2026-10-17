//! Actual watcher-service admission for the standard ShowModal prefix.
use super::*;
use crate::{
    DomEventCallback, DomEventListenerOptions, DomEventType, DomSourceEventServiceLimits,
    DomSourceEventServiceProgress,
};

fn cancel() -> AtomicBool {
    AtomicBool::new(false)
}
fn begin(session: &mut DomSession, subject: NodeId) -> DomSourceEventRequest {
    let progress = session
        .source_execution()
        .apply(
            &mut session.dom(),
            DomSourceEdit::show_modal_dialog(subject),
            source_limits(),
            &mut DomSourceServices {
                execution_scope: None,
                style_policy: &mut |_| None,
                url: None,
            },
            &cancel(),
        )
        .unwrap();
    match progress {
        DomSourceDriveProgress::Pending {
            request: DomSourceRequest::Event(request),
            ..
        } => request,
        other => panic!("real ShowModal beforetoggle required: {other:?}"),
    }
}
fn dispatch<C>(
    session: &mut DomSession,
    request: &DomSourceEventRequest,
    registry: &mut crate::DomEventDispatcher<C>,
    context: &mut C,
) {
    let result = session
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
        .unwrap();
    match result {
        DomSourceEventServiceProgress::Serviced { receipt } => {
            assert!(!receipt.default_prevented());
            assert!(receipt.failures().is_empty());
        }
        other => panic!("actual completed beforetoggle required: {other:?}"),
    }
}
#[test]
fn show_modal_watcher_refusal_and_lease_reacquisition_preserve_prefix_and_settle_exact_record() {
    let (mut session, top, _, subject) = fixture();
    let mut registry = dispatcher::<usize>();
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
    let before = begin(&mut session, subject);
    let mut calls = 0;
    dispatch(&mut session, &before, &mut registry, &mut calls);
    let hosted = session.hosted_execution();
    let tasks = session.element_task_execution();
    let owner = hosted.state().unwrap();
    let watcher_request = {
        let mut dom = session.dom();
        let mut lease = dom
            .raw_mut()
            .resume_source_edit(before.operation(), source_limits(), &cancel())
            .unwrap();
        let queued = pending(&mut lease);
        assert!(matches!(
            queued.action(),
            HtmlDialogAction::QueueToggle { .. }
        ));
        tasks.queue_dialog(&mut lease, &queued, &cancel()).unwrap();
        let request = pending(&mut lease);
        assert_eq!(request.action(), HtmlDialogAction::EstablishCloseWatcher);
        assert_eq!(
            lease
                .snapshot()
                .node(subject)
                .unwrap()
                .attribute(&"open".into()),
            Some(&"".into())
        );
        assert!(
            !lease
                .snapshot()
                .node(subject)
                .unwrap()
                .as_dialog()
                .unwrap()
                .has_close_watcher()
        );
        let stamp = lease.snapshot().stamp();
        let spent = owner.borrow().work;
        owner.borrow_mut().limits.work = spent;
        assert!(matches!(
            hosted.service_dialog_watcher(&mut lease, &request, &cancel()),
            Err(DomHostedRuntimeError::Admission)
        ));
        assert_eq!(lease.snapshot().stamp(), stamp);
        assert_eq!(pending(&mut lease), request);
        assert_eq!(counts(&hosted), (1, 0, 0));
        request
    };
    assert_eq!(calls, 1);
    assert_eq!(tasks.pending_total_tasks().unwrap(), 1);
    let task = tasks
        .pending_task(DomElementTaskSourceKind::DomManipulation, 0)
        .unwrap()
        .unwrap();
    owner.borrow_mut().limits.work = session_limits().hosted.work;
    {
        let mut dom = session.dom();
        let mut lease = dom
            .raw_mut()
            .resume_source_edit(before.operation(), source_limits(), &cancel())
            .unwrap();
        assert!(matches!(
            hosted.service_dialog_watcher(&mut lease, &watcher_request, &AtomicBool::new(true)),
            Err(DomHostedRuntimeError::Cancelled
                | DomHostedRuntimeError::Dom(DomHostedError::Dom(DomError::Cancelled)))
        ));
        assert_eq!(pending(&mut lease), watcher_request);
        assert!(
            hosted
                .service_dialog_watcher(&mut lease, &watcher_request, &cancel())
                .unwrap()
        );
        assert_eq!(counts(&hosted), (1, 1, 1));
        // Runtime owns the real registration receipt until canonical readback;
        // dropping this lease does not repeat establishment on reacquisition.
        assert!(
            !lease
                .snapshot()
                .node(subject)
                .unwrap()
                .as_dialog()
                .unwrap()
                .has_close_watcher()
        );
    }
    drop(hosted);
    let progress = session
        .source_execution()
        .resume(
            &mut session.dom(),
            before.operation(),
            source_limits(),
            &mut DomSourceServices {
                execution_scope: None,
                style_policy: &mut |_| None,
                url: None,
            },
            &cancel(),
        )
        .unwrap();
    assert!(
        matches!(progress,DomSourceDriveProgress::Pending{request:DomSourceRequest::Dialog(request),reason:crate::DomSourceUnavailable::Dialog(HtmlDialogAction::SetModal)} if request.action()==HtmlDialogAction::SetModal)
    );
    let snapshot = session.snapshot();
    let dialog = snapshot.node(subject).unwrap().as_dialog().unwrap();
    let id = dialog.close_watcher().unwrap();
    assert_eq!(id.global(), top.global());
    assert_eq!(id.subject(), subject);
    assert!(!dialog.is_modal());
    let state = owner.borrow();
    let window = &state.hosts[0].top_levels[0]
        .group
        .as_ref()
        .unwrap()
        .clusters[0]
        .agent
        .windows
        .rows[0];
    assert_eq!(window.groups.len(), 1);
    assert_eq!(window.groups[0].members.len(), 1);
    assert!(window.custody.is_empty());
    assert_eq!(window.groups[0].members[0].borrow().id, id);
    assert_eq!(
        window.groups[0].members[0].borrow().phase,
        WatcherPhase::Registered
    );
    assert_eq!(calls, 1);
    assert_eq!(
        tasks
            .pending_task(DomElementTaskSourceKind::DomManipulation, 0)
            .unwrap(),
        Some(task)
    );
}

fn work_limit(work: u64) -> DomSourceLimits {
    let base = source_limits();
    DomSourceLimits::new(
        base.retained_bytes(),
        base.retained_string_code_units(),
        work,
        base.phase_publications(),
        base.output_records(),
        base.published_native_identities(),
        base.nested_operations(),
    )
}

fn original_window(state: &HostedState, global: DomHostedGlobalId) -> &NativeWindow {
    state
        .hosts
        .iter()
        .flat_map(|host| &host.top_levels)
        .filter_map(|composition| composition.group.as_ref())
        .flat_map(|group| &group.clusters)
        .flat_map(|cluster| &cluster.agent.windows.rows)
        .find(|window| window.id == global)
        .expect("actual original Global must retain its Window")
}
fn assert_original_managers(
    hosted: &HostedRuntime,
    a: DomHostedGlobalId,
    b: DomHostedGlobalId,
    sentinel: DomCloseWatcherId,
    original: &std::rc::Weak<RefCell<Watcher>>,
) {
    let owner = hosted.state().unwrap();
    let state = owner.borrow();
    let active = original_window(&state, a);
    assert!(!active.retired);
    assert!(active.groups.is_empty());
    assert!(active.custody.is_empty());
    let retired = original_window(&state, b);
    assert!(retired.retired);
    assert_eq!(retired.groups.len(), 1);
    assert_eq!(retired.groups[0].members.len(), 1);
    assert!(retired.custody.is_empty());
    let original = original
        .upgrade()
        .expect("B's actual manager retains its original sentinel record");
    assert!(Rc::ptr_eq(&retired.groups[0].members[0], &original));
    let record = original.borrow();
    assert_eq!(record.id, sentinel);
    assert_eq!(record.phase, WatcherPhase::Registered);
    assert!(!record.state.running_cancel_action());
}

#[test]
fn inactive_adoption_assertion_retains_real_cleanup_cause_across_admission_drop_and_cancellation() {
    let (mut session, a, _, subject) = fixture();
    let mut hosted = session.hosted_execution();
    let b = hosted
        .create_top_level(&mut session.dom(), a.host(), &cancel())
        .unwrap();
    let snapshot = session.snapshot();
    let html = snapshot.node(b.document().node()).unwrap().children()[0];
    let body_b = snapshot.node(html).unwrap().children()[1];
    let mut batch = DomBatch::new();
    let sentinel = batch.create_element_ns(b.document(), Namespace::html(), "dialog");
    batch.insert_before(&sentinel, body_b, None);
    let sentinel = session
        .dom()
        .apply(batch, &cancel())
        .unwrap()
        .created(&sentinel)
        .unwrap()
        .unwrap();
    drop(apply(
        &mut session,
        DomSourceEdit::set_attribute(sentinel, "open", ""),
    ));
    let sentinel_watcher = session
        .snapshot()
        .node(sentinel)
        .unwrap()
        .as_dialog()
        .unwrap()
        .close_watcher()
        .unwrap();
    let destruction = hosted
        .destroy_top_level(&mut session.dom(), b.navigable(), &cancel())
        .unwrap();
    let mut retired = false;
    for _ in 0..32 {
        if matches!(hosted.run_next(&mut session.dom(),destruction.traversable(),&cancel()).unwrap(),DomHostedTaskProgress::Complete{operation} if operation==destruction.operation())
        {
            retired = true;
            break;
        }
    }
    assert!(retired);
    assert!(
        !session
            .snapshot()
            .document(b.document())
            .unwrap()
            .is_fully_active()
            .unwrap()
    );
    // No-opener top levels have separate original Agents/Window banks. Capture
    // B's actual preexisting record weakly, rather than treating A's bank as an
    // aggregate or introducing a test owner that could mask lost custody.
    let sentinel_record = {
        let owner = hosted.state().unwrap();
        let state = owner.borrow();
        Rc::downgrade(&original_window(&state, b.global()).groups[0].members[0])
    };
    assert_original_managers(
        &hosted,
        a.global(),
        b.global(),
        sentinel_watcher,
        &sentinel_record,
    );
    let mut registry = dispatcher::<usize>();
    registry
        .add_listener(
            &session.dom(),
            subject,
            DomEventType::BeforeToggle,
            DomEventCallback::new(move |dom, _, calls: &mut usize, _| {
                *calls += 1;
                let mut batch = DomBatch::new();
                batch.insert_before(subject, body_b, None);
                dom.apply(batch, &cancel()).unwrap();
                Ok(())
            }),
            DomEventListenerOptions {
                capture: false,
                once: true,
            },
        )
        .unwrap();
    let before = begin(&mut session, subject);
    let mut calls = 0;
    dispatch(&mut session, &before, &mut registry, &mut calls);
    let tasks = session.element_task_execution();
    {
        let mut dom = session.dom();
        let mut lease = dom
            .raw_mut()
            .resume_source_edit(before.operation(), source_limits(), &cancel())
            .unwrap();
        let queued = pending(&mut lease);
        assert!(matches!(
            queued.action(),
            HtmlDialogAction::QueueToggle { .. }
        ));
        tasks.queue_dialog(&mut lease, &queued, &cancel()).unwrap();
    }
    let task = tasks
        .pending_task(DomElementTaskSourceKind::DomManipulation, 0)
        .unwrap()
        .unwrap();
    assert_eq!(task.queue_association().unwrap().global(), a.global());
    assert_eq!(task.queue_association().unwrap().document(), a.document());
    // Calibrate only the actual canonical resume admission prelude, without
    // advancing a method, changing counters artificially, or sampling a tariff.
    let resume_work = {
        let spent = session.dom().pending_source_edit().unwrap().spent_work();
        let mut dom = session.dom();
        let lease = dom
            .raw_mut()
            .resume_source_edit(before.operation(), source_limits(), &cancel())
            .unwrap();
        lease.status().spent_work() - spent
    };
    // This inactive-hook trace has two retained publication states: the queued
    // Toggle, then completed AddOpen/assertion basis. Give each state a bounded
    // one-step caller-policy sequence so an expensive earlier segment cannot
    // carry excess allowance through the shorter terminal tail. This locates
    // actual admission phases; it does not assert a numeric work tariff.
    let mut found = None;
    let queue_stamp = session.snapshot().stamp();
    let queue_publications = session
        .dom()
        .pending_source_edit()
        .unwrap()
        .published_phases();
    let mut prefix_published = false;
    'search: for prefix_state in 0..2 {
        for allowance in 1..=2048 {
            let spent = session.dom().pending_source_edit().unwrap().spent_work();
            let cap = spent
                .checked_add(resume_work)
                .unwrap()
                .checked_add(allowance)
                .unwrap();
            let error = {
                let mut dom = session.dom();
                match dom.raw_mut().resume_source_edit(
                    before.operation(),
                    work_limit(cap),
                    &cancel(),
                ) {
                    Ok(mut lease) => lease.advance(&cancel()).unwrap_err(),
                    Err(error) => error,
                }
            };
            assert!(
                matches!(error,DomError::SourceWorkLimit{limit} if limit==cap),
                "only the actual caller work refusal may park this operation: {error:?}"
            );
            let status = session
                .dom()
                .pending_source_edit()
                .expect("refusal retains exact Source guard");
            assert_eq!(status.operation(), before.operation());
            assert_eq!(calls, 1);
            assert_eq!(
                tasks
                    .pending_task(DomElementTaskSourceKind::DomManipulation, 0)
                    .unwrap(),
                Some(task)
            );
            if let Some(cause) = status.cleanup_cause() {
                found = Some(cause);
                break 'search;
            }
            if prefix_state == 0 && status.published_phases() != queue_publications {
                assert_eq!(status.published_phases(), queue_publications + 1);
                let snapshot = session.snapshot();
                assert_ne!(snapshot.stamp(), queue_stamp);
                let node = snapshot.node(subject).unwrap();
                assert_eq!(node.attribute(&"open".into()), Some(&"".into()));
                assert!(!node.as_dialog().unwrap().has_close_watcher());
                assert_eq!(
                    snapshot.incomplete_source_operation(),
                    Some(before.operation())
                );
                prefix_published = true;
                break;
            }
            assert_eq!(status.published_phases(), queue_publications + prefix_state);
        }
        assert!(
            prefix_published,
            "bounded admission must reach the actual completed AddOpen publication"
        );
    }
    let cause = found.expect(
        "bounded real caller admission must expose frozen failure before terminal publication",
    );
    assert_eq!(
        session
            .dom()
            .pending_source_edit()
            .unwrap()
            .published_phases(),
        queue_publications + 1
    );
    let DomSourceCleanupCause::InvariantFailed(failure) = cause else {
        panic!("actual missing watcher must retain invariant cause: {cause:?}");
    };
    assert_eq!(
        failure.dialog_operation(),
        surgeist_html::HtmlDialogOperation::ShowModal
    );
    assert_eq!(failure.subject(), subject);
    assert_eq!(
        failure.action(),
        HtmlDialogAction::AssertCloseWatcherPresent
    );
    assert_eq!(
        failure.reason(),
        DomSourceInvariantReason::MissingDialogCloseWatcher
    );
    assert_eq!(failure.basis(), session.snapshot().stamp());
    assert_eq!(
        session.dom().pending_source_edit().unwrap().phase(),
        DomSourcePhase::PendingCleanup
    );
    assert_eq!(
        session.snapshot().incomplete_source_operation(),
        Some(before.operation())
    );
    let snapshot = session.snapshot();
    let node = snapshot.node(subject).unwrap();
    assert_eq!(node.attribute(&"open".into()), Some(&"".into()));
    assert_eq!(node.owner_document(), Some(b.document()));
    assert!(!node.as_dialog().unwrap().is_modal());
    assert!(!node.as_dialog().unwrap().has_close_watcher());
    assert_eq!(
        snapshot
            .node(sentinel)
            .unwrap()
            .as_dialog()
            .unwrap()
            .close_watcher(),
        Some(sentinel_watcher)
    );
    assert_original_managers(
        &hosted,
        a.global(),
        b.global(),
        sentinel_watcher,
        &sentinel_record,
    );
    let stamp = snapshot.stamp();
    {
        let mut dom = session.dom();
        assert!(matches!(
            dom.raw_mut().resume_source_edit(
                before.operation(),
                source_limits(),
                &AtomicBool::new(true)
            ),
            Err(DomError::Cancelled)
        ));
    }
    assert_eq!(session.snapshot().stamp(), stamp);
    assert_eq!(
        session.dom().pending_source_edit().unwrap().cleanup_cause(),
        Some(cause)
    );
    {
        let work =
            session.dom().pending_source_edit().unwrap().spent_work() + source_limits().work();
        let mut dom = session.dom();
        let mut lease = dom
            .raw_mut()
            .resume_source_edit(before.operation(), work_limit(work), &cancel())
            .unwrap();
        lease.request_cancel();
        assert_eq!(lease.status().cleanup_cause(), Some(cause));
    }
    // Dropping/replacing weak execution permission cannot own or clear failure.
    drop(hosted);
    drop(session.source_execution());
    let work = session.dom().pending_source_edit().unwrap().spent_work() + source_limits().work();
    let terminal = session.source_execution().resume(
        &mut session.dom(),
        before.operation(),
        work_limit(work),
        &mut DomSourceServices {
            execution_scope: None,
            style_policy: &mut |_| None,
            url: None,
        },
        &cancel(),
    );
    assert!(
        matches!(terminal,Err(crate::DomSourceRuntimeError::Dom(DomError::SourceInvariantFailed{operation,failure:actual})) if operation==before.operation() && actual==failure)
    );
    assert!(session.dom().pending_source_edit().is_none());
    assert_eq!(session.snapshot().incomplete_source_operation(), None);
    assert_eq!(calls, 1);
    assert_eq!(tasks.pending_total_tasks().unwrap(), 1);
    assert_eq!(
        tasks
            .pending_task(DomElementTaskSourceKind::DomManipulation, 0)
            .unwrap(),
        Some(task)
    );
    assert_original_managers(
        &session.hosted_execution(),
        a.global(),
        b.global(),
        sentinel_watcher,
        &sentinel_record,
    );
}
