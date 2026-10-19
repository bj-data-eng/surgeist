//! DOM3071 boundary setters, replace-data, pre-remove, insert and adopt oracles;
//! #1243 supplies native association and explicit identity-retirement policy.
use std::sync::Arc;
use std::sync::atomic::AtomicBool;
use surgeist_dom::{
    DocumentId, DocumentString, Dom, DomActiveRangeState, DomBatch, DomChange, DomCommit, DomError,
    DomLimits, DomRangeId, DomResource, DomShadowRootInit, DomSnapshot, DomSourceEdit,
    DomSourceLimits, DomSourceProgress, DomSourceRequest, HtmlShadowRootMode, Namespace, NodeId,
};
use surgeist_html::HtmlStyleUpdateAction;

fn limits(ranges: usize) -> DomLimits {
    DomLimits::new(
        256,
        65_536,
        512,
        0,
        0,
        64,
        1_000_000,
        surgeist_dom::DomNameLimits::new(16_777_216, 16_384, 4_000_000, 1_048_576).unwrap(),
    )
    .unwrap()
    .with_ranges(ranges)
    .unwrap()
}
fn dom() -> Dom {
    Dom::new(limits(64)).unwrap()
}
fn apply(dom: &mut Dom, batch: DomBatch) -> DomCommit {
    dom.apply(batch, &AtomicBool::new(false)).unwrap()
}
fn bounds(snapshot: &DomSnapshot, id: DomRangeId) -> ((NodeId, usize), (NodeId, usize)) {
    let range = snapshot.range(id).unwrap();
    assert_eq!(range.id(), id);
    (
        (range.start().container(), range.start().offset()),
        (range.end().container(), range.end().offset()),
    )
}
fn active(snapshot: &DomSnapshot, document: DocumentId) -> Option<DomRangeId> {
    match snapshot.document(document).unwrap().active_range() {
        DomActiveRangeState::Known(value) => value,
        DomActiveRangeState::Unavailable => panic!("native constructor has actual selection state"),
    }
}
fn at(
    dom: &mut Dom,
    document: DocumentId,
    start: (NodeId, usize),
    end: (NodeId, usize),
) -> DomRangeId {
    let mut batch = DomBatch::new();
    let ticket = batch.create_range(document);
    batch.set_range_start(&ticket, start.0, start.1);
    batch.set_range_end(&ticket, end.0, end.1);
    apply(dom, batch).created_range(&ticket).unwrap().unwrap()
}
fn document_and_parent(dom: &mut Dom) -> (DocumentId, NodeId) {
    let mut batch = DomBatch::new();
    let document = batch.create_document();
    let parent = batch.create_element_ns(&document, Namespace::html(), "div");
    batch.insert_before(&parent, &document, None);
    let commit = apply(dom, batch);
    (
        commit.document(&document).unwrap(),
        commit.created(&parent).unwrap().unwrap(),
    )
}
fn associate(dom: &mut Dom, document: DocumentId, id: DomRangeId) {
    let mut batch = DomBatch::new();
    batch.set_active_range(document, Some(id.into()));
    apply(dom, batch);
}

#[test]
fn native_creation_publishes_document_zero_and_complete_change_evidence_without_implicit_association()
 {
    let mut dom = dom();
    let (document, parent) = document_and_parent(&mut dom);
    let before = dom.snapshot();
    assert_eq!(active(&before, document), None);
    let mut batch = DomBatch::new();
    let ticket = batch.create_range(document);
    let commit = apply(&mut dom, batch);
    let id = commit.created_range(&ticket).unwrap().unwrap();
    assert_eq!(
        bounds(&dom.snapshot(), id),
        ((document.node(), 0), (document.node(), 0))
    );
    assert!(dom.snapshot().range(id).unwrap().collapsed());
    assert_eq!(commit.changes().changes(), [DomChange::RangeCreated(id)]);
    assert_eq!(active(&dom.snapshot(), document), None);
    assert_eq!(dom.usage().ranges(), 1);
    let mut batch = DomBatch::new();
    batch.set_range_start(id, parent, 0);
    batch.set_active_range(document, Some(id.into()));
    let commit = apply(&mut dom, batch);
    assert_eq!(commit.changes().changes().len(), 2);
    assert!(
        commit
            .changes()
            .changes()
            .contains(&DomChange::RangeBoundary(id))
    );
    assert!(
        commit
            .changes()
            .changes()
            .contains(&DomChange::DocumentActiveRange(document))
    );
    let mut batch = DomBatch::new();
    batch.set_active_range(document, None);
    apply(&mut dom, batch);
    assert_eq!(dom.usage().ranges(), 1);
    assert_eq!(bounds(&dom.snapshot(), id), ((parent, 0), (parent, 0)));
    assert_eq!(active(&before, document), None);
    let mut producer = DomBatch::new();
    let foreign_ticket = producer.create_range(document);
    let before = dom.snapshot();
    let mut wrong = DomBatch::new();
    wrong.set_range_start(&foreign_ticket, parent, 0);
    assert_eq!(
        dom.apply(wrong, &AtomicBool::new(false)).unwrap_err(),
        DomError::ForeignBatch
    );
    assert_eq!(dom.snapshot().stamp(), before.stamp());
    assert_eq!(
        commit.created_range(&foreign_ticket).unwrap_err(),
        DomError::ForeignBatch
    );
    assert!(
        apply(&mut dom, producer)
            .created_range(&foreign_ticket)
            .unwrap()
            .is_some()
    );
}

#[test]
fn character_data_maintenance_uses_utf16_interval_thresholds_for_both_endpoints() {
    let mut dom = dom();
    let (document, parent) = document_and_parent(&mut dom);
    for kind in 0..3 {
        let mut batch = DomBatch::new();
        let data = match kind {
            0 => batch.create_text(document, "A😀BC"),
            1 => batch.create_comment(document, "A😀BC"),
            _ => batch.create_processing_instruction(document, "range", "A😀BC"),
        };
        batch.insert_before(&data, parent, None);
        let data = apply(&mut dom, batch).created(&data).unwrap().unwrap();
        let ids: Vec<_> = (0..=5)
            .map(|offset| at(&mut dom, document, (data, offset), (data, offset)))
            .collect();
        let before = dom.snapshot();
        let mut batch = DomBatch::new();
        batch.replace_data(data, 1, 2, "xyZ");
        let commit = apply(&mut dom, batch);
        let after = dom.snapshot();
        assert_eq!(
            after.node(data).unwrap().data(),
            Some(&DocumentString::from("AxyZBC"))
        );
        for ((id, old), expected) in ids.iter().zip(0..=5).zip([0, 1, 1, 1, 5, 6]) {
            assert_eq!(bounds(&before, *id), ((data, old), (data, old)));
            assert_eq!(bounds(&after, *id), ((data, expected), (data, expected)));
        }
        assert!(commit.changes().changes().contains(&DomChange::Data(data)));
        let id = at(&mut dom, document, (data, 4), (data, 6));
        let mut batch = DomBatch::new();
        batch.replace_data(data, 4, usize::MAX, "😀");
        apply(&mut dom, batch);
        assert_eq!(bounds(&dom.snapshot(), id), ((data, 4), (data, 4)));
        let mut batch = DomBatch::new();
        batch.replace_data(data, 4, 0, "xy");
        apply(&mut dom, batch);
        assert_eq!(bounds(&dom.snapshot(), id), ((data, 4), (data, 4)));
    }
}

