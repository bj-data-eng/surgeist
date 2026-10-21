//! A real cross-Document Source command retains an earlier Style receipt.
use super::*;
use surgeist_cssom::{CssomDeclarationSupport, CssomError, CssomLimits, CssomPublication};
use surgeist_dom::{
    DomInlineStyleInputs, DomInlineStyleLimits, DomInlineStyleProfile, DomInlineStyleRequestLimits,
    DomInlineStyleSourceContext,
};

#[test]
fn prior_document_style_receipt_and_new_document_meta_share_the_real_inline_source() {
    let (mut session, first) = native_session();
    let second = bind_document(&mut session, CssomLimits::default());
    let second_a = named_style(&mut session, second, "A".into());
    let second_b = named_style(&mut session, second, "B".into());
    let mut batch = DomBatch::new();
    let subtree = batch.create_element_ns(first, Namespace::html(), "div");
    let meta = batch.create_element_ns(first, Namespace::html(), "meta");
    batch.set_attribute(&meta, "http-equiv", "default-style");
    batch.set_attribute(&meta, "content", "B");
    batch.insert_before(&meta, &subtree, None);
    let style = batch.create_element_ns(first, Namespace::html(), "style");
    batch.set_attribute(&style, "title", "A");
    batch.insert_before(&style, &subtree, None);
    let made = session.dom().apply(batch, &cancel()).unwrap();
    let subtree = made.created(&subtree).unwrap().unwrap();
    let style = made.created(&style).unwrap().unwrap();
    let first_parent = container(&session, first);
    let initial = session
        .source_execution()
        .apply(
            &mut session.dom(),
            DomSourceEdit::insert_before(subtree, first_parent, None),
            source_limits(),
            &mut DomSourceServices {
                meta: Some(envelope()),
                execution_scope: None,
                style_policy: &mut policy,
                url: None,
            },
            &cancel(),
        )
        .unwrap();
    let DomSourceDriveProgress::MetaSource(initial) = initial else {
        panic!("initial real Meta");
    };
    let DomSourceProgress::Request(DomSourceRequest::Style(request)) = initial.source() else {
        panic!("later original Style");
    };
    let operation = request.source().operation();
    assert!(matches!(
        session
            .source_execution()
            .resume(
                &mut session.dom(),
                operation,
                source_limits(),
                &mut DomSourceServices {
                    meta: Some(envelope()),
                    execution_scope: None,
                    style_policy: &mut policy,
                    url: None
                },
                &cancel()
            )
            .unwrap(),
        DomSourceDriveProgress::Complete(_)
    ));
    let styles = session.style_execution();
    let old_sheet = styles.snapshot(first).unwrap().unwrap().sheets()[0].clone();
    let first_revision = styles.snapshot(first).unwrap().unwrap().revision().clone();
    let first_preference = observe(&mut session, first);
    session
        .dom()
        .raw_mut()
        .configure_inline_styles(
            DomInlineStyleLimits::new(CssomLimits::default(), 8, 16_000_000, 1_000_000, 1_000_000)
                .unwrap(),
            &cancel(),
        )
        .unwrap();
    let support = CssomDeclarationSupport::default();
    let version = CssomInputVersion {
        role: CssomInputRole::Support,
        identity: "cross-document-meta".into(),
        revision: 1,
    };
    let first_id = session
        .snapshot()
        .node(first)
        .unwrap()
        .as_document()
        .unwrap()
        .id();
    let second_id = session
        .snapshot()
        .node(second)
        .unwrap()
        .as_document()
        .unwrap()
        .id();
    let inputs = DomInlineStyleInputs::new(
        session.snapshot().document(first_id).unwrap().stamp(),
        CssParserContext::default(),
        version.clone(),
        support.clone(),
    )
    .unwrap();
    let requests = DomInlineStyleRequestLimits {
        input: CssUtf16InputLimits::default(),
        declaration: Default::default(),
    };
    let properties = session
        .dom()
        .raw_mut()
        .inline_style(
            subtree,
            &inputs,
            requests,
            |_, _| panic!("empty declaration"),
            &cancel(),
        )
        .unwrap();
    let historical = session
        .dom()
        .raw()
        .capture_inline_style(subtree)
        .unwrap()
        .unwrap();
    let profiles = [first_id, second_id]
        .into_iter()
        .map(|document| {
            DomInlineStyleProfile::new(
                subtree,
                document,
                CssParserContext::default(),
                version.clone(),
                support.clone(),
            )
            .unwrap()
        })
        .collect();
    let context = DomInlineStyleSourceContext::new(session.snapshot().stamp(), profiles).unwrap();
    let second_parent = container(&session, second);
    let mut progress = session
        .source_execution()
        .apply_with_inline_styles(
            &mut session.dom(),
            crate::DomSourceInlineStyleEdit::new(
                DomSourceEdit::insert_before(subtree, second_parent, None),
                context,
                requests,
            ),
            source_limits(),
            &mut DomSourceServices {
                meta: Some(envelope()),
                execution_scope: None,
                style_policy: &mut policy,
                url: None,
            },
            |_, _| panic!("context-only synchronization never reparses declarations"),
            &cancel(),
        )
        .unwrap();
    let mut earlier_inline = 0;
    let joined = loop {
        match progress {
            DomSourceDriveProgress::MetaSource(joined) => break joined,
            DomSourceDriveProgress::InlineSource {
                source: DomSourceProgress::Request(DomSourceRequest::Style(request)),
                publication,
            } => {
                earlier_inline += 1;
                assert!(matches!(
                    publication.inline().publication(),
                    CssomPublication::Changed(_)
                ));
                progress = session
                    .source_execution()
                    .resume_with_inline_styles(
                        &mut session.dom(),
                        request.source().operation(),
                        source_limits(),
                        &mut DomSourceServices {
                            meta: Some(envelope()),
                            execution_scope: None,
                            style_policy: &mut policy,
                            url: None,
                        },
                        |_, _| panic!("context-only synchronization"),
                        &cancel(),
                    )
                    .unwrap();
            }
            other => panic!("actual Meta boundary must be returned: {other:?}"),
        }
    };
    assert_eq!(joined.preferences().len(), 1);
    assert_eq!(joined.preferences()[0].observation().document(), second);
    assert_eq!(
        joined.roots().len(),
        1,
        "receipt-only A has no fabricated producer output"
    );
    assert_eq!(joined.roots()[0].root(), second);
    assert!(
        earlier_inline > 0 || joined.inline().is_some(),
        "the bound subtree made a real inline publication"
    );
    let after_first = styles.snapshot(first).unwrap().unwrap();
    assert_ne!(after_first.revision(), &first_revision);
    assert!(after_first.sheets().is_empty());
    assert!(
        after_first
            .sheet(&old_sheet)
            .unwrap()
            .inputs()
            .owner
            .is_none()
    );
    assert!(matches!(
        session.dom().raw().capture_inline_style(subtree),
        Err(surgeist_dom::DomInlineStyleError::Dom(
            DomError::IncompleteSourceOperation
        ))
    ));
    let DomSourceProgress::Request(DomSourceRequest::Style(request)) = joined.source() else {
        panic!("later new-Document Style request remains authentic");
    };
    let first_accepted_revision = after_first.revision().clone();
    let mut progress = session
        .source_execution()
        .resume_with_inline_styles(
            &mut session.dom(),
            request.source().operation(),
            source_limits(),
            &mut DomSourceServices {
                meta: Some(envelope()),
                execution_scope: None,
                style_policy: &mut policy,
                url: None,
            },
            |_, _| panic!("context-only synchronization"),
            &cancel(),
        )
        .unwrap();
    while let DomSourceDriveProgress::InlineSource {
        source: DomSourceProgress::Request(DomSourceRequest::Style(request)),
        ..
    } = progress
    {
        progress = session
            .source_execution()
            .resume_with_inline_styles(
                &mut session.dom(),
                request.source().operation(),
                source_limits(),
                &mut DomSourceServices {
                    meta: Some(envelope()),
                    execution_scope: None,
                    style_policy: &mut policy,
                    url: None,
                },
                |_, _| panic!("context-only synchronization"),
                &cancel(),
            )
            .unwrap();
    }
    assert!(matches!(
        progress,
        DomSourceDriveProgress::Complete(_)
            | DomSourceDriveProgress::InlineSource {
                source: DomSourceProgress::Complete(_),
                ..
            }
    ));
    assert_eq!(
        session.snapshot().node(subtree).unwrap().parent(),
        Some(second_parent)
    );
    assert_eq!(
        styles.snapshot(first).unwrap().unwrap().revision(),
        &first_accepted_revision,
        "accepted old-Document Style removal was not replayed"
    );
    let current = session
        .dom()
        .raw()
        .capture_inline_style(subtree)
        .unwrap()
        .unwrap();
    assert_eq!(current.properties().handle(), &properties);
    assert_ne!(
        current.properties().snapshot().revision(),
        historical.properties().snapshot().revision()
    );
    assert!(is_disabled(&mut session, second_a));
    assert!(!is_disabled(&mut session, second_b));
    assert!(is_disabled(&mut session, style));
    let first_after = observe(&mut session, first);
    assert_eq!(
        first_after.preferred_name(),
        first_preference.preferred_name()
    );
    assert_eq!(first_after.revision(), first_preference.revision());
}

