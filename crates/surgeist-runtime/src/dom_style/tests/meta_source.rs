//! Real Session Source preferred-name boundaries and subsequent native work.
mod cross_document;
mod events;
use super::meta::*;
use super::*;
use surgeist_dom::{DocumentString, DomSourceProgress, DomSourceRequest};

#[test]
fn one_source_subtree_preserves_b_then_a_order_and_one_final_producer_basis() {
    let (mut session, document) = native_session();
    let a = named_style(&mut session, document, "A".into());
    let b = named_style(&mut session, document, "B".into());
    let before = observe(&mut session, document);
    let mut batch = DomBatch::new();
    let subtree = batch.create_element_ns(document, Namespace::html(), "div");
    for name in ["B", "A"] {
        let meta = batch.create_element_ns(document, Namespace::html(), "meta");
        batch.set_attribute(&meta, "http-equiv", "default-style");
        batch.set_attribute(&meta, "content", name);
        batch.insert_before(&meta, &subtree, None);
    }
    let made = session.dom().apply(batch, &cancel()).unwrap();
    let subtree = made.created(&subtree).unwrap().unwrap();
    let parent = container(&session, document);
    let result = session
        .source_execution()
        .apply(
            &mut session.dom(),
            DomSourceEdit::insert_before(subtree, parent, None),
            source_limits(),
            &mut DomSourceServices {
                meta: Some(envelope()),
                execution_scope: None,
                style_policy: &mut |_| panic!("no later service before output"),
                url: None,
            },
            &cancel(),
        )
        .unwrap();
    let DomSourceDriveProgress::MetaSource(outcome) = result else {
        panic!("one actual joined Source publication");
    };
    assert!(matches!(outcome.source(), DomSourceProgress::Complete(_)));
    assert_eq!(outcome.roots().len(), 1);
    assert_eq!(outcome.preferences().len(), 2);
    let final_cssom = session
        .style_execution()
        .snapshot(document)
        .unwrap()
        .unwrap();
    for (preference, (name, revision)) in outcome
        .preferences()
        .iter()
        .zip([("B", before.revision() + 1), ("A", before.revision() + 2)])
    {
        assert_eq!(
            preference.observation().preferred_name(),
            &DocumentString::from(name)
        );
        assert_eq!(preference.observation().revision(), revision);
        assert_eq!(
            preference.observation().cssom_revision(),
            final_cssom.revision()
        );
    }
    assert_ne!(
        outcome.preferences()[0].meta(),
        outcome.preferences()[1].meta()
    );
    assert!(!is_disabled(&mut session, a));
    assert!(is_disabled(&mut session, b));
}

fn detached_meta(session: &mut DomSession, document: NodeId, name: DocumentString) -> NodeId {
    let mut batch = DomBatch::new();
    let meta = batch.create_element_ns(document, Namespace::html(), "meta");
    batch.set_attribute(&meta, "http-equiv", "default-style");
    batch.set_attribute(&meta, "content", name);
    let made = session.dom().apply(batch, &cancel()).unwrap();
    made.created(&meta).unwrap().unwrap()
}

#[test]
fn source_zero_sheet_raw_name_returns_its_complete_and_controls_the_next_real_add() {
    let (mut session, document) = native_session();
    let raw = DocumentString::from_utf16(vec![66, 0xd800, 98]);
    let subject = detached_meta(&mut session, document, raw.clone());
    let parent = container(&session, document);
    let result = session
        .source_execution()
        .apply(
            &mut session.dom(),
            DomSourceEdit::insert_before(subject, parent, None),
            source_limits(),
            &mut DomSourceServices {
                meta: Some(envelope()),
                execution_scope: None,
                style_policy: &mut |_| panic!("no Style request before the returned Meta output"),
                url: None,
            },
            &cancel(),
        )
        .unwrap();
    let DomSourceDriveProgress::MetaSource(outcome) = result else {
        panic!("authentic Meta output");
    };
    let DomSourceProgress::Complete(done) = outcome.source() else {
        panic!("original completion");
    };
    assert!(session.dom().pending_source_edit().is_none());
    assert_eq!(outcome.preferences().len(), 1);
    assert_eq!(
        outcome.preferences()[0].observation().preferred_name(),
        &raw
    );
    assert_eq!(
        outcome.preferences()[0].observation().stamp(),
        done.commit().changes().to()
    );
    assert_eq!(outcome.roots().len(), 1);
    assert_eq!(outcome.roots()[0].root(), document);
    assert!(matches!(
        outcome.roots()[0].cssom().publication(),
        surgeist_cssom::CssomPublication::Unchanged { .. }
    ));
    assert!(outcome.inline().is_none());
    assert_eq!(observe(&mut session, document).preferred_name(), &raw);
    let a = named_style(&mut session, document, "A".into());
    let selected = named_style(&mut session, document, raw);
    assert!(is_disabled(&mut session, a));
    assert!(!is_disabled(&mut session, selected));
}

