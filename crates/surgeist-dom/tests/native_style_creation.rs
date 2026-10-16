//! DOM3071 namespace creation and HTML877d native Style initialization.
use std::sync::atomic::AtomicBool;
use surgeist_dom::{DocumentString, Dom, DomBatch, DomLimits, Namespace};

#[test]
fn native_html_style_creation_publishes_an_actual_element() {
    let mut dom = Dom::new(DomLimits::new(16, 4096, 64, 0, 0, 8, 65536).unwrap()).unwrap();
    let mut batch = DomBatch::new();
    let document = batch.create_document();
    let element = batch.create_element_ns(&document, Namespace::html(), "style");
    let commit = dom
        .apply(batch, &AtomicBool::new(false))
        .expect("source-defined detached native Style creation must succeed");
    let snapshot = dom.snapshot();
    let node = snapshot
        .node(commit.created(&element).unwrap().unwrap())
        .unwrap();
    assert_eq!(node.node_name(), DocumentString::from("style"));
    assert_eq!(
        node.owner_document(),
        Some(commit.document(&document).unwrap())
    );
    assert_eq!(
        node.as_element().unwrap().name().namespace(),
        &Namespace::html()
    );
}
