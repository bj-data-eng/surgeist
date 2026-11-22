use super::*;

impl PreparedGraph<'_> {
    pub(super) fn encode_turbulence(
        &self,
        encoder: &mut wgpu::CommandEncoder,
        request: &PreparedPassEncodingRequest,
        account: &Account,
    ) -> Result<()> {
        let RuntimePassKind::Turbulence(work) = &request.kind else {
            return Err(preparation_error("turbulence requires its admitted pass"));
        };
        if !request.reads.is_empty() {
            return Err(preparation_error("turbulence has no texture inputs"));
        }
        let RuntimeResultBinding::Resource(target) = request.result else {
            return Err(preparation_error(
                "turbulence requires a concrete texture target",
            ));
        };
        let format = self.plan.working_format.texture_format();
        let target_binding = self.texture_binding_for_pass(request.id, target, account)?;
        let spatial = self.validate_texture_binding(&target_binding, target, account)?;
        account.work(RenderWork::CoordinateVisits, 1)?;
        if target_binding.texture().format() != format
            || !target_binding
                .texture()
                .usage()
                .contains(wgpu::TextureUsages::RENDER_ATTACHMENT)
            || spatial != RuntimeSpatialDescriptor::from_graph(work.target_plan())
        {
            return Err(preparation_error(
                "turbulence target differs from its admitted working plane",
            ));
        }
        let binding = self
            .acquired_bindings
            .turbulence_parameter_bindings
            .get(&request.id, account)?
            .ok_or_else(|| preparation_error("turbulence has no prepared uniform owner"))?;
        account.work(RenderWork::ResourceProbes, 1)?;
        if !binding.bytes.belongs_to(work) {
            return Err(preparation_error(
                "turbulence uniform differs from its admitted pass",
            ));
        }
        let uniform = binding.buffer.active(
            request.id,
            PassBufferRole::Turbulence,
            TurbulenceParameterBytes::SIZE,
            self.resources.generation(),
        )?;
        let keys = request
            .cache_keys
            .as_ref()
            .ok_or_else(|| preparation_error("turbulence cache keys are missing"))?;
        let objects = self
            .pass_cache_update
            .as_ref()
            .ok_or_else(|| preparation_error("turbulence provisional cache is missing"))?
            .turbulence_encoding_objects(
                self.pass_cache,
                account,
                TurbulencePassKeyRefs {
                    samplers: keys.samplers(),
                    layout: keys.layout(),
                    shader: keys.shader(),
                    pipeline: keys.pipeline(),
                },
                crate::shader::ShaderTextureFormatKey::working(self.plan.working_format),
            )?;
        account.work(RenderWork::ShaderBindingVisits, 1)?;
        let bindings =
            self.resources
                .generation()
                .opaque(GpuObjectKind::BindGroup, Some(account), || {
                    self.device.create_bind_group(&wgpu::BindGroupDescriptor {
                        label: Some("Surgeist turbulence exact bindings"),
                        layout: objects.layout,
                        entries: &[wgpu::BindGroupEntry {
                            binding: 0,
                            resource: uniform.as_entire_binding(),
                        }],
                    })
                })?;
        let region = GraphRenderRegion::full(spatial.device_extent)?;
        let texels = scheduled_region_texels(region);
        let octave_visits = texels
            .checked_mul(u64::from(work.octaves()))
            .ok_or(crate::accounting::RenderAccountingError::Invariant)?;
        account.work_batch([
            (RenderWork::ImagePasses, 1),
            (RenderWork::ScheduledFilterTexels, texels),
            (RenderWork::ScheduledTurbulenceOctaves, octave_visits),
        ])?;
        draw_bounded_spatial_filter(
            encoder,
            target_binding.view(),
            SpatialPassDrawBindings {
                #[cfg(test)]
                target_kind_for_test: SpatialPassTargetForTest::Dedicated,
                pipeline: objects.pipeline,
                bind_group: Some(&bindings),
            },
            region,
            "Surgeist turbulence bounded pass",
        );
        Ok(())
    }
}
