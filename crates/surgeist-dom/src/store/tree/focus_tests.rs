//! Unavailable removal is exercised at the actual unqualified private Document
//! constructor, without supplying or altering any original HTML/report fact.
use super::*;
use crate::DomFocusedAreaDesignation::Unavailable;
use crate::*;

fn unqualified_owner_candidate() -> (Dom, DocumentId, NodeId, NodeId) {
    unqualified_owner_with_meta(None)
}

fn unqualified_owner_with_meta(
    meta: Option<surgeist_html_parser::HtmlParseDocumentMeta<'_>>,
) -> (Dom, DocumentId, NodeId, NodeId) {
    let mut dom = Dom::new(
        DomLimits::new(
            1024,
            1_000_000,
            1024,
            4096,
            4096,
            32,
            1_000_000,
            DomNameLimits::new(16_777_216, 16_384, 4_000_000, 1_048_576).unwrap(),
        )
        .unwrap(),
    )
    .unwrap();
    let cancel = AtomicBool::new(false);
    let mut draft = Draft::new(&dom, Arc::new(BatchIdentity), &cancel).unwrap();
    // This is the existing actual unqualified ingress constructor. No HTML
    // creation fact is supplied, and no KnownNull field/report is overwritten.
    let mut metadata = DomDocumentMetadata::parsed_template_owner(
        DomDocumentClass::Html,
        DomNativeInputPlatform::headless(),
        false,
    );
    if let Some(meta) = meta {
        // Supply only independently observed Meta history. This raw ingress
        // does not receive the source Document's focused-area qualification.
        assert!(meta.first_missing().is_none());
        assert!(meta.csp_association().is_none());
        assert!(meta.color_scheme().is_none());
        let surgeist_html_parser::HtmlParseMetaRefresh::Known(refresh) = meta.refresh() else {
            panic!("the completed plain report supplies actual refresh history");
        };
        metadata.establish_parsed_meta_history(
            meta.pragma_language().cloned(),
            meta.referrer_policy(),
            refresh,
            meta.theme_color()
                .unwrap_or_else(|_| panic!("actual completed theme selection")),
            None,
        );
    }
    let key = Key::Created(draft.created.len());
    let key = draft
        .allocate(NewNode::Document(metadata), key, dom.limits(), &cancel)
        .unwrap();
    assert_eq!(
        draft.node(key).unwrap().focused_area_designation(),
        Unavailable
    );
    let (prepared, mapped) = draft
        .finish_with_mapping(&dom, &cancel, std::iter::once(key))
        .unwrap();
    let document = DocumentId(mapped[0]);
    drop(draft);
    let mut publication = dom.prepare_publication(prepared, &cancel).unwrap();
    publication.close_ordinary_sharing(&cancel).unwrap();
    dom.publish_prepared(publication);
    let mut batch = DomBatch::new();
    let parent = batch.create_element_ns(document, Namespace::html(), "body");
    let subject = batch.create_element_ns(document, Namespace::html(), "div");
    batch.set_attribute(&subject, "tabindex", "0");
    batch.insert_before(&subject, &parent, None);
    batch.insert_before(&parent, document, None);
    let committed = dom.apply(batch, &cancel).unwrap();
    let subject = committed.created(&subject).unwrap().unwrap();
    let parent = committed.created(&parent).unwrap().unwrap();
    let snapshot = dom.snapshot();
    assert_eq!(
        snapshot
            .document(document)
            .unwrap()
            .focused_area_designation(),
        Unavailable
    );
    assert_eq!(
        snapshot.node(subject).unwrap().owner_document(),
        Some(document)
    );
    assert_eq!(snapshot.node(subject).unwrap().parent(), Some(parent));
    assert!(dom.pending_parsed_html_associations().next().is_none());
    assert!(dom.pending_source_edit().is_none());
    (dom, document, subject, parent)
}

#[test]
fn unqualified_owner_element_atomic_detach_refuses() {
    let (mut dom, document, subject, parent) = unqualified_owner_candidate();
    let before = dom.snapshot();
    let mut batch = DomBatch::new();
    batch.detach(subject);
    assert_eq!(
        dom.apply(batch, &AtomicBool::new(false)).unwrap_err(),
        DomError::FocusedAreaDesignationUnavailable {
            subject: Some(subject),
            document: Some(document)
        }
    );
    let after = dom.snapshot();
    assert_eq!(after.stamp(), before.stamp());
    assert_eq!(after.node(subject).unwrap().parent(), Some(parent));
    assert_eq!(
        after.document(document).unwrap().focused_area_designation(),
        Unavailable
    );
}

