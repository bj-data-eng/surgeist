use crate::accounting::{
    Account, AccountedVec, GenerationClass, GpuObjectKind, OwnedGpu, RenderPhase, RenderWork,
    SharedValue,
};

use std::sync::Arc;

#[path = "encode/convolution.rs"]
mod convolution;
#[path = "encode/offset.rs"]
mod offset;

#[cfg(test)]
use crate::BlendMode;

use super::super::{
    Color, Format, PhysicalSize, Result, Transform,
    encode::{encode_vello_clip_coverage_scene, encode_vello_scene_with_initial_transform},
    renderer::Antialiasing,
    resource::{FrameCleanup, FrameResourceScope, GaussianKernelKey, ResourceManager},
    shader::{
        BlurEdgeParameterBytes, ColorFilterOperationBufferLimits, ColorFilterOperationBytes,
        CompositeParameterBytes, ConvolutionParameterBytes, DevicePassCache,
        DropShadowParameterBytes, OffsetParameterBytes, PassCacheCommitPreparation,
        PassSpatialUniformBytes, ProvisionalColorFilterPassObjects,
        ProvisionalCompositePassObjects, ProvisionalCorePassObjects,
        ProvisionalDevicePassCacheUpdate, ReadyPassCacheUpdate, ShaderCompositePathKey,
    },
    vello_engine::{
        ActiveVelloEncodingScope, EncodedVelloCaptureProof, PendingVelloResourceCommit,
        RasterParameters, TransactionEncodingState, TransactionTargetIntent, VelloEngineState,
        VelloResourceLeaseAggregate, scene::VelloScene,
    },
};
use super::{
    PassMap,
    close::{ExecutableFilterStepFacts, ExecutableVelloCaptureFacts, preparation_error},
    model::{
        RuntimeBlurInput, RuntimeColorFilter, RuntimeComposite, RuntimeCompositeKind,
        RuntimeInitialization, RuntimeLayerCompositeParameters, RuntimePassCacheKeys,
        RuntimePassId, RuntimePassKind, RuntimeReadBinding, RuntimeReadRole, RuntimeResourceFormat,
        RuntimeResourceId, RuntimeResourceRole, RuntimeResultBinding, RuntimeSamplingEdge,
        RuntimeSamplingFilter, RuntimeSpatialDescriptor, RuntimeVelloCapture,
    },
    parameters::backdrop_blur_edge_uniform_bytes,
    prepare::{
        PreparedColorFilterOperationBinding, PreparedConvolutionWeightsBinding, PreparedGraph,
        PreparedKernelBinding, PreparedOffsetParameterBinding, PreparedResourceBinding,
        PreparedSpatialFilterPassObjects, PreparedTextureBinding, RuntimePassPreparationRequest,
        VELLO_CAPTURE_TEXTURE_USAGES,
    },
};

pub(super) fn filter_step_passes<'a>(
    steps: &'a [ExecutableFilterStepFacts],
    account: &'a Account,
) -> impl Iterator<Item = Result<RuntimePassId>> + 'a {
    steps.iter().flat_map(move |step| {
        let count = match step {
            ExecutableFilterStepFacts::Color(_) => 1,
            ExecutableFilterStepFacts::Offset { rebase, .. } => 1 + usize::from(rebase.is_some()),
            ExecutableFilterStepFacts::Blur { .. } => 2,
            ExecutableFilterStepFacts::DropShadow { .. } => 4,
            ExecutableFilterStepFacts::Convolution { passes, .. } => passes.len().max(1),
        };
        (0..count).map(move |index| {
            if index == 0 {
                account.work(RenderWork::GraphNodeVisits, 1)?;
            }
            let pass = match step {
                ExecutableFilterStepFacts::Color(pass) => Some(*pass),
                ExecutableFilterStepFacts::Offset { rebase, sample, .. } => {
                    if index == 0 {
                        Some(rebase.unwrap_or(*sample))
                    } else {
                        Some(*sample)
                    }
                }
                ExecutableFilterStepFacts::Blur {
                    horizontal,
                    vertical,
                } => [*horizontal, *vertical].get(index).copied(),
                ExecutableFilterStepFacts::DropShadow {
                    horizontal,
                    vertical,
                    colorize,
                    merge,
                } => [*horizontal, *vertical, *colorize, *merge]
                    .get(index)
                    .copied(),
                ExecutableFilterStepFacts::Convolution { passes, .. } => passes.get(index).copied(),
            };
            pass.ok_or_else(|| {
                preparation_error("filter receipt contains an empty convolution chain")
            })
        })
    })
}

pub(super) fn vello_capture_raster_parameters(
    target_extent: PhysicalSize,
    antialiasing: Antialiasing,
) -> Result<RasterParameters> {
    RasterParameters::try_new(target_extent, peniko::Color::TRANSPARENT, antialiasing)
}

pub(crate) struct VelloCaptureEncodingHandoff<'prepared> {
    pass: RuntimePassId,
    target: RuntimeResourceId,
    work: &'prepared RuntimeVelloCapture,
    initial_transform: Transform,
    antialiasing: Antialiasing,
    target_extent: PhysicalSize,
    raster_scale: f64,
    texture: &'prepared wgpu::Texture,
    view: &'prepared wgpu::TextureView,
    session: Arc<()>,
}

struct VelloCaptureCompletionSeal;

/// Opaque proof that one exact capture finished inside the active graph encoding
/// session. Only a successfully encoded internal Vello capture can seal it.
#[must_use = "a capture completion receipt must return to the owning custom-spine scheduler"]
pub(crate) struct VelloCaptureCompletionReceipt {
    pass: RuntimePassId,
    target: RuntimeResourceId,
    session: Arc<()>,
    _seal: VelloCaptureCompletionSeal,
}

impl VelloCaptureEncodingHandoff<'_> {
    pub(crate) const fn target(&self) -> RuntimeResourceId {
        self.target
    }

    const fn work(&self) -> &RuntimeVelloCapture {
        self.work
    }

    fn has_bounded_work(&self) -> bool {
        match self.work() {
            RuntimeVelloCapture::Span(span) => !span.commands.commands.is_empty(),
            RuntimeVelloCapture::ClipCoverage(coverage) => !coverage.elements.is_empty(),
        }
    }

    pub(crate) const fn initial_transform(&self) -> Transform {
        self.initial_transform
    }

    pub(crate) const fn antialiasing(&self) -> Antialiasing {
        self.antialiasing
    }

    pub(crate) const fn target_extent(&self) -> PhysicalSize {
        self.target_extent
    }

    pub(crate) const fn raster_scale(&self) -> f64 {
        self.raster_scale
    }

    pub(crate) const fn texture(&self) -> &wgpu::Texture {
        self.texture
    }

    pub(crate) const fn view(&self) -> &wgpu::TextureView {
        self.view
    }

    fn complete_after_encoded_capture(
        self,
        proof: EncodedVelloCaptureProof,
    ) -> Result<VelloCaptureCompletionReceipt> {
        if !proof.proves_capture_contract(
            self.target_extent,
            wgpu::TextureFormat::Rgba8Unorm,
            VELLO_CAPTURE_TEXTURE_USAGES,
            self.antialiasing,
        ) {
            return Err(preparation_error(
                "encoded graph Vello capture proof changed its exact raster target contract",
            ));
        }
        Ok(VelloCaptureCompletionReceipt {
            pass: self.pass,
            target: self.target,
            session: self.session,
            _seal: VelloCaptureCompletionSeal,
        })
    }
}

pub(crate) struct GraphExternalOutputView<'output> {
    view: &'output wgpu::TextureView,
    format: Format,
    extent: PhysicalSize,
}

impl<'output> GraphExternalOutputView<'output> {
    pub(crate) fn try_new(
        view: &'output wgpu::TextureView,
        format: Format,
        extent: PhysicalSize,
    ) -> Result<Self> {
        if extent.width() == 0 || extent.height() == 0 {
            return Err(preparation_error(
                "graph external output view must have a positive exact extent",
            ));
        }
        Ok(Self {
            view,
            format,
            extent,
        })
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(super) enum GraphScheduledEncodingKind {
    ClearRoot,
    VelloCapture,
    CanonicalizeCapture,
    CopyBackdrop,
    ColorFilter,
    Convolution,
    Offset,
    BlurHorizontalRgba,
    BlurVerticalRgba,
    BlurHorizontalSourceAlpha,
    BlurVerticalSourceAlpha,
    DropShadowColorize,
    DropShadowMerge,
    SpanSourceOver,
    LayerComposite,
    Present,
}

#[cfg(test)]
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum ScheduledFilterRawFact {
    Color,
    Convolution,
    Offset,
    BlurHorizontalRgba,
    BlurVerticalRgba,
    BlurHorizontalSourceAlpha,
    BlurVerticalSourceAlpha,
    DropShadowColorize,
    DropShadowMerge,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(super) enum CustomSpineEncodingState {
    Ready,
    Encoding,
    Complete,
    AbortOnly,
}

pub(crate) struct CustomSpineEncodingSummary {
    pub(crate) offset_pass_count: usize,
    pub(crate) offset_parameters_and_resources_are_exact: bool,
    pub(crate) convolution_pass_count: usize,
    pub(crate) convolution_parameters_and_resources_are_exact: bool,
    pub(crate) encodes_custom_passes_in_order: bool,
    pub(crate) clears_full_root_once: bool,
    pub(crate) uses_exact_prepared_spatial_mapping: bool,
    pub(crate) presents_to_exact_external_output: bool,
    pub(crate) capture_handoffs_match_expected_count: bool,
    pub(crate) validates_checked_capture_completion: bool,
    pub(crate) completes_custom_passes_after_encoding: bool,
    pub(crate) parent_and_result_are_distinct: bool,
    pub(crate) copies_full_parent_before_bounded_source_render: bool,
    pub(crate) samples_only_source_with_fixed_premultiplied_blend: bool,
    pub(crate) preserves_signed_source_origin: bool,
    pub(crate) keeps_cache_update_provisional: bool,
    pub(crate) layer_composite_count: usize,
    pub(crate) normal_composite_count: usize,
    pub(crate) destination_composite_count: usize,
    pub(crate) normal_composites_use_fixed_premultiplied_blend: bool,
    pub(crate) normal_composites_omit_parent_sample: bool,
    pub(crate) destination_composites_copy_full_parent: bool,
    pub(crate) destination_composites_avoid_read_write_alias: bool,
    pub(crate) layer_composites_bind_exact_resources_and_parameters: bool,
    pub(crate) layer_composites_preserve_signed_mapping: bool,
    pub(crate) color_filter_count: usize,
    pub(crate) color_filters_preserve_authored_order: bool,
    pub(crate) color_filters_bind_exact_source_spatial_and_operations: bool,
    pub(crate) color_filter_sources_and_results_are_distinct: bool,
    pub(crate) color_filters_use_validated_viewport_and_scissor: bool,
    pub(crate) color_filters_preserve_signed_texel_mapping: bool,
    pub(crate) color_filter_operation_buffers_released: bool,
    pub(crate) copy_backdrop_count: usize,
    pub(crate) copy_backdrop_binds_exact_prepared_resources: bool,
    pub(crate) copy_backdrop_source_and_result_are_distinct: bool,
    pub(crate) copy_backdrop_uses_validated_viewport_and_scissor: bool,
    pub(crate) copy_backdrop_preserves_signed_mapping: bool,
    backdrop_filter_order_is_exact: bool,
    pub(crate) backdrop_group_order_is_exact: bool,
    pub(crate) backdrop_group_resources_are_distinct: bool,
    #[cfg(test)]
    pub(crate) backdrop_later_sibling_transition_is_exact: bool,
    pub(crate) blur_pass_count: usize,
    pub(crate) drop_shadow_colorize_count: usize,
    pub(crate) drop_shadow_merge_count: usize,
    pub(crate) spatial_filter_binds_exact_prepared_resources: bool,
    pub(crate) spatial_filter_uses_signed_viewport_and_scissor: bool,
    pub(crate) blur_sources_intermediates_and_results_are_distinct: bool,
    pub(crate) spatial_filter_kernels_release_at_validated_last_use: bool,
    pub(crate) spatial_filter_textures_release_at_validated_last_use: bool,
    pub(crate) drop_shadow_reads_original_source_twice: bool,
    pub(crate) original_source_releases_after_merge: bool,
    pub(crate) advances_every_pass_once: bool,
    #[cfg(test)]
    pub(crate) spatial_filter_pass_order: Vec<ScheduledFilterRawFact>,
    #[cfg(test)]
    pub(crate) capture_count: usize,
    #[cfg(test)]
    pub(crate) captures_share_one_command_encoder: bool,
    #[cfg(test)]
    pub(crate) captures_share_one_active_vello_scope: bool,
    #[cfg(test)]
    pub(crate) graph_work_shares_one_command_encoder: bool,
    #[cfg(test)]
    pub(crate) capture_observations: Vec<EncodedCaptureRawFact>,
}

struct CustomSpineEncodingProgress {
    offset_pass_count: usize,
    offset_parameters_and_resources_are_exact: bool,
    convolution_pass_count: usize,
    convolution_parameters_and_resources_are_exact: bool,
    scheduled: AccountedVec<GraphScheduledEncodingKind>,
    expected_capture_count: usize,
    expected_pass_count: usize,
    capture_count: usize,
    validated_capture_receipts: usize,
    bounded_capture_handoffs: bool,
    custom_encoded: usize,
    custom_completed: usize,
    completed_pass_count: usize,
    root_clear_count: usize,
    clears_full_root: bool,
    exact_spatial: bool,
    exact_external_output: bool,
    source_over_count: usize,
    layer_composite_count: usize,
    normal_composite_count: usize,
    destination_composite_count: usize,
    parent_and_result_are_distinct: bool,
    full_copy_before_bounded_render: bool,
    samples_source_with_fixed_blend: bool,
    preserves_signed_origin: bool,
    normal_fixed_blend: bool,
    normal_omits_parent_sample: bool,
    destination_copies_full_parent: bool,
    destination_avoids_alias: bool,
    layer_bindings_are_exact: bool,
    layer_signed_mapping_is_exact: bool,
    color_filter_count: usize,
    color_filters_preserve_authored_order: bool,
    color_filter_bindings_are_exact: bool,
    color_filter_sources_and_results_are_distinct: bool,
    color_filter_regions_are_validated: bool,
    color_filter_signed_texel_mapping_is_exact: bool,
    color_filter_operation_buffers_released: bool,
    copy_backdrop_count: usize,
    copy_backdrop_bindings_are_exact: bool,
    copy_backdrop_allocations_are_distinct: bool,
    copy_backdrop_regions_are_validated: bool,
    copy_backdrop_signed_mapping_is_exact: bool,
    blur_pass_count: usize,
    drop_shadow_colorize_count: usize,
    drop_shadow_merge_count: usize,
    spatial_filter_bindings_are_exact: bool,
    spatial_filter_regions_are_exact: bool,
    blur_allocations_are_distinct: bool,
    spatial_filter_kernels_released: bool,
    spatial_filter_textures_released: bool,
    shadow_source_read_twice: bool,
    shadow_source_released_after_merge: bool,
    #[cfg(test)]
    capture_observations: Vec<EncodedCaptureRawFact>,
    #[cfg(test)]
    composite_encoder_identities: Vec<usize>,
}

impl CustomSpineEncodingProgress {
    fn new(
        expected_pass_count: usize,
        expected_capture_count: usize,
        account: &Account,
    ) -> Result<Self> {
        Ok(Self {
            offset_pass_count: 0,
            offset_parameters_and_resources_are_exact: true,
            convolution_pass_count: 0,
            convolution_parameters_and_resources_are_exact: true,
            scheduled: account.temporary(expected_pass_count)?,
            expected_capture_count,
            expected_pass_count,
            capture_count: 0,
            validated_capture_receipts: 0,
            bounded_capture_handoffs: true,
            custom_encoded: 0,
            custom_completed: 0,
            completed_pass_count: 0,
            root_clear_count: 0,
            clears_full_root: true,
            exact_spatial: true,
            exact_external_output: false,
            source_over_count: 0,
            layer_composite_count: 0,
            normal_composite_count: 0,
            destination_composite_count: 0,
            parent_and_result_are_distinct: true,
            full_copy_before_bounded_render: true,
            samples_source_with_fixed_blend: true,
            preserves_signed_origin: true,
            normal_fixed_blend: true,
            normal_omits_parent_sample: true,
            destination_copies_full_parent: true,
            destination_avoids_alias: true,
            layer_bindings_are_exact: true,
            layer_signed_mapping_is_exact: true,
            color_filter_count: 0,
            color_filters_preserve_authored_order: true,
            color_filter_bindings_are_exact: true,
            color_filter_sources_and_results_are_distinct: true,
            color_filter_regions_are_validated: true,
            color_filter_signed_texel_mapping_is_exact: true,
            color_filter_operation_buffers_released: true,
            copy_backdrop_count: 0,
            copy_backdrop_bindings_are_exact: true,
            copy_backdrop_allocations_are_distinct: true,
            copy_backdrop_regions_are_validated: true,
            copy_backdrop_signed_mapping_is_exact: true,
            blur_pass_count: 0,
            drop_shadow_colorize_count: 0,
            drop_shadow_merge_count: 0,
            spatial_filter_bindings_are_exact: true,
            spatial_filter_regions_are_exact: true,
            blur_allocations_are_distinct: true,
            spatial_filter_kernels_released: true,
            spatial_filter_textures_released: true,
            shadow_source_read_twice: true,
            shadow_source_released_after_merge: true,
            #[cfg(test)]
            capture_observations: Vec::with_capacity(expected_capture_count),
            #[cfg(test)]
            composite_encoder_identities: Vec::new(),
        })
    }

    fn record_custom_completion(&mut self, kind: GraphScheduledEncodingKind) -> Result<()> {
        self.scheduled.append_slot()?.write(kind);
        self.custom_encoded = self.custom_encoded.saturating_add(1);
        self.custom_completed = self.custom_completed.saturating_add(1);
        self.completed_pass_count = self.completed_pass_count.saturating_add(1);
        Ok(())
    }

    fn record_capture_completion(&mut self) -> Result<()> {
        self.scheduled
            .append_slot()?
            .write(GraphScheduledEncodingKind::VelloCapture);
        self.capture_count = self.capture_count.saturating_add(1);
        self.validated_capture_receipts = self.validated_capture_receipts.saturating_add(1);
        self.completed_pass_count = self.completed_pass_count.saturating_add(1);
        Ok(())
    }

    fn finish(
        self,
        prepared: &PreparedGraph<'_>,
        account: &Account,
    ) -> Result<CustomSpineEncodingSummary> {
        let total_composites = self
            .source_over_count
            .saturating_add(self.layer_composite_count);
        let backdrop = backdrop_execution_receipt(prepared, account)?;
        Ok(CustomSpineEncodingSummary {
            offset_pass_count: self.offset_pass_count,
            offset_parameters_and_resources_are_exact: self
                .offset_parameters_and_resources_are_exact,
            convolution_pass_count: self.convolution_pass_count,
            convolution_parameters_and_resources_are_exact: self
                .convolution_parameters_and_resources_are_exact,
            encodes_custom_passes_in_order: graph_scheduled_encoding_order_is_exact(
                &self.scheduled,
                &prepared.plan.passes,
                account,
            )?,
            clears_full_root_once: self.root_clear_count == 1 && self.clears_full_root,
            uses_exact_prepared_spatial_mapping: self.exact_spatial,
            presents_to_exact_external_output: self.exact_external_output,
            capture_handoffs_match_expected_count: self.capture_count
                == self.expected_capture_count
                && self.bounded_capture_handoffs,
            validates_checked_capture_completion: self.validated_capture_receipts
                == self.expected_capture_count,
            completes_custom_passes_after_encoding: self.custom_encoded > 0
                && self.custom_completed == self.custom_encoded,
            parent_and_result_are_distinct: total_composites > 0
                && self.parent_and_result_are_distinct,
            copies_full_parent_before_bounded_source_render: total_composites > 0
                && self.full_copy_before_bounded_render,
            samples_only_source_with_fixed_premultiplied_blend: (self.source_over_count > 0
                || self.normal_composite_count > 0
                || self.destination_composite_count > 0)
                && self.samples_source_with_fixed_blend,
            preserves_signed_source_origin: total_composites > 0 && self.preserves_signed_origin,
            keeps_cache_update_provisional: prepared.pass_cache_update.is_some(),
            layer_composite_count: self.layer_composite_count,
            normal_composite_count: self.normal_composite_count,
            destination_composite_count: self.destination_composite_count,
            normal_composites_use_fixed_premultiplied_blend: self.normal_fixed_blend,
            normal_composites_omit_parent_sample: self.normal_omits_parent_sample,
            destination_composites_copy_full_parent: self.destination_copies_full_parent,
            destination_composites_avoid_read_write_alias: self.destination_avoids_alias,
            layer_composites_bind_exact_resources_and_parameters: self.layer_bindings_are_exact,
            layer_composites_preserve_signed_mapping: self.layer_signed_mapping_is_exact,
            color_filter_count: self.color_filter_count,
            color_filters_preserve_authored_order: self.color_filter_count > 0
                && self.color_filters_preserve_authored_order,
            color_filters_bind_exact_source_spatial_and_operations: self.color_filter_count > 0
                && self.color_filter_bindings_are_exact,
            color_filter_sources_and_results_are_distinct: self.color_filter_count > 0
                && self.color_filter_sources_and_results_are_distinct,
            color_filters_use_validated_viewport_and_scissor: self.color_filter_count > 0
                && self.color_filter_regions_are_validated,
            color_filters_preserve_signed_texel_mapping: self.color_filter_count > 0
                && self.color_filter_signed_texel_mapping_is_exact,
            color_filter_operation_buffers_released: self.color_filter_count > 0
                && self.color_filter_operation_buffers_released,
            copy_backdrop_count: self.copy_backdrop_count,
            copy_backdrop_binds_exact_prepared_resources: self.copy_backdrop_count > 0
                && self.copy_backdrop_bindings_are_exact,
            copy_backdrop_source_and_result_are_distinct: self.copy_backdrop_count > 0
                && self.copy_backdrop_allocations_are_distinct,
            copy_backdrop_uses_validated_viewport_and_scissor: self.copy_backdrop_count > 0
                && self.copy_backdrop_regions_are_validated,
            copy_backdrop_preserves_signed_mapping: self.copy_backdrop_count > 0
                && self.copy_backdrop_signed_mapping_is_exact,
            backdrop_filter_order_is_exact: backdrop.filter_order_is_exact,
            backdrop_group_order_is_exact: backdrop.group_order_is_exact,
            backdrop_group_resources_are_distinct: backdrop.group_resources_are_distinct,
            #[cfg(test)]
            backdrop_later_sibling_transition_is_exact: backdrop.later_sibling_transition_is_exact,
            blur_pass_count: self.blur_pass_count,
            drop_shadow_colorize_count: self.drop_shadow_colorize_count,
            drop_shadow_merge_count: self.drop_shadow_merge_count,
            spatial_filter_binds_exact_prepared_resources: self.spatial_filter_bindings_are_exact,
            spatial_filter_uses_signed_viewport_and_scissor: self.spatial_filter_regions_are_exact,
            blur_sources_intermediates_and_results_are_distinct: self.blur_allocations_are_distinct,
            spatial_filter_kernels_release_at_validated_last_use: self
                .spatial_filter_kernels_released,
            spatial_filter_textures_release_at_validated_last_use: self
                .spatial_filter_textures_released,
            drop_shadow_reads_original_source_twice: self.shadow_source_read_twice,
            original_source_releases_after_merge: self.shadow_source_released_after_merge,
            advances_every_pass_once: self.completed_pass_count == self.expected_pass_count
                && prepared.next_pass == self.expected_pass_count,
            #[cfg(test)]
            spatial_filter_pass_order: scheduled_filter_raw_facts(&self.scheduled),
            #[cfg(test)]
            capture_count: self.capture_count,
            #[cfg(test)]
            captures_share_one_command_encoder: self.captures_share_one_command_encoder(),
            #[cfg(test)]
            captures_share_one_active_vello_scope: self.captures_share_one_active_vello_scope(),
            #[cfg(test)]
            graph_work_shares_one_command_encoder: self.graph_work_shares_one_command_encoder(
                prepared.color_filter_execution.is_some()
                    || prepared.spatial_filter_execution.is_some()
                    || prepared.backdrop_execution.is_some(),
            ),
            #[cfg(test)]
            capture_observations: self.capture_observations,
        })
    }

    #[cfg(test)]
    fn captures_share_one_command_encoder(&self) -> bool {
        self.capture_observations.first().is_some_and(|first| {
            self.capture_observations.len() == self.expected_capture_count
                && self
                    .capture_observations
                    .iter()
                    .all(|capture| capture.encoder_identity == first.encoder_identity)
        })
    }

    #[cfg(test)]
    fn captures_share_one_active_vello_scope(&self) -> bool {
        self.capture_observations.first().is_some_and(|first| {
            self.capture_observations.len() == self.expected_capture_count
                && self
                    .capture_observations
                    .iter()
                    .all(|capture| capture.scope_identity == first.scope_identity)
        })
    }

    #[cfg(test)]
    fn graph_work_shares_one_command_encoder(&self, has_color_filters: bool) -> bool {
        self.capture_observations
            .first()
            .map(|capture| capture.encoder_identity)
            .or_else(|| self.composite_encoder_identities.first().copied())
            .is_some_and(|identity| {
                self.capture_observations
                    .iter()
                    .all(|capture| capture.encoder_identity == identity)
                    && self
                        .composite_encoder_identities
                        .iter()
                        .all(|composite| *composite == identity)
            })
            && (self.color_filter_count == 0 || has_color_filters)
    }
}

#[cfg(test)]
fn scheduled_filter_raw_facts(
    scheduled: &[GraphScheduledEncodingKind],
) -> Vec<ScheduledFilterRawFact> {
    scheduled
        .iter()
        .filter_map(|kind| match kind {
            GraphScheduledEncodingKind::ColorFilter => Some(ScheduledFilterRawFact::Color),
            GraphScheduledEncodingKind::Convolution => Some(ScheduledFilterRawFact::Convolution),
            GraphScheduledEncodingKind::Offset => Some(ScheduledFilterRawFact::Offset),
            GraphScheduledEncodingKind::BlurHorizontalRgba => {
                Some(ScheduledFilterRawFact::BlurHorizontalRgba)
            }
            GraphScheduledEncodingKind::BlurVerticalRgba => {
                Some(ScheduledFilterRawFact::BlurVerticalRgba)
            }
            GraphScheduledEncodingKind::BlurHorizontalSourceAlpha => {
                Some(ScheduledFilterRawFact::BlurHorizontalSourceAlpha)
            }
            GraphScheduledEncodingKind::BlurVerticalSourceAlpha => {
                Some(ScheduledFilterRawFact::BlurVerticalSourceAlpha)
            }
            GraphScheduledEncodingKind::DropShadowColorize => {
                Some(ScheduledFilterRawFact::DropShadowColorize)
            }
            GraphScheduledEncodingKind::DropShadowMerge => {
                Some(ScheduledFilterRawFact::DropShadowMerge)
            }
            GraphScheduledEncodingKind::ClearRoot
            | GraphScheduledEncodingKind::VelloCapture
            | GraphScheduledEncodingKind::CanonicalizeCapture
            | GraphScheduledEncodingKind::CopyBackdrop
            | GraphScheduledEncodingKind::SpanSourceOver
            | GraphScheduledEncodingKind::LayerComposite
            | GraphScheduledEncodingKind::Present => None,
        })
        .collect()
}

#[derive(Clone, Copy, Debug, Default)]
struct BackdropExecutionReceipt {
    filter_order_is_exact: bool,
    group_order_is_exact: bool,
    group_resources_are_distinct: bool,
    #[cfg(test)]
    later_sibling_transition_is_exact: bool,
}

fn backdrop_execution_receipt(
    prepared: &PreparedGraph<'_>,
    account: &Account,
) -> Result<BackdropExecutionReceipt> {
    let Some(execution) = prepared.backdrop_execution.as_ref() else {
        return Ok(BackdropExecutionReceipt::default());
    };
    let [backdrop] = execution.backdrops.as_slice() else {
        return Ok(BackdropExecutionReceipt::default());
    };
    // These are bounded correctness receipts, not a second activity tally.
    // Direct searches avoid allocating another index after native encoding.
    let position = |id| -> Result<Option<usize>> {
        for (index, pass) in prepared.plan.passes.iter().enumerate() {
            account.work(RenderWork::ResourceProbes, 1)?;
            if pass.runtime.id == id {
                return Ok(Some(index));
            }
        }
        Ok(None)
    };
    let Some(copy) = position(backdrop.copy)? else {
        return Ok(BackdropExecutionReceipt::default());
    };
    let Some(clear) = position(backdrop.group_clear)? else {
        return Ok(BackdropExecutionReceipt::default());
    };
    let Some(backdrop_composite) = position(backdrop.backdrop_composite)? else {
        return Ok(BackdropExecutionReceipt::default());
    };
    let Some(outer) = position(backdrop.outer_composite)? else {
        return Ok(BackdropExecutionReceipt::default());
    };
    let filters_are_ordered = |steps: &[ExecutableFilterStepFacts], start, end| -> Result<bool> {
        let Some(passes) = prepared.plan.passes.get(start + 1..end) else {
            return Ok(false);
        };
        let mut passes = passes.iter();
        for expected in filter_step_passes(steps, account) {
            let expected = expected?;
            let Some(pass) = passes.next() else {
                return Ok(false);
            };
            account.work(RenderWork::ResourceProbes, 1)?;
            if pass.runtime.id != expected {
                return Ok(false);
            }
        }
        Ok(true)
    };
    let backdrop_filters_are_ordered = if backdrop.filter_steps.is_empty() {
        copy.checked_add(1) == Some(clear)
    } else {
        filters_are_ordered(&backdrop.filter_steps, copy, clear)?
    };
    let assembled_group = match backdrop.foreground_composite {
        Some(pass) => {
            let Some(foreground) = position(pass)? else {
                return Ok(BackdropExecutionReceipt::default());
            };
            foreground
        }
        None => backdrop_composite,
    };
    let Some(group_steps) = execution
        .filter_steps
        .get(backdrop.group_filter_steps.clone())
    else {
        return Ok(BackdropExecutionReceipt::default());
    };
    let group_filters_are_ordered = filters_are_ordered(group_steps, assembled_group, outer)?;
    let resources = [
        Some(backdrop.completed_parent),
        Some(backdrop.copied),
        backdrop.distinct_filtered_result(),
        Some(backdrop.completed_group),
        Some(backdrop.result),
        backdrop.foreground,
        (!group_steps.is_empty()).then_some(backdrop.outer_source),
    ];
    let mut resources_are_distinct = true;
    'resources: for (index, resource) in resources.iter().enumerate() {
        if resource.is_some() {
            for candidate in &resources[..index] {
                account.work(RenderWork::ResourceProbes, 1)?;
                if candidate == resource {
                    resources_are_distinct = false;
                    break 'resources;
                }
            }
        }
    }
    #[cfg(test)]
    let later_sibling_transition_is_exact =
        prepared.plan.passes.iter().skip(outer + 1).any(|pass| {
            pass.runtime
                .dependencies
                .contains(&backdrop.outer_composite)
                && pass
                    .runtime
                    .reads
                    .iter()
                    .any(|read| read.resource == backdrop.result)
        });
    Ok(BackdropExecutionReceipt {
        filter_order_is_exact: backdrop_filters_are_ordered,
        group_order_is_exact: copy < clear
            && clear < backdrop_composite
            && (backdrop.foreground_composite.is_none() || backdrop_composite < assembled_group)
            && assembled_group < outer
            && group_filters_are_ordered,
        group_resources_are_distinct: resources_are_distinct,
        #[cfg(test)]
        later_sibling_transition_is_exact,
    })
}

impl CustomSpineEncodingSummary {
    fn proves_complete_submission(&self) -> bool {
        let common = (self.offset_pass_count == 0
            || self.offset_parameters_and_resources_are_exact)
            && self.encodes_custom_passes_in_order
            && (self.convolution_pass_count == 0
                || self.convolution_parameters_and_resources_are_exact)
            && self.clears_full_root_once
            && self.uses_exact_prepared_spatial_mapping
            && self.presents_to_exact_external_output
            && self.capture_handoffs_match_expected_count
            && self.validates_checked_capture_completion
            && self.completes_custom_passes_after_encoding
            && self.parent_and_result_are_distinct
            && self.copies_full_parent_before_bounded_source_render
            && self.samples_only_source_with_fixed_premultiplied_blend
            && self.preserves_signed_source_origin
            && self.keeps_cache_update_provisional
            && self.advances_every_pass_once;
        let exact_layers = self
            .normal_composite_count
            .saturating_add(self.destination_composite_count)
            == self.layer_composite_count
            && (self.normal_composite_count == 0
                || (self.normal_composites_use_fixed_premultiplied_blend
                    && self.normal_composites_omit_parent_sample))
            && (self.destination_composite_count == 0
                || (self.destination_composites_copy_full_parent
                    && self.destination_composites_avoid_read_write_alias))
            && (self.layer_composite_count == 0
                || (self.layer_composites_bind_exact_resources_and_parameters
                    && self.layer_composites_preserve_signed_mapping));
        let spatial_pass_count = self
            .blur_pass_count
            .saturating_add(self.drop_shadow_colorize_count)
            .saturating_add(self.drop_shadow_merge_count);
        let spatial_pass_count = spatial_pass_count
            .saturating_add(self.convolution_pass_count)
            .saturating_add(self.offset_pass_count);
        let exact_base = self.layer_composite_count == 0
            && self.color_filter_count == 0
            && self.copy_backdrop_count == 0
            && spatial_pass_count == 0;
        let exact_composition = self.layer_composite_count > 0
            && self.color_filter_count == 0
            && self.copy_backdrop_count == 0
            && spatial_pass_count == 0
            && self
                .normal_composite_count
                .saturating_add(self.destination_composite_count)
                == self.layer_composite_count
            && (self.normal_composite_count == 0
                || (self.normal_composites_use_fixed_premultiplied_blend
                    && self.normal_composites_omit_parent_sample))
            && (self.destination_composite_count == 0
                || (self.destination_composites_copy_full_parent
                    && self.destination_composites_avoid_read_write_alias))
            && self.layer_composites_bind_exact_resources_and_parameters
            && self.layer_composites_preserve_signed_mapping;
        let exact_color_filter = self.color_filter_count > 0
            && self.copy_backdrop_count == 0
            && spatial_pass_count == 0
            && self.color_filters_preserve_authored_order
            && self.color_filters_bind_exact_source_spatial_and_operations
            && self.color_filter_sources_and_results_are_distinct
            && self.color_filters_use_validated_viewport_and_scissor
            && self.color_filters_preserve_signed_texel_mapping
            && self.color_filter_operation_buffers_released
            && exact_layers;
        let exact_drop_shadows = (self.drop_shadow_colorize_count == 0
            && self.drop_shadow_merge_count == 0)
            || (self.drop_shadow_colorize_count > 0
                && self.drop_shadow_colorize_count == self.drop_shadow_merge_count
                && self.drop_shadow_reads_original_source_twice
                && self.original_source_releases_after_merge);
        let exact_color_filters = self.color_filter_count == 0
            || (self.color_filters_preserve_authored_order
                && self.color_filters_bind_exact_source_spatial_and_operations
                && self.color_filter_sources_and_results_are_distinct
                && self.color_filters_use_validated_viewport_and_scissor
                && self.color_filters_preserve_signed_texel_mapping
                && self.color_filter_operation_buffers_released);
        let exact_spatial_passes = (self.blur_pass_count == 0
            && self.drop_shadow_colorize_count == 0
            && self.drop_shadow_merge_count == 0)
            || (self.blur_pass_count > 0
                && exact_drop_shadows
                && self.spatial_filter_binds_exact_prepared_resources
                && self.spatial_filter_uses_signed_viewport_and_scissor
                && self.blur_sources_intermediates_and_results_are_distinct
                && self.spatial_filter_kernels_release_at_validated_last_use
                && self.spatial_filter_textures_release_at_validated_last_use);
        let exact_spatial_filter = self.copy_backdrop_count == 0
            && (self.blur_pass_count > 0
                || self.convolution_pass_count > 0
                || self.offset_pass_count > 0)
            && exact_layers
            && exact_color_filters
            && exact_spatial_passes;
        let exact_backdrop = self.copy_backdrop_count == 1
            && self.backdrop_filter_order_is_exact
            && self.copy_backdrop_binds_exact_prepared_resources
            && self.copy_backdrop_source_and_result_are_distinct
            && self.copy_backdrop_uses_validated_viewport_and_scissor
            && self.copy_backdrop_preserves_signed_mapping
            && self.backdrop_group_order_is_exact
            && self.backdrop_group_resources_are_distinct
            && exact_layers
            && exact_color_filters
            && exact_spatial_passes;
        common
            && (exact_base
                || exact_composition
                || exact_color_filter
                || exact_spatial_filter
                || exact_backdrop)
    }
}

struct EncodedVelloCaptureResult {
    receipt: VelloCaptureCompletionReceipt,
    #[cfg(test)]
    observation: EncodedCaptureRawFact,
}

#[cfg(test)]
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct EncodedCaptureRawFact {
    pub(crate) lowers_with_exact_initial_transform: bool,
    pub(crate) uses_transparent_base: bool,
    pub(crate) antialiasing: Antialiasing,
    pub(crate) target_extent: PhysicalSize,
    pub(crate) target_format: wgpu::TextureFormat,
    pub(crate) target_usage: wgpu::TextureUsages,
    pub(crate) target_and_view_are_exact: bool,
    encoder_identity: usize,
    scope_identity: usize,
}

struct VelloCaptureEncodingContext<'encoding, 'device> {
    engine: &'device VelloEngineState,
    resources: &'device ResourceManager,
    queue: &'device wgpu::Queue,
    scope: &'encoding mut ActiveVelloEncodingScope<'device>,
    leases: &'encoding mut VelloResourceLeaseAggregate,
    session: &'encoding Arc<()>,
}

struct SpatialPassDrawBindings<'a> {
    pipeline: &'a wgpu::RenderPipeline,
    bind_group: &'a wgpu::BindGroup,
}

/// Owns the scope-clean capture leases until they enter the transaction
/// submission payload. Dropping this value aborts every capture lease.
#[must_use = "encoded graph captures must remain pending until transaction resolution"]
pub(crate) struct PendingGraphEncoding {
    pub(super) summary: CustomSpineEncodingSummary,
    pub(super) resources: PendingVelloResourceCommit,
    session: Arc<()>,
}

#[must_use = "prepared graph frame state must commit only after graph transaction success"]
pub(crate) struct PendingPreparedFrameCommit {
    pub(super) frame_scope: FrameResourceScope,
    pass_cache_update: ProvisionalDevicePassCacheUpdate,
}

/// Owns the checked cache reservation and prepared frame between graph
/// readiness checks.
#[must_use = "accounting-ready prepared graph state must be committed or aborted on drop"]
pub(crate) struct AccountingReadyPreparedFrameCommit<'cache> {
    frame_scope: FrameResourceScope,
    pass_cache_update: ReadyPassCacheUpdate<'cache>,
}

