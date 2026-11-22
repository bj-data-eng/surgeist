//! DOM3071 namespace creation and HTML877d native image-family initialization.
use std::sync::atomic::AtomicBool;
use surgeist_dom::{
    Dom, DomBatch, DomError,
    edit::{DomDetachedNodeCreation, DomNodeRetirementLimits},
    limits::DomLimits,
    source::DomSourceLimits,
};
use surgeist_html::values::{DocumentString, Namespace};

fn dom() -> Dom {
    Dom::new(
        DomLimits::new(
            16,
            4096,
            64,
            0,
            0,
            8,
            65536,
            surgeist_dom::limits::DomNameLimits::new(16_777_216, 16_384, 4_000_000, 1_048_576)
                .unwrap(),
        )
        .unwrap(),
    )
    .unwrap()
}

fn native_image_element(local: &str) {
    let mut dom = dom();
    let mut batch = DomBatch::new();
    let document = batch.create_document();
    let element = batch.create_element_ns(&document, Namespace::html(), local);
    let commit = dom
        .apply(batch, &AtomicBool::new(false))
        .expect("source-defined native image-family creation must succeed");
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
fn native_html_img_creation_publishes_an_actual_element() {
    native_image_element("img");
}

#[test]
fn native_html_picture_creation_publishes_an_actual_element() {
    native_image_element("picture");
}

#[test]
fn native_html_source_creation_publishes_an_actual_element() {
    native_image_element("source");
}

#[test]
fn image_factory_rolls_back_both_dimensions_if_the_watch_is_refused() {
    let mut dom = dom();
    let cancel = AtomicBool::new(false);
    let mut batch = DomBatch::new();
    let document = batch.create_document();
    let commit = dom.apply(batch, &cancel).unwrap();
    let document = commit.document(&document).unwrap();
    let watch_limits = DomNodeRetirementLimits::new(16_000_000, 20_000);
    let mut inbox = dom.node_retirement_inbox(watch_limits, &cancel).unwrap();
    let before = dom.snapshot();
    let source = DomSourceLimits::new(16_000_000, 200_000, 20_000, 64, 64, 64, 2);
    let creation = || DomDetachedNodeCreation::HtmlImage {
        width: Some(u32::MAX),
        height: Some(0),
    };

    let refused = dom.create_detached_with_node_retirement_watch(
        document,
        creation(),
        (&inbox, 1),
        (source, DomNodeRetirementLimits::new(16_000_000, 0)),
        &cancel,
    );
    assert!(matches!(
        refused,
        Err(DomError::NodeRetirementWorkLimit { limit: 0 })
    ));
    assert_eq!(dom.snapshot().stamp(), before.stamp());
    assert_eq!(dom.snapshot().nodes().count(), before.nodes().count());
    assert!(dom.pending_source_edit().is_none());
    assert!(inbox.take().is_none());

    let (commit, node, _watch) = dom
        .create_detached_with_node_retirement_watch(
            document,
            creation(),
            (&inbox, 1),
            (source, watch_limits),
            &cancel,
        )
        .unwrap()
        .into_parts();
    let after = dom.snapshot();
    let image = after.node(node).unwrap();
    assert_eq!(commit.changes().from(), before.stamp());
    assert_eq!(commit.changes().to(), after.stamp());
    assert_eq!(image.owner_document(), Some(document));
    assert_eq!(image.parent(), None);
    assert_eq!(image.node_name(), "img".into());
    for (name, value) in [("width", "4294967295"), ("height", "0")] {
        assert_eq!(
            image.attribute_ns(&Namespace::none(), &name.into()),
            Some(&value.into())
        );
    }
    assert!(dom.pending_source_edit().is_none());
}
