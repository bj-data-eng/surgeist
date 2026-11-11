use super::*;

const GENEROUS: DomNodeRetirementLimits = DomNodeRetirementLimits::new(16_000_000, 1000);

fn fixture(cancel: &AtomicBool) -> (Dom, DocumentId, NodeId) {
    let mut dom = Dom::new(
        DomLimits::new(
            128,
            4096,
            1024,
            0,
            0,
            0,
            0,
            crate::DomNameLimits::new(16_777_216, 16_384, 4_000_000, 1_048_576).unwrap(),
        )
        .unwrap(),
    )
    .unwrap();
    let mut batch = DomBatch::new();
    let document = batch.create_document();
    let comment = batch.create_comment(&document, "ordinary watch");
    batch.insert_before(&comment, &document, None);
    let commit = dom.apply(batch, cancel).unwrap();
    (
        dom,
        commit.document(&document).unwrap(),
        commit.created(&comment).unwrap().unwrap(),
    )
}

fn bytes(dom: &Dom) -> usize {
    dom.custody_registry.ledger().total().bytes()
}

#[test]
fn creation_and_watch_refusal_preserve_the_original_dom_publication() {
    let cancel = AtomicBool::new(false);
    let (mut dom, document, original) = fixture(&cancel);
    let mut inbox = dom.node_retirement_inbox(GENEROUS, &cancel).unwrap();
    let before = dom.snapshot();
    // Settle releases from the successful fixture batch before measuring
    // which custody belongs to the attempted creation.
    dom.custody_registry.reconcile_releases();
    let original_bytes = bytes(&dom);

    let mut batch = DomBatch::new();
    let ticket = batch.create_fragment(document);
    let result = dom.apply_with_node_retirement_watch(
        batch,
        &ticket,
        &inbox,
        9,
        DomNodeRetirementLimits::new(16_000_000, 0),
        &cancel,
    );
    assert!(matches!(
        result,
        Err(DomError::NodeRetirementWorkLimit { limit: 0 })
    ));
    assert_eq!(dom.snapshot().stamp(), before.stamp());
    assert_eq!(dom.snapshot().nodes().count(), before.nodes().count());
    assert!(dom.snapshot().node(original).is_ok());
    assert_eq!(bytes(&dom), original_bytes);
    assert!(dom.node_retirements.is_none());
    assert!(inbox.take().is_none());
}

