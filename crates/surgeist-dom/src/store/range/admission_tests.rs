//! Sparse Range capture and genuine Source checkpoint rollback admission.
//! No parser, semantic fact, cursor or receipt is replaced by this harness.
use super::*;
use std::mem::size_of;

fn policy() -> crate::DomSourceLimits {
    crate::DomSourceLimits::new(16_000_000, 100_000, 10_000_000, 128, 4096, 4096, 1)
}
fn fixture() -> (Dom, DocumentId, NodeId) {
    let limits = DomLimits::new(32, 4096, 128, 0, 0, 32, 1_000_000)
        .unwrap()
        .with_ranges(8)
        .unwrap();
    let mut dom = Dom::new(limits).unwrap();
    let mut batch = DomBatch::new();
    let document = batch.create_document();
    let parent = batch.create_element_ns(&document, crate::Namespace::html(), "div");
    let text = batch.create_text(&document, "abcd");
    batch.insert_before(&text, &parent, None);
    batch.insert_before(&parent, &document, None);
    let commit = dom.apply(batch, &AtomicBool::new(false)).unwrap();
    (
        dom,
        commit.document(&document).unwrap(),
        commit.created(&text).unwrap().unwrap(),
    )
}
fn add_ranges(dom: &mut Dom, document: DocumentId, text: NodeId) -> Vec<DomRangeId> {
    let mut batch = DomBatch::new();
    let mut tickets = Vec::new();
    for _ in 0..4 {
        let ticket = batch.create_range(document);
        batch.set_range_start(&ticket, text, 3);
        batch.set_range_end(&ticket, text, 3);
        tickets.push(ticket);
    }
    batch.set_active_range(document, Some((&tickets[0]).into()));
    let commit = dom.apply(batch, &AtomicBool::new(false)).unwrap();
    tickets
        .iter()
        .map(|ticket| commit.created_range(ticket).unwrap().unwrap())
        .collect()
}
fn source_draft(dom: &Dom) -> Draft {
    let batch = DomBatch::new();
    Draft::new_source(
        dom,
        batch.owner.clone(),
        &batch.commands,
        policy(),
        &AtomicBool::new(false),
    )
    .unwrap()
    .0
}

#[test]
fn canonical_capture_is_sparse_and_created_identity_output_remains_exact() {
    let cancel = AtomicBool::new(false);
    let (mut dom, document, text) = fixture();
    let ids = add_ranges(&mut dom, document, text);
    let captured = dom.snapshot();
    let draft = source_draft(&dom);
    assert_eq!(draft.ranges.len(), 4);
    assert_eq!(draft.ranges.touched().count(), 0);
    for id in &ids {
        let record = draft.range_record(*id).unwrap();
        assert_eq!(record.start.container, Key::Existing(text));
        assert_eq!(record.start.offset, 3);
    }
    assert_eq!(
        draft.range_publication_ready(&cancel),
        draft.full_range_publication_ready(&cancel)
    );
    drop(draft);
    let mut retirement = DomBatch::new();
    retirement.retire_range(ids[3]);
    dom.apply(retirement, &cancel).unwrap();
    let mut creation = DomBatch::new();
    creation.create_range(document);
    let mut draft = Draft::new_source(
        &dom,
        creation.owner.clone(),
        &creation.commands,
        policy(),
        &cancel,
    )
    .unwrap()
    .0;
    assert_eq!(draft.ranges.touched().count(), 0);
    draft
        .apply(&creation.commands[0], dom.limits, &cancel)
        .unwrap();
    let id = draft.range_tickets[0];
    assert_eq!(
        (id.slot, id.generation),
        (ids[3].slot, ids[3].generation + 1)
    );
    assert_eq!(draft.ranges.touched().collect::<Vec<_>>(), [id.slot]);
    let prepared = draft.finish(&dom, &cancel).unwrap();
    assert_eq!(prepared.created_ranges, [Some(id)]);
    assert_eq!(prepared.changes, [DomChange::RangeCreated(id)]);
    let output_bytes = prepared.created_ranges.capacity() * size_of::<Option<DomRangeId>>()
        + prepared.changes.capacity() * size_of::<DomChange>();
    assert!(output_bytes >= size_of::<Option<DomRangeId>>() + size_of::<DomChange>());
    dom.publish(prepared, &cancel).unwrap();
    assert!(captured.range(ids[3]).is_ok());
    assert!(matches!(
        dom.snapshot().range(ids[3]),
        Err(DomError::StaleRange)
    ));
    assert_eq!(
        dom.snapshot().range(id).unwrap().start().container(),
        document.node()
    );
}

