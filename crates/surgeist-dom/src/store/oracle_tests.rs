//! Full canonical recomputation for the sparse publication tests.
//!
//! Root wires this test-only module after its producer seams exist:
//! - `store::full_difference(owner, old, new, cancel)` retains the former full
//!   comparison loops, without consulting journal/dirty/dependency indexes.
//! - `Bank::inspect_graph(...).index_charge()` walks actual Page/Vec storage
//!   without summaries; the supplied descriptor recomputes each record.
//!
//! Nothing here is a production fallback or a second mutable tree.

use std::collections::{BTreeMap, HashMap, HashSet};
use std::mem::size_of;
use std::sync::atomic::AtomicBool;

use super::{DomUsage, State, bank, canonical, custody, source, transaction};
use crate::{
    DocumentId, DocumentString, Dom, DomActiveRangeState, DomBatch, DomChange, DomCommit, DomError,
    DomLimits, DomNodeKind, DomOwnerId, DomSnapshot, DomSourceEdit, DomSourceLimits,
    DomSourceProgress, NodeId,
};

fn add(total: &mut usize, value: usize) {
    *total = total
        .checked_add(value)
        .expect("oracle representation capacity");
}

// Immutable producer descriptors are allowed at crate ownership boundaries;
// DOM's mutable receipts and aggregate summaries are never expected values.
fn full_document_units(metadata: &crate::DomDocumentMetadata) -> usize {
    let mut units =
        metadata.content_type().len() + metadata.encoding().len() + metadata.url().len();
    if let Some(language) = metadata
        .meta_state()
        .and_then(|state| state.pragma_language())
    {
        add(&mut units, language.shared_code_units().capacity());
    }
    if let Some(Some(selected)) = &metadata.meta_color_scheme {
        add(&mut units, selected.report.retained_storage().utf16_units());
    }
    units
}
fn qualified_units(name: &crate::QualifiedName) -> usize {
    name.local_name().len()
        + name.prefix().map_or(0, DocumentString::len)
        + name.namespace().uri().map_or(0, DocumentString::len)
}
fn full_element_units(element: &crate::DomElement) -> usize {
    let mut units = qualified_units(element.name());
    add(
        &mut units,
        element.saved_is().map_or(0, DocumentString::len),
    );
    add(&mut units, element.nonce().map_or(0, DocumentString::len));
    add(&mut units, element.family_units().unwrap());
    for attribute in element.attributes().iter() {
        add(&mut units, qualified_units(attribute.name()));
        add(&mut units, attribute.value().len());
    }
    units
}
fn full_kind_payload(kind: &DomNodeKind) -> source::storage::Payload {
    let mut out = source::storage::Payload::default();
    match kind {
        DomNodeKind::Document(metadata) => {
            for string in [
                Some(metadata.content_type()),
                Some(metadata.encoding()),
                Some(metadata.url()),
                metadata
                    .meta_state()
                    .and_then(|state| state.pragma_language()),
            ]
            .into_iter()
            .flatten()
            {
                out.string(string).unwrap();
            }
            if let Some(Some(selected)) = &metadata.meta_color_scheme {
                let actual = selected.report.retained_storage();
                out.add_bytes(actual.bytes()).unwrap();
                out.add_units(actual.utf16_units()).unwrap();
            }
            if let crate::DomDocumentCreationCause::ParsedOffline { url, .. } =
                metadata.creation_cause()
            {
                out.producer(url.retained_storage_usage()).unwrap();
            }
            if metadata.document_environment().hosted.is_some() {
                out.add_bytes(size_of::<crate::hosted::DocumentAssociation>())
                    .unwrap();
            }
            if let Some(inputs) = &metadata.hosted_inputs {
                out.add_bytes(inputs.payload_bytes().unwrap()).unwrap();
                out.add_units(inputs.payload_units().unwrap()).unwrap();
            }
        }
        DomNodeKind::Element(element) => {
            out.name(element.name()).unwrap();
            out.add_bytes(size_of::<Vec<crate::Attribute>>()).unwrap();
            out.bank::<crate::Attribute>(element.attributes().shared_attributes().capacity())
                .unwrap();
            for attribute in element.attributes().iter() {
                out.name(attribute.name()).unwrap();
                out.string(attribute.value()).unwrap();
            }
            for value in [element.saved_is(), element.nonce()].into_iter().flatten() {
                out.string(value).unwrap();
            }
            if let Some(state) = element.fieldset_state() {
                out.string(&state.control.message).unwrap();
            }
            if let Some(state) = element.button_state() {
                out.string(&state.control.message).unwrap();
            }
            if let Some(state) = element.input_state() {
                out.add_bytes(size_of::<crate::DomInputState>()).unwrap();
                out.string(&state.current).unwrap();
                out.string(&state.control.message).unwrap();
                if let surgeist_html::HtmlInputImageHistory::Last {
                    source: Some(value),
                    ..
                } = &state.image
                {
                    out.string(value).unwrap();
                }
            }
            if let Some(state) = element.textarea_state() {
                out.add_bytes(size_of::<crate::DomTextareaState>()).unwrap();
                out.string(&state.raw).unwrap();
                out.string(&state.control.message).unwrap();
            }
            if let Some(state) = element.output_state() {
                out.add_bytes(size_of::<crate::DomOutputState>()).unwrap();
                out.string(&state.control.message).unwrap();
                if let Some(value) = &state.default_override {
                    out.string(value).unwrap();
                }
            }
            if let Some(state) = element.select_state() {
                out.add_bytes(size_of::<crate::DomSelectState>()).unwrap();
                out.string(&state.control.message).unwrap();
            }
            if let Some(state) = element.dialog_state() {
                out.add_bytes(size_of::<crate::dialog::DomDialogState>())
                    .unwrap();
                out.string(state.return_value()).unwrap();
                if let Some(value) = &state.request_return_value {
                    out.string(value).unwrap();
                }
            }
            if element.style_state().is_some() {
                out.add_bytes(size_of::<crate::DomStyleState>()).unwrap();
            }
            if let Some(state) = element.canvas_none_state() {
                out.add_bytes(size_of::<surgeist_html::HtmlCanvasNoneState>())
                    .unwrap();
                for dimension in [state.width(), state.height()] {
                    if let surgeist_html::HtmlCanvasBitmapDimension::Parsed(value) = dimension {
                        out.add_bytes(value.retained_buffer_bytes()).unwrap();
                        out.add_units(value.retained_buffer_bytes()).unwrap();
                    }
                }
            }
        }
        DomNodeKind::Text(value) | DomNodeKind::Comment(value) => out.string(value).unwrap(),
        DomNodeKind::ProcessingInstruction(value) => out
            .append(value.retained_payload(|| Ok(())).unwrap())
            .unwrap(),
        DomNodeKind::DocumentType(value) => {
            for string in [value.name(), value.public_id(), value.system_id()] {
                out.string(string).unwrap();
            }
        }
        DomNodeKind::DocumentFragment | DomNodeKind::ShadowRoot(_) => {}
    }
    out
}
fn full_names_payload(node: &super::Node) -> source::storage::Payload {
    let mut out = source::storage::Payload::default();
    if let Some(names) = &node.form_past_names {
        if names.entries.has_backing() {
            out.add_bytes(size_of::<Vec<crate::form::FormPastName<NodeId>>>())
                .unwrap();
        }
        out.bank::<crate::form::FormPastName<NodeId>>(names.entries.capacity())
            .unwrap();
        for entry in names.entries.iter() {
            out.string(&entry.name).unwrap();
        }
    }
    out
}
fn full_fields_payload(node: &super::Node) -> source::storage::Payload {
    let mut out = full_kind_payload(&node.kind);
    if let Some(frozen) = node.base.as_ref().and_then(|state| state.frozen.as_ref()) {
        out.producer(frozen.value.retained_storage_usage()).unwrap();
    }
    if let Some(facts) = node
        .base_document
        .as_ref()
        .and_then(|state| state.facts.as_ref())
    {
        out.producer(facts.retained_storage_usage()).unwrap();
    }
    if let Some(parsed) = node
        .hyperlink
        .as_ref()
        .and_then(|state| state.parsed.as_ref())
    {
        let actual = parsed.value.retained_storage_usage();
        out.add_bytes(
            actual.retained_bytes()
                - size_of::<surgeist_html_semantics::HtmlHyperlinkUrlSnapshot>(),
        )
        .unwrap();
        out.add_units(actual.utf16_extent()).unwrap();
    }
    if let Some(facts) = node
        .encoding_url_document
        .as_ref()
        .and_then(|state| state.facts.as_ref())
    {
        out.producer(facts.retained_storage_usage()).unwrap();
    }
    if let Some(image) = &node.image {
        out.string(image.native.current_request().current_url())
            .unwrap();
    }
    out.append(full_names_payload(node)).unwrap();
    out
}
fn sequence<T>(out: &mut source::storage::Payload, values: &super::SharedVec<T>) {
    if values.has_backing() {
        out.add_bytes(size_of::<Vec<T>>()).unwrap();
        out.bank::<T>(values.capacity()).unwrap();
    }
}
fn full_node_payload(node: &super::Node) -> source::storage::Payload {
    let mut out = full_fields_payload(node);
    sequence(&mut out, &node.children);
    sequence(&mut out, &node.manual_nodes);
    sequence(&mut out, &node.assigned_nodes);
    sequence(&mut out, &node.fullscreen_events);
    if let Some(document) = &node.base_document {
        sequence(&mut out, &document.render_blocking);
    }
    if let Some(canonical::Autofocus::Known { candidates, .. }) = &node.autofocus_document {
        sequence(&mut out, candidates);
    }
    if let Some(canonical::Interactive::Known {
        open_dialogs,
        top_layer,
        pending_top_layer_removals,
    }) = &node.interactive_document
    {
        sequence(&mut out, open_dialogs);
        sequence(&mut out, top_layer);
        sequence(&mut out, pending_top_layer_removals);
    }
    out
}
fn full_slot_payload(slot: &super::Slot) -> source::storage::Payload {
    let mut out = source::storage::Payload {
        bytes: size_of::<super::Slot>(),
        units: 0,
    };
    if let Some(node) = &slot.node {
        out.append(full_node_payload(node)).unwrap();
    }
    out
}
fn graph_payload<T>(
    values: &bank::Bank<T>,
    mut describe: impl FnMut(&T) -> source::storage::Payload,
) -> source::storage::Payload {
    let graph = values
        .inspect_graph(|value| {
            let actual = describe(value);
            Ok(bank::Totals {
                bytes: actual.bytes,
                units: actual.units,
                occurrences: 1,
            })
        })
        .unwrap();
    source::storage::Payload {
        bytes: graph.index_charge().bytes + graph.record_totals().bytes,
        units: graph.record_totals().units,
    }
}
fn full_transition_payload(
    value: &crate::view_transition::ViewTransitions,
) -> source::storage::Payload {
    use crate::view_transition::{DocumentTransitions, Record, Transition};
    let mut out = graph_payload(&value.documents, |_| source::storage::Payload {
        bytes: size_of::<crate::DomDocumentViewTransitions>(),
        units: 0,
    });
    out.append(graph_payload(&value.transitions, |transition| {
        let mut own = source::storage::Payload {
            bytes: size_of::<Transition>(),
            units: 0,
        };
        own.append(graph_payload(&transition.records, |record| {
            source::storage::Payload {
                bytes: size_of::<Record>()
                    + size_of::<surgeist_css::CssViewTransitionIdent>()
                    + record.name.retained_buffer_bytes(),
                units: 0,
            }
        }))
        .unwrap();
        let graph = transition.membership.inspect_graph().unwrap();
        own.append(source::storage::Payload {
            bytes: graph.index_charge().bytes + graph.record_totals().bytes,
            units: graph.record_totals().units,
        })
        .unwrap();
        own
    }))
    .unwrap();
    out.append(graph_payload(&value.document_transitions, |document| {
        let mut own = source::storage::Payload {
            bytes: size_of::<DocumentTransitions>(),
            units: 0,
        };
        own.append(graph_payload(&document.ids, |_| source::storage::Payload {
            bytes: size_of::<crate::DomViewTransitionId>(),
            units: 0,
        }))
        .unwrap();
        own
    }))
    .unwrap();
    out
}

