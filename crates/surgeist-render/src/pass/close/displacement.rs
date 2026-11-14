//! Authenticate the source, raw map, tables and records of each displacement route.
use super::*;
use crate::frame::{DisplacementSourceKind, DisplacementStage};

struct Chain {
    count: usize,
    source: Option<RuntimeResourceId>,
    result: RuntimeResourceId,
}

impl ClosedGraphTraversal<'_, '_> {
    pub(super) fn visit_displacement(&mut self) -> Option<()> {
        let first = self.pass_at(self.cursor)?;
        let RuntimePassKind::Displacement(initial) = &first.kind else {
            return None;
        };
        let standalone = initial.work.mapping().source_spatial().is_none();
        let context = if standalone {
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
            ExecutableFilterStepFacts::Displacement {
                passes,
                source: chain.source,
                result: chain.result,
            },
        ))?;
        self.has_displacement = true;
        let result = ExecutableCompositionContext {
            current: chain.result,
            producer,
            contains_captured_source: true,
        };
        if standalone {
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
        let present = proof_nodes(steps, proof)
            .any(|step| matches!(step, ExecutableFilterStepFacts::Displacement { .. }));
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
            let ExecutableFilterStepFacts::Displacement {
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
            if chain.count != passes.len()
                || chain.source != *source
                || chain.result != *result
                || !proof_relationships_equal(
                    plan.passes
                        .get(cursor..cursor.checked_add(chain.count)?)?
                        .iter()
                        .map(|pass| pass.id),
                    passes.iter().copied(),
                    proof,
                    |a, b| a == b,
                )
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
    context: Option<ExecutableCompositionContext>,
    resources: &ProofMap<'_, RuntimeResourceId, &RuntimeResourceRequest>,
) -> Option<Chain> {
    let proof = resources.proof;
    let first = plan.passes.get(cursor)?;
    let RuntimePassKind::Displacement(initial) = &first.kind else {
        return None;
    };
    let count = initial.work.route_len();
    if initial.work.route_index() != 0 || !(count == 1 || count == 2) {
        return None;
    }
    let mut current = context.and_then(|context| resources.get(&context.current).copied());
    if initial.work.mapping().source_spatial().is_some() != current.is_some() {
        return None;
    }
    if let (Some(context), Some(resource)) = (context, current)
        && (resource.producer != RuntimeResourceProducer::Pass(context.producer)
            || resource.expected_reads != 1
            || resource.last_use != first.id)
    {
        return None;
    }
    let source = current.map(|resource| resource.id);
    for index in 0..count {
        let pass = plan.passes.get(cursor.checked_add(index)?)?;
        if !proof.work(RenderWork::GraphNodeVisits, 1) {
            return None;
        }
        let RuntimePassKind::Displacement(runtime) = &pass.kind else {
            return None;
        };
        let work = &runtime.work;
        if work.route_index() != index
            || work.route_len() != count
            || !proof.capture(initial.work.same_route_in(work, proof.account))?
            || !proof.capture(runtime.proves_exact(pass.id, proof.account))?
        {
            return None;
        }
        let valid_stage = if count == 2 {
            if index == 0 {
                work.stage() == DisplacementStage::RealizeSource
            } else {
                work.stage() == DisplacementStage::Displace
            }
        } else {
            matches!(
                work.stage(),
                DisplacementStage::PassThrough | DisplacementStage::Transparent
            )
        };
        if !valid_stage {
            return None;
        }
        let input = if work.mapping().source_spatial().is_some() {
            Some(current?)
        } else {
            None
        };
        let reads = usize::from(input.is_some());
        if pass.reads.len() != reads
            || pass.dependencies.len() != reads
            || pass.releases.len() != reads
            || pass.cache_keys.is_none()
        {
            return None;
        }
        let RuntimeResultBinding::Resource(id) = pass.result else {
            return None;
        };
        let result = resources.get(&id).copied()?;
        if !base_graph_resource_has_fixed_facts(
            result,
            RuntimeResourceRole::FilterIntermediate,
            RuntimeResourceFormat::Working(plan.working_format),
            RuntimeResourceProducer::Pass(pass.id),
        ) || result.spatial
            != RuntimeSpatialDescriptor::from_graph(work.mapping().target_spatial())
        {
            return None;
        }
        if let Some(input) = input {
            let RuntimeResourceProducer::Pass(producer) = input.producer else {
                return None;
            };
            if input.id == id
                || input.format != RuntimeResourceFormat::Working(plan.working_format)
                || Some(input.spatial)
                    != work
                        .mapping()
                        .source_spatial()
                        .map(RuntimeSpatialDescriptor::from_graph)
                || input.expected_reads != 1
                || input.last_use != pass.id
                || pass.dependencies.first() != Some(&producer)
                || !runtime_read_has_exact_facts(
                    proof_read(pass, 0, proof)?,
                    RuntimeReadRole::DisplacementSource,
                    input,
                    RuntimeSamplingFilter::Nearest,
                    RuntimeSamplingEdge::TransparentBlack,
                    proof,
                )
                || !proof_contains_edge(&pass.releases, &input.id, proof)
            {
                return None;
            }
        } else if !matches!(
            work.mapping().source_kind(),
            DisplacementSourceKind::Constant
        ) {
            return None;
        }
        current = Some(result);
    }
    Some(Chain {
        count,
        source,
        result: current?.id,
    })
}
