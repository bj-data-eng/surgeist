//! Owning local qualifications only; no fabricated completed Show or queue.
use super::*;

fn limits(work: u64) -> crate::DomSourceLimits {
    crate::DomSourceLimits::new(16_000_000, 100_000, work, 16, 4096, 4096, 1)
}
fn fixture() -> (Dom, DocumentId, DocumentId, NodeId) {
    let mut dom =
        Dom::new(DomLimits::new(64, 100_000, 256, 256, 256, 16, 1_000_000).unwrap()).unwrap();
    let mut batch = DomBatch::new();
    let first = batch.create_document();
    let second = batch.create_document();
    let dialog = batch.create_element_ns(&first, crate::Namespace::html(), "dialog");
    batch.insert_before(&dialog, &first, None);
    let done = dom.apply(batch, &AtomicBool::new(false)).unwrap();
    (
        dom,
        done.document(&first).unwrap(),
        done.document(&second).unwrap(),
        done.created(&dialog).unwrap().unwrap(),
    )
}

#[test]
fn known_empty_raw_layer_qualifies_even_with_open_dialogs_but_unknown_and_members_do_not() {
    let (dom, first, _, dialog) = fixture();
    let cancel = AtomicBool::new(false);
    let batch = DomBatch::new();
    let (mut draft, _, _) = Draft::new_source(
        &dom,
        batch.owner.clone(),
        &batch.commands,
        limits(10_000_000),
        &cancel,
    )
    .unwrap();
    let document = draft.resolve(first.node().into()).unwrap();
    let subject = draft.resolve(dialog.into()).unwrap();
    assert_eq!(
        draft.dialog_show_node_document(subject, &cancel).unwrap(),
        first
    );
    // Exercise the real list owner, independently of active watcher eligibility.
    draft.dialog_list_add(document, subject, &cancel).unwrap();
    assert!(
        draft
            .dialog_show_known_empty_top_layer(first, &cancel)
            .unwrap()
    );

    // This is an unpublished negative raw-state fixture, not a claim that a
    // modal/popover producer or full Show inserted this genuine Element.
    let formation = source::formation::Formation::new(&draft, &cancel)
        .unwrap()
        .unwrap();
    formation.retain_bytes(size_of::<Key>(), &cancel).unwrap();
    let mut members = Vec::new();
    members.try_reserve_exact(1).unwrap();
    assert_eq!(members.capacity(), 1);
    members.push(subject);
    let Some(DocumentInteractiveState::Known { top_layer, .. }) =
        draft.node_mut(document).interactive_document.as_mut()
    else {
        panic!("native known lists")
    };
    *top_layer = members;
    drop(formation);
    draft.source_admit(0, &cancel).unwrap();
    assert!(
        !draft
            .dialog_show_known_empty_top_layer(first, &cancel)
            .unwrap()
    );
    let Some(DocumentInteractiveState::Known { top_layer, .. }) =
        draft.node_mut(document).interactive_document.as_mut()
    else {
        panic!("native known lists")
    };
    top_layer.clear();
    assert_eq!(top_layer.capacity(), 1);
    assert!(
        draft
            .dialog_show_known_empty_top_layer(first, &cancel)
            .unwrap()
    );
    draft.node_mut(document).interactive_document = Some(DocumentInteractiveState::Unavailable);
    assert!(
        !draft
            .dialog_show_known_empty_top_layer(first, &cancel)
            .unwrap()
    );
    draft.node_mut(document).interactive_document = None;
    assert!(
        !draft
            .dialog_show_known_empty_top_layer(first, &cancel)
            .unwrap()
    );
    assert_eq!(draft.source_formation_bytes.load(Ordering::Relaxed), 0);
    assert_eq!(draft.source_temporary_units.load(Ordering::Relaxed), 0);
    assert!(
        dom.snapshot()
            .node(first.node())
            .unwrap()
            .children()
            .contains(&dialog)
    );
}

#[test]
fn current_receiver_document_and_previous_capture_are_independent_after_real_adoption() {
    let (dom, first, second, dialog) = fixture();
    let cancel = AtomicBool::new(false);
    let before = dom.snapshot();
    let batch = DomBatch::new();
    let (mut draft, _, _) = Draft::new_source(
        &dom,
        batch.owner.clone(),
        &batch.commands,
        limits(10_000_000),
        &cancel,
    )
    .unwrap();
    let subject = draft.resolve(dialog.into()).unwrap();
    let captured = draft.dialog_show_node_document(subject, &cancel).unwrap();
    assert_eq!(captured, first);
    let destination = draft.resolve(second.node().into()).unwrap();
    draft
        .adopt(subject, destination, dom.limits, &cancel)
        .unwrap();
    let current = draft.dialog_show_node_document(subject, &cancel).unwrap();
    assert_eq!(current, second);
    let original = draft.resolve(captured.node().into()).unwrap();
    draft.node_mut(original).interactive_document = Some(DocumentInteractiveState::Unavailable);
    // Find qualifies the current receiver Document; Hide must separately read
    // the previously captured exact Document, never substitute this new one.
    assert!(
        draft
            .dialog_show_known_empty_top_layer(current, &cancel)
            .unwrap()
    );
    assert!(
        !draft
            .dialog_show_known_empty_top_layer(captured, &cancel)
            .unwrap()
    );
    assert_eq!(before.node(dialog).unwrap().owner_document(), Some(first));
    assert_eq!(dom.snapshot().stamp(), before.stamp());
}

