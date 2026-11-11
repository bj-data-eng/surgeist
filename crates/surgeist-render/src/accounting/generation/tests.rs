use super::*;
use std::sync::atomic::{AtomicBool, AtomicUsize, Ordering};

#[derive(Clone)]
struct Native {
    generation: Generation,
    drops: Arc<AtomicUsize>,
}
impl Drop for Native {
    fn drop(&mut self) {
        assert_eq!(
            self.generation
                .usage()
                .unwrap()
                .live(RenderResource::GpuBytes),
            16
        );
        self.drops.fetch_add(1, Ordering::SeqCst);
    }
}

#[test]
fn native_clones_share_physical_hold_and_drop_callbacks_run_before_release_unlocked() {
    let generation = Generation::new(RenderAccountingOptions::default()).unwrap();
    let drops = Arc::new(AtomicUsize::new(0));
    let value = generation
        .allocation(GenerationClass::CacheEffects, 16, None, || {
            assert_eq!(
                generation.usage().unwrap().live(RenderResource::GpuBytes),
                0
            );
            Native {
                generation: generation.clone(),
                drops: drops.clone(),
            }
        })
        .unwrap();
    let clone = value.clone();
    let mut registration = generation
        .cache_registration(GenerationClass::CacheEffects, 16)
        .unwrap();
    registration.set_idle(true).unwrap();
    let usage = generation.usage().unwrap();
    assert_eq!(usage.live(RenderResource::GpuBytes), 16);
    assert_eq!(usage.live(RenderResource::RegisteredCacheGpuBytes), 16);
    assert_eq!(usage.live(RenderResource::IdleCacheGpuBytes), 16);
    drop(registration);
    assert_eq!(generation.registered_cache_bytes().unwrap(), 0);
    assert_eq!(generation.idle_cache_bytes().unwrap(), 0);
    drop(value);
    assert_eq!(generation.cache_bytes().unwrap(), 16);
    drop(clone);
    assert_eq!(generation.cache_bytes().unwrap(), 0);
    assert_eq!(drops.load(Ordering::SeqCst), 2);
}

#[test]
fn independent_allocations_have_exact_overlap_without_summing_historical_peaks() {
    let generation = Generation::new(RenderAccountingOptions::default()).unwrap();
    let first = generation
        .allocation(GenerationClass::CacheEffects, 16, None, || ())
        .unwrap();
    let second = generation
        .allocation(GenerationClass::SurfaceTargets, 8, None, || ())
        .unwrap();
    assert_eq!(
        generation.usage().unwrap().peak(RenderResource::GpuBytes),
        24
    );
    drop(second);
    let third = generation
        .allocation(GenerationClass::PassBuffers, 4, None, || ())
        .unwrap();
    assert_eq!(
        generation.usage().unwrap().live(RenderResource::GpuBytes),
        20
    );
    assert_eq!(
        generation.usage().unwrap().peak(RenderResource::GpuBytes),
        24
    );
    drop(first);
    drop(third);
    assert_eq!(
        generation.usage().unwrap().live(RenderResource::GpuBytes),
        0
    );
}

