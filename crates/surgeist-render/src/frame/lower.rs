use super::CompositeSourceSpace;
use super::{
    bounds::NonEmptyFrameSpatialPlan,
    filter::{
        DropShadowAlphaSource, DropShadowOffsetSampling, DropShadowSourceComposition,
        FilterEdgePolicy, ResolvedFilterOperationIntent, ResolvedFilterSpatialMapping,
    },
    graph::{
        BlurInput, GpuRenderGraph, GraphBuildResult, GraphGeneration, GraphValidationError,
        PassIndex, SemanticClipCoverage, SemanticCompositeKind, SemanticCompositePlan,
        SemanticFilterStepPlan, SemanticGraphPass, SemanticGraphResource, SemanticImportKind,
        SemanticImportPlan, SemanticPassId, SemanticPassIntent, SemanticPassResult,
        SemanticResourceDescriptor, SemanticResourceId, SemanticResourceProducer,
        SemanticResourceRole, SemanticVelloSpan, SemanticVelloSpanScope,
        WorkingImageInitialization, graph_build,
    },
    validate::validate_lowering_graph,
};
use crate::accounting::{Account, AccountedVec, RenderPhase, RenderWork, SharedValue};
use crate::{
    command::{RenderClip, RenderCommands},
    error::Result,
    filter::ColorClampBoundary,
    geometry::{PhysicalSize, Point, Rect, Transform},
    image::{Extend, ImageQuality, ImageRendering, ResolvedMaskUploadDescriptor},
    layer::LayerComposition,
    paint::Color,
    renderer::Antialiasing,
    style::ColorFilterOp,
};

#[cfg(test)]
mod repeated_proof_edge_tests {
    use super::*;
    use crate::accounting::{
        RenderAccountingContext, RenderAccountingOptions, RenderAttemptKind, RenderLimits,
    };

    fn color_filter_lookup_graph(
        count: usize,
        setup: &Account,
    ) -> crate::accounting::OwnedBox<GpuRenderGraph> {
        let mut scene = crate::Scene::new();
        for _ in 0..count {
            let filters = crate::FilterList::try_ops(vec![crate::FilterOp::brightness(
                crate::FilterAmount::try_new(1.25).unwrap(),
            )])
            .unwrap();
            scene.layer(crate::Layer::new().with_filter_list(filters), |child| {
                child.fill(Rect::new(0.0, 0.0, 1.0, 1.0), Color::BLACK);
            });
        }
        let commands = scene
            .normalize(crate::Capabilities::CURRENT, setup)
            .unwrap();
        let context = crate::frame::FrameContext::try_new(
            crate::Size::new(4.0, 4.0),
            1.0,
            Antialiasing::Area,
            Color::TRANSPARENT,
        )
        .unwrap();
        let crate::frame::FramePlan::GpuGraph(graph) = commands.plan_for(context, setup).unwrap()
        else {
            panic!("foreground filters must select the graph");
        };
        assert_eq!(graph.filter_steps.len(), count);
        graph
    }

    #[test]
    fn last_color_filter_lookup_uses_a_bounded_step_search() {
        use crate::{RenderAttemptStatus, RenderQuantity, RenderResource};

        let setup = Account::for_test();
        let graph = color_filter_lookup_graph(16, &setup);
        let passes = graph.lowering_view(&setup).unwrap().passes(&setup).unwrap();
        let last = passes
            .iter()
            .rev()
            .find(|view| view.pass.intent == SemanticPassIntent::ColorFilter)
            .copied()
            .unwrap();

        let node = RenderWork::GraphNodeVisits;
        let edge = RenderWork::GraphEdgeVisits;
        for limit in [4, 5] {
            let limits = RenderLimits::new()
                .with_work_limit(node, limit)
                .unwrap()
                .with_work_limit(edge, 5)
                .unwrap();
            let mut accounting =
                RenderAccountingContext::new(RenderAccountingOptions::new().with_limits(limits));
            let result = accounting.run(RenderAttemptKind::Frame, RenderPhase::Lower, |account| {
                last.kind(account).map(|kind| {
                    assert!(matches!(kind, GraphLoweringPassKind::ColorFilter(Some(_))));
                })
            });
            let terminal = accounting.last_attempt().unwrap();
            assert_eq!(terminal.cleanup_fault(), None);
            let attempt = terminal.attempt().unwrap();
            assert_eq!(attempt.usage().work(node), limit);
            assert_eq!(attempt.usage().live(RenderResource::RetainedCpuBytes), 0);
            if limit == 5 {
                result.unwrap();
                // The pass and four search probes. Each probe reads a start
                // ID and the selected step proves membership.
                assert_eq!(attempt.usage().work(edge), 5);
            } else {
                assert!(result.unwrap_err().accounting_diagnostic().is_some());
                assert_eq!(attempt.status(), RenderAttemptStatus::Refused);
                let refusal = attempt.refusal().unwrap();
                assert_eq!(refusal.quantity(), RenderQuantity::Work(node));
                assert_eq!((refusal.current(), refusal.amount()), (4, 1));
                assert_eq!(attempt.usage().work(edge), 3);
            }
        }
    }

    #[test]
    fn filter_step_lookup_rejects_duplicate_and_unordered_membership() {
        let setup = Account::for_test();
        let mut graph = color_filter_lookup_graph(2, &setup);
        let first = graph.filter_steps[0].passes[0];
        let second = graph.filter_steps[1].passes[0];
        let gap = graph
            .passes
            .iter()
            .find(|pass| pass.id.index.0 > first.index.0 && pass.id.index.0 < second.index.0)
            .unwrap();
        let steps = QualifiedFilterSteps::try_new(&graph, &setup).unwrap();
        assert!(steps.find(gap.id, &setup).unwrap().is_none());
        assert!(steps.find(second, &setup).unwrap().is_some());

        graph.filter_steps.swap(0, 1);
        assert!(matches!(
            QualifiedFilterSteps::try_new(&graph, &setup),
            Err(GraphValidationError::InvalidPassArity)
        ));
        assert!(graph.lowering_view(&setup).is_err());

        graph.filter_steps.swap(0, 1);
        graph.filter_steps[1].passes = graph.filter_steps[0].passes.clone();
        assert!(matches!(
            QualifiedFilterSteps::try_new(&graph, &setup),
            Err(GraphValidationError::InvalidPassArity)
        ));
        assert!(graph.lowering_view(&setup).is_err());
    }

    #[test]
    fn repeated_proof_filter_spatial_mapping_admits_its_source_read() {
        let setup = Account::for_test();
        let mut scene = crate::Scene::new();
        scene.fill(Rect::new(0.0, 0.0, 1.0, 1.0), Color::BLACK);
        let commands = scene
            .normalize(crate::Capabilities::CURRENT, &setup)
            .unwrap();
        let context = crate::frame::FrameContext::try_new(
            crate::Size::new(4.0, 4.0),
            1.0,
            Antialiasing::Area,
            Color::TRANSPARENT,
        )
        .unwrap();
        let filters = crate::FilterList::try_ops(vec![crate::FilterOp::brightness(
            crate::FilterAmount::try_new(1.25).unwrap(),
        )])
        .unwrap();
        let graph =
            crate::frame::authored_filter_graph_for_test(vec![filters], commands, context, &setup)
                .unwrap();
        let pass = graph
            .passes
            .iter()
            .find(|pass| pass.intent == SemanticPassIntent::ColorFilter)
            .unwrap();
        assert_eq!(pass.reads.len(), 1);
        let source = &graph.resources[pass.reads[0].index.as_usize().unwrap()];
        let SemanticPassResult::Resource(result_id) = pass.result else {
            unreachable!()
        };
        let result_resource = &graph.resources[result_id.index.as_usize().unwrap()];
        for limit in [1, 0] {
            let options = RenderAccountingOptions::new().with_limits(
                RenderLimits::new()
                    .with_work_limit(RenderWork::GraphEdgeVisits, limit)
                    .unwrap(),
            );
            let mut context = RenderAccountingContext::new(options);
            let result = context.run(RenderAttemptKind::Frame, RenderPhase::Lower, |account| {
                graph_build(graph_lowering_pass_spatial(&graph, pass, account))
            });
            let report = context.last_attempt().unwrap().attempt().unwrap();
            if limit == 1 {
                let mapping = result.unwrap();
                assert_eq!(
                    mapping.source(),
                    GraphLoweringSpatialDescriptor::from_semantic(source.descriptor)
                );
                assert_eq!(
                    mapping.result(),
                    GraphLoweringSpatialDescriptor::from_semantic(result_resource.descriptor)
                );
                assert_eq!(report.usage().work(RenderWork::GraphEdgeVisits), 1);
            } else {
                assert!(
                    result.is_err(),
                    "the source identity must refuse before resource-node lookup"
                );
                let refusal = report.refusal().unwrap();
                assert_eq!(
                    refusal.quantity,
                    crate::RenderQuantity::Work(RenderWork::GraphEdgeVisits)
                );
                assert_eq!((refusal.current, refusal.amount), (0, 1));
                assert_eq!(report.usage().work(RenderWork::GraphNodeVisits), 0);
            }
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub(crate) struct GraphLoweringGeneration(u64);

impl GraphLoweringGeneration {
    const fn from_semantic(generation: GraphGeneration) -> Self {
        Self(generation.0)
    }
}

#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub(crate) struct GraphLoweringResourceId {
    generation: GraphLoweringGeneration,
    index: u32,
}

impl GraphLoweringResourceId {
    const fn from_semantic(id: SemanticResourceId) -> Self {
        Self {
            generation: GraphLoweringGeneration::from_semantic(id.generation),
            index: id.index.0,
        }
    }
}

impl GpuRenderGraph {
    pub(crate) fn lowering_view(&self, account: &Account) -> Result<GraphLoweringView<'_>> {
        let account = &account.at(RenderPhase::Lower);
        let graph = validate_lowering_graph(self, account)?;
        Ok(GraphLoweringView { graph })
    }
}

#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub(crate) struct GraphLoweringPassId {
    generation: GraphLoweringGeneration,
    index: u32,
}

