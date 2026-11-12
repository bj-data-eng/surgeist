//! Pinned DOM removal and move semantics on the native Source.

use std::sync::atomic::AtomicBool;
use surgeist_dom::{
    DocumentId, Dom, DomBatch, DomChange, DomChildListObservationLimits, DomChildListRecord,
    DomError, DomLimits, DomNameLimits, DomNativeCheckpoint, DomNativeCheckpointLimits,
    DomNodeIteratorFilter, DomNodeIteratorProgress, DomResource, DomSelectCompletionStart,
    DomSelectTask, DomSelectTaskCancellationReason, DomSelectTaskOutcome, DomShadowRootInit,
    DomSlotQueryLimits, DomSnapshot, DomSourceEdit, DomSourceLimits, DomSourceProgress,
    HtmlShadowRootMode, Namespace, NodeId,
};

struct Tree {
    dom: Dom,
    document: DocumentId,
    root: NodeId,
    left: NodeId,
    right: NodeId,
    moved: NodeId,
    tail: NodeId,
    reference: NodeId,
    detached: NodeId,
}

fn limits() -> DomLimits {
    let names = DomNameLimits::new(16_777_216, 16_384, 4_000_000, 1_048_576).unwrap();
    let observation = DomChildListObservationLimits::new(4, 16, 16, 96).unwrap();
    DomLimits::new(64, 4096, 128, 64, 0, 64, 1_000_000, names)
        .unwrap()
        .with_child_list_observation(observation)
        .with_ranges(8)
        .unwrap()
        .with_node_iterators(8)
        .with_details_toggle_intentions(16)
        .unwrap()
        .with_select_processing(16, 4_000_000, 10_000_000)
        .unwrap()
}

fn tree(connected: bool, cancel: &AtomicBool) -> Tree {
    let mut dom = Dom::new(limits()).unwrap();
    let mut batch = DomBatch::new();
    let document = batch.create_document();
    let root = batch.create_element(&document, "root");
    let left = batch.create_element(&document, "left");
    let right = batch.create_element(&document, "right");
    let moved = batch.create_element(&document, "moved");
    let tail = batch.create_text(&document, "tail");
    let reference = batch.create_comment(&document, "reference");
    let detached = batch.create_element(&document, "detached");
    if connected {
        batch.insert_before(&root, &document, None);
    }
    batch.insert_before(&left, &root, None);
    batch.insert_before(&right, &root, None);
    batch.insert_before(&moved, &left, None);
    batch.insert_before(&tail, &left, None);
    batch.insert_before(&reference, &right, None);
    let commit = dom.apply(batch, cancel).unwrap();
    Tree {
        dom,
        document: commit.document(&document).unwrap(),
        root: commit.created(&root).unwrap().unwrap(),
        left: commit.created(&left).unwrap().unwrap(),
        right: commit.created(&right).unwrap().unwrap(),
        moved: commit.created(&moved).unwrap().unwrap(),
        tail: commit.created(&tail).unwrap().unwrap(),
        reference: commit.created(&reference).unwrap().unwrap(),
        detached: commit.created(&detached).unwrap().unwrap(),
    }
}

fn policy() -> DomSourceLimits {
    DomSourceLimits::new(64_000_000, 1_000_000, 20_000_000, 128, 4096, 4096, 8)
}

fn checkpoint() -> DomNativeCheckpointLimits {
    DomNativeCheckpointLimits::new(
        4_000_000,
        1_000_000,
        DomSlotQueryLimits::new(64, 64).unwrap(),
    )
    .unwrap()
}

fn move_node(
    dom: &mut Dom,
    node: NodeId,
    parent: NodeId,
    before: Option<NodeId>,
    cancel: &AtomicBool,
) -> Result<(), DomError> {
    let mut lease = dom.begin_source_edit(
        DomSourceEdit::move_before(node, parent, before),
        policy(),
        cancel,
    )?;
    match lease.advance(cancel)? {
        DomSourceProgress::Complete(done) => {
            assert_eq!(done.output(), None);
            Ok(())
        }
        DomSourceProgress::Failed(failure) => Err(failure.error().clone()),
        DomSourceProgress::Request(request) => panic!("unexpected move service: {request:?}"),
    }
}