#[test]
fn unqualified_owner_element_source_detach_refuses() {
    let (mut dom, document, subject, parent) = unqualified_owner_candidate();
    let before = dom.snapshot();
    let limits = DomSourceLimits::new(128_000_000, 1_000_000, 20_000_000, 64, 4096, 4096, 4);
    let mut lease = dom
        .begin_source_edit(
            DomSourceEdit::detach(subject),
            limits,
            &AtomicBool::new(false),
        )
        .expect("the actual unqualified owner has no competing Source or parsed obligation");
    assert!(!lease.status().has_published());
    let expected = DomError::FocusedAreaDesignationUnavailable {
        subject: Some(subject),
        document: Some(document),
    };
    assert_eq!(
        lease.advance(&AtomicBool::new(false)).unwrap_err(),
        expected
    );
    let spent = lease.status().spent_work();
    assert_eq!(
        lease.advance(&AtomicBool::new(false)).unwrap_err(),
        expected
    );
    assert!(lease.status().spent_work() > spent);
    assert!(!lease.status().has_published());
    let after = lease.snapshot();
    assert_eq!(after.stamp(), before.stamp());
    assert_eq!(after.node(subject).unwrap().parent(), Some(parent));
    assert_eq!(
        after.document(document).unwrap().focused_area_designation(),
        Unavailable
    );
    let operation = lease.operation();
    assert_eq!(
        lease.advance(&AtomicBool::new(true)).unwrap_err(),
        DomError::Cancelled
    );
    assert_eq!(lease.snapshot().stamp(), before.stamp());
    drop(lease);
    assert!(dom.pending_source_edit().is_none());
    assert_eq!(
        dom.resume_source_edit(operation, limits, &AtomicBool::new(false))
            .err(),
        Some(DomError::InvalidSourceOperation)
    );
    let mut retry = dom
        .begin_source_edit(
            DomSourceEdit::detach(subject),
            limits,
            &AtomicBool::new(false),
        )
        .unwrap();
    assert_ne!(retry.operation(), operation);
    assert_eq!(
        retry.advance(&AtomicBool::new(false)).unwrap_err(),
        expected
    );
    assert_eq!(retry.snapshot().stamp(), before.stamp());
}

#[test]
fn focus_removal_range_and_slot_prefix_roll_back_on_the_same_comparison() {
    let (mut dom, document, host, _) = unqualified_owner_candidate();
    dom.set_limits(dom.limits().with_ranges(2).unwrap())
        .unwrap();
    let cancel = AtomicBool::new(false);
    let mut batch = DomBatch::new();
    let shadow = batch.attach_shadow(host, DomShadowRootInit::new(HtmlShadowRootMode::Open));
    let slot = batch.create_element_ns(document, Namespace::html(), "slot");
    batch.insert_before(&slot, &shadow, None);
    let subject = batch.create_element_ns(document, Namespace::html(), "span");
    batch.insert_before(&subject, host, None);
    let range = batch.create_range(document);
    batch.set_range_start(&range, &subject, 0);
    batch.set_range_end(&range, &subject, 0);
    let commit = dom.apply(batch, &cancel).unwrap();
    let subject = commit.created(&subject).unwrap().unwrap();
    let slot = commit.created(&slot).unwrap().unwrap();
    let range = commit.created_range(&range).unwrap().unwrap();
    let before = dom.snapshot();
    let usage = dom.usage();
    assert_eq!(
        before
            .node(subject)
            .unwrap()
            .stored_assigned_slot()
            .unwrap(),
        Some(slot)
    );
    let mut batch = DomBatch::new();
    batch.detach(subject);
    assert_eq!(
        dom.apply(batch, &cancel).unwrap_err(),
        DomError::FocusedAreaDesignationUnavailable {
            subject: Some(subject),
            document: Some(document)
        }
    );
    let after = dom.snapshot();
    assert_eq!(after.stamp(), before.stamp());
    assert_eq!(dom.usage(), usage);
    assert_eq!(after.node(subject).unwrap().parent(), Some(host));
    assert_eq!(
        after.node(subject).unwrap().stored_assigned_slot().unwrap(),
        Some(slot)
    );
    assert_eq!(after.slot(slot).unwrap().assigned_nodes(), &[subject]);
    for snapshot in [&before, &after] {
        assert_eq!(snapshot.range(range).unwrap().start().container(), subject);
        assert_eq!(snapshot.range(range).unwrap().end().container(), subject);
        assert_eq!(snapshot.range(range).unwrap().start().offset(), 0);
    }
}

