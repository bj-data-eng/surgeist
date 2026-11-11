use super::graph::{
    GpuRenderGraph, GraphBuildResult, GraphValidationError, PassIndex, ResourceIndex,
    SemanticCompositeKind, SemanticGraphPass, SemanticPassId, SemanticPassIntent,
    SemanticPassResult, SemanticResourceId, SemanticResourceProducer, SemanticResourceRole,
    WorkingImageInitialization, graph_build,
};
use super::lower::{graph_lowering_pass_kind, graph_lowering_read_bindings};
use crate::accounting::{Account, AccountedVec, RenderWork, StorageContractError};

pub(super) fn validate_semantic_frame_graph(
    graph: &GpuRenderGraph,
    account: &Account,
) -> GraphBuildResult<()> {
    validate_vello_span_metadata(graph, account)?;
    validate_clip_coverage_metadata(graph, account)?;
    validate_backdrop_metadata(graph, account)?;
    validate_import_metadata(graph, account)?;
    if any_node(&graph.passes, account, |pass| {
        Ok(
            matches!(pass.intent, SemanticPassIntent::VelloCapture { .. })
                && !pass.reads.is_empty(),
        )
    })? {
        return Err(GraphValidationError::InvalidCaptureResult);
    }
    Ok(())
}

fn validate_vello_span_metadata(graph: &GpuRenderGraph, account: &Account) -> GraphBuildResult<()> {
    for span in &graph.vello_spans {
        account.work(RenderWork::GraphNodeVisits, 1)?;
        let pass = find_node(&graph.passes, account, |pass| {
            Ok(pass.id == span.capture_pass)
        })?
        .ok_or(GraphValidationError::UnknownPass(span.capture_pass))?;
        let SemanticPassResult::Resource(capture) = pass.result else {
            return Err(GraphValidationError::InvalidCaptureResult);
        };
        if !matches!(
            pass.intent,
            SemanticPassIntent::VelloCapture {
                initialization: WorkingImageInitialization::Transparent
            }
        ) || !pass.reads.is_empty()
            || !span.captured_before_outer_semantics
        {
            return Err(GraphValidationError::InvalidCaptureResult);
        }
        if span.source_space == super::CompositeSourceSpace::SurfaceGrid {
            let root = find_node(&graph.resources, account, |resource| {
                Ok(resource.id == graph.root_working_image)
            })?
            .ok_or(GraphValidationError::UnknownResource(
                graph.root_working_image,
            ))?;
            let source = find_node(&graph.resources, account, |resource| {
                Ok(resource.id == capture)
            })?
            .ok_or(GraphValidationError::UnknownResource(capture))?;
            if span.parent_to_surface != crate::Transform::identity()
                || source.descriptor.texel_center_mapping.raster_scale
                    != root.descriptor.texel_center_mapping.raster_scale
            {
                return Err(GraphValidationError::InvalidCaptureResult);
            }
        }
        let canonical_consumers = count_nodes(&graph.passes, account, |candidate| {
            Ok(candidate.intent == SemanticPassIntent::CanonicalizeCapture
                && edges_equal(&candidate.reads, &[capture], account)?)
        })?;
        if canonical_consumers != 1 {
            return Err(GraphValidationError::InvalidCaptureResult);
        }
    }
    Ok(())
}

