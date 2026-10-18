//! Actual canonical Range payload, checkpoint-copy and rollback admission.
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
fn actual_range_banks_tickets_and_created_identity_output_are_independently_priced() {
    let cancel = AtomicBool::new(false);
    let (mut dom, document, text) = fixture();
    let empty_state = dom.state.clone();
    let empty_draft = source_draft(&dom);
    let empty_bytes = empty_draft.source_storage_bytes(&cancel).unwrap();
    let ids = add_ranges(&mut dom, document, text);
    let filled_draft = source_draft(&dom);
    let canonical_bytes = source::storage::state(&dom.state, || Ok(())).unwrap().bytes;
    let old_canonical_bytes = source::storage::state(&empty_state, || Ok(()))
        .unwrap()
        .bytes;
    // The first fixture publication grows the node-slot bank one creation at a
    // time; the next publication reserves its established length all at once.
    // Equal topology does not promise equal spare backing. Price both actual
    // node-slot capacities independently rather than treating their shrink as
    // a negative Range charge. Other fixture node payload/banks remain equal;
    // the changed association is inline in the already-priced node.
    assert_eq!(dom.state.slots.len(), empty_state.slots.len());
    for (after, before) in dom.state.slots.values().zip(&empty_state.slots) {
        let after = after.node.as_ref().unwrap();
        let before = before.node.as_ref().unwrap();
        assert_eq!(after.kind, before.kind);
        assert_eq!(after.children.capacity(), before.children.capacity());
        assert_eq!(
            after.manual_nodes.capacity(),
            before.manual_nodes.capacity()
        );
        assert_eq!(
            after.assigned_nodes.capacity(),
            before.assigned_nodes.capacity()
        );
    }
    assert_eq!(
        dom.state.signal_slots.capacity(),
        empty_state.signal_slots.capacity()
    );
    assert_eq!(
        dom.state.native_tasks.capacity(),
        empty_state.native_tasks.capacity()
    );
    assert_eq!(
        dom.state.details_intentions.capacity(),
        empty_state.details_intentions.capacity()
    );
    let old_node_slots = empty_state.slots.index_charge().bytes;
    let new_node_slots = dom.state.slots.index_charge().bytes;
    let canonical_bank = dom.state.ranges.index_charge().bytes + dom.state.ranges.totals().bytes;
    assert_eq!(
        canonical_bytes + old_node_slots,
        old_canonical_bytes + new_node_slots + canonical_bank
    );
    let draft_bank = filled_draft.ranges.capacity() * size_of::<crate::range::Slot<Key>>();
    assert_eq!(
        filled_draft.source_storage_bytes(&cancel).unwrap() - empty_bytes,
        draft_bank
    );
    assert!(canonical_bytes >= canonical_bank);
    drop((empty_draft, filled_draft));

    // Real retirement supplies an actual reusable slot. Creation below changes
    // its fixed record without growing the bank; its new ticket is additional
    // owned storage, independently of the permanent canonical bank.
    let mut batch = DomBatch::new();
    batch.retire_range(ids[3]);
    dom.apply(batch, &cancel).unwrap();
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
    let bank_capacity = draft.ranges.capacity();
    let before = draft.source_storage_bytes(&cancel).unwrap();
    draft
        .apply(&creation.commands[0], dom.limits, &cancel)
        .unwrap();
    assert_eq!(draft.ranges.capacity(), bank_capacity);
    let id = draft.range_tickets[0];
    let ticket_bytes = draft.range_tickets.capacity() * size_of::<DomRangeId>();
    assert_eq!(
        draft.source_storage_bytes(&cancel).unwrap() - before,
        ticket_bytes
    );
    assert_ne!(id, ids[3]);

    let prepared = draft.finish(&dom, &cancel).unwrap();
    assert_eq!(prepared.created_ranges, [Some(id)]);
    assert_eq!(prepared.changes, [DomChange::RangeCreated(id)]);
    let new_output_bytes = prepared.created_ranges.capacity() * size_of::<Option<DomRangeId>>()
        + prepared.changes.capacity() * size_of::<DomChange>();
    let formed_with_output = draft.source_formation_bytes.load(Ordering::Relaxed);
    dom.publish(prepared, &cancel).unwrap();
    assert_eq!(draft.source_formation_bytes.load(Ordering::Relaxed), 0);
    // The real published state is now the same candidate, so an unchanged
    // finishing Draft has every same canonical output bank except the genuine
    // creation result and its change row. Their exact real capacities, not a
    // storage function's output, determine the required additional charge.
    let unchanged = source_draft(&dom);
    let prepared = unchanged.finish(&dom, &cancel).unwrap();
    assert!(prepared.created_ranges.is_empty());
    assert!(prepared.changes.is_empty());
    assert_eq!(
        formed_with_output - unchanged.source_formation_bytes.load(Ordering::Relaxed),
        new_output_bytes
    );
    drop(prepared);
    assert_eq!(unchanged.source_formation_bytes.load(Ordering::Relaxed), 0);
    assert_eq!(
        dom.snapshot().range(id).unwrap().start().container(),
        document.node()
    );
}

