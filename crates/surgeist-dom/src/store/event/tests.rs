use super::*;
use crate::{DomSourceEdit, DomSourceLimits, DomSourceProgress, DomSourceRequest, Namespace};

fn policy(registrations: usize, bytes: usize, work: u64) -> DomEventRecipientLimits {
    DomEventRecipientLimits::new(registrations, bytes, 4_000_000, work).unwrap()
}
fn limits() -> DomLimits {
    DomLimits::new(
        64,
        65536,
        128,
        64,
        64,
        0,
        0,
        crate::DomNameLimits::new(16_777_216, 16_384, 4_000_000, 1_048_576).unwrap(),
    )
    .unwrap()
    .with_event_recipients(policy(16, 4_000_000, 1_000_000))
}
fn fixture() -> (Dom, DocumentId, NodeId) {
    let mut dom = Dom::new(limits()).unwrap();
    let mut batch = DomBatch::new();
    let document = batch.create_document();
    let target = batch.create_comment(&document, "recipient");
    let commit = dom.apply(batch, &AtomicBool::new(false)).unwrap();
    (
        dom,
        commit.document(&document).unwrap(),
        commit.created(&target).unwrap().unwrap(),
    )
}
fn source_limits() -> DomSourceLimits {
    DomSourceLimits::new(16_000_000, 1_000_000, 10_000_000, 16, 4096, 4096, 1)
}
struct Parked {
    operation: crate::DomSourceOperationId,
    request: crate::DomStyleRequest,
    container: NodeId,
    script: NodeId,
    status: crate::DomSourceStatus,
}
fn park_before_svg(dom: &mut Dom, document: DocumentId) -> Parked {
    let cancel = AtomicBool::new(false);
    let mut batch = DomBatch::new();
    let container = batch.create_element_ns(document, Namespace::html(), "div");
    let style = batch.create_element_ns(document, Namespace::html(), "style");
    let text = batch.create_text(document, "a{}");
    batch.insert_before(&text, &style, None);
    batch.insert_before(&style, &container, None);
    let script = batch.create_element_ns(document, Namespace::svg(), "script");
    batch.insert_before(&script, &container, None);
    let commit = dom.apply(batch, &cancel).unwrap();
    let container = commit.created(&container).unwrap().unwrap();
    let script = commit.created(&script).unwrap().unwrap();
    let mut lease = dom
        .begin_source_edit(
            DomSourceEdit::insert_before(container, document.node(), None),
            source_limits(),
            &cancel,
        )
        .unwrap();
    let operation = lease.operation();
    let DomSourceProgress::Request(DomSourceRequest::Style(request)) =
        lease.advance(&cancel).unwrap()
    else {
        panic!("actual first Style policy request precedes sibling SVG processing")
    };
    assert_eq!(
        request.action(),
        crate::DomStyleAction::Update(surgeist_html::HtmlStyleUpdateAction::CheckInlineCsp)
    );
    let status = lease.status();
    assert!(status.has_published());
    drop(lease);
    assert_eq!(dom.pending_source_edit().unwrap().operation(), operation);
    let snapshot = dom.snapshot();
    assert_eq!(snapshot.node(script).unwrap().parent(), Some(container));
    assert!(snapshot.node(script).unwrap().is_connected());
    assert!(
        !snapshot
            .node(script)
            .unwrap()
            .as_svg_script()
            .unwrap()
            .flags()
            .already_processed()
    );
    Parked {
        operation,
        request,
        container,
        script,
        status,
    }
}
fn accept_block(lease: &mut crate::DomSourceLease<'_>, request: &crate::DomStyleRequest) {
    let observation = crate::DomStyleInlineCspObservation::evaluated(
        request.source(),
        Arc::new(vec!["style-src 'none'".into()]),
        19,
        false,
    );
    lease
        .accept_inline_csp(request, &observation, &AtomicBool::new(false))
        .unwrap();
}
fn retain(dom: &mut Dom, target: NodeId, capture: bool) -> DomEventRecipient {
    dom.retain_event_recipient(
        target.into(),
        DomEventType::Error,
        capture,
        &AtomicBool::new(false),
    )
    .unwrap()
    .unwrap()
}
fn has(dom: &Dom, target: NodeId, at_target: bool) -> bool {
    let cancel = AtomicBool::new(false);
    let processing = Processing::new(dom.limits.event_recipients(), None, &cancel);
    dom.event_recipients
        .as_ref()
        .unwrap()
        .has_recipient(target.into(), DomEventType::Error, at_target, &processing)
        .unwrap()
}
// Independent raw backing enumeration: no cached receipt or aggregate counter
// prices index pages, records, endpoint bodies or the affine occurrences.
fn raw_bank(bank: &OwnedBank<Part>) -> usize {
    bank.inspect_storage_bytes()
        + bank
            .values()
            .map(|part| match part {
                Part::Branch(branch) => raw_bank(branch),
                Part::Endpoint(_) => size_of::<Bucket>(),
            })
            .sum::<usize>()
}
fn raw_owner(owner: &Owner, occurrences: usize) -> usize {
    let index = owner.index.lock().unwrap();
    let index = index.as_ref().unwrap();
    size_of::<Owner>()
        + size_of::<Accounting>()
        + raw_bank(&index.nodes)
        + raw_bank(&index.globals)
        + occurrences * size_of::<DomEventRecipient>()
}

