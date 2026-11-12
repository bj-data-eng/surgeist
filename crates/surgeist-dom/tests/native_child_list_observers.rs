//! Native child-list records from the pinned DOM tree and observer algorithms.

use std::sync::atomic::AtomicBool;
use surgeist_dom::{
    DocumentId, Dom, DomBatch, DomChildListObservationLimits, DomChildListObserverId,
    DomChildListRecord, DomError, DomLimits, DomNameLimits, DomNativeCheckpoint,
    DomNativeCheckpointLimits, DomResource, DomShadowRootInit, DomSlotQueryLimits, DomSourceEdit,
    DomSourceLimits, DomSourceProgress, DomSourceRequest, DomStyleInlineCspObservation,
    HtmlShadowRootMode, Namespace, NodeId,
};

struct Tree {
    dom: Dom,
    document: DocumentId,
    root: NodeId,
    first: NodeId,
    branch: NodeId,
    nested: NodeId,
    last: NodeId,
}

fn tree() -> Tree {
    let cancel = AtomicBool::new(false);
    let names = DomNameLimits::new(16_777_216, 16_384, 4_000_000, 1_048_576).unwrap();
    let observations = DomChildListObservationLimits::new(8, 32, 32, 256).unwrap();
    let limits = DomLimits::new(64, 4096, 128, 0, 0, 64, 1_000_000, names)
        .unwrap()
        .with_child_list_observation(observations);
    let mut dom = Dom::new(limits).unwrap();
    let mut batch = DomBatch::new();
    let document = batch.create_document();
    let root = batch.create_element_ns(&document, Namespace::html(), "div");
    let first = batch.create_element_ns(&document, Namespace::html(), "p");
    let branch = batch.create_element_ns(&document, Namespace::html(), "section");
    let nested = batch.create_element_ns(&document, Namespace::html(), "span");
    let last = batch.create_comment(&document, "last");
    batch.insert_before(&root, &document, None);
    batch.insert_before(&first, &root, None);
    batch.insert_before(&branch, &root, None);
    batch.insert_before(&nested, &branch, None);
    batch.insert_before(&last, &root, None);
    let commit = dom.apply(batch, &cancel).unwrap();
    Tree {
        dom,
        document: commit.document(&document).unwrap(),
        root: commit.created(&root).unwrap().unwrap(),
        first: commit.created(&first).unwrap().unwrap(),
        branch: commit.created(&branch).unwrap().unwrap(),
        nested: commit.created(&nested).unwrap().unwrap(),
        last: commit.created(&last).unwrap().unwrap(),
    }
}

fn observe(dom: &mut Dom, root: NodeId, subtree: bool) -> DomChildListObserverId {
    let cancel = AtomicBool::new(false);
    let id = dom.create_child_list_observer(&cancel).unwrap();
    dom.observe_child_list(id, root, subtree, &cancel).unwrap();
    id
}

fn assert_record(
    record: &DomChildListRecord,
    target: NodeId,
    added: &[NodeId],
    removed: &[NodeId],
    previous: Option<NodeId>,
    next: Option<NodeId>,
) {
    assert_eq!(record.target(), target);
    assert_eq!(record.added_nodes(), added);
    assert_eq!(record.removed_nodes(), removed);
    assert_eq!(record.previous_sibling(), previous);
    assert_eq!(record.next_sibling(), next);
}

#[test]
fn take_records_keeps_detached_subtree_registration_until_cleanup() {
    let Tree {
        mut dom,
        document,
        root,
        first,
        branch,
        nested,
        last,
    } = tree();
    let cancel = AtomicBool::new(false);
    let direct = observe(&mut dom, root, false);
    let subtree = observe(&mut dom, root, true);
    let mut batch = DomBatch::new();
    batch.detach(branch);
    dom.apply(batch, &cancel).unwrap();
    for observer in [direct, subtree] {
        let records = dom.take_child_list_records(observer, &cancel).unwrap();
        assert_eq!(records.len(), 1);
        assert_record(&records[0], root, &[], &[branch], Some(first), Some(last));
    }

    let mut batch = DomBatch::new();
    let text = batch.create_text(document, "detached");
    batch.insert_before(&text, nested, None);
    let text = dom
        .apply(batch, &cancel)
        .unwrap()
        .created(&text)
        .unwrap()
        .unwrap();
    assert!(
        dom.take_child_list_records(direct, &cancel)
            .unwrap()
            .is_empty()
    );
    let records = dom.take_child_list_records(subtree, &cancel).unwrap();
    assert_eq!(records.len(), 1);
    assert_record(&records[0], nested, &[text], &[], None, None);

    dom.disconnect_child_list_observer(subtree, &cancel)
        .unwrap();
    let mut batch = DomBatch::new();
    batch.detach(text);
    dom.apply(batch, &cancel).unwrap();
    assert!(
        dom.take_child_list_records(subtree, &cancel)
            .unwrap()
            .is_empty()
    );
}

