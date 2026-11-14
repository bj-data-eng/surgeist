use super::{PassMap, PassSet};
#[cfg(test)]
#[path = "prepare_boxed_tests.rs"]
mod boxed_tests;
use crate::accounting::{
    Account, AccountedVec, Generation, GenerationClass, OwnedGpu, RenderPhase, RenderWork,
    SharedValue,
};

use std::{marker::PhantomData, sync::Arc};

use super::super::{
    Format, PhysicalSize, Result,
    backend::DeviceCapabilities,
    filter::CSS_FILTER_KERNEL_SUPPORT_STANDARD_DEVIATIONS,
    frame::GpuRenderGraph,
    image::ResolvedMaskUploadDescriptor,
    renderer::EffectQualityPolicy,
    resource::{
        ConvolutionWeightsPlan, FrameResourceScope, GaussianKernelBufferLimits, GaussianKernelKey,
        GaussianKernelPlan, GaussianKernelSamplingForm, ResourceAllocationPreflight,
        ResourceIdentity, ResourceLease, ResourceManager, WorkingFormat,
    },
    shader::{
        BlurEdgeParameterBytes, BlurPassKeyRefs, ColorFilterOperationBufferLimits,
        ColorFilterOperationBytes, ColorFilterPassKeyRefs, CompositeParameterBytes,
        CompositePassKeyRefs, ConvolutionParameterBytes, ConvolutionPassKeyRefs,
        CopyBackdropPassKeyRefs, CorePassKeyRefs, DevicePassCache, DropShadowColorizePassKeyRefs,
        DropShadowParameterBytes, PassSpatialUniformBytes, ProvisionalCopyBackdropPassObjects,
        ProvisionalDevicePassCacheUpdate,
    },
    texture::EffectTextureDescriptor,
    vello_engine::VelloEngineState,
};
use super::{
    BackdropPreparableGraph, BaseExecutionFacts, BasePreparableGraph, ColorFilterPreparableGraph,
    CompositionPreparableGraph, CustomSpineEncodingState, ExecutableGraphWorkingFormatRequest,
    ForegroundFilterPreparableGraph, LoweredGraphPlan, RuntimeBlur, RuntimeComposite,
    RuntimeCompositeKind, RuntimeGraphGeneration, RuntimePass, RuntimePassCacheKeys, RuntimePassId,
    RuntimePassKind, RuntimeReadBinding, RuntimeResourceFormat, RuntimeResourceId,
    RuntimeResourceImport, RuntimeResourceProducer, RuntimeResourceRequest, RuntimeResourceRole,
    RuntimeResultBinding, SpatialFilterPreparableGraph,
    close::{
        ClosedExecutableGraph, ClosedExecutableGraphFacts, PrePreparationGraphClassification,
        preparation_error,
    },
    lower::{runtime_pass_cache_keys, runtime_resource_format},
    parameters::{
        prepare_blur_edge_parameters, prepare_color_filter_operations,
        prepare_drop_shadow_parameters, prepared_pass_composite_parameters,
        prepared_pass_spatial_uniform,
    },
};

pub(crate) const VELLO_CAPTURE_TEXTURE_USAGES: wgpu::TextureUsages =
    wgpu::TextureUsages::STORAGE_BINDING
        .union(wgpu::TextureUsages::TEXTURE_BINDING)
        .union(wgpu::TextureUsages::COPY_SRC)
        .union(wgpu::TextureUsages::COPY_DST);
const RESOLVED_MASK_TEXTURE_USAGES: wgpu::TextureUsages =
    wgpu::TextureUsages::TEXTURE_BINDING.union(wgpu::TextureUsages::COPY_DST);

#[derive(Clone, Debug, Eq, PartialEq)]
pub(super) enum RuntimeAllocationRequest {
    EffectTexture(EffectTextureDescriptor),
    ResolvedMask(SharedValue<ResolvedMaskUploadDescriptor>),
}

impl RuntimeAllocationRequest {
    fn preflight(&self, account: &Account) -> Result<ResourceAllocationPreflight> {
        match self {
            Self::EffectTexture(descriptor) => {
                ResourceAllocationPreflight::effect_texture(*descriptor)
            }
            Self::ResolvedMask(descriptor) => {
                ResourceAllocationPreflight::resolved_mask(descriptor, account)?.ok_or_else(|| {
                    preparation_error(
                        "an explicitly empty resolved mask survived graph contribution pruning",
                    )
                })
            }
        }
    }

    fn acquire(
        &self,
        frame_scope: &mut FrameResourceScope,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        capabilities: &DeviceCapabilities,
    ) -> Result<ResourceLease> {
        match self {
            Self::EffectTexture(descriptor) => {
                frame_scope.acquire_effect_texture(device, capabilities, *descriptor)
            }
            Self::ResolvedMask(descriptor) => {
                frame_scope.acquire_resolved_mask_upload(device, queue, capabilities, descriptor)
            }
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub(super) struct RuntimeResourcePreparationRequest {
    pub(super) runtime: RuntimeResourceRequest,
    pub(super) allocation: RuntimeAllocationRequest,
}

#[derive(Clone, Debug, PartialEq)]
pub(super) struct RuntimeKernelPreparationRequest {
    pub(super) key: GaussianKernelKey,
    pub(super) plan: GaussianKernelPlan,
    pub(super) last_use: RuntimePassId,
}

#[derive(Clone, Debug, PartialEq)]
pub(super) struct RuntimePassPreparationRequest {
    pub(super) runtime: RuntimePass,
    pub(super) spatial_uniform: Option<PassSpatialUniformBytes>,
    pub(super) blur_edge_parameters: Option<BlurEdgeParameterBytes>,
    pub(super) color_filter_operations: Option<ColorFilterOperationBytes>,
    pub(super) drop_shadow_parameters: Option<DropShadowParameterBytes>,
    pub(super) composite_parameters: Option<CompositeParameterBytes>,
    pub(super) cache_keys: Option<RuntimePassCacheKeys>,
    pub(super) kernel: Option<GaussianKernelKey>,
    pub(super) kernel_releases: SharedValue<Vec<GaussianKernelKey>>,
    pub(super) convolution_parameters: Option<ConvolutionParameterBytes>,
    pub(super) convolution_weights: Option<RuntimePassId>,
}

#[derive(Clone, Debug, PartialEq)]
pub(super) struct RuntimeConvolutionWeightsRequest {
    pub(super) owner: RuntimePassId,
    pub(super) plan: ConvolutionWeightsPlan,
    pub(super) last_use: RuntimePassId,
}

#[derive(Debug, PartialEq)]
pub(super) struct RuntimeGraphPreparationPlan {
    pub(super) generation: RuntimeGraphGeneration,
    pub(super) working_format: WorkingFormat,
    pub(super) output_format: Format,
    pub(super) resources: AccountedVec<RuntimeResourcePreparationRequest>,
    pub(super) kernels: AccountedVec<RuntimeKernelPreparationRequest>,
    pub(super) convolution_weights: AccountedVec<RuntimeConvolutionWeightsRequest>,
    pub(super) passes: AccountedVec<RuntimePassPreparationRequest>,
    pub(super) root_working_image: RuntimeResourceId,
    pub(super) final_present: RuntimePassId,
    pub(super) allocation_preflights: AccountedVec<ResourceAllocationPreflight>,
}

impl RuntimeGraphPreparationPlan {
    fn try_derive(
        lowered: &LoweredGraphPlan,
        selected_working_format: WorkingFormat,
        capabilities: &DeviceCapabilities,
        device: &wgpu::Device,
        account: &Account,
    ) -> Result<Self> {
        Self::try_derive_with_color_filter_limits(
            lowered,
            selected_working_format,
            capabilities,
            device,
            ColorFilterOperationBufferLimits::from_device_limits(&device.limits()),
            account,
        )
    }

    fn try_derive_with_color_filter_limits(
        lowered: &LoweredGraphPlan,
        selected_working_format: WorkingFormat,
        capabilities: &DeviceCapabilities,
        device: &wgpu::Device,
        color_filter_limits: ColorFilterOperationBufferLimits,
        account: &Account,
    ) -> Result<Self> {
        capabilities.validate_supported_working_format(selected_working_format)?;
        if selected_working_format != lowered.working_format {
            return Err(preparation_error(
                "the lowered graph working format does not match immutable device policy",
            ));
        }
        let mut prepared_resources = prepare_runtime_resources(lowered, capabilities, account)?;
        let pass_facts =
            analyze_runtime_passes(lowered, &prepared_resources.resource_by_id, device, account)?;
        validate_prepared_graph_lifetimes(
            lowered,
            &prepared_resources.resource_by_id,
            &pass_facts,
            account,
        )?;
        let mut passes = prepare_runtime_pass_requests(
            lowered,
            &prepared_resources.resource_by_id,
            &pass_facts,
            color_filter_limits,
            account,
        )?;
        let convolution_weights = prepare_convolution_weights(&mut passes, device, account)?;
        let mut kernels = account.retained(pass_facts.kernels.len())?;
        for kernel in pass_facts.kernels.into_values() {
            account.work(RenderWork::GraphNodeVisits, 1)?;
            kernels.append_slot()?.write(kernel);
        }
        for kernel in &kernels {
            account.work(RenderWork::GraphNodeVisits, 1)?;
            prepared_resources
                .allocation_preflights
                .push(ResourceAllocationPreflight::gaussian_kernel(&kernel.plan)?)?;
        }
        for weights in &convolution_weights {
            account.work(RenderWork::GraphNodeVisits, 1)?;
            prepared_resources.allocation_preflights.push(
                ResourceAllocationPreflight::convolution_weights(&weights.plan)?,
            )?;
        }

        Ok(Self {
            generation: lowered.generation,
            working_format: lowered.working_format,
            output_format: lowered.output_format,
            resources: prepared_resources.resources,
            kernels,
            convolution_weights,
            passes,
            root_working_image: lowered.root_working_image,
            final_present: lowered.final_present,
            allocation_preflights: prepared_resources.allocation_preflights,
        })
    }
}

struct PreparedRuntimeResources<'a> {
    resource_by_id: PassMap<RuntimeResourceId, &'a RuntimeResourceRequest>,
    resources: AccountedVec<RuntimeResourcePreparationRequest>,
    allocation_preflights: AccountedVec<ResourceAllocationPreflight>,
}

fn prepare_runtime_resources<'a>(
    lowered: &'a LoweredGraphPlan,
    capabilities: &DeviceCapabilities,
    account: &Account,
) -> Result<PreparedRuntimeResources<'a>> {
    let mut prepared = PreparedRuntimeResources {
        resource_by_id: PassMap::temporary(account, lowered.resources.len())?,
        resources: account.retained(lowered.resources.len())?,
        allocation_preflights: account.retained(lowered.resources.len())?,
    };
    for resource in &lowered.resources {
        account.work(RenderWork::GraphNodeVisits, 1)?;
        if prepared
            .resource_by_id
            .insert_with(resource.id, resource, account)?
            .is_some()
        {
            return Err(preparation_error(
                "duplicate runtime resource reached graph preparation",
            ));
        }
        validate_runtime_resource_request(resource, lowered.working_format, capabilities)?;
        let allocation =
            prepare_runtime_resource_allocation(resource, lowered.working_format, capabilities)?;
        prepared
            .allocation_preflights
            .push(allocation.preflight(account)?)?;
        prepared.resources.push(RuntimeResourcePreparationRequest {
            runtime: resource.clone(),
            allocation,
        })?;
    }
    Ok(prepared)
}

fn validate_runtime_resource_request(
    resource: &RuntimeResourceRequest,
    working_format: WorkingFormat,
    capabilities: &DeviceCapabilities,
) -> Result<()> {
    if runtime_resource_format(resource.role, working_format) != resource.format {
        return Err(preparation_error(
            "runtime resource role and working format are inconsistent",
        ));
    }
    if resource.expected_reads == 0 {
        return Err(preparation_error(
            "a prepared runtime resource has no scheduled reader",
        ));
    }
    let extent = resource.spatial.device_extent;
    if extent.width() == 0 || extent.height() == 0 {
        return Err(preparation_error(
            "a concrete runtime resource has an empty allocation extent",
        ));
    }
    capabilities.validate_effect_texture_extent(extent)
}