#[test]
fn late_sheet_revision_failure_keeps_the_joined_meta_tree_names_and_inline_original() {
    let (mut session, _) = native_session();
    let document = bind_document(
        &mut session,
        CssomLimits {
            max_revision: 2,
            ..Default::default()
        },
    );
    let a = named_style(&mut session, document, "A".into());
    let b = named_style(&mut session, document, "B".into());
    let subject = detached_meta(&mut session, document, "B".into());
    session
        .dom()
        .raw_mut()
        .configure_inline_styles(
            DomInlineStyleLimits::new(CssomLimits::default(), 8, 16_000_000, 1_000_000, 1_000_000)
                .unwrap(),
            &cancel(),
        )
        .unwrap();
    let id = session
        .snapshot()
        .node(document)
        .unwrap()
        .as_document()
        .unwrap()
        .id();
    let version = CssomInputVersion {
        role: CssomInputRole::Support,
        identity: "late-native-meta".into(),
        revision: 1,
    };
    let inputs = DomInlineStyleInputs::new(
        session.snapshot().document(id).unwrap().stamp(),
        CssParserContext::default(),
        version.clone(),
        CssomDeclarationSupport::default(),
    )
    .unwrap();
    let requests = DomInlineStyleRequestLimits {
        input: CssUtf16InputLimits::default(),
        declaration: Default::default(),
    };
    let properties = session
        .dom()
        .raw_mut()
        .inline_style(
            subject,
            &inputs,
            requests,
            |_, _| panic!("empty"),
            &cancel(),
        )
        .unwrap();
    let historical = session
        .dom()
        .raw()
        .capture_inline_style(subject)
        .unwrap()
        .unwrap();
    let before = observe(&mut session, document);
    let canonical = session.snapshot();
    let producer = session
        .style_execution()
        .snapshot(document)
        .unwrap()
        .unwrap();
    let context = DomInlineStyleSourceContext::new(
        canonical.stamp(),
        vec![
            DomInlineStyleProfile::new(
                subject,
                id,
                CssParserContext::default(),
                version,
                CssomDeclarationSupport::default(),
            )
            .unwrap(),
        ],
    )
    .unwrap();
    let parent = container(&session, document);
    let error = session
        .source_execution()
        .apply_with_inline_styles(
            &mut session.dom(),
            crate::DomSourceInlineStyleEdit::new(
                DomSourceEdit::insert_before(subject, parent, None),
                context,
                requests,
            ),
            source_limits(),
            &mut DomSourceServices {
                meta: Some(envelope()),
                execution_scope: None,
                style_policy: &mut |_| panic!("no Style service"),
                url: None,
            },
            |_, _| panic!("context synchronization must not reparse"),
            &cancel(),
        )
        .unwrap_err();
    assert!(
        matches!(error, crate::DomSourceDriveError::Meta(DomStyleMetaError::Meta(surgeist_dom::DomMetaAtomicError::Cssom {root, error: CssomError::RevisionExhausted})) if root == document)
    );
    assert_eq!(session.snapshot().stamp(), canonical.stamp());
    assert_eq!(session.snapshot().node(subject).unwrap().parent(), None);
    assert_eq!(
        session
            .style_execution()
            .snapshot(document)
            .unwrap()
            .unwrap()
            .revision(),
        producer.revision()
    );
    let current = session
        .dom()
        .raw()
        .capture_inline_style(subject)
        .unwrap()
        .unwrap();
    assert_eq!(current.properties().handle(), &properties);
    assert_eq!(
        current.properties().snapshot().revision(),
        historical.properties().snapshot().revision()
    );
    let owner = session.style_execution().original().unwrap();
    let state = owner.borrow();
    let root = state
        .roots
        .iter()
        .find(|root| root.node == document)
        .unwrap();
    let current_name = root.sheet_sets.state().unwrap();
    assert_eq!(
        current_name.names().next().unwrap(),
        before.preferred_name()
    );
    drop(state);
    assert!(session.dom().pending_source_edit().is_some());
    // Exact snapshot flags stay available without invoking an unrelated current
    // association read while the original operation remains parked.
    for (element, disabled) in [(a, false), (b, true)] {
        let sheet = canonical
            .node(element)
            .unwrap()
            .as_style()
            .unwrap()
            .sheet()
            .unwrap()
            .unwrap();
        assert_eq!(
            producer.sheet(sheet.id()).unwrap().inputs().disabled,
            disabled
        );
        assert_eq!(
            session
                .style_execution()
                .snapshot(document)
                .unwrap()
                .unwrap()
                .sheet(sheet.id())
                .unwrap()
                .inputs()
                .disabled,
            disabled
        );
    }
}
