//! Real public Source edits with fixed named-assignment participants. Candidate
//! discovery is measured separately from required assignment and output work.
use super::discovery_observation::{Counts, Observation};
use crate::{
    DocumentId, Dom, DomBatch, DomChanges, DomLimits, DomNativeCheckpoint,
    DomNativeCheckpointLimits, DomSlotQueryLimits, DomSourceEdit, DomSourceLimits,
    DomSourceProgress, HtmlShadowRootMode, HtmlShadowRootProperties, HtmlSlotAssignment, Namespace,
    NodeId,
};
use std::sync::atomic::AtomicBool;

const NAME_UNITS: usize = 5;
const EXTENTS: [usize; 3] = [0, 32, 1024];
const SOURCE_WORK: u64 = 2_000_000;

// Three complete qualified-key/name paths allow descent, representative
// validation and ordered-head discovery. A UTF-16 unit includes its continuation
// bit; the final group terminator is explicit. This deliberately allows bounded
// index probes on a miss and does not require a particular trie shape. All
// authored/query names in the growing cases have exactly five code units.
const QUALIFIED_BITS: usize = 1 + 64 + usize::BITS as usize + 64;
const DISCOVERY_PATH_BOUND: usize = 3 * (QUALIFIED_BITS + 17 * (NAME_UNITS + 1));

fn cancelled() -> AtomicBool {
    AtomicBool::new(false)
}

struct Fixture {
    dom: Dom,
    document: DocumentId,
    host: NodeId,
    shadow: NodeId,
    slots: Vec<NodeId>,
    unrelated_slots: Vec<NodeId>,
    child: NodeId,
    unrelated_children: Vec<NodeId>,
}

struct Layout<'a> {
    unrelated_slots: usize,
    names: &'a [Option<&'a str>],
    unrelated_children: usize,
    child_name: Option<&'a str>,
    attach_child: bool,
    text_child: bool,
}

fn fixture(layout: Layout<'_>) -> Fixture {
    let mut dom = Dom::new(DomLimits::new(4096, 1_000_000, 16_384, 64, 1, 0, 0).unwrap()).unwrap();
    let mut batch = DomBatch::new();
    let document = batch.create_document();
    let host = batch.create_element_ns(&document, Namespace::html(), "div");
    batch.insert_before(&host, &document, None);
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
    let mut unrelated_slots = Vec::new();
    for index in 0..layout.unrelated_slots {
        let slot = batch.create_element_ns(&document, Namespace::html(), "slot");
        let name = format!("u{index:04}");
        assert_eq!(name.encode_utf16().count(), NAME_UNITS);
        batch.set_attribute(&slot, "name", name);
        batch.insert_before(&slot, &shadow, None);
        unrelated_slots.push(slot);
    }
    let mut slots = Vec::new();
    for name in layout.names {
        let slot = batch.create_element_ns(&document, Namespace::html(), "slot");
        if let Some(name) = name {
            batch.set_attribute(&slot, "name", *name);
        }
        batch.insert_before(&slot, &shadow, None);
        slots.push(slot);
    }
    let child = if layout.text_child {
        batch.create_text(&document, "fixed")
    } else {
        let child = batch.create_element_ns(&document, Namespace::html(), "span");
        if let Some(name) = layout.child_name {
            batch.set_attribute(&child, "slot", name);
        }
        child
    };
    if layout.attach_child {
        batch.insert_before(&child, &host, None);
    }
    let mut unrelated_children = Vec::new();
    for _ in 0..layout.unrelated_children {
        let child = batch.create_element_ns(&document, Namespace::html(), "span");
        batch.set_attribute(&child, "slot", "other");
        batch.insert_before(&child, &host, None);
        unrelated_children.push(child);
    }
    let commit = dom.apply(batch, &cancelled()).unwrap();
    let id = |ticket: &crate::CreatedNode| commit.created(ticket).unwrap().unwrap();
    Fixture {
        document: commit.document(&document).unwrap(),
        host: id(&host),
        shadow: id(&shadow),
        slots: slots.iter().map(id).collect(),
        unrelated_slots: unrelated_slots.iter().map(id).collect(),
        child: id(&child),
        unrelated_children: unrelated_children.iter().map(id).collect(),
        dom,
    }
}

