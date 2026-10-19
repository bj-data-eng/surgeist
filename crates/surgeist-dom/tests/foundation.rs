use std::sync::atomic::AtomicBool;

use surgeist_dom::{
    DocumentId, DocumentString, Dom, DomBatch, DomChange, DomError, DomHistoryError, DomLimits,
    DomNodeKind, DomResource, NodeId,
};

// Each native Document retains application/xml, UTF-8 and about:blank.
const DOCUMENT_UNITS: usize = 31;

fn limits() -> DomLimits {
    DomLimits::new(
        64,
        1024,
        128,
        0,
        0,
        16,
        64 * 1024,
        surgeist_dom::DomNameLimits::new(16_777_216, 16_384, 4_000_000, 1_048_576).unwrap(),
    )
    .unwrap()
}
fn apply(dom: &mut Dom, batch: DomBatch) -> surgeist_dom::DomCommit {
    dom.apply(batch, &AtomicBool::new(false)).unwrap()
}
fn document(dom: &mut Dom) -> DocumentId {
    let mut batch = DomBatch::new();
    let document = batch.create_document();
    apply(dom, batch).document(&document).unwrap()
}
fn comment(dom: &mut Dom, document: DocumentId, value: &str) -> NodeId {
    let mut batch = DomBatch::new();
    let node = batch.create_comment(document, value);
    apply(dom, batch).created(&node).unwrap().unwrap()
}

#[test]
fn document_and_detached_text_have_distinct_roots_and_owner_document_readback() {
    let mut dom = Dom::new(limits()).unwrap();
    let mut batch = DomBatch::new();
    let document = batch.create_document();
    let text = batch.create_text(&document, "detached");
    let commit = apply(&mut dom, batch);
    let document = commit.document(&document).unwrap();
    let text = commit.created(&text).unwrap().unwrap();
    let capture = dom.snapshot();
    assert_eq!(capture.revision().get(), 1);
    assert_eq!(
        capture.node(document.node()).unwrap().owner_document(),
        None
    );
    assert_eq!(capture.node(text).unwrap().owner_document(), Some(document));
    assert_eq!(capture.node(text).unwrap().root(), text);
    assert!(!capture.node(text).unwrap().is_connected());
    assert!(capture.node(document.node()).unwrap().is_connected());
    let view = capture.document(document).unwrap();
    assert_eq!(view.node(document.node()).unwrap().id(), document.node());
    assert_eq!(
        view.node(text).unwrap().data().unwrap().to_utf8().unwrap(),
        "detached"
    );
    assert_eq!(view.stamp().document(), document);
    assert_eq!(view.stamp().store(), capture.stamp());
}

#[test]
fn ordered_fragment_transfer_moves_comments_and_empties_the_fragment() {
    let mut dom = Dom::new(limits()).unwrap();
    let mut batch = DomBatch::new();
    let document = batch.create_document();
    let fragment = batch.create_fragment(&document);
    let first = batch.create_comment(&document, "first");
    let second = batch.create_comment(&document, "second");
    let last = batch.create_comment(&document, "last");
    batch.insert_before(&first, &fragment, None);
    batch.insert_before(&second, &fragment, None);
    batch.insert_before(&last, &document, None);
    batch.insert_before(&fragment, &document, Some((&last).into()));
    let report = apply(&mut dom, batch);
    let first = report.created(&first).unwrap().unwrap();
    let second = report.created(&second).unwrap().unwrap();
    let last = report.created(&last).unwrap().unwrap();
    let fragment = report.created(&fragment).unwrap().unwrap();
    let document = report.document(&document).unwrap();
    let capture = dom.snapshot();
    assert_eq!(
        capture.node(document.node()).unwrap().children(),
        [first, second, last]
    );
    assert!(capture.node(fragment).unwrap().children().is_empty());
    assert_eq!(capture.node(fragment).unwrap().parent(), None);
    assert_eq!(capture.node(first).unwrap().root(), document.node());
    assert!(capture.node(first).unwrap().is_connected());
}

