//! DOM issuer/constructor boundary only. This fixture models trusted native
//! publication inputs; it does not certify Runtime Global construction or any
//! browser orientation/viewport service. Runtime separately tests composition.
use super::*;
use crate::{
    DomFullscreenExitReturn, DomHostedCreationStep, DomHostedEntryInitialization, DomHostedLimits,
    DomHostedRecordLimits, DomInitialPermissionsPolicy, DomPermissionsPolicyValue, DomSourceEdit,
};

fn policy(outputs: usize) -> DomSourceLimits {
    DomSourceLimits::new(32_000_000, 1_000_000, 100_000, 64, outputs, 64, 4)
}
fn fixture() -> (Dom, crate::DomHostedSource, DocumentId, NodeId) {
    fixture_with_limits(
        DomLimits::new(
            64,
            100_000,
            256,
            128,
            128,
            16,
            4_000_000,
            crate::DomNameLimits::new(16_777_216, 16_384, 4_000_000, 1_048_576).unwrap(),
        )
        .unwrap()
        .with_fullscreen_exits(4)
        .unwrap()
        .with_fullscreen_events(4)
        .unwrap(),
    )
}

pub(super) fn fixture_with_limits(
    limits: DomLimits,
) -> (Dom, crate::DomHostedSource, DocumentId, NodeId) {
    let cancel = AtomicBool::new(false);
    let mut dom = Dom::new(limits).unwrap();
    let mut source = dom
        .register_hosted_source(
            DomHostedLimits::new(
                DomHostedRecordLimits::new(32, 32, 32, 32, 32, 32, 32),
                16_000_000,
                16_000_000,
                10_000_000,
            ),
            &cancel,
        )
        .unwrap();
    let host = source.issue_host(&cancel).unwrap();
    let mut preparation = Some(source.prepare_top_level(host, &cancel).unwrap());
    let mut prepared = Some(source.seal_top_level(&mut preparation, &cancel).unwrap());
    let mut lease = dom
        .begin_hosted_top_level(&mut source, &mut prepared, &cancel)
        .unwrap();
    loop {
        let publication = match lease.step(&dom).unwrap() {
            DomHostedCreationStep::PermissionsPolicy(request) => {
                let mut publication = Some(
                    source
                        .seal_permissions_policy(
                            request,
                            DomInitialPermissionsPolicy::new(
                                DomPermissionsPolicyValue::Enabled,
                                DomPermissionsPolicyValue::Enabled,
                            ),
                            &cancel,
                        )
                        .unwrap(),
                );
                lease
                    .accept_permissions_policy(&mut dom, &mut source, &mut publication, &cancel)
                    .unwrap();
                continue;
            }
            DomHostedCreationStep::NativeExecution(request) => {
                let global = source.issue_global(request, &cancel).unwrap();
                source
                    .seal_native_execution(request, global, 0, &cancel)
                    .unwrap()
            }
            DomHostedCreationStep::MakeActive(request) => {
                source.seal_make_active(request, &cancel).unwrap()
            }
            DomHostedCreationStep::FinishLoading(request) => {
                source.seal_finish_loading(request, 100, &cancel).unwrap()
            }
            DomHostedCreationStep::InitialEntry(request) => {
                let entry = source.issue_entry(request, &cancel).unwrap();
                let navigable = source.issue_navigable(request, &cancel).unwrap();
                let uuid = |last| {
                    let mut out = [0u16; 36];
                    for (target, unit) in out
                        .iter_mut()
                        .zip("00000000-0000-4000-8000-000000000000".encode_utf16())
                    {
                        *target = unit;
                    }
                    out[35] = last;
                    out
                };
                let mut initialization = Some(DomHostedEntryInitialization::new(
                    uuid(u16::from(b'1')),
                    uuid(u16::from(b'2')),
                ));
                source
                    .seal_initial_entry(request, entry, navigable, &mut initialization, &cancel)
                    .unwrap()
            }
            DomHostedCreationStep::Complete(_) => break,
        };
        lease
            .accept_publication(&mut dom, &mut source, &mut Some(publication), &cancel)
            .unwrap();
    }
    let document = lease
        .finish(&mut dom, &mut source, &cancel)
        .unwrap()
        .document();
    let parent = dom
        .snapshot()
        .document(document)
        .unwrap()
        .document_element()
        .unwrap();
    let mut batch = DomBatch::new();
    let subject = batch.create_element_ns(document, crate::Namespace::none(), "fullscreen-subject");
    batch.insert_before(&subject, parent, None);
    batch.fullscreen_element_model(&subject);
    let subject = dom
        .apply(batch, &cancel)
        .unwrap()
        .created(&subject)
        .unwrap()
        .unwrap();
    (dom, source, document, subject)
}