#[test]
fn equal_text_replacement_still_collapses_boundaries_and_reports_range_only_change() {
    let mut dom = dom();
    let (document, parent) = document_and_parent(&mut dom);
    let mut batch = DomBatch::new();
    let text = batch.create_text(document, "abcd");
    batch.insert_before(&text, parent, None);
    let text = apply(&mut dom, batch).created(&text).unwrap().unwrap();
    let id = at(&mut dom, document, (text, 1), (text, 3));
    associate(&mut dom, document, id);
    let before = dom.snapshot();
    let mut batch = DomBatch::new();
    batch.replace_data(text, 1, 2, "bc");
    let commit = apply(&mut dom, batch);
    let after = dom.snapshot();
    assert_ne!(after.stamp(), before.stamp());
    assert_eq!(
        after.node(text).unwrap().data(),
        before.node(text).unwrap().data()
    );
    assert_eq!(bounds(&after, id), ((text, 1), (text, 1)));
    assert_eq!(bounds(&before, id), ((text, 1), (text, 3)));
    assert_eq!(active(&after, document), Some(id));
    assert!(!commit.changes().changes().contains(&DomChange::Data(text)));
    assert_eq!(commit.changes().changes(), [DomChange::RangeBoundary(id)]);
    let mut noop = DomBatch::new();
    noop.set_range_start(id, text, 1);
    noop.set_range_end(id, text, 1);
    noop.set_active_range(document, Some(id.into()));
    let commit = apply(&mut dom, noop);
    assert_eq!(dom.snapshot().stamp(), after.stamp());
    assert!(commit.changes().changes().is_empty());
}

#[test]
fn fragment_insertion_shifts_strictly_after_reference_by_full_count_not_one_child() {
    let mut dom = dom();
    let (document, parent) = document_and_parent(&mut dom);
    let mut batch = DomBatch::new();
    let a = batch.create_comment(document, "a");
    let b = batch.create_comment(document, "b");
    let c = batch.create_comment(document, "c");
    let fragment = batch.create_fragment(document);
    let mut incoming = Vec::new();
    for name in ["x", "y", "z"] {
        let node = batch.create_comment(document, name);
        batch.insert_before(&node, &fragment, None);
        incoming.push(node);
    }
    for node in [&a, &b, &c] {
        batch.insert_before(node, parent, None);
    }
    let commit = apply(&mut dom, batch);
    let a = commit.created(&a).unwrap().unwrap();
    let b = commit.created(&b).unwrap().unwrap();
    let c = commit.created(&c).unwrap().unwrap();
    let fragment = commit.created(&fragment).unwrap().unwrap();
    let incoming: Vec<_> = incoming
        .iter()
        .map(|ticket| commit.created(ticket).unwrap().unwrap())
        .collect();
    let equality = at(&mut dom, document, (parent, 1), (parent, 1));
    let greater = at(&mut dom, document, (parent, 2), (parent, 3));
    let old_fragment = at(&mut dom, document, (fragment, 1), (fragment, 3));
    let old_child = at(&mut dom, document, (incoming[1], 0), (incoming[1], 1));
    let before = dom.snapshot();
    let mut batch = DomBatch::new();
    batch.insert_before(fragment, parent, Some(b.into()));
    apply(&mut dom, batch);
    let after = dom.snapshot();
    assert_eq!(
        after.node(parent).unwrap().children(),
        [a, incoming[0], incoming[1], incoming[2], b, c]
    );
    assert_eq!(bounds(&after, equality), ((parent, 1), (parent, 1)));
    assert_eq!(bounds(&after, greater), ((parent, 5), (parent, 6)));
    assert_eq!(bounds(&after, old_fragment), ((fragment, 0), (fragment, 0)));
    assert_eq!(bounds(&after, old_child), ((fragment, 0), (fragment, 0)));
    assert_eq!(bounds(&before, greater), ((parent, 2), (parent, 3)));
    let mut append = DomBatch::new();
    let tail = append.create_comment(document, "tail");
    append.insert_before(&tail, parent, None);
    apply(&mut dom, append);
    assert_eq!(bounds(&dom.snapshot(), greater), ((parent, 5), (parent, 6)));
    let stamp = dom.snapshot().stamp();
    let mut empty = DomBatch::new();
    empty.insert_before(fragment, parent, Some(b.into()));
    assert!(apply(&mut dom, empty).changes().changes().is_empty());
    assert_eq!(dom.snapshot().stamp(), stamp);
}

#[test]
fn removal_relocates_ordinary_descendants_but_does_not_cross_shadow_host_relation() {
    let mut dom = dom();
    let (document, parent) = document_and_parent(&mut dom);
    let mut batch = DomBatch::new();
    let left = batch.create_comment(document, "left");
    let host = batch.create_element_ns(document, Namespace::html(), "div");
    let light = batch.create_text(document, "light");
    let shadow = batch.attach_shadow(&host, DomShadowRootInit::new(HtmlShadowRootMode::Closed));
    let deep = batch.create_text(document, "shadow");
    let right = batch.create_comment(document, "right");
    batch.insert_before(&light, &host, None);
    batch.insert_before(&deep, &shadow, None);
    for node in [&left, &host, &right] {
        batch.insert_before(node, parent, None);
    }
    let commit = apply(&mut dom, batch);
    let host = commit.created(&host).unwrap().unwrap();
    let light = commit.created(&light).unwrap().unwrap();
    let deep = commit.created(&deep).unwrap().unwrap();
    let shadow = commit.created(&shadow).unwrap().unwrap();
    let inside = at(&mut dom, document, (light, 1), (light, 4));
    let hosted = at(&mut dom, document, (deep, 1), (deep, 5));
    let equality = at(&mut dom, document, (parent, 1), (parent, 1));
    let beyond = at(&mut dom, document, (parent, 2), (parent, 3));
    associate(&mut dom, document, inside);
    let before = dom.snapshot();
    let mut batch = DomBatch::new();
    batch.detach(host);
    apply(&mut dom, batch);
    let after = dom.snapshot();
    assert_eq!(bounds(&after, inside), ((parent, 1), (parent, 1)));
    assert_eq!(bounds(&after, equality), ((parent, 1), (parent, 1)));
    assert_eq!(bounds(&after, beyond), ((parent, 1), (parent, 2)));
    assert_eq!(bounds(&after, hosted), ((deep, 1), (deep, 5)));
    assert_eq!(after.node(deep).unwrap().root(), shadow);
    assert_eq!(active(&after, document), Some(inside));
    assert_eq!(bounds(&before, inside), ((light, 1), (light, 4)));
}

