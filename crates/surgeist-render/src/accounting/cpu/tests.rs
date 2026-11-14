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

#[test]
fn final_readback_transfer_consumes_actual_backing_without_a_later_drop_observation() {
    let _serial = CLOCK_LOCK.lock().unwrap();
    for requested in [0, 24] {
        let cpu = RendererCpu::new(
            RenderAccountingOptions::new()
                .with_clock(super::super::RenderClock::Host(clock))
                .with_profile(RenderProfile::Detailed {
                    timing: true,
                    event_limit: 64,
                    filter: super::super::RenderPhaseFilter::All,
                }),
        )
        .unwrap();
        let account = StorageAccount::Renderer(cpu.at(RenderCpuScope::ReadbackOutput));
        let checkpoint = cpu.begin_interval().unwrap();
        let mut output =
            AccountedVec::<u8>::new_in(&account, requested, RenderResource::RetainedCpuBytes)
                .unwrap();
        output.extend_from_slice(&vec![7; requested]).unwrap();
        let (values, mut hold) = output.into_parts().unwrap();
        let actual = values.capacity() as u64;
        let closing = checkpoint.finish().unwrap();
        assert_eq!(
            closing
                .scope_usage(RenderCpuScope::ReadbackOutput)
                .live(RenderResource::RetainedCpuBytes),
            actual
        );
        assert_eq!(
            closing.usage().work(RenderWork::ReadbackTransferredBytes),
            0
        );
        hold.transfer_readback().unwrap();
        let after_transfer = CLOCK_READS.load(Ordering::Relaxed);
        drop(hold);
        assert_eq!(
            CLOCK_READS.load(Ordering::Relaxed),
            after_transfer,
            "consumed custody must not emit even a zero-sized release after handoff"
        );
        assert_eq!(values, vec![7; requested]);
        let current = cpu.snapshot().unwrap();
        assert_eq!(
            current
                .scope_usage(RenderCpuScope::ReadbackOutput)
                .unwrap()
                .live(RenderResource::RetainedCpuBytes),
            0
        );
        assert_eq!(
            current.usage().work(RenderWork::ReadbackTransferredBytes),
            actual
        );
    }
}

#[test]
fn final_readback_transfer_overflow_keeps_custody_until_mandatory_disposal() {
    let cpu = RendererCpu::new(RenderAccountingOptions::default()).unwrap();
    let opening = retained(&cpu);
    let account = cpu.at(RenderCpuScope::ReadbackOutput);
    let mut hold = account
        .acquire(RenderResource::RetainedCpuBytes, 24)
        .unwrap();
    cpu.0
        .core
        .ledger
        .apply(
            cpu.0.core.scopes[RenderCpuScope::ReadbackOutput.index()],
            Change::Work(RenderWork::ReadbackTransferredBytes, u64::MAX),
        )
        .unwrap();
    let error = hold.transfer_readback().unwrap_err();
    let diagnostic = error.accounting_diagnostic().unwrap();
    assert!(matches!(
        diagnostic,
        RenderAccountingError::CpuRefused {
            scope: RenderCpuScope::ReadbackOutput,
            quantity: RenderQuantity::Work(RenderWork::ReadbackTransferredBytes),
            current: u64::MAX,
            amount: 24,
            reason: RenderRefusalReason::Overflow,
        }
    ));
    assert_eq!(hold.active_amount(), Some(24));
    let refused = cpu.snapshot().unwrap();
    assert_eq!(
        refused.usage().live(RenderResource::RetainedCpuBytes),
        opening + 24
    );
    assert_eq!(
        refused.usage().work(RenderWork::ReadbackTransferredBytes),
        u64::MAX
    );
    drop(hold);
    let cleaned = cpu.snapshot().unwrap();
    assert_eq!(
        cleaned.usage().live(RenderResource::RetainedCpuBytes),
        opening
    );
    assert_eq!(cleaned.fault(), Some(diagnostic));
    assert_eq!(
        cleaned.usage().work(RenderWork::ReadbackTransferredBytes),
        u64::MAX
    );
}

