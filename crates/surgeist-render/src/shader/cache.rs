use std::{collections::HashMap, hash::Hash, ops::Deref, sync::Arc};

use crate::{
    Error, Result,
    accounting::{
        Account, CpuAccount, CpuHold, Generation, Hold, OwnedGpu, RenderResource, RenderWork,
    },
};

pub(super) struct CacheIndex<K, V> {
    values: HashMap<K, V>,
    backing: Option<CpuHold>,
}

impl<K: Eq + Hash, V> CacheIndex<K, V> {
    fn new() -> Self {
        Self {
            values: HashMap::new(),
            backing: None,
        }
    }

    fn reserve(&mut self, additional: usize, cpu: &CpuAccount) -> Result<()> {
        let resource = RenderResource::RetainedCpuBytes;
        let needed = self
            .len()
            .checked_add(additional)
            .ok_or_else(|| cpu.capacity_error(resource))?;
        if needed <= self.capacity() {
            return Ok(());
        }
        let existing = self.backing.as_ref().map(CpuHold::account);
        let cpu = existing.as_ref().unwrap_or(cpu);
        cpu.phase(|| {
            cpu.preview(resource, cpu.sized::<(K, V)>(needed, resource)?)?;
            let mut replacement = Self {
                values: HashMap::with_hasher(self.values.hasher().clone()),
                backing: None,
            };
            replacement
                .values
                .try_reserve(needed)
                .map_err(|_| cpu.allocation_error(resource))?;
            let actual_bytes = cpu.sized::<(K, V)>(replacement.capacity(), resource)?;
            replacement.backing = Some(cpu.acquire(resource, actual_bytes)?);
            if !self.is_empty() {
                cpu.work(RenderWork::StorageRelocations, self.len())?;
            }
            for (key, value) in self.values.drain() {
                replacement.values.insert(key, value);
            }
            std::mem::swap(self, &mut replacement);
            Ok(())
        })
    }

    pub(super) fn slot(&mut self, cpu: &CpuAccount) -> Result<CacheIndexSlot<'_, K, V>> {
        self.reserve(1, cpu)?;
        Ok(CacheIndexSlot(&mut self.values))
    }

    fn append_reserved(&mut self, mut incoming: Self) {
        for (key, value) in incoming.values.drain() {
            self.values.insert(key, value);
        }
    }

    #[cfg(test)]
    pub(super) fn replace_for_test(&mut self, key: &K, value: V) -> Option<V> {
        self.values
            .get_mut(key)
            .map(|current| std::mem::replace(current, value))
    }
}

impl<K, V> Deref for CacheIndex<K, V> {
    type Target = HashMap<K, V>;

    fn deref(&self) -> &Self::Target {
        &self.values
    }
}

impl<K, V> Drop for CacheIndex<K, V> {
    fn drop(&mut self) {
        let values = std::mem::take(&mut self.values);
        if let Some(backing) = self.backing.take() {
            Hold::Renderer(backing).dispose(|| drop(values));
        } else {
            drop(values);
        }
    }
}

pub(super) struct CacheIndexSlot<'a, K, V>(&'a mut HashMap<K, V>);

impl<K: Eq + Hash, V> CacheIndexSlot<'_, K, V> {
    pub(super) fn write(self, key: K, value: V) -> Option<V> {
        self.0.insert(key, value)
    }
}

use super::key::{
    BindGroupLayoutKey, RenderPipelineKey, SamplerKey, ShaderCompositePathKey, ShaderModuleKey,
};
use super::pipeline::{
    create_blur_bind_group_layout, create_blur_render_pipeline, create_blur_shader_module,
    create_color_filter_bind_group_layout, create_color_filter_render_pipeline,
    create_color_filter_shader_module, create_composite_bind_group_layout,
    create_composite_render_pipeline, create_composite_shader_module,
    create_convolution_bind_group_layout, create_convolution_render_pipeline,
    create_convolution_shader_module, create_copy_backdrop_bind_group_layout,
    create_copy_backdrop_pipeline, create_copy_backdrop_shader_module,
    create_core_pass_bind_group_layout, create_core_pass_render_pipeline,
    create_core_pass_shader_module, create_drop_shadow_colorize_bind_group_layout,
    create_drop_shadow_colorize_render_pipeline, create_drop_shadow_colorize_shader_module,
    create_gaussian_bind_group_layout, create_gaussian_render_pipeline,
    create_gaussian_shader_module, create_native_shadow_bind_group_layout,
    create_native_shadow_render_pipeline, create_native_shadow_shader_module,
    create_offset_bind_group_layout, create_offset_render_pipeline, create_offset_shader_module,
    create_sampler,
};
use super::validate::{
    BlurPassDescription, BlurPassKeyRefs, ColorFilterPassDescription, ColorFilterPassKeyRefs,
    CompositePassDescription, CompositePassKeyRefs, ConvolutionPassDescription,
    ConvolutionPassKeyRefs, CopyBackdropPassDescription, CopyBackdropPassKeyRefs, CorePassKeyRefs,
    CorePassProgram, DropShadowColorizePassDescription, DropShadowColorizePassKeyRefs,
    GaussianPassDescription, GaussianPassKeyRefs, NativeShadowPassDescription,
    NativeShadowPassKeyRefs, OffsetPassDescription, OffsetPassKeyRefs, blur_cache_error,
    color_filter_cache_error, convolution_cache_error, copy_backdrop_cache_error,
    core_pass_cache_error, drop_shadow_cache_error, gaussian_cache_error,
    native_shadow_cache_error, offset_cache_error, validate_blur_pass_keys,
    validate_color_filter_pass_keys, validate_composite_pass_keys, validate_convolution_pass_keys,
    validate_copy_backdrop_pass_keys, validate_core_pass_keys,
    validate_drop_shadow_colorize_pass_keys, validate_gaussian_pass_keys,
    validate_native_shadow_pass_keys, validate_offset_pass_keys,
};

/// Non-clone handles created inside one checked GPU-operation scope. New entries
/// remain private to this phase until the caller explicitly commits after the
/// owning transaction resolves cleanly.
#[must_use = "provisional pass-cache handles must be committed only after checked success"]
pub(crate) struct ProvisionalDevicePassCacheUpdate {
    pub(super) cache_identity: Arc<()>,
    pub(super) samplers: CacheIndex<SamplerKey, OwnedGpu<wgpu::Sampler>>,
    pub(super) layouts: CacheIndex<BindGroupLayoutKey, OwnedGpu<wgpu::BindGroupLayout>>,
    pub(super) shaders: CacheIndex<ShaderModuleKey, OwnedGpu<wgpu::ShaderModule>>,
    pub(super) pipelines: CacheIndex<RenderPipelineKey, OwnedGpu<wgpu::RenderPipeline>>,
}