#[test]
fn actual_output_replace_all_applies_removal_maintenance_before_appending_new_text() {
    let mut dom = dom();
    let (document, parent) = document_and_parent(&mut dom);
    let mut batch = DomBatch::new();
    let output = batch.create_element_ns(document, Namespace::html(), "output");
    let first = batch.create_text(document, "first");
    let second = batch.create_text(document, "second");
    batch.insert_before(&first, &output, None);
    batch.insert_before(&second, &output, None);
    batch.insert_before(&output, parent, None);
    let commit = apply(&mut dom, batch);
    let output = commit.created(&output).unwrap().unwrap();
    let first = commit.created(&first).unwrap().unwrap();
    let second = commit.created(&second).unwrap().unwrap();
    let inside = at(&mut dom, document, (first, 2), (second, 3));
    let around = at(&mut dom, document, (output, 1), (output, 2));
    associate(&mut dom, document, inside);
    let before = dom.snapshot();
    let cancel = AtomicBool::new(false);
    let mut lease = dom
        .begin_source_edit(
            DomSourceEdit::set_output_value(output, "replacement"),
            source_limits(),
            &cancel,
        )
        .unwrap();
    assert!(matches!(
        lease.advance(&cancel).unwrap(),
        DomSourceProgress::Complete(_)
    ));
    drop(lease);
    let after = dom.snapshot();
    for id in [inside, around] {
        assert_eq!(bounds(&after, id), ((output, 0), (output, 0)));
    }
    assert_eq!(active(&after, document), Some(inside));
    assert_eq!(after.node(first).unwrap().parent(), None);
    assert_eq!(after.node(second).unwrap().parent(), None);
    let children = after.node(output).unwrap().children();
    assert_eq!(children.len(), 1);
    assert_eq!(
        after.node(children[0]).unwrap().data(),
        Some(&DocumentString::from("replacement"))
    );
    assert_eq!(bounds(&before, inside), ((first, 2), (second, 3)));
    assert!(dom.pending_source_edit().is_none());
}

#[test]
fn subsequent_setters_and_adoption_keep_the_same_association_across_boundary_roots() {
    let mut dom = dom();
    let (document, parent) = document_and_parent(&mut dom);
    let mut batch = DomBatch::new();
    let other = batch.create_document();
    let detached = batch.create_element_ns(document, Namespace::html(), "div");
    let text = batch.create_text(document, "detached");
    let shadow_host = batch.create_element_ns(document, Namespace::html(), "div");
    let shadow = batch.attach_shadow(
        &shadow_host,
        DomShadowRootInit::new(HtmlShadowRootMode::Open),
    );
    batch.insert_before(&text, &detached, None);
    batch.insert_before(&shadow_host, parent, None);
    let commit = apply(&mut dom, batch);
    let other = commit.document(&other).unwrap();
    let detached = commit.created(&detached).unwrap().unwrap();
    let text = commit.created(&text).unwrap().unwrap();
    let shadow = commit.created(&shadow).unwrap().unwrap();
    let id = at(&mut dom, document, (parent, 0), (parent, 1));
    associate(&mut dom, document, id);
    let original = dom.snapshot();
    for point in [(text, 4), (shadow, 0), (other.node(), 0)] {
        let mut batch = DomBatch::new();
        batch.set_range_start(id, point.0, point.1);
        apply(&mut dom, batch);
        assert_eq!(bounds(&dom.snapshot(), id), (point, point));
        assert_eq!(active(&dom.snapshot(), document), Some(id));
    }
    let mut batch = DomBatch::new();
    batch.set_range_end(id, text, 2);
    apply(&mut dom, batch);
    assert_eq!(bounds(&dom.snapshot(), id), ((text, 2), (text, 2)));
    let mut batch = DomBatch::new();
    batch.adopt(detached, other);
    apply(&mut dom, batch);
    let after = dom.snapshot();
    assert_eq!(after.node(text).unwrap().owner_document(), Some(other));
    assert_eq!(bounds(&after, id), ((text, 2), (text, 2)));
    assert_eq!(active(&after, document), Some(id));
    assert_eq!(active(&after, other), None);
    assert_eq!(bounds(&original, id), ((parent, 0), (parent, 1)));
    assert_eq!(
        original.node(text).unwrap().owner_document(),
        Some(document)
    );
}

#[test]
fn attached_adoption_relocates_to_the_original_parent_without_following_the_moved_subtree() {
    let mut dom = dom();
    let (document, parent) = document_and_parent(&mut dom);
    let mut batch = DomBatch::new();
    let other = batch.create_document();
    let child = batch.create_element_ns(document, Namespace::html(), "div");
    let text = batch.create_text(document, "range");
    batch.insert_before(&text, &child, None);
    batch.insert_before(&child, parent, None);
    let commit = apply(&mut dom, batch);
    let other = commit.document(&other).unwrap();
    let child = commit.created(&child).unwrap().unwrap();
    let text = commit.created(&text).unwrap().unwrap();
    let id = at(&mut dom, document, (text, 1), (text, 4));
    associate(&mut dom, document, id);
    let mut batch = DomBatch::new();
    batch.adopt(child, other);
    apply(&mut dom, batch);
    assert_eq!(bounds(&dom.snapshot(), id), ((parent, 0), (parent, 0)));
    assert_eq!(active(&dom.snapshot(), document), Some(id));
    assert_eq!(
        dom.snapshot().node(text).unwrap().owner_document(),
        Some(other)
    );
}

fn source_limits() -> DomSourceLimits {
    DomSourceLimits::new(16_000_000, 100_000, 10_000_000, 128, 4096, 4096, 1)
}

#[test]
fn initial_association_requires_ordinary_document_root_and_refusal_is_atomic() {
    let mut dom = dom();
    let (document, parent) = document_and_parent(&mut dom);
    let mut batch = DomBatch::new();
    let other = batch.create_document();
    let detached = batch.create_element_ns(document, Namespace::html(), "div");
    let fragment = batch.create_fragment(document);
    let host = batch.create_element_ns(document, Namespace::html(), "div");
    let shadow = batch.attach_shadow(&host, DomShadowRootInit::new(HtmlShadowRootMode::Open));
    batch.insert_before(&host, parent, None);
    let commit = apply(&mut dom, batch);
    let other = commit.document(&other).unwrap();
    let roots = [
        commit.created(&detached).unwrap().unwrap(),
        commit.created(&fragment).unwrap().unwrap(),
        commit.created(&shadow).unwrap().unwrap(),
        other.node(),
    ];
    let original = at(&mut dom, document, (parent, 0), (parent, 1));
    associate(&mut dom, document, original);
    for root in roots {
        let candidate = at(&mut dom, document, (root, 0), (root, 0));
        let before = dom.snapshot();
        let usage = dom.usage();
        let mut batch = DomBatch::new();
        batch.set_attribute(parent, "data-unpublished", "candidate");
        batch.set_active_range(document, Some(candidate.into()));
        assert_eq!(
            dom.apply(batch, &AtomicBool::new(false)).unwrap_err(),
            DomError::RangeAssociationIneligible
        );
        assert_eq!(dom.snapshot().stamp(), before.stamp());
        assert_eq!(dom.usage(), usage);
        assert_eq!(active(&dom.snapshot(), document), Some(original));
        assert_eq!(bounds(&dom.snapshot(), candidate), ((root, 0), (root, 0)));
        assert_eq!(
            dom.snapshot().node(parent).unwrap().kind(),
            before.node(parent).unwrap().kind()
        );
    }
    let mut clear = DomBatch::new();
    clear.set_active_range(document, None);
    let commit = apply(&mut dom, clear);
    assert_eq!(
        commit.changes().changes(),
        [DomChange::DocumentActiveRange(document)]
    );
    assert_eq!(active(&dom.snapshot(), document), None);
    assert_eq!(
        bounds(&dom.snapshot(), original),
        ((parent, 0), (parent, 1))
    );
}

