//! Genuine ordinary native preferred-name publication through Runtime.
use super::*;
use std::cell::Cell;
use surgeist_cssom::{CssomError, CssomLimits, CssomPublication};
use surgeist_dom::{DocumentString, DomMetaAtomicError, DomMetaAtomicLimits, DomMetaInlineBatch};

fn cancel() -> AtomicBool {
    AtomicBool::new(false)
}
fn envelope() -> DomMetaAtomicLimits {
    DomMetaAtomicLimits::new(16_000_000, 1_000_000, 1_000_000, 128).unwrap()
}
fn native_session() -> (DomSession, NodeId) {
    let (mut session, document) = bound_session();
    let limits = session.dom().limits().with_meta_processing_work(1_000_000);
    session.dom().raw_mut().set_limits(limits).unwrap();
    (session, document)
}
fn container(session: &DomSession, document: NodeId) -> NodeId {
    session.snapshot().node(document).unwrap().children()[0]
}
fn observe(session: &mut DomSession, document: NodeId) -> DomStyleSheetSetObservation {
    session
        .style_execution()
        .sheet_sets(&session.dom(), document, &cancel())
        .unwrap()
}
fn assert_names_equal(
    actual: &DomStyleSheetSetObservation,
    expected: &DomStyleSheetSetObservation,
) {
    assert_eq!(actual.stamp(), expected.stamp());
    assert_eq!(actual.document(), expected.document());
    assert_eq!(actual.revision(), expected.revision());
    assert_eq!(actual.cssom_revision(), expected.cssom_revision());
    assert_eq!(actual.preferred_name(), expected.preferred_name());
    assert_eq!(actual.last_selected_name(), expected.last_selected_name());
}
fn meta(
    batch: &mut DomBatch,
    document: NodeId,
    parent: NodeId,
    name: DocumentString,
) -> surgeist_dom::CreatedNode {
    let element = batch.create_element_ns(document, Namespace::html(), "meta");
    batch.set_attribute(&element, "http-equiv", "DeFaUlT-StYlE");
    batch.set_attribute(&element, "content", name);
    batch.insert_before(&element, parent, None);
    element
}
fn named_style(session: &mut DomSession, document: NodeId, title: DocumentString) -> NodeId {
    let mut batch = DomBatch::new();
    let style = batch.create_element_ns(document, Namespace::html(), "style");
    batch.set_attribute(&style, "title", title);
    let made = session.dom().apply(batch, &cancel()).unwrap();
    let style = made.created(&style).unwrap().unwrap();
    let parent = container(session, document);
    let result = session
        .source_execution()
        .apply(
            &mut session.dom(),
            DomSourceEdit::insert_before(style, parent, None),
            source_limits(),
            &mut DomSourceServices {
                execution_scope: None,
                style_policy: &mut policy,
                url: None,
            },
            &cancel(),
        )
        .unwrap();
    assert!(matches!(result, DomSourceDriveProgress::Complete(_)));
    style
}
fn is_disabled(session: &mut DomSession, style: NodeId) -> bool {
    session
        .style_execution()
        .disabled(&session.dom(), style)
        .unwrap()
        .disabled()
}
fn bind_document(session: &mut DomSession, producer: CssomLimits) -> NodeId {
    let mut batch = DomBatch::new();
    let document = batch.create_document();
    let parent = batch.create_element_ns(&document, Namespace::html(), "div");
    batch.insert_before(&parent, &document, None);
    let made = session.dom().apply(batch, &cancel()).unwrap();
    let document = made.created(&document).unwrap().unwrap();
    session
        .style_execution()
        .bind_root(
            &session.dom(),
            "other-native-root".into(),
            &mut Some(DomStyleCssomRoot::new(
                document,
                CssomStore::new(producer, Default::default()).unwrap(),
            )),
            &cancel(),
        )
        .unwrap();
    document
}

