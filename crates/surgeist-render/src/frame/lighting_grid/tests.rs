use super::*;
use crate::lighting::LightingPreparationError;

fn plan(
    source: Rect,
    filter: Rect,
    result: Rect,
    spacing: Option<[f64; 2]>,
    scale: f64,
    has_source: bool,
) -> Result<Option<LightingGrid>> {
    LightingGrid::try_new(
        FrameContext::try_for_spatial_test(scale).unwrap(),
        FilterDomains::try_new(source, filter, result).unwrap(),
        Transform::identity(),
        spacing,
        has_source,
        &Account::for_test(),
    )
}

#[test]
fn reconstruction_and_sobel_reach_different_windows() {
    // F contains the kernel centers 1, 3 and 5. Output center 2.5
    // reconstructs from nodes 0 and 1. Node 1 also differentiates at node 2.
    let filter = Rect::new(0.0, 0.0, 6.0, 1.0);
    let grid = plan(
        filter,
        filter,
        Rect::new(2.0, 0.0, 1.0, 1.0),
        Some([2.0, 1.0]),
        1.0,
        true,
    )
    .unwrap()
    .unwrap();
    assert_eq!(grid.counts, [3, 1]);
    assert_eq!(grid.shade.index_origin, [0, 0]);
    assert_eq!(grid.shade.edge_flags, 0b1101);
    assert_eq!(filter_grid::extent(grid.shade.spatial), [2, 1]);
    assert_eq!(filter_grid::extent(grid.bump.spatial), [3, 1]);
    assert_eq!(grid.bump.edge_flags, 0b1111);
    assert_eq!(grid.shade_source_offset, [0, 0]);
    assert_eq!(grid.resolve_map, [0.75, 0.0, 0.5, 1.0]);
    assert_eq!(grid.output_clip, [0.0, 0.0, 1.0, 1.0]);

    // The reached input tap pairs are [0,1], [2,3], [4,5].
    let capture = grid.capture.unwrap();
    assert_eq!(device_origin(capture.spatial), [0, 0]);
    assert_eq!(filter_grid::extent(capture.spatial), [6, 1]);
    assert_eq!(capture.sample_map, [0.5, 0.0, 2.0, 1.0]);
    assert_eq!(capture.clip, [0.0, 0.0, 6.0, 1.0]);
}

#[test]
fn allocation_crops_do_not_reanchor_canvas_or_input_coverage() {
    let filter = Rect::new(0.0, 0.0, 12.0, 4.0);
    let grid = plan(
        Rect::new(3.25, 0.5, 4.25, 2.25),
        filter,
        Rect::new(4.25, 1.25, 2.25, 0.5),
        None,
        1.0,
        true,
    )
    .unwrap()
    .unwrap();

    // O uses x cells 4,5,6 and y cell 1. Each resolves its
    // centered node and the next, then Sobel adds one neighbor either side.
    assert_eq!(device_origin(grid.output), [4, 1]);
    assert_eq!(grid.output_clip, [0.25, 0.25, 2.5, 0.75]);
    assert_eq!(grid.shade.index_origin, [4, 1]);
    assert_eq!(grid.shade.edge_flags, 0);
    assert_eq!(filter_grid::extent(grid.shade.spatial), [4, 2]);
    assert_eq!(grid.bump.index_origin, [3, 0]);
    assert_eq!(grid.bump.edge_flags, 0b1100);
    assert_eq!(filter_grid::extent(grid.bump.spatial), [6, 4]);
    assert_eq!(grid.shade_source_offset, [1, 1]);
    assert_eq!(grid.resolve_map, [0.0, 0.0, 1.0, 1.0]);

    let capture = grid.capture.unwrap();
    assert_eq!(device_origin(capture.spatial), [3, 0]);
    assert_eq!(filter_grid::extent(capture.spatial), [5, 3]);
    assert_eq!(capture.clip, [0.25, 0.5, 4.5, 2.75]);
    assert_eq!(capture.sample_map, [0.0, 0.0, 1.0, 1.0]);
}