impl PendingPreparedFrameCommit {
    pub(crate) fn into_accounting_ready<'cache>(
        self,
        pass_cache: &'cache mut DevicePassCache,
        account: &Account,
    ) -> Result<AccountingReadyPreparedFrameCommit<'cache>> {
        let Self {
            frame_scope,
            pass_cache_update,
        } = self;
        let pass_cache_update = match pass_cache_update.prepare_commit(pass_cache, account) {
            PassCacheCommitPreparation::Ready(update) => update,
            PassCacheCommitPreparation::Rejected { update, error } => {
                // The frame scope releases before the provisional cache handles.
                drop(frame_scope);
                drop(update);
                return Err(error);
            }
        };
        let ready = AccountingReadyPreparedFrameCommit {
            frame_scope,
            pass_cache_update,
        };
        ready.frame_scope.ensure_commit_ready(&[])?;
        Ok(ready)
    }
}

impl AccountingReadyPreparedFrameCommit<'_> {
    pub(crate) fn ensure_commit_ready(&mut self, account: &Account) -> Result<()> {
        self.pass_cache_update.recheck(account)?;
        self.frame_scope.ensure_commit_ready(&[])
    }

    pub(crate) fn commit(self) -> Result<FrameCleanup> {
        let frame_cleanup = self.frame_scope.finish_checked()?;
        self.pass_cache_update.commit();
        Ok(frame_cleanup)
    }
}

#[must_use = "graph submission state must remain owned by one transaction payload"]
pub(crate) struct PreparedGraphSubmission {
    capture_resources: PendingVelloResourceCommit,
    prepared_frame: PendingPreparedFrameCommit,
}

impl PreparedGraphSubmission {
    pub(crate) fn into_parts(self) -> (PendingVelloResourceCommit, PendingPreparedFrameCommit) {
        (self.capture_resources, self.prepared_frame)
    }
}

#[derive(Clone)]
struct PreparedPassEncodingRequest {
    id: RuntimePassId,
    kind: RuntimePassKind,
    reads: SharedValue<Vec<RuntimeReadBinding>>,
    result: RuntimeResultBinding,
    spatial_uniform: Option<PassSpatialUniformBytes>,
    blur_edge_parameters: Option<BlurEdgeParameterBytes>,
    color_filter_operations: Option<ColorFilterOperationBytes>,
    drop_shadow_parameters: Option<DropShadowParameterBytes>,
    composite_parameters: Option<CompositeParameterBytes>,
    cache_keys: Option<RuntimePassCacheKeys>,
    releases: SharedValue<Vec<RuntimeResourceId>>,
    kernel_releases: SharedValue<Vec<GaussianKernelKey>>,
    convolution_parameters: Option<ConvolutionParameterBytes>,
    offset_parameters: Option<OffsetParameterBytes>,
    convolution_weights: Option<RuntimePassId>,
}

/// Created only after atomic lease resolution and every corresponding binding
/// take succeeds. Encoding requests can only clone these immutable release
/// sources from their exact prepared request, so fixed headers bind this linear
/// witness to the pass without a post-effect content walk or lookup.
#[must_use = "completed releases must supply their semantic receipt or be discarded"]
struct CompletedPassReleases {
    pass: RuntimePassId,
    resources: SharedValue<Vec<RuntimeResourceId>>,
    kernels: SharedValue<Vec<GaussianKernelKey>>,
    color_buffer_taken: bool,
    offset_buffer_taken: bool,
}

/// Scheduled bindings from the graph whose frame resolved their exact leases.
struct ResolvedPassReleases<'graph> {
    pass: RuntimePassId,
    resources: SharedValue<Vec<RuntimeResourceId>>,
    kernels: SharedValue<Vec<GaussianKernelKey>>,
    releases_color_filter_operation: bool,
    releases_offset_parameters: bool,
    offset_parameter_bindings: &'graph mut PassMap<RuntimePassId, PreparedOffsetParameterBinding>,
    resource_bindings: &'graph mut PassMap<RuntimeResourceId, PreparedResourceBinding>,
    kernel_bindings: &'graph mut PassMap<GaussianKernelKey, PreparedKernelBinding>,
    color_filter_operation_bindings:
        &'graph mut PassMap<RuntimePassId, PreparedColorFilterOperationBinding>,
    convolution_weight_bindings:
        &'graph mut PassMap<RuntimePassId, PreparedConvolutionWeightsBinding>,
    convolution_release: Option<RuntimePassId>,
}

impl<'graph> ResolvedPassReleases<'graph> {
    fn try_resolve(
        graph: &'graph mut PreparedGraph<'_>,
        pass: RuntimePassId,
        account: &Account,
    ) -> Result<Self> {
        let request = graph.require_current_pass(pass, account)?;
        let resource_releases = request.runtime.releases.clone();
        let kernel_releases = request.kernel_releases.clone();
        let releases_color_filter_operation = request.color_filter_operations.is_some();
        let releases_offset_parameters = request.offset_parameters.is_some();
        let convolution_release = request
            .convolution_weights
            .map(|owner| -> Result<Option<RuntimePassId>> {
                let binding = graph
                    .convolution_weight_bindings
                    .get(&owner, account)?
                    .ok_or_else(|| {
                        preparation_error("convolution weight release binding is missing")
                    })?;
                if binding.lease.is_none() {
                    return Err(preparation_error(
                        "convolution weights are stale or already released",
                    ));
                }
                Ok((binding.last_use == pass).then_some(owner))
            })
            .transpose()?
            .flatten();

        for resource in &resource_releases {
            account.work(RenderWork::GraphEdgeVisits, 1)?;
            let binding = graph
                .resource_bindings
                .get(resource, account)?
                .ok_or_else(|| preparation_error("prepared runtime release binding is missing"))?;
            if binding.lease.is_none() {
                return Err(preparation_error(
                    "prepared runtime release is stale or duplicate",
                ));
            }
        }
        for kernel in &kernel_releases {
            account.work(RenderWork::GraphEdgeVisits, 1)?;
            let binding = graph
                .kernel_bindings
                .get(kernel, account)?
                .ok_or_else(|| preparation_error("prepared Gaussian release binding is missing"))?;
            if binding.lease.is_none() {
                return Err(preparation_error(
                    "prepared Gaussian release is stale or duplicate",
                ));
            }
        }
        if releases_color_filter_operation
            && graph
                .color_filter_operation_bindings
                .get(&pass, account)?
                .is_none_or(|binding| binding.buffer.is_none())
        {
            return Err(preparation_error(
                "prepared color-filter operation release is stale or missing",
            ));
        }

        if releases_offset_parameters
            && graph
                .offset_parameter_bindings
                .get(&pass, account)?
                .is_none_or(|binding| binding.buffer.is_none())
        {
            return Err(preparation_error(
                "prepared offset uniform release is stale or missing",
            ));
        }

        let PreparedGraph {
            offset_parameter_bindings,
            resource_bindings,
            kernel_bindings,
            color_filter_operation_bindings,
            convolution_weight_bindings,
            frame_scope,
            ..
        } = graph;
        let mut leases = account.temporary(
            resource_releases.len()
                + kernel_releases.len()
                + usize::from(convolution_release.is_some()),
        )?;
        for resource in &resource_releases {
            account.work(RenderWork::GraphEdgeVisits, 1)?;
            let lease = resource_bindings
                .get(resource, account)?
                .and_then(|binding| binding.lease.as_ref())
                .ok_or_else(|| {
                    preparation_error("prepared resource release is unavailable after validation")
                })?;
            leases.append_slot()?.write(lease);
        }
        for kernel in &kernel_releases {
            account.work(RenderWork::GraphEdgeVisits, 1)?;
            let lease = kernel_bindings
                .get(kernel, account)?
                .and_then(|binding| binding.lease.as_ref())
                .ok_or_else(|| {
                    preparation_error("prepared Gaussian release is unavailable after validation")
                })?;
            leases.append_slot()?.write(lease);
        }
        if let Some(owner) = convolution_release {
            let lease = convolution_weight_bindings
                .get(&owner, account)?
                .and_then(|binding| binding.lease.as_ref())
                .ok_or_else(|| preparation_error("convolution release became unavailable"))?;
            leases.append_slot()?.write(lease);
        }
        frame_scope
            .as_mut()
            .ok_or_else(|| preparation_error("prepared frame resource scope is closed"))?
            .resolve_leases_atomically(&leases)?;
        drop(leases);
        Ok(Self {
            pass,
            resources: resource_releases,
            kernels: kernel_releases,
            releases_color_filter_operation,
            releases_offset_parameters,
            offset_parameter_bindings,
            resource_bindings,
            kernel_bindings,
            color_filter_operation_bindings,
            convolution_weight_bindings,
            convolution_release,
        })
    }

    fn take_bindings(self) -> Result<CompletedPassReleases> {
        let mut first_missing = None;
        for resource in self.resources.iter() {
            let binding = self
                .resource_bindings
                .maintenance_get_mut(resource)
                .and_then(|binding| binding.lease.take());
            Self::drop_required_binding(
                binding,
                "prepared resource release is unavailable after atomic resolution",
                &mut first_missing,
            );
        }
        for kernel in self.kernels.iter() {
            let binding = self
                .kernel_bindings
                .maintenance_get_mut(kernel)
                .and_then(|binding| binding.lease.take());
            Self::drop_required_binding(
                binding,
                "prepared Gaussian release is unavailable after atomic resolution",
                &mut first_missing,
            );
        }
        if self.releases_color_filter_operation {
            let binding = self
                .color_filter_operation_bindings
                .maintenance_get_mut(&self.pass)
                .and_then(|binding| binding.buffer.take());
            Self::drop_required_binding(
                binding,
                "prepared color-filter operation release is unavailable after atomic resolution",
                &mut first_missing,
            );
        }
        if self.releases_offset_parameters {
            let binding = self
                .offset_parameter_bindings
                .maintenance_get_mut(&self.pass)
                .and_then(|binding| binding.buffer.take());
            Self::drop_required_binding(
                binding,
                "offset uniform missing after atomic resource release",
                &mut first_missing,
            );
        }
        if let Some(owner) = self.convolution_release {
            let lease = self
                .convolution_weight_bindings
                .maintenance_get_mut(&owner)
                .and_then(|binding| binding.lease.take());
            Self::drop_required_binding(
                lease,
                "convolution coefficient lease missing after atomic release",
                &mut first_missing,
            );
        }
        if let Some(message) = first_missing {
            return Err(preparation_error(message));
        }
        Ok(CompletedPassReleases {
            pass: self.pass,
            resources: self.resources,
            kernels: self.kernels,
            color_buffer_taken: self.releases_color_filter_operation,
            offset_buffer_taken: self.releases_offset_parameters,
        })
    }

    fn drop_required_binding<T>(
        binding: Option<T>,
        missing: &'static str,
        first_missing: &mut Option<&'static str>,
    ) {
        match binding {
            Some(binding) => drop(binding),
            None => {
                first_missing.get_or_insert(missing);
            }
        }
    }
}

struct CompletedReleaseFacts {
    resources: bool,
    kernels: bool,
    color_buffer: bool,
    offset_buffer: bool,
}

impl CompletedPassReleases {
    fn prove_for(self, request: &PreparedPassEncodingRequest) -> CompletedReleaseFacts {
        let exact_source = self.pass == request.id
            && self.resources.same_allocation(&request.releases)
            && self.kernels.same_allocation(&request.kernel_releases);
        CompletedReleaseFacts {
            resources: exact_source,
            kernels: exact_source,
            color_buffer: exact_source && self.color_buffer_taken,
            offset_buffer: exact_source && self.offset_buffer_taken,
        }
    }
}

impl From<&RuntimePassPreparationRequest> for PreparedPassEncodingRequest {
    fn from(request: &RuntimePassPreparationRequest) -> Self {
        Self {
            id: request.runtime.id,
            kind: request.runtime.kind.clone(),
            reads: request.runtime.reads.clone(),
            result: request.runtime.result,
            spatial_uniform: request.spatial_uniform.clone(),
            blur_edge_parameters: request.blur_edge_parameters.clone(),
            color_filter_operations: request.color_filter_operations.clone(),
            drop_shadow_parameters: request.drop_shadow_parameters.clone(),
            composite_parameters: request.composite_parameters.clone(),
            cache_keys: request.cache_keys.clone(),
            releases: request.runtime.releases.clone(),
            kernel_releases: request.kernel_releases.clone(),
            convolution_parameters: request.convolution_parameters.clone(),
            offset_parameters: request.offset_parameters.clone(),
            convolution_weights: request.convolution_weights,
        }
    }
}

#[derive(Clone, Copy, Debug)]
struct GraphRenderRegion {
    viewport_x: f32,
    viewport_y: f32,
    viewport_width: f32,
    viewport_height: f32,
    scissor_x: u32,
    scissor_y: u32,
    scissor_width: u32,
    scissor_height: u32,
    unclipped_x: f64,
    unclipped_y: f64,
}

#[derive(Clone, Copy, Debug, Default)]
struct CorePassEncodingFacts {
    full_target: bool,
    exact_spatial_uniform: bool,
    external_output_exact: bool,
    parent_and_result_distinct: bool,
    copied_full_parent_before_render: bool,
    sampled_only_source: bool,
    fixed_source_over_blend: bool,
    preserved_signed_source_origin: bool,
}

#[derive(Clone, Copy, Debug)]
struct LayerCompositeEncodingFacts {
    normal_path: bool,
    destination_path: bool,
    fixed_premultiplied_blend: bool,
    omits_parent_sample: bool,
    copied_full_parent: bool,
    avoids_read_write_alias: bool,
    exact_resources_and_parameters: bool,
    preserved_signed_mapping: bool,
    #[cfg(test)]
    encoder_identity: usize,
}

#[derive(Clone, Copy, Debug, Default)]
struct ColorFilterEncodingFacts {
    exact_operation_bytes: bool,
    exact_source_spatial_and_operations: bool,
    source_and_result_are_distinct: bool,
    validated_viewport_and_scissor: bool,
    preserved_signed_texel_mapping: bool,
}

#[derive(Clone, Copy, Debug, Default)]
struct CopyBackdropEncodingFacts {
    exact_prepared_bindings: bool,
    source_and_result_are_distinct: bool,
    validated_viewport_and_scissor: bool,
    preserved_signed_mapping: bool,
}

#[derive(Clone, Copy, Debug, Default)]
struct BlurEncodingFacts {
    exact_prepared_bindings: bool,
    distinct_source_and_result: bool,
    validated_region: bool,
    preserved_signed_mapping: bool,
}

#[derive(Clone, Copy, Debug, Default)]
struct DropShadowColorizeEncodingFacts {
    exact_prepared_bindings: bool,
    distinct_source_and_result: bool,
    validated_region: bool,
    preserved_signed_mapping: bool,
}

