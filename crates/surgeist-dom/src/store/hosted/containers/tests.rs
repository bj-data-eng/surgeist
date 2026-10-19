//! Independent raw participants and owning candidate rollback.
use super::*;
fn cancel() -> AtomicBool {
    AtomicBool::new(false)
}
fn limits() -> DomLimits {
    DomLimits::new(
        128,
        65536,
        1024,
        0,
        0,
        16,
        16_000_000,
        crate::DomNameLimits::new(16_777_216, 16_384, 4_000_000, 1_048_576).unwrap(),
    )
    .unwrap()
}
#[test]
fn actual_null_containers_keep_shadow_order_and_raw_payloads_across_snapshot_moves() {
    let mut dom = Dom::new(limits()).unwrap();
    let mut batch = DomBatch::new();
    let document = batch.create_document();
    let root = batch.create_element_ns(&document, crate::Namespace::html(), "div");
    let first = batch.create_element_ns(&document, crate::Namespace::html(), "iframe");
    batch.insert_before(&first, &root, None);
    let host = batch.create_element_ns(&document, crate::Namespace::html(), "div");
    batch.insert_before(&host, &root, None);
    let shadow = batch.attach_shadow(
        &host,
        crate::DomShadowRootInit::new(crate::HtmlShadowRootMode::Closed),
    );
    let in_shadow = batch.create_element_ns(&document, crate::Namespace::html(), "iframe");
    batch.insert_before(&in_shadow, &shadow, None);
    let light = batch.create_element_ns(&document, crate::Namespace::html(), "iframe");
    batch.insert_before(&light, &host, None);
    let commit = dom.apply(batch, &cancel()).unwrap();
    let key = |ticket: &CreatedNode| Key::Existing(commit.created(ticket).unwrap().unwrap());
    let (root, first, in_shadow, light) = (key(&root), key(&first), key(&in_shadow), key(&light));
    let snapshot = dom.snapshot();
    let index = &snapshot.state.container_index;
    assert_eq!(
        index.inspect_groups().unwrap(),
        vec![(root, vec![first, in_shadow, light])]
    );
    let graph = index.inspect_graph().unwrap();
    let records = graph.record_totals();
    assert_eq!(
        index.retained_usage().unwrap(),
        custody::Usage::new(graph.index_charge().bytes + records.bytes, records.units)
    );
    assert_eq!(index.inspect_facts().len(), 3);
    let mut draft = Draft::new(&dom, DomBatch::new().owner, &cancel()).unwrap();
    draft
        .relations_remove(root, RelationField::Children, 0, 1, &cancel())
        .unwrap();
    draft
        .relations_insert(root, RelationField::Children, 1, &[first], &cancel())
        .unwrap();
    assert_eq!(
        draft.containers.current.index.inspect_groups().unwrap(),
        vec![(root, vec![in_shadow, light, first])]
    );
    assert_eq!(
        snapshot.state.container_index.inspect_groups().unwrap(),
        vec![(root, vec![first, in_shadow, light])]
    );
}
#[test]
fn source_checkpoint_restores_ordered_pair_and_disconnected_selected_container() {
    let mut dom = Dom::new(limits()).unwrap();
    let mut batch = DomBatch::new();
    let document = batch.create_document();
    let root = batch.create_element_ns(&document, crate::Namespace::html(), "div");
    let first = batch.create_element_ns(&document, crate::Namespace::html(), "iframe");
    let second = batch.create_element_ns(&document, crate::Namespace::html(), "iframe");
    batch.insert_before(&first, &root, None);
    batch.insert_before(&second, &root, None);
    let commit = dom.apply(batch, &cancel()).unwrap();
    let key = |ticket: &CreatedNode| Key::Existing(commit.created(ticket).unwrap().unwrap());
    let (root, first, second) = (key(&root), key(&first), key(&second));
    let empty = DomBatch::new();
    let mut draft = Draft::new_source(
        &dom,
        empty.owner,
        &empty.commands,
        crate::DomSourceLimits::new(100_000_000, 1_000_000, 10_000_000, 32, 1024, 1024, 1),
        &cancel(),
    )
    .unwrap()
    .0;
    let checkpoint = draft
        .source_checkpoint(
            draft.source_other_bytes,
            draft.source_other_units,
            &cancel(),
        )
        .unwrap();
    draft
        .relations_remove(root, RelationField::Children, 0, 1, &cancel())
        .unwrap();
    assert_eq!(
        draft.containers.current.index.inspect_groups().unwrap(),
        vec![(root, vec![second]), (first, vec![first])]
    );
    checkpoint.restore(&mut draft);
    assert_eq!(
        draft.containers.current.index.inspect_groups().unwrap(),
        vec![(root, vec![first, second])]
    );
    assert_eq!(draft.slot_index.position(first).unwrap().parent, root);
}