#[test]
fn setters_collapse_the_other_endpoint_and_reject_invalid_containers_without_publication() {
    let mut dom = dom();
    let (document, parent) = document_and_parent(&mut dom);
    let mut batch = DomBatch::new();
    let text = batch.create_text(document, "abcd");
    let doctype = batch.create_document_type(document, "html", "", "");
    batch.insert_before(&text, parent, None);
    let commit = apply(&mut dom, batch);
    let text = commit.created(&text).unwrap().unwrap();
    let doctype = commit.created(&doctype).unwrap().unwrap();
    let id = at(&mut dom, document, (text, 1), (text, 3));
    associate(&mut dom, document, id);
    let mut batch = DomBatch::new();
    batch.set_range_start(id, text, 4);
    apply(&mut dom, batch);
    assert_eq!(bounds(&dom.snapshot(), id), ((text, 4), (text, 4)));
    assert!(dom.snapshot().range(id).unwrap().collapsed());
    let mut batch = DomBatch::new();
    batch.set_range_end(id, text, 0);
    apply(&mut dom, batch);
    assert_eq!(bounds(&dom.snapshot(), id), ((text, 0), (text, 0)));
    for (node, offset, expected) in [
        (text, 5, DomError::IndexSize),
        (doctype, 0, DomError::WrongNodeKind),
    ] {
        let before = dom.snapshot();
        let mut batch = DomBatch::new();
        batch.set_range_start(id, text, 2);
        batch.set_range_end(id, node, offset);
        assert_eq!(
            dom.apply(batch, &AtomicBool::new(false)).unwrap_err(),
            expected
        );
        assert_eq!(dom.snapshot().stamp(), before.stamp());
        assert_eq!(bounds(&dom.snapshot(), id), ((text, 0), (text, 0)));
        assert_eq!(active(&dom.snapshot(), document), Some(id));
    }
}

#[test]
fn destruction_decides_retirement_after_real_removal_maintenance_and_clears_exact_associations() {
    let mut dom = dom();
    let (first, parent) = document_and_parent(&mut dom);
    let mut batch = DomBatch::new();
    let second = batch.create_document();
    let third = batch.create_document();
    let root = batch.create_element_ns(first, Namespace::html(), "div");
    let text = batch.create_text(first, "retire");
    let attached = batch.create_element_ns(first, Namespace::html(), "div");
    let survivor_text = batch.create_text(first, "survive");
    batch.insert_before(&text, &root, None);
    batch.insert_before(&survivor_text, &attached, None);
    batch.insert_before(&attached, parent, None);
    let commit = apply(&mut dom, batch);
    let second = commit.document(&second).unwrap();
    let third = commit.document(&third).unwrap();
    let root = commit.created(&root).unwrap().unwrap();
    let text = commit.created(&text).unwrap().unwrap();
    let attached = commit.created(&attached).unwrap().unwrap();
    let survivor_text = commit.created(&survivor_text).unwrap().unwrap();
    let retiring = at(&mut dom, first, (first.node(), 0), (first.node(), 0));
    associate(&mut dom, first, retiring);
    let mut batch = DomBatch::new();
    batch.set_range_start(retiring, second, 0);
    batch.set_active_range(second, Some(retiring.into()));
    batch.set_range_start(retiring, root, 0);
    batch.set_range_end(retiring, text, 4);
    apply(&mut dom, batch);
    let unrelated = at(&mut dom, third, (third.node(), 0), (third.node(), 0));
    associate(&mut dom, third, unrelated);
    let surviving = at(&mut dom, first, (survivor_text, 1), (survivor_text, 5));
    let before = dom.snapshot();
    let mut batch = DomBatch::new();
    batch.destroy(root);
    let commit = apply(&mut dom, batch);
    let after = dom.snapshot();
    assert_eq!(after.range(retiring).unwrap_err(), DomError::StaleRange);
    assert_eq!(after.node(root).unwrap_err(), DomError::StaleNode);
    assert_eq!(after.node(text).unwrap_err(), DomError::StaleNode);
    assert_eq!(active(&after, first), None);
    assert_eq!(active(&after, second), None);
    assert_eq!(active(&after, third), Some(unrelated));
    assert_eq!(
        bounds(&after, unrelated),
        ((third.node(), 0), (third.node(), 0))
    );
    assert_eq!(bounds(&before, retiring), ((root, 0), (text, 4)));
    assert_eq!(active(&before, first), Some(retiring));
    assert_eq!(active(&before, second), Some(retiring));
    for change in [
        DomChange::RangeRetired(retiring),
        DomChange::DocumentActiveRange(first),
        DomChange::DocumentActiveRange(second),
    ] {
        assert_eq!(
            commit
                .changes()
                .changes()
                .iter()
                .filter(|actual| **actual == change)
                .count(),
            1
        );
    }
    // Standard pre-remove relocates this different Range to the surviving
    // parent. Later explicit subtree retirement must not retire it by history.
    let mut batch = DomBatch::new();
    batch.detach(attached);
    batch.destroy(attached);
    apply(&mut dom, batch);
    assert_eq!(
        bounds(&dom.snapshot(), surviving),
        ((parent, 0), (parent, 0))
    );
}

#[test]
fn either_boundary_and_collapsed_boundary_on_a_destroyed_detached_root_retire_once() {
    for endpoints in 0..3 {
        let mut dom = dom();
        let (document, _) = document_and_parent(&mut dom);
        let mut batch = DomBatch::new();
        let root = batch.create_element_ns(document, Namespace::html(), "div");
        let text = batch.create_text(document, "child");
        batch.insert_before(&text, &root, None);
        let commit = apply(&mut dom, batch);
        let root = commit.created(&root).unwrap().unwrap();
        let text = commit.created(&text).unwrap().unwrap();
        let (start, end) = match endpoints {
            0 => ((root, 0), (text, 2)),
            1 => ((text, 1), (root, 1)),
            _ => ((root, 0), (root, 0)),
        };
        let id = at(&mut dom, document, start, end);
        let before = dom.snapshot();
        let mut batch = DomBatch::new();
        batch.destroy(root);
        let commit = apply(&mut dom, batch);
        assert_eq!(dom.snapshot().range(id).unwrap_err(), DomError::StaleRange);
        assert_eq!(bounds(&before, id), (start, end));
        assert_eq!(
            commit
                .changes()
                .changes()
                .iter()
                .filter(|change| **change == DomChange::RangeRetired(id))
                .count(),
            1
        );
    }
}

