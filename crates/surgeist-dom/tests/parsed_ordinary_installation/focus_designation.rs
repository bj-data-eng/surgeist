//! The parser supplies no focused-designation history. Actual owner reuse must
//! retain its existing qualification; it is not a fresh native constructor.
use super::*;
use DomFocusedAreaDesignation::{KnownNull, Unavailable};
use surgeist_html_parser::HtmlFragmentParseRequest;
use surgeist_html_semantics::{HtmlCaptureLimits, HtmlFragmentContext};

fn capture(snapshot: &DomSnapshot, document: DocumentId) -> DomHtmlCapture {
    snapshot
        .html_capture(
            document,
            None,
            HtmlCaptureLimits::new(1024, 4096, 4096, 1_000_000, 1024, 0, 0, 16_777_216),
            DomHtmlCaptureLimits::new(1024, 1_000_000),
            &AtomicBool::new(false),
        )
        .unwrap()
}

#[test]
fn focused_designation_parsed_history_is_unavailable_and_native_inert_creation_is_fresh() {
    let (mut dom, environment) = setup();
    let installed = install(
        &mut dom,
        parse(HtmlSource::utf8(
            "<!doctype html><template><b>x</b></template>",
        )),
        environment,
    );
    let outer = installed.document();
    let before = installed.snapshot();
    let template = element(&installed, "template");
    let contents = before
        .node(template)
        .unwrap()
        .as_template()
        .unwrap()
        .contents()
        .unwrap();
    let inert = before.node(contents).unwrap().owner_document().unwrap();
    assert_ne!(outer, inert);
    for document in [outer, inert] {
        assert_eq!(
            before
                .document(document)
                .unwrap()
                .focused_area_designation(),
            Unavailable
        );
    }
    let captured = capture(before, outer);
    assert_eq!(
        captured
            .source()
            .document(inert)
            .unwrap()
            .focused_area_designation(),
        Unavailable
    );
    // Creating another Template reuses the authentic parsed inert owner.
    let mut batch = DomBatch::new();
    let reused = batch.create_element_ns(outer, Namespace::html(), "template");
    let commit = dom.apply(batch, &AtomicBool::new(false)).unwrap();
    let current = dom.snapshot();
    let reused = commit.created(&reused).unwrap().unwrap();
    let fragment = current
        .node(reused)
        .unwrap()
        .as_template()
        .unwrap()
        .contents()
        .unwrap();
    assert_eq!(
        current.node(fragment).unwrap().owner_document(),
        Some(inert)
    );
    assert_eq!(
        current.document(inert).unwrap().focused_area_designation(),
        Unavailable
    );

    // A parsed Document without an inert cache gets an actual fresh native
    // inert owner when its first native Template is later constructed.
    let (mut fresh, environment) = setup();
    let parsed = install(
        &mut fresh,
        parse(HtmlSource::utf8("<!doctype html><p>x</p>")),
        environment,
    );
    let document = parsed.document();
    let mut batch = DomBatch::new();
    let native = batch.create_element_ns(document, Namespace::html(), "template");
    let commit = fresh.apply(batch, &AtomicBool::new(false)).unwrap();
    let snapshot = fresh.snapshot();
    let contents = snapshot
        .node(commit.created(&native).unwrap().unwrap())
        .unwrap()
        .as_template()
        .unwrap()
        .contents()
        .unwrap();
    let native_inert = snapshot.node(contents).unwrap().owner_document().unwrap();
    assert_eq!(
        snapshot
            .document(native_inert)
            .unwrap()
            .focused_area_designation(),
        KnownNull
    );
    assert_eq!(
        snapshot
            .document(document)
            .unwrap()
            .focused_area_designation(),
        Unavailable
    );
    drop(dom);
    assert_eq!(
        captured
            .source()
            .document(outer)
            .unwrap()
            .focused_area_designation(),
        Unavailable
    );
}

