use super::*;
use crate::{
    RenderAccountingContext, RenderAccountingOptions, RenderAttemptKind, RenderAttemptStatus,
    RenderLimits, RenderPhase, RenderQuantity, RenderWork,
    resource::{ResourceManager, ResourceRetentionOutcome, lock_state},
};

fn two_idle_entries() -> ResourceManager {
    let manager = ResourceManager::default();
    let mut frame = manager.begin_frame(None).unwrap();
    let first = frame.acquire(ResourceCacheKey::VelloAtlas, 4).unwrap();
    let second = frame
        .acquire(ResourceCacheKey::EffectTexture(descriptor().cache_key()), 4)
        .unwrap();
    frame.release(first).unwrap();
    frame.release(second).unwrap();
    drop(frame.finish());
    manager
}

fn descriptor() -> EffectTextureDescriptor {
    EffectTextureDescriptor::try_capture(
        crate::PhysicalSize::new(1, 1),
        wgpu::TextureUsages::TEXTURE_BINDING,
    )
    .unwrap()
}

fn context(limit: u64) -> RenderAccountingContext {
    RenderAccountingContext::new(
        RenderAccountingOptions::new().with_limits(
            RenderLimits::new()
                .with_work_limit(RenderWork::ResourceProbes, limit)
                .unwrap(),
        ),
    )
}

fn registry_facts(manager: &ResourceManager) -> (usize, usize, u64, u64) {
    let state = lock_state(&manager.state);
    (
        state.entries.len(),
        state.active_frames.len(),
        state.active_frames.last_identity(),
        state.next_resource,
    )
}

fn lifetime_probes(manager: &ResourceManager) -> u64 {
    manager
        .generation()
        .usage()
        .unwrap()
        .work(RenderWork::ResourceProbes)
}

#[test]
fn resource_entry_backing_refuses_before_payload_creation() {
    use crate::accounting::{RenderCpuScope, RendererCpu};

    let options = RenderAccountingOptions::new();
    let opening = RendererCpu::new(options)
        .unwrap()
        .snapshot()
        .unwrap()
        .usage()
        .live(RenderResource::RetainedCpuBytes);
    let frame_bytes =
        std::mem::size_of::<(FrameIdentity, crate::resource::lease::FrameScratch)>() as u64;
    let options =
        options.with_limits(RenderLimits::new().with_retained_bytes(opening + frame_bytes));
    let cpu = RendererCpu::new(options).unwrap();
    let manager = ResourceManager::with_generation(
        ResourceCacheBudget::DEFAULT,
        Generation::new(options).unwrap(),
        cpu.at(RenderCpuScope::DeviceBacking),
    );
    let mut frame = manager.begin_frame(None).unwrap();
    let before = manager.observation_for_test();
    let error = frame
        .acquire(ResourceCacheKey::VelloAtlas, 4)
        .expect_err("fresh resource admission needs funded index capacity");
    assert!(matches!(
        error.accounting_diagnostic(),
        Some(RenderAccountingError::CpuRefused {
            quantity: RenderQuantity::Resource(RenderResource::RetainedCpuBytes),
            ..
        })
    ));
    assert_eq!(manager.observation_for_test(), before);
    assert_eq!(registry_facts(&manager), (0, 1, 1, 0));
    drop(frame.finish_checked().unwrap());
    cpu.ensure_healthy().unwrap();
    manager.generation().ensure_healthy().unwrap();
}