impl GraphLoweringPassId {
    const fn from_semantic(id: SemanticPassId) -> Self {
        Self {
            generation: GraphLoweringGeneration::from_semantic(id.generation),
            index: id.index.0,
        }
    }

    #[cfg(test)]
    pub(crate) fn stale_generation_for_test(self) -> Option<Self> {
        self.generation.0.checked_add(1).map(|generation| Self {
            generation: GraphLoweringGeneration(generation),
            index: self.index,
        })
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum GraphLoweringResourceRole {
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

impl GraphLoweringResourceRole {
    const fn from_semantic(role: SemanticResourceRole) -> Self {
        match role {
            SemanticResourceRole::RootWorkingImage => Self::RootWorkingImage,
            SemanticResourceRole::CaptureWorkingImage => Self::CaptureWorkingImage,
            SemanticResourceRole::ClipCoverage => Self::ClipCoverage,
            SemanticResourceRole::IsolationWorkingImage => Self::IsolationWorkingImage,
            SemanticResourceRole::ImportedImage => Self::ImportedImage,
            SemanticResourceRole::BackdropCopy => Self::BackdropCopy,
            SemanticResourceRole::FilterIntermediate => Self::FilterIntermediate,
            SemanticResourceRole::ShadowImage => Self::ShadowImage,
            SemanticResourceRole::CompositeResult => Self::CompositeResult,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) struct GraphLoweringSpatialDescriptor {
    logical_bounds: Rect,
    device_origin: (i32, i32),
    device_extent: PhysicalSize,
    texel_origin: Point,
    raster_scale: f64,
}

impl GraphLoweringSpatialDescriptor {
    const fn from_semantic(descriptor: SemanticResourceDescriptor) -> Self {
        Self {
            logical_bounds: descriptor.logical_bounds.rect(),
            device_origin: (descriptor.device_origin.x, descriptor.device_origin.y),
            device_extent: PhysicalSize::new(
                descriptor.device_extent.width,
                descriptor.device_extent.height,
            ),
            texel_origin: descriptor.texel_center_mapping.origin,
            raster_scale: descriptor.texel_center_mapping.raster_scale.get(),
        }
    }

    #[must_use]
    pub(crate) const fn logical_bounds(self) -> Rect {
        self.logical_bounds
    }

    #[must_use]
    pub(crate) const fn device_origin(self) -> (i32, i32) {
        self.device_origin
    }

    #[must_use]
    pub(crate) const fn device_extent(self) -> PhysicalSize {
        self.device_extent
    }

    #[must_use]
    pub(crate) const fn texel_origin(self) -> Point {
        self.texel_origin
    }

    #[must_use]
    pub(crate) const fn raster_scale(self) -> f64 {
        self.raster_scale
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum GraphLoweringResourceProducer {
    Imported,
    Pass(GraphLoweringPassId),
}

#[derive(Clone, Copy, Debug)]
pub(crate) enum GraphLoweringImportView<'graph> {
    ResolvedAlphaMask(&'graph SharedValue<ResolvedMaskUploadDescriptor>),
}

#[derive(Clone, Copy, Debug)]
pub(crate) struct GraphLoweringResourceView<'graph> {
    resource: &'graph SemanticGraphResource,
    binding: GraphLoweringResourceBinding<'graph>,
}

#[derive(Clone, Copy, Debug)]
#[repr(u32)]
enum GraphLoweringResourceBinding<'graph> {
    Pass {
        last_use: PassIndex,
        producer: PassIndex,
    },
    Imported {
        // Keep the tag and last-use index together before the borrowed upload.
        last_use: PassIndex,
        import: GraphLoweringImportView<'graph>,
    },
}

impl<'graph> GraphLoweringResourceView<'graph> {
    fn try_from_validated(
        resource: &'graph SemanticGraphResource,
        import: Option<&'graph SemanticImportPlan>,
    ) -> GraphBuildResult<Self> {
        let producer = resource
            .producer
            .ok_or(GraphValidationError::ResourceWithoutProducer(resource.id))?;
        let last_use = resource
            .releasable_after
            .ok_or(GraphValidationError::UnscheduledReads {
                resource: resource.id,
                remaining: 0,
            })?
            .index;
        // Validation has matched both pass identities to this resource's
        // generation, so the binding only needs to carry their indices.
        let binding = match (producer, import) {
            (SemanticResourceProducer::Pass(producer), None) => {
                GraphLoweringResourceBinding::Pass {
                    last_use,
                    producer: producer.index,
                }
            }
            (SemanticResourceProducer::Imported, Some(import)) => {
                let SemanticImportKind::ResolvedAlphaMask { upload } = &import.kind;
                GraphLoweringResourceBinding::Imported {
                    last_use,
                    import: GraphLoweringImportView::ResolvedAlphaMask(upload),
                }
            }
            _ => return Err(GraphValidationError::InvalidImportedResourceRole),
        };
        Ok(Self { resource, binding })
    }

    const fn pass_id(self, index: PassIndex) -> GraphLoweringPassId {
        GraphLoweringPassId::from_semantic(SemanticPassId::new(self.resource.id.generation, index))
    }

    #[must_use]
    pub(crate) const fn id(self) -> GraphLoweringResourceId {
        GraphLoweringResourceId::from_semantic(self.resource.id)
    }

    #[must_use]
    pub(crate) const fn role(self) -> GraphLoweringResourceRole {
        GraphLoweringResourceRole::from_semantic(self.resource.descriptor.role)
    }

    #[must_use]
    pub(crate) const fn spatial(self) -> GraphLoweringSpatialDescriptor {
        GraphLoweringSpatialDescriptor::from_semantic(self.resource.descriptor)
    }

    #[must_use]
    pub(crate) const fn producer(self) -> GraphLoweringResourceProducer {
        match self.binding {
            GraphLoweringResourceBinding::Imported { .. } => {
                GraphLoweringResourceProducer::Imported
            }
            GraphLoweringResourceBinding::Pass { producer, .. } => {
                GraphLoweringResourceProducer::Pass(self.pass_id(producer))
            }
        }
    }

    #[must_use]
    pub(crate) const fn expected_reads(self) -> u32 {
        self.resource.descriptor.expected_reads
    }

    #[must_use]
    pub(crate) const fn last_use(self) -> GraphLoweringPassId {
        let (GraphLoweringResourceBinding::Pass { last_use, .. }
        | GraphLoweringResourceBinding::Imported { last_use, .. }) = self.binding;
        self.pass_id(last_use)
    }

    #[must_use]
    pub(crate) const fn import(self) -> Option<GraphLoweringImportView<'graph>> {
        match self.binding {
            GraphLoweringResourceBinding::Pass { .. } => None,
            GraphLoweringResourceBinding::Imported { import, .. } => Some(import),
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum GraphLoweringInitialization {
    SurfaceBaseColor,
    Transparent,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum GraphLoweringVelloSpanScope {
    CurrentParent,
    LayerSource,
}

#[derive(Clone, Debug, PartialEq)]
pub(crate) struct GraphLoweringVelloSpan {
    scope: GraphLoweringVelloSpanScope,
    source_space: CompositeSourceSpace,
    commands: RenderCommands,
    capture_transform: Transform,
    parent_to_surface: Transform,
    antialiasing: Antialiasing,
    captured_before_outer_semantics: bool,
}

impl GraphLoweringVelloSpan {
    #[must_use]
    pub(crate) const fn scope(&self) -> GraphLoweringVelloSpanScope {
        self.scope
    }

    #[must_use]
    pub(crate) const fn source_space(&self) -> CompositeSourceSpace {
        self.source_space
    }

    #[must_use]
    pub(crate) fn commands(&self) -> &RenderCommands {
        &self.commands
    }

    #[must_use]
    pub(crate) const fn capture_transform(&self) -> Transform {
        self.capture_transform
    }

    #[must_use]
    pub(crate) const fn parent_to_surface(&self) -> Transform {
        self.parent_to_surface
    }

    #[must_use]
    pub(crate) const fn antialiasing(&self) -> Antialiasing {
        self.antialiasing
    }

    #[must_use]
    pub(crate) const fn captured_before_outer_semantics(&self) -> bool {
        self.captured_before_outer_semantics
    }
}

#[derive(Clone, Debug, PartialEq)]
pub(crate) struct GraphLoweringClipCoverageElement {
    clip: RenderClip,
    transform: Transform,
}

impl GraphLoweringClipCoverageElement {
    #[must_use]
    pub(crate) const fn clip(&self) -> &RenderClip {
        &self.clip
    }

    #[must_use]
    pub(crate) const fn transform(&self) -> Transform {
        self.transform
    }
}

#[derive(Clone, Debug, PartialEq)]
pub(crate) struct GraphLoweringClipCoverage {
    elements: SharedValue<Vec<GraphLoweringClipCoverageElement>>,
    antialiasing: Antialiasing,
}

impl GraphLoweringClipCoverage {
    #[must_use]
    pub(crate) fn elements(&self) -> &[GraphLoweringClipCoverageElement] {
        &self.elements
    }

    #[must_use]
    pub(crate) const fn antialiasing(&self) -> Antialiasing {
        self.antialiasing
    }
}

#[derive(Clone, Debug, PartialEq)]
pub(crate) enum GraphLoweringVelloCapture {
    Span(GraphLoweringVelloSpan),
    ClipCoverage(GraphLoweringClipCoverage),
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) enum GraphLoweringEdgePolicy {
    NoSampling,
    TransparentBlack,
    SemanticBorderMirror(Rect),
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) struct GraphLoweringFilterSpatialMapping {
    source: GraphLoweringSpatialDescriptor,
    result: GraphLoweringSpatialDescriptor,
}

impl GraphLoweringFilterSpatialMapping {
    #[must_use]
    pub(crate) const fn source(self) -> GraphLoweringSpatialDescriptor {
        self.source
    }

    #[must_use]
    pub(crate) const fn result(self) -> GraphLoweringSpatialDescriptor {
        self.result
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) struct GraphLoweringColorOperation {
    operation: ColorFilterOp,
    clamp_boundary: ColorClampBoundary,
}

impl GraphLoweringColorOperation {
    #[must_use]
    pub(crate) const fn operation(self) -> ColorFilterOp {
        self.operation
    }

    #[must_use]
    pub(crate) const fn clamp_boundary(self) -> ColorClampBoundary {
        self.clamp_boundary
    }
}

#[derive(Clone, Debug, PartialEq)]
pub(crate) struct GraphLoweringColorFilter {
    operations: SharedValue<Vec<GraphLoweringColorOperation>>,
    spatial: GraphLoweringFilterSpatialMapping,
    edge: GraphLoweringEdgePolicy,
}

impl GraphLoweringColorFilter {
    #[must_use]
    pub(crate) fn operations(&self) -> &[GraphLoweringColorOperation] {
        &self.operations
    }

    #[must_use]
    pub(crate) const fn spatial(&self) -> GraphLoweringFilterSpatialMapping {
        self.spatial
    }

    #[must_use]
    pub(crate) const fn edge(&self) -> GraphLoweringEdgePolicy {
        self.edge
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum GraphLoweringBlurInput {
    Rgba,
    SourceAlpha,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) struct GraphLoweringBlur {
    input: GraphLoweringBlurInput,
    standard_deviation: f64,
    support_radius: u32,
    spatial: GraphLoweringFilterSpatialMapping,
    edge: GraphLoweringEdgePolicy,
}

impl GraphLoweringBlur {
    #[must_use]
    pub(crate) const fn input(self) -> GraphLoweringBlurInput {
        self.input
    }

    #[must_use]
    pub(crate) const fn standard_deviation(self) -> f64 {
        self.standard_deviation
    }

    #[must_use]
    pub(crate) const fn support_radius(self) -> u32 {
        self.support_radius
    }

    #[must_use]
    pub(crate) const fn spatial(self) -> GraphLoweringFilterSpatialMapping {
        self.spatial
    }

    #[must_use]
    pub(crate) const fn edge(self) -> GraphLoweringEdgePolicy {
        self.edge
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) struct GraphLoweringDropShadow {
    offset: Point,
    standard_deviation: f64,
    color: Color,
    support_radius: u32,
    spatial: GraphLoweringFilterSpatialMapping,
    edge: GraphLoweringEdgePolicy,
    source_alpha: bool,
    continuous_offset: bool,
    retains_unchanged_source: bool,
}

impl GraphLoweringDropShadow {
    #[must_use]
    pub(crate) const fn offset(self) -> Point {
        self.offset
    }

    #[must_use]
    pub(crate) const fn standard_deviation(self) -> f64 {
        self.standard_deviation
    }

    #[must_use]
    pub(crate) const fn color(self) -> Color {
        self.color
    }

    #[must_use]
    pub(crate) const fn support_radius(self) -> u32 {
        self.support_radius
    }

    #[must_use]
    pub(crate) const fn spatial(self) -> GraphLoweringFilterSpatialMapping {
        self.spatial
    }

    #[must_use]
    pub(crate) const fn edge(self) -> GraphLoweringEdgePolicy {
        self.edge
    }

    #[must_use]
    pub(crate) const fn uses_source_alpha(self) -> bool {
        self.source_alpha
    }

    #[must_use]
    pub(crate) const fn uses_continuous_offset(self) -> bool {
        self.continuous_offset
    }

    #[must_use]
    pub(crate) const fn retains_unchanged_source(self) -> bool {
        self.retains_unchanged_source
    }
}

#[derive(Clone, Debug, PartialEq)]
pub(crate) struct GraphLoweringOuterClip {
    clip: RenderClip,
    transform: Transform,
}

impl GraphLoweringOuterClip {
    #[must_use]
    pub(crate) const fn clip(&self) -> &RenderClip {
        &self.clip
    }

    #[must_use]
    pub(crate) const fn transform(&self) -> Transform {
        self.transform
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) struct GraphLoweringDestinationToLayerLocal {
    affine: Transform,
}

impl GraphLoweringDestinationToLayerLocal {
    #[must_use]
    pub(crate) const fn affine(self) -> Transform {
        self.affine
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) struct GraphLoweringResolvedAlphaMaskComposition {
    resource: GraphLoweringResourceId,
    bounds: Rect,
    image_dimensions: PhysicalSize,
    quality: ImageQuality,
    rendering: Option<ImageRendering>,
    extend: Extend,
    value: crate::layer::MaskValue,
    application: crate::layer::MaskApplication,
}

impl GraphLoweringResolvedAlphaMaskComposition {
    #[must_use]
    pub(crate) const fn resource(self) -> GraphLoweringResourceId {
        self.resource
    }

    #[must_use]
    pub(crate) const fn bounds(self) -> Rect {
        self.bounds
    }

    #[must_use]
    pub(crate) const fn image_dimensions(self) -> PhysicalSize {
        self.image_dimensions
    }

    #[must_use]
    pub(crate) const fn quality(self) -> ImageQuality {
        self.quality
    }

    #[must_use]
    pub(crate) const fn rendering_hint(self) -> Option<ImageRendering> {
        self.rendering
    }

    #[must_use]
    pub(crate) const fn extend(self) -> Extend {
        self.extend
    }

    pub(crate) const fn value(self) -> crate::layer::MaskValue {
        self.value
    }

    pub(crate) const fn application(self) -> crate::layer::MaskApplication {
        self.application
    }
}

#[derive(Clone, Debug, PartialEq)]
pub(crate) enum GraphLoweringCompositeKind {
    SpanSourceOver,
    Layer {
        transform: Transform,
        destination_to_layer_local: GraphLoweringDestinationToLayerLocal,
        source_space: CompositeSourceSpace,
        opacity: f32,
        composition: LayerComposition,
        clip: Option<SharedValue<RenderClip>>,
        outer_clips: SharedValue<Vec<GraphLoweringOuterClip>>,
        clip_coverage: Option<GraphLoweringResourceId>,
        alpha_mask: Option<SharedValue<GraphLoweringResolvedAlphaMaskComposition>>,
    },
    DropShadow,
}

#[derive(Clone, Debug, PartialEq)]
pub(crate) struct GraphLoweringComposite {
    kind: GraphLoweringCompositeKind,
    source_captured_before_outer_semantics: bool,
}

impl GraphLoweringComposite {
    #[must_use]
    pub(crate) const fn kind(&self) -> &GraphLoweringCompositeKind {
        &self.kind
    }

    #[must_use]
    pub(crate) const fn source_captured_before_outer_semantics(&self) -> bool {
        self.source_captured_before_outer_semantics
    }
}

#[derive(Clone, Debug, PartialEq)]
pub(crate) enum GraphLoweringPassKind {
    ClearRoot {
        initialization: GraphLoweringInitialization,
        color: Color,
    },
    VelloCapture(Option<GraphLoweringVelloCapture>),
    CanonicalizeCapture,
    CopyBackdrop,
    ColorFilter(Option<GraphLoweringColorFilter>),
    BlurHorizontal(Option<GraphLoweringBlur>),
    BlurVertical(Option<GraphLoweringBlur>),
    DropShadowColorize(Option<GraphLoweringDropShadow>),
    Composite(Option<GraphLoweringComposite>),
    Present,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum GraphLoweringReadRole {
    CaptureSource,
    CompletedParent,
    FilterSource,
    BlurredSourceAlpha,
    CompositeParent,
    CompositeSource,
    ClipCoverage,
    AlphaMask,
    Shadow,
    FinalWorkingImage,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum GraphLoweringSamplingFilter {
    Nearest,
    Linear,
    GaussianKernel,
    ImportedMask,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) enum GraphLoweringSamplingEdge {
    ClampToExtent,
    TransparentBlack,
    SemanticBorderMirror(Rect),
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) struct GraphLoweringReadBinding {
    role: GraphLoweringReadRole,
    resource: GraphLoweringResourceId,
    sampling_filter: GraphLoweringSamplingFilter,
    sampling_edge: GraphLoweringSamplingEdge,
}

impl GraphLoweringReadBinding {
    #[must_use]
    pub(crate) const fn role(self) -> GraphLoweringReadRole {
        self.role
    }

    #[must_use]
    pub(crate) const fn resource(self) -> GraphLoweringResourceId {
        self.resource
    }

    #[must_use]
    pub(crate) const fn sampling_filter(self) -> GraphLoweringSamplingFilter {
        self.sampling_filter
    }

    #[must_use]
    pub(crate) const fn sampling_edge(self) -> GraphLoweringSamplingEdge {
        self.sampling_edge
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum GraphLoweringPassResult {
    Empty,
    Resource(GraphLoweringResourceId),
}

#[derive(Clone, Copy, Debug)]
pub(crate) struct GraphLoweringPassView<'graph> {
    graph: QualifiedFilterSteps<'graph>,
    pass: &'graph SemanticGraphPass,
}

impl GraphLoweringPassView<'_> {
    #[must_use]
    pub(crate) const fn id(self) -> GraphLoweringPassId {
        GraphLoweringPassId::from_semantic(self.pass.id)
    }

    pub(crate) fn dependencies(
        self,
        account: &Account,
    ) -> Result<AccountedVec<GraphLoweringPassId>> {
        let account = &account.at(RenderPhase::Lower);
        let mut result = account.temporary(self.pass.dependencies.len())?;
        for dependency in self.pass.dependencies.iter() {
            account.work(RenderWork::GraphEdgeVisits, 1)?;
            result
                .append_slot()?
                .write(GraphLoweringPassId::from_semantic(*dependency));
        }
        Ok(result)
    }

    pub(crate) fn kind(self, account: &Account) -> Result<GraphLoweringPassKind> {
        let account = &account.at(RenderPhase::Lower);
        graph_build(graph_lowering_pass_kind(self.graph, self.pass, account))
    }

    /// Converts the kind and derives ordered reads from the same pass metadata.
    /// A kind conversion failure stops before any read binding is prepared.
    pub(crate) fn project<T>(
        self,
        account: &Account,
        convert_kind: impl FnOnce(&GraphLoweringPassKind) -> Result<T>,
    ) -> Result<(T, AccountedVec<GraphLoweringReadBinding>)> {
        let kind = self.kind(account)?;
        let converted = convert_kind(&kind)?;
        let account = &account.at(RenderPhase::Lower);
        let reads = graph_build(graph_lowering_read_bindings(self.pass, &kind, account))?;
        Ok((converted, reads))
    }

    #[cfg(test)]
    pub(crate) fn reads(self, account: &Account) -> Result<AccountedVec<GraphLoweringReadBinding>> {
        self.project(account, |_| Ok(())).map(|(_, reads)| reads)
    }

    #[must_use]
    pub(crate) const fn result(self) -> GraphLoweringPassResult {
        match self.pass.result {
            SemanticPassResult::Empty => GraphLoweringPassResult::Empty,
            SemanticPassResult::Resource(resource) => {
                GraphLoweringPassResult::Resource(GraphLoweringResourceId::from_semantic(resource))
            }
        }
    }
}

#[derive(Clone, Copy, Debug)]
pub(crate) struct GraphLoweringView<'graph> {
    graph: QualifiedFilterSteps<'graph>,
}

impl<'graph> GraphLoweringView<'graph> {
    #[must_use]
    pub(crate) const fn generation(self) -> GraphLoweringGeneration {
        GraphLoweringGeneration::from_semantic(self.graph.source.generation)
    }

    #[must_use]
    pub(crate) const fn root_working_image(self) -> GraphLoweringResourceId {
        GraphLoweringResourceId::from_semantic(self.graph.source.root_working_image)
    }

    #[must_use]
    pub(crate) const fn final_present(self) -> GraphLoweringPassId {
        GraphLoweringPassId::from_semantic(self.graph.source.final_present)
    }

    pub(crate) fn resources(
        self,
        account: &Account,
    ) -> Result<AccountedVec<GraphLoweringResourceView<'graph>>> {
        let account = &account.at(RenderPhase::Lower);
        let graph = self.graph.source;
        let mut result = account.temporary(graph.resources.len())?;
        for resource in graph.resources.iter() {
            account.work(RenderWork::GraphNodeVisits, 1)?;
            let mut import = None;
            for candidate in graph.imports.iter() {
                account.work(RenderWork::GraphNodeVisits, 1)?;
                if candidate.resource == resource.id {
                    import = Some(candidate);
                    break;
                }
            }
            let resource_view = graph_build(GraphLoweringResourceView::try_from_validated(
                resource, import,
            ))?;
            result.append_slot()?.write(resource_view);
        }
        Ok(result)
    }

    pub(crate) fn passes(
        self,
        account: &Account,
    ) -> Result<AccountedVec<GraphLoweringPassView<'graph>>> {
        let account = &account.at(RenderPhase::Lower);
        let mut result = account.temporary(self.graph.source.passes.len())?;
        for pass in self.graph.source.passes.iter() {
            account.work(RenderWork::GraphNodeVisits, 1)?;
            result.append_slot()?.write(self.graph.pass_view(pass));
        }
        Ok(result)
    }
}

fn graph_lowering_pass_kind(
    filter_steps: QualifiedFilterSteps<'_>,
    pass: &SemanticGraphPass,
    account: &Account,
) -> GraphBuildResult<GraphLoweringPassKind> {
    account.work(RenderWork::GraphNodeVisits, 1)?;
    let graph = filter_steps.source;
    match pass.intent {
        SemanticPassIntent::ClearRoot { initialization } => {
            let (initialization, color) = match initialization {
                WorkingImageInitialization::SurfaceBaseColor(color) => {
                    (GraphLoweringInitialization::SurfaceBaseColor, color)
                }
                WorkingImageInitialization::Transparent => {
                    (GraphLoweringInitialization::Transparent, Color::TRANSPARENT)
                }
            };
            Ok(GraphLoweringPassKind::ClearRoot {
                initialization,
                color,
            })
        }
        SemanticPassIntent::VelloCapture { .. } => {
            graph_lowering_vello_capture(graph, pass, account)
        }
        SemanticPassIntent::CanonicalizeCapture => Ok(GraphLoweringPassKind::CanonicalizeCapture),
        SemanticPassIntent::CopyBackdrop => {
            let reads = matching(&graph.backdrop_reads, account, |read| read.pass == pass.id)?;
            match (pass.result, reads) {
                (SemanticPassResult::Empty, Matches::None)
                | (SemanticPassResult::Resource(_), Matches::One(_)) => {}
                _ => return Err(GraphValidationError::InvalidPassArity),
            }
            Ok(GraphLoweringPassKind::CopyBackdrop)
        }
        SemanticPassIntent::ColorFilter => {
            graph_lowering_color_filter(pass, filter_steps.find(pass.id, account)?, account)
        }
        SemanticPassIntent::BlurHorizontal { input }
        | SemanticPassIntent::BlurVertical { input } => graph_lowering_blur_pass(
            graph,
            pass,
            input,
            filter_steps.find(pass.id, account)?,
            account,
        ),
        SemanticPassIntent::DropShadowColorize => graph_lowering_drop_shadow_pass(
            graph,
            pass,
            filter_steps.find(pass.id, account)?,
            account,
        ),
        SemanticPassIntent::Composite => graph_lowering_composite_pass(filter_steps, pass, account),
        SemanticPassIntent::Present => Ok(GraphLoweringPassKind::Present),
    }
}

fn graph_lowering_vello_capture(
    graph: &GpuRenderGraph,
    pass: &SemanticGraphPass,

    account: &Account,
) -> GraphBuildResult<GraphLoweringPassKind> {
    let spans = matching(graph.vello_spans.as_slice(), account, |span| {
        span.capture_pass == pass.id
    })?;
    let coverages = matching(graph.clip_coverages.as_slice(), account, |coverage| {
        coverage.capture_pass == pass.id
    })?;
    let work = match (pass.result, spans, coverages) {
        (SemanticPassResult::Empty, Matches::None, Matches::None) => None,
        (SemanticPassResult::Resource(_), Matches::One(span), Matches::None) => Some(
            GraphLoweringVelloCapture::Span(graph_lowering_vello_span(span)),
        ),
        (SemanticPassResult::Resource(_), Matches::None, Matches::One(coverage)) => {
            Some(GraphLoweringVelloCapture::ClipCoverage(
                graph_lowering_clip_coverage(coverage, account)?,
            ))
        }
        _ => {
            return Err(GraphValidationError::InvalidCaptureResult);
        }
    };
    Ok(GraphLoweringPassKind::VelloCapture(work))
}

fn graph_lowering_color_filter(
    pass: &SemanticGraphPass,
    step: Option<&SemanticFilterStepPlan>,
    account: &Account,
) -> GraphBuildResult<GraphLoweringPassKind> {
    let filter = match (pass.result, step) {
        (SemanticPassResult::Empty, None) => None,
        (SemanticPassResult::Resource(_), Some(step)) => {
            let ResolvedFilterOperationIntent::ColorRun(run) = &step.step.operation_intent else {
                return Err(GraphValidationError::InvalidPassArity);
            };
            if step.passes.len() != 1 {
                return Err(GraphValidationError::InvalidPassArity);
            }
            Some(GraphLoweringColorFilter {
                operations: mapped_shared(
                    run.operations(),
                    account,
                    RenderWork::FilterOperationVisits,
                    |operation| {
                        Ok(GraphLoweringColorOperation {
                            operation: operation.operation(),
                            clamp_boundary: operation.clamp_boundary(),
                        })
                    },
                )?,
                spatial: graph_lowering_filter_spatial(step.step.spatial_mapping),
                edge: graph_lowering_edge(step.step.edge_policy),
            })
        }
        (SemanticPassResult::Empty, Some(_)) | (SemanticPassResult::Resource(_), None) => {
            return Err(GraphValidationError::InvalidPassArity);
        }
    };
    Ok(GraphLoweringPassKind::ColorFilter(filter))
}

fn graph_lowering_blur_pass(
    graph: &GpuRenderGraph,
    pass: &SemanticGraphPass,
    input: BlurInput,
    step: Option<&SemanticFilterStepPlan>,
    account: &Account,
) -> GraphBuildResult<GraphLoweringPassKind> {
    let blur = match (pass.result, step) {
        (SemanticPassResult::Empty, None) => None,
        (SemanticPassResult::Resource(_), Some(step)) => {
            Some(graph_lowering_blur(graph, pass, input, step, account)?)
        }
        (SemanticPassResult::Empty, Some(_)) | (SemanticPassResult::Resource(_), None) => {
            return Err(GraphValidationError::InvalidPassArity);
        }
    };
    if matches!(pass.intent, SemanticPassIntent::BlurHorizontal { .. }) {
        Ok(GraphLoweringPassKind::BlurHorizontal(blur))
    } else {
        Ok(GraphLoweringPassKind::BlurVertical(blur))
    }
}

fn graph_lowering_drop_shadow_pass(
    graph: &GpuRenderGraph,
    pass: &SemanticGraphPass,
    step: Option<&SemanticFilterStepPlan>,
    account: &Account,
) -> GraphBuildResult<GraphLoweringPassKind> {
    let shadow = match (pass.result, step) {
        (SemanticPassResult::Empty, None) => None,
        (SemanticPassResult::Resource(_), Some(step)) => {
            Some(graph_lowering_drop_shadow(graph, pass, step, account)?)
        }
        (SemanticPassResult::Empty, Some(_)) | (SemanticPassResult::Resource(_), None) => {
            return Err(GraphValidationError::InvalidPassArity);
        }
    };
    Ok(GraphLoweringPassKind::DropShadowColorize(shadow))
}

fn graph_lowering_composite_pass(
    filter_steps: QualifiedFilterSteps<'_>,
    pass: &SemanticGraphPass,
    account: &Account,
) -> GraphBuildResult<GraphLoweringPassKind> {
    let graph = filter_steps.source;
    let composites = matching(graph.composites.as_slice(), account, |composite| {
        composite.pass == pass.id
    })?;
    let composite = match (pass.result, composites) {
        (SemanticPassResult::Empty, Matches::None) => None,
        (SemanticPassResult::Resource(_), Matches::One(composite)) => {
            let step = filter_steps.find(pass.id, account)?;
            if composite.kind == SemanticCompositeKind::DropShadow {
                let step = step.ok_or(GraphValidationError::InvalidPassArity)?;
                let colorize_id = step
                    .passes
                    .get(2)
                    .ok_or(GraphValidationError::InvalidPassArity)?;
                account.work(RenderWork::GraphEdgeVisits, 1)?;
                let colorize = matching(&graph.passes, account, |candidate| {
                    candidate.id == *colorize_id
                })?
                .first()
                .ok_or(GraphValidationError::InvalidPassArity)?;
                graph_lowering_drop_shadow(graph, colorize, step, account)?;
            } else if step.is_some() {
                return Err(GraphValidationError::InvalidPassArity);
            }
            Some(graph_lowering_composite(composite, account)?)
        }
        _ => {
            return Err(GraphValidationError::InvalidPassArity);
        }
    };
    Ok(GraphLoweringPassKind::Composite(composite))
}

fn graph_lowering_vello_span(span: &SemanticVelloSpan) -> GraphLoweringVelloSpan {
    GraphLoweringVelloSpan {
        scope: match span.scope {
            SemanticVelloSpanScope::CurrentParent => GraphLoweringVelloSpanScope::CurrentParent,
            SemanticVelloSpanScope::LayerSource => GraphLoweringVelloSpanScope::LayerSource,
        },
        source_space: span.source_space,
        commands: span.commands.clone(),
        capture_transform: span.capture_transform,
        parent_to_surface: span.parent_to_surface,
        antialiasing: span.antialiasing,
        captured_before_outer_semantics: span.captured_before_outer_semantics,
    }
}

fn graph_lowering_clip_coverage(
    coverage: &SemanticClipCoverage,
    account: &Account,
) -> GraphBuildResult<GraphLoweringClipCoverage> {
    Ok(GraphLoweringClipCoverage {
        elements: mapped_shared(
            &coverage.elements,
            account,
            RenderWork::GraphNodeVisits,
            |element| {
                Ok(GraphLoweringClipCoverageElement {
                    clip: element.clip.clone(),
                    transform: element.transform,
                })
            },
        )?,
        antialiasing: coverage.antialiasing,
    })
}

/// Borrowed filter steps whose pass IDs are nonempty, disjoint and ordered.
#[derive(Clone, Copy, Debug)]
pub(super) struct QualifiedFilterSteps<'graph> {
    source: &'graph GpuRenderGraph,
}

impl<'graph> QualifiedFilterSteps<'graph> {
    pub(super) fn try_new(
        graph: &'graph GpuRenderGraph,
        account: &Account,
    ) -> GraphBuildResult<Self> {
        let mut previous = None;
        for step in graph.filter_steps.iter() {
            account.work(RenderWork::GraphNodeVisits, 1)?;
            let intents: &[SemanticPassIntent] = match &step.step.operation_intent {
                ResolvedFilterOperationIntent::ColorRun(_) => &[SemanticPassIntent::ColorFilter],
                ResolvedFilterOperationIntent::Blur(_) => &[
                    SemanticPassIntent::BlurHorizontal {
                        input: BlurInput::Rgba,
                    },
                    SemanticPassIntent::BlurVertical {
                        input: BlurInput::Rgba,
                    },
                ],
                ResolvedFilterOperationIntent::DropShadow(_) => &[
                    SemanticPassIntent::BlurHorizontal {
                        input: BlurInput::SourceAlpha,
                    },
                    SemanticPassIntent::BlurVertical {
                        input: BlurInput::SourceAlpha,
                    },
                    SemanticPassIntent::DropShadowColorize,
                    SemanticPassIntent::Composite,
                ],
            };
            if step.passes.len() != intents.len() {
                return Err(GraphValidationError::InvalidPassArity);
            }
            for (id, intent) in step.passes.iter().zip(intents) {
                account.work(RenderWork::GraphEdgeVisits, 1)?;
                if id.generation != graph.generation {
                    return Err(GraphValidationError::WrongPassGeneration {
                        expected: graph.generation,
                        actual: id.generation,
                    });
                }
                if previous.is_some_and(|previous| previous >= id.index.0) {
                    return Err(GraphValidationError::InvalidPassArity);
                }
                account.work(RenderWork::GraphNodeVisits, 1)?;
                let pass = graph
                    .passes
                    .get(id.index.as_usize()?)
                    .filter(|pass| pass.id == *id)
                    .ok_or(GraphValidationError::UnknownPass(*id))?;
                if !matches!(pass.result, SemanticPassResult::Resource(_)) || pass.intent != *intent
                {
                    return Err(GraphValidationError::InvalidPassArity);
                }
                previous = Some(id.index.0);
            }
        }
        Ok(Self { source: graph })
    }

    pub(super) fn validate_pass_projection(
        self,
        pass: &'graph SemanticGraphPass,
        account: &Account,
    ) -> Result<()> {
        self.pass_view(pass).project(account, |_| Ok(())).map(drop)
    }

    fn pass_view(self, pass: &'graph SemanticGraphPass) -> GraphLoweringPassView<'graph> {
        GraphLoweringPassView { graph: self, pass }
    }

    fn find(
        self,
        pass: SemanticPassId,
        account: &Account,
    ) -> GraphBuildResult<Option<&'graph SemanticFilterStepPlan>> {
        let steps = &self.source.filter_steps;
        let mut low = 0;
        let mut high = steps.len();
        let mut candidate = None;
        while low < high {
            let middle = low + (high - low) / 2;
            account.work(RenderWork::GraphNodeVisits, 1)?;
            let step = steps
                .get(middle)
                .ok_or(GraphValidationError::InvalidPassArity)?;
            account.work(RenderWork::GraphEdgeVisits, 1)?;
            let first = step
                .passes
                .first()
                .ok_or(GraphValidationError::InvalidPassArity)?;
            if first.index.0 <= pass.index.0 {
                candidate = Some(step);
                low = middle + 1;
            } else {
                high = middle;
            }
        }

        // Only the step with the nearest start at or before this pass can
        // contain it. A pass between steps still has no filter metadata.
        let Some(step) = candidate else {
            return Ok(None);
        };
        for id in step.passes.iter() {
            account.work(RenderWork::GraphEdgeVisits, 1)?;
            if *id == pass {
                return Ok(Some(step));
            }
        }
        Ok(None)
    }
}

fn graph_lowering_filter_spatial(
    spatial: ResolvedFilterSpatialMapping,
) -> GraphLoweringFilterSpatialMapping {
    GraphLoweringFilterSpatialMapping {
        source: graph_lowering_spatial(spatial.source),
        result: graph_lowering_spatial(spatial.result),
    }
}

fn graph_lowering_spatial(spatial: NonEmptyFrameSpatialPlan) -> GraphLoweringSpatialDescriptor {
    GraphLoweringSpatialDescriptor {
        logical_bounds: spatial.logical_bounds.rect(),
        device_origin: (spatial.device_origin.x, spatial.device_origin.y),
        device_extent: PhysicalSize::new(spatial.device_extent.width, spatial.device_extent.height),
        texel_origin: spatial.texel_center_mapping.origin,
        raster_scale: spatial.raster_scale.get(),
    }
}

fn graph_lowering_edge(edge: FilterEdgePolicy) -> GraphLoweringEdgePolicy {
    match edge {
        FilterEdgePolicy::NoSampling => GraphLoweringEdgePolicy::NoSampling,
        FilterEdgePolicy::TransparentBlack => GraphLoweringEdgePolicy::TransparentBlack,
        FilterEdgePolicy::SemanticBorderMirror { semantic_border } => {
            GraphLoweringEdgePolicy::SemanticBorderMirror(semantic_border.rect())
        }
    }
}

fn graph_lowering_blur(
    graph: &GpuRenderGraph,
    pass: &SemanticGraphPass,
    input: BlurInput,
    step: &SemanticFilterStepPlan,

    account: &Account,
) -> GraphBuildResult<GraphLoweringBlur> {
    let (standard_deviation, support_radius, expected_len) =
        match (&step.step.operation_intent, input) {
            (ResolvedFilterOperationIntent::Blur(intent), BlurInput::Rgba) => (
                intent.authored_blur.radius(),
                intent.support.device_radius,
                2,
            ),
            (ResolvedFilterOperationIntent::DropShadow(intent), BlurInput::SourceAlpha) => (
                intent.authored_shadow.blur().radius(),
                intent.support.device_radius,
                4,
            ),
            (ResolvedFilterOperationIntent::ColorRun(_), _)
            | (ResolvedFilterOperationIntent::Blur(_), BlurInput::SourceAlpha)
            | (ResolvedFilterOperationIntent::DropShadow(_), BlurInput::Rgba) => {
                return Err(GraphValidationError::InvalidPassArity);
            }
        };
    let axis_offset = usize::from(matches!(
        pass.intent,
        SemanticPassIntent::BlurVertical { .. }
    ));
    account.work(RenderWork::GraphEdgeVisits, 1)?;
    if step.passes.len() != expected_len || step.passes.get(axis_offset) != Some(&pass.id) {
        return Err(GraphValidationError::InvalidPassArity);
    }
    Ok(GraphLoweringBlur {
        input: match input {
            BlurInput::Rgba => GraphLoweringBlurInput::Rgba,
            BlurInput::SourceAlpha => GraphLoweringBlurInput::SourceAlpha,
        },
        standard_deviation,
        support_radius,
        spatial: graph_lowering_pass_spatial(graph, pass, account)?,
        edge: graph_lowering_edge(step.step.edge_policy),
    })
}

fn graph_lowering_drop_shadow(
    graph: &GpuRenderGraph,
    pass: &SemanticGraphPass,
    step: &SemanticFilterStepPlan,

    account: &Account,
) -> GraphBuildResult<GraphLoweringDropShadow> {
    let ResolvedFilterOperationIntent::DropShadow(intent) = &step.step.operation_intent else {
        return Err(GraphValidationError::InvalidPassArity);
    };
    if step.passes.len() != 4 {
        return Err(GraphValidationError::InvalidPassArity);
    }
    Ok(GraphLoweringDropShadow {
        offset: intent.authored_shadow.offset(),
        standard_deviation: intent.authored_shadow.blur().radius(),
        color: intent.authored_shadow.color(),
        support_radius: intent.support.device_radius,
        spatial: graph_lowering_pass_spatial(graph, pass, account)?,
        edge: graph_lowering_edge(step.step.edge_policy),
        source_alpha: intent.alpha_source == DropShadowAlphaSource::SourceAlpha,
        continuous_offset: intent.offset_sampling == DropShadowOffsetSampling::ContinuousLinear,
        retains_unchanged_source: intent.source_composition
            == DropShadowSourceComposition::RetainUnchangedForSourceOver,
    })
}

fn graph_lowering_pass_spatial(
    graph: &GpuRenderGraph,
    pass: &SemanticGraphPass,

    account: &Account,
) -> GraphBuildResult<GraphLoweringFilterSpatialMapping> {
    let [source] = pass.reads.as_slice() else {
        return Err(GraphValidationError::InvalidPassArity);
    };
    let SemanticPassResult::Resource(result) = pass.result else {
        return Err(GraphValidationError::InvalidPassArity);
    };
    account.work(RenderWork::GraphEdgeVisits, 1)?;
    account.work(RenderWork::GraphNodeVisits, 1)?;
    let source = graph
        .resources
        .get(source.index.as_usize()?)
        .ok_or(GraphValidationError::UnknownResource(*source))?;
    account.work(RenderWork::GraphNodeVisits, 1)?;
    let result = graph
        .resources
        .get(result.index.as_usize()?)
        .ok_or(GraphValidationError::UnknownResource(result))?;
    Ok(GraphLoweringFilterSpatialMapping {
        source: GraphLoweringSpatialDescriptor::from_semantic(source.descriptor),
        result: GraphLoweringSpatialDescriptor::from_semantic(result.descriptor),
    })
}

fn graph_lowering_composite(
    composite: &SemanticCompositePlan,
    account: &Account,
) -> GraphBuildResult<GraphLoweringComposite> {
    Ok(GraphLoweringComposite {
        kind: match &composite.kind {
            SemanticCompositeKind::SpanSourceOver => GraphLoweringCompositeKind::SpanSourceOver,
            SemanticCompositeKind::Layer {
                transform,
                destination_to_layer_local,
                source_space,
                opacity,
                composition,
                clip,
                outer_clips,
                clip_coverage,
                alpha_mask,
            } => GraphLoweringCompositeKind::Layer {
                transform: *transform,
                destination_to_layer_local: GraphLoweringDestinationToLayerLocal {
                    affine: destination_to_layer_local.affine,
                },
                source_space: *source_space,
                opacity: *opacity,
                composition: *composition,
                clip: clip.clone(),
                outer_clips: mapped_shared(
                    outer_clips,
                    account,
                    RenderWork::GraphNodeVisits,
                    |clip| {
                        Ok(GraphLoweringOuterClip {
                            clip: clip.clip.clone(),
                            transform: clip.transform,
                        })
                    },
                )?,
                clip_coverage: clip_coverage.map(GraphLoweringResourceId::from_semantic),
                alpha_mask: alpha_mask
                    .as_ref()
                    .map(|mask| {
                        SharedValue::new(
                            GraphLoweringResolvedAlphaMaskComposition {
                                resource: GraphLoweringResourceId::from_semantic(mask.resource),
                                bounds: mask.bounds,
                                image_dimensions: mask.image_dimensions,
                                quality: mask.quality,
                                rendering: mask.rendering,
                                extend: mask.extend,
                                value: mask.value,
                                application: mask.application,
                            },
                            &account.storage(),
                        )
                    })
                    .transpose()?,
            },
            SemanticCompositeKind::DropShadow => GraphLoweringCompositeKind::DropShadow,
        },
        source_captured_before_outer_semantics: composite.source_captured_before_outer_semantics,
    })
}

fn graph_lowering_read_bindings(
    pass: &SemanticGraphPass,
    kind: &GraphLoweringPassKind,
    account: &Account,
) -> GraphBuildResult<AccountedVec<GraphLoweringReadBinding>> {
    let make = |index: usize,
                role: GraphLoweringReadRole,
                sampling_filter: GraphLoweringSamplingFilter,
                sampling_edge: GraphLoweringSamplingEdge|
     -> GraphBuildResult<GraphLoweringReadBinding> {
        account.work(RenderWork::GraphEdgeVisits, 1)?;
        let resource = pass
            .reads
            .get(index)
            .copied()
            .ok_or(GraphValidationError::InvalidPassArity)?;
        Ok(GraphLoweringReadBinding {
            role,
            resource: GraphLoweringResourceId::from_semantic(resource),
            sampling_filter,
            sampling_edge,
        })
    };
    let bindings = match kind {
        GraphLoweringPassKind::ClearRoot { .. } | GraphLoweringPassKind::VelloCapture(None) => {
            if !pass.reads.is_empty() {
                return Err(GraphValidationError::InvalidPassArity);
            }
            account.temporary(0)?
        }
        GraphLoweringPassKind::VelloCapture(Some(_)) => {
            if !pass.reads.is_empty() {
                return Err(GraphValidationError::InvalidCaptureResult);
            }
            account.temporary(0)?
        }
        GraphLoweringPassKind::CanonicalizeCapture => temp_array(
            &[make(
                0,
                GraphLoweringReadRole::CaptureSource,
                GraphLoweringSamplingFilter::Linear,
                GraphLoweringSamplingEdge::ClampToExtent,
            )?],
            account,
        )?,
        GraphLoweringPassKind::CopyBackdrop => temp_array(
            &[make(
                0,
                GraphLoweringReadRole::CompletedParent,
                GraphLoweringSamplingFilter::Nearest,
                GraphLoweringSamplingEdge::TransparentBlack,
            )?],
            account,
        )?,
        GraphLoweringPassKind::ColorFilter(Some(filter)) => temp_array(
            &[make(
                0,
                GraphLoweringReadRole::FilterSource,
                GraphLoweringSamplingFilter::Nearest,
                sampling_edge_for_filter(filter.edge()),
            )?],
            account,
        )?,
        GraphLoweringPassKind::BlurHorizontal(Some(blur))
        | GraphLoweringPassKind::BlurVertical(Some(blur)) => temp_array(
            &[make(
                0,
                GraphLoweringReadRole::FilterSource,
                GraphLoweringSamplingFilter::GaussianKernel,
                sampling_edge_for_filter(blur.edge()),
            )?],
            account,
        )?,
        GraphLoweringPassKind::DropShadowColorize(Some(_)) => temp_array(
            &[make(
                0,
                GraphLoweringReadRole::BlurredSourceAlpha,
                GraphLoweringSamplingFilter::Linear,
                GraphLoweringSamplingEdge::TransparentBlack,
            )?],
            account,
        )?,
        GraphLoweringPassKind::ColorFilter(None)
        | GraphLoweringPassKind::BlurHorizontal(None)
        | GraphLoweringPassKind::BlurVertical(None)
        | GraphLoweringPassKind::DropShadowColorize(None)
        | GraphLoweringPassKind::Composite(None) => {
            if !pass.reads.is_empty() {
                return Err(GraphValidationError::InvalidPassArity);
            }
            account.temporary(0)?
        }
        GraphLoweringPassKind::Composite(Some(composite)) => {
            graph_lowering_composite_read_bindings(pass, composite, account)?
        }
        GraphLoweringPassKind::Present => temp_array(
            &[make(
                0,
                GraphLoweringReadRole::FinalWorkingImage,
                GraphLoweringSamplingFilter::Linear,
                GraphLoweringSamplingEdge::ClampToExtent,
            )?],
            account,
        )?,
    };
    if bindings.len() != pass.reads.len() {
        return Err(GraphValidationError::InvalidPassArity);
    }
    Ok(bindings)
}

fn graph_lowering_composite_read_bindings(
    pass: &SemanticGraphPass,
    composite: &GraphLoweringComposite,

    account: &Account,
) -> GraphBuildResult<AccountedVec<GraphLoweringReadBinding>> {
    let make = |index: usize,
                role: GraphLoweringReadRole,
                sampling_filter: GraphLoweringSamplingFilter,
                sampling_edge: GraphLoweringSamplingEdge|
     -> GraphBuildResult<GraphLoweringReadBinding> {
        account.work(RenderWork::GraphEdgeVisits, 1)?;
        let resource = pass
            .reads
            .get(index)
            .copied()
            .ok_or(GraphValidationError::InvalidPassArity)?;
        Ok(GraphLoweringReadBinding {
            role,
            resource: GraphLoweringResourceId::from_semantic(resource),
            sampling_filter,
            sampling_edge,
        })
    };
    let parent = || {
        make(
            0,
            GraphLoweringReadRole::CompositeParent,
            GraphLoweringSamplingFilter::Linear,
            GraphLoweringSamplingEdge::ClampToExtent,
        )
    };
    let source = |index| {
        make(
            index,
            GraphLoweringReadRole::CompositeSource,
            GraphLoweringSamplingFilter::Linear,
            GraphLoweringSamplingEdge::TransparentBlack,
        )
    };
    match composite.kind() {
        GraphLoweringCompositeKind::SpanSourceOver => {
            Ok(temp_array(&[parent()?, source(1)?], account)?)
        }
        GraphLoweringCompositeKind::Layer {
            clip_coverage,
            alpha_mask,
            ..
        } => {
            let mut bindings = temp_array(&[parent()?, source(1)?], account)?;
            let mut next_read = 2;
            if let Some(clip_coverage) = clip_coverage {
                let binding = make(
                    next_read,
                    GraphLoweringReadRole::ClipCoverage,
                    GraphLoweringSamplingFilter::Linear,
                    GraphLoweringSamplingEdge::TransparentBlack,
                )?;
                if binding.resource != *clip_coverage {
                    return Err(GraphValidationError::InvalidPassArity);
                }
                bindings.push(binding)?;
                next_read += 1;
            }
            if let Some(alpha_mask) = alpha_mask {
                let binding = make(
                    next_read,
                    GraphLoweringReadRole::AlphaMask,
                    GraphLoweringSamplingFilter::ImportedMask,
                    GraphLoweringSamplingEdge::ClampToExtent,
                )?;
                if binding.resource != alpha_mask.resource() {
                    return Err(GraphValidationError::InvalidPassArity);
                }
                bindings.push(binding)?;
            }
            Ok(bindings)
        }
        GraphLoweringCompositeKind::DropShadow => Ok(temp_array(
            &[
                source(0)?,
                make(
                    1,
                    GraphLoweringReadRole::Shadow,
                    GraphLoweringSamplingFilter::Linear,
                    GraphLoweringSamplingEdge::TransparentBlack,
                )?,
            ],
            account,
        )?),
    }
}

fn sampling_edge_for_filter(edge: GraphLoweringEdgePolicy) -> GraphLoweringSamplingEdge {
    match edge {
        GraphLoweringEdgePolicy::NoSampling => GraphLoweringSamplingEdge::ClampToExtent,
        GraphLoweringEdgePolicy::TransparentBlack => GraphLoweringSamplingEdge::TransparentBlack,
        GraphLoweringEdgePolicy::SemanticBorderMirror(bounds) => {
            GraphLoweringSamplingEdge::SemanticBorderMirror(bounds)
        }
    }
}

enum Matches<'a, T> {
    None,
    One(&'a T),
    Multiple { first: &'a T },
}
impl<'a, T> Matches<'a, T> {
    fn first(self) -> Option<&'a T> {
        match self {
            Self::None => None,
            Self::One(first) | Self::Multiple { first } => Some(first),
        }
    }
}
fn matching<'a, T>(
    values: &'a [T],
    account: &Account,
    mut predicate: impl FnMut(&T) -> bool,
) -> GraphBuildResult<Matches<'a, T>> {
    let mut result = Matches::None;
    for value in values {
        account.work(RenderWork::GraphNodeVisits, 1)?;
        if predicate(value) {
            result = match result {
                Matches::None => Matches::One(value),
                Matches::One(first) => Matches::Multiple { first },
                multiple @ Matches::Multiple { .. } => multiple,
            };
        }
    }
    Ok(result)
}

#[cfg(test)]
mod metadata_match_tests {
    use super::*;
    use crate::{
        RenderAccountingContext, RenderAccountingError, RenderAccountingOptions, RenderAttemptKind,
        RenderLimits, RenderQuantity, RenderRefusalReason,
    };

    #[test]
    fn metadata_scan_admits_later_values_after_multiple_matches() {
        let metadata = [7, 2, 9, 6, 11];
        for (limit, expected) in [(3, &[7, 2, 9][..]), (5, &[7, 2, 9, 6, 11][..])] {
            let options = RenderAccountingOptions::new().with_limits(
                RenderLimits::new()
                    .with_work_limit(RenderWork::GraphNodeVisits, limit)
                    .unwrap(),
            );
            let mut context = RenderAccountingContext::new(options);
            let mut inspected = Vec::new();
            let result = context.run(RenderAttemptKind::Frame, RenderPhase::Lower, |account| {
                graph_build(matching(&metadata, account, |value| {
                    inspected.push(*value);
                    value % 2 != 0
                }))
                .map(|_| ())
            });
            assert_eq!(inspected, expected);
            let report = context.last_attempt().unwrap().attempt().unwrap();
            assert_eq!(
                report
                    .phase_usage(RenderPhase::Lower)
                    .work(RenderWork::GraphNodeVisits),
                limit
            );
            if limit == 3 {
                let error = result.unwrap_err();
                let Some(RenderAccountingError::Refused(refusal)) = error.accounting_diagnostic()
                else {
                    panic!("the fourth value needs a graph node visit");
                };
                // The attempt owns the configured limit on Lower work.
                assert_eq!(refusal.phase(), RenderPhase::Attempt);
                assert_eq!(
                    refusal.quantity(),
                    RenderQuantity::Work(RenderWork::GraphNodeVisits)
                );
                assert_eq!((refusal.current(), refusal.amount()), (3, 1));
                assert_eq!(refusal.reason(), RenderRefusalReason::Limit { limit: 3 });
            } else {
                result.unwrap();
                assert!(report.refusal().is_none());
            }
        }
    }
}

fn mapped_shared<T, U>(
    values: &[T],
    account: &Account,
    work: RenderWork,
    mut map: impl FnMut(&T) -> Result<U>,
) -> Result<SharedValue<Vec<U>>> {
    let mut result = account.retained(values.len())?;
    for value in values {
        account.work(work, 1)?;
        result.append_slot()?.write(map(value)?);
    }
    SharedValue::from_accounted(result, &account.storage())
}
fn temp_array<T: Clone>(values: &[T], account: &Account) -> Result<AccountedVec<T>> {
    let mut result = account.temporary(values.len())?;
    result.extend_from_slice(values)?;
    Ok(result)
}

#[cfg(test)]
mod resource_binding_tests {
    use super::*;
    use crate::{
        Image, Layer, RenderAccountingContext, RenderAccountingError, RenderAccountingOptions,
        RenderAttemptKind, RenderLimits, RenderQuantity, RenderRefusalReason, RenderResource,
        ResolvedLayerAlphaMask, Scene, Size,
    };

    fn masked_graph(setup: &Account) -> crate::accounting::OwnedBox<GpuRenderGraph> {
        let bounds = Rect::new(0.0, 0.0, 2.0, 2.0);
        let image = Image::from_rgba(Size::new(1.0, 1.0), vec![255; 4]).unwrap();
        let mask = ResolvedLayerAlphaMask::try_new(image, bounds).unwrap();
        let mut scene = Scene::new();
        scene.layer(Layer::new().with_resolved_alpha_mask(mask), |child| {
            child.fill(bounds, Color::BLACK);
        });
        let commands = scene
            .normalize(crate::Capabilities::CURRENT, setup)
            .unwrap();
        let frame = crate::frame::FrameContext::try_new(
            Size::new(2.0, 2.0),
            1.0,
            Antialiasing::Area,
            Color::TRANSPARENT,
        )
        .unwrap();
        let crate::frame::FramePlan::GpuGraph(graph) = commands.plan_for(frame, setup).unwrap()
        else {
            panic!("a masked scene needs graph lowering");
        };
        graph
    }

    #[test]
    fn qualified_resource_materialization_accounts_for_scan_and_backing() {
        let setup = Account::for_test();
        let graph = masked_graph(&setup);
        let view = graph.lowering_view(&setup).unwrap();
        assert_eq!(graph.imports.len(), 1);
        let count = graph.resources.len();
        let passes = view.passes(&setup).unwrap();
        assert!(count > 1);
        let expected_visits = u64::try_from(count * 2).unwrap();
        let element_bytes = std::mem::size_of::<GraphLoweringResourceView<'_>>();
        let requested_bytes = u64::try_from(count * element_bytes).unwrap();
        let mut success = RenderAccountingContext::new(RenderAccountingOptions::new());
        let actual_bytes = success
            .run(RenderAttemptKind::Frame, RenderPhase::Lower, |account| {
                let resources = view.resources(account)?;
                assert_eq!(resources.len(), count);
                let mut imports = 0;
                for resource in resources.iter() {
                    assert!(passes.iter().any(|pass| pass.id() == resource.last_use()));
                    match resource.producer() {
                        GraphLoweringResourceProducer::Imported => {
                            imports += 1;
                            assert!(resource.import().is_some());
                        }
                        GraphLoweringResourceProducer::Pass(producer) => {
                            assert!(passes.iter().any(|pass| pass.id() == producer));
                            assert!(resource.import().is_none());
                        }
                    }
                }
                assert_eq!(imports, 1);
                Ok(u64::try_from(resources.capacity() * element_bytes).unwrap())
            })
            .unwrap();
        let report = success.last_attempt().unwrap().attempt().unwrap();
        let lower = report.phase_usage(RenderPhase::Lower);
        assert_eq!(lower.work(RenderWork::GraphNodeVisits), expected_visits);
        assert_eq!(lower.work(RenderWork::GraphEdgeVisits), 0);
        assert_eq!(
            report
                .phase_usage(RenderPhase::Preparation)
                .peak(RenderResource::ScratchCpuBytes),
            actual_bytes
        );
        eprintln!(
            "{count} resource views, {element_bytes} bytes per view, {actual_bytes} bytes of scratch backing"
        );
        assert_eq!(report.usage().live(RenderResource::ScratchCpuBytes), 0);

        for (limits, quantity, current, amount, limit, visits) in [
            (
                RenderLimits::new()
                    .with_work_limit(RenderWork::GraphNodeVisits, expected_visits - 1)
                    .unwrap(),
                RenderQuantity::Work(RenderWork::GraphNodeVisits),
                expected_visits - 1,
                1,
                expected_visits - 1,
                expected_visits - 1,
            ),
            (
                RenderLimits::new().with_scratch_bytes(requested_bytes - 1),
                RenderQuantity::Resource(RenderResource::ScratchCpuBytes),
                0,
                requested_bytes,
                requested_bytes - 1,
                0,
            ),
        ] {
            let mut context =
                RenderAccountingContext::new(RenderAccountingOptions::new().with_limits(limits));
            let error = context
                .run(RenderAttemptKind::Frame, RenderPhase::Lower, |account| {
                    view.resources(account).map(|_| ())
                })
                .unwrap_err();
            let Some(RenderAccountingError::Refused(refusal)) = error.accounting_diagnostic()
            else {
                panic!("resource view admission must return the typed refusal");
            };
            assert_eq!(refusal.phase(), RenderPhase::Attempt);
            assert_eq!(refusal.quantity(), quantity);
            assert_eq!((refusal.current(), refusal.amount()), (current, amount));
            assert_eq!(refusal.reason(), RenderRefusalReason::Limit { limit });
            let report = context.last_attempt().unwrap().attempt().unwrap();
            assert_eq!(
                report
                    .phase_usage(RenderPhase::Lower)
                    .work(RenderWork::GraphNodeVisits),
                visits
            );
            let peak = report
                .phase_usage(RenderPhase::Preparation)
                .peak(RenderResource::ScratchCpuBytes);
            assert_eq!(peak, if visits == 0 { 0 } else { actual_bytes });
            assert_eq!(report.usage().live(RenderResource::ScratchCpuBytes), 0);
        }
    }

    #[test]
    fn incomplete_resource_bindings_fail_before_materialization() {
        let setup = Account::for_test();
        let mut graph = masked_graph(&setup);
        let index = graph
            .resources
            .iter()
            .position(|resource| {
                matches!(resource.producer, Some(SemanticResourceProducer::Pass(_)))
            })
            .unwrap();
        let id = graph.resources[index].id;
        let producer = graph.resources[index].producer.take();
        let error = graph.lowering_view(&setup).unwrap_err();
        assert_eq!(error.code(), crate::ErrorCode::RenderFailed);
        assert_eq!(
            error.message(),
            format!("semantic frame graph validation failed: ResourceWithoutProducer({id:?})")
        );
        graph.resources[index].producer = producer;

        let last_use = graph.resources[index].releasable_after.take();
        let error = graph.lowering_view(&setup).unwrap_err();
        assert_eq!(error.code(), crate::ErrorCode::RenderFailed);
        assert_eq!(
            error.message(),
            format!(
                "semantic frame graph validation failed: UnscheduledReads {{ resource: {id:?}, remaining: 0 }}"
            )
        );
        graph.resources[index].releasable_after = last_use;

        assert_eq!(graph.imports.len(), 1);
        drop(graph.imports.pop().unwrap());
        let error = graph.lowering_view(&setup).unwrap_err();
        assert_eq!(error.code(), crate::ErrorCode::RenderFailed);
        assert_eq!(
            error.message(),
            "semantic frame graph validation failed: InvalidImportedResourceRole"
        );
    }
}