fn assert_record(
    record: &DomChildListRecord,
    target: NodeId,
    added: &[NodeId],
    removed: &[NodeId],
    next: NodeId,
) {
    assert_eq!(record.target(), target);
    assert_eq!(record.added_nodes(), added);
    assert_eq!(record.removed_nodes(), removed);
    assert_eq!(record.previous_sibling(), None);
    assert_eq!(record.next_sibling(), Some(next));
}

#[test]
fn remove_child_checks_the_exact_parent_and_runs_canonical_removal() {
    let cancel = AtomicBool::new(false);
    let Tree {
        mut dom,
        document,
        left,
        right,
        moved,
        tail,
        detached,
        ..
    } = tree(true, &cancel);
    let foreign = tree(false, &cancel).moved;
    let mut batch = DomBatch::new();
    let point = batch.create_range(document);
    batch.set_range_start(&point, moved, 0);
    batch.set_range_end(&point, moved, 0);
    let commit = dom.apply(batch, &cancel).unwrap();
    let point = commit.created_range(&point).unwrap().unwrap();
    let observer = dom.create_child_list_observer(&cancel).unwrap();
    dom.observe_child_list(observer, left, false, &cancel)
        .unwrap();

    let before = dom.snapshot();
    for (parent, child, expected) in [
        (right, moved, DomError::NotFound),
        (left, detached, DomError::NotFound),
        (left, foreign, DomError::ForeignOwner),
    ] {
        let mut lease = dom
            .begin_source_edit(
                DomSourceEdit::remove_child(parent, child),
                policy(),
                &cancel,
            )
            .unwrap();
        let actual = match lease.advance(&cancel) {
            Ok(DomSourceProgress::Failed(failure)) => failure.error().clone(),
            Err(error) => error,
            other => panic!("expected checked removal refusal, got {other:?}"),
        };
        assert_eq!(actual, expected);
        assert_eq!(lease.snapshot().stamp(), before.stamp());
    }
    assert!(
        dom.take_child_list_records(observer, &cancel)
            .unwrap()
            .is_empty()
    );

    let mut lease = dom
        .begin_source_edit(DomSourceEdit::remove_child(left, moved), policy(), &cancel)
        .unwrap();
    assert!(matches!(
        lease.advance(&AtomicBool::new(true)),
        Err(DomError::Cancelled)
    ));
    assert_eq!(lease.snapshot().stamp(), before.stamp());
    let DomSourceProgress::Complete(done) = lease.advance(&cancel).unwrap() else {
        panic!("ordinary checked removal should complete");
    };
    assert_eq!(done.output(), None);
    let after = lease.snapshot();
    drop(lease);
    assert_eq!(before.node(moved).unwrap().parent(), Some(left));
    assert_eq!(after.node(moved).unwrap().id(), moved);
    assert_eq!(after.node(moved).unwrap().parent(), None);
    assert_eq!(after.node(left).unwrap().children(), [tail]);
    let point = after.range(point).unwrap();
    assert_eq!(
        (point.start().container(), point.start().offset()),
        (left, 0)
    );
    assert_eq!((point.end().container(), point.end().offset()), (left, 0));
    let records = dom.take_child_list_records(observer, &cancel).unwrap();
    assert_eq!(records.len(), 1);
    assert_record(&records[0], left, &[], &[moved], tail);
}

