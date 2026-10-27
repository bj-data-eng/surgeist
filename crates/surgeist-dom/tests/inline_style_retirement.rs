//! Atomic bound-inline retirement, exact live identities and historical captures.
use std::sync::atomic::AtomicBool;
use surgeist_css::property::CssPropertyGrammar;
use surgeist_css::source::CssUtf16InputLimits;
use surgeist_css::{CssParserContext, CssParserMode};
use surgeist_cssom::*;
use surgeist_dom::*;

fn cancel() -> AtomicBool {
    AtomicBool::new(false)
}
fn dom_limits(nodes: usize) -> DomLimits {
    DomLimits::new(
        nodes,
        100_000,
        512,
        256,
        256,
        32,
        4_000_000,
        DomNameLimits::new(16_777_216, 16_384, 4_000_000, 1_048_576).unwrap(),
    )
    .unwrap()
}
fn requests() -> DomInlineStyleRequestLimits {
    DomInlineStyleRequestLimits {
        input: CssUtf16InputLimits::default(),
        declaration: CssomDeclarationRequestLimits::default(),
    }
}
fn source_limits(outputs: usize) -> DomSourceLimits {
    DomSourceLimits::new(64_000_000, 1_000_000, 20_000_000, 128, outputs, 8192, 4)
}
fn version() -> CssomInputVersion {
    CssomInputVersion {
        role: CssomInputRole::Support,
        identity: "retirement-width-profile".into(),
        revision: 1,
    }
}
fn support() -> CssomDeclarationSupport {
    CssomDeclarationSupport::try_new(vec![(
        CssPropertyGrammar::from_name("width").unwrap(),
        CssomUsabilityRequirement::AllCheckedValues,
    )])
    .unwrap()
}
fn parser(dom: &Dom, document: DocumentId) -> CssParserContext {
    CssParserContext::new(
        if dom.snapshot().document(document).unwrap().metadata().mode() == HtmlDocumentMode::Quirks
        {
            CssParserMode::Quirks
        } else {
            CssParserMode::Standards
        },
    )
}
fn inputs(dom: &Dom, document: DocumentId) -> DomInlineStyleInputs {
    DomInlineStyleInputs::new(
        dom.snapshot().document(document).unwrap().stamp(),
        parser(dom, document),
        version(),
        support(),
    )
    .unwrap()
}
fn no_decisions(
    _: &[CssomDeclarationCandidate],
    _: &CssomContext,
) -> Result<CssomAdmission, CssomError> {
    panic!("retirement and checked width values require no admission resolver")
}
fn bind(dom: &mut Dom, document: DocumentId, element: NodeId) -> CssStyleProperties {
    let input = inputs(dom, document);
    dom.inline_style(element, &input, requests(), no_decisions, &cancel())
        .unwrap()
}
fn capture(dom: &Dom, element: NodeId) -> DomInlineStyleCapture {
    dom.capture_inline_style(element).unwrap().unwrap()
}
fn width(capture: &DomInlineStyleCapture) -> String {
    capture
        .properties()
        .get_property_value("width", requests().declaration)
        .unwrap()
}
fn owner(bindings: usize, nodes: usize, cssom: CssomLimits) -> (Dom, DocumentId) {
    let mut dom = Dom::new(dom_limits(nodes)).unwrap();
    let mut batch = DomBatch::new();
    let document = batch.create_document();
    let commit = dom.apply(batch, &cancel()).unwrap();
    let document = commit.document(&document).unwrap();
    dom.configure_inline_styles(
        DomInlineStyleLimits::new(cssom, bindings, 64_000_000, 8_000_000, 4_000_000).unwrap(),
        &cancel(),
    )
    .unwrap();
    (dom, document)
}
struct Fixture {
    dom: Dom,
    document: DocumentId,
    root: NodeId,
    first: NodeId,
    second: NodeId,
    keeper: NodeId,
}
fn fixture(cssom: CssomLimits) -> Fixture {
    let (mut dom, document) = owner(3, 128, cssom);
    let mut batch = DomBatch::new();
    // The root is already detached. Destroy still runs each real child removal
    // and its context hooks before retirement; those live contexts are explicit.
    let root = batch.create_element_ns(document, Namespace::html(), "section");
    let first = batch.create_element_ns(document, Namespace::html(), "div");
    let second = batch.create_element_ns(document, Namespace::html(), "span");
    let keeper = batch.create_element_ns(document, Namespace::html(), "aside");
    batch.insert_before(&first, &root, None);
    batch.insert_before(&second, &root, None);
    batch.set_attribute(&first, "style", "width: 3px");
    batch.set_attribute(&second, "style", "width: 6px");
    batch.set_attribute(&keeper, "style", "width: 9px");
    let commit = dom.apply(batch, &cancel()).unwrap();
    let first = commit.created(&first).unwrap().unwrap();
    let second = commit.created(&second).unwrap().unwrap();
    let keeper = commit.created(&keeper).unwrap().unwrap();
    for element in [first, second, keeper] {
        bind(&mut dom, document, element);
    }
    Fixture {
        dom,
        document,
        root: commit.created(&root).unwrap().unwrap(),
        first,
        second,
        keeper,
    }
}
fn batch_context(dom: &Dom, batch: &DomBatch) -> DomInlineStyleBatchContext {
    DomInlineStyleBatchContext::new(batch, dom.snapshot().stamp(), Vec::new()).unwrap()
}
fn source_context(dom: &Dom) -> DomInlineStyleSourceContext {
    DomInlineStyleSourceContext::new(dom.snapshot().stamp(), Vec::new()).unwrap()
}
fn checked_profile(dom: &Dom, element: NodeId, document: DocumentId) -> DomInlineStyleProfile {
    DomInlineStyleProfile::new(
        element,
        document,
        parser(dom, document),
        version(),
        support(),
    )
    .unwrap()
}
fn subtree_profiles(f: &Fixture) -> Vec<DomInlineStyleProfile> {
    // These qualify actual live child-detachment context occurrences. Retirement
    // itself has no parser profile and must not reconstruct destroyed-node facts.
    [f.first, f.second]
        .into_iter()
        .map(|element| checked_profile(&f.dom, element, f.document))
        .collect()
}
fn subtree_batch_context(f: &Fixture, batch: &DomBatch) -> DomInlineStyleBatchContext {
    DomInlineStyleBatchContext::new(batch, f.dom.snapshot().stamp(), subtree_profiles(f)).unwrap()
}
fn subtree_source_context(f: &Fixture) -> DomInlineStyleSourceContext {
    DomInlineStyleSourceContext::new(f.dom.snapshot().stamp(), subtree_profiles(f)).unwrap()
}
fn assert_unchanged(dom: &Dom, old: &DomInlineStyleCapture) {
    let now = capture(dom, old.element());
    assert_eq!(now.dom().stamp(), old.dom().stamp());
    assert_eq!(now.properties().handle(), old.properties().handle());
    assert_eq!(
        now.properties().snapshot().revision(),
        old.properties().snapshot().revision()
    );
    assert_eq!(
        now.properties().snapshot().inputs(),
        old.properties().snapshot().inputs()
    );
    assert_eq!(width(&now), width(old));
}
fn assert_retirement_receipt(
    receipt: &CssomCommit,
    before: &CssomRevision,
    after: &CssomRevision,
    retired: &[&CssStyleProperties],
) {
    assert!(receipt.owner_effects().is_empty());
    let CssomPublication::Changed(summary) = receipt.publication() else {
        panic!("removing current bound blocks must publish actual producer changes");
    };
    assert_eq!(summary.from(), before);
    assert_eq!(summary.to(), after);
    assert!(summary.categories().contains(&CssomChange::Owner));
    assert!(summary.categories().contains(&CssomChange::Declarations));
    for handle in retired {
        assert!(
            summary
                .affected()
                .contains(&CssomObjectId::Block(handle.id().clone()))
        );
        assert!(
            summary
                .before_inputs()
                .inline_declarations
                .iter()
                .any(|(id, _)| id == handle.id())
        );
        assert!(
            !summary
                .after_inputs()
                .inline_declarations
                .iter()
                .any(|(id, _)| id == handle.id())
        );
    }
}
fn assert_retired(dom: &Dom, keeper: NodeId, retired: &[&DomInlineStyleCapture]) {
    let current = capture(dom, keeper);
    for old in retired {
        assert!(matches!(
            dom.snapshot().node(old.element()),
            Err(DomError::StaleNode)
        ));
        assert!(matches!(
            dom.capture_inline_style(old.element()),
            Err(DomInlineStyleError::Dom(DomError::StaleNode))
        ));
        assert!(matches!(
            old.properties()
                .handle()
                .capture(current.properties().snapshot()),
            Err(CssomError::MissingObject)
        ));
        assert!(matches!(
            current
                .properties()
                .snapshot()
                .block(old.properties().handle().id()),
            Err(CssomError::MissingObject)
        ));
        // Old observations retain both the original node and producer record.
        assert!(old.dom().node(old.element()).is_ok());
        assert!(!width(old).is_empty());
    }
    assert_eq!(width(&current), "9px");
}

