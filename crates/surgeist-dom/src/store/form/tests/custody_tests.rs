use super::*;
use crate::form::{FormPastNames, History};

fn raw_extent(names: &FormPastNames<NodeId>) -> custody::Usage {
    let Some(payload) = names.payload() else {
        return custody::Usage::ZERO;
    };
    raw_history_extent(&payload.value.entries)
}
fn raw_history_extent<K>(entries: &Vec<FormPastName<K>>) -> custody::Usage {
    let mut bytes = size_of::<payload::Payload<History<K>>>()
        + entries.capacity() * size_of::<FormPastName<K>>();
    let mut units = 0;
    for entry in entries {
        let capacity = entry.name.shared_code_units().capacity();
        bytes += size_of::<Vec<u16>>() + capacity * size_of::<u16>();
        units += capacity;
    }
    custody::Usage::new(bytes, units)
}
fn names(dom: &Dom, form: NodeId) -> &FormPastNames<NodeId> {
    dom.state
        .node(dom.owner, form)
        .unwrap()
        .form_past_names
        .as_ref()
        .unwrap()
}
fn assert_dependency(custody: &transaction::StateCustody, expected: &FormPastNames<NodeId>) {
    let payload = expected.payload().expect("populated or reserved history");
    assert_eq!(
        payload::inspect_slot_form_custody(&custody.slots),
        [(payload.identity, raw_extent(expected))],
        "the actual Slot owner retains its Form child; a receipt calculation alone is insufficient"
    );
}

