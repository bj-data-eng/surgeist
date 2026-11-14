//! Serialization of the frame-owned, numerically qualified Gaussian payload.
use crate::{
    FilterColorSpace, FilterEdgeMode, Result,
    accounting::Account,
    frame::{GaussianPassFacts, GaussianStage},
    gaussian::GaussianSourceDescriptor,
    shader::{
        GaussianParameterBytes, GaussianParameterFacts, ShaderDataBindingKey, ShaderProgramKey,
    },
};

pub(super) fn shader_program(work: &GaussianPassFacts) -> ShaderProgramKey {
    match work.stage() {
        GaussianStage::Constant => ShaderProgramKey::GaussianConstant,
        GaussianStage::Accumulate { .. } => ShaderProgramKey::Gaussian,
        _ => ShaderProgramKey::GaussianSource,
    }
}
pub(super) fn data_bindings(work: &GaussianPassFacts) -> &'static [ShaderDataBindingKey] {
    match work.stage() {
        GaussianStage::Accumulate { .. } => &[
            ShaderDataBindingKey::GaussianParameters,
            ShaderDataBindingKey::GaussianKernel,
        ],
        _ => &[ShaderDataBindingKey::GaussianParameters],
    }
}
pub(super) fn parameters(
    work: &GaussianPassFacts,
    account: &Account,
) -> Result<GaussianParameterBytes> {
    let map = work.mapping();
    let (stage, axis, first_tap, tap_count, first) = match work.stage() {
        GaussianStage::RebaseSource => (0, 0, 0, 0, false),
        GaussianStage::PassThrough => (1, 0, 0, 0, false),
        GaussianStage::Prepare => (2, 0, 0, 0, false),
        GaussianStage::Accumulate {
            axis,
            first_tap,
            tap_count,
            first,
        } => (3, axis.shader_value(), first_tap, tap_count, first),
        GaussianStage::Resolve => (4, 0, 0, 0, false),
        GaussianStage::Constant => (5, 0, 0, 0, false),
    };
    let edge_mode = match work.source() {
        GaussianSourceDescriptor::Finite { edge, .. } => match edge {
            FilterEdgeMode::Duplicate => 0,
            FilterEdgeMode::Wrap => 1,
            FilterEdgeMode::Mirror => 2,
            FilterEdgeMode::None => 3,
        },
        GaussianSourceDescriptor::Periodic { .. } => 1,
        GaussianSourceDescriptor::Constant { .. } => 3,
    };
    let color = if work.stage() == GaussianStage::Constant {
        account.work(crate::accounting::RenderWork::CoordinateVisits, 4)?;
        let color = map.color();
        [
            color.r() * color.a(),
            color.g() * color.a(),
            color.b() * color.a(),
            color.a(),
        ]
    } else {
        [0.0; 4]
    };
    let facts = GaussianParameterFacts {
        stage,
        source_size: map.source_extent(),
        target_size: map.target_extent(),
        source_base: map.source_base(),
        source_clip: map.source_clip(),
        output_clip: map.output_clip(),
        periodic_base: map.periodic_base(),
        periodic_step: map.periodic_step(),
        edge_mode,
        periodic: map.periodic(),
        direct: map.direct(),
        axis,
        first_tap,
        tap_count,
        first,
        linear: work.operation().color_space() == FilterColorSpace::LinearSrgb,
        converted: map.converted(),
        color,
    };
    match work.channels() {
        crate::frame::GaussianChannels::Rgba => GaussianParameterBytes::try_new(facts, account),
        crate::frame::GaussianChannels::Alpha => GaussianParameterBytes::try_new_for_channels(
            facts,
            crate::frame::GaussianChannels::Alpha,
            account,
        ),
    }
}
