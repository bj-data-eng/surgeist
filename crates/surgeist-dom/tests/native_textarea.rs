//! DOM3071 createElementNS and HTML877d native Textarea initialization.
use std::sync::atomic::AtomicBool;
use surgeist_dom::{DocumentString, Dom, DomBatch, DomLimits, Namespace};

#[test]
fn native_html_textarea_creation_publishes_an_actual_element() {
    let mut dom = Dom::new(DomLimits::new(8, 1024, 16, 0, 0, 4, 4096).unwrap()).unwrap();
    let mut batch = DomBatch::new();
    let document = batch.create_document();
    let textarea = batch.create_element_ns(&document, Namespace::html(), "textarea");
    let commit = dom
        .apply(batch, &AtomicBool::new(false))
        .expect("source-defined native HTML Textarea creation must succeed");
    let snapshot = dom.snapshot();
    let node = snapshot
        .node(commit.created(&textarea).unwrap().unwrap())
        .unwrap();
    assert_eq!(node.node_name(), DocumentString::from("textarea"));
    assert_eq!(
        node.owner_document(),
        Some(commit.document(&document).unwrap())
    );
    assert_eq!(
        node.as_element().unwrap().name().namespace(),
        &Namespace::html()
    );
}
