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
        | ShaderProgramKey::Gaussian
        | ShaderProgramKey::GaussianSource
        | ShaderProgramKey::GaussianConstant
        | ShaderProgramKey::Morphology { .. }
        | ShaderProgramKey::Offset
        | ShaderProgramKey::NativeShadow { .. }
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
                ShaderTextureFormatKey::FilterRgba32Float
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
        || keys.shader.working_format != Some(ShaderTextureFormatKey::FilterRgba32Float)
        || keys.shader.output_format != Some(target_format)
        || source.binding_role != ShaderBindingRoleKey::ConvolutionSource
        || auxiliary.binding_role != ShaderBindingRoleKey::ConvolutionAuxiliary
        || !valid_input(source.source_format)
        || !valid_input(auxiliary.source_format)
        || !(is_working_format(target_format)
            || target_format == ShaderTextureFormatKey::FilterRgba32Float)
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
pub(super) struct GaussianPassDescription {
    pub(super) target_format: ShaderTextureFormatKey,
    pub(super) program: ShaderProgramKey,
}

#[derive(Clone, Copy)]
pub(crate) struct GaussianPassKeyRefs<'a> {
    pub(crate) samplers: &'a [SamplerKey],
    pub(crate) layout: &'a BindGroupLayoutKey,
    pub(crate) shader: &'a ShaderModuleKey,
    pub(crate) pipeline: &'a RenderPipelineKey,
}

pub(super) fn validate_gaussian_pass_keys(
    keys: GaussianPassKeyRefs<'_>,
    observer: Option<&Account>,
) -> Result<GaussianPassDescription> {
    let program = keys.layout.program;
    let roles: &[ShaderBindingRoleKey] = match program {
        ShaderProgramKey::Gaussian => &[
            ShaderBindingRoleKey::GaussianSource,
            ShaderBindingRoleKey::GaussianAuxiliary,
        ],
        ShaderProgramKey::GaussianSource => &[ShaderBindingRoleKey::GaussianSource],
        ShaderProgramKey::GaussianConstant => &[],
        _ => {
            return Err(gaussian_cache_error(
                "Gaussian keys require a Gaussian program",
            ));
        }
    };
    let data: &[ShaderDataBindingKey] = if program == ShaderProgramKey::Gaussian {
        &[
            ShaderDataBindingKey::GaussianParameters,
            ShaderDataBindingKey::GaussianKernel,
        ]
    } else {
        &[ShaderDataBindingKey::GaussianParameters]
    };
    let target_format = keys.pipeline.target_format;
    if keys.layout.sampled_textures.len() != roles.len() {
        return Err(gaussian_cache_error(
            "Gaussian texture bindings disagree with the program",
        ));
    }
    for (texture, role) in keys.layout.sampled_textures.iter().zip(roles) {
        binding_visit(observer)?;
        if (program == ShaderProgramKey::Gaussian
            && texture.source_format != ShaderTextureFormatKey::FilterRgba32Float)
            || texture.binding_role != *role
            || !(is_working_format(texture.source_format)
                || matches!(
                    texture.source_format,
                    ShaderTextureFormatKey::FilterRgba32Float
                        | ShaderTextureFormatKey::VelloCaptureRgba8Unorm
                ))
        {
            return Err(gaussian_cache_error(
                "Gaussian texture role or format is invalid",
            ));
        }
    }
    if keys.shader.program != program
        || !keys.samplers.is_empty()
        || !keys.shader.samplers.is_empty()
        || !keys.pipeline.samplers.is_empty()
        || !binding_eq(keys.layout.data_bindings.as_slice(), data, observer)?
        || !layout_eq(&keys.shader.layout, keys.layout, observer)?
        || !shader_eq(&keys.pipeline.shader, keys.shader, observer)?
        || !layout_eq(&keys.pipeline.layout, keys.layout, observer)?
        || keys.shader.working_format != Some(ShaderTextureFormatKey::FilterRgba32Float)
        || keys.shader.output_format != Some(target_format)
        || !(is_working_format(target_format)
            || target_format == ShaderTextureFormatKey::FilterRgba32Float)
        || (program == ShaderProgramKey::Gaussian
            && target_format != ShaderTextureFormatKey::FilterRgba32Float)
    {
        return Err(gaussian_cache_error(
            "Gaussian keys disagree across bindings, programs, or formats",
        ));
    }
    Ok(GaussianPassDescription {
        target_format,
        program,
    })
}