#[test]
fn ordinary_bound_subtree_destruction_returns_one_retirement_commit_and_preserves_history() {
    let mut f = fixture(Default::default());
    let first = capture(&f.dom, f.first);
    let second = capture(&f.dom, f.second);
    let keeper = capture(&f.dom, f.keeper);
    let before = f.dom.snapshot().stamp();
    let mut batch = DomBatch::new();
    batch.destroy(f.root);
    let context = subtree_batch_context(&f, &batch);
    let result = f
        .dom
        .apply_with_inline_styles(batch, context, requests(), no_decisions, &cancel())
        .unwrap();
    assert_eq!(result.dom().changes().from(), before);
    for element in [f.root, f.first, f.second] {
        assert!(
            result
                .dom()
                .changes()
                .changes()
                .contains(&DomChange::Destroyed(element))
        );
    }
    let current_keeper = capture(&f.dom, f.keeper);
    assert_retirement_receipt(
        result.inline().expect("actual retirement producer receipt"),
        keeper.properties().snapshot().revision(),
        current_keeper.properties().snapshot().revision(),
        &[first.properties().handle(), second.properties().handle()],
    );
    assert_retired(&f.dom, f.keeper, &[&first, &second]);
    assert_eq!(width(&first), "3px");
    assert_eq!(width(&second), "6px");
    assert_eq!(
        capture(&f.dom, f.keeper).properties().handle(),
        keeper.properties().handle()
    );
}

