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
    let generation = Generation::new().unwrap();
    let drops = Arc::new(AtomicUsize::new(0));
    let value = generation
        .allocation(GenerationClass::CacheEffects, 16, || {
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
    let generation = Generation::new().unwrap();
    let first = generation
        .allocation(GenerationClass::CacheEffects, 16, || ())
        .unwrap();
    let second = generation
        .allocation(GenerationClass::SurfaceTargets, 8, || ())
        .unwrap();
    assert_eq!(
        generation.usage().unwrap().peak(RenderResource::GpuBytes),
        24
    );
    drop(second);
    let third = generation
        .allocation(GenerationClass::PassBuffers, 4, || ())
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
    let generation = Generation::new().unwrap();
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
        .map(|(i, class)| generation.allocation(class, (i + 1) as u64, || ()).unwrap())
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
        .map(|kind| generation.opaque(kind, || ()).unwrap())
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
            .opaque(GpuObjectKind::Sampler, || invoked
                .store(true, Ordering::SeqCst))
            .is_err()
    );
    assert!(!invoked.load(Ordering::SeqCst));
}

#[test]
fn retirement_during_native_creation_disposes_unpublished_handle_without_admission() {
    struct Dropped(Arc<AtomicUsize>);
    impl Drop for Dropped {
        fn drop(&mut self) {
            self.0.fetch_add(1, Ordering::SeqCst);
        }
    }
    let generation = Generation::new().unwrap();
    let drops = Arc::new(AtomicUsize::new(0));
    let result = generation.opaque(GpuObjectKind::ShaderModule, || {
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
    let generation = Generation::new().unwrap();
    let existing = generation
        .allocation(GenerationClass::CacheEffects, u64::MAX, || ())
        .unwrap();
    let invoked = AtomicBool::new(false);
    let error = generation
        .allocation(GenerationClass::SurfaceTargets, 1, || {
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
            .allocation(GenerationClass::SurfaceTargets, 4, || ())
            .unwrap(),
    );
}

#[test]
fn lifecycle_work_overflow_records_first_fault_after_mandatory_custody_cleanup() {
    let generation = Generation::new().unwrap();
    let value = generation
        .allocation(GenerationClass::CacheEffects, 16, || ())
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
            Change::Work(RenderWork::ResourceLeaseReturns.dimension(), u64::MAX),
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
    let generation = Generation::new().unwrap();
    let canceled = generation.begin_interval().unwrap();
    drop(canceled);
    let checkpoint = generation.begin_interval().unwrap();
    let value = generation
        .allocation(GenerationClass::CacheEffects, 16, || ())
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
    let generation = Generation::new().unwrap();
    let active = generation.begin_interval().unwrap();
    assert!(generation.begin_interval().is_err());
    generation.ensure_healthy().unwrap();
    drop(active);
    drop(generation.begin_interval().unwrap());
    generation.ensure_healthy().unwrap();
}

#[test]
fn retirement_cleans_fault_injected_leaf_classification_and_preserves_first_fault() {
    let generation = Generation::new().unwrap();
    let native = generation
        .allocation(GenerationClass::CacheEffects, 16, || ())
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
