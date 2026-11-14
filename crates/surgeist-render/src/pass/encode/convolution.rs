use super::*;
use crate::frame::ConvolutionStage;

impl PreparedGraph<'_> {
    pub(super) fn encode_convolution(
        &self,
        encoder: &mut wgpu::CommandEncoder,
        request: &PreparedPassEncodingRequest,
        account: &Account,
    ) -> Result<()> {
        let RuntimePassKind::Convolution(operation) = &request.kind else {
            return Err(preparation_error(
                "convolution encoding requires its admitted payload",
            ));
        };
        let source = exact_core_pass_read(request, RuntimeReadRole::ConvolutionSource, account)?;
        let mut auxiliary = None;
        for read in &request.reads {
            account.work(RenderWork::ResourceProbes, 1)?;
            if read.role == RuntimeReadRole::ConvolutionAuxiliary {
                auxiliary = Some(read);
            } else if read.role != RuntimeReadRole::ConvolutionSource {
                return Err(preparation_error(
                    "convolution contains an unrelated texture read",
                ));
            }
        }
        let requires_aux = matches!(
            operation.stage(),
            ConvolutionStage::Finalize | ConvolutionStage::Accumulate { first: false, .. }
        );
        if auxiliary.is_some() != requires_aux
            || request.reads.len() != if requires_aux { 2 } else { 1 }
        {
            return Err(preparation_error(
                "convolution texture roles differ from the exact stage",
            ));
        }
        let auxiliary = auxiliary.unwrap_or(source);
        let RuntimeResultBinding::Resource(target) = request.result else {
            return Err(preparation_error(
                "convolution has no concrete texture target",
            ));
        };
        let source_binding = self.texture_binding_for_pass(request.id, source.resource, account)?;
        self.validate_texture_binding(&source_binding, source.resource, account)?;
        let auxiliary_binding =
            self.texture_binding_for_pass(request.id, auxiliary.resource, account)?;
        self.validate_texture_binding(&auxiliary_binding, auxiliary.resource, account)?;
        let target_binding = self.texture_binding_for_pass(request.id, target, account)?;
        let target_spatial = self.validate_texture_binding(&target_binding, target, account)?;
        for binding in [&source_binding, &auxiliary_binding] {
            if binding.runtime_resource() == target
                || binding.allocation_resource() == target_binding.allocation_resource()
                || !binding
                    .texture()
                    .usage()
                    .contains(wgpu::TextureUsages::TEXTURE_BINDING)
            {
                return Err(preparation_error(
                    "convolution texture target aliases a sampled source",
                ));
            }
        }
        let target_format = match operation.stage() {
            ConvolutionStage::PassThrough
            | ConvolutionStage::Resolve
            | ConvolutionStage::RebaseSource => self.plan.working_format.texture_format(),
            _ => wgpu::TextureFormat::Rgba32Float,
        };
        if target_binding.texture().format() != target_format
            || !target_binding
                .texture()
                .usage()
                .contains(wgpu::TextureUsages::RENDER_ATTACHMENT)
        {
            return Err(preparation_error(
                "convolution target is not the exact admitted attachment format",
            ));
        }
        let parameters = request
            .convolution_parameters
            .as_ref()
            .ok_or_else(|| preparation_error("convolution has no serialized grid parameters"))?;
        if *parameters != super::super::convolution::parameters(operation, account)? {
            return Err(preparation_error(
                "convolution serialized parameters differ from the checked payload",
            ));
        }
        let owner = request
            .convolution_weights
            .ok_or_else(|| preparation_error("convolution has no coefficient owner"))?;
        let weights = self
            .acquired_bindings
            .convolution_weight_bindings
            .get(&owner, account)?
            .and_then(|binding| binding.lease.as_ref())
            .ok_or_else(|| {
                preparation_error("convolution coefficient lease is missing or stale")
            })?;
        let scope = self
            .frame_scope
            .as_ref()
            .ok_or_else(|| preparation_error("convolution resource scope is closed"))?;
        let weight_buffer = scope.convolution_weights_buffer(weights)?;
        let keys = request
            .cache_keys
            .as_ref()
            .ok_or_else(|| preparation_error("convolution cache keys are missing"))?;
        let objects: crate::shader::ProvisionalConvolutionPassObjects<'_> = self
            .pass_cache_update
            .as_ref()
            .ok_or_else(|| preparation_error("convolution provisional cache is missing"))?
            .convolution_encoding_objects(
                self.pass_cache,
                account,
                keys.samplers(),
                keys.layout(),
                keys.shader(),
                keys.pipeline(),
            )?;
        objects.require_encoding_ready()?;
        let uniform = self.resources.generation().allocation_with_upload(
            GenerationClass::PassBuffers,
            256,
            256,
            Some(account),
            || {
                self.device.create_buffer(&wgpu::BufferDescriptor {
                    label: Some("Surgeist convolution checked grid uniform"),
                    size: 256,
                    usage: wgpu::BufferUsages::UNIFORM.union(wgpu::BufferUsages::COPY_DST),
                    mapped_at_creation: false,
                })
            },
        )?;
        self.queue.write_buffer(&uniform, 0, parameters.as_bytes());
        account.work(RenderWork::ShaderBindingVisits, 4)?;
        let bindings =
            self.resources
                .generation()
                .opaque(GpuObjectKind::BindGroup, Some(account), || {
                    self.device.create_bind_group(&wgpu::BindGroupDescriptor {
                        label: Some("Surgeist convolution exact source sum and coefficients"),
                        layout: objects.bind_group_layout(),
                        entries: &[
                            wgpu::BindGroupEntry {
                                binding: 0,
                                resource: wgpu::BindingResource::TextureView(source_binding.view()),
                            },
                            wgpu::BindGroupEntry {
                                binding: 1,
                                resource: wgpu::BindingResource::TextureView(
                                    auxiliary_binding.view(),
                                ),
                            },
                            wgpu::BindGroupEntry {
                                binding: 2,
                                resource: uniform.as_entire_binding(),
                            },
                            wgpu::BindGroupEntry {
                                binding: 3,
                                resource: weight_buffer.as_entire_binding(),
                            },
                        ],
                    })
                })?;
        let region = GraphRenderRegion::full(target_spatial.device_extent)?;
        if let ConvolutionStage::Accumulate { tap_count, .. } = operation.stage() {
            let opportunities = scheduled_region_texels(region)
                .checked_mul(u64::from(tap_count))
                .ok_or(crate::RenderAccountingError::Invariant)?;
            account.work(
                RenderWork::ScheduledConvolutionTaps,
                usize::try_from(opportunities)
                    .map_err(|_| crate::RenderAccountingError::Invariant)?,
            )?;
        }
        encode_spatial_filter_full_target_pass(
            encoder,
            target_binding.view(),
            SpatialPassDrawBindings {
                #[cfg(test)]
                target_kind_for_test: SpatialPassTargetForTest::Dedicated,
                pipeline: objects.render_pipeline(),
                bind_group: &bindings,
            },
            region,
            "Surgeist convolution bounded stage",
            true,
            account,
        )
    }
}