#[test]
fn watched_creation_admits_the_exact_peak_and_retires_the_exact_generation() {
    let cancel = AtomicBool::new(false);
    let (mut dom, document, _) = fixture(&cancel);
    let mut inbox = dom.node_retirement_inbox(GENEROUS, &cancel).unwrap();
    let before = dom.snapshot();
    let fragment = || {
        let mut batch = DomBatch::new();
        let ticket = batch.create_fragment(document);
        (batch, ticket)
    };
    dom.custody_registry.reconcile_releases();
    let baseline = bytes(&dom);

    // Independently measure the ordinary publication's actual retained
    // preparation, then add the fixed first-watch index, leaf and notice.
    let (batch, ticket) = fragment();
    let prepared = dom.prepare(batch, &cancel).unwrap();
    let mut publication = dom.prepare_publication(prepared, &cancel).unwrap();
    publication.close_ordinary_sharing(&cancel).unwrap();
    let exact = bytes(&dom) + size_of::<Index>() + size_of::<Armed>() + size_of::<Notice>();
    // The initial native lookup uses one slot page. Retirement reserves
    // descent, unwind and exact-leaf disarm from the same work allowance.
    let work = 5 + 2 * u64::from(usize::BITS) + 1;
    let staged = resolve_created_node(&publication.owner, &publication.created, &ticket)
        .unwrap()
        .unwrap();
    let admit = |limit| {
        let admission = dom.ordinary_watch_admission(
            &publication.state,
            staged,
            &inbox,
            29,
            DomNodeRetirementLimits::new(limit, work),
            &cancel,
        )?;
        arm(dom.node_retirements.as_ref(), &admission, None)
    };
    assert!(matches!(
        admit(exact - 1),
        Err(DomError::ResourceLimit {
            resource: DomResource::NodeRetirementCanonicalBytes,
            limit,
        }) if limit == exact - 1
    ));
    let provisional = admit(exact).unwrap();
    assert_eq!(bytes(&dom), exact);
    drop(provisional);
    drop(publication);
    dom.custody_registry.reconcile_releases();
    assert_eq!(bytes(&dom), baseline);

    let (batch, ticket) = fragment();
    assert!(matches!(
        dom.apply_with_node_retirement_watch(
            batch,
            &ticket,
            &inbox,
            29,
            DomNodeRetirementLimits::new(16_000_000, work - 1),
            &cancel,
        ),
        Err(DomError::NodeRetirementWorkLimit { limit }) if limit == work - 1
    ));
    assert_eq!(dom.snapshot().stamp(), before.stamp());
    assert_eq!(bytes(&dom), baseline);
    assert!(dom.node_retirements.is_none());
    assert!(inbox.take().is_none());

    let (batch, ticket) = fragment();
    let created = dom
        .apply_with_node_retirement_watch(
            batch,
            &ticket,
            &inbox,
            29,
            DomNodeRetirementLimits::new(16_000_000, work),
            &cancel,
        )
        .unwrap();
    let observed = created.node();
    let (commit, node, watch) = created.into_parts();
    assert_eq!(node, observed);
    assert_eq!(commit.created(&ticket).unwrap(), Some(node));
    assert_eq!(commit.changes().from(), before.stamp());
    assert_eq!(commit.changes().to(), dom.snapshot().stamp());
    assert!(inbox.take().is_none());

    let mut destroy = DomBatch::new();
    destroy.destroy(node);
    dom.apply(destroy, &cancel).unwrap();
    let notice = inbox.take().unwrap();
    assert_eq!(notice.node(), node);
    assert_eq!(notice.custody_key(), 29);
    assert!(inbox.take().is_none());
    let (batch, ticket) = fragment();
    let (_, replacement, replacement_watch) = dom
        .apply_with_node_retirement_watch(batch, &ticket, &inbox, 31, GENEROUS, &cancel)
        .unwrap()
        .into_parts();
    assert_eq!(replacement.slot, node.slot);
    assert_ne!(replacement.generation, node.generation);
    drop(watch);
    let mut destroy = DomBatch::new();
    destroy.destroy(replacement);
    dom.apply(destroy, &cancel).unwrap();
    let notice = inbox.take().unwrap();
    assert_eq!(notice.node(), replacement);
    assert_eq!(notice.custody_key(), 31);
    assert!(inbox.take().is_none());
    drop(replacement_watch);
}

#[test]
fn watched_creation_refuses_cancelled_foreign_and_destroyed_tickets() {
    let cancel = AtomicBool::new(false);
    let (mut dom, document, _) = fixture(&cancel);
    let mut inbox = dom.node_retirement_inbox(GENEROUS, &cancel).unwrap();
    let before = dom.snapshot();
    dom.custody_registry.reconcile_releases();
    let baseline = bytes(&dom);

    let mut batch = DomBatch::new();
    let foreign_ticket = batch.create_fragment(document);
    let mut other = DomBatch::new();
    other.create_fragment(document);
    assert!(matches!(
        dom.apply_with_node_retirement_watch(other, &foreign_ticket, &inbox, 1, GENEROUS, &cancel),
        Err(DomError::ForeignBatch)
    ));
    cancel.store(true, Ordering::Relaxed);
    assert!(matches!(
        dom.apply_with_node_retirement_watch(batch, &foreign_ticket, &inbox, 1, GENEROUS, &cancel),
        Err(DomError::Cancelled)
    ));
    cancel.store(false, Ordering::Relaxed);
    let mut batch = DomBatch::new();
    let ticket = batch.create_fragment(document);
    batch.destroy(&ticket);
    assert!(matches!(
        dom.apply_with_node_retirement_watch(batch, &ticket, &inbox, 1, GENEROUS, &cancel),
        Err(DomError::UnavailableCreation)
    ));
    assert_eq!(dom.snapshot().stamp(), before.stamp());
    assert_eq!(bytes(&dom), baseline);
    assert!(inbox.take().is_none());
    assert!(dom.node_retirements.is_none());
}

