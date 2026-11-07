//! One device generation's observed native-handle custody. Descriptor units
//! include returned provisional handles, not proof of driver allocation.
use super::{
    RESOURCE_COUNT, RenderAccountingError, RenderLedger, RenderQuantity, RenderRefusalReason,
    RenderResource, RenderWork, WORK_COUNT, map_ledger_failure,
};
use crate::{Error, Result};
use std::{
    fmt,
    ops::{Deref, DerefMut},
    sync::{Arc, Mutex, MutexGuard},
};
use surgeist_ledger::{
    Change, Checkpoint, IntervalReport, Limits, Operation, Scope, Tariffs, Usage,
};

const SCOPE_COUNT: usize = 23;
const CACHE: usize = 1;
const RETIREMENT: usize = 22;
const LEAVES: [usize; 18] = [
    2, 3, 4, 5, 7, 8, 9, 11, 12, 13, 14, 15, 16, 17, 18, 19, 20, 21,
];

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum GenerationClass {
    CacheVello,
    CacheEffects,
    CacheMasks,
    CacheKernels,
    SurfaceTargets,
    PassBuffers,
    ReadbackStaging,
}
impl GenerationClass {
    const fn index(self) -> usize {
        match self {
            Self::CacheVello => 2,
            Self::CacheEffects => 3,
            Self::CacheMasks => 4,
            Self::CacheKernels => 5,
            Self::SurfaceTargets => 7,
            Self::PassBuffers => 8,
            Self::ReadbackStaging => 9,
        }
    }
    const fn is_cache(self) -> bool {
        matches!(
            self,
            Self::CacheVello | Self::CacheEffects | Self::CacheMasks | Self::CacheKernels
        )
    }
}
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum GpuObjectKind {
    ShaderModule,
    BindGroupLayout,
    PipelineLayout,
    ComputePipeline,
    RenderPipeline,
    Sampler,
    BindGroup,
    TextureView,
    CommandEncoder,
    CommandBuffer,
    #[cfg(any(
        test,
        feature = "render-window",
        all(feature = "render-web", target_arch = "wasm32")
    ))]
    DependencyUtility,
}
impl GpuObjectKind {
    const fn index(self) -> usize {
        11 + self as usize
    }
}