#[test]
fn retirement_zeros_every_leaf_and_surviving_native_and_registration_drops_are_harmless() {
    let generation = Generation::new(RenderAccountingOptions::default()).unwrap();
    let classes = [
        GenerationClass::CacheVello,
        GenerationClass::CacheEffects,
        GenerationClass::CacheMasks,
        GenerationClass::CacheKernels,
        GenerationClass::SurfaceTargets,
        GenerationClass::PassBuffers,
        GenerationClass::ReadbackStaging,
    ];
    let values: Vec<_> = classes
        .into_iter()
        .enumerate()
        .map(|(i, class)| {
            generation
                .allocation(class, (i + 1) as u64, None, || ())
                .unwrap()
        })
        .collect();
    let mut registrations: Vec<_> = classes[..4]
        .iter()
        .enumerate()
        .map(|(i, class)| {
            generation
                .cache_registration(*class, (i + 1) as u64)
                .unwrap()
        })
        .collect();
    for registration in &mut registrations {
        registration.set_idle(true).unwrap();
    }
    let kinds = [
        GpuObjectKind::ShaderModule,
        GpuObjectKind::BindGroupLayout,
        GpuObjectKind::PipelineLayout,
        GpuObjectKind::ComputePipeline,
        GpuObjectKind::RenderPipeline,
        GpuObjectKind::Sampler,
        GpuObjectKind::BindGroup,
        GpuObjectKind::TextureView,
        GpuObjectKind::CommandEncoder,
        GpuObjectKind::CommandBuffer,
        GpuObjectKind::DependencyUtility,
    ];
    let objects: Vec<_> = kinds
        .into_iter()
        .map(|kind| generation.opaque(kind, None, || ()).unwrap())
        .collect();
    assert_eq!(
        generation.usage().unwrap().live(RenderResource::GpuBytes),
        28
    );
    assert_eq!(
        generation
            .usage()
            .unwrap()
            .live(RenderResource::OpaqueGpuObjects),
        11
    );
    generation.retire();
    generation.retire();
    for index in LEAVES {
        let usage = generation.usage_at(index).unwrap();
        for resource in [
            RenderResource::GpuBytes,
            RenderResource::OpaqueGpuObjects,
            RenderResource::RegisteredCacheGpuBytes,
            RenderResource::IdleCacheGpuBytes,
        ] {
            assert_eq!(usage.live(resource), 0);
        }
    }
    assert_eq!(
        generation.usage().unwrap().live(RenderResource::GpuBytes),
        0
    );
    drop(values);
    drop(objects);
    drop(registrations);
    assert!(generation.lock().fault.is_none());
    assert_eq!(
        generation
            .usage()
            .unwrap()
            .work(RenderWork::ResourceRetirements),
        1
    );
    let invoked = AtomicBool::new(false);
    assert!(
        generation
            .opaque(GpuObjectKind::Sampler, None, || invoked
                .store(true, Ordering::SeqCst))
            .is_err()
    );
    assert!(!invoked.load(Ordering::SeqCst));
}

