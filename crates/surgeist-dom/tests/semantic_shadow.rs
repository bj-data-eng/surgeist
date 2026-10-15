//! Literal DOM3071 replacement/host relations and #1124 capture qualification.
use std::sync::atomic::AtomicBool;

use surgeist_dom::{
    DocumentId, DocumentString, Dom, DomBatch, DomError, DomHtmlCapture, DomHtmlCaptureError,
    DomHtmlCaptureLimits, DomHtmlCaptureResource, DomLimits, DomShadowRootInit, DomSnapshot,
    HtmlShadowRootMode, HtmlShadowRootProperties, HtmlSlotAssignment, Namespace, NodeId,
};
use surgeist_html::HtmlValueInterpreter;
use surgeist_html_semantics::{
    HtmlCaptureLimits, HtmlCapturedChildren, HtmlCapturedFragmentKind, HtmlCapturedHost,
    HtmlCapturedNodeKind, HtmlCapturedParent, HtmlCapturedRegistry, HtmlCapturedShadowRoot,
    HtmlLanguageOrigin, HtmlSemanticCancellation, HtmlSemanticFact, HtmlSemanticLimits,
    HtmlSemanticQuery, HtmlSemanticRequest, HtmlSemanticValue,
};

fn dom() -> Dom {
    Dom::new(DomLimits::new(128, 16384, 256, 0, 0, 16, 65536).unwrap()).unwrap()
}
fn apply(dom: &mut Dom, batch: DomBatch) -> surgeist_dom::DomCommit {
    dom.apply(batch, &AtomicBool::new(false)).unwrap()
}
fn limits() -> HtmlCaptureLimits {
    HtmlCaptureLimits::new(128, 2048, 128, 16384, 128, 0, 0)
}
fn capture(snapshot: &DomSnapshot, document: DocumentId) -> DomHtmlCapture {
    snapshot
        .html_capture(
            document,
            None,
            limits(),
            DomHtmlCaptureLimits::new(128, 100000),
            &AtomicBool::new(false),
        )
        .unwrap()
}
fn document_of(capture: &DomHtmlCapture, node: NodeId) -> NodeId {
    capture
        .node(
            capture
                .capture()
                .node(&capture.subject(node).unwrap())
                .unwrap()
                .node_document()
                .subject(),
        )
        .unwrap()
}
fn assert_shadow(capture: &DomHtmlCapture, host: NodeId, root: NodeId) {
    let host_view = capture
        .capture()
        .node(&capture.subject(host).unwrap())
        .unwrap();
    let Some(HtmlCapturedShadowRoot::Present(actual)) = host_view.shadow_root() else {
        panic!("actual reciprocal shadow root")
    };
    assert_eq!(capture.node(actual.subject()).unwrap(), root);
    let root_view = capture
        .capture()
        .node(&capture.subject(root).unwrap())
        .unwrap();
    let Some(HtmlCapturedHost::Element(actual)) = root_view.host() else {
        panic!("actual reciprocal host")
    };
    assert_eq!(capture.node(actual.subject()).unwrap(), host);
    assert!(matches!(root_view.parent(), HtmlCapturedParent::Root));
    assert!(matches!(
        root_view.lookup_registry(),
        HtmlCapturedRegistry::Null
    ));
}

