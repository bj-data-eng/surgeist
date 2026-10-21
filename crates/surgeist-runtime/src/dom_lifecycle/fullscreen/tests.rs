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
                meta: None,
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
                meta: None,
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

fn inline_fixture(
    max_revision: u64,
) -> (
    DomSession,
    HostedRuntime,
    DomHostedTopLevel,
    NodeId,
    NodeId,
    surgeist_cssom::CssStyleProperties,
    surgeist_dom::DomInlineStyleSourceContext,
) {
    use surgeist_css::{CssParserContext, CssParserMode, CssUtf16InputLimits};
    use surgeist_cssom::{
        CssomDeclarationRequestLimits, CssomDeclarationSupport, CssomInputRole, CssomInputVersion,
        CssomLimits,
    };
    use surgeist_dom::{
        DomInlineStyleInputs, DomInlineStyleLimits, DomInlineStyleProfile,
        DomInlineStyleRequestLimits, DomInlineStyleSourceContext,
    };
    let cancel = AtomicBool::new(false);
    let (mut session, hosted, top, root) = fixture();
    let mut batch = DomBatch::new();
    let child = batch.create_element_ns(top.document(), Namespace::html(), "div");
    batch.insert_before(&child, root, None);
    batch.fullscreen_element_model(root);
    let child = session
        .dom()
        .apply(batch, &cancel)
        .unwrap()
        .created(&child)
        .unwrap()
        .unwrap();
    let cssom = CssomLimits {
        max_revision,
        ..CssomLimits::default()
    };
    session
        .dom()
        .raw_mut()
        .configure_inline_styles(
            DomInlineStyleLimits::new(cssom, 8, 64_000_000, 8_000_000, 16_000_000).unwrap(),
            &cancel,
        )
        .unwrap();
    let snapshot = session.snapshot();
    let document = snapshot.document(top.document()).unwrap();
    let mode = if document.metadata().mode() == surgeist_dom::HtmlDocumentMode::Quirks {
        CssParserMode::Quirks
    } else {
        CssParserMode::Standards
    };
    let parser = CssParserContext::new(mode);
    let version = CssomInputVersion {
        role: CssomInputRole::Support,
        identity: "runtime-fullscreen".into(),
        revision: 1,
    };
    let input = DomInlineStyleInputs::new(
        document.stamp(),
        parser,
        version.clone(),
        CssomDeclarationSupport::default(),
    )
    .unwrap();
    let limits = DomInlineStyleRequestLimits {
        input: CssUtf16InputLimits::default(),
        declaration: CssomDeclarationRequestLimits::default(),
    };
    let properties = session
        .dom()
        .raw_mut()
        .inline_style(
            child,
            &input,
            limits,
            |_, _| panic!("empty initial attribute"),
            &cancel,
        )
        .unwrap();
    let context = DomInlineStyleSourceContext::new(
        session.snapshot().stamp(),
        vec![
            DomInlineStyleProfile::new(
                child,
                top.document(),
                parser,
                version,
                CssomDeclarationSupport::default(),
            )
            .unwrap(),
        ],
    )
    .unwrap();
    (session, hosted, top, root, child, properties, context)
}

fn inline_requests() -> surgeist_dom::DomInlineStyleRequestLimits {
    surgeist_dom::DomInlineStyleRequestLimits {
        input: surgeist_css::CssUtf16InputLimits::default(),
        declaration: surgeist_cssom::CssomDeclarationRequestLimits::default(),
    }
}