/// Keeps a rejected update available for its caller's ordered cleanup.
#[must_use]
pub(crate) enum PassCacheCommitPreparation<'cache> {
    Ready(ReadyPassCacheUpdate<'cache>),
    Rejected {
        update: ProvisionalDevicePassCacheUpdate,
        error: Error,
    },
}

/// Owns the checked keys and the exclusive reservation in their destination.
#[must_use = "ready pass-cache updates must be committed or aborted on drop"]
pub(crate) struct ReadyPassCacheUpdate<'cache> {
    update: ProvisionalDevicePassCacheUpdate,
    cache: &'cache mut DevicePassCache,
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

/// Provisional convolution objects; all textures use explicit textureLoad.
pub(crate) struct ProvisionalConvolutionPassObjects<'a> {
    description: ConvolutionPassDescription,
    layout: &'a OwnedGpu<wgpu::BindGroupLayout>,
    shader: &'a OwnedGpu<wgpu::ShaderModule>,
    pipeline: &'a OwnedGpu<wgpu::RenderPipeline>,
}

/// Provisional gaussian objects; all textures use explicit textureLoad.
pub(crate) struct ProvisionalGaussianPassObjects<'a> {
    description: GaussianPassDescription,
    layout: &'a OwnedGpu<wgpu::BindGroupLayout>,
    shader: &'a OwnedGpu<wgpu::ShaderModule>,
    pipeline: &'a OwnedGpu<wgpu::RenderPipeline>,
}

/// Provisional offset objects; all textures use explicit textureLoad.
pub(crate) struct ProvisionalOffsetPassObjects<'a> {
    description: OffsetPassDescription,
    layout: &'a OwnedGpu<wgpu::BindGroupLayout>,
    shader: &'a OwnedGpu<wgpu::ShaderModule>,
    pipeline: &'a OwnedGpu<wgpu::RenderPipeline>,
}

/// Provisional native shadow objects; all textures use explicit textureLoad.
pub(crate) struct ProvisionalNativeShadowPassObjects<'a> {
    description: NativeShadowPassDescription,
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
    pub(super) samplers: CacheIndex<SamplerKey, OwnedGpu<wgpu::Sampler>>,
    pub(super) layouts: CacheIndex<BindGroupLayoutKey, OwnedGpu<wgpu::BindGroupLayout>>,
    pub(super) shaders: CacheIndex<ShaderModuleKey, OwnedGpu<wgpu::ShaderModule>>,
    pub(super) pipelines: CacheIndex<RenderPipelineKey, OwnedGpu<wgpu::RenderPipeline>>,
}

impl DevicePassCache {
    #[must_use]
    pub(crate) fn new(generation: Generation, cpu: CpuAccount) -> Self {
        Self {
            generation,
            identity: Arc::new(()),
            samplers: CacheIndex::new(),
            layouts: CacheIndex::new(),
            shaders: CacheIndex::new(),
            pipelines: CacheIndex::new(),
            cpu,
        }
    }

    pub(crate) fn clear(&mut self) {
        self.samplers.values.clear();
        self.layouts.values.clear();
        self.shaders.values.clear();
        self.pipelines.values.clear();
    }

