//! Generation-scoped observations of actual native parameter owners.
use super::{PreparedGraph, RuntimePassId};
use crate::accounting::{
    Generation, GenerationClass, GpuObjectKind, OwnedGpu, RenderGenerationIdentity, RenderResource,
};
use std::{
    cell::RefCell,
    hash::{Hash, Hasher},
    rc::Rc,
};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum ParameterFamilyForTest {
    ColorOperation,
    Gaussian,
    Morphology,
    NativeShadow,
    Offset,
}
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum ParameterStageForTest {
    Prepared,
    Encoded,
    BeforeSubmit,
    Submitted,
}
#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct ParameterBufferForTest {
    pub(crate) family: ParameterFamilyForTest,
    pub(crate) pass: RuntimePassId,
    pub(crate) native_identity: u64,
    pub(crate) bytes: u64,
    pub(crate) usage: wgpu::BufferUsages,
}
#[derive(Clone, Debug)]
pub(crate) struct ParameterSnapshotForTest {
    pub(crate) stage: ParameterStageForTest,
    // None means only generation custody is observable at this owner boundary.
    pub(crate) buffers: Option<Vec<ParameterBufferForTest>>,
    pub(crate) pass_buffer_bytes: u64,
    pub(crate) command_encoder_objects: u64,
    pub(crate) command_buffer_objects: u64,
}
struct Observation {
    target: RenderGenerationIdentity,
    snapshots: RefCell<Vec<ParameterSnapshotForTest>>,
}
thread_local! { static OBSERVER: RefCell<Option<Rc<Observation>>> = const { RefCell::new(None) }; }
pub(crate) struct ParameterProbeForTest {
    observation: Rc<Observation>,
    previous: Option<Rc<Observation>>,
}
impl ParameterProbeForTest {
    pub(crate) fn new(generation: &Generation) -> Self {
        let observation = Rc::new(Observation {
            target: generation.identity_for_test(),
            snapshots: RefCell::new(Vec::new()),
        });
        let previous = OBSERVER.with_borrow_mut(|slot| slot.replace(Rc::clone(&observation)));
        Self {
            observation,
            previous,
        }
    }
    pub(crate) fn reader(&self) -> Rc<dyn Fn() -> Vec<ParameterSnapshotForTest>> {
        let observation = Rc::clone(&self.observation);
        Rc::new(move || observation.snapshots.borrow().clone())
    }
}
impl Drop for ParameterProbeForTest {
    fn drop(&mut self) {
        OBSERVER.with_borrow_mut(|slot| *slot = self.previous.take());
    }
}
fn observer(generation: &Generation) -> Option<Rc<Observation>> {
    OBSERVER
        .with_borrow(Clone::clone)
        .filter(|o| o.target == generation.identity_for_test())
}
fn record(
    observation: &Observation,
    generation: &Generation,
    stage: ParameterStageForTest,
    buffers: Option<Vec<ParameterBufferForTest>>,
) {
    let objects = |kind| {
        generation
            .object_usage_for_test(kind)
            .unwrap()
            .live(RenderResource::OpaqueGpuObjects)
    };
    observation
        .snapshots
        .borrow_mut()
        .push(ParameterSnapshotForTest {
            stage,
            buffers,
            pass_buffer_bytes: generation
                .class_usage(GenerationClass::PassBuffers)
                .unwrap()
                .live(RenderResource::GpuBytes),
            command_encoder_objects: objects(GpuObjectKind::CommandEncoder),
            command_buffer_objects: objects(GpuObjectKind::CommandBuffer),
        });
}
pub(super) fn record_scope(generation: &Generation, stage: ParameterStageForTest) {
    if let Some(observation) = observer(generation) {
        record(&observation, generation, stage, None);
    }
}
fn append(
    output: &mut Vec<ParameterBufferForTest>,
    family: ParameterFamilyForTest,
    pass: RuntimePassId,
    buffer: &Option<OwnedGpu<wgpu::Buffer>>,
) {
    if let Some(buffer) = buffer {
        let mut identity = std::collections::hash_map::DefaultHasher::new();
        std::ops::Deref::deref(buffer).hash(&mut identity);
        output.push(ParameterBufferForTest {
            family,
            pass,
            native_identity: identity.finish(),
            bytes: buffer.size(),
            usage: buffer.usage(),
        });
    }
}
impl PreparedGraph<'_> {
    pub(crate) fn observe_parameter_custody_for_test(&self, stage: ParameterStageForTest) {
        let generation = self.resources.generation();
        let Some(observation) = observer(generation) else {
            return;
        };
        let mut buffers = Vec::new();
        for (pass, binding) in self.color_filter_operation_bindings.entries.iter() {
            append(
                &mut buffers,
                ParameterFamilyForTest::ColorOperation,
                *pass,
                &binding.buffer,
            );
        }
        for (pass, binding) in self.gaussian_parameter_bindings.entries.iter() {
            append(
                &mut buffers,
                ParameterFamilyForTest::Gaussian,
                *pass,
                &binding.buffer,
            );
        }
        for (pass, binding) in self.morphology_parameter_bindings.entries.iter() {
            append(
                &mut buffers,
                ParameterFamilyForTest::Morphology,
                *pass,
                &binding.buffer,
            );
        }
        for (pass, binding) in self.native_shadow_parameter_bindings.entries.iter() {
            append(
                &mut buffers,
                ParameterFamilyForTest::NativeShadow,
                *pass,
                &binding.buffer,
            );
        }
        for (pass, binding) in self.offset_parameter_bindings.entries.iter() {
            append(
                &mut buffers,
                ParameterFamilyForTest::Offset,
                *pass,
                &binding.buffer,
            );
        }
        record(&observation, generation, stage, Some(buffers));
    }
}
