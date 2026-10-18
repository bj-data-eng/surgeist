//! Fullscreen7c38 removing steps call ExitFullscreen for the current model
//! element. A real fully active Document reaches the unavailable active-exit
//! work; it cannot take the inactive rejection branch or fabricate completion.
//! HTML877d destroy-Document raw75874–82 runs unloading cleanup before clearing
//! the browsing context and active history Document. Iframe raw23908–14 excludes
//! unloading events, without waiving that cleanup.
#![cfg(any(
    target_os = "macos",
    target_os = "linux",
    target_os = "windows",
    target_os = "android",
    target_os = "ios"
))]

use std::sync::atomic::AtomicBool;

use surgeist_dom::{
    DocumentId, Dom, DomBatch, DomDispatchCustodyLimits, DomError, DomFullscreenModelStep,
    DomHostedError, DomHostedGlobalId, DomHostedLimits, DomHostedNavigableId,
    DomHostedRecordLimits, DomLimits, DomSourceCompletion, DomSourceEdit, DomSourceLimits,
    DomTopLayerUaRule, Namespace, NodeId,
};
use surgeist_runtime::{
    DomElementTaskRuntimeLimits, DomHostedRuntimeError, DomHostedRuntimeLimits, DomHostedTaskKind,
    DomHostedTaskProgress, DomHostedTopLevel, DomSession, DomSessionLimits, DomSourceDriveProgress,
    DomSourceEventRuntimeLimits, DomSourceServices, DomStyleRuntimeLimits, HostedRuntime,
};

fn session() -> DomSession {
    DomSession::new(
        Dom::new(
            DomLimits::new(128, 100_000, 256, 128, 128, 32, 4_000_000)
                .unwrap()
                .with_dispatch_custody(
                    DomDispatchCustodyLimits::new(16, 1024, 16_000_000, 16_000_000, 4_000_000)
                        .unwrap(),
                ),
        )
        .unwrap(),
        DomSessionLimits {
            source_events: DomSourceEventRuntimeLimits {
                jobs: 4,
                storage_bytes: 16_000_000,
                work: 1_000_000,
            },
            style: DomStyleRuntimeLimits {
                roots: 4,
                completions: 8,
                reports: 8,
                storage_bytes: 16_000_000,
                work: 1_000_000,
                css_input: surgeist_css::CssUtf16InputLimits::default(),
            },
            element_tasks: DomElementTaskRuntimeLimits {
                tasks: 8,
                storage_bytes: 16_000_000,
                work: 1_000_000,
            },
            hosted: DomHostedRuntimeLimits {
                canonical: DomHostedLimits::new(
                    DomHostedRecordLimits::new(8, 8, 8, 8, 8, 8, 8),
                    16_000_000,
                    16_000_000,
                    1_000_000,
                ),
                tasks: 8,
                storage_bytes: 16_000_000,
                scratch_bytes: 16_000_000,
                work: 1_000_000,
            },
        },
    )
    .unwrap()
}

