//! Actual graph parameter backing and its one-way logical permission.
use super::prepare::{AcquiredGraphBindings, RuntimePassPreparationRequest};
use super::{close::preparation_error, model::RuntimePassId};
use crate::accounting::RenderWork;
use crate::{
    Result,
    accounting::{Account, Generation, GenerationClass, OwnedGpu},
};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(super) enum PassBufferRole {
    ColorOperation,
    Gaussian,
    Morphology,
    NativeShadow,
    Offset,
    Displacement,
    Convolution,
    Spatial,
    Composite,
    CssShadow,
    BlurEdge,
}
impl PassBufferRole {
    pub(super) const fn index(self) -> usize {
        self as usize
    }
    pub(super) const fn usage(self) -> wgpu::BufferUsages {
        let binding = match self {
            Self::ColorOperation => wgpu::BufferUsages::STORAGE,
            _ => wgpu::BufferUsages::UNIFORM,
        };
        binding.union(wgpu::BufferUsages::COPY_DST)
    }
    fn valid_byte_len(self, size: usize) -> bool {
        match self {
            Self::ColorOperation => size >= 16 && (size - 16).is_multiple_of(32),
            Self::Spatial => size == 48,
            Self::Composite => size == 128,
            Self::CssShadow => size == 32,
            Self::BlurEdge => size == 16,
            _ => size == 256,
        }
    }
}

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub(super) struct PassBufferRoles(u16);
impl PassBufferRoles {
    pub(super) fn insert(&mut self, role: PassBufferRole) {
        self.0 |= 1 << role.index();
    }
    pub(super) const fn contains(self, role: PassBufferRole) -> bool {
        self.0 & (1 << role.index()) != 0
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum Permission {
    Ready,
    Consumed,
}

pub(super) struct PassBufferCustody {
    buffer: OwnedGpu<wgpu::Buffer>,
    owner: RuntimePassId,
    role: PassBufferRole,
    permission: Permission,
}
impl PassBufferCustody {
    pub(super) fn acquire(
        owner: RuntimePassId,
        role: PassBufferRole,
        bytes: &[u8],
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        generation: &Generation,
        account: &Account,
    ) -> Result<Self> {
        let size = u64::try_from(bytes.len())
            .map_err(|_| preparation_error("parameter buffer length overflows"))?;
        if !role.valid_byte_len(bytes.len()) {
            return Err(preparation_error(
                "parameter buffer byte length differs from its role",
            ));
        }
        let buffer = generation.allocation_with_upload(
            GenerationClass::PassBuffers,
            size,
            bytes.len(),
            Some(account),
            || {
                device.create_buffer(&wgpu::BufferDescriptor {
                    label: Some("Surgeist graph checked parameter backing"),
                    size,
                    usage: role.usage(),
                    mapped_at_creation: false,
                })
            },
        )?;
        queue.write_buffer(&buffer, 0, bytes);
        Ok(Self {
            buffer,
            owner,
            role,
            permission: Permission::Ready,
        })
    }
    fn validate(
        &self,
        owner: RuntimePassId,
        role: PassBufferRole,
        bytes: usize,
        generation: &Generation,
        consumed: bool,
    ) -> Result<()> {
        if self.owner != owner
            || self.role != role
            || !role.valid_byte_len(bytes)
            || !self.buffer.generation().same_owner(generation)
            || self.buffer.size()
                != u64::try_from(bytes)
                    .map_err(|_| preparation_error("parameter buffer length overflows"))?
            || self.buffer.usage() != role.usage()
            || self.is_consumed() != consumed
        {
            return Err(preparation_error(
                "parameter buffer owner, descriptor, or permission differs",
            ));
        }
        Ok(())
    }
    pub(super) fn active(
        &self,
        owner: RuntimePassId,
        role: PassBufferRole,
        bytes: usize,
        generation: &Generation,
    ) -> Result<&wgpu::Buffer> {
        self.validate(owner, role, bytes, generation, false)?;
        Ok(&self.buffer)
    }
    pub(super) fn consume(
        &mut self,
        owner: RuntimePassId,
        role: PassBufferRole,
        bytes: usize,
        generation: &Generation,
    ) -> Result<()> {
        self.validate(owner, role, bytes, generation, false)?;
        self.permission = Permission::Consumed;
        Ok(())
    }
    pub(super) fn validate_consumed(
        &self,
        owner: RuntimePassId,
        role: PassBufferRole,
        bytes: usize,
        generation: &Generation,
    ) -> Result<()> {
        self.validate(owner, role, bytes, generation, true)
    }
    pub(super) const fn is_consumed(&self) -> bool {
        matches!(self.permission, Permission::Consumed)
    }
    #[cfg(test)]
    pub(super) const fn native_for_test(&self) -> &OwnedGpu<wgpu::Buffer> {
        &self.buffer
    }
}

#[cfg(all(test, not(target_arch = "wasm32")))]
impl Drop for PassBufferCustody {
    fn drop(&mut self) {
        super::parameter_custody_observation::record_disposing(self.owner, self.role, self);
    }
}

pub(super) struct PreparedPassBuffer<T> {
    pub(super) bytes: T,
    pub(super) buffer: PassBufferCustody,
}

impl AcquiredGraphBindings {
    pub(super) fn validate_active_parameters(
        &self,
        request: &RuntimePassPreparationRequest,
        generation: &Generation,
        account: &Account,
    ) -> Result<PassBufferRoles> {
        let roles = request.parameter_roles(account)?;
        macro_rules! check {
            ($field:ident, $payload:ident, $role:ident) => {
                match (
                    &request.$payload,
                    self.$field.get(&request.runtime.id, account)?,
                ) {
                    (Some(expected), Some(binding)) if binding.bytes == *expected => {
                        binding.buffer.active(
                            request.runtime.id,
                            PassBufferRole::$role,
                            expected.as_bytes().len(),
                            generation,
                        )?;
                    }
                    (None, None) => {}
                    _ => {
                        return Err(preparation_error(
                            "parameter binding differs from its canonical request",
                        ))
                    }
                }
            };
        }
        check!(offset_parameter_bindings, offset_parameters, Offset);
        check!(
            native_shadow_parameter_bindings,
            native_shadow_parameters,
            NativeShadow
        );
        check!(gaussian_parameter_bindings, gaussian_parameters, Gaussian);
        check!(
            morphology_parameter_bindings,
            morphology_parameters,
            Morphology
        );
        check!(spatial_parameter_bindings, spatial_uniform, Spatial);
        check!(
            composite_parameter_bindings,
            composite_parameters,
            Composite
        );
        check!(
            css_shadow_parameter_bindings,
            drop_shadow_parameters,
            CssShadow
        );
        check!(blur_edge_parameter_bindings, blur_edge_parameters, BlurEdge);
        check!(
            convolution_parameter_bindings,
            convolution_parameters,
            Convolution
        );
        match (
            &request.color_filter_operations,
            self.color_filter_operation_bindings
                .get(&request.runtime.id, account)?,
        ) {
            (Some(expected), Some(binding)) if binding.bytes.matches_in(expected, account)? => {
                binding.buffer.active(
                    request.runtime.id,
                    PassBufferRole::ColorOperation,
                    expected.as_bytes().len(),
                    generation,
                )?;
            }
            (None, None) => {}
            _ => {
                return Err(preparation_error(
                    "color operation binding differs from its canonical request",
                ));
            }
        }
        match (
            &request.displacement,
            self.displacement_bindings
                .get(&request.runtime.id, account)?,
        ) {
            (Some(expected), Some(binding)) if expected.same_allocation(&binding.preparation) => {
                binding.uniform.active(
                    request.runtime.id,
                    PassBufferRole::Displacement,
                    expected.parameters.as_bytes().len(),
                    generation,
                )?;
            }
            (None, None) => {}
            _ => {
                return Err(preparation_error(
                    "displacement binding differs from its canonical request",
                ));
            }
        }
        Ok(roles)
    }
    pub(super) fn consume_parameters(
        &mut self,
        pass: RuntimePassId,
        required: PassBufferRoles,
        generation: &Generation,
    ) -> Result<PassBufferRoles> {
        let mut consumed = PassBufferRoles::default();
        macro_rules! consume {
            ($field:ident, $role:ident) => {
                if required.contains(PassBufferRole::$role) {
                    let binding = self.$field.maintenance_get_mut(&pass).ok_or_else(|| {
                        preparation_error("parameter custody disappeared after atomic resolution")
                    })?;
                    binding.buffer.consume(
                        pass,
                        PassBufferRole::$role,
                        binding.bytes.as_bytes().len(),
                        generation,
                    )?;
                    consumed.insert(PassBufferRole::$role);
                }
            };
        }
        consume!(offset_parameter_bindings, Offset);
        consume!(native_shadow_parameter_bindings, NativeShadow);
        consume!(gaussian_parameter_bindings, Gaussian);
        consume!(morphology_parameter_bindings, Morphology);
        consume!(color_filter_operation_bindings, ColorOperation);
        consume!(spatial_parameter_bindings, Spatial);
        consume!(composite_parameter_bindings, Composite);
        consume!(css_shadow_parameter_bindings, CssShadow);
        consume!(blur_edge_parameter_bindings, BlurEdge);
        consume!(convolution_parameter_bindings, Convolution);
        if required.contains(PassBufferRole::Displacement) {
            let binding = self
                .displacement_bindings
                .maintenance_get_mut(&pass)
                .ok_or_else(|| {
                    preparation_error("displacement custody disappeared after atomic resolution")
                })?;
            binding.uniform.consume(
                pass,
                PassBufferRole::Displacement,
                binding.preparation.parameters.as_bytes().len(),
                generation,
            )?;
            consumed.insert(PassBufferRole::Displacement);
        }
        if consumed != required {
            return Err(preparation_error(
                "parameter consumption roles are incomplete",
            ));
        }
        Ok(consumed)
    }
    pub(super) fn validate_consumed_parameters(
        &self,
        generation: &Generation,
        account: &Account,
    ) -> Result<()> {
        macro_rules! check {
            ($field:ident, $role:ident) => {
                for (pass, binding) in self.$field.entries.iter() {
                    account.work(RenderWork::ResourceProbes, 1)?;
                    binding.buffer.validate_consumed(
                        *pass,
                        PassBufferRole::$role,
                        binding.bytes.as_bytes().len(),
                        generation,
                    )?;
                }
            };
        }
        check!(offset_parameter_bindings, Offset);
        check!(native_shadow_parameter_bindings, NativeShadow);
        check!(gaussian_parameter_bindings, Gaussian);
        check!(morphology_parameter_bindings, Morphology);
        check!(color_filter_operation_bindings, ColorOperation);
        check!(spatial_parameter_bindings, Spatial);
        check!(composite_parameter_bindings, Composite);
        check!(css_shadow_parameter_bindings, CssShadow);
        check!(blur_edge_parameter_bindings, BlurEdge);
        check!(convolution_parameter_bindings, Convolution);
        for (pass, binding) in self.displacement_bindings.entries.iter() {
            account.work(RenderWork::ResourceProbes, 1)?;
            binding.uniform.validate_consumed(
                *pass,
                PassBufferRole::Displacement,
                binding.preparation.parameters.as_bytes().len(),
                generation,
            )?;
            for resource in &binding.resources {
                account.work(RenderWork::ResourceProbes, 1)?;
                if resource.lease.is_some() {
                    return Err(preparation_error(
                        "displacement resource lease is unresolved",
                    ));
                }
            }
        }
        Ok(())
    }
}

#[cfg(all(test, not(target_arch = "wasm32")))]
mod tests {
    use super::*;
    use crate::{
        accounting::{RenderAccountingOptions, RenderResource, RendererCpu},
        backend::Backend,
        pass::test_support::lower_filter_proof_fixture,
        renderer::EffectQualityPolicy,
        renderer::ResourceCacheBudget,
        style::filter::FilterAmount,
        style::filter::FilterList,
        style::filter::FilterOp,
    };

    #[test]
    fn actual_parameter_owner_rejects_changed_payload_descriptors_owners_and_consumed_permission() {
        pollster::block_on(async {
            let options = RenderAccountingOptions::new();
            let mut backend = Backend::new(
                ResourceCacheBudget::DISABLED,
                RendererCpu::new(options).unwrap(),
                options,
            )
            .unwrap();
            let identity = backend.select_device(None).await.unwrap().unwrap();
            let filter = FilterList::try_ops(vec![FilterOp::brightness(
                FilterAmount::try_new(1.25).unwrap(),
            )])
            .unwrap();
            let graph = lower_filter_proof_fixture(vec![filter.clone(), filter]);
            let account = Account::for_test();
            let mut prepared = backend
                .prepare_graph_resources(
                    identity,
                    graph,
                    EffectQualityPolicy::AllowReducedPrecision,
                    &account,
                )
                .unwrap();
            let generation = prepared.resources.generation();
            let request = prepared
                .plan
                .passes
                .iter()
                .find(|p| p.color_filter_operations.is_some())
                .unwrap();
            let pass = request.runtime.id;
            let roles = prepared
                .acquired_bindings
                .validate_active_parameters(request, generation, &account)
                .unwrap();
            let before = generation
                .class_usage(GenerationClass::PassBuffers)
                .unwrap()
                .live(RenderResource::GpuBytes);

            // The two actual color payloads are equal; ownership still differs.
            let entries = &mut prepared
                .acquired_bindings
                .color_filter_operation_bindings
                .entries;
            let (first, rest) = entries.split_at_mut(1);
            assert_eq!(first[0].1.bytes, rest[0].1.bytes);
            std::mem::swap(&mut first[0].1.buffer, &mut rest[0].1.buffer);
            assert!(
                prepared
                    .acquired_bindings
                    .validate_active_parameters(request, generation, &account)
                    .is_err()
            );
            let entries = &mut prepared
                .acquired_bindings
                .color_filter_operation_bindings
                .entries;
            let (first, rest) = entries.split_at_mut(1);
            std::mem::swap(&mut first[0].1.buffer, &mut rest[0].1.buffer);

            let wrong_filter = FilterList::try_ops(vec![FilterOp::brightness(
                FilterAmount::try_new(2.0).unwrap(),
            )])
            .unwrap();
            let wrong_graph = lower_filter_proof_fixture(vec![wrong_filter]);
            let wrong_operations = wrong_graph
                .passes
                .iter()
                .find_map(|pass| {
                    if let super::super::model::RuntimePassKind::ColorFilter(Some(filter)) =
                        &pass.kind
                    {
                        Some(filter.operations())
                    } else {
                        None
                    }
                })
                .unwrap();
            let wrong_bytes =
                crate::shader::ColorFilterOperationBytes::try_from_runtime_operations_with_limits(
                    &account,
                    wrong_operations,
                    crate::shader::ColorFilterOperationBufferLimits::from_device_limits(
                        &prepared.device.limits(),
                    ),
                )
                .unwrap();
            let binding = prepared
                .acquired_bindings
                .color_filter_operation_bindings
                .get_mut(&pass, &account)
                .unwrap()
                .unwrap();
            let original_bytes = std::mem::replace(&mut binding.bytes, wrong_bytes);
            assert!(
                prepared
                    .acquired_bindings
                    .validate_active_parameters(request, generation, &account)
                    .is_err()
            );
            prepared
                .acquired_bindings
                .color_filter_operation_bindings
                .get_mut(&pass, &account)
                .unwrap()
                .unwrap()
                .bytes = original_bytes;

            // Both real allocations are 48 bytes in this same pass and native
            // generation. Swapping only their native wrappers leaves owner,
            // role and CPU payload certificates intact, but exact usages differ.
            {
                let bindings = &mut *prepared.acquired_bindings;
                let color = bindings
                    .color_filter_operation_bindings
                    .get_mut(&pass, &account)
                    .unwrap()
                    .unwrap();
                let spatial = bindings
                    .spatial_parameter_bindings
                    .get_mut(&pass, &account)
                    .unwrap()
                    .unwrap();
                assert_eq!(color.buffer.buffer.size(), spatial.buffer.buffer.size());
                std::mem::swap(&mut color.buffer.buffer, &mut spatial.buffer.buffer);
            }
            assert!(
                prepared
                    .acquired_bindings
                    .validate_active_parameters(request, generation, &account)
                    .is_err()
            );
            {
                let bindings = &mut *prepared.acquired_bindings;
                let color = bindings
                    .color_filter_operation_bindings
                    .get_mut(&pass, &account)
                    .unwrap()
                    .unwrap();
                let spatial = bindings
                    .spatial_parameter_bindings
                    .get_mut(&pass, &account)
                    .unwrap()
                    .unwrap();
                std::mem::swap(&mut color.buffer.buffer, &mut spatial.buffer.buffer);
            }
            prepared
                .acquired_bindings
                .validate_active_parameters(request, generation, &account)
                .unwrap();

            let missing = prepared
                .acquired_bindings
                .spatial_parameter_bindings
                .remove_with(&pass, &account)
                .unwrap()
                .unwrap();
            assert!(
                prepared
                    .acquired_bindings
                    .validate_active_parameters(request, generation, &account)
                    .is_err()
            );
            prepared
                .acquired_bindings
                .spatial_parameter_bindings
                .insert_with(pass, missing, &account)
                .unwrap();
            let other = Generation::new(options).unwrap();
            assert!(
                prepared
                    .acquired_bindings
                    .validate_active_parameters(request, &other, &account)
                    .is_err()
            );
            assert!(
                prepared
                    .acquired_bindings
                    .validate_consumed_parameters(generation, &account)
                    .is_err()
            );

            // A partial receipt cannot pass final validation; consuming one
            // role neither permits borrowing it again nor releases its bytes.
            let mut color_only = PassBufferRoles::default();
            color_only.insert(PassBufferRole::ColorOperation);
            let consumed = prepared
                .acquired_bindings
                .consume_parameters(pass, color_only, generation)
                .unwrap();
            assert_ne!(consumed, roles);
            assert!(
                prepared
                    .acquired_bindings
                    .validate_consumed_parameters(generation, &account)
                    .is_err()
            );
            assert!(
                prepared
                    .acquired_bindings
                    .consume_parameters(pass, color_only, generation)
                    .is_err()
            );
            let binding = prepared
                .acquired_bindings
                .color_filter_operation_bindings
                .get(&pass, &account)
                .unwrap()
                .unwrap();
            assert!(
                binding
                    .buffer
                    .active(pass, PassBufferRole::ColorOperation, 48, generation)
                    .is_err()
            );
            assert_eq!(
                generation
                    .class_usage(GenerationClass::PassBuffers)
                    .unwrap()
                    .live(RenderResource::GpuBytes),
                before
            );
            drop(prepared);
            assert_eq!(
                backend
                    .ready_device_state_borrow_for_test(identity)
                    .unwrap()
                    .generation_for_test()
                    .class_usage(GenerationClass::PassBuffers)
                    .unwrap()
                    .live(RenderResource::GpuBytes),
                0
            );
        });
    }
}
