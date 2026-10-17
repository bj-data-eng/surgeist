//! Independent literal oracles: DOM3071 §4.2.2, HTML877d slot.assign,
//! CSS Shadow §4.1 and #1124's selected native destruction policy.
use std::sync::atomic::AtomicBool;
use surgeist_dom::*;

fn cancel() -> AtomicBool {
    AtomicBool::new(false)
}
fn limits(references: usize, inputs: usize) -> DomLimits {
    DomLimits::new(128, 32768, 512, references, inputs, 16, 131072).unwrap()
}
fn queries() -> DomSlotQueryLimits {
    DomSlotQueryLimits::new(128, 512).unwrap()
}
fn apply(dom: &mut Dom, batch: DomBatch) -> DomCommit {
    dom.apply(batch, &cancel()).unwrap()
}
struct Fixture {
    dom: Dom,
    document: DocumentId,
    top: NodeId,
    h1: NodeId,
    h2: NodeId,
    r1: NodeId,
    r2: NodeId,
    s1: NodeId,
    s2: NodeId,
    x: NodeId,
    fallback: NodeId,
}
fn fixture(mode: HtmlSlotAssignment, closed: bool) -> Fixture {
    let mut dom = Dom::new(limits(128, 128)).unwrap();
    let mut b = DomBatch::new();
    let d = b.create_document();
    let top = b.create_element_ns(&d, Namespace::html(), "div");
    let h1 = b.create_element_ns(&d, Namespace::html(), "div");
    let h2 = b.create_element_ns(&d, Namespace::html(), "div");
    b.insert_before(&top, &d, None);
    b.insert_before(&h1, &top, None);
    b.insert_before(&h2, &top, None);
    let properties = HtmlShadowRootProperties::new(
        if closed {
            HtmlShadowRootMode::Closed
        } else {
            HtmlShadowRootMode::Open
        },
        mode,
        false,
        true,
        false,
    );
    let r1 = b.attach_shadow(&h1, properties);
    let r2 = b.attach_shadow(&h2, properties);
    let s1 = b.create_element_ns(&d, Namespace::html(), "slot");
    let s2 = b.create_element_ns(&d, Namespace::html(), "p:slot");
    b.insert_before(&s1, &r1, None);
    b.insert_before(&s2, &r2, None);
    let x = b.create_text(&d, "\u{05d0}");
    b.insert_before(&x, &h1, None);
    let fallback = b.create_text(&d, "LTR");
    b.insert_before(&fallback, &s1, None);
    if mode == HtmlSlotAssignment::Manual {
        b.assign_slot(&s1, vec![(&x).into()]);
    }
    let c = apply(&mut dom, b);
    let id = |t: &CreatedNode| c.created(t).unwrap().unwrap();
    Fixture {
        document: c.document(&d).unwrap(),
        top: id(&top),
        h1: id(&h1),
        h2: id(&h2),
        r1: id(&r1),
        r2: id(&r2),
        s1: id(&s1),
        s2: id(&s2),
        x: id(&x),
        fallback: id(&fallback),
        dom,
    }
}

#[test]
fn cross_root_manual_transfer_preserves_stored_history_but_changes_fresh_queries() {
    let mut f = fixture(HtmlSlotAssignment::Manual, false);
    slot_checkpoint(&mut f.dom, queries(), &cancel()).unwrap();
    let old = f.dom.snapshot();
    let mut b = DomBatch::new();
    b.assign_slot(f.s2, vec![f.x.into()]);
    let commit = apply(&mut f.dom, b);
    let s = f.dom.snapshot();
    assert_eq!(s.slot(f.s1).unwrap().manual_assignments(), &[]);
    assert_eq!(s.slot(f.s2).unwrap().manual_assignments(), &[f.x]);
    assert_eq!(
        s.node(f.x).unwrap().manual_assignment().unwrap(),
        Some(f.s2)
    );
    assert_eq!(s.slot(f.s1).unwrap().assigned_nodes(), &[f.x]);
    assert_eq!(s.slot(f.s2).unwrap().assigned_nodes(), &[]);
    assert_eq!(
        s.node(f.x).unwrap().stored_assigned_slot().unwrap(),
        Some(f.s1)
    );
    assert_eq!(s.node(f.x).unwrap().assigned_slot(&cancel()).unwrap(), None);
    assert_eq!(
        s.slot(f.s1)
            .unwrap()
            .find_slottables(queries(), &cancel())
            .unwrap()
            .nodes(),
        &[]
    );
    assert_eq!(
        s.slot(f.s1)
            .unwrap()
            .flattened_slottables(queries(), &cancel())
            .unwrap()
            .nodes(),
        &[f.fallback]
    );
    assert_eq!(
        s.flat_tree(f.document, queries(), &cancel())
            .unwrap()
            .relation(f.x)
            .unwrap(),
        DomFlatRelation::Excluded
    );
    assert_eq!(s.pending_slot_signals(), &[]);
    assert_eq!(s.node(f.x).unwrap().parent(), Some(f.h1));
    assert_eq!(
        old.node(f.x).unwrap().assigned_slot(&cancel()).unwrap(),
        Some(f.s1)
    );
    assert!(commit.changes().changes().contains(&DomChange::SlotQueries));
    assert!(
        !commit
            .changes()
            .changes()
            .contains(&DomChange::SlotAssignedNodes(f.s1))
    );
    let mut b = DomBatch::new();
    b.set_attribute(f.h1, "ordinary", "unchanged assignment");
    b.set_attribute(f.x, "slot", "x");
    assert_eq!(
        f.dom.apply(b, &cancel()).unwrap_err(),
        DomError::WrongNodeKind
    );
    assert_eq!(f.dom.snapshot().stamp(), s.stamp());
    // Actual removal consults the historical stored field and refreshes S1.
    let mut b = DomBatch::new();
    b.detach(f.x);
    apply(&mut f.dom, b);
    assert_eq!(f.dom.snapshot().slot(f.s1).unwrap().assigned_nodes(), &[]);
    assert_eq!(f.dom.snapshot().pending_slot_signals(), &[f.s1]);
}

