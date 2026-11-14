//! Native Gaussian stages use only their real stage-specific resources.
use super::*;
use crate::frame::GaussianStage;

impl PreparedGraph<'_> {
    pub(super) fn encode_gaussian(
        &self,
        encoder: &mut wgpu::CommandEncoder,
        request: &PreparedPassEncodingRequest,
        account: &Account,
    ) -> Result<()> {
        let RuntimePassKind::Gaussian(work) = &request.kind else {
            return Err(preparation_error(
                "Gaussian encoding requires its admitted payload",
            ));
        };
        let map = work.mapping();
        let source = if work.stage() == GaussianStage::Constant {
            if !request.reads.is_empty() {
                return Err(preparation_error("constant Gaussian samples no image"));
            }
            None
        } else {
            Some(exact_core_pass_read(
                request,
                RuntimeReadRole::GaussianSource,
                account,
            )?)
        };
        let carry = matches!(work.stage(), GaussianStage::Accumulate { first: false, .. });
        let accumulate = matches!(work.stage(), GaussianStage::Accumulate { .. });
        let mut auxiliary = None;
        for read in &request.reads {
            account.work(RenderWork::ResourceProbes, 1)?;
            match read.role {
                RuntimeReadRole::GaussianSource => {}
                RuntimeReadRole::GaussianAuxiliary if carry => auxiliary = Some(read),
                _ => return Err(preparation_error("Gaussian has an unrelated texture read")),
            }
        }
        if auxiliary.is_some() != carry
            || request.reads.len() != usize::from(source.is_some()) + usize::from(carry)
        {
            return Err(preparation_error(
                "Gaussian reads differ from its exact stage",
            ));
        }
        let RuntimeResultBinding::Resource(target) = request.result else {
            return Err(preparation_error("Gaussian requires a concrete target"));
        };
        let target_binding = self.texture_binding_for_pass(request.id, target, account)?;
        let target_spatial = self.validate_texture_binding(&target_binding, target, account)?;
        if target_spatial != RuntimeSpatialDescriptor::from_graph(map.target_spatial()) {
            return Err(preparation_error(
                "Gaussian target differs from its checked lattice",
            ));
        }
        let target_format = if matches!(
            work.stage(),
            GaussianStage::Prepare | GaussianStage::Accumulate { .. }
        ) {
            wgpu::TextureFormat::Rgba32Float
        } else {
            self.plan.working_format.texture_format()
        };
        if target_binding.texture().format() != target_format
            || !target_binding
                .texture()
                .usage()
                .contains(wgpu::TextureUsages::RENDER_ATTACHMENT)
        {
            return Err(preparation_error(
                "Gaussian target is not its admitted attachment",
            ));
        }
        let source_binding = source
            .map(|read| -> Result<_> {
                let binding = self.texture_binding_for_pass(request.id, read.resource, account)?;
                let spatial = self.validate_texture_binding(&binding, read.resource, account)?;
                if Some(spatial)
                    != map
                        .source_spatial()
                        .map(RuntimeSpatialDescriptor::from_graph)
                {
                    return Err(preparation_error(
                        "Gaussian source differs from its checked lattice",
                    ));
                }
                Ok(binding)
            })
            .transpose()?;
        let auxiliary_binding = auxiliary
            .map(|read| -> Result<_> {
                let binding = self.texture_binding_for_pass(request.id, read.resource, account)?;
                let spatial = self.validate_texture_binding(&binding, read.resource, account)?;
                if spatial != target_spatial
                    || binding.texture().format() != wgpu::TextureFormat::Rgba32Float
                {
                    return Err(preparation_error(
                        "Gaussian carry differs from its target float lattice",
                    ));
                }
                Ok(binding)
            })
            .transpose()?;
        for binding in source_binding.iter().chain(auxiliary_binding.iter()) {
            if binding.runtime_resource() == target
                || binding.allocation_resource() == target_binding.allocation_resource()
                || !binding
                    .texture()
                    .usage()
                    .contains(wgpu::TextureUsages::TEXTURE_BINDING)
            {
                return Err(preparation_error(
                    "Gaussian target aliases or cannot sample a bound source",
                ));
            }
        }
        let bytes = request
            .gaussian_parameters
            .as_ref()
            .ok_or_else(|| preparation_error("Gaussian parameters are missing"))?;
        account.work(RenderWork::ResourceProbes, 1)?;
        if *bytes != super::super::gaussian::parameters(work, account)? {
            return Err(preparation_error(
                "Gaussian parameters differ from its checked payload",
            ));
        }
        let uniform_binding = self
            .gaussian_parameter_bindings
            .get(&request.id, account)?
            .ok_or_else(|| preparation_error("Gaussian uniform owner is missing"))?;
        account.work(RenderWork::ResourceProbes, 1)?;
        if uniform_binding.bytes != *bytes {
            return Err(preparation_error("Gaussian uniform bytes changed"));
        }
        let uniform = uniform_binding
            .buffer
            .as_ref()
            .ok_or_else(|| preparation_error("Gaussian uniform is stale"))?;
        if uniform.size() != 256 || !uniform.usage().contains(wgpu::BufferUsages::UNIFORM) {
            return Err(preparation_error("Gaussian uniform allocation is invalid"));
        }
        let keys = request
            .cache_keys
            .as_ref()
            .ok_or_else(|| preparation_error("Gaussian cache keys are missing"))?;
        let objects = self
            .pass_cache_update
            .as_ref()
            .ok_or_else(|| preparation_error("Gaussian cache is not provisional"))?
            .gaussian_encoding_objects(
                self.pass_cache,
                account,
                keys.samplers(),
                keys.layout(),
                keys.shader(),
                keys.pipeline(),
            )?;
        objects.require_encoding_ready()?;
        let kernel = self.gaussian_kernel_binding_for_pass(request.id, account)?;
        if kernel.is_some() != accumulate {
            return Err(preparation_error(
                "Gaussian kernel reachability differs from stage",
            ));
        }
        let mut entries = account.temporary(4)?;
        if let Some(source_binding) = &source_binding {
            entries.append_slot()?.write(wgpu::BindGroupEntry {
                binding: 0,
                resource: wgpu::BindingResource::TextureView(source_binding.view()),
            });
            if accumulate {
                let auxiliary = auxiliary_binding.as_ref().unwrap_or(source_binding);
                entries.append_slot()?.write(wgpu::BindGroupEntry {
                    binding: 1,
                    resource: wgpu::BindingResource::TextureView(auxiliary.view()),
                });
            }
        }
        entries.append_slot()?.write(wgpu::BindGroupEntry {
            binding: 2,
            resource: uniform.as_entire_binding(),
        });
        if let Some(kernel) = &kernel {
            let GaussianStage::Accumulate {
                axis,
                first_tap,
                tap_count,
                ..
            } = work.stage()
            else {
                return Err(preparation_error(
                    "unreachable Gaussian coefficient binding",
                ));
            };
            let geometry = map
                .geometry(axis)
                .ok_or_else(|| preparation_error("Gaussian axis geometry is missing"))?;
            if kernel.key() != geometry.key()
                || kernel.buffer().size() != geometry.byte_len()
                || first_tap
                    .checked_add(tap_count)
                    .is_none_or(|end| end > geometry.sample_count())
                || tap_count == 0
                || tap_count > 32
            {
                return Err(preparation_error(
                    "Gaussian coefficient range differs from its actual buffer",
                ));
            }
            entries.append_slot()?.write(wgpu::BindGroupEntry {
                binding: 3,
                resource: wgpu::BindingResource::Buffer(wgpu::BufferBinding {
                    buffer: kernel.buffer(),
                    offset: u64::from(first_tap) * 8,
                    size: wgpu::BufferSize::new(u64::from(tap_count) * 8),
                }),
            });
        }
        account.work(RenderWork::ShaderBindingVisits, entries.len())?;
        let bindings =
            self.resources
                .generation()
                .opaque(GpuObjectKind::BindGroup, Some(account), || {
                    self.device.create_bind_group(&wgpu::BindGroupDescriptor {
                        label: Some("Surgeist Gaussian exact stage bindings"),
                        layout: objects.bind_group_layout(),
                        entries: &entries,
                    })
                })?;
        let region = GraphRenderRegion::full(target_spatial.device_extent)?;
        let footprint = match work.stage() {
            GaussianStage::Constant => 0,
            GaussianStage::Accumulate {
                tap_count, first, ..
            } => u64::from(tap_count) + u64::from(!first),
            GaussianStage::PassThrough | GaussianStage::Resolve if map.periodic() => 4,
            _ => 1,
        };
        let samples = scheduled_region_texels(region)
            .checked_mul(footprint)
            .ok_or(crate::RenderAccountingError::Invariant)?;
        account.work_batch([(RenderWork::ScheduledGaussianSamples, samples)])?;
        encode_spatial_filter_full_target_pass(
            encoder,
            target_binding.view(),
            SpatialPassDrawBindings {
                pipeline: objects.render_pipeline(),
                bind_group: &bindings,
            },
            region,
            "Surgeist Gaussian bounded stage",
            true,
            account,
        )
    }
}
