//! Native cursor transitions from the pinned DOM NodeIterator algorithm.

use std::sync::atomic::AtomicBool;
use surgeist_dom::{
    DocumentId, Dom, DomBatch, DomChange, DomError, DomLimits, DomNameLimits,
    DomNodeFilterDecision, DomNodeIteratorFilter, DomNodeIteratorFilterRequest, DomNodeIteratorId,
    DomNodeIteratorProgress, DomResource, DomSourceEdit, DomSourceEventExecutor,
    DomSourceEventOutcome, DomSourceEventRequest, DomSourceLimits, DomSourceProgress,
    DomSourceRequest, DomStyleInlineCspObservation, Namespace, NodeId,
};

struct Tree {
    dom: Dom,
    second_document: DocumentId,
    root: NodeId,
    first: NodeId,
    removed: NodeId,
    nested: NodeId,
    last: NodeId,
}

fn tree() -> Tree {
    let names = DomNameLimits::new(16_777_216, 16_384, 4_000_000, 1_048_576).unwrap();
    let limits = DomLimits::new(32, 4096, 64, 0, 0, 8, 1_000_000, names)
        .unwrap()
        .with_node_iterators(8);
    let mut dom = Dom::new(limits).unwrap();
    let mut batch = DomBatch::new();
    let document = batch.create_document();
    let second_document = batch.create_document();
    let root = batch.create_element_ns(&document, Namespace::html(), "div");
    let first = batch.create_element_ns(&document, Namespace::html(), "p");
    let removed = batch.create_element_ns(&document, Namespace::html(), "section");
    let nested = batch.create_text(&document, "nested");
    let last = batch.create_comment(&document, "last");
    batch.insert_before(&root, &document, None);
    batch.insert_before(&first, &root, None);
    batch.insert_before(&removed, &root, None);
    batch.insert_before(&nested, &removed, None);
    batch.insert_before(&last, &root, None);
    let commit = dom.apply(batch, &AtomicBool::new(false)).unwrap();
    Tree {
        dom,
        second_document: commit.document(&second_document).unwrap(),
        root: commit.created(&root).unwrap().unwrap(),
        first: commit.created(&first).unwrap().unwrap(),
        removed: commit.created(&removed).unwrap().unwrap(),
        nested: commit.created(&nested).unwrap().unwrap(),
        last: commit.created(&last).unwrap().unwrap(),
    }
}

fn iterator(dom: &mut Dom, root: NodeId, mask: u32) -> DomNodeIteratorId {
    dom.create_node_iterator(
        root,
        mask,
        DomNodeIteratorFilter::None,
        &AtomicBool::new(false),
    )
    .unwrap()
}

fn step(dom: &mut Dom, id: DomNodeIteratorId, next: bool) -> Option<NodeId> {
    let cancel = AtomicBool::new(false);
    let result = if next {
        dom.node_iterator_next(id, 256, &cancel)
    } else {
        dom.node_iterator_previous(id, 256, &cancel)
    };
    let DomNodeIteratorProgress::Complete(result) = result.unwrap() else {
        panic!("the null filter cannot request a callback");
    };
    result
}

fn reference(dom: &Dom, id: DomNodeIteratorId) -> (NodeId, bool) {
    let cursor = dom.node_iterator(id).unwrap();
    (
        cursor.reference_node(),
        cursor.pointer_before_reference_node(),
    )
}

fn request(progress: DomNodeIteratorProgress, expected: NodeId) -> DomNodeIteratorFilterRequest {
    let DomNodeIteratorProgress::Filter(request) = progress else {
        panic!("expected a filter call for {expected:?}");
    };
    assert_eq!(request.node(), expected);
    request
}

