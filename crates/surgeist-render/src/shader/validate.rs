use crate::{
    BackendErrorCode, Error, Result,
    accounting::{Account, RenderWork},
};

use super::key::{
    BindGroupLayoutKey, RenderPipelineKey, SampledTextureLayoutKey, SamplerKey,
    ShaderBindingRoleKey, ShaderCompositeKey, ShaderCompositePathKey, ShaderDataBindingKey,
    ShaderModuleKey, ShaderProgramKey, ShaderSamplingEdgeKey, ShaderSamplingFilterKey,
    ShaderTextureFormatKey,
};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(super) enum CorePassProgram {
    CanonicalizeCapture,
    SpanSourceOver,
    DropShadowMerge,
    Present,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(super) struct CorePassDescription {
    pub(super) program: CorePassProgram,
    pub(super) target_format: ShaderTextureFormatKey,
}

#[derive(Clone, Copy)]
pub(crate) struct CorePassKeyRefs<'a> {
    pub(crate) samplers: &'a [SamplerKey],
    pub(crate) layout: &'a BindGroupLayoutKey,
    pub(crate) shader: &'a ShaderModuleKey,
    pub(crate) pipeline: &'a RenderPipelineKey,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(super) struct CompositePassDescription {
    pub(super) path: ShaderCompositePathKey,
    pub(super) has_clip_coverage: bool,
    pub(super) has_alpha_mask: bool,
    pub(super) target_format: ShaderTextureFormatKey,
}

#[derive(Clone, Copy)]
pub(crate) struct CompositePassKeyRefs<'a> {
    pub(crate) samplers: &'a [SamplerKey],
    pub(crate) layout: &'a BindGroupLayoutKey,
    pub(crate) shader: &'a ShaderModuleKey,
    pub(crate) pipeline: &'a RenderPipelineKey,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(super) struct ColorFilterPassDescription {
    pub(super) working_format: ShaderTextureFormatKey,
}

#[derive(Clone, Copy)]
pub(crate) struct ColorFilterPassKeyRefs<'a> {
    pub(crate) samplers: &'a [SamplerKey],
    pub(crate) layout: &'a BindGroupLayoutKey,
    pub(crate) shader: &'a ShaderModuleKey,
    pub(crate) pipeline: &'a RenderPipelineKey,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(super) struct CopyBackdropPassDescription {
    pub(super) working_format: ShaderTextureFormatKey,
}

#[derive(Clone, Copy)]
pub(crate) struct CopyBackdropPassKeyRefs<'a> {
    pub(crate) samplers: &'a [SamplerKey],
    pub(crate) layout: &'a BindGroupLayoutKey,
    pub(crate) shader: &'a ShaderModuleKey,
    pub(crate) pipeline: &'a RenderPipelineKey,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(super) enum BlurAxis {
    Horizontal,
    Vertical,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(super) enum BlurInput {
    Rgba,
    SourceAlpha,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(super) struct BlurPassDescription {
    pub(super) axis: BlurAxis,
    pub(super) input: BlurInput,
    pub(super) edge: ShaderSamplingEdgeKey,
    pub(super) working_format: ShaderTextureFormatKey,
}

#[derive(Clone, Copy)]
pub(crate) struct BlurPassKeyRefs<'a> {
    pub(crate) samplers: &'a [SamplerKey],
    pub(crate) layout: &'a BindGroupLayoutKey,
    pub(crate) shader: &'a ShaderModuleKey,
    pub(crate) pipeline: &'a RenderPipelineKey,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(super) struct DropShadowColorizePassDescription {
    pub(super) working_format: ShaderTextureFormatKey,
}

#[derive(Clone, Copy)]
pub(crate) struct DropShadowColorizePassKeyRefs<'a> {
    pub(crate) samplers: &'a [SamplerKey],
    pub(crate) layout: &'a BindGroupLayoutKey,
    pub(crate) shader: &'a ShaderModuleKey,
    pub(crate) pipeline: &'a RenderPipelineKey,
}

pub(super) fn validate_core_pass_keys(
    keys: CorePassKeyRefs<'_>,
    observer: Option<&Account>,
) -> Result<CorePassDescription> {
    let [sampled_texture] = keys.layout.sampled_textures.as_slice() else {
        return Err(core_pass_cache_error(
            "a core pass layout must bind exactly one sampled texture",
        ));
    };
    binding_visit(observer)?;
    let [sampler] = keys.samplers else {
        return Err(core_pass_cache_error(
            "a core pass must bind exactly one exact sampler",
        ));
    };
    binding_visit(observer)?;
    validate_core_pass_key_consistency(keys, sampled_texture, sampler, observer)?;
    let (program, expected_role, expected_filter, expected_edge) =
        core_pass_program_sampling(keys.layout.program)?;
    if sampled_texture.binding_role != expected_role
        || sampler.filter != expected_filter
        || sampler.edge != expected_edge
    {
        return Err(core_pass_cache_error(
            "core pass sampled texture and sampler semantics are not exact",
        ));
    }

    let Some(working_format) = keys.shader.working_format else {
        return Err(core_pass_cache_error(
            "core-pass shader key has no selected working format",
        ));
    };
    if !is_working_format(working_format) {
        return Err(core_pass_cache_error(
            "core-pass shader key selected a non-working intermediate format",
        ));
    }
    let target_format = core_pass_target_format(keys, sampled_texture, program, working_format)?;
    Ok(CorePassDescription {
        program,
        target_format,
    })
}

fn validate_core_pass_key_consistency(
    keys: CorePassKeyRefs<'_>,
    sampled_texture: &SampledTextureLayoutKey,
    sampler: &SamplerKey,
    observer: Option<&Account>,
) -> Result<()> {
    if !binding_eq(
        keys.layout.data_bindings.as_slice(),
        &[ShaderDataBindingKey::SpatialUniform],
        observer,
    )? || !bindings_exclude(
        keys.layout.data_bindings.as_slice(),
        &[
            ShaderDataBindingKey::CompositeParameters,
            ShaderDataBindingKey::PresentParameters,
        ],
        observer,
    )? || keys.shader.program != keys.layout.program
        || !layout_eq(&keys.shader.layout, keys.layout, observer)?
        || !binding_eq(keys.shader.samplers.as_slice(), keys.samplers, observer)?
        || !shader_eq(&keys.pipeline.shader, keys.shader, observer)?
        || !layout_eq(&keys.pipeline.layout, keys.layout, observer)?
        || !binding_eq(keys.pipeline.samplers.as_slice(), keys.samplers, observer)?
        || sampler.binding_role != sampled_texture.binding_role
        || sampler.source_format != sampled_texture.source_format
        || sampler.resolved_mask_sampling.is_some()
    {
        return Err(core_pass_cache_error(
            "core pass keys disagree across sampler, layout, shader, or pipeline phases",
        ));
    }
    Ok(())
}