#[derive(Clone, Copy, Debug, Default)]
struct DropShadowMergeEncodingFacts {
    exact_prepared_bindings: bool,
    distinct_source_shadow_and_result: bool,
    validated_region: bool,
    preserved_signed_mapping: bool,
    fixed_source_over_blend: bool,
    reads_original_source_and_shadow: bool,
}

struct PreparedColorFilterEncoding<'prepared> {
    source_binding: PreparedTextureBinding<'prepared>,
    target_binding: PreparedTextureBinding<'prepared>,
    objects: ProvisionalColorFilterPassObjects<'prepared>,
    spatial: &'prepared PassSpatialUniformBytes,
    operation_buffer: &'prepared wgpu::Buffer,
    region: GraphRenderRegion,
    facts: ColorFilterEncodingFacts,
}

struct LayerCompositeSemantic<'prepared> {
    transform: Transform,
    parameters: &'prepared RuntimeLayerCompositeParameters,
    parent: RuntimeReadBinding,
    source: RuntimeReadBinding,
    clip: Option<RuntimeReadBinding>,
    alpha_mask: Option<RuntimeReadBinding>,
    target: RuntimeResourceId,
    normal_path: bool,
    destination_path: bool,
}

struct LayerCompositeBindings<'prepared> {
    semantic: LayerCompositeSemantic<'prepared>,
    parent: PreparedTextureBinding<'prepared>,
    source: PreparedTextureBinding<'prepared>,
    target: PreparedTextureBinding<'prepared>,
    clip: Option<PreparedTextureBinding<'prepared>>,
    mask: Option<PreparedTextureBinding<'prepared>>,
    parent_spatial: RuntimeSpatialDescriptor,
    source_spatial: RuntimeSpatialDescriptor,
    target_spatial: RuntimeSpatialDescriptor,
    parent_and_result_are_distinct: bool,
    sampled_allocations_are_distinct: bool,
}

struct CorePassSampledRenderTarget<'target> {
    view: &'target wgpu::TextureView,
    extent: PhysicalSize,
    region: Option<GraphRenderRegion>,
    load: wgpu::LoadOp<wgpu::Color>,
    label: &'static str,
}

impl GraphRenderRegion {
    fn full(extent: PhysicalSize) -> Result<Self> {
        if extent.width() == 0 || extent.height() == 0 {
            return Err(preparation_error(
                "the graph render region requires a positive target extent",
            ));
        }
        let viewport_width = extent.width() as f32;
        let viewport_height = extent.height() as f32;
        if !viewport_width.is_finite() || !viewport_height.is_finite() {
            return Err(preparation_error(
                "the graph render extent cannot be represented by WGPU viewport coordinates",
            ));
        }
        Ok(Self {
            viewport_x: 0.0,
            viewport_y: 0.0,
            viewport_width,
            viewport_height,
            scissor_x: 0,
            scissor_y: 0,
            scissor_width: extent.width(),
            scissor_height: extent.height(),
            unclipped_x: 0.0,
            unclipped_y: 0.0,
        })
    }

    fn bounded_source(
        source: RuntimeSpatialDescriptor,
        destination: RuntimeSpatialDescriptor,
    ) -> Result<Option<Self>> {
        let source_width = f64::from(source.device_extent.width()) / source.raster_scale;
        let source_height = f64::from(source.device_extent.height()) / source.raster_scale;
        let unclipped_x =
            (source.texel_origin.x() - destination.texel_origin.x()) * destination.raster_scale;
        let unclipped_y =
            (source.texel_origin.y() - destination.texel_origin.y()) * destination.raster_scale;
        let unclipped_end_x = (source.texel_origin.x() + source_width
            - destination.texel_origin.x())
            * destination.raster_scale;
        let unclipped_end_y = (source.texel_origin.y() + source_height
            - destination.texel_origin.y())
            * destination.raster_scale;
        Self::bounded_unclipped(
            [unclipped_x, unclipped_y, unclipped_end_x, unclipped_end_y],
            destination,
            "the graph signed bounded render mapping is non-finite",
            "the graph bounded viewport or scissor cannot represent its signed mapping",
        )
    }

    fn bounded_transformed_source(
        source: RuntimeSpatialDescriptor,
        destination: RuntimeSpatialDescriptor,
        source_to_destination: Transform,
    ) -> Result<(Option<Self>, [f64; 4])> {
        let bounds = composition_transformed_source_device_bounds(
            source,
            destination,
            source_to_destination,
        )?;
        let region = Self::bounded_unclipped(
            bounds,
            destination,
            "the composition transformed bounded render mapping is non-finite",
            "the composition transformed viewport or scissor cannot represent its signed mapping",
        )?;
        Ok((region, bounds))
    }

    fn bounded_unclipped(
        [unclipped_x, unclipped_y, unclipped_end_x, unclipped_end_y]: [f64; 4],
        destination: RuntimeSpatialDescriptor,
        non_finite_message: &'static str,
        invalid_region_message: &'static str,
    ) -> Result<Option<Self>> {
        if [unclipped_x, unclipped_y, unclipped_end_x, unclipped_end_y]
            .iter()
            .any(|value| !value.is_finite())
        {
            return Err(preparation_error(non_finite_message));
        }

        let destination_width = f64::from(destination.device_extent.width());
        let destination_height = f64::from(destination.device_extent.height());
        let clipped_x = unclipped_x.max(0.0).min(destination_width);
        let clipped_y = unclipped_y.max(0.0).min(destination_height);
        let clipped_end_x = unclipped_end_x.max(0.0).min(destination_width);
        let clipped_end_y = unclipped_end_y.max(0.0).min(destination_height);
        if clipped_end_x <= clipped_x || clipped_end_y <= clipped_y {
            return Ok(None);
        }

        let scissor_x = clipped_x.floor() as u32;
        let scissor_y = clipped_y.floor() as u32;
        let scissor_end_x = clipped_end_x.ceil() as u32;
        let scissor_end_y = clipped_end_y.ceil() as u32;
        let viewport_x = clipped_x as f32;
        let viewport_y = clipped_y as f32;
        let viewport_width = (clipped_end_x - clipped_x) as f32;
        let viewport_height = (clipped_end_y - clipped_y) as f32;
        if !viewport_x.is_finite()
            || !viewport_y.is_finite()
            || !viewport_width.is_finite()
            || !viewport_height.is_finite()
            || viewport_width <= 0.0
            || viewport_height <= 0.0
            || scissor_end_x <= scissor_x
            || scissor_end_y <= scissor_y
        {
            return Err(preparation_error(invalid_region_message));
        }
        Ok(Some(Self {
            viewport_x,
            viewport_y,
            viewport_width,
            viewport_height,
            scissor_x,
            scissor_y,
            scissor_width: scissor_end_x - scissor_x,
            scissor_height: scissor_end_y - scissor_y,
            unclipped_x,
            unclipped_y,
        }))
    }
}

fn composition_transformed_source_device_bounds(
    source: RuntimeSpatialDescriptor,
    destination: RuntimeSpatialDescriptor,
    source_to_destination: Transform,
) -> Result<[f64; 4]> {
    let source_width = f64::from(source.device_extent.width()) / source.raster_scale;
    let source_height = f64::from(source.device_extent.height()) / source.raster_scale;
    let source_end_x = source.texel_origin.x() + source_width;
    let source_end_y = source.texel_origin.y() + source_height;
    let [a, b, c, d, e, f] = source_to_destination.as_array();
    let mut minimum_x = f64::INFINITY;
    let mut minimum_y = f64::INFINITY;
    let mut maximum_x = f64::NEG_INFINITY;
    let mut maximum_y = f64::NEG_INFINITY;
    for (source_x, source_y) in [
        (source.texel_origin.x(), source.texel_origin.y()),
        (source_end_x, source.texel_origin.y()),
        (source.texel_origin.x(), source_end_y),
        (source_end_x, source_end_y),
    ] {
        let destination_x = a * source_x + c * source_y + e;
        let destination_y = b * source_x + d * source_y + f;
        let device_x = (destination_x - destination.texel_origin.x()) * destination.raster_scale;
        let device_y = (destination_y - destination.texel_origin.y()) * destination.raster_scale;
        if !device_x.is_finite() || !device_y.is_finite() {
            return Err(preparation_error(
                "the composition transformed source bounds are non-finite",
            ));
        }
        minimum_x = minimum_x.min(device_x);
        minimum_y = minimum_y.min(device_y);
        maximum_x = maximum_x.max(device_x);
        maximum_y = maximum_y.max(device_y);
    }
    Ok([minimum_x, minimum_y, maximum_x, maximum_y])
}

fn encode_spatial_filter_full_target_pass(
    encoder: &mut wgpu::CommandEncoder,
    target: &wgpu::TextureView,
    bindings: SpatialPassDrawBindings<'_>,
    region: GraphRenderRegion,
    label: &'static str,
    filter: bool,
    account: &Account,
) -> Result<()> {
    encode_spatial_filter_full_target_pass_with_samples(
        encoder, target, bindings, region, label, filter, None, account,
    )
}

#[expect(
    clippy::too_many_arguments,
    reason = "one exact draw carries its optional scheduled sample footprint"
)]
fn encode_spatial_filter_full_target_pass_with_samples(
    encoder: &mut wgpu::CommandEncoder,
    target: &wgpu::TextureView,
    bindings: SpatialPassDrawBindings<'_>,
    region: GraphRenderRegion,
    label: &'static str,
    filter: bool,
    offset_samples: Option<u64>,
    account: &Account,
) -> Result<()> {
    if let Some(samples) = offset_samples {
        account.work_batch([
            (RenderWork::ImagePasses, 1),
            (
                RenderWork::ScheduledFilterTexels,
                scheduled_region_texels(region),
            ),
            (RenderWork::ScheduledOffsetSamples, samples),
        ])?;
    } else if filter {
        account.work_batch([
            (RenderWork::ImagePasses, 1),
            (
                RenderWork::ScheduledFilterTexels,
                scheduled_region_texels(region),
            ),
        ])?;
    } else {
        account.work(RenderWork::ImagePasses, 1)?;
    }
    let mut pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
        label: Some(label),
        color_attachments: &[Some(wgpu::RenderPassColorAttachment {
            view: target,
            resolve_target: None,
            ops: wgpu::Operations {
                load: wgpu::LoadOp::Clear(wgpu::Color::TRANSPARENT),
                store: wgpu::StoreOp::Store,
            },
            depth_slice: None,
        })],
        depth_stencil_attachment: None,
        occlusion_query_set: None,
        timestamp_writes: None,
        multiview_mask: None,
    });
    pass.set_pipeline(bindings.pipeline);
    pass.set_bind_group(0, bindings.bind_group, &[]);
    pass.set_viewport(
        region.viewport_x,
        region.viewport_y,
        region.viewport_width,
        region.viewport_height,
        0.0,
        1.0,
    );
    pass.set_scissor_rect(
        region.scissor_x,
        region.scissor_y,
        region.scissor_width,
        region.scissor_height,
    );
    pass.draw(0..3, 0..1);
    Ok(())
}

fn spatial_filter_full_region_is_exact(region: GraphRenderRegion, extent: PhysicalSize) -> bool {
    region.viewport_x == 0.0
        && region.viewport_y == 0.0
        && region.viewport_width == extent.width() as f32
        && region.viewport_height == extent.height() as f32
        && region.scissor_x == 0
        && region.scissor_y == 0
        && region.scissor_width == extent.width()
        && region.scissor_height == extent.height()
}

fn scheduled_region_texels(region: GraphRenderRegion) -> u64 {
    // Both dimensions are u32 scissor extents, so their product fits u64.
    u64::from(region.scissor_width) * u64::from(region.scissor_height)
}

fn exact_core_pass_read<'a>(
    request: &'a PreparedPassEncodingRequest,
    role: RuntimeReadRole,
    account: &Account,
) -> Result<&'a RuntimeReadBinding> {
    let mut matching = None;
    for read in request.reads.iter() {
        account.work(RenderWork::ResourceProbes, 1)?;
        if read.role == role && matching.replace(read).is_some() {
            return Err(preparation_error(
                "the prepared graph source binding is duplicated",
            ));
        }
    }
    matching.ok_or_else(|| preparation_error("the prepared graph source binding is missing"))
}

fn graph_scheduled_encoding_order_is_exact(
    scheduled: &[GraphScheduledEncodingKind],
    passes: &[RuntimePassPreparationRequest],
    account: &Account,
) -> Result<bool> {
    if scheduled.len() != passes.len() {
        return Ok(false);
    }
    for (scheduled, pass) in scheduled.iter().zip(passes) {
        account.work(RenderWork::GraphNodeVisits, 1)?;
        let expected = match &pass.runtime.kind {
            RuntimePassKind::ClearRoot { .. } => GraphScheduledEncodingKind::ClearRoot,
            RuntimePassKind::VelloCapture(Some(_)) => GraphScheduledEncodingKind::VelloCapture,
            RuntimePassKind::CanonicalizeCapture => GraphScheduledEncodingKind::CanonicalizeCapture,
            RuntimePassKind::CopyBackdrop => GraphScheduledEncodingKind::CopyBackdrop,
            RuntimePassKind::ColorFilter(Some(_)) => GraphScheduledEncodingKind::ColorFilter,
            RuntimePassKind::Convolution(_) => GraphScheduledEncodingKind::Convolution,
            RuntimePassKind::Offset(_) => GraphScheduledEncodingKind::Offset,
            RuntimePassKind::BlurHorizontal(Some(blur)) => match blur.input {
                RuntimeBlurInput::Rgba => GraphScheduledEncodingKind::BlurHorizontalRgba,
                RuntimeBlurInput::SourceAlpha => {
                    GraphScheduledEncodingKind::BlurHorizontalSourceAlpha
                }
            },
            RuntimePassKind::BlurVertical(Some(blur)) => match blur.input {
                RuntimeBlurInput::Rgba => GraphScheduledEncodingKind::BlurVerticalRgba,
                RuntimeBlurInput::SourceAlpha => {
                    GraphScheduledEncodingKind::BlurVerticalSourceAlpha
                }
            },
            RuntimePassKind::DropShadowColorize(Some(_)) => {
                GraphScheduledEncodingKind::DropShadowColorize
            }
            RuntimePassKind::Composite(Some(RuntimeComposite {
                kind: RuntimeCompositeKind::SpanSourceOver,
                ..
            })) => GraphScheduledEncodingKind::SpanSourceOver,
            RuntimePassKind::Composite(Some(RuntimeComposite {
                kind: RuntimeCompositeKind::DropShadow,
                ..
            })) => GraphScheduledEncodingKind::DropShadowMerge,
            RuntimePassKind::Composite(Some(RuntimeComposite {
                kind: RuntimeCompositeKind::Layer { .. },
                ..
            })) => GraphScheduledEncodingKind::LayerComposite,
            RuntimePassKind::Present => GraphScheduledEncodingKind::Present,
            RuntimePassKind::VelloCapture(None)
            | RuntimePassKind::ColorFilter(None)
            | RuntimePassKind::BlurHorizontal(None)
            | RuntimePassKind::BlurVertical(None)
            | RuntimePassKind::DropShadowColorize(None)
            | RuntimePassKind::Composite(None) => return Ok(false),
        };
        if *scheduled != expected {
            return Ok(false);
        }
    }
    Ok(true)
}

fn vello_capture_handoff_is_bounded(handoff: &VelloCaptureEncodingHandoff<'_>) -> bool {
    handoff.has_bounded_work()
        && handoff.target_extent().width() > 0
        && handoff.target_extent().height() > 0
        && handoff.texture().width() == handoff.target_extent().width()
        && handoff.texture().height() == handoff.target_extent().height()
        && handoff.texture().depth_or_array_layers() == 1
        && handoff.texture().mip_level_count() == 1
        && handoff.texture().sample_count() == 1
        && handoff.raster_scale().is_finite()
        && handoff.raster_scale() > 0.0
        && handoff
            .initial_transform()
            .as_array()
            .iter()
            .all(|value| value.is_finite())
}

fn core_pass_spatial_uniform_preserves_source_origin(
    bytes: &PassSpatialUniformBytes,
    source: RuntimeSpatialDescriptor,
) -> bool {
    let encoded_x = f32::from_le_bytes(bytes.as_bytes()[0..4].try_into().unwrap_or([0; 4]));
    let encoded_y = f32::from_le_bytes(bytes.as_bytes()[4..8].try_into().unwrap_or([0; 4]));
    encoded_x == source.texel_origin.x() as f32 && encoded_y == source.texel_origin.y() as f32
}

fn close_f64(left: f64, right: f64) -> bool {
    let tolerance = f64::EPSILON * left.abs().max(right.abs()).max(1.0) * 8.0;
    (left - right).abs() <= tolerance
}

fn validate_color_filter_render_region(
    spatial: &PassSpatialUniformBytes,
    source: RuntimeSpatialDescriptor,
    target: RuntimeSpatialDescriptor,
) -> Result<(GraphRenderRegion, bool, bool)> {
    let region = GraphRenderRegion::bounded_source(source, target)?.ok_or_else(|| {
        preparation_error("the color-filter color pass has an empty bounded region")
    })?;
    let viewport_and_scissor = region.scissor_x.saturating_add(region.scissor_width)
        <= target.device_extent.width()
        && region.scissor_y.saturating_add(region.scissor_height) <= target.device_extent.height()
        && region.viewport_width > 0.0
        && region.viewport_height > 0.0;
    let signed_texel_mapping = core_pass_spatial_uniform_preserves_source_origin(spatial, source)
        && close_f64(region.unclipped_x, 0.0)
        && close_f64(region.unclipped_y, 0.0)
        && source.texel_origin == target.texel_origin
        && source.device_origin == target.device_origin;
    if !viewport_and_scissor || !signed_texel_mapping {
        return Err(preparation_error(
            "the color-filter bounded viewport or signed texel mapping changed after validation",
        ));
    }
    Ok((region, viewport_and_scissor, signed_texel_mapping))
}

fn encode_layer_composite_region(
    encoder: &mut wgpu::CommandEncoder,
    target: &PreparedTextureBinding<'_>,
    objects: &ProvisionalCompositePassObjects<'_>,
    bind_group: &wgpu::BindGroup,
    region: Option<GraphRenderRegion>,
    target_spatial: RuntimeSpatialDescriptor,
    account: &Account,
) -> Result<()> {
    let Some(region) = region else {
        return Ok(());
    };
    if region.scissor_x.saturating_add(region.scissor_width) > target_spatial.device_extent.width()
        || region.scissor_y.saturating_add(region.scissor_height)
            > target_spatial.device_extent.height()
    {
        return Err(preparation_error(
            "the composition bounded composite exceeds its exact parent extent",
        ));
    }
    account.work(RenderWork::CompositePasses, 1)?;
    let mut pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
        label: Some("Surgeist composition bounded layer composite"),
        color_attachments: &[Some(wgpu::RenderPassColorAttachment {
            view: target.view(),
            resolve_target: None,
            ops: wgpu::Operations {
                load: wgpu::LoadOp::Load,
                store: wgpu::StoreOp::Store,
            },
            depth_slice: None,
        })],
        depth_stencil_attachment: None,
        occlusion_query_set: None,
        timestamp_writes: None,
        multiview_mask: None,
    });
    pass.set_pipeline(objects.render_pipeline());
    pass.set_bind_group(0, bind_group, &[]);
    pass.set_viewport(
        region.viewport_x,
        region.viewport_y,
        region.viewport_width,
        region.viewport_height,
        0.0,
        1.0,
    );
    pass.set_scissor_rect(
        region.scissor_x,
        region.scissor_y,
        region.scissor_width,
        region.scissor_height,
    );
    pass.draw(0..3, 0..1);
    Ok(())
}

fn copy_layer_composite_parent(
    encoder: &mut wgpu::CommandEncoder,
    parent: &PreparedTextureBinding<'_>,
    target: &PreparedTextureBinding<'_>,
    extent: wgpu::Extent3d,
    account: &Account,
) -> Result<()> {
    account.work(RenderWork::CopyOperations, 1)?;
    encoder.copy_texture_to_texture(
        wgpu::TexelCopyTextureInfo {
            texture: parent.texture(),
            mip_level: 0,
            origin: wgpu::Origin3d::ZERO,
            aspect: wgpu::TextureAspect::All,
        },
        wgpu::TexelCopyTextureInfo {
            texture: target.texture(),
            mip_level: 0,
            origin: wgpu::Origin3d::ZERO,
            aspect: wgpu::TextureAspect::All,
        },
        extent,
    );
    Ok(())
}

fn layer_composite_copy_extent(spatial: RuntimeSpatialDescriptor) -> wgpu::Extent3d {
    wgpu::Extent3d {
        width: spatial.device_extent.width(),
        height: spatial.device_extent.height(),
        depth_or_array_layers: 1,
    }
}

fn layer_composite_region_mapping(
    spatial: &PassSpatialUniformBytes,
    source: RuntimeSpatialDescriptor,
    target: RuntimeSpatialDescriptor,
    transform: Transform,
) -> Result<(Option<GraphRenderRegion>, bool)> {
    let (region, transformed_source_bounds) =
        GraphRenderRegion::bounded_transformed_source(source, target, transform)?;
    let preserved = core_pass_spatial_uniform_preserves_source_origin(spatial, source)
        && region.is_none_or(|region| {
            close_f64(region.unclipped_x, transformed_source_bounds[0])
                && close_f64(region.unclipped_y, transformed_source_bounds[1])
        });
    Ok((region, preserved))
}

/// One allocation-backed, generation-bound prepared-graph handoff. Its lifetime prevents
/// the ready device bundle from transitioning while graph encoding owns its frame scope.
fn validate_vello_capture_target(handoff: &VelloCaptureEncodingHandoff<'_>) -> Result<()> {
    let target_extent = handoff.target_extent();
    if target_extent.width() == 0
        || target_extent.height() == 0
        || handoff.texture().width() != target_extent.width()
        || handoff.texture().height() != target_extent.height()
        || handoff.texture().depth_or_array_layers() != 1
        || handoff.texture().mip_level_count() != 1
        || handoff.texture().sample_count() != 1
        || handoff.texture().dimension() != wgpu::TextureDimension::D2
        || handoff.texture().format() != wgpu::TextureFormat::Rgba8Unorm
        || handoff.texture().usage() != VELLO_CAPTURE_TEXTURE_USAGES
    {
        return Err(preparation_error(
            "the graph Vello capture target changed its exact RGBA8 storage contract",
        ));
    }
    Ok(())
}

fn vello_capture_scene(
    handoff: &VelloCaptureEncodingHandoff<'_>,
    account: &Account,
) -> Result<VelloScene> {
    let initial_transform = handoff.initial_transform();
    match handoff.work() {
        RuntimeVelloCapture::Span(span) => encode_vello_scene_with_initial_transform(
            &span.commands,
            initial_transform,
            handoff.target_extent(),
            account,
        ),
        RuntimeVelloCapture::ClipCoverage(coverage) => {
            let mut elements = account.temporary(coverage.elements.len())?;
            for element in coverage.elements.iter() {
                account.work(RenderWork::GraphEdgeVisits, 1)?;
                elements
                    .append_slot()?
                    .write((element.clip.clone(), element.transform));
            }
            encode_vello_clip_coverage_scene(
                &elements,
                initial_transform,
                handoff.target_extent(),
                account,
            )
        }
    }
}

impl<'device> PreparedGraph<'device> {
    pub(crate) fn with_vello_engine(mut self, engine: &'device VelloEngineState) -> Self {
        self.vello_engine = Some(engine);
        self
    }
    pub(crate) async fn encode_custom_spine(
        &mut self,
        encoder: &mut wgpu::CommandEncoder,
        output: GraphExternalOutputView<'_>,
        account: &Account,
    ) -> Result<PendingGraphEncoding> {
        let expected_capture_count = self.custom_spine_requirements(&output, account)?;
        let engine = self.vello_engine.ok_or_else(|| {
            preparation_error("the Vello capture scheduler has no ready internal Vello engine")
        })?;
        let resources = self.resources;
        let queue = self.queue;

        let session = Arc::new(());
        let mut scope = ActiveVelloEncodingScope::begin(self.device);
        let mut leases = VelloResourceLeaseAggregate::new(account, expected_capture_count)?;
        self.custom_spine_encoding_state = Some(CustomSpineEncodingState::Encoding);
        let encode_account = account.at(RenderPhase::Encode);
        let result = account.phase(RenderPhase::Encode, || {
            let mut capture_encoding = VelloCaptureEncodingContext {
                engine,
                resources,
                queue,
                scope: &mut scope,
                leases: &mut leases,
                session: &session,
            };
            self.encode_custom_spine_once(
                encoder,
                &output,
                expected_capture_count,
                &mut capture_encoding,
                &encode_account,
            )
        });
        let summary = match result {
            Ok(summary) => summary,
            Err(encoding_error) => {
                let _ = leases.abort();
                let scope_result = scope.finish(account).await;
                self.custom_spine_encoding_state = Some(CustomSpineEncodingState::AbortOnly);
                return match scope_result {
                    Ok(()) => Err(encoding_error),
                    Err(scope_error) => Err(scope_error),
                };
            }
        };
        let leases = match scope.finish_with_leases(leases, account).await {
            Ok(leases) => leases,
            Err(failure) => {
                self.custom_spine_encoding_state = Some(CustomSpineEncodingState::AbortOnly);
                return Err(failure.into_error_and_aborted_resources().0);
            }
        };
        self.custom_spine_encoding_state = Some(CustomSpineEncodingState::Complete);
        self.custom_spine_completed_session = Some(Arc::clone(&session));
        Ok(PendingGraphEncoding {
            summary,
            resources: PendingVelloResourceCommit::from_aggregate(leases),
            session,
        })
    }

    fn custom_spine_requirements(
        &self,
        output: &GraphExternalOutputView<'_>,
        account: &Account,
    ) -> Result<usize> {
        match self.custom_spine_encoding_state {
            Some(CustomSpineEncodingState::Ready) => {}
            Some(
                CustomSpineEncodingState::Encoding
                | CustomSpineEncodingState::Complete
                | CustomSpineEncodingState::AbortOnly,
            ) => {
                return Err(preparation_error(
                    "the custom-spine encoding is one-shot; discard this prepared graph and its encoder",
                ));
            }
            None => {
                return Err(preparation_error(
                    "the custom-spine scheduler requires validated execution facts",
                ));
            }
        }
        let (execution_working_format, execution_output_format, expected_capture_count) =
            if let Some(execution) = self.base_execution.as_ref() {
                (
                    execution.working_format(),
                    execution.output_format(),
                    execution.captures().len(),
                )
            } else if let Some(execution) = self.composition_execution.as_ref() {
                (
                    execution.working_format,
                    execution.output_format,
                    execution.captures.len(),
                )
            } else if let Some(execution) = self.color_filter_execution.as_ref() {
                (
                    execution.working_format,
                    execution.output_format,
                    execution.captures.len(),
                )
            } else if let Some(execution) = self.spatial_filter_execution.as_ref() {
                (
                    execution.working_format,
                    execution.output_format,
                    execution.captures.len(),
                )
            } else if let Some(execution) = self.backdrop_execution.as_ref() {
                (
                    execution.working_format,
                    execution.output_format,
                    execution.captures.len(),
                )
            } else {
                return Err(preparation_error(
                    "the custom-spine scheduler requires validated execution facts",
                ));
            };
        if execution_working_format != self.plan.working_format
            || execution_output_format != self.plan.output_format
            || output.format != self.plan.output_format
            || output.extent != self.output_extent(account)?
        {
            return Err(preparation_error(
                "the graph external output differs from the exact prepared format or extent",
            ));
        }
        if self.pass_cache_update.is_none() {
            return Err(preparation_error(
                "the custom-spine scheduler requires transaction-provisional pass objects",
            ));
        }
        if self.next_pass != 0 {
            return Err(preparation_error(
                "the custom-spine scheduler requires an unstarted validated graph",
            ));
        }
        Ok(expected_capture_count)
    }

