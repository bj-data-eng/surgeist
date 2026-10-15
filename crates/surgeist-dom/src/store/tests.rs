use super::*;

#[test]
fn cancelled_family_publication_preserves_dimensions_marquee_slots_history_and_allocations() {
    let mut dom = dom();
    let cancel = AtomicBool::new(false);
    let mut batch = DomBatch::new();
    let document = batch.create_document();
    let canvas = batch.create_element_ns(
        &document,
        crate::Namespace::from_uri("http://www.w3.org/1999/xhtml"),
        "canvas",
    );
    let moving = batch.create_element_ns(
        &document,
        crate::Namespace::from_uri("http://www.w3.org/1999/xhtml"),
        "marquee",
    );
    let commit = dom.apply(batch, &cancel).unwrap();
    let document = commit.document(&document).unwrap();
    let canvas = commit.created(&canvas).unwrap().unwrap();
    let moving = commit.created(&moving).unwrap().unwrap();
    let before = dom.snapshot();
    let usage = dom.usage();
    let mut batch = DomBatch::new();
    batch.set_attribute(canvas, "width", "12345");
    batch.stop_marquee(moving);
    batch.create_element_ns(
        document,
        crate::Namespace::from_uri("http://www.w3.org/1999/xhtml"),
        "canvas",
    );
    let prepared = dom.prepare(batch, &cancel).unwrap();
    cancel.store(true, Ordering::Release);
    assert_eq!(
        dom.publish(prepared, &cancel).unwrap_err(),
        DomError::Cancelled
    );
    assert_eq!(*dom.state, *before.state);
    assert_eq!(dom.usage(), usage);
    assert!(
        dom.changes_since(before.stamp())
            .unwrap()
            .changes()
            .is_empty()
    );
}

#[test]
fn cancelled_common_nonce_publication_preserves_all_state_and_history() {
    let mut dom = dom();
    let cancel = AtomicBool::new(false);
    let mut batch = DomBatch::new();
    let document = batch.create_document();
    let element = batch.create_element_ns(
        &document,
        crate::Namespace::from_uri("http://www.w3.org/1999/xhtml"),
        "div",
    );
    batch.set_attribute(&element, "nonce", "content");
    let commit = dom.apply(batch, &cancel).unwrap();
    let document = commit.document(&document).unwrap();
    let element = commit.created(&element).unwrap().unwrap();
    let before = dom.snapshot();
    let usage = dom.usage();
    let mut batch = DomBatch::new();
    batch.set_nonce(element, "unpublished");
    batch.insert_before(element, document, None);
    let transient = batch.create_element_ns(
        document,
        crate::Namespace::from_uri("http://www.w3.org/2000/svg"),
        "g",
    );
    batch.set_nonce(&transient, "also-unpublished");
    let prepared = dom.prepare(batch, &cancel).unwrap();
    cancel.store(true, Ordering::Release);
    assert_eq!(
        dom.publish(prepared, &cancel).unwrap_err(),
        DomError::Cancelled
    );
    assert_eq!(*dom.state, *before.state);
    assert_eq!(dom.usage(), usage);
    assert!(
        dom.changes_since(before.stamp())
            .unwrap()
            .changes()
            .is_empty()
    );
}

fn dom() -> Dom {
    Dom::new(DomLimits::new(32, 256, 64, 0, 0, 8, 8192).unwrap()).unwrap()
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
fn cancelled_prepared_import_discards_private_descendants_and_ticket_allocations() {
    let mut source = dom();
    let mut target = dom();
    let cancel = AtomicBool::new(false);
    let mut batch = DomBatch::new();
    let document = batch.create_document();
    let root = batch.create_element(&document, "root");
    let child = batch.create_text(&document, "child");
    batch.set_attribute(&root, "key", "source");
    batch.insert_before(&child, &root, None);
    let root = source
        .apply(batch, &cancel)
        .unwrap()
        .created(&root)
        .unwrap()
        .unwrap();
    let snapshot = source.snapshot();
    let mut batch = DomBatch::new();
    let document = batch.create_document();
    let document = target
        .apply(batch, &cancel)
        .unwrap()
        .document(&document)
        .unwrap();
    let before = target.snapshot();
    let usage = target.usage();
    let mut batch = DomBatch::new();
    batch.create_comment(document, "before import");
    let imported = batch.import_node(&snapshot, root, document, true);
    batch.set_attribute(&imported, "key", "not published");
    batch.create_document();
    let prepared = target.prepare(batch, &cancel).unwrap();
    cancel.store(true, Ordering::Release);
    assert_eq!(
        target.publish(prepared, &cancel).unwrap_err(),
        DomError::Cancelled
    );
    assert_eq!(*target.state, *before.state);
    assert_eq!(target.snapshot().stamp(), before.stamp());
    assert_eq!(target.usage(), usage);
    assert!(
        target
            .changes_since(before.stamp())
            .unwrap()
            .changes()
            .is_empty()
    );
    assert_eq!(*source.state, *snapshot.state);
    // A subsequent creation uses the first unpublished slot/generation.
    cancel.store(false, Ordering::Release);
    let mut batch = DomBatch::new();
    let next = batch.create_comment(document, "next");
    let report = target.apply(batch, &cancel).unwrap();
    let next = report.created(&next).unwrap().unwrap();
    assert_eq!(next.slot, 1);
    assert_eq!(next.generation, 0);
    assert_eq!(report.created(&imported), Err(DomError::ForeignBatch));
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
