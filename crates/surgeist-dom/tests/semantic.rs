use std::sync::atomic::AtomicBool;

use surgeist_dom::{
    DocumentId, Dom, DomBatch, DomError, DomHostDomains, DomHostError, DomHostObservations,
    DomHostPublisher, DomHtmlCapture, DomHtmlCaptureError, DomHtmlCaptureLimits,
    DomHtmlCaptureResource, DomLimits, DomSnapshot, Namespace, NodeId, Observed,
};
use surgeist_html::{
    DocumentString, HtmlBrowsingContextPresence, HtmlCapturedNonce, HtmlHeaderDeliveredCsp,
    HtmlValueInterpreter,
};
use surgeist_html_semantics::{
    HtmlCaptureError, HtmlCaptureLimits, HtmlCapturedChildren, HtmlCapturedCustomElementState,
    HtmlCapturedHost, HtmlCapturedIsValue, HtmlCapturedNodeKind, HtmlCapturedParent,
    HtmlCapturedRegistry, HtmlCapturedShadowRoot, HtmlDocumentClass, HtmlImageSupport,
    HtmlLanguageOrigin, HtmlScriptingStatus, HtmlSemanticCancellation, HtmlSemanticDependencies,
    HtmlSemanticFact, HtmlSemanticLimits, HtmlSemanticQuery, HtmlSemanticRequest,
    HtmlSemanticValue,
};

fn dom() -> Dom {
    Dom::new(DomLimits::new(64, 8192, 128, 0, 0, 8, 65536).unwrap()).unwrap()
}
fn apply(dom: &mut Dom, batch: DomBatch) -> surgeist_dom::DomCommit {
    dom.apply(batch, &AtomicBool::new(false)).unwrap()
}
fn limits() -> HtmlCaptureLimits {
    HtmlCaptureLimits::new(64, 1024, 128, 16384, 64, 0, 0)
}
fn adapter() -> DomHtmlCaptureLimits {
    DomHtmlCaptureLimits::new(64, 10000)
}
fn capture(snapshot: &DomSnapshot, document: DocumentId) -> DomHtmlCapture {
    snapshot
        .html_capture(document, None, limits(), adapter(), &AtomicBool::new(false))
        .unwrap()
}
fn query(
    capture: &DomHtmlCapture,
    node: NodeId,
    query: HtmlSemanticQuery,
) -> surgeist_html_semantics::HtmlSemanticReport {
    capture
        .query(
            &[HtmlSemanticRequest::new(
                capture.subject(node).unwrap(),
                query,
            )],
            HtmlValueInterpreter::new(8192),
            HtmlSemanticLimits::new(100000, 16, 1024, 64, 1000000, 1024),
            HtmlSemanticCancellation::new(),
        )
        .unwrap()
}

#[test]
fn xml_language_inheritance_uses_the_real_producer_and_maps_the_selecting_ancestor() {
    let mut dom = dom();
    let mut batch = DomBatch::new();
    let document = batch.create_document();
    let root = batch.create_element(&document, "Root");
    let child = batch.create_element(&document, "Child");
    batch.set_attribute_ns(
        &root,
        Namespace::from_uri("http://www.w3.org/XML/1998/namespace"),
        "xml:lang",
        "x-example-INVALID",
    );
    batch.set_attribute(&child, "xml:lang", "literal must not select");
    batch.insert_before(&child, &root, None);
    batch.insert_before(&root, &document, None);
    let commit = apply(&mut dom, batch);
    let document = commit.document(&document).unwrap();
    let root = commit.created(&root).unwrap().unwrap();
    let child = commit.created(&child).unwrap().unwrap();
    let snapshot = dom.snapshot();
    let captured = capture(&snapshot, document);
    let report = query(&captured, child, HtmlSemanticQuery::Language);
    let HtmlSemanticFact::Language(HtmlSemanticValue::Known(language)) = report.results()[0].fact()
    else {
        panic!("known inherited XML language")
    };
    assert_eq!(language.tag(), &DocumentString::from("x-example-INVALID"));
    let HtmlLanguageOrigin::XmlAttribute(origin) = language.origin() else {
        panic!("XML language origin")
    };
    assert_eq!(captured.node(origin).unwrap(), root);
    assert_eq!(
        captured
            .node(report.results()[0].request().subject())
            .unwrap(),
        child
    );
    assert_eq!(report.capture(), captured.capture().identity());
    assert_eq!(
        report.host_revision(),
        captured.capture().host_facts_revision()
    );
    assert_eq!(
        report.dependencies(),
        HtmlSemanticDependencies::ConservativeRecompute
    );
    assert_eq!(
        captured.basis(),
        snapshot.document(document).unwrap().stamp()
    );
}