    fn encode_custom_spine_once(
        &mut self,
        encoder: &mut wgpu::CommandEncoder,
        output: &GraphExternalOutputView<'_>,
        expected_capture_count: usize,
        capture_encoding: &mut VelloCaptureEncodingContext<'_, '_>,
        account: &Account,
    ) -> Result<CustomSpineEncodingSummary> {
        let mut progress = CustomSpineEncodingProgress::new(
            self.plan.passes.len(),
            expected_capture_count,
            account,
        )?;
        while let Some(request) = self
            .plan
            .passes
            .get(self.next_pass)
            .map(PreparedPassEncodingRequest::from)
        {
            account.work(RenderWork::GraphNodeVisits, 1)?;
            self.encode_custom_spine_request(
                encoder,
                output,
                capture_encoding,
                &request,
                &mut progress,
                account,
            )?;
        }
        progress.finish(self, account)
    }

    fn encode_custom_spine_request(
        &mut self,
        encoder: &mut wgpu::CommandEncoder,
        output: &GraphExternalOutputView<'_>,
        capture_encoding: &mut VelloCaptureEncodingContext<'_, '_>,
        request: &PreparedPassEncodingRequest,
        progress: &mut CustomSpineEncodingProgress,
        account: &Account,
    ) -> Result<()> {
        match &request.kind {
            RuntimePassKind::ClearRoot { initialization, .. } => {
                self.encode_clear_root_step(encoder, request, *initialization, progress, account)
            }
            RuntimePassKind::VelloCapture(Some(_)) => self.encode_vello_capture_step(
                encoder,
                request,
                capture_encoding,
                progress,
                account,
            ),
            RuntimePassKind::CanonicalizeCapture => {
                self.encode_canonicalize_capture_step(encoder, request, progress, account)
            }
            RuntimePassKind::CopyBackdrop => {
                self.encode_copy_backdrop_step(encoder, request, progress, account)
            }
            RuntimePassKind::ColorFilter(Some(_)) => {
                self.encode_color_filter_step(encoder, request, progress, account)
            }
            RuntimePassKind::Offset(_) => {
                self.encode_offset(encoder, request, account)?;
                let released = self
                    .complete_custom_spine_pass(request.id, account)?
                    .prove_for(request);
                progress.offset_parameters_and_resources_are_exact &=
                    released.resources && released.offset_buffer;
                progress.offset_pass_count = progress.offset_pass_count.saturating_add(1);
                progress.record_custom_completion(GraphScheduledEncodingKind::Offset)
            }
            RuntimePassKind::Convolution(_) => {
                self.encode_convolution(encoder, request, account)?;
                self.complete_custom_spine_pass(request.id, account)?
                    .prove_for(request);
                progress.convolution_pass_count = progress.convolution_pass_count.saturating_add(1);
                progress.record_custom_completion(GraphScheduledEncodingKind::Convolution)
            }
            RuntimePassKind::BlurHorizontal(Some(_)) | RuntimePassKind::BlurVertical(Some(_)) => {
                self.encode_blur_step(encoder, request, progress, account)
            }
            RuntimePassKind::DropShadowColorize(Some(_)) => {
                self.encode_drop_shadow_colorize_step(encoder, request, progress, account)
            }
            RuntimePassKind::Composite(Some(composite))
                if matches!(composite.kind, RuntimeCompositeKind::DropShadow) =>
            {
                self.encode_drop_shadow_merge_step(encoder, request, progress, account)
            }
            RuntimePassKind::Composite(Some(composite))
                if matches!(composite.kind, RuntimeCompositeKind::SpanSourceOver) =>
            {
                self.encode_span_source_over_step(encoder, request, progress, account)
            }
            RuntimePassKind::Composite(Some(composite))
                if matches!(composite.kind, RuntimeCompositeKind::Layer { .. }) =>
            {
                self.encode_composition_layer_step(encoder, request, progress, account)
            }
            RuntimePassKind::Present => {
                self.encode_base_graph_present_step(encoder, output, request, progress, account)
            }
            RuntimePassKind::VelloCapture(None)
            | RuntimePassKind::ColorFilter(None)
            | RuntimePassKind::BlurHorizontal(None)
            | RuntimePassKind::BlurVertical(None)
            | RuntimePassKind::DropShadowColorize(None)
            | RuntimePassKind::Composite(_) => Err(preparation_error(
                "a non-core pass reached the custom graph spine scheduler",
            )),
        }
    }

    fn encode_clear_root_step(
        &mut self,
        encoder: &mut wgpu::CommandEncoder,
        request: &PreparedPassEncodingRequest,
        initialization: RuntimeInitialization,
        progress: &mut CustomSpineEncodingProgress,
        account: &Account,
    ) -> Result<()> {
        let facts = self.encode_clear_root(encoder, request, account)?;
        if initialization == RuntimeInitialization::SurfaceBaseColor {
            progress.root_clear_count = progress.root_clear_count.saturating_add(1);
            progress.clears_full_root &= facts.full_target;
        }
        self.complete_custom_spine_pass(request.id, account)
            .map(drop)?;
        progress.record_custom_completion(GraphScheduledEncodingKind::ClearRoot)?;
        Ok(())
    }

    fn encode_vello_capture_step(
        &mut self,
        encoder: &mut wgpu::CommandEncoder,
        request: &PreparedPassEncodingRequest,
        capture_encoding: &mut VelloCaptureEncodingContext<'_, '_>,
        progress: &mut CustomSpineEncodingProgress,
        account: &Account,
    ) -> Result<()> {
        let handoff = self.vello_capture_handoff(request, capture_encoding.session, account)?;
        let target = handoff.target();
        progress.bounded_capture_handoffs &= vello_capture_handoff_is_bounded(&handoff);
        let encoded = Self::encode_vello_capture(handoff, encoder, capture_encoding, account)?;
        #[cfg(test)]
        progress.capture_observations.push(encoded.observation);
        self.complete_vello_capture(
            request.id,
            target,
            capture_encoding.session,
            encoded.receipt,
            account,
        )?;
        #[cfg(test)]
        {
            self.acquired_capture_lease_count_raw_fact =
                self.acquired_capture_lease_count_raw_fact.saturating_add(1);
        }
        progress.record_capture_completion()?;
        Ok(())
    }

    fn encode_canonicalize_capture_step(
        &mut self,
        encoder: &mut wgpu::CommandEncoder,
        request: &PreparedPassEncodingRequest,
        progress: &mut CustomSpineEncodingProgress,
        account: &Account,
    ) -> Result<()> {
        let facts = self.encode_canonicalize_capture(encoder, request, account)?;
        progress.exact_spatial &= facts.exact_spatial_uniform;
        self.complete_custom_spine_pass(request.id, account)
            .map(drop)?;
        progress.record_custom_completion(GraphScheduledEncodingKind::CanonicalizeCapture)?;
        Ok(())
    }

    fn encode_color_filter_step(
        &mut self,
        encoder: &mut wgpu::CommandEncoder,
        request: &PreparedPassEncodingRequest,
        progress: &mut CustomSpineEncodingProgress,
        account: &Account,
    ) -> Result<()> {
        let facts = self.encode_color_filter(encoder, request, account)?;
        progress.color_filter_count = progress.color_filter_count.saturating_add(1);
        progress.color_filters_preserve_authored_order &= facts.exact_operation_bytes;
        progress.color_filter_bindings_are_exact &= facts.exact_source_spatial_and_operations;
        progress.color_filter_sources_and_results_are_distinct &=
            facts.source_and_result_are_distinct;
        progress.color_filter_regions_are_validated &= facts.validated_viewport_and_scissor;
        progress.color_filter_signed_texel_mapping_is_exact &= facts.preserved_signed_texel_mapping;
        let released = self
            .complete_custom_spine_pass(request.id, account)?
            .prove_for(request);
        progress.color_filter_operation_buffers_released &= released.color_buffer;
        progress.record_custom_completion(GraphScheduledEncodingKind::ColorFilter)?;
        Ok(())
    }

    fn encode_copy_backdrop_step(
        &mut self,
        encoder: &mut wgpu::CommandEncoder,
        request: &PreparedPassEncodingRequest,
        progress: &mut CustomSpineEncodingProgress,
        account: &Account,
    ) -> Result<()> {
        let facts = self.encode_copy_backdrop(encoder, request, account)?;
        progress.copy_backdrop_count = progress.copy_backdrop_count.saturating_add(1);
        progress.copy_backdrop_bindings_are_exact &= facts.exact_prepared_bindings;
        progress.copy_backdrop_allocations_are_distinct &= facts.source_and_result_are_distinct;
        progress.copy_backdrop_regions_are_validated &= facts.validated_viewport_and_scissor;
        progress.copy_backdrop_signed_mapping_is_exact &= facts.preserved_signed_mapping;
        #[cfg(test)]
        progress
            .composite_encoder_identities
            .push(std::ptr::from_mut(&mut *encoder) as usize);
        let released = self
            .complete_custom_spine_pass(request.id, account)?
            .prove_for(request);
        progress.spatial_filter_textures_released &= released.resources;
        progress.record_custom_completion(GraphScheduledEncodingKind::CopyBackdrop)?;
        Ok(())
    }

    fn encode_blur_step(
        &mut self,
        encoder: &mut wgpu::CommandEncoder,
        request: &PreparedPassEncodingRequest,
        progress: &mut CustomSpineEncodingProgress,
        account: &Account,
    ) -> Result<()> {
        let facts = self.encode_blur(encoder, request, account)?;
        let kind = match &request.kind {
            RuntimePassKind::BlurHorizontal(Some(blur)) => match blur.input {
                RuntimeBlurInput::Rgba => GraphScheduledEncodingKind::BlurHorizontalRgba,
                RuntimeBlurInput::SourceAlpha => {
                    GraphScheduledEncodingKind::BlurHorizontalSourceAlpha
                }
            },
            RuntimePassKind::BlurVertical(Some(blur)) => match blur.input {
                RuntimeBlurInput::Rgba => GraphScheduledEncodingKind::BlurVerticalRgba,
                RuntimeBlurInput::SourceAlpha => {
                    GraphScheduledEncodingKind::BlurVerticalSourceAlpha
                }
            },
            _ => {
                return Err(preparation_error(
                    "the spatial-filter blur scheduler lost its pass kind",
                ));
            }
        };
        #[cfg(test)]
        progress
            .composite_encoder_identities
            .push(std::ptr::from_mut(&mut *encoder) as usize);
        let released = self
            .complete_custom_spine_pass(request.id, account)?
            .prove_for(request);
        progress.blur_pass_count = progress.blur_pass_count.saturating_add(1);
        progress.spatial_filter_bindings_are_exact &= facts.exact_prepared_bindings;
        progress.spatial_filter_regions_are_exact &=
            facts.validated_region && facts.preserved_signed_mapping;
        progress.blur_allocations_are_distinct &= facts.distinct_source_and_result;
        progress.spatial_filter_kernels_released &= released.kernels;
        progress.spatial_filter_textures_released &= released.resources;
        progress.record_custom_completion(kind)?;
        Ok(())
    }

    fn encode_drop_shadow_colorize_step(
        &mut self,
        encoder: &mut wgpu::CommandEncoder,
        request: &PreparedPassEncodingRequest,
        progress: &mut CustomSpineEncodingProgress,
        account: &Account,
    ) -> Result<()> {
        let facts = self.encode_drop_shadow_colorize(encoder, request, account)?;
        #[cfg(test)]
        progress
            .composite_encoder_identities
            .push(std::ptr::from_mut(&mut *encoder) as usize);
        let released = self
            .complete_custom_spine_pass(request.id, account)?
            .prove_for(request);
        progress.drop_shadow_colorize_count = progress.drop_shadow_colorize_count.saturating_add(1);
        progress.spatial_filter_bindings_are_exact &= facts.exact_prepared_bindings;
        progress.spatial_filter_regions_are_exact &=
            facts.validated_region && facts.preserved_signed_mapping;
        progress.blur_allocations_are_distinct &= facts.distinct_source_and_result;
        progress.spatial_filter_textures_released &= released.resources;
        progress.record_custom_completion(GraphScheduledEncodingKind::DropShadowColorize)?;
        Ok(())
    }

    fn encode_drop_shadow_merge_step(
        &mut self,
        encoder: &mut wgpu::CommandEncoder,
        request: &PreparedPassEncodingRequest,
        progress: &mut CustomSpineEncodingProgress,
        account: &Account,
    ) -> Result<()> {
        let facts = self.encode_drop_shadow_merge(encoder, request, account)?;
        let source =
            exact_core_pass_read(request, RuntimeReadRole::CompositeSource, account)?.resource();
        let source_is_live = self
            .resource_bindings
            .get(&source, account)?
            .is_some_and(|binding| binding.lease.is_some());
        let source_expected_reads = self.resource_request(source, account)?.expected_reads;
        let mut source_read_twice = false;
        'shadow_facts: for execution in [&self.spatial_filter_execution, &self.backdrop_execution]
            .into_iter()
            .flatten()
        {
            for shadow in execution.drop_shadows.iter() {
                account.work(RenderWork::GraphNodeVisits, 1)?;
                if shadow.merge == request.id
                    && shadow.source == source
                    && source_expected_reads == 2
                {
                    source_read_twice = true;
                    break 'shadow_facts;
                }
            }
        }
        #[cfg(test)]
        progress
            .composite_encoder_identities
            .push(std::ptr::from_mut(&mut *encoder) as usize);
        let mut source_is_scheduled_for_release = false;
        for resource in request.releases.iter() {
            account.work(RenderWork::GraphEdgeVisits, 1)?;
            if *resource == source {
                source_is_scheduled_for_release = true;
                break;
            }
        }
        let released = self
            .complete_custom_spine_pass(request.id, account)?
            .prove_for(request);
        progress.drop_shadow_merge_count = progress.drop_shadow_merge_count.saturating_add(1);
        progress.source_over_count = progress.source_over_count.saturating_add(1);
        progress.spatial_filter_bindings_are_exact &= facts.exact_prepared_bindings;
        progress.spatial_filter_regions_are_exact &=
            facts.validated_region && facts.preserved_signed_mapping;
        progress.parent_and_result_are_distinct &= facts.distinct_source_shadow_and_result;
        progress.full_copy_before_bounded_render &= facts.distinct_source_shadow_and_result;
        progress.samples_source_with_fixed_blend &= facts.fixed_source_over_blend;
        progress.preserves_signed_origin &= facts.preserved_signed_mapping;
        progress.shadow_source_read_twice &=
            facts.reads_original_source_and_shadow && source_read_twice;
        progress.shadow_source_released_after_merge &=
            source_is_live && source_is_scheduled_for_release && released.resources;
        progress.spatial_filter_textures_released &= released.resources;
        progress.record_custom_completion(GraphScheduledEncodingKind::DropShadowMerge)?;
        Ok(())
    }

    fn encode_span_source_over_step(
        &mut self,
        encoder: &mut wgpu::CommandEncoder,
        request: &PreparedPassEncodingRequest,
        progress: &mut CustomSpineEncodingProgress,
        account: &Account,
    ) -> Result<()> {
        let facts = self.encode_span_source_over(encoder, request, account)?;
        progress.source_over_count = progress.source_over_count.saturating_add(1);
        progress.exact_spatial &= facts.exact_spatial_uniform;
        progress.parent_and_result_are_distinct &= facts.parent_and_result_distinct;
        progress.full_copy_before_bounded_render &= facts.copied_full_parent_before_render;
        progress.samples_source_with_fixed_blend &=
            facts.sampled_only_source && facts.fixed_source_over_blend;
        progress.preserves_signed_origin &= facts.preserved_signed_source_origin;
        self.complete_custom_spine_pass(request.id, account)
            .map(drop)?;
        progress.record_custom_completion(GraphScheduledEncodingKind::SpanSourceOver)?;
        Ok(())
    }

    fn encode_composition_layer_step(
        &mut self,
        encoder: &mut wgpu::CommandEncoder,
        request: &PreparedPassEncodingRequest,
        progress: &mut CustomSpineEncodingProgress,
        account: &Account,
    ) -> Result<()> {
        let facts = self.encode_composition_layer_composite(encoder, request, account)?;
        progress.layer_composite_count = progress.layer_composite_count.saturating_add(1);
        progress.normal_composite_count = progress
            .normal_composite_count
            .saturating_add(usize::from(facts.normal_path));
        progress.destination_composite_count = progress
            .destination_composite_count
            .saturating_add(usize::from(facts.destination_path));
        progress.parent_and_result_are_distinct &= facts.avoids_read_write_alias;
        progress.full_copy_before_bounded_render &= facts.copied_full_parent;
        if facts.normal_path {
            progress.samples_source_with_fixed_blend &=
                facts.fixed_premultiplied_blend && facts.omits_parent_sample;
            progress.normal_fixed_blend &= facts.fixed_premultiplied_blend;
            progress.normal_omits_parent_sample &= facts.omits_parent_sample;
        }
        if facts.destination_path {
            progress.destination_copies_full_parent &= facts.copied_full_parent;
            progress.destination_avoids_alias &= facts.avoids_read_write_alias;
        }
        progress.layer_bindings_are_exact &= facts.exact_resources_and_parameters;
        progress.layer_signed_mapping_is_exact &= facts.preserved_signed_mapping;
        progress.preserves_signed_origin &= facts.preserved_signed_mapping;
        #[cfg(test)]
        progress
            .composite_encoder_identities
            .push(facts.encoder_identity);
        self.complete_custom_spine_pass(request.id, account)
            .map(drop)?;
        progress.record_custom_completion(GraphScheduledEncodingKind::LayerComposite)?;
        Ok(())
    }

    fn encode_base_graph_present_step(
        &mut self,
        encoder: &mut wgpu::CommandEncoder,
        output: &GraphExternalOutputView<'_>,
        request: &PreparedPassEncodingRequest,
        progress: &mut CustomSpineEncodingProgress,
        account: &Account,
    ) -> Result<()> {
        let facts = self.encode_base_graph_present(encoder, request, output, account)?;
        progress.exact_spatial &= facts.exact_spatial_uniform;
        progress.exact_external_output |= facts.external_output_exact;
        self.complete_custom_spine_pass(request.id, account)
            .map(drop)?;
        progress.record_custom_completion(GraphScheduledEncodingKind::Present)?;
        Ok(())
    }

    fn encode_vello_capture(
        handoff: VelloCaptureEncodingHandoff<'_>,
        encoder: &mut wgpu::CommandEncoder,
        capture_encoding: &mut VelloCaptureEncodingContext<'_, '_>,
        account: &Account,
    ) -> Result<EncodedVelloCaptureResult> {
        let target_extent = handoff.target_extent();
        let antialiasing = handoff.antialiasing();
        validate_vello_capture_target(&handoff)?;
        let scene = vello_capture_scene(&handoff, account)?;
        #[cfg(test)]
        let lowers_with_exact_initial_transform = match handoff.work() {
            RuntimeVelloCapture::Span(_) => scene
                .observation_for_test()
                .first_glyph_run_for_test()
                .is_some_and(|run| {
                    run.transform_components_for_test()
                        .iter()
                        .zip(handoff.initial_transform().as_array())
                        .all(|(actual, expected)| (*actual - expected as f32).abs() <= 1.0e-5)
                }),
            RuntimeVelloCapture::ClipCoverage(_) => true,
        };
        let prepared = scene.prepare_raster(
            vello_capture_raster_parameters(target_extent, antialiasing)?,
            account,
        )?;
        #[cfg(test)]
        let encoder_identity = std::ptr::from_mut(&mut *encoder) as usize;
        #[cfg(test)]
        let scope_identity = std::ptr::from_ref(&*capture_encoding.scope) as usize;
        #[cfg(test)]
        let target_view_identity = std::ptr::from_ref(handoff.view()) as usize;
        let encoded = {
            let mut encoding = TransactionEncodingState::new_reusable_graph_capture(
                capture_encoding.scope,
                capture_encoding.queue,
                encoder,
                handoff.view(),
                TransactionTargetIntent::new(
                    target_extent,
                    wgpu::TextureFormat::Rgba8Unorm,
                    VELLO_CAPTURE_TEXTURE_USAGES,
                ),
                account,
            );
            prepared.encode_capture_into(
                capture_encoding.engine,
                capture_encoding.resources,
                &mut encoding,
            )
        };
        let encoded = match encoded {
            Ok(encoded) => encoded,
            Err(failure) => return Err(failure.into_error_and_aborted_resources().0),
        };
        let (lease, proof) = encoded.into_resources_and_proof();
        #[cfg(test)]
        let observation = EncodedCaptureRawFact {
            lowers_with_exact_initial_transform,
            uses_transparent_base: proof.transparent_base_for_test(),
            antialiasing: proof.antialiasing_for_test(),
            target_extent: proof.target_extent_for_test(),
            target_format: proof.target_format_for_test(),
            target_usage: proof.target_usage_for_test(),
            target_and_view_are_exact: handoff.texture().format() == proof.target_format_for_test()
                && handoff.texture().width() == proof.target_extent_for_test().width()
                && handoff.texture().height() == proof.target_extent_for_test().height()
                && proof.target_view_identity_for_test() == target_view_identity,
            encoder_identity,
            scope_identity,
        };
        let receipt = match handoff.complete_after_encoded_capture(proof) {
            Ok(receipt) => receipt,
            Err(error) => {
                let _ = lease.abort();
                return Err(error);
            }
        };
        capture_encoding.leases.push(lease)?;
        Ok(EncodedVelloCaptureResult {
            receipt,
            #[cfg(test)]
            observation,
        })
    }

    fn validate_texture_binding(
        &self,
        binding: &PreparedTextureBinding<'_>,
        resource: RuntimeResourceId,
        account: &Account,
    ) -> Result<RuntimeSpatialDescriptor> {
        let request = self.resource_request(resource, account)?;
        let texture = binding.texture();
        let expected_format = match request.format {
            RuntimeResourceFormat::VelloCaptureRgba8Unorm
            | RuntimeResourceFormat::ClipCoverageRgba8Unorm
            | RuntimeResourceFormat::ResolvedMaskRgba8Unorm => wgpu::TextureFormat::Rgba8Unorm,
            RuntimeResourceFormat::Working(format) => format.texture_format(),
            RuntimeResourceFormat::FilterRgba32Float => wgpu::TextureFormat::Rgba32Float,
        };
        if binding.runtime_resource() != resource
            || texture.width() != request.spatial.device_extent.width()
            || texture.height() != request.spatial.device_extent.height()
            || texture.depth_or_array_layers() != 1
            || texture.mip_level_count() != 1
            || texture.sample_count() != 1
            || texture.dimension() != wgpu::TextureDimension::D2
            || texture.format() != expected_format
        {
            return Err(preparation_error(
                "the prepared graph texture differs from its exact runtime binding",
            ));
        }
        Ok(request.spatial)
    }