#[test]
fn duplicate_raw_inputs_keep_first_order_and_named_assignment_ignores_manual_intent() {
    let mut f = fixture(HtmlSlotAssignment::Manual, false);
    let mut b = DomBatch::new();
    let y = b.create_text(f.document, "Y");
    b.insert_before(&y, f.h1, None);
    b.assign_slot(f.s1, vec![(&y).into(), f.x.into(), (&y).into(), f.x.into()]);
    let c = apply(&mut f.dom, b);
    let y = c.created(&y).unwrap().unwrap();
    let s = f.dom.snapshot();
    assert_eq!(s.slot(f.s1).unwrap().manual_assignments(), &[y, f.x]);
    assert_eq!(s.slot(f.s1).unwrap().assigned_nodes(), &[y, f.x]);
    let mut f = fixture(HtmlSlotAssignment::Named, false);
    let mut b = DomBatch::new();
    b.assign_slot(f.s2, vec![f.x.into()]);
    apply(&mut f.dom, b);
    let s = f.dom.snapshot();
    assert_eq!(s.slot(f.s1).unwrap().assigned_nodes(), &[f.x]);
    assert_eq!(
        s.node(f.x).unwrap().assigned_slot(&cancel()).unwrap(),
        Some(f.s1)
    );
    assert_eq!(
        s.node(f.x).unwrap().manual_assignment().unwrap(),
        Some(f.s2)
    );
}

#[test]
fn closed_root_hides_standard_getter_but_preserves_canonical_assignment() {
    let f = fixture(HtmlSlotAssignment::Named, true);
    let s = f.dom.snapshot();
    assert_eq!(s.node(f.h1).unwrap().shadow_root(), None);
    assert_eq!(s.node(f.x).unwrap().assigned_slot(&cancel()).unwrap(), None);
    assert_eq!(
        s.node(f.x).unwrap().find_slot(false, &cancel()).unwrap(),
        Some(f.s1)
    );
    assert_eq!(s.slot(f.s1).unwrap().assigned_nodes(), &[f.x]);
}

#[test]
fn slot_name_and_slottable_attribute_hooks_are_namespace_sensitive_and_ordered() {
    let mut f = fixture(HtmlSlotAssignment::Named, false);
    let mut b = DomBatch::new();
    let y = b.create_element_ns(f.document, Namespace::html(), "span");
    b.set_attribute(&y, "slot", "named");
    b.insert_before(&y, f.h1, None);
    let c = apply(&mut f.dom, b);
    let y = c.created(&y).unwrap().unwrap();
    slot_checkpoint(&mut f.dom, queries(), &cancel()).unwrap();
    let mut b = DomBatch::new();
    b.set_attribute_ns(f.s1, Namespace::from_uri("urn:x"), "p:name", "named");
    apply(&mut f.dom, b);
    assert_eq!(
        f.dom.snapshot().slot(f.s1).unwrap().assigned_nodes(),
        &[f.x]
    );
    assert!(f.dom.snapshot().pending_slot_signals().is_empty());
    let mut b = DomBatch::new();
    b.set_attribute(f.s1, "name", "named");
    apply(&mut f.dom, b);
    let s = f.dom.snapshot();
    assert_eq!(s.slot(f.s1).unwrap().assigned_nodes(), &[y]);
    // Removed members' inverse is deliberately historical, not cleared.
    assert_eq!(
        s.node(f.x).unwrap().stored_assigned_slot().unwrap(),
        Some(f.s1)
    );
    let mut b = DomBatch::new();
    b.remove_attribute(f.s1, "name");
    apply(&mut f.dom, b);
    assert_eq!(
        f.dom.snapshot().slot(f.s1).unwrap().assigned_nodes(),
        &[f.x]
    );
    let mut b = DomBatch::new();
    b.remove_attribute(y, "slot");
    apply(&mut f.dom, b);
    assert_eq!(
        f.dom.snapshot().slot(f.s1).unwrap().assigned_nodes(),
        &[f.x, y]
    );
    slot_checkpoint(&mut f.dom, queries(), &cancel()).unwrap();
    let mut b = DomBatch::new();
    b.set_attribute(f.s1, "name", "");
    b.set_attribute(y, "slot", "");
    apply(&mut f.dom, b);
    assert_eq!(f.dom.snapshot().pending_slot_signals(), &[]);
}

#[test]
fn fallback_flattening_filters_comments_while_css_flat_tree_retains_them() {
    let mut f = fixture(HtmlSlotAssignment::Manual, false);
    let mut b = DomBatch::new();
    b.assign_slot(f.s1, Vec::new());
    let comment = b.create_comment(f.document, "fallback comment");
    b.insert_before(&comment, f.s1, None);
    let nested = b.create_element_ns(f.document, Namespace::html(), "slot");
    b.insert_before(&nested, f.s1, None);
    let t = b.create_text(f.document, "");
    b.insert_before(&t, &nested, None);
    let c = apply(&mut f.dom, b);
    let comment = c.created(&comment).unwrap().unwrap();
    let nested = c.created(&nested).unwrap().unwrap();
    let t = c.created(&t).unwrap().unwrap();
    let s = f.dom.snapshot();
    assert_eq!(
        s.slot(f.s1)
            .unwrap()
            .flattened_slottables(queries(), &cancel())
            .unwrap()
            .nodes(),
        &[f.fallback, t]
    );
    let flat = s.flat_tree(f.document, queries(), &cancel()).unwrap();
    assert_eq!(
        flat.relation(comment).unwrap(),
        DomFlatRelation::Parent(f.s1)
    );
    assert_eq!(
        flat.relation(nested).unwrap(),
        DomFlatRelation::Parent(f.s1)
    );
    assert_eq!(flat.relation(t).unwrap(), DomFlatRelation::Parent(nested));
    assert_eq!(flat.relation(f.r1).unwrap(), DomFlatRelation::Excluded);
    assert_eq!(flat.relation(f.r2).unwrap(), DomFlatRelation::Excluded);
    assert_eq!(flat.relation(f.s1).unwrap(), DomFlatRelation::Parent(f.h1));
    assert_eq!(flat.relation(f.top).unwrap(), DomFlatRelation::Root);
    assert_eq!(flat.relation(f.x).unwrap(), DomFlatRelation::Excluded);
    assert_eq!(
        flat.nodes().collect::<Vec<_>>(),
        vec![
            f.top, f.h1, f.h2, f.s1, f.s2, f.fallback, comment, nested, t
        ]
    );
    let mut b = DomBatch::new();
    b.assign_slot(f.s1, vec![f.x.into()]);
    apply(&mut f.dom, b);
    assert_eq!(
        f.dom
            .snapshot()
            .slot(f.s1)
            .unwrap()
            .flattened_slottables(queries(), &cancel())
            .unwrap()
            .nodes(),
        &[f.x]
    );
    assert_eq!(
        flat.relation(f.fallback).unwrap(),
        DomFlatRelation::Parent(f.s1)
    );
}