#[test]
fn move_keeps_identity_and_emits_ordered_records_within_a_shared_root() {
    let cancel = AtomicBool::new(false);
    for connected in [false, true] {
        let Tree {
            mut dom,
            root,
            left,
            right,
            moved,
            tail,
            reference,
            ..
        } = tree(connected, &cancel);
        let observer = dom.create_child_list_observer(&cancel).unwrap();
        dom.observe_child_list(observer, root, true, &cancel)
            .unwrap();
        let before = dom.snapshot();
        let nodes = dom.usage().managed_nodes();

        move_node(&mut dom, moved, left, Some(moved), &cancel).unwrap();
        assert_eq!(dom.snapshot().node(left).unwrap().children(), [moved, tail]);
        let records = dom.take_child_list_records(observer, &cancel).unwrap();
        assert_eq!(records.len(), 2);
        assert_record(&records[0], left, &[], &[moved], tail);
        assert_record(&records[1], left, &[moved], &[], tail);

        move_node(&mut dom, moved, right, Some(reference), &cancel).unwrap();
        let after = dom.snapshot();
        assert_eq!(after.node(left).unwrap().children(), [tail]);
        assert_eq!(after.node(right).unwrap().children(), [moved, reference]);
        assert_eq!(after.node(moved).unwrap().parent(), Some(right));
        assert_eq!(before.node(moved).unwrap().parent(), Some(left));
        assert_eq!(dom.usage().managed_nodes(), nodes);
        let records = dom.take_child_list_records(observer, &cancel).unwrap();
        assert_eq!(records.len(), 2);
        assert_record(&records[0], left, &[], &[moved], tail);
        assert_record(&records[1], right, &[moved], &[], reference);
    }
}

#[test]
fn move_checks_root_and_ancestor_before_reference_then_document_hierarchy() {
    let cancel = AtomicBool::new(false);
    let Tree {
        mut dom,
        document,
        root,
        left,
        right,
        moved,
        tail,
        reference,
        detached,
    } = tree(true, &cancel);
    let before = dom.snapshot();
    let observer = dom.create_child_list_observer(&cancel).unwrap();
    dom.observe_child_list(observer, root, true, &cancel)
        .unwrap();
    let stamp = dom.snapshot().stamp();

    for (node, parent, before, expected) in [
        (detached, left, Some(reference), DomError::HierarchyRequest),
        (left, moved, Some(reference), DomError::HierarchyRequest),
        (tail, document.node(), Some(reference), DomError::NotFound),
        (
            tail,
            document.node(),
            Some(root),
            DomError::HierarchyRequest,
        ),
        (root, document.node(), None, DomError::HierarchyRequest),
        (moved, right, Some(tail), DomError::NotFound),
    ] {
        assert_eq!(
            move_node(&mut dom, node, parent, before, &cancel),
            Err(expected)
        );
        assert_eq!(dom.snapshot().stamp(), stamp);
    }
    assert_eq!(
        dom.snapshot().node(moved).unwrap().parent(),
        before.node(moved).unwrap().parent()
    );
    assert!(
        dom.take_child_list_records(observer, &cancel)
            .unwrap()
            .is_empty()
    );
}

#[test]
fn move_adjusts_ranges_and_iterator_at_the_old_position_before_inserting() {
    let cancel = AtomicBool::new(false);
    let Tree {
        mut dom,
        document,
        root,
        left,
        right,
        moved,
        tail,
        reference,
        ..
    } = tree(true, &cancel);
    let mut batch = DomBatch::new();
    let removed_point = batch.create_range(document);
    batch.set_range_start(&removed_point, moved, 0);
    batch.set_range_end(&removed_point, moved, 0);
    let insertion_point = batch.create_range(document);
    batch.set_range_start(&insertion_point, right, 1);
    batch.set_range_end(&insertion_point, right, 1);
    let commit = dom.apply(batch, &cancel).unwrap();
    let removed_point = commit.created_range(&removed_point).unwrap().unwrap();
    let insertion_point = commit.created_range(&insertion_point).unwrap().unwrap();
    let iterator = dom
        .create_node_iterator(root, u32::MAX, DomNodeIteratorFilter::None, &cancel)
        .unwrap();
    for id in [root, left, moved] {
        assert!(matches!(
            dom.node_iterator_next(iterator, 256, &cancel).unwrap(),
            DomNodeIteratorProgress::Complete(Some(actual)) if actual == id
        ));
    }

    move_node(&mut dom, moved, right, Some(reference), &cancel).unwrap();
    let after = dom.snapshot();
    for (id, container, offset) in [(removed_point, left, 0), (insertion_point, right, 2)] {
        let range = after.range(id).unwrap();
        assert_eq!(
            (range.start().container(), range.start().offset()),
            (container, offset)
        );
        assert_eq!(
            (range.end().container(), range.end().offset()),
            (container, offset)
        );
    }
    let cursor = dom.node_iterator(iterator).unwrap();
    assert_eq!(cursor.reference_node(), left);
    assert!(!cursor.pointer_before_reference_node());
    assert!(matches!(
        dom.node_iterator_next(iterator, 256, &cancel).unwrap(),
        DomNodeIteratorProgress::Complete(Some(actual)) if actual == tail
    ));
}

