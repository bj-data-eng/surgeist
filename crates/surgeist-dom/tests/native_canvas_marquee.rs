//! HTML877d native Canvas creation and obsolete Marquee creation must establish
//! actual family state before canonical publication. These constructor tests use
//! only the existing DOM API; family-specific readback belongs to later tests.
use std::sync::atomic::AtomicBool;
use surgeist_dom::{DocumentString, Dom, DomBatch, DomLimits, Namespace};

const HTML: &str = "http://www.w3.org/1999/xhtml";

#[test]
fn native_html_canvas_creation_publishes_an_actual_element() {
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
    let canvas = batch.create_element_ns(&document, Namespace::from_uri(HTML), "canvas");
    let commit = dom
        .apply(batch, &AtomicBool::new(false))
        .expect("source-defined native HTML Canvas creation must succeed");
    let capture = dom.snapshot();
    let node = capture
        .node(commit.created(&canvas).unwrap().unwrap())
        .unwrap();
    assert_eq!(node.node_name(), DocumentString::from("canvas"));
    assert_eq!(
        node.owner_document(),
        Some(commit.document(&document).unwrap())
    );
    assert_eq!(
        node.as_element().unwrap().name().namespace_uri(),
        Namespace::from_uri(HTML).uri()
    );
}

#[test]
fn native_html_marquee_creation_publishes_an_actual_element() {
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
    let marquee = batch.create_element_ns(&document, Namespace::from_uri(HTML), "marquee");
    let commit = dom
        .apply(batch, &AtomicBool::new(false))
        .expect("source-defined native HTML Marquee creation must succeed");
    let capture = dom.snapshot();
    let node = capture
        .node(commit.created(&marquee).unwrap().unwrap())
        .unwrap();
    assert_eq!(node.node_name(), DocumentString::from("marquee"));
    assert_eq!(
        node.owner_document(),
        Some(commit.document(&document).unwrap())
    );
    assert_eq!(
        node.as_element().unwrap().name().namespace_uri(),
        Namespace::from_uri(HTML).uri()
    );
}