#[test]
fn enumerated_documents_provide_checked_views_without_creation_reports() {
    let mut dom = Dom::new(limits()).unwrap();
    let mut batch = DomBatch::new();
    let first = batch.create_document();
    let second = batch.create_document();
    batch.create_text(&first, "text");
    batch.create_comment(&second, "comment");
    batch.create_fragment(&first);
    apply(&mut dom, batch);
    let capture = dom.snapshot();
    let mut documents = Vec::new();
    let mut ordinary = 0;
    for id in capture.nodes() {
        let node = capture.node(id).unwrap();
        if let Some(document) = node.as_document() {
            assert_eq!(document.node(id).unwrap().id(), id);
            assert_eq!(
                capture.document(document.id()).unwrap().stamp(),
                document.stamp()
            );
            documents.push(document.id());
        } else {
            assert!(!matches!(node.kind(), DomNodeKind::Document(_)));
            ordinary += 1;
        }
    }
    assert_eq!(documents.len(), 2);
    assert_ne!(documents[0], documents[1]);
    assert_eq!(ordinary, 3);
}

#[test]
fn document_text_rejection_rolls_back_fragment_transfer_and_prior_commands() {
    let mut dom = Dom::new(limits()).unwrap();
    let document = document(&mut dom);
    let mut setup = DomBatch::new();
    let fragment = setup.create_fragment(document);
    let marker = setup.create_comment(document, "marker");
    let text = setup.create_text(document, "text");
    setup.insert_before(&marker, &fragment, None);
    setup.insert_before(&text, &fragment, None);
    let setup = apply(&mut dom, setup);
    let fragment = setup.created(&fragment).unwrap().unwrap();
    let text = setup.created(&text).unwrap().unwrap();
    let before = dom.snapshot();
    let usage = dom.usage();
    let mut batch = DomBatch::new();
    let unpublished = batch.create_comment(document, "unpublished");
    batch.replace_data(text, 0, 4, "changed");
    batch.insert_before(fragment, document, None);
    assert_eq!(
        dom.apply(batch, &AtomicBool::new(false)).unwrap_err(),
        DomError::HierarchyRequest
    );
    assert_eq!(dom.snapshot().stamp(), before.stamp());
    assert_eq!(dom.usage(), usage);
    assert_eq!(
        dom.snapshot().node(fragment).unwrap().children(),
        before.node(fragment).unwrap().children()
    );
    assert_eq!(
        dom.snapshot().node(text).unwrap().data(),
        before.node(text).unwrap().data()
    );
    assert!(
        dom.changes_since(before.stamp())
            .unwrap()
            .changes()
            .is_empty()
    );
    let mut next = DomBatch::new();
    next.detach(&unpublished);
    assert_eq!(
        dom.apply(next, &AtomicBool::new(false)).unwrap_err(),
        DomError::ForeignBatch
    );
}

#[test]
fn utf16_replacement_uses_units_preserves_surrogates_and_clamps_count() {
    let mut dom = Dom::new(limits()).unwrap();
    let document = document(&mut dom);
    let mut batch = DomBatch::new();
    let text = batch.create_text(
        document,
        DocumentString::from_utf16(vec![0x41, 0xd83d, 0xde00, 0xd800, 0x42]),
    );
    let commit = apply(&mut dom, batch);
    let text = commit.created(&text).unwrap().unwrap();
    let mut edit = DomBatch::new();
    edit.replace_data(text, 2, 1, DocumentString::from_utf16(vec![0xdc00]));
    apply(&mut dom, edit);
    let capture = dom.snapshot();
    assert_eq!(
        capture.node(text).unwrap().data().unwrap().code_units(),
        [0x41, 0xd83d, 0xdc00, 0xd800, 0x42]
    );
    assert!(
        capture
            .node(text)
            .unwrap()
            .data()
            .unwrap()
            .to_utf8()
            .is_err()
    );
    let mut edit = DomBatch::new();
    edit.replace_data(text, 3, usize::MAX, "!");
    apply(&mut dom, edit);
    assert_eq!(
        dom.snapshot()
            .node(text)
            .unwrap()
            .data()
            .unwrap()
            .code_units(),
        [0x41, 0xd83d, 0xdc00, 0x21]
    );
    assert_eq!(dom.usage().string_code_units(), DOCUMENT_UNITS + 4);
    let before = dom.snapshot().stamp();
    let mut invalid = DomBatch::new();
    invalid.replace_data(text, 5, 0, "");
    assert_eq!(
        dom.apply(invalid, &AtomicBool::new(false)).unwrap_err(),
        DomError::IndexSize
    );
    assert_eq!(dom.snapshot().stamp(), before);
}

