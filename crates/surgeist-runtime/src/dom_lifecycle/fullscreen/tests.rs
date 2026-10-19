//! Real handoff custody under the existing hosted task envelope.
use super::*;
use crate::{
    DomElementTaskRuntimeLimits, DomSession, DomSessionLimits, DomSourceDriveProgress,
    DomSourceEventRuntimeLimits, DomSourceServices, DomStyleRuntimeLimits,
};
use surgeist_dom::{
    DomBatch, DomError, DomFullscreenEventKind, DomFullscreenExitReturn, DomFullscreenExitStep,
    DomHostedLimits, DomHostedRecordLimits, DomLimits, DomSourceEdit, DomSourceLimits,
    DomSourceProgress, DomSourceRequest, Namespace, NodeId,
};

fn limits() -> DomSessionLimits {
    DomSessionLimits {
        source_events: DomSourceEventRuntimeLimits {
            jobs: 4,
            storage_bytes: 16_000_000,
            work: 4_000_000,
        },
        style: DomStyleRuntimeLimits {
            roots: 4,
            completions: 8,
            reports: 8,
            storage_bytes: 16_000_000,
            work: 4_000_000,
            css_input: surgeist_css::CssUtf16InputLimits::default(),
        },
        element_tasks: DomElementTaskRuntimeLimits {
            tasks: 8,
            storage_bytes: 16_000_000,
            work: 4_000_000,
        },
        hosted: DomHostedRuntimeLimits {
            canonical: DomHostedLimits::new(
                DomHostedRecordLimits::new(16, 16, 16, 16, 16, 16, 16),
                32_000_000,
                32_000_000,
                4_000_000,
            ),
            tasks: 8,
            storage_bytes: 32_000_000,
            scratch_bytes: 32_000_000,
            work: 4_000_000,
        },
    }
}

fn source_limits() -> DomSourceLimits {
    DomSourceLimits::new(64_000_000, 1_000_000, 100_000_000, 128, 4096, 4096, 4)
}

fn fixture() -> (DomSession, HostedRuntime, DomHostedTopLevel, NodeId) {
    let cancel = AtomicBool::new(false);
    let mut session = DomSession::new(
        Dom::new(
            DomLimits::new(
                64,
                100_000,
                128,
                128,
                128,
                32,
                4_000_000,
                surgeist_dom::DomNameLimits::new(16_777_216, 16_384, 4_000_000, 1_048_576).unwrap(),
            )
            .unwrap()
            .with_fullscreen_exits(4)
            .unwrap()
            .with_fullscreen_events(4)
            .unwrap(),
        )
        .unwrap(),
        limits(),
    )
    .unwrap();
    let mut hosted = session.hosted_execution();
    let top = {
        let mut dom = session.dom();
        let host = hosted.create_host(&mut dom, &cancel).unwrap();
        hosted.create_top_level(&mut dom, host, &cancel).unwrap()
    };
    let root = session
        .snapshot()
        .document(top.document())
        .unwrap()
        .document_element()
        .unwrap();
    (session, hosted, top, root)
}

fn handoff(session: &mut DomSession, top: DomHostedTopLevel, root: NodeId) -> DomFullscreenExitId {
    let cancel = AtomicBool::new(false);
    let mut batch = DomBatch::new();
    batch.fullscreen_element_model(root);
    session.dom().apply(batch, &cancel).unwrap();
    let DomSourceDriveProgress::Complete(done) = session
        .source_execution()
        .apply(
            &mut session.dom(),
            DomSourceEdit::exit_fullscreen(top.document()),
            source_limits(),
            &mut DomSourceServices {
                execution_scope: None,
                style_policy: &mut |_| None,
                url: None,
            },
            &cancel,
        )
        .unwrap()
    else {
        panic!("the synchronous exit returns its actually queued invocation");
    };
    let Some(DomFullscreenExitReturn::Pending(id)) = done.fullscreen_exit_return() else {
        panic!("a real active fullscreen Document reaches parallel handoff");
    };
    id
}