pub(super) fn gaussian_cache_error(message: &'static str) -> Error {
    Error::new(BackendErrorCode::RenderFailed, message)
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(super) struct MorphologyPassDescription {
    pub(super) target_format: ShaderTextureFormatKey,
    pub(super) stage: super::ShaderMorphologyStageKey,
}

#[derive(Clone, Copy)]
pub(crate) struct MorphologyPassKeyRefs<'a> {
    pub(crate) samplers: &'a [SamplerKey],
    pub(crate) layout: &'a BindGroupLayoutKey,
    pub(crate) shader: &'a ShaderModuleKey,
    pub(crate) pipeline: &'a RenderPipelineKey,
}

pub(super) fn validate_morphology_pass_keys(
    keys: MorphologyPassKeyRefs<'_>,
    observer: Option<&Account>,
) -> Result<MorphologyPassDescription> {
    use super::ShaderMorphologyStageKey as Stage;
    let ShaderProgramKey::Morphology { stage } = keys.layout.program else {
        return Err(morphology_cache_error(
            "morphology keys require a morphology program",
        ));
    };
    let roles: &[ShaderBindingRoleKey] = match stage {
        Stage::Constant => &[],
        Stage::RefineQuery => &[
            ShaderBindingRoleKey::MorphologySource,
            ShaderBindingRoleKey::MorphologyPreviousQuery,
        ],
        _ => &[ShaderBindingRoleKey::MorphologySource],
    };
    let target_format = keys.pipeline.target_format;
    let float_target = matches!(
        stage,
        Stage::Prepare | Stage::Table | Stage::FirstQuery | Stage::RefineQuery | Stage::Query
    );
    let float_source = matches!(
        stage,
        Stage::Table | Stage::FirstQuery | Stage::RefineQuery | Stage::Query | Stage::Resolve
    );
    if keys.layout.sampled_textures.len() != roles.len() {
        return Err(morphology_cache_error(
            "morphology texture bindings disagree with the concrete stage",
        ));
    }
    for (texture, role) in keys.layout.sampled_textures.iter().zip(roles) {
        binding_visit(observer)?;
        if texture.binding_role != *role
            || if float_source {
                texture.source_format != ShaderTextureFormatKey::FilterRgba32Float
            } else {
                !(is_working_format(texture.source_format)
                    || texture.source_format == ShaderTextureFormatKey::VelloCaptureRgba8Unorm)
            }
        {
            return Err(morphology_cache_error(
                "morphology texture role or format is invalid",
            ));
        }
    }
    if keys.shader.program != keys.layout.program
        || !keys.samplers.is_empty()
        || !keys.shader.samplers.is_empty()
        || !keys.pipeline.samplers.is_empty()
        || !binding_eq(
            keys.layout.data_bindings.as_slice(),
            &[ShaderDataBindingKey::MorphologyParameters],
            observer,
        )?
        || !layout_eq(&keys.shader.layout, keys.layout, observer)?
        || !shader_eq(&keys.pipeline.shader, keys.shader, observer)?
        || !layout_eq(&keys.pipeline.layout, keys.layout, observer)?
        || keys.shader.working_format != Some(ShaderTextureFormatKey::FilterRgba32Float)
        || keys.shader.output_format != Some(target_format)
        || if float_target {
            target_format != ShaderTextureFormatKey::FilterRgba32Float
        } else {
            !is_working_format(target_format)
        }
    {
        return Err(morphology_cache_error(
            "morphology keys disagree across bindings, programs, or formats",
        ));
    }
    Ok(MorphologyPassDescription {
        target_format,
        stage,
    })
}

pub(super) fn morphology_cache_error(message: &'static str) -> Error {
    Error::new(BackendErrorCode::RenderFailed, message)
}

#[cfg(test)]
mod morphology_key_tests {
    use super::*;
    use crate::accounting::RendererCpu;
    use crate::shader::ShaderMorphologyStageKey as Stage;

