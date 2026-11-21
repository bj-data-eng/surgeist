use super::*;
use crate::shader::{
    pipeline::{
        create_lighting_bind_group_layout, create_lighting_render_pipeline,
        create_lighting_shader_module,
    },
    validate::{LightingPassKeyRefs, lighting_cache_error, validate_lighting_pass_keys},
};

pub(crate) struct ProvisionalLightingPassObjects<'a> {
    pub(crate) layout: &'a wgpu::BindGroupLayout,
    pub(crate) pipeline: &'a wgpu::RenderPipeline,
}

impl ProvisionalDevicePassCacheUpdate {
    pub(crate) fn realize_lighting_pass(
        &mut self,
        device: &wgpu::Device,
        cache: &DevicePassCache,
        account: &Account,
        keys: LightingPassKeyRefs<'_>,
    ) -> Result<()> {
        self.require_lighting_cache_owner(cache)?;
        let description = validate_lighting_pass_keys(keys, Some(account))?;
        if !contains(&cache.layouts, keys.layout, account)?
            && !contains(&self.layouts, keys.layout, account)?
        {
            self.layouts.slot(&cache.cpu)?.write(
                keys.layout.clone(),
                create_lighting_bind_group_layout(
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
            self.shaders.slot(&cache.cpu)?.write(
                keys.shader.clone(),
                create_lighting_shader_module(
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
                .ok_or_else(|| lighting_cache_error("lighting layout realization was lost"))?;
            let shader = lookup_pair(&self.shaders, &cache.shaders, keys.shader, account)?
                .ok_or_else(|| lighting_cache_error("lighting shader realization was lost"))?;
            slot.write(
                keys.pipeline.clone(),
                create_lighting_render_pipeline(
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
        Ok(())
    }

    pub(crate) fn lighting_encoding_objects<'a>(
        &'a self,
        cache: &'a DevicePassCache,
        account: &Account,
        keys: LightingPassKeyRefs<'_>,
        stage: crate::shader::ShaderLightingStageKey,
    ) -> Result<ProvisionalLightingPassObjects<'a>> {
        self.require_lighting_cache_owner(cache)?;
        if validate_lighting_pass_keys(keys, Some(account))?.stage != stage {
            return Err(lighting_cache_error(
                "lighting objects differ from the exact pass stage",
            ));
        }
        let layout = lookup_pair(&self.layouts, &cache.layouts, keys.layout, account)?
            .ok_or_else(|| lighting_cache_error("lighting layout realization was lost"))?;
        let pipeline = lookup_pair(&self.pipelines, &cache.pipelines, keys.pipeline, account)?
            .ok_or_else(|| lighting_cache_error("lighting pipeline realization was lost"))?;
        Ok(ProvisionalLightingPassObjects { layout, pipeline })
    }

    fn require_lighting_cache_owner(&self, cache: &DevicePassCache) -> Result<()> {
        if !Arc::ptr_eq(&self.cache_identity, &cache.identity) {
            return Err(lighting_cache_error(
                "provisional lighting objects belong to another device cache",
            ));
        }
        Ok(())
    }
}