struct Lifecycle {
    retired: bool,
    fault: Option<RenderAccountingError>,
}
struct State {
    ledger: RenderLedger,
    scopes: [Scope; SCOPE_COUNT],
    lifecycle: Mutex<Lifecycle>,
}
#[derive(Clone)]
pub(crate) struct Generation(Arc<State>);
impl fmt::Debug for Generation {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Generation")
            .field("retired", &self.lock().retired)
            .finish_non_exhaustive()
    }
}
impl Generation {
    pub(crate) fn new() -> Result<Self> {
        let mut op = Operation::new(
            "render.device",
            Limits::<WORK_COUNT, RESOURCE_COUNT>::unlimited(),
            SCOPE_COUNT,
        )
        .map_err(|e| Error::from(map_ledger_failure(e)))?
        .with_checkpoint_limit(1)
        .map_err(|e| Error::from(map_ledger_failure(e)))?;
        let root = op.root();
        let mut scopes = [root; SCOPE_COUNT];
        let table = [
            (0, "cache"),
            (1, "vello"),
            (1, "effects"),
            (1, "masks"),
            (1, "kernels"),
            (0, "external"),
            (6, "surface_targets"),
            (6, "pass_buffers"),
            (6, "readback_staging"),
            (0, "objects"),
            (10, "shader_module"),
            (10, "bind_group_layout"),
            (10, "pipeline_layout"),
            (10, "compute_pipeline"),
            (10, "render_pipeline"),
            (10, "sampler"),
            (10, "bind_group"),
            (10, "texture_view"),
            (10, "command_encoder"),
            (10, "command_buffer"),
            (10, "dependency_utility"),
            (0, "retirement"),
        ];
        for (index, (parent, name)) in table.into_iter().enumerate() {
            scopes[index + 1] = op
                .child(
                    scopes[parent],
                    name,
                    Limits::unlimited(),
                    Tariffs::identity(),
                )
                .map_err(|e| Error::from(map_ledger_failure(e)))?;
        }
        Ok(Self(Arc::new(State {
            ledger: op.share(),
            scopes,
            lifecycle: Mutex::new(Lifecycle {
                retired: false,
                fault: None,
            }),
        })))
    }
    fn lock(&self) -> MutexGuard<'_, Lifecycle> {
        self.0
            .lifecycle
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
    }
    fn diagnostic(error: super::LedgerError) -> RenderAccountingError {
        let surgeist_ledger::Error::Refused(refusal) = error else {
            return map_ledger_failure(error);
        };
        let (quantity, amount) = match refusal.change {
            Change::Work(d, n) => (
                RenderQuantity::Work(
                    RenderWork::ALL
                        .into_iter()
                        .find(|w| w.dimension() == d)
                        .expect("fixed generation work schema"),
                ),
                n,
            ),
            Change::Acquire(d, n) | Change::Release(d, n) => (
                RenderQuantity::Resource(
                    RenderResource::ALL
                        .into_iter()
                        .find(|r| r.dimension() == d)
                        .expect("fixed generation resource schema"),
                ),
                n,
            ),
        };
        let reason = match refusal.reason {
            surgeist_ledger::RefusalReason::Limit { limit } => RenderRefusalReason::Limit { limit },
            surgeist_ledger::RefusalReason::Overflow => RenderRefusalReason::Overflow,
            surgeist_ledger::RefusalReason::Underflow => RenderRefusalReason::Underflow,
        };
        RenderAccountingError::GenerationRefused {
            quantity,
            current: refusal.current,
            amount,
            reason,
        }
    }
    fn record(&self, life: &mut Lifecycle, error: super::LedgerError) -> Error {
        let error = Self::diagnostic(error);
        life.fault.get_or_insert(error);
        error.into()
    }
    pub(crate) fn ensure_healthy(&self) -> Result<()> {
        let life = self.lock();
        if let Some(error) = life.fault {
            return Err(error.into());
        }
        if life.retired {
            return Err(RenderAccountingError::GenerationRetired.into());
        }
        Ok(())
    }
    fn apply(
        &self,
        index: usize,
        changes: &[Change<WORK_COUNT, RESOURCE_COUNT>],
        cleanup: bool,
    ) -> Result<()> {
        let mut life = self.lock();
        if life.retired {
            return if cleanup {
                Ok(())
            } else {
                Err(RenderAccountingError::GenerationRetired.into())
            };
        }
        if !cleanup && let Some(error) = life.fault {
            return Err(error.into());
        }
        self.0
            .ledger
            .apply_batch(self.0.scopes[index], changes)
            .map_err(|e| self.record(&mut life, e))
    }
    fn preview(&self, index: usize, changes: &[Change<WORK_COUNT, RESOURCE_COUNT>]) -> Result<()> {
        let life = self.lock();
        if life.retired {
            return Err(RenderAccountingError::GenerationRetired.into());
        }
        if let Some(error) = life.fault {
            return Err(error.into());
        }
        self.0
            .ledger
            .check_batch(self.0.scopes[index], changes)
            .map_err(|e| Error::from(Self::diagnostic(e)))
    }
    pub(crate) fn preview_cache_allocations(
        &self,
        bytes: impl IntoIterator<Item = u64>,
    ) -> Result<()> {
        let mut changes = Vec::new();
        for bytes in bytes {
            changes.try_reserve(3).map_err(|_| {
                Error::from(RenderAccountingError::Setup(
                    super::RenderAccountingSetupError::Allocation,
                ))
            })?;
            changes.extend([
                Change::Work(RenderWork::ResourceAllocationRequests.dimension(), 1),
                Change::Acquire(RenderResource::GpuBytes.dimension(), bytes),
                Change::Work(RenderWork::ResourceAllocations.dimension(), 1),
            ]);
        }
        self.preview(CACHE, &changes)
    }
    pub(crate) fn allocation<T>(
        &self,
        class: GenerationClass,
        bytes: u64,
        create: impl FnOnce() -> T,
    ) -> Result<OwnedGpu<T>> {
        let index = class.index();
        let resource = RenderResource::GpuBytes;
        let changes = [
            Change::Acquire(resource.dimension(), bytes),
            Change::Work(RenderWork::ResourceAllocations.dimension(), 1),
        ];
        self.preview(index, &changes)?;
        self.apply(
            index,
            &[Change::Work(
                RenderWork::ResourceAllocationRequests.dimension(),
                1,
            )],
            false,
        )?;
        let value = create(); // no lifecycle or Ledger lock spans a native call
        self.apply(index, &changes, false)?;
        Ok(OwnedGpu {
            value,
            hold: Arc::new(NativeHold {
                generation: self.clone(),
                index,
                resource,
                amount: bytes,
            }),
            parent: None,
        })
    }
    pub(crate) fn opaque<T>(
        &self,
        kind: GpuObjectKind,
        create: impl FnOnce() -> T,
    ) -> Result<OwnedGpu<T>> {
        let index = kind.index();
        let resource = RenderResource::OpaqueGpuObjects;
        let changes = [
            Change::Acquire(resource.dimension(), 1),
            Change::Work(RenderWork::GpuObjectsCreated.dimension(), 1),
        ];
        self.preview(index, &changes)?;
        self.apply(
            index,
            &[Change::Work(
                RenderWork::ResourceAllocationRequests.dimension(),
                1,
            )],
            false,
        )?;
        let value = create();
        self.apply(index, &changes, false)?;
        Ok(OwnedGpu {
            value,
            hold: Arc::new(NativeHold {
                generation: self.clone(),
                index,
                resource,
                amount: 1,
            }),
            parent: None,
        })
    }
    pub(crate) fn cache_registration(
        &self,
        class: GenerationClass,
        bytes: u64,
    ) -> Result<CacheRegistration> {
        if !class.is_cache() {
            return Err(RenderAccountingError::Invariant.into());
        }
        self.apply(
            class.index(),
            &[Change::Acquire(
                RenderResource::RegisteredCacheGpuBytes.dimension(),
                bytes,
            )],
            false,
        )?;
        Ok(CacheRegistration {
            generation: self.clone(),
            class,
            bytes,
            idle: false,
        })
    }
    pub(crate) fn reuse(&self, class: GenerationClass) -> Result<()> {
        self.apply(
            class.index(),
            &[Change::Work(RenderWork::ResourceReuses.dimension(), 1)],
            false,
        )
    }
    pub(crate) fn probe(&self, class: GenerationClass) -> Result<()> {
        self.apply(
            class.index(),
            &[Change::Work(RenderWork::ResourceProbes.dimension(), 1)],
            false,
        )
    }
    pub(crate) fn lease_return(&self, class: GenerationClass) {
        let _ = self.apply(
            class.index(),
            &[Change::Work(
                RenderWork::ResourceLeaseReturns.dimension(),
                1,
            )],
            true,
        );
    }
    pub(crate) fn eviction(&self, class: GenerationClass) {
        let _ = self.apply(
            class.index(),
            &[Change::Work(RenderWork::ResourceEvictions.dimension(), 1)],
            true,
        );
    }
    fn usage_at(&self, index: usize) -> Result<GenerationUsage> {
        let mut life = self.lock();
        self.0
            .ledger
            .usage(self.0.scopes[index])
            .map(GenerationUsage)
            .map_err(|e| self.record(&mut life, e))
    }
    #[cfg(test)]
    pub(crate) fn usage(&self) -> Result<GenerationUsage> {
        self.usage_at(0)
    }
    #[cfg(test)]
    pub(crate) fn class_usage(&self, class: GenerationClass) -> Result<GenerationUsage> {
        self.usage_at(class.index())
    }
    pub(crate) fn cache_bytes(&self) -> Result<u64> {
        Ok(self.usage_at(CACHE)?.live(RenderResource::GpuBytes))
    }
    pub(crate) fn registered_cache_bytes(&self) -> Result<u64> {
        Ok(self
            .usage_at(CACHE)?
            .live(RenderResource::RegisteredCacheGpuBytes))
    }
    #[cfg(test)]
    pub(crate) fn idle_cache_bytes(&self) -> Result<u64> {
        Ok(self
            .usage_at(CACHE)?
            .live(RenderResource::IdleCacheGpuBytes))
    }
    #[cfg(test)]
    pub(crate) fn cache_work(&self, work: RenderWork) -> Result<u64> {
        Ok(self.usage_at(CACHE)?.work(work))
    }
    pub(crate) fn retire(&self) {
        let mut life = self.lock();
        if life.retired {
            return;
        }
        // All physical and classification holds originate at these fixed leaves.
        for index in LEAVES {
            match self.0.ledger.usage(self.0.scopes[index]) {
                Ok(usage) => {
                    for resource in [
                        RenderResource::GpuBytes,
                        RenderResource::OpaqueGpuObjects,
                        RenderResource::RegisteredCacheGpuBytes,
                        RenderResource::IdleCacheGpuBytes,
                    ] {
                        if let Err(e) = self.0.ledger.apply(
                            self.0.scopes[index],
                            Change::Release(resource.dimension(), usage.live(resource.dimension())),
                        ) {
                            self.record(&mut life, e);
                        }
                    }
                }
                Err(e) => {
                    self.record(&mut life, e);
                }
            }
        }
        life.retired = true;
        // Cleanup work cannot veto the releases above.
        if let Err(e) = self.0.ledger.apply(
            self.0.scopes[RETIREMENT],
            Change::Work(RenderWork::ResourceRetirements.dimension(), 1),
        ) {
            self.record(&mut life, e);
        }
    }
    pub(crate) fn begin_interval(&self) -> Result<GenerationCheckpoint> {
        self.ensure_healthy()?;
        let checkpoint = self
            .0
            .ledger
            .begin_checkpoint()
            .map_err(|e| Error::from(map_ledger_failure(e)))?;
        Ok(GenerationCheckpoint {
            generation: self.clone(),
            checkpoint: Some(checkpoint),
        })
    }
    #[cfg(test)]
    pub(crate) fn adjust_registered_cache_bytes_for_test(&self, value: u64) {
        let resource = RenderResource::RegisteredCacheGpuBytes.dimension();
        let current = self.registered_cache_bytes().unwrap();
        if value >= current {
            let index = [2, 3, 4, 5]
                .into_iter()
                .find(|i| {
                    self.0
                        .ledger
                        .usage(self.0.scopes[*i])
                        .unwrap()
                        .live(resource)
                        > 0
                })
                .unwrap_or(3);
            self.0
                .ledger
                .apply(
                    self.0.scopes[index],
                    Change::Acquire(resource, value - current),
                )
                .unwrap();
        } else {
            let mut remaining = current - value;
            for index in [2, 3, 4, 5] {
                let live = self
                    .0
                    .ledger
                    .usage(self.0.scopes[index])
                    .unwrap()
                    .live(resource);
                let amount = live.min(remaining);
                self.0
                    .ledger
                    .apply(self.0.scopes[index], Change::Release(resource, amount))
                    .unwrap();
                remaining -= amount;
            }
            assert_eq!(
                remaining, 0,
                "all registration holds originate in cache leaves"
            );
        }
    }
    #[cfg(test)]
    pub(crate) fn observe_registered_cache_bytes_for_test(&self) -> u64 {
        self.registered_cache_bytes().unwrap()
    }
}

