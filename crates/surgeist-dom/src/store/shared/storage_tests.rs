use super::{SharedVec, allocation_observation::Observation};
use crate::store::{
    self, Dom, DomBatch, DomError, DomLimits, DomResource, Key, Slot, bank, custody, transaction,
};
use std::{
    mem::size_of,
    sync::{
        Arc,
        atomic::{AtomicBool, Ordering},
    },
};

#[test]
#[ignore = "requires SURGEIST_DOM_STORAGE_COUNTERS=1; select explicitly"]
fn zero_capacity_construction_and_clones_allocate_no_sequence_targets() {
    let observation = Observation::begin();
    let first = SharedVec::<u32>::default();
    let second = SharedVec::<u32>::from(Vec::new());
    let before_clones = observation.counts();
    let capture = first.clone();
    let capture_again = capture.clone();
    assert_eq!(observation.counts(), before_clones);
    assert_eq!(&first[..], &[]);
    assert_eq!(first.capacity(), 0);
    assert_eq!(second.capacity(), 0);
    assert!(first.same_storage(&second));
    assert!(!first.has_backing());
    assert_eq!(first.retained_bytes().unwrap(), 0);
    assert_eq!(first, second);
    drop(first);
    drop(capture);
    assert!(capture_again.is_empty());
    let actual = observation.finish();
    assert_eq!(
        actual.headers, 0,
        "zero-capacity lists have no Arc<Vec> target"
    );
    assert_eq!(actual.zero_capacity_headers, 0);
    assert_eq!(actual.header_bytes, 0);
    assert_eq!(actual.retained_capacity_bytes, 0);
}

#[test]
#[ignore = "requires SURGEIST_DOM_STORAGE_COUNTERS=1; select explicitly"]
fn allocation_observation_records_real_reserved_and_populated_targets() {
    let mut reserved = Vec::<u16>::new();
    reserved.try_reserve_exact(37).unwrap();
    let capacity = reserved.capacity();
    let populated = vec![7u16, 11, 13];
    let populated_capacity = populated.capacity();
    let observation = Observation::begin();
    let reserved = SharedVec::from(reserved);
    let populated = SharedVec::from(populated);
    let capture = reserved.clone();
    let actual = observation.finish();
    assert_eq!(actual.headers, 2);
    assert_eq!(actual.zero_capacity_headers, 0);
    assert_eq!(actual.header_bytes, 2 * size_of::<Vec<u16>>());
    assert_eq!(
        actual.retained_capacity_bytes,
        (capacity + populated_capacity) * size_of::<u16>()
    );
    assert!(capture.same_storage(&reserved));
    assert_eq!(&populated[..], &[7, 11, 13]);
}

#[test]
fn reserved_empty_preserves_capacity_identity_and_clone_lifetime() {
    let mut raw = Vec::<u64>::new();
    raw.try_reserve_exact(37).unwrap();
    let capacity = raw.capacity();
    let pointer = raw.as_ptr();
    let first = SharedVec::from(raw);
    let clone = first.clone();
    assert!(first.is_empty());
    assert!(first.has_backing());
    assert_eq!(first.capacity(), capacity);
    assert_eq!(first.as_ptr(), pointer);
    assert!(first.same_storage(&clone));
    let other = SharedVec::from(Vec::<u64>::with_capacity(capacity));
    assert_eq!(first, other);
    assert!(!first.same_storage(&other));
    let absent = SharedVec::<u64>::default();
    assert_eq!(first, absent);
    assert!(!first.same_storage(&absent));
    assert!(!absent.has_backing());
    assert_eq!(
        first.retained_bytes().unwrap(),
        size_of::<Vec<u64>>() + capacity * size_of::<u64>()
    );
    drop(first);
    assert_eq!(clone.capacity(), capacity);
    assert_eq!(clone.as_ptr(), pointer);
    assert_eq!((&clone).into_iter().count(), 0);
}

#[test]
fn supplied_zero_capacity_target_keeps_its_identity_and_owner() {
    let raw = Arc::new(Vec::<u32>::new());
    let weak = Arc::downgrade(&raw);
    let target = Arc::as_ptr(&raw);
    let values = SharedVec::from_shared(raw);
    let clone = values.clone();
    let absent = SharedVec::<u32>::default();
    assert_eq!(values.capacity(), 0);
    assert!(values.has_backing());
    assert!(values.same_storage(&clone));
    assert!(!values.same_storage(&absent));
    assert_eq!(values, absent);
    assert_eq!(values.retained_bytes().unwrap(), size_of::<Vec<u32>>());
    assert_eq!(Arc::as_ptr(&weak.upgrade().unwrap()), target);
    drop(values);
    assert!(weak.upgrade().is_some());
    drop(clone);
    assert!(weak.upgrade().is_none());
}