#[test]
fn removing_real_active_fullscreen_document_element_refuses_without_publishing_a_prefix() {
    let cancel = AtomicBool::new(false);
    let mut session = session();
    let mut hosted = session.hosted_execution();
    let top = {
        let mut dom = session.dom();
        let host = hosted.create_host(&mut dom, &cancel).unwrap();
        hosted.create_top_level(&mut dom, host, &cancel).unwrap()
    };
    let initial = session.snapshot();
    let document = initial.document(top.document()).unwrap();
    let root = document.document_element().unwrap();
    assert!(document.is_fully_active().unwrap());
    assert_eq!(document.navigable(), Some(top.navigable()));
    assert_eq!(
        initial.global_document(top.global()).unwrap(),
        top.document()
    );
    let navigable = initial.hosted_navigable(top.navigable()).unwrap();
    assert_eq!(navigable.active_document(), Some(top.document()));
    assert_eq!(navigable.global(), top.global());

    let mut batch = DomBatch::new();
    batch.fullscreen_element_model(root);
    session.dom().apply(batch, &cancel).unwrap();
    let before = session.snapshot();
    let usage = session.dom().usage();
    assert!(
        before
            .document(top.document())
            .unwrap()
            .is_fully_active()
            .unwrap()
    );
    assert!(
        before
            .node(root)
            .unwrap()
            .as_element()
            .unwrap()
            .fullscreen_flag()
    );
    assert_eq!(
        before
            .document(top.document())
            .unwrap()
            .fullscreen_model_element()
            .unwrap(),
        Some(root)
    );
    assert_eq!(
        before
            .document(top.document())
            .unwrap()
            .fullscreen_element()
            .unwrap(),
        Some(root)
    );

    let mut batch = DomBatch::new();
    batch.detach(root);
    assert_eq!(
        session.dom().apply(batch, &cancel).unwrap_err(),
        DomError::FullscreenModelUnavailable {
            step: DomFullscreenModelStep::ExitFullscreen,
        }
    );

    let after = session.snapshot();
    assert_eq!(after.stamp(), before.stamp());
    assert_eq!(session.dom().usage(), usage);
    assert_eq!(
        after.nodes().collect::<Vec<_>>(),
        before.nodes().collect::<Vec<_>>()
    );
    for node in before
        .node(top.document().node())
        .unwrap()
        .shadow_including_tree_order()
    {
        assert_eq!(
            after.node(node).unwrap().parent(),
            before.node(node).unwrap().parent()
        );
        assert_eq!(
            after.node(node).unwrap().children(),
            before.node(node).unwrap().children()
        );
    }
    assert_eq!(
        after.node(root).unwrap().parent(),
        Some(top.document().node())
    );
    assert!(after.node(root).unwrap().is_connected());
    assert!(
        after
            .node(root)
            .unwrap()
            .as_element()
            .unwrap()
            .fullscreen_flag()
    );
    let document = after.document(top.document()).unwrap();
    assert!(document.is_fully_active().unwrap());
    assert_eq!(document.document_element(), Some(root));
    assert_eq!(document.fullscreen_model_element().unwrap(), Some(root));
    let layer = document.top_layer().unwrap();
    assert_eq!(layer.raw_members(), &[root]);
    assert_eq!(layer.pending_removals(), &[]);
    assert_eq!(
        layer.ua_rules().collect::<Vec<_>>(),
        [
            DomTopLayerUaRule::UniversalOverlayNone,
            DomTopLayerUaRule::ElementOverlayAuto(root),
        ]
    );
    assert!(after.incomplete_source_operation().is_none());
    assert!(after.incomplete_hosted_operation().is_none());
    assert!(session.dom().pending_source_edit().is_none());

    // Explicit model unfullscreen supplies the actual local cleanup before
    // the independent ordinary hosted teardown. No platform exit is asserted.
    let mut batch = DomBatch::new();
    batch.unfullscreen_element_model(root);
    session.dom().apply(batch, &cancel).unwrap();
    let cleaned = session.snapshot();
    assert!(
        !cleaned
            .node(root)
            .unwrap()
            .as_element()
            .unwrap()
            .fullscreen_flag()
    );
    assert!(
        cleaned
            .document(top.document())
            .unwrap()
            .top_layer()
            .unwrap()
            .raw_members()
            .is_empty()
    );
    assert!(
        before
            .node(root)
            .unwrap()
            .as_element()
            .unwrap()
            .fullscreen_flag()
    );
    assert_eq!(
        before
            .document(top.document())
            .unwrap()
            .top_layer()
            .unwrap()
            .raw_members(),
        &[root]
    );

    let queued = hosted
        .destroy_top_level(&mut session.dom(), top.navigable(), &cancel)
        .unwrap();
    let mut last = DomHostedTaskProgress::Idle;
    for _ in 0..32 {
        last = hosted
            .run_next(&mut session.dom(), queued.traversable(), &cancel)
            .unwrap();
        if matches!(last, DomHostedTaskProgress::Complete { operation } if operation == queued.operation())
        {
            break;
        }
    }
    assert!(
        matches!(last, DomHostedTaskProgress::Complete { operation } if operation == queued.operation()),
        "ordinary hosted teardown did not complete within 32 queue advances: {last:?}"
    );
}