fn validate_clip_coverage_metadata(
    graph: &GpuRenderGraph,
    account: &Account,
) -> GraphBuildResult<()> {
    for coverage in &graph.clip_coverages {
        account.work(RenderWork::GraphNodeVisits, 1)?;
        let pass = find_node(&graph.passes, account, |pass| {
            Ok(pass.id == coverage.capture_pass)
        })?
        .ok_or(GraphValidationError::UnknownPass(coverage.capture_pass))?;
        let SemanticPassResult::Resource(capture) = pass.result else {
            return Err(GraphValidationError::InvalidCaptureResult);
        };
        let resource = find_node(&graph.resources, account, |resource| {
            Ok(resource.id == capture)
        })?
        .ok_or(GraphValidationError::UnknownResource(capture))?;
        if !matches!(
            pass.intent,
            SemanticPassIntent::VelloCapture {
                initialization: WorkingImageInitialization::Transparent
            }
        ) || !pass.reads.is_empty()
            || resource.descriptor.role != SemanticResourceRole::ClipCoverage
            || coverage.elements.is_empty()
        {
            return Err(GraphValidationError::InvalidCaptureResult);
        }
        let composite_consumers = count_nodes(&graph.composites, account, |composite| {
            if !matches!(&composite.kind, SemanticCompositeKind::Layer { clip_coverage: Some(resource), .. } if *resource == capture)
            {
                return Ok(false);
            }
            let candidate = find_node(&graph.passes, account, |candidate| {
                Ok(candidate.id == composite.pass)
            })?;
            match candidate {
                Some(candidate) => contains_edge(&candidate.reads, capture, account),
                None => Ok(false),
            }
        })?;
        if composite_consumers != 1 {
            return Err(GraphValidationError::InvalidCaptureResult);
        }
    }
    Ok(())
}

fn validate_backdrop_metadata(graph: &GpuRenderGraph, account: &Account) -> GraphBuildResult<()> {
    for backdrop in &graph.backdrop_reads {
        account.work(RenderWork::GraphNodeVisits, 1)?;
        let pass = find_node(&graph.passes, account, |pass| Ok(pass.id == backdrop.pass))?
            .ok_or(GraphValidationError::UnknownPass(backdrop.pass))?;
        if pass.intent != SemanticPassIntent::CopyBackdrop
            || !edges_equal(&pass.reads, &[backdrop.completed_parent], account)?
        {
            return Err(GraphValidationError::InvalidPassArity);
        }
    }
    Ok(())
}

fn validate_import_metadata(graph: &GpuRenderGraph, account: &Account) -> GraphBuildResult<()> {
    for import in &graph.imports {
        account.work(RenderWork::GraphNodeVisits, 1)?;
        let resource = find_node(&graph.resources, account, |resource| {
            Ok(resource.id == import.resource)
        })?
        .ok_or(GraphValidationError::UnknownResource(import.resource))?;
        if resource.descriptor.role != SemanticResourceRole::ImportedImage
            || resource.producer != Some(SemanticResourceProducer::Imported)
        {
            return Err(GraphValidationError::InvalidImportedResourceRole);
        }
    }
    Ok(())
}

pub(super) fn validate_lowering_graph(
    graph: &GpuRenderGraph,
    account: &Account,
) -> crate::Result<()> {
    let mut validation = graph_build(LoweringValidationState::begin(graph, account))?;
    for (index, pass) in graph.passes.iter().enumerate() {
        graph_build(validation.validate_pass(index, pass))?;
        graph_build(graph_lowering_pass_kind(graph, pass, account))?;
        graph_build(graph_lowering_read_bindings(graph, pass, account))?;
    }
    graph_build(validation.finish())
}

struct LoweringValidationState<'graph, 'account> {
    account: &'account Account,
    graph: &'graph GpuRenderGraph,
    actual_reads: AccountedVec<u32>,
    last_reads: AccountedVec<Option<SemanticPassId>>,
}

impl<'graph, 'account> LoweringValidationState<'graph, 'account> {
    fn begin(graph: &'graph GpuRenderGraph, account: &'account Account) -> GraphBuildResult<Self> {
        validate_semantic_frame_graph(graph, account)?;
        if graph.resources.is_empty() {
            return Err(GraphValidationError::MissingRootWorkingImage);
        }
        if graph.passes.is_empty() {
            return Err(GraphValidationError::MissingFinalPresent);
        }

        let mut actual_reads = account.temporary(graph.resources.len())?;
        actual_reads.resize(graph.resources.len(), 0)?;
        let mut last_reads = account.temporary(graph.resources.len())?;
        last_reads.resize(graph.resources.len(), None)?;
        validate_lowering_resources(graph, account)?;
        validate_lowering_imports(graph, account)?;
        Ok(Self {
            account,
            graph,
            actual_reads,
            last_reads,
        })
    }