/// Enumerate records and their current actual backing capacities. Never read
/// `Bank::totals`, State.nodes/units, or a transaction's touched metadata here.
fn full_slot_totals(state: &State) -> bank::Totals {
    let mut totals = bank::Totals::default();
    for (_, slot) in state.slots.iter() {
        let payload = full_slot_payload(slot);
        add(&mut totals.bytes, payload.bytes);
        add(&mut totals.units, payload.units);
        add(&mut totals.occurrences, 1);
    }
    totals
}

fn full_range_totals(state: &State) -> bank::Totals {
    let mut totals = bank::Totals::default();
    for _ in state.ranges.iter() {
        add(&mut totals.bytes, size_of::<crate::range::Slot<NodeId>>());
        add(&mut totals.occurrences, 1);
    }
    totals
}

fn slot_index_charge(state: &State) -> bank::IndexCharge {
    state
        .slots
        .inspect_graph(|slot| {
            let payload = full_slot_payload(slot);
            Ok(bank::Totals {
                bytes: payload.bytes,
                units: payload.units,
                occurrences: 1,
            })
        })
        .unwrap()
        .index_charge()
}

fn range_index_charge(state: &State) -> bank::IndexCharge {
    state
        .ranges
        .inspect_graph(|_| {
            Ok(bank::Totals {
                bytes: size_of::<crate::range::Slot<NodeId>>(),
                units: 0,
                occurrences: 1,
            })
        })
        .unwrap()
        .index_charge()
}

/// Independent root-only charge. Bank pages/records and nested domains are
/// compared separately; no aggregate State receipt is assumed or invented.
fn full_root_payload(state: &State) -> source::storage::Payload {
    let mut payload = source::storage::Payload::default();
    payload.add_bytes(size_of::<State>()).unwrap();
    sequence(&mut payload, &state.signal_slots);
    sequence(&mut payload, &state.native_tasks);
    sequence(&mut payload, &state.details_intentions);

    payload
}

/// Recompute logical public usage independently of aggregate counters. Kind
/// owners still define their own string semantics; this oracle owns the full
/// traversal and summation, including related producer facts and past names.
fn full_usage(dom: &Dom) -> DomUsage {
    let state = &dom.state;
    let mut usage = DomUsage {
        ranges: 0,
        meta_color_scheme_bytes: 0,
        autofocus_candidates: 0,
        slot_references: state.signal_slots.len(),
        nodes: 0,
        units: 0,
        history_entries: 0,
        history_bytes: 0,
        transitions: 0,
        records: 0,
        pseudos: 0,
    };
    for (_, slot) in state.ranges.iter() {
        if slot.record.is_some() {
            add(&mut usage.ranges, 1);
        }
    }
    for (_, slot) in state.slots.iter() {
        let Some(node) = &slot.node else {
            continue;
        };
        add(&mut usage.nodes, 1);
        let units = match &node.kind {
            DomNodeKind::Document(metadata) => {
                add(
                    &mut usage.meta_color_scheme_bytes,
                    metadata
                        .meta_color_scheme
                        .as_ref()
                        .and_then(Option::as_ref)
                        .map_or(0, |value| value.report.retained_storage().bytes()),
                );
                full_document_units(metadata)
            }
            DomNodeKind::Element(element) => full_element_units(element),
            DomNodeKind::Text(value) | DomNodeKind::Comment(value) => value.len(),
            DomNodeKind::ProcessingInstruction(pi) => pi.units().unwrap(),
            DomNodeKind::DocumentType(doctype) => doctype.units().unwrap(),
            DomNodeKind::DocumentFragment | DomNodeKind::ShadowRoot(_) => 0,
        };
        add(&mut usage.units, units);
        add(
            &mut usage.units,
            node.base.as_ref().map_or(0, crate::base::BaseState::units),
        );
        add(
            &mut usage.units,
            node.hyperlink
                .as_ref()
                .map_or(0, crate::hyperlink::HyperlinkState::units),
        );
        add(
            &mut usage.units,
            node.encoding_url_document
                .as_ref()
                .map_or(0, crate::hyperlink::EncodingUrlDocument::units),
        );
        if let Some(facts) = node
            .base_document
            .as_ref()
            .and_then(|document| document.facts.as_ref())
        {
            add(&mut usage.units, facts.formation_usage().utf16_extent());
        }
        if let Some(names) = &node.form_past_names {
            for name in names.entries.iter() {
                add(&mut usage.units, name.name.len());
            }
        }
        if let Some(canonical::Autofocus::Known { candidates, .. }) = &node.autofocus_document {
            add(&mut usage.autofocus_candidates, candidates.len());
        }
        add(&mut usage.slot_references, node.manual_nodes.len());
        add(&mut usage.slot_references, node.assigned_nodes.len());
    }
    for transition in &state.view_transitions.transitions {
        add(&mut usage.transitions, 1);
        add(&mut usage.records, transition.records.iter().count());
        for record in &transition.records {
            add(
                &mut usage.units,
                record.name.as_str().encode_utf16().count(),
            );
        }
        add(&mut usage.pseudos, 1);
        if transition.setup {
            for record in &transition.records {
                add(
                    &mut usage.pseudos,
                    2 + usize::from(record.branches.has_old())
                        + usize::from(record.branches.new_element().is_some()),
                );
            }
        }
    }
    // Walk the actual retained chunk entries. Cached entry quotas are only
    // the actual side; lengths and the owning header determine expectations.
    dom.history.inspect_entries(|_from, changes, quota| {
        add(&mut usage.history_entries, 1);
        let bytes = size_of::<super::HistoryEntry>()
            .checked_add(changes.len().checked_mul(size_of::<DomChange>()).unwrap())
            .unwrap();
        assert_eq!(quota, bytes);
        add(&mut usage.history_bytes, bytes);
    });
    usage
}

fn ancestor_path(state: &State, owner: DomOwnerId, mut id: NodeId) -> Vec<NodeId> {
    let mut path = Vec::new();
    let mut seen = HashSet::new();
    loop {
        assert!(seen.insert(id), "canonical ordinary parent cycle");
        path.push(id);
        match state.node(owner, id).unwrap().parent {
            Some(parent) => id = parent,
            None => break,
        }
    }
    path.reverse();
    path
}

fn boundary_length(state: &State, owner: DomOwnerId, id: NodeId) -> usize {
    let node = state.node(owner, id).unwrap();
    match &node.kind {
        DomNodeKind::Text(value) | DomNodeKind::Comment(value) => value.len(),
        DomNodeKind::ProcessingInstruction(pi) => pi.data().len(),
        _ => node.children.len(),
    }
}

/// An independent literal ancestor/LCA ordering algorithm. It uses the actual
/// captured tree, not Range dependency caches or production Range cursors.
fn ordered_boundaries(
    state: &State,
    owner: DomOwnerId,
    start: crate::range::Boundary<NodeId>,
    end: crate::range::Boundary<NodeId>,
) -> bool {
    if start.container == end.container {
        return start.offset <= end.offset;
    }
    let left = ancestor_path(state, owner, start.container);
    let right = ancestor_path(state, owner, end.container);
    if left.first() != right.first() {
        return false;
    }
    let common = left.iter().zip(&right).take_while(|(a, b)| a == b).count();
    if common == left.len() {
        let child_index = state
            .node(owner, start.container)
            .unwrap()
            .children
            .iter()
            .position(|id| *id == right[common])
            .unwrap();
        start.offset <= child_index
    } else if common == right.len() {
        let child_index = state
            .node(owner, end.container)
            .unwrap()
            .children
            .iter()
            .position(|id| *id == left[common])
            .unwrap();
        child_index < end.offset
    } else {
        let parent = state.node(owner, left[common - 1]).unwrap();
        let left_index = parent
            .children
            .iter()
            .position(|id| *id == left[common])
            .unwrap();
        let right_index = parent
            .children
            .iter()
            .position(|id| *id == right[common])
            .unwrap();
        left_index < right_index
    }
}

fn canonical_key(key: super::Key) -> NodeId {
    match key {
        super::Key::Existing(id) => id,
        super::Key::Created(_) => panic!("canonical derived index contains a private ticket"),
    }
}

