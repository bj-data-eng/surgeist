//! DOM3071 namespace creation; HTML877d Meta has no extra fresh Element slot.
use std::sync::atomic::AtomicBool;
use surgeist_dom::{DocumentString, Dom, DomBatch, DomLimits, Namespace};

#[test]
fn detached_native_meta_creation_publishes_the_actual_element() {
    let mut dom = Dom::new(
        DomLimits::new(
            16,
            4096,
            64,
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
    let element = batch.create_element_ns(&document, Namespace::html(), "meta");
    let commit = dom
        .apply(batch, &AtomicBool::new(false))
        .expect("source-defined detached native Meta creation must succeed");
    let snapshot = dom.snapshot();
    let node = snapshot
        .node(commit.created(&element).unwrap().unwrap())
        .unwrap();
    assert_eq!(node.node_name(), DocumentString::from("meta"));
    assert_eq!(
        node.owner_document(),
        Some(commit.document(&document).unwrap())
    );
    let element = node.as_element().unwrap();
    assert_eq!(element.name().namespace_uri(), Namespace::html().uri());
    assert_eq!(element.nonce(), Some(&DocumentString::from("")));
    assert!(node.parent().is_none());
}