    fn validate_pass(&mut self, index: usize, pass: &SemanticGraphPass) -> GraphBuildResult<()> {
        let account = self.account;
        account.work(RenderWork::GraphNodeVisits, 1)?;
        let graph = self.graph;
        let expected_id = SemanticPassId::new(graph.generation, PassIndex::try_from_len(index)?);
        if pass.id != expected_id {
            return if pass.id.generation != graph.generation {
                Err(GraphValidationError::WrongPassGeneration {
                    expected: graph.generation,
                    actual: pass.id.generation,
                })
            } else {
                Err(GraphValidationError::UnknownPass(pass.id))
            };
        }
        if !pass.scheduled {
            return Err(GraphValidationError::UnscheduledPass(pass.id));
        }

        let mut seen_dependencies = account.temporary(pass.dependencies.len())?;
        for dependency in pass.dependencies.iter() {
            account.work(RenderWork::GraphEdgeVisits, 1)?;
            if contains_edge(&seen_dependencies, *dependency, account)? {
                return Err(GraphValidationError::DuplicateDependency(*dependency));
            }
            let dependency_index = validate_graph_pass_id(graph, *dependency, account)?;
            if dependency_index >= index {
                return Err(GraphValidationError::ForwardDependency(*dependency));
            }
            seen_dependencies.append_slot()?.write(*dependency);
        }

        let mut seen_reads = account.temporary(pass.reads.len())?;
        for read in pass.reads.iter() {
            account.work(RenderWork::GraphEdgeVisits, 1)?;
            if contains_edge(&seen_reads, *read, account)? {
                return Err(GraphValidationError::DuplicateRead(*read));
            }
            let resource_index = validate_graph_resource_id(graph, *read, account)?;
            let resource = graph
                .resources
                .get(resource_index)
                .ok_or(GraphValidationError::UnknownResource(*read))?;
            if pass.result == SemanticPassResult::Resource(*read) {
                return Err(GraphValidationError::ReadWriteAlias(*read));
            }
            if let Some(SemanticResourceProducer::Pass(producer)) = resource.producer {
                let producer_index = validate_graph_pass_id(graph, producer, account)?;
                if producer_index >= index {
                    return Err(GraphValidationError::ForwardRead(*read));
                }
                if !contains_edge(&pass.dependencies, producer, account)? {
                    return Err(GraphValidationError::MissingProducerDependency {
                        resource: *read,
                        producer,
                    });
                }
            }
            let count_len = self.actual_reads.len();
            let count = self
                .actual_reads
                .get_mut(resource_index)
                .ok_or_else(|| invalid_read_slot(resource_index, count_len))?;
            *count = count
                .checked_add(1)
                .ok_or(GraphValidationError::ReadCountOverflow(*read))?;
            let last_len = self.last_reads.len();
            let last = self
                .last_reads
                .get_mut(resource_index)
                .ok_or_else(|| invalid_read_slot(resource_index, last_len))?;
            *last = Some(pass.id);
            seen_reads.append_slot()?.write(*read);
        }
        if let SemanticPassResult::Resource(result) = pass.result {
            let resource_index = validate_graph_resource_id(graph, result, account)?;
            let resource = graph
                .resources
                .get(resource_index)
                .ok_or(GraphValidationError::UnknownResource(result))?;
            if resource.producer != Some(SemanticResourceProducer::Pass(pass.id)) {
                return Err(GraphValidationError::DuplicateProducer(result));
            }
        }

        Ok(())
    }

