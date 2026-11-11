use crate::accounting::{Account, AccountedVec, RenderWork, SharedValue};
#[cfg(test)]
use std::sync::atomic::{AtomicU64, Ordering};

use super::bounds::{
    DestinationToLayerLocalMapping, FrameSpatialPlan, LogicalBounds, NonEmptyFrameSpatialPlan,
    NonEmptyLogicalBounds, PositiveDeviceExtent, SemanticContributionDomain, SemanticSourceBounds,
    SemanticSourceContribution, SignedDeviceOrigin, TexelCenterMapping,
    destination_to_layer_local_mapping, mask_upload_spatial,
};
use super::filter::{
    FilterSourceRole, ResolvedFilterOperationIntent, ResolvedFilterStep, ResolvedFrameFilterPlan,
};
use super::{CompositeSourceSpace, FrameContext, GraphSelectionRequirement};
use crate::command::{
    LayerIsolation, NormalizedLayer, RenderClip, RenderCommand, RenderCommands, RenderLayerMask,
};
use crate::error::{BackendErrorCode, Error, Result};
use crate::geometry::{PhysicalSize, Rect, Transform};
use crate::image::{
    Extend, ImageQuality, ImageRendering, ResolvedMaskUploadDescriptor, ResolvedMaskUploadKey,
};
use crate::renderer::Antialiasing;
#[cfg(test)]
use crate::style::FilterList;
use crate::style::FilterOp;

#[cfg(test)]
static NEXT_GRAPH_GENERATION: AtomicU64 = AtomicU64::new(1);

pub(super) type GraphBuildResult<T> = std::result::Result<T, GraphValidationError>;

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub(super) struct GraphGeneration(pub(super) u64);

impl GraphGeneration {
    const FRAME_PLAN: Self = Self(1);

    #[cfg(test)]
    fn try_next() -> GraphBuildResult<Self> {
        NEXT_GRAPH_GENERATION
            .try_update(Ordering::Relaxed, Ordering::Relaxed, |generation| {
                generation.checked_add(1)
            })
            .map(Self)
            .map_err(|_| GraphValidationError::GenerationExhausted)
    }
}

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub(super) struct ResourceIndex(pub(super) u32);

impl ResourceIndex {
    pub(super) fn try_from_len(len: usize) -> GraphBuildResult<Self> {
        u32::try_from(len)
            .map(Self)
            .map_err(|_| GraphValidationError::ResourceIdentityExhausted)
    }

    pub(super) fn as_usize(self) -> GraphBuildResult<usize> {
        usize::try_from(self.0).map_err(|_| GraphValidationError::UnknownResourceIndex)
    }
}

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub(super) struct PassIndex(pub(super) u32);

impl PassIndex {
    pub(super) fn try_from_len(len: usize) -> GraphBuildResult<Self> {
        u32::try_from(len)
            .map(Self)
            .map_err(|_| GraphValidationError::PassIdentityExhausted)
    }

    pub(super) fn as_usize(self) -> GraphBuildResult<usize> {
        usize::try_from(self.0).map_err(|_| GraphValidationError::UnknownPassIndex)
    }
}

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub(super) struct SemanticResourceId {
    pub(super) generation: GraphGeneration,
    pub(super) index: ResourceIndex,
}

impl SemanticResourceId {
    pub(super) const fn new(generation: GraphGeneration, index: ResourceIndex) -> Self {
        Self { generation, index }
    }
}

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub(super) struct SemanticPassId {
    pub(super) generation: GraphGeneration,
    pub(super) index: PassIndex,
}

