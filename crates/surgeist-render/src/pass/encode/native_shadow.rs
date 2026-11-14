//! The fused native shadow merge binds only the real original/shifted branches.
use super::*;
use crate::frame::NativeShadowOriginal;

impl PreparedGraph<'_> {
    pub(super) fn encode_native_shadow(
        &self,
        encoder: &mut wgpu::CommandEncoder,
        request: &PreparedPassEncodingRequest,
        account: &Account,
    ) -> Result<()> {
        let RuntimePassKind::NativeShadowMerge(work) = &request.kind else {
            return Err(preparation_error(
                "native shadow merge requires its admitted payload",
            ));
        };
        let map = work.mapping();
        let original_required = map.original_kind() != NativeShadowOriginal::Constant;
        if request.reads.len()
            != usize::from(original_required) + usize::from(map.has_shifted_alpha())
        {
            return Err(preparation_error(
                "native shadow merge has an unexpected sampled branch",
            ));
        }
        let RuntimeResultBinding::Resource(target) = request.result else {
            return Err(preparation_error(
                "native shadow merge requires a concrete working target",
            ));
        };
        let target_binding = self.texture_binding_for_pass(request.id, target, account)?;
        let target_spatial = self.validate_texture_binding(&target_binding, target, account)?;
        account.work(RenderWork::CoordinateVisits, 1)?;
        if target_binding.texture().format() != self.plan.working_format.texture_format()
            || !target_binding
                .texture()
                .usage()
                .contains(wgpu::TextureUsages::RENDER_ATTACHMENT)
            || target_spatial != RuntimeSpatialDescriptor::from_graph(map.target_spatial())
        {
            return Err(preparation_error(
                "native shadow target differs from its qualified working lattice",
            ));
        }
        let original_binding = if original_required {
            let read =
                exact_core_pass_read(request, RuntimeReadRole::NativeShadowOriginal, account)?;
            let binding = self.texture_binding_for_pass(request.id, read.resource, account)?;
            let spatial = self.validate_texture_binding(&binding, read.resource, account)?;
            if read.resource == target
                || binding.allocation_resource() == target_binding.allocation_resource()
                || binding.texture().format() != self.plan.working_format.texture_format()
                || !binding
                    .texture()
                    .usage()
                    .contains(wgpu::TextureUsages::TEXTURE_BINDING)
                || Some(spatial)
                    != map
                        .original_mapping()
                        .source_spatial()
                        .map(RuntimeSpatialDescriptor::from_graph)
            {
                return Err(preparation_error(
                    "native shadow original is not the retained raw working lattice",
                ));
            }
            Some(binding)
        } else {
            None
        };
        let alpha_binding = if map.has_shifted_alpha() {
            let read = exact_core_pass_read(request, RuntimeReadRole::NativeShadowAlpha, account)?;
            let binding = self.texture_binding_for_pass(request.id, read.resource, account)?;
            let spatial = self.validate_texture_binding(&binding, read.resource, account)?;
            if read.resource == target
                || binding.allocation_resource() == target_binding.allocation_resource()
                || original_binding.as_ref().is_some_and(|original| {
                    original.allocation_resource() == binding.allocation_resource()
                })
                || binding.texture().format() != self.plan.working_format.texture_format()
                || !binding
                    .texture()
                    .usage()
                    .contains(wgpu::TextureUsages::TEXTURE_BINDING)
                || spatial != target_spatial
            {
                return Err(preparation_error(
                    "native shadow shifted alpha differs from its exact merge lattice",
                ));
            }
            Some(binding)
        } else {
            None
        };
        let parameters = request
            .native_shadow_parameters
            .as_ref()
            .ok_or_else(|| preparation_error("native shadow has no serialized merge parameters"))?;
        let expected = super::super::native_shadow::parameters(work, account)?;
        account.work(RenderWork::ResourceProbes, 1)?;
        if *parameters != expected {
            return Err(preparation_error(
                "native shadow serialized parameters differ from its admitted facts",
            ));
        }
        let binding = self
            .native_shadow_parameter_bindings
            .get(&request.id, account)?
            .ok_or_else(|| preparation_error("native shadow has no prepared uniform owner"))?;
        account.work(RenderWork::ResourceProbes, 1)?;
        if binding.bytes != *parameters {
            return Err(preparation_error(
                "native shadow prepared uniform bytes differ from its request",
            ));
        }
        let uniform = binding.buffer.as_ref().ok_or_else(|| {
            preparation_error("native shadow uniform is missing or already released")
        })?;
        if uniform.size() != 256 || !uniform.usage().contains(wgpu::BufferUsages::UNIFORM) {
            return Err(preparation_error(
                "native shadow uniform has an invalid concrete allocation",
            ));
        }
        let keys = request
            .cache_keys
            .as_ref()
            .ok_or_else(|| preparation_error("native shadow cache keys are missing"))?;
        let objects: crate::shader::ProvisionalNativeShadowPassObjects<'_> = self
            .pass_cache_update
            .as_ref()
            .ok_or_else(|| preparation_error("native shadow provisional cache is missing"))?
            .native_shadow_encoding_objects(
                self.pass_cache,
                account,
                keys.samplers(),
                keys.layout(),
                keys.shader(),
                keys.pipeline(),
            )?;
        objects.require_encoding_ready()?;
        let mut entries = account
            .temporary(1 + usize::from(original_required) + usize::from(map.has_shifted_alpha()))?;
        if let Some(original) = original_binding.as_ref() {
            entries.append_slot()?.write(wgpu::BindGroupEntry {
                binding: 0,
                resource: wgpu::BindingResource::TextureView(original.view()),
            });
        }
        if let Some(alpha) = alpha_binding.as_ref() {
            entries.append_slot()?.write(wgpu::BindGroupEntry {
                binding: 1,
                resource: wgpu::BindingResource::TextureView(alpha.view()),
            });
        }
        entries.append_slot()?.write(wgpu::BindGroupEntry {
            binding: 2,
            resource: uniform.as_entire_binding(),
        });
        account.work(RenderWork::ShaderBindingVisits, entries.len())?;
        let bindings =
            self.resources
                .generation()
                .opaque(GpuObjectKind::BindGroup, Some(account), || {
                    self.device.create_bind_group(&wgpu::BindGroupDescriptor {
                        label: Some("Surgeist native shadow retained original and shifted alpha"),
                        layout: objects.bind_group_layout(),
                        entries: &entries,
                    })
                })?;
        let region = GraphRenderRegion::full(target_spatial.device_extent)?;
        let original_samples: u64 = match map.original_kind() {
            NativeShadowOriginal::Finite => 1,
            NativeShadowOriginal::Periodic => 4,
            NativeShadowOriginal::Constant => 0,
        };
        let samples = scheduled_region_texels(region)
            .checked_mul(original_samples + u64::from(map.has_shifted_alpha()))
            .ok_or(crate::RenderAccountingError::Invariant)?;
        account.work_u64(RenderWork::ScheduledShadowSamples, samples)?;
        encode_spatial_filter_full_target_pass(
            encoder,
            target_binding.view(),
            SpatialPassDrawBindings {
                pipeline: objects.render_pipeline(),
                bind_group: &bindings,
            },
            region,
            "Surgeist native shadow flood-in and original-over merge",
            true,
            account,
        )
    }
}
