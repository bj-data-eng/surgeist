//! Positive owner fixtures start at structural state, not a completed native
//! ShowModal invocation. No watcher receipt, focus effect or callback is faked.
use super::*;
use crate::{DomTopLayerUaRule, Namespace};

fn cancel() -> AtomicBool {
    AtomicBool::new(false)
}
fn limits(work: u64) -> crate::DomSourceLimits {
    crate::DomSourceLimits::new(32_000_000, 1_000_000, work, 32, 4096, 4096, 2)
}
fn fixture() -> (Dom, DocumentId, DocumentId, [NodeId; 3]) {
    let mut dom =
        Dom::new(DomLimits::new(64, 100000, 256, 128, 128, 16, 1_000_000).unwrap()).unwrap();
    let mut batch = DomBatch::new();
    let document = batch.create_document();
    let destination = batch.create_document();
    let root = batch.create_element_ns(&document, Namespace::html(), "div");
    batch.insert_before(&root, &document, None);
    let first = batch.create_element_ns(&document, Namespace::html(), "dialog");
    let other = batch.create_element_ns(&document, Namespace::html(), "div");
    let last = batch.create_element_ns(&document, Namespace::html(), "dialog");
    for node in [&first, &other, &last] {
        batch.insert_before(node, &root, None);
    }
    batch.set_attribute(&first, "open", "");
    batch.set_attribute(&last, "popover", "auto");
    batch.set_attribute(&other, "popover", "hint");
    let commit = dom.apply(batch, &cancel()).unwrap();
    (
        dom,
        commit.document(&document).unwrap(),
        commit.document(&destination).unwrap(),
        [
            commit.created(&first).unwrap().unwrap(),
            commit.created(&other).unwrap().unwrap(),
            commit.created(&last).unwrap().unwrap(),
        ],
    )
}
fn draft(dom: &Dom) -> Draft {
    let batch = DomBatch::new();
    Draft::new_source(
        dom,
        batch.owner.clone(),
        &batch.commands,
        limits(10_000_000),
        &cancel(),
    )
    .unwrap()
    .0
}
fn publish(dom: &mut Dom, draft: &Draft) -> DomCommit {
    let prepared = draft.finish(dom, &cancel()).unwrap();
    dom.publish(prepared, &cancel()).unwrap()
}
fn positive_fixture() -> (Dom, DocumentId, DocumentId, [NodeId; 3]) {
    let (mut dom, document, destination, ids) = fixture();
    let mut staged = draft(&dom);
    for id in ids {
        let key = staged.resolve(id.into()).unwrap();
        staged.add_top_layer(key, &cancel()).unwrap();
    }
    let key = staged.resolve(ids[0].into()).unwrap();
    staged.set_dialog_modal(key, true, &cancel()).unwrap();
    publish(&mut dom, &staged);
    (dom, document, destination, ids)
}