#[test]
fn moving_between_light_and_shadow_trees_updates_slot_without_details_insertion() {
    let cancel = AtomicBool::new(false);
    let mut dom = Dom::new(limits()).unwrap();
    let mut batch = DomBatch::new();
    let document = batch.create_document();
    let host = batch.create_element_ns(&document, Namespace::html(), "div");
    let shadow = batch.attach_shadow(&host, DomShadowRootInit::new(HtmlShadowRootMode::Open));
    let slot = batch.create_element_ns(&document, Namespace::html(), "slot");
    let light = batch.create_element_ns(&document, Namespace::html(), "details");
    let inner = batch.create_element_ns(&document, Namespace::html(), "details");
    batch.insert_before(&host, &document, None);
    batch.insert_before(&slot, &shadow, None);
    batch.insert_before(&inner, &shadow, None);
    batch.insert_before(&light, &host, None);
    for details in [&light, &inner] {
        batch.set_attribute(details, "name", "group");
        batch.set_attribute(details, "open", "");
    }
    let commit = dom.apply(batch, &cancel).unwrap();
    let slot = commit.created(&slot).unwrap().unwrap();
    let light = commit.created(&light).unwrap().unwrap();
    let inner = commit.created(&inner).unwrap().unwrap();
    let before = dom.snapshot();
    assert_eq!(before.slot(slot).unwrap().assigned_nodes(), [light]);
    assert!(matches!(
        dom.native_checkpoint_step(checkpoint(), &cancel).unwrap(),
        DomNativeCheckpoint::Slots(notifications) if notifications.targets() == [slot]
    ));

    move_node(&mut dom, light, slot, None, &cancel).unwrap();
    let after = dom.snapshot();
    assert!(after.slot(slot).unwrap().assigned_nodes().is_empty());
    assert_eq!(after.node(slot).unwrap().children(), [light]);
    for details in [light, inner] {
        let node = after.node(details).unwrap();
        assert!(node.is_connected());
        assert!(node.attribute(&"open".into()).is_some());
    }
    assert_eq!(before.slot(slot).unwrap().assigned_nodes(), [light]);
    assert!(matches!(
        dom.native_checkpoint_step(checkpoint(), &cancel).unwrap(),
        DomNativeCheckpoint::Slots(notifications) if notifications.targets() == [slot]
    ));
    assert!(matches!(
        dom.native_checkpoint_step(checkpoint(), &cancel).unwrap(),
        DomNativeCheckpoint::Idle
    ));
}

struct SelectTree {
    dom: Dom,
    root: NodeId,
    first: NodeId,
    second: NodeId,
    option: NodeId,
    target: NodeId,
}

fn select_tree(cancel: &AtomicBool) -> SelectTree {
    let mut dom = Dom::new(limits()).unwrap();
    let mut batch = DomBatch::new();
    let document = batch.create_document();
    let root = batch.create_element_ns(&document, Namespace::html(), "div");
    let first = batch.create_element_ns(&document, Namespace::html(), "select");
    let second = batch.create_element_ns(&document, Namespace::html(), "select");
    let option = batch.create_element_ns(&document, Namespace::html(), "option");
    let other_option = batch.create_element_ns(&document, Namespace::html(), "option");
    let target = batch.create_element_ns(&document, Namespace::html(), "selectedcontent");
    batch.insert_before(&first, &root, None);
    batch.insert_before(&second, &root, None);
    batch.insert_before(&target, &root, None);
    batch.insert_before(&option, &first, None);
    batch.insert_before(&other_option, &second, None);
    for (node, value) in [
        (&option, "first"),
        (&other_option, "second"),
        (&target, "original"),
    ] {
        let text = batch.create_text(&document, value);
        batch.insert_before(&text, node, None);
    }
    batch.set_attribute(&option, "selected", "");
    let commit = dom.apply(batch, cancel).unwrap();
    SelectTree {
        dom,
        root: commit.created(&root).unwrap().unwrap(),
        first: commit.created(&first).unwrap().unwrap(),
        second: commit.created(&second).unwrap().unwrap(),
        option: commit.created(&option).unwrap().unwrap(),
        target: commit.created(&target).unwrap().unwrap(),
    }
}

