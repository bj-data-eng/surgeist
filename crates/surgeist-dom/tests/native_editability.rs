//! Canonical Document design-mode facts in HTML editability queries.
//! HTML877d html-editability lines121,151: fresh Document design mode is false;
//! this direct HTML child has no explicit contenteditable host state.
use std::sync::atomic::AtomicBool;

use surgeist_dom::{Dom, DomBatch, DomHtmlCaptureLimits, DomLimits, Namespace};
use surgeist_html::HtmlValueInterpreter;
use surgeist_html_semantics::{
    HtmlCaptureLimits, HtmlSemanticCancellation, HtmlSemanticFact, HtmlSemanticLimits,
    HtmlSemanticQuery, HtmlSemanticRequest, HtmlSemanticValue,
};

#[test]
fn native_fresh_document_design_mode_disables_implicit_editing_host() {
    let cancel = AtomicBool::new(false);
    let mut dom = Dom::new(
        DomLimits::new(
            8,
            4096,
            16,
            0,
            0,
            8,
            65536,
            surgeist_dom::DomNameLimits::new(16_777_216, 16_384, 4_000_000, 1_048_576).unwrap(),
        )
        .unwrap(),
    )
    .unwrap();
    let mut batch = DomBatch::new();
    let document = batch.create_document();
    let root = batch.create_element_ns(&document, Namespace::html(), "div");
    batch.insert_before(&root, &document, None);
    let commit = dom.apply(batch, &cancel).unwrap();
    let document = commit.document(&document).unwrap();
    let root = commit.created(&root).unwrap().unwrap();
    let snapshot = dom.snapshot();
    assert_eq!(snapshot.node(root).unwrap().parent(), Some(document.node()));
    assert!(
        snapshot
            .node(root)
            .unwrap()
            .as_element()
            .unwrap()
            .attributes()
            .is_empty()
    );
    let capture = snapshot
        .html_capture(
            document,
            None,
            HtmlCaptureLimits::new(8, 64, 16, 4096, 8, 0, 0, 16_777_216),
            DomHtmlCaptureLimits::new(8, 10000),
            &cancel,
        )
        .unwrap();
    let report = capture
        .query(
            &[HtmlSemanticRequest::new(
                capture.subject(root).unwrap(),
                HtmlSemanticQuery::EditingHost,
            )],
            HtmlValueInterpreter::new(1024),
            HtmlSemanticLimits::new(100000, 16, 1024, 64, 1000000, 1024),
            HtmlSemanticCancellation::new(),
        )
        .unwrap();
    assert!(
        matches!(
            report.results()[0].fact(),
            HtmlSemanticFact::EditingHost(HtmlSemanticValue::Known(false))
        ),
        "fresh native Document design mode is disabled; got {:?}",
        report.results()[0].fact()
    );
}

use surgeist_dom::{DocumentId, DomHtmlCapture, DomSnapshot, NodeId};
use surgeist_html_semantics::HtmlSemanticMissing;

fn owner() -> Dom {
    Dom::new(
        DomLimits::new(
            256,
            100_000,
            1024,
            1024,
            1024,
            16,
            1_000_000,
            surgeist_dom::DomNameLimits::new(16_777_216, 16_384, 4_000_000, 1_048_576).unwrap(),
        )
        .unwrap(),
    )
    .unwrap()
}
fn capture(snapshot: &DomSnapshot, document: DocumentId) -> DomHtmlCapture {
    snapshot
        .html_capture(
            document,
            None,
            HtmlCaptureLimits::new(512, 8192, 1024, 200_000, 512, 0, 0, 16_777_216),
            DomHtmlCaptureLimits::new(512, 1_000_000),
            &AtomicBool::new(false),
        )
        .unwrap()
}
fn fact(capture: &DomHtmlCapture, node: NodeId, query: HtmlSemanticQuery) -> HtmlSemanticFact {
    let subject = capture.subject(node).unwrap();
    assert_eq!(capture.node(&subject).unwrap(), node);
    capture
        .query(
            &[HtmlSemanticRequest::new(subject, query)],
            HtmlValueInterpreter::new(8192),
            HtmlSemanticLimits::new(1_000_000, 16, 8192, 1024, 1_000_000, 8192),
            HtmlSemanticCancellation::new(),
        )
        .unwrap()
        .results()[0]
        .fact()
        .clone()
}
fn with_slots(capture: DomHtmlCapture) -> DomHtmlCapture {
    capture
        .with_slot_semantics(
            surgeist_dom::DomSlotSemanticLimits::new(
                surgeist_dom::DomSlotQueryLimits::new(512, 4096).unwrap(),
                DomHtmlCaptureLimits::new(512, 1_000_000),
                surgeist_html_semantics::HtmlLanguageDirectionLimits::new(
                    512, 512, 200_000, 1_000_000,
                ),
                surgeist_html_semantics::HtmlInertFactLimits::new(512, 1_000_000),
            ),
            &AtomicBool::new(false),
        )
        .unwrap()
}