fn source(dom: &mut Dom, edit: DomSourceEdit) -> DomChanges {
    // Storage/work policy is finite and identical for all extents. The
    // independent candidate-read bound below, rather than this generous whole
    // operation budget, is the locality oracle.
    let policy = DomSourceLimits::new(4_000_000_000, 10_000_000, SOURCE_WORK, 8, 4096, 4096, 1);
    let mut lease = dom.begin_source_edit(edit, policy, &cancelled()).unwrap();
    let changes = match lease.advance(&cancelled()).unwrap() {
        DomSourceProgress::Complete(done) => {
            assert!(!done.cancellation_requested());
            done.commit().changes().clone()
        }
        DomSourceProgress::Request(request) => {
            panic!("named Slot assignment has no external service: {request:?}")
        }
    };
    drop(lease);
    changes
}

fn observed_source(dom: &mut Dom, edit: DomSourceEdit) -> (DomChanges, Counts) {
    let observation = Observation::begin();
    let changes = source(dom, edit);
    (changes, observation.finish())
}

fn drain_signals(dom: &mut Dom) {
    let slots = DomSlotQueryLimits::new(64, 4096).unwrap();
    let limits = DomNativeCheckpointLimits::new(0, 0, slots).unwrap();
    assert!(matches!(
        dom.native_checkpoint_step(limits, &cancelled()).unwrap(),
        DomNativeCheckpoint::Slots(_)
    ));
    assert!(dom.snapshot().pending_slot_signals().is_empty());
}

#[test]
#[ignore = "requires SURGEIST_DOM_STORAGE_COUNTERS=1; select explicitly"]
fn source_first_named_slot_ignores_unrelated_slot_names_and_keeps_first_duplicate() {
    assert!(
        EXTENTS
            .iter()
            .any(|extent| *extent > DISCOVERY_PATH_BOUND + 2)
    );
    for unrelated in EXTENTS {
        let mut f = fixture(Layout {
            unrelated_slots: unrelated,
            names: &[Some("match"), Some("match")],
            unrelated_children: 0,
            child_name: Some("match"),
            attach_child: false,
            text_child: false,
        });
        let old = f.dom.snapshot();
        let shadow_children = old.node(f.shadow).unwrap().children().to_vec();
        let (_, discovery) = observed_source(
            &mut f.dom,
            DomSourceEdit::insert_before(f.child, f.host, None),
        );
        let current = f.dom.snapshot();
        assert!(old.node(f.host).unwrap().children().is_empty());
        assert_eq!(old.node(f.child).unwrap().parent(), None);
        assert_eq!(current.node(f.host).unwrap().children(), &[f.child]);
        assert_eq!(current.node(f.child).unwrap().parent(), Some(f.host));
        assert_eq!(
            current.node(f.child).unwrap().owner_document(),
            Some(f.document)
        );
        assert_eq!(current.node(f.shadow).unwrap().children(), shadow_children);
        assert_eq!(
            current.slot(f.slots[0]).unwrap().assigned_nodes(),
            &[f.child]
        );
        assert!(
            current
                .slot(f.slots[1])
                .unwrap()
                .assigned_nodes()
                .is_empty()
        );
        assert_eq!(
            current
                .node(f.child)
                .unwrap()
                .find_slot(false, &cancelled())
                .unwrap(),
            Some(f.slots[0])
        );
        assert_eq!(current.pending_slot_signals(), &[f.slots[0]]);
        for slot in f.unrelated_slots {
            assert!(current.slot(slot).unwrap().assigned_nodes().is_empty());
        }
        assert!(discovery.named_lookups > 0 && discovery.named_candidates > 0);
        assert!(
            discovery.max_named_candidates <= DISCOVERY_PATH_BOUND + 2,
            "unrelated Slot names={unrelated}, actual discovery={discovery:?}"
        );
    }
}