#[test]
fn watched_creation_reuses_a_slot_retired_by_the_same_publication() {
    let cancel = AtomicBool::new(false);
    let (mut dom, document, original) = fixture(&cancel);
    let mut detach = DomBatch::new();
    detach.detach(original);
    dom.apply(detach, &cancel).unwrap();
    let mut inbox = dom.node_retirement_inbox(GENEROUS, &cancel).unwrap();
    let mut peer_inbox = dom.node_retirement_inbox(GENEROUS, &cancel).unwrap();
    let original_watch = dom
        .watch_node_retirement(original, &inbox, 11, GENEROUS, &cancel)
        .unwrap();
    let peer_watch = dom
        .watch_node_retirement(original, &peer_inbox, 11, GENEROUS, &cancel)
        .unwrap();
    let before = dom.snapshot();
    dom.custody_registry.reconcile_releases();
    let baseline = bytes(&dom);
    let replace = || {
        let mut batch = DomBatch::new();
        batch.destroy(original);
        let ticket = batch.create_fragment(document);
        (batch, ticket)
    };

    let (batch, ticket) = replace();
    let Err(error) = dom.apply_with_node_retirement_watch(
        batch,
        &ticket,
        &inbox,
        13,
        DomNodeRetirementLimits::new(16_000_000, 0),
        &cancel,
    ) else {
        panic!("one short watch budget must refuse the whole batch");
    };
    assert!(
        matches!(error, DomError::NodeRetirementWorkLimit { limit: 0 }),
        "{error:?}"
    );
    assert_eq!(dom.snapshot().stamp(), before.stamp());
    assert!(dom.snapshot().node(original).is_ok());
    assert_eq!(bytes(&dom), baseline);
    assert!(inbox.take().is_none());
    assert!(peer_inbox.take().is_none());

    let (batch, ticket) = replace();
    let (_, successor, successor_watch) = dom
        .apply_with_node_retirement_watch(batch, &ticket, &inbox, 13, GENEROUS, &cancel)
        .unwrap()
        .into_parts();
    assert_eq!(successor.slot, original.slot);
    assert_ne!(successor.generation, original.generation);
    assert!(dom.snapshot().node(original).is_err());
    let notice = inbox.take().unwrap();
    assert_eq!((notice.node(), notice.custody_key()), (original, 11));
    assert!(inbox.take().is_none());
    let notice = peer_inbox.take().unwrap();
    assert_eq!((notice.node(), notice.custody_key()), (original, 11));
    assert!(peer_inbox.take().is_none());
    drop(original_watch);
    drop(peer_watch);

    let mut destroy = DomBatch::new();
    destroy.destroy(successor);
    dom.apply(destroy, &cancel).unwrap();
    let notice = inbox.take().unwrap();
    assert_eq!((notice.node(), notice.custody_key()), (successor, 13));
    assert!(inbox.take().is_none());
    assert!(peer_inbox.take().is_none());
    drop(successor_watch);
}

