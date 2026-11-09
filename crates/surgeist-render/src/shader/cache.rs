use std::{collections::HashMap, sync::Arc};

use crate::{
    Result,
    accounting::{Account, CpuAccount, Generation, OwnedGpu, RenderResource, RenderWork},
};

use super::key::{
    BindGroupLayoutKey, RenderPipelineKey, SamplerKey, ShaderCompositePathKey, ShaderModuleKey,
};
use super::pipeline::{
    create_blur_bind_group_layout, create_blur_render_pipeline, create_blur_shader_module,
    create_color_filter_bind_group_layout, create_color_filter_render_pipeline,
    create_color_filter_shader_module, create_composite_bind_group_layout,
    create_composite_render_pipeline, create_composite_shader_module,
    create_copy_backdrop_bind_group_layout, create_copy_backdrop_pipeline,
    create_copy_backdrop_shader_module, create_core_pass_bind_group_layout,
    create_core_pass_render_pipeline, create_core_pass_shader_module,
    create_drop_shadow_colorize_bind_group_layout, create_drop_shadow_colorize_render_pipeline,
    create_drop_shadow_colorize_shader_module, create_sampler,
};
use super::validate::{
    BlurPassDescription, BlurPassKeyRefs, ColorFilterPassDescription, ColorFilterPassKeyRefs,
    CompositePassDescription, CompositePassKeyRefs, CopyBackdropPassDescription,
    CopyBackdropPassKeyRefs, CorePassKeyRefs, CorePassProgram, DropShadowColorizePassDescription,
    DropShadowColorizePassKeyRefs, blur_cache_error, color_filter_cache_error,
    copy_backdrop_cache_error, core_pass_cache_error, drop_shadow_cache_error,
    validate_blur_pass_keys, validate_color_filter_pass_keys, validate_composite_pass_keys,
    validate_copy_backdrop_pass_keys, validate_core_pass_keys,
    validate_drop_shadow_colorize_pass_keys,
};

/// Non-clone handles created inside one checked GPU-operation scope. New entries
/// remain private to this phase until the caller explicitly commits after the
/// owning transaction resolves cleanly.
#[must_use = "provisional pass-cache handles must be committed only after checked success"]
pub(crate) struct ProvisionalDevicePassCacheUpdate {
    pub(super) cache_identity: Arc<()>,
    commit_ready: bool,
    pub(super) samplers: HashMap<SamplerKey, OwnedGpu<wgpu::Sampler>>,
    pub(super) layouts: HashMap<BindGroupLayoutKey, OwnedGpu<wgpu::BindGroupLayout>>,
    pub(super) shaders: HashMap<ShaderModuleKey, OwnedGpu<wgpu::ShaderModule>>,
    pub(super) pipelines: HashMap<RenderPipelineKey, OwnedGpu<wgpu::RenderPipeline>>,
}

/// Borrowed base graph objects that are ready for later bind-group creation and pass
/// encoding even while newly created handles remain provisional.
pub(crate) struct ProvisionalCorePassObjects<'a> {
    program: CorePassProgram,
    samplers: [&'a OwnedGpu<wgpu::Sampler>; 1],
    layout: &'a OwnedGpu<wgpu::BindGroupLayout>,
    shader: &'a OwnedGpu<wgpu::ShaderModule>,
    pipeline: &'a OwnedGpu<wgpu::RenderPipeline>,
}

/// Borrowed layer-compositor objects that remain provisional until the owning
/// checked transaction resolves successfully.
pub(crate) struct ProvisionalCompositePassObjects<'a> {
    description: CompositePassDescription,
    source_sampler: &'a OwnedGpu<wgpu::Sampler>,
    layout: &'a OwnedGpu<wgpu::BindGroupLayout>,
    shader: &'a OwnedGpu<wgpu::ShaderModule>,
    pipeline: &'a OwnedGpu<wgpu::RenderPipeline>,
}

/// Borrowed ColorFilter color-filter objects that remain provisional until the owning
/// checked GPU operation resolves successfully.
pub(crate) struct ProvisionalColorFilterPassObjects<'a> {
    description: ColorFilterPassDescription,
    source_sampler: &'a OwnedGpu<wgpu::Sampler>,
    layout: &'a OwnedGpu<wgpu::BindGroupLayout>,
    shader: &'a OwnedGpu<wgpu::ShaderModule>,
    pipeline: &'a OwnedGpu<wgpu::RenderPipeline>,
}

/// Borrowed Backdrop backdrop-copy objects selected entirely by checked cache facts.
pub(crate) struct ProvisionalCopyBackdropPassObjects<'a> {
    description: CopyBackdropPassDescription,
    parent_sampler: &'a OwnedGpu<wgpu::Sampler>,
    layout: &'a OwnedGpu<wgpu::BindGroupLayout>,
    shader: &'a OwnedGpu<wgpu::ShaderModule>,
    pipeline: &'a OwnedGpu<wgpu::RenderPipeline>,
}

/// Borrowed SpatialFilter blur objects selected entirely by checked cache-key facts.
pub(crate) struct ProvisionalBlurPassObjects<'a> {
    description: BlurPassDescription,
    source_sampler: &'a OwnedGpu<wgpu::Sampler>,
    layout: &'a OwnedGpu<wgpu::BindGroupLayout>,
    shader: &'a OwnedGpu<wgpu::ShaderModule>,
    pipeline: &'a OwnedGpu<wgpu::RenderPipeline>,
}

