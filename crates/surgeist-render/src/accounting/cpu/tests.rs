use super::*;
use crate::accounting::{
    AccountedVec, RenderClock, RenderLimits, RenderProfile, StorageAccount, reserve_backing_with,
};
use std::{
    mem::size_of,
    sync::atomic::{AtomicU64, Ordering},
    time::Duration,
};

fn retained(cpu: &RendererCpu) -> u64 {
    cpu.snapshot()
        .unwrap()
        .usage()
        .live(RenderResource::RetainedCpuBytes)
}
fn baseline() -> u64 {
    size_of::<State>() as u64
}
fn entry_bytes() -> u64 {
    size_of::<Entry>() as u64
}
fn image(bytes: [u8; 4]) -> Image {
    Image::from_rgba(crate::Size::new(1.0, 1.0), Arc::<[u8]>::from(bytes)).unwrap()
}

#[test]
fn vector_growth_counts_actual_old_and_new_capacity_overlap_and_releases() {
    let cpu = RendererCpu::new(RenderAccountingOptions::default()).unwrap();
    let account = cpu.at(RenderCpuScope::FrameBacking);
    let checkpoint = cpu.begin_interval().unwrap();
    let mut values = AccountedVec::<u32>::new_in(
        &StorageAccount::Renderer(account.clone()),
        2,
        RenderResource::RetainedCpuBytes,
    )
    .unwrap();
    let old = (values.capacity() * 4) as u64;
    values.push(11).unwrap();
    values.push(22).unwrap();
    values.push(33).unwrap();
    let new = (values.capacity() * 4) as u64;
    assert_eq!(values.as_slice(), &[11, 22, 33]);
    let interval = checkpoint.finish().unwrap();
    assert_eq!(
        interval
            .usage()
            .opening_live(RenderResource::RetainedCpuBytes),
        baseline()
    );
    assert_eq!(
        interval.usage().live(RenderResource::RetainedCpuBytes),
        baseline() + new
    );
    assert_eq!(
        interval.usage().peak(RenderResource::RetainedCpuBytes),
        baseline() + old + new
    );
    drop(values);
    assert_eq!(retained(&cpu), baseline());
    // A closed interval keeps its real earlier closing boundary.
    assert_eq!(
        interval.usage().live(RenderResource::RetainedCpuBytes),
        baseline() + new
    );
}

#[test]
fn refused_growth_preserves_values_and_does_not_admit_a_fake_peak() {
    let cpu = RendererCpu::new(
        RenderAccountingOptions::default().with_limits(RenderLimits::new().with_scratch_bytes(19)),
    )
    .unwrap();
    let storage = StorageAccount::Renderer(cpu.at(RenderCpuScope::FrameBacking));
    let mut values = AccountedVec::new_in(&storage, 2, RenderResource::ScratchCpuBytes).unwrap();
    values.push(1u32).unwrap();
    values.push(2u32).unwrap();
    assert!(values.push(3u32).is_err());
    assert_eq!(values.as_slice(), &[1, 2]);
    let snapshot = cpu.snapshot().unwrap();
    assert_eq!(snapshot.usage().live(RenderResource::ScratchCpuBytes), 8);
    assert_eq!(snapshot.usage().peak(RenderResource::ScratchCpuBytes), 8);
    assert_eq!(snapshot.fault(), None);
    assert!(cpu.ensure_healthy().is_ok());
}

#[test]
fn producer_backing_reserves_before_move_and_keeps_its_exact_hold() {
    let cpu = RendererCpu::new(RenderAccountingOptions::default()).unwrap();
    let storage = StorageAccount::Renderer(cpu.at(RenderCpuScope::FrameBacking));
    let mut values = Vec::<u32>::new();
    let mut hold = None;
    reserve_backing_with(
        &mut values,
        &mut hold,
        2,
        &storage,
        RenderResource::RetainedCpuBytes,
        |_| Ok(()),
    )
    .unwrap();
    values.extend([7, 9]);
    let old = (values.capacity() * 4) as u64;
    reserve_backing_with(
        &mut values,
        &mut hold,
        3,
        &storage,
        RenderResource::RetainedCpuBytes,
        |_| Ok(()),
    )
    .unwrap();
    let new = (values.capacity() * 4) as u64;
    assert_eq!(values, [7, 9]);
    assert_eq!(retained(&cpu), baseline() + new);
    assert_eq!(
        cpu.snapshot()
            .unwrap()
            .usage()
            .peak(RenderResource::RetainedCpuBytes),
        baseline() + old + new
    );
    drop(values);
    drop(hold);
    assert_eq!(retained(&cpu), baseline());
}