#[test]
fn zero_capacity_canonical_sequence_retains_no_backing_occurrence() {
    let ledger = custody::Ledger::default();
    let sequence =
        transaction::Sequence::<u32, u32>::empty(std::convert::identity, &ledger, |_| Ok(()))
            .unwrap();
    assert!(sequence.view().is_empty());
    assert_eq!(sequence.capacity(), 0);
    assert_eq!(sequence.staged_values(), Some(&[][..]));
    assert!(sequence.canonical_backing().is_none());
    assert_eq!(sequence.receipt(), custody::Usage::ZERO);
    assert_eq!(ledger.total(), custody::Usage::ZERO);
}

#[test]
fn zero_sized_values_keep_vec_capacity_without_requiring_element_clone() {
    struct Value;
    let raw = vec![Value];
    let capacity = raw.capacity();
    let values = SharedVec::from(raw);
    let clone = values.clone();
    assert_eq!(size_of::<Value>(), 0);
    assert_eq!(capacity, usize::MAX);
    assert_eq!(clone.capacity(), capacity);
    assert_eq!(clone.len(), 1);
    assert!(clone.same_storage(&values));
    assert!(clone.has_backing());
    assert_eq!(clone.retained_bytes().unwrap(), size_of::<Vec<Value>>());
    drop(values);
    assert_eq!(clone.len(), 1);

    let empty = SharedVec::<()>::default();
    assert!(empty.is_empty());
    assert_eq!(empty.capacity(), Vec::<()>::new().capacity());
    assert!(empty.has_backing());
    assert_eq!(empty.retained_bytes().unwrap(), size_of::<Vec<()>>());
}

#[test]
fn absent_first_write_refuses_one_short_and_rollback_restores_absence() {
    let ledger = custody::Ledger::default();
    let mut sequence =
        transaction::Sequence::<u32, u32>::empty(std::convert::identity, &ledger, |_| Ok(()))
            .unwrap();
    let exact = size_of::<u32>();
    assert!(matches!(
        sequence.insert(
            0,
            &[13],
            true,
            &ledger,
            |usage| store::admit(usage.bytes(), exact - 1, DomResource::SourceRetainedBytes),
            || Ok(())
        ),
        Err(DomError::ResourceLimit {
            resource: DomResource::SourceRetainedBytes,
            ..
        })
    ));
    assert_eq!(ledger.total(), custody::Usage::ZERO);
    assert!(sequence.view().is_empty());
    assert_eq!(sequence.capacity(), 0);
    assert_eq!(sequence.receipt(), custody::Usage::ZERO);
    assert!(sequence.canonical_backing().is_none());
    assert!(matches!(
        sequence.insert(
            0,
            &[13],
            true,
            &ledger,
            |_| Ok(()),
            || Err(DomError::Cancelled)
        ),
        Err(DomError::Cancelled)
    ));
    assert_eq!(ledger.total(), custody::Usage::ZERO);
    let inverse = sequence
        .insert(
            0,
            &[13],
            true,
            &ledger,
            |usage| store::admit(usage.bytes(), exact, DomResource::SourceRetainedBytes),
            || Ok(()),
        )
        .unwrap()
        .unwrap();
    assert_eq!(sequence.view().first(), Some(13));
    assert_eq!(ledger.total(), custody::Usage::new(exact, 0));
    sequence.restore(inverse);
    assert_eq!(ledger.total(), custody::Usage::ZERO);
    assert!(sequence.view().is_empty());
    assert_eq!(sequence.capacity(), 0);
    assert_eq!(sequence.staged_values(), Some(&[][..]));
    assert_eq!(sequence.receipt(), custody::Usage::ZERO);
    assert!(sequence.canonical_backing().is_none());
}