#[test]
fn every_native_document_creator_establishes_its_own_disabled_state_and_snapshots_retain_it() {
    let mut dom = owner();
    let cancel = AtomicBool::new(false);
    let mut b = DomBatch::new();
    let native = b.create_document();
    let html = b.create_html_document(&native, None);
    let xml = b.create_xml_document(&html, Namespace::html(), "root", None);
    let template = b.create_element_ns(&native, Namespace::html(), "template");
    let done = dom.apply(b, &cancel).unwrap();
    let native = done.document(&native).unwrap();
    let html = done.document(&html).unwrap();
    let xml = done.document(&xml).unwrap();
    let old = dom.snapshot();
    let contents = old
        .node(done.created(&template).unwrap().unwrap())
        .unwrap()
        .as_template()
        .unwrap()
        .contents()
        .unwrap();
    let inert = old.node(contents).unwrap().owner_document().unwrap();
    assert_ne!(inert, native);
    for document in [native, html, xml, inert] {
        assert!(
            !old.document(document)
                .unwrap()
                .metadata()
                .design_mode_enabled()
        );
    }
    let mut b = DomBatch::new();
    b.create_comment(native, "unrelated revision");
    dom.apply(b, &cancel).unwrap();
    assert_ne!(old.stamp(), dom.snapshot().stamp());
    for document in [native, html, xml, inert] {
        assert!(
            !dom.snapshot()
                .document(document)
                .unwrap()
                .metadata()
                .design_mode_enabled()
        );
        assert!(
            !old.document(document)
                .unwrap()
                .metadata()
                .design_mode_enabled()
        );
    }
}