    #[test]
    fn morphology_keys_require_exact_stage_resources_and_real_intermediate_precision() {
        let owner = RendererCpu::new(crate::RenderAccountingOptions::default()).unwrap();
        let cpu = owner.at(crate::RenderCpuScope::DeviceBacking);
        for working in [
            ShaderTextureFormatKey::WorkingHighPrecisionRgba16Float,
            ShaderTextureFormatKey::WorkingReducedPrecisionRgba8Unorm,
        ] {
            for stage in [
                Stage::RebaseSource,
                Stage::Copy,
                Stage::Prepare,
                Stage::Table,
                Stage::FirstQuery,
                Stage::RefineQuery,
                Stage::Query,
                Stage::Resolve,
                Stage::Constant,
            ] {
                let program = ShaderProgramKey::Morphology { stage };
                let float_source = matches!(
                    stage,
                    Stage::Table
                        | Stage::FirstQuery
                        | Stage::RefineQuery
                        | Stage::Query
                        | Stage::Resolve
                );
                let float_target = matches!(
                    stage,
                    Stage::Prepare
                        | Stage::Table
                        | Stage::FirstQuery
                        | Stage::RefineQuery
                        | Stage::Query
                );
                let source = if float_source {
                    ShaderTextureFormatKey::FilterRgba32Float
                } else {
                    working
                };
                let target = if float_target {
                    ShaderTextureFormatKey::FilterRgba32Float
                } else {
                    working
                };
                let mut textures = Vec::new();
                if stage != Stage::Constant {
                    textures.push((ShaderBindingRoleKey::MorphologySource, source));
                }
                if stage == Stage::RefineQuery {
                    textures.push((ShaderBindingRoleKey::MorphologyPreviousQuery, source));
                }
                let make = |textures: &[(ShaderBindingRoleKey, ShaderTextureFormatKey)], target| {
                    let layout = BindGroupLayoutKey::new(
                        &cpu,
                        None,
                        program,
                        textures,
                        &[ShaderDataBindingKey::MorphologyParameters],
                    )
                    .unwrap();
                    let shader = ShaderModuleKey::new(
                        &cpu,
                        None,
                        program,
                        layout.clone(),
                        &[],
                        Some(ShaderTextureFormatKey::FilterRgba32Float),
                        Some(target),
                    )
                    .unwrap();
                    let pipeline = RenderPipelineKey::new(
                        &cpu,
                        None,
                        shader.clone(),
                        layout.clone(),
                        &[],
                        target,
                    )
                    .unwrap();
                    (layout, shader, pipeline)
                };
                let validate = |(layout, shader, pipeline): &(
                    BindGroupLayoutKey,
                    ShaderModuleKey,
                    RenderPipelineKey,
                )| {
                    validate_morphology_pass_keys(
                        MorphologyPassKeyRefs {
                            samplers: &[],
                            layout,
                            shader,
                            pipeline,
                        },
                        None,
                    )
                };
                assert!(validate(&make(&textures, target)).is_ok());
                let mut extra = textures.clone();
                extra.push((ShaderBindingRoleKey::MorphologySource, source));
                assert!(validate(&make(&extra, target)).is_err());
                if !textures.is_empty() {
                    let mut wrong = textures.clone();
                    wrong[0].0 = ShaderBindingRoleKey::GaussianSource;
                    assert!(validate(&make(&wrong, target)).is_err());
                    if float_source {
                        wrong = textures.clone();
                        wrong[0].1 = working;
                        assert!(validate(&make(&wrong, target)).is_err());
                    }
                }
                if stage == Stage::RefineQuery {
                    assert!(validate(&make(&textures[..1], target)).is_err());
                    let mut wrong = textures.clone();
                    wrong.swap(0, 1);
                    assert!(validate(&make(&wrong, target)).is_err());
                }
                let wrong_target = if float_target {
                    working
                } else {
                    ShaderTextureFormatKey::FilterRgba32Float
                };
                assert!(validate(&make(&textures, wrong_target)).is_err());
            }
        }
    }
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

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(super) struct NativeShadowPassDescription {
    pub(super) target_format: ShaderTextureFormatKey,
    pub(super) original: super::ShaderNativeShadowSourceKey,
    pub(super) shifted_alpha: bool,
}

#[derive(Clone, Copy)]
pub(crate) struct NativeShadowPassKeyRefs<'a> {
    pub(crate) samplers: &'a [SamplerKey],
    pub(crate) layout: &'a BindGroupLayoutKey,
    pub(crate) shader: &'a ShaderModuleKey,
    pub(crate) pipeline: &'a RenderPipelineKey,
}

