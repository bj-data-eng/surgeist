use super::*;
use crate::accounting::{
    Account, FrameFixture, Generation, RenderAccountingOptions, RenderClock, RenderCpuScope,
    RenderProfile, RenderResource, RenderWork, RendererCpu,
};
use crate::resource::{
    ResourceIdentity, ResourceManagerObservationForTest, ResourceRetentionOutcome,
};
use std::time::Duration;
use surgeist_ledger::Dimension;

#[derive(Clone, Copy, Debug)]
enum Input {
    Empty,
    Buffer,
    BufferAndAtlas,
}
impl Input {
    fn count(self) -> usize {
        match self {
            Self::Empty => 0,
            Self::Buffer => 1,
            Self::BufferAndAtlas => 2,
        }
    }
    fn bytes(self) -> u64 {
        match self {
            Self::Empty => 0,
            Self::Buffer => 4,
            Self::BufferAndAtlas => 20,
        }
    }
    fn atlas(self) -> bool {
        matches!(self, Self::BufferAndAtlas)
    }
}
fn profiles() -> [RenderProfile; 3] {
    [
        RenderProfile::Off,
        RenderProfile::Summary { timing: false },
        RenderProfile::Summary { timing: true },
    ]
}
fn policies() -> [CleanVelloResourceRetention; 2] {
    [
        CleanVelloResourceRetention::DirectAtlasOnly,
        CleanVelloResourceRetention::ReusableGraphFrame,
    ]
}
fn clock() -> Duration {
    Duration::ZERO
}
fn options(profile: RenderProfile) -> RenderAccountingOptions {
    RenderAccountingOptions::new()
        .with_profile(profile)
        .with_clock(RenderClock::Host(clock))
}
fn manager(cpu: &RendererCpu, options: RenderAccountingOptions) -> ResourceManager {
    ResourceManager::with_generation(
        crate::ResourceCacheBudget::DEFAULT,
        Generation::new(options).unwrap(),
        cpu.at(RenderCpuScope::DeviceBacking),
    )
}
fn clean_lease(
    device: &wgpu::Device,
    manager: &ResourceManager,
    account: &Account,
    input: Input,
    policy: CleanVelloResourceRetention,
) -> ScopeResolvedVelloResourceLease {
    let mut recording = RecordingBuilder::new(account).unwrap();
    if input.count() > 0 {
        recording.new_buffer(BufferRole::Scene, 4).unwrap();
    }
    if input.atlas() {
        recording
            .request_image_atlas(PhysicalSize::new(2, 2))
            .unwrap();
    }
    let (_recording, intents) = recording.finish();
    let scope = ActiveVelloEncodingScope::begin(device);
    let lease = VelloResourceLease::allocate_with_retention(
        &scope,
        manager,
        &intents,
        policy,
        Some(account),
    )
    .unwrap();
    pollster::block_on(scope.finish_with_lease(lease, account)).unwrap_or_else(|failure| {
        panic!("valid individual checked scope failed: {}", failure.error())
    })
}
fn cleanup_cost(
    profile: RenderProfile,
    cpu: &RendererCpu,
    before: &crate::RenderCpuSnapshot,
    phases: u64,
    probes: u64,
) {
    let after = cpu.snapshot().unwrap();
    assert_eq!(
        after.scope_count(RenderCpuScope::Cleanup) - before.scope_count(RenderCpuScope::Cleanup),
        if matches!(profile, RenderProfile::Off) {
            0
        } else {
            phases
        }
    );
    assert_eq!(
        after.scope_elapsed(RenderCpuScope::Cleanup),
        matches!(profile, RenderProfile::Summary { timing: true }).then_some(Duration::ZERO)
    );
    for &work in RenderWork::ALL {
        assert_eq!(
            after.usage().work(work) - before.usage().work(work),
            if work == RenderWork::ResourceProbes {
                probes
            } else {
                0
            },
            "CPU {work:?}"
        );
    }
    assert_eq!(after.fault(), None);
}
fn closed(
    manager: &ResourceManager,
    entries: usize,
    bytes: u64,
) -> ResourceManagerObservationForTest {
    let after = manager.observation_for_test();
    assert_eq!(after.active_frame_count, 0);
    assert_eq!(after.leased_count, 0);
    assert_eq!(after.resolved_lease_count, 0);
    assert_eq!(after.entry_count, entries);
    assert_eq!(after.accounted_entry_bytes, Some(bytes));
    after
}