#[test]
fn zero_sheet_raw_name_survives_meta_removal_and_controls_later_real_add() {
    let (mut session, document) = native_session();
    let styles = session.style_execution();
    let before = observe(&mut session, document);
    let raw = DocumentString::from_utf16(vec![0x0042, 0xd800, 0x0062]);
    let mut batch = DomBatch::new();
    let ticket = meta(
        &mut batch,
        document,
        container(&session, document),
        raw.clone(),
    );
    batch.detach(&ticket);
    let joined = styles
        .apply_meta(&mut session.dom(), batch, None, envelope(), &cancel())
        .unwrap();
    assert_eq!(joined.preferences().len(), 1);
    assert_eq!(joined.commit().roots().len(), 1);
    assert!(joined.commit().inline().is_none());
    let node = joined.commit().dom().created(&ticket).unwrap().unwrap();
    assert!(session.snapshot().node(node).unwrap().parent().is_none());
    let observation = joined.preferences()[0].observation();
    assert_eq!(observation.preferred_name(), &raw);
    assert!(observation.last_selected_name().is_none());
    assert_eq!(observation.revision(), before.revision() + 1);
    assert_eq!(observation.stamp(), session.snapshot().stamp());
    assert!(
        matches!(joined.commit().roots()[0].cssom().publication(), CssomPublication::Unchanged { revision } if revision == before.cssom_revision())
    );
    assert_eq!(observation.cssom_revision(), before.cssom_revision());
    drop(joined);
    let other = named_style(&mut session, document, "B".into());
    let matching = named_style(&mut session, document, raw.clone());
    assert!(is_disabled(&mut session, other));
    assert!(!is_disabled(&mut session, matching));
    assert_eq!(observe(&mut session, document).preferred_name(), &raw);
}

#[test]
fn ordered_same_root_b_then_a_restores_flags_and_emits_only_one_real_commit() {
    let (mut session, document) = native_session();
    let styles = session.style_execution();
    let a = named_style(&mut session, document, "A".into());
    let b = named_style(&mut session, document, "B".into());
    let before = observe(&mut session, document);
    assert!(!is_disabled(&mut session, a));
    assert!(is_disabled(&mut session, b));
    let mut batch = DomBatch::new();
    let first = meta(
        &mut batch,
        document,
        container(&session, document),
        "B".into(),
    );
    let second = meta(
        &mut batch,
        document,
        container(&session, document),
        "A".into(),
    );
    let repeated = meta(
        &mut batch,
        document,
        container(&session, document),
        "A".into(),
    );
    batch.detach(&first);
    batch.destroy(&first);
    batch.detach(&second);
    batch.destroy(&second);
    batch.detach(&repeated);
    batch.destroy(&repeated);
    let joined = styles
        .apply_meta(&mut session.dom(), batch, None, envelope(), &cancel())
        .unwrap();
    assert_eq!(joined.preferences().len(), 3);
    assert_eq!(joined.commit().roots().len(), 1);
    assert_eq!(joined.commit().roots()[0].root(), document);
    assert!(joined.commit().dom().created(&first).unwrap().is_none());
    let current = styles.snapshot(document).unwrap().unwrap();
    for (row, (name, revision)) in joined.preferences().iter().zip([
        ("B", before.revision() + 1),
        ("A", before.revision() + 2),
        ("A", before.revision() + 2),
    ]) {
        assert_eq!(
            row.observation().preferred_name(),
            &DocumentString::from(name)
        );
        assert_eq!(row.observation().revision(), revision);
        assert_eq!(row.observation().cssom_revision(), current.revision());
        assert_eq!(row.observation().stamp(), session.snapshot().stamp());
    }
    assert!(!is_disabled(&mut session, a));
    assert!(is_disabled(&mut session, b));
    assert_eq!(
        observe(&mut session, document).preferred_name(),
        &DocumentString::from("A")
    );
}

#[test]
fn attribute_edits_do_not_replay_insertion_and_reinsertion_uses_actual_current_name() {
    let (mut session, document) = native_session();
    let styles = session.style_execution();
    let mut batch = DomBatch::new();
    let ticket = meta(
        &mut batch,
        document,
        container(&session, document),
        "B".into(),
    );
    let joined = styles
        .apply_meta(&mut session.dom(), batch, None, envelope(), &cancel())
        .unwrap();
    let element = joined.commit().dom().created(&ticket).unwrap().unwrap();
    let mut batch = DomBatch::new();
    batch.set_attribute(element, "content", "C");
    batch.set_attribute(element, "http-equiv", "DEFAULT-STYLE");
    let edited = styles
        .apply_meta(&mut session.dom(), batch, None, envelope(), &cancel())
        .unwrap();
    assert!(edited.preferences().is_empty());
    assert!(edited.commit().roots().is_empty());
    assert_eq!(
        observe(&mut session, document).preferred_name(),
        &DocumentString::from("B")
    );
    let mut batch = DomBatch::new();
    batch.detach(element);
    batch.insert_before(element, container(&session, document), None);
    let reinserted = styles
        .apply_meta(&mut session.dom(), batch, None, envelope(), &cancel())
        .unwrap();
    assert_eq!(reinserted.preferences().len(), 1);
    assert_eq!(
        reinserted.preferences()[0].observation().preferred_name(),
        &DocumentString::from("C")
    );
}