#[test]
fn source_bound_subtree_destruction_returns_original_paired_retirement_once() {
    let mut f = fixture(Default::default());
    let first = capture(&f.dom, f.first);
    let second = capture(&f.dom, f.second);
    let keeper = capture(&f.dom, f.keeper);
    let before = f.dom.snapshot().stamp();
    let context = subtree_source_context(&f);
    let mut lease = f
        .dom
        .begin_source_edit_with_inline_styles(
            DomSourceEdit::destroy(f.root),
            context,
            source_limits(8192),
            requests(),
            &cancel(),
        )
        .unwrap();
    let operation = lease.operation();
    let (progress, pair) = lease
        .advance_with_inline_styles(no_decisions, &cancel())
        .unwrap()
        .into_parts();
    assert!(
        matches!(progress, DomSourceProgress::Complete(ref completion) if completion.operation() == operation)
    );
    let pair = pair.expect("actual paired producer retirement receipt");
    assert_eq!(pair.dom().from(), before);
    assert_eq!(pair.dom().to(), lease.snapshot().stamp());
    for element in [f.root, f.first, f.second] {
        assert!(
            pair.dom()
                .changes()
                .contains(&DomChange::Destroyed(element))
        );
    }
    drop(lease);
    let current_keeper = capture(&f.dom, f.keeper);
    assert_retirement_receipt(
        pair.inline(),
        keeper.properties().snapshot().revision(),
        current_keeper.properties().snapshot().revision(),
        &[first.properties().handle(), second.properties().handle()],
    );
    assert!(f.dom.pending_source_edit().is_none());
    assert_retired(&f.dom, f.keeper, &[&first, &second]);
}

#[test]
fn later_invalid_dom_command_preserves_destroyed_candidates_and_both_owner_revisions() {
    let mut f = fixture(Default::default());
    let first = capture(&f.dom, f.first);
    let second = capture(&f.dom, f.second);
    let keeper = capture(&f.dom, f.keeper);
    let mut batch = DomBatch::new();
    batch.destroy(f.root);
    batch.insert_before(f.keeper, f.keeper, None);
    let context = subtree_batch_context(&f, &batch);
    let result =
        f.dom
            .apply_with_inline_styles(batch, context, requests(), no_decisions, &cancel());
    assert!(
        matches!(
            &result,
            Err(DomInlineStyleError::Dom(DomError::HierarchyRequest))
        ),
        "actual refusal before the later invalid command: {:?}",
        result.as_ref().err()
    );
    for old in [&first, &second, &keeper] {
        assert_unchanged(&f.dom, old);
    }
}

