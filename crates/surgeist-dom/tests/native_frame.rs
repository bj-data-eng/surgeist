//! DOM createElementNS and HTML877d Frame initialization: an actual native
//! null browsing context permits fresh null-content Frame construction.
use std::sync::atomic::AtomicBool;
use surgeist_dom::{DocumentString, Dom, DomBatch, DomLimits, Namespace};

const HTML: &str = "http://www.w3.org/1999/xhtml";

#[test]
fn native_html_frame_creation_publishes_an_actual_element() {
    let mut dom = Dom::new(DomLimits::new(8, 1024, 16, 4, 4096).unwrap()).unwrap();
    let mut batch = DomBatch::new();
    let document = batch.create_document();
    let frame = batch.create_element_ns(&document, Namespace::from_uri(HTML), "frame");
    let commit = dom
        .apply(batch, &AtomicBool::new(false))
        .expect("source-defined native HTML Frame creation must succeed");
    let capture = dom.snapshot();
    let node = capture
        .node(commit.created(&frame).unwrap().unwrap())
        .unwrap();
    assert_eq!(node.node_name(), DocumentString::from("frame"));
    assert_eq!(
        node.owner_document(),
        Some(commit.document(&document).unwrap())
    );
    assert_eq!(
        node.as_element().unwrap().name().namespace(),
        &Namespace::from_uri(HTML)
    );
}
