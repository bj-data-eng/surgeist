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
    let key = Key::Existing(document.0.slot);
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
fn formed_vacancy_frontier_prices_actual_capacity_and_simultaneous_checkpoint_copy() {
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
    assert_eq!(draft.range_vacancies.len(), 2);
    assert_eq!(draft.range_vacancies.peek(), Some(&Reverse(1)));
    // Initial formation owns the actual reserved backing even though only two
    // entries are eligible. Removing that one bank isolates its control debit.
    let frontier = std::mem::take(&mut draft.range_vacancies);
    let without = draft.source_storage_bytes(&cancel).unwrap();
    let bytes = frontier.capacity() * std::mem::size_of::<Reverse<usize>>();
    assert!(bytes >= 2 * std::mem::size_of::<usize>());
    draft.range_vacancies = frontier;
    assert_eq!(
        draft.source_storage_bytes(&cancel).unwrap(),
        without + bytes
    );
    // Independently reserve exactly the checkpoint's two requested entries.
    let mut probe = Vec::<Reverse<usize>>::new();
    probe.try_reserve_exact(2).unwrap();
    let copied_bytes = probe.capacity() * std::mem::size_of::<Reverse<usize>>();
    let baseline = draft.source_storage_bytes(&cancel).unwrap() + draft.source_other_bytes;
    let mut limit = policy;
    limit.retained_bytes = baseline + copied_bytes - 1;
    draft.source_limits = Some(limit);
    {
        let mut admission = source::checkpoint::Admission::new(&draft, baseline).unwrap();
        assert!(matches!(
            admission.copy(draft.range_vacancies.as_slice(), &cancel),
            Err(DomError::ResourceLimit {
                resource: DomResource::SourceRetainedBytes,
                ..
            })
        ));
    }
    assert_eq!(draft.select_temporary_bytes.load(Ordering::Relaxed), 0);
    limit.retained_bytes += 1;
    draft.source_limits = Some(limit);
    {
        let mut admission = source::checkpoint::Admission::new(&draft, baseline).unwrap();
        let copied = admission
            .copy(draft.range_vacancies.as_slice(), &cancel)
            .unwrap();
        let frontier = form_vacancies(copied, || draft.source_step(&cancel)).unwrap();
        assert_eq!(frontier.into_sorted_vec(), [Reverse(7), Reverse(1)]);
        let charge = admission.finish();
        assert_eq!(
            draft.select_temporary_bytes.load(Ordering::Relaxed),
            copied_bytes
        );
        drop(charge);
    }
    assert_eq!(draft.select_temporary_bytes.load(Ordering::Relaxed), 0);
}
