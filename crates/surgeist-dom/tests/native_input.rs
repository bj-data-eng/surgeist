//! DOM3071 createElementNS and HTML877d native Input initialization.
use std::sync::atomic::AtomicBool;
use surgeist_dom::{DocumentString, Dom, DomBatch, DomLimits, Namespace};

#[test]
fn native_html_input_creation_publishes_an_actual_element() {
    let mut dom = Dom::new(DomLimits::new(8, 1024, 16, 0, 0, 4, 4096).unwrap()).unwrap();
    let mut batch = DomBatch::new();
    let document = batch.create_document();
    let input = batch.create_element_ns(&document, Namespace::html(), "input");
    let commit = dom
        .apply(batch, &AtomicBool::new(false))
        .expect("source-defined native HTML Input creation must succeed");
    let capture = dom.snapshot();
    let node = capture
        .node(commit.created(&input).unwrap().unwrap())
        .unwrap();
    assert_eq!(node.node_name(), DocumentString::from("input"));
    assert_eq!(
        node.owner_document(),
        Some(commit.document(&document).unwrap())
    );
    assert_eq!(
        node.as_element().unwrap().name().namespace(),
        &Namespace::html()
    );
}