#[test]
fn shared_arc_payload_counts_once_but_independent_equal_allocations_count_twice() {
    let cpu = RendererCpu::new(RenderAccountingOptions::default()).unwrap();
    let account = cpu.at(RenderCpuScope::ImagePayloads);
    let first = image([1, 2, 3, 4]);
    let equal = image([1, 2, 3, 4]);
    let lease = account.import_image(&first, None).unwrap();
    assert_eq!(retained(&cpu), baseline() + entry_bytes() + 4);
    let duplicate = account.import_image(&first, None).unwrap();
    let cloned = lease.try_clone_with(None).unwrap();
    assert_eq!(retained(&cpu), baseline() + entry_bytes() + 4);
    let separate = account.import_image(&equal, None).unwrap();
    assert_eq!(retained(&cpu), baseline() + 2 * entry_bytes() + 8);
    drop(first);
    drop(lease);
    drop(duplicate);
    assert_eq!(retained(&cpu), baseline() + 2 * entry_bytes() + 8);
    drop(cloned);
    assert_eq!(retained(&cpu), baseline() + 2 * entry_bytes() + 4);
    drop(separate);
    // Registry capacity remains real after semantic entries are removed.
    assert_eq!(retained(&cpu), baseline() + 2 * entry_bytes());
    assert_eq!(cpu.snapshot().unwrap().fault(), None);
}

#[test]
fn producer_blob_bridge_shares_original_arc_and_refuses_unknown_source() {
    let cpu = RendererCpu::new(RenderAccountingOptions::default()).unwrap();
    let account = cpu.at(RenderCpuScope::ImagePayloads);
    let source = image([5, 6, 7, 8]);
    let original = account.import_image(&source, None).unwrap();
    let blob = account.import_image_data(&source.data, None).unwrap();
    assert_eq!(retained(&cpu), baseline() + entry_bytes() + 4);
    drop(original);
    assert_eq!(retained(&cpu), baseline() + entry_bytes() + 4);
    drop(blob);
    assert_eq!(retained(&cpu), baseline() + entry_bytes());
    let unknown = image([9, 9, 9, 9]);
    assert!(account.import_image_data(&unknown.data, None).is_err());
    assert_eq!(cpu.fault(), Some(RenderAccountingError::Invariant));
}

#[test]
fn failed_payload_admission_rolls_back_lease_but_keeps_real_registry_growth() {
    let limit = baseline() + entry_bytes() + 3;
    let cpu = RendererCpu::new(
        RenderAccountingOptions::default()
            .with_limits(RenderLimits::new().with_retained_bytes(limit)),
    )
    .unwrap();
    let account = cpu.at(RenderCpuScope::ImagePayloads);
    let error = account
        .import_image(&image([1, 2, 3, 4]), None)
        .unwrap_err();
    assert!(
        matches!(error.accounting_diagnostic(), Some(RenderAccountingError::CpuRefused {
        scope: RenderCpuScope::Root,
        quantity: RenderQuantity::Resource(RenderResource::RetainedCpuBytes),
        amount: 4,
        reason: RenderRefusalReason::Limit { limit: observed },
        ..
    }) if observed == limit)
    );
    assert_eq!(retained(&cpu), baseline() + entry_bytes());
    assert_eq!(
        cpu.snapshot()
            .unwrap()
            .usage()
            .peak(RenderResource::RetainedCpuBytes),
        baseline() + entry_bytes()
    );
    assert!(account.import_image(&image([1, 2, 3, 4]), None).is_err());
    assert!(account.owner().unwrap().lock().entries.is_empty());
    assert_eq!(cpu.fault(), None);
}

