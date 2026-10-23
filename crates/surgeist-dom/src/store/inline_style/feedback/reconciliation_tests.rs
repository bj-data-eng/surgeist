//! Real producer terminal transitions and authentic current-state recovery.
use super::*;
use crate::*;
use surgeist_css::*;
use surgeist_cssom::*;

fn flag() -> AtomicBool {
    AtomicBool::new(false)
}
fn limits(work: u64, outputs: usize) -> DomInlineStyleMutationLimits {
    DomInlineStyleMutationLimits {
        declaration: Default::default(),
        feedback: DomInlineStyleFeedbackLimits::new(64_000_000, 1_000_000, work, outputs),
    }
}
fn fixture(identities: u64) -> (Dom, DocumentId, NodeId, NodeId) {
    let mut dom = Dom::new(
        DomLimits::new(
            64,
            100_000,
            128,
            64,
            64,
            16,
            4_000_000,
            DomNameLimits::new(16_777_216, 16_384, 4_000_000, 1_048_576).unwrap(),
        )
        .unwrap(),
    )
    .unwrap();
    let mut b = DomBatch::new();
    let d = b.create_document();
    let a = b.create_element_ns(&d, Namespace::html(), "div");
    let other = b.create_element_ns(&d, Namespace::html(), "aside");
    let c = dom.apply(b, &flag()).unwrap();
    let d = c.document(&d).unwrap();
    let a = c.created(&a).unwrap().unwrap();
    let other = c.created(&other).unwrap().unwrap();
    dom.configure_inline_styles(
        DomInlineStyleLimits::new(
            CssomLimits {
                max_owner_effect_identity: identities,
                max_pending_owner_effects: 1,
                ..Default::default()
            },
            8,
            64_000_000,
            8_000_000,
            4_000_000,
        )
        .unwrap(),
        &flag(),
    )
    .unwrap();
    let input = inputs(&dom, d);
    dom.inline_style(
        a,
        &input,
        DomInlineStyleRequestLimits {
            input: Default::default(),
            declaration: Default::default(),
        },
        |_, _| panic!("empty"),
        &flag(),
    )
    .unwrap();
    (dom, d, a, other)
}
fn inputs(dom: &Dom, d: DocumentId) -> DomInlineStyleInputs {
    DomInlineStyleInputs::new(
        dom.snapshot().document(d).unwrap().stamp(),
        Default::default(),
        CssomInputVersion {
            role: CssomInputRole::Support,
            identity: "reconciliation-support".into(),
            revision: 1,
        },
        CssomDeclarationSupport::try_new(vec![(
            CssPropertyGrammar::from_name("width").unwrap(),
            CssomUsabilityRequirement::WholeDeclarationDecision,
        )])
        .unwrap(),
    )
    .unwrap()
}
fn park(dom: &mut Dom, d: DocumentId, a: NodeId, text: &str, outputs: usize) {
    let input = inputs(dom, d);
    match dom
        .set_inline_style_css_text(
            a,
            &input,
            text,
            limits(0, outputs),
            |_| panic!("custom property"),
            &flag(),
        )
        .unwrap()
    {
        DomInlineStyleMutation::Incomplete { reason, .. } => assert!(matches!(
            reason,
            DomInlineStyleError::Dom(DomError::SourceWorkLimit { limit: 0 })
        )),
        _ => panic!("actual feedback work must refuse"),
    }
}
fn terminal(dom: &mut Dom, status: CssomOwnerEffectStatus) {
    let owner = dom.inline_styles.as_mut().unwrap();
    let effect = &mut owner
        .feedback
        .as_mut()
        .unwrap()
        .value_mut()
        .original
        .as_mut()
        .unwrap()
        .effects_mut()[0];
    match status {
        CssomOwnerEffectStatus::Canceled => effect.cancel().unwrap(),
        CssomOwnerEffectStatus::HostFailed => owner
            .store
            .as_mut()
            .unwrap()
            .begin_owner_application(effect)
            .unwrap()
            .finish_host_failure(),
        CssomOwnerEffectStatus::Abandoned => drop(
            owner
                .store
                .as_mut()
                .unwrap()
                .begin_owner_application(effect)
                .unwrap(),
        ),
        _ => panic!("test must exercise actual terminal protocol"),
    }
}
fn producer_revision(dom: &Dom) -> CssomRevision {
    dom.inline_styles
        .as_ref()
        .unwrap()
        .store
        .as_ref()
        .unwrap()
        .snapshot()
        .revision()
        .clone()
}
fn finish(operation: DomInlineStyleOperation<'_>) -> DomInlineStyleCompletion {
    match operation.complete(&flag()) {
        DomInlineStyleMutation::Complete(c) => c,
        DomInlineStyleMutation::Incomplete { reason, .. } => panic!("feedback refused: {reason:?}"),
    }
}
fn publication(p: &CssomPublication) -> (CssomRevision, CssomRevision) {
    match p {
        CssomPublication::Changed(c) => (c.from().clone(), c.to().clone()),
        CssomPublication::Unchanged { revision } => (revision.clone(), revision.clone()),
    }
}
#[test]
fn all_real_terminal_failures_preserve_original_recovery_and_use_fresh_sync() {
    for status in [
        CssomOwnerEffectStatus::Canceled,
        CssomOwnerEffectStatus::HostFailed,
        CssomOwnerEffectStatus::Abandoned,
    ] {
        let (mut dom, d, a, other) = fixture(2);
        let before = dom.capture_inline_style(a).unwrap().unwrap();
        let input = inputs(&dom, d);
        let DomInlineStyleMutation::Incomplete { reason, .. } = dom
            .set_inline_style_property(
                a,
                &input,
                DomInlineStyleProperty {
                    name: "--x",
                    value: "f([",
                    priority: None,
                },
                limits(0, 64),
                |_| panic!("custom property"),
                &flag(),
            )
            .unwrap()
        else {
            panic!("actual work")
        };
        assert!(matches!(
            reason,
            DomInlineStyleError::Dom(DomError::SourceWorkLimit { limit: 0 })
        ));
        let (old_id, old_block, old_owner, old_publication, diagnostics) = {
            let lease = dom.resume_inline_style_feedback().unwrap();
            let old_id = lease.owner_effects()[0].id().clone();
            let old_block = lease.owner_effects()[0].block().clone();
            let old_owner = lease.owner_effects()[0].owner().clone();
            let old_publication = publication(lease.publication());
            let DomInlineStyleOutcome::Standard { diagnostics, .. } = lease.outcome() else {
                panic!("standard")
            };
            let diagnostics = diagnostics.clone();
            assert_eq!(diagnostics.len(), 2);
            for diag in &diagnostics {
                assert_eq!(diag.error().code(), CssErrorCode::UnexpectedEnd);
                assert_eq!(diag.action(), CssRecoveryAction::RetainWithImplicitClosure);
                assert_eq!(diag.span().start().byte_offset().value(), 3);
                assert_eq!(diag.span().end().byte_offset().value(), 3);
            }
            (old_id, old_block, old_owner, old_publication, diagnostics)
        };
        terminal(&mut dom, status);
        match dom
            .resume_inline_style_feedback()
            .unwrap()
            .complete(&flag())
        {
            DomInlineStyleMutation::Incomplete { operation, reason } => {
                assert!(
                    matches!(reason, DomInlineStyleError::TerminalEffect { status: actual } if actual == status)
                );
                assert_eq!(operation.owner_effects()[0].id(), &old_id);
            }
            _ => panic!("terminal failure cannot become composed success"),
        }
        // Permitted unrelated canonical work changes DOM revision, not this binding.
        let mut b = DomBatch::new();
        b.set_attribute(other, "title", "unrelated");
        dom.apply(b, &flag()).unwrap();
        let canonical = dom.snapshot().stamp();
        let cssom = producer_revision(&dom);
        let (failed, fresh_id) = {
            let (failed, lease) = dom
                .resume_inline_style_feedback()
                .unwrap()
                .reconcile(limits(4_000_000, 64), &flag())
                .unwrap();
            assert_eq!(failed.element(), a);
            assert!(failed.spent_work() > 0);
            let DomInlineStyleOutcome::Standard {
                value,
                diagnostics: kept,
                ..
            } = failed.outcome()
            else {
                panic!("original standard")
            };
            assert!(matches!(value, DomInlineStyleReturn::Unit));
            assert_eq!(kept, &diagnostics);
            for (actual, original) in kept.iter().zip(&diagnostics) {
                assert_eq!(
                    actual.source().map(CssSourceSnapshot::as_str),
                    original.source().map(CssSourceSnapshot::as_str)
                );
                if let (Some(actual), Some(original)) = (actual.source(), original.source()) {
                    assert!(actual.same_snapshot(original));
                }
            }
            assert_eq!(publication(failed.outcome().publication()), old_publication);
            assert_eq!(failed.outcome().owner_effects()[0].id(), &old_id);
            assert_eq!(failed.outcome().owner_effects()[0].status(), status);
            let DomInlineStyleOutcome::Synchronization { commit, .. } = lease.outcome() else {
                panic!("fresh synchronization")
            };
            assert!(commit.owner_effects().is_empty());
            assert!(
                matches!(commit.publication(), CssomPublication::Unchanged { revision } if revision == &cssom)
            );
            let fresh = &lease.owner_effects()[0];
            assert_ne!(fresh.id(), &old_id);
            assert_eq!(fresh.block(), &old_block);
            assert_eq!(fresh.owner(), &old_owner);
            assert_eq!(fresh.value(), "--x: f([]);");
            assert_eq!(fresh.status(), CssomOwnerEffectStatus::Pending);
            let fresh_id = fresh.id().clone();
            (failed, fresh_id)
        };
        assert_eq!(dom.snapshot().stamp(), canonical);
        assert_eq!(producer_revision(&dom), cssom);
        assert!(
            matches!(dom.capture_inline_style(a), Err(DomInlineStyleError::OperationPending { element }) if element == a)
        );
        let completed = finish(dom.resume_inline_style_feedback().unwrap());
        assert_eq!(completed.owner_effects()[0].id(), &fresh_id);
        assert_eq!(
            completed.owner_effects()[0].status(),
            CssomOwnerEffectStatus::Applied
        );
        assert!(matches!(
            completed.outcome(),
            DomInlineStyleOutcome::Synchronization { .. }
        ));
        let current = dom.capture_inline_style(a).unwrap().unwrap();
        assert_eq!(current.properties().handle().id(), &old_block);
        assert_eq!(
            current.properties().css_text(Default::default()).unwrap(),
            "--x: f([]);"
        );
        assert_eq!(
            before.properties().css_text(Default::default()).unwrap(),
            ""
        );
        assert_eq!(failed.outcome().owner_effects()[0].status(), status);
    }
}
#[test]
fn original_removed_string_survives_fresh_empty_owner_write() {
    let (mut dom, d, a, _) = fixture(3);
    let input = inputs(&dom, d);
    let DomInlineStyleMutation::Complete(_) = dom
        .set_inline_style_css_text(
            a,
            &input,
            "--x: RAW",
            limits(4_000_000, 64),
            |_| panic!("custom"),
            &flag(),
        )
        .unwrap()
    else {
        panic!("initial")
    };
    let input = inputs(&dom, d);
    let DomInlineStyleMutation::Incomplete { .. } = dom
        .remove_inline_style_property(a, &input, "--x", limits(0, 64), &flag())
        .unwrap()
    else {
        panic!("actual work")
    };
    terminal(&mut dom, CssomOwnerEffectStatus::Canceled);
    let (failed, lease) = dom
        .resume_inline_style_feedback()
        .unwrap()
        .reconcile(limits(4_000_000, 64), &flag())
        .unwrap();
    assert!(
        matches!(failed.outcome(), DomInlineStyleOutcome::Standard { value: DomInlineStyleReturn::Removed(v), .. } if v == "RAW")
    );
    assert_eq!(lease.owner_effects()[0].value(), "");
    finish(lease);
    let snapshot = dom.snapshot();
    assert_eq!(
        snapshot.node(a).unwrap().attribute(&"style".into()),
        Some(&DocumentString::from(""))
    );
}
#[test]
fn producer_refusals_leave_old_evidence_and_do_not_spend_fresh_identity() {
    for identity_limit in [1, 2] {
        let (mut dom, d, a, _) = fixture(identity_limit);
        park(&mut dom, d, a, "--x: 1", 64);
        terminal(&mut dom, CssomOwnerEffectStatus::Canceled);
        let canonical = dom.snapshot().stamp();
        let cssom = producer_revision(&dom);
        let old = {
            let lease = dom.resume_inline_style_feedback().unwrap();
            lease.owner_effects()[0].id().clone()
        };
        let mut request = limits(4_000_000, 64);
        if identity_limit == 2 {
            request.declaration.provider = CssSerializationLimits::new(1000, 1000, 0);
        }
        let err = dom
            .resume_inline_style_feedback()
            .unwrap()
            .reconcile(request, &flag())
            .err()
            .unwrap();
        if identity_limit == 1 {
            assert!(matches!(
                err,
                DomInlineStyleError::Cssom(CssomError::EffectIdentityExhausted)
            ));
        } else {
            assert!(matches!(
                err,
                DomInlineStyleError::Cssom(CssomError::Declaration(_))
            ));
        }
        assert_eq!(dom.snapshot().stamp(), canonical);
        assert_eq!(producer_revision(&dom), cssom);
        let lease = dom.resume_inline_style_feedback().unwrap();
        assert_eq!(lease.owner_effects()[0].id(), &old);
        assert_eq!(
            lease.owner_effects()[0].status(),
            CssomOwnerEffectStatus::Canceled
        );
        if identity_limit == 2 {
            let (_, lease) = lease.reconcile(limits(4_000_000, 64), &flag()).unwrap();
            finish(lease);
        }
    }
}
#[test]
fn cumulative_outputs_limits_and_work_survive_failed_and_repeated_repairs() {
    let (mut dom, d, a, _) = fixture(3);
    park(&mut dom, d, a, "--x: 1", 1);
    terminal(&mut dom, CssomOwnerEffectStatus::Canceled);
    let old = dom.resume_inline_style_feedback().unwrap().owner_effects()[0]
        .id()
        .clone();
    assert!(matches!(
        dom.resume_inline_style_feedback()
            .unwrap()
            .reconcile(limits(0, 2), &flag()),
        Err(DomInlineStyleError::Dom(DomError::SourceWorkLimit {
            limit: 0
        }))
    ));
    assert!(matches!(
        dom.resume_inline_style_feedback()
            .unwrap()
            .reconcile(limits(4_000_000, 1), &flag()),
        Err(DomInlineStyleError::Dom(DomError::SourceOutputLimit))
    ));
    assert_eq!(
        dom.resume_inline_style_feedback().unwrap().owner_effects()[0].id(),
        &old
    );
    let (first, second_id, spent) = {
        let (first, lease) = dom
            .resume_inline_style_feedback()
            .unwrap()
            .reconcile(limits(4_000_000, 2), &flag())
            .unwrap();
        let second_id = lease.owner_effects()[0].id().clone();
        let spent = lease.spent_work();
        (first, second_id, spent)
    };
    terminal(&mut dom, CssomOwnerEffectStatus::Abandoned);
    assert!(matches!(
        dom.resume_inline_style_feedback()
            .unwrap()
            .reconcile(limits(4_000_000, 2), &flag()),
        Err(DomInlineStyleError::Dom(DomError::SourceOutputLimit))
    ));
    assert!(matches!(
        dom.resume_inline_style_feedback()
            .unwrap()
            .reconcile(limits(0, 64), &flag()),
        Err(DomInlineStyleError::Dom(DomError::SourceLimitReduction))
    ));
    let (second, lease) = dom
        .resume_inline_style_feedback()
        .unwrap()
        .reconcile(limits(4_000_000, 64), &flag())
        .unwrap();
    assert!(lease.spent_work() >= spent);
    assert!(matches!(
        first.outcome(),
        DomInlineStyleOutcome::Standard { .. }
    ));
    assert!(matches!(
        second.outcome(),
        DomInlineStyleOutcome::Synchronization { .. }
    ));
    assert_eq!(second.outcome().owner_effects()[0].id(), &second_id);
    assert_eq!(
        second.outcome().owner_effects()[0].status(),
        CssomOwnerEffectStatus::Abandoned
    );
    assert_ne!(lease.owner_effects()[0].id(), &second_id);
    finish(lease);
    assert_eq!(
        first.outcome().owner_effects()[0].status(),
        CssomOwnerEffectStatus::Canceled
    );
}
#[test]
fn pending_reconciliation_is_refused_and_cancelled_feedback_parks_fresh_effect() {
    let (mut dom, d, a, _) = fixture(2);
    park(&mut dom, d, a, "--x: 1", 64);
    assert!(matches!(
        dom.resume_inline_style_feedback()
            .unwrap()
            .reconcile(limits(4_000_000, 64), &flag()),
        Err(DomInlineStyleError::ReconciliationRequiresTerminal {
            status: CssomOwnerEffectStatus::Pending
        })
    ));
    terminal(&mut dom, CssomOwnerEffectStatus::Canceled);
    let (failed, lease) = dom
        .resume_inline_style_feedback()
        .unwrap()
        .reconcile(limits(4_000_000, 64), &flag())
        .unwrap();
    let id = lease.owner_effects()[0].id().clone();
    let receipt = publication(lease.publication());
    let mut spent = lease.spent_work();
    let canceled = AtomicBool::new(true);
    match lease.complete(&canceled) {
        DomInlineStyleMutation::Incomplete { operation, reason } => {
            assert!(matches!(
                reason,
                DomInlineStyleError::Dom(DomError::Cancelled)
            ));
            assert_eq!(operation.owner_effects()[0].id(), &id);
        }
        _ => panic!("actual cancellation"),
    }
    for _ in 0..2 {
        let lease = dom.resume_inline_style_feedback().unwrap();
        assert_eq!(lease.owner_effects()[0].id(), &id);
        assert_eq!(publication(lease.publication()), receipt);
        assert!(lease.spent_work() >= spent);
        spent = lease.spent_work();
    }
    let c = finish(dom.resume_inline_style_feedback().unwrap());
    assert_eq!(c.owner_effects()[0].id(), &id);
    assert_eq!(
        failed.outcome().owner_effects()[0].status(),
        CssomOwnerEffectStatus::Canceled
    );
}
fn admission(request: &CssomAdmissionRequest<'_>) -> Result<CssomAdmission, CssomError> {
    let context = request.context();
    let support = context
        .inputs
        .iter()
        .find(|i| i.version.role == CssomInputRole::Support)
        .unwrap()
        .version
        .clone();
    Ok(CssomAdmission::new(
        request
            .candidates()
            .iter()
            .map(|c| {
                c.decide(support.clone(), CssomDeclarationUsability::Usable)
                    .unwrap()
            })
            .collect(),
        context.inputs.iter().map(|i| i.version.clone()).collect(),
    ))
}
fn dialog(dom: &mut Dom, d: DocumentId) -> NodeId {
    dom.set_limits(dom.limits().with_dispatch_custody(
        DomDispatchCustodyLimits::new(64, 4096, 16_000_000, 16_000_000, 4_000_000).unwrap(),
    ))
    .unwrap();
    let mut b = DomBatch::new();
    let root = b.create_element_ns(d, Namespace::html(), "main");
    let dialog = b.create_element_ns(d, Namespace::html(), "dialog");
    b.insert_before(&dialog, &root, None);
    b.insert_before(&root, d, None);
    dom.apply(b, &flag())
        .unwrap()
        .created(&dialog)
        .unwrap()
        .unwrap()
}
fn source_limits() -> DomSourceLimits {
    DomSourceLimits::new(64_000_000, 1_000_000, 20_000_000, 128, 256, 8192, 4)
}
#[test]
fn authentic_origin_observer_reconciles_after_receipt_and_preserves_its_tail() {
    let (mut dom, d, a, _) = fixture(2);
    let dialog = dialog(&mut dom, d);
    let executor = dom.bind_source_event_executor(&flag()).unwrap();
    let mut source = dom
        .begin_source_edit(DomSourceEdit::show_dialog(dialog), source_limits(), &flag())
        .unwrap();
    let source_id = source.operation();
    let DomSourceProgress::Request(DomSourceRequest::Event(event)) =
        source.advance(&flag()).unwrap()
    else {
        panic!("BeforeToggle")
    };
    drop(source);
    let mut permit = Some(executor.begin_dispatch(&dom, &event, &flag()).unwrap());
    let scope = executor
        .enter_dispatch(&dom, permit.as_ref().unwrap(), &flag())
        .unwrap();
    let input = inputs(&dom, d);
    let cancel = flag();
    let calls = std::cell::Cell::new(0);
    let DomInlineStyleMutation::Incomplete { reason, .. } = dom
        .set_inline_style_css_text(
            a,
            &input,
            "width: 7px",
            limits(4_000_000, 64),
            |r| {
                calls.set(calls.get() + 1);
                let value = admission(r)?;
                cancel.store(true, Ordering::Relaxed);
                Ok(value)
            },
            &cancel,
        )
        .unwrap()
    else {
        panic!("cancelled feedback")
    };
    assert!(matches!(
        reason,
        DomInlineStyleError::Dom(DomError::Cancelled)
    ));
    cancel.store(false, Ordering::Relaxed);
    drop(scope);
    let mut receipt = Some(
        executor
            .seal_completed(&dom, &mut permit, DomSourceEventOutcome::Canceled, &cancel)
            .unwrap(),
    );
    let mut source = dom
        .resume_source_edit(source_id, source_limits(), &cancel)
        .unwrap();
    source
        .accept_source_event(&event, &mut receipt, &cancel)
        .unwrap();
    assert!(receipt.is_none());
    assert!(
        matches!(source.advance(&cancel),Err(DomError::InlineStyleOperationPending {element}) if element==a)
    );
    let source_work = source.status().spent_work();
    drop(source);
    terminal(&mut dom, CssomOwnerEffectStatus::HostFailed);
    let (failed, fresh) = {
        let (failed, lease) = dom
            .resume_inline_style_feedback()
            .unwrap()
            .reconcile(limits(4_000_000, 64), &cancel)
            .unwrap();
        let fresh = lease.owner_effects()[0].id().clone();
        (failed, fresh)
    };
    let mut source = dom
        .resume_source_edit(source_id, source_limits(), &cancel)
        .unwrap();
    assert!(
        matches!(source.advance(&cancel),Err(DomError::InlineStyleOperationPending {element}) if element==a)
    );
    assert!(source.status().spent_work() >= source_work);
    drop(source);
    let completion = finish(dom.resume_inline_style_feedback().unwrap());
    assert_eq!(completion.owner_effects()[0].id(), &fresh);
    let mut source = dom
        .resume_source_edit(source_id, source_limits(), &cancel)
        .unwrap();
    assert!(matches!(
        source.advance(&cancel).unwrap(),
        DomSourceProgress::Complete(_)
    ));
    drop(source);
    assert!(dom.pending_source_edit().is_none());
    assert_eq!(calls.get(), 1);
    assert_eq!(
        dom.capture_inline_style(a)
            .unwrap()
            .unwrap()
            .properties()
            .get_property_value("width", Default::default())
            .unwrap(),
        "7px"
    );
    assert_eq!(
        failed.outcome().owner_effects()[0].status(),
        CssomOwnerEffectStatus::HostFailed
    );
}
#[test]
fn unrelated_dormant_observer_does_not_authorize_terminal_reconciliation() {
    let (mut dom, d, a, _) = fixture(2);
    let dialog = dialog(&mut dom, d);
    park(&mut dom, d, a, "--x: 1", 64);
    terminal(&mut dom, CssomOwnerEffectStatus::Canceled);
    let cssom = producer_revision(&dom);
    let _executor = dom.bind_source_event_executor(&flag()).unwrap();
    let mut source = dom
        .begin_source_edit(DomSourceEdit::show_dialog(dialog), source_limits(), &flag())
        .unwrap();
    assert!(matches!(
        source.advance(&flag()).unwrap(),
        DomSourceProgress::Request(DomSourceRequest::Event(_))
    ));
    drop(source);
    assert!(matches!(
        dom.resume_inline_style_feedback()
            .unwrap()
            .reconcile(limits(4_000_000, 64), &flag()),
        Err(DomInlineStyleError::Dom(DomError::SourceOperationPending))
    ));
    assert_eq!(producer_revision(&dom), cssom);
    assert_eq!(
        dom.resume_inline_style_feedback().unwrap().owner_effects()[0].status(),
        CssomOwnerEffectStatus::Canceled
    );
}
#[test]
fn exhausted_repair_work_is_retained_and_only_a_raised_cap_allows_progress() {
    let (mut dom, d, a, _) = fixture(260);
    park(&mut dom, d, a, "--x: 1", 64);
    terminal(&mut dom, CssomOwnerEffectStatus::Canceled);
    let original = dom.resume_inline_style_feedback().unwrap().owner_effects()[0]
        .id()
        .clone();
    let mut spent = 0;
    for _ in 0..2 {
        assert!(matches!(
            dom.resume_inline_style_feedback()
                .unwrap()
                .reconcile(limits(1, 64), &flag()),
            Err(DomInlineStyleError::Dom(DomError::SourceWorkLimit {
                limit: 1
            }))
        ));
        let lease = dom.resume_inline_style_feedback().unwrap();
        assert!(lease.spent_work() >= spent);
        spent = lease.spent_work();
        assert_eq!(lease.owner_effects()[0].id(), &original);
    }
    assert!(spent > 0);
    let mut outcomes = Vec::new();
    // Each successful repair consumes real lineage work. A reset on replacement
    // would keep succeeding under this same cap and fail the bounded oracle.
    for _ in 0..=256 {
        let id = dom.resume_inline_style_feedback().unwrap().owner_effects()[0]
            .id()
            .clone();
        match dom
            .resume_inline_style_feedback()
            .unwrap()
            .reconcile(limits(256, 1024), &flag())
        {
            Ok((failed, operation)) => {
                assert_eq!(failed.outcome().owner_effects()[0].id(), &id);
                assert!(operation.spent_work() > spent);
                spent = operation.spent_work();
                outcomes.push(failed);
            }
            Err(DomInlineStyleError::Dom(DomError::SourceWorkLimit { limit: 256 })) => {
                assert!(
                    !outcomes.is_empty(),
                    "the cap must admit a real full repair first"
                );
                let lease = dom.resume_inline_style_feedback().unwrap();
                assert_eq!(lease.owner_effects()[0].id(), &id);
                assert!(lease.spent_work() >= spent);
                let (failed, lease) = lease.reconcile(limits(4_000_000, 1024), &flag()).unwrap();
                assert_eq!(failed.outcome().owner_effects()[0].id(), &id);
                assert!(lease.spent_work() > spent);
                finish(lease);
                assert!(
                    outcomes
                        .iter()
                        .all(|e| e.outcome().owner_effects()[0].status()
                            == CssomOwnerEffectStatus::Canceled)
                );
                return;
            }
            Err(other) => panic!("work exhaustion: {other:?}"),
        }
        terminal(&mut dom, CssomOwnerEffectStatus::Canceled);
    }
    panic!("same finite lineage work cap did not exhaust");
}
#[test]
fn finite_retained_envelope_refuses_storage_before_new_producer_identity() {
    // Small finite envelopes exercise real controlled storage. Earlier Source
    // formation may refuse before the replacement holder; both precede CSSOM.
    for extra in [4096, 8192, 16_384, 32_768] {
        let (mut dom, d, a, _) = fixture(2);
        let bytes = dom
            .custody_registry
            .ledger()
            .total()
            .bytes()
            .checked_add(extra)
            .unwrap();
        let mut request = limits(0, 64);
        request.feedback.retained_bytes = bytes;
        let input = inputs(&dom, d);
        match dom.set_inline_style_css_text(
            a,
            &input,
            "--x: 1",
            request,
            |_| panic!("custom"),
            &flag(),
        ) {
            Ok(DomInlineStyleMutation::Incomplete { reason, .. }) => assert!(matches!(
                reason,
                DomInlineStyleError::Dom(DomError::SourceWorkLimit { limit: 0 })
            )),
            Err(DomInlineStyleError::Dom(DomError::ResourceLimit {
                resource: DomResource::SourceRetainedBytes,
                ..
            })) => continue,
            Err(other) => panic!("initial admission: {other:?}"),
            _ => panic!("actual work must refuse"),
        }
        terminal(&mut dom, CssomOwnerEffectStatus::Canceled);
        let canonical = dom.snapshot().stamp();
        let cssom = producer_revision(&dom);
        let id = dom.resume_inline_style_feedback().unwrap().owner_effects()[0]
            .id()
            .clone();
        request.feedback.work = 4_000_000;
        match dom
            .resume_inline_style_feedback()
            .unwrap()
            .reconcile(request, &flag())
        {
            Ok((_, _lease)) => {}
            Err(DomInlineStyleError::Dom(DomError::ResourceLimit {
                resource: DomResource::SourceRetainedBytes,
                ..
            })) => {
                assert_eq!(dom.snapshot().stamp(), canonical);
                assert_eq!(producer_revision(&dom), cssom);
                let lease = dom.resume_inline_style_feedback().unwrap();
                assert_eq!(lease.owner_effects()[0].id(), &id);
                assert_eq!(
                    lease.owner_effects()[0].status(),
                    CssomOwnerEffectStatus::Canceled
                );
                let (failed, lease) = lease.reconcile(limits(4_000_000, 64), &flag()).unwrap();
                assert_eq!(failed.outcome().owner_effects()[0].id(), &id);
                // The original used identity1. With ceiling2, a successful retry
                // proves the retained-storage refusal did not spend identity2.
                finish(lease);
                assert_eq!(
                    failed.outcome().owner_effects()[0].status(),
                    CssomOwnerEffectStatus::Canceled
                );
                return;
            }
            Err(other) => panic!("storage admission: {other:?}"),
        }
    }
    panic!("finite fixture did not reach storage refusal");
}
#[test]
fn fresh_sync_survives_canonical_refusal_and_resumes_without_resolver_reentry() {
    let (mut dom, d, a, _) = fixture(2);
    let old = dom.capture_inline_style(a).unwrap().unwrap();
    let input = inputs(&dom, d);
    let cancel = flag();
    let calls = std::cell::Cell::new(0);
    let DomInlineStyleMutation::Incomplete { reason, .. } = dom
        .set_inline_style_property(
            a,
            &input,
            DomInlineStyleProperty {
                name: "width",
                value: "7px",
                priority: None,
            },
            limits(4_000_000, 64),
            |request| {
                calls.set(calls.get() + 1);
                let admitted = admission(request)?;
                cancel.store(true, Ordering::Relaxed);
                Ok(admitted)
            },
            &cancel,
        )
        .unwrap()
    else {
        panic!("postcommit cancellation")
    };
    assert!(matches!(
        reason,
        DomInlineStyleError::Dom(DomError::Cancelled)
    ));
    cancel.store(false, Ordering::Relaxed);
    terminal(&mut dom, CssomOwnerEffectStatus::Abandoned);
    let (failed, id, receipt, spent) = {
        let (failed, lease) = dom
            .resume_inline_style_feedback()
            .unwrap()
            .reconcile(limits(4_000_000, 64), &cancel)
            .unwrap();
        let id = lease.owner_effects()[0].id().clone();
        let receipt = publication(lease.publication());
        let spent = lease.spent_work();
        (failed, id, receipt, spent)
    };
    let original_limits = dom.limits();
    let limited = DomLimits::new(
        64,
        dom.usage().string_code_units(),
        128,
        64,
        64,
        16,
        4_000_000,
        DomNameLimits::new(16_777_216, 16_384, 4_000_000, 1_048_576).unwrap(),
    )
    .unwrap();
    dom.set_limits(limited).unwrap();
    match dom
        .resume_inline_style_feedback()
        .unwrap()
        .complete(&cancel)
    {
        DomInlineStyleMutation::Incomplete { operation, reason } => {
            assert!(matches!(
                reason,
                DomInlineStyleError::Dom(DomError::ResourceLimit {
                    resource: DomResource::StringCodeUnits,
                    ..
                })
            ));
            assert_eq!(operation.owner_effects()[0].id(), &id);
            assert_eq!(
                operation.owner_effects()[0].status(),
                CssomOwnerEffectStatus::Pending
            );
            assert!(operation.spent_work() >= spent);
        }
        _ => panic!("real canonical storage refusal"),
    }
    dom.set_limits(original_limits).unwrap();
    let lease = dom.resume_inline_style_feedback().unwrap();
    assert_eq!(publication(lease.publication()), receipt);
    let completed = finish(lease);
    assert_eq!(completed.owner_effects()[0].id(), &id);
    assert_eq!(calls.get(), 1);
    assert_eq!(old.properties().css_text(Default::default()).unwrap(), "");
    assert_eq!(
        dom.capture_inline_style(a)
            .unwrap()
            .unwrap()
            .properties()
            .get_property_value("width", Default::default())
            .unwrap(),
        "7px"
    );
    assert_eq!(
        failed.outcome().owner_effects()[0].status(),
        CssomOwnerEffectStatus::Abandoned
    );
}