#[test]
fn detach_and_reinsert_preserve_identity_and_old_snapshot_relations() {
    let mut dom = Dom::new(limits()).unwrap();
    let document = document(&mut dom);
    let node = comment(&mut dom, document, "kept");
    let mut attach = DomBatch::new();
    attach.insert_before(node, document, None);
    apply(&mut dom, attach);
    let attached = dom.snapshot();
    let mut detach = DomBatch::new();
    detach.detach(node);
    apply(&mut dom, detach);
    let detached = dom.snapshot();
    assert_eq!(attached.node(node).unwrap().parent(), Some(document.node()));
    assert_eq!(detached.node(node).unwrap().parent(), None);
    assert_eq!(
        detached.node(node).unwrap().owner_document(),
        Some(document)
    );
    assert!(!detached.node(node).unwrap().is_connected());
    let mut attach = DomBatch::new();
    attach.insert_before(node, document, None);
    apply(&mut dom, attach);
    assert_eq!(dom.snapshot().node(node).unwrap().id(), node);
    assert!(dom.snapshot().node(node).unwrap().is_connected());
    assert_eq!(detached.node(node).unwrap().parent(), None);
}

#[test]
fn destruction_retires_subtree_ids_but_captured_data_outlives_store() {
    let mut dom = Dom::new(limits()).unwrap();
    let document = document(&mut dom);
    let mut batch = DomBatch::new();
    let fragment = batch.create_fragment(document);
    let child = batch.create_text(document, "old");
    batch.insert_before(&child, &fragment, None);
    let commit = apply(&mut dom, batch);
    let fragment = commit.created(&fragment).unwrap().unwrap();
    let child = commit.created(&child).unwrap().unwrap();
    let old = dom.snapshot();
    let mut destroy = DomBatch::new();
    destroy.destroy(fragment);
    apply(&mut dom, destroy);
    assert_eq!(
        dom.snapshot().node(fragment).unwrap_err(),
        DomError::StaleNode
    );
    assert_eq!(dom.snapshot().node(child).unwrap_err(), DomError::StaleNode);
    let replacement = comment(&mut dom, document, "replacement");
    assert_ne!(replacement, fragment);
    assert_ne!(replacement, child);
    assert_eq!(dom.snapshot().node(child).unwrap_err(), DomError::StaleNode);
    assert_eq!(dom.usage().managed_nodes(), 2);
    drop(dom);
    assert_eq!(
        old.node(child).unwrap().data().unwrap().to_utf8().unwrap(),
        "old"
    );
    assert_eq!(old.node(child).unwrap().parent(), Some(fragment));
}

#[test]
fn foreign_store_and_wrong_document_are_distinct_from_stale_ids() {
    let mut left = Dom::new(limits()).unwrap();
    let mut right = Dom::new(limits()).unwrap();
    let first = document(&mut left);
    let other = document(&mut right);
    assert_ne!(left.owner(), right.owner());
    assert_eq!(
        left.snapshot().node(other.node()).unwrap_err(),
        DomError::ForeignOwner
    );
    let second = document(&mut left);
    let node = comment(&mut left, first, "first document");
    assert_eq!(
        left.snapshot()
            .document(second)
            .unwrap()
            .node(node)
            .unwrap_err(),
        DomError::WrongDocument
    );
    let mut edit = DomBatch::new();
    edit.insert_before(node, second, None);
    apply(&mut left, edit);
    assert_eq!(
        left.snapshot().node(node).unwrap().owner_document(),
        Some(second)
    );
    assert_eq!(
        left.snapshot()
            .document(first)
            .unwrap()
            .node(node)
            .unwrap_err(),
        DomError::WrongDocument
    );
    let before = left.snapshot().stamp();
    let mut foreign = DomBatch::new();
    foreign.insert_before(node, other, None);
    assert_eq!(
        left.apply(foreign, &AtomicBool::new(false)).unwrap_err(),
        DomError::ForeignOwner
    );
    assert_eq!(left.snapshot().stamp(), before);
}

