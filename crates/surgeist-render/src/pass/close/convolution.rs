//! Closure proof for the complete, unclamped float32 convolution chain.
use super::*;
use crate::{
    PreparedConvolution, Rect,
    frame::{ConvolutionMapping, ConvolutionStage, GraphLoweringConvolution},
};

struct Chain {
    count: usize,
    source: RuntimeResourceId,
    result: RuntimeResourceId,
}

impl ClosedGraphTraversal<'_, '_> {
    pub(super) fn visit_convolution(&mut self) -> Option<()> {
        let context = *self.contexts.last()?;
        self.initialized_context_source(context)?;
        let chain = validate_chain(self.plan, self.cursor, context, &self.resources)?;
        let mut ids = self.proof.capture(AccountedVec::new(
            self.proof.account,
            chain.count,
            RenderResource::RetainedCpuBytes,
        ))?;
        for index in self.cursor..self.cursor.checked_add(chain.count)? {
            let pass = self.pass_at(index)?;
            self.proof.capture(ids.append_slot())?.write(pass.id);
            let RuntimeResultBinding::Resource(result) = pass.result else {
                return None;
            };
            self.expected_resources.insert(result);
        }
        let producer = ids.last().copied()?;
        let passes = self.proof.capture(SharedValue::from_accounted(
            ids,
            &self.proof.account.storage(),
        ))?;
        self.proof.capture_storage_write(self.filter_steps.push(
            ExecutableFilterStepFacts::Convolution {
                passes,
                source: chain.source,
                result: chain.result,
            },
        ))?;
        self.has_convolution = true;
        let context = self.contexts.last_mut()?;
        context.current = chain.result;
        context.producer = producer;
        // Even a transparent source can yield pixels through bias.
        context.contains_captured_source = true;
        self.advance(chain.count)
    }
}

pub(super) fn steps_are_exact(
    plan: &LoweredGraphPlan,
    steps: &[ExecutableFilterStepFacts],
    expected_convolution: bool,
    proof: &ProofContext<'_>,
) -> bool {
    let exact = (|| -> Option<()> {
        let has_convolution = proof_nodes(steps, proof)
            .any(|step| matches!(step, ExecutableFilterStepFacts::Convolution { .. }));
        if has_convolution != expected_convolution {
            return None;
        }
        if !has_convolution {
            return Some(());
        }
        let maps = proof.capture(ClosedGraphMaps::try_new(plan, proof))?;
        for step in proof_nodes(steps, proof) {
            let ExecutableFilterStepFacts::Convolution {
                passes,
                source,
                result,
            } = step
            else {
                continue;
            };
            let first = passes.first()?;
            let cursor = *maps.pass_positions.get(first)?;
            let resource = maps.resource_by_id.get(source)?;
            let RuntimeResourceProducer::Pass(producer) = resource.producer else {
                return None;
            };
            let context = ExecutableCompositionContext {
                current: *source,
                producer,
                contains_captured_source: true,
            };
            let chain = validate_chain(plan, cursor, context, &maps.resource_by_id)?;
            if chain.count != passes.len() || chain.source != *source || chain.result != *result {
                return None;
            }
            let end = cursor.checked_add(chain.count)?;
            if !proof_relationships_equal(
                plan.passes.get(cursor..end)?.iter().map(|p| p.id),
                passes.iter().copied(),
                proof,
                |a, b| a == b,
            ) {
                return None;
            }
        }
        Some(())
    })();
    exact.is_some() && !proof.invalid.get()
}

