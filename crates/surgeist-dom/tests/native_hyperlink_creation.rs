//! DOM3071 namespace creation and HTML877d native A/Area URL initialization.
use std::sync::atomic::AtomicBool;
use surgeist_dom::{DocumentString, Dom, DomBatch, DomLimits, Namespace};

fn native_hyperlink_element(local: &str) {
    let mut dom = Dom::new(
        DomLimits::new(
            16,
            4096,
            64,
            0,
            0,
            8,
            65536,
            surgeist_dom::DomNameLimits::new(16_777_216, 16_384, 4_000_000, 1_048_576).unwrap(),
        )
        .unwrap(),
    )
    .unwrap();
    let mut batch = DomBatch::new();
    let document = batch.create_document();
    let element = batch.create_element_ns(&document, Namespace::html(), local);
    let commit = dom
        .apply(batch, &AtomicBool::new(false))
        .expect("source-defined native hyperlink creation must succeed");
    let snapshot = dom.snapshot();
    let node = snapshot
        .node(commit.created(&element).unwrap().unwrap())
        .unwrap();
    assert_eq!(node.node_name(), DocumentString::from(local));
    assert_eq!(
        node.owner_document(),
        Some(commit.document(&document).unwrap())
    );
    assert_eq!(
        node.as_element().unwrap().name().namespace_uri(),
        Namespace::html().uri()
    );
}

#[test]
fn native_html_anchor_creation_publishes_an_actual_element() {
    native_hyperlink_element("a");
}

#[test]
fn native_html_area_creation_publishes_an_actual_element() {
    native_hyperlink_element("area");
}