fn begin_dialog_observation(
    dom: &mut Dom,
    root: NodeId,
    limits: DomSourceLimits,
    cancel: &AtomicBool,
) -> (DomSourceEventExecutor, DomSourceEventRequest) {
    let custody =
        surgeist_dom::DomDispatchCustodyLimits::new(64, 4096, 16_000_000, 16_000_000, 4_000_000)
            .unwrap();
    dom.set_limits(dom.limits().with_dispatch_custody(custody))
        .unwrap();
    let document = dom.snapshot().node(root).unwrap().owner_document().unwrap();
    let mut batch = DomBatch::new();
    let dialog = batch.create_element_ns(document, Namespace::html(), "dialog");
    batch.insert_before(&dialog, root, None);
    let dialog = dom
        .apply(batch, cancel)
        .unwrap()
        .created(&dialog)
        .unwrap()
        .unwrap();
    let executor = dom.bind_source_event_executor(cancel).unwrap();
    let mut lease = dom
        .begin_source_edit(DomSourceEdit::show_dialog(dialog), limits, cancel)
        .unwrap();
    let DomSourceProgress::Request(DomSourceRequest::Event(request)) =
        lease.advance(cancel).unwrap()
    else {
        panic!("showing the connected dialog must reach its native event");
    };
    (executor, request)
}

#[test]
fn empty_traversal_in_a_live_source_callback_spends_the_observers_work() {
    let Tree { mut dom, root, .. } = tree();
    let cancel = AtomicBool::new(false);
    let id = iterator(&mut dom, root, 0);
    let limits = DomSourceLimits::new(32_000_000, 1_000_000, 4096, 64, 4096, 4096, 4);
    let (executor, event) = begin_dialog_observation(&mut dom, root, limits, &cancel);
    let permit = executor.begin_dispatch(&dom, &event, &cancel).unwrap();
    let scope = executor.enter_dispatch(&dom, &permit, &cancel).unwrap();
    let before = dom.pending_source_edit().unwrap().spent_work();
    assert!(matches!(
        dom.node_iterator_next(id, 256, &cancel).unwrap(),
        DomNodeIteratorProgress::Complete(None)
    ));
    let after = dom.pending_source_edit().unwrap().spent_work();
    assert!(
        after > before,
        "the live Source observer must own read-only traversal work"
    );
    for _ in after..limits.work() - 1 {
        dom.source_event_step(&event, &cancel).unwrap();
    }
    assert_eq!(
        dom.node_iterator_next(id, 256, &cancel).unwrap_err(),
        DomError::SourceWorkLimit {
            limit: limits.work()
        }
    );
    assert_eq!(reference(&dom, id), (root, true));
    drop(scope);
    let resumed = DomSourceLimits::new(32_000_000, 1_000_000, 1_000_000, 64, 4096, 4096, 4);
    dom.raise_source_event_limits(&event, resumed, &cancel)
        .unwrap();
    let mut permit = Some(permit);
    let mut outcome = Some(
        executor
            .seal_completed(&dom, &mut permit, DomSourceEventOutcome::Canceled, &cancel)
            .unwrap(),
    );
    let mut lease = dom
        .resume_source_edit(event.operation(), resumed, &cancel)
        .unwrap();
    lease
        .accept_source_event(&event, &mut outcome, &cancel)
        .unwrap();
    assert!(matches!(
        lease.advance(&cancel).unwrap(),
        DomSourceProgress::Complete(_)
    ));
}

