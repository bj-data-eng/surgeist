use crate::ErrorCode;
use std::time::Duration;
use surgeist_ledger::{IntervalReport, IntervalUsage};

/// Named work units for preparation and internal generation custody.
/// Preparation limits accept only the documented preparation dimensions; the
/// generation remains checked-unlimited. Reserved dimensions are identified below.
/// Values of different dimensions are not additive.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[non_exhaustive]
pub enum RenderWork {
    /// Reserved CommandVisits unit for later scene/GPU accounting; preparation reports zero.
    CommandVisits,
    /// Raw contour endpoints inspected or provider path-element callbacks consumed.
    PathElementVisits,
    /// Glyph facts inspected by Render, including repeated validation and dispatch.
    GlyphVisits,
    /// Raw COLR paint nodes, recorded layers and retained paints inspected.
    PaintVisits,
    /// Reserved GradientStopVisits unit for later scene/GPU accounting; preparation reports zero.
    GradientStopVisits,
    /// Directory/instance records, bitmap strikes and bitmap index records inspected.
    FontRecordVisits,
    /// Axis records or scalar coordinates inspected or converted by Render.
    CoordinateVisits,
    /// Qualifier point-flag records, points, decoded deltas or provider points inspected.
    OutlinePointVisits,
    /// Variation tuples inspected by the qualifier.
    VariationTupleVisits,
    /// Composite child edges inspected by the closure qualifier.
    GlyphComponentVisits,
    /// Reserved FilterOperationVisits unit for later scene/GPU accounting; preparation reports zero.
    FilterOperationVisits,
    /// Reserved KernelSampleVisits unit for later scene/GPU accounting; preparation reports zero.
    KernelSampleVisits,
    /// Reserved GraphNodeVisits unit for later scene/GPU accounting; preparation reports zero.
    GraphNodeVisits,
    /// Reserved GraphEdgeVisits unit for later scene/GPU accounting; preparation reports zero.
    GraphEdgeVisits,
    /// Shared-font identity candidates or managed cache entries actually inspected.
    ResourceProbes,
    /// Original glyph facts accepted once at the public preparation boundary.
    GlyphInputs,
    /// Original normalized coordinate values accepted once per input instance.
    CoordinateInputs,
    /// Nonempty placed Text runs accepted once in whole-layout preflight.
    TextRuns,
    /// Placed Text decoration records accepted once in preflight.
    TextDecorations,
    /// Retained glyph path elements appended, including synthesis and clip expansion.
    PathElementsEmitted,
    /// Reserved CommandsEmitted unit for later scene/GPU accounting; preparation reports zero.
    CommandsEmitted,
    /// Reserved FillsEmitted unit for later scene/GPU accounting; preparation reports zero.
    FillsEmitted,
    /// Reserved StrokesEmitted unit for later scene/GPU accounting; preparation reports zero.
    StrokesEmitted,
    /// Reserved ShadowsEmitted unit for later scene/GPU accounting; preparation reports zero.
    ShadowsEmitted,
    /// Reserved ImagesEmitted unit for later scene/GPU accounting; preparation reports zero.
    ImagesEmitted,
    /// Reserved GlyphsEmitted unit for later scene/GPU accounting; preparation reports zero.
    GlyphsEmitted,
    /// Reserved LayersEmitted unit for later scene/GPU accounting; preparation reports zero.
    LayersEmitted,
    /// Reserved RepeatTiles unit for later scene/GPU accounting; preparation reports zero.
    RepeatTiles,
    /// Reserved ImageSourceBytes unit for later scene/GPU accounting; preparation reports zero.
    ImageSourceBytes,
    /// Reserved AtlasTexels unit for later scene/GPU accounting; preparation reports zero.
    AtlasTexels,
    /// Reserved UploadBytes unit for later scene/GPU accounting; preparation reports zero.
    UploadBytes,
    /// IDAT payload bytes copied into Render-owned compressed storage.
    SerializedBytes,
    /// Reserved ScheduledFilterTexels unit for later scene/GPU accounting; preparation reports zero.
    ScheduledFilterTexels,
    /// Reserved RasterDispatches unit for later scene/GPU accounting; preparation reports zero.
    RasterDispatches,
    /// Reserved VelloPasses unit for later scene/GPU accounting; preparation reports zero.
    VelloPasses,
    /// Reserved ImagePasses unit for later scene/GPU accounting; preparation reports zero.
    ImagePasses,
    /// Reserved CompositePasses unit for later scene/GPU accounting; preparation reports zero.
    CompositePasses,
    /// Reserved CopyOperations unit for later scene/GPU accounting; preparation reports zero.
    CopyOperations,
    /// Reserved CustomPresentPasses unit for later scene/GPU accounting; preparation reports zero.
    CustomPresentPasses,
    /// Reserved Submissions unit for later scene/GPU accounting; preparation reports zero.
    Submissions,
    /// Reserved Presentations unit for later scene/GPU accounting; preparation reports zero.
    Presentations,
    /// Reserved ReadbackRows unit for later scene/GPU accounting; preparation reports zero.
    ReadbackRows,
    /// Reserved ReadbackBytes unit for later scene/GPU accounting; preparation reports zero.
    ReadbackBytes,
    /// Reserved ReadbackTransferredBytes unit for later scene/GPU accounting; preparation reports zero.
    ReadbackTransferredBytes,
    /// Explicit prepared-output backing units transferred to caller custody.
    PreparedTransferredBytes,
    /// Native descriptor/opaque constructor calls about to be invoked; failed previews do not count.
    ResourceAllocationRequests,
    /// Returned descriptor-bearing managed or external handles, including provisional handles.
    ResourceAllocations,
    /// Compatible idle managed cache entries selected for reuse.
    ResourceReuses,
    /// Managed leases returned to idle custody after exact identity validation.
    ResourceLeaseReturns,
    /// Managed entries removed by discard, idle retention or atlas replacement cleanup.
    ResourceEvictions,
    /// Device generations retired once, after mandatory live-custody release.
    ResourceRetirements,
    /// Returned opaque native handles or explicit dependency utility aggregates.
    GpuObjectsCreated,
    /// Reserved ImageContentHits unit for later scene/GPU accounting; preparation reports zero.
    ImageContentHits,
    /// Reserved ImageContentMisses unit for later scene/GPU accounting; preparation reports zero.
    ImageContentMisses,
    /// Reserved ImageFirstObservedBytes unit for later scene/GPU accounting; preparation reports zero.
    ImageFirstObservedBytes,
    /// PNG chunk type/payload bytes CRC-processed, plus scanline bytes unfiltered.
    PngByteVisits,
    /// Decoded RGBA pixels inspected for the transparent-image early exit.
    ImageTexelVisits,
}
impl RenderWork {
    pub(crate) const ALL: [Self; 57] = [
        Self::CommandVisits,
        Self::PathElementVisits,
        Self::GlyphVisits,
        Self::PaintVisits,
        Self::GradientStopVisits,
        Self::FontRecordVisits,
        Self::CoordinateVisits,
        Self::OutlinePointVisits,
        Self::VariationTupleVisits,
        Self::GlyphComponentVisits,
        Self::FilterOperationVisits,
        Self::KernelSampleVisits,
        Self::GraphNodeVisits,
        Self::GraphEdgeVisits,
        Self::ResourceProbes,
        Self::GlyphInputs,
        Self::CoordinateInputs,
        Self::TextRuns,
        Self::TextDecorations,
        Self::PathElementsEmitted,
        Self::CommandsEmitted,
        Self::FillsEmitted,
        Self::StrokesEmitted,
        Self::ShadowsEmitted,
        Self::ImagesEmitted,
        Self::GlyphsEmitted,
        Self::LayersEmitted,
        Self::RepeatTiles,
        Self::ImageSourceBytes,
        Self::AtlasTexels,
        Self::UploadBytes,
        Self::SerializedBytes,
        Self::ScheduledFilterTexels,
        Self::RasterDispatches,
        Self::VelloPasses,
        Self::ImagePasses,
        Self::CompositePasses,
        Self::CopyOperations,
        Self::CustomPresentPasses,
        Self::Submissions,
        Self::Presentations,
        Self::ReadbackRows,
        Self::ReadbackBytes,
        Self::ReadbackTransferredBytes,
        Self::PreparedTransferredBytes,
        Self::ResourceAllocationRequests,
        Self::ResourceAllocations,
        Self::ResourceReuses,
        Self::ResourceLeaseReturns,
        Self::ResourceEvictions,
        Self::ResourceRetirements,
        Self::GpuObjectsCreated,
        Self::ImageContentHits,
        Self::ImageContentMisses,
        Self::ImageFirstObservedBytes,
        Self::PngByteVisits,
        Self::ImageTexelVisits,
    ];
    pub(crate) const fn index(self) -> usize {
        self as usize
    }
    pub(crate) const fn dimension(self) -> surgeist_ledger::Work<WORK_COUNT> {
        match surgeist_ledger::Work::new(self.index()) {
            Some(d) => d,
            None => unreachable!(),
        }
    }
}
/// Named resource units. See the crate accounting reference for units.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[non_exhaustive]
pub enum RenderResource {
    /// Live explicit result backing: actual Vec capacity bytes, font/image payloads and documented fixed result units.
    RetainedCpuBytes,
    /// Live actual temporary Vec capacity bytes and declared fixed decoder workspace.
    ScratchCpuBytes,
    /// Live descriptor-defined native buffer/texture payload bytes held by Render; not driver allocation or RSS.
    GpuBytes,
    /// Live observed opaque native handles and explicitly documented dependency utility aggregates.
    OpaqueGpuObjects,
    /// Reserved LocalRepeatTileQuota unit for later scene/GPU accounting; preparation reports zero.
    LocalRepeatTileQuota,
    /// Reserved LocalAtlasSourceByteQuota unit for later scene/GPU accounting; preparation reports zero.
    LocalAtlasSourceByteQuota,
    /// Reserved LocalAtlasTexelQuota unit for later scene/GPU accounting; preparation reports zero.
    LocalAtlasTexelQuota,
    /// Classification of managed allocation bytes registered in cache entries; not additional physical storage.
    RegisteredCacheGpuBytes,
    /// Classification of managed allocation bytes eligible for idle retention; not additional physical storage.
    IdleCacheGpuBytes,
}
impl RenderResource {
    pub(crate) const ALL: [Self; 9] = [
        Self::RetainedCpuBytes,
        Self::ScratchCpuBytes,
        Self::GpuBytes,
        Self::OpaqueGpuObjects,
        Self::LocalRepeatTileQuota,
        Self::LocalAtlasSourceByteQuota,
        Self::LocalAtlasTexelQuota,
        Self::RegisteredCacheGpuBytes,
        Self::IdleCacheGpuBytes,
    ];
    pub(crate) const fn index(self) -> usize {
        self as usize
    }
    pub(crate) const fn dimension(self) -> surgeist_ledger::Resource<RESOURCE_COUNT> {
        match surgeist_ledger::Resource::new(self.index()) {
            Some(d) => d,
            None => unreachable!(),
        }
    }
}
/// Named fixed preparation phase. See the crate accounting reference for units.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[non_exhaustive]
pub enum RenderPhase {
    /// Complete operation interval; the general-limit ancestor of preparation.
    Attempt,
    /// Whole glyph/Text operation with legacy limits and direct CPU backing custody.
    Preparation,
    /// Whole-layout validation and once-only semantic input admission.
    TextPreflight,
    /// One exact font-instance glyph run, including nested representation work.
    GlyphPrepare,
    /// Font, variation, raw outline, COLRv1 program and prepared-placement qualification.
    Qualify,
    /// Provider draw and canonical outline collection.
    Outline,
    /// Selected intrinsic color-program recording and retained layer processing.
    Color,
    /// Selected bitmap qualification, transparency inspection and image placement.
    Bitmap,
    /// PNG validation, decompression and scanline reconstruction.
    Png,
    /// Normalized-coordinate conversion into the consumed representation.
    Coordinates,
    /// Final immutable result assembly after successful preparation.
    Output,
    /// Release of remaining attempt custody and successful output transfer.
    Cleanup,
}
impl RenderPhase {
    pub(crate) const ALL: [Self; 12] = [
        Self::Attempt,
        Self::Preparation,
        Self::TextPreflight,
        Self::GlyphPrepare,
        Self::Qualify,
        Self::Outline,
        Self::Color,
        Self::Bitmap,
        Self::Png,
        Self::Coordinates,
        Self::Output,
        Self::Cleanup,
    ];
    pub(crate) const fn index(self) -> usize {
        self as usize
    }
    pub(crate) const fn parent(self) -> Option<Self> {
        match self {
            Self::Attempt => None,
            Self::Preparation => Some(Self::Attempt),
            Self::TextPreflight => Some(Self::Preparation),
            Self::GlyphPrepare => Some(Self::Preparation),
            Self::Qualify => Some(Self::GlyphPrepare),
            Self::Outline => Some(Self::GlyphPrepare),
            Self::Color => Some(Self::GlyphPrepare),
            Self::Bitmap => Some(Self::GlyphPrepare),
            Self::Png => Some(Self::Bitmap),
            Self::Coordinates => Some(Self::Preparation),
            Self::Output => Some(Self::Preparation),
            Self::Cleanup => Some(Self::Preparation),
        }
    }
    /// Fixed human-readable phase name.
    pub const fn name(self) -> &'static str {
        match self {
            Self::Attempt => "render.attempt",
            Self::Preparation => "preparation",
            Self::TextPreflight => "text_preflight",
            Self::GlyphPrepare => "glyph_prepare",
            Self::Qualify => "qualify",
            Self::Outline => "outline",
            Self::Color => "color",
            Self::Bitmap => "bitmap",
            Self::Png => "png",
            Self::Coordinates => "coordinates",
            Self::Output => "output",
            Self::Cleanup => "cleanup",
        }
    }
}
pub(crate) const WORK_COUNT: usize = 57;
pub(crate) const RESOURCE_COUNT: usize = 9;
pub(crate) const PHASE_COUNT: usize = 12;