#[test]
fn raw_pending_and_ua_input_follow_add_request_readd_and_immediate_removal_order() {
    let (mut dom, document, _, ids) = positive_fixture();
    let original = dom.snapshot();
    let mut staged = draft(&dom);
    let [first, other, last] = ids.map(|id| staged.resolve(id.into()).unwrap());
    staged.request_top_layer_removal(last, &cancel()).unwrap();
    staged.request_top_layer_removal(first, &cancel()).unwrap();
    staged.request_top_layer_removal(first, &cancel()).unwrap();
    publish(&mut dom, &staged);
    let requested = dom.snapshot();
    let view = requested.document(document).unwrap().top_layer().unwrap();
    assert_eq!(view.raw_members(), &ids);
    assert_eq!(view.pending_removals(), &[ids[2], ids[0]]);
    assert_eq!(
        view.ua_rules().collect::<Vec<_>>(),
        [
            DomTopLayerUaRule::UniversalOverlayNone,
            DomTopLayerUaRule::ElementOverlayAuto(ids[1])
        ]
    );
    let mut staged = draft(&dom);
    staged.add_top_layer(first, &cancel()).unwrap();
    publish(&mut dom, &staged);
    let readded = dom.snapshot();
    let view = readded.document(document).unwrap().top_layer().unwrap();
    assert_eq!(view.raw_members(), &[ids[1], ids[2], ids[0]]);
    assert_eq!(view.pending_removals(), &[ids[2]]);
    assert_eq!(
        view.ua_rules().collect::<Vec<_>>(),
        [
            DomTopLayerUaRule::UniversalOverlayNone,
            DomTopLayerUaRule::ElementOverlayAuto(ids[1]),
            DomTopLayerUaRule::ElementOverlayAuto(ids[0])
        ]
    );
    let mut staged = draft(&dom);
    assert_eq!(
        staged.add_top_layer(first, &cancel()),
        Err(DomError::TopLayerAlreadyActive {
            subject: ids[0],
            document
        })
    );
    staged
        .remove_top_layer_immediately(last, &cancel())
        .unwrap();
    staged.request_top_layer_removal(last, &cancel()).unwrap();
    staged
        .remove_top_layer_immediately(last, &cancel())
        .unwrap();
    publish(&mut dom, &staged);
    assert_eq!(
        dom.snapshot()
            .document(document)
            .unwrap()
            .top_layer()
            .unwrap()
            .raw_members(),
        &[ids[1], ids[0]]
    );
    assert_eq!(
        original
            .document(document)
            .unwrap()
            .top_layer()
            .unwrap()
            .raw_members(),
        &ids
    );
    assert!(
        original
            .document(document)
            .unwrap()
            .top_layer()
            .unwrap()
            .pending_removals()
            .is_empty()
    );
    assert_eq!(
        requested
            .document(document)
            .unwrap()
            .top_layer()
            .unwrap()
            .pending_removals(),
        &[ids[2], ids[0]]
    );
    assert_ne!(view.stamp(), original.document(document).unwrap().stamp());
    let _ = other;
}

#[test]
fn manual_open_removal_preserves_modal_raw_but_request_and_real_removal_clear_their_own_state() {
    let (mut dom, document, destination, ids) = positive_fixture();
    let original = dom.snapshot();
    let mut batch = DomBatch::new();
    batch.remove_attribute(ids[0], "open");
    dom.apply(batch, &cancel()).unwrap();
    assert!(
        dom.snapshot()
            .node(ids[0])
            .unwrap()
            .as_dialog()
            .unwrap()
            .is_modal()
    );
    assert_eq!(
        dom.snapshot()
            .document(document)
            .unwrap()
            .top_layer()
            .unwrap()
            .raw_members(),
        &ids
    );
    // Close's two structural actions share these production helpers. This
    // owning-tail fixture does not claim a completed event-bearing Close call.
    let mut staged = draft(&dom);
    let first = staged.resolve(ids[0].into()).unwrap();
    staged.request_top_layer_removal(first, &cancel()).unwrap();
    staged.set_dialog_modal(first, false, &cancel()).unwrap();
    let structural = publish(&mut dom, &staged);
    assert_eq!(structural.changes().changes().len(), 2);
    assert!(
        structural
            .changes()
            .changes()
            .contains(&DomChange::ElementState(ids[0]))
    );
    assert!(
        structural
            .changes()
            .changes()
            .contains(&DomChange::DocumentInteractiveState(document))
    );
    assert!(
        !dom.snapshot()
            .node(ids[0])
            .unwrap()
            .as_dialog()
            .unwrap()
            .is_modal()
    );
    assert_eq!(
        dom.snapshot()
            .document(document)
            .unwrap()
            .top_layer()
            .unwrap()
            .pending_removals(),
        &[ids[0]]
    );
    // Actual adoption runs removal against the old Document before membership
    // changes, including immediate removal from both raw and pending lists.
    let mut batch = DomBatch::new();
    batch.adopt(ids[0], destination);
    dom.apply(batch, &cancel()).unwrap();
    assert_eq!(
        dom.snapshot().node(ids[0]).unwrap().owner_document(),
        Some(destination)
    );
    let after = dom.snapshot();
    let view = after.document(document).unwrap().top_layer().unwrap();
    assert_eq!(view.raw_members(), &[ids[1], ids[2]]);
    assert!(view.pending_removals().is_empty());
    assert!(
        original
            .node(ids[0])
            .unwrap()
            .as_dialog()
            .unwrap()
            .is_modal()
    );
    let mut batch = DomBatch::new();
    batch.detach(ids[2]);
    dom.apply(batch, &cancel()).unwrap();
    assert_eq!(
        dom.snapshot()
            .document(document)
            .unwrap()
            .top_layer()
            .unwrap()
            .raw_members(),
        &[ids[1]]
    );
}