#[test]
fn focused_designation_contextual_auxiliary_is_unknown_and_captured_owners_are_preserved() {
    for parsed_owner in [false, true] {
        let (mut dom, environment) = setup();
        let (document, template) = if parsed_owner {
            let installed = install(
                &mut dom,
                parse(HtmlSource::utf8("<template></template>")),
                environment,
            );
            (installed.document(), element(&installed, "template"))
        } else {
            let mut batch = DomBatch::new();
            let document = batch.create_document();
            let template = batch.create_element_ns(&document, Namespace::html(), "template");
            batch.insert_before(&template, &document, None);
            let commit = dom.apply(batch, &AtomicBool::new(false)).unwrap();
            (
                commit.document(&document).unwrap(),
                commit.created(&template).unwrap().unwrap(),
            )
        };
        let expected = if parsed_owner { Unavailable } else { KnownNull };
        let before = dom.snapshot();
        let contents = before
            .node(template)
            .unwrap()
            .as_template()
            .unwrap()
            .contents()
            .unwrap();
        let inert = before.node(contents).unwrap().owner_document().unwrap();
        let captured = capture(&before, document);
        let context = HtmlFragmentContext::hosted_fragment(
            captured.capture(),
            &captured.subject(contents).unwrap(),
        )
        .unwrap();
        let report = Arc::new(
            HtmlFragmentParseRequest::with_options(
                HtmlSource::utf8("<style type=text/plain></style><template><b>x</b></template>"),
                context,
                parse_limits(),
                surgeist_html::HtmlParserScriptingMode::Inert,
                false,
            )
            .unwrap()
            .parse()
            .unwrap(),
        );
        assert!(report.effects().unwrap().is_empty());
        let auxiliary = report.fragment_parser_document().unwrap().document().id();
        let installed = dom
            .install_parsed_html_fragment(report, &captured, limits(), &AtomicBool::new(false))
            .unwrap();
        assert_eq!(installed.document(), inert);
        for owner in [document, inert] {
            assert_eq!(
                installed
                    .snapshot()
                    .document(owner)
                    .unwrap()
                    .focused_area_designation(),
                expected
            );
            assert_eq!(
                captured
                    .source()
                    .document(owner)
                    .unwrap()
                    .focused_area_designation(),
                expected
            );
        }
        let auxiliary = installed
            .snapshot()
            .node(installed.node_for(&auxiliary).unwrap())
            .unwrap()
            .as_document()
            .unwrap();
        assert_eq!(auxiliary.focused_area_designation(), Unavailable);
    }
}

#[test]
fn focused_designation_subtree_transfer_preserves_documents_and_cannot_clone_document_history() {
    let (mut dom, environment) = setup();
    let installed = install(
        &mut dom,
        parse(HtmlSource::utf8("<div><span>x</span></div>")),
        environment,
    );
    let parsed = installed.document();
    let subtree = element(&installed, "div");
    let before = dom.snapshot();
    let mut batch = DomBatch::new();
    let native = batch.create_document();
    let copy = batch.import_node(&before, subtree, &native, true);
    let same_owner_copy = batch.import_node(&before, subtree, parsed, true);
    let commit = dom.apply(batch, &AtomicBool::new(false)).unwrap();
    let native = commit.document(&native).unwrap();
    let copy = commit.created(&copy).unwrap().unwrap();
    let same_owner_copy = commit.created(&same_owner_copy).unwrap().unwrap();
    let mut batch = DomBatch::new();
    batch.adopt(subtree, native);
    assert_eq!(
        dom.apply(batch, &AtomicBool::new(false)).unwrap_err(),
        DomError::FocusedAreaDesignationUnavailable {
            subject: Some(subtree),
            document: Some(parsed)
        }
    );
    let mut batch = DomBatch::new();
    batch.adopt(copy, parsed);
    dom.apply(batch, &AtomicBool::new(false)).unwrap();
    let after = dom.snapshot();
    assert_eq!(after.node(subtree).unwrap().owner_document(), Some(parsed));
    assert_eq!(after.node(copy).unwrap().owner_document(), Some(parsed));
    assert_eq!(
        after.node(same_owner_copy).unwrap().owner_document(),
        Some(parsed)
    );
    assert_eq!(
        after.document(parsed).unwrap().focused_area_designation(),
        Unavailable
    );
    assert_eq!(
        after.document(native).unwrap().focused_area_designation(),
        KnownNull
    );
    assert_eq!(
        before.document(parsed).unwrap().focused_area_designation(),
        Unavailable
    );
    for adopt in [false, true] {
        let mut batch = DomBatch::new();
        if adopt {
            batch.adopt(parsed, native);
        } else {
            batch.import_node(&before, parsed.node(), native, true);
        }
        assert_eq!(
            dom.apply(batch, &AtomicBool::new(false)).unwrap_err(),
            DomError::NotSupported
        );
        assert_eq!(dom.snapshot().stamp(), after.stamp());
    }
}