#[test]
fn actual_attribute_admission_before_destruction_is_preserved_and_can_refuse_both_owners() {
    for refuse in [false, true] {
        let mut f = fixture(Default::default());
        let first = capture(&f.dom, f.first);
        let second = capture(&f.dom, f.second);
        let keeper = capture(&f.dom, f.keeper);
        let mut batch = DomBatch::new();
        batch.set_attribute(f.first, "style", "width: 11px");
        batch.destroy(f.root);
        // This row qualifies the real earlier attribute and child-detachment
        // occurrences while its Element and Document are present. The second
        // child's actual detachment has its own checked profile below.
        let profile = DomInlineStyleProfile::new(
            f.first,
            f.document,
            parser(&f.dom, f.document),
            CssomInputVersion {
                role: CssomInputRole::Support,
                identity: "retirement-width-whole-declaration".into(),
                revision: 1,
            },
            CssomDeclarationSupport::try_new(vec![(
                CssPropertyGrammar::from_name("width").unwrap(),
                CssomUsabilityRequirement::WholeDeclarationDecision,
            )])
            .unwrap(),
        )
        .unwrap();
        let context = DomInlineStyleBatchContext::new(
            &batch,
            f.dom.snapshot().stamp(),
            vec![profile, checked_profile(&f.dom, f.second, f.document)],
        )
        .unwrap();
        let mut calls = 0;
        let result = f.dom.apply_with_inline_styles(
            batch,
            context,
            requests(),
            |candidates, context| {
                calls += 1;
                assert_eq!(candidates.len(), 1, "the actual earlier width admission");
                if refuse {
                    return Err(CssomError::InvalidInput(
                        "retirement declaration admission refused",
                    ));
                }
                let support = context
                    .inputs
                    .iter()
                    .find(|input| input.version.role == CssomInputRole::Support)
                    .unwrap()
                    .version
                    .clone();
                Ok(CssomAdmission::new(
                    candidates
                        .iter()
                        .map(|candidate| {
                            candidate
                                .decide(support.clone(), CssomDeclarationUsability::Usable)
                                .unwrap()
                        })
                        .collect(),
                    context
                        .inputs
                        .iter()
                        .map(|input| input.version.clone())
                        .collect(),
                ))
            },
            &cancel(),
        );
        if refuse {
            assert!(matches!(
                result,
                Err(DomInlineStyleError::Cssom(CssomError::InvalidInput(
                    "retirement declaration admission refused"
                )))
            ));
            for old in [&first, &second, &keeper] {
                assert_unchanged(&f.dom, old);
            }
        } else {
            let result = result.unwrap();
            let current_keeper = capture(&f.dom, f.keeper);
            assert_retirement_receipt(
                result.inline().unwrap(),
                keeper.properties().snapshot().revision(),
                current_keeper.properties().snapshot().revision(),
                &[first.properties().handle(), second.properties().handle()],
            );
            assert_retired(&f.dom, f.keeper, &[&first, &second]);
        }
        assert_eq!(
            calls, 1,
            "retirement must not erase a reached declaration operation"
        );
        assert_eq!(width(&first), "3px");
    }
}

#[test]
fn real_cssom_revision_exhaustion_refuses_ordinary_and_source_retirement_atomically() {
    for source in [false, true] {
        // Three actual first-binding commits consume precisely three revisions.
        // Retirement then reaches the real producer commit's revision guard.
        let mut f = fixture(CssomLimits {
            max_revision: 3,
            ..Default::default()
        });
        let first = capture(&f.dom, f.first);
        let second = capture(&f.dom, f.second);
        let keeper = capture(&f.dom, f.keeper);
        if source {
            let context = subtree_source_context(&f);
            let mut lease = f
                .dom
                .begin_source_edit_with_inline_styles(
                    DomSourceEdit::destroy(f.root),
                    context,
                    source_limits(8192),
                    requests(),
                    &cancel(),
                )
                .unwrap();
            let operation = lease.operation();
            let result = lease.advance_with_inline_styles(no_decisions, &cancel());
            assert!(
                matches!(
                    &result,
                    Err(DomInlineStyleError::Cssom(CssomError::RevisionExhausted))
                ),
                "actual Source refusal before producer commit: {:?}",
                result.as_ref().err()
            );
            assert_eq!(lease.operation(), operation);
            assert_eq!(lease.snapshot().stamp(), first.dom().stamp());
            assert!(!lease.status().has_published());
        } else {
            let mut batch = DomBatch::new();
            batch.destroy(f.root);
            let context = subtree_batch_context(&f, &batch);
            let result =
                f.dom
                    .apply_with_inline_styles(batch, context, requests(), no_decisions, &cancel());
            assert!(
                matches!(
                    &result,
                    Err(DomInlineStyleError::Cssom(CssomError::RevisionExhausted))
                ),
                "actual ordinary refusal before producer commit: {:?}",
                result.as_ref().err()
            );
        }
        for old in [&first, &second, &keeper] {
            assert_unchanged(&f.dom, old);
        }
    }
}

