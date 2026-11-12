//! Native child-list records from the pinned DOM tree and observer algorithms.

use std::sync::atomic::AtomicBool;
use surgeist_dom::{
    DocumentId, Dom, DomBatch, DomChildListObservationLimits, DomChildListObserverId,
    DomChildListRecord, DomLimits, DomNameLimits, Namespace, NodeId,
};

struct Tree {
    dom: Dom,
    document: DocumentId,
    root: NodeId,
    first: NodeId,
    branch: NodeId,
    nested: NodeId,
    last: NodeId,
}

fn tree() -> Tree {
    let cancel = AtomicBool::new(false);
    let names = DomNameLimits::new(16_777_216, 16_384, 4_000_000, 1_048_576).unwrap();
    let observations = DomChildListObservationLimits::new(8, 32, 32, 256).unwrap();
    let limits = DomLimits::new(64, 4096, 128, 0, 0, 64, 1_000_000, names)
        .unwrap()
        .with_child_list_observation(observations);
    let mut dom = Dom::new(limits).unwrap();
    let mut batch = DomBatch::new();
    let document = batch.create_document();
    let root = batch.create_element_ns(&document, Namespace::html(), "div");
    let first = batch.create_element_ns(&document, Namespace::html(), "p");
    let branch = batch.create_element_ns(&document, Namespace::html(), "section");
    let nested = batch.create_element_ns(&document, Namespace::html(), "span");
    let last = batch.create_comment(&document, "last");
    batch.insert_before(&root, &document, None);
    batch.insert_before(&first, &root, None);
    batch.insert_before(&branch, &root, None);
    batch.insert_before(&nested, &branch, None);
    batch.insert_before(&last, &root, None);
    let commit = dom.apply(batch, &cancel).unwrap();
    Tree {
        dom,
        document: commit.document(&document).unwrap(),
        root: commit.created(&root).unwrap().unwrap(),
        first: commit.created(&first).unwrap().unwrap(),
        branch: commit.created(&branch).unwrap().unwrap(),
        nested: commit.created(&nested).unwrap().unwrap(),
        last: commit.created(&last).unwrap().unwrap(),
    }
}

fn observe(dom: &mut Dom, root: NodeId, subtree: bool) -> DomChildListObserverId {
    let cancel = AtomicBool::new(false);
    let id = dom.create_child_list_observer(&cancel).unwrap();
    dom.observe_child_list(id, root, subtree, &cancel).unwrap();
    id
}

fn assert_record(
    record: &DomChildListRecord,
    target: NodeId,
    added: &[NodeId],
    removed: &[NodeId],
    previous: Option<NodeId>,
    next: Option<NodeId>,
) {
    assert_eq!(record.target(), target);
    assert_eq!(record.added_nodes(), added);
    assert_eq!(record.removed_nodes(), removed);
    assert_eq!(record.previous_sibling(), previous);
    assert_eq!(record.next_sibling(), next);
}

#[test]
fn take_records_keeps_detached_subtree_registration_until_cleanup() {
    let Tree {
        mut dom,
        document,
        root,
        first,
        branch,
        nested,
        last,
    } = tree();
    let cancel = AtomicBool::new(false);
    let direct = observe(&mut dom, root, false);
    let subtree = observe(&mut dom, root, true);
    let mut batch = DomBatch::new();
    batch.detach(branch);
    dom.apply(batch, &cancel).unwrap();
    for observer in [direct, subtree] {
        let records = dom.take_child_list_records(observer, &cancel).unwrap();
        assert_eq!(records.len(), 1);
        assert_record(&records[0], root, &[], &[branch], Some(first), Some(last));
    }

    let mut batch = DomBatch::new();
    let text = batch.create_text(document, "detached");
    batch.insert_before(&text, nested, None);
    let text = dom
        .apply(batch, &cancel)
        .unwrap()
        .created(&text)
        .unwrap()
        .unwrap();
    assert!(
        dom.take_child_list_records(direct, &cancel)
            .unwrap()
            .is_empty()
    );
    let records = dom.take_child_list_records(subtree, &cancel).unwrap();
    assert_eq!(records.len(), 1);
    assert_record(&records[0], nested, &[text], &[], None, None);

    dom.disconnect_child_list_observer(subtree, &cancel)
        .unwrap();
    let mut batch = DomBatch::new();
    batch.detach(text);
    dom.apply(batch, &cancel).unwrap();
    assert!(
        dom.take_child_list_records(subtree, &cancel)
            .unwrap()
            .is_empty()
    );
}

#[test]
fn same_parent_insertion_reports_removal_then_insertion_with_old_and_new_neighbors() {
    let Tree {
        mut dom,
        root,
        first,
        branch,
        last,
        ..
    } = tree();
    let cancel = AtomicBool::new(false);
    let observer = observe(&mut dom, root, false);
    let mut batch = DomBatch::new();
    batch.insert_before(branch, root, None);
    dom.apply(batch, &cancel).unwrap();
    let records = dom.take_child_list_records(observer, &cancel).unwrap();
    assert_eq!(records.len(), 2);
    assert_record(&records[0], root, &[], &[branch], Some(first), Some(last));
    assert_record(&records[1], root, &[branch], &[], Some(last), None);
}

#[test]
fn fragment_transfer_reports_two_aggregate_records_in_source_order() {
    let Tree {
        mut dom,
        document,
        root,
        first,
        branch,
        ..
    } = tree();
    let cancel = AtomicBool::new(false);
    let mut batch = DomBatch::new();
    let fragment = batch.create_fragment(document);
    let one = batch.create_text(document, "one");
    let two = batch.create_comment(document, "two");
    batch.insert_before(&one, &fragment, None);
    batch.insert_before(&two, &fragment, None);
    let commit = dom.apply(batch, &cancel).unwrap();
    let fragment = commit.created(&fragment).unwrap().unwrap();
    let one = commit.created(&one).unwrap().unwrap();
    let two = commit.created(&two).unwrap().unwrap();

    let observer = observe(&mut dom, fragment, false);
    dom.observe_child_list(observer, root, false, &cancel)
        .unwrap();
    let mut batch = DomBatch::new();
    batch.insert_before(fragment, root, Some(branch.into()));
    dom.apply(batch, &cancel).unwrap();
    let records = dom.take_child_list_records(observer, &cancel).unwrap();
    assert_eq!(records.len(), 2);
    assert_record(&records[0], fragment, &[], &[one, two], None, None);
    assert_record(
        &records[1],
        root,
        &[one, two],
        &[],
        Some(first),
        Some(branch),
    );
}