fn attached_original_focus_candidate() -> (Dom, DocumentId, NodeId, NodeId) {
    use surgeist_html_parser::{
        HtmlParseAssociationCaptureDisposition, HtmlParsePendingAssociationStep,
    };
    let report = parse(HtmlSource::utf8("<!doctype html><div tabindex=0></div>"));
    let mut cursor = report.pending_associations();
    let mut documents = 0;
    loop {
        match cursor.advance() {
            HtmlParsePendingAssociationStep::Document(row) => {
                documents += 1;
                assert_eq!(
                    row.disposition(),
                    HtmlParseAssociationCaptureDisposition::PendingComplete
                );
                assert!(!row.has_obligations());
            }
            HtmlParsePendingAssociationStep::Occurrence(row) => {
                assert!(!row.requires_processing());
            }
            HtmlParsePendingAssociationStep::Skipped => {}
            HtmlParsePendingAssociationStep::Complete => break,
        }
    }
    assert!(documents > 0);
    let (mut dom, environment) = setup();
    let installed = install(&mut dom, report, environment);
    let document = installed.document();
    let subject = element(&installed, "div");
    let parent = element(&installed, "body");
    let snapshot = installed.snapshot();
    assert_eq!(snapshot.node(subject).unwrap().parent(), Some(parent));
    assert_eq!(
        snapshot.node(subject).unwrap().owner_document(),
        Some(document)
    );
    assert_eq!(
        snapshot
            .node(subject)
            .unwrap()
            .as_element()
            .unwrap()
            .attribute(&"tabindex".into()),
        Some(&"0".into())
    );
    assert_eq!(
        snapshot
            .document(document)
            .unwrap()
            .focused_area_designation(),
        Unavailable
    );
    assert!(installed.pending_association().is_none());
    assert_eq!(dom.pending_parsed_html_association(document).unwrap(), None);
    assert!(dom.pending_parsed_html_associations().next().is_none());
    assert!(dom.pending_source_edit().is_none());
    (dom, document, subject, parent)
}

