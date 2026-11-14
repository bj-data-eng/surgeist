use super::*;

impl PreparedGraph<'_> {
    pub(super) fn encode_offset(
        &self,
        encoder: &mut wgpu::CommandEncoder,
        request: &PreparedPassEncodingRequest,
        account: &Account,
    ) -> Result<()> {
        let RuntimePassKind::Offset(operation) = &request.kind else {
            return Err(preparation_error(
                "offset encoding requires its admitted payload",
            ));
        };
        let source = exact_core_pass_read(request, RuntimeReadRole::OffsetSource, account)?;
        if request.reads.len() != 1 {
            return Err(preparation_error(
                "offset must have exactly one sampled source",
            ));
        }
        let RuntimeResultBinding::Resource(target) = request.result else {
            return Err(preparation_error(
                "offset requires a concrete texture target",
            ));
        };
        let source_binding = self.texture_binding_for_pass(request.id, source.resource, account)?;
        let source_spatial =
            self.validate_texture_binding(&source_binding, source.resource, account)?;
        let target_binding = self.texture_binding_for_pass(request.id, target, account)?;
        let target_spatial = self.validate_texture_binding(&target_binding, target, account)?;
        let map = operation.mapping();
        account.work(RenderWork::CoordinateVisits, 2)?;
        if source.resource == target
            || source_binding.allocation_resource() == target_binding.allocation_resource()
            || !source_binding
                .texture()
                .usage()
                .contains(wgpu::TextureUsages::TEXTURE_BINDING)
            || !target_binding
                .texture()
                .usage()
                .contains(wgpu::TextureUsages::RENDER_ATTACHMENT)
            || source_binding.texture().format() != self.plan.working_format.texture_format()
            || target_binding.texture().format() != self.plan.working_format.texture_format()
            || [
                source_spatial.device_extent.width(),
                source_spatial.device_extent.height(),
            ] != map.source_extent()
            || [
                target_spatial.device_extent.width(),
                target_spatial.device_extent.height(),
            ] != map.result_extent()
            || [
                source_spatial.texel_origin.x(),
                source_spatial.texel_origin.y(),
            ] != map.source_origin()
            || [
                target_spatial.texel_origin.x(),
                target_spatial.texel_origin.y(),
            ] != map.result_origin()
            || source_spatial.raster_scale != map.source_scale()
            || target_spatial.raster_scale != map.result_scale()
        {
            return Err(preparation_error(
                "offset resources differ from the exact nonaliasing working lattices",
            ));
        }
        let parameters = request
            .offset_parameters
            .as_ref()
            .ok_or_else(|| preparation_error("offset has no serialized texel mapping"))?;
        let expected_parameters = super::super::offset::parameters(operation, account)?;
        account.work(RenderWork::ResourceProbes, 1)?;
        if *parameters != expected_parameters {
            return Err(preparation_error(
                "offset serialized parameters differ from its checked payload",
            ));
        }
        let binding = self
            .offset_parameter_bindings
            .get(&request.id, account)?
            .ok_or_else(|| preparation_error("offset has no prepared uniform owner"))?;
        account.work(RenderWork::ResourceProbes, 1)?;
        if binding.bytes != *parameters {
            return Err(preparation_error(
                "offset uniform bytes differ from its prepared request",
            ));
        }
        let uniform = binding
            .buffer
            .as_ref()
            .ok_or_else(|| preparation_error("offset uniform is missing or already released"))?;
        if uniform.size() != 256 || !uniform.usage().contains(wgpu::BufferUsages::UNIFORM) {
            return Err(preparation_error(
                "offset prepared uniform has an invalid concrete allocation",
            ));
        }
        let keys = request
            .cache_keys
            .as_ref()
            .ok_or_else(|| preparation_error("offset cache keys are missing"))?;
        let objects: crate::shader::ProvisionalOffsetPassObjects<'_> = self
            .pass_cache_update
            .as_ref()
            .ok_or_else(|| preparation_error("offset provisional cache is missing"))?
            .offset_encoding_objects(
                self.pass_cache,
                account,
                keys.samplers(),
                keys.layout(),
                keys.shader(),
                keys.pipeline(),
            )?;
        objects.require_encoding_ready()?;
        account.work(RenderWork::ShaderBindingVisits, 2)?;
        let bindings =
            self.resources
                .generation()
                .opaque(GpuObjectKind::BindGroup, Some(account), || {
                    self.device.create_bind_group(&wgpu::BindGroupDescriptor {
                        label: Some("Surgeist offset exact source and texel mapping"),
                        layout: objects.bind_group_layout(),
                        entries: &[
                            wgpu::BindGroupEntry {
                                binding: 0,
                                resource: wgpu::BindingResource::TextureView(source_binding.view()),
                            },
                            wgpu::BindGroupEntry {
                                binding: 1,
                                resource: uniform.as_entire_binding(),
                            },
                        ],
                    })
                })?;
        let region = GraphRenderRegion::full(target_spatial.device_extent)?;
        let samples = scheduled_region_texels(region)
            .checked_mul(map.sampling().footprint())
            .ok_or(crate::RenderAccountingError::Invariant)?;
        encode_spatial_filter_full_target_pass_with_samples(
            encoder,
            target_binding.view(),
            SpatialPassDrawBindings {
                #[cfg(test)]
                target_kind_for_test: SpatialPassTargetForTest::Dedicated,
                pipeline: objects.render_pipeline(),
                bind_group: &bindings,
            },
            region,
            "Surgeist offset bounded sample",
            true,
            Some(samples),
            account,
        )
    }
}