#[test]
fn ordinary_editability_does_not_cross_shadow_or_template_boundaries_and_survives_slot_rebuild() {
    use surgeist_html::{HtmlShadowRootMode, HtmlShadowRootProperties, HtmlSlotAssignment};
    let cancel = AtomicBool::new(false);
    let mut dom = owner();
    let mut b = DomBatch::new();
    let d = b.create_document();
    let host = b.create_element_ns(&d, Namespace::html(), "div");
    b.set_attribute(&host, "contenteditable", "TrUe");
    b.insert_before(&host, &d, None);
    let child = b.create_element_ns(&d, Namespace::html(), "span");
    b.insert_before(&child, &host, None);
    let blocked = b.create_element_ns(&d, Namespace::html(), "span");
    b.set_attribute(&blocked, "contenteditable", "false");
    b.insert_before(&blocked, &host, None);
    let foreign = b.create_element_ns(&d, Namespace::none(), "div");
    b.set_attribute(&foreign, "contenteditable", "true");
    b.insert_before(&foreign, &host, None);
    let svg = b.create_element_ns(&d, Namespace::svg(), "svg");
    b.insert_before(&svg, &host, None);
    let detached = b.create_element_ns(&d, Namespace::html(), "div");
    b.set_attribute(&detached, "contenteditable", "plaintext-only");
    let shadow = b.attach_shadow(
        &host,
        HtmlShadowRootProperties::new(
            HtmlShadowRootMode::Closed,
            HtmlSlotAssignment::Named,
            false,
            false,
            false,
        ),
    );
    let shadow_child = b.create_element_ns(&d, Namespace::html(), "span");
    b.insert_before(&shadow_child, &shadow, None);
    let template = b.create_element_ns(&d, Namespace::html(), "template");
    b.insert_before(&template, &host, None);
    let done = dom.apply(b, &cancel).unwrap();
    let d = done.document(&d).unwrap();
    let id = |ticket: &surgeist_dom::CreatedNode| done.created(ticket).unwrap().unwrap();
    let (host, child, blocked, foreign, svg, detached, shadow_child, template) = (
        id(&host),
        id(&child),
        id(&blocked),
        id(&foreign),
        id(&svg),
        id(&detached),
        id(&shadow_child),
        id(&template),
    );
    let snapshot = dom.snapshot();
    let contents = snapshot
        .node(template)
        .unwrap()
        .as_template()
        .unwrap()
        .contents()
        .unwrap();
    let inert = snapshot.node(contents).unwrap().owner_document().unwrap();
    let mut b = DomBatch::new();
    let template_child = b.create_element_ns(inert, Namespace::html(), "span");
    b.insert_before(&template_child, contents, None);
    let done = dom.apply(b, &cancel).unwrap();
    let template_child = done.created(&template_child).unwrap().unwrap();
    let snapshot = dom.snapshot();
    let stamp = snapshot.stamp();
    for c in [capture(&snapshot, d), with_slots(capture(&snapshot, d))] {
        for node in [host, detached] {
            assert_eq!(
                fact(&c, node, HtmlSemanticQuery::EditingHost),
                HtmlSemanticFact::EditingHost(HtmlSemanticValue::Known(true))
            );
            assert_eq!(
                fact(&c, node, HtmlSemanticQuery::IsContentEditable),
                HtmlSemanticFact::IsContentEditable(HtmlSemanticValue::Known(true))
            );
        }
        assert_eq!(
            fact(&c, child, HtmlSemanticQuery::Editable),
            HtmlSemanticFact::Editable(HtmlSemanticValue::Known(true))
        );
        assert_eq!(
            fact(&c, svg, HtmlSemanticQuery::Editable),
            HtmlSemanticFact::Editable(HtmlSemanticValue::Known(true))
        );
        for node in [blocked, foreign, shadow_child, template_child] {
            assert_eq!(
                fact(&c, node, HtmlSemanticQuery::Editable),
                HtmlSemanticFact::Editable(HtmlSemanticValue::Known(false))
            );
        }
        assert!(
            !c.source()
                .document(inert)
                .unwrap()
                .metadata()
                .design_mode_enabled()
        );
    }
    assert_eq!(dom.snapshot().stamp(), stamp);
}

#[test]
fn explicit_spellcheck_and_contenteditable_mutation_preserve_old_captures_without_policy_defaults()
{
    let cancel = AtomicBool::new(false);
    let mut dom = owner();
    let mut b = DomBatch::new();
    let d = b.create_document();
    let e = b.create_element_ns(&d, Namespace::html(), "div");
    b.insert_before(&e, &d, None);
    let done = dom.apply(b, &cancel).unwrap();
    let d = done.document(&d).unwrap();
    let e = done.created(&e).unwrap().unwrap();
    let old = capture(&dom.snapshot(), d);
    assert!(matches!(
        fact(&old, e, HtmlSemanticQuery::Spellcheck),
        HtmlSemanticFact::Spellcheck(HtmlSemanticValue::Missing(
            HtmlSemanticMissing::SpellcheckDefault(_)
        ))
    ));
    for (value, expected) in [("", true), ("TrUe", true), ("FALSE", false)] {
        let mut b = DomBatch::new();
        b.set_attribute(e, "spellcheck", value);
        b.set_attribute(e, "contenteditable", "plaintext-only");
        dom.apply(b, &cancel).unwrap();
        let current = capture(&dom.snapshot(), d);
        assert_eq!(
            fact(&current, e, HtmlSemanticQuery::Spellcheck),
            HtmlSemanticFact::Spellcheck(HtmlSemanticValue::Known(expected))
        );
        assert_eq!(
            fact(
                &with_slots(current),
                e,
                HtmlSemanticQuery::IsContentEditable
            ),
            HtmlSemanticFact::IsContentEditable(HtmlSemanticValue::Known(true))
        );
    }
    let mut b = DomBatch::new();
    b.set_attribute(e, "spellcheck", "unknown");
    dom.apply(b, &cancel).unwrap();
    assert!(matches!(
        fact(
            &capture(&dom.snapshot(), d),
            e,
            HtmlSemanticQuery::Spellcheck
        ),
        HtmlSemanticFact::Spellcheck(HtmlSemanticValue::Missing(
            HtmlSemanticMissing::SpellcheckDefault(_)
        ))
    ));
    assert_eq!(
        fact(&old, e, HtmlSemanticQuery::EditingHost),
        HtmlSemanticFact::EditingHost(HtmlSemanticValue::Known(false))
    );
    assert!(matches!(
        fact(&old, e, HtmlSemanticQuery::Spellcheck),
        HtmlSemanticFact::Spellcheck(HtmlSemanticValue::Missing(
            HtmlSemanticMissing::SpellcheckDefault(_)
        ))
    ));
}