#[test]
fn nowhere_reached_source_can_fall_between_coarse_taps() {
    // Centers 2, 6, 10 can read x texels 1,2 / 5,6 / 9,10.
    // D exclusively covers x texels 3,4 within the overall tap envelope.
    let filter = Rect::new(0.0, 0.0, 12.0, 1.0);
    let grid = plan(
        Rect::new(3.0, 0.0, 2.0, 1.0),
        filter,
        Rect::new(4.0, 0.0, 1.0, 1.0),
        Some([4.0, 1.0]),
        1.0,
        true,
    )
    .unwrap()
    .unwrap();
    assert!(grid.capture.is_none());
    assert_eq!(filter_grid::extent(grid.output), [1, 1]);
    assert_eq!(grid.resolve_map, [0.625, 0.0, 0.25, 1.0]);
}

#[test]
fn full_canvas_can_exceed_wgsl_index_range() {
    // The indices near O are too large to add a half cell directly in f64.
    let filter = Rect::new(-5_000_000_000_000_000.0, 0.0, 5_000_000_000_000_010.0, 1.0);
    let grid = plan(
        filter,
        filter,
        Rect::new(1.0, 0.0, 1.0, 1.0),
        None,
        1.0,
        true,
    )
    .unwrap()
    .unwrap();
    assert_eq!(grid.counts, [5_000_000_000_000_010, 1]);
    assert_eq!(grid.shade.index_origin, [5_000_000_000_000_001, 0]);
    assert_eq!(filter_grid::extent(grid.shade.spatial), [2, 1]);
    assert_eq!(grid.bump.index_origin, [5_000_000_000_000_000, 0]);
    let capture = grid.capture.unwrap();
    assert_eq!(device_origin(capture.spatial), [0, 0]);
    assert_eq!(filter_grid::extent(capture.spatial), [5, 1]);
    assert_eq!(grid.resolve_map, [0.0, 0.0, 1.0, 1.0]);
    assert_eq!(capture.sample_map, [0.0, 0.0, 1.0, 1.0]);
}

#[test]
fn integral_translation_preserves_both_mapping_phases_and_coverage() {
    let at = |x| {
        let grid = plan(
            Rect::new(x + 1.25, 0.0, 3.0, 2.0),
            Rect::new(x + 0.25, 0.0, 8.0, 2.0),
            Rect::new(x + 2.25, 0.0, 2.5, 1.0),
            Some([2.0, 1.0]),
            1.0,
            true,
        )
        .unwrap()
        .unwrap();
        let capture = grid.capture.unwrap();
        (
            grid.resolve_map,
            grid.output_clip,
            grid.shade.index_origin,
            filter_grid::extent(capture.spatial),
            capture.sample_map,
            capture.clip,
        )
    };
    assert_eq!(at(1_000_000_000.0), at(0.0));
}

#[test]
fn default_spacing_at_nonbinary_density_keeps_the_raster_aligned() {
    let unit = Rect::new(0.0, 0.0, 1.0, 1.0);
    let grid = plan(unit, unit, unit, None, 3.0, true).unwrap().unwrap();
    assert_eq!(grid.counts, [3, 3]);
    assert_eq!(grid.resolve_map, [0.0, 0.0, 1.0, 1.0]);
    assert_eq!(grid.capture.unwrap().sample_map, [0.0, 0.0, 1.0, 1.0]);
}

#[test]
fn empty_intersection_and_absent_bump_have_different_output_membership() {
    let unit = Rect::new(0.0, 0.0, 1.0, 1.0);
    assert!(
        plan(unit, unit, Rect::new(1.0, 1.0, 1.0, 1.0), None, 1.0, true)
            .unwrap()
            .is_none()
    );
    let zero_bump = plan(unit, unit, unit, None, 1.0, false).unwrap().unwrap();
    assert!(zero_bump.capture.is_none());
    assert_eq!(filter_grid::extent(zero_bump.output), [1, 1]);
    assert_eq!(zero_bump.shade.edge_flags, 0b1111);
}