fn prepare_runtime_resource_allocation(
    resource: &RuntimeResourceRequest,
    working_format: WorkingFormat,
    capabilities: &DeviceCapabilities,
) -> Result<RuntimeAllocationRequest> {
    let extent = resource.spatial.device_extent;
    match (&resource.format, &resource.import) {
        (RuntimeResourceFormat::ConvolutionRgba32Float, None)
            if resource.role == RuntimeResourceRole::ConvolutionFloat
                && matches!(resource.producer, RuntimeResourceProducer::Pass(_)) =>
        {
            let descriptor = EffectTextureDescriptor::try_convolution_float32(
                extent,
                wgpu::TextureUsages::RENDER_ATTACHMENT
                    .union(wgpu::TextureUsages::TEXTURE_BINDING)
                    .union(wgpu::TextureUsages::COPY_SRC)
                    .union(wgpu::TextureUsages::COPY_DST),
            )?;
            capabilities.validate_effect_texture_allocation(
                extent,
                None,
                descriptor.texture_format(),
                descriptor.usage(),
            )?;
            Ok(RuntimeAllocationRequest::EffectTexture(descriptor))
        }
        (RuntimeResourceFormat::VelloCaptureRgba8Unorm, None)
            if resource.role == RuntimeResourceRole::CaptureWorkingImage
                && matches!(resource.producer, RuntimeResourceProducer::Pass(_)) =>
        {
            let descriptor =
                EffectTextureDescriptor::try_capture(extent, VELLO_CAPTURE_TEXTURE_USAGES)?;
            capabilities.validate_effect_texture_allocation(
                extent,
                None,
                descriptor.texture_format(),
                descriptor.usage(),
            )?;
            Ok(RuntimeAllocationRequest::EffectTexture(descriptor))
        }
        (RuntimeResourceFormat::ClipCoverageRgba8Unorm, None)
            if resource.role == RuntimeResourceRole::ClipCoverage
                && matches!(resource.producer, RuntimeResourceProducer::Pass(_)) =>
        {
            let descriptor =
                EffectTextureDescriptor::try_coverage(extent, VELLO_CAPTURE_TEXTURE_USAGES)?;
            capabilities.validate_effect_texture_allocation(
                extent,
                None,
                descriptor.texture_format(),
                descriptor.usage(),
            )?;
            Ok(RuntimeAllocationRequest::EffectTexture(descriptor))
        }
        (RuntimeResourceFormat::Working(format), None)
            if *format == working_format
                && resource.role != RuntimeResourceRole::CaptureWorkingImage
                && resource.role != RuntimeResourceRole::ClipCoverage
                && resource.role != RuntimeResourceRole::ImportedImage =>
        {
            let descriptor =
                EffectTextureDescriptor::try_working(*format, extent, format.required_usages())?;
            capabilities.validate_effect_texture_allocation(
                extent,
                Some(*format),
                descriptor.texture_format(),
                descriptor.usage(),
            )?;
            Ok(RuntimeAllocationRequest::EffectTexture(descriptor))
        }
        (
            RuntimeResourceFormat::ResolvedMaskRgba8Unorm,
            Some(RuntimeResourceImport::ResolvedAlphaMask(descriptor)),
        ) if resource.role == RuntimeResourceRole::ImportedImage
            && matches!(resource.producer, RuntimeResourceProducer::Imported) =>
        {
            if descriptor.physical_size() != extent {
                return Err(preparation_error(
                    "resolved-mask upload extent differs from its runtime resource",
                ));
            }
            descriptor.validate_upload_byte_len(descriptor.bytes().len())?;
            capabilities.validate_effect_texture_allocation(
                extent,
                None,
                wgpu::TextureFormat::Rgba8Unorm,
                RESOLVED_MASK_TEXTURE_USAGES,
            )?;
            Ok(RuntimeAllocationRequest::ResolvedMask(descriptor.clone()))
        }
        _ => Err(preparation_error(
            "runtime resource has no exact concrete preparation request",
        )),
    }
}

struct RuntimePassAnalysis {
    actual_reads: PassMap<RuntimeResourceId, u32>,
    actual_last_reads: PassMap<RuntimeResourceId, RuntimePassId>,
    release_passes: PassMap<RuntimeResourceId, RuntimePassId>,
    produced_results: PassMap<RuntimeResourceId, RuntimePassId>,
    kernel_by_pass: PassMap<RuntimePassId, GaussianKernelKey>,
    kernels: PassMap<GaussianKernelKey, RuntimeKernelPreparationRequest>,
}

fn analyze_runtime_passes(
    lowered: &LoweredGraphPlan,
    resources: &PassMap<RuntimeResourceId, &RuntimeResourceRequest>,
    device: &wgpu::Device,
    account: &Account,
) -> Result<RuntimePassAnalysis> {
    let mut pass_positions = PassMap::temporary(account, lowered.passes.len())?;
    let mut kernel_bound = 0usize;
    for (position, pass) in lowered.passes.iter().enumerate() {
        account.work(RenderWork::GraphNodeVisits, 1)?;
        if runtime_blur_for_kernel(&pass.kind).is_some() {
            kernel_bound = kernel_bound
                .checked_add(1)
                .ok_or(crate::GlyphPreparationError::CapacityOverflow)?;
        }
        if pass_positions
            .insert_with(pass.id, position, account)?
            .is_some()
        {
            return Err(preparation_error(
                "duplicate runtime pass reached graph preparation",
            ));
        }
    }
    let mut facts = RuntimePassAnalysis {
        actual_reads: PassMap::temporary(account, lowered.resources.len())?,
        actual_last_reads: PassMap::temporary(account, lowered.resources.len())?,
        release_passes: PassMap::temporary(account, lowered.resources.len())?,
        produced_results: PassMap::temporary(account, lowered.resources.len())?,
        kernel_by_pass: PassMap::temporary(account, kernel_bound)?,
        kernels: PassMap::temporary(account, kernel_bound)?,
    };
    for (position, pass) in lowered.passes.iter().enumerate() {
        account.work(RenderWork::GraphNodeVisits, 1)?;
        let pass_reads = analyze_runtime_pass(
            pass,
            position,
            lowered,
            resources,
            &pass_positions,
            &mut facts,
            account,
        )?;
        analyze_runtime_kernel(pass, device, &mut facts, account)?;
        drop(pass_reads);
    }
    Ok(facts)
}

fn analyze_runtime_pass(
    pass: &RuntimePass,
    position: usize,
    lowered: &LoweredGraphPlan,
    resources: &PassMap<RuntimeResourceId, &RuntimeResourceRequest>,
    pass_positions: &PassMap<RuntimePassId, usize>,
    facts: &mut RuntimePassAnalysis,
    account: &Account,
) -> Result<PassSet<RuntimeResourceId>> {
    for dependency in pass.dependencies.iter() {
        account.work(RenderWork::GraphEdgeVisits, 1)?;
        if pass_positions
            .get(dependency, account)?
            .is_none_or(|dependency_position| *dependency_position >= position)
        {
            return Err(preparation_error(
                "prepared pass has a missing or forward dependency",
            ));
        }
    }
    let pass_reads =
        analyze_runtime_reads(pass, position, resources, pass_positions, facts, account)?;
    analyze_runtime_result(pass, lowered, resources, &pass_reads, facts, account)?;
    let expected_cache_keys = runtime_pass_cache_keys(
        &pass.kind,
        &pass.reads,
        pass.result,
        lowered.working_format,
        lowered.output_format,
        |id| Ok(resources.get(&id, account)?.map(|resource| resource.format)),
        account,
    )?;
    let keys_match = match (&expected_cache_keys, &pass.cache_keys) {
        (Some(expected), Some(actual)) => expected.matches_in(actual, account)?,
        (None, None) => true,
        _ => false,
    };
    if !keys_match {
        return Err(preparation_error(
            "prepared pass cache keys differ from exact runtime lowering",
        ));
    }
    for resource in &pass.releases {
        account.work(RenderWork::GraphEdgeVisits, 1)?;
        if !pass_reads.contains_with(resource, account)?
            || facts
                .release_passes
                .insert_with(*resource, pass.id, account)?
                .is_some()
        {
            return Err(preparation_error(
                "prepared pass release is missing, duplicate, or not a last read",
            ));
        }
    }
    Ok(pass_reads)
}

fn analyze_runtime_reads(
    pass: &RuntimePass,
    position: usize,
    resources: &PassMap<RuntimeResourceId, &RuntimeResourceRequest>,
    pass_positions: &PassMap<RuntimePassId, usize>,
    facts: &mut RuntimePassAnalysis,
    account: &Account,
) -> Result<PassSet<RuntimeResourceId>> {
    let mut pass_reads = PassSet::new(account, pass.reads.len())?;
    for read in &pass.reads {
        account.work(RenderWork::GraphEdgeVisits, 1)?;
        if !pass_reads.insert_with(read.resource, account)? {
            return Err(preparation_error(
                "prepared pass contains a duplicate runtime read binding",
            ));
        }
        let resource = resources
            .get(&read.resource, account)?
            .ok_or_else(|| preparation_error("prepared pass names a missing runtime resource"))?;
        if let RuntimeResourceProducer::Pass(producer) = resource.producer
            && pass_positions
                .get(&producer, account)?
                .is_none_or(|producer_position| *producer_position >= position)
        {
            return Err(preparation_error(
                "prepared pass reads before its runtime resource producer",
            ));
        }
        let reads = facts
            .actual_reads
            .get(&read.resource, account)?
            .copied()
            .unwrap_or(0u32)
            .checked_add(1)
            .ok_or_else(|| preparation_error("prepared runtime read count overflowed"))?;
        facts
            .actual_reads
            .insert_with(read.resource, reads, account)?;
        facts
            .actual_last_reads
            .insert_with(read.resource, pass.id, account)?;
    }
    Ok(pass_reads)
}

fn analyze_runtime_result(
    pass: &RuntimePass,
    lowered: &LoweredGraphPlan,
    resources: &PassMap<RuntimeResourceId, &RuntimeResourceRequest>,
    pass_reads: &PassSet<RuntimeResourceId>,
    facts: &mut RuntimePassAnalysis,
    account: &Account,
) -> Result<()> {
    match pass.result {
        RuntimeResultBinding::Resource(resource_id) => {
            let resource = resources
                .get(&resource_id, account)?
                .ok_or_else(|| preparation_error("prepared pass result resource is missing"))?;
            if resource.producer != RuntimeResourceProducer::Pass(pass.id)
                || pass_reads.contains_with(&resource_id, account)?
                || facts
                    .produced_results
                    .insert_with(resource_id, pass.id, account)?
                    .is_some()
            {
                return Err(preparation_error(
                    "prepared pass result binding has no unique matching producer",
                ));
            }
        }
        RuntimeResultBinding::Output(format)
            if !matches!(pass.kind, RuntimePassKind::Present)
                || format != lowered.output_format =>
        {
            return Err(preparation_error(
                "prepared output binding differs from the terminal present target",
            ));
        }
        RuntimeResultBinding::Output(_) | RuntimeResultBinding::Empty => {}
    }
    Ok(())
}

