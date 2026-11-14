use std::borrow::Cow;

use crate::{
    Result,
    accounting::{
        Account, AccountedVec, CpuAccount, Generation, GpuObjectKind, OwnedGpu, RenderResource,
        RenderWork, StorageAccount,
    },
};

use super::{
    LAYER_COMPOSITE_WGSL, ShaderMaskExtendKey,
    key::{
        SamplerKey, ShaderCompositePathKey, ShaderMaskSamplingKey, ShaderSamplingEdgeKey,
        ShaderSamplingFilterKey, ShaderTextureFormatKey,
    },
    validate::{
        BlurAxis, BlurInput, BlurPassDescription, ColorFilterPassDescription,
        CompositePassDescription, ConvolutionPassDescription, CopyBackdropPassDescription,
        CorePassDescription, CorePassProgram, DropShadowColorizePassDescription,
        GaussianPassDescription, OffsetPassDescription,
    },
};

const CANONICALIZE_CAPTURE_WGSL: &str = include_str!("../shaders/canonicalize_capture.wgsl");
const SPAN_SOURCE_OVER_WGSL: &str = include_str!("../shaders/span_source_over.wgsl");
const PRESENT_WGSL: &str = include_str!("../shaders/present.wgsl");
pub(super) const CONVOLUTION_WGSL: &str = concat!(
    include_str!("../shaders/native_filter_color.wgsl"),
    include_str!("../shaders/convolution.wgsl")
);
pub(super) const GAUSSIAN_WGSL: &str = concat!(
    include_str!("../shaders/native_filter_color.wgsl"),
    include_str!("../shaders/gaussian.wgsl")
);
const OFFSET_WGSL: &str = concat!(
    include_str!("../shaders/cubic.wgsl"),
    include_str!("../shaders/offset.wgsl"),
);
const COLOR_FILTER_WGSL: &str = include_str!("../shaders/color_filter.wgsl");
pub(super) const BLUR_WGSL: &str = include_str!("../shaders/blur.wgsl");
const DROP_SHADOW_WGSL: &str = include_str!("../shaders/drop_shadow.wgsl");
const COPY_BACKDROP_WGSL: &str = include_str!("../shaders/copy_backdrop.wgsl");

pub(super) fn create_sampler(
    generation: &Generation,
    _cpu: &CpuAccount,
    observer: Option<&Account>,
    device: &wgpu::Device,
    key: SamplerKey,
) -> Result<OwnedGpu<wgpu::Sampler>> {
    generation.opaque(GpuObjectKind::Sampler, observer, || {
        device.create_sampler(&sampler_descriptor(key))
    })
}

fn sampler_descriptor(key: SamplerKey) -> wgpu::SamplerDescriptor<'static> {
    let filter = match key.filter {
        ShaderSamplingFilterKey::Nearest => wgpu::FilterMode::Nearest,
        ShaderSamplingFilterKey::Linear => wgpu::FilterMode::Linear,
    };
    let address_mode = match key
        .resolved_mask_sampling
        .map(ShaderMaskSamplingKey::extend)
    {
        None | Some(ShaderMaskExtendKey::Pad) => wgpu::AddressMode::ClampToEdge,
        Some(ShaderMaskExtendKey::Repeat) => wgpu::AddressMode::Repeat,
        Some(ShaderMaskExtendKey::Reflect) => wgpu::AddressMode::MirrorRepeat,
    };
    wgpu::SamplerDescriptor {
        label: Some("Surgeist sampled-image sampler"),
        address_mode_u: address_mode,
        address_mode_v: address_mode,
        address_mode_w: address_mode,
        mag_filter: filter,
        min_filter: filter,
        mipmap_filter: wgpu::MipmapFilterMode::Nearest,
        ..Default::default()
    }
}

pub(super) fn create_core_pass_bind_group_layout(
    generation: &Generation,
    _cpu: &CpuAccount,
    observer: Option<&Account>,
    device: &wgpu::Device,
    description: CorePassDescription,
) -> Result<OwnedGpu<wgpu::BindGroupLayout>> {
    let entries = [
        wgpu::BindGroupLayoutEntry {
            binding: 0,
            visibility: wgpu::ShaderStages::FRAGMENT,
            ty: wgpu::BindingType::Texture {
                sample_type: wgpu::TextureSampleType::Float { filterable: true },
                view_dimension: wgpu::TextureViewDimension::D2,
                multisampled: false,
            },
            count: None,
        },
        wgpu::BindGroupLayoutEntry {
            binding: 1,
            visibility: wgpu::ShaderStages::FRAGMENT,
            ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
            count: None,
        },
        wgpu::BindGroupLayoutEntry {
            binding: 2,
            visibility: wgpu::ShaderStages::FRAGMENT,
            ty: wgpu::BindingType::Buffer {
                ty: wgpu::BufferBindingType::Uniform,
                has_dynamic_offset: false,
                min_binding_size: wgpu::BufferSize::new(48),
            },
            count: None,
        },
    ];
    super::key::shader_work(
        _cpu,
        observer,
        RenderWork::ShaderBindingVisits,
        entries.len(),
    )?;
    generation.opaque(GpuObjectKind::BindGroupLayout, observer, || {
        device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some(core_pass_bind_group_layout_label(description.program)),
            entries: &entries,
        })
    })
}

pub(super) fn create_core_pass_shader_module(
    generation: &Generation,
    _cpu: &CpuAccount,
    observer: Option<&Account>,
    device: &wgpu::Device,
    description: CorePassDescription,
) -> Result<OwnedGpu<wgpu::ShaderModule>> {
    generation.opaque(GpuObjectKind::ShaderModule, observer, || {
        device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some(core_pass_shader_label(description.program)),
            source: wgpu::ShaderSource::Wgsl(Cow::Borrowed(core_pass_shader_source(
                description.program,
            ))),
        })
    })
}

pub(super) fn create_core_pass_render_pipeline(
    generation: &Generation,
    observer: Option<&Account>,
    device: &wgpu::Device,
    description: CorePassDescription,
    layout: &wgpu::BindGroupLayout,
    shader: &wgpu::ShaderModule,
    fragment_entry: &'static str,
) -> Result<OwnedGpu<wgpu::RenderPipeline>> {
    let pipeline_layout = generation.opaque(GpuObjectKind::PipelineLayout, observer, || {
        device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some(core_pass_pipeline_layout_label(description.program)),
            bind_group_layouts: &[Some(layout)],
            immediate_size: 0,
        })
    })?;
    let blend = matches!(
        description.program,
        CorePassProgram::SpanSourceOver | CorePassProgram::DropShadowMerge
    )
    .then_some(span_source_over_blend());
    let target = wgpu::ColorTargetState {
        format: texture_format(description.target_format)?,
        blend,
        write_mask: wgpu::ColorWrites::ALL,
    };
    generation.opaque(GpuObjectKind::RenderPipeline, observer, || {
        device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some(core_pass_pipeline_label(description.program)),
            layout: Some(&pipeline_layout),
            vertex: wgpu::VertexState {
                module: shader,
                entry_point: Some("vertex_main"),
                buffers: &[],
                compilation_options: wgpu::PipelineCompilationOptions::default(),
            },
            primitive: wgpu::PrimitiveState {
                topology: wgpu::PrimitiveTopology::TriangleList,
                ..Default::default()
            },
            depth_stencil: None,
            multisample: wgpu::MultisampleState::default(),
            fragment: Some(wgpu::FragmentState {
                module: shader,
                entry_point: Some(fragment_entry),
                targets: &[Some(target)],
                compilation_options: wgpu::PipelineCompilationOptions::default(),
            }),
            multiview_mask: None,
            cache: None,
        })
    })
}

