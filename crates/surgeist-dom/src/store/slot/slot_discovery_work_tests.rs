//! Slot discovery excludes unrelated ordinary Comment descendants.
//! Public setup and real Source formation precede the selected work window.
use crate::store::Draft;
use crate::{Dom, DomBatch, DomLimits, DomSourceLimits, Namespace, NodeId};
use std::sync::atomic::{AtomicBool, Ordering};

fn fixture(comments: usize, slots: usize) -> (Dom, NodeId) {
    let cancel = AtomicBool::new(false);
    let mut dom =
        Dom::new(DomLimits::new(2048, 1_000_000, 8192, 8192, 0, 8, 64_000_000).unwrap()).unwrap();
    let mut batch = DomBatch::new();
    let document = batch.create_document();
    let root = batch.create_element(&document, "root");
    for _ in 0..slots {
        let slot = batch.create_element_ns(&document, Namespace::html(), "slot");
        batch.insert_before(&slot, &root, None);
    }
    for _ in 0..comments {
        let comment = batch.create_comment(&document, "payload");
        batch.insert_before(&comment, &root, None);
    }
    let commit = dom.apply(batch, &cancel).unwrap();
    let root = commit.created(&root).unwrap().unwrap();
    (dom, root)
}

fn selected_assignment(comments: usize, slots: usize) {
    let cancel = AtomicBool::new(false);
    let (dom, root) = fixture(comments, slots);
    let batch = DomBatch::new();
    let mut draft = Draft::new_source(
        &dom,
        batch.owner,
        &batch.commands,
        DomSourceLimits::new(500_000_000, 10_000_000, 100_000_000, 8, 4096, 4096, 1),
        &cancel,
    )
    .unwrap()
    .0;
    let before = draft.source_work.load(Ordering::Relaxed);
    let allowance = 256;
    draft.source_limits.as_mut().unwrap().work = before + allowance;
    let root = draft.resolve(root.into()).unwrap();
    let result = draft.assign_tree(root, dom.limits(), &cancel);
    let spent = draft.source_work.load(Ordering::Relaxed) - before;
    eprintln!(
        "selected actual assignment comments={comments} slots={slots} work={spent} result={result:?}"
    );
    assert_eq!(
        result,
        Ok(()),
        "fixed Slot discovery excludes non-Slot descendants"
    );
    assert!(spent <= allowance);
    assert!(draft.signal_slots.is_empty());
    assert_eq!(draft.node(root).unwrap().children.len(), comments + slots);
}

#[test]
fn source_assignment_with_no_slots_does_not_visit_comment_siblings() {
    for comments in [0, 8, 128, 512] {
        selected_assignment(comments, 0);
    }
}

#[test]
fn source_assignment_with_two_slots_does_not_visit_comment_siblings() {
    for comments in [0, 8, 128, 512] {
        selected_assignment(comments, 2);
    }
}