fn top_level(session: &mut DomSession) -> DomHostedTopLevel {
    let cancel = AtomicBool::new(false);
    let mut hosted = session.hosted_execution();
    let mut dom = session.dom();
    let host = hosted.create_host(&mut dom, &cancel).unwrap();
    hosted.create_top_level(&mut dom, host, &cancel).unwrap()
}

fn complete_source(session: &mut DomSession, edit: DomSourceEdit) -> DomSourceCompletion {
    let cancel = AtomicBool::new(false);
    let progress = session
        .source_execution()
        .apply(
            &mut session.dom(),
            edit,
            DomSourceLimits::new(16_000_000, 1_000_000, 10_000_000, 128, 4096, 4096, 1),
            &mut DomSourceServices {
                execution_scope: None,
                style_policy: &mut |_| None,
                url: None,
            },
            &cancel,
        )
        .unwrap();
    let DomSourceDriveProgress::Complete(done) = progress else {
        panic!("native initial Iframe source hooks must complete locally");
    };
    done
}

fn fully_exit_unavailable() -> DomHostedRuntimeError {
    DomHostedRuntimeError::Dom(DomHostedError::Dom(DomError::FullscreenModelUnavailable {
        step: DomFullscreenModelStep::FullyExitFullscreen,
    }))
}

fn pump_until_full_exit_refusal(
    session: &mut DomSession,
    hosted: &mut HostedRuntime,
    traversable: DomHostedNavigableId,
) -> Vec<DomHostedTaskProgress> {
    let cancel = AtomicBool::new(false);
    let mut prior = Vec::new();
    for _ in 0..64 {
        let before = session.snapshot();
        match hosted.run_next(&mut session.dom(), traversable, &cancel) {
            Err(error) => {
                assert_eq!(error, fully_exit_unavailable());
                assert_eq!(session.snapshot().stamp(), before.stamp());
                // Repeating the exact queue advance must reach the same owned
                // cleanup, rather than consume it or acknowledge destruction.
                assert_eq!(
                    hosted
                        .run_next(&mut session.dom(), traversable, &cancel)
                        .unwrap_err(),
                    fully_exit_unavailable()
                );
                assert_eq!(session.snapshot().stamp(), before.stamp());
                return prior;
            }
            Ok(progress) => {
                assert!(
                    !matches!(progress, DomHostedTaskProgress::Idle),
                    "queued cleanup disappeared before its full-exit requirement"
                );
                assert!(
                    !matches!(
                        progress,
                        DomHostedTaskProgress::Executed {
                            kind: DomHostedTaskKind::Destruction,
                            ..
                        }
                    ),
                    "destruction executed before required unloading cleanup"
                );
                prior.push(progress);
            }
        }
    }
    panic!("queued cleanup did not reach full-exit refusal within 64 advances")
}

fn assert_retained_fullscreen(
    session: &DomSession,
    document: DocumentId,
    navigable: DomHostedNavigableId,
    root: NodeId,
) {
    let snapshot = session.snapshot();
    let nav = snapshot.hosted_navigable(navigable).unwrap();
    assert!(nav.is_retiring());
    assert!(!nav.is_retired());
    assert_eq!(nav.active_document(), Some(document));
    assert_eq!(
        snapshot
            .hosted_entry(nav.active_entry())
            .unwrap()
            .document(),
        Some(document)
    );
    assert_eq!(snapshot.global_document(nav.global()).unwrap(), document);
    assert_eq!(snapshot.node(root).unwrap().parent(), Some(document.node()));
    assert!(
        snapshot
            .node(root)
            .unwrap()
            .as_element()
            .unwrap()
            .fullscreen_flag()
    );
    let document = snapshot.document(document).unwrap();
    assert_eq!(document.fullscreen_model_element().unwrap(), Some(root));
    let layer = document.top_layer().unwrap();
    assert_eq!(layer.raw_members(), &[root]);
    assert!(layer.pending_removals().is_empty());
    assert_eq!(
        layer.ua_rules().collect::<Vec<_>>(),
        [
            DomTopLayerUaRule::UniversalOverlayNone,
            DomTopLayerUaRule::ElementOverlayAuto(root),
        ]
    );
}