#[test]
fn disjoint_output_skips_unreached_density_arithmetic() {
    let unit = Rect::new(0.0, 0.0, 1.0, 1.0);
    let disjoint = Rect::new(2.0, 0.0, 1.0, 1.0);
    let context = FrameContext::try_for_spatial_test(f64::MAX).unwrap();
    let transform = Transform::try_new([2.0, 0.0, 0.0, 2.0, 0.0, 0.0]).unwrap();
    let domains = FilterDomains::try_new(unit, unit, disjoint).unwrap();
    assert!(
        LightingGrid::try_new(
            context,
            domains,
            transform,
            None,
            true,
            &Account::for_test()
        )
        .unwrap()
        .is_none()
    );
}

#[test]
fn collapsed_canvas_and_subtexel_address_report_lighting_numeric_error() {
    let unit = Rect::new(0.0, 0.0, 1.0, 1.0);
    let collapsed = Rect::new(1.0e20, 0.0, 1.0, 1.0);
    let error = plan(unit, collapsed, collapsed, None, 1.0, true).unwrap_err();
    assert!(matches!(
        error.lighting_diagnostic(),
        Some(LightingPreparationError::Numeric {
            phase: Phase::GridExtent,
            ..
        })
    ));

    let oversized = Rect::new(0.0, 0.0, 9_007_199_254_740_992.0, 1.0);
    let error = plan(unit, oversized, unit, None, 1.0, true).unwrap_err();
    assert!(matches!(
        error.lighting_diagnostic(),
        Some(LightingPreparationError::Numeric {
            phase: Phase::GridExtent,
            ..
        })
    ));

    // Output reconstructs from the first node. Its Sobel neighbor is
    // 16,777,218 cells farther than the capture at the first node.
    let filter = Rect::new(0.0, 0.0, 33_554_436.0, 1.0);
    let error = plan(
        Rect::new(8_388_608.0, 0.0, 2.0, 1.0),
        filter,
        unit,
        Some([16_777_218.0, 1.0]),
        1.0,
        true,
    )
    .unwrap_err();
    assert!(matches!(
        error.lighting_diagnostic(),
        Some(LightingPreparationError::Numeric {
            phase: Phase::Sampling,
            ..
        })
    ));
}

#[test]
fn capture_rejects_a_phase_that_rounds_to_the_next_texel() {
    let filter = Rect::new(1.0 - 2.0_f64.powi(-26), 0.0, 5.0, 1.0);
    let result = Rect::new(3.0, 0.0, 1.0, 1.0);
    let source = Rect::new(1.0, 0.0, 4.0, 1.0);
    // The capture starts at x=1. Its first bump coordinate is just
    // below 1 relative to that origin, with integral D and O coverage.
    let error = plan(source, filter, result, None, 1.0, true).unwrap_err();
    assert!(matches!(
        error.lighting_diagnostic(),
        Some(LightingPreparationError::Numeric {
            phase: Phase::Sampling,
            value
        }) if *value < 1.0 && *value > 0.99
    ));
}

#[test]
fn capture_does_not_discard_a_covered_cell_when_absolute_edges_round_together() {
    // Both D edges round to device x=9. Its sliver is actually in cell 8,
    // which is the +1 capture tap of the last reached bump cell (7).
    let filter = Rect::new(0.0, 0.0, 1.0, 1.0);
    let edge = 0.9_f64.next_down();
    let source = Rect::new(edge, 0.0, 0.9 - edge, 1.0);
    let result = Rect::new(0.5, 0.0, 0.1, 0.1);
    let error = plan(source, filter, result, None, 10.0, true).unwrap_err();
    assert!(matches!(
        error.lighting_diagnostic(),
        Some(LightingPreparationError::Numeric {
            phase: Phase::GridMapping,
            ..
        })
    ));
}