#[test]
fn replacement_watch_can_still_disarm_the_old_generation_before_publication() {
    let cancel = AtomicBool::new(false);
    let (mut dom, document, original) = fixture(&cancel);
    let mut detach = DomBatch::new();
    detach.detach(original);
    dom.apply(detach, &cancel).unwrap();
    let mut inbox = dom.node_retirement_inbox(GENEROUS, &cancel).unwrap();
    let old_watch = dom
        .watch_node_retirement(original, &inbox, 17, GENEROUS, &cancel)
        .unwrap();
    let mut batch = DomBatch::new();
    batch.destroy(original);
    let ticket = batch.create_fragment(document);
    let prepared = dom.prepare(batch, &cancel).unwrap();
    let created = resolve_created_node(&prepared.owner, &prepared.created, &ticket)
        .unwrap()
        .unwrap();
    let mut publication = dom.prepare_publication(prepared, &cancel).unwrap();
    publication.close_ordinary_sharing(&cancel).unwrap();
    let staged_bytes = bytes(&dom);
    let exact = staged_bytes + size_of::<Replacement>() + size_of::<Armed>() + size_of::<Notice>();
    let attempt = |limit| {
        let admission = dom.ordinary_watch_admission(
            &publication.state,
            created,
            &inbox,
            19,
            DomNodeRetirementLimits::new(limit, 1000),
            &cancel,
        )?;
        arm(
            dom.node_retirements.as_ref(),
            &admission,
            publication.node_retirement_publication.as_ref(),
        )
    };
    assert!(matches!(
        attempt(exact - 1),
        Err(DomError::ResourceLimit {
            resource: DomResource::NodeRetirementCanonicalBytes,
            limit,
        }) if limit == exact - 1
    ));
    assert_eq!(bytes(&dom), staged_bytes);
    let watch = attempt(exact).unwrap();
    assert_eq!(bytes(&dom), exact);
    assert!(watch.index.prepare_match(original, || Ok(())).unwrap());
    assert!(watch.index.prepare_match(created, || Ok(())).unwrap());
    drop(old_watch);
    assert!(!watch.index.prepare_match(original, || Ok(())).unwrap());
    assert!(watch.index.prepare_match(created, || Ok(())).unwrap());
    let watch = dom.finish_watch(watch, created);
    dom.publish_prepared(publication);
    assert!(inbox.take().is_none());

    let mut destroy = DomBatch::new();
    destroy.destroy(created);
    dom.apply(destroy, &cancel).unwrap();
    let notice = inbox.take().unwrap();
    assert_eq!((notice.node(), notice.custody_key()), (created, 19));
    assert!(inbox.take().is_none());
    drop(watch);
}

#[test]
fn exact_inbox_and_watch_admission_and_cancelled_disarm() {
    let cancel = AtomicBool::new(false);
    let (mut dom, document, node) = fixture(&cancel);
    // Cross the bank's 16-slot leaf width while keeping this exact node in
    // the first leaf. A lookup reaches the outer page and that original leaf.
    let mut expand = DomBatch::new();
    for _ in 0..32 {
        expand.create_comment(document.node(), "");
    }
    dom.apply(expand, &cancel).unwrap();
    let baseline = bytes(&dom);
    let inbox_bytes = baseline + size_of::<Mutex<Queue>>();
    assert!(matches!(
        dom.node_retirement_inbox(DomNodeRetirementLimits::new(inbox_bytes, 0), &cancel),
        Err(DomError::NodeRetirementWorkLimit { limit: 0 })
    ));
    assert!(matches!(
        dom.node_retirement_inbox(DomNodeRetirementLimits::new(inbox_bytes - 1, 2), &cancel),
        Err(DomError::ResourceLimit {
            resource: DomResource::NodeRetirementCanonicalBytes,
            limit,
        }) if limit == inbox_bytes - 1
    ));
    assert_eq!(bytes(&dom), baseline);

    let inbox = dom
        .node_retirement_inbox(DomNodeRetirementLimits::new(inbox_bytes, 2), &cancel)
        .unwrap();
    assert_eq!(bytes(&dom), inbox_bytes);
    let exact = inbox_bytes + size_of::<Index>() + size_of::<Armed>() + size_of::<Notice>();
    // Entry, canonical lookup, its two radix pages, index formation and
    // custody admission each take one step. Disarm can visit every possible
    // slot bit on descent and unwind, plus the exact leaf.
    let work = 6 + 2 * u64::from(usize::BITS) + 1;
    assert!(matches!(
        dom.watch_node_retirement(node, &inbox, 14, DomNodeRetirementLimits::new(exact, work - 1), &cancel),
        Err(DomError::NodeRetirementWorkLimit { limit }) if limit == work - 1
    ));
    assert!(matches!(
        dom.watch_node_retirement(node, &inbox, 14, DomNodeRetirementLimits::new(exact - 1, work), &cancel),
        Err(DomError::ResourceLimit {
            resource: DomResource::NodeRetirementCanonicalBytes,
            limit,
        }) if limit == exact - 1
    ));
    assert_eq!(bytes(&dom), inbox_bytes);
    assert!(dom.node_retirements.is_none());

    let watch = dom
        .watch_node_retirement(
            node,
            &inbox,
            14,
            DomNodeRetirementLimits::new(exact, work),
            &cancel,
        )
        .unwrap();
    assert_eq!(bytes(&dom), exact);
    cancel.store(true, Ordering::Relaxed);
    drop(watch);
    assert_eq!(bytes(&dom), inbox_bytes + size_of::<Index>());
    drop(inbox);
    // The reusable index administration belongs to the native DOM.
    assert_eq!(bytes(&dom), baseline + size_of::<Index>());
}