#[test]
fn focus_removal_unexposed_native_identity_is_negative_but_published_subject_still_refuses() {
    for namespace in [
        Namespace::html(),
        Namespace::svg(),
        Namespace::none(),
        Namespace::from_uri("urn:test"),
    ] {
        let (mut dom, document, parent, _) = unqualified_owner_candidate();
        let before = dom.snapshot();
        let mut batch = DomBatch::new();
        let subject = batch.create_element_ns(document, namespace.clone(), "candidate");
        batch.insert_before(&subject, parent, None);
        batch.detach(&subject);
        let commit = dom
            .apply(batch, &AtomicBool::new(false))
            .expect("a wholly unpublished fresh identity cannot equal a pre-existing designation");
        let unexposed = commit.created(&subject).unwrap().unwrap();
        let after = dom.snapshot();
        assert_eq!(after.node(unexposed).unwrap().parent(), None);
        assert_eq!(
            after.node(unexposed).unwrap().owner_document(),
            Some(document)
        );
        assert_eq!(
            after.document(document).unwrap().focused_area_designation(),
            Unavailable
        );
        assert_eq!(
            after.node(parent).unwrap().children(),
            before.node(parent).unwrap().children()
        );
        let mut batch = DomBatch::new();
        let subject = batch.create_element_ns(document, namespace, "candidate");
        batch.insert_before(&subject, parent, None);
        let subject = dom
            .apply(batch, &AtomicBool::new(false))
            .unwrap()
            .created(&subject)
            .unwrap()
            .unwrap();
        let before = dom.snapshot();
        let mut batch = DomBatch::new();
        batch.detach(subject);
        assert_eq!(
            dom.apply(batch, &AtomicBool::new(false)).unwrap_err(),
            DomError::FocusedAreaDesignationUnavailable {
                subject: Some(subject),
                document: Some(document)
            }
        );
        assert_eq!(dom.snapshot().stamp(), before.stamp());
    }
}

#[test]
fn focus_removal_adoption_replacement_replace_all_and_destruction_share_the_guard() {
    let cancel = AtomicBool::new(false);
    for operation in 0..4 {
        let (mut dom, document, original, parent) = unqualified_owner_candidate();
        let mut setup = DomBatch::new();
        let other = setup.create_document();
        let fragment = setup.create_fragment(document);
        let child = setup.create_element_ns(document, Namespace::html(), "span");
        setup.insert_before(&child, &fragment, None);
        let output = setup.create_element_ns(document, Namespace::html(), "output");
        let output_child = setup.create_element_ns(document, Namespace::html(), "span");
        setup.insert_before(&output_child, &output, None);
        let commit = dom.apply(setup, &cancel).unwrap();
        let other = commit.document(&other).unwrap();
        let fragment = commit.created(&fragment).unwrap().unwrap();
        let child = commit.created(&child).unwrap().unwrap();
        let output = commit.created(&output).unwrap().unwrap();
        let output_child = commit.created(&output_child).unwrap().unwrap();
        let before = dom.snapshot();
        let mut batch = DomBatch::new();
        let subject = match operation {
            0 => {
                batch.adopt(original, other);
                original
            }
            1 => {
                let text = batch.create_text(document, "replacement");
                batch.replace_child(&text, parent, original);
                original
            }
            2 => {
                batch.set_output_value(output, "replacement");
                output_child
            }
            _ => {
                batch.destroy(fragment);
                child
            }
        };
        assert_eq!(
            dom.apply(batch, &cancel).unwrap_err(),
            DomError::FocusedAreaDesignationUnavailable {
                subject: Some(subject),
                document: Some(document)
            }
        );
        assert_eq!(dom.snapshot().stamp(), before.stamp());
        assert_eq!(
            dom.snapshot().node(original).unwrap().parent(),
            Some(parent)
        );
        assert_eq!(
            dom.snapshot().node(original).unwrap().owner_document(),
            Some(document)
        );
        assert_eq!(dom.snapshot().node(child).unwrap().parent(), Some(fragment));
        assert_eq!(
            dom.snapshot().node(output_child).unwrap().parent(),
            Some(output)
        );
    }
}