#[test]
fn generation_named_roles_preserve_exact_usage_and_retirement_order() {
    use crate::accounting::{
        RenderChangeKind, RenderGenerationEvent, RenderPhaseFilter, RenderProfile,
    };
    use RenderGenerationScope as Scope;
    use RenderResource::{GpuBytes, IdleCacheGpuBytes, OpaqueGpuObjects, RegisteredCacheGpuBytes};
    use RenderWork::{
        GpuObjectsCreated, ResourceAllocationRequests, ResourceAllocations, ResourceRetirements,
    };

    let generation = Generation::new(RenderAccountingOptions::new().with_profile(
        RenderProfile::Detailed {
            timing: false,
            event_limit: 512,
            filter: RenderPhaseFilter::All,
        },
    ))
    .unwrap();
    let descriptors = [
        (GenerationClass::CacheVello, Scope::Vello, 1, 1),
        (GenerationClass::CacheEffects, Scope::Effects, 2, 2),
        (GenerationClass::CacheMasks, Scope::Masks, 3, 3),
        (GenerationClass::CacheKernels, Scope::Kernels, 4, 4),
        (GenerationClass::SurfaceTargets, Scope::SurfaceTargets, 5, 0),
        (GenerationClass::PassBuffers, Scope::PassBuffers, 6, 0),
        (
            GenerationClass::ReadbackStaging,
            Scope::ReadbackStaging,
            7,
            0,
        ),
    ];
    let object_kinds = [
        (GpuObjectKind::ShaderModule, Scope::ShaderModule, 1),
        (GpuObjectKind::BindGroupLayout, Scope::BindGroupLayout, 2),
        (GpuObjectKind::PipelineLayout, Scope::PipelineLayout, 3),
        (GpuObjectKind::ComputePipeline, Scope::ComputePipeline, 4),
        (GpuObjectKind::RenderPipeline, Scope::RenderPipeline, 5),
        (GpuObjectKind::Sampler, Scope::Sampler, 6),
        (GpuObjectKind::BindGroup, Scope::BindGroup, 7),
        (GpuObjectKind::TextureView, Scope::TextureView, 8),
        (GpuObjectKind::CommandEncoder, Scope::CommandEncoder, 9),
        (GpuObjectKind::CommandBuffer, Scope::CommandBuffer, 10),
        (
            GpuObjectKind::DependencyUtility,
            Scope::DependencyUtility,
            11,
        ),
    ];
    let mut allocations = Vec::new();
    let mut registrations = Vec::new();
    let mut objects = Vec::new();
    // Reverse creation makes the retirement sequence independent of insertion order.
    for (kind, _, count) in object_kinds.into_iter().rev() {
        for _ in 0..count {
            objects.push(generation.opaque(kind, None, || ()).unwrap());
        }
    }
    for (class, _, bytes, registered) in descriptors.into_iter().rev() {
        allocations.push(generation.allocation(class, bytes, None, || ()).unwrap());
        if registered != 0 {
            let mut registration = generation.cache_registration(class, registered).unwrap();
            registration.set_idle(true).unwrap();
            registrations.push(registration);
        }
    }

    let resources = [
        GpuBytes,
        OpaqueGpuObjects,
        RegisteredCacheGpuBytes,
        IdleCacheGpuBytes,
    ];
    let work = [
        ResourceAllocationRequests,
        ResourceAllocations,
        GpuObjectsCreated,
    ];
    let descriptor_rows =
        descriptors.map(|(_, scope, bytes, cache)| (scope, [bytes, 0, cache, cache], [1, 1, 0]));
    let object_rows =
        object_kinds.map(|(_, scope, count)| (scope, [0, count, 0, 0], [count, 0, count]));
    let parent_rows = [
        (Scope::Cache, [10, 0, 10, 10], [4, 4, 0]),
        (Scope::External, [18, 0, 0, 0], [3, 3, 0]),
        (Scope::Objects, [0, 66, 0, 0], [66, 0, 66]),
        (Scope::Root, [28, 66, 10, 10], [73, 7, 66]),
    ];
    let rows = descriptor_rows
        .into_iter()
        .chain(object_rows)
        .chain(parent_rows);
    let before = generation.snapshot().unwrap();
    assert_eq!(before.dropped_events(), 0);
    assert!(!before.retired());
    assert_eq!(before.fault(), None);
    assert_eq!(before.scope_count(Scope::Initialization), 0);

    let checkpoint = generation.begin_interval().unwrap();
    generation.retire();
    generation.retire();
    let retirement = checkpoint.finish().unwrap();
    let after = generation.snapshot().unwrap();
    assert!(after.retired());
    assert_eq!(after.identity(), before.identity());
    assert_eq!(retirement.identity(), before.identity());
    assert_eq!(after.fault(), None);
    assert_eq!(after.dropped_events(), 0);
    assert_eq!(retirement.dropped_events(), 0);
    assert_eq!(retirement.scope_count(Scope::Retirement), 1);
    for (scope, live, admitted) in rows {
        let initial = before.scope_usage(scope).unwrap();
        let final_usage = after.scope_usage(scope).unwrap();
        let interval = retirement.scope_usage(scope);
        assert_eq!(
            resources.map(|resource| initial.live(resource)),
            live,
            "{scope:?}"
        );
        assert_eq!(
            resources.map(|resource| retirement.scope_opening(scope, resource)),
            live,
            "{scope:?}"
        );
        assert_eq!(
            resources.map(|resource| final_usage.live(resource)),
            [0; 4],
            "{scope:?}"
        );
        assert_eq!(
            resources.map(|resource| interval.live(resource)),
            [0; 4],
            "{scope:?}"
        );
        assert_eq!(work.map(|unit| initial.work(unit)), admitted, "{scope:?}");
        assert_eq!(
            work.map(|unit| final_usage.work(unit)),
            admitted,
            "{scope:?}"
        );
        assert_eq!(work.map(|unit| interval.work(unit)), [0; 3], "{scope:?}");
    }
    assert_eq!(after.usage().work(ResourceRetirements), 1);
    assert_eq!(
        retirement
            .scope_usage(Scope::Retirement)
            .work(ResourceRetirements),
        1
    );

    let expected_events: Vec<_> = descriptor_rows
        .into_iter()
        .chain(object_rows)
        .flat_map(|(scope, live, _)| {
            resources
                .into_iter()
                .zip(live)
                .map(move |(resource, amount)| RenderGenerationEvent::Change {
                    scope,
                    quantity: RenderQuantity::Resource(resource),
                    amount,
                    kind: RenderChangeKind::Release,
                    elapsed: None,
                })
        })
        .chain([
            RenderGenerationEvent::Change {
                scope: Scope::Retirement,
                quantity: RenderQuantity::Work(ResourceRetirements),
                amount: 1,
                kind: RenderChangeKind::Work,
                elapsed: None,
            },
            RenderGenerationEvent::Phase {
                scope: Scope::Retirement,
                duration: None,
                elapsed: None,
            },
        ])
        .collect();
    assert_eq!(retirement.events().collect::<Vec<_>>(), expected_events);

    drop(allocations);
    drop(objects);
    drop(registrations);
    assert_eq!(generation.snapshot().unwrap(), after);
}

