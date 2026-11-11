//! Preferred intrinsic inline sizing uses Flex contributions before Normal contents.
use super::*;
use crate::test_support::readback::{self, Phase};
use crate::test_support::requests::NativeRootFrameOf;

fn n<S: LayoutScalar>(value: f64) -> S {
    S::from_f64(value)
}

fn physical<T>(flow: FlowAxes, inline: T, block: T) -> Size<T> {
    flow.physical_size(crate::geometry::LogicalSizeOf::new(inline, block))
}

fn request<S: LayoutScalar>(
    flow: FlowAxes,
    offer: Option<f64>,
    known_inline: Option<f64>,
) -> LayoutRootRequestOf<S> {
    let fallback = OrthogonalFallbackOf::new(
        Size::splat(Some(NonNegativeFiniteOf::new(n(800.0)).unwrap())),
        NearestScrollportOf::Absent,
        Size::splat(OrthogonalFallbackLimitOf::Unbounded),
    );
    let axes = physical(
        flow,
        AxisSizingRequestOf::new(
            offer.map_or(AvailableSpaceOf::Indefinite, |offer| {
                AvailableSpaceOf::Definite(NonNegativeFiniteOf::new(n(offer)).unwrap())
            }),
            SizingOperation::Normal,
        ),
        AxisSizingRequestOf::new(AvailableSpaceOf::Indefinite, SizingOperation::Normal),
    );
    LayoutRootRequestOf::flex_item(
        SizingRequestOf::new(axes, fallback),
        FlexItemRootContextOf::new(
            flow,
            Size::NONE,
            physical(
                flow,
                known_inline.map(|size| NonNegativeFiniteOf::new(n(size)).unwrap()),
                None,
            ),
        ),
    )
}

