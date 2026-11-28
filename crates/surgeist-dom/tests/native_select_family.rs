//! DOM3071 createElementNS and HTML877d native Select-family initialization.
use std::sync::atomic::AtomicBool;
use surgeist_dom::{
    Dom, DomBatch, DomError, DomSnapshot,
    identity::NodeId,
    limits::{DomLimits, DomResource},
    shadow::DomShadowRootInit,
};
use surgeist_html::construction::HtmlShadowRootMode;
use surgeist_html::values::{DocumentString, Namespace};

fn dom(nodes: usize) -> Dom {
    Dom::new(
        DomLimits::new(
            nodes,
            1024,
            16,
            0,
            0,
            4,
            4096,
            surgeist_dom::limits::DomNameLimits::new(16_777_216, 16_384, 4_000_000, 1_048_576)
                .unwrap(),
        )
        .unwrap(),
    )
    .unwrap()
}

fn native_element(local_name: &str) {
    let mut dom = dom(8);
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
        node.as_element().unwrap().name().namespace_uri(),
        Namespace::html().uri()
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

fn fallback_tree(snapshot: &DomSnapshot, select: NodeId) -> [NodeId; 3] {
    let select = snapshot.node(select).unwrap();
    let root = select
        .hosted_shadow_root()
        .expect("a native Select owns the UA fallback tree");
    assert_eq!(select.shadow_root(), None);
    let shadow = snapshot.shadow_root(root).unwrap();
    assert_eq!(shadow.host(), select.id());
    assert_eq!(shadow.state().mode(), HtmlShadowRootMode::Closed);
    let button_slot = snapshot.node(shadow.node().children()[0]).unwrap();
    assert_eq!(button_slot.node_name(), DocumentString::from("slot"));
    assert_eq!(button_slot.children().len(), 1);
    let fallback = snapshot.node(button_slot.children()[0]).unwrap();
    assert_eq!(fallback.node_name(), DocumentString::from("div"));
    assert_eq!(fallback.parent(), Some(button_slot.id()));
    assert!(
        [shadow.node(), button_slot, fallback]
            .iter()
            .all(|node| node.owner_document() == select.owner_document())
    );
    [root, button_slot.id(), fallback.id()]
}

#[test]
fn native_select_fallback_keeps_its_identity_across_mode_changes() {
    let mut dom = dom(24);
    let mut batch = DomBatch::new();
    let document = batch.create_document();
    let select = batch.create_element_ns(&document, Namespace::html(), "select");
    let done = dom.apply(batch, &AtomicBool::new(false)).unwrap();
    let select = done.created(&select).unwrap().unwrap();
    let original = dom.snapshot();
    let fallback = fallback_tree(&original, select);
    assert_eq!(original.node(select).unwrap().children(), []);
    let mut batch = DomBatch::new();
    batch.set_attribute(select, "multiple", "");
    batch.set_attribute(select, "size", "1");
    dom.apply(batch, &AtomicBool::new(false)).unwrap();
    let changed = dom.snapshot();
    assert_eq!(fallback_tree(&changed, select), fallback);
    assert_eq!(
        changed
            .node(select)
            .unwrap()
            .tree_order()
            .collect::<Vec<_>>(),
        [select]
    );
    let mut batch = DomBatch::new();
    batch.attach_shadow(select, DomShadowRootInit::new(HtmlShadowRootMode::Open));
    assert_eq!(
        dom.apply(batch, &AtomicBool::new(false)).unwrap_err(),
        DomError::NotSupported
    );
    assert_eq!(dom.snapshot().stamp(), changed.stamp());
}

#[test]
fn select_fallback_follows_adoption_and_is_fresh_on_clone_and_import() {
    let mut dom = dom(48);
    let mut batch = DomBatch::new();
    let from = batch.create_document();
    let to = batch.create_document();
    let select = batch.create_element_ns(&from, Namespace::html(), "select");
    let done = dom.apply(batch, &AtomicBool::new(false)).unwrap();
    let from = done.document(&from).unwrap();
    let to = done.document(&to).unwrap();
    let select = done.created(&select).unwrap().unwrap();
    let original = dom.snapshot();
    let fallback = fallback_tree(&original, select);
    let mut batch = DomBatch::new();
    batch.adopt(select, to);
    let clone = batch.clone_node(select, false);
    let imported = batch.import_node(&original, select, from, true);
    let done = dom.apply(batch, &AtomicBool::new(false)).unwrap();
    let live = dom.snapshot();
    assert_eq!(fallback_tree(&live, select), fallback);
    assert_eq!(live.node(fallback[2]).unwrap().owner_document(), Some(to));
    assert_eq!(
        original.node(fallback[2]).unwrap().owner_document(),
        Some(from)
    );
    let clone = fallback_tree(&live, done.created(&clone).unwrap().unwrap());
    let imported = fallback_tree(&live, done.created(&imported).unwrap().unwrap());
    for id in clone {
        assert!(!fallback.contains(&id));
        assert!(!imported.contains(&id));
    }
    for id in imported {
        assert!(!fallback.contains(&id));
    }
}

#[test]
fn failure_to_allocate_the_select_fallback_publishes_nothing() {
    let mut dom = dom(4);
    let before = dom.snapshot();
    let usage = dom.usage();
    let mut batch = DomBatch::new();
    let document = batch.create_document();
    batch.create_element_ns(&document, Namespace::html(), "select");
    assert_eq!(
        dom.apply(batch, &AtomicBool::new(false)).unwrap_err(),
        DomError::ResourceLimit {
            resource: DomResource::ManagedNodes,
            limit: 4,
        }
    );
    assert_eq!(dom.snapshot().stamp(), before.stamp());
    assert_eq!(dom.usage(), usage);
}