#[test]
fn retirement_during_native_creation_disposes_unpublished_handle_without_admission() {
    struct Dropped(Arc<AtomicUsize>);
    impl Drop for Dropped {
        fn drop(&mut self) {
            self.0.fetch_add(1, Ordering::SeqCst);
        }
    }
    let generation = Generation::new(RenderAccountingOptions::default()).unwrap();
    let drops = Arc::new(AtomicUsize::new(0));
    let result = generation.opaque(GpuObjectKind::ShaderModule, None, || {
        generation.retire();
        Dropped(drops.clone())
    });
    assert!(matches!(
        result.err().unwrap().accounting_diagnostic(),
        Some(RenderAccountingError::GenerationRetired)
    ));
    let usage = generation.usage().unwrap();
    assert_eq!(drops.load(Ordering::SeqCst), 1);
    assert_eq!(usage.peak(RenderResource::OpaqueGpuObjects), 0);
    assert_eq!(usage.work(RenderWork::ResourceAllocationRequests), 1);
    assert_eq!(usage.work(RenderWork::GpuObjectsCreated), 0);
}

#[test]
fn checked_overflow_prevents_constructor_and_does_not_veto_existing_custody_release() {
    let generation = Generation::new(RenderAccountingOptions::default()).unwrap();
    let existing = generation
        .allocation(GenerationClass::CacheEffects, u64::MAX, None, || ())
        .unwrap();
    let invoked = AtomicBool::new(false);
    let error = generation
        .allocation(GenerationClass::SurfaceTargets, 1, None, || {
            invoked.store(true, Ordering::SeqCst)
        })
        .err()
        .unwrap();
    assert!(matches!(
        error.accounting_diagnostic(),
        Some(RenderAccountingError::GenerationRefused {
            quantity: RenderQuantity::Resource(RenderResource::GpuBytes),
            reason: RenderRefusalReason::Overflow,
            ..
        })
    ));
    assert!(!invoked.load(Ordering::SeqCst));
    drop(existing);
    assert_eq!(
        generation.usage().unwrap().live(RenderResource::GpuBytes),
        0
    );
    generation.ensure_healthy().unwrap();
    drop(
        generation
            .allocation(GenerationClass::SurfaceTargets, 4, None, || ())
            .unwrap(),
    );
}

#[test]
fn lifecycle_work_overflow_records_first_fault_after_mandatory_custody_cleanup() {
    let generation = Generation::new(RenderAccountingOptions::default()).unwrap();
    let value = generation
        .allocation(GenerationClass::CacheEffects, 16, None, || ())
        .unwrap();
    let mut registration = generation
        .cache_registration(GenerationClass::CacheEffects, 16)
        .unwrap();
    registration.set_idle(true).unwrap();
    let scope = generation.0.scopes[GenerationClass::CacheEffects.index()];
    generation
        .0
        .ledger
        .apply(
            scope,
            Change::Work(RenderWork::ResourceLeaseReturns, u64::MAX),
        )
        .unwrap();
    generation.lease_return(GenerationClass::CacheEffects);
    let first = generation
        .ensure_healthy()
        .unwrap_err()
        .accounting_diagnostic();
    assert!(matches!(
        first,
        Some(RenderAccountingError::GenerationRefused {
            quantity: RenderQuantity::Work(RenderWork::ResourceLeaseReturns),
            reason: RenderRefusalReason::Overflow,
            ..
        })
    ));
    drop(registration);
    drop(value);
    for resource in [
        RenderResource::GpuBytes,
        RenderResource::RegisteredCacheGpuBytes,
        RenderResource::IdleCacheGpuBytes,
    ] {
        assert_eq!(generation.usage().unwrap().live(resource), 0);
    }
    assert_eq!(
        generation
            .ensure_healthy()
            .unwrap_err()
            .accounting_diagnostic(),
        first
    );
}

#[test]
fn one_outer_checkpoint_reuses_reserved_slot_after_cancellation_and_counts_exact_cache_work() {
    let generation = Generation::new(RenderAccountingOptions::default()).unwrap();
    let canceled = generation.begin_interval().unwrap();
    drop(canceled);
    let checkpoint = generation.begin_interval().unwrap();
    let value = generation
        .allocation(GenerationClass::CacheEffects, 16, None, || ())
        .unwrap();
    generation.reuse(GenerationClass::CacheEffects).unwrap();
    let mut registration = generation
        .cache_registration(GenerationClass::CacheEffects, 16)
        .unwrap();
    registration.set_idle(true).unwrap();
    let interval = checkpoint.finish().unwrap();
    assert_eq!(
        (
            interval.cache_allocations(),
            interval.cache_reuses(),
            interval.idle_cache_bytes()
        ),
        (1, 1, 16)
    );
    drop(registration);
    drop(value);
    drop(generation.begin_interval().unwrap());
}

