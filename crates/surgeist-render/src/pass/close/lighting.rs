//! Close the finite bump and zero-bump routes over their exact working planes.
use super::*;
use crate::frame::{LightingPassFacts, LightingStage};

struct Chain {
    step: ExecutableFilterStepFacts,
    bump: Option<RuntimeResourceId>,
    shaded: RuntimeResourceId,
    result: RuntimeResourceId,
    resolve: RuntimePassId,
}

impl ClosedGraphTraversal<'_, '_> {
    pub(super) fn visit_lighting(&mut self) -> Option<()> {
        let first = self.pass_at(self.cursor)?;
        let RuntimePassKind::Lighting(work) = &first.kind else {
            return None;
        };
        let context = if work.stage() == LightingStage::ResampleBump {
            let context = *self.contexts.last()?;
            self.initialized_context_source(context)?;
            Some(context)
        } else {
            None
        };
        let chain = validate_chain(self.plan, self.cursor, context, &self.resources)?;
        for result in chain.bump.into_iter().chain([chain.shaded, chain.result]) {
            self.expected_resources.insert(result);
        }
        self.proof
            .capture_storage_write(self.filter_steps.push(chain.step))?;
        self.has_lighting = true;
        let result = ExecutableCompositionContext {
            current: chain.result,
            producer: chain.resolve,
            contains_captured_source: true,
        };
        if context.is_some() {
            *self.contexts.last_mut()? = result;
        } else {
            self.proof
                .capture_storage_write(self.contexts.push(result))?;
        }
        self.advance(2 + usize::from(chain.bump.is_some()))
    }
}