    fn core_pass_objects<'prepared>(
        &'prepared self,
        keys: &RuntimePassCacheKeys,
        account: &Account,
    ) -> Result<ProvisionalCorePassObjects<'prepared>> {
        self.pass_cache_update
            .as_ref()
            .ok_or_else(|| preparation_error("core-pass provisional pass objects are unavailable"))?
            .encoding_objects(
                self.pass_cache,
                account,
                keys.samplers(),
                keys.layout(),
                keys.shader(),
                keys.pipeline(),
            )
    }

    fn layer_composite_pass_objects<'prepared>(
        &'prepared self,
        keys: &RuntimePassCacheKeys,
        account: &Account,
    ) -> Result<ProvisionalCompositePassObjects<'prepared>> {
        self.pass_cache_update
            .as_ref()
            .ok_or_else(|| {
                preparation_error("composition provisional pass objects are unavailable")
            })?
            .composite_encoding_objects(
                self.pass_cache,
                account,
                keys.samplers(),
                keys.layout(),
                keys.shader(),
                keys.pipeline(),
            )
    }

    fn color_filter_pass_objects<'prepared>(
        &'prepared self,
        keys: &RuntimePassCacheKeys,
        account: &Account,
    ) -> Result<ProvisionalColorFilterPassObjects<'prepared>> {
        self.pass_cache_update
            .as_ref()
            .ok_or_else(|| {
                preparation_error("color-filter provisional pass objects are unavailable")
            })?
            .color_filter_encoding_objects(
                self.pass_cache,
                account,
                keys.samplers(),
                keys.layout(),
                keys.shader(),
                keys.pipeline(),
            )
    }

    fn copy_backdrop_pass_objects(
        &self,
        pass: RuntimePassId,
        account: &Account,
    ) -> Result<(
        &wgpu::Sampler,
        &wgpu::BindGroupLayout,
        &wgpu::RenderPipeline,
    )> {
        match self.spatial_filter_pass_objects.get(&pass, account)? {
            Some(PreparedSpatialFilterPassObjects::CopyBackdrop {
                parent_sampler,
                layout,
                pipeline,
            }) => Ok((parent_sampler, layout, pipeline)),
            Some(
                PreparedSpatialFilterPassObjects::Blur { .. }
                | PreparedSpatialFilterPassObjects::DropShadowColorize { .. },
            )
            | None => Err(preparation_error(
                "the backdrop-copy pass lost its exact prepared object handles",
            )),
        }
    }

    fn blur_pass_objects(
        &self,
        pass: RuntimePassId,
        account: &Account,
    ) -> Result<(
        &wgpu::Sampler,
        &wgpu::BindGroupLayout,
        &wgpu::RenderPipeline,
    )> {
        match self.spatial_filter_pass_objects.get(&pass, account)? {
            Some(PreparedSpatialFilterPassObjects::Blur {
                source_sampler,
                layout,
                pipeline,
            }) => Ok((source_sampler, layout, pipeline)),
            Some(
                PreparedSpatialFilterPassObjects::CopyBackdrop { .. }
                | PreparedSpatialFilterPassObjects::DropShadowColorize { .. },
            )
            | None => Err(preparation_error(
                "the spatial-filter blur pass lost its exact prepared object handles",
            )),
        }
    }

    fn drop_shadow_colorize_pass_objects(
        &self,
        pass: RuntimePassId,
        account: &Account,
    ) -> Result<(
        &wgpu::Sampler,
        &wgpu::BindGroupLayout,
        &wgpu::RenderPipeline,
    )> {
        match self.spatial_filter_pass_objects.get(&pass, account)? {
            Some(PreparedSpatialFilterPassObjects::DropShadowColorize {
                source_sampler,
                layout,
                pipeline,
            }) => Ok((source_sampler, layout, pipeline)),
            Some(
                PreparedSpatialFilterPassObjects::CopyBackdrop { .. }
                | PreparedSpatialFilterPassObjects::Blur { .. },
            )
            | None => Err(preparation_error(
                "the spatial-filter drop-shadow pass lost its exact prepared object handles",
            )),
        }
    }

    fn create_core_pass_spatial_uniform_buffer(
        &self,
        bytes: &PassSpatialUniformBytes,
        account: &Account,
    ) -> Result<OwnedGpu<wgpu::Buffer>> {
        let buffer = self.resources.generation().allocation(
            GenerationClass::PassBuffers,
            bytes.as_bytes().len() as u64,
            Some(account),
            || {
                self.device.create_buffer(&wgpu::BufferDescriptor {
                    label: Some("Surgeist core pass spatial uniform"),
                    size: bytes.as_bytes().len() as u64,
                    usage: wgpu::BufferUsages::UNIFORM.union(wgpu::BufferUsages::COPY_DST),
                    mapped_at_creation: false,
                })
            },
        )?;
        account.work(RenderWork::UploadBytes, bytes.as_bytes().len())?;
        self.queue.write_buffer(&buffer, 0, bytes.as_bytes());
        Ok(buffer)
    }

    fn create_layer_composite_parameter_buffer(
        &self,
        bytes: &CompositeParameterBytes,
        account: &Account,
    ) -> Result<OwnedGpu<wgpu::Buffer>> {
        let buffer = self.resources.generation().allocation(
            GenerationClass::PassBuffers,
            bytes.as_bytes().len() as u64,
            Some(account),
            || {
                self.device.create_buffer(&wgpu::BufferDescriptor {
                    label: Some("Surgeist composition composite parameter uniform"),
                    size: bytes.as_bytes().len() as u64,
                    usage: wgpu::BufferUsages::UNIFORM.union(wgpu::BufferUsages::COPY_DST),
                    mapped_at_creation: false,
                })
            },
        )?;
        account.work(RenderWork::UploadBytes, bytes.as_bytes().len())?;
        self.queue.write_buffer(&buffer, 0, bytes.as_bytes());
        Ok(buffer)
    }

    fn create_drop_shadow_parameter_buffer(
        &self,
        bytes: &DropShadowParameterBytes,
        account: &Account,
    ) -> Result<OwnedGpu<wgpu::Buffer>> {
        let buffer = self.resources.generation().allocation(
            GenerationClass::PassBuffers,
            bytes.as_bytes().len() as u64,
            Some(account),
            || {
                self.device.create_buffer(&wgpu::BufferDescriptor {
                    label: Some("Surgeist spatial-filter drop-shadow parameter uniform"),
                    size: bytes.as_bytes().len() as u64,
                    usage: wgpu::BufferUsages::UNIFORM.union(wgpu::BufferUsages::COPY_DST),
                    mapped_at_creation: false,
                })
            },
        )?;
        account.work(RenderWork::UploadBytes, bytes.as_bytes().len())?;
        self.queue.write_buffer(&buffer, 0, bytes.as_bytes());
        Ok(buffer)
    }

    fn create_backdrop_blur_edge_parameter_buffer(
        &self,
        bytes: &[u8; 16],
        account: &Account,
    ) -> Result<OwnedGpu<wgpu::Buffer>> {
        let buffer = self.resources.generation().allocation(
            GenerationClass::PassBuffers,
            bytes.len() as u64,
            Some(account),
            || {
                self.device.create_buffer(&wgpu::BufferDescriptor {
                    label: Some("Surgeist backdrop semantic backdrop edge uniform"),
                    size: bytes.len() as u64,
                    usage: wgpu::BufferUsages::UNIFORM.union(wgpu::BufferUsages::COPY_DST),
                    mapped_at_creation: false,
                })
            },
        )?;
        account.work(RenderWork::UploadBytes, bytes.len())?;
        self.queue.write_buffer(&buffer, 0, bytes);
        Ok(buffer)
    }

    fn vello_capture_handoff<'prepared>(
        &'prepared self,
        request: &PreparedPassEncodingRequest,
        session: &Arc<()>,
        account: &Account,
    ) -> Result<VelloCaptureEncodingHandoff<'prepared>> {
        let RuntimeResultBinding::Resource(target) = request.result else {
            return Err(preparation_error(
                "the graph Vello capture has no exact prepared target",
            ));
        };
        let capture_families = [
            self.base_execution
                .as_ref()
                .map(|execution| execution.captures()),
            self.composition_execution
                .as_ref()
                .map(|execution| execution.captures.as_slice()),
            self.color_filter_execution
                .as_ref()
                .map(|execution| execution.captures.as_slice()),
            self.spatial_filter_execution
                .as_ref()
                .map(|execution| execution.captures.as_slice()),
            self.backdrop_execution
                .as_ref()
                .map(|execution| execution.captures.as_slice()),
        ];
        let mut capture = None;
        'capture_facts: for captures in capture_families.into_iter().flatten() {
            for candidate in captures {
                account.work(RenderWork::GraphNodeVisits, 1)?;
                if candidate.pass() == request.id && candidate.target() == target {
                    capture = Some(candidate);
                    break 'capture_facts;
                }
            }
        }
        let capture: &ExecutableVelloCaptureFacts = capture
            .ok_or_else(|| preparation_error("the bounded Vello capture handoff is missing"))?;
        let binding = self.texture_binding_for_pass(request.id, target, account)?;
        let spatial = self.validate_texture_binding(&binding, target, account)?;
        if spatial.device_extent != capture.target_extent()
            || spatial.texel_origin != capture.texel_origin()
            || spatial.raster_scale != capture.raster_scale()
        {
            return Err(preparation_error(
                "the bounded Vello capture target changed after preparation",
            ));
        }
        Ok(VelloCaptureEncodingHandoff {
            pass: request.id,
            target,
            work: capture.work(),
            initial_transform: capture.initial_transform(),
            antialiasing: capture.antialiasing(),
            target_extent: capture.target_extent(),
            raster_scale: capture.raster_scale(),
            texture: binding.texture(),
            view: binding.view(),
            session: Arc::clone(session),
        })
    }

    fn encode_clear_root(
        &self,
        encoder: &mut wgpu::CommandEncoder,
        request: &PreparedPassEncodingRequest,
        account: &Account,
    ) -> Result<CorePassEncodingFacts> {
        let RuntimePassKind::ClearRoot {
            initialization,
            color,
        } = &request.kind
        else {
            return Err(preparation_error(
                "the base-graph root clear changed its initialization contract",
            ));
        };
        let RuntimeResultBinding::Resource(target) = request.result else {
            return Err(preparation_error("the base-graph root clear has no target"));
        };
        let target_request = self.resource_request(target, account)?;
        let exact_initialization = match initialization {
            RuntimeInitialization::SurfaceBaseColor => {
                target == self.plan.root_working_image
                    && target_request.role == RuntimeResourceRole::RootWorkingImage
            }
            RuntimeInitialization::Transparent => {
                *color == Color::TRANSPARENT
                    && target_request.role == RuntimeResourceRole::IsolationWorkingImage
            }
        };
        if !exact_initialization
            || !request.reads.is_empty()
            || request.spatial_uniform.is_some()
            || request.color_filter_operations.is_some()
            || request.composite_parameters.is_some()
            || request.cache_keys.is_some()
        {
            return Err(preparation_error(
                "the base-graph root clear has non-root or sampled bindings",
            ));
        }
        let binding = self.texture_binding_for_pass(request.id, target, account)?;
        let spatial = self.validate_texture_binding(&binding, target, account)?;
        let alpha = f64::from(color.a());
        {
            account.work(RenderWork::ImagePasses, 1)?;
            let _pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("Surgeist base graph full root clear"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: binding.view(),
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(wgpu::Color {
                            r: f64::from(color.r()) * alpha,
                            g: f64::from(color.g()) * alpha,
                            b: f64::from(color.b()) * alpha,
                            a: alpha,
                        }),
                        store: wgpu::StoreOp::Store,
                    },
                    depth_slice: None,
                })],
                depth_stencil_attachment: None,
                occlusion_query_set: None,
                timestamp_writes: None,
                multiview_mask: None,
            });
        }
        Ok(CorePassEncodingFacts {
            full_target: spatial.device_extent == target_request.spatial.device_extent,
            ..CorePassEncodingFacts::default()
        })
    }

    fn encode_canonicalize_capture(
        &self,
        encoder: &mut wgpu::CommandEncoder,
        request: &PreparedPassEncodingRequest,
        account: &Account,
    ) -> Result<CorePassEncodingFacts> {
        let source = exact_core_pass_read(request, RuntimeReadRole::CaptureSource, account)?;
        let RuntimeResultBinding::Resource(target) = request.result else {
            return Err(preparation_error(
                "the core-pass canonicalization pass has no prepared result",
            ));
        };
        let source_binding = self.texture_binding_for_pass(request.id, source.resource, account)?;
        let source_spatial =
            self.validate_texture_binding(&source_binding, source.resource, account)?;
        let target_binding = self.texture_binding_for_pass(request.id, target, account)?;
        let target_spatial = self.validate_texture_binding(&target_binding, target, account)?;
        if source_spatial != target_spatial
            || self.resource_request(source.resource, account)?.format
                != RuntimeResourceFormat::VelloCaptureRgba8Unorm
            || self.resource_request(target, account)?.format
                != RuntimeResourceFormat::Working(self.plan.working_format)
        {
            return Err(preparation_error(
                "core-pass canonicalization changed its exact capture-to-working binding",
            ));
        }
        let region = GraphRenderRegion::full(target_spatial.device_extent)?;
        let fixed_blend = self.encode_core_sampled_render_pass(
            encoder,
            request,
            source,
            CorePassSampledRenderTarget {
                view: target_binding.view(),
                extent: target_spatial.device_extent,
                region: Some(region),
                load: wgpu::LoadOp::Clear(wgpu::Color::TRANSPARENT),
                label: "Surgeist core-pass canonicalize capture",
            },
            account,
        )?;
        Ok(CorePassEncodingFacts {
            full_target: true,
            exact_spatial_uniform: true,
            fixed_source_over_blend: fixed_blend,
            ..CorePassEncodingFacts::default()
        })
    }

    fn encode_span_source_over(
        &self,
        encoder: &mut wgpu::CommandEncoder,
        request: &PreparedPassEncodingRequest,
        account: &Account,
    ) -> Result<CorePassEncodingFacts> {
        let parent = exact_core_pass_read(request, RuntimeReadRole::CompositeParent, account)?;
        let source = exact_core_pass_read(request, RuntimeReadRole::CompositeSource, account)?;
        let RuntimeResultBinding::Resource(target) = request.result else {
            return Err(preparation_error(
                "the core-pass source-over pass has no prepared result",
            ));
        };
        let parent_binding = self.texture_binding_for_pass(request.id, parent.resource, account)?;
        let parent_spatial =
            self.validate_texture_binding(&parent_binding, parent.resource, account)?;
        let source_binding = self.texture_binding_for_pass(request.id, source.resource, account)?;
        let source_spatial =
            self.validate_texture_binding(&source_binding, source.resource, account)?;
        let target_binding = self.texture_binding_for_pass(request.id, target, account)?;
        let target_spatial = self.validate_texture_binding(&target_binding, target, account)?;
        let parent_and_result_distinct = parent.resource != target
            && parent_binding.allocation_resource() != target_binding.allocation_resource();
        if !parent_and_result_distinct
            || parent_spatial != target_spatial
            || parent_binding.texture().format() != target_binding.texture().format()
            || self.resource_request(parent.resource, account)?.format
                != RuntimeResourceFormat::Working(self.plan.working_format)
            || self.resource_request(source.resource, account)?.format
                != RuntimeResourceFormat::Working(self.plan.working_format)
            || self.resource_request(target, account)?.format
                != RuntimeResourceFormat::Working(self.plan.working_format)
        {
            return Err(preparation_error(
                "core-pass source-over parent, source, and distinct result bindings are inconsistent",
            ));
        }

        let copy_extent = layer_composite_copy_extent(parent_spatial);
        copy_layer_composite_parent(
            encoder,
            &parent_binding,
            &target_binding,
            copy_extent,
            account,
        )?;

        let region = GraphRenderRegion::bounded_source(source_spatial, target_spatial)?;
        let preserved_signed_source_origin =
            request.spatial_uniform.as_ref().is_some_and(|bytes| {
                core_pass_spatial_uniform_preserves_source_origin(bytes, source_spatial)
                    && region.is_none_or(|region| {
                        let expected_x = (source_spatial.texel_origin.x()
                            - target_spatial.texel_origin.x())
                            * target_spatial.raster_scale;
                        let expected_y = (source_spatial.texel_origin.y()
                            - target_spatial.texel_origin.y())
                            * target_spatial.raster_scale;
                        close_f64(region.unclipped_x, expected_x)
                            && close_f64(region.unclipped_y, expected_y)
                    })
            });
        let fixed_blend = self.encode_core_sampled_render_pass(
            encoder,
            request,
            source,
            CorePassSampledRenderTarget {
                view: target_binding.view(),
                extent: target_spatial.device_extent,
                region,
                load: wgpu::LoadOp::Load,
                label: "Surgeist graph bounded span source-over",
            },
            account,
        )?;
        Ok(CorePassEncodingFacts {
            exact_spatial_uniform: true,
            parent_and_result_distinct,
            copied_full_parent_before_render: copy_extent.width == target_binding.texture().width()
                && copy_extent.height == target_binding.texture().height(),
            sampled_only_source: request.reads.len() == 2,
            fixed_source_over_blend: fixed_blend,
            preserved_signed_source_origin,
            ..CorePassEncodingFacts::default()
        })
    }

    fn encode_copy_backdrop(
        &self,
        encoder: &mut wgpu::CommandEncoder,
        request: &PreparedPassEncodingRequest,
        account: &Account,
    ) -> Result<CopyBackdropEncodingFacts> {
        if !matches!(request.kind, RuntimePassKind::CopyBackdrop) {
            return Err(preparation_error(
                "the backdrop-copy payload changed after preparation",
            ));
        }
        let parent = exact_core_pass_read(request, RuntimeReadRole::CompletedParent, account)?;
        let RuntimeResultBinding::Resource(target) = request.result else {
            return Err(preparation_error(
                "the backdrop-copy pass has no prepared result",
            ));
        };
        let parent_binding =
            self.texture_binding_for_pass(request.id, parent.resource(), account)?;
        let parent_spatial =
            self.validate_texture_binding(&parent_binding, parent.resource(), account)?;
        let target_binding = self.texture_binding_for_pass(request.id, target, account)?;
        let target_spatial = self.validate_texture_binding(&target_binding, target, account)?;
        let distinct = parent.resource() != target
            && parent_binding.allocation_resource() != target_binding.allocation_resource();
        let spatial = request
            .spatial_uniform
            .as_ref()
            .ok_or_else(|| preparation_error("the backdrop-copy spatial bytes are missing"))?;
        let expected_spatial = PassSpatialUniformBytes::try_from_runtime_spatial_descriptors(
            parent_spatial,
            target_spatial,
            Some(account),
        )?;
        let keys = request
            .cache_keys
            .as_ref()
            .ok_or_else(|| preparation_error("the backdrop-copy cache keys are missing"))?;
        let (sampler, layout, pipeline) = self.copy_backdrop_pass_objects(request.id, account)?;
        if request.reads.len() != 1
            || !distinct
            || spatial != &expected_spatial
            || !keys.samplers_match_in(&[parent.sampler_key()], account)?
            || parent.sampling_filter() != RuntimeSamplingFilter::Nearest
            || parent.sampling_edge() != RuntimeSamplingEdge::TransparentBlack
            || self.resource_request(parent.resource(), account)?.format
                != RuntimeResourceFormat::Working(self.plan.working_format)
            || self.resource_request(target, account)?.role != RuntimeResourceRole::BackdropCopy
            || self.resource_request(target, account)?.format
                != RuntimeResourceFormat::Working(self.plan.working_format)
            || !parent_binding
                .texture()
                .usage()
                .contains(wgpu::TextureUsages::TEXTURE_BINDING)
            || !target_binding
                .texture()
                .usage()
                .contains(wgpu::TextureUsages::RENDER_ATTACHMENT)
        {
            return Err(preparation_error(
                "the backdrop-copy bindings differ from the checked pass",
            ));
        }
        let region = GraphRenderRegion::full(target_spatial.device_extent)?;
        let uniform = self.create_core_pass_spatial_uniform_buffer(spatial, account)?;
        let bind_group =
            self.resources
                .generation()
                .opaque(GpuObjectKind::BindGroup, Some(account), || {
                    self.device.create_bind_group(&wgpu::BindGroupDescriptor {
                        label: Some("Surgeist backdrop exact backdrop-copy bindings"),
                        layout,
                        entries: &[
                            wgpu::BindGroupEntry {
                                binding: 0,
                                resource: wgpu::BindingResource::TextureView(parent_binding.view()),
                            },
                            wgpu::BindGroupEntry {
                                binding: 1,
                                resource: wgpu::BindingResource::Sampler(sampler),
                            },
                            wgpu::BindGroupEntry {
                                binding: 2,
                                resource: uniform.as_entire_binding(),
                            },
                        ],
                    })
                })?;
        encode_spatial_filter_full_target_pass(
            encoder,
            target_binding.view(),
            SpatialPassDrawBindings {
                pipeline,
                bind_group: &bind_group,
            },
            region,
            "Surgeist backdrop bounded completed-parent copy",
            false,
            account,
        )?;
        Ok(CopyBackdropEncodingFacts {
            exact_prepared_bindings: true,
            source_and_result_are_distinct: distinct,
            validated_viewport_and_scissor: spatial_filter_full_region_is_exact(
                region,
                target_spatial.device_extent,
            ),
            preserved_signed_mapping: core_pass_spatial_uniform_preserves_source_origin(
                spatial,
                parent_spatial,
            ),
        })
    }

    fn prepare_color_filter_encoding<'prepared>(
        &'prepared self,
        request: &'prepared PreparedPassEncodingRequest,
        account: &Account,
    ) -> Result<PreparedColorFilterEncoding<'prepared>> {
        let RuntimePassKind::ColorFilter(Some(filter)) = &request.kind else {
            return Err(preparation_error(
                "the color-filter color pass changed its checked semantic payload",
            ));
        };
        let source = exact_core_pass_read(request, RuntimeReadRole::FilterSource, account)?;
        let RuntimeResultBinding::Resource(target) = request.result else {
            return Err(preparation_error(
                "the color-filter color pass has no prepared working result",
            ));
        };
        if request.reads.len() != 1 || filter.operations().is_empty() {
            return Err(preparation_error(
                "the color-filter color pass must have one source and a nonempty ordered program",
            ));
        }
        let source_binding =
            self.texture_binding_for_pass(request.id, source.resource(), account)?;
        let source_spatial =
            self.validate_texture_binding(&source_binding, source.resource(), account)?;
        let target_binding = self.texture_binding_for_pass(request.id, target, account)?;
        let target_spatial = self.validate_texture_binding(&target_binding, target, account)?;
        let source_and_result_are_distinct = source.resource() != target
            && source_binding.allocation_resource() != target_binding.allocation_resource();
        if !source_and_result_are_distinct
            || source_spatial != target_spatial
            || filter.spatial.source != source_spatial
            || filter.spatial.result != target_spatial
            || filter.edge != RuntimeSamplingEdge::ClampToExtent
            || source.sampling_filter() != RuntimeSamplingFilter::Nearest
            || source.sampling_edge() != RuntimeSamplingEdge::ClampToExtent
            || self.resource_request(source.resource(), account)?.format
                != RuntimeResourceFormat::Working(self.plan.working_format)
            || self.resource_request(target, account)?.format
                != RuntimeResourceFormat::Working(self.plan.working_format)
            || !source_binding
                .texture()
                .usage()
                .contains(wgpu::TextureUsages::TEXTURE_BINDING)
            || !target_binding
                .texture()
                .usage()
                .contains(wgpu::TextureUsages::RENDER_ATTACHMENT)
        {
            return Err(preparation_error(
                "the color-filter source and distinct working result bindings are inconsistent",
            ));
        }
        let keys = request.cache_keys.as_ref().ok_or_else(|| {
            preparation_error("the color-filter color pass has no provisional cache keys")
        })?;
        if !keys.samplers_match_in(&[source.sampler_key()], account)? {
            return Err(preparation_error(
                "the color-filter color pass changed its nearest ClampToExtent sampler",
            ));
        }
        let spatial = request.spatial_uniform.as_ref().ok_or_else(|| {
            preparation_error("the color-filter color pass has no prepared spatial bytes")
        })?;
        let expected_spatial = PassSpatialUniformBytes::try_from_runtime_spatial_descriptors(
            source_spatial,
            target_spatial,
            Some(account),
        )?;
        if spatial != &expected_spatial {
            return Err(preparation_error(
                "the color-filter color spatial bytes changed after immutable preparation",
            ));
        }
        let (operation_buffer, exact_operation_bytes) =
            self.validate_color_filter_operation_buffer(request, filter, account)?;
        let objects = self.color_filter_pass_objects(keys, account)?;
        objects.require_encoding_ready()?;
        let (region, validated_viewport_and_scissor, preserved_signed_texel_mapping) =
            validate_color_filter_render_region(spatial, source_spatial, target_spatial)?;
        Ok(PreparedColorFilterEncoding {
            source_binding,
            target_binding,
            objects,
            spatial,
            operation_buffer,
            region,
            facts: ColorFilterEncodingFacts {
                exact_operation_bytes,
                exact_source_spatial_and_operations: spatial == &expected_spatial,
                source_and_result_are_distinct,
                validated_viewport_and_scissor,
                preserved_signed_texel_mapping,
            },
        })
    }

    fn validate_color_filter_operation_buffer<'prepared>(
        &'prepared self,
        request: &'prepared PreparedPassEncodingRequest,
        filter: &RuntimeColorFilter,
        account: &Account,
    ) -> Result<(&'prepared wgpu::Buffer, bool)> {
        let operation_bytes = request.color_filter_operations.as_ref().ok_or_else(|| {
            preparation_error("the color-filter color pass has no checked operation bytes")
        })?;
        let expected_operation_bytes =
            ColorFilterOperationBytes::try_from_runtime_operations_with_limits(
                account,
                filter.operations(),
                ColorFilterOperationBufferLimits::from_device_limits(&self.device.limits()),
            )?;
        let operation_binding = self
            .color_filter_operation_bindings
            .get(&request.id, account)?
            .ok_or_else(|| {
                preparation_error("the color-filter operation buffer binding is missing")
            })?;
        let operation_buffer = operation_binding.buffer.as_ref().ok_or_else(|| {
            preparation_error("the color-filter operation buffer is stale or already released")
        })?;
        let exact = operation_bytes.matches_in(&expected_operation_bytes, account)?
            && operation_binding
                .bytes
                .matches_in(operation_bytes, account)?
            && operation_buffer.size()
                == u64::try_from(operation_bytes.as_bytes().len()).map_err(|_| {
                    preparation_error("the color-filter operation buffer size does not fit u64")
                })?
            && operation_buffer.usage()
                == wgpu::BufferUsages::STORAGE.union(wgpu::BufferUsages::COPY_DST);
        if !exact {
            return Err(preparation_error(
                "the color-filter operation buffer differs from its exact checked bytes",
            ));
        }
        Ok((operation_buffer, exact))
    }

    fn encode_color_filter(
        &self,
        encoder: &mut wgpu::CommandEncoder,
        request: &PreparedPassEncodingRequest,
        account: &Account,
    ) -> Result<ColorFilterEncodingFacts> {
        let prepared = self.prepare_color_filter_encoding(request, account)?;
        let spatial_buffer =
            self.create_core_pass_spatial_uniform_buffer(prepared.spatial, account)?;
        let bind_group =
            self.resources
                .generation()
                .opaque(GpuObjectKind::BindGroup, Some(account), || {
                    self.device.create_bind_group(&wgpu::BindGroupDescriptor {
                        label: Some("Surgeist color-filter exact color-filter bindings"),
                        layout: prepared.objects.bind_group_layout(),
                        entries: &[
                            wgpu::BindGroupEntry {
                                binding: 0,
                                resource: wgpu::BindingResource::TextureView(
                                    prepared.source_binding.view(),
                                ),
                            },
                            wgpu::BindGroupEntry {
                                binding: 1,
                                resource: wgpu::BindingResource::Sampler(
                                    prepared.objects.source_sampler(),
                                ),
                            },
                            wgpu::BindGroupEntry {
                                binding: 2,
                                resource: spatial_buffer.as_entire_binding(),
                            },
                            wgpu::BindGroupEntry {
                                binding: 3,
                                resource: prepared.operation_buffer.as_entire_binding(),
                            },
                        ],
                    })
                })?;
        account.work_batch([
            (RenderWork::ImagePasses, 1),
            (
                RenderWork::ScheduledFilterTexels,
                scheduled_region_texels(prepared.region),
            ),
        ])?;
        let mut pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
            label: Some("Surgeist color-filter bounded ordered color filter"),
            color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                view: prepared.target_binding.view(),
                resolve_target: None,
                ops: wgpu::Operations {
                    load: wgpu::LoadOp::Clear(wgpu::Color::TRANSPARENT),
                    store: wgpu::StoreOp::Store,
                },
                depth_slice: None,
            })],
            depth_stencil_attachment: None,
            occlusion_query_set: None,
            timestamp_writes: None,
            multiview_mask: None,
        });
        pass.set_pipeline(prepared.objects.render_pipeline());
        pass.set_bind_group(0, &*bind_group, &[]);
        pass.set_viewport(
            prepared.region.viewport_x,
            prepared.region.viewport_y,
            prepared.region.viewport_width,
            prepared.region.viewport_height,
            0.0,
            1.0,
        );
        pass.set_scissor_rect(
            prepared.region.scissor_x,
            prepared.region.scissor_y,
            prepared.region.scissor_width,
            prepared.region.scissor_height,
        );
        pass.draw(0..3, 0..1);

        Ok(prepared.facts)
    }

    fn encode_blur(
        &self,
        encoder: &mut wgpu::CommandEncoder,
        request: &PreparedPassEncodingRequest,
        account: &Account,
    ) -> Result<BlurEncodingFacts> {
        let blur = match &request.kind {
            RuntimePassKind::BlurHorizontal(Some(blur))
            | RuntimePassKind::BlurVertical(Some(blur)) => blur,
            _ => {
                return Err(preparation_error(
                    "the spatial-filter blur payload is missing",
                ));
            }
        };
        let source = exact_core_pass_read(request, RuntimeReadRole::FilterSource, account)?;
        let RuntimeResultBinding::Resource(target) = request.result else {
            return Err(preparation_error(
                "the spatial-filter blur result is missing",
            ));
        };
        let source_binding =
            self.texture_binding_for_pass(request.id, source.resource(), account)?;
        let source_spatial =
            self.validate_texture_binding(&source_binding, source.resource(), account)?;
        let target_binding = self.texture_binding_for_pass(request.id, target, account)?;
        let target_spatial = self.validate_texture_binding(&target_binding, target, account)?;
        let distinct = source.resource() != target
            && source_binding.allocation_resource() != target_binding.allocation_resource();
        let spatial = request.spatial_uniform.as_ref().ok_or_else(|| {
            preparation_error("the spatial-filter blur spatial bytes are missing")
        })?;
        let expected_spatial = PassSpatialUniformBytes::try_from_runtime_spatial_descriptors(
            source_spatial,
            target_spatial,
            Some(account),
        )?;
        let kernel = self
            .gaussian_kernel_binding_for_pass(request.id, account)?
            .ok_or_else(|| {
                preparation_error("the spatial-filter blur kernel binding is missing")
            })?;
        let keys = request
            .cache_keys
            .as_ref()
            .ok_or_else(|| preparation_error("the spatial-filter blur cache keys are missing"))?;
        let (sampler, layout, pipeline) = self.blur_pass_objects(request.id, account)?;
        let edge_bytes = backdrop_blur_edge_uniform_bytes(
            blur,
            source,
            request.blur_edge_parameters.as_ref(),
            account,
        )?;
        if request.reads.len() != 1
            || !distinct
            || blur.spatial.source != source_spatial
            || blur.spatial.result != target_spatial
            || blur.kernel != kernel.key()
            || spatial != &expected_spatial
            || !keys.samplers_match_in(&[source.sampler_key()], account)?
            || source.sampling_filter() != RuntimeSamplingFilter::Linear
            || self.resource_request(source.resource(), account)?.format
                != RuntimeResourceFormat::Working(self.plan.working_format)
            || self.resource_request(target, account)?.format
                != RuntimeResourceFormat::Working(self.plan.working_format)
        {
            return Err(preparation_error(
                "the spatial-filter blur bindings differ from the checked pass",
            ));
        }
        let region = GraphRenderRegion::full(target_spatial.device_extent)?;
        let spatial_buffer = self.create_core_pass_spatial_uniform_buffer(spatial, account)?;
        let edge_buffer = edge_bytes
            .as_ref()
            .map(|bytes| self.create_backdrop_blur_edge_parameter_buffer(bytes, account))
            .transpose()?;
        let mut entries = account.temporary(4 + usize::from(edge_buffer.is_some()))?;
        entries.extend_from_slice(&[
            wgpu::BindGroupEntry {
                binding: 0,
                resource: wgpu::BindingResource::TextureView(source_binding.view()),
            },
            wgpu::BindGroupEntry {
                binding: 1,
                resource: wgpu::BindingResource::Sampler(sampler),
            },
            wgpu::BindGroupEntry {
                binding: 2,
                resource: spatial_buffer.as_entire_binding(),
            },
            wgpu::BindGroupEntry {
                binding: 3,
                resource: kernel.buffer().as_entire_binding(),
            },
        ])?;
        if let Some(edge_buffer) = &edge_buffer {
            entries.append_slot()?.write(wgpu::BindGroupEntry {
                binding: 4,
                resource: edge_buffer.as_entire_binding(),
            });
        }
        let bind_group =
            self.resources
                .generation()
                .opaque(GpuObjectKind::BindGroup, Some(account), || {
                    self.device.create_bind_group(&wgpu::BindGroupDescriptor {
                        label: Some("Surgeist spatial-filter exact blur bindings"),
                        layout,
                        entries: &entries,
                    })
                })?;
        encode_spatial_filter_full_target_pass(
            encoder,
            target_binding.view(),
            SpatialPassDrawBindings {
                pipeline,
                bind_group: &bind_group,
            },
            region,
            "Surgeist spatial-filter Gaussian blur",
            true,
            account,
        )?;
        Ok(BlurEncodingFacts {
            exact_prepared_bindings: true,
            distinct_source_and_result: distinct,
            validated_region: spatial_filter_full_region_is_exact(
                region,
                target_spatial.device_extent,
            ),
            preserved_signed_mapping: core_pass_spatial_uniform_preserves_source_origin(
                spatial,
                source_spatial,
            ),
        })
    }

    fn encode_drop_shadow_colorize(
        &self,
        encoder: &mut wgpu::CommandEncoder,
        request: &PreparedPassEncodingRequest,
        account: &Account,
    ) -> Result<DropShadowColorizeEncodingFacts> {
        let RuntimePassKind::DropShadowColorize(Some(shadow)) = &request.kind else {
            return Err(preparation_error(
                "the spatial-filter drop-shadow payload is missing",
            ));
        };
        let source = exact_core_pass_read(request, RuntimeReadRole::BlurredSourceAlpha, account)?;
        let RuntimeResultBinding::Resource(target) = request.result else {
            return Err(preparation_error(
                "the spatial-filter drop-shadow result is missing",
            ));
        };
        let source_binding =
            self.texture_binding_for_pass(request.id, source.resource(), account)?;
        let source_spatial =
            self.validate_texture_binding(&source_binding, source.resource(), account)?;
        let target_binding = self.texture_binding_for_pass(request.id, target, account)?;
        let target_spatial = self.validate_texture_binding(&target_binding, target, account)?;
        let distinct = source.resource() != target
            && source_binding.allocation_resource() != target_binding.allocation_resource();
        let spatial = request.spatial_uniform.as_ref().ok_or_else(|| {
            preparation_error("the spatial-filter drop-shadow spatial bytes are missing")
        })?;
        let expected_spatial = PassSpatialUniformBytes::try_from_runtime_spatial_descriptors(
            source_spatial,
            target_spatial,
            Some(account),
        )?;
        let parameters = request.drop_shadow_parameters.as_ref().ok_or_else(|| {
            preparation_error("the spatial-filter drop-shadow parameter bytes are missing")
        })?;
        let expected_parameters =
            DropShadowParameterBytes::try_new(shadow.offset, shadow.color, Some(account))?;
        let keys = request.cache_keys.as_ref().ok_or_else(|| {
            preparation_error("the spatial-filter drop-shadow cache keys are missing")
        })?;
        let (sampler, layout, pipeline) =
            self.drop_shadow_colorize_pass_objects(request.id, account)?;
        if request.reads.len() != 1
            || !distinct
            || shadow.spatial.source != source_spatial
            || shadow.spatial.result != target_spatial
            || spatial != &expected_spatial
            || parameters != &expected_parameters
            || !keys.samplers_match_in(&[source.sampler_key()], account)?
            || source.sampling_filter() != RuntimeSamplingFilter::Linear
            || source.sampling_edge() != RuntimeSamplingEdge::TransparentBlack
        {
            return Err(preparation_error(
                "the spatial-filter drop-shadow bindings differ from the checked pass",
            ));
        }
        let region = GraphRenderRegion::full(target_spatial.device_extent)?;
        let spatial_buffer = self.create_core_pass_spatial_uniform_buffer(spatial, account)?;
        let parameter_buffer = self.create_drop_shadow_parameter_buffer(parameters, account)?;
        let bind_group =
            self.resources
                .generation()
                .opaque(GpuObjectKind::BindGroup, Some(account), || {
                    self.device.create_bind_group(&wgpu::BindGroupDescriptor {
                        label: Some("Surgeist spatial-filter exact drop-shadow colorize bindings"),
                        layout,
                        entries: &[
                            wgpu::BindGroupEntry {
                                binding: 0,
                                resource: wgpu::BindingResource::TextureView(source_binding.view()),
                            },
                            wgpu::BindGroupEntry {
                                binding: 1,
                                resource: wgpu::BindingResource::Sampler(sampler),
                            },
                            wgpu::BindGroupEntry {
                                binding: 2,
                                resource: spatial_buffer.as_entire_binding(),
                            },
                            wgpu::BindGroupEntry {
                                binding: 3,
                                resource: parameter_buffer.as_entire_binding(),
                            },
                        ],
                    })
                })?;
        encode_spatial_filter_full_target_pass(
            encoder,
            target_binding.view(),
            SpatialPassDrawBindings {
                pipeline,
                bind_group: &bind_group,
            },
            region,
            "Surgeist spatial-filter drop-shadow colorize",
            true,
            account,
        )?;
        Ok(DropShadowColorizeEncodingFacts {
            exact_prepared_bindings: true,
            distinct_source_and_result: distinct,
            validated_region: spatial_filter_full_region_is_exact(
                region,
                target_spatial.device_extent,
            ),
            preserved_signed_mapping: core_pass_spatial_uniform_preserves_source_origin(
                spatial,
                source_spatial,
            ),
        })
    }

    fn encode_drop_shadow_merge(
        &self,
        encoder: &mut wgpu::CommandEncoder,
        request: &PreparedPassEncodingRequest,
        account: &Account,
    ) -> Result<DropShadowMergeEncodingFacts> {
        let RuntimePassKind::Composite(Some(RuntimeComposite {
            kind: RuntimeCompositeKind::DropShadow,
            source_captured_before_outer_semantics: true,
        })) = &request.kind
        else {
            return Err(preparation_error(
                "the spatial-filter drop-shadow merge payload is missing",
            ));
        };
        let source = exact_core_pass_read(request, RuntimeReadRole::CompositeSource, account)?;
        let shadow = exact_core_pass_read(request, RuntimeReadRole::Shadow, account)?;
        let RuntimeResultBinding::Resource(target) = request.result else {
            return Err(preparation_error(
                "the spatial-filter drop-shadow merge result is missing",
            ));
        };
        let source_binding =
            self.texture_binding_for_pass(request.id, source.resource(), account)?;
        let source_spatial =
            self.validate_texture_binding(&source_binding, source.resource(), account)?;
        let shadow_binding =
            self.texture_binding_for_pass(request.id, shadow.resource(), account)?;
        let shadow_spatial =
            self.validate_texture_binding(&shadow_binding, shadow.resource(), account)?;
        let target_binding = self.texture_binding_for_pass(request.id, target, account)?;
        let target_spatial = self.validate_texture_binding(&target_binding, target, account)?;
        let distinct = source_binding.allocation_resource() != shadow_binding.allocation_resource()
            && source_binding.allocation_resource() != target_binding.allocation_resource()
            && shadow_binding.allocation_resource() != target_binding.allocation_resource();
        if request.reads.len() != 2 || !distinct || shadow_spatial != target_spatial {
            return Err(preparation_error(
                "the spatial-filter drop-shadow merge aliases or changed its shadow grid",
            ));
        }
        copy_layer_composite_parent(
            encoder,
            &shadow_binding,
            &target_binding,
            layer_composite_copy_extent(shadow_spatial),
            account,
        )?;
        let region = GraphRenderRegion::bounded_source(source_spatial, target_spatial)?;
        let fixed_blend = self.encode_core_sampled_render_pass(
            encoder,
            request,
            source,
            CorePassSampledRenderTarget {
                view: target_binding.view(),
                extent: target_spatial.device_extent,
                region,
                load: wgpu::LoadOp::Load,
                label: "Surgeist spatial-filter unchanged source over drop shadow",
            },
            account,
        )?;
        let spatial = request.spatial_uniform.as_ref().ok_or_else(|| {
            preparation_error("the spatial-filter merge spatial bytes are missing")
        })?;
        Ok(DropShadowMergeEncodingFacts {
            exact_prepared_bindings: true,
            distinct_source_shadow_and_result: distinct,
            validated_region: region.is_none_or(|region| {
                region.scissor_x.saturating_add(region.scissor_width)
                    <= target_spatial.device_extent.width()
                    && region.scissor_y.saturating_add(region.scissor_height)
                        <= target_spatial.device_extent.height()
            }),
            preserved_signed_mapping: core_pass_spatial_uniform_preserves_source_origin(
                spatial,
                source_spatial,
            ),
            fixed_source_over_blend: fixed_blend,
            reads_original_source_and_shadow: source.sampling_edge()
                == RuntimeSamplingEdge::TransparentBlack
                && shadow.sampling_edge() == RuntimeSamplingEdge::TransparentBlack,
        })
    }

    fn layer_composite_semantic<'prepared>(
        request: &'prepared PreparedPassEncodingRequest,
        account: &Account,
    ) -> Result<LayerCompositeSemantic<'prepared>> {
        let RuntimePassKind::Composite(Some(RuntimeComposite {
            kind:
                RuntimeCompositeKind::Layer {
                    transform,
                    parameters,
                    clip_coverage,
                    ..
                },
            source_captured_before_outer_semantics: true,
        })) = &request.kind
        else {
            return Err(preparation_error(
                "the composition layer composite changed its checked semantic payload",
            ));
        };
        let parent = *exact_core_pass_read(request, RuntimeReadRole::CompositeParent, account)?;
        let source = *exact_core_pass_read(request, RuntimeReadRole::CompositeSource, account)?;
        let clip = clip_coverage
            .map(|resource| {
                let read = *exact_core_pass_read(request, RuntimeReadRole::ClipCoverage, account)?;
                if read.resource() != resource {
                    return Err(preparation_error(
                        "the composition clip coverage read changed its exact resource",
                    ));
                }
                Ok(read)
            })
            .transpose()?;
        let alpha_mask = parameters
            .alpha_mask()
            .map(|mask| {
                let read = *exact_core_pass_read(request, RuntimeReadRole::AlphaMask, account)?;
                if read.resource() != mask.resource() {
                    return Err(preparation_error(
                        "the composition alpha-mask read changed its exact retained upload",
                    ));
                }
                Ok(read)
            })
            .transpose()?;
        let RuntimeResultBinding::Resource(target) = request.result else {
            return Err(preparation_error(
                "the composition layer composite has no prepared result",
            ));
        };
        let expected_read_count = 2usize
            .saturating_add(usize::from(clip.is_some()))
            .saturating_add(usize::from(alpha_mask.is_some()));
        if request.reads.len() != expected_read_count {
            return Err(preparation_error(
                "the composition layer composite contains an absent or duplicated semantic read",
            ));
        }
        let normal_path = parameters.composition().is_source_over_normal();
        Ok(LayerCompositeSemantic {
            transform: *transform,
            parameters,
            parent,
            source,
            clip,
            alpha_mask,
            target,
            normal_path,
            destination_path: !normal_path,
        })
    }

    fn layer_composite_bindings<'prepared>(
        &'prepared self,
        request: &'prepared PreparedPassEncodingRequest,
        semantic: LayerCompositeSemantic<'prepared>,
        account: &Account,
    ) -> Result<LayerCompositeBindings<'prepared>> {
        let parent =
            self.texture_binding_for_pass(request.id, semantic.parent.resource(), account)?;
        let parent_spatial =
            self.validate_texture_binding(&parent, semantic.parent.resource(), account)?;
        let source =
            self.texture_binding_for_pass(request.id, semantic.source.resource(), account)?;
        let source_spatial =
            self.validate_texture_binding(&source, semantic.source.resource(), account)?;
        let target = self.texture_binding_for_pass(request.id, semantic.target, account)?;
        let target_spatial = self.validate_texture_binding(&target, semantic.target, account)?;
        let clip = semantic
            .clip
            .map(|read| {
                let binding =
                    self.texture_binding_for_pass(request.id, read.resource(), account)?;
                if self.validate_texture_binding(&binding, read.resource(), account)?
                    != target_spatial
                {
                    return Err(preparation_error(
                        "the composition clip coverage grid changed from its parent mapping",
                    ));
                }
                Ok(binding)
            })
            .transpose()?;
        let mask = semantic
            .alpha_mask
            .map(|read| {
                let binding =
                    self.texture_binding_for_pass(request.id, read.resource(), account)?;
                let spatial = self.validate_texture_binding(&binding, read.resource(), account)?;
                if semantic
                    .parameters
                    .alpha_mask()
                    .is_none_or(|mask| spatial.device_extent != mask.image_dimensions())
                {
                    return Err(preparation_error(
                        "the composition alpha-mask texture changed from its exact image extent",
                    ));
                }
                Ok(binding)
            })
            .transpose()?;
        let parent_and_result_are_distinct = semantic.parent.resource() != semantic.target
            && parent.allocation_resource() != target.allocation_resource();
        let sampled_allocations_are_distinct = source.allocation_resource()
            != target.allocation_resource()
            && clip.as_ref().is_none_or(|binding| {
                binding.allocation_resource() != target.allocation_resource()
            })
            && mask.as_ref().is_none_or(|binding| {
                binding.allocation_resource() != target.allocation_resource()
            });
        let bindings = LayerCompositeBindings {
            semantic,
            parent,
            source,
            target,
            clip,
            mask,
            parent_spatial,
            source_spatial,
            target_spatial,
            parent_and_result_are_distinct,
            sampled_allocations_are_distinct,
        };
        self.validate_layer_composite_bindings(&bindings, account)?;
        Ok(bindings)
    }

    fn validate_layer_composite_bindings(
        &self,
        bindings: &LayerCompositeBindings<'_>,
        account: &Account,
    ) -> Result<()> {
        let semantic = &bindings.semantic;
        let clip_format_is_exact = match semantic.clip {
            Some(read) => {
                self.resource_request(read.resource(), account)?.format
                    == RuntimeResourceFormat::ClipCoverageRgba8Unorm
            }
            None => true,
        };
        let mask_format_is_exact = match semantic.alpha_mask {
            Some(read) => {
                self.resource_request(read.resource(), account)?.format
                    == RuntimeResourceFormat::ResolvedMaskRgba8Unorm
            }
            None => true,
        };
        if !bindings.parent_and_result_are_distinct
            || !bindings.sampled_allocations_are_distinct
            || bindings.parent_spatial != bindings.target_spatial
            || bindings.parent.texture().format() != bindings.target.texture().format()
            || self
                .resource_request(semantic.parent.resource(), account)?
                .format
                != RuntimeResourceFormat::Working(self.plan.working_format)
            || self
                .resource_request(semantic.source.resource(), account)?
                .format
                != RuntimeResourceFormat::Working(self.plan.working_format)
            || self.resource_request(semantic.target, account)?.format
                != RuntimeResourceFormat::Working(self.plan.working_format)
            || !clip_format_is_exact
            || !mask_format_is_exact
            || !bindings
                .parent
                .texture()
                .usage()
                .contains(wgpu::TextureUsages::COPY_SRC)
            || (semantic.destination_path
                && !bindings
                    .parent
                    .texture()
                    .usage()
                    .contains(wgpu::TextureUsages::TEXTURE_BINDING))
            || !bindings
                .target
                .texture()
                .usage()
                .contains(wgpu::TextureUsages::COPY_DST | wgpu::TextureUsages::RENDER_ATTACHMENT)
            || !bindings
                .source
                .texture()
                .usage()
                .contains(wgpu::TextureUsages::TEXTURE_BINDING)
            || bindings.clip.as_ref().is_some_and(|binding| {
                !binding
                    .texture()
                    .usage()
                    .contains(wgpu::TextureUsages::TEXTURE_BINDING)
            })
            || bindings.mask.as_ref().is_some_and(|binding| {
                !binding
                    .texture()
                    .usage()
                    .contains(wgpu::TextureUsages::TEXTURE_BINDING)
            })
        {
            return Err(preparation_error(
                "composition parent, source, and distinct composite result bindings are inconsistent",
            ));
        }
        Ok(())
    }

    fn create_layer_composite_bind_group(
        &self,
        bindings: &LayerCompositeBindings<'_>,
        objects: &ProvisionalCompositePassObjects<'_>,
        spatial: &PassSpatialUniformBytes,
        parameters: &CompositeParameterBytes,
        expected_read_count: usize,
        account: &Account,
    ) -> Result<(OwnedGpu<wgpu::BindGroup>, bool)> {
        let spatial_buffer = self.create_core_pass_spatial_uniform_buffer(spatial, account)?;
        let parameter_buffer = self.create_layer_composite_parameter_buffer(parameters, account)?;
        let mut entries = account.temporary(expected_read_count.saturating_add(4))?;
        entries.append_slot()?.write(wgpu::BindGroupEntry {
            binding: 0,
            resource: wgpu::BindingResource::TextureView(bindings.source.view()),
        });
        entries.append_slot()?.write(wgpu::BindGroupEntry {
            binding: 1,
            resource: wgpu::BindingResource::Sampler(objects.source_sampler()),
        });
        if bindings.semantic.destination_path {
            entries.append_slot()?.write(wgpu::BindGroupEntry {
                binding: 2,
                resource: wgpu::BindingResource::TextureView(bindings.parent.view()),
            });
        }
        if let Some(binding) = &bindings.clip {
            entries.append_slot()?.write(wgpu::BindGroupEntry {
                binding: 3,
                resource: wgpu::BindingResource::TextureView(binding.view()),
            });
        }
        if let Some(binding) = &bindings.mask {
            entries.append_slot()?.write(wgpu::BindGroupEntry {
                binding: 4,
                resource: wgpu::BindingResource::TextureView(binding.view()),
            });
        }
        entries.append_slot()?.write(wgpu::BindGroupEntry {
            binding: 5,
            resource: spatial_buffer.as_entire_binding(),
        });
        entries.append_slot()?.write(wgpu::BindGroupEntry {
            binding: 6,
            resource: parameter_buffer.as_entire_binding(),
        });
        let mut binds_parent_sample = false;
        for entry in &entries {
            account.work(RenderWork::ShaderBindingVisits, 1)?;
            if entry.binding == 2 {
                binds_parent_sample = true;
                break;
            }
        }
        let expected_entry_count = 4usize
            .saturating_add(usize::from(bindings.semantic.destination_path))
            .saturating_add(usize::from(bindings.clip.is_some()))
            .saturating_add(usize::from(bindings.mask.is_some()));
        if entries.len() != expected_entry_count
            || binds_parent_sample != bindings.semantic.destination_path
        {
            return Err(preparation_error(
                "the composition composite bind group contains a dummy or missing resource",
            ));
        }
        Ok((
            self.resources
                .generation()
                .opaque(GpuObjectKind::BindGroup, Some(account), || {
                    self.device.create_bind_group(&wgpu::BindGroupDescriptor {
                        label: Some("Surgeist composition exact layer-composite bindings"),
                        layout: objects.bind_group_layout(),
                        entries: &entries,
                    })
                })?,
            binds_parent_sample,
        ))
    }

    fn encode_composition_layer_composite(
        &self,
        encoder: &mut wgpu::CommandEncoder,
        request: &PreparedPassEncodingRequest,
        account: &Account,
    ) -> Result<LayerCompositeEncodingFacts> {
        let semantic = Self::layer_composite_semantic(request, account)?;
        let bindings = self.layer_composite_bindings(request, semantic, account)?;
        let normal_path = bindings.semantic.normal_path;
        let destination_path = bindings.semantic.destination_path;
        let source = bindings.semantic.source;
        let source_spatial = bindings.source_spatial;
        let target_spatial = bindings.target_spatial;
        let parent_spatial = bindings.parent_spatial;
        let parameters = bindings.semantic.parameters;
        let transform = match parameters.source_space() {
            crate::frame::CompositeSourceSpace::LayerLocal => bindings.semantic.transform,
            crate::frame::CompositeSourceSpace::SurfaceGrid => Transform::identity(),
        };
        let parent_binding = &bindings.parent;
        let target_binding = &bindings.target;
        let parent_and_result_are_distinct = bindings.parent_and_result_are_distinct;
        let sampled_allocations_are_distinct = bindings.sampled_allocations_are_distinct;
        let expected_read_count = request.reads.len();

        let keys = request.cache_keys.as_ref().ok_or_else(|| {
            preparation_error("the composition layer composite has no provisional cache keys")
        })?;
        if !keys.samplers_match_in(&[source.sampler_key()], account)? {
            return Err(preparation_error(
                "the composition layer composite changed its one exact source sampler",
            ));
        }
        let spatial = request.spatial_uniform.as_ref().ok_or_else(|| {
            preparation_error("the composition layer composite has no prepared spatial bytes")
        })?;
        let expected_spatial = PassSpatialUniformBytes::try_from_runtime_spatial_descriptors(
            source_spatial,
            target_spatial,
            Some(account),
        )?;
        if spatial != &expected_spatial {
            return Err(preparation_error(
                "the composition layer composite spatial bytes changed after preparation",
            ));
        }
        let composite_parameters = request.composite_parameters.as_ref().ok_or_else(|| {
            preparation_error("the composition layer composite has no prepared parameter bytes")
        })?;
        let expected_parameters = CompositeParameterBytes::try_from_runtime_layer(
            parameters,
            source_spatial,
            target_spatial,
            account,
        )?;
        if composite_parameters != &expected_parameters {
            return Err(preparation_error(
                "the composition layer composite parameter bytes changed after preparation",
            ));
        }
        let objects = self.layer_composite_pass_objects(keys, account)?;
        objects.require_encoding_ready()?;
        if objects.path()
            != if normal_path {
                ShaderCompositePathKey::Normal
            } else {
                ShaderCompositePathKey::DestinationSampling
            }
            || objects.has_clip_coverage() != bindings.clip.is_some()
            || objects.has_alpha_mask() != bindings.mask.is_some()
        {
            return Err(preparation_error(
                "the composition composite objects changed their checked entry-point interface",
            ));
        }

        let copy_extent = layer_composite_copy_extent(parent_spatial);
        copy_layer_composite_parent(
            encoder,
            parent_binding,
            target_binding,
            copy_extent,
            account,
        )?;

        let (region, preserved_signed_mapping) =
            layer_composite_region_mapping(spatial, source_spatial, target_spatial, transform)?;
        let (bind_group, binds_parent_sample) = self.create_layer_composite_bind_group(
            &bindings,
            &objects,
            spatial,
            composite_parameters,
            expected_read_count,
            account,
        )?;
        #[cfg(test)]
        let encoder_identity = std::ptr::from_mut(&mut *encoder) as usize;
        encode_layer_composite_region(
            encoder,
            target_binding,
            &objects,
            &bind_group,
            region,
            target_spatial,
            account,
        )?;

        Ok(LayerCompositeEncodingFacts {
            normal_path,
            destination_path,
            fixed_premultiplied_blend: normal_path && objects.uses_fixed_source_over_blend(),
            omits_parent_sample: normal_path && !binds_parent_sample,
            copied_full_parent: copy_extent.width == target_binding.texture().width()
                && copy_extent.height == target_binding.texture().height(),
            avoids_read_write_alias: parent_and_result_are_distinct
                && sampled_allocations_are_distinct,
            exact_resources_and_parameters: spatial == &expected_spatial
                && &expected_parameters == composite_parameters
                && binds_parent_sample == destination_path
                && objects.uses_fixed_source_over_blend() == normal_path
                && objects.uses_replace_blend() == destination_path,
            preserved_signed_mapping,
            #[cfg(test)]
            encoder_identity,
        })
    }

    fn encode_base_graph_present(
        &self,
        encoder: &mut wgpu::CommandEncoder,
        request: &PreparedPassEncodingRequest,
        output: &GraphExternalOutputView<'_>,
        account: &Account,
    ) -> Result<CorePassEncodingFacts> {
        let source = exact_core_pass_read(request, RuntimeReadRole::FinalWorkingImage, account)?;
        if request.result != RuntimeResultBinding::Output(output.format)
            || output.format != self.plan.output_format
            || output.extent != self.output_extent(account)?
        {
            return Err(preparation_error(
                "the core-pass present pass changed its external output binding",
            ));
        }
        let source_binding = self.texture_binding_for_pass(request.id, source.resource, account)?;
        let _ = self.validate_texture_binding(&source_binding, source.resource, account)?;
        let region = GraphRenderRegion::full(output.extent)?;
        let fixed_blend = self.encode_core_sampled_render_pass(
            encoder,
            request,
            source,
            CorePassSampledRenderTarget {
                view: output.view,
                extent: output.extent,
                region: Some(region),
                load: wgpu::LoadOp::Clear(wgpu::Color::TRANSPARENT),
                label: "Surgeist core-pass present external output",
            },
            account,
        )?;
        Ok(CorePassEncodingFacts {
            full_target: true,
            exact_spatial_uniform: true,
            external_output_exact: true,
            fixed_source_over_blend: fixed_blend,
            ..CorePassEncodingFacts::default()
        })
    }

    fn encode_core_sampled_render_pass(
        &self,
        encoder: &mut wgpu::CommandEncoder,
        request: &PreparedPassEncodingRequest,
        source: &RuntimeReadBinding,
        target: CorePassSampledRenderTarget<'_>,
        account: &Account,
    ) -> Result<bool> {
        let spatial = request.spatial_uniform.as_ref().ok_or_else(|| {
            preparation_error("the core sampled pass has no exact prepared spatial uniform")
        })?;
        let keys = request.cache_keys.as_ref().ok_or_else(|| {
            preparation_error("the core sampled pass has no provisional cache keys")
        })?;
        if !keys.samplers_match_in(&[source.sampler_key()], account)? {
            return Err(preparation_error(
                "the core sampled pass changed its exact source sampler",
            ));
        }
        let source_binding =
            self.texture_binding_for_pass(request.id, source.resource(), account)?;
        let _ = self.validate_texture_binding(&source_binding, source.resource(), account)?;
        let objects = self.core_pass_objects(keys, account)?;
        objects.require_encoding_ready()?;
        let sampler = objects.sampler()?;
        let uniform = self.create_core_pass_spatial_uniform_buffer(spatial, account)?;
        let bind_group =
            self.resources
                .generation()
                .opaque(GpuObjectKind::BindGroup, Some(account), || {
                    self.device.create_bind_group(&wgpu::BindGroupDescriptor {
                        label: Some("Surgeist core sampled pass bindings"),
                        layout: objects.bind_group_layout(),
                        entries: &[
                            wgpu::BindGroupEntry {
                                binding: 0,
                                resource: wgpu::BindingResource::TextureView(source_binding.view()),
                            },
                            wgpu::BindGroupEntry {
                                binding: 1,
                                resource: wgpu::BindingResource::Sampler(sampler),
                            },
                            wgpu::BindGroupEntry {
                                binding: 2,
                                resource: uniform.as_entire_binding(),
                            },
                        ],
                    })
                })?;
        if let Some(region) = target.region {
            if region.scissor_x.saturating_add(region.scissor_width) > target.extent.width()
                || region.scissor_y.saturating_add(region.scissor_height) > target.extent.height()
            {
                return Err(preparation_error(
                    "the graph bounded render region exceeds its exact target extent",
                ));
            }
            let work = match request.kind {
                RuntimePassKind::CanonicalizeCapture => RenderWork::ImagePasses,
                RuntimePassKind::Present => RenderWork::CustomPresentPasses,
                RuntimePassKind::Composite(_) => RenderWork::CompositePasses,
                _ => {
                    return Err(preparation_error(
                        "sampled render pass has no owned work unit",
                    ));
                }
            };
            if matches!(
                &request.kind,
                RuntimePassKind::Composite(Some(RuntimeComposite {
                    kind: RuntimeCompositeKind::DropShadow,
                    ..
                }))
            ) {
                account.work_batch([
                    (work, 1),
                    (
                        RenderWork::ScheduledFilterTexels,
                        scheduled_region_texels(region),
                    ),
                ])?;
            } else {
                account.work(work, 1)?;
            }
            let mut pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some(target.label),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: target.view,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: target.load,
                        store: wgpu::StoreOp::Store,
                    },
                    depth_slice: None,
                })],
                depth_stencil_attachment: None,
                occlusion_query_set: None,
                timestamp_writes: None,
                multiview_mask: None,
            });
            pass.set_pipeline(objects.render_pipeline());
            pass.set_bind_group(0, &*bind_group, &[]);
            pass.set_viewport(
                region.viewport_x,
                region.viewport_y,
                region.viewport_width,
                region.viewport_height,
                0.0,
                1.0,
            );
            pass.set_scissor_rect(
                region.scissor_x,
                region.scissor_y,
                region.scissor_width,
                region.scissor_height,
            );
            pass.draw(0..3, 0..1);
        }
        Ok(objects.uses_fixed_source_over_blend())
    }

    #[cfg_attr(
        not(test),
        expect(
            dead_code,
            reason = "graph encoding resolves prepared leases at each validated runtime last-use point"
        )
    )]
    pub(crate) fn complete_pass(&mut self, pass: RuntimePassId, account: &Account) -> Result<()> {
        if self.custom_spine_encoding_state.is_some() {
            return Err(preparation_error(
                "core pass completion belongs to its one-shot scheduler; discard an aborted graph and encoder",
            ));
        }
        self.complete_pass_inner(pass, account).map(drop)
    }

    fn complete_custom_spine_pass(
        &mut self,
        pass: RuntimePassId,
        account: &Account,
    ) -> Result<CompletedPassReleases> {
        if self.custom_spine_encoding_state != Some(CustomSpineEncodingState::Encoding) {
            return Err(preparation_error(
                "custom-spine-pass progress requires the active one-shot encoding session",
            ));
        }
        self.complete_pass_inner(pass, account)
    }

    fn complete_vello_capture(
        &mut self,
        pass: RuntimePassId,
        target: RuntimeResourceId,
        session: &Arc<()>,
        receipt: VelloCaptureCompletionReceipt,
        account: &Account,
    ) -> Result<()> {
        let request = self.require_current_pass(pass, account)?;
        if self.custom_spine_encoding_state != Some(CustomSpineEncodingState::Encoding)
            || !matches!(request.runtime.kind, RuntimePassKind::VelloCapture(Some(_)))
            || request.runtime.result != RuntimeResultBinding::Resource(target)
            || receipt.pass != pass
            || receipt.target != target
            || !Arc::ptr_eq(&receipt.session, session)
        {
            return Err(preparation_error(
                "Vello capture completion does not match the exact pass, target, and encoding session",
            ));
        }
        self.complete_pass_inner(pass, account).map(drop)
    }

    fn complete_pass_inner(
        &mut self,
        pass: RuntimePassId,
        account: &Account,
    ) -> Result<CompletedPassReleases> {
        let completed = ResolvedPassReleases::try_resolve(self, pass, account)?.take_bindings()?;
        self.next_pass = self.next_pass.saturating_add(1);
        Ok(completed)
    }

    pub(crate) fn finish_graph_submission(
        mut self,
        pending: PendingGraphEncoding,
        account: &Account,
    ) -> Result<PreparedGraphSubmission> {
        let completed_session = self.custom_spine_completed_session.take().ok_or_else(|| {
            preparation_error("the prepared graph has no completed encoding session")
        })?;
        if self.custom_spine_encoding_state != Some(CustomSpineEncodingState::Complete)
            || !Arc::ptr_eq(&completed_session, &pending.session)
            || !pending.summary.proves_complete_submission()
            || self.next_pass != self.plan.passes.len()
        {
            return Err(preparation_error(
                "the graph submission does not own one exact completed prepared frame",
            ));
        }
        for binding in self.resource_bindings.values() {
            account.work(RenderWork::ResourceProbes, 1)?;
            if binding.lease.is_some() {
                return Err(preparation_error(
                    "the graph submission retains an unresolved texture lease",
                ));
            }
        }
        for binding in self.kernel_bindings.values() {
            account.work(RenderWork::ResourceProbes, 1)?;
            if binding.lease.is_some() {
                return Err(preparation_error(
                    "the graph submission retains an unresolved kernel lease",
                ));
            }
        }
        for binding in self.convolution_weight_bindings.values() {
            account.work(RenderWork::ResourceProbes, 1)?;
            if binding.lease.is_some() {
                return Err(preparation_error(
                    "graph submission retains a convolution coefficient lease",
                ));
            }
        }
        for binding in self.offset_parameter_bindings.values() {
            account.work(RenderWork::ResourceProbes, 1)?;
            if binding.buffer.is_some() {
                return Err(preparation_error(
                    "graph submission retains an unresolved offset uniform",
                ));
            }
        }
        for binding in self.color_filter_operation_bindings.values() {
            account.work(RenderWork::ResourceProbes, 1)?;
            if binding.buffer.is_some() {
                return Err(preparation_error(
                    "the graph submission retains an unresolved operation buffer",
                ));
            }
        }
        let pass_cache_update = self.pass_cache_update.take().ok_or_else(|| {
            preparation_error("the completed graph lost its provisional pass-cache update")
        })?;
        let frame_scope = self.frame_scope.take().ok_or_else(|| {
            preparation_error("the completed graph lost its prepared frame scope")
        })?;
        let PendingGraphEncoding {
            summary: _,
            resources: capture_resources,
            session: _,
        } = pending;
        Ok(PreparedGraphSubmission {
            capture_resources,
            prepared_frame: PendingPreparedFrameCommit {
                frame_scope,
                pass_cache_update,
            },
        })
    }

    #[cfg_attr(
        not(test),
        expect(
            dead_code,
            reason = "graph encoding finishes a completely released prepared frame scope after execution"
        )
    )]
    pub(crate) fn finish(mut self) -> Result<FrameCleanup> {
        if self.next_pass != self.plan.passes.len()
            || self
                .resource_bindings
                .values()
                .any(|binding| binding.lease.is_some())
            || self
                .kernel_bindings
                .values()
                .any(|binding| binding.lease.is_some())
            || self
                .color_filter_operation_bindings
                .values()
                .any(|binding| binding.buffer.is_some())
        {
            return Err(preparation_error(
                "prepared graph cannot finish before every pass and last-use release",
            ));
        }
        let _ = self.pass_cache_update.take();
        self.frame_scope
            .take()
            .ok_or_else(|| preparation_error("prepared frame resource scope is already closed"))?
            .finish_checked()
    }
}