#[test]
fn same_parent_insertion_reports_removal_then_insertion_with_old_and_new_neighbors() {
    let Tree {
        mut dom,
        root,
        first,
        branch,
        last,
        ..
    } = tree();
    let cancel = AtomicBool::new(false);
    let observer = observe(&mut dom, root, false);
    let mut batch = DomBatch::new();
    batch.insert_before(branch, root, None);
    dom.apply(batch, &cancel).unwrap();
    let records = dom.take_child_list_records(observer, &cancel).unwrap();
    assert_eq!(records.len(), 2);
    assert_record(&records[0], root, &[], &[branch], Some(first), Some(last));
    assert_record(&records[1], root, &[branch], &[], Some(last), None);
}

#[test]
fn fragment_transfer_reports_two_aggregate_records_in_source_order() {
    let Tree {
        mut dom,
        document,
        root,
        first,
        branch,
        ..
    } = tree();
    let cancel = AtomicBool::new(false);
    let mut batch = DomBatch::new();
    let fragment = batch.create_fragment(document);
    let one = batch.create_text(document, "one");
    let two = batch.create_comment(document, "two");
    batch.insert_before(&one, &fragment, None);
    batch.insert_before(&two, &fragment, None);
    let commit = dom.apply(batch, &cancel).unwrap();
    let fragment = commit.created(&fragment).unwrap().unwrap();
    let one = commit.created(&one).unwrap().unwrap();
    let two = commit.created(&two).unwrap().unwrap();

    let observer = observe(&mut dom, fragment, false);
    dom.observe_child_list(observer, root, false, &cancel)
        .unwrap();
    let mut batch = DomBatch::new();
    batch.insert_before(fragment, root, Some(branch.into()));
    dom.apply(batch, &cancel).unwrap();
    let records = dom.take_child_list_records(observer, &cancel).unwrap();
    assert_eq!(records.len(), 2);
    assert_record(&records[0], fragment, &[], &[one, two], None, None);
    assert_record(
        &records[1],
        root,
        &[one, two],
        &[],
        Some(first),
        Some(branch),
    );
}

#[test]
fn replace_child_emits_one_record_with_the_original_neighbors() {
    let Tree {
        mut dom,
        document,
        root,
        first,
        branch,
        last,
        ..
    } = tree();
    let cancel = AtomicBool::new(false);
    let observer = observe(&mut dom, root, false);
    let mut replace = DomBatch::new();
    let added = replace.create_text(document, "replacement");
    replace.replace_child(&added, root, branch);
    let added = dom
        .apply(replace, &cancel)
        .unwrap()
        .created(&added)
        .unwrap()
        .unwrap();
    let records = dom.take_child_list_records(observer, &cancel).unwrap();
    assert_eq!(records.len(), 1);
    assert_record(
        &records[0],
        root,
        &[added],
        &[branch],
        Some(first),
        Some(last),
    );
}

#[test]
fn text_content_replacement_records_all_removed_and_added_nodes_once() {
    let Tree {
        mut dom,
        root,
        first,
        branch,
        last,
        ..
    } = tree();
    let cancel = AtomicBool::new(false);
    let observer = observe(&mut dom, root, true);
    let limits = DomSourceLimits::new(32_000_000, 100_000, 1_000_000, 128, 4096, 4096, 4);
    let mut lease = dom
        .begin_source_edit(
            DomSourceEdit::set_text_content(root, "one"),
            limits,
            &cancel,
        )
        .unwrap();
    assert!(matches!(
        lease.advance(&cancel).unwrap(),
        DomSourceProgress::Complete(_)
    ));
    drop(lease);
    let snapshot = dom.snapshot();
    let [added] = snapshot.node(root).unwrap().children() else {
        panic!("one replacement Text");
    };
    let records = dom.take_child_list_records(observer, &cancel).unwrap();
    assert_eq!(records.len(), 1);
    assert_record(
        &records[0],
        root,
        &[*added],
        &[first, branch, last],
        None,
        None,
    );
}