    fn finish(self) -> GraphBuildResult<()> {
        let account = self.account;
        for (index, resource) in self.graph.resources.iter().enumerate() {
            account.work(RenderWork::GraphNodeVisits, 1)?;
            let count = *self
                .actual_reads
                .get(index)
                .ok_or_else(|| invalid_read_slot(index, self.actual_reads.len()))?;
            if resource.descriptor.expected_reads != count || resource.recorded_reads != count {
                return Err(GraphValidationError::DeclaredReadCountMismatch {
                    resource: resource.id,
                    declared: resource.descriptor.expected_reads,
                    recorded: count,
                });
            }
            let last_read = self
                .last_reads
                .get(index)
                .ok_or_else(|| invalid_read_slot(index, self.last_reads.len()))?;
            let Some(last_read) = *last_read else {
                return Err(GraphValidationError::OrphanResult(resource.id));
            };
            if resource.releasable_after != Some(last_read) {
                return Err(GraphValidationError::UnscheduledReads {
                    resource: resource.id,
                    remaining: 0,
                });
            }
        }
        validate_lowering_anchors(self.graph, account)
    }
}

fn invalid_read_slot(index: usize, len: usize) -> GraphValidationError {
    crate::Error::from(StorageContractError::InvalidPosition { index, len }).into()
}

fn validate_lowering_resources(graph: &GpuRenderGraph, account: &Account) -> GraphBuildResult<()> {
    for (index, resource) in graph.resources.iter().enumerate() {
        account.work(RenderWork::GraphNodeVisits, 1)?;
        let expected_id =
            SemanticResourceId::new(graph.generation, ResourceIndex::try_from_len(index)?);
        if resource.id != expected_id {
            return if resource.id.generation != graph.generation {
                Err(GraphValidationError::WrongResourceGeneration {
                    expected: graph.generation,
                    actual: resource.id.generation,
                })
            } else {
                Err(GraphValidationError::UnknownResource(resource.id))
            };
        }
        if resource.remaining_reads != Some(0) {
            return Err(GraphValidationError::UnscheduledReads {
                resource: resource.id,
                remaining: resource.remaining_reads.unwrap_or(u32::MAX),
            });
        }
        let import_count = count_nodes(&graph.imports, account, |import| {
            Ok(import.resource == resource.id)
        })?;
        match resource.producer {
            Some(SemanticResourceProducer::Imported)
                if resource.descriptor.role == SemanticResourceRole::ImportedImage
                    && import_count == 1 => {}
            Some(SemanticResourceProducer::Imported) => {
                return Err(GraphValidationError::InvalidImportedResourceRole);
            }
            Some(SemanticResourceProducer::Pass(producer)) => {
                if import_count != 0 {
                    return Err(GraphValidationError::InvalidImportedResourceRole);
                }
                validate_graph_pass_id(graph, producer, account)?;
                let producer_pass = graph
                    .passes
                    .get(producer.index.as_usize()?)
                    .ok_or(GraphValidationError::UnknownPass(producer))?;
                if producer_pass.result != SemanticPassResult::Resource(resource.id) {
                    return Err(GraphValidationError::DuplicateProducer(resource.id));
                }
            }
            None => return Err(GraphValidationError::ResourceWithoutProducer(resource.id)),
        }
    }
    Ok(())
}

fn validate_lowering_imports(graph: &GpuRenderGraph, account: &Account) -> GraphBuildResult<()> {
    for import in &graph.imports {
        account.work(RenderWork::GraphNodeVisits, 1)?;
        let index = validate_graph_resource_id(graph, import.resource, account)?;
        let resource = graph
            .resources
            .get(index)
            .ok_or(GraphValidationError::UnknownResource(import.resource))?;
        if resource.producer != Some(SemanticResourceProducer::Imported)
            || resource.descriptor.role != SemanticResourceRole::ImportedImage
            || count_nodes(&graph.imports, account, |candidate| {
                Ok(candidate.resource == import.resource)
            })? != 1
        {
            return Err(GraphValidationError::InvalidImportedResourceRole);
        }
    }
    Ok(())
}

