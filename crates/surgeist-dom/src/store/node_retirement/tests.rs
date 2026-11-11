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
fn exact_inbox_and_watch_admission_and_cancelled_disarm() {
    let cancel = AtomicBool::new(false);
    let (mut dom, _, node) = fixture(&cancel);
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
    // Entry, index formation and custody admission each take one step. Every
    // possible slot bit can be visited on descent and unwind, plus the leaf.
    let work = 3 + 2 * u64::from(usize::BITS) + 1;
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