#[test]
fn original_checkpoint_range_and_ticket_copies_require_exact_simultaneous_bank_backing() {
    let cancel = AtomicBool::new(false);
    let (mut dom, document, text) = fixture();
    let ids = add_ranges(&mut dom, document, text);
    let mut batch = DomBatch::new();
    batch.retire_range(ids[3]);
    dom.apply(batch, &cancel).unwrap();
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
    draft
        .apply(&creation.commands[0], dom.limits, &cancel)
        .unwrap();
    let before = dom.snapshot();
    let baseline = draft.source_storage_bytes(&cancel).unwrap() + draft.source_other_bytes;
    assert!(
        baseline
            >= draft.ranges.capacity() * size_of::<crate::range::Slot<Key>>()
                + draft.range_tickets.capacity() * size_of::<DomRangeId>()
    );
    // These are independent actual allocator capacities for the exact copy
    // requests. The original banks stay live during both additional copies.
    let mut range_probe = Vec::<crate::range::Slot<Key>>::new();
    range_probe.try_reserve_exact(draft.ranges.len()).unwrap();
    let mut ticket_probe = Vec::<DomRangeId>::new();
    ticket_probe
        .try_reserve_exact(draft.range_tickets.len())
        .unwrap();
    let range_bytes = range_probe.capacity() * size_of::<crate::range::Slot<Key>>();
    let ticket_bytes = ticket_probe.capacity() * size_of::<DomRangeId>();
    drop((range_probe, ticket_probe));
    let exact = baseline + range_bytes + ticket_bytes;
    let mut short = policy();
    short.retained_bytes = exact - 1;
    draft.source_limits = Some(short);
    {
        let mut admission = source::checkpoint::Admission::new(&draft, baseline).unwrap();
        let ranges = admission.copy(&draft.ranges, &cancel).unwrap();
        assert_eq!(ranges, draft.ranges);
        assert_eq!(
            draft.select_temporary_bytes.load(Ordering::Relaxed),
            range_bytes
        );
        assert!(
            matches!(admission.copy(&draft.range_tickets, &cancel), Err(DomError::ResourceLimit { resource: DomResource::SourceRetainedBytes, limit }) if limit == exact - 1)
        );
        drop(ranges);
        drop(admission);
    }
    assert_eq!(draft.select_temporary_bytes.load(Ordering::Relaxed), 0);
    let mut full = policy();
    full.retained_bytes = exact;
    draft.source_limits = Some(full);
    let spent = draft.source_work.load(Ordering::Relaxed);
    {
        let mut admission = source::checkpoint::Admission::new(&draft, baseline).unwrap();
        let ranges = admission.copy(&draft.ranges, &cancel).unwrap();
        let tickets = admission.copy(&draft.range_tickets, &cancel).unwrap();
        assert_eq!(ranges, draft.ranges);
        assert_eq!(tickets, draft.range_tickets);
        let charge = admission.finish();
        assert_eq!(
            draft.select_temporary_bytes.load(Ordering::Relaxed),
            range_bytes + ticket_bytes
        );
        // One genuine copied-row visit per original Range slot and ticket.
        assert_eq!(
            draft.source_work.load(Ordering::Relaxed) - spent,
            (draft.ranges.len() + draft.range_tickets.len()) as u64
        );
        drop((ranges, tickets));
        drop(charge);
    }
    assert_eq!(draft.select_temporary_bytes.load(Ordering::Relaxed), 0);
    assert_eq!(dom.snapshot().stamp(), before.stamp());
    assert_eq!(
        dom.snapshot().document(document).unwrap().active_range(),
        before.document(document).unwrap().active_range()
    );
}

