//! Existing public DOM constructor boundary. HTML877d allows native Iframe
//! creation with the accepted fresh never-BC/unprocessed/null-child phase.
use std::sync::atomic::AtomicBool;
use surgeist_dom::{DocumentString, Dom, DomBatch, DomLimits, Namespace};

const HTML: &str = "http://www.w3.org/1999/xhtml";

#[test]
fn native_html_iframe_creation_publishes_an_actual_element() {
    let mut dom = Dom::new(DomLimits::new(8, 1024, 16, 0, 0, 4, 4096).unwrap()).unwrap();
    let mut batch = DomBatch::new();
    let document = batch.create_document();
    let iframe = batch.create_element_ns(&document, Namespace::from_uri(HTML), "iframe");
    let commit = dom
        .apply(batch, &AtomicBool::new(false))
        .expect("source-defined native HTML Iframe creation must succeed");
    let capture = dom.snapshot();
    let node = capture
        .node(commit.created(&iframe).unwrap().unwrap())
        .unwrap();
    assert_eq!(node.node_name(), DocumentString::from("iframe"));
    assert_eq!(
        node.owner_document(),
        Some(commit.document(&document).unwrap())
    );
    assert_eq!(
        node.as_element().unwrap().name().namespace(),
        &Namespace::from_uri(HTML)
    );
}
