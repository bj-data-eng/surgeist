//! Real public creation exercises the owning allocator's actual slot reads.
use super::*;

fn occupied_ranges() -> (Dom, DocumentId, Vec<DomRangeId>) {
    let cancel = AtomicBool::new(false);
    let limits = DomLimits::new(4, 4096, 256, 0, 0, 16, 1_000_000)
        .unwrap()
        .with_ranges(128)
        .unwrap();
    let mut dom = Dom::new(limits).unwrap();
    let mut batch = DomBatch::new();
    let document = batch.create_document();
    let ranges: Vec<_> = (0..64).map(|_| batch.create_range(&document)).collect();
    let commit = dom.apply(batch, &cancel).unwrap();
    let document = commit.document(&document).unwrap();
    let ranges = ranges
        .iter()
        .map(|ticket| commit.created_range(ticket).unwrap().unwrap())
        .collect();
    (dom, document, ranges)
}

#[test]
fn bulk_range_creation_does_not_revisit_an_occupied_canonical_prefix() {
    let cancel = AtomicBool::new(false);
    let limits = DomLimits::new(4, 4096, 256, 0, 0, 16, 1_000_000)
        .unwrap()
        .with_ranges(128)
        .unwrap();
    let mut dom = Dom::new(limits).unwrap();
    let mut setup = DomBatch::new();
    let document = setup.create_document();
    let original: Vec<_> = (0..64).map(|_| setup.create_range(&document)).collect();
    let commit = dom.apply(setup, &cancel).unwrap();
    let document = commit.document(&document).unwrap();
    let original: Vec<_> = original
        .iter()
        .map(|ticket| commit.created_range(ticket).unwrap().unwrap())
        .collect();
    let captured = dom.snapshot();

    let mut batch = DomBatch::new();
    let tickets: Vec<_> = (0..32).map(|_| batch.create_range(document)).collect();
    let observation = allocation_observation::start();
    let commit = dom.apply(batch, &cancel).unwrap();
    let visits = observation.finish();
    let created: Vec<_> = tickets
        .iter()
        .map(|ticket| commit.created_range(ticket).unwrap().unwrap())
        .collect();
    assert_eq!(
        created.iter().map(|id| id.slot).collect::<Vec<_>>(),
        (64..96).collect::<Vec<_>>()
    );
    assert!(created.iter().all(|id| id.generation == 1));
    for id in original {
        assert_eq!(
            captured.range(id).unwrap().start(),
            dom.snapshot().range(id).unwrap().start()
        );
    }
    assert!(
        created
            .iter()
            .all(|id| matches!(captured.range(*id), Err(DomError::StaleRange)))
    );
    assert!(
        visits <= 32,
        "32 creations must not repeatedly read the 64 occupied Range records; actual allocator eligibility visits: {visits}"
    );
}

#[test]
fn alternating_low_retirement_reuse_and_append_does_not_rescan_the_occupied_suffix() {
    let cancel = AtomicBool::new(false);
    let (mut dom, document, original) = occupied_ranges();
    let captured = dom.snapshot();
    let mut batch = DomBatch::new();
    let mut retiring = DomRangeOperand::Existing(original[0]);
    let mut reused = Vec::new();
    let mut appended = Vec::new();
    for _ in 0..16 {
        batch.retire_range(retiring);
        let ticket = batch.create_range(document);
        retiring = (&ticket).into();
        reused.push(ticket);
        appended.push(batch.create_range(document));
    }
    let observation = allocation_observation::start();
    let commit = dom.apply(batch, &cancel).unwrap();
    let visits = observation.finish();
    for ticket in &reused[..15] {
        assert_eq!(commit.created_range(ticket).unwrap(), None);
    }
    let last = commit.created_range(&reused[15]).unwrap().unwrap();
    assert_eq!((last.slot, last.generation), (0, 17));
    let appended: Vec<_> = appended
        .iter()
        .map(|ticket| commit.created_range(ticket).unwrap().unwrap())
        .collect();
    assert_eq!(
        appended
            .iter()
            .map(|id| (id.slot, id.generation))
            .collect::<Vec<_>>(),
        (64..80).map(|slot| (slot, 1)).collect::<Vec<_>>()
    );
    assert!(captured.range(original[0]).is_ok());
    assert!(matches!(
        dom.snapshot().range(original[0]),
        Err(DomError::StaleRange)
    ));
    assert!(
        visits <= 32,
        "alternating low-slot reuse and append must not restart suffix scans; actual allocator eligibility visits: {visits}"
    );
}