/// Native value drops before its shared hold. Raw native clones must not escape.
pub(crate) struct OwnedGpu<T> {
    value: T,
    hold: Arc<NativeHold>,
    parent: Option<Arc<NativeHold>>,
}
impl<T: Clone> Clone for OwnedGpu<T> {
    fn clone(&self) -> Self {
        Self {
            value: self.value.clone(),
            hold: self.hold.clone(),
            parent: self.parent.clone(),
        }
    }
}
impl<T: fmt::Debug> fmt::Debug for OwnedGpu<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.value.fmt(f)
    }
}
impl<T> Deref for OwnedGpu<T> {
    type Target = T;
    fn deref(&self) -> &T {
        &self.value
    }
}
impl<T> OwnedGpu<T> {
    pub(crate) fn generation(&self) -> &Generation {
        &self.hold.generation
    }
}
struct NativeHold {
    generation: Generation,
    index: usize,
    resource: RenderResource,
    amount: u64,
}
impl Drop for NativeHold {
    fn drop(&mut self) {
        let _ = self.generation.apply(
            self.index,
            &[Change::Release(self.resource.dimension(), self.amount)],
            true,
        );
    }
}
impl OwnedGpu<wgpu::Texture> {
    pub(crate) fn create_view(
        &self,
        desc: &wgpu::TextureViewDescriptor<'_>,
    ) -> Result<OwnedGpu<wgpu::TextureView>> {
        {
            let mut view = self
                .generation()
                .opaque(GpuObjectKind::TextureView, || self.value.create_view(desc))?;
            view.parent = Some(self.hold.clone());
            Ok(view)
        }
    }
}
impl DerefMut for OwnedGpu<wgpu::CommandEncoder> {
    fn deref_mut(&mut self) -> &mut wgpu::CommandEncoder {
        &mut self.value
    }
}
impl OwnedGpu<wgpu::CommandEncoder> {
    pub(crate) fn finish(self) -> Result<OwnedGpu<wgpu::CommandBuffer>> {
        let generation = self.generation().clone();
        generation.opaque(GpuObjectKind::CommandBuffer, || {
            let Self {
                value,
                hold,
                parent,
            } = self;
            let value = value.finish();
            drop(hold);
            drop(parent);
            value
        })
    }
}
impl OwnedGpu<wgpu::CommandBuffer> {
    pub(crate) fn submit(self, queue: &wgpu::Queue) -> wgpu::SubmissionIndex {
        let Self {
            value,
            hold,
            parent,
        } = self;
        let submission = queue.submit([value]);
        drop(hold);
        drop(parent);
        submission
    }
}