/// Borrowed SpatialFilter drop-shadow colorize objects selected by checked key facts.
pub(crate) struct ProvisionalDropShadowColorizePassObjects<'a> {
    description: DropShadowColorizePassDescription,
    source_sampler: &'a OwnedGpu<wgpu::Sampler>,
    layout: &'a OwnedGpu<wgpu::BindGroupLayout>,
    shader: &'a OwnedGpu<wgpu::ShaderModule>,
    pipeline: &'a OwnedGpu<wgpu::RenderPipeline>,
}

/// Device-lifetime ownership for the four exact custom-pass WGPU handle spaces.
pub(crate) struct DevicePassCache {
    pub(super) generation: Generation,
    pub(super) cpu: CpuAccount,
    pub(super) identity: Arc<()>,
    pub(super) samplers: HashMap<SamplerKey, OwnedGpu<wgpu::Sampler>>,
    pub(super) layouts: HashMap<BindGroupLayoutKey, OwnedGpu<wgpu::BindGroupLayout>>,
    pub(super) shaders: HashMap<ShaderModuleKey, OwnedGpu<wgpu::ShaderModule>>,
    pub(super) pipelines: HashMap<RenderPipelineKey, OwnedGpu<wgpu::RenderPipeline>>,
}

impl DevicePassCache {
    #[must_use]
    pub(crate) fn new(generation: Generation, cpu: CpuAccount) -> Self {
        Self {
            generation,
            cpu,
            identity: Arc::new(()),
            samplers: HashMap::new(),
            layouts: HashMap::new(),
            shaders: HashMap::new(),
            pipelines: HashMap::new(),
        }
    }

    pub(crate) fn clear(&mut self) {
        self.samplers.clear();
        self.layouts.clear();
        self.shaders.clear();
        self.pipelines.clear();
    }

    pub(crate) fn provisional_update(&self) -> ProvisionalDevicePassCacheUpdate {
        ProvisionalDevicePassCacheUpdate {
            cache_identity: Arc::clone(&self.identity),
            commit_ready: false,
            samplers: HashMap::new(),
            layouts: HashMap::new(),
            shaders: HashMap::new(),
            pipelines: HashMap::new(),
        }
    }
}