#[test]
fn source_same_name_noop_still_returns_each_actual_occurrence_and_unchanged_receipt() {
    let (mut session, document) = native_session();
    let mut original = None;
    for _ in 0..2 {
        let subject = detached_meta(&mut session, document, "B".into());
        let parent = container(&session, document);
        let result = session
            .source_execution()
            .apply(
                &mut session.dom(),
                DomSourceEdit::insert_before(subject, parent, None),
                source_limits(),
                &mut DomSourceServices {
                    meta: Some(envelope()),
                    execution_scope: None,
                    style_policy: &mut |_| panic!("no later service"),
                    url: None,
                },
                &cancel(),
            )
            .unwrap();
        let DomSourceDriveProgress::MetaSource(outcome) = result else {
            panic!("no-op occurrence is observable");
        };
        assert!(matches!(outcome.source(), DomSourceProgress::Complete(_)));
        assert_eq!(outcome.preferences().len(), 1);
        assert_eq!(outcome.roots().len(), 1);
        assert!(matches!(
            outcome.roots()[0].cssom().publication(),
            surgeist_cssom::CssomPublication::Unchanged { .. }
        ));
        let current = outcome.preferences()[0].observation();
        if let Some((revision, cssom)) = original.as_ref() {
            assert_eq!(&current.revision(), revision);
            assert_eq!(current.cssom_revision(), cssom);
        }
        original = Some((current.revision(), current.cssom_revision().clone()));
    }
}

#[test]
fn source_prior_style_then_meta_returns_before_later_style_policy_and_addition() {
    let (mut session, document) = native_session();
    let mut batch = DomBatch::new();
    let subtree = batch.create_element_ns(document, Namespace::html(), "div");
    let first = batch.create_element_ns(document, Namespace::html(), "style");
    batch.set_attribute(&first, "title", "A");
    batch.set_attribute(&first, "media", "screen and");
    batch.insert_before(&first, &subtree, None);
    let meta = batch.create_element_ns(document, Namespace::html(), "meta");
    batch.set_attribute(&meta, "http-equiv", "default-style");
    batch.set_attribute(&meta, "content", "B");
    batch.insert_before(&meta, &subtree, None);
    let last = batch.create_element_ns(document, Namespace::html(), "style");
    batch.set_attribute(&last, "title", "B");
    batch.insert_before(&last, &subtree, None);
    let made = session.dom().apply(batch, &cancel()).unwrap();
    let subtree = made.created(&subtree).unwrap().unwrap();
    let first = made.created(&first).unwrap().unwrap();
    let last = made.created(&last).unwrap().unwrap();
    let calls = std::cell::Cell::new(0);
    let mut observation = |source: &DomStyleSource| {
        calls.set(calls.get() + 1);
        policy(source)
    };
    let parent = container(&session, document);
    let result = session
        .source_execution()
        .apply(
            &mut session.dom(),
            DomSourceEdit::insert_before(subtree, parent, None),
            source_limits(),
            &mut DomSourceServices {
                meta: Some(envelope()),
                execution_scope: None,
                style_policy: &mut observation,
                url: None,
            },
            &cancel(),
        )
        .unwrap();
    let DomSourceDriveProgress::MetaSource(outcome) = result else {
        panic!("joined Meta must return before the later service");
    };
    let DomSourceProgress::Request(DomSourceRequest::Style(request)) = outcome.source() else {
        panic!("original later Style request");
    };
    let operation = request.source().operation();
    assert_eq!(calls.get(), 1, "only the actual first Style policy ran");
    let report = session
        .style_execution()
        .processed_sheet(0)
        .unwrap()
        .unwrap();
    assert!(
        !report.media_diagnostics().is_empty(),
        "real prior diagnostics exercise the admitted source collector"
    );
    assert!(report.media_diagnostics()[0].source().is_some());
    assert_eq!(
        session
            .style_execution()
            .snapshot(document)
            .unwrap()
            .unwrap()
            .sheets()
            .len(),
        1
    );
    assert_eq!(
        outcome.preferences()[0]
            .observation()
            .preferred_name()
            .code_units(),
        &[66]
    );
    assert_eq!(outcome.roots().len(), 1);
    let result = session
        .source_execution()
        .resume(
            &mut session.dom(),
            operation,
            source_limits(),
            &mut DomSourceServices {
                meta: Some(envelope()),
                execution_scope: None,
                style_policy: &mut observation,
                url: None,
            },
            &cancel(),
        )
        .unwrap();
    assert!(matches!(result, DomSourceDriveProgress::Complete(_)));
    assert_eq!(calls.get(), 2);
    assert!(is_disabled(&mut session, first));
    assert!(!is_disabled(&mut session, last));
}

