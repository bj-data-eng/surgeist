use super::*;
use crate::accounting::{
    Account, AccountedVec, FrameFixture, Generation, RenderAccountingOptions, RenderClock,
    RenderCpuScope, RenderLimits, RenderProfile, RenderResource, RenderWork, RendererCpu,
};
use crate::gpu_transaction::{
    GpuOperationStage, GpuOperationTransaction,
    test_support::finish_vello_resources_without_submission_for_test,
};
use std::time::Duration;

#[derive(Clone, Copy, Debug)]
enum Exit {
    Abort,
    ActiveDrop,
    CleanDrop,
    PendingDrop,
    ReadyDrop,
    Unwind,
    Commit,
}

fn profiles() -> [RenderProfile; 3] {
    [
        RenderProfile::Off,
        RenderProfile::Summary { timing: false },
        RenderProfile::Summary { timing: true },
    ]
}

fn clock() -> Duration {
    Duration::ZERO
}

fn forbidden_clock() -> Duration {
    panic!("untimed lease collection must not read the clock")
}

fn aggregate(
    scope: &ActiveVelloEncodingScope<'_>,
    manager: &ResourceManager,
    account: &Account,
    count: usize,
) -> VelloResourceLeaseAggregate {
    // Spare capacity also exercises an allocated empty collection.
    let mut aggregate = VelloResourceLeaseAggregate::new(account, 3).unwrap();
    for _ in 0..count {
        let mut recording = RecordingBuilder::new(account).unwrap();
        recording.new_buffer(BufferRole::Scene, 4).unwrap();
        recording
            .request_image_atlas(PhysicalSize::new(2, 2))
            .unwrap();
        let (_recording, intents) = recording.finish();
        aggregate
            .push(VelloResourceLease::allocate(scope, manager, &intents, Some(account)).unwrap())
            .unwrap();
    }
    aggregate
}

fn identity(
    leases: &AccountedVec<VelloResourceLease>,
) -> (*const VelloResourceLease, usize, usize) {
    (leases.as_ptr(), leases.len(), leases.capacity())
}

fn active_leases(aggregate: &VelloResourceLeaseAggregate) -> &AccountedVec<VelloResourceLease> {
    aggregate.leases.backing_for_test()
}

fn clean_leases(
    aggregate: &ScopeResolvedVelloResourceLeaseAggregate,
) -> &AccountedVec<VelloResourceLease> {
    aggregate.leases.backing_for_test()
}

fn pending_leases(pending: &PendingVelloResourceCommit) -> &AccountedVec<VelloResourceLease> {
    pending.leases.backing_for_test()
}

fn frame_identities(leases: &[VelloResourceLease]) -> Vec<crate::resource::FrameIdentity> {
    leases
        .iter()
        .map(|lease| {
            lease
                .pending
                .frame_scope()
                .unwrap()
                .frame_identity_for_test()
        })
        .collect()
}