#[test]
fn source_destruction_retires_original_associated_range_after_its_real_child_removal() {
    let mut dom = dom();
    let (document, _) = document_and_parent(&mut dom);
    let mut batch = DomBatch::new();
    let root = batch.create_element_ns(document, Namespace::html(), "div");
    let child = batch.create_text(document, "original");
    batch.insert_before(&child, &root, None);
    let commit = apply(&mut dom, batch);
    let root = commit.created(&root).unwrap().unwrap();
    let child = commit.created(&child).unwrap().unwrap();
    let id = at(
        &mut dom,
        document,
        (document.node(), 0),
        (document.node(), 0),
    );
    associate(&mut dom, document, id);
    let mut batch = DomBatch::new();
    batch.set_range_start(id, child, 1);
    batch.set_range_end(id, child, 5);
    apply(&mut dom, batch);
    let before = dom.snapshot();
    let cancel = AtomicBool::new(false);
    let mut lease = dom
        .begin_source_edit(DomSourceEdit::destroy(root), source_limits(), &cancel)
        .unwrap();
    let DomSourceProgress::Complete(done) = lease.advance(&cancel).unwrap() else {
        panic!("ordinary source destruction completes");
    };
    assert!(
        done.commit()
            .changes()
            .changes()
            .contains(&DomChange::RangeRetired(id))
    );
    assert!(
        done.commit()
            .changes()
            .changes()
            .contains(&DomChange::DocumentActiveRange(document))
    );
    drop(lease);
    assert_eq!(dom.snapshot().range(id).unwrap_err(), DomError::StaleRange);
    assert_eq!(active(&dom.snapshot(), document), None);
    assert_eq!(bounds(&before, id), ((child, 1), (child, 5)));
    assert!(dom.pending_source_edit().is_none());
    let mut batch = DomBatch::new();
    let replacement_node = batch.create_element_ns(document, Namespace::html(), "div");
    let replacement_range = batch.create_range(document);
    batch.set_range_start(&replacement_range, &replacement_node, 0);
    let commit = apply(&mut dom, batch);
    let replacement_node = commit.created(&replacement_node).unwrap().unwrap();
    let replacement_range = commit.created_range(&replacement_range).unwrap().unwrap();
    assert_ne!(replacement_node, root);
    assert_ne!(replacement_range, id);
    assert_eq!(active(&dom.snapshot(), document), None);
    assert_eq!(
        bounds(&dom.snapshot(), replacement_range),
        ((replacement_node, 0), (replacement_node, 0))
    );
    assert_eq!(dom.snapshot().range(id).unwrap_err(), DomError::StaleRange);
}

#[test]
fn explicit_retirement_reclaims_finite_capacity_without_destroying_boundary_nodes_or_history() {
    let mut dom = Dom::new(limits(1)).unwrap();
    let (document, parent) = document_and_parent(&mut dom);
    let id = at(&mut dom, document, (parent, 0), (parent, 0));
    associate(&mut dom, document, id);
    let old = dom.snapshot();
    let usage = dom.usage();
    let mut too_many = DomBatch::new();
    too_many.create_range(document);
    assert_eq!(
        dom.apply(too_many, &AtomicBool::new(false)).unwrap_err(),
        DomError::ResourceLimit {
            resource: DomResource::Ranges,
            limit: 1
        }
    );
    assert_eq!(dom.usage(), usage);
    assert_eq!(dom.snapshot().stamp(), old.stamp());
    let mut batch = DomBatch::new();
    batch.retire_range(id);
    let commit = apply(&mut dom, batch);
    assert_eq!(dom.usage().ranges(), 0);
    assert_eq!(commit.changes().changes().len(), 2);
    assert!(
        commit
            .changes()
            .changes()
            .contains(&DomChange::RangeRetired(id))
    );
    assert!(
        commit
            .changes()
            .changes()
            .contains(&DomChange::DocumentActiveRange(document))
    );
    assert_eq!(active(&dom.snapshot(), document), None);
    assert_eq!(
        dom.snapshot().node(parent).unwrap().children(),
        old.node(parent).unwrap().children()
    );
    assert_eq!(bounds(&old, id), ((parent, 0), (parent, 0)));
    assert_eq!(active(&old, document), Some(id));
    let replacement = at(
        &mut dom,
        document,
        (document.node(), 0),
        (document.node(), 0),
    );
    assert_ne!(replacement, id);
    assert_eq!(dom.usage().ranges(), 1);
    let before = dom.snapshot();
    for retire in [false, true] {
        let mut batch = DomBatch::new();
        if retire {
            batch.retire_range(id);
        } else {
            batch.set_range_end(id, parent, 0);
        }
        assert_eq!(
            dom.apply(batch, &AtomicBool::new(false)).unwrap_err(),
            DomError::StaleRange
        );
        assert_eq!(dom.snapshot().stamp(), before.stamp());
        assert_eq!(
            bounds(&dom.snapshot(), replacement),
            ((document.node(), 0), (document.node(), 0))
        );
    }
    let mut other = Dom::new(limits(64)).unwrap();
    let (foreign_document, _) = document_and_parent(&mut other);
    let foreign = at(
        &mut other,
        foreign_document,
        (foreign_document.node(), 0),
        (foreign_document.node(), 0),
    );
    assert_eq!(
        dom.snapshot().range(foreign).unwrap_err(),
        DomError::ForeignOwner
    );
    let mut batch = DomBatch::new();
    batch.retire_range(foreign);
    assert_eq!(
        dom.apply(batch, &AtomicBool::new(false)).unwrap_err(),
        DomError::ForeignOwner
    );
    assert_eq!(dom.snapshot().stamp(), before.stamp());
}

#[test]
fn staged_retirement_failure_and_cancellation_publish_no_dangling_range_or_association() {
    let mut dom = dom();
    let (document, parent) = document_and_parent(&mut dom);
    let before = dom.snapshot();
    let mut batch = DomBatch::new();
    let ephemeral = batch.create_range(document);
    batch.set_active_range(document, Some((&ephemeral).into()));
    batch.retire_range(&ephemeral);
    let commit = apply(&mut dom, batch);
    assert_eq!(commit.created_range(&ephemeral).unwrap(), None);
    assert_eq!(dom.usage().ranges(), 0);
    assert_eq!(active(&dom.snapshot(), document), None);
    assert_eq!(dom.snapshot().stamp(), before.stamp());
    assert!(commit.changes().changes().is_empty());
    let mut batch = DomBatch::new();
    let root = batch.create_element_ns(document, Namespace::html(), "div");
    let ephemeral = batch.create_range(document);
    batch.set_range_start(&ephemeral, &root, 0);
    batch.destroy(&root);
    let commit = apply(&mut dom, batch);
    assert_eq!(commit.created(&root).unwrap(), None);
    assert_eq!(commit.created_range(&ephemeral).unwrap(), None);
    assert_eq!(dom.usage().ranges(), 0);
    let retained = at(&mut dom, document, (parent, 0), (parent, 0));
    associate(&mut dom, document, retained);
    let before = dom.snapshot();
    let usage = dom.usage();
    let mut cancelled = DomBatch::new();
    cancelled.retire_range(retained);
    assert_eq!(
        dom.apply(cancelled, &AtomicBool::new(true)).unwrap_err(),
        DomError::Cancelled
    );
    let mut failed = DomBatch::new();
    failed.retire_range(retained);
    failed.set_range_start(retained, parent, 0);
    assert_eq!(
        dom.apply(failed, &AtomicBool::new(false)).unwrap_err(),
        DomError::StaleRange
    );
    assert_eq!(dom.snapshot().stamp(), before.stamp());
    assert_eq!(dom.usage(), usage);
    assert_eq!(active(&dom.snapshot(), document), Some(retained));
    assert_eq!(
        bounds(&dom.snapshot(), retained),
        ((parent, 0), (parent, 0))
    );
}

