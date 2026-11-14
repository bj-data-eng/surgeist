//! Scoped raw test evidence from actual owners, without retaining native handles.
use super::{
    PreparedGraph, RuntimePassId, displacement::DisplacementReadRole,
    prepare::AcquiredGraphBindings,
};
use crate::{
    accounting::{Generation, GenerationClass, GpuObjectKind, RenderResource},
    resource::ResourceIdentity,
};
use std::{
    cell::RefCell,
    hash::{Hash, Hasher},
    rc::Rc,
};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum DisplacementUniformStageForTest {
    Prepared,
    BeforeSubmit,
    Submitted,
    UniformDisposing,
}
#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct DisplacementUniformObservationForTest {
    pub(crate) pass: RuntimePassId,
    pub(crate) native_identity: u64,
    pub(crate) bytes: u64,
    pub(crate) usage: wgpu::BufferUsages,
    pub(crate) consumed: bool,
}
#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct DisplacementUniformSnapshotForTest {
    pub(crate) stage: DisplacementUniformStageForTest,
    pub(crate) pass_buffer_bytes: u64,
    pub(crate) uniforms: Vec<DisplacementUniformObservationForTest>,
    pub(crate) disposing: Option<DisplacementUniformDisposalForTest>,
    pub(crate) command_encoder_objects: u64,
    pub(crate) command_buffer_objects: u64,
}
#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct DisplacementUniformDisposalForTest {
    pub(crate) native_identity: u64,
    pub(crate) bytes: u64,
    pub(crate) usage: wgpu::BufferUsages,
    pub(crate) consumed: bool,
    pub(crate) declared_reads: usize,
    pub(crate) acquired_reads: Vec<DisplacementAcquiredReadForTest>,
}
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum DisplacementAcquiredReadRoleForTest {
    RawMap,
    AddressTables,
    MapRecords,
}
#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct DisplacementAcquiredReadForTest {
    pub(crate) role: DisplacementAcquiredReadRoleForTest,
    pub(crate) resource: ResourceIdentity,
    pub(crate) allocation_generation: u64,
}
type Observations = Rc<RefCell<Vec<DisplacementUniformSnapshotForTest>>>;
thread_local! { static OBSERVATIONS: RefCell<Option<Observations>> = const { RefCell::new(None) }; }

pub(crate) struct DisplacementUniformProbeForTest {
    observations: Observations,
    previous: Option<Observations>,
}
impl DisplacementUniformProbeForTest {
    pub(crate) fn new() -> Self {
        let observations = Rc::new(RefCell::new(Vec::new()));
        let previous = OBSERVATIONS.with_borrow_mut(|slot| slot.replace(Rc::clone(&observations)));
        Self {
            observations,
            previous,
        }
    }
    pub(crate) fn reader(&self) -> Rc<dyn Fn() -> Vec<super::DisplacementUniformSnapshotForTest>> {
        let observations = Rc::clone(&self.observations);
        Rc::new(move || observations.borrow().clone())
    }
}
impl Drop for DisplacementUniformProbeForTest {
    fn drop(&mut self) {
        OBSERVATIONS.with_borrow_mut(|slot| *slot = self.previous.take());
    }
}

pub(super) fn record(
    custody: Option<&AcquiredGraphBindings>,
    generation: &Generation,
    stage: DisplacementUniformStageForTest,
) {
    let Some(observations) = OBSERVATIONS.with_borrow(Clone::clone) else {
        return;
    };
    let mut uniforms = Vec::new();
    if let Some(custody) = custody {
        for (pass, binding) in custody.displacement_bindings.entries.iter() {
            let mut native = std::collections::hash_map::DefaultHasher::new();
            std::ops::Deref::deref(&binding.uniform).hash(&mut native);
            uniforms.push(DisplacementUniformObservationForTest {
                pass: *pass,
                native_identity: native.finish(),
                bytes: binding.uniform.size(),
                usage: binding.uniform.usage(),
                consumed: binding.consumed,
            });
        }
    }
    let pass_buffer_bytes = generation
        .class_usage(GenerationClass::PassBuffers)
        .expect("real pass-buffer custody remains observable")
        .live(RenderResource::GpuBytes);
    let (command_encoder_objects, command_buffer_objects) = command_objects(generation);
    observations
        .borrow_mut()
        .push(DisplacementUniformSnapshotForTest {
            stage,
            pass_buffer_bytes,
            uniforms,
            disposing: None,
            command_encoder_objects,
            command_buffer_objects,
        });
}
fn command_objects(generation: &Generation) -> (u64, u64) {
    let live = |kind| {
        generation
            .object_usage_for_test(kind)
            .expect("real command ownership remains observable")
            .live(RenderResource::OpaqueGpuObjects)
    };
    (
        live(GpuObjectKind::CommandEncoder),
        live(GpuObjectKind::CommandBuffer),
    )
}
pub(super) fn record_disposing(binding: &super::displacement::PreparedDisplacementBinding) {
    let Some(observations) = OBSERVATIONS.with_borrow(Clone::clone) else {
        return;
    };
    let generation = binding.uniform.generation();
    let mut native = std::collections::hash_map::DefaultHasher::new();
    std::ops::Deref::deref(&binding.uniform).hash(&mut native);
    let pass_buffer_bytes = generation
        .class_usage(GenerationClass::PassBuffers)
        .expect("real pass-buffer custody remains observable")
        .live(RenderResource::GpuBytes);
    let (command_encoder_objects, command_buffer_objects) = command_objects(generation);
    observations
        .borrow_mut()
        .push(DisplacementUniformSnapshotForTest {
            stage: DisplacementUniformStageForTest::UniformDisposing,
            pass_buffer_bytes,
            uniforms: Vec::new(),
            command_encoder_objects,
            command_buffer_objects,
            disposing: Some(DisplacementUniformDisposalForTest {
                native_identity: native.finish(),
                bytes: binding.uniform.size(),
                usage: binding.uniform.usage(),
                consumed: binding.consumed,
                declared_reads: binding.preparation.reads.len(),
                acquired_reads: binding
                    .resources
                    .iter()
                    .filter_map(|lease_binding| {
                        let token = lease_binding.lease.as_ref()?.token_for_test();
                        Some(DisplacementAcquiredReadForTest {
                            role: match lease_binding.read.read.id.role {
                                DisplacementReadRole::RawMap => {
                                    DisplacementAcquiredReadRoleForTest::RawMap
                                }
                                DisplacementReadRole::AddressTables => {
                                    DisplacementAcquiredReadRoleForTest::AddressTables
                                }
                                DisplacementReadRole::MapRecords => {
                                    DisplacementAcquiredReadRoleForTest::MapRecords
                                }
                            },
                            resource: token.resource_identity,
                            allocation_generation: token.allocation_generation.get_for_test(),
                        })
                    })
                    .collect(),
            }),
        });
}
impl PreparedGraph<'_> {
    pub(crate) fn observe_displacement_uniforms_for_test(&self) {
        record(
            Some(&self.acquired_bindings),
            self.resources.generation(),
            DisplacementUniformStageForTest::Prepared,
        );
    }
}
