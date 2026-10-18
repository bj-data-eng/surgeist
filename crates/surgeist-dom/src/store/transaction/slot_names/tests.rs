//! New private-index transition coverage; public named-discovery REDs live in
//! crate::slot::named_discovery_tests at their unchanged owning boundary.
use super::*;

fn fixture() -> (Dom, Key, [Key; 2]) {
    let cancel = AtomicBool::new(false);
    let mut dom = Dom::new(DomLimits::new(64, 4096, 128, 64, 0, 0, 0).unwrap()).unwrap();
    let mut batch = DomBatch::new();
    let document = batch.create_document();
    let root = batch.create_element(&document, "root");
    let a = batch.create_element_ns(&document, crate::Namespace::html(), "slot");
    let b = batch.create_element_ns(&document, crate::Namespace::html(), "slot");
    batch.insert_before(&a, &root, None);
    batch.insert_before(&b, &root, None);
    let commit = dom.apply(batch, &cancel).unwrap();
    let id = |ticket: &CreatedNode| Key::Existing(commit.created(ticket).unwrap().unwrap());
    (dom, id(&root), [id(&a), id(&b)])
}
fn source(dom: &Dom, cancel: &AtomicBool) -> Draft {
    let batch = DomBatch::new();
    Draft::new_source(
        dom,
        batch.owner,
        &batch.commands,
        crate::DomSourceLimits::new(500_000_000, 10_000_000, 100_000_000, 64, 4096, 4096, 1),
        cancel,
    )
    .unwrap()
    .0
}
fn members(index: &Index, scope: Key) -> Vec<Key> {
    index
        .inspect_groups()
        .unwrap()
        .into_iter()
        .find(|group| group.kind == 0 && group.scope == scope && group.value.is_none())
        .map_or_else(Vec::new, |group| group.members)
}

#[test]
fn relation_reorder_and_checkpoint_restore_named_groups_without_native_refresh() {
    let (dom, root, children) = fixture();
    let cancel = AtomicBool::new(false);
    let mut draft = source(&dom, &cancel);
    assert_eq!(members(&dom.state.slot_index.named, root), children);
    let checkpoint = draft
        .source_checkpoint(draft.source_other_bytes, draft.source_other_units, &cancel)
        .unwrap();
    draft
        .relations_replace(
            root,
            super::super::RelationField::Children,
            children.into_iter().rev().collect(),
            &cancel,
        )
        .unwrap();
    assert_eq!(
        members(&draft.slot_names.current.value, root),
        [children[1], children[0]]
    );
    checkpoint.restore(&mut draft);
    assert_eq!(members(&draft.slot_names.current.value, root), children);
    assert_eq!(
        draft
            .node(root)
            .unwrap()
            .children
            .iter()
            .collect::<Vec<_>>(),
        children
    );
}

#[test]
fn restored_order_and_unpublished_slot_retirement_publish_the_original_root() {
    let (mut dom, root, children) = fixture();
    let cancel = AtomicBool::new(false);
    let original = dom.state.clone();
    let stamp = dom.snapshot().stamp();
    let mut draft = source(&dom, &cancel);
    draft
        .relations_replace(
            root,
            super::super::RelationField::Children,
            vec![children[1], children[0]],
            &cancel,
        )
        .unwrap();
    draft
        .relations_replace(
            root,
            super::super::RelationField::Children,
            children.to_vec(),
            &cancel,
        )
        .unwrap();
    let document = draft.node(root).unwrap().document;
    let kind = draft.node(children[0]).unwrap().kind.clone();
    let temporary = draft
        .allocate(kind, document, dom.limits(), &cancel)
        .unwrap();
    draft.retire_node(temporary, &cancel).unwrap();
    assert_eq!(members(&draft.slot_names.current.value, root), children);
    let prepared = draft.finish(&dom, &cancel).unwrap();
    assert!(!prepared.semantic_changed);
    let commit = dom.publish(prepared, &cancel).unwrap();
    assert!(commit.changes().changes().is_empty());
    assert_eq!(dom.snapshot().stamp(), stamp);
    assert!(Arc::ptr_eq(&dom.state, &original));
    assert_eq!(members(&dom.state.slot_index.named, root), children);
}