pub(super) fn create_composite_bind_group_layout(
    generation: &Generation,
    cpu: &CpuAccount,
    observer: Option<&Account>,
    device: &wgpu::Device,
    description: CompositePassDescription,
) -> Result<OwnedGpu<wgpu::BindGroupLayout>> {
    let texture_entry = |binding, filterable| wgpu::BindGroupLayoutEntry {
        binding,
        visibility: wgpu::ShaderStages::FRAGMENT,
        ty: wgpu::BindingType::Texture {
            sample_type: wgpu::TextureSampleType::Float { filterable },
            view_dimension: wgpu::TextureViewDimension::D2,
            multisampled: false,
        },
        count: None,
    };
    let mut entries = AccountedVec::new_in(
        &StorageAccount::Renderer(cpu.clone()),
        4 + usize::from(description.path == ShaderCompositePathKey::DestinationSampling)
            + usize::from(description.has_clip_coverage)
            + usize::from(description.has_alpha_mask),
        RenderResource::ScratchCpuBytes,
    )?;
    entries.push(texture_entry(0, true))?;
    entries.push(wgpu::BindGroupLayoutEntry {
        binding: 1,
        visibility: wgpu::ShaderStages::FRAGMENT,
        ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
        count: None,
    })?;
    if description.path == ShaderCompositePathKey::DestinationSampling {
        entries.push(texture_entry(2, false))?;
    }
    if description.has_clip_coverage {
        entries.push(texture_entry(3, false))?;
    }
    if description.has_alpha_mask {
        entries.push(texture_entry(4, false))?;
    }
    entries.push(wgpu::BindGroupLayoutEntry {
        binding: 5,
        visibility: wgpu::ShaderStages::FRAGMENT,
        ty: wgpu::BindingType::Buffer {
            ty: wgpu::BufferBindingType::Uniform,
            has_dynamic_offset: false,
            min_binding_size: wgpu::BufferSize::new(48),
        },
        count: None,
    })?;
    entries.push(wgpu::BindGroupLayoutEntry {
        binding: 6,
        visibility: wgpu::ShaderStages::FRAGMENT,
        ty: wgpu::BindingType::Buffer {
            ty: wgpu::BufferBindingType::Uniform,
            has_dynamic_offset: false,
            min_binding_size: wgpu::BufferSize::new(128),
        },
        count: None,
    })?;
    super::key::shader_work(
        cpu,
        observer,
        RenderWork::ShaderBindingVisits,
        entries.len(),
    )?;
    generation.opaque(GpuObjectKind::BindGroupLayout, observer, || {
        device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("Surgeist composition layer-composite bindings"),
            entries: &entries,
        })
    })
}

pub(super) fn create_composite_shader_module(
    generation: &Generation,
    _cpu: &CpuAccount,
    observer: Option<&Account>,
    device: &wgpu::Device,
) -> Result<OwnedGpu<wgpu::ShaderModule>> {
    generation.opaque(GpuObjectKind::ShaderModule, observer, || {
        device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("Surgeist composition layer-composite shader"),
            source: wgpu::ShaderSource::Wgsl(Cow::Borrowed(LAYER_COMPOSITE_WGSL)),
        })
    })
}

pub(super) fn create_composite_render_pipeline(
    generation: &Generation,
    observer: Option<&Account>,
    device: &wgpu::Device,
    description: CompositePassDescription,
    layout: &wgpu::BindGroupLayout,
    shader: &wgpu::ShaderModule,
    fragment_entry_override: Option<&'static str>,
) -> Result<OwnedGpu<wgpu::RenderPipeline>> {
    let pipeline_layout = generation.opaque(GpuObjectKind::PipelineLayout, observer, || {
        device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("Surgeist composition layer-composite pipeline layout"),
            bind_group_layouts: &[Some(layout)],
            immediate_size: 0,
        })
    })?;
    let target = wgpu::ColorTargetState {
        format: texture_format(description.target_format)?,
        blend: match description.path {
            ShaderCompositePathKey::Normal => Some(span_source_over_blend()),
            ShaderCompositePathKey::DestinationSampling => None,
        },
        write_mask: wgpu::ColorWrites::ALL,
    };
    let fragment_entry =
        fragment_entry_override.unwrap_or_else(|| composite_fragment_entry(description));
    generation.opaque(GpuObjectKind::RenderPipeline, observer, || {
        device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("Surgeist composition layer-composite pipeline"),
            layout: Some(&pipeline_layout),
            vertex: wgpu::VertexState {
                module: shader,
                entry_point: Some("vertex_main"),
                buffers: &[],
                compilation_options: wgpu::PipelineCompilationOptions::default(),
            },
            primitive: wgpu::PrimitiveState {
                topology: wgpu::PrimitiveTopology::TriangleList,
                ..Default::default()
            },
            depth_stencil: None,
            multisample: wgpu::MultisampleState::default(),
            fragment: Some(wgpu::FragmentState {
                module: shader,
                entry_point: Some(fragment_entry),
                targets: &[Some(target)],
                compilation_options: wgpu::PipelineCompilationOptions::default(),
            }),
            multiview_mask: None,
            cache: None,
        })
    })
}

pub(super) fn create_copy_backdrop_bind_group_layout(
    generation: &Generation,
    _cpu: &CpuAccount,
    observer: Option<&Account>,
    device: &wgpu::Device,
) -> Result<OwnedGpu<wgpu::BindGroupLayout>> {
    let entries = [
        wgpu::BindGroupLayoutEntry {
            binding: 0,
            visibility: wgpu::ShaderStages::FRAGMENT,
            ty: wgpu::BindingType::Texture {
                sample_type: wgpu::TextureSampleType::Float { filterable: false },
                view_dimension: wgpu::TextureViewDimension::D2,
                multisampled: false,
            },
            count: None,
        },
        wgpu::BindGroupLayoutEntry {
            binding: 1,
            visibility: wgpu::ShaderStages::FRAGMENT,
            ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::NonFiltering),
            count: None,
        },
        wgpu::BindGroupLayoutEntry {
            binding: 2,
            visibility: wgpu::ShaderStages::FRAGMENT,
            ty: wgpu::BindingType::Buffer {
                ty: wgpu::BufferBindingType::Uniform,
                has_dynamic_offset: false,
                min_binding_size: wgpu::BufferSize::new(48),
            },
            count: None,
        },
    ];
    super::key::shader_work(
        _cpu,
        observer,
        RenderWork::ShaderBindingVisits,
        entries.len(),
    )?;
    generation.opaque(GpuObjectKind::BindGroupLayout, observer, || {
        device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("Surgeist backdrop-copy bindings"),
            entries: &entries,
        })
    })
}

pub(super) fn create_copy_backdrop_shader_module(
    generation: &Generation,
    _cpu: &CpuAccount,
    observer: Option<&Account>,
    device: &wgpu::Device,
) -> Result<OwnedGpu<wgpu::ShaderModule>> {
    generation.opaque(GpuObjectKind::ShaderModule, observer, || {
        device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("Surgeist backdrop-copy shader"),
            source: wgpu::ShaderSource::Wgsl(Cow::Borrowed(COPY_BACKDROP_WGSL)),
        })
    })
}

pub(super) fn create_copy_backdrop_pipeline(
    generation: &Generation,
    _cpu: &CpuAccount,
    observer: Option<&Account>,
    device: &wgpu::Device,
    description: CopyBackdropPassDescription,
    layout: &wgpu::BindGroupLayout,
    shader: &wgpu::ShaderModule,
) -> Result<OwnedGpu<wgpu::RenderPipeline>> {
    let pipeline_layout = generation.opaque(GpuObjectKind::PipelineLayout, observer, || {
        device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("Surgeist backdrop-copy pipeline layout"),
            bind_group_layouts: &[Some(layout)],
            immediate_size: 0,
        })
    })?;
    let target = wgpu::ColorTargetState {
        format: texture_format(description.working_format)?,
        blend: None,
        write_mask: wgpu::ColorWrites::ALL,
    };
    generation.opaque(GpuObjectKind::RenderPipeline, observer, || {
        device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("Surgeist backdrop-copy pipeline"),
            layout: Some(&pipeline_layout),
            vertex: wgpu::VertexState {
                module: shader,
                entry_point: Some("vertex_main"),
                buffers: &[],
                compilation_options: wgpu::PipelineCompilationOptions::default(),
            },
            primitive: wgpu::PrimitiveState {
                topology: wgpu::PrimitiveTopology::TriangleList,
                ..Default::default()
            },
            depth_stencil: None,
            multisample: wgpu::MultisampleState::default(),
            fragment: Some(wgpu::FragmentState {
                module: shader,
                entry_point: Some("fragment_main"),
                targets: &[Some(target)],
                compilation_options: wgpu::PipelineCompilationOptions::default(),
            }),
            multiview_mask: None,
            cache: None,
        })
    })
}