#[test]
fn actual_handoff_and_outer_tail_rollback_share_the_original_cumulative_work_cap() {
    let cancel = AtomicBool::new(false);
    let (mut dom, source, document, subject) = fixture();
    let mut lease = dom
        .begin_source_edit(DomSourceEdit::detach(subject), policy(1024), &cancel)
        .unwrap();
    let DomSourceProgress::Request(DomSourceRequest::FullscreenExitHandoff(request)) =
        lease.advance(&cancel).unwrap()
    else {
        panic!("real reached exit handoff");
    };
    let operation = request.operation();
    let invocation = request.invocation();
    let _job = lease
        .prepare_fullscreen_exit_handoff(&request, &source, &cancel)
        .unwrap()
        .publish();
    let prefix = lease.snapshot();
    assert_eq!(prefix.node(subject).unwrap().parent(), None);
    assert!(
        !prefix
            .node(subject)
            .unwrap()
            .as_element()
            .unwrap()
            .fullscreen_flag()
    );
    assert_eq!(lease.dom.fullscreen_exits.len(), 1);
    let outputs = lease.dom.source_operation.as_ref().unwrap().outputs;
    let mut retry = lease
        .dom
        .resume_source_edit(operation, policy(outputs), &cancel)
        .unwrap();
    let before = retry.status().spent_work();
    assert_eq!(
        retry.advance(&cancel).unwrap_err(),
        DomError::SourceOutputLimit
    );
    let after = retry.status().spent_work();
    assert!(after > before);
    assert_eq!(
        retry.dom.fullscreen_exits[invocation.slot]
            .work
            .load(Ordering::Relaxed),
        after
    );
    assert_eq!(retry.snapshot().stamp(), prefix.stamp());
    assert_eq!(
        retry
            .dom
            .state
            .node(retry.dom.owner, document.node())
            .unwrap()
            .fullscreen_events()
            .len(),
        1
    );
    // Repeated genuine final-output refusal spends this one original budget.
    // It never queues another event or invocation and cannot renew either meter.
    let mut exhausted = false;
    for _ in 0..policy(outputs).work() {
        match retry.advance(&cancel) {
            Err(DomError::SourceOutputLimit) => {}
            Err(DomError::SourceWorkLimit { limit }) => {
                assert_eq!(limit, policy(outputs).work());
                exhausted = true;
                break;
            }
            other => panic!("original unfinished tail: {other:?}"),
        }
    }
    assert!(exhausted);
    assert_eq!(retry.status().spent_work(), policy(outputs).work());
    let retained = &retry.dom.fullscreen_exits[invocation.slot];
    assert_eq!(
        retained.step(&cancel),
        Err(DomError::SourceWorkLimit {
            limit: policy(outputs).work()
        })
    );
    assert_eq!(
        retained.work.load(Ordering::Relaxed),
        retry.status().spent_work()
    );
    assert_eq!(retry.dom.fullscreen_exits.len(), 1);
    assert_eq!(retry.snapshot().stamp(), prefix.stamp());
}

#[test]
fn dropped_prepared_handoff_restores_its_prefix_and_reuses_only_the_same_exposed_invocation() {
    let cancel = AtomicBool::new(false);
    let (mut dom, source, document, subject) = fixture();
    let before = dom.snapshot();
    let mut lease = dom
        .begin_source_edit(
            DomSourceEdit::exit_fullscreen(document),
            policy(1024),
            &cancel,
        )
        .unwrap();
    let DomSourceProgress::Request(DomSourceRequest::FullscreenExitHandoff(request)) =
        lease.advance(&cancel).unwrap()
    else {
        panic!("actual request");
    };
    let operation = request.operation();
    let generation = lease.dom.next_fullscreen_exit;
    drop(
        lease
            .prepare_fullscreen_exit_handoff(&request, &source, &cancel)
            .unwrap(),
    );
    assert_eq!(lease.snapshot().stamp(), before.stamp());
    assert!(lease.dom.fullscreen_exits.is_empty());
    assert!(
        lease
            .dom
            .state
            .node(lease.dom.owner, document.node())
            .unwrap()
            .fullscreen_events()
            .is_empty()
    );
    drop(lease);
    let mut retry = dom
        .resume_source_edit(operation, policy(1024), &cancel)
        .unwrap();
    let DomSourceProgress::Request(DomSourceRequest::FullscreenExitHandoff(again)) =
        retry.advance(&cancel).unwrap()
    else {
        panic!("same retained request");
    };
    assert_eq!(again, request);
    assert_eq!(retry.dom.next_fullscreen_exit, generation);
    let mut job = retry
        .prepare_fullscreen_exit_handoff(&again, &source, &cancel)
        .unwrap()
        .publish();
    let DomSourceProgress::Complete(done) = retry.advance(&cancel).unwrap() else {
        panic!("synchronous return");
    };
    assert_eq!(
        done.fullscreen_exit_return(),
        Some(DomFullscreenExitReturn::Pending(request.invocation()))
    );
    drop(retry);
    assert_eq!(
        dom.resume_fullscreen_exit(&source, &mut job, &cancel)
            .unwrap()
            .id(),
        request.invocation()
    );
    assert!(
        dom.snapshot()
            .node(subject)
            .unwrap()
            .as_element()
            .unwrap()
            .fullscreen_flag(),
        "connected subject has no step7 unfullscreen"
    );
    let mut stale = request.invocation();
    stale.generation -= 1;
    assert_eq!(
        dom.fullscreen_exit_status(stale),
        Err(DomError::StaleFullscreenExit)
    );
    assert_eq!(dom.next_fullscreen_exit, generation);
}

