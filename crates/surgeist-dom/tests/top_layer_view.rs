//! Qualified public readback of canonical layer state and generated UA input.
use std::sync::atomic::AtomicBool;
use surgeist_dom::*;
use surgeist_html::HtmlStyleUpdateAction;

fn cancel() -> AtomicBool {
    AtomicBool::new(false)
}
fn limits() -> DomSourceLimits {
    DomSourceLimits::new(16_000_000, 1_000_000, 10_000_000, 128, 4096, 4096, 1)
}
fn fixture() -> (Dom, DocumentId, NodeId, NodeId) {
    let mut dom = Dom::new(
        DomLimits::new(
            64,
            32768,
            128,
            128,
            128,
            16,
            1_000_000,
            surgeist_dom::DomNameLimits::new(16_777_216, 16_384, 4_000_000, 1_048_576).unwrap(),
        )
        .unwrap(),
    )
    .unwrap();
    let mut batch = DomBatch::new();
    let document = batch.create_document();
    let root = batch.create_element_ns(&document, Namespace::html(), "div");
    let style = batch.create_element_ns(&document, Namespace::html(), "style");
    batch.insert_before(&root, &document, None);
    let done = dom.apply(batch, &cancel()).unwrap();
    let document = done.document(&document).unwrap();
    let root = done.created(&root).unwrap().unwrap();
    let style = done.created(&style).unwrap().unwrap();
    (dom, document, root, style)
}

#[test]
fn complete_native_empty_layer_exposes_universal_important_none_with_document_stamp() {
    let (mut dom, document, root, _) = fixture();
    let before = dom.snapshot();
    let old_document = before.document(document).unwrap();
    let old = old_document.top_layer().unwrap();
    assert_eq!(old.stamp(), old_document.stamp());
    assert!(old.raw_members().is_empty());
    assert!(old.pending_removals().is_empty());
    // The closed variant denotes UA origin, important, universal overlay:none;
    // an empty layer has no generated Element overlay:auto declarations.
    assert_eq!(
        old.ua_rules().collect::<Vec<_>>(),
        [DomTopLayerUaRule::UniversalOverlayNone]
    );
    let mut batch = DomBatch::new();
    batch.set_attribute(root, "data-revision", "next");
    dom.apply(batch, &cancel()).unwrap();
    let current = dom.snapshot();
    let current_document = current.document(document).unwrap();
    let view = current_document.top_layer().unwrap();
    assert_eq!(view.stamp(), current_document.stamp());
    assert_ne!(view.stamp(), old.stamp());
    assert!(view.raw_members().is_empty());
    assert!(view.pending_removals().is_empty());
    assert_eq!(
        view.ua_rules().collect::<Vec<_>>(),
        [DomTopLayerUaRule::UniversalOverlayNone]
    );
    assert_eq!(old.stamp(), old_document.stamp());
    assert!(old.raw_members().is_empty());
    assert!(old.pending_removals().is_empty());
    assert_eq!(
        old.ua_rules().collect::<Vec<_>>(),
        [DomTopLayerUaRule::UniversalOverlayNone]
    );
}

#[test]
fn actual_incomplete_source_refuses_layer_view_while_old_complete_snapshot_remains_qualified() {
    let (mut dom, document, root, style) = fixture();
    let before = dom.snapshot();
    let old_document = before.document(document).unwrap();
    let old = old_document.top_layer().unwrap();
    let operation;
    let request;
    {
        let mut lease = dom
            .begin_source_edit(
                DomSourceEdit::insert_before(style, root, None),
                limits(),
                &cancel(),
            )
            .unwrap();
        operation = lease.operation();
        request = match lease.advance(&cancel()).unwrap() {
            DomSourceProgress::Request(DomSourceRequest::Style(request)) => request,
            other => panic!("actual connected Style must reach its policy boundary: {other:?}"),
        };
        assert_eq!(request.source().element(), style);
        assert_eq!(
            request.action(),
            DomStyleAction::Update(HtmlStyleUpdateAction::CheckInlineCsp)
        );
        let pending = lease.snapshot();
        assert_eq!(pending.incomplete_source_operation(), Some(operation));
        assert_eq!(pending.node(style).unwrap().parent(), Some(root));
        assert!(matches!(
            pending.document(document).unwrap().top_layer(),
            Err(DomError::IncompleteSourceOperation)
        ));
        assert_eq!(old.stamp(), old_document.stamp());
        assert!(old.raw_members().is_empty());
        assert!(old.pending_removals().is_empty());
        assert_eq!(
            old.ua_rules().collect::<Vec<_>>(),
            [DomTopLayerUaRule::UniversalOverlayNone]
        );
    }
    assert_eq!(dom.pending_source_edit().unwrap().operation(), operation);
    assert!(matches!(
        dom.snapshot().document(document).unwrap().top_layer(),
        Err(DomError::IncompleteSourceOperation)
    ));
    let mut lease = dom
        .resume_source_edit(operation, limits(), &cancel())
        .unwrap();
    let resumed = match lease.advance(&cancel()).unwrap() {
        DomSourceProgress::Request(DomSourceRequest::Style(request)) => request,
        other => panic!("unfulfilled policy must remain the actual Source boundary: {other:?}"),
    };
    assert_eq!(resumed.source().operation(), request.source().operation());
    assert_eq!(resumed.source().ordinal(), request.source().ordinal());
    assert_eq!(resumed.source().element(), request.source().element());
    assert_eq!(resumed.source().stamp(), request.source().stamp());
    assert_eq!(resumed.action(), request.action());
    assert!(matches!(
        lease.snapshot().document(document).unwrap().top_layer(),
        Err(DomError::IncompleteSourceOperation)
    ));
    assert_eq!(old.stamp(), old_document.stamp());
    assert_eq!(
        old.ua_rules().collect::<Vec<_>>(),
        [DomTopLayerUaRule::UniversalOverlayNone]
    );
}