#[test]
fn filter_response_cannot_outlive_its_original_source_callback() {
    let Tree { mut dom, root, .. } = tree();
    let cancel = AtomicBool::new(false);
    let id = dom
        .create_node_iterator(root, u32::MAX, DomNodeIteratorFilter::Callback, &cancel)
        .unwrap();
    let limits = DomSourceLimits::new(32_000_000, 1_000_000, 1_000_000, 64, 4096, 4096, 4);
    let (executor, event) = begin_dialog_observation(&mut dom, root, limits, &cancel);
    let permit = executor.begin_dispatch(&dom, &event, &cancel).unwrap();
    let scope = executor.enter_dispatch(&dom, &permit, &cancel).unwrap();
    let filter = request(dom.node_iterator_next(id, 256, &cancel).unwrap(), root);
    drop(scope);
    let mut permit = Some(permit);
    let mut outcome = Some(
        executor
            .seal_completed(&dom, &mut permit, DomSourceEventOutcome::Canceled, &cancel)
            .unwrap(),
    );
    let mut lease = dom
        .resume_source_edit(event.operation(), limits, &cancel)
        .unwrap();
    lease
        .accept_source_event(&event, &mut outcome, &cancel)
        .unwrap();
    assert!(matches!(
        lease.advance(&cancel).unwrap(),
        DomSourceProgress::Complete(_)
    ));
    drop(lease);
    assert_eq!(
        filter
            .respond(&mut dom, DomNodeFilterDecision::Accept, &cancel)
            .unwrap_err(),
        DomError::InvalidSourceEvent
    );
    assert_eq!(reference(&dom, id), (root, true));
    let filter = request(dom.node_iterator_next(id, 256, &cancel).unwrap(), root);
    drop(filter);
}

#[test]
fn traverse_checks_the_mask_without_pruning_children_and_keeps_boundary_position() {
    let Tree {
        mut dom,
        root,
        nested,
        ..
    } = tree();
    const SHOW_TEXT: u32 = 1 << 2;
    let id = iterator(&mut dom, root, SHOW_TEXT);
    assert_eq!(reference(&dom, id), (root, true));
    assert_eq!(step(&mut dom, id, false), None);
    assert_eq!(step(&mut dom, id, true), Some(nested));
    assert_eq!(reference(&dom, id), (nested, false));
    assert_eq!(step(&mut dom, id, true), None);
    assert_eq!(reference(&dom, id), (nested, false));
    assert_eq!(step(&mut dom, id, false), Some(nested));
    assert_eq!(step(&mut dom, id, false), None);
    assert_eq!(reference(&dom, id), (nested, true));
}

#[test]
fn pre_remove_relocates_both_pointer_directions_and_preserves_a_contained_root() {
    let Tree {
        mut dom,
        second_document,
        root,
        first,
        removed,
        nested,
        last,
        ..
    } = tree();
    let before = iterator(&mut dom, root, u32::MAX);
    let after = iterator(&mut dom, root, u32::MAX);
    let inside = iterator(&mut dom, removed, u32::MAX);
    let unrelated = iterator(&mut dom, second_document.node(), u32::MAX);
    for expected in [root, first, removed, nested] {
        assert_eq!(step(&mut dom, before, true), Some(expected));
        assert_eq!(step(&mut dom, after, true), Some(expected));
    }
    assert_eq!(step(&mut dom, before, false), Some(nested));
    assert_eq!(step(&mut dom, inside, true), Some(removed));
    assert_eq!(step(&mut dom, inside, true), Some(nested));

    let mut batch = DomBatch::new();
    batch.detach(removed);
    dom.apply(batch, &AtomicBool::new(false)).unwrap();
    assert_eq!(reference(&dom, before), (last, true));
    assert_eq!(reference(&dom, after), (first, false));
    assert_eq!(reference(&dom, inside), (nested, false));
    assert_eq!(reference(&dom, unrelated), (second_document.node(), true));
    assert_eq!(step(&mut dom, before, true), Some(last));
    assert_eq!(step(&mut dom, after, true), Some(last));
}

