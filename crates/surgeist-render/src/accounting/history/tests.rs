use super::*;
use crate::{
    GlyphRunLimits, ImageId, RenderAccountingContext, RenderAccountingOptions, RenderAttemptKind,
    RenderLimits, RenderProfile, RenderStandaloneReport, Size,
};
use std::mem::size_of;

mod ownership;

fn attempt<T>(
    limits: RenderLimits,
    run: impl FnOnce(&Account) -> Result<T>,
) -> (Result<T>, RenderStandaloneReport) {
    let mut context =
        RenderAccountingContext::new(RenderAccountingOptions::new().with_limits(limits));
    let result = context.prepare(
        RenderAttemptKind::Frame,
        GlyphRunLimits::new(usize::MAX, usize::MAX, usize::MAX, usize::MAX),
        #[cfg(feature = "render-text")]
        None,
        run,
    );
    (
        result,
        context.take_last_attempt().expect("started attempt"),
    )
}

fn unlimited<T>(run: impl FnOnce(&Account) -> Result<T>) -> T {
    attempt(RenderLimits::new(), run).0.unwrap()
}

fn cpu() -> RendererCpu {
    RendererCpu::new(
        RenderAccountingOptions::new().with_profile(RenderProfile::Summary { timing: false }),
    )
    .unwrap()
}

fn image(id: u64, hash: u64, size: Size, bytes: Arc<[u8]>) -> Image {
    Image::from_rgba_with_id_for_test(size, bytes, ImageId::new(id))
        .unwrap()
        .with_content_hash_for_test(hash)
}

fn pixel(id: u64, hash: u64, bytes: [u8; 4]) -> Image {
    image(id, hash, Size::new(1.0, 1.0), Arc::from(bytes))
}

fn publish(history: &ImageHistory, images: &[&Image]) {
    let draft = unlimited(|account| {
        let draft = history.begin(account)?;
        let observer = draft.observer();
        for image in images {
            observer.observe(image, account)?;
        }
        Ok(draft)
    });
    draft
        .into_publication()
        .unwrap()
        .retire_old()
        .unwrap()
        .commit();
}

fn canonical(history: &ImageHistory) -> Vec<(u64, Arc<[u8]>)> {
    history
        .inner
        .lock()
        .canonical_for_test()
        .map(|record| {
            (
                record.identity.fingerprint().get(),
                Arc::clone(record.identity.source_bytes()),
            )
        })
        .collect()
}

fn canonical_capacity(history: &ImageHistory) -> usize {
    let state = history.inner.lock();
    match &state.custody {
        Custody::Canonical
        | Custody::Transaction {
            baseline: BaselineCustody::DraftOnly,
            ..
        } => state.records.capacity(),
        Custody::Transaction {
            baseline: BaselineCustody::Retained { old, .. },
            ..
        } => old.capacity(),
    }
}

fn candidate_capacity(history: &ImageHistory) -> usize {
    let state = history.inner.lock();
    assert!(matches!(&state.custody, Custody::Transaction { .. }));
    state.records.capacity()
}

fn retained(cpu: &RendererCpu) -> u64 {
    cpu.snapshot()
        .unwrap()
        .usage()
        .live(RenderResource::RetainedCpuBytes)
}

#[test]
fn independent_equal_content_hits_and_preserves_the_first_source() {
    let cpu = cpu();
    let history = ImageHistory::new(&cpu).unwrap();
    let first = pixel(7, 11, [1, 2, 3, 4]);
    let equal = pixel(99, 11, [1, 2, 3, 4]);
    assert!(!Arc::ptr_eq(&first.bytes, &equal.bytes));
    let draft = unlimited(|account| {
        let draft = history.begin(account)?;
        let observer = draft.observer();
        assert!(observer.observe(&first, account)?);
        assert!(!observer.observe(&equal, account)?);
        assert!(!observer.observe(&first, account)?);
        Ok(draft)
    });
    draft
        .into_publication()
        .unwrap()
        .retire_old()
        .unwrap()
        .commit();
    let entries = canonical(&history);
    assert_eq!(entries.len(), 1);
    assert_eq!(entries[0].0, 7);
    assert!(Arc::ptr_eq(&entries[0].1, &first.bytes));
    publish(&history, &[&equal]);
    let entries = canonical(&history);
    assert_eq!(entries[0].0, 7);
    assert!(Arc::ptr_eq(&entries[0].1, &first.bytes));
}

