//! Closure proof for one offset sample and its optional aligned raw rebase.
use super::*;
use crate::{
    ImageRendering, Rect,
    frame::{GraphLoweringOffset, OffsetSampling, OffsetStage},
};

struct Chain {
    rebase: Option<RuntimePassId>,
    sample: RuntimePassId,
    source: RuntimeResourceId,
    result: RuntimeResourceId,
}
impl Chain {
    fn count(&self) -> usize {
        1 + usize::from(self.rebase.is_some())
    }
}

impl ClosedGraphTraversal<'_, '_> {
    pub(super) fn visit_offset(&mut self) -> Option<()> {
        let context = *self.contexts.last()?;
        self.initialized_context_source(context)?;
        let chain = validate_chain(self.plan, self.cursor, context, &self.resources)?;
        for index in self.cursor..self.cursor.checked_add(chain.count())? {
            let pass = self.pass_at(index)?;
            let RuntimeResultBinding::Resource(result) = pass.result else {
                return None;
            };
            self.expected_resources.insert(result);
        }
        self.proof.capture_storage_write(self.filter_steps.push(
            ExecutableFilterStepFacts::Offset {
                rebase: chain.rebase,
                sample: chain.sample,
                source: chain.source,
                result: chain.result,
            },
        ))?;
        self.has_offset = true;
        let current = self.contexts.last_mut()?;
        current.current = chain.result;
        current.producer = chain.sample;
        current.contains_captured_source = true;
        self.advance(chain.count())
    }
}