fn task(dom: &mut Dom, cancel: &AtomicBool) -> DomSelectTask {
    let DomNativeCheckpoint::Select(task) =
        dom.native_checkpoint_step(checkpoint(), cancel).unwrap()
    else {
        panic!("a native Select operation must be pending");
    };
    task
}

fn text(snapshot: &DomSnapshot, target: NodeId) -> surgeist_dom::DocumentString {
    let [text] = snapshot.node(target).unwrap().children() else {
        panic!("one Text child");
    };
    snapshot.node(*text).unwrap().data().unwrap().clone()
}

#[test]
fn option_move_queues_the_old_then_new_select_after_normalizing_both() {
    let cancel = AtomicBool::new(false);
    let SelectTree {
        mut dom,
        first,
        second,
        option,
        ..
    } = select_tree(&cancel);
    move_node(&mut dom, option, second, None, &cancel).unwrap();
    assert_eq!(
        dom.snapshot()
            .node(option)
            .unwrap()
            .as_option()
            .unwrap()
            .cached_select()
            .unwrap(),
        Some(second)
    );
    assert_eq!(dom.pending_native_tasks(), 2);
    for expected in [first, second] {
        let task = task(&mut dom, &cancel);
        assert_eq!(task.select(), expected);
        assert_eq!(task.selectedcontent(), None);
        assert_eq!(
            dom.complete_select_task(&task, checkpoint(), &cancel)
                .unwrap()
                .outcome(),
            DomSelectTaskOutcome::Updated
        );
    }
    assert_eq!(dom.pending_native_tasks(), 0);
}

#[test]
fn moved_selectedcontent_is_deferred_and_keeps_each_captured_select() {
    let cancel = AtomicBool::new(false);
    let SelectTree {
        mut dom,
        first,
        second,
        target,
        ..
    } = select_tree(&cancel);
    move_node(&mut dom, target, first, None, &cancel).unwrap();
    move_node(&mut dom, target, second, None, &cancel).unwrap();
    assert_eq!(text(&dom.snapshot(), target), "original".into());
    assert_eq!(dom.pending_native_tasks(), 2);

    let original = task(&mut dom, &cancel);
    assert_eq!(
        (original.select(), original.selectedcontent()),
        (first, Some(target))
    );
    let DomSelectCompletionStart::Source(mut lease) = dom
        .begin_select_completion(&original, checkpoint(), policy(), &cancel)
        .unwrap()
    else {
        panic!("live captured pair");
    };
    assert!(matches!(
        lease.advance(&cancel).unwrap(),
        DomSourceProgress::Complete(_)
    ));
    drop(lease);
    assert_eq!(dom.snapshot().node(target).unwrap().parent(), Some(second));
    assert_eq!(text(&dom.snapshot(), target), "first".into());

    let later = task(&mut dom, &cancel);
    assert_eq!(
        (later.select(), later.selectedcontent()),
        (second, Some(target))
    );
    assert_ne!(original.operation(), later.operation());
    dom.complete_select_task(&later, checkpoint(), &cancel)
        .unwrap();
    assert_eq!(text(&dom.snapshot(), target), "second".into());
    assert_eq!(dom.pending_native_tasks(), 0);
}

