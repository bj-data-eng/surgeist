//! Typed observations of one retained native device-generation owner.
use super::report::{IntervalNumbers, LifetimeNumbers, ReportBacking, ResolvedReport};
use super::{
    RenderAccountingError, RenderChangeKind, RenderLifetimeUsage, RenderQuantity, RenderResource,
    RenderUsage, RenderWork,
};
use std::{fmt, sync::Arc, time::Duration};
use surgeist_ledger::{IntervalReport, Report, Scope};

/// Identity of one generation's accounting owner, independent of native health.
#[derive(Clone)]
pub struct RenderGenerationIdentity(Arc<()>);
impl RenderGenerationIdentity {
    pub(crate) fn new() -> Self {
        Self(Arc::new(()))
    }
}
impl PartialEq for RenderGenerationIdentity {
    fn eq(&self, other: &Self) -> bool {
        Arc::ptr_eq(&self.0, &other.0)
    }
}
impl Eq for RenderGenerationIdentity {}
impl fmt::Debug for RenderGenerationIdentity {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("RenderGenerationIdentity(..)")
    }
}

/// Fixed inclusive attribution within a device generation. Ancestor and child
/// usage and elapsed durations are not additive.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum RenderGenerationScope {
    /// All generation-owned work and native custody.
    Root,
    /// Managed reusable descriptor custody.
    Cache,
    /// Internal raster resources.
    Vello,
    /// Effect working textures.
    Effects,
    /// Resolved mask textures.
    Masks,
    /// Gaussian kernel buffers.
    Kernels,
    /// Descriptor-bearing resources outside the cache.
    External,
    /// Surface target textures.
    SurfaceTargets,
    /// Pass buffers.
    PassBuffers,
    /// Explicit readback staging buffers.
    ReadbackStaging,
    /// Opaque native objects.
    Objects,
    /// Native shader module creation.
    ShaderModule,
    /// Native bind-group layout creation.
    BindGroupLayout,
    /// Native pipeline layout creation.
    PipelineLayout,
    /// Native compute pipeline creation.
    ComputePipeline,
    /// Native render pipeline creation.
    RenderPipeline,
    /// Native sampler creation.
    Sampler,
    /// Native bind-group creation.
    BindGroup,
    /// Native texture-view creation.
    TextureView,
    /// Native command-encoder creation.
    CommandEncoder,
    /// Native command-buffer creation and submission.
    CommandBuffer,
    /// Explicit dependency utility aggregates.
    DependencyUtility,
    /// Mandatory terminal retirement.
    Retirement,
    /// Actual awaited native engine initialization.
    Initialization,
}
impl RenderGenerationScope {
    pub(crate) const ALL: [Self; 24] = [
        Self::Root,
        Self::Cache,
        Self::Vello,
        Self::Effects,
        Self::Masks,
        Self::Kernels,
        Self::External,
        Self::SurfaceTargets,
        Self::PassBuffers,
        Self::ReadbackStaging,
        Self::Objects,
        Self::ShaderModule,
        Self::BindGroupLayout,
        Self::PipelineLayout,
        Self::ComputePipeline,
        Self::RenderPipeline,
        Self::Sampler,
        Self::BindGroup,
        Self::TextureView,
        Self::CommandEncoder,
        Self::CommandBuffer,
        Self::DependencyUtility,
        Self::Retirement,
        Self::Initialization,
    ];
    pub(crate) const fn index(self) -> usize {
        self as usize
    }
    pub(crate) const fn parent(self) -> Option<Self> {
        match self {
            Self::Root => None,
            Self::Vello | Self::Effects | Self::Masks | Self::Kernels => Some(Self::Cache),
            Self::SurfaceTargets | Self::PassBuffers | Self::ReadbackStaging => {
                Some(Self::External)
            }
            Self::ShaderModule
            | Self::BindGroupLayout
            | Self::PipelineLayout
            | Self::ComputePipeline
            | Self::RenderPipeline
            | Self::Sampler
            | Self::BindGroup
            | Self::TextureView
            | Self::CommandEncoder
            | Self::CommandBuffer
            | Self::DependencyUtility => Some(Self::Objects),
            _ => Some(Self::Root),
        }
    }
    pub(crate) const fn name(self) -> &'static str {
        match self {
            Self::Root => "render.device",
            Self::Cache => "cache",
            Self::Vello => "vello",
            Self::Effects => "effects",
            Self::Masks => "masks",
            Self::Kernels => "kernels",
            Self::External => "external",
            Self::SurfaceTargets => "surface_targets",
            Self::PassBuffers => "pass_buffers",
            Self::ReadbackStaging => "readback_staging",
            Self::Objects => "objects",
            Self::ShaderModule => "shader_module",
            Self::BindGroupLayout => "bind_group_layout",
            Self::PipelineLayout => "pipeline_layout",
            Self::ComputePipeline => "compute_pipeline",
            Self::RenderPipeline => "render_pipeline",
            Self::Sampler => "sampler",
            Self::BindGroup => "bind_group",
            Self::TextureView => "texture_view",
            Self::CommandEncoder => "command_encoder",
            Self::CommandBuffer => "command_buffer",
            Self::DependencyUtility => "dependency_utility",
            Self::Retirement => "retirement",
            Self::Initialization => "initialization",
        }
    }
}