#[test]
fn authentic_explicit_empty_and_named_selections_preserve_flags_on_preferred_change() {
    for selection in ["", "A"] {
        let (mut session, document) = native_session();
        let mut styles = session.style_execution();
        let a = named_style(&mut session, document, "A".into());
        let b = named_style(&mut session, document, "B".into());
        let basis = observe(&mut session, document);
        styles
            .select_sheet_set(&session.dom(), &basis, &selection.into(), 1, &cancel())
            .unwrap();
        let flags = (is_disabled(&mut session, a), is_disabled(&mut session, b));
        let mut batch = DomBatch::new();
        meta(
            &mut batch,
            document,
            container(&session, document),
            "B".into(),
        );
        let joined = styles
            .apply_meta(&mut session.dom(), batch, None, envelope(), &cancel())
            .unwrap();
        assert_eq!(
            joined.preferences()[0].observation().last_selected_name(),
            Some(&DocumentString::from(selection))
        );
        assert_eq!(
            joined.preferences()[0].observation().preferred_name(),
            &DocumentString::from("B")
        );
        assert!(matches!(
            joined.commit().roots()[0].cssom().publication(),
            CssomPublication::Unchanged { .. }
        ));
        assert_eq!(
            (is_disabled(&mut session, a), is_disabled(&mut session, b)),
            flags
        );
    }
}

#[test]
fn late_second_producer_revision_refusal_preserves_every_document_and_name() {
    let (mut session, first) = native_session();
    let second = bind_document(
        &mut session,
        CssomLimits {
            max_revision: 2,
            ..Default::default()
        },
    );
    let styles = session.style_execution();
    let first_a = named_style(&mut session, first, "A".into());
    let first_b = named_style(&mut session, first, "B".into());
    let second_a = named_style(&mut session, second, "A".into());
    let second_b = named_style(&mut session, second, "B".into());
    let names = [observe(&mut session, first), observe(&mut session, second)];
    let revisions = [
        styles.snapshot(first).unwrap().unwrap().revision().clone(),
        styles.snapshot(second).unwrap().unwrap().revision().clone(),
    ];
    let before = session.snapshot().stamp();
    let usage = session.dom().usage();
    let mut batch = DomBatch::new();
    meta(&mut batch, first, container(&session, first), "B".into());
    meta(&mut batch, second, container(&session, second), "B".into());
    assert!(
        matches!(styles.apply_meta(&mut session.dom(), batch, None, envelope(), &cancel()), Err(DomStyleMetaError::Meta(DomMetaAtomicError::Cssom { root, error: CssomError::RevisionExhausted })) if root==second)
    );
    assert_eq!(session.snapshot().stamp(), before);
    assert_eq!(session.dom().usage(), usage);
    assert!(
        session
            .dom()
            .raw()
            .changes_since(before)
            .unwrap()
            .changes()
            .is_empty()
    );
    for (index, document) in [first, second].into_iter().enumerate() {
        assert_names_equal(&observe(&mut session, document), &names[index]);
        assert_eq!(
            styles.snapshot(document).unwrap().unwrap().revision(),
            &revisions[index]
        );
    }
    assert!(!is_disabled(&mut session, first_a));
    assert!(is_disabled(&mut session, first_b));
    assert!(!is_disabled(&mut session, second_a));
    assert!(is_disabled(&mut session, second_b));
}

#[test]
fn late_native_hook_cancellation_and_explicit_envelopes_leave_all_owners_unchanged() {
    for refusal in 0..5 {
        let (mut session, document) = native_session();
        let styles = session.style_execution();
        let a = named_style(&mut session, document, "A".into());
        let b = named_style(&mut session, document, "B".into());
        let before = observe(&mut session, document);
        let usage = session.dom().usage();
        let mut batch = DomBatch::new();
        meta(
            &mut batch,
            document,
            container(&session, document),
            "B".into(),
        );
        if refusal == 0 {
            let csp = batch.create_element_ns(document, Namespace::html(), "meta");
            batch.set_attribute(&csp, "http-equiv", "content-security-policy");
            batch.set_attribute(&csp, "content", "default-src 'none'");
            let head = batch.create_element_ns(document, Namespace::html(), "head");
            batch.insert_before(&head, container(&session, document), None);
            batch.insert_before(&csp, &head, None);
        }
        let limits = match refusal {
            2 => DomMetaAtomicLimits::new(0, 1_000_000, 1_000_000, 128).unwrap(),
            3 => DomMetaAtomicLimits::new(16_000_000, 1_000_000, 0, 128).unwrap(),
            4 => DomMetaAtomicLimits::new(16_000_000, 1_000_000, 1_000_000, 0).unwrap(),
            _ => envelope(),
        };
        assert!(
            styles
                .apply_meta(
                    &mut session.dom(),
                    batch,
                    None,
                    limits,
                    &AtomicBool::new(refusal == 1)
                )
                .is_err()
        );
        assert_eq!(session.snapshot().stamp(), before.stamp());
        assert_eq!(session.dom().usage(), usage);
        assert_names_equal(&observe(&mut session, document), &before);
        assert!(
            session
                .dom()
                .raw()
                .changes_since(before.stamp())
                .unwrap()
                .changes()
                .is_empty()
        );
        assert!(!is_disabled(&mut session, a));
        assert!(is_disabled(&mut session, b));
    }
}