#[test]
fn zero_policy_is_lazy_and_unrelated_classes_need_no_recipient_capacity() {
    let (mut dom, _, target) = fixture();
    dom.set_limits(limits().with_event_recipients(DomEventRecipientLimits::default()))
        .unwrap();
    let before = dom.custody_registry.ledger().total();
    assert!(
        dom.retain_event_recipient(
            target.into(),
            DomEventType::Toggle,
            true,
            &AtomicBool::new(false)
        )
        .unwrap()
        .is_none()
    );
    assert!(matches!(
        dom.retain_event_recipient(
            target.into(),
            DomEventType::Load,
            true,
            &AtomicBool::new(false)
        ),
        Err(crate::DomHostedError::Dom(DomError::ResourceLimit {
            resource: DomResource::EventRecipients,
            limit: 0
        }))
    ));
    assert!(dom.event_recipients.is_none());
    assert_eq!(dom.custody_registry.ledger().total(), before);
}

#[test]
fn independent_occurrences_preserve_capture_and_last_release_skips_zero_work_policy() {
    let (mut dom, document, target) = fixture();
    let first = retain(&mut dom, target, true);
    let second = retain(&mut dom, target, false);
    assert!(has(&dom, target, false));
    drop(first);
    assert!(!has(&dom, target, false));
    assert!(has(&dom, target, true));
    drop(second);
    assert!(!dom.event_recipients.as_ref().unwrap().has_live());
    // Actual retained empty buckets remain; their zero aggregate is sufficient
    // to process without any new recipient work or temporary allowance.
    let retained = raw_owner(dom.event_recipients.as_ref().unwrap(), 0);
    dom.set_limits(
        limits().with_event_recipients(DomEventRecipientLimits::new(0, retained, 0, 0).unwrap()),
    )
    .unwrap();
    let mut batch = DomBatch::new();
    let script = batch.create_element_ns(document, Namespace::svg(), "script");
    batch.insert_before(&script, document.node(), None);
    let commit = dom.apply(batch, &AtomicBool::new(false)).unwrap();
    assert!(
        dom.snapshot()
            .node(commit.created(&script).unwrap().unwrap())
            .unwrap()
            .as_svg_script()
            .unwrap()
            .flags()
            .already_processed()
    );
}

#[test]
fn exact_occurrence_boundary_refuses_one_byte_short_without_changing_index_or_counts() {
    let (mut dom, _, target) = fixture();
    let first = retain(&mut dom, target, true);
    let ledger = dom.custody_registry.ledger().clone();
    let before = ledger.total();
    let raw = raw_owner(dom.event_recipients.as_ref().unwrap(), 1);
    assert_eq!(
        dom.event_recipients
            .as_ref()
            .unwrap()
            .accounting
            .bytes
            .load(Ordering::Acquire),
        raw
    );
    let exact = raw + size_of::<DomEventRecipient>();
    dom.set_limits(limits().with_event_recipients(policy(2, exact - 1, 1_000_000)))
        .unwrap();
    assert!(
        matches!(dom.retain_event_recipient(target.into(), DomEventType::Error, false, &AtomicBool::new(false)),
        Err(crate::DomHostedError::Dom(DomError::ResourceLimit { resource: DomResource::EventRecipientRetainedBytes, limit })) if limit == exact - 1)
    );
    assert_eq!(ledger.total(), before);
    assert_eq!(
        dom.event_recipients
            .as_ref()
            .unwrap()
            .accounting
            .live
            .load(Ordering::Acquire),
        1
    );
    assert!(has(&dom, target, false));
    dom.set_limits(limits().with_event_recipients(policy(2, exact, 1_000_000)))
        .unwrap();
    let second = retain(&mut dom, target, false);
    assert_eq!(
        ledger.total(),
        before
            .checked_add(custody::Usage::new(size_of::<DomEventRecipient>(), 0))
            .unwrap()
    );
    drop(second);
    drop(first);
}

