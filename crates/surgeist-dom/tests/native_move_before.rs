//! Pinned DOM move and ParentNode.moveBefore semantics on the native Source.

use std::sync::atomic::AtomicBool;
use surgeist_dom::{
    DocumentId, Dom, DomBatch, DomChildListObservationLimits, DomChildListRecord, DomError,
    DomLimits, DomNameLimits, DomSourceEdit, DomSourceLimits, DomSourceProgress, NodeId,
};

struct Tree {
    dom: Dom,
    document: DocumentId,
    root: NodeId,
    left: NodeId,
    right: NodeId,
    moved: NodeId,
    tail: NodeId,
    reference: NodeId,
    detached: NodeId,
}

fn tree(connected: bool, cancel: &AtomicBool) -> Tree {
    let names = DomNameLimits::new(16_777_216, 16_384, 4_000_000, 1_048_576).unwrap();
    let observation = DomChildListObservationLimits::new(4, 16, 16, 96).unwrap();
    let limits = DomLimits::new(64, 4096, 128, 0, 0, 64, 1_000_000, names)
        .unwrap()
        .with_child_list_observation(observation);
    let mut dom = Dom::new(limits).unwrap();
    let mut batch = DomBatch::new();
    let document = batch.create_document();
    let root = batch.create_element(&document, "root");
    let left = batch.create_element(&document, "left");
    let right = batch.create_element(&document, "right");
    let moved = batch.create_element(&document, "moved");
    let tail = batch.create_text(&document, "tail");
    let reference = batch.create_comment(&document, "reference");
    let detached = batch.create_element(&document, "detached");
    if connected {
        batch.insert_before(&root, &document, None);
    }
    batch.insert_before(&left, &root, None);
    batch.insert_before(&right, &root, None);
    batch.insert_before(&moved, &left, None);
    batch.insert_before(&tail, &left, None);
    batch.insert_before(&reference, &right, None);
    let commit = dom.apply(batch, cancel).unwrap();
    Tree {
        dom,
        document: commit.document(&document).unwrap(),
        root: commit.created(&root).unwrap().unwrap(),
        left: commit.created(&left).unwrap().unwrap(),
        right: commit.created(&right).unwrap().unwrap(),
        moved: commit.created(&moved).unwrap().unwrap(),
        tail: commit.created(&tail).unwrap().unwrap(),
        reference: commit.created(&reference).unwrap().unwrap(),
        detached: commit.created(&detached).unwrap().unwrap(),
    }
}

fn policy() -> DomSourceLimits {
    DomSourceLimits::new(64_000_000, 1_000_000, 20_000_000, 128, 4096, 4096, 8)
}

fn move_node(
    dom: &mut Dom,
    node: NodeId,
    parent: NodeId,
    before: Option<NodeId>,
    cancel: &AtomicBool,
) -> Result<(), DomError> {
    let mut lease = dom.begin_source_edit(
        DomSourceEdit::move_before(node, parent, before),
        policy(),
        cancel,
    )?;
    match lease.advance(cancel)? {
        DomSourceProgress::Complete(done) => {
            assert_eq!(done.output(), None);
            Ok(())
        }
        DomSourceProgress::Failed(failure) => Err(failure.error().clone()),
        DomSourceProgress::Request(request) => panic!("unexpected move service: {request:?}"),
    }
}

fn assert_record(
    record: &DomChildListRecord,
    target: NodeId,
    added: &[NodeId],
    removed: &[NodeId],
    next: NodeId,
) {
    assert_eq!(record.target(), target);
    assert_eq!(record.added_nodes(), added);
    assert_eq!(record.removed_nodes(), removed);
    assert_eq!(record.previous_sibling(), None);
    assert_eq!(record.next_sibling(), Some(next));
}

#[test]
fn move_keeps_identity_and_emits_ordered_records_within_a_shared_root() {
    let cancel = AtomicBool::new(false);
    for connected in [false, true] {
        let Tree {
            mut dom,
            root,
            left,
            right,
            moved,
            tail,
            reference,
            ..
        } = tree(connected, &cancel);
        let observer = dom.create_child_list_observer(&cancel).unwrap();
        dom.observe_child_list(observer, root, true, &cancel)
            .unwrap();
        let before = dom.snapshot();
        let nodes = dom.usage().managed_nodes();

        move_node(&mut dom, moved, left, Some(moved), &cancel).unwrap();
        assert_eq!(dom.snapshot().node(left).unwrap().children(), [moved, tail]);
        let records = dom.take_child_list_records(observer, &cancel).unwrap();
        assert_eq!(records.len(), 2);
        assert_record(&records[0], left, &[], &[moved], tail);
        assert_record(&records[1], left, &[moved], &[], tail);

        move_node(&mut dom, moved, right, Some(reference), &cancel).unwrap();
        let after = dom.snapshot();
        assert_eq!(after.node(left).unwrap().children(), [tail]);
        assert_eq!(after.node(right).unwrap().children(), [moved, reference]);
        assert_eq!(after.node(moved).unwrap().parent(), Some(right));
        assert_eq!(before.node(moved).unwrap().parent(), Some(left));
        assert_eq!(dom.usage().managed_nodes(), nodes);
        let records = dom.take_child_list_records(observer, &cancel).unwrap();
        assert_eq!(records.len(), 2);
        assert_record(&records[0], left, &[], &[moved], tail);
        assert_record(&records[1], right, &[moved], &[], reference);
    }
}

#[test]
fn move_checks_root_and_ancestor_before_reference_then_document_hierarchy() {
    let cancel = AtomicBool::new(false);
    let Tree {
        mut dom,
        document,
        root,
        left,
        right,
        moved,
        tail,
        reference,
        detached,
    } = tree(true, &cancel);
    let before = dom.snapshot();
    let observer = dom.create_child_list_observer(&cancel).unwrap();
    dom.observe_child_list(observer, root, true, &cancel)
        .unwrap();
    let stamp = dom.snapshot().stamp();

    for (node, parent, before, expected) in [
        (detached, left, Some(reference), DomError::HierarchyRequest),
        (left, moved, Some(reference), DomError::HierarchyRequest),
        (tail, document.node(), Some(reference), DomError::NotFound),
        (
            tail,
            document.node(),
            Some(root),
            DomError::HierarchyRequest,
        ),
        (root, document.node(), None, DomError::HierarchyRequest),
        (moved, right, Some(tail), DomError::NotFound),
    ] {
        assert_eq!(
            move_node(&mut dom, node, parent, before, &cancel),
            Err(expected)
        );
        assert_eq!(dom.snapshot().stamp(), stamp);
    }
    assert_eq!(
        dom.snapshot().node(moved).unwrap().parent(),
        before.node(moved).unwrap().parent()
    );
    assert!(
        dom.take_child_list_records(observer, &cancel)
            .unwrap()
            .is_empty()
    );
}