#[test]
fn complete_document_closure_includes_detached_roots_and_exact_intrinsic_values() {
    let mut dom = dom();
    let mut batch = DomBatch::new();
    let document = batch.create_document();
    let other_document = batch.create_document();
    let fragment = batch.create_fragment(&document);
    let element = batch.create_element(&document, "MiXeD");
    let text = batch.create_text(&document, DocumentString::from_utf16(vec![0xd800, 0x41]));
    let comment = batch.create_comment(&document, "comment");
    let pi = batch.create_processing_instruction(&document, "pi", "a='1'");
    let doctype = batch.create_document_type(&document, "", "", "system");
    let foreign = batch.create_comment(&other_document, "outside document");
    batch.insert_before(&element, &fragment, None);
    batch.insert_before(&text, &element, None);
    batch.replace_data(&pi, 0, usize::MAX, "a='?>'");
    let commit = apply(&mut dom, batch);
    let document = commit.document(&document).unwrap();
    let fragment = commit.created(&fragment).unwrap().unwrap();
    let element = commit.created(&element).unwrap().unwrap();
    let text = commit.created(&text).unwrap().unwrap();
    let comment = commit.created(&comment).unwrap().unwrap();
    let pi = commit.created(&pi).unwrap().unwrap();
    let doctype = commit.created(&doctype).unwrap().unwrap();
    let foreign = commit.created(&foreign).unwrap().unwrap();
    let captured = capture(&dom.snapshot(), document);
    for id in [
        document.node(),
        fragment,
        element,
        text,
        comment,
        pi,
        doctype,
    ] {
        let subject = captured.subject(id).unwrap();
        assert_eq!(captured.node(&subject).unwrap(), id);
        assert_eq!(
            captured
                .node(
                    captured
                        .capture()
                        .node(&subject)
                        .unwrap()
                        .node_document()
                        .subject()
                )
                .unwrap(),
            document.node()
        );
    }
    assert!(matches!(
        captured.subject(foreign),
        Err(DomHtmlCaptureError::Source(DomError::WrongDocument))
    ));
    let fragment = captured
        .capture()
        .node(&captured.subject(fragment).unwrap())
        .unwrap();
    assert!(matches!(fragment.host(), Some(HtmlCapturedHost::Absent)));
    let HtmlCapturedChildren::Complete(children) = fragment.children() else {
        panic!("complete children")
    };
    assert_eq!(
        children
            .map(|subject| captured.node(&subject).unwrap())
            .collect::<Vec<_>>(),
        [element]
    );
    assert!(matches!(
        captured
            .capture()
            .node(&captured.subject(comment).unwrap())
            .unwrap()
            .parent(),
        HtmlCapturedParent::Root
    ));
    let HtmlCapturedNodeKind::ProcessingInstruction(value) = captured
        .capture()
        .node(&captured.subject(pi).unwrap())
        .unwrap()
        .kind()
    else {
        panic!("PI")
    };
    assert_eq!(value.data(), &DocumentString::from("a='?>'"));
    let HtmlCapturedNodeKind::Doctype(value) = captured
        .capture()
        .node(&captured.subject(doctype).unwrap())
        .unwrap()
        .kind()
    else {
        panic!("doctype")
    };
    assert_eq!(value.public_id(), Some(&DocumentString::from("")));
    assert_eq!(value.system_id(), Some(&DocumentString::from("system")));
}

#[test]
fn native_document_environment_proves_disabled_scripting_without_external_policy_defaults() {
    let mut dom = dom();
    let mut batch = DomBatch::new();
    let document = batch.create_document();
    let document = apply(&mut dom, batch).document(&document).unwrap();
    let captured = capture(&dom.snapshot(), document);
    let subject = captured.subject(document.node()).unwrap();
    let HtmlCapturedNodeKind::Document(facts) = captured.capture().node(&subject).unwrap().kind()
    else {
        panic!("Document")
    };
    assert_eq!(facts.scripting(), HtmlScriptingStatus::Disabled);
    assert_eq!(facts.document_class(), HtmlDocumentClass::NonHtml);
    assert_eq!(facts.image_support(), HtmlImageSupport::Unavailable);
    assert_eq!(
        facts.parser_creation_image_support(),
        HtmlImageSupport::Unavailable
    );
    assert_eq!(
        facts.nonce_environment().browsing_context(),
        HtmlBrowsingContextPresence::Absent
    );
    assert_eq!(
        facts.nonce_environment().header_delivered_csp(),
        HtmlHeaderDeliveredCsp::Unavailable
    );
    assert!(captured.host().is_none());
}