#[test]
fn source_retained_bank_refusal_parks_the_original_edit_and_retry_keeps_spent_work() {
    let (mut session, document) = native_session();
    let subject = detached_meta(&mut session, document, "B".into());
    let parent = container(&session, document);
    let before = observe(&mut session, document);
    let styles = session.style_execution();
    let original = styles.original().unwrap();
    let (limits, bytes) = {
        let state = original.borrow();
        (
            state.limits,
            state.storage(None, &cancel(), state.limits.work).unwrap(),
        )
    };
    let mut tight = limits;
    tight.storage_bytes = bytes;
    session
        .style_execution()
        .set_limits(&session.dom(), tight, &cancel())
        .unwrap();
    let runtime_before = original.borrow().work.get();
    let error = session
        .source_execution()
        .apply(
            &mut session.dom(),
            DomSourceEdit::insert_before(subject, parent, None),
            source_limits(),
            &mut DomSourceServices {
                meta: Some(envelope()),
                execution_scope: None,
                style_policy: &mut |_| panic!("no published request"),
                url: None,
            },
            &cancel(),
        )
        .unwrap_err();
    assert!(matches!(
        error,
        crate::DomSourceDriveError::Meta(DomStyleMetaError::Runtime(
            DomStyleRuntimeError::Admission
        ))
    ));
    assert_eq!(session.snapshot().stamp(), before.stamp());
    assert!(session.dom().style_cssom_root(document).unwrap().is_none());
    let pending = session
        .dom()
        .pending_source_edit()
        .expect("bound policy retains the original unstarted edit");
    let operation = pending.operation();
    let source_work = pending.spent_work();
    assert!(source_work > 0);
    assert!(original.borrow().work.get() > runtime_before);
    {
        let state = original.borrow();
        let root = state
            .roots
            .iter()
            .find(|root| root.node == document)
            .unwrap();
        let current = root.sheet_sets.state().unwrap();
        assert_eq!(current.names().next().unwrap(), before.preferred_name());
    }
    let native = envelope();
    let lowered = surgeist_dom::DomMetaAtomicLimits::new(
        native.retained_bytes(),
        native.retained_units(),
        native.work() - 1,
        native.outputs(),
    )
    .unwrap();
    let lower_error = session
        .source_execution()
        .resume(
            &mut session.dom(),
            operation,
            source_limits(),
            &mut DomSourceServices {
                meta: Some(lowered),
                execution_scope: None,
                style_policy: &mut |_| panic!("lowering cannot reach a service"),
                url: None,
            },
            &cancel(),
        )
        .unwrap_err();
    assert!(matches!(
        lower_error,
        crate::DomSourceDriveError::Meta(DomStyleMetaError::Runtime(DomStyleRuntimeError::Dom(
            DomError::SourceLimitReduction
        )))
    ));
    let still_pending = session.dom().pending_source_edit().unwrap();
    assert_eq!(still_pending.operation(), operation);
    assert!(still_pending.spent_work() >= source_work);
    assert_eq!(session.snapshot().stamp(), before.stamp());
    let absent = session
        .source_execution()
        .resume(
            &mut session.dom(),
            operation,
            source_limits(),
            &mut DomSourceServices {
                meta: None,
                execution_scope: None,
                style_policy: &mut |_| panic!("absent native permission cannot consume Meta"),
                url: None,
            },
            &cancel(),
        )
        .unwrap_err();
    assert!(matches!(
        absent,
        crate::DomSourceDriveError::Source(DomSourceRuntimeError::Dom(DomError::Meta(
            surgeist_dom::DomMetaError::DefaultStyleUnsupported
        )))
    ));
    assert_eq!(session.snapshot().stamp(), before.stamp());
    assert_eq!(
        session.dom().pending_source_edit().unwrap().operation(),
        operation
    );
    session
        .style_execution()
        .set_limits(&session.dom(), limits, &cancel())
        .unwrap();
    let resumed = session
        .source_execution()
        .resume(
            &mut session.dom(),
            operation,
            source_limits(),
            &mut DomSourceServices {
                meta: Some(envelope()),
                execution_scope: None,
                style_policy: &mut |_| panic!("no Style request"),
                url: None,
            },
            &cancel(),
        )
        .unwrap();
    let DomSourceDriveProgress::MetaSource(outcome) = resumed else {
        panic!("same original edit completes");
    };
    let DomSourceProgress::Complete(done) = outcome.source() else {
        panic!("original completion");
    };
    assert_eq!(done.operation(), operation);
    assert_eq!(
        outcome.preferences()[0].observation().revision(),
        before.revision() + 1
    );
    assert!(original.borrow().work.get() > runtime_before);
    assert_eq!(
        observe(&mut session, document)
            .preferred_name()
            .code_units(),
        &[66]
    );
}

