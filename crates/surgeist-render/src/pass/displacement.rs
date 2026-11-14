//! Closed displacement reads and their concrete, centrally owned allocations.
use super::{RuntimePassId, close::preparation_error};
use crate::{
    Result,
    accounting::{Account, RenderWork, SharedValue},
    frame::{
        DisplacementAddressEntry, DisplacementMapRecord, DisplacementPassFacts,
        DisplacementSourceKind, DisplacementStage,
    },
    image::RawImageUploadDescriptor,
    resource::{DisplacementBufferPlan, ResourceAllocationPreflight},
    shader::{
        DisplacementParameterBytes, DisplacementParameterFacts, ShaderDisplacementSourceKey,
        ShaderDisplacementStageKey, ShaderProgramKey,
    },
};

#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub(super) enum DisplacementReadRole {
    RawMap,
    AddressTables,
    MapRecords,
}
#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub(super) struct DisplacementResourceId {
    pub(super) owner: RuntimePassId,
    pub(super) role: DisplacementReadRole,
}
#[derive(Clone, Debug, PartialEq)]
pub(super) enum DisplacementReadPayload {
    RawMap(RawImageUploadDescriptor),
    AddressTables(SharedValue<Vec<DisplacementAddressEntry>>),
    MapRecords(SharedValue<Vec<DisplacementMapRecord>>),
}
#[derive(Clone, Debug, PartialEq)]
pub(super) struct DisplacementRead {
    pub(super) id: DisplacementResourceId,
    pub(super) expected_reads: u32,
    pub(super) last_use: RuntimePassId,
    pub(super) payload: DisplacementReadPayload,
}
#[derive(Clone, Debug, PartialEq)]
pub(crate) struct RuntimeDisplacement {
    pub(super) work: DisplacementPassFacts,
    pub(super) reads: SharedValue<Vec<DisplacementRead>>,
}
impl RuntimeDisplacement {
    pub(super) fn new(
        work: DisplacementPassFacts,
        owner: RuntimePassId,
        account: &Account,
    ) -> Result<Self> {
        let count = usize::from(work.address_entries().is_some())
            + usize::from(work.map_upload().is_some())
            + usize::from(work.map_records().is_some());
        let mut reads = account.retained(count)?;
        let mut append = |role, payload| -> Result<()> {
            account.work(RenderWork::GraphEdgeVisits, 1)?;
            reads.append_slot()?.write(DisplacementRead {
                id: DisplacementResourceId { owner, role },
                expected_reads: 1,
                last_use: owner,
                payload,
            });
            Ok(())
        };
        if let Some(map) = work.map_upload() {
            append(
                DisplacementReadRole::RawMap,
                DisplacementReadPayload::RawMap(map.clone()),
            )?;
        }
        if let Some(tables) = work.address_entries() {
            append(
                DisplacementReadRole::AddressTables,
                DisplacementReadPayload::AddressTables(tables.clone()),
            )?;
        }
        if let Some(records) = work.map_records() {
            append(
                DisplacementReadRole::MapRecords,
                DisplacementReadPayload::MapRecords(records.clone()),
            )?;
        }
        let result = Self {
            work,
            reads: SharedValue::from_accounted(reads, &account.storage())?,
        };
        if !result.proves_exact(owner, account)? {
            return Err(preparation_error(
                "displacement resource facts do not match their graph pass",
            ));
        }
        Ok(result)
    }
    pub(super) fn proves_exact(&self, owner: RuntimePassId, account: &Account) -> Result<bool> {
        if !self.work.proves_exact(account)? {
            return Ok(false);
        }
        let active = self.work.stage() == DisplacementStage::Displace;
        let raw = active && !self.work.map_is_zero();
        if self.reads.len() != usize::from(active) + 2 * usize::from(raw) {
            return Ok(false);
        }
        let mut roles = [false; 3];
        for read in self.reads.iter() {
            account.work(RenderWork::GraphEdgeVisits, 1)?;
            if read.id.owner != owner || read.last_use != owner || read.expected_reads != 1 {
                return Ok(false);
            }
            let (slot, valid) = match (&read.id.role, &read.payload) {
                (DisplacementReadRole::RawMap, DisplacementReadPayload::RawMap(map)) => (
                    0,
                    raw && self
                        .work
                        .map_upload()
                        .is_some_and(|expected| expected.same_allocation(map)),
                ),
                (
                    DisplacementReadRole::AddressTables,
                    DisplacementReadPayload::AddressTables(tables),
                ) => (
                    1,
                    active
                        && self
                            .work
                            .address_entries()
                            .is_some_and(|expected| expected.same_allocation(tables)),
                ),
                (
                    DisplacementReadRole::MapRecords,
                    DisplacementReadPayload::MapRecords(records),
                ) => (
                    2,
                    raw && self
                        .work
                        .map_records()
                        .is_some_and(|expected| expected.same_allocation(records)),
                ),
                _ => return Ok(false),
            };
            if !valid || roles[slot] {
                return Ok(false);
            }
            roles[slot] = true;
        }
        Ok(roles == [raw, active, raw])
    }
}

