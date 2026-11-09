//! Typed observations of one retained native device-generation owner.
use super::{
    RenderAccountingError, RenderChangeKind, RenderLifetimeUsage, RenderQuantity, RenderResource,
    RenderUsage, RenderWork, interval_resolves, profile_resolves,
};
use std::{fmt, sync::Arc, time::Duration};
use surgeist_ledger::{Change, Event, IntervalReport, Report, Scope};

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
fn project(
    event: &Event<RenderWork, RenderResource>,
    scopes: impl Fn(Scope) -> Option<RenderGenerationScope>,
) -> Option<RenderGenerationEvent> {
    Some(match *event {
        Event::Phase {
            scope,
            duration,
            elapsed,
        } => RenderGenerationEvent::Phase {
            scope: scopes(scope)?,
            duration,
            elapsed,
        },
        Event::Refused {
            origin, elapsed, ..
        } => RenderGenerationEvent::Refused {
            scope: scopes(origin)?,
            elapsed,
        },
        Event::Accepted {
            scope,
            change,
            elapsed,
        } => {
            let (quantity, amount, kind) = match change {
                Change::Work(d, n) => (RenderQuantity::Work(d), n, RenderChangeKind::Work),
                Change::Acquire(d, n) => {
                    (RenderQuantity::Resource(d), n, RenderChangeKind::Acquire)
                }
                Change::Release(d, n) => {
                    (RenderQuantity::Resource(d), n, RenderChangeKind::Release)
                }
            };
            RenderGenerationEvent::Change {
                scope: scopes(scope)?,
                quantity,
                amount,
                kind,
                elapsed,
            }
        }
    })
}

/// One exact generation checkpoint interval. Its peak and timing are distinct
/// from the attempt and CPU intervals and cannot be added to them.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct RenderGenerationInterval {
    identity: RenderGenerationIdentity,
    interval: IntervalReport<RenderWork, RenderResource>,
}
impl RenderGenerationInterval {
    /// Builds an interval, or refuses one that does not hold the fixed scopes.
    pub(crate) fn new(
        identity: RenderGenerationIdentity,
        interval: IntervalReport<RenderWork, RenderResource>,
    ) -> Result<Self, RenderAccountingError> {
        if interval_resolves(&interval, RenderGenerationScope::ALL.len()) {
            Ok(Self { identity, interval })
        } else {
            Err(RenderAccountingError::Invariant)
        }
    }
    /// Identity of the selected generation owner.
    pub const fn identity(&self) -> &RenderGenerationIdentity {
        &self.identity
    }
    /// Exact root work, closing custody and interval peak.
    pub fn usage(&self) -> RenderUsage {
        self.scope_usage(RenderGenerationScope::Root)
    }
    /// Exact inclusive fixed-scope interval attribution in every profile mode.
    pub fn scope_usage(&self, scope: RenderGenerationScope) -> RenderUsage {
        RenderUsage::new(self.interval.scopes[scope.index()].usage)
    }
    /// Custody already live when this interval opened.
    pub fn opening(&self, resource: RenderResource) -> u64 {
        self.scope_opening(RenderGenerationScope::Root, resource)
    }
    /// Inclusive scope custody live when this interval opened.
    pub fn scope_opening(&self, scope: RenderGenerationScope, resource: RenderResource) -> u64 {
        self.interval.scopes[scope.index()]
            .usage
            .opening_live(resource)
    }
    /// This generation interval's host elapsed time, absent Off or untimed.
    pub fn elapsed(&self) -> Option<Duration> {
        self.interval.profile.as_ref().and_then(|p| p.elapsed)
    }
    /// Sum of actual ended inclusive phases at this scope.
    pub fn scope_elapsed(&self, scope: RenderGenerationScope) -> Option<Duration> {
        self.interval
            .profile
            .as_ref()
            .and_then(|p| p.scopes[scope.index()].phase_elapsed)
    }
    /// Actual ended phase count; zero when observation is Off.
    pub fn scope_count(&self, scope: RenderGenerationScope) -> u64 {
        self.interval
            .profile
            .as_ref()
            .map_or(0, |p| p.scopes[scope.index()].phases)
    }
    /// Selected events discarded by the bounded observer.
    pub fn dropped_events(&self) -> u64 {
        self.interval
            .profile
            .as_ref()
            .map_or(0, |p| p.dropped_events)
    }
    /// Whether optional observational counters saturated.
    pub fn saturated(&self) -> bool {
        self.interval.profile.as_ref().is_some_and(|p| p.saturated)
    }
    /// Selected detail without allocating another event buffer.
    pub fn events(&self) -> impl Iterator<Item = RenderGenerationEvent> + '_ {
        self.interval
            .profile
            .iter()
            .flat_map(|p| &p.events)
            .filter_map(|e| {
                project(e, |s| {
                    RenderGenerationScope::ALL.into_iter().find(|r| {
                        self.interval
                            .scopes
                            .get(r.index())
                            .is_some_and(|entry| entry.scope == s)
                    })
                })
            })
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
}