#[test]
fn resource_entry_growth_preserves_old_entries_on_refusal_and_reuses_vacancy() {
    use crate::accounting::{RenderCpuScope, RenderProfile, RendererCpu};

    let bytes = crate::resource::entries::RECORD_BYTES;
    let frame_bytes =
        std::mem::size_of::<(FrameIdentity, crate::resource::lease::FrameScratch)>() as u64;
    let options =
        RenderAccountingOptions::new().with_profile(RenderProfile::Summary { timing: false });
    let opening = RendererCpu::new(options)
        .unwrap()
        .snapshot()
        .unwrap()
        .usage()
        .live(RenderResource::RetainedCpuBytes);
    let options = options
        .with_limits(RenderLimits::new().with_retained_bytes(opening + frame_bytes + 3 * bytes));
    let cpu = RendererCpu::new(options).unwrap();
    let manager = ResourceManager::with_generation(
        ResourceCacheBudget::DISABLED,
        Generation::new(options).unwrap(),
        cpu.at(RenderCpuScope::DeviceBacking),
    );
    let mut frame = manager.begin_frame(None).unwrap();
    let first = frame.acquire(ResourceCacheKey::VelloAtlas, 4).unwrap();
    let second = frame.acquire(ResourceCacheKey::VelloAtlas, 4).unwrap();
    let before = manager.observation_for_test();
    let error = frame.acquire(ResourceCacheKey::VelloAtlas, 4).unwrap_err();
    assert!(matches!(
        error.accounting_diagnostic(),
        Some(RenderAccountingError::CpuRefused {
            quantity: RenderQuantity::Resource(RenderResource::RetainedCpuBytes),
            current,
            amount,
            ..
        }) if current == opening + frame_bytes + 2 * bytes && amount == 3 * bytes
    ));
    assert_eq!(manager.observation_for_test(), before);
    let second_id = second.resource_identity();
    frame.discard(first).unwrap();
    let third = frame.acquire(ResourceCacheKey::VelloAtlas, 4).unwrap();
    assert_eq!(third.resource_identity(), ResourceIdentity(3));
    assert_eq!(
        manager.observation_for_test().entry_identities_for_test(),
        &[second_id, ResourceIdentity(3)]
    );
    let snapshot = cpu.snapshot().unwrap();
    let backing = snapshot.scope_usage(RenderCpuScope::DeviceBacking).unwrap();
    assert_eq!(
        backing.peak(RenderResource::RetainedCpuBytes),
        frame_bytes + 3 * bytes
    );
    assert_eq!(
        backing.live(RenderResource::RetainedCpuBytes),
        frame_bytes + 2 * bytes
    );
    assert_eq!(backing.work(RenderWork::StorageRelocations), 1);
    assert_eq!(
        snapshot
            .scope_usage(RenderCpuScope::Cleanup)
            .unwrap()
            .work(RenderWork::ResourceCacheRecordMoves),
        1
    );
    frame.release(second).unwrap();
    frame.release(third).unwrap();
    assert_eq!(
        frame.finish().evicted_resources(),
        &[second_id, ResourceIdentity(3)]
    );
    assert_eq!(manager.generation().registered_cache_bytes().unwrap(), 0);
    drop(manager);
    assert_eq!(
        cpu.snapshot()
            .unwrap()
            .usage()
            .live(RenderResource::RetainedCpuBytes),
        opening
    );
}

#[test]
fn graph_preflight_reserves_all_fresh_entry_slots_before_creation() {
    use crate::accounting::{RenderCpuScope, RenderProfile, RendererCpu};

    let bytes = crate::resource::entries::RECORD_BYTES;
    let options =
        RenderAccountingOptions::new().with_profile(RenderProfile::Summary { timing: false });
    let cpu = RendererCpu::new(options).unwrap();
    let manager = ResourceManager::with_generation(
        ResourceCacheBudget::DEFAULT,
        Generation::new(options).unwrap(),
        cpu.at(RenderCpuScope::DeviceBacking),
    );
    let request = ResourceAllocationPreflight::effect_texture(descriptor()).unwrap();
    manager
        .preflight_graph_acquisitions(&[request.clone(), request], None)
        .unwrap();
    assert_eq!(registry_facts(&manager), (0, 0, 0, 0));
    assert_eq!(manager.observation_for_test().payload_creation_attempts, 0);
    let backing = cpu
        .snapshot()
        .unwrap()
        .scope_usage(RenderCpuScope::DeviceBacking)
        .unwrap();
    assert_eq!(backing.live(RenderResource::RetainedCpuBytes), 2 * bytes);
    let mut frame = manager.begin_frame(None).unwrap();
    let key = ResourceCacheKey::EffectTexture(descriptor().cache_key());
    let first = frame.acquire(key.clone(), 4).unwrap();
    let second = frame.acquire(key, 4).unwrap();
    let backing = cpu
        .snapshot()
        .unwrap()
        .scope_usage(RenderCpuScope::DeviceBacking)
        .unwrap();
    assert_eq!(backing.work(RenderWork::StorageRelocations), 0);
    frame.release(first).unwrap();
    frame.release(second).unwrap();
    drop(frame.finish_checked().unwrap());
    assert_eq!(manager.observation_for_test().idle_count, 2);
}

#[test]
fn active_frame_admission_refuses_unfunded_index_without_advancing_identity() {
    use crate::accounting::{RenderCpuScope, RenderResource, RendererCpu};

    let options = RenderAccountingOptions::new();
    let opening = RendererCpu::new(options)
        .unwrap()
        .snapshot()
        .unwrap()
        .usage()
        .live(RenderResource::RetainedCpuBytes);
    let options = options.with_limits(RenderLimits::new().with_retained_bytes(opening));
    let cpu = RendererCpu::new(options).unwrap();
    let manager = ResourceManager::with_generation(
        ResourceCacheBudget::DEFAULT,
        Generation::new(options).unwrap(),
        cpu.at(RenderCpuScope::DeviceBacking),
    );
    let error = manager
        .begin_frame(None)
        .err()
        .expect("the first active-frame record needs retained backing");
    assert!(matches!(
        error.accounting_diagnostic(),
        Some(RenderAccountingError::CpuRefused {
            quantity: RenderQuantity::Resource(RenderResource::RetainedCpuBytes),
            current,
            ..
        }) if current == opening
    ));
    assert_eq!(registry_facts(&manager), (0, 0, 0, 0));
    let usage = cpu.snapshot().unwrap().usage();
    assert_eq!(usage.live(RenderResource::RetainedCpuBytes), opening);
    assert_eq!(usage.peak(RenderResource::RetainedCpuBytes), opening);
    cpu.ensure_healthy().unwrap();
}