#[test]
fn retiring_either_captured_node_cancels_the_exact_issued_selectedcontent_task() {
    let cancel = AtomicBool::new(false);
    for retire_select in [false, true] {
        let SelectTree {
            mut dom,
            root,
            first,
            target,
            ..
        } = select_tree(&cancel);
        move_node(&mut dom, target, first, None, &cancel).unwrap();
        let issued = task(&mut dom, &cancel);
        let mut retire = DomBatch::new();
        if retire_select {
            retire.insert_before(target, root, None);
        }
        let dead = if retire_select { first } else { target };
        retire.detach(dead);
        retire.destroy(dead);
        let retired = dom.apply(retire, &cancel).unwrap();
        assert!(
            retired
                .changes()
                .changes()
                .contains(&DomChange::SelectedcontentTaskCancelled {
                    operation: issued.operation(),
                    select: first,
                    target,
                    reason: DomSelectTaskCancellationReason::TargetDestroyed,
                })
        );
        assert!(issued.source().node(first).is_ok());
        assert!(issued.source().node(target).is_ok());
        let survivor = if retire_select { target } else { first };
        assert!(dom.snapshot().node(survivor).is_ok());
        assert_eq!(
            dom.complete_select_task(&issued, checkpoint(), &cancel)
                .unwrap()
                .outcome(),
            DomSelectTaskOutcome::SkippedTargetDestroyed
        );
        assert!(matches!(
            dom.complete_select_task(&issued, checkpoint(), &cancel),
            Err(DomError::InvalidNativeTask)
        ));
    }
}

#[test]
fn a_move_propagates_old_subtree_observation_until_its_notification() {
    let cancel = AtomicBool::new(false);
    let Tree {
        mut dom,
        document,
        left,
        right,
        moved,
        tail,
        ..
    } = tree(true, &cancel);
    let observer = dom.create_child_list_observer(&cancel).unwrap();
    dom.observe_child_list(observer, left, true, &cancel)
        .unwrap();
    move_node(&mut dom, moved, right, None, &cancel).unwrap();
    let records = dom.take_child_list_records(observer, &cancel).unwrap();
    assert_eq!(records.len(), 1);
    assert_record(&records[0], left, &[], &[moved], tail);

    let mut append = DomBatch::new();
    let first = append.create_text(document, "old observer still follows this");
    append.insert_before(&first, moved, None);
    let first = dom
        .apply(append, &cancel)
        .unwrap()
        .created(&first)
        .unwrap()
        .unwrap();
    let DomNativeCheckpoint::ChildList(delivery) =
        dom.native_checkpoint_step(checkpoint(), &cancel).unwrap()
    else {
        panic!("transient registration delivers its actual descendant record");
    };
    assert_eq!(delivery.observer(), observer);
    let [record] = delivery.records() else {
        panic!("one queued record");
    };
    assert_eq!(record.target(), moved);
    assert_eq!(record.added_nodes(), [first]);
    drop(delivery);
    assert!(matches!(
        dom.native_checkpoint_step(checkpoint(), &cancel).unwrap(),
        DomNativeCheckpoint::Idle
    ));

    let mut append = DomBatch::new();
    let second = append.create_text(document, "cleanup removed the old registration");
    append.insert_before(&second, moved, None);
    dom.apply(append, &cancel).unwrap();
    assert!(
        dom.take_child_list_records(observer, &cancel)
            .unwrap()
            .is_empty()
    );
}

#[test]
fn refusing_the_second_move_record_leaves_the_tree_and_first_record_private() {
    let cancel = AtomicBool::new(false);
    let Tree {
        mut dom,
        root,
        left,
        right,
        moved,
        tail,
        reference,
        ..
    } = tree(true, &cancel);
    let observer = dom.create_child_list_observer(&cancel).unwrap();
    dom.observe_child_list(observer, root, true, &cancel)
        .unwrap();
    let observation = |records| DomChildListObservationLimits::new(4, 16, records, 96).unwrap();
    dom.set_limits(dom.limits().with_child_list_observation(observation(1)))
        .unwrap();
    let before = dom.snapshot();
    let mut lease = dom
        .begin_source_edit(
            DomSourceEdit::move_before(moved, right, Some(reference)),
            policy(),
            &cancel,
        )
        .unwrap();
    assert!(matches!(
        lease.advance(&cancel),
        Err(DomError::ResourceLimit {
            resource: DomResource::ChildListRecords,
            limit: 1
        })
    ));
    assert!(!lease.status().has_published());
    assert_eq!(lease.snapshot().stamp(), before.stamp());
    drop(lease);
    assert!(dom.pending_source_edit().is_none());
    assert_eq!(dom.pending_native_tasks(), 0);
    assert!(
        dom.take_child_list_records(observer, &cancel)
            .unwrap()
            .is_empty()
    );

    dom.set_limits(dom.limits().with_child_list_observation(observation(2)))
        .unwrap();
    let mut lease = dom
        .begin_source_edit(
            DomSourceEdit::move_before(moved, right, Some(reference)),
            policy(),
            &cancel,
        )
        .unwrap();
    let canceled = AtomicBool::new(true);
    assert!(matches!(lease.advance(&canceled), Err(DomError::Cancelled)));
    assert_eq!(lease.snapshot().stamp(), before.stamp());
    assert!(matches!(
        lease.advance(&cancel).unwrap(),
        DomSourceProgress::Complete(_)
    ));
    drop(lease);
    let records = dom.take_child_list_records(observer, &cancel).unwrap();
    assert_eq!(records.len(), 2);
    assert_record(&records[0], left, &[], &[moved], tail);
    assert_record(&records[1], right, &[moved], &[], reference);
}