#[test]
fn final_readback_transfer_rechecks_health_and_rejects_other_backing_roles() {
    let cpu = RendererCpu::new(RenderAccountingOptions::default()).unwrap();
    let opening = retained(&cpu);
    let mut unrelated = cpu
        .at(RenderCpuScope::FrameBacking)
        .acquire(RenderResource::RetainedCpuBytes, 4)
        .unwrap();
    assert_eq!(
        unrelated
            .transfer_readback()
            .unwrap_err()
            .accounting_diagnostic(),
        Some(RenderAccountingError::Invariant)
    );
    drop(unrelated);
    let account = cpu.at(RenderCpuScope::ReadbackOutput);
    let mut hold = account
        .acquire(RenderResource::RetainedCpuBytes, 24)
        .unwrap();
    account.note_fault(RenderAccountingError::Invariant);
    assert_eq!(
        hold.transfer_readback()
            .unwrap_err()
            .accounting_diagnostic(),
        Some(RenderAccountingError::Invariant)
    );
    assert_eq!(hold.active_amount(), Some(24));
    drop(hold);
    let cleaned = cpu.snapshot().unwrap();
    assert_eq!(
        cleaned.usage().live(RenderResource::RetainedCpuBytes),
        opening
    );
    assert_eq!(
        cleaned.usage().work(RenderWork::ReadbackTransferredBytes),
        0
    );
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

fn warm_image_registry(cpu: &RendererCpu) {
    // Keep metadata capacity out of the payload oracle. Eight simultaneous
    // sources leave spare registry cells for the later one- and two-image cases.
    let images: [Image; 8] = std::array::from_fn(|i| image([i as u8, 0, 0, 255]));
    let account = cpu.at(RenderCpuScope::ImagePayloads);
    let imports = images
        .each_ref()
        .map(|image| account.import_image(image, None).unwrap());
    drop(imports);
}

#[test]
fn image_adapter_clones_and_blob_leases_share_one_typed_payload() {
    let cpu = RendererCpu::new(RenderAccountingOptions::default()).unwrap();
    warm_image_registry(&cpu);
    let background = retained(&cpu);
    let checkpoint = cpu.begin_interval().unwrap();
    let account = cpu.at(RenderCpuScope::ImagePayloads);
    let source = image([1, 2, 3, 4]);
    let source_clone = source.clone();
    let data_clone = source.data.clone();
    let original = account.import_image(&source, None).unwrap();
    let shared = account.import_image(&source_clone, None).unwrap();
    let token_clone = original.try_clone_with(None).unwrap();
    let blob = account.import_image_data(&data_clone, None).unwrap();
    // from_rgba owns four pixel bytes and one separately allocated Arc<[u8]>
    // adapter payload. Image/Blob/token clones share those same allocations.
    let payload = 4 + size_of::<Arc<[u8]>>() as u64;
    assert_eq!(retained(&cpu), background + payload);
    drop(source);
    drop(source_clone);
    drop(original);
    drop(shared);
    drop(token_clone);
    assert_eq!(retained(&cpu), background + payload);
    assert_eq!(data_clone.data.data(), &[1, 2, 3, 4]);
    drop(blob);
    assert_eq!(retained(&cpu), background);
    // The caller's remaining Blob does not retain Render attribution.
    assert_eq!(data_clone.data.data(), &[1, 2, 3, 4]);
    let interval = checkpoint.finish().unwrap();
    assert_eq!(
        interval.usage().live(RenderResource::RetainedCpuBytes),
        background
    );
    assert_eq!(
        interval.usage().peak(RenderResource::RetainedCpuBytes),
        background + payload
    );
    assert_eq!(cpu.fault(), None);
}

#[test]
fn image_adapter_independent_values_share_pixels_but_not_typed_payloads() {
    let cpu = RendererCpu::new(RenderAccountingOptions::default()).unwrap();
    warm_image_registry(&cpu);
    let background = retained(&cpu);
    let checkpoint = cpu.begin_interval().unwrap();
    let account = cpu.at(RenderCpuScope::ImagePayloads);
    let pixels = Arc::<[u8]>::from([1, 2, 3, 4]);
    let first = Image::from_rgba(crate::Size::new(1.0, 1.0), pixels.clone()).unwrap();
    let second = Image::from_rgba(crate::Size::new(1.0, 1.0), pixels.clone()).unwrap();
    assert!(Arc::ptr_eq(&first.bytes, &second.bytes));
    assert_ne!(first.data.data.id(), second.data.data.id());
    let first_import = account.import_image(&first, None).unwrap();
    let second_import = account.import_image(&second, None).unwrap();
    let adapter = size_of::<Arc<[u8]>>() as u64;
    assert_eq!(retained(&cpu), background + 4 + 2 * adapter);
    drop(first_import);
    assert_eq!(retained(&cpu), background + 4 + adapter);
    drop(second_import);
    assert_eq!(retained(&cpu), background);
    let interval = checkpoint.finish().unwrap();
    assert_eq!(
        interval.usage().live(RenderResource::RetainedCpuBytes),
        background
    );
    assert_eq!(
        interval.usage().peak(RenderResource::RetainedCpuBytes),
        background + 4 + 2 * adapter
    );
    assert_eq!(cpu.fault(), None);
}

#[test]
fn image_adapter_empty_sources_keep_known_blob_provenance_through_account_bridges() {
    let forward = Account::for_test();
    let account = forward.cpu().unwrap();
    let cpu = RendererCpu(Arc::clone(account.owner().unwrap()));
    warm_image_registry(&cpu);
    let background = retained(&cpu);
    let checkpoint = cpu.begin_interval().unwrap();
    let source = Image::from_rgba(crate::Size::new(0.0, 0.0), Arc::<[u8]>::from([])).unwrap();
    let data = source.data.clone();
    let original = account.import_image(&source, None).unwrap();
    let adapter = size_of::<Arc<[u8]>>() as u64;
    assert_eq!(retained(&cpu), background + adapter);
    let direct_blob = account.import_image_data(&data, None).unwrap();
    let frame_image = forward
        .import_image(&source)
        .unwrap()
        .expect("known Image adapter");
    let frame_blob = forward
        .import_image_data(&data)
        .unwrap()
        .expect("known empty Blob adapter");
    assert_eq!(retained(&cpu), background + adapter);
    drop(source);
    drop(original);
    drop(direct_blob);
    drop(frame_image);
    assert_eq!(retained(&cpu), background + adapter);
    drop(frame_blob);
    assert_eq!(retained(&cpu), background);
    let interval = checkpoint.finish().unwrap();
    assert_eq!(
        interval.usage().live(RenderResource::RetainedCpuBytes),
        background
    );
    assert_eq!(
        interval.usage().peak(RenderResource::RetainedCpuBytes),
        background + adapter
    );
    assert_eq!(cpu.fault(), None);

    // Equal dangling empty-slice addresses do not establish Blob provenance.
    let unknown = Image::from_rgba(crate::Size::new(0.0, 0.0), Arc::<[u8]>::from([])).unwrap();
    assert_ne!(unknown.data.data.id(), data.data.id());
    assert!(forward.import_image_data(&unknown.data).is_err());
    assert_eq!(cpu.fault(), Some(RenderAccountingError::Invariant));
}

#[test]
fn image_adapter_retained_limit_accepts_exact_payload_and_refuses_one_byte_under() {
    const LIMIT: u64 = 64 * 1024;
    let payload = 4 + size_of::<Arc<[u8]>>() as u64;
    for available in [payload, payload - 1] {
        let cpu = RendererCpu::new(
            RenderAccountingOptions::default()
                .with_limits(RenderLimits::new().with_retained_bytes(LIMIT)),
        )
        .unwrap();
        warm_image_registry(&cpu);
        let metadata = retained(&cpu);
        let ballast_bytes = usize::try_from(LIMIT - metadata - available).unwrap();
        let ballast = AccountedVec::<u8>::new_in(
            &StorageAccount::Renderer(cpu.at(RenderCpuScope::FrameBacking)),
            ballast_bytes,
            RenderResource::RetainedCpuBytes,
        )
        .unwrap();
        assert_eq!(ballast.capacity(), ballast_bytes);
        let opening = LIMIT - available;
        assert_eq!(retained(&cpu), opening);
        let checkpoint = cpu.begin_interval().unwrap();
        let result = cpu
            .at(RenderCpuScope::ImagePayloads)
            .import_image(&image([1, 2, 3, 4]), None);
        if available == payload {
            let import = result.expect("the complete pixel and adapter payload fits exactly");
            assert_eq!(retained(&cpu), LIMIT);
            drop(import);
        } else {
            let error = result.expect_err("one missing adapter byte must refuse the import");
            assert!(matches!(
                error.accounting_diagnostic(),
                Some(RenderAccountingError::CpuRefused {
                    scope: RenderCpuScope::Root,
                    quantity: RenderQuantity::Resource(RenderResource::RetainedCpuBytes),
                    reason: RenderRefusalReason::Limit { limit: LIMIT },
                    ..
                })
            ));
        }
        assert_eq!(retained(&cpu), opening);
        let interval = checkpoint.finish().unwrap();
        assert_eq!(
            interval
                .usage()
                .opening_live(RenderResource::RetainedCpuBytes),
            opening
        );
        assert_eq!(
            interval.usage().live(RenderResource::RetainedCpuBytes),
            opening
        );
        if available == payload {
            assert_eq!(
                interval.usage().peak(RenderResource::RetainedCpuBytes),
                LIMIT
            );
        }
        assert_eq!(cpu.fault(), None);
        drop(ballast);
        assert_eq!(retained(&cpu), metadata);
    }
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
    let adapter = size_of::<Arc<[u8]>>() as u64;
    let lease = account.import_image(&first, None).unwrap();
    assert_eq!(retained(&cpu), baseline() + 2 * entry_bytes() + 4 + adapter);
    let duplicate = account.import_image(&first, None).unwrap();
    let cloned = lease.try_clone_with(None).unwrap();
    assert_eq!(retained(&cpu), baseline() + 2 * entry_bytes() + 4 + adapter);
    let separate = account.import_image(&equal, None).unwrap();
    assert_eq!(
        retained(&cpu),
        baseline() + 4 * entry_bytes() + 8 + 2 * adapter
    );
    drop(first);
    drop(lease);
    drop(duplicate);
    assert_eq!(
        retained(&cpu),
        baseline() + 4 * entry_bytes() + 8 + 2 * adapter
    );
    drop(cloned);
    assert_eq!(retained(&cpu), baseline() + 4 * entry_bytes() + 4 + adapter);
    drop(separate);
    // Registry capacity remains real after semantic entries are removed.
    assert_eq!(retained(&cpu), baseline() + 4 * entry_bytes());
    assert_eq!(cpu.snapshot().unwrap().fault(), None);
}

#[test]
fn producer_blob_bridge_shares_original_arc_and_refuses_unknown_source() {
    let cpu = RendererCpu::new(RenderAccountingOptions::default()).unwrap();
    let account = cpu.at(RenderCpuScope::ImagePayloads);
    let source = image([5, 6, 7, 8]);
    let adapter = size_of::<Arc<[u8]>>() as u64;
    let original = account.import_image(&source, None).unwrap();
    let blob = account.import_image_data(&source.data, None).unwrap();
    assert_eq!(retained(&cpu), baseline() + 2 * entry_bytes() + 4 + adapter);
    drop(original);
    assert_eq!(retained(&cpu), baseline() + 2 * entry_bytes() + 4 + adapter);
    // A different adapter exposing the same pixel slice is still unregistered.
    let unknown = Image::from_rgba(source.size(), Arc::clone(&source.bytes)).unwrap();
    assert!(Arc::ptr_eq(&source.bytes, &unknown.bytes));
    assert_ne!(source.data.data.id(), unknown.data.data.id());
    assert!(account.import_image_data(&unknown.data, None).is_err());
    drop(blob);
    assert_eq!(retained(&cpu), baseline() + 2 * entry_bytes());
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
            .live(RenderResource::RetainedCpuBytes),
        baseline() + 2 * entry_bytes() + 4 + size_of::<Arc<[u8]>>() as u64
    );
    drop(import);
    assert_eq!(
        ledger
            .usage(root)
            .unwrap()
            .live(RenderResource::RetainedCpuBytes),
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

#[test]
fn mandatory_phase_returns_actual_values_and_closes_once_on_return_error_and_unwind() {
    use crate::accounting::{RenderCpuEvent, RenderPhaseFilter};
    use std::sync::atomic::AtomicUsize;

    static TIME: AtomicU64 = AtomicU64::new(0);
    static READS: AtomicUsize = AtomicUsize::new(0);
    fn now() -> Duration {
        READS.fetch_add(1, Ordering::SeqCst);
        Duration::from_millis(TIME.load(Ordering::SeqCst))
    }
    for profile in [
        RenderProfile::Off,
        RenderProfile::Summary { timing: false },
        RenderProfile::Summary { timing: true },
        RenderProfile::Detailed {
            timing: false,
            event_limit: 128,
            filter: RenderPhaseFilter::All,
        },
        RenderProfile::Detailed {
            timing: true,
            event_limit: 128,
            filter: RenderPhaseFilter::All,
        },
    ] {
        TIME.store(0, Ordering::SeqCst);
        READS.store(0, Ordering::SeqCst);
        let cpu = RendererCpu::new(
            RenderAccountingOptions::new()
                .with_profile(profile)
                .with_clock(RenderClock::Host(now)),
        )
        .unwrap();
        let account = cpu.at(RenderCpuScope::Cleanup);
        let calls = AtomicUsize::new(0);
        let value = String::from("the original moved result");
        let value_address = value.as_ptr();
        let returned = account.maintenance_phase(|| {
            calls.fetch_add(1, Ordering::SeqCst);
            TIME.fetch_add(7, Ordering::SeqCst);
            value
        });
        assert_eq!(returned.as_ptr(), value_address);
        assert_eq!(returned, "the original moved result");
        let error = String::from("the original callback error");
        let error_address = error.as_ptr();
        let returned: std::result::Result<String, String> = account.maintenance_phase(|| {
            calls.fetch_add(1, Ordering::SeqCst);
            TIME.fetch_add(7, Ordering::SeqCst);
            Err(error)
        });
        let returned_error = returned.unwrap_err();
        assert_eq!(returned_error.as_ptr(), error_address);
        assert_eq!(returned_error, "the original callback error");
        let hold = account.acquire(RenderResource::ScratchCpuBytes, 3).unwrap();
        assert!(
            std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
                account.maintenance_phase(|| {
                    calls.fetch_add(1, Ordering::SeqCst);
                    drop(hold);
                    TIME.fetch_add(7, Ordering::SeqCst);
                    panic!("mandatory callback fixture unwind");
                });
            }))
            .is_err()
        );
        assert_eq!(calls.load(Ordering::SeqCst), 3);
        let snapshot = cpu.snapshot().unwrap();
        assert_eq!(snapshot.usage().live(RenderResource::ScratchCpuBytes), 0);
        assert_eq!(snapshot.fault(), None);
        let timed = matches!(
            profile,
            RenderProfile::Summary { timing: true } | RenderProfile::Detailed { timing: true, .. }
        );
        assert_eq!(
            snapshot.scope_elapsed(RenderCpuScope::Cleanup),
            timed.then_some(Duration::from_millis(21))
        );
        if matches!(profile, RenderProfile::Detailed { .. }) {
            let phases: Vec<_> = snapshot
                .events()
                .filter_map(|event| match event {
                    RenderCpuEvent::Phase {
                        scope: RenderCpuScope::Cleanup,
                        duration,
                        ..
                    } => Some(duration),
                    _ => None,
                })
                .collect();
            assert_eq!(phases, vec![timed.then_some(Duration::from_millis(7)); 3]);
        }
        drop(account);
        drop(cpu);
        if !timed {
            assert_eq!(READS.load(Ordering::SeqCst), 0);
        }
    }
}