fn assert_range_dependencies(owner: DomOwnerId, state: &State) {
    let mut containers: HashMap<NodeId, HashSet<crate::DomRangeId>> = HashMap::new();
    let mut associations: HashMap<crate::DomRangeId, HashSet<NodeId>> = HashMap::new();
    let mut materialized = Vec::new();
    let mut lowest = None;
    for (index, slot) in state.ranges.iter() {
        materialized.push(index);
        if slot.record.is_none() && !slot.retired && lowest.is_none() {
            lowest = Some(index);
        }
        if let Some(record) = slot.record {
            let id = crate::DomRangeId {
                owner,
                slot: index,
                generation: slot.generation,
            };
            containers
                .entry(record.start.container)
                .or_default()
                .insert(id);
            containers
                .entry(record.end.container)
                .or_default()
                .insert(id);
        }
    }
    for (index, slot) in state.slots.iter() {
        if let Some(node) = &slot.node
            && let DomActiveRangeState::Known(Some(id)) = node.active_range
        {
            let document = NodeId {
                owner,
                slot: index,
                generation: slot.generation,
            };
            assert!(matches!(node.kind, DomNodeKind::Document(_)));
            associations.entry(id).or_default().insert(document);
        }
    }
    // Cached summaries are only the actual side. The expected graph walks
    // concrete page capacities and record allocations without reading them.
    let mut actual_totals = bank::Totals::default();
    let mut actual_index = bank::IndexCharge::default();
    for (totals, index) in state.range_dependencies.inspect_summaries() {
        add(&mut actual_totals.bytes, totals.bytes);
        add(&mut actual_totals.units, totals.units);
        add(&mut actual_totals.occurrences, totals.occurrences);
        add(&mut actual_index.bytes, index.bytes);
        add(&mut actual_index.pages, index.pages);
    }
    let expected_graph = state.range_dependencies.inspect_graph().unwrap();
    assert_eq!(actual_totals, expected_graph.record_totals());
    assert_eq!(actual_index, expected_graph.index_charge());
    let actual = state.range_dependencies.inspect_memberships();
    let mut actual_containers = HashMap::new();
    for (key, ids) in actual.containers {
        let length = ids.len();
        let ids: HashSet<_> = ids.into_iter().collect();
        assert_eq!(length, ids.len(), "duplicate endpoint dependency");
        assert!(actual_containers.insert(canonical_key(key), ids).is_none());
    }
    let mut actual_associations = HashMap::new();
    for (id, keys) in actual.associations {
        let length = keys.len();
        let keys: HashSet<_> = keys.into_iter().map(canonical_key).collect();
        assert_eq!(length, keys.len(), "duplicate active Range association");
        assert!(actual_associations.insert(id, keys).is_none());
    }
    assert_eq!(actual_containers, containers);
    assert_eq!(actual_associations, associations);
    assert_eq!(
        state.range_dependencies.inspect_availability(),
        (materialized, lowest)
    );
    assert_eq!(state.ranges.lowest_vacant(), lowest);
}

/// This inventory is independent of the production reference visitor: it
/// enumerates immutable canonical records, including repeated occurrences.
fn assert_inbound(owner: DomOwnerId, state: &State) {
    use crate::reflected::ReflectedTarget;
    let mut expected = HashMap::new();
    for (index, slot) in state.slots.iter() {
        let Some(node) = &slot.node else { continue };
        let source = NodeId {
            owner,
            slot: index,
            generation: slot.generation,
        };
        let mut reference = |target| {
            *expected.entry((target, source)).or_insert(0usize) += 1;
        };
        reference(node.document.node());
        for target in [
            node.parent,
            node.shadow_root,
            node.host,
            node.template_contents,
            node.template_patch.and_then(|patch| patch.target),
            node.template_patch.and_then(|patch| patch.start),
            node.template_patch.and_then(|patch| patch.end),
            node.template_document
                .and_then(crate::template::TemplateDocument::inert),
            node.manual_slot,
            node.assigned_slot,
            node.reflected_targets
                .and_then(|values| values.get(ReflectedTarget::CommandFor))
                .flatten(),
            node.reflected_targets
                .and_then(|values| values.get(ReflectedTarget::PopoverTarget))
                .flatten(),
            node.control_owner,
            node.option_select,
            node.script
                .as_ref()
                .and_then(|script| script.parser_document),
            node.script
                .as_ref()
                .and_then(|script| script.preparation_document),
            node.style_parser_document,
            node.base
                .as_ref()
                .and_then(|base| base.frozen.as_ref())
                .map(|value| value.document),
            node.base_document
                .as_ref()
                .and_then(|document| document.first),
            node.link.as_ref().and_then(|link| link.parser_document),
            node.image.as_ref().map(|image| image.dimension_source),
            node.hyperlink
                .as_ref()
                .and_then(|link| link.parsed.as_ref())
                .map(|value| value.document),
            match &node.kind {
                DomNodeKind::Element(element) => element
                    .common_state()
                    .html()
                    .and_then(|html| html.previously_focused_element().copied()),
                _ => None,
            },
        ]
        .into_iter()
        .flatten()
        {
            reference(target);
        }
        for values in [&node.children, &node.manual_nodes, &node.assigned_nodes] {
            for target in values.iter().copied() {
                reference(target);
            }
        }
        if let Some(names) = &node.form_past_names {
            for value in names.entries.iter() {
                reference(value.target);
            }
        }
        if let Some(document) = &node.base_document {
            for target in document.render_blocking.iter().copied() {
                reference(target);
            }
        }
        if let Some(canonical::Autofocus::Known { candidates, .. }) = &node.autofocus_document {
            for target in candidates.iter().copied() {
                reference(target);
            }
        }
        if let Some(canonical::Interactive::Known {
            open_dialogs,
            top_layer,
            pending_top_layer_removals,
        }) = &node.interactive_document
        {
            for values in [open_dialogs, top_layer, pending_top_layer_removals] {
                for target in values.iter().copied() {
                    reference(target);
                }
            }
        }
    }
    let mut actual = HashMap::new();
    for (target, source, count) in state.inbound.inspect_references() {
        assert_ne!(count, 0);
        assert!(
            actual
                .insert((canonical_key(target), canonical_key(source)), count)
                .is_none()
        );
    }
    assert_eq!(actual, expected);
    let graph = state.inbound.inspect_graph().unwrap();
    assert_eq!(
        state.inbound.retained_usage(),
        custody::Usage::new(
            graph.index_charge().bytes + graph.record_totals().bytes,
            graph.record_totals().units
        )
    );
}

fn assert_queue_targets(owner: DomOwnerId, state: &State) {
    fn counted(values: Vec<(NodeId, usize)>) -> HashMap<NodeId, usize> {
        let mut map = HashMap::new();
        for (id, count) in values {
            assert_ne!(count, 0);
            assert!(map.insert(id, count).is_none());
        }
        map
    }
    for (index, targets) in [
        &state.signal_targets,
        &state.native_targets,
        &state.details_targets,
    ]
    .into_iter()
    .enumerate()
    {
        let mut expected_targets = HashMap::new();
        let mut expected_documents = HashMap::new();
        let mut expected_associations = HashMap::new();
        let mut add_target = |target, document: Option<NodeId>| {
            *expected_targets.entry(target).or_insert(0usize) += 1;
            if let Some(document) = document {
                *expected_documents.entry(document).or_insert(0usize) += 1;
            }
        };
        match index {
            0 => {
                for id in state.signal_slots.iter().copied() {
                    add_target(id, Some(state.node(owner, id).unwrap().document.node()));
                }
            }
            1 => {
                for task in state.native_tasks.iter() {
                    if let super::select::Task::Update { target, .. } = task {
                        add_target(
                            *target,
                            Some(state.node(owner, *target).unwrap().document.node()),
                        );
                    }
                }
            }
            2 => {
                for intention in state.details_intentions.iter() {
                    let original_document = intention
                        .queue_association()
                        .map(|association| association.document().node());
                    add_target(intention.target(), original_document);
                    if let Some(document) = original_document {
                        // The queued row records its original association;
                        // later adoption does not substitute the current owner.
                        *expected_associations
                            .entry((intention.target(), document))
                            .or_insert(0usize) += 1;
                    }
                }
            }
            _ => unreachable!(),
        }
        let (actual_targets, actual_documents) = targets.inspect_counts();
        assert_eq!(counted(actual_targets), expected_targets);
        assert_eq!(counted(actual_documents), expected_documents);
        let mut actual_associations = HashMap::new();
        for (target, document, count) in targets.inspect_associations() {
            assert_ne!(count, 0);
            assert!(
                actual_associations
                    .insert((canonical_key(target), canonical_key(document)), count)
                    .is_none()
            );
        }
        assert_eq!(actual_associations, expected_associations);
        let graph = targets.inspect_graph().unwrap();
        assert_eq!(targets.index_charge(), graph.index_charge());
        assert_eq!(targets.totals(), graph.record_totals());
    }
}

#[derive(Debug, Eq, Hash, PartialEq)]
enum ControlGroup {
    Document(NodeId),
    Tree(NodeId),
    Radio(NodeId, Option<NodeId>, Vec<u16>),
}

fn assert_control_index(owner: DomOwnerId, state: &State) {
    use transaction::controls::ControlGroupKind;
    // The supported native listed families are concrete initialized records;
    // tag text alone does not establish native association capability.
    let mut facts = HashMap::new();
    let mut groups: HashMap<ControlGroup, HashSet<NodeId>> = HashMap::new();
    for (index, slot) in state.slots.iter() {
        let Some(node) = &slot.node else { continue };
        let DomNodeKind::Element(element) = &node.kind else {
            continue;
        };
        let listed = element.button_state().is_some()
            || element.fieldset_state().is_some()
            || element.input_state().is_some()
            || element.output_state().is_some()
            || element.select_state().is_some()
            || element.textarea_state().is_some();
        if !listed {
            continue;
        }
        let id = NodeId {
            owner,
            slot: index,
            generation: slot.generation,
        };
        let document = node.document.node();
        let root = *ancestor_path(state, owner, id).first().unwrap();
        let mut form_present = false;
        let mut name = None;
        for attribute in element.attributes().iter() {
            if attribute.name().namespace().uri().is_some() {
                continue;
            }
            match attribute.name().local_name().code_units() {
                [102, 111, 114, 109] => form_present = true,
                [110, 97, 109, 101] if element.input_state().is_some() => {
                    name = Some(attribute.value().clone())
                }
                _ => {}
            }
        }
        let radio = element
            .input_state()
            .is_some_and(|input| input.input_type() == surgeist_html::HtmlInputType::Radio);
        let fact = (
            document,
            root,
            node.control_owner,
            form_present,
            radio,
            name.clone(),
        );
        assert!(facts.insert(id, fact).is_none());
        if form_present {
            groups
                .entry(ControlGroup::Document(document))
                .or_default()
                .insert(id);
            // Ordinary-tree membership is independent of a participant's
            // node-document association during real staged adoption phases.
            groups
                .entry(ControlGroup::Tree(root))
                .or_default()
                .insert(id);
        }
        if radio && let Some(name) = name.filter(|name| !name.is_empty()) {
            groups
                .entry(ControlGroup::Radio(
                    root,
                    node.control_owner,
                    name.code_units().to_vec(),
                ))
                .or_default()
                .insert(id);
        }
    }
    let mut actual_facts = HashMap::new();
    for fact in state.controls.inspect_facts() {
        let id = canonical_key(fact.key);
        assert!(
            actual_facts
                .insert(
                    id,
                    (
                        canonical_key(fact.document),
                        canonical_key(fact.root),
                        fact.owner.map(canonical_key),
                        fact.form_present,
                        fact.radio,
                        fact.name
                    )
                )
                .is_none()
        );
    }
    assert_eq!(actual_facts, facts);
    let mut actual_groups = HashMap::new();
    for group in state.controls.inspect_groups() {
        let representative = canonical_key(group.representative);
        let (document, root, owner, form_present, radio, name) = facts
            .get(&representative)
            .expect("group representative is an actual listed control");
        let key = match group.kind {
            ControlGroupKind::Document => {
                assert!(*form_present);
                ControlGroup::Document(*document)
            }
            ControlGroupKind::Tree => {
                assert!(*form_present);
                ControlGroup::Tree(*root)
            }
            ControlGroupKind::Radio => {
                assert!(*radio);
                let name = name
                    .as_ref()
                    .filter(|name| !name.is_empty())
                    .expect("radio group has an exact nonempty name");
                ControlGroup::Radio(*root, *owner, name.code_units().to_vec())
            }
        };
        let length = group.members.len();
        let members: HashSet<_> = group.members.into_iter().map(canonical_key).collect();
        assert_eq!(members.len(), length, "unique control group members");
        assert!(actual_groups.insert(key, members).is_none());
    }
    assert_eq!(actual_groups, groups);
    let mut totals = bank::Totals::default();
    let mut index = bank::IndexCharge::default();
    for (cached, charge) in state.controls.inspect_summaries() {
        add(&mut totals.bytes, cached.bytes);
        add(&mut totals.units, cached.units);
        add(&mut totals.occurrences, cached.occurrences);
        add(&mut index.bytes, charge.bytes);
        add(&mut index.pages, charge.pages);
    }
    let physical = state.controls.inspect_graph().unwrap();
    assert_eq!(totals, physical.record_totals());
    assert_eq!(index, physical.index_charge());
}