#[test]
fn imports_keep_lifetime_custody_without_owner_cycles_or_caller_tokens() {
    let cpu = RendererCpu::new(RenderAccountingOptions::default()).unwrap();
    let ledger = cpu.0.core.ledger.clone();
    let root = cpu.0.core.scopes[RenderCpuScope::Root.index()];
    let caller = image([1, 1, 1, 1]);
    let import = cpu
        .at(RenderCpuScope::ImagePayloads)
        .import_image(&caller, None)
        .unwrap();
    drop(cpu);
    assert_eq!(
        ledger
            .usage(root)
            .unwrap()
            .live(RenderResource::RetainedCpuBytes.dimension()),
        baseline() + entry_bytes() + 4
    );
    drop(import);
    assert_eq!(
        ledger
            .usage(root)
            .unwrap()
            .live(RenderResource::RetainedCpuBytes.dimension()),
        0
    );
    // Ordinary caller values remain valid after Render attribution ends.
    assert_eq!(&*caller.bytes, &[1, 1, 1, 1]);
}

#[test]
fn distinct_preparations_share_font_payload_and_cloned_preparation_shares_own_backing() {
    let font = crate::FontData::try_from_bytes(
        include_bytes!("../../../tests/fixtures/fonts/ahem/Ahem.ttf").to_vec(),
        0,
        surgeist_test::fixtures::FONT_LIMITS,
    )
    .unwrap();
    let instance =
        crate::TextInstance::try_new(20.0, vec![], crate::GlyphSynthesis::none()).unwrap();
    let mut preparation = super::super::RenderAccountingContext::default();
    let limits = crate::GlyphRunLimits::new(0, 0, 0, usize::MAX);
    let first = font
        .prepare_glyph_run(&instance, &[], limits, &mut preparation)
        .unwrap();
    let second = font
        .prepare_glyph_run(&instance, &[], limits, &mut preparation)
        .unwrap();
    let cpu = RendererCpu::new(
        RenderAccountingOptions::default().with_profile(RenderProfile::Summary { timing: false }),
    )
    .unwrap();
    let account = cpu.at(RenderCpuScope::PreparedValues);
    let one = account.import_prepared(&first, None).unwrap();
    let snapshot = cpu.snapshot().unwrap();
    let own = snapshot
        .scope_usage(RenderCpuScope::PreparedValues)
        .unwrap()
        .live(RenderResource::RetainedCpuBytes);
    let font_bytes = font.admitted_face().resource().retained_bytes() as u64;
    assert_eq!(
        snapshot
            .scope_usage(RenderCpuScope::FontPayloads)
            .unwrap()
            .live(RenderResource::RetainedCpuBytes),
        font_bytes
    );
    let duplicate = account.import_prepared(&first.clone(), None).unwrap();
    assert_eq!(
        cpu.snapshot()
            .unwrap()
            .scope_usage(RenderCpuScope::PreparedValues)
            .unwrap()
            .live(RenderResource::RetainedCpuBytes),
        own
    );
    let another = account.import_prepared(&second, None).unwrap();
    let snapshot = cpu.snapshot().unwrap();
    assert_eq!(
        snapshot
            .scope_usage(RenderCpuScope::PreparedValues)
            .unwrap()
            .live(RenderResource::RetainedCpuBytes),
        2 * own
    );
    assert_eq!(
        snapshot
            .scope_usage(RenderCpuScope::FontPayloads)
            .unwrap()
            .live(RenderResource::RetainedCpuBytes),
        font_bytes
    );
    drop(one);
    drop(duplicate);
    assert_eq!(
        cpu.snapshot()
            .unwrap()
            .scope_usage(RenderCpuScope::FontPayloads)
            .unwrap()
            .live(RenderResource::RetainedCpuBytes),
        font_bytes
    );
    drop(another);
    assert_eq!(
        cpu.snapshot()
            .unwrap()
            .scope_usage(RenderCpuScope::FontPayloads)
            .unwrap()
            .live(RenderResource::RetainedCpuBytes),
        0
    );
}

#[test]
fn maintenance_fault_preserves_first_diagnostic_and_does_not_block_release_or_cleanup() {
    let cpu = RendererCpu::new(RenderAccountingOptions::default()).unwrap();
    let account = cpu.at(RenderCpuScope::FrameBacking);
    let hold = account.acquire(RenderResource::ScratchCpuBytes, 5).unwrap();
    let cleanup = cpu.at(RenderCpuScope::Cleanup);
    cleanup.maintenance(RenderWork::ResourceProbes, u64::MAX);
    cleanup.maintenance(RenderWork::ResourceProbes, 1);
    let first = cpu.fault().unwrap();
    assert!(matches!(
        first,
        RenderAccountingError::CpuRefused {
            reason: RenderRefusalReason::Overflow,
            ..
        }
    ));
    let mut ran = false;
    cleanup.maintenance_phase(|| {
        ran = true;
        drop(hold);
    });
    assert!(ran);
    cleanup.note_fault(RenderAccountingError::Invariant);
    let snapshot = cpu.snapshot().unwrap();
    assert_eq!(snapshot.fault(), Some(first));
    assert_eq!(snapshot.usage().live(RenderResource::ScratchCpuBytes), 0);
    assert_eq!(snapshot.usage().work(RenderWork::ResourceProbes), u64::MAX);
    assert!(cpu.ensure_healthy().is_err());
}