#[test]
fn every_new_backing_matches_raw_union_and_outliving_lease_releases_its_actual_bodies() {
    fn send<T: Send>() {}
    send::<Dom>();
    send::<DomEventRecipient>();
    let (mut dom, _, target) = fixture();
    let ledger = dom.custody_registry.ledger().clone();
    let before = ledger.total();
    let lease = retain(&mut dom, target, true);
    let raw = raw_owner(dom.event_recipients.as_ref().unwrap(), 1);
    assert_eq!(
        ledger.total(),
        before.checked_add(custody::Usage::new(raw, 0)).unwrap()
    );
    drop(dom);
    // The old domain's Ledger administration follows its existing Dom lifetime;
    // this enumerates the NEW operational allocations/occurrence still held.
    assert_eq!(
        ledger.total(),
        custody::Usage::new(
            size_of::<Accounting>() + size_of::<Bucket>() + size_of::<DomEventRecipient>(),
            0
        )
    );
    std::thread::spawn(move || drop(lease)).join().unwrap();
    assert_eq!(ledger.total(), custody::Usage::ZERO);
}

#[test]
fn retired_node_slot_reuse_does_not_alias_a_retained_recipient_generation() {
    let (mut dom, document, old) = fixture();
    let previous = retain(&mut dom, old, true);
    let mut batch = DomBatch::new();
    batch.destroy(old);
    dom.apply(batch, &AtomicBool::new(false)).unwrap();
    let mut batch = DomBatch::new();
    let replacement = batch.create_comment(document, "new generation");
    let commit = dom.apply(batch, &AtomicBool::new(false)).unwrap();
    let replacement = commit.created(&replacement).unwrap().unwrap();
    assert_eq!(old.slot, replacement.slot, "real vacant slot reused");
    assert_ne!(old.generation, replacement.generation);
    assert!(!has(&dom, replacement, true));
    let current = retain(&mut dom, replacement, false);
    drop(previous);
    assert!(has(&dom, replacement, true));
    assert!(!has(&dom, replacement, false));
    drop(current);
}

#[test]
fn cancellation_and_caller_unwind_leave_no_positive_capability() {
    let (mut dom, _, target) = fixture();
    let before = dom.custody_registry.ledger().total();
    assert!(matches!(
        dom.retain_event_recipient(
            target.into(),
            DomEventType::Load,
            false,
            &AtomicBool::new(true)
        ),
        Err(crate::DomHostedError::Dom(DomError::Cancelled))
    ));
    assert!(dom.event_recipients.is_none());
    assert_eq!(dom.custody_registry.ledger().total(), before);
    let unwind = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
        let _lease = retain(&mut dom, target, true);
        panic!("actual caller drops its affine capability during unwind");
    }));
    assert!(unwind.is_err());
    assert!(!dom.event_recipients.as_ref().unwrap().has_live());
    assert!(!has(&dom, target, true));
}

#[test]
fn original_source_work_and_byte_refusals_preserve_index_and_allow_exact_retry() {
    let (mut dom, _, target) = fixture();
    let _survivor = retain(&mut dom, target, true);
    let owner = dom.event_recipients.as_ref().unwrap().clone();
    let cancel = AtomicBool::new(false);
    let mut draft = Draft::new(&dom, Arc::new(BatchIdentity), &cancel).unwrap();
    let ledger = draft.custody_ledger().clone();
    let before = ledger.total();
    let source = |bytes, work| DomSourceLimits::new(bytes, 1_000_000, work, 16, 4096, 4096, 1);
    draft.source_limits = Some(source(16_000_000, 6));
    let formation = Formation::new(&draft, &cancel).unwrap().unwrap();
    let processing = Processing::new(limits().event_recipients(), Some(&formation), &cancel);
    assert!(matches!(
        owner.retain(target.into(), 1, false, &processing),
        Err(DomError::SourceWorkLimit { limit: 6 })
    ));
    assert_eq!(draft.source_work.load(Ordering::Relaxed), 6);
    assert_eq!(ledger.total(), before);
    assert_eq!(owner.accounting.live.load(Ordering::Acquire), 1);
    drop(formation);
    let exact = before.bytes() + size_of::<DomEventRecipient>();
    draft.source_limits = Some(source(exact - 1, 1_000_000));
    let formation = Formation::new(&draft, &cancel).unwrap().unwrap();
    let processing = Processing::new(limits().event_recipients(), Some(&formation), &cancel);
    assert!(
        matches!(owner.retain(target.into(), 1, false, &processing), Err(DomError::ResourceLimit { resource: DomResource::SourceRetainedBytes, limit }) if limit == exact - 1)
    );
    assert!(draft.source_work.load(Ordering::Relaxed) > 6);
    assert_eq!(ledger.total(), before);
    drop(formation);
    draft.source_limits = Some(source(exact, 1_000_000));
    let formation = Formation::new(&draft, &cancel).unwrap().unwrap();
    let processing = Processing::new(limits().event_recipients(), Some(&formation), &cancel);
    let retry = owner.retain(target.into(), 1, false, &processing).unwrap();
    assert_eq!(
        ledger.total(),
        before
            .checked_add(custody::Usage::new(size_of::<DomEventRecipient>(), 0))
            .unwrap()
    );
    assert_eq!(owner.accounting.live.load(Ordering::Acquire), 2);
    drop(retry);
    assert_eq!(ledger.total(), before);
}