pub(super) fn create_convolution_bind_group_layout(
    generation: &Generation,
    _cpu: &CpuAccount,
    observer: Option<&Account>,
    device: &wgpu::Device,
) -> Result<OwnedGpu<wgpu::BindGroupLayout>> {
    let texture = |binding| wgpu::BindGroupLayoutEntry {
        binding,
        visibility: wgpu::ShaderStages::FRAGMENT,
        ty: wgpu::BindingType::Texture {
            sample_type: wgpu::TextureSampleType::Float { filterable: false },
            view_dimension: wgpu::TextureViewDimension::D2,
            multisampled: false,
        },
        count: None,
    };
    let entries = [
        texture(0),
        texture(1),
        wgpu::BindGroupLayoutEntry {
            binding: 2,
            visibility: wgpu::ShaderStages::FRAGMENT,
            ty: wgpu::BindingType::Buffer {
                ty: wgpu::BufferBindingType::Uniform,
                has_dynamic_offset: false,
                min_binding_size: wgpu::BufferSize::new(256),
            },
            count: None,
        },
        wgpu::BindGroupLayoutEntry {
            binding: 3,
            visibility: wgpu::ShaderStages::FRAGMENT,
            ty: wgpu::BindingType::Buffer {
                ty: wgpu::BufferBindingType::Storage { read_only: true },
                has_dynamic_offset: false,
                min_binding_size: wgpu::BufferSize::new(4),
            },
            count: None,
        },
    ];
    super::key::shader_work(
        _cpu,
        observer,
        RenderWork::ShaderBindingVisits,
        entries.len(),
    )?;
    generation.opaque(GpuObjectKind::BindGroupLayout, observer, || {
        device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("Surgeist convolution bindings"),
            entries: &entries,
        })
    })
}

pub(super) fn create_convolution_shader_module(
    generation: &Generation,
    _cpu: &CpuAccount,
    observer: Option<&Account>,
    device: &wgpu::Device,
) -> Result<OwnedGpu<wgpu::ShaderModule>> {
    generation.opaque(GpuObjectKind::ShaderModule, observer, || {
        device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("Surgeist convolution shader"),
            source: wgpu::ShaderSource::Wgsl(Cow::Borrowed(CONVOLUTION_WGSL)),
        })
    })
}

pub(super) fn create_convolution_render_pipeline(
    generation: &Generation,
    _cpu: &CpuAccount,
    observer: Option<&Account>,
    device: &wgpu::Device,
    description: ConvolutionPassDescription,
    layout: &wgpu::BindGroupLayout,
    shader: &wgpu::ShaderModule,
) -> Result<OwnedGpu<wgpu::RenderPipeline>> {
    let pipeline_layout = generation.opaque(GpuObjectKind::PipelineLayout, observer, || {
        device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("Surgeist convolution pipeline layout"),
            bind_group_layouts: &[Some(layout)],
            immediate_size: 0,
        })
    })?;
    let target = wgpu::ColorTargetState {
        format: texture_format(description.target_format)?,
        blend: None,
        write_mask: wgpu::ColorWrites::ALL,
    };
    generation.opaque(GpuObjectKind::RenderPipeline, observer, || {
        device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("Surgeist convolution pipeline"),
            layout: Some(&pipeline_layout),
            vertex: wgpu::VertexState {
                module: shader,
                entry_point: Some("vertex_main"),
                buffers: &[],
                compilation_options: wgpu::PipelineCompilationOptions::default(),
            },
            primitive: wgpu::PrimitiveState {
                topology: wgpu::PrimitiveTopology::TriangleList,
                ..Default::default()
            },
            depth_stencil: None,
            multisample: wgpu::MultisampleState::default(),
            fragment: Some(wgpu::FragmentState {
                module: shader,
                entry_point: Some("fragment_main"),
                targets: &[Some(target)],
                compilation_options: wgpu::PipelineCompilationOptions::default(),
            }),
            multiview_mask: None,
            cache: None,
        })
    })
}

pub(super) fn create_gaussian_bind_group_layout(
    generation: &Generation,
    _cpu: &CpuAccount,
    observer: Option<&Account>,
    device: &wgpu::Device,
    description: GaussianPassDescription,
) -> Result<OwnedGpu<wgpu::BindGroupLayout>> {
    let texture = |binding| wgpu::BindGroupLayoutEntry {
        binding,
        visibility: wgpu::ShaderStages::FRAGMENT,
        ty: wgpu::BindingType::Texture {
            sample_type: wgpu::TextureSampleType::Float { filterable: false },
            view_dimension: wgpu::TextureViewDimension::D2,
            multisampled: false,
        },
        count: None,
    };
    let all_entries = [
        texture(0),
        texture(1),
        wgpu::BindGroupLayoutEntry {
            binding: 2,
            visibility: wgpu::ShaderStages::FRAGMENT,
            ty: wgpu::BindingType::Buffer {
                ty: wgpu::BufferBindingType::Uniform,
                has_dynamic_offset: false,
                min_binding_size: wgpu::BufferSize::new(256),
            },
            count: None,
        },
        wgpu::BindGroupLayoutEntry {
            binding: 3,
            visibility: wgpu::ShaderStages::FRAGMENT,
            ty: wgpu::BindingType::Buffer {
                ty: wgpu::BufferBindingType::Storage { read_only: true },
                has_dynamic_offset: false,
                min_binding_size: wgpu::BufferSize::new(8),
            },
            count: None,
        },
    ];
    let entries: &[wgpu::BindGroupLayoutEntry] = match description.program {
        super::ShaderProgramKey::GaussianConstant => &all_entries[2..3],
        super::ShaderProgramKey::GaussianSource => &[all_entries[0], all_entries[2]],
        _ => &all_entries,
    };
    super::key::shader_work(
        _cpu,
        observer,
        RenderWork::ShaderBindingVisits,
        entries.len(),
    )?;
    generation.opaque(GpuObjectKind::BindGroupLayout, observer, || {
        device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("Surgeist gaussian bindings"),
            entries,
        })
    })
}

pub(super) fn create_gaussian_shader_module(
    generation: &Generation,
    _cpu: &CpuAccount,
    observer: Option<&Account>,
    device: &wgpu::Device,
) -> Result<OwnedGpu<wgpu::ShaderModule>> {
    generation.opaque(GpuObjectKind::ShaderModule, observer, || {
        device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("Surgeist gaussian shader"),
            source: wgpu::ShaderSource::Wgsl(Cow::Borrowed(GAUSSIAN_WGSL)),
        })
    })
}

pub(super) fn create_gaussian_render_pipeline(
    generation: &Generation,
    _cpu: &CpuAccount,
    observer: Option<&Account>,
    device: &wgpu::Device,
    description: GaussianPassDescription,
    layout: &wgpu::BindGroupLayout,
    shader: &wgpu::ShaderModule,
) -> Result<OwnedGpu<wgpu::RenderPipeline>> {
    let pipeline_layout = generation.opaque(GpuObjectKind::PipelineLayout, observer, || {
        device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("Surgeist gaussian pipeline layout"),
            bind_group_layouts: &[Some(layout)],
            immediate_size: 0,
        })
    })?;
    let target = wgpu::ColorTargetState {
        format: texture_format(description.target_format)?,
        blend: None,
        write_mask: wgpu::ColorWrites::ALL,
    };
    generation.opaque(GpuObjectKind::RenderPipeline, observer, || {
        device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("Surgeist gaussian pipeline"),
            layout: Some(&pipeline_layout),
            vertex: wgpu::VertexState {
                module: shader,
                entry_point: Some("vertex_main"),
                buffers: &[],
                compilation_options: wgpu::PipelineCompilationOptions::default(),
            },
            primitive: wgpu::PrimitiveState {
                topology: wgpu::PrimitiveTopology::TriangleList,
                ..Default::default()
            },
            depth_stencil: None,
            multisample: wgpu::MultisampleState::default(),
            fragment: Some(wgpu::FragmentState {
                module: shader,
                entry_point: Some(match description.program {
                    super::ShaderProgramKey::GaussianConstant => "constant_main",
                    super::ShaderProgramKey::GaussianSource => "source_main",
                    _ => "fragment_main",
                }),
                targets: &[Some(target)],
                compilation_options: wgpu::PipelineCompilationOptions::default(),
            }),
            multiview_mask: None,
            cache: None,
        })
    })
}