#[test]
fn identity_admission_and_generation_exhaustion_refuse_before_handoff_or_prefix() {
    let cancel = AtomicBool::new(false);
    for exhausted in [false, true] {
        let (mut dom, _source, document, _) = fixture();
        let before = dom.snapshot();
        let mut limits = policy(1024);
        if exhausted {
            dom.next_fullscreen_exit = u64::MAX;
        } else {
            limits.identities = 0;
        }
        let next = dom.next_fullscreen_exit;
        let mut lease = dom
            .begin_source_edit(DomSourceEdit::exit_fullscreen(document), limits, &cancel)
            .unwrap();
        assert_eq!(
            lease.advance(&cancel).unwrap_err(),
            if exhausted {
                DomError::RepresentationExhausted
            } else {
                DomError::SourceIdentityLimit
            }
        );
        assert_eq!(lease.snapshot().stamp(), before.stamp());
        assert!(lease.dom.fullscreen_exits.is_empty());
        assert_eq!(lease.dom.next_fullscreen_exit, next);
        assert!(!lease.status().has_published());
    }
}

#[test]
fn work_custody_lookup_does_not_visit_unrelated_retained_invocations() {
    let cancel = AtomicBool::new(false);
    let (mut dom, source, document, subject) = fixture();
    dom.set_limits(dom.limits.with_fullscreen_exits(64).unwrap())
        .unwrap();
    let mut ids = Vec::new();
    for _ in 0..32 {
        let mut lease = dom
            .begin_source_edit(
                DomSourceEdit::exit_fullscreen(document),
                policy(1024),
                &cancel,
            )
            .unwrap();
        let DomSourceProgress::Request(DomSourceRequest::FullscreenExitHandoff(request)) =
            lease.advance(&cancel).unwrap()
        else {
            panic!("actual native handoff");
        };
        let job = lease
            .prepare_fullscreen_exit_handoff(&request, &source, &cancel)
            .unwrap()
            .publish();
        let DomSourceProgress::Complete(done) = lease.advance(&cancel).unwrap() else {
            panic!("original synchronous tail completes");
        };
        assert_eq!(
            done.fullscreen_exit_return(),
            Some(DomFullscreenExitReturn::Pending(job.id()))
        );
        ids.push(job.id());
    }
    let before = dom.snapshot();
    assert!(
        before
            .node(subject)
            .unwrap()
            .as_element()
            .unwrap()
            .fullscreen_flag()
    );
    let id = ids[31];
    let work = dom.fullscreen_exits[id.slot].work.clone();
    let mut steps = 0;
    assert_eq!(
        dom.fullscreen_work_custody(&work, || {
            steps += 1;
            if steps > 16 {
                Err(DomError::SourceWorkLimit { limit: 16 })
            } else {
                Ok(())
            }
        }),
        Ok(Some(id)),
        "one exact work identity must not walk unrelated retained invocations"
    );
    assert_eq!(dom.snapshot().stamp(), before.stamp());
    for id in ids {
        assert_eq!(dom.fullscreen_exit_status(id).unwrap().document(), document);
    }
}