#[test]
#[ignore = "requires SURGEIST_DOM_STORAGE_COUNTERS=1; select explicitly"]
fn source_missing_slot_name_does_not_search_unrelated_named_slots() {
    for unrelated in EXTENTS {
        let mut f = fixture(Layout {
            unrelated_slots: unrelated,
            names: &[],
            unrelated_children: 0,
            child_name: Some("absnt"),
            attach_child: false,
            text_child: false,
        });
        let old = f.dom.snapshot();
        let shadow_children = old.node(f.shadow).unwrap().children().to_vec();
        let (_, discovery) = observed_source(
            &mut f.dom,
            DomSourceEdit::insert_before(f.child, f.host, None),
        );
        let current = f.dom.snapshot();
        assert!(old.node(f.host).unwrap().children().is_empty());
        assert_eq!(current.node(f.host).unwrap().children(), &[f.child]);
        assert_eq!(current.node(f.shadow).unwrap().children(), shadow_children);
        assert_eq!(
            current
                .node(f.child)
                .unwrap()
                .stored_assigned_slot()
                .unwrap(),
            None
        );
        assert_eq!(
            current
                .node(f.child)
                .unwrap()
                .find_slot(false, &cancelled())
                .unwrap(),
            None
        );
        assert!(current.pending_slot_signals().is_empty());
        assert!(discovery.named_lookups > 0);
        assert!(
            discovery.max_named_candidates <= DISCOVERY_PATH_BOUND,
            "absent name, unrelated Slots={unrelated}, actual discovery={discovery:?}"
        );
    }
}

#[test]
#[ignore = "requires SURGEIST_DOM_STORAGE_COUNTERS=1; select explicitly"]
fn source_named_assignment_discovers_only_the_matching_host_slottables() {
    for unrelated in EXTENTS {
        let mut f = fixture(Layout {
            unrelated_slots: 0,
            names: &[Some("match")],
            unrelated_children: unrelated,
            child_name: Some("match"),
            attach_child: true,
            text_child: false,
        });
        drain_signals(&mut f.dom);
        let old = f.dom.snapshot();
        let host_children = old.node(f.host).unwrap().children().to_vec();
        let (_, discovery) = observed_source(
            &mut f.dom,
            DomSourceEdit::assign_slot(f.slots[0], vec![f.child]),
        );
        let current = f.dom.snapshot();
        assert!(
            old.slot(f.slots[0])
                .unwrap()
                .manual_assignments()
                .is_empty()
        );
        assert_eq!(old.slot(f.slots[0]).unwrap().assigned_nodes(), &[f.child]);
        assert_eq!(current.node(f.host).unwrap().children(), host_children);
        assert_eq!(
            current.slot(f.slots[0]).unwrap().manual_assignments(),
            &[f.child]
        );
        assert_eq!(
            current.slot(f.slots[0]).unwrap().assigned_nodes(),
            &[f.child]
        );
        assert_eq!(
            current.node(f.child).unwrap().manual_assignment().unwrap(),
            Some(f.slots[0])
        );
        assert!(current.pending_slot_signals().is_empty());
        for child in f.unrelated_children {
            assert_eq!(
                current.node(child).unwrap().stored_assigned_slot().unwrap(),
                None
            );
        }
        assert!(discovery.host_lookups > 0 && discovery.host_candidates > 0);
        assert!(
            discovery.max_host_candidates <= DISCOVERY_PATH_BOUND + 1,
            "fixed assignment, unrelated host children={unrelated}, actual discovery={discovery:?}"
        );
    }
}