fn unfullscreen(session: &mut DomSession, node: NodeId) {
    let mut batch = DomBatch::new();
    batch.unfullscreen_element_model(node);
    session.dom().apply(batch, &AtomicBool::new(false)).unwrap();
}

fn assert_one_destruction(
    observations: &[DomHostedTaskProgress],
    document: DocumentId,
    global: DomHostedGlobalId,
) {
    assert_eq!(
        observations
            .iter()
            .filter(|progress| matches!(progress,
                DomHostedTaskProgress::Executed {
                    document: actual_document,
                    global: actual_global,
                    kind: DomHostedTaskKind::Destruction,
                    ..
                } if *actual_document == document && *actual_global == global
            ))
            .count(),
        1
    );
}

#[test]
fn top_level_cleanup_retains_its_original_queue_until_fullscreen_model_is_actually_cleared() {
    let cancel = AtomicBool::new(false);
    let mut session = session();
    let top = top_level(&mut session);
    let initial = session.snapshot();
    assert!(
        initial
            .document(top.document())
            .unwrap()
            .is_fully_active()
            .unwrap()
    );
    let root = initial
        .document(top.document())
        .unwrap()
        .document_element()
        .unwrap();
    let mut batch = DomBatch::new();
    batch.fullscreen_element_model(root);
    session.dom().apply(batch, &cancel).unwrap();
    let fullscreen = session.snapshot();

    let mut hosted = session.hosted_execution();
    let queued = hosted
        .destroy_top_level(&mut session.dom(), top.navigable(), &cancel)
        .unwrap();
    let prior = pump_until_full_exit_refusal(&mut session, &mut hosted, queued.traversable());
    assert!(prior.iter().any(|progress| matches!(progress,
        DomHostedTaskProgress::Pending { operation, .. } if *operation == queued.operation()
    )));
    assert!(!prior.iter().any(|progress| matches!(progress,
        DomHostedTaskProgress::Complete { operation } if *operation == queued.operation()
    )));
    assert_retained_fullscreen(&session, top.document(), top.navigable(), root);
    assert_eq!(
        session
            .snapshot()
            .hosted_entry(top.entry())
            .unwrap()
            .document(),
        Some(top.document())
    );

    unfullscreen(&mut session, root);
    let mut observations = Vec::new();
    let mut completed = false;
    for _ in 0..64 {
        let progress = hosted
            .run_next(&mut session.dom(), queued.traversable(), &cancel)
            .unwrap();
        observations.push(progress);
        if matches!(progress, DomHostedTaskProgress::Complete { operation } if operation == queued.operation())
        {
            completed = true;
            break;
        }
    }
    assert!(
        completed,
        "the original destruction invocation did not resume after actual cleanup"
    );
    assert_one_destruction(&observations, top.document(), top.global());
    let retired = session.snapshot();
    assert!(
        retired
            .hosted_navigable(top.navigable())
            .unwrap()
            .is_retired()
    );
    assert_eq!(retired.hosted_entry(top.entry()).unwrap().document(), None);
    assert!(
        !retired
            .document(top.document())
            .unwrap()
            .is_fully_active()
            .unwrap()
    );
    assert!(
        fullscreen
            .document(top.document())
            .unwrap()
            .is_fully_active()
            .unwrap()
    );
    assert_eq!(
        fullscreen
            .document(top.document())
            .unwrap()
            .top_layer()
            .unwrap()
            .raw_members(),
        &[root]
    );
}