#[test]
fn checkpoint_capacity_refusal_preserves_healthy_custody_and_retries_after_closing_active_slot() {
    let generation = Generation::new(RenderAccountingOptions::default()).unwrap();
    let active = generation.begin_interval().unwrap();
    assert!(generation.begin_interval().is_err());
    generation.ensure_healthy().unwrap();
    drop(active);
    drop(generation.begin_interval().unwrap());
    generation.ensure_healthy().unwrap();
}

#[test]
fn retirement_cleans_fault_injected_leaf_classification_and_preserves_first_fault() {
    let generation = Generation::new(RenderAccountingOptions::default()).unwrap();
    let native = generation
        .allocation(GenerationClass::CacheEffects, 16, None, || ())
        .unwrap();
    let registration = generation
        .cache_registration(GenerationClass::CacheEffects, 16)
        .unwrap();
    // A real leaf balance disagrees with the independent 16-byte entry contract.
    generation.adjust_registered_cache_bytes_for_test(8);
    drop(registration);
    let first = generation
        .ensure_healthy()
        .unwrap_err()
        .accounting_diagnostic()
        .unwrap();
    assert!(matches!(
        first,
        RenderAccountingError::GenerationRefused {
            quantity: RenderQuantity::Resource(RenderResource::RegisteredCacheGpuBytes),
            current: 8,
            amount: 16,
            reason: RenderRefusalReason::Underflow,
        }
    ));
    generation.retire();
    assert_eq!(
        generation.usage().unwrap().live(RenderResource::GpuBytes),
        0
    );
    assert_eq!(generation.registered_cache_bytes().unwrap(), 0);
    assert_eq!(
        generation
            .class_usage(GenerationClass::CacheEffects)
            .unwrap()
            .live(RenderResource::RegisteredCacheGpuBytes),
        0
    );
    drop(native);
    assert_eq!(
        generation
            .ensure_healthy()
            .unwrap_err()
            .accounting_diagnostic(),
        Some(first)
    );
}

#[test]
fn attempt_constructor_and_probe_work_does_not_duplicate_generation_forward_work() {
    use crate::accounting::{RenderAccountingContext, RenderAttemptKind, RenderPhase};
    let generation = Generation::new(RenderAccountingOptions::default()).unwrap();
    let mut context = RenderAccountingContext::default();
    context
        .run(RenderAttemptKind::Frame, RenderPhase::Frame, |account| {
            let allocation =
                generation.allocation(GenerationClass::CacheEffects, 12, Some(account), || ())?;
            let object = generation.opaque(GpuObjectKind::Sampler, Some(account), || ())?;
            generation.probe(GenerationClass::CacheEffects, Some(account))?;
            assert_eq!(
                generation
                    .snapshot()?
                    .usage()
                    .live(RenderResource::GpuBytes),
                12
            );
            drop(allocation);
            drop(object);
            Ok(())
        })
        .unwrap();
    let attempt = context.last_attempt().unwrap().attempt().unwrap().usage();
    assert_eq!(attempt.work(RenderWork::ResourceAllocationRequests), 2);
    assert_eq!(attempt.work(RenderWork::ResourceProbes), 1);
    let snapshot = generation.snapshot().unwrap();
    assert_eq!(
        snapshot
            .usage()
            .work(RenderWork::ResourceAllocationRequests),
        0
    );
    assert_eq!(snapshot.usage().work(RenderWork::ResourceProbes), 0);
    assert_eq!(snapshot.usage().work(RenderWork::ResourceAllocations), 1);
    assert_eq!(snapshot.usage().work(RenderWork::GpuObjectsCreated), 1);
    assert_eq!(snapshot.usage().live(RenderResource::GpuBytes), 0);
    assert_eq!(snapshot.usage().peak(RenderResource::GpuBytes), 12);
}