fn validate_chain(
    plan: &LoweredGraphPlan,
    cursor: usize,
    context: ExecutableCompositionContext,
    resources: &ProofMap<'_, RuntimeResourceId, &RuntimeResourceRequest>,
) -> Option<Chain> {
    let proof = resources.proof;
    let at = |offset: usize| {
        let pass = plan.passes.get(cursor.checked_add(offset)?)?;
        proof.work(RenderWork::GraphNodeVisits, 1).then_some(pass)
    };
    let first = at(0)?;
    let RuntimePassKind::Convolution(work) = &first.kind else {
        return None;
    };
    let operation = work.operation();
    let mapping = work.mapping();
    let source = resources.get(&context.current).copied()?;
    if source.producer != RuntimeResourceProducer::Pass(context.producer)
        || source.format != RuntimeResourceFormat::Working(plan.working_format)
        || !matches!(
            source.role,
            RuntimeResourceRole::FilterIntermediate
                | RuntimeResourceRole::CompositeResult
                | RuntimeResourceRole::IsolationWorkingImage
        )
        || source.expected_reads != 1
        || source.last_use != first.id
    {
        return None;
    }
    if work.stage() == ConvolutionStage::RebaseSource {
        return validate_rebase(first, at(1)?, work, source, resources, plan.working_format);
    }
    if !mapping_is_exact(operation, mapping, source.spatial, proof) {
        return None;
    }
    if mapping.source_domain().width() == 0.0 || mapping.source_domain().height() == 0.0 {
        // Frame creates the empty-domain dummy directly from its transparent
        // isolation clear; captured children cannot supply this binding.
        if source.role != RuntimeResourceRole::IsolationWorkingImage {
            return None;
        }
        let clear = proof_passes(plan, proof).find(|pass| pass.id == context.producer)?;
        if !matches!(&clear.kind, RuntimePassKind::ClearRoot {
            initialization: RuntimeInitialization::Transparent,
            color,
        } if *color == Color::TRANSPARENT)
            || clear.result != RuntimeResultBinding::Resource(source.id)
            || !clear.dependencies.is_empty()
            || !clear.reads.is_empty()
            || !clear.releases.is_empty()
            || clear.cache_keys.is_some()
        {
            return None;
        }
    }
    if operation.is_passthrough() {
        stage_is_exact(
            first,
            work,
            ConvolutionStage::PassThrough,
            &[source],
            &[source.id],
            proof,
        )?;
        let result = result_resource(
            first,
            resources,
            RuntimeResourceRole::FilterIntermediate,
            RuntimeResourceFormat::Working(plan.working_format),
        )?;
        result_capture_is_exact(result.spatial, mapping).then_some(())?;
        return Some(Chain {
            count: 1,
            source: source.id,
            result: result.id,
        });
    }
    let total = u32::try_from(operation.weights().len()).ok()?;
    if total == 0
        || u64::from(operation.order()[0]) * u64::from(operation.order()[1]) != u64::from(total)
    {
        return None;
    }
    let batches = total.div_ceil(32);
    let finalize_index = usize::try_from(batches).ok()?.checked_add(1)?;
    let finalize = at(finalize_index)?;
    stage_is_exact(
        first,
        work,
        ConvolutionStage::Prepare,
        &[source],
        &[source.id],
        proof,
    )?;
    let prepared = result_resource(
        first,
        resources,
        RuntimeResourceRole::FilterFloat,
        RuntimeResourceFormat::FilterRgba32Float,
    )?;
    if !grid_is_exact(prepared.spatial, mapping.source_grid_extent())
        || prepared.expected_reads != batches.checked_add(1)?
        || prepared.last_use != finalize.id
    {
        return None;
    }
    let mut previous: Option<&RuntimeResourceRequest> = None;
    let mut first_tap = 0;
    for batch in 0..batches {
        let index = usize::try_from(batch).ok()?.checked_add(1)?;
        let pass = at(index)?;
        let next = at(index.checked_add(1)?)?;
        let count = (total - first_tap).min(32);
        let stage = ConvolutionStage::Accumulate {
            first_tap,
            tap_count: count,
            first: batch == 0,
        };
        if let Some(sum) = previous {
            stage_is_exact(pass, work, stage, &[prepared, sum], &[sum.id], proof)?;
        } else {
            stage_is_exact(pass, work, stage, &[prepared], &[], proof)?;
        }
        let sum = result_resource(
            pass,
            resources,
            RuntimeResourceRole::FilterFloat,
            RuntimeResourceFormat::FilterRgba32Float,
        )?;
        if !grid_is_exact(sum.spatial, mapping.result_grid_extent())
            || sum.expected_reads != 1
            || sum.last_use != next.id
        {
            return None;
        }
        previous = Some(sum);
        first_tap = first_tap.checked_add(count)?;
    }
    if first_tap != total {
        return None;
    }
    let sum = previous?;
    // Finalization alone clamps/premultiplies; both input textures remain float32.
    stage_is_exact(
        finalize,
        work,
        ConvolutionStage::Finalize,
        &[prepared, sum],
        &[prepared.id, sum.id],
        proof,
    )?;
    let finalized = result_resource(
        finalize,
        resources,
        RuntimeResourceRole::FilterFloat,
        RuntimeResourceFormat::FilterRgba32Float,
    )?;
    let resolve = at(finalize_index.checked_add(1)?)?;
    if !grid_is_exact(finalized.spatial, mapping.result_grid_extent())
        || finalized.expected_reads != 1
        || finalized.last_use != resolve.id
    {
        return None;
    }
    stage_is_exact(
        resolve,
        work,
        ConvolutionStage::Resolve,
        &[finalized],
        &[finalized.id],
        proof,
    )?;
    let result = result_resource(
        resolve,
        resources,
        RuntimeResourceRole::FilterIntermediate,
        RuntimeResourceFormat::Working(plan.working_format),
    )?;
    result_capture_is_exact(result.spatial, mapping).then_some(())?;
    Some(Chain {
        count: finalize_index.checked_add(2)?,
        source: source.id,
        result: result.id,
    })
}