#[test]
fn imported_clones_are_fresh_and_destruction_cleans_actual_dialog_membership() {
    let (mut dom, document, destination, ids) = positive_fixture();
    let old = dom.snapshot();
    let mut batch = DomBatch::new();
    let clone = batch.import_node(&old, ids[0], document, false);
    let imported = batch.import_node(&old, ids[0], destination, false);
    let commit = dom.apply(batch, &cancel()).unwrap();
    for id in [
        commit.created(&clone).unwrap().unwrap(),
        commit.created(&imported).unwrap().unwrap(),
    ] {
        assert!(
            !dom.snapshot()
                .node(id)
                .unwrap()
                .as_dialog()
                .unwrap()
                .is_modal()
        );
        assert!(
            !dom.snapshot()
                .document(document)
                .unwrap()
                .top_layer()
                .unwrap()
                .raw_members()
                .contains(&id)
        );
    }
    let before_rejection = dom.snapshot();
    let history = dom.changes_since(old.stamp()).unwrap();
    let mut batch = DomBatch::new();
    batch.destroy(ids[0]);
    assert_eq!(
        dom.apply(batch, &cancel()).unwrap_err(),
        DomError::InvalidDestruction
    );
    assert_eq!(dom.snapshot().stamp(), before_rejection.stamp());
    assert!(
        dom.snapshot()
            .node(ids[0])
            .unwrap()
            .as_dialog()
            .unwrap()
            .is_modal()
    );
    assert_eq!(
        dom.snapshot()
            .document(document)
            .unwrap()
            .top_layer()
            .unwrap()
            .raw_members(),
        &ids
    );
    let retained_history = dom.changes_since(old.stamp()).unwrap();
    assert_eq!(retained_history.from(), history.from());
    assert_eq!(retained_history.to(), history.to());
    assert_eq!(retained_history.changes(), history.changes());
    let mut batch = DomBatch::new();
    batch.detach(ids[0]);
    batch.destroy(ids[0]);
    dom.apply(batch, &cancel()).unwrap();
    assert!(matches!(
        dom.snapshot().node(ids[0]),
        Err(DomError::StaleNode)
    ));
    assert_eq!(
        dom.snapshot()
            .document(document)
            .unwrap()
            .top_layer()
            .unwrap()
            .raw_members(),
        &[ids[1], ids[2]]
    );
    assert_eq!(
        old.document(document)
            .unwrap()
            .top_layer()
            .unwrap()
            .raw_members(),
        &ids
    );
}

#[test]
fn hidden_actual_popovers_allow_nonempty_modal_raw_and_unknown_state_refuses() {
    let (mut dom, document, _, ids) = positive_fixture();
    let mut staged = draft(&dom);
    assert!(
        staged
            .document_known_empty_popovers(document, &cancel())
            .unwrap()
    );
    let key = staged.resolve(document.node().into()).unwrap();
    staged
        .with_node_fields(key, &cancel(), |fields| {
            fields.interactive_document = Some(DocumentInteractiveState::Unavailable)
        })
        .unwrap();
    assert_eq!(
        staged.add_top_layer(staged.resolve(ids[0].into()).unwrap(), &cancel()),
        Err(DomError::DialogDocumentStateUnavailable)
    );
    assert!(
        !staged
            .document_known_empty_popovers(document, &cancel())
            .unwrap()
    );
    assert_eq!(
        staged.add_top_layer(key, &cancel()),
        Err(DomError::WrongNodeKind)
    );
    publish(&mut dom, &staged);
    assert!(matches!(
        dom.snapshot().document(document).unwrap().top_layer(),
        Err(DomError::DialogDocumentStateUnavailable)
    ));
}

