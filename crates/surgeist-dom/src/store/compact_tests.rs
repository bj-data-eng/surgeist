//! Small raw allocation graphs for the compact-node transition. Expected
//! physical extents read actual objects/capacities, never cached Usage or Debit.
//! Arc control blocks and allocator metadata remain opaque and excluded.
use super::*;
use std::collections::BTreeMap;
mod empty_sequence_tests;

#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub(crate) struct AllocationGraph(BTreeMap<(usize, bool), (&'static str, usize)>);
impl AllocationGraph {
    pub(crate) fn allocation(&mut self, address: usize, kind: &'static str, bytes: usize) {
        // A zero-sized Arc target can sit one-past its opaque control block,
        // at the same address as a separate nonzero allocation. Keep the real
        // target count and zero extent without inferring that control block.
        if let Some(previous) = self.0.insert((address, bytes == 0), (kind, bytes)) {
            assert_eq!(
                previous,
                (kind, bytes),
                "one live allocation has one raw extent"
            );
        }
    }
    pub(crate) fn target<T: ?Sized>(&mut self, value: &Arc<T>, kind: &'static str) {
        self.allocation(
            Arc::as_ptr(value) as *const () as usize,
            kind,
            std::mem::size_of_val(&**value),
        );
    }
    pub(crate) fn buffer<T>(&mut self, values: &Vec<T>, kind: &'static str) {
        let bytes = values
            .capacity()
            .checked_mul(size_of::<T>())
            .expect("raw buffer extent");
        if bytes != 0 {
            self.allocation(values.as_ptr() as usize, kind, bytes);
        }
    }
    pub(crate) fn vector<T>(
        &mut self,
        values: &Arc<Vec<T>>,
        target: &'static str,
        buffer: &'static str,
    ) {
        self.target(values, target);
        self.buffer(values, buffer);
    }
    pub(crate) fn string(&mut self, value: &DocumentString) {
        self.vector(value.shared_code_units(), "string.target", "string.buffer");
    }
    pub(in crate::store) fn merge(&mut self, other: &Self) {
        for ((address, _), (kind, bytes)) in &other.0 {
            self.allocation(*address, kind, *bytes);
        }
    }
    pub(in crate::store) fn bytes(&self) -> usize {
        self.0
            .values()
            .try_fold(0usize, |sum, (_, bytes)| sum.checked_add(*bytes))
            .expect("raw graph extent")
    }
    pub(in crate::store) fn allocations(&self) -> usize {
        self.0.len()
    }
    fn count(&self, kind: &str) -> usize {
        self.0.values().filter(|(k, _)| *k == kind).count()
    }
    fn summary(&self) -> BTreeMap<&'static str, (usize, usize)> {
        let mut out = BTreeMap::<_, (usize, usize)>::new();
        for (kind, bytes) in self.0.values() {
            let entry = out.entry(*kind).or_default();
            entry.0 += 1;
            entry.1 = entry.1.checked_add(*bytes).expect("raw category extent");
        }
        out
    }
}

#[test]
fn zero_extent_target_addresses_do_not_alias_nonzero_storage() {
    let origin = Arc::new(());
    let address = Arc::as_ptr(&origin) as usize;
    let mut graph = AllocationGraph::default();
    graph.target(&origin, "document.origin");
    // Feed the observed collision shape directly: graph descriptors may share
    // an address while their zero/nonzero extents identify different storage.
    // This does not rely on the allocator arranging adjacent allocations.
    graph.allocation(address, "string.buffer", 80);
    assert_eq!(graph.count("document.origin"), 1);
    assert_eq!(graph.count("string.buffer"), 1);
    assert_eq!(graph.summary()["document.origin"], (1, 0));
    assert_eq!(graph.bytes(), 80);
    let before = graph.clone();
    graph.target(&origin.clone(), "document.origin");
    graph.merge(&before);
    assert_eq!(graph, before);
}

#[test]
#[should_panic(expected = "one live allocation has one raw extent")]
fn nonzero_allocation_aliases_reject_conflicting_extents() {
    let values = Vec::<u8>::with_capacity(17);
    let mut graph = AllocationGraph::default();
    graph.buffer(&values, "string.buffer");
    graph.allocation(
        values.as_ptr() as usize,
        "string.buffer",
        values.capacity() + 1,
    );
}