#[test]
fn adoption_preserves_root_and_uses_its_new_document_for_later_removals() {
    let Tree {
        mut dom,
        second_document,
        root,
        first,
        removed,
        nested,
        last,
    } = tree();
    let id = iterator(&mut dom, root, u32::MAX);
    for expected in [root, first, removed, nested, last] {
        assert_eq!(step(&mut dom, id, true), Some(expected));
    }
    assert_eq!(step(&mut dom, id, false), Some(last));

    let mut adoption = DomBatch::new();
    adoption.adopt(root, second_document);
    dom.apply(adoption, &AtomicBool::new(false)).unwrap();
    assert_eq!(dom.node_iterator(id).unwrap().root(), root);
    assert_eq!(reference(&dom, id), (last, true));

    let mut detach = DomBatch::new();
    detach.detach(last);
    dom.apply(detach, &AtomicBool::new(false)).unwrap();
    assert_eq!(reference(&dom, id), (nested, false));
    assert_eq!(step(&mut dom, id, false), Some(nested));
}

#[test]
fn accepting_a_removed_candidate_returns_the_original_callback_argument() {
    let Tree {
        mut dom,
        root,
        first,
        removed,
        nested,
        last,
        ..
    } = tree();
    let cancel = AtomicBool::new(false);
    let id = dom
        .create_node_iterator(
            root,
            1 << 2 | 1 << 7,
            DomNodeIteratorFilter::Callback,
            &cancel,
        )
        .unwrap();
    let revision = dom.revision();
    let filter = request(dom.node_iterator_next(id, 256, &cancel).unwrap(), nested);
    assert_eq!(filter.iterator(), id);
    assert_eq!(dom.revision(), revision);
    assert_eq!(reference(&dom, id), (root, true));
    assert_eq!(
        dom.node_iterator_next(id, 256, &cancel).unwrap_err(),
        DomError::NodeIteratorFilterActive
    );
    assert!(matches!(
        dom.node_iterator_previous(id, 256, &cancel).unwrap(),
        DomNodeIteratorProgress::Complete(None)
    ));

    let mut detach = DomBatch::new();
    detach.detach(removed);
    let commit = dom.apply(detach, &cancel).unwrap();
    assert!(
        !commit
            .changes()
            .changes()
            .contains(&DomChange::NodeIteratorReference(id))
    );
    assert_eq!(reference(&dom, id), (root, true));
    assert!(
        matches!(filter.respond(&mut dom, DomNodeFilterDecision::Accept, &cancel).unwrap(), DomNodeIteratorProgress::Complete(Some(node)) if node == nested)
    );
    assert_eq!(reference(&dom, id), (first, false));

    let filter = request(dom.node_iterator_next(id, 256, &cancel).unwrap(), last);
    assert!(matches!(
        filter
            .respond(&mut dom, DomNodeFilterDecision::Skip, &cancel)
            .unwrap(),
        DomNodeIteratorProgress::Complete(None)
    ));
    assert_eq!(reference(&dom, id), (first, false));
}

#[test]
fn removing_a_subtree_relocates_the_committed_and_backward_candidate_independently() {
    let Tree {
        mut dom,
        root,
        first,
        removed,
        nested,
        last,
        ..
    } = tree();
    let cancel = AtomicBool::new(false);
    let id = dom
        .create_node_iterator(
            root,
            1 << 2 | 1 << 7,
            DomNodeIteratorFilter::Callback,
            &cancel,
        )
        .unwrap();
    let filter = request(dom.node_iterator_next(id, 256, &cancel).unwrap(), nested);
    assert!(
        matches!(filter.respond(&mut dom, DomNodeFilterDecision::Accept, &cancel).unwrap(), DomNodeIteratorProgress::Complete(Some(node)) if node == nested)
    );
    let filter = request(
        dom.node_iterator_previous(id, 256, &cancel).unwrap(),
        nested,
    );
    assert_eq!(reference(&dom, id), (nested, false));

    let mut detach = DomBatch::new();
    detach.detach(removed);
    dom.apply(detach, &cancel).unwrap();
    assert_eq!(reference(&dom, id), (first, false));
    assert!(
        matches!(filter.respond(&mut dom, DomNodeFilterDecision::Accept, &cancel).unwrap(), DomNodeIteratorProgress::Complete(Some(node)) if node == nested)
    );
    assert_eq!(reference(&dom, id), (last, true));
    let filter = request(dom.node_iterator_next(id, 256, &cancel).unwrap(), last);
    assert!(
        matches!(filter.respond(&mut dom, DomNodeFilterDecision::Accept, &cancel).unwrap(), DomNodeIteratorProgress::Complete(Some(node)) if node == last)
    );
}