/// Expected positions and order come only from the published child arrays.
/// Neither the production Slot comparator nor its participant iterator is used.
fn assert_slot_participants(owner: DomOwnerId, state: &State) {
    let mut nodes = HashMap::new();
    for (index, slot) in state.slots.iter() {
        if let Some(node) = &slot.node {
            let id = NodeId {
                owner,
                slot: index,
                generation: slot.generation,
            };
            assert!(nodes.insert(id, node).is_none());
        }
    }
    let mut positions = HashMap::new();
    for (&parent, node) in &nodes {
        for (ordinal, &child) in node.children.iter().enumerate() {
            assert_eq!(nodes[&child].parent, Some(parent));
            assert!(positions.insert(child, (parent, ordinal)).is_none());
        }
    }
    let mut actual_positions = HashMap::new();
    for position in state.slot_index.inspect_positions() {
        assert!(
            actual_positions
                .insert(
                    canonical_key(position.key),
                    (canonical_key(position.parent), position.ordinal)
                )
                .is_none()
        );
    }
    assert_eq!(actual_positions, positions);

    let mut expected = HashMap::new();
    let mut ordinary_nodes = HashSet::new();
    for &root in nodes.keys().filter(|key| !positions.contains_key(*key)) {
        let mut slots = Vec::new();
        let mut pending = vec![root];
        while let Some(id) = pending.pop() {
            assert!(
                ordinary_nodes.insert(id),
                "ordinary child graph is a forest"
            );
            let node = nodes[&id];
            if matches!(&node.kind, DomNodeKind::Element(element) if element.is_slot()) {
                slots.push(id);
            }
            // Ordinary children alone define this walk. A ShadowRoot and
            // Template contents start their own groups through the root loop.
            pending.extend(node.children.iter().rev().copied());
        }
        if !slots.is_empty() {
            assert!(expected.insert(root, slots).is_none());
        }
    }
    assert_eq!(ordinary_nodes.len(), nodes.len());

    use transaction::slot_index::SlotMemberInspection;
    let mut members = HashMap::new();
    for member in state.slot_index.inspect_members() {
        assert!(members.insert(canonical_key(member.key), member).is_none());
    }
    fn walk_avl(
        id: NodeId,
        root: NodeId,
        parent: Option<NodeId>,
        members: &HashMap<NodeId, SlotMemberInspection>,
        visited: &mut HashSet<NodeId>,
        order: &mut Vec<NodeId>,
    ) -> usize {
        assert!(visited.insert(id), "each Slot has one reachable AVL row");
        let member = &members[&id];
        assert_eq!(canonical_key(member.key), id);
        assert_eq!(canonical_key(member.root), root);
        assert_eq!(member.order_parent.map(canonical_key), parent);
        let left = member.left.map(canonical_key).map_or(0, |child| {
            walk_avl(child, root, Some(id), members, visited, order)
        });
        order.push(id);
        let right = member.right.map(canonical_key).map_or(0, |child| {
            walk_avl(child, root, Some(id), members, visited, order)
        });
        assert!(left.abs_diff(right) <= 1, "Slot AVL remains balanced");
        let height = 1 + left.max(right);
        assert_eq!(usize::from(member.height), height);
        height
    }
    let mut visited = HashSet::new();
    let mut actual = HashMap::new();
    for (root, member) in state.slot_index.inspect_roots() {
        let root = canonical_key(root);
        assert!(nodes.contains_key(&root));
        assert!(
            !positions.contains_key(&root),
            "header names an ordinary root"
        );
        let mut order = Vec::new();
        walk_avl(
            canonical_key(member),
            root,
            None,
            &members,
            &mut visited,
            &mut order,
        );
        assert!(actual.insert(root, order).is_none());
    }
    assert_eq!(actual, expected);
    assert_eq!(visited.len(), members.len(), "no orphan Slot AVL rows");

    let mut totals = bank::Totals::default();
    let mut index = bank::IndexCharge::default();
    for (cached, charge) in state.slot_index.inspect_summaries() {
        add(&mut totals.bytes, cached.bytes);
        add(&mut totals.units, cached.units);
        add(&mut totals.occurrences, cached.occurrences);
        add(&mut index.bytes, charge.bytes);
        add(&mut index.pages, charge.pages);
    }
    // The producer inspector enumerates actual Page capacities and record
    // bodies, independently of these cached summary actuals.
    let physical = state.slot_index.inspect_graph().unwrap();
    assert_eq!(totals, physical.record_totals());
    assert_eq!(index, physical.index_charge());
}

/// Reconstruct exact names from actual null-namespace attributes. Empty and
/// missing names share the default group; physical field occurrences are
/// independently checked by the raw graph inspector.
fn assert_slot_names(owner: DomOwnerId, state: &State) {
    fn name(node: &super::Node, attribute: &[u16]) -> Option<DocumentString> {
        let DomNodeKind::Element(element) = &node.kind else {
            return None;
        };
        element
            .attributes()
            .iter()
            .find(|value| {
                value.name().namespace().uri().is_none()
                    && value.name().local_name().code_units() == attribute
            })
            .map(|value| value.value().clone())
            .filter(|value| !value.is_empty())
    }
    let order = |id| {
        ancestor_path(state, owner, id)
            .windows(2)
            .map(|edge| {
                state
                    .node(owner, edge[0])
                    .unwrap()
                    .children
                    .iter()
                    .position(|&child| child == edge[1])
                    .unwrap()
            })
            .collect::<Vec<_>>()
    };
    let mut facts = HashMap::new();
    let mut groups = HashMap::<(usize, NodeId, Option<DocumentString>), Vec<NodeId>>::new();
    for (index, slot) in state.slots.iter() {
        let Some(node) = &slot.node else { continue };
        let id = NodeId {
            owner,
            slot: index,
            generation: slot.generation,
        };
        let slot = if matches!(&node.kind,DomNodeKind::Element(element) if element.is_slot()) {
            let root = *ancestor_path(state, owner, id).first().unwrap();
            Some((root, name(node, &[110, 97, 109, 101])))
        } else {
            None
        };
        let light = if matches!(node.kind, DomNodeKind::Element(_) | DomNodeKind::Text(_)) {
            node.parent.and_then(|parent|{
                let host=state.node(owner,parent).unwrap();
                let shadow=host.shadow_root.and_then(|root|state.node(owner,root).ok())?;
                (matches!(&shadow.kind,DomNodeKind::ShadowRoot(value) if value.slot_assignment()==crate::HtmlSlotAssignment::Named)&&shadow.host==Some(parent))
                    .then(||(parent,name(node,&[115,108,111,116])))
            })
        } else {
            None
        };
        for (kind, value) in [(0, &slot), (1, &light)] {
            if let Some((scope, value)) = value {
                groups
                    .entry((kind, *scope, value.clone()))
                    .or_default()
                    .push(id);
            }
        }
        if slot.is_some() || light.is_some() {
            assert!(facts.insert(id, (slot, light)).is_none());
        }
    }
    for members in groups.values_mut() {
        members.sort_by_key(|&id| order(id));
    }
    let mut actual_facts = HashMap::new();
    for fact in state.slot_index.named.inspect_facts() {
        let map =
            |name: transaction::slot_names::NameInspection| (canonical_key(name.scope), name.value);
        assert!(
            actual_facts
                .insert(
                    canonical_key(fact.key),
                    (fact.slot.map(map), fact.light.map(map))
                )
                .is_none()
        );
    }
    assert_eq!(actual_facts, facts);
    let mut actual_groups = HashMap::new();
    for group in state.slot_index.named.inspect_groups().unwrap() {
        assert!(
            actual_groups
                .insert(
                    (group.kind, canonical_key(group.scope), group.value),
                    group
                        .members
                        .into_iter()
                        .map(canonical_key)
                        .collect::<Vec<_>>()
                )
                .is_none()
        );
    }
    assert_eq!(actual_groups, groups);
}

