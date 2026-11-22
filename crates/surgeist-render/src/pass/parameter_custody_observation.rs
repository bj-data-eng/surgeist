//! Generation-scoped observations of actual native parameter owners.
use super::{
    PreparedGraph, RuntimePassId,
    buffer_custody::{PassBufferCustody, PassBufferRole},
    prepare::AcquiredGraphBindings,
};
use crate::accounting::{
    Generation, GenerationClass, GpuObjectKind, RenderGenerationIdentity, RenderResource,
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
    Lighting,
    Turbulence,
    Offset,
    Displacement,
    Convolution,
    Spatial,
    Composite,
    CssShadow,
    BlurEdge,
}
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum ParameterStageForTest {
    Prepared,
    Encoded,
    BeforeSubmit,
    Submitted,
    AccountingReady,
    Disposing,
}
#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct ParameterBufferForTest {
    pub(crate) family: ParameterFamilyForTest,
    pub(crate) pass: RuntimePassId,
    pub(crate) native_identity: u64,
    pub(crate) bytes: u64,
    pub(crate) usage: wgpu::BufferUsages,
    pub(crate) consumed: bool,
}
#[derive(Clone, Debug)]
pub(crate) struct ParameterSnapshotForTest {
    pub(crate) stage: ParameterStageForTest,
    // None is used only when there is no aggregate at this exact boundary.
    pub(crate) buffers: Option<Vec<ParameterBufferForTest>>,
    pub(crate) disposing: Option<ParameterBufferForTest>,
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
    disposing: Option<ParameterBufferForTest>,
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
            disposing,
            pass_buffer_bytes: generation
                .class_usage(GenerationClass::PassBuffers)
                .unwrap()
                .live(RenderResource::GpuBytes),
            command_encoder_objects: objects(GpuObjectKind::CommandEncoder),
            command_buffer_objects: objects(GpuObjectKind::CommandBuffer),
        });
}
fn descriptor(
    family: ParameterFamilyForTest,
    pass: RuntimePassId,
    custody: &PassBufferCustody,
) -> ParameterBufferForTest {
    let buffer = custody.native_for_test();
    let mut identity = std::collections::hash_map::DefaultHasher::new();
    std::ops::Deref::deref(buffer).hash(&mut identity);
    ParameterBufferForTest {
        family,
        pass,
        native_identity: identity.finish(),
        bytes: buffer.size(),
        usage: buffer.usage(),
        consumed: custody.is_consumed(),
    }
}
pub(super) fn record_owner(
    custody: &AcquiredGraphBindings,
    generation: &Generation,
    stage: ParameterStageForTest,
) {
    let Some(observation) = observer(generation) else {
        return;
    };
    let mut buffers = Vec::new();
    macro_rules! append {
        ($field:ident, $family:ident) => {
            for (pass, binding) in custody.$field.entries.iter() {
                buffers.push(descriptor(
                    ParameterFamilyForTest::$family,
                    *pass,
                    &binding.buffer,
                ));
            }
        };
    }
    append!(color_filter_operation_bindings, ColorOperation);
    append!(gaussian_parameter_bindings, Gaussian);
    append!(morphology_parameter_bindings, Morphology);
    append!(native_shadow_parameter_bindings, NativeShadow);
    append!(lighting_parameter_bindings, Lighting);
    append!(turbulence_parameter_bindings, Turbulence);
    append!(offset_parameter_bindings, Offset);
    append!(convolution_parameter_bindings, Convolution);
    append!(spatial_parameter_bindings, Spatial);
    append!(composite_parameter_bindings, Composite);
    append!(css_shadow_parameter_bindings, CssShadow);
    append!(blur_edge_parameter_bindings, BlurEdge);
    for (pass, binding) in custody.displacement_bindings.entries.iter() {
        buffers.push(descriptor(
            ParameterFamilyForTest::Displacement,
            *pass,
            &binding.uniform,
        ));
    }
    record(&observation, generation, stage, Some(buffers), None);
}
pub(super) fn record_disposing(
    pass: RuntimePassId,
    role: PassBufferRole,
    custody: &PassBufferCustody,
) {
    let generation = custody.native_for_test().generation();
    let Some(observation) = observer(generation) else {
        return;
    };
    let family = match role {
        PassBufferRole::ColorOperation => ParameterFamilyForTest::ColorOperation,
        PassBufferRole::Gaussian => ParameterFamilyForTest::Gaussian,
        PassBufferRole::Morphology => ParameterFamilyForTest::Morphology,
        PassBufferRole::NativeShadow => ParameterFamilyForTest::NativeShadow,
        PassBufferRole::Lighting => ParameterFamilyForTest::Lighting,
        PassBufferRole::Turbulence => ParameterFamilyForTest::Turbulence,
        PassBufferRole::Offset => ParameterFamilyForTest::Offset,
        PassBufferRole::Displacement => ParameterFamilyForTest::Displacement,
        PassBufferRole::Convolution => ParameterFamilyForTest::Convolution,
        PassBufferRole::Spatial => ParameterFamilyForTest::Spatial,
        PassBufferRole::Composite => ParameterFamilyForTest::Composite,
        PassBufferRole::CssShadow => ParameterFamilyForTest::CssShadow,
        PassBufferRole::BlurEdge => ParameterFamilyForTest::BlurEdge,
    };
    record(
        &observation,
        generation,
        ParameterStageForTest::Disposing,
        None,
        Some(descriptor(family, pass, custody)),
    );
}
impl PreparedGraph<'_> {
    pub(crate) fn observe_parameter_custody_for_test(&self, stage: ParameterStageForTest) {
        record_owner(&self.acquired_bindings, self.resources.generation(), stage);
    }
}
