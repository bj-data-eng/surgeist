//! DOM3071 createElementNS and HTML877d HTMLTemplateElement native creation.
//! This admission oracle uses only the already callable public DOM surface.
use std::sync::atomic::AtomicBool;
use surgeist_dom::{DocumentString, Dom, DomBatch, DomLimits, Namespace};

#[test]
fn native_html_template_creation_publishes_an_actual_element() {
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
    let template = batch.create_element_ns(&document, Namespace::html(), "template");
    let commit = dom
        .apply(batch, &AtomicBool::new(false))
        .expect("source-defined native HTML Template creation must succeed");
    let capture = dom.snapshot();
    let node = capture
        .node(commit.created(&template).unwrap().unwrap())
        .unwrap();
    assert_eq!(node.node_name(), DocumentString::from("template"));
    assert_eq!(
        node.owner_document(),
        Some(commit.document(&document).unwrap())
    );
    assert_eq!(
        node.as_element().unwrap().name().namespace_uri(),
        Namespace::html().uri()
    );
    assert!(node.children().is_empty());
}
