use super::*;
use crate::{
    AvailableSpaceOf, AxisSizingRequestOf, ComputedOverflow, Display, Edges, LayoutRootRequestOf,
    LengthOf, NearestScrollportOf, NodeInputOf, NonNegativeFiniteOf, OrthogonalFallbackLimitOf,
    OrthogonalFallbackOf, Overflow, PreferredSizeOf, ScrollbarWidthOf, SizingOperation,
    SizingRequestOf, compute_layout, test_support::layout_tree::PublicLayoutTreeOf,
};

fn request<S: LayoutScalar>(axes: FlowAxes) -> LayoutRootRequestOf<S> {
    let available = NonNegativeFiniteOf::new(S::from_f64(100.0)).unwrap();
    let axis = AxisSizingRequestOf::new(
        AvailableSpaceOf::Definite(available),
        SizingOperation::Normal,
    );
    LayoutRootRequestOf::standalone(
        SizingRequestOf::new(
            Size::splat(axis),
            OrthogonalFallbackOf::new(
                Size::NONE,
                NearestScrollportOf::Absent,
                Size::splat(OrthogonalFallbackLimitOf::Unbounded),
            ),
        ),
        axes,
        Size::NONE,
    )
}

fn accepted_box<S: LayoutScalar>() {
    let n = S::from_f64;
    let px = |value| LengthOf::px(n(value));
    let preferred = |value| PreferredSizeOf::px(n(value));
    let grid = ResizeObserverPixelGrid::try_new(Point::new(-0.5, 0.25), 1.0).unwrap();
    let shifted = ResizeObserverPixelGrid::try_new(Point::new(0.0, 0.25), 1.0).unwrap();
    for mode in [
        WritingMode::HorizontalTb,
        WritingMode::VerticalRl,
        WritingMode::VerticalLr,
        WritingMode::SidewaysRl,
        WritingMode::SidewaysLr,
    ] {
        let tree = PublicLayoutTreeOf::new()
            .style(
                0,
                NodeInputOf {
                    display: Display::Block,
                    writing_mode: mode,
                    size: Size::new(preferred(20.25), preferred(15.75)),
                    padding: Edges::new(px(0.25), px(1.5), px(1.75), px(0.75)),
                    border: Edges::new(px(1.0), px(2.0), px(1.5), px(2.5)),
                    overflow: ComputedOverflow::try_new(Overflow::Hidden, Overflow::Scroll)
                        .unwrap(),
                    scrollbar_width: ScrollbarWidthOf::try_new(n(2.0)).unwrap(),
                    ..NodeInputOf::default()
                },
            )
            .style(
                1,
                NodeInputOf {
                    display: Display::Block,
                    writing_mode: mode,
                    size: Size::new(preferred(40.0), preferred(8.0)),
                    ..NodeInputOf::default()
                },
            )
            .children(0, [1]);
        let batch = compute_layout(&tree, 0, request(FlowAxes::new(mode, Direction::Ltr))).unwrap();
        let view = batch.node_layout(0).unwrap().single_box().unwrap();
        assert_ne!(view.final_output().size, Size::new(n(20.25), n(15.75)));
        assert!(view.unrounded_output().content_size.width >= n(40.0));

        let measurement = view
            .resize_observer_measurement(mode, grid)
            .unwrap()
            .unwrap();
        let moved = view
            .resize_observer_measurement(mode, shifted)
            .unwrap()
            .unwrap();
        let (border, content, pixels) = if mode == WritingMode::HorizontalTb {
            ((20.25, 15.75), (11.5, 11.25), (11, 12))
        } else {
            ((15.75, 20.25), (11.25, 11.5), (12, 11))
        };
        assert_eq!(
            measurement.border_box(),
            ResizeObserverBoxSize {
                inline: n(border.0),
                block: n(border.1)
            },
            "{mode:?}"
        );
        assert_eq!(
            measurement.content_box(),
            ResizeObserverBoxSize {
                inline: n(content.0),
                block: n(content.1)
            },
            "{mode:?}"
        );
        assert_eq!(
            measurement.device_pixel_content_box(),
            ResizeObserverBoxSize {
                inline: pixels.0,
                block: pixels.1
            },
            "{mode:?}"
        );
        assert_eq!(
            moved.device_pixel_content_box(),
            ResizeObserverBoxSize {
                inline: 12,
                block: 12
            },
            "{mode:?}"
        );
        assert_eq!(
            measurement.content_rect(),
            ResizeObserverContentRectOf {
                origin: Point::new(n(0.75), n(0.25)),
                size: Size::new(n(11.5), n(11.25)),
            }
        );
        assert_eq!(moved.content_rect(), measurement.content_rect());
        assert_eq!(moved.border_box(), measurement.border_box());
        assert_eq!(moved.content_box(), measurement.content_box());

        let overflowing = ResizeObserverPixelGrid::try_new(Point::ZERO, f64::MAX).unwrap();
        assert_eq!(
            view.resize_observer_measurement(mode, overflowing),
            Err(
                ResizeObserverMeasurementError::DevicePixelExtentOutOfRange {
                    axis: PhysicalAxis::Horizontal
                }
            ),
        );
    }
}