#[test]
fn detached_self_and_other_document_manual_intent_never_create_flatten_cycles() {
    let mut f = fixture(HtmlSlotAssignment::Manual, false);
    let mut b = DomBatch::new();
    let d = b.create_document();
    let x = b.create_text(&d, "other document");
    b.assign_slot(f.s1, vec![f.s1.into(), (&x).into(), f.s2.into()]);
    let c = apply(&mut f.dom, b);
    let x = c.created(&x).unwrap().unwrap();
    let s = f.dom.snapshot();
    assert_eq!(s.slot(f.s1).unwrap().manual_assignments(), &[f.s1, x, f.s2]);
    assert_eq!(s.slot(f.s1).unwrap().assigned_nodes(), &[]);
    assert_eq!(
        s.slot(f.s1)
            .unwrap()
            .flattened_slottables(queries(), &cancel())
            .unwrap()
            .nodes(),
        &[f.fallback]
    );
    let mut b = DomBatch::new();
    b.detach(f.s1);
    apply(&mut f.dom, b);
    let s = f.dom.snapshot();
    assert_eq!(s.slot(f.s1).unwrap().manual_assignments(), &[f.s1, x, f.s2]);
    assert!(
        s.slot(f.s1)
            .unwrap()
            .flattened_slottables(DomSlotQueryLimits::new(0, 0).unwrap(), &cancel())
            .unwrap()
            .nodes()
            .is_empty()
    );
}

#[test]
fn explicit_retirement_clears_only_exact_stored_inverse_and_pending_recipient() {
    let mut f = fixture(HtmlSlotAssignment::Manual, false);
    let mut b = DomBatch::new();
    b.assign_slot(f.s1, Vec::new());
    b.detach(f.s1);
    apply(&mut f.dom, b);
    let old = f.dom.snapshot();
    assert_eq!(
        old.node(f.x).unwrap().stored_assigned_slot().unwrap(),
        Some(f.s1)
    );
    let mut b = DomBatch::new();
    b.destroy(f.s1);
    let c = apply(&mut f.dom, b);
    let s = f.dom.snapshot();
    assert_eq!(s.node(f.x).unwrap().stored_assigned_slot().unwrap(), None);
    assert!(s.pending_slot_signals().is_empty());
    assert_eq!(s.node(f.s1).unwrap_err(), DomError::StaleNode);
    assert!(
        c.changes()
            .changes()
            .contains(&DomChange::StoredAssignedSlot(f.x))
    );
    assert!(
        c.changes()
            .changes()
            .contains(&DomChange::SlotSignalCancelled {
                slot: f.s1,
                reason: DomSlotSignalCancellationReason::Destroyed
            })
    );
    assert_eq!(
        old.node(f.x).unwrap().stored_assigned_slot().unwrap(),
        Some(f.s1)
    );
    let mut b = DomBatch::new();
    let replacement = b.create_element_ns(f.document, Namespace::html(), "slot");
    let c = apply(&mut f.dom, b);
    assert_ne!(c.created(&replacement).unwrap(), Some(f.s1));
    assert_eq!(
        f.dom
            .snapshot()
            .node(f.x)
            .unwrap()
            .stored_assigned_slot()
            .unwrap(),
        None
    );
}

#[test]
fn checkpoints_are_ordered_atomic_historical_and_revalidate_retired_generations() {
    let mut f = fixture(HtmlSlotAssignment::Manual, false);
    let old = f.dom.snapshot();
    assert_eq!(old.pending_slot_signals(), &[f.s1]);
    let too_small = DomSlotQueryLimits::new(0, 0).unwrap();
    assert!(matches!(
        slot_checkpoint(&mut f.dom, too_small, &cancel()),
        Err(DomError::ResourceLimit {
            resource: DomResource::SlotQueryOutput,
            ..
        })
    ));
    assert_eq!(f.dom.snapshot().stamp(), old.stamp());
    assert!(matches!(
        slot_checkpoint(&mut f.dom, queries(), &AtomicBool::new(true)),
        Err(DomError::Cancelled)
    ));
    assert_eq!(f.dom.snapshot().pending_slot_signals(), &[f.s1]);
    let drained = slot_checkpoint(&mut f.dom, queries(), &cancel()).unwrap();
    assert_eq!(drained.targets(), &[f.s1]);
    assert!(!f.dom.snapshot().slot_notification_queued());
    let stamp = f.dom.snapshot().stamp();
    assert!(matches!(
        f.dom
            .native_checkpoint_step(checkpoint_limits(queries()), &cancel())
            .unwrap(),
        surgeist_dom::DomNativeCheckpoint::Idle
    ));
    assert_eq!(f.dom.snapshot().stamp(), stamp);
    let mut b = DomBatch::new();
    b.detach(f.s1);
    apply(&mut f.dom, b);
    assert_eq!(f.dom.snapshot().pending_slot_signals(), &[f.s1]);
    let mut b = DomBatch::new();
    b.destroy(f.s1);
    apply(&mut f.dom, b);
    assert_eq!(drained.targets(), &[f.s1]);
    assert_eq!(
        drained.revalidate(&f.dom, queries(), &cancel()).unwrap(),
        vec![DomSlotNotificationTarget::SkippedDestroyedTarget(f.s1)]
    );
    assert!(drained.source().node(f.s1).is_ok());
}

#[test]
fn cloned_notification_batches_keep_historical_targets_when_storage_is_reused() {
    let mut f = fixture(HtmlSlotAssignment::Manual, false);
    let drained = slot_checkpoint(&mut f.dom, queries(), &cancel()).unwrap();
    let cloned = drained.clone();
    let historical_stamp = drained.source().stamp();
    let mut b = DomBatch::new();
    b.detach(f.s1);
    b.destroy(f.s1);
    let replacement = b.create_element_ns(f.document, Namespace::html(), "slot");
    b.insert_before(&replacement, f.r1, None);
    b.assign_slot(&replacement, vec![f.x.into()]);
    let c = apply(&mut f.dom, b);
    let replacement = c.created(&replacement).unwrap().unwrap();
    assert_ne!(replacement, f.s1);
    assert!(matches!(
        f.dom.snapshot().node(f.s1),
        Err(DomError::StaleNode)
    ));
    assert!(f.dom.snapshot().slot(replacement).is_ok());
    for historical in [&drained, &cloned] {
        assert_eq!(historical.targets(), &[f.s1]);
        assert_eq!(historical.source().stamp(), historical_stamp);
        assert!(historical.source().slot(f.s1).is_ok());
        assert!(matches!(
            historical.source().node(replacement),
            Err(DomError::StaleNode)
        ));
        assert_eq!(
            historical.revalidate(&f.dom, queries(), &cancel()).unwrap(),
            vec![DomSlotNotificationTarget::SkippedDestroyedTarget(f.s1)]
        );
    }
    assert_eq!(cloned.changes().from(), drained.changes().from());
    assert_eq!(cloned.changes().to(), drained.changes().to());
    assert_eq!(cloned.changes().changes(), drained.changes().changes());
    let next = slot_checkpoint(&mut f.dom, queries(), &cancel()).unwrap();
    assert_eq!(next.targets(), &[replacement]);
    assert_eq!(
        next.revalidate(&f.dom, queries(), &cancel()).unwrap(),
        vec![DomSlotNotificationTarget::Live(replacement)]
    );
    assert_eq!(cloned.targets(), &[f.s1]);
}