#[test]
fn active_frame_index_growth_refusal_preserves_live_scope_and_retry_identity() {
    use crate::accounting::{RenderCpuScope, RenderProfile, RendererCpu};

    let bytes = std::mem::size_of::<(FrameIdentity, crate::resource::lease::FrameScratch)>() as u64;
    let options =
        RenderAccountingOptions::new().with_profile(RenderProfile::Summary { timing: false });
    let opening = RendererCpu::new(options)
        .unwrap()
        .snapshot()
        .unwrap()
        .usage()
        .live(RenderResource::RetainedCpuBytes);
    let options = options.with_limits(RenderLimits::new().with_retained_bytes(opening + bytes));
    let cpu = RendererCpu::new(options).unwrap();
    let manager = ResourceManager::with_generation(
        ResourceCacheBudget::DEFAULT,
        Generation::new(options).unwrap(),
        cpu.at(RenderCpuScope::DeviceBacking),
    );
    let first = manager.begin_frame(None).unwrap();
    assert_eq!(first.frame, FrameIdentity(1));
    let error = manager
        .begin_frame(None)
        .err()
        .expect("replacement overlaps the first allocation");
    assert!(matches!(
        error.accounting_diagnostic(),
        Some(RenderAccountingError::CpuRefused {
            quantity: RenderQuantity::Resource(RenderResource::RetainedCpuBytes),
            current,
            amount,
            ..
        }) if current == opening + bytes && amount == 2 * bytes
    ));
    assert_eq!(registry_facts(&manager), (0, 1, 1, 0));
    drop(first.finish_checked().unwrap());
    let retry = manager.begin_frame(None).unwrap();
    assert_eq!(retry.frame, FrameIdentity(2));
    drop(retry);
    let snapshot = cpu.snapshot().unwrap();
    let backing = snapshot.scope_usage(RenderCpuScope::DeviceBacking).unwrap();
    assert_eq!(backing.live(RenderResource::RetainedCpuBytes), bytes);
    assert_eq!(backing.peak(RenderResource::RetainedCpuBytes), bytes);
    assert_eq!(registry_facts(&manager), (0, 0, 2, 0));
    drop(manager);
    assert_eq!(
        cpu.snapshot()
            .unwrap()
            .usage()
            .live(RenderResource::RetainedCpuBytes),
        opening
    );
    cpu.ensure_healthy().unwrap();
}

#[test]
fn active_frame_index_growth_charges_overlap_and_ordered_removal() {
    use crate::accounting::{RenderCpuScope, RenderProfile, RendererCpu};

    let bytes = std::mem::size_of::<(FrameIdentity, crate::resource::lease::FrameScratch)>() as u64;
    let options =
        RenderAccountingOptions::new().with_profile(RenderProfile::Summary { timing: false });
    let cpu = RendererCpu::new(options).unwrap();
    let manager = ResourceManager::with_generation(
        ResourceCacheBudget::DEFAULT,
        Generation::new(options).unwrap(),
        cpu.at(RenderCpuScope::DeviceBacking),
    );
    let first = manager.begin_frame(None).unwrap();
    let second = manager.begin_frame(None).unwrap();
    let snapshot = cpu.snapshot().unwrap();
    let backing = snapshot.scope_usage(RenderCpuScope::DeviceBacking).unwrap();
    assert_eq!(backing.live(RenderResource::RetainedCpuBytes), 2 * bytes);
    assert_eq!(backing.peak(RenderResource::RetainedCpuBytes), 3 * bytes);
    assert_eq!(backing.work(RenderWork::StorageRelocations), 1);
    let third = manager.begin_frame(None).unwrap();
    drop(second.finish_checked().unwrap());
    let frames: Vec<_> = lock_state(&manager.state)
        .active_frames
        .iter()
        .map(|(&frame, _)| frame)
        .collect();
    assert_eq!(frames, [first.frame, third.frame]);
    drop(third);
    drop(first.finish_checked().unwrap());
    assert_eq!(registry_facts(&manager), (0, 0, 3, 0));
    let snapshot = cpu.snapshot().unwrap();
    let cleanup = snapshot.scope_usage(RenderCpuScope::Cleanup).unwrap();
    assert_eq!(cleanup.work(RenderWork::ActiveFrameRecordMoves), 1);
    assert_eq!(cleanup.live(RenderResource::ScratchCpuBytes), 0);
}