#[test]
fn reference_membership_self_cycles_and_noncontainer_parents_are_validated() {
    let mut dom = Dom::new(limits()).unwrap();
    let document = document(&mut dom);
    let child = comment(&mut dom, document, "child");
    let detached = comment(&mut dom, document, "detached");
    let initial = dom.snapshot().stamp();
    let mut edit = DomBatch::new();
    edit.insert_before(child, document, Some(detached.into()));
    assert_eq!(
        dom.apply(edit, &AtomicBool::new(false)).unwrap_err(),
        DomError::NotFound
    );
    let mut edit = DomBatch::new();
    let fragment = edit.create_fragment(document);
    edit.insert_before(&fragment, &fragment, None);
    assert_eq!(
        dom.apply(edit, &AtomicBool::new(false)).unwrap_err(),
        DomError::HierarchyRequest
    );
    let mut edit = DomBatch::new();
    edit.insert_before(child, detached, None);
    assert_eq!(
        dom.apply(edit, &AtomicBool::new(false)).unwrap_err(),
        DomError::HierarchyRequest
    );
    assert_eq!(dom.snapshot().stamp(), initial);
}

#[test]
fn attached_roots_and_documents_cannot_be_destroyed() {
    let mut dom = Dom::new(limits()).unwrap();
    let document = document(&mut dom);
    let child = comment(&mut dom, document, "child");
    let mut edit = DomBatch::new();
    edit.insert_before(child, document, None);
    apply(&mut dom, edit);
    for id in [document.node(), child] {
        let before = dom.snapshot().stamp();
        let mut edit = DomBatch::new();
        edit.destroy(id);
        assert_eq!(
            dom.apply(edit, &AtomicBool::new(false)).unwrap_err(),
            DomError::InvalidDestruction
        );
        assert_eq!(dom.snapshot().stamp(), before);
    }
}

#[test]
fn final_noops_and_unpublished_creation_destruction_do_not_advance_revision() {
    let mut dom = Dom::new(limits()).unwrap();
    let document = document(&mut dom);
    let child = comment(&mut dom, document, "same");
    let mut edit = DomBatch::new();
    edit.insert_before(child, document, None);
    apply(&mut dom, edit);
    let stamp = dom.snapshot().stamp();
    let usage = dom.usage();
    let mut edit = DomBatch::new();
    let transient = edit.create_text(document, "temporary");
    edit.destroy(&transient);
    edit.replace_data(child, 0, usize::MAX, "different");
    edit.replace_data(child, 0, usize::MAX, "same");
    edit.insert_before(child, document, Some(child.into()));
    let commit = apply(&mut dom, edit);
    assert_eq!(commit.created(&transient).unwrap(), None);
    assert!(commit.changes().changes().is_empty());
    assert_eq!(dom.snapshot().stamp(), stamp);
    assert_eq!(dom.usage(), usage);
}

#[test]
fn finite_nodes_strings_commands_and_lowered_limits_preserve_admitted_state() {
    let mut dom = Dom::new(
        DomLimits::new(
            2,
            DOCUMENT_UNITS + 3,
            3,
            0,
            0,
            4,
            4096,
            surgeist_dom::DomNameLimits::new(16_777_216, 16_384, 4_000_000, 1_048_576).unwrap(),
        )
        .unwrap(),
    )
    .unwrap();
    let document = document(&mut dom);
    let child = comment(&mut dom, document, "abc");
    let stamp = dom.snapshot().stamp();
    let usage = dom.usage();
    let mut edit = DomBatch::new();
    edit.create_comment(document, "");
    assert_eq!(
        dom.apply(edit, &AtomicBool::new(false)).unwrap_err(),
        DomError::ResourceLimit {
            resource: DomResource::ManagedNodes,
            limit: 2
        }
    );
    let mut edit = DomBatch::new();
    edit.replace_data(child, 0, 0, "x");
    assert_eq!(
        dom.apply(edit, &AtomicBool::new(false)).unwrap_err(),
        DomError::ResourceLimit {
            resource: DomResource::StringCodeUnits,
            limit: DOCUMENT_UNITS + 3
        }
    );
    let mut edit = DomBatch::new();
    for _ in 0..4 {
        edit.detach(child);
    }
    assert_eq!(
        dom.apply(edit, &AtomicBool::new(false)).unwrap_err(),
        DomError::ResourceLimit {
            resource: DomResource::Commands,
            limit: 3
        }
    );
    assert_eq!(
        dom.set_limits(
            DomLimits::new(
                1,
                DOCUMENT_UNITS + 3,
                3,
                0,
                0,
                4,
                4096,
                surgeist_dom::DomNameLimits::new(16_777_216, 16_384, 4_000_000, 1_048_576).unwrap()
            )
            .unwrap()
        ),
        Err(DomError::ResourceLimit {
            resource: DomResource::ManagedNodes,
            limit: 1
        })
    );
    assert_eq!(
        dom.set_limits(
            DomLimits::new(
                2,
                DOCUMENT_UNITS + 2,
                3,
                0,
                0,
                4,
                4096,
                surgeist_dom::DomNameLimits::new(16_777_216, 16_384, 4_000_000, 1_048_576).unwrap()
            )
            .unwrap()
        ),
        Err(DomError::ResourceLimit {
            resource: DomResource::StringCodeUnits,
            limit: DOCUMENT_UNITS + 2
        })
    );
    assert_eq!(dom.snapshot().stamp(), stamp);
    assert_eq!(dom.usage(), usage);
    assert_eq!(
        DomLimits::new(
            usize::MAX,
            0,
            0,
            0,
            0,
            0,
            0,
            surgeist_dom::DomNameLimits::new(16_777_216, 16_384, 4_000_000, 1_048_576).unwrap()
        ),
        Err(DomError::InvalidLimit(DomResource::ManagedNodes))
    );
}