impl ProvisionalDevicePassCacheUpdate {
    pub(crate) fn realize_core_pass<'a>(
        &'a mut self,
        device: &wgpu::Device,
        cache: &'a DevicePassCache,
        account: &Account,
        keys: CorePassKeyRefs<'_>,
    ) -> Result<ProvisionalCorePassObjects<'a>> {
        self.realize_core_pass_with_fragment_entry(device, cache, account, keys, "fragment_main")
    }

    pub(super) fn realize_core_pass_with_fragment_entry<'a>(
        &'a mut self,
        device: &wgpu::Device,
        cache: &'a DevicePassCache,
        account: &Account,
        keys: CorePassKeyRefs<'_>,
        fragment_entry: &'static str,
    ) -> Result<ProvisionalCorePassObjects<'a>> {
        if !Arc::ptr_eq(&self.cache_identity, &cache.identity) {
            return Err(core_pass_cache_error(
                "provisional core pass objects belong to another device cache",
            ));
        }
        self.commit_ready = false;
        let description = validate_core_pass_keys(keys, Some(account))?;

        for sampler_key in keys.samplers {
            if !contains(&cache.samplers, sampler_key, account)?
                && !contains(&self.samplers, sampler_key, account)?
            {
                reserve_map(&mut self.samplers, 1, &cache.cpu)?;
                self.samplers.insert(
                    *sampler_key,
                    create_sampler(
                        &cache.generation,
                        &cache.cpu,
                        Some(account),
                        device,
                        *sampler_key,
                    )?,
                );
            }
        }
        if !contains(&cache.layouts, keys.layout, account)?
            && !contains(&self.layouts, keys.layout, account)?
        {
            reserve_map(&mut self.layouts, 1, &cache.cpu)?;
            self.layouts.insert(
                keys.layout.clone(),
                create_core_pass_bind_group_layout(
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
            reserve_map(&mut self.shaders, 1, &cache.cpu)?;
            self.shaders.insert(
                keys.shader.clone(),
                create_core_pass_shader_module(
                    &cache.generation,
                    &cache.cpu,
                    Some(account),
                    device,
                    description,
                )?,
            );
        }
        if !contains(&cache.pipelines, keys.pipeline, account)?
            && !contains(&self.pipelines, keys.pipeline, account)?
        {
            reserve_map(&mut self.pipelines, 1, &cache.cpu)?;
            let layout_handle = lookup_pair(&self.layouts, &cache.layouts, keys.layout, account)?
                .ok_or_else(|| {
                core_pass_cache_error("core-pass bind-group layout realization was lost")
            })?;
            let shader_handle = lookup_pair(&self.shaders, &cache.shaders, keys.shader, account)?
                .ok_or_else(|| {
                core_pass_cache_error("core-pass shader-module realization was lost")
            })?;
            let created = create_core_pass_render_pipeline(
                &cache.generation,
                Some(account),
                device,
                description,
                layout_handle,
                shader_handle,
                fragment_entry,
            )?;
            self.pipelines.insert(keys.pipeline.clone(), created);
        }

        self.pass_objects(cache, account, keys)
    }

    pub(super) fn pass_objects<'a>(
        &'a self,
        cache: &'a DevicePassCache,
        account: &Account,
        keys: CorePassKeyRefs<'_>,
    ) -> Result<ProvisionalCorePassObjects<'a>> {
        let [key] = keys.samplers else {
            return Err(core_pass_cache_error("base graph requires one sampler"));
        };
        let samplers = [lookup_pair(&self.samplers, &cache.samplers, key, account)?
            .ok_or_else(|| core_pass_cache_error("base graph sampler realization was lost"))?];
        let layout =
            lookup_pair(&self.layouts, &cache.layouts, keys.layout, account)?.ok_or_else(|| {
                core_pass_cache_error("core-pass bind-group layout realization was lost")
            })?;
        let shader = lookup_pair(&self.shaders, &cache.shaders, keys.shader, account)?
            .ok_or_else(|| core_pass_cache_error("core-pass shader-module realization was lost"))?;
        let pipeline = lookup_pair(&self.pipelines, &cache.pipelines, keys.pipeline, account)?
            .ok_or_else(|| core_pass_cache_error("graph render-pipeline realization was lost"))?;
        Ok(ProvisionalCorePassObjects {
            program: validate_core_pass_keys(keys, Some(account))?.program,
            samplers,
            layout,
            shader,
            pipeline,
        })
    }

    pub(crate) fn encoding_objects<'a>(
        &'a self,
        cache: &'a DevicePassCache,
        account: &Account,
        samplers: &[SamplerKey],
        layout: &BindGroupLayoutKey,
        shader: &ShaderModuleKey,
        pipeline: &RenderPipelineKey,
    ) -> Result<ProvisionalCorePassObjects<'a>> {
        self.pass_objects(
            cache,
            account,
            CorePassKeyRefs {
                samplers,
                layout,
                shader,
                pipeline,
            },
        )
    }

    pub(crate) fn realize_copy_backdrop_pass<'a>(
        &'a mut self,
        device: &wgpu::Device,
        cache: &'a DevicePassCache,
        account: &Account,
        keys: CopyBackdropPassKeyRefs<'_>,
    ) -> Result<ProvisionalCopyBackdropPassObjects<'a>> {
        if !Arc::ptr_eq(&self.cache_identity, &cache.identity) {
            return Err(copy_backdrop_cache_error(
                "provisional backdrop-copy objects belong to another device cache",
            ));
        }
        self.commit_ready = false;
        let description = validate_copy_backdrop_pass_keys(keys, Some(account))?;
        for sampler_key in keys.samplers {
            if !contains(&cache.samplers, sampler_key, account)?
                && !contains(&self.samplers, sampler_key, account)?
            {
                reserve_map(&mut self.samplers, 1, &cache.cpu)?;
                self.samplers.insert(
                    *sampler_key,
                    create_sampler(
                        &cache.generation,
                        &cache.cpu,
                        Some(account),
                        device,
                        *sampler_key,
                    )?,
                );
            }
        }
        if !contains(&cache.layouts, keys.layout, account)?
            && !contains(&self.layouts, keys.layout, account)?
        {
            reserve_map(&mut self.layouts, 1, &cache.cpu)?;
            self.layouts.insert(
                keys.layout.clone(),
                create_copy_backdrop_bind_group_layout(
                    &cache.generation,
                    &cache.cpu,
                    Some(account),
                    device,
                )?,
            );
        }
        if !contains(&cache.shaders, keys.shader, account)?
            && !contains(&self.shaders, keys.shader, account)?
        {
            reserve_map(&mut self.shaders, 1, &cache.cpu)?;
            self.shaders.insert(
                keys.shader.clone(),
                create_copy_backdrop_shader_module(
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
            reserve_map(&mut self.pipelines, 1, &cache.cpu)?;
            let created = create_copy_backdrop_pipeline(
                &cache.generation,
                &cache.cpu,
                Some(account),
                device,
                description,
                lookup_pair(&self.layouts, &cache.layouts, keys.layout, account)?.ok_or_else(
                    || {
                        copy_backdrop_cache_error(
                            "backdrop-copy bind-group layout realization was lost",
                        )
                    },
                )?,
                lookup_pair(&self.shaders, &cache.shaders, keys.shader, account)?.ok_or_else(
                    || {
                        copy_backdrop_cache_error(
                            "backdrop-copy shader-module realization was lost",
                        )
                    },
                )?,
            )?;
            self.pipelines.insert(keys.pipeline.clone(), created);
        }
        self.copy_backdrop_pass_objects(cache, account, keys)
    }

    fn copy_backdrop_pass_objects<'a>(
        &'a self,
        cache: &'a DevicePassCache,
        account: &Account,
        keys: CopyBackdropPassKeyRefs<'_>,
    ) -> Result<ProvisionalCopyBackdropPassObjects<'a>> {
        let [parent_sampler_key] = keys.samplers else {
            return Err(copy_backdrop_cache_error(
                "backdrop-copy realization requires one parent sampler",
            ));
        };
        let parent_sampler =
            lookup_pair(&self.samplers, &cache.samplers, parent_sampler_key, account)?.ok_or_else(
                || copy_backdrop_cache_error("backdrop-copy sampler realization was lost"),
            )?;
        let layout =
            lookup_pair(&self.layouts, &cache.layouts, keys.layout, account)?.ok_or_else(|| {
                copy_backdrop_cache_error("backdrop-copy layout realization was lost")
            })?;
        let shader =
            lookup_pair(&self.shaders, &cache.shaders, keys.shader, account)?.ok_or_else(|| {
                copy_backdrop_cache_error("backdrop-copy shader realization was lost")
            })?;
        let pipeline = lookup_pair(&self.pipelines, &cache.pipelines, keys.pipeline, account)?
            .ok_or_else(|| {
                copy_backdrop_cache_error("backdrop-copy pipeline realization was lost")
            })?;
        Ok(ProvisionalCopyBackdropPassObjects {
            description: validate_copy_backdrop_pass_keys(keys, Some(account))?,
            parent_sampler,
            layout,
            shader,
            pipeline,
        })
    }

    pub(crate) fn realize_color_filter_pass<'a>(
        &'a mut self,
        device: &wgpu::Device,
        cache: &'a DevicePassCache,
        account: &Account,
        keys: ColorFilterPassKeyRefs<'_>,
    ) -> Result<ProvisionalColorFilterPassObjects<'a>> {
        if !Arc::ptr_eq(&self.cache_identity, &cache.identity) {
            return Err(color_filter_cache_error(
                "provisional color-filter objects belong to another device cache",
            ));
        }
        self.commit_ready = false;
        let description = validate_color_filter_pass_keys(keys, Some(account))?;
        for sampler_key in keys.samplers {
            if !contains(&cache.samplers, sampler_key, account)?
                && !contains(&self.samplers, sampler_key, account)?
            {
                reserve_map(&mut self.samplers, 1, &cache.cpu)?;
                self.samplers.insert(
                    *sampler_key,
                    create_sampler(
                        &cache.generation,
                        &cache.cpu,
                        Some(account),
                        device,
                        *sampler_key,
                    )?,
                );
            }
        }
        if !contains(&cache.layouts, keys.layout, account)?
            && !contains(&self.layouts, keys.layout, account)?
        {
            reserve_map(&mut self.layouts, 1, &cache.cpu)?;
            self.layouts.insert(
                keys.layout.clone(),
                create_color_filter_bind_group_layout(
                    &cache.generation,
                    &cache.cpu,
                    Some(account),
                    device,
                )?,
            );
        }
        if !contains(&cache.shaders, keys.shader, account)?
            && !contains(&self.shaders, keys.shader, account)?
        {
            reserve_map(&mut self.shaders, 1, &cache.cpu)?;
            self.shaders.insert(
                keys.shader.clone(),
                create_color_filter_shader_module(
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
            reserve_map(&mut self.pipelines, 1, &cache.cpu)?;
            let layout_handle = lookup_pair(&self.layouts, &cache.layouts, keys.layout, account)?
                .ok_or_else(|| {
                color_filter_cache_error("color-filter bind-group layout realization was lost")
            })?;
            let shader_handle = lookup_pair(&self.shaders, &cache.shaders, keys.shader, account)?
                .ok_or_else(|| {
                color_filter_cache_error("color-filter shader-module realization was lost")
            })?;
            let created = create_color_filter_render_pipeline(
                &cache.generation,
                &cache.cpu,
                Some(account),
                device,
                description,
                layout_handle,
                shader_handle,
            )?;
            self.pipelines.insert(keys.pipeline.clone(), created);
        }
        self.color_filter_pass_objects(cache, account, keys)
    }

    pub(crate) fn color_filter_encoding_objects<'a>(
        &'a self,
        cache: &'a DevicePassCache,
        account: &Account,
        samplers: &[SamplerKey],
        layout: &BindGroupLayoutKey,
        shader: &ShaderModuleKey,
        pipeline: &RenderPipelineKey,
    ) -> Result<ProvisionalColorFilterPassObjects<'a>> {
        self.color_filter_pass_objects(
            cache,
            account,
            ColorFilterPassKeyRefs {
                samplers,
                layout,
                shader,
                pipeline,
            },
        )
    }

    pub(crate) fn realize_blur_pass<'a>(
        &'a mut self,
        device: &wgpu::Device,
        cache: &'a DevicePassCache,
        account: &Account,
        keys: BlurPassKeyRefs<'_>,
    ) -> Result<ProvisionalBlurPassObjects<'a>> {
        if !Arc::ptr_eq(&self.cache_identity, &cache.identity) {
            return Err(blur_cache_error(
                "provisional blur objects belong to another device cache",
            ));
        }
        self.commit_ready = false;
        let description = validate_blur_pass_keys(keys, Some(account))?;
        for sampler_key in keys.samplers {
            if !contains(&cache.samplers, sampler_key, account)?
                && !contains(&self.samplers, sampler_key, account)?
            {
                reserve_map(&mut self.samplers, 1, &cache.cpu)?;
                self.samplers.insert(
                    *sampler_key,
                    create_sampler(
                        &cache.generation,
                        &cache.cpu,
                        Some(account),
                        device,
                        *sampler_key,
                    )?,
                );
            }
        }
        if !contains(&cache.layouts, keys.layout, account)?
            && !contains(&self.layouts, keys.layout, account)?
        {
            reserve_map(&mut self.layouts, 1, &cache.cpu)?;
            self.layouts.insert(
                keys.layout.clone(),
                create_blur_bind_group_layout(
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
            reserve_map(&mut self.shaders, 1, &cache.cpu)?;
            self.shaders.insert(
                keys.shader.clone(),
                create_blur_shader_module(&cache.generation, &cache.cpu, Some(account), device)?,
            );
        }
        if !contains(&cache.pipelines, keys.pipeline, account)?
            && !contains(&self.pipelines, keys.pipeline, account)?
        {
            reserve_map(&mut self.pipelines, 1, &cache.cpu)?;
            let layout_handle = lookup_pair(&self.layouts, &cache.layouts, keys.layout, account)?
                .ok_or_else(|| {
                blur_cache_error("blur bind-group layout realization was lost")
            })?;
            let shader_handle =
                lookup_pair(&self.shaders, &cache.shaders, keys.shader, account)?
                    .ok_or_else(|| blur_cache_error("blur shader-module realization was lost"))?;
            let created = create_blur_render_pipeline(
                &cache.generation,
                &cache.cpu,
                Some(account),
                device,
                description,
                layout_handle,
                shader_handle,
            )?;
            self.pipelines.insert(keys.pipeline.clone(), created);
        }
        self.blur_pass_objects(cache, account, keys)
    }

    pub(crate) fn realize_drop_shadow_colorize_pass<'a>(
        &'a mut self,
        device: &wgpu::Device,
        cache: &'a DevicePassCache,
        account: &Account,
        keys: DropShadowColorizePassKeyRefs<'_>,
    ) -> Result<ProvisionalDropShadowColorizePassObjects<'a>> {
        if !Arc::ptr_eq(&self.cache_identity, &cache.identity) {
            return Err(drop_shadow_cache_error(
                "provisional drop-shadow objects belong to another device cache",
            ));
        }
        self.commit_ready = false;
        let description = validate_drop_shadow_colorize_pass_keys(keys, Some(account))?;
        for sampler_key in keys.samplers {
            if !contains(&cache.samplers, sampler_key, account)?
                && !contains(&self.samplers, sampler_key, account)?
            {
                reserve_map(&mut self.samplers, 1, &cache.cpu)?;
                self.samplers.insert(
                    *sampler_key,
                    create_sampler(
                        &cache.generation,
                        &cache.cpu,
                        Some(account),
                        device,
                        *sampler_key,
                    )?,
                );
            }
        }
        if !contains(&cache.layouts, keys.layout, account)?
            && !contains(&self.layouts, keys.layout, account)?
        {
            reserve_map(&mut self.layouts, 1, &cache.cpu)?;
            self.layouts.insert(
                keys.layout.clone(),
                create_drop_shadow_colorize_bind_group_layout(
                    &cache.generation,
                    &cache.cpu,
                    Some(account),
                    device,
                )?,
            );
        }
        if !contains(&cache.shaders, keys.shader, account)?
            && !contains(&self.shaders, keys.shader, account)?
        {
            reserve_map(&mut self.shaders, 1, &cache.cpu)?;
            self.shaders.insert(
                keys.shader.clone(),
                create_drop_shadow_colorize_shader_module(
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
            reserve_map(&mut self.pipelines, 1, &cache.cpu)?;
            let layout_handle = lookup_pair(&self.layouts, &cache.layouts, keys.layout, account)?
                .ok_or_else(|| {
                drop_shadow_cache_error(
                    "drop-shadow colorize bind-group layout realization was lost",
                )
            })?;
            let shader_handle = lookup_pair(&self.shaders, &cache.shaders, keys.shader, account)?
                .ok_or_else(|| {
                drop_shadow_cache_error("drop-shadow colorize shader-module realization was lost")
            })?;
            let created = create_drop_shadow_colorize_render_pipeline(
                &cache.generation,
                &cache.cpu,
                Some(account),
                device,
                description,
                layout_handle,
                shader_handle,
            )?;
            self.pipelines.insert(keys.pipeline.clone(), created);
        }
        self.drop_shadow_colorize_pass_objects(cache, account, keys)
    }

    fn drop_shadow_colorize_pass_objects<'a>(
        &'a self,
        cache: &'a DevicePassCache,
        account: &Account,
        keys: DropShadowColorizePassKeyRefs<'_>,
    ) -> Result<ProvisionalDropShadowColorizePassObjects<'a>> {
        let [source_sampler_key] = keys.samplers else {
            return Err(drop_shadow_cache_error(
                "drop-shadow colorize realization requires one source sampler",
            ));
        };
        let source_sampler =
            lookup_pair(&self.samplers, &cache.samplers, source_sampler_key, account)?.ok_or_else(
                || drop_shadow_cache_error("drop-shadow colorize sampler realization was lost"),
            )?;
        let layout =
            lookup_pair(&self.layouts, &cache.layouts, keys.layout, account)?.ok_or_else(|| {
                drop_shadow_cache_error("drop-shadow colorize layout realization was lost")
            })?;
        let shader =
            lookup_pair(&self.shaders, &cache.shaders, keys.shader, account)?.ok_or_else(|| {
                drop_shadow_cache_error("drop-shadow colorize shader realization was lost")
            })?;
        let pipeline = lookup_pair(&self.pipelines, &cache.pipelines, keys.pipeline, account)?
            .ok_or_else(|| {
                drop_shadow_cache_error("drop-shadow colorize pipeline realization was lost")
            })?;
        Ok(ProvisionalDropShadowColorizePassObjects {
            description: validate_drop_shadow_colorize_pass_keys(keys, Some(account))?,
            source_sampler,
            layout,
            shader,
            pipeline,
        })
    }

    fn blur_pass_objects<'a>(
        &'a self,
        cache: &'a DevicePassCache,
        account: &Account,
        keys: BlurPassKeyRefs<'_>,
    ) -> Result<ProvisionalBlurPassObjects<'a>> {
        let [source_sampler_key] = keys.samplers else {
            return Err(blur_cache_error(
                "blur realization requires one source sampler",
            ));
        };
        let source_sampler =
            lookup_pair(&self.samplers, &cache.samplers, source_sampler_key, account)?
                .ok_or_else(|| blur_cache_error("blur sampler realization was lost"))?;
        let layout = lookup_pair(&self.layouts, &cache.layouts, keys.layout, account)?
            .ok_or_else(|| blur_cache_error("blur bind-group layout realization was lost"))?;
        let shader = lookup_pair(&self.shaders, &cache.shaders, keys.shader, account)?
            .ok_or_else(|| blur_cache_error("blur shader-module realization was lost"))?;
        let pipeline = lookup_pair(&self.pipelines, &cache.pipelines, keys.pipeline, account)?
            .ok_or_else(|| blur_cache_error("blur render-pipeline realization was lost"))?;
        Ok(ProvisionalBlurPassObjects {
            description: validate_blur_pass_keys(keys, Some(account))?,
            source_sampler,
            layout,
            shader,
            pipeline,
        })
    }

    fn color_filter_pass_objects<'a>(
        &'a self,
        cache: &'a DevicePassCache,
        account: &Account,
        keys: ColorFilterPassKeyRefs<'_>,
    ) -> Result<ProvisionalColorFilterPassObjects<'a>> {
        let [source_sampler_key] = keys.samplers else {
            return Err(color_filter_cache_error(
                "color-filter realization requires one source sampler",
            ));
        };
        let source_sampler =
            lookup_pair(&self.samplers, &cache.samplers, source_sampler_key, account)?.ok_or_else(
                || color_filter_cache_error("color-filter sampler realization was lost"),
            )?;
        let layout =
            lookup_pair(&self.layouts, &cache.layouts, keys.layout, account)?.ok_or_else(|| {
                color_filter_cache_error("color-filter bind-group layout realization was lost")
            })?;
        let shader =
            lookup_pair(&self.shaders, &cache.shaders, keys.shader, account)?.ok_or_else(|| {
                color_filter_cache_error("color-filter shader-module realization was lost")
            })?;
        let pipeline = lookup_pair(&self.pipelines, &cache.pipelines, keys.pipeline, account)?
            .ok_or_else(|| {
                color_filter_cache_error("color-filter render-pipeline realization was lost")
            })?;
        Ok(ProvisionalColorFilterPassObjects {
            description: validate_color_filter_pass_keys(keys, Some(account))?,
            source_sampler,
            layout,
            shader,
            pipeline,
        })
    }

    pub(crate) fn realize_composite_pass<'a>(
        &'a mut self,
        device: &wgpu::Device,
        cache: &'a DevicePassCache,
        account: &Account,
        keys: CompositePassKeyRefs<'_>,
    ) -> Result<ProvisionalCompositePassObjects<'a>> {
        self.realize_composite_pass_with_fragment_entry(device, cache, account, keys, None)
    }

    pub(super) fn realize_composite_pass_with_fragment_entry<'a>(
        &'a mut self,
        device: &wgpu::Device,
        cache: &'a DevicePassCache,
        account: &Account,
        keys: CompositePassKeyRefs<'_>,
        fragment_entry_override: Option<&'static str>,
    ) -> Result<ProvisionalCompositePassObjects<'a>> {
        if !Arc::ptr_eq(&self.cache_identity, &cache.identity) {
            return Err(core_pass_cache_error(
                "provisional composite pass objects belong to another device cache",
            ));
        }
        self.commit_ready = false;
        let description = validate_composite_pass_keys(keys, Some(account))?;
        for sampler_key in keys.samplers {
            if !contains(&cache.samplers, sampler_key, account)?
                && !contains(&self.samplers, sampler_key, account)?
            {
                reserve_map(&mut self.samplers, 1, &cache.cpu)?;
                self.samplers.insert(
                    *sampler_key,
                    create_sampler(
                        &cache.generation,
                        &cache.cpu,
                        Some(account),
                        device,
                        *sampler_key,
                    )?,
                );
            }
        }
        if !contains(&cache.layouts, keys.layout, account)?
            && !contains(&self.layouts, keys.layout, account)?
        {
            reserve_map(&mut self.layouts, 1, &cache.cpu)?;
            self.layouts.insert(
                keys.layout.clone(),
                create_composite_bind_group_layout(
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
            reserve_map(&mut self.shaders, 1, &cache.cpu)?;
            self.shaders.insert(
                keys.shader.clone(),
                create_composite_shader_module(
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
            reserve_map(&mut self.pipelines, 1, &cache.cpu)?;
            let layout_handle = lookup_pair(&self.layouts, &cache.layouts, keys.layout, account)?
                .ok_or_else(|| {
                core_pass_cache_error("composite bind-group layout realization was lost")
            })?;
            let shader_handle = lookup_pair(&self.shaders, &cache.shaders, keys.shader, account)?
                .ok_or_else(|| {
                core_pass_cache_error("composite shader-module realization was lost")
            })?;
            let created = create_composite_render_pipeline(
                &cache.generation,
                Some(account),
                device,
                description,
                layout_handle,
                shader_handle,
                fragment_entry_override,
            )?;
            self.pipelines.insert(keys.pipeline.clone(), created);
        }
        self.composite_pass_objects(cache, account, keys)
    }

    pub(super) fn composite_pass_objects<'a>(
        &'a self,
        cache: &'a DevicePassCache,
        account: &Account,
        keys: CompositePassKeyRefs<'_>,
    ) -> Result<ProvisionalCompositePassObjects<'a>> {
        let [source_sampler_key] = keys.samplers else {
            return Err(core_pass_cache_error(
                "composite pass realization requires one source sampler",
            ));
        };
        let source_sampler =
            lookup_pair(&self.samplers, &cache.samplers, source_sampler_key, account)?.ok_or_else(
                || core_pass_cache_error("composite source sampler realization was lost"),
            )?;
        let layout =
            lookup_pair(&self.layouts, &cache.layouts, keys.layout, account)?.ok_or_else(|| {
                core_pass_cache_error("composite bind-group layout realization was lost")
            })?;
        let shader = lookup_pair(&self.shaders, &cache.shaders, keys.shader, account)?
            .ok_or_else(|| core_pass_cache_error("composite shader-module realization was lost"))?;
        let pipeline = lookup_pair(&self.pipelines, &cache.pipelines, keys.pipeline, account)?
            .ok_or_else(|| {
                core_pass_cache_error("composite render-pipeline realization was lost")
            })?;
        Ok(ProvisionalCompositePassObjects {
            description: validate_composite_pass_keys(keys, Some(account))?,
            source_sampler,
            layout,
            shader,
            pipeline,
        })
    }

    pub(crate) fn composite_encoding_objects<'a>(
        &'a self,
        cache: &'a DevicePassCache,
        account: &Account,
        samplers: &[SamplerKey],
        layout: &BindGroupLayoutKey,
        shader: &ShaderModuleKey,
        pipeline: &RenderPipelineKey,
    ) -> Result<ProvisionalCompositePassObjects<'a>> {
        self.composite_pass_objects(
            cache,
            account,
            CompositePassKeyRefs {
                samplers,
                layout,
                shader,
                pipeline,
            },
        )
    }

    pub(crate) fn ensure_commit_ready(
        &mut self,
        cache: &mut DevicePassCache,
        account: &Account,
    ) -> Result<()> {
        if !Arc::ptr_eq(&self.cache_identity, &cache.identity) {
            return Err(core_pass_cache_error(
                "provisional pass objects cannot enter another device cache",
            ));
        }
        macro_rules! prepare {
            ($field:ident) => {{
                for key in self.$field.keys() {
                    if contains(&cache.$field, key, account)? {
                        return Err(core_pass_cache_error(
                            "persistent pass cache changed during provisional realization",
                        ));
                    }
                }
                reserve_map(&mut cache.$field, self.$field.len(), &cache.cpu)?;
            }};
        }
        prepare!(samplers);
        prepare!(layouts);
        prepare!(shaders);
        prepare!(pipelines);
        self.commit_ready = true;
        Ok(())
    }

    pub(crate) fn commit(self, cache: &mut DevicePassCache) {
        assert!(
            self.commit_ready && Arc::ptr_eq(&self.cache_identity, &cache.identity),
            "pass cache requires checked readiness"
        );
        assert!(
            cache.samplers.capacity() - cache.samplers.len() >= self.samplers.len()
                && cache.layouts.capacity() - cache.layouts.len() >= self.layouts.len()
                && cache.shaders.capacity() - cache.shaders.len() >= self.shaders.len()
                && cache.pipelines.capacity() - cache.pipelines.len() >= self.pipelines.len(),
            "checked pass-cache capacity changed"
        );
        // No duplicate keys and sufficient capacity were proved before host authorization.
        // Individual insertion avoids HashMap::extend making a speculative reservation.
        for (key, value) in self.samplers {
            cache.samplers.insert(key, value);
        }
        for (key, value) in self.layouts {
            cache.layouts.insert(key, value);
        }
        for (key, value) in self.shaders {
            cache.shaders.insert(key, value);
        }
        for (key, value) in self.pipelines {
            cache.pipelines.insert(key, value);
        }
    }
}

