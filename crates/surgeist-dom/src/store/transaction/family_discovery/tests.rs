//! New private-owner characterizations. The original public/native discovery
//! failures remain unchanged in the owning Details and Select tests.
use super::*;

fn limits() -> DomLimits {
    DomLimits::new(256, 65536, 1024, 256, 128, 16, 2_000_000)
        .unwrap()
        .with_details_toggle_intentions(32)
        .unwrap()
        .with_select_processing(128, 8_000_000, 10_000_000)
        .unwrap()
}
fn rows(draft: &Draft, root: RootHeader, cancel: &AtomicBool) -> Vec<Key> {
    let mut result = Vec::new();
    let mut next = draft
        .families
        .first(root, &mut || draft.source_step(cancel))
        .unwrap();
    while let Some(key) = next {
        result.push(key);
        next = draft
            .families
            .after(root, key, draft.order_view(), &mut || {
                draft.source_step(cancel)
            })
            .unwrap();
    }
    result
}

#[test]
fn canonical_participants_recompute_actual_payload_and_keep_exact_names_and_order() {
    let cancel = AtomicBool::new(false);
    let mut dom = Dom::new(limits()).unwrap();
    let raw = DocumentString::from_utf16(vec![0xd800]);
    let mut batch = DomBatch::new();
    let document = batch.create_document();
    let root = batch.create_element_ns(&document, crate::Namespace::html(), "div");
    let first = batch.create_element_ns(&document, crate::Namespace::html(), "details");
    batch.set_attribute(&first, "name", raw.clone());
    batch.insert_before(&first, &root, None);
    let other = batch.create_element_ns(&document, crate::Namespace::html(), "details");
    batch.set_attribute(&other, "name", "\u{fffd}");
    batch.insert_before(&other, &root, None);
    let last = batch.create_element_ns(&document, crate::Namespace::html(), "details");
    batch.set_attribute(&last, "name", raw.clone());
    batch.insert_before(&last, &root, None);
    let select = batch.create_element_ns(&document, crate::Namespace::html(), "select");
    let option = batch.create_element_ns(&document, crate::Namespace::html(), "option");
    batch.insert_before(&option, &select, None);
    let target = batch.create_element_ns(&document, crate::Namespace::html(), "selectedcontent");
    batch.insert_before(&target, &select, None);
    let commit = dom.apply(batch, &cancel).unwrap();
    let key = |ticket: &CreatedNode| Key::Existing(commit.created(ticket).unwrap().unwrap());
    let draft = Draft::new(&dom, DomBatch::new().owner, &cancel).unwrap();
    let exact = draft
        .families
        .details(key(&root), &raw, &mut || draft.source_step(&cancel))
        .unwrap();
    assert_eq!(rows(&draft, exact, &cancel), [key(&first), key(&last)]);
    assert_eq!(
        rows(&draft, draft.families.options(key(&select)), &cancel),
        [key(&option)]
    );
    assert_eq!(
        rows(&draft, draft.families.targets(key(&select)), &cancel),
        [key(&target)]
    );
    let snapshot = dom.snapshot();
    let index = &snapshot.state.family_index;
    let graph = index.inspect_graph().unwrap();
    let records = graph.record_totals();
    let raw_usage = custody::Usage::new(graph.index_charge().bytes + records.bytes, records.units);
    assert!(records.occurrences > 0);
    assert_eq!(index.heap_usage().unwrap(), raw_usage);
}

#[test]
fn one_stored_enabled_target_has_separate_containing_groups_and_source_undo() {
    let cancel = AtomicBool::new(false);
    let mut dom = Dom::new(limits()).unwrap();
    let mut batch = DomBatch::new();
    let document = batch.create_document();
    let outer = batch.create_element_ns(&document, crate::Namespace::html(), "select");
    let inner = batch.create_element_ns(&document, crate::Namespace::html(), "select");
    let target = batch.create_element_ns(&document, crate::Namespace::html(), "selectedcontent");
    batch.insert_before(&target, &inner, None);
    batch.insert_before(&inner, &outer, None);
    let commit = dom.apply(batch, &cancel).unwrap();
    let key = |ticket: &CreatedNode| Key::Existing(commit.created(ticket).unwrap().unwrap());
    let (outer, inner, target) = (key(&outer), key(&inner), key(&target));
    assert!(
        dom.snapshot()
            .node(match target {
                Key::Existing(id) => id,
                _ => unreachable!(),
            })
            .unwrap()
            .as_selectedcontent()
            .unwrap()
            .disabled()
    );
    let empty = DomBatch::new();
    let mut draft = Draft::new_source(
        &dom,
        empty.owner,
        &empty.commands,
        crate::DomSourceLimits::new(500_000_000, 20_000_000, 100_000_000, 64, 4096, 4096, 1),
        &cancel,
    )
    .unwrap()
    .0;
    let checkpoint = draft
        .source_checkpoint(draft.source_other_bytes, draft.source_other_units, &cancel)
        .unwrap();
    // This characterizes the private field barrier, not an invented native
    // refresh schedule. Only the actual stored bit changes; ancestry stays put.
    draft
        .with_element_fields(target, &cancel, |element| {
            *element.selectedcontent_disabled_mut().unwrap() = false
        })
        .unwrap();
    assert_eq!(
        rows(&draft, draft.families.targets(outer), &cancel),
        [target]
    );
    assert_eq!(
        rows(&draft, draft.families.targets(inner), &cancel),
        [target]
    );
    checkpoint.restore(&mut draft);
    assert!(rows(&draft, draft.families.targets(outer), &cancel).is_empty());
    assert!(rows(&draft, draft.families.targets(inner), &cancel).is_empty());
    assert_eq!(
        draft.element(target).unwrap().selectedcontent_disabled(),
        Some(true)
    );
}

#[test]
fn dirty_identity_record_has_an_exact_first_gate_and_refusal_keeps_the_pair_empty() {
    for short in [true, false] {
        let ledger = custody::Ledger::default();
        let registry = bank::CustodyRegistry::new(&ledger, |_| Ok(())).unwrap();
        let baseline = ledger.total();
        let mut keys = Keys::default();
        // This prices the actual first dirty Key record and its Debit target,
        // not the later persistent index/registry paths or a whole operation.
        let first = custody::Usage::new(size_of::<Key>() + size_of::<custody::Debit>(), 0);
        let limit = baseline.bytes() + first.bytes() - usize::from(short);
        let calls = std::cell::Cell::new(0);
        let exact_first_passed = std::cell::Cell::new(false);
        let result = keys.values.change(
            &mut keys.custody,
            Key::Created(7),
            Some(Key::Created(7)),
            &mut Edit {
                registry: &registry,
                admit: &mut |usage| {
                    calls.set(calls.get() + 1);
                    if calls.get() == 1 {
                        assert_eq!(usage, baseline.checked_add(first).unwrap());
                        if usage.bytes() <= limit {
                            exact_first_passed.set(true);
                        }
                    }
                    if usage.bytes() > limit {
                        Err(DomError::ResourceLimit {
                            resource: DomResource::SourceRetainedBytes,
                            limit,
                        })
                    } else {
                        Ok(())
                    }
                },
                step: &mut || Ok(()),
            },
        );
        assert!(matches!(
            result,
            Err(DomError::ResourceLimit {
                resource: DomResource::SourceRetainedBytes,
                ..
            })
        ));
        assert_eq!(exact_first_passed.get(), !short);
        assert!(keys.values.get(Key::Created(7)).is_none());
        registry.reconcile_releases();
        assert_eq!(ledger.total(), baseline);
    }
}
