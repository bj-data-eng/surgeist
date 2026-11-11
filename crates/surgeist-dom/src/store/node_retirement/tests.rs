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
    let (batch, _) = fragment();
    let prepared = dom.prepare(batch, &cancel).unwrap();
    let mut publication = dom.prepare_publication(prepared, &cancel).unwrap();
    publication.close_ordinary_sharing(&cancel).unwrap();
    let exact = bytes(&dom) + size_of::<Index>() + size_of::<Armed>() + size_of::<Notice>();
    drop(publication);
    dom.custody_registry.reconcile_releases();
    assert_eq!(bytes(&dom), baseline);
    // The initial native lookup uses one slot page. Retirement reserves
    // descent, unwind and exact-leaf disarm from the same work allowance.
    let work = 5 + 2 * u64::from(usize::BITS) + 1;
    for limits in [
        DomNodeRetirementLimits::new(exact, work - 1),
        DomNodeRetirementLimits::new(exact - 1, work),
    ] {
        let (batch, ticket) = fragment();
        let refused =
            dom.apply_with_node_retirement_watch(batch, &ticket, &inbox, 29, limits, &cancel);
        assert!(match refused {
            Err(DomError::NodeRetirementWorkLimit { limit }) => limit == work - 1,
            Err(DomError::ResourceLimit {
                resource: DomResource::NodeRetirementCanonicalBytes,
                limit,
            }) => limit == exact - 1,
            _ => false,
        });
        assert_eq!(dom.snapshot().stamp(), before.stamp());
        assert_eq!(bytes(&dom), baseline);
        assert!(dom.node_retirements.is_none());
        assert!(inbox.take().is_none());
    }

    let (batch, ticket) = fragment();
    let created = dom
        .apply_with_node_retirement_watch(
            batch,
            &ticket,
            &inbox,
            29,
            DomNodeRetirementLimits::new(exact, work),
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