#[test]
fn focus_removal_intrinsic_nonareas_and_detached_elements_do_not_need_unknown_history() {
    let (mut dom, document, parent, _) = unqualified_owner_candidate();
    let cancel = AtomicBool::new(false);
    let mut batch = DomBatch::new();
    let text = batch.create_text(document, "text");
    let comment = batch.create_comment(document, "comment");
    batch.insert_before(&text, parent, None);
    batch.insert_before(&comment, parent, None);
    let detached = batch.create_element_ns(document, Namespace::html(), "div");
    let commit = dom.apply(batch, &cancel).unwrap();
    let text = commit.created(&text).unwrap().unwrap();
    let comment = commit.created(&comment).unwrap().unwrap();
    let detached = commit.created(&detached).unwrap().unwrap();
    for subject in [text, comment] {
        let mut batch = DomBatch::new();
        batch.detach(subject);
        dom.apply(batch, &cancel).unwrap();
        assert_eq!(dom.snapshot().node(subject).unwrap().parent(), None);
    }
    let before = dom.snapshot();
    let mut batch = DomBatch::new();
    batch.detach(detached);
    assert!(
        dom.apply(batch, &cancel)
            .unwrap()
            .changes()
            .changes()
            .is_empty()
    );
    assert_eq!(dom.snapshot().stamp(), before.stamp());
    let mut lease = dom
        .begin_source_edit(
            DomSourceEdit::detach(detached),
            DomSourceLimits::new(128_000_000, 1_000_000, 20_000_000, 64, 4096, 4096, 4),
            &cancel,
        )
        .unwrap();
    assert!(matches!(
        lease.advance(&cancel).unwrap(),
        DomSourceProgress::Complete(_)
    ));
    assert_eq!(lease.snapshot().stamp(), before.stamp());
}

#[test]
fn focus_removal_shadow_including_published_descendant_is_checked_after_unexposed_host() {
    let (mut dom, document, _, parent) = unqualified_owner_candidate();
    let cancel = AtomicBool::new(false);
    let mut setup = DomBatch::new();
    let descendant = setup.create_element_ns(document, Namespace::html(), "span");
    setup.set_attribute(&descendant, "tabindex", "0");
    let descendant = dom
        .apply(setup, &cancel)
        .unwrap()
        .created(&descendant)
        .unwrap()
        .unwrap();
    let before = dom.snapshot();
    let usage = dom.usage();
    assert_eq!(before.node(descendant).unwrap().parent(), None);
    assert_eq!(
        before
            .document(document)
            .unwrap()
            .focused_area_designation(),
        Unavailable
    );
    let mut batch = DomBatch::new();
    let host = batch.create_element_ns(document, Namespace::html(), "div");
    let shadow = batch.attach_shadow(&host, DomShadowRootInit::new(HtmlShadowRootMode::Open));
    batch.insert_before(&host, parent, None);
    batch.insert_before(descendant, &shadow, None);
    batch.detach(&host);
    assert_eq!(
        dom.apply(batch, &cancel).unwrap_err(),
        DomError::FocusedAreaDesignationUnavailable {
            subject: Some(descendant),
            document: Some(document)
        }
    );
    let after = dom.snapshot();
    assert_eq!(after.stamp(), before.stamp());
    assert_eq!(dom.usage(), usage);
    assert_eq!(after.node(descendant).unwrap().parent(), None);
    assert_eq!(
        after.node(descendant).unwrap().owner_document(),
        Some(document)
    );
    assert_eq!(
        after.node(parent).unwrap().children(),
        before.node(parent).unwrap().children()
    );
    assert_eq!(after.nodes().count(), before.nodes().count());
}

