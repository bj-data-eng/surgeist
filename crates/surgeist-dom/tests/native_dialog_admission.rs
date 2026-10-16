//! DOM3071 createElementNS and HTML877d Dialog native creation.
//! This checks constructor admission without asserting Dialog-specific state.
use std::sync::atomic::AtomicBool;
use surgeist_dom::{DocumentString, Dom, DomBatch, DomLimits, Namespace};

#[test]
fn native_html_dialog_can_be_created_and_inserted_with_common_state() {
    let limits = DomLimits::new(16, 4096, 32, 0, 0, 8, 65536).unwrap();
    let mut dom = Dom::new(limits).unwrap();
    let mut batch = DomBatch::new();
    let document = batch.create_document();
    let dialog = batch.create_element_ns(&document, Namespace::html(), "dialog");
    batch.insert_before(&dialog, &document, None);
    let commit = dom.apply(batch, &AtomicBool::new(false)).unwrap();
    let document = commit.created(&document).unwrap().unwrap();
    let dialog = commit.created(&dialog).unwrap().unwrap();
    let snapshot = dom.snapshot();
    let node = snapshot.node(dialog).unwrap();
    assert_eq!(node.parent(), Some(document));
    assert_eq!(node.node_name(), DocumentString::from("dialog"));
    let element = node.as_element().unwrap();
    assert_eq!(element.name().namespace(), &Namespace::html());
    assert_eq!(element.nonce(), Some(&DocumentString::from("")));
    assert!(element.common_state().element().reaction_queue_is_empty());
    assert!(
        !element
            .common_state()
            .html()
            .unwrap()
            .has_previously_focused_element()
    );
}
