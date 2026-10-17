//! Document blocking facts and flat-tree facts have independent availability.
use std::sync::atomic::AtomicBool;
use surgeist_dom::*;
use surgeist_html::HtmlValueInterpreter;
use surgeist_html_semantics::{
    HtmlCaptureLimits, HtmlInertFactLimits, HtmlLanguageDirectionLimits, HtmlSemanticCancellation,
    HtmlSemanticFact, HtmlSemanticLimits, HtmlSemanticMissing, HtmlSemanticQuery,
    HtmlSemanticRequest, HtmlSemanticValue,
};

fn cancel() -> AtomicBool {
    AtomicBool::new(false)
}

struct Fixture {
    dom: Dom,
    document: DocumentId,
    host: NodeId,
    assigned: NodeId,
    detached_inert: NodeId,
    detached_child: NodeId,
}

fn fixture() -> Fixture {
    let mut dom = Dom::new(DomLimits::new(32, 16384, 64, 64, 64, 8, 65536).unwrap()).unwrap();
    let mut batch = DomBatch::new();
    let document = batch.create_document();
    let host = batch.create_element_ns(&document, Namespace::html(), "div");
    batch.insert_before(&host, &document, None);
    let root = batch.attach_shadow(&host, DomShadowRootInit::new(HtmlShadowRootMode::Open));
    let slot = batch.create_element_ns(&document, Namespace::html(), "slot");
    batch.set_attribute(&slot, "inert", "");
    batch.insert_before(&slot, &root, None);
    let assigned = batch.create_element_ns(&document, Namespace::html(), "span");
    batch.insert_before(&assigned, &host, None);
    let detached_inert = batch.create_element_ns(&document, Namespace::html(), "div");
    batch.set_attribute(&detached_inert, "inert", "");
    let detached_child = batch.create_element_ns(&document, Namespace::html(), "span");
    batch.insert_before(&detached_child, &detached_inert, None);
    let commit = dom.apply(batch, &cancel()).unwrap();
    Fixture {
        document: commit.document(&document).unwrap(),
        host: commit.created(&host).unwrap().unwrap(),
        assigned: commit.created(&assigned).unwrap().unwrap(),
        detached_inert: commit.created(&detached_inert).unwrap().unwrap(),
        detached_child: commit.created(&detached_child).unwrap().unwrap(),
        dom,
    }
}

fn capture(fixture: &Fixture) -> DomHtmlCapture {
    fixture
        .dom
        .snapshot()
        .html_capture(
            fixture.document,
            None,
            HtmlCaptureLimits::new(64, 4096, 128, 32768, 64, 0, 0),
            DomHtmlCaptureLimits::new(64, 1000000),
            &cancel(),
        )
        .unwrap()
}

fn inertness(capture: &DomHtmlCapture, node: NodeId) -> HtmlSemanticFact {
    capture
        .query(
            &[HtmlSemanticRequest::new(
                capture.subject(node).unwrap(),
                HtmlSemanticQuery::Inertness,
            )],
            HtmlValueInterpreter::new(16384),
            HtmlSemanticLimits::new(1000000, 16, 1024, 128, 1000000, 1024),
            HtmlSemanticCancellation::new(),
        )
        .unwrap()
        .results()[0]
        .fact()
        .clone()
}

#[test]
fn native_known_empty_layer_proves_document_unblocked_without_initial_flat_node_facts() {
    let fixture = fixture();
    let capture = capture(&fixture);
    assert_eq!(
        inertness(&capture, fixture.detached_inert),
        HtmlSemanticFact::Inertness(HtmlSemanticValue::Known(true))
    );
    // HTML 6.3.1 selects the topmost actual Dialog in the Document's top layer.
    // A fresh native Document's known-empty layer has none. The Document itself
    // needs no flat-node ancestry to establish its default non-inert result.
    assert_eq!(
        inertness(&capture, fixture.document.node()),
        HtmlSemanticFact::Inertness(HtmlSemanticValue::Known(false))
    );
    // Supplying the Document fact does not supply an Element's flat relation.
    assert_eq!(
        inertness(&capture, fixture.assigned),
        HtmlSemanticFact::Inertness(HtmlSemanticValue::Missing(
            HtmlSemanticMissing::InertNodeFacts(capture.subject(fixture.assigned).unwrap())
        ))
    );
}

#[test]
fn slot_supplement_preserves_known_empty_document_blocking_and_real_inert_ancestry() {
    let fixture = fixture();
    let capture = capture(&fixture)
        .with_slot_semantics(
            DomSlotSemanticLimits::new(
                DomSlotQueryLimits::new(64, 128).unwrap(),
                DomHtmlCaptureLimits::new(64, 1000000),
                HtmlLanguageDirectionLimits::new(128, 128, 16384, 1000000),
                HtmlInertFactLimits::new(128, 1000000),
            ),
            &cancel(),
        )
        .unwrap();
    assert_eq!(
        inertness(&capture, fixture.document.node()),
        HtmlSemanticFact::Inertness(HtmlSemanticValue::Known(false))
    );
    assert_eq!(
        inertness(&capture, fixture.host),
        HtmlSemanticFact::Inertness(HtmlSemanticValue::Known(false))
    );
    assert_eq!(
        inertness(&capture, fixture.assigned),
        HtmlSemanticFact::Inertness(HtmlSemanticValue::Known(true))
    );
    assert_eq!(
        inertness(&capture, fixture.detached_inert),
        HtmlSemanticFact::Inertness(HtmlSemanticValue::Known(true))
    );
    assert_eq!(
        inertness(&capture, fixture.detached_child),
        HtmlSemanticFact::Inertness(HtmlSemanticValue::Missing(
            HtmlSemanticMissing::InertAncestry(capture.subject(fixture.detached_child).unwrap())
        ))
    );
}
