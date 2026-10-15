//! DOM3071 createElementNS and HTML877d HTMLSlotElement native initialization:
//! an actual HTML Slot is a constructible Element with its own family state.
//! This admission test uses only the existing public constructor/readback API.
use std::sync::atomic::AtomicBool;
use surgeist_dom::{DocumentString, Dom, DomBatch, DomLimits, Namespace};

#[test]
fn native_html_slot_creation_publishes_an_actual_element() {
    let mut dom = Dom::new(DomLimits::new(8, 1024, 16, 0, 0, 4, 4096).unwrap()).unwrap();
    let mut batch = DomBatch::new();
    let document = batch.create_document();
    let slot = batch.create_element_ns(&document, Namespace::html(), "slot");
    let commit = dom
        .apply(batch, &AtomicBool::new(false))
        .expect("source-defined native HTML Slot creation must succeed");
    let capture = dom.snapshot();
    let node = capture
        .node(commit.created(&slot).unwrap().unwrap())
        .unwrap();
    assert_eq!(node.node_name(), DocumentString::from("slot"));
    assert_eq!(
        node.owner_document(),
        Some(commit.document(&document).unwrap())
    );
    assert_eq!(
        node.as_element().unwrap().name().namespace(),
        &Namespace::html()
    );
}