fn checkpoint_limits(work: u64) -> DomNativeCheckpointLimits {
    DomNativeCheckpointLimits::new(16_000_000, work, DomSlotQueryLimits::new(64, 64).unwrap())
        .unwrap()
}

#[test]
fn notification_reads_each_observers_queue_when_reached_after_earlier_callback_edits() {
    let Tree {
        mut dom,
        root,
        first,
        branch,
        last,
        ..
    } = tree();
    let cancel = AtomicBool::new(false);
    let one = observe(&mut dom, root, false);
    let two = observe(&mut dom, root, false);
    let mut detach = DomBatch::new();
    detach.detach(branch);
    dom.apply(detach, &cancel).unwrap();
    let checkpoint = checkpoint_limits(1_000_000);
    let DomNativeCheckpoint::ChildList(delivery) =
        dom.native_checkpoint_step(checkpoint, &cancel).unwrap()
    else {
        panic!("first observer must be reached");
    };
    assert_eq!(delivery.observer(), one);
    assert_eq!(dom.pending_native_tasks(), 1);
    assert_eq!(delivery.records().len(), 1);
    assert_record(
        &delivery.records()[0],
        root,
        &[],
        &[branch],
        Some(first),
        Some(last),
    );

    let mut insert = DomBatch::new();
    insert.insert_before(branch, root, None);
    dom.apply(insert, &cancel).unwrap();
    assert_eq!(
        dom.native_checkpoint_step(checkpoint, &cancel).unwrap_err(),
        DomError::ChildListDeliveryActive
    );
    drop(delivery);

    let DomNativeCheckpoint::ChildList(delivery) =
        dom.native_checkpoint_step(checkpoint, &cancel).unwrap()
    else {
        panic!("second observer belongs to the original notification");
    };
    assert_eq!(delivery.observer(), two);
    assert_eq!(delivery.records().len(), 2);
    assert_record(
        &delivery.records()[0],
        root,
        &[],
        &[branch],
        Some(first),
        Some(last),
    );
    assert_record(
        &delivery.records()[1],
        root,
        &[branch],
        &[],
        Some(last),
        None,
    );
    drop(delivery);

    let DomNativeCheckpoint::ChildList(delivery) =
        dom.native_checkpoint_step(checkpoint, &cancel).unwrap()
    else {
        panic!("first observer is pending in the next notification");
    };
    assert_eq!(delivery.observer(), one);
    assert_eq!(delivery.records().len(), 1);
    assert_record(
        &delivery.records()[0],
        root,
        &[branch],
        &[],
        Some(last),
        None,
    );
    drop(delivery);
    assert!(matches!(
        dom.native_checkpoint_step(checkpoint, &cancel).unwrap(),
        DomNativeCheckpoint::Idle
    ));
}

#[test]
fn a_notification_without_records_still_removes_transient_registrations() {
    let Tree {
        mut dom,
        document,
        root,
        branch,
        nested,
        ..
    } = tree();
    let cancel = AtomicBool::new(false);
    let observer = observe(&mut dom, root, true);
    let mut detach = DomBatch::new();
    detach.detach(branch);
    dom.apply(detach, &cancel).unwrap();
    assert_eq!(
        dom.take_child_list_records(observer, &cancel)
            .unwrap()
            .len(),
        1
    );
    assert_eq!(
        dom.native_checkpoint_step(checkpoint_limits(0), &cancel)
            .unwrap_err(),
        DomError::ChildListWorkLimit { limit: 0 }
    );
    assert!(matches!(
        dom.native_checkpoint_step(checkpoint_limits(1_000_000), &cancel)
            .unwrap(),
        DomNativeCheckpoint::Idle
    ));

    let mut insert = DomBatch::new();
    let text = insert.create_text(document, "after cleanup");
    insert.insert_before(&text, nested, None);
    dom.apply(insert, &cancel).unwrap();
    assert!(
        dom.take_child_list_records(observer, &cancel)
            .unwrap()
            .is_empty()
    );
}

