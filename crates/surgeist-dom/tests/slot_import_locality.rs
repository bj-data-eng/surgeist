//! Source import excludes non-Slot descendants from assignment discovery.
//! Fixed Slot participants keep discovery work bounded at each insertion hook.
use std::sync::atomic::AtomicBool;
use surgeist_dom::{
    DocumentString, Dom, DomBatch, DomLimits, DomNodeKind, DomSourceEdit, DomSourceLimits,
    DomSourceOutput, DomSourceProgress, Namespace,
};

const COMMENTS: usize = 512;
const IMPORT_WORK: u64 = 2_000_000;

fn limits() -> DomLimits {
    DomLimits::new(
        2048,
        1_000_000,
        8192,
        8192,
        0,
        8,
        64_000_000,
        surgeist_dom::DomNameLimits::new(16_777_216, 16_384, 4_000_000, 1_048_576).unwrap(),
    )
    .unwrap()
}

fn import_comments_with_slots(slots: usize, work: u64) {
    let cancel = AtomicBool::new(false);
    let mut input = Dom::new(limits()).unwrap();
    let mut batch = DomBatch::new();
    let document = batch.create_document();
    let root = batch.create_element(&document, "import-root");
    let mut expected_slots = Vec::new();
    for index in 0..COMMENTS {
        if slots != 0 && index % (COMMENTS / slots) == 0 {
            let slot = batch.create_element_ns(&document, Namespace::html(), "slot");
            batch.set_attribute(&slot, "name", "same");
            batch.insert_before(&slot, &root, None);
            expected_slots.push(true);
        }
        let comment = batch.create_comment(&document, "payload");
        batch.insert_before(&comment, &root, None);
        expected_slots.push(false);
    }
    let commit = input.apply(batch, &cancel).unwrap();
    let root = commit.created(&root).unwrap().unwrap();
    let snapshot = input.snapshot();
    assert_eq!(
        snapshot.node(root).unwrap().children().len(),
        COMMENTS + slots
    );
    drop(input);

    let mut target = Dom::new(limits()).unwrap();
    let mut batch = DomBatch::new();
    let document = batch.create_document();
    let commit = target.apply(batch, &cancel).unwrap();
    let document = commit.created(&document).unwrap().unwrap();
    let mut lease = target
        .begin_source_edit(
            DomSourceEdit::import_node(&snapshot, root, document, true),
            DomSourceLimits::new(500_000_000, 10_000_000, work, 8, 4096, 4096, 1),
            &cancel,
        )
        .unwrap_or_else(|error| {
            panic!("public import begin: slots={slots} work={work} error={error:?}")
        });
    let progress = lease.advance(&cancel);
    if let Err(error) = &progress {
        eprintln!(
            "real public import refused: slots={slots} status={:?} error={error:?}",
            lease.status()
        );
    }
    let DomSourceProgress::Complete(done) = progress.unwrap() else {
        panic!("generic XML/Slot import requires no external service");
    };
    let imported = done.published_node().unwrap();
    assert_eq!(done.output(), Some(DomSourceOutput::Node(imported)));
    drop(lease);
    let current = target.snapshot();
    let imported_root = current.node(imported).unwrap();
    assert_eq!(imported_root.parent(), None);
    assert_eq!(imported_root.owner_document().unwrap().node(), document);
    assert_eq!(imported_root.children().len(), expected_slots.len());
    for (&child, expected_slot) in imported_root.children().iter().zip(expected_slots) {
        let node = current.node(child).unwrap();
        assert_eq!(node.parent(), Some(imported));
        if expected_slot {
            assert_eq!(node.node_name(), DocumentString::from("slot"));
            assert!(current.slot(child).unwrap().assigned_nodes().is_empty());
        } else {
            assert!(matches!(node.kind(), DomNodeKind::Comment(_)));
            assert_eq!(node.data(), Some(&DocumentString::from("payload")));
        }
    }
    assert!(current.pending_slot_signals().is_empty());
}

#[test]
fn source_import_comment_siblings_with_no_slots_fits_finite_work() {
    import_comments_with_slots(0, IMPORT_WORK);
}

#[test]
fn source_import_comment_siblings_with_two_slots_fits_same_finite_work() {
    import_comments_with_slots(2, IMPORT_WORK);
}

#[test]
fn source_import_growing_slot_participants_preserves_tree_order_and_empty_assignment() {
    // Actual Slot work across insertion hooks may grow quadratically. This
    // semantic witness deliberately has no fixed-participant work expectation.
    import_comments_with_slots(64, 100_000_000);
}
