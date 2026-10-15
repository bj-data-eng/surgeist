use super::*;

fn dom() -> Dom {
    Dom::new(DomLimits::new(32, 256, 64, 8, 8192).unwrap()).unwrap()
}

#[test]
fn cancelled_prepared_changes_leave_slots_generations_and_history_untouched() {
    let mut dom = dom();
    let cancel = AtomicBool::new(false);
    let mut initial = DomBatch::new();
    let document = initial.create_document();
    let old = initial.create_comment(&document, "existing");
    let initial = dom.apply(initial, &cancel).unwrap();
    let document = initial.document(&document).unwrap();
    let old = initial.created(&old).unwrap().unwrap();
    let before = dom.snapshot();
    let usage = dom.usage();
    let mut edit = DomBatch::new();
    edit.destroy(old);
    edit.create_document();
    edit.create_text(document, "not published");
    let prepared = dom.prepare(edit, &cancel).unwrap();
    cancel.store(true, Ordering::Release);
    assert_eq!(
        dom.publish(prepared, &cancel).unwrap_err(),
        DomError::Cancelled
    );
    assert_eq!(*dom.state, *before.state);
    assert_eq!(dom.snapshot().stamp(), before.stamp());
    assert_eq!(dom.usage(), usage);
}

#[test]
fn cancellation_at_publication_rolls_back_element_attributes_and_subtree_adoption() {
    let mut dom = dom();
    let cancel = AtomicBool::new(false);
    let mut initial = DomBatch::new();
    let source = initial.create_document();
    let target = initial.create_document();
    let root = initial.create_element(&source, "root");
    let text = initial.create_text(&source, "old");
    initial.set_attribute(&root, "kept", "old");
    initial.insert_before(&text, &root, None);
    initial.insert_before(&root, &source, None);
    let initial = dom.apply(initial, &cancel).unwrap();
    let target = initial.document(&target).unwrap();
    let root = initial.created(&root).unwrap().unwrap();
    let before = dom.snapshot();
    let usage = dom.usage();
    let mut edit = DomBatch::new();
    edit.set_attribute(root, "kept", "unpublished");
    edit.adopt(root, target);
    let transient = edit.create_element(target, "unpublished");
    edit.set_attribute(&transient, "new", "new");
    let prepared = dom.prepare(edit, &cancel).unwrap();
    cancel.store(true, Ordering::Release);
    assert_eq!(
        dom.publish(prepared, &cancel).unwrap_err(),
        DomError::Cancelled
    );
    assert_eq!(*dom.state, *before.state);
    assert_eq!(dom.snapshot().stamp(), before.stamp());
    assert_eq!(dom.usage(), usage);
    assert!(
        dom.changes_since(before.stamp())
            .unwrap()
            .changes()
            .is_empty()
    );
}

#[test]
fn unpublished_create_destroy_does_not_consume_a_slot_or_generation() {
    let mut dom = dom();
    let cancel = AtomicBool::new(false);
    let mut batch = DomBatch::new();
    let document = batch.create_document();
    let document = dom
        .apply(batch, &cancel)
        .unwrap()
        .document(&document)
        .unwrap();
    let before = dom.snapshot();
    let mut batch = DomBatch::new();
    let transient = batch.create_text(document, "temporary");
    batch.destroy(&transient);
    dom.apply(batch, &cancel).unwrap();
    assert_eq!(*dom.state, *before.state);
    let mut batch = DomBatch::new();
    let text = batch.create_text(document, "published");
    let text = dom
        .apply(batch, &cancel)
        .unwrap()
        .created(&text)
        .unwrap()
        .unwrap();
    assert_eq!(text.slot, 1);
    assert_eq!(text.generation, 0);
}

#[test]
fn revision_exhaustion_rejects_changed_batches_but_allows_noops() {
    let mut dom = dom();
    dom.revision = DomRevision(u64::MAX);
    let cancel = AtomicBool::new(false);
    let mut batch = DomBatch::new();
    batch.create_document();
    assert_eq!(
        dom.apply(batch, &cancel).unwrap_err(),
        DomError::RevisionExhausted
    );
    assert_eq!(dom.state.nodes, 0);
    assert!(dom.state.slots.is_empty());
    assert_eq!(dom.usage().history_entries(), 0);
    assert!(
        dom.apply(DomBatch::new(), &cancel)
            .unwrap()
            .changes()
            .changes()
            .is_empty()
    );
    assert_eq!(dom.revision, DomRevision(u64::MAX));
}

#[test]
fn generation_exhaustion_retires_the_slot_instead_of_reusing_identity() {
    let mut dom = dom();
    let cancel = AtomicBool::new(false);
    let mut batch = DomBatch::new();
    let document = batch.create_document();
    let document = dom
        .apply(batch, &cancel)
        .unwrap()
        .document(&document)
        .unwrap();
    let mut batch = DomBatch::new();
    let text = batch.create_text(document, "old");
    let text = dom
        .apply(batch, &cancel)
        .unwrap()
        .created(&text)
        .unwrap()
        .unwrap();
    Arc::get_mut(&mut dom.state).unwrap().slots[text.slot].generation = u64::MAX;
    let exhausted = NodeId {
        generation: u64::MAX,
        ..text
    };
    let mut batch = DomBatch::new();
    batch.destroy(exhausted);
    let fresh = batch.create_text(document, "new");
    let fresh = dom
        .apply(batch, &cancel)
        .unwrap()
        .created(&fresh)
        .unwrap()
        .unwrap();
    assert_ne!(fresh.slot, exhausted.slot);
    assert!(dom.state.slots[exhausted.slot].retired);
    assert_eq!(
        dom.snapshot().node(exhausted).unwrap_err(),
        DomError::StaleNode
    );
}