fn validate_lowering_anchors(graph: &GpuRenderGraph, account: &Account) -> GraphBuildResult<()> {
    let root_index = validate_graph_resource_id(graph, graph.root_working_image, account)?;
    if graph
        .resources
        .get(root_index)
        .is_none_or(|resource| resource.descriptor.role != SemanticResourceRole::RootWorkingImage)
    {
        return Err(GraphValidationError::MissingRootWorkingImage);
    }
    let present_index = validate_graph_pass_id(graph, graph.final_present, account)?;
    if present_index + 1 != graph.passes.len()
        || graph
            .passes
            .get(present_index)
            .is_none_or(|pass| pass.intent != SemanticPassIntent::Present)
    {
        return Err(GraphValidationError::MissingFinalPresent);
    }
    Ok(())
}

fn validate_graph_resource_id(
    graph: &GpuRenderGraph,
    id: SemanticResourceId,

    account: &Account,
) -> GraphBuildResult<usize> {
    account.work(RenderWork::GraphNodeVisits, 1)?;
    if id.generation != graph.generation {
        return Err(GraphValidationError::WrongResourceGeneration {
            expected: graph.generation,
            actual: id.generation,
        });
    }
    let index = id.index.as_usize()?;
    if graph.resources.get(index).is_none() {
        return Err(GraphValidationError::UnknownResource(id));
    }
    Ok(index)
}

fn validate_graph_pass_id(
    graph: &GpuRenderGraph,
    id: SemanticPassId,
    account: &Account,
) -> GraphBuildResult<usize> {
    account.work(RenderWork::GraphNodeVisits, 1)?;
    if id.generation != graph.generation {
        return Err(GraphValidationError::WrongPassGeneration {
            expected: graph.generation,
            actual: id.generation,
        });
    }
    let index = id.index.as_usize()?;
    if graph.passes.get(index).is_none() {
        return Err(GraphValidationError::UnknownPass(id));
    }
    Ok(index)
}

fn find_node<'a, T>(
    values: &'a [T],
    account: &Account,
    mut predicate: impl FnMut(&T) -> GraphBuildResult<bool>,
) -> GraphBuildResult<Option<&'a T>> {
    for value in values {
        account.work(RenderWork::GraphNodeVisits, 1)?;
        if predicate(value)? {
            return Ok(Some(value));
        }
    }
    Ok(None)
}
fn count_nodes<T>(
    values: &[T],
    account: &Account,
    mut predicate: impl FnMut(&T) -> GraphBuildResult<bool>,
) -> GraphBuildResult<usize> {
    let mut count = 0;
    for value in values {
        account.work(RenderWork::GraphNodeVisits, 1)?;
        if predicate(value)? {
            count += 1;
        }
    }
    Ok(count)
}
fn any_node<T>(
    values: &[T],
    account: &Account,
    mut predicate: impl FnMut(&T) -> GraphBuildResult<bool>,
) -> GraphBuildResult<bool> {
    for value in values {
        account.work(RenderWork::GraphNodeVisits, 1)?;
        if predicate(value)? {
            return Ok(true);
        }
    }
    Ok(false)
}
fn contains_edge<T: PartialEq>(
    values: &[T],
    needle: T,
    account: &Account,
) -> GraphBuildResult<bool> {
    for value in values {
        account.work(RenderWork::GraphEdgeVisits, 1)?;
        if *value == needle {
            return Ok(true);
        }
    }
    Ok(false)
}

fn edges_equal<T: PartialEq>(
    actual: &[T],
    expected: &[T],
    account: &Account,
) -> GraphBuildResult<bool> {
    if actual.len() != expected.len() {
        return Ok(false);
    }
    for (actual, expected) in actual.iter().zip(expected) {
        account.work(RenderWork::GraphEdgeVisits, 1)?;
        if actual != expected {
            return Ok(false);
        }
    }
    Ok(true)
}

