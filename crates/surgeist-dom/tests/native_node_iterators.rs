//! Native cursor transitions from the pinned DOM NodeIterator algorithm.

use std::sync::atomic::AtomicBool;
use surgeist_dom::{
    DocumentId, Dom, DomBatch, DomLimits, DomNameLimits, DomNodeIteratorFilter, DomNodeIteratorId,
    DomNodeIteratorProgress, Namespace, NodeId,
};

struct Tree {
    dom: Dom,
    second_document: DocumentId,
    root: NodeId,
    first: NodeId,
    removed: NodeId,
    nested: NodeId,
    last: NodeId,
}

fn tree() -> Tree {
    let names = DomNameLimits::new(16_777_216, 16_384, 4_000_000, 1_048_576).unwrap();
    let limits = DomLimits::new(32, 4096, 64, 0, 0, 8, 1_000_000, names)
        .unwrap()
        .with_node_iterators(8);
    let mut dom = Dom::new(limits).unwrap();
    let mut batch = DomBatch::new();
    let document = batch.create_document();
    let second_document = batch.create_document();
    let root = batch.create_element_ns(&document, Namespace::html(), "div");
    let first = batch.create_element_ns(&document, Namespace::html(), "p");
    let removed = batch.create_element_ns(&document, Namespace::html(), "section");
    let nested = batch.create_text(&document, "nested");
    let last = batch.create_comment(&document, "last");
    batch.insert_before(&root, &document, None);
    batch.insert_before(&first, &root, None);
    batch.insert_before(&removed, &root, None);
    batch.insert_before(&nested, &removed, None);
    batch.insert_before(&last, &root, None);
    let commit = dom.apply(batch, &AtomicBool::new(false)).unwrap();
    Tree {
        dom,
        second_document: commit.document(&second_document).unwrap(),
        root: commit.created(&root).unwrap().unwrap(),
        first: commit.created(&first).unwrap().unwrap(),
        removed: commit.created(&removed).unwrap().unwrap(),
        nested: commit.created(&nested).unwrap().unwrap(),
        last: commit.created(&last).unwrap().unwrap(),
    }
}

fn iterator(dom: &mut Dom, root: NodeId, mask: u32) -> DomNodeIteratorId {
    dom.create_node_iterator(
        root,
        mask,
        DomNodeIteratorFilter::None,
        &AtomicBool::new(false),
    )
    .unwrap()
}

fn step(dom: &mut Dom, id: DomNodeIteratorId, next: bool) -> Option<NodeId> {
    let cancel = AtomicBool::new(false);
    let result = if next {
        dom.node_iterator_next(id, 256, &cancel)
    } else {
        dom.node_iterator_previous(id, 256, &cancel)
    };
    let DomNodeIteratorProgress::Complete(result) = result.unwrap() else {
        panic!("the null filter cannot request a callback");
    };
    result
}

fn reference(dom: &Dom, id: DomNodeIteratorId) -> (NodeId, bool) {
    let cursor = dom.node_iterator(id).unwrap();
    (
        cursor.reference_node(),
        cursor.pointer_before_reference_node(),
    )
}

#[test]
fn traverse_checks_the_mask_without_pruning_children_and_keeps_boundary_position() {
    let Tree {
        mut dom,
        root,
        nested,
        ..
    } = tree();
    const SHOW_TEXT: u32 = 1 << 2;
    let id = iterator(&mut dom, root, SHOW_TEXT);
    assert_eq!(reference(&dom, id), (root, true));
    assert_eq!(step(&mut dom, id, false), None);
    assert_eq!(step(&mut dom, id, true), Some(nested));
    assert_eq!(reference(&dom, id), (nested, false));
    assert_eq!(step(&mut dom, id, true), None);
    assert_eq!(reference(&dom, id), (nested, false));
    assert_eq!(step(&mut dom, id, false), Some(nested));
    assert_eq!(step(&mut dom, id, false), None);
    assert_eq!(reference(&dom, id), (nested, true));
}

#[test]
fn pre_remove_relocates_both_pointer_directions_and_preserves_a_contained_root() {
    let Tree {
        mut dom,
        root,
        first,
        removed,
        nested,
        last,
        ..
    } = tree();
    let before = iterator(&mut dom, root, u32::MAX);
    let after = iterator(&mut dom, root, u32::MAX);
    let inside = iterator(&mut dom, removed, u32::MAX);
    for expected in [root, first, removed, nested] {
        assert_eq!(step(&mut dom, before, true), Some(expected));
        assert_eq!(step(&mut dom, after, true), Some(expected));
    }
    assert_eq!(step(&mut dom, before, false), Some(nested));
    assert_eq!(step(&mut dom, inside, true), Some(removed));
    assert_eq!(step(&mut dom, inside, true), Some(nested));

    let mut batch = DomBatch::new();
    batch.detach(removed);
    dom.apply(batch, &AtomicBool::new(false)).unwrap();
    assert_eq!(reference(&dom, before), (last, true));
    assert_eq!(reference(&dom, after), (first, false));
    assert_eq!(reference(&dom, inside), (nested, false));
    assert_eq!(step(&mut dom, before, true), Some(last));
    assert_eq!(step(&mut dom, after, true), Some(last));
}

#[test]
fn adoption_preserves_root_and_uses_its_new_document_for_later_removals() {
    let Tree {
        mut dom,
        second_document,
        root,
        first,
        removed,
        nested,
        last,
    } = tree();
    let id = iterator(&mut dom, root, u32::MAX);
    for expected in [root, first, removed, nested, last] {
        assert_eq!(step(&mut dom, id, true), Some(expected));
    }
    assert_eq!(step(&mut dom, id, false), Some(last));

    let mut adoption = DomBatch::new();
    adoption.adopt(root, second_document);
    dom.apply(adoption, &AtomicBool::new(false)).unwrap();
    assert_eq!(dom.node_iterator(id).unwrap().root(), root);
    assert_eq!(reference(&dom, id), (last, true));

    let mut detach = DomBatch::new();
    detach.detach(last);
    dom.apply(detach, &AtomicBool::new(false)).unwrap();
    assert_eq!(reference(&dom, id), (nested, false));
    assert_eq!(step(&mut dom, id, false), Some(nested));
}