static CLOCK_LOCK: Mutex<()> = Mutex::new(());
static CLOCK_READS: AtomicU64 = AtomicU64::new(0);
fn clock() -> Duration {
    Duration::from_nanos(CLOCK_READS.fetch_add(10, Ordering::Relaxed))
}

#[test]
fn off_and_untimed_profile_read_no_clock_and_timed_zero_detail_keeps_elapsed() {
    let _serial = CLOCK_LOCK.lock().unwrap();
    for profile in [
        RenderProfile::Off,
        RenderProfile::Summary { timing: false },
        RenderProfile::Detailed {
            timing: false,
            event_limit: 0,
            filter: super::super::RenderPhaseFilter::All,
        },
    ] {
        CLOCK_READS.store(0, Ordering::Relaxed);
        let cpu = RendererCpu::new(
            RenderAccountingOptions::default()
                .with_profile(profile)
                .with_clock(RenderClock::Host(clock)),
        )
        .unwrap();
        let checkpoint = cpu.begin_interval().unwrap();
        let hold = cpu
            .at(RenderCpuScope::FrameBacking)
            .acquire(RenderResource::ScratchCpuBytes, 3)
            .unwrap();
        drop(hold);
        let interval = checkpoint.finish().unwrap();
        assert_eq!(interval.elapsed(), None);
        let snapshot = cpu.snapshot().unwrap();
        assert_eq!(snapshot.usage().live(RenderResource::ScratchCpuBytes), 0);
        drop(cpu);
        assert_eq!(CLOCK_READS.load(Ordering::Relaxed), 0);
    }
    CLOCK_READS.store(0, Ordering::Relaxed);
    let cpu = RendererCpu::new(
        RenderAccountingOptions::default()
            .with_profile(RenderProfile::Detailed {
                timing: true,
                event_limit: 0,
                filter: super::super::RenderPhaseFilter::All,
            })
            .with_clock(RenderClock::Host(clock)),
    )
    .unwrap();
    let account = cpu.at(RenderCpuScope::FrameBacking);
    let checkpoint = cpu.begin_interval().unwrap();
    let hold = account
        .phase(|| account.acquire(RenderResource::ScratchCpuBytes, 3))
        .unwrap();
    let interval = checkpoint.finish().unwrap();
    assert!(interval.elapsed().unwrap() > Duration::ZERO);
    assert!(
        interval
            .scope_elapsed(RenderCpuScope::FrameBacking)
            .unwrap()
            > Duration::ZERO
    );
    assert_eq!(interval.events().count(), 0);
    assert!(interval.dropped_events() > 0);
    let frozen = interval.clone();
    let before = CLOCK_READS.load(Ordering::Relaxed);
    drop(hold);
    let snapshot = cpu.snapshot().unwrap();
    assert!(CLOCK_READS.load(Ordering::Relaxed) > before);
    assert_eq!(snapshot.usage().live(RenderResource::ScratchCpuBytes), 0);
    assert_eq!(interval, frozen);
}

#[test]
fn dropped_checkpoint_closes_once_and_new_intervals_keep_opening_custody() {
    let cpu = RendererCpu::new(RenderAccountingOptions::default()).unwrap();
    drop(cpu.begin_interval().unwrap());
    let checkpoint = cpu.begin_interval().unwrap();
    let hold = cpu
        .at(RenderCpuScope::FrameBacking)
        .acquire(RenderResource::ScratchCpuBytes, 7)
        .unwrap();
    let interval = checkpoint.finish().unwrap();
    assert_eq!(
        interval
            .usage()
            .opening_live(RenderResource::ScratchCpuBytes),
        0
    );
    assert_eq!(interval.usage().live(RenderResource::ScratchCpuBytes), 7);
    let next = cpu.begin_interval().unwrap();
    drop(hold);
    let closed = next.finish().unwrap();
    assert_eq!(
        closed.usage().opening_live(RenderResource::ScratchCpuBytes),
        7
    );
    assert_eq!(closed.usage().live(RenderResource::ScratchCpuBytes), 0);
    assert_eq!(closed.usage().peak(RenderResource::ScratchCpuBytes), 7);
}