#[test]
fn exact_reference_input_and_query_limits_refuse_without_partial_state() {
    let mut f = fixture(HtmlSlotAssignment::Manual, false);
    assert_eq!(f.dom.usage().slot_references(), 3); // manual + stored + queued
    assert!(matches!(
        f.dom.set_limits(limits(2, 128)),
        Err(DomError::ResourceLimit {
            resource: DomResource::SlotReferences,
            ..
        })
    ));
    slot_checkpoint(&mut f.dom, queries(), &cancel()).unwrap();
    assert_eq!(f.dom.usage().slot_references(), 2);
    f.dom.set_limits(limits(2, 2)).unwrap();
    let before = f.dom.snapshot();
    let usage = f.dom.usage();
    let mut b = DomBatch::new();
    b.assign_slot(f.s1, vec![f.x.into(), f.x.into(), f.x.into()]);
    assert_eq!(
        f.dom.apply(b, &cancel()).unwrap_err(),
        DomError::ResourceLimit {
            resource: DomResource::SlotAssignmentInputs,
            limit: 2
        }
    );
    assert_eq!(f.dom.snapshot().stamp(), before.stamp());
    assert_eq!(f.dom.usage(), usage);
    let mut b = DomBatch::new();
    b.assign_slot(f.s1, vec![f.x.into(), f.x.into()]);
    apply(&mut f.dom, b);
    assert_eq!(f.dom.snapshot().stamp(), before.stamp());
    assert!(matches!(
        before
            .slot(f.s1)
            .unwrap()
            .find_slottables(DomSlotQueryLimits::new(0, 0).unwrap(), &cancel()),
        Err(DomError::ResourceLimit {
            resource: DomResource::SlotQueryOutput,
            ..
        })
    ));
    assert!(matches!(
        before
            .slot(f.s1)
            .unwrap()
            .flattened_slottables(DomSlotQueryLimits::new(1, 1).unwrap(), &cancel()),
        Err(DomError::ResourceLimit {
            resource: DomResource::SlotQueryScratch,
            ..
        })
    ));
    assert_eq!(
        before
            .slot(f.s1)
            .unwrap()
            .flattened_slottables(DomSlotQueryLimits::new(1, 2).unwrap(), &cancel())
            .unwrap()
            .nodes(),
        &[f.x]
    );
    let mut b = DomBatch::new();
    b.assign_slot(f.s2, vec![f.x.into()]);
    b.insert_before(f.top, f.x, None);
    assert_eq!(
        f.dom.apply(b, &cancel()).unwrap_err(),
        DomError::HierarchyRequest
    );
    assert_eq!(f.dom.snapshot().stamp(), before.stamp());
    assert_eq!(f.dom.usage(), usage);
}

#[test]
fn import_and_adoption_run_real_slot_hooks_without_copying_source_intent() {
    let mut f = fixture(HtmlSlotAssignment::Manual, false);
    let old = f.dom.snapshot();
    let mut b = DomBatch::new();
    let d = b.create_document();
    let copied = b.import_node(&old, f.h1, &d, false);
    let c = apply(&mut f.dom, b);
    let copied = c.created(&copied).unwrap().unwrap();
    let s = f.dom.snapshot();
    let root = s.node(copied).unwrap().hosted_shadow_root().unwrap();
    let slot = s.node(root).unwrap().children()[0];
    assert!(s.slot(slot).unwrap().manual_assignments().is_empty());
    assert!(s.slot(slot).unwrap().assigned_nodes().is_empty());
    assert_eq!(s.node(slot).unwrap().children().len(), 1); // shallow host, deep clonable root
    let mut b = DomBatch::new();
    b.adopt(f.x, c.document(&d).unwrap());
    apply(&mut f.dom, b);
    let s = f.dom.snapshot();
    assert_eq!(
        s.node(f.x).unwrap().manual_assignment().unwrap(),
        Some(f.s1)
    );
    assert!(s.slot(f.s1).unwrap().assigned_nodes().is_empty());
    assert_eq!(
        s.node(f.x).unwrap().stored_assigned_slot().unwrap(),
        Some(f.s1)
    );
    assert_eq!(old.slot(f.s1).unwrap().assigned_nodes(), &[f.x]);
}

#[test]
fn restoring_assignment_still_commits_a_new_pending_signal_but_data_only_edits_do_not() {
    let mut f = fixture(HtmlSlotAssignment::Manual, false);
    slot_checkpoint(&mut f.dom, queries(), &cancel()).unwrap();
    let before = f.dom.snapshot();
    let mut b = DomBatch::new();
    b.assign_slot(f.s1, Vec::new());
    b.assign_slot(f.s1, vec![f.x.into()]);
    let c = apply(&mut f.dom, b);
    let s = f.dom.snapshot();
    assert_eq!(
        s.slot(f.s1).unwrap().assigned_nodes(),
        before.slot(f.s1).unwrap().assigned_nodes()
    );
    assert_ne!(s.stamp(), before.stamp());
    assert_eq!(s.pending_slot_signals(), &[f.s1]);
    assert!(c.changes().changes().contains(&DomChange::IntrinsicTasks));
    assert!(
        !c.changes()
            .changes()
            .contains(&DomChange::SlotAssignedNodes(f.s1))
    );
    slot_checkpoint(&mut f.dom, queries(), &cancel()).unwrap();
    let mut b = DomBatch::new();
    b.replace_data(f.x, 0, usize::MAX, "changed text");
    apply(&mut f.dom, b);
    assert!(f.dom.snapshot().pending_slot_signals().is_empty());
    assert_eq!(
        f.dom.snapshot().slot(f.s1).unwrap().assigned_nodes(),
        &[f.x]
    );
}

