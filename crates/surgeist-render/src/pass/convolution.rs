//! Runtime preparation of the checked convolution graph payload.
use crate::{
    FilterColorSpace, FilterEdgeMode, Result,
    accounting::Account,
    frame::{ConvolutionStage, GraphLoweringConvolution},
    shader::{ConvolutionParameterBytes, ConvolutionParameterFacts},
};

pub(super) fn parameters(
    operation: &GraphLoweringConvolution,
    account: &Account,
) -> Result<ConvolutionParameterBytes> {
    let map = operation.mapping();
    let prepared = operation.operation();
    let (stage, first_tap, tap_count, first_batch) = match operation.stage() {
        ConvolutionStage::RebaseSource => (5, 0, 0, false),
        ConvolutionStage::Prepare => (0, 0, 0, false),
        ConvolutionStage::Accumulate {
            first_tap,
            tap_count,
            first,
        } => (1, first_tap, tap_count, first),
        ConvolutionStage::Finalize => (2, 0, 0, false),
        ConvolutionStage::Resolve => (3, 0, 0, false),
        ConvolutionStage::PassThrough => (4, 0, 0, false),
    };
    ConvolutionParameterBytes::try_new(
        ConvolutionParameterFacts {
            capture_origin: map.capture_origin(),
            capture_scale: map.capture_scale(),
            kernel_origin: map.kernel_origin(),
            unit_length: map.unit_length(),
            source_grid_origin: map.source_grid_origin(),
            source_grid_extent: map.source_grid_extent(),
            result_grid_origin: map.result_grid_origin(),
            result_grid_extent: map.result_grid_extent(),
            source_domain: map.source_domain(),
            result_domain: map.result_domain(),
            result_capture_origin: map.result_capture_origin(),
            result_capture_scale: map.result_capture_scale(),
            order: prepared.order(),
            target: prepared.target(),
            stage,
            first_tap,
            tap_count,
            edge: match prepared.edge_mode() {
                FilterEdgeMode::Duplicate => 0,
                FilterEdgeMode::Wrap => 1,
                FilterEdgeMode::Mirror => 2,
                FilterEdgeMode::None => 3,
            },
            preserve_alpha: prepared.preserve_alpha(),
            linear_srgb: prepared.color_space() == FilterColorSpace::LinearSrgb,
            first_batch,
            bias: f64::from(prepared.bias()),
        },
        account,
    )
}
