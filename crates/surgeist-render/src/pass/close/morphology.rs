//! Authenticate every stage and both resource histories of an exact extrema route.
use super::*;
use crate::frame::{MorphologyAxis, MorphologyPassFacts, MorphologyStage};

pub(super) struct Chain {
    count: usize,
    source: Option<RuntimeResourceId>,
    result: RuntimeResourceId,
}

impl ClosedGraphTraversal<'_, '_> {
    pub(super) fn visit_morphology(&mut self) -> Option<()> {
        let first = self.pass_at(self.cursor)?;
        let RuntimePassKind::Morphology(work) = &first.kind else {
            return None;
        };
        let constant = work.stage() == MorphologyStage::Constant;
        let context = if constant {
            None
        } else {
            let context = *self.contexts.last()?;
            self.initialized_context_source(context)?;
            Some(context)
        };
        let chain = validate_chain(self.plan, self.cursor, context, &self.resources)?;
        let mut ids = self
            .proof
            .capture(self.proof.account.retained(chain.count))?;
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
            ExecutableFilterStepFacts::Morphology {
                passes,
                source: chain.source,
                result: chain.result,
            },
        ))?;
        self.has_morphology = true;
        let result = ExecutableCompositionContext {
            current: chain.result,
            producer,
            contains_captured_source: true,
        };
        if constant {
            self.proof
                .capture_storage_write(self.contexts.push(result))?;
        } else {
            *self.contexts.last_mut()? = result;
        }
        self.advance(chain.count)
    }
}