#[test]
fn iframe_destroyed_without_unload_events_still_retains_full_exit_cleanup_and_retries_same_queue() {
    let cancel = AtomicBool::new(false);
    let mut session = session();
    let top = top_level(&mut session);
    let initial = session.snapshot();
    let top_root = initial
        .document(top.document())
        .unwrap()
        .document_element()
        .unwrap();
    let body = initial.node(top_root).unwrap().children()[1];
    let mut batch = DomBatch::new();
    let closed = batch.create_html_document(top.document(), None);
    let container = batch.create_element_ns(&closed, Namespace::html(), "iframe");
    let container = session
        .dom()
        .apply(batch, &cancel)
        .unwrap()
        .created(&container)
        .unwrap()
        .unwrap();
    complete_source(
        &mut session,
        DomSourceEdit::insert_before(container, body, None),
    );
    let inserted = session.snapshot();
    let child = inserted
        .node(container)
        .unwrap()
        .as_element()
        .unwrap()
        .child_navigable()
        .unwrap()
        .unwrap();
    let child_view = inserted.hosted_navigable(child).unwrap();
    let document = child_view.active_document().unwrap();
    let global = child_view.global();
    let entry = child_view.active_entry();
    let root = inserted
        .document(document)
        .unwrap()
        .document_element()
        .unwrap();
    assert!(
        inserted
            .document(document)
            .unwrap()
            .is_fully_active()
            .unwrap()
    );

    // Settle the actual initial history task before selecting the child's
    // fullscreen model state and triggering the independent removal invocation.
    let mut hosted = session.hosted_execution();
    let mut initial_idle = false;
    for _ in 0..64 {
        if matches!(
            hosted
                .run_next(&mut session.dom(), top.navigable(), &cancel)
                .unwrap(),
            DomHostedTaskProgress::Idle
        ) {
            initial_idle = true;
            break;
        }
    }
    assert!(
        initial_idle,
        "initial child history did not reach its real idle state"
    );
    let mut batch = DomBatch::new();
    batch.fullscreen_element_model(root);
    session.dom().apply(batch, &cancel).unwrap();
    let fullscreen = session.snapshot();
    complete_source(&mut session, DomSourceEdit::detach(container));
    let removed = session.snapshot();
    assert_eq!(removed.node(container).unwrap().parent(), None);
    assert_eq!(
        removed
            .node(container)
            .unwrap()
            .as_element()
            .unwrap()
            .child_navigable(),
        Some(None)
    );
    assert_eq!(
        removed.hosted_entry(entry).unwrap().document(),
        Some(document)
    );
    let prior = pump_until_full_exit_refusal(&mut session, &mut hosted, top.navigable());
    assert_retained_fullscreen(&session, document, child, root);
    let pending = prior
        .iter()
        .filter_map(|progress| {
            match progress {
        DomHostedTaskProgress::Pending { operation, .. }
            if !prior.iter().any(|finished| matches!(finished,
                DomHostedTaskProgress::Complete { operation: completed } if completed == operation
            )) => Some(*operation),
        _ => None,
    }
        })
        .collect::<Vec<_>>();
    assert!(
        !pending.is_empty(),
        "the actual child destruction queue was not observed before refusal"
    );

    unfullscreen(&mut session, root);
    let mut observations = Vec::new();
    let mut idle = false;
    for _ in 0..128 {
        let progress = hosted
            .run_next(&mut session.dom(), top.navigable(), &cancel)
            .unwrap();
        observations.push(progress);
        if matches!(progress, DomHostedTaskProgress::Idle) {
            idle = true;
            break;
        }
    }
    assert!(
        idle,
        "the original child removal and destruction queues did not settle after cleanup"
    );
    for operation in pending {
        assert!(
            observations.iter().any(|progress| matches!(progress,
                DomHostedTaskProgress::Complete { operation: actual } if *actual == operation
            )),
            "the original queued operation was not completed after retry"
        );
    }
    assert_one_destruction(&observations, document, global);
    let retired = session.snapshot();
    assert!(retired.hosted_navigable(child).unwrap().is_retired());
    assert_eq!(retired.hosted_entry(entry).unwrap().document(), None);
    assert!(
        !retired
            .document(document)
            .unwrap()
            .is_fully_active()
            .unwrap()
    );
    assert!(
        retired
            .document(top.document())
            .unwrap()
            .is_fully_active()
            .unwrap()
    );
    assert!(
        fullscreen
            .document(document)
            .unwrap()
            .is_fully_active()
            .unwrap()
    );
    assert_eq!(
        fullscreen
            .document(document)
            .unwrap()
            .top_layer()
            .unwrap()
            .raw_members(),
        &[root]
    );

    let queued = hosted
        .destroy_top_level(&mut session.dom(), top.navigable(), &cancel)
        .unwrap();
    let mut complete = false;
    for _ in 0..64 {
        if matches!(hosted.run_next(&mut session.dom(), queued.traversable(), &cancel).unwrap(),
            DomHostedTaskProgress::Complete { operation } if operation == queued.operation())
        {
            complete = true;
            break;
        }
    }
    assert!(complete, "surviving top-level teardown did not complete");
}