/// Canonical shadow edges precede ordinary children. Template ownership does
/// not join an ordinary tree. Null child-navigable content is still a real
/// container participant, independent of hosted graph-parent agreement.
fn assert_hosted_containers(owner: DomOwnerId, state: &State) {
    let mut roots = Vec::new();
    for (index, slot) in state.slots.iter() {
        let Some(node) = &slot.node else { continue };
        if node.parent.is_some() {
            continue;
        }
        if matches!(node.kind, DomNodeKind::ShadowRoot(_))
            && node.host.is_some_and(|host| {
                state.node(owner, host).unwrap().shadow_root
                    == Some(NodeId {
                        owner,
                        slot: index,
                        generation: slot.generation,
                    })
            })
        {
            continue;
        }
        roots.push(NodeId {
            owner,
            slot: index,
            generation: slot.generation,
        });
    }
    let mut expected = HashMap::<NodeId, Vec<NodeId>>::new();
    let mut visited = HashSet::new();
    for root in roots {
        let mut pending = vec![root];
        while let Some(id) = pending.pop() {
            assert!(
                visited.insert(id),
                "canonical shadow-inclusive closure visited once"
            );
            let node = state.node(owner, id).unwrap();
            if matches!(&node.kind,DomNodeKind::Element(element) if element.child_navigable().is_some())
            {
                expected.entry(root).or_default().push(id);
            }
            pending.extend(node.children.iter().rev().copied());
            if let Some(shadow) = node.shadow_root {
                pending.push(shadow);
            }
        }
    }
    assert_eq!(
        visited.len(),
        state
            .slots
            .iter()
            .filter(|(_, slot)| slot.node.is_some())
            .count()
    );
    let mut actual_members = HashSet::new();
    for (root, member) in state.container_index.inspect_members() {
        assert!(actual_members.insert((canonical_key(root), canonical_key(member))));
    }
    let expected_members = expected
        .iter()
        .flat_map(|(&root, members)| members.iter().map(move |&member| (root, member)))
        .collect::<HashSet<_>>();
    assert_eq!(actual_members, expected_members);
    let order = transaction::ordered_members::OrderView::state(state, owner);
    for (root, members) in expected {
        let root = super::Key::Existing(root);
        let mut next = state.container_index.first(root, &mut || Ok(())).unwrap();
        for member in members {
            assert_eq!(next, Some(super::Key::Existing(member)));
            next = state
                .container_index
                .after(root, super::Key::Existing(member), order, &mut || Ok(()))
                .unwrap();
        }
        assert_eq!(
            next, None,
            "no additional container after independently enumerated closure"
        );
    }
    let graph = state.container_index.inspect_graph().unwrap();
    assert_eq!(
        state.container_index.retained_usage().unwrap(),
        custody::Usage::new(
            graph.index_charge().bytes + graph.record_totals().bytes,
            graph.record_totals().units
        )
    );
}

/// Family expectations use raw ordinary ancestry and actual names/native bits.
/// Option's cached nearest Select and the production membership cursors are not
/// inputs to this recomputation.
fn assert_family_discovery(owner: DomOwnerId, state: &State) {
    use surgeist_html::KnownHtmlElement as Tag;
    use transaction::family_discovery::FactKindInspection as Kind;
    let tag = |id| match &state.node(owner, id).unwrap().kind {
        DomNodeKind::Element(element) => Tag::from_name(element.name()),
        _ => None,
    };
    let order = |id| {
        ancestor_path(state, owner, id)
            .windows(2)
            .map(|edge| {
                state
                    .node(owner, edge[0])
                    .unwrap()
                    .children
                    .iter()
                    .position(|&child| child == edge[1])
                    .unwrap()
            })
            .collect::<Vec<_>>()
    };
    let mut facts = HashMap::new();
    let mut groups = HashMap::<(usize, NodeId, Option<DocumentString>), Vec<NodeId>>::new();
    for (index, slot) in state.slots.iter() {
        let Some(node) = &slot.node else { continue };
        let DomNodeKind::Element(element) = &node.kind else {
            continue;
        };
        let id = NodeId {
            owner,
            slot: index,
            generation: slot.generation,
        };
        let mut root = id;
        let mut name = None;
        let mut select = None;
        let kind = match Tag::from_name(element.name()) {
            Some(Tag::Details) => {
                name = element
                    .attributes()
                    .iter()
                    .find(|attribute| {
                        attribute.name().namespace().uri().is_none()
                            && attribute.name().local_name().code_units() == [110, 97, 109, 101]
                    })
                    .map(|attribute| attribute.value().clone())
                    .filter(|value| !value.is_empty());
                if name.is_none() {
                    continue;
                }
                root = *ancestor_path(state, owner, id).first().unwrap();
                groups
                    .entry((Kind::Details as usize, root, name.clone()))
                    .or_default()
                    .push(id);
                Kind::Details
            }
            Some(Tag::Option) => {
                let mut parent = node.parent;
                let mut optgroup = false;
                while let Some(ancestor) = parent {
                    match tag(ancestor) {
                        Some(Tag::Datalist | Tag::Hr | Tag::Option) => break,
                        Some(Tag::Optgroup) if optgroup => break,
                        Some(Tag::Optgroup) => optgroup = true,
                        Some(Tag::Select) => {
                            select = Some(ancestor);
                            break;
                        }
                        _ => (),
                    }
                    parent = state.node(owner, ancestor).unwrap().parent;
                }
                if let Some(select) = select {
                    groups
                        .entry((Kind::Option as usize, select, None))
                        .or_default()
                        .push(id);
                }
                Kind::Option
            }
            Some(Tag::Selectedcontent) if element.selectedcontent_disabled() == Some(false) => {
                let mut parent = node.parent;
                while let Some(ancestor) = parent {
                    if tag(ancestor) == Some(Tag::Select) {
                        groups
                            .entry((Kind::Target as usize, ancestor, None))
                            .or_default()
                            .push(id);
                    }
                    parent = state.node(owner, ancestor).unwrap().parent;
                }
                Kind::Target
            }
            _ => continue,
        };
        assert!(
            facts
                .insert(id, (kind as usize, root, name, select))
                .is_none()
        );
    }
    for members in groups.values_mut() {
        members.sort_by_key(|&id| order(id));
    }
    let mut actual_facts = HashMap::new();
    for fact in state.family_index.inspect_facts() {
        assert!(
            actual_facts
                .insert(
                    canonical_key(fact.key),
                    (
                        fact.kind as usize,
                        canonical_key(fact.root),
                        fact.name,
                        fact.select.map(canonical_key)
                    )
                )
                .is_none()
        );
    }
    assert_eq!(actual_facts, facts);
    let mut actual_groups = HashMap::new();
    for group in state.family_index.inspect_groups().unwrap() {
        assert!(
            actual_groups
                .insert(
                    (group.kind as usize, canonical_key(group.owner), group.name),
                    group
                        .members
                        .into_iter()
                        .map(canonical_key)
                        .collect::<Vec<_>>()
                )
                .is_none()
        );
    }
    assert_eq!(actual_groups, groups);
    let graph = state.family_index.inspect_graph().unwrap();
    assert_eq!(
        state.family_index.heap_usage().unwrap(),
        custody::Usage::new(
            graph.index_charge().bytes + graph.record_totals().bytes,
            graph.record_totals().units
        )
    );
}

/// Recount each role from the actual canonical relation owning that fact.
/// In particular Raw top-layer ownership can differ from node-document after
/// adoption; multiplicity and role identity are not collapsed together.
fn assert_role_facts(owner: DomOwnerId, state: &State) {
    use transaction::role_facts::Role;
    let mut expected = HashMap::<(usize, NodeId, NodeId), usize>::new();
    let mut occurrence = |role: Role, source: NodeId, target: NodeId| {
        *expected.entry((role as usize, source, target)).or_default() += 1;
    };
    for (index, slot) in state.slots.iter() {
        let Some(node) = &slot.node else { continue };
        let source = NodeId {
            owner,
            slot: index,
            generation: slot.generation,
        };
        if let Some(base) = &node.base_document {
            for &target in base.render_blocking.iter() {
                occurrence(Role::RenderBlocking, source, target);
            }
        }
        if let Some(canonical::Autofocus::Known { candidates, .. }) = &node.autofocus_document {
            for &target in candidates.iter() {
                occurrence(Role::AutofocusCandidates, source, target);
            }
        }
        if let Some(canonical::Interactive::Known {
            open_dialogs,
            top_layer,
            pending_top_layer_removals,
        }) = &node.interactive_document
        {
            for (role, values) in [
                (Role::OpenDialogs, open_dialogs),
                (Role::TopLayer, top_layer),
                (Role::PendingTopLayerRemovals, pending_top_layer_removals),
            ] {
                for &target in values.iter() {
                    occurrence(role, source, target);
                }
            }
            for &target in top_layer.iter() {
                if let DomNodeKind::Element(element) = &state.node(owner, target).unwrap().kind {
                    if element
                        .common_state()
                        .html()
                        .is_some_and(|html| !html.popover_is_hidden())
                    {
                        occurrence(Role::ShowingPopover, source, target);
                    }
                    if element.fullscreen_flag() {
                        occurrence(Role::FullscreenFlagged, source, target);
                    }
                }
            }
        }
        for &target in node.children.iter() {
            let selected = state.node(owner, target).unwrap();
            if let DomNodeKind::Element(element) = &selected.kind {
                if matches!(node.kind, DomNodeKind::Document(_)) {
                    occurrence(Role::DocumentElement, source, target);
                }
                let name = element.name();
                if name.namespace() == &crate::Namespace::html()
                    && matches!(
                        name.local_name().code_units(),
                        [98, 111, 100, 121] | [102, 114, 97, 109, 101, 115, 101, 116]
                    )
                {
                    occurrence(Role::BodyChild, source, target);
                }
            }
        }
    }
    let mut actual = HashMap::new();
    for (role, source, target, count) in state.role_facts.inspect_relations() {
        assert_ne!(count, 0);
        assert!(
            actual
                .insert(
                    (role as usize, canonical_key(source), canonical_key(target)),
                    count
                )
                .is_none()
        );
    }
    assert_eq!(actual, expected);
    let graph = state.role_facts.inspect_graph().unwrap();
    assert_eq!(
        state.role_facts.retained_usage(),
        custody::Usage::new(
            graph.index_charge().bytes + graph.record_totals().bytes,
            graph.record_totals().units
        )
    );
}