#[test]
fn common_html_nonce_and_custom_state_come_from_current_slots_including_imported_values() {
    let mut source = dom();
    let mut batch = DomBatch::new();
    let document = batch.create_document();
    let root = batch.create_element_ns(&document, Namespace::html(), "x-example");
    let empty = batch.create_element_ns(&document, Namespace::html(), "div");
    batch.set_attribute(&root, "nonce", "content");
    batch.set_nonce(&root, "actual slot");
    batch.set_attribute(&root, "is", "current content only");
    let commit = apply(&mut source, batch);
    let document = commit.document(&document).unwrap();
    let root = commit.created(&root).unwrap().unwrap();
    let empty = commit.created(&empty).unwrap().unwrap();
    let snapshot = source.snapshot();
    let captured = capture(&snapshot, document);
    let subject = captured.subject(root).unwrap();
    let node = captured.capture().node(&subject).unwrap();
    assert!(matches!(node.lookup_registry(), HtmlCapturedRegistry::Null));
    assert!(matches!(
        node.shadow_root(),
        Some(HtmlCapturedShadowRoot::Absent)
    ));
    let HtmlCapturedNodeKind::Element(value) = node.kind() else {
        panic!("Element")
    };
    assert_eq!(value.is_value(), &HtmlCapturedIsValue::Absent);
    assert_eq!(
        value.custom_element_state(),
        HtmlCapturedCustomElementState::Known(surgeist_dom::HtmlCustomElementState::Undefined)
    );
    assert_eq!(
        value.cryptographic_nonce(),
        &HtmlCapturedNonce::Value("actual slot".into())
    );
    assert!(
        matches!(query(&captured, root, HtmlSemanticQuery::Nonce).results()[0].fact(), HtmlSemanticFact::Nonce(HtmlSemanticValue::Known(value)) if value == &DocumentString::from("actual slot"))
    );
    assert!(
        matches!(query(&captured, empty, HtmlSemanticQuery::Nonce).results()[0].fact(), HtmlSemanticFact::Nonce(HtmlSemanticValue::Known(value)) if value.is_empty())
    );
    assert!(
        captured
            .source()
            .node(root)
            .unwrap()
            .as_element()
            .unwrap()
            .common_state()
            .html()
            .is_some()
    );
    let mut target = dom();
    let mut batch = DomBatch::new();
    let target_document = batch.create_document();
    let imported = batch.import_node(&snapshot, root, &target_document, false);
    let commit = apply(&mut target, batch);
    let target_document = commit.document(&target_document).unwrap();
    let imported = commit.created(&imported).unwrap().unwrap();
    let captured = capture(&target.snapshot(), target_document);
    assert!(
        matches!(query(&captured, imported, HtmlSemanticQuery::Nonce).results()[0].fact(), HtmlSemanticFact::Nonce(HtmlSemanticValue::Known(value)) if value == &DocumentString::from("actual slot"))
    );
}

#[test]
fn unavailable_language_fallback_and_inapplicable_nonce_remain_distinct() {
    let mut dom = dom();
    let mut batch = DomBatch::new();
    let document = batch.create_document();
    let root = batch.create_element(&document, "Root");
    let commit = apply(&mut dom, batch);
    let document = commit.document(&document).unwrap();
    let root = commit.created(&root).unwrap().unwrap();
    let captured = capture(&dom.snapshot(), document);
    assert!(matches!(
        query(&captured, root, HtmlSemanticQuery::Language).results()[0].fact(),
        HtmlSemanticFact::Language(HtmlSemanticValue::Missing(_))
    ));
    assert!(matches!(
        query(&captured, root, HtmlSemanticQuery::Nonce).results()[0].fact(),
        HtmlSemanticFact::Nonce(HtmlSemanticValue::Inapplicable)
    ));
}