#[test]
fn parked_invocation_still_consumes_task_capacity_and_prevents_group_pruning() {
    let cancel = AtomicBool::new(false);
    let (mut session, mut hosted, top, root) = fixture();
    let id = handoff(&mut session, top, root);
    assert_eq!(
        hosted
            .run_next(&mut session.dom(), top.navigable(), &cancel)
            .unwrap(),
        DomHostedTaskProgress::FullscreenExitPending {
            invocation: id,
            step: DomFullscreenExitStep::FullyUnlockScreenOrientation,
        }
    );
    let owner = hosted.state().unwrap();
    {
        let mut state = owner.borrow_mut();
        let state = &mut *state;
        assert_eq!(
            native_task_count(&state.hosts, &mut state.work, state.limits.work, &cancel).unwrap(),
            1
        );
        let row = &state.hosts[0].top_levels[0]
            .group
            .as_ref()
            .unwrap()
            .fullscreen
            .rows[0];
        assert!(row.permission.is_some());
        assert!(!row.ready);
    }
    let mut reduced = limits().hosted;
    reduced.tasks = 0;
    assert_eq!(
        hosted.set_limits(&mut session.dom(), reduced, &cancel),
        Err(DomHostedRuntimeError::Admission)
    );
    let mut batch = DomBatch::new();
    batch.unfullscreen_document_model(top.document());
    session.dom().apply(batch, &cancel).unwrap();
    let destruction = hosted
        .destroy_top_level(&mut session.dom(), top.navigable(), &cancel)
        .unwrap();
    let mut destroyed = false;
    for _ in 0..64 {
        if matches!(
            hosted.run_next(&mut session.dom(), top.navigable(), &cancel).unwrap(),
            DomHostedTaskProgress::Complete { operation } if operation == destruction.operation()
        ) {
            destroyed = true;
            break;
        }
    }
    assert!(destroyed, "the actual hosted destruction wrapper settles");
    drop(hosted);
    let mut replacement = session.hosted_execution();
    assert_eq!(
        replacement
            .run_next(&mut session.dom(), top.navigable(), &cancel)
            .unwrap(),
        DomHostedTaskProgress::Quiescent
    );
    assert_eq!(session.dom().fullscreen_exit_status(id).unwrap().id(), id);
}

#[test]
fn separate_calls_retain_separate_permissions_without_holding_the_window_loop() {
    let cancel = AtomicBool::new(false);
    let (mut session, mut hosted, top, root) = fixture();
    let first = handoff(&mut session, top, root);
    let second = handoff(&mut session, top, root);
    assert_ne!(first, second);
    for id in [first, second] {
        assert_eq!(
            hosted
                .run_next(&mut session.dom(), top.navigable(), &cancel)
                .unwrap(),
            DomHostedTaskProgress::FullscreenExitPending {
                invocation: id,
                step: DomFullscreenExitStep::FullyUnlockScreenOrientation,
            }
        );
    }
    assert_eq!(
        hosted
            .run_next(&mut session.dom(), top.navigable(), &cancel)
            .unwrap(),
        DomHostedTaskProgress::Quiescent
    );
    let owner = hosted.state().unwrap();
    let state = owner.borrow();
    let group = state.hosts[0].top_levels[0].group.as_ref().unwrap();
    let event_loop = group.clusters[0].agent.event_loop.borrow();
    assert!(event_loop.current.is_none());
    assert!(!event_loop.executing);
    assert_eq!(event_loop.checkpoints, event_loop.next_checkpoint);
    assert_eq!(group.fullscreen.rows.len(), 2);
}