fn core_pass_program_sampling(
    program: ShaderProgramKey,
) -> Result<(
    CorePassProgram,
    ShaderBindingRoleKey,
    ShaderSamplingFilterKey,
    ShaderSamplingEdgeKey,
)> {
    let sampling = match program {
        ShaderProgramKey::CanonicalizeCapture => (
            CorePassProgram::CanonicalizeCapture,
            ShaderBindingRoleKey::CaptureSource,
            ShaderSamplingFilterKey::Linear,
            ShaderSamplingEdgeKey::ClampToExtent,
        ),
        ShaderProgramKey::Composite(ShaderCompositeKey::SpanSourceOver) => (
            CorePassProgram::SpanSourceOver,
            ShaderBindingRoleKey::CompositeSource,
            ShaderSamplingFilterKey::Linear,
            ShaderSamplingEdgeKey::TransparentBlack,
        ),
        ShaderProgramKey::Composite(ShaderCompositeKey::DropShadow) => (
            CorePassProgram::DropShadowMerge,
            ShaderBindingRoleKey::CompositeSource,
            ShaderSamplingFilterKey::Linear,
            ShaderSamplingEdgeKey::TransparentBlack,
        ),
        ShaderProgramKey::Present => (
            CorePassProgram::Present,
            ShaderBindingRoleKey::FinalWorkingImage,
            ShaderSamplingFilterKey::Linear,
            ShaderSamplingEdgeKey::ClampToExtent,
        ),
        ShaderProgramKey::CopyBackdrop
        | ShaderProgramKey::Convolution
        | ShaderProgramKey::Offset
        | ShaderProgramKey::ColorFilter
        | ShaderProgramKey::BlurHorizontal { .. }
        | ShaderProgramKey::BlurVertical { .. }
        | ShaderProgramKey::DropShadowColorize
        | ShaderProgramKey::Composite(ShaderCompositeKey::Layer { .. }) => {
            return Err(core_pass_cache_error(
                "an unsupported shader program reached core pass realization",
            ));
        }
    };
    Ok(sampling)
}

fn core_pass_target_format(
    keys: CorePassKeyRefs<'_>,
    sampled_texture: &SampledTextureLayoutKey,
    program: CorePassProgram,
    working_format: ShaderTextureFormatKey,
) -> Result<ShaderTextureFormatKey> {
    let target_format = match program {
        CorePassProgram::CanonicalizeCapture => {
            if sampled_texture.source_format != ShaderTextureFormatKey::VelloCaptureRgba8Unorm
                || keys.shader.output_format.is_some()
                || keys.pipeline.target_format != working_format
            {
                return Err(core_pass_cache_error(
                    "canonicalization keys changed capture or working formats",
                ));
            }
            working_format
        }
        CorePassProgram::SpanSourceOver | CorePassProgram::DropShadowMerge => {
            if sampled_texture.source_format != working_format
                || keys.shader.output_format.is_some()
                || keys.pipeline.target_format != working_format
            {
                return Err(core_pass_cache_error(
                    "source-over keys changed source or working target formats",
                ));
            }
            working_format
        }
        CorePassProgram::Present => {
            let Some(output_format) = keys.shader.output_format else {
                return Err(core_pass_cache_error(
                    "present key has no exact output format",
                ));
            };
            if sampled_texture.source_format != working_format
                || !is_output_format(output_format)
                || keys.pipeline.target_format != output_format
            {
                return Err(core_pass_cache_error(
                    "present keys changed working source or output specialization",
                ));
            }
            output_format
        }
    };
    Ok(target_format)
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(super) struct ConvolutionPassDescription {
    pub(super) target_format: ShaderTextureFormatKey,
}

#[derive(Clone, Copy)]
pub(crate) struct ConvolutionPassKeyRefs<'a> {
    pub(crate) samplers: &'a [SamplerKey],
    pub(crate) layout: &'a BindGroupLayoutKey,
    pub(crate) shader: &'a ShaderModuleKey,
    pub(crate) pipeline: &'a RenderPipelineKey,
}

pub(super) fn validate_convolution_pass_keys(
    keys: ConvolutionPassKeyRefs<'_>,
    observer: Option<&Account>,
) -> Result<ConvolutionPassDescription> {
    let [source, auxiliary] = keys.layout.sampled_textures.as_slice() else {
        return Err(convolution_cache_error(
            "convolution requires exactly two texture bindings",
        ));
    };
    binding_visit(observer)?;
    binding_visit(observer)?;
    let valid_input = |format| {
        is_working_format(format)
            || matches!(
                format,
                ShaderTextureFormatKey::ConvolutionRgba32Float
                    | ShaderTextureFormatKey::VelloCaptureRgba8Unorm
            )
    };
    let target_format = keys.pipeline.target_format;
    if keys.layout.program != ShaderProgramKey::Convolution
        || keys.shader.program != ShaderProgramKey::Convolution
        || !keys.samplers.is_empty()
        || !keys.shader.samplers.is_empty()
        || !keys.pipeline.samplers.is_empty()
        || !binding_eq(
            keys.layout.data_bindings.as_slice(),
            &[
                ShaderDataBindingKey::ConvolutionParameters,
                ShaderDataBindingKey::ConvolutionWeights,
            ],
            observer,
        )?
        || !layout_eq(&keys.shader.layout, keys.layout, observer)?
        || !shader_eq(&keys.pipeline.shader, keys.shader, observer)?
        || !layout_eq(&keys.pipeline.layout, keys.layout, observer)?
        || keys.shader.working_format != Some(ShaderTextureFormatKey::ConvolutionRgba32Float)
        || keys.shader.output_format != Some(target_format)
        || source.binding_role != ShaderBindingRoleKey::ConvolutionSource
        || auxiliary.binding_role != ShaderBindingRoleKey::ConvolutionAuxiliary
        || !valid_input(source.source_format)
        || !valid_input(auxiliary.source_format)
        || !(is_working_format(target_format)
            || target_format == ShaderTextureFormatKey::ConvolutionRgba32Float)
    {
        return Err(convolution_cache_error(
            "convolution keys disagree across bindings, programs, or formats",
        ));
    }
    Ok(ConvolutionPassDescription { target_format })
}