    pub(crate) fn provisional_update(&self) -> ProvisionalDevicePassCacheUpdate {
        ProvisionalDevicePassCacheUpdate {
            cache_identity: Arc::clone(&self.identity),
            samplers: CacheIndex::new(),
            layouts: CacheIndex::new(),
            shaders: CacheIndex::new(),
            pipelines: CacheIndex::new(),
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
        let description = validate_core_pass_keys(keys, Some(account))?;

        for sampler_key in keys.samplers {
            if !contains(&cache.samplers, sampler_key, account)?
                && !contains(&self.samplers, sampler_key, account)?
            {
                let slot = self.samplers.slot(&cache.cpu)?;
                slot.write(
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
            let slot = self.layouts.slot(&cache.cpu)?;
            slot.write(
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
            let slot = self.shaders.slot(&cache.cpu)?;
            slot.write(
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
            let slot = self.pipelines.slot(&cache.cpu)?;
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
            slot.write(keys.pipeline.clone(), created);
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
        let description = validate_copy_backdrop_pass_keys(keys, Some(account))?;
        for sampler_key in keys.samplers {
            if !contains(&cache.samplers, sampler_key, account)?
                && !contains(&self.samplers, sampler_key, account)?
            {
                let slot = self.samplers.slot(&cache.cpu)?;
                slot.write(
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
            let slot = self.layouts.slot(&cache.cpu)?;
            slot.write(
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
            let slot = self.shaders.slot(&cache.cpu)?;
            slot.write(
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
            let slot = self.pipelines.slot(&cache.cpu)?;
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
            slot.write(keys.pipeline.clone(), created);
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

    pub(crate) fn realize_convolution_pass<'a>(
        &'a mut self,
        device: &wgpu::Device,
        cache: &'a DevicePassCache,
        account: &Account,
        keys: ConvolutionPassKeyRefs<'_>,
    ) -> Result<ProvisionalConvolutionPassObjects<'a>> {
        if !Arc::ptr_eq(&self.cache_identity, &cache.identity) {
            return Err(convolution_cache_error(
                "provisional convolution objects belong to another device cache",
            ));
        }
        let description = validate_convolution_pass_keys(keys, Some(account))?;
        if !contains(&cache.layouts, keys.layout, account)?
            && !contains(&self.layouts, keys.layout, account)?
        {
            let slot = self.layouts.slot(&cache.cpu)?;
            slot.write(
                keys.layout.clone(),
                create_convolution_bind_group_layout(
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
            let slot = self.shaders.slot(&cache.cpu)?;
            slot.write(
                keys.shader.clone(),
                create_convolution_shader_module(
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
            let layout_handle = lookup_pair(&self.layouts, &cache.layouts, keys.layout, account)?
                .ok_or_else(|| {
                convolution_cache_error("convolution bind-group layout realization was lost")
            })?;
            let shader_handle = lookup_pair(&self.shaders, &cache.shaders, keys.shader, account)?
                .ok_or_else(|| {
                convolution_cache_error("convolution shader-module realization was lost")
            })?;
            let created = create_convolution_render_pipeline(
                &cache.generation,
                &cache.cpu,
                Some(account),
                device,
                description,
                layout_handle,
                shader_handle,
            )?;
            slot.write(keys.pipeline.clone(), created);
        }
        self.convolution_pass_objects(cache, account, keys)
    }

    pub(crate) fn convolution_encoding_objects<'a>(
        &'a self,
        cache: &'a DevicePassCache,
        account: &Account,
        samplers: &[SamplerKey],
        layout: &BindGroupLayoutKey,
        shader: &ShaderModuleKey,
        pipeline: &RenderPipelineKey,
    ) -> Result<ProvisionalConvolutionPassObjects<'a>> {
        self.convolution_pass_objects(
            cache,
            account,
            ConvolutionPassKeyRefs {
                samplers,
                layout,
                shader,
                pipeline,
            },
        )
    }

    pub(crate) fn realize_gaussian_pass<'a>(
        &'a mut self,
        device: &wgpu::Device,
        cache: &'a DevicePassCache,
        account: &Account,
        keys: GaussianPassKeyRefs<'_>,
    ) -> Result<ProvisionalGaussianPassObjects<'a>> {
        if !Arc::ptr_eq(&self.cache_identity, &cache.identity) {
            return Err(gaussian_cache_error(
                "provisional gaussian objects belong to another device cache",
            ));
        }
        let description = validate_gaussian_pass_keys(keys, Some(account))?;
        if !contains(&cache.layouts, keys.layout, account)?
            && !contains(&self.layouts, keys.layout, account)?
        {
            let slot = self.layouts.slot(&cache.cpu)?;
            slot.write(
                keys.layout.clone(),
                create_gaussian_bind_group_layout(
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
                create_gaussian_shader_module(
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
            let layout_handle = lookup_pair(&self.layouts, &cache.layouts, keys.layout, account)?
                .ok_or_else(|| {
                gaussian_cache_error("gaussian bind-group layout realization was lost")
            })?;
            let shader_handle = lookup_pair(&self.shaders, &cache.shaders, keys.shader, account)?
                .ok_or_else(|| {
                gaussian_cache_error("gaussian shader-module realization was lost")
            })?;
            let created = create_gaussian_render_pipeline(
                &cache.generation,
                &cache.cpu,
                Some(account),
                device,
                description,
                layout_handle,
                shader_handle,
            )?;
            slot.write(keys.pipeline.clone(), created);
        }
        self.gaussian_pass_objects(cache, account, keys)
    }

    pub(crate) fn gaussian_encoding_objects<'a>(
        &'a self,
        cache: &'a DevicePassCache,
        account: &Account,
        samplers: &[SamplerKey],
        layout: &BindGroupLayoutKey,
        shader: &ShaderModuleKey,
        pipeline: &RenderPipelineKey,
    ) -> Result<ProvisionalGaussianPassObjects<'a>> {
        self.gaussian_pass_objects(
            cache,
            account,
            GaussianPassKeyRefs {
                samplers,
                layout,
                shader,
                pipeline,
            },
        )
    }

    pub(crate) fn realize_offset_pass<'a>(
        &'a mut self,
        device: &wgpu::Device,
        cache: &'a DevicePassCache,
        account: &Account,
        keys: OffsetPassKeyRefs<'_>,
    ) -> Result<ProvisionalOffsetPassObjects<'a>> {
        if !Arc::ptr_eq(&self.cache_identity, &cache.identity) {
            return Err(offset_cache_error(
                "provisional offset objects belong to another device cache",
            ));
        }
        let description = validate_offset_pass_keys(keys, Some(account))?;
        if !contains(&cache.layouts, keys.layout, account)?
            && !contains(&self.layouts, keys.layout, account)?
        {
            let slot = self.layouts.slot(&cache.cpu)?;
            slot.write(
                keys.layout.clone(),
                create_offset_bind_group_layout(
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
            let slot = self.shaders.slot(&cache.cpu)?;
            slot.write(
                keys.shader.clone(),
                create_offset_shader_module(&cache.generation, &cache.cpu, Some(account), device)?,
            );
        }
        if !contains(&cache.pipelines, keys.pipeline, account)?
            && !contains(&self.pipelines, keys.pipeline, account)?
        {
            let slot = self.pipelines.slot(&cache.cpu)?;
            let layout_handle = lookup_pair(&self.layouts, &cache.layouts, keys.layout, account)?
                .ok_or_else(|| {
                offset_cache_error("offset bind-group layout realization was lost")
            })?;
            let shader_handle = lookup_pair(&self.shaders, &cache.shaders, keys.shader, account)?
                .ok_or_else(|| {
                offset_cache_error("offset shader-module realization was lost")
            })?;
            let created = create_offset_render_pipeline(
                &cache.generation,
                &cache.cpu,
                Some(account),
                device,
                description,
                layout_handle,
                shader_handle,
            )?;
            slot.write(keys.pipeline.clone(), created);
        }
        self.offset_pass_objects(cache, account, keys)
    }

    pub(crate) fn offset_encoding_objects<'a>(
        &'a self,
        cache: &'a DevicePassCache,
        account: &Account,
        samplers: &[SamplerKey],
        layout: &BindGroupLayoutKey,
        shader: &ShaderModuleKey,
        pipeline: &RenderPipelineKey,
    ) -> Result<ProvisionalOffsetPassObjects<'a>> {
        self.offset_pass_objects(
            cache,
            account,
            OffsetPassKeyRefs {
                samplers,
                layout,
                shader,
                pipeline,
            },
        )
    }

    pub(crate) fn realize_native_shadow_pass<'a>(
        &'a mut self,
        device: &wgpu::Device,
        cache: &'a DevicePassCache,
        account: &Account,
        keys: NativeShadowPassKeyRefs<'_>,
    ) -> Result<ProvisionalNativeShadowPassObjects<'a>> {
        if !Arc::ptr_eq(&self.cache_identity, &cache.identity) {
            return Err(native_shadow_cache_error(
                "provisional native shadow objects belong to another device cache",
            ));
        }
        let description = validate_native_shadow_pass_keys(keys, Some(account))?;
        if !contains(&cache.layouts, keys.layout, account)?
            && !contains(&self.layouts, keys.layout, account)?
        {
            let slot = self.layouts.slot(&cache.cpu)?;
            slot.write(
                keys.layout.clone(),
                create_native_shadow_bind_group_layout(
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
                create_native_shadow_shader_module(
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
            let layout_handle = lookup_pair(&self.layouts, &cache.layouts, keys.layout, account)?
                .ok_or_else(|| {
                native_shadow_cache_error("native shadow bind-group layout realization was lost")
            })?;
            let shader_handle = lookup_pair(&self.shaders, &cache.shaders, keys.shader, account)?
                .ok_or_else(|| {
                native_shadow_cache_error("native shadow shader-module realization was lost")
            })?;
            let created = create_native_shadow_render_pipeline(
                &cache.generation,
                &cache.cpu,
                Some(account),
                device,
                description,
                layout_handle,
                shader_handle,
            )?;
            slot.write(keys.pipeline.clone(), created);
        }
        self.native_shadow_pass_objects(cache, account, keys)
    }

    pub(crate) fn native_shadow_encoding_objects<'a>(
        &'a self,
        cache: &'a DevicePassCache,
        account: &Account,
        samplers: &[SamplerKey],
        layout: &BindGroupLayoutKey,
        shader: &ShaderModuleKey,
        pipeline: &RenderPipelineKey,
    ) -> Result<ProvisionalNativeShadowPassObjects<'a>> {
        self.native_shadow_pass_objects(
            cache,
            account,
            NativeShadowPassKeyRefs {
                samplers,
                layout,
                shader,
                pipeline,
            },
        )
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
        let description = validate_color_filter_pass_keys(keys, Some(account))?;
        for sampler_key in keys.samplers {
            if !contains(&cache.samplers, sampler_key, account)?
                && !contains(&self.samplers, sampler_key, account)?
            {
                let slot = self.samplers.slot(&cache.cpu)?;
                slot.write(
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
            let slot = self.layouts.slot(&cache.cpu)?;
            slot.write(
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
            let slot = self.shaders.slot(&cache.cpu)?;
            slot.write(
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
            let slot = self.pipelines.slot(&cache.cpu)?;
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
            slot.write(keys.pipeline.clone(), created);
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
        let description = validate_blur_pass_keys(keys, Some(account))?;
        for sampler_key in keys.samplers {
            if !contains(&cache.samplers, sampler_key, account)?
                && !contains(&self.samplers, sampler_key, account)?
            {
                let slot = self.samplers.slot(&cache.cpu)?;
                slot.write(
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
            let slot = self.layouts.slot(&cache.cpu)?;
            slot.write(
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
            let slot = self.shaders.slot(&cache.cpu)?;
            slot.write(
                keys.shader.clone(),
                create_blur_shader_module(&cache.generation, &cache.cpu, Some(account), device)?,
            );
        }
        if !contains(&cache.pipelines, keys.pipeline, account)?
            && !contains(&self.pipelines, keys.pipeline, account)?
        {
            let slot = self.pipelines.slot(&cache.cpu)?;
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
            slot.write(keys.pipeline.clone(), created);
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
        let description = validate_drop_shadow_colorize_pass_keys(keys, Some(account))?;
        for sampler_key in keys.samplers {
            if !contains(&cache.samplers, sampler_key, account)?
                && !contains(&self.samplers, sampler_key, account)?
            {
                let slot = self.samplers.slot(&cache.cpu)?;
                slot.write(
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
            let slot = self.layouts.slot(&cache.cpu)?;
            slot.write(
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
            let slot = self.shaders.slot(&cache.cpu)?;
            slot.write(
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
            let slot = self.pipelines.slot(&cache.cpu)?;
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
            slot.write(keys.pipeline.clone(), created);
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

    fn convolution_pass_objects<'a>(
        &'a self,
        cache: &'a DevicePassCache,
        account: &Account,
        keys: ConvolutionPassKeyRefs<'_>,
    ) -> Result<ProvisionalConvolutionPassObjects<'a>> {
        let layout =
            lookup_pair(&self.layouts, &cache.layouts, keys.layout, account)?.ok_or_else(|| {
                convolution_cache_error("convolution bind-group layout realization was lost")
            })?;
        let shader =
            lookup_pair(&self.shaders, &cache.shaders, keys.shader, account)?.ok_or_else(|| {
                convolution_cache_error("convolution shader-module realization was lost")
            })?;
        let pipeline = lookup_pair(&self.pipelines, &cache.pipelines, keys.pipeline, account)?
            .ok_or_else(|| {
                convolution_cache_error("convolution render-pipeline realization was lost")
            })?;
        Ok(ProvisionalConvolutionPassObjects {
            description: validate_convolution_pass_keys(keys, Some(account))?,
            layout,
            shader,
            pipeline,
        })
    }

    fn gaussian_pass_objects<'a>(
        &'a self,
        cache: &'a DevicePassCache,
        account: &Account,
        keys: GaussianPassKeyRefs<'_>,
    ) -> Result<ProvisionalGaussianPassObjects<'a>> {
        let layout =
            lookup_pair(&self.layouts, &cache.layouts, keys.layout, account)?.ok_or_else(|| {
                gaussian_cache_error("gaussian bind-group layout realization was lost")
            })?;
        let shader = lookup_pair(&self.shaders, &cache.shaders, keys.shader, account)?
            .ok_or_else(|| gaussian_cache_error("gaussian shader-module realization was lost"))?;
        let pipeline = lookup_pair(&self.pipelines, &cache.pipelines, keys.pipeline, account)?
            .ok_or_else(|| gaussian_cache_error("gaussian render-pipeline realization was lost"))?;
        Ok(ProvisionalGaussianPassObjects {
            description: validate_gaussian_pass_keys(keys, Some(account))?,
            layout,
            shader,
            pipeline,
        })
    }

    fn offset_pass_objects<'a>(
        &'a self,
        cache: &'a DevicePassCache,
        account: &Account,
        keys: OffsetPassKeyRefs<'_>,
    ) -> Result<ProvisionalOffsetPassObjects<'a>> {
        let layout = lookup_pair(&self.layouts, &cache.layouts, keys.layout, account)?
            .ok_or_else(|| offset_cache_error("offset bind-group layout realization was lost"))?;
        let shader = lookup_pair(&self.shaders, &cache.shaders, keys.shader, account)?
            .ok_or_else(|| offset_cache_error("offset shader-module realization was lost"))?;
        let pipeline = lookup_pair(&self.pipelines, &cache.pipelines, keys.pipeline, account)?
            .ok_or_else(|| offset_cache_error("offset render-pipeline realization was lost"))?;
        Ok(ProvisionalOffsetPassObjects {
            description: validate_offset_pass_keys(keys, Some(account))?,
            layout,
            shader,
            pipeline,
        })
    }

    fn native_shadow_pass_objects<'a>(
        &'a self,
        cache: &'a DevicePassCache,
        account: &Account,
        keys: NativeShadowPassKeyRefs<'_>,
    ) -> Result<ProvisionalNativeShadowPassObjects<'a>> {
        let layout =
            lookup_pair(&self.layouts, &cache.layouts, keys.layout, account)?.ok_or_else(|| {
                native_shadow_cache_error("native shadow bind-group layout realization was lost")
            })?;
        let shader =
            lookup_pair(&self.shaders, &cache.shaders, keys.shader, account)?.ok_or_else(|| {
                native_shadow_cache_error("native shadow shader-module realization was lost")
            })?;
        let pipeline = lookup_pair(&self.pipelines, &cache.pipelines, keys.pipeline, account)?
            .ok_or_else(|| {
                native_shadow_cache_error("native shadow render-pipeline realization was lost")
            })?;
        Ok(ProvisionalNativeShadowPassObjects {
            description: validate_native_shadow_pass_keys(keys, Some(account))?,
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
        let description = validate_composite_pass_keys(keys, Some(account))?;
        for sampler_key in keys.samplers {
            if !contains(&cache.samplers, sampler_key, account)?
                && !contains(&self.samplers, sampler_key, account)?
            {
                let slot = self.samplers.slot(&cache.cpu)?;
                slot.write(
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
            let slot = self.layouts.slot(&cache.cpu)?;
            slot.write(
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
            let slot = self.shaders.slot(&cache.cpu)?;
            slot.write(
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
            let slot = self.pipelines.slot(&cache.cpu)?;
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
            slot.write(keys.pipeline.clone(), created);
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

    pub(crate) fn prepare_commit<'cache>(
        self,
        cache: &'cache mut DevicePassCache,
        account: &Account,
    ) -> PassCacheCommitPreparation<'cache> {
        match self.reserve_commit_capacity(cache, account) {
            Ok(()) => PassCacheCommitPreparation::Ready(ReadyPassCacheUpdate {
                update: self,
                cache,
            }),
            Err(error) => PassCacheCommitPreparation::Rejected {
                update: self,
                error,
            },
        }
    }

    pub(crate) fn commit_into(self, cache: &mut DevicePassCache, account: &Account) -> Result<()> {
        match self.prepare_commit(cache, account) {
            PassCacheCommitPreparation::Ready(ready) => {
                ready.commit();
                Ok(())
            }
            PassCacheCommitPreparation::Rejected { error, .. } => Err(error),
        }
    }

    fn reserve_commit_capacity(
        &self,
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
                cache.$field.reserve(self.$field.len(), &cache.cpu)?;
            }};
        }
        prepare!(samplers);
        prepare!(layouts);
        prepare!(shaders);
        prepare!(pipelines);
        Ok(())
    }
}

impl ReadyPassCacheUpdate<'_> {
    pub(crate) fn recheck(&mut self, account: &Account) -> Result<()> {
        self.update.reserve_commit_capacity(self.cache, account)
    }

    pub(crate) fn commit(self) {
        let Self { update, cache } = self;
        // The exclusive destination borrow preserves the checked keys and capacity.
        // Individual insertion avoids HashMap::extend making a speculative reservation.
        cache.samplers.append_reserved(update.samplers);
        cache.layouts.append_reserved(update.layouts);
        cache.shaders.append_reserved(update.shaders);
        cache.pipelines.append_reserved(update.pipelines);
    }
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

impl ProvisionalConvolutionPassObjects<'_> {
    pub(crate) fn require_encoding_ready(&self) -> Result<()> {
        let _ = (self.description, self.layout, self.shader, self.pipeline);
        Ok(())
    }

    pub(crate) const fn bind_group_layout(&self) -> &OwnedGpu<wgpu::BindGroupLayout> {
        self.layout
    }

    pub(crate) const fn render_pipeline(&self) -> &OwnedGpu<wgpu::RenderPipeline> {
        self.pipeline
    }
}

impl ProvisionalGaussianPassObjects<'_> {
    pub(crate) fn require_encoding_ready(&self) -> Result<()> {
        let _ = (self.description, self.layout, self.shader, self.pipeline);
        Ok(())
    }

    pub(crate) const fn bind_group_layout(&self) -> &OwnedGpu<wgpu::BindGroupLayout> {
        self.layout
    }

    pub(crate) const fn render_pipeline(&self) -> &OwnedGpu<wgpu::RenderPipeline> {
        self.pipeline
    }
}

impl ProvisionalOffsetPassObjects<'_> {
    pub(crate) fn require_encoding_ready(&self) -> Result<()> {
        let _ = (self.description, self.layout, self.shader, self.pipeline);
        Ok(())
    }

    pub(crate) const fn bind_group_layout(&self) -> &OwnedGpu<wgpu::BindGroupLayout> {
        self.layout
    }

    pub(crate) const fn render_pipeline(&self) -> &OwnedGpu<wgpu::RenderPipeline> {
        self.pipeline
    }
}

impl ProvisionalNativeShadowPassObjects<'_> {
    pub(crate) fn require_encoding_ready(&self) -> Result<()> {
        let _ = (self.description, self.layout, self.shader, self.pipeline);
        Ok(())
    }

    pub(crate) const fn bind_group_layout(&self) -> &OwnedGpu<wgpu::BindGroupLayout> {
        self.layout
    }

    pub(crate) const fn render_pipeline(&self) -> &OwnedGpu<wgpu::RenderPipeline> {
        self.pipeline
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

    struct CheckedCorePassFixture {
        backend: crate::backend::Backend,
        identity: crate::backend::DeviceSlotIdentity,
        generation: Generation,
        cpu: CpuAccount,
        renderer_cpu: crate::accounting::RendererCpu,
        keys: crate::pass::RuntimePassCacheKeys,
    }

    impl CheckedCorePassFixture {
        fn new() -> Self {
            use crate::backend::{Backend, DeviceCapabilities};
            use crate::{
                Antialiasing, Capabilities, Color, EffectQualityPolicy, Format, Rect, Scene, Size,
            };

            let options = RenderAccountingOptions::new();
            let cpu = crate::accounting::RendererCpu::new(options).unwrap();
            let mut backend =
                Backend::new(crate::ResourceCacheBudget::DISABLED, cpu.clone(), options).unwrap();
            let identity = pollster::block_on(backend.select_device(None))
                .unwrap()
                .unwrap();
            let ready = backend
                .ready_device_state_borrow_for_test(identity)
                .unwrap();
            let generation = ready.generation_for_test().clone();
            let capabilities =
                DeviceCapabilities::from_device(ready.adapter_for_test(), ready.device_for_test());
            let format = capabilities
                .resolve_effect_working_format(EffectQualityPolicy::AllowReducedPrecision)
                .unwrap();
            let setup = Account::for_test();
            let mut scene = Scene::new();
            scene.fill(Rect::new(0.0, 0.0, 2.0, 2.0), Color::BLACK);
            let commands = scene.normalize(Capabilities::CURRENT, &setup).unwrap();
            let context = crate::frame::FrameContext::try_new(
                Size::new(2.0, 2.0),
                1.0,
                Antialiasing::Area,
                Color::TRANSPARENT,
            )
            .unwrap();
            let requests = crate::pass::core_pass_cache_requests_for_test(
                commands,
                context,
                capabilities,
                format,
                Format::Rgba8,
            )
            .unwrap();
            let keys = requests
                .passes()
                .iter()
                .find(|keys| {
                    crate::shader::core_pass_key_facts_for_test(
                        keys.samplers(),
                        keys.layout(),
                        keys.shader(),
                        keys.pipeline(),
                    )
                    .is_some_and(|facts| {
                        facts.program == crate::shader::CorePassProgramForTest::CanonicalizeCapture
                    })
                })
                .unwrap()
                .clone();
            assert_eq!(keys.samplers().len(), 1);
            Self {
                backend,
                identity,
                generation,
                cpu: cpu.at(crate::RenderCpuScope::DeviceBacking),
                renderer_cpu: cpu,
                keys,
            }
        }

        fn empty_cache(&self) -> DevicePassCache {
            DevicePassCache::new(self.generation.clone(), self.cpu.clone())
        }

        fn checked<T>(&mut self, realize: impl FnOnce(&wgpu::Device, &Account) -> Result<T>) -> T {
            let operation = crate::RuntimeOperation::EffectRendering;
            let transaction = self
                .backend
                .begin_gpu_operation(
                    self.identity,
                    crate::gpu_transaction::GpuOperationStage::Render,
                    operation,
                )
                .unwrap();
            let ready = self
                .backend
                .ready_device_state_borrow_for_test(self.identity)
                .unwrap();
            let account = Account::for_test();
            let realized = realize(ready.device_for_test(), &account).unwrap();
            pollster::block_on(transaction.finish(operation, Some(&account))).unwrap();
            realized
        }

        fn update(&mut self, cache: &DevicePassCache) -> ProvisionalDevicePassCacheUpdate {
            let mut update = cache.provisional_update();
            let keys = self.keys.clone();
            self.checked(|device, account| {
                update
                    .realize_core_pass(
                        device,
                        cache,
                        account,
                        CorePassKeyRefs {
                            samplers: keys.samplers(),
                            layout: keys.layout(),
                            shader: keys.shader(),
                            pipeline: keys.pipeline(),
                        },
                    )?
                    .require_encoding_ready()
            });
            assert_eq!(
                [
                    update.samplers.len(),
                    update.layouts.len(),
                    update.shaders.len(),
                    update.pipelines.len()
                ],
                [1; 4]
            );
            update
        }

        fn live_objects(&self) -> u64 {
            let snapshot = self.generation.snapshot().unwrap();
            assert_eq!(snapshot.fault(), None);
            snapshot.usage().live(RenderResource::OpaqueGpuObjects)
        }
    }

    fn cache_entries(cache: &DevicePassCache) -> [usize; 4] {
        [
            cache.samplers.len(),
            cache.layouts.len(),
            cache.shaders.len(),
            cache.pipelines.len(),
        ]
    }

    fn cache_capacity(cache: &DevicePassCache) -> [usize; 4] {
        [
            cache.samplers.capacity(),
            cache.layouts.capacity(),
            cache.shaders.capacity(),
            cache.pipelines.capacity(),
        ]
    }

    fn capacity_bytes([samplers, layouts, shaders, pipelines]: [usize; 4]) -> u64 {
        use std::mem::size_of;

        (samplers * size_of::<(SamplerKey, OwnedGpu<wgpu::Sampler>)>()
            + layouts * size_of::<(BindGroupLayoutKey, OwnedGpu<wgpu::BindGroupLayout>)>()
            + shaders * size_of::<(ShaderModuleKey, OwnedGpu<wgpu::ShaderModule>)>()
            + pipelines * size_of::<(RenderPipelineKey, OwnedGpu<wgpu::RenderPipeline>)>())
            as u64
    }

    #[test]
    fn pass_cache_accounts_provisional_and_committed_usable_capacity() {
        let mut fixture = CheckedCorePassFixture::new();
        let owner = fixture.renderer_cpu.clone();
        let live = || {
            owner
                .snapshot()
                .unwrap()
                .usage()
                .live(RenderResource::RetainedCpuBytes)
        };
        let opening = live();
        let mut cache = fixture.empty_cache();
        let update = fixture.update(&cache);
        let provisional = capacity_bytes([
            update.samplers.capacity(),
            update.layouts.capacity(),
            update.shaders.capacity(),
            update.pipelines.capacity(),
        ]);
        assert!(provisional > 0);
        assert_eq!(live() - opening, provisional);

        let account = Account::for_test();
        let PassCacheCommitPreparation::Ready(ready) = update.prepare_commit(&mut cache, &account)
        else {
            panic!("prepared cache must reserve its destination");
        };
        let committed = capacity_bytes(cache_capacity(ready.cache));
        assert_eq!(live() - opening, provisional + committed);
        ready.commit();
        assert_eq!(cache_entries(&cache), [1; 4]);
        assert_eq!(live() - opening, committed);
        drop(cache);
        assert_eq!(live(), opening);
    }

    #[test]
    fn pass_cache_destination_capacity_refusal_keeps_update_for_retry() {
        use crate::accounting::RendererCpu;
        use crate::{RenderAccountingError, RenderCpuScope, RenderQuantity, RenderRefusalReason};

        let mut fixture = CheckedCorePassFixture::new();
        let native_opening = fixture.live_objects();
        let capacities = {
            let cache = fixture.empty_cache();
            let update = fixture.update(&cache);
            [
                update.samplers.capacity(),
                update.layouts.capacity(),
                update.shaders.capacity(),
                update.pipelines.capacity(),
            ]
        };
        let total = capacity_bytes(capacities);
        let options = RenderAccountingOptions::new();
        let opening = RendererCpu::new(options)
            .unwrap()
            .snapshot()
            .unwrap()
            .usage()
            .live(RenderResource::RetainedCpuBytes);
        let owner = RendererCpu::new(
            options.with_limits(RenderLimits::new().with_retained_bytes(opening + 2 * total)),
        )
        .unwrap();
        let live = || {
            owner
                .snapshot()
                .unwrap()
                .usage()
                .live(RenderResource::RetainedCpuBytes)
        };
        let cpu = owner.at(RenderCpuScope::DeviceBacking);
        let mut cache = DevicePassCache::new(fixture.generation.clone(), cpu.clone());
        let update = fixture.update(&cache);
        assert_eq!(live(), opening + total);
        let pressure = cpu.acquire(RenderResource::RetainedCpuBytes, 1).unwrap();
        let (update, error, report) = rejected_readiness(update, &mut cache, 4);
        let [samplers, layouts, shaders, _] = capacities;
        let prefix = [samplers, layouts, shaders, 0];
        let live_prefix = opening + total + capacity_bytes(prefix) + 1;
        assert_eq!(
            error.accounting_diagnostic(),
            Some(RenderAccountingError::CpuRefused {
                scope: RenderCpuScope::Root,
                quantity: RenderQuantity::Resource(RenderResource::RetainedCpuBytes),
                current: live_prefix,
                amount: total - capacity_bytes(prefix),
                reason: RenderRefusalReason::Limit {
                    limit: opening + 2 * total,
                },
            }),
        );
        assert_readiness_work(&report, 4);
        assert_eq!(cache_capacity(&cache), prefix);
        assert_eq!(cache_entries(&cache), [0; 4]);
        assert_eq!(live(), live_prefix);
        assert_eq!(fixture.live_objects(), native_opening + 4);

        drop(pressure);
        let account = Account::for_test();
        let PassCacheCommitPreparation::Ready(ready) = update.prepare_commit(&mut cache, &account)
        else {
            panic!("retry must retain the provisional native handles");
        };
        assert_eq!(live(), opening + 2 * total);
        ready.commit();
        assert_eq!(cache_entries(&cache), [1; 4]);
        assert_eq!(live(), opening + total);
        assert_eq!(fixture.live_objects(), native_opening + 4);
        drop(cache);
        assert_eq!(live(), opening);
        assert_eq!(fixture.live_objects(), native_opening);
        cpu.ensure_healthy().unwrap();
    }

    #[test]
    fn pass_cache_index_growth_preserves_old_entries_on_refusal() {
        use crate::accounting::RendererCpu;
        use crate::{RenderAccountingError, RenderCpuScope, RenderQuantity};

        let actual_capacity = |count| {
            let mut map = HashMap::<usize, usize>::new();
            map.try_reserve(count).unwrap();
            map.capacity()
        };
        let first = actual_capacity(1);
        let second = actual_capacity(first + 1);
        let bytes = |capacity| (capacity * std::mem::size_of::<(usize, usize)>()) as u64;
        let options = RenderAccountingOptions::new();
        let opening = RendererCpu::new(options)
            .unwrap()
            .snapshot()
            .unwrap()
            .usage()
            .live(RenderResource::RetainedCpuBytes);
        let owner = RendererCpu::new(options.with_limits(
            RenderLimits::new().with_retained_bytes(opening + bytes(first) + bytes(second)),
        ))
        .unwrap();
        let usage = || owner.snapshot().unwrap().usage();
        let cpu = owner.at(RenderCpuScope::DeviceBacking);
        let mut index = CacheIndex::new();
        for key in 0..first {
            index.slot(&cpu).unwrap().write(key, key + 10);
        }
        assert_eq!(
            usage().live(RenderResource::RetainedCpuBytes),
            opening + bytes(first)
        );
        let pressure = cpu.acquire(RenderResource::RetainedCpuBytes, 1).unwrap();
        let error = index.slot(&cpu).err().unwrap();
        assert!(matches!(
            error.accounting_diagnostic(),
            Some(RenderAccountingError::CpuRefused {
                quantity: RenderQuantity::Resource(RenderResource::RetainedCpuBytes),
                ..
            })
        ));
        assert_eq!(index.capacity(), first);
        assert_eq!(usage().work(RenderWork::StorageRelocations), 0);
        for key in 0..first {
            assert_eq!(index.get(&key), Some(&(key + 10)));
        }

        drop(pressure);
        index.slot(&cpu).unwrap().write(first, first + 10);
        assert_eq!(index.capacity(), second);
        assert_eq!(
            usage().peak(RenderResource::RetainedCpuBytes),
            opening + bytes(first) + bytes(second)
        );
        assert_eq!(
            usage().live(RenderResource::RetainedCpuBytes),
            opening + bytes(second)
        );
        assert_eq!(usage().work(RenderWork::StorageRelocations), first as u64);
        for key in 0..=first {
            assert_eq!(index.get(&key), Some(&(key + 10)));
        }
        drop(index);
        assert_eq!(usage().live(RenderResource::RetainedCpuBytes), opening);
        cpu.ensure_healthy().unwrap();
    }

    #[test]
    fn pass_cache_first_record_refusal_precedes_native_creation() {
        use crate::accounting::RendererCpu;
        use crate::{RenderAccountingError, RenderCpuScope, RenderQuantity};

        let mut fixture = CheckedCorePassFixture::new();
        let native_opening = fixture.live_objects();
        let options = RenderAccountingOptions::new();
        let opening = RendererCpu::new(options)
            .unwrap()
            .snapshot()
            .unwrap()
            .usage()
            .live(RenderResource::RetainedCpuBytes);
        let owner =
            RendererCpu::new(options.with_limits(RenderLimits::new().with_retained_bytes(opening)))
                .unwrap();
        let cache = DevicePassCache::new(
            fixture.generation.clone(),
            owner.at(RenderCpuScope::DeviceBacking),
        );
        let mut update = cache.provisional_update();
        let keys = fixture.keys.clone();
        fixture.checked(|device, account| {
            let result = update.realize_core_pass(
                device,
                &cache,
                account,
                CorePassKeyRefs {
                    samplers: keys.samplers(),
                    layout: keys.layout(),
                    shader: keys.shader(),
                    pipeline: keys.pipeline(),
                },
            );
            let error = result.err().unwrap();
            assert!(matches!(
                error.accounting_diagnostic(),
                Some(RenderAccountingError::CpuRefused {
                    quantity: RenderQuantity::Resource(RenderResource::RetainedCpuBytes),
                    ..
                })
            ));
            assert_eq!(account.work_count(RenderWork::GpuObjectsCreated)?, 0);
            Ok(())
        });
        assert_eq!(fixture.live_objects(), native_opening);
        assert_eq!(update.samplers.capacity(), 0);
        assert_eq!(
            owner
                .snapshot()
                .unwrap()
                .usage()
                .live(RenderResource::RetainedCpuBytes),
            opening,
        );
    }

    fn measure_readiness(
        limit: u64,
        run: impl FnOnce(&Account) -> Result<()>,
    ) -> (Result<()>, crate::RenderStandaloneReport) {
        let limits = RenderLimits::new()
            .with_work_limit(RenderWork::ResourceProbes, limit)
            .unwrap();
        let options = RenderAccountingOptions::new()
            .with_limits(limits)
            .with_profile(crate::RenderProfile::Summary { timing: false });
        let mut context = RenderAccountingContext::new(options);
        let result = context.run(
            RenderAttemptKind::Frame,
            crate::RenderPhase::GpuPrepare,
            run,
        );
        let report = context.take_last_attempt().unwrap();
        assert_eq!(report.cleanup_fault(), None);
        (result, report)
    }

    fn assert_readiness_work(report: &crate::RenderStandaloneReport, probes: u64) {
        let attempt = report.attempt().unwrap();
        assert_eq!(attempt.usage().work(RenderWork::ResourceProbes), probes);
        assert_eq!(
            attempt
                .phase_usage(crate::RenderPhase::GpuPrepare)
                .work(RenderWork::ResourceProbes),
            probes
        );
        assert_eq!(
            attempt.usage().work(RenderWork::ResourceAllocationRequests),
            0
        );
        assert_eq!(attempt.usage().work(RenderWork::GpuObjectsCreated), 0);
    }

    #[test]
    fn checked_pass_cache_readiness_preserves_probe_rounds_and_reserved_capacity() {
        use crate::{
            ErrorCode, RenderAccountingError, RenderPhase, RenderQuantity, RenderRefusalReason,
        };

        let mut fixture = CheckedCorePassFixture::new();
        let baseline = fixture.live_objects();
        for (rounds, limit, commit) in [
            (1, 3, false),
            (1, 4, true),
            (2, 4, false),
            (2, 7, false),
            (2, 8, false),
            (2, 8, true),
        ] {
            let mut cache = fixture.empty_cache();
            let update = fixture.update(&cache);
            assert_eq!(cache_entries(&cache), [0; 4]);
            assert_eq!(cache_capacity(&cache), [0; 4]);
            assert_eq!(fixture.live_objects(), baseline + 4);
            let mut first_capacity = None;
            let mut ready = None;
            let mut rejected = None;
            let (result, report) = measure_readiness(limit, {
                let destination = &mut cache;
                let first_capacity = &mut first_capacity;
                let ready = &mut ready;
                let rejected = &mut rejected;
                move |account| {
                    let mut bound = match update.prepare_commit(destination, account) {
                        PassCacheCommitPreparation::Ready(bound) => bound,
                        PassCacheCommitPreparation::Rejected { update, error } => {
                            *rejected = Some(update);
                            return Err(error);
                        }
                    };
                    *first_capacity = Some(cache_capacity(bound.cache));
                    let result = if rounds == 2 {
                        bound.recheck(account)
                    } else {
                        Ok(())
                    };
                    *ready = Some(bound);
                    result
                }
            });
            let ready_capacity = ready.as_ref().map(|ready| {
                assert_eq!(cache_entries(ready.cache), [0; 4]);
                cache_capacity(ready.cache)
            });
            assert_eq!(fixture.live_objects(), baseline + 4);
            assert_readiness_work(&report, limit);
            if limit < 4 * rounds {
                let error = result.unwrap_err();
                let refusal = report.attempt().unwrap().refusal().unwrap();
                assert_eq!(error.code(), ErrorCode::Accounting);
                assert_eq!(
                    error.accounting_diagnostic(),
                    Some(RenderAccountingError::Refused(refusal))
                );
                assert_eq!(report.status(), RenderAttemptStatus::Refused);
                assert_eq!(refusal.phase(), RenderPhase::Attempt);
                assert_eq!(
                    refusal.quantity(),
                    RenderQuantity::Work(RenderWork::ResourceProbes)
                );
                assert_eq!((refusal.current(), refusal.amount()), (limit, 1));
                assert_eq!(refusal.reason(), RenderRefusalReason::Limit { limit });
            } else {
                result.unwrap();
                assert_eq!(report.status(), RenderAttemptStatus::Succeeded);
                assert_eq!(report.attempt().unwrap().refusal(), None);
            }
            assert_eq!(first_capacity, ready_capacity);
            assert_eq!(rejected.is_some(), limit == 3);

            let expected = if commit {
                let ready = ready.expect("successful readiness owns the destination");
                let keys = &fixture.keys;
                let sampler = ready.update.samplers[&keys.samplers()[0]].clone();
                let layout = ready.update.layouts[keys.layout()].clone();
                let shader = ready.update.shaders[keys.shader()].clone();
                let pipeline = ready.update.pipelines[keys.pipeline()].clone();
                ready.commit();
                Some((sampler, layout, shader, pipeline))
            } else {
                drop(ready);
                None
            };
            drop(rejected);
            if let Some((sampler, layout, shader, pipeline)) = expected {
                let keys = &fixture.keys;
                assert_eq!(*cache.samplers[&keys.samplers()[0]], *sampler);
                assert_eq!(*cache.layouts[keys.layout()], *layout);
                assert_eq!(*cache.shaders[keys.shader()], *shader);
                assert_eq!(*cache.pipelines[keys.pipeline()], *pipeline);
            }
            assert_eq!(cache_entries(&cache), [usize::from(commit); 4]);
            let reserved = cache_capacity(&cache);
            if limit == 3 {
                assert_eq!(first_capacity, None);
                assert_eq!(
                    reserved.map(|capacity| capacity > 0),
                    [true, true, true, false]
                );
            } else {
                assert_eq!(ready_capacity, Some(reserved));
                assert!(reserved.into_iter().all(|capacity| capacity >= 1));
            }
            assert_eq!(
                fixture.live_objects(),
                baseline + if commit { 4 } else { 0 }
            );
            drop(cache);
            assert_eq!(fixture.live_objects(), baseline);
        }
    }

    fn rejected_readiness(
        update: ProvisionalDevicePassCacheUpdate,
        cache: &mut DevicePassCache,
        limit: u64,
    ) -> (
        ProvisionalDevicePassCacheUpdate,
        Error,
        crate::RenderStandaloneReport,
    ) {
        let mut rejected = None;
        let (result, report) = measure_readiness(limit, |account| {
            let PassCacheCommitPreparation::Rejected { update, error } =
                update.prepare_commit(cache, account)
            else {
                panic!("expected readiness rejection");
            };
            rejected = Some(update);
            Err(error)
        });
        (rejected.unwrap(), result.unwrap_err(), report)
    }

    #[test]
    fn checked_pass_cache_readiness_refuses_foreign_destination_and_first_collision() {
        let mut fixture = CheckedCorePassFixture::new();
        let baseline = fixture.live_objects();
        let mut cache = fixture.empty_cache();
        let mut foreign = fixture.empty_cache();
        let update = fixture.update(&cache);
        let (update, error, report) = rejected_readiness(update, &mut foreign, 0);
        assert_eq!(error.code(), crate::ErrorCode::RenderFailed);
        assert_eq!(
            error.message(),
            "provisional pass objects cannot enter another device cache"
        );
        assert_eq!(error.accounting_diagnostic(), None);
        assert_eq!(
            report.status(),
            RenderAttemptStatus::Failed(crate::ErrorCode::RenderFailed)
        );
        assert_readiness_work(&report, 0);
        assert_eq!(cache_entries(&foreign), [0; 4]);
        assert_eq!(cache_capacity(&foreign), [0; 4]);
        assert_eq!(fixture.live_objects(), baseline + 4);

        let key = fixture.keys.samplers()[0];
        let sampler = fixture.checked(|device, account| {
            create_sampler(&cache.generation, &cache.cpu, Some(account), device, key)
        });
        let expected = sampler.clone();
        assert!(
            cache
                .samplers
                .slot(&cache.cpu)
                .unwrap()
                .write(key, sampler)
                .is_none()
        );
        let reserved = cache_capacity(&cache);
        let (update, error, report) = rejected_readiness(update, &mut cache, 1);
        assert_eq!(error.code(), crate::ErrorCode::RenderFailed);
        assert_eq!(
            error.message(),
            "persistent pass cache changed during provisional realization"
        );
        assert_eq!(error.accounting_diagnostic(), None);
        assert_eq!(
            report.status(),
            RenderAttemptStatus::Failed(crate::ErrorCode::RenderFailed)
        );
        assert_readiness_work(&report, 1);
        assert_eq!(cache_entries(&cache), [1, 0, 0, 0]);
        assert_eq!(cache_capacity(&cache), reserved);
        assert_eq!(*cache.samplers[&key], *expected);
        assert_eq!(fixture.live_objects(), baseline + 5);
        drop(update);
        drop(expected);
        assert_eq!(fixture.live_objects(), baseline + 1);
        drop(cache);
        assert_eq!(fixture.live_objects(), baseline);
    }

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