pub(super) fn create_offset_bind_group_layout(
    generation: &Generation,
    _cpu: &CpuAccount,
    observer: Option<&Account>,
    device: &wgpu::Device,
) -> Result<OwnedGpu<wgpu::BindGroupLayout>> {
    let texture = |binding| wgpu::BindGroupLayoutEntry {
        binding,
        visibility: wgpu::ShaderStages::FRAGMENT,
        ty: wgpu::BindingType::Texture {
            sample_type: wgpu::TextureSampleType::Float { filterable: false },
            view_dimension: wgpu::TextureViewDimension::D2,
            multisampled: false,
        },
        count: None,
    };
    let entries = [
        texture(0),
        wgpu::BindGroupLayoutEntry {
            binding: 1,
            visibility: wgpu::ShaderStages::FRAGMENT,
            ty: wgpu::BindingType::Buffer {
                ty: wgpu::BufferBindingType::Uniform,
                has_dynamic_offset: false,
                min_binding_size: wgpu::BufferSize::new(256),
            },
            count: None,
        },
    ];
    super::key::shader_work(
        _cpu,
        observer,
        RenderWork::ShaderBindingVisits,
        entries.len(),
    )?;
    generation.opaque(GpuObjectKind::BindGroupLayout, observer, || {
        device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("Surgeist offset bindings"),
            entries: &entries,
        })
    })
}

pub(super) fn create_offset_shader_module(
    generation: &Generation,
    _cpu: &CpuAccount,
    observer: Option<&Account>,
    device: &wgpu::Device,
) -> Result<OwnedGpu<wgpu::ShaderModule>> {
    generation.opaque(GpuObjectKind::ShaderModule, observer, || {
        device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("Surgeist offset shader"),
            source: wgpu::ShaderSource::Wgsl(Cow::Borrowed(OFFSET_WGSL)),
        })
    })
}

pub(super) fn create_offset_render_pipeline(
    generation: &Generation,
    _cpu: &CpuAccount,
    observer: Option<&Account>,
    device: &wgpu::Device,
    description: OffsetPassDescription,
    layout: &wgpu::BindGroupLayout,
    shader: &wgpu::ShaderModule,
) -> Result<OwnedGpu<wgpu::RenderPipeline>> {
    let pipeline_layout = generation.opaque(GpuObjectKind::PipelineLayout, observer, || {
        device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("Surgeist offset pipeline layout"),
            bind_group_layouts: &[Some(layout)],
            immediate_size: 0,
        })
    })?;
    let target = wgpu::ColorTargetState {
        format: texture_format(description.target_format)?,
        blend: None,
        write_mask: wgpu::ColorWrites::ALL,
    };
    generation.opaque(GpuObjectKind::RenderPipeline, observer, || {
        device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("Surgeist offset pipeline"),
            layout: Some(&pipeline_layout),
            vertex: wgpu::VertexState {
                module: shader,
                entry_point: Some("vertex_main"),
                buffers: &[],
                compilation_options: wgpu::PipelineCompilationOptions::default(),
            },
            primitive: wgpu::PrimitiveState {
                topology: wgpu::PrimitiveTopology::TriangleList,
                ..Default::default()
            },
            depth_stencil: None,
            multisample: wgpu::MultisampleState::default(),
            fragment: Some(wgpu::FragmentState {
                module: shader,
                entry_point: Some("fragment_main"),
                targets: &[Some(target)],
                compilation_options: wgpu::PipelineCompilationOptions::default(),
            }),
            multiview_mask: None,
            cache: None,
        })
    })
}

pub(super) fn create_color_filter_bind_group_layout(
    generation: &Generation,
    _cpu: &CpuAccount,
    observer: Option<&Account>,
    device: &wgpu::Device,
) -> Result<OwnedGpu<wgpu::BindGroupLayout>> {
    let entries = [
        wgpu::BindGroupLayoutEntry {
            binding: 0,
            visibility: wgpu::ShaderStages::FRAGMENT,
            ty: wgpu::BindingType::Texture {
                sample_type: wgpu::TextureSampleType::Float { filterable: false },
                view_dimension: wgpu::TextureViewDimension::D2,
                multisampled: false,
            },
            count: None,
        },
        wgpu::BindGroupLayoutEntry {
            binding: 1,
            visibility: wgpu::ShaderStages::FRAGMENT,
            ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::NonFiltering),
            count: None,
        },
        wgpu::BindGroupLayoutEntry {
            binding: 2,
            visibility: wgpu::ShaderStages::FRAGMENT,
            ty: wgpu::BindingType::Buffer {
                ty: wgpu::BufferBindingType::Uniform,
                has_dynamic_offset: false,
                min_binding_size: wgpu::BufferSize::new(48),
            },
            count: None,
        },
        wgpu::BindGroupLayoutEntry {
            binding: 3,
            visibility: wgpu::ShaderStages::FRAGMENT,
            ty: wgpu::BindingType::Buffer {
                ty: wgpu::BufferBindingType::Storage { read_only: true },
                has_dynamic_offset: false,
                min_binding_size: wgpu::BufferSize::new(48),
            },
            count: None,
        },
    ];
    super::key::shader_work(
        _cpu,
        observer,
        RenderWork::ShaderBindingVisits,
        entries.len(),
    )?;
    generation.opaque(GpuObjectKind::BindGroupLayout, observer, || {
        device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("Surgeist color-filter bindings"),
            entries: &entries,
        })
    })
}

pub(super) fn create_color_filter_shader_module(
    generation: &Generation,
    _cpu: &CpuAccount,
    observer: Option<&Account>,
    device: &wgpu::Device,
) -> Result<OwnedGpu<wgpu::ShaderModule>> {
    generation.opaque(GpuObjectKind::ShaderModule, observer, || {
        device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("Surgeist color-filter shader"),
            source: wgpu::ShaderSource::Wgsl(Cow::Borrowed(COLOR_FILTER_WGSL)),
        })
    })
}

pub(super) fn create_color_filter_render_pipeline(
    generation: &Generation,
    _cpu: &CpuAccount,
    observer: Option<&Account>,
    device: &wgpu::Device,
    description: ColorFilterPassDescription,
    layout: &wgpu::BindGroupLayout,
    shader: &wgpu::ShaderModule,
) -> Result<OwnedGpu<wgpu::RenderPipeline>> {
    let pipeline_layout = generation.opaque(GpuObjectKind::PipelineLayout, observer, || {
        device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("Surgeist color-filter pipeline layout"),
            bind_group_layouts: &[Some(layout)],
            immediate_size: 0,
        })
    })?;
    let target = wgpu::ColorTargetState {
        format: texture_format(description.working_format)?,
        blend: None,
        write_mask: wgpu::ColorWrites::ALL,
    };
    generation.opaque(GpuObjectKind::RenderPipeline, observer, || {
        device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("Surgeist color-filter pipeline"),
            layout: Some(&pipeline_layout),
            vertex: wgpu::VertexState {
                module: shader,
                entry_point: Some("vertex_main"),
                buffers: &[],
                compilation_options: wgpu::PipelineCompilationOptions::default(),
            },
            primitive: wgpu::PrimitiveState {
                topology: wgpu::PrimitiveTopology::TriangleList,
                ..Default::default()
            },
            depth_stencil: None,
            multisample: wgpu::MultisampleState::default(),
            fragment: Some(wgpu::FragmentState {
                module: shader,
                entry_point: Some("fragment_main"),
                targets: &[Some(target)],
                compilation_options: wgpu::PipelineCompilationOptions::default(),
            }),
            multiview_mask: None,
            cache: None,
        })
    })
}