pub(super) fn convolution_cache_error(message: &'static str) -> Error {
    Error::new(BackendErrorCode::RenderFailed, message)
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(super) struct OffsetPassDescription {
    pub(super) target_format: ShaderTextureFormatKey,
}

#[derive(Clone, Copy)]
pub(crate) struct OffsetPassKeyRefs<'a> {
    pub(crate) samplers: &'a [SamplerKey],
    pub(crate) layout: &'a BindGroupLayoutKey,
    pub(crate) shader: &'a ShaderModuleKey,
    pub(crate) pipeline: &'a RenderPipelineKey,
}

pub(super) fn validate_offset_pass_keys(
    keys: OffsetPassKeyRefs<'_>,
    observer: Option<&Account>,
) -> Result<OffsetPassDescription> {
    let [source] = keys.layout.sampled_textures.as_slice() else {
        return Err(offset_cache_error(
            "offset requires exactly one source texture",
        ));
    };
    binding_visit(observer)?;
    let target_format = keys.pipeline.target_format;
    if keys.layout.program != ShaderProgramKey::Offset
        || keys.shader.program != ShaderProgramKey::Offset
        || !keys.samplers.is_empty()
        || !keys.shader.samplers.is_empty()
        || !keys.pipeline.samplers.is_empty()
        || !binding_eq(
            keys.layout.data_bindings.as_slice(),
            &[ShaderDataBindingKey::OffsetParameters],
            observer,
        )?
        || !layout_eq(&keys.shader.layout, keys.layout, observer)?
        || !shader_eq(&keys.pipeline.shader, keys.shader, observer)?
        || !layout_eq(&keys.pipeline.layout, keys.layout, observer)?
        || keys.shader.working_format != Some(target_format)
        || keys.shader.output_format != Some(target_format)
        || source.binding_role != ShaderBindingRoleKey::OffsetSource
        || source.source_format != target_format
        || !is_working_format(target_format)
    {
        return Err(offset_cache_error(
            "offset keys disagree across bindings, programs, or working formats",
        ));
    }
    Ok(OffsetPassDescription { target_format })
}

pub(super) fn offset_cache_error(message: &'static str) -> Error {
    Error::new(BackendErrorCode::RenderFailed, message)
}

pub(super) fn validate_color_filter_pass_keys(
    keys: ColorFilterPassKeyRefs<'_>,
    observer: Option<&Account>,
) -> Result<ColorFilterPassDescription> {
    let [sampled_texture] = keys.layout.sampled_textures.as_slice() else {
        return Err(color_filter_cache_error(
            "a color-filter layout must bind exactly one sampled texture",
        ));
    };
    binding_visit(observer)?;
    let [sampler] = keys.samplers else {
        return Err(color_filter_cache_error(
            "a color-filter pass must bind exactly one source sampler",
        ));
    };
    binding_visit(observer)?;
    let Some(working_format) = keys.shader.working_format else {
        return Err(color_filter_cache_error(
            "a color-filter shader key has no selected working format",
        ));
    };
    if keys.layout.program != ShaderProgramKey::ColorFilter
        || !binding_eq(
            keys.layout.data_bindings.as_slice(),
            &[
                ShaderDataBindingKey::SpatialUniform,
                ShaderDataBindingKey::ColorFilterOperations,
            ],
            observer,
        )?
        || keys.shader.program != ShaderProgramKey::ColorFilter
        || !layout_eq(&keys.shader.layout, keys.layout, observer)?
        || !binding_eq(keys.shader.samplers.as_slice(), keys.samplers, observer)?
        || keys.shader.output_format.is_some()
        || !shader_eq(&keys.pipeline.shader, keys.shader, observer)?
        || !layout_eq(&keys.pipeline.layout, keys.layout, observer)?
        || !binding_eq(keys.pipeline.samplers.as_slice(), keys.samplers, observer)?
        || !is_working_format(working_format)
        || keys.pipeline.target_format != working_format
        || sampled_texture.binding_role != ShaderBindingRoleKey::FilterSource
        || sampled_texture.source_format != working_format
        || sampler.binding_role != ShaderBindingRoleKey::FilterSource
        || sampler.source_format != working_format
        || sampler.filter != ShaderSamplingFilterKey::Nearest
        || sampler.edge != ShaderSamplingEdgeKey::ClampToExtent
        || sampler.resolved_mask_sampling.is_some()
    {
        return Err(color_filter_cache_error(
            "color-filter keys disagree across source, layout, shader, or working target",
        ));
    }
    Ok(ColorFilterPassDescription { working_format })
}