fn assert_transition_indexes(owner: DomOwnerId, state: &State) {
    let value = &state.view_transitions;
    let mut expected: HashMap<DocumentId, HashSet<crate::DomViewTransitionId>> = HashMap::new();
    let mut records = 0usize;
    let mut pseudos = 0usize;
    for (index, transition) in value.transitions.iter() {
        assert_eq!(transition.id.owner, owner);
        assert_eq!(usize::try_from(transition.id.generation).unwrap(), index);
        expected
            .entry(transition.document)
            .or_default()
            .insert(transition.id);
        let mut units = 0;
        let mut own_pseudos = 1;
        for (record_index, record) in transition.records.iter() {
            assert_eq!(record.id.transition, transition.id);
            assert_eq!(usize::try_from(record.id.generation).unwrap(), record_index);
            add(&mut units, record.name.as_str().encode_utf16().count());
            add(&mut records, 1);
            if transition.setup {
                add(
                    &mut own_pseudos,
                    2 + usize::from(record.branches.has_old())
                        + usize::from(record.branches.new_element().is_some()),
                );
            }
        }
        // Expected inverse coverage comes only from actual canonical records.
        // Raw leaves must be published full identities; ticket numbers belong
        // exclusively to a particular private operation.
        use super::view_transition::RecordKey;
        let (name_leaves, element_leaves) = transition.membership.inspect_entries();
        let published = |key| match key {
            RecordKey::Existing(id) => id,
            RecordKey::Created(_) => panic!("private record ticket in canonical inverse"),
        };
        let name_count = name_leaves.len();
        let element_count = element_leaves.len();
        let actual_names = name_leaves
            .into_iter()
            .map(published)
            .collect::<HashSet<_>>();
        let actual_elements = element_leaves
            .into_iter()
            .map(published)
            .collect::<HashSet<_>>();
        assert_eq!(
            actual_names.len(),
            name_count,
            "no duplicate name inverse leaves"
        );
        assert_eq!(
            actual_elements.len(),
            element_count,
            "no duplicate element inverse leaves"
        );
        let mut names = HashSet::new();
        let mut elements = HashSet::new();
        let mut expected_names = HashSet::new();
        let mut expected_elements = HashSet::new();
        let record = |key| {
            let id = published(key);
            let selected = transition
                .records
                .get(usize::try_from(id.generation).unwrap())
                .unwrap();
            assert_eq!(selected.id, id);
            selected
        };
        for (_, selected) in transition.records.iter() {
            assert!(
                names.insert(selected.name.as_str()),
                "exact CSS name is unique within transition"
            );
            expected_names.insert(selected.id);
            assert_eq!(
                transition
                    .membership
                    .name_owner(
                        &selected.name,
                        |key| Ok(record(key).name.as_ref()),
                        &mut || Ok(())
                    )
                    .unwrap(),
                Some(RecordKey::Existing(selected.id))
            );
            if let Some(node) = selected.branches.new_element() {
                assert!(
                    elements.insert(node),
                    "full element identity has one claimant"
                );
                expected_elements.insert(selected.id);
                assert_eq!(
                    transition
                        .membership
                        .element_owner(
                            node,
                            |key| Ok(record(key).branches.new_element().unwrap()),
                            &mut || Ok(())
                        )
                        .unwrap(),
                    Some(RecordKey::Existing(selected.id))
                );
            }
        }
        assert_eq!(actual_names, expected_names);
        assert_eq!(actual_elements, expected_elements);
        assert_eq!(transition.name_units, units);
        assert_eq!(transition.pseudos, own_pseudos);
        assert_eq!(
            transition.records.len(),
            usize::try_from(transition.next_record).unwrap()
        );
        add(&mut pseudos, own_pseudos);
    }
    let mut actual = HashMap::new();
    for (index, document) in value.document_transitions.iter() {
        assert_eq!(index, document.document.node().slot);
        let mut ids = HashSet::new();
        for (index, id) in document.ids.iter() {
            assert_eq!(index, usize::try_from(id.generation).unwrap());
            assert!(ids.insert(*id));
        }
        if !ids.is_empty() {
            assert!(actual.insert(document.document, ids).is_none());
        }
    }
    assert_eq!(actual, expected);
    let mut ordinals = HashSet::new();
    let mut next = 0;
    for (index, document) in value.documents.iter() {
        assert_eq!(index, document.document.node().slot);
        assert!(ordinals.insert(document.init_ordinal));
        next = next.max(document.init_ordinal.checked_add(1).unwrap());
        if let Some(active) = document.active {
            assert_eq!(
                value
                    .transitions
                    .get(usize::try_from(active.generation).unwrap())
                    .unwrap()
                    .document,
                document.document
            );
        }
    }
    assert_eq!(value.next_document, next);
    assert_eq!(
        value.counts(),
        (value.transitions.iter().count(), records, pseudos)
    );
    let payload = full_transition_payload(value);
    assert_eq!(
        value.retained_usage().unwrap(),
        custody::Usage::new(payload.bytes, payload.units)
    );
}

fn full_counts(state: &State, usage: DomUsage) -> transaction::Counts {
    let mut counts = transaction::Counts {
        live_ranges: usage.ranges,
        meta_color_scheme_bytes: usage.meta_color_scheme_bytes,
        autofocus_candidates: usage.autofocus_candidates,
        // Counts stores per-node references. Admission adds the independent
        // signal queue length; public DomUsage includes both domains.
        slot_references: 0,
        select_tasks: state
            .native_tasks
            .iter()
            .filter(|task| matches!(task, super::select::Task::Update { .. }))
            .count(),
        details_intentions: state.details_intentions.len(),
        transitions: usage.transitions,
        records: usage.records,
        pseudos: usage.pseudos,
        ..transaction::Counts::default()
    };
    for (_, slot) in state.slots.iter() {
        if let Some(node) = &slot.node {
            add(&mut counts.slot_references, node.manual_nodes.len());
            add(&mut counts.slot_references, node.assigned_nodes.len());
            add(&mut counts.fullscreen_events, node.fullscreen_events.len());
            add(
                &mut counts.base_observations,
                node.base_document
                    .as_ref()
                    .and_then(|base| base.facts.as_ref())
                    .map_or(0, |facts| facts.formation_usage().record_capacity()),
            );
            add(
                &mut counts.encoding_observations,
                node.encoding_url_document
                    .as_ref()
                    .and_then(|document| document.facts.as_ref())
                    .map_or(0, |facts| facts.formation_usage().record_capacity()),
            );
            add(
                &mut counts.templates,
                usize::from(
                    node.template_contents.is_some()
                        || node.template_document.is_some_and(|document| {
                            matches!(document, crate::template::TemplateDocument::InertTemplate)
                                || document.inert().is_some()
                        }),
                ),
            );
            add(
                &mut counts.shadow_roots,
                usize::from(matches!(node.kind, DomNodeKind::ShadowRoot(_))),
            );
            add(
                &mut counts.slots,
                usize::from(
                    matches!(&node.kind,DomNodeKind::Element(element) if element.is_slot()),
                ),
            );
        }
    }
    counts
}

fn validate_full_state(owner: DomOwnerId, state: &State) {
    assert_eq!(
        state.slots.len(),
        state.slots.iter().last().map_or(0, |(index, _)| index + 1)
    );
    assert_eq!(
        state.ranges.len(),
        state.ranges.iter().last().map_or(0, |(index, _)| index + 1)
    );
    assert_eq!(
        state.slots.lowest_vacant(),
        state
            .slots
            .iter()
            .find(|(_, slot)| slot.node.is_none() && !slot.retired)
            .map(|(index, _)| index)
    );
    for (slot_index, slot) in state.slots.iter() {
        let Some(node) = &slot.node else {
            continue;
        };
        let id = NodeId {
            owner,
            slot: slot_index,
            generation: slot.generation,
        };
        let payload = full_node_payload(node);
        assert_eq!(
            node.receipts.payload,
            custody::Usage::new(payload.bytes, payload.units)
        );
        let fields = full_fields_payload(node);
        assert_eq!(
            node.receipts.fields,
            custody::Usage::new(
                size_of::<transaction::NodeFields>() + fields.bytes,
                fields.units
            )
        );
        let names = full_names_payload(node);
        assert_eq!(
            node.receipts.names,
            custody::Usage::new(names.bytes, names.units)
        );
        assert_eq!(
            node.receipts.name_units,
            node.form_past_names.as_ref().map_or(0, |names| names
                .entries
                .iter()
                .map(|entry| entry.name.len())
                .sum())
        );
        assert!(matches!(
            state.node(owner, node.document.node()).unwrap().kind,
            DomNodeKind::Document(_)
        ));
        let mut children = HashSet::new();
        for child in &node.children {
            assert!(children.insert(*child), "duplicate ordinary child");
            assert_eq!(state.node(owner, *child).unwrap().parent, Some(id));
        }
        if let Some(parent) = node.parent {
            assert!(state.node(owner, parent).unwrap().children.contains(&id));
        }
        ancestor_path(state, owner, id);
        if let Some(root) = node.shadow_root {
            let root = state.node(owner, root).unwrap();
            assert!(matches!(root.kind, DomNodeKind::ShadowRoot(_)));
            assert_eq!(root.host, Some(id));
        }
        if let Some(contents) = node.template_contents {
            let contents = state.node(owner, contents).unwrap();
            assert!(matches!(contents.kind, DomNodeKind::DocumentFragment));
            assert_eq!(contents.host, Some(id));
        }
        if let Some(host) = node.host {
            let host = state.node(owner, host).unwrap();
            match node.kind {
                DomNodeKind::ShadowRoot(_) => assert_eq!(host.shadow_root, Some(id)),
                DomNodeKind::DocumentFragment => assert_eq!(host.template_contents, Some(id)),
                _ => panic!("host association belongs to ShadowRoot or Template contents"),
            }
        }
        for target in &node.manual_nodes {
            assert_eq!(state.node(owner, *target).unwrap().manual_slot, Some(id));
        }
        // Assigned lists and stored inverse fields are separate historical
        // source records. Recomputing a list does not clear removed members'
        // old inverse (docs/slots-and-checkpoints.md; DOM assign-slottables).
        let mut assigned = HashSet::new();
        for target in &node.assigned_nodes {
            assert!(assigned.insert(*target), "unique stored assigned nodes");
            assert!(matches!(
                state.node(owner, *target).unwrap().kind,
                DomNodeKind::Element(_) | DomNodeKind::Text(_)
            ));
        }
        if let Some(slot) = node.manual_slot {
            assert!(state.node(owner, slot).unwrap().manual_nodes.contains(&id));
        }
        if let Some(slot) = node.assigned_slot {
            assert!(
                matches!(&state.node(owner,slot).unwrap().kind,DomNodeKind::Element(element) if element.is_slot())
            );
        }
        if let DomActiveRangeState::Known(Some(range)) = node.active_range {
            assert_eq!(range.owner, owner);
            let slot = state.ranges.get(range.slot).unwrap();
            assert_eq!(slot.generation, range.generation);
            assert!(slot.record.is_some());
        }
    }
    for (_, slot) in state.ranges.iter() {
        let Some(record) = slot.record else {
            continue;
        };
        assert!(record.start.offset <= boundary_length(state, owner, record.start.container));
        assert!(record.end.offset <= boundary_length(state, owner, record.end.container));
        assert!(ordered_boundaries(state, owner, record.start, record.end));
    }
}

/// Compare independently enumerated semantic object bodies with their actual
/// owner-side payload tokens. Administrative/control union is qualified by the
/// owning capsule tests, separately from these semantic materializations.
fn assert_auxiliary_payloads(dom: &Dom) {
    fn union(
        objects: impl IntoIterator<Item = (custody::MaterializationId, custody::Usage)>,
    ) -> BTreeMap<custody::MaterializationId, custody::Usage> {
        let mut result = BTreeMap::new();
        for (identity, usage) in objects {
            if let Some(previous) = result.insert(identity, usage) {
                assert_eq!(
                    previous, usage,
                    "one immutable materialization has one extent"
                );
            }
        }
        result
    }
    let mut raw = Vec::new();
    if let Some(history) = &dom.state.hosted_history {
        raw.extend(history.inspect_materializations().unwrap());
    }
    if let Some(registries) = &dom.state.registries {
        raw.extend(registries.inspect_materializations().unwrap());
    }
    if let Some(globals) = &dom.state.hosted_globals {
        raw.extend(globals.inspect_materializations().unwrap());
    }
    let claimed = dom
        .state_custody
        .auxiliary
        .inspect_claimed_materializations();
    assert_eq!(union(claimed), union(raw));
}