#[test]
fn individual_clean_commit_preserves_retention_and_exact_readiness_maintenance() {
    let mut renderer = pollster::block_on(crate::Renderer::new(crate::Options::default())).unwrap();
    let device = renderer.default_wgpu_device_queue().unwrap().0;
    for input in [Input::Empty, Input::Buffer, Input::BufferAndAtlas] {
        for policy in policies() {
            let mut expected_work = None;
            for profile in profiles() {
                let options = options(profile);
                let cpu = RendererCpu::new(options).unwrap();
                let manager = manager(&cpu, options);
                let mut fixture = FrameFixture::new(&cpu, options);
                let finished = fixture
                    .run_sync(|attempt| {
                        let account = attempt.account();
                        let clean = clean_lease(device, &manager, account, input, policy);
                        assert_eq!(manager.observation_for_test().active_frame_count, 1);
                        let before = cpu.snapshot().unwrap();
                        let generation_before = manager.generation().snapshot().unwrap();
                        assert_eq!(
                            generation_before.usage().live(RenderResource::GpuBytes),
                            input.bytes()
                        );
                        assert_eq!(
                            generation_before.usage().peak(RenderResource::GpuBytes),
                            input.bytes()
                        );
                        assert_eq!(
                            generation_before
                                .usage()
                                .live(RenderResource::RegisteredCacheGpuBytes),
                            input.bytes()
                        );
                        assert_eq!(
                            generation_before
                                .usage()
                                .work(RenderWork::ResourceAllocations),
                            input.count() as u64
                        );
                        assert_eq!(
                            account
                                .work_count(RenderWork::ResourceAllocationRequests)
                                .unwrap(),
                            (input.count() + usize::from(input.atlas())) as u64
                        );
                        let forward_before = RenderWork::ALL
                            .iter()
                            .map(|&work| account.work_count(work).unwrap())
                            .collect::<Vec<_>>();
                        // This valid checked single lease enters the final individual
                        // readiness gate directly, without collection readiness.
                        let committed = clean.commit().unwrap();
                        let after_commit = cpu.snapshot().unwrap();
                        assert_eq!(
                            after_commit.usage().peak(RenderResource::ScratchCpuBytes),
                            before.usage().peak(RenderResource::ScratchCpuBytes)
                        );
                        assert!(
                            before.usage().live(RenderResource::ScratchCpuBytes)
                                >= (input.count()
                                    * (std::mem::size_of::<(u64, ResourceIdentity)>()
                                        + std::mem::size_of::<ResourceIdentity>()))
                                    as u64
                        );
                        assert!(
                            after_commit.usage().live(RenderResource::ScratchCpuBytes)
                                >= (input.count() * std::mem::size_of::<ResourceIdentity>()) as u64,
                            "returned cleanup retains its original reserved backing even when empty"
                        );
                        let reusable = policy == CleanVelloResourceRetention::ReusableGraphFrame;
                        let retained = if reusable {
                            input.count()
                        } else {
                            usize::from(input.atlas())
                        };
                        let bytes = if reusable {
                            input.bytes()
                        } else {
                            16 * u64::from(input.atlas())
                        };
                        assert_eq!(
                            committed.atlas_outcome(),
                            if input.atlas() {
                                VelloAtlasOutcome::Retain
                            } else {
                                VelloAtlasOutcome::NoAtlas
                            }
                        );
                        assert!(committed.frame_cleanup.evicted_resources().is_empty());
                        assert_eq!(
                            committed.frame_cleanup.retention(),
                            if retained == 0 {
                                ResourceRetentionOutcome::NoIdleResources
                            } else {
                                ResourceRetentionOutcome::RetainedReusable {
                                    resource_count: retained,
                                    byte_len: bytes,
                                }
                            }
                        );
                        let after = closed(&manager, retained, bytes);
                        assert_eq!(after.recovery_outcome_for_test(), None);
                        assert_eq!(after.accounting_fault_for_test(), None);
                        let generation_after = manager.generation().snapshot().unwrap();
                        for resource in [
                            RenderResource::GpuBytes,
                            RenderResource::RegisteredCacheGpuBytes,
                            RenderResource::IdleCacheGpuBytes,
                        ] {
                            assert_eq!(generation_after.usage().live(resource), bytes);
                        }
                        assert_eq!(
                            generation_after.usage().peak(RenderResource::GpuBytes),
                            input.bytes()
                        );
                        assert_eq!(
                            generation_after
                                .usage()
                                .work(RenderWork::ResourceLeaseReturns)
                                - generation_before
                                    .usage()
                                    .work(RenderWork::ResourceLeaseReturns),
                            retained as u64
                        );
                        assert_eq!(
                            generation_after.usage().work(RenderWork::ResourceEvictions)
                                - generation_before
                                    .usage()
                                    .work(RenderWork::ResourceEvictions),
                            (input.count() - retained) as u64
                        );
                        for work in [
                            RenderWork::ResourceAllocations,
                            RenderWork::ResourceAllocationRequests,
                            RenderWork::GpuObjectsCreated,
                            RenderWork::Submissions,
                        ] {
                            assert_eq!(
                                generation_after.usage().work(work),
                                generation_before.usage().work(work)
                            );
                        }
                        // For zero/one/two tokens, proof preparation costs 3*n,
                        // plus one sort and one duplicate comparison when n=2.
                        // Trim adds one push per retained entry and one sort at two.
                        let proof = |n: usize| 3 * n as u64 + 2 * u64::from(n == 2);
                        let probes = proof(input.count())
                            + proof(retained)
                            + retained as u64
                            + u64::from(retained == 2);
                        // Final readiness, finish readiness, frame resolution, index
                        // backing disposal; direct atlas retirement adds one phase.
                        let phases = 4 + u64::from(input.atlas() && !reusable);
                        cleanup_cost(profile, &cpu, &before, phases, probes);
                        let work = RenderWork::ALL
                            .iter()
                            .map(|&work| {
                                (
                                    after_commit.usage().work(work) - before.usage().work(work),
                                    generation_after.usage().work(work)
                                        - generation_before.usage().work(work),
                                )
                            })
                            .collect::<Vec<_>>();
                        if let Some(expected) = &expected_work {
                            assert_eq!(&work, expected);
                        } else {
                            expected_work = Some(work);
                        }
                        assert_eq!(
                            RenderWork::ALL
                                .iter()
                                .map(|&work| account.work_count(work).unwrap())
                                .collect::<Vec<_>>(),
                            forward_before
                        );
                        drop(committed);
                        cleanup_cost(profile, &cpu, &before, phases + 1, probes);
                        assert_eq!(
                            cpu.snapshot()
                                .unwrap()
                                .usage()
                                .live(RenderResource::ScratchCpuBytes),
                            0
                        );
                        Ok(())
                    })
                    .unwrap();
                let (result, started, report, cleanup_fault) = finished.into_parts();
                result.unwrap();
                assert!(started);
                assert_eq!(cleanup_fault, None);
                let interval = report.renderer_cpu().unwrap();
                assert_eq!(interval.usage().live(RenderResource::ScratchCpuBytes), 0);
                assert_eq!(
                    interval.usage().peak(RenderResource::ScratchCpuBytes),
                    cpu.snapshot()
                        .unwrap()
                        .usage()
                        .peak(RenderResource::ScratchCpuBytes)
                );
                let generation = manager.generation().clone();
                drop(manager);
                for resource in [
                    RenderResource::GpuBytes,
                    RenderResource::RegisteredCacheGpuBytes,
                    RenderResource::IdleCacheGpuBytes,
                    RenderResource::OpaqueGpuObjects,
                ] {
                    assert_eq!(generation.usage().unwrap().live(resource), 0);
                }
            }
        }
    }
}

