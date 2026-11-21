use crate::{
    frame::{LightingPassFacts, LightingStage},
    shader::ShaderLightingStageKey,
};

pub(super) fn shader_stage(facts: &LightingPassFacts) -> ShaderLightingStageKey {
    match facts.stage() {
        LightingStage::ResampleBump => ShaderLightingStageKey::ResampleBump,
        LightingStage::Shade if facts.reads_source() => ShaderLightingStageKey::Shade,
        LightingStage::Shade => ShaderLightingStageKey::ShadeZeroBump,
        LightingStage::Resolve => ShaderLightingStageKey::Resolve,
    }
}

pub(super) fn valid_scissor(
    [x, y, width, height]: [u32; 4],
    extent: crate::geometry::PhysicalSize,
) -> bool {
    width != 0
        && height != 0
        && x.checked_add(width)
            .is_some_and(|end| end <= extent.width())
        && y.checked_add(height)
            .is_some_and(|end| end <= extent.height())
}