pub(super) fn create_blur_bind_group_layout(
    generation: &Generation,
    _cpu: &CpuAccount,
    observer: Option<&Account>,
    device: &wgpu::Device,
    description: BlurPassDescription,
) -> Result<OwnedGpu<wgpu::BindGroupLayout>> {
    let initial_entries = [
        wgpu::BindGroupLayoutEntry {
            binding: 0,
            visibility: wgpu::ShaderStages::FRAGMENT,
            ty: wgpu::BindingType::Texture {
                sample_type: wgpu::TextureSampleType::Float { filterable: true },
                view_dimension: wgpu::TextureViewDimension::D2,
                multisampled: false,
            },
            count: None,
        },
        wgpu::BindGroupLayoutEntry {
            binding: 1,
            visibility: wgpu::ShaderStages::FRAGMENT,
            ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
            count: None,
        },
        wgpu::BindGroupLayoutEntry {
            binding: 2,
            visibility: wgpu::ShaderStages::FRAGMENT,
            ty: wgpu::BindingType::Buffer {
                ty: wgpu::BufferBindingType::Uniform,
                has_dynamic_offset: false,
                min_binding_size: wgpu::BufferSize::new(48),
            },
            count: None,
        },
        wgpu::BindGroupLayoutEntry {
            binding: 3,
            visibility: wgpu::ShaderStages::FRAGMENT,
            ty: wgpu::BindingType::Buffer {
                ty: wgpu::BufferBindingType::Storage { read_only: true },
                has_dynamic_offset: false,
                min_binding_size: wgpu::BufferSize::new(8),
            },
            count: None,
        },
    ];
    let mut entries = AccountedVec::new_in(
        &StorageAccount::Renderer(_cpu.clone()),
        4 + usize::from(description.edge == ShaderSamplingEdgeKey::SemanticBorderMirror),
        RenderResource::ScratchCpuBytes,
    )?;
    entries.extend_from_slice(&initial_entries)?;
    if description.edge == ShaderSamplingEdgeKey::SemanticBorderMirror {
        entries.push(wgpu::BindGroupLayoutEntry {
            binding: 4,
            visibility: wgpu::ShaderStages::FRAGMENT,
            ty: wgpu::BindingType::Buffer {
                ty: wgpu::BufferBindingType::Uniform,
                has_dynamic_offset: false,
                min_binding_size: wgpu::BufferSize::new(16),
            },
            count: None,
        })?;
    }
    super::key::shader_work(
        _cpu,
        observer,
        RenderWork::ShaderBindingVisits,
        entries.len(),
    )?;
    generation.opaque(GpuObjectKind::BindGroupLayout, observer, || {
        device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("Surgeist checked Gaussian blur bindings"),
            entries: &entries,
        })
    })
}

pub(super) fn create_blur_shader_module(
    generation: &Generation,
    _cpu: &CpuAccount,
    observer: Option<&Account>,
    device: &wgpu::Device,
) -> Result<OwnedGpu<wgpu::ShaderModule>> {
    generation.opaque(GpuObjectKind::ShaderModule, observer, || {
        device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("Surgeist checked Gaussian blur shader"),
            source: wgpu::ShaderSource::Wgsl(Cow::Borrowed(BLUR_WGSL)),
        })
    })
}

pub(super) fn create_blur_render_pipeline(
    generation: &Generation,
    _cpu: &CpuAccount,
    observer: Option<&Account>,
    device: &wgpu::Device,
    description: BlurPassDescription,
    layout: &wgpu::BindGroupLayout,
    shader: &wgpu::ShaderModule,
) -> Result<OwnedGpu<wgpu::RenderPipeline>> {
    let pipeline_layout = generation.opaque(GpuObjectKind::PipelineLayout, observer, || {
        device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("Surgeist checked Gaussian blur pipeline layout"),
            bind_group_layouts: &[Some(layout)],
            immediate_size: 0,
        })
    })?;
    let target = wgpu::ColorTargetState {
        format: texture_format(description.working_format)?,
        blend: None,
        write_mask: wgpu::ColorWrites::ALL,
    };
    generation.opaque(GpuObjectKind::RenderPipeline, observer, || {
        device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("Surgeist checked Gaussian blur pipeline"),
            layout: Some(&pipeline_layout),
            vertex: wgpu::VertexState {
                module: shader,
                entry_point: Some("vertex_main"),
                buffers: &[],
                compilation_options: wgpu::PipelineCompilationOptions::default(),
            },
            primitive: wgpu::PrimitiveState {
                topology: wgpu::PrimitiveTopology::TriangleList,
                ..Default::default()
            },
            depth_stencil: None,
            multisample: wgpu::MultisampleState::default(),
            fragment: Some(wgpu::FragmentState {
                module: shader,
                entry_point: Some(blur_fragment_entry(description)),
                targets: &[Some(target)],
                compilation_options: wgpu::PipelineCompilationOptions::default(),
            }),
            multiview_mask: None,
            cache: None,
        })
    })
}

pub(super) fn create_drop_shadow_colorize_bind_group_layout(
    generation: &Generation,
    _cpu: &CpuAccount,
    observer: Option<&Account>,
    device: &wgpu::Device,
) -> Result<OwnedGpu<wgpu::BindGroupLayout>> {
    let entries = [
        wgpu::BindGroupLayoutEntry {
            binding: 0,
            visibility: wgpu::ShaderStages::FRAGMENT,
            ty: wgpu::BindingType::Texture {
                sample_type: wgpu::TextureSampleType::Float { filterable: true },
                view_dimension: wgpu::TextureViewDimension::D2,
                multisampled: false,
            },
            count: None,
        },
        wgpu::BindGroupLayoutEntry {
            binding: 1,
            visibility: wgpu::ShaderStages::FRAGMENT,
            ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
            count: None,
        },
        wgpu::BindGroupLayoutEntry {
            binding: 2,
            visibility: wgpu::ShaderStages::FRAGMENT,
            ty: wgpu::BindingType::Buffer {
                ty: wgpu::BufferBindingType::Uniform,
                has_dynamic_offset: false,
                min_binding_size: wgpu::BufferSize::new(48),
            },
            count: None,
        },
        wgpu::BindGroupLayoutEntry {
            binding: 3,
            visibility: wgpu::ShaderStages::FRAGMENT,
            ty: wgpu::BindingType::Buffer {
                ty: wgpu::BufferBindingType::Uniform,
                has_dynamic_offset: false,
                min_binding_size: wgpu::BufferSize::new(32),
            },
            count: None,
        },
    ];
    super::key::shader_work(
        _cpu,
        observer,
        RenderWork::ShaderBindingVisits,
        entries.len(),
    )?;
    generation.opaque(GpuObjectKind::BindGroupLayout, observer, || {
        device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("Surgeist spatial-filter drop-shadow colorize bindings"),
            entries: &entries,
        })
    })
}

pub(super) fn create_drop_shadow_colorize_shader_module(
    generation: &Generation,
    _cpu: &CpuAccount,
    observer: Option<&Account>,
    device: &wgpu::Device,
) -> Result<OwnedGpu<wgpu::ShaderModule>> {
    generation.opaque(GpuObjectKind::ShaderModule, observer, || {
        device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("Surgeist spatial-filter drop-shadow colorize shader"),
            source: wgpu::ShaderSource::Wgsl(Cow::Borrowed(DROP_SHADOW_WGSL)),
        })
    })
}

pub(super) fn create_drop_shadow_colorize_render_pipeline(
    generation: &Generation,
    _cpu: &CpuAccount,
    observer: Option<&Account>,
    device: &wgpu::Device,
    description: DropShadowColorizePassDescription,
    layout: &wgpu::BindGroupLayout,
    shader: &wgpu::ShaderModule,
) -> Result<OwnedGpu<wgpu::RenderPipeline>> {
    let pipeline_layout = generation.opaque(GpuObjectKind::PipelineLayout, observer, || {
        device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("Surgeist spatial-filter drop-shadow colorize pipeline layout"),
            bind_group_layouts: &[Some(layout)],
            immediate_size: 0,
        })
    })?;
    let target = wgpu::ColorTargetState {
        format: texture_format(description.working_format)?,
        blend: None,
        write_mask: wgpu::ColorWrites::ALL,
    };
    generation.opaque(GpuObjectKind::RenderPipeline, observer, || {
        device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("Surgeist spatial-filter drop-shadow colorize pipeline"),
            layout: Some(&pipeline_layout),
            vertex: wgpu::VertexState {
                module: shader,
                entry_point: Some("vertex_main"),
                buffers: &[],
                compilation_options: wgpu::PipelineCompilationOptions::default(),
            },
            primitive: wgpu::PrimitiveState {
                topology: wgpu::PrimitiveTopology::TriangleList,
                ..Default::default()
            },
            depth_stencil: None,
            multisample: wgpu::MultisampleState::default(),
            fragment: Some(wgpu::FragmentState {
                module: shader,
                entry_point: Some("fragment_main"),
                targets: &[Some(target)],
                compilation_options: wgpu::PipelineCompilationOptions::default(),
            }),
            multiview_mask: None,
            cache: None,
        })
    })
}

