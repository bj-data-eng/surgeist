use super::*;
use crate::shader::{
    pipeline::{
        create_displacement_bind_group_layout, create_displacement_render_pipeline,
        create_displacement_shader_module,
    },
    validate::{
        DisplacementPassDescription, DisplacementPassKeyRefs, displacement_cache_error,
        validate_displacement_pass_keys,
    },
};

pub(crate) struct ProvisionalDisplacementPassObjects<'a> {
    description: DisplacementPassDescription,
    layout: &'a OwnedGpu<wgpu::BindGroupLayout>,
    shader: &'a OwnedGpu<wgpu::ShaderModule>,
    pipeline: &'a OwnedGpu<wgpu::RenderPipeline>,
}
impl ProvisionalDisplacementPassObjects<'_> {
    pub(crate) fn require_encoding_ready(&self) -> Result<()> {
        let _ = (self.description, self.layout, self.shader, self.pipeline);
        Ok(())
    }
    pub(crate) fn bind_group_layout(&self) -> &wgpu::BindGroupLayout {
        self.layout
    }
    pub(crate) fn render_pipeline(&self) -> &wgpu::RenderPipeline {
        self.pipeline
    }
}

impl ProvisionalDevicePassCacheUpdate {
    pub(crate) fn realize_displacement_pass<'a>(
        &'a mut self,
        device: &wgpu::Device,
        cache: &'a DevicePassCache,
        account: &Account,
        keys: DisplacementPassKeyRefs<'_>,
    ) -> Result<ProvisionalDisplacementPassObjects<'a>> {
        let description = validate_displacement_pass_keys(keys, Some(account))?;
        if !contains(&cache.layouts, keys.layout, account)?
            && !contains(&self.layouts, keys.layout, account)?
        {
            let slot = self.layouts.slot(&cache.cpu)?;
            slot.write(
                keys.layout.clone(),
                create_displacement_bind_group_layout(
                    &cache.generation,
                    &cache.cpu,
                    Some(account),
                    device,
                    description,
                )?,
            );
        }
        if !contains(&cache.shaders, keys.shader, account)?
            && !contains(&self.shaders, keys.shader, account)?
        {
            let slot = self.shaders.slot(&cache.cpu)?;
            slot.write(
                keys.shader.clone(),
                create_displacement_shader_module(
                    &cache.generation,
                    &cache.cpu,
                    Some(account),
                    device,
                )?,
            );
        }
        if !contains(&cache.pipelines, keys.pipeline, account)?
            && !contains(&self.pipelines, keys.pipeline, account)?
        {
            let slot = self.pipelines.slot(&cache.cpu)?;
            let layout = lookup_pair(&self.layouts, &cache.layouts, keys.layout, account)?
                .ok_or_else(|| {
                    displacement_cache_error("displacement layout realization was lost")
                })?;
            let shader = lookup_pair(&self.shaders, &cache.shaders, keys.shader, account)?
                .ok_or_else(|| {
                    displacement_cache_error("displacement shader realization was lost")
                })?;
            slot.write(
                keys.pipeline.clone(),
                create_displacement_render_pipeline(
                    &cache.generation,
                    &cache.cpu,
                    Some(account),
                    device,
                    description,
                    layout,
                    shader,
                )?,
            );
        }
        self.displacement_pass_objects(cache, account, keys)
    }
    pub(crate) fn displacement_encoding_objects<'a>(
        &'a self,
        cache: &'a DevicePassCache,
        account: &Account,
        samplers: &[SamplerKey],
        layout: &BindGroupLayoutKey,
        shader: &ShaderModuleKey,
        pipeline: &RenderPipelineKey,
    ) -> Result<ProvisionalDisplacementPassObjects<'a>> {
        self.displacement_pass_objects(
            cache,
            account,
            DisplacementPassKeyRefs {
                samplers,
                layout,
                shader,
                pipeline,
            },
        )
    }
    fn displacement_pass_objects<'a>(
        &'a self,
        cache: &'a DevicePassCache,
        account: &Account,
        keys: DisplacementPassKeyRefs<'_>,
    ) -> Result<ProvisionalDisplacementPassObjects<'a>> {
        let layout = lookup_pair(&self.layouts, &cache.layouts, keys.layout, account)?
            .ok_or_else(|| displacement_cache_error("displacement layout binding is missing"))?;
        let shader = lookup_pair(&self.shaders, &cache.shaders, keys.shader, account)?
            .ok_or_else(|| displacement_cache_error("displacement shader binding is missing"))?;
        let pipeline = lookup_pair(&self.pipelines, &cache.pipelines, keys.pipeline, account)?
            .ok_or_else(|| displacement_cache_error("displacement pipeline binding is missing"))?;
        Ok(ProvisionalDisplacementPassObjects {
            description: validate_displacement_pass_keys(keys, Some(account))?,
            layout,
            shader,
            pipeline,
        })
    }
}
