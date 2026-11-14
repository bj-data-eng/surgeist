use super::PassMap;
use crate::accounting::{
    Account, AccountedVec, RenderPhase, RenderWork, SharedValue, StorageAccount,
};

use super::model::{
    LoweredGraphPlan, RuntimeBlur, RuntimeBlurInput, RuntimeColorClampBoundary, RuntimeColorFilter,
    RuntimeColorOperation, RuntimeColorOperationKind, RuntimeComposite, RuntimeCompositeKind,
    RuntimeGraphGeneration, RuntimeInitialization, RuntimeLayerCompositeParameters, RuntimePass,
    RuntimePassCacheKeys, RuntimePassId, RuntimePassKind, RuntimeReadBinding, RuntimeReadRole,
    RuntimeResourceFormat, RuntimeResourceId, RuntimeResourceImport, RuntimeResourceProducer,
    RuntimeResourceRequest, RuntimeResourceRole, RuntimeResultBinding, RuntimeSamplingEdge,
    RuntimeSamplingFilter, RuntimeSpatialDescriptor, RuntimeVelloCapture,
};
use super::{
    RuntimeBlurAxis, RuntimeClipCoverage, RuntimeClipCoverageElement, RuntimeDropShadow,
    RuntimeFilterSpatialMapping, RuntimeOuterClip, RuntimeResolvedAlphaMaskComposition,
    RuntimeVelloSpan, RuntimeVelloSpanScope,
};
use crate::{
    BackendErrorCode, Error, Format, Result,
    backend::DeviceCapabilities,
    filter::{
        CSS_FILTER_KERNEL_SUPPORT_STANDARD_DEVIATIONS, ColorClampBoundary, RuntimeFilterAmount,
        RuntimeFilterAngle, RuntimeUnitFilterAmount,
    },
    frame::{
        GpuRenderGraph, GraphLoweringBlur, GraphLoweringBlurInput, GraphLoweringClipCoverage,
        GraphLoweringColorFilter, GraphLoweringColorOperation, GraphLoweringComposite,
        GraphLoweringCompositeKind, GraphLoweringDropShadow, GraphLoweringEdgePolicy,
        GraphLoweringFilterSpatialMapping, GraphLoweringImportView, GraphLoweringInitialization,
        GraphLoweringPassKind, GraphLoweringPassResult, GraphLoweringPassView,
        GraphLoweringReadBinding, GraphLoweringReadRole, GraphLoweringResourceProducer,
        GraphLoweringResourceRole, GraphLoweringResourceView, GraphLoweringSamplingEdge,
        GraphLoweringSamplingFilter, GraphLoweringSpatialDescriptor, GraphLoweringVelloCapture,
        GraphLoweringVelloSpan, GraphLoweringVelloSpanScope,
    },
    resource::{GaussianKernelKey, GaussianKernelSamplingForm, WorkingFormat},
    shader::{
        BindGroupLayoutKey, RenderPipelineKey, SamplerKey, ShaderBindingRoleKey,
        ShaderCompositeKey, ShaderCompositePathKey, ShaderDataBindingKey, ShaderMaskQualityKey,
        ShaderMaskSamplingKey, ShaderModuleKey, ShaderProgramKey, ShaderSamplingEdgeKey,
        ShaderSamplingFilterKey, ShaderTextureFormatKey,
    },
    style::ColorFilterOp,
};

impl RuntimeSpatialDescriptor {
    fn from_graph(spatial: GraphLoweringSpatialDescriptor) -> Self {
        Self {
            logical_bounds: spatial.logical_bounds(),
            device_origin: spatial.device_origin(),
            device_extent: spatial.device_extent(),
            texel_origin: spatial.texel_origin(),
            raster_scale: spatial.raster_scale(),
        }
    }
}

#[derive(Clone, Copy)]
enum GraphLoweringCapabilityValidation<'capabilities> {
    Required(&'capabilities DeviceCapabilities),
    ClassificationOnly,
}

struct LoweredResourceSet {
    values: SharedValue<Vec<RuntimeResourceRequest>>,
    index: PassMap<RuntimeResourceId, usize>,
}
fn resource_by_id<'a>(
    resources: &'a [RuntimeResourceRequest],
    index: &PassMap<RuntimeResourceId, usize>,
    id: RuntimeResourceId,
    account: &Account,
) -> Result<Option<&'a RuntimeResourceRequest>> {
    Ok(index.get(&id, account)?.map(|index| &resources[*index]))
}