pub(super) fn assert_full_canonical(dom: &Dom) {
    validate_full_state(dom.owner(), &dom.state);
    assert_eq!(dom.usage(), full_usage(dom));
    // This independent producer walk uses actual chunk/header capacities and
    // raw change slices; its claimed side reads the live owner Debits only.
    let (history_actual, history_claimed) = dom.history.inspect_storage().unwrap();
    assert_eq!(history_claimed, history_actual);
    assert_eq!(dom.state.slots.totals(), full_slot_totals(&dom.state));
    assert_eq!(dom.state.ranges.totals(), full_range_totals(&dom.state));
    assert_eq!(
        dom.state.slots.index_charge(),
        slot_index_charge(&dom.state)
    );
    assert_eq!(
        dom.state.ranges.index_charge(),
        range_index_charge(&dom.state)
    );
    let expected = full_root_payload(&dom.state);
    assert_eq!(
        dom.state.root_usage().unwrap(),
        custody::Usage::new(expected.bytes, expected.units)
    );
    assert_eq!(dom.state.counts, full_counts(&dom.state, full_usage(dom)));
    assert_range_dependencies(dom.owner(), &dom.state);
    assert_inbound(dom.owner(), &dom.state);
    assert_queue_targets(dom.owner(), &dom.state);
    assert_control_index(dom.owner(), &dom.state);
    assert_slot_participants(dom.owner(), &dom.state);
    assert_slot_names(dom.owner(), &dom.state);
    assert_transition_indexes(dom.owner(), &dom.state);
    assert_role_facts(dom.owner(), &dom.state);
    assert_family_discovery(dom.owner(), &dom.state);
    assert_hosted_containers(dom.owner(), &dom.state);
    assert_auxiliary_payloads(dom);
}

fn assert_full_delta(dom: &Dom, before: &DomSnapshot, actual: &[DomChange]) {
    let expected = super::full_difference(
        dom.owner(),
        &before.state,
        &dom.state,
        &AtomicBool::new(false),
    )
    .unwrap();
    assert_eq!(actual, expected);
    assert_full_canonical(dom);
}

fn owner() -> Dom {
    Dom::new(
        DomLimits::new(256, 65_536, 512, 1024, 1024, 32, 1_000_000)
            .unwrap()
            .with_ranges(64)
            .unwrap(),
    )
    .unwrap()
}

fn apply(dom: &mut Dom, batch: DomBatch) -> DomCommit {
    let before = dom.snapshot();
    let commit = dom.apply(batch, &AtomicBool::new(false)).unwrap();
    assert_full_delta(dom, &before, commit.changes().changes());
    commit
}

fn fixture() -> (Dom, DocumentId, NodeId, NodeId) {
    let mut dom = owner();
    let mut batch = DomBatch::new();
    let document = batch.create_document();
    let parent = batch.create_element(&document, "root");
    let text = batch.create_text(&document, "abcd");
    batch.insert_before(&parent, &document, None);
    batch.insert_before(&text, &parent, None);
    let commit = apply(&mut dom, batch);
    (
        dom,
        commit.document(&document).unwrap(),
        commit.created(&parent).unwrap().unwrap(),
        commit.created(&text).unwrap().unwrap(),
    )
}

#[test]
fn full_oracle_detects_indirect_range_boundary_and_local_text_changes() {
    let (mut dom, document, _, text) = fixture();
    let mut batch = DomBatch::new();
    let range = batch.create_range(document);
    batch.set_range_start(&range, text, 1);
    batch.set_range_end(&range, text, 3);
    let range = apply(&mut dom, batch)
        .created_range(&range)
        .unwrap()
        .unwrap();
    let old = dom.snapshot();
    let mut batch = DomBatch::new();
    batch.replace_data(text, 1, 2, "X");
    let commit = apply(&mut dom, batch);
    assert_eq!(
        commit.changes().changes(),
        &[DomChange::RangeBoundary(range), DomChange::Data(text)]
    );
    // DOM replace-data collapses boundaries in (offset, offset+count] to
    // offset; it does not shift an endpoint within the replaced interval to
    // the end of the newly supplied string.
    assert_eq!(dom.snapshot().range(range).unwrap().end().offset(), 1);
    assert_eq!(old.range(range).unwrap().end().offset(), 3);
    assert_eq!(
        dom.snapshot().node(text).unwrap().data(),
        Some(&DocumentString::from("aXd"))
    );
}

#[test]
fn full_oracle_covers_tickets_retirement_and_exact_semantic_noop() {
    let (mut dom, document, parent, _) = fixture();
    let mut batch = DomBatch::new();
    let old = batch.create_comment(document, "equal");
    let old = apply(&mut dom, batch).created(&old).unwrap().unwrap();
    let before = dom.snapshot();
    let usage = dom.usage();
    let mut batch = DomBatch::new();
    let private = batch.create_comment(document, "private");
    batch.destroy(&private);
    batch.set_attribute(parent, "temporary", "value");
    batch.remove_attribute(parent, "temporary");
    let commit = apply(&mut dom, batch);
    assert_eq!(commit.created(&private).unwrap(), None);
    assert!(commit.changes().changes().is_empty());
    assert_eq!(dom.snapshot().stamp(), before.stamp());
    assert_eq!(dom.usage(), usage);
    let mut batch = DomBatch::new();
    batch.destroy(old);
    let fresh = batch.create_comment(document, "equal");
    let commit = apply(&mut dom, batch);
    let fresh = commit.created(&fresh).unwrap().unwrap();
    assert_eq!(
        commit.changes().changes(),
        &[DomChange::Destroyed(old), DomChange::Created(fresh)]
    );
    assert_ne!(fresh, old);
    assert_eq!(dom.snapshot().node(old).unwrap_err(), DomError::StaleNode);
    assert_eq!(
        before.node(old).unwrap().data(),
        Some(&DocumentString::from("equal"))
    );
}

#[test]
fn source_attribute_publication_matches_full_oracle_and_unpublished_drop_preserves_root() {
    let (mut dom, _, parent, _) = fixture();
    let before = dom.snapshot();
    let cancel = AtomicBool::new(false);
    let limits = DomSourceLimits::new(32_000_000, 1_000_000, 1_000_000, 8, 32, 32, 1);
    let changes = {
        let mut lease = dom
            .begin_source_edit(
                DomSourceEdit::set_attribute(parent, "data-local", "kept"),
                limits,
                &cancel,
            )
            .unwrap();
        let DomSourceProgress::Complete(done) = lease.advance(&cancel).unwrap() else {
            panic!("plain XML requires no service");
        };
        done.commit().changes().changes().to_vec()
    };
    assert_eq!(changes, [DomChange::Attributes(parent)]);
    assert_full_delta(&dom, &before, &changes);
    let before = dom.snapshot();
    let usage = dom.usage();
    let lease = dom
        .begin_source_edit(
            DomSourceEdit::create_comment(
                before
                    .node(parent)
                    .unwrap()
                    .owner_document()
                    .unwrap()
                    .node(),
                "unpublished",
            ),
            limits,
            &cancel,
        )
        .unwrap();
    drop(lease);
    assert_eq!(dom.snapshot().stamp(), before.stamp());
    assert_eq!(dom.usage(), usage);
    assert_full_delta(&dom, &before, &[]);
}

#[test]
fn source_equal_text_replacement_publishes_the_actual_range_only_change() {
    let (mut dom, document, _, text) = fixture();
    let mut batch = DomBatch::new();
    let range = batch.create_range(document);
    batch.set_range_start(&range, text, 1);
    batch.set_range_end(&range, text, 3);
    let range = apply(&mut dom, batch)
        .created_range(&range)
        .unwrap()
        .unwrap();
    let before = dom.snapshot();
    let cancel = AtomicBool::new(false);
    let limits = DomSourceLimits::new(32_000_000, 1_000_000, 1_000_000, 8, 32, 32, 1);
    let changes = {
        let mut lease = dom
            .begin_source_edit(
                DomSourceEdit::replace_data(text, 1, 2, "bc"),
                limits,
                &cancel,
            )
            .unwrap();
        let DomSourceProgress::Complete(done) = lease.advance(&cancel).unwrap() else {
            panic!("native text replacement requires no external service");
        };
        done.commit().changes().changes().to_vec()
    };
    assert_eq!(changes, [DomChange::RangeBoundary(range)]);
    assert_full_delta(&dom, &before, &changes);
    let now = dom.snapshot();
    assert_eq!(
        now.node(text).unwrap().data(),
        Some(&DocumentString::from("abcd"))
    );
    assert!(now.range(range).unwrap().collapsed());
    assert_eq!(now.range(range).unwrap().start().offset(), 1);
    assert_eq!(before.range(range).unwrap().end().offset(), 3);
}