pub(crate) struct CacheRegistration {
    generation: Generation,
    class: GenerationClass,
    bytes: u64,
    idle: bool,
}
impl CacheRegistration {
    pub(crate) fn set_idle(&mut self, idle: bool) -> Result<()> {
        if idle == self.idle {
            return Ok(());
        }
        let dimension = RenderResource::IdleCacheGpuBytes.dimension();
        let change = if idle {
            Change::Acquire(dimension, self.bytes)
        } else {
            Change::Release(dimension, self.bytes)
        };
        self.generation
            .apply(self.class.index(), &[change], !idle)?;
        self.idle = idle;
        Ok(())
    }
}
impl Drop for CacheRegistration {
    fn drop(&mut self) {
        if self.idle {
            let _ = self.generation.apply(
                self.class.index(),
                &[Change::Release(
                    RenderResource::IdleCacheGpuBytes.dimension(),
                    self.bytes,
                )],
                true,
            );
        }
        let _ = self.generation.apply(
            self.class.index(),
            &[Change::Release(
                RenderResource::RegisteredCacheGpuBytes.dimension(),
                self.bytes,
            )],
            true,
        );
    }
}
pub(crate) struct GenerationUsage(Usage<WORK_COUNT, RESOURCE_COUNT>);
impl GenerationUsage {
    #[cfg(test)]
    pub(crate) fn work(&self, work: RenderWork) -> u64 {
        self.0.work(work.dimension())
    }
    pub(crate) fn live(&self, resource: RenderResource) -> u64 {
        self.0.live(resource.dimension())
    }
    #[cfg(test)]
    pub(crate) fn peak(&self, resource: RenderResource) -> u64 {
        self.0.peak(resource.dimension())
    }
}
pub(crate) struct GenerationCheckpoint {
    generation: Generation,
    checkpoint: Option<Checkpoint>,
}
impl GenerationCheckpoint {
    pub(crate) fn finish(mut self) -> Result<GenerationInterval> {
        let checkpoint = self
            .checkpoint
            .take()
            .ok_or(RenderAccountingError::Invariant)?;
        self.generation
            .0
            .ledger
            .end_checkpoint(&checkpoint)
            .map(GenerationInterval)
            .map_err(|e| {
                let mut life = self.generation.lock();
                self.generation.record(&mut life, e)
            })
    }
}
impl Drop for GenerationCheckpoint {
    fn drop(&mut self) {
        if let Some(checkpoint) = self.checkpoint.take()
            && let Err(e) = self.generation.0.ledger.end_checkpoint(&checkpoint)
        {
            let mut life = self.generation.lock();
            self.generation.record(&mut life, e);
        }
    }
}
pub(crate) struct GenerationInterval(IntervalReport<WORK_COUNT, RESOURCE_COUNT>);
impl GenerationInterval {
    pub(crate) fn cache_allocations(&self) -> u64 {
        self.0.scopes[CACHE]
            .usage
            .work(RenderWork::ResourceAllocations.dimension())
    }
    pub(crate) fn cache_reuses(&self) -> u64 {
        self.0.scopes[CACHE]
            .usage
            .work(RenderWork::ResourceReuses.dimension())
    }
    pub(crate) fn idle_cache_bytes(&self) -> u64 {
        self.0.scopes[CACHE]
            .usage
            .live(RenderResource::IdleCacheGpuBytes.dimension())
    }
}

#[cfg(test)]
mod tests;