/// Bounded generation detail, in this owner's own elapsed coordinate.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum RenderGenerationEvent {
    /// An admitted work or custody change.
    Change {
        /// Actual origin.
        scope: RenderGenerationScope,
        /// Named unit.
        quantity: RenderQuantity,
        /// Admitted amount.
        amount: u64,
        /// Work/acquisition/release classification.
        kind: RenderChangeKind,
        /// Time since lifetime or interval opening.
        elapsed: Option<Duration>,
    },
    /// One actual ended native or initialization phase.
    Phase {
        /// Actual origin.
        scope: RenderGenerationScope,
        /// Inclusive host elapsed time.
        duration: Option<Duration>,
        /// Time since lifetime or interval opening.
        elapsed: Option<Duration>,
    },
    /// A checked admission failed without committing a prefix.
    Refused {
        /// Actual origin.
        scope: RenderGenerationScope,
        /// Time since lifetime or interval opening.
        elapsed: Option<Duration>,
    },
}
/// One exact generation interval, with its own peak and timing authority.
#[derive(Debug, Eq, PartialEq)]
pub struct RenderGenerationInterval {
    identity: RenderGenerationIdentity,
    data: ResolvedReport<RenderGenerationScope, IntervalNumbers>,
}
impl RenderGenerationInterval {
    pub(super) fn new(
        identity: RenderGenerationIdentity,
        interval: IntervalReport<RenderWork, RenderResource>,
        handles: &[Scope],
        backing: ReportBacking<RenderGenerationScope, IntervalNumbers>,
    ) -> Result<Self, RenderAccountingError> {
        let (_, data) = ResolvedReport::interval(interval, handles, backing)?;
        Ok(Self { identity, data })
    }
    /// Identity of the selected generation owner.
    pub const fn identity(&self) -> &RenderGenerationIdentity {
        &self.identity
    }
    /// Exact root work, closing custody and interval peak.
    pub fn usage(&self) -> RenderUsage {
        self.scope_usage(RenderGenerationScope::Root)
    }
    /// Inclusive fixed-scope interval attribution in every profile mode.
    pub fn scope_usage(&self, scope: RenderGenerationScope) -> RenderUsage {
        RenderUsage::new(self.data.rows[scope.index()].usage)
    }
    /// Root custody already live when the interval opened.
    pub fn opening(&self, resource: RenderResource) -> u64 {
        self.scope_opening(RenderGenerationScope::Root, resource)
    }
    /// Scope custody already live when the interval opened.
    pub fn scope_opening(&self, scope: RenderGenerationScope, resource: RenderResource) -> u64 {
        self.data.rows[scope.index()].usage.opening_live(resource)
    }
    /// This interval's host elapsed time.
    pub fn elapsed(&self) -> Option<Duration> {
        self.data.elapsed
    }
    /// Sum of inclusive ended phases at this scope.
    pub fn scope_elapsed(&self, scope: RenderGenerationScope) -> Option<Duration> {
        self.data.rows[scope.index()].phase_elapsed
    }
    /// Ended phase invocations; zero when observation is Off.
    pub fn scope_count(&self, scope: RenderGenerationScope) -> u64 {
        self.data.rows[scope.index()].phases
    }
    /// Selected detail lost to the configured bound.
    pub fn dropped_events(&self) -> u64 {
        self.data.dropped_events
    }
    /// Whether optional observational counters saturated.
    pub fn saturated(&self) -> bool {
        self.data.saturated
    }
    /// Already resolved bounded detail in its original order.
    pub fn events(&self) -> impl Iterator<Item = RenderGenerationEvent> + '_ {
        self.data.events.iter().copied()
    }
    pub(crate) fn cache_allocations(&self) -> u64 {
        self.scope_usage(RenderGenerationScope::Cache)
            .work(RenderWork::ResourceAllocations)
    }
    pub(crate) fn cache_reuses(&self) -> u64 {
        self.scope_usage(RenderGenerationScope::Cache)
            .work(RenderWork::ResourceReuses)
    }
    pub(crate) fn idle_cache_bytes(&self) -> u64 {
        self.scope_usage(RenderGenerationScope::Cache)
            .live(RenderResource::IdleCacheGpuBytes)
    }
    #[cfg(test)]
    pub(crate) fn snapshot_for_test(&self) -> Self {
        Self {
            identity: self.identity.clone(),
            data: self.data.snapshot_for_test(),
        }
    }
}