fn inline_fixture(
    session: &mut DomSession,
    document: NodeId,
) -> (
    NodeId,
    surgeist_cssom::CssStyleProperties,
    surgeist_dom::DomInlineStyleProfile,
) {
    use surgeist_cssom::{CssomDeclarationSupport, CssomUsabilityRequirement};
    use surgeist_dom::{DomInlineStyleInputs, DomInlineStyleLimits, DomInlineStyleProfile};
    let mut batch = DomBatch::new();
    let element = batch.create_element_ns(document, Namespace::html(), "aside");
    batch.insert_before(&element, container(session, document), None);
    let made = session.dom().apply(batch, &cancel()).unwrap();
    let element = made.created(&element).unwrap().unwrap();
    session
        .dom()
        .raw_mut()
        .configure_inline_styles(
            DomInlineStyleLimits::new(CssomLimits::default(), 8, 16_000_000, 1_000_000, 1_000_000)
                .unwrap(),
            &cancel(),
        )
        .unwrap();
    let version = CssomInputVersion {
        role: CssomInputRole::Support,
        identity: "native-meta-width".into(),
        revision: 1,
    };
    let support = CssomDeclarationSupport::try_new(vec![(
        surgeist_css::CssPropertyGrammar::from_name("width").unwrap(),
        CssomUsabilityRequirement::WholeDeclarationDecision,
    )])
    .unwrap();
    let document = session
        .snapshot()
        .node(document)
        .unwrap()
        .as_document()
        .unwrap()
        .id();
    let inputs = DomInlineStyleInputs::new(
        session.snapshot().document(document).unwrap().stamp(),
        CssParserContext::default(),
        version.clone(),
        support.clone(),
    )
    .unwrap();
    let properties = session
        .dom()
        .raw_mut()
        .inline_style(
            element,
            &inputs,
            inline_requests(),
            |_, _| panic!("empty initial declaration"),
            &cancel(),
        )
        .unwrap();
    let profile = DomInlineStyleProfile::new(
        element,
        document,
        CssParserContext::default(),
        version,
        support,
    )
    .unwrap();
    (element, properties, profile)
}
fn inline_requests() -> surgeist_dom::DomInlineStyleRequestLimits {
    surgeist_dom::DomInlineStyleRequestLimits {
        input: CssUtf16InputLimits::default(),
        declaration: Default::default(),
    }
}
fn decide_width(
    candidates: &[surgeist_cssom::CssomDeclarationCandidate],
    context: &CssomContext,
) -> Result<surgeist_cssom::CssomAdmission, CssomError> {
    let version = context
        .inputs
        .iter()
        .find(|input| input.version.role == CssomInputRole::Support)
        .unwrap()
        .version
        .clone();
    Ok(surgeist_cssom::CssomAdmission::new(
        candidates
            .iter()
            .map(|candidate| {
                candidate
                    .decide(
                        version.clone(),
                        surgeist_cssom::CssomDeclarationUsability::Usable,
                    )
                    .unwrap()
            })
            .collect(),
        context
            .inputs
            .iter()
            .map(|input| input.version.clone())
            .collect(),
    ))
}

