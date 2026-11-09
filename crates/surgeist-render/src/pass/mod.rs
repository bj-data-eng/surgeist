mod close;
mod encode;
mod lower;
mod model;
mod parameters;
mod prepare;
#[cfg(test)]
mod test_support;

use encode::CustomSpineEncodingState;

/// A private sorted explicit entry allocation. Callers reserve a graph-derived
/// upper bound before forward native work; insertion then moves only owned data.
pub(super) struct PassMap<K, V> {
    entries: crate::accounting::AccountedVec<(K, V)>,
}
impl<K: Ord, V> PassMap<K, V> {
    fn new(account: &crate::accounting::Account, count: usize) -> Result<Self> {
        Ok(Self {
            entries: account.retained(count)?,
        })
    }
    fn temporary(account: &crate::accounting::Account, count: usize) -> Result<Self> {
        Ok(Self {
            entries: account.temporary(count)?,
        })
    }
    fn maintenance_position(&self, key: &K) -> std::result::Result<usize, usize> {
        let mut low = 0;
        let mut high = self.entries.len();
        while low < high {
            let middle = low + (high - low) / 2;
            self.entries
                .renderer_maintenance(crate::accounting::RenderWork::ResourceProbes, 1);
            match self.entries[middle].0.cmp(key) {
                std::cmp::Ordering::Less => low = middle + 1,
                std::cmp::Ordering::Greater => high = middle,
                std::cmp::Ordering::Equal => return Ok(middle),
            }
        }
        Err(low)
    }
    fn position(
        &self,
        key: &K,
        account: &crate::accounting::Account,
    ) -> Result<std::result::Result<usize, usize>> {
        let mut low = 0;
        let mut high = self.entries.len();
        while low < high {
            let middle = low + (high - low) / 2;
            account.work(crate::accounting::RenderWork::ResourceProbes, 1)?;
            match self.entries[middle].0.cmp(key) {
                std::cmp::Ordering::Less => low = middle + 1,
                std::cmp::Ordering::Greater => high = middle,
                std::cmp::Ordering::Equal => return Ok(Ok(middle)),
            }
        }
        Ok(Err(low))
    }
    pub(super) fn get(&self, key: &K, account: &crate::accounting::Account) -> Result<Option<&V>> {
        Ok(self
            .position(key, account)?
            .ok()
            .map(|index| &self.entries[index].1))
    }
    #[cfg(test)]
    pub(super) fn get_mut(
        &mut self,
        key: &K,
        account: &crate::accounting::Account,
    ) -> Result<Option<&mut V>> {
        Ok(self
            .position(key, account)?
            .ok()
            .map(|index| &mut self.entries[index].1))
    }
    pub(super) fn remove_with(
        &mut self,
        key: &K,
        account: &crate::accounting::Account,
    ) -> Result<Option<V>> {
        let Ok(index) = self.position(key, account)? else {
            return Ok(None);
        };
        self.entries[index..].rotate_left(1);
        Ok(self.entries.pop().map(|(_, value)| value))
    }
    pub(super) fn maintenance_get_mut(&mut self, key: &K) -> Option<&mut V> {
        self.maintenance_position(key)
            .ok()
            .map(|index| &mut self.entries[index].1)
    }
    #[cfg(test)]
    fn insert(&mut self, key: K, value: V) -> Option<V> {
        match self.maintenance_position(&key) {
            Ok(index) => Some(std::mem::replace(&mut self.entries[index].1, value)),
            Err(index) => {
                self.entries.insert_slot(index).unwrap().write((key, value));
                None
            }
        }
    }
    fn insert_at_reserved(&mut self, index: usize, key: K, value: V) -> Result<()> {
        self.entries.insert_slot(index)?.write((key, value));
        Ok(())
    }
    fn insert_with(
        &mut self,
        key: K,
        value: V,
        account: &crate::accounting::Account,
    ) -> Result<Option<V>> {
        Ok(match self.position(&key, account)? {
            Ok(index) => Some(std::mem::replace(&mut self.entries[index].1, value)),
            Err(index) => {
                self.entries.insert_slot(index)?.write((key, value));
                None
            }
        })
    }
    pub(super) fn values(&self) -> impl Iterator<Item = &V> {
        self.entries.iter().map(|(_, value)| value)
    }
    fn into_values(self) -> impl Iterator<Item = V> {
        self.entries.into_iter().map(|(_, value)| value)
    }
    pub(super) fn len(&self) -> usize {
        self.entries.len()
    }
    fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }
}
struct PassSet<K>(PassMap<K, ()>);
impl<K: Ord> PassSet<K> {
    fn new(account: &crate::accounting::Account, count: usize) -> Result<Self> {
        Ok(Self(PassMap::temporary(account, count)?))
    }
    fn insert_with(&mut self, key: K, account: &crate::accounting::Account) -> Result<bool> {
        Ok(self.0.insert_with(key, (), account)?.is_none())
    }
    fn contains_with(&self, key: &K, account: &crate::accounting::Account) -> Result<bool> {
        Ok(self.0.get(key, account)?.is_some())
    }
    fn len(&self) -> usize {
        self.0.len()
    }
}