#[test]
fn contextual_installation_preserves_unqualified_owner_and_qualifies_only_new_auxiliary() {
    use surgeist_html_parser::{
        HtmlDocumentParseRequest, HtmlFragmentParseRequest, HtmlParseLimits, HtmlParseMetadataStep,
        HtmlSource,
    };
    use surgeist_html_semantics::{HtmlCaptureLimits, HtmlFragmentContext};
    let parse_limits = || {
        HtmlParseLimits::builder()
            .source_units(4096)
            .buffered_utf16_units(4096)
            .token_attributes(256)
            .auxiliary_entries(100_000)
            .managed_nodes(256)
            .managed_edges(512)
            .managed_attributes(512)
            .managed_utf16_units(4096)
            .source_map_entries(4096)
            .source_map_payload_units(4096)
            .diagnostics(512)
            .diagnostic_payload_units(4096)
            .context_effects(256)
            .open_elements(64)
            .tree_depth(256)
            .managed_capacity_bytes(16_000_000)
            .peak_transient_bytes(16_000_000)
            .atomic_layout_bytes(16_000_000)
            .work_credits(10_000_000)
            .finish()
            .unwrap()
    };
    let report =
        HtmlDocumentParseRequest::offline(HtmlSource::utf8("<!doctype html>"), parse_limits())
            .parse()
            .unwrap();
    let original = report.document().unwrap().root().id();
    let mut cursor = report.metadata();
    let meta = loop {
        match cursor.advance() {
            HtmlParseMetadataStep::Fault(fault) => panic!("{fault}"),
            HtmlParseMetadataStep::Record(row) => {
                if let Some(meta) = row.document_meta()
                    && meta.document().id() == original
                {
                    break meta;
                }
            }
            HtmlParseMetadataStep::Skipped => {}
            HtmlParseMetadataStep::Complete => panic!("actual original Document Meta history"),
        }
    };
    // This is deliberately the actual private unqualified ingress, supplied
    // only separate authentic Meta history. It is not a full installation of
    // the ordinary report and cannot acquire its constructor focus proof.
    let (mut dom, document, context_element, _) = unqualified_owner_with_meta(Some(meta));
    let before = dom.snapshot();
    assert_eq!(
        before
            .document(document)
            .unwrap()
            .focused_area_designation(),
        Unavailable
    );
    let capture = before
        .html_capture(
            document,
            None,
            HtmlCaptureLimits::new(1024, 4096, 4096, 1_000_000, 1024, 0, 0, 16_777_216),
            DomHtmlCaptureLimits::new(1024, 1_000_000),
            &AtomicBool::new(false),
        )
        .unwrap();
    let context = HtmlFragmentContext::element(
        capture.capture(),
        &capture.subject(context_element).unwrap(),
    )
    .unwrap();
    let fragment = Arc::new(
        // The ignored Style retains its authentic parser-Document association
        // without requiring executable Style processing.
        HtmlFragmentParseRequest::new(
            HtmlSource::utf8("<style type=text/plain></style>"),
            context,
            parse_limits(),
        )
        .unwrap()
        .parse()
        .unwrap(),
    );
    let original_auxiliary = fragment
        .fragment_parser_document()
        .unwrap()
        .unwrap()
        .document()
        .id();
    let installed = dom
        .install_parsed_html_fragment(
            fragment,
            &capture,
            DomParsedHtmlInstallLimits::new(1024, 10_000_000, 16_000_000, 100_000_000),
            &AtomicBool::new(false),
        )
        .unwrap();
    assert_eq!(installed.document(), document);
    assert_eq!(
        installed
            .snapshot()
            .document(document)
            .unwrap()
            .focused_area_designation(),
        Unavailable
    );
    assert_eq!(
        capture
            .source()
            .document(document)
            .unwrap()
            .focused_area_designation(),
        Unavailable
    );
    let auxiliary = DocumentId(installed.node_for(&original_auxiliary).unwrap());
    assert_ne!(auxiliary, document);
    let auxiliary = installed.snapshot().document(auxiliary).unwrap();
    assert_eq!(
        auxiliary.focused_area_designation(),
        crate::DomFocusedAreaDesignation::KnownNull
    );
    assert_eq!(
        auxiliary.metadata().creation_cause(),
        &DomDocumentCreationCause::ParsedFragmentParser
    );
    assert_eq!(
        installed
            .snapshot()
            .document(document)
            .unwrap()
            .metadata()
            .creation_cause(),
        &DomDocumentCreationCause::ParsedTemplateOwner
    );
    assert_eq!(
        before
            .document(document)
            .unwrap()
            .focused_area_designation(),
        Unavailable
    );
}
