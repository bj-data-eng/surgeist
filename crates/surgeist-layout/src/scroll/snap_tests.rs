use crate::{
    AvailableSpaceOf, AxisSizingRequestOf, Direction, Display, FlowAxes, LayoutRootRequestOf,
    LayoutScalar, NearestScrollportOf, NodeInputOf, OrthogonalFallbackLimitOf,
    OrthogonalFallbackOf, PhysicalAxis, PhysicalScrollOffsetOf, PhysicalScrollRangeOf, Point,
    PreferredSizeOf, ScrollCoordinateErrorOf, ScrollMarginOf, ScrollRectErrorOf, ScrollRectOf,
    ScrollSnapAlign, ScrollSnapAlignValue, ScrollSnapAxis, ScrollSnapContextOf, ScrollSnapType,
    ScrollTargetGeometryOf, Size, SizingOperation, SizingRequestOf, WritingMode, compute_layout,
    test_support::layout_tree::PublicLayoutTreeOf,
};

fn target<S: LayoutScalar>(margin: ScrollMarginOf<S>) -> ScrollTargetGeometryOf<S> {
    configured_target(
        margin,
        FlowAxes::new(WritingMode::HorizontalTb, Direction::Ltr),
        ScrollSnapAlign::default(),
    )
}

fn configured_target<S: LayoutScalar>(
    margin: ScrollMarginOf<S>,
    axes: FlowAxes,
    alignment: ScrollSnapAlign,
) -> ScrollTargetGeometryOf<S> {
    let tree = PublicLayoutTreeOf::new().style(
        0,
        NodeInputOf {
            display: Display::Block,
            writing_mode: axes.writing_mode(),
            direction: axes.direction(),
            scroll_snap_align: alignment,
            size: Size::new(
                PreferredSizeOf::px(S::from_f64(30.0)),
                PreferredSizeOf::px(S::from_f64(30.0)),
            ),
            scroll_margin: margin,
            ..NodeInputOf::default()
        },
    );
    let request = LayoutRootRequestOf::standalone(
        SizingRequestOf::new(
            Size::splat(AxisSizingRequestOf::new(
                AvailableSpaceOf::Indefinite,
                SizingOperation::Normal,
            )),
            OrthogonalFallbackOf::new(
                Size::NONE,
                NearestScrollportOf::Absent,
                Size::splat(OrthogonalFallbackLimitOf::Unbounded),
            ),
        ),
        axes,
        Size::NONE,
    );
    let batch = compute_layout(&tree, 0, request).unwrap();
    let geometry = batch
        .node_layout(0)
        .unwrap()
        .single_box()
        .unwrap()
        .final_output()
        .scroll_geometry
        .unwrap();
    assert_eq!(geometry.scroll_snap_type(), ScrollSnapType::None);
    assert_eq!(geometry.target().snap_align(), alignment);
    geometry.target()
}

fn rect<S: LayoutScalar>(x: f64, y: f64, width: f64, height: f64) -> ScrollRectOf<S> {
    ScrollRectOf::try_new(
        Point::new(S::from_f64(x), S::from_f64(y)),
        Size::new(S::from_f64(width), S::from_f64(height)),
    )
    .unwrap()
}

fn assert_signed_snap_area<S: LayoutScalar>() {
    let border = rect::<S>(-20.0, 35.0, 60.0, 10.0);
    for (edges, expected) in [
        ([3.0, 7.0, 9.0, 11.0], Some(rect(-31.0, 32.0, 78.0, 22.0))),
        ([-2.0, -7.0, 0.0, 5.0], Some(rect(-25.0, 37.0, 58.0, 8.0))),
        ([-6.0, 0.0, -4.0, 0.0], Some(rect(-20.0, 41.0, 60.0, 0.0))),
        ([-7.0, 0.0, -4.0, 0.0], None),
    ] {
        let [top, right, bottom, left] = edges.map(S::from_f64);
        let margin = ScrollMarginOf::try_new(top, right, bottom, left).unwrap();
        assert_eq!(target(margin).snap_area_in_scroller(border), Ok(expected));
    }
}

