use super::*;

impl PreparedGraph<'_> {
    pub(super) fn encode_lighting(
        &self,
        encoder: &mut wgpu::CommandEncoder,
        request: &PreparedPassEncodingRequest,
        account: &Account,
    ) -> Result<()> {
        let RuntimePassKind::Lighting(work) = &request.kind else {
            return Err(preparation_error("lighting requires its admitted stage"));
        };
        let stage = super::super::lighting::shader_stage(work);
        if request.reads.len() != usize::from(stage.reads_source())
            || stage.reads_source() != work.source_plan().is_some()
        {
            return Err(preparation_error(
                "lighting source count differs from its stage",
            ));
        }
        let RuntimeResultBinding::Resource(target) = request.result else {
            return Err(preparation_error(
                "lighting requires a concrete texture target",
            ));
        };
        let format = self.plan.working_format.texture_format();
        let target_binding = self.texture_binding_for_pass(request.id, target, account)?;
        let target_spatial = self.validate_texture_binding(&target_binding, target, account)?;
        let scissor = work.scissor();
        account.work(RenderWork::CoordinateVisits, 2)?;
        if target_binding.texture().format() != format
            || !target_binding
                .texture()
                .usage()
                .contains(wgpu::TextureUsages::RENDER_ATTACHMENT)
            || target_spatial != RuntimeSpatialDescriptor::from_graph(work.target_plan())
            || !super::super::lighting::valid_scissor(scissor, target_spatial.device_extent)
        {
            return Err(preparation_error(
                "lighting target differs from its checked working plane",
            ));
        }

        let source_binding = if let Some(source_plan) = work.source_plan() {
            let source = exact_core_pass_read(request, RuntimeReadRole::LightingSource, account)?;
            let binding = self.texture_binding_for_pass(request.id, source.resource, account)?;
            let spatial = self.validate_texture_binding(&binding, source.resource, account)?;
            account.work(RenderWork::CoordinateVisits, 1)?;
            if source.resource == target
                || binding.allocation_resource() == target_binding.allocation_resource()
                || binding.texture().format() != format
                || !binding
                    .texture()
                    .usage()
                    .contains(wgpu::TextureUsages::TEXTURE_BINDING)
                || spatial != RuntimeSpatialDescriptor::from_graph(source_plan)
            {
                return Err(preparation_error(
                    "lighting source differs from its exact nonaliasing working plane",
                ));
            }
            Some(binding)
        } else {
            None
        };

        let binding = self
            .acquired_bindings
            .lighting_parameter_bindings
            .get(&request.id, account)?
            .ok_or_else(|| preparation_error("lighting has no prepared uniform owner"))?;
        account.work(RenderWork::ResourceProbes, 1)?;
        if binding.bytes.as_bytes() != work.uniform_bytes() {
            return Err(preparation_error(
                "lighting uniform differs from its admitted stage",
            ));
        }
        let uniform = binding.buffer.active(
            request.id,
            PassBufferRole::Lighting,
            LightingParameterBytes::SIZE,
            self.resources.generation(),
        )?;
        let keys = request
            .cache_keys
            .as_ref()
            .ok_or_else(|| preparation_error("lighting cache keys are missing"))?;
        let objects = self
            .pass_cache_update
            .as_ref()
            .ok_or_else(|| preparation_error("lighting provisional cache is missing"))?
            .lighting_encoding_objects(
                self.pass_cache,
                account,
                LightingPassKeyRefs {
                    samplers: keys.samplers(),
                    layout: keys.layout(),
                    shader: keys.shader(),
                    pipeline: keys.pipeline(),
                },
                stage,
            )?;
        let parameter_entry = wgpu::BindGroupEntry {
            binding: 1,
            resource: uniform.as_entire_binding(),
        };
        let entries_storage;
        let entries = if let Some(source) = &source_binding {
            entries_storage = [
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: wgpu::BindingResource::TextureView(source.view()),
                },
                parameter_entry,
            ];
            &entries_storage[..]
        } else {
            std::slice::from_ref(&parameter_entry)
        };
        account.work(RenderWork::ShaderBindingVisits, entries.len())?;
        let bindings =
            self.resources
                .generation()
                .opaque(GpuObjectKind::BindGroup, Some(account), || {
                    self.device.create_bind_group(&wgpu::BindGroupDescriptor {
                        label: Some("Surgeist lighting exact stage bindings"),
                        layout: objects.layout,
                        entries,
                    })
                })?;
        let mut region = GraphRenderRegion::full(target_spatial.device_extent)?;
        [
            region.scissor_x,
            region.scissor_y,
            region.scissor_width,
            region.scissor_height,
        ] = scissor;
        let texels = scheduled_region_texels(region);
        observe_draw_work(stage, texels, account)?;
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
            "Surgeist lighting bounded stage",
        );
        Ok(())
    }
}

fn observe_draw_work(stage: ShaderLightingStageKey, texels: u64, account: &Account) -> Result<()> {
    let pass = (RenderWork::ImagePasses, 1);
    let filter = (RenderWork::ScheduledFilterTexels, texels);
    let samples = |count| {
        texels
            .checked_mul(count)
            .ok_or(crate::accounting::RenderAccountingError::Invariant)
    };
    let evaluations = (RenderWork::ScheduledLightingEvaluations, texels);
    match stage {
        ShaderLightingStageKey::ResampleBump => account.work_batch([
            pass,
            filter,
            (RenderWork::ScheduledLightingAlphaSamples, samples(4)?),
        ]),
        ShaderLightingStageKey::Shade => account.work_batch([
            pass,
            filter,
            (RenderWork::ScheduledLightingAlphaSamples, samples(9)?),
            evaluations,
        ]),
        ShaderLightingStageKey::ShadeZeroBump => account.work_batch([pass, filter, evaluations]),
        ShaderLightingStageKey::Resolve => account.work_batch([
            pass,
            filter,
            (RenderWork::ScheduledLightingResultSamples, samples(4)?),
        ]),
    }
}