#[test]
fn ordinary_watches_survive_detach_adopt_and_failed_destruction() {
    let cancel = AtomicBool::new(false);
    let (mut dom, document, watched) = fixture(&cancel);
    let mut other = DomBatch::new();
    let receiver = other.create_document();
    let receiver = dom
        .apply(other, &cancel)
        .unwrap()
        .document(&receiver)
        .unwrap();
    let (mut foreign, _, foreign_node) = fixture(&cancel);
    let mut inbox = dom.node_retirement_inbox(GENEROUS, &cancel).unwrap();
    let foreign_inbox = foreign.node_retirement_inbox(GENEROUS, &cancel).unwrap();
    assert!(matches!(
        dom.watch_node_retirement(foreign_node, &inbox, 0, GENEROUS, &cancel),
        Err(DomError::ForeignOwner)
    ));
    assert!(matches!(
        foreign.watch_node_retirement(foreign_node, &inbox, 0, GENEROUS, &cancel),
        Err(DomError::ForeignOwner)
    ));
    assert!(matches!(
        dom.watch_node_retirement(watched, &foreign_inbox, 0, GENEROUS, &cancel),
        Err(DomError::ForeignOwner)
    ));
    let watch = dom
        .watch_node_retirement(watched, &inbox, 72, GENEROUS, &cancel)
        .unwrap();

    let mut detach = DomBatch::new();
    detach.detach(watched);
    dom.apply(detach, &cancel).unwrap();
    assert!(inbox.take().is_none());
    let mut adopt = DomBatch::new();
    adopt.adopt(watched, receiver.node());
    dom.apply(adopt, &cancel).unwrap();
    assert!(inbox.take().is_none());

    let mut refused = DomBatch::new();
    refused.destroy(watched);
    refused.insert_before(document.node(), document.node(), None);
    assert!(matches!(
        dom.apply(refused, &cancel),
        Err(DomError::HierarchyRequest)
    ));
    assert!(dom.snapshot().node(watched).is_ok());
    assert!(inbox.take().is_none());

    let mut destroy = DomBatch::new();
    destroy.destroy(watched);
    dom.apply(destroy, &cancel).unwrap();
    let mut replacement = DomBatch::new();
    let comment = replacement.create_comment(document.node(), "replacement");
    let fresh = dom
        .apply(replacement, &cancel)
        .unwrap()
        .created(&comment)
        .unwrap()
        .unwrap();
    assert_eq!(fresh.slot, watched.slot);
    assert_ne!(fresh.generation, watched.generation);
    assert!(matches!(
        dom.watch_node_retirement(watched, &inbox, 0, GENEROUS, &cancel),
        Err(DomError::StaleNode)
    ));
    let fresh_watch = dom
        .watch_node_retirement(fresh, &inbox, 73, GENEROUS, &cancel)
        .unwrap();
    drop(watch);

    let notice = inbox.take().unwrap();
    assert_eq!((notice.node(), notice.custody_key()), (watched, 72));
    assert!(inbox.take().is_none());
    let before = bytes(&dom);
    drop(notice);
    assert_eq!(bytes(&dom), before - size_of::<Notice>());
    let mut destroy = DomBatch::new();
    destroy.destroy(fresh);
    dom.apply(destroy, &cancel).unwrap();
    drop(fresh_watch);
    let notice = inbox.take().unwrap();
    assert_eq!((notice.node(), notice.custody_key()), (fresh, 73));
    assert!(inbox.take().is_none());
}

