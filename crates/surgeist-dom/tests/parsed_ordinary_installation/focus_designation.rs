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
