use crate::ErrorCode;
use std::time::Duration;
use surgeist_ledger::{Dimension, IntervalReport, IntervalUsage, Scope, dimensions};

dimensions! {
    /// Named work units for preparation, frame execution and generation custody.
    /// Limits apply only to dimensions produced by the selected operation. Generation
    /// lifecycle maintenance and result transfer remain checked-unlimited.
    /// Values of different dimensions are not additive.
    #[non_exhaustive]
    pub enum RenderWork => RenderWorkTable {
        /// Authored or normalized command records actually inspected, including repeated traversals.
        CommandVisits,
        /// Contour endpoints or path elements inspected, including provider callbacks
        /// and repeated validation, materialization and encoding walks.
        PathElementVisits,
        /// Glyph facts inspected by Render, including repeated validation and dispatch.
        GlyphVisits,
        /// Paint records or nodes inspected during COLR qualification, retained-paint
        /// consumption, normalization and encoding.
        PaintVisits,
        /// Gradient stops inspected during validation, materialization or encoding.
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
        /// Filter operations inspected during normalization, planning or encoding,
        /// including paired fixed-size serialized operation records in exact proofs.
        FilterOperationVisits,
        /// Gaussian sample records calculated or inspected by Render, including
        /// paired fixed-size serialized samples in exact proofs.
        KernelSampleVisits,
        /// Frame, pass or runtime graph nodes actually inspected, including repeated proof passes.
        GraphNodeVisits,
        /// Frame, pass or runtime graph relationships actually inspected.
        GraphEdgeVisits,
        /// Explicit resource/key/cache lookups or candidate comparisons, including
        /// repeated registry/index/proof searches; excludes opaque bucket probes.
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
        /// Normalized command records appended, including nested records and style expansion.
        CommandsEmitted,
        /// Normalized fill commands appended.
        FillsEmitted,
        /// Normalized stroke commands appended.
        StrokesEmitted,
        /// Normalized shadow instances appended.
        ShadowsEmitted,
        /// Normalized image draw occurrences appended.
        ImagesEmitted,
        /// Normalized glyph instances retained in emitted commands.
        GlyphsEmitted,
        /// Normalized layer commands appended.
        LayersEmitted,
        /// Repeat tile records emitted by the image-repeat producer.
        RepeatTiles,
        /// Image source payload bytes admitted to an actual atlas build, once per selected source.
        ImageSourceBytes,
        /// Selected expanded image-region texels admitted to an atlas build; excludes unused packing space.
        AtlasTexels,
        /// Actual source bytes passed to queue buffer or texture write calls.
        UploadBytes,
        /// Bytes serialized into PNG compressed storage, shader sources, native parameters,
        /// kernels or packed raster streams.
        SerializedBytes,
        /// Output-scissor area scheduled for actual color, blur, colorize or shadow-merge passes.
        ScheduledFilterTexels,
        /// Nonempty native compute dispatch calls issued by the internal raster engine.
        RasterDispatches,
        /// Logical internal Vello raster passes actually encoded.
        VelloPasses,
        /// Native attachment-clear passes and image-processing shader draws, including backdrop-copy draws.
        ImagePasses,
        /// Native composition draws encoded by the closed GPU graph.
        CompositePasses,
        /// Actual native texture-to-texture copy commands encoded.
        CopyOperations,
        /// Custom graph-to-surface presentation draws encoded.
        CustomPresentPasses,
        /// Actual queue submission calls issued.
        Submissions,
        /// Actual host surface presentation calls issued.
        Presentations,
        /// Actual texture-to-buffer copy commands encoded for explicit readback.
        ReadbackCopies,
        /// Actual native mapping requests registered for explicit readback.
        ReadbackMapRequests,
        /// Readback source rows inspected, including an incomplete row that fails validation.
        ReadbackRows,
        /// Logical unpadded RGBA8 bytes actually copied from mapped readback rows.
        ReadbackBytes,
        /// Actual admitted output capacity transferred at successful caller handoff.
        /// This is Renderer CPU lifetime work after the readback interval closes.
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
        /// Image occurrences whose exact content was already observed in the current history draft.
        ImageContentHits,
        /// Image occurrences that introduce distinct exact content into the current history draft.
        ImageContentMisses,
        /// Source payload bytes of first-observed exact image content; independent of GPU uploads.
        ImageFirstObservedBytes,
        /// PNG chunk type/payload bytes CRC-processed, plus scanline bytes unfiltered.
        PngByteVisits,
        /// Decoded RGBA pixels inspected for the transparent-image early exit.
        ImageTexelVisits,
        /// Authored command records inspected, including nested records.
        AuthoredCommands,
        /// Authored fill commands inspected.
        AuthoredFills,
        /// Authored stroke commands inspected.
        AuthoredStrokes,
        /// Authored shadow instances inspected.
        AuthoredShadows,
        /// Authored image occurrences inspected, including image paints and intrinsic bitmaps.
        AuthoredImages,
        /// Authored glyph instances inspected.
        AuthoredGlyphs,
        /// Authored nested layer commands inspected.
        AuthoredLayers,
        /// Exact image identity records actually copied or shifted; excludes
        /// replacement-backing relocation.
        ImageIdentityCopies,
        /// Pairs of bytes actually compared for exact image content, including first difference.
        ImageIdentityByteComparisons,
        /// Owned retained identity/history records disposed by uncapped maintenance;
        /// excludes borrowed observation references and ordinary Image clones.
        ImageIdentityDrops,
        /// Actual host surface texture acquisition calls.
        SurfaceAcquisitions,
        /// Shader binding records actually inspected or converted.
        ShaderBindingVisits,
        /// Shader key elements actually copied into distinct owned backing.
        ShaderKeyCopies,
        /// Existing elements physically moved between replacement owned-vector
        /// backings; excludes new writes, insertion shifts and in-place rotations.
        /// Standalone preparation charges its forward owner. Renderer vectors
        /// charge checked-unlimited CPU lifetime work, never frame/readback work.
        StorageRelocations,
        /// Typed elements actually copied between independently owned Vello
        /// encoding streams; excludes replacement-backing relocation.
        EncodingStreamCopies,
        /// Qualified opaque dash initializers entered, admitted once before
        /// dependency entry. Internal arithmetic is indivisible; owned cycle
        /// proof and interval conversion retain their separate visit units.
        DashInitializations,
        /// Private raster recording commands actually appended.
        RasterCommandsEmitted,
        /// Private raster recording command records actually inspected.
        RasterCommandVisits,
        /// Render-owned raster source/index records copied or shifted, including
        /// insertion shifts; excludes replacement relocation and eleven streams.
        RasterResourceCopies,
    }
}
impl RenderWork {
    pub(crate) const fn limit_supported(self) -> bool {
        self.supported_for(RenderAttemptKind::Frame)
            || self.supported_for(RenderAttemptKind::Readback)
            || self.supported_for(RenderAttemptKind::GlyphRun)
            || self.supported_for(RenderAttemptKind::TextLayout)
            || self.supported_for(RenderAttemptKind::SceneInspection)
            || self.supported_for(RenderAttemptKind::ImageRepeat)
            || self.supported_for(RenderAttemptKind::Background)
            || self.supported_for(RenderAttemptKind::Clip)
            || self.supported_for(RenderAttemptKind::BoxDecoration)
            || self.supported_for(RenderAttemptKind::PaintGradient)
            || self.supported_for(RenderAttemptKind::SdrColor)
    }
    pub(crate) const fn supported_for(self, kind: RenderAttemptKind) -> bool {
        match kind {
            RenderAttemptKind::SdrColor => {
                matches!(self, Self::PaintVisits | Self::CoordinateVisits)
            }
            RenderAttemptKind::PaintGradient => {
                cfg!(feature = "render-paint")
                    && matches!(
                        self,
                        Self::PaintVisits | Self::GradientStopVisits | Self::CoordinateVisits
                    )
            }
            RenderAttemptKind::Readback => matches!(
                self,
                Self::ReadbackCopies
                    | Self::ReadbackMapRequests
                    | Self::ReadbackRows
                    | Self::ReadbackBytes
                    | Self::Submissions
                    | Self::ResourceAllocationRequests
            ),
            RenderAttemptKind::GlyphRun => matches!(
                self,
                Self::StorageRelocations
                    | Self::PathElementVisits
                    | Self::GlyphVisits
                    | Self::PaintVisits
                    | Self::FontRecordVisits
                    | Self::CoordinateVisits
                    | Self::OutlinePointVisits
                    | Self::VariationTupleVisits
                    | Self::GlyphComponentVisits
                    | Self::GlyphInputs
                    | Self::CoordinateInputs
                    | Self::PathElementsEmitted
                    | Self::SerializedBytes
                    | Self::PngByteVisits
                    | Self::ImageTexelVisits
            ),
            RenderAttemptKind::TextLayout => {
                cfg!(feature = "render-text")
                    && (self.supported_for(RenderAttemptKind::GlyphRun)
                        || matches!(
                            self,
                            Self::ResourceProbes | Self::TextRuns | Self::TextDecorations
                        ))
            }
            RenderAttemptKind::Frame => {
                matches!(
                    self,
                    Self::CommandVisits
                        | Self::PathElementVisits
                        | Self::GlyphVisits
                        | Self::PaintVisits
                        | Self::GradientStopVisits
                        | Self::FontRecordVisits
                        | Self::CoordinateVisits
                        | Self::FilterOperationVisits
                        | Self::KernelSampleVisits
                        | Self::GraphNodeVisits
                        | Self::GraphEdgeVisits
                        | Self::ResourceProbes
                        | Self::CommandsEmitted
                        | Self::FillsEmitted
                        | Self::StrokesEmitted
                        | Self::ShadowsEmitted
                        | Self::ImagesEmitted
                        | Self::GlyphsEmitted
                        | Self::LayersEmitted
                        | Self::RepeatTiles
                        | Self::ImageSourceBytes
                        | Self::AtlasTexels
                        | Self::UploadBytes
                        | Self::SerializedBytes
                        | Self::ScheduledFilterTexels
                        | Self::RasterDispatches
                        | Self::VelloPasses
                        | Self::ImagePasses
                        | Self::CompositePasses
                        | Self::CopyOperations
                        | Self::CustomPresentPasses
                        | Self::Submissions
                        | Self::ResourceAllocationRequests
                        | Self::ImageContentHits
                        | Self::ImageContentMisses
                        | Self::ImageFirstObservedBytes
                        | Self::ImageIdentityCopies
                        | Self::ImageIdentityByteComparisons
                        | Self::ShaderBindingVisits
                        | Self::ShaderKeyCopies
                        | Self::EncodingStreamCopies
                        | Self::DashInitializations
                        | Self::RasterCommandsEmitted
                        | Self::RasterCommandVisits
                        | Self::RasterResourceCopies
                ) || (cfg!(any(
                    feature = "render-window",
                    all(feature = "render-web", target_arch = "wasm32")
                )) && matches!(self, Self::SurfaceAcquisitions | Self::Presentations))
            }
            RenderAttemptKind::SceneInspection => matches!(
                self,
                Self::StorageRelocations
                    | Self::CommandVisits
                    | Self::GlyphVisits
                    | Self::PaintVisits
                    | Self::ResourceProbes
                    | Self::AuthoredCommands
                    | Self::AuthoredFills
                    | Self::AuthoredStrokes
                    | Self::AuthoredShadows
                    | Self::AuthoredImages
                    | Self::AuthoredGlyphs
                    | Self::AuthoredLayers
                    | Self::ImageIdentityByteComparisons
                    | Self::ImageContentHits
                    | Self::ImageContentMisses
                    | Self::ImageFirstObservedBytes
            ),
            RenderAttemptKind::ImageRepeat => {
                matches!(self, Self::CoordinateVisits | Self::RepeatTiles)
            }
            RenderAttemptKind::Background => matches!(
                self,
                Self::StorageRelocations
                    | Self::CommandVisits
                    | Self::CommandsEmitted
                    | Self::FillsEmitted
                    | Self::ImagesEmitted
                    | Self::RepeatTiles
                    | Self::CoordinateVisits
                    | Self::PaintVisits
                    | Self::GradientStopVisits
                    | Self::PathElementVisits
                    | Self::ResourceProbes
                    | Self::ImageIdentityCopies
            ),
            RenderAttemptKind::Clip => matches!(self, Self::PathElementVisits),
            RenderAttemptKind::BoxDecoration => matches!(
                self,
                Self::StorageRelocations
                    | Self::CommandsEmitted
                    | Self::PaintVisits
                    | Self::GradientStopVisits
                    | Self::PathElementVisits
                    | Self::ResourceProbes
                    | Self::ImageIdentityCopies
            ),
        }
    }
}
dimensions! {
    /// Named resource units. See the crate accounting reference for units.
    #[non_exhaustive]
    pub enum RenderResource => RenderResourceTable {
        /// Live explicit result backing: actual Vec capacity bytes, font/image payloads and documented fixed result units.
        RetainedCpuBytes,
        /// Live actual temporary Vec capacity bytes and declared fixed decoder workspace.
        ScratchCpuBytes,
        /// Live descriptor-defined native buffer/texture payload bytes held by Render; not driver allocation or RSS.
        GpuBytes,
        /// Live observed opaque native handles and explicitly documented dependency utility aggregates.
        OpaqueGpuObjects,
        /// Tile records retained by one actual repeat call; nested calls renew this local quota.
        LocalRepeatTileQuota,
        /// Selected image source payload bytes for one actual atlas build.
        LocalAtlasSourceByteQuota,
        /// Selected expanded image-region texels for one actual atlas build.
        LocalAtlasTexelQuota,
        /// Classification of managed allocation bytes registered in cache entries; not additional physical storage.
        RegisteredCacheGpuBytes,
        /// Classification of managed allocation bytes eligible for idle retention; not additional physical storage.
        IdleCacheGpuBytes,
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
    /// Complete forward frame body before publication finalization.
    Frame,
    /// Validation, normalization, planning and encoding before backend execution.
    FramePrepare,
    /// Inclusive backend execution and waits; acquisition and presentation are child observations.
    Render,
    /// Authored scene inspection without normalization or rendering.
    SceneInspect,
    /// Authored-to-normalized command expansion and validation.
    Normalize,
    /// Tile geometry construction for one repeat call.
    Repeat,
    /// One repeat call's non-timed local quota owner.
    RepeatQuota,
    /// Allocating clip geometry normalization.
    Clip,
    /// Allocating box decoration normalization.
    Decoration,
    /// Frame and GPU pass graph planning.
    Plan,
    /// Graph lowering and closed-subset proof work.
    Lower,
    /// Raster or graph command encoding.
    Encode,
    /// Ordinary or repeating gradient realization in final paint-to-output space.
    GradientPrepare,
    /// Image packing and atlas source preparation.
    Atlas,
    /// One atlas build's non-timed local quota owner.
    AtlasQuota,
    /// Filter planning and parameter preparation.
    Filter,
    /// Gaussian kernel generation and serialization.
    Kernel,
    /// Backend shader, binding and native-resource preparation.
    GpuPrepare,
    /// Actual surface configuration and checked resolution.
    Configure,
    /// Actual host surface acquisition call.
    Acquire,
    /// Actual queue submission and checked resolution.
    Submit,
    /// Awaiting checked backend error scopes.
    Resolve,
    /// Actual presentation preparation and host presentation call.
    Present,
    /// Fallible history and metadata finalization before fixed publication moves.
    Publish,
    /// Actual renderer or generation initialization.
    Initialization,
    /// Explicit readback through its guarded pretransfer completion boundary.
    Readback,
    /// Readback descriptor validation, native construction and copy encoding,
    /// inclusive of its queue-submission child.
    ReadbackPrepare,
    /// Mapping registration, pending wait and completion, inclusive of decoding.
    ReadbackMap,
    /// Mapped row inspection, logical payload copy and guarded output validation.
    ReadbackDecode,
    /// Actual readback queue submission.
    ReadbackSubmit,
    /// Awaiting readback native error scopes.
    ReadbackResolve,
    /// Actual readback staging-owner disposal before interval closure.
    ReadbackCleanup,
}
impl RenderPhase {
    pub(crate) const ALL: [Self; 44] = [
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
        Self::Frame,
        Self::FramePrepare,
        Self::Render,
        Self::SceneInspect,
        Self::Normalize,
        Self::Repeat,
        Self::RepeatQuota,
        Self::Clip,
        Self::Decoration,
        Self::Plan,
        Self::Lower,
        Self::Encode,
        Self::GradientPrepare,
        Self::Atlas,
        Self::AtlasQuota,
        Self::Filter,
        Self::GpuPrepare,
        Self::Kernel,
        Self::Configure,
        Self::Present,
        Self::Acquire,
        Self::Submit,
        Self::Resolve,
        Self::Publish,
        Self::Initialization,
        Self::Readback,
        Self::ReadbackPrepare,
        Self::ReadbackMap,
        Self::ReadbackDecode,
        Self::ReadbackSubmit,
        Self::ReadbackResolve,
        Self::ReadbackCleanup,
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
            Self::Frame => Some(Self::Attempt),
            Self::FramePrepare => Some(Self::Frame),
            Self::Render => Some(Self::Frame),
            Self::SceneInspect => Some(Self::Attempt),
            Self::Normalize => Some(Self::FramePrepare),
            Self::Repeat => Some(Self::Normalize),
            Self::RepeatQuota => Some(Self::Repeat),
            Self::Clip => Some(Self::Normalize),
            Self::Decoration => Some(Self::Normalize),
            Self::Plan => Some(Self::FramePrepare),
            Self::Lower => Some(Self::FramePrepare),
            Self::Encode => Some(Self::FramePrepare),
            Self::GradientPrepare => Some(Self::Encode),
            Self::Atlas => Some(Self::Encode),
            Self::AtlasQuota => Some(Self::Atlas),
            Self::Filter => Some(Self::Plan),
            Self::Kernel => Some(Self::GpuPrepare),
            Self::GpuPrepare => Some(Self::Render),
            Self::Configure => Some(Self::Frame),
            Self::Acquire => Some(Self::Present),
            Self::Submit => Some(Self::Render),
            Self::Resolve => Some(Self::Render),
            Self::Present => Some(Self::Frame),
            Self::Publish => Some(Self::Frame),
            Self::Initialization => Some(Self::Attempt),
            Self::Readback => Some(Self::Attempt),
            Self::ReadbackPrepare
            | Self::ReadbackMap
            | Self::ReadbackResolve
            | Self::ReadbackCleanup => Some(Self::Readback),
            Self::ReadbackSubmit => Some(Self::ReadbackPrepare),
            Self::ReadbackDecode => Some(Self::ReadbackMap),
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
            Self::Frame => "frame",
            Self::FramePrepare => "frame_prepare",
            Self::Render => "render",
            Self::SceneInspect => "scene_inspect",
            Self::Normalize => "normalize",
            Self::Repeat => "repeat",
            Self::RepeatQuota => "repeat_quota",
            Self::Clip => "clip",
            Self::Decoration => "decoration",
            Self::Plan => "plan",
            Self::Lower => "lower",
            Self::Encode => "encode",
            Self::GradientPrepare => "gradient_prepare",
            Self::Atlas => "atlas",
            Self::AtlasQuota => "atlas_quota",
            Self::Filter => "filter",
            Self::Kernel => "kernel",
            Self::GpuPrepare => "gpu_prepare",
            Self::Configure => "configure",
            Self::Acquire => "acquire",
            Self::Submit => "submit",
            Self::Resolve => "resolve",
            Self::Present => "present",
            Self::Publish => "publish",
            Self::Initialization => "initialization",
            Self::Readback => "readback",
            Self::ReadbackPrepare => "readback_prepare",
            Self::ReadbackMap => "readback_map",
            Self::ReadbackDecode => "readback_decode",
            Self::ReadbackSubmit => "readback_submit",
            Self::ReadbackResolve => "readback_resolve",
            Self::ReadbackCleanup => "readback_cleanup",
        }
    }
}
pub(crate) const PHASE_COUNT: usize = RenderPhase::ALL.len();