pub(super) fn shader_program(work: &DisplacementPassFacts) -> ShaderProgramKey {
    let stage = match work.stage() {
        DisplacementStage::RealizeSource => ShaderDisplacementStageKey::RealizeSource,
        DisplacementStage::PassThrough => ShaderDisplacementStageKey::PassThrough,
        DisplacementStage::Displace => ShaderDisplacementStageKey::Displace,
        DisplacementStage::Transparent => ShaderDisplacementStageKey::Transparent,
    };
    let source = match work.mapping().source_kind() {
        DisplacementSourceKind::Image => ShaderDisplacementSourceKey::Image,
        DisplacementSourceKind::Constant => ShaderDisplacementSourceKey::Constant,
        DisplacementSourceKind::Periodic => ShaderDisplacementSourceKey::Periodic,
    };
    ShaderProgramKey::Displacement {
        stage,
        source,
        raw_map: work.map_upload().is_some(),
    }
}
pub(super) fn parameters(
    work: &DisplacementPassFacts,
    account: &Account,
) -> Result<DisplacementParameterBytes> {
    let map = work.mapping();
    let color = if map.source_kind() == DisplacementSourceKind::Constant
        && work.stage() != DisplacementStage::Transparent
    {
        account.work(RenderWork::CoordinateVisits, 3)?;
        let color = map.color();
        [
            color.r() * color.a(),
            color.g() * color.a(),
            color.b() * color.a(),
            color.a(),
        ]
    } else {
        [0.0; 4]
    };
    let ShaderProgramKey::Displacement {
        stage,
        source,
        raw_map,
    } = shader_program(work)
    else {
        return Err(crate::RenderAccountingError::Invariant.into());
    };
    let map_size = work.map_upload().map_or([0; 2], |upload| {
        [
            upload.physical_size().width(),
            upload.physical_size().height(),
        ]
    });
    DisplacementParameterBytes::try_new(
        DisplacementParameterFacts {
            source_size: map.source_extent(),
            target_size: map.target_extent(),
            source_base: map.source_base(),
            source_clip: map.source_clip(),
            output_clip: map.output_clip(),
            periodic_base: map.periodic_base(),
            periodic_step: map.periodic_step(),
            color,
            stage: stage.parameter_code(),
            source_kind: match source {
                ShaderDisplacementSourceKey::Image => 0,
                ShaderDisplacementSourceKey::Constant => 1,
                ShaderDisplacementSourceKey::Periodic => 2,
            },
            channels: work.operation().channels().map(|channel| channel.index()),
            raw_map,
            map_size,
        },
        account,
    )
}

#[derive(Clone, Debug, PartialEq)]
pub(super) enum DisplacementAllocation {
    RawMap(RawImageUploadDescriptor),
    Buffer(DisplacementBufferPlan),
}
impl DisplacementAllocation {
    pub(super) fn preflight(&self, account: &Account) -> Result<ResourceAllocationPreflight> {
        match self {
            Self::RawMap(map) => ResourceAllocationPreflight::raw_image(map, account),
            Self::Buffer(plan) => ResourceAllocationPreflight::displacement_buffer(plan),
        }
    }
}
#[derive(Clone, Debug, PartialEq)]
pub(super) struct PreparedDisplacementRead {
    pub(super) read: DisplacementRead,
    pub(super) allocation: DisplacementAllocation,
}
#[derive(Clone, Debug, PartialEq)]
pub(super) struct DisplacementPreparation {
    pub(super) parameters: DisplacementParameterBytes,
    pub(super) reads: SharedValue<Vec<PreparedDisplacementRead>>,
}