#[cfg(test)]
mod repeated_proof_accounting_tests {
    use super::*;
    use crate::accounting::{RenderAccountingOptions, RendererCpu};
    use crate::backend::{Backend, DeviceSlotIdentity};
    use crate::pass::test_support::{
        lower_capture_proof_fixture, lower_filter_proof_fixture, measure_proof_work,
    };

    async fn backend() -> (Backend, DeviceSlotIdentity) {
        let options = RenderAccountingOptions::new();
        let cpu = RendererCpu::new(options).unwrap();
        let mut backend = Backend::new(crate::ResourceCacheBudget::DISABLED, cpu, options).unwrap();
        let identity = backend
            .select_device(None)
            .await
            .unwrap()
            .expect("the real handoff fixture requires a compatible GPU adapter");
        (backend, identity)
    }

    fn lower_receipt_scene(
        scene: crate::Scene,
        capabilities: &crate::backend::DeviceCapabilities,
        setup: &Account,
    ) -> super::super::model::LoweredGraphPlan {
        let commands = scene
            .normalize(crate::Capabilities::CURRENT, setup)
            .unwrap();
        let context = crate::frame::FrameContext::try_new(
            crate::Size::new(4.0, 4.0),
            1.0,
            Antialiasing::Area,
            Color::TRANSPARENT,
        )
        .unwrap();
        let crate::frame::FramePlan::GpuGraph(graph) = commands.plan_for(context, setup).unwrap()
        else {
            panic!("the authored receipt fixture requires a GPU graph");
        };
        let format = capabilities
            .resolve_effect_working_format(crate::EffectQualityPolicy::AllowReducedPrecision)
            .unwrap();
        super::super::model::LoweredGraphPlan::try_lower_validated_graph(
            &graph,
            format,
            Format::Rgba8,
            capabilities,
            setup,
        )
        .unwrap()
    }