#[test]
fn parked_source_rejects_positive_admission_before_allocating_but_keeps_other_classes() {
    let (mut dom, document, target) = fixture();
    let cancel = AtomicBool::new(false);
    let parked = park_before_svg(&mut dom, document);
    assert!(matches!(
        dom.retain_event_recipient(target.into(), DomEventType::Error, true, &cancel),
        Err(crate::DomHostedError::Dom(DomError::SourceOperationPending))
    ));
    assert!(dom.event_recipients.is_none());
    assert!(
        dom.retain_event_recipient(target.into(), DomEventType::Toggle, false, &cancel)
            .unwrap()
            .is_none()
    );
    let mut lease = dom
        .resume_source_edit(parked.operation, source_limits(), &cancel)
        .unwrap();
    accept_block(&mut lease, &parked.request);
    let DomSourceProgress::Complete(done) = lease.advance(&cancel).unwrap() else {
        panic!("original insertion finishes after the actual blocking policy observation")
    };
    assert_eq!(done.operation(), parked.operation);
}

#[test]
fn parked_source_observes_lowered_recipient_policy_and_raised_original_tail_retry() {
    // The initial allowance also covers real command preflight under Restore
    // before the first Style publication. Restrict only the published tail.
    for scratch in [false, true] {
        let (mut dom, document, unrelated) = fixture();
        let _recipient = retain(&mut dom, unrelated, true);
        let parked = park_before_svg(&mut dom, document);
        let published = dom.snapshot().stamp();
        let low = DomEventRecipientLimits::new(
            16,
            4_000_000,
            if scratch { 0 } else { 4_000_000 },
            if scratch { 1_000_000 } else { 0 },
        )
        .unwrap();
        dom.set_limits(limits().with_event_recipients(low)).unwrap();
        assert_eq!(dom.snapshot().stamp(), published);
        let cancel = AtomicBool::new(false);
        let spent = {
            let mut lease = dom
                .resume_source_edit(parked.operation, source_limits(), &cancel)
                .unwrap();
            assert!(lease.status().spent_work() >= parked.status.spent_work());
            accept_block(&mut lease, &parked.request);
            let phases = lease.status().published_phases();
            let error = lease.advance(&cancel).unwrap_err();
            if scratch {
                assert_eq!(
                    error,
                    DomError::ResourceLimit {
                        resource: DomResource::EventRecipientScratch,
                        limit: 0
                    }
                );
            } else {
                assert_eq!(
                    error,
                    DomError::WorkLimit {
                        resource: DomResource::EventRecipientWork,
                        limit: 0
                    }
                );
            }
            assert_eq!(lease.operation(), parked.operation);
            assert_eq!(lease.status().published_phases(), phases);
            lease.status().spent_work()
        };
        assert_eq!(dom.snapshot().stamp(), published);
        assert_eq!(
            dom.snapshot().node(parked.script).unwrap().parent(),
            Some(parked.container)
        );
        assert!(
            !dom.snapshot()
                .node(parked.script)
                .unwrap()
                .as_svg_script()
                .unwrap()
                .flags()
                .already_processed()
        );
        dom.set_limits(limits()).unwrap();
        assert_eq!(dom.snapshot().stamp(), published);
        let mut lease = dom
            .resume_source_edit(parked.operation, source_limits(), &cancel)
            .unwrap();
        assert!(lease.status().spent_work() >= spent);
        let DomSourceProgress::Complete(done) = lease.advance(&cancel).unwrap() else {
            panic!("raise completes the original tail without reissuing the Style prefix")
        };
        assert_eq!(done.operation(), parked.operation);
        drop(lease);
        assert!(dom.pending_source_edit().is_none());
        assert!(
            dom.snapshot()
                .node(parked.script)
                .unwrap()
                .as_svg_script()
                .unwrap()
                .flags()
                .already_processed()
        );
        assert!(has(&dom, unrelated, false));
    }
}