#[test]
fn two_documents_and_inline_publish_one_authentic_complete_atomic_result() {
    let (mut session, first) = native_session();
    let second = bind_document(&mut session, CssomLimits::default());
    let styles = session.style_execution();
    let first_a = named_style(&mut session, first, "A".into());
    let first_b = named_style(&mut session, first, "B".into());
    let second_a = named_style(&mut session, second, "A".into());
    let second_b = named_style(&mut session, second, "B".into());
    let persistent = append_style(&mut session, first, None);
    let (element, properties, profile) = inline_fixture(&mut session, first);
    let before = session.snapshot().stamp();
    let old_inline = session
        .dom()
        .raw()
        .capture_inline_style(element)
        .unwrap()
        .unwrap();
    let mut batch = DomBatch::new();
    meta(&mut batch, first, container(&session, first), "B".into());
    meta(&mut batch, second, container(&session, second), "B".into());
    batch.set_attribute(element, "style", "width: 17px;");
    let context =
        surgeist_dom::DomInlineStyleBatchContext::new(&batch, before, vec![profile]).unwrap();
    let calls = Cell::new(0);
    let mut resolver = |candidates: &[surgeist_cssom::CssomDeclarationCandidate],
                        context: &CssomContext| {
        calls.set(calls.get() + 1);
        decide_width(candidates, context)
    };
    let joined = styles
        .apply_meta(
            &mut session.dom(),
            batch,
            Some(DomMetaInlineBatch::new(
                context,
                inline_requests(),
                &mut resolver,
            )),
            envelope(),
            &cancel(),
        )
        .unwrap();
    assert_eq!(calls.get(), 1);
    assert_eq!(joined.commit().dom().changes().from(), before);
    assert_eq!(
        joined.commit().dom().changes().to(),
        session.snapshot().stamp()
    );
    assert_eq!(joined.preferences().len(), 2);
    assert_eq!(joined.commit().roots().len(), 2);
    for (index, document) in [first, second].into_iter().enumerate() {
        assert_eq!(joined.commit().roots()[index].root(), document);
        assert!(matches!(
            joined.commit().roots()[index].cssom().publication(),
            CssomPublication::Changed(_)
        ));
        let observation = joined.preferences()[index].observation();
        assert_eq!(observation.document(), document);
        assert_eq!(observation.preferred_name(), &DocumentString::from("B"));
        assert_eq!(
            observation.cssom_revision(),
            styles.snapshot(document).unwrap().unwrap().revision()
        );
    }
    let inline = joined
        .commit()
        .inline()
        .expect("actual mixed declaration commit");
    assert!(matches!(inline.publication(), CssomPublication::Changed(_)));
    assert!(
        inline.owner_effects().is_empty(),
        "DOM ingestion does not fabricate owner-write feedback"
    );
    let current_inline = session
        .dom()
        .raw()
        .capture_inline_style(element)
        .unwrap()
        .unwrap();
    assert_eq!(current_inline.properties().handle(), &properties);
    assert_eq!(
        current_inline
            .properties()
            .get_property_value("width", inline_requests().declaration)
            .unwrap(),
        "17px"
    );
    assert_eq!(
        old_inline
            .properties()
            .get_property_value("width", inline_requests().declaration)
            .unwrap(),
        ""
    );
    assert!(is_disabled(&mut session, first_a));
    assert!(!is_disabled(&mut session, first_b));
    assert!(is_disabled(&mut session, second_a));
    assert!(!is_disabled(&mut session, second_b));
    assert!(
        !is_disabled(&mut session, persistent),
        "unnamed persistent sheet is not an enable-set target"
    );
}

#[test]
fn mixed_inline_refusal_and_late_second_owner_refusal_preserve_all_three_owners() {
    for refusal in 0..3 {
        let (mut session, first) = native_session();
        let second = bind_document(
            &mut session,
            CssomLimits {
                max_revision: if refusal == 0 { 2 } else { u64::MAX },
                ..Default::default()
            },
        );
        let styles = session.style_execution();
        let first_a = named_style(&mut session, first, "A".into());
        let first_b = named_style(&mut session, first, "B".into());
        let second_a = named_style(&mut session, second, "A".into());
        let second_b = named_style(&mut session, second, "B".into());
        let (element, properties, profile) = inline_fixture(&mut session, first);
        let names = [observe(&mut session, first), observe(&mut session, second)];
        let before = session.snapshot().stamp();
        let usage = session.dom().usage();
        let original_inline = session
            .dom()
            .raw()
            .capture_inline_style(element)
            .unwrap()
            .unwrap();
        let mut batch = DomBatch::new();
        meta(&mut batch, first, container(&session, first), "B".into());
        meta(&mut batch, second, container(&session, second), "B".into());
        batch.set_attribute(element, "style", "width: 19px;");
        let context =
            surgeist_dom::DomInlineStyleBatchContext::new(&batch, before, vec![profile]).unwrap();
        let mut request = inline_requests();
        if refusal == 1 {
            request.declaration.max_entries = 0;
        }
        let operation_cancel = AtomicBool::new(false);
        let calls = Cell::new(0);
        let mut resolver = |candidates: &[surgeist_cssom::CssomDeclarationCandidate],
                            context: &CssomContext| {
            calls.set(calls.get() + 1);
            let admitted = decide_width(candidates, context)?;
            if refusal == 2 {
                operation_cancel.store(true, Ordering::Relaxed);
            }
            Ok(admitted)
        };
        let result = styles.apply_meta(
            &mut session.dom(),
            batch,
            Some(DomMetaInlineBatch::new(context, request, &mut resolver)),
            envelope(),
            &operation_cancel,
        );
        match refusal {
            0 => assert!(
                matches!(result, Err(DomStyleMetaError::Meta(DomMetaAtomicError::Cssom { root, error: CssomError::RevisionExhausted })) if root == second)
            ),
            1 => assert!(matches!(
                result,
                Err(DomStyleMetaError::Meta(DomMetaAtomicError::Inline(
                    surgeist_dom::DomInlineStyleError::Cssom(CssomError::Limit { maximum: 0, .. })
                )))
            )),
            2 => assert!(result.is_err()),
            _ => unreachable!(),
        }
        if refusal == 2 {
            assert_eq!(
                calls.get(),
                1,
                "real inline admission precedes late cancellation"
            );
            assert!(operation_cancel.load(Ordering::Relaxed));
        }
        assert_eq!(session.snapshot().stamp(), before);
        assert_eq!(session.dom().usage(), usage);
        for (index, document) in [first, second].into_iter().enumerate() {
            assert_names_equal(&observe(&mut session, document), &names[index]);
        }
        let current = session
            .dom()
            .raw()
            .capture_inline_style(element)
            .unwrap()
            .unwrap();
        assert_eq!(current.properties().handle(), &properties);
        assert_eq!(
            current
                .properties()
                .get_property_value("width", inline_requests().declaration)
                .unwrap(),
            ""
        );
        assert_eq!(
            current.properties().snapshot().revision(),
            original_inline.properties().snapshot().revision()
        );
        assert!(!is_disabled(&mut session, first_a));
        assert!(is_disabled(&mut session, first_b));
        assert!(!is_disabled(&mut session, second_a));
        assert!(is_disabled(&mut session, second_b));
    }
}