fn limits() -> DomLimits {
    DomLimits::new(
        32,
        100_000,
        128,
        0,
        0,
        16,
        1_000_000,
        crate::DomNameLimits::new(16_777_216, 16_384, 4_000_000, 1_048_576).unwrap(),
    )
    .unwrap()
}
fn document(dom: &mut Dom) -> DocumentId {
    let mut batch = DomBatch::new();
    let document = batch.create_document();
    dom.apply(batch, &AtomicBool::new(false))
        .unwrap()
        .document(&document)
        .unwrap()
}

fn node_storage(state: &Arc<State>) -> AllocationGraph {
    let mut graph = AllocationGraph::default();
    graph.target(state, "state.target");
    state.slots.inspect_raw_allocations(&mut graph);
    for (_, slot) in state.slots.iter() {
        let Some(node) = &slot.node else {
            continue;
        };
        match &node.kind {
            DomNodeKind::Document(value) => value.inspect_compact_baseline(&mut graph),
            DomNodeKind::Element(value) => value.inspect_compact_baseline(&mut graph),
            DomNodeKind::Text(value) | DomNodeKind::Comment(value) => graph.string(value),
            DomNodeKind::ProcessingInstruction(value) => value.inspect_compact_baseline(&mut graph),
            DomNodeKind::DocumentType(value) => {
                for string in [value.name(), value.public_id(), value.system_id()] {
                    graph.string(string);
                }
            }
            _ => panic!("compact baseline must describe this kind"),
        }
        for values in [&node.children, &node.manual_nodes, &node.assigned_nodes] {
            values.inspect_raw_allocations(&mut graph);
        }
        node.fullscreen_events.inspect_raw_allocations(&mut graph);
        if let Some(names) = &node.form_past_names {
            names.inspect_allocations(&mut graph);
            for entry in names.entries().iter() {
                graph.string(&entry.name);
            }
        }
        if let Some(document) = &node.base_document {
            assert!(document.facts.is_none());
            document.render_blocking.inspect_raw_allocations(&mut graph);
        }
        if let Some(canonical::Autofocus::Known { candidates, .. }) = &node.autofocus_document {
            candidates.inspect_raw_allocations(&mut graph);
        }
        if let Some(canonical::Interactive::Known {
            open_dialogs,
            top_layer,
            pending_top_layer_removals,
        }) = &node.interactive_document
        {
            for values in [open_dialogs, top_layer, pending_top_layer_removals] {
                values.inspect_raw_allocations(&mut graph);
            }
        }
        assert!(
            node.encoding_url_document
                .as_ref()
                .is_none_or(|value| value.facts.is_none())
        );
        assert!(
            node.base.is_none()
                && node.link.is_none()
                && node.image.is_none()
                && node.hyperlink.is_none()
        );
    }
    state.signal_slots.inspect_raw_allocations(&mut graph);
    state.native_tasks.inspect_raw_allocations(&mut graph);
    state.details_intentions.inspect_raw_allocations(&mut graph);
    graph
}

fn report(label: &str, dom: &Dom, graph: &AllocationGraph) {
    let mut controls = AllocationGraph::default();
    dom.custody_registry.inspect_raw_allocations(&mut controls);
    payload::inspect_attribute_child_allocations(&dom.state_custody.slots, &mut controls);
    let mut measured_union = graph.clone();
    measured_union.merge(&controls);
    eprintln!(
        "compact_baseline {label}: node_owned={:?} registry_and_control={:?} measured_union_bytes={} ledger_conservative={:?} slot_occurrences={:?}; node-owned scope excludes derived-index semantic pages, history disabled; opaque Arc/allocator metadata excluded",
        graph.summary(),
        controls.summary(),
        measured_union.bytes(),
        dom.custody_registry.ledger().total(),
        dom.state.slots.totals()
    );
    assert!(controls.count("control.record") > 0);
    assert_eq!(controls.count("registry.state"), 1);
    assert_eq!(controls.count("control.ledger"), 1);
    assert!(controls.count("control.debit") > 0);
}