#[test]
fn source_output_refusal_keeps_bound_subtree_and_producer_records_unchanged() {
    let mut f = fixture(Default::default());
    let first = capture(&f.dom, f.first);
    let second = capture(&f.dom, f.second);
    let keeper = capture(&f.dom, f.keeper);
    let context = subtree_source_context(&f);
    let mut lease = f
        .dom
        .begin_source_edit_with_inline_styles(
            DomSourceEdit::destroy(f.root),
            context,
            source_limits(0),
            requests(),
            &cancel(),
        )
        .unwrap();
    let operation = lease.operation();
    let result = lease.advance_with_inline_styles(no_decisions, &cancel());
    assert!(
        matches!(
            &result,
            Err(DomInlineStyleError::Dom(DomError::SourceOutputLimit))
        ),
        "actual Source refusal before output admission: {:?}",
        result.as_ref().err()
    );
    assert_eq!(lease.operation(), operation);
    assert_eq!(lease.snapshot().stamp(), first.dom().stamp());
    assert!(!lease.status().has_published());
    drop(lease);
    for old in [&first, &second, &keeper] {
        assert_unchanged(&f.dom, old);
    }
}

#[test]
fn repeated_retirement_reuses_live_capacity_and_never_rebinds_stale_nodes_or_blocks() {
    // One producer root sheet-list plus keeper and target are three objects.
    let (mut dom, document) = owner(
        2,
        3,
        CssomLimits {
            max_objects: 3,
            ..Default::default()
        },
    );
    let mut initial = DomBatch::new();
    let keeper = initial.create_element_ns(document, Namespace::html(), "aside");
    initial.set_attribute(&keeper, "style", "width: 9px");
    let keeper = dom
        .apply(initial, &cancel())
        .unwrap()
        .created(&keeper)
        .unwrap()
        .unwrap();
    bind(&mut dom, document, keeper);
    let mut historical: Vec<DomInlineStyleCapture> = Vec::new();
    for cycle in 0..8 {
        let mut batch = DomBatch::new();
        let element = batch.create_element_ns(document, Namespace::html(), "div");
        batch.set_attribute(&element, "style", "width: 3px");
        let element = dom
            .apply(batch, &cancel())
            .unwrap()
            .created(&element)
            .unwrap()
            .unwrap();
        let handle = bind(&mut dom, document, element);
        let old = capture(&dom, element);
        assert_eq!(
            old.properties()
                .snapshot()
                .inputs()
                .inline_declarations
                .len(),
            2
        );
        for prior in &historical {
            assert_ne!(element, prior.element());
            assert_ne!(handle.id(), prior.properties().handle().id());
            let input = inputs(&dom, document);
            assert!(matches!(
                dom.inline_style(prior.element(), &input, requests(), no_decisions, &cancel()),
                Err(DomInlineStyleError::Dom(DomError::StaleNode))
            ));
            assert!(matches!(
                prior
                    .properties()
                    .handle()
                    .capture(old.properties().snapshot()),
                Err(CssomError::MissingObject)
            ));
            assert_eq!(width(prior), "3px");
        }
        if cycle % 2 == 0 {
            let mut batch = DomBatch::new();
            batch.destroy(element);
            let context = batch_context(&dom, &batch);
            let result = dom
                .apply_with_inline_styles(batch, context, requests(), no_decisions, &cancel())
                .unwrap();
            let current_keeper = capture(&dom, keeper);
            assert_retirement_receipt(
                result.inline().unwrap(),
                old.properties().snapshot().revision(),
                current_keeper.properties().snapshot().revision(),
                &[&handle],
            );
        } else {
            let context = source_context(&dom);
            let mut lease = dom
                .begin_source_edit_with_inline_styles(
                    DomSourceEdit::destroy(element),
                    context,
                    source_limits(8192),
                    requests(),
                    &cancel(),
                )
                .unwrap();
            let result = lease
                .advance_with_inline_styles(no_decisions, &cancel())
                .unwrap();
            assert!(matches!(result.source(), DomSourceProgress::Complete(_)));
            drop(lease);
            let current_keeper = capture(&dom, keeper);
            assert_retirement_receipt(
                result.inline().unwrap().inline(),
                old.properties().snapshot().revision(),
                current_keeper.properties().snapshot().revision(),
                &[&handle],
            );
        }
        assert_retired(&dom, keeper, &[&old]);
        assert_eq!(
            capture(&dom, keeper)
                .properties()
                .snapshot()
                .inputs()
                .inline_declarations
                .len(),
            1
        );
        historical.push(old);
    }
}

