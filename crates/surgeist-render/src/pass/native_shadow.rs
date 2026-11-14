//! Serialization of the admitted native shadow merge, with only actual reads.
use crate::{
    FilterColorSpace, Result,
    accounting::Account,
    frame::{NativeShadowOriginal, NativeShadowPassFacts},
    shader::{
        NativeShadowParameterBytes, NativeShadowParameterFacts, ShaderNativeShadowSourceKey,
        ShaderProgramKey,
    },
};

pub(super) const fn source_key(source: NativeShadowOriginal) -> ShaderNativeShadowSourceKey {
    match source {
        NativeShadowOriginal::Finite => ShaderNativeShadowSourceKey::Finite,
        NativeShadowOriginal::Periodic => ShaderNativeShadowSourceKey::Periodic,
        NativeShadowOriginal::Constant => ShaderNativeShadowSourceKey::Constant,
    }
}
pub(super) fn shader_program(work: &NativeShadowPassFacts) -> ShaderProgramKey {
    let map = work.mapping();
    ShaderProgramKey::NativeShadow {
        original: source_key(map.original_kind()),
        shifted_alpha: map.has_shifted_alpha(),
    }
}
pub(super) fn parameters(
    work: &NativeShadowPassFacts,
    account: &Account,
) -> Result<NativeShadowParameterBytes> {
    let map = work.mapping();
    let original = map.original_mapping();
    let operation = work.operation();
    let color = original.color();
    let flood = operation.flood_color();
    account.work(crate::accounting::RenderWork::CoordinateVisits, 8)?;
    NativeShadowParameterBytes::try_new(
        NativeShadowParameterFacts {
            original: source_key(map.original_kind()),
            shifted_alpha: map.has_shifted_alpha(),
            source_size: original.source_extent(),
            target_size: map.target_extent(),
            source_base: original.source_base(),
            source_clip: original.source_clip(),
            original_filter_clip: map.original_filter_clip(),
            output_clip: map.output_clip(),
            periodic_base: original.periodic_base(),
            periodic_step: original.periodic_step(),
            original_color: [
                color.r() * color.a(),
                color.g() * color.a(),
                color.b() * color.a(),
                color.a(),
            ],
            flood_color: [flood.r(), flood.g(), flood.b(), flood.a()],
            flood_opacity: operation.flood_opacity(),
            linear: operation.color_space() == FilterColorSpace::LinearSrgb,
        },
        account,
    )
}