#[test]
fn record_limit_refuses_the_tree_edit_and_retry_keeps_exact_record_order() {
    let Tree {
        mut dom,
        document,
        nested,
        ..
    } = tree();
    let cancel = AtomicBool::new(false);
    let limits = DomChildListObservationLimits::new(1, 1, 1, 2).unwrap();
    dom.set_limits(dom.limits().with_child_list_observation(limits))
        .unwrap();
    let observer = observe(&mut dom, nested, false);
    let mut insert = DomBatch::new();
    let text = insert.create_text(document, "first");
    insert.insert_before(&text, nested, None);
    let text = dom
        .apply(insert, &cancel)
        .unwrap()
        .created(&text)
        .unwrap()
        .unwrap();
    let stamp = dom.snapshot().stamp();

    let mut detach = DomBatch::new();
    detach.detach(text);
    assert_eq!(
        dom.apply(detach, &cancel).unwrap_err(),
        DomError::ResourceLimit {
            resource: DomResource::ChildListRecords,
            limit: 1
        }
    );
    assert_eq!(dom.snapshot().stamp(), stamp);
    assert_eq!(dom.snapshot().node(text).unwrap().parent(), Some(nested));
    let records = dom.take_child_list_records(observer, &cancel).unwrap();
    assert_eq!(records.len(), 1);
    assert_record(&records[0], nested, &[text], &[], None, None);

    let mut detach = DomBatch::new();
    detach.detach(text);
    dom.apply(detach, &cancel).unwrap();
    let records = dom.take_child_list_records(observer, &cancel).unwrap();
    assert_eq!(records.len(), 1);
    assert_record(&records[0], nested, &[], &[text], None, None);
}

#[test]
fn records_retain_a_real_historical_identity_for_a_node_retired_in_the_same_batch() {
    let Tree {
        mut dom,
        document,
        nested,
        ..
    } = tree();
    let cancel = AtomicBool::new(false);
    let observer = observe(&mut dom, nested, false);
    let mut edit = DomBatch::new();
    let temporary = edit.create_text(document, "temporary");
    edit.insert_before(&temporary, nested, None);
    edit.detach(&temporary);
    edit.destroy(&temporary);
    let commit = dom.apply(edit, &cancel).unwrap();
    assert_eq!(commit.created(&temporary).unwrap(), None);
    let records = dom.take_child_list_records(observer, &cancel).unwrap();
    assert_eq!(records.len(), 2);
    let [historical] = records[0].added_nodes() else {
        panic!("one actual historical identity");
    };
    assert_record(&records[0], nested, &[*historical], &[], None, None);
    assert_record(&records[1], nested, &[], &[*historical], None, None);
    assert!(matches!(
        dom.snapshot().node(*historical),
        Err(DomError::StaleNode)
    ));
    let mut create = DomBatch::new();
    let next = create.create_text(document, "next generation");
    let next = dom
        .apply(create, &cancel)
        .unwrap()
        .created(&next)
        .unwrap()
        .unwrap();
    assert_ne!(next, *historical);
}

#[test]
fn retiring_a_pending_observer_unlinks_it_without_losing_the_next_generation_or_recipient() {
    let Tree {
        mut dom,
        root,
        first,
        branch,
        last,
        ..
    } = tree();
    let cancel = AtomicBool::new(false);
    let limits = DomChildListObservationLimits::new(2, 8, 8, 32).unwrap();
    dom.set_limits(dom.limits().with_child_list_observation(limits))
        .unwrap();
    let retired = observe(&mut dom, root, false);
    let surviving = observe(&mut dom, root, false);
    let mut detach = DomBatch::new();
    detach.detach(branch);
    dom.apply(detach, &cancel).unwrap();

    dom.retire_child_list_observer(retired, &cancel).unwrap();
    let replacement = observe(&mut dom, root, false);
    assert_ne!(replacement, retired);
    assert_eq!(
        dom.take_child_list_records(retired, &cancel).unwrap_err(),
        DomError::StaleChildListObserver
    );
    let mut foreign = tree().dom;
    assert_eq!(
        foreign.retire_child_list_observer(surviving, &cancel),
        Err(DomError::ForeignOwner)
    );
    let checkpoint = checkpoint_limits(1_000_000);
    let DomNativeCheckpoint::ChildList(delivery) =
        dom.native_checkpoint_step(checkpoint, &cancel).unwrap()
    else {
        panic!("only the surviving observer is pending");
    };
    assert_eq!(delivery.observer(), surviving);
    assert_eq!(delivery.records().len(), 1);
    assert_record(
        &delivery.records()[0],
        root,
        &[],
        &[branch],
        Some(first),
        Some(last),
    );
    drop(delivery);
    assert!(matches!(
        dom.native_checkpoint_step(checkpoint, &cancel).unwrap(),
        DomNativeCheckpoint::Idle
    ));
    assert!(
        dom.take_child_list_records(replacement, &cancel)
            .unwrap()
            .is_empty()
    );
}