#[test]
fn pending_feedback_still_refuses_unrelated_bound_retirement_and_preserves_original_effect() {
    for source in [false, true] {
        let mut f = fixture(Default::default());
        let document = f
            .dom
            .snapshot()
            .node(f.keeper)
            .unwrap()
            .owner_document()
            .unwrap();
        let before = f.dom.snapshot().stamp();
        let input = inputs(&f.dom, document);
        let pending = f
            .dom
            .set_inline_style_css_text(
                f.keeper,
                &input,
                "--pending: yes",
                DomInlineStyleMutationLimits {
                    declaration: Default::default(),
                    feedback: DomInlineStyleFeedbackLimits::new(64_000_000, 1_000_000, 0, 64),
                },
                |_| panic!("custom property requires no resolver"),
                &cancel(),
            )
            .unwrap();
        let effect = match pending {
            DomInlineStyleMutation::Incomplete { operation, reason } => {
                assert!(matches!(
                    reason,
                    DomInlineStyleError::Dom(DomError::SourceWorkLimit { limit: 0 })
                ));
                assert_eq!(
                    operation.owner_effects()[0].status(),
                    CssomOwnerEffectStatus::Pending
                );
                operation.owner_effects()[0].id().clone()
            }
            _ => panic!("zero feedback work must park the actual committed effect"),
        };
        let first_before = capture(&f.dom, f.first);
        let second_before = capture(&f.dom, f.second);
        if source {
            let context = subtree_source_context(&f);
            let result = match f.dom.begin_source_edit_with_inline_styles(
                DomSourceEdit::destroy(f.root),
                context,
                source_limits(8192),
                requests(),
                &cancel(),
            ) {
                Ok(mut lease) => lease
                    .advance_with_inline_styles(no_decisions, &cancel())
                    .map(|_| ()),
                Err(reason) => Err(reason),
            };
            assert!(matches!(result,
                Err(DomInlineStyleError::OperationPending { element })
                | Err(DomInlineStyleError::Dom(DomError::InlineStyleOperationPending { element }))
                if element == f.keeper
            ));
        } else {
            let mut batch = DomBatch::new();
            batch.destroy(f.root);
            let context = subtree_batch_context(&f, &batch);
            let result =
                f.dom
                    .apply_with_inline_styles(batch, context, requests(), no_decisions, &cancel());
            assert!(matches!(result,
                Err(DomInlineStyleError::OperationPending { element })
                | Err(DomInlineStyleError::Dom(DomError::InlineStyleOperationPending { element }))
                if element == f.keeper
            ));
        }
        assert_eq!(f.dom.snapshot().stamp(), before);
        assert!(f.dom.snapshot().node(f.root).is_ok());
        assert!(f.dom.snapshot().node(f.first).is_ok());
        assert!(f.dom.snapshot().node(f.second).is_ok());
        assert_unchanged(&f.dom, &first_before);
        assert_unchanged(&f.dom, &second_before);
        let operation = f.dom.resume_inline_style_feedback().unwrap();
        assert_eq!(operation.owner_effects()[0].id(), &effect);
        assert_eq!(
            operation.owner_effects()[0].status(),
            CssomOwnerEffectStatus::Pending
        );
    }
}