fn analyze_runtime_kernel(
    pass: &RuntimePass,
    device: &wgpu::Device,
    facts: &mut RuntimePassAnalysis,
    account: &Account,
) -> Result<()> {
    let Some(blur) = runtime_blur_for_kernel(&pass.kind) else {
        return Ok(());
    };
    let kernel_plan = GaussianKernelPlan::try_new(
        blur.standard_deviation,
        blur.spatial.result.raster_scale,
        CSS_FILTER_KERNEL_SUPPORT_STANDARD_DEVIATIONS,
        GaussianKernelSamplingForm::PairedLinear,
        account,
    )?;
    if kernel_plan.key() != blur.kernel
        || kernel_plan
            .validate_buffer_limits(GaussianKernelBufferLimits::from_device_limits(
                &device.limits(),
            ))
            .is_err()
    {
        return Err(preparation_error(
            "Gaussian kernel preparation differs from the exact runtime blur plan",
        ));
    }
    facts
        .kernel_by_pass
        .insert_with(pass.id, blur.kernel, account)?;
    match facts.kernels.position(&blur.kernel, account)? {
        Err(index) => facts.kernels.insert_at_reserved(
            index,
            blur.kernel,
            RuntimeKernelPreparationRequest {
                key: blur.kernel,
                plan: kernel_plan,
                last_use: pass.id,
            },
        )?,
        Ok(index) => {
            let entry = &mut facts.kernels.entries[index].1;
            if !entry.plan.matches_in(&kernel_plan, account)? {
                return Err(preparation_error(
                    "one Gaussian kernel identity names conflicting plans",
                ));
            }
            entry.last_use = pass.id;
        }
    }
    Ok(())
}

fn validate_prepared_graph_lifetimes(
    lowered: &LoweredGraphPlan,
    resources: &PassMap<RuntimeResourceId, &RuntimeResourceRequest>,
    facts: &RuntimePassAnalysis,
    account: &Account,
) -> Result<()> {
    for resource in &lowered.resources {
        account.work(RenderWork::GraphNodeVisits, 1)?;
        if facts
            .actual_reads
            .get(&resource.id, account)?
            .copied()
            .unwrap_or(0)
            != resource.expected_reads
            || facts.actual_last_reads.get(&resource.id, account)?.copied()
                != Some(resource.last_use)
            || facts.release_passes.get(&resource.id, account)?.copied() != Some(resource.last_use)
        {
            return Err(preparation_error(
                "prepared runtime resource lifetime differs from exact lowering",
            ));
        }
        match resource.producer {
            RuntimeResourceProducer::Imported if resource.import.is_some() => {}
            RuntimeResourceProducer::Pass(pass)
                if resource.import.is_none()
                    && facts.produced_results.get(&resource.id, account)?.copied()
                        == Some(pass) => {}
            RuntimeResourceProducer::Imported | RuntimeResourceProducer::Pass(_) => {
                return Err(preparation_error(
                    "prepared runtime resource producer/import binding is inconsistent",
                ));
            }
        }
    }
    validate_prepared_graph_root(lowered, resources, account)
}

fn validate_prepared_graph_root(
    lowered: &LoweredGraphPlan,
    resources: &PassMap<RuntimeResourceId, &RuntimeResourceRequest>,
    account: &Account,
) -> Result<()> {
    let root = resources
        .get(&lowered.root_working_image, account)?
        .ok_or_else(|| preparation_error("prepared root working resource is missing"))?;
    if root.format != RuntimeResourceFormat::Working(lowered.working_format) {
        return Err(preparation_error(
            "prepared root resource does not use the selected working format",
        ));
    }
    let terminal_matches = match lowered.passes.last() {
        Some(pass) => {
            account.work(RenderWork::GraphNodeVisits, 1)?;
            pass.id == lowered.final_present
                && matches!(pass.kind, RuntimePassKind::Present)
                && pass.result == RuntimeResultBinding::Output(lowered.output_format)
        }
        None => false,
    };
    if !terminal_matches {
        return Err(preparation_error(
            "prepared graph has no exact terminal present binding",
        ));
    }
    Ok(())
}

fn prepare_runtime_pass_requests(
    lowered: &LoweredGraphPlan,
    resources: &PassMap<RuntimeResourceId, &RuntimeResourceRequest>,
    facts: &RuntimePassAnalysis,
    color_filter_limits: ColorFilterOperationBufferLimits,
    account: &Account,
) -> Result<AccountedVec<RuntimePassPreparationRequest>> {
    let mut requests = account.retained(lowered.passes.len())?;
    for pass in lowered.passes.iter() {
        account.work(RenderWork::GraphNodeVisits, 1)?;
        let spatial_uniform =
            prepared_pass_spatial_uniform(pass, resources, lowered.root_working_image, account)?;
        let blur_edge_parameters = prepare_blur_edge_parameters(pass, account)?;
        let color_filter_operations =
            prepare_color_filter_operations(pass, color_filter_limits, account)?;
        let drop_shadow_parameters = prepare_drop_shadow_parameters(pass, account)?;
        let composite_parameters = prepared_pass_composite_parameters(pass, resources, account)?;
        let convolution_parameters = match &pass.kind {
            RuntimePassKind::Convolution(operation) => {
                Some(super::convolution::parameters(operation, account)?)
            }
            _ => None,
        };
        if (spatial_uniform.is_some() || convolution_parameters.is_some())
            != pass.cache_keys.is_some()
        {
            return Err(preparation_error(
                "prepared pass spatial bytes and executable cache keys disagree",
            ));
        }
        let mut kernel_releases = account.retained(0)?;
        for kernel in facts.kernels.values() {
            account.work(RenderWork::GraphEdgeVisits, 1)?;
            if kernel.last_use == pass.id {
                kernel_releases.push(kernel.key)?;
            }
        }
        requests
            .append_slot()?
            .write(RuntimePassPreparationRequest {
                runtime: pass.clone(),
                spatial_uniform,
                blur_edge_parameters,
                color_filter_operations,
                drop_shadow_parameters,
                composite_parameters,
                cache_keys: pass.cache_keys.clone(),
                kernel: facts.kernel_by_pass.get(&pass.id, account)?.copied(),
                kernel_releases: SharedValue::from_accounted(kernel_releases, &account.storage())?,
                convolution_parameters,
                convolution_weights: None,
            });
    }
    Ok(requests)
}

fn prepare_convolution_weights(
    passes: &mut [RuntimePassPreparationRequest],
    device: &wgpu::Device,
    account: &Account,
) -> Result<AccountedVec<RuntimeConvolutionWeightsRequest>> {
    use crate::frame::ConvolutionStage;
    let mut count = 0usize;
    for pass in passes.iter() {
        account.work(RenderWork::GraphNodeVisits, 1)?;
        if matches!(&pass.runtime.kind, RuntimePassKind::Convolution(op) if matches!(op.stage(), ConvolutionStage::Prepare | ConvolutionStage::PassThrough | ConvolutionStage::RebaseSource))
        {
            count = count
                .checked_add(1)
                .ok_or(crate::GlyphPreparationError::CapacityOverflow)?;
        }
    }
    let mut weights = account.retained::<RuntimeConvolutionWeightsRequest>(count)?;
    let mut active = None;
    for pass in passes {
        account.work(RenderWork::GraphNodeVisits, 1)?;
        let RuntimePassKind::Convolution(op) = &pass.runtime.kind else {
            continue;
        };
        if matches!(
            op.stage(),
            ConvolutionStage::Prepare
                | ConvolutionStage::PassThrough
                | ConvolutionStage::RebaseSource
        ) {
            if active.is_some() {
                return Err(preparation_error(
                    "convolution weight chain was not completed",
                ));
            }
            let input = if matches!(
                op.stage(),
                ConvolutionStage::PassThrough | ConvolutionStage::RebaseSource
            ) {
                &[][..]
            } else {
                op.operation().weights()
            };
            let plan = ConvolutionWeightsPlan::try_new(input, account, &device.limits())?;
            let index = weights.len();
            weights
                .append_slot()?
                .write(RuntimeConvolutionWeightsRequest {
                    owner: pass.runtime.id,
                    plan,
                    last_use: pass.runtime.id,
                });
            active = Some(index);
        }
        let index = active.ok_or_else(|| {
            preparation_error("convolution stage has no prepared coefficient owner")
        })?;
        let request = &mut weights[index];
        pass.convolution_weights = Some(request.owner);
        request.last_use = pass.runtime.id;
        if matches!(
            op.stage(),
            ConvolutionStage::Resolve
                | ConvolutionStage::PassThrough
                | ConvolutionStage::RebaseSource
        ) {
            active = None;
        }
    }
    if active.is_some() {
        return Err(preparation_error(
            "convolution coefficient chain has no terminal stage",
        ));
    }
    Ok(weights)
}

fn runtime_blur_for_kernel(kind: &RuntimePassKind) -> Option<&RuntimeBlur> {
    match kind {
        RuntimePassKind::BlurHorizontal(Some(blur)) | RuntimePassKind::BlurVertical(Some(blur)) => {
            Some(blur)
        }
        _ => None,
    }
}

pub(super) struct PreparedResourceBinding {
    pub(super) allocation: RuntimeAllocationRequest,
    pub(super) lease: Option<ResourceLease>,
}

pub(super) struct PreparedKernelBinding {
    pub(super) lease: Option<ResourceLease>,
}

pub(super) struct PreparedConvolutionWeightsBinding {
    pub(super) lease: Option<ResourceLease>,
    pub(super) last_use: RuntimePassId,
}

pub(super) struct PreparedColorFilterOperationBinding {
    pub(super) bytes: ColorFilterOperationBytes,
    pub(super) buffer: Option<OwnedGpu<wgpu::Buffer>>,
}

pub(super) enum PreparedSpatialFilterPassObjects {
    CopyBackdrop {
        parent_sampler: OwnedGpu<wgpu::Sampler>,
        layout: OwnedGpu<wgpu::BindGroupLayout>,
        pipeline: OwnedGpu<wgpu::RenderPipeline>,
    },
    Blur {
        source_sampler: OwnedGpu<wgpu::Sampler>,
        layout: OwnedGpu<wgpu::BindGroupLayout>,
        pipeline: OwnedGpu<wgpu::RenderPipeline>,
    },
    DropShadowColorize {
        source_sampler: OwnedGpu<wgpu::Sampler>,
        layout: OwnedGpu<wgpu::BindGroupLayout>,
        pipeline: OwnedGpu<wgpu::RenderPipeline>,
    },
}

struct PreparedPassRealization {
    update: Option<ProvisionalDevicePassCacheUpdate>,
    spatial_filter_objects: PassMap<RuntimePassId, PreparedSpatialFilterPassObjects>,
}

#[must_use = "the closed graph dispatch result must select exactly one renderer route"]
pub(crate) enum ExecutableGraphDispatchEligibility {
    ExactBase(BasePreparableGraph),
    ExactComposition(CompositionPreparableGraph),
    ExactBackdrop(BackdropPreparableGraph),
    ExactForegroundFilters(ForegroundFilterPreparableGraph),
    FuturePasses,
}

impl ExecutableGraphDispatchEligibility {
    fn try_classify_backdrop(
        graph: &GpuRenderGraph,
        output_format: Format,
        working_format: ExecutableGraphWorkingFormatRequest,
        capabilities: &DeviceCapabilities,
        preparable: BackdropPreparableGraph,
        account: &Account,
    ) -> Result<Self> {
        if !preparable.proves_closed_backdrop_facts(account)? {
            return Err(preparation_error(
                "backdrop classification lost its closed pre-allocation facts",
            ));
        }
        let working_format = working_format.resolve(capabilities)?;
        let lowered = LoweredGraphPlan::try_lower_validated_graph(
            graph,
            working_format,
            output_format,
            capabilities,
            account,
        )?;
        match PrePreparationGraphClassification::classify(lowered, account)? {
            PrePreparationGraphClassification::ExactBackdrop(preparable)
                if preparable.proves_closed_backdrop_facts(account)? =>
            {
                Ok(Self::ExactBackdrop(preparable))
            }
            PrePreparationGraphClassification::ExactBase(_)
            | PrePreparationGraphClassification::ExactComposition(_)
            | PrePreparationGraphClassification::ExactColorFilter(_)
            | PrePreparationGraphClassification::ExactSpatialFilter(_)
            | PrePreparationGraphClassification::ExactBackdrop(_)
            | PrePreparationGraphClassification::FuturePasses
            | PrePreparationGraphClassification::Ineligible(_) => Err(preparation_error(
                "checked backdrop dispatch lowering changed its closed eligibility result",
            )),
        }
    }