const fn blur_fragment_entry(description: BlurPassDescription) -> &'static str {
    match (description.axis, description.input, description.edge) {
        (BlurAxis::Horizontal, BlurInput::Rgba, ShaderSamplingEdgeKey::TransparentBlack) => {
            "fragment_horizontal_rgba"
        }
        (BlurAxis::Vertical, BlurInput::Rgba, ShaderSamplingEdgeKey::TransparentBlack) => {
            "fragment_vertical_rgba"
        }
        (BlurAxis::Horizontal, BlurInput::SourceAlpha, ShaderSamplingEdgeKey::TransparentBlack) => {
            "fragment_horizontal_source_alpha"
        }
        (BlurAxis::Vertical, BlurInput::SourceAlpha, ShaderSamplingEdgeKey::TransparentBlack) => {
            "fragment_vertical_source_alpha"
        }
        (BlurAxis::Horizontal, BlurInput::Rgba, ShaderSamplingEdgeKey::SemanticBorderMirror) => {
            "fragment_horizontal_rgba_mirror"
        }
        (BlurAxis::Vertical, BlurInput::Rgba, ShaderSamplingEdgeKey::SemanticBorderMirror) => {
            "fragment_vertical_rgba_mirror"
        }
        (
            BlurAxis::Horizontal,
            BlurInput::SourceAlpha,
            ShaderSamplingEdgeKey::SemanticBorderMirror,
        ) => "fragment_horizontal_source_alpha_mirror",
        (
            BlurAxis::Vertical,
            BlurInput::SourceAlpha,
            ShaderSamplingEdgeKey::SemanticBorderMirror,
        ) => "fragment_vertical_source_alpha_mirror",
        (_, _, ShaderSamplingEdgeKey::ClampToExtent) => "invalid_blur_edge_policy",
    }
}

const fn composite_fragment_entry(description: CompositePassDescription) -> &'static str {
    match (
        description.path,
        description.has_clip_coverage,
        description.has_alpha_mask,
    ) {
        (ShaderCompositePathKey::Normal, false, false) => "fragment_normal",
        (ShaderCompositePathKey::Normal, true, false) => "fragment_normal_clip",
        (ShaderCompositePathKey::Normal, false, true) => "fragment_normal_mask",
        (ShaderCompositePathKey::Normal, true, true) => "fragment_normal_clip_mask",
        (ShaderCompositePathKey::DestinationSampling, false, false) => "fragment_destination",
        (ShaderCompositePathKey::DestinationSampling, true, false) => "fragment_destination_clip",
        (ShaderCompositePathKey::DestinationSampling, false, true) => "fragment_destination_mask",
        (ShaderCompositePathKey::DestinationSampling, true, true) => {
            "fragment_destination_clip_mask"
        }
    }
}

const fn core_pass_shader_source(program: CorePassProgram) -> &'static str {
    match program {
        CorePassProgram::CanonicalizeCapture => CANONICALIZE_CAPTURE_WGSL,
        CorePassProgram::SpanSourceOver | CorePassProgram::DropShadowMerge => SPAN_SOURCE_OVER_WGSL,
        CorePassProgram::Present => PRESENT_WGSL,
    }
}

const fn core_pass_shader_label(program: CorePassProgram) -> &'static str {
    match program {
        CorePassProgram::CanonicalizeCapture => "Surgeist core-pass canonicalize-capture shader",
        CorePassProgram::SpanSourceOver => "Surgeist core-pass span source-over shader",
        CorePassProgram::DropShadowMerge => "Surgeist spatial-filter drop-shadow merge shader",
        CorePassProgram::Present => "Surgeist core-pass present shader",
    }
}

const fn core_pass_bind_group_layout_label(program: CorePassProgram) -> &'static str {
    match program {
        CorePassProgram::CanonicalizeCapture => "Surgeist core-pass canonicalize-capture bindings",
        CorePassProgram::SpanSourceOver => "Surgeist core-pass span source-over bindings",
        CorePassProgram::DropShadowMerge => "Surgeist spatial-filter drop-shadow merge bindings",
        CorePassProgram::Present => "Surgeist core-pass present bindings",
    }
}

const fn core_pass_pipeline_layout_label(program: CorePassProgram) -> &'static str {
    match program {
        CorePassProgram::CanonicalizeCapture => {
            "Surgeist core-pass canonicalize-capture pipeline layout"
        }
        CorePassProgram::SpanSourceOver => "Surgeist core-pass span source-over pipeline layout",
        CorePassProgram::DropShadowMerge => {
            "Surgeist spatial-filter drop-shadow merge pipeline layout"
        }
        CorePassProgram::Present => "Surgeist core-pass present pipeline layout",
    }
}

const fn core_pass_pipeline_label(program: CorePassProgram) -> &'static str {
    match program {
        CorePassProgram::CanonicalizeCapture => "Surgeist core-pass canonicalize-capture pipeline",
        CorePassProgram::SpanSourceOver => "Surgeist core-pass span source-over pipeline",
        CorePassProgram::DropShadowMerge => "Surgeist spatial-filter drop-shadow merge pipeline",
        CorePassProgram::Present => "Surgeist core-pass present pipeline",
    }
}

pub(super) const fn span_source_over_blend() -> wgpu::BlendState {
    let component = wgpu::BlendComponent {
        src_factor: wgpu::BlendFactor::One,
        dst_factor: wgpu::BlendFactor::OneMinusSrcAlpha,
        operation: wgpu::BlendOperation::Add,
    };
    wgpu::BlendState {
        color: component,
        alpha: component,
    }
}

const fn texture_format(format: ShaderTextureFormatKey) -> Result<wgpu::TextureFormat> {
    match format {
        ShaderTextureFormatKey::VelloCaptureRgba8Unorm
        | ShaderTextureFormatKey::ClipCoverageRgba8Unorm
        | ShaderTextureFormatKey::WorkingReducedPrecisionRgba8Unorm
        | ShaderTextureFormatKey::ResolvedMaskRgba8Unorm
        | ShaderTextureFormatKey::OutputRgba8Unorm => Ok(wgpu::TextureFormat::Rgba8Unorm),
        ShaderTextureFormatKey::FilterRgba32Float => Ok(wgpu::TextureFormat::Rgba32Float),
        ShaderTextureFormatKey::WorkingHighPrecisionRgba16Float => {
            Ok(wgpu::TextureFormat::Rgba16Float)
        }
        ShaderTextureFormatKey::OutputBgra8Unorm => Ok(wgpu::TextureFormat::Bgra8Unorm),
    }
}

#[cfg(all(test, not(target_arch = "wasm32")))]
mod convolution_shader_tests {
    use super::*;

    #[test]
    fn convolution_shader_validates_with_exact_uniform_and_sampler_free_bindings() {
        let module = wgpu::naga::front::wgsl::parse_str(CONVOLUTION_WGSL)
            .unwrap_or_else(|error| panic!("{}", error.emit_to_string(CONVOLUTION_WGSL)));
        wgpu::naga::valid::Validator::new(
            wgpu::naga::valid::ValidationFlags::all(),
            wgpu::naga::valid::Capabilities::empty(),
        )
        .validate(&module)
        .unwrap();
        let bindings: Vec<_> = module
            .global_variables
            .iter()
            .filter_map(|(_, variable)| {
                variable
                    .binding
                    .as_ref()
                    .map(|binding| (binding.binding, variable))
            })
            .collect();
        assert_eq!(bindings.len(), 4);
        assert!(bindings.iter().all(|(_, variable)| !matches!(
            module.types[variable.ty].inner,
            wgpu::naga::TypeInner::Sampler { .. }
        )));
        let uniform = bindings
            .iter()
            .find(|(binding, _)| *binding == 2)
            .unwrap()
            .1;
        let wgpu::naga::TypeInner::Struct { span, .. } = module.types[uniform.ty].inner else {
            panic!("convolution binding 2 must be the uniform structure");
        };
        assert_eq!(span, 256);
        assert_eq!(
            texture_format(ShaderTextureFormatKey::FilterRgba32Float).unwrap(),
            wgpu::TextureFormat::Rgba32Float
        );
    }
}

