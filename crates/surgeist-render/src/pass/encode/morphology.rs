//! Every morphology stage binds only its actual image inputs and one uniform.
use super::*;
use crate::frame::MorphologyStage;

impl PreparedGraph<'_> {
    pub(super) fn encode_morphology(
        &self,
        encoder: &mut wgpu::CommandEncoder,
        request: &PreparedPassEncodingRequest,
        account: &Account,
    ) -> Result<()> {
        let RuntimePassKind::Morphology(work) = &request.kind else {
            return Err(preparation_error(
                "morphology encoding requires its checked payload",
            ));
        };
        let stage = work.stage();
        let map = work.mapping();
        let refine = matches!(stage, MorphologyStage::RefineQuery { .. });
        let constant = stage == MorphologyStage::Constant;
        let source = if constant {
            None
        } else {
            Some(exact_core_pass_read(
                request,
                RuntimeReadRole::MorphologySource,
                account,
            )?)
        };
        let mut previous = None;
        for read in &request.reads {
            account.work(RenderWork::ResourceProbes, 1)?;
            match read.role {
                RuntimeReadRole::MorphologySource if !constant => {}
                RuntimeReadRole::MorphologyPreviousQuery if refine && previous.is_none() => {
                    previous = Some(read);
                }
                _ => {
                    return Err(preparation_error(
                        "morphology has an unrelated or duplicate read",
                    ));
                }
            }
        }
        if previous.is_some() != refine
            || request.reads.len() != usize::from(!constant) + usize::from(refine)
        {
            return Err(preparation_error(
                "morphology reads differ from its exact stage",
            ));
        }
        let RuntimeResultBinding::Resource(target) = request.result else {
            return Err(preparation_error("morphology requires a concrete target"));
        };
        let target_binding = self.texture_binding_for_pass(request.id, target, account)?;
        let target_spatial = self.validate_texture_binding(&target_binding, target, account)?;
        let target_format = if work.needs_float32() {
            wgpu::TextureFormat::Rgba32Float
        } else {
            self.plan.working_format.texture_format()
        };
        if target_spatial != RuntimeSpatialDescriptor::from_graph(map.target_spatial())
            || target_binding.texture().format() != target_format
            || !target_binding
                .texture()
                .usage()
                .contains(wgpu::TextureUsages::RENDER_ATTACHMENT)
        {
            return Err(preparation_error(
                "morphology target differs from its checked attachment",
            ));
        }
        let source_binding = source
            .map(|read| -> Result<_> {
                let binding = self.texture_binding_for_pass(request.id, read.resource, account)?;
                let spatial = self.validate_texture_binding(&binding, read.resource, account)?;
                let expected_format = if matches!(
                    stage,
                    MorphologyStage::Table { .. }
                        | MorphologyStage::FirstQuery { .. }
                        | MorphologyStage::RefineQuery { .. }
                        | MorphologyStage::Query { .. }
                        | MorphologyStage::Resolve
                ) {
                    wgpu::TextureFormat::Rgba32Float
                } else {
                    self.plan.working_format.texture_format()
                };
                if Some(spatial)
                    != map
                        .source_spatial()
                        .map(RuntimeSpatialDescriptor::from_graph)
                    || binding.texture().format() != expected_format
                {
                    return Err(preparation_error(
                        "morphology source differs from its checked lattice or format",
                    ));
                }
                Ok(binding)
            })
            .transpose()?;
        let previous_binding = previous
            .map(|read| -> Result<_> {
                let binding = self.texture_binding_for_pass(request.id, read.resource, account)?;
                let spatial = self.validate_texture_binding(&binding, read.resource, account)?;
                if Some(spatial)
                    != map
                        .previous_query_spatial()
                        .map(RuntimeSpatialDescriptor::from_graph)
                    || spatial != target_spatial
                    || binding.texture().format() != wgpu::TextureFormat::Rgba32Float
                    || source_binding.as_ref().is_some_and(|source| {
                        source.runtime_resource() == binding.runtime_resource()
                            || source.allocation_resource() == binding.allocation_resource()
                    })
                {
                    return Err(preparation_error(
                        "morphology previous query is not its distinct float lattice",
                    ));
                }
                Ok(binding)
            })
            .transpose()?;
        for binding in source_binding.iter().chain(previous_binding.iter()) {
            if binding.runtime_resource() == target
                || binding.allocation_resource() == target_binding.allocation_resource()
                || !binding
                    .texture()
                    .usage()
                    .contains(wgpu::TextureUsages::TEXTURE_BINDING)
            {
                return Err(preparation_error(
                    "morphology target aliases or cannot sample an input",
                ));
            }
        }
        let bytes = request
            .morphology_parameters
            .as_ref()
            .ok_or_else(|| preparation_error("morphology parameters are missing"))?;
        account.work(RenderWork::ResourceProbes, 1)?;
        if *bytes != super::super::morphology::parameters(work, account)? {
            return Err(preparation_error(
                "morphology parameters differ from its checked payload",
            ));
        }
        let uniform_binding = self
            .morphology_parameter_bindings
            .get(&request.id, account)?
            .ok_or_else(|| preparation_error("morphology uniform owner is missing"))?;
        account.work(RenderWork::ResourceProbes, 1)?;
        if uniform_binding.bytes != *bytes {
            return Err(preparation_error("morphology uniform bytes changed"));
        }
        let uniform = uniform_binding
            .buffer
            .as_ref()
            .ok_or_else(|| preparation_error("morphology uniform is stale"))?;
        if uniform.size() != 256 || !uniform.usage().contains(wgpu::BufferUsages::UNIFORM) {
            return Err(preparation_error(
                "morphology uniform allocation is invalid",
            ));
        }
        let keys = request
            .cache_keys
            .as_ref()
            .ok_or_else(|| preparation_error("morphology cache keys are missing"))?;
        let objects = self
            .pass_cache_update
            .as_ref()
            .ok_or_else(|| preparation_error("morphology cache is not provisional"))?
            .morphology_encoding_objects(
                self.pass_cache,
                account,
                keys.samplers(),
                keys.layout(),
                keys.shader(),
                keys.pipeline(),
            )?;
        objects.require_encoding_ready()?;
        let mut entries = account.temporary(request.reads.len() + 1)?;
        if let Some(source) = &source_binding {
            entries.append_slot()?.write(wgpu::BindGroupEntry {
                binding: 0,
                resource: wgpu::BindingResource::TextureView(source.view()),
            });
        }
        if let Some(previous) = &previous_binding {
            entries.append_slot()?.write(wgpu::BindGroupEntry {
                binding: 1,
                resource: wgpu::BindingResource::TextureView(previous.view()),
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
                        label: Some("Surgeist morphology exact stage bindings"),
                        layout: objects.bind_group_layout(),
                        entries: &entries,
                    })
                })?;
        let region = GraphRenderRegion::full(target_spatial.device_extent)?;
        let samples = scheduled_region_texels(region)
            .checked_mul(work.sample_opportunities())
            .ok_or(crate::RenderAccountingError::Invariant)?;
        account.work_batch([(RenderWork::ScheduledMorphologySamples, samples)])?;
        encode_spatial_filter_full_target_pass(
            encoder,
            target_binding.view(),
            SpatialPassDrawBindings {
                pipeline: objects.render_pipeline(),
                bind_group: &bindings,
            },
            region,
            "Surgeist morphology bounded stage",
            true,
            account,
        )
    }
}
