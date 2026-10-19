//! DOM3071 createElementNS and the selected native SVG Script contract.
use std::sync::atomic::AtomicBool;
use surgeist_dom::{DocumentString, Dom, DomBatch, DomLimits, Namespace};

#[test]
fn native_svg_script_creation_publishes_an_actual_element() {
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
    let script = batch.create_element_ns(&document, Namespace::svg(), "script");
    let commit = dom
        .apply(batch, &AtomicBool::new(false))
        .expect("source-defined native SVG Script creation must succeed");
    let snapshot = dom.snapshot();
    let node = snapshot
        .node(commit.created(&script).unwrap().unwrap())
        .unwrap();
    assert_eq!(node.node_name(), DocumentString::from("script"));
    assert_eq!(
        node.owner_document(),
        Some(commit.document(&document).unwrap())
    );
    assert_eq!(
        node.as_element().unwrap().name().namespace_uri(),
        Namespace::svg().uri()
    );
}