#[cfg(test)]
mod repeated_proof_edge_tests {
    use super::*;
    use crate::accounting::{
        RenderAccountingContext, RenderAccountingOptions, RenderAttemptKind, RenderLimits,
        RenderPhase,
    };
    use crate::frame::{FrameContext, FramePlan, forced_base_graph_for_test};
    use crate::{
        Antialiasing, BackdropCaptureBounds, BackdropFilterInput, Capabilities, Color,
        FilterAmount, FilterList, FilterOp, Layer, Rect, Scene, Size,
    };

    fn graph(backdrop: bool) -> crate::accounting::OwnedBox<GpuRenderGraph> {
        let setup = Account::for_test();
        let mut scene = Scene::new();
        if backdrop {
            scene.fill(Rect::new(0.0, 0.0, 2.0, 2.0), Color::BLACK);
            let filters = FilterList::try_ops(vec![FilterOp::brightness(
                FilterAmount::try_new(1.25).unwrap(),
            )])
            .unwrap();
            let input = BackdropFilterInput::try_new(
                filters,
                BackdropCaptureBounds::try_new(Rect::new(0.0, 0.0, 2.0, 2.0)).unwrap(),
                None,
            )
            .unwrap();
            scene.layer(Layer::new().try_backdrop_filter(input).unwrap(), |child| {
                child.fill(Rect::new(0.0, 0.0, 1.0, 1.0), Color::BLACK);
            });
        } else {
            scene.fill(Rect::new(0.0, 0.0, 1.0, 1.0), Color::BLACK);
        }
        let commands = scene.normalize(Capabilities::CURRENT, &setup).unwrap();
        let context = FrameContext::try_new(
            Size::new(4.0, 4.0),
            1.0,
            Antialiasing::Area,
            Color::TRANSPARENT,
        )
        .unwrap();
        if backdrop {
            let FramePlan::GpuGraph(graph) = commands.plan_for(context, &setup).unwrap() else {
                panic!("backdrop selects a graph")
            };
            graph
        } else {
            crate::accounting::OwnedBox::new(
                forced_base_graph_for_test(commands, context).unwrap(),
                &setup.storage(),
            )
            .unwrap()
        }
    }

    #[test]
    fn repeated_proof_canonical_and_backdrop_metadata_admit_the_read_identity() {
        for backdrop in [false, true] {
            let graph = graph(backdrop);
            if backdrop {
                assert_eq!(graph.backdrop_reads.len(), 1);
            } else {
                assert_eq!(graph.vello_spans.len(), 1);
            }
            for limit in [1, 0] {
                let options = RenderAccountingOptions::new().with_limits(
                    RenderLimits::new()
                        .with_work_limit(RenderWork::GraphEdgeVisits, limit)
                        .unwrap(),
                );
                let mut context = RenderAccountingContext::new(options);
                let result = context.run(RenderAttemptKind::Frame, RenderPhase::Lower, |account| {
                    super::super::graph::graph_build(if backdrop {
                        validate_backdrop_metadata(&graph, account)
                    } else {
                        validate_vello_span_metadata(&graph, account)
                    })
                });
                let report = context.last_attempt().unwrap().attempt().unwrap();
                if limit == 1 {
                    result.unwrap();
                    assert_eq!(report.usage().work(RenderWork::GraphEdgeVisits), 1);
                } else {
                    assert!(
                        result.is_err(),
                        "the one reached read identity comparison must refuse"
                    );
                    let refusal = report.refusal().unwrap();
                    assert_eq!(
                        refusal.quantity,
                        crate::RenderQuantity::Work(RenderWork::GraphEdgeVisits)
                    );
                    assert_eq!((refusal.current, refusal.amount), (0, 1));
                }
            }
        }
    }
}