#[test]
fn hash_collisions_compare_the_actual_byte_prefix_and_insert_distinct_content() {
    let cpu = cpu();
    let history = ImageHistory::new(&cpu).unwrap();
    let first = pixel(1, 3, [1, 2, 3, 4]);
    let different = pixel(2, 3, [1, 2, 9, 4]);
    let (draft, report) = attempt(RenderLimits::new(), |account| {
        let draft = history.begin(account)?;
        let observer = draft.observer();
        assert!(observer.observe(&first, account)?);
        assert!(observer.observe(&different, account)?);
        Ok(draft)
    });
    // Each observation writes one semantic record. The intervening relocation
    // belongs to the CPU history owner, outside the forward copy allowance.
    assert_eq!(
        report
            .attempt()
            .unwrap()
            .usage()
            .work(RenderWork::ImageIdentityCopies),
        2
    );
    assert_eq!(
        report
            .attempt()
            .unwrap()
            .usage()
            .work(RenderWork::StorageRelocations),
        0
    );
    assert_eq!(
        cpu.snapshot()
            .unwrap()
            .scope_usage(super::super::RenderCpuScope::ImageHistory)
            .unwrap()
            .work(RenderWork::StorageRelocations),
        1
    );
    assert_eq!(
        report
            .attempt()
            .unwrap()
            .usage()
            .work(RenderWork::ImageIdentityByteComparisons),
        3
    );
    draft
        .unwrap()
        .into_publication()
        .unwrap()
        .retire_old()
        .unwrap()
        .commit();
    assert_eq!(
        canonical(&history).iter().map(|x| x.0).collect::<Vec<_>>(),
        [1, 2]
    );
}

#[test]
fn pointer_equal_hit_inspects_two_history_candidates_without_scanning_bytes() {
    let cpu = cpu();
    let history = ImageHistory::new(&cpu).unwrap();
    let first = pixel(1, 3, [1, 2, 3, 4]);
    let (_, report) = attempt(RenderLimits::new(), |account| {
        let draft = history.begin(account)?;
        let observer = draft.observer();
        assert!(observer.observe(&first, account)?);
        assert!(!observer.observe(&first, account)?);
        Ok(())
    });
    assert_eq!(
        report
            .attempt()
            .unwrap()
            .usage()
            .work(RenderWork::ResourceProbes),
        2
    );
    assert_eq!(
        report
            .attempt()
            .unwrap()
            .usage()
            .work(RenderWork::ImageIdentityByteComparisons),
        0
    );
    assert_eq!(canonical(&history).len(), 0);
}

#[test]
fn byte_comparison_refusal_keeps_the_admitted_prefix_and_publishes_nothing() {
    let cpu = cpu();
    let history = ImageHistory::new(&cpu).unwrap();
    let first = pixel(1, 3, [1, 2, 3, 4]);
    let different = pixel(2, 3, [1, 2, 9, 4]);
    let limits = RenderLimits::new()
        .with_work_limit(RenderWork::ImageIdentityByteComparisons, 2)
        .unwrap();
    let (result, report) = attempt(limits, |account| {
        let draft = history.begin(account)?;
        let observer = draft.observer();
        assert!(observer.observe(&first, account)?);
        observer.observe(&different, account)
    });
    assert!(result.is_err());
    assert_eq!(
        report
            .attempt()
            .unwrap()
            .usage()
            .work(RenderWork::ImageIdentityByteComparisons),
        2
    );
    assert_eq!(canonical(&history).len(), 0);
    assert_eq!(
        cpu.snapshot()
            .unwrap()
            .usage()
            .work(RenderWork::ImageIdentityDrops),
        1
    );
}