impl SemanticPassId {
    pub(super) const fn new(generation: GraphGeneration, index: PassIndex) -> Self {
        Self { generation, index }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(super) enum SemanticResourceRole {
    RootWorkingImage,
    CaptureWorkingImage,
    ClipCoverage,
    IsolationWorkingImage,
    ImportedImage,
    BackdropCopy,
    FilterIntermediate,
    ShadowImage,
    CompositeResult,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub(super) struct SemanticResourceDescriptor {
    pub(super) role: SemanticResourceRole,
    pub(super) logical_bounds: NonEmptyLogicalBounds,
    pub(super) device_origin: SignedDeviceOrigin,
    pub(super) device_extent: PositiveDeviceExtent,
    pub(super) texel_center_mapping: TexelCenterMapping,
    pub(super) expected_reads: u32,
}

impl SemanticResourceDescriptor {
    pub(super) const fn new(
        role: SemanticResourceRole,
        spatial: NonEmptyFrameSpatialPlan,
        expected_reads: u32,
    ) -> Self {
        Self {
            role,
            logical_bounds: spatial.logical_bounds,
            device_origin: spatial.device_origin,
            device_extent: spatial.device_extent,
            texel_center_mapping: spatial.texel_center_mapping,
            expected_reads,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub(super) enum WorkingImageInitialization {
    SurfaceBaseColor(crate::paint::Color),
    Transparent,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(super) enum BlurInput {
    Rgba,
    SourceAlpha,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub(super) enum SemanticPassIntent {
    ClearRoot {
        initialization: WorkingImageInitialization,
    },
    VelloCapture {
        initialization: WorkingImageInitialization,
    },
    CanonicalizeCapture,
    CopyBackdrop,
    ColorFilter,
    BlurHorizontal {
        input: BlurInput,
    },
    BlurVertical {
        input: BlurInput,
    },
    DropShadowColorize,
    Composite,
    Present,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(super) enum SemanticPassResult {
    Empty,
    Resource(SemanticResourceId),
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(super) enum SemanticResourceProducer {
    Imported,
    Pass(SemanticPassId),
}

#[derive(Clone, Debug, PartialEq)]
pub(super) struct SemanticGraphResource {
    pub(super) id: SemanticResourceId,
    pub(super) descriptor: SemanticResourceDescriptor,
    pub(super) producer: Option<SemanticResourceProducer>,
    pub(super) recorded_reads: u32,
    pub(super) remaining_reads: Option<u32>,
    pub(super) releasable_after: Option<SemanticPassId>,
}

#[derive(Clone, Debug, PartialEq)]
pub(super) struct SemanticGraphPass {
    pub(super) id: SemanticPassId,
    pub(super) intent: SemanticPassIntent,
    pub(super) dependencies: SharedValue<Vec<SemanticPassId>>,
    pub(super) reads: SharedValue<Vec<SemanticResourceId>>,
    pub(super) result: SemanticPassResult,
    pub(super) scheduled: bool,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum GraphBuildPhase {
    RecordingConsumers,
    Scheduling,
    FinalPresentScheduled,
}

#[cfg_attr(
    not(test),
    expect(
        dead_code,
        reason = "Typed graph failures remain available to graph-validation tests."
    )
)]
pub(super) enum GraphValidationError {
    Accounting(Error),
    GenerationExhausted,
    ResourceIdentityExhausted,
    PassIdentityExhausted,
    UnknownResourceIndex,
    UnknownPassIndex,
    WrongResourceGeneration {
        expected: GraphGeneration,
        actual: GraphGeneration,
    },
    WrongPassGeneration {
        expected: GraphGeneration,
        actual: GraphGeneration,
    },
    UnknownResource(SemanticResourceId),
    UnknownPass(SemanticPassId),
    ReleasedResource(SemanticResourceId),
    ForwardDependency(SemanticPassId),
    ForwardRead(SemanticResourceId),
    ReadWriteAlias(SemanticResourceId),
    DuplicateProducer(SemanticResourceId),
    DuplicateDependency(SemanticPassId),
    DuplicateRead(SemanticResourceId),
    MissingProducerDependency {
        resource: SemanticResourceId,
        producer: SemanticPassId,
    },
    ReadCountOverflow(SemanticResourceId),
    DeclaredReadCountMismatch {
        resource: SemanticResourceId,
        declared: u32,
        recorded: u32,
    },
    ResourceWithoutProducer(SemanticResourceId),
    OrphanResult(SemanticResourceId),
    MissingRootWorkingImage,
    DuplicateRootWorkingImage,
    MissingFinalPresent,
    DuplicateFinalPresent,
    DeclarationAfterFinalPresent,
    MissingSurfaceBaseInitialization,
    RepeatedSurfaceBaseInitialization,
    RootMustUseSurfaceBase,
    NonTransparentCaptureBase,
    InvalidClearRootResult,
    InvalidCaptureResult,
    InvalidImportedResourceRole,
    InvalidPassArity,
    InvalidPassResultRole,
    InvalidPresentIntent,
    RootProducedByNonClearPass,
    ConsumersNotSealed,
    ConsumersAlreadySealed,
    PassAlreadyScheduled(SemanticPassId),
    PresentScheduledBeforeOtherPasses(SemanticPassId),
    SchedulingAfterFinalPresent,
    UnscheduledDependency {
        pass: SemanticPassId,
        dependency: SemanticPassId,
    },
    UnscheduledProducer {
        resource: SemanticResourceId,
        producer: SemanticPassId,
    },
    UnscheduledPass(SemanticPassId),
    UnscheduledReads {
        resource: SemanticResourceId,
        remaining: u32,
    },
}

impl std::fmt::Debug for GraphValidationError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Accounting(value) => f.debug_tuple("Accounting").field(value).finish(),
            Self::GenerationExhausted => f.write_str("GenerationExhausted"),
            Self::ResourceIdentityExhausted => f.write_str("ResourceIdentityExhausted"),
            Self::PassIdentityExhausted => f.write_str("PassIdentityExhausted"),
            Self::UnknownResourceIndex => f.write_str("UnknownResourceIndex"),
            Self::UnknownPassIndex => f.write_str("UnknownPassIndex"),
            Self::WrongResourceGeneration { expected, actual } => f
                .debug_struct("WrongResourceGeneration")
                .field("expected", expected)
                .field("actual", actual)
                .finish(),
            Self::WrongPassGeneration { expected, actual } => f
                .debug_struct("WrongPassGeneration")
                .field("expected", expected)
                .field("actual", actual)
                .finish(),
            Self::UnknownResource(value) => f.debug_tuple("UnknownResource").field(value).finish(),
            Self::UnknownPass(value) => f.debug_tuple("UnknownPass").field(value).finish(),
            Self::ReleasedResource(value) => {
                f.debug_tuple("ReleasedResource").field(value).finish()
            }
            Self::ForwardDependency(value) => {
                f.debug_tuple("ForwardDependency").field(value).finish()
            }
            Self::ForwardRead(value) => f.debug_tuple("ForwardRead").field(value).finish(),
            Self::ReadWriteAlias(value) => f.debug_tuple("ReadWriteAlias").field(value).finish(),
            Self::DuplicateProducer(value) => {
                f.debug_tuple("DuplicateProducer").field(value).finish()
            }
            Self::DuplicateDependency(value) => {
                f.debug_tuple("DuplicateDependency").field(value).finish()
            }
            Self::DuplicateRead(value) => f.debug_tuple("DuplicateRead").field(value).finish(),
            Self::MissingProducerDependency { resource, producer } => f
                .debug_struct("MissingProducerDependency")
                .field("resource", resource)
                .field("producer", producer)
                .finish(),
            Self::ReadCountOverflow(value) => {
                f.debug_tuple("ReadCountOverflow").field(value).finish()
            }
            Self::DeclaredReadCountMismatch {
                resource,
                declared,
                recorded,
            } => f
                .debug_struct("DeclaredReadCountMismatch")
                .field("resource", resource)
                .field("declared", declared)
                .field("recorded", recorded)
                .finish(),
            Self::ResourceWithoutProducer(value) => f
                .debug_tuple("ResourceWithoutProducer")
                .field(value)
                .finish(),
            Self::OrphanResult(value) => f.debug_tuple("OrphanResult").field(value).finish(),
            Self::MissingRootWorkingImage => f.write_str("MissingRootWorkingImage"),
            Self::DuplicateRootWorkingImage => f.write_str("DuplicateRootWorkingImage"),
            Self::MissingFinalPresent => f.write_str("MissingFinalPresent"),
            Self::DuplicateFinalPresent => f.write_str("DuplicateFinalPresent"),
            Self::DeclarationAfterFinalPresent => f.write_str("DeclarationAfterFinalPresent"),
            Self::MissingSurfaceBaseInitialization => {
                f.write_str("MissingSurfaceBaseInitialization")
            }
            Self::RepeatedSurfaceBaseInitialization => {
                f.write_str("RepeatedSurfaceBaseInitialization")
            }
            Self::RootMustUseSurfaceBase => f.write_str("RootMustUseSurfaceBase"),
            Self::NonTransparentCaptureBase => f.write_str("NonTransparentCaptureBase"),
            Self::InvalidClearRootResult => f.write_str("InvalidClearRootResult"),
            Self::InvalidCaptureResult => f.write_str("InvalidCaptureResult"),
            Self::InvalidImportedResourceRole => f.write_str("InvalidImportedResourceRole"),
            Self::InvalidPassArity => f.write_str("InvalidPassArity"),
            Self::InvalidPassResultRole => f.write_str("InvalidPassResultRole"),
            Self::InvalidPresentIntent => f.write_str("InvalidPresentIntent"),
            Self::RootProducedByNonClearPass => f.write_str("RootProducedByNonClearPass"),
            Self::ConsumersNotSealed => f.write_str("ConsumersNotSealed"),
            Self::ConsumersAlreadySealed => f.write_str("ConsumersAlreadySealed"),
            Self::PassAlreadyScheduled(value) => {
                f.debug_tuple("PassAlreadyScheduled").field(value).finish()
            }
            Self::PresentScheduledBeforeOtherPasses(value) => f
                .debug_tuple("PresentScheduledBeforeOtherPasses")
                .field(value)
                .finish(),
            Self::SchedulingAfterFinalPresent => f.write_str("SchedulingAfterFinalPresent"),
            Self::UnscheduledDependency { pass, dependency } => f
                .debug_struct("UnscheduledDependency")
                .field("pass", pass)
                .field("dependency", dependency)
                .finish(),
            Self::UnscheduledProducer { resource, producer } => f
                .debug_struct("UnscheduledProducer")
                .field("resource", resource)
                .field("producer", producer)
                .finish(),
            Self::UnscheduledPass(value) => f.debug_tuple("UnscheduledPass").field(value).finish(),
            Self::UnscheduledReads {
                resource,
                remaining,
            } => f
                .debug_struct("UnscheduledReads")
                .field("resource", resource)
                .field("remaining", remaining)
                .finish(),
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(super) enum SemanticVelloSpanScope {
    CurrentParent,
    LayerSource,
}

#[derive(Clone, Debug, PartialEq)]
pub(super) struct SemanticVelloSpan {
    pub(super) capture_pass: SemanticPassId,
    pub(super) scope: SemanticVelloSpanScope,
    pub(super) source_space: CompositeSourceSpace,
    pub(super) commands: RenderCommands,
    pub(super) capture_transform: Transform,
    pub(super) parent_to_surface: Transform,
    pub(super) antialiasing: Antialiasing,
    pub(super) captured_before_outer_semantics: bool,
}

#[derive(Clone, Debug, PartialEq)]
pub(super) struct SemanticClipCoverage {
    pub(super) capture_pass: SemanticPassId,
    pub(super) elements: SharedValue<Vec<SemanticOuterClip>>,
    pub(super) antialiasing: Antialiasing,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub(super) struct SemanticResolvedAlphaMaskComposition {
    pub(super) resource: SemanticResourceId,
    pub(super) bounds: Rect,
    pub(super) image_dimensions: PhysicalSize,
    pub(super) quality: ImageQuality,
    pub(super) rendering: Option<ImageRendering>,
    pub(super) extend: Extend,
}

#[derive(Clone, Debug, PartialEq)]
pub(super) enum SemanticCompositeKind {
    SpanSourceOver,
    Layer {
        transform: Transform,
        destination_to_layer_local: DestinationToLayerLocalMapping,
        source_space: CompositeSourceSpace,
        opacity: f32,
        composition: crate::layer::LayerComposition,
        clip: Option<SharedValue<RenderClip>>,
        outer_clips: SharedValue<Vec<SemanticOuterClip>>,
        clip_coverage: Option<SemanticResourceId>,
        alpha_mask: Option<SharedValue<SemanticResolvedAlphaMaskComposition>>,
    },
    DropShadow,
}

#[derive(Clone, Debug, PartialEq)]
pub(super) struct SemanticCompositePlan {
    pub(super) pass: SemanticPassId,
    pub(super) kind: SemanticCompositeKind,
    pub(super) source_captured_before_outer_semantics: bool,
}

#[derive(Clone, Debug, PartialEq)]
pub(super) struct SemanticFilterStepPlan {
    pub(super) passes: SharedValue<Vec<SemanticPassId>>,
    pub(super) step: ResolvedFilterStep,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(super) struct SemanticBackdropRead {
    pub(super) pass: SemanticPassId,
    pub(super) completed_parent: SemanticResourceId,
}

#[derive(Clone, Debug, PartialEq)]
pub(super) enum SemanticImportKind {
    ResolvedAlphaMask {
        upload: SharedValue<ResolvedMaskUploadDescriptor>,
    },
}

#[derive(Clone, Debug, PartialEq)]
pub(super) struct SemanticImportPlan {
    pub(super) resource: SemanticResourceId,
    pub(super) kind: SemanticImportKind,
}

#[derive(Debug, PartialEq)]
pub(crate) struct GpuRenderGraph {
    pub(super) generation: GraphGeneration,
    pub(super) resources: AccountedVec<SemanticGraphResource>,
    pub(super) passes: AccountedVec<SemanticGraphPass>,
    pub(super) root_working_image: SemanticResourceId,
    pub(super) final_present: SemanticPassId,
    pub(super) selection_requirements: AccountedVec<GraphSelectionRequirement>,
    pub(super) vello_spans: AccountedVec<SemanticVelloSpan>,
    pub(super) clip_coverages: AccountedVec<SemanticClipCoverage>,
    pub(super) composites: AccountedVec<SemanticCompositePlan>,
    pub(super) filter_steps: AccountedVec<SemanticFilterStepPlan>,
    pub(super) backdrop_reads: AccountedVec<SemanticBackdropRead>,
    pub(super) imports: AccountedVec<SemanticImportPlan>,
}

impl GpuRenderGraph {
    pub(crate) fn has_foreground_blur(&self) -> bool {
        self.selection_requirements
            .contains(&GraphSelectionRequirement::ForegroundBlur)
    }
}

pub(super) struct SemanticGraphBuilder {
    account: Account,
    pub(super) generation: GraphGeneration,
    phase: GraphBuildPhase,
    pub(super) resources: AccountedVec<SemanticGraphResource>,
    pub(super) passes: AccountedVec<SemanticGraphPass>,
    root_working_image: Option<SemanticResourceId>,
    final_present: Option<SemanticPassId>,
    surface_base_initializations: u32,
}

#[cfg_attr(
    not(test),
    expect(
        dead_code,
        reason = "The private graph builder is consumed by graph planning and validation."
    )
)]
impl SemanticGraphBuilder {
    #[cfg(test)]
    pub(super) fn try_new(account: &Account) -> GraphBuildResult<Self> {
        Self::with_generation(GraphGeneration::try_next()?, account)
    }

    fn for_frame_plan(account: &Account) -> GraphBuildResult<Self> {
        Self::with_generation(GraphGeneration::FRAME_PLAN, account)
    }

    fn with_generation(generation: GraphGeneration, account: &Account) -> GraphBuildResult<Self> {
        Ok(Self {
            generation,
            account: account.clone(),
            phase: GraphBuildPhase::RecordingConsumers,
            resources: account.retained(0)?,
            passes: account.retained(0)?,
            root_working_image: None,
            final_present: None,
            surface_base_initializations: 0,
        })
    }

    pub(super) fn declare_resource(
        &mut self,
        descriptor: SemanticResourceDescriptor,
    ) -> GraphBuildResult<SemanticResourceId> {
        self.insert_resource(descriptor, None)
    }

    pub(super) fn import_resource(
        &mut self,
        descriptor: SemanticResourceDescriptor,
    ) -> GraphBuildResult<SemanticResourceId> {
        if descriptor.role != SemanticResourceRole::ImportedImage {
            return Err(GraphValidationError::InvalidImportedResourceRole);
        }
        self.insert_resource(descriptor, Some(SemanticResourceProducer::Imported))
    }

    fn insert_resource(
        &mut self,
        descriptor: SemanticResourceDescriptor,
        producer: Option<SemanticResourceProducer>,
    ) -> GraphBuildResult<SemanticResourceId> {
        self.require_recording_phase()?;
        if self.final_present.is_some() {
            return Err(GraphValidationError::DeclarationAfterFinalPresent);
        }
        self.account.work(RenderWork::GraphNodeVisits, 1)?;
        if descriptor.role == SemanticResourceRole::RootWorkingImage
            && self.root_working_image.is_some()
        {
            return Err(GraphValidationError::DuplicateRootWorkingImage);
        }

        let id = SemanticResourceId::new(
            self.generation,
            ResourceIndex::try_from_len(self.resources.len())?,
        );
        self.resources.push(SemanticGraphResource {
            id,
            descriptor,
            producer,
            recorded_reads: 0,
            remaining_reads: None,
            releasable_after: None,
        })?;
        if descriptor.role == SemanticResourceRole::RootWorkingImage {
            self.root_working_image = Some(id);
        }
        Ok(id)
    }

    pub(super) fn declare_pass(
        &mut self,
        intent: SemanticPassIntent,
        dependencies: &[SemanticPassId],
        reads: &[SemanticResourceId],
        result: SemanticPassResult,
    ) -> GraphBuildResult<SemanticPassId> {
        self.require_recording_phase()?;
        self.account.work(RenderWork::GraphNodeVisits, 1)?;
        if self.final_present.is_some() {
            return if intent == SemanticPassIntent::Present {
                Err(GraphValidationError::DuplicateFinalPresent)
            } else {
                Err(GraphValidationError::DeclarationAfterFinalPresent)
            };
        }
        let id = SemanticPassId::new(self.generation, PassIndex::try_from_len(self.passes.len())?);

        self.validate_recorded_dependencies(dependencies)?;
        let read_indices = self.validate_recorded_reads(dependencies, reads)?;

        let result_index = match result {
            SemanticPassResult::Empty => None,
            SemanticPassResult::Resource(resource) => {
                let resource_index = self.validate_resource_id(resource)?;
                if edge_contains(reads, &resource, &self.account)? {
                    return Err(GraphValidationError::ReadWriteAlias(resource));
                }
                if self
                    .resources
                    .get(resource_index)
                    .and_then(|resource| resource.producer)
                    .is_some()
                {
                    return Err(GraphValidationError::DuplicateProducer(resource));
                }
                Some(resource_index)
            }
        };

        self.validate_pass_shape(intent, dependencies, reads, result)?;

        let initializes_surface = matches!(
            intent,
            SemanticPassIntent::ClearRoot {
                initialization: WorkingImageInitialization::SurfaceBaseColor(_)
            }
        );
        let is_present = intent == SemanticPassIntent::Present;
        self.passes.grow(1)?;
        let owned_dependencies = shared_copy(dependencies, &self.account)?;
        let owned_reads = shared_copy(reads, &self.account)?;
        let resources = &mut self.resources;
        if let Some(resource_index) = result_index {
            let resource = resources.get_mut(resource_index).ok_or(match result {
                SemanticPassResult::Resource(resource) => {
                    GraphValidationError::UnknownResource(resource)
                }
                SemanticPassResult::Empty => GraphValidationError::UnknownResourceIndex,
            })?;
            resource.producer = Some(SemanticResourceProducer::Pass(id));
        }
        for (resource, resource_index) in reads.iter().zip(read_indices) {
            self.account.work(RenderWork::GraphEdgeVisits, 1)?;
            let graph_resource = resources
                .get_mut(resource_index)
                .ok_or(GraphValidationError::UnknownResource(*resource))?;
            graph_resource.recorded_reads = graph_resource
                .recorded_reads
                .checked_add(1)
                .ok_or(GraphValidationError::ReadCountOverflow(*resource))?;
        }

        let dependencies = owned_dependencies;
        let reads = owned_reads;
        self.passes.append_slot()?.write(SemanticGraphPass {
            id,
            intent,
            dependencies,
            reads,
            result,
            scheduled: false,
        });
        if initializes_surface {
            self.surface_base_initializations = 1;
        }
        if is_present {
            self.final_present = Some(id);
        }
        Ok(id)
    }

    fn validate_recorded_dependencies(
        &self,
        dependencies: &[SemanticPassId],
    ) -> GraphBuildResult<()> {
        let mut seen = self.account.temporary(dependencies.len())?;
        for dependency in dependencies {
            self.account.work(RenderWork::GraphEdgeVisits, 1)?;
            if edge_contains(&seen, dependency, &self.account)? {
                return Err(GraphValidationError::DuplicateDependency(*dependency));
            }
            self.validate_recorded_dependency(*dependency)?;
            seen.append_slot()?.write(*dependency);
        }
        Ok(())
    }

    fn validate_recorded_reads(
        &self,
        dependencies: &[SemanticPassId],
        reads: &[SemanticResourceId],
    ) -> GraphBuildResult<AccountedVec<usize>> {
        let mut indices = self.account.temporary(reads.len())?;
        let mut seen = self.account.temporary(reads.len())?;
        for resource in reads {
            self.account.work(RenderWork::GraphEdgeVisits, 1)?;
            if edge_contains(&seen, resource, &self.account)? {
                return Err(GraphValidationError::DuplicateRead(*resource));
            }
            let resource_index = self.validate_resource_id(*resource)?;
            let graph_resource = self
                .resources
                .get(resource_index)
                .ok_or(GraphValidationError::UnknownResource(*resource))?;
            match graph_resource.producer {
                Some(SemanticResourceProducer::Imported) => {}
                Some(SemanticResourceProducer::Pass(producer))
                    if !edge_contains(dependencies, &producer, &self.account)? =>
                {
                    return Err(GraphValidationError::MissingProducerDependency {
                        resource: *resource,
                        producer,
                    });
                }
                Some(SemanticResourceProducer::Pass(_)) => {}
                None => return Err(GraphValidationError::ForwardRead(*resource)),
            }
            graph_resource
                .recorded_reads
                .checked_add(1)
                .ok_or(GraphValidationError::ReadCountOverflow(*resource))?;
            seen.append_slot()?.write(*resource);
            indices.append_slot()?.write(resource_index);
        }
        Ok(indices)
    }

    fn validate_pass_shape(
        &self,
        intent: SemanticPassIntent,
        dependencies: &[SemanticPassId],
        reads: &[SemanticResourceId],
        result: SemanticPassResult,
    ) -> GraphBuildResult<()> {
        match intent {
            SemanticPassIntent::ClearRoot { initialization } => {
                self.validate_clear_root_shape(initialization, dependencies, reads, result)?
            }
            SemanticPassIntent::VelloCapture { initialization } => {
                self.validate_capture_shape(initialization, reads, result)?;
            }
            SemanticPassIntent::Present => {
                if self.final_present.is_some() {
                    return Err(GraphValidationError::DuplicateFinalPresent);
                }
                if reads.len() != 1 || result != SemanticPassResult::Empty {
                    return Err(GraphValidationError::InvalidPresentIntent);
                }
            }
            SemanticPassIntent::CanonicalizeCapture
            | SemanticPassIntent::CopyBackdrop
            | SemanticPassIntent::ColorFilter
            | SemanticPassIntent::BlurHorizontal { .. }
            | SemanticPassIntent::BlurVertical { .. }
            | SemanticPassIntent::DropShadowColorize
            | SemanticPassIntent::Composite => {
                self.validate_transform_pass_shape(intent, reads, result)?;
            }
        }
        Ok(())
    }

    fn validate_clear_root_shape(
        &self,
        initialization: WorkingImageInitialization,
        dependencies: &[SemanticPassId],
        reads: &[SemanticResourceId],
        result: SemanticPassResult,
    ) -> GraphBuildResult<()> {
        if matches!(
            initialization,
            WorkingImageInitialization::SurfaceBaseColor(_)
        ) && self.surface_base_initializations != 0
        {
            return Err(GraphValidationError::RepeatedSurfaceBaseInitialization);
        }
        if !dependencies.is_empty() || !reads.is_empty() {
            return Err(GraphValidationError::InvalidClearRootResult);
        }
        let SemanticPassResult::Resource(resource) = result else {
            return Err(GraphValidationError::InvalidClearRootResult);
        };
        let resource_index = self.validate_resource_id(resource)?;
        let Some(resource) = self.resources.get(resource_index) else {
            return Err(GraphValidationError::InvalidClearRootResult);
        };
        match (initialization, resource.descriptor.role) {
            (
                WorkingImageInitialization::SurfaceBaseColor(_),
                SemanticResourceRole::RootWorkingImage,
            )
            | (
                WorkingImageInitialization::Transparent,
                SemanticResourceRole::IsolationWorkingImage,
            ) => Ok(()),
            (WorkingImageInitialization::Transparent, _)
            | (
                WorkingImageInitialization::SurfaceBaseColor(_),
                SemanticResourceRole::IsolationWorkingImage,
            ) => Err(GraphValidationError::RootMustUseSurfaceBase),
            (WorkingImageInitialization::SurfaceBaseColor(_), _) => {
                Err(GraphValidationError::InvalidClearRootResult)
            }
        }
    }

    fn validate_capture_shape(
        &self,
        initialization: WorkingImageInitialization,
        reads: &[SemanticResourceId],
        result: SemanticPassResult,
    ) -> GraphBuildResult<()> {
        if initialization != WorkingImageInitialization::Transparent {
            return Err(GraphValidationError::NonTransparentCaptureBase);
        }
        if !reads.is_empty() {
            return Err(GraphValidationError::InvalidCaptureResult);
        }
        if let SemanticPassResult::Resource(resource) = result {
            let resource_index = self.validate_resource_id(resource)?;
            if self.resources.get(resource_index).is_none_or(|resource| {
                !matches!(
                    resource.descriptor.role,
                    SemanticResourceRole::CaptureWorkingImage
                        | SemanticResourceRole::ClipCoverage
                        | SemanticResourceRole::IsolationWorkingImage
                )
            }) {
                return Err(GraphValidationError::InvalidCaptureResult);
            }
        }
        Ok(())
    }

    fn validate_transform_pass_shape(
        &self,
        intent: SemanticPassIntent,
        reads: &[SemanticResourceId],
        result: SemanticPassResult,
    ) -> GraphBuildResult<()> {
        let SemanticPassResult::Resource(resource) = result else {
            return if reads.is_empty() {
                Ok(())
            } else {
                Err(GraphValidationError::InvalidPassArity)
            };
        };
        let reads_are_valid = if intent == SemanticPassIntent::Composite {
            reads.len() >= 2
        } else {
            reads.len() == 1
        };
        if !reads_are_valid {
            return Err(GraphValidationError::InvalidPassArity);
        }
        let expected_role = match intent {
            SemanticPassIntent::CopyBackdrop => SemanticResourceRole::BackdropCopy,
            SemanticPassIntent::DropShadowColorize => SemanticResourceRole::ShadowImage,
            SemanticPassIntent::Composite => SemanticResourceRole::CompositeResult,
            SemanticPassIntent::CanonicalizeCapture
            | SemanticPassIntent::ColorFilter
            | SemanticPassIntent::BlurHorizontal { .. }
            | SemanticPassIntent::BlurVertical { .. } => SemanticResourceRole::FilterIntermediate,
            _ => return Err(GraphValidationError::InvalidPassResultRole),
        };
        let resource_index = self.validate_resource_id(resource)?;
        if self
            .resources
            .get(resource_index)
            .is_none_or(|resource| resource.descriptor.role != expected_role)
        {
            return Err(GraphValidationError::InvalidPassResultRole);
        }
        Ok(())
    }

    fn seal_recorded_read_counts(&mut self) -> GraphBuildResult<()> {
        self.require_recording_phase()?;
        for resource in self.resources.iter_mut() {
            self.account.work(RenderWork::GraphNodeVisits, 1)?;
            resource.descriptor.expected_reads = resource.recorded_reads;
        }
        Ok(())
    }

    pub(super) fn begin_scheduling(&mut self) -> GraphBuildResult<()> {
        self.require_recording_phase()?;
        let root = self
            .root_working_image
            .ok_or(GraphValidationError::MissingRootWorkingImage)?;
        self.final_present
            .ok_or(GraphValidationError::MissingFinalPresent)?;
        if self.surface_base_initializations == 0 {
            return Err(GraphValidationError::MissingSurfaceBaseInitialization);
        }
        if self.surface_base_initializations != 1 {
            return Err(GraphValidationError::RepeatedSurfaceBaseInitialization);
        }

        for resource in &self.resources {
            self.account.work(RenderWork::GraphNodeVisits, 1)?;
            let producer = resource
                .producer
                .ok_or(GraphValidationError::ResourceWithoutProducer(resource.id))?;
            if resource.id == root
                && !matches!(producer, SemanticResourceProducer::Pass(pass) if self
                    .passes
                    .get(pass.index.as_usize()?)
                    .is_some_and(|pass| matches!(pass.intent, SemanticPassIntent::ClearRoot { .. })))
            {
                return Err(GraphValidationError::RootProducedByNonClearPass);
            }
            if resource.descriptor.expected_reads != resource.recorded_reads {
                return Err(GraphValidationError::DeclaredReadCountMismatch {
                    resource: resource.id,
                    declared: resource.descriptor.expected_reads,
                    recorded: resource.recorded_reads,
                });
            }
            if resource.recorded_reads == 0 {
                return Err(GraphValidationError::OrphanResult(resource.id));
            }
        }

        for pass in &self.passes {
            self.account.work(RenderWork::GraphNodeVisits, 1)?;
            if let SemanticPassIntent::VelloCapture { initialization } = pass.intent
                && initialization != WorkingImageInitialization::Transparent
            {
                return Err(GraphValidationError::NonTransparentCaptureBase);
            }
        }

        for resource in self.resources.iter_mut() {
            self.account.work(RenderWork::GraphNodeVisits, 1)?;
            resource.remaining_reads = Some(resource.descriptor.expected_reads);
        }
        self.phase = GraphBuildPhase::Scheduling;
        Ok(())
    }

    pub(super) fn schedule_pass(&mut self, id: SemanticPassId) -> GraphBuildResult<()> {
        match self.phase {
            GraphBuildPhase::RecordingConsumers => {
                return Err(GraphValidationError::ConsumersNotSealed);
            }
            GraphBuildPhase::Scheduling => {}
            GraphBuildPhase::FinalPresentScheduled => {
                return Err(GraphValidationError::SchedulingAfterFinalPresent);
            }
        }
        let pass_index = self.validate_existing_pass_id(id)?;
        let pass = self
            .passes
            .get(pass_index)
            .ok_or(GraphValidationError::UnknownPass(id))?;
        if pass.scheduled {
            return Err(GraphValidationError::PassAlreadyScheduled(id));
        }
        let is_present = pass.intent == SemanticPassIntent::Present;
        if is_present {
            self.validate_present_schedule(id, &pass.reads)?;
        }
        for dependency in pass.dependencies.iter() {
            self.account.work(RenderWork::GraphEdgeVisits, 1)?;
            let dependency_index = self.validate_existing_pass_id(*dependency)?;
            if self
                .passes
                .get(dependency_index)
                .is_none_or(|dependency| !dependency.scheduled)
            {
                return Err(GraphValidationError::UnscheduledDependency {
                    pass: id,
                    dependency: *dependency,
                });
            }
        }

        let read_indices = self.validate_scheduled_reads(&pass.reads)?;

        let resources = &mut self.resources;
        for (resource, resource_index) in pass.reads.iter().zip(read_indices) {
            self.account.work(RenderWork::GraphEdgeVisits, 1)?;
            let graph_resource = resources
                .get_mut(resource_index)
                .ok_or(GraphValidationError::UnknownResource(*resource))?;
            let remaining = graph_resource
                .remaining_reads
                .and_then(|remaining| remaining.checked_sub(1))
                .ok_or(GraphValidationError::ReleasedResource(*resource))?;
            graph_resource.remaining_reads = Some(remaining);
            if remaining == 0 {
                graph_resource.releasable_after = Some(id);
            }
        }
        let scheduled_pass = self
            .passes
            .get_mut(pass_index)
            .ok_or(GraphValidationError::UnknownPass(id))?;
        scheduled_pass.scheduled = true;
        if is_present {
            self.phase = GraphBuildPhase::FinalPresentScheduled;
        }
        Ok(())
    }

    fn validate_present_schedule(
        &self,
        id: SemanticPassId,
        present_reads: &[SemanticResourceId],
    ) -> GraphBuildResult<()> {
        for candidate in self.passes.iter() {
            self.account.work(RenderWork::GraphNodeVisits, 1)?;
            if candidate.id != id && !candidate.scheduled {
                return Err(GraphValidationError::PresentScheduledBeforeOtherPasses(
                    candidate.id,
                ));
            }
        }
        for resource in &self.resources {
            self.account.work(RenderWork::GraphNodeVisits, 1)?;
            let required_by_present =
                u32::from(edge_contains(present_reads, &resource.id, &self.account)?);
            match resource.remaining_reads {
                Some(remaining) if remaining == required_by_present => {}
                Some(remaining) => {
                    return Err(GraphValidationError::UnscheduledReads {
                        resource: resource.id,
                        remaining,
                    });
                }
                None => return Err(GraphValidationError::ConsumersNotSealed),
            }
        }
        Ok(())
    }

    fn validate_scheduled_reads(
        &self,
        reads: &[SemanticResourceId],
    ) -> GraphBuildResult<AccountedVec<usize>> {
        let mut indices = self.account.temporary(reads.len())?;
        for resource in reads {
            self.account.work(RenderWork::GraphEdgeVisits, 1)?;
            let resource_index = self.validate_resource_id(*resource)?;
            let graph_resource = self
                .resources
                .get(resource_index)
                .ok_or(GraphValidationError::UnknownResource(*resource))?;
            match graph_resource.producer {
                Some(SemanticResourceProducer::Imported) => {}
                Some(SemanticResourceProducer::Pass(producer)) => {
                    let producer_index = self.validate_existing_pass_id(producer)?;
                    if self
                        .passes
                        .get(producer_index)
                        .is_none_or(|producer| !producer.scheduled)
                    {
                        return Err(GraphValidationError::UnscheduledProducer {
                            resource: *resource,
                            producer,
                        });
                    }
                }
                None => return Err(GraphValidationError::ForwardRead(*resource)),
            }
            match graph_resource.remaining_reads {
                Some(0) => return Err(GraphValidationError::ReleasedResource(*resource)),
                Some(_) => {}
                None => return Err(GraphValidationError::ConsumersNotSealed),
            }
            indices.append_slot()?.write(resource_index);
        }
        Ok(indices)
    }

    pub(super) fn ensure_resource_readable(
        &self,
        resource: SemanticResourceId,
    ) -> GraphBuildResult<()> {
        if self.phase == GraphBuildPhase::RecordingConsumers {
            return Err(GraphValidationError::ConsumersNotSealed);
        }
        let resource_index = self.validate_resource_id(resource)?;
        match self
            .resources
            .get(resource_index)
            .ok_or(GraphValidationError::UnknownResource(resource))?
            .remaining_reads
        {
            Some(0) => Err(GraphValidationError::ReleasedResource(resource)),
            Some(_) => Ok(()),
            None => Err(GraphValidationError::ConsumersNotSealed),
        }
    }

    pub(super) fn finish(self) -> GraphBuildResult<GpuRenderGraph> {
        if self.phase == GraphBuildPhase::RecordingConsumers {
            return Err(GraphValidationError::ConsumersNotSealed);
        }
        for pass in &self.passes {
            self.account.work(RenderWork::GraphNodeVisits, 1)?;
            if !pass.scheduled {
                return Err(GraphValidationError::UnscheduledPass(pass.id));
            }
        }
        for resource in &self.resources {
            self.account.work(RenderWork::GraphNodeVisits, 1)?;
            match resource.remaining_reads {
                Some(0) if resource.releasable_after.is_some() => {}
                Some(remaining) => {
                    return Err(GraphValidationError::UnscheduledReads {
                        resource: resource.id,
                        remaining,
                    });
                }
                None => return Err(GraphValidationError::ConsumersNotSealed),
            }
        }
        let root_working_image = self
            .root_working_image
            .ok_or(GraphValidationError::MissingRootWorkingImage)?;
        let final_present = self
            .final_present
            .ok_or(GraphValidationError::MissingFinalPresent)?;
        if self.phase != GraphBuildPhase::FinalPresentScheduled {
            return Err(GraphValidationError::UnscheduledPass(final_present));
        }
        Ok(GpuRenderGraph {
            generation: self.generation,
            resources: self.resources,
            passes: self.passes,
            root_working_image,
            final_present,
            selection_requirements: self.account.retained(0)?,
            vello_spans: self.account.retained(0)?,
            clip_coverages: self.account.retained(0)?,
            composites: self.account.retained(0)?,
            filter_steps: self.account.retained(0)?,
            backdrop_reads: self.account.retained(0)?,
            imports: self.account.retained(0)?,
        })
    }

    fn require_recording_phase(&self) -> GraphBuildResult<()> {
        match self.phase {
            GraphBuildPhase::RecordingConsumers => Ok(()),
            GraphBuildPhase::Scheduling | GraphBuildPhase::FinalPresentScheduled => {
                Err(GraphValidationError::ConsumersAlreadySealed)
            }
        }
    }

    pub(super) fn validate_resource_id(&self, id: SemanticResourceId) -> GraphBuildResult<usize> {
        self.account.work(RenderWork::GraphNodeVisits, 1)?;
        if id.generation != self.generation {
            return Err(GraphValidationError::WrongResourceGeneration {
                expected: self.generation,
                actual: id.generation,
            });
        }
        let index = id.index.as_usize()?;
        if self.resources.get(index).is_none() {
            return Err(GraphValidationError::UnknownResource(id));
        }
        Ok(index)
    }

    fn validate_recorded_dependency(&self, id: SemanticPassId) -> GraphBuildResult<usize> {
        self.account.work(RenderWork::GraphNodeVisits, 1)?;
        if id.generation != self.generation {
            return Err(GraphValidationError::WrongPassGeneration {
                expected: self.generation,
                actual: id.generation,
            });
        }
        let index = id.index.as_usize()?;
        if index == self.passes.len() {
            return Err(GraphValidationError::ForwardDependency(id));
        }
        if self.passes.get(index).is_none() {
            return Err(GraphValidationError::UnknownPass(id));
        }
        Ok(index)
    }

    fn validate_existing_pass_id(&self, id: SemanticPassId) -> GraphBuildResult<usize> {
        self.account.work(RenderWork::GraphNodeVisits, 1)?;
        if id.generation != self.generation {
            return Err(GraphValidationError::WrongPassGeneration {
                expected: self.generation,
                actual: id.generation,
            });
        }
        let index = id.index.as_usize()?;
        if self.passes.get(index).is_none() {
            return Err(GraphValidationError::UnknownPass(id));
        }
        Ok(index)
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
struct PlannedGraphResource {
    id: SemanticResourceId,
    producer: Option<SemanticPassId>,
    logical_bounds: NonEmptyLogicalBounds,
    spatial: NonEmptyFrameSpatialPlan,
}

#[derive(Clone, Copy, Debug, PartialEq)]
struct PlannedGraphParent {
    current: PlannedGraphResource,
    spatial: NonEmptyFrameSpatialPlan,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum CaptureParentLocality {
    External,
    CaptureLocal,
}

#[derive(Clone, Debug, PartialEq)]
pub(super) struct SemanticOuterClip {
    pub(super) clip: RenderClip,
    pub(super) transform: Transform,
}

#[derive(Clone, Debug, PartialEq)]
struct SemanticCommandPlanningState {
    parent_locality: CaptureParentLocality,
    span_scope: SemanticVelloSpanScope,
    capture_transform: Transform,
    parent_to_surface: Transform,
    capture_space: CaptureBoundsCoordinateSpace,
    outer_clips: SharedValue<Vec<SemanticOuterClip>>,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(super) enum CaptureBoundsCoordinateSpace {
    Local,
    FilterLocal,
    SurfaceGrid,
    #[cfg(test)]
    ForcedMappedForTest,
}

pub(super) struct CaptureMapping {
    pub(super) capture_transform: Transform,
    pub(super) parent_to_surface: Transform,
    pub(super) bounds_coordinate_space: CaptureBoundsCoordinateSpace,
}

impl CaptureBoundsCoordinateSpace {
    fn resolve(
        self,
        bounds: NonEmptyLogicalBounds,
        capture_transform: Transform,
        raster_transform: Transform,
    ) -> Result<Option<NonEmptyLogicalBounds>> {
        let transform = match self {
            Self::Local => return Ok(Some(bounds)),
            Self::FilterLocal => capture_transform,
            Self::SurfaceGrid => raster_transform,
            #[cfg(test)]
            Self::ForcedMappedForTest => raster_transform,
        };
        match LogicalBounds::NonEmpty(bounds).try_transform(transform, "mapped capture bounds")? {
            LogicalBounds::Empty(_) => Ok(None),
            LogicalBounds::NonEmpty(bounds) => Ok(Some(bounds)),
        }
    }

    fn source_space(self) -> CompositeSourceSpace {
        match self {
            Self::SurfaceGrid => CompositeSourceSpace::SurfaceGrid,
            _ => CompositeSourceSpace::LayerLocal,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
struct VelloSpanFlush {
    parent: PlannedGraphParent,
    next_parent_locality: CaptureParentLocality,
}

pub(super) struct SemanticFrameGraphPlanner<'a> {
    account: &'a Account,
    context: FrameContext,
    builder: SemanticGraphBuilder,
    selection_requirements: AccountedVec<GraphSelectionRequirement>,
    vello_spans: AccountedVec<SemanticVelloSpan>,
    clip_coverages: AccountedVec<SemanticClipCoverage>,
    composites: AccountedVec<SemanticCompositePlan>,
    filter_steps: AccountedVec<SemanticFilterStepPlan>,
    backdrop_reads: AccountedVec<SemanticBackdropRead>,
    imports: AccountedVec<SemanticImportPlan>,
    resolved_mask_imports: AccountedVec<(ResolvedMaskUploadKey, PlannedGraphResource)>,
}

impl<'a> SemanticFrameGraphPlanner<'a> {
    pub(super) fn build(
        commands: RenderCommands,
        context: FrameContext,
        output_spatial: NonEmptyFrameSpatialPlan,
        selection_requirements: AccountedVec<GraphSelectionRequirement>,
        account: &'a Account,
    ) -> Result<GpuRenderGraph> {
        let capture_space = if commands_require_surface_grid(&commands.commands, account)? {
            CaptureBoundsCoordinateSpace::SurfaceGrid
        } else {
            CaptureBoundsCoordinateSpace::Local
        };
        Self::build_with_capture_mapping(
            commands,
            context,
            output_spatial,
            selection_requirements,
            CaptureMapping {
                capture_transform: Transform::identity(),
                parent_to_surface: Transform::identity(),
                bounds_coordinate_space: capture_space,
            },
            account,
        )
    }

    pub(super) fn build_with_capture_mapping(
        commands: RenderCommands,
        context: FrameContext,
        output_spatial: NonEmptyFrameSpatialPlan,
        selection_requirements: AccountedVec<GraphSelectionRequirement>,
        mapping: CaptureMapping,
        account: &'a Account,
    ) -> Result<GpuRenderGraph> {
        let mut planner = Self {
            context,
            account,
            builder: graph_build(SemanticGraphBuilder::for_frame_plan(account))?,
            selection_requirements,
            vello_spans: account.retained(0)?,
            clip_coverages: account.retained(0)?,
            composites: account.retained(0)?,
            filter_steps: account.retained(0)?,
            backdrop_reads: account.retained(0)?,
            imports: account.retained(0)?,
            resolved_mask_imports: account.retained(0)?,
        };
        let root_id = graph_build(planner.builder.declare_resource(
            SemanticResourceDescriptor::new(
                SemanticResourceRole::RootWorkingImage,
                output_spatial,
                0,
            ),
        ))?;
        let clear_root = graph_build(planner.builder.declare_pass(
            SemanticPassIntent::ClearRoot {
                initialization: WorkingImageInitialization::SurfaceBaseColor(context.base_color),
            },
            &[],
            &[],
            SemanticPassResult::Resource(root_id),
        ))?;
        let root = PlannedGraphResource {
            id: root_id,
            producer: Some(clear_root),
            logical_bounds: output_spatial.logical_bounds,
            spatial: output_spatial,
        };
        let parent = PlannedGraphParent {
            current: root,
            spatial: output_spatial,
        };
        let parent = planner.plan_commands(
            commands,
            parent,
            SemanticCommandPlanningState {
                parent_locality: CaptureParentLocality::External,
                span_scope: SemanticVelloSpanScope::CurrentParent,
                capture_transform: mapping.capture_transform,
                parent_to_surface: mapping.parent_to_surface,
                capture_space: mapping.bounds_coordinate_space,
                outer_clips: shared_copy(&[], account)?,
            },
        )?;
        graph_build(planner.builder.declare_pass(
            SemanticPassIntent::Present,
            &dependencies_for(&[parent.current], account)?,
            &[parent.current.id],
            SemanticPassResult::Empty,
        ))?;
        graph_build(planner.builder.seal_recorded_read_counts())?;
        graph_build(planner.builder.begin_scheduling())?;
        for index in 0..planner.builder.passes.len() {
            account.work(RenderWork::GraphNodeVisits, 1)?;
            let pass = planner.builder.passes[index].id;
            graph_build(planner.builder.schedule_pass(pass))?;
        }
        let mut graph = graph_build(planner.builder.finish())?;
        graph.selection_requirements = planner.selection_requirements;
        graph.vello_spans = planner.vello_spans;
        graph.clip_coverages = planner.clip_coverages;
        graph.composites = planner.composites;
        graph.filter_steps = planner.filter_steps;
        graph.backdrop_reads = planner.backdrop_reads;
        graph.imports = planner.imports;
        Ok(graph)
    }

    #[cfg(test)]
    pub(super) fn build_authored_filter_fixture(
        filters: Vec<FilterList>,
        commands: RenderCommands,
        context: FrameContext,
        account: &'a Account,
    ) -> Result<GpuRenderGraph> {
        if filters.is_empty() {
            return Err(Error::invalid_value(
                "authored filter fixture",
                0,
                "must begin with at least one authored FilterList",
            ));
        }
        let output_spatial = match context.output_spatial_plan()? {
            FrameSpatialPlan::NonEmpty(spatial) => spatial,
            FrameSpatialPlan::Empty(_) => {
                return Err(Error::invalid_value(
                    "authored filter fixture output bounds",
                    "empty",
                    "must be non-empty before the private graph fixture is planned",
                ));
            }
        };
        let mut planner = Self {
            context,
            account,
            builder: graph_build(SemanticGraphBuilder::for_frame_plan(account))?,
            selection_requirements: account.retained(0)?,
            vello_spans: account.retained(0)?,
            clip_coverages: account.retained(0)?,
            composites: account.retained(0)?,
            filter_steps: account.retained(0)?,
            backdrop_reads: account.retained(0)?,
            imports: account.retained(0)?,
            resolved_mask_imports: account.retained(0)?,
        };
        let root_id = graph_build(planner.builder.declare_resource(
            SemanticResourceDescriptor::new(
                SemanticResourceRole::RootWorkingImage,
                output_spatial,
                0,
            ),
        ))?;
        let clear_root = graph_build(planner.builder.declare_pass(
            SemanticPassIntent::ClearRoot {
                initialization: WorkingImageInitialization::SurfaceBaseColor(context.base_color),
            },
            &[],
            &[],
            SemanticPassResult::Resource(root_id),
        ))?;
        let root = PlannedGraphResource {
            id: root_id,
            producer: Some(clear_root),
            logical_bounds: output_spatial.logical_bounds,
            spatial: output_spatial,
        };
        let parent = PlannedGraphParent {
            current: root,
            spatial: output_spatial,
        };
        let mut source = planner
            .plan_layer_source(
                commands,
                Transform::identity(),
                CaptureBoundsCoordinateSpace::Local,
                None,
            )?
            .ok_or_else(|| {
                Error::invalid_value(
                    "authored filter fixture capture",
                    "empty",
                    "must contain ordinary capture input",
                )
            })?;
        for authored in &filters {
            source = planner.apply_filter_operations(
                source,
                authored.ops(),
                FilterSourceRole::Ordinary,
                Transform::identity(),
            )?;
        }
        let parent = planner.composite_into_parent(
            parent,
            source,
            &[],
            SemanticCompositeKind::Layer {
                transform: Transform::identity(),
                destination_to_layer_local: DestinationToLayerLocalMapping {
                    affine: Transform::identity(),
                },
                source_space: CompositeSourceSpace::LayerLocal,
                opacity: 1.0,
                composition: crate::layer::LayerComposition::from_blend(
                    crate::layer::BlendMode::Normal,
                ),
                clip: None,
                outer_clips: shared_copy(&[], account)?,
                clip_coverage: None,
                alpha_mask: None,
            },
            true,
        )?;
        Self::finish_authored_filter_fixture(planner, parent)
    }

    #[cfg(test)]
    fn finish_authored_filter_fixture(
        mut planner: Self,
        parent: PlannedGraphParent,
    ) -> Result<GpuRenderGraph> {
        let present = graph_build(planner.builder.declare_pass(
            SemanticPassIntent::Present,
            &dependencies_for(&[parent.current], planner.account)?,
            &[parent.current.id],
            SemanticPassResult::Empty,
        ))?;
        debug_assert_eq!(planner.builder.final_present, Some(present));

        graph_build(planner.builder.seal_recorded_read_counts())?;
        graph_build(planner.builder.begin_scheduling())?;
        for index in 0..planner.builder.passes.len() {
            planner.account.work(RenderWork::GraphNodeVisits, 1)?;
            let pass = planner.builder.passes[index].id;
            graph_build(planner.builder.schedule_pass(pass))?;
        }
        let mut graph = graph_build(planner.builder.finish())?;
        graph.vello_spans = planner.vello_spans;
        graph.clip_coverages = planner.clip_coverages;
        graph.composites = planner.composites;
        graph.filter_steps = planner.filter_steps;
        graph.backdrop_reads = planner.backdrop_reads;
        graph.imports = planner.imports;
        Ok(graph)
    }

    // Bounds have already been resolved into the frame's selected source
    // coordinates. SurfaceGrid allocation adds only device-pixel conversion;
    // the owning local-to-surface transform must never be applied a second time.
    fn plan_foreground_bounds(
        &self,
        bounds: NonEmptyLogicalBounds,
        local_to_surface: Transform,
        capture_space: CaptureBoundsCoordinateSpace,
    ) -> Result<FrameSpatialPlan> {
        match capture_space.source_space() {
            CompositeSourceSpace::LayerLocal => self
                .context
                .plan_local_bounds(LogicalBounds::NonEmpty(bounds), local_to_surface),
            CompositeSourceSpace::SurfaceGrid => self
                .context
                .plan_local_bounds(LogicalBounds::NonEmpty(bounds), Transform::identity()),
        }
    }

    fn plan_commands(
        &mut self,
        commands: RenderCommands,
        mut parent: PlannedGraphParent,
        mut state: SemanticCommandPlanningState,
    ) -> Result<PlannedGraphParent> {
        let mut span = self.account.retained(0)?;
        for command in commands.iter() {
            self.account.work(RenderWork::CommandVisits, 1)?;
            let command = command.clone();
            if command_is_local_to_capture(&command, state.parent_locality, self.account)? {
                span.push(command)?;
                continue;
            }

            let flush =
                self.flush_vello_span(RenderCommands::new_in(span, self.account)?, parent, &state)?;
            parent = flush.parent;
            state.parent_locality = flush.next_parent_locality;
            span = self.account.retained(0)?;
            let RenderCommand::Layer { layer, children } = command else {
                return Err(Error::new(
                    BackendErrorCode::RenderFailed,
                    "frame partition classified a non-layer command as a custom boundary",
                ));
            };
            parent = self.plan_external_layer(layer, children, parent, &state)?;
            state.parent_locality = CaptureParentLocality::External;
        }
        Ok(self
            .flush_vello_span(RenderCommands::new_in(span, self.account)?, parent, &state)?
            .parent)
    }

    fn flush_vello_span(
        &mut self,
        commands: RenderCommands,
        parent: PlannedGraphParent,
        state: &SemanticCommandPlanningState,
    ) -> Result<VelloSpanFlush> {
        let raster_transform = state.capture_transform.then(state.parent_to_surface)?;
        let contribution = SemanticSourceContribution::try_from_commands(
            commands,
            SemanticSourceBounds::exactly_empty(),
            SemanticContributionDomain::LocalUnbounded,
            self.context,
            raster_transform,
            self.account,
        )?;
        let Some(logical_bounds) = contribution
            .source_bounds
            .require_non_empty_for_graph("Vello span bounds")?
        else {
            return Ok(VelloSpanFlush {
                parent,
                next_parent_locality: state.parent_locality,
            });
        };
        let Some(logical_bounds) = state.capture_space.resolve(
            logical_bounds,
            state.capture_transform,
            raster_transform,
        )?
        else {
            return Ok(VelloSpanFlush {
                parent,
                next_parent_locality: state.parent_locality,
            });
        };
        let grid_to_surface = if state.capture_space == CaptureBoundsCoordinateSpace::FilterLocal {
            state.parent_to_surface
        } else {
            raster_transform
        };
        let spatial = match self.plan_foreground_bounds(
            logical_bounds,
            grid_to_surface,
            state.capture_space,
        )? {
            FrameSpatialPlan::Empty(_) => {
                return Ok(VelloSpanFlush {
                    parent,
                    next_parent_locality: state.parent_locality,
                });
            }
            FrameSpatialPlan::NonEmpty(spatial) => spatial,
        };
        let capture_resource = graph_build(self.builder.declare_resource(
            SemanticResourceDescriptor::new(SemanticResourceRole::CaptureWorkingImage, spatial, 0),
        ))?;
        let capture_pass = graph_build(self.builder.declare_pass(
            SemanticPassIntent::VelloCapture {
                initialization: WorkingImageInitialization::Transparent,
            },
            &[],
            &[],
            SemanticPassResult::Resource(capture_resource),
        ))?;
        let commands = contribution.commands;
        self.vello_spans.push(SemanticVelloSpan {
            capture_pass,
            scope: state.span_scope,
            source_space: state.capture_space.source_space(),
            commands,
            capture_transform: state.capture_transform,
            parent_to_surface: state.parent_to_surface,
            antialiasing: self.context.antialiasing,
            captured_before_outer_semantics: true,
        })?;
        let capture = PlannedGraphResource {
            id: capture_resource,
            producer: Some(capture_pass),
            logical_bounds: spatial.logical_bounds,
            spatial,
        };
        let canonical = self.declare_unary_resource_pass(
            capture,
            SemanticResourceRole::FilterIntermediate,
            spatial,
            SemanticPassIntent::CanonicalizeCapture,
        )?;
        let composite_kind = if state.outer_clips.is_empty() {
            SemanticCompositeKind::SpanSourceOver
        } else {
            SemanticCompositeKind::Layer {
                transform: Transform::identity(),
                destination_to_layer_local: DestinationToLayerLocalMapping {
                    affine: Transform::identity(),
                },
                source_space: state.capture_space.source_space(),
                opacity: 1.0,
                composition: crate::layer::LayerComposition::from_blend(
                    crate::layer::BlendMode::Normal,
                ),
                clip: None,
                outer_clips: state.outer_clips.clone(),
                clip_coverage: None,
                alpha_mask: None,
            }
        };
        let parent = self.composite_into_parent(parent, canonical, &[], composite_kind, true)?;
        Ok(VelloSpanFlush {
            parent,
            next_parent_locality: CaptureParentLocality::External,
        })
    }

    fn plan_external_layer(
        &mut self,
        layer: NormalizedLayer,
        children: RenderCommands,
        parent: PlannedGraphParent,
        state: &SemanticCommandPlanningState,
    ) -> Result<PlannedGraphParent> {
        let layer_transform = layer.transform.then(state.capture_transform)?;
        let layer_to_surface = layer_transform.then(state.parent_to_surface)?;
        let layer_to_destination =
            if state.capture_space == CaptureBoundsCoordinateSpace::FilterLocal {
                layer_transform
            } else {
                layer_to_surface
            };
        let inverse = destination_to_layer_local_mapping(layer_to_destination)?;
        // A singular own geometric clip has no operation coverage. A singular
        // source without that clip remains a transparent source operation.
        if inverse.is_none() && layer.clip.is_some() {
            return Ok(parent);
        }
        if layer.isolation == LayerIsolation::ClipOnly {
            let clip = layer.clip.clone().ok_or_else(|| {
                Error::new(
                    BackendErrorCode::RenderFailed,
                    "a clip-only layer reached graph planning without clip geometry",
                )
            })?;
            let mut child_state = state.clone();
            child_state.capture_transform = layer_transform;
            let mut clips = clone_list(&child_state.outer_clips, self.account)?;
            clips.push(SemanticOuterClip {
                clip,
                transform: layer_transform,
            })?;
            child_state.outer_clips = SharedValue::from_accounted(clips, &self.account.storage())?;
            return self.plan_commands(children, parent, child_state);
        }
        let is_transparent_wrapper = layer.clip.is_none()
            && !layer.explicit_isolation
            && layer.mask.is_none()
            && layer.backdrop.is_none()
            && layer.foreground_blur.is_none()
            && (layer.opacity - 1.0).abs() < f32::EPSILON
            && layer.composition.is_source_over_normal();
        if is_transparent_wrapper {
            let mut child_state = state.clone();
            child_state.capture_transform = layer_transform;
            return self.plan_commands(children, parent, child_state);
        }

        let bounded_domain = layer.explicit_isolation
            || (!layer.composition.is_source_over() && !layer.composition.is_plus())
            || commands_require_composition_domain(&children, self.account)?;
        let destination_to_layer_local = match inverse {
            Some(inverse) => inverse,
            None if bounded_domain => DestinationToLayerLocalMapping {
                affine: Transform::identity(),
            },
            None => return Ok(parent),
        };
        let source_forced_empty = inverse.is_none()
            || layer.opacity <= 0.0
            || layer
                .mask
                .as_ref()
                .is_some_and(crate::command::RenderLayerMask::annihilates_source);
        // The blur samples its own grid, even when the parent and other layers
        // use the final surface grid for pixel-sensitive drawing.
        let capture_space = if layer.foreground_blur.is_some() {
            CaptureBoundsCoordinateSpace::FilterLocal
        } else {
            state.capture_space
        };
        let finite_source = bounded_domain && layer.foreground_blur.is_none();
        let mut source = if source_forced_empty {
            Some(self.declare_transparent_parent(parent.spatial)?.current)
        } else {
            self.plan_layer_source(
                children,
                layer_to_surface,
                capture_space,
                finite_source.then_some((parent.spatial, layer_to_destination)),
            )?
        };
        if let Some(backdrop) = layer.backdrop.as_ref().filter(|_| !source_forced_empty) {
            source = self.plan_backdrop_group(
                backdrop,
                source,
                parent,
                layer_to_surface,
                capture_space.source_space(),
            )?;
        }
        if let Some(blur) = layer.foreground_blur
            && let Some(group) = source
            && !source_forced_empty
        {
            source = Some(self.apply_filter_operations(
                group,
                &[FilterOp::blur(blur)],
                FilterSourceRole::Ordinary,
                layer_to_surface,
            )?);
        }
        if source.is_none()
            && layer.foreground_blur.is_some()
            && !layer.composition.transparent_source_preserves_parent()
        {
            source = Some(self.declare_transparent_parent(parent.spatial)?.current);
        }
        let Some(source) = source else {
            return Ok(parent);
        };

        let alpha_mask = match layer.mask.as_ref().filter(|_| !source_forced_empty) {
            Some(mask) => {
                let imported = self.import_alpha_mask(mask)?;
                Some((
                    SemanticResolvedAlphaMaskComposition {
                        resource: imported.id,
                        bounds: mask.bounds(),
                        image_dimensions: mask.upload().physical_size(),
                        quality: mask.upload().quality(),
                        rendering: mask.upload().rendering_hint(),
                        extend: mask.upload().extend(),
                    },
                    imported,
                ))
            }
            None => None,
        };
        let additional_sources = alpha_mask.as_ref().map(|(_, resource)| *resource);
        self.composite_into_parent(
            parent,
            source,
            additional_sources.as_slice(),
            SemanticCompositeKind::Layer {
                transform: layer_transform,
                destination_to_layer_local,
                source_space: if source_forced_empty {
                    state.capture_space.source_space()
                } else {
                    capture_space.source_space()
                },
                opacity: layer.opacity,
                composition: layer.composition,
                clip: layer
                    .clip
                    .map(|clip| SharedValue::new(clip, &self.account.storage()))
                    .transpose()?,
                outer_clips: state.outer_clips.clone(),
                clip_coverage: None,
                alpha_mask: alpha_mask
                    .map(|(mask, _)| SharedValue::new(mask, &self.account.storage()))
                    .transpose()?,
            },
            true,
        )
    }

    fn plan_layer_source(
        &mut self,
        children: RenderCommands,
        raster_transform: Transform,
        capture_space: CaptureBoundsCoordinateSpace,
        finite_domain: Option<(NonEmptyFrameSpatialPlan, Transform)>,
    ) -> Result<Option<PlannedGraphResource>> {
        let domain = match finite_domain {
            Some((spatial, layer_to_destination)) => {
                let Some(inverse) = destination_to_layer_local_mapping(layer_to_destination)?
                else {
                    return Ok(Some(self.declare_transparent_parent(spatial)?.current));
                };
                SemanticContributionDomain::RootOutputBounded(
                    LogicalBounds::NonEmpty(spatial.logical_bounds)
                        .try_transform(inverse.affine, "finite group source domain")?,
                )
            }
            None => SemanticContributionDomain::LocalUnbounded,
        };
        let contribution = SemanticSourceContribution::try_from_commands(
            children,
            SemanticSourceBounds::exactly_empty(),
            domain,
            self.context,
            raster_transform,
            self.account,
        )?;
        if contribution.commands.is_empty() && finite_domain.is_none() {
            return Ok(None);
        }
        let spatial = if let Some((spatial, _)) = finite_domain
            && capture_space != CaptureBoundsCoordinateSpace::FilterLocal
        {
            spatial
        } else {
            let spatial_bounds = match domain {
                SemanticContributionDomain::RootOutputBounded(bounds) => {
                    SemanticSourceBounds::from_logical_bounds(bounds)
                }
                SemanticContributionDomain::LocalUnbounded => contribution.affected_bounds,
            };
            let Some(logical_bounds) =
                spatial_bounds.require_non_empty_for_graph("layer source bounds")?
            else {
                return Ok(None);
            };
            let logical_bounds = if capture_space == CaptureBoundsCoordinateSpace::SurfaceGrid {
                let Some(bounds) = capture_space.resolve(
                    logical_bounds,
                    Transform::identity(),
                    raster_transform,
                )?
                else {
                    return Ok(None);
                };
                bounds
            } else {
                logical_bounds
            };
            match self.plan_foreground_bounds(logical_bounds, raster_transform, capture_space)? {
                FrameSpatialPlan::Empty(_) => return Ok(None),
                FrameSpatialPlan::NonEmpty(spatial) => spatial,
            }
        };
        let source_parent = self.declare_transparent_parent(spatial)?;
        let source_parent = self.plan_commands(
            contribution.commands,
            source_parent,
            SemanticCommandPlanningState {
                parent_locality: CaptureParentLocality::CaptureLocal,
                span_scope: SemanticVelloSpanScope::LayerSource,
                capture_transform: if capture_space == CaptureBoundsCoordinateSpace::SurfaceGrid {
                    raster_transform
                } else {
                    Transform::identity()
                },
                parent_to_surface: if capture_space == CaptureBoundsCoordinateSpace::SurfaceGrid {
                    Transform::identity()
                } else {
                    raster_transform
                },
                capture_space,
                outer_clips: shared_copy(&[], self.account)?,
            },
        )?;
        Ok(Some(source_parent.current))
    }

    fn plan_backdrop_group(
        &mut self,
        backdrop: &crate::command::RenderBackdropCapture,
        foreground: Option<PlannedGraphResource>,
        parent: PlannedGraphParent,
        raster_transform: Transform,
        source_space: CompositeSourceSpace,
    ) -> Result<Option<PlannedGraphResource>> {
        let capture_bounds = LogicalBounds::try_from_rect(
            backdrop.capture_bounds().rect(),
            "backdrop capture bounds",
        )?;
        let capture_spatial = match self
            .context
            .plan_local_bounds(capture_bounds, raster_transform)?
        {
            FrameSpatialPlan::Empty(_) => return Ok(foreground),
            FrameSpatialPlan::NonEmpty(spatial) => spatial,
        };
        let copied_id = graph_build(self.builder.declare_resource(
            SemanticResourceDescriptor::new(SemanticResourceRole::BackdropCopy, capture_spatial, 0),
        ))?;
        let copy_pass = graph_build(self.builder.declare_pass(
            SemanticPassIntent::CopyBackdrop,
            &dependencies_for(&[parent.current], self.account)?,
            &[parent.current.id],
            SemanticPassResult::Resource(copied_id),
        ))?;
        self.backdrop_reads.push(SemanticBackdropRead {
            pass: copy_pass,
            completed_parent: parent.current.id,
        })?;
        let copied = PlannedGraphResource {
            id: copied_id,
            producer: Some(copy_pass),
            logical_bounds: capture_spatial.logical_bounds,
            spatial: capture_spatial,
        };
        let filtered = self.apply_filter_operations(
            copied,
            backdrop.filters().ops(),
            FilterSourceRole::Backdrop,
            raster_transform,
        )?;

        let mut backdrop_contribution = SemanticSourceBounds::exact_known(filtered.logical_bounds);
        if let Some(clip) = backdrop.clip() {
            backdrop_contribution = backdrop_contribution.try_intersect(
                SemanticSourceBounds::try_for_clip(clip, self.account)?,
                "post-filter backdrop clip intersection",
            )?;
        }
        let backdrop_bounds = backdrop_contribution
            .require_non_empty_for_graph("post-filter backdrop contribution bounds")?
            .ok_or_else(|| {
                Error::new(
                    BackendErrorCode::RenderFailed,
                    "post-filter backdrop contribution became empty after semantic planning",
                )
            })?;
        let group_bounds = match foreground {
            Some(foreground) => backdrop_bounds.try_union(
                foreground.logical_bounds,
                "backdrop foreground group bounds",
            )?,
            None => backdrop_bounds,
        };
        let group_spatial = match self
            .context
            .plan_local_bounds(LogicalBounds::NonEmpty(group_bounds), raster_transform)?
        {
            FrameSpatialPlan::Empty(_) => return Ok(None),
            FrameSpatialPlan::NonEmpty(spatial) => spatial,
        };
        let mut group = self.declare_transparent_parent(group_spatial)?;
        group = self.composite_into_parent(
            group,
            filtered,
            &[],
            SemanticCompositeKind::Layer {
                transform: Transform::identity(),
                destination_to_layer_local: DestinationToLayerLocalMapping {
                    affine: Transform::identity(),
                },
                source_space,
                opacity: 1.0,
                composition: crate::layer::LayerComposition::from_blend(
                    crate::layer::BlendMode::Normal,
                ),
                clip: backdrop
                    .clip()
                    .map(|clip| SharedValue::new(clip.clone(), &self.account.storage()))
                    .transpose()?,
                outer_clips: shared_copy(&[], self.account)?,
                clip_coverage: None,
                alpha_mask: None,
            },
            true,
        )?;
        if let Some(foreground) = foreground {
            group = self.composite_into_parent(
                group,
                foreground,
                &[],
                SemanticCompositeKind::SpanSourceOver,
                true,
            )?;
        }
        Ok(Some(group.current))
    }

    fn apply_filter_operations(
        &mut self,
        mut source: PlannedGraphResource,
        operations: &[FilterOp],
        source_role: FilterSourceRole,
        raster_transform: Transform,
    ) -> Result<PlannedGraphResource> {
        let plan = self.context.plan_filter_operations(
            LogicalBounds::NonEmpty(source.logical_bounds),
            raster_transform,
            operations,
            source_role,
            self.account,
        )?;
        let ResolvedFrameFilterPlan::NonEmpty(plan) = plan else {
            return Ok(source);
        };

        for step in plan.steps.iter().cloned() {
            self.account.work(RenderWork::FilterOperationVisits, 1)?;
            let mut passes = self.account.retained(4)?;
            match step.operation_intent {
                ResolvedFilterOperationIntent::ColorRun(_) => {
                    source = self.declare_unary_resource_pass(
                        source,
                        SemanticResourceRole::FilterIntermediate,
                        step.spatial_mapping.result,
                        SemanticPassIntent::ColorFilter,
                    )?;
                    passes
                        .append_slot()?
                        .write(planned_resource_producer(source)?);
                }
                ResolvedFilterOperationIntent::Blur(_) => {
                    let horizontal = self.declare_unary_resource_pass(
                        source,
                        SemanticResourceRole::FilterIntermediate,
                        step.spatial_mapping.result,
                        SemanticPassIntent::BlurHorizontal {
                            input: BlurInput::Rgba,
                        },
                    )?;
                    passes
                        .append_slot()?
                        .write(planned_resource_producer(horizontal)?);
                    source = self.declare_unary_resource_pass(
                        horizontal,
                        SemanticResourceRole::FilterIntermediate,
                        step.spatial_mapping.result,
                        SemanticPassIntent::BlurVertical {
                            input: BlurInput::Rgba,
                        },
                    )?;
                    passes
                        .append_slot()?
                        .write(planned_resource_producer(source)?);
                }
                ResolvedFilterOperationIntent::DropShadow(_) => {
                    let horizontal = self.declare_unary_resource_pass(
                        source,
                        SemanticResourceRole::FilterIntermediate,
                        step.spatial_mapping.result,
                        SemanticPassIntent::BlurHorizontal {
                            input: BlurInput::SourceAlpha,
                        },
                    )?;
                    passes
                        .append_slot()?
                        .write(planned_resource_producer(horizontal)?);
                    let vertical = self.declare_unary_resource_pass(
                        horizontal,
                        SemanticResourceRole::FilterIntermediate,
                        step.spatial_mapping.result,
                        SemanticPassIntent::BlurVertical {
                            input: BlurInput::SourceAlpha,
                        },
                    )?;
                    passes
                        .append_slot()?
                        .write(planned_resource_producer(vertical)?);
                    let shadow = self.declare_unary_resource_pass(
                        vertical,
                        SemanticResourceRole::ShadowImage,
                        step.spatial_mapping.result,
                        SemanticPassIntent::DropShadowColorize,
                    )?;
                    passes
                        .append_slot()?
                        .write(planned_resource_producer(shadow)?);
                    let result_id = graph_build(self.builder.declare_resource(
                        SemanticResourceDescriptor::new(
                            SemanticResourceRole::CompositeResult,
                            step.spatial_mapping.result,
                            0,
                        ),
                    ))?;
                    let merge = graph_build(self.builder.declare_pass(
                        SemanticPassIntent::Composite,
                        &dependencies_for(&[source, shadow], self.account)?,
                        &[source.id, shadow.id],
                        SemanticPassResult::Resource(result_id),
                    ))?;
                    passes.append_slot()?.write(merge);
                    self.composites.push(SemanticCompositePlan {
                        pass: merge,
                        kind: SemanticCompositeKind::DropShadow,
                        source_captured_before_outer_semantics: true,
                    })?;
                    source = PlannedGraphResource {
                        id: result_id,
                        producer: Some(merge),
                        logical_bounds: step.result_bounds,
                        spatial: step.spatial_mapping.result,
                    };
                }
            }
            self.filter_steps.push(SemanticFilterStepPlan {
                passes: SharedValue::from_accounted(passes, &self.account.storage())?,
                step,
            })?;
        }
        Ok(source)
    }

    fn declare_transparent_parent(
        &mut self,
        spatial: NonEmptyFrameSpatialPlan,
    ) -> Result<PlannedGraphParent> {
        let resource = graph_build(self.builder.declare_resource(
            SemanticResourceDescriptor::new(
                SemanticResourceRole::IsolationWorkingImage,
                spatial,
                0,
            ),
        ))?;
        let clear = graph_build(self.builder.declare_pass(
            SemanticPassIntent::ClearRoot {
                initialization: WorkingImageInitialization::Transparent,
            },
            &[],
            &[],
            SemanticPassResult::Resource(resource),
        ))?;
        Ok(PlannedGraphParent {
            current: PlannedGraphResource {
                id: resource,
                producer: Some(clear),
                logical_bounds: spatial.logical_bounds,
                spatial,
            },
            spatial,
        })
    }

    fn import_alpha_mask(&mut self, mask: &RenderLayerMask) -> Result<PlannedGraphResource> {
        let key = mask.upload().cache_key_in(self.account)?;
        for (existing, resource) in self.resolved_mask_imports.iter() {
            self.account.work(RenderWork::ResourceProbes, 1)?;
            if existing.matches_in(&key, Some(self.account))? {
                return Ok(*resource);
            }
        }
        let spatial = mask_upload_spatial(mask.upload().physical_size())?;
        let resource = graph_build(
            self.builder
                .import_resource(SemanticResourceDescriptor::new(
                    SemanticResourceRole::ImportedImage,
                    spatial,
                    0,
                )),
        )?;
        self.imports.push(SemanticImportPlan {
            resource,
            kind: SemanticImportKind::ResolvedAlphaMask {
                upload: mask.shared_upload(),
            },
        })?;
        let planned = PlannedGraphResource {
            id: resource,
            producer: None,
            logical_bounds: spatial.logical_bounds,
            spatial,
        };
        self.resolved_mask_imports.push((key, planned))?;
        Ok(planned)
    }

    fn declare_unary_resource_pass(
        &mut self,
        source: PlannedGraphResource,
        role: SemanticResourceRole,
        spatial: NonEmptyFrameSpatialPlan,
        intent: SemanticPassIntent,
    ) -> Result<PlannedGraphResource> {
        let resource = graph_build(
            self.builder
                .declare_resource(SemanticResourceDescriptor::new(role, spatial, 0)),
        )?;
        let pass = graph_build(self.builder.declare_pass(
            intent,
            &dependencies_for(&[source], self.account)?,
            &[source.id],
            SemanticPassResult::Resource(resource),
        ))?;
        Ok(PlannedGraphResource {
            id: resource,
            producer: Some(pass),
            logical_bounds: spatial.logical_bounds,
            spatial,
        })
    }

    fn declare_clip_coverage(
        &mut self,
        spatial: NonEmptyFrameSpatialPlan,
        elements: AccountedVec<SemanticOuterClip>,
    ) -> Result<PlannedGraphResource> {
        if elements.is_empty() {
            return Err(Error::new(
                BackendErrorCode::RenderFailed,
                "clip coverage requires at least one ordered RenderClip",
            ));
        }
        let resource = graph_build(self.builder.declare_resource(
            SemanticResourceDescriptor::new(SemanticResourceRole::ClipCoverage, spatial, 0),
        ))?;
        let capture_pass = graph_build(self.builder.declare_pass(
            SemanticPassIntent::VelloCapture {
                initialization: WorkingImageInitialization::Transparent,
            },
            &[],
            &[],
            SemanticPassResult::Resource(resource),
        ))?;
        self.clip_coverages.push(SemanticClipCoverage {
            capture_pass,
            elements: SharedValue::from_accounted(elements, &self.account.storage())?,
            antialiasing: self.context.antialiasing,
        })?;
        Ok(PlannedGraphResource {
            id: resource,
            producer: Some(capture_pass),
            logical_bounds: spatial.logical_bounds,
            spatial,
        })
    }

    fn composite_into_parent(
        &mut self,
        parent: PlannedGraphParent,
        source: PlannedGraphResource,
        additional_sources: &[PlannedGraphResource],
        mut kind: SemanticCompositeKind,
        source_captured_before_outer_semantics: bool,
    ) -> Result<PlannedGraphParent> {
        let clip_elements = match &kind {
            SemanticCompositeKind::Layer {
                transform,
                clip,
                outer_clips,
                clip_coverage,
                ..
            } => {
                if clip_coverage.is_some() {
                    return Err(Error::new(
                        BackendErrorCode::RenderFailed,
                        "clip coverage must be owned by graph composition planning",
                    ));
                }
                let mut elements = clone_list(outer_clips, self.account)?;
                if let Some(clip) = clip {
                    elements.push(SemanticOuterClip {
                        clip: (**clip).clone(),
                        transform: *transform,
                    })?;
                }
                elements
            }
            SemanticCompositeKind::SpanSourceOver | SemanticCompositeKind::DropShadow => {
                self.account.retained(0)?
            }
        };
        let clip_coverage = if clip_elements.is_empty() {
            None
        } else {
            Some(self.declare_clip_coverage(parent.spatial, clip_elements)?)
        };
        if let SemanticCompositeKind::Layer {
            clip_coverage: coverage,
            ..
        } = &mut kind
        {
            *coverage = clip_coverage.map(|resource| resource.id);
        }

        let mut sources = self.account.temporary(additional_sources.len() + 3)?;
        sources.append_slot()?.write(parent.current);
        sources.append_slot()?.write(source);
        if let Some(coverage) = clip_coverage {
            sources.append_slot()?.write(coverage);
        }
        sources.extend_from_slice(additional_sources)?;
        let resource = graph_build(self.builder.declare_resource(
            SemanticResourceDescriptor::new(
                SemanticResourceRole::CompositeResult,
                parent.spatial,
                0,
            ),
        ))?;
        let pass = graph_build(self.builder.declare_pass(
            SemanticPassIntent::Composite,
            &dependencies_for(&sources, self.account)?,
            &{
                let mut reads = self.account.temporary(sources.len())?;
                for source in sources.iter() {
                    self.account.work(RenderWork::GraphEdgeVisits, 1)?;
                    reads.append_slot()?.write(source.id);
                }
                reads
            },
            SemanticPassResult::Resource(resource),
        ))?;
        self.composites.push(SemanticCompositePlan {
            pass,
            kind,
            source_captured_before_outer_semantics,
        })?;
        Ok(PlannedGraphParent {
            current: PlannedGraphResource {
                id: resource,
                producer: Some(pass),
                logical_bounds: parent.spatial.logical_bounds,
                spatial: parent.spatial,
            },
            spatial: parent.spatial,
        })
    }
}

fn commands_require_surface_grid(commands: &[RenderCommand], account: &Account) -> Result<bool> {
    fn sensitive(image: &crate::Image) -> bool {
        matches!(
            image.rendering_hint(),
            Some(ImageRendering::Pixelated | ImageRendering::CrispEdges)
        )
    }
    fn paint_sensitive(paint: &crate::Paint, account: &Account) -> Result<bool> {
        account.work(RenderWork::PaintVisits, 1)?;
        Ok(match paint.kind() {
            crate::paint::PaintKind::RepeatingGradient(_) => true,
            crate::paint::PaintKind::Gradient(gradient) => {
                !matches!(gradient.kind(), crate::paint::GradientKind::Sweep { .. })
            }
            crate::paint::PaintKind::Image(image) => sensitive(image),
            crate::paint::PaintKind::Color(_) => false,
        })
    }
    for command in commands {
        account.work(RenderWork::CommandVisits, 1)?;
        let requires = match command {
            RenderCommand::Image { image, .. } => sensitive(image),
            RenderCommand::Fill { paint, .. } | RenderCommand::Stroke { paint, .. } => {
                account.work(RenderWork::PaintVisits, 1)?;
                match paint {
                    crate::command::RenderPaint::RepeatingGradient(_) => true,
                    crate::command::RenderPaint::Gradient(gradient) => {
                        !matches!(gradient.kind(), crate::paint::GradientKind::Sweep { .. })
                    }
                    crate::command::RenderPaint::Image(image) => sensitive(image),
                    crate::command::RenderPaint::Color(_) => false,
                }
            }
            RenderCommand::TextRun { paint, .. } => paint_sensitive(paint.fill(), account)?,
            RenderCommand::PreparedGlyphRun { run, paint, .. } => {
                let mut requires = paint_sensitive(paint.fill(), account)?;
                if !requires {
                    for operation in run.paints() {
                        account.work(RenderWork::PaintVisits, 1)?;
                        if matches!(operation, crate::text::GlyphPaint::Bitmap { image, .. } if sensitive(image))
                        {
                            requires = true;
                            break;
                        }
                    }
                }
                requires
            }
            RenderCommand::Layer { layer, children } => {
                layer.explicit_isolation
                    || (!layer.composition.is_source_over() && !layer.composition.is_plus())
                    || layer.mask.as_ref().is_some_and(|mask| {
                        matches!(
                            mask.upload().rendering_hint(),
                            Some(ImageRendering::Pixelated | ImageRendering::CrispEdges)
                        )
                    })
                    || commands_require_surface_grid(children, account)?
            }
            RenderCommand::Shadow { .. } => false,
        };
        if requires {
            return Ok(true);
        }
    }
    Ok(false)
}

fn commands_require_composition_domain(
    commands: &[RenderCommand],
    account: &Account,
) -> Result<bool> {
    for command in commands {
        account.work(RenderWork::CommandVisits, 1)?;
        if let RenderCommand::Layer { layer, children } = command
            && (layer.explicit_isolation
                || (!layer.composition.is_source_over() && !layer.composition.is_plus())
                || commands_require_composition_domain(children, account)?)
        {
            return Ok(true);
        }
    }
    Ok(false)
}

fn command_is_local_to_capture(
    command: &RenderCommand,
    parent_locality: CaptureParentLocality,
    account: &Account,
) -> Result<bool> {
    account.work(RenderWork::CommandVisits, 1)?;
    let RenderCommand::Layer { layer, children } = command else {
        return Ok(true);
    };
    if layer.mask.is_some()
        || layer.backdrop.is_some()
        || layer.foreground_blur.is_some()
        || layer.explicit_isolation
        || (!layer.composition.is_source_over() && !layer.composition.is_plus())
    {
        return Ok(false);
    }
    if !layer.composition.is_source_over_normal()
        && parent_locality == CaptureParentLocality::External
    {
        return Ok(false);
    }
    let child_parent_locality = if parent_locality == CaptureParentLocality::CaptureLocal
        || layer.isolation == LayerIsolation::BackendLayer
    {
        CaptureParentLocality::CaptureLocal
    } else {
        CaptureParentLocality::External
    };
    for child in children {
        if !command_is_local_to_capture(child, child_parent_locality, account)? {
            return Ok(false);
        }
    }
    Ok(true)
}

fn dependencies_for(
    resources: &[PlannedGraphResource],
    account: &Account,
) -> Result<AccountedVec<SemanticPassId>> {
    let mut dependencies = account.temporary(resources.len())?;
    for resource in resources {
        account.work(RenderWork::GraphEdgeVisits, 1)?;
        let Some(producer) = resource.producer else {
            continue;
        };
        if !edge_contains(&dependencies, &producer, account)? {
            dependencies.append_slot()?.write(producer);
        }
    }
    Ok(dependencies)
}

fn planned_resource_producer(resource: PlannedGraphResource) -> Result<SemanticPassId> {
    resource.producer.ok_or_else(|| {
        Error::new(
            BackendErrorCode::RenderFailed,
            "a planned graph result has no producing pass",
        )
    })
}

impl From<Error> for GraphValidationError {
    fn from(error: Error) -> Self {
        Self::Accounting(error)
    }
}
fn edge_contains<T: PartialEq>(values: &[T], needle: &T, account: &Account) -> Result<bool> {
    for value in values {
        account.work(RenderWork::GraphEdgeVisits, 1)?;
        if value == needle {
            return Ok(true);
        }
    }
    Ok(false)
}
fn clone_list<T: Clone>(values: &[T], account: &Account) -> Result<AccountedVec<T>> {
    let mut result = account.retained(values.len())?;
    result.extend_from_slice(values)?;
    Ok(result)
}
fn shared_copy<T: Clone>(values: &[T], account: &Account) -> Result<SharedValue<Vec<T>>> {
    SharedValue::from_accounted(clone_list(values, account)?, &account.storage())
}

pub(super) fn graph_build<T>(result: GraphBuildResult<T>) -> Result<T> {
    result.map_err(|error| {
        if let GraphValidationError::Accounting(error) = error {
            return error;
        }
        Error::new(
            BackendErrorCode::RenderFailed,
            format!("semantic frame graph validation failed: {error:?}"),
        )
    })
}

#[cfg(test)]
mod diagnostic_tests {
    use super::*;

    #[test]
    fn graph_failure_preserves_dependency_identities_in_the_returned_error() {
        let error = graph_build::<()>(Err(GraphValidationError::MissingProducerDependency {
            resource: SemanticResourceId::new(GraphGeneration(7), ResourceIndex(3)),
            producer: SemanticPassId::new(GraphGeneration(7), PassIndex(4)),
        }))
        .unwrap_err();
        assert_eq!(
            error.message(),
            "semantic frame graph validation failed: MissingProducerDependency { resource: SemanticResourceId { generation: GraphGeneration(7), index: ResourceIndex(3) }, producer: SemanticPassId { generation: GraphGeneration(7), index: PassIndex(4) } }",
        );
    }
}

#[cfg(test)]
#[path = "surface_grid_tests.rs"]
mod surface_grid_tests;