#[cfg(test)]
mod accounting_tests {
    use super::*;
    use crate::accounting::{
        RenderAccountingContext, RenderAccountingOptions, RenderAttemptKind, RenderLimits,
        RenderPhase, RenderResource, RenderWork,
    };

    #[test]
    fn sorted_pass_bindings_use_exact_visible_probe_prefixes_and_reserved_capacity() {
        let mut context = RenderAccountingContext::default();
        context
            .prepare(
                RenderAttemptKind::GlyphRun,
                crate::GlyphRunLimits::new(0, 0, 0, 24),
                #[cfg(feature = "render-text")]
                None,
                |account| {
                    let mut map = PassMap::new(account, 3)?;
                    map.insert(3u32, 30u32);
                    map.insert(1, 10);
                    map.insert(2, 20);
                    assert_eq!(map.get(&3, account)?, Some(&30));
                    assert_eq!(map.get(&4, account)?, None);
                    *map.get_mut(&1, account)?.unwrap() = 11;
                    assert_eq!(map.remove_with(&2, account)?, Some(20));
                    assert_eq!(map.entries.as_slice(), &[(1, 11), (3, 30)]);
                    assert_eq!(map.entries.capacity(), 3);
                    Ok(())
                },
            )
            .unwrap();
        let usage = context.last_attempt().unwrap().attempt().unwrap().usage();
        assert_eq!(usage.work(RenderWork::ResourceProbes), 7);
        assert_eq!(usage.peak(RenderResource::RetainedCpuBytes), 24);
        assert_eq!(usage.live(RenderResource::RetainedCpuBytes), 0);
    }

    #[test]
    fn refusal_before_second_candidate_preserves_the_binding_value() {
        let options = RenderAccountingOptions::default().with_limits(
            RenderLimits::new()
                .with_retained_bytes(24)
                .with_work_limit(RenderWork::ResourceProbes, 1)
                .unwrap(),
        );
        let mut context = RenderAccountingContext::new(options);
        let result: Result<()> =
            context.run(RenderAttemptKind::Frame, RenderPhase::Plan, |account| {
                let mut map = PassMap::new(account, 3)?;
                map.insert(1u32, 10u32);
                map.insert(2, 20);
                map.insert(3, 30);
                let error = map.get_mut(&3, account).unwrap_err();
                assert_eq!(
                    map.get(&3, &crate::accounting::Account::for_test())
                        .unwrap(),
                    Some(&30)
                );
                Err(error)
            });
        assert!(result.is_err());
        assert_eq!(
            context
                .last_attempt()
                .unwrap()
                .attempt()
                .unwrap()
                .usage()
                .work(RenderWork::ResourceProbes),
            1
        );
    }

    #[test]
    fn cleanup_mutation_lookups_charge_lifetime_candidates_without_forward_limits() {
        use crate::accounting::{AccountedVec, RenderCpuScope, RendererCpu, StorageAccount};
        let cpu = RendererCpu::new(RenderAccountingOptions::default()).unwrap();
        let storage = StorageAccount::Renderer(cpu.at(RenderCpuScope::FrameBacking));
        let mut map = PassMap {
            entries: AccountedVec::new_in(&storage, 3, RenderResource::RetainedCpuBytes).unwrap(),
        };
        map.insert(3u32, 30u32);
        map.insert(1, 10);
        map.insert(2, 20);
        assert_eq!(map.maintenance_get_mut(&3).copied(), Some(30));
        // Insertion visits 0+1+2 candidates; the final lookup visits 2.
        assert_eq!(
            cpu.snapshot()
                .unwrap()
                .usage()
                .work(RenderWork::ResourceProbes),
            5
        );
        assert_eq!(map.entries.as_slice(), &[(1, 10), (2, 20), (3, 30)]);
    }
}