fn reserve_map<K: Eq + std::hash::Hash, V>(
    map: &mut HashMap<K, V>,
    additional: usize,
    cpu: &CpuAccount,
) -> Result<()> {
    map.try_reserve(additional)
        .map_err(|_| cpu.allocation_error(RenderResource::RetainedCpuBytes))
}
fn contains<K: Eq + std::hash::Hash, V>(
    map: &HashMap<K, V>,
    key: &K,
    account: &Account,
) -> Result<bool> {
    account.work(RenderWork::ResourceProbes, 1)?;
    Ok(map.contains_key(key))
}
fn lookup_pair<'a, K: Eq + std::hash::Hash, V>(
    first: &'a HashMap<K, V>,
    second: &'a HashMap<K, V>,
    key: &K,
    account: &Account,
) -> Result<Option<&'a V>> {
    account.work(RenderWork::ResourceProbes, 1)?;
    if let Some(value) = first.get(key) {
        return Ok(Some(value));
    }
    account.work(RenderWork::ResourceProbes, 1)?;
    Ok(second.get(key))
}

impl ProvisionalCorePassObjects<'_> {
    pub(crate) fn require_encoding_ready(&self) -> Result<()> {
        let _ = (self.layout, self.shader, self.pipeline);
        if self.samplers.len() != 1 {
            return Err(core_pass_cache_error(
                "core pass realization did not retain its exact encoding handles",
            ));
        }
        Ok(())
    }

    pub(crate) fn sampler(&self) -> Result<&OwnedGpu<wgpu::Sampler>> {
        let [sampler] = self.samplers.as_slice() else {
            return Err(core_pass_cache_error(
                "core pass encoding requires one exact sampled-image sampler",
            ));
        };
        Ok(sampler)
    }

    pub(crate) const fn bind_group_layout(&self) -> &OwnedGpu<wgpu::BindGroupLayout> {
        self.layout
    }

    pub(crate) const fn render_pipeline(&self) -> &OwnedGpu<wgpu::RenderPipeline> {
        self.pipeline
    }

    pub(crate) const fn uses_fixed_source_over_blend(&self) -> bool {
        matches!(
            self.program,
            CorePassProgram::SpanSourceOver | CorePassProgram::DropShadowMerge
        )
    }
}