/// General attempt limits in named units. Defaults are checked-unlimited.
/// Existing glyph/Text limits remain a separate preparation ancestor. Forward
/// work renews per operation. Renderer CPU caps cover simultaneous lifetime
/// backing, including persistent baseline, unpublished drafts and growth overlap.
/// Generation custody and mandatory cleanup remain checked-unlimited.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct RenderLimits {
    work: RenderWorkTable<u64>,
    retained: u64,
    scratch: u64,
}
impl RenderLimits {
    /// Checked-unlimited general limits; zero is a meaningful configured cap.
    pub const fn new() -> Self {
        Self {
            work: RenderWorkTable::splat(u64::MAX),
            retained: u64::MAX,
            scratch: u64::MAX,
        }
    }
    /// Sets an instrumented forward-work cap. Text-only units require `render-text`.
    /// Unsupported lifecycle/transfer units refuse here. Operation setup
    /// also rejects configured dimensions without producers at that entrypoint.
    pub fn with_work_limit(
        mut self,
        work: RenderWork,
        limit: u64,
    ) -> Result<Self, RenderAccountingError> {
        if !work.limit_supported() {
            return Err(RenderAccountingError::InvalidLimit(work));
        }
        *RenderWork::get_mut(&mut self.work, work) = limit;
        Ok(self)
    }
    /// Sets the total simultaneously admitted retained CPU backing byte cap.
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
    pub fn work_limit(self, work: RenderWork) -> u64 {
        *RenderWork::get(&self.work, work)
    }
    /// Returns the general retained-backing cap in bytes.
    pub const fn retained_bytes(self) -> u64 {
        self.retained
    }
    /// Returns the general scratch-backing cap in bytes.
    pub const fn scratch_bytes(self) -> u64 {
        self.scratch
    }
    pub(crate) fn validate(self, kind: RenderAttemptKind) -> Result<(), RenderAccountingError> {
        for &work in RenderWork::ALL {
            let supported =
                if matches!(kind, RenderAttemptKind::Frame | RenderAttemptKind::Readback) {
                    work.supported_for(RenderAttemptKind::Frame)
                        || work.supported_for(RenderAttemptKind::Readback)
                } else {
                    work.supported_for(kind)
                };
            if self.work_limit(work) != u64::MAX && !supported {
                return Err(RenderAccountingError::InvalidLimit(work));
            }
        }
        Ok(())
    }
    pub(crate) const fn ledger(self) -> surgeist_ledger::Limits<RenderWork, RenderResource> {
        surgeist_ledger::Limits::new(
            self.work,
            RenderResourceTable {
                RetainedCpuBytes: self.retained,
                ScratchCpuBytes: self.scratch,
                ..RenderResourceTable::splat(u64::MAX)
            },
        )
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
    /// Detail from one Renderer CPU lifetime role.
    RendererCpuScope(super::RenderCpuScope),
    /// Detail from one device generation role.
    GenerationScope(super::RenderGenerationScope),
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
                    RenderPhaseFilter::RendererCpuScope(_)
                    | RenderPhaseFilter::GenerationScope(_) => {
                        surgeist_ledger::EventFilter::ScopeName("__other_domain")
                    }
                },
            },
        }
    }
    pub(crate) fn ledger_cpu(self) -> surgeist_ledger::ProfileMode {
        self.ledger_domain(true)
    }
    pub(crate) fn ledger_generation(self) -> surgeist_ledger::ProfileMode {
        self.ledger_domain(false)
    }
    fn ledger_domain(self, cpu: bool) -> surgeist_ledger::ProfileMode {
        match self {
            Self::Detailed {
                timing,
                event_limit,
                filter,
            } => surgeist_ledger::ProfileMode::Detailed {
                timing,
                event_limit,
                filter: match filter {
                    RenderPhaseFilter::All => surgeist_ledger::EventFilter::All,
                    RenderPhaseFilter::RendererCpuScope(scope) if cpu => {
                        surgeist_ledger::EventFilter::ScopeName(scope.name())
                    }
                    RenderPhaseFilter::GenerationScope(scope) if !cpu => {
                        surgeist_ledger::EventFilter::ScopeName(scope.name())
                    }
                    _ => surgeist_ledger::EventFilter::ScopeName("__other_domain"),
                },
            },
            mode => mode.ledger(),
        }
    }
}
/// Standalone/frame accounting configuration. Default observation is off and
/// limits are checked-unlimited. Renderer options fix this for its lifetime.
#[derive(Clone, Copy, Debug, Default)]
pub struct RenderAccountingOptions {
    pub(crate) limits: RenderLimits,
    pub(crate) profile: RenderProfile,
    pub(crate) clock: RenderClock,
}
impl RenderAccountingOptions {
    /// Returns default configuration.
    pub const fn new() -> Self {
        Self {
            limits: RenderLimits::new(),
            profile: RenderProfile::Off,
            clock: RenderClock::Platform,
        }
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
    /// One complete Renderer frame attempt.
    Frame,
    /// One explicit headless readback, distinct from frame publication.
    Readback,
    /// Authored scene structural inspection.
    SceneInspection,
    /// Standalone image repetition.
    ImageRepeat,
    /// Standalone background normalization.
    Background,
    /// Standalone clip normalization.
    Clip,
    /// Standalone box decoration emission.
    BoxDecoration,
    /// Whole placed Text-layout preparation.
    TextLayout,
    /// Standalone native Paint geometry/used-stop consumption.
    PaintGradient,
    /// Standalone resolved color preparation for an SDR sRGB destination.
    SdrColor,
}
/// Semantic terminal outcome, separate from last-successful frame statistics.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[non_exhaustive]
pub enum RenderAttemptStatus {
    /// Standalone output transferred to its caller or a complete frame published.
    Succeeded,
    /// Named admission refused; accepted-prefix work remains.
    Refused,
    /// A semantic, provider, accounting or native error prevented publication.
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
    /// This dimension has no producer at the selected operation boundary.
    InvalidLimit(RenderWork),
    /// A private fixed-scope, checkpoint or participant contract was violated.
    Invariant,
    /// Ledger found a named internal invariant failure. Valid Ledger sequencing
    /// cannot produce this; the original provider diagnostic remains intact.
    LedgerDefect(surgeist_ledger::Defect),
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
    /// CPU backing admission or mandatory transition at the actual owning role.
    CpuRefused {
        /// Owning CPU role.
        scope: super::RenderCpuScope,
        /// Actual unit.
        quantity: RenderQuantity,
        /// Accepted prior usage.
        current: u64,
        /// Requested transition.
        amount: u64,
        /// Checked refusal classification.
        reason: RenderRefusalReason,
    },
    /// A fallible CPU allocation failed without admitting backing.
    CpuAllocation {
        /// Owning CPU role.
        scope: super::RenderCpuScope,
        /// Requested backing unit.
        resource: RenderResource,
    },
    /// Requested typed CPU backing cannot be represented in platform byte sizes;
    /// no allocation or custody transition was attempted.
    CpuCapacityOverflow {
        /// Owning CPU role.
        scope: super::RenderCpuScope,
        /// Requested backing unit.
        resource: RenderResource,
    },
}
impl std::fmt::Display for RenderAccountingError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{self:?}")
    }
}
impl std::error::Error for RenderAccountingError {}
/// Exact inclusive accepted usage. Parent and child values are attribution, not sums.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct RenderUsage(IntervalUsage<RenderWork, RenderResource>);
impl RenderUsage {
    pub(crate) const fn new(usage: IntervalUsage<RenderWork, RenderResource>) -> Self {
        Self(usage)
    }
    /// Accepted work in the named dimension's units.
    pub fn work(self, work: RenderWork) -> u64 {
        self.0.work(work)
    }
    /// Closing admitted custody in the named resource's units.
    pub fn live(self, resource: RenderResource) -> u64 {
        self.0.live(resource)
    }
    /// Exact interval maximum of admitted custody, including growth overlap.
    pub fn peak(self, resource: RenderResource) -> u64 {
        self.0.peak(resource)
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
/// Closed forward-work report, available with profiling off and after failures.
/// Standalone contexts retain one latest started attempt; a frame composes this
/// interval with its CPU and generation observations. Event detail is bounded.
#[derive(Debug, Eq, PartialEq)]
pub struct RenderAttemptReport {
    pub(crate) kind: RenderAttemptKind,
    pub(crate) status: RenderAttemptStatus,
    identity: AttemptIdentity,
    data: super::report::ResolvedReport<RenderPhase, super::report::IntervalNumbers>,
    pub(crate) refusal: Option<RenderRefusal>,
}

/// Terminal record for one standalone operation, including cancellation and
/// mandatory cleanup failure. Exact interval extraction may independently fail.
/// Take this record from its context before destroying the context to retain
/// the observation without starting another operation.
#[derive(Debug, Eq, PartialEq)]
pub struct RenderStandaloneReport {
    pub(crate) kind: RenderAttemptKind,
    pub(crate) status: RenderAttemptStatus,
    pub(crate) attempt: Result<RenderAttemptReport, RenderAccountingError>,
    pub(crate) cleanup_fault: Option<RenderAccountingError>,
}

impl RenderStandaloneReport {
    #[cfg(test)]
    pub(crate) fn snapshot_for_test(&self) -> Self {
        Self {
            kind: self.kind,
            status: self.status,
            attempt: self
                .attempt
                .as_ref()
                .map(RenderAttemptReport::snapshot_for_test)
                .map_err(|error| *error),
            cleanup_fault: self.cleanup_fault,
        }
    }
    /// Operation that produced this terminal record.
    pub const fn kind(&self) -> RenderAttemptKind {
        self.kind
    }

    /// Original semantic outcome or cancellation. Cleanup failure prevents a
    /// successful status but does not replace an earlier failure or cancellation.
    pub const fn status(&self) -> RenderAttemptStatus {
        self.status
    }

    /// Exact closed interval, or the typed fault that prevented its extraction.
    pub fn attempt(&self) -> Result<&RenderAttemptReport, RenderAccountingError> {
        self.attempt.as_ref().map_err(|error| *error)
    }

    /// First mandatory cleanup or finalization fault, separate from the semantic
    /// outcome and from interval extraction availability.
    pub const fn cleanup_fault(&self) -> Option<RenderAccountingError> {
        self.cleanup_fault
    }
}
#[derive(Clone, Copy, Eq, PartialEq)]
struct AttemptIdentity(Scope);
impl std::fmt::Debug for AttemptIdentity {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter.write_str("AttemptIdentity(..)")
    }
}
impl RenderAttemptReport {
    /// Resolves the complete interval against its owner's registered handles.
    pub(super) fn new(
        kind: RenderAttemptKind,
        status: RenderAttemptStatus,
        interval: IntervalReport<RenderWork, RenderResource>,
        refusal: Option<RenderRefusal>,
        handles: &[Scope],
        backing: super::report::ReportBacking<RenderPhase, super::report::IntervalNumbers>,
    ) -> Result<Self, RenderAccountingError> {
        let (root, data) = super::report::ResolvedReport::interval(interval, handles, backing)?;
        Ok(Self {
            kind,
            status,
            identity: AttemptIdentity(root),
            data,
            refusal,
        })
    }
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
        RenderUsage(self.data.rows[phase.index()].usage)
    }
    /// First refusal, including a noncommitting pre-allocation preview.
    pub const fn refusal(&self) -> Option<RenderRefusal> {
        self.refusal
    }
    /// Total measured interval, absent when off/untimed.
    pub fn elapsed(&self) -> Option<Duration> {
        self.data.elapsed
    }
    /// Sum of ended inclusive phase durations, absent when off/untimed.
    pub fn phase_elapsed(&self, phase: RenderPhase) -> Option<Duration> {
        self.data.rows[phase.index()].phase_elapsed
    }
    /// Number of ended invocations; zero with profiling off.
    pub fn phase_count(&self, phase: RenderPhase) -> u64 {
        self.data.rows[phase.index()].phases
    }
    /// Detail lost to the configured bounded buffer.
    pub fn dropped_events(&self) -> u64 {
        self.data.dropped_events
    }
    /// Whether optional observational counters saturated.
    pub fn saturated(&self) -> bool {
        self.data.saturated
    }
    /// Already resolved bounded detail in its original order.
    pub fn events(&self) -> impl Iterator<Item = RenderEvent> + '_ {
        self.data.events.iter().copied()
    }
    #[cfg(test)]
    pub(crate) fn snapshot_for_test(&self) -> Self {
        Self {
            kind: self.kind,
            status: self.status,
            identity: self.identity,
            data: self.data.snapshot_for_test(),
            refusal: self.refusal,
        }
    }
}
