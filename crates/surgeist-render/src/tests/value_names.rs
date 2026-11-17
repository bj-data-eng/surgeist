//! Names that stand for values in error messages, in place of their `Debug` form.

#[test]
fn values_in_messages_have_written_names() {
    use crate::{geometry::CoordinateSpaceId, geometry::CoordinateSpaceKind, layer::BlendMode};
    for (mode, name) in [
        (BlendMode::Normal, "normal"),
        (BlendMode::Multiply, "multiply"),
        (BlendMode::Screen, "screen"),
        (BlendMode::Overlay, "overlay"),
        (BlendMode::Darken, "darken"),
        (BlendMode::Lighten, "lighten"),
        (BlendMode::ColorDodge, "color dodge"),
        (BlendMode::ColorBurn, "color burn"),
        (BlendMode::HardLight, "hard light"),
        (BlendMode::SoftLight, "soft light"),
        (BlendMode::Difference, "difference"),
        (BlendMode::Exclusion, "exclusion"),
        (BlendMode::Hue, "hue"),
        (BlendMode::Saturation, "saturation"),
        (BlendMode::Color, "color"),
        (BlendMode::Luminosity, "luminosity"),
        (BlendMode::Plus, "plus"),
    ] {
        assert_eq!(mode.label(), name);
    }
    for (kind, name) in [
        (CoordinateSpaceKind::Local, "local"),
        (CoordinateSpaceKind::Viewport, "viewport"),
        (CoordinateSpaceKind::Surface, "surface"),
        (
            CoordinateSpaceKind::Named(CoordinateSpaceId::try_new(9).unwrap()),
            "named",
        ),
    ] {
        assert_eq!(kind.label(), name);
    }
    for (format, name) in [
        (crate::surface::Format::Rgba8, "RGBA8"),
        (crate::surface::Format::Bgra8, "BGRA8"),
    ] {
        assert_eq!(format.label(), name);
    }
    for (format, name) in [
        (wgpu::TextureFormat::Rgba8Unorm, "Rgba8Unorm"),
        (wgpu::TextureFormat::Rgba8UnormSrgb, "Rgba8UnormSrgb"),
        (wgpu::TextureFormat::Bgra8Unorm, "Bgra8Unorm"),
        (wgpu::TextureFormat::Bgra8UnormSrgb, "Bgra8UnormSrgb"),
        (wgpu::TextureFormat::Rgba16Float, "Rgba16Float"),
        (wgpu::TextureFormat::Rgba32Float, "Rgba32Float"),
        (
            wgpu::TextureFormat::R8Unorm,
            "a texture format this crate does not use",
        ),
    ] {
        assert_eq!(crate::texture::texture_format_label(format), name);
    }
    for (format, name) in [
        (
            crate::resource::WorkingFormat::HighPrecision,
            "high precision",
        ),
        (
            crate::resource::WorkingFormat::ReducedPrecision,
            "reduced precision",
        ),
    ] {
        assert_eq!(format.label(), name);
    }
    use wgpu::TextureUsages as U;
    for (usage, text) in [
        (U::empty(), "no usage"),
        (U::COPY_SRC, "copy source"),
        (
            U::TEXTURE_BINDING | U::STORAGE_BINDING,
            "texture binding, storage binding",
        ),
        (
            U::COPY_SRC
                | U::COPY_DST
                | U::TEXTURE_BINDING
                | U::STORAGE_BINDING
                | U::RENDER_ATTACHMENT,
            "copy source, copy destination, texture binding, storage binding, render attachment",
        ),
    ] {
        assert_eq!(crate::texture::UsageText(usage).to_string(), text);
    }
}