#[cfg_attr(
    not(test),
    expect(
        unused_imports,
        reason = "the pass front door preserves its crate-visible closure contract"
    )
)]
pub(crate) use close::{
    BackdropPreparableGraph, BaseExecutionFacts, BasePreparableGraph, ColorFilterPreparableGraph,
    CompositionPreparableGraph, ExecutableVelloCaptureFacts, SpatialFilterPreparableGraph,
};
#[expect(
    unused_imports,
    reason = "the pass front door preserves its crate-visible encoding contract"
)]
pub(crate) use encode::{
    AccountingReadyPreparedFrameCommit, CustomSpineEncodingSummary, GraphExternalOutputView,
    PendingGraphEncoding, PendingPreparedFrameCommit, PreparedGraphSubmission,
};
#[expect(
    unused_imports,
    reason = "the pass front door preserves its crate-visible capture contract"
)]
pub(crate) use encode::{VelloCaptureCompletionReceipt, VelloCaptureEncodingHandoff};
#[expect(
    unused_imports,
    reason = "the pass front door preserves its crate-visible color-filter contract"
)]
pub(crate) use model::RuntimeColorFilter;
#[expect(
    unused_imports,
    reason = "the pass front door preserves its crate-visible runtime model contract"
)]
pub(crate) use model::{
    LoweredGraphPlan, RuntimeBlur, RuntimeBlurAxis, RuntimeBlurInput, RuntimeClipCoverage,
    RuntimeClipCoverageElement, RuntimeColorClampBoundary, RuntimeColorOperation,
    RuntimeColorOperationKind, RuntimeComposite, RuntimeCompositeKind, RuntimeDropShadow,
    RuntimeFilterSpatialMapping, RuntimeGraphGeneration, RuntimeInitialization,
    RuntimeLayerCompositeParameters, RuntimeOuterClip, RuntimePass, RuntimePassCacheKeys,
    RuntimePassId, RuntimePassKind, RuntimeReadBinding, RuntimeReadRole,
    RuntimeResolvedAlphaMaskComposition, RuntimeResourceFormat, RuntimeResourceId,
    RuntimeResourceImport, RuntimeResourceProducer, RuntimeResourceRequest, RuntimeResourceRole,
    RuntimeResultBinding, RuntimeSamplingEdge, RuntimeSamplingFilter, RuntimeSpatialDescriptor,
    RuntimeVelloCapture, RuntimeVelloSpan, RuntimeVelloSpanScope,
};
#[expect(
    unused_imports,
    reason = "the pass front door preserves its crate-visible texture-binding contract"
)]
pub(crate) use prepare::PreparedTextureBinding;
#[cfg_attr(
    not(test),
    expect(
        unused_imports,
        reason = "the pass front door preserves its crate-visible prepared-binding contract"
    )
)]
pub(crate) use prepare::{
    ExecutableGraphDispatchEligibility, PreparedGaussianKernelBinding, PreparedGraph,
    PreparedPassView, VELLO_CAPTURE_TEXTURE_USAGES,
};