#[test]
fn mutable_registration_receipt_names_the_missing_identity_and_preserves_custody() {
    use crate::RenderResource;
    use surgeist_ledger::Dimension;

    let manager = two_idle_entries();
    let before = manager.observation_for_test();
    let generation = manager.generation();
    let usage = generation.usage().unwrap();
    {
        let mut state = lock_state(&manager.state);
        let identities: Vec<_> = state.entries.keys().copied().collect();
        let missing = ResourceIdentity(u64::MAX);
        assert_eq!(
            ResourceManagerState::selected_entry_mut(&mut state.entries, missing).err(),
            Some(ResourceAccountingFault::SelectedResourceMissing { resource: missing })
        );
        for (identity, class) in [
            (identities[0], GenerationClass::CacheVello),
            (identities[1], GenerationClass::CacheEffects),
        ] {
            let entry =
                ResourceManagerState::selected_entry_mut(&mut state.entries, identity).unwrap();
            assert_eq!(entry.byte_len, 4);
            assert_eq!(entry.key.generation_class(), class);
        }
        assert_eq!(state.accounting_fault, None);
    }
    assert_eq!(manager.observation_for_test(), before);
    let after = generation.usage().unwrap();
    for &work in RenderWork::ALL {
        assert_eq!(after.work(work), usage.work(work));
    }
    for &resource in RenderResource::ALL {
        assert_eq!(after.live(resource), usage.live(resource));
        assert_eq!(after.peak(resource), usage.peak(resource));
    }
    generation.ensure_healthy().unwrap();
}

#[test]
fn idle_reservation_receipt_returns_exact_insertion_positions_or_duplicate_identity() {
    let reserved = [ResourceIdentity(2), ResourceIdentity(5)];
    assert_eq!(
        ResourceManagerState::idle_reservation_index(&[], ResourceIdentity(3)),
        Ok(0)
    );
    for (identity, index) in [
        (ResourceIdentity(1), 0),
        (ResourceIdentity(3), 1),
        (ResourceIdentity(8), 2),
    ] {
        assert_eq!(
            ResourceManagerState::idle_reservation_index(&reserved, identity),
            Ok(index)
        );
    }
    for identity in reserved {
        assert_eq!(
            ResourceManagerState::idle_reservation_index(&reserved, identity),
            Err(ResourceAccountingFault::SelectedResourceAlreadyReserved { resource: identity })
        );
    }
    assert_eq!(reserved, [ResourceIdentity(2), ResourceIdentity(5)]);
}

#[test]
fn absent_selected_entry_preserves_custody_and_real_removal_evicts_its_exact_bytes() {
    let manager = two_idle_entries();
    let mut guard = lock_state(&manager.state);
    let state = &mut *guard;
    let generation = state.generation.clone();
    let identities: Vec<_> = state.entries.keys().copied().collect();
    let vello_before = generation.class_usage(GenerationClass::CacheVello).unwrap();
    let effects_before = generation
        .class_usage(GenerationClass::CacheEffects)
        .unwrap();
    let probes_before = generation.cache_work(RenderWork::ResourceProbes).unwrap();
    let missing = ResourceIdentity(u64::MAX);

    assert_eq!(
        ResourceManagerState::remove_selected_entry(&generation, &mut state.entries, missing),
        Err(ResourceAccountingFault::SelectedResourceMissing { resource: missing })
    );
    assert_eq!(
        state.entries.keys().copied().collect::<Vec<_>>(),
        identities
    );
    assert_eq!(state.accounting_fault, None);
    assert_eq!(generation.cache_bytes().unwrap(), 8);
    assert_eq!(generation.registered_cache_bytes().unwrap(), 8);
    assert_eq!(generation.idle_cache_bytes().unwrap(), 8);
    for (class, before) in [
        (GenerationClass::CacheVello, &vello_before),
        (GenerationClass::CacheEffects, &effects_before),
    ] {
        let after = generation.class_usage(class).unwrap();
        for work in [
            RenderWork::ResourceEvictions,
            RenderWork::ResourceLeaseReturns,
            RenderWork::ResourceProbes,
        ] {
            assert_eq!(after.work(work), before.work(work));
        }
    }

    assert_eq!(
        ResourceManagerState::remove_selected_entry(&generation, &mut state.entries, identities[0]),
        Ok(4)
    );
    assert_eq!(
        state.entries.keys().copied().collect::<Vec<_>>(),
        vec![identities[1]]
    );
    assert_eq!(generation.cache_bytes().unwrap(), 4);
    assert_eq!(generation.registered_cache_bytes().unwrap(), 4);
    assert_eq!(generation.idle_cache_bytes().unwrap(), 4);
    let vello_after = generation.class_usage(GenerationClass::CacheVello).unwrap();
    assert_eq!(
        vello_after.work(RenderWork::ResourceEvictions)
            - vello_before.work(RenderWork::ResourceEvictions),
        1
    );
    assert_eq!(
        vello_after.work(RenderWork::ResourceLeaseReturns),
        vello_before.work(RenderWork::ResourceLeaseReturns)
    );
    let effects_after = generation
        .class_usage(GenerationClass::CacheEffects)
        .unwrap();
    for work in [
        RenderWork::ResourceEvictions,
        RenderWork::ResourceLeaseReturns,
        RenderWork::ResourceProbes,
    ] {
        assert_eq!(effects_after.work(work), effects_before.work(work));
    }
    assert_eq!(
        generation.cache_work(RenderWork::ResourceProbes).unwrap(),
        probes_before
    );
    assert_eq!(state.accounting_fault, None);
    generation.ensure_healthy().unwrap();
}