#[test]
fn first_middle_last_and_multiple_retirements_preserve_each_surviving_live_object() {
    for source in [false, true] {
        for retired in [&[0usize][..], &[3][..], &[5][..], &[0, 1, 3][..]] {
            let (mut dom, document) = owner(
                6,
                32,
                CssomLimits {
                    max_objects: 7, // Root sheet-list plus six live bound objects.
                    ..Default::default()
                },
            );
            let mut batch = DomBatch::new();
            let root = batch.create_element_ns(document, Namespace::html(), "section");
            let tickets: Vec<_> = (0..6)
                .map(|index| {
                    let element = batch.create_element_ns(document, Namespace::html(), "div");
                    batch.set_attribute(&element, "style", format!("width: {}px", index + 1));
                    if retired.contains(&index) {
                        batch.insert_before(&element, &root, None);
                    }
                    element
                })
                .collect();
            let commit = dom.apply(batch, &cancel()).unwrap();
            let root = commit.created(&root).unwrap().unwrap();
            let elements: Vec<_> = tickets
                .iter()
                .map(|ticket| commit.created(ticket).unwrap().unwrap())
                .collect();
            for &element in &elements {
                bind(&mut dom, document, element);
            }
            let old: Vec<_> = elements
                .iter()
                .map(|&element| capture(&dom, element))
                .collect();
            let profiles: Vec<_> = retired
                .iter()
                .map(|&index| checked_profile(&dom, elements[index], document))
                .collect();
            let receipt = if source {
                let context =
                    DomInlineStyleSourceContext::new(dom.snapshot().stamp(), profiles).unwrap();
                let mut lease = dom
                    .begin_source_edit_with_inline_styles(
                        DomSourceEdit::destroy(root),
                        context,
                        source_limits(8192),
                        requests(),
                        &cancel(),
                    )
                    .unwrap();
                let result = lease
                    .advance_with_inline_styles(no_decisions, &cancel())
                    .unwrap();
                assert!(matches!(result.source(), DomSourceProgress::Complete(_)));
                let (_, pair) = result.into_parts();
                drop(lease);
                pair.unwrap().into_parts().1
            } else {
                let mut batch = DomBatch::new();
                batch.destroy(root);
                let context =
                    DomInlineStyleBatchContext::new(&batch, dom.snapshot().stamp(), profiles)
                        .unwrap();
                dom.apply_with_inline_styles(batch, context, requests(), no_decisions, &cancel())
                    .unwrap()
                    .into_parts()
                    .1
                    .unwrap()
            };
            let keeper = (0..elements.len())
                .find(|index| !retired.contains(index))
                .unwrap();
            let current = capture(&dom, elements[keeper]);
            let retired_handles: Vec<_> = retired
                .iter()
                .map(|&index| old[index].properties().handle())
                .collect();
            assert_retirement_receipt(
                &receipt,
                old[0].properties().snapshot().revision(),
                current.properties().snapshot().revision(),
                &retired_handles,
            );
            for (index, &element) in elements.iter().enumerate() {
                let expected = format!("{}px", index + 1);
                assert_eq!(width(&old[index]), expected);
                if retired.contains(&index) {
                    assert!(matches!(
                        dom.snapshot().node(element),
                        Err(DomError::StaleNode)
                    ));
                    assert!(matches!(
                        old[index]
                            .properties()
                            .handle()
                            .capture(current.properties().snapshot()),
                        Err(CssomError::MissingObject)
                    ));
                    continue;
                }
                let before = capture(&dom, element);
                assert_eq!(
                    before.properties().handle(),
                    old[index].properties().handle()
                );
                assert_eq!(width(&before), expected);
                let context = DomInlineStyleSourceContext::new(
                    dom.snapshot().stamp(),
                    vec![checked_profile(&dom, element, document)],
                )
                .unwrap();
                let mut lease = dom
                    .begin_source_edit_with_inline_styles(
                        DomSourceEdit::set_attribute(element, "style", "width: 19px"),
                        context,
                        source_limits(8192),
                        requests(),
                        &cancel(),
                    )
                    .unwrap();
                let result = lease
                    .advance_with_inline_styles(no_decisions, &cancel())
                    .unwrap();
                assert!(matches!(result.source(), DomSourceProgress::Complete(_)));
                assert!(result.inline().is_some());
                drop(lease);
                let after = capture(&dom, element);
                assert_eq!(
                    after.properties().handle(),
                    old[index].properties().handle()
                );
                assert_eq!(width(&after), "19px");
                assert_eq!(width(&old[index]), expected);
            }
        }
    }
}