#[test]
fn snap_area_uses_transformed_bounds_and_signed_outsets_without_snapping() {
    assert_signed_snap_area::<f32>();
    assert_signed_snap_area::<f64>();
}

fn assert_snap_area_numeric_boundary<S: LayoutScalar>(maximum: S) {
    let edge =
        ScrollRectOf::try_new(Point::new(maximum, S::ZERO), Size::new(S::ONE, S::ZERO)).unwrap();
    assert_eq!(
        target(ScrollMarginOf::default()).snap_area_in_scroller(edge),
        Ok(Some(edge)),
    );
    let margin = ScrollMarginOf::try_new(S::ZERO, S::ZERO, S::ZERO, -maximum).unwrap();
    assert_eq!(
        target(margin).snap_area_in_scroller(edge),
        Err(ScrollRectErrorOf::NonFiniteOrigin {
            axis: PhysicalAxis::Horizontal,
            value: S::INFINITY,
        }),
    );
}

#[test]
fn snap_area_preserves_zero_outsets_and_reports_unrepresentable_origin() {
    assert_snap_area_numeric_boundary(f32::MAX);
    assert_snap_area_numeric_boundary(f64::MAX);
}

fn snap_context<S: LayoutScalar>(axis: ScrollSnapAxis) -> ScrollSnapContextOf<S> {
    ScrollSnapContextOf::new(
        rect(10.0, 20.0, 100.0, 50.0),
        PhysicalScrollRangeOf::try_new(
            S::from_f64(-75.0),
            S::from_f64(120.0),
            S::from_f64(-20.0),
            S::from_f64(200.0),
        )
        .unwrap(),
        FlowAxes::new(WritingMode::HorizontalTb, Direction::Ltr),
        axis,
    )
}

fn assert_alignment_direction<S: LayoutScalar>() {
    let target = configured_target(
        ScrollMarginOf::<S>::default(),
        FlowAxes::new(WritingMode::VerticalRl, Direction::Rtl),
        ScrollSnapAlign::new(ScrollSnapAlignValue::End, ScrollSnapAlignValue::Start),
    );
    for (width, height, expected_x, expected_y) in [
        // Equality in the target inline axis retains the container's directions.
        (20.0, 50.0, 50.0, -20.0),
        // Oversized target block extent alone leaves directions unchanged.
        (120.0, 50.0, 50.0, -20.0),
        // Oversized target inline extent flips both physical directions.
        (20.0, 90.0, -30.0, -20.0),
    ] {
        let geometry = target
            .snap_geometry_in_scroller(
                rect(60.0, -30.0, width, height),
                snap_context(ScrollSnapAxis::Both),
            )
            .unwrap()
            .unwrap();
        assert_eq!(
            geometry.aligned_offset(PhysicalAxis::Horizontal),
            Some(S::from_f64(expected_x))
        );
        assert_eq!(
            geometry.aligned_offset(PhysicalAxis::Vertical),
            Some(S::from_f64(expected_y))
        );
    }

    let block_only = target
        .snap_geometry_in_scroller(
            rect(60.0, 120.0, 20.0, 40.0),
            snap_context(ScrollSnapAxis::Block),
        )
        .unwrap()
        .unwrap();
    assert_eq!(block_only.aligned_offset(PhysicalAxis::Horizontal), None);
    assert_eq!(
        block_only.aligned_offset(PhysicalAxis::Vertical),
        Some(S::from_f64(90.0))
    );
}

#[test]
fn snap_offsets_keep_container_axis_roles_and_use_target_edges_when_oversized() {
    assert_alignment_direction::<f32>();
    assert_alignment_direction::<f64>();
}

