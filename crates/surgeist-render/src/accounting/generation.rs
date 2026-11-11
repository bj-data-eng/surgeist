//! One device generation's observed native-handle custody. Descriptor units
//! include returned provisional handles, not proof of driver allocation.
use super::{
    Account, IntervalNumbers, ObserverBudget, RenderAccountingError, RenderAccountingOptions,
    RenderClock, RenderDeviceSnapshot, RenderGenerationIdentity, RenderGenerationInterval,
    RenderGenerationScope, RenderLedger, RenderLifetimeUsage, RenderQuantity, RenderRefusalReason,
    RenderResource, RenderWork, ReportBacking, map_ledger_failure,
};
use crate::{Error, Result};
use std::{
    fmt,
    ops::{Deref, DerefMut},
    sync::{Arc, Mutex, MutexGuard},
};
use surgeist_ledger::{Change, Checkpoint, Limits, Operation, Scope, Tariffs, Usage};

const SCOPE_COUNT: usize = RenderGenerationScope::ALL.len();
const RETIREMENT_LEAVES: [RenderGenerationScope; 18] = [
    RenderGenerationScope::Vello,
    RenderGenerationScope::Effects,
    RenderGenerationScope::Masks,
    RenderGenerationScope::Kernels,
    RenderGenerationScope::SurfaceTargets,
    RenderGenerationScope::PassBuffers,
    RenderGenerationScope::ReadbackStaging,
    RenderGenerationScope::ShaderModule,
    RenderGenerationScope::BindGroupLayout,
    RenderGenerationScope::PipelineLayout,
    RenderGenerationScope::ComputePipeline,
    RenderGenerationScope::RenderPipeline,
    RenderGenerationScope::Sampler,
    RenderGenerationScope::BindGroup,
    RenderGenerationScope::TextureView,
    RenderGenerationScope::CommandEncoder,
    RenderGenerationScope::CommandBuffer,
    RenderGenerationScope::DependencyUtility,
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
    const fn scope(self) -> RenderGenerationScope {
        match self {
            Self::CacheVello => RenderGenerationScope::Vello,
            Self::CacheEffects => RenderGenerationScope::Effects,
            Self::CacheMasks => RenderGenerationScope::Masks,
            Self::CacheKernels => RenderGenerationScope::Kernels,
            Self::SurfaceTargets => RenderGenerationScope::SurfaceTargets,
            Self::PassBuffers => RenderGenerationScope::PassBuffers,
            Self::ReadbackStaging => RenderGenerationScope::ReadbackStaging,
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
    const fn scope(self) -> RenderGenerationScope {
        match self {
            Self::ShaderModule => RenderGenerationScope::ShaderModule,
            Self::BindGroupLayout => RenderGenerationScope::BindGroupLayout,
            Self::PipelineLayout => RenderGenerationScope::PipelineLayout,
            Self::ComputePipeline => RenderGenerationScope::ComputePipeline,
            Self::RenderPipeline => RenderGenerationScope::RenderPipeline,
            Self::Sampler => RenderGenerationScope::Sampler,
            Self::BindGroup => RenderGenerationScope::BindGroup,
            Self::TextureView => RenderGenerationScope::TextureView,
            Self::CommandEncoder => RenderGenerationScope::CommandEncoder,
            Self::CommandBuffer => RenderGenerationScope::CommandBuffer,
            #[cfg(any(
                test,
                feature = "render-window",
                all(feature = "render-web", target_arch = "wasm32")
            ))]
            Self::DependencyUtility => RenderGenerationScope::DependencyUtility,
        }
    }
}