#[test]
fn native_installation_returns_the_original_mapping_with_its_meta_publication() {
    let (mut session, document) = native_session();
    let mut builder = surgeist_html::HtmlFragment::builder(
        surgeist_html::HtmlConstructionLimits::new(16, 16, 16, 4096, 16),
    )
    .unwrap();
    builder
        .append_element(
            &builder.root(),
            surgeist_html::QualifiedName::new(Namespace::html(), None, "meta"),
            surgeist_html::OrderedAttributes::new([
                surgeist_html::Attribute::new(
                    surgeist_html::QualifiedName::new(Namespace::none(), None, "http-equiv"),
                    "default-style",
                ),
                surgeist_html::Attribute::new(
                    surgeist_html::QualifiedName::new(Namespace::none(), None, "content"),
                    "B",
                ),
            ])
            .unwrap(),
        )
        .unwrap();
    let fragment = builder.finish();
    let input = fragment.root().children().next().unwrap();
    let before = session.snapshot();
    let parent = container(&session, document);
    let target = surgeist_dom::DomNativeHtmlInstallTarget::new(
        before
            .node(document)
            .unwrap()
            .as_document()
            .unwrap()
            .stamp(),
        parent,
        None,
    );
    let result = session
        .source_execution()
        .install_native_html(
            &mut session.dom(),
            target,
            &fragment,
            surgeist_dom::DomSourceNativeHtmlInstallLimits::new(
                surgeist_dom::DomNativeHtmlInstallLimits::new(16, 100_000, 1_000_000),
                source_limits(),
            ),
            &mut DomSourceServices {
                meta: Some(envelope()),
                execution_scope: None,
                style_policy: &mut |_| panic!("no Style service in this fragment"),
                url: None,
            },
            &cancel(),
        )
        .unwrap();
    let DomSourceDriveProgress::MetaSource(outcome) = result else {
        panic!("real native Meta output");
    };
    let DomSourceProgress::Complete(done) = outcome.source() else {
        panic!("original native installation completed");
    };
    let subject = done
        .native_installation()
        .unwrap()
        .node_for(&input)
        .unwrap();
    assert!(before.node(subject).is_err());
    assert_eq!(
        session.snapshot().node(subject).unwrap().parent(),
        Some(parent)
    );
    assert_eq!(outcome.preferences().len(), 1);
    assert_eq!(
        outcome.preferences()[0].observation().stamp(),
        done.commit().changes().to()
    );
    assert_eq!(
        observe(&mut session, document)
            .preferred_name()
            .code_units(),
        &[66]
    );
}