#[test]
fn exact_utf16_names_and_slot_tree_order_select_the_first_match() {
    let mut f = fixture(HtmlSlotAssignment::Named, false);
    let raw = DocumentString::from_utf16(vec![0xd800]);
    let mut b = DomBatch::new();
    let other = b.create_element_ns(f.document, Namespace::html(), "slot");
    let x = b.create_element_ns(f.document, Namespace::html(), "span");
    b.set_attribute(f.s1, "name", raw.clone());
    b.set_attribute(&other, "name", raw.clone());
    b.set_attribute(&x, "slot", raw);
    b.insert_before(&other, f.r1, None);
    b.insert_before(&x, f.h1, None);
    let c = apply(&mut f.dom, b);
    let other = c.created(&other).unwrap().unwrap();
    let x = c.created(&x).unwrap().unwrap();
    assert_eq!(f.dom.snapshot().slot(f.s1).unwrap().assigned_nodes(), &[x]);
    let mut b = DomBatch::new();
    b.insert_before(other, f.r1, Some(f.s1.into()));
    apply(&mut f.dom, b);
    assert_eq!(f.dom.snapshot().slot(other).unwrap().assigned_nodes(), &[x]);
    assert!(
        f.dom
            .snapshot()
            .slot(f.s1)
            .unwrap()
            .assigned_nodes()
            .is_empty()
    );
    let mut b = DomBatch::new();
    b.set_attribute(x, "slot", "\u{fffd}");
    apply(&mut f.dom, b);
    assert!(
        f.dom
            .snapshot()
            .slot(other)
            .unwrap()
            .assigned_nodes()
            .is_empty()
    );
    assert_eq!(
        f.dom
            .snapshot()
            .node(x)
            .unwrap()
            .stored_assigned_slot()
            .unwrap(),
        Some(other)
    );
}

#[test]
fn source_weak_manual_cleanup_and_wrong_foreign_or_stale_operands_are_atomic() {
    let mut f = fixture(HtmlSlotAssignment::Manual, false);
    let foreign = fixture(HtmlSlotAssignment::Manual, false);
    let before = f.dom.snapshot();
    for (node, error) in [
        (foreign.x, DomError::ForeignOwner),
        (f.document.node(), DomError::WrongNodeKind),
    ] {
        let mut b = DomBatch::new();
        b.assign_slot(f.s1, vec![node.into()]);
        assert_eq!(f.dom.apply(b, &cancel()).unwrap_err(), error);
        assert_eq!(f.dom.snapshot().stamp(), before.stamp());
    }
    let mut b = DomBatch::new();
    b.detach(f.x);
    b.destroy(f.x);
    apply(&mut f.dom, b);
    assert!(
        f.dom
            .snapshot()
            .slot(f.s1)
            .unwrap()
            .manual_assignments()
            .is_empty()
    );
    let before = f.dom.snapshot();
    let mut b = DomBatch::new();
    b.assign_slot(f.s1, vec![f.x.into()]);
    assert_eq!(f.dom.apply(b, &cancel()).unwrap_err(), DomError::StaleNode);
    assert_eq!(f.dom.snapshot().stamp(), before.stamp());
}

#[test]
fn fragment_transfer_and_replacement_update_assignment_in_source_order() {
    let mut f = fixture(HtmlSlotAssignment::Named, false);
    let mut b = DomBatch::new();
    let fragment = b.create_fragment(f.document);
    let a = b.create_element_ns(f.document, Namespace::html(), "span");
    let t = b.create_text(f.document, "");
    b.insert_before(&a, &fragment, None);
    b.insert_before(&t, &fragment, None);
    b.insert_before(&fragment, f.h1, Some(f.x.into()));
    let c = apply(&mut f.dom, b);
    let a = c.created(&a).unwrap().unwrap();
    let t = c.created(&t).unwrap().unwrap();
    assert!(
        f.dom
            .snapshot()
            .node(c.created(&fragment).unwrap().unwrap())
            .unwrap()
            .children()
            .is_empty()
    );
    assert_eq!(
        f.dom.snapshot().slot(f.s1).unwrap().assigned_nodes(),
        &[a, t, f.x]
    );
    let mut b = DomBatch::new();
    let replacement = b.create_text(f.document, "replacement");
    b.replace_child(&replacement, f.h1, t);
    let c = apply(&mut f.dom, b);
    let replacement = c.created(&replacement).unwrap().unwrap();
    assert_eq!(
        f.dom.snapshot().slot(f.s1).unwrap().assigned_nodes(),
        &[a, replacement, f.x]
    );
    assert_eq!(f.dom.snapshot().node(t).unwrap().parent(), None);
}

#[test]
fn deep_fallback_is_iterative_fallible_and_keeps_order_without_following_raw_self_intent() {
    let mut dom = Dom::new(DomLimits::new(512, 65536, 1024, 2048, 1024, 0, 0).unwrap()).unwrap();
    let mut b = DomBatch::new();
    let d = b.create_document();
    let host = b.create_element_ns(&d, Namespace::html(), "div");
    b.insert_before(&host, &d, None);
    let root = b.attach_shadow(&host, DomShadowRootInit::new(HtmlShadowRootMode::Open));
    let first = b.create_element_ns(&d, Namespace::html(), "slot");
    b.insert_before(&first, &root, None);
    let mut parent = first.clone();
    for _ in 0..128 {
        let child = b.create_element_ns(&d, Namespace::html(), "slot");
        b.insert_before(&child, &parent, None);
        parent = child;
    }
    let text = b.create_text(&d, "bottom");
    b.insert_before(&text, &parent, None);
    b.assign_slot(&first, vec![(&first).into()]);
    let c = apply(&mut dom, b);
    let first = c.created(&first).unwrap().unwrap();
    let text = c.created(&text).unwrap().unwrap();
    let s = dom.snapshot();
    assert_eq!(
        s.slot(first)
            .unwrap()
            .flattened_slottables(DomSlotQueryLimits::new(1, 258).unwrap(), &cancel())
            .unwrap()
            .nodes(),
        &[text]
    );
    assert!(matches!(
        s.slot(first)
            .unwrap()
            .flattened_slottables(DomSlotQueryLimits::new(1, 257).unwrap(), &cancel()),
        Err(DomError::ResourceLimit {
            resource: DomResource::SlotQueryScratch,
            ..
        })
    ));
    assert!(matches!(
        s.slot(first)
            .unwrap()
            .flattened_slottables(queries(), &AtomicBool::new(true)),
        Err(DomError::Cancelled)
    ));
}