impl ProvisionalCompositePassObjects<'_> {
    pub(crate) fn require_encoding_ready(&self) -> Result<()> {
        let _ = (
            self.description,
            self.layout,
            self.shader,
            self.pipeline,
            self.source_sampler,
        );
        Ok(())
    }

    pub(crate) const fn source_sampler(&self) -> &OwnedGpu<wgpu::Sampler> {
        self.source_sampler
    }

    pub(crate) const fn bind_group_layout(&self) -> &OwnedGpu<wgpu::BindGroupLayout> {
        self.layout
    }

    pub(crate) const fn render_pipeline(&self) -> &OwnedGpu<wgpu::RenderPipeline> {
        self.pipeline
    }

    pub(crate) const fn path(&self) -> ShaderCompositePathKey {
        self.description.path
    }

    pub(crate) const fn has_clip_coverage(&self) -> bool {
        self.description.has_clip_coverage
    }

    pub(crate) const fn has_alpha_mask(&self) -> bool {
        self.description.has_alpha_mask
    }

    pub(crate) const fn uses_fixed_source_over_blend(&self) -> bool {
        matches!(self.description.path, ShaderCompositePathKey::Normal)
    }

    pub(crate) const fn uses_replace_blend(&self) -> bool {
        matches!(
            self.description.path,
            ShaderCompositePathKey::DestinationSampling
        )
    }
}