#[test]
#[ignore = "requires SURGEIST_DOM_STORAGE_COUNTERS=1; select explicitly"]
fn source_absent_named_assignment_is_a_noop_without_searching_host_children() {
    for unrelated in EXTENTS {
        let mut f = fixture(Layout {
            unrelated_slots: 0,
            names: &[Some("absnt")],
            unrelated_children: unrelated,
            child_name: Some("match"),
            attach_child: false,
            text_child: false,
        });
        let old = f.dom.snapshot();
        let usage = f.dom.usage();
        let children = old.node(f.host).unwrap().children().to_vec();
        let (changes, discovery) =
            observed_source(&mut f.dom, DomSourceEdit::assign_slot(f.slots[0], vec![]));
        let current = f.dom.snapshot();
        assert_eq!(current.stamp(), old.stamp());
        assert_eq!(f.dom.usage(), usage);
        assert!(changes.changes().is_empty());
        assert_eq!(current.node(f.host).unwrap().children(), children);
        assert!(
            current
                .slot(f.slots[0])
                .unwrap()
                .manual_assignments()
                .is_empty()
        );
        assert!(
            current
                .slot(f.slots[0])
                .unwrap()
                .assigned_nodes()
                .is_empty()
        );
        assert!(current.pending_slot_signals().is_empty());
        assert!(discovery.host_lookups > 0);
        assert!(
            discovery.max_host_candidates <= DISCOVERY_PATH_BOUND,
            "no matching slottables, unrelated host children={unrelated}, actual discovery={discovery:?}"
        );
    }
}

#[test]
fn source_default_names_share_empty_group_and_duplicates_follow_tree_order() {
    let mut f = fixture(Layout {
        unrelated_slots: 0,
        names: &[None, Some("")],
        unrelated_children: 0,
        child_name: None,
        attach_child: false,
        text_child: true,
    });
    let old = f.dom.snapshot();
    source(
        &mut f.dom,
        DomSourceEdit::insert_before(f.child, f.host, None),
    );
    let first = f.dom.snapshot();
    assert!(old.slot(f.slots[0]).unwrap().assigned_nodes().is_empty());
    assert_eq!(first.slot(f.slots[0]).unwrap().assigned_nodes(), &[f.child]);
    assert!(first.slot(f.slots[1]).unwrap().assigned_nodes().is_empty());

    let mut batch = DomBatch::new();
    let empty = batch.create_element_ns(f.document, Namespace::html(), "span");
    batch.set_attribute(&empty, "slot", "");
    let commit = f.dom.apply(batch, &cancelled()).unwrap();
    let empty = commit.created(&empty).unwrap().unwrap();
    source(
        &mut f.dom,
        DomSourceEdit::insert_before(empty, f.host, None),
    );
    let second = f.dom.snapshot();
    assert_eq!(
        second.slot(f.slots[0]).unwrap().assigned_nodes(),
        &[f.child, empty]
    );
    assert!(second.slot(f.slots[1]).unwrap().assigned_nodes().is_empty());

    let mut batch = DomBatch::new();
    batch.insert_before(f.slots[0], f.shadow, None);
    f.dom.apply(batch, &cancelled()).unwrap();
    drain_signals(&mut f.dom);
    source(
        &mut f.dom,
        DomSourceEdit::assign_slot(f.slots[0], vec![f.child]),
    );
    let current = f.dom.snapshot();
    assert_eq!(
        current.node(f.shadow).unwrap().children(),
        &[f.slots[1], f.slots[0]]
    );
    assert!(
        current
            .slot(f.slots[0])
            .unwrap()
            .assigned_nodes()
            .is_empty()
    );
    assert_eq!(
        current.slot(f.slots[1]).unwrap().assigned_nodes(),
        &[f.child, empty]
    );
    assert_eq!(
        current.slot(f.slots[0]).unwrap().manual_assignments(),
        &[f.child]
    );
    assert_eq!(
        current
            .node(f.child)
            .unwrap()
            .find_slot(false, &cancelled())
            .unwrap(),
        Some(f.slots[1])
    );
    assert_eq!(
        current
            .node(empty)
            .unwrap()
            .find_slot(false, &cancelled())
            .unwrap(),
        Some(f.slots[1])
    );
    assert_eq!(
        second.slot(f.slots[0]).unwrap().assigned_nodes(),
        &[f.child, empty]
    );
    assert_eq!(
        second.node(f.shadow).unwrap().children(),
        &[f.slots[0], f.slots[1]]
    );
}
