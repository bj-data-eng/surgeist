//! One closed two-branch chain: raw original, consumed alpha Gaussian, covered
//! offset, and the final merge. Standalone filter lifetime rules stay separate.
use super::*;
use crate::frame::{
    GaussianPurpose, GaussianStage, NativeShadowOriginal, OffsetPurpose, OffsetStage,
};

struct Chain {
    count: usize,
    source: Option<RuntimeResourceId>,
    result: RuntimeResourceId,
}

impl ClosedGraphTraversal<'_, '_> {
    pub(super) fn visit_native_shadow(&mut self) -> Option<()> {
        let first = self.pass_at(self.cursor)?;
        let has_source = match &first.kind {
            RuntimePassKind::Gaussian(work) => work.stage() != GaussianStage::Constant,
            RuntimePassKind::NativeShadowMerge(work) => {
                work.mapping().original_kind() != NativeShadowOriginal::Constant
            }
            _ => return None,
        };
        let context = if has_source {
            let context = *self.contexts.last()?;
            self.initialized_context_source(context)?;
            Some(context)
        } else {
            None
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
        let producer = *ids.last()?;
        let passes = self.proof.capture(SharedValue::from_accounted(
            ids,
            &self.proof.account.storage(),
        ))?;
        self.proof.capture_storage_write(self.filter_steps.push(
            ExecutableFilterStepFacts::NativeShadow {
                passes,
                source: chain.source,
                result: chain.result,
            },
        ))?;
        self.has_native_shadow = true;
        let result = ExecutableCompositionContext {
            current: chain.result,
            producer,
            contains_captured_source: true,
        };
        if has_source {
            *self.contexts.last_mut()? = result;
        } else {
            self.proof
                .capture_storage_write(self.contexts.push(result))?;
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
            if matches!(step, ExecutableFilterStepFacts::NativeShadow { .. }) {
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
            let ExecutableFilterStepFacts::NativeShadow {
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
    let mut index = 0usize;
    let mut original = context.map(|c| c.current);
    let mut gaussian_result = None;
    let mut alpha = None;
    let mut initial_gaussian = None;
    if let RuntimePassKind::Gaussian(initial) = &first.kind {
        if initial.purpose() != GaussianPurpose::ConsumedAlpha {
            return None;
        }
        initial_gaussian = Some(initial);
        // A backdrop raw rebase can also precede an original-only merge.
        if initial.stage() == GaussianStage::RebaseSource
            && matches!(at(1)?.kind, RuntimePassKind::NativeShadowMerge(_))
        {
            let context = context?;
            let source = resources.get(&context.current).copied()?;
            if source.producer != RuntimeResourceProducer::Pass(context.producer)
                || source.expected_reads != 1
                || source.last_use != first.id
            {
                return None;
            }
            let raw = gaussian::stage(
                first,
                initial,
                GaussianStage::RebaseSource,
                &[source],
                resources,
                plan.working_format,
            )?;
            original = Some(raw.id);
            index = 1;
        } else {
            let chain = gaussian::validate_chain(plan, cursor, context, resources)?;
            original = chain.original;
            gaussian_result = Some(chain.result);
            index = chain.count;
            let sample = at(index)?;
            let RuntimePassKind::Offset(work) = &sample.kind else {
                return None;
            };
            if work.purpose() != OffsetPurpose::CoveredAlphaDeferred
                || work.stage() != OffsetStage::Sample
            {
                return None;
            }
            let source = resources.get(&chain.result).copied()?;
            let shifted =
                offset::validate_stage(sample, work, source, resources, plan.working_format)?;
            if !offset::mapping_is_exact(work, source.spatial, shifted.spatial, proof) {
                return None;
            }
            alpha = Some(shifted.id);
            index = index.checked_add(1)?;
        }
    }
    let merge = at(index)?;
    let RuntimePassKind::NativeShadowMerge(work) = &merge.kind else {
        return None;
    };
    let map = work.mapping();
    if map.has_shifted_alpha() != alpha.is_some()
        || merge.cache_keys.is_none()
        || !proof.capture(map.proves_exact(work.operation(), work.source(), proof.account))?
    {
        return None;
    }
    if let Some(initial) = initial_gaussian
        && (initial.operation() != &work.operation().gaussian()
            || initial.source() != work.source().for_filter_output()
            || initial.channels() != crate::frame::GaussianChannels::Alpha)
    {
        return None;
    }
    if let Some(gaussian_result) = gaussian_result {
        let gaussian = resources.get(&gaussian_result).copied()?;
        let shifted = resources.get(&alpha?).copied()?;
        let RuntimeResourceProducer::Pass(sample_id) = shifted.producer else {
            return None;
        };
        let sample = at(index.checked_sub(1)?)?;
        let RuntimePassKind::Offset(offset) = &sample.kind else {
            return None;
        };
        let filter = work.source().filter();
        let output = offset::intersection(work.source().result(), filter);
        let domains = crate::FilterDomains::try_new(filter, filter, output).ok()?;
        if sample.id != sample_id
            || offset.operation() != &work.operation().offset(domains)
            || gaussian.spatial.logical_bounds != filter
            || shifted.spatial != RuntimeSpatialDescriptor::from_graph(map.target_spatial())
        {
            return None;
        }
    }
    let original_required = map.original_kind() != NativeShadowOriginal::Constant;
    if original_required != original.is_some() {
        return None;
    }
    let count = usize::from(original_required) + usize::from(alpha.is_some());
    if merge.reads.len() != count
        || merge.dependencies.len() != count
        || merge.releases.len() != count
    {
        return None;
    }
    let mut read_index = 0usize;
    for (id, role, expected_reads) in original
        .into_iter()
        .map(|id| {
            (
                id,
                RuntimeReadRole::NativeShadowOriginal,
                if gaussian_result.is_some() { 2 } else { 1 },
            )
        })
        .chain(
            alpha
                .into_iter()
                .map(|id| (id, RuntimeReadRole::NativeShadowAlpha, 1)),
        )
    {
        if !proof.work(RenderWork::GraphEdgeVisits, 1) {
            return None;
        }
        let input = resources.get(&id).copied()?;
        let RuntimeResourceProducer::Pass(producer) = input.producer else {
            return None;
        };
        if input.format != RuntimeResourceFormat::Working(plan.working_format)
            || input.import.is_some()
            || input.expected_reads != expected_reads
            || input.last_use != merge.id
            || merge.dependencies.get(read_index) != Some(&producer)
            || !proof_contains_edge(&merge.releases, &id, proof)
            || !runtime_read_has_exact_facts(
                proof_read(merge, read_index, proof)?,
                role,
                input,
                RuntimeSamplingFilter::Nearest,
                RuntimeSamplingEdge::TransparentBlack,
                proof,
            )
        {
            return None;
        }
        if role == RuntimeReadRole::NativeShadowOriginal
            && Some(input.spatial)
                != map
                    .original_mapping()
                    .source_spatial()
                    .map(RuntimeSpatialDescriptor::from_graph)
        {
            return None;
        }
        if role == RuntimeReadRole::NativeShadowAlpha
            && input.spatial != RuntimeSpatialDescriptor::from_graph(map.target_spatial())
        {
            return None;
        }
        read_index = read_index.checked_add(1)?;
    }
    let RuntimeResultBinding::Resource(id) = merge.result else {
        return None;
    };
    let result = resources.get(&id).copied()?;
    if original == Some(id)
        || alpha == Some(id)
        || original.is_some() && original == alpha
        || !base_graph_resource_has_fixed_facts(
            result,
            RuntimeResourceRole::FilterIntermediate,
            RuntimeResourceFormat::Working(plan.working_format),
            RuntimeResourceProducer::Pass(merge.id),
        )
        || result.spatial != RuntimeSpatialDescriptor::from_graph(map.target_spatial())
    {
        return None;
    }
    Some(Chain {
        count: index.checked_add(1)?,
        source: context.map(|c| c.current),
        result: id,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        Capabilities, DropShadowInput, DropShadowSource, FilterColorSpace, FilterDomains, Layer,
        Rect, RenderAccountingContext, Scene, Size,
        backend::DeviceCapabilities,
        frame::{FrameContext, FramePlan},
        prepare_drop_shadow,
    };

    fn shared<T: Clone>(values: &[T]) -> SharedValue<Vec<T>> {
        let account = Account::for_test();
        let mut data = account.retained(values.len()).unwrap();
        for value in values {
            data.append_slot().unwrap().write(value.clone());
        }
        SharedValue::from_accounted(data, &account.storage()).unwrap()
    }
    fn plan(
        kind: u8,
        sigma: [f64; 2],
        transparent: bool,
        format: WorkingFormat,
    ) -> LoweredGraphPlan {
        let bounds = Rect::new(0.0, 0.0, 4.0, 4.0);
        let operation = prepare_drop_shadow(
            DropShadowInput::new()
                .with_dx(0.5)
                .with_dy(-0.25)
                .with_standard_deviations(sigma[0], sigma[1])
                .with_flood_color(if transparent {
                    Color::TRANSPARENT
                } else {
                    Color::BLACK
                }),
            FilterColorSpace::Srgb,
            &mut RenderAccountingContext::default(),
        )
        .unwrap();
        let mut input = Scene::new();
        input.fill(bounds, Color::BLACK);
        let source = match kind {
            1 => DropShadowSource::constant(Color::BLACK, bounds, bounds).unwrap(),
            2 => DropShadowSource::periodic(input, Rect::new(0.0, 0.0, 2.5, 1.25), bounds, bounds)
                .unwrap(),
            _ => DropShadowSource::finite(
                input,
                FilterDomains::try_new(bounds, bounds, bounds).unwrap(),
            ),
        };
        let mut scene = Scene::new();
        if kind == 3 {
            scene.layer(Layer::new().try_opacity(0.5).unwrap(), |child| {
                child
                    .drop_shadow_layer(Layer::new(), operation, source)
                    .unwrap();
            });
        } else {
            scene
                .drop_shadow_layer(Layer::new(), operation, source)
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
            panic!("native shadow needs native graph");
        };
        LoweredGraphPlan::try_lower_validated_graph(
            &graph,
            format,
            Format::Rgba8,
            &DeviceCapabilities::from_test_facts(true, true, 4096),
            &account,
        )
        .unwrap()
    }
    #[test]
    fn complete_native_shadow_chains_close_for_actual_modes_sources_and_working_formats() {
        for format in [
            WorkingFormat::HighPrecision,
            WorkingFormat::ReducedPrecision,
        ] {
            for kind in 0..4 {
                for sigma in [[4.1, 2.0], [0.0, 2.0], [2.0, 0.0], [0.0, 0.0], [-1.0, 2.0]] {
                    for transparent in [false, true] {
                        let account = Account::for_test();
                        let closed = ClosedExecutableGraph::try_from_lowered(
                            plan(kind, sigma, transparent, format),
                            &account,
                        )
                        .unwrap()
                        .expect("actual complete native shadow chain closes");
                        assert!(closed.has_spatial_filters());
                        assert!(closed.facts.has_native_shadow);
                        assert!(
                            !closed.facts.has_gaussian,
                            "consumed alpha cannot become standalone Gaussian evidence"
                        );
                        assert!(
                            !closed.facts.has_offset,
                            "covered deferred offset cannot become standalone offset evidence"
                        );
                        assert!(
                            closed
                                .facts
                                .proves_exact_facts_for(&closed.lowered, &account)
                                .unwrap()
                        );
                        let mut forged = (*closed.facts).clone();
                        forged.has_native_shadow = false;
                        assert!(
                            !forged
                                .proves_exact_facts_for(&closed.lowered, &account)
                                .unwrap()
                        );
                    }
                }
            }
        }
    }
    #[test]
    fn native_shadow_closure_rejects_original_read_lifetime_mode_and_merge_mutations() {
        for kind in [0, 2] {
            let original = plan(kind, [4.1, 2.0], false, WorkingFormat::HighPrecision);
            let account = Account::for_test();
            let closed = ClosedExecutableGraph::try_from_lowered(original.clone(), &account)
                .unwrap()
                .unwrap();
            let merge = original
                .passes
                .iter()
                .position(|p| matches!(p.kind, RuntimePassKind::NativeShadowMerge(_)))
                .unwrap();
            let prepare=original.passes.iter().position(|p|matches!(&p.kind,RuntimePassKind::Gaussian(work) if work.stage()==GaussianStage::Prepare)).unwrap();
            let offset = original
                .passes
                .iter()
                .position(|p| matches!(p.kind, RuntimePassKind::Offset(_)))
                .unwrap();
            let raw = original.passes[merge].reads[0].resource;
            let shifted = original.passes[merge].reads[1].resource;
            for fault in 0..10 {
                let mut changed = original.clone();
                let mut passes = changed.passes.to_vec();
                match fault {
                    0 => {
                        let mut reads = passes[merge].reads.to_vec();
                        reads.remove(0);
                        passes[merge].reads = shared(&reads);
                    }
                    1 => {
                        let mut reads = passes[merge].reads.to_vec();
                        reads[0].resource = shifted;
                        passes[merge].reads = shared(&reads);
                    }
                    2 | 3 => {
                        let mut resources = changed.resources.to_vec();
                        let input = resources.iter_mut().find(|r| r.id == raw).unwrap();
                        if fault == 2 {
                            input.last_use = passes[prepare].id;
                        } else {
                            input.expected_reads = 1;
                        }
                        changed.resources = shared(&resources);
                    }
                    4 => {
                        let mut releases = passes[merge].releases.to_vec();
                        releases.retain(|id| *id != raw);
                        passes[merge].releases = shared(&releases);
                    }
                    5 => {
                        let mut releases = passes[prepare].releases.to_vec();
                        releases.push(raw);
                        passes[prepare].releases = shared(&releases);
                    }
                    6 => {
                        let RuntimePassKind::Gaussian(work) = &mut passes[prepare].kind else {
                            panic!();
                        };
                        *work = work
                            .with_purpose_for_test(GaussianPurpose::Standalone, &account)
                            .unwrap();
                    }
                    7 => {
                        let RuntimePassKind::Offset(work) = &mut passes[offset].kind else {
                            panic!();
                        };
                        *work = work.with_purpose_for_test(OffsetPurpose::Standalone);
                    }
                    8 => {
                        passes.remove(merge);
                    }
                    9 => {
                        let mut reads = passes[merge].reads.to_vec();
                        reads.swap(0, 1);
                        passes[merge].reads = shared(&reads);
                    }
                    _ => unreachable!(),
                }
                changed.passes = shared(&passes);
                assert!(
                    ClosedExecutableGraph::try_from_lowered(changed.clone(), &account)
                        .unwrap()
                        .is_err(),
                    "kind {kind}, mutation {fault}"
                );
                assert!(
                    !closed
                        .facts
                        .proves_exact_facts_for(&changed, &account)
                        .unwrap(),
                    "repeated kind {kind}, mutation {fault}"
                );
            }
        }
    }
    #[test]
    fn native_shadow_compositor_authenticates_footprint_presence_and_consumed_producer() {
        let original = plan(3, [1.0; 2], false, WorkingFormat::HighPrecision);
        let account = Account::for_test();
        let closed = ClosedExecutableGraph::try_from_lowered(original.clone(), &account)
            .unwrap()
            .unwrap();
        let native=original.passes.iter().position(|p|matches!(&p.kind,RuntimePassKind::Composite(Some(RuntimeComposite {kind:RuntimeCompositeKind::Layer{parameters,..},..})) if parameters.filter_footprint().is_some())).unwrap();
        let ordinary=original.passes.iter().rposition(|p|matches!(&p.kind,RuntimePassKind::Composite(Some(RuntimeComposite {kind:RuntimeCompositeKind::Layer{parameters,..},..})) if parameters.filter_footprint().is_none())).unwrap();
        let RuntimePassKind::Composite(Some(RuntimeComposite {
            kind: RuntimeCompositeKind::Layer { parameters, .. },
            ..
        })) = &original.passes[native].kind
        else {
            panic!();
        };
        let footprint = parameters.filter_footprint().unwrap();
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
        // A consumed alpha resolve is never an independently publishable bounded result.
        let resolve=original.passes.iter().find(|p|matches!(&p.kind,RuntimePassKind::Gaussian(work) if work.stage()==GaussianStage::Resolve)).unwrap();
        let RuntimeResultBinding::Resource(alpha) = resolve.result else {
            panic!();
        };
        let mut changed = original.clone();
        let mut passes = changed.passes.to_vec();
        let mut reads = passes[native].reads.to_vec();
        reads
            .iter_mut()
            .find(|r| r.role == RuntimeReadRole::CompositeSource)
            .unwrap()
            .resource = alpha;
        passes[native].reads = shared(&reads);
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