#[cfg(test)]
#[expect(
    unused_imports,
    reason = "the test-only pass front door preserves every existing crate-visible test contract"
)]
pub(crate) use test_support::{
    BackdropBlurCacheRealizationObservationForTest, BackdropBlurLayoutObservationForTest,
    BackdropExecutableGraphObservationForTest, BackdropFilterChainObservationForTest,
    BackdropGraphObservationForTest, BaseGraphExecutableSubsetObservationForTest,
    BlurCacheRealizationObservationForTest, BlurLayoutObservationForTest,
    BoundedCaptureTransformObservationForTest, ClipCoverageCaptureObservationForTest,
    ClipCoverageElementObservationForTest, ColorFilterCacheRealizationObservationForTest,
    ColorFilterCacheRequestsForTest, ColorFilterExecutableGraphObservationForTest,
    ColorFilterGraphObservationForTest, ColorFilterLayoutObservationForTest,
    ColorFilterOperationBufferLimitObservationForTest, ColorFilterOperationBytesObservationForTest,
    ColorFilterSpatialObservationForTest, CompositionExecutableGraphObservationForTest,
    CompositionGraphObservationForTest, CompositionOuterOperationObservationForTest,
    CompositionReadObservationForTest, CopyBackdropCacheRealizationObservationForTest,
    CopyBackdropLayoutObservationForTest, CorePassCacheRequestsForTest,
    CorePassLayoutObservationForTest, DropShadowCacheRealizationObservationForTest,
    DropShadowLayoutObservationForTest, EncodedVelloCaptureObservationForTest,
    GraphClipCoverageObservationForTest, LayerCompositeCacheRequestsForTest,
    LayerCompositeLayoutObservationForTest, LayerCompositionObservationForTest,
    MaskUploadAllocationObservationForTest, MixedColorUnsupportedDiagnosticObservationForTest,
    PreparedAllocationIdentitiesForTest, PreparedGraphExerciseObservationForTest,
    RuntimeColorFilterObservationForTest, RuntimeColorOperationObservationForTest,
    RuntimeColorOperationTagForTest, RuntimeColorScalarObservationForTest,
    RuntimeFilterAmountObservationForTest, RuntimeLoweringObservationForTest,
    ScopedColorFilterShaderFailureForTest, SpatialFilterExecutableGraphObservationForTest,
    SpatialFilterGraphObservationForTest, SpatialFilterPassTagForTest, VelloCaptureGridForTest,
    backdrop_blur_cache_realization_observation_for_test,
    backdrop_blur_layout_observation_for_test, backdrop_executable_graph_observation_for_test,
    backdrop_filter_chain_observation_for_test, backdrop_graph_observation_for_test,
    backdrop_preparable_graph_from_graph_for_test,
    base_graph_executable_subset_observation_for_test, blur_cache_realization_observation_for_test,
    blur_layout_observation_for_test, bounded_capture_transform_observation_for_test,
    color_filter_cache_realization_observation_for_test,
    color_filter_executable_graph_observation_for_test, color_filter_graph_observation_for_test,
    color_filter_layout_observation_for_test,
    color_filter_operation_buffer_limit_observation_for_test,
    color_filter_operation_bytes_observation_for_test, color_filter_preparable_graph_for_test,
    composite_parameter_bytes_for_test, composition_executable_graph_observation_for_test,
    composition_graph_observation_for_test, copy_backdrop_cache_realization_observation_for_test,
    copy_backdrop_layout_observation_for_test, core_pass_cache_requests_for_test,
    core_pass_layout_observation_for_test, drop_shadow_cache_realization_observation_for_test,
    drop_shadow_layout_observation_for_test, graph_clip_coverage_observation_for_test,
    layer_composite_cache_requests_for_test, layer_composite_layout_observation_for_test,
    mask_pipeline_keys_exclude_image_identity_for_test,
    mask_upload_allocation_observation_for_test,
    mixed_color_unsupported_diagnostic_observation_for_test,
    normalize_color_filter_shader_failure_for_test, normalize_scope_resolution_failure_for_test,
    pass_spatial_uniform_bytes_for_test, runtime_color_filter_observation_for_test,
    runtime_lowering_observation_for_test, spatial_filter_executable_graph_observation_for_test,
    spatial_filter_graph_observation_for_test, spatial_filter_preparable_graph_from_graph_for_test,
    two_capture_spine_lowered_for_test, zero_capture_spine_lowered_for_test,
};

use super::{
    Result, backend::DeviceCapabilities, renderer::EffectQualityPolicy, resource::WorkingFormat,
};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum ExecutableGraphWorkingFormatRequest {
    ConfiguredPolicy(EffectQualityPolicy),
    #[cfg(test)]
    Exact(WorkingFormat),
}

impl ExecutableGraphWorkingFormatRequest {
    fn resolve(self, capabilities: &DeviceCapabilities) -> Result<WorkingFormat> {
        match self {
            Self::ConfiguredPolicy(policy) => capabilities.resolve_effect_working_format(policy),
            #[cfg(test)]
            Self::Exact(working_format) => {
                capabilities.validate_supported_working_format(working_format)?;
                Ok(working_format)
            }
        }
    }
}