struct Lifecycle {
    retired: bool,
    fault: Option<RenderAccountingError>,
}
struct State {
    identity: RenderGenerationIdentity,
    ledger: RenderLedger,
    scopes: [Scope; SCOPE_COUNT],
    observer_budget: ObserverBudget,
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
    pub(crate) fn new(options: RenderAccountingOptions) -> Result<Self> {
        let mut op = match options.clock {
            RenderClock::Platform => Operation::with_profile(
                "render.device",
                Limits::<RenderWork, RenderResource>::unlimited(),
                SCOPE_COUNT,
                options.profile.ledger_generation(),
            ),
            RenderClock::Host(clock) => Operation::with_profile_clock(
                "render.device",
                Limits::<RenderWork, RenderResource>::unlimited(),
                SCOPE_COUNT,
                options.profile.ledger_generation(),
                clock,
            ),
        }
        .map_err(|e| Error::from(map_ledger_failure(e)))?
        .with_checkpoint_limit(1)
        .map_err(|e| Error::from(map_ledger_failure(e)))?;
        let root = op.root();
        let mut scopes = [root; SCOPE_COUNT];
        for scope in RenderGenerationScope::ALL.into_iter().skip(1) {
            let parent = scope.parent().ok_or(RenderAccountingError::Invariant)?;
            let parent_scope = *scopes
                .get(parent.index())
                .ok_or(RenderAccountingError::Invariant)?;
            if parent != RenderGenerationScope::Root && parent_scope == root {
                return Err(RenderAccountingError::Invariant.into());
            }
            scopes[scope.index()] = op
                .child(
                    parent_scope,
                    scope.name(),
                    Limits::unlimited(),
                    Tariffs::identity(),
                )
                .map_err(|error| Error::from(map_ledger_failure(error)))?;
        }
        Ok(Self(Arc::new(State {
            identity: RenderGenerationIdentity::new(),
            ledger: op.share(),
            scopes,
            observer_budget: ObserverBudget::new(options.profile.ledger_generation()),
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
            Change::Work(d, n) => (RenderQuantity::Work(d), n),
            Change::Acquire(d, n) | Change::Release(d, n) => (RenderQuantity::Resource(d), n),
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
        scope: RenderGenerationScope,
        changes: &[Change<RenderWork, RenderResource>],
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
            .apply_batch(self.0.scopes[scope.index()], changes)
            .map_err(|e| self.record(&mut life, e))
    }
    fn preview(
        &self,
        scope: RenderGenerationScope,
        changes: &[Change<RenderWork, RenderResource>],
    ) -> Result<()> {
        let life = self.lock();
        if life.retired {
            return Err(RenderAccountingError::GenerationRetired.into());
        }
        if let Some(error) = life.fault {
            return Err(error.into());
        }
        self.0
            .ledger
            .check_batch(self.0.scopes[scope.index()], changes)
            .map_err(|e| Error::from(Self::diagnostic(e)))
    }
    pub(crate) fn preview_cache_allocations(
        &self,
        bytes: impl IntoIterator<Item = u64>,
        observer: Option<&Account>,
    ) -> Result<()> {
        let mut total_bytes = 0u64;
        let mut count = 0u64;
        for bytes in bytes {
            total_bytes = total_bytes.checked_add(bytes).ok_or_else(|| {
                Error::from(RenderAccountingError::GenerationRefused {
                    quantity: RenderQuantity::Resource(RenderResource::GpuBytes),
                    current: total_bytes,
                    amount: bytes,
                    reason: RenderRefusalReason::Overflow,
                })
            })?;
            count = count
                .checked_add(1)
                .ok_or(RenderAccountingError::Invariant)?;
        }
        let changes = [
            Change::Acquire(RenderResource::GpuBytes, total_bytes),
            Change::Work(RenderWork::ResourceAllocations, count),
            Change::Work(
                RenderWork::ResourceAllocationRequests,
                if observer.is_none() { count } else { 0 },
            ),
        ];
        self.preview(RenderGenerationScope::Cache, &changes)
    }

    pub(crate) fn allocation<T>(
        &self,
        class: GenerationClass,
        bytes: u64,
        observer: Option<&Account>,
        create: impl FnOnce() -> T,
    ) -> Result<OwnedGpu<T>> {
        self.allocation_inner(class, bytes, None, observer, create)
    }
    /// The native helper constructs and copies the initialized payload in one call.
    /// Admit both actual forward units atomically before entering that call.
    pub(crate) fn allocation_with_upload<T>(
        &self,
        class: GenerationClass,
        bytes: u64,
        upload_bytes: usize,
        observer: Option<&Account>,
        create: impl FnOnce() -> T,
    ) -> Result<OwnedGpu<T>> {
        self.allocation_inner(
            class,
            bytes,
            Some(super::as_u64(upload_bytes)?),
            observer,
            create,
        )
    }
    fn allocation_inner<T>(
        &self,
        class: GenerationClass,
        bytes: u64,
        upload_bytes: Option<u64>,
        observer: Option<&Account>,
        create: impl FnOnce() -> T,
    ) -> Result<OwnedGpu<T>> {
        let scope = class.scope();
        let resource = RenderResource::GpuBytes;
        let changes = [
            Change::Acquire(resource, bytes),
            Change::Work(RenderWork::ResourceAllocations, 1),
        ];
        self.preview(scope, &changes)?;
        if let Some(upload_bytes) = upload_bytes {
            if let Some(observer) = observer {
                observer.work_batch([
                    (RenderWork::ResourceAllocationRequests, 1),
                    (RenderWork::UploadBytes, upload_bytes),
                ])?;
            } else {
                self.apply(
                    scope,
                    &[
                        Change::Work(RenderWork::ResourceAllocationRequests, 1),
                        Change::Work(RenderWork::UploadBytes, upload_bytes),
                    ],
                    false,
                )?;
            }
        } else {
            self.forward_work(scope, observer, RenderWork::ResourceAllocationRequests, 1)?;
        }
        let value = self.phase_at(scope, create)?; // native calls run unlocked
        self.apply(scope, &changes, false)?;
        Ok(OwnedGpu {
            value,
            hold: Arc::new(NativeHold {
                generation: self.clone(),
                scope,
                resource,
                amount: bytes,
            }),
            parent: None,
        })
    }
    pub(crate) fn opaque<T>(
        &self,
        kind: GpuObjectKind,
        observer: Option<&Account>,
        create: impl FnOnce() -> T,
    ) -> Result<OwnedGpu<T>> {
        let scope = kind.scope();
        let resource = RenderResource::OpaqueGpuObjects;
        let changes = [
            Change::Acquire(resource, 1),
            Change::Work(RenderWork::GpuObjectsCreated, 1),
        ];
        self.preview(scope, &changes)?;
        self.forward_work(scope, observer, RenderWork::ResourceAllocationRequests, 1)?;
        let value = self.phase_at(scope, create)?;
        self.apply(scope, &changes, false)?;
        Ok(OwnedGpu {
            value,
            hold: Arc::new(NativeHold {
                generation: self.clone(),
                scope,
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
            class.scope(),
            &[Change::Acquire(
                RenderResource::RegisteredCacheGpuBytes,
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
            class.scope(),
            &[Change::Work(RenderWork::ResourceReuses, 1)],
            false,
        )
    }
    pub(crate) fn probe(&self, class: GenerationClass, observer: Option<&Account>) -> Result<()> {
        if observer.is_some() {
            self.ensure_healthy()?;
        }
        match observer {
            Some(observer) => observer.work(RenderWork::ResourceProbes, 1),
            None => {
                let _ = self.apply(
                    class.scope(),
                    &[Change::Work(RenderWork::ResourceProbes, 1)],
                    true,
                );
                Ok(())
            }
        }
    }
    pub(crate) fn upload(
        &self,
        class: GenerationClass,
        observer: Option<&Account>,
        bytes: usize,
    ) -> Result<()> {
        self.ensure_healthy()?;
        self.forward_work(class.scope(), observer, RenderWork::UploadBytes, bytes)
    }
    pub(crate) fn probe_cache(&self, observer: Option<&Account>) -> Result<()> {
        if observer.is_some() {
            self.ensure_healthy()?;
        }
        match observer {
            Some(observer) => observer.work(RenderWork::ResourceProbes, 1),
            None => {
                let _ = self.apply(
                    RenderGenerationScope::Cache,
                    &[Change::Work(RenderWork::ResourceProbes, 1)],
                    true,
                );
                Ok(())
            }
        }
    }
    fn forward_work(
        &self,
        scope: RenderGenerationScope,
        observer: Option<&Account>,
        work: RenderWork,
        amount: usize,
    ) -> Result<()> {
        if let Some(observer) = observer {
            observer.work(work, amount)
        } else {
            self.apply(scope, &[Change::Work(work, super::as_u64(amount)?)], false)
        }
    }
    fn phase_at<T>(&self, scope: RenderGenerationScope, run: impl FnOnce() -> T) -> Result<T> {
        self.0
            .ledger
            .with_phase(self.0.scopes[scope.index()], run)
            .map_err(|e| {
                let mut life = self.lock();
                self.record(&mut life, e)
            })
    }
    pub(crate) async fn initialization<T>(
        &self,
        run: impl std::ops::AsyncFnOnce() -> Result<T>,
    ) -> Result<T> {
        self.ensure_healthy()?;
        self.0
            .ledger
            .with_async_phase(
                self.0.scopes[RenderGenerationScope::Initialization.index()],
                run,
            )
            .await
            .map_err(|e| {
                let mut life = self.lock();
                self.record(&mut life, e)
            })?
    }
    /// Pure observation: neither observes a native signal nor retires custody.
    pub(crate) fn snapshot(&self) -> Result<RenderDeviceSnapshot> {
        let life = self.lock();
        let backing = ReportBacking::snapshot(self.0.observer_budget)?;
        let profile = self
            .0
            .ledger
            .snapshot()
            .map_err(|e| Error::from(map_ledger_failure(e)))?;
        let usage = match &profile {
            Some(profile) => profile
                .scopes
                .first()
                .map(|root| root.usage)
                .ok_or(RenderAccountingError::Invariant)?,
            None => self
                .0
                .ledger
                .usage(self.0.scopes[RenderGenerationScope::Root.index()])
                .map_err(|e| Error::from(map_ledger_failure(e)))?,
        };
        Ok(RenderDeviceSnapshot::new(
            self.0.identity.clone(),
            RenderLifetimeUsage::new(usage),
            profile,
            life.retired,
            life.fault,
            &self.0.scopes,
            backing,
        )?)
    }
    pub(crate) fn lease_return(&self, class: GenerationClass) {
        let _ = self.apply(
            class.scope(),
            &[Change::Work(RenderWork::ResourceLeaseReturns, 1)],
            true,
        );
    }
    pub(crate) fn eviction(&self, class: GenerationClass) {
        let _ = self.apply(
            class.scope(),
            &[Change::Work(RenderWork::ResourceEvictions, 1)],
            true,
        );
    }
    fn usage_at(&self, scope: RenderGenerationScope) -> Result<GenerationUsage> {
        let mut life = self.lock();
        self.0
            .ledger
            .usage(self.0.scopes[scope.index()])
            .map(GenerationUsage)
            .map_err(|e| self.record(&mut life, e))
    }
    #[cfg(test)]
    pub(crate) fn usage(&self) -> Result<GenerationUsage> {
        self.usage_at(RenderGenerationScope::Root)
    }
    #[cfg(test)]
    pub(crate) fn class_usage(&self, class: GenerationClass) -> Result<GenerationUsage> {
        self.usage_at(class.scope())
    }
    pub(crate) fn cache_bytes(&self) -> Result<u64> {
        Ok(self
            .usage_at(RenderGenerationScope::Cache)?
            .live(RenderResource::GpuBytes))
    }
    pub(crate) fn registered_cache_bytes(&self) -> Result<u64> {
        Ok(self
            .usage_at(RenderGenerationScope::Cache)?
            .live(RenderResource::RegisteredCacheGpuBytes))
    }
    #[cfg(test)]
    pub(crate) fn idle_cache_bytes(&self) -> Result<u64> {
        Ok(self
            .usage_at(RenderGenerationScope::Cache)?
            .live(RenderResource::IdleCacheGpuBytes))
    }
    #[cfg(test)]
    pub(crate) fn cache_work(&self, work: RenderWork) -> Result<u64> {
        Ok(self.usage_at(RenderGenerationScope::Cache)?.work(work))
    }
    pub(crate) fn retire(&self) {
        let mut life = self.lock();
        if life.retired {
            return;
        }
        if let Err(error) = self.0.ledger.with_phase(
            self.0.scopes[RenderGenerationScope::Retirement.index()],
            || self.retire_locked(&mut life),
        ) {
            self.record(&mut life, error);
        }
    }
    fn retire_locked(&self, life: &mut Lifecycle) {
        // All physical and classification holds originate at these fixed leaves.
        for leaf in RETIREMENT_LEAVES {
            let scope = self.0.scopes[leaf.index()];
            match self.0.ledger.usage(scope) {
                Ok(usage) => {
                    for resource in [
                        RenderResource::GpuBytes,
                        RenderResource::OpaqueGpuObjects,
                        RenderResource::RegisteredCacheGpuBytes,
                        RenderResource::IdleCacheGpuBytes,
                    ] {
                        if let Err(e) = self
                            .0
                            .ledger
                            .apply(scope, Change::Release(resource, usage.live(resource)))
                        {
                            self.record(life, e);
                        }
                    }
                }
                Err(e) => {
                    self.record(life, e);
                }
            }
        }
        life.retired = true;
        // Cleanup work cannot veto the releases above.
        if let Err(e) = self.0.ledger.apply(
            self.0.scopes[RenderGenerationScope::Retirement.index()],
            Change::Work(RenderWork::ResourceRetirements, 1),
        ) {
            self.record(life, e);
        }
    }
    pub(crate) fn begin_interval(&self) -> Result<GenerationCheckpoint> {
        self.ensure_healthy()?;
        let report_backing = ReportBacking::interval(self.0.observer_budget)?;
        let checkpoint = self
            .0
            .ledger
            .begin_checkpoint()
            .map_err(|e| Error::from(map_ledger_failure(e)))?;
        Ok(GenerationCheckpoint {
            generation: self.clone(),
            checkpoint: Some(checkpoint),
            report_backing: Some(report_backing),
        })
    }
    #[cfg(test)]
    pub(crate) fn adjust_registered_cache_bytes_for_test(&self, value: u64) {
        let resource = RenderResource::RegisteredCacheGpuBytes;
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
    scope: RenderGenerationScope,
    resource: RenderResource,
    amount: u64,
}
impl Drop for NativeHold {
    fn drop(&mut self) {
        let _ = self.generation.apply(
            self.scope,
            &[Change::Release(self.resource, self.amount)],
            true,
        );
    }
}
impl OwnedGpu<wgpu::Texture> {
    pub(crate) fn create_view(
        &self,
        desc: &wgpu::TextureViewDescriptor<'_>,
        observer: Option<&Account>,
    ) -> Result<OwnedGpu<wgpu::TextureView>> {
        {
            let mut view =
                self.generation()
                    .opaque(GpuObjectKind::TextureView, observer, || {
                        self.value.create_view(desc)
                    })?;
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
    pub(crate) fn finish(
        self,
        observer: Option<&Account>,
    ) -> Result<OwnedGpu<wgpu::CommandBuffer>> {
        let generation = self.generation().clone();
        generation.opaque(GpuObjectKind::CommandBuffer, observer, || {
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
    pub(crate) fn submit(
        self,
        queue: &wgpu::Queue,
        observer: Option<&Account>,
    ) -> Result<wgpu::SubmissionIndex> {
        let generation = self.generation().clone();
        generation.ensure_healthy()?;
        let scope = GpuObjectKind::CommandBuffer.scope();
        generation.forward_work(scope, observer, RenderWork::Submissions, 1)?;
        let Self {
            value,
            hold,
            parent,
        } = self;
        let submission = generation.phase_at(scope, || queue.submit([value]))?;
        drop(hold);
        drop(parent);
        Ok(submission)
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
        let dimension = RenderResource::IdleCacheGpuBytes;
        let change = if idle {
            Change::Acquire(dimension, self.bytes)
        } else {
            Change::Release(dimension, self.bytes)
        };
        self.generation
            .apply(self.class.scope(), &[change], !idle)?;
        self.idle = idle;
        Ok(())
    }
}
impl Drop for CacheRegistration {
    fn drop(&mut self) {
        if self.idle {
            let _ = self.generation.apply(
                self.class.scope(),
                &[Change::Release(
                    RenderResource::IdleCacheGpuBytes,
                    self.bytes,
                )],
                true,
            );
        }
        let _ = self.generation.apply(
            self.class.scope(),
            &[Change::Release(
                RenderResource::RegisteredCacheGpuBytes,
                self.bytes,
            )],
            true,
        );
    }
}
pub(crate) struct GenerationUsage(Usage<RenderWork, RenderResource>);
impl GenerationUsage {
    #[cfg(test)]
    pub(crate) fn work(&self, work: RenderWork) -> u64 {
        self.0.work(work)
    }
    pub(crate) fn live(&self, resource: RenderResource) -> u64 {
        self.0.live(resource)
    }
    #[cfg(test)]
    pub(crate) fn peak(&self, resource: RenderResource) -> u64 {
        self.0.peak(resource)
    }
}
pub(crate) struct GenerationCheckpoint {
    generation: Generation,
    checkpoint: Option<Checkpoint>,
    report_backing: Option<ReportBacking<RenderGenerationScope, IntervalNumbers>>,
}
impl GenerationCheckpoint {
    pub(crate) fn finish(mut self) -> Result<RenderGenerationInterval> {
        let checkpoint = self
            .checkpoint
            .take()
            .ok_or(RenderAccountingError::Invariant)?;
        self.generation
            .0
            .ledger
            .end_checkpoint(&checkpoint)
            .map_err(|e| {
                let mut life = self.generation.lock();
                self.generation.record(&mut life, e)
            })
            .and_then(|interval| {
                Ok(RenderGenerationInterval::new(
                    self.generation.0.identity.clone(),
                    interval,
                    &self.generation.0.scopes,
                    self.report_backing
                        .take()
                        .ok_or(RenderAccountingError::Invariant)?,
                )?)
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

#[cfg(test)]
mod tests;
