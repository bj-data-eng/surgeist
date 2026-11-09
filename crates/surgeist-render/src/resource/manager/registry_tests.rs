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
        state.next_frame,
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