#[test]
fn original_checkpoint_preserves_pending_modal_state_capacity_and_cumulative_admission() {
    let (dom, document, _, ids) = positive_fixture();
    let mut staged = draft(&dom);
    let first = staged.resolve(ids[0].into()).unwrap();
    let last = staged.resolve(ids[2].into()).unwrap();
    staged.request_top_layer_removal(last, &cancel()).unwrap();
    let storage = staged.source_storage_bytes(&cancel()).unwrap();
    let checkpoint = staged.source_checkpoint(0, 0, &cancel()).unwrap();
    staged.request_top_layer_removal(first, &cancel()).unwrap();
    staged.set_dialog_modal(first, false, &cancel()).unwrap();
    let spent = staged.source_work.load(Ordering::Relaxed);
    checkpoint.restore(&mut staged);
    assert_eq!(staged.source_work.load(Ordering::Relaxed), spent);
    assert!(staged.dialog_state(first).unwrap().is_modal());
    assert_eq!(staged.source_storage_bytes(&cancel()).unwrap(), storage);
    let document = staged.resolve(document.node().into()).unwrap();
    assert_eq!(
        staged
            .staged_dialog_list(document, DialogList::Pending)
            .unwrap(),
        &[last]
    );
    let baseline = staged.source_work.load(Ordering::Relaxed);
    staged.source_limits = Some(limits(baseline));
    assert_eq!(
        staged.request_top_layer_removal(first, &cancel()),
        Err(DomError::SourceWorkLimit { limit: baseline })
    );
    assert_eq!(staged.source_work.load(Ordering::Relaxed), baseline);
    assert_eq!(
        staged.request_top_layer_removal(first, &AtomicBool::new(true)),
        Err(DomError::Cancelled)
    );
    staged.source_limits = Some(limits(10_000_000));
    staged.request_top_layer_removal(first, &cancel()).unwrap();
    assert_eq!(
        staged
            .staged_dialog_list(document, DialogList::Pending)
            .unwrap(),
        &[last, first]
    );
}