#[test]
fn owned_shadow_and_template_contents_retire_with_their_host_and_preserve_captures() {
    for source in [false, true] {
        let (mut dom, document) = owner(3, 64, Default::default());
        let mut batch = DomBatch::new();
        let host = batch.create_element_ns(document, Namespace::html(), "div");
        let shadow = batch.attach_shadow(&host, DomShadowRootInit::new(HtmlShadowRootMode::Closed));
        let template = batch.create_element_ns(document, Namespace::html(), "template");
        batch.insert_before(&template, &host, None);
        let keeper = batch.create_element_ns(document, Namespace::html(), "aside");
        batch.set_attribute(&keeper, "style", "width: 9px");
        let commit = dom.apply(batch, &cancel()).unwrap();
        let host = commit.created(&host).unwrap().unwrap();
        let shadow = commit.created(&shadow).unwrap().unwrap();
        let template = commit.created(&template).unwrap().unwrap();
        let keeper = commit.created(&keeper).unwrap().unwrap();
        let snapshot = dom.snapshot();
        let contents = snapshot
            .node(template)
            .unwrap()
            .as_template()
            .unwrap()
            .contents()
            .unwrap();
        let inert_document = snapshot.node(contents).unwrap().owner_document().unwrap();
        let mut batch = DomBatch::new();
        let shadow_child = batch.create_element_ns(document, Namespace::html(), "span");
        batch.insert_before(&shadow_child, shadow, None);
        batch.set_attribute(&shadow_child, "style", "width: 3px");
        let contents_child = batch.create_element_ns(inert_document, Namespace::html(), "div");
        batch.insert_before(&contents_child, contents, None);
        batch.set_attribute(&contents_child, "style", "width: 6px");
        let commit = dom.apply(batch, &cancel()).unwrap();
        let shadow_child = commit.created(&shadow_child).unwrap().unwrap();
        let contents_child = commit.created(&contents_child).unwrap().unwrap();
        bind(&mut dom, document, shadow_child);
        bind(&mut dom, inert_document, contents_child);
        let keeper_handle = bind(&mut dom, document, keeper);
        let old_shadow = capture(&dom, shadow_child);
        let old_contents = capture(&dom, contents_child);
        let old_keeper = capture(&dom, keeper);
        assert_eq!(old_shadow.tree_scope().root(), shadow);
        assert_eq!(old_shadow.tree_scope().host(), Some(host));
        assert_eq!(old_contents.tree_scope().root(), contents);
        assert_eq!(
            old_contents.dom().node(contents).unwrap().fragment_host(),
            Some(template)
        );
        let profiles = vec![
            checked_profile(&dom, shadow_child, document),
            checked_profile(&dom, contents_child, inert_document),
        ];
        let receipt = if source {
            let context =
                DomInlineStyleSourceContext::new(dom.snapshot().stamp(), profiles).unwrap();
            let mut lease = dom
                .begin_source_edit_with_inline_styles(
                    DomSourceEdit::destroy(host),
                    context,
                    source_limits(8192),
                    requests(),
                    &cancel(),
                )
                .unwrap();
            let result = lease
                .advance_with_inline_styles(no_decisions, &cancel())
                .unwrap();
            assert!(matches!(result.source(), DomSourceProgress::Complete(_)));
            let (_, pair) = result.into_parts();
            drop(lease);
            pair.unwrap().into_parts().1
        } else {
            let mut batch = DomBatch::new();
            batch.destroy(host);
            let context =
                DomInlineStyleBatchContext::new(&batch, dom.snapshot().stamp(), profiles).unwrap();
            dom.apply_with_inline_styles(batch, context, requests(), no_decisions, &cancel())
                .unwrap()
                .into_parts()
                .1
                .unwrap()
        };
        let current = capture(&dom, keeper);
        assert_retirement_receipt(
            &receipt,
            old_keeper.properties().snapshot().revision(),
            current.properties().snapshot().revision(),
            &[
                old_shadow.properties().handle(),
                old_contents.properties().handle(),
            ],
        );
        for element in [
            host,
            shadow,
            template,
            contents,
            shadow_child,
            contents_child,
        ] {
            assert!(matches!(
                dom.snapshot().node(element),
                Err(DomError::StaleNode)
            ));
        }
        assert_retired(&dom, keeper, &[&old_shadow, &old_contents]);
        assert_eq!(width(&old_shadow), "3px");
        assert_eq!(width(&old_contents), "6px");
        let context = DomInlineStyleSourceContext::new(
            dom.snapshot().stamp(),
            vec![checked_profile(&dom, keeper, document)],
        )
        .unwrap();
        let mut lease = dom
            .begin_source_edit_with_inline_styles(
                DomSourceEdit::set_attribute(keeper, "style", "width: 17px"),
                context,
                source_limits(8192),
                requests(),
                &cancel(),
            )
            .unwrap();
        let result = lease
            .advance_with_inline_styles(no_decisions, &cancel())
            .unwrap();
        assert!(matches!(result.source(), DomSourceProgress::Complete(_)));
        assert!(result.inline().is_some());
        drop(lease);
        let current = capture(&dom, keeper);
        assert_eq!(current.properties().handle(), &keeper_handle);
        assert_eq!(width(&current), "17px");
        assert_eq!(width(&old_keeper), "9px");
    }
}