#[test]
fn registry_frame_audit_refuses_the_first_unadmitted_entry_and_retries_without_mutation() {
    let manager = two_idle_entries();
    let before = registry_facts(&manager);
    for limit in [0, 1] {
        let mut context = context(limit);
        let error = context
            .run(
                RenderAttemptKind::Frame,
                RenderPhase::Frame,
                |account| match manager.begin_frame(Some(account)) {
                    Err(error) => Err::<(), _>(error),
                    Ok(_) => panic!("two registry entries must require two audit probes"),
                },
            )
            .unwrap_err();
        assert!(matches!(
            error.accounting_diagnostic(),
            Some(RenderAccountingError::Refused(refusal))
                if refusal.quantity() == RenderQuantity::Work(RenderWork::ResourceProbes)
        ));
        let report = context.last_attempt().unwrap().attempt().unwrap();
        assert_eq!(report.status(), RenderAttemptStatus::Refused);
        assert_eq!(report.usage().work(RenderWork::ResourceProbes), limit);
        assert_eq!(
            report.usage().work(RenderWork::ResourceAllocationRequests),
            0
        );
        assert_eq!(registry_facts(&manager), before);
        assert_eq!(manager.generation().cache_bytes().unwrap(), 8);
        manager.generation().ensure_healthy().unwrap();
    }
    let mut retry = context(2);
    retry
        .run(RenderAttemptKind::Frame, RenderPhase::Frame, |account| {
            let lifetime_before = lifetime_probes(&manager);
            let frame = manager.begin_frame(Some(account))?;
            assert_eq!(lifetime_probes(&manager), lifetime_before);
            drop(frame.finish());
            Ok(())
        })
        .unwrap();
    assert_eq!(
        retry
            .last_attempt()
            .unwrap()
            .attempt()
            .unwrap()
            .usage()
            .work(RenderWork::ResourceProbes),
        2
    );
}