fn parse_limits() -> surgeist_html_parser::HtmlParseLimits {
    surgeist_html_parser::HtmlParseLimits::builder()
        .source_units(4096)
        .buffered_utf16_units(4096)
        .token_attributes(1024)
        .auxiliary_entries(100_000)
        .managed_nodes(1024)
        .managed_edges(1024)
        .managed_attributes(1024)
        .managed_utf16_units(65536)
        .source_map_entries(65536)
        .source_map_payload_units(65536)
        .diagnostics(1024)
        .diagnostic_payload_units(65536)
        .context_effects(1024)
        .open_elements(64)
        .tree_depth(128)
        .managed_capacity_bytes(16_000_000)
        .peak_transient_bytes(16_000_000)
        .atomic_layout_bytes(16_000_000)
        .work_credits(100_000_000)
        .finish()
        .unwrap()
}

#[test]
fn actual_reports_transfer_separate_owned_document_states_and_preserve_captured_target() {
    use std::sync::Arc;
    use surgeist_dom::{
        DomNativeInputPlatform, DomParsedDocumentEnvironment, DomParsedHtmlInstallLimits,
    };
    use surgeist_html_parser::{HtmlDocumentParseRequest, HtmlFragmentParseRequest, HtmlSource};
    use surgeist_html_semantics::{
        HtmlBaseEncoding, HtmlBaseFactsRevision, HtmlBaseProducerId, HtmlBaseUrlRecord,
        HtmlBaseUrlScheme, HtmlFragmentContext,
    };
    let cancel = AtomicBool::new(false);
    let mut dom = owner();
    let mut b = DomBatch::new();
    let native = b.create_document();
    let target = b.create_element_ns(&native, Namespace::html(), "div");
    b.insert_before(&target, &native, None);
    let done = dom.apply(b, &cancel).unwrap();
    let native = done.document(&native).unwrap();
    let target = done.created(&target).unwrap().unwrap();
    let before = dom.snapshot();
    let url = HtmlBaseUrlRecord::observed(
        HtmlBaseFactsRevision::new(HtmlBaseProducerId::fresh(), 1, 1, 1),
        "about:blank".into(),
        HtmlBaseUrlScheme::Other,
        false,
        None,
    )
    .unwrap();
    let environment = DomParsedDocumentEnvironment::headless(
        before.document(native).unwrap().metadata().origin().clone(),
        url,
        HtmlBaseEncoding::fresh(),
        "UTF-8".into(),
        DomNativeInputPlatform::headless(),
    );
    let report = Arc::new(
        HtmlDocumentParseRequest::offline(
            HtmlSource::utf8("<!doctype html><template><span></span></template>"),
            parse_limits(),
        )
        .parse()
        .unwrap(),
    );
    let independent =
        HtmlDocumentParseRequest::offline(HtmlSource::utf8("<div></div>"), parse_limits())
            .parse()
            .unwrap();
    assert_eq!(
        report
            .document()
            .unwrap()
            .root()
            .document_design_mode_enabled(),
        Some(false)
    );
    let installation = dom
        .install_parsed_html_document(
            before.stamp(),
            report.clone(),
            environment,
            DomParsedHtmlInstallLimits::new(1024, 10_000_000, 16_000_000, 100_000_000),
            &cancel,
        )
        .unwrap();
    assert!(
        installation
            .node_for(&independent.document().unwrap().root().id())
            .is_none()
    );
    let original = installation.document();
    let snapshot = dom.snapshot();
    let installed_documents: Vec<_> = snapshot
        .nodes()
        .filter_map(|n| snapshot.node(n).unwrap().as_document())
        .filter(|d| d.id() != native)
        .collect();
    assert_eq!(installed_documents.len(), 2); // ordinary + distinct inert Template owner
    for document in &installed_documents {
        assert!(!document.metadata().design_mode_enabled());
    }
    let fresh_capture = capture(&snapshot, native);
    let context = HtmlFragmentContext::element(
        fresh_capture.capture(),
        &fresh_capture.subject(target).unwrap(),
    )
    .unwrap();
    let fragment = Arc::new(
        HtmlFragmentParseRequest::new(
            HtmlSource::utf8("<script>inert</script><style>a{}</style>"),
            context,
            parse_limits(),
        )
        .unwrap()
        .parse()
        .unwrap(),
    );
    let descriptor = fragment.fragment_parser_document().unwrap();
    assert_eq!(
        descriptor.document().document_design_mode_enabled(),
        Some(false)
    );
    let auxiliary_source = descriptor.document().id();
    let installed = dom
        .install_parsed_html_fragment(
            fragment,
            &fresh_capture,
            DomParsedHtmlInstallLimits::new(1024, 10_000_000, 16_000_000, 100_000_000),
            &cancel,
        )
        .unwrap();
    let auxiliary = installed
        .snapshot()
        .node(installed.node_for(&auxiliary_source).unwrap())
        .unwrap()
        .as_document()
        .unwrap()
        .id();
    assert_ne!(auxiliary, native);
    assert_ne!(auxiliary, original);
    assert!(
        !installed
            .snapshot()
            .document(auxiliary)
            .unwrap()
            .metadata()
            .design_mode_enabled()
    );
    assert_eq!(
        installed.captured_node_for(&fresh_capture.subject(native.node()).unwrap()),
        Some(native.node())
    );
    assert!(
        !installed
            .snapshot()
            .document(native)
            .unwrap()
            .metadata()
            .design_mode_enabled()
    );
    assert!(
        !before
            .document(native)
            .unwrap()
            .metadata()
            .design_mode_enabled()
    );
}