#[test]
fn native_node_graphs_retain_real_banks_payloads_and_snapshot_union() {
    let cancel = AtomicBool::new(false);
    let mut dom = Dom::new(limits()).unwrap();
    let document = document(&mut dom);
    eprintln!(
        "compact_baseline layout Slot={} Node={} NodeFields={} DomNodeKind={} DomElement={} Document={} PI={} doctype={}; layouts accompany raw allocation graphs, not size quotas",
        size_of::<Slot>(),
        size_of::<Node>(),
        size_of::<transaction::NodeFields>(),
        size_of::<DomNodeKind>(),
        size_of::<DomElement>(),
        size_of::<DomDocumentMetadata>(),
        size_of::<DomProcessingInstruction>(),
        size_of::<DomDocumentType>()
    );
    let document_graph = node_storage(&dom.state);
    report("Document", &dom, &document_graph);
    let mut batch = DomBatch::new();
    let plain = batch.create_element(document, "plain");
    batch.create_element_ns(document, crate::Namespace::html(), "div");
    let text = batch.create_text(document, "shared text");
    batch.create_comment(document, "comment");
    batch.create_processing_instruction(document, "proc", "x='value'");
    batch.create_document_type(document, "root", "public", "system");
    let form = batch.create_element_ns(document, crate::Namespace::html(), "form");
    batch.create_element_ns(document, crate::Namespace::html(), "canvas");
    let done = dom.apply(batch, &cancel).unwrap();
    let plain = done.created(&plain).unwrap().unwrap();
    let text = done.created(&text).unwrap().unwrap();
    let form = done.created(&form).unwrap().unwrap();
    let held = dom.snapshot();
    let before = node_storage(&held.state);
    let mut clone_union = before.clone();
    clone_union.merge(&node_storage(&held.clone().state));
    assert_eq!(clone_union, before);
    assert_eq!(before.count("bank.record"), 9);
    assert!(before.count("bank.page") > 0 && before.count("bank.leaf") > 0);
    assert_eq!(before.count("native.canvas"), 1);
    // Four freshly created Elements have no attribute bank allocation. Their
    // four local atoms share one additional HTML namespace atom across the three
    // HTML names in this DOM, without atomizing the PI/doctype names.
    assert_eq!(before.count("attributes.target"), 0);
    assert_eq!(before.count("attributes.buffer"), 0);
    assert_eq!(before.count("name.record"), 5);
    report("all native kinds", &dom, &before);

    let mut batch = DomBatch::new();
    batch.set_attribute(plain, "id", "changed");
    dom.apply(batch, &cancel).unwrap();
    let after = node_storage(&dom.state);
    assert_eq!(after.count("attributes.target"), 1);
    assert_eq!(after.count("attributes.buffer"), 1);
    assert_eq!(after.count("name.record"), 6); // The independently interned id.
    let mut union = before.clone();
    union.merge(&after);
    assert!(union.bytes() >= before.bytes() && union.bytes() >= after.bytes());
    assert_eq!(node_storage(&held.state), before);
    assert_eq!(
        held.node(plain)
            .unwrap()
            .as_element()
            .unwrap()
            .attributes()
            .len(),
        0
    );
    assert_eq!(
        dom.snapshot()
            .node(plain)
            .unwrap()
            .as_element()
            .unwrap()
            .attributes()
            .len(),
        1
    );
    assert_eq!(
        held.node(form)
            .unwrap()
            .as_form()
            .unwrap()
            .past_names()
            .len(),
        0
    );
    report("attribute edit with held snapshot", &dom, &union);

    let mut batch = DomBatch::new();
    batch.destroy(text);
    dom.apply(batch, &cancel).unwrap();
    assert!(matches!(
        dom.snapshot().node(text),
        Err(DomError::StaleNode)
    ));
    assert!(matches!(
        held.node(text).unwrap().kind(),
        DomNodeKind::Text(_)
    ));
    let vacant = node_storage(&dom.state);
    assert_eq!(vacant.count("bank.record"), 9);
    report("vacant Slot", &dom, &vacant);
    let mut batch = DomBatch::new();
    let replacement = batch.create_text(document, "replacement");
    let done = dom.apply(batch, &cancel).unwrap();
    let replacement = done.created(&replacement).unwrap().unwrap();
    assert_eq!(replacement.slot, text.slot);
    assert_ne!(replacement.generation, text.generation);
    assert_eq!(node_storage(&held.state), before);
    report("reused Slot", &dom, &node_storage(&dom.state));
}

