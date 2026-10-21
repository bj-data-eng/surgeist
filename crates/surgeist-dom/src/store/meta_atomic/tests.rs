//! Owning functional evidence with actual DOM, CSSOM roots and inline producer.
use super::*;
use crate::*;
use surgeist_css::*;
use surgeist_cssom::*;

fn flag() -> AtomicBool {
    AtomicBool::new(false)
}
fn limits() -> DomMetaAtomicLimits {
    DomMetaAtomicLimits::new(16_000_000, 1_000_000, 1_000_000, 128).unwrap()
}
fn request() -> DomInlineStyleRequestLimits {
    DomInlineStyleRequestLimits {
        input: Default::default(),
        declaration: Default::default(),
    }
}
fn support() -> CssomDeclarationSupport {
    CssomDeclarationSupport::try_new(vec![(
        CssPropertyGrammar::from_name("width").unwrap(),
        CssomUsabilityRequirement::WholeDeclarationDecision,
    )])
    .unwrap()
}
fn version() -> CssomInputVersion {
    CssomInputVersion {
        role: CssomInputRole::Support,
        identity: "native-meta-width".into(),
        revision: 1,
    }
}
fn profile(element: NodeId, document: DocumentId) -> DomInlineStyleProfile {
    DomInlineStyleProfile::new(
        element,
        document,
        CssParserContext::default(),
        version(),
        support(),
    )
    .unwrap()
}
fn admission(
    candidates: &[CssomDeclarationCandidate],
    context: &CssomContext,
) -> Result<CssomAdmission, CssomError> {
    Ok(CssomAdmission::new(
        candidates
            .iter()
            .map(|c| {
                c.decide(version(), CssomDeclarationUsability::Usable)
                    .unwrap()
            })
            .collect(),
        context.inputs.iter().map(|i| i.version.clone()).collect(),
    ))
}
struct Fixture {
    dom: Dom,
    docs: [DocumentId; 2],
    heads: [NodeId; 2],
    owners: Vec<DomStyleCssomRoot>,
    sheets: Vec<CssomSheetId>,
    element: NodeId,
}
fn fixture(second_revision: u64) -> Fixture {
    fixture_binding(second_revision, true)
}
fn fixture_binding(second_revision: u64, bound: bool) -> Fixture {
    let policy = DomLimits::new(
        256,
        100_000,
        512,
        128,
        128,
        32,
        4_000_000,
        DomNameLimits::new(16_777_216, 16384, 4_000_000, 1_048_576).unwrap(),
    )
    .unwrap()
    .with_meta_processing_work(1_000_000)
    .with_meta_theme_color_scratch_bytes(262144)
    .unwrap();
    let mut dom = Dom::new(policy).unwrap();
    let mut batch = DomBatch::new();
    let a = batch.create_document();
    let b = batch.create_document();
    let ah = batch.create_element_ns(&a, Namespace::html(), "head");
    let bh = batch.create_element_ns(&b, Namespace::html(), "head");
    let e = batch.create_element_ns(&a, Namespace::html(), "div");
    batch.insert_before(&ah, &a, None);
    batch.insert_before(&bh, &b, None);
    batch.insert_before(&e, &ah, None);
    let commit = dom.apply(batch, &flag()).unwrap();
    let docs = [commit.document(&a).unwrap(), commit.document(&b).unwrap()];
    let heads = [
        commit.created(&ah).unwrap().unwrap(),
        commit.created(&bh).unwrap().unwrap(),
    ];
    let element = commit.created(&e).unwrap().unwrap();
    let mut owners = Vec::new();
    let mut sheets = Vec::new();
    for (index, document) in docs.iter().enumerate() {
        let mut store = CssomStore::new(
            CssomLimits {
                max_revision: if index == 1 {
                    second_revision
                } else {
                    u64::MAX
                },
                ..Default::default()
            },
            Default::default(),
        )
        .unwrap();
        let mut batch = store
            .batch(CssomEditGuard::from_snapshot(&store.snapshot()))
            .unwrap();
        let ticket = batch
            .create_parsed_sheet(
                "a {}",
                CssomSheetInputs::constructed(
                    CssomInputVersion {
                        role: CssomInputRole::Document,
                        identity: format!("document-{index}"),
                        revision: 1,
                    },
                    format!("document-{index}"),
                    None,
                ),
            )
            .unwrap();
        let commit = store.commit(batch).unwrap();
        sheets.push(commit.sheet(&ticket).unwrap().clone());
        let owner = DomStyleCssomRoot::new(document.node(), store);
        // Actual canonical custody installation through its owning private bank,
        // not a fabricated receipt or stand-in CSSOM implementation.
        if bound {
            dom.style_cssom_roots = dom
                .style_cssom_roots
                .prepare_insert(&owner, *document, |_| Ok(()), || Ok(()))
                .unwrap();
        }
        owners.push(owner);
    }
    dom.configure_inline_styles(
        DomInlineStyleLimits::new(CssomLimits::default(), 8, 64_000_000, 8_000_000, 4_000_000)
            .unwrap(),
        &flag(),
    )
    .unwrap();
    let inputs = DomInlineStyleInputs::new(
        dom.snapshot().document(docs[0]).unwrap().stamp(),
        CssParserContext::default(),
        version(),
        support(),
    )
    .unwrap();
    dom.inline_style(
        element,
        &inputs,
        request(),
        |_, _| panic!("empty initial attribute"),
        &flag(),
    )
    .unwrap();
    Fixture {
        dom,
        docs,
        heads,
        owners,
        sheets,
        element,
    }
}
fn insert(
    batch: &mut DomBatch,
    document: DocumentId,
    parent: NodeId,
    name: DocumentString,
) -> CreatedNode {
    let meta = batch.create_element_ns(document, Namespace::html(), "meta");
    batch.set_attribute(&meta, "http-equiv", "DeFaUlT-StYlE");
    batch.set_attribute(&meta, "content", name);
    batch.insert_before(&meta, parent, None);
    meta
}
#[expect(
    clippy::result_large_err,
    reason = "return the real root-qualified producer refusal allocation-free for precise failure assertions"
)]
fn prepare<'d, 'c>(
    dom: &'d mut Dom,
    batch: DomBatch,
    caps: &'c [&'c DomStyleCssomRoot],
    inline: Option<DomMetaInlineBatch<'_>>,
    limit: DomMetaAtomicLimits,
) -> Result<DomMetaOrdinaryPreparation<'d, 'c>, DomMetaAtomicError> {
    dom.prepare_meta_batch(
        batch,
        caps,
        inline,
        limit,
        DomMetaRuntimePrefix::new(DomMetaRuntimeUsage::new(0, 0, 0).unwrap(), 0, limit).unwrap(),
        &flag(),
    )
}
fn snapshots(f: &Fixture) -> (DomStamp, Vec<CssomSnapshot>, CssomSnapshot) {
    (
        f.dom.snapshot().stamp(),
        f.owners.iter().map(|o| o.snapshot().unwrap()).collect(),
        f.dom
            .inline_styles
            .as_ref()
            .unwrap()
            .store
            .as_ref()
            .unwrap()
            .snapshot(),
    )
}
fn unchanged(f: &Fixture, before: &(DomStamp, Vec<CssomSnapshot>, CssomSnapshot)) {
    assert_eq!(f.dom.snapshot().stamp(), before.0);
    for (owner, prior) in f.owners.iter().zip(&before.1) {
        assert_eq!(owner.snapshot().unwrap().revision(), prior.revision());
        assert!(
            !owner
                .snapshot()
                .unwrap()
                .sheet(&prior.sheets()[0])
                .unwrap()
                .inputs()
                .disabled
        );
    }
    assert_eq!(
        f.dom
            .inline_styles
            .as_ref()
            .unwrap()
            .store
            .as_ref()
            .unwrap()
            .snapshot()
            .revision(),
        before.2.revision()
    );
    assert!(
        f.dom
            .snapshot()
            .node(f.element)
            .unwrap()
            .as_element()
            .unwrap()
            .attribute(&"style".into())
            .is_none()
    );
}
#[test]
fn two_document_and_inline_publication_returns_genuine_outputs_and_retains_locks() {
    let mut f = fixture(u64::MAX);
    let before = snapshots(&f);
    let mut batch = DomBatch::new();
    let raw = DocumentString::from_utf16(vec![66, 0xD800, 13, 10]);
    insert(&mut batch, f.docs[0], f.heads[0], raw.clone());
    insert(&mut batch, f.docs[1], f.heads[1], "C".into());
    batch.set_attribute(f.element, "style", "width: 3px");
    let context = DomInlineStyleBatchContext::new(
        &batch,
        f.dom.snapshot().stamp(),
        vec![profile(f.element, f.docs[0])],
    )
    .unwrap();
    let mut resolver = admission;
    let caps = f.owners.iter().collect::<Vec<_>>();
    let mut prepared = prepare(
        &mut f.dom,
        batch,
        &caps,
        Some(DomMetaInlineBatch::new(context, request(), &mut resolver)),
        limits(),
    )
    .unwrap();
    assert_eq!(prepared.occurrences().len(), 2);
    assert_eq!(prepared.occurrences()[0].content(), &raw);
    for i in 0..2 {
        prepared
            .set_disabled(f.docs[i].node(), &f.sheets[i], true, &flag())
            .unwrap();
    }
    let published = prepared.commit(&flag()).unwrap();
    for owner in &f.owners {
        assert!(matches!(
            owner.snapshot(),
            Err(DomStyleCssomRootError::Busy)
        ));
    }
    let (commit, retirement) = published.into_parts();
    assert_eq!(commit.roots().len(), 2);
    assert!(commit.inline().is_some());
    assert_eq!(commit.dom().changes().from(), before.0);
    assert_eq!(commit.dom().changes().to(), f.dom.snapshot().stamp());
    for (i, row) in commit.roots().iter().enumerate() {
        assert_eq!(row.root(), f.docs[i].node());
        let CssomPublication::Changed(summary) = row.cssom().publication() else {
            panic!("actual flag publication")
        };
        assert_eq!(summary.from(), before.1[i].revision());
        assert_eq!(summary.to(), f.owners[i].snapshot().unwrap().revision());
        assert!(
            f.owners[i]
                .snapshot()
                .unwrap()
                .sheet(&f.sheets[i])
                .unwrap()
                .inputs()
                .disabled
        );
    }
    assert!(commit.inline().unwrap().owner_effects().is_empty());
    assert_eq!(
        f.dom
            .capture_inline_style(f.element)
            .unwrap()
            .unwrap()
            .properties()
            .get_property_value("width", Default::default())
            .unwrap(),
        "3px"
    );
    drop(retirement);
}
#[test]
fn later_document_refusal_aborts_earlier_owner_inline_and_canonical_dom() {
    let mut f = fixture(1);
    let before = snapshots(&f);
    let mut batch = DomBatch::new();
    for i in 0..2 {
        insert(&mut batch, f.docs[i], f.heads[i], "B".into());
    }
    batch.set_attribute(f.element, "style", "width: 3px");
    let context = DomInlineStyleBatchContext::new(
        &batch,
        f.dom.snapshot().stamp(),
        vec![profile(f.element, f.docs[0])],
    )
    .unwrap();
    let mut resolver = admission;
    let caps = f.owners.iter().collect::<Vec<_>>();
    let mut prepared = prepare(
        &mut f.dom,
        batch,
        &caps,
        Some(DomMetaInlineBatch::new(context, request(), &mut resolver)),
        limits(),
    )
    .unwrap();
    for i in 0..2 {
        prepared
            .set_disabled(f.docs[i].node(), &f.sheets[i], true, &flag())
            .unwrap();
    }
    let error = match prepared.commit(&flag()) {
        Err(error) => error,
        Ok(_) => panic!("the actual second producer has exhausted its revision"),
    };
    assert!(matches!(
        &error,
        DomMetaAtomicError::Cssom {
            root,
            error: CssomError::RevisionExhausted,
        } if *root == f.docs[1].node()
    ));
    assert!(matches!(
        std::error::Error::source(&error).and_then(|source| source.downcast_ref::<CssomError>()),
        Some(CssomError::RevisionExhausted)
    ));
    unchanged(&f, &before);
}
#[test]
fn drop_cancel_and_output_refusal_publish_neither_document_nor_inline() {
    for mode in 0..3 {
        let mut f = fixture(u64::MAX);
        let before = snapshots(&f);
        let mut batch = DomBatch::new();
        for i in 0..2 {
            insert(&mut batch, f.docs[i], f.heads[i], "B".into());
        }
        batch.set_attribute(f.element, "style", "width: 3px");
        let context = DomInlineStyleBatchContext::new(
            &batch,
            f.dom.snapshot().stamp(),
            vec![profile(f.element, f.docs[0])],
        )
        .unwrap();
        let mut resolver = admission;
        let caps = f.owners.iter().collect::<Vec<_>>();
        let limit = if mode == 2 {
            DomMetaAtomicLimits::new(16_000_000, 1_000_000, 1_000_000, 0).unwrap()
        } else {
            limits()
        };
        let mut prepared = prepare(
            &mut f.dom,
            batch,
            &caps,
            Some(DomMetaInlineBatch::new(context, request(), &mut resolver)),
            limit,
        )
        .unwrap();
        for i in 0..2 {
            prepared
                .set_disabled(f.docs[i].node(), &f.sheets[i], true, &flag())
                .unwrap();
        }
        match mode {
            0 => drop(prepared),
            1 => assert!(matches!(
                prepared.commit(&AtomicBool::new(true)),
                Err(DomMetaAtomicError::Dom(DomError::Cancelled))
            )),
            _ => assert!(matches!(
                prepared.commit(&flag()),
                Err(DomMetaAtomicError::Dom(DomError::ResourceLimit {
                    resource: DomResource::MetaAtomicOutputs,
                    ..
                }))
            )),
        }
        unchanged(&f, &before);
    }
}