#[test]
fn independent_inboxes_publish_and_disarm_their_original_watches() {
    for disarm_first in [false, true] {
        let cancel = AtomicBool::new(false);
        let (mut dom, _, node) = fixture(&cancel);
        let mut first_inbox = dom.node_retirement_inbox(GENEROUS, &cancel).unwrap();
        let mut second_inbox = dom.node_retirement_inbox(GENEROUS, &cancel).unwrap();
        let mut first = Some(
            dom.watch_node_retirement(node, &first_inbox, 41, GENEROUS, &cancel)
                .unwrap(),
        );
        let second = dom
            .watch_node_retirement(node, &second_inbox, 41, GENEROUS, &cancel)
            .unwrap();

        let mut refused = DomBatch::new();
        refused.destroy(node);
        assert!(matches!(
            dom.apply(refused, &cancel),
            Err(DomError::InvalidDestruction)
        ));
        assert!(first_inbox.take().is_none());
        assert!(second_inbox.take().is_none());

        if disarm_first {
            let watch = first.take();
            std::thread::scope(|scope| scope.spawn(move || drop(watch)).join().unwrap());
        }
        let mut detach = DomBatch::new();
        detach.detach(node);
        dom.apply(detach, &cancel).unwrap();
        assert!(first_inbox.take().is_none());
        assert!(second_inbox.take().is_none());

        let mut destruction = DomBatch::new();
        destruction.destroy(node);
        dom.apply(destruction, &cancel).unwrap();
        let first_notice = first_inbox.take();
        assert_eq!(
            first_notice.map(|notice| (notice.node(), notice.custody_key())),
            (!disarm_first).then_some((node, 41))
        );
        let second_notice = second_inbox.take().unwrap();
        assert_eq!(
            (second_notice.node(), second_notice.custody_key()),
            (node, 41)
        );
        assert!(first_inbox.take().is_none());
        assert!(second_inbox.take().is_none());
        drop(first);
        drop(second);
    }
}