struct Split {
    dom: Dom,
    a: DocumentId,
    b: DocumentId,
    ancestor: NodeId,
    host: NodeId,
    root: NodeId,
    child: NodeId,
    target: NodeId,
    old: NodeId,
    unrelated: NodeId,
    light: NodeId,
}
fn split_fixture() -> Split {
    let mut dom = dom();
    let mut batch = DomBatch::new();
    let a = batch.create_document();
    let b = batch.create_document();
    let ancestor = batch.create_element(&a, "Ancestor");
    batch.set_attribute_ns(
        &ancestor,
        Namespace::from_uri("http://www.w3.org/XML/1998/namespace"),
        "xml:lang",
        "ancestor-language",
    );
    let host = batch.create_element_ns(&a, Namespace::html(), "div");
    let root = batch.attach_shadow(&host, DomShadowRootInit::new(HtmlShadowRootMode::Closed));
    let child = batch.create_comment(&a, "transferred");
    let unrelated = batch.create_comment(&a, "unrelated document child");
    let light = batch.create_comment(&a, "unrelated light child");
    let target = batch.create_element(&b, "Target");
    let old = batch.create_comment(&b, "old");
    batch.insert_before(&host, &ancestor, None);
    batch.insert_before(&ancestor, &a, None);
    batch.insert_before(&unrelated, &a, None);
    batch.insert_before(&light, &host, None);
    batch.insert_before(&child, &root, None);
    batch.insert_before(&old, &target, None);
    let commit = apply(&mut dom, batch);
    Split {
        a: commit.document(&a).unwrap(),
        b: commit.document(&b).unwrap(),
        ancestor: commit.created(&ancestor).unwrap().unwrap(),
        host: commit.created(&host).unwrap().unwrap(),
        root: commit.created(&root).unwrap().unwrap(),
        child: commit.created(&child).unwrap().unwrap(),
        target: commit.created(&target).unwrap().unwrap(),
        old: commit.created(&old).unwrap().unwrap(),
        unrelated: commit.created(&unrelated).unwrap().unwrap(),
        light: commit.created(&light).unwrap().unwrap(),
        dom,
    }
}

#[test]
fn replacement_captures_actual_distinct_root_and_host_documents_without_completing_other_trees() {
    let mut f = split_fixture();
    let before = capture(&f.dom.snapshot(), f.a);
    let mut batch = DomBatch::new();
    batch.replace_child(f.root, f.target, f.old);
    apply(&mut f.dom, batch);
    let snapshot = f.dom.snapshot();
    let b = capture(&snapshot, f.b);
    assert_eq!(b.basis(), snapshot.document(f.b).unwrap().stamp());
    assert_eq!(b.capture().usage().subjects(), 8);
    for node in [
        f.b.node(),
        f.target,
        f.old,
        f.child,
        f.root,
        f.host,
        f.ancestor,
        f.a.node(),
    ] {
        assert_eq!(b.node(&b.subject(node).unwrap()).unwrap(), node);
    }
    assert_shadow(&b, f.host, f.root);
    assert_eq!(document_of(&b, f.root), f.b.node());
    assert_eq!(document_of(&b, f.host), f.a.node());
    for node in [f.a.node(), f.ancestor, f.host] {
        assert!(matches!(
            b.capture()
                .node(&b.subject(node).unwrap())
                .unwrap()
                .children(),
            HtmlCapturedChildren::Unavailable
        ));
    }
    for node in [f.unrelated, f.light] {
        assert!(matches!(
            b.subject(node),
            Err(DomHtmlCaptureError::Source(DomError::WrongDocument))
        ));
    }
    let HtmlCapturedChildren::Complete(mut children) = b
        .capture()
        .node(&b.subject(f.root).unwrap())
        .unwrap()
        .children()
    else {
        panic!("emptied root is complete")
    };
    assert!(children.next().is_none());
    assert!(snapshot.node(f.root).unwrap().is_connected());
    assert_eq!(snapshot.node(f.root).unwrap().root(), f.root);
    assert_eq!(
        snapshot.node(f.root).unwrap().shadow_including_root(),
        f.a.node()
    );
    assert_eq!(
        snapshot.node(f.root).unwrap().tree_scope().host(),
        Some(f.host)
    );
    assert_eq!(document_of(&before, f.root), f.a.node());
    assert_eq!(before.source().node(f.root).unwrap().children(), &[f.child]);

    let a = capture(&snapshot, f.a);
    assert_shadow(&a, f.host, f.root);
    assert_eq!(a.capture().usage().subjects(), 7);
    assert_eq!(document_of(&a, f.root), f.b.node());
    assert!(matches!(
        a.capture()
            .node(&a.subject(f.b.node()).unwrap())
            .unwrap()
            .children(),
        HtmlCapturedChildren::Unavailable
    ));
    assert!(matches!(
        a.subject(f.target),
        Err(DomHtmlCaptureError::Source(DomError::WrongDocument))
    ));
    assert!(matches!(
        a.subject(f.child),
        Err(DomHtmlCaptureError::Source(DomError::WrongDocument))
    ));
}

