//! Standard object executors and actual affine canonical feedback.
use std::{
    cell::Cell,
    sync::atomic::{AtomicBool, Ordering},
};
use surgeist_css::*;
use surgeist_cssom::*;
use surgeist_dom::*;
fn cancel() -> AtomicBool {
    AtomicBool::new(false)
}
fn request() -> DomInlineStyleMutationLimits {
    DomInlineStyleMutationLimits {
        declaration: Default::default(),
        feedback: DomInlineStyleFeedbackLimits::new(64_000_000, 1_000_000, 4_000_000, 64),
    }
}
fn inputs(dom: &Dom, document: DocumentId) -> DomInlineStyleInputs {
    inputs_with_requirement(dom, document, CssomUsabilityRequirement::AllCheckedValues)
}
fn inputs_with_requirement(
    dom: &Dom,
    document: DocumentId,
    requirement: CssomUsabilityRequirement,
) -> DomInlineStyleInputs {
    DomInlineStyleInputs::new(
        dom.snapshot().document(document).unwrap().stamp(),
        CssParserContext::default(),
        CssomInputVersion {
            role: CssomInputRole::Support,
            identity: if requirement == CssomUsabilityRequirement::WholeDeclarationDecision {
                "feedback-support-whole-declaration"
            } else {
                "feedback-support"
            }
            .into(),
            revision: 1,
        },
        CssomDeclarationSupport::try_new(vec![(
            CssPropertyGrammar::from_name("width").unwrap(),
            requirement,
        )])
        .unwrap(),
    )
    .unwrap()
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
fn dom_limits(units: usize) -> DomLimits {
    DomLimits::new(
        64,
        units,
        128,
        64,
        64,
        16,
        4_000_000,
        DomNameLimits::new(16_777_216, 16_384, 4_000_000, 1_048_576).unwrap(),
    )
    .unwrap()
}
fn fixture() -> (Dom, DocumentId, NodeId, NodeId) {
    fixture_with_requirement(CssomUsabilityRequirement::AllCheckedValues)
}
fn fixture_with_requirement(
    requirement: CssomUsabilityRequirement,
) -> (Dom, DocumentId, NodeId, NodeId) {
    let mut dom = Dom::new(dom_limits(100_000)).unwrap();
    let mut b = DomBatch::new();
    let d = b.create_document();
    let a = b.create_element_ns(&d, Namespace::html(), "div");
    let other = b.create_element_ns(&d, Namespace::html(), "aside");
    let c = dom.apply(b, &cancel()).unwrap();
    let d = c.document(&d).unwrap();
    let a = c.created(&a).unwrap().unwrap();
    let other = c.created(&other).unwrap().unwrap();
    dom.configure_inline_styles(
        DomInlineStyleLimits::new(CssomLimits::default(), 8, 64_000_000, 8_000_000, 4_000_000)
            .unwrap(),
        &cancel(),
    )
    .unwrap();
    for e in [a, other] {
        let i = inputs_with_requirement(&dom, d, requirement);
        dom.inline_style(
            e,
            &i,
            DomInlineStyleRequestLimits {
                input: Default::default(),
                declaration: Default::default(),
            },
            |_, _| panic!("empty"),
            &cancel(),
        )
        .unwrap();
    }
    (dom, d, a, other)
}
fn standard_value(outcome: &DomInlineStyleOutcome) -> &DomInlineStyleReturn {
    match outcome {
        DomInlineStyleOutcome::Standard { value, .. } => value,
        DomInlineStyleOutcome::Synchronization { .. } => panic!("expected standard executor"),
    }
}
fn complete(result: DomInlineStyleMutation<'_>) -> DomInlineStyleCompletion {
    match result {
        DomInlineStyleMutation::Complete(c) => c,
        DomInlineStyleMutation::Incomplete { reason, .. } => {
            panic!("unexpected feedback refusal: {reason:?}")
        }
    }
}
fn endpoints(p: &CssomPublication) -> (CssomRevision, CssomRevision) {
    match p {
        CssomPublication::Changed(c) => (c.from().clone(), c.to().clone()),
        CssomPublication::Unchanged { revision } => (revision.clone(), revision.clone()),
    }
}
fn width(c: &DomInlineStyleCapture) -> String {
    c.properties()
        .get_property_value("width", Default::default())
        .unwrap()
}
#[test]
fn standard_setters_and_remove_move_real_results_and_apply_canonical_echo_once() {
    let (mut dom, d, a, _) = fixture();
    let old = dom.capture_inline_style(a).unwrap().unwrap();
    let i = inputs(&dom, d);
    let c = complete(
        dom.set_inline_style_css_text(
            a,
            &i,
            "width: 3px; --Case: RAW",
            request(),
            admission,
            &cancel(),
        )
        .unwrap(),
    );
    assert!(matches!(
        standard_value(c.outcome()),
        DomInlineStyleReturn::Unit
    ));
    assert_eq!(c.owner_effects().len(), 1);
    assert_eq!(
        c.owner_effects()[0].status(),
        CssomOwnerEffectStatus::Applied
    );
    assert!(c.dom().is_some());
    assert_eq!(width(&dom.capture_inline_style(a).unwrap().unwrap()), "3px");
    let capture = dom.capture_inline_style(a).unwrap().unwrap();
    assert_eq!(
        capture
            .dom()
            .node(a)
            .unwrap()
            .as_element()
            .unwrap()
            .attribute(&DocumentString::from("style"))
            .unwrap()
            .to_utf8()
            .unwrap(),
        "width: 3px; --Case: RAW;"
    );
    assert_eq!(old.properties().handle(), capture.properties().handle());
    assert_eq!(width(&old), "");
    let i = inputs(&dom, d);
    let c = complete(
        dom.set_inline_style_property(
            a,
            &i,
            DomInlineStyleProperty {
                name: "width",
                value: "4px",
                priority: Some("important"),
            },
            request(),
            admission,
            &cancel(),
        )
        .unwrap(),
    );
    assert_eq!(
        c.owner_effects()[0].status(),
        CssomOwnerEffectStatus::Applied
    );
    let i = inputs(&dom, d);
    let c = complete(
        dom.remove_inline_style_property(a, &i, "width", request(), &cancel())
            .unwrap(),
    );
    assert!(matches!(standard_value(c.outcome()),DomInlineStyleReturn::Removed(v) if v=="4px"));
    assert_eq!(width(&dom.capture_inline_style(a).unwrap().unwrap()), "");
    assert_eq!(
        c.owner_effects()[0].status(),
        CssomOwnerEffectStatus::Applied
    );
}
#[test]
fn cancellation_after_actual_cssom_commit_parks_and_resumes_without_resolver_replay() {
    let (mut dom, d, a, other) =
        fixture_with_requirement(CssomUsabilityRequirement::WholeDeclarationDecision);
    let old = dom.capture_inline_style(a).unwrap().unwrap();
    let before = dom.snapshot().stamp();
    let flag = cancel();
    let calls = Cell::new(0);
    let i = inputs_with_requirement(&dom, d, CssomUsabilityRequirement::WholeDeclarationDecision);
    let pending = dom
        .set_inline_style_css_text(
            a,
            &i,
            "width: 7px",
            request(),
            |r| {
                calls.set(calls.get() + 1);
                let a = admission(r)?;
                flag.store(true, Ordering::Relaxed);
                Ok(a)
            },
            &flag,
        )
        .unwrap();
    let (published, spent) = match pending {
        DomInlineStyleMutation::Incomplete { operation, reason } => {
            assert!(matches!(
                reason,
                DomInlineStyleError::Dom(DomError::Cancelled)
            ));
            assert_eq!(
                operation.owner_effects()[0].status(),
                CssomOwnerEffectStatus::Pending
            );
            (operation.publication().clone(), operation.spent_work())
        }
        _ => panic!("must retain committed outcome"),
    };
    assert_eq!(calls.get(), 1);
    assert_eq!(dom.snapshot().stamp(), before);
    assert!(
        matches!(dom.capture_inline_style(a),Err(DomInlineStyleError::OperationPending{element}) if element==a)
    );
    assert!(dom.capture_inline_style(other).unwrap().is_some());
    flag.store(false, Ordering::Relaxed);
    let lease = dom.resume_inline_style_feedback().unwrap();
    assert!(lease.spent_work() >= spent);
    assert_eq!(endpoints(lease.publication()), endpoints(&published));
    let completion = complete(lease.complete(&flag));
    assert_eq!(endpoints(completion.publication()), endpoints(&published));
    assert_eq!(calls.get(), 1);
    assert_eq!(
        completion.owner_effects()[0].status(),
        CssomOwnerEffectStatus::Applied
    );
    assert_eq!(width(&dom.capture_inline_style(a).unwrap().unwrap()), "7px");
    assert_eq!(width(&old), "");
}
#[test]
fn wrapper_refusal_precedes_mutation_and_changed_external_support_cannot_impersonate_packet() {
    let (mut dom, d, a, _) = fixture();
    let old = dom.capture_inline_style(a).unwrap().unwrap();
    let i = inputs(&dom, d);
    let mut limits = request();
    limits.feedback = DomInlineStyleFeedbackLimits::new(64_000_000, 1_000_000, 4_000_000, 0);
    assert!(matches!(
        dom.set_inline_style_css_text(
            a,
            &i,
            "width: 1px",
            limits,
            |_| panic!("wrapper refuses first"),
            &cancel()
        ),
        Err(DomInlineStyleError::Dom(DomError::SourceOutputLimit))
    ));
    let bad = DomInlineStyleInputs::new(
        dom.snapshot().document(d).unwrap().stamp(),
        CssParserContext::default(),
        CssomInputVersion {
            role: CssomInputRole::Support,
            identity: "feedback-support".into(),
            revision: 2,
        },
        CssomDeclarationSupport::default(),
    )
    .unwrap();
    assert!(matches!(
        dom.set_inline_style_css_text(
            a,
            &bad,
            "width: 2px",
            request(),
            |_| panic!("profile refuses first"),
            &cancel()
        ),
        Err(DomInlineStyleError::ContextUpdateRequired)
    ));
    assert_eq!(
        old.properties().snapshot().revision(),
        dom.capture_inline_style(a)
            .unwrap()
            .unwrap()
            .properties()
            .snapshot()
            .revision()
    );
    assert_eq!(old.dom().stamp(), dom.snapshot().stamp());
}
#[test]
fn last_removal_keeps_empty_attribute_and_second_remove_has_no_echo_publication() {
    let (mut dom, d, a, _) = fixture();
    let i = inputs(&dom, d);
    complete(
        dom.set_inline_style_property(
            a,
            &i,
            DomInlineStyleProperty {
                name: "width",
                value: "9px",
                priority: None,
            },
            request(),
            admission,
            &cancel(),
        )
        .unwrap(),
    );
    let i = inputs(&dom, d);
    let c = complete(
        dom.remove_inline_style_property(a, &i, "width", request(), &cancel())
            .unwrap(),
    );
    assert!(matches!(standard_value(c.outcome()),DomInlineStyleReturn::Removed(v) if v=="9px"));
    assert_eq!(
        dom.snapshot()
            .node(a)
            .unwrap()
            .as_element()
            .unwrap()
            .attribute(&DocumentString::from("style"))
            .unwrap()
            .to_utf8()
            .unwrap(),
        ""
    );
    let before = dom.snapshot().stamp();
    let i = inputs(&dom, d);
    let c = complete(
        dom.remove_inline_style_property(a, &i, "width", request(), &cancel())
            .unwrap(),
    );
    assert!(matches!(standard_value(c.outcome()),DomInlineStyleReturn::Removed(v) if v.is_empty()));
    assert!(c.owner_effects().is_empty());
    assert!(c.dom().is_none());
    assert_eq!(dom.snapshot().stamp(), before);
}
#[test]
fn effect_free_equal_set_preserves_raw_spelling_and_echo_does_not_suppress_later_dom_ingress() {
    let (mut dom, d, a, _) = fixture();
    let mut b = DomBatch::new();
    b.set_attribute(a, "style", "width: 3px");
    let row = DomInlineStyleProfile::new(
        a,
        d,
        CssParserContext::default(),
        CssomInputVersion {
            role: CssomInputRole::Support,
            identity: "feedback-support".into(),
            revision: 1,
        },
        CssomDeclarationSupport::try_new(vec![(
            CssPropertyGrammar::from_name("width").unwrap(),
            CssomUsabilityRequirement::AllCheckedValues,
        )])
        .unwrap(),
    )
    .unwrap();
    let context =
        DomInlineStyleBatchContext::new(&b, dom.snapshot().stamp(), vec![row.clone()]).unwrap();
    dom.apply_with_inline_styles(
        b,
        context,
        DomInlineStyleRequestLimits {
            input: Default::default(),
            declaration: Default::default(),
        },
        |c, ctx| {
            let support = ctx
                .inputs
                .iter()
                .find(|i| i.version.role == CssomInputRole::Support)
                .unwrap()
                .version
                .clone();
            Ok(CssomAdmission::new(
                c.iter()
                    .map(|c| {
                        c.decide(support.clone(), CssomDeclarationUsability::Usable)
                            .unwrap()
                    })
                    .collect(),
                ctx.inputs.iter().map(|i| i.version.clone()).collect(),
            ))
        },
        &cancel(),
    )
    .unwrap();
    let before = dom.snapshot().stamp();
    let i = inputs(&dom, d);
    let c = complete(
        dom.set_inline_style_property(
            a,
            &i,
            DomInlineStyleProperty {
                name: "width",
                value: "3px",
                priority: None,
            },
            request(),
            admission,
            &cancel(),
        )
        .unwrap(),
    );
    assert!(c.owner_effects().is_empty());
    assert!(c.dom().is_none());
    assert_eq!(dom.snapshot().stamp(), before);
    assert_eq!(
        dom.snapshot()
            .node(a)
            .unwrap()
            .as_element()
            .unwrap()
            .attribute(&DocumentString::from("style"))
            .unwrap()
            .to_utf8()
            .unwrap(),
        "width: 3px"
    );
    let i = inputs(&dom, d);
    complete(
        dom.set_inline_style_property(
            a,
            &i,
            DomInlineStyleProperty {
                name: "width",
                value: "4px",
                priority: None,
            },
            request(),
            admission,
            &cancel(),
        )
        .unwrap(),
    );
    let mut b = DomBatch::new();
    b.set_attribute(a, "style", "width: 4px;");
    assert!(
        matches!(dom.apply(b,&cancel()),Err(DomError::InlineStyleContextRequired{element,change:DomInlineStyleChange::Attribute}) if element==a)
    );
}
#[test]
fn nonzero_work_refusal_retains_original_effect_and_blocks_only_relevant_work() {
    let (mut dom, d, a, other) = fixture();
    let old = dom.capture_inline_style(a).unwrap().unwrap();
    let units = dom.usage().string_code_units();
    dom.set_limits(dom_limits(units)).unwrap();
    let i = inputs(&dom, d);
    let pending = dom
        .set_inline_style_css_text(a, &i, "width: 12345px", request(), admission, &cancel())
        .unwrap();
    let (effect, pub_end, spent) = match pending {
        DomInlineStyleMutation::Incomplete { operation, reason } => {
            assert!(matches!(
                reason,
                DomInlineStyleError::Dom(DomError::ResourceLimit {
                    resource: DomResource::StringCodeUnits,
                    ..
                })
            ));
            assert!(operation.spent_work() > 0);
            assert_eq!(
                operation.owner_effects()[0].status(),
                CssomOwnerEffectStatus::Pending
            );
            (
                operation.owner_effects()[0].id().clone(),
                endpoints(operation.publication()),
                operation.spent_work(),
            )
        }
        _ => panic!("canonical capacity must refuse"),
    };
    dom.set_limits(dom_limits(100_000)).unwrap();
    let before = dom.snapshot().stamp();
    let i = inputs(&dom, d);
    assert!(
        matches!(dom.set_inline_style_css_text(other,&i,"width: 2px",request(),|_|panic!("serialized before resolver"),&cancel()),Err(DomInlineStyleError::OperationPending{element}) if element==a)
    );
    let mut b = DomBatch::new();
    b.insert_before(a, other, None);
    assert!(
        matches!(dom.apply(b,&cancel()),Err(DomError::InlineStyleOperationPending{element}) if element==a)
    );
    assert_eq!(dom.snapshot().stamp(), before);
    let mut unrelated = DomBatch::new();
    unrelated.set_attribute(other, "title", "allowed");
    dom.apply(unrelated, &cancel()).unwrap();
    assert!(dom.capture_inline_style(other).unwrap().is_some());
    let lease = dom.resume_inline_style_feedback().unwrap();
    assert!(lease.spent_work() >= spent);
    assert_eq!(lease.owner_effects()[0].id(), &effect);
    assert_eq!(endpoints(lease.publication()), pub_end);
    let c = complete(lease.complete(&cancel()));
    assert_eq!(c.owner_effects()[0].id(), &effect);
    assert_eq!(endpoints(c.publication()), pub_end);
    assert_eq!(
        width(&dom.capture_inline_style(a).unwrap().unwrap()),
        "12345px"
    );
    assert_eq!(width(&old), "");
}

#[test]
fn original_observer_can_drain_parked_feedback_after_scope_and_receipt_completion() {
    let (mut dom, d, a, _) =
        fixture_with_requirement(CssomUsabilityRequirement::WholeDeclarationDecision);
    let limits = dom.limits().with_dispatch_custody(
        DomDispatchCustodyLimits::new(64, 4096, 16_000_000, 16_000_000, 4_000_000).unwrap(),
    );
    dom.set_limits(limits).unwrap();
    let mut b = DomBatch::new();
    let root = b.create_element_ns(d, Namespace::html(), "main");
    let dialog = b.create_element_ns(d, Namespace::html(), "dialog");
    b.insert_before(&dialog, &root, None);
    b.insert_before(&root, d, None);
    let c = dom.apply(b, &cancel()).unwrap();
    let dialog = c.created(&dialog).unwrap().unwrap();
    let executor = dom.bind_source_event_executor(&cancel()).unwrap();
    let source_limits = DomSourceLimits::new(64_000_000, 1_000_000, 20_000_000, 128, 256, 8192, 4);
    let mut source = dom
        .begin_source_edit(DomSourceEdit::show_dialog(dialog), source_limits, &cancel())
        .unwrap();
    let source_id = source.operation();
    let DomSourceProgress::Request(DomSourceRequest::Event(event)) =
        source.advance(&cancel()).unwrap()
    else {
        panic!("genuine BeforeToggle");
    };
    drop(source);
    let mut permit = Some(executor.begin_dispatch(&dom, &event, &cancel()).unwrap());
    let scope = executor
        .enter_dispatch(&dom, permit.as_ref().unwrap(), &cancel())
        .unwrap();
    let input =
        inputs_with_requirement(&dom, d, CssomUsabilityRequirement::WholeDeclarationDecision);
    let flag = cancel();
    let calls = Cell::new(0);
    let pending = dom
        .set_inline_style_css_text(
            a,
            &input,
            "width: 7px",
            request(),
            |r| {
                calls.set(calls.get() + 1);
                let admission = admission(r)?;
                flag.store(true, Ordering::Relaxed);
                Ok(admission)
            },
            &flag,
        )
        .unwrap();
    let effect = match pending {
        DomInlineStyleMutation::Incomplete { operation, reason } => {
            assert!(matches!(
                reason,
                DomInlineStyleError::Dom(DomError::Cancelled)
            ));
            operation.owner_effects()[0].id().clone()
        }
        _ => panic!("actual committed outcome remains Pending"),
    };
    flag.store(false, Ordering::Relaxed);
    drop(scope); // No longer grants ordinary callback reentry.
    let mut receipt = Some(
        executor
            .seal_completed(&dom, &mut permit, DomSourceEventOutcome::Canceled, &flag)
            .unwrap(),
    );
    let mut source = dom
        .resume_source_edit(source_id, source_limits, &flag)
        .unwrap();
    source
        .accept_source_event(&event, &mut receipt, &flag)
        .unwrap();
    assert!(
        matches!(source.advance(&flag), Err(DomError::InlineStyleOperationPending { element }) if element == a)
    );
    let source_work = source.status().spent_work();
    drop(source);
    let lease = dom.resume_inline_style_feedback().unwrap();
    assert_eq!(lease.owner_effects()[0].id(), &effect);
    let completion = complete(lease.complete(&flag));
    assert_eq!(completion.owner_effects()[0].id(), &effect);
    assert_eq!(
        completion.owner_effects()[0].status(),
        CssomOwnerEffectStatus::Applied
    );
    assert_eq!(calls.get(), 1);
    let mut source = dom
        .resume_source_edit(source_id, source_limits, &flag)
        .unwrap();
    assert!(source.status().spent_work() >= source_work);
    assert!(matches!(
        source.advance(&flag).unwrap(),
        DomSourceProgress::Complete(_)
    ));
    drop(source);
    assert_eq!(width(&dom.capture_inline_style(a).unwrap().unwrap()), "7px");
    assert!(dom.pending_source_edit().is_none());
}

#[test]
fn effect_free_standard_result_completes_even_when_its_resolver_requested_cancellation() {
    let (mut dom, document, element, _) =
        fixture_with_requirement(CssomUsabilityRequirement::WholeDeclarationDecision);
    let input = inputs_with_requirement(
        &dom,
        document,
        CssomUsabilityRequirement::WholeDeclarationDecision,
    );
    complete(
        dom.set_inline_style_property(
            element,
            &input,
            DomInlineStyleProperty {
                name: "width",
                value: "3px",
                priority: None,
            },
            request(),
            admission,
            &cancel(),
        )
        .unwrap(),
    );
    let before = dom.snapshot().stamp();
    let input = inputs_with_requirement(
        &dom,
        document,
        CssomUsabilityRequirement::WholeDeclarationDecision,
    );
    let flag = cancel();
    let calls = Cell::new(0);
    let outcome = complete(
        dom.set_inline_style_property(
            element,
            &input,
            DomInlineStyleProperty {
                name: "width",
                value: "3px",
                priority: None,
            },
            request(),
            |r| {
                calls.set(calls.get() + 1);
                let result = admission(r)?;
                flag.store(true, Ordering::Relaxed);
                Ok(result)
            },
            &flag,
        )
        .unwrap(),
    );
    assert_eq!(calls.get(), 1);
    assert!(flag.load(Ordering::Relaxed));
    assert!(outcome.owner_effects().is_empty());
    assert!(outcome.dom().is_none());
    assert!(matches!(
        standard_value(outcome.outcome()),
        DomInlineStyleReturn::Unit
    ));
    assert_eq!(dom.snapshot().stamp(), before);
    assert_eq!(
        width(&dom.capture_inline_style(element).unwrap().unwrap()),
        "3px"
    );
}

#[test]
fn producer_resolver_unwind_discards_only_uncommitted_holder_and_allows_next_operation() {
    let (mut dom, document, element, _) =
        fixture_with_requirement(CssomUsabilityRequirement::WholeDeclarationDecision);
    let before = dom.capture_inline_style(element).unwrap().unwrap();
    let input = inputs_with_requirement(
        &dom,
        document,
        CssomUsabilityRequirement::WholeDeclarationDecision,
    );
    let panic = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
        let _ = dom.set_inline_style_css_text(
            element,
            &input,
            "width: 3px",
            request(),
            |_| std::panic::panic_any(29u32),
            &cancel(),
        );
    }))
    .unwrap_err();
    assert_eq!(panic.downcast_ref::<u32>(), Some(&29));
    let current = dom.capture_inline_style(element).unwrap().unwrap();
    assert_eq!(current.dom().stamp(), before.dom().stamp());
    assert_eq!(
        current.properties().snapshot().revision(),
        before.properties().snapshot().revision()
    );
    assert!(matches!(
        dom.resume_inline_style_feedback(),
        Err(DomInlineStyleError::Unbound)
    ));
    complete(
        dom.set_inline_style_css_text(
            element,
            &input,
            "width: 3px",
            request(),
            admission,
            &cancel(),
        )
        .unwrap(),
    );
    assert_eq!(
        width(&dom.capture_inline_style(element).unwrap().unwrap()),
        "3px"
    );
}