pub(super) fn validate_native_shadow_pass_keys(
    keys: NativeShadowPassKeyRefs<'_>,
    observer: Option<&Account>,
) -> Result<NativeShadowPassDescription> {
    use super::ShaderNativeShadowSourceKey;
    let ShaderProgramKey::NativeShadow {
        original,
        shifted_alpha,
    } = keys.layout.program
    else {
        return Err(native_shadow_cache_error(
            "native shadow requires its concrete merge program",
        ));
    };
    let expected: &[ShaderBindingRoleKey] = match (original, shifted_alpha) {
        (ShaderNativeShadowSourceKey::Constant, false) => &[],
        (ShaderNativeShadowSourceKey::Constant, true) => &[ShaderBindingRoleKey::NativeShadowAlpha],
        (_, false) => &[ShaderBindingRoleKey::NativeShadowOriginal],
        (_, true) => &[
            ShaderBindingRoleKey::NativeShadowOriginal,
            ShaderBindingRoleKey::NativeShadowAlpha,
        ],
    };
    let target_format = keys.pipeline.target_format;
    if keys.layout.sampled_textures.len() != expected.len() {
        return Err(native_shadow_cache_error(
            "native shadow texture count disagrees with its reached branches",
        ));
    }
    for (texture, role) in keys.layout.sampled_textures.iter().zip(expected) {
        binding_visit(observer)?;
        if texture.binding_role != *role || texture.source_format != target_format {
            return Err(native_shadow_cache_error(
                "native shadow branch role or working format is invalid",
            ));
        }
    }
    if keys.shader.program != keys.layout.program
        || !keys.samplers.is_empty()
        || !keys.shader.samplers.is_empty()
        || !keys.pipeline.samplers.is_empty()
        || !binding_eq(
            keys.layout.data_bindings.as_slice(),
            &[ShaderDataBindingKey::NativeShadowParameters],
            observer,
        )?
        || !layout_eq(&keys.shader.layout, keys.layout, observer)?
        || !shader_eq(&keys.pipeline.shader, keys.shader, observer)?
        || !layout_eq(&keys.pipeline.layout, keys.layout, observer)?
        || keys.shader.working_format != Some(target_format)
        || keys.shader.output_format != Some(target_format)
        || !is_working_format(target_format)
    {
        return Err(native_shadow_cache_error(
            "native shadow keys disagree across programs, bindings, or working formats",
        ));
    }
    Ok(NativeShadowPassDescription {
        target_format,
        original,
        shifted_alpha,
    })
}