#[test]
fn repeated_preflight_requests_require_each_distinct_idle_reservation_lookup() {
    use crate::{
        RenderResource, ResourceCacheBudget,
        accounting::{RenderCpuScope, RendererCpu},
    };
    use surgeist_ledger::Dimension;

    let options = RenderAccountingOptions::default();
    let cpu = RendererCpu::new(options).unwrap();
    let manager = ResourceManager::with_generation(
        ResourceCacheBudget::DEFAULT,
        Generation::new(options).unwrap(),
        cpu.at(RenderCpuScope::DeviceBacking),
    );
    let key = ResourceCacheKey::EffectTexture(descriptor().cache_key());
    let mut frame = manager.begin_frame(None).unwrap();
    let first = frame.acquire(key.clone(), 4).unwrap();
    let second = frame.acquire(key, 4).unwrap();
    assert_ne!(first.resource_identity(), second.resource_identity());
    frame.release(first).unwrap();
    frame.release(second).unwrap();
    drop(frame.finish());
    let request = ResourceAllocationPreflight::effect_texture(descriptor()).unwrap();
    let requests = [request.clone(), request];
    let before = manager.observation_for_test();
    let facts = registry_facts(&manager);
    let generation = manager.generation().usage().unwrap();
    assert_eq!(before.idle_count, 2);
    assert_eq!(
        cpu.snapshot()
            .unwrap()
            .usage()
            .live(RenderResource::ScratchCpuBytes),
        0
    );

    // Two audit visits, four candidate visits, four membership lookups and
    // two separate reservation searches require twelve admitted probes.
    for limit in [11, 12] {
        let mut context = context(limit);
        let result = context.run(RenderAttemptKind::Frame, RenderPhase::Frame, |account| {
            manager.preflight_graph_acquisitions(&requests, Some(account))
        });
        if limit == 11 {
            let error = result.unwrap_err();
            let Some(RenderAccountingError::Refused(refusal)) = error.accounting_diagnostic()
            else {
                panic!("the second reservation lookup requires its own admission");
            };
            assert_eq!(
                refusal.quantity(),
                RenderQuantity::Work(RenderWork::ResourceProbes)
            );
            assert_eq!((refusal.current(), refusal.amount()), (11, 1));
            assert_eq!(refusal.reason(), RenderRefusalReason::Limit { limit: 11 });
        } else {
            result.unwrap();
        }
        let report = context.last_attempt().unwrap().attempt().unwrap();
        assert_eq!(report.usage().work(RenderWork::ResourceProbes), limit);
        assert_eq!(
            report.usage().work(RenderWork::ResourceAllocationRequests),
            0
        );
        assert_eq!(registry_facts(&manager), facts);
        assert_eq!(manager.observation_for_test(), before);
        let after = manager.generation().usage().unwrap();
        for &resource in RenderResource::ALL {
            assert_eq!(after.live(resource), generation.live(resource));
            assert_eq!(after.peak(resource), generation.peak(resource));
        }
        for &work in RenderWork::ALL {
            assert_eq!(after.work(work), generation.work(work));
        }
        assert_eq!(
            cpu.snapshot()
                .unwrap()
                .usage()
                .live(RenderResource::ScratchCpuBytes),
            0
        );
        manager.generation().ensure_healthy().unwrap();
    }
}

#[test]
fn preflight_reserves_distinct_idle_entries_before_previewing_fresh_bytes_at_capacity() {
    use crate::{
        RenderResource, ResourceCacheBudget,
        accounting::{RenderCpuScope, RendererCpu},
    };
    use surgeist_ledger::Dimension;

    let options = RenderAccountingOptions::default();
    let cpu = RendererCpu::new(options).unwrap();
    let manager = ResourceManager::with_generation(
        ResourceCacheBudget::new(u64::MAX),
        Generation::new(options).unwrap(),
        cpu.at(RenderCpuScope::DeviceBacking),
    );
    let key = ResourceCacheKey::EffectTexture(descriptor().cache_key());
    let mut frame = manager.begin_frame(None).unwrap();
    let first = frame.acquire(key.clone(), 4).unwrap();
    let second = frame.acquire(key, 4).unwrap();
    assert_ne!(first.resource_identity(), second.resource_identity());
    frame.release(first).unwrap();
    frame.release(second).unwrap();
    drop(frame.finish());
    let generation = manager.generation();
    let filler = generation
        .allocation(GenerationClass::CacheVello, u64::MAX - 8, None, || ())
        .unwrap();
    assert_eq!(generation.cache_bytes().unwrap(), u64::MAX);
    assert_eq!(generation.registered_cache_bytes().unwrap(), 8);
    assert_eq!(generation.idle_cache_bytes().unwrap(), 8);
    let before = manager.observation_for_test();
    let facts = registry_facts(&manager);
    let usage = generation.usage().unwrap();
    let request = ResourceAllocationPreflight::effect_texture(descriptor()).unwrap();
    let requests = [request.clone(), request.clone(), request];

    for (count, probes) in [(2, 12), (3, 16)] {
        let mut context = context(u64::MAX);
        let result = context.run(RenderAttemptKind::Frame, RenderPhase::Frame, |account| {
            manager.preflight_graph_acquisitions(&requests[..count], Some(account))
        });
        if count == 2 {
            result.unwrap();
        } else {
            let error = result.unwrap_err();
            assert_eq!(error.code(), crate::ErrorCode::InvalidInput);
            let value = error.invalid_value_diagnostic().unwrap();
            assert_eq!(value.field(), "retained resource byte length");
            assert_eq!(value.value(), "18446744073709551615 + 4");
            assert_eq!(value.invariant(), "must fit in u64");
        }
        let report = context.last_attempt().unwrap().attempt().unwrap();
        assert_eq!(report.usage().work(RenderWork::ResourceProbes), probes);
        assert_eq!(
            report.usage().work(RenderWork::ResourceAllocationRequests),
            0
        );
        assert_eq!(registry_facts(&manager), facts);
        assert_eq!(manager.observation_for_test(), before);
        let after = generation.usage().unwrap();
        for &resource in RenderResource::ALL {
            assert_eq!(after.live(resource), usage.live(resource));
            assert_eq!(after.peak(resource), usage.peak(resource));
        }
        for &work in RenderWork::ALL {
            assert_eq!(after.work(work), usage.work(work));
        }
        assert_eq!(
            cpu.snapshot()
                .unwrap()
                .usage()
                .live(RenderResource::ScratchCpuBytes),
            0
        );
        generation.ensure_healthy().unwrap();
    }
    drop(filler);
    assert_eq!(generation.cache_bytes().unwrap(), 8);
    assert_eq!(generation.registered_cache_bytes().unwrap(), 8);
    assert_eq!(generation.idle_cache_bytes().unwrap(), 8);
}