#[test]
fn raw_dom_refusal_and_unbound_or_foreign_runtime_capabilities_remain_precise() {
    let (mut session, document) = native_session();
    let styles = session.style_execution();
    let before = session.snapshot().stamp();
    let mut raw = DomBatch::new();
    meta(
        &mut raw,
        document,
        container(&session, document),
        "B".into(),
    );
    assert!(matches!(
        session.dom().apply(raw, &cancel()),
        Err(DomError::Meta(
            surgeist_dom::DomMetaError::DefaultStyleUnsupported
        ))
    ));
    assert_eq!(session.snapshot().stamp(), before);
    let mut make = DomBatch::new();
    let unbound = make.create_document();
    let parent = make.create_element_ns(&unbound, Namespace::html(), "div");
    make.insert_before(&parent, &unbound, None);
    let made = session.dom().apply(make, &cancel()).unwrap();
    let unbound = made.created(&unbound).unwrap().unwrap();
    let parent = made.created(&parent).unwrap().unwrap();
    let unchanged = session.snapshot().stamp();
    let mut batch = DomBatch::new();
    meta(&mut batch, unbound, parent, "C".into());
    assert!(
        matches!(styles.apply_meta(&mut session.dom(), batch, None, envelope(), &cancel()), Err(DomStyleMetaError::Meta(DomMetaAtomicError::MissingOwner { root })) if root==unbound)
    );
    assert_eq!(session.snapshot().stamp(), unchanged);
    let (mut foreign, other) = native_session();
    let mut batch = DomBatch::new();
    meta(&mut batch, other, container(&foreign, other), "C".into());
    assert!(matches!(
        styles.apply_meta(&mut foreign.dom(), batch, None, envelope(), &cancel()),
        Err(DomStyleMetaError::Runtime(
            DomStyleRuntimeError::WrongSession
        ))
    ));
    assert!(observe(&mut foreign, other).preferred_name().is_empty());
}

#[test]
fn actual_runtime_reentry_and_name_storage_refusal_cannot_publish_a_native_batch() {
    let (mut session, document) = native_session();
    let styles = session.style_execution();
    let before = observe(&mut session, document);
    let original = styles.original().unwrap();
    let held = original.borrow_mut();
    let mut batch = DomBatch::new();
    meta(
        &mut batch,
        document,
        container(&session, document),
        "B".into(),
    );
    assert!(matches!(
        styles.apply_meta(&mut session.dom(), batch, None, envelope(), &cancel()),
        Err(DomStyleMetaError::Runtime(DomStyleRuntimeError::Binding))
    ));
    drop(held);
    assert_names_equal(&observe(&mut session, document), &before);
    let current = {
        let state = original.borrow();
        state.storage(None, &cancel(), state.limits.work).unwrap()
    };
    let original_limits = original.borrow().limits;
    let mut tight = original_limits;
    tight.storage_bytes = current;
    session
        .style_execution()
        .set_limits(&session.dom(), tight, &cancel())
        .unwrap();
    let mut batch = DomBatch::new();
    meta(
        &mut batch,
        document,
        container(&session, document),
        DocumentString::from_utf16(vec![0x42; 2048]),
    );
    assert!(matches!(
        styles.apply_meta(&mut session.dom(), batch, None, envelope(), &cancel()),
        Err(DomStyleMetaError::Runtime(DomStyleRuntimeError::Admission))
    ));
    session
        .style_execution()
        .set_limits(&session.dom(), original_limits, &cancel())
        .unwrap();
    assert_names_equal(&observe(&mut session, document), &before);
}