pub(super) fn steps_are_exact(
    plan: &LoweredGraphPlan,
    steps: &[ExecutableFilterStepFacts],
    expected: bool,
    proof: &ProofContext<'_>,
) -> bool {
    let exact = (|| -> Option<()> {
        // The pass-order proof revalidates the presence flag in its traversal.
        if !expected {
            return Some(());
        }
        let maps = proof.capture(ClosedGraphMaps::try_new(plan, proof))?;
        for pass in proof_passes(plan, proof) {
            gaussian::composite_footprint_is_exact(pass, plan, &maps)?;
        }
        for step in proof_nodes(steps, proof) {
            let ExecutableFilterStepFacts::Lighting {
                resample,
                shade,
                source,
                ..
            } = step
            else {
                continue;
            };
            let cursor = *maps.pass_positions.get(&resample.unwrap_or(*shade))?;
            let context = match source {
                Some(current) => {
                    let resource = maps.resource_by_id.get(current)?;
                    let RuntimeResourceProducer::Pass(producer) = resource.producer else {
                        return None;
                    };
                    Some(ExecutableCompositionContext {
                        current: *current,
                        producer,
                        contains_captured_source: true,
                    })
                }
                None => None,
            };
            let chain = validate_chain(plan, cursor, context, &maps.resource_by_id)?;
            if chain.step != *step {
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
    context: Option<ExecutableCompositionContext>,
    resources: &ProofMap<'_, RuntimeResourceId, &RuntimeResourceRequest>,
) -> Option<Chain> {
    let proof = resources.proof;
    let at = |offset: usize| {
        let pass = plan.passes.get(cursor.checked_add(offset)?)?;
        let RuntimePassKind::Lighting(work) = &pass.kind else {
            return None;
        };
        proof
            .work(RenderWork::GraphNodeVisits, 1)
            .then_some((pass, work))
    };
    let working = RuntimeResourceFormat::Working(plan.working_format);
    let (first_pass, operation) = at(0)?;
    let bump = if let Some(context) = context {
        let source = resources.get(&context.current).copied()?;
        if operation.stage() != LightingStage::ResampleBump
            || source.producer != RuntimeResourceProducer::Pass(context.producer)
        {
            return None;
        }
        Some(validate_stage(
            first_pass,
            operation,
            Some(source),
            working,
            resources,
        )?)
    } else {
        None
    };
    let shade_index = usize::from(bump.is_some());
    let (shade, shade_work) = at(shade_index)?;
    if shade_work.stage() != LightingStage::Shade
        || !shade_work.same_operation(operation)
        || !proof.work(RenderWork::GraphEdgeVisits, 1)
    {
        return None;
    }
    let shaded = validate_stage(shade, shade_work, bump, working, resources)?;
    let (resolve, resolve_work) = at(shade_index.checked_add(1)?)?;
    if resolve_work.stage() != LightingStage::Resolve
        || !resolve_work.same_operation(operation)
        || !proof.work(RenderWork::GraphEdgeVisits, 1)
    {
        return None;
    }
    let result = validate_stage(resolve, resolve_work, Some(shaded), working, resources)?;
    Some(Chain {
        step: ExecutableFilterStepFacts::Lighting {
            resample: bump.map(|_| first_pass.id),
            shade: shade.id,
            resolve: resolve.id,
            source: context.map(|context| context.current),
            result: result.id,
        },
        bump: bump.map(|resource| resource.id),
        shaded: shaded.id,
        result: result.id,
        resolve: resolve.id,
    })
}

fn validate_stage<'plan>(
    pass: &RuntimePass,
    work: &LightingPassFacts,
    source: Option<&RuntimeResourceRequest>,
    format: RuntimeResourceFormat,
    resources: &ProofMap<'_, RuntimeResourceId, &'plan RuntimeResourceRequest>,
) -> Option<&'plan RuntimeResourceRequest> {
    let proof = resources.proof;
    let count = usize::from(source.is_some());
    if work.reads_source() != source.is_some()
        || work.source_plan().is_some() != source.is_some()
        || pass.reads.len() != count
        || pass.dependencies.len() != count
        || pass.releases.len() != count
        || pass.cache_keys.is_none()
    {
        return None;
    }
    let RuntimeResultBinding::Resource(target_id) = pass.result else {
        return None;
    };
    let target = resources.get(&target_id).copied()?;
    let target_spatial = RuntimeSpatialDescriptor::from_graph(work.target_plan());
    if !proof.work(RenderWork::CoordinateVisits, 2)
        || !base_graph_resource_has_fixed_facts(
            target,
            RuntimeResourceRole::FilterIntermediate,
            format,
            RuntimeResourceProducer::Pass(pass.id),
        )
        || target.spatial != target_spatial
        || target.expected_reads != 1
        || !super::super::lighting::valid_scissor(work.scissor(), target_spatial.device_extent)
    {
        return None;
    }
    if let Some(source) = source {
        let RuntimeResourceProducer::Pass(producer) = source.producer else {
            return None;
        };
        let edge = match work.stage() {
            LightingStage::ResampleBump => RuntimeSamplingEdge::TransparentBlack,
            LightingStage::Shade | LightingStage::Resolve => RuntimeSamplingEdge::ClampToExtent,
        };
        if !proof.work(RenderWork::CoordinateVisits, 1)
            || source.id == target.id
            || source.format != format
            || source.import.is_some()
            || source.expected_reads != 1
            || source.last_use != pass.id
            || source.spatial != RuntimeSpatialDescriptor::from_graph(work.source_plan()?)
            || !proof_contains_edge(&pass.dependencies, &producer, proof)
            || !proof_contains_edge(&pass.releases, &source.id, proof)
            || !runtime_read_has_exact_facts(
                proof_read(pass, 0, proof)?,
                RuntimeReadRole::LightingSource,
                source,
                RuntimeSamplingFilter::Nearest,
                edge,
                proof,
            )
        {
            return None;
        }
    }
    Some(target)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        accounting::RenderAccountingContext,
        backend::DeviceCapabilities,
        capability::Capabilities,
        effect::{FilterColorSpace, FilterDomains},
        frame::{FrameContext, FramePlan},
        geometry::{Rect, Size},
        lighting::{DiffuseLightingInput, LightingLightInput, LightingSource, prepare_lighting},
        scene::{Layer, Scene},
    };

    fn lower(textured: bool, elevation: f64, account: &Account) -> LoweredGraphPlan {
        let bounds = Rect::new(0.0, 0.0, 3.0, 2.0);
        let operation = prepare_lighting(
            DiffuseLightingInput::new(),
            LightingLightInput::Distant {
                azimuth: 0.0,
                elevation,
            },
            FilterColorSpace::Srgb,
            &mut RenderAccountingContext::default(),
        )
        .unwrap();
        let mut bump = Scene::new();
        if textured {
            bump.fill(Rect::new(0.0, 0.0, 1.0, 2.0), Color::BLACK);
        }
        let mut scene = Scene::new();
        scene.lighting_layer(
            Layer::new(),
            operation,
            LightingSource::finite(
                bump,
                FilterDomains::try_new(bounds, bounds, bounds).unwrap(),
            ),
        );
        let commands = scene.normalize(Capabilities::CURRENT, account).unwrap();
        let context = FrameContext::try_new(
            Size::new(3.0, 2.0),
            1.0,
            Antialiasing::Area,
            Color::TRANSPARENT,
        )
        .unwrap();
        let FramePlan::GpuGraph(graph) = commands.plan_for(context, account).unwrap() else {
            panic!("lighting requires the native graph");
        };
        LoweredGraphPlan::try_lower_validated_graph(
            &graph,
            WorkingFormat::HighPrecision,
            Format::Rgba8,
            &DeviceCapabilities::from_test_facts(true, true, 4096),
            account,
        )
        .unwrap()
    }

    #[test]
    fn lighting_closure_rejects_a_shade_from_another_operation_with_the_same_spatial_planes() {
        let account = Account::for_test();
        for textured in [false, true] {
            let original = lower(textured, 90.0, &account);
            let foreign = lower(textured, 30.0, &account);
            let shade_index = original.passes.iter().position(|pass| {
                matches!(&pass.kind, RuntimePassKind::Lighting(work) if work.stage() == LightingStage::Shade)
            }).unwrap();
            assert!(
                ClosedExecutableGraph::try_from_lowered(original.clone(), &account)
                    .unwrap()
                    .is_ok()
            );
            let mut forged = original;
            let mut passes = account.retained(forged.passes.len()).unwrap();
            for (index, pass) in forged.passes.iter().enumerate() {
                let mut pass = pass.clone();
                if index == shade_index {
                    pass.kind = foreign.passes[index].kind.clone();
                }
                passes.append_slot().unwrap().write(pass);
            }
            forged.passes = SharedValue::from_accounted(passes, &account.storage()).unwrap();
            assert!(
                ClosedExecutableGraph::try_from_lowered(forged, &account)
                    .unwrap()
                    .is_err()
            );
        }
    }
}