#[cfg(all(test, not(target_arch = "wasm32")))]
mod gaussian_shader_tests {
    use super::*;

    #[test]
    fn gaussian_shader_validates_with_exact_uniform_and_sampler_free_bindings() {
        let module = wgpu::naga::front::wgsl::parse_str(GAUSSIAN_WGSL)
            .unwrap_or_else(|error| panic!("{}", error.emit_to_string(GAUSSIAN_WGSL)));
        wgpu::naga::valid::Validator::new(
            wgpu::naga::valid::ValidationFlags::all(),
            wgpu::naga::valid::Capabilities::empty(),
        )
        .validate(&module)
        .unwrap();
        let bindings: Vec<_> = module
            .global_variables
            .iter()
            .filter_map(|(_, variable)| {
                variable
                    .binding
                    .as_ref()
                    .map(|binding| (binding.binding, variable))
            })
            .collect();
        assert_eq!(bindings.len(), 4);
        assert!(bindings.iter().all(|(_, variable)| !matches!(
            module.types[variable.ty].inner,
            wgpu::naga::TypeInner::Sampler { .. }
        )));
        let uniform = bindings
            .iter()
            .find(|(binding, _)| *binding == 2)
            .unwrap()
            .1;
        let wgpu::naga::TypeInner::Struct { span, .. } = module.types[uniform.ty].inner else {
            panic!("gaussian binding 2 must be the uniform structure");
        };
        assert_eq!(span, 256);
        assert_eq!(
            texture_format(ShaderTextureFormatKey::FilterRgba32Float).unwrap(),
            wgpu::TextureFormat::Rgba32Float
        );
    }
}

#[cfg(all(test, not(target_arch = "wasm32")))]
mod convolution_alpha_gpu_tests {
    use super::*;
    use crate::shader::{ConvolutionParameterBytes, ConvolutionParameterFacts};
    use wgpu::util::DeviceExt;

    #[test]
    fn convolution_preserve_alpha_gpu_guards_zero_and_subnormal_divisors() {
        let mut renderer =
            pollster::block_on(crate::Renderer::new(crate::Options::default())).unwrap();
        let (device, queue) = renderer
            .default_wgpu_device_queue()
            .expect("tiny-alpha shader coverage requires a GPU adapter");
        let validation = device.push_error_scope(wgpu::ErrorFilter::Validation);
        let account = Account::for_test();
        let generation = Generation::new(crate::RenderAccountingOptions::default()).unwrap();
        let cpu = account.cpu().unwrap();
        let layout =
            create_convolution_bind_group_layout(&generation, cpu, Some(&account), device).unwrap();
        let shader =
            create_convolution_shader_module(&generation, cpu, Some(&account), device).unwrap();
        let pipeline = create_convolution_render_pipeline(
            &generation,
            cpu,
            Some(&account),
            device,
            ConvolutionPassDescription {
                target_format: ShaderTextureFormatKey::FilterRgba32Float,
            },
            &layout,
            &shader,
        )
        .unwrap();
        let extent = wgpu::Extent3d {
            width: 4,
            height: 1,
            depth_or_array_layers: 1,
        };
        let texture = |label| {
            device.create_texture(&wgpu::TextureDescriptor {
                label: Some(label),
                size: extent,
                mip_level_count: 1,
                sample_count: 1,
                dimension: wgpu::TextureDimension::D2,
                format: wgpu::TextureFormat::Rgba32Float,
                usage: wgpu::TextureUsages::TEXTURE_BINDING
                    | wgpu::TextureUsages::RENDER_ATTACHMENT
                    | wgpu::TextureUsages::COPY_DST
                    | wgpu::TextureUsages::COPY_SRC,
                view_formats: &[],
            })
        };
        let source = texture("convolution tiny-alpha source");
        let sums = texture("convolution tiny-alpha straight sums");
        let output = texture("convolution tiny-alpha finalized output");
        let empty_output = texture("convolution empty-domain output");
        let min = f32::MIN_POSITIVE;
        let subnormal = f32::from_bits(min.to_bits() / 2);
        let input = [
            [0.0, 0.0, 0.0, 0.0],
            [subnormal, 0.0, 0.0, subnormal],
            [min, 0.0, 0.0, min],
            [0.5, 0.0, 0.0, 0.5],
        ];
        queue.write_texture(
            wgpu::TexelCopyTextureInfo {
                texture: &source,
                mip_level: 0,
                origin: wgpu::Origin3d::ZERO,
                aspect: wgpu::TextureAspect::All,
            },
            bytemuck::cast_slice(&input),
            wgpu::TexelCopyBufferLayout {
                offset: 0,
                bytes_per_row: Some(64),
                rows_per_image: None,
            },
            extent,
        );
        let weights = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("convolution tiny-alpha identity weight"),
            contents: &1.0_f32.to_le_bytes(),
            usage: wgpu::BufferUsages::STORAGE,
        });
        let facts = |stage| ConvolutionParameterFacts {
            capture_origin: [0.0; 2],
            capture_scale: 1.0,
            kernel_origin: [0.0; 2],
            unit_length: [1.0; 2],
            source_grid_origin: [0; 2],
            source_grid_extent: [4, 1],
            result_grid_origin: [0; 2],
            result_grid_extent: [4, 1],
            source_domain: crate::Rect::new(0.0, 0.0, 4.0, 1.0),
            result_domain: crate::Rect::new(0.0, 0.0, 4.0, 1.0),
            result_capture_origin: [0.0; 2],
            result_capture_scale: 1.0,
            order: [1; 2],
            target: [0; 2],
            stage,
            first_tap: 0,
            tap_count: 1,
            edge: 0,
            preserve_alpha: true,
            linear_srgb: false,
            first_batch: true,
            bias: 0.0,
        };
        let uniform = |facts| {
            let bytes = ConvolutionParameterBytes::try_new(facts, &account).unwrap();
            device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: Some("convolution tiny-alpha parameters"),
                contents: bytes.as_bytes(),
                usage: wgpu::BufferUsages::UNIFORM,
            })
        };
        let accumulate_parameters = uniform(facts(1));
        let finalize_parameters = uniform(facts(2));
        let mut empty_facts = facts(4);
        empty_facts.source_domain = crate::Rect::new(0.0, 0.0, 0.0, 1.0e12);
        let empty_parameters = uniform(empty_facts);
        let source_view = source.create_view(&wgpu::TextureViewDescriptor::default());
        let sums_view = sums.create_view(&wgpu::TextureViewDescriptor::default());
        let output_view = output.create_view(&wgpu::TextureViewDescriptor::default());
        let empty_output_view = empty_output.create_view(&wgpu::TextureViewDescriptor::default());
        let bindings = |auxiliary, parameters: &wgpu::Buffer| {
            device.create_bind_group(&wgpu::BindGroupDescriptor {
                label: Some("convolution tiny-alpha bindings"),
                layout: &layout,
                entries: &[
                    wgpu::BindGroupEntry {
                        binding: 0,
                        resource: wgpu::BindingResource::TextureView(&source_view),
                    },
                    wgpu::BindGroupEntry {
                        binding: 1,
                        resource: wgpu::BindingResource::TextureView(auxiliary),
                    },
                    wgpu::BindGroupEntry {
                        binding: 2,
                        resource: parameters.as_entire_binding(),
                    },
                    wgpu::BindGroupEntry {
                        binding: 3,
                        resource: weights.as_entire_binding(),
                    },
                ],
            })
        };
        let accumulate_bindings = bindings(&source_view, &accumulate_parameters);
        let finalize_bindings = bindings(&sums_view, &finalize_parameters);
        let empty_bindings = bindings(&source_view, &empty_parameters);
        let staging = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("convolution tiny-alpha readback"),
            size: 768,
            usage: wgpu::BufferUsages::MAP_READ | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });
        let mut encoder = device.create_command_encoder(&wgpu::CommandEncoderDescriptor::default());
        for (view, bindings) in [
            (&sums_view, &accumulate_bindings),
            (&output_view, &finalize_bindings),
            (&empty_output_view, &empty_bindings),
        ] {
            let attachments = [Some(wgpu::RenderPassColorAttachment {
                view,
                depth_slice: None,
                resolve_target: None,
                ops: wgpu::Operations {
                    load: wgpu::LoadOp::Clear(wgpu::Color::TRANSPARENT),
                    store: wgpu::StoreOp::Store,
                },
            })];
            let mut pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                color_attachments: &attachments,
                ..Default::default()
            });
            pass.set_pipeline(&pipeline);
            pass.set_bind_group(0, bindings, &[]);
            pass.draw(0..3, 0..1);
        }
        for (texture, offset) in [(&sums, 0), (&output, 256), (&empty_output, 512)] {
            encoder.copy_texture_to_buffer(
                wgpu::TexelCopyTextureInfo {
                    texture,
                    mip_level: 0,
                    origin: wgpu::Origin3d::ZERO,
                    aspect: wgpu::TextureAspect::All,
                },
                wgpu::TexelCopyBufferInfo {
                    buffer: &staging,
                    layout: wgpu::TexelCopyBufferLayout {
                        offset,
                        bytes_per_row: Some(256),
                        rows_per_image: None,
                    },
                },
                extent,
            );
        }
        queue.submit([encoder.finish()]);
        let (sender, receiver) = std::sync::mpsc::sync_channel(1);
        let slice = staging.slice(..);
        slice.map_async(wgpu::MapMode::Read, move |result| {
            let _ = sender.send(result);
        });
        device
            .poll(wgpu::PollType::Wait {
                submission_index: None,
                timeout: Some(std::time::Duration::from_secs(5)),
            })
            .unwrap();
        receiver
            .recv_timeout(std::time::Duration::from_secs(5))
            .unwrap()
            .unwrap();
        let mapped = slice.get_mapped_range();
        let read_pixel = |row: usize, index: usize| {
            let mut pixel = [0.0_f32; 4];
            for (channel, value) in pixel.iter_mut().enumerate() {
                let offset = row * 256 + index * 16 + channel * 4;
                *value = f32::from_le_bytes(mapped[offset..offset + 4].try_into().unwrap());
            }
            pixel
        };
        let straight: Vec<_> = (0..4).map(|index| read_pixel(0, index)).collect();
        let finalized: Vec<_> = (0..4).map(|index| read_pixel(1, index)).collect();
        let empty: Vec<_> = (0..4).map(|index| read_pixel(2, index)).collect();
        drop(mapped);
        staging.unmap();
        assert!(
            pollster::block_on(validation.pop()).is_none(),
            "the actual convolution passes must validate"
        );
        for row in [&straight, &finalized] {
            assert!(
                row.iter().flatten().all(|value| value.is_finite()),
                "guarded alpha division must remain finite: {row:?}"
            );
            assert_eq!(
                row[0], [0.0; 4],
                "zero alpha has no straight or premultiplied color"
            );
            assert_eq!(
                row[1], [0.0; 4],
                "a positive subnormal alpha is effective zero before division"
            );
        }
        assert_eq!(
            empty,
            vec![[0.0; 4]; 4],
            "an empty source ignores its huge unused axis before integer mapping"
        );
        assert_eq!(straight[3], [1.0, 0.0, 0.0, 0.0]);
        assert_eq!(finalized[3], [0.5, 0.0, 0.0, 0.5]);
        // At the normal/subnormal boundary, WGSL permits loss in intermediate
        // operations. Check finite bounded output, without claiming bit identity.
        assert!((0.0..=1.0).contains(&straight[2][0]));
        assert!((0.0..=min).contains(&finalized[2][3]));
        assert!(
            finalized[2][..3]
                .iter()
                .all(|value| (0.0..=finalized[2][3]).contains(value))
        );
    }
}