    #[test]
    fn backdrop_receipt_counts_source_steps_before_expanding_their_passes() {
        pollster::block_on(async {
            let (mut backend, identity) = backend().await;
            let capabilities = backend.device_capabilities(identity).unwrap();
            let setup = Account::for_test();
            let filters = crate::FilterList::try_ops(vec![
                crate::FilterOp::brightness(crate::FilterAmount::try_new(1.25).unwrap()),
                crate::FilterOp::blur(crate::FilterBlur::try_new(0.25).unwrap()),
                crate::FilterOp::drop_shadow(
                    crate::FilterDropShadow::try_new(
                        crate::Point::new(0.0, 0.0),
                        crate::FilterBlur::try_new(0.25).unwrap(),
                        Color::BLACK,
                    )
                    .unwrap(),
                ),
            ])
            .unwrap();
            let input = crate::BackdropFilterInput::try_new(
                filters,
                crate::BackdropCaptureBounds::try_new(crate::Rect::new(0.0, 0.0, 2.0, 2.0))
                    .unwrap(),
                None,
            )
            .unwrap();
            let mut scene = crate::Scene::new();
            scene.fill(crate::Rect::new(0.0, 0.0, 2.0, 2.0), Color::BLACK);
            scene.layer(
                crate::Layer::new().try_backdrop_filter(input).unwrap(),
                |child| {
                    child.fill(crate::Rect::new(0.0, 0.0, 1.0, 1.0), Color::BLACK);
                },
            );
            let lowered = lower_receipt_scene(scene, &capabilities, &setup);
            let prepared = backend
                .prepare_graph_resources(
                    identity,
                    lowered,
                    crate::EffectQualityPolicy::AllowReducedPrecision,
                    &setup,
                )
                .unwrap();
            let execution = prepared.backdrop_execution.as_ref().unwrap();
            let [backdrop] = execution.backdrops.as_slice() else {
                unreachable!()
            };
            assert!(matches!(
                backdrop.filter_steps.as_slice(),
                [
                    ExecutableFilterStepFacts::Color(_),
                    ExecutableFilterStepFacts::Blur { .. },
                    ExecutableFilterStepFacts::DropShadow { .. },
                ]
            ));
            assert_eq!(
                filter_step_passes(&backdrop.filter_steps, &setup)
                    .try_fold(0usize, |count, pass| pass.map(|_| count + 1))
                    .unwrap(),
                7
            );
            for limit in [3, 2] {
                let (result, report) =
                    measure_proof_work(RenderWork::GraphNodeVisits, limit, |account| {
                        backdrop_execution_receipt(&prepared, account)
                    });
                let attempt = report.attempt().unwrap();
                if limit == 3 {
                    let receipt = result.unwrap();
                    assert!(receipt.group_order_is_exact);
                    assert!(receipt.group_resources_are_distinct);
                    assert_eq!(attempt.usage().work(RenderWork::GraphNodeVisits), 3);
                } else {
                    assert!(
                        result.is_err(),
                        "the third source step must refuse before its match"
                    );
                    let refusal = attempt.refusal().unwrap();
                    assert_eq!(
                        refusal.quantity,
                        crate::RenderQuantity::Work(RenderWork::GraphNodeVisits)
                    );
                    assert_eq!((refusal.current, refusal.amount), (2, 1));
                }
                assert_eq!(
                    attempt.usage().work(RenderWork::ResourceAllocationRequests),
                    0
                );
            }
            let ExecutableFilterStepFacts::Color(first) = backdrop.filter_steps[0] else {
                unreachable!()
            };
            let (result, report) = measure_proof_work(RenderWork::GraphNodeVisits, 1, |account| {
                filter_step_passes(&backdrop.filter_steps, account)
                    .next()
                    .transpose()
            });
            assert_eq!(result.unwrap(), Some(first));
            // Dropping the unconsumed tail must not spend its two source visits.
            assert_eq!(
                report
                    .attempt()
                    .unwrap()
                    .usage()
                    .work(RenderWork::GraphNodeVisits),
                1
            );
            assert!(report.attempt().unwrap().refusal().is_none());
        });
    }

    #[test]
    fn normal_composite_entry_search_admits_four_entries_before_native_creation() {
        assert_composite_entry_search(0, 4);
    }

    #[test]
    fn clipped_masked_composite_entry_search_admits_six_entries_before_native_creation() {
        assert_composite_entry_search(1, 6);
    }

    #[test]
    fn destination_composite_entry_search_stops_at_the_third_entry() {
        assert_composite_entry_search(2, 3);
    }

    fn assert_composite_entry_search(mode: u8, exact: u64) {
        pollster::block_on(async {
            let (mut backend, identity) = backend().await;
            let capabilities = backend.device_capabilities(identity).unwrap();
            for limit in [exact, exact - 1] {
                let setup = Account::for_test();
                let lowered = match mode {
                    0 => lower_filter_proof_fixture(vec![
                        crate::FilterList::try_ops(vec![crate::FilterOp::brightness(
                            crate::FilterAmount::try_new(1.25).unwrap(),
                        )])
                        .unwrap(),
                    ]),
                    1 => lower_capture_proof_fixture(1, true, &capabilities),
                    _ => {
                        let bounds = crate::Rect::new(0.0, 0.0, 2.0, 2.0);
                        let image =
                            crate::Image::from_rgba(crate::Size::new(1.0, 1.0), vec![255; 4])
                                .unwrap();
                        let mask = crate::ResolvedLayerAlphaMask::try_new(image, bounds).unwrap();
                        let mut scene = crate::Scene::new();
                        scene.layer(
                            crate::Layer::new()
                                .blend(BlendMode::Multiply)
                                .with_resolved_alpha_mask(mask),
                            |child| {
                                child.fill(crate::Rect::new(0.0, 0.0, 1.0, 1.0), Color::BLACK);
                            },
                        );
                        lower_receipt_scene(scene, &capabilities, &setup)
                    }
                };
                let transaction = backend
                    .begin_gpu_operation(
                        identity,
                        crate::gpu_transaction::GpuOperationStage::Render,
                        crate::RuntimeOperation::EffectRendering,
                    )
                    .unwrap();
                let mut prepared = backend
                    .prepare_graph_resources(
                        identity,
                        lowered,
                        crate::EffectQualityPolicy::AllowReducedPrecision,
                        &setup,
                    )
                    .unwrap();
                let index = prepared.plan.passes.iter().position(|pass| {
                    matches!(&pass.runtime.kind, RuntimePassKind::Composite(Some(RuntimeComposite {
                        kind: RuntimeCompositeKind::Layer { parameters, clip_coverage, .. }, ..
                    })) if match mode {
                        0 => parameters.composition().is_source_over_normal() && parameters.alpha_mask().is_none() && clip_coverage.is_none(),
                        1 => parameters.composition().is_source_over_normal() && parameters.alpha_mask().is_some() && clip_coverage.is_some(),
                        _ => parameters.composition().blend_mode() == BlendMode::Multiply,
                    })
                }).unwrap();
                let request = PreparedPassEncodingRequest::from(&prepared.plan.passes[index]);
                // Select the real current-pass boundary for this isolated
                // constructor check, without claiming earlier passes encoded.
                prepared.next_pass = index;
                let semantic = PreparedGraph::layer_composite_semantic(&request, &setup).unwrap();
                let bindings = prepared
                    .layer_composite_bindings(&request, semantic, &setup)
                    .unwrap();
                assert_eq!(bindings.semantic.destination_path, mode == 2);
                assert_eq!(bindings.clip.is_some(), mode == 1);
                assert_eq!(bindings.mask.is_some(), mode != 0);
                let objects = prepared
                    .layer_composite_pass_objects(request.cache_keys.as_ref().unwrap(), &setup)
                    .unwrap();
                objects.require_encoding_ready().unwrap();
                let (result, report) =
                    measure_proof_work(RenderWork::ShaderBindingVisits, limit, |account| {
                        prepared
                            .create_layer_composite_bind_group(
                                &bindings,
                                &objects,
                                request.spatial_uniform.as_ref().unwrap(),
                                request.composite_parameters.as_ref().unwrap(),
                                request.reads.len(),
                                account,
                            )
                            .map(|(group, parent_sample)| {
                                drop(group);
                                parent_sample
                            })
                    });
                let attempt = report.attempt().unwrap();
                if limit == exact {
                    assert_eq!(result.unwrap(), mode == 2);
                    assert_eq!(attempt.usage().work(RenderWork::ShaderBindingVisits), exact);
                } else {
                    assert!(
                        result.is_err(),
                        "the next real binding entry must refuse before native creation"
                    );
                    let refusal = attempt.refusal().unwrap();
                    assert_eq!(
                        refusal.quantity,
                        crate::RenderQuantity::Work(RenderWork::ShaderBindingVisits)
                    );
                    assert_eq!((refusal.current, refusal.amount), (exact - 1, 1));
                }
                // Two uniform buffers are constructed before this inspection;
                // only a successful inspection permits the third constructor.
                assert_eq!(
                    attempt.usage().work(RenderWork::ResourceAllocationRequests),
                    if limit == exact { 3 } else { 2 }
                );
                // Returned native creation events belong to the generation;
                // the attempt records only forward constructor requests.
                assert_eq!(attempt.usage().work(RenderWork::GpuObjectsCreated), 0);
                drop(request);
                drop(prepared);
                transaction
                    .finish(crate::RuntimeOperation::EffectRendering, Some(&setup))
                    .await
                    .unwrap();
            }
        });
    }

    #[test]
    fn completed_release_witness_binds_the_actual_pass_and_canonical_sources() {
        pollster::block_on(async {
            let (mut backend, identity) = backend().await;
            let capabilities = backend.device_capabilities(identity).unwrap();
            for alteration in 0..3 {
                let lowered = lower_capture_proof_fixture(1, false, &capabilities);
                let setup = Account::for_test();
                let mut prepared = backend
                    .prepare_graph_resources(
                        identity,
                        lowered,
                        crate::EffectQualityPolicy::AllowReducedPrecision,
                        &setup,
                    )
                    .unwrap();
                let index = prepared
                    .plan
                    .passes
                    .iter()
                    .position(|pass| {
                        matches!(pass.runtime.kind, RuntimePassKind::CanonicalizeCapture)
                    })
                    .unwrap();
                let mut request = PreparedPassEncodingRequest::from(&prepared.plan.passes[index]);
                let actual_pass = request.id;
                let resource = request.releases[0];
                if alteration == 1 {
                    request.id = prepared.plan.passes[0].runtime.id;
                } else if alteration == 2 {
                    let mut replacement = setup.retained(request.releases.len()).unwrap();
                    replacement.extend_from_slice(&request.releases).unwrap();
                    request.releases =
                        SharedValue::from_accounted(replacement, &setup.storage()).unwrap();
                }
                prepared.next_pass = index;
                let completed = prepared.complete_pass_inner(actual_pass, &setup).unwrap();
                let proof = completed.prove_for(&request);
                assert_eq!(proof.resources, alteration == 0);
                assert_eq!(proof.kernels, alteration == 0);
                assert!(
                    !proof.color_buffer,
                    "canonicalization has no color-operation buffer"
                );
                assert_eq!(prepared.next_pass, index + 1);
                assert!(
                    prepared
                        .resource_bindings
                        .get(&resource, &setup)
                        .unwrap()
                        .unwrap()
                        .lease
                        .is_none()
                );
            }
        });
    }

    #[test]
    fn repeated_proof_release_preflight_counts_texture_and_kernel_edges_before_atomic_release() {
        pollster::block_on(async {
            let (mut backend, identity) = backend().await;
            for limit in [4, 3] {
                let filters = crate::FilterList::try_ops(vec![crate::FilterOp::blur(
                    crate::FilterBlur::try_new(0.25).unwrap(),
                )])
                .unwrap();
                let lowered = lower_filter_proof_fixture(vec![filters]);
                let setup = Account::for_test();
                let mut prepared = backend
                    .prepare_graph_resources(
                        identity,
                        lowered,
                        crate::EffectQualityPolicy::AllowReducedPrecision,
                        &setup,
                    )
                    .unwrap();
                let index = prepared
                    .plan
                    .passes
                    .iter()
                    .position(|request| {
                        matches!(request.runtime.kind, RuntimePassKind::BlurVertical(Some(_)))
                    })
                    .unwrap();
                let request = &prepared.plan.passes[index];
                assert_eq!(request.runtime.releases.len(), 1);
                assert_eq!(request.kernel_releases.len(), 1);
                let pass = request.runtime.id;
                let resource = request.runtime.releases[0];
                let kernel = request.kernel_releases[0];
                // Select the real prepared release boundary without claiming
                // that this isolated fixture encoded the preceding frame.
                prepared.next_pass = index;
                let generation = prepared.resources.generation().clone();
                let cache_custody = || {
                    [
                        generation.cache_bytes().unwrap(),
                        generation.registered_cache_bytes().unwrap(),
                        generation.idle_cache_bytes().unwrap(),
                    ]
                };
                let cache_before = cache_custody();
                let probes_before = generation.cache_work(RenderWork::ResourceProbes).unwrap();
                let returns = |class| {
                    generation
                        .class_usage(class)
                        .unwrap()
                        .work(RenderWork::ResourceLeaseReturns)
                };
                let texture_before = returns(GenerationClass::CacheEffects);
                let kernel_before = returns(GenerationClass::CacheKernels);
                let (result, report) =
                    measure_proof_work(RenderWork::GraphEdgeVisits, limit, |account| {
                        prepared
                            .complete_pass_inner(pass, &account.at(RenderPhase::Encode))
                            .map(|completed| completed.pass)
                    });
                let completed = u64::from(limit == 4);
                assert_eq!(cache_custody(), cache_before);
                assert_eq!(
                    generation.cache_work(RenderWork::ResourceProbes).unwrap() - probes_before,
                    completed * 4
                );
                assert_eq!(
                    returns(GenerationClass::CacheEffects) - texture_before,
                    completed
                );
                assert_eq!(
                    returns(GenerationClass::CacheKernels) - kernel_before,
                    completed
                );
                if limit == 4 {
                    result.unwrap();
                    // Each of the two relationships is inspected once while
                    // validating and once while assembling the atomic input.
                    assert_eq!(
                        report
                            .attempt()
                            .unwrap()
                            .usage()
                            .work(RenderWork::GraphEdgeVisits),
                        4
                    );
                    assert_eq!(prepared.next_pass, index + 1);
                    assert!(
                        prepared
                            .resource_bindings
                            .get(&resource, &setup)
                            .unwrap()
                            .unwrap()
                            .lease
                            .is_none()
                    );
                    assert!(
                        prepared
                            .kernel_bindings
                            .get(&kernel, &setup)
                            .unwrap()
                            .unwrap()
                            .lease
                            .is_none()
                    );
                } else {
                    assert!(
                        result.is_err(),
                        "the final input edge must refuse before atomic resolution"
                    );
                    let refusal = report.attempt().unwrap().refusal().unwrap();
                    assert_eq!(
                        refusal.quantity,
                        crate::RenderQuantity::Work(RenderWork::GraphEdgeVisits)
                    );
                    assert_eq!((refusal.current, refusal.amount), (3, 1));
                    assert_eq!(prepared.next_pass, index);
                    assert!(
                        prepared
                            .resource_bindings
                            .get(&resource, &setup)
                            .unwrap()
                            .unwrap()
                            .lease
                            .is_some()
                    );
                    assert!(
                        prepared
                            .kernel_bindings
                            .get(&kernel, &setup)
                            .unwrap()
                            .unwrap()
                            .lease
                            .is_some()
                    );
                }
                assert_eq!(
                    report
                        .attempt()
                        .unwrap()
                        .usage()
                        .work(RenderWork::Submissions),
                    0
                );
            }
        });
    }