pub(super) fn steps_are_exact(
    plan: &LoweredGraphPlan,
    steps: &[ExecutableFilterStepFacts],
    expected: bool,
    proof: &ProofContext<'_>,
) -> bool {
    let exact = (|| -> Option<()> {
        let mut present = false;
        for step in proof_nodes(steps, proof) {
            if matches!(step, ExecutableFilterStepFacts::Morphology { .. }) {
                present = true;
            }
        }
        if present != expected {
            return None;
        }
        if !present {
            return Some(());
        }
        let maps = proof.capture(ClosedGraphMaps::try_new(plan, proof))?;
        for pass in proof_passes(plan, proof) {
            gaussian::composite_footprint_is_exact(pass, plan, &maps)?;
        }
        for step in proof_nodes(steps, proof) {
            let ExecutableFilterStepFacts::Morphology {
                passes,
                source,
                result,
            } = step
            else {
                continue;
            };
            let cursor = *maps.pass_positions.get(passes.first()?)?;
            let context = match source {
                Some(id) => {
                    let resource = maps.resource_by_id.get(id)?;
                    let RuntimeResourceProducer::Pass(producer) = resource.producer else {
                        return None;
                    };
                    Some(ExecutableCompositionContext {
                        current: *id,
                        producer,
                        contains_captured_source: true,
                    })
                }
                None => None,
            };
            let chain = validate_chain(plan, cursor, context, &maps.resource_by_id)?;
            if chain.count != passes.len() || chain.source != *source || chain.result != *result {
                return None;
            }
            if !proof_relationships_equal(
                plan.passes
                    .get(cursor..cursor.checked_add(chain.count)?)?
                    .iter()
                    .map(|pass| pass.id),
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
    context: Option<ExecutableCompositionContext>,
    resources: &ProofMap<'_, RuntimeResourceId, &RuntimeResourceRequest>,
) -> Option<Chain> {
    let proof = resources.proof;
    let at = |offset: usize| {
        let pass = plan.passes.get(cursor.checked_add(offset)?)?;
        proof.work(RenderWork::GraphNodeVisits, 1).then_some(pass)
    };
    let first = at(0)?;
    let RuntimePassKind::Morphology(initial) = &first.kind else {
        return None;
    };
    let count = initial.route_len();
    if initial.route_index() != 0 || count == 0 {
        return None;
    }
    let mut completed = match context {
        Some(context) => {
            let resource = resources.get(&context.current).copied()?;
            if resource.producer != RuntimeResourceProducer::Pass(context.producer)
                || resource.format != RuntimeResourceFormat::Working(plan.working_format)
                || resource.expected_reads != 1
                || resource.last_use != first.id
                || !matches!(
                    resource.role,
                    RuntimeResourceRole::FilterIntermediate
                        | RuntimeResourceRole::CompositeResult
                        | RuntimeResourceRole::IsolationWorkingImage
                )
            {
                return None;
            }
            Some(resource)
        }
        None => None,
    };
    let source = completed.map(|resource| resource.id);
    if (initial.stage() == MorphologyStage::Constant) != completed.is_none() {
        return None;
    }
    let mut axis: Option<MorphologyAxis> = None;
    let mut table = None;
    let mut query = None;
    let mut query_complete = true;
    for index in 0..count {
        let pass = at(index)?;
        let RuntimePassKind::Morphology(work) = &pass.kind else {
            return None;
        };
        if work.route_index() != index
            || work.route_len() != count
            || !proof.capture(initial.same_route_in(work, proof.account))?
            || !proof.capture(work.proves_exact(proof.account))?
        {
            return None;
        }
        let expected = proof.capture(initial.expected_stage(index, proof.account))??;
        if work.stage() != expected {
            return None;
        }
        if let Some(next_axis) = expected.axis()
            && axis != Some(next_axis)
        {
            // A later axis may consume only the complete result of its predecessor.
            if !query_complete {
                return None;
            }
            axis = Some(next_axis);
            table = completed;
            query = None;
        }
        let result = match expected {
            MorphologyStage::Constant => {
                if index != 0 || count != 1 || completed.is_some() {
                    return None;
                }
                stage(pass, work, &[], resources, plan.working_format)?
            }
            MorphologyStage::RebaseSource | MorphologyStage::Copy | MorphologyStage::Prepare => {
                if !query_complete {
                    return None;
                }
                stage(pass, work, &[completed?], resources, plan.working_format)?
            }
            MorphologyStage::Table { .. } => {
                let result = stage(pass, work, &[table?], resources, plan.working_format)?;
                table = Some(result);
                continue;
            }
            MorphologyStage::FirstQuery { .. } => {
                if query.is_some() {
                    return None;
                }
                let result = stage(pass, work, &[table?], resources, plan.working_format)?;
                query = Some(result);
                query_complete = work.mapping().query()?.max_level() == 0;
                if query_complete {
                    completed = Some(result);
                }
                continue;
            }
            MorphologyStage::RefineQuery { level, .. } => {
                if query_complete {
                    return None;
                }
                let result = stage(
                    pass,
                    work,
                    &[table?, query?],
                    resources,
                    plan.working_format,
                )?;
                query = Some(result);
                query_complete = level == work.mapping().query()?.max_level();
                if query_complete {
                    completed = Some(result);
                }
                continue;
            }
            MorphologyStage::Query { .. } => {
                if query.is_some() {
                    return None;
                }
                let result = stage(pass, work, &[table?], resources, plan.working_format)?;
                query = Some(result);
                query_complete = true;
                completed = Some(result);
                continue;
            }
            MorphologyStage::Resolve => {
                if !query_complete {
                    return None;
                }
                stage(pass, work, &[completed?], resources, plan.working_format)?
            }
        };
        completed = Some(result);
    }
    let last = at(count.checked_sub(1)?)?;
    let RuntimePassKind::Morphology(work) = &last.kind else {
        return None;
    };
    if !query_complete
        || !matches!(
            work.stage(),
            MorphologyStage::Constant | MorphologyStage::Copy | MorphologyStage::Resolve
        )
    {
        return None;
    }
    Some(Chain {
        count,
        source,
        result: completed?.id,
    })
}

fn stage<'a>(
    pass: &RuntimePass,
    work: &MorphologyPassFacts,
    inputs: &[&'a RuntimeResourceRequest],
    resources: &ProofMap<'_, RuntimeResourceId, &'a RuntimeResourceRequest>,
    working: WorkingFormat,
) -> Option<&'a RuntimeResourceRequest> {
    let proof = resources.proof;
    let map = work.mapping();
    let expected_count = match work.stage() {
        MorphologyStage::Constant => 0,
        MorphologyStage::RefineQuery { .. } => 2,
        _ => 1,
    };
    if inputs.len() != expected_count
        || pass.reads.len() != expected_count
        || pass.dependencies.len() != expected_count
        || pass.cache_keys.is_none()
    {
        return None;
    }
    let RuntimeResultBinding::Resource(id) = pass.result else {
        return None;
    };
    let result = resources.get(&id).copied()?;
    let float = work.needs_float32();
    if !base_graph_resource_has_fixed_facts(
        result,
        if float {
            RuntimeResourceRole::FilterFloat
        } else {
            RuntimeResourceRole::FilterIntermediate
        },
        if float {
            RuntimeResourceFormat::FilterRgba32Float
        } else {
            RuntimeResourceFormat::Working(working)
        },
        RuntimeResourceProducer::Pass(pass.id),
    ) || result.spatial != RuntimeSpatialDescriptor::from_graph(map.target_spatial())
        || map
            .source_spatial()
            .map(RuntimeSpatialDescriptor::from_graph)
            != inputs.first().map(|r| r.spatial)
        || map
            .previous_query_spatial()
            .map(RuntimeSpatialDescriptor::from_graph)
            != inputs.get(1).map(|r| r.spatial)
    {
        return None;
    }
    let mut releases = 0usize;
    for (index, input) in inputs.iter().enumerate() {
        if !proof.work(RenderWork::GraphEdgeVisits, 1) {
            return None;
        }
        let RuntimeResourceProducer::Pass(producer) = input.producer else {
            return None;
        };
        let expected_format = if index == 1
            || matches!(
                work.stage(),
                MorphologyStage::Table { .. }
                    | MorphologyStage::FirstQuery { .. }
                    | MorphologyStage::RefineQuery { .. }
                    | MorphologyStage::Query { .. }
                    | MorphologyStage::Resolve
            ) {
            RuntimeResourceFormat::FilterRgba32Float
        } else {
            RuntimeResourceFormat::Working(working)
        };
        if input.id == id
            || input.format != expected_format
            || (index == 1 && inputs[0].id == input.id)
            || pass.dependencies.get(index) != Some(&producer)
            || !runtime_read_has_exact_facts(
                proof_read(pass, index, proof)?,
                if index == 0 {
                    RuntimeReadRole::MorphologySource
                } else {
                    RuntimeReadRole::MorphologyPreviousQuery
                },
                input,
                RuntimeSamplingFilter::Nearest,
                RuntimeSamplingEdge::TransparentBlack,
                proof,
            )
        {
            return None;
        }
        if input.last_use == pass.id {
            releases = releases.checked_add(1)?;
            if !proof_contains_edge(&pass.releases, &input.id, proof) {
                return None;
            }
        }
    }
    if releases != pass.releases.len() {
        return None;
    }
    Some(result)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        Capabilities, FilterColorSpace, FilterDomains, Layer, MorphologyInput, MorphologyOperator,
        MorphologySource, Rect, RenderAccountingContext, Scene, Size,
        backend::DeviceCapabilities,
        frame::{FrameContext, FramePlan},
        prepare_morphology,
    };

    fn shared<T: Clone>(values: &[T]) -> SharedValue<Vec<T>> {
        let account = Account::for_test();
        let mut data = account.retained(values.len()).unwrap();
        for value in values {
            data.append_slot().unwrap().write(value.clone());
        }
        SharedValue::from_accounted(data, &account.storage()).unwrap()
    }

    fn plan(kind: u8, radii: [f64; 2], working: WorkingFormat) -> LoweredGraphPlan {
        let bounds = Rect::new(0.0, 0.0, 5.0, 5.0);
        let operation = prepare_morphology(
            MorphologyInput::new()
                .with_operator(MorphologyOperator::Dilate)
                .with_radii(radii[0], radii[1]),
            FilterColorSpace::LinearSrgb,
            &mut RenderAccountingContext::default(),
        )
        .unwrap();
        let mut input = Scene::new();
        input.fill(bounds, Color::BLACK);
        let source = match kind {
            1 => MorphologySource::constant(Color::BLACK, bounds, bounds).unwrap(),
            2 => MorphologySource::periodic(input, Rect::new(0.0, 0.0, 2.5, 1.25), bounds, bounds)
                .unwrap(),
            _ => MorphologySource::finite(
                input,
                FilterDomains::try_new(bounds, bounds, bounds).unwrap(),
            ),
        };
        let mut scene = Scene::new();
        if kind == 3 {
            scene.layer(Layer::new().try_opacity(0.5).unwrap(), |child| {
                child
                    .morphology_layer(Layer::new(), operation, source)
                    .unwrap();
            });
        } else {
            scene
                .morphology_layer(Layer::new(), operation, source)
                .unwrap();
        }
        let account = Account::for_test();
        let commands = scene.normalize(Capabilities::CURRENT, &account).unwrap();
        let context = FrameContext::try_new(
            Size::new(5.0, 5.0),
            1.0,
            Antialiasing::Area,
            Color::TRANSPARENT,
        )
        .unwrap();
        let FramePlan::GpuGraph(graph) = commands.plan_for(context, &account).unwrap() else {
            panic!("morphology uses its native graph");
        };
        LoweredGraphPlan::try_lower_validated_graph(
            &graph,
            working,
            Format::Rgba8,
            &DeviceCapabilities::from_test_facts(true, true, 4096),
            &account,
        )
        .unwrap()
    }

    #[test]
    fn native_morphology_complete_routes_close_for_sources_point_axes_and_working_formats() {
        for working in [
            WorkingFormat::HighPrecision,
            WorkingFormat::ReducedPrecision,
        ] {
            for kind in 0..4 {
                for radii in [
                    [2.0; 2],
                    [0.25, 2.0],
                    [2.0, 0.25],
                    [0.25; 2],
                    [0.0, 2.0],
                    [-1.0, 2.0],
                ] {
                    let account = Account::for_test();
                    let closed = ClosedExecutableGraph::try_from_lowered(
                        plan(kind, radii, working),
                        &account,
                    )
                    .unwrap()
                    .unwrap();
                    assert!(closed.has_spatial_filters());
                    assert!(closed.facts.has_morphology);
                    assert!(
                        closed
                            .facts
                            .proves_exact_facts_for(&closed.lowered, &account)
                            .unwrap()
                    );
                    let mut forged = (*closed.facts).clone();
                    forged.has_morphology = false;
                    assert!(
                        !forged
                            .proves_exact_facts_for(&closed.lowered, &account)
                            .unwrap()
                    );
                }
            }
        }
    }

    #[test]
    fn morphology_closure_rejects_refinement_reads_levels_incomplete_queries_and_early_retirement()
    {
        let original = plan(0, [2.0; 2], WorkingFormat::HighPrecision);
        let account = Account::for_test();
        let closed = ClosedExecutableGraph::try_from_lowered(original.clone(), &account)
            .unwrap()
            .unwrap();
        let refine = original.passes.iter().position(|pass| matches!(&pass.kind,
            RuntimePassKind::Morphology(work) if matches!(work.stage(), MorphologyStage::RefineQuery { level: 1, .. }))).unwrap();
        let first = original.passes.iter().position(|pass| matches!(&pass.kind,
            RuntimePassKind::Morphology(work) if matches!(work.stage(), MorphologyStage::FirstQuery { .. }))).unwrap();
        let resolve = original
            .passes
            .iter()
            .position(|pass| {
                matches!(&pass.kind,
            RuntimePassKind::Morphology(work) if work.stage() == MorphologyStage::Resolve)
            })
            .unwrap();
        let seed = match original.passes[first].result {
            RuntimeResultBinding::Resource(id) => id,
            _ => panic!(),
        };
        let table = original.passes[refine].reads[0].resource;
        let table_pass = original
            .passes
            .iter()
            .position(|pass| pass.result == RuntimeResultBinding::Resource(table))
            .unwrap();
        for alteration in 0..7 {
            let mut changed = original.clone();
            let mut passes = changed.passes.to_vec();
            match alteration {
                0 | 1 => {
                    let mut reads = passes[refine].reads.to_vec();
                    reads.remove(alteration);
                    passes[refine].reads = shared(&reads);
                }
                2 => {
                    let RuntimePassKind::Morphology(work) = &passes[table_pass].kind else {
                        panic!();
                    };
                    let MorphologyStage::Table { axis, .. } = work.stage() else {
                        panic!();
                    };
                    passes[table_pass].kind = RuntimePassKind::Morphology(
                        work.with_stage_for_test(
                            MorphologyStage::Table { axis, level: 2 },
                            &account,
                        )
                        .unwrap(),
                    );
                }
                3 => {
                    let mut reads = passes[resolve].reads.to_vec();
                    reads[0].resource = seed;
                    passes[resolve].reads = shared(&reads);
                }
                4 | 5 => {
                    let mut resources = changed.resources.to_vec();
                    let id = if alteration == 4 { table } else { seed };
                    let resource = resources
                        .iter_mut()
                        .find(|resource| resource.id == id)
                        .unwrap();
                    resource.last_use = if alteration == 4 {
                        passes[refine].id
                    } else {
                        passes[first].id
                    };
                    changed.resources = shared(&resources);
                }
                6 => {
                    passes.remove(refine);
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
                "repeated alteration {alteration}"
            );
        }
    }

    #[test]
    fn morphology_footprint_rejects_seed_and_authenticates_only_final_output() {
        let original = plan(3, [2.0; 2], WorkingFormat::HighPrecision);
        let account = Account::for_test();
        let closed = ClosedExecutableGraph::try_from_lowered(original.clone(), &account)
            .unwrap()
            .unwrap();
        let composite = original.passes.iter().position(|pass| matches!(&pass.kind,
            RuntimePassKind::Composite(Some(RuntimeComposite { kind: RuntimeCompositeKind::Layer { parameters, .. }, .. }))
                if parameters.filter_footprint().is_some())).unwrap();
        let seed = original.passes.iter().find(|pass| matches!(&pass.kind,
            RuntimePassKind::Morphology(work) if matches!(work.stage(), MorphologyStage::FirstQuery { .. }))).unwrap();
        let RuntimeResultBinding::Resource(seed) = seed.result else {
            panic!();
        };
        let mut changed = original.clone();
        let mut passes = changed.passes.to_vec();
        let mut reads = passes[composite].reads.to_vec();
        reads
            .iter_mut()
            .find(|read| read.role == RuntimeReadRole::CompositeSource)
            .unwrap()
            .resource = seed;
        passes[composite].reads = shared(&reads);
        changed.passes = shared(&passes);
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
