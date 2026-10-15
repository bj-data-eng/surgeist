//! DOM3071 createElementNS and HTML877d HTMLFormElement initialization.
//! Native Form construction includes its real scalar state and past names map.
use std::sync::atomic::AtomicBool;
use surgeist_dom::{DocumentString, Dom, DomBatch, DomLimits, Namespace};

#[test]
fn native_html_form_creation_publishes_an_actual_element() {
    let mut dom = Dom::new(DomLimits::new(8, 1024, 16, 0, 0, 4, 4096).unwrap()).unwrap();
    let mut batch = DomBatch::new();
    let document = batch.create_document();
    let form = batch.create_element_ns(&document, Namespace::html(), "form");
    let commit = dom
        .apply(batch, &AtomicBool::new(false))
        .expect("source-defined native HTML Form creation must succeed");
    let capture = dom.snapshot();
    let node = capture
        .node(commit.created(&form).unwrap().unwrap())
        .unwrap();
    assert_eq!(node.node_name(), DocumentString::from("form"));
    assert_eq!(
        node.owner_document(),
        Some(commit.document(&document).unwrap())
    );
    assert_eq!(
        node.as_element().unwrap().name().namespace(),
        &Namespace::html()
    );
}