#[test]
fn off_and_untimed_generation_owners_never_read_the_selected_clock() {
    use crate::accounting::{RenderPhaseFilter, RenderProfile};
    fn forbidden_clock() -> std::time::Duration {
        panic!("untimed generation read its clock")
    }
    for profile in [
        RenderProfile::Off,
        RenderProfile::Summary { timing: false },
        RenderProfile::Detailed {
            timing: false,
            event_limit: 0,
            filter: RenderPhaseFilter::All,
        },
    ] {
        let generation = Generation::new(
            RenderAccountingOptions::new()
                .with_clock(RenderClock::Host(forbidden_clock))
                .with_profile(profile),
        )
        .unwrap();
        let checkpoint = generation.begin_interval().unwrap();
        let native = pollster::block_on(generation.initialization(async || {
            generation.allocation(GenerationClass::SurfaceTargets, 7, None, || ())
        }))
        .unwrap();
        let first = generation.snapshot().unwrap();
        assert_eq!(first.usage().live(RenderResource::GpuBytes), 7);
        assert_eq!(first.usage().peak(RenderResource::GpuBytes), 7);
        assert_eq!(
            first.usage().work(RenderWork::ResourceAllocationRequests),
            1
        );
        assert_eq!(first.elapsed(), None);
        drop(native);
        let interval = checkpoint.finish().unwrap();
        assert_eq!(interval.usage().peak(RenderResource::GpuBytes), 7);
        assert_eq!(interval.usage().live(RenderResource::GpuBytes), 0);
        assert_eq!(interval.elapsed(), None);
        generation.retire();
        let retired = generation.snapshot().unwrap();
        assert!(retired.retired());
        assert_eq!(retired.fault(), None);
        assert_eq!(retired.usage().work(RenderWork::ResourceRetirements), 1);
        assert_eq!(first.identity(), retired.identity());
        assert_eq!(first.usage().live(RenderResource::GpuBytes), 7);
    }
}

#[test]
fn timed_generation_initialization_includes_pending_and_real_native_leaf_phases() {
    use crate::accounting::{RenderPhaseFilter, RenderProfile};
    use std::{
        future::Future,
        sync::atomic::AtomicU64,
        task::{Context, Poll, Waker},
        time::Duration,
    };
    static NOW: AtomicU64 = AtomicU64::new(0);
    static READS: AtomicUsize = AtomicUsize::new(0);
    fn clock() -> Duration {
        READS.fetch_add(1, Ordering::SeqCst);
        Duration::from_millis(NOW.load(Ordering::SeqCst))
    }
    NOW.store(0, Ordering::SeqCst);
    READS.store(0, Ordering::SeqCst);
    let generation = Generation::new(
        RenderAccountingOptions::new()
            .with_clock(RenderClock::Host(clock))
            .with_profile(RenderProfile::Detailed {
                timing: true,
                event_limit: 0,
                filter: RenderPhaseFilter::All,
            }),
    )
    .unwrap();
    let start_reads = READS.load(Ordering::SeqCst);
    let ready = AtomicBool::new(false);
    let mut initialization = std::pin::pin!(generation.initialization(async || {
        let native = generation.opaque(GpuObjectKind::Sampler, None, || {
            NOW.store(11, Ordering::SeqCst);
        })?;
        std::future::poll_fn(|_| {
            if ready.load(Ordering::SeqCst) {
                Poll::Ready(())
            } else {
                Poll::Pending
            }
        })
        .await;
        drop(native);
        Ok(())
    }));
    assert_eq!(READS.load(Ordering::SeqCst), start_reads);
    NOW.store(5, Ordering::SeqCst);
    let mut cx = Context::from_waker(Waker::noop());
    assert!(initialization.as_mut().poll(&mut cx).is_pending());
    NOW.store(29, Ordering::SeqCst);
    ready.store(true, Ordering::SeqCst);
    assert!(matches!(
        initialization.as_mut().poll(&mut cx),
        Poll::Ready(Ok(()))
    ));
    let snapshot = generation.snapshot().unwrap();
    assert_eq!(snapshot.elapsed(), Some(Duration::from_millis(29)));
    assert_eq!(
        snapshot.scope_count(RenderGenerationScope::Initialization),
        1
    );
    assert_eq!(
        snapshot.scope_elapsed(RenderGenerationScope::Initialization),
        Some(Duration::from_millis(24))
    );
    assert_eq!(snapshot.scope_count(RenderGenerationScope::Sampler), 1);
    assert_eq!(
        snapshot.scope_elapsed(RenderGenerationScope::Sampler),
        Some(Duration::from_millis(6))
    );
    assert_eq!(snapshot.events().count(), 0);
    assert!(snapshot.dropped_events() > 0);
    assert_eq!(snapshot.usage().work(RenderWork::GpuObjectsCreated), 1);
    assert_eq!(snapshot.usage().live(RenderResource::OpaqueGpuObjects), 0);
}