#[test]
fn failed_checkpoint_extraction_is_a_fault_and_never_a_zero_interval() {
    let cpu = RendererCpu::new(RenderAccountingOptions::default()).unwrap();
    let checkpoint = cpu.begin_interval().unwrap();
    // Private fault injection: the custodian still owns its token but the
    // underlying interval was already closed. Completion must preserve Stale as
    // an invariant fault instead of inventing another complete interval.
    let actual = cpu
        .0
        .core
        .ledger
        .end_checkpoint(checkpoint.checkpoint.as_ref().unwrap())
        .unwrap();
    assert_eq!(
        actual.scopes[0]
            .usage
            .live(RenderResource::RetainedCpuBytes.dimension()),
        baseline()
    );
    assert!(checkpoint.finish().is_err());
    assert_eq!(
        cpu.snapshot().unwrap().fault(),
        Some(RenderAccountingError::Invariant)
    );
    assert!(cpu.begin_interval().is_err());
}

#[test]
fn observation_modes_preserve_identical_scripted_cpu_work_and_custody() {
    let mut baseline = None;
    for profile in [
        RenderProfile::Off,
        RenderProfile::Summary { timing: false },
        RenderProfile::Detailed {
            timing: false,
            event_limit: 0,
            filter: super::super::RenderPhaseFilter::All,
        },
        RenderProfile::Detailed {
            timing: false,
            event_limit: 4,
            filter: super::super::RenderPhaseFilter::RendererCpuScope(RenderCpuScope::FrameBacking),
        },
    ] {
        let cpu =
            RendererCpu::new(RenderAccountingOptions::default().with_profile(profile)).unwrap();
        let opening = retained(&cpu);
        let checkpoint = cpu.begin_interval().unwrap();
        let account = cpu.at(RenderCpuScope::FrameBacking);
        account.maintenance(RenderWork::ResourceProbes, 3);
        let scratch = account.acquire(RenderResource::ScratchCpuBytes, 3).unwrap();
        let retained = account
            .acquire(RenderResource::RetainedCpuBytes, 5)
            .unwrap();
        drop(retained);
        drop(scratch);
        let interval = checkpoint.finish().unwrap();
        let usage = interval.usage();
        assert_eq!(usage.work(RenderWork::ResourceProbes), 3);
        assert_eq!(
            usage.opening_live(RenderResource::RetainedCpuBytes),
            opening
        );
        assert_eq!(usage.live(RenderResource::RetainedCpuBytes), opening);
        assert_eq!(usage.peak(RenderResource::RetainedCpuBytes), opening + 5);
        assert_eq!(usage.live(RenderResource::ScratchCpuBytes), 0);
        assert_eq!(usage.peak(RenderResource::ScratchCpuBytes), 3);
        let frame = interval.scope_usage(RenderCpuScope::FrameBacking);
        assert_eq!(frame.peak(RenderResource::RetainedCpuBytes), 5);
        assert_eq!(frame.peak(RenderResource::ScratchCpuBytes), 3);
        assert_eq!(frame.live(RenderResource::RetainedCpuBytes), 0);
        assert_eq!(frame.live(RenderResource::ScratchCpuBytes), 0);
        assert_eq!(
            interval
                .scope_usage(RenderCpuScope::Cleanup)
                .work(RenderWork::ResourceProbes),
            3
        );
        if let Some(expected) = baseline {
            assert_eq!(usage, expected);
        } else {
            baseline = Some(usage);
        }
        if let RenderProfile::Detailed { event_limit, .. } = profile {
            assert!(interval.events().count() <= event_limit);
            if event_limit == 0 {
                assert!(interval.dropped_events() > 0);
            } else {
                assert_eq!(interval.events().count(), 4);
                for event in interval.events() {
                    let scope = match event {
                        super::super::RenderCpuEvent::Change { scope, .. }
                        | super::super::RenderCpuEvent::Phase { scope, .. }
                        | super::super::RenderCpuEvent::Refused { scope, .. } => scope,
                    };
                    assert_eq!(scope, RenderCpuScope::FrameBacking);
                }
            }
        }
    }
}