fn assert_snap_coverage_and_visibility<S: LayoutScalar>() {
    let target = configured_target(
        ScrollMarginOf::<S>::default(),
        FlowAxes::new(WritingMode::HorizontalTb, Direction::Rtl),
        ScrollSnapAlign::new(ScrollSnapAlignValue::End, ScrollSnapAlignValue::Start),
    );
    let geometry = target
        .snap_geometry_in_scroller(
            rect(-40.0, 100.0, 120.0, 20.0),
            snap_context(ScrollSnapAxis::Both),
        )
        .unwrap()
        .unwrap();
    let axis = PhysicalAxis::Horizontal;
    assert_eq!(geometry.aligned_offset(axis), Some(S::from_f64(-30.0)));
    assert_eq!(
        geometry.aligned_offset(PhysicalAxis::Vertical),
        Some(S::from_f64(50.0))
    );

    for (previous, subsequent, expected) in [
        (Some(-75.0), Some(25.0), None),
        (Some(-75.0), Some(50.0), Some((-50.0, -30.0))),
        (Some(-40.0), Some(100.0), Some((-40.0, -30.0))),
        (None, Some(-40.0), Some((-50.0, -40.0))),
        (Some(-75.0), Some(-60.0), None),
    ] {
        let covering = geometry
            .covering_offsets(axis, previous.map(S::from_f64), subsequent.map(S::from_f64))
            .unwrap()
            .map(|range| (range.minimum().to_f64(), range.maximum().to_f64()));
        assert_eq!(covering, expected);
    }
    assert_eq!(
        geometry.covering_offsets(PhysicalAxis::Vertical, None, None),
        Ok(None)
    );

    assert_eq!(
        geometry.covering_offsets(axis, None, Some(S::INFINITY)),
        Err(ScrollCoordinateErrorOf::NonFinitePhysicalRangeMaximum {
            axis,
            value: S::INFINITY,
        }),
    );
    assert_eq!(
        geometry.covering_offsets(axis, Some(S::ONE), Some(S::ZERO)),
        Err(ScrollCoordinateErrorOf::InvertedPhysicalRange {
            axis,
            minimum: S::ONE,
            maximum: S::ZERO,
        }),
    );

    for (offset_y, expected) in [(50.0, true), (100.0, true), (101.0, false)] {
        let offset =
            PhysicalScrollOffsetOf::try_new(S::from_f64(-30.0), S::from_f64(offset_y)).unwrap();
        assert_eq!(geometry.is_visible_at(offset), expected);
    }
}

#[test]
fn snap_covering_offsets_use_adjacent_gap_and_visibility_uses_both_axes() {
    assert_snap_coverage_and_visibility::<f32>();
    assert_snap_coverage_and_visibility::<f64>();
}

fn assert_unreachable_snap_offset<S: LayoutScalar>() {
    let target = configured_target(
        ScrollMarginOf::<S>::default(),
        FlowAxes::new(WritingMode::HorizontalTb, Direction::Ltr),
        ScrollSnapAlign::new(ScrollSnapAlignValue::None, ScrollSnapAlignValue::Center),
    );
    let geometry = target
        .snap_geometry_in_scroller(
            rect(500.0, 35.0, 120.0, 40.0),
            snap_context(ScrollSnapAxis::Both),
        )
        .unwrap()
        .unwrap();
    assert_eq!(
        geometry.aligned_offset(PhysicalAxis::Horizontal),
        Some(S::from_f64(120.0))
    );
    assert_eq!(geometry.aligned_offset(PhysicalAxis::Vertical), None);
    assert_eq!(
        geometry.covering_offsets(PhysicalAxis::Horizontal, None, None),
        Ok(None)
    );
}

#[test]
fn snap_alignment_clamps_unreachable_offsets_without_creating_coverage() {
    assert_unreachable_snap_offset::<f32>();
    assert_unreachable_snap_offset::<f64>();
}