#[test]
fn insertion_keeps_root_document_and_public_adoption_rejects_without_changing_captures() {
    let mut f = split_fixture();
    let old = capture(&f.dom.snapshot(), f.a);
    let mut batch = DomBatch::new();
    batch.insert_before(f.root, f.target, None);
    apply(&mut f.dom, batch);
    let snapshot = f.dom.snapshot();
    let a = capture(&snapshot, f.a);
    assert_shadow(&a, f.host, f.root);
    assert_eq!(document_of(&a, f.root), f.a.node());
    let b = capture(&snapshot, f.b);
    assert_eq!(document_of(&b, f.child), f.b.node());
    assert!(matches!(
        b.subject(f.root),
        Err(DomHtmlCaptureError::Source(DomError::WrongDocument))
    ));
    assert!(matches!(
        a.subject(f.b.node()),
        Err(DomHtmlCaptureError::Source(DomError::WrongDocument))
    ));
    let mut batch = DomBatch::new();
    batch.adopt(f.root, f.b);
    assert!(matches!(
        f.dom.apply(batch, &AtomicBool::new(false)),
        Err(DomError::HierarchyRequest)
    ));
    assert_eq!(f.dom.snapshot().stamp(), snapshot.stamp());
    assert_eq!(old.source().node(f.root).unwrap().children(), &[f.child]);
    assert_eq!(old.node(&old.subject(f.child).unwrap()).unwrap(), f.child);
}

#[test]
fn nested_open_closed_roots_preserve_actual_properties_language_and_detached_scope() {
    let mut dom = dom();
    let mut batch = DomBatch::new();
    let document = batch.create_document();
    let host = batch.create_element_ns(&document, Namespace::html(), "div");
    batch.set_attribute_ns(
        &host,
        Namespace::from_uri("http://www.w3.org/XML/1998/namespace"),
        "xml:lang",
        "host-language",
    );
    let properties = HtmlShadowRootProperties::new(
        HtmlShadowRootMode::Open,
        HtmlSlotAssignment::Manual,
        true,
        true,
        true,
    );
    let outer = batch.attach_shadow(&host, properties);
    let inner_host = batch.create_element_ns(&document, Namespace::html(), "span");
    let inner = batch.attach_shadow(
        &inner_host,
        DomShadowRootInit::new(HtmlShadowRootMode::Closed),
    );
    let leaf = batch.create_element(&document, "Leaf");
    batch.insert_before(&host, &document, None);
    batch.insert_before(&inner_host, &outer, None);
    batch.insert_before(&leaf, &inner, None);
    let commit = apply(&mut dom, batch);
    let document = commit.document(&document).unwrap();
    let host = commit.created(&host).unwrap().unwrap();
    let outer = commit.created(&outer).unwrap().unwrap();
    let inner_host = commit.created(&inner_host).unwrap().unwrap();
    let inner = commit.created(&inner).unwrap().unwrap();
    let leaf = commit.created(&leaf).unwrap().unwrap();
    let connected = capture(&dom.snapshot(), document);
    let mut batch = DomBatch::new();
    batch.detach(host);
    apply(&mut dom, batch);
    let snapshot = dom.snapshot();
    let detached = capture(&snapshot, document);
    for capture in [&connected, &detached] {
        assert_shadow(capture, host, outer);
        assert_shadow(capture, inner_host, inner);
        for node in [document.node(), host, outer, inner_host, inner, leaf] {
            assert_eq!(capture.node(&capture.subject(node).unwrap()).unwrap(), node);
        }
        let HtmlCapturedNodeKind::Fragment(value) = capture
            .capture()
            .node(&capture.subject(outer).unwrap())
            .unwrap()
            .kind()
        else {
            panic!("shadow fragment")
        };
        let HtmlCapturedFragmentKind::ShadowRoot(value) = value.kind() else {
            panic!("shadow properties")
        };
        assert_eq!(value.properties(), properties);
        assert!(!value.declarative());
        assert!(!value.available_to_internals());
        assert!(!value.keep_registry_null());
        let report = capture
            .query(
                &[HtmlSemanticRequest::new(
                    capture.subject(leaf).unwrap(),
                    HtmlSemanticQuery::Language,
                )],
                HtmlValueInterpreter::new(1024),
                HtmlSemanticLimits::new(10000, 8, 128, 64, 100000, 1024),
                HtmlSemanticCancellation::new(),
            )
            .unwrap();
        let HtmlSemanticFact::Language(HtmlSemanticValue::Known(value)) =
            report.results()[0].fact()
        else {
            panic!("host inherited language")
        };
        assert_eq!(value.tag(), &DocumentString::from("host-language"));
        let HtmlLanguageOrigin::XmlAttribute(origin) = value.origin() else {
            panic!("XML origin")
        };
        assert_eq!(capture.node(origin).unwrap(), host);
    }
    assert_eq!(snapshot.node(inner_host).unwrap().shadow_root(), None);
    assert_eq!(snapshot.node(host).unwrap().shadow_root(), Some(outer));
    assert_eq!(snapshot.node(leaf).unwrap().root(), inner);
    assert_eq!(
        snapshot.node(leaf).unwrap().tree_scope().host(),
        Some(inner_host)
    );
    assert_eq!(snapshot.node(leaf).unwrap().shadow_including_root(), host);
    assert!(!snapshot.node(leaf).unwrap().is_connected());
    assert!(connected.source().node(leaf).unwrap().is_connected());
}

