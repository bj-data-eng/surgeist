//! Closure proves the complete native Gaussian chain before native acquisition.
use super::*;
use crate::frame::{GaussianAxis, GaussianPassFacts, GaussianStage};

struct Chain {
    count: usize,
    source: Option<RuntimeResourceId>,
    result: RuntimeResourceId,
}

impl ClosedGraphTraversal<'_, '_> {
    pub(super) fn visit_gaussian(&mut self) -> Option<()> {
        let first = self.pass_at(self.cursor)?;
        let RuntimePassKind::Gaussian(work) = &first.kind else {
            return None;
        };
        let constant = work.stage() == GaussianStage::Constant;
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
            ExecutableFilterStepFacts::Gaussian {
                passes,
                source: chain.source,
                result: chain.result,
            },
        ))?;
        self.has_gaussian = true;
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
            if matches!(step, ExecutableFilterStepFacts::Gaussian { .. }) {
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
            composite_footprint_is_exact(pass, plan, &maps)?;
        }
        for step in proof_nodes(steps, proof) {
            let ExecutableFilterStepFacts::Gaussian {
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
                    .map(|p| p.id),
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
    let RuntimePassKind::Gaussian(initial) = &first.kind else {
        return None;
    };
    let mut index = 0usize;
    let mut current = match context {
        Some(c) => {
            let resource = resources.get(&c.current).copied()?;
            if resource.producer != RuntimeResourceProducer::Pass(c.producer)
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
    let source = current.map(|r| r.id);
    if initial.stage() == GaussianStage::Constant {
        if current.is_some() {
            return None;
        }
        let result = stage(
            first,
            initial,
            GaussianStage::Constant,
            &[],
            resources,
            plan.working_format,
        )?;
        return Some(Chain {
            count: 1,
            source: None,
            result: result.id,
        });
    }
    if initial.stage() == GaussianStage::RebaseSource {
        let input = current?;
        current = Some(stage(
            at(index)?,
            initial,
            GaussianStage::RebaseSource,
            &[input],
            resources,
            plan.working_format,
        )?);
        index = index.checked_add(1)?;
        let rebased = current?;
        if rebased.expected_reads != 1 || rebased.last_use != at(index)?.id {
            return None;
        }
    }
    let pass = at(index)?;
    let RuntimePassKind::Gaussian(work) = &pass.kind else {
        return None;
    };
    if !same_operation(work, initial, proof) {
        return None;
    }
    if initial.operation().is_passthrough() {
        let result = stage(
            pass,
            initial,
            GaussianStage::PassThrough,
            &[current?],
            resources,
            plan.working_format,
        )?;
        return Some(Chain {
            count: index.checked_add(1)?,
            source,
            result: result.id,
        });
    }
    let prepared = stage(
        pass,
        initial,
        GaussianStage::Prepare,
        &[current?],
        resources,
        plan.working_format,
    )?;
    let active = work.mapping().active_axes();
    if !active.into_iter().any(|v| v) {
        return None;
    }
    let geometries = [
        work.mapping().geometry(GaussianAxis::X),
        work.mapping().geometry(GaussianAxis::Y),
    ];
    current = Some(prepared);
    index = index.checked_add(1)?;
    for axis in [GaussianAxis::X, GaussianAxis::Y] {
        let Some(geometry) = geometries[axis.index()] else {
            continue;
        };
        let axis_source = current?;
        let batches = usize::try_from(geometry.sample_count().div_ceil(32)).ok()?;
        let last_batch = at(index.checked_add(batches)?.checked_sub(1)?)?;
        if axis_source.expected_reads != u32::try_from(batches).ok()?
            || axis_source.last_use != last_batch.id
        {
            return None;
        }
        let mut previous: Option<&RuntimeResourceRequest> = None;
        let mut offset = 0u32;
        while offset < geometry.sample_count() {
            let pass = at(index)?;
            let RuntimePassKind::Gaussian(actual) = &pass.kind else {
                return None;
            };
            let map = actual.mapping();
            if map.active_axes() != active
                || [map.geometry(GaussianAxis::X), map.geometry(GaussianAxis::Y)] != geometries
            {
                return None;
            }
            let count = (geometry.sample_count() - offset).min(32);
            let expected = GaussianStage::Accumulate {
                axis,
                first_tap: offset,
                tap_count: count,
                first: offset == 0,
            };
            if let Some(carry) = previous
                && (carry.expected_reads != 1 || carry.last_use != pass.id)
            {
                return None;
            }
            let sum = match previous {
                Some(sum) => stage(
                    pass,
                    initial,
                    expected,
                    &[axis_source, sum],
                    resources,
                    plan.working_format,
                )?,
                None => stage(
                    pass,
                    initial,
                    expected,
                    &[axis_source],
                    resources,
                    plan.working_format,
                )?,
            };
            previous = Some(sum);
            offset = offset.checked_add(count)?;
            index = index.checked_add(1)?;
        }
        current = previous;
    }
    let resolve = at(index)?;
    let completed = current?;
    if completed.expected_reads != 1 || completed.last_use != resolve.id {
        return None;
    }
    let result = stage(
        resolve,
        initial,
        GaussianStage::Resolve,
        &[current?],
        resources,
        plan.working_format,
    )?;
    Some(Chain {
        count: index.checked_add(1)?,
        source,
        result: result.id,
    })
}
fn same_operation(a: &GaussianPassFacts, b: &GaussianPassFacts, proof: &ProofContext<'_>) -> bool {
    proof.work(RenderWork::GraphNodeVisits, 1)
        && a.operation() == b.operation()
        && a.source() == b.source()
}
fn stage<'a>(
    pass: &RuntimePass,
    initial: &GaussianPassFacts,
    expected: GaussianStage,
    inputs: &[&'a RuntimeResourceRequest],
    resources: &ProofMap<'_, RuntimeResourceId, &'a RuntimeResourceRequest>,
    working: WorkingFormat,
) -> Option<&'a RuntimeResourceRequest> {
    let proof = resources.proof;
    let RuntimePassKind::Gaussian(actual) = &pass.kind else {
        return None;
    };
    let map = actual.mapping();
    if actual.stage() != expected
        || !same_operation(actual, initial, proof)
        || pass.reads.len() != inputs.len()
        || pass.dependencies.len() != inputs.len()
        || pass.cache_keys.is_none()
        || !proof.capture(map.proves_exact(
            actual.operation(),
            actual.source(),
            expected,
            proof.account,
        ))?
    {
        return None;
    }
    let float = matches!(
        expected,
        GaussianStage::Prepare | GaussianStage::Accumulate { .. }
    );
    let RuntimeResultBinding::Resource(id) = pass.result else {
        return None;
    };
    let result = resources.get(&id).copied()?;
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
    {
        return None;
    }
    if map
        .source_spatial()
        .map(RuntimeSpatialDescriptor::from_graph)
        != inputs.first().map(|r| r.spatial)
    {
        return None;
    }
    let mut release_count = 0usize;
    for input in inputs {
        if !proof.work(RenderWork::GraphEdgeVisits, 1) {
            return None;
        }
        if input.last_use == pass.id {
            release_count = release_count.checked_add(1)?;
            if !proof_contains_edge(&pass.releases, &input.id, proof) {
                return None;
            }
        }
    }
    if pass.releases.len() != release_count {
        return None;
    }
    for (index, (read, input)) in pass.reads.iter().zip(inputs).enumerate() {
        let RuntimeResourceProducer::Pass(producer) = input.producer else {
            return None;
        };
        if !proof.work(RenderWork::GraphEdgeVisits, 1)
            || input.id == id
            || pass.dependencies.get(index) != Some(&producer)
            || !runtime_read_has_exact_facts(
                read,
                if index == 0 {
                    RuntimeReadRole::GaussianSource
                } else {
                    RuntimeReadRole::GaussianAuxiliary
                },
                input,
                RuntimeSamplingFilter::Nearest,
                RuntimeSamplingEdge::TransparentBlack,
                proof,
            )
        {
            return None;
        }
        if index == 1
            && (input.spatial != result.spatial
                || input.format != RuntimeResourceFormat::FilterRgba32Float
                || inputs[0].id == input.id)
        {
            return None;
        }
        if index == 0 {
            let expected_format = if matches!(
                expected,
                GaussianStage::Accumulate { .. } | GaussianStage::Resolve
            ) {
                RuntimeResourceFormat::FilterRgba32Float
            } else {
                RuntimeResourceFormat::Working(working)
            };
            if input.format != expected_format {
                return None;
            }
        }
    }
    Some(result)
}

/// The footprint is valid exactly at the consumer of a completed native chain.
/// Uses the maps already owned by the accounted graph traversal.
pub(super) fn composite_footprint_is_exact(
    pass: &RuntimePass,
    plan: &LoweredGraphPlan,
    maps: &ClosedGraphMaps<'_, '_>,
) -> Option<()> {
    let RuntimePassKind::Composite(Some(RuntimeComposite {
        kind: RuntimeCompositeKind::Layer { parameters, .. },
        ..
    })) = &pass.kind
    else {
        return Some(());
    };
    let proof = maps.resource_by_id.proof;
    let read =
        proof_reads(pass, proof).find(|read| read.role == RuntimeReadRole::CompositeSource)?;
    let source = maps.resource_by_id.get(&read.resource).copied()?;
    let RuntimeResourceProducer::Pass(producer) = source.producer else {
        return None;
    };
    let source_pass = plan.passes.get(*maps.pass_positions.get(&producer)?)?;
    if !proof.work(RenderWork::GraphNodeVisits, 1) {
        return None;
    }
    let expected = match &source_pass.kind {
        RuntimePassKind::Gaussian(work)
            if matches!(
                work.stage(),
                GaussianStage::Constant | GaussianStage::PassThrough | GaussianStage::Resolve
            ) =>
        {
            Some(work.mapping())
        }
        RuntimePassKind::Gaussian(_) => return None,
        _ => None,
    };
    let mut source_releases = 0usize;
    if expected.is_some() {
        for release in &pass.releases {
            if !proof.work(RenderWork::GraphEdgeVisits, 1) {
                return None;
            }
            if *release == source.id {
                source_releases = source_releases.checked_add(1)?;
            }
        }
    }
    match (parameters.gaussian_footprint(), expected) {
        (None, None) => Some(()),
        (Some(footprint), Some(mapping))
            if source.expected_reads == 1
                && source.last_use == pass.id
                && source_releases == 1
                && RuntimeSpatialDescriptor::from_graph(footprint.source_spatial())
                    == source.spatial
                && source.spatial
                    == RuntimeSpatialDescriptor::from_graph(mapping.target_spatial())
                && footprint.output_clip() == mapping.output_clip() =>
        {
            Some(())
        }
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        Capabilities, FilterColorSpace, FilterDomains, GaussianInput, GaussianSource, Layer,
        RenderAccountingContext, Scene, Size,
        backend::DeviceCapabilities,
        frame::{FrameContext, FramePlan},
        prepare_gaussian,
        renderer::Antialiasing,
    };

    fn shared<T: Clone>(values: &[T]) -> SharedValue<Vec<T>> {
        let account = Account::for_test();
        let mut data = account.retained(values.len()).unwrap();
        for value in values {
            data.append_slot().unwrap().write(value.clone());
        }
        SharedValue::from_accounted(data, &account.storage()).unwrap()
    }
    fn plan(kind: u8, sigma: [f64; 2]) -> LoweredGraphPlan {
        let bounds = crate::Rect::new(0.0, 0.0, 4.0, 4.0);
        let operation = prepare_gaussian(
            GaussianInput::new().with_standard_deviations(sigma[0], sigma[1]),
            FilterColorSpace::LinearSrgb,
            &mut RenderAccountingContext::default(),
        )
        .unwrap();
        let mut input = Scene::new();
        input.fill(bounds, Color::BLACK);
        let source = match kind {
            0 | 3 => GaussianSource::finite(
                input,
                FilterDomains::try_new(bounds, bounds, bounds).unwrap(),
            ),
            1 => GaussianSource::constant(Color::BLACK, bounds, bounds).unwrap(),
            _ => GaussianSource::periodic(
                input,
                crate::Rect::new(0.0, 0.0, 2.5, 1.25),
                bounds,
                bounds,
            )
            .unwrap(),
        };
        let mut scene = Scene::new();
        if kind == 3 {
            scene.layer(Layer::new().try_opacity(0.5).unwrap(), |child| {
                child
                    .gaussian_layer(Layer::new(), operation, source)
                    .unwrap();
            });
        } else {
            scene
                .gaussian_layer(Layer::new(), operation, source)
                .unwrap();
        }
        let account = Account::for_test();
        let commands = scene.normalize(Capabilities::CURRENT, &account).unwrap();
        let context = FrameContext::try_new(
            Size::new(4.0, 4.0),
            1.0,
            Antialiasing::Area,
            Color::TRANSPARENT,
        )
        .unwrap();
        let FramePlan::GpuGraph(graph) = commands.plan_for(context, &account).unwrap() else {
            panic!("Gaussian uses native graph");
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
    fn native_gaussian_chains_close_for_finite_constant_periodic_and_identity_sources() {
        for kind in 0..3 {
            for sigma in [[4.1, 2.0], [0.0, 2.0], [2.0, 0.0], [0.0, 0.0], [-1.0, 2.0]] {
                let closed = ClosedExecutableGraph::try_from_lowered(
                    plan(kind, sigma),
                    &Account::for_test(),
                )
                .unwrap()
                .expect("actual native source chain closes");
                assert!(closed.has_spatial_filters());
                assert!(closed.facts.has_gaussian);
                assert!(
                    closed
                        .facts
                        .proves_exact_facts_for(&closed.lowered, &Account::for_test())
                        .unwrap()
                );
                let mut forged = (*closed.facts).clone();
                forged.has_gaussian = false;
                assert!(
                    !forged
                        .proves_exact_facts_for(&closed.lowered, &Account::for_test())
                        .unwrap()
                );
            }
        }
    }
    #[test]
    fn gaussian_closure_rejects_missing_batches_wrong_carry_format_and_last_use() {
        let original = plan(0, [4.1, 2.0]);
        let account = Account::for_test();
        let closed = ClosedExecutableGraph::try_from_lowered(original.clone(), &account)
            .unwrap()
            .expect("native batch chain closes");
        let position=original.passes.iter().position(|pass|matches!(&pass.kind,RuntimePassKind::Gaussian(work) if matches!(work.stage(),GaussianStage::Accumulate{first:false,..}))).unwrap();
        for alteration in 0..7 {
            let mut changed = original.clone();
            let mut passes = changed.passes.to_vec();
            match alteration {
                0 => {
                    passes.remove(position);
                }
                1 => {
                    passes[position].kind = passes[position - 1].kind.clone();
                }
                2 => {
                    let mut reads = passes[position].reads.to_vec();
                    reads[1].resource = reads[0].resource;
                    passes[position].reads = shared(&reads);
                }
                3..=5 => {
                    let RuntimeResultBinding::Resource(id) = passes[position].result else {
                        panic!();
                    };
                    let mut resources = changed.resources.to_vec();
                    let result = resources.iter_mut().find(|r| r.id == id).unwrap();
                    if alteration == 3 {
                        result.format =
                            RuntimeResourceFormat::Working(WorkingFormat::HighPrecision);
                    } else if alteration == 4 {
                        result.last_use = passes[position].id;
                    } else {
                        result.expected_reads += 1;
                    }
                    changed.resources = shared(&resources);
                }
                6 => passes[position].releases = shared(&[]),
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
    fn closure_authenticates_gaussian_footprint_presence_at_the_actual_source_consumer() {
        let original = plan(3, [0.5; 2]);
        let account = Account::for_test();
        let closed = ClosedExecutableGraph::try_from_lowered(original.clone(), &account)
            .unwrap()
            .expect("nested native and ordinary layers close");
        let native=original.passes.iter().position(|pass|matches!(&pass.kind,RuntimePassKind::Composite(Some(RuntimeComposite {kind:RuntimeCompositeKind::Layer {parameters,..},..})) if parameters.gaussian_footprint().is_some())).unwrap();
        let ordinary=original.passes.iter().rposition(|pass|matches!(&pass.kind,RuntimePassKind::Composite(Some(RuntimeComposite {kind:RuntimeCompositeKind::Layer {parameters,..},..})) if parameters.gaussian_footprint().is_none())).unwrap();
        let RuntimePassKind::Composite(Some(RuntimeComposite {
            kind: RuntimeCompositeKind::Layer { parameters, .. },
            ..
        })) = &original.passes[native].kind
        else {
            panic!();
        };
        let footprint = parameters.gaussian_footprint().unwrap();
        for (index, footprint) in [(native, None), (ordinary, Some(footprint))] {
            let mut changed = original.clone();
            let mut passes = changed.passes.to_vec();
            let RuntimePassKind::Composite(Some(RuntimeComposite {
                kind: RuntimeCompositeKind::Layer { parameters, .. },
                ..
            })) = &mut passes[index].kind
            else {
                panic!();
            };
            *parameters = SharedValue::new(
                RuntimeLayerCompositeParameters::try_new(
                    parameters.destination_to_layer_local().affine(),
                    parameters.source_space(),
                    parameters.opacity(),
                    parameters.composition(),
                    parameters.has_clip(),
                    parameters.alpha_mask(),
                    footprint,
                )
                .unwrap(),
                &account.storage(),
            )
            .unwrap();
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
}