#[test]
fn dropping_real_prepared_handoff_preserves_the_exposed_request_and_retries_once() {
    let cancel = AtomicBool::new(false);
    let (mut session, mut hosted, top, _) = fixture();
    // A genuine disconnected model target reaches step 7 without removing an
    // active tree node or manufacturing a post-prefix continuation.
    let mut batch = DomBatch::new();
    let target = batch.create_element_ns(top.document(), Namespace::html(), "div");
    batch.fullscreen_element_model(&target);
    let target = session
        .dom()
        .apply(batch, &cancel)
        .unwrap()
        .created(&target)
        .unwrap()
        .unwrap();
    let before = session.snapshot();
    let before_document = before.document(top.document()).unwrap();
    assert_eq!(
        before_document.top_layer().unwrap().raw_members(),
        &[target]
    );
    assert!(
        before_document
            .pending_fullscreen_events()
            .unwrap()
            .is_empty()
    );
    let owner = hosted.state().unwrap();
    let (operation, invocation) = {
        let mut dom = session.dom();
        let mut lease = dom
            .begin_source_edit(
                DomSourceEdit::exit_fullscreen(top.document()),
                source_limits(),
                &cancel,
            )
            .unwrap();
        let operation = lease.operation();
        let DomSourceProgress::Request(DomSourceRequest::FullscreenExitHandoff(request)) =
            lease.advance(&cancel).unwrap()
        else {
            panic!("the actual active invocation requires Runtime custody");
        };
        let invocation = request.invocation();
        {
            // Deliberate private consumer boundary: exercise the actual
            // reservation and prepared loan, then abandon before its public
            // publish fence. No positive service receipt is supplied.
            let mut state = owner.borrow_mut();
            let (h, t, c, g) = state
                .find_global(request.global(), &cancel, || source_work(&lease, &cancel))
                .unwrap();
            assert_eq!(state.hosts[h].id, request.host());
            let group = state.hosts[h].top_levels[t].group.as_ref().unwrap();
            assert_eq!(group.traversal.traversable, Some(request.traversable()));
            assert_eq!(
                group.clusters[c].agent.globals[g].settings.document,
                Some(request.document())
            );
            assert!(group.clusters[c].agent.globals[g].settings.ready);
            let index = state
                .reserve_fullscreen_row(h, t, &mut lease, &request, &cancel)
                .unwrap();
            let work_before_prepare = lease.status().spent_work();
            let prepared = lease
                .prepare_fullscreen_exit_handoff(&request, &state.source, &cancel)
                .unwrap();
            drop(prepared);
            assert!(lease.status().spent_work() > work_before_prepare);
            assert_eq!(lease.snapshot().stamp(), before.stamp());
            let jobs = &mut state.hosts[h].top_levels[t]
                .group
                .as_mut()
                .unwrap()
                .fullscreen;
            assert_eq!(index + 1, jobs.rows.len());
            let reserved = jobs.rows.pop().unwrap();
            assert_eq!(reserved.invocation, invocation);
            assert!(reserved.permission.is_none());
            assert!(!reserved.ready);
            // Consumer abandonment releases the unissued logical row through
            // its original owner; actual spare allocation remains admitted.
            assert!(jobs.rows.is_empty());
            assert!(jobs.retained_bytes().unwrap() > 0);
            let state = &mut *state;
            assert_eq!(
                native_task_count(&state.hosts, &mut state.work, state.limits.work, &cancel)
                    .unwrap(),
                0
            );
        }
        let spent_after_drop = lease.status().spent_work();
        assert_eq!(lease.snapshot().stamp(), before.stamp());
        let snapshot = lease.snapshot();
        let document = snapshot.document(top.document()).unwrap();
        // The retained request qualifies current complete-view reads as
        // incomplete; the prior immutable capture remains complete.
        assert_eq!(
            document.top_layer().unwrap_err(),
            DomError::IncompleteSourceOperation
        );
        assert_eq!(
            document.pending_fullscreen_events().unwrap_err(),
            DomError::IncompleteSourceOperation
        );
        assert_eq!(
            before_document.top_layer().unwrap().raw_members(),
            &[target]
        );
        assert!(
            before_document
                .pending_fullscreen_events()
                .unwrap()
                .is_empty()
        );
        assert!(
            snapshot
                .node(target)
                .unwrap()
                .as_element()
                .unwrap()
                .fullscreen_flag()
        );
        let DomSourceProgress::Request(DomSourceRequest::FullscreenExitHandoff(retry)) =
            lease.advance(&cancel).unwrap()
        else {
            panic!("dropped preparation retains the original source request");
        };
        assert_eq!(retry, request);
        assert!(lease.status().spent_work() > spent_after_drop);
        drop(lease);
        assert_eq!(dom.pending_source_edit().unwrap().operation(), operation);
        assert!(dom.pending_source_edit().unwrap().spent_work() > spent_after_drop);
        assert_eq!(
            dom.fullscreen_exit_status(invocation),
            Err(DomError::StaleFullscreenExit)
        );
        (operation, invocation)
    };
    let DomSourceDriveProgress::Complete(done) = session
        .source_execution()
        .resume(
            &mut session.dom(),
            operation,
            source_limits(),
            &mut DomSourceServices {
                execution_scope: None,
                style_policy: &mut |_| None,
                url: None,
            },
            &cancel,
        )
        .unwrap()
    else {
        panic!("the production adapter secures the same actual invocation once");
    };
    assert_eq!(
        done.fullscreen_exit_return(),
        Some(DomFullscreenExitReturn::Pending(invocation))
    );
    assert!(session.dom().pending_source_edit().is_none());
    let after = session.snapshot();
    let document = after.document(top.document()).unwrap();
    assert!(document.top_layer().unwrap().raw_members().is_empty());
    assert!(
        !after
            .node(target)
            .unwrap()
            .as_element()
            .unwrap()
            .fullscreen_flag()
    );
    let events = document.pending_fullscreen_events().unwrap();
    assert_eq!(events.len(), 1);
    assert_eq!(events[0].document(), top.document());
    assert_eq!(events[0].subject(), target);
    assert_eq!(events[0].kind(), DomFullscreenEventKind::Change);
    {
        let state = owner.borrow();
        let rows = &state.hosts[0].top_levels[0]
            .group
            .as_ref()
            .unwrap()
            .fullscreen
            .rows;
        assert_eq!(rows.len(), 1);
        assert_eq!(rows[0].invocation, invocation);
        assert_eq!(rows[0].permission.as_ref().unwrap().id(), invocation);
        assert!(rows[0].ready);
    }
    assert_eq!(
        hosted
            .run_next(&mut session.dom(), top.navigable(), &cancel)
            .unwrap(),
        DomHostedTaskProgress::FullscreenExitPending {
            invocation,
            step: DomFullscreenExitStep::FullyUnlockScreenOrientation,
        }
    );
    assert_eq!(
        hosted
            .run_next(&mut session.dom(), top.navigable(), &cancel)
            .unwrap(),
        DomHostedTaskProgress::Quiescent
    );
}