#[test]
fn fallback_signal_capacity_and_late_destroy_failure_preserve_every_published_relation() {
    let mut f = fixture(HtmlSlotAssignment::Manual, false);
    slot_checkpoint(&mut f.dom, queries(), &cancel()).unwrap();
    let mut b = DomBatch::new();
    b.assign_slot(f.s1, Vec::new());
    apply(&mut f.dom, b);
    slot_checkpoint(&mut f.dom, queries(), &cancel()).unwrap();
    assert_eq!(f.dom.usage().slot_references(), 0);
    f.dom.set_limits(limits(0, 128)).unwrap();
    let before = f.dom.snapshot();
    let usage = f.dom.usage();
    let mut b = DomBatch::new();
    let comment = b.create_comment(f.document, "fallback");
    b.insert_before(&comment, f.s1, None);
    assert_eq!(
        f.dom.apply(b, &cancel()).unwrap_err(),
        DomError::ResourceLimit {
            resource: DomResource::SlotReferences,
            limit: 0
        }
    );
    assert_eq!(f.dom.snapshot().stamp(), before.stamp());
    assert_eq!(f.dom.usage(), usage);
    f.dom.set_limits(limits(128, 128)).unwrap();
    let mut b = DomBatch::new();
    b.detach(f.s1);
    apply(&mut f.dom, b);
    let before = f.dom.snapshot();
    let usage = f.dom.usage();
    let mut b = DomBatch::new();
    b.destroy(f.s1);
    b.insert_before(f.h1, f.h1, None);
    assert_eq!(
        f.dom.apply(b, &cancel()).unwrap_err(),
        DomError::HierarchyRequest
    );
    let s = f.dom.snapshot();
    assert_eq!(s.stamp(), before.stamp());
    assert_eq!(f.dom.usage(), usage);
    assert_eq!(s.pending_slot_signals(), before.pending_slot_signals());
    assert_eq!(
        s.node(f.x).unwrap().stored_assigned_slot().unwrap(),
        Some(f.s1)
    );
}

#[test]
fn checkpoint_order_is_first_signal_order_and_a_later_checkpoint_gets_fresh_signals() {
    let mut f = fixture(HtmlSlotAssignment::Manual, false);
    slot_checkpoint(&mut f.dom, queries(), &cancel()).unwrap();
    let mut b = DomBatch::new();
    let a = b.create_comment(f.document, "A");
    b.insert_before(&a, f.s2, None);
    b.assign_slot(f.s1, Vec::new());
    let another = b.create_comment(f.document, "B");
    b.insert_before(&another, f.s2, None);
    apply(&mut f.dom, b);
    let first = slot_checkpoint(&mut f.dom, queries(), &cancel()).unwrap();
    assert_eq!(first.targets(), &[f.s2, f.s1]);
    let mut b = DomBatch::new();
    b.assign_slot(f.s1, vec![f.x.into()]);
    apply(&mut f.dom, b);
    assert_eq!(first.targets(), &[f.s2, f.s1]);
    assert_eq!(
        slot_checkpoint(&mut f.dom, queries(), &cancel())
            .unwrap()
            .targets(),
        &[f.s1]
    );
}

#[test]
fn source_assignments_keep_finite_history_complete_or_explicitly_require_recompute() {
    let mut f = fixture(HtmlSlotAssignment::Manual, false);
    let before = f.dom.snapshot();
    let mut b = DomBatch::new();
    b.assign_slot(f.s2, vec![f.x.into()]);
    let c = apply(&mut f.dom, b);
    assert_eq!(
        f.dom.changes_since(before.stamp()).unwrap().changes(),
        c.changes().changes()
    );
    f.dom
        .set_limits(DomLimits::new(128, 32768, 512, 128, 128, 0, 0).unwrap())
        .unwrap();
    let before = f.dom.snapshot();
    let mut b = DomBatch::new();
    b.assign_slot(f.s1, vec![f.x.into()]);
    apply(&mut f.dom, b);
    assert!(matches!(
        f.dom.changes_since(before.stamp()),
        Err(DomHistoryError::FullRecomputeRequired { .. })
    ));
    assert_eq!(
        f.dom.snapshot().slot(f.s1).unwrap().assigned_nodes(),
        &[f.x]
    );
}

#[test]
fn insertion_into_a_manual_host_does_not_invent_a_stored_assignment_refresh() {
    let mut f = fixture(HtmlSlotAssignment::Manual, false);
    let mut b = DomBatch::new();
    let fresh = b.create_text(f.document, "fresh manual slottable");
    b.assign_slot(f.s1, vec![(&fresh).into()]);
    let c = apply(&mut f.dom, b);
    let fresh = c.created(&fresh).unwrap().unwrap();
    slot_checkpoint(&mut f.dom, queries(), &cancel()).unwrap();
    let mut b = DomBatch::new();
    b.insert_before(fresh, f.h1, None);
    apply(&mut f.dom, b);
    let s = f.dom.snapshot();
    assert_eq!(s.slot(f.s1).unwrap().manual_assignments(), &[fresh]);
    assert_eq!(
        s.node(fresh).unwrap().assigned_slot(&cancel()).unwrap(),
        Some(f.s1)
    );
    assert_eq!(
        s.slot(f.s1)
            .unwrap()
            .find_slottables(queries(), &cancel())
            .unwrap()
            .nodes(),
        &[fresh]
    );
    assert!(s.slot(f.s1).unwrap().assigned_nodes().is_empty());
    assert_eq!(s.node(fresh).unwrap().stored_assigned_slot().unwrap(), None);
    assert!(s.pending_slot_signals().is_empty());
    assert_eq!(
        s.flat_tree(f.document, queries(), &cancel())
            .unwrap()
            .relation(fresh)
            .unwrap(),
        DomFlatRelation::Parent(f.s1)
    );
}

fn checkpoint_limits(slots: DomSlotQueryLimits) -> surgeist_dom::DomNativeCheckpointLimits {
    surgeist_dom::DomNativeCheckpointLimits::new(0, 0, slots).unwrap()
}
fn slot_checkpoint(
    dom: &mut Dom,
    slots: DomSlotQueryLimits,
    cancel: &AtomicBool,
) -> Result<surgeist_dom::DomSlotNotifications, DomError> {
    match dom.native_checkpoint_step(checkpoint_limits(slots), cancel)? {
        surgeist_dom::DomNativeCheckpoint::Slots(n) => Ok(n),
        other => panic!("expected reached Slot marker, got {other:?}"),
    }
}