pub(super) fn native_shadow_cache_error(message: &'static str) -> Error {
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
        let format = ShaderTextureFormatKey::FilterRgba32Float;
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
            ShaderTextureFormatKey::FilterRgba32Float,
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
            (ShaderTextureFormatKey::FilterRgba32Float, false),
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

#[cfg(test)]
mod gaussian_key_tests {
    use super::*;
    use crate::accounting::RendererCpu;

    #[test]
    fn gaussian_programs_require_their_real_texture_and_buffer_bindings() {
        let owner = RendererCpu::new(crate::RenderAccountingOptions::default()).unwrap();
        let cpu = owner.at(crate::RenderCpuScope::DeviceBacking);
        let float = ShaderTextureFormatKey::FilterRgba32Float;
        for (program, textures, data) in [
            (
                ShaderProgramKey::Gaussian,
                vec![
                    (ShaderBindingRoleKey::GaussianSource, float),
                    (ShaderBindingRoleKey::GaussianAuxiliary, float),
                ],
                vec![
                    ShaderDataBindingKey::GaussianParameters,
                    ShaderDataBindingKey::GaussianKernel,
                ],
            ),
            (
                ShaderProgramKey::GaussianSource,
                vec![(ShaderBindingRoleKey::GaussianSource, float)],
                vec![ShaderDataBindingKey::GaussianParameters],
            ),
            (
                ShaderProgramKey::GaussianConstant,
                vec![],
                vec![ShaderDataBindingKey::GaussianParameters],
            ),
        ] {
            let layout = BindGroupLayoutKey::new(&cpu, None, program, &textures, &data).unwrap();
            let shader = ShaderModuleKey::new(
                &cpu,
                None,
                program,
                layout.clone(),
                &[],
                Some(float),
                Some(float),
            )
            .unwrap();
            let pipeline =
                RenderPipelineKey::new(&cpu, None, shader.clone(), layout.clone(), &[], float)
                    .unwrap();
            assert!(
                validate_gaussian_pass_keys(
                    GaussianPassKeyRefs {
                        samplers: &[],
                        layout: &layout,
                        shader: &shader,
                        pipeline: &pipeline
                    },
                    None
                )
                .is_ok()
            );
            let mut foreign = shader.clone();
            foreign.program = ShaderProgramKey::Convolution;
            assert!(
                validate_gaussian_pass_keys(
                    GaussianPassKeyRefs {
                        samplers: &[],
                        layout: &layout,
                        shader: &foreign,
                        pipeline: &pipeline
                    },
                    None
                )
                .is_err()
            );
            // An extra storage binding is a different real resource contract.
            let wrong_layout = BindGroupLayoutKey::new(
                &cpu,
                None,
                program,
                &textures,
                &[
                    ShaderDataBindingKey::GaussianParameters,
                    ShaderDataBindingKey::ConvolutionWeights,
                ],
            )
            .unwrap();
            assert!(
                validate_gaussian_pass_keys(
                    GaussianPassKeyRefs {
                        samplers: &[],
                        layout: &wrong_layout,
                        shader: &shader,
                        pipeline: &pipeline
                    },
                    None
                )
                .is_err()
            );
        }
    }
}

#[cfg(test)]
mod native_shadow_key_tests {
    use super::*;
    use crate::accounting::RendererCpu;
    use crate::shader::ShaderNativeShadowSourceKey;

    #[test]
    fn native_shadow_keys_require_exact_real_branch_roles_in_each_working_format() {
        let owner = RendererCpu::new(crate::RenderAccountingOptions::default()).unwrap();
        let cpu = owner.at(crate::RenderCpuScope::DeviceBacking);
        for format in [
            ShaderTextureFormatKey::WorkingHighPrecisionRgba16Float,
            ShaderTextureFormatKey::WorkingReducedPrecisionRgba8Unorm,
        ] {
            for original in [
                ShaderNativeShadowSourceKey::Finite,
                ShaderNativeShadowSourceKey::Periodic,
                ShaderNativeShadowSourceKey::Constant,
            ] {
                for shifted_alpha in [false, true] {
                    let program = ShaderProgramKey::NativeShadow {
                        original,
                        shifted_alpha,
                    };
                    let mut textures = Vec::new();
                    if original != ShaderNativeShadowSourceKey::Constant {
                        textures.push((ShaderBindingRoleKey::NativeShadowOriginal, format));
                    }
                    if shifted_alpha {
                        textures.push((ShaderBindingRoleKey::NativeShadowAlpha, format));
                    }
                    let make = |textures: &[(ShaderBindingRoleKey, ShaderTextureFormatKey)]| {
                        let layout = BindGroupLayoutKey::new(
                            &cpu,
                            None,
                            program,
                            textures,
                            &[ShaderDataBindingKey::NativeShadowParameters],
                        )
                        .unwrap();
                        let shader = ShaderModuleKey::new(
                            &cpu,
                            None,
                            program,
                            layout.clone(),
                            &[],
                            Some(format),
                            Some(format),
                        )
                        .unwrap();
                        let pipeline = RenderPipelineKey::new(
                            &cpu,
                            None,
                            shader.clone(),
                            layout.clone(),
                            &[],
                            format,
                        )
                        .unwrap();
                        (layout, shader, pipeline)
                    };
                    let validate = |(layout, shader, pipeline): &(
                        BindGroupLayoutKey,
                        ShaderModuleKey,
                        RenderPipelineKey,
                    )| {
                        validate_native_shadow_pass_keys(
                            NativeShadowPassKeyRefs {
                                samplers: &[],
                                layout,
                                shader,
                                pipeline,
                            },
                            None,
                        )
                    };
                    assert!(validate(&make(&textures)).is_ok());
                    let mut wrong = textures.clone();
                    wrong.push((ShaderBindingRoleKey::NativeShadowOriginal, format));
                    assert!(validate(&make(&wrong)).is_err());
                    if let Some((role, source_format)) = wrong.first_mut() {
                        *role = ShaderBindingRoleKey::OffsetSource;
                        *source_format = ShaderTextureFormatKey::FilterRgba32Float;
                    }
                    wrong.pop();
                    if !wrong.is_empty() {
                        assert!(validate(&make(&wrong)).is_err());
                    }
                    if textures.len() == 2 {
                        textures.swap(0, 1);
                        assert!(validate(&make(&textures)).is_err());
                    }
                }
            }
        }
    }
}