#[test]
fn failed_or_cancelled_capture_does_not_mutate_canonical_state_and_good_limits_recover() {
    use surgeist_dom::{DomHtmlCaptureError, DomHtmlCaptureResource};
    let cancel = AtomicBool::new(false);
    let mut dom = owner();
    let mut b = DomBatch::new();
    let d = b.create_document();
    let root = b.create_element_ns(&d, Namespace::html(), "div");
    b.insert_before(&root, &d, None);
    let done = dom.apply(b, &cancel).unwrap();
    let d = done.document(&d).unwrap();
    let root = done.created(&root).unwrap().unwrap();
    let snapshot = dom.snapshot();
    let stamp = snapshot.stamp();
    let usage = dom.usage();
    let limits = HtmlCaptureLimits::new(512, 8192, 1024, 200_000, 512, 0, 0, 16_777_216);
    assert!(matches!(
        snapshot.html_capture(
            d,
            None,
            limits,
            DomHtmlCaptureLimits::new(1, 1_000_000),
            &cancel
        ),
        Err(DomHtmlCaptureError::ResourceLimit {
            resource: DomHtmlCaptureResource::Subjects,
            ..
        })
    ));
    assert!(matches!(
        snapshot.html_capture(d, None, limits, DomHtmlCaptureLimits::new(512, 0), &cancel),
        Err(DomHtmlCaptureError::ResourceLimit {
            resource: DomHtmlCaptureResource::Work,
            ..
        })
    ));
    assert!(matches!(
        snapshot.html_capture(
            d,
            None,
            limits,
            DomHtmlCaptureLimits::new(512, 1_000_000),
            &AtomicBool::new(true)
        ),
        Err(DomHtmlCaptureError::Cancelled)
    ));
    assert_eq!(
        fact(&capture(&snapshot, d), root, HtmlSemanticQuery::EditingHost),
        HtmlSemanticFact::EditingHost(HtmlSemanticValue::Known(false))
    );
    assert_eq!(dom.snapshot().stamp(), stamp);
    assert_eq!(dom.usage(), usage);
    assert!(
        !dom.snapshot()
            .document(d)
            .unwrap()
            .metadata()
            .design_mode_enabled()
    );
}