#[test]
fn foreign_phase_entry_still_runs_cleanup_and_preserves_the_first_fault() {
    let cpu = RendererCpu::new(RenderAccountingOptions::default()).unwrap();
    let foreign = RendererCpu::new(RenderAccountingOptions::default()).unwrap();
    for first in [
        None,
        Some(RenderAccountingError::CpuAllocation {
            scope: RenderCpuScope::FrameBacking,
            resource: RenderResource::RetainedCpuBytes,
        }),
    ] {
        // Private owning fault injection: public construction always pairs these
        // scopes with their own Ledger. Exercise the real foreign-scope refusal.
        let mut scopes = cpu.0.core.scopes;
        scopes[RenderCpuScope::Cleanup.index()] =
            foreign.0.core.scopes[RenderCpuScope::Cleanup.index()];
        let account = CpuAccount {
            core: Arc::new(Core {
                ledger: cpu.0.core.ledger.clone(),
                scopes,
                observer_budget: cpu.0.core.observer_budget,
                fault: Mutex::new(first),
            }),
            owner: None,
            scope: RenderCpuScope::Cleanup,
        };
        let hold = cpu
            .at(RenderCpuScope::FrameBacking)
            .acquire(RenderResource::ScratchCpuBytes, 5)
            .unwrap();
        let mut calls = 0;
        let returned = account.maintenance_phase(|| {
            calls += 1;
            drop(hold);
            String::from("cleaned despite entry refusal")
        });
        assert_eq!(returned, "cleaned despite entry refusal");
        assert_eq!(calls, 1);
        assert_eq!(
            *account.core.lock(),
            first.or(Some(RenderAccountingError::Invariant))
        );
        assert_eq!(
            cpu.snapshot()
                .unwrap()
                .usage()
                .live(RenderResource::ScratchCpuBytes),
            0
        );
    }
}