#[test]
fn detached_empty_and_shadow_only_meta_do_not_select_a_document_set() {
    let (mut session, document) = native_session();
    let styles = session.style_execution();
    let before = observe(&mut session, document);
    let mut batch = DomBatch::new();
    let detached = batch.create_element_ns(document, Namespace::html(), "meta");
    batch.set_attribute(&detached, "http-equiv", "default-style");
    batch.set_attribute(&detached, "content", "detached");
    meta(
        &mut batch,
        document,
        container(&session, document),
        DocumentString::default(),
    );
    let host = batch.create_element_ns(document, Namespace::html(), "div");
    let shadow = batch.attach_shadow(
        &host,
        surgeist_dom::DomShadowRootInit::new(surgeist_html::HtmlShadowRootMode::Closed),
    );
    let hidden = batch.create_element_ns(document, Namespace::html(), "meta");
    batch.set_attribute(&hidden, "http-equiv", "default-style");
    batch.set_attribute(&hidden, "content", "shadow");
    batch.insert_before(&hidden, &shadow, None);
    batch.insert_before(&host, container(&session, document), None);
    let joined = styles
        .apply_meta(&mut session.dom(), batch, None, envelope(), &cancel())
        .unwrap();
    assert!(joined.preferences().is_empty());
    assert!(joined.commit().roots().is_empty());
    let after = observe(&mut session, document);
    assert_eq!(after.preferred_name(), before.preferred_name());
    assert_eq!(after.last_selected_name(), before.last_selected_name());
    assert_eq!(after.revision(), before.revision());
    assert_eq!(after.cssom_revision(), before.cssom_revision());
}

#[test]
fn shared_original_name_backing_is_charged_once_across_native_and_runtime_owners() {
    let raw = DocumentString::from_utf16(vec![0x0042, 0xd800, 0x0062]);
    let capacity = raw.shared_code_units().capacity();
    assert!(capacity > 0);
    for admitted_units in [capacity - 1, capacity] {
        let (mut session, document) = native_session();
        let styles = session.style_execution();
        let before = observe(&mut session, document);
        let usage = session.dom().usage();
        let mut batch = DomBatch::new();
        meta(
            &mut batch,
            document,
            container(&session, document),
            raw.clone(),
        );
        meta(
            &mut batch,
            document,
            container(&session, document),
            raw.clone(),
        );
        let limits = DomMetaAtomicLimits::new(16_000_000, admitted_units, 1_000_000, 128).unwrap();
        let result = styles.apply_meta(&mut session.dom(), batch, None, limits, &cancel());
        if admitted_units == capacity {
            let joined = result.unwrap();
            assert_eq!(joined.preferences().len(), 2);
            assert_eq!(joined.commit().roots().len(), 1);
            assert!(matches!(
                joined.commit().roots()[0].cssom().publication(),
                CssomPublication::Unchanged { .. }
            ));
            for row in joined.preferences() {
                assert_eq!(row.observation().preferred_name(), &raw);
                assert!(Arc::ptr_eq(
                    row.observation().preferred_name().shared_code_units(),
                    raw.shared_code_units()
                ));
                assert_eq!(row.observation().revision(), before.revision() + 1);
            }
        } else {
            assert!(
                matches!(result, Err(DomStyleMetaError::Meta(DomMetaAtomicError::Dom(DomError::ResourceLimit { resource: surgeist_dom::DomResource::MetaAtomicRetainedUnits, limit }))) if limit == admitted_units)
            );
            assert_eq!(session.dom().usage(), usage);
            assert_names_equal(&observe(&mut session, document), &before);
            assert!(
                session
                    .dom()
                    .raw()
                    .changes_since(before.stamp())
                    .unwrap()
                    .changes()
                    .is_empty()
            );
        }
    }
}