#[test]
fn focus_removal_unavailable_original_element_atomic_detach_refuses() {
    let (mut dom, document, subject, parent) = attached_original_focus_candidate();
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
fn focus_removal_unavailable_original_element_source_detach_refuses() {
    let (mut dom, document, subject, parent) = attached_original_focus_candidate();
    let before = dom.snapshot();
    let limits = DomSourceLimits::new(128_000_000, 1_000_000, 20_000_000, 64, 4096, 4096, 4);
    let mut lease = dom
        .begin_source_edit(
            DomSourceEdit::detach(subject),
            limits,
            &AtomicBool::new(false),
        )
        .expect("the authentic plain Element has no competing Source or parsed obligation");
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
    let (mut dom, document, host, _) = attached_original_focus_candidate();
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
        let (mut dom, document, parent, _) = attached_original_focus_candidate();
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
        let (mut dom, document, original, parent) = attached_original_focus_candidate();
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
    let (mut dom, document, parent, _) = attached_original_focus_candidate();
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
fn focus_removal_known_null_native_source_completes_without_window_or_focus_effects() {
    let (mut dom, _) = setup();
    let cancel = AtomicBool::new(false);
    let mut batch = DomBatch::new();
    let document = batch.create_document();
    let parent = batch.create_element_ns(&document, Namespace::svg(), "svg");
    let child = batch.create_element_ns(&document, Namespace::svg(), "g");
    batch.insert_before(&parent, &document, None);
    batch.insert_before(&child, &parent, None);
    let commit = dom.apply(batch, &cancel).unwrap();
    let document = commit.document(&document).unwrap();
    let parent = commit.created(&parent).unwrap().unwrap();
    let child = commit.created(&child).unwrap().unwrap();
    let before = dom.snapshot();
    assert_eq!(
        before
            .document(document)
            .unwrap()
            .focused_area_designation(),
        KnownNull
    );
    assert!(
        !before
            .document(document)
            .unwrap()
            .is_fully_active()
            .unwrap()
    );
    let mut lease = dom
        .begin_source_edit(
            DomSourceEdit::detach(child),
            DomSourceLimits::new(128_000_000, 1_000_000, 20_000_000, 64, 4096, 4096, 4),
            &cancel,
        )
        .unwrap();
    let DomSourceProgress::Complete(done) = lease.advance(&cancel).unwrap() else {
        panic!("known-null comparison needs no service");
    };
    assert_eq!(done.output(), None);
    assert_eq!(lease.snapshot().node(child).unwrap().parent(), None);
    assert_eq!(lease.snapshot().node(parent).unwrap().children(), &[]);
    assert_eq!(before.node(child).unwrap().parent(), Some(parent));
    assert_eq!(
        lease
            .snapshot()
            .document(document)
            .unwrap()
            .focused_area_designation(),
        KnownNull
    );
}

#[test]
fn focus_removal_shadow_including_published_descendant_is_checked_after_unexposed_host() {
    let (mut dom, document, _, parent) = attached_original_focus_candidate();
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

fn controlled_original_focus_candidate() -> (Dom, DocumentId, NodeId, NodeId) {
    use surgeist_html_parser::{
        HtmlParseAssociationCaptureDisposition, HtmlParsePendingAssociationStep,
    };
    let report = parse(HtmlSource::utf8("<!doctype html><div tabindex=0></div>"));
    let mut cursor = report.pending_associations();
    let mut documents = 0;
    loop {
        match cursor.advance() {
            HtmlParsePendingAssociationStep::Document(row) => {
                documents += 1;
                assert_eq!(
                    row.disposition(),
                    HtmlParseAssociationCaptureDisposition::PendingComplete
                );
                assert!(!row.has_obligations());
            }
            HtmlParsePendingAssociationStep::Occurrence(row) => {
                assert!(!row.requires_processing());
            }
            HtmlParsePendingAssociationStep::Skipped => {}
            HtmlParsePendingAssociationStep::Complete => break,
        }
    }
    assert!(documents > 0);
    let (mut dom, environment) = setup();
    let installed = install(&mut dom, report, environment);
    let document = installed.document();
    let subject = element(&installed, "div");
    let parent = element(&installed, "body");
    let snapshot = installed.snapshot();
    assert_eq!(snapshot.node(subject).unwrap().parent(), Some(parent));
    assert_eq!(
        snapshot.node(subject).unwrap().owner_document(),
        Some(document)
    );
    assert_eq!(
        snapshot
            .node(subject)
            .unwrap()
            .as_element()
            .unwrap()
            .attribute(&"tabindex".into()),
        Some(&"0".into())
    );
    assert!(installed.pending_association().is_none());
    assert_eq!(dom.pending_parsed_html_association(document).unwrap(), None);
    assert!(dom.pending_parsed_html_associations().next().is_none());
    assert!(dom.pending_source_edit().is_none());
    (dom, document, subject, parent)
}

#[test]
fn focus_constructor_controlled_original_document_retains_known_null() {
    let (dom, document, _, _) = controlled_original_focus_candidate();
    assert_eq!(
        dom.snapshot()
            .document(document)
            .unwrap()
            .focused_area_designation(),
        KnownNull
    );
}

#[test]
fn focus_constructor_controlled_original_element_atomic_removal_succeeds() {
    let (mut dom, document, subject, parent) = controlled_original_focus_candidate();
    let before = dom.snapshot();
    let mut batch = DomBatch::new();
    batch.detach(subject);
    dom.apply(batch, &AtomicBool::new(false))
        .expect("the original controlled constructor supplies a definite null comparison");
    let after = dom.snapshot();
    assert_eq!(after.node(subject).unwrap().parent(), None);
    assert_eq!(
        after.node(subject).unwrap().owner_document(),
        Some(document)
    );
    assert_eq!(before.node(subject).unwrap().parent(), Some(parent));
    assert_eq!(
        after.document(document).unwrap().focused_area_designation(),
        KnownNull
    );
}

#[test]
fn focus_constructor_controlled_original_element_source_removal_completes() {
    let (mut dom, document, subject, parent) = controlled_original_focus_candidate();
    let before = dom.snapshot();
    let cancel = AtomicBool::new(false);
    let mut lease = dom
        .begin_source_edit(
            DomSourceEdit::detach(subject),
            DomSourceLimits::new(128_000_000, 1_000_000, 20_000_000, 64, 4096, 4096, 4),
            &cancel,
        )
        .unwrap();
    let DomSourceProgress::Complete(done) = lease
        .advance(&cancel)
        .expect("the original controlled constructor supplies a definite null comparison")
    else {
        panic!("a known-null removal needs no external focus service");
    };
    assert_eq!(done.output(), None);
    let after = lease.snapshot();
    assert_eq!(after.node(subject).unwrap().parent(), None);
    assert_eq!(
        after.node(subject).unwrap().owner_document(),
        Some(document)
    );
    assert_eq!(before.node(subject).unwrap().parent(), Some(parent));
    assert_eq!(
        after.document(document).unwrap().focused_area_designation(),
        KnownNull
    );
}
