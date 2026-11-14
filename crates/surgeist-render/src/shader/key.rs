use crate::{
    Format, Result,
    accounting::{
        Account, AccountedVec, CpuAccount, RenderResource, RenderWork, SharedValue, StorageAccount,
    },
    image::{Extend, ImageQuality, ImageRendering},
    resource::WorkingFormat,
};

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub(crate) enum ShaderCompositePathKey {
    Normal,
    DestinationSampling,
}

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub(crate) enum ShaderCompositeKey {
    SpanSourceOver,
    Layer {
        path: ShaderCompositePathKey,
        has_clip_coverage: bool,
        has_alpha_mask: bool,
    },
    DropShadow,
}

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub(crate) enum ShaderProgramKey {
    CanonicalizeCapture,
    CopyBackdrop,
    ColorFilter,
    Convolution,
    BlurHorizontal {
        source_alpha: bool,
        edge: ShaderSamplingEdgeKey,
    },
    BlurVertical {
        source_alpha: bool,
        edge: ShaderSamplingEdgeKey,
    },
    DropShadowColorize,
    Composite(ShaderCompositeKey),
    Present,
}

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub(crate) enum ShaderTextureFormatKey {
    VelloCaptureRgba8Unorm,
    ConvolutionRgba32Float,
    ClipCoverageRgba8Unorm,
    WorkingHighPrecisionRgba16Float,
    WorkingReducedPrecisionRgba8Unorm,
    ResolvedMaskRgba8Unorm,
    OutputRgba8Unorm,
    OutputBgra8Unorm,
}

impl ShaderTextureFormatKey {
    #[must_use]
    pub(crate) const fn working(format: WorkingFormat) -> Self {
        match format {
            WorkingFormat::HighPrecision => Self::WorkingHighPrecisionRgba16Float,
            WorkingFormat::ReducedPrecision => Self::WorkingReducedPrecisionRgba8Unorm,
        }
    }