#[test]
fn ordered_destroyed_creations_keep_exact_subjects_and_restored_flags_are_unchanged() {
    let mut f = fixture(u64::MAX);
    let before = snapshots(&f);
    let mut batch = DomBatch::new();
    let first = insert(&mut batch, f.docs[0], f.heads[0], "B".into());
    let second = insert(&mut batch, f.docs[0], f.heads[0], "A".into());
    for node in [&first, &second] {
        batch.detach(node);
        batch.destroy(node);
    }
    let caps = f.owners.iter().collect::<Vec<_>>();
    let mut prepared = prepare(&mut f.dom, batch, &caps, None, limits()).unwrap();
    assert_eq!(
        prepared
            .occurrences()
            .iter()
            .map(|o| o.content().clone())
            .collect::<Vec<_>>(),
        [DocumentString::from("B"), DocumentString::from("A")]
    );
    assert_ne!(
        prepared.occurrences()[0].meta(),
        prepared.occurrences()[1].meta()
    );
    let actual = prepared.occurrences()[0].meta().clone();
    assert_eq!(&actual, prepared.occurrences()[0].meta());
    prepared
        .set_disabled(f.docs[0].node(), &f.sheets[0], true, &flag())
        .unwrap();
    prepared
        .set_disabled(f.docs[0].node(), &f.sheets[0], false, &flag())
        .unwrap();
    let (commit, dead) = prepared.commit(&flag()).unwrap().into_parts();
    assert_eq!(commit.roots().len(), 1);
    assert!(
        matches!(commit.roots()[0].cssom().publication(),CssomPublication::Unchanged {revision} if revision==before.1[0].revision())
    );
    assert_eq!(commit.dom().created(&first).unwrap(), None);
    assert_eq!(commit.dom().created(&second).unwrap(), None);
    assert!(
        !f.owners[0]
            .snapshot()
            .unwrap()
            .sheet(&f.sheets[0])
            .unwrap()
            .inputs()
            .disabled
    );
    drop(dead);
}
fn detached_subtree(dom: &mut Dom, document: DocumentId) -> NodeId {
    let mut batch = DomBatch::new();
    let container = batch.create_element_ns(document, Namespace::html(), "div");
    let meta = batch.create_element_ns(document, Namespace::html(), "meta");
    batch.set_attribute(&meta, "http-equiv", "default-style");
    batch.set_attribute(&meta, "content", "imported");
    batch.insert_before(&meta, &container, None);
    dom.apply(batch, &flag())
        .unwrap()
        .created(&container)
        .unwrap()
        .unwrap()
}
#[test]
fn same_owner_cloned_and_foreign_imported_destroyed_descendants_retain_real_private_identity() {
    for foreign in [false, true] {
        let mut f = fixture(u64::MAX);
        let (snapshot, node) = if foreign {
            let mut origin = Dom::new(f.dom.limits()).unwrap();
            let mut b = DomBatch::new();
            let d = b.create_document();
            let c = origin.apply(b, &flag()).unwrap();
            let d = c.document(&d).unwrap();
            let root = detached_subtree(&mut origin, d);
            (origin.snapshot(), root)
        } else {
            let root = detached_subtree(&mut f.dom, f.docs[0]);
            (f.dom.snapshot(), root)
        };
        let mut batch = DomBatch::new();
        let mut roots = Vec::new();
        for _ in 0..2 {
            let root = batch.import_node(&snapshot, node, f.docs[0], true);
            batch.insert_before(&root, f.heads[0], None);
            batch.detach(&root);
            batch.destroy(&root);
            roots.push(root);
        }
        let caps = f.owners.iter().collect::<Vec<_>>();
        let prepared = prepare(&mut f.dom, batch, &caps, None, limits()).unwrap();
        assert_eq!(prepared.occurrences().len(), 2);
        assert_ne!(
            prepared.occurrences()[0].meta(),
            prepared.occurrences()[1].meta()
        );
        for occurrence in prepared.occurrences() {
            assert_eq!(occurrence.document(), f.docs[0].node());
            assert_eq!(occurrence.content(), &DocumentString::from("imported"));
            assert!(matches!(
                occurrence.meta().kind,
                MetaSubjectKind::Created { .. }
            ));
        }
        let (commit, dead) = prepared.commit(&flag()).unwrap().into_parts();
        for root in &roots {
            assert_eq!(commit.dom().created(root).unwrap(), None);
        }
        assert!(matches!(
            commit.roots()[0].cssom().publication(),
            CssomPublication::Unchanged { .. }
        ));
        drop(dead);
    }
}
#[test]
fn missing_mismatched_busy_and_poisoned_owner_access_publish_nothing_and_unlock_prior_rows() {
    for mode in 0..4 {
        let mut f = fixture(u64::MAX);
        let before = snapshots(&f);
        let mut batch = DomBatch::new();
        for i in 0..2 {
            insert(&mut batch, f.docs[i], f.heads[i], "B".into());
        }
        let wrong = DomStyleCssomRoot::new(
            f.docs[1].node(),
            CssomStore::new(Default::default(), Default::default()).unwrap(),
        );
        if mode == 3 {
            let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
                let _actual = f.owners[1].lock().unwrap();
                panic!("authentic poison while retaining the actual canonical owner");
            }));
            assert!(result.is_err());
        }
        let held = if mode == 2 {
            Some(f.owners[1].lock().unwrap())
        } else {
            None
        };
        let caps = if mode == 0 {
            vec![&f.owners[0]]
        } else if mode == 1 {
            vec![&f.owners[0], &wrong]
        } else {
            f.owners.iter().collect()
        };
        let error = match prepare(&mut f.dom, batch, &caps, None, limits()) {
            Ok(_) => panic!("actual missing/mismatched/busy capability must refuse"),
            Err(error) => error,
        };
        match mode {
            0 => assert!(
                matches!(error,DomMetaAtomicError::MissingOwner {root} if root==f.docs[1].node())
            ),
            1 => assert!(
                matches!(error,DomMetaAtomicError::MismatchedOwner {root} if root==f.docs[1].node())
            ),
            2 => assert!(
                matches!(error, DomMetaAtomicError::OwnerBusy { root } if root == f.docs[1].node())
            ),
            _ => assert!(
                matches!(error, DomMetaAtomicError::OwnerPoisoned { root } if root == f.docs[1].node())
            ),
        }
        drop(held);
        if mode == 3 {
            assert_eq!(f.dom.snapshot().stamp(), before.0);
            assert_eq!(
                f.owners[0].snapshot().unwrap().revision(),
                before.1[0].revision()
            );
            let actual = f.owners[1].poisoned_snapshot_for_test();
            assert_eq!(actual.revision(), before.1[1].revision());
            assert!(!actual.sheet(&f.sheets[1]).unwrap().inputs().disabled);
            assert_eq!(
                f.dom
                    .inline_styles
                    .as_ref()
                    .unwrap()
                    .store
                    .as_ref()
                    .unwrap()
                    .snapshot()
                    .revision(),
                before.2.revision()
            );
        } else {
            unchanged(&f, &before);
        }
    }
}
#[test]
fn later_native_csp_refusal_precedes_any_producer_and_new_document_has_no_fabricated_root() {
    let mut f = fixture(u64::MAX);
    let before = snapshots(&f);
    let mut batch = DomBatch::new();
    insert(&mut batch, f.docs[0], f.heads[0], "B".into());
    let csp = batch.create_element_ns(f.docs[1], Namespace::html(), "meta");
    batch.set_attribute(&csp, "http-equiv", "content-security-policy");
    batch.set_attribute(&csp, "content", "default-src 'none'");
    batch.insert_before(&csp, f.heads[1], None);
    let caps = f.owners.iter().collect::<Vec<_>>();
    assert!(matches!(
        prepare(&mut f.dom, batch, &caps, None, limits()),
        Err(DomMetaAtomicError::Dom(DomError::Meta(
            DomMetaError::ContentSecurityPolicyUnsupported
        )))
    ));
    unchanged(&f, &before);
    let mut batch = DomBatch::new();
    let document = batch.create_document();
    let head = batch.create_element_ns(&document, Namespace::html(), "head");
    let meta = batch.create_element_ns(&document, Namespace::html(), "meta");
    batch.set_attribute(&meta, "http-equiv", "default-style");
    batch.set_attribute(&meta, "content", "B");
    batch.insert_before(&head, &document, None);
    batch.insert_before(&meta, &head, None);
    assert!(matches!(
        prepare(&mut f.dom, batch, &caps, None, limits()),
        Err(DomMetaAtomicError::Dom(DomError::Meta(
            DomMetaError::DefaultStyleUnsupported
        )))
    ));
    unchanged(&f, &before);
}
#[test]
fn prefix_is_revalidated_and_exhausted_work_is_not_reset_before_lower_preparation() {
    let mut f = fixture(u64::MAX);
    let before = snapshots(&f);
    let caps = f.owners.iter().collect::<Vec<_>>();
    let usage = DomMetaRuntimeUsage::new(64, 0, 0).unwrap();
    let prefix = DomMetaRuntimePrefix::new(usage, 0, limits()).unwrap();
    let tiny = DomMetaAtomicLimits::new(63, 1_000_000, 1_000_000, 128).unwrap();
    assert!(matches!(
        f.dom
            .prepare_meta_batch(DomBatch::new(), &caps, None, tiny, prefix, &flag()),
        Err(DomMetaAtomicError::Dom(DomError::ResourceLimit {
            resource: DomResource::MetaAtomicRetainedBytes,
            limit: 63
        }))
    ));
    let exhausted = DomMetaAtomicLimits::new(1_000_000, 1_000_000, 1, 128).unwrap();
    let prefix =
        DomMetaRuntimePrefix::new(DomMetaRuntimeUsage::new(0, 0, 0).unwrap(), 1, exhausted)
            .unwrap();
    assert!(matches!(
        f.dom
            .prepare_meta_batch(DomBatch::new(), &caps, None, exhausted, prefix, &flag()),
        Err(DomMetaAtomicError::Dom(DomError::Meta(
            DomMetaError::WorkLimit { limit: 1 }
        )))
    ));
    unchanged(&f, &before);
}
#[test]
fn repeated_whole_runtime_union_reports_do_not_add_or_refund_the_same_live_bank() {
    let mut f = fixture(u64::MAX);
    let actual_bank = vec![0u8; 80_000];
    let native = DomMetaAtomicLimits::new(100_000, 1000, 100_000, 128).unwrap();
    let usage = DomMetaRuntimeUsage::new(actual_bank.capacity(), 0, 0).unwrap();
    let prefix = DomMetaRuntimePrefix::new(usage, 0, native).unwrap();
    let caps = f.owners.iter().collect::<Vec<_>>();
    let mut prepared = f
        .dom
        .prepare_meta_batch(DomBatch::new(), &caps, None, native, prefix, &flag())
        .unwrap();
    for _ in 0..4 {
        prepared.admit_runtime(usage, &flag()).unwrap();
    }
    prepared
        .admit_runtime(DomMetaRuntimeUsage::new(0, 0, 0).unwrap(), &flag())
        .unwrap();
    assert!(matches!(
        prepared.admit_runtime(DomMetaRuntimeUsage::new(100_000, 0, 0).unwrap(), &flag()),
        Err(DomError::ResourceLimit {
            resource: DomResource::MetaAtomicRetainedBytes,
            ..
        })
    ));
    let (commit, dead) = prepared.commit(&flag()).unwrap().into_parts();
    assert!(commit.roots().is_empty());
    assert!(commit.inline().is_none());
    assert_eq!(actual_bank.len(), 80_000);
    drop(dead);
}
#[test]
fn authentic_observed_callback_cannot_use_ordinary_prefix_as_source_parent_permission() {
    let mut f = fixture(u64::MAX);
    f.dom
        .set_limits(f.dom.limits().with_dispatch_custody(
            DomDispatchCustodyLimits::new(64, 4096, 16_000_000, 16_000_000, 4_000_000).unwrap(),
        ))
        .unwrap();
    let mut batch = DomBatch::new();
    let dialog = batch.create_element_ns(f.docs[0], Namespace::html(), "dialog");
    batch.insert_before(&dialog, f.heads[0], None);
    let dialog = f
        .dom
        .apply(batch, &flag())
        .unwrap()
        .created(&dialog)
        .unwrap()
        .unwrap();
    let executor = f.dom.bind_source_event_executor(&flag()).unwrap();
    let source_limits = DomSourceLimits::new(64_000_000, 1_000_000, 10_000_000, 64, 4096, 4096, 4);
    let request = {
        let mut lease = f
            .dom
            .begin_source_edit(DomSourceEdit::show_dialog(dialog), source_limits, &flag())
            .unwrap();
        let DomSourceProgress::Request(DomSourceRequest::Event(request)) =
            lease.advance(&flag()).unwrap()
        else {
            panic!("actual beforetoggle request")
        };
        request
    };
    let mut permit = Some(executor.begin_dispatch(&f.dom, &request, &flag()).unwrap());
    let scope = executor
        .enter_dispatch(&f.dom, permit.as_ref().unwrap(), &flag())
        .unwrap();
    f.dom.require_source_entry().unwrap(); // this is an actual authorized observer
    let before = snapshots(&f);
    let work = f.dom.pending_source_edit().unwrap().spent_work();
    let mut batch = DomBatch::new();
    insert(&mut batch, f.docs[0], f.heads[0], "B".into());
    let caps = f.owners.iter().collect::<Vec<_>>();
    assert!(matches!(
        prepare(&mut f.dom, batch, &caps, None, limits()),
        Err(DomMetaAtomicError::Dom(DomError::SourceOperationPending))
    ));
    unchanged(&f, &before);
    assert_eq!(f.dom.pending_source_edit().unwrap().spent_work(), work);
    drop(scope);
    let mut receipt = Some(
        executor
            .seal_completed(
                &f.dom,
                &mut permit,
                DomSourceEventOutcome::Canceled,
                &flag(),
            )
            .unwrap(),
    );
    let mut lease = f
        .dom
        .resume_source_edit(request.operation(), source_limits, &flag())
        .unwrap();
    lease
        .accept_source_event(&request, &mut receipt, &flag())
        .unwrap();
    assert!(receipt.is_none());
    assert!(matches!(
        lease.advance(&flag()).unwrap(),
        DomSourceProgress::Complete(_)
    ));
    drop(lease);
    assert!(f.dom.pending_source_edit().is_none());
}
#[test]
fn replacement_bank_overlap_refuses_before_growing_the_retained_old_bank() {
    let native = DomMetaAtomicLimits::new(29, 0, 1000, 0).unwrap();
    let envelope = Envelope::new(native, DomMetaRuntimeUsage::new(0, 0, 0).unwrap(), 0).unwrap();
    let mut bank = Vec::<u8>::new();
    envelope.reserve(&mut bank, 10, &flag()).unwrap();
    bank.extend(0u8..10);
    assert_eq!(bank.capacity(), 10);
    assert!(matches!(
        envelope.reserve(&mut bank, 10, &flag()),
        Err(DomError::ResourceLimit {
            resource: DomResource::MetaAtomicRetainedBytes,
            limit: 29
        })
    ));
    assert_eq!(bank.capacity(), 10);
    assert_eq!(bank, (0u8..10).collect::<Vec<_>>());
    assert_eq!(envelope.bytes.get(), 10);
}
#[test]
fn shared_content_and_reinserted_meta_keep_distinct_occurrences_with_one_actual_payload() {
    let mut f = fixture(u64::MAX);
    let content = DocumentString::from("B".repeat(4096));
    let native = DomMetaAtomicLimits::new(
        1_000_000,
        content.shared_code_units().capacity(),
        1_000_000,
        128,
    )
    .unwrap();
    let mut batch = DomBatch::new();
    let first = insert(&mut batch, f.docs[0], f.heads[0], content.clone());
    insert(&mut batch, f.docs[0], f.heads[0], content);
    batch.detach(&first);
    batch.insert_before(&first, f.heads[0], None);
    let caps = f.owners.iter().collect::<Vec<_>>();
    let prepared = prepare(&mut f.dom, batch, &caps, None, native).unwrap();
    assert_eq!(prepared.occurrences().len(), 3);
    let rows = prepared.occurrences();
    assert_eq!(rows[0].meta(), rows[2].meta());
    assert_ne!(rows[0].meta(), rows[1].meta());
    for row in rows {
        assert!(Arc::ptr_eq(
            rows[0].content().shared_code_units(),
            row.content().shared_code_units()
        ));
    }
    let (commit, dead) = prepared.commit(&flag()).unwrap().into_parts();
    assert_eq!(commit.roots().len(), 1);
    drop(dead);
}
#[test]
fn authentic_cancellation_refuses_a_held_root_snapshot_lookup_without_work_or_publication() {
    let mut f = fixture(u64::MAX);
    let before = snapshots(&f);
    let mut batch = DomBatch::new();
    insert(&mut batch, f.docs[0], f.heads[0], "B".into());
    let caps = f.owners.iter().collect::<Vec<_>>();
    let prepared = prepare(&mut f.dom, batch, &caps, None, limits()).unwrap();
    let work = prepared.envelope.work.get();
    assert!(matches!(
        prepared.root_snapshot(f.docs[0].node(), &AtomicBool::new(true)),
        Err(DomMetaAtomicError::Dom(DomError::Cancelled))
    ));
    assert_eq!(prepared.envelope.work.get(), work);
    drop(prepared);
    unchanged(&f, &before);
}

mod first_binding;