    fn try_classify_foreground_filters(
        graph: &GpuRenderGraph,
        output_format: Format,
        working_format: ExecutableGraphWorkingFormatRequest,
        capabilities: &DeviceCapabilities,
        preparable: ForegroundFilterPreparableGraph,
        account: &Account,
    ) -> Result<Self> {
        if !preparable.proves_closed_filter_facts(account)? {
            return Err(preparation_error(
                "filter classification lost its closed pre-allocation facts",
            ));
        }
        if !graph.has_foreground_filters() {
            return Ok(Self::FuturePasses);
        }

        let working_format = working_format.resolve(capabilities)?;
        let lowered = LoweredGraphPlan::try_lower_validated_graph(
            graph,
            working_format,
            output_format,
            capabilities,
            account,
        )?;
        let classified = PrePreparationGraphClassification::classify(lowered, account)?;
        let selected = match (preparable, classified) {
            (
                ForegroundFilterPreparableGraph::Color(_),
                PrePreparationGraphClassification::ExactColorFilter(graph),
            ) => ForegroundFilterPreparableGraph::Color(graph),
            (
                ForegroundFilterPreparableGraph::Spatial(_),
                PrePreparationGraphClassification::ExactSpatialFilter(graph),
            ) => ForegroundFilterPreparableGraph::Spatial(graph),
            _ => {
                return Err(preparation_error(
                    "checked foreground filter lowering changed its closed eligibility result",
                ));
            }
        };
        if !selected.proves_closed_filter_facts(account)? {
            return Err(preparation_error(
                "checked foreground filter lowering lost its closed pre-allocation facts",
            ));
        }
        Ok(Self::ExactForegroundFilters(selected))
    }

    pub(crate) fn try_classify(
        graph: &GpuRenderGraph,
        output_format: Format,
        working_format: ExecutableGraphWorkingFormatRequest,
        capabilities: &DeviceCapabilities,
        account: &Account,
    ) -> Result<Self> {
        let classification = PrePreparationGraphClassification::classify(
            LoweredGraphPlan::try_lower_for_dispatch_classification(
                graph,
                WorkingFormat::HighPrecision,
                output_format,
                account,
            )?,
            account,
        )?;
        match classification {
            PrePreparationGraphClassification::ExactBackdrop(preparable) => {
                Self::try_classify_backdrop(
                    graph,
                    output_format,
                    working_format,
                    capabilities,
                    preparable,
                    account,
                )
            }
            PrePreparationGraphClassification::ExactSpatialFilter(preparable) => {
                Self::try_classify_foreground_filters(
                    graph,
                    output_format,
                    working_format,
                    capabilities,
                    ForegroundFilterPreparableGraph::Spatial(preparable),
                    account,
                )
            }
            PrePreparationGraphClassification::ExactColorFilter(preparable) => {
                Self::try_classify_foreground_filters(
                    graph,
                    output_format,
                    working_format,
                    capabilities,
                    ForegroundFilterPreparableGraph::Color(preparable),
                    account,
                )
            }
            PrePreparationGraphClassification::ExactComposition(_) => {
                let working_format = working_format.resolve(capabilities)?;
                let lowered = LoweredGraphPlan::try_lower_validated_graph(
                    graph,
                    working_format,
                    output_format,
                    capabilities,
                    account,
                )?;
                match PrePreparationGraphClassification::classify(lowered, account,)? {
                    PrePreparationGraphClassification::ExactComposition(closed) => {
                        CompositionPreparableGraph::try_from_closed(closed)
                            .map(Self::ExactComposition)
                            .map_err(|_| {
                                preparation_error(
                                    "checked composition dispatch lowering lost its composition-only facts",
                                )
                            })
                    }
                    PrePreparationGraphClassification::ExactBase(_)
                    | PrePreparationGraphClassification::ExactColorFilter(_)
                    | PrePreparationGraphClassification::ExactSpatialFilter(_)
                    | PrePreparationGraphClassification::ExactBackdrop(_)
                    | PrePreparationGraphClassification::FuturePasses
                    | PrePreparationGraphClassification::Ineligible(_) => Err(preparation_error(
                        "checked composition dispatch lowering changed its closed eligibility result",
                    )),
                }
            }
            PrePreparationGraphClassification::FuturePasses => Ok(Self::FuturePasses),
            PrePreparationGraphClassification::Ineligible(ineligibility) => {
                Err(ineligibility.into_error())
            }
            PrePreparationGraphClassification::ExactBase(_) => {
                let working_format = working_format.resolve(capabilities)?;
                let lowered = LoweredGraphPlan::try_lower_validated_graph(
                    graph,
                    working_format,
                    output_format,
                    capabilities,
                    account,
                )?;
                match PrePreparationGraphClassification::classify(lowered, account)? {
                    PrePreparationGraphClassification::ExactBase(preparable) => {
                        Ok(Self::ExactBase(preparable))
                    }
                    PrePreparationGraphClassification::ExactComposition(_)
                    | PrePreparationGraphClassification::ExactColorFilter(_)
                    | PrePreparationGraphClassification::ExactSpatialFilter(_)
                    | PrePreparationGraphClassification::ExactBackdrop(_)
                    | PrePreparationGraphClassification::FuturePasses
                    | PrePreparationGraphClassification::Ineligible(_) => Err(preparation_error(
                        "checked base-graph dispatch lowering changed its closed eligibility result",
                    )),
                }
            }
        }
    }
}

pub(super) enum GraphPreparationSource<'source> {
    Base(&'source BasePreparableGraph),
    Composition(&'source ClosedExecutableGraph),
    ColorFilter {
        preparable: &'source ColorFilterPreparableGraph,
        operation_limits: Option<ColorFilterOperationBufferLimits>,
    },
    SpatialFilter(&'source SpatialFilterPreparableGraph),
    Backdrop(&'source BackdropPreparableGraph),
}

type GraphPreparationParts<'source> = (
    &'source LoweredGraphPlan,
    Option<BaseExecutionFacts>,
    Option<SharedValue<ClosedExecutableGraphFacts>>,
    Option<SharedValue<ClosedExecutableGraphFacts>>,
    Option<SharedValue<ClosedExecutableGraphFacts>>,
    Option<SharedValue<ClosedExecutableGraphFacts>>,
);

impl<'source> GraphPreparationSource<'source> {
    const fn color_filter_operation_limits(&self) -> Option<ColorFilterOperationBufferLimits> {
        match self {
            Self::ColorFilter {
                operation_limits, ..
            } => *operation_limits,
            Self::Base(_) | Self::Composition(_) | Self::SpatialFilter(_) | Self::Backdrop(_) => {
                None
            }
        }
    }

    fn parts(self) -> GraphPreparationParts<'source> {
        match self {
            Self::Base(preparable) => {
                let (lowered, execution) = preparable.parts();
                (lowered, Some(execution.clone()), None, None, None, None)
            }
            Self::Composition(closed) => (
                &closed.lowered,
                None,
                Some(closed.facts.clone()),
                None,
                None,
                None,
            ),
            Self::ColorFilter { preparable, .. } => {
                let closed = &preparable.closed;
                (
                    &closed.lowered,
                    None,
                    None,
                    Some(closed.facts.clone()),
                    None,
                    None,
                )
            }
            Self::SpatialFilter(preparable) => {
                let closed = &preparable.closed;
                (
                    &closed.lowered,
                    None,
                    None,
                    None,
                    Some(closed.facts.clone()),
                    None,
                )
            }
            Self::Backdrop(preparable) => {
                let closed = &preparable.closed;
                (
                    &closed.lowered,
                    None,
                    None,
                    None,
                    None,
                    Some(closed.facts.clone()),
                )
            }
        }
    }
}

pub(crate) struct PreparedGraph<'device> {
    pub(super) plan: RuntimeGraphPreparationPlan,
    pub(super) base_execution: Option<BaseExecutionFacts>,
    pub(super) composition_execution: Option<SharedValue<ClosedExecutableGraphFacts>>,
    pub(super) color_filter_execution: Option<SharedValue<ClosedExecutableGraphFacts>>,
    pub(super) spatial_filter_execution: Option<SharedValue<ClosedExecutableGraphFacts>>,
    pub(super) backdrop_execution: Option<SharedValue<ClosedExecutableGraphFacts>>,
    pub(super) resource_bindings: PassMap<RuntimeResourceId, PreparedResourceBinding>,
    pub(super) kernel_bindings: PassMap<GaussianKernelKey, PreparedKernelBinding>,
    pub(super) convolution_weight_bindings:
        PassMap<RuntimePassId, PreparedConvolutionWeightsBinding>,
    pub(super) color_filter_operation_bindings:
        PassMap<RuntimePassId, PreparedColorFilterOperationBinding>,
    pub(super) spatial_filter_pass_objects:
        PassMap<RuntimePassId, PreparedSpatialFilterPassObjects>,
    pub(super) pass_cache_update: Option<ProvisionalDevicePassCacheUpdate>,
    pub(super) frame_scope: Option<FrameResourceScope>,
    pub(super) next_pass: usize,
    pub(super) custom_spine_encoding_state: Option<CustomSpineEncodingState>,
    pub(super) custom_spine_completed_session: Option<Arc<()>>,
    #[cfg(test)]
    pub(super) acquired_capture_lease_count_raw_fact: usize,
    pub(super) device: &'device wgpu::Device,
    pub(super) queue: &'device wgpu::Queue,
    pub(super) vello_engine: Option<&'device VelloEngineState>,
    pub(super) resources: &'device ResourceManager,
    pub(super) pass_cache: &'device DevicePassCache,
    _ready_device: PhantomData<&'device ResourceManager>,
}

struct AcquiredGraphBindings {
    runtime_bindings: PassMap<RuntimeResourceId, PreparedResourceBinding>,
    gaussian_kernel_bindings: PassMap<GaussianKernelKey, PreparedKernelBinding>,
    convolution_weight_bindings: PassMap<RuntimePassId, PreparedConvolutionWeightsBinding>,
}

fn acquire_prepared_graph_resources(
    plan: &RuntimeGraphPreparationPlan,
    frame_scope: &mut FrameResourceScope,
    device: &wgpu::Device,
    queue: &wgpu::Queue,
    capabilities: &DeviceCapabilities,
    account: &Account,
) -> Result<AcquiredGraphBindings> {
    let mut runtime_bindings = PassMap::new(account, plan.resources.len())?;
    let mut gaussian_kernel_bindings = PassMap::new(account, plan.kernels.len())?;
    let mut convolution_weight_bindings = PassMap::new(account, plan.convolution_weights.len())?;
    for request in &plan.resources {
        account.work(RenderWork::GraphNodeVisits, 1)?;
        let Err(slot) = runtime_bindings.position(&request.runtime.id, account)? else {
            return Err(preparation_error(
                "one runtime resource acquired more than one concrete binding",
            ));
        };
        let lease = request
            .allocation
            .acquire(frame_scope, device, queue, capabilities)?;
        runtime_bindings.insert_at_reserved(
            slot,
            request.runtime.id,
            PreparedResourceBinding {
                allocation: request.allocation.clone(),
                lease: Some(lease),
            },
        )?;
    }
    for request in &plan.kernels {
        account.work(RenderWork::GraphNodeVisits, 1)?;
        let Err(slot) = gaussian_kernel_bindings.position(&request.key, account)? else {
            return Err(preparation_error(
                "one Gaussian kernel acquired more than one concrete binding",
            ));
        };
        let lease = frame_scope.acquire_gaussian_kernel_buffer(device, &request.plan)?;
        gaussian_kernel_bindings.insert_at_reserved(
            slot,
            request.key,
            PreparedKernelBinding { lease: Some(lease) },
        )?;
    }
    for request in &plan.convolution_weights {
        account.work(RenderWork::GraphNodeVisits, 1)?;
        let lease = frame_scope.acquire_convolution_weights(device, queue, &request.plan)?;
        convolution_weight_bindings.insert_with(
            request.owner,
            PreparedConvolutionWeightsBinding {
                lease: Some(lease),
                last_use: request.last_use,
            },
            account,
        )?;
    }
    Ok(AcquiredGraphBindings {
        runtime_bindings,
        gaussian_kernel_bindings,
        convolution_weight_bindings,
    })
}