#[test]
fn shared_watch_storage_and_growth_are_admitted_before_changing_custody() {
    let cancel = AtomicBool::new(false);
    let (mut dom, _, node) = fixture(&cancel);
    let first_inbox = dom.node_retirement_inbox(GENEROUS, &cancel).unwrap();
    let second_inbox = dom.node_retirement_inbox(GENEROUS, &cancel).unwrap();
    let third_inbox = dom.node_retirement_inbox(GENEROUS, &cancel).unwrap();
    dom.custody_registry.reconcile_releases();
    let baseline = bytes(&dom);
    let first = dom
        .watch_node_retirement(node, &first_inbox, 17, GENEROUS, &cancel)
        .unwrap();
    let per_watch = size_of::<Armed>() + size_of::<Notice>();
    let per_slot = size_of::<watches::Entry>();
    let before_second = baseline + size_of::<Index>() + per_watch;
    assert_eq!(bytes(&dom), before_second);

    // One native page, one index leaf and the original custody admit. The
    // new two-entry bank prepays both entries' eventual release and both
    // exact slot unlinks, including the watch already armed.
    let disarm = 2 * u64::from(usize::BITS) + 1;
    let second_work = disarm + 9;
    let second_peak = before_second + per_watch + size_of::<watches::Shared>() + 2 * per_slot;
    assert!(matches!(
        dom.watch_node_retirement(
            node,
            &second_inbox,
            17,
            DomNodeRetirementLimits::new(second_peak - 1, second_work),
            &cancel
        ),
        Err(DomError::ResourceLimit {
            resource: DomResource::NodeRetirementCanonicalBytes,
            limit,
        }) if limit == second_peak - 1
    ));
    assert!(matches!(
        dom.watch_node_retirement(
            node,
            &second_inbox,
            17,
            DomNodeRetirementLimits::new(second_peak, second_work - 1),
            &cancel
        ),
        Err(DomError::NodeRetirementWorkLimit { limit }) if limit == second_work - 1
    ));
    assert_eq!(bytes(&dom), before_second);
    let second = dom
        .watch_node_retirement(
            node,
            &second_inbox,
            17,
            DomNodeRetirementLimits::new(second_peak, second_work),
            &cancel,
        )
        .unwrap();
    assert_eq!(bytes(&dom), second_peak);
    assert!(matches!(
        dom.watch_node_retirement(node, &first_inbox, 81, GENEROUS, &cancel),
        Err(DomError::NodeRetirementAlreadyWatched)
    ));

    // The replacement vector has four slots. Admit its overlap with the
    // still-live two-slot vector, relocation/teardown visits and one unlink.
    let third_peak = second_peak + per_watch + 4 * per_slot;
    let third_work = disarm + 14;
    assert!(matches!(
        dom.watch_node_retirement(
            node,
            &third_inbox,
            17,
            DomNodeRetirementLimits::new(third_peak - 1, third_work),
            &cancel
        ),
        Err(DomError::ResourceLimit {
            resource: DomResource::NodeRetirementCanonicalBytes,
            limit,
        }) if limit == third_peak - 1
    ));
    assert!(matches!(
        dom.watch_node_retirement(
            node,
            &third_inbox,
            17,
            DomNodeRetirementLimits::new(third_peak, third_work - 1),
            &cancel
        ),
        Err(DomError::NodeRetirementWorkLimit { limit }) if limit == third_work - 1
    ));
    assert_eq!(bytes(&dom), second_peak);
    let third = dom
        .watch_node_retirement(
            node,
            &third_inbox,
            17,
            DomNodeRetirementLimits::new(third_peak, third_work),
            &cancel,
        )
        .unwrap();
    let third_retained = third_peak - 2 * per_slot;
    assert_eq!(bytes(&dom), third_retained);

    drop(second);
    assert_eq!(bytes(&dom), third_retained - per_watch);
    let second = dom
        .watch_node_retirement(
            node,
            &second_inbox,
            81,
            DomNodeRetirementLimits::new(third_retained, 1000),
            &cancel,
        )
        .unwrap();
    assert_eq!(bytes(&dom), third_retained);
    cancel.store(true, Ordering::Relaxed);
    drop((third, first, second));
    assert_eq!(bytes(&dom), baseline + size_of::<Index>());
}

#[test]
fn closing_one_inbox_leaves_the_other_native_delivery_live() {
    let cancel = AtomicBool::new(false);
    let (mut dom, _, node) = fixture(&cancel);
    let closed = dom.node_retirement_inbox(GENEROUS, &cancel).unwrap();
    let mut remaining = dom.node_retirement_inbox(GENEROUS, &cancel).unwrap();
    let closed_watch = dom
        .watch_node_retirement(node, &closed, 21, GENEROUS, &cancel)
        .unwrap();
    let remaining_watch = dom
        .watch_node_retirement(node, &remaining, 21, GENEROUS, &cancel)
        .unwrap();
    drop(closed);
    let mut edit = DomBatch::new();
    edit.detach(node);
    edit.destroy(node);
    dom.apply(edit, &cancel).unwrap();
    let notice = remaining.take().unwrap();
    assert_eq!((notice.node(), notice.custody_key()), (node, 21));
    assert!(remaining.take().is_none());
    drop((closed_watch, remaining_watch));
}
