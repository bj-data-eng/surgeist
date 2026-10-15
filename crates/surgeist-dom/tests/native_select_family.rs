//! DOM3071 createElementNS and HTML877d native Select-family initialization.
use std::sync::atomic::AtomicBool;
use surgeist_dom::{DocumentString, Dom, DomBatch, DomLimits, Namespace};

fn native_element(local_name: &str) {
    let mut dom = Dom::new(DomLimits::new(8, 1024, 16, 0, 0, 4, 4096).unwrap()).unwrap();
    let mut batch = DomBatch::new();
    let document = batch.create_document();
    let element = batch.create_element_ns(&document, Namespace::html(), local_name);
    let commit = dom
        .apply(batch, &AtomicBool::new(false))
        .expect("source-defined native HTML family creation must succeed");
    let snapshot = dom.snapshot();
    let node = snapshot
        .node(commit.created(&element).unwrap().unwrap())
        .unwrap();
    assert_eq!(node.node_name(), DocumentString::from(local_name));
    assert_eq!(
        node.owner_document(),
        Some(commit.document(&document).unwrap())
    );
    assert_eq!(
        node.as_element().unwrap().name().namespace(),
        &Namespace::html()
    );
}

#[test]
fn native_html_select_creation_publishes_an_actual_element() {
    native_element("select");
}

#[test]
fn native_html_option_creation_publishes_an_actual_element() {
    native_element("option");
}

#[test]
fn native_html_optgroup_creation_publishes_an_actual_element() {
    native_element("optgroup");
}

#[test]
fn native_html_selectedcontent_creation_publishes_an_actual_element() {
    native_element("selectedcontent");
}