fn create_color_filter_operation_bindings(
    generation: &Generation,
    plan: &RuntimeGraphPreparationPlan,
    device: &wgpu::Device,
    queue: &wgpu::Queue,
    account: &Account,
) -> Result<PassMap<RuntimePassId, PreparedColorFilterOperationBinding>> {
    let mut count = 0usize;
    for request in &plan.passes {
        account.work(RenderWork::GraphNodeVisits, 1)?;
        if request.color_filter_operations.is_some() {
            count = count
                .checked_add(1)
                .ok_or(crate::GlyphPreparationError::CapacityOverflow)?;
        }
    }
    let mut bindings = PassMap::new(account, count)?;
    for request in &plan.passes {
        account.work(RenderWork::GraphNodeVisits, 1)?;
        let Some(bytes) = request.color_filter_operations.as_ref() else {
            continue;
        };
        if !matches!(request.runtime.kind, RuntimePassKind::ColorFilter(Some(_)))
            || bytes.as_bytes().is_empty()
        {
            return Err(preparation_error(
                "prepared color-filter bytes have no exact runtime pass",
            ));
        }
        let size = u64::try_from(bytes.as_bytes().len()).map_err(|_| {
            preparation_error("prepared color-filter buffer length does not fit u64")
        })?;
        let Err(slot) = bindings.position(&request.runtime.id, account)? else {
            return Err(preparation_error(
                "one color-filter pass acquired more than one operation buffer",
            ));
        };
        let buffer = generation.allocation_with_upload(
            GenerationClass::PassBuffers,
            size,
            bytes.as_bytes().len(),
            Some(account),
            || {
                device.create_buffer(&wgpu::BufferDescriptor {
                    label: Some("Surgeist color-filter ordered color-filter operations"),
                    size,
                    usage: wgpu::BufferUsages::STORAGE.union(wgpu::BufferUsages::COPY_DST),
                    mapped_at_creation: false,
                })
            },
        )?;
        queue.write_buffer(&buffer, 0, bytes.as_bytes());
        bindings.insert_at_reserved(
            slot,
            request.runtime.id,
            PreparedColorFilterOperationBinding {
                bytes: bytes.clone(),
                buffer: Some(buffer),
            },
        )?;
    }
    Ok(bindings)
}

fn realize_prepared_graph_passes(
    plan: &RuntimeGraphPreparationPlan,
    device: &wgpu::Device,
    pass_cache: &DevicePassCache,
    enabled: bool,
    account: &Account,
) -> Result<PreparedPassRealization> {
    if !enabled {
        return Ok(PreparedPassRealization {
            update: None,
            spatial_filter_objects: PassMap::new(account, 0)?,
        });
    }
    let mut update = pass_cache.provisional_update();
    let mut realized_pass = false;
    let mut count = 0usize;
    for request in &plan.passes {
        account.work(RenderWork::GraphNodeVisits, 1)?;
        if request.cache_keys.is_some()
            && matches!(
                request.runtime.kind,
                RuntimePassKind::CopyBackdrop
                    | RuntimePassKind::BlurHorizontal(Some(_))
                    | RuntimePassKind::BlurVertical(Some(_))
                    | RuntimePassKind::DropShadowColorize(Some(_))
            )
        {
            count = count
                .checked_add(1)
                .ok_or(crate::GlyphPreparationError::CapacityOverflow)?;
        }
    }
    let mut spatial_filter_objects = PassMap::new(account, count)?;
    for request in &plan.passes {
        account.work(RenderWork::GraphNodeVisits, 1)?;
        let Some(keys) = request.cache_keys.as_ref() else {
            continue;
        };
        let retains_objects = matches!(
            request.runtime.kind,
            RuntimePassKind::CopyBackdrop
                | RuntimePassKind::BlurHorizontal(Some(_))
                | RuntimePassKind::BlurVertical(Some(_))
                | RuntimePassKind::DropShadowColorize(Some(_))
        );
        let slot = if retains_objects {
            let Err(slot) = spatial_filter_objects.position(&request.runtime.id, account)? else {
                return Err(preparation_error(
                    "one spatial-filter pass retained more than one prepared object set",
                ));
            };
            Some(slot)
        } else {
            None
        };
        let objects =
            realize_prepared_graph_pass(&mut update, device, pass_cache, request, keys, account)?;
        match (slot, objects) {
            (Some(slot), Some(objects)) => {
                spatial_filter_objects.insert_at_reserved(slot, request.runtime.id, objects)?
            }
            (None, None) => {}
            _ => {
                return Err(preparation_error(
                    "prepared pass realization changed its exact object retention contract",
                ));
            }
        }
        realized_pass = true;
    }
    Ok(PreparedPassRealization {
        update: realized_pass.then_some(update),
        spatial_filter_objects,
    })
}

fn realize_prepared_graph_pass(
    update: &mut ProvisionalDevicePassCacheUpdate,
    device: &wgpu::Device,
    pass_cache: &DevicePassCache,
    request: &RuntimePassPreparationRequest,
    keys: &RuntimePassCacheKeys,
    account: &Account,
) -> Result<Option<PreparedSpatialFilterPassObjects>> {
    match &request.runtime.kind {
        RuntimePassKind::CopyBackdrop => {
            let objects =
                realize_copy_backdrop_for_preparation(update, device, pass_cache, keys, account)?;
            Ok(Some(PreparedSpatialFilterPassObjects::CopyBackdrop {
                parent_sampler: objects.parent_sampler().clone(),
                layout: objects.bind_group_layout().clone(),
                pipeline: objects.render_pipeline().clone(),
            }))
        }
        RuntimePassKind::ColorFilter(Some(_)) => {
            update
                .realize_color_filter_pass(
                    device,
                    pass_cache,
                    account,
                    ColorFilterPassKeyRefs {
                        samplers: keys.samplers(),
                        layout: keys.layout(),
                        shader: keys.shader(),
                        pipeline: keys.pipeline(),
                    },
                )?
                .require_encoding_ready()?;
            Ok(None)
        }
        RuntimePassKind::Convolution(_) => {
            update
                .realize_convolution_pass(
                    device,
                    pass_cache,
                    account,
                    ConvolutionPassKeyRefs {
                        samplers: keys.samplers(),
                        layout: keys.layout(),
                        shader: keys.shader(),
                        pipeline: keys.pipeline(),
                    },
                )?
                .require_encoding_ready()?;
            Ok(None)
        }
        RuntimePassKind::BlurHorizontal(Some(_)) | RuntimePassKind::BlurVertical(Some(_)) => {
            let objects = update.realize_blur_pass(
                device,
                pass_cache,
                account,
                BlurPassKeyRefs {
                    samplers: keys.samplers(),
                    layout: keys.layout(),
                    shader: keys.shader(),
                    pipeline: keys.pipeline(),
                },
            )?;
            objects.require_encoding_ready()?;
            Ok(Some(PreparedSpatialFilterPassObjects::Blur {
                source_sampler: objects.source_sampler().clone(),
                layout: objects.bind_group_layout().clone(),
                pipeline: objects.render_pipeline().clone(),
            }))
        }
        RuntimePassKind::DropShadowColorize(Some(_)) => {
            let objects = update.realize_drop_shadow_colorize_pass(
                device,
                pass_cache,
                account,
                DropShadowColorizePassKeyRefs {
                    samplers: keys.samplers(),
                    layout: keys.layout(),
                    shader: keys.shader(),
                    pipeline: keys.pipeline(),
                },
            )?;
            objects.require_encoding_ready()?;
            Ok(Some(PreparedSpatialFilterPassObjects::DropShadowColorize {
                source_sampler: objects.source_sampler().clone(),
                layout: objects.bind_group_layout().clone(),
                pipeline: objects.render_pipeline().clone(),
            }))
        }
        RuntimePassKind::Composite(Some(RuntimeComposite {
            kind: RuntimeCompositeKind::Layer { .. },
            ..
        })) => {
            update
                .realize_composite_pass(
                    device,
                    pass_cache,
                    account,
                    CompositePassKeyRefs {
                        samplers: keys.samplers(),
                        layout: keys.layout(),
                        shader: keys.shader(),
                        pipeline: keys.pipeline(),
                    },
                )?
                .require_encoding_ready()?;
            Ok(None)
        }
        RuntimePassKind::CanonicalizeCapture
        | RuntimePassKind::Composite(Some(RuntimeComposite {
            kind: RuntimeCompositeKind::SpanSourceOver | RuntimeCompositeKind::DropShadow,
            ..
        }))
        | RuntimePassKind::Present => {
            update
                .realize_core_pass(
                    device,
                    pass_cache,
                    account,
                    CorePassKeyRefs {
                        samplers: keys.samplers(),
                        layout: keys.layout(),
                        shader: keys.shader(),
                        pipeline: keys.pipeline(),
                    },
                )?
                .require_encoding_ready()?;
            Ok(None)
        }
        RuntimePassKind::ClearRoot { .. }
        | RuntimePassKind::VelloCapture(_)
        | RuntimePassKind::ColorFilter(None)
        | RuntimePassKind::BlurHorizontal(None)
        | RuntimePassKind::BlurVertical(None)
        | RuntimePassKind::DropShadowColorize(None)
        | RuntimePassKind::Composite(None) => Err(preparation_error(
            "checked pass realization reached an unsupported graph pass",
        )),
    }
}

fn realize_copy_backdrop_for_preparation<'a>(
    update: &'a mut ProvisionalDevicePassCacheUpdate,
    device: &wgpu::Device,
    pass_cache: &'a DevicePassCache,
    keys: &RuntimePassCacheKeys,
    account: &Account,
) -> Result<ProvisionalCopyBackdropPassObjects<'a>> {
    let objects = update.realize_copy_backdrop_pass(
        device,
        pass_cache,
        account,
        CopyBackdropPassKeyRefs {
            samplers: keys.samplers(),
            layout: keys.layout(),
            shader: keys.shader(),
            pipeline: keys.pipeline(),
        },
    )?;
    objects.require_encoding_ready()?;
    Ok(objects)
}

impl<'device> PreparedGraph<'device> {
    pub(crate) fn try_prepare_base(
        preparable: &BasePreparableGraph,
        policy: EffectQualityPolicy,
        capabilities: &DeviceCapabilities,
        (device, queue): (&'device wgpu::Device, &'device wgpu::Queue),
        resources: &'device ResourceManager,
        pass_cache: &'device DevicePassCache,
        account: &Account,
    ) -> Result<Self> {
        let selected_working_format = capabilities.resolve_effect_working_format(policy)?;
        Self::try_prepare_base_with_working_format(
            preparable,
            selected_working_format,
            capabilities,
            (device, queue),
            resources,
            (pass_cache, false),
            account,
        )
    }