/// A supported backdrop may enlarge its completed foreground group beyond D.
/// Rebase copies its aligned raw working pixels into the original D capture;
/// normal preparation then owns the single D-coverage/color/grid conversion.
fn validate_rebase(
    pass: &RuntimePass,
    next: &RuntimePass,
    work: &GraphLoweringConvolution,
    source: &RuntimeResourceRequest,
    resources: &ProofMap<'_, RuntimeResourceId, &RuntimeResourceRequest>,
    working_format: WorkingFormat,
) -> Option<Chain> {
    let operation = work.operation();
    let mapping = work.mapping();
    let domains = operation.domains();
    let domain = intersection(domains.source(), domains.filter());
    let source_bounds = source.spatial.logical_bounds;
    if domain.width() <= 0.0
        || domain.height() <= 0.0
        || source.role != RuntimeResourceRole::CompositeResult
        || !capture_spatial_is_exact(source.spatial)
        || source_bounds.x() > domain.x()
        || source_bounds.y() > domain.y()
        || source_bounds.max().x() < domain.max().x()
        || source_bounds.max().y() < domain.max().y()
        || mapping.source_domain() != domain
        || mapping.result_domain() != domain
        || mapping.capture_origin()
            != [
                source.spatial.texel_origin.x(),
                source.spatial.texel_origin.y(),
            ]
        || mapping.kernel_origin() != mapping.capture_origin()
        || mapping.capture_scale() != source.spatial.raster_scale
        || mapping.result_capture_scale() != source.spatial.raster_scale
        || mapping.unit_length() != [1.0 / source.spatial.raster_scale; 2]
        || mapping.source_grid_origin() != [0; 2]
        || mapping.source_grid_extent() != [0; 2]
        || mapping.result_grid_origin() != [0; 2]
        || mapping.result_grid_extent() != [0; 2]
    {
        return None;
    }
    let RuntimePassKind::Convolution(next_work) = &next.kind else {
        return None;
    };
    let next_stage = if operation.is_passthrough() {
        ConvolutionStage::PassThrough
    } else {
        ConvolutionStage::Prepare
    };
    if next_work.stage() != next_stage || !next_work.operation().same_allocation(operation) {
        return None;
    }
    stage_is_exact(
        pass,
        work,
        ConvolutionStage::RebaseSource,
        &[source],
        &[source.id],
        resources.proof,
    )?;
    let result = result_resource(
        pass,
        resources,
        RuntimeResourceRole::FilterIntermediate,
        RuntimeResourceFormat::Working(working_format),
    )?;
    if result.expected_reads != 1
        || result.last_use != next.id
        || result.spatial == source.spatial
        || !result_capture_is_exact(result.spatial, mapping)
    {
        return None;
    }
    Some(Chain {
        count: 1,
        source: source.id,
        result: result.id,
    })
}

fn stage_is_exact(
    pass: &RuntimePass,
    expected: &GraphLoweringConvolution,
    stage: ConvolutionStage,
    inputs: &[&RuntimeResourceRequest],
    releases: &[RuntimeResourceId],
    proof: &ProofContext<'_>,
) -> Option<()> {
    let RuntimePassKind::Convolution(actual) = &pass.kind else {
        return None;
    };
    if actual.stage() != stage
        || !actual.operation().same_allocation(expected.operation())
        || actual.mapping() != expected.mapping()
        || pass.reads.len() != inputs.len()
        || pass.dependencies.len() != inputs.len()
        || pass.cache_keys.is_none()
        || !same_resource_set(&pass.releases, releases, proof)
    {
        return None;
    }
    for (index, (read, input)) in pass.reads.iter().zip(inputs).enumerate() {
        let RuntimeResourceProducer::Pass(producer) = input.producer else {
            return None;
        };
        let role = if index == 0 {
            RuntimeReadRole::ConvolutionSource
        } else {
            RuntimeReadRole::ConvolutionAuxiliary
        };
        if pass.dependencies.get(index) != Some(&producer)
            || !runtime_read_has_exact_facts(
                read,
                role,
                input,
                RuntimeSamplingFilter::Nearest,
                RuntimeSamplingEdge::TransparentBlack,
                proof,
            )
        {
            return None;
        }
    }
    Some(())
}

fn result_resource<'a>(
    pass: &RuntimePass,
    resources: &ProofMap<'_, RuntimeResourceId, &'a RuntimeResourceRequest>,
    role: RuntimeResourceRole,
    format: RuntimeResourceFormat,
) -> Option<&'a RuntimeResourceRequest> {
    let RuntimeResultBinding::Resource(id) = pass.result else {
        return None;
    };
    let result = resources.get(&id).copied()?;
    (base_graph_resource_has_fixed_facts(
        result,
        role,
        format,
        RuntimeResourceProducer::Pass(pass.id),
    ) && !pass.reads.iter().any(|read| read.resource == id))
    .then_some(result)
}

fn grid_is_exact(spatial: RuntimeSpatialDescriptor, extent: [u32; 2]) -> bool {
    let [width, height] = extent.map(|v| v.max(1));
    spatial.logical_bounds == Rect::new(0.0, 0.0, f64::from(width), f64::from(height))
        && spatial.device_origin == (0, 0)
        && spatial.texel_origin == Point::new(0.0, 0.0)
        && spatial.device_extent.width() == width
        && spatial.device_extent.height() == height
        && spatial.raster_scale == 1.0
}

fn intersection(a: Rect, b: Rect) -> Rect {
    let x = a.x().max(b.x());
    let y = a.y().max(b.y());
    Rect::new(
        x,
        y,
        (a.max().x().min(b.max().x()) - x).max(0.0),
        (a.max().y().min(b.max().y()) - y).max(0.0),
    )
}