#[test]
fn host_publications_require_exact_dom_basis_and_retain_the_actual_external_revision() {
    let mut dom = dom();
    let mut batch = DomBatch::new();
    let document = batch.create_document();
    let other_document = batch.create_document();
    let root = batch.create_element(&document, "Root");
    let commit = apply(&mut dom, batch);
    let document = commit.document(&document).unwrap();
    let other_document = commit.document(&other_document).unwrap();
    let root = commit.created(&root).unwrap().unwrap();
    let snapshot = dom.snapshot();
    let view = snapshot.document(document).unwrap();
    let mut publisher = DomHostPublisher::new(DomHostDomains::Focus);
    let first = publisher
        .publish(
            view,
            view.stamp(),
            DomHostObservations::Focus(Observed::Supported(Some(root))),
        )
        .unwrap();
    let first_capture = snapshot
        .html_capture(
            document,
            Some(first.capture()),
            limits(),
            adapter(),
            &AtomicBool::new(false),
        )
        .unwrap();
    assert!(matches!(
        snapshot.html_capture(
            other_document,
            Some(first.capture()),
            limits(),
            adapter(),
            &AtomicBool::new(false)
        ),
        Err(DomHtmlCaptureError::Host(DomHostError::WrongDocument))
    ));
    let second = publisher
        .publish(
            view,
            view.stamp(),
            DomHostObservations::Focus(Observed::Unsupported),
        )
        .unwrap();
    let second_capture = snapshot
        .html_capture(
            document,
            Some(second.capture()),
            limits(),
            adapter(),
            &AtomicBool::new(false),
        )
        .unwrap();
    assert_eq!(first_capture.host().unwrap(), first.capture());
    assert_eq!(second_capture.host().unwrap(), second.capture());
    assert_ne!(
        first_capture.host().unwrap().stamp(),
        second_capture.host().unwrap().stamp()
    );
    assert_eq!(first_capture.basis(), second_capture.basis());
    let mut batch = DomBatch::new();
    batch.set_attribute(root, "changed", "new tree");
    apply(&mut dom, batch);
    assert!(matches!(
        dom.snapshot().html_capture(
            document,
            Some(second.capture()),
            limits(),
            adapter(),
            &AtomicBool::new(false)
        ),
        Err(DomHtmlCaptureError::Host(
            DomHostError::DomRevisionMismatch { .. }
        ))
    ));
}

#[test]
fn identity_map_is_capture_qualified_and_old_adopted_destroyed_nodes_remain_readable() {
    let mut dom = dom();
    let mut batch = DomBatch::new();
    let source = batch.create_document();
    let target = batch.create_document();
    let root = batch.create_element(&source, "Root");
    let commit = apply(&mut dom, batch);
    let source = commit.document(&source).unwrap();
    let target = commit.document(&target).unwrap();
    let root = commit.created(&root).unwrap().unwrap();
    let snapshot = dom.snapshot();
    let old = capture(&snapshot, source);
    let separate = capture(&snapshot, source);
    let subject = old.subject(root).unwrap();
    assert!(matches!(
        separate.node(&subject),
        Err(DomHtmlCaptureError::Capture(
            HtmlCaptureError::ForeignCapture
        ))
    ));
    let mut batch = DomBatch::new();
    batch.adopt(root, target);
    apply(&mut dom, batch);
    let current = capture(&dom.snapshot(), target);
    assert_eq!(current.node(&current.subject(root).unwrap()).unwrap(), root);
    assert_eq!(old.node(&subject).unwrap(), root);
    let mut batch = DomBatch::new();
    batch.destroy(root);
    apply(&mut dom, batch);
    assert_eq!(old.subject(root).unwrap(), subject);
    assert!(matches!(
        capture(&dom.snapshot(), target).subject(root),
        Err(DomHtmlCaptureError::Source(DomError::StaleNode))
    ));
}

