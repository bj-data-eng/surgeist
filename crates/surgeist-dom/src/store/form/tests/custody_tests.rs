use super::*;
use crate::form::{FormPastNames, History};

fn raw_extent(names: &FormPastNames<NodeId>) -> custody::Usage {
    let Some(payload) = names.payload() else {
        return custody::Usage::ZERO;
    };
    let mut bytes = size_of::<payload::Payload<History<NodeId>>>()
        + payload.value.entries.capacity() * size_of::<FormPastName<NodeId>>();
    let mut units = 0;
    for entry in &payload.value.entries {
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
    let mut dom = Dom::new(DomLimits::new(8, 4096, 16, 0, 0, 8, 1_000_000).unwrap()).unwrap();
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
fn refused_slot_dependency_preparation_preserves_the_original_controlled_union() {
    let (dom, _, form, _, _) = fixture();
    let slot = dom.state.slots.get(form.slot).unwrap();
    dom.custody_registry.reconcile_releases();
    let before = dom.custody_registry.total();
    for refusal in 0..2 {
        let mut calls = 0;
        let result = payload::slot_dependency(slot, &dom.custody_registry, |_| {
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
        transaction::StateCustody::adopt(&snapshot.state, &registry, |_| Ok(()), || Ok(()))
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