#[test]
fn pure_generation_snapshots_preserve_terminal_custody_and_first_health_fault() {
    let generation = Generation::new(RenderAccountingOptions::default()).unwrap();
    let native = generation
        .allocation(GenerationClass::CacheEffects, 9, None, || ())
        .unwrap();
    let registration = generation
        .cache_registration(GenerationClass::CacheEffects, 9)
        .unwrap();
    generation.adjust_registered_cache_bytes_for_test(8);
    drop(registration);
    let first = generation
        .ensure_healthy()
        .unwrap_err()
        .accounting_diagnostic()
        .unwrap();
    let before = generation.snapshot().unwrap();
    assert!(!before.retired());
    assert_eq!(before.fault(), Some(first));
    assert_eq!(before.usage().live(RenderResource::GpuBytes), 9);
    assert_eq!(generation.snapshot().unwrap(), before);
    generation.retire();
    let retired = generation.snapshot().unwrap();
    assert!(retired.retired());
    assert_eq!(retired.fault(), Some(first));
    assert_eq!(retired.usage().live(RenderResource::GpuBytes), 0);
    drop(native);
    assert_eq!(generation.snapshot().unwrap(), retired);
    assert_eq!(before.usage().live(RenderResource::GpuBytes), 9);
    assert_ne!(
        before.identity(),
        Generation::new(RenderAccountingOptions::default())
            .unwrap()
            .snapshot()
            .unwrap()
            .identity()
    );
}

#[test]
fn generation_detail_filter_changes_only_observation_and_interval_closes_before_later_release() {
    use crate::accounting::{RenderGenerationEvent, RenderPhaseFilter, RenderProfile};
    let generation = Generation::new(RenderAccountingOptions::new().with_profile(
        RenderProfile::Detailed {
            timing: false,
            event_limit: 16,
            filter: RenderPhaseFilter::GenerationScope(RenderGenerationScope::Effects),
        },
    ))
    .unwrap();
    let checkpoint = generation.begin_interval().unwrap();
    let effect = generation
        .allocation(GenerationClass::CacheEffects, 3, None, || ())
        .unwrap();
    let target = generation
        .allocation(GenerationClass::SurfaceTargets, 7, None, || ())
        .unwrap();
    let sampler = generation
        .opaque(GpuObjectKind::Sampler, None, || ())
        .unwrap();
    let interval = checkpoint.finish().unwrap();
    assert_eq!(interval.usage().live(RenderResource::GpuBytes), 10);
    assert_eq!(interval.usage().peak(RenderResource::GpuBytes), 10);
    assert_eq!(
        interval
            .usage()
            .work(RenderWork::ResourceAllocationRequests),
        3
    );
    assert_eq!(interval.events().count(), 4);
    drop(effect);
    drop(target);
    drop(sampler);
    let snapshot = generation.snapshot().unwrap();
    assert_eq!(snapshot.usage().live(RenderResource::GpuBytes), 0);
    assert_eq!(snapshot.usage().peak(RenderResource::GpuBytes), 10);
    assert_eq!(snapshot.usage().work(RenderWork::ResourceAllocations), 2);
    assert_eq!(snapshot.usage().work(RenderWork::GpuObjectsCreated), 1);
    assert_eq!(snapshot.events().count(), 5);
    assert!(snapshot.events().all(|event| match event {
        RenderGenerationEvent::Change { scope, .. }
        | RenderGenerationEvent::Phase { scope, .. }
        | RenderGenerationEvent::Refused { scope, .. } => scope == RenderGenerationScope::Effects,
    }));
    assert_eq!(snapshot.dropped_events(), 0);
    assert_eq!(interval.usage().live(RenderResource::GpuBytes), 10);
    assert_eq!(interval.events().count(), 4);
}

#[test]
fn initialization_phase_starts_on_first_poll_and_cancellation_closes_it_once() {
    use crate::accounting::RenderProfile;
    use std::{
        future::Future,
        task::{Context, Waker},
    };
    let generation = Generation::new(
        RenderAccountingOptions::new().with_profile(RenderProfile::Summary { timing: false }),
    )
    .unwrap();
    drop(generation.initialization(async || Ok(())));
    assert_eq!(
        generation
            .snapshot()
            .unwrap()
            .scope_count(RenderGenerationScope::Initialization),
        0
    );
    let mut pending =
        Box::pin(generation.initialization(async || std::future::pending::<Result<()>>().await));
    let mut cx = Context::from_waker(Waker::noop());
    assert!(pending.as_mut().poll(&mut cx).is_pending());
    assert_eq!(
        generation
            .snapshot()
            .unwrap()
            .scope_count(RenderGenerationScope::Initialization),
        0
    );
    drop(pending);
    assert_eq!(
        generation
            .snapshot()
            .unwrap()
            .scope_count(RenderGenerationScope::Initialization),
        1
    );
    generation.ensure_healthy().unwrap();
}