fn mapping_is_exact(
    operation: &PreparedConvolution,
    mapping: ConvolutionMapping,
    source: RuntimeSpatialDescriptor,
    proof: &ProofContext<'_>,
) -> bool {
    let domains = operation.domains();
    let authored_source_domain = intersection(domains.source(), domains.filter());
    let source_is_empty =
        authored_source_domain.width() == 0.0 || authored_source_domain.height() == 0.0;
    let source_domain = if source_is_empty {
        Rect::new(0.0, 0.0, 0.0, 0.0)
    } else {
        authored_source_domain
    };
    let result_domain = intersection(domains.result(), domains.filter());
    let source_spatial_is_exact = if source_is_empty {
        empty_source_spatial_is_exact(source, mapping)
    } else {
        source.logical_bounds == source_domain && capture_spatial_is_exact(source)
    };
    if !source_spatial_is_exact
        || mapping.source_domain() != source_domain
        || mapping.result_domain() != result_domain
        || result_domain.width() <= 0.0
        || result_domain.height() <= 0.0
        || mapping.capture_origin() != [source.texel_origin.x(), source.texel_origin.y()]
        || mapping.kernel_origin() != mapping.capture_origin()
        || mapping.capture_scale() != source.raster_scale
        || mapping.unit_length()
            != operation
                .unit_length()
                .unwrap_or([1.0 / source.raster_scale; 2])
    {
        return false;
    }
    // Empty two-dimensional input has no source lattice on either axis.
    // Qualifying its other authored axis would reject unused coordinates.
    if source_is_empty
        && (mapping.source_grid_origin() != [0; 2] || mapping.source_grid_extent() != [0; 2])
    {
        return false;
    }
    // A matrix-count mismatch bypasses conversion and the kernel lattice.
    // The frame owner deliberately retains zero grid facts on that branch.
    if operation.is_passthrough() {
        return mapping.source_grid_origin() == [0; 2]
            && mapping.source_grid_extent() == [0; 2]
            && mapping.result_grid_origin() == [0; 2]
            && mapping.result_grid_extent() == [0; 2];
    }
    for axis in 0..2 {
        if !proof.work(RenderWork::CoordinateVisits, 1) {
            return false;
        }
        let unit = mapping.unit_length()[axis];
        if !unit.is_finite() || unit <= 0.0 || !(unit as f32).is_normal() {
            return false;
        }
        for (domain, origin, extent, halo) in [
            (
                source_domain,
                mapping.source_grid_origin(),
                mapping.source_grid_extent(),
                false,
            ),
            (
                result_domain,
                mapping.result_grid_origin(),
                mapping.result_grid_extent(),
                true,
            ),
        ] {
            if source_is_empty && !halo {
                continue;
            }
            if !proof.work(RenderWork::CoordinateVisits, 1) {
                return false;
            }
            let (min, length) = if axis == 0 {
                (domain.x(), domain.width())
            } else {
                (domain.y(), domain.height())
            };
            if length == 0.0 {
                if origin[axis] != 0 || extent[axis] != 0 {
                    return false;
                }
                continue;
            }
            let lo = (min - mapping.kernel_origin()[axis]) / unit;
            let hi = (min + length - mapping.kernel_origin()[axis]) / unit;
            let start = if halo { (lo - 0.5).floor() } else { lo.floor() };
            let end = if halo {
                (hi - 0.5).ceil() + 1.0
            } else {
                hi.ceil()
            };
            if !start.is_finite()
                || !end.is_finite()
                || start < -8_388_608.0
                || end > 8_388_608.0
                || end <= start
                || f64::from(origin[axis]) != start
                || f64::from(extent[axis]) != end - start
            {
                return false;
            }
        }
    }
    true
}

/// The empty input's transparent binding is one physical cell at the result
/// capture origin and density. Its reciprocal logical width is descriptive;
/// reapplying outward rounding could turn that exact one-cell allocation into
/// two cells solely through division/multiplication roundoff.
fn empty_source_spatial_is_exact(
    source: RuntimeSpatialDescriptor,
    mapping: ConvolutionMapping,
) -> bool {
    let origin = mapping.result_capture_origin();
    let scale = mapping.result_capture_scale();
    scale.is_finite()
        && scale > 0.0
        && source.device_extent == PhysicalSize::new(1, 1)
        && source.texel_origin == Point::new(origin[0], origin[1])
        && source.raster_scale == scale
        && source.logical_bounds == Rect::new(origin[0], origin[1], 1.0 / scale, 1.0 / scale)
        && [
            f64::from(source.device_origin.0) / scale,
            f64::from(source.device_origin.1) / scale,
        ] == origin
}

fn result_capture_is_exact(spatial: RuntimeSpatialDescriptor, mapping: ConvolutionMapping) -> bool {
    capture_spatial_is_exact(spatial)
        && spatial.logical_bounds == mapping.result_domain()
        && [spatial.texel_origin.x(), spatial.texel_origin.y()] == mapping.result_capture_origin()
        && spatial.raster_scale == mapping.result_capture_scale()
        && spatial.raster_scale == mapping.capture_scale()
}