pub(super) fn validate_copy_backdrop_pass_keys(
    keys: CopyBackdropPassKeyRefs<'_>,
    observer: Option<&Account>,
) -> Result<CopyBackdropPassDescription> {
    let [sampled_texture] = keys.layout.sampled_textures.as_slice() else {
        return Err(copy_backdrop_cache_error(
            "a backdrop-copy layout must bind exactly one sampled texture",
        ));
    };
    binding_visit(observer)?;
    let [sampler] = keys.samplers else {
        return Err(copy_backdrop_cache_error(
            "a backdrop-copy pass must bind exactly one parent sampler",
        ));
    };
    binding_visit(observer)?;
    let Some(working_format) = keys.shader.working_format else {
        return Err(copy_backdrop_cache_error(
            "a backdrop-copy shader key has no selected working format",
        ));
    };
    if keys.layout.program != ShaderProgramKey::CopyBackdrop
        || !binding_eq(
            keys.layout.data_bindings.as_slice(),
            &[ShaderDataBindingKey::SpatialUniform],
            observer,
        )?
        || keys.shader.program != ShaderProgramKey::CopyBackdrop
        || !layout_eq(&keys.shader.layout, keys.layout, observer)?
        || !binding_eq(keys.shader.samplers.as_slice(), keys.samplers, observer)?
        || keys.shader.output_format.is_some()
        || !shader_eq(&keys.pipeline.shader, keys.shader, observer)?
        || !layout_eq(&keys.pipeline.layout, keys.layout, observer)?
        || !binding_eq(keys.pipeline.samplers.as_slice(), keys.samplers, observer)?
        || !is_working_format(working_format)
        || keys.pipeline.target_format != working_format
        || sampled_texture.binding_role != ShaderBindingRoleKey::CompletedParent
        || sampled_texture.source_format != working_format
        || sampler.binding_role != ShaderBindingRoleKey::CompletedParent
        || sampler.source_format != working_format
        || sampler.filter != ShaderSamplingFilterKey::Nearest
        || sampler.edge != ShaderSamplingEdgeKey::TransparentBlack
        || sampler.resolved_mask_sampling.is_some()
    {
        return Err(copy_backdrop_cache_error(
            "backdrop-copy keys disagree across parent, layout, shader, or working target",
        ));
    }
    Ok(CopyBackdropPassDescription { working_format })
}

pub(super) fn validate_blur_pass_keys(
    keys: BlurPassKeyRefs<'_>,
    observer: Option<&Account>,
) -> Result<BlurPassDescription> {
    let [sampled_texture] = keys.layout.sampled_textures.as_slice() else {
        return Err(blur_cache_error(
            "a blur layout must bind exactly one sampled texture",
        ));
    };
    binding_visit(observer)?;
    let [sampler] = keys.samplers else {
        return Err(blur_cache_error(
            "a blur pass must bind exactly one source sampler",
        ));
    };
    binding_visit(observer)?;
    let (axis, input, edge) = blur_program_facts(keys.layout.program).ok_or_else(|| {
        blur_cache_error("a non-blur program reached spatial-filter blur realization")
    })?;
    let Some(working_format) = keys.shader.working_format else {
        return Err(blur_cache_error(
            "a blur shader key has no selected working format",
        ));
    };
    let expected_data_bindings: &[ShaderDataBindingKey] = match edge {
        ShaderSamplingEdgeKey::TransparentBlack => &[
            ShaderDataBindingKey::SpatialUniform,
            ShaderDataBindingKey::GaussianKernel,
        ],
        ShaderSamplingEdgeKey::SemanticBorderMirror => &[
            ShaderDataBindingKey::SpatialUniform,
            ShaderDataBindingKey::GaussianKernel,
            ShaderDataBindingKey::BlurEdgeParameters,
        ],
        ShaderSamplingEdgeKey::ClampToExtent => {
            return Err(blur_cache_error(
                "a Gaussian blur program has no clamp-to-extent edge policy",
            ));
        }
    };
    if !binding_eq(
        keys.layout.data_bindings.as_slice(),
        expected_data_bindings,
        observer,
    )? || keys.shader.program != keys.layout.program
        || !layout_eq(&keys.shader.layout, keys.layout, observer)?
        || !binding_eq(keys.shader.samplers.as_slice(), keys.samplers, observer)?
        || keys.shader.output_format.is_some()
        || !shader_eq(&keys.pipeline.shader, keys.shader, observer)?
        || !layout_eq(&keys.pipeline.layout, keys.layout, observer)?
        || !binding_eq(keys.pipeline.samplers.as_slice(), keys.samplers, observer)?
        || !is_working_format(working_format)
        || keys.pipeline.target_format != working_format
        || sampled_texture.binding_role != ShaderBindingRoleKey::FilterSource
        || sampled_texture.source_format != working_format
        || sampler.binding_role != ShaderBindingRoleKey::FilterSource
        || sampler.source_format != working_format
        || sampler.filter != ShaderSamplingFilterKey::Linear
        || sampler.edge != edge
        || sampler.resolved_mask_sampling.is_some()
    {
        return Err(blur_cache_error(
            "blur keys disagree across source, layout, shader, or working target",
        ));
    }
    Ok(BlurPassDescription {
        axis,
        input,
        edge,
        working_format,
    })
}

pub(super) fn validate_drop_shadow_colorize_pass_keys(
    keys: DropShadowColorizePassKeyRefs<'_>,
    observer: Option<&Account>,
) -> Result<DropShadowColorizePassDescription> {
    let [sampled_texture] = keys.layout.sampled_textures.as_slice() else {
        return Err(drop_shadow_cache_error(
            "a drop-shadow colorize layout must bind exactly one sampled texture",
        ));
    };
    binding_visit(observer)?;
    let [sampler] = keys.samplers else {
        return Err(drop_shadow_cache_error(
            "a drop-shadow colorize pass must bind exactly one source sampler",
        ));
    };
    binding_visit(observer)?;
    let Some(working_format) = keys.shader.working_format else {
        return Err(drop_shadow_cache_error(
            "a drop-shadow colorize shader key has no selected working format",
        ));
    };
    if keys.layout.program != ShaderProgramKey::DropShadowColorize
        || !binding_eq(
            keys.layout.data_bindings.as_slice(),
            &[
                ShaderDataBindingKey::SpatialUniform,
                ShaderDataBindingKey::DropShadowParameters,
            ],
            observer,
        )?
        || keys.shader.program != ShaderProgramKey::DropShadowColorize
        || !layout_eq(&keys.shader.layout, keys.layout, observer)?
        || !binding_eq(keys.shader.samplers.as_slice(), keys.samplers, observer)?
        || keys.shader.output_format.is_some()
        || !shader_eq(&keys.pipeline.shader, keys.shader, observer)?
        || !layout_eq(&keys.pipeline.layout, keys.layout, observer)?
        || !binding_eq(keys.pipeline.samplers.as_slice(), keys.samplers, observer)?
        || !is_working_format(working_format)
        || keys.pipeline.target_format != working_format
        || sampled_texture.binding_role != ShaderBindingRoleKey::BlurredSourceAlpha
        || sampled_texture.source_format != working_format
        || sampler.binding_role != ShaderBindingRoleKey::BlurredSourceAlpha
        || sampler.source_format != working_format
        || sampler.filter != ShaderSamplingFilterKey::Linear
        || sampler.edge != ShaderSamplingEdgeKey::TransparentBlack
        || sampler.resolved_mask_sampling.is_some()
    {
        return Err(drop_shadow_cache_error(
            "drop-shadow colorize keys disagree across source, layout, shader, or target",
        ));
    }
    Ok(DropShadowColorizePassDescription { working_format })
}

