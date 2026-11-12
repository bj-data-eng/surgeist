use crate::{
    AvailableSpaceOf, AxisSizingRequestOf, Direction, Display, FlowAxes, LayoutRootRequestOf,
    LayoutScalar, NearestScrollportOf, NodeInputOf, OrthogonalFallbackLimitOf,
    OrthogonalFallbackOf, PhysicalAxis, Point, PreferredSizeOf, ScrollMarginOf, ScrollRectErrorOf,
    ScrollRectOf, ScrollSnapAlign, ScrollSnapType, ScrollTargetGeometryOf, Size, SizingOperation,
    SizingRequestOf, WritingMode, compute_layout, test_support::layout_tree::PublicLayoutTreeOf,
};

fn target<S: LayoutScalar>(margin: ScrollMarginOf<S>) -> ScrollTargetGeometryOf<S> {
    let tree = PublicLayoutTreeOf::new().style(
        0,
        NodeInputOf {
            display: Display::Block,
            size: Size::new(
                PreferredSizeOf::px(S::from_f64(30.0)),
                PreferredSizeOf::px(S::from_f64(30.0)),
            ),
            scroll_margin: margin,
            ..NodeInputOf::default()
        },
    );
    let axes = FlowAxes::new(WritingMode::HorizontalTb, Direction::Ltr);
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
    assert_eq!(geometry.target().snap_align(), ScrollSnapAlign::default());
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