pub(super) fn steps_are_exact(
    plan: &LoweredGraphPlan,
    steps: &[ExecutableFilterStepFacts],
    expected_offset: bool,
    proof: &ProofContext<'_>,
) -> bool {
    let exact = (|| -> Option<()> {
        // The already-accounted pass/step-order proof validates this retained
        // classification before calling the offset-specific relationship proof.
        if !expected_offset {
            return Some(());
        }
        let maps = proof.capture(ClosedGraphMaps::try_new(plan, proof))?;
        for step in proof_nodes(steps, proof) {
            let ExecutableFilterStepFacts::Offset {
                rebase,
                sample,
                source,
                result,
            } = step
            else {
                continue;
            };
            let first = rebase.unwrap_or(*sample);
            let cursor = *maps.pass_positions.get(&first)?;
            let source_resource = maps.resource_by_id.get(source)?;
            let RuntimeResourceProducer::Pass(producer) = source_resource.producer else {
                return None;
            };
            let context = ExecutableCompositionContext {
                current: *source,
                producer,
                contains_captured_source: true,
            };
            let chain = validate_chain(plan, cursor, context, &maps.resource_by_id)?;
            if chain.rebase != *rebase
                || chain.sample != *sample
                || chain.source != *source
                || chain.result != *result
            {
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
    let RuntimePassKind::Offset(work) = &first.kind else {
        return None;
    };
    if work.purpose() != crate::frame::OffsetPurpose::Standalone {
        return None;
    }
    let source = resources.get(&context.current).copied()?;
    if source.producer != RuntimeResourceProducer::Pass(context.producer)
        || source.format != RuntimeResourceFormat::Working(plan.working_format)
        || source.import.is_some()
        || !matches!(
            source.role,
            RuntimeResourceRole::FilterIntermediate
                | RuntimeResourceRole::CompositeResult
                | RuntimeResourceRole::IsolationWorkingImage
        )
    {
        return None;
    }
    let (rebase, sample, input) = if work.stage() == OffsetStage::RebaseSource {
        let sample = at(1)?;
        let RuntimePassKind::Offset(sample_work) = &sample.kind else {
            return None;
        };
        if sample_work.stage() != OffsetStage::Sample || sample_work.operation() != work.operation()
        {
            return None;
        }
        let cropped = validate_stage(first, work, source, resources, plan.working_format)?;
        if source.role != RuntimeResourceRole::CompositeResult
            || source.spatial == cropped.spatial
            || !mapping_is_exact(work, source.spatial, cropped.spatial, proof)
        {
            return None;
        }
        (Some(first.id), sample, cropped)
    } else {
        (None, first, source)
    };
    let RuntimePassKind::Offset(sample_work) = &sample.kind else {
        return None;
    };
    if sample_work.stage() != OffsetStage::Sample {
        return None;
    }
    let result = validate_stage(sample, sample_work, input, resources, plan.working_format)?;
    if !mapping_is_exact(sample_work, input.spatial, result.spatial, proof) {
        return None;
    }
    Some(Chain {
        rebase,
        sample: sample.id,
        source: source.id,
        result: result.id,
    })
}

pub(super) fn validate_stage<'a>(
    pass: &RuntimePass,
    work: &GraphLoweringOffset,
    source: &RuntimeResourceRequest,
    resources: &ProofMap<'_, RuntimeResourceId, &'a RuntimeResourceRequest>,
    working_format: WorkingFormat,
) -> Option<&'a RuntimeResourceRequest> {
    let RuntimeResourceProducer::Pass(producer) = source.producer else {
        return None;
    };
    let proof = resources.proof;
    if pass.reads.len() != 1
        || pass.dependencies.len() != 1
        || !proof_relationships_equal(
            pass.dependencies.iter().copied(),
            [producer],
            proof,
            |a, b| a == b,
        )
        || pass.cache_keys.is_none()
        || source.expected_reads != 1
        || source.last_use != pass.id
        || !same_resource_set(&pass.releases, &[source.id], proof)
        || !runtime_read_has_exact_facts(
            proof_read(pass, 0, proof)?,
            RuntimeReadRole::OffsetSource,
            source,
            RuntimeSamplingFilter::Nearest,
            RuntimeSamplingEdge::TransparentBlack,
            proof,
        )
    {
        return None;
    }
    let RuntimeResultBinding::Resource(id) = pass.result else {
        return None;
    };
    let result = resources.get(&id).copied()?;
    if id == source.id
        || !base_graph_resource_has_fixed_facts(
            result,
            RuntimeResourceRole::FilterIntermediate,
            RuntimeResourceFormat::Working(working_format),
            RuntimeResourceProducer::Pass(pass.id),
        )
    {
        return None;
    }
    // The immutable frame payload is the sole numeric qualification owner.
    // Every descriptor is nevertheless rebound to the exact admitted lattice.
    let map = work.mapping();
    if !descriptor_matches(
        source.spatial,
        map.source_origin(),
        map.source_scale(),
        map.source_extent(),
        proof,
    ) || !descriptor_matches(
        result.spatial,
        map.result_origin(),
        map.result_scale(),
        map.result_extent(),
        proof,
    ) {
        return None;
    }
    Some(result)
}

pub(super) fn mapping_is_exact(
    work: &GraphLoweringOffset,
    source: RuntimeSpatialDescriptor,
    result: RuntimeSpatialDescriptor,
    proof: &ProofContext<'_>,
) -> bool {
    let op = work.operation();
    let map = work.mapping();
    let domains = op.domains();
    let input = intersection(domains.source(), domains.filter());
    let output = if work.stage() == OffsetStage::RebaseSource {
        input
    } else {
        intersection(domains.result(), domains.filter())
    };
    if input.width() <= 0.0
        || input.height() <= 0.0
        || output.width() <= 0.0
        || output.height() <= 0.0
        || map.source_domain() != input
        || map.result_domain() != output
        || result.logical_bounds != output
        || !capture_is_exact(source, proof)
        || !capture_is_exact(result, proof)
        || (work.stage() == OffsetStage::Sample && source.logical_bounds != input)
        || (work.stage() == OffsetStage::RebaseSource && !contains(source.logical_bounds, input))
    {
        return false;
    }
    let delta = if work.stage() == OffsetStage::RebaseSource {
        [0.0; 2]
    } else {
        [op.dx(), op.dy()]
    };
    let mut aligned = source.raster_scale == result.raster_scale;
    for (axis, &displacement) in delta.iter().enumerate() {
        if !proof.work(RenderWork::CoordinateVisits, 1) {
            return false;
        }
        let source_origin = map.source_origin()[axis];
        let result_origin = map.result_origin()[axis];
        let ratio = source.raster_scale / result.raster_scale;
        let lattice_delta = if source.raster_scale == result.raster_scale {
            let source_device = if axis == 0 {
                source.device_origin.0
            } else {
                source.device_origin.1
            };
            let result_device = if axis == 0 {
                result.device_origin.0
            } else {
                result.device_origin.1
            };
            (i64::from(result_device) - i64::from(source_device)) as f64
        } else {
            (result_origin - source_origin) * source.raster_scale
        };
        let translation = if source.raster_scale == result.raster_scale {
            lattice_delta - displacement * source.raster_scale
        } else {
            ((result_origin - source_origin) - displacement) * source.raster_scale
                + (0.5 * ratio - 0.5)
        };
        if map.ratio()[axis] != ratio || map.translation()[axis] != translation {
            return false;
        }
        let origin_delta = lattice_delta;
        let offset = displacement * source.raster_scale;
        aligned &=
            origin_delta.fract() == 0.0 && offset.fract() == 0.0 && translation.fract() == 0.0;
    }
    if map.source_clip()
        != clip(
            input,
            map.source_origin(),
            source.raster_scale,
            map.source_extent(),
            proof,
        )
        || map.result_clip()
            != clip(
                output,
                map.result_origin(),
                result.raster_scale,
                map.result_extent(),
                proof,
            )
    {
        return false;
    }
    let expected_sampling = if aligned {
        OffsetSampling::Copy
    } else {
        match op.rendering() {
            ImageRendering::Auto | ImageRendering::Smooth | ImageRendering::Pixelated => {
                OffsetSampling::Bilinear
            }
            ImageRendering::CrispEdges => OffsetSampling::Nearest,
            ImageRendering::HighQuality => OffsetSampling::Cubic,
        }
    };
    map.sampling() == expected_sampling && (work.stage() != OffsetStage::RebaseSource || aligned)
}

fn descriptor_matches(
    spatial: RuntimeSpatialDescriptor,
    origin: [f64; 2],
    scale: f64,
    extent: [u32; 2],
    proof: &ProofContext<'_>,
) -> bool {
    proof.work(RenderWork::CoordinateVisits, 1)
        && [spatial.texel_origin.x(), spatial.texel_origin.y()] == origin
        && spatial.raster_scale == scale
        && [
            spatial.device_extent.width(),
            spatial.device_extent.height(),
        ] == extent
}
fn capture_is_exact(spatial: RuntimeSpatialDescriptor, proof: &ProofContext<'_>) -> bool {
    use crate::filter::{
        DevicePixelConversionPolicy, FilterOutset, FilterRegionPlan, FilterSourceBounds,
    };
    if !proof.work(RenderWork::CoordinateVisits, 1) {
        return false;
    }
    let exact = (|| -> Option<()> {
        let bounds = FilterSourceBounds::try_new(spatial.logical_bounds).ok()?;
        let region = FilterRegionPlan::try_new(bounds, FilterOutset::zero(), None).ok()?;
        let device = DevicePixelConversionPolicy::outward()
            .convert_region(region.execution_region(), spatial.raster_scale)
            .ok()?;
        (spatial.device_origin == (device.x(), device.y())
            && spatial.device_extent == PhysicalSize::new(device.width(), device.height())
            && spatial.texel_origin
                == Point::new(
                    f64::from(device.x()) / spatial.raster_scale,
                    f64::from(device.y()) / spatial.raster_scale,
                ))
        .then_some(())
    })();
    exact.is_some()
}
fn clip(
    domain: Rect,
    origin: [f64; 2],
    scale: f64,
    extent: [u32; 2],
    proof: &ProofContext<'_>,
) -> [f64; 4] {
    let bounds = [domain.x(), domain.y(), domain.max().x(), domain.max().y()];
    std::array::from_fn(|i| {
        if !proof.work(RenderWork::CoordinateVisits, 1) {
            return f64::NAN;
        }
        ((bounds[i] - origin[i % 2]) * scale).clamp(0.0, f64::from(extent[i % 2]))
    })
}
fn contains(outer: Rect, inner: Rect) -> bool {
    outer.x() <= inner.x()
        && outer.y() <= inner.y()
        && outer.max().x() >= inner.max().x()
        && outer.max().y() >= inner.max().y()
}
pub(super) fn intersection(a: Rect, b: Rect) -> Rect {
    let x = a.x().max(b.x());
    let y = a.y().max(b.y());
    Rect::new(
        x,
        y,
        (a.max().x().min(b.max().x()) - x).max(0.0),
        (a.max().y().min(b.max().y()) - y).max(0.0),
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        Capabilities, FilterDomains, Layer, OffsetInput, RenderAccountingContext, Scene, Size,
        backend::DeviceCapabilities,
        frame::{FrameContext, FramePlan},
        prepare_offset,
    };

    fn lower(dx: f64, rendering: ImageRendering, backdrop: bool) -> LoweredGraphPlan {
        let d = Rect::new(1.25, 0.0, 1.5, 1.0);
        let f = Rect::new(0.0, 0.0, 6.0, 1.0);
        let operation = prepare_offset(
            OffsetInput::new().with_dx(dx).with_rendering(rendering),
            FilterDomains::try_new(d, f, f).unwrap(),
            &mut RenderAccountingContext::default(),
        )
        .unwrap();
        let layer = if backdrop {
            let filter = crate::FilterList::try_ops(vec![crate::FilterOp::blur(
                crate::FilterBlur::try_new(0.0).unwrap(),
            )])
            .unwrap();
            Layer::new()
                .try_backdrop_filter(
                    crate::BackdropFilterInput::try_new(
                        filter,
                        crate::BackdropCaptureBounds::try_new(f).unwrap(),
                        None,
                    )
                    .unwrap(),
                )
                .unwrap()
                .with_offset(operation)
        } else {
            Layer::new().with_offset(operation)
        };
        let mut scene = Scene::new();
        scene.fill(f, Color::BLACK);
        scene.layer(layer, |child| {
            child.fill(d, Color::BLACK);
        });
        let account = Account::for_test();
        let commands = scene.normalize(Capabilities::CURRENT, &account).unwrap();
        let context = FrameContext::try_new(
            Size::new(6.0, 1.0),
            1.0,
            Antialiasing::Area,
            Color::TRANSPARENT,
        )
        .unwrap();
        let FramePlan::GpuGraph(graph) = commands.plan_for(context, &account).unwrap() else {
            panic!("offset requires the graph executor");
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
    fn fractional_and_integral_offset_chains_close_for_every_hint_and_backdrop() {
        for rendering in [
            ImageRendering::Auto,
            ImageRendering::Smooth,
            ImageRendering::CrispEdges,
            ImageRendering::HighQuality,
            ImageRendering::Pixelated,
        ] {
            for dx in [-1.0, 0.0, 0.5, 1.0] {
                for backdrop in [false, true] {
                    let closed = ClosedExecutableGraph::try_from_lowered(
                        lower(dx, rendering, backdrop),
                        &Account::for_test(),
                    )
                    .unwrap()
                    .expect("the complete offset chain must close");
                    assert!(closed.has_spatial_filters());
                    assert!(closed.facts.has_offset);
                    assert!(
                        closed
                            .facts
                            .proves_exact_facts_for(&closed.lowered, &Account::for_test())
                            .unwrap()
                    );
                    let offset = closed
                        .facts
                        .filter_steps
                        .iter()
                        .find_map(|step| match step {
                            ExecutableFilterStepFacts::Offset { rebase, .. } => Some(*rebase),
                            _ => None,
                        })
                        .unwrap();
                    assert_eq!(offset.is_some(), backdrop);
                    let route = PrePreparationGraphClassification::classify(
                        closed.lowered,
                        &Account::for_test(),
                    )
                    .unwrap();
                    assert!(matches!(
                        (backdrop, route),
                        (
                            false,
                            PrePreparationGraphClassification::ExactSpatialFilter(_)
                        ) | (true, PrePreparationGraphClassification::ExactBackdrop(_))
                    ));
                }
            }
        }
    }

    fn edit<T: Clone>(values: &mut SharedValue<Vec<T>>) -> impl DerefMut<Target = Vec<T>> + '_ {
        values
            .edit_for_test(4, &Account::for_test().storage())
            .unwrap()
    }

    #[test]
    fn offset_closure_rejects_aliases_wrong_roles_lattices_and_lifetimes() {
        for backdrop in [false, true] {
            let original = lower(0.5, ImageRendering::HighQuality, backdrop);
            let sample = original.passes.iter().position(|pass| matches!(&pass.kind, RuntimePassKind::Offset(work) if work.stage() == OffsetStage::Sample)).unwrap();
            let input = original.passes[sample].reads[0].resource;
            let RuntimeResultBinding::Resource(output) = original.passes[sample].result else {
                unreachable!();
            };
            for fault in 0..10 {
                let mut plan = original.clone();
                match fault {
                    0 => {
                        let mut passes = edit(&mut plan.passes);
                        edit(&mut passes[sample].reads)[0].role = RuntimeReadRole::FilterSource;
                    }
                    1 => {
                        let mut passes = edit(&mut plan.passes);
                        edit(&mut passes[sample].reads)[0].resource = output;
                    }
                    2 => {
                        let mut passes = edit(&mut plan.passes);
                        edit(&mut passes[sample].dependencies).clear();
                    }
                    3 => {
                        let mut passes = edit(&mut plan.passes);
                        edit(&mut passes[sample].releases).clear();
                    }
                    _ => {
                        let mut resources = edit(&mut plan.resources);
                        let id = if fault < 7 { input } else { output };
                        let resource = resources
                            .iter_mut()
                            .find(|resource| resource.id == id)
                            .unwrap();
                        match fault {
                            4 => resource.expected_reads += 1,
                            5 => resource.last_use = plan.final_present,
                            6 => resource.spatial.texel_origin = Point::new(0.0, 0.0),
                            7 => resource.spatial.raster_scale = 2.0,
                            8 => resource.spatial.device_extent = PhysicalSize::new(1, 1),
                            _ => resource.role = RuntimeResourceRole::CompositeResult,
                        }
                    }
                }
                assert!(
                    ClosedExecutableGraph::try_from_lowered(plan, &Account::for_test())
                        .unwrap()
                        .is_err(),
                    "backdrop {backdrop}, fault {fault}"
                );
            }
        }
    }

    #[test]
    fn offset_presence_and_repeated_proof_obey_exact_accounted_limits() {
        use crate::pass::test_support::measure_proof_work;
        let closed = ClosedExecutableGraph::try_from_lowered(
            lower(0.5, ImageRendering::Auto, true),
            &Account::for_test(),
        )
        .unwrap()
        .unwrap();
        let mut forged = (*closed.facts).clone();
        forged.has_offset = false;
        assert!(
            !forged
                .proves_exact_facts_for(&closed.lowered, &Account::for_test())
                .unwrap()
        );
        let (result, report) =
            measure_proof_work(RenderWork::GraphNodeVisits, u64::MAX, |account| {
                closed
                    .facts
                    .proves_exact_facts_for(&closed.lowered, account)
            });
        assert!(result.unwrap());
        let visits = report
            .attempt()
            .unwrap()
            .usage()
            .work(RenderWork::GraphNodeVisits);
        for limit in [visits, visits - 1] {
            let (result, report) =
                measure_proof_work(RenderWork::GraphNodeVisits, limit, |account| {
                    closed
                        .facts
                        .proves_exact_facts_for(&closed.lowered, account)
                });
            if limit == visits {
                assert!(result.unwrap());
            } else {
                assert!(result.is_err());
            }
            assert_eq!(
                report
                    .attempt()
                    .unwrap()
                    .usage()
                    .work(RenderWork::GraphNodeVisits),
                limit
            );
            assert_eq!(
                report
                    .attempt()
                    .unwrap()
                    .usage()
                    .live(RenderResource::ScratchCpuBytes),
                0
            );
            assert_eq!(report.cleanup_fault(), None);
        }
        let route_closed = ClosedExecutableGraph::try_from_lowered(
            lower(0.5, ImageRendering::Auto, false),
            &Account::for_test(),
        )
        .unwrap()
        .unwrap();
        let (result, report) = measure_proof_work(RenderWork::GraphNodeVisits, 0, |_account| {
            assert!(route_closed.has_spatial_filters());
            assert!(SpatialFilterPreparableGraph::try_from_closed(route_closed).is_ok());
            Ok(())
        });
        result.unwrap();
        assert_eq!(
            report
                .attempt()
                .unwrap()
                .usage()
                .work(RenderWork::GraphNodeVisits),
            0
        );
    }
}