#[test]
fn semantic_blocker_is_last_actual_raw_dialog_independent_of_pending_open_and_modal() {
    use surgeist_html_semantics::{
        HtmlCaptureLimits, HtmlInertFactLimits, HtmlLanguageDirectionLimits,
        HtmlSemanticCancellation, HtmlSemanticFact, HtmlSemanticLimits, HtmlSemanticQuery,
        HtmlSemanticRequest, HtmlSemanticValue,
    };
    let (mut dom, document, _, ids) = positive_fixture();
    let mut staged = draft(&dom);
    let other = staged.resolve(ids[1].into()).unwrap();
    let last = staged.resolve(ids[2].into()).unwrap();
    staged.request_top_layer_removal(other, &cancel()).unwrap();
    staged.add_top_layer(other, &cancel()).unwrap();
    staged.request_top_layer_removal(last, &cancel()).unwrap();
    publish(&mut dom, &staged);
    let snapshot = dom.snapshot();
    assert!(
        !snapshot
            .node(ids[2])
            .unwrap()
            .as_dialog()
            .unwrap()
            .is_modal()
    );
    assert!(
        snapshot
            .node(ids[2])
            .unwrap()
            .attribute(&"open".into())
            .is_none()
    );
    assert_eq!(
        snapshot
            .document(document)
            .unwrap()
            .top_layer()
            .unwrap()
            .raw_members(),
        &[ids[0], ids[2], ids[1]]
    );
    let captured = snapshot
        .html_capture(
            document,
            None,
            HtmlCaptureLimits::new(64, 1024, 128, 16384, 64, 0, 0),
            crate::DomHtmlCaptureLimits::new(64, 1000000),
            &cancel(),
        )
        .unwrap();
    let captured = captured
        .with_slot_semantics(
            crate::DomSlotSemanticLimits::new(
                crate::DomSlotQueryLimits::new(64, 128).unwrap(),
                crate::DomHtmlCaptureLimits::new(64, 1000000),
                HtmlLanguageDirectionLimits::new(128, 128, 16384, 1000000),
                HtmlInertFactLimits::new(128, 1000000),
            ),
            &cancel(),
        )
        .unwrap();
    for (node, expected) in [
        (document.node(), true),
        (ids[0], true),
        (ids[2], false),
        (ids[1], true),
    ] {
        let report = captured
            .query(
                &[HtmlSemanticRequest::new(
                    captured.subject(node).unwrap(),
                    HtmlSemanticQuery::Inertness,
                )],
                surgeist_html::HtmlValueInterpreter::new(8192),
                HtmlSemanticLimits::new(1000000, 16, 1024, 64, 1000000, 1024),
                HtmlSemanticCancellation::new(),
            )
            .unwrap();
        assert_eq!(
            report.results()[0].fact(),
            &HtmlSemanticFact::Inertness(HtmlSemanticValue::Known(expected))
        );
    }
    // With only a non-Dialog raw member, neither its authored Hint attribute
    // nor the earlier Dialog's retained modal flag invents a blocking Dialog.
    let mut staged = draft(&dom);
    for id in [ids[0], ids[2]] {
        let key = staged.resolve(id.into()).unwrap();
        staged.remove_top_layer_immediately(key, &cancel()).unwrap();
    }
    publish(&mut dom, &staged);
    let current = dom
        .snapshot()
        .html_capture(
            document,
            None,
            HtmlCaptureLimits::new(64, 1024, 128, 16384, 64, 0, 0),
            crate::DomHtmlCaptureLimits::new(64, 1000000),
            &cancel(),
        )
        .unwrap();
    let report = current
        .query(
            &[HtmlSemanticRequest::new(
                current.subject(document.node()).unwrap(),
                HtmlSemanticQuery::Inertness,
            )],
            surgeist_html::HtmlValueInterpreter::new(8192),
            HtmlSemanticLimits::new(1000000, 16, 1024, 64, 1000000, 1024),
            HtmlSemanticCancellation::new(),
        )
        .unwrap();
    assert_eq!(
        report.results()[0].fact(),
        &HtmlSemanticFact::Inertness(HtmlSemanticValue::Known(false))
    );
}

#[test]
fn pending_growth_counts_replacement_overlap_and_one_short_preserves_the_old_bank() {
    for short in [true, false] {
        let (dom, document, _, ids) = positive_fixture();
        let mut staged = draft(&dom);
        let first = staged.resolve(ids[0].into()).unwrap();
        let last = staged.resolve(ids[2].into()).unwrap();
        staged.request_top_layer_removal(first, &cancel()).unwrap();
        let key = staged.resolve(document.node().into()).unwrap();
        let old = staged
            .staged_dialog_list(key, DialogList::Pending)
            .unwrap()
            .as_ptr();
        let boundary = transaction::queues::admission_observation::Binding::relation(
            key,
            transaction::RelationField::PendingTopLayerRemovals,
            2 * size_of::<Key>(),
            short,
        );
        let result = staged.request_top_layer_removal(last, &cancel());
        let limit = boundary.limit();
        drop(boundary);
        if short {
            assert_eq!(
                result,
                Err(DomError::ResourceLimit {
                    resource: DomResource::SourceRetainedBytes,
                    limit
                })
            );
            assert_eq!(
                staged.staged_dialog_list(key, DialogList::Pending).unwrap(),
                &[first]
            );
            assert_eq!(
                staged
                    .staged_dialog_list(key, DialogList::Pending)
                    .unwrap()
                    .as_ptr(),
                old
            );
            staged.source_limits = Some(limits(10_000_000));
            staged.request_top_layer_removal(last, &cancel()).unwrap();
        } else {
            result.unwrap();
        }
        assert_eq!(
            staged.staged_dialog_list(key, DialogList::Pending).unwrap(),
            &[first, last]
        );
    }
}