impl ProvisionalColorFilterPassObjects<'_> {
    pub(crate) fn require_encoding_ready(&self) -> Result<()> {
        let _ = (
            self.description,
            self.source_sampler,
            self.layout,
            self.shader,
            self.pipeline,
        );
        Ok(())
    }

    pub(crate) const fn source_sampler(&self) -> &OwnedGpu<wgpu::Sampler> {
        self.source_sampler
    }

    pub(crate) const fn bind_group_layout(&self) -> &OwnedGpu<wgpu::BindGroupLayout> {
        self.layout
    }

    pub(crate) const fn render_pipeline(&self) -> &OwnedGpu<wgpu::RenderPipeline> {
        self.pipeline
    }
}

impl ProvisionalCopyBackdropPassObjects<'_> {
    pub(crate) fn require_encoding_ready(&self) -> Result<()> {
        let _ = (
            self.description,
            self.parent_sampler(),
            self.bind_group_layout(),
            self.shader,
            self.render_pipeline(),
        );
        Ok(())
    }

    pub(crate) const fn parent_sampler(&self) -> &OwnedGpu<wgpu::Sampler> {
        self.parent_sampler
    }

    pub(crate) const fn bind_group_layout(&self) -> &OwnedGpu<wgpu::BindGroupLayout> {
        self.layout
    }

    pub(crate) const fn render_pipeline(&self) -> &OwnedGpu<wgpu::RenderPipeline> {
        self.pipeline
    }
}