#[test]
fn different_dimensions_share_one_payload_but_keep_two_exact_contents() {
    let cpu = cpu();
    let history = ImageHistory::new(&cpu).unwrap();
    let bytes: Arc<[u8]> = Arc::from([1, 2, 3, 4, 5, 6, 7, 8]);
    let wide = image(1, 3, Size::new(2.0, 1.0), Arc::clone(&bytes));
    let tall = image(2, 3, Size::new(1.0, 2.0), bytes);
    publish(&history, &[&wide]);
    let before = cpu.snapshot().unwrap();
    let payload_before = before.scope_usage(RenderCpuScope::ImagePayloads).unwrap();
    publish(&history, &[&tall]);
    let after = cpu.snapshot().unwrap();
    let payload_after = after.scope_usage(RenderCpuScope::ImagePayloads).unwrap();
    assert_eq!(canonical(&history).len(), 2);
    assert_eq!(
        payload_before.live(RenderResource::RetainedCpuBytes),
        payload_after.live(RenderResource::RetainedCpuBytes),
    );
    assert!(
        canonical(&history)
            .iter()
            .all(|(_, source)| Arc::ptr_eq(source, &wide.bytes))
    );
}

#[test]
fn baseline_draft_and_growth_charge_actual_metadata_overlap_once_per_allocation() {
    let cpu = cpu();
    let history = ImageHistory::new(&cpu).unwrap();
    let bytes: Arc<[u8]> = Arc::from([1, 2, 3, 4, 5, 6, 7, 8]);
    let wide = image(1, 3, Size::new(2.0, 1.0), Arc::clone(&bytes));
    let tall = image(2, 3, Size::new(1.0, 2.0), bytes);
    publish(&history, &[&wide]);
    let baseline = retained(&cpu);
    let old_capacity = canonical_capacity(&history);
    assert_eq!(old_capacity, 1);
    let draft = unlimited(|account| {
        let draft = history.begin(account)?;
        let draft_capacity = candidate_capacity(&history);
        assert_eq!(
            retained(&cpu),
            baseline + (draft_capacity * size_of::<ContentRecord>()) as u64
        );
        assert!(draft.observer().observe(&tall, account)?);
        Ok(draft)
    });
    let draft_capacity = candidate_capacity(&history);
    assert_eq!(draft_capacity, 2);
    let snapshot = cpu.snapshot().unwrap();
    assert_eq!(
        snapshot.usage().live(RenderResource::RetainedCpuBytes),
        baseline + (2 * size_of::<ContentRecord>()) as u64
    );
    assert_eq!(
        snapshot.usage().peak(RenderResource::RetainedCpuBytes),
        baseline + (3 * size_of::<ContentRecord>()) as u64
    );
    let ready = draft.into_publication().unwrap().retire_old().unwrap();
    assert_eq!(retained(&cpu), baseline + size_of::<ContentRecord>() as u64);
    ready.commit();
}

#[test]
fn one_under_draft_capacity_refuses_without_changing_the_existing_baseline() {
    let sample_cpu = cpu();
    let sample = ImageHistory::new(&sample_cpu).unwrap();
    let first = pixel(7, 11, [1, 2, 3, 4]);
    publish(&sample, &[&first]);
    let baseline = retained(&sample_cpu);
    let draft_bytes = size_of::<ContentRecord>() as u64;
    for (cap, accepted) in [
        (baseline + draft_bytes - 1, false),
        (baseline + draft_bytes, true),
    ] {
        let cpu = RendererCpu::new(
            RenderAccountingOptions::new()
                .with_limits(RenderLimits::new().with_retained_bytes(cap)),
        )
        .unwrap();
        let history = ImageHistory::new(&cpu).unwrap();
        publish(&history, &[&first]);
        assert_eq!(retained(&cpu), baseline);
        let (result, _) = attempt(RenderLimits::new(), |account| history.begin(account));
        assert_eq!(result.is_ok(), accepted);
        drop(result);
        assert_eq!(canonical(&history)[0].0, 7);
        assert_eq!(retained(&cpu), baseline);
        cpu.ensure_healthy().unwrap();
    }
}