#[test]
fn explicit_retirement_clears_all_matching_documents_and_unassociated_retirement_preserves_nodes() {
    let mut dom = dom();
    let (first, parent) = document_and_parent(&mut dom);
    let mut batch = DomBatch::new();
    let second = batch.create_document();
    let second = apply(&mut dom, batch).document(&second).unwrap();
    let id = at(&mut dom, first, (parent, 0), (parent, 0));
    associate(&mut dom, first, id);
    let mut batch = DomBatch::new();
    batch.set_range_end(id, second, 0);
    batch.set_active_range(second, Some(id.into()));
    apply(&mut dom, batch);
    let before = dom.snapshot();
    let mut batch = DomBatch::new();
    batch.retire_range(id);
    let commit = apply(&mut dom, batch);
    for document in [first, second] {
        assert_eq!(active(&dom.snapshot(), document), None);
        assert_eq!(active(&before, document), Some(id));
        assert!(
            commit
                .changes()
                .changes()
                .contains(&DomChange::DocumentActiveRange(document))
        );
    }
    assert_eq!(
        bounds(&before, id),
        ((second.node(), 0), (second.node(), 0))
    );
    assert_eq!(dom.snapshot().node(second.node()).unwrap().children(), &[]);
    let unassociated = at(&mut dom, first, (parent, 0), (parent, 0));
    let mut batch = DomBatch::new();
    batch.retire_range(unassociated);
    let commit = apply(&mut dom, batch);
    assert_eq!(
        commit.changes().changes(),
        [DomChange::RangeRetired(unassociated)]
    );
    assert_eq!(
        dom.snapshot().node(parent).unwrap().parent(),
        Some(first.node())
    );
}

#[test]
fn default_disabled_range_allowance_and_source_work_refusal_preserve_original_state() {
    let mut dom = Dom::new(
        DomLimits::new(
            256,
            65_536,
            512,
            0,
            0,
            64,
            1_000_000,
            surgeist_dom::DomNameLimits::new(16_777_216, 16_384, 4_000_000, 1_048_576).unwrap(),
        )
        .unwrap(),
    )
    .unwrap();
    let (document, parent) = document_and_parent(&mut dom);
    let before = dom.snapshot();
    let mut batch = DomBatch::new();
    batch.create_range(document);
    assert_eq!(
        dom.apply(batch, &AtomicBool::new(false)).unwrap_err(),
        DomError::ResourceLimit {
            resource: DomResource::Ranges,
            limit: 0
        }
    );
    assert_eq!(dom.snapshot().stamp(), before.stamp());
    dom.set_limits(limits(1)).unwrap();
    let mut batch = DomBatch::new();
    let text = batch.create_text(document, "abc");
    batch.insert_before(&text, parent, None);
    let text = apply(&mut dom, batch).created(&text).unwrap().unwrap();
    let id = at(&mut dom, document, (text, 1), (text, 3));
    associate(&mut dom, document, id);
    let before = dom.snapshot();
    let usage = dom.usage();
    let cancel = AtomicBool::new(false);
    let refused = DomSourceLimits::new(16_000_000, 100_000, 0, 128, 4096, 4096, 1);
    let error = match dom.begin_source_edit(
        DomSourceEdit::replace_data(text, 0, 2, "x"),
        refused,
        &cancel,
    ) {
        Err(error) => error,
        Ok(mut lease) => {
            let error = lease.advance(&cancel).unwrap_err();
            assert!(!lease.status().has_published());
            error
        }
    };
    assert_eq!(error, DomError::SourceWorkLimit { limit: 0 });
    assert_eq!(dom.snapshot().stamp(), before.stamp());
    assert_eq!(dom.usage(), usage);
    assert_eq!(active(&dom.snapshot(), document), Some(id));
    assert_eq!(bounds(&dom.snapshot(), id), ((text, 1), (text, 3)));
    assert!(dom.pending_source_edit().is_none());
    let mut lease = dom
        .begin_source_edit(
            DomSourceEdit::replace_data(text, 0, 2, "x"),
            source_limits(),
            &cancel,
        )
        .unwrap();
    assert!(matches!(
        lease.advance(&cancel).unwrap(),
        DomSourceProgress::Complete(_)
    ));
    drop(lease);
    assert_eq!(bounds(&dom.snapshot(), id), ((text, 0), (text, 2)));
}

struct InsertionFixture {
    dom: Dom,
    document: DocumentId,
    parent: NodeId,
    a: NodeId,
    b: NodeId,
    fragment: NodeId,
    style: NodeId,
    span: NodeId,
    range: Option<DomRangeId>,
}
fn insertion_fixture(offset: Option<usize>, plain: bool) -> InsertionFixture {
    let mut dom = dom();
    let (document, parent) = document_and_parent(&mut dom);
    let mut batch = DomBatch::new();
    let a = batch.create_element_ns(document, Namespace::html(), "span");
    let b = batch.create_element_ns(document, Namespace::html(), "span");
    let fragment = batch.create_fragment(document);
    let style = batch.create_element_ns(document, Namespace::html(), "style");
    let text = batch.create_text(document, "a { color: red }");
    let span = batch.create_element_ns(document, Namespace::html(), "span");
    if plain {
        batch.set_attribute(&style, "type", "text/plain");
    }
    batch.insert_before(&text, &style, None);
    batch.insert_before(&style, &fragment, None);
    batch.insert_before(&span, &fragment, None);
    batch.insert_before(&a, parent, None);
    batch.insert_before(&b, parent, None);
    let commit = apply(&mut dom, batch);
    let a = commit.created(&a).unwrap().unwrap();
    let b = commit.created(&b).unwrap().unwrap();
    let fragment = commit.created(&fragment).unwrap().unwrap();
    let style = commit.created(&style).unwrap().unwrap();
    let span = commit.created(&span).unwrap().unwrap();
    let range = offset.map(|offset| at(&mut dom, document, (parent, offset), (parent, offset)));
    if let Some(range) = range {
        associate(&mut dom, document, range);
    }
    InsertionFixture {
        dom,
        document,
        parent,
        a,
        b,
        fragment,
        style,
        span,
        range,
    }
}