#[test]
fn foreign_phase_completion_records_a_render_association_fault_without_retry() {
    use crate::accounting::{RenderCpuEvent, RenderPhaseFilter};
    let _serial = CLOCK_LOCK.lock().unwrap();
    let options = || {
        RenderAccountingOptions::new()
            .with_profile(RenderProfile::Detailed {
                timing: true,
                event_limit: 64,
                filter: RenderPhaseFilter::All,
            })
            .with_clock(RenderClock::Host(clock))
    };
    for first in [
        None,
        Some(RenderAccountingError::CpuAllocation {
            scope: RenderCpuScope::FrameBacking,
            resource: RenderResource::RetainedCpuBytes,
        }),
    ] {
        let cpu = RendererCpu::new(options()).unwrap();
        let foreign = RendererCpu::new(options()).unwrap();
        let observation = foreign
            .0
            .core
            .ledger
            .begin_phase(foreign.0.core.scopes[RenderCpuScope::Cleanup.index()])
            .unwrap();
        if let Some(first) = first {
            cpu.0.core.remember(first);
        }
        // Only this owning private probe can violate the guard constructor's
        // core/marker pairing. It is not a publicly constructible failure.
        let phase = MaintenancePhase {
            core: cpu.0.core.clone(),
            observation: Some(observation),
        };
        let reads = CLOCK_READS.load(Ordering::Relaxed);
        drop(phase);
        assert_eq!(CLOCK_READS.load(Ordering::Relaxed), reads);
        assert_eq!(
            cpu.fault(),
            first.or(Some(RenderAccountingError::Invariant))
        );
        for owner in [&cpu, &foreign] {
            assert!(!owner.snapshot().unwrap().events().any(|event| matches!(
                event,
                RenderCpuEvent::Phase {
                    scope: RenderCpuScope::Cleanup,
                    ..
                }
            )));
        }
        assert_eq!(foreign.fault(), None);
    }
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
    let frozen = interval.snapshot_for_test();
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
            .live(RenderResource::RetainedCpuBytes),
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

#[test]
fn native_import_registry_growth_charges_existing_entries_once_to_actual_cpu_owner() {
    let cpu = RendererCpu::new(
        RenderAccountingOptions::default().with_profile(RenderProfile::Summary { timing: false }),
    )
    .unwrap();
    let account = cpu.at(RenderCpuScope::ImagePayloads);
    let image = Image::from_rgba(crate::Size::new(1.0, 1.0), vec![255, 0, 0, 255]).unwrap();
    let baseline = retained(&cpu);
    let import = account.import_image(&image, None).unwrap();
    let opening = cpu.snapshot().unwrap();
    let opening_moves = opening.usage().work(RenderWork::StorageRelocations);
    let backing_bytes;
    {
        let mut registry = cpu.0.lock();
        // One authentic image has its pixel identity and native Blob adapter.
        assert_eq!(registry.entries.len(), 2);
        let existing = registry.entries.len();
        assert_eq!(registry.entries.capacity(), existing);
        registry.reserve(&account).unwrap();
        let capacity = registry.entries.capacity();
        assert!(capacity > existing);
        backing_bytes = (capacity * size_of::<Entry>()) as u64;
        assert_eq!(registry.entries.len(), existing);
        let grown = cpu.snapshot().unwrap();
        assert_eq!(
            grown.usage().work(RenderWork::StorageRelocations) - opening_moves,
            2
        );
        assert_eq!(
            grown
                .scope_usage(RenderCpuScope::ImagePayloads)
                .unwrap()
                .work(RenderWork::StorageRelocations)
                - opening
                    .scope_usage(RenderCpuScope::ImagePayloads)
                    .unwrap()
                    .work(RenderWork::StorageRelocations),
            2
        );
        assert_eq!(
            grown
                .scope_usage(RenderCpuScope::FrameBacking)
                .unwrap()
                .work(RenderWork::StorageRelocations),
            0
        );
        registry.reserve(&account).unwrap(); // actual spare slot: no replacement
        assert_eq!(registry.entries.capacity(), capacity);
        assert_eq!(
            cpu.snapshot()
                .unwrap()
                .usage()
                .work(RenderWork::StorageRelocations),
            grown.usage().work(RenderWork::StorageRelocations)
        );
    }
    drop(import);
    assert!(cpu.0.lock().entries.is_empty());
    assert!(cpu.ensure_healthy().is_ok());
    assert_eq!(cpu.snapshot().unwrap().fault(), None);
    // Semantic imports are gone; the actual reusable registry backing remains.
    assert_eq!(retained(&cpu), baseline + backing_bytes);
}

#[test]
fn convolution_import_deduplicates_shared_backing_and_releases_last_lease() {
    let rect = crate::Rect::try_new(0.0, 0.0, 2.0, 2.0).unwrap();
    let domains = crate::ConvolutionDomains::try_new(rect, rect, rect).unwrap();
    let mut context = super::super::RenderAccountingContext::default();
    let first = crate::prepare_convolution(
        crate::ConvolutionInput::new(&[1.0]).with_order(1.0, None),
        domains,
        crate::FilterColorSpace::Srgb,
        &mut context,
    )
    .unwrap();
    let second = crate::prepare_convolution(
        crate::ConvolutionInput::new(&[1.0]).with_order(1.0, None),
        domains,
        crate::FilterColorSpace::Srgb,
        &mut context,
    )
    .unwrap();
    let bytes = first.own_backing_bytes().unwrap() as u64;
    // Per-scope snapshots are observable only with profiling enabled; Off retains
    // exact root custody while deliberately omitting scope report backing.
    let cpu = RendererCpu::new(
        RenderAccountingOptions::default().with_profile(RenderProfile::Summary { timing: false }),
    )
    .unwrap();
    let account = cpu.at(RenderCpuScope::PreparedValues);
    let live = || {
        cpu.snapshot()
            .unwrap()
            .scope_usage(RenderCpuScope::PreparedValues)
            .unwrap()
            .live(RenderResource::RetainedCpuBytes)
    };
    let one = account.import_convolution(&first, None).unwrap();
    assert_eq!(live(), bytes);
    let duplicate = account.import_convolution(&first.clone(), None).unwrap();
    assert_eq!(live(), bytes);
    let another = account.import_convolution(&second, None).unwrap();
    assert_eq!(live(), bytes * 2);
    drop(one);
    assert_eq!(live(), bytes * 2);
    drop(duplicate);
    assert_eq!(live(), bytes);
    drop(another);
    assert_eq!(live(), 0);
}