#[test]
fn real_lease_collections_preserve_backing_and_close_every_owning_phase() {
    let mut renderer = pollster::block_on(crate::Renderer::new(crate::Options::default())).unwrap();
    let device = renderer.default_wgpu_device_queue().unwrap().0.clone();
    let signal = renderer.default_device_signal_for_test().unwrap();
    for profile in profiles() {
        let options = RenderAccountingOptions::new()
            .with_profile(profile)
            .with_clock(RenderClock::Host(
                if matches!(profile, RenderProfile::Summary { timing: true }) {
                    clock
                } else {
                    forbidden_clock
                },
            ));
        for count in [0, 1, 3] {
            for exit in [
                Exit::Abort,
                Exit::ActiveDrop,
                Exit::CleanDrop,
                Exit::PendingDrop,
                Exit::ReadyDrop,
                Exit::Unwind,
                Exit::Commit,
            ] {
                let cpu = RendererCpu::new(options).unwrap();
                let manager = ResourceManager::with_generation(
                    crate::ResourceCacheBudget::DEFAULT,
                    Generation::new(options).unwrap(),
                    cpu.at(RenderCpuScope::DeviceBacking),
                );
                let mut fixture = FrameFixture::new(&cpu, options);
                fixture
                    .run_sync(|attempt| {
                        let account = attempt.account();
                        let transaction = GpuOperationTransaction::begin(
                            &device,
                            signal.clone(),
                            signal.next_test_generation().unwrap(),
                            GpuOperationStage::Render,
                        );
                        let scope = ActiveVelloEncodingScope::begin(&device);
                        let aggregate = aggregate(&scope, &manager, account, count);
                        let leases = active_leases(&aggregate);
                        let backing = identity(leases);
                        let frames = frame_identities(leases);
                        assert_eq!(backing.1, count);
                        assert!(backing.2 >= 3);
                        let charged =
                            (backing.2 * std::mem::size_of::<VelloResourceLease>()) as u64;
                        let before = cpu.snapshot().unwrap();
                        assert!(before.usage().live(RenderResource::ScratchCpuBytes) >= charged);
                        let resources = manager.observation_for_test();
                        assert_eq!(resources.active_frame_count, count);
                        assert_eq!(resources.leased_count, 2 * count);
                        assert_eq!(resources.entry_count, 2 * count);
                        assert_eq!(
                            account
                                .work_count(RenderWork::ResourceAllocationRequests)
                                .unwrap(),
                            // Buffer, texture and its separately owned view.
                            (3 * count) as u64
                        );
                        assert_eq!(
                            manager
                                .generation()
                                .usage()
                                .unwrap()
                                .work(RenderWork::ResourceAllocations),
                            (2 * count) as u64
                        );

                        match exit {
                            Exit::Abort => {
                                let summary = aggregate.abort();
                                assert_eq!(summary.discarded_resource_count, 2 * count);
                                assert_eq!(
                                    summary.atlas_outcome,
                                    if count == 0 {
                                        VelloAtlasOutcome::NoAtlas
                                    } else {
                                        VelloAtlasOutcome::Recreate
                                    }
                                );
                                pollster::block_on(scope.finish(account)).unwrap();
                            }
                            Exit::ActiveDrop => {
                                drop(aggregate);
                                pollster::block_on(scope.finish(account)).unwrap();
                            }
                            Exit::Unwind => {
                                assert!(
                                    std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
                                        let _aggregate = aggregate;
                                        panic!("external unwind with real active lease owners");
                                    }))
                                    .is_err()
                                );
                                pollster::block_on(scope.finish(account)).unwrap();
                            }
                            _ => {
                                let clean = pollster::block_on(
                                    scope.finish_with_leases(aggregate, account),
                                )
                                .unwrap_or_else(|failure| {
                                    panic!("valid checked scope failed: {}", failure.error())
                                });
                                let leases = clean_leases(&clean);
                                assert_eq!(identity(leases), backing);
                                assert_eq!(frame_identities(leases), frames);
                                if matches!(exit, Exit::CleanDrop) {
                                    drop(clean);
                                } else {
                                    let before_transfer = cpu.snapshot().unwrap();
                                    let pending = PendingVelloResourceCommit::from_aggregate(clean);
                                    let after_transfer = cpu.snapshot().unwrap();
                                    assert_eq!(identity(pending_leases(&pending)), backing);
                                    assert_eq!(frame_identities(pending_leases(&pending)), frames);
                                    assert_eq!(manager.observation_for_test(), resources);
                                    assert_eq!(after_transfer.usage(), before_transfer.usage());
                                    assert_eq!(
                                        after_transfer.scope_count(RenderCpuScope::FrameBacking),
                                        before_transfer.scope_count(RenderCpuScope::FrameBacking)
                                    );
                                    assert_eq!(
                                        after_transfer.scope_elapsed(RenderCpuScope::FrameBacking),
                                        before_transfer.scope_elapsed(RenderCpuScope::FrameBacking)
                                    );
                                    if matches!(exit, Exit::Commit) {
                                        pollster::block_on(
                                            finish_vello_resources_without_submission_for_test(
                                                transaction,
                                                pending,
                                                crate::RuntimeOperation::EffectRendering,
                                            ),
                                        )
                                        .unwrap();
                                        return Ok(());
                                    }
                                    if matches!(exit, Exit::ReadyDrop) {
                                        let ready = pending.into_accounting_ready().unwrap();
                                        assert_eq!(
                                            identity(pending_leases(&ready.pending)),
                                            backing
                                        );
                                        assert_eq!(
                                            frame_identities(pending_leases(&ready.pending)),
                                            frames
                                        );
                                        drop(ready);
                                    } else {
                                        drop(pending);
                                    }
                                }
                            }
                        }
                        drop(transaction);
                        Ok(())
                    })
                    .unwrap()
                    .into_parts()
                    .0
                    .unwrap();
                let after = manager.observation_for_test();
                assert_eq!(
                    after.active_frame_count, 0,
                    "{profile:?}, {count}, {exit:?}"
                );
                assert_eq!(after.leased_count, 0);
                assert_eq!(after.resolved_lease_count, 0);
                let retained = usize::from(matches!(exit, Exit::Commit) && count > 0);
                assert_eq!(after.entry_count, retained);
                assert_eq!(after.retained_bytes, (16 * retained) as u64);
                assert_eq!(after.accounted_entry_bytes, Some((16 * retained) as u64));
                assert_eq!(after.accounting_fault_for_test(), None);
                assert_eq!(
                    after.recovery_outcome_for_test(),
                    if count > 0 && !matches!(exit, Exit::Commit) {
                        Some(VelloAtlasOutcome::Recreate)
                    } else {
                        None
                    }
                );
                assert_eq!(
                    cpu.snapshot()
                        .unwrap()
                        .usage()
                        .live(RenderResource::ScratchCpuBytes),
                    0
                );
                assert_eq!(cpu.snapshot().unwrap().fault(), None);
            }
        }
    }
}