fn lower_runtime_resources(
    resource_views: &[GraphLoweringResourceView<'_>],
    pass_ids: &PassMap<RuntimePassId, ()>,
    working_format: WorkingFormat,
    capability_validation: GraphLoweringCapabilityValidation<'_>,
    account: &Account,
) -> Result<LoweredResourceSet> {
    let mut resources = account.retained(resource_views.len())?;
    let mut index = PassMap::temporary(account, resource_views.len())?;
    for resource in resource_views {
        account.work(RenderWork::GraphNodeVisits, 1)?;
        let resource = *resource;
        let id = RuntimeResourceId(resource.id());
        let Err(slot) = index.position(&id, account)? else {
            return Err(lowering_error("duplicate runtime resource binding"));
        };
        let role = runtime_resource_role(resource.role());
        let spatial = RuntimeSpatialDescriptor::from_graph(resource.spatial());
        let format = runtime_resource_format(role, working_format);
        if let GraphLoweringCapabilityValidation::Required(capabilities) = capability_validation {
            capabilities.validate_effect_texture_extent(spatial.device_extent)?;
            if let RuntimeResourceFormat::Working(format) = format {
                capabilities.validate_effect_texture_allocation(
                    spatial.device_extent,
                    Some(format),
                    format.texture_format(),
                    format.required_usages(),
                )?;
            }
        }
        let producer = match resource.producer() {
            GraphLoweringResourceProducer::Imported => RuntimeResourceProducer::Imported,
            GraphLoweringResourceProducer::Pass(pass) => {
                let pass = RuntimePassId(pass);
                if pass_ids.get(&pass, account)?.is_none() {
                    return Err(lowering_error("resource producer binding is missing"));
                }
                RuntimeResourceProducer::Pass(pass)
            }
        };
        let import = resource.import().map(|import| match import {
            GraphLoweringImportView::ResolvedAlphaMask(upload) => {
                RuntimeResourceImport::ResolvedAlphaMask(upload.clone())
            }
        });
        if matches!(producer, RuntimeResourceProducer::Imported) != import.is_some() {
            return Err(lowering_error(
                "imported runtime resource binding is inconsistent",
            ));
        }
        let last_use = RuntimePassId(resource.last_use());
        if pass_ids.get(&last_use, account)?.is_none() {
            return Err(lowering_error("resource last-use binding is missing"));
        }
        let position = resources.len();
        resources.append_slot()?.write(RuntimeResourceRequest {
            id,
            role,
            format,
            spatial,
            producer,
            expected_reads: resource.expected_reads(),
            last_use,
            import,
        });
        index.insert_at_reserved(slot, id, position)?;
    }
    Ok(LoweredResourceSet {
        values: SharedValue::from_accounted(resources, &account.storage())?,
        index,
    })
}

fn lower_runtime_passes(
    pass_views: &[GraphLoweringPassView<'_>],
    resources: &[RuntimeResourceRequest],
    resource_index: &PassMap<RuntimeResourceId, usize>,
    releases: &mut PassMap<RuntimePassId, AccountedVec<RuntimeResourceId>>,
    working_format: WorkingFormat,
    output_format: Format,
    account: &Account,
) -> Result<SharedValue<Vec<RuntimePass>>> {
    let mut passes = account.retained::<RuntimePass>(pass_views.len())?;
    let mut seen = PassMap::temporary(account, pass_views.len())?;
    for pass in pass_views {
        account.work(RenderWork::GraphNodeVisits, 1)?;
        let pass = *pass;
        let id = RuntimePassId(pass.id());
        let Err(slot) = seen.position(&id, account)? else {
            return Err(lowering_error("duplicate runtime pass binding"));
        };
        let input_dependencies = pass.dependencies(account)?;
        let mut dependencies = account.retained(input_dependencies.len())?;
        for dependency in input_dependencies.as_slice() {
            account.work(RenderWork::GraphEdgeVisits, 1)?;
            let dependency = RuntimePassId(*dependency);
            if seen.get(&dependency, account)?.is_none() {
                return Err(lowering_error(
                    "runtime pass has a missing or forward dependency",
                ));
            }
            dependencies.append_slot()?.write(dependency);
        }
        let (kind, input_reads) = pass.project(account, |kind| {
            runtime_pass_kind(kind, working_format, account)
        })?;
        let reads = lower_read_bindings(&input_reads, resources, resource_index, account)?;
        drop(input_reads);
        let result = lower_runtime_result(
            pass.result(),
            &kind,
            &reads,
            resources,
            resource_index,
            output_format,
            account,
        )?;
        let pass_releases = match releases.remove_with(&id, account)? {
            Some(values) => values,
            None => account.retained(0)?,
        };
        for resource in pass_releases.iter() {
            account.work(RenderWork::GraphEdgeVisits, 1)?;
            let mut found = false;
            for read in reads.iter() {
                account.work(RenderWork::ResourceProbes, 1)?;
                if read.resource == *resource {
                    found = true;
                    break;
                }
            }
            if !found {
                return Err(lowering_error(
                    "runtime release is not the resource's last read",
                ));
            }
        }
        let cache_keys = runtime_pass_cache_keys(
            &kind,
            &reads,
            result,
            working_format,
            output_format,
            |id| Ok(resource_by_id(resources, resource_index, id, account)?.map(|r| r.format)),
            account,
        )?;
        passes.append_slot()?.write(RuntimePass {
            id,
            kind,
            dependencies: SharedValue::from_accounted(dependencies, &account.storage())?,
            reads,
            result,
            releases: SharedValue::from_accounted(pass_releases, &account.storage())?,
            cache_keys,
        });
        seen.insert_at_reserved(slot, id, ())?;
    }
    SharedValue::from_accounted(passes, &account.storage())
}

fn lower_runtime_result(
    result: GraphLoweringPassResult,
    kind: &RuntimePassKind,
    reads: &[RuntimeReadBinding],
    resources: &[RuntimeResourceRequest],
    resource_index: &PassMap<RuntimeResourceId, usize>,
    output_format: Format,
    account: &Account,
) -> Result<RuntimeResultBinding> {
    match result {
        GraphLoweringPassResult::Empty if matches!(kind, RuntimePassKind::Present) => {
            Ok(RuntimeResultBinding::Output(output_format))
        }
        GraphLoweringPassResult::Empty => Ok(RuntimeResultBinding::Empty),
        GraphLoweringPassResult::Resource(resource) => {
            let resource = RuntimeResourceId(resource);
            if resource_by_id(resources, resource_index, resource, account)?.is_none() {
                return Err(lowering_error(
                    "runtime pass result binding is inconsistent",
                ));
            }
            for read in reads {
                account.work(RenderWork::GraphEdgeVisits, 1)?;
                if read.resource == resource {
                    return Err(lowering_error(
                        "runtime pass result binding is inconsistent",
                    ));
                }
            }
            Ok(RuntimeResultBinding::Resource(resource))
        }
    }
}

impl LoweredGraphPlan {
    pub(crate) fn try_lower_validated_graph(
        graph: &GpuRenderGraph,
        working_format: WorkingFormat,
        output_format: Format,
        capabilities: &DeviceCapabilities,
        account: &Account,
    ) -> Result<Self> {
        Self::try_lower_validated_graph_inner(
            graph,
            working_format,
            output_format,
            GraphLoweringCapabilityValidation::Required(capabilities),
            account,
        )
    }
    pub(super) fn try_lower_for_dispatch_classification(
        graph: &GpuRenderGraph,
        working_format: WorkingFormat,
        output_format: Format,
        account: &Account,
    ) -> Result<Self> {
        Self::try_lower_validated_graph_inner(
            graph,
            working_format,
            output_format,
            GraphLoweringCapabilityValidation::ClassificationOnly,
            account,
        )
    }
    fn try_lower_validated_graph_inner(
        graph: &GpuRenderGraph,
        working_format: WorkingFormat,
        output_format: Format,
        capability_validation: GraphLoweringCapabilityValidation<'_>,
        account: &Account,
    ) -> Result<Self> {
        let account = &account.at(RenderPhase::Lower);
        account.phase(RenderPhase::Lower, || {
            let view = graph.lowering_view(account)?;
            let resource_views = view.resources(account)?;
            let pass_views = view.passes(account)?;
            let mut pass_ids = PassMap::temporary(account, pass_views.len())?;
            for pass in pass_views.iter() {
                account.work(RenderWork::GraphNodeVisits, 1)?;
                pass_ids.insert_with(RuntimePassId(pass.id()), (), account)?;
            }
            let LoweredResourceSet {
                values: resources,
                index: resource_index,
            } = lower_runtime_resources(
                &resource_views,
                &pass_ids,
                working_format,
                capability_validation,
                account,
            )?;
            let mut releases =
                PassMap::<RuntimePassId, AccountedVec<RuntimeResourceId>>::temporary(
                    account,
                    resources.len(),
                )?;
            for resource in resources.iter() {
                account.work(RenderWork::GraphEdgeVisits, 1)?;
                match releases.position(&resource.last_use, account)? {
                    Ok(index) => releases.entries[index].1.push(resource.id)?,
                    Err(slot) => {
                        let mut values = account.retained(1)?;
                        values.append_slot()?.write(resource.id);
                        releases.insert_at_reserved(slot, resource.last_use, values)?;
                    }
                }
            }
            let passes = lower_runtime_passes(
                &pass_views,
                &resources,
                &resource_index,
                &mut releases,
                working_format,
                output_format,
                account,
            )?;
            if !releases.is_empty() {
                return Err(lowering_error(
                    "one or more release bindings have no runtime pass",
                ));
            }
            let root_working_image = RuntimeResourceId(view.root_working_image());
            if resource_by_id(&resources, &resource_index, root_working_image, account)?
                .map(|r| r.format)
                != Some(RuntimeResourceFormat::Working(working_format))
            {
                return Err(lowering_error(
                    "root working image does not use the graph format",
                ));
            }
            let final_present = RuntimePassId(view.final_present());
            let terminal_matches = match passes.last() {
                Some(pass) => {
                    account.work(RenderWork::GraphNodeVisits, 1)?;
                    pass.id == final_present && matches!(pass.kind, RuntimePassKind::Present)
                }
                None => false,
            };
            if !terminal_matches {
                return Err(lowering_error(
                    "the runtime plan has no terminal present pass",
                ));
            }
            Ok(Self {
                generation: RuntimeGraphGeneration(view.generation()),
                working_format,
                output_format,
                resources,
                passes,
                root_working_image,
                final_present,
            })
        })
    }
}

pub(super) fn lowering_error(message: &'static str) -> Error {
    Error::new(BackendErrorCode::RenderFailed, message)
}

const fn runtime_resource_role(role: GraphLoweringResourceRole) -> RuntimeResourceRole {
    match role {
        GraphLoweringResourceRole::RootWorkingImage => RuntimeResourceRole::RootWorkingImage,
        GraphLoweringResourceRole::CaptureWorkingImage => RuntimeResourceRole::CaptureWorkingImage,
        GraphLoweringResourceRole::ClipCoverage => RuntimeResourceRole::ClipCoverage,
        GraphLoweringResourceRole::IsolationWorkingImage => {
            RuntimeResourceRole::IsolationWorkingImage
        }
        GraphLoweringResourceRole::ImportedImage => RuntimeResourceRole::ImportedImage,
        GraphLoweringResourceRole::BackdropCopy => RuntimeResourceRole::BackdropCopy,
        GraphLoweringResourceRole::FilterIntermediate => RuntimeResourceRole::FilterIntermediate,
        GraphLoweringResourceRole::ConvolutionFloat => RuntimeResourceRole::ConvolutionFloat,
        GraphLoweringResourceRole::ShadowImage => RuntimeResourceRole::ShadowImage,
        GraphLoweringResourceRole::CompositeResult => RuntimeResourceRole::CompositeResult,
    }
}

pub(super) const fn runtime_resource_format(
    role: RuntimeResourceRole,
    working_format: WorkingFormat,
) -> RuntimeResourceFormat {
    match role {
        RuntimeResourceRole::CaptureWorkingImage => RuntimeResourceFormat::VelloCaptureRgba8Unorm,
        RuntimeResourceRole::ClipCoverage => RuntimeResourceFormat::ClipCoverageRgba8Unorm,
        RuntimeResourceRole::ImportedImage => RuntimeResourceFormat::ResolvedMaskRgba8Unorm,
        RuntimeResourceRole::ConvolutionFloat => RuntimeResourceFormat::ConvolutionRgba32Float,
        RuntimeResourceRole::RootWorkingImage
        | RuntimeResourceRole::IsolationWorkingImage
        | RuntimeResourceRole::BackdropCopy
        | RuntimeResourceRole::FilterIntermediate
        | RuntimeResourceRole::ShadowImage
        | RuntimeResourceRole::CompositeResult => RuntimeResourceFormat::Working(working_format),
    }
}

fn runtime_pass_kind(
    kind: &GraphLoweringPassKind,
    working_format: WorkingFormat,
    account: &Account,
) -> Result<RuntimePassKind> {
    Ok(match kind {
        GraphLoweringPassKind::ClearRoot {
            initialization,
            color,
        } => RuntimePassKind::ClearRoot {
            initialization: match initialization {
                GraphLoweringInitialization::SurfaceBaseColor => {
                    RuntimeInitialization::SurfaceBaseColor
                }
                GraphLoweringInitialization::Transparent => RuntimeInitialization::Transparent,
            },
            color: *color,
        },
        GraphLoweringPassKind::VelloCapture(work) => RuntimePassKind::VelloCapture(
            work.as_ref()
                .map(|work| runtime_vello_capture(work, account))
                .transpose()?,
        ),
        GraphLoweringPassKind::CanonicalizeCapture => RuntimePassKind::CanonicalizeCapture,
        GraphLoweringPassKind::CopyBackdrop => RuntimePassKind::CopyBackdrop,
        GraphLoweringPassKind::ColorFilter(filter) => RuntimePassKind::ColorFilter(
            filter
                .as_ref()
                .map(|filter| runtime_color_filter(filter, account))
                .transpose()?,
        ),
        GraphLoweringPassKind::Convolution(convolution) => {
            RuntimePassKind::Convolution(convolution.clone())
        }
        GraphLoweringPassKind::BlurHorizontal(blur) => RuntimePassKind::BlurHorizontal(
            blur.map(|blur| runtime_blur(blur, RuntimeBlurAxis::Horizontal, working_format)),
        ),
        GraphLoweringPassKind::BlurVertical(blur) => RuntimePassKind::BlurVertical(
            blur.map(|blur| runtime_blur(blur, RuntimeBlurAxis::Vertical, working_format)),
        ),
        GraphLoweringPassKind::DropShadowColorize(shadow) => {
            RuntimePassKind::DropShadowColorize(shadow.map(runtime_drop_shadow))
        }
        GraphLoweringPassKind::Composite(composite) => RuntimePassKind::Composite(
            composite
                .as_ref()
                .map(|composite| runtime_composite(composite, account))
                .transpose()?,
        ),
        GraphLoweringPassKind::Present => RuntimePassKind::Present,
    })
}

fn runtime_vello_capture(
    capture: &GraphLoweringVelloCapture,
    account: &Account,
) -> Result<RuntimeVelloCapture> {
    Ok(match capture {
        GraphLoweringVelloCapture::Span(span) => {
            RuntimeVelloCapture::Span(runtime_vello_span(span))
        }
        GraphLoweringVelloCapture::ClipCoverage(coverage) => {
            RuntimeVelloCapture::ClipCoverage(runtime_clip_coverage(coverage, account)?)
        }
    })
}

fn runtime_vello_span(span: &GraphLoweringVelloSpan) -> RuntimeVelloSpan {
    RuntimeVelloSpan {
        scope: match span.scope() {
            GraphLoweringVelloSpanScope::CurrentParent => RuntimeVelloSpanScope::CurrentParent,
            GraphLoweringVelloSpanScope::LayerSource => RuntimeVelloSpanScope::LayerSource,
        },
        source_space: span.source_space(),
        commands: span.commands().clone(),
        capture_transform: span.capture_transform(),
        parent_to_surface: span.parent_to_surface(),
        antialiasing: span.antialiasing(),
        captured_before_outer_semantics: span.captured_before_outer_semantics(),
    }
}

fn runtime_clip_coverage(
    coverage: &GraphLoweringClipCoverage,
    account: &Account,
) -> Result<RuntimeClipCoverage> {
    let mut elements = account.retained(coverage.elements().len())?;
    for element in coverage.elements() {
        account.work(RenderWork::GraphNodeVisits, 1)?;
        elements.append_slot()?.write(RuntimeClipCoverageElement {
            clip: element.clip().clone(),
            transform: element.transform(),
        });
    }
    Ok(RuntimeClipCoverage {
        elements: SharedValue::from_accounted(elements, &account.storage())?,
        antialiasing: coverage.antialiasing(),
    })
}

fn runtime_filter_spatial(
    spatial: GraphLoweringFilterSpatialMapping,
) -> RuntimeFilterSpatialMapping {
    RuntimeFilterSpatialMapping {
        source: RuntimeSpatialDescriptor::from_graph(spatial.source()),
        result: RuntimeSpatialDescriptor::from_graph(spatial.result()),
    }
}

fn runtime_edge(edge: GraphLoweringEdgePolicy) -> RuntimeSamplingEdge {
    match edge {
        GraphLoweringEdgePolicy::NoSampling => RuntimeSamplingEdge::ClampToExtent,
        GraphLoweringEdgePolicy::TransparentBlack => RuntimeSamplingEdge::TransparentBlack,
        GraphLoweringEdgePolicy::SemanticBorderMirror(bounds) => {
            RuntimeSamplingEdge::SemanticBorderMirror(bounds)
        }
    }
}

fn runtime_color_filter(
    filter: &GraphLoweringColorFilter,
    account: &Account,
) -> Result<RuntimeColorFilter> {
    let mut operations = account.retained(filter.operations().len())?;
    for operation in filter.operations() {
        account.work(RenderWork::FilterOperationVisits, 1)?;
        operations
            .append_slot()?
            .write(runtime_color_operation(*operation)?);
    }
    Ok(RuntimeColorFilter {
        operations: SharedValue::from_accounted(operations, &account.storage())?,
        spatial: runtime_filter_spatial(filter.spatial()),
        edge: runtime_edge(filter.edge()),
    })
}

fn runtime_color_operation(
    operation: GraphLoweringColorOperation,
) -> Result<RuntimeColorOperation> {
    let runtime_operation = match operation.operation() {
        ColorFilterOp::Brightness(amount) => {
            RuntimeColorOperationKind::Brightness(RuntimeFilterAmount::try_from_algorithm(amount)?)
        }
        ColorFilterOp::Contrast(amount) => {
            RuntimeColorOperationKind::Contrast(RuntimeFilterAmount::try_from_algorithm(amount)?)
        }
        ColorFilterOp::Grayscale(amount) => RuntimeColorOperationKind::Grayscale(
            RuntimeUnitFilterAmount::try_from_algorithm(amount)?,
        ),
        ColorFilterOp::HueRotate(angle) => {
            RuntimeColorOperationKind::HueRotate(RuntimeFilterAngle::try_from_algorithm(angle)?)
        }
        ColorFilterOp::Invert(amount) => {
            RuntimeColorOperationKind::Invert(RuntimeUnitFilterAmount::try_from_algorithm(amount)?)
        }
        ColorFilterOp::Opacity(amount) => {
            RuntimeColorOperationKind::Opacity(RuntimeUnitFilterAmount::try_from_algorithm(amount)?)
        }
        ColorFilterOp::Saturate(amount) => {
            RuntimeColorOperationKind::Saturate(RuntimeFilterAmount::try_from_algorithm(amount)?)
        }
        ColorFilterOp::Sepia(amount) => {
            RuntimeColorOperationKind::Sepia(RuntimeUnitFilterAmount::try_from_algorithm(amount)?)
        }
    };
    let clamp_boundary = match operation.clamp_boundary() {
        ColorClampBoundary::ClampStraightRgbaToUnitThenPremultiply => {
            RuntimeColorClampBoundary::ClampStraightRgbaToUnitThenPremultiply
        }
    };
    Ok(RuntimeColorOperation {
        operation: runtime_operation,
        clamp_boundary,
    })
}

fn runtime_blur(
    blur: GraphLoweringBlur,
    axis: RuntimeBlurAxis,
    _working_format: WorkingFormat,
) -> RuntimeBlur {
    let spatial = runtime_filter_spatial(blur.spatial());
    let kernel = GaussianKernelKey::from_exact_plan(
        blur.standard_deviation().to_bits(),
        spatial.result.raster_scale.to_bits(),
        CSS_FILTER_KERNEL_SUPPORT_STANDARD_DEVIATIONS.to_bits(),
        blur.support_radius(),
        GaussianKernelSamplingForm::PairedLinear,
    );
    RuntimeBlur {
        axis,
        input: match blur.input() {
            GraphLoweringBlurInput::Rgba => RuntimeBlurInput::Rgba,
            GraphLoweringBlurInput::SourceAlpha => RuntimeBlurInput::SourceAlpha,
        },
        standard_deviation: blur.standard_deviation(),
        support_radius: blur.support_radius(),
        kernel,
        spatial,
        edge: runtime_edge(blur.edge()),
    }
}

fn runtime_drop_shadow(shadow: GraphLoweringDropShadow) -> RuntimeDropShadow {
    RuntimeDropShadow {
        offset: shadow.offset(),
        standard_deviation: shadow.standard_deviation(),
        color: shadow.color(),
        support_radius: shadow.support_radius(),
        spatial: runtime_filter_spatial(shadow.spatial()),
        edge: runtime_edge(shadow.edge()),
        uses_source_alpha: shadow.uses_source_alpha(),
        uses_continuous_offset: shadow.uses_continuous_offset(),
        retains_unchanged_source: shadow.retains_unchanged_source(),
    }
}

fn runtime_composite(
    composite: &GraphLoweringComposite,
    account: &Account,
) -> Result<RuntimeComposite> {
    let kind = match composite.kind() {
        GraphLoweringCompositeKind::SpanSourceOver => RuntimeCompositeKind::SpanSourceOver,
        GraphLoweringCompositeKind::Layer {
            transform,
            destination_to_layer_local,
            source_space,
            opacity,
            composition,
            clip,
            outer_clips,
            clip_coverage,
            alpha_mask,
        } => {
            let alpha_mask = alpha_mask
                .as_ref()
                .map(|mask| {
                    RuntimeResolvedAlphaMaskComposition::try_new(
                        RuntimeResourceId(mask.resource()),
                        mask.bounds(),
                        mask.image_dimensions(),
                        ShaderMaskSamplingKey::with_rendering(
                            mask.quality(),
                            mask.rendering_hint(),
                            mask.extend(),
                        ),
                        mask.value(),
                        mask.application(),
                    )
                })
                .transpose()?;
            RuntimeCompositeKind::Layer {
                transform: *transform,
                parameters: SharedValue::new(
                    RuntimeLayerCompositeParameters::try_new(
                        destination_to_layer_local.affine(),
                        *source_space,
                        *opacity,
                        *composition,
                        clip.is_some() || !outer_clips.is_empty(),
                        alpha_mask,
                    )?,
                    &account.storage(),
                )?,
                clip: clip.clone(),
                outer_clips: {
                    let mut values = account.retained(outer_clips.len())?;
                    for clip in outer_clips.iter() {
                        account.work(RenderWork::GraphNodeVisits, 1)?;
                        values.append_slot()?.write(RuntimeOuterClip {
                            clip: clip.clip().clone(),
                            transform: clip.transform(),
                        });
                    }
                    SharedValue::from_accounted(values, &account.storage())?
                },
                clip_coverage: clip_coverage.map(RuntimeResourceId),
            }
        }
        GraphLoweringCompositeKind::DropShadow => RuntimeCompositeKind::DropShadow,
    };
    Ok(RuntimeComposite {
        kind,
        source_captured_before_outer_semantics: composite.source_captured_before_outer_semantics(),
    })
}

fn lower_read_bindings(
    reads: &[GraphLoweringReadBinding],
    resources: &[RuntimeResourceRequest],
    resource_index: &PassMap<RuntimeResourceId, usize>,
    account: &Account,
) -> Result<SharedValue<Vec<RuntimeReadBinding>>> {
    let mut bindings = account.retained::<RuntimeReadBinding>(reads.len())?;
    for read in reads.iter().copied() {
        account.work(RenderWork::GraphEdgeVisits, 1)?;
        let resource = RuntimeResourceId(read.resource());
        for binding in bindings.iter() {
            account.work(RenderWork::ResourceProbes, 1)?;
            if binding.resource == resource {
                return Err(lowering_error("duplicate runtime read binding"));
            }
        }
        let resource_request = resource_by_id(resources, resource_index, resource, account)?
            .ok_or_else(|| lowering_error("runtime read resource is missing"))?;
        let source_format = resource_request.format;
        let role = runtime_read_role(read.role());
        let sampling_edge = runtime_sampling_edge(read.sampling_edge());
        let resolved_mask_sampling = match read.sampling_filter() {
            GraphLoweringSamplingFilter::ImportedMask => match &resource_request.import {
                Some(RuntimeResourceImport::ResolvedAlphaMask(upload)) => {
                    Some(ShaderMaskSamplingKey::with_rendering(
                        upload.quality(),
                        upload.rendering_hint(),
                        upload.extend(),
                    ))
                }
                None => {
                    return Err(lowering_error(
                        "mask sampler is not bound to an imported resolved mask",
                    ));
                }
            },
            GraphLoweringSamplingFilter::Nearest
            | GraphLoweringSamplingFilter::Linear
            | GraphLoweringSamplingFilter::GaussianKernel => None,
        };
        let sampling_filter = match (read.sampling_filter(), resolved_mask_sampling) {
            (GraphLoweringSamplingFilter::Nearest, _) => RuntimeSamplingFilter::Nearest,
            (GraphLoweringSamplingFilter::ImportedMask, Some(sampling))
                if sampling.quality() == ShaderMaskQualityKey::Low =>
            {
                RuntimeSamplingFilter::Nearest
            }
            (GraphLoweringSamplingFilter::Linear, _)
            | (GraphLoweringSamplingFilter::GaussianKernel, _)
            | (GraphLoweringSamplingFilter::ImportedMask, Some(_)) => RuntimeSamplingFilter::Linear,
            (GraphLoweringSamplingFilter::ImportedMask, None) => {
                return Err(lowering_error(
                    "mask sampling policy is missing from an imported mask read",
                ));
            }
        };
        let sampler_key = SamplerKey::new(
            shader_binding_role(role),
            source_format.shader_key(),
            match sampling_filter {
                RuntimeSamplingFilter::Nearest => ShaderSamplingFilterKey::Nearest,
                RuntimeSamplingFilter::Linear => ShaderSamplingFilterKey::Linear,
            },
            shader_sampling_edge(sampling_edge),
            resolved_mask_sampling,
        );
        bindings.append_slot()?.write(RuntimeReadBinding {
            role,
            resource,
            sampling_filter,
            sampling_edge,
            sampler_key,
        });
    }
    SharedValue::from_accounted(bindings, &account.storage())
}

const fn runtime_read_role(role: GraphLoweringReadRole) -> RuntimeReadRole {
    match role {
        GraphLoweringReadRole::CaptureSource => RuntimeReadRole::CaptureSource,
        GraphLoweringReadRole::CompletedParent => RuntimeReadRole::CompletedParent,
        GraphLoweringReadRole::FilterSource => RuntimeReadRole::FilterSource,
        GraphLoweringReadRole::ConvolutionSource => RuntimeReadRole::ConvolutionSource,
        GraphLoweringReadRole::ConvolutionAuxiliary => RuntimeReadRole::ConvolutionAuxiliary,
        GraphLoweringReadRole::BlurredSourceAlpha => RuntimeReadRole::BlurredSourceAlpha,
        GraphLoweringReadRole::CompositeParent => RuntimeReadRole::CompositeParent,
        GraphLoweringReadRole::CompositeSource => RuntimeReadRole::CompositeSource,
        GraphLoweringReadRole::ClipCoverage => RuntimeReadRole::ClipCoverage,
        GraphLoweringReadRole::AlphaMask => RuntimeReadRole::AlphaMask,
        GraphLoweringReadRole::Shadow => RuntimeReadRole::Shadow,
        GraphLoweringReadRole::FinalWorkingImage => RuntimeReadRole::FinalWorkingImage,
    }
}

const fn runtime_sampling_edge(edge: GraphLoweringSamplingEdge) -> RuntimeSamplingEdge {
    match edge {
        GraphLoweringSamplingEdge::ClampToExtent => RuntimeSamplingEdge::ClampToExtent,
        GraphLoweringSamplingEdge::TransparentBlack => RuntimeSamplingEdge::TransparentBlack,
        GraphLoweringSamplingEdge::SemanticBorderMirror(bounds) => {
            RuntimeSamplingEdge::SemanticBorderMirror(bounds)
        }
    }
}

pub(super) const fn shader_binding_role(role: RuntimeReadRole) -> ShaderBindingRoleKey {
    match role {
        RuntimeReadRole::CaptureSource => ShaderBindingRoleKey::CaptureSource,
        RuntimeReadRole::CompletedParent => ShaderBindingRoleKey::CompletedParent,
        RuntimeReadRole::FilterSource => ShaderBindingRoleKey::FilterSource,
        RuntimeReadRole::ConvolutionSource => ShaderBindingRoleKey::ConvolutionSource,
        RuntimeReadRole::ConvolutionAuxiliary => ShaderBindingRoleKey::ConvolutionAuxiliary,
        RuntimeReadRole::BlurredSourceAlpha => ShaderBindingRoleKey::BlurredSourceAlpha,
        RuntimeReadRole::CompositeParent => ShaderBindingRoleKey::CompositeParent,
        RuntimeReadRole::CompositeSource => ShaderBindingRoleKey::CompositeSource,
        RuntimeReadRole::ClipCoverage => ShaderBindingRoleKey::ClipCoverage,
        RuntimeReadRole::AlphaMask => ShaderBindingRoleKey::AlphaMask,
        RuntimeReadRole::Shadow => ShaderBindingRoleKey::Shadow,
        RuntimeReadRole::FinalWorkingImage => ShaderBindingRoleKey::FinalWorkingImage,
    }
}

pub(super) const fn shader_sampling_edge(edge: RuntimeSamplingEdge) -> ShaderSamplingEdgeKey {
    match edge {
        RuntimeSamplingEdge::ClampToExtent => ShaderSamplingEdgeKey::ClampToExtent,
        RuntimeSamplingEdge::TransparentBlack => ShaderSamplingEdgeKey::TransparentBlack,
        RuntimeSamplingEdge::SemanticBorderMirror(_) => ShaderSamplingEdgeKey::SemanticBorderMirror,
    }
}

const fn runtime_read_uses_shader_sampler(kind: &RuntimePassKind, role: RuntimeReadRole) -> bool {
    match kind {
        RuntimePassKind::Convolution(_) => false,
        RuntimePassKind::Composite(Some(RuntimeComposite {
            kind: RuntimeCompositeKind::Layer { .. },
            ..
        })) => matches!(role, RuntimeReadRole::CompositeSource),
        _ => true,
    }
}

const fn runtime_read_uses_shader_texture(kind: &RuntimePassKind, role: RuntimeReadRole) -> bool {
    !matches!(
        (kind, role),
        (
            RuntimePassKind::Composite(Some(RuntimeComposite {
                kind: RuntimeCompositeKind::DropShadow,
                ..
            })),
            RuntimeReadRole::Shadow
        )
    )
}

pub(super) fn runtime_pass_cache_keys(
    kind: &RuntimePassKind,
    reads: &[RuntimeReadBinding],
    result: RuntimeResultBinding,
    working_format: WorkingFormat,
    output_format: Format,
    mut resource_format: impl FnMut(RuntimeResourceId) -> Result<Option<RuntimeResourceFormat>>,
    account: &Account,
) -> Result<Option<RuntimePassCacheKeys>> {
    if matches!(
        kind,
        RuntimePassKind::ClearRoot { .. } | RuntimePassKind::VelloCapture(_)
    ) || result == RuntimeResultBinding::Empty
    {
        return Ok(None);
    }
    let program = shader_program(kind)?;
    let composite_samples_parent = matches!(
        kind, RuntimePassKind::Composite(Some(RuntimeComposite {
            kind: RuntimeCompositeKind::Layer { parameters, .. }, ..
        })) if !parameters.composition().is_source_over_normal()
    );
    let mut sampled_textures = account.temporary(reads.len().max(2))?;
    let mut samplers = account.retained(reads.len())?;
    for read in reads {
        account.work(RenderWork::GraphEdgeVisits, 1)?;
        let sampled = runtime_read_uses_shader_texture(kind, read.role)
            && (read.role != RuntimeReadRole::CompositeParent
                || (!matches!(
                    kind,
                    RuntimePassKind::Composite(Some(RuntimeComposite {
                        kind: RuntimeCompositeKind::SpanSourceOver,
                        ..
                    }))
                ) && composite_samples_parent));
        if !sampled {
            continue;
        }
        let format = resource_format(read.resource)?
            .ok_or_else(|| lowering_error("cache key source format is missing"))?;
        sampled_textures
            .append_slot()?
            .write((shader_binding_role(read.role), format.shader_key()));
        if runtime_read_uses_shader_sampler(kind, read.role) {
            samplers.append_slot()?.write(read.sampler_key);
        }
    }
    if matches!(kind, RuntimePassKind::Convolution(_)) && reads.len() == 1 {
        let format = resource_format(reads[0].resource)?
            .ok_or_else(|| lowering_error("convolution source format is missing"))?;
        sampled_textures.append_slot()?.write((
            ShaderBindingRoleKey::ConvolutionAuxiliary,
            format.shader_key(),
        ));
    }
    let StorageAccount::Renderer(cpu) = account.storage() else {
        return Err(crate::accounting::RenderAccountingError::Invariant.into());
    };
    let cpu = cpu.at(crate::accounting::RenderCpuScope::DeviceBacking);
    let layout = BindGroupLayoutKey::new(
        &cpu,
        Some(account),
        program,
        &sampled_textures,
        shader_data_bindings(kind),
    )?;
    let output_key = matches!(kind, RuntimePassKind::Present)
        .then(|| ShaderTextureFormatKey::output(output_format));
    let shader = ShaderModuleKey::new(
        &cpu,
        Some(account),
        program,
        layout.clone(),
        &samplers,
        Some(if matches!(kind, RuntimePassKind::Convolution(_)) {
            ShaderTextureFormatKey::ConvolutionRgba32Float
        } else {
            ShaderTextureFormatKey::working(working_format)
        }),
        if matches!(kind, RuntimePassKind::Convolution(_)) {
            Some(match result {
                RuntimeResultBinding::Resource(resource) => resource_format(resource)?
                    .ok_or_else(|| lowering_error("convolution target format is missing"))?
                    .shader_key(),
                _ => {
                    return Err(lowering_error(
                        "convolution requires a concrete resource target",
                    ));
                }
            })
        } else {
            output_key
        },
    )?;
    let target_format = match result {
        RuntimeResultBinding::Resource(resource) => resource_format(resource)?
            .map(RuntimeResourceFormat::shader_key)
            .ok_or_else(|| lowering_error("cache key target format is missing"))?,
        RuntimeResultBinding::Output(format) => ShaderTextureFormatKey::output(format),
        RuntimeResultBinding::Empty => {
            return Err(lowering_error("executable pass has no target binding"));
        }
    };
    let pipeline = RenderPipelineKey::new(
        &cpu,
        Some(account),
        shader.clone(),
        layout.clone(),
        &samplers,
        target_format,
    )?;
    Ok(Some(RuntimePassCacheKeys {
        samplers: SharedValue::from_accounted(samplers, &account.storage())?,
        layout,
        shader,
        pipeline,
    }))
}

fn shader_program(kind: &RuntimePassKind) -> Result<ShaderProgramKey> {
    match kind {
        RuntimePassKind::CanonicalizeCapture => Ok(ShaderProgramKey::CanonicalizeCapture),
        RuntimePassKind::CopyBackdrop => Ok(ShaderProgramKey::CopyBackdrop),
        RuntimePassKind::ColorFilter(Some(_)) => Ok(ShaderProgramKey::ColorFilter),
        RuntimePassKind::Convolution(_) => Ok(ShaderProgramKey::Convolution),
        RuntimePassKind::BlurHorizontal(Some(blur)) => Ok(ShaderProgramKey::BlurHorizontal {
            source_alpha: blur.input == RuntimeBlurInput::SourceAlpha,
            edge: shader_sampling_edge(blur.edge),
        }),
        RuntimePassKind::BlurVertical(Some(blur)) => Ok(ShaderProgramKey::BlurVertical {
            source_alpha: blur.input == RuntimeBlurInput::SourceAlpha,
            edge: shader_sampling_edge(blur.edge),
        }),
        RuntimePassKind::DropShadowColorize(Some(_)) => Ok(ShaderProgramKey::DropShadowColorize),
        RuntimePassKind::Composite(Some(composite)) => Ok(ShaderProgramKey::Composite(
            shader_composite_key(&composite.kind),
        )),
        RuntimePassKind::Present => Ok(ShaderProgramKey::Present),
        RuntimePassKind::ClearRoot { .. }
        | RuntimePassKind::VelloCapture(_)
        | RuntimePassKind::ColorFilter(None)
        | RuntimePassKind::BlurHorizontal(None)
        | RuntimePassKind::BlurVertical(None)
        | RuntimePassKind::DropShadowColorize(None)
        | RuntimePassKind::Composite(None) => Err(lowering_error(
            "a specialized or empty pass requested a custom shader key",
        )),
    }
}

fn shader_composite_key(kind: &RuntimeCompositeKind) -> ShaderCompositeKey {
    match kind {
        RuntimeCompositeKind::SpanSourceOver => ShaderCompositeKey::SpanSourceOver,
        RuntimeCompositeKind::Layer {
            parameters,
            clip_coverage,
            ..
        } => ShaderCompositeKey::Layer {
            path: if parameters.composition().is_source_over_normal() {
                ShaderCompositePathKey::Normal
            } else {
                ShaderCompositePathKey::DestinationSampling
            },
            has_clip_coverage: clip_coverage.is_some(),
            has_alpha_mask: parameters.alpha_mask().is_some(),
        },
        RuntimeCompositeKind::DropShadow => ShaderCompositeKey::DropShadow,
    }
}

fn shader_data_bindings(kind: &RuntimePassKind) -> &'static [ShaderDataBindingKey] {
    use ShaderDataBindingKey::{
        BlurEdgeParameters, ColorFilterOperations, CompositeParameters, DropShadowParameters,
        GaussianKernel, SpatialUniform,
    };
    match kind {
        RuntimePassKind::CanonicalizeCapture
        | RuntimePassKind::CopyBackdrop
        | RuntimePassKind::Present => &[SpatialUniform],
        RuntimePassKind::ColorFilter(Some(_)) => &[SpatialUniform, ColorFilterOperations],
        RuntimePassKind::Convolution(_) => &[
            ShaderDataBindingKey::ConvolutionParameters,
            ShaderDataBindingKey::ConvolutionWeights,
        ],
        RuntimePassKind::BlurHorizontal(Some(blur)) | RuntimePassKind::BlurVertical(Some(blur)) => {
            if matches!(blur.edge, RuntimeSamplingEdge::SemanticBorderMirror(_)) {
                &[SpatialUniform, GaussianKernel, BlurEdgeParameters]
            } else {
                &[SpatialUniform, GaussianKernel]
            }
        }
        RuntimePassKind::DropShadowColorize(Some(_)) => &[SpatialUniform, DropShadowParameters],
        RuntimePassKind::Composite(Some(RuntimeComposite {
            kind: RuntimeCompositeKind::SpanSourceOver | RuntimeCompositeKind::DropShadow,
            ..
        })) => &[SpatialUniform],
        RuntimePassKind::Composite(Some(_)) => &[SpatialUniform, CompositeParameters],
        RuntimePassKind::ClearRoot { .. }
        | RuntimePassKind::VelloCapture(_)
        | RuntimePassKind::ColorFilter(None)
        | RuntimePassKind::BlurHorizontal(None)
        | RuntimePassKind::BlurVertical(None)
        | RuntimePassKind::DropShadowColorize(None)
        | RuntimePassKind::Composite(None) => &[],
    }
}

#[cfg(test)]
mod terminal_accounting_tests {
    use super::*;

    #[test]
    fn lowering_admits_the_terminal_pass_after_the_complete_graph_prefix() {
        let setup = Account::for_test();
        let commands = crate::Scene::new()
            .normalize(crate::Capabilities::CURRENT, &setup)
            .unwrap();
        let context = crate::frame::FrameContext::try_new(
            crate::Size::new(4.0, 4.0),
            1.0,
            crate::Antialiasing::Area,
            crate::Color::TRANSPARENT,
        )
        .unwrap();
        let graph = crate::frame::forced_base_graph_for_test(commands, context).unwrap();
        // The real empty-scene graph contains one resource and Clear/Present.
        // Its validation, views, indexes and lowering require 25 node visits
        // after removing four duplicate projections of Clear and Present.
        // The terminal runtime record requires a separate visit.
        for limit in [26, 25] {
            let (result, report) = crate::pass::test_support::measure_proof_work(
                RenderWork::GraphNodeVisits,
                limit,
                |account| {
                    let lowered = LoweredGraphPlan::try_lower_for_dispatch_classification(
                        &graph,
                        WorkingFormat::HighPrecision,
                        Format::Rgba8,
                        account,
                    )?;
                    assert_eq!(lowered.resources.len(), 1);
                    assert_eq!(lowered.passes.len(), 2);
                    assert_eq!(lowered.resources[0].id, lowered.root_working_image);
                    assert_eq!(lowered.passes[1].id, lowered.final_present);
                    assert!(matches!(lowered.passes[1].kind, RuntimePassKind::Present));
                    Ok(())
                },
            );
            let attempt = report.attempt().unwrap();
            if limit == 26 {
                result.unwrap();
                assert_eq!(attempt.usage().work(RenderWork::GraphNodeVisits), 26);
            } else {
                assert!(
                    result.is_err(),
                    "terminal inspection requires its own admission"
                );
                assert_eq!(attempt.status(), crate::RenderAttemptStatus::Refused);
                let refusal = attempt.refusal().unwrap();
                assert_eq!(
                    refusal.quantity,
                    crate::RenderQuantity::Work(RenderWork::GraphNodeVisits)
                );
                assert_eq!((refusal.current, refusal.amount), (25, 1));
            }
        }
    }
}