/// Qualifies actual immutable descriptor lengths without serializing or scanning
/// an address/record array. Required lowering and preparation share this owner.
pub(super) fn validate_device(
    work: &DisplacementPassFacts,
    capabilities: &crate::backend::DeviceCapabilities,
    limits: crate::resource::DisplacementBufferLimits,
    account: &Account,
) -> Result<()> {
    use crate::resource::DisplacementBufferKind;
    let active = work.stage() == DisplacementStage::Displace;
    let raw = work.map_upload().is_some();
    let bindings = if active { if raw { 5 } else { 4 } } else { 3 };
    let textures = u32::from(work.mapping().source_spatial().is_some()) + u32::from(raw);
    limits.validate_bindings(u32::from(active) + u32::from(raw), textures, bindings)?;
    if let Some(map) = work.map_upload() {
        account.work(RenderWork::GraphEdgeVisits, 1)?;
        capabilities.validate_displacement_map(map.physical_size())?;
        map.validate_upload_byte_len(map.bytes().len())?;
        map.upload_row_bytes()?;
    }
    if let Some(entries) = work.address_entries() {
        account.work(RenderWork::GraphEdgeVisits, 1)?;
        DisplacementBufferPlan::validate_layout(
            DisplacementBufferKind::AddressTables,
            entries.len(),
            limits,
        )?;
    }
    if let Some(records) = work.map_records() {
        account.work(RenderWork::GraphEdgeVisits, 1)?;
        DisplacementBufferPlan::validate_layout(
            DisplacementBufferKind::MapRecords,
            records.len(),
            limits,
        )?;
    }
    Ok(())
}
pub(super) fn prepare(
    runtime: &RuntimeDisplacement,
    owner: RuntimePassId,
    capabilities: &crate::backend::DeviceCapabilities,
    limits: &wgpu::Limits,
    account: &Account,
) -> Result<SharedValue<DisplacementPreparation>> {
    if !runtime.proves_exact(owner, account)? {
        return Err(preparation_error(
            "displacement resource reads are stale or changed",
        ));
    }
    validate_device(
        &runtime.work,
        capabilities,
        crate::resource::DisplacementBufferLimits::from_device_limits(limits),
        account,
    )?;
    let mut reads = account.retained(runtime.reads.len())?;
    for read in runtime.reads.iter() {
        account.work(RenderWork::GraphEdgeVisits, 1)?;
        let allocation = match &read.payload {
            DisplacementReadPayload::RawMap(map) => DisplacementAllocation::RawMap(map.clone()),
            DisplacementReadPayload::AddressTables(entries) => DisplacementAllocation::Buffer(
                DisplacementBufferPlan::address_tables(entries, account, limits)?,
            ),
            DisplacementReadPayload::MapRecords(records) => DisplacementAllocation::Buffer(
                DisplacementBufferPlan::map_records(records, account, limits)?,
            ),
        };
        reads.append_slot()?.write(PreparedDisplacementRead {
            read: read.clone(),
            allocation,
        });
    }
    SharedValue::new(
        DisplacementPreparation {
            parameters: parameters(&runtime.work, account)?,
            reads: SharedValue::from_accounted(reads, &account.storage())?,
        },
        &account.storage(),
    )
}