const fn blur_program_facts(
    program: ShaderProgramKey,
) -> Option<(BlurAxis, BlurInput, ShaderSamplingEdgeKey)> {
    match program {
        ShaderProgramKey::BlurHorizontal { source_alpha, edge } => Some((
            BlurAxis::Horizontal,
            if source_alpha {
                BlurInput::SourceAlpha
            } else {
                BlurInput::Rgba
            },
            edge,
        )),
        ShaderProgramKey::BlurVertical { source_alpha, edge } => Some((
            BlurAxis::Vertical,
            if source_alpha {
                BlurInput::SourceAlpha
            } else {
                BlurInput::Rgba
            },
            edge,
        )),
        _ => None,
    }
}

#[cfg(test)]
pub(super) const fn is_blur_program(program: ShaderProgramKey) -> bool {
    blur_program_facts(program).is_some()
}

pub(super) fn validate_composite_pass_keys(
    keys: CompositePassKeyRefs<'_>,
    observer: Option<&Account>,
) -> Result<CompositePassDescription> {
    let ShaderProgramKey::Composite(ShaderCompositeKey::Layer {
        path,
        has_clip_coverage,
        has_alpha_mask,
    }) = keys.layout.program
    else {
        return Err(core_pass_cache_error(
            "a non-layer program reached composition composite realization",
        ));
    };
    let Some(working_format) = keys.shader.working_format else {
        return Err(core_pass_cache_error(
            "composition composite shader key has no selected working format",
        ));
    };
    if !is_working_format(working_format)
        || !binding_eq(
            keys.layout.data_bindings.as_slice(),
            &[
                ShaderDataBindingKey::SpatialUniform,
                ShaderDataBindingKey::CompositeParameters,
            ],
            observer,
        )?
        || keys.shader.program != keys.layout.program
        || !layout_eq(&keys.shader.layout, keys.layout, observer)?
        || !binding_eq(keys.shader.samplers.as_slice(), keys.samplers, observer)?
        || keys.shader.output_format.is_some()
        || !shader_eq(&keys.pipeline.shader, keys.shader, observer)?
        || !layout_eq(&keys.pipeline.layout, keys.layout, observer)?
        || !binding_eq(keys.pipeline.samplers.as_slice(), keys.samplers, observer)?
        || keys.pipeline.target_format != working_format
    {
        return Err(core_pass_cache_error(
            "composition composite keys disagree across layout, shader, format, or pipeline phases",
        ));
    }

    let [source_sampler] = keys.samplers else {
        return Err(core_pass_cache_error(
            "composition composite pass must bind exactly one source sampler",
        ));
    };
    binding_visit(observer)?;
    if source_sampler.binding_role != ShaderBindingRoleKey::CompositeSource
        || source_sampler.source_format != working_format
        || source_sampler.filter != ShaderSamplingFilterKey::Linear
        || source_sampler.edge != ShaderSamplingEdgeKey::TransparentBlack
        || source_sampler.resolved_mask_sampling.is_some()
    {
        return Err(core_pass_cache_error(
            "composition composite source sampler semantics are not exact",
        ));
    }

    let expected_textures = [
        (path == ShaderCompositePathKey::DestinationSampling).then_some(SampledTextureLayoutKey {
            binding_role: ShaderBindingRoleKey::CompositeParent,
            source_format: working_format,
        }),
        Some(SampledTextureLayoutKey {
            binding_role: ShaderBindingRoleKey::CompositeSource,
            source_format: working_format,
        }),
        has_clip_coverage.then_some(SampledTextureLayoutKey {
            binding_role: ShaderBindingRoleKey::ClipCoverage,
            source_format: ShaderTextureFormatKey::ClipCoverageRgba8Unorm,
        }),
        has_alpha_mask.then_some(SampledTextureLayoutKey {
            binding_role: ShaderBindingRoleKey::AlphaMask,
            source_format: ShaderTextureFormatKey::ResolvedMaskRgba8Unorm,
        }),
    ];
    if !binding_eq_iter(
        keys.layout.sampled_textures.as_slice(),
        expected_textures.into_iter().flatten(),
        observer,
    )? {
        return Err(core_pass_cache_error(
            "composition composite layout contains an absent or missing semantic texture",
        ));
    }

    Ok(CompositePassDescription {
        path,
        has_clip_coverage,
        has_alpha_mask,
        target_format: working_format,
    })
}

const fn is_working_format(format: ShaderTextureFormatKey) -> bool {
    matches!(
        format,
        ShaderTextureFormatKey::WorkingHighPrecisionRgba16Float
            | ShaderTextureFormatKey::WorkingReducedPrecisionRgba8Unorm
    )
}

const fn is_output_format(format: ShaderTextureFormatKey) -> bool {
    matches!(
        format,
        ShaderTextureFormatKey::OutputRgba8Unorm | ShaderTextureFormatKey::OutputBgra8Unorm
    )
}

pub(super) fn core_pass_cache_error(message: &'static str) -> Error {
    Error::new(BackendErrorCode::RenderFailed, message)
}

pub(super) fn color_filter_cache_error(message: &'static str) -> Error {
    Error::new(BackendErrorCode::RenderFailed, message)
}

pub(super) fn copy_backdrop_cache_error(message: &'static str) -> Error {
    Error::new(BackendErrorCode::RenderFailed, message)
}

pub(super) fn blur_cache_error(message: &'static str) -> Error {
    Error::new(BackendErrorCode::RenderFailed, message)
}

pub(super) fn drop_shadow_cache_error(message: &'static str) -> Error {
    Error::new(BackendErrorCode::RenderFailed, message)
}

fn binding_visit(observer: Option<&Account>) -> Result<()> {
    if let Some(observer) = observer {
        observer.work(RenderWork::ShaderBindingVisits, 1)?;
    }
    Ok(())
}
fn bindings_exclude(
    bindings: &[ShaderDataBindingKey],
    excluded: &[ShaderDataBindingKey],
    observer: Option<&Account>,
) -> Result<bool> {
    for binding in bindings {
        binding_visit(observer)?;
        if excluded.contains(binding) {
            return Ok(false);
        }
    }
    Ok(true)
}