#[test]
fn individual_final_readiness_fault_preserves_first_error_and_aborts_actual_frame() {
    const READINESS_ERROR: &str =
        "resource manager is unavailable after a retained-byte accounting invariant failure";
    let mut renderer = pollster::block_on(crate::Renderer::new(crate::Options::default())).unwrap();
    let device = renderer.default_wgpu_device_queue().unwrap().0;
    for input in [Input::Buffer, Input::BufferAndAtlas] {
        for policy in policies() {
            for profile in profiles() {
                let options = options(profile);
                let cpu = RendererCpu::new(options).unwrap();
                let manager = manager(&cpu, options);
                let mut fixture = FrameFixture::new(&cpu, options);
                fixture
                    .run_sync(|attempt| {
                        let clean = clean_lease(device, &manager, attempt.account(), input, policy);
                        let fault = clean
                            .lease
                            .pending
                            .frame_scope()
                            .unwrap()
                            .poison_retained_byte_accounting_for_test();
                        assert_eq!(
                            fault,
                            ResourceAccountingFault::RetainedByteMismatch {
                                retained_bytes: input.bytes() - 1,
                                registered_entry_bytes: input.bytes(),
                            }
                        );
                        let before = cpu.snapshot().unwrap();
                        let error = match clean.commit() {
                            Ok(_) => {
                                panic!("supported accounting fault must refuse final readiness")
                            }
                            Err(error) => error,
                        };
                        assert_eq!(error.code(), crate::ErrorCode::RenderFailed);
                        assert_eq!(error.message(), READINESS_ERROR);
                        assert_eq!(error.accounting_diagnostic(), None);
                        let after = closed(&manager, 0, 0);
                        assert_eq!(after.accounting_fault_for_test(), Some(fault));
                        assert_eq!(
                            after.recovery_outcome_for_test(),
                            input.atlas().then_some(VelloAtlasOutcome::Recreate)
                        );
                        // Buffer discard precedes atlas discard. The one-byte
                        // deficit refuses the final registration credit release.
                        let last_bytes = if input.atlas() { 16 } else { 4 };
                        assert_eq!(after.retained_bytes, last_bytes - 1);
                        let generation = manager.generation();
                        let diagnostic = generation
                            .ensure_healthy()
                            .unwrap_err()
                            .accounting_diagnostic();
                        let quantity = crate::RenderQuantity::Resource(
                            RenderResource::RegisteredCacheGpuBytes,
                        );
                        assert_eq!(
                            diagnostic,
                            Some(crate::RenderAccountingError::GenerationRefused {
                                quantity,
                                current: last_bytes - 1,
                                amount: last_bytes,
                                reason: crate::RenderRefusalReason::Underflow,
                            })
                        );
                        let usage = generation.usage().unwrap();
                        assert_eq!(usage.live(RenderResource::GpuBytes), 0);
                        assert_eq!(usage.live(RenderResource::OpaqueGpuObjects), 0);
                        assert_eq!(
                            usage.work(RenderWork::ResourceEvictions),
                            input.count() as u64
                        );
                        assert_eq!(usage.work(RenderWork::ResourceLeaseReturns), 0);
                        // Refused readiness, abort resolution and two scratch
                        // disposals. No proof work follows refusal.
                        cleanup_cost(profile, &cpu, &before, 4, 0);
                        assert_eq!(
                            cpu.snapshot()
                                .unwrap()
                                .usage()
                                .live(RenderResource::ScratchCpuBytes),
                            0
                        );
                        Ok(())
                    })
                    .unwrap()
                    .into_parts()
                    .0
                    .unwrap();
            }
        }
    }
}