#[test]
fn source_fragment_rejects_invalid_first_exposure_before_any_prefix_or_service_request() {
    let mut fixture = insertion_fixture(Some(2), false);
    let id = fixture.range.unwrap();
    let before = fixture.dom.snapshot();
    let usage = fixture.dom.usage();
    let cancel = AtomicBool::new(false);
    // DOM insertion shifts by the full count2 before the first Style hook:
    // offset2->4, while that first prospective exposure has only3 children.
    let error = match fixture.dom.begin_source_edit(
        DomSourceEdit::insert_before(fixture.fragment, fixture.parent, Some(fixture.a)),
        source_limits(),
        &cancel,
    ) {
        Err(error) => error,
        Ok(mut lease) => {
            let error = lease.advance(&cancel).unwrap_err();
            assert!(!lease.status().has_published());
            assert_eq!(lease.snapshot().stamp(), before.stamp());
            error
        }
    };
    assert_eq!(error, DomError::RangePublicationUnavailable);
    let after = fixture.dom.snapshot();
    assert_eq!(after.stamp(), before.stamp());
    assert_eq!(fixture.dom.usage(), usage);
    assert_eq!(
        after.node(fixture.parent).unwrap().children(),
        [fixture.a, fixture.b]
    );
    assert_eq!(
        after.node(fixture.fragment).unwrap().children(),
        [fixture.style, fixture.span]
    );
    assert_eq!(
        bounds(&after, id),
        ((fixture.parent, 2), (fixture.parent, 2))
    );
    assert_eq!(active(&after, fixture.document), Some(id));
    assert!(fixture.dom.pending_source_edit().is_none());
    assert!(
        fixture
            .dom
            .changes_since(before.stamp())
            .unwrap()
            .changes()
            .is_empty()
    );
}

#[test]
fn source_fragment_safe_offsets_append_and_absent_ranges_keep_original_style_boundary_and_retry() {
    // offset1 shifts to3 and fits the actual first exposure; equality at
    // index0 stays0; append does not shift; no Range imposes no Range refusal.
    for (offset, append, expected) in [
        (Some(1), false, Some(3)),
        (Some(0), false, Some(0)),
        (Some(2), true, Some(2)),
        (None, false, None),
    ] {
        let mut fixture = insertion_fixture(offset, false);
        let before = fixture.dom.snapshot();
        let cancel = AtomicBool::new(false);
        let operation;
        let ordinal;
        {
            let mut lease = fixture
                .dom
                .begin_source_edit(
                    DomSourceEdit::insert_before(
                        fixture.fragment,
                        fixture.parent,
                        (!append).then_some(fixture.a),
                    ),
                    source_limits(),
                    &cancel,
                )
                .unwrap();
            operation = lease.operation();
            let DomSourceProgress::Request(DomSourceRequest::Style(request)) =
                lease.advance(&cancel).unwrap()
            else {
                panic!("actual first inline CSP boundary");
            };
            assert_eq!(request.source().element(), fixture.style);
            assert_eq!(
                request.action(),
                surgeist_dom::DomStyleAction::Update(HtmlStyleUpdateAction::CheckInlineCsp)
            );
            ordinal = request.source().ordinal();
            let snapshot = lease.snapshot();
            assert_eq!(snapshot.node(fixture.parent).unwrap().children().len(), 3);
            if let (Some(id), Some(offset)) = (fixture.range, expected) {
                assert_eq!(
                    bounds(&snapshot, id),
                    ((fixture.parent, offset), (fixture.parent, offset))
                );
                assert!(offset <= snapshot.node(fixture.parent).unwrap().children().len());
            }
        }
        let mut lease = fixture
            .dom
            .resume_source_edit(operation, source_limits(), &cancel)
            .unwrap();
        let DomSourceProgress::Request(DomSourceRequest::Style(request)) =
            lease.advance(&cancel).unwrap()
        else {
            panic!("same retained CSP request");
        };
        assert_eq!(request.source().ordinal(), ordinal);
        assert_eq!(request.source().element(), fixture.style);
        lease.request_cancel();
        // Supply a real explicit policy outcome. It prohibits inline Style,
        // so no CSSOM creation or synthetic Applied receipt is necessary.
        let block = surgeist_dom::DomStyleInlineCspObservation::evaluated(
            request.source(),
            Arc::new(vec!["style-src 'none'".into()]),
            19,
            false,
        );
        lease.accept_inline_csp(&request, &block, &cancel).unwrap();
        let DomSourceProgress::Complete(done) = lease.advance(&cancel).unwrap() else {
            panic!("original supported insertion tail completes");
        };
        assert_eq!(done.operation(), operation);
        assert!(done.cancellation_requested());
        drop(lease);
        let after = fixture.dom.snapshot();
        let children = if append {
            [fixture.a, fixture.b, fixture.style, fixture.span]
        } else {
            [fixture.style, fixture.span, fixture.a, fixture.b]
        };
        assert_eq!(after.node(fixture.parent).unwrap().children(), children);
        assert!(after.node(fixture.fragment).unwrap().children().is_empty());
        if let (Some(id), Some(final_offset)) = (fixture.range, expected) {
            assert_eq!(
                bounds(&after, id),
                (
                    (fixture.parent, final_offset),
                    (fixture.parent, final_offset)
                )
            );
            assert_eq!(active(&after, fixture.document), Some(id));
            let old_offset = offset.unwrap();
            assert_eq!(
                bounds(&before, id),
                ((fixture.parent, old_offset), (fixture.parent, old_offset))
            );
        }
        assert_eq!(
            after
                .node(fixture.style)
                .unwrap()
                .as_style()
                .unwrap()
                .sheet()
                .unwrap(),
            None
        );
        assert!(fixture.dom.pending_source_edit().is_none());
        assert_eq!(after.incomplete_source_operation(), None);
    }
}

#[test]
fn full_count_insertion_remains_supported_when_no_external_exposure_is_reached() {
    for through_source in [false, true] {
        let mut fixture = insertion_fixture(Some(2), true);
        let id = fixture.range.unwrap();
        let before = fixture.dom.snapshot();
        if through_source {
            let cancel = AtomicBool::new(false);
            let mut lease = fixture
                .dom
                .begin_source_edit(
                    DomSourceEdit::insert_before(fixture.fragment, fixture.parent, Some(fixture.a)),
                    source_limits(),
                    &cancel,
                )
                .unwrap();
            assert!(matches!(
                lease.advance(&cancel).unwrap(),
                DomSourceProgress::Complete(_)
            ));
        } else {
            let mut batch = DomBatch::new();
            batch.insert_before(fixture.fragment, fixture.parent, Some(fixture.a.into()));
            apply(&mut fixture.dom, batch);
        }
        let after = fixture.dom.snapshot();
        assert_eq!(
            after.node(fixture.parent).unwrap().children(),
            [fixture.style, fixture.span, fixture.a, fixture.b]
        );
        assert_eq!(
            bounds(&after, id),
            ((fixture.parent, 4), (fixture.parent, 4))
        );
        assert_eq!(
            bounds(&before, id),
            ((fixture.parent, 2), (fixture.parent, 2))
        );
        assert_eq!(active(&after, fixture.document), Some(id));
        assert!(fixture.dom.pending_source_edit().is_none());
    }
}