#[test]
fn accepted_box_reports_unrounded_gutter_adjusted_extents_in_both_lanes() {
    accepted_box::<f32>();
    accepted_box::<f64>();
}

#[test]
fn known_empty_observation_is_available_with_zero_sizes() {
    let tree = PublicLayoutTreeOf::<f64>::new().style(
        0,
        NodeInputOf {
            display: Display::None,
            ..NodeInputOf::default()
        },
    );
    let mode = WritingMode::HorizontalTb;
    let batch = compute_layout(&tree, 0, request(FlowAxes::new(mode, Direction::Ltr))).unwrap();
    assert!(batch.node_layout(0).unwrap().non_box().is_some());
    let unavailable = batch.node_layout(1).and_then(|layout| layout.single_box());
    assert!(unavailable.is_none());

    let empty = ResizeObserverMeasurementOf::<f64>::empty();
    assert_eq!(
        empty.border_box(),
        ResizeObserverBoxSize {
            inline: 0.0,
            block: 0.0
        }
    );
    assert_eq!(
        empty.content_box(),
        ResizeObserverBoxSize {
            inline: 0.0,
            block: 0.0
        }
    );
    assert_eq!(
        empty.device_pixel_content_box(),
        ResizeObserverBoxSize {
            inline: 0,
            block: 0
        }
    );
    assert_eq!(
        empty.content_rect(),
        ResizeObserverContentRectOf {
            origin: Point::ZERO,
            size: Size::ZERO
        }
    );
}

#[test]
fn pixel_grid_rejects_nonfinite_origin_and_invalid_scale() {
    let horizontal = ResizeObserverPixelGrid::try_new(Point::new(f64::NAN, 0.0), 1.0);
    assert!(
        matches!(horizontal, Err(ResizeObserverMeasurementError::NonFiniteContentOrigin {
        axis: PhysicalAxis::Horizontal, coordinate
    }) if coordinate.is_nan())
    );
    assert_eq!(
        ResizeObserverPixelGrid::try_new(Point::new(0.0, f64::NEG_INFINITY), 1.0),
        Err(ResizeObserverMeasurementError::NonFiniteContentOrigin {
            axis: PhysicalAxis::Vertical,
            coordinate: f64::NEG_INFINITY,
        }),
    );
    for scale in [0.0, -0.0, -1.0, f64::INFINITY, f64::NAN] {
        assert!(matches!(
            ResizeObserverPixelGrid::try_new(Point::ZERO, scale),
            Err(ResizeObserverMeasurementError::InvalidScale { scale: got }) if got.to_bits() == scale.to_bits()
        ));
    }
}

#[test]
fn pixel_edges_resolve_signed_ties_and_subpixel_phase() {
    let extent = pixel_edge::pixel_extent;
    let below_half = f64::from_bits(0.5_f64.to_bits() - 1);
    let above_half = f64::from_bits(0.5_f64.to_bits() + 1);
    assert_eq!(extent(0.0, below_half, 1.0), Some(0));
    assert_eq!(extent(0.0, 0.5, 1.0), Some(1));
    assert_eq!(extent(0.0, above_half, 1.0), Some(1));
    assert_eq!(extent(-above_half, 0.5, 1.0), Some(1));
    assert_eq!(extent(-0.5, 0.5, 1.0), Some(0));
    assert_eq!(extent(-below_half, 0.5, 1.0), Some(0));
    assert_eq!(extent(-2.0_f64.powi(-120), 0.5, 1.0), Some(0));
    assert_eq!(extent(0.0, 5.0, 0.1), Some(1));
}

#[test]
fn pixel_extent_range_is_independent_of_absolute_origin_range() {
    let extent = pixel_edge::pixel_extent;
    let word = 2.0_f64.powi(32);
    assert_eq!(extent(0.0, word - 1.0, word + 1.0), Some(u64::MAX));
    assert_eq!(extent(0.0, word, word), None);
    assert_eq!(
        extent(2.0_f64.powi(1023), 2.0_f64.powi(-1000), 2.0_f64.powi(1000)),
        Some(1),
    );
    assert_eq!(extent(-f64::MAX, 0.0, f64::MAX), Some(0));
}