    pub(crate) fn try_prepare_base_with_working_format(
        preparable: &BasePreparableGraph,
        selected_working_format: WorkingFormat,
        capabilities: &DeviceCapabilities,
        (device, queue): (&'device wgpu::Device, &'device wgpu::Queue),
        resources: &'device ResourceManager,
        pass_cache_phase: (&'device DevicePassCache, bool),
        account: &Account,
    ) -> Result<Self> {
        let prepared = Self::try_prepare_inner(
            GraphPreparationSource::Base(preparable),
            selected_working_format,
            capabilities,
            (device, queue),
            resources,
            pass_cache_phase,
            account,
        )?;
        if prepared.base_execution_facts().is_none() {
            return Err(preparation_error(
                "base-graph preparation lost its validated execution facts",
            ));
        }
        Ok(prepared)
    }

    pub(crate) fn try_prepare_composition(
        preparable: &CompositionPreparableGraph,
        capabilities: &DeviceCapabilities,
        (device, queue): (&'device wgpu::Device, &'device wgpu::Queue),
        resources: &'device ResourceManager,
        pass_cache_phase: (&'device DevicePassCache, bool),
        account: &Account,
    ) -> Result<Self> {
        let selected_working_format = preparable.working_format();
        let prepared = Self::try_prepare_inner(
            GraphPreparationSource::Composition(preparable.closed()),
            selected_working_format,
            capabilities,
            (device, queue),
            resources,
            pass_cache_phase,
            account,
        )?;
        if prepared.composition_execution.is_none() {
            return Err(preparation_error(
                "composition preparation lost its validated closed execution facts",
            ));
        }
        Ok(prepared)
    }

    pub(crate) fn try_prepare_backdrop(
        preparable: &BackdropPreparableGraph,
        selected_working_format: WorkingFormat,
        capabilities: &DeviceCapabilities,
        (device, queue): (&'device wgpu::Device, &'device wgpu::Queue),
        resources: &'device ResourceManager,
        pass_cache_phase: (&'device DevicePassCache, bool),
        account: &Account,
    ) -> Result<Self> {
        let prepared = Self::try_prepare_inner(
            GraphPreparationSource::Backdrop(preparable),
            selected_working_format,
            capabilities,
            (device, queue),
            resources,
            pass_cache_phase,
            account,
        )?;
        if prepared.backdrop_execution.is_none() {
            return Err(preparation_error(
                "backdrop preparation lost its validated closed backdrop facts",
            ));
        }
        Ok(prepared)
    }

    pub(crate) fn try_prepare_color_filter(
        preparable: &ColorFilterPreparableGraph,
        selected_working_format: WorkingFormat,
        capabilities: &DeviceCapabilities,
        (device, queue): (&'device wgpu::Device, &'device wgpu::Queue),
        resources: &'device ResourceManager,
        pass_cache_phase: (&'device DevicePassCache, bool),
        account: &Account,
    ) -> Result<Self> {
        let prepared = Self::try_prepare_inner(
            GraphPreparationSource::ColorFilter {
                preparable,
                operation_limits: None,
            },
            selected_working_format,
            capabilities,
            (device, queue),
            resources,
            pass_cache_phase,
            account,
        )?;
        if prepared.color_filter_execution.is_none() {
            return Err(preparation_error(
                "color-filter preparation lost its validated closed execution facts",
            ));
        }
        Ok(prepared)
    }

    pub(crate) fn try_prepare_spatial_filter(
        preparable: &SpatialFilterPreparableGraph,
        selected_working_format: WorkingFormat,
        capabilities: &DeviceCapabilities,
        (device, queue): (&'device wgpu::Device, &'device wgpu::Queue),
        resources: &'device ResourceManager,
        pass_cache_phase: (&'device DevicePassCache, bool),
        account: &Account,
    ) -> Result<Self> {
        let prepared = Self::try_prepare_inner(
            GraphPreparationSource::SpatialFilter(preparable),
            selected_working_format,
            capabilities,
            (device, queue),
            resources,
            pass_cache_phase,
            account,
        )?;
        if prepared.spatial_filter_execution.is_none() {
            return Err(preparation_error(
                "spatial-filter preparation lost its validated closed execution facts",
            ));
        }
        Ok(prepared)
    }

    pub(crate) fn try_prepare(
        lowered: LoweredGraphPlan,
        policy: EffectQualityPolicy,
        capabilities: &DeviceCapabilities,
        (device, queue): (&'device wgpu::Device, &'device wgpu::Queue),
        resources: &'device ResourceManager,
        pass_cache_phase: (&'device DevicePassCache, bool),
        account: &Account,
    ) -> Result<Self> {
        match PrePreparationGraphClassification::classify(lowered, account)? {
            PrePreparationGraphClassification::ExactBase(preparable) if pass_cache_phase.1 => {
                let selected_working_format = capabilities.resolve_effect_working_format(policy)?;
                let prepared = Self::try_prepare_inner(
                    GraphPreparationSource::Base(&preparable),
                    selected_working_format,
                    capabilities,
                    (device, queue),
                    resources,
                    pass_cache_phase,
                    account,
                )?;
                if prepared.base_execution_facts().is_none() {
                    return Err(preparation_error(
                        "base-graph preparation lost its validated execution facts",
                    ));
                }
                Ok(prepared)
            }
            PrePreparationGraphClassification::ExactBase(preparable) => Self::try_prepare_base(
                &preparable,
                policy,
                capabilities,
                (device, queue),
                resources,
                pass_cache_phase.0,
                account,
            ),
            PrePreparationGraphClassification::ExactComposition(closed) => {
                let selected_working_format = capabilities.resolve_effect_working_format(policy)?;
                Self::try_prepare_inner(
                    GraphPreparationSource::Composition(&closed),
                    selected_working_format,
                    capabilities,
                    (device, queue),
                    resources,
                    pass_cache_phase,
                    account,
                )
            }
            PrePreparationGraphClassification::ExactColorFilter(preparable) => {
                let selected_working_format = capabilities.resolve_effect_working_format(policy)?;
                Self::try_prepare_color_filter(
                    &preparable,
                    selected_working_format,
                    capabilities,
                    (device, queue),
                    resources,
                    pass_cache_phase,
                    account,
                )
            }
            PrePreparationGraphClassification::ExactSpatialFilter(preparable) => {
                let selected_working_format = capabilities.resolve_effect_working_format(policy)?;
                Self::try_prepare_spatial_filter(
                    &preparable,
                    selected_working_format,
                    capabilities,
                    (device, queue),
                    resources,
                    pass_cache_phase,
                    account,
                )
            }
            PrePreparationGraphClassification::ExactBackdrop(preparable) => {
                let selected_working_format = capabilities.resolve_effect_working_format(policy)?;
                Self::try_prepare_backdrop(
                    &preparable,
                    selected_working_format,
                    capabilities,
                    (device, queue),
                    resources,
                    pass_cache_phase,
                    account,
                )
            }
            PrePreparationGraphClassification::FuturePasses => Err(preparation_error(
                "a future GPU pass cannot enter composition resource preparation",
            )),
            PrePreparationGraphClassification::Ineligible(ineligibility) => {
                Err(ineligibility.into_error())
            }
        }
    }

    pub(super) fn try_prepare_inner(
        source: GraphPreparationSource<'_>,
        selected_working_format: WorkingFormat,
        capabilities: &DeviceCapabilities,
        (device, queue): (&'device wgpu::Device, &'device wgpu::Queue),
        resources: &'device ResourceManager,
        pass_cache_phase: (&'device DevicePassCache, bool),
        account: &Account,
    ) -> Result<Self> {
        let account = &account.at(RenderPhase::GpuPrepare);
        account.phase(RenderPhase::GpuPrepare, || {
            let (pass_cache, realize_checked_passes) = pass_cache_phase;
            let color_filter_operation_limits = source.color_filter_operation_limits();
            let (
                lowered,
                base_execution,
                composition_execution,
                color_filter_execution,
                spatial_filter_execution,
                backdrop_execution,
            ) = source.parts();
            let plan = match color_filter_operation_limits {
                Some(limits) => RuntimeGraphPreparationPlan::try_derive_with_color_filter_limits(
                    lowered,
                    selected_working_format,
                    capabilities,
                    device,
                    limits,
                    account,
                )?,
                None => RuntimeGraphPreparationPlan::try_derive(
                    lowered,
                    selected_working_format,
                    capabilities,
                    device,
                    account,
                )?,
            };
            resources.preflight_graph_acquisitions(&plan.allocation_preflights, Some(account))?;

            let mut frame_scope = resources.begin_frame(Some(account))?;
            frame_scope.abort_provisional_on_drop();
            if base_execution.is_some()
                || composition_execution.is_some()
                || color_filter_execution.is_some()
                || spatial_filter_execution.is_some()
                || backdrop_execution.is_some()
            {
                frame_scope.discard_on_drop();
            }
            let acquired_resources = acquire_prepared_graph_resources(
                &plan,
                &mut frame_scope,
                device,
                queue,
                capabilities,
                account,
            )?;
            let color_filter_operation_bindings = create_color_filter_operation_bindings(
                frame_scope.generation(),
                &plan,
                device,
                queue,
                account,
            )?;
            let pass_realization = realize_prepared_graph_passes(
                &plan,
                device,
                pass_cache,
                realize_checked_passes,
                account,
            )?;
            let custom_spine_encoding_state = (base_execution.is_some()
                || composition_execution.is_some()
                || color_filter_execution.is_some()
                || spatial_filter_execution.is_some()
                || backdrop_execution.is_some())
            .then_some(CustomSpineEncodingState::Ready);

            Ok(Self {
                plan,
                base_execution,
                composition_execution,
                color_filter_execution,
                spatial_filter_execution,
                backdrop_execution,
                resource_bindings: acquired_resources.runtime_bindings,
                kernel_bindings: acquired_resources.gaussian_kernel_bindings,
                convolution_weight_bindings: acquired_resources.convolution_weight_bindings,
                color_filter_operation_bindings,
                spatial_filter_pass_objects: pass_realization.spatial_filter_objects,
                pass_cache_update: pass_realization.update,
                frame_scope: Some(frame_scope),
                next_pass: 0,
                custom_spine_encoding_state,
                custom_spine_completed_session: None,
                #[cfg(test)]
                acquired_capture_lease_count_raw_fact: 0,
                device,
                queue,
                vello_engine: None,
                resources,
                pass_cache,
                _ready_device: PhantomData,
            })
        })
    }

    pub(crate) const fn base_execution_facts(&self) -> Option<&BaseExecutionFacts> {
        self.base_execution.as_ref()
    }

    #[cfg_attr(
        not(test),
        expect(
            dead_code,
            reason = "graph encoding consumes the typed prepared graph generation for stale-binding checks"
        )
    )]
    pub(crate) const fn generation(&self) -> RuntimeGraphGeneration {
        self.plan.generation
    }

    pub(crate) const fn working_format(&self) -> WorkingFormat {
        self.plan.working_format
    }

    pub(crate) const fn output_format(&self) -> Format {
        self.plan.output_format
    }

    #[cfg_attr(
        not(test),
        expect(
            dead_code,
            reason = "graph encoding consumes the typed prepared root and terminal identities"
        )
    )]
    pub(crate) const fn root_and_final(&self) -> (RuntimeResourceId, RuntimePassId) {
        (self.plan.root_working_image, self.plan.final_present)
    }

    pub(crate) fn output_extent(&self, account: &Account) -> Result<PhysicalSize> {
        self.resource_request(self.plan.root_working_image, account)
            .map(|resource| resource.spatial.device_extent)
    }
}

impl PreparedGraph<'_> {
    pub(super) fn resource_request(
        &self,
        resource: RuntimeResourceId,
        account: &Account,
    ) -> Result<&RuntimeResourceRequest> {
        for request in self.plan.resources.iter() {
            account.work(RenderWork::ResourceProbes, 1)?;
            if request.runtime.id == resource {
                return Ok(&request.runtime);
            }
        }
        Err(preparation_error(
            "the prepared graph resource request is missing",
        ))
    }
}

impl PreparedGraph<'_> {
    #[cfg_attr(
        not(test),
        expect(
            dead_code,
            reason = "graph encoding consumes prepared pass requests through this narrow iterator"
        )
    )]
    pub(crate) fn current_pass(&self) -> Option<super::PreparedPassView<'_>> {
        self.plan
            .passes
            .get(self.next_pass)
            .map(|request| super::PreparedPassView { request })
    }

    pub(super) fn require_current_pass(
        &self,
        pass: RuntimePassId,
        account: &Account,
    ) -> Result<&RuntimePassPreparationRequest> {
        account.work(RenderWork::GraphNodeVisits, 1)?;
        let request = self
            .plan
            .passes
            .get(self.next_pass)
            .ok_or_else(|| preparation_error("prepared graph has no remaining pass"))?;
        if request.runtime.id != pass {
            return Err(preparation_error(
                "prepared pass request is missing, stale, duplicate, or out of order",
            ));
        }
        Ok(request)
    }

    pub(crate) fn texture_binding_for_pass(
        &self,
        pass: RuntimePassId,
        resource: RuntimeResourceId,
        account: &Account,
    ) -> Result<PreparedTextureBinding<'_>> {
        let request = self.require_current_pass(pass, account)?;
        let mut bound = false;
        for read in request.runtime.reads.iter() {
            account.work(RenderWork::GraphEdgeVisits, 1)?;
            if read.resource == resource {
                bound = true;
                break;
            }
        }
        bound |= request.runtime.result == RuntimeResultBinding::Resource(resource);
        if !bound {
            return Err(preparation_error(
                "runtime resource is not bound to the requested prepared pass",
            ));
        }
        let binding = self
            .resource_bindings
            .get(&resource, account)?
            .ok_or_else(|| preparation_error("prepared runtime resource binding is missing"))?;
        let lease = binding.lease.as_ref().ok_or_else(|| {
            preparation_error("prepared runtime resource binding is stale or already released")
        })?;
        let frame_scope = self
            .frame_scope
            .as_ref()
            .ok_or_else(|| preparation_error("prepared frame resource scope is closed"))?;
        let (texture, view) = match &binding.allocation {
            RuntimeAllocationRequest::EffectTexture(_) => frame_scope.effect_texture(lease)?,
            RuntimeAllocationRequest::ResolvedMask(_) => {
                frame_scope.resolved_mask_texture(lease)?
            }
        };
        Ok(PreparedTextureBinding {
            runtime_resource: resource,
            allocation_resource: lease.resource_identity(),
            texture,
            view,
        })
    }

    pub(crate) fn gaussian_kernel_binding_for_pass(
        &self,
        pass: RuntimePassId,
        account: &Account,
    ) -> Result<Option<super::PreparedGaussianKernelBinding<'_>>> {
        let request = self.require_current_pass(pass, account)?;
        let Some(kernel) = request.kernel else {
            return Ok(None);
        };
        let binding = self
            .kernel_bindings
            .get(&kernel, account)?
            .ok_or_else(|| preparation_error("prepared Gaussian kernel binding is missing"))?;
        let lease = binding.lease.as_ref().ok_or_else(|| {
            preparation_error("prepared Gaussian kernel binding is stale or already released")
        })?;
        let frame_scope = self
            .frame_scope
            .as_ref()
            .ok_or_else(|| preparation_error("prepared frame resource scope is closed"))?;
        Ok(Some(super::PreparedGaussianKernelBinding {
            key: kernel,
            allocation_resource: lease.resource_identity(),
            buffer: frame_scope.gaussian_kernel_buffer(lease)?,
        }))
    }
}