#[test]
fn an_image_move_uses_the_old_picture_and_does_not_run_picture_insertion() {
    let cancel = AtomicBool::new(false);
    let mut dom = Dom::new(limits()).unwrap();
    let mut batch = DomBatch::new();
    let document = batch.create_document();
    let root = batch.create_element_ns(&document, Namespace::html(), "div");
    let first = batch.create_element_ns(&document, Namespace::html(), "picture");
    let second = batch.create_element_ns(&document, Namespace::html(), "picture");
    let image = batch.create_element_ns(&document, Namespace::html(), "img");
    batch.insert_before(&first, &root, None);
    batch.insert_before(&second, &root, None);
    batch.insert_before(&image, &first, None);
    let commit = dom.apply(batch, &cancel).unwrap();
    let root = commit.created(&root).unwrap().unwrap();
    let second = commit.created(&second).unwrap().unwrap();
    let image = commit.created(&image).unwrap().unwrap();
    let update = |dom: &Dom| {
        dom.snapshot()
            .node(image)
            .unwrap()
            .as_img()
            .unwrap()
            .suspended_update()
            .unwrap()
            .unwrap()
            .id()
    };
    let inserted = update(&dom);
    move_node(&mut dom, image, root, None, &cancel).unwrap();
    let moved_from_picture = update(&dom);
    assert_ne!(inserted, moved_from_picture);
    move_node(&mut dom, image, second, None, &cancel).unwrap();
    assert_eq!(update(&dom), moved_from_picture);
}

#[test]
fn moving_id_bearing_form_and_control_update_the_actual_ordinary_tree_owner() {
    let cancel = AtomicBool::new(false);
    let mut dom = Dom::new(limits()).unwrap();
    let mut batch = DomBatch::new();
    let document = batch.create_document();
    let root = batch.create_element_ns(&document, Namespace::html(), "div");
    let host = batch.create_element_ns(&document, Namespace::html(), "div");
    let shadow = batch.attach_shadow(&host, DomShadowRootInit::new(HtmlShadowRootMode::Open));
    let first = batch.create_element_ns(&document, Namespace::html(), "form");
    let second = batch.create_element_ns(&document, Namespace::html(), "form");
    let input = batch.create_element_ns(&document, Namespace::html(), "input");
    batch.set_attribute(&first, "id", "owner");
    batch.set_attribute(&second, "id", "owner");
    batch.set_attribute(&input, "form", "owner");
    batch.insert_before(&root, &document, None);
    for child in [&first, &second, &input, &host] {
        batch.insert_before(child, &root, None);
    }
    let commit = dom.apply(batch, &cancel).unwrap();
    let root = commit.created(&root).unwrap().unwrap();
    let shadow = commit.attached_shadow_root(&shadow).unwrap().unwrap();
    let first = commit.created(&first).unwrap().unwrap();
    let second = commit.created(&second).unwrap().unwrap();
    let input = commit.created(&input).unwrap().unwrap();
    let owner = |dom: &Dom| {
        dom.snapshot()
            .node(input)
            .unwrap()
            .as_input()
            .unwrap()
            .form_owner()
            .unwrap()
    };
    assert_eq!(owner(&dom), Some(first));
    move_node(&mut dom, first, root, None, &cancel).unwrap();
    assert_eq!(owner(&dom), Some(second));
    move_node(&mut dom, input, shadow, None, &cancel).unwrap();
    assert_eq!(owner(&dom), None);
    assert!(dom.snapshot().node(input).unwrap().is_connected());
}