#[test]
fn reserved_empty_first_write_admits_overlap_and_removal_keeps_selected_capacity() {
    let mut raw = Vec::<u32>::new();
    raw.try_reserve_exact(7).unwrap();
    let capacity = raw.capacity();
    let original = SharedVec::from(raw);
    let ledger = custody::Ledger::default();
    let registry = bank::CustodyRegistry::new(&ledger, |_| Ok(())).unwrap();
    let payload_usage = custody::Usage::new(
        size_of::<store::payload::Payload<store::canonical::ContainerChildren<u32>>>()
            + size_of::<Vec<u32>>()
            + capacity * size_of::<u32>(),
        0,
    );
    let reserved = store::payload::Reserved::new().unwrap();
    let debit = ledger.retain(payload_usage, |_| Ok(())).unwrap();
    let group = reserved
        .materialize(
            custody::Held::new(
                store::canonical::ContainerChildren {
                    values: original.clone(),
                },
                debit,
            ),
            payload_usage,
            |_| Ok(()),
        )
        .unwrap()
        .seal(&registry, |_| Ok(()))
        .unwrap();
    let mut sequence = transaction::Sequence::captured(
        &group.value.value.values,
        group.custody.clone(),
        std::convert::identity,
    );
    let before = ledger.total();
    let raw_group = custody::Usage::new(
        std::mem::size_of_val(&*group.value)
            + size_of::<Vec<u32>>()
            + group.value.value.values.capacity() * size_of::<u32>(),
        0,
    );
    assert_eq!(
        before,
        raw_group
            .checked_add(group.custody.inspect_control_graph().usage())
            .unwrap()
            .checked_add(registry.inspect_storage())
            .unwrap()
    );
    let requested = 2 * size_of::<u32>();
    let exact = before.bytes() + requested;
    let result = sequence.insert(
        0,
        &[8, 13],
        true,
        &ledger,
        |usage| {
            if usage.bytes() > exact - 1 {
                Err(DomError::ResourceLimit {
                    resource: DomResource::SourceRetainedBytes,
                    limit: exact - 1,
                })
            } else {
                Ok(())
            }
        },
        || Ok(()),
    );
    assert!(matches!(
        result,
        Err(DomError::ResourceLimit {
            resource: DomResource::SourceRetainedBytes,
            ..
        })
    ));
    assert_eq!(ledger.total(), before);
    assert!(
        sequence
            .canonical_backing()
            .unwrap()
            .same_storage(&original)
    );
    let cancelled = AtomicBool::new(true);
    assert!(matches!(
        sequence.insert(
            0,
            &[8, 13],
            true,
            &ledger,
            |_| Ok(()),
            || store::check_cancel(&cancelled)
        ),
        Err(DomError::Cancelled)
    ));
    assert_eq!(ledger.total(), before);
    let inverse = sequence
        .insert(
            0,
            &[8, 13],
            true,
            &ledger,
            |usage| {
                if usage.bytes() > exact {
                    Err(DomError::ResourceLimit {
                        resource: DomResource::SourceRetainedBytes,
                        limit: exact,
                    })
                } else {
                    Ok(())
                }
            },
            || Ok(()),
        )
        .unwrap()
        .unwrap();
    assert_eq!(sequence.view().iter().collect::<Vec<_>>(), [8, 13]);
    let selected_capacity = sequence.capacity();
    assert_eq!(ledger.total(), custody::Usage::new(exact, 0));
    sequence
        .remove(0, 2, false, &ledger, |_| Ok(()), || Ok(()))
        .unwrap();
    assert!(sequence.view().is_empty());
    assert_eq!(sequence.capacity(), selected_capacity);
    sequence.restore(inverse);
    assert_eq!(sequence.capacity(), capacity);
    assert!(
        sequence
            .canonical_backing()
            .unwrap()
            .same_storage(&original)
    );
    assert_eq!(ledger.total(), before);
}

