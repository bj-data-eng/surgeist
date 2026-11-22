use super::*;
use crate::frame::TurbulencePassFacts;

impl ClosedGraphTraversal<'_, '_> {
    pub(super) fn visit_turbulence(
        &mut self,
        pass: &RuntimePass,
        work: &TurbulencePassFacts,
    ) -> Option<()> {
        let result = validate_pass(pass, work, self.plan.working_format, &self.resources)?;
        self.expected_resources.insert(result);
        self.proof.capture_storage_write(self.filter_steps.push(
            ExecutableFilterStepFacts::Turbulence {
                pass: pass.id,
                result,
            },
        ))?;
        self.proof
            .capture_storage_write(self.contexts.push(ExecutableCompositionContext {
                current: result,
                producer: pass.id,
                contains_captured_source: true,
            }))?;
        self.has_turbulence = true;
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
            let ExecutableFilterStepFacts::Turbulence { pass, result } = step else {
                continue;
            };
            let pass = plan.passes.get(*maps.pass_positions.get(pass)?)?;
            let RuntimePassKind::Turbulence(work) = &pass.kind else {
                return None;
            };
            if !proof.work(RenderWork::GraphNodeVisits, 1)
                || validate_pass(pass, work, plan.working_format, &maps.resource_by_id)? != *result
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
    work: &TurbulencePassFacts,
    working_format: WorkingFormat,
    resources: &ProofMap<'_, RuntimeResourceId, &RuntimeResourceRequest>,
) -> Option<RuntimeResourceId> {
    if !pass.reads.is_empty()
        || !pass.dependencies.is_empty()
        || !pass.releases.is_empty()
        || pass.cache_keys.is_none()
    {
        return None;
    }
    let RuntimeResultBinding::Resource(target) = pass.result else {
        return None;
    };
    let resource = resources.get(&target).copied()?;
    (resources.proof.work(RenderWork::CoordinateVisits, 1)
        && base_graph_resource_has_fixed_facts(
            resource,
            RuntimeResourceRole::FilterIntermediate,
            RuntimeResourceFormat::Working(working_format),
            RuntimeResourceProducer::Pass(pass.id),
        )
        && resource.spatial == RuntimeSpatialDescriptor::from_graph(work.target_plan())
        && resource.expected_reads == 1)
        .then_some(target)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        backend::DeviceCapabilities,
        capability::Capabilities,
        effect::{Turbulence, TurbulenceRegion},
        frame::{FrameContext, FramePlan},
        geometry::{Rect, Size},
        pass::model::RuntimePassCacheKeys,
        scene::{Layer, Scene},
    };

    fn lower(seed: f64, working_format: WorkingFormat, account: &Account) -> LoweredGraphPlan {
        let bounds = Rect::new(0.0, 0.0, 3.0, 2.0);
        let mut scene = Scene::new();
        scene
            .turbulence_layer(
                Layer::new(),
                Turbulence::try_new([0.125, 0.25], seed).unwrap(),
                TurbulenceRegion::try_new(bounds, bounds).unwrap(),
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
            panic!("turbulence requires the native graph");
        };
        LoweredGraphPlan::try_lower_validated_graph(
            &graph,
            working_format,
            Format::Rgba8,
            &DeviceCapabilities::from_test_facts(true, true, 4096),
            account,
        )
        .unwrap()
    }

    #[test]
    fn source_free_turbulence_closes_only_without_input_dependencies() {
        let account = Account::for_test();
        let mut lowered = lower(1.0, WorkingFormat::HighPrecision, &account);
        let original = ClosedExecutableGraph::try_from_lowered(lowered.clone(), &account)
            .unwrap()
            .expect("the exact source-free graph closes");
        assert!(matches!(
            original.facts.filter_steps.as_slice(),
            [ExecutableFilterStepFacts::Turbulence { .. }]
        ));

        let mut passes = account.retained(lowered.passes.len()).unwrap();
        let mut changed = false;
        for pass in &*lowered.passes {
            let mut pass = pass.clone();
            if matches!(pass.kind, RuntimePassKind::Turbulence(_)) {
                // An earlier, real producer still cannot be an input to a
                // source-free pass.
                pass.dependencies =
                    SharedValue::new(vec![lowered.passes[0].id], &account.storage()).unwrap();
                changed = true;
            }
            passes.append_slot().unwrap().write(pass);
        }
        assert!(changed);
        lowered.passes = SharedValue::from_accounted(passes, &account.storage()).unwrap();
        assert!(
            ClosedExecutableGraph::try_from_lowered(lowered, &account)
                .unwrap()
                .is_err()
        );
    }

    #[test]
    fn turbulence_cache_keys_share_one_program_across_seeds_for_each_working_format() {
        let account = Account::for_test();
        let first = lower(1.0, WorkingFormat::HighPrecision, &account);
        let other_seed = lower(7.0, WorkingFormat::HighPrecision, &account);
        let other_format = lower(1.0, WorkingFormat::ReducedPrecision, &account);
        fn turbulence_pass(
            plan: &LoweredGraphPlan,
        ) -> (&TurbulencePassFacts, &RuntimePassCacheKeys) {
            plan.passes
                .iter()
                .find_map(|pass| match &pass.kind {
                    RuntimePassKind::Turbulence(work) => Some((work, pass.cache_keys.as_ref()?)),
                    _ => None,
                })
                .expect("each lowered plan has turbulence keys")
        }
        let (first, first_keys) = turbulence_pass(&first);
        let (other_seed, other_seed_keys) = turbulence_pass(&other_seed);
        let (_, other_format_keys) = turbulence_pass(&other_format);
        assert_ne!(first.uniform_bytes(), other_seed.uniform_bytes());
        assert!(first_keys.matches_in(other_seed_keys, &account).unwrap());
        assert!(!first_keys.matches_in(other_format_keys, &account).unwrap());
    }
}