#[test]
fn registry_preflight_audits_even_an_empty_request_batch_before_any_mutation() {
    let manager = two_idle_entries();
    let before = registry_facts(&manager);
    let mut refused = context(1);
    refused
        .run(RenderAttemptKind::Frame, RenderPhase::Frame, |account| {
            manager.preflight_graph_acquisitions(&[], Some(account))
        })
        .expect_err("the second retained entry must exceed one probe");
    assert_eq!(
        refused
            .last_attempt()
            .unwrap()
            .attempt()
            .unwrap()
            .usage()
            .work(RenderWork::ResourceProbes),
        1
    );
    assert_eq!(registry_facts(&manager), before);
    let mut retry = context(2);
    retry
        .run(RenderAttemptKind::Frame, RenderPhase::Frame, |account| {
            manager.preflight_graph_acquisitions(&[], Some(account))
        })
        .unwrap();
    assert_eq!(
        retry
            .last_attempt()
            .unwrap()
            .attempt()
            .unwrap()
            .usage()
            .work(RenderWork::ResourceProbes),
        2
    );
    assert_eq!(registry_facts(&manager), before);
}

#[test]
fn registry_acquisition_repeats_its_audit_before_candidate_selection() {
    let manager = two_idle_entries();
    let mut refused = context(3);
    refused
        .run(RenderAttemptKind::Frame, RenderPhase::Frame, |account| {
            let mut frame = manager.begin_frame(Some(account))?;
            let before = registry_facts(&manager);
            let result = frame.acquire(ResourceCacheKey::VelloAtlas, 4);
            assert!(
                result.is_err(),
                "two setup probes leave one for the repeated two-entry audit"
            );
            assert_eq!(registry_facts(&manager), before);
            result.map(drop)
        })
        .unwrap_err();
    let report = refused.last_attempt().unwrap().attempt().unwrap();
    assert_eq!(report.usage().work(RenderWork::ResourceProbes), 3);
    assert_eq!(
        report.usage().work(RenderWork::ResourceAllocationRequests),
        0
    );
    assert_eq!(manager.generation().cache_bytes().unwrap(), 8);
    assert_eq!(registry_facts(&manager).1, 0);
}

#[test]
fn registry_idle_summary_observes_both_complete_entry_walks() {
    let manager = two_idle_entries();
    let before = lifetime_probes(&manager);
    assert_eq!(
        lock_state(&manager.state).checked_idle_summary(),
        Some((2, 8))
    );
    assert_eq!(lifetime_probes(&manager) - before, 4);
}

#[test]
fn registry_reuse_requires_its_selected_entry_lookup_before_changing_the_lease() {
    let manager = two_idle_entries();
    let mut refused = context(6);
    refused
        .run(RenderAttemptKind::Frame, RenderPhase::Frame, |account| {
            let mut frame = manager.begin_frame(Some(account))?;
            let before = registry_facts(&manager);
            let reuses = manager
                .generation()
                .cache_work(RenderWork::ResourceReuses)?;
            let result = frame.acquire(ResourceCacheKey::VelloAtlas, 4);
            assert!(
                result.is_err(),
                "two setup, two audit and two candidate probes leave none for selection"
            );
            assert_eq!(registry_facts(&manager), before);
            assert_eq!(
                manager
                    .generation()
                    .cache_work(RenderWork::ResourceReuses)?,
                reuses
            );
            result.map(drop)
        })
        .unwrap_err();
    assert_eq!(
        refused
            .last_attempt()
            .unwrap()
            .attempt()
            .unwrap()
            .usage()
            .work(RenderWork::ResourceProbes),
        6
    );
    let mut retry = context(7);
    retry
        .run(RenderAttemptKind::Frame, RenderPhase::Frame, |account| {
            let mut frame = manager.begin_frame(Some(account))?;
            let lease = frame.acquire(ResourceCacheKey::VelloAtlas, 4)?;
            frame.release(lease)?;
            drop(frame.finish());
            Ok(())
        })
        .unwrap();
    assert_eq!(
        retry
            .last_attempt()
            .unwrap()
            .attempt()
            .unwrap()
            .usage()
            .work(RenderWork::ResourceProbes),
        7
    );
    assert_eq!(
        retry
            .last_attempt()
            .unwrap()
            .attempt()
            .unwrap()
            .usage()
            .work(RenderWork::ResourceAllocationRequests),
        0
    );
}

