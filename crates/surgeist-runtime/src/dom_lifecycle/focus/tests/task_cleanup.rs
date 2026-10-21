//! Original Hosted task checkpoint composed with actual Source and dispatcher outputs.
use super::*;
use crate::{
    DomElementTaskEventContext, DomElementTaskProgress, DomElementTaskRunLimits,
    DomElementTaskRuntime, DomElementTaskRuntimeError, DomEventCallback, DomEventDom,
    DomEventListenerOptions, DomEventType,
};

struct Cleanup {
    tasks: DomElementTaskRuntime,
    hosted: HostedRuntime,
    calls: usize,
    exhausted_at: Option<u64>,
}
impl DomElementTaskEventContext for Cleanup {
    fn element_tasks(&mut self) -> &mut DomElementTaskRuntime {
        &mut self.tasks
    }
}
fn dispatch_limits() -> DomEventDispatchLimits {
    DomEventDispatchLimits {
        path_items: 32,
        snapshot_listeners: 8,
        failure_records: 8,
        callbacks: 8,
        work: 100_000,
    }
}
#[test]
fn original_toggle_cleanup_result_survives_native_checkpoint_work_refusal_without_dispatch_replay()
{
    let (mut session, host, _) = fixture();
    let cancel = AtomicBool::new(false);
    let mut style = |_: &surgeist_dom::DomStyleSource| None;
    let mut services = DomSourceServices {
        meta: None,
        execution_scope: None,
        style_policy: &mut style,
        url: None,
    };
    let top = session
        .hosted_execution()
        .create_top_level(&mut session.dom(), host, &cancel)
        .unwrap();
    let snapshot = session.snapshot();
    let html = snapshot.node(top.document().node()).unwrap().children()[0];
    let body = snapshot.node(html).unwrap().children()[1];
    let mut batch = DomBatch::new();
    let dialog = batch.create_element_ns(top.document(), Namespace::html(), "dialog");
    batch.insert_before(&dialog, body, None);
    let subject = session
        .dom()
        .apply(batch, &cancel)
        .unwrap()
        .created(&dialog)
        .unwrap()
        .unwrap();
    let opened = session
        .source_execution()
        .apply(
            &mut session.dom(),
            DomSourceEdit::set_attribute(subject, "open", ""),
            source_limits(10_000_000),
            &mut services,
            &cancel,
        )
        .unwrap();
    assert!(matches!(opened, DomSourceDriveProgress::Complete(_)));
    assert!(
        session
            .snapshot()
            .node(subject)
            .unwrap()
            .as_dialog()
            .unwrap()
            .has_close_watcher()
    );
    let before = match session
        .source_execution()
        .apply(
            &mut session.dom(),
            DomSourceEdit::close_dialog(subject, None),
            source_limits(10_000_000),
            &mut services,
            &cancel,
        )
        .unwrap()
    {
        DomSourceDriveProgress::Pending {
            request: DomSourceRequest::Event(event),
            ..
        } => event,
        other => panic!("actual Close beforetoggle: {other:?}"),
    };
    let mut registry = DomEventDispatcher::new(DomEventRegistryLimits {
        registrations: 1,
        pending_events: 1,
        nested_dispatches: 1,
    })
    .unwrap();
    let result = session
        .source_execution()
        .service_event(
            &mut session.dom(),
            &before,
            &mut registry,
            &mut (),
            DomSourceEventServiceLimits {
                source: source_limits(10_000_000),
                dispatch: dispatch_limits(),
            },
            &cancel,
        )
        .unwrap();
    let DomSourceEventServiceProgress::Serviced { receipt } = result else {
        panic!("real receipt: {result:?}")
    };
    assert_eq!(receipt.callbacks(), 0);
    assert!(receipt.failures().is_empty());
    let result = session
        .source_execution()
        .resume(
            &mut session.dom(),
            before.operation(),
            source_limits(10_000_000),
            &mut services,
            &cancel,
        )
        .unwrap();
    assert!(matches!(result, DomSourceDriveProgress::Complete(_)));
    let tasks = session.element_task_execution();
    let task = tasks
        .pending_task(DomElementTaskSourceKind::DomManipulation, 0)
        .unwrap()
        .unwrap();
    assert!(task.dialog_toggle().is_some());
    let queued = session.snapshot();
    assert_eq!(
        queued
            .node(subject)
            .unwrap()
            .as_dialog()
            .unwrap()
            .toggle_task_tracker()
            .unwrap()
            .task(),
        task.id()
    );
    let mut context = Cleanup {
        tasks: session.element_task_execution(),
        hosted: session.hosted_execution(),
        calls: 0,
        exhausted_at: None,
    };
    let mut registry = DomEventDispatcher::new(DomEventRegistryLimits {
        registrations: 1,
        pending_events: 1,
        nested_dispatches: 1,
    })
    .unwrap();
    let listener = registry
        .add_listener(
            &mut session.dom(),
            subject,
            DomEventType::Toggle,
            DomEventCallback::new(|dom: &mut DomEventDom<'_>, _, context: &mut Cleanup, _| {
                context.calls += 1;
                let owner = context.hosted.state().unwrap();
                let spent = {
                    let state = owner.borrow();
                    assert_eq!(state.hosts.len(), 1);
                    assert_eq!(state.hosts[0].top_levels.len(), 1);
                    let group = state.hosts[0].top_levels[0].group.as_ref().unwrap();
                    assert_eq!(group.clusters.len(), 1);
                    let agent = &group.clusters[0].agent;
                    assert_eq!(agent.windows.rows.len(), 1);
                    assert!(
                        !agent.windows.rows[0].needed(),
                        "real Close released watcher custody"
                    );
                    assert_eq!(agent.globals.len(), 1);
                    state.work
                };
                // Actual set_limits reads one Host, composition, cluster, empty
                // Window and two Settings fields (6); task-count reads Host,
                // composition, group and cluster (4); policy transition costs1.
                // This admits that genuine policy update, leaving no next unit for
                // NativeTaskRunning::checkpoint. No counter is seeded or refunded.
                let cap = spent + (1 + 1 + 1 + 1 + 2) + (1 + 1 + 1 + 1) + 1;
                let mut policy = session_limits().hosted;
                policy.work = cap;
                context
                    .hosted
                    .set_limits(dom, policy, &AtomicBool::new(false))
                    .unwrap();
                assert_eq!(owner.borrow().work, cap);
                context.exhausted_at = Some(cap);
                Ok(())
            }),
            DomEventListenerOptions {
                capture: false,
                once: true,
            },
        )
        .unwrap();
    let policy = DomElementTaskRunLimits {
        dispatch: dispatch_limits(),
        source: source_limits(10_000_000),
    };
    let result = DomElementTaskRuntime::run_next(
        &mut session.dom(),
        DomElementTaskSourceKind::DomManipulation,
        &mut registry,
        &mut context,
        policy,
        &cancel,
    );
    assert!(
        matches!(
            result,
            Err(DomElementTaskRuntimeError::Hosted(
                DomHostedRuntimeError::Admission
            ))
        ),
        "actual checkpoint must refuse after Source completion: {result:?}"
    );
    assert_eq!(context.calls, 1);
    assert!(context.exhausted_at.is_some());
    assert!(!registry.remove_listener(listener));
    assert!(
        session.dom().pending_source_edit().is_none(),
        "real Source cleanup already completed"
    );
    let settled = session.snapshot();
    assert_eq!(settled.incomplete_source_operation(), None);
    assert_eq!(
        settled
            .node(subject)
            .unwrap()
            .as_dialog()
            .unwrap()
            .toggle_task_tracker(),
        None
    );
    assert_eq!(
        tasks
            .pending_task(DomElementTaskSourceKind::DomManipulation, 0)
            .unwrap(),
        Some(task)
    );
    assert_eq!(
        tasks.pending_total_tasks().unwrap(),
        2,
        "original receipt and independent Close remain owned"
    );
    drop(context.tasks);
    context.tasks = session.element_task_execution();
    context
        .hosted
        .set_limits(&mut session.dom(), session_limits().hosted, &cancel)
        .unwrap();
    let result = DomElementTaskRuntime::run_next(
        &mut session.dom(),
        DomElementTaskSourceKind::DomManipulation,
        &mut registry,
        &mut context,
        policy,
        &cancel,
    )
    .unwrap();
    let DomElementTaskProgress::Complete {
        task: actual,
        receipt,
        dialog_toggle_disposition,
        displaced_owner,
    } = result
    else {
        panic!("same task must transfer original outputs: {result:?}")
    };
    assert_eq!(actual, task);
    assert_eq!(
        dialog_toggle_disposition,
        Some(DomDialogToggleTaskDisposition::TrackerCleared)
    );
    assert!(displaced_owner.is_none());
    assert_eq!(receipt.callbacks(), 1);
    assert!(receipt.failures().is_empty(), "{:?}", receipt.failures());
    assert_eq!(
        context.calls, 1,
        "completed callback cannot replay after checkpoint refusal"
    );
    assert_eq!(session.snapshot().stamp(), settled.stamp());
    let result = DomElementTaskRuntime::run_next(
        &mut session.dom(),
        DomElementTaskSourceKind::UserInteraction,
        &mut registry,
        &mut context,
        policy,
        &cancel,
    )
    .unwrap();
    let DomElementTaskProgress::Complete {
        dialog_toggle_disposition,
        receipt,
        ..
    } = result
    else {
        panic!("independent Close must survive: {result:?}")
    };
    assert_eq!(dialog_toggle_disposition, None);
    assert_eq!(receipt.callbacks(), 0);
    assert_eq!(tasks.pending_total_tasks().unwrap(), 0);
    assert!(session.dom().pending_source_edit().is_none());
}