fn binding_eq<T: PartialEq>(left: &[T], right: &[T], observer: Option<&Account>) -> Result<bool> {
    if left.len() != right.len() {
        return Ok(false);
    }
    for (left, right) in left.iter().zip(right) {
        binding_visit(observer)?;
        if left != right {
            return Ok(false);
        }
    }
    Ok(true)
}
fn binding_eq_iter<T: PartialEq>(
    left: &[T],
    right: impl IntoIterator<Item = T>,
    observer: Option<&Account>,
) -> Result<bool> {
    let mut left = left.iter();
    for right in right {
        let Some(left) = left.next() else {
            return Ok(false);
        };
        binding_visit(observer)?;
        if *left != right {
            return Ok(false);
        }
    }
    if left.next().is_some() {
        binding_visit(observer)?;
        return Ok(false);
    }
    Ok(true)
}
fn layout_eq(
    left: &BindGroupLayoutKey,
    right: &BindGroupLayoutKey,
    observer: Option<&Account>,
) -> Result<bool> {
    Ok(left.program == right.program
        && binding_eq(
            left.sampled_textures.as_slice(),
            right.sampled_textures.as_slice(),
            observer,
        )?
        && binding_eq(
            left.data_bindings.as_slice(),
            right.data_bindings.as_slice(),
            observer,
        )?)
}
fn shader_eq(
    left: &ShaderModuleKey,
    right: &ShaderModuleKey,
    observer: Option<&Account>,
) -> Result<bool> {
    Ok(left.program == right.program
        && left.working_format == right.working_format
        && left.output_format == right.output_format
        && layout_eq(&left.layout, &right.layout, observer)?
        && binding_eq(
            left.samplers.as_slice(),
            right.samplers.as_slice(),
            observer,
        )?)
}

// Graph closure reconstructs these keys independently. Compare their content
// through the shader owner so every reached nested binding is admitted, even
// when equal keys happen to share immutable backing.
impl BindGroupLayoutKey {
    pub(crate) fn matches_in(&self, other: &Self, account: &Account) -> Result<bool> {
        layout_eq(self, other, Some(account))
    }
}

impl ShaderModuleKey {
    pub(crate) fn matches_in(&self, other: &Self, account: &Account) -> Result<bool> {
        shader_eq(self, other, Some(account))
    }
}

impl RenderPipelineKey {
    pub(crate) fn matches_in(&self, other: &Self, account: &Account) -> Result<bool> {
        Ok(self.target_format == other.target_format
            && shader_eq(&self.shader, &other.shader, Some(account))?
            && layout_eq(&self.layout, &other.layout, Some(account))?
            && binding_eq(
                self.samplers.as_slice(),
                other.samplers.as_slice(),
                Some(account),
            )?)
    }
}

#[cfg(test)]
mod accounting_tests {
    use super::*;
    use crate::accounting::RendererCpu;
    use crate::{GlyphRunLimits, RenderAccountingContext, RenderAttemptKind};

    struct KeyFixture {
        layout: BindGroupLayoutKey,
        shader: ShaderModuleKey,
        pipeline: RenderPipelineKey,
    }

    fn keys(
        data: &[ShaderDataBindingKey],
        shader_edge: ShaderSamplingEdgeKey,
        pipeline_edge: ShaderSamplingEdgeKey,
    ) -> KeyFixture {
        let owner = RendererCpu::new(crate::RenderAccountingOptions::default()).unwrap();
        let cpu = owner.at(crate::RenderCpuScope::DeviceBacking);
        let format = ShaderTextureFormatKey::WorkingHighPrecisionRgba16Float;
        let layout = BindGroupLayoutKey::new(
            &cpu,
            None,
            ShaderProgramKey::ColorFilter,
            &[(ShaderBindingRoleKey::FilterSource, format)],
            data,
        )
        .unwrap();
        let sampler = |edge| {
            SamplerKey::new(
                ShaderBindingRoleKey::FilterSource,
                format,
                ShaderSamplingFilterKey::Nearest,
                edge,
                None,
            )
        };
        let shader = ShaderModuleKey::new(
            &cpu,
            None,
            ShaderProgramKey::ColorFilter,
            layout.clone(),
            &[sampler(shader_edge)],
            Some(format),
            None,
        )
        .unwrap();
        let pipeline = RenderPipelineKey::new(
            &cpu,
            None,
            shader.clone(),
            layout.clone(),
            &[sampler(pipeline_edge)],
            format,
        )
        .unwrap();
        KeyFixture {
            layout,
            shader,
            pipeline,
        }
    }

    fn measured_comparison(
        limit: u64,
        compare: impl FnOnce(&Account) -> Result<bool>,
    ) -> (Result<bool>, crate::RenderStandaloneReport) {
        let options = crate::RenderAccountingOptions::new().with_limits(
            crate::RenderLimits::new()
                .with_work_limit(RenderWork::ShaderBindingVisits, limit)
                .unwrap(),
        );
        let mut context = RenderAccountingContext::new(options);
        let result = context.run(RenderAttemptKind::Frame, crate::RenderPhase::Lower, compare);
        (result, context.take_last_attempt().unwrap())
    }