#[test]
fn source_range_lifecycle_returns_typed_identity_and_preserves_association_across_roots() {
    use surgeist_dom::DomSourceOutput;
    let mut dom = dom();
    let (document, parent) = document_and_parent(&mut dom);
    let mut setup = DomBatch::new();
    let other = setup.create_document();
    let text = setup.create_text(document, "original");
    setup.insert_before(&text, parent, None);
    let commit = apply(&mut dom, setup);
    let other = commit.document(&other).unwrap();
    let text = commit.created(&text).unwrap().unwrap();
    let old = at(&mut dom, document, (parent, 0), (parent, 1));
    associate(&mut dom, document, old);
    let before = dom.snapshot();
    let cancel = AtomicBool::new(false);
    let id = {
        let mut lease = dom
            .begin_source_edit(
                DomSourceEdit::create_range(document),
                source_limits(),
                &cancel,
            )
            .unwrap();
        let DomSourceProgress::Complete(done) = lease.advance(&cancel).unwrap() else {
            panic!("actual Range creation completes without external services");
        };
        let Some(DomSourceOutput::Range(id)) = done.output() else {
            panic!("created Range has its own typed output");
        };
        assert_eq!(
            done.commit().changes().changes(),
            [DomChange::RangeCreated(id)]
        );
        id
    };
    assert_ne!(id, old);
    assert_eq!(
        bounds(&dom.snapshot(), id),
        ((document.node(), 0), (document.node(), 0))
    );
    assert_eq!(active(&dom.snapshot(), document), Some(old));
    for edit in [
        DomSourceEdit::set_range_start(id, text, 2),
        DomSourceEdit::set_range_end(id, text, 5),
        DomSourceEdit::set_active_range(document, Some(id)),
    ] {
        let mut lease = dom
            .begin_source_edit(edit, source_limits(), &cancel)
            .unwrap();
        let DomSourceProgress::Complete(done) = lease.advance(&cancel).unwrap() else {
            panic!("intrinsic Range edit completes");
        };
        assert_eq!(done.output(), None);
    }
    let associated = dom.snapshot();
    assert_eq!(bounds(&associated, id), ((text, 2), (text, 5)));
    assert_eq!(active(&associated, document), Some(id));
    for edit in [
        DomSourceEdit::set_range_end(id, other.node(), 0),
        DomSourceEdit::set_active_range(other, Some(id)),
    ] {
        let mut lease = dom
            .begin_source_edit(edit, source_limits(), &cancel)
            .unwrap();
        assert!(matches!(
            lease.advance(&cancel).unwrap(),
            DomSourceProgress::Complete(_)
        ));
    }
    let moved = dom.snapshot();
    assert_eq!(bounds(&moved, id), ((other.node(), 0), (other.node(), 0)));
    assert_eq!(active(&moved, document), Some(id));
    assert_eq!(active(&moved, other), Some(id));
    {
        let mut lease = dom
            .begin_source_edit(DomSourceEdit::retire_range(id), source_limits(), &cancel)
            .unwrap();
        let DomSourceProgress::Complete(done) = lease.advance(&cancel).unwrap() else {
            panic!("exact original Range retirement completes");
        };
        assert_eq!(done.output(), None);
        for change in [
            DomChange::RangeRetired(id),
            DomChange::DocumentActiveRange(document),
            DomChange::DocumentActiveRange(other),
        ] {
            assert!(done.commit().changes().changes().contains(&change));
        }
    }
    assert_eq!(dom.snapshot().range(id).unwrap_err(), DomError::StaleRange);
    assert_eq!(active(&dom.snapshot(), document), None);
    assert_eq!(active(&dom.snapshot(), other), None);
    assert_eq!(bounds(&dom.snapshot(), old), ((parent, 0), (parent, 1)));
    assert_eq!(active(&before, document), Some(old));
    assert_eq!(bounds(&associated, id), ((text, 2), (text, 5)));
    assert!(dom.pending_source_edit().is_none());
}

#[test]
fn source_range_creation_requires_one_identity_and_one_output_without_publishing_refused_candidates()
 {
    use surgeist_dom::DomSourceOutput;
    let mut dom = dom();
    let (document, parent) = document_and_parent(&mut dom);
    let original = at(&mut dom, document, (parent, 0), (parent, 0));
    associate(&mut dom, document, original);
    let before = dom.snapshot();
    let usage = dom.usage();
    let cancel = AtomicBool::new(false);
    // No externally published prefix is needed. This intrinsic operation has
    // one new Range identity and exactly one RangeCreated change output; the
    // existing Range/Document association supplies neither an extra identity
    // nor a synthetic Source-marker change.
    for (bytes, outputs, identities, expected) in [
        (16_000_000, 1, 0, DomError::SourceIdentityLimit),
        (16_000_000, 0, 1, DomError::SourceOutputLimit),
        (
            0,
            1,
            1,
            DomError::ResourceLimit {
                resource: DomResource::SourceRetainedBytes,
                limit: 0,
            },
        ),
    ] {
        let policy = DomSourceLimits::new(bytes, 100_000, 10_000_000, 1, outputs, identities, 1);
        let error =
            match dom.begin_source_edit(DomSourceEdit::create_range(document), policy, &cancel) {
                Err(error) => error,
                Ok(mut lease) => {
                    let error = lease.advance(&cancel).unwrap_err();
                    assert!(!lease.status().has_published());
                    assert_eq!(lease.snapshot().stamp(), before.stamp());
                    error
                }
            };
        assert_eq!(error, expected);
        assert_eq!(dom.snapshot().stamp(), before.stamp());
        assert_eq!(dom.usage(), usage);
        assert_eq!(
            bounds(&dom.snapshot(), original),
            ((parent, 0), (parent, 0))
        );
        assert_eq!(active(&dom.snapshot(), document), Some(original));
        assert!(dom.pending_source_edit().is_none());
    }
    let exact = DomSourceLimits::new(16_000_000, 100_000, 10_000_000, 1, 1, 1, 1);
    let id = {
        let mut lease = dom
            .begin_source_edit(DomSourceEdit::create_range(document), exact, &cancel)
            .unwrap();
        let DomSourceProgress::Complete(done) = lease.advance(&cancel).unwrap() else {
            panic!("exact identity/output allowance completes");
        };
        let Some(DomSourceOutput::Range(id)) = done.output() else {
            panic!("actual newly created identity");
        };
        assert_eq!(
            done.commit().changes().changes(),
            [DomChange::RangeCreated(id)]
        );
        id
    };
    assert_ne!(id, original);
    assert_eq!(dom.usage().ranges(), usage.ranges() + 1);
    assert_eq!(
        bounds(&dom.snapshot(), id),
        ((document.node(), 0), (document.node(), 0))
    );
    assert_eq!(active(&dom.snapshot(), document), Some(original));
    assert_eq!(bounds(&before, original), ((parent, 0), (parent, 0)));
    assert!(dom.pending_source_edit().is_none());
}
