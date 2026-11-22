use super::*;
use crate::shader::{
    pipeline::{
        create_turbulence_bind_group_layout, create_turbulence_render_pipeline,
        create_turbulence_shader_module,
    },
    validate::{TurbulencePassKeyRefs, turbulence_cache_error, validate_turbulence_pass_keys},
};

pub(crate) struct ProvisionalTurbulencePassObjects<'a> {
    pub(crate) layout: &'a wgpu::BindGroupLayout,
    pub(crate) pipeline: &'a wgpu::RenderPipeline,
}

impl ProvisionalDevicePassCacheUpdate {
    pub(crate) fn realize_turbulence_pass(
        &mut self,
        device: &wgpu::Device,
        cache: &DevicePassCache,
        account: &Account,
        keys: TurbulencePassKeyRefs<'_>,
    ) -> Result<()> {
        self.require_turbulence_cache_owner(cache)?;
        let target_format = validate_turbulence_pass_keys(keys, Some(account))?;
        if !contains(&cache.layouts, keys.layout, account)?
            && !contains(&self.layouts, keys.layout, account)?
        {
            self.layouts.slot(&cache.cpu)?.write(
                keys.layout.clone(),
                create_turbulence_bind_group_layout(&cache.generation, account, device)?,
            );
        }
        if !contains(&cache.shaders, keys.shader, account)?
            && !contains(&self.shaders, keys.shader, account)?
        {
            self.shaders.slot(&cache.cpu)?.write(
                keys.shader.clone(),
                create_turbulence_shader_module(&cache.generation, account, device)?,
            );
        }
        if !contains(&cache.pipelines, keys.pipeline, account)?
            && !contains(&self.pipelines, keys.pipeline, account)?
        {
            let slot = self.pipelines.slot(&cache.cpu)?;
            let layout = lookup_pair(&self.layouts, &cache.layouts, keys.layout, account)?
                .ok_or_else(|| turbulence_cache_error("turbulence layout realization was lost"))?;
            let shader = lookup_pair(&self.shaders, &cache.shaders, keys.shader, account)?
                .ok_or_else(|| turbulence_cache_error("turbulence shader realization was lost"))?;
            slot.write(
                keys.pipeline.clone(),
                create_turbulence_render_pipeline(
                    &cache.generation,
                    account,
                    device,
                    target_format,
                    layout,
                    shader,
                )?,
            );
        }
        Ok(())
    }

    pub(crate) fn turbulence_encoding_objects<'a>(
        &'a self,
        cache: &'a DevicePassCache,
        account: &Account,
        keys: TurbulencePassKeyRefs<'_>,
        target_format: crate::shader::ShaderTextureFormatKey,
    ) -> Result<ProvisionalTurbulencePassObjects<'a>> {
        self.require_turbulence_cache_owner(cache)?;
        if validate_turbulence_pass_keys(keys, Some(account))? != target_format {
            return Err(turbulence_cache_error(
                "turbulence pipeline differs from its admitted working format",
            ));
        }
        let layout = lookup_pair(&self.layouts, &cache.layouts, keys.layout, account)?
            .ok_or_else(|| turbulence_cache_error("turbulence layout realization was lost"))?;
        let pipeline = lookup_pair(&self.pipelines, &cache.pipelines, keys.pipeline, account)?
            .ok_or_else(|| turbulence_cache_error("turbulence pipeline realization was lost"))?;
        Ok(ProvisionalTurbulencePassObjects { layout, pipeline })
    }

    fn require_turbulence_cache_owner(&self, cache: &DevicePassCache) -> Result<()> {
        if !Arc::ptr_eq(&self.cache_identity, &cache.identity) {
            return Err(turbulence_cache_error(
                "provisional turbulence objects belong to another device cache",
            ));
        }
        Ok(())
    }
}