impl ProvisionalBlurPassObjects<'_> {
    pub(crate) fn require_encoding_ready(&self) -> Result<()> {
        let _ = (
            self.description,
            self.source_sampler(),
            self.bind_group_layout(),
            self.shader,
            self.render_pipeline(),
        );
        Ok(())
    }

    pub(crate) const fn source_sampler(&self) -> &OwnedGpu<wgpu::Sampler> {
        self.source_sampler
    }

    pub(crate) const fn bind_group_layout(&self) -> &OwnedGpu<wgpu::BindGroupLayout> {
        self.layout
    }

    pub(crate) const fn render_pipeline(&self) -> &OwnedGpu<wgpu::RenderPipeline> {
        self.pipeline
    }
}

impl ProvisionalDropShadowColorizePassObjects<'_> {
    pub(crate) fn require_encoding_ready(&self) -> Result<()> {
        let _ = (
            self.description,
            self.source_sampler(),
            self.bind_group_layout(),
            self.shader,
            self.render_pipeline(),
        );
        Ok(())
    }

    pub(crate) const fn source_sampler(&self) -> &OwnedGpu<wgpu::Sampler> {
        self.source_sampler
    }

    pub(crate) const fn bind_group_layout(&self) -> &OwnedGpu<wgpu::BindGroupLayout> {
        self.layout
    }

    pub(crate) const fn render_pipeline(&self) -> &OwnedGpu<wgpu::RenderPipeline> {
        self.pipeline
    }
}