/// Pure coherent lifetime observation. Off retains root usage without role buffers.
#[derive(Debug, Eq, PartialEq)]
pub struct RenderDeviceSnapshot {
    identity: RenderGenerationIdentity,
    usage: RenderLifetimeUsage,
    data: ResolvedReport<RenderGenerationScope, LifetimeNumbers>,
    retired: bool,
    fault: Option<RenderAccountingError>,
}
impl RenderDeviceSnapshot {
    pub(super) fn new(
        identity: RenderGenerationIdentity,
        usage: RenderLifetimeUsage,
        profile: Option<Report<RenderWork, RenderResource>>,
        retired: bool,
        fault: Option<RenderAccountingError>,
        handles: &[Scope],
        backing: ReportBacking<RenderGenerationScope, LifetimeNumbers>,
    ) -> Result<Self, RenderAccountingError> {
        let (_, data) = ResolvedReport::snapshot(profile, handles, backing)?;
        Ok(Self {
            identity,
            usage,
            data,
            retired,
            fault,
        })
    }
    /// Identity of the retained native generation owner.
    pub const fn identity(&self) -> &RenderGenerationIdentity {
        &self.identity
    }
    /// Exact cumulative work, live custody and lifetime peak, including Off.
    pub const fn usage(&self) -> RenderLifetimeUsage {
        self.usage
    }
    /// Scoped usage is unavailable when observation is Off.
    pub fn scope_usage(&self, scope: RenderGenerationScope) -> Option<RenderLifetimeUsage> {
        self.data
            .observed
            .then(|| RenderLifetimeUsage::new(self.data.rows[scope.index()].usage))
    }
    /// Whether this owner already retired its native custody.
    pub const fn retired(&self) -> bool {
        self.retired
    }
    /// First persistent checked-accounting fault.
    pub const fn fault(&self) -> Option<RenderAccountingError> {
        self.fault
    }
    /// Host elapsed time since generation construction.
    pub fn elapsed(&self) -> Option<Duration> {
        self.data.elapsed
    }
    /// Inclusive ended-scope duration; unavailable when observation is Off.
    pub fn scope_elapsed(&self, scope: RenderGenerationScope) -> Option<Duration> {
        if self.data.observed {
            self.data.rows[scope.index()].phase_elapsed
        } else {
            None
        }
    }
    /// Ended invocations; zero when observation is Off.
    pub fn scope_count(&self, scope: RenderGenerationScope) -> u64 {
        if self.data.observed {
            self.data.rows[scope.index()].phases
        } else {
            0
        }
    }
    /// Selected lifetime detail lost to the configured bound.
    pub fn dropped_events(&self) -> u64 {
        self.data.dropped_events
    }
    /// Whether optional observational counters saturated.
    pub fn saturated(&self) -> bool {
        self.data.saturated
    }
    /// Already resolved bounded detail.
    pub fn events(&self) -> impl Iterator<Item = RenderGenerationEvent> + '_ {
        self.data.events.iter().copied()
    }
    #[cfg(test)]
    pub(crate) fn snapshot_for_test(&self) -> Self {
        Self {
            identity: self.identity.clone(),
            usage: self.usage,
            data: self.data.snapshot_for_test(),
            retired: self.retired,
            fault: self.fault,
        }
    }
}
