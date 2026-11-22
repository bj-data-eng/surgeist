use super::*;
use crate::shader::{
    ShaderTextureFormatKey, ShaderTileStageKey,
    pipeline::{
        create_tile_bind_group_layout, create_tile_render_pipeline, create_tile_shader_module,
    },
    validate::{TilePassDescription, TilePassKeyRefs, tile_cache_error, validate_tile_pass_keys},
};

pub(crate) struct ProvisionalTilePassObjects<'a> {
    pub(crate) layout: Option<&'a wgpu::BindGroupLayout>,
    pub(crate) pipeline: &'a wgpu::RenderPipeline,
}

impl ProvisionalDevicePassCacheUpdate {
    pub(crate) fn realize_tile_pass(
        &mut self,
        device: &wgpu::Device,
        cache: &DevicePassCache,
        account: &Account,
        keys: TilePassKeyRefs<'_>,
    ) -> Result<()> {
        let description = self.checked_tile_description(cache, account, keys)?;
        if description.stage.reads_source()
            && !contains(&cache.layouts, keys.layout, account)?
            && !contains(&self.layouts, keys.layout, account)?
        {
            self.layouts.slot(&cache.cpu)?.write(
                keys.layout.clone(),
                create_tile_bind_group_layout(&cache.generation, account, device)?,
            );
        }
        if !contains(&cache.shaders, keys.shader, account)?
            && !contains(&self.shaders, keys.shader, account)?
        {
            self.shaders.slot(&cache.cpu)?.write(
                keys.shader.clone(),
                create_tile_shader_module(&cache.generation, account, device)?,
            );
        }
        if !contains(&cache.pipelines, keys.pipeline, account)?
            && !contains(&self.pipelines, keys.pipeline, account)?
        {
            let slot = self.pipelines.slot(&cache.cpu)?;
            let layout = tile_layout(
                &self.layouts,
                cache,
                account,
                keys.layout,
                description.stage,
            )?;
            let shader = lookup_pair(&self.shaders, &cache.shaders, keys.shader, account)?
                .ok_or_else(|| tile_cache_error("tile shader realization was lost"))?;
            let pipeline = create_tile_render_pipeline(
                &cache.generation,
                account,
                device,
                description,
                layout,
                shader,
            )?;
            slot.write(keys.pipeline.clone(), pipeline);
        }
        Ok(())
    }

    pub(crate) fn tile_encoding_objects<'a>(
        &'a self,
        cache: &'a DevicePassCache,
        account: &Account,
        keys: TilePassKeyRefs<'_>,
        stage: ShaderTileStageKey,
        target_format: ShaderTextureFormatKey,
    ) -> Result<ProvisionalTilePassObjects<'a>> {
        let description = self.checked_tile_description(cache, account, keys)?;
        if description
            != (TilePassDescription {
                stage,
                target_format,
            })
        {
            return Err(tile_cache_error(
                "tile objects differ from the admitted stage or working format",
            ));
        }
        let layout = tile_layout(&self.layouts, cache, account, keys.layout, stage)?;
        let pipeline = lookup_pair(&self.pipelines, &cache.pipelines, keys.pipeline, account)?
            .ok_or_else(|| tile_cache_error("tile pipeline realization was lost"))?;
        Ok(ProvisionalTilePassObjects { layout, pipeline })
    }

    fn checked_tile_description(
        &self,
        cache: &DevicePassCache,
        account: &Account,
        keys: TilePassKeyRefs<'_>,
    ) -> Result<TilePassDescription> {
        if !Arc::ptr_eq(&self.cache_identity, &cache.identity) {
            return Err(tile_cache_error(
                "provisional tile objects belong to another device cache",
            ));
        }
        validate_tile_pass_keys(keys, Some(account))
    }
}

fn tile_layout<'a>(
    pending: &'a CacheIndex<BindGroupLayoutKey, OwnedGpu<wgpu::BindGroupLayout>>,
    cache: &'a DevicePassCache,
    account: &Account,
    key: &BindGroupLayoutKey,
    stage: ShaderTileStageKey,
) -> Result<Option<&'a wgpu::BindGroupLayout>> {
    if !stage.reads_source() {
        return Ok(None);
    }
    let layout = lookup_pair(pending, &cache.layouts, key, account)?
        .ok_or_else(|| tile_cache_error("tile layout realization was lost"))?;
    Ok(Some(layout))
}