/// Pure coherent lifetime observation of one retained generation. Retirement
/// reports already-observed custody state, not a live driver-health probe.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct RenderDeviceSnapshot {
    identity: RenderGenerationIdentity,
    usage: RenderLifetimeUsage,
    profile: Option<Report<RenderWork, RenderResource>>,
    retired: bool,
    fault: Option<RenderAccountingError>,
}
impl RenderDeviceSnapshot {
    /// Builds a snapshot, or refuses a profile that does not hold the fixed scopes.
    pub(crate) fn new(
        identity: RenderGenerationIdentity,
        usage: RenderLifetimeUsage,
        profile: Option<Report<RenderWork, RenderResource>>,
        retired: bool,
        fault: Option<RenderAccountingError>,
    ) -> Result<Self, RenderAccountingError> {
        let resolves = profile.as_ref().is_none_or(|profile| {
            profile_resolves(profile, RenderGenerationScope::ALL.len(), |scope| {
                profile.scopes.iter().any(|entry| entry.scope == scope)
            })
        });
        if resolves {
            Ok(Self {
                identity,
                usage,
                profile,
                retired,
                fault,
            })
        } else {
            Err(RenderAccountingError::Invariant)
        }
    }
    /// Identity of the retained native generation owner.
    pub const fn identity(&self) -> &RenderGenerationIdentity {
        &self.identity
    }
    /// Exact cumulative work, live custody and lifetime peak, including Off.
    pub const fn usage(&self) -> RenderLifetimeUsage {
        self.usage
    }
    /// Inclusive lifetime attribution from the same coherent profile snapshot.
    /// Absent when observation is Off.
    pub fn scope_usage(&self, scope: RenderGenerationScope) -> Option<RenderLifetimeUsage> {
        self.profile
            .as_ref()
            .map(|p| RenderLifetimeUsage::new(p.scopes[scope.index()].usage))
    }
    /// Whether this owner has already retired its native custody.
    pub const fn retired(&self) -> bool {
        self.retired
    }
    /// First persistent checked-accounting fault, if any.
    pub const fn fault(&self) -> Option<RenderAccountingError> {
        self.fault
    }
    /// Host time since this generation owner was constructed, including idle gaps.
    pub fn elapsed(&self) -> Option<Duration> {
        self.profile.as_ref().and_then(|p| p.elapsed)
    }
    /// Sum of ended inclusive phase durations over this owner's lifetime.
    pub fn scope_elapsed(&self, scope: RenderGenerationScope) -> Option<Duration> {
        self.profile
            .as_ref()
            .and_then(|p| p.scopes[scope.index()].phase_elapsed)
    }
    /// Ended lifetime phase count; zero when observation is Off.
    pub fn scope_count(&self, scope: RenderGenerationScope) -> u64 {
        self.profile
            .as_ref()
            .map_or(0, |p| p.scopes[scope.index()].phases)
    }
    /// Selected lifetime events discarded by this bounded observer.
    pub fn dropped_events(&self) -> u64 {
        self.profile.as_ref().map_or(0, |p| p.dropped_events)
    }
    /// Whether optional observational counters saturated.
    pub fn saturated(&self) -> bool {
        self.profile.as_ref().is_some_and(|p| p.saturated)
    }
    /// Selected lifetime detail without allocating another event buffer.
    pub fn events(&self) -> impl Iterator<Item = RenderGenerationEvent> + '_ {
        let profile = self.profile.as_ref();
        self.profile
            .iter()
            .flat_map(|p| &p.events)
            .filter_map(move |e| {
                project(e, |s| {
                    RenderGenerationScope::ALL.into_iter().find(|r| {
                        profile
                            .and_then(|p| p.scopes.get(r.index()))
                            .is_some_and(|entry| entry.scope == s)
                    })
                })
            })
    }
}
