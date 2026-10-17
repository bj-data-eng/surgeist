//! Real capture composition retains independent modal and flat-fact admission.
use std::sync::atomic::AtomicBool;
use surgeist_dom::*;
use surgeist_html::HtmlValueInterpreter;
use surgeist_html_semantics::{
    HtmlCaptureLimits, HtmlCapturedChildren, HtmlInertFactError, HtmlInertFactLimits,
    HtmlLanguageDirectionLimits, HtmlSemanticCancellation, HtmlSemanticFact, HtmlSemanticLimits,
    HtmlSemanticQuery, HtmlSemanticRequest, HtmlSemanticValue,
};

fn cancel() -> AtomicBool {
    AtomicBool::new(false)
}
fn owner() -> Dom {
    Dom::new(DomLimits::new(64, 16384, 128, 128, 128, 8, 65536).unwrap()).unwrap()
}
fn capture(source: &DomSnapshot, document: DocumentId) -> DomHtmlCapture {
    source
        .html_capture(
            document,
            None,
            HtmlCaptureLimits::new(128, 4096, 256, 32768, 128, 0, 0),
            DomHtmlCaptureLimits::new(128, 1000000),
            &cancel(),
        )
        .unwrap()
}
fn slot_limits(records: usize, producer_work: usize) -> DomSlotSemanticLimits {
    DomSlotSemanticLimits::new(
        DomSlotQueryLimits::new(64, 128).unwrap(),
        DomHtmlCaptureLimits::new(128, 1000000),
        HtmlLanguageDirectionLimits::new(128, 128, 16384, 1000000),
        HtmlInertFactLimits::new(records, producer_work),
    )
}
fn inertness(capture: &DomHtmlCapture, node: NodeId) -> HtmlSemanticFact {
    capture
        .query(
            &[HtmlSemanticRequest::new(
                capture.subject(node).unwrap(),
                HtmlSemanticQuery::Inertness,
            )],
            HtmlValueInterpreter::new(16384),
            HtmlSemanticLimits::new(1000000, 16, 1024, 256, 1000000, 1024),
            HtmlSemanticCancellation::new(),
        )
        .unwrap()
        .results()[0]
        .fact()
        .clone()
}

#[test]
fn document_only_slot_composition_counts_both_flat_and_modal_rows_and_rejects_partial_results() {
    let mut dom = owner();
    let mut batch = DomBatch::new();
    let document = batch.create_document();
    let commit = dom.apply(batch, &cancel()).unwrap();
    let document = commit.document(&document).unwrap();
    let source = dom.snapshot();
    let stamp = source.stamp();
    let usage = dom.usage();
    // The real flat Document Root and its independent modal row occupy two
    // allocated producer record slots. One slot cannot admit both facts.
    assert!(matches!(
        capture(&source, document).with_slot_semantics(slot_limits(1, 1000000), &cancel()),
        Err(DomHtmlCaptureError::Inert(
            HtmlInertFactError::RecordCapacity {
                limit: 1,
                required: 2
            }
        ))
    ));
    assert!(matches!(
        capture(&source, document).with_slot_semantics(slot_limits(2, 0), &cancel()),
        Err(DomHtmlCaptureError::Inert(
            HtmlInertFactError::ValidationWork { limit: 0, .. }
        ))
    ));
    assert!(matches!(
        capture(&source, document)
            .with_slot_semantics(slot_limits(2, 1000000), &AtomicBool::new(true)),
        Err(DomHtmlCaptureError::Cancelled)
    ));
    let capture = capture(&source, document)
        .with_slot_semantics(slot_limits(2, 1000000), &cancel())
        .unwrap();
    assert_eq!(
        inertness(&capture, document.node()),
        HtmlSemanticFact::Inertness(HtmlSemanticValue::Known(false))
    );
    assert_eq!(capture.basis(), source.document(document).unwrap().stamp());
    assert_eq!(dom.snapshot().stamp(), stamp);
    assert_eq!(dom.usage(), usage);
}