#[test]
fn original_source_checkpoint_restores_allocation_order_without_refunding_work() {
    let cancel = AtomicBool::new(false);
    let (dom, document, original) = occupied_ranges();
    let commands = DomBatch::new();
    let policy = crate::DomSourceLimits::new(16_000_000, 100_000, 10_000_000, 128, 4096, 4096, 1);
    let mut draft = Draft::new_source(
        &dom,
        commands.owner.clone(),
        &commands.commands,
        policy,
        &cancel,
    )
    .unwrap()
    .0;
    // Capture an already populated frontier, rather than an empty allocator.
    draft.retire_range(original[0], &cancel).unwrap();
    draft.retire_range(original[2], &cancel).unwrap();
    let checkpoint = draft
        .source_checkpoint(draft.source_other_bytes, 0, &cancel)
        .unwrap();
    let key = Key::Existing(document.0);
    for ticket in 0..3 {
        draft
            .create_range(key, ticket, dom.limits, &cancel)
            .unwrap();
    }
    assert_eq!(
        draft
            .range_tickets
            .iter()
            .map(|id| (id.slot, id.generation))
            .collect::<Vec<_>>(),
        [(0, 2), (2, 2), (64, 1)]
    );
    let spent = draft.source_work.load(Ordering::Relaxed);
    checkpoint.restore(&mut draft);
    assert_eq!(draft.source_work.load(Ordering::Relaxed), spent);
    assert!(draft.range_tickets.is_empty());
    assert_eq!(draft.ranges.len(), 64);
    assert!(matches!(
        draft.range_record(original[0]),
        Err(DomError::StaleRange)
    ));
    assert!(matches!(
        draft.range_record(original[2]),
        Err(DomError::StaleRange)
    ));
    for ticket in 0..3 {
        draft
            .create_range(key, ticket, dom.limits, &cancel)
            .unwrap();
    }
    assert_eq!(
        draft
            .range_tickets
            .iter()
            .map(|id| (id.slot, id.generation))
            .collect::<Vec<_>>(),
        [(0, 2), (2, 2), (64, 1)]
    );
    assert!(draft.source_work.load(Ordering::Relaxed) > spent);
}

#[test]
fn sparse_vacancy_root_is_shared_by_checkpoint_and_restores_exact_lowest_slot() {
    let cancel = AtomicBool::new(false);
    let (mut dom, _, original) = occupied_ranges();
    let mut retirement = DomBatch::new();
    retirement.retire_range(original[1]);
    retirement.retire_range(original[7]);
    dom.apply(retirement, &cancel).unwrap();
    let commands = DomBatch::new();
    let policy = crate::DomSourceLimits::new(16_000_000, 100_000, 10_000_000, 128, 4096, 4096, 1);
    let mut draft = Draft::new_source(
        &dom,
        commands.owner.clone(),
        &commands.commands,
        policy,
        &cancel,
    )
    .unwrap()
    .0;
    assert_eq!(draft.ranges.lowest_vacant(), Some(1));
    assert_eq!(draft.ranges.touched().count(), 0);
    let before = draft.ranges.allocation_graph();
    let before_pages: Vec<_> = before.pages().collect();
    let checkpoint = draft
        .source_checkpoint(draft.source_other_bytes, 0, &cancel)
        .unwrap();
    let captured = draft.ranges.allocation_graph();
    assert_eq!(
        captured.pages().collect::<Vec<_>>(),
        before.pages().collect::<Vec<_>>()
    );
    let document = draft.range_record(original[0]).unwrap().start.container;
    draft
        .create_range(document, 0, dom.limits, &cancel)
        .unwrap();
    assert_eq!(draft.range_tickets[0].slot, 1);
    assert_eq!(draft.ranges.lowest_vacant(), Some(7));
    let spent = draft.source_work.load(Ordering::Relaxed);
    checkpoint.restore(&mut draft);
    assert_eq!(draft.source_work.load(Ordering::Relaxed), spent);
    assert!(draft.range_tickets.is_empty());
    assert_eq!(draft.ranges.lowest_vacant(), Some(1));
    let restored = draft.ranges.allocation_graph();
    assert_eq!(restored.pages().collect::<Vec<_>>(), before_pages);
    draft
        .create_range(document, 0, dom.limits, &cancel)
        .unwrap();
    assert_eq!(draft.range_tickets[0].slot, 1);
}