#[test]
fn cancellation_publishes_neither_creation_nor_revision() {
    let mut dom = Dom::new(limits()).unwrap();
    let mut batch = DomBatch::new();
    batch.create_document();
    assert_eq!(
        dom.apply(batch, &AtomicBool::new(true)).unwrap_err(),
        DomError::Cancelled
    );
    assert_eq!(dom.revision().get(), 0);
    assert_eq!(dom.snapshot().nodes().count(), 0);
    assert_eq!(dom.usage().history_entries(), 0);
}

#[test]
fn history_is_complete_in_order_or_reports_explicit_recomputation() {
    let mut dom = Dom::new(limits()).unwrap();
    let document = document(&mut dom);
    let baseline = dom.snapshot().stamp();
    let child = comment(&mut dom, document, "a");
    let after_creation = dom.snapshot().stamp();
    let mut edit = DomBatch::new();
    edit.replace_data(child, 0, 1, "b");
    apply(&mut dom, edit);
    let changes = dom.changes_since(baseline).unwrap();
    assert_eq!(
        changes.changes(),
        [DomChange::Created(child), DomChange::Data(child)]
    );
    assert_eq!(changes.from(), baseline);
    assert_eq!(changes.to(), dom.snapshot().stamp());
    let current = dom.snapshot().stamp();
    dom.set_limits(
        DomLimits::new(
            64,
            1024,
            128,
            0,
            0,
            1,
            65536,
            surgeist_dom::DomNameLimits::new(16_777_216, 16_384, 4_000_000, 1_048_576).unwrap(),
        )
        .unwrap(),
    )
    .unwrap();
    assert_eq!(
        dom.changes_since(after_creation).unwrap().changes(),
        [DomChange::Data(child)]
    );
    assert!(matches!(
        dom.changes_since(baseline),
        Err(DomHistoryError::FullRecomputeRequired { .. })
    ));
    assert!(dom.changes_since(current).unwrap().changes().is_empty());
    dom.set_limits(
        DomLimits::new(
            64,
            1024,
            128,
            0,
            0,
            16,
            0,
            surgeist_dom::DomNameLimits::new(16_777_216, 16_384, 4_000_000, 1_048_576).unwrap(),
        )
        .unwrap(),
    )
    .unwrap();
    let child2 = comment(&mut dom, document, "still legal");
    assert!(matches!(
        dom.snapshot().node(child2).unwrap().kind(),
        DomNodeKind::Comment(_)
    ));
    assert_eq!(dom.usage().history_entries(), 0);
    assert_eq!(dom.usage().history_bytes(), 0);
    assert!(matches!(
        dom.changes_since(current),
        Err(DomHistoryError::FullRecomputeRequired { .. })
    ));
    let other = Dom::new(limits()).unwrap();
    assert_eq!(
        dom.changes_since(other.snapshot().stamp()).unwrap_err(),
        DomHistoryError::ForeignOwner
    );
}