#[test]
fn external_unwind_discards_checked_individual_owners_under_both_retention_policies() {
    let mut renderer = pollster::block_on(crate::Renderer::new(crate::Options::default())).unwrap();
    let device = renderer.default_wgpu_device_queue().unwrap().0;
    for input in [Input::Empty, Input::Buffer, Input::BufferAndAtlas] {
        for policy in policies() {
            for profile in profiles() {
                let options = options(profile);
                let cpu = RendererCpu::new(options).unwrap();
                let manager = manager(&cpu, options);
                let mut fixture = FrameFixture::new(&cpu, options);
                fixture
                    .run_sync(|attempt| {
                        let clean = clean_lease(device, &manager, attempt.account(), input, policy);
                        let before = cpu.snapshot().unwrap();
                        assert!(
                            std::panic::catch_unwind(std::panic::AssertUnwindSafe(move || {
                                let _clean = clean;
                                panic!("external unwind around a real checked individual lease");
                            }))
                            .is_err()
                        );
                        let after = closed(&manager, 0, 0);
                        assert_eq!(after.accounting_fault_for_test(), None);
                        assert_eq!(
                            after.recovery_outcome_for_test(),
                            input.atlas().then_some(VelloAtlasOutcome::Recreate)
                        );
                        for resource in [
                            RenderResource::GpuBytes,
                            RenderResource::RegisteredCacheGpuBytes,
                            RenderResource::IdleCacheGpuBytes,
                            RenderResource::OpaqueGpuObjects,
                        ] {
                            assert_eq!(manager.generation().usage().unwrap().live(resource), 0);
                        }
                        assert_eq!(
                            manager
                                .generation()
                                .usage()
                                .unwrap()
                                .work(RenderWork::ResourceEvictions),
                            input.count() as u64
                        );
                        assert_eq!(
                            manager
                                .generation()
                                .usage()
                                .unwrap()
                                .work(RenderWork::ResourceLeaseReturns),
                            0
                        );
                        cleanup_cost(profile, &cpu, &before, 3, 0);
                        assert_eq!(
                            cpu.snapshot()
                                .unwrap()
                                .usage()
                                .live(RenderResource::ScratchCpuBytes),
                            0
                        );
                        Ok(())
                    })
                    .unwrap()
                    .into_parts()
                    .0
                    .unwrap();
            }
        }
    }
}