#[test]
fn reject_does_not_prune_and_abrupt_filter_exit_clears_the_active_call() {
    let Tree {
        mut dom,
        root,
        first,
        removed,
        nested,
        ..
    } = tree();
    let cancel = AtomicBool::new(false);
    let id = dom
        .create_node_iterator(root, u32::MAX, DomNodeIteratorFilter::Callback, &cancel)
        .unwrap();
    let filter = request(dom.node_iterator_next(id, 256, &cancel).unwrap(), root);
    let filter = request(
        filter
            .respond(&mut dom, DomNodeFilterDecision::Reject, &cancel)
            .unwrap(),
        first,
    );
    let filter = request(
        filter
            .respond(&mut dom, DomNodeFilterDecision::Skip, &cancel)
            .unwrap(),
        removed,
    );
    let filter = request(
        filter
            .respond(&mut dom, DomNodeFilterDecision::Reject, &cancel)
            .unwrap(),
        nested,
    );
    drop(filter);
    assert_eq!(reference(&dom, id), (root, true));
    let filter = request(dom.node_iterator_next(id, 256, &cancel).unwrap(), root);
    let cancelled = AtomicBool::new(true);
    assert_eq!(
        filter
            .respond(&mut dom, DomNodeFilterDecision::Accept, &cancelled)
            .unwrap_err(),
        DomError::Cancelled
    );
    assert_eq!(reference(&dom, id), (root, true));
    let filter = request(dom.node_iterator_next(id, 256, &cancel).unwrap(), root);
    assert!(
        matches!(filter.respond(&mut dom, DomNodeFilterDecision::Accept, &cancel).unwrap(), DomNodeIteratorProgress::Complete(Some(node)) if node == root)
    );
    assert_eq!(reference(&dom, id), (root, false));
}

#[test]
fn a_filter_response_spends_the_original_traversals_remaining_work() {
    let Tree { mut dom, root, .. } = tree();
    let cancel = AtomicBool::new(false);
    let id = dom
        .create_node_iterator(root, u32::MAX, DomNodeIteratorFilter::Callback, &cancel)
        .unwrap();
    let revision = dom.revision();
    // The initial advance, the node lookup and its one bank page cost three
    // units. Acceptance needs one more unit from the same allowance.
    let filter = request(dom.node_iterator_next(id, 3, &cancel).unwrap(), root);
    assert_eq!(
        filter
            .respond(&mut dom, DomNodeFilterDecision::Accept, &cancel)
            .unwrap_err(),
        DomError::NodeIteratorWorkLimit { limit: 3 }
    );
    assert_eq!(dom.revision(), revision);
    assert_eq!(reference(&dom, id), (root, true));
    let filter = request(dom.node_iterator_next(id, 4, &cancel).unwrap(), root);
    assert!(
        matches!(filter.respond(&mut dom, DomNodeFilterDecision::Accept, &cancel).unwrap(), DomNodeIteratorProgress::Complete(Some(node)) if node == root)
    );
    assert_eq!(reference(&dom, id), (root, false));
}

