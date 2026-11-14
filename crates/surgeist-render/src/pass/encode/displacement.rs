use super::*;
use crate::frame::DisplacementStage;
use crate::pass::displacement::{DisplacementAllocation, DisplacementReadRole};

impl PreparedGraph<'_> {
    pub(super) fn encode_displacement(
        &self,
        encoder: &mut wgpu::CommandEncoder,
        request: &PreparedPassEncodingRequest,
        account: &Account,
    ) -> Result<()> {
        let RuntimePassKind::Displacement(runtime) = &request.kind else {
            return Err(preparation_error(
                "displacement encoding requires checked facts",
            ));
        };
        if !runtime.proves_exact(request.id, account)? {
            return Err(preparation_error(
                "displacement reads differ from the closed graph",
            ));
        }
        let work = &runtime.work;
        let map = work.mapping();
        let expected_source = map.source_spatial().is_some();
        if request.reads.len() != usize::from(expected_source) {
            return Err(preparation_error(
                "displacement has an unexpected image read",
            ));
        }
        let RuntimeResultBinding::Resource(target) = request.result else {
            return Err(preparation_error(
                "displacement requires a bounded image target",
            ));
        };
        let target_binding = self.texture_binding_for_pass(request.id, target, account)?;
        let target_spatial = self.validate_texture_binding(&target_binding, target, account)?;
        if target_spatial != RuntimeSpatialDescriptor::from_graph(map.target_spatial())
            || target_binding.texture().format() != self.plan.working_format.texture_format()
            || !target_binding
                .texture()
                .usage()
                .contains(wgpu::TextureUsages::RENDER_ATTACHMENT)
        {
            return Err(preparation_error(
                "displacement target is not the qualified working image",
            ));
        }
        let source = if expected_source {
            let read = exact_core_pass_read(request, RuntimeReadRole::DisplacementSource, account)?;
            let binding = self.texture_binding_for_pass(request.id, read.resource, account)?;
            let spatial = self.validate_texture_binding(&binding, read.resource, account)?;
            if read.resource == target
                || binding.allocation_resource() == target_binding.allocation_resource()
                || Some(spatial)
                    != map
                        .source_spatial()
                        .map(RuntimeSpatialDescriptor::from_graph)
                || binding.texture().format() != self.plan.working_format.texture_format()
                || !binding
                    .texture()
                    .usage()
                    .contains(wgpu::TextureUsages::TEXTURE_BINDING)
            {
                return Err(preparation_error(
                    "displacement source is stale, aliased, or on a different lattice",
                ));
            }
            Some(binding)
        } else {
            None
        };
        let preparation = request
            .displacement
            .as_ref()
            .ok_or_else(|| preparation_error("displacement preparation is missing"))?;
        let binding = self
            .acquired_bindings
            .displacement_bindings
            .get(&request.id, account)?
            .ok_or_else(|| preparation_error("displacement concrete custody is missing"))?;
        if binding.uniform.is_consumed()
            || !preparation.same_allocation(&binding.preparation)
            || preparation.parameters != super::super::displacement::parameters(work, account)?
            || binding.resources.len() != preparation.reads.len()
        {
            return Err(preparation_error(
                "displacement concrete uniform or upload custody changed",
            ));
        }
        let uniform = binding.uniform.active(
            request.id,
            PassBufferRole::Displacement,
            256,
            self.resources.generation(),
        )?;
        let scope = self
            .frame_scope
            .as_ref()
            .ok_or_else(|| preparation_error("displacement frame scope is closed"))?;
        let mut entries = account.temporary(2 + binding.resources.len())?;
        if let Some(source) = &source {
            entries.append_slot()?.write(wgpu::BindGroupEntry {
                binding: 0,
                resource: wgpu::BindingResource::TextureView(source.view()),
            });
        }
        entries.append_slot()?.write(wgpu::BindGroupEntry {
            binding: 2,
            resource: uniform.as_entire_binding(),
        });
        for (actual, expected) in binding.resources.iter().zip(preparation.reads.iter()) {
            account.work(RenderWork::GraphEdgeVisits, 1)?;
            if actual.read.read.id != expected.read.id
                || actual.read.read.expected_reads != 1
                || actual.read.read.last_use != request.id
            {
                return Err(preparation_error(
                    "displacement concrete read identity or last use changed",
                ));
            }
            let lease = actual
                .lease
                .as_ref()
                .ok_or_else(|| preparation_error("displacement read lease is stale"))?;
            if lease.resource_identity() == target_binding.allocation_resource()
                || source
                    .as_ref()
                    .is_some_and(|source| source.allocation_resource() == lease.resource_identity())
            {
                return Err(preparation_error(
                    "displacement source and auxiliary resources alias",
                ));
            }
            let entry = match (&expected.read.id.role, &expected.allocation) {
                (DisplacementReadRole::RawMap, DisplacementAllocation::RawMap(upload)) => {
                    let (texture, view) = scope.raw_image_texture(lease, upload)?;
                    let size = upload.physical_size();
                    if texture.format() != wgpu::TextureFormat::Rgba8Unorm
                        || texture.width() != size.width()
                        || texture.height() != size.height()
                        || !texture.usage().contains(
                            wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST,
                        )
                    {
                        return Err(preparation_error(
                            "raw displacement map allocation differs from its descriptor",
                        ));
                    }
                    wgpu::BindGroupEntry {
                        binding: 1,
                        resource: wgpu::BindingResource::TextureView(view),
                    }
                }
                (
                    role @ (DisplacementReadRole::AddressTables | DisplacementReadRole::MapRecords),
                    DisplacementAllocation::Buffer(plan),
                ) => {
                    if plan.kind()
                        != if *role == DisplacementReadRole::AddressTables {
                            crate::resource::DisplacementBufferKind::AddressTables
                        } else {
                            crate::resource::DisplacementBufferKind::MapRecords
                        }
                    {
                        return Err(preparation_error(
                            "displacement buffer namespace differs from its read role",
                        ));
                    }
                    let buffer = scope.displacement_buffer(lease, plan)?;
                    if buffer.size() != plan.byte_len()
                        || !buffer
                            .usage()
                            .contains(wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_DST)
                    {
                        return Err(preparation_error(
                            "displacement storage allocation differs from its descriptor",
                        ));
                    }
                    wgpu::BindGroupEntry {
                        binding: if *role == DisplacementReadRole::AddressTables {
                            3
                        } else {
                            4
                        },
                        resource: buffer.as_entire_binding(),
                    }
                }
                _ => {
                    return Err(preparation_error(
                        "displacement concrete resource role changed",
                    ));
                }
            };
            entries.append_slot()?.write(entry);
        }
        let keys = request
            .cache_keys
            .as_ref()
            .ok_or_else(|| preparation_error("displacement shader keys are missing"))?;
        let objects: crate::shader::ProvisionalDisplacementPassObjects<'_> = self
            .pass_cache_update
            .as_ref()
            .ok_or_else(|| preparation_error("displacement cache is not provisional"))?
            .displacement_encoding_objects(
                self.pass_cache,
                account,
                keys.samplers(),
                keys.layout(),
                keys.shader(),
                keys.pipeline(),
            )?;
        objects.require_encoding_ready()?;
        account.work(RenderWork::ShaderBindingVisits, entries.len())?;
        let group =
            self.resources
                .generation()
                .opaque(GpuObjectKind::BindGroup, Some(account), || {
                    self.device.create_bind_group(&wgpu::BindGroupDescriptor {
                        label: Some("Surgeist displacement real source map tables and records"),
                        layout: objects.bind_group_layout(),
                        entries: &entries,
                    })
                })?;
        let region = GraphRenderRegion::full(target_spatial.device_extent)?;
        let cells = scheduled_region_texels(region);
        if work.stage() == DisplacementStage::Displace {
            account.work_u64(
                RenderWork::ScheduledDisplacementAddressReads,
                cells
                    .checked_mul(2)
                    .ok_or(crate::RenderAccountingError::Invariant)?,
            )?;
            account.work_u64(RenderWork::ScheduledDisplacementSourceSamples, cells)?;
            if !work.map_is_zero() {
                account.work_u64(
                    RenderWork::ScheduledDisplacementMapRecordReads,
                    cells
                        .checked_mul(2)
                        .ok_or(crate::RenderAccountingError::Invariant)?,
                )?;
                account.work_u64(RenderWork::ScheduledDisplacementMapSamples, cells)?;
            }
        }
        encode_spatial_filter_full_target_pass(
            encoder,
            target_binding.view(),
            SpatialPassDrawBindings {
                #[cfg(test)]
                target_kind_for_test: SpatialPassTargetForTest::Dedicated,
                pipeline: objects.render_pipeline(),
                bind_group: &group,
            },
            region,
            "Surgeist bounded native displacement",
            true,
            account,
        )
    }
}
