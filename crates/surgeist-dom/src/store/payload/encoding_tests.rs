//! Small graphs of the selected production representation. Raw expected
//! extents enumerate actual targets/capacities, excluding opaque Arc metadata.
use super::super::{
    Draft, SharedVec, bank, canonical, compact_tests::AllocationGraph, transaction,
};
use super::*;
use crate::{DocumentId, DocumentString, Dom, DomBatch, DomNodeKind, NodeId};
use std::sync::atomic::{AtomicBool, Ordering};

fn unlimited(_: Usage) -> Result<(), DomError> {
    Ok(())
}
fn seal<T>(value: T, extra: Usage, registry: &bank::CustodyRegistry) -> Sealed<T> {
    let own = Usage::new(size_of::<Payload<T>>(), 0)
        .checked_add(extra)
        .unwrap();
    Pending::new(
        custody::Held::new(
            value,
            registry.ledger().retain(Usage::ZERO, unlimited).unwrap(),
        ),
        own,
        unlimited,
    )
    .unwrap()
    .seal(registry, unlimited)
    .unwrap()
}
fn fixture() -> (Dom, DocumentId) {
    let mut dom = Dom::new(super::super::compact_tests::limits()).unwrap();
    let mut batch = DomBatch::new();
    let document = batch.create_document();
    let done = dom.apply(batch, &AtomicBool::new(false)).unwrap();
    (dom, done.document(&document).unwrap())
}
#[test]
fn actual_slot_root_aggregate_preserves_projection_refusal_and_historical_readoption() {
    let (mut dom, document) = fixture();
    let original = dom.snapshot();
    let cancel = AtomicBool::new(false);
    let mut batch = DomBatch::new();
    // This genuine PI command reaches the immutable parser's failed-map path:
    // an actual reserved bank survives, while the observed attribute list is
    // empty. Capture its raw backing before projection, not an admission total.
    let pi_ticket = batch.create_processing_instruction(document, "target", "a='valid' broken");
    let doctype_ticket = batch.create_document_type(document, "html", "public", "system");
    let mut draft = Draft::new_source(
        &dom,
        batch.owner.clone(),
        &batch.commands,
        crate::DomSourceLimits::new(32_000_000, 100_000, 1_000_000, 16, 64, 64, 1),
        &cancel,
    )
    .unwrap()
    .0;
    for command in &batch.commands {
        draft.apply(command, dom.limits, &cancel).unwrap();
    }
    let pi = draft.resolve((&pi_ticket).into()).unwrap();
    let doctype = draft.resolve((&doctype_ticket).into()).unwrap();
    let transaction::KindRead::ProcessingInstruction(staged_pi) = draft.node(pi).unwrap().kind()
    else {
        panic!("actual staged PI")
    };
    assert_eq!(staged_pi.attributes().len(), 0);
    let mut staged_graph = AllocationGraph::default();
    staged_pi.inspect_compact_baseline(&mut staged_graph);
    let bank_target = staged_graph.summary()["pi.attributes.target"];
    let bank_buffer = staged_graph.summary()["pi.attributes.buffer"];
    assert_eq!(
        bank_target,
        (1, size_of::<Vec<(DocumentString, DocumentString)>>())
    );
    assert_eq!(bank_buffer.0, 1);
    assert!(
        bank_buffer.1 > 0,
        "the actual parser retained spare map capacity"
    );
    draft.custody_registry().reconcile_releases();
    let before = draft.custody_ledger().total();
    let work = draft.source_work.source_arc().clone();
    let spent = work.load(Ordering::Relaxed);
    let refusal = identity_refusal::Refusal::begin();
    assert!(matches!(
        draft.finish(&dom, &cancel),
        Err(DomError::RepresentationExhausted)
    ));
    assert!(
        refusal.reached(),
        "actual projected root identity reservation reached"
    );
    drop(refusal);
    draft.custody_registry().reconcile_releases();
    assert_eq!(draft.custody_ledger().total(), before);
    assert_eq!(dom.snapshot().stamp(), original.stamp());
    assert!(Arc::ptr_eq(&dom.state, &original.state));
    assert!(
        matches!(draft.node(pi).unwrap().kind(), transaction::KindRead::ProcessingInstruction(value) if value.target() == &DocumentString::from("target") && value.data() == &DocumentString::from("a='valid' broken"))
    );
    assert!(
        matches!(draft.node(doctype).unwrap().kind(), transaction::KindRead::DocumentType(value) if value.name() == &DocumentString::from("html"))
    );
    assert!(Arc::ptr_eq(&work, draft.source_work.source_arc()));
    assert!(work.load(Ordering::Relaxed) >= spent);

    let prepared = draft.finish(&dom, &cancel).unwrap();
    let commit = dom.publish(prepared, &cancel).unwrap();
    let pi = commit.created(&pi_ticket).unwrap().unwrap();
    let doctype = commit.created(&doctype_ticket).unwrap().unwrap();
    drop(draft);
    drop(original);
    let historical = dom.snapshot();
    let canonical::Node::ProcessingInstruction(root) = dom.state.node(dom.owner, pi).unwrap()
    else {
        panic!("actual PI root")
    };
    let root_id = root.identity;
    let leaf_id = root.value.data.identity;
    let root_weak = Arc::downgrade(root);
    let leaf_weak = Arc::downgrade(&root.value.data);
    assert_eq!(root.value.data.value.attributes().len(), 0);
    let mut graph = AllocationGraph::default();
    super::super::compact_tests::typed_graph::inspect_node(
        dom.state.node(dom.owner, pi).unwrap(),
        &mut graph,
    );
    assert_eq!(graph.summary()["pi.attributes.target"], bank_target);
    assert_eq!(graph.summary()["pi.attributes.buffer"], bank_buffer);
    let mut canonical_pi_graph = AllocationGraph::default();
    root.value
        .data
        .value
        .inspect_compact_baseline(&mut canonical_pi_graph);
    assert_eq!(
        canonical_pi_graph, staged_graph,
        "canonical projection aliases the original independently controlled PI backing"
    );
    super::super::oracle_tests::assert_full_canonical(&dom);
    let claimed = inspect_slot_typed_custody(&dom.state_custody.slots);
    assert!(
        !claimed.contains_key(&root_id),
        "root body belongs to its actual Slot receipt, not a root control"
    );
    assert!(
        claimed.contains_key(&leaf_id),
        "independent language target remains controlled"
    );
    let canonical::Node::DocumentType(doctype_root) = dom.state.node(dom.owner, doctype).unwrap()
    else {
        panic!("actual doctype root")
    };
    let doctype_weak = Arc::downgrade(doctype_root);
    assert_eq!(
        doctype_root.value.data.value.system_id(),
        &DocumentString::from("system")
    );
    let ledger = dom.custody_registry.ledger().clone();
    drop(dom);
    assert_eq!(ledger.total(), Usage::ZERO);
    assert!(root_weak.upgrade().is_some());
    assert!(leaf_weak.upgrade().is_some());
    assert!(doctype_weak.upgrade().is_some());
    let registry = bank::CustodyRegistry::new(&ledger, unlimited).unwrap();
    let baseline = ledger.total();
    let captured =
        transaction::StateCustody::adopt(&historical.state, &registry, None, unlimited, || Ok(()))
            .unwrap();
    assert_eq!(
        inspect_slot_typed_custody(&captured.slots),
        super::super::compact_tests::typed_graph::materializations(&historical.state)
    );
    let captured_total = ledger.total();
    let destruction = destruction_observation::Observation::begin(root_id, &ledger);
    drop(historical);
    assert_eq!(
        destruction.observed(),
        Some(captured_total),
        "actual root data dies while its Slot aggregate receipt is still retained"
    );
    assert!(root_weak.upgrade().is_none());
    assert!(leaf_weak.upgrade().is_none());
    assert!(doctype_weak.upgrade().is_none());
    drop(destruction);
    drop(captured);
    registry.reconcile_releases();
    assert_eq!(ledger.total(), baseline);
    drop(registry);
    assert_eq!(ledger.total(), Usage::ZERO);
}