#[test]
fn captured_document_qualification_rejects_actual_foreign_and_wrong_generation() {
    let (dom, first, second, _) = fixture();
    let cancel = AtomicBool::new(false);
    // Deliberately invalid owner-private input, not a claim that ordinary
    // Destroy supports Documents or that a public retirement occurred.
    let wrong_generation = DocumentId(NodeId {
        generation: first.node().generation.checked_add(1).unwrap(),
        ..first.node()
    });
    let (_, foreign, _, _) = fixture();
    let batch = DomBatch::new();
    let (draft, _, _) = Draft::new_source(
        &dom,
        batch.owner.clone(),
        &batch.commands,
        limits(10_000_000),
        &cancel,
    )
    .unwrap();
    assert_eq!(
        draft.dialog_show_known_empty_top_layer(wrong_generation, &cancel),
        Err(DomError::StaleNode)
    );
    assert_eq!(
        draft.dialog_show_known_empty_top_layer(foreign, &cancel),
        Err(DomError::ForeignOwner)
    );
    assert!(
        draft
            .dialog_show_known_empty_top_layer(first, &cancel)
            .unwrap()
    );
    assert!(
        draft
            .dialog_show_known_empty_top_layer(second, &cancel)
            .unwrap()
    );
}

#[test]
fn two_actual_reads_need_exact_work_and_cancellation_preserves_same_draft_for_retry() {
    for capture in [true, false] {
        for short in [true, false] {
            let (dom, first, _, dialog) = fixture();
            let cancel = AtomicBool::new(false);
            let batch = DomBatch::new();
            let (mut draft, _, _) = Draft::new_source(
                &dom,
                batch.owner.clone(),
                &batch.commands,
                limits(10_000_000),
                &cancel,
            )
            .unwrap();
            let subject = draft.resolve(dialog.into()).unwrap();
            let storage = draft.source_storage_bytes(&cancel).unwrap();
            let baseline = draft.source_work.load(Ordering::Relaxed);
            // Subject/document reads or exact-document/state reads: two real
            // Source steps. Initial canonical formation is a separate scaffold.
            let cap = baseline + 2 - u64::from(short);
            draft.source_limits = Some(limits(cap));
            cancel.store(true, Ordering::Relaxed);
            let cancelled = if capture {
                draft
                    .dialog_show_node_document(subject, &cancel)
                    .map(|_| ())
            } else {
                draft
                    .dialog_show_known_empty_top_layer(first, &cancel)
                    .map(|_| ())
            };
            assert_eq!(cancelled, Err(DomError::Cancelled));
            assert_eq!(draft.source_work.load(Ordering::Relaxed), baseline);
            cancel.store(false, Ordering::Relaxed);
            let result = if capture {
                draft
                    .dialog_show_node_document(subject, &cancel)
                    .map(|document| assert_eq!(document, first))
            } else {
                draft
                    .dialog_show_known_empty_top_layer(first, &cancel)
                    .map(|empty| assert!(empty))
            };
            if short {
                assert_eq!(result, Err(DomError::SourceWorkLimit { limit: cap }));
            } else {
                result.unwrap();
            }
            assert_eq!(draft.source_work.load(Ordering::Relaxed), cap);
            draft.source_limits = Some(limits(10_000_000));
            assert_eq!(
                draft.dialog_show_node_document(subject, &cancel).unwrap(),
                first
            );
            assert!(
                draft
                    .dialog_show_known_empty_top_layer(first, &cancel)
                    .unwrap()
            );
            assert_eq!(draft.source_storage_bytes(&cancel).unwrap(), storage);
            assert_eq!(draft.source_formation_bytes.load(Ordering::Relaxed), 0);
            assert_eq!(draft.source_temporary_units.load(Ordering::Relaxed), 0);
        }
    }
}

#[test]
fn original_checkpoint_restores_actual_adoption_and_keeps_spent_work_for_retry() {
    let (dom, first, second, dialog) = fixture();
    let cancel = AtomicBool::new(false);
    let batch = DomBatch::new();
    let (mut draft, _, _) = Draft::new_source(
        &dom,
        batch.owner.clone(),
        &batch.commands,
        limits(10_000_000),
        &cancel,
    )
    .unwrap();
    let subject = draft.resolve(dialog.into()).unwrap();
    let destination = draft.resolve(second.node().into()).unwrap();
    let checkpoint = draft
        .source_checkpoint(draft.source_other_bytes, draft.source_other_units, &cancel)
        .unwrap();
    let captured = draft.dialog_show_node_document(subject, &cancel).unwrap();
    draft
        .adopt(subject, destination, dom.limits, &cancel)
        .unwrap();
    assert_eq!(
        draft.dialog_show_node_document(subject, &cancel).unwrap(),
        second
    );
    let spent = draft.source_work.load(Ordering::Relaxed);
    cancel.store(true, Ordering::Relaxed);
    assert_eq!(
        draft.dialog_show_known_empty_top_layer(captured, &cancel),
        Err(DomError::Cancelled)
    );
    checkpoint.restore(&mut draft);
    assert_eq!(draft.source_work.load(Ordering::Relaxed), spent);
    cancel.store(false, Ordering::Relaxed);
    assert_eq!(
        draft.dialog_show_node_document(subject, &cancel).unwrap(),
        first
    );
    assert!(
        draft
            .dialog_show_known_empty_top_layer(first, &cancel)
            .unwrap()
    );
    assert_eq!(draft.source_formation_bytes.load(Ordering::Relaxed), 0);
    assert_eq!(draft.source_temporary_units.load(Ordering::Relaxed), 0);
}