/// General attempt limits in named units. Defaults are checked-unlimited.
/// Existing glyph/Text limits remain a separate preparation ancestor. Only CPU
/// backing resources and preparation work can be capped in this delivery.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct RenderLimits {
    work: [u64; WORK_COUNT],
    retained: u64,
    scratch: u64,
}
impl RenderLimits {
    /// Checked-unlimited general limits; zero is a meaningful configured cap.
    pub const fn new() -> Self {
        Self {
            work: [u64::MAX; WORK_COUNT],
            retained: u64::MAX,
            scratch: u64::MAX,
        }
    }
    /// Sets an instrumented preparation-work cap. Text-only units require
    /// `render-text`. Other schema dimensions and
    /// transfer/cleanup events return [`RenderAccountingError::InvalidLimit`].
    pub fn with_work_limit(
        mut self,
        work: RenderWork,
        limit: u64,
    ) -> Result<Self, RenderAccountingError> {
        if !matches!(
            work,
            RenderWork::PathElementVisits
                | RenderWork::GlyphVisits
                | RenderWork::PaintVisits
                | RenderWork::FontRecordVisits
                | RenderWork::CoordinateVisits
                | RenderWork::OutlinePointVisits
                | RenderWork::VariationTupleVisits
                | RenderWork::GlyphComponentVisits
                | RenderWork::ResourceProbes
                | RenderWork::GlyphInputs
                | RenderWork::CoordinateInputs
                | RenderWork::TextRuns
                | RenderWork::TextDecorations
                | RenderWork::PathElementsEmitted
                | RenderWork::SerializedBytes
                | RenderWork::PngByteVisits
                | RenderWork::ImageTexelVisits
        ) {
            return Err(RenderAccountingError::InvalidLimit(work));
        }
        if !cfg!(feature = "render-text")
            && matches!(
                work,
                RenderWork::ResourceProbes | RenderWork::TextRuns | RenderWork::TextDecorations
            )
        {
            return Err(RenderAccountingError::InvalidLimit(work));
        }
        self.work[work.index()] = limit;
        Ok(self)
    }
    /// Sets the total simultaneously admitted CPU output backing byte cap.
    pub const fn with_retained_bytes(mut self, bytes: u64) -> Self {
        self.retained = bytes;
        self
    }
    /// Sets the total simultaneously admitted scratch backing byte cap.
    pub const fn with_scratch_bytes(mut self, bytes: u64) -> Self {
        self.scratch = bytes;
        self
    }
    /// Returns the configured work limit in that dimension's units.
    pub const fn work_limit(self, work: RenderWork) -> u64 {
        self.work[work.index()]
    }
    /// Returns the general retained-backing cap in bytes.
    pub const fn retained_bytes(self) -> u64 {
        self.retained
    }
    /// Returns the general scratch-backing cap in bytes.
    pub const fn scratch_bytes(self) -> u64 {
        self.scratch
    }
    pub(crate) fn ledger(self) -> surgeist_ledger::Limits<WORK_COUNT, RESOURCE_COUNT> {
        let mut r = [u64::MAX; RESOURCE_COUNT];
        r[RenderResource::RetainedCpuBytes.index()] = self.retained;
        r[RenderResource::ScratchCpuBytes.index()] = self.scratch;
        surgeist_ledger::Limits::new(self.work, r)
    }
}
impl Default for RenderLimits {
    fn default() -> Self {
        Self::new()
    }
}
/// Source of monotonic elapsed time; callbacks must not panic or have side effects.
#[derive(Clone, Copy, Debug, Default)]
pub enum RenderClock {
    /// Native Ledger clock. Timed requests on unsupported targets refuse setup.
    #[default]
    Platform,
    /// Host-provided duration from an arbitrary fixed epoch.
    Host(fn() -> Duration),
}
/// Optional bounded observation, independent of authoritative usage.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub enum RenderProfile {
    /// No observer, events or clock reads. Exact interval usage remains available.
    #[default]
    Off,
    /// Per-phase summary, optionally timed.
    Summary {
        /// Whether elapsed time is collected.
        timing: bool,
    },
    /// Per-phase summary and explicitly bounded selected events.
    Detailed {
        /// Whether elapsed time is collected.
        timing: bool,
        /// Maximum retained events, including meaningful zero.
        event_limit: usize,
        /// Phase-origin selection for detail only.
        filter: RenderPhaseFilter,
    },
}
/// Origins selected for detailed observation; summary/usage always covers all.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum RenderPhaseFilter {
    /// Every origin.
    All,
    /// Only the named fixed phase.
    Phase(RenderPhase),
}
impl RenderProfile {
    pub(crate) fn ledger(self) -> surgeist_ledger::ProfileMode {
        match self {
            Self::Off => surgeist_ledger::ProfileMode::Off,
            Self::Summary { timing } => surgeist_ledger::ProfileMode::Summary { timing },
            Self::Detailed {
                timing,
                event_limit,
                filter,
            } => surgeist_ledger::ProfileMode::Detailed {
                timing,
                event_limit,
                filter: match filter {
                    RenderPhaseFilter::All => surgeist_ledger::EventFilter::All,
                    RenderPhaseFilter::Phase(p) => {
                        surgeist_ledger::EventFilter::ScopeName(p.name())
                    }
                },
            },
        }
    }
}
/// Preparation configuration. Default observation is off, limits unlimited.
#[derive(Clone, Copy, Debug, Default)]
pub struct RenderAccountingOptions {
    pub(crate) limits: RenderLimits,
    pub(crate) profile: RenderProfile,
    pub(crate) clock: RenderClock,
}
impl RenderAccountingOptions {
    /// Returns default configuration.
    pub fn new() -> Self {
        Self::default()
    }
    /// Sets general limits without changing legacy caller limits.
    pub const fn with_limits(mut self, limits: RenderLimits) -> Self {
        self.limits = limits;
        self
    }
    /// Selects optional observation.
    pub const fn with_profile(mut self, profile: RenderProfile) -> Self {
        self.profile = profile;
        self
    }
    /// Selects the Ledger clock for timed profiles.
    pub const fn with_clock(mut self, clock: RenderClock) -> Self {
        self.clock = clock;
        self
    }
    /// Returns general limits.
    pub const fn limits(self) -> RenderLimits {
        self.limits
    }
    /// Returns profile configuration.
    pub const fn profile(self) -> RenderProfile {
        self.profile
    }
    /// Returns clock configuration.
    pub const fn clock(self) -> RenderClock {
        self.clock
    }
}
/// Operation whose report is retained; later rendering deliveries extend this set.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[non_exhaustive]
pub enum RenderAttemptKind {
    /// Standalone exact glyph preparation.
    GlyphRun,
    /// Whole placed Text-layout preparation.
    TextLayout,
}
/// Semantic terminal outcome, separate from last-successful frame statistics.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[non_exhaustive]
pub enum RenderAttemptStatus {
    /// Output transferred to caller.
    Succeeded,
    /// Named admission refused; accepted-prefix work remains.
    Refused,
    /// A semantic/provider error prevented output.
    Failed(ErrorCode),
    /// Unwind or canceled work completed cleanup.
    Canceled,
}
/// Dimension involved in a refusal or detailed event.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum RenderQuantity {
    /// Cumulative work in its named unit.
    Work(RenderWork),
    /// Live custody or named algorithm quota.
    Resource(RenderResource),
}
/// Why a checked admission failed.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum RenderRefusalReason {
    /// Proposed usage exceeded this cap.
    Limit {
        /// Configured cap in the quantity's units.
        limit: u64,
    },
    /// Checked arithmetic overflowed.
    Overflow,
    /// A release exceeded direct custody.
    Underflow,
}
/// First refusal, including noncommitting capacity previews.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct RenderRefusal {
    pub(crate) phase: RenderPhase,
    pub(crate) quantity: RenderQuantity,
    pub(crate) current: u64,
    pub(crate) amount: u64,
    pub(crate) reason: RenderRefusalReason,
}
impl RenderRefusal {
    /// Scope whose contract refused.
    pub const fn phase(self) -> RenderPhase {
        self.phase
    }
    /// Work/resource dimension and its explicit unit.
    pub const fn quantity(self) -> RenderQuantity {
        self.quantity
    }
    /// Usage before the refused change (including its provisional batch prefix).
    pub const fn current(self) -> u64 {
        self.current
    }
    /// Refused amount in that quantity's units.
    pub const fn amount(self) -> u64 {
        self.amount
    }
    /// Limit/arithmetic classification.
    pub const fn reason(self) -> RenderRefusalReason {
        self.reason
    }
}
/// Setup failure occurs before an attempt starts or runs native/provider work.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum RenderAccountingSetupError {
    /// Bounded report allocation failed.
    Allocation,
    /// Timed profiling has no clock on this target.
    TimingUnavailable,
    /// Operation identities were exhausted.
    IdentityExhausted,
}
/// General accounting failure. Legacy limits retain their glyph/Text errors.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[non_exhaustive]
pub enum RenderAccountingError {
    /// No attempt started; previous report is preserved.
    Setup(RenderAccountingSetupError),
    /// A general named limit refused.
    Refused(RenderRefusal),
    /// This dimension has no limitable preparation boundary.
    InvalidLimit(RenderWork),
    /// A private fixed-scope, checkpoint or participant contract was violated.
    Invariant,
    /// Generation work or native custody failed a checked transition or preview.
    /// A preview refusal leaves the generation healthy; committed transition
    /// failures retain the first fault while mandatory cleanup continues.
    GenerationRefused {
        /// Actual named work or custody unit.
        quantity: RenderQuantity,
        /// Accepted usage before the change.
        current: u64,
        /// Requested work/acquisition/release amount.
        amount: u64,
        /// Checked arithmetic classification.
        reason: RenderRefusalReason,
    },
    /// A retired device generation cannot acquire or start new work.
    /// Existing native owners and late callbacks may still release harmlessly.
    GenerationRetired,
}
impl std::fmt::Display for RenderAccountingError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{self:?}")
    }
}
impl std::error::Error for RenderAccountingError {}
/// Exact inclusive accepted usage. Parent and child values are attribution, not sums.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct RenderUsage(IntervalUsage<WORK_COUNT, RESOURCE_COUNT>);
impl RenderUsage {
    /// Accepted work in the named dimension's units.
    pub const fn work(self, work: RenderWork) -> u64 {
        self.0.work(work.dimension())
    }
    /// Closing admitted custody in the named resource's units.
    pub const fn live(self, resource: RenderResource) -> u64 {
        self.0.live(resource.dimension())
    }
    /// Exact interval maximum of admitted custody, including growth overlap.
    pub const fn peak(self, resource: RenderResource) -> u64 {
        self.0.peak(resource.dimension())
    }
}
/// Bounded selected observation of an admitted change or ended inclusive phase.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum RenderEvent {
    /// One admitted atomic batch item; acquire/release names distinguish custody.
    Change {
        /// Actual origin.
        phase: RenderPhase,
        /// Named dimension.
        quantity: RenderQuantity,
        /// Admitted amount.
        amount: u64,
        /// Work/acquire/release classification.
        kind: RenderChangeKind,
        /// Optional time since interval opening.
        elapsed: Option<Duration>,
    },
    /// One ended phase; timing is inclusive and may overlap other phases.
    Phase {
        /// Phase that ended.
        phase: RenderPhase,
        /// Optional inclusive elapsed time.
        duration: Option<Duration>,
        /// Optional completion time from interval opening.
        elapsed: Option<Duration>,
    },
    /// An admission failed without committing its provisional prefix.
    Refused {
        /// Refusal origin.
        phase: RenderPhase,
        /// Optional time since opening.
        elapsed: Option<Duration>,
    },
}
/// Meaning of an admitted detailed change.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum RenderChangeKind {
    /// Cumulative work.
    Work,
    /// New admitted custody.
    Acquire,
    /// Custody released or transferred.
    Release,
}
/// Closed preparation report, available with profiling off and after failures.
/// Only the latest started attempt is retained by a context; event detail is bounded.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct RenderAttemptReport {
    pub(crate) kind: RenderAttemptKind,
    pub(crate) status: RenderAttemptStatus,
    pub(crate) interval: IntervalReport<WORK_COUNT, RESOURCE_COUNT>,
    pub(crate) refusal: Option<RenderRefusal>,
}
impl RenderAttemptReport {
    /// Operation represented by this report.
    pub const fn kind(&self) -> RenderAttemptKind {
        self.kind
    }
    /// Semantic outcome after cleanup.
    pub const fn status(&self) -> RenderAttemptStatus {
        self.status
    }
    /// Exact root usage, regardless of profiling mode.
    pub fn usage(&self) -> RenderUsage {
        self.phase_usage(RenderPhase::Attempt)
    }
    /// Inclusive phase attribution. Do not add it to ancestor totals.
    pub fn phase_usage(&self, phase: RenderPhase) -> RenderUsage {
        RenderUsage(self.interval.scopes[phase.index()].usage)
    }
    /// First refusal, including a noncommitting pre-allocation preview.
    pub const fn refusal(&self) -> Option<RenderRefusal> {
        self.refusal
    }
    /// Total measured interval, absent when off/untimed.
    pub fn elapsed(&self) -> Option<Duration> {
        self.interval.profile.as_ref().and_then(|p| p.elapsed)
    }
    /// Sum of ended inclusive phase durations, absent when off/untimed.
    pub fn phase_elapsed(&self, phase: RenderPhase) -> Option<Duration> {
        self.interval
            .profile
            .as_ref()
            .and_then(|p| p.scopes[phase.index()].phase_elapsed)
    }
    /// Number of ended invocations (observational, zero with profiling off).
    pub fn phase_count(&self, phase: RenderPhase) -> u64 {
        self.interval
            .profile
            .as_ref()
            .map_or(0, |p| p.scopes[phase.index()].phases)
    }
    /// Detail lost to the configured bounded buffer; exact usage is unaffected.
    pub fn dropped_events(&self) -> u64 {
        self.interval
            .profile
            .as_ref()
            .map_or(0, |p| p.dropped_events)
    }
    /// Whether any optional observational counter saturated.
    pub fn saturated(&self) -> bool {
        self.interval.profile.as_ref().is_some_and(|p| p.saturated)
    }
    /// Bounded selected detail, projected without allocating another event buffer.
    pub fn events(&self) -> impl Iterator<Item = RenderEvent> + '_ {
        self.interval
            .profile
            .iter()
            .flat_map(|p| p.events.iter())
            .map(|e| {
                let phase = |scope| {
                    RenderPhase::ALL
                        .into_iter()
                        .find(|p| self.interval.scopes[p.index()].scope == scope)
                        .expect("owned report scope")
                };
                match *e {
                    surgeist_ledger::Event::Phase {
                        scope,
                        duration,
                        elapsed,
                    } => RenderEvent::Phase {
                        phase: phase(scope),
                        duration,
                        elapsed,
                    },
                    surgeist_ledger::Event::Refused {
                        origin, elapsed, ..
                    } => RenderEvent::Refused {
                        phase: phase(origin),
                        elapsed,
                    },
                    surgeist_ledger::Event::Accepted {
                        scope,
                        change,
                        elapsed,
                    } => {
                        let (quantity, amount, kind) = match change {
                            surgeist_ledger::Change::Work(d, n) => (
                                RenderQuantity::Work(
                                    RenderWork::ALL
                                        .into_iter()
                                        .find(|w| w.dimension() == d)
                                        .expect("owned dimension"),
                                ),
                                n,
                                RenderChangeKind::Work,
                            ),
                            surgeist_ledger::Change::Acquire(d, n) => (
                                RenderQuantity::Resource(
                                    RenderResource::ALL
                                        .into_iter()
                                        .find(|w| w.dimension() == d)
                                        .expect("owned dimension"),
                                ),
                                n,
                                RenderChangeKind::Acquire,
                            ),
                            surgeist_ledger::Change::Release(d, n) => (
                                RenderQuantity::Resource(
                                    RenderResource::ALL
                                        .into_iter()
                                        .find(|w| w.dimension() == d)
                                        .expect("owned dimension"),
                                ),
                                n,
                                RenderChangeKind::Release,
                            ),
                        };
                        RenderEvent::Change {
                            phase: phase(scope),
                            quantity,
                            amount,
                            kind,
                            elapsed,
                        }
                    }
                }
            })
    }
}