#[test]
fn registry_cleanup_observes_its_walks_after_the_attempt_allowance_is_spent() {
    let manager = ResourceManager::default();
    let mut context = context(0);
    context
        .run(RenderAttemptKind::Frame, RenderPhase::Frame, |account| {
            let mut frame = manager.begin_frame(Some(account))?;
            let lease = frame.acquire(ResourceCacheKey::VelloAtlas, 4)?;
            drop(lease);
            let first = account.work(RenderWork::ResourceProbes, 1).unwrap_err();
            let before = lifetime_probes(&manager);
            let cleanup = frame.finish();
            assert_eq!(
                cleanup.retention,
                ResourceRetentionOutcome::RetainedReusable {
                    resource_count: 1,
                    byte_len: 4
                }
            );
            // One lease-return walk, two invariant audits, one idle-selection
            // walk and two idle-summary walks each inspect the one entry.
            assert_eq!(lifetime_probes(&manager) - before, 6);
            assert_eq!(registry_facts(&manager).1, 0);
            assert_eq!(manager.generation().cache_bytes().unwrap(), 4);
            Err::<(), _>(first)
        })
        .unwrap_err();
    assert_eq!(
        context
            .last_attempt()
            .unwrap()
            .attempt()
            .unwrap()
            .usage()
            .work(RenderWork::ResourceProbes),
        0
    );
    assert_eq!(
        context.last_attempt().unwrap().attempt().unwrap().status(),
        RenderAttemptStatus::Refused
    );
    manager.generation().ensure_healthy().unwrap();
}

#[test]
fn registry_release_observes_validation_and_the_later_mutable_lookup() {
    let manager = ResourceManager::default();
    let mut frame = manager.begin_frame(None).unwrap();
    let lease = frame.acquire(ResourceCacheKey::VelloAtlas, 4).unwrap();
    let before = lifetime_probes(&manager);
    frame.release(lease).unwrap();
    assert_eq!(lifetime_probes(&manager) - before, 2);
    drop(frame.finish());
}

#[test]
fn registry_resolution_observes_each_validation_and_mutable_lookup() {
    let manager = ResourceManager::default();
    let mut frame = manager.begin_frame(None).unwrap();
    let first = frame.acquire(ResourceCacheKey::VelloAtlas, 4).unwrap();
    let second = frame
        .acquire(ResourceCacheKey::EffectTexture(descriptor().cache_key()), 4)
        .unwrap();
    let before = lifetime_probes(&manager);
    frame.resolve_leases_atomically(&[&first, &second]).unwrap();
    // Two validation lookups and one later mutable lookup per lease. The
    // selected entry also supplies its class, without another indexed lookup.
    assert_eq!(lifetime_probes(&manager) - before, 4);
    drop((first, second));
    drop(frame.finish());
}

#[test]
fn registry_discard_observes_validation_and_the_actual_removal_lookup() {
    let manager = ResourceManager::default();
    let mut frame = manager.begin_frame(None).unwrap();
    let lease = frame.acquire(ResourceCacheKey::VelloAtlas, 4).unwrap();
    let before = lifetime_probes(&manager);
    frame.discard(lease).unwrap();
    assert_eq!(lifetime_probes(&manager) - before, 2);
    assert_eq!(manager.generation().cache_bytes().unwrap(), 0);
    drop(frame.finish());
}

#[test]
fn registry_provisional_abort_observes_its_predicate_and_removal_after_refusal() {
    let manager = ResourceManager::default();
    let mut context = context(0);
    context
        .run(RenderAttemptKind::Frame, RenderPhase::Frame, |account| {
            let mut frame = manager.begin_frame(Some(account))?;
            let lease = frame.acquire(ResourceCacheKey::VelloAtlas, 4)?;
            drop(lease);
            frame.abort_provisional_on_drop();
            let first = account.work(RenderWork::ResourceProbes, 1).unwrap_err();
            let before = lifetime_probes(&manager);
            drop(frame);
            assert_eq!(lifetime_probes(&manager) - before, 2);
            assert_eq!(registry_facts(&manager).0, 0);
            assert_eq!(registry_facts(&manager).1, 0);
            assert_eq!(manager.generation().cache_bytes().unwrap(), 0);
            Err::<(), _>(first)
        })
        .unwrap_err();
    assert_eq!(
        context
            .last_attempt()
            .unwrap()
            .attempt()
            .unwrap()
            .usage()
            .work(RenderWork::ResourceProbes),
        0
    );
    assert_eq!(
        context.last_attempt().unwrap().attempt().unwrap().status(),
        RenderAttemptStatus::Refused
    );
}