#[test]
fn full_oracle_composes_native_relations_template_and_queued_targets() {
    use crate::{HtmlShadowRootMode, HtmlShadowRootProperties, HtmlSlotAssignment, Namespace};
    let limits = DomLimits::new(256, 65_536, 512, 1024, 1024, 32, 1_000_000)
        .unwrap()
        .with_select_processing(32, 4_000_000, 10_000_000)
        .unwrap()
        .with_details_toggle_intentions(32)
        .unwrap();
    let mut dom = Dom::new(limits).unwrap();
    let mut batch = DomBatch::new();
    let document = batch.create_document();
    let host = batch.create_element_ns(&document, Namespace::html(), "div");
    batch.insert_before(&host, &document, None);
    let shadow = batch.attach_shadow(
        &host,
        HtmlShadowRootProperties::new(
            HtmlShadowRootMode::Open,
            HtmlSlotAssignment::Manual,
            false,
            true,
            false,
        ),
    );
    let slot = batch.create_element_ns(&document, Namespace::html(), "slot");
    batch.insert_before(&slot, &shadow, None);
    let text = batch.create_text(&document, "slotted");
    batch.insert_before(&text, &host, None);
    batch.assign_slot(&slot, vec![(&text).into()]);
    let template = batch.create_element_ns(&document, Namespace::html(), "template");
    let select = batch.create_element_ns(&document, Namespace::html(), "select");
    // Presence includes the empty form attribute. Both radios initially have
    // one ordinary root, null owners and the exact same nonempty name.
    batch.set_attribute(&select, "form", "");
    let radio_a = batch.create_element_ns(&document, Namespace::html(), "input");
    let radio_b = batch.create_element_ns(&document, Namespace::html(), "input");
    for radio in [&radio_a, &radio_b] {
        batch.set_attribute(radio, "type", "radio");
        batch.set_attribute(radio, "name", "g");
        batch.set_attribute(radio, "form", "");
        batch.insert_before(radio, &host, None);
    }
    let option = batch.create_element_ns(&document, Namespace::html(), "option");
    let details = batch.create_element_ns(&document, Namespace::html(), "details");
    for child in [&template, &select, &details] {
        batch.insert_before(child, &host, None);
    }
    batch.insert_before(&option, &select, None);
    let commit = apply(&mut dom, batch);
    let id = |ticket: &crate::CreatedNode| commit.created(ticket).unwrap().unwrap();
    let slot = id(&slot);
    let text = id(&text);
    let details = id(&details);
    let option = id(&option);
    let template = id(&template);
    let shadow = id(&shadow);
    let radio_b = id(&radio_b);
    let old = dom.snapshot();
    let contents = old
        .node(template)
        .unwrap()
        .as_template()
        .unwrap()
        .contents()
        .unwrap();
    assert_eq!(old.node(contents).unwrap().fragment_host(), Some(template));
    assert_eq!(old.slot(slot).unwrap().manual_assignments(), &[text]);
    let mut batch = DomBatch::new();
    batch.set_attribute(details, "open", "");
    batch.set_attribute(option, "selected", "");
    // Removing an actually selected Option queues its original Select update.
    batch.detach(option);
    apply(&mut dom, batch);
    assert!(!dom.state.details_intentions.is_empty());
    assert!(
        dom.state
            .native_tasks
            .iter()
            .any(|task| matches!(task, super::select::Task::Update { .. }))
    );
    assert!(
        old.node(details)
            .unwrap()
            .as_element()
            .unwrap()
            .attributes()
            .is_empty()
    );
    let mut batch = DomBatch::new();
    batch.assign_slot(slot, Vec::new());
    batch.set_attribute(details, "data-local", "after");
    // Preserve form presence but move one actual radio to a different ordinary
    // root and give it an empty name, which excludes radio-group membership.
    batch.set_attribute(radio_b, "name", "");
    batch.insert_before(radio_b, shadow, None);
    apply(&mut dom, batch);
    assert_eq!(old.slot(slot).unwrap().manual_assignments(), &[text]);
    let current = dom.snapshot();
    assert!(current.slot(slot).unwrap().assigned_nodes().is_empty());
    assert_eq!(
        current.node(text).unwrap().stored_assigned_slot().unwrap(),
        Some(slot)
    );
    assert_eq!(old.slot(slot).unwrap().assigned_nodes(), &[text]);
    assert!(
        dom.snapshot()
            .slot(slot)
            .unwrap()
            .manual_assignments()
            .is_empty()
    );
}

#[test]
fn full_oracle_tracks_slot_order_across_separate_roots_adoption_and_reuse() {
    use crate::{HtmlShadowRootMode, HtmlShadowRootProperties, HtmlSlotAssignment, Namespace};
    let mut dom = owner();
    let mut batch = DomBatch::new();
    let document = batch.create_document();
    let other = batch.create_document();
    let root = batch.create_element(&document, "root");
    let branch = batch.create_element(&document, "branch");
    let host = batch.create_element_ns(&document, Namespace::html(), "div");
    let template = batch.create_element_ns(&document, Namespace::html(), "template");
    batch.insert_before(&root, &document, None);
    for child in [&branch, &host, &template] {
        batch.insert_before(child, &root, None);
    }
    let shadow = batch.attach_shadow(
        &host,
        HtmlShadowRootProperties::new(
            HtmlShadowRootMode::Open,
            HtmlSlotAssignment::Named,
            false,
            true,
            false,
        ),
    );
    let commit = apply(&mut dom, batch);
    let document = commit.document(&document).unwrap();
    let other = commit.document(&other).unwrap();
    let root = commit.created(&root).unwrap().unwrap();
    let branch = commit.created(&branch).unwrap().unwrap();
    let host = commit.created(&host).unwrap().unwrap();
    let template = commit.created(&template).unwrap().unwrap();
    let shadow = commit.created(&shadow).unwrap().unwrap();
    let contents = dom
        .snapshot()
        .node(template)
        .unwrap()
        .as_template()
        .unwrap()
        .contents()
        .unwrap();

    let mut batch = DomBatch::new();
    let tickets: Vec<_> = (0..9)
        .map(|_| batch.create_element_ns(document, Namespace::html(), "slot"))
        .collect();
    batch.insert_before(&tickets[0], root, Some(branch.into()));
    batch.insert_before(&tickets[1], branch, None);
    batch.insert_before(&tickets[2], branch, None);
    batch.insert_before(&tickets[3], root, Some(host.into()));
    batch.insert_before(&tickets[4], &tickets[3], None);
    batch.insert_before(&tickets[5], root, Some(host.into()));
    batch.insert_before(&tickets[6], root, Some(host.into()));
    batch.insert_before(&tickets[7], shadow, None);
    batch.insert_before(&tickets[8], contents, None);
    let commit = apply(&mut dom, batch);
    let slots: Vec<_> = tickets
        .iter()
        .map(|ticket| commit.created(ticket).unwrap().unwrap())
        .collect();
    let before = dom.snapshot();
    assert_eq!(
        before.node(root).unwrap().children(),
        &[
            slots[0], branch, slots[3], slots[5], slots[6], host, template
        ]
    );
    assert_eq!(
        before.node(branch).unwrap().children(),
        &[slots[1], slots[2]]
    );
    assert_eq!(before.node(slots[3]).unwrap().children(), &[slots[4]]);
    assert_eq!(before.node(shadow).unwrap().children(), &[slots[7]]);
    assert_eq!(before.node(contents).unwrap().children(), &[slots[8]]);

    let mut batch = DomBatch::new();
    batch.insert_before(branch, root, Some(slots[0].into()));
    batch.insert_before(slots[6], branch, Some(slots[2].into()));
    batch.detach(slots[3]);
    batch.adopt(branch, other);
    apply(&mut dom, batch);
    let moved = dom.snapshot();
    assert_eq!(
        moved.node(root).unwrap().children(),
        &[slots[0], slots[5], host, template]
    );
    assert_eq!(moved.node(branch).unwrap().parent(), None);
    assert_eq!(
        moved.node(branch).unwrap().children(),
        &[slots[1], slots[6], slots[2]]
    );
    for node in [branch, slots[1], slots[6], slots[2]] {
        assert_eq!(moved.node(node).unwrap().owner_document(), Some(other));
    }
    // A detached Slot is itself the inclusive first participant of its root;
    // its ordinary child remains in that group without host/template edges.
    assert_eq!(moved.node(slots[3]).unwrap().parent(), None);
    assert_eq!(moved.node(slots[3]).unwrap().children(), &[slots[4]]);
    assert_eq!(moved.node(shadow).unwrap().children(), &[slots[7]]);
    assert_eq!(moved.node(contents).unwrap().children(), &[slots[8]]);
    assert_eq!(
        before.node(branch).unwrap().children(),
        &[slots[1], slots[2]]
    );
    assert_eq!(before.node(slots[6]).unwrap().parent(), Some(root));
    assert_slot_participants(dom.owner(), &before.state);

    let mut batch = DomBatch::new();
    batch.detach(slots[2]);
    batch.destroy(slots[2]);
    let replacement = batch.create_element_ns(other, Namespace::html(), "slot");
    batch.insert_before(&replacement, branch, Some(slots[1].into()));
    let commit = apply(&mut dom, batch);
    let replacement = commit.created(&replacement).unwrap().unwrap();
    assert_eq!(replacement.slot, slots[2].slot);
    assert_eq!(replacement.generation, slots[2].generation + 1);
    assert_eq!(
        dom.snapshot().node(slots[2]).unwrap_err(),
        DomError::StaleNode
    );
    assert_eq!(
        dom.snapshot().node(branch).unwrap().children(),
        &[replacement, slots[1], slots[6]]
    );
    assert_eq!(moved.node(slots[2]).unwrap().owner_document(), Some(other));
    assert_eq!(
        before.node(slots[2]).unwrap().owner_document(),
        Some(document)
    );
    assert_slot_participants(dom.owner(), &before.state);
    assert_slot_participants(dom.owner(), &moved.state);
}

#[test]
fn full_oracle_keeps_queued_details_original_document_after_adoption() {
    let limits = DomLimits::new(256, 100_000, 512, 1024, 1024, 32, 4_000_000)
        .unwrap()
        .with_details_toggle_intentions(32)
        .unwrap();
    // Reuse the actual DOM issuer/constructor boundary. The live original
    // source validates the association; this asserts no Runtime Window service.
    let (mut dom, _source, original_document, _) =
        source::operation::fullscreen::native_fixture(limits);
    assert_full_canonical(&dom);
    let parent = dom
        .snapshot()
        .document(original_document)
        .unwrap()
        .document_element()
        .unwrap();
    let mut batch = DomBatch::new();
    let details = batch.create_element_ns(original_document, crate::Namespace::html(), "details");
    batch.insert_before(&details, parent, None);
    let other_document = batch.create_document();
    let commit = apply(&mut dom, batch);
    let details = commit.created(&details).unwrap().unwrap();
    let other_document = commit.document(&other_document).unwrap();
    let mut batch = DomBatch::new();
    batch.set_attribute(details, "open", "");
    apply(&mut dom, batch);
    let queued = dom.snapshot();
    let row = dom
        .state
        .details_intentions
        .iter()
        .find(|row| row.target() == details)
        .unwrap();
    let operation = row.operation();
    let association = row
        .queue_association()
        .expect("actual live hosted association");
    assert_eq!(association.document(), original_document);
    assert_eq!(
        dom.state.details_targets.inspect_associations(),
        [(
            super::Key::Existing(details),
            super::Key::Existing(original_document.node()),
            1
        )]
    );
    let mut batch = DomBatch::new();
    batch.adopt(details, other_document);
    apply(&mut dom, batch);
    assert_eq!(
        dom.snapshot().node(details).unwrap().owner_document(),
        Some(other_document)
    );
    assert_eq!(
        queued.node(details).unwrap().owner_document(),
        Some(original_document)
    );
    let row = dom
        .state
        .details_intentions
        .iter()
        .find(|row| row.operation() == operation)
        .unwrap();
    assert_eq!(row.target(), details);
    assert_eq!(row.queue_association(), Some(association));
    assert_eq!(
        dom.state.details_targets.inspect_associations(),
        [(
            super::Key::Existing(details),
            super::Key::Existing(original_document.node()),
            1
        )]
    );
}