    #[test]
    fn reconstructed_shader_keys_admit_each_nested_binding_and_refuse_one_under() {
        let data = [
            ShaderDataBindingKey::SpatialUniform,
            ShaderDataBindingKey::ColorFilterOperations,
        ];
        let left = keys(
            &data,
            ShaderSamplingEdgeKey::ClampToExtent,
            ShaderSamplingEdgeKey::ClampToExtent,
        );
        let right = keys(
            &data,
            ShaderSamplingEdgeKey::ClampToExtent,
            ShaderSamplingEdgeKey::ClampToExtent,
        );
        assert!(!std::ptr::eq(
            &*left.layout.sampled_textures,
            &*right.layout.sampled_textures,
        ));
        // Layout: one texture + two data bindings. Shader: layout + one
        // sampler. Pipeline: shader + layout + its own sampler.
        type Comparison = fn(&KeyFixture, &KeyFixture, &Account) -> Result<bool>;
        let comparisons: [(u64, Comparison); 3] = [
            (3, |a, b, account| a.layout.matches_in(&b.layout, account)),
            (4, |a, b, account| a.shader.matches_in(&b.shader, account)),
            (8, |a, b, account| {
                a.pipeline.matches_in(&b.pipeline, account)
            }),
        ];
        for (visits, compare) in comparisons {
            for limit in [visits, visits - 1] {
                let (result, report) =
                    measured_comparison(limit, |account| compare(&left, &right, account));
                assert_eq!(
                    report
                        .attempt()
                        .unwrap()
                        .usage()
                        .work(RenderWork::ShaderBindingVisits),
                    limit
                );
                if limit == visits {
                    assert!(result.unwrap());
                } else {
                    let Some(crate::RenderAccountingError::Refused(refusal)) =
                        result.unwrap_err().accounting_diagnostic()
                    else {
                        panic!("the next nested binding must refuse");
                    };
                    assert_eq!(
                        refusal.quantity(),
                        crate::RenderQuantity::Work(RenderWork::ShaderBindingVisits)
                    );
                    assert_eq!((refusal.current(), refusal.amount()), (limit, 1));
                }
            }
        }
    }

    #[test]
    fn reconstructed_shader_key_difference_counts_only_the_reached_prefix() {
        let data = [
            ShaderDataBindingKey::SpatialUniform,
            ShaderDataBindingKey::ColorFilterOperations,
        ];
        let clamp = ShaderSamplingEdgeKey::ClampToExtent;
        let transparent = ShaderSamplingEdgeKey::TransparentBlack;
        let left = keys(&data, clamp, clamp);
        let changed_data = keys(
            &[
                ShaderDataBindingKey::SpatialUniform,
                ShaderDataBindingKey::GaussianKernel,
            ],
            clamp,
            clamp,
        );
        let changed_shader_sampler = keys(&data, transparent, clamp);
        let changed_pipeline_sampler = keys(&data, clamp, transparent);
        let short = keys(&data[..1], clamp, clamp);
        let mut changed_target = keys(&data, clamp, clamp);
        changed_target.pipeline.target_format = ShaderTextureFormatKey::OutputBgra8Unorm;
        let mut changed_program = keys(&data, clamp, clamp);
        changed_program.shader.program = ShaderProgramKey::Present;
        type PrefixCase<'a> = (u64, &'a dyn Fn(&Account) -> Result<bool>);
        let cases: [PrefixCase<'_>; 6] = [
            (3, &|account| {
                left.layout.matches_in(&changed_data.layout, account)
            }),
            (4, &|account| {
                left.shader
                    .matches_in(&changed_shader_sampler.shader, account)
            }),
            (8, &|account| {
                left.pipeline
                    .matches_in(&changed_pipeline_sampler.pipeline, account)
            }),
            (1, &|account| left.layout.matches_in(&short.layout, account)),
            (0, &|account| {
                left.pipeline.matches_in(&changed_target.pipeline, account)
            }),
            (0, &|account| {
                left.shader.matches_in(&changed_program.shader, account)
            }),
        ];
        for (visits, compare) in cases {
            let (result, report) = measured_comparison(visits, compare);
            assert!(!result.unwrap());
            assert_eq!(
                report
                    .attempt()
                    .unwrap()
                    .usage()
                    .work(RenderWork::ShaderBindingVisits),
                visits
            );
        }
    }

    #[test]
    fn binding_comparison_stops_charging_at_the_first_unequal_element() {
        let mut context = RenderAccountingContext::default();
        context
            .prepare(
                RenderAttemptKind::GlyphRun,
                GlyphRunLimits::new(0, 0, 0, 0),
                #[cfg(feature = "render-text")]
                None,
                |account| {
                    assert!(!binding_eq(&[1, 2, 3], &[1, 7, 3], Some(account))?);
                    assert!(!binding_eq(&[1, 2], &[1], Some(account))?);
                    Ok(())
                },
            )
            .unwrap();
        assert_eq!(
            context
                .last_attempt()
                .unwrap()
                .attempt()
                .unwrap()
                .usage()
                .work(RenderWork::ShaderBindingVisits),
            2
        );
    }
}

#[cfg(test)]
mod convolution_key_tests {
    use super::*;
    use crate::accounting::RendererCpu;

    fn fixture() -> (BindGroupLayoutKey, ShaderModuleKey, RenderPipelineKey) {
        let owner = RendererCpu::new(crate::RenderAccountingOptions::default()).unwrap();
        let cpu = owner.at(crate::RenderCpuScope::DeviceBacking);
        let format = ShaderTextureFormatKey::ConvolutionRgba32Float;
        let layout = BindGroupLayoutKey::new(
            &cpu,
            None,
            ShaderProgramKey::Convolution,
            &[
                (ShaderBindingRoleKey::ConvolutionSource, format),
                (ShaderBindingRoleKey::ConvolutionAuxiliary, format),
            ],
            &[
                ShaderDataBindingKey::ConvolutionParameters,
                ShaderDataBindingKey::ConvolutionWeights,
            ],
        )
        .unwrap();
        let shader = ShaderModuleKey::new(
            &cpu,
            None,
            ShaderProgramKey::Convolution,
            layout.clone(),
            &[],
            Some(format),
            Some(format),
        )
        .unwrap();
        let pipeline =
            RenderPipelineKey::new(&cpu, None, shader.clone(), layout.clone(), &[], format)
                .unwrap();
        (layout, shader, pipeline)
    }

