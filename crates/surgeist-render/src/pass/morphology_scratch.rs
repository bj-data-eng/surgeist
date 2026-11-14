//! Morphology logical lifetimes mapped to a fixed set of real capacity textures.
use super::{
    close::preparation_error,
    model::{RuntimePassKind, RuntimeResourceId, RuntimeResourceProducer},
    prepare::{RuntimePassPreparationRequest, RuntimeResourcePreparationRequest},
};
#[cfg(test)]
use crate::resource::ResourceIdentity;
use crate::{
    PhysicalSize, Result,
    accounting::{Account, AccountedVec, RenderWork},
    backend::DeviceCapabilities,
    frame::MorphologyStage,
    resource::{FrameResourceScope, ResourceLease},
    texture::EffectTextureDescriptor,
};

#[cfg(test)]
thread_local! {
    static PADDING_CLEAR_FOR_TEST: std::cell::Cell<Option<wgpu::Color>> = const { std::cell::Cell::new(None) };
}

/// Scoped padding poison for actual morphology float attachments only.
#[cfg(test)]
pub(crate) struct MorphologyPaddingClearGuardForTest {
    previous: Option<wgpu::Color>,
    _same_thread: std::marker::PhantomData<std::rc::Rc<()>>,
}
#[cfg(test)]
impl MorphologyPaddingClearGuardForTest {
    pub(crate) fn install(color: wgpu::Color) -> Self {
        Self {
            previous: PADDING_CLEAR_FOR_TEST.with(|value| value.replace(Some(color))),
            _same_thread: std::marker::PhantomData,
        }
    }
}
#[cfg(test)]
impl Drop for MorphologyPaddingClearGuardForTest {
    fn drop(&mut self) {
        PADDING_CLEAR_FOR_TEST.with(|value| value.set(self.previous));
    }
}
#[cfg(test)]
pub(super) fn padding_clear_for_test() -> Option<wgpu::Color> {
    PADDING_CLEAR_FOR_TEST.with(std::cell::Cell::get)
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(super) struct ScratchAssignmentRef(usize);
impl ScratchAssignmentRef {
    pub(super) const fn new(index: usize) -> Self {
        Self(index)
    }
    pub(super) const fn index(self) -> usize {
        self.0
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(super) struct ScratchAssignment {
    pub(super) resource: RuntimeResourceId,
    pub(super) slot: usize,
    pub(super) producer: usize,
    pub(super) last_read: usize,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(super) struct ScratchSlot {
    pub(super) descriptor: EffectTextureDescriptor,
    last_read: usize,
}

#[derive(Debug, PartialEq)]
pub(super) struct MorphologyScratchPlan {
    pub(super) slots: AccountedVec<ScratchSlot>,
    pub(super) assignments: AccountedVec<ScratchAssignment>,
}

impl MorphologyScratchPlan {
    pub(super) fn try_new(
        resources: &[RuntimeResourcePreparationRequest],
        passes: &[RuntimePassPreparationRequest],
        account: &Account,
    ) -> Result<Self> {
        let mut result = Self {
            slots: account.retained(0)?,
            assignments: account.retained(0)?,
        };
        for (producer, pass) in passes.iter().enumerate() {
            account.work(RenderWork::GraphNodeVisits, 1)?;
            let RuntimePassKind::Morphology(work) = &pass.runtime.kind else {
                continue;
            };
            if !work.needs_float32() {
                continue;
            }
            let super::model::RuntimeResultBinding::Resource(id) = pass.runtime.result else {
                return Err(preparation_error("scratch producer has no logical result"));
            };
            let mut logical = None;
            for resource in resources {
                account.work(RenderWork::ResourceProbes, 1)?;
                if resource.runtime.id == id {
                    logical = Some(resource);
                    break;
                }
            }
            let logical =
                logical.ok_or_else(|| preparation_error("scratch logical result is missing"))?;
            if logical.runtime.producer != RuntimeResourceProducer::Pass(pass.runtime.id)
                || logical.runtime.format != super::model::RuntimeResourceFormat::FilterRgba32Float
                || logical.runtime.role != super::model::RuntimeResourceRole::FilterFloat
            {
                return Err(preparation_error(
                    "scratch result is not an exact morphology float resource",
                ));
            }
            let mut last_read = None;
            for (index, last) in passes.iter().enumerate() {
                account.work(RenderWork::ResourceProbes, 1)?;
                if last.runtime.id == logical.runtime.last_use {
                    last_read = Some(index);
                    break;
                }
            }
            let last_read =
                last_read.ok_or_else(|| preparation_error("scratch last reader is absent"))?;
            if last_read <= producer {
                return Err(preparation_error("scratch interval has no later reader"));
            }
            account.work(RenderWork::CoordinateVisits, 4)?;
            let (table, query) = work
                .capacity_extents()
                .ok_or_else(|| preparation_error("active scratch capacity is missing"))?;
            let extent = match work.stage() {
                MorphologyStage::Prepare | MorphologyStage::Table { .. } => table,
                MorphologyStage::FirstQuery { .. }
                | MorphologyStage::RefineQuery { .. }
                | MorphologyStage::Query { .. } => query,
                _ => {
                    return Err(preparation_error(
                        "non-extrema stage requested scratch capacity",
                    ));
                }
            };
            let descriptor = EffectTextureDescriptor::try_filter_float32(
                PhysicalSize::new(extent[0], extent[1]),
                scratch_usages(),
            )?;
            descriptor.checked_byte_len()?;
            let logical_extent = logical.runtime.spatial.device_extent;
            if logical_extent.width() > extent[0] || logical_extent.height() > extent[1] {
                return Err(preparation_error(
                    "scratch logical region exceeds its route capacity",
                ));
            }
            let mut selected = None;
            for (index, slot) in result.slots.iter().enumerate() {
                account.work(RenderWork::ResourceProbes, 1)?;
                if slot.descriptor == descriptor && slot.last_read < producer {
                    selected = Some(index);
                    break;
                }
            }
            let slot = match selected {
                Some(index) => {
                    result.slots[index].last_read = last_read;
                    index
                }
                None => {
                    let index = result.slots.len();
                    result.slots.push(ScratchSlot {
                        descriptor,
                        last_read,
                    })?;
                    index
                }
            };
            result.assignments.push(ScratchAssignment {
                resource: id,
                slot,
                producer,
                last_read,
            })?;
        }
        result.replay(passes, account)?;
        Ok(result)
    }

    pub(super) fn assignment_for(
        &self,
        resource: RuntimeResourceId,
        account: &Account,
    ) -> Result<Option<usize>> {
        for (index, assignment) in self.assignments.iter().enumerate() {
            account.work(RenderWork::ResourceProbes, 1)?;
            if assignment.resource == resource {
                return Ok(Some(index));
            }
        }
        Ok(None)
    }

    pub(super) fn proves_exact(
        &self,
        resources: &[RuntimeResourcePreparationRequest],
        passes: &[RuntimePassPreparationRequest],
        account: &Account,
    ) -> Result<bool> {
        let expected = Self::try_new(resources, passes, account)?;
        if self.slots.len() != expected.slots.len()
            || self.assignments.len() != expected.assignments.len()
        {
            return Ok(false);
        }
        for (actual, expected) in self.slots.iter().zip(&expected.slots) {
            account.work(RenderWork::ResourceProbes, 1)?;
            if actual != expected {
                return Ok(false);
            }
        }
        for (actual, expected) in self.assignments.iter().zip(&expected.assignments) {
            account.work(RenderWork::ResourceProbes, 1)?;
            if actual != expected {
                return Ok(false);
            }
        }
        self.replay(passes, account)?;
        Ok(true)
    }

    fn replay(&self, passes: &[RuntimePassPreparationRequest], account: &Account) -> Result<()> {
        let mut occupants = account.temporary(self.slots.len())?;
        for _ in &self.slots {
            occupants.append_slot()?.write(None::<usize>);
        }
        for (ordinal, pass) in passes.iter().enumerate() {
            account.work(RenderWork::GraphNodeVisits, 1)?;
            for read in &pass.runtime.reads {
                account.work(RenderWork::GraphEdgeVisits, 1)?;
                if let Some(index) = self.assignment_for(read.resource, account)? {
                    account.work(RenderWork::ResourceProbes, 2)?;
                    let assignment = self.assignments[index];
                    if occupants.get(assignment.slot) != Some(&Some(index))
                        || assignment.producer >= ordinal
                        || assignment.last_read < ordinal
                    {
                        return Err(preparation_error(
                            "scratch replay encountered a stale or retired logical read",
                        ));
                    }
                }
            }
            if let super::model::RuntimeResultBinding::Resource(resource) = pass.runtime.result
                && let Some(index) = self.assignment_for(resource, account)?
            {
                let assignment = self.assignments[index];
                let occupant = occupants
                    .get_mut(assignment.slot)
                    .ok_or_else(|| preparation_error("scratch replay slot is missing"))?;
                if assignment.producer != ordinal || occupant.is_some() {
                    return Err(preparation_error(
                        "scratch inclusive intervals alias at their producer",
                    ));
                }
                *occupant = Some(index);
            }
            for resource in &pass.runtime.releases {
                account.work(RenderWork::GraphEdgeVisits, 1)?;
                if let Some(index) = self.assignment_for(*resource, account)? {
                    account.work(RenderWork::ResourceProbes, 2)?;
                    let assignment = self.assignments[index];
                    let occupant = occupants
                        .get_mut(assignment.slot)
                        .ok_or_else(|| preparation_error("scratch retirement slot is missing"))?;
                    if assignment.last_read != ordinal || *occupant != Some(index) {
                        return Err(preparation_error(
                            "scratch retirement precedes its exact final read",
                        ));
                    }
                    *occupant = None;
                }
            }
        }
        for occupant in &occupants {
            account.work(RenderWork::ResourceProbes, 1)?;
            if occupant.is_some() {
                return Err(preparation_error(
                    "scratch replay leaves a live logical interval",
                ));
            }
        }
        Ok(())
    }
}

const fn scratch_usages() -> wgpu::TextureUsages {
    wgpu::TextureUsages::RENDER_ATTACHMENT
        .union(wgpu::TextureUsages::TEXTURE_BINDING)
        .union(wgpu::TextureUsages::COPY_SRC)
        .union(wgpu::TextureUsages::COPY_DST)
}

pub(super) struct PhysicalSlotBinding {
    pub(super) descriptor: EffectTextureDescriptor,
    pub(super) lease: ResourceLease,
    pub(super) occupant: Option<usize>,
    pub(super) resolved: bool,
}
#[derive(Clone, Copy, Default)]
pub(super) struct LogicalSlotState {
    pub(super) activated: bool,
    pub(super) retired: bool,
}
pub(super) struct MorphologyScratchBindings {
    pub(super) slots: AccountedVec<PhysicalSlotBinding>,
    pub(super) logical: AccountedVec<LogicalSlotState>,
}

impl MorphologyScratchBindings {
    pub(super) fn reserve(plan: &MorphologyScratchPlan, account: &Account) -> Result<Self> {
        Ok(Self {
            slots: account.retained(plan.slots.len())?,
            logical: account.retained(plan.assignments.len())?,
        })
    }
    pub(super) fn acquire(
        &mut self,
        plan: &MorphologyScratchPlan,
        frame: &mut FrameResourceScope,
        device: &wgpu::Device,
        capabilities: &DeviceCapabilities,
        account: &Account,
    ) -> Result<()> {
        for slot in &plan.slots {
            account.work(RenderWork::GraphNodeVisits, 1)?;
            let lease = frame.acquire_effect_texture(device, capabilities, slot.descriptor)?;
            self.slots.append_slot()?.write(PhysicalSlotBinding {
                descriptor: slot.descriptor,
                lease,
                occupant: None,
                resolved: false,
            });
        }
        for _ in &plan.assignments {
            account.work(RenderWork::GraphNodeVisits, 1)?;
            self.logical
                .append_slot()?
                .write(LogicalSlotState::default());
        }
        Ok(())
    }

    fn check_read(&self, plan: &MorphologyScratchPlan, index: usize, ordinal: usize) -> Result<()> {
        let assignment = plan
            .assignments
            .get(index)
            .ok_or_else(|| preparation_error("scratch assignment is missing"))?;
        let state = self
            .logical
            .get(index)
            .ok_or_else(|| preparation_error("scratch logical state is missing"))?;
        let slot = self
            .slots
            .get(assignment.slot)
            .ok_or_else(|| preparation_error("scratch physical slot is missing"))?;
        if plan
            .slots
            .get(assignment.slot)
            .map(|planned| planned.descriptor)
            != Some(slot.descriptor)
            || !state.activated
            || state.retired
            || slot.resolved
            || slot.occupant != Some(index)
            || ordinal < assignment.producer
            || ordinal > assignment.last_read
        {
            return Err(preparation_error(
                "scratch logical ID is stale, inactive or retired",
            ));
        }
        Ok(())
    }

    pub(super) fn validate_read(
        &self,
        plan: &MorphologyScratchPlan,
        index: usize,
        ordinal: usize,
        account: &Account,
    ) -> Result<()> {
        account.work(RenderWork::ResourceProbes, 4)?;
        self.check_read(plan, index, ordinal)
    }

    pub(super) fn validate_retirement(
        &self,
        plan: &MorphologyScratchPlan,
        index: usize,
        ordinal: usize,
        account: &Account,
    ) -> Result<()> {
        self.validate_read(plan, index, ordinal, account)?;
        account.work(RenderWork::ResourceProbes, 1)?;
        if plan.assignments[index].last_read != ordinal {
            return Err(preparation_error(
                "scratch occupant cannot retire before its final read",
            ));
        }
        Ok(())
    }

    pub(super) fn retire(
        &mut self,
        plan: &MorphologyScratchPlan,
        index: usize,
        ordinal: usize,
    ) -> Result<()> {
        self.logical
            .renderer_maintenance(RenderWork::ResourceProbes, 5);
        self.check_read(plan, index, ordinal)?;
        if plan.assignments[index].last_read != ordinal {
            return Err(preparation_error(
                "scratch logical retirement changed its exact ordinal",
            ));
        }
        self.logical[index].retired = true;
        self.slots[plan.assignments[index].slot].occupant = None;
        Ok(())
    }

    pub(super) fn resolve(
        &mut self,
        frame: &mut FrameResourceScope,
        account: &Account,
    ) -> Result<()> {
        for logical in &self.logical {
            account.work(RenderWork::ResourceProbes, 1)?;
            if !logical.activated || !logical.retired {
                return Err(preparation_error(
                    "scratch physical resolution has unfinished logical intervals",
                ));
            }
        }
        let mut leases = account.temporary(self.slots.len())?;
        for slot in self.slots.iter_mut() {
            account.work(RenderWork::ResourceProbes, 1)?;
            if slot.occupant.is_some() || slot.resolved {
                return Err(preparation_error(
                    "scratch physical resolution is live or duplicate",
                ));
            }
            leases.append_slot()?.write(&slot.lease);
        }
        frame.resolve_leases_atomically(&leases)?;
        drop(leases);
        for slot in self.slots.iter_mut() {
            slot.resolved = true;
        }
        Ok(())
    }
}

#[cfg(test)]
#[derive(Clone, Debug)]
pub(crate) struct MorphologyPhysicalSlotForTest {
    pub(crate) resource: ResourceIdentity,
    pub(crate) native_identity: u64,
    pub(crate) leased: bool,
    pub(crate) extent: PhysicalSize,
    pub(crate) format: wgpu::TextureFormat,
    pub(crate) usage: wgpu::TextureUsages,
    pub(crate) bytes: u64,
    pub(crate) occupant: Option<RuntimeResourceId>,
    pub(crate) resolved: bool,
}
#[cfg(test)]
#[derive(Clone, Debug)]
pub(crate) struct MorphologyLogicalAssignmentForTest {
    pub(crate) resource: RuntimeResourceId,
    pub(crate) slot: usize,
    pub(crate) producer_ordinal: usize,
    pub(crate) last_read_ordinal: usize,
    pub(crate) activated: bool,
    pub(crate) retired: bool,
}
#[cfg(test)]
#[derive(Clone, Debug)]
pub(crate) struct MorphologySlotObservationForTest {
    pub(crate) slots: Vec<MorphologyPhysicalSlotForTest>,
    pub(crate) assignments: Vec<MorphologyLogicalAssignmentForTest>,
}
#[cfg(test)]
impl MorphologyScratchBindings {
    pub(super) fn observe(
        &self,
        plan: &MorphologyScratchPlan,
        frame: &FrameResourceScope,
        account: &Account,
    ) -> Result<MorphologySlotObservationForTest> {
        let mut slots = Vec::new();
        for slot in &self.slots {
            account.work(RenderWork::ResourceProbes, 1)?;
            let native = frame.effect_texture_observation_for_test(&slot.lease)?;
            if native.resolved != slot.resolved {
                return Err(preparation_error(
                    "scratch physical resolution differs from its real provider",
                ));
            }
            slots.push(MorphologyPhysicalSlotForTest {
                resource: native.resource,
                native_identity: native.native_identity,
                leased: native.leased,
                extent: native.extent,
                format: native.format,
                usage: native.usage,
                bytes: native.bytes,
                occupant: slot.occupant.map(|index| plan.assignments[index].resource),
                resolved: slot.resolved,
            });
        }
        let mut assignments = Vec::new();
        for (index, assignment) in plan.assignments.iter().enumerate() {
            account.work(RenderWork::ResourceProbes, 1)?;
            assignments.push(MorphologyLogicalAssignmentForTest {
                resource: assignment.resource,
                slot: assignment.slot,
                producer_ordinal: assignment.producer,
                last_read_ordinal: assignment.last_read,
                activated: self.logical[index].activated,
                retired: self.logical[index].retired,
            });
        }
        Ok(MorphologySlotObservationForTest { slots, assignments })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        Capabilities, Color, FilterColorSpace, FilterDomains, Layer, MorphologyInput,
        MorphologyOperator, MorphologySource, Rect, RenderAccountingContext, Scene, Size,
        accounting::{GenerationClass, GpuObjectKind, RenderAccountingOptions, RendererCpu},
        backend::Backend,
        frame::{FrameContext, FramePlan},
        pass::{GraphExternalOutputView, LoweredGraphPlan},
        prepare_morphology,
        resource::WorkingFormat,
    };

    fn lower(
        chains: &[(u32, u32, bool)],
        capabilities: &DeviceCapabilities,
        account: &Account,
    ) -> LoweredGraphPlan {
        lower_with_height(chains, 1, 0.25, capabilities, account)
    }
    fn lower_with_height(
        chains: &[(u32, u32, bool)],
        height: u32,
        radius_y: f64,
        capabilities: &DeviceCapabilities,
        account: &Account,
    ) -> LoweredGraphPlan {
        let mut scene = Scene::new();
        let width = chains.iter().map(|chain| chain.0).max().unwrap();
        for &(input_width, output_width, periodic) in chains {
            let input_bounds = Rect::new(0.0, 0.0, f64::from(input_width), f64::from(height));
            let output_bounds = Rect::new(0.0, 0.0, f64::from(output_width), f64::from(height));
            let operation = prepare_morphology(
                MorphologyInput::new()
                    .with_operator(MorphologyOperator::Dilate)
                    .with_radii(f64::from(input_width / 2), radius_y),
                FilterColorSpace::Srgb,
                &mut RenderAccountingContext::default(),
            )
            .unwrap();
            let mut input = Scene::new();
            input.fill(input_bounds, Color::BLACK);
            let source = if periodic {
                MorphologySource::periodic(input, input_bounds, input_bounds, output_bounds)
                    .unwrap()
            } else {
                MorphologySource::finite(
                    input,
                    FilterDomains::try_new(input_bounds, input_bounds, output_bounds).unwrap(),
                )
            };
            scene
                .morphology_layer(Layer::new(), operation, source)
                .unwrap();
        }
        let commands = scene.normalize(Capabilities::CURRENT, account).unwrap();
        let context = FrameContext::try_new(
            Size::new(f64::from(width), f64::from(height)),
            1.0,
            crate::Antialiasing::Area,
            Color::TRANSPARENT,
        )
        .unwrap();
        let FramePlan::GpuGraph(graph) = commands.plan_for(context, account).unwrap() else {
            panic!("native morphology requires its graph");
        };
        LoweredGraphPlan::try_lower_validated_graph(
            &graph,
            WorkingFormat::HighPrecision,
            crate::Format::Rgba8,
            capabilities,
            account,
        )
        .unwrap()
    }

    async fn backend() -> (Backend, crate::backend::DeviceSlotIdentity) {
        let options = RenderAccountingOptions::new();
        let cpu = RendererCpu::new(options).unwrap();
        let mut backend = Backend::new(crate::ResourceCacheBudget::DISABLED, cpu, options).unwrap();
        let identity = backend
            .select_device(None)
            .await
            .unwrap()
            .expect("real morphology provider");
        (backend, identity)
    }

    #[test]
    fn real_native_slots_bound_many_levels_and_survive_an_unsubmitted_command_buffer() {
        pollster::block_on(async {
            let (mut backend, identity) = backend().await;
            let capabilities = backend.device_capabilities(identity).unwrap();
            for (chains, expected) in [
                (vec![(64, 64, true)], vec![64, 64]),
                (vec![(65, 65, false)], vec![65, 65, 65]),
                (vec![(65, 33, false)], vec![65, 33, 65, 33]),
                (vec![(64, 64, true), (64, 64, true)], vec![64, 64]),
                (vec![(64, 64, true), (32, 32, true)], vec![64, 64, 32, 32]),
            ] {
                let account = Account::for_test();
                let lowered = lower(&chains, &capabilities, &account);
                let transaction = backend
                    .begin_gpu_operation(
                        identity,
                        crate::gpu_transaction::GpuOperationStage::Render,
                        crate::RuntimeOperation::MorphologyRendering,
                    )
                    .unwrap();
                let mut prepared = backend
                    .prepare_graph_resources(
                        identity,
                        lowered,
                        crate::EffectQualityPolicy::RequireHighPrecision,
                        &account,
                    )
                    .unwrap();
                let before = prepared
                    .morphology_slot_observation_for_test(&account)
                    .unwrap();
                assert_eq!(
                    before
                        .slots
                        .iter()
                        .map(|slot| slot.extent.width())
                        .collect::<Vec<_>>(),
                    expected
                );
                assert_eq!(
                    before.slots.iter().map(|slot| slot.bytes).sum::<u64>(),
                    expected.iter().map(|width| u64::from(*width) * 16).sum()
                );
                assert!(before.assignments.len() > before.slots.len());
                for (index, assignment) in before.assignments.iter().enumerate() {
                    let raw = &prepared
                        .plan
                        .resources
                        .iter()
                        .find(|request| request.runtime.id == assignment.resource)
                        .unwrap()
                        .runtime;
                    let producer = &prepared.plan.passes[assignment.producer_ordinal].runtime;
                    let last_read = &prepared.plan.passes[assignment.last_read_ordinal].runtime;
                    assert_eq!(raw.producer, RuntimeResourceProducer::Pass(producer.id));
                    assert_eq!(
                        producer.result,
                        super::super::model::RuntimeResultBinding::Resource(assignment.resource)
                    );
                    assert_eq!(raw.last_use, last_read.id);
                    assert!(
                        last_read
                            .reads
                            .iter()
                            .any(|read| read.resource == assignment.resource)
                    );
                    assert!(assignment.producer_ordinal < assignment.last_read_ordinal);
                    let slot = &before.slots[assignment.slot];
                    assert!(
                        raw.spatial.device_extent.width() <= slot.extent.width()
                            && raw.spatial.device_extent.height() <= slot.extent.height()
                    );
                    let binding = prepared
                        .acquired_bindings
                        .runtime_bindings
                        .get(&assignment.resource, &account)
                        .unwrap()
                        .unwrap();
                    let super::super::prepare::PreparedResourceBacking::MorphologyScratch {
                        assignment: bound,
                    } = &binding.backing
                    else {
                        panic!("observed logical resource requires its actual slot binding");
                    };
                    let actual_slot =
                        prepared.plan.morphology_scratch.assignments[bound.index()].slot;
                    assert_eq!(actual_slot, assignment.slot);
                    assert_eq!(
                        prepared.acquired_bindings.morphology_scratch_bindings.slots[actual_slot]
                            .lease
                            .resource_identity(),
                        slot.resource
                    );
                    for previous in &before.assignments[..index] {
                        if previous.slot == assignment.slot {
                            assert!(
                                previous.last_read_ordinal < assignment.producer_ordinal
                                    || assignment.last_read_ordinal < previous.producer_ordinal,
                                "inclusive logical intervals sharing one actual texture must be strictly disjoint"
                            );
                        }
                    }
                }
                assert!(before.slots.iter().all(|slot| slot.leased
                    && !slot.resolved
                    && slot.occupant.is_none()
                    && slot.extent.height() == 1
                    && slot.format == wgpu::TextureFormat::Rgba32Float
                    && slot.usage == scratch_usages()));
                for (index, slot) in before.slots.iter().enumerate() {
                    assert!(
                        before.slots[..index]
                            .iter()
                            .all(|other| other.resource != slot.resource
                                && other.native_identity != slot.native_identity)
                    );
                }
                let provider_before = prepared.resources.observation_for_test();
                let native_float = provider_before
                    .effect_textures
                    .iter()
                    .filter(|texture| texture.format == wgpu::TextureFormat::Rgba32Float)
                    .collect::<Vec<_>>();
                assert_eq!(native_float.len(), before.slots.len());
                for slot in &before.slots {
                    assert!(
                        native_float
                            .iter()
                            .any(|native| native.resource == slot.resource
                                && native.native_identity == slot.native_identity
                                && native.bytes == slot.bytes)
                    );
                }
                let extent = prepared.output_extent(&account).unwrap();
                let generation = prepared.resources.generation();
                let output = generation
                    .allocation(
                        GenerationClass::SurfaceTargets,
                        u64::from(extent.width()) * u64::from(extent.height()) * 4,
                        Some(&account),
                        || {
                            prepared.device.create_texture(&wgpu::TextureDescriptor {
                                label: Some("morphology scratch unsubmitted output"),
                                size: wgpu::Extent3d {
                                    width: extent.width(),
                                    height: extent.height(),
                                    depth_or_array_layers: 1,
                                },
                                mip_level_count: 1,
                                sample_count: 1,
                                dimension: wgpu::TextureDimension::D2,
                                format: wgpu::TextureFormat::Rgba8Unorm,
                                usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
                                view_formats: &[],
                            })
                        },
                    )
                    .unwrap();
                let view = output
                    .create_view(&wgpu::TextureViewDescriptor::default(), Some(&account))
                    .unwrap();
                let mut encoder = generation
                    .opaque(GpuObjectKind::CommandEncoder, Some(&account), || {
                        prepared
                            .device
                            .create_command_encoder(&wgpu::CommandEncoderDescriptor::default())
                    })
                    .unwrap();
                let pending = prepared
                    .encode_custom_spine(
                        &mut encoder,
                        GraphExternalOutputView::try_new(&view, crate::Format::Rgba8, extent)
                            .unwrap(),
                        &account,
                    )
                    .await
                    .unwrap();
                let command_buffer = encoder.finish(Some(&account)).unwrap();
                let after = prepared
                    .morphology_slot_observation_for_test(&account)
                    .unwrap();
                assert!(
                    after
                        .slots
                        .iter()
                        .all(|slot| slot.resolved && slot.leased && slot.occupant.is_none())
                );
                assert!(
                    after
                        .assignments
                        .iter()
                        .all(|assignment| assignment.activated && assignment.retired)
                );
                assert_eq!(
                    before
                        .slots
                        .iter()
                        .map(|slot| (slot.resource, slot.native_identity, slot.bytes))
                        .collect::<Vec<_>>(),
                    after
                        .slots
                        .iter()
                        .map(|slot| (slot.resource, slot.native_identity, slot.bytes))
                        .collect::<Vec<_>>()
                );
                assert_eq!(
                    prepared.resources.observation_for_test().effects_gpu_bytes,
                    provider_before.effects_gpu_bytes
                );
                assert!(
                    prepared
                        .acquired_bindings
                        .morphology_scratch_bindings
                        .resolve(prepared.frame_scope.as_mut().unwrap(), &account)
                        .is_err(),
                    "physical resolution is exactly once"
                );
                // The command buffer is deliberately still unsubmitted at every observation.
                drop(command_buffer);
                drop(pending);
                drop(prepared);
                drop((view, output));
                transaction
                    .finish(crate::RuntimeOperation::MorphologyRendering, Some(&account))
                    .await
                    .unwrap();
                let manager = backend
                    .ready_device_state_borrow_for_test(identity)
                    .unwrap()
                    .internal_resource_manager_observation_for_test();
                assert_eq!((manager.active_frame_count, manager.leased_count), (0, 0));
            }
        });
    }

    #[test]
    fn scratch_exact_replay_rejects_overlap_capacity_format_usage_and_inherited_y_retirement() {
        pollster::block_on(async {
            let (mut backend, identity) = backend().await;
            let capabilities = backend.device_capabilities(identity).unwrap();
            let account = Account::for_test();
            let lowered = lower(&[(65, 65, false)], &capabilities, &account);
            let prepared = backend
                .prepare_graph_resources(
                    identity,
                    lowered,
                    crate::EffectQualityPolicy::RequireHighPrecision,
                    &account,
                )
                .unwrap();
            for alteration in 0..4 {
                let mut plan = MorphologyScratchPlan::try_new(
                    &prepared.plan.resources,
                    &prepared.plan.passes,
                    &account,
                )
                .unwrap();
                match alteration {
                    0 => plan.assignments[1].slot = plan.assignments[0].slot,
                    1 => {
                        plan.slots[0].descriptor = EffectTextureDescriptor::try_filter_float32(
                            PhysicalSize::new(1, 1),
                            scratch_usages(),
                        )
                        .unwrap()
                    }
                    2 => {
                        plan.slots[0].descriptor = EffectTextureDescriptor::try_working(
                            WorkingFormat::HighPrecision,
                            PhysicalSize::new(65, 1),
                            scratch_usages(),
                        )
                        .unwrap()
                    }
                    3 => {
                        plan.slots[0].descriptor = EffectTextureDescriptor::try_filter_float32(
                            PhysicalSize::new(65, 1),
                            wgpu::TextureUsages::TEXTURE_BINDING,
                        )
                        .unwrap()
                    }
                    _ => unreachable!(),
                }
                assert!(
                    !plan
                        .proves_exact(&prepared.plan.resources, &prepared.plan.passes, &account)
                        .unwrap(),
                    "alteration {alteration}"
                );
                if alteration == 0 {
                    assert!(plan.replay(&prepared.plan.passes, &account).is_err());
                }
            }
            drop(prepared);
            let lowered = lower_with_height(&[(9, 9, false)], 9, 2.0, &capabilities, &account);
            let prepared = backend
                .prepare_graph_resources(
                    identity,
                    lowered,
                    crate::EffectQualityPolicy::RequireHighPrecision,
                    &account,
                )
                .unwrap();
            let mut plan = MorphologyScratchPlan::try_new(
                &prepared.plan.resources,
                &prepared.plan.passes,
                &account,
            )
            .unwrap();
            let y = prepared.plan.passes.iter().enumerate().find(|(_,pass)| matches!(&pass.runtime.kind,
                RuntimePassKind::Morphology(work) if matches!(work.stage(), MorphologyStage::FirstQuery { axis: crate::frame::MorphologyAxis::Y }))).unwrap();
            let inherited = y.1.runtime.reads[0].resource;
            let index = plan.assignment_for(inherited, &account).unwrap().unwrap();
            assert!(
                plan.assignments[index].last_read > y.0,
                "Y table-zero survives its first query and next table build"
            );
            plan.assignments[index].last_read = y.0;
            assert!(plan.replay(&prepared.plan.passes, &account).is_err());
            assert!(
                !plan
                    .proves_exact(&prepared.plan.resources, &prepared.plan.passes, &account)
                    .unwrap()
            );
            drop(prepared);
        });
    }

    #[test]
    fn real_slot_occupancy_rejects_alias_stale_identity_capacity_and_false_physical_completion() {
        pollster::block_on(async {
            let (mut backend, identity) = backend().await;
            let capabilities = backend.device_capabilities(identity).unwrap();
            let account = Account::for_test();
            for alteration in 0..5 {
                let lowered = lower_with_height(&[(9, 9, false)], 9, 2.0, &capabilities, &account);
                let mut prepared = backend
                    .prepare_graph_resources(
                        identity,
                        lowered,
                        crate::EffectQualityPolicy::RequireHighPrecision,
                        &account,
                    )
                    .unwrap();
                let refine = prepared.plan.passes.iter().position(|pass| matches!(&pass.runtime.kind,
                    RuntimePassKind::Morphology(work) if matches!(work.stage(), MorphologyStage::RefineQuery { .. }))).unwrap();
                // This private state-owner harness performs no pixel rendering or publication.
                // Actual encoding and native command-buffer custody are exercised above.
                prepared.custom_spine_encoding_state = None;
                while prepared.next_pass < refine {
                    let id = prepared.plan.passes[prepared.next_pass].runtime.id;
                    prepared.activate_morphology_scratch(id, &account).unwrap();
                    prepared.complete_pass(id, &account).unwrap();
                }
                let pass = prepared.plan.passes[refine].runtime.clone();
                let super::super::model::RuntimeResultBinding::Resource(target) = pass.result
                else {
                    panic!();
                };
                let target_index = prepared
                    .plan
                    .morphology_scratch
                    .assignment_for(target, &account)
                    .unwrap()
                    .unwrap();
                let source_index = prepared
                    .plan
                    .morphology_scratch
                    .assignment_for(pass.reads[0].resource, &account)
                    .unwrap()
                    .unwrap();
                match alteration {
                    0 => {
                        prepared.plan.morphology_scratch.assignments[target_index].slot =
                            prepared.plan.morphology_scratch.assignments[source_index].slot;
                        assert!(
                            prepared
                                .activate_morphology_scratch(pass.id, &account)
                                .is_err(),
                            "source/table/previous and attachment require distinct active physical slots"
                        );
                    }
                    1 => {
                        let read = prepared
                            .acquired_bindings
                            .runtime_bindings
                            .get_mut(&pass.reads[0].resource, &account)
                            .unwrap()
                            .unwrap();
                        read.backing =
                            super::super::prepare::PreparedResourceBacking::MorphologyScratch {
                                assignment: ScratchAssignmentRef::new(target_index),
                            };
                        assert!(
                            prepared
                                .activate_morphology_scratch(pass.id, &account)
                                .is_err(),
                            "a slot reference cannot replace the exact logical resource identity"
                        );
                    }
                    2 => {
                        prepared
                            .activate_morphology_scratch(pass.id, &account)
                            .unwrap();
                        let old = prepared
                            .plan
                            .morphology_scratch
                            .assignments
                            .iter()
                            .enumerate()
                            .find(|(index, assignment)| {
                                *index != target_index
                                    && assignment.slot
                                        == prepared.plan.morphology_scratch.assignments
                                            [target_index]
                                            .slot
                                    && assignment.last_read < refine
                            })
                            .map(|(index, _)| index)
                            .unwrap();
                        assert!(
                            prepared
                                .acquired_bindings
                                .morphology_scratch_bindings
                                .validate_read(
                                    &prepared.plan.morphology_scratch,
                                    old,
                                    refine,
                                    &account
                                )
                                .is_err(),
                            "retired logical IDs stay stale after the physical slot is reassigned"
                        );
                    }
                    3 => {
                        prepared
                            .activate_morphology_scratch(pass.id, &account)
                            .unwrap();
                        let slot = prepared.plan.morphology_scratch.assignments[target_index].slot;
                        prepared.acquired_bindings.morphology_scratch_bindings.slots[slot]
                            .descriptor = EffectTextureDescriptor::try_filter_float32(
                            PhysicalSize::new(1, 1),
                            scratch_usages(),
                        )
                        .unwrap();
                        assert!(
                            prepared
                                .texture_binding_for_pass(pass.id, target, &account)
                                .is_err(),
                            "the actual capacity descriptor must equal the proved physical class"
                        );
                    }
                    4 => {
                        assert!(
                            prepared
                                .acquired_bindings
                                .morphology_scratch_bindings
                                .resolve(prepared.frame_scope.as_mut().unwrap(), &account)
                                .is_err(),
                            "physical completion cannot precede all logical retirements"
                        );
                        prepared.acquired_bindings.morphology_scratch_bindings.slots[0].resolved =
                            true;
                        assert!(
                            prepared
                                .morphology_slot_observation_for_test(&account)
                                .is_err(),
                            "omitting the real physical resolution cannot forge its completion flag"
                        );
                    }
                    _ => unreachable!(),
                }
                drop(prepared);
                let manager = backend
                    .ready_device_state_borrow_for_test(identity)
                    .unwrap()
                    .internal_resource_manager_observation_for_test();
                assert_eq!((manager.active_frame_count, manager.leased_count), (0, 0));
            }
        });
    }
}
