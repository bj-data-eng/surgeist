use super::*;

impl PreparedGraph<'_> {
    pub(super) fn encode_tile(
        &self,
        encoder: &mut wgpu::CommandEncoder,
        request: &PreparedPassEncodingRequest,
        account: &Account,
    ) -> Result<()> {
        let RuntimePassKind::Tile(work) = &request.kind else {
            return Err(preparation_error("tile requires its admitted pass"));
        };
        let stage = super::super::tile_stage(work);
        if request.reads.len() != usize::from(stage.reads_source()) {
            return Err(preparation_error(
                "tile source count differs from its stage",
            ));
        }
        let RuntimeResultBinding::Resource(target) = request.result else {
            return Err(preparation_error("tile requires a concrete texture target"));
        };
        let format = self.plan.working_format.texture_format();
        let target_binding = self.texture_binding_for_pass(request.id, target, account)?;
        let target_spatial = self.validate_texture_binding(&target_binding, target, account)?;
        account.work(RenderWork::CoordinateVisits, 1)?;
        if target_binding.texture().format() != format
            || !target_binding
                .texture()
                .usage()
                .contains(wgpu::TextureUsages::RENDER_ATTACHMENT)
            || target_spatial != RuntimeSpatialDescriptor::from_graph(work.target_plan())
        {
            return Err(preparation_error(
                "tile target differs from its admitted working plane",
            ));
        }
        let keys = request
            .cache_keys
            .as_ref()
            .ok_or_else(|| preparation_error("tile cache keys are missing"))?;
        let objects = self
            .pass_cache_update
            .as_ref()
            .ok_or_else(|| preparation_error("tile provisional cache is missing"))?
            .tile_encoding_objects(
                self.pass_cache,
                account,
                TilePassKeyRefs {
                    samplers: keys.samplers(),
                    layout: keys.layout(),
                    shader: keys.shader(),
                    pipeline: keys.pipeline(),
                },
                stage,
                crate::shader::ShaderTextureFormatKey::working(self.plan.working_format),
            )?;

        let bindings = match (work.source_plan(), objects.layout) {
            (Some(source_plan), Some(layout)) => {
                let source = exact_core_pass_read(request, RuntimeReadRole::TileSource, account)?;
                let binding =
                    self.texture_binding_for_pass(request.id, source.resource, account)?;
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
                        "tile source differs from its exact nonaliasing working plane",
                    ));
                }
                let parameters = self
                    .acquired_bindings
                    .tile_parameter_bindings
                    .get(&request.id, account)?
                    .ok_or_else(|| preparation_error("tile has no prepared uniform owner"))?;
                account.work(RenderWork::ResourceProbes, 1)?;
                if parameters.bytes.as_bytes() != work.uniform_bytes() {
                    return Err(preparation_error(
                        "tile uniform differs from its admitted pass",
                    ));
                }
                let uniform = parameters.buffer.active(
                    request.id,
                    PassBufferRole::Tile,
                    TileParameterBytes::SIZE,
                    self.resources.generation(),
                )?;
                let entries = [
                    wgpu::BindGroupEntry {
                        binding: 0,
                        resource: wgpu::BindingResource::TextureView(binding.view()),
                    },
                    wgpu::BindGroupEntry {
                        binding: 1,
                        resource: uniform.as_entire_binding(),
                    },
                ];
                account.work(RenderWork::ShaderBindingVisits, entries.len())?;
                Some(self.resources.generation().opaque(
                    GpuObjectKind::BindGroup,
                    Some(account),
                    || {
                        self.device.create_bind_group(&wgpu::BindGroupDescriptor {
                            label: Some("Surgeist tile exact repeat bindings"),
                            layout,
                            entries: &entries,
                        })
                    },
                )?)
            }
            (None, None) => None,
            _ => {
                return Err(preparation_error(
                    "tile bindings differ from the admitted stage",
                ));
            }
        };
        let region = GraphRenderRegion::full(target_spatial.device_extent)?;
        let texels = scheduled_region_texels(region);
        let samples = texels
            .checked_mul(work.samples_per_cell())
            .ok_or(crate::accounting::RenderAccountingError::Invariant)?;
        account.work_batch([
            (RenderWork::ImagePasses, 1),
            (RenderWork::ScheduledFilterTexels, texels),
            (RenderWork::ScheduledTileSamples, samples),
        ])?;
        draw_bounded_spatial_filter(
            encoder,
            target_binding.view(),
            SpatialPassDrawBindings {
                #[cfg(test)]
                target_kind_for_test: SpatialPassTargetForTest::Dedicated,
                pipeline: objects.pipeline,
                bind_group: bindings.as_deref(),
            },
            region,
            "Surgeist bounded native tile",
        );
        Ok(())
    }
}