    #[must_use]
    pub(crate) const fn output(format: Format) -> Self {
        match format {
            Format::Rgba8 => Self::OutputRgba8Unorm,
            Format::Bgra8 => Self::OutputBgra8Unorm,
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub(crate) enum ShaderBindingRoleKey {
    CaptureSource,
    ConvolutionSource,
    ConvolutionAuxiliary,
    CompletedParent,
    FilterSource,
    BlurredSourceAlpha,
    CompositeParent,
    CompositeSource,
    ClipCoverage,
    AlphaMask,
    Shadow,
    FinalWorkingImage,
}

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub(crate) enum ShaderSamplingFilterKey {
    Nearest,
    Linear,
}

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub(crate) enum ShaderSamplingEdgeKey {
    ClampToExtent,
    TransparentBlack,
    SemanticBorderMirror,
}

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub(crate) enum ShaderMaskQualityKey {
    Low,
    Medium,
    High,
    Pixelated,
}

impl ShaderMaskQualityKey {
    #[must_use]
    pub(crate) const fn from_image_quality(quality: ImageQuality) -> Self {
        match quality {
            ImageQuality::Low => Self::Low,
            ImageQuality::Medium => Self::Medium,
            ImageQuality::High => Self::High,
        }
    }

    pub(super) const fn parameter_code(self) -> u32 {
        match self {
            Self::Low => 0,
            Self::Medium => 1,
            Self::High => 2,
            Self::Pixelated => 3,
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub(crate) enum ShaderMaskExtendKey {
    Pad,
    Repeat,
    Reflect,
}

impl ShaderMaskExtendKey {
    #[must_use]
    pub(crate) const fn from_extend(extend: Extend) -> Self {
        match extend {
            Extend::Pad => Self::Pad,
            Extend::Repeat => Self::Repeat,
            Extend::Reflect => Self::Reflect,
        }
    }

    pub(super) const fn parameter_code(self) -> u32 {
        match self {
            Self::Pad => 0,
            Self::Repeat => 1,
            Self::Reflect => 2,
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub(crate) struct ShaderMaskSamplingKey {
    pub(super) quality: ShaderMaskQualityKey,
    pub(super) extend: ShaderMaskExtendKey,
}

impl ShaderMaskSamplingKey {
    #[must_use]
    pub(crate) const fn with_rendering(
        quality: ImageQuality,
        rendering: Option<ImageRendering>,
        extend: Extend,
    ) -> Self {
        let quality = match rendering {
            Some(ImageRendering::Pixelated) => ShaderMaskQualityKey::Pixelated,
            _ => ShaderMaskQualityKey::from_image_quality(quality),
        };
        Self {
            quality,
            extend: ShaderMaskExtendKey::from_extend(extend),
        }
    }

    #[must_use]
    pub(crate) const fn quality(self) -> ShaderMaskQualityKey {
        self.quality
    }

    #[must_use]
    pub(crate) const fn extend(self) -> ShaderMaskExtendKey {
        self.extend
    }
}

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub(crate) enum ShaderDataBindingKey {
    SpatialUniform,
    ConvolutionParameters,
    ConvolutionWeights,
    ColorFilterOperations,
    GaussianKernel,
    BlurEdgeParameters,
    DropShadowParameters,
    CompositeParameters,
    PresentParameters,
}

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub(crate) struct SamplerKey {
    pub(super) binding_role: ShaderBindingRoleKey,
    pub(super) source_format: ShaderTextureFormatKey,
    pub(super) filter: ShaderSamplingFilterKey,
    pub(super) edge: ShaderSamplingEdgeKey,
    pub(super) resolved_mask_sampling: Option<ShaderMaskSamplingKey>,
}

impl SamplerKey {
    #[must_use]
    pub(crate) const fn new(
        binding_role: ShaderBindingRoleKey,
        source_format: ShaderTextureFormatKey,
        filter: ShaderSamplingFilterKey,
        edge: ShaderSamplingEdgeKey,
        resolved_mask_sampling: Option<ShaderMaskSamplingKey>,
    ) -> Self {
        Self {
            binding_role,
            source_format,
            filter,
            edge,
            resolved_mask_sampling,
        }
    }

    #[cfg(test)]
    #[must_use]
    pub(crate) const fn facts_for_test(
        self,
    ) -> (
        ShaderBindingRoleKey,
        ShaderTextureFormatKey,
        ShaderSamplingFilterKey,
        ShaderSamplingEdgeKey,
        Option<ShaderMaskSamplingKey>,
    ) {
        (
            self.binding_role,
            self.source_format,
            self.filter,
            self.edge,
            self.resolved_mask_sampling,
        )
    }
}

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub(super) struct SampledTextureLayoutKey {
    pub(super) binding_role: ShaderBindingRoleKey,
    pub(super) source_format: ShaderTextureFormatKey,
}

/// Copies real key elements once; all later clones share the common measured
/// immutable owner instead of allocating another nested Vec.
fn copy_key_slice<T, U>(
    cpu: &CpuAccount,
    observer: Option<&Account>,
    input: &[U],
    mut copy: impl FnMut(&U) -> T,
) -> Result<SharedValue<Vec<T>>> {
    cpu.phase(|| {
        let storage = StorageAccount::Renderer(cpu.clone());
        let mut values =
            AccountedVec::new_in(&storage, input.len(), RenderResource::RetainedCpuBytes)?;
        for item in input {
            shader_work(cpu, observer, RenderWork::ShaderKeyCopies, 1)?;
            values.push(copy(item))?;
        }
        SharedValue::from_accounted(values, &storage)
    })
}

pub(super) fn shader_work(
    cpu: &CpuAccount,
    observer: Option<&Account>,
    work: RenderWork,
    amount: usize,
) -> Result<()> {
    if let Some(observer) = observer {
        observer.work(work, amount)
    } else {
        cpu.maintenance(
            work,
            u64::try_from(amount)
                .map_err(|_| crate::accounting::RenderAccountingError::Invariant)?,
        );
        Ok(())
    }
}

#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub(crate) struct BindGroupLayoutKey {
    pub(super) program: ShaderProgramKey,
    pub(super) sampled_textures: SharedValue<Vec<SampledTextureLayoutKey>>,
    pub(super) data_bindings: SharedValue<Vec<ShaderDataBindingKey>>,
}

impl BindGroupLayoutKey {
    pub(crate) fn new(
        cpu: &CpuAccount,
        observer: Option<&Account>,
        program: ShaderProgramKey,
        sampled_textures: &[(ShaderBindingRoleKey, ShaderTextureFormatKey)],
        data_bindings: &[ShaderDataBindingKey],
    ) -> Result<Self> {
        Ok(Self {
            program,
            sampled_textures: copy_key_slice(
                cpu,
                observer,
                sampled_textures,
                |&(binding_role, source_format)| SampledTextureLayoutKey {
                    binding_role,
                    source_format,
                },
            )?,
            data_bindings: copy_key_slice(cpu, observer, data_bindings, |item| *item)?,
        })
    }
}

#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub(crate) struct ShaderModuleKey {
    pub(super) program: ShaderProgramKey,
    pub(super) layout: BindGroupLayoutKey,
    pub(super) samplers: SharedValue<Vec<SamplerKey>>,
    pub(super) working_format: Option<ShaderTextureFormatKey>,
    pub(super) output_format: Option<ShaderTextureFormatKey>,
}

impl ShaderModuleKey {
    pub(crate) fn new(
        cpu: &CpuAccount,
        observer: Option<&Account>,
        program: ShaderProgramKey,
        layout: BindGroupLayoutKey,
        samplers: &[SamplerKey],
        working_format: Option<ShaderTextureFormatKey>,
        output_format: Option<ShaderTextureFormatKey>,
    ) -> Result<Self> {
        Ok(Self {
            program,
            layout,
            samplers: copy_key_slice(cpu, observer, samplers, |item| *item)?,
            working_format,
            output_format,
        })
    }
}

#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub(crate) struct RenderPipelineKey {
    pub(super) shader: ShaderModuleKey,
    pub(super) layout: BindGroupLayoutKey,
    pub(super) samplers: SharedValue<Vec<SamplerKey>>,
    pub(super) target_format: ShaderTextureFormatKey,
}

impl RenderPipelineKey {
    pub(crate) fn new(
        cpu: &CpuAccount,
        observer: Option<&Account>,
        shader: ShaderModuleKey,
        layout: BindGroupLayoutKey,
        samplers: &[SamplerKey],
        target_format: ShaderTextureFormatKey,
    ) -> Result<Self> {
        Ok(Self {
            shader,
            layout,
            samplers: copy_key_slice(cpu, observer, samplers, |item| *item)?,
            target_format,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::accounting::RendererCpu;
    use crate::{RenderAccountingOptions, RenderCpuScope, RenderLimits};
    use std::hash::{Hash, Hasher};

    fn live(cpu: &RendererCpu) -> u64 {
        cpu.snapshot()
            .unwrap()
            .usage()
            .live(RenderResource::RetainedCpuBytes)
    }

    #[test]
    fn nested_key_clones_share_one_physical_admission_until_the_last_drop() {
        let owner = RendererCpu::new(RenderAccountingOptions::default()).unwrap();
        let cpu = owner.at(RenderCpuScope::DeviceBacking);
        let baseline = live(&owner);
        let sampler = SamplerKey::new(
            ShaderBindingRoleKey::CaptureSource,
            ShaderTextureFormatKey::VelloCaptureRgba8Unorm,
            ShaderSamplingFilterKey::Nearest,
            ShaderSamplingEdgeKey::ClampToExtent,
            None,
        );
        let layout = BindGroupLayoutKey::new(
            &cpu,
            None,
            ShaderProgramKey::CanonicalizeCapture,
            &[(
                ShaderBindingRoleKey::CaptureSource,
                ShaderTextureFormatKey::VelloCaptureRgba8Unorm,
            )],
            &[ShaderDataBindingKey::SpatialUniform],
        )
        .unwrap();
        let shader = ShaderModuleKey::new(
            &cpu,
            None,
            ShaderProgramKey::CanonicalizeCapture,
            layout.clone(),
            &[sampler],
            Some(ShaderTextureFormatKey::WorkingHighPrecisionRgba16Float),
            None,
        )
        .unwrap();
        let pipeline = RenderPipelineKey::new(
            &cpu,
            None,
            shader.clone(),
            layout.clone(),
            &[sampler],
            ShaderTextureFormatKey::WorkingHighPrecisionRgba16Float,
        )
        .unwrap();
        let allocated = live(&owner);
        assert!(allocated > baseline);
        let copies = owner
            .snapshot()
            .unwrap()
            .usage()
            .work(RenderWork::ShaderKeyCopies);
        assert_eq!(copies, 4);
        let retained = pipeline.clone();
        assert_eq!(pipeline, retained);
        assert_eq!(live(&owner), allocated);
        drop((pipeline, shader, layout));
        assert_eq!(live(&owner), allocated);
        assert_eq!(
            owner
                .snapshot()
                .unwrap()
                .usage()
                .work(RenderWork::ShaderKeyCopies),
            copies
        );
        assert_eq!(retained.shader.samplers.as_slice(), [sampler]);
        assert_eq!(
            retained.layout.data_bindings.as_slice(),
            [ShaderDataBindingKey::SpatialUniform]
        );
        drop(retained);
        assert_eq!(live(&owner), baseline);
        owner.ensure_healthy().unwrap();
    }

    #[test]
    fn key_capacity_refusal_keeps_only_the_initial_cpu_owner_live() {
        let baseline = live(&RendererCpu::new(RenderAccountingOptions::default()).unwrap());
        let owner = RendererCpu::new(
            RenderAccountingOptions::default()
                .with_limits(RenderLimits::new().with_retained_bytes(baseline)),
        )
        .unwrap();
        let result = BindGroupLayoutKey::new(
            &owner.at(RenderCpuScope::DeviceBacking),
            None,
            ShaderProgramKey::CanonicalizeCapture,
            &[(
                ShaderBindingRoleKey::CaptureSource,
                ShaderTextureFormatKey::VelloCaptureRgba8Unorm,
            )],
            &[ShaderDataBindingKey::SpatialUniform],
        );
        assert!(result.is_err());
        assert_eq!(live(&owner), baseline);
        assert_eq!(
            owner
                .snapshot()
                .unwrap()
                .usage()
                .work(RenderWork::ShaderKeyCopies),
            0
        );
        owner.ensure_healthy().unwrap();
    }

    #[test]
    fn independently_created_equal_keys_preserve_content_identity_and_own_two_backings() {
        let owner = RendererCpu::new(RenderAccountingOptions::default()).unwrap();
        let cpu = owner.at(RenderCpuScope::DeviceBacking);
        let baseline = live(&owner);
        let make = || {
            BindGroupLayoutKey::new(
                &cpu,
                None,
                ShaderProgramKey::Present,
                &[(
                    ShaderBindingRoleKey::FinalWorkingImage,
                    ShaderTextureFormatKey::WorkingHighPrecisionRgba16Float,
                )],
                &[
                    ShaderDataBindingKey::SpatialUniform,
                    ShaderDataBindingKey::PresentParameters,
                ],
            )
            .unwrap()
        };
        let first = make();
        let one = live(&owner);
        let second = make();
        assert_eq!(first, second);
        assert_eq!(live(&owner) - one, one - baseline);
        let hash = |key: &BindGroupLayoutKey| {
            let mut state = std::collections::hash_map::DefaultHasher::new();
            key.hash(&mut state);
            state.finish()
        };
        assert_eq!(hash(&first), hash(&second));
        drop(first);
        assert_eq!(live(&owner), one);
        drop(second);
        assert_eq!(live(&owner), baseline);
    }
}