#[test]
fn bound_detach_joins_inline_context_and_one_runtime_job_before_returning_receipt() {
    let cancel = AtomicBool::new(false);
    let (mut session, mut hosted, top, root, child, properties, context) = inline_fixture(u64::MAX);
    let before = session.snapshot();
    let old = session
        .dom()
        .raw_mut()
        .capture_inline_style(child)
        .unwrap()
        .unwrap();
    let result = session
        .source_execution()
        .apply_with_inline_styles(
            &mut session.dom(),
            crate::DomSourceInlineStyleEdit::new(
                DomSourceEdit::detach(root),
                context,
                inline_requests(),
            ),
            source_limits(),
            &mut DomSourceServices {
                meta: None,
                execution_scope: None,
                style_policy: &mut |_| panic!("no sheet service"),
                url: None,
            },
            |_, _| panic!("context-only prefix never resolves declarations"),
            &cancel,
        )
        .unwrap();
    let DomSourceDriveProgress::InlineFullscreenHandoff {
        operation,
        publication,
    } = result
    else {
        panic!("actual serviced handoff boundary")
    };
    assert_eq!(publication.dom().from(), before.stamp());
    assert_eq!(publication.dom().to(), session.snapshot().stamp());
    assert!(publication.inline().owner_effects().is_empty());
    assert_eq!(session.snapshot().node(root).unwrap().parent(), None);
    assert_eq!(
        session.dom().pending_source_edit().unwrap().operation(),
        operation
    );
    let owner = hosted.state().unwrap();
    let invocation = {
        let state = owner.borrow();
        let rows = &state.hosts[0].top_levels[0]
            .group
            .as_ref()
            .unwrap()
            .fullscreen
            .rows;
        assert_eq!(rows.len(), 1);
        assert!(rows[0].ready);
        assert_eq!(
            rows[0].permission.as_ref().unwrap().id(),
            rows[0].invocation
        );
        rows[0].invocation
    };
    let done = session
        .source_execution()
        .resume_with_inline_styles(
            &mut session.dom(),
            operation,
            source_limits(),
            &mut DomSourceServices {
                meta: None,
                execution_scope: None,
                style_policy: &mut |_| panic!("no sheet service"),
                url: None,
            },
            |_, _| panic!("removal tail never reparses"),
            &cancel,
        )
        .unwrap();
    assert!(matches!(done, DomSourceDriveProgress::Complete(_)));
    assert!(session.dom().pending_source_edit().is_none());
    let current = session
        .dom()
        .raw_mut()
        .capture_inline_style(child)
        .unwrap()
        .unwrap();
    assert_eq!(current.properties().handle(), &properties);
    assert_ne!(
        current.properties().snapshot().revision(),
        old.properties().snapshot().revision()
    );
    assert_eq!(current.tree_scope().root(), root);
    assert_eq!(old.tree_scope().root(), top.document().node());
    assert_eq!(
        hosted
            .run_next(&mut session.dom(), top.navigable(), &cancel)
            .unwrap(),
        DomHostedTaskProgress::FullscreenExitPending {
            invocation,
            step: DomFullscreenExitStep::FullyUnlockScreenOrientation
        }
    );
    assert_eq!(
        hosted
            .run_next(&mut session.dom(), top.navigable(), &cancel)
            .unwrap(),
        DomHostedTaskProgress::Quiescent
    );
}

#[test]
fn inline_commit_refusal_releases_reserved_runtime_row_and_preserves_exposed_source() {
    let cancel = AtomicBool::new(false);
    let (mut session, hosted, _, root, child, _, context) = inline_fixture(1);
    let before = session.snapshot();
    let old = session
        .dom()
        .raw_mut()
        .capture_inline_style(child)
        .unwrap()
        .unwrap();
    let result = session.source_execution().apply_with_inline_styles(
        &mut session.dom(),
        crate::DomSourceInlineStyleEdit::new(
            DomSourceEdit::detach(root),
            context,
            inline_requests(),
        ),
        source_limits(),
        &mut DomSourceServices {
            meta: None,
            execution_scope: None,
            style_policy: &mut |_| None,
            url: None,
        },
        |_, _| panic!("context-only prefix never resolves"),
        &cancel,
    );
    assert!(matches!(
        result,
        Err(crate::DomSourceDriveError::Source(
            crate::DomSourceRuntimeError::Inline(surgeist_dom::DomInlineStyleError::Cssom(
                surgeist_cssom::CssomError::RevisionExhausted
            ))
        ))
    ));
    assert_eq!(session.snapshot().stamp(), before.stamp());
    assert_eq!(
        session.snapshot().node(root).unwrap().parent(),
        before.node(root).unwrap().parent()
    );
    let status = session.dom().pending_source_edit().unwrap();
    assert!(!status.has_published());
    let owner = hosted.state().unwrap();
    assert!(
        owner.borrow().hosts[0].top_levels[0]
            .group
            .as_ref()
            .unwrap()
            .fullscreen
            .rows
            .is_empty()
    );
    assert_eq!(
        old.properties()
            .length(inline_requests().declaration)
            .unwrap(),
        0
    );
    let mut dom = session.dom();
    let mut lease = dom
        .resume_source_edit(status.operation(), source_limits(), &cancel)
        .unwrap();
    let progress = lease
        .advance_with_inline_styles(|_, _| panic!("same retained handoff"), &cancel)
        .unwrap();
    assert!(progress.inline().is_none());
    assert!(matches!(
        progress.into_parts().0,
        DomSourceProgress::Request(DomSourceRequest::FullscreenExitHandoff(_))
    ));
    assert!(lease.status().spent_work() > status.spent_work());
}