#[test]
fn initial_work_overflow_and_zero_native_work_refuse_before_any_semantic_publication() {
    for maximum in [u64::MAX, 0] {
        let (mut session, document) = native_session();
        let styles = session.style_execution();
        let before = observe(&mut session, document);
        let usage = session.dom().usage();
        let owner = styles.original().unwrap();
        let work = owner.borrow().work.get();
        assert!(
            work > 0,
            "actual binding/observation already spent Runtime work"
        );
        let mut batch = DomBatch::new();
        meta(
            &mut batch,
            document,
            container(&session, document),
            "B".into(),
        );
        let limits = DomMetaAtomicLimits::new(16_000_000, 1_000_000, maximum, 128).unwrap();
        let result = styles.apply_meta(&mut session.dom(), batch, None, limits, &cancel());
        if maximum == u64::MAX {
            assert!(matches!(
                result,
                Err(DomStyleMetaError::Runtime(DomStyleRuntimeError::Dom(
                    DomError::RepresentationExhausted
                )))
            ));
            assert_eq!(
                owner.borrow().work.get(),
                work,
                "checked overflow precedes controlled bank work"
            );
        } else {
            assert!(matches!(
                result,
                Err(DomStyleMetaError::Runtime(DomStyleRuntimeError::Admission))
            ));
            assert!(
                owner.borrow().work.get() > work,
                "the existing Runtime counter retains the refused admission attempt"
            );
        }
        assert_eq!(session.dom().usage(), usage);
        assert_names_equal(&observe(&mut session, document), &before);
        assert!(
            session
                .dom()
                .raw()
                .changes_since(before.stamp())
                .unwrap()
                .changes()
                .is_empty()
        );
    }
}

#[test]
fn initial_empty_binding_is_not_published_when_a_later_real_producer_refuses() {
    let (mut session, first) = native_session();
    let second = bind_document(
        &mut session,
        CssomLimits {
            max_revision: 2,
            ..Default::default()
        },
    );
    let styles = session.style_execution();
    named_style(&mut session, second, "A".into());
    named_style(&mut session, second, "B".into());
    assert!(session.dom().style_cssom_root(first).unwrap().is_none());
    assert!(session.dom().style_cssom_root(second).unwrap().is_some());
    let names = [observe(&mut session, first), observe(&mut session, second)];
    let revisions = [
        styles.snapshot(first).unwrap().unwrap().revision().clone(),
        styles.snapshot(second).unwrap().unwrap().revision().clone(),
    ];
    let before = session.snapshot().stamp();
    let usage = session.dom().usage();
    let mut batch = DomBatch::new();
    meta(&mut batch, first, container(&session, first), "B".into());
    meta(&mut batch, second, container(&session, second), "B".into());
    assert!(
        matches!(styles.apply_meta(&mut session.dom(), batch, None, envelope(), &cancel()), Err(DomStyleMetaError::Meta(DomMetaAtomicError::Cssom { root, error: CssomError::RevisionExhausted })) if root == second)
    );
    assert!(
        session.dom().style_cssom_root(first).unwrap().is_none(),
        "first owner registration belongs to the same uncommitted fence"
    );
    assert_eq!(session.snapshot().stamp(), before);
    assert_eq!(session.dom().usage(), usage);
    assert!(
        session
            .dom()
            .raw()
            .changes_since(before)
            .unwrap()
            .changes()
            .is_empty()
    );
    for (index, document) in [first, second].into_iter().enumerate() {
        assert_names_equal(&observe(&mut session, document), &names[index]);
        assert_eq!(
            styles.snapshot(document).unwrap().unwrap().revision(),
            &revisions[index]
        );
    }
}

#[test]
fn invoked_then_destroyed_initial_meta_retains_names_and_owner_with_unchanged_final_dom() {
    let (mut session, document) = native_session();
    let styles = session.style_execution();
    let before = observe(&mut session, document);
    assert!(session.dom().style_cssom_root(document).unwrap().is_none());
    let mut batch = DomBatch::new();
    let ticket = meta(
        &mut batch,
        document,
        container(&session, document),
        "B".into(),
    );
    batch.detach(&ticket);
    batch.destroy(&ticket);
    let joined = styles
        .apply_meta(&mut session.dom(), batch, None, envelope(), &cancel())
        .unwrap();
    assert!(joined.commit().dom().created(&ticket).unwrap().is_none());
    assert!(joined.commit().dom().changes().changes().is_empty());
    assert_eq!(session.snapshot().stamp(), before.stamp());
    assert_eq!(joined.commit().roots().len(), 1);
    assert!(
        matches!(joined.commit().roots()[0].cssom().publication(), CssomPublication::Unchanged { revision } if revision == before.cssom_revision())
    );
    assert_eq!(joined.preferences().len(), 1);
    assert_eq!(
        joined.preferences()[0].observation().preferred_name(),
        &DocumentString::from("B")
    );
    assert_eq!(
        joined.preferences()[0].observation().revision(),
        before.revision() + 1
    );
    drop(joined);
    let canonical = session
        .dom()
        .style_cssom_root(document)
        .unwrap()
        .expect("surviving Document retains its actual admitted owner");
    assert_eq!(
        canonical.snapshot().unwrap().revision(),
        before.cssom_revision()
    );
    let current = observe(&mut session, document);
    assert_eq!(current.preferred_name(), &DocumentString::from("B"));
    assert!(current.last_selected_name().is_none());
}