#[test]
fn nested_resource_scopes_reserve_cleanup_before_sealing_and_merge_without_acquisition() {
    use crate::accounting::{
        RenderCpuEvent, RenderCpuScope, RenderPhaseFilter, RenderProfile, RendererCpu,
    };
    use crate::resource::{ResourceCacheKey, ResourceManager};
    let options = RenderAccountingOptions::new().with_profile(RenderProfile::Detailed {
        timing: false,
        event_limit: 256,
        filter: RenderPhaseFilter::All,
    });
    let cpu = RendererCpu::new(options).unwrap();
    let generation = Generation::new(options).unwrap();
    let manager = ResourceManager::with_generation(
        crate::ResourceCacheBudget::DISABLED,
        generation,
        cpu.at(RenderCpuScope::DeviceBacking),
    );
    let mut first = manager.begin_frame(None).unwrap();
    let a = first.acquire(ResourceCacheKey::VelloAtlas, 4).unwrap();
    let a_id = a.resource_identity();
    let mut second = manager.begin_frame(None).unwrap();
    let b = second.acquire(ResourceCacheKey::VelloAtlas, 8).unwrap();
    let b_id = b.resource_identity();
    first.ensure_commit_ready(&[&a]).unwrap();
    second.ensure_commit_ready(&[&b]).unwrap();
    let cleanup_interval = cpu.begin_interval().unwrap();
    first.release(a).unwrap();
    second.release(b).unwrap();
    let cleanup = first
        .finish_checked()
        .unwrap()
        .followed_by(second.finish_checked().unwrap())
        .unwrap();
    assert_eq!(cleanup.evicted_resources(), &[a_id, b_id]);
    let interval = cleanup_interval.finish().unwrap();
    assert_eq!(
        interval.usage().peak(RenderResource::ScratchCpuBytes),
        interval
            .usage()
            .opening_live(RenderResource::ScratchCpuBytes)
    );
    assert!(interval.events().all(|event| !matches!(
        event,
        RenderCpuEvent::Change {
            kind: super::super::RenderChangeKind::Acquire,
            ..
        }
    )));
    assert_eq!(manager.generation().cache_bytes().unwrap(), 0);
    drop(cleanup);
    assert_eq!(
        cpu.snapshot()
            .unwrap()
            .usage()
            .live(RenderResource::ScratchCpuBytes),
        0
    );
}

#[test]
fn initialized_native_constructor_admits_request_and_upload_atomically_before_invocation() {
    use crate::accounting::{
        RenderAccountingContext, RenderAttemptKind, RenderAttemptStatus, RenderLimits, RenderPhase,
    };
    for (request_limit, upload_limit, succeeds) in [(0, 3, false), (1, 2, false), (1, 3, true)] {
        let generation = Generation::new(RenderAccountingOptions::default()).unwrap();
        let limits = RenderLimits::new()
            .with_work_limit(RenderWork::ResourceAllocationRequests, request_limit)
            .unwrap()
            .with_work_limit(RenderWork::UploadBytes, upload_limit)
            .unwrap();
        let mut context =
            RenderAccountingContext::new(RenderAccountingOptions::new().with_limits(limits));
        let invoked = AtomicUsize::new(0);
        let result = context.run(
            RenderAttemptKind::Frame,
            RenderPhase::GpuPrepare,
            |account| {
                generation.allocation_with_upload(
                    GenerationClass::CacheKernels,
                    4,
                    3,
                    Some(account),
                    || {
                        invoked.fetch_add(1, Ordering::SeqCst);
                    },
                )
            },
        );
        let report = context.last_attempt().unwrap().attempt().unwrap();
        assert_eq!(result.is_ok(), succeeds);
        assert_eq!(invoked.load(Ordering::SeqCst), usize::from(succeeds));
        assert_eq!(
            report.status(),
            if succeeds {
                RenderAttemptStatus::Succeeded
            } else {
                RenderAttemptStatus::Refused
            }
        );
        assert_eq!(
            report.usage().work(RenderWork::ResourceAllocationRequests),
            u64::from(succeeds)
        );
        assert_eq!(
            report.usage().work(RenderWork::UploadBytes),
            if succeeds { 3 } else { 0 }
        );
        let snapshot = generation.snapshot().unwrap();
        assert_eq!(
            snapshot
                .usage()
                .work(RenderWork::ResourceAllocationRequests),
            0
        );
        assert_eq!(snapshot.usage().work(RenderWork::UploadBytes), 0);
        assert_eq!(
            snapshot.usage().work(RenderWork::ResourceAllocations),
            u64::from(succeeds)
        );
        assert_eq!(
            snapshot.usage().peak(RenderResource::GpuBytes),
            if succeeds { 4 } else { 0 }
        );
        drop(result);
        assert_eq!(
            generation
                .snapshot()
                .unwrap()
                .usage()
                .live(RenderResource::GpuBytes),
            0
        );
    }
}