#[test]
fn reserved_row_unwinds_without_publishing_or_losing_the_real_request() {
    let cancel = AtomicBool::new(false);
    let (mut session, hosted, top, root) = fixture();
    let mut batch = DomBatch::new();
    batch.fullscreen_element_model(root);
    session.dom().apply(batch, &cancel).unwrap();
    let mut dom = session.dom();
    let mut lease = dom
        .begin_source_edit(
            DomSourceEdit::exit_fullscreen(top.document()),
            source_limits(),
            &cancel,
        )
        .unwrap();
    let DomSourceProgress::Request(DomSourceRequest::FullscreenExitHandoff(request)) =
        lease.advance(&cancel).unwrap()
    else {
        panic!("actual handoff")
    };
    let before = lease.snapshot();
    let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
        hosted.service_fullscreen_handoff::<(), DomHostedRuntimeError>(
            &mut lease,
            &request,
            &cancel,
            |_, _| panic!("reserved-row unwind"),
        )
    }));
    assert!(result.is_err());
    assert_eq!(lease.snapshot().stamp(), before.stamp());
    let owner = hosted.state().unwrap();
    assert!(
        owner.borrow().hosts[0].top_levels[0]
            .group
            .as_ref()
            .unwrap()
            .fullscreen
            .rows
            .is_empty()
    );
    hosted
        .service_fullscreen_exit_handoff(&mut lease, &request, &cancel)
        .unwrap();
    let state = owner.borrow();
    let rows = &state.hosts[0].top_levels[0]
        .group
        .as_ref()
        .unwrap()
        .fullscreen
        .rows;
    assert_eq!(rows.len(), 1);
    assert!(rows[0].permission.is_some());
    assert!(rows[0].ready);
}

#[test]
fn runtime_capacity_refusal_retries_original_inline_handoff_once() {
    let cancel = AtomicBool::new(false);
    let (mut session, mut hosted, _, root, child, _, context) = inline_fixture(u64::MAX);
    let before = session.snapshot();
    let old = session
        .dom()
        .raw_mut()
        .capture_inline_style(child)
        .unwrap()
        .unwrap();
    let mut reduced = limits().hosted;
    reduced.tasks = 0;
    hosted
        .set_limits(&mut session.dom(), reduced, &cancel)
        .unwrap();
    let result = session.source_execution().apply_with_inline_styles(
        &mut session.dom(),
        crate::DomSourceInlineStyleEdit::new(
            DomSourceEdit::detach(root),
            context,
            inline_requests(),
        ),
        source_limits(),
        &mut DomSourceServices {
            meta: None,
            execution_scope: None,
            style_policy: &mut |_| None,
            url: None,
        },
        |_, _| panic!("context-only prefix never resolves"),
        &cancel,
    );
    assert!(matches!(
        result,
        Err(crate::DomSourceDriveError::Source(
            crate::DomSourceRuntimeError::Hosted(DomHostedRuntimeError::Admission)
        ))
    ));
    assert_eq!(session.snapshot().stamp(), before.stamp());
    assert_eq!(
        session.snapshot().node(root).unwrap().parent(),
        before.node(root).unwrap().parent()
    );
    let original = session.dom().pending_source_edit().unwrap();
    assert!(!original.has_published());
    assert_eq!(
        old.properties()
            .length(inline_requests().declaration)
            .unwrap(),
        0
    );
    hosted
        .set_limits(&mut session.dom(), limits().hosted, &cancel)
        .unwrap();
    let result = session
        .source_execution()
        .resume_with_inline_styles(
            &mut session.dom(),
            original.operation(),
            source_limits(),
            &mut DomSourceServices {
                meta: None,
                execution_scope: None,
                style_policy: &mut |_| None,
                url: None,
            },
            |_, _| panic!("same context-only operation"),
            &cancel,
        )
        .unwrap();
    let DomSourceDriveProgress::InlineFullscreenHandoff {
        operation,
        publication,
    } = result
    else {
        panic!("same real handoff commits")
    };
    assert_eq!(operation, original.operation());
    assert_eq!(publication.dom().from(), before.stamp());
    assert!(session.dom().pending_source_edit().unwrap().spent_work() > original.spent_work());
    let owner = hosted.state().unwrap();
    assert_eq!(
        owner.borrow().hosts[0].top_levels[0]
            .group
            .as_ref()
            .unwrap()
            .fullscreen
            .rows
            .len(),
        1
    );
}