#[test]
fn moving_hooks_visit_shadow_descendants_before_light_descendants() {
    let cancel = AtomicBool::new(false);
    let mut dom = Dom::new(limits()).unwrap();
    let mut batch = DomBatch::new();
    let document = batch.create_document();
    let root = batch.create_element_ns(&document, Namespace::html(), "div");
    let host = batch.create_element_ns(&document, Namespace::html(), "div");
    let destination = batch.create_element_ns(&document, Namespace::html(), "div");
    let shadow = batch.attach_shadow(&host, DomShadowRootInit::new(HtmlShadowRootMode::Open));
    let shadow_select = batch.create_element_ns(&document, Namespace::html(), "select");
    let light_select = batch.create_element_ns(&document, Namespace::html(), "select");
    let shadow_target = batch.create_element_ns(&document, Namespace::html(), "selectedcontent");
    let light_target = batch.create_element_ns(&document, Namespace::html(), "selectedcontent");
    batch.insert_before(&host, &root, None);
    batch.insert_before(&destination, &root, None);
    batch.insert_before(&shadow_select, &shadow, None);
    batch.insert_before(&light_select, &host, None);
    batch.insert_before(&shadow_target, &shadow_select, None);
    batch.insert_before(&light_target, &light_select, None);
    let commit = dom.apply(batch, &cancel).unwrap();
    let host = commit.created(&host).unwrap().unwrap();
    let destination = commit.created(&destination).unwrap().unwrap();
    let resolve = |ticket| commit.created(ticket).unwrap().unwrap();

    move_node(&mut dom, host, destination, None, &cancel).unwrap();
    assert_eq!(dom.pending_native_tasks(), 2);
    for (select, target) in [
        (&shadow_select, &shadow_target),
        (&light_select, &light_target),
    ] {
        let task = task(&mut dom, &cancel);
        assert_eq!(
            (task.select(), task.selectedcontent()),
            (resolve(select), Some(resolve(target)))
        );
        dom.complete_select_task(&task, checkpoint(), &cancel)
            .unwrap();
    }
}

#[test]
fn moving_a_source_into_unloaded_media_refuses_the_actual_resource_selection() {
    let cancel = AtomicBool::new(false);
    let mut dom = Dom::new(limits()).unwrap();
    let mut batch = DomBatch::new();
    let document = batch.create_document();
    let root = batch.create_element_ns(&document, Namespace::html(), "div");
    let source = batch.create_element_ns(&document, Namespace::html(), "source");
    let media = batch.create_element_ns(&document, Namespace::html(), "video");
    batch.insert_before(&source, &root, None);
    batch.insert_before(&media, &root, None);
    let commit = dom.apply(batch, &cancel).unwrap();
    let root = commit.created(&root).unwrap().unwrap();
    let source = commit.created(&source).unwrap().unwrap();
    let media = commit.created(&media).unwrap().unwrap();
    let observer = dom.create_child_list_observer(&cancel).unwrap();
    dom.observe_child_list(observer, root, true, &cancel)
        .unwrap();
    let before = dom.snapshot();
    assert!(matches!(
        move_node(&mut dom, source, media, None, &cancel),
        Err(DomError::MediaResourceEffectUnavailable(
            surgeist_html::elements::media::HtmlMediaResourceEffect::SelectResource,
        ))
    ));
    assert_eq!(dom.snapshot().stamp(), before.stamp());
    assert_eq!(
        dom.snapshot().node(root).unwrap().children(),
        [source, media]
    );
    assert!(
        dom.take_child_list_records(observer, &cancel)
            .unwrap()
            .is_empty()
    );
    assert_eq!(dom.pending_native_tasks(), 0);
}