pub(super) struct DisplacementLeaseBinding {
    pub(super) read: PreparedDisplacementRead,
    pub(super) lease: Option<crate::resource::ResourceLease>,
}
pub(super) struct PreparedDisplacementBinding {
    pub(super) preparation: SharedValue<DisplacementPreparation>,
    pub(super) resources: crate::accounting::AccountedVec<DisplacementLeaseBinding>,
    pub(super) uniform: crate::accounting::OwnedGpu<wgpu::Buffer>,
    pub(super) consumed: bool,
}
#[cfg(test)]
impl Drop for PreparedDisplacementBinding {
    fn drop(&mut self) {
        super::displacement_observation::record_disposing(self);
    }
}
impl PreparedDisplacementBinding {
    pub(super) fn acquire(
        preparation: &SharedValue<DisplacementPreparation>,
        frame: &mut crate::resource::FrameResourceScope,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        capabilities: &crate::backend::DeviceCapabilities,
        generation: &crate::accounting::Generation,
        account: &Account,
    ) -> Result<Self> {
        let resources = account.retained(preparation.reads.len())?;
        let uniform = generation.allocation_with_upload(
            crate::accounting::GenerationClass::PassBuffers,
            256,
            256,
            Some(account),
            || {
                let buffer = device.create_buffer(&wgpu::BufferDescriptor {
                    label: Some("Surgeist retained displacement uniform"),
                    size: 256,
                    usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
                    mapped_at_creation: false,
                });
                queue.write_buffer(&buffer, 0, preparation.parameters.as_bytes());
                buffer
            },
        )?;
        let mut binding = Self {
            preparation: preparation.clone(),
            resources,
            uniform,
            consumed: false,
        };
        for read in preparation.reads.iter() {
            account.work(RenderWork::GraphEdgeVisits, 1)?;
            let lease = match &read.allocation {
                DisplacementAllocation::RawMap(map) => {
                    frame.acquire_raw_image_upload(device, queue, capabilities, map)?
                }
                DisplacementAllocation::Buffer(plan) => {
                    frame.acquire_displacement_buffer(device, queue, plan)?
                }
            };
            binding
                .resources
                .append_slot()?
                .write(DisplacementLeaseBinding {
                    read: read.clone(),
                    lease: Some(lease),
                });
        }
        Ok(binding)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        Capabilities, Color, DisplacementInput, DisplacementSource, FilterColorSpace,
        FilterDomains, Format, Layer, Rect, RenderAccountingContext, Scene, Size,
        backend::DeviceCapabilities,
        frame::{FrameContext, FramePlan},
        pass::{LoweredGraphPlan, RuntimePassKind},
        renderer::Antialiasing,
        resource::WorkingFormat,
    };
    fn shared<T: Clone>(values: &[T], account: &Account) -> SharedValue<Vec<T>> {
        let mut result = account.retained(values.len()).unwrap();
        result.extend_from_slice(values).unwrap();
        SharedValue::from_accounted(result, &account.storage()).unwrap()
    }
    fn graph(scale: f64, zero: bool) -> crate::accounting::OwnedBox<crate::frame::GpuRenderGraph> {
        let bounds = Rect::new(0.0, 0.0, 6.0, 2.0);
        let mut preparation = RenderAccountingContext::default();
        let operation = crate::prepare_displacement(
            DisplacementInput::new().with_scale(scale),
            FilterColorSpace::Srgb,
            &mut preparation,
        )
        .unwrap();
        let (size, pixels): (_, std::sync::Arc<[u8]>) = if zero {
            (Size::new(0.0, 1.0), std::sync::Arc::from([]))
        } else {
            (Size::new(1.0, 1.0), std::sync::Arc::from([255, 0, 0, 255]))
        };
        let map =
            crate::prepare_native_displacement_map(size, pixels, bounds, &mut preparation).unwrap();
        let mut input = Scene::new();
        input.fill(bounds, Color::BLACK);
        let source = DisplacementSource::finite(
            input,
            FilterDomains::try_new(bounds, bounds, bounds).unwrap(),
        );
        let mut scene = Scene::new();
        scene
            .displacement_layer(Layer::new(), operation, source, map)
            .unwrap();
        let account = Account::for_test();
        let commands = scene.normalize(Capabilities::CURRENT, &account).unwrap();
        let context = FrameContext::try_new(
            Size::new(6.0, 2.0),
            1.0,
            Antialiasing::Area,
            Color::TRANSPARENT,
        )
        .unwrap();
        let FramePlan::GpuGraph(graph) = commands.plan_for(context, &account).unwrap() else {
            panic!("displacement requires its native graph");
        };
        graph
    }
    fn plan() -> LoweredGraphPlan {
        let graph = graph(4.0, false);
        let account = Account::for_test();
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
    fn required_lowering_refuses_storage_one_under_while_classification_remains_device_independent()
    {
        let graph = graph(4.0, false);
        let account = Account::for_test();
        let limits = wgpu::Limits {
            max_storage_buffer_binding_size: 4095,
            ..wgpu::Limits::default()
        };
        let capabilities = DeviceCapabilities::from_test_facts(true, true, 4096)
            .with_displacement_buffer_limits_for_test(&limits);
        let error = LoweredGraphPlan::try_lower_validated_graph(
            &graph,
            WorkingFormat::HighPrecision,
            Format::Rgba8,
            &capabilities,
            &account,
        )
        .err()
        .unwrap();
        assert!(matches!(
            error.displacement_diagnostic(),
            Some(crate::DisplacementPreparationError::Numeric {
                phase: crate::DisplacementNumericPhase::TableLayout,
                value: 4096.0
            })
        ));
        assert!(
            LoweredGraphPlan::try_lower_for_dispatch_classification(
                &graph,
                WorkingFormat::HighPrecision,
                Format::Rgba8,
                &account
            )
            .is_ok()
        );
        assert_eq!(account.work_count(RenderWork::SerializedBytes).unwrap(), 0);
    }
    #[test]
    fn one_storage_binding_admits_zero_map_and_refuses_raw_map_without_serialization() {
        let account = Account::for_test();
        let limits = wgpu::Limits {
            max_storage_buffers_per_shader_stage: 1,
            ..wgpu::Limits::default()
        };
        let capabilities = DeviceCapabilities::from_test_facts(true, true, 4096)
            .with_displacement_buffer_limits_for_test(&limits);
        for zero in [true, false] {
            let graph = graph(4.0, zero);
            assert_eq!(
                LoweredGraphPlan::try_lower_validated_graph(
                    &graph,
                    WorkingFormat::HighPrecision,
                    Format::Rgba8,
                    &capabilities,
                    &account
                )
                .is_ok(),
                zero
            );
        }
        assert_eq!(account.work_count(RenderWork::SerializedBytes).unwrap(), 0);
    }
    #[test]
    fn bypass_needs_no_storage_binding_and_active_zero_map_still_needs_its_tables() {
        let account = Account::for_test();
        let limits = wgpu::Limits {
            max_storage_buffers_per_shader_stage: 0,
            ..wgpu::Limits::default()
        };
        let capabilities = DeviceCapabilities::from_test_facts(true, true, 4096)
            .with_displacement_buffer_limits_for_test(&limits);
        for scale in [0.0, 4.0] {
            let graph = graph(scale, true);
            assert_eq!(
                LoweredGraphPlan::try_lower_validated_graph(
                    &graph,
                    WorkingFormat::HighPrecision,
                    Format::Rgba8,
                    &capabilities,
                    &account
                )
                .is_ok(),
                scale == 0.0
            );
        }
        assert_eq!(account.work_count(RenderWork::SerializedBytes).unwrap(), 0);
    }
    #[test]
    fn closed_displacement_rejects_each_auxiliary_generation_read_count_last_use_and_role_tamper() {
        let original = plan();
        let account = Account::for_test();
        let closed =
            crate::pass::close::ClosedExecutableGraph::try_from_lowered(original.clone(), &account)
                .unwrap()
                .unwrap();
        let index = original.passes.iter().position(|pass| matches!(&pass.kind, RuntimePassKind::Displacement(runtime) if runtime.work.stage() == DisplacementStage::Displace)).unwrap();
        for read_index in 0..3 {
            for alteration in 0..4 {
                let mut changed = original.clone();
                let mut passes = changed.passes.to_vec();
                let RuntimePassKind::Displacement(runtime) = &mut passes[index].kind else {
                    panic!();
                };
                let mut reads = runtime.reads.to_vec();
                match alteration {
                    0 => {
                        reads[read_index].id.owner = RuntimePassId(
                            reads[read_index]
                                .id
                                .owner
                                .0
                                .stale_generation_for_test()
                                .unwrap(),
                        )
                    }
                    1 => reads[read_index].expected_reads = 2,
                    2 => reads[read_index].last_use = original.final_present,
                    3 => {
                        reads[read_index].id.role = match reads[read_index].id.role {
                            DisplacementReadRole::RawMap => DisplacementReadRole::MapRecords,
                            _ => DisplacementReadRole::RawMap,
                        }
                    }
                    _ => unreachable!(),
                }
                runtime.reads = shared(&reads, &account);
                changed.passes = shared(&passes, &account);
                assert!(
                    crate::pass::close::ClosedExecutableGraph::try_from_lowered(
                        changed.clone(),
                        &account
                    )
                    .unwrap()
                    .is_err(),
                    "read {read_index}, alteration {alteration}"
                );
                assert!(
                    !closed
                        .facts
                        .proves_exact_facts_for(&changed, &account)
                        .unwrap(),
                    "repeated read {read_index}, alteration {alteration}"
                );
            }
        }
    }
}