#[cfg(test)]
mod accounting_tests {
    use super::*;
    use crate::{
        RenderAccountingContext, RenderAccountingOptions, RenderAttemptKind, RenderAttemptStatus,
        RenderLimits,
    };

    #[test]
    fn visible_lookup_probes_charge_the_accepted_prefix_and_retry_with_a_fresh_cap() {
        let first = HashMap::from([(1_u32, "first")]);
        let second = HashMap::from([(2_u32, "second")]);
        for (limit, expected_work, expected_status) in [
            (5, 5, RenderAttemptStatus::Refused),
            (6, 6, RenderAttemptStatus::Succeeded),
        ] {
            let limits = RenderLimits::new()
                .with_work_limit(RenderWork::ResourceProbes, limit)
                .unwrap();
            let mut context = RenderAccountingContext::new(
                RenderAccountingOptions::default().with_limits(limits),
            );
            let result = context.run(
                RenderAttemptKind::Frame,
                crate::RenderPhase::GpuPrepare,
                |account| {
                    assert!(contains(&first, &1, account)?);
                    assert_eq!(lookup_pair(&first, &second, &1, account)?, Some(&"first"));
                    assert_eq!(lookup_pair(&first, &second, &2, account)?, Some(&"second"));
                    assert_eq!(lookup_pair(&first, &second, &3, account)?, None);
                    Ok(())
                },
            );
            assert_eq!(result.is_ok(), limit == 6);
            let report = context.last_attempt().unwrap().attempt().unwrap();
            assert_eq!(report.status(), expected_status);
            assert_eq!(
                report.usage().work(RenderWork::ResourceProbes),
                expected_work
            );
        }
    }
}