#[test]
fn fragment_records_keep_their_order_across_the_real_style_source_boundary() {
    let Tree {
        mut dom,
        document,
        root,
        last,
        ..
    } = tree();
    let cancel = AtomicBool::new(false);
    let mut setup = DomBatch::new();
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
    let observer = observe(&mut dom, fragment, false);
    dom.observe_child_list(observer, root, false, &cancel)
        .unwrap();
    let limits = DomSourceLimits::new(32_000_000, 100_000, 1_000_000, 128, 4096, 4096, 4);
    let mut lease = dom
        .begin_source_edit(
            DomSourceEdit::insert_before(fragment, root, None),
            limits,
            &cancel,
        )
        .unwrap();
    let operation = lease.operation();
    let DomSourceProgress::Request(DomSourceRequest::Style(request)) =
        lease.advance(&cancel).unwrap()
    else {
        panic!("real inline Style CSP request");
    };
    assert_eq!(request.source().element(), style);
    let ordinal = request.source().ordinal();
    drop(lease);
    assert_eq!(
        dom.take_child_list_records(observer, &cancel).unwrap_err(),
        DomError::SourceOperationPending
    );
    let mut lease = dom.resume_source_edit(operation, limits, &cancel).unwrap();
    let DomSourceProgress::Request(DomSourceRequest::Style(request)) =
        lease.advance(&cancel).unwrap()
    else {
        panic!("original retained request");
    };
    assert_eq!(request.source().ordinal(), ordinal);
    lease.request_cancel();
    let denied = DomStyleInlineCspObservation::evaluated(
        request.source(),
        std::sync::Arc::new(vec!["style-src 'none'".into()]),
        19,
        false,
    );
    lease.accept_inline_csp(&request, &denied, &cancel).unwrap();
    let DomSourceProgress::Complete(done) = lease.advance(&cancel).unwrap() else {
        panic!("original mandatory tail");
    };
    assert!(done.cancellation_requested());
    drop(lease);
    let records = dom.take_child_list_records(observer, &cancel).unwrap();
    assert_eq!(records.len(), 2);
    assert_record(&records[0], fragment, &[], &[style, sibling], None, None);
    assert_record(&records[1], root, &[style, sibling], &[], Some(last), None);
}

#[test]
fn one_notification_turn_delivers_observers_before_its_captured_slot_signals() {
    let cancel = AtomicBool::new(false);
    let names = DomNameLimits::new(16_777_216, 16_384, 4_000_000, 1_048_576).unwrap();
    let observations = DomChildListObservationLimits::new(8, 32, 32, 256).unwrap();
    let limits = DomLimits::new(64, 4096, 128, 64, 0, 64, 1_000_000, names)
        .unwrap()
        .with_child_list_observation(observations);
    let mut dom = Dom::new(limits).unwrap();
    let mut setup = DomBatch::new();
    let document = setup.create_document();
    let host = setup.create_element_ns(&document, Namespace::html(), "div");
    setup.insert_before(&host, &document, None);
    let shadow = setup.attach_shadow(&host, DomShadowRootInit::new(HtmlShadowRootMode::Open));
    let slot = setup.create_element_ns(&document, Namespace::html(), "slot");
    setup.insert_before(&slot, &shadow, None);
    let commit = dom.apply(setup, &cancel).unwrap();
    let document = commit.document(&document).unwrap();
    let host = commit.created(&host).unwrap().unwrap();
    let slot = commit.created(&slot).unwrap().unwrap();
    let checkpoint = checkpoint_limits(1_000_000);
    assert!(matches!(
        dom.native_checkpoint_step(checkpoint, &cancel).unwrap(),
        DomNativeCheckpoint::Idle
    ));
    let observer = observe(&mut dom, host, false);
    let mut append = DomBatch::new();
    let text = append.create_text(document, "slotted");
    append.insert_before(&text, host, None);
    let text = dom
        .apply(append, &cancel)
        .unwrap()
        .created(&text)
        .unwrap()
        .unwrap();
    assert_eq!(dom.pending_native_tasks(), 1);
    let DomNativeCheckpoint::ChildList(delivery) =
        dom.native_checkpoint_step(checkpoint, &cancel).unwrap()
    else {
        panic!("observer is reached before Slot signals");
    };
    assert_eq!(delivery.observer(), observer);
    assert_eq!(delivery.records().len(), 1);
    assert_record(&delivery.records()[0], host, &[text], &[], None, None);
    drop(delivery);
    let DomNativeCheckpoint::Slots(signals) =
        dom.native_checkpoint_step(checkpoint, &cancel).unwrap()
    else {
        panic!("original captured Slot signal follows observers");
    };
    assert_eq!(signals.targets(), &[slot]);
    assert!(matches!(
        dom.native_checkpoint_step(checkpoint, &cancel).unwrap(),
        DomNativeCheckpoint::Idle
    ));
}