#[test]
fn cross_parent_and_same_parent_insertions_use_the_actual_old_tree() {
    let Tree {
        mut dom,
        root,
        first,
        removed,
        nested,
        last,
        ..
    } = tree();
    let cancel = AtomicBool::new(false);
    let id = iterator(&mut dom, root, u32::MAX);
    for expected in [root, first, removed, nested] {
        assert_eq!(step(&mut dom, id, true), Some(expected));
    }
    let mut reorder = DomBatch::new();
    reorder.insert_before(removed, root, Some(first.into()));
    let commit = dom.apply(reorder, &cancel).unwrap();
    assert!(
        commit
            .changes()
            .changes()
            .contains(&DomChange::NodeIteratorReference(id))
    );
    assert_eq!(reference(&dom, id), (first, false));
    assert_eq!(step(&mut dom, id, false), Some(first));
    assert_eq!(step(&mut dom, id, false), Some(nested));

    let mut reparent = DomBatch::new();
    reparent.insert_before(nested, first, None);
    dom.apply(reparent, &cancel).unwrap();
    assert_eq!(reference(&dom, id), (first, true));
    assert_eq!(step(&mut dom, id, true), Some(first));
    assert_eq!(step(&mut dom, id, true), Some(nested));
    assert_eq!(step(&mut dom, id, true), Some(last));
}

#[test]
fn retained_slot_admission_reuse_and_destroy_use_exact_owners_and_generations() {
    let Tree {
        mut dom,
        root,
        first,
        ..
    } = tree();
    let mut other = tree();
    let cancel = AtomicBool::new(false);
    dom.set_limits(dom.limits().with_node_iterators(1)).unwrap();
    let id = iterator(&mut dom, root, u32::MAX);
    let revision = dom.revision();
    assert_eq!(dom.usage().node_iterators(), 1);
    assert_eq!(other.dom.node_iterator(id), Err(DomError::ForeignOwner));
    assert_eq!(
        other
            .dom
            .create_node_iterator(root, u32::MAX, DomNodeIteratorFilter::None, &cancel),
        Err(DomError::ForeignOwner)
    );
    assert_eq!(
        dom.create_node_iterator(first, u32::MAX, DomNodeIteratorFilter::None, &cancel),
        Err(DomError::ResourceLimit {
            resource: DomResource::NodeIterators,
            limit: 1
        })
    );
    assert_eq!(dom.revision(), revision);
    assert_eq!(
        dom.node_iterator_next(id, 0, &cancel).unwrap_err(),
        DomError::NodeIteratorWorkLimit { limit: 0 }
    );
    assert_eq!(reference(&dom, id), (root, true));
    assert_eq!(step(&mut dom, id, true), Some(root));
    dom.retire_node_iterator(id, &cancel).unwrap();
    assert_eq!(dom.usage().node_iterators(), 0);
    let replacement = iterator(&mut dom, first, u32::MAX);
    assert_ne!(replacement, id);
    assert_eq!(dom.node_iterator(id), Err(DomError::StaleNodeIterator));
    let mut destroy = DomBatch::new();
    destroy.detach(first);
    destroy.destroy(first);
    let commit = dom.apply(destroy, &cancel).unwrap();
    assert!(
        commit
            .changes()
            .changes()
            .contains(&DomChange::NodeIteratorRetired(replacement))
    );
    assert_eq!(
        dom.node_iterator(replacement),
        Err(DomError::StaleNodeIterator)
    );
    assert_eq!(dom.usage().node_iterators(), 0);
}

#[test]
fn source_refusal_and_retry_keep_the_cursor_with_its_published_tree() {
    let Tree {
        mut dom,
        root,
        first,
        removed,
        nested,
        ..
    } = tree();
    let cancel = AtomicBool::new(false);
    let id = iterator(&mut dom, root, u32::MAX);
    for expected in [root, first, removed, nested] {
        assert_eq!(step(&mut dom, id, true), Some(expected));
    }
    let revision = dom.revision();
    let refused = DomSourceLimits::new(16_000_000, 100_000, 0, 128, 4096, 4096, 1);
    let error = match dom.begin_source_edit(DomSourceEdit::detach(removed), refused, &cancel) {
        Err(error) => error,
        Ok(mut lease) => {
            let error = lease.advance(&cancel).unwrap_err();
            assert!(!lease.status().has_published());
            error
        }
    };
    assert_eq!(error, DomError::SourceWorkLimit { limit: 0 });
    assert_eq!(dom.revision(), revision);
    assert_eq!(reference(&dom, id), (nested, false));
    assert_eq!(dom.snapshot().node(removed).unwrap().parent(), Some(root));

    let admitted = DomSourceLimits::new(16_000_000, 100_000, 1_000_000, 128, 4096, 4096, 1);
    let mut lease = dom
        .begin_source_edit(DomSourceEdit::detach(removed), admitted, &cancel)
        .unwrap();
    assert!(matches!(
        lease.advance(&cancel).unwrap(),
        DomSourceProgress::Complete(_)
    ));
    drop(lease);
    assert_eq!(reference(&dom, id), (first, false));
    assert_eq!(dom.snapshot().node(removed).unwrap().parent(), None);
}