#[test]
#[ignore = "requires SURGEIST_DOM_STORAGE_COUNTERS=1; select explicitly"]
fn native_known_empty_form_history_has_no_target_allocation() {
    use crate::form::allocation_observation::Observation;
    let mut dom = Dom::new(limits()).unwrap();
    let document = document(&mut dom);
    let observation = Observation::begin();
    let mut batch = DomBatch::new();
    let form = batch.create_element_ns(document, crate::Namespace::html(), "form");
    let done = dom.apply(batch, &AtomicBool::new(false)).unwrap();
    let form = done.created(&form).unwrap().unwrap();
    let before_clones = observation.counts();
    let snapshot = dom.snapshot();
    let clone = snapshot.clone();
    assert_eq!(observation.counts(), before_clones);
    assert_eq!(
        clone
            .node(form)
            .unwrap()
            .as_form()
            .unwrap()
            .past_names()
            .len(),
        0
    );
    let mut graph = AllocationGraph::default();
    if let Some(names) = &dom.state.slots[form.slot]
        .node
        .as_ref()
        .unwrap()
        .form_past_names
    {
        names.inspect_allocations(&mut graph);
    }
    let actual = observation.finish();
    assert_eq!(
        actual.targets,
        graph.count("form.names.target"),
        "actual constructor observation agrees with surviving raw targets"
    );
    eprintln!(
        "known_empty_form actual={actual:?} raw={:?}; Arc control blocks excluded",
        graph.summary()
    );
    assert_eq!(
        actual.targets, 0,
        "known-empty Form history has no backing allocation"
    );
    assert_eq!(actual.target_bytes, 0);
    assert_eq!(actual.capacity_bytes, 0);
}

#[test]
fn absent_and_reserved_empty_form_history_are_content_equal() {
    use crate::form::{FormPastName, FormPastNames};
    let absent = FormPastNames::<usize>::fresh();
    let reserved = FormPastNames::from_entries(Vec::<FormPastName<usize>>::with_capacity(7));
    assert_eq!(absent, reserved);
    assert_eq!(reserved, absent);
}

#[test]
fn aliased_form_history_equality_does_not_compare_retained_entries() {
    use crate::form::{FormPastName, FormPastNames};
    use std::{cell::Cell, rc::Rc};

    #[derive(Debug)]
    struct Target {
        value: u8,
        comparisons: Rc<Cell<usize>>,
    }
    impl PartialEq for Target {
        fn eq(&self, other: &Self) -> bool {
            self.comparisons.set(self.comparisons.get() + 1);
            self.value == other.value
        }
    }
    impl Eq for Target {}

    let comparisons = Rc::new(Cell::new(0));
    let history = || {
        FormPastNames::from_entries(
            [("first", 3), ("second", 5)]
                .into_iter()
                .map(|(name, value)| FormPastName {
                    name: name.into(),
                    target: Target {
                        value,
                        comparisons: comparisons.clone(),
                    },
                })
                .collect(),
        )
    };
    let names = history();
    let alias = names.clone();
    assert_eq!(names, alias);
    assert_eq!(
        comparisons.get(),
        0,
        "unchanged lineage is an O(1) comparison"
    );
    let independent = history();
    assert_eq!(names, independent);
    assert_eq!(
        comparisons.get(),
        2,
        "the element equality witness is active"
    );
}

#[test]
fn reserved_and_populated_form_history_preserve_storage_order_and_clones() {
    use crate::form::{FormPastName, FormPastNames};
    for populated in [false, true] {
        let mut entries = Vec::<FormPastName<usize>>::with_capacity(7);
        if populated {
            entries.push(FormPastName {
                name: "second".into(),
                target: 9,
            });
            entries.push(FormPastName {
                name: "first".into(),
                target: 4,
            });
            entries.push(FormPastName {
                name: "second".into(),
                target: 9,
            });
        }
        let capacity = entries.capacity();
        let observation =
            scale_metrics::ENABLED.then(crate::form::allocation_observation::Observation::begin);
        let names = FormPastNames::from_entries(entries);
        let clone = names.clone();
        if let Some(observation) = observation {
            let actual = observation.finish();
            assert_eq!(actual.targets, 1);
            assert_eq!(
                actual.target_bytes,
                size_of::<payload::Payload<crate::form::History<usize>>>()
            );
            assert_eq!(
                actual.capacity_bytes,
                capacity * size_of::<FormPastName<usize>>()
            );
        }
        let mut graph = AllocationGraph::default();
        names.inspect_allocations(&mut graph);
        let before = graph.clone();
        clone.inspect_allocations(&mut graph);
        assert_eq!(graph, before);
        assert_eq!(graph.count("form.names.target"), 1);
        assert_eq!(graph.count("form.names.buffer"), 1);
        assert_eq!(
            graph.bytes(),
            size_of::<payload::Payload<crate::form::History<usize>>>()
                + capacity * size_of::<FormPastName<usize>>()
        );
        drop(names);
        assert_eq!(clone.capacity(), capacity);
        let actual: Vec<_> = clone
            .entries()
            .iter()
            .map(|entry| (entry.name.clone(), entry.target))
            .collect();
        let expected: Vec<(DocumentString, usize)> = if populated {
            vec![
                ("second".into(), 9),
                ("first".into(), 4),
                ("second".into(), 9),
            ]
        } else {
            Vec::new()
        };
        assert_eq!(actual, expected);
    }
}