#[test]
fn supplemental_document_known_empty_fact_does_not_require_complete_child_coverage() {
    let mut dom = owner();
    let mut batch = DomBatch::new();
    let a = batch.create_document();
    let b = batch.create_document();
    let host = batch.create_element_ns(&a, Namespace::html(), "div");
    batch.insert_before(&host, &a, None);
    let root = batch.attach_shadow(&host, DomShadowRootInit::new(HtmlShadowRootMode::Open));
    let target = batch.create_element(&b, "Target");
    let old = batch.create_comment(&b, "old");
    batch.insert_before(&old, &target, None);
    let commit = dom.apply(batch, &cancel()).unwrap();
    let a = commit.document(&a).unwrap();
    let b = commit.document(&b).unwrap();
    let root = commit.created(&root).unwrap().unwrap();
    let target = commit.created(&target).unwrap().unwrap();
    let old = commit.created(&old).unwrap().unwrap();
    let mut batch = DomBatch::new();
    batch.replace_child(root, target, old);
    dom.apply(batch, &cancel()).unwrap();
    let source = dom.snapshot();
    let initial = capture(&source, b);
    let supplemented = capture(&source, b)
        .with_slot_semantics(slot_limits(256, 1000000), &cancel())
        .unwrap();
    for captured in [initial, supplemented] {
        assert!(matches!(
            captured
                .capture()
                .node(&captured.subject(a.node()).unwrap())
                .unwrap()
                .children(),
            HtmlCapturedChildren::Unavailable
        ));
        for document in [a, b] {
            assert_eq!(
                inertness(&captured, document.node()),
                HtmlSemanticFact::Inertness(HtmlSemanticValue::Known(false))
            );
        }
    }
}

#[test]
fn historical_capture_keeps_its_blocking_basis_and_flat_ancestry_after_real_edits() {
    let mut dom = owner();
    let mut batch = DomBatch::new();
    let document = batch.create_document();
    let root = batch.create_element_ns(&document, Namespace::html(), "div");
    let child = batch.create_element_ns(&document, Namespace::html(), "span");
    batch.insert_before(&root, &document, None);
    batch.insert_before(&child, &root, None);
    let commit = dom.apply(batch, &cancel()).unwrap();
    let document = commit.document(&document).unwrap();
    let root = commit.created(&root).unwrap().unwrap();
    let child = commit.created(&child).unwrap().unwrap();
    let before = dom.snapshot();
    let historical = capture(&before, document);
    let mut batch = DomBatch::new();
    batch.set_attribute(root, "inert", "");
    dom.apply(batch, &cancel()).unwrap();
    // Reconstructing from the original capture consumes its original snapshot,
    // independently of the live owner's newer canonical revision.
    let historical = historical
        .with_slot_semantics(slot_limits(128, 1000000), &cancel())
        .unwrap();
    let current = capture(&dom.snapshot(), document)
        .with_slot_semantics(slot_limits(128, 1000000), &cancel())
        .unwrap();
    assert_eq!(
        inertness(&historical, child),
        HtmlSemanticFact::Inertness(HtmlSemanticValue::Known(false))
    );
    assert_eq!(
        inertness(&current, child),
        HtmlSemanticFact::Inertness(HtmlSemanticValue::Known(true))
    );
    for captured in [&historical, &current] {
        assert_eq!(
            inertness(captured, document.node()),
            HtmlSemanticFact::Inertness(HtmlSemanticValue::Known(false))
        );
    }
    assert_eq!(
        historical.basis(),
        before.document(document).unwrap().stamp()
    );
    assert_ne!(historical.basis(), current.basis());
    assert_ne!(
        historical.capture().identity(),
        current.capture().identity()
    );
    assert_ne!(
        historical.capture().host_facts_revision(),
        current.capture().host_facts_revision()
    );
}