#[test]
fn notification_order_uses_creation_across_reversed_registration_and_slot_reuse() {
    let Tree {
        mut dom,
        root,
        branch,
        ..
    } = tree();
    let cancel = AtomicBool::new(false);
    let checkpoint = checkpoint_limits(1_000_000);
    let older = dom.create_child_list_observer(&cancel).unwrap();
    let younger = dom.create_child_list_observer(&cancel).unwrap();
    dom.observe_child_list(younger, root, false, &cancel)
        .unwrap();
    dom.observe_child_list(older, root, false, &cancel).unwrap();

    let mut detach = DomBatch::new();
    detach.detach(branch);
    dom.apply(detach, &cancel).unwrap();
    for expected in [older, younger] {
        let DomNativeCheckpoint::ChildList(delivery) =
            dom.native_checkpoint_step(checkpoint, &cancel).unwrap()
        else {
            panic!("expected observer delivery");
        };
        assert_eq!(delivery.observer(), expected);
    }
    assert!(matches!(
        dom.native_checkpoint_step(checkpoint, &cancel).unwrap(),
        DomNativeCheckpoint::Idle
    ));

    dom.retire_child_list_observer(older, &cancel).unwrap();
    dom.disconnect_child_list_observer(younger, &cancel)
        .unwrap();
    let newest = observe(&mut dom, root, false);
    dom.observe_child_list(younger, root, false, &cancel)
        .unwrap();
    let mut insert = DomBatch::new();
    insert.insert_before(branch, root, None);
    dom.apply(insert, &cancel).unwrap();
    for expected in [younger, newest] {
        let DomNativeCheckpoint::ChildList(delivery) =
            dom.native_checkpoint_step(checkpoint, &cancel).unwrap()
        else {
            panic!("expected observer delivery after slot reuse");
        };
        assert_eq!(delivery.observer(), expected);
    }
}

#[test]
fn transient_and_reference_admission_refuse_without_leaking_a_partial_registration() {
    let Tree {
        mut dom,
        root,
        first,
        branch,
        last,
        ..
    } = tree();
    let cancel = AtomicBool::new(false);
    let limits = |registrations, references| {
        DomChildListObservationLimits::new(1, registrations, 1, references).unwrap()
    };
    dom.set_limits(dom.limits().with_child_list_observation(limits(1, 4)))
        .unwrap();
    let observer = observe(&mut dom, root, true);
    let before = dom.snapshot().stamp();
    for (registrations, references, resource, limit) in [
        (1, 4, DomResource::ChildListRegistrations, 1),
        (2, 3, DomResource::ChildListRecordReferences, 3),
    ] {
        dom.set_limits(
            dom.limits()
                .with_child_list_observation(limits(registrations, references)),
        )
        .unwrap();
        let mut detach = DomBatch::new();
        detach.detach(branch);
        assert_eq!(
            dom.apply(detach, &cancel).unwrap_err(),
            DomError::ResourceLimit { resource, limit }
        );
        assert_eq!(dom.snapshot().stamp(), before);
        assert_eq!(dom.snapshot().node(branch).unwrap().parent(), Some(root));
        assert_eq!(dom.pending_native_tasks(), 0);
        assert!(
            dom.take_child_list_records(observer, &cancel)
                .unwrap()
                .is_empty()
        );
    }
    dom.set_limits(dom.limits().with_child_list_observation(limits(2, 4)))
        .unwrap();
    let mut detach = DomBatch::new();
    detach.detach(branch);
    dom.apply(detach, &cancel).unwrap();
    let records = dom.take_child_list_records(observer, &cancel).unwrap();
    assert_eq!(records.len(), 1);
    assert_record(&records[0], root, &[], &[branch], Some(first), Some(last));
}
