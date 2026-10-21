//! Original controlled constructors retain their qualified designation.
//! Captured owner reuse preserves the existing canonical qualification.
use super::*;
use DomFocusedAreaDesignation::KnownNull;
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
fn focused_designation_controlled_history_and_native_inert_creation_retain_known_null() {
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
            KnownNull
        );
    }
    let captured = capture(before, outer);
    assert_eq!(
        captured
            .source()
            .document(inert)
            .unwrap()
            .focused_area_designation(),
        KnownNull
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
        KnownNull
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
        KnownNull
    );
    drop(dom);
    assert_eq!(
        captured
            .source()
            .document(outer)
            .unwrap()
            .focused_area_designation(),
        KnownNull
    );
}

#[test]
fn focused_designation_contextual_auxiliary_is_qualified_and_captured_owners_are_preserved() {
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
        let expected = KnownNull;
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
        assert_eq!(auxiliary.focused_area_designation(), KnownNull);
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
    batch.adopt(copy, parsed);
    dom.apply(batch, &AtomicBool::new(false)).unwrap();
    let after = dom.snapshot();
    assert_eq!(after.node(subtree).unwrap().owner_document(), Some(native));
    assert_eq!(after.node(copy).unwrap().owner_document(), Some(parsed));
    assert_eq!(
        after.node(same_owner_copy).unwrap().owner_document(),
        Some(parsed)
    );
    assert_eq!(
        after.document(parsed).unwrap().focused_area_designation(),
        KnownNull
    );
    assert_eq!(
        after.document(native).unwrap().focused_area_designation(),
        KnownNull
    );
    assert_eq!(
        before.document(parsed).unwrap().focused_area_designation(),
        KnownNull
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

#[test]
fn focus_constructor_original_fact_survives_report_release_and_independent_installations() {
    let report = parse(HtmlSource::utf8("<!doctype html><div tabindex=0></div>"));
    let weak = Arc::downgrade(&report);
    let (mut dom, environment) = setup();
    let before = dom.snapshot();
    let usage = dom.usage();
    assert!(matches!(
        dom.install_parsed_html_document(
            before.stamp(),
            report.clone(),
            environment.clone(),
            DomParsedHtmlInstallLimits::new(1024, 0, 16_000_000, 100_000_000),
            &AtomicBool::new(false),
        ),
        Err(DomParsedHtmlInstallError::ResourceLimit {
            resource: DomParsedHtmlInstallResource::Work,
            limit: 0,
        })
    ));
    assert_eq!(dom.snapshot().stamp(), before.stamp());
    assert_eq!(dom.usage(), usage);
    assert!(matches!(
        dom.install_parsed_html_document(
            before.stamp(),
            report.clone(),
            environment.clone(),
            limits(),
            &AtomicBool::new(true),
        ),
        Err(DomParsedHtmlInstallError::Cancelled)
    ));
    assert_eq!(dom.snapshot().stamp(), before.stamp());
    assert_eq!(dom.usage(), usage);
    let first = install(&mut dom, report.clone(), environment.clone());
    let first_document = first.document();
    let first_subject = element(&first, "div");
    let original = first.snapshot().clone();
    let second = install(&mut dom, report.clone(), environment);
    let second_document = second.document();
    let second_subject = element(&second, "div");
    assert_ne!(first_document, second_document);
    assert_ne!(first_subject, second_subject);
    assert!(matches!(
        original
            .document(first_document)
            .unwrap()
            .metadata()
            .creation_cause(),
        DomDocumentCreationCause::ParsedOffline { .. }
    ));
    for document in [first_document, second_document] {
        assert_eq!(
            dom.snapshot()
                .document(document)
                .unwrap()
                .focused_area_designation(),
            KnownNull
        );
    }
    drop(report);
    drop(first);
    drop(second);
    assert!(weak.upgrade().is_none());
    let mut batch = DomBatch::new();
    batch.detach(first_subject);
    dom.apply(batch, &AtomicBool::new(false)).unwrap();
    let current = dom.snapshot();
    assert_eq!(current.node(first_subject).unwrap().parent(), None);
    assert!(current.node(second_subject).unwrap().parent().is_some());
    assert!(original.node(first_subject).unwrap().parent().is_some());
    let capture = capture(&current, first_document);
    drop(dom);
    assert_eq!(
        original
            .document(first_document)
            .unwrap()
            .focused_area_designation(),
        KnownNull
    );
    assert_eq!(
        capture
            .source()
            .document(first_document)
            .unwrap()
            .focused_area_designation(),
        KnownNull
    );
}