#[cfg_attr(
    not(test),
    expect(
        dead_code,
        reason = "graph encoding consumes this immutable view of the current prepared runtime pass"
    )
)]
pub(crate) struct PreparedPassView<'prepared> {
    pub(super) request: &'prepared RuntimePassPreparationRequest,
}

#[cfg_attr(
    not(test),
    expect(
        dead_code,
        reason = "graph encoding consumes these narrow immutable prepared-pass facts"
    )
)]
impl PreparedPassView<'_> {
    pub(crate) const fn id(&self) -> RuntimePassId {
        self.request.runtime.id
    }

    pub(crate) const fn kind(&self) -> &RuntimePassKind {
        &self.request.runtime.kind
    }

    pub(crate) fn dependencies(&self) -> &[RuntimePassId] {
        &self.request.runtime.dependencies
    }

    pub(crate) fn reads(&self) -> &[RuntimeReadBinding] {
        &self.request.runtime.reads
    }

    pub(crate) const fn result(&self) -> RuntimeResultBinding {
        self.request.runtime.result
    }

    pub(crate) const fn spatial_uniform(&self) -> Option<&PassSpatialUniformBytes> {
        self.request.spatial_uniform.as_ref()
    }

    pub(crate) const fn blur_edge_parameters(&self) -> Option<&BlurEdgeParameterBytes> {
        self.request.blur_edge_parameters.as_ref()
    }

    pub(crate) const fn composite_parameters(&self) -> Option<&CompositeParameterBytes> {
        self.request.composite_parameters.as_ref()
    }

    pub(crate) const fn drop_shadow_parameters(&self) -> Option<&DropShadowParameterBytes> {
        self.request.drop_shadow_parameters.as_ref()
    }

    pub(crate) const fn cache_keys(&self) -> Option<&RuntimePassCacheKeys> {
        self.request.cache_keys.as_ref()
    }
}

pub(crate) struct PreparedTextureBinding<'prepared> {
    runtime_resource: RuntimeResourceId,
    allocation_resource: ResourceIdentity,
    texture: &'prepared wgpu::Texture,
    view: &'prepared wgpu::TextureView,
}

impl<'prepared> PreparedTextureBinding<'prepared> {
    pub(crate) const fn runtime_resource(&self) -> RuntimeResourceId {
        self.runtime_resource
    }

    pub(crate) const fn allocation_resource(&self) -> ResourceIdentity {
        self.allocation_resource
    }

    pub(crate) const fn texture(&self) -> &'prepared wgpu::Texture {
        self.texture
    }

    pub(crate) const fn view(&self) -> &'prepared wgpu::TextureView {
        self.view
    }
}

pub(crate) struct PreparedGaussianKernelBinding<'prepared> {
    key: GaussianKernelKey,
    allocation_resource: ResourceIdentity,
    buffer: &'prepared wgpu::Buffer,
}

#[cfg_attr(
    not(test),
    expect(
        dead_code,
        reason = "graph encoding reads these exact typed Gaussian binding facts during blur encoding"
    )
)]
impl<'prepared> PreparedGaussianKernelBinding<'prepared> {
    pub(crate) const fn key(&self) -> GaussianKernelKey {
        self.key
    }

    pub(crate) const fn allocation_resource(&self) -> ResourceIdentity {
        self.allocation_resource
    }

    pub(crate) const fn buffer(&self) -> &'prepared wgpu::Buffer {
        self.buffer
    }
}

#[cfg(test)]
mod reconstructed_key_accounting_tests {
    use super::*;
    use crate::pass::test_support::{lower_filter_proof_fixture, measure_proof_work};

    fn terminal_fixture() -> LoweredGraphPlan {
        lower_filter_proof_fixture(vec![
            crate::FilterList::try_ops(vec![crate::FilterOp::brightness(
                crate::FilterAmount::try_new(1.25).unwrap(),
            )])
            .unwrap(),
        ])
    }

    #[test]
    fn preparation_admits_its_independent_terminal_record_inspection() {
        let lowered = terminal_fixture();
        let setup = Account::for_test();
        let root = lowered
            .resources
            .iter()
            .find(|resource| resource.id == lowered.root_working_image)
            .unwrap();
        let mut resources = PassMap::temporary(&setup, 1).unwrap();
        resources.insert_with(root.id, root, &setup).unwrap();
        for limit in [1, 0] {
            let (result, report) =
                measure_proof_work(RenderWork::GraphNodeVisits, limit, |account| {
                    validate_prepared_graph_root(&lowered, &resources, account)
                });
            let attempt = report.attempt().unwrap();
            assert_eq!(attempt.usage().work(RenderWork::ResourceProbes), 1);
            if limit == 1 {
                result.unwrap();
                assert_eq!(attempt.usage().work(RenderWork::GraphNodeVisits), 1);
            } else {
                assert!(
                    result.is_err(),
                    "preparation must admit the reached terminal record"
                );
                assert_eq!(attempt.status(), crate::RenderAttemptStatus::Refused);
                let refusal = attempt.refusal().unwrap();
                assert_eq!(
                    refusal.quantity,
                    crate::RenderQuantity::Work(RenderWork::GraphNodeVisits)
                );
                assert_eq!((refusal.current, refusal.amount), (0, 1));
            }
        }
    }

    #[test]
    fn preparation_terminal_refusal_precedes_payload_checks_but_empty_has_no_node() {
        let setup = Account::for_test();
        for empty in [false, true] {
            let mut lowered = terminal_fixture();
            lowered
                .passes
                .try_edit_for_test(0, &setup.storage(), |passes| {
                    if empty {
                        passes.clear();
                    } else {
                        passes.last_mut().unwrap().kind = RuntimePassKind::CopyBackdrop;
                    }
                })
                .unwrap();
            let root = lowered
                .resources
                .iter()
                .find(|resource| resource.id == lowered.root_working_image)
                .unwrap();
            let mut resources = PassMap::temporary(&setup, 1).unwrap();
            resources.insert_with(root.id, root, &setup).unwrap();
            for limit in [0, 1] {
                let (result, report) =
                    measure_proof_work(RenderWork::GraphNodeVisits, limit, |account| {
                        validate_prepared_graph_root(&lowered, &resources, account)
                    });
                let error = result.unwrap_err();
                let attempt = report.attempt().unwrap();
                if !empty && limit == 0 {
                    assert_eq!(attempt.status(), crate::RenderAttemptStatus::Refused);
                    let refusal = attempt.refusal().unwrap();
                    assert_eq!((refusal.current, refusal.amount), (0, 1));
                } else {
                    assert!(error.accounting_diagnostic().is_none());
                    assert!(attempt.refusal().is_none());
                    assert_eq!(
                        attempt.usage().work(RenderWork::GraphNodeVisits),
                        u64::from(!empty)
                    );
                }
            }
        }
    }