#[test]
fn recorded_bound_descendant_context_refuses_context_free_handoff_before_either_publication() {
    use crate::{
        DomInlineStyleChange, DomInlineStyleInputs, DomInlineStyleLimits, DomInlineStyleProfile,
        DomInlineStyleRequestLimits, DomInlineStyleSourceContext,
    };
    use surgeist_css::{CssParserContext, CssParserMode, CssUtf16InputLimits};
    use surgeist_cssom::{
        CssomDeclarationRequestLimits, CssomDeclarationSupport, CssomInputRole, CssomInputVersion,
        CssomLimits,
    };
    let cancel = AtomicBool::new(false);
    let (mut dom, source, document, subject) = fixture();
    let mut batch = DomBatch::new();
    let descendant = batch.create_element_ns(document, crate::Namespace::html(), "div");
    batch.insert_before(&descendant, subject, None);
    let descendant = dom
        .apply(batch, &cancel)
        .unwrap()
        .created(&descendant)
        .unwrap()
        .unwrap();
    dom.configure_inline_styles(
        DomInlineStyleLimits::new(CssomLimits::default(), 8, 64_000_000, 8_000_000, 4_000_000)
            .unwrap(),
        &cancel,
    )
    .unwrap();
    let snapshot = dom.snapshot();
    let document_view = snapshot.document(document).unwrap();
    let parser = CssParserContext::new(
        if document_view.metadata().mode() == crate::HtmlDocumentMode::Quirks {
            CssParserMode::Quirks
        } else {
            CssParserMode::Standards
        },
    );
    let version = CssomInputVersion {
        role: CssomInputRole::Support,
        identity: "fullscreen-inline-empty".into(),
        revision: 1,
    };
    let inputs = DomInlineStyleInputs::new(
        document_view.stamp(),
        parser,
        version.clone(),
        CssomDeclarationSupport::default(),
    )
    .unwrap();
    let requests = DomInlineStyleRequestLimits {
        input: CssUtf16InputLimits::default(),
        declaration: CssomDeclarationRequestLimits::default(),
    };
    let properties = dom
        .inline_style(
            descendant,
            &inputs,
            requests,
            |_, _| panic!("empty attribute"),
            &cancel,
        )
        .unwrap();
    let old = dom.capture_inline_style(descendant).unwrap().unwrap();
    let before = dom.snapshot();
    let context = DomInlineStyleSourceContext::new(
        before.stamp(),
        vec![
            DomInlineStyleProfile::new(
                descendant,
                document,
                parser,
                version,
                CssomDeclarationSupport::default(),
            )
            .unwrap(),
        ],
    )
    .unwrap();
    let mut lease = dom
        .begin_source_edit_with_inline_styles(
            DomSourceEdit::detach(subject),
            context,
            policy(1024),
            requests,
            &cancel,
        )
        .unwrap();
    let operation = lease.operation();
    let progress = lease
        .advance_with_inline_styles(|_, _| panic!("context changes never reparse"), &cancel)
        .unwrap();
    assert!(progress.inline().is_none());
    let DomSourceProgress::Request(DomSourceRequest::FullscreenExitHandoff(request)) =
        progress.into_parts().0
    else {
        panic!("actual removing-step handoff")
    };
    assert_eq!(lease.snapshot().stamp(), before.stamp());
    assert_eq!(
        lease.snapshot().node(subject).unwrap().parent(),
        before.node(subject).unwrap().parent()
    );
    let work = lease.status().spent_work();
    match lease.prepare_fullscreen_exit_handoff(&request, &source, &cancel) {
        Err(error) => assert_eq!(
            error,
            DomError::InlineStyleContextRequired {
                element: descendant,
                change: DomInlineStyleChange::Context
            }
        ),
        Ok(_) => panic!("a context-free handoff cannot publish the recorded context"),
    }
    assert_eq!(lease.operation(), operation);
    assert!(lease.status().spent_work() > work);
    assert!(!lease.status().has_published());
    assert_eq!(lease.snapshot().stamp(), before.stamp());
    assert_eq!(
        lease.snapshot().node(subject).unwrap().parent(),
        before.node(subject).unwrap().parent()
    );
    assert!(
        lease
            .snapshot()
            .node(subject)
            .unwrap()
            .as_element()
            .unwrap()
            .fullscreen_flag()
    );
    assert!(lease.dom.fullscreen_exits.is_empty());
    assert!(matches!(
        lease.dom.capture_inline_style(descendant),
        Err(crate::DomInlineStyleError::Dom(
            DomError::IncompleteSourceOperation
        ))
    ));
    let snapshot = lease.dom.inline_snapshot_for_test().unwrap();
    let current = properties.capture(&snapshot).unwrap();
    assert_eq!(current.handle(), &properties);
    assert_eq!(
        current.snapshot().revision(),
        old.properties().snapshot().revision()
    );
    assert_eq!(
        lease.snapshot().node(descendant).unwrap().tree_scope(),
        old.tree_scope()
    );
    assert_eq!(current.length(requests.declaration).unwrap(), 0);
    assert_eq!(old.properties().length(requests.declaration).unwrap(), 0);
    drop(lease);
    assert_eq!(dom.pending_source_edit().unwrap().operation(), operation);
    // The exposed service request parks the original tail, without claiming a
    // completed orientation service or dropping its mandatory continuation.
}