#[test]
#[ignore = "requires SURGEIST_DOM_STORAGE_COUNTERS=1; select explicitly"]
fn canonical_empty_node_lists_allocate_no_shared_targets_and_snapshots_only_alias() {
    let observation = Observation::begin();
    let mut dom = Dom::new(
        DomLimits::new(
            16,
            4096,
            64,
            0,
            0,
            8,
            1_000_000,
            crate::DomNameLimits::new(16_777_216, 16_384, 4_000_000, 1_048_576).unwrap(),
        )
        .unwrap(),
    )
    .unwrap();
    let mut batch = DomBatch::new();
    let document = batch.create_document();
    let element = batch.create_element(&document, "plain");
    let text = batch.create_text(&document, "text");
    let comment = batch.create_comment(&document, "comment");
    let commit = dom.apply(batch, &AtomicBool::new(false)).unwrap();
    let ids = [&document, &element, &text, &comment]
        .map(|ticket| commit.created(ticket).unwrap().unwrap());
    let before_snapshots = observation.counts();
    let snapshot = dom.snapshot();
    let snapshot_clone = snapshot.clone();
    assert_eq!(observation.counts(), before_snapshots);
    for id in ids {
        let node = snapshot_clone.state.slots[id.slot].node.as_ref().unwrap();
        assert!(node.children().is_empty());
        assert!(node.children_backing().is_none());
        assert!(node.manual_nodes().is_empty());
        assert!(node.assigned_nodes().is_empty());
        assert!(matches!(
            node.associations(),
            None | Some(store::canonical::ElementAssociations::None)
        ));
        assert!(node.fullscreen_events().is_empty());
        if let store::Node::Document(document) = node {
            assert!(document.value.fullscreen_events.is_none());
        }
    }
    let graph = dom
        .state
        .slots
        .inspect_graph(|_| {
            Ok(bank::Totals {
                bytes: size_of::<Slot>(),
                units: 0,
                occurrences: 1,
            })
        })
        .unwrap();
    let actual = observation.finish();
    eprintln!(
        "actual_shared_targets={actual:?}; remaining_canonical_slot_bank_pages={} records={}",
        graph.pages().count(),
        graph.records().count()
    );
    assert!(graph.pages().count() > 0);
    assert_eq!(graph.records().count(), 4);
    assert_eq!(
        actual.zero_capacity_headers, 0,
        "canonical empty lists have no sequence target allocation"
    );
}

#[test]
fn ordinary_source_and_rollback_preserve_empty_snapshots_and_noop_roots() {
    let cancel = AtomicBool::new(false);
    let limits = DomLimits::new(
        16,
        4096,
        64,
        0,
        0,
        16,
        1_000_000,
        crate::DomNameLimits::new(16_777_216, 16_384, 4_000_000, 1_048_576).unwrap(),
    )
    .unwrap();
    let mut dom = Dom::new(limits).unwrap();
    let mut batch = DomBatch::new();
    let document = batch.create_document();
    let root = batch.create_element(&document, "root");
    let child = batch.create_text(&document, "child");
    let done = dom.apply(batch, &cancel).unwrap();
    let root = done.created(&root).unwrap().unwrap();
    let child = done.created(&child).unwrap().unwrap();
    let empty = dom.snapshot();
    let mut batch = DomBatch::new();
    batch.insert_before(child, root, None);
    dom.apply(batch, &cancel).unwrap();
    let populated = dom.snapshot();
    assert!(empty.node(root).unwrap().children().is_empty());
    assert_eq!(populated.node(root).unwrap().children(), [child]);
    let policy = crate::DomSourceLimits::new(64_000_000, 1_000_000, 10_000_000, 16, 64, 64, 1);
    let mut source = dom
        .begin_source_edit(crate::DomSourceEdit::detach(child), policy, &cancel)
        .unwrap();
    assert!(matches!(
        source.advance(&cancel).unwrap(),
        crate::DomSourceProgress::Complete(_)
    ));
    drop(source);
    assert!(dom.snapshot().node(root).unwrap().children().is_empty());
    assert_eq!(populated.node(root).unwrap().children(), [child]);
    let batch = DomBatch::new();
    let mut draft = store::Draft::new_source(&dom, batch.owner, &batch.commands, policy, &cancel)
        .unwrap()
        .0;
    let original = dom.state.clone();
    let stamp = dom.snapshot().stamp();
    let mark = draft
        .source_checkpoint(draft.source_other_bytes, draft.source_other_units, &cancel)
        .unwrap();
    draft
        .pre_insert(
            Key::Existing(child),
            Key::Existing(root),
            None,
            limits,
            &cancel,
        )
        .unwrap();
    assert_eq!(
        draft
            .node(Key::Existing(root))
            .unwrap()
            .children()
            .iter()
            .collect::<Vec<_>>(),
        [Key::Existing(child)]
    );
    let spent = draft.source_work.load(Ordering::Relaxed);
    mark.restore(&mut draft);
    assert!(
        draft
            .node(Key::Existing(root))
            .unwrap()
            .children()
            .is_empty()
    );
    assert_eq!(draft.source_work.load(Ordering::Relaxed), spent);
    let prepared = draft.finish(&dom, &cancel).unwrap();
    let commit = dom.publish(prepared, &cancel).unwrap();
    assert!(commit.changes().changes().is_empty());
    assert_eq!(dom.snapshot().stamp(), stamp);
    assert!(Arc::ptr_eq(&dom.state, &original));
}