#[test]
fn owned_subtree_retirement_cancels_multiple_slots_without_reordering_surviving_recipients() {
    let mut f = fixture(HtmlSlotAssignment::Manual, false);
    let mut b = DomBatch::new();
    let victim = b.create_element_ns(f.document, Namespace::html(), "div");
    b.insert_before(&victim, f.top, None);
    let root = b.attach_shadow(&victim, DomShadowRootInit::new(HtmlShadowRootMode::Open));
    let a = b.create_element_ns(f.document, Namespace::html(), "slot");
    let z = b.create_element_ns(f.document, Namespace::html(), "slot");
    b.insert_before(&a, &root, None);
    b.insert_before(&z, &root, None);
    let c = apply(&mut f.dom, b);
    let victim = c.created(&victim).unwrap().unwrap();
    let root = c.created(&root).unwrap().unwrap();
    let a = c.created(&a).unwrap().unwrap();
    let z = c.created(&z).unwrap().unwrap();
    slot_checkpoint(&mut f.dom, queries(), &cancel()).unwrap();

    // Actual fallback insertions signal empty slots; actual reassignment signals
    // S1. First-signal order deliberately differs from tree/destruction order.
    let mut b = DomBatch::new();
    let live_fallback = b.create_comment(f.document, "live");
    b.insert_before(&live_fallback, f.s2, None);
    let z_fallback = b.create_comment(f.document, "Z");
    b.insert_before(&z_fallback, z, None);
    b.assign_slot(f.s1, Vec::new());
    let a_fallback = b.create_comment(f.document, "A");
    b.insert_before(&a_fallback, a, None);
    let duplicate = b.create_comment(f.document, "second Z signal");
    b.insert_before(&duplicate, z, None);
    let c = apply(&mut f.dom, b);
    let z_fallback = c.created(&z_fallback).unwrap().unwrap();
    let a_fallback = c.created(&a_fallback).unwrap().unwrap();
    let duplicate = c.created(&duplicate).unwrap().unwrap();
    let old = f.dom.snapshot();
    assert_eq!(old.pending_slot_signals(), &[f.s2, z, f.s1, a]);

    let mut b = DomBatch::new();
    b.detach(victim);
    b.destroy(victim);
    let c = apply(&mut f.dom, b);
    let now = f.dom.snapshot();
    assert_eq!(now.pending_slot_signals(), &[f.s2, f.s1]);
    for slot in [a, z] {
        assert!(
            c.changes()
                .changes()
                .contains(&DomChange::SlotSignalCancelled {
                    slot,
                    reason: DomSlotSignalCancellationReason::Destroyed,
                })
        );
    }
    for slot in [f.s2, f.s1] {
        assert!(
            !c.changes()
                .changes()
                .contains(&DomChange::SlotSignalCancelled {
                    slot,
                    reason: DomSlotSignalCancellationReason::Destroyed,
                })
        );
        assert!(now.slot(slot).is_ok());
    }
    for id in [victim, root, a, z, a_fallback, z_fallback, duplicate] {
        assert_eq!(now.node(id).unwrap_err(), DomError::StaleNode);
        assert!(old.node(id).is_ok());
    }
    assert_eq!(old.pending_slot_signals(), &[f.s2, z, f.s1, a]);
    // Cancellation is not delivery: the next genuine checkpoint transfers only
    // the surviving intentions, in their original relative order.
    let next = slot_checkpoint(&mut f.dom, queries(), &cancel()).unwrap();
    assert_eq!(next.targets(), &[f.s2, f.s1]);
    assert_eq!(old.pending_slot_signals(), &[f.s2, z, f.s1, a]);
}

#[test]
fn subtree_slot_retirement_clears_multiple_empty_list_inverses_but_preserves_live_target() {
    let mut f = fixture(HtmlSlotAssignment::Named, false);
    let mut b = DomBatch::new();
    let wrapper = b.create_element_ns(f.document, Namespace::html(), "section");
    let a = b.create_element_ns(f.document, Namespace::html(), "slot");
    let z = b.create_element_ns(f.document, Namespace::html(), "slot");
    b.set_attribute(&a, "name", "a");
    b.set_attribute(&z, "name", "z");
    b.insert_before(&a, &wrapper, None);
    b.insert_before(&z, &wrapper, None);
    b.insert_before(&wrapper, f.r1, None);
    let first = b.create_element_ns(f.document, Namespace::html(), "span");
    let second = b.create_element_ns(f.document, Namespace::html(), "span");
    let third = b.create_element_ns(f.document, Namespace::html(), "span");
    for (node, name) in [(&first, "a"), (&second, "a"), (&third, "z")] {
        b.set_attribute(node, "slot", name);
        b.insert_before(node, f.h1, None);
    }
    let c = apply(&mut f.dom, b);
    let wrapper = c.created(&wrapper).unwrap().unwrap();
    let a = c.created(&a).unwrap().unwrap();
    let z = c.created(&z).unwrap().unwrap();
    let first = c.created(&first).unwrap().unwrap();
    let second = c.created(&second).unwrap().unwrap();
    let third = c.created(&third).unwrap().unwrap();
    let assigned = f.dom.snapshot();
    assert_eq!(assigned.slot(a).unwrap().assigned_nodes(), &[first, second]);
    assert_eq!(assigned.slot(z).unwrap().assigned_nodes(), &[third]);
    let mut b = DomBatch::new();
    b.set_attribute(a, "name", "no-a-match");
    b.set_attribute(z, "name", "no-z-match");
    apply(&mut f.dom, b);
    let mut b = DomBatch::new();
    b.detach(wrapper);
    apply(&mut f.dom, b);
    let old = f.dom.snapshot();
    for slot in [a, z] {
        assert!(old.slot(slot).unwrap().assigned_nodes().is_empty());
        assert!(old.pending_slot_signals().contains(&slot));
    }
    for (node, slot) in [(first, a), (second, a), (third, z), (f.x, f.s1)] {
        assert_eq!(
            old.node(node).unwrap().stored_assigned_slot().unwrap(),
            Some(slot)
        );
        assert_eq!(old.node(node).unwrap().parent(), Some(f.h1));
    }

    // #1237 requires finding surviving inverses independently of the now-empty
    // stored lists, for every exact Slot retired inside the owned subtree.
    let mut b = DomBatch::new();
    b.destroy(wrapper);
    let c = apply(&mut f.dom, b);
    let now = f.dom.snapshot();
    for node in [first, second, third] {
        assert_eq!(
            now.node(node).unwrap().stored_assigned_slot().unwrap(),
            None
        );
        assert!(
            c.changes()
                .changes()
                .contains(&DomChange::StoredAssignedSlot(node))
        );
    }
    assert_eq!(
        now.node(f.x).unwrap().stored_assigned_slot().unwrap(),
        Some(f.s1)
    );
    assert!(
        !c.changes()
            .changes()
            .contains(&DomChange::StoredAssignedSlot(f.x))
    );
    assert_eq!(now.slot(f.s1).unwrap().assigned_nodes(), &[f.x]);
    for slot in [a, z] {
        assert_eq!(now.node(slot).unwrap_err(), DomError::StaleNode);
        assert!(
            c.changes()
                .changes()
                .contains(&DomChange::SlotSignalCancelled {
                    slot,
                    reason: DomSlotSignalCancellationReason::Destroyed,
                })
        );
        assert!(old.slot(slot).unwrap().assigned_nodes().is_empty());
    }
    for (node, slot) in [(first, a), (second, a), (third, z)] {
        assert_eq!(
            old.node(node).unwrap().stored_assigned_slot().unwrap(),
            Some(slot)
        );
    }
    assert_eq!(assigned.slot(a).unwrap().assigned_nodes(), &[first, second]);
    assert_eq!(assigned.slot(z).unwrap().assigned_nodes(), &[third]);

    // New allocations cannot revive either retired full identity or acquire its
    // cleared inverses. Public NodeIds intentionally expose no arena-slot index.
    let mut b = DomBatch::new();
    let replacement_a = b.create_element_ns(f.document, Namespace::html(), "slot");
    let replacement_z = b.create_element_ns(f.document, Namespace::html(), "slot");
    let c = apply(&mut f.dom, b);
    let replacement_a = c.created(&replacement_a).unwrap().unwrap();
    let replacement_z = c.created(&replacement_z).unwrap().unwrap();
    for replacement in [replacement_a, replacement_z] {
        assert_ne!(replacement, a);
        assert_ne!(replacement, z);
        assert!(
            f.dom
                .snapshot()
                .slot(replacement)
                .unwrap()
                .assigned_nodes()
                .is_empty()
        );
    }
    for node in [first, second, third] {
        assert_eq!(
            f.dom
                .snapshot()
                .node(node)
                .unwrap()
                .stored_assigned_slot()
                .unwrap(),
            None
        );
    }
    let mut b = DomBatch::new();
    for node in [first, second, third] {
        b.set_attribute(node, "slot", "");
    }
    apply(&mut f.dom, b);
    let mutated = f.dom.snapshot();
    assert_eq!(
        mutated.slot(f.s1).unwrap().assigned_nodes(),
        &[f.x, first, second, third]
    );
    for node in [first, second, third] {
        assert_eq!(
            mutated.node(node).unwrap().stored_assigned_slot().unwrap(),
            Some(f.s1)
        );
        assert_eq!(old.node(node).unwrap().parent(), Some(f.h1));
    }
    for slot in [a, z] {
        assert_eq!(mutated.node(slot).unwrap_err(), DomError::StaleNode);
    }
}