#[test]
fn actual_language_scope_distinguishes_absent_nonce_from_applicable_empty_nonce_and_null_focus() {
    let (mut dom, document) = fixture();
    let mut batch = DomBatch::new();
    let plain = batch.create_element_ns(document, crate::Namespace::none(), "plain");
    let html = batch.create_element_ns(document, crate::Namespace::html(), "div");
    let done = dom.apply(batch, &AtomicBool::new(false)).unwrap();
    let plain = done.created(&plain).unwrap().unwrap();
    let html = done.created(&html).unwrap().unwrap();
    let DomNodeKind::Element(plain) = dom.state.node(dom.owner, plain).unwrap().kind() else {
        panic!("plain Element")
    };
    let DomNodeKind::Element(html) = dom.state.node(dom.owner, html).unwrap().kind() else {
        panic!("HTML Element")
    };
    assert!(plain.nonce_payload().is_none());
    assert!(plain.nonce().is_none());
    assert!(plain.focus_payload().is_none());
    let nonce = html.nonce_payload().unwrap();
    assert!(nonce.value.value.is_empty());
    assert_eq!(
        nonce.own_usage,
        Usage::new(
            size_of::<Payload<crate::element::NonceSlot>>() + size_of::<Vec<u16>>(),
            0
        )
    );
    assert!(html.focus_payload().is_none());
    assert!(
        html.common_state()
            .html()
            .unwrap()
            .previously_focused_element()
            .is_none()
    );
    super::super::oracle_tests::assert_full_canonical(&dom);
}