    #[test]
    fn prepared_color_release_admits_its_exact_forward_prefix() {
        use crate::accounting::{FrameFixture, RenderCpuScope, RenderProfile};
        use crate::{RenderAccountingError, RenderQuantity, RenderRefusalReason, RenderResource};

        pollster::block_on(async {
            let (mut backend, identity) = backend().await;
            let options = RenderAccountingOptions::new()
                .with_profile(RenderProfile::Summary { timing: false });
            let preparation_cpu = RendererCpu::new(options).unwrap();
            let filters = crate::FilterList::try_ops(vec![crate::FilterOp::brightness(
                crate::FilterAmount::try_new(1.25).unwrap(),
            )])
            .unwrap();
            let lowered = lower_filter_proof_fixture(vec![filters]);
            let (result, _, _, fault) = FrameFixture::new(&preparation_cpu, options)
                .run(async |attempt| {
                    let setup = attempt.account();
                    let transaction = backend.begin_gpu_operation(
                        identity,
                        crate::gpu_transaction::GpuOperationStage::Render,
                        crate::RuntimeOperation::EffectRendering,
                    )?;
                    let mut prepared = backend.prepare_graph_resources(
                        identity,
                        lowered,
                        crate::EffectQualityPolicy::AllowReducedPrecision,
                        setup,
                    )?;
                    let index = prepared
                        .plan
                        .passes
                        .iter()
                        .position(|request| {
                            matches!(request.runtime.kind, RuntimePassKind::ColorFilter(Some(_)))
                        })
                        .unwrap();
                    let request = &prepared.plan.passes[index];
                    assert_eq!(request.runtime.releases.len(), 1);
                    assert!(request.kernel_releases.is_empty());
                    let pass = request.runtime.id;
                    let source = request.runtime.releases[0];
                    assert_eq!(prepared.resource_bindings.len(), 7);
                    assert_eq!(prepared.resource_bindings.entries[4].0, source);
                    assert_eq!(prepared.color_filter_operation_bindings.len(), 1);
                    assert_eq!(prepared.color_filter_operation_bindings.entries[0].0, pass);
                    let source_id = prepared.resource_bindings.entries[4]
                        .1
                        .lease
                        .as_ref()
                        .unwrap()
                        .resource_identity();
                    assert_eq!(
                        prepared.color_filter_operation_bindings.entries[0]
                            .1
                            .buffer
                            .as_ref()
                            .unwrap()
                            .size(),
                        48
                    );
                    prepared.next_pass = index;

                    let generation = prepared.resources.generation().clone();
                    let cache_custody = || {
                        [
                            generation.cache_bytes().unwrap(),
                            generation.registered_cache_bytes().unwrap(),
                            generation.idle_cache_bytes().unwrap(),
                        ]
                    };
                    let buffer_bytes = || {
                        generation
                            .class_usage(GenerationClass::PassBuffers)
                            .unwrap()
                            .live(RenderResource::GpuBytes)
                    };
                    let texture_returns = || {
                        generation
                            .class_usage(GenerationClass::CacheEffects)
                            .unwrap()
                            .work(RenderWork::ResourceLeaseReturns)
                    };
                    let cleanup_probes = || {
                        preparation_cpu
                            .snapshot()
                            .unwrap()
                            .scope_usage(RenderCpuScope::Cleanup)
                            .unwrap()
                            .work(RenderWork::ResourceProbes)
                    };
                    // A search for index four among seven sorted keys compares
                    // indices three, five and four. The color map has one key.
                    // Two source searches and one color search need seven probes.
                    for limit in [6, 7] {
                        let cache_before = cache_custody();
                        let buffer_before = buffer_bytes();
                        let returns_before = texture_returns();
                        let probes_before =
                            generation.cache_work(RenderWork::ResourceProbes).unwrap();
                        let cleanup_before = cleanup_probes();
                        let (result, report) =
                            measure_proof_work(RenderWork::ResourceProbes, limit, |account| {
                                prepared
                                    .complete_pass_inner(pass, &account.at(RenderPhase::Encode))
                                    .map(|receipt| (receipt.pass, receipt.color_buffer_taken))
                            });
                        let attempt = report.attempt().unwrap();
                        let usage = attempt.usage();
                        assert_eq!(usage.work(RenderWork::GraphNodeVisits), 1);
                        assert_eq!(usage.work(RenderWork::GraphEdgeVisits), 2);
                        assert_eq!(usage.work(RenderWork::ResourceProbes), limit);
                        let succeeded = limit == 7;
                        if succeeded {
                            assert_eq!(result.unwrap(), (pass, true));
                            assert_eq!(attempt.refusal(), None);
                        } else {
                            let refusal = attempt.refusal().unwrap();
                            assert_eq!(refusal.phase(), RenderPhase::Attempt);
                            assert_eq!(
                                refusal.quantity(),
                                RenderQuantity::Work(RenderWork::ResourceProbes)
                            );
                            assert_eq!((refusal.current(), refusal.amount()), (6, 1));
                            assert_eq!(refusal.reason(), RenderRefusalReason::Limit { limit: 6 });
                            assert_eq!(
                                result.unwrap_err().accounting_diagnostic(),
                                Some(RenderAccountingError::Refused(refusal))
                            );
                        }
                        assert_eq!(prepared.next_pass, index + usize::from(succeeded));
                        assert_eq!(
                            prepared.resource_bindings.entries[4]
                                .1
                                .lease
                                .as_ref()
                                .map(|lease| lease.resource_identity()),
                            (!succeeded).then_some(source_id)
                        );
                        assert_eq!(
                            prepared.color_filter_operation_bindings.entries[0]
                                .1
                                .buffer
                                .as_ref()
                                .map(|buffer| buffer.size()),
                            (!succeeded).then_some(48)
                        );
                        assert_eq!(cache_custody(), cache_before);
                        assert_eq!(buffer_before - buffer_bytes(), 48 * u64::from(succeeded));
                        assert_eq!(texture_returns() - returns_before, u64::from(succeeded));
                        assert_eq!(
                            generation.cache_work(RenderWork::ResourceProbes).unwrap()
                                - probes_before,
                            2 * u64::from(succeeded)
                        );
                        assert_eq!(cleanup_probes() - cleanup_before, 4 * u64::from(succeeded));
                    }
                    drop(prepared);
                    transaction
                        .finish(crate::RuntimeOperation::EffectRendering, Some(setup))
                        .await
                })
                .await
                .unwrap()
                .into_parts();
            result.unwrap();
            assert_eq!(fault, None);
        });
    }

    #[test]
    fn late_release_fault_continues_gaussian_and_color_takes() {
        use crate::RenderResource;
        use crate::accounting::{FrameFixture, RenderCpuScope, RenderProfile};
        use surgeist_ledger::Dimension;

        enum Case {
            LaterKernel,
            MissingKernel,
            LaterColorBuffer,
        }

        pollster::block_on(async {
            let (mut backend, identity) = backend().await;
            let options = RenderAccountingOptions::new()
                .with_profile(RenderProfile::Summary { timing: false });
            let preparation_cpu = RendererCpu::new(options).unwrap();
            for case in [
                Case::LaterKernel,
                Case::MissingKernel,
                Case::LaterColorBuffer,
            ] {
                let color = matches!(case, Case::LaterColorBuffer);
                let filter = if color {
                    crate::FilterOp::brightness(crate::FilterAmount::try_new(1.25).unwrap())
                } else {
                    crate::FilterOp::blur(crate::FilterBlur::try_new(0.25).unwrap())
                };
                let lowered = lower_filter_proof_fixture(vec![
                    crate::FilterList::try_ops(vec![filter]).unwrap(),
                ]);
                let (result, _, _, fault) = FrameFixture::new(&preparation_cpu, options)
                    .run(async |attempt| {
                        let setup = attempt.account();
                        let transaction = backend.begin_gpu_operation(
                            identity,
                            crate::gpu_transaction::GpuOperationStage::Render,
                            crate::RuntimeOperation::EffectRendering,
                        )?;
                        let mut prepared = backend.prepare_graph_resources(
                            identity,
                            lowered,
                            crate::EffectQualityPolicy::AllowReducedPrecision,
                            setup,
                        )?;
                        let index = prepared
                            .plan
                            .passes
                            .iter()
                            .position(|request| {
                                if color {
                                    matches!(
                                        request.runtime.kind,
                                        RuntimePassKind::ColorFilter(Some(_))
                                    )
                                } else {
                                    matches!(
                                        request.runtime.kind,
                                        RuntimePassKind::BlurVertical(Some(_))
                                    )
                                }
                            })
                            .unwrap();
                        let request = &prepared.plan.passes[index];
                        assert_eq!(request.runtime.releases.len(), 1);
                        assert_eq!(request.kernel_releases.len(), usize::from(!color));
                        let pass = request.runtime.id;
                        let source = request.runtime.releases[0];
                        let kernel = request.kernel_releases.first().cloned();
                        // Brightness follows the inner composite at index four.
                        // Vertical blur follows its horizontal intermediate at five.
                        let source_index = if color { 4 } else { 5 };
                        assert_eq!(prepared.resource_bindings.len(), if color { 7 } else { 8 });
                        assert_eq!(prepared.resource_bindings.entries[source_index].0, source);
                        assert_eq!(prepared.kernel_bindings.len(), usize::from(!color));
                        assert_eq!(
                            prepared.color_filter_operation_bindings.len(),
                            usize::from(color)
                        );
                        prepared.next_pass = index;

                        let generation = prepared.resources.generation().clone();
                        let cache_custody = || {
                            [
                                generation.cache_bytes().unwrap(),
                                generation.registered_cache_bytes().unwrap(),
                                generation.idle_cache_bytes().unwrap(),
                            ]
                        };
                        let operation_bytes = || {
                            generation
                                .class_usage(GenerationClass::PassBuffers)
                                .unwrap()
                                .live(RenderResource::GpuBytes)
                        };
                        let cleanup_probes = || {
                            preparation_cpu
                                .snapshot()
                                .unwrap()
                                .scope_usage(RenderCpuScope::Cleanup)
                                .unwrap()
                                .work(RenderWork::ResourceProbes)
                        };
                        let returns_before =
                            generation.cache_work(RenderWork::ResourceLeaseReturns)?;
                        let resolved =
                            ResolvedPassReleases::try_resolve(&mut prepared, pass, setup)?;
                        assert_eq!(
                            generation.cache_work(RenderWork::ResourceLeaseReturns)?
                                - returns_before,
                            if color { 1 } else { 2 }
                        );
                        // The private receipt permits exercising an unavailable
                        // slot after real atomic resolution. Leave its key in
                        // the original map and observe the scheduled takes alone.
                        drop(
                            resolved
                                .resource_bindings
                                .maintenance_get_mut(&source)
                                .unwrap()
                                .lease
                                .take()
                                .unwrap(),
                        );
                        if matches!(case, Case::MissingKernel) {
                            drop(
                                resolved
                                    .kernel_bindings
                                    .maintenance_get_mut(kernel.as_ref().unwrap())
                                    .unwrap()
                                    .lease
                                    .take()
                                    .unwrap(),
                            );
                        }
                        let cache_before = cache_custody();
                        let bytes_before = operation_bytes();
                        let work_before = generation.usage()?;
                        let cleanup_before = cleanup_probes();
                        let probes_before = setup.work_count(RenderWork::ResourceProbes)?;
                        let edges_before = setup.work_count(RenderWork::GraphEdgeVisits)?;

                        let error = resolved.take_bindings().err().unwrap();
                        assert_eq!(error.code(), crate::ErrorCode::RenderFailed);
                        assert_eq!(
                            error.message(),
                            "prepared resource release is unavailable after atomic resolution"
                        );
                        assert_eq!(prepared.next_pass, index);
                        assert!(
                            prepared.resource_bindings.entries[source_index]
                                .1
                                .lease
                                .is_none()
                        );
                        if color {
                            assert!(
                                prepared.color_filter_operation_bindings.entries[0]
                                    .1
                                    .buffer
                                    .is_none()
                            );
                        } else {
                            assert!(prepared.kernel_bindings.entries[0].1.lease.is_none());
                        }
                        // Both source searches take three comparisons. The
                        // single later kernel or color binding adds one.
                        assert_eq!(cleanup_probes() - cleanup_before, 4);
                        assert_eq!(setup.work_count(RenderWork::ResourceProbes)?, probes_before);
                        assert_eq!(setup.work_count(RenderWork::GraphEdgeVisits)?, edges_before);
                        assert_eq!(cache_custody(), cache_before);
                        assert_eq!(bytes_before - operation_bytes(), if color { 48 } else { 0 });
                        let work_after = generation.usage()?;
                        for &work in RenderWork::ALL {
                            assert_eq!(work_after.work(work), work_before.work(work));
                        }
                        drop(prepared);
                        assert_eq!(cache_custody(), [0, 0, 0]);
                        transaction
                            .finish(crate::RuntimeOperation::EffectRendering, Some(setup))
                            .await
                    })
                    .await
                    .unwrap()
                    .into_parts();
                result.unwrap();
                assert_eq!(fault, None);
            }
        });
    }

    #[test]
    fn repeated_proof_color_completion_admits_all_probes_before_atomic_release() {
        assert_color_completion_work(RenderWork::ResourceProbes, 40);
    }

    #[test]
    fn repeated_proof_color_completion_admits_the_sampler_and_nested_bindings() {
        // One source-sampler comparison, followed by checked shader bindings:
        // 1 texture + 1 sampler + 2 data + 3 layout + 1 shader sampler +
        // 4 pipeline shader + 3 pipeline layout + 1 pipeline sampler = 17.
        assert_color_completion_work(RenderWork::ShaderBindingVisits, 17);
    }

    #[test]
    fn repeated_proof_color_completion_admits_serialized_operation_comparisons() {
        // Rebuild one operation, compare its serialized record once, and use
        // the original immutable byte allocation for the binding certificate.
        assert_color_completion_work(RenderWork::FilterOperationVisits, 2);
    }

    fn assert_color_completion_work(work: RenderWork, exact: u64) {
        pollster::block_on(async {
            let (mut backend, identity) = backend().await;
            for limit in [exact, exact - 1] {
                let filters = crate::FilterList::try_ops(vec![crate::FilterOp::brightness(
                    crate::FilterAmount::try_new(1.25).unwrap(),
                )])
                .unwrap();
                let lowered = lower_filter_proof_fixture(vec![filters]);
                let setup = Account::for_test();
                // The backend enables checked shader realization only while
                // its real GPU operation owns the active generation/scopes.
                let transaction = backend
                    .begin_gpu_operation(
                        identity,
                        crate::gpu_transaction::GpuOperationStage::Render,
                        crate::RuntimeOperation::EffectRendering,
                    )
                    .unwrap();
                let mut prepared = backend
                    .prepare_graph_resources(
                        identity,
                        lowered,
                        crate::EffectQualityPolicy::AllowReducedPrecision,
                        &setup,
                    )
                    .unwrap();
                let index = prepared
                    .plan
                    .passes
                    .iter()
                    .position(|request| {
                        matches!(request.runtime.kind, RuntimePassKind::ColorFilter(Some(_)))
                    })
                    .unwrap();
                let request = PreparedPassEncodingRequest::from(&prepared.plan.passes[index]);
                assert_eq!(
                    prepared.custom_spine_encoding_state,
                    Some(CustomSpineEncodingState::Ready)
                );
                assert!(prepared.pass_cache_update.is_some());
                prepared
                    .color_filter_pass_objects(request.cache_keys.as_ref().unwrap(), &setup)
                    .unwrap()
                    .require_encoding_ready()
                    .unwrap();
                let source = request.reads[0].resource;
                let RuntimeResultBinding::Resource(target) = request.result else {
                    unreachable!()
                };
                // build_authored_filter_fixture declares root0, then
                // plan_layer_source declares transparent parent1 and flushes
                // capture2 -> canonical3 -> inner composite4. The filter
                // creates result5, followed by the outer composite6.
                assert_eq!(prepared.plan.resources.len(), 7);
                assert!(
                    prepared
                        .plan
                        .resources
                        .iter()
                        .map(|resource| resource.runtime.role)
                        .eq([
                            RuntimeResourceRole::RootWorkingImage,
                            RuntimeResourceRole::IsolationWorkingImage,
                            RuntimeResourceRole::CaptureWorkingImage,
                            RuntimeResourceRole::FilterIntermediate,
                            RuntimeResourceRole::CompositeResult,
                            RuntimeResourceRole::FilterIntermediate,
                            RuntimeResourceRole::CompositeResult,
                        ])
                );
                assert_eq!(prepared.plan.resources[4].runtime.id, source);
                assert_eq!(prepared.plan.resources[5].runtime.id, target);
                assert_eq!(prepared.resource_bindings.entries.len(), 7);
                assert_eq!(prepared.resource_bindings.entries[4].0, source);
                assert_eq!(prepared.resource_bindings.entries[5].0, target);
                assert_eq!(prepared.color_filter_operation_bindings.len(), 1);
                prepared.next_pass = index;
                // The isolated step has the same active scheduler state as
                // its real encode_custom_spine caller; no earlier pass receipt
                // or complete-frame proof is fabricated by this fixture.
                prepared.custom_spine_encoding_state = Some(CustomSpineEncodingState::Encoding);
                let mut progress =
                    CustomSpineEncodingProgress::new(prepared.plan.passes.len(), 1, &setup)
                        .unwrap();
                let mut encoder = prepared
                    .resources
                    .generation()
                    .opaque(GpuObjectKind::CommandEncoder, Some(&setup), || {
                        prepared
                            .device
                            .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                                label: Some("Surgeist isolated color completion receipt"),
                            })
                    })
                    .unwrap();
                let (result, report) = measure_proof_work(work, limit, |account| {
                    prepared.encode_color_filter_step(
                        &mut encoder,
                        &request,
                        &mut progress,
                        &account.at(RenderPhase::Encode),
                    )
                });
                if limit == exact {
                    result.unwrap();
                    // Read-role lookup1; seven-entry texture maps3+2;
                    // resource scans(5+6)*2; operation binding1; four
                    // provisional shader lookups; release preflight3+1
                    // and lease-input3: 1+5+22+1+4+4+3 = 40. A
                    // completed-release witness needs no later map lookup.
                    assert_eq!(report.attempt().unwrap().usage().work(work), exact);
                    assert_eq!(progress.completed_pass_count, 1);
                    assert!(progress.color_filter_operation_buffers_released);
                    assert_eq!(prepared.next_pass, index + 1);
                    assert!(
                        prepared
                            .resource_bindings
                            .get(&source, &setup)
                            .unwrap()
                            .unwrap()
                            .lease
                            .is_none()
                    );
                    assert!(
                        prepared
                            .color_filter_operation_bindings
                            .get(&request.id, &setup)
                            .unwrap()
                            .unwrap()
                            .buffer
                            .is_none()
                    );
                } else {
                    assert!(
                        result.is_err(),
                        "the next inspection must refuse before atomic release"
                    );
                    let refusal = report.attempt().unwrap().refusal().unwrap();
                    assert_eq!(refusal.quantity, crate::RenderQuantity::Work(work));
                    assert_eq!((refusal.current, refusal.amount), (exact - 1, 1));
                    assert_eq!(progress.completed_pass_count, 0);
                    assert_eq!(prepared.next_pass, index);
                    assert!(
                        prepared
                            .resource_bindings
                            .get(&source, &setup)
                            .unwrap()
                            .unwrap()
                            .lease
                            .is_some()
                    );
                    assert!(
                        prepared
                            .color_filter_operation_bindings
                            .get(&request.id, &setup)
                            .unwrap()
                            .unwrap()
                            .buffer
                            .is_some()
                    );
                }
                // Probe refusal follows native encoding. Binding/operation
                // refusal precedes it. All refusals precede atomic release.
                assert_eq!(
                    report
                        .attempt()
                        .unwrap()
                        .usage()
                        .work(RenderWork::ImagePasses),
                    u64::from(limit == exact || work == RenderWork::ResourceProbes)
                );
                assert_eq!(
                    report
                        .attempt()
                        .unwrap()
                        .usage()
                        .work(RenderWork::Submissions),
                    0
                );
                drop(encoder);
                drop(prepared);
                transaction
                    .finish(crate::RuntimeOperation::EffectRendering, Some(&setup))
                    .await
                    .unwrap();
            }
        });
    }

    #[test]
    fn repeated_proof_capture_handoff_counts_two_and_three_candidates() {
        pollster::block_on(async {
            let (mut backend, identity) = backend().await;
            let capabilities = backend.device_capabilities(identity).unwrap();
            let lowered = lower_capture_proof_fixture(3, false, &capabilities);
            let setup = Account::for_test();
            let mut prepared = backend
                .prepare_graph_resources(
                    identity,
                    lowered,
                    crate::EffectQualityPolicy::AllowReducedPrecision,
                    &setup,
                )
                .unwrap();
            assert_eq!(
                prepared
                    .composition_execution
                    .as_ref()
                    .unwrap()
                    .captures
                    .len(),
                3
            );
            let captures = prepared
                .plan
                .passes
                .iter()
                .enumerate()
                .filter_map(|(index, request)| {
                    matches!(request.runtime.kind, RuntimePassKind::VelloCapture(Some(_)))
                        .then_some(index)
                })
                .collect::<Vec<_>>();
            assert_eq!(captures.len(), 3);
            let session = Arc::new(());
            for (index, expected) in [(captures[1], 3), (captures[2], 4)] {
                // This boundary returns an unencoded handoff, not a completion
                // receipt. Select its current prepared pass without inventing one.
                prepared.next_pass = index;
                let request = PreparedPassEncodingRequest::from(&prepared.plan.passes[index]);
                let (result, report) =
                    measure_proof_work(RenderWork::GraphNodeVisits, expected, |account| {
                        let account = account.at(RenderPhase::Encode);
                        prepared
                            .vello_capture_handoff(&request, &session, &account)
                            .map(|handoff| handoff.target())
                    });
                let RuntimeResultBinding::Resource(target) = request.result else {
                    unreachable!()
                };
                assert_eq!(result.unwrap(), target);
                // Two/three candidate facts, then the existing current-pass node.
                assert_eq!(
                    report
                        .attempt()
                        .unwrap()
                        .usage()
                        .work(RenderWork::GraphNodeVisits),
                    expected
                );
                assert_eq!(prepared.next_pass, index);
            }
        });
    }

    #[test]
    fn repeated_proof_capture_one_under_refuses_before_any_handoff() {
        pollster::block_on(async {
            let (mut backend, identity) = backend().await;
            let capabilities = backend.device_capabilities(identity).unwrap();
            let lowered = lower_capture_proof_fixture(3, false, &capabilities);
            let setup = Account::for_test();
            let mut prepared = backend
                .prepare_graph_resources(
                    identity,
                    lowered,
                    crate::EffectQualityPolicy::AllowReducedPrecision,
                    &setup,
                )
                .unwrap();
            let index = prepared
                .plan
                .passes
                .iter()
                .rposition(|request| {
                    matches!(request.runtime.kind, RuntimePassKind::VelloCapture(Some(_)))
                })
                .unwrap();
            prepared.next_pass = index;
            let request = PreparedPassEncodingRequest::from(&prepared.plan.passes[index]);
            let session = Arc::new(());
            let (result, report) = measure_proof_work(RenderWork::GraphNodeVisits, 3, |account| {
                let account = account.at(RenderPhase::Encode);
                let error = match prepared.vello_capture_handoff(&request, &session, &account) {
                    Ok(_) => panic!(
                        "three candidate facts plus the current-pass node require four visits"
                    ),
                    Err(error) => error,
                };
                assert!(
                    prepared
                        .vello_capture_handoff(&request, &session, &account)
                        .is_err(),
                    "an exhausted attempt cannot return a later handoff"
                );
                Err::<(), _>(error)
            });
            assert!(result.is_err());
            let attempt = report.attempt().unwrap();
            let refusal = attempt.refusal().unwrap();
            assert_eq!(
                refusal.quantity,
                crate::RenderQuantity::Work(RenderWork::GraphNodeVisits)
            );
            assert_eq!((refusal.current, refusal.amount), (3, 1));
            assert_eq!(
                attempt.usage().work(RenderWork::ResourceProbes),
                0,
                "no texture binding may be inspected after refusal"
            );
            assert_eq!(attempt.usage().work(RenderWork::Submissions), 0);
            assert_eq!(prepared.next_pass, index);
        });
    }

    #[test]
    fn repeated_proof_clip_copy_uses_relationship_visits_before_path_encoding() {
        pollster::block_on(async {
            let (mut backend, identity) = backend().await;
            let capabilities = backend.device_capabilities(identity).unwrap();
            let lowered = lower_capture_proof_fixture(1, true, &capabilities);
            let setup = Account::for_test();
            let mut prepared = backend
                .prepare_graph_resources(
                    identity,
                    lowered,
                    crate::EffectQualityPolicy::AllowReducedPrecision,
                    &setup,
                )
                .unwrap();
            let index = prepared
                .plan
                .passes
                .iter()
                .position(|request| {
                    matches!(
                        request.runtime.kind,
                        RuntimePassKind::VelloCapture(Some(RuntimeVelloCapture::ClipCoverage(_)))
                    )
                })
                .unwrap();
            prepared.next_pass = index;
            let request = PreparedPassEncodingRequest::from(&prepared.plan.passes[index]);
            let session = Arc::new(());
            let handoff = prepared
                .vello_capture_handoff(&request, &session, &setup)
                .unwrap();
            let RuntimeVelloCapture::ClipCoverage(coverage) = handoff.work() else {
                unreachable!()
            };
            assert_eq!(coverage.elements.len(), 2);
            for limit in [2, 1] {
                let (result, report) =
                    measure_proof_work(RenderWork::GraphEdgeVisits, limit, |account| {
                        vello_capture_scene(&handoff, account).map(|_scene| ())
                    });
                if limit == 2 {
                    result.unwrap();
                    assert_eq!(
                        report
                            .attempt()
                            .unwrap()
                            .usage()
                            .work(RenderWork::GraphEdgeVisits),
                        2
                    );
                } else {
                    assert!(
                        result.is_err(),
                        "the second clip relationship must refuse before geometry encoding"
                    );
                    let attempt = report.attempt().unwrap();
                    let refusal = attempt.refusal().unwrap();
                    assert_eq!(
                        refusal.quantity,
                        crate::RenderQuantity::Work(RenderWork::GraphEdgeVisits)
                    );
                    assert_eq!((refusal.current, refusal.amount), (1, 1));
                    assert_eq!(attempt.usage().work(RenderWork::PathElementVisits), 0);
                }
            }
        });
    }
}