    struct AnalysisPrefix<'plan> {
        position: usize,
        resources: PassMap<RuntimeResourceId, &'plan RuntimeResourceRequest>,
        positions: PassMap<RuntimePassId, usize>,
        facts: RuntimePassAnalysis,
    }

    impl<'plan> AnalysisPrefix<'plan> {
        fn new(lowered: &'plan LoweredGraphPlan, account: &Account) -> Result<Self> {
            let mut resources = PassMap::temporary(account, lowered.resources.len())?;
            for resource in lowered.resources.iter() {
                resources.insert_with(resource.id, resource, account)?;
            }
            let mut positions = PassMap::temporary(account, lowered.passes.len())?;
            for (position, pass) in lowered.passes.iter().enumerate() {
                positions.insert_with(pass.id, position, account)?;
            }
            let mut facts = RuntimePassAnalysis {
                actual_reads: PassMap::temporary(account, lowered.resources.len())?,
                actual_last_reads: PassMap::temporary(account, lowered.resources.len())?,
                release_passes: PassMap::temporary(account, lowered.resources.len())?,
                produced_results: PassMap::temporary(account, lowered.resources.len())?,
                kernel_by_pass: PassMap::temporary(account, 0)?,
                kernels: PassMap::temporary(account, 0)?,
            };
            let position = lowered
                .passes
                .iter()
                .position(|pass| pass.cache_keys.is_some())
                .unwrap();
            for (position, pass) in lowered.passes.iter().take(position).enumerate() {
                drop(analyze_runtime_pass(
                    pass, position, lowered, &resources, &positions, &mut facts, account,
                )?);
            }
            Ok(Self {
                position,
                resources,
                positions,
                facts,
            })
        }

        fn analyze_next(&mut self, lowered: &LoweredGraphPlan, account: &Account) -> Result<usize> {
            analyze_runtime_pass(
                &lowered.passes[self.position],
                self.position,
                lowered,
                &self.resources,
                &self.positions,
                &mut self.facts,
                account,
            )
            .map(|reads| reads.len())
        }
    }

    #[test]
    fn preparation_reconstructed_cache_keys_refuse_before_the_first_binding_comparison() {
        let filters = crate::FilterList::try_ops(vec![crate::FilterOp::brightness(
            crate::FilterAmount::try_new(1.25).unwrap(),
        )])
        .unwrap();
        let lowered = lower_filter_proof_fixture(vec![filters]);
        let setup = Account::for_test();
        let mut control = AnalysisPrefix::new(&lowered, &setup).unwrap();
        let first_keyed = &lowered.passes[control.position];
        assert!(matches!(
            first_keyed.kind,
            RuntimePassKind::CanonicalizeCapture
        ));
        assert_eq!(first_keyed.reads.len(), 1);
        assert_eq!(first_keyed.cache_keys.as_ref().unwrap().samplers().len(), 1);
        assert_eq!(control.analyze_next(&lowered, &setup).unwrap(), 1);

        let mut refused = AnalysisPrefix::new(&lowered, &setup).unwrap();
        let (result, report) = measure_proof_work(RenderWork::ShaderBindingVisits, 0, |account| {
            refused.analyze_next(&lowered, account)
        });
        assert!(
            result.is_err(),
            "preparation must admit its own reconstructed binding comparison"
        );
        let attempt = report.attempt().unwrap();
        assert_eq!(attempt.status(), crate::RenderAttemptStatus::Refused);
        let refusal = attempt.refusal().unwrap();
        assert_eq!(
            refusal.quantity,
            crate::RenderQuantity::Work(RenderWork::ShaderBindingVisits)
        );
        assert_eq!((refusal.current, refusal.amount), (0, 1));
    }

    #[test]
    fn preparation_duplicate_runtime_read_follows_its_edge_admission() {
        use crate::{
            ErrorCode, RenderAccountingError, RenderAttemptStatus, RenderQuantity,
            RenderRefusalReason, RenderResource,
        };

        let lowered = terminal_fixture();
        let setup = Account::for_test();
        for limit in [1, 2] {
            let mut prefix = AnalysisPrefix::new(&lowered, &setup).unwrap();
            let mut pass = lowered.passes[prefix.position].clone();
            assert!(matches!(pass.kind, RuntimePassKind::CanonicalizeCapture));
            assert_eq!(pass.reads.len(), 1);
            assert!(pass.cache_keys.is_some());
            pass.dependencies
                .try_edit_for_test(0, &setup.storage(), Vec::clear)
                .unwrap();
            pass.reads
                .try_edit_for_test(1, &setup.storage(), |reads| reads.push(reads[0]))
                .unwrap();
            let (result, report) =
                measure_proof_work(RenderWork::GraphEdgeVisits, limit, |account| {
                    analyze_runtime_pass(
                        &pass,
                        prefix.position,
                        &lowered,
                        &prefix.resources,
                        &prefix.positions,
                        &mut prefix.facts,
                        account,
                    )
                    .map(drop)
                });
            let error = result.unwrap_err();
            let attempt = report.attempt().unwrap();
            if limit == 1 {
                let refusal = attempt.refusal().unwrap();
                assert_eq!(error.code(), ErrorCode::Accounting);
                assert_eq!(
                    error.accounting_diagnostic(),
                    Some(RenderAccountingError::Refused(refusal))
                );
                assert_eq!(report.status(), RenderAttemptStatus::Refused);
                assert_eq!(refusal.phase(), RenderPhase::Attempt);
                assert_eq!(
                    refusal.quantity(),
                    RenderQuantity::Work(RenderWork::GraphEdgeVisits)
                );
                assert_eq!((refusal.current(), refusal.amount()), (1, 1));
                assert_eq!(refusal.reason(), RenderRefusalReason::Limit { limit: 1 });
            } else {
                assert_eq!(error.code(), ErrorCode::RenderFailed);
                assert_eq!(
                    error.message(),
                    "prepared pass contains a duplicate runtime read binding"
                );
                assert_eq!(error.accounting_diagnostic(), None);
                assert_eq!(
                    report.status(),
                    RenderAttemptStatus::Failed(ErrorCode::RenderFailed)
                );
                assert_eq!(attempt.refusal(), None);
            }
            let usage = attempt.usage();
            assert_eq!(usage.work(RenderWork::GraphEdgeVisits), limit);
            assert_eq!(usage.work(RenderWork::ShaderBindingVisits), 0);
            assert_eq!(usage.work(RenderWork::KernelSampleVisits), 0);
            assert_eq!(
                report
                    .renderer_cpu()
                    .unwrap()
                    .usage()
                    .live(RenderResource::ScratchCpuBytes),
                0
            );
        }
    }

    #[test]
    fn preparation_read_set_backing_overlaps_gaussian_weights() {
        use crate::accounting::{
            FrameFixture, RenderAccountingError, RenderAccountingOptions, RenderAttemptStatus,
            RenderCpuScope, RenderLimits, RenderQuantity, RenderRefusalReason, RenderResource,
            RendererCpu,
        };

        fn reserved_bytes<T>(setup: &Account, count: usize) -> u64 {
            let backing = setup.temporary::<T>(count).unwrap();
            assert_eq!(backing.capacity(), count);
            (backing.capacity() * std::mem::size_of::<T>()) as u64
        }

        let options = RenderAccountingOptions::new();
        let cpu = RendererCpu::new(options).unwrap();
        let mut backend =
            crate::backend::Backend::new(crate::ResourceCacheBudget::DISABLED, cpu, options)
                .unwrap();
        let identity = pollster::block_on(backend.select_device(None))
            .unwrap()
            .unwrap();
        let (device, _) = backend
            .device_queue(identity, crate::RuntimeOperation::EffectRendering)
            .unwrap();
        let filters = crate::FilterList::try_ops(vec![crate::FilterOp::blur(
            crate::FilterBlur::try_new(4.0).unwrap(),
        )])
        .unwrap();
        let lowered = lower_filter_proof_fixture(vec![filters]);
        let setup = Account::for_test();
        let mut resources = PassMap::temporary(&setup, lowered.resources.len()).unwrap();
        for resource in &lowered.resources {
            resources
                .insert_with(resource.id, resource, &setup)
                .unwrap();
        }
        let mut blurs = lowered
            .passes
            .iter()
            .filter_map(|pass| runtime_blur_for_kernel(&pass.kind).map(|blur| (pass, blur)));
        for _ in 0..2 {
            let (pass, blur) = blurs.next().unwrap();
            assert_eq!(pass.reads.len(), 1);
            assert_eq!(blur.spatial.result.raster_scale, 1.0);
            assert_eq!(blur.support_radius, 10);
        }
        assert!(blurs.next().is_none());

        // The position map and six fact maps remain live at either blur.
        let resource_count = lowered.resources.len();
        let maps = reserved_bytes::<(RuntimePassId, usize)>(&setup, lowered.passes.len())
            + reserved_bytes::<(RuntimeResourceId, u32)>(&setup, resource_count)
            + 3 * reserved_bytes::<(RuntimeResourceId, RuntimePassId)>(&setup, resource_count)
            + reserved_bytes::<(RuntimePassId, GaussianKernelKey)>(&setup, 2)
            + reserved_bytes::<(GaussianKernelKey, RuntimeKernelPreparationRequest)>(&setup, 2);
        let reads = reserved_bytes::<(RuntimeResourceId, ())>(&setup, 1);
        let weights = reserved_bytes::<f64>(&setup, 11);
        let exact = maps + reads + weights;

        for limit in [exact, exact - 1] {
            let options = RenderAccountingOptions::new()
                .with_limits(RenderLimits::new().with_scratch_bytes(limit));
            let cpu = RendererCpu::new(options).unwrap();
            let (result, _, report, fault) = FrameFixture::new(&cpu, options)
                .run_sync(|attempt| {
                    analyze_runtime_passes(&lowered, &resources, device, attempt.account())
                        .map(drop)
                })
                .unwrap()
                .into_parts();
            assert_eq!(fault, None);
            let usage = report.renderer_cpu().unwrap().usage();
            assert_eq!(usage.opening_live(RenderResource::ScratchCpuBytes), 0);
            assert_eq!(usage.live(RenderResource::ScratchCpuBytes), 0);
            let attempt = report.attempt().unwrap();
            if limit == exact {
                result.unwrap();
                assert_eq!(report.status(), RenderAttemptStatus::Succeeded);
                assert_eq!(usage.peak(RenderResource::ScratchCpuBytes), exact);
                // Each build computes 11 weights, sums 10, normalizes 11 and
                // emits 11 samples. Reuse compares all 11 rebuilt samples.
                assert_eq!(attempt.usage().work(RenderWork::KernelSampleVisits), 97);
            } else {
                let expected = RenderAccountingError::CpuRefused {
                    scope: RenderCpuScope::Root,
                    quantity: RenderQuantity::Resource(RenderResource::ScratchCpuBytes),
                    current: maps + reads,
                    amount: weights,
                    reason: RenderRefusalReason::Limit { limit },
                };
                assert_eq!(result.unwrap_err().accounting_diagnostic(), Some(expected));
                assert_eq!(report.accounting_diagnostic(), Some(expected));
                assert_eq!(report.status(), RenderAttemptStatus::Refused);
                assert_eq!(attempt.usage().work(RenderWork::KernelSampleVisits), 0);
            }
        }
    }

    #[test]
    fn preparation_kernel_reuse_admits_each_reconstructed_sample_comparison() {
        pollster::block_on(async {
            let options = crate::RenderAccountingOptions::new();
            let cpu = crate::accounting::RendererCpu::new(options).unwrap();
            let mut backend =
                crate::backend::Backend::new(crate::ResourceCacheBudget::DISABLED, cpu, options)
                    .unwrap();
            let identity = backend.select_device(None).await.unwrap().unwrap();
            let (device, _) = backend
                .device_queue(identity, crate::RuntimeOperation::EffectRendering)
                .unwrap();
            let filters = crate::FilterList::try_ops(vec![crate::FilterOp::blur(
                crate::FilterBlur::try_new(0.25).unwrap(),
            )])
            .unwrap();
            let lowered = lower_filter_proof_fixture(vec![filters]);
            let horizontal = lowered
                .passes
                .iter()
                .find(|pass| matches!(pass.kind, RuntimePassKind::BlurHorizontal(Some(_))))
                .unwrap();
            let vertical = lowered
                .passes
                .iter()
                .find(|pass| matches!(pass.kind, RuntimePassKind::BlurVertical(Some(_))))
                .unwrap();
            let blur = runtime_blur_for_kernel(&horizontal.kind).unwrap();
            assert_eq!(blur.standard_deviation, 0.25);
            assert_eq!(blur.spatial.result.raster_scale, 1.0);
            assert_eq!(blur.support_radius, 1);
            assert_eq!(
                blur.kernel,
                runtime_blur_for_kernel(&vertical.kind).unwrap().kernel
            );
            for limit in [11, 10] {
                let setup = Account::for_test();
                let mut prefix = AnalysisPrefix::new(&lowered, &setup).unwrap();
                prefix.facts.kernel_by_pass = PassMap::temporary(&setup, 2).unwrap();
                prefix.facts.kernels = PassMap::temporary(&setup, 1).unwrap();
                analyze_runtime_kernel(horizontal, device, &mut prefix.facts, &setup).unwrap();
                let stored = prefix
                    .facts
                    .kernels
                    .get(&blur.kernel, &setup)
                    .unwrap()
                    .unwrap();
                assert_eq!(stored.plan.byte_len(), 24);
                assert_eq!(stored.last_use, horizontal.id);
                let (result, report) =
                    measure_proof_work(RenderWork::KernelSampleVisits, limit, |account| {
                        analyze_runtime_kernel(vertical, device, &mut prefix.facts, account)
                    });
                let attempt = report.attempt().unwrap();
                if limit == 11 {
                    result.unwrap();
                    // Radius1: calculate2 weights, sum1, normalize2, serialize3
                    // samples, then compare the3 independently rebuilt samples.
                    assert_eq!(attempt.usage().work(RenderWork::KernelSampleVisits), 11);
                    assert_eq!(
                        prefix
                            .facts
                            .kernels
                            .get(&blur.kernel, &setup)
                            .unwrap()
                            .unwrap()
                            .last_use,
                        vertical.id
                    );
                } else {
                    assert!(
                        result.is_err(),
                        "the final serialized sample comparison must refuse"
                    );
                    let refusal = attempt.refusal().unwrap();
                    assert_eq!(
                        refusal.quantity,
                        crate::RenderQuantity::Work(RenderWork::KernelSampleVisits)
                    );
                    assert_eq!((refusal.current, refusal.amount), (10, 1));
                    assert_eq!(
                        prefix
                            .facts
                            .kernels
                            .get(&blur.kernel, &setup)
                            .unwrap()
                            .unwrap()
                            .last_use,
                        horizontal.id
                    );
                }
            }
        });
    }
}