fn empty_form(reserved: bool) -> (Dom, NodeId) {
    let mut dom = Dom::new(
        DomLimits::new(
            8,
            4096,
            16,
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
    let form = batch.create_element_ns(&document, crate::Namespace::html(), "form");
    let cancel = AtomicBool::new(false);
    let mut draft = Draft::new(&dom, batch.owner.clone(), &cancel).unwrap();
    for command in &batch.commands {
        draft.apply(command, dom.limits, &cancel).unwrap();
    }
    if reserved {
        let value = FormPastNames::from_entries(Vec::with_capacity(7));
        let mut candidate = transaction::Names::canonical(&value);
        candidate
            .adopt(draft.custody_registry(), |usage| {
                draft.admit_custody_usage(usage, &cancel)
            })
            .unwrap();
        draft
            .with_node_fields(Key::Created(form.index), &cancel, |fields| {
                fields.form_past_names = Some(candidate);
            })
            .unwrap();
    }
    let prepared = draft.finish(&dom, &cancel).unwrap();
    let commit = dom.publish(prepared, &cancel).unwrap();
    let form = commit.created(&form).unwrap().unwrap();
    (dom, form)
}

#[test]
fn absent_and_reserved_empty_replacements_retain_exact_original_root_and_backing() {
    for reserved in [false, true] {
        for source in [false, true] {
            let (mut dom, form) = empty_form(reserved);
            let snapshot = dom.snapshot();
            let original = names(&dom, form).clone();
            assert_eq!(original.payload().is_some(), reserved);
            let cancel = AtomicBool::new(false);
            let batch = DomBatch::new();
            let mut draft = if source {
                Draft::new_source(
                    &dom,
                    batch.owner,
                    &batch.commands,
                    crate::DomSourceLimits::new(32_000_000, 100_000, 1_000_000, 16, 64, 64, 1),
                    &cancel,
                )
                .unwrap()
                .0
            } else {
                Draft::new(&dom, batch.owner, &cancel).unwrap()
            };
            let replacement = if reserved {
                FormPastNames::fresh()
            } else {
                FormPastNames::from_entries(Vec::with_capacity(7))
            };
            assert_eq!(original, replacement);
            assert!(!original.same_storage(&replacement));
            let mut candidate = transaction::Names::canonical(&replacement);
            candidate
                .adopt(draft.custody_registry(), |usage| {
                    draft.admit_custody_usage(usage, &cancel)
                })
                .unwrap();
            draft
                .with_node_fields(Key::Existing(form), &cancel, |fields| {
                    fields.form_past_names = Some(candidate)
                })
                .unwrap();
            let prepared = draft.finish(&dom, &cancel).unwrap();
            let commit = dom.publish(prepared, &cancel).unwrap();
            assert!(commit.changes().changes().is_empty());
            assert!(Arc::ptr_eq(&dom.state, &snapshot.state));
            assert_eq!(dom.snapshot().stamp(), snapshot.stamp());
            assert!(original.same_storage(names(&dom, form)));
            assert_eq!(names(&dom, form).capacity(), original.capacity());
            assert!(
                dom.changes_since(snapshot.stamp())
                    .unwrap()
                    .changes()
                    .is_empty()
            );
        }
    }
}

#[test]
fn ordinary_and_source_field_edits_share_the_actual_slot_form_dependency() {
    let (mut dom, _, form, button, field) = fixture();
    let snapshot = dom.snapshot();
    let original = names(&dom, form).clone();
    assert_dependency(&dom.state_custody, &original);
    let mut batch = DomBatch::new();
    batch.set_attribute(form, "title", "ordinary");
    dom.apply(batch, &AtomicBool::new(false)).unwrap();
    assert!(original.same_storage(names(&dom, form)));
    assert_dependency(&dom.state_custody, &original);
    let cancel = AtomicBool::new(false);
    let mut lease = dom
        .begin_source_edit(
            crate::DomSourceEdit::set_attribute(form, "title", "source"),
            crate::DomSourceLimits::new(32_000_000, 100_000, 1_000_000, 16, 64, 64, 1),
            &cancel,
        )
        .unwrap();
    assert!(matches!(
        lease.advance(&cancel).unwrap(),
        crate::DomSourceProgress::Complete(_)
    ));
    drop(lease);
    assert!(original.same_storage(names(&dom, form)));
    assert_dependency(&dom.state_custody, &original);
    let mut cleanup = DomBatch::new();
    cleanup.set_attribute(button, "form", "f");
    dom.apply(cleanup, &cancel).unwrap();
    assert!(!original.same_storage(names(&dom, form)));
    assert_dependency(&dom.state_custody, names(&dom, form));
    assert_eq!(names(&dom, form).entries()[0].target, field);
    assert_eq!(names(&dom, form).entries().len(), 1);
    assert_eq!(
        snapshot
            .node(form)
            .unwrap()
            .as_form()
            .unwrap()
            .past_names()
            .len(),
        3
    );
}

#[test]
fn mapped_output_excess_survives_slot_installation_and_releases_with_its_owner() {
    let (mut dom, _, form, button, field) = fixture();
    let cancel = AtomicBool::new(false);
    let batch = DomBatch::new();
    let mut draft = Draft::new_source(
        &dom,
        batch.owner,
        &batch.commands,
        crate::DomSourceLimits::new(32_000_000, 100_000, 1_000_000, 16, 64, 64, 1),
        &cancel,
    )
    .unwrap()
    .0;
    draft
        .remove_form_aliases(Key::Existing(form), Key::Existing(button), &cancel)
        .unwrap();
    let output = Vec::with_capacity(13);
    let capacity = output.capacity();
    let supplied = transaction::names::mapped_output_tests::SuppliedOutput::begin(output);
    let prepared = draft.finish(&dom, &cancel).unwrap();
    assert!(supplied.taken());
    drop(supplied);
    dom.publish(prepared, &cancel).unwrap();
    drop(draft);
    let current = names(&dom, form);
    assert_eq!(current.capacity(), capacity);
    assert_eq!(current.entries()[0].target, field);
    assert_eq!(current.entries().len(), 1);
    assert_eq!(current.own_usage(), raw_extent(current));
    assert_dependency(&dom.state_custody, current);
    let weak = Arc::downgrade(current.payload().unwrap());
    let ledger = dom.custody_registry.ledger().clone();
    drop(dom);
    assert!(weak.upgrade().is_none());
    assert_eq!(ledger.total(), custody::Usage::ZERO);
}

#[test]
fn persisted_mapped_excess_has_exact_and_one_short_historical_admission() {
    let (mut dom, _, form, button, field) = fixture();
    let cancel = AtomicBool::new(false);
    let batch = DomBatch::new();
    let mut draft = Draft::new_source(
        &dom,
        batch.owner,
        &batch.commands,
        crate::DomSourceLimits::new(32_000_000, 100_000, 1_000_000, 16, 64, 64, 1),
        &cancel,
    )
    .unwrap()
    .0;
    draft
        .remove_form_aliases(Key::Existing(form), Key::Existing(button), &cancel)
        .unwrap();
    let output = Vec::with_capacity(13);
    let capacity = output.capacity();
    let supplied = transaction::names::mapped_output_tests::SuppliedOutput::begin(output);
    let prepared = draft.finish(&dom, &cancel).unwrap();
    assert!(supplied.taken());
    drop(supplied);
    dom.publish(prepared, &cancel).unwrap();
    drop(draft);
    let external = names(&dom, form).clone();
    assert_eq!(external.entries().len(), 1);
    assert_eq!(external.entries()[0].target, field);
    assert_eq!(external.entries()[0].name, DocumentString::from("keep"));
    assert_eq!(external.capacity(), capacity);
    assert!(capacity > external.entries().len());
    assert_dependency(&dom.state_custody, &external);
    let ledger = dom.custody_registry.ledger().clone();
    drop(dom);
    assert_eq!(ledger.total(), custody::Usage::ZERO);

    // This persisted object now survives only in external semantics. The
    // Form adopter used by Slot custody must charge its actual excess again.
    let registry = bank::CustodyRegistry::new(&ledger, |_| Ok(())).unwrap();
    let baseline = ledger.total();
    assert_eq!(baseline, registry.inspect_storage());
    let payload = external.payload().unwrap();
    let raw = raw_extent(&external);
    let peak = std::cell::Cell::new(custody::Usage::ZERO);
    let controlled = registry
        .adopt_object(payload.identity, payload.own_usage, |next| {
            let old = peak.get();
            peak.set(custody::Usage::new(
                old.bytes().max(next.bytes()),
                old.units().max(next.units()),
            ));
            Ok(())
        })
        .unwrap();
    // The oracle walks actual control targets, receipt bodies, release nodes,
    // registry pages/capacities and weak occurrences. It reads no cached charge.
    let exact = raw
        .checked_add(controlled.inspect_control_graph().usage())
        .unwrap()
        .checked_add(registry.inspect_storage())
        .unwrap();
    assert_eq!(ledger.total(), exact);
    assert_eq!(
        peak.get(),
        exact,
        "the independently enumerated union also covers the reached staging peak"
    );
    assert_eq!(controlled.inspect_payload_usage(), raw);
    drop(controlled);
    registry.reconcile_releases();
    assert_eq!(ledger.total(), baseline);

    // Reuse these SAME materializations and registry, so checked issuer/path
    // geometry cannot change the exact/one-short comparison.
    for short in [true, false] {
        let limit = exact.bytes() - usize::from(short);
        let maximum = std::cell::Cell::new(0usize);
        let result = registry.adopt_object(payload.identity, payload.own_usage, |next| {
            maximum.set(maximum.get().max(next.bytes()));
            if next.bytes() > limit {
                Err(DomError::ResourceLimit {
                    resource: DomResource::SourceRetainedBytes,
                    limit,
                })
            } else {
                Ok(())
            }
        });
        if short {
            assert!(matches!(
                result,
                Err(DomError::ResourceLimit {
                    resource: DomResource::SourceRetainedBytes,
                    limit: actual,
                }) if actual == limit
            ));
        } else {
            let control = result.unwrap();
            assert_eq!(control.inspect_payload_usage(), raw);
            assert_eq!(ledger.total(), exact);
            drop(control);
        }
        assert_eq!(maximum.get(), exact.bytes());
        registry.reconcile_releases();
        assert_eq!(ledger.total(), baseline);
        assert_eq!(external.capacity(), capacity);
        assert_eq!(external.entries()[0].target, field);
    }
    let weak = Arc::downgrade(payload);
    let destruction =
        payload::destruction_observation::Observation::begin(payload.identity, &ledger);
    drop(external);
    assert!(weak.upgrade().is_none());
    assert_eq!(destruction.observed(), Some(baseline));
    drop(destruction);
    drop(registry);
    assert_eq!(ledger.total(), custody::Usage::ZERO);
}

#[test]
fn fresh_mapped_form_sealing_refusal_drops_data_before_credit_and_keeps_source_draft() {
    let (mut dom, _, form, button, field) = fixture();
    let snapshot = dom.snapshot();
    let original = names(&dom, form).clone();
    let cancel = AtomicBool::new(false);
    let batch = DomBatch::new();
    let mut draft = Draft::new_source(
        &dom,
        batch.owner,
        &batch.commands,
        crate::DomSourceLimits::new(32_000_000, 100_000, 1_000_000, 16, 64, 64, 1),
        &cancel,
    )
    .unwrap()
    .0;
    draft
        .remove_form_aliases(Key::Existing(form), Key::Existing(button), &cancel)
        .unwrap();
    let staged = draft
        .records_for(Key::Existing(form))
        .fields
        .value()
        .form_past_names
        .as_ref()
        .unwrap()
        .clone();
    let staged_identity = staged.identity().unwrap();
    let map = |key| match key {
        Key::Existing(id) => id,
        Key::Created(_) => panic!("this real edit creates no nodes"),
    };
    draft.custody_registry().reconcile_releases();
    let before = draft.custody_ledger().total();
    let work = draft.source_work.clone();
    let spent = work.load(Ordering::Relaxed);
    for refusal in 0..2 {
        let supplied =
            transaction::names::mapped_output_tests::SuppliedOutput::begin(Vec::with_capacity(13));
        let prepared = staged.prepare_projection(&draft, &map, &cancel).unwrap();
        assert!(supplied.taken());
        drop(supplied);
        let transaction::names::PreparedNames::Pending(pending) = prepared else {
            panic!("a changed nonempty Form map must project a fresh materialization")
        };
        let projected = pending.value();
        assert_ne!(projected.identity, staged_identity);
        assert_eq!(projected.value.entries.len(), 1);
        assert_eq!(projected.value.entries[0].target, field);
        assert_eq!(
            projected.value.entries[0].name,
            DocumentString::from("keep")
        );
        assert!(projected.value.entries.capacity() > projected.value.entries.len());
        let body = raw_history_extent(&projected.value.entries);
        assert_eq!(
            draft.custody_ledger().total(),
            before.checked_add(body).unwrap()
        );
        let destruction = payload::destruction_observation::Observation::begin(
            projected.identity,
            draft.custody_ledger(),
        );
        let mut calls = 0;
        let result = pending.seal(draft.custody_registry(), |next| {
            draft.admit_custody_usage(next, &cancel)?;
            let call = calls;
            calls += 1;
            if call == refusal {
                Err(DomError::Cancelled)
            } else {
                Ok(())
            }
        });
        assert!(matches!(result, Err(DomError::Cancelled)));
        assert_eq!(calls, refusal + 1);
        // Gate zero refuses shared receipt storage. Gate one refuses control
        // registration with that receipt already live beside the Pending body.
        let during = before
            .checked_add(body)
            .unwrap()
            .checked_add(custody::Usage::new(
                refusal * size_of::<custody::Debit>(),
                0,
            ))
            .unwrap();
        assert_eq!(destruction.observed(), Some(during));
        drop(destruction);
        draft.custody_registry().reconcile_releases();
        assert_eq!(draft.custody_ledger().total(), before);
        assert!(Arc::ptr_eq(&dom.state, &snapshot.state));
        assert!(original.same_storage(names(&dom, form)));
        assert_eq!(original.entries().len(), 3);
        assert_eq!(staged.identity(), Some(staged_identity));
        let pending = draft
            .node(Key::Existing(form))
            .unwrap()
            .form_past_names
            .unwrap();
        assert_eq!(pending.len(), 1);
        assert_eq!(pending.iter().next().unwrap().target, Key::Existing(field));
        assert!(Arc::ptr_eq(&draft.source_work, &work));
        assert!(work.load(Ordering::Relaxed) >= spent);
    }
    // Retry the original staged edit through the complete publication path.
    // This is Source-Draft preservation, not an acknowledged external tail.
    let prepared = draft.finish(&dom, &cancel).unwrap();
    dom.publish(prepared, &cancel).unwrap();
    assert_dependency(&dom.state_custody, names(&dom, form));
    assert_eq!(names(&dom, form).entries().len(), 1);
    assert_eq!(names(&dom, form).entries()[0].target, field);
    assert_eq!(original.entries().len(), 3);
}

#[test]
fn refused_slot_dependency_preparation_preserves_the_original_controlled_union() {
    let (dom, _, form, _, _) = fixture();
    let slot = dom.state.slots.get(form.slot).unwrap();
    dom.custody_registry.reconcile_releases();
    let before = dom.custody_registry.total();
    for refusal in 0..2 {
        let mut calls = 0;
        let result = payload::slot_dependency(slot, &dom.custody_registry, None, |_| {
            let current = calls;
            calls += 1;
            if current == refusal {
                Err(DomError::Cancelled)
            } else {
                Ok(())
            }
        });
        assert!(matches!(result, Err(DomError::Cancelled)));
        assert_eq!(calls, refusal + 1);
        dom.custody_registry.reconcile_releases();
        assert_eq!(dom.custody_registry.total(), before);
        assert_dependency(&dom.state_custody, names(&dom, form));
    }
}

#[test]
fn nested_source_restore_keeps_the_original_slot_child_until_final_publication() {
    let (mut dom, _, form, button, field) = fixture();
    let original = names(&dom, form).clone();
    let cancel = AtomicBool::new(false);
    let batch = DomBatch::new();
    let mut draft = Draft::new_source(
        &dom,
        batch.owner,
        &batch.commands,
        crate::DomSourceLimits::new(32_000_000, 100_000, 1_000_000, 16, 64, 64, 1),
        &cancel,
    )
    .unwrap()
    .0;
    let outer = draft
        .source_checkpoint(draft.source_other_bytes, draft.source_other_units, &cancel)
        .unwrap();
    draft
        .remove_form_aliases(Key::Existing(form), Key::Existing(button), &cancel)
        .unwrap();
    let inner = draft
        .source_checkpoint(draft.source_other_bytes, draft.source_other_units, &cancel)
        .unwrap();
    draft
        .remove_form_aliases(Key::Existing(form), Key::Existing(field), &cancel)
        .unwrap();
    assert_eq!(
        draft
            .node(Key::Existing(form))
            .unwrap()
            .form_past_names
            .unwrap()
            .len(),
        0
    );
    inner.restore(&mut draft);
    assert_eq!(
        draft
            .node(Key::Existing(form))
            .unwrap()
            .form_past_names
            .unwrap()
            .len(),
        1
    );
    assert_dependency(&dom.state_custody, &original);
    outer.restore(&mut draft);
    assert_eq!(
        draft
            .node(Key::Existing(form))
            .unwrap()
            .form_past_names
            .unwrap()
            .len(),
        3
    );
    assert_dependency(&dom.state_custody, &original);
    draft
        .remove_form_aliases(Key::Existing(form), Key::Existing(button), &cancel)
        .unwrap();
    let prepared = draft.finish(&dom, &cancel).unwrap();
    dom.publish(prepared, &cancel).unwrap();
    assert_dependency(&dom.state_custody, names(&dom, form));
    assert_eq!(names(&dom, form).entries()[0].target, field);
    assert_eq!(original.entries().len(), 3);
}

#[test]
fn final_controlled_release_and_historical_readoption_distinguish_semantic_lifetime() {
    let (dom, _, form, _, _) = fixture();
    let snapshot = dom.snapshot();
    let original = names(&dom, form);
    let identity = original.payload().unwrap().identity;
    let weak = Arc::downgrade(original.payload().unwrap());
    assert_dependency(&dom.state_custody, original);
    let ledger = dom.custody_registry.ledger().clone();
    drop(dom);
    assert_eq!(ledger.total(), custody::Usage::ZERO);
    assert!(
        weak.upgrade().is_some(),
        "an external semantic snapshot remains physical"
    );
    assert_eq!(
        snapshot
            .node(form)
            .unwrap()
            .as_form()
            .unwrap()
            .past_names()
            .len(),
        3
    );
    let registry = bank::CustodyRegistry::new(&ledger, |_| Ok(())).unwrap();
    let baseline = ledger.total();
    let captured =
        transaction::StateCustody::adopt(&snapshot.state, &registry, None, |_| Ok(()), || Ok(()))
            .unwrap();
    let historical = snapshot
        .node(form)
        .unwrap()
        .node
        .form_past_names
        .as_ref()
        .unwrap();
    assert_eq!(historical.payload().unwrap().identity, identity);
    assert_dependency(&captured, historical);
    drop(captured);
    registry.reconcile_releases();
    assert_eq!(ledger.total(), baseline);
    let destruction = payload::destruction_observation::Observation::begin(identity, &ledger);
    drop(snapshot);
    assert!(weak.upgrade().is_none());
    assert_eq!(destruction.observed(), Some(baseline));
    drop(destruction);
    drop(registry);
    assert_eq!(ledger.total(), custody::Usage::ZERO);
}

#[test]
fn owned_form_payload_enters_destruction_before_its_controlled_receipt_releases() {
    let (dom, _, form, _, _) = fixture();
    let original = names(&dom, form);
    let identity = original.payload().unwrap().identity;
    let extent = raw_extent(original);
    let weak = Arc::downgrade(original.payload().unwrap());
    let ledger = dom.custody_registry.ledger().clone();
    let destruction = payload::destruction_observation::Observation::begin(identity, &ledger);
    drop(dom);
    assert!(weak.upgrade().is_none());
    let during = destruction
        .observed()
        .expect("the actual Form payload destructor ran");
    assert!(during.bytes() >= extent.bytes());
    assert!(during.units() >= extent.units());
    assert_eq!(ledger.total(), custody::Usage::ZERO);
}
