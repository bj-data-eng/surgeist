use crate::accounting::{Generation, GenerationClass, OwnedGpu};
use crate::texture::{TextureDescriptor, headless_texture_descriptor};
use crate::{Format, PhysicalSize, Result};

pub(crate) fn create_headless_texture(
    device: &wgpu::Device,
    generation: &Generation,
    physical_size: PhysicalSize,
    format: Format,
) -> Result<(OwnedGpu<wgpu::Texture>, OwnedGpu<wgpu::TextureView>)> {
    let descriptor = headless_texture_descriptor(physical_size, format)?;
    create_texture(device, generation, "Surgeist headless target", descriptor)
}

pub(crate) fn create_texture(
    device: &wgpu::Device,
    generation: &Generation,
    label: &'static str,
    descriptor: TextureDescriptor,
) -> Result<(OwnedGpu<wgpu::Texture>, OwnedGpu<wgpu::TextureView>)> {
    let bytes = u64::from(descriptor.physical_size().width())
        .checked_mul(u64::from(descriptor.physical_size().height()))
        .and_then(|n| n.checked_mul(4))
        .ok_or_else(|| {
            crate::Error::invalid_value(
                "target texture byte length",
                format!(
                    "{}x{} RGBA8 pixels",
                    descriptor.physical_size().width(),
                    descriptor.physical_size().height()
                ),
                "must fit in u64",
            )
        })?;
    let texture = generation.allocation(GenerationClass::SurfaceTargets, bytes, || {
        device.create_texture(&wgpu::TextureDescriptor {
            label: Some(label),
            size: wgpu::Extent3d {
                width: descriptor.physical_size().width(),
                height: descriptor.physical_size().height(),
                depth_or_array_layers: 1,
            },
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: descriptor.format().into(),
            usage: descriptor.wgpu_usage(),
            view_formats: &[],
        })
    })?;
    let view = texture.create_view(&wgpu::TextureViewDescriptor::default())?;
    Ok((texture, view))
}
