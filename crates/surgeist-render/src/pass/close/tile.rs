use super::*;
use crate::frame::TilePassFacts;

impl ClosedGraphTraversal<'_, '_> {
    pub(super) fn visit_tile(&mut self, pass: &RuntimePass, work: &TilePassFacts) -> Option<()> {
        let current = if work.source_plan().is_some() {
            let context = *self.contexts.last()?;
            let source = self.initialized_context_source(context)?;
            if source.producer != RuntimeResourceProducer::Pass(context.producer) {
                return None;
            }
            Some(source)
        } else {
            None
        };
        let result = validate_pass(
            pass,
            work,
            current,
            self.plan.working_format,
            &self.resources,
        )?;
        self.expected_resources.insert(result);
        self.proof.capture_storage_write(self.filter_steps.push(
            ExecutableFilterStepFacts::Tile {
                pass: pass.id,
                source: current.map(|source| source.id),
                result,
            },
        ))?;
        let result = ExecutableCompositionContext {
            current: result,
            producer: pass.id,
            contains_captured_source: true,
        };
        if current.is_some() {
            *self.contexts.last_mut()? = result;
        } else {
            self.proof
                .capture_storage_write(self.contexts.push(result))?;
        }
        self.has_tile = true;
        self.advance(1)
    }
}

pub(super) fn steps_are_exact(
    plan: &LoweredGraphPlan,
    steps: &[ExecutableFilterStepFacts],
    expected: bool,
    proof: &ProofContext<'_>,
) -> bool {
    if !expected {
        return true;
    }
    let exact = (|| -> Option<()> {
        let maps = proof.capture(ClosedGraphMaps::try_new(plan, proof))?;
        for pass in proof_passes(plan, proof) {
            gaussian::composite_footprint_is_exact(pass, plan, &maps)?;
        }
        for step in proof_nodes(steps, proof) {
            let ExecutableFilterStepFacts::Tile {
                pass,
                source,
                result,
            } = step
            else {
                continue;
            };
            let pass = plan.passes.get(*maps.pass_positions.get(pass)?)?;
            let RuntimePassKind::Tile(work) = &pass.kind else {
                return None;
            };
            let source = match source {
                Some(id) => Some(maps.resource_by_id.get(id).copied()?),
                None => None,
            };
            if !proof.work(RenderWork::GraphNodeVisits, 1)
                || validate_pass(
                    pass,
                    work,
                    source,
                    plan.working_format,
                    &maps.resource_by_id,
                )? != *result
            {
                return None;
            }
        }
        Some(())
    })();
    exact.is_some() && !proof.invalid.get()
}

fn validate_pass(
    pass: &RuntimePass,
    work: &TilePassFacts,
    source: Option<&RuntimeResourceRequest>,
    working_format: WorkingFormat,
    resources: &ProofMap<'_, RuntimeResourceId, &RuntimeResourceRequest>,
) -> Option<RuntimeResourceId> {
    let proof = resources.proof;
    let count = usize::from(source.is_some());
    if pass.reads.len() != count
        || pass.dependencies.len() != count
        || pass.releases.len() != count
        || pass.cache_keys.is_none()
    {
        return None;
    }
    let RuntimeResultBinding::Resource(target) = pass.result else {
        return None;
    };
    let resource = resources.get(&target).copied()?;
    let format = RuntimeResourceFormat::Working(working_format);
    if !proof.work(RenderWork::CoordinateVisits, 1)
        || !base_graph_resource_has_fixed_facts(
            resource,
            RuntimeResourceRole::FilterIntermediate,
            format,
            RuntimeResourceProducer::Pass(pass.id),
        )
        || resource.spatial != RuntimeSpatialDescriptor::from_graph(work.target_plan())
        || resource.expected_reads != 1
    {
        return None;
    }
    match (source, work.source_plan()) {
        (Some(source), Some(spatial)) => {
            let RuntimeResourceProducer::Pass(producer) = source.producer else {
                return None;
            };
            if !proof.work(RenderWork::CoordinateVisits, 1)
                || source.id == target
                || source.format != format
                || source.import.is_some()
                || !matches!(
                    source.role,
                    RuntimeResourceRole::FilterIntermediate
                        | RuntimeResourceRole::CompositeResult
                        | RuntimeResourceRole::IsolationWorkingImage
                )
                || source.expected_reads != 1
                || source.last_use != pass.id
                || source.spatial != RuntimeSpatialDescriptor::from_graph(spatial)
                || !proof_contains_edge(&pass.dependencies, &producer, proof)
                || !proof_contains_edge(&pass.releases, &source.id, proof)
                || !runtime_read_has_exact_facts(
                    proof_read(pass, 0, proof)?,
                    RuntimeReadRole::TileSource,
                    source,
                    RuntimeSamplingFilter::Nearest,
                    RuntimeSamplingEdge::TransparentBlack,
                    proof,
                )
            {
                return None;
            }
        }
        (None, None) => {}
        _ => return None,
    }
    Some(target)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        backend::DeviceCapabilities,
        capability::Capabilities,
        effect::FilterDomains,
        frame::{FrameContext, FramePlan},
        geometry::{Rect, Size},
        layer::{LayerComposition, PorterDuffOperator},
        scene::{Layer, Scene},
    };

    fn lower(ink: bool, account: &Account) -> LoweredGraphPlan {
        let bounds = Rect::new(0.0, 0.0, 3.0, 2.0);
        let mut producer = Scene::new();
        if ink {
            producer.fill(bounds, Color::BLACK);
        }
        let layer = Layer::new().with_composition(LayerComposition::normal_porter_duff(
            PorterDuffOperator::Copy,
        ));
        let mut scene = Scene::new();
        scene
            .tile_layer(
                layer,
                producer,
                FilterDomains::try_new(bounds, bounds, bounds).unwrap(),
            )
            .unwrap();
        let commands = scene.normalize(Capabilities::CURRENT, account).unwrap();
        let context = FrameContext::try_new(
            Size::new(3.0, 2.0),
            1.0,
            Antialiasing::Area,
            Color::TRANSPARENT,
        )
        .unwrap();
        let FramePlan::GpuGraph(graph) = commands.plan_for(context, account).unwrap() else {
            panic!("bounded tile must use the GPU graph");
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
    fn tile_closes_repeat_with_exact_source_release_and_transparent_without_a_source() {
        let account = Account::for_test();
        for ink in [false, true] {
            let mut graph = lower(ink, &account);
            let original = ClosedExecutableGraph::try_from_lowered(graph.clone(), &account)
                .unwrap()
                .expect("each admitted tile stage closes");
            assert!(matches!(
                original.facts.filter_steps.as_slice(),
                [ExecutableFilterStepFacts::Tile { source, .. }] if source.is_some() == ink
            ));

            let mut passes = account.retained(graph.passes.len()).unwrap();
            for pass in &*graph.passes {
                let mut pass = pass.clone();
                if matches!(pass.kind, RuntimePassKind::Tile(_)) {
                    if ink {
                        pass.releases = SharedValue::new(Vec::new(), &account.storage()).unwrap();
                    } else {
                        pass.dependencies =
                            SharedValue::new(vec![graph.passes[0].id], &account.storage()).unwrap();
                    }
                }
                passes.append_slot().unwrap().write(pass);
            }
            graph.passes = SharedValue::from_accounted(passes, &account.storage()).unwrap();
            assert!(
                ClosedExecutableGraph::try_from_lowered(graph, &account)
                    .unwrap()
                    .is_err()
            );
        }
    }
}