#[test]
fn known_empty_form_snapshot_and_source_rollback_keep_original_root() {
    let cancel = AtomicBool::new(false);
    let mut dom = Dom::new(limits()).unwrap();
    let document = document(&mut dom);
    let mut batch = DomBatch::new();
    let form = batch.create_element_ns(document, crate::Namespace::html(), "form");
    let text = batch.create_text(document, "child");
    let done = dom.apply(batch, &cancel).unwrap();
    let form = done.created(&form).unwrap().unwrap();
    let text = done.created(&text).unwrap().unwrap();
    let held = dom.snapshot();
    let raw_before = node_storage(&held.state);
    let original = dom.state.clone();
    let policy = crate::DomSourceLimits::new(64_000_000, 1_000_000, 10_000_000, 32, 128, 128, 1);
    let batch = DomBatch::new();
    let mut draft = Draft::new_source(&dom, batch.owner, &batch.commands, policy, &cancel)
        .unwrap()
        .0;
    let mark = draft
        .source_checkpoint(draft.source_other_bytes, draft.source_other_units, &cancel)
        .unwrap();
    draft
        .pre_insert(
            Key::Existing(text),
            Key::Existing(form),
            None,
            limits(),
            &cancel,
        )
        .unwrap();
    assert_eq!(
        draft.node(Key::Existing(form)).unwrap().children.first(),
        Some(Key::Existing(text))
    );
    assert_eq!(
        draft
            .node(Key::Existing(form))
            .unwrap()
            .form_past_names
            .unwrap()
            .len(),
        0
    );
    let spent = draft.source_work.load(Ordering::Relaxed);
    mark.restore(&mut draft);
    assert!(draft.node(Key::Existing(form)).unwrap().children.is_empty());
    assert_eq!(draft.source_work.load(Ordering::Relaxed), spent);
    let prepared = draft.finish(&dom, &cancel).unwrap();
    let done = dom.publish(prepared, &cancel).unwrap();
    assert!(done.changes().changes().is_empty());
    assert!(Arc::ptr_eq(&dom.state, &original));
    assert_eq!(node_storage(&dom.state), raw_before);
    assert_eq!(
        held.node(form)
            .unwrap()
            .as_form()
            .unwrap()
            .past_names()
            .len(),
        0
    );
    assert_eq!(
        dom.snapshot()
            .node(form)
            .unwrap()
            .as_form()
            .unwrap()
            .past_names()
            .len(),
        0
    );
    assert_eq!(dom.snapshot().stamp(), held.stamp());
    let mut source = dom
        .begin_source_edit(
            crate::DomSourceEdit::set_attribute(form, "title", "changed"),
            policy,
            &cancel,
        )
        .unwrap();
    assert!(matches!(
        source.advance(&cancel).unwrap(),
        crate::DomSourceProgress::Complete(_)
    ));
    drop(source);
    assert_eq!(
        held.node(form)
            .unwrap()
            .as_element()
            .unwrap()
            .attributes()
            .len(),
        0
    );
    assert_eq!(
        dom.snapshot()
            .node(form)
            .unwrap()
            .as_element()
            .unwrap()
            .attributes()
            .len(),
        1
    );
    assert_eq!(
        held.node(form)
            .unwrap()
            .as_form()
            .unwrap()
            .past_names()
            .len(),
        0
    );
    assert_eq!(
        dom.snapshot()
            .node(form)
            .unwrap()
            .as_form()
            .unwrap()
            .past_names()
            .len(),
        0
    );
}