#[test]
fn refused_insertion_releases_its_provisional_import_and_keeps_real_growth_capacity() {
    let cpu = cpu();
    let history = ImageHistory::new(&cpu).unwrap();
    let first = pixel(1, 10, [1, 2, 3, 4]);
    let second = pixel(2, 20, [5, 6, 7, 8]);
    publish(&history, &[&first]);
    let caller_count = Arc::strong_count(&second.bytes);
    // The baseline clone consumes one forward copy. Growth is CPU lifetime
    // relocation, so a one-copy cap refuses the new-slot admission after its
    // real payload import has been acquired.
    let limits = RenderLimits::new()
        .with_work_limit(RenderWork::ImageIdentityCopies, 1)
        .unwrap();
    let (result, report) = attempt(limits, |account| {
        let draft = history.begin(account)?;
        draft.observer().observe(&second, account)
    });
    assert!(result.is_err());
    assert_eq!(
        report
            .attempt()
            .unwrap()
            .usage()
            .work(RenderWork::ImageIdentityCopies),
        1
    );
    assert_eq!(
        report
            .attempt()
            .unwrap()
            .usage()
            .work(RenderWork::StorageRelocations),
        0
    );
    assert_eq!(
        cpu.snapshot()
            .unwrap()
            .scope_usage(super::super::RenderCpuScope::ImageHistory)
            .unwrap()
            .work(RenderWork::StorageRelocations),
        1
    );
    assert_eq!(Arc::strong_count(&second.bytes), caller_count);
    assert_eq!(
        canonical(&history).iter().map(|x| x.0).collect::<Vec<_>>(),
        [1]
    );
    cpu.ensure_healthy().unwrap();
    // A new attempt has fresh forward limits and can import that source again.
    publish(&history, &[&second]);
    assert_eq!(canonical(&history).len(), 2);
}

#[test]
fn exhausted_forward_copy_budget_cannot_veto_cancellation_cleanup() {
    let cpu = cpu();
    let history = ImageHistory::new(&cpu).unwrap();
    let first = pixel(1, 10, [1, 2, 3, 4]);
    publish(&history, &[&first]);
    let before = retained(&cpu);
    let limits = RenderLimits::new()
        .with_work_limit(RenderWork::ImageIdentityCopies, 1)
        .unwrap();
    let (result, report) = attempt(limits, |account| {
        let draft = history.begin(account)?;
        drop(draft);
        Ok(())
    });
    result.unwrap();
    assert_eq!(
        report
            .attempt()
            .unwrap()
            .usage()
            .work(RenderWork::ImageIdentityCopies),
        1
    );
    assert_eq!(retained(&cpu), before);
    assert_eq!(canonical(&history)[0].0, 1);
    assert!(
        cpu.snapshot()
            .unwrap()
            .usage()
            .work(RenderWork::ImageIdentityDrops)
            >= 1
    );
}