#[test]
fn pending_collection_readiness_refusal_keeps_first_fault_and_closes_all_frames() {
    let mut renderer = pollster::block_on(crate::Renderer::new(crate::Options::default())).unwrap();
    let device = renderer.default_wgpu_device_queue().unwrap().0;
    let account = Account::for_test();
    for ready_first in [false, true] {
        let manager = ResourceManager::default();
        let scope = ActiveVelloEncodingScope::begin(device);
        let aggregate = aggregate(&scope, &manager, &account, 3);
        let clean = pollster::block_on(scope.finish_with_leases(aggregate, &account))
            .unwrap_or_else(|failure| panic!("valid checked scope failed: {}", failure.error()));
        let pending = PendingVelloResourceCommit::from_aggregate(clean);
        let (fault, error) = if ready_first {
            let ready = pending.into_accounting_ready().unwrap();
            let fault = ready.pending.poison_retained_byte_accounting_for_test();
            let error = ready.ensure_commit_ready().unwrap_err();
            drop(ready);
            (fault, error)
        } else {
            let fault = pending.poison_retained_byte_accounting_for_test();
            let error = match pending.into_accounting_ready() {
                Ok(_) => panic!("supported resource fault must refuse readiness"),
                Err(error) => error,
            };
            (fault, error)
        };
        assert_eq!(error.code(), crate::ErrorCode::RenderFailed);
        assert_eq!(
            error.message(),
            "resource manager is unavailable after a retained-byte accounting invariant failure"
        );
        assert_eq!(error.accounting_diagnostic(), None);
        let after = manager.observation_for_test();
        assert_eq!(after.accounting_fault_for_test(), Some(fault));
        assert_eq!(after.active_frame_count, 0);
        assert_eq!(after.entry_count, 0);
        assert_eq!(after.leased_count, 0);
        assert_eq!(after.resolved_lease_count, 0);
        assert_eq!(
            after.recovery_outcome_for_test(),
            Some(VelloAtlasOutcome::Recreate)
        );
    }
}

#[test]
fn single_pending_backing_refusal_closes_its_real_empty_frame() {
    let mut renderer = pollster::block_on(crate::Renderer::new(crate::Options::default())).unwrap();
    let device = renderer.default_wgpu_device_queue().unwrap().0;
    let account = Account::for_test();
    let options =
        RenderAccountingOptions::new().with_limits(RenderLimits::new().with_scratch_bytes(1));
    let cpu = RendererCpu::new(options).unwrap();
    let manager = ResourceManager::with_generation(
        crate::ResourceCacheBudget::DEFAULT,
        Generation::new(options).unwrap(),
        cpu.at(RenderCpuScope::DeviceBacking),
    );
    let scope = ActiveVelloEncodingScope::begin(device);
    let lease = VelloResourceLease::allocate(&scope, &manager, &[], Some(&account)).unwrap();
    assert_eq!(manager.observation_for_test().active_frame_count, 1);
    let clean = pollster::block_on(scope.finish_with_lease(lease, &account))
        .unwrap_or_else(|failure| panic!("valid checked scope failed: {}", failure.error()));
    let error = match PendingVelloResourceCommit::new(clean) {
        Ok(_) => panic!("one lease backing cannot fit a one-byte budget"),
        Err(error) => error,
    };
    assert!(matches!(
        error.accounting_diagnostic(),
        Some(crate::RenderAccountingError::CpuRefused {
            quantity: crate::RenderQuantity::Resource(RenderResource::ScratchCpuBytes),
            reason: crate::RenderRefusalReason::Limit { limit: 1 },
            ..
        })
    ));
    assert_eq!(manager.observation_for_test().active_frame_count, 0);
    assert_eq!(manager.observation_for_test().entry_count, 0);
    assert_eq!(
        cpu.snapshot()
            .unwrap()
            .usage()
            .live(RenderResource::ScratchCpuBytes),
        0
    );
    assert_eq!(cpu.snapshot().unwrap().fault(), None);
}
