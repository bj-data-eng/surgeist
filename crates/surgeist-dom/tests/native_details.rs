//! DOM3071 createElementNS and HTML877d native Details initialization.
use std::sync::atomic::AtomicBool;
use surgeist_dom::{DocumentString, Dom, DomBatch, DomLimits, Namespace};

#[test]
fn native_html_details_creation_publishes_an_actual_element() {
    let mut dom = Dom::new(
        DomLimits::new(
            8,
            1024,
            16,
            0,
            0,
            4,
            4096,
            surgeist_dom::DomNameLimits::new(16_777_216, 16_384, 4_000_000, 1_048_576).unwrap(),
        )
        .unwrap(),
    )
    .unwrap();
    let mut batch = DomBatch::new();
    let document = batch.create_document();
    let details = batch.create_element_ns(&document, Namespace::html(), "details");
    let commit = dom
        .apply(batch, &AtomicBool::new(false))
        .expect("source-defined native HTML Details creation must succeed");
    let snapshot = dom.snapshot();
    let node = snapshot
        .node(commit.created(&details).unwrap().unwrap())
        .unwrap();
    assert_eq!(node.node_name(), DocumentString::from("details"));
    assert_eq!(
        node.owner_document(),
        Some(commit.document(&document).unwrap())
    );
    assert_eq!(
        node.as_element().unwrap().name().namespace_uri(),
        Namespace::html().uri()
    );
}