#[test]
fn supplemental_closure_has_exact_subject_admission_and_cancelled_capture_is_read_only() {
    let mut f = split_fixture();
    let mut batch = DomBatch::new();
    batch.replace_child(f.root, f.target, f.old);
    apply(&mut f.dom, batch);
    let snapshot = f.dom.snapshot();
    let stamp = snapshot.stamp();
    let usage = f.dom.usage();
    assert!(
        snapshot
            .html_capture(
                f.b,
                None,
                limits(),
                DomHtmlCaptureLimits::new(8, 100000),
                &AtomicBool::new(false)
            )
            .is_ok()
    );
    assert!(matches!(
        snapshot.html_capture(
            f.b,
            None,
            limits(),
            DomHtmlCaptureLimits::new(7, 100000),
            &AtomicBool::new(false)
        ),
        Err(DomHtmlCaptureError::ResourceLimit {
            resource: DomHtmlCaptureResource::Subjects,
            limit: 7
        })
    ));
    assert!(
        snapshot
            .html_capture(
                f.b,
                None,
                HtmlCaptureLimits::new(8, 2048, 128, 16384, 128, 0, 0),
                DomHtmlCaptureLimits::new(8, 100000),
                &AtomicBool::new(false)
            )
            .is_ok()
    );
    assert!(matches!(
        snapshot.html_capture(
            f.b,
            None,
            HtmlCaptureLimits::new(7, 2048, 128, 16384, 128, 0, 0),
            DomHtmlCaptureLimits::new(8, 100000),
            &AtomicBool::new(false)
        ),
        Err(DomHtmlCaptureError::Capture(_))
    ));
    assert!(matches!(
        snapshot.html_capture(
            f.b,
            None,
            limits(),
            DomHtmlCaptureLimits::new(8, 0),
            &AtomicBool::new(false)
        ),
        Err(DomHtmlCaptureError::ResourceLimit {
            resource: DomHtmlCaptureResource::Work,
            limit: 0
        })
    ));
    assert!(matches!(
        snapshot.html_capture(
            f.b,
            None,
            limits(),
            DomHtmlCaptureLimits::new(8, 100000),
            &AtomicBool::new(true)
        ),
        Err(DomHtmlCaptureError::Cancelled)
    ));
    assert_eq!(f.dom.snapshot().stamp(), stamp);
    assert_eq!(f.dom.usage(), usage);
}