#[cfg(all(test, not(target_arch = "wasm32")))]
mod offset_shader_tests {
    use super::*;

    #[test]
    fn offset_shader_validates_with_one_texture_and_exact_sampler_free_uniform() {
        let module = wgpu::naga::front::wgsl::parse_str(OFFSET_WGSL)
            .unwrap_or_else(|error| panic!("{}", error.emit_to_string(OFFSET_WGSL)));
        wgpu::naga::valid::Validator::new(
            wgpu::naga::valid::ValidationFlags::all(),
            wgpu::naga::valid::Capabilities::empty(),
        )
        .validate(&module)
        .unwrap();
        let bindings: Vec<_> = module
            .global_variables
            .iter()
            .filter_map(|(_, variable)| {
                variable
                    .binding
                    .as_ref()
                    .map(|binding| (binding.group, binding.binding, variable))
            })
            .collect();
        assert_eq!(bindings.len(), 2);
        let (_, _, source) = bindings
            .iter()
            .find(|(group, binding, _)| *group == 0 && *binding == 0)
            .unwrap();
        assert!(matches!(
            module.types[source.ty].inner,
            wgpu::naga::TypeInner::Image { .. }
        ));
        let (_, _, uniform) = bindings
            .iter()
            .find(|(group, binding, _)| *group == 0 && *binding == 1)
            .unwrap();
        assert_eq!(uniform.space, wgpu::naga::AddressSpace::Uniform);
        let wgpu::naga::TypeInner::Struct { span, ref members } = module.types[uniform.ty].inner
        else {
            panic!("offset binding 1 must contain the exact wire structure");
        };
        assert_eq!(span, 256);
        assert_eq!(
            members
                .iter()
                .map(|member| member.offset)
                .collect::<Vec<_>>(),
            [0, 16, 32, 48, 64, 80]
        );
        for (format, expected) in [
            (
                ShaderTextureFormatKey::WorkingHighPrecisionRgba16Float,
                wgpu::TextureFormat::Rgba16Float,
            ),
            (
                ShaderTextureFormatKey::WorkingReducedPrecisionRgba8Unorm,
                wgpu::TextureFormat::Rgba8Unorm,
            ),
        ] {
            assert_eq!(texture_format(format).unwrap(), expected);
        }
    }

    #[test]
    fn alpha_mask_shader_retains_valid_composition_after_shared_cubic_extraction() {
        let module = wgpu::naga::front::wgsl::parse_str(LAYER_COMPOSITE_WGSL)
            .unwrap_or_else(|error| panic!("{}", error.emit_to_string(LAYER_COMPOSITE_WGSL)));
        wgpu::naga::valid::Validator::new(
            wgpu::naga::valid::ValidationFlags::all(),
            wgpu::naga::valid::Capabilities::empty(),
        )
        .validate(&module)
        .unwrap();
    }
}

#[cfg(all(test, not(target_arch = "wasm32")))]
mod gaussian_pipeline_gpu_tests {
    use super::*;

    #[test]
    fn gaussian_pipeline_entry_points_validate_with_only_reached_resources() {
        let mut renderer =
            pollster::block_on(crate::Renderer::new(crate::Options::default())).unwrap();
        let (device, _) = renderer
            .default_wgpu_device_queue()
            .expect("Gaussian pipeline validation requires a GPU adapter");
        let validation = device.push_error_scope(wgpu::ErrorFilter::Validation);
        let account = Account::for_test();
        let generation = Generation::new(crate::RenderAccountingOptions::default()).unwrap();
        let cpu = account.cpu().unwrap();
        let shader =
            create_gaussian_shader_module(&generation, cpu, Some(&account), device).unwrap();
        for program in [
            super::super::ShaderProgramKey::GaussianConstant,
            super::super::ShaderProgramKey::GaussianSource,
            super::super::ShaderProgramKey::Gaussian,
        ] {
            for target_format in [
                ShaderTextureFormatKey::WorkingHighPrecisionRgba16Float,
                ShaderTextureFormatKey::WorkingReducedPrecisionRgba8Unorm,
                ShaderTextureFormatKey::FilterRgba32Float,
            ] {
                if program == super::super::ShaderProgramKey::Gaussian
                    && target_format != ShaderTextureFormatKey::FilterRgba32Float
                {
                    continue;
                }
                let description = GaussianPassDescription {
                    target_format,
                    program,
                };
                let layout = create_gaussian_bind_group_layout(
                    &generation,
                    cpu,
                    Some(&account),
                    device,
                    description,
                )
                .unwrap();
                let _pipeline = create_gaussian_render_pipeline(
                    &generation,
                    cpu,
                    Some(&account),
                    device,
                    description,
                    &layout,
                    &shader,
                )
                .unwrap();
            }
        }
        assert!(
            pollster::block_on(validation.pop()).is_none(),
            "actual Gaussian entry points must accept their exact resource layouts"
        );
    }
}
