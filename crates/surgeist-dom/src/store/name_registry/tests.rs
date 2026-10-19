use super::*;
use std::{
    collections::HashSet,
    sync::atomic::{AtomicBool, Ordering},
};

fn limits() -> DomNameLimits {
    DomNameLimits::new(32_000_000, 4096, 1_000_000, 32_000_000).unwrap()
}
fn admitted<T>(action: impl FnOnce(&mut Admission<'_>) -> T) -> T {
    action(&mut Admission {
        global: &mut |_| Ok(()),
        step: &mut || Ok(()),
    })
}
fn fixture() -> (custody::Ledger, bank::CustodyRegistry, Registry) {
    let ledger = custody::Ledger::default();
    let control = bank::CustodyRegistry::new(&ledger, |_| Ok(())).unwrap();
    let registry = admitted(|admission| {
        Registry::new(
            DomOwnerId::allocate().unwrap(),
            &control,
            limits(),
            admission,
        )
    })
    .unwrap();
    (ledger, control, registry)
}
fn own_bytes(text: &DocumentString) -> usize {
    size_of::<NameRecord>()
        + size_of::<Vec<u16>>()
        + text.shared_code_units().capacity() * size_of::<u16>()
        + size_of::<custody::Debit>()
}
fn assert_raw_local(registry: &Registry) {
    let state = registry.lock();
    let storage = size_of::<Mutex<State>>() + state.index.inspect_storage_bytes();
    assert_eq!(state.storage.usage(), custody::Usage::new(storage, 0));
    let mut records = custody::Usage::ZERO;
    for row in state.index.inspect_rows() {
        let record = row.record.upgrade().expect("live raw record witness");
        let raw = custody::Usage::new(
            own_bytes(record.text()),
            record.text().shared_code_units().capacity(),
        );
        assert_eq!(row.own, raw);
        assert_eq!(row._custody.identity(), record.identity());
        assert_eq!(
            row._custody.inspect_payload_usage().bytes() + size_of::<custody::Debit>(),
            raw.bytes()
        );
        records = records.checked_add(raw).unwrap();
    }
    assert_eq!(state.records, records);
    assert_eq!(
        state.usage().unwrap(),
        records
            .checked_add(custody::Usage::new(storage + state.controls, 0))
            .unwrap()
    );
}

#[test]
fn exact_text_has_one_owner_identity_and_tokens_do_not_retain_the_owner() {
    let (_ledger, _control, registry) = fixture();
    let operation = admitted(|admission| registry.begin(None, admission)).unwrap();
    let left = admitted(|admission| {
        operation.reserve(DocumentString::from_utf16(vec![0xd800, 0, 0x61]), admission)
    })
    .unwrap()
    .publish();
    let right = admitted(|admission| {
        operation.reserve(DocumentString::from_utf16(vec![0xd800, 0, 0x61]), admission)
    })
    .unwrap()
    .publish();
    assert_eq!(left, right);
    assert_eq!(left.identity(), right.identity());
    assert!(!Arc::ptr_eq(
        left.text().shared_code_units(),
        DocumentString::from_utf16(vec![0xd800, 0, 0x61]).shared_code_units()
    ));
    let different = admitted(|admission| {
        operation.reserve(DocumentString::from_utf16(vec![0xd801, 0, 0x61]), admission)
    })
    .unwrap()
    .publish();
    assert_ne!(left, different);
    let (_, _, foreign) = fixture();
    let foreign_operation = admitted(|admission| foreign.begin(None, admission)).unwrap();
    assert_eq!(
        foreign_operation.check_owner(&left),
        Err(DomError::ForeignOwner)
    );
    let foreign_atom =
        admitted(|admission| foreign_operation.reserve(left.text().clone(), admission))
            .unwrap()
            .publish();
    assert_eq!(foreign_atom.text(), left.text());
    assert_ne!(foreign_atom, left);
    let mut set = HashSet::new();
    set.insert(left.clone());
    set.insert(right);
    set.insert(different);
    assert_eq!(set.len(), 2);
    assert_raw_local(&registry);
    let owner = Arc::downgrade(&registry.0);
    drop(operation);
    drop(registry);
    assert!(owner.upgrade().is_none(), "semantic atoms retain no owner");
    assert_eq!(left.text().code_units(), &[0xd800, 0, 0x61]);
}

#[test]
fn child_promotion_survives_parent_and_independent_private_reservation_rollback() {
    let (_, _, registry) = fixture();
    let parent = admitted(|admission| registry.begin(None, admission)).unwrap();
    let pending = admitted(|admission| parent.reserve("shared".into(), admission)).unwrap();
    let identity = pending.atom.as_ref().unwrap().identity();
    let independent = admitted(|admission| registry.begin(None, admission)).unwrap();
    let sibling = admitted(|admission| independent.reserve("shared".into(), admission)).unwrap();
    assert_eq!(sibling.atom.as_ref().unwrap().identity(), identity);
    let child = admitted(|admission| registry.begin(Some(&parent), admission)).unwrap();
    let promoted = admitted(|admission| child.reserve("shared".into(), admission))
        .unwrap()
        .publish();
    assert_eq!(promoted.identity(), identity);
    drop(pending);
    drop(sibling);
    drop(child);
    drop(parent);
    let visible = admitted(|admission| independent.reserve("shared".into(), admission))
        .unwrap()
        .publish();
    assert_eq!(visible, promoted);
    assert_eq!(registry.lock().index.entries(), 1);
    assert_raw_local(&registry);
}

#[test]
fn independent_preparations_share_the_original_reservation_and_close_it_once() {
    for winner in 0..2 {
        for drop_loser_first in [false, true] {
            let (_, _, registry) = fixture();
            let first = admitted(|admission| registry.begin(None, admission)).unwrap();
            let second = admitted(|admission| registry.begin(None, admission)).unwrap();
            let first_scope = Arc::downgrade(&first.scope);
            let second_scope = Arc::downgrade(&second.scope);
            let mut reservations = [
                Some(admitted(|a| first.reserve("prepared".into(), a)).unwrap()),
                Some(admitted(|a| second.reserve("prepared".into(), a)).unwrap()),
            ];
            let identity = reservations[0].as_ref().unwrap().key;
            assert_eq!(reservations[1].as_ref().unwrap().key, identity);
            let slot = reservations[0].as_ref().unwrap().slot;
            let pending_bytes = registry
                .lock()
                .index
                .row(slot)
                .pending
                .as_ref()
                .unwrap()
                .bytes;
            let scratch = *first.scope.scratch.lock().unwrap();
            let current = registry.inspect_usage().bytes();
            assert_eq!(
                registry.set_retained_limits(current - 1, 1),
                Err(DomError::ResourceLimit {
                    resource: DomResource::NameRetainedBytes,
                    limit: current - 1,
                })
            );
            registry.set_retained_limits(current, 1).unwrap();
            let loser = 1 - winner;
            if drop_loser_first {
                drop(reservations[loser].take());
                assert!(registry.lock().index.row(slot).pending.is_some());
                assert_eq!(*first.scope.scratch.lock().unwrap(), scratch);
            }
            let visible = reservations[winner].take().unwrap().publish();
            assert_eq!(visible.identity(), identity);
            assert!(registry.lock().index.row(slot).pending.is_none());
            assert_eq!(
                *first.scope.scratch.lock().unwrap(),
                scratch - pending_bytes
            );
            drop(reservations);
            assert_eq!(
                *first.scope.scratch.lock().unwrap(),
                scratch - pending_bytes
            );
            assert_eq!(registry.lock().index.entries(), 1);
            assert_raw_local(&registry);
            drop(first);
            drop(second);
            assert!(first_scope.upgrade().is_none());
            assert!(second_scope.upgrade().is_none());
            let next = admitted(|admission| registry.begin(None, admission)).unwrap();
            let retained = admitted(|admission| next.reserve("prepared".into(), admission))
                .unwrap()
                .publish();
            assert_eq!(retained, visible);
        }
    }
    let (_, _, registry) = fixture();
    let first = admitted(|admission| registry.begin(None, admission)).unwrap();
    let second = admitted(|admission| registry.begin(None, admission)).unwrap();
    let origin = Arc::downgrade(&first.scope);
    let initial = admitted(|a| first.reserve("outlives origin operation".into(), a)).unwrap();
    let sibling = admitted(|a| second.reserve("outlives origin operation".into(), a)).unwrap();
    let identity = initial.key;
    drop(initial);
    drop(first);
    assert!(
        origin.upgrade().is_some(),
        "the remaining pin owns the original receipt"
    );
    let published = sibling.publish();
    assert_eq!(published.identity(), identity);
    assert!(
        origin.upgrade().is_none(),
        "promotion closes the original receipt once"
    );
    assert_raw_local(&registry);
}

#[test]
fn exact_first_name_extent_and_one_byte_short_use_the_same_owner_and_no_generic_delta() {
    let (_ledger, _, registry) = fixture();
    let operation = admitted(|admission| registry.begin(None, admission)).unwrap();
    let mut raw = Vec::with_capacity(7);
    raw.extend([0x61, 0x62]);
    let text = DocumentString::from_utf16(raw);
    let exact = size_of::<Mutex<State>>()
        + size_of::<Scope>()
        + size_of::<Mutex<Retained>>()
        + index::Index::inspect_first_reservation_bytes()
        + own_bytes(&text);
    registry.set_retained_limits(exact - 1, 1).unwrap();
    let result = admitted(|admission| operation.reserve(text.clone(), admission));
    assert_eq!(
        result.err(),
        Some(DomError::ResourceLimit {
            resource: DomResource::NameRetainedBytes,
            limit: exact - 1
        })
    );
    assert_eq!(registry.lock().index.entries(), 0);
    assert_eq!(registry.lock().index.inspect_storage_bytes(), 0);
    registry.set_retained_limits(exact, 1).unwrap();
    let token = admitted(|admission| operation.reserve(text, admission))
        .unwrap()
        .publish();
    assert_eq!(registry.lock().usage().unwrap().bytes(), exact);
    assert_raw_local(&registry);
    let before = registry.lock().limits;
    assert_eq!(
        registry.set_retained_limits(exact - 1, 1),
        Err(DomError::ResourceLimit {
            resource: DomResource::NameRetainedBytes,
            limit: exact - 1
        })
    );
    assert_eq!(
        registry.lock().limits.retained_bytes(),
        before.retained_bytes()
    );
    assert_eq!(token.text().code_units(), &[0x61, 0x62]);
}

#[test]
fn entry_work_scratch_and_cancellation_refuse_without_uninterned_success() {
    let (_, _, registry) = fixture();
    registry
        .set_retained_limits(limits().retained_bytes(), 0)
        .unwrap();
    let operation = admitted(|admission| registry.begin(None, admission)).unwrap();
    assert_eq!(
        admitted(|admission| operation.reserve("a".into(), admission)).err(),
        Some(DomError::ResourceLimit {
            resource: DomResource::NameEntries,
            limit: 0
        })
    );
    assert_eq!(registry.lock().index.entries(), 0);
    registry
        .set_retained_limits(limits().retained_bytes(), 10)
        .unwrap();
    let cancelled = operation.reserve(
        "a".into(),
        &mut Admission {
            global: &mut |_| Err(DomError::Cancelled),
            step: &mut || Ok(()),
        },
    );
    assert_eq!(cancelled.err(), Some(DomError::Cancelled));
    assert_eq!(registry.lock().index.entries(), 0);
    {
        let mut state = registry.lock();
        state.limits = DomNameLimits::new(
            state.limits.retained_bytes(),
            state.limits.entries(),
            1,
            state.limits.operation_scratch(),
        )
        .unwrap();
    }
    let narrow = admitted(|admission| registry.begin(None, admission)).unwrap();
    assert_eq!(
        admitted(|admission| narrow.reserve("ab".into(), admission)).err(),
        Some(DomError::WorkLimit {
            resource: DomResource::NameWork,
            limit: 1
        })
    );
    let baseline = size_of::<Scope>() + size_of::<Mutex<Retained>>();
    {
        let mut state = registry.lock();
        state.limits = DomNameLimits::new(
            state.limits.retained_bytes(),
            state.limits.entries(),
            limits().operation_work(),
            baseline,
        )
        .unwrap();
    }
    let scratch = admitted(|admission| registry.begin(None, admission)).unwrap();
    assert_eq!(
        admitted(|admission| scratch.reserve("a".into(), admission)).err(),
        Some(DomError::ResourceLimit {
            resource: DomResource::NameScratch,
            limit: baseline
        })
    );
    assert_eq!(registry.lock().index.entries(), 0);
}

#[test]
fn forced_collisions_are_exact_and_split_growth_preserves_all_published_identities() {
    let (_, _, registry) = fixture();
    let operation = admitted(|admission| registry.begin(None, admission)).unwrap();
    let mut tokens = Vec::new();
    for value in 0..96u16 {
        let text = DocumentString::from_utf16(vec![value, 0xd800]);
        let fingerprint = if value < 8 { 7 } else { u64::from(value) };
        tokens.push(
            admitted(|admission| operation.reserve_hashed(text, fingerprint, admission))
                .unwrap()
                .publish(),
        );
    }
    for (value, token) in tokens.iter().enumerate().rev() {
        let value = value as u16;
        let fingerprint = if value < 8 { 7 } else { u64::from(value) };
        let again = admitted(|admission| {
            operation.reserve_hashed(
                DocumentString::from_utf16(vec![value, 0xd800]),
                fingerprint,
                admission,
            )
        })
        .unwrap()
        .publish();
        assert_eq!(&again, token);
    }
    assert_eq!(registry.lock().index.entries(), 96);
    assert_raw_local(&registry);
}

#[test]
fn repeated_private_failure_reuses_index_capacity_and_generic_release_paths() {
    let (ledger, control, registry) = fixture();
    let operation = admitted(|admission| registry.begin(None, admission)).unwrap();
    let mut stable = None;
    for value in 0..128u16 {
        let pending = admitted(|admission| {
            operation.reserve(DocumentString::from_utf16(vec![value]), admission)
        })
        .unwrap();
        let key = pending.key;
        let slot = pending.slot;
        drop(pending);
        control.reconcile_releases();
        let state = registry.lock();
        assert_eq!(state.index.entries(), 0);
        assert_eq!(state.index.inspect_row_extent(), 1);
        assert_eq!(state.records, custody::Usage::ZERO);
        assert_eq!(ledger.total().units(), 0);
        assert_eq!(
            state.index.inspect_storage_bytes()
                + size_of::<Mutex<State>>()
                + size_of::<Scope>()
                + size_of::<Mutex<Retained>>(),
            state.usage().unwrap().bytes()
        );
        let usage = ledger.total();
        if let Some(stable) = stable {
            assert_eq!(usage, stable);
        } else {
            stable = Some(usage);
        }
        drop(state);
        let replacement =
            admitted(|admission| operation.reserve("live".into(), admission)).unwrap();
        assert_eq!(replacement.slot, slot);
        assert_ne!(replacement.key, key);
        assert!(
            registry.lock().index.find_row_mut(slot, key).is_none(),
            "burned row key cannot touch its replacement"
        );
        drop(replacement);
    }
    drop(operation);
    drop(registry);
    control.reconcile_releases();
    assert_eq!(
        ledger.total(),
        bank::CustodyRegistry::administration_usage()
    );
}

#[test]
fn dead_weak_retains_record_receipt_until_incremental_cleanup_but_not_string_backing() {
    let (ledger, control, registry) = fixture();
    let operation = admitted(|admission| registry.begin(None, admission)).unwrap();
    let text = DocumentString::from_utf16(vec![1, 2, 3]);
    let backing = Arc::downgrade(text.shared_code_units());
    let atom = admitted(|admission| operation.reserve(text, admission))
        .unwrap()
        .publish();
    let old_identity = atom.identity();
    let record = Arc::downgrade(atom.record());
    let receipt = registry.lock().records;
    drop(atom);
    assert!(record.upgrade().is_none());
    assert!(
        backing.upgrade().is_none(),
        "weak record does not retain destroyed fields"
    );
    assert_eq!(registry.lock().records, receipt);
    assert_eq!(
        admitted(|admission| operation.cleanup(1, admission)).unwrap(),
        1
    );
    assert_eq!(registry.lock().records, custody::Usage::ZERO);
    control.reconcile_releases();
    let new = admitted(|admission| {
        operation.reserve(DocumentString::from_utf16(vec![1, 2, 3]), admission)
    })
    .unwrap()
    .publish();
    assert_ne!(new.identity(), old_identity);
    assert_eq!(new.text().code_units(), &[1, 2, 3]);
    drop(new);
    drop(operation);
    drop(registry);
    control.reconcile_releases();
    assert_eq!(
        ledger.total(),
        bank::CustodyRegistry::administration_usage()
    );
}

#[test]
fn unrelated_generic_admission_and_release_do_not_change_name_local_receipts() {
    let (ledger, control, registry) = fixture();
    let operation = admitted(|admission| registry.begin(None, admission)).unwrap();
    let baseline = registry.lock().usage().unwrap();
    let unrelated = ledger
        .retain(custody::Usage::new(12345, 777), |_| Ok(()))
        .unwrap();
    assert_eq!(registry.lock().usage().unwrap(), baseline);
    let other_control = bank::CustodyRegistry::new(&ledger, |_| Ok(())).unwrap();
    let mut callback_count = 0;
    let token = operation
        .reserve(
            "name".into(),
            &mut Admission {
                global: &mut |_| {
                    callback_count += 1;
                    let temporary = other_control.adopt_object(
                        custody::MaterializationId::fresh()?,
                        custody::Usage::new(37, 0),
                        |_| Ok(()),
                    )?;
                    drop(temporary);
                    other_control.reconcile_releases();
                    Ok(())
                },
                step: &mut || Ok(()),
            },
        )
        .unwrap()
        .publish();
    assert!(callback_count > 1);
    assert_raw_local(&registry);
    let local = registry.lock().usage().unwrap();
    drop(unrelated);
    drop(other_control);
    control.reconcile_releases();
    assert_eq!(registry.lock().usage().unwrap(), local);
    assert_eq!(token.text().code_units(), &[110, 97, 109, 101]);
}

#[test]
fn actual_source_restore_keeps_reserved_rows_and_mandatory_promotion_needs_no_new_admission() {
    use super::super::{Draft, source::formation::Formation};
    use crate::{Dom, DomBatch, DomLimits, DomSourceLimits};
    let cancel = AtomicBool::new(false);
    let dom = Dom::new(
        DomLimits::new(
            64,
            4096,
            64,
            0,
            0,
            16,
            4_000_000,
            crate::DomNameLimits::new(16_777_216, 16_384, 4_000_000, 1_048_576).unwrap(),
        )
        .unwrap(),
    )
    .unwrap();
    let batch = DomBatch::new();
    let (mut draft, _) = Draft::new_source(
        &dom,
        batch.owner,
        &batch.commands,
        DomSourceLimits::new(32_000_000, 100_000, 1_000_000, 16, 64, 64, 1),
        &cancel,
    )
    .unwrap();
    let formation = Formation::new(&draft, &cancel).unwrap().unwrap();
    let mut source = Admission {
        global: &mut |usage| formation.admit_custody_usage(usage, &cancel),
        step: &mut || formation.step(&cancel),
    };
    let registry =
        Registry::new(dom.owner, draft.custody_registry(), limits(), &mut source).unwrap();
    let parent = registry.begin(None, &mut source).unwrap();
    let checkpoint = draft
        .source_checkpoint(draft.source_other_bytes, draft.source_other_units, &cancel)
        .unwrap();
    let mandatory = parent.reserve("mandatory".into(), &mut source).unwrap();
    let identity = mandatory.key;
    let work_before = draft.source_work.load(Ordering::Relaxed);
    checkpoint.restore(&mut draft);
    assert!(draft.source_work.load(Ordering::Relaxed) >= work_before);
    let child = registry.begin(Some(&parent), &mut source).unwrap();
    let competing = child
        .reserve("competing".into(), &mut source)
        .unwrap()
        .publish();
    let exact_live = registry.lock().usage().unwrap().bytes();
    registry.set_retained_limits(exact_live, 2).unwrap();
    assert!(parent.reserve("third".into(), &mut source).is_err());
    let before = draft.custody_ledger().total();
    let admitted_work = draft.source_work.load(Ordering::Relaxed);
    // No callback, issuer or allocation is available to this final transition.
    let published = mandatory.publish();
    assert_eq!(published.identity(), identity);
    assert_eq!(draft.custody_ledger().total(), before);
    assert_eq!(draft.source_work.load(Ordering::Relaxed), admitted_work);
    assert_eq!(
        competing.text().code_units(),
        &[99, 111, 109, 112, 101, 116, 105, 110, 103]
    );
    assert_raw_local(&registry);
}

#[test]
fn public_binding_precedes_insertion_and_changes_only_operational_state() {
    let cancel = AtomicBool::new(false);
    let mut dom = super::super::Dom::new(
        super::super::DomLimits::new(32, 65_536, 64, 0, 0, 16, 4_000_000, limits()).unwrap(),
    )
    .unwrap();
    let original = dom.snapshot();
    let bound = dom.bind_name(&"div".into(), &cancel).unwrap();
    let duplicate = dom
        .bind_name(&DocumentString::from_utf16(vec![100, 105, 118]), &cancel)
        .unwrap();
    assert_eq!(bound, duplicate);
    assert_eq!(dom.snapshot().stamp(), original.stamp());
    assert!(Arc::ptr_eq(&dom.state, &original.state));
    assert_eq!(dom.usage().history_entries(), 0);
    let mut setup = crate::DomBatch::new();
    let document = setup.create_document();
    let element = setup.create_element_ns(&document, crate::Namespace::html(), "div");
    setup.set_attribute_ns(&element, crate::Namespace::html(), "p:div", "unchanged");
    let commit = dom.apply(setup, &cancel).unwrap();
    let snapshot = dom.snapshot();
    let element = snapshot
        .node(commit.created(&element).unwrap().unwrap())
        .unwrap()
        .as_element()
        .unwrap();
    assert_eq!(element.name().local_name_atom(), &bound);
    assert_eq!(element.attributes()[0].name().local_name_atom(), &bound);
    assert_eq!(element.attributes()[0].name().prefix(), Some(&"p".into()));
    let old_row = bound.record().row();
    let identity = bound.identity();
    let mut unrelated = Vec::new();
    for value in 0..40 {
        unrelated.push(
            dom.bind_name(&format!("unrelated-{value}").into(), &cancel)
                .unwrap(),
        );
    }
    assert!(dom.names.lock().index.find_row(old_row, identity).is_some());
    assert_eq!(dom.bind_name(&"div".into(), &cancel).unwrap(), bound);
    drop(dom);
    assert_eq!(bound.text().code_units(), &[100, 105, 118]);
    assert_eq!(element.name().local_name_atom(), &bound);
}

#[test]
fn retained_operation_replay_uses_the_same_qualified_cache_without_allocating_an_overlay() {
    let (_ledger, _control, registry) = fixture();
    let operation = admitted(|admission| registry.begin(None, admission)).unwrap();
    let first = admitted(|admission| operation.retain("reserved".into(), admission)).unwrap();
    {
        let retained = operation.retained.lock().unwrap();
        let storage = retained.values.inspect_storage_bytes();
        assert_eq!(
            registry.lock().controls,
            size_of::<Scope>() + size_of::<Mutex<Retained>>() + storage
        );
        assert_eq!(
            retained.storage.as_ref().unwrap().debit.usage().bytes(),
            size_of::<Mutex<Retained>>() + storage
        );
    }
    let before = registry.lock().usage().unwrap();
    let again = admitted(|admission| operation.retain("reserved".into(), admission)).unwrap();
    assert_eq!(again, first);
    assert_eq!(registry.lock().usage().unwrap(), before);
    assert_eq!(operation.retained.lock().unwrap().values.active_count(), 1);
    let stale = custody::MaterializationId::fresh().unwrap();
    assert!(
        registry
            .lock()
            .index
            .find_row(first.record().row(), stale)
            .is_none()
    );
    assert_raw_local(&registry);
    drop(operation);
    assert_eq!(
        registry.lock().index.entries(),
        0,
        "unpublished cache leaves with its original operation"
    );
    assert_eq!(
        first.text().code_units(),
        &[114, 101, 115, 101, 114, 118, 101, 100]
    );
}

#[test]
fn real_source_preflight_retains_names_across_restore_and_competing_child_publication() {
    use super::super::{Dom, DomBatch, DomLimits, Draft, source};
    let cancel = AtomicBool::new(false);
    let mut dom =
        Dom::new(DomLimits::new(32, 65_536, 64, 0, 0, 16, 4_000_000, limits()).unwrap()).unwrap();
    let mut setup = DomBatch::new();
    let document = setup.create_document();
    let document = crate::DocumentId(
        dom.apply(setup, &cancel)
            .unwrap()
            .created(&document)
            .unwrap()
            .unwrap(),
    );
    let mut batch = DomBatch::new();
    let ticket = batch.create_element_ns(document, crate::Namespace::none(), "mandatory");
    let mut draft = Draft::new_source(
        &dom,
        batch.owner.clone(),
        &batch.commands,
        crate::DomSourceLimits::new(32_000_000, 100_000, 4_000_000, 16, 64, 64, 1),
        &cancel,
    )
    .unwrap()
    .0;
    draft
        .admit_source_command_headroom(
            &dom,
            source::headroom::Entry::Command(&batch.commands[0]),
            0,
            &cancel,
        )
        .unwrap();
    assert!(
        draft.created.is_empty(),
        "actual Restore removed the probed semantic node"
    );
    let parent = draft.names.clone();
    let retained = parent.retained.lock().unwrap();
    let atom = retained
        .values
        .values()
        .find_map(|row| {
            row.atom
                .as_ref()
                .filter(|atom| atom.text() == &"mandatory".into())
        })
        .unwrap()
        .clone();
    drop(retained);
    let child = admitted(|admission| dom.names.begin(Some(&parent), admission)).unwrap();
    let promoted = admitted(|admission| child.reserve("mandatory".into(), admission))
        .unwrap()
        .publish();
    assert_eq!(promoted, atom);
    let competing = admitted(|admission| child.reserve("competing".into(), admission))
        .unwrap()
        .publish();
    let (live, records, index) = {
        let state = dom.names.lock();
        (
            state.usage().unwrap().bytes(),
            state.records,
            state.index.inspect_storage_bytes(),
        )
    };
    // This command has not acknowledged an external event. Its record/index
    // reservation is the claim here; a newly formed staged containing capsule
    // is a distinct actual allocation, independently priced from its body.
    let containing = super::super::payload::inspect_element_language_child_extent();
    // The whole qualified name has one real dedicated target, dependency body,
    // and two shared-debit bodies. Registry selection aliases that name while
    // replacing only the Element language capsule, so the two capsules overlap.
    // These raw extents are separate from record/index/overlay reservations.
    let name = size_of::<super::super::payload::Payload<crate::element::ElementName>>()
        + super::super::payload::inspect_element_name_children_body_bytes()
        + 2 * size_of::<custody::Debit>();
    dom.names
        .set_retained_limits(live + name + 2 * containing, 2)
        .unwrap();
    draft
        .apply(&batch.commands[0], dom.limits, &cancel)
        .unwrap();
    let key = draft.resolve((&ticket).into()).unwrap();
    assert_eq!(draft.element(key).unwrap().name().local_name_atom(), &atom);
    let state = dom.names.lock();
    assert_eq!(
        state.records, records,
        "replay retained the actual name records"
    );
    assert_eq!(
        state.index.inspect_storage_bytes(),
        index,
        "competing publication did not consume the parent's occupied paths"
    );
    assert_eq!(
        state.usage().unwrap().bytes(),
        live + name + containing,
        "the actual name target and staged language capsule remain retained"
    );
    drop(state);
    drop(draft);
    drop(parent);
    drop(child);
    dom.names
        .set_retained_limits(limits().retained_bytes(), limits().entries())
        .unwrap();
    assert_eq!(
        dom.bind_name(&"mandatory".into(), &cancel).unwrap(),
        promoted
    );
    assert_eq!(competing.text(), &"competing".into());
}

#[test]
fn public_single_output_exact_name_extent_and_one_short_are_independently_priced() {
    let cancel = AtomicBool::new(false);
    let mut dom = super::super::Dom::new(
        super::super::DomLimits::new(16, 4096, 64, 0, 0, 0, 0, limits()).unwrap(),
    )
    .unwrap();
    let mut units = Vec::with_capacity(7);
    units.extend([0x61, 0x62]);
    let text = DocumentString::from_utf16(units);
    let exact = size_of::<Mutex<State>>()
        + size_of::<Scope>()
        + size_of::<Mutex<Retained>>()
        + size_of::<DomNameAtom>()
        + index::Index::inspect_first_reservation_bytes()
        + own_bytes(&text);
    let original = dom.snapshot();
    let policy = |bytes| {
        super::super::DomLimits::new(
            16,
            4096,
            64,
            0,
            0,
            0,
            0,
            DomNameLimits::new(bytes, 1, 1_000_000, 32_000_000).unwrap(),
        )
        .unwrap()
    };
    dom.set_limits(policy(exact - 1)).unwrap();
    assert_eq!(
        dom.bind_name(&text, &cancel),
        Err(DomError::ResourceLimit {
            resource: DomResource::NameRetainedBytes,
            limit: exact - 1
        })
    );
    assert_eq!(dom.names.lock().index.entries(), 0);
    assert_eq!(
        dom.names.lock().usage().unwrap().bytes(),
        size_of::<Mutex<State>>()
    );
    assert!(Arc::ptr_eq(&dom.state, &original.state));
    dom.set_limits(policy(exact)).unwrap();
    let atom = dom.bind_name(&text, &cancel).unwrap();
    assert_eq!(atom.text().code_units(), &[0x61, 0x62]);
    assert_eq!(dom.snapshot().stamp(), original.stamp());
    assert_raw_local(&dom.names);
    let used = dom.names.lock().usage().unwrap().bytes();
    assert_eq!(
        used,
        size_of::<Mutex<State>>()
            + index::Index::inspect_first_reservation_bytes()
            + own_bytes(&text),
        "temporary operation and output formation end after the caller handoff"
    );
}

#[test]
fn measured_capsule_extent_survives_child_scope_without_ancestor_scratch_or_double_release() {
    struct Capsule {
        _values: Vec<u64>,
        _local: Attribution,
    }
    let (ledger, control, registry) = fixture();
    let parent = admitted(|admit| registry.begin(None, admit)).unwrap();
    let scratch = *parent.scope.scratch.lock().unwrap();
    let child = admitted(|admit| registry.begin(Some(&parent), admit)).unwrap();
    let values = Vec::<u64>::with_capacity(7);
    let extent = size_of::<Capsule>() + values.capacity() * size_of::<u64>();
    let local = extent + size_of::<custody::Debit>();
    let baseline = registry.lock().usage().unwrap().bytes();
    registry
        .set_retained_limits(baseline + local - 1, limits().entries())
        .unwrap();
    assert!(matches!(
        child.attribute(local),
        Err(DomError::ResourceLimit {
            resource: DomResource::NameRetainedBytes,
            ..
        })
    ));
    assert_eq!(registry.lock().usage().unwrap().bytes(), baseline);
    registry
        .set_retained_limits(baseline + local, limits().entries())
        .unwrap();
    let attribution = child.attribute(local).unwrap();
    let debit = ledger
        .retain(custody::Usage::new(extent, 0), |_| Ok(()))
        .unwrap();
    let capsule = bank::Dependency::from_retained(
        custody::Held::new(
            Capsule {
                _values: values,
                _local: attribution,
            },
            debit,
        ),
        custody::Usage::new(extent, 0),
        |_| Ok(()),
    )
    .unwrap();
    let scope = Arc::downgrade(&child.scope);
    drop(child);
    assert!(
        scope.upgrade().is_none(),
        "published child receipt never retains an operation"
    );
    assert_eq!(
        *parent.scope.scratch.lock().unwrap(),
        scratch,
        "child close removes its remaining formation scratch exactly once"
    );
    let before = registry.lock().usage().unwrap().bytes();
    assert_eq!(
        before,
        size_of::<Mutex<State>>() + size_of::<Scope>() + size_of::<Mutex<Retained>>() + local
    );
    let unrelated = ledger
        .retain(custody::Usage::new(19_007, 0), |_| Ok(()))
        .unwrap();
    assert_eq!(
        registry.lock().usage().unwrap().bytes(),
        before,
        "unrelated original-ledger storage is outside the local receipt"
    );
    drop(unrelated);
    drop(capsule);
    control.reconcile_releases();
    assert_eq!(registry.lock().usage().unwrap().bytes(), before - local);
    assert_eq!(
        *parent.scope.scratch.lock().unwrap(),
        scratch,
        "late capsule release cannot release ancestor scratch twice"
    );
}

#[test]
fn admission_callbacks_can_observe_local_receipts_and_release_unrelated_storage() {
    let (ledger, control, registry) = fixture();
    let operation = admitted(|admit| registry.begin(None, admit)).unwrap();
    let unrelated = std::cell::RefCell::new(Some(
        ledger
            .retain(custody::Usage::new(8191, 0), |_| Ok(()))
            .unwrap(),
    ));
    let calls = std::cell::Cell::new(0usize);
    let atom = operation
        .retain(
            "callback-name".into(),
            &mut Admission {
                global: &mut |_| {
                    let local = registry.lock().usage().unwrap().bytes();
                    assert!(local >= size_of::<Mutex<State>>());
                    drop(unrelated.borrow_mut().take());
                    calls.set(calls.get() + 1);
                    Ok(())
                },
                step: &mut || Ok(()),
            },
        )
        .unwrap();
    assert!(
        calls.get() > 0,
        "real custody admission reached the callback"
    );
    assert_eq!(atom.text(), &"callback-name".into());
    assert_raw_local(&registry);
    drop(operation);
    control.reconcile_releases();
    assert_eq!(registry.lock().index.entries(), 0);
}

#[test]
fn normalized_name_conversion_admits_actual_scratch_and_charges_each_completed_unit() {
    let (ledger, _control, registry) = fixture();
    let operation = admitted(|admit| registry.begin(None, admit)).unwrap();
    let scratch = *operation.scope.scratch.lock().unwrap();
    let baseline = ledger.total();
    let extent = size_of::<Vec<u16>>() + 4 * size_of::<u16>();
    let policy = |extra| {
        DomNameLimits::new(
            limits().retained_bytes(),
            limits().entries(),
            4,
            scratch + extra,
        )
        .unwrap()
    };
    operation.replace_limits(policy(extent - 1));
    let mut formed = Normalized::new(&operation);
    assert!(matches!(
        formed.units(&[65, 66, 0xd800, 0], true, None, &AtomicBool::new(false)),
        Err(DomError::ResourceLimit {
            resource: DomResource::NameScratch,
            ..
        })
    ));
    assert_eq!(
        *operation.scope.work.lock().unwrap(),
        0,
        "refusal precedes actual conversion reads"
    );
    assert_eq!(ledger.total(), baseline);
    operation.replace_limits(policy(extent));
    let value = formed
        .units(&[65, 66, 0xd800, 0], true, None, &AtomicBool::new(false))
        .unwrap();
    assert_eq!(value.code_units(), &[97, 98, 0xd800, 0]);
    assert_eq!(
        value.shared_code_units().capacity(),
        4,
        "actual exact-capacity premise"
    );
    assert_eq!(
        *operation.scope.work.lock().unwrap(),
        4,
        "real conversion read/copy uses the original cumulative name work"
    );
    assert_eq!(*operation.scope.scratch.lock().unwrap(), scratch + extent);
    drop(value);
    drop(formed);
    assert_eq!(*operation.scope.scratch.lock().unwrap(), scratch);
    assert_eq!(ledger.total(), baseline);
}