fn capture_spatial_is_exact(spatial: RuntimeSpatialDescriptor) -> bool {
    use crate::filter::{
        DevicePixelConversionPolicy, FilterOutset, FilterRegionPlan, FilterSourceBounds,
    };
    let exact = (|| -> Option<()> {
        let source = FilterSourceBounds::try_new(spatial.logical_bounds).ok()?;
        let region = FilterRegionPlan::try_new(source, FilterOutset::zero(), None).ok()?;
        let device = DevicePixelConversionPolicy::outward()
            .convert_region(region.execution_region(), spatial.raster_scale)
            .ok()?;
        if spatial.device_origin != (device.x(), device.y())
            || spatial.device_extent.width() != device.width()
            || spatial.device_extent.height() != device.height()
            || spatial.texel_origin
                != Point::new(
                    f64::from(device.x()) / spatial.raster_scale,
                    f64::from(device.y()) / spatial.raster_scale,
                )
        {
            return None;
        }
        Some(())
    })();
    exact.is_some()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        Capabilities, Color, ConvolutionInput, FilterColorSpace, FilterDomains, Layer,
        RenderAccountingContext, RenderAccountingOptions, Scene, Size,
        backend::DeviceCapabilities,
        frame::{FrameContext, FramePlan},
        prepare_convolution,
    };
    use surgeist_ledger::Dimension;

    fn shared<T: Clone>(values: &[T]) -> SharedValue<Vec<T>> {
        let account = Account::for_test();
        let mut data =
            AccountedVec::new(&account, values.len(), RenderResource::RetainedCpuBytes).unwrap();
        data.extend_from_slice(values).unwrap();
        SharedValue::from_accounted(data, &account.storage()).unwrap()
    }

    fn plan(kernel: &[f64], order: f64, empty: bool) -> LoweredGraphPlan {
        let bounds = Rect::new(0.0, 0.0, 2.0, 2.0);
        plan_with_domains(
            kernel,
            order,
            empty,
            FilterDomains::try_new(bounds, bounds, bounds).unwrap(),
        )
    }

    fn plan_with_domains(
        kernel: &[f64],
        order: f64,
        empty: bool,
        domains: FilterDomains,
    ) -> LoweredGraphPlan {
        let bounds = domains.result();
        let mut accounting = RenderAccountingContext::new(RenderAccountingOptions::new());
        let operation = prepare_convolution(
            ConvolutionInput::new(kernel)
                .with_order(order, Some(1.0))
                .with_bias(0.25),
            domains,
            FilterColorSpace::Srgb,
            &mut accounting,
        )
        .unwrap();
        let mut scene = Scene::new();
        scene.layer(Layer::new().with_convolution(operation), |child| {
            if !empty {
                child.fill(bounds, Color::BLACK);
            }
        });
        lower_scene(&scene, Size::new(2.0, 2.0))
    }

    fn lower_scene(scene: &Scene, size: Size) -> LoweredGraphPlan {
        let account = Account::for_test();
        let commands = scene.normalize(Capabilities::CURRENT, &account).unwrap();
        let context =
            FrameContext::try_new(size, 1.0, Antialiasing::Area, Color::TRANSPARENT).unwrap();
        let FramePlan::GpuGraph(graph) = commands.plan_for(context, &account).unwrap() else {
            panic!("convolution requires graph execution");
        };
        LoweredGraphPlan::try_lower_validated_graph(
            &graph,
            WorkingFormat::HighPrecision,
            Format::Rgba8,
            &DeviceCapabilities::from_test_facts(true, true, 4096),
            &account,
        )
        .unwrap()
    }

    #[test]
    fn complete_convolution_chain_is_a_closed_spatial_route_including_empty_bias_and_mismatch() {
        for (kernel, order, empty, count) in [
            (&[1.0; 33][..], 33.0, false, 5),
            (&[1.0][..], 1.0, true, 4),
            (&[1.0][..], 2.0, false, 1),
        ] {
            let plan = plan(kernel, order, empty);
            let closed = ClosedExecutableGraph::try_from_lowered(plan, &Account::for_test())
                .unwrap()
                .expect("complete chain closes");
            assert!(closed.has_spatial_filters());
            let step = closed
                .facts
                .filter_steps
                .iter()
                .find_map(|step| match step {
                    ExecutableFilterStepFacts::Convolution { passes, .. } => Some(passes),
                    _ => None,
                })
                .unwrap();
            assert_eq!(step.len(), count);
            assert!(
                closed
                    .facts
                    .proves_exact_facts_for(&closed.lowered, &Account::for_test())
                    .unwrap()
            );
        }
    }

    #[test]
    fn empty_domain_closure_requires_canonical_zero_grid_and_one_transparent_capture_cell() {
        let result = Rect::new(0.0, 0.0, 2.0, 2.0);
        let filter = Rect::new(0.0, 0.0, 1.0e12, 1.0e12);
        for source in [
            Rect::new(0.0, 0.0, 0.0, 1.0e12),
            Rect::new(0.0, 0.0, 1.0e12, 0.0),
        ] {
            let original = plan_with_domains(
                &[1.0],
                1.0,
                false,
                FilterDomains::try_new(source, filter, result).unwrap(),
            );
            let account = Account::for_test();
            let closed = ClosedExecutableGraph::try_from_lowered(original.clone(), &account)
                .unwrap()
                .expect("empty-domain bias keeps a closed executable route");
            let prepare = original
                .passes
                .iter()
                .find(|pass| {
                    matches!(&pass.kind,
                RuntimePassKind::Convolution(work) if work.stage() == ConvolutionStage::Prepare)
                })
                .unwrap();
            let RuntimePassKind::Convolution(work) = &prepare.kind else {
                unreachable!();
            };
            assert_eq!(
                work.mapping().source_domain(),
                Rect::new(0.0, 0.0, 0.0, 0.0)
            );
            assert_eq!(work.mapping().source_grid_origin(), [0; 2]);
            assert_eq!(work.mapping().source_grid_extent(), [0; 2]);
            let source_id = prepare.reads[0].resource;
            let source = original
                .resources
                .iter()
                .find(|resource| resource.id == source_id)
                .unwrap();
            assert_eq!(source.spatial.device_extent, PhysicalSize::new(1, 1));
            assert_eq!(source.role, RuntimeResourceRole::IsolationWorkingImage);
            assert!(
                closed.facts.captures.is_empty(),
                "children outside empty input cannot create captures"
            );
            for alteration in 0..2 {
                let mut changed = original.clone();
                if alteration == 0 {
                    let mut resources = original.resources.to_vec();
                    resources
                        .iter_mut()
                        .find(|resource| resource.id == source_id)
                        .unwrap()
                        .spatial
                        .device_extent = PhysicalSize::new(2, 1);
                    changed.resources = shared(&resources);
                } else {
                    let mut passes = original.passes.to_vec();
                    let clear = passes
                        .iter_mut()
                        .find(|pass| pass.result == RuntimeResultBinding::Resource(source_id))
                        .unwrap();
                    let RuntimePassKind::ClearRoot { color, .. } = &mut clear.kind else {
                        unreachable!();
                    };
                    *color = Color::BLACK;
                    changed.passes = shared(&passes);
                }
                assert!(
                    ClosedExecutableGraph::try_from_lowered(changed.clone(), &account)
                        .unwrap()
                        .is_err()
                );
                assert!(
                    !closed
                        .facts
                        .proves_exact_facts_for(&changed, &account)
                        .unwrap()
                );
            }
        }
    }

    #[test]
    fn backdrop_union_rebase_closes_only_as_an_exact_crop_before_the_same_convolution() {
        use crate::{BackdropCaptureBounds, BackdropFilterInput, FilterBlur, FilterList, FilterOp};
        let domain = Rect::new(1.0, 0.0, 2.0, 1.0);
        let root = Rect::new(0.0, 0.0, 6.0, 1.0);
        let operation = prepare_convolution(
            ConvolutionInput::new(&[1.0])
                .with_order(1.0, None)
                .with_unit_length(2.0, None),
            FilterDomains::try_new(domain, root, domain).unwrap(),
            FilterColorSpace::Srgb,
            &mut RenderAccountingContext::default(),
        )
        .unwrap();
        let backdrop = BackdropFilterInput::try_new(
            FilterList::try_ops(vec![FilterOp::blur(FilterBlur::try_new(0.0).unwrap())]).unwrap(),
            BackdropCaptureBounds::try_new(root).unwrap(),
            None,
        )
        .unwrap();
        let mut scene = Scene::new();
        scene.fill(root, Color::BLACK);
        scene.layer(
            Layer::new()
                .with_convolution(operation)
                .try_backdrop_filter(backdrop)
                .unwrap(),
            |_| {},
        );
        let original = lower_scene(&scene, Size::new(6.0, 1.0));
        let account = Account::for_test();
        let closed = ClosedExecutableGraph::try_from_lowered(original.clone(), &account)
            .unwrap()
            .expect("supported backdrop group and exact rebase must close");
        assert_eq!(closed.facts.backdrops.len(), 1);
        let index = original
            .passes
            .iter()
            .position(|pass| {
                matches!(&pass.kind,
            RuntimePassKind::Convolution(work) if work.stage() == ConvolutionStage::RebaseSource)
            })
            .unwrap();
        let rebase = &original.passes[index];
        let RuntimeResultBinding::Resource(id) = rebase.result else {
            unreachable!();
        };
        let result = original
            .resources
            .iter()
            .find(|resource| resource.id == id)
            .unwrap();
        assert_eq!(result.spatial.logical_bounds, domain);
        assert_eq!(result.spatial.device_origin, (1, 0));
        assert_eq!(result.spatial.device_extent, PhysicalSize::new(2, 1));
        for alteration in 0..3 {
            let mut changed = original.clone();
            match alteration {
                0 => {
                    let mut passes = original.passes.to_vec();
                    passes.remove(index + 1);
                    changed.passes = shared(&passes);
                }
                1 => {
                    let mut passes = original.passes.to_vec();
                    passes[index].releases = shared(&[]);
                    changed.passes = shared(&passes);
                }
                _ => {
                    let mut resources = original.resources.to_vec();
                    resources
                        .iter_mut()
                        .find(|resource| resource.id == id)
                        .unwrap()
                        .spatial
                        .logical_bounds = root;
                    changed.resources = shared(&resources);
                }
            }
            assert!(
                ClosedExecutableGraph::try_from_lowered(changed.clone(), &account)
                    .unwrap()
                    .is_err()
            );
            assert!(
                !closed
                    .facts
                    .proves_exact_facts_for(&changed, &account)
                    .unwrap()
            );
        }
    }

    #[test]
    fn convolution_closure_rejects_missing_batches_wrong_inputs_clamps_and_capture_geometry() {
        let original = plan(&[1.0; 33], 33.0, false);
        let account = Account::for_test();
        let closed = ClosedExecutableGraph::try_from_lowered(original.clone(), &account)
            .unwrap()
            .ok()
            .unwrap();
        for alteration in 0..9 {
            let mut changed = original.clone();
            let mut passes = changed.passes.to_vec();
            let prepare = passes.iter().position(|p| matches!(&p.kind, RuntimePassKind::Convolution(work) if work.stage() == ConvolutionStage::Prepare)).unwrap();
            match alteration {
                0 => {
                    passes.remove(prepare + 2);
                }
                1 => {
                    passes[prepare + 2].kind = passes[prepare + 1].kind.clone();
                }
                2 => {
                    let mut reads = passes[prepare + 2].reads.to_vec();
                    reads[1].resource = reads[0].resource;
                    passes[prepare + 2].reads = shared(&reads);
                }
                3 => {
                    passes[prepare + 1].releases = shared(&[passes[prepare + 1].reads[0].resource]);
                }
                8 => {
                    let finalize = prepare + 3;
                    let mut reads = passes[finalize].reads.to_vec();
                    // Keep roles fixed while swapping the resource identities:
                    // the primary source must be the prepared kernel image.
                    let source = reads[0].resource;
                    reads[0].resource = reads[1].resource;
                    reads[1].resource = source;
                    passes[finalize].reads = shared(&reads);
                }
                4..=7 => {
                    let RuntimeResultBinding::Resource(id) = passes[prepare + 1].result else {
                        panic!();
                    };
                    let mut resources = changed.resources.to_vec();
                    let resource = resources.iter_mut().find(|r| r.id == id).unwrap();
                    match alteration {
                        4 => {
                            resource.format =
                                RuntimeResourceFormat::Working(WorkingFormat::HighPrecision)
                        }
                        5 => resource.spatial.raster_scale = 2.0,
                        6 => resource.expected_reads += 1,
                        _ => resource.spatial.device_extent = PhysicalSize::new(1, 1),
                    }
                    changed.resources = shared(&resources);
                }
                _ => unreachable!(),
            }
            changed.passes = shared(&passes);
            assert!(
                ClosedExecutableGraph::try_from_lowered(changed.clone(), &account)
                    .unwrap()
                    .is_err(),
                "alteration {alteration}"
            );
            assert!(
                !closed
                    .facts
                    .proves_exact_facts_for(&changed, &account)
                    .unwrap(),
                "repeated proof alteration {alteration}"
            );
        }
    }

    fn classification_chain(convolution: Option<bool>) -> LoweredGraphPlan {
        use crate::{FilterAmount, FilterList, FilterOp};
        fn colors(scene: &mut Scene, count: usize, domain: Rect) {
            if count == 0 {
                scene.fill(domain, Color::BLACK);
            } else {
                let filter = FilterList::try_ops(vec![FilterOp::brightness(
                    FilterAmount::try_new(0.75).unwrap(),
                )])
                .unwrap();
                scene.layer(Layer::new().with_filter_list(filter), |child| {
                    colors(child, count - 1, domain);
                });
            }
        }
        let domain = Rect::new(0.0, 0.0, 2.0, 2.0);
        let mut scene = Scene::new();
        if let Some(mismatch) = convolution {
            let kernel: &[f64] = if mismatch { &[] } else { &[1.0] };
            let operation = prepare_convolution(
                ConvolutionInput::new(kernel).with_order(1.0, None),
                FilterDomains::try_new(domain, domain, domain).unwrap(),
                FilterColorSpace::Srgb,
                &mut RenderAccountingContext::default(),
            )
            .unwrap();
            scene.layer(Layer::new().with_convolution(operation), |child| {
                colors(child, 4, domain)
            });
        } else {
            colors(&mut scene, 4, domain);
        }
        lower_scene(&scene, Size::new(2.0, 2.0))
    }

    fn measured_proof<T>(
        limit: Option<u64>,
        run: impl FnOnce(&Account) -> Result<T>,
    ) -> (Result<T>, crate::RenderStandaloneReport) {
        use crate::{RenderAttemptKind, RenderLimits, RenderPhase};
        let mut limits = RenderLimits::new();
        if let Some(limit) = limit {
            limits = limits
                .with_work_limit(RenderWork::GraphNodeVisits, limit)
                .unwrap();
        }
        let mut context =
            RenderAccountingContext::new(RenderAccountingOptions::new().with_limits(limits));
        let result = context.run(RenderAttemptKind::Frame, RenderPhase::Lower, run);
        (result, context.take_last_attempt().unwrap())
    }

    /// Characterization of the retained proof/accounting contract. Replacing a
    /// hidden scan with validated constant-size metadata changes neither the
    /// public route nor reported work, so behavioral pre-correction RED does not
    /// apply to that correction; these cases establish the existing boundaries.
    #[test]
    fn pure_color_and_late_convolution_routes_preserve_exact_accounted_proof_limits() {
        use crate::{RenderQuantity, RenderWork};
        for convolution in [None, Some(false), Some(true)] {
            let plan = classification_chain(convolution);
            let spatial = convolution.is_some();
            let closed =
                ClosedExecutableGraph::try_from_lowered(plan.clone(), &Account::for_test())
                    .unwrap()
                    .expect("the actual authored filter chain must close");
            assert_eq!(closed.facts.filter_steps.len(), 4 + usize::from(spatial));
            assert!(
                closed.facts.filter_steps[..4]
                    .iter()
                    .all(|step| matches!(step, ExecutableFilterStepFacts::Color(_)))
            );
            if spatial {
                assert!(matches!(
                    closed.facts.filter_steps.last(),
                    Some(ExecutableFilterStepFacts::Convolution { .. })
                ));
            }
            let route =
                PrePreparationGraphClassification::classify(plan, &Account::for_test()).unwrap();
            assert!(matches!(
                (spatial, route),
                (
                    false,
                    PrePreparationGraphClassification::ExactColorFilter(_)
                ) | (
                    true,
                    PrePreparationGraphClassification::ExactSpatialFilter(_)
                )
            ));

            let (result, measured) = measured_proof(None, |account| {
                closed
                    .facts
                    .proves_exact_facts_for(&closed.lowered, account)
            });
            assert!(result.unwrap());
            let visits = measured
                .attempt()
                .unwrap()
                .usage()
                .work(RenderWork::GraphNodeVisits);
            assert!(visits >= u64::try_from(closed.facts.filter_steps.len()).unwrap());
            for limit in [visits, visits - 1] {
                let (result, report) = measured_proof(Some(limit), |account| {
                    closed
                        .facts
                        .proves_exact_facts_for(&closed.lowered, account)
                });
                let attempt = report.attempt().unwrap();
                assert_eq!(attempt.usage().work(RenderWork::GraphNodeVisits), limit);
                assert_eq!(attempt.usage().live(RenderResource::ScratchCpuBytes), 0);
                assert_eq!(report.cleanup_fault(), None);
                if limit == visits {
                    assert!(result.unwrap());
                    assert!(attempt.refusal().is_none());
                } else {
                    assert!(result.is_err());
                    let refusal = attempt.refusal().unwrap();
                    assert_eq!(
                        refusal.quantity(),
                        RenderQuantity::Work(RenderWork::GraphNodeVisits)
                    );
                    assert_eq!((refusal.current(), refusal.amount()), (limit, 1));
                }
            }
        }
    }

    #[test]
    fn validated_filter_route_classifiers_add_no_accounted_work_or_custody() {
        for convolution in [None, Some(false), Some(true)] {
            let closed = ClosedExecutableGraph::try_from_lowered(
                classification_chain(convolution),
                &Account::for_test(),
            )
            .unwrap()
            .expect("the actual authored chain must close before classification");
            let spatial = convolution.is_some();
            let (result, report) = measured_proof(Some(0), |_account| {
                assert_eq!(closed.has_spatial_filters(), spatial);
                match SpatialFilterPreparableGraph::try_from_closed(closed) {
                    Ok(graph) => {
                        assert!(spatial);
                        assert!(graph.closed.has_spatial_filters());
                    }
                    Err(closed) => {
                        assert!(!spatial);
                        assert!(ColorFilterPreparableGraph::try_from_closed(closed).is_ok());
                    }
                }
                Ok(())
            });
            result.unwrap();
            let attempt = report.attempt().unwrap();
            for &work in RenderWork::ALL {
                assert_eq!(attempt.usage().work(work), 0, "post-proof {work:?}");
            }
            for resource in [
                RenderResource::RetainedCpuBytes,
                RenderResource::ScratchCpuBytes,
            ] {
                assert_eq!(attempt.usage().live(resource), 0, "post-proof {resource:?}");
            }
            assert_eq!(report.cleanup_fault(), None);
        }
    }

    #[test]
    fn retained_convolution_presence_is_checked_at_the_accounted_step_boundary() {
        use crate::RenderQuantity;
        for convolution in [None, Some(false), Some(true)] {
            let closed = ClosedExecutableGraph::try_from_lowered(
                classification_chain(convolution),
                &Account::for_test(),
            )
            .unwrap()
            .expect("the authored chain must close");
            assert_eq!(closed.facts.has_convolution, convolution.is_some());
            let mut forged = (*closed.facts).clone();
            forged.has_convolution = !forged.has_convolution;
            assert!(
                !forged
                    .proves_exact_facts_for(&closed.lowered, &Account::for_test())
                    .unwrap()
            );
            let prefix_visits = if convolution.is_some() { 5 } else { 4 };
            for limit in [prefix_visits, prefix_visits - 1] {
                let (result, report) = measured_proof(Some(limit), |account| {
                    let proof = ProofContext::new(account);
                    let exact = super::steps_are_exact(
                        &closed.lowered,
                        &closed.facts.filter_steps,
                        !closed.facts.has_convolution,
                        &proof,
                    );
                    proof.finish()?;
                    Ok(exact)
                });
                let attempt = report.attempt().unwrap();
                assert_eq!(attempt.usage().work(RenderWork::GraphNodeVisits), limit);
                assert_eq!(attempt.usage().live(RenderResource::ScratchCpuBytes), 0);
                assert_eq!(report.cleanup_fault(), None);
                if limit == prefix_visits {
                    assert!(
                        !result.unwrap(),
                        "a flipped cached presence fact must fail proof"
                    );
                    assert!(attempt.refusal().is_none());
                } else {
                    assert!(result.is_err());
                    let refusal = attempt.refusal().unwrap();
                    assert_eq!(
                        refusal.quantity(),
                        RenderQuantity::Work(RenderWork::GraphNodeVisits)
                    );
                    assert_eq!((refusal.current(), refusal.amount()), (limit, 1));
                }
            }
        }
    }
}