#[test]
fn actual_document_presence_preserves_empty_semantics_and_reserved_storage_custody() {
    type State = canonical::Interactive<NodeId>;
    assert_ne!(State::Absent, State::Unavailable);
    assert_ne!(State::Unavailable, State::KnownEmpty);
    assert_ne!(State::Absent, State::KnownEmpty);
    let ledger = custody::Ledger::default();
    let registry = bank::CustodyRegistry::new(&ledger, unlimited).unwrap();
    let baseline = registry.total();
    let values = Vec::<NodeId>::with_capacity(7);
    let capacity = values.capacity();
    let values = canonical::InteractiveValues {
        open_dialogs: SharedVec::from(values),
        top_layer: SharedVec::default(),
        pending_top_layer_removals: SharedVec::default(),
    };
    let raw = Usage::new(size_of::<Vec<NodeId>>() + capacity * size_of::<NodeId>(), 0);
    let sealed = seal(values, raw, &registry);
    let weak = Arc::downgrade(&sealed.value);
    let state = State::Known(sealed.value.clone());
    let alias = state.clone();
    assert_eq!(
        state,
        State::KnownEmpty,
        "reserved empty backing does not change the observed empty lists"
    );
    let State::Known(value) = &state else {
        unreachable!()
    };
    assert_eq!(value.identity, sealed.custody.identity());
    assert!(capacity >= 7, "the caller supplied real reserved backing");
    let State::Known(alias_value) = &alias else {
        unreachable!()
    };
    assert!(Arc::ptr_eq(value, alias_value));
    assert!(value.value.open_dialogs.is_empty());
    assert_eq!(value.value.open_dialogs.capacity(), capacity);
    assert!(!value.value.top_layer.has_backing());
    assert!(!value.value.pending_top_layer_removals.has_backing());
    let mut graph = AllocationGraph::default();
    graph.target(value, "document.interactive");
    value.value.open_dialogs.inspect_raw_allocations(&mut graph);
    assert_eq!(
        graph.bytes(),
        size_of::<Payload<canonical::InteractiveValues<NodeId>>>()
            + size_of::<Vec<NodeId>>()
            + capacity * size_of::<NodeId>()
    );
    assert_eq!(
        sealed.custody.inspect_payload_usage(),
        Usage::new(graph.bytes(), 0)
    );
    drop(state);
    registry.reconcile_releases();
    assert!(registry.total().bytes() > baseline.bytes());
    drop(alias);
    drop(sealed);
    registry.reconcile_releases();
    assert!(weak.upgrade().is_none());
    assert_eq!(registry.total(), baseline);
}
