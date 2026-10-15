//! DOM3071 createElementNS and HTML877d HTMLFieldSetElement initialization:
//! an actual native HTML Fieldset is a constructible form-associated Element.
//! This acceptance fixture uses only existing public construction/readback APIs.
use std::sync::atomic::AtomicBool;
use surgeist_dom::{DocumentString, Dom, DomBatch, DomLimits, Namespace};

#[test]
fn native_html_fieldset_creation_publishes_an_actual_element() {
    let mut dom = Dom::new(DomLimits::new(8, 1024, 16, 0, 0, 4, 4096).unwrap()).unwrap();
    let mut batch = DomBatch::new();
    let document = batch.create_document();
    let fieldset = batch.create_element_ns(&document, Namespace::html(), "fieldset");
    let commit = dom
        .apply(batch, &AtomicBool::new(false))
        .expect("source-defined native HTML Fieldset creation must succeed");
    let capture = dom.snapshot();
    let node = capture
        .node(commit.created(&fieldset).unwrap().unwrap())
        .unwrap();
    assert_eq!(node.node_name(), DocumentString::from("fieldset"));
    assert_eq!(
        node.owner_document(),
        Some(commit.document(&document).unwrap())
    );
    assert_eq!(
        node.as_element().unwrap().name().namespace(),
        &Namespace::html()
    );
}