#[test]
fn resolve_rejects_an_interior_floor_crossing_after_upload() {
    // The first and last phases survive upload at 0.5 and 5.5. The middle
    // coordinate is just below 3 in f64 and rounds to 3 in the shader.
    let filter = Rect::new(0.1, 0.0, 4.0, 1.0);
    let result = Rect::new(0.0, 0.0, 3.0, 1.0);
    let error = plan(
        filter,
        filter,
        result,
        Some([0.4_f64.next_up(), 1.0]),
        1.0,
        false,
    )
    .unwrap_err();
    assert!(matches!(
        error.lighting_diagnostic(),
        Some(LightingPreparationError::Numeric {
            phase: Phase::Sampling,
            value
        }) if *value < 3.0 && *value > 2.99
    ));
}

#[test]
fn three_raster_unit_stride_preserves_an_aligned_interior_center() {
    // Pixel 1 and the first kernel node both have center x=1.5. Their
    // canonical coordinate is exactly zero despite the reciprocal rounding.
    let filter = Rect::new(0.0, 0.0, 9.0, 1.0);
    let result = Rect::new(0.0, 0.0, 3.0, 1.0);
    let grid = plan(filter, filter, result, Some([3.0, 1.0]), 1.0, true)
        .unwrap()
        .unwrap();
    assert_eq!(grid.shade.index_origin, [0, 0]);
    assert_eq!(filter_grid::extent(grid.shade.spatial), [2, 1]);
    assert_eq!(grid.resolve_map, [-1.0 / 3.0, 0.0, 1.0 / 3.0, 1.0]);
    assert_eq!(grid.capture.unwrap().sample_map, [0.0, 0.0, 3.0, 1.0]);
}

#[test]
fn output_does_not_discard_a_covered_cell_due_to_absolute_rounding() {
    // P covers cells 8 and 9 at rho=10. The leading sliver in cell 8
    // cannot survive f32 clip upload even after qualifying its allocation.
    let filter = Rect::new(0.0, 0.0, 1.0, 1.0);
    let edge = 0.9_f64.next_down();
    let result = Rect::new(edge, 0.0, 1.0 - edge, 0.1);
    let error = plan(filter, filter, result, None, 10.0, false).unwrap_err();
    assert!(matches!(
        error.lighting_diagnostic(),
        Some(LightingPreparationError::Numeric {
            phase: Phase::GridMapping,
            ..
        })
    ));
}

#[test]
fn distant_filter_anchor_retains_the_default_raster_node_phase() {
    let filter = Rect::new(-2.0e15, 0.0, 2.0e15 + 1.0, 1.0);
    let result = Rect::new(0.0, 0.0, 1.0, 1.0);
    let grid = plan(filter, filter, result, None, 3.0, false)
        .unwrap()
        .unwrap();
    let index = 6_000_000_000_000_000;
    assert_eq!(grid.shade.index_origin[0], index);
    assert_eq!(grid.relative_node(0, index, 0.0).unwrap(), 1.0 / 6.0);
    assert_eq!(grid.relative_node(0, index, 1.0 / 6.0).unwrap(), 0.0);
}

#[test]
fn explicit_spacing_retains_the_rebased_product_residual() {
    let filter = Rect::new(-2.0e15, 0.0, 2.0e15 + 1.0, 1.0);
    let result = Rect::new(0.0, 0.0, 1.0, 1.0);
    let grid = plan(filter, filter, result, Some([1.0 / 3.0, 1.0]), 3.0, false)
        .unwrap()
        .unwrap();
    let index = 6_000_000_000_000_000;
    assert_eq!(grid.shade.index_origin[0], index);
    // The explicit binary64 1/3 is (1 - 2^-54)/3. Over 6e15 steps
    // the difference from exact thirds moves the center by about 0.111.
    let expected = 1.0 / 6.0 - 2.0e15 * 2.0_f64.powi(-54);
    assert!((grid.relative_node(0, index, 0.0).unwrap() - expected).abs() < 1.0e-15);
}