#[test]
fn reached_mid_range_scan_work_refusal_restores_real_checkpoint_and_retries_once_after_cancellation()
 {
    let cancel = AtomicBool::new(false);
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
    let old_records = draft.ranges.clone();
    let checkpoint = draft
        .source_checkpoint(draft.source_other_bytes, 0, &cancel)
        .unwrap();
    let checkpoint_bytes = draft.select_temporary_bytes.load(Ordering::Relaxed);
    assert!(checkpoint_bytes >= draft.ranges.len() * size_of::<crate::range::Slot<Key>>());
    let spent = draft.source_work.load(Ordering::Relaxed);
    let mut short = policy();
    short.work = spent + 1;
    draft.source_limits = Some(short);
    // There is one admitted visit per Range slot. Permit the first real record
    // and refuse the second; this is inside maintenance, not entry work0.
    assert_eq!(
        draft.range_replace_data(key, 1, 1, 2, &cancel),
        Err(DomError::SourceWorkLimit { limit: spent + 1 })
    );
    assert_eq!(draft.ranges[0].record.unwrap().start.offset, 4);
    assert_eq!(draft.ranges[1..], old_records[1..]);
    assert_eq!(draft.source_work.load(Ordering::Relaxed), spent + 1);
    checkpoint.restore(&mut draft);
    assert_eq!(draft.ranges, old_records);
    assert_eq!(draft.source_work.load(Ordering::Relaxed), spent + 1);
    assert_eq!(draft.select_temporary_bytes.load(Ordering::Relaxed), 0);
    assert_eq!(dom.snapshot().stamp(), before.stamp());
    assert_eq!(dom.usage(), usage);

    // Cancellation remains the caller's actual latched flag across restore.
    // It cannot turn the unpublished partial scan into success or consume a
    // renewed work meter. Cleanup requires no fallible admission.
    cancel.store(true, Ordering::Release);
    assert_eq!(
        draft.range_replace_data(key, 1, 1, 2, &cancel),
        Err(DomError::Cancelled)
    );
    assert!(cancel.load(Ordering::Acquire));
    assert_eq!(draft.ranges, old_records);
    assert_eq!(draft.source_work.load(Ordering::Relaxed), spent + 1);
    cancel.store(false, Ordering::Release);
    draft.source_limits = Some(policy());
    // Retry the actual complete CharacterData command, not a synthetic text
    // write after calling maintenance. The original owner orders both effects.
    draft
        .apply(&operation.commands[0], dom.limits, &cancel)
        .unwrap();
    for id in &ids {
        assert_eq!(draft.range_record(*id).unwrap().start.offset, 4);
    }
    let prepared = draft.finish(&dom, &cancel).unwrap();
    dom.publish(prepared, &cancel).unwrap();
    let after = dom.snapshot();
    assert_eq!(
        after.node(text).unwrap().data(),
        Some(&DocumentString::from("aXYcd"))
    );
    for id in ids {
        let range = after.range(id).unwrap();
        assert_eq!(range.start().offset(), 4);
        assert_eq!(range.end().offset(), 4);
        assert_eq!(before.range(id).unwrap().start().offset(), 3);
    }
    assert_eq!(
        after.document(document).unwrap().active_range(),
        before.document(document).unwrap().active_range()
    );
    assert_eq!(draft.source_formation_bytes.load(Ordering::Relaxed), 0);
    assert_eq!(draft.select_temporary_bytes.load(Ordering::Relaxed), 0);
    assert!(dom.pending_source_edit().is_none());
}