#[test]
fn adopted_residual_flag_without_raw_membership_does_not_block_real_hosted_cleanup() {
    let cancel = AtomicBool::new(false);
    let mut session = session();
    let top = top_level(&mut session);
    let mut batch = DomBatch::new();
    let closed = batch.create_document();
    let node = batch.create_element(&closed, "ordinary");
    batch.insert_before(&node, &closed, None);
    let commit = session.dom().apply(batch, &cancel).unwrap();
    let closed = commit.document(&closed).unwrap();
    let node = commit.created(&node).unwrap().unwrap();
    assert!(
        !session
            .snapshot()
            .document(closed)
            .unwrap()
            .is_fully_active()
            .unwrap()
    );
    let mut batch = DomBatch::new();
    batch.fullscreen_element_model(node);
    session.dom().apply(batch, &cancel).unwrap();
    let mut batch = DomBatch::new();
    batch.detach(node);
    session.dom().apply(batch, &cancel).unwrap();
    let detached = session.snapshot();
    assert!(
        detached
            .node(node)
            .unwrap()
            .as_element()
            .unwrap()
            .fullscreen_flag()
    );
    assert!(
        detached
            .document(closed)
            .unwrap()
            .top_layer()
            .unwrap()
            .raw_members()
            .is_empty()
    );

    // Real inactive removing steps retained the flag and removed the member.
    // Actual cross-Document adoption preserves that flag without transferring
    // any membership. No setter installs this state or bypasses the owner.
    let mut batch = DomBatch::new();
    batch.adopt(node, top.document());
    session.dom().apply(batch, &cancel).unwrap();
    let hosted_flag = session.snapshot();
    assert_eq!(
        hosted_flag.node(node).unwrap().owner_document(),
        Some(top.document())
    );
    assert!(
        hosted_flag
            .node(node)
            .unwrap()
            .as_element()
            .unwrap()
            .fullscreen_flag()
    );
    let document = hosted_flag.document(top.document()).unwrap();
    assert!(document.is_fully_active().unwrap());
    assert_eq!(document.fullscreen_model_element().unwrap(), None);
    assert!(document.top_layer().unwrap().raw_members().is_empty());

    let mut hosted = session.hosted_execution();
    let queued = hosted
        .destroy_top_level(&mut session.dom(), top.navigable(), &cancel)
        .unwrap();
    let mut observations = Vec::new();
    let mut completed = false;
    for _ in 0..64 {
        let progress = hosted
            .run_next(&mut session.dom(), queued.traversable(), &cancel)
            .unwrap();
        observations.push(progress);
        if matches!(progress, DomHostedTaskProgress::Complete { operation } if operation == queued.operation())
        {
            completed = true;
            break;
        }
    }
    assert!(
        completed,
        "raw-empty cleanup must not substitute an all-Element flag scan"
    );
    assert_one_destruction(&observations, top.document(), top.global());
    let retired = session.snapshot();
    assert!(
        retired
            .hosted_navigable(top.navigable())
            .unwrap()
            .is_retired()
    );
    assert_eq!(retired.hosted_entry(top.entry()).unwrap().document(), None);
    assert!(
        retired
            .node(node)
            .unwrap()
            .as_element()
            .unwrap()
            .fullscreen_flag()
    );
    assert_eq!(
        retired.node(node).unwrap().owner_document(),
        Some(top.document())
    );
    assert!(
        hosted_flag
            .document(top.document())
            .unwrap()
            .is_fully_active()
            .unwrap()
    );
}