fn contributions<S: LayoutScalar>() {
    for mode in [
        WritingMode::HorizontalTb,
        WritingMode::VerticalRl,
        WritingMode::VerticalLr,
        WritingMode::SidewaysRl,
        WritingMode::SidewaysLr,
    ] {
        for direction in [Direction::Ltr, Direction::Rtl] {
            let flow = FlowAxes::new(mode, direction);
            for flex_direction in [FlexDirection::Row, FlexDirection::RowReverse] {
                for wrap in [FlexWrap::NoWrap, FlexWrap::Wrap] {
                    let tree = PublicLayoutTreeOf::<S>::new()
                        .children(0, [1, 2])
                        .children(1, [])
                        .children(2, [])
                        .style(
                            0,
                            NodeInputOf {
                                display: Display::Flex,
                                writing_mode: mode,
                                direction,
                                flex_direction,
                                flex_wrap: wrap,
                                size: Size::new(
                                    PreferredSizeOf::MAX_CONTENT,
                                    PreferredSizeOf::MAX_CONTENT,
                                ),
                                gap: physical(flow, LengthOf::px(n(5.0)), LengthOf::ZERO),
                                align_items: Some(AlignItems::Start),
                                ..NodeInputOf::default()
                            },
                        )
                        .style(
                            1,
                            NodeInputOf {
                                writing_mode: mode,
                                min_size: Size::new(MinSizeOf::ZERO, MinSizeOf::ZERO),
                                ..NodeInputOf::default()
                            },
                        )
                        .style(
                            2,
                            NodeInputOf {
                                writing_mode: mode,
                                min_size: Size::new(MinSizeOf::ZERO, MinSizeOf::ZERO),
                                ..NodeInputOf::default()
                            },
                        )
                        .measure(1, physical(flow, n(40.0), n(10.0)))
                        .measure(2, physical(flow, n(20.0), n(10.0)));
                    for offer in [None, Some(5.0), Some(300.0)] {
                        for (known, expected) in [(None, 65.0), (Some(90.0), 90.0)] {
                            let batch = compute_layout(&tree, 0, request(flow, offer, known))
                                .unwrap_or_else(|error| {
                                    panic!("preferred max-content {mode:?}/{direction:?}/{flex_direction:?}/{wrap:?}, offer {offer:?}, known {known:?}: {error:?}")
                                });
                            for phase in [Phase::Unrounded, Phase::Final] {
                                for (node, inline) in [(0, expected), (1, 40.0), (2, 20.0)] {
                                    let output = readback::single_box_output(&batch, node, phase);
                                    assert_eq!(output.size, physical(flow, n(inline), n(10.0)));
                                    if node != 0 {
                                        let backwards = (mode == WritingMode::SidewaysLr)
                                            ^ (direction == Direction::Rtl)
                                            ^ (flex_direction == FlexDirection::RowReverse);
                                        let start = if node == 1 { 0.0 } else { 45.0 };
                                        let coordinate = if backwards {
                                            expected - start - inline
                                        } else {
                                            start
                                        };
                                        let location = if mode == WritingMode::HorizontalTb {
                                            Point::new(n(coordinate), S::ZERO)
                                        } else {
                                            Point::new(S::ZERO, n(coordinate))
                                        };
                                        assert_eq!(
                                            output.location, location,
                                            "{mode:?}/{direction:?}/{flex_direction:?}, node {node}"
                                        );
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
    }
}

#[test]
fn preferred_main_max_content_uses_contributions_and_preserves_known_allocations() {
    contributions::<f32>();
    contributions::<f64>();
}

fn percentage_tree<S: LayoutScalar>(
    direction: Direction,
    box_sizing: BoxSizing,
) -> PublicLayoutTreeOf<S> {
    let style = |display, width, height| NodeInputOf {
        display,
        direction,
        box_sizing,
        size: Size::new(width, height),
        align_items: Some(AlignItems::Start),
        align_content: Some(AlignContent::Start),
        justify_content: Some(AlignContent::Start),
        ..NodeInputOf::default()
    };
    let root = style(
        Display::Flex,
        PreferredSizeOf::MAX_CONTENT,
        PreferredSizeOf::MAX_CONTENT,
    );
    let mut grid = style(
        Display::Grid,
        PreferredSizeOf::percent(n(1.0)),
        PreferredSizeOf::px(n(40.0)),
    );
    grid.justify_items = Some(AlignItems::Center);
    let child = style(
        Display::Flex,
        PreferredSizeOf::AUTO,
        PreferredSizeOf::percent(n(1.0)),
    );
    let mut ratio = child.clone();
    ratio.aspect_ratio = AspectRatioOf::new(n(1.0));
    PublicLayoutTreeOf::<S>::new()
        .children(0, [1])
        .children(1, [2])
        .children(2, [3])
        .children(3, [])
        .style(0, root)
        .style(1, grid)
        .style(2, child)
        .style(3, ratio)
}

fn percentage_ratio<S: LayoutScalar>() {
    for direction in [Direction::Ltr, Direction::Rtl] {
        for box_sizing in [BoxSizing::BorderBox, BoxSizing::ContentBox] {
            let tree = percentage_tree::<S>(direction, box_sizing);
            let frame = NativeRootFrameOf::new(
                FlowAxes::new(WritingMode::HorizontalTb, Direction::Ltr),
                Size::splat(Some(NonNegativeFiniteOf::new(n(800.0)).unwrap())),
                OrthogonalFallbackOf::new(
                    Size::splat(Some(NonNegativeFiniteOf::new(n(800.0)).unwrap())),
                    NearestScrollportOf::Absent,
                    Size::splat(OrthogonalFallbackLimitOf::Unbounded),
                ),
            );
            let root_request = frame.request(Size::splat(AxisSizingRequestOf::new(
                AvailableSpaceOf::Definite(NonNegativeFiniteOf::new(n(800.0)).unwrap()),
                SizingOperation::Normal,
            )));
            let batch = compute_layout(&tree, 0, root_request).unwrap_or_else(|error| {
                panic!("original percentage Grid {direction:?}/{box_sizing:?}: {error:?}")
            });
            for phase in [Phase::Unrounded, Phase::Final] {
                for node in 0..4 {
                    let output = readback::single_box_output(&batch, node, phase);
                    assert_eq!(output.size, Size::splat(n(40.0)), "node {node}");
                    assert_eq!(output.location, Point::ZERO, "node {node}");
                }
            }
        }
    }
}

#[test]
fn preferred_main_max_content_finishes_original_percentage_grid_and_ratio() {
    percentage_ratio::<f32>();
    percentage_ratio::<f64>();
}

fn bounds<S: LayoutScalar>() {
    let flow = FlowAxes::new(WritingMode::HorizontalTb, Direction::Ltr);
    for box_sizing in [BoxSizing::ContentBox, BoxSizing::BorderBox] {
        // Two 1px edges surround 65px of natural content. Quantitative bounds
        // target the declared sizing box; intrinsic preferred size does not.
        let adjustment = if box_sizing == BoxSizing::ContentBox {
            2.0
        } else {
            0.0
        };
        for (minimum, maximum, expected) in [
            (0.0, None, 67.0),
            (0.0, Some(50.0), 50.0 + adjustment),
            (80.0, Some(50.0), 80.0 + adjustment),
        ] {
            let tree = PublicLayoutTreeOf::<S>::new()
                .children(0, [1])
                .children(1, [])
                .style(
                    0,
                    NodeInputOf {
                        display: Display::Flex,
                        box_sizing,
                        size: Size::new(PreferredSizeOf::MAX_CONTENT, PreferredSizeOf::MAX_CONTENT),
                        min_size: Size::new(MinSizeOf::px(n(minimum)), MinSizeOf::ZERO),
                        max_size: Size::new(
                            maximum.map_or(MaxSizeOf::NONE, |value| MaxSizeOf::px(n(value))),
                            MaxSizeOf::NONE,
                        ),
                        padding: Edges::all(LengthOf::px(n(1.0))),
                        align_items: Some(AlignItems::Start),
                        ..NodeInputOf::default()
                    },
                )
                .style(
                    1,
                    NodeInputOf {
                        min_size: Size::new(MinSizeOf::ZERO, MinSizeOf::ZERO),
                        ..NodeInputOf::default()
                    },
                )
                .measure(1, Size::new(n(65.0), n(10.0)));
            let batch = compute_layout(&tree, 0, request(flow, Some(300.0), None)).unwrap_or_else(
                |error| {
                    panic!("intrinsic main bound {minimum}/{maximum:?}/{box_sizing:?}: {error:?}")
                },
            );
            for phase in [Phase::Unrounded, Phase::Final] {
                assert_eq!(
                    readback::single_box_output(&batch, 0, phase).size,
                    Size::new(n(expected), n(12.0))
                );
            }
        }
    }
}

#[test]
fn preferred_main_max_content_applies_sizing_box_bounds_and_the_minimum_floor() {
    bounds::<f32>();
    bounds::<f64>();
}

fn percentage_request_phases<S: LayoutScalar>() {
    for (operation, sizing_mode, run_mode) in [
        (
            SizingOperation::MaxContent,
            SizingMode::ContentSize,
            RunMode::ComputeSize,
        ),
        (
            SizingOperation::MinContent,
            SizingMode::ContentSize,
            RunMode::ComputeSize,
        ),
        (
            SizingOperation::Normal,
            SizingMode::InherentSize,
            RunMode::ComputeSize,
        ),
        (
            SizingOperation::Normal,
            SizingMode::InherentSize,
            RunMode::PerformLayout,
        ),
    ] {
        let source = percentage_tree::<S>(Direction::Ltr, BoxSizing::BorderBox);
        let mut tree = OracleTreeOf::<S>::new();
        for node in 0..4 {
            tree = tree
                .children(node, Traverse::children(&source, node))
                .style(node, source.node_input(node).clone());
        }
        let flow = FlowAxes::new(WritingMode::HorizontalTb, Direction::Ltr);
        let frame = Size::splat(Some(NonNegativeFiniteOf::new(n(800.0)).unwrap()));
        let sizing_request = SizingRequestOf::new(
            Size::new(
                AxisSizingRequestOf::new(
                    AvailableSpaceOf::Definite(NonNegativeFiniteOf::new(n(800.0)).unwrap()),
                    operation,
                ),
                AxisSizingRequestOf::new(AvailableSpaceOf::Indefinite, SizingOperation::Normal),
            ),
            OrthogonalFallbackOf::new(
                frame,
                NearestScrollportOf::Absent,
                Size::splat(OrthogonalFallbackLimitOf::Unbounded),
            ),
        );
        let input = ComputeInputOf::for_child(
            run_mode,
            sizing_mode,
            RequestedAxis::Both,
            Size::NONE,
            frame.map(|size| size.map(NonNegativeFiniteOf::get)),
            ContainingLayoutContext::new(flow, ParentFormattingContext::NoParent),
            sizing_request,
        );
        let output = tree
            .borrow_compute()
            .compute_child(0, input)
            .unwrap_or_else(|error| {
                panic!("{operation:?}/{sizing_mode:?}/{run_mode:?}: {error:?}")
            });
        assert_eq!(output.size, Size::splat(n(40.0)));
        assert_eq!(tree.inputs(0).first(), Some(&input));
        let grid = tree.inputs(1).last().expect("actual Grid query");
        if operation == SizingOperation::Normal {
            assert_eq!(
                grid.sizing_request().axes().width.operation(),
                SizingOperation::Normal
            );
            assert_eq!(
                grid.parent().width,
                Some(n(40.0)),
                "accepted Normal inline basis"
            );
        } else {
            assert_eq!(
                grid.parent().width,
                None,
                "formal measurement cannot donate its finite result as a parent basis"
            );
        }
        assert_eq!(
            grid.parent().height,
            None,
            "content-derived Flex height remains indefinite"
        );
        for node in 0..4 {
            assert_eq!(
                tree.layout(node).is_some(),
                run_mode == RunMode::PerformLayout
            );
            assert_eq!(
                tree.final_layout(node),
                None,
                "internal computation cannot finalize publication"
            );
        }
        eprintln!("qualified {operation:?}/{sizing_mode:?}/{run_mode:?} with Grid query {grid:?}");
    }
}

#[test]
fn preferred_main_max_content_keeps_intrinsic_measurement_and_normal_contents_separate() {
    percentage_request_phases::<f32>();
    percentage_request_phases::<f64>();
}

fn nonautomatic_preferred_sizes<S: LayoutScalar>() {
    let tree = PublicLayoutTreeOf::<S>::new()
        .children(0, [1])
        .children(1, [])
        .style(
            0,
            NodeInputOf {
                display: Display::Flex,
                size: Size::new(PreferredSizeOf::MAX_CONTENT, PreferredSizeOf::px(n(10.0))),
                aspect_ratio: AspectRatioOf::new(n(2.0)),
                ..NodeInputOf::default()
            },
        )
        .style(1, NodeInputOf::default())
        .measure(1, Size::new(n(65.0), n(10.0)));
    let flow = FlowAxes::new(WritingMode::HorizontalTb, Direction::Ltr);
    let batch = compute_layout(&tree, 0, request(flow, Some(300.0), None))
        .expect("the ratio does not replace either nonautomatic preferred size");
    for phase in [Phase::Unrounded, Phase::Final] {
        assert_eq!(
            readback::single_box_output(&batch, 0, phase).size,
            Size::new(n(65.0), n(10.0))
        );
    }
}

#[test]
fn preferred_main_max_content_is_not_backfilled_from_the_cross_size_and_ratio() {
    nonautomatic_preferred_sizes::<f32>();
    nonautomatic_preferred_sizes::<f64>();
}