#[test]
fn fragment_cursor_adjustments_survive_the_real_style_source_suspension() {
    let Tree { mut dom, root, .. } = tree();
    let cancel = AtomicBool::new(false);
    let mut setup = DomBatch::new();
    let document = dom.snapshot().node(root).unwrap().owner_document().unwrap();
    let fragment = setup.create_fragment(document);
    let style = setup.create_element_ns(document, Namespace::html(), "style");
    let text = setup.create_text(document, "p { color: red }");
    let sibling = setup.create_element_ns(document, Namespace::html(), "p");
    setup.insert_before(&text, &style, None);
    setup.insert_before(&style, &fragment, None);
    setup.insert_before(&sibling, &fragment, None);
    let commit = dom.apply(setup, &cancel).unwrap();
    let fragment = commit.created(&fragment).unwrap().unwrap();
    let style = commit.created(&style).unwrap().unwrap();
    let sibling = commit.created(&sibling).unwrap().unwrap();
    let id = iterator(&mut dom, fragment, u32::MAX);
    assert_eq!(step(&mut dom, id, true), Some(fragment));
    assert_eq!(step(&mut dom, id, true), Some(style));
    assert_eq!(step(&mut dom, id, false), Some(style));
    let limits = DomSourceLimits::new(16_000_000, 100_000, 1_000_000, 128, 4096, 4096, 1);

    let mut lease = dom
        .begin_source_edit(
            DomSourceEdit::insert_before(fragment, root, None),
            limits,
            &cancel,
        )
        .unwrap();
    let operation = lease.operation();
    let DomSourceProgress::Request(DomSourceRequest::Style(check)) =
        lease.advance(&cancel).unwrap()
    else {
        panic!("the connected Style must reach the real CSP boundary");
    };
    let ordinal = check.source().ordinal();
    assert_eq!(check.source().element(), style);
    drop(lease);
    let prefix = dom.snapshot();
    assert_eq!(prefix.node(fragment).unwrap().children(), []);
    assert_eq!(prefix.node(style).unwrap().parent(), Some(root));
    assert_eq!(reference(&dom, id), (fragment, false));

    let mut lease = dom.resume_source_edit(operation, limits, &cancel).unwrap();
    let DomSourceProgress::Request(DomSourceRequest::Style(check)) =
        lease.advance(&cancel).unwrap()
    else {
        panic!("resume must reach the retained CSP boundary");
    };
    assert_eq!(check.source().ordinal(), ordinal);
    lease.request_cancel();
    let denied = DomStyleInlineCspObservation::evaluated(
        check.source(),
        std::sync::Arc::new(vec!["style-src 'none'".into()]),
        19,
        false,
    );
    lease.accept_inline_csp(&check, &denied, &cancel).unwrap();
    let DomSourceProgress::Complete(done) = lease.advance(&cancel).unwrap() else {
        panic!("the original Source tail must settle");
    };
    assert!(done.cancellation_requested());
    drop(lease);
    assert_eq!(dom.snapshot().node(sibling).unwrap().parent(), Some(root));
    assert_eq!(reference(&dom, id), (fragment, false));
    assert_eq!(step(&mut dom, id, true), None);
}