#[test]
fn positive_history_budget_too_small_for_one_record_falls_back_without_rejecting_edits() {
    let mut dom = Dom::new(
        DomLimits::new(
            4,
            DOCUMENT_UNITS + 32,
            8,
            0,
            0,
            8,
            1,
            surgeist_dom::DomNameLimits::new(16_777_216, 16_384, 4_000_000, 1_048_576).unwrap(),
        )
        .unwrap(),
    )
    .unwrap();
    let baseline = dom.snapshot().stamp();
    let document = document(&mut dom);
    let node = comment(&mut dom, document, "admitted");
    assert_eq!(
        dom.snapshot()
            .node(node)
            .unwrap()
            .data()
            .unwrap()
            .to_utf8()
            .unwrap(),
        "admitted"
    );
    assert_eq!(dom.usage().history_entries(), 0);
    assert_eq!(dom.usage().history_bytes(), 0);
    assert!(matches!(
        dom.changes_since(baseline),
        Err(DomHistoryError::FullRecomputeRequired { .. })
    ));
    assert!(
        dom.changes_since(dom.snapshot().stamp())
            .unwrap()
            .changes()
            .is_empty()
    );
}

#[test]
fn fresh_identity_replacement_is_a_change_even_when_contents_are_equal() {
    let mut dom = Dom::new(limits()).unwrap();
    let document = document(&mut dom);
    let old = comment(&mut dom, document, "equal");
    let before = dom.snapshot();
    let mut edit = DomBatch::new();
    edit.destroy(old);
    let fresh = edit.create_comment(document, "equal");
    let commit = apply(&mut dom, edit);
    let fresh = commit.created(&fresh).unwrap().unwrap();
    assert_ne!(fresh, old);
    assert_eq!(dom.revision().get(), before.revision().get() + 1);
    assert_eq!(
        commit.changes().changes(),
        [DomChange::Destroyed(old), DomChange::Created(fresh)]
    );
    assert_eq!(
        before.node(old).unwrap().data(),
        dom.snapshot().node(fresh).unwrap().data()
    );
}

#[test]
fn borrowed_kinds_copy_exact_snapshot_payload_references() {
    fn copy_view<'a>(kind: DomNodeKind<'a>) -> (DomNodeKind<'a>, DomNodeKind<'a>) {
        (kind, kind)
    }
    let mut dom = Dom::new(limits()).unwrap();
    let mut batch = DomBatch::new();
    let document = batch.create_document();
    let element = batch.create_element_ns(&document, surgeist_dom::Namespace::html(), "div");
    let text = batch.create_text(&document, DocumentString::from_utf16(vec![0xd800, 0x61]));
    batch.insert_before(&element, &document, None);
    batch.insert_before(&text, &element, None);
    let commit = apply(&mut dom, batch);
    let document = commit.document(&document).unwrap();
    let element = commit.created(&element).unwrap().unwrap();
    let text = commit.created(&text).unwrap().unwrap();
    let snapshot = dom.snapshot();
    let (first, second) = copy_view(snapshot.node(text).unwrap().kind());
    let (DomNodeKind::Text(first), DomNodeKind::Text(second)) = (first, second) else {
        panic!("actual Text payload");
    };
    assert!(std::ptr::eq(first, second));
    assert!(std::ptr::eq(
        first,
        snapshot.node(text).unwrap().data().unwrap()
    ));
    let DomNodeKind::Element(language) = snapshot.node(element).unwrap().kind() else {
        panic!("actual Element payload");
    };
    assert!(std::ptr::eq(
        language,
        snapshot.node(element).unwrap().as_element().unwrap()
    ));
    let DomNodeKind::Document(metadata) = snapshot.node(document.node()).unwrap().kind() else {
        panic!("actual Document payload");
    };
    assert!(std::ptr::eq(
        metadata,
        snapshot.document(document).unwrap().metadata()
    ));
    let mut edit = DomBatch::new();
    edit.replace_data(text, 0, usize::MAX, "new");
    apply(&mut dom, edit);
    assert_eq!(
        dom.snapshot()
            .node(text)
            .unwrap()
            .data()
            .unwrap()
            .code_units(),
        &[110, 101, 119]
    );
    drop(dom);
    // The views borrow the original snapshot rather than the temporary DomNode
    // values used to obtain them or the subsequently edited live owner.
    assert_eq!(first.code_units(), &[0xd800, 0x61]);
    assert_eq!(language.name().local_name().code_units(), &[100, 105, 118]);
    assert_eq!(metadata.class(), surgeist_dom::DomDocumentClass::Xml);
}