#[test]
fn authored_display_none_and_contents_preserve_canonical_shadow_slot_and_flat_relations() {
    let mut f = fixture(HtmlSlotAssignment::Manual, false);
    let initial = f.dom.snapshot();
    let flat = initial.flat_tree(f.document, queries(), &cancel()).unwrap();
    let mut captures = vec![(initial, flat)];
    // #1236's DOM-only invariant: authored display input does not rewrite the
    // canonical tree. This does not parse CSS or certify #172 box suppression.
    for value in ["display:none", "display:contents"] {
        let before = f.dom.snapshot().stamp();
        let mut b = DomBatch::new();
        b.set_attribute(f.h1, "style", value);
        b.set_attribute(f.s1, "style", value);
        apply(&mut f.dom, b);
        let snapshot = f.dom.snapshot();
        assert_ne!(snapshot.stamp(), before);
        let flat = snapshot
            .flat_tree(f.document, queries(), &cancel())
            .unwrap();
        assert_eq!(flat.source().stamp(), snapshot.stamp());
        assert_eq!(
            flat.source().document(f.document).unwrap().stamp(),
            snapshot.document(f.document).unwrap().stamp()
        );
        captures.push((snapshot, flat));
    }
    for ((snapshot, flat), style) in
        captures
            .iter()
            .zip([None, Some("display:none"), Some("display:contents")])
    {
        for node in [f.h1, f.s1] {
            assert_eq!(
                snapshot
                    .node(node)
                    .unwrap()
                    .attribute(&"style".into())
                    .cloned(),
                style.map(DocumentString::from)
            );
        }
        assert_eq!(
            snapshot.node(f.top).unwrap().parent(),
            Some(f.document.node())
        );
        assert_eq!(snapshot.node(f.h1).unwrap().parent(), Some(f.top));
        assert_eq!(snapshot.node(f.h1).unwrap().children(), &[f.x]);
        assert_eq!(snapshot.node(f.x).unwrap().parent(), Some(f.h1));
        assert_eq!(snapshot.node(f.r1).unwrap().parent(), None);
        assert_eq!(snapshot.node(f.r1).unwrap().children(), &[f.s1]);
        assert_eq!(snapshot.node(f.s1).unwrap().parent(), Some(f.r1));
        assert_eq!(snapshot.node(f.s1).unwrap().children(), &[f.fallback]);
        assert_eq!(
            snapshot.node(f.h1).unwrap().hosted_shadow_root(),
            Some(f.r1)
        );
        assert_eq!(snapshot.node(f.r1).unwrap().shadow_host(), Some(f.h1));
        for node in [f.top, f.h1, f.r1, f.s1, f.x, f.fallback] {
            assert_eq!(
                snapshot.node(node).unwrap().owner_document(),
                Some(f.document)
            );
        }
        assert_eq!(snapshot.slot(f.s1).unwrap().manual_assignments(), &[f.x]);
        assert_eq!(snapshot.slot(f.s1).unwrap().assigned_nodes(), &[f.x]);
        assert_eq!(
            snapshot.node(f.x).unwrap().stored_assigned_slot().unwrap(),
            Some(f.s1)
        );
        assert_eq!(
            snapshot
                .node(f.x)
                .unwrap()
                .assigned_slot(&cancel())
                .unwrap(),
            Some(f.s1)
        );
        assert_eq!(flat.document(), f.document);
        for (node, relation) in [
            (f.top, DomFlatRelation::Root),
            (f.h1, DomFlatRelation::Parent(f.top)),
            (f.s1, DomFlatRelation::Parent(f.h1)),
            (f.x, DomFlatRelation::Parent(f.s1)),
            (f.r1, DomFlatRelation::Excluded),
            (f.fallback, DomFlatRelation::Excluded),
        ] {
            assert_eq!(flat.relation(node).unwrap(), relation);
        }
    }
    assert_ne!(
        captures[0].1.source().stamp(),
        captures[1].1.source().stamp()
    );
    assert_ne!(
        captures[1].1.source().stamp(),
        captures[2].1.source().stamp()
    );
}