    #[test]
    fn convolution_keys_require_exact_no_sampler_layout_and_output_identity() {
        let (layout, shader, pipeline) = fixture();
        let validate = |layout, shader, pipeline| {
            validate_convolution_pass_keys(
                ConvolutionPassKeyRefs {
                    samplers: &[],
                    layout,
                    shader,
                    pipeline,
                },
                None,
            )
        };
        assert!(validate(&layout, &shader, &pipeline).is_ok());
        let mut wrong = shader.clone();
        wrong.output_format = None;
        assert!(validate(&layout, &wrong, &pipeline).is_err());
        let mut wrong = pipeline.clone();
        wrong.target_format = ShaderTextureFormatKey::OutputBgra8Unorm;
        assert!(validate(&layout, &shader, &wrong).is_err());
        let sampler = SamplerKey::new(
            ShaderBindingRoleKey::ConvolutionSource,
            ShaderTextureFormatKey::ConvolutionRgba32Float,
            ShaderSamplingFilterKey::Nearest,
            ShaderSamplingEdgeKey::ClampToExtent,
            None,
        );
        assert!(
            validate_convolution_pass_keys(
                ConvolutionPassKeyRefs {
                    samplers: &[sampler],
                    layout: &layout,
                    shader: &shader,
                    pipeline: &pipeline,
                },
                None
            )
            .is_err()
        );
        let (mut wrong_layout, mut wrong_shader, mut wrong_pipeline) = fixture();
        wrong_layout.program = ShaderProgramKey::ColorFilter;
        wrong_shader.layout = wrong_layout.clone();
        wrong_pipeline.layout = wrong_layout.clone();
        wrong_pipeline.shader = wrong_shader.clone();
        assert!(validate(&wrong_layout, &wrong_shader, &wrong_pipeline).is_err());
    }
}

#[cfg(test)]
mod offset_key_tests {
    use super::*;
    use crate::accounting::RendererCpu;

    fn fixture(
        format: ShaderTextureFormatKey,
    ) -> (BindGroupLayoutKey, ShaderModuleKey, RenderPipelineKey) {
        let owner = RendererCpu::new(crate::RenderAccountingOptions::default()).unwrap();
        let cpu = owner.at(crate::RenderCpuScope::DeviceBacking);
        let layout = BindGroupLayoutKey::new(
            &cpu,
            None,
            ShaderProgramKey::Offset,
            &[(ShaderBindingRoleKey::OffsetSource, format)],
            &[ShaderDataBindingKey::OffsetParameters],
        )
        .unwrap();
        let shader = ShaderModuleKey::new(
            &cpu,
            None,
            ShaderProgramKey::Offset,
            layout.clone(),
            &[],
            Some(format),
            Some(format),
        )
        .unwrap();
        let pipeline =
            RenderPipelineKey::new(&cpu, None, shader.clone(), layout.clone(), &[], format)
                .unwrap();
        (layout, shader, pipeline)
    }

    #[test]
    fn offset_keys_accept_both_working_formats_and_refuse_foreign_or_mixed_formats() {
        for (format, valid) in [
            (
                ShaderTextureFormatKey::WorkingHighPrecisionRgba16Float,
                true,
            ),
            (
                ShaderTextureFormatKey::WorkingReducedPrecisionRgba8Unorm,
                true,
            ),
            (ShaderTextureFormatKey::ConvolutionRgba32Float, false),
            (ShaderTextureFormatKey::VelloCaptureRgba8Unorm, false),
            (ShaderTextureFormatKey::OutputRgba8Unorm, false),
        ] {
            let (layout, shader, pipeline) = fixture(format);
            let result = validate_offset_pass_keys(
                OffsetPassKeyRefs {
                    samplers: &[],
                    layout: &layout,
                    shader: &shader,
                    pipeline: &pipeline,
                },
                None,
            );
            assert_eq!(result.is_ok(), valid, "{format:?}");
        }
        let format = ShaderTextureFormatKey::WorkingHighPrecisionRgba16Float;
        let (layout, shader, mut pipeline) = fixture(format);
        pipeline.target_format = ShaderTextureFormatKey::WorkingReducedPrecisionRgba8Unorm;
        assert!(
            validate_offset_pass_keys(
                OffsetPassKeyRefs {
                    samplers: &[],
                    layout: &layout,
                    shader: &shader,
                    pipeline: &pipeline,
                },
                None
            )
            .is_err()
        );
    }

    #[test]
    fn offset_keys_require_exact_source_uniform_and_sampler_free_phase_identity() {
        let format = ShaderTextureFormatKey::WorkingHighPrecisionRgba16Float;
        let (layout, shader, pipeline) = fixture(format);
        fn validate(
            layout: &BindGroupLayoutKey,
            shader: &ShaderModuleKey,
            pipeline: &RenderPipelineKey,
        ) -> Result<OffsetPassDescription> {
            validate_offset_pass_keys(
                OffsetPassKeyRefs {
                    samplers: &[],
                    layout,
                    shader,
                    pipeline,
                },
                None,
            )
        }
        let mut wrong = shader.clone();
        wrong.output_format = None;
        assert!(validate(&layout, &wrong, &pipeline).is_err());
        let mut wrong = layout.clone();
        wrong.program = ShaderProgramKey::Convolution;
        assert!(validate(&wrong, &shader, &pipeline).is_err());
        let owner = RendererCpu::new(crate::RenderAccountingOptions::default()).unwrap();
        let cpu = owner.at(crate::RenderCpuScope::DeviceBacking);
        for (role, data) in [
            (
                ShaderBindingRoleKey::ConvolutionSource,
                ShaderDataBindingKey::OffsetParameters,
            ),
            (
                ShaderBindingRoleKey::OffsetSource,
                ShaderDataBindingKey::ConvolutionParameters,
            ),
        ] {
            let layout = BindGroupLayoutKey::new(
                &cpu,
                None,
                ShaderProgramKey::Offset,
                &[(role, format)],
                &[data],
            )
            .unwrap();
            let shader = ShaderModuleKey::new(
                &cpu,
                None,
                ShaderProgramKey::Offset,
                layout.clone(),
                &[],
                Some(format),
                Some(format),
            )
            .unwrap();
            let pipeline =
                RenderPipelineKey::new(&cpu, None, shader.clone(), layout.clone(), &[], format)
                    .unwrap();
            assert!(validate(&layout, &shader, &pipeline).is_err());
        }
        let sampler = SamplerKey::new(
            ShaderBindingRoleKey::OffsetSource,
            format,
            ShaderSamplingFilterKey::Nearest,
            ShaderSamplingEdgeKey::TransparentBlack,
            None,
        );
        assert!(
            validate_offset_pass_keys(
                OffsetPassKeyRefs {
                    samplers: &[sampler],
                    layout: &layout,
                    shader: &shader,
                    pipeline: &pipeline,
                },
                None
            )
            .is_err()
        );
    }
}