#[test]
fn actual_range_write_refusal_preserves_checkpoint_records_and_spent_work() {
    let cancel = AtomicBool::new(false);
    let (mut dom, document, text) = fixture();
    let ids = add_ranges(&mut dom, document, text);
    let snapshot = dom.snapshot();
    let mut draft = source_draft(&dom);
    let before: Vec<_> = (0..draft.ranges.len())
        .map(|index| draft.ranges.slot(index))
        .collect();
    let graph = draft.ranges.allocation_graph();
    let checkpoint = draft
        .source_checkpoint(draft.source_other_bytes, 0, &cancel)
        .unwrap();
    assert_eq!(
        draft.ranges.allocation_graph().pages().collect::<Vec<_>>(),
        graph.pages().collect::<Vec<_>>()
    );
    let spent = draft.source_work.load(Ordering::Relaxed);
    let mut short = policy();
    short.retained_bytes = draft.source_storage_bytes(&cancel).unwrap() + draft.source_other_bytes;
    draft.source_limits = Some(short);
    assert!(matches!(
        draft.set_range_boundary(ids[0], Key::Existing(text), 2, true, &cancel),
        Err(DomError::ResourceLimit {
            resource: DomResource::SourceRetainedBytes,
            ..
        })
    ));
    assert_eq!(
        (0..draft.ranges.len())
            .map(|index| draft.ranges.slot(index))
            .collect::<Vec<_>>(),
        before
    );
    let refused = draft.source_work.load(Ordering::Relaxed);
    assert!(refused >= spent);
    checkpoint.restore(&mut draft);
    assert_eq!(draft.source_work.load(Ordering::Relaxed), refused);
    assert_eq!(
        (0..draft.ranges.len())
            .map(|index| draft.ranges.slot(index))
            .collect::<Vec<_>>(),
        before
    );
    assert_eq!(dom.snapshot().stamp(), snapshot.stamp());
    assert_eq!(
        dom.snapshot().document(document).unwrap().active_range(),
        snapshot.document(document).unwrap().active_range()
    );
}

#[test]
fn reached_participant_refusal_and_late_cancellation_restore_real_checkpoint_then_retry() {
    let cancel = Arc::new(AtomicBool::new(false));
    let (mut dom, document, text) = fixture();
    let ids = add_ranges(&mut dom, document, text);
    let before = dom.snapshot();
    let usage = dom.usage();
    let mut operation = DomBatch::new();
    operation.replace_data(text, 1, 1, "XY");
    let mut draft = Draft::new_source(
        &dom,
        operation.owner.clone(),
        &operation.commands,
        policy(),
        &cancel,
    )
    .unwrap()
    .0;
    let key = draft.resolve(text.into()).unwrap();
    let old_records: Vec<_> = (0..draft.ranges.len())
        .map(|index| draft.ranges.slot(index))
        .collect();
    let checkpoint = draft
        .source_checkpoint(draft.source_other_bytes, 0, &cancel)
        .unwrap();
    let spent = draft.source_work.load(Ordering::Relaxed);
    let mut short = policy();
    short.work = spent + 1;
    draft.source_limits = Some(short);
    assert_eq!(
        draft.range_replace_data(key, 1, 1, 2, &cancel),
        Err(DomError::SourceWorkLimit { limit: spent + 1 })
    );
    assert_eq!(draft.source_work.load(Ordering::Relaxed), spent + 1);
    checkpoint.restore(&mut draft);
    assert_eq!(
        (0..draft.ranges.len())
            .map(|index| draft.ranges.slot(index))
            .collect::<Vec<_>>(),
        old_records
    );
    assert_eq!(draft.source_work.load(Ordering::Relaxed), spent + 1);

    draft.source_limits = Some(policy());
    let checkpoint = draft
        .source_checkpoint(draft.source_other_bytes, 0, &cancel)
        .unwrap();
    let late_cancel = cancel.clone();
    let observation =
        maintenance_observation::after_write(move || late_cancel.store(true, Ordering::Release));
    assert_eq!(
        draft.range_replace_data(key, 1, 1, 2, &cancel),
        Err(DomError::Cancelled)
    );
    drop(observation);
    assert_eq!(draft.range_record(ids[0]).unwrap().start.offset, 4);
    for id in &ids[1..] {
        assert_eq!(draft.range_record(*id).unwrap().start.offset, 3);
    }
    let spent = draft.source_work.load(Ordering::Relaxed);
    checkpoint.restore(&mut draft);
    assert_eq!(draft.source_work.load(Ordering::Relaxed), spent);
    assert!(cancel.load(Ordering::Acquire));
    assert_eq!(
        (0..draft.ranges.len())
            .map(|index| draft.ranges.slot(index))
            .collect::<Vec<_>>(),
        old_records
    );
    assert_eq!(dom.snapshot().stamp(), before.stamp());
    assert_eq!(dom.usage(), usage);

    cancel.store(false, Ordering::Release);
    draft
        .apply(&operation.commands[0], dom.limits, &cancel)
        .unwrap();
    assert_eq!(
        draft.range_publication_ready(&cancel),
        draft.full_range_publication_ready(&cancel)
    );
    let prepared = draft.finish(&dom, &cancel).unwrap();
    dom.publish(prepared, &cancel).unwrap();
    let after = dom.snapshot();
    assert_eq!(
        after.node(text).unwrap().data(),
        Some(&DocumentString::from("aXYcd"))
    );
    for id in ids {
        assert_eq!(after.range(id).unwrap().start().offset(), 4);
        assert_eq!(after.range(id).unwrap().end().offset(), 4);
        assert_eq!(before.range(id).unwrap().start().offset(), 3);
    }
    assert_eq!(
        after.document(document).unwrap().active_range(),
        before.document(document).unwrap().active_range()
    );
    assert!(dom.pending_source_edit().is_none());
}