#[test]
fn admission_and_cancellation_return_no_partial_capture_or_dom_change() {
    let mut dom = dom();
    let mut batch = DomBatch::new();
    let document = batch.create_document();
    let root = batch.create_element(&document, "Root");
    batch.set_attribute(&root, "key", "value");
    let document = apply(&mut dom, batch).document(&document).unwrap();
    let snapshot = dom.snapshot();
    let stamp = snapshot.stamp();
    let usage = dom.usage();
    assert!(matches!(
        snapshot.html_capture(
            document,
            None,
            limits(),
            DomHtmlCaptureLimits::new(1, 10000),
            &AtomicBool::new(false)
        ),
        Err(DomHtmlCaptureError::ResourceLimit {
            resource: DomHtmlCaptureResource::Subjects,
            limit: 1
        })
    ));
    assert!(
        snapshot
            .html_capture(
                document,
                None,
                limits(),
                DomHtmlCaptureLimits::new(2, 10000),
                &AtomicBool::new(false)
            )
            .is_ok()
    );
    assert!(matches!(
        snapshot.html_capture(
            document,
            None,
            limits(),
            DomHtmlCaptureLimits::new(2, 0),
            &AtomicBool::new(false)
        ),
        Err(DomHtmlCaptureError::ResourceLimit {
            resource: DomHtmlCaptureResource::Work,
            limit: 0
        })
    ));
    assert!(matches!(
        snapshot.html_capture(
            document,
            None,
            HtmlCaptureLimits::new(64, 1024, 128, 1, 64, 0, 0),
            adapter(),
            &AtomicBool::new(false)
        ),
        Err(DomHtmlCaptureError::Capture(_))
    ));
    assert!(matches!(
        snapshot.html_capture(document, None, limits(), adapter(), &AtomicBool::new(true)),
        Err(DomHtmlCaptureError::Cancelled)
    ));
    assert_eq!(dom.snapshot().stamp(), stamp);
    assert_eq!(dom.usage(), usage);
}

#[test]
fn sparse_snapshot_scanning_obeys_work_admission() {
    let mut dom = dom();
    let mut batch = DomBatch::new();
    let document = batch.create_document();
    let discarded: Vec<_> = (0..32)
        .map(|_| batch.create_comment(&document, "discarded"))
        .collect();
    let commit = apply(&mut dom, batch);
    let document = commit.document(&document).unwrap();
    let mut batch = DomBatch::new();
    for node in discarded {
        batch.destroy(commit.created(&node).unwrap().unwrap());
    }
    apply(&mut dom, batch);
    let snapshot = dom.snapshot();
    assert_eq!(snapshot.nodes().count(), 1);
    assert!(matches!(
        snapshot.html_capture(
            document,
            None,
            limits(),
            DomHtmlCaptureLimits::new(1, 16),
            &AtomicBool::new(false)
        ),
        Err(DomHtmlCaptureError::ResourceLimit {
            resource: DomHtmlCaptureResource::Work,
            limit: 16
        })
    ));
    assert!(
        snapshot
            .html_capture(
                document,
                None,
                limits(),
                DomHtmlCaptureLimits::new(1, 10000),
                &AtomicBool::new(false)
            )
            .is_ok()
    );
    assert_eq!(dom.snapshot().stamp(), snapshot.stamp());
}

#[test]
fn producer_query_rejects_foreign_requests_exhaustion_and_cancellation() {
    let mut dom = dom();
    let mut batch = DomBatch::new();
    let document = batch.create_document();
    let root = batch.create_element(&document, "Root");
    batch.set_attribute_ns(
        &root,
        Namespace::from_uri("http://www.w3.org/XML/1998/namespace"),
        "xml:lang",
        "en",
    );
    let commit = apply(&mut dom, batch);
    let document = commit.document(&document).unwrap();
    let root = commit.created(&root).unwrap().unwrap();
    let snapshot = dom.snapshot();
    let first = capture(&snapshot, document);
    let second = capture(&snapshot, document);
    let limits = HtmlSemanticLimits::new(100000, 16, 1024, 64, 1000000, 1024);
    let request =
        HtmlSemanticRequest::new(first.subject(root).unwrap(), HtmlSemanticQuery::Language);
    assert!(matches!(
        second.query(
            std::slice::from_ref(&request),
            HtmlValueInterpreter::new(8192),
            limits,
            HtmlSemanticCancellation::new()
        ),
        Err(DomHtmlCaptureError::Semantic(_))
    ));
    let cancelled = HtmlSemanticCancellation::new();
    cancelled.cancel();
    assert!(matches!(
        first.query(
            std::slice::from_ref(&request),
            HtmlValueInterpreter::new(8192),
            limits,
            cancelled
        ),
        Err(DomHtmlCaptureError::Semantic(_))
    ));
    assert!(matches!(
        first.query(
            &[request],
            HtmlValueInterpreter::new(8192),
            HtmlSemanticLimits::new(0, 16, 1024, 64, 1000000, 1024),
            HtmlSemanticCancellation::new()
        ),
        Err(DomHtmlCaptureError::Semantic(_))
    ));
    assert_eq!(dom.snapshot().stamp(), snapshot.stamp());
}