#[test]
fn late_abort_restores_only_canonical_baseline_after_old_vector_disposal() {
    let cpu = cpu();
    let history = ImageHistory::new(&cpu).unwrap();
    let bytes: Arc<[u8]> = Arc::from([1, 2, 3, 4, 5, 6, 7, 8]);
    let old_a = image(7, 20, Size::new(2.0, 1.0), Arc::clone(&bytes));
    let old_b = pixel(9, 40, [4, 3, 2, 1]);
    let new_a = image(80, 10, Size::new(1.0, 2.0), bytes);
    let new_b = pixel(90, 30, [9, 8, 7, 6]);
    let equal_a = image(
        100,
        20,
        Size::new(2.0, 1.0),
        Arc::from([1, 2, 3, 4, 5, 6, 7, 8]),
    );
    publish(&history, &[&old_a, &old_b]);
    let baseline = canonical(&history);
    let draft = unlimited(|account| {
        let draft = history.begin(account)?;
        let observer = draft.observer();
        assert!(observer.observe(&new_a, account)?);
        assert!(observer.observe(&new_b, account)?);
        assert!(!observer.observe(&equal_a, account)?);
        Ok(draft)
    });
    let larger_capacity = candidate_capacity(&history);
    let checkpoint = cpu.begin_interval().unwrap();
    let ready = draft.into_publication().unwrap().retire_old().unwrap();
    let closed = checkpoint.finish().unwrap();
    assert_eq!(closed.usage().work(RenderWork::ImageIdentityDrops), 2);
    drop(ready); // A later phase/checkpoint/projection fault or cancellation.
    let restored = canonical(&history);
    assert_eq!(restored.iter().map(|x| x.0).collect::<Vec<_>>(), [7, 9]);
    for (restored, original) in restored.iter().zip(&baseline) {
        assert!(Arc::ptr_eq(&restored.1, &original.1));
    }
    assert_eq!(canonical_capacity(&history), larger_capacity);
    assert_eq!(closed.usage().work(RenderWork::ImageIdentityDrops), 2);
    assert_eq!(
        cpu.snapshot()
            .unwrap()
            .usage()
            .work(RenderWork::ImageIdentityDrops),
        4
    );
    assert_eq!(
        cpu.snapshot()
            .unwrap()
            .usage()
            .work(RenderWork::ImageIdentityCopies),
        4
    );
    cpu.ensure_healthy().unwrap();
}

#[test]
fn mandatory_drop_overflow_preserves_first_fault_and_restores_baseline() {
    let cpu = cpu();
    let history = ImageHistory::new(&cpu).unwrap();
    let first = pixel(7, 10, [1, 2, 3, 4]);
    let second = pixel(9, 20, [4, 3, 2, 1]);
    publish(&history, &[&first]);
    let original = canonical(&history);
    let draft = unlimited(|account| {
        let draft = history.begin(account)?;
        assert!(draft.observer().observe(&second, account)?);
        Ok(draft)
    });
    cpu.at(RenderCpuScope::Cleanup)
        .maintenance(RenderWork::ImageIdentityDrops, u64::MAX);
    assert!(draft.into_publication().unwrap().retire_old().is_err());
    let fault = cpu.snapshot().unwrap().fault().expect("mandatory overflow");
    let restored = canonical(&history);
    assert_eq!(restored.len(), 1);
    assert_eq!(restored[0].0, 7);
    assert!(Arc::ptr_eq(&restored[0].1, &original[0].1));
    drop(history);
    assert_eq!(cpu.snapshot().unwrap().fault(), Some(fault));
}

#[test]
fn expired_observer_cannot_mutate_a_later_draft_or_retain_history_payloads() {
    let cpu = cpu();
    let owner_baseline = retained(&cpu);
    let history = ImageHistory::new(&cpu).unwrap();
    let first = pixel(7, 10, [1, 2, 3, 4]);
    let (observer, draft) = unlimited(|account| {
        let draft = history.begin(account)?;
        Ok((draft.observer(), draft))
    });
    drop(draft);
    let next = unlimited(|account| history.begin(account));
    assert!(
        attempt(RenderLimits::new(), |account| observer
            .observe(&first, account))
        .0
        .is_err()
    );
    drop(next);
    assert_eq!(canonical(&history).len(), 0);
    drop(history);
    assert!(
        attempt(RenderLimits::new(), |account| observer
            .observe(&first, account))
        .0
        .is_err()
    );
    assert_eq!(retained(&cpu), owner_baseline);
}
