//! Preferred intrinsic inline sizing uses Flex contributions before Normal contents.
use super::*;
use crate::test_support::readback::{self, Phase};

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

fn two_item_row<S: LayoutScalar>(
    flow: FlowAxes,
    direction: FlexDirection,
    wrap: FlexWrap,
    maximum: Option<f64>,
) -> PublicLayoutTreeOf<S> {
    let mut tree = PublicLayoutTreeOf::<S>::new().children(0, [1, 2]).style(
        0,
        NodeInputOf {
            display: Display::Flex,
            writing_mode: flow.writing_mode(),
            direction: flow.direction(),
            flex_direction: direction,
            flex_wrap: wrap,
            size: Size::new(PreferredSizeOf::MAX_CONTENT, PreferredSizeOf::MAX_CONTENT),
            max_size: physical(
                flow,
                maximum.map_or(MaxSizeOf::NONE, |value| MaxSizeOf::px(n(value))),
                MaxSizeOf::NONE,
            ),
            gap: physical(flow, LengthOf::px(n(5.0)), LengthOf::ZERO),
            align_items: Some(AlignItems::Start),
            ..NodeInputOf::default()
        },
    );
    for (node, width) in [(1, 40.0), (2, 20.0)] {
        tree = tree
            .children(node, [])
            .style(
                node,
                NodeInputOf {
                    writing_mode: flow.writing_mode(),
                    min_size: Size::new(MinSizeOf::ZERO, MinSizeOf::ZERO),
                    ..NodeInputOf::default()
                },
            )
            .measure(node, physical(flow, n(width), n(10.0)));
    }
    tree
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
                    let tree = two_item_row::<S>(flow, flex_direction, wrap, None);
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
    let mut root = style(
        Display::Flex,
        PreferredSizeOf::MAX_CONTENT,
        PreferredSizeOf::MAX_CONTENT,
    );
    root.position = Position::Absolute;
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
            let containing = Size::new(
                NonNegativeFiniteOf::new(n(800.0)).unwrap(),
                NonNegativeFiniteOf::new(n(600.0)).unwrap(),
            );
            let fallback = OrthogonalFallbackOf::new(
                containing.map(Some),
                NearestScrollportOf::Absent,
                Size::splat(OrthogonalFallbackLimitOf::Unbounded),
            );
            let context = RootLocalPositionedContextOf::horizontal_auto_insets(
                containing,
                Direction::Ltr,
                S::ZERO,
                fallback,
            )
            .unwrap();
            let root_request = LayoutRootRequestOf::positioned_root_local(context).unwrap();
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
        let grid_queries = tree.inputs(1);
        let (discovery_operation, discovery_space) = match operation {
            SizingOperation::Normal => (SizingOperation::MaxContent, AvailableSpaceOf::Indefinite),
            formal => (formal, AvailableSpaceOf::Definite(frame.width.unwrap())),
        };
        let discover = grid_queries
            .iter()
            .position(|query| {
                let inline = query.sizing_request().axes().width;
                query.run_mode() == RunMode::ComputeSize
                    && query.sizing_mode() == SizingMode::ContentSize
                    && inline.operation() == discovery_operation
                    && inline.space() == discovery_space
                    && query.known().width.is_none()
                    && query.parent().width.is_none()
            })
            .unwrap_or_else(|| {
                panic!("{operation:?}/{run_mode:?}: no contribution query in {grid_queries:?}")
            });
        if operation == SizingOperation::Normal {
            let complete = grid_queries
                .iter()
                .position(|query| {
                    let inline = query.sizing_request().axes().width;
                    query.sizing_mode() == SizingMode::InherentSize
                        && inline.operation() == SizingOperation::Normal
                        && inline.space()
                            == AvailableSpaceOf::Definite(
                                NonNegativeFiniteOf::new(n(40.0)).unwrap(),
                            )
                        && query.parent().width == Some(n(40.0))
                })
                .unwrap_or_else(|| {
                    panic!("{run_mode:?}: no Normal contents query in {grid_queries:?}")
                });
            assert!(
                discover < complete,
                "Normal completion follows intrinsic discovery"
            );
        } else {
            assert!(
                grid_queries
                    .iter()
                    .all(|query| query.parent().width.is_none()),
                "formal measurement cannot donate a finite result as a parent basis"
            );
        }
        assert!(
            grid_queries
                .iter()
                .all(|query| query.parent().height.is_none()),
            "content-derived Flex height remains indefinite"
        );
        if run_mode == RunMode::ComputeSize {
            for node in 0..4 {
                assert_eq!(tree.layout(node), None, "size query cannot stage geometry");
            }
        }
        assert_eq!(
            output.scroll_geometry.is_some(),
            run_mode == RunMode::PerformLayout
        );
        for node in 0..4 {
            assert_eq!(
                tree.final_layout(node),
                None,
                "internal computation cannot finalize publication"
            );
        }
        eprintln!(
            "qualified {operation:?}/{sizing_mode:?}/{run_mode:?}, contribution query {discover}"
        );
    }
}

#[test]
fn preferred_main_max_content_keeps_intrinsic_measurement_and_normal_contents_separate() {
    percentage_request_phases::<f32>();
    percentage_request_phases::<f64>();
}

fn own_ratio_tree<S: LayoutScalar>(
    flow: FlowAxes,
    block: PreferredSizeOf<S>,
) -> PublicLayoutTreeOf<S> {
    let ratio = if flow.writing_mode() == WritingMode::HorizontalTb {
        2.0
    } else {
        0.5
    };
    PublicLayoutTreeOf::<S>::new()
        .children(0, [1])
        .children(1, [])
        .style(
            0,
            NodeInputOf {
                display: Display::Flex,
                writing_mode: flow.writing_mode(),
                direction: flow.direction(),
                size: physical(flow, PreferredSizeOf::MAX_CONTENT, block),
                aspect_ratio: AspectRatioOf::new(n(ratio)),
                align_items: Some(AlignItems::Start),
                ..NodeInputOf::default()
            },
        )
        .style(
            1,
            NodeInputOf {
                writing_mode: flow.writing_mode(),
                direction: flow.direction(),
                ..NodeInputOf::default()
            },
        )
        .measure(1, physical(flow, n(65.0), n(10.0)))
}

fn nonautomatic_preferred_sizes<S: LayoutScalar>() {
    let flow = FlowAxes::new(WritingMode::HorizontalTb, Direction::Ltr);
    let tree = own_ratio_tree::<S>(flow, PreferredSizeOf::px(n(10.0)));
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

fn own_ratio_cross_from_preferred_inline<S: LayoutScalar>() {
    for mode in [WritingMode::HorizontalTb, WritingMode::VerticalRl] {
        for direction in [Direction::Ltr, Direction::Rtl] {
            let flow = FlowAxes::new(mode, direction);
            let tree = own_ratio_tree::<S>(flow, PreferredSizeOf::AUTO);
            let batch = compute_layout(&tree, 0, request(flow, Some(300.0), None))
                .unwrap_or_else(|error| panic!("ratio cross sizing in {flow:?}: {error:?}"));
            for (phase, block) in [(Phase::Unrounded, 32.5), (Phase::Final, 33.0)] {
                assert_eq!(
                    readback::single_box_output(&batch, 0, phase).size,
                    physical(flow, n(65.0), n(block)),
                );
                assert_eq!(
                    readback::single_box_output(&batch, 1, phase).size,
                    physical(flow, n(65.0), n(10.0)),
                );
            }
        }
    }
}

#[test]
fn preferred_main_max_content_settles_automatic_own_ratio_cross_f32() {
    own_ratio_cross_from_preferred_inline::<f32>();
}

#[test]
fn preferred_main_max_content_settles_automatic_own_ratio_cross_f64() {
    own_ratio_cross_from_preferred_inline::<f64>();
}

fn own_ratio_cross_minimum_controls<S: LayoutScalar>() {
    fn measure<S: LayoutScalar>(input: LeafMeasureInputOf<S>) -> Size<S> {
        let known = input.known_content_size();
        let cross = if input.content_sizing_request().axes().height.operation()
            == SizingOperation::MinContent
        {
            60.0
        } else {
            80.0
        };
        Size::new(
            known.width.unwrap_or(n(65.0)),
            known.height.unwrap_or(n(cross)),
        )
    }

    let flow = FlowAxes::new(WritingMode::HorizontalTb, Direction::Ltr);
    let scroll = ComputedOverflow::try_new(Overflow::Hidden, Overflow::Hidden).unwrap();
    for (label, minimum, maximum, overflow, expected_cross) in [
        (
            "content minimum",
            MinSizeOf::AUTO,
            MaxSizeOf::NONE,
            ComputedOverflow::VISIBLE,
            60.0,
        ),
        (
            "capped content minimum",
            MinSizeOf::AUTO,
            MaxSizeOf::px(n(45.0)),
            ComputedOverflow::VISIBLE,
            45.0,
        ),
        (
            "explicit zero minimum",
            MinSizeOf::ZERO,
            MaxSizeOf::NONE,
            ComputedOverflow::VISIBLE,
            32.5,
        ),
        (
            "scroll container",
            MinSizeOf::AUTO,
            MaxSizeOf::NONE,
            scroll,
            32.5,
        ),
    ] {
        let tree = own_ratio_tree::<S>(flow, PreferredSizeOf::AUTO);
        let mut root = tree.node_input(0).clone();
        root.min_size.height = minimum;
        root.max_size.height = maximum;
        root.overflow = overflow;
        let tree = tree.style(0, root).measure_with(1, measure::<S>);
        let batch = compute_layout(&tree, 0, request(flow, None, None)).unwrap();
        assert_eq!(
            readback::single_box_output(&batch, 0, Phase::Unrounded).size,
            Size::new(n(65.0), n(expected_cross)),
            "{label}",
        );
    }
}

fn own_ratio_cross_sizing_box_controls<S: LayoutScalar>() {
    let flow = FlowAxes::new(WritingMode::HorizontalTb, Direction::Ltr);
    for (box_sizing, cross) in [(BoxSizing::ContentBox, 42.5), (BoxSizing::BorderBox, 37.5)] {
        let tree = own_ratio_tree::<S>(flow, PreferredSizeOf::AUTO);
        let mut root = tree.node_input(0).clone();
        root.box_sizing = box_sizing;
        root.align_items = Some(AlignItems::End);
        root.padding = Edges::all(LengthOf::px(n(3.0)));
        root.border = Edges::all(LengthOf::px(n(2.0)));
        let batch = compute_layout(&tree.style(0, root), 0, request(flow, None, None)).unwrap();
        assert_eq!(
            readback::single_box_output(&batch, 0, Phase::Unrounded).size,
            Size::new(n(75.0), n(cross)),
            "{box_sizing:?}",
        );
        assert_eq!(
            readback::single_box_output(&batch, 1, Phase::Unrounded).location,
            Point::new(n(5.0), n(cross - 15.0)),
        );
    }
}

#[test]
fn preferred_main_max_content_applies_own_ratio_cross_minimum() {
    own_ratio_cross_minimum_controls::<f32>();
    own_ratio_cross_minimum_controls::<f64>();
}

#[test]
fn preferred_main_max_content_transfers_own_ratio_in_sizing_box() {
    own_ratio_cross_sizing_box_controls::<f32>();
    own_ratio_cross_sizing_box_controls::<f64>();
}

fn own_ratio_cross_request_authority<S: LayoutScalar>() {
    let flow = FlowAxes::new(WritingMode::HorizontalTb, Direction::Ltr);
    let source = own_ratio_tree::<S>(flow, PreferredSizeOf::AUTO);
    for run_mode in [RunMode::ComputeSize, RunMode::PerformLayout] {
        let mut tree = OracleTreeOf::<S>::new()
            .children(0, [1])
            .children(1, [])
            .style(0, source.node_input(0).clone())
            .style(1, source.node_input(1).clone())
            .measure(
                1,
                ComputeOutputOf::from_outer_size(Size::new(n(65.0), n(10.0))),
            );
        let input = ComputeInputOf::for_child(
            run_mode,
            SizingMode::InherentSize,
            RequestedAxis::Both,
            Size::NONE,
            Size::NONE,
            ContainingLayoutContext::new(flow, ParentFormattingContext::NoParent),
            request::<S>(flow, None, None).sizing_request(),
        );
        let output = tree.borrow_compute().compute_child(0, input).unwrap();
        assert_eq!(output.size, Size::new(n(65.0), n(32.5)), "{run_mode:?}");
        let queries = tree.inputs(1);
        assert!(
            queries.iter().any(|query| {
                query.run_mode() == RunMode::ComputeSize
                    && query.sizing_request().axes().height.operation()
                        == SizingOperation::MinContent
                    && query.known().width == Some(n(65.0))
                    && query.known().height.is_none()
            }),
            "{run_mode:?}: {queries:?}"
        );
        assert!(
            queries.iter().all(|query| query.parent().height.is_none()),
            "{run_mode:?}: ratio geometry cannot supply a cross percentage basis: {queries:?}",
        );
        if run_mode == RunMode::ComputeSize {
            assert!(tree.layout(1).is_none());
        }
    }
}

#[test]
fn preferred_main_max_content_ratio_cross_preserves_request_authority() {
    own_ratio_cross_request_authority::<f32>();
    own_ratio_cross_request_authority::<f64>();
}

#[test]
fn preferred_main_max_content_ratio_cross_accepts_percentage_with_a_definite_flex_basis() {
    let flow = FlowAxes::new(WritingMode::HorizontalTb, Direction::Ltr);
    let tree = own_ratio_tree::<f64>(flow, PreferredSizeOf::AUTO);
    let mut leaf = tree.node_input(1).clone();
    leaf.size.height = PreferredSizeOf::percent(1.0);
    leaf.flex_basis = FlexBasisOf::px(65.0);
    let tree = tree.style(1, leaf);
    let batch = compute_layout(&tree, 0, request(flow, None, None)).unwrap();
    assert_eq!(
        readback::single_box_output(&batch, 0, Phase::Unrounded).size,
        Size::new(65.0, 32.5),
    );
    assert_eq!(
        readback::single_box_output(&batch, 1, Phase::Unrounded).size,
        Size::new(65.0, 10.0),
    );
}

#[test]
fn preferred_main_max_content_ratio_cross_transfers_definite_imposed_inline() {
    let flow = FlowAxes::new(WritingMode::HorizontalTb, Direction::Ltr);
    let tree = own_ratio_tree::<f64>(flow, PreferredSizeOf::AUTO);
    let mut leaf = tree.node_input(1).clone();
    leaf.size.height = PreferredSizeOf::percent(0.5);
    let tree = tree.style(1, leaf);
    let batch = compute_layout(&tree, 0, request(flow, None, Some(80.0))).unwrap();
    assert_eq!(
        readback::single_box_output(&batch, 0, Phase::Unrounded).size,
        Size::new(80.0, 40.0),
    );
    assert_eq!(
        readback::single_box_output(&batch, 1, Phase::Unrounded)
            .size
            .height,
        20.0,
    );
}

fn bounded_wrapping<S: LayoutScalar>() {
    let flow = FlowAxes::new(WritingMode::HorizontalTb, Direction::Ltr);
    let tree = two_item_row::<S>(flow, FlexDirection::Row, FlexWrap::Wrap, Some(62.0));
    let batch = compute_layout(&tree, 0, request(flow, Some(300.0), None)).unwrap();
    // Intrinsic width is 40 + 5 + 20. The maximum then accepts 62px;
    // only the contents pass wraps at that width.
    for phase in [Phase::Unrounded, Phase::Final] {
        assert_eq!(
            readback::single_box_output(&batch, 0, phase).size,
            Size::new(n(62.0), n(20.0))
        );
        let second = readback::single_box_output(&batch, 2, phase);
        assert_eq!(second.size, Size::new(n(20.0), n(10.0)));
        assert_eq!(second.location, Point::new(S::ZERO, n(10.0)));
    }
}

#[test]
fn preferred_main_max_content_wraps_only_after_the_maximum_is_applied() {
    bounded_wrapping::<f32>();
    bounded_wrapping::<f64>();
}

fn imposed_main_with_bound<S: LayoutScalar>() {
    let flow = FlowAxes::new(WritingMode::HorizontalTb, Direction::Ltr);
    let tree = two_item_row::<S>(
        flow,
        FlexDirection::RowReverse,
        FlexWrap::NoWrap,
        Some(80.0),
    );
    let batch = compute_layout(&tree, 0, request(flow, Some(300.0), Some(90.0))).unwrap();
    for phase in [Phase::Unrounded, Phase::Final] {
        for (node, width, x) in [(0, 90.0, 0.0), (1, 40.0, 50.0), (2, 20.0, 25.0)] {
            let output = readback::single_box_output(&batch, node, phase);
            assert_eq!(output.size, Size::new(n(width), n(10.0)), "node {node}");
            assert_eq!(output.location, Point::new(n(x), S::ZERO), "node {node}");
        }
    }
}

#[test]
fn preferred_main_max_content_uses_imposed_size_for_placement_despite_authored_maximum() {
    imposed_main_with_bound::<f32>();
    imposed_main_with_bound::<f64>();
}

fn block_parent<S: LayoutScalar>(
    flex: NodeInputOf<S>,
    mut leaf: NodeInputOf<S>,
    natural_width: f64,
) -> CompletedLayoutBatchOf<u32, S> {
    leaf.display = Display::Block;
    let tree = PublicLayoutTreeOf::<S>::new()
        .children(0, [1])
        .children(1, [2])
        .children(2, [])
        .style(
            0,
            NodeInputOf {
                display: Display::Block,
                size: Size::new(PreferredSizeOf::px(n(100.0)), PreferredSizeOf::AUTO),
                ..NodeInputOf::default()
            },
        )
        .style(1, flex)
        .style(2, leaf)
        .measure(2, Size::new(n(natural_width), n(10.0)));
    let flow = FlowAxes::new(WritingMode::HorizontalTb, Direction::Ltr);
    compute_layout(&tree, 0, request(flow, Some(300.0), None)).unwrap()
}

fn block_preserves_operation<S: LayoutScalar>() {
    let flex = NodeInputOf::<S> {
        display: Display::Flex,
        size: Size::new(PreferredSizeOf::MAX_CONTENT, PreferredSizeOf::AUTO),
        align_items: Some(AlignItems::Start),
        ..NodeInputOf::default()
    };
    let leaf = NodeInputOf {
        size: Size::new(
            PreferredSizeOf::percent(n(0.5)),
            PreferredSizeOf::px(n(10.0)),
        ),
        min_size: Size::new(MinSizeOf::ZERO, MinSizeOf::ZERO),
        ..NodeInputOf::default()
    };
    let batch = block_parent(flex, leaf, 40.0);
    for phase in [Phase::Unrounded, Phase::Final] {
        for (node, width) in [(0, 100.0), (1, 40.0), (2, 20.0)] {
            let output = readback::single_box_output(&batch, node, phase);
            assert_eq!(output.size, Size::new(n(width), n(10.0)), "node {node}");
            assert_eq!(output.location, Point::ZERO, "node {node}");
        }
    }
}

#[test]
fn preferred_main_max_content_under_normal_block_finishes_percentages() {
    block_preserves_operation::<f32>();
    block_preserves_operation::<f64>();
}

fn block_preserves_preference<S: LayoutScalar>() {
    let flex = NodeInputOf::<S> {
        display: Display::Flex,
        size: Size::new(PreferredSizeOf::MAX_CONTENT, PreferredSizeOf::px(n(10.0))),
        aspect_ratio: AspectRatioOf::new(n(2.0)),
        ..NodeInputOf::default()
    };
    let batch = block_parent(flex, NodeInputOf::default(), 65.0);
    for phase in [Phase::Unrounded, Phase::Final] {
        assert_eq!(
            readback::single_box_output(&batch, 1, phase).size,
            Size::new(n(65.0), n(10.0))
        );
    }
}

#[test]
fn preferred_main_max_content_under_block_is_not_backfilled_by_ratio() {
    block_preserves_preference::<f32>();
    block_preserves_preference::<f64>();
}

fn discovery_precedes_leaf_perform<S: LayoutScalar>() {
    let tree = PublicLayoutTreeOf::<S>::new()
        .children(0, [1])
        .children(1, [])
        .style(
            0,
            NodeInputOf {
                display: Display::Flex,
                size: Size::new(PreferredSizeOf::MAX_CONTENT, PreferredSizeOf::AUTO),
                align_items: Some(AlignItems::Start),
                ..NodeInputOf::default()
            },
        )
        .style(
            1,
            NodeInputOf {
                display: Display::Block,
                size: Size::new(PreferredSizeOf::percent(n(0.5)), PreferredSizeOf::AUTO),
                flex_basis: FlexBasisOf::px(n(40.0)),
                min_size: Size::new(MinSizeOf::ZERO, MinSizeOf::ZERO),
                ..NodeInputOf::default()
            },
        )
        .measure(1, Size::new(n(40.0), n(10.0)));
    let flow = FlowAxes::new(WritingMode::HorizontalTb, Direction::Ltr);
    let batch = compute_layout(&tree, 0, request(flow, Some(300.0), None))
        .expect("intrinsic discovery does not perform a leaf with an unresolved percentage");
    for phase in [Phase::Unrounded, Phase::Final] {
        for node in 0..2 {
            let output = readback::single_box_output(&batch, node, phase);
            assert_eq!(output.size, Size::new(n(40.0), n(10.0)), "node {node}");
            assert_eq!(output.location, Point::ZERO, "node {node}");
        }
    }
}

#[test]
fn preferred_main_max_content_measures_cyclic_leaf_before_performing_layout() {
    discovery_precedes_leaf_perform::<f32>();
    discovery_precedes_leaf_perform::<f64>();
}

fn formal_inline_request<S: LayoutScalar>(operation: SizingOperation) -> LayoutRootRequestOf<S> {
    let flow = FlowAxes::new(WritingMode::HorizontalTb, Direction::Ltr);
    let sizing = request::<S>(flow, Some(300.0), None).sizing_request();
    let mut axes = sizing.axes();
    axes.width = axes.width.with_operation(operation);
    LayoutRootRequestOf::flex_item(
        sizing.with_axes(axes),
        FlexItemRootContextOf::new(flow, Size::NONE, Size::NONE),
    )
}

#[test]
fn preferred_main_max_content_preserves_formal_auto_wrapping_bound() {
    let flow = FlowAxes::new(WritingMode::HorizontalTb, Direction::Ltr);
    let tree = two_item_row::<f64>(flow, FlexDirection::Row, FlexWrap::Wrap, Some(62.0));
    let mut root = tree.node_input(0).clone();
    root.size = Size::new(PreferredSizeOf::AUTO, PreferredSizeOf::AUTO);
    let tree = tree.style(0, root);
    let batch =
        compute_layout(&tree, 0, formal_inline_request(SizingOperation::MaxContent)).unwrap();
    // The formal Auto path collects lines against its existing authored bound.
    // These items occupy two lines, with no within-line gap in the contribution.
    for phase in [Phase::Unrounded, Phase::Final] {
        assert_eq!(
            readback::single_box_output(&batch, 0, phase).size,
            Size::new(60.0, 20.0)
        );
        assert_eq!(
            readback::single_box_output(&batch, 2, phase).location,
            Point::new(0.0, 10.0)
        );
    }
}

#[test]
fn preferred_main_max_content_preserves_formal_inherent_child_operation() {
    let source = percentage_tree::<f64>(Direction::Ltr, BoxSizing::BorderBox);
    let mut tree = OracleTreeOf::<f64>::new();
    for node in 0..4 {
        tree = tree
            .children(node, Traverse::children(&source, node))
            .style(node, source.node_input(node).clone());
    }
    let flow = FlowAxes::new(WritingMode::HorizontalTb, Direction::Ltr);
    let input = ComputeInputOf::for_child(
        RunMode::ComputeSize,
        SizingMode::InherentSize,
        RequestedAxis::Both,
        Size::NONE,
        Size::NONE,
        ContainingLayoutContext::new(flow, ParentFormattingContext::Flex),
        formal_inline_request(SizingOperation::MinContent).sizing_request(),
    );
    let output = tree.borrow_compute().compute_child(0, input).unwrap();
    assert_eq!(output.size, Size::new(40.0, 40.0));
    let queries = tree.inputs(1);
    assert!(
        queries.iter().any(|query| {
            query.sizing_mode() == SizingMode::InherentSize
                && query.sizing_request().axes().width.operation() == SizingOperation::MinContent
                && query.parent().width.is_none()
        }),
        "the authored preference measurement lost the formal child operation: {queries:?}"
    );
    assert!(queries.iter().all(|query| query.parent().width.is_none()));
    for node in 0..4 {
        assert_eq!(tree.layout(node), None);
        assert_eq!(tree.final_layout(node), None);
    }
}

#[test]
fn preferred_main_max_content_under_grid_is_not_backfilled_by_ratio() {
    let tree = PublicLayoutTreeOf::<f64>::new()
        .children(0, [1])
        .children(1, [2])
        .children(2, [])
        .style(
            0,
            NodeInputOf {
                display: Display::Grid,
                size: Size::new(PreferredSizeOf::px(100.0), PreferredSizeOf::AUTO),
                grid_template_columns: vec![TrackComponentOf::px(100.0)],
                ..NodeInputOf::default()
            },
        )
        .style(
            1,
            NodeInputOf {
                display: Display::Flex,
                size: Size::new(PreferredSizeOf::MAX_CONTENT, PreferredSizeOf::px(10.0)),
                aspect_ratio: AspectRatioOf::new(2.0),
                justify_self: Some(AlignItems::Start),
                align_self: Some(AlignItems::Start),
                ..NodeInputOf::default()
            },
        )
        .style(
            2,
            NodeInputOf {
                display: Display::Block,
                ..NodeInputOf::default()
            },
        )
        .measure(2, Size::new(65.0, 10.0));
    let flow = FlowAxes::new(WritingMode::HorizontalTb, Direction::Ltr);
    let batch = compute_layout(&tree, 0, request(flow, Some(300.0), None)).unwrap();
    for phase in [Phase::Unrounded, Phase::Final] {
        assert_eq!(
            readback::single_box_output(&batch, 1, phase).size,
            Size::new(65.0, 10.0)
        );
    }
}

#[test]
fn preferred_main_max_content_inside_flex_item_reaches_the_child_owner() {
    check_preferred_main_max_content_inside_flex_item(FlexBasisOf::AUTO, 20.0);
    check_preferred_main_max_content_inside_flex_item(FlexBasisOf::px(40.0), 40.0);
}

fn check_preferred_main_max_content_inside_flex_item(basis: FlexBasisOf<f64>, leaf_width: f64) {
    let tree = PublicLayoutTreeOf::<f64>::new()
        .children(0, [1])
        .children(1, [2])
        .children(2, [])
        .style(
            0,
            NodeInputOf {
                display: Display::Flex,
                size: Size::new(PreferredSizeOf::px(100.0), PreferredSizeOf::AUTO),
                align_items: Some(AlignItems::Start),
                ..NodeInputOf::default()
            },
        )
        .style(
            1,
            NodeInputOf {
                display: Display::Flex,
                size: Size::new(PreferredSizeOf::MAX_CONTENT, PreferredSizeOf::AUTO),
                align_items: Some(AlignItems::Start),
                ..NodeInputOf::default()
            },
        )
        .style(
            2,
            NodeInputOf {
                display: Display::Block,
                size: Size::new(PreferredSizeOf::percent(0.5), PreferredSizeOf::AUTO),
                flex_basis: basis,
                min_size: Size::new(MinSizeOf::ZERO, MinSizeOf::ZERO),
                ..NodeInputOf::default()
            },
        )
        .measure(2, Size::new(40.0, 10.0));
    let flow = FlowAxes::new(WritingMode::HorizontalTb, Direction::Ltr);
    let batch = compute_layout(&tree, 0, request(flow, Some(300.0), None)).unwrap();
    for phase in [Phase::Unrounded, Phase::Final] {
        for (node, width) in [(0, 100.0), (1, 40.0), (2, leaf_width)] {
            assert_eq!(
                readback::single_box_output(&batch, node, phase).size,
                Size::new(width, 10.0),
                "node {node}"
            );
        }
    }
}

#[test]
fn preferred_main_max_content_positioned_child_reaches_the_child_owner() {
    let flow = FlowAxes::new(WritingMode::HorizontalTb, Direction::Ltr);
    let mut refused = Vec::new();
    for display in [Display::Block, Display::Flex, Display::Grid] {
        let tree = PublicLayoutTreeOf::<f64>::new()
            .children(0, [1])
            .children(1, [2])
            .children(2, [])
            .style(
                0,
                NodeInputOf {
                    display,
                    size: Size::new(PreferredSizeOf::px(50.0), PreferredSizeOf::px(100.0)),
                    ..NodeInputOf::default()
                },
            )
            .style(
                1,
                NodeInputOf {
                    display: Display::Flex,
                    position: Position::Absolute,
                    size: Size::new(PreferredSizeOf::MAX_CONTENT, PreferredSizeOf::px(10.0)),
                    align_items: Some(AlignItems::Start),
                    ..NodeInputOf::default()
                },
            )
            .style(
                2,
                NodeInputOf {
                    display: Display::Block,
                    ..NodeInputOf::default()
                },
            )
            .measure_with(2, |input| {
                let inline = input.content_sizing_request().axes().width;
                let width = match inline.operation() {
                    SizingOperation::MinContent => 25.0,
                    SizingOperation::MaxContent => 65.0,
                    SizingOperation::Normal => inline
                        .space()
                        .definite_size()
                        .map_or(65.0, |size| size.get().clamp(25.0, 65.0)),
                };
                Size::new(width, 10.0)
            });
        match compute_layout(&tree, 0, request(flow, Some(300.0), None)) {
            Ok(batch) => {
                for phase in [Phase::Unrounded, Phase::Final] {
                    let output = readback::single_box_output(&batch, 1, phase);
                    assert_eq!(output.size, Size::new(65.0, 10.0), "{display:?}");
                    assert_eq!(output.location, Point::ZERO, "{display:?}");
                }
            }
            Err(error) => refused.push(format!("{display:?}: {error:?}")),
        }
    }
    assert!(refused.is_empty(), "{refused:?}");
}

fn nested_ratio_flex<S: LayoutScalar>(
    outer_inline: PreferredSizeOf<S>,
    inner_block: PreferredSizeOf<S>,
    ratio: f64,
) -> PublicLayoutTreeOf<S> {
    PublicLayoutTreeOf::new()
        .children(0, [1])
        .children(1, [2])
        .children(2, [])
        .style(
            0,
            NodeInputOf {
                display: Display::Flex,
                size: Size::new(outer_inline, PreferredSizeOf::AUTO),
                align_items: Some(AlignItems::Start),
                ..NodeInputOf::default()
            },
        )
        .style(
            1,
            NodeInputOf {
                display: Display::Flex,
                size: Size::new(PreferredSizeOf::MAX_CONTENT, inner_block),
                align_items: Some(AlignItems::Start),
                aspect_ratio: AspectRatioOf::new(n(ratio)),
                min_size: Size::new(MinSizeOf::ZERO, MinSizeOf::AUTO),
                flex_grow: crate::FlexGrowOf::try_new(n(1.0)).unwrap(),
                ..NodeInputOf::default()
            },
        )
        .style(2, NodeInputOf::default())
        .measure(2, Size::new(n(65.0), n(10.0)))
}

#[test]
fn preferred_main_max_content_settles_nested_ratio_cross_from_flexed_inline() {
    let tree = nested_ratio_flex::<f64>(PreferredSizeOf::px(100.0), PreferredSizeOf::AUTO, 2.0);
    let flow = FlowAxes::new(WritingMode::HorizontalTb, Direction::Ltr);
    let batch = compute_layout(&tree, 0, request(flow, Some(300.0), None)).unwrap();
    for phase in [Phase::Unrounded, Phase::Final] {
        for node in [0, 1] {
            assert_eq!(
                readback::single_box_output(&batch, node, phase).size,
                Size::new(100.0, 50.0),
            );
        }
    }
}

#[test]
fn preferred_main_max_content_nested_ratio_cross_respects_main_definiteness() {
    let flow = FlowAxes::new(WritingMode::HorizontalTb, Direction::Ltr);
    for (outer_inline, inline, cross, percentage_cross) in [
        (PreferredSizeOf::MAX_CONTENT, 65.0, 32.5, 10.0),
        (PreferredSizeOf::px(100.0), 100.0, 50.0, 50.0),
    ] {
        let tree = nested_ratio_flex::<f64>(outer_inline, PreferredSizeOf::AUTO, 2.0);
        let mut leaf = tree.node_input(2).clone();
        leaf.size.height = PreferredSizeOf::percent(1.0);
        let tree = tree.style(2, leaf);
        let batch = compute_layout(&tree, 0, request(flow, Some(300.0), None)).unwrap();
        assert_eq!(
            readback::single_box_output(&batch, 1, Phase::Unrounded).size,
            Size::new(inline, cross),
        );
        assert_eq!(
            readback::single_box_output(&batch, 2, Phase::Unrounded)
                .size
                .height,
            percentage_cross,
        );
    }
}

#[test]
fn preferred_main_max_content_nested_ratio_cross_cannot_feed_back_into_flex_basis() {
    fn measure(input: LeafMeasureInputOf<f64>) -> Size<f64> {
        let known = input.known_content_size();
        let available_cross = input.content_sizing_request().axes().height.space();
        let main = if known.height.is_some() || available_cross.definite_size().is_some() {
            90.0
        } else {
            65.0
        };
        Size::new(known.width.unwrap_or(main), known.height.unwrap_or(10.0))
    }

    let flow = FlowAxes::new(WritingMode::HorizontalTb, Direction::Ltr);
    let tree = nested_ratio_flex::<f64>(PreferredSizeOf::px(200.0), PreferredSizeOf::AUTO, 2.0);
    let mut inner = tree.node_input(1).clone();
    inner.flex_grow = FlexGrowOf::ZERO;
    inner.align_items = Some(AlignItems::Stretch);
    let tree = tree.style(1, inner).measure_with(2, measure);
    let batch = compute_layout(&tree, 0, request(flow, Some(300.0), None)).unwrap();
    assert_eq!(
        readback::single_box_output(&batch, 1, Phase::Unrounded).size,
        Size::new(65.0, 32.5),
    );
}

#[test]
fn preferred_main_max_content_keeps_nested_ratio_out_of_intrinsic_contribution() {
    let tree = nested_ratio_flex::<f64>(
        PreferredSizeOf::MAX_CONTENT,
        PreferredSizeOf::px(10.0),
        10.0,
    );
    let flow = FlowAxes::new(WritingMode::HorizontalTb, Direction::Ltr);
    let batch = compute_layout(&tree, 0, request(flow, Some(300.0), None)).unwrap();
    for phase in [Phase::Unrounded, Phase::Final] {
        for node in 0..3 {
            assert_eq!(
                readback::single_box_output(&batch, node, phase).size,
                Size::new(65.0, 10.0),
                "node {node}"
            );
        }
    }
}

#[test]
fn preferred_main_max_content_remeasures_nested_cross_at_the_accepted_inline_size() {
    let tree = PublicLayoutTreeOf::<f64>::new()
        .children(0, [1])
        .children(1, [2])
        .children(2, [])
        .style(
            0,
            NodeInputOf {
                display: Display::Flex,
                size: Size::new(PreferredSizeOf::px(100.0), PreferredSizeOf::AUTO),
                align_items: Some(AlignItems::Start),
                ..NodeInputOf::default()
            },
        )
        .style(
            1,
            NodeInputOf {
                display: Display::Flex,
                size: Size::new(PreferredSizeOf::MAX_CONTENT, PreferredSizeOf::AUTO),
                align_items: Some(AlignItems::Start),
                ..NodeInputOf::default()
            },
        )
        .style(
            2,
            NodeInputOf {
                size: Size::new(PreferredSizeOf::percent(0.5), PreferredSizeOf::AUTO),
                min_size: Size::new(MinSizeOf::ZERO, MinSizeOf::ZERO),
                ..NodeInputOf::default()
            },
        )
        .measure_with(2, |input| {
            let width = input.known_content_size().width.unwrap_or(40.0);
            let height = if width <= 20.0 { 30.0 } else { 10.0 };
            Size::new(width, height)
        });
    let flow = FlowAxes::new(WritingMode::HorizontalTb, Direction::Ltr);
    let batch = compute_layout(&tree, 0, request(flow, Some(300.0), None)).unwrap();
    for phase in [Phase::Unrounded, Phase::Final] {
        for (node, width) in [(0, 100.0), (1, 40.0), (2, 20.0)] {
            assert_eq!(
                readback::single_box_output(&batch, node, phase).size,
                Size::new(width, 30.0),
                "node {node}"
            );
        }
    }
}

#[test]
fn preferred_main_max_content_preserves_auto_imposed_size_placement_control() {
    let flow = FlowAxes::new(WritingMode::HorizontalTb, Direction::Ltr);
    let tree = two_item_row::<f64>(
        flow,
        FlexDirection::RowReverse,
        FlexWrap::NoWrap,
        Some(80.0),
    );
    let mut auto = tree.node_input(0).clone();
    auto.size.width = PreferredSizeOf::AUTO;
    let tree = tree.style(0, auto);
    let batch = compute_layout(&tree, 0, request(flow, Some(300.0), Some(90.0))).unwrap();
    for phase in [Phase::Unrounded, Phase::Final] {
        assert_eq!(
            readback::single_box_output(&batch, 0, phase).size.width,
            90.0
        );
        assert_eq!(
            readback::single_box_output(&batch, 1, phase).location.x,
            40.0
        );
        assert_eq!(
            readback::single_box_output(&batch, 2, phase).location.x,
            15.0
        );
    }
}

#[test]
fn preferred_main_max_content_preserves_independent_cross_through_parent_bounds() {
    let flow = FlowAxes::new(WritingMode::HorizontalTb, Direction::Ltr);
    let child = NodeInputOf {
        display: Display::Flex,
        size: Size::new(PreferredSizeOf::MAX_CONTENT, PreferredSizeOf::px(40.0)),
        min_size: Size::new(MinSizeOf::ZERO, MinSizeOf::AUTO),
        max_size: Size::new(MaxSizeOf::px(20.0), MaxSizeOf::NONE),
        aspect_ratio: AspectRatioOf::new(2.0),
        align_self: Some(AlignItems::Start),
        align_items: Some(AlignItems::Start),
        justify_self: Some(AlignItems::Start),
        ..NodeInputOf::default()
    };
    let root = PublicLayoutTreeOf::<f64>::new()
        .children(0, [1])
        .children(1, [])
        .style(0, child.clone())
        .style(1, NodeInputOf::default())
        .measure(1, Size::new(65.0, 10.0));
    let root_batch = compute_layout(&root, 0, request(flow, Some(300.0), None)).unwrap();
    let expected = Size::new(20.0, 40.0);
    for phase in [Phase::Unrounded, Phase::Final] {
        assert_eq!(
            readback::single_box_output(&root_batch, 0, phase).size,
            expected
        );
    }

    let mut differences = Vec::new();
    for display in [Display::Block, Display::Flex, Display::Grid] {
        let tree = PublicLayoutTreeOf::<f64>::new()
            .children(0, [1])
            .children(1, [2])
            .children(2, [])
            .style(
                0,
                NodeInputOf {
                    display,
                    size: Size::new(PreferredSizeOf::px(100.0), PreferredSizeOf::AUTO),
                    align_items: Some(AlignItems::Start),
                    grid_template_columns: if display == Display::Grid {
                        vec![TrackComponentOf::px(100.0)]
                    } else {
                        Vec::new()
                    },
                    ..NodeInputOf::default()
                },
            )
            .style(1, child.clone())
            .style(2, NodeInputOf::default())
            .measure(2, Size::new(65.0, 10.0));
        let batch = compute_layout(&tree, 0, request(flow, Some(300.0), None)).unwrap();
        for phase in [Phase::Unrounded, Phase::Final] {
            let observed = readback::single_box_output(&batch, 1, phase).size;
            if observed != expected {
                differences.push((display, observed));
            }
        }
    }
    assert!(
        differences.is_empty(),
        "expected {expected:?}; {differences:?}"
    );
}

#[test]
fn preferred_main_max_content_preserves_the_existing_intrinsic_cross_preference() {
    let original = percentage_tree::<f64>(Direction::Ltr, BoxSizing::BorderBox);
    let flow = FlowAxes::new(WritingMode::HorizontalTb, Direction::Ltr);
    let mut refused = Vec::new();
    for (display, position) in [
        (Display::Flex, Position::Relative),
        (Display::Block, Position::Absolute),
        (Display::Flex, Position::Absolute),
        (Display::Grid, Position::Absolute),
    ] {
        let mut flex = original.node_input(0).clone();
        flex.position = position;
        let tree = PublicLayoutTreeOf::<f64>::new()
            .children(0, [1])
            .children(1, [2])
            .children(2, [3])
            .children(3, [4])
            .children(4, [])
            .style(
                0,
                NodeInputOf {
                    display,
                    size: Size::new(PreferredSizeOf::px(800.0), PreferredSizeOf::px(600.0)),
                    align_items: Some(AlignItems::Start),
                    ..NodeInputOf::default()
                },
            )
            .style(1, flex)
            .style(2, original.node_input(1).clone())
            .style(3, original.node_input(2).clone())
            .style(4, original.node_input(3).clone());
        match compute_layout(&tree, 0, request(flow, Some(800.0), None)) {
            Ok(batch) => {
                for phase in [Phase::Unrounded, Phase::Final] {
                    for node in 1..5 {
                        let output = readback::single_box_output(&batch, node, phase);
                        assert_eq!(
                            output.size,
                            Size::new(40.0, 40.0),
                            "{display:?}/{position:?} node {node}"
                        );
                    }
                }
            }
            Err(error) => refused.push((display, position, error)),
        }
    }
    assert!(refused.is_empty(), "{refused:?}");
}

#[test]
fn preferred_main_max_content_uses_the_authored_cross_operation_in_nested_probes() {
    fn measure(input: LeafMeasureInputOf<f64>) -> Size<f64> {
        let requests = input.content_sizing_request().axes();
        let known = input.known_content_size();
        Size::new(
            known.width.unwrap_or(
                if requests.height.operation() == SizingOperation::MaxContent {
                    70.0
                } else {
                    40.0
                },
            ),
            known.height.unwrap_or(10.0),
        )
    }

    let flow = FlowAxes::new(WritingMode::HorizontalTb, Direction::Ltr);
    let child = NodeInputOf {
        display: Display::Flex,
        size: Size::new(PreferredSizeOf::MAX_CONTENT, PreferredSizeOf::MAX_CONTENT),
        min_size: Size::new(MinSizeOf::ZERO, MinSizeOf::AUTO),
        align_items: Some(AlignItems::Start),
        ..NodeInputOf::default()
    };
    let leaf = NodeInputOf {
        min_size: Size::new(MinSizeOf::ZERO, MinSizeOf::AUTO),
        ..NodeInputOf::default()
    };
    let root = PublicLayoutTreeOf::<f64>::new()
        .children(0, [1])
        .children(1, [])
        .style(0, child.clone())
        .style(1, leaf.clone())
        .measure_with(1, measure);
    let expected = Size::new(70.0, 10.0);
    let batch = compute_layout(&root, 0, request(flow, Some(300.0), None)).unwrap();
    for phase in [Phase::Unrounded, Phase::Final] {
        assert_eq!(readback::single_box_output(&batch, 0, phase).size, expected);
    }

    let mut differences = Vec::new();
    for basis in [FlexBasisOf::AUTO, FlexBasisOf::CONTENT] {
        let mut child = child.clone();
        child.flex_basis = basis.clone();
        for (outer_inline, outer_width) in [
            (PreferredSizeOf::px(100.0), 100.0),
            (PreferredSizeOf::MAX_CONTENT, 70.0),
        ] {
            let tree = PublicLayoutTreeOf::<f64>::new()
                .children(0, [1])
                .children(1, [2])
                .children(2, [])
                .style(
                    0,
                    NodeInputOf {
                        display: Display::Flex,
                        size: Size::new(outer_inline, PreferredSizeOf::AUTO),
                        align_items: Some(AlignItems::Start),
                        ..NodeInputOf::default()
                    },
                )
                .style(1, child.clone())
                .style(2, leaf.clone())
                .measure_with(2, measure);
            let batch = compute_layout(&tree, 0, request(flow, Some(300.0), None)).unwrap();
            for phase in [Phase::Unrounded, Phase::Final] {
                let outer = readback::single_box_output(&batch, 0, phase).size;
                let inner = readback::single_box_output(&batch, 1, phase).size;
                if outer.width != outer_width || inner != expected {
                    differences.push((basis.clone(), outer_width, outer, inner));
                }
            }
        }
    }
    assert!(
        differences.is_empty(),
        "expected inner {expected:?}; {differences:?}"
    );
}

#[test]
fn preferred_main_max_content_basis_uses_the_bounded_authored_cross_size() {
    fn measure(input: LeafMeasureInputOf<f64>) -> Size<f64> {
        let known = input.known_content_size();
        let cross = known
            .height
            .or_else(|| {
                input
                    .content_sizing_request()
                    .axes()
                    .height
                    .space()
                    .definite_size()
                    .map(NonNegativeFiniteOf::get)
            })
            .unwrap_or(10.0);
        Size::new(known.width.unwrap_or(cross * 2.0), 10.0)
    }

    let flow = FlowAxes::new(WritingMode::HorizontalTb, Direction::Ltr);
    let child = NodeInputOf {
        display: Display::Flex,
        size: Size::new(PreferredSizeOf::MAX_CONTENT, PreferredSizeOf::px(40.0)),
        max_size: Size::new(MaxSizeOf::NONE, MaxSizeOf::px(20.0)),
        min_size: Size::new(MinSizeOf::ZERO, MinSizeOf::AUTO),
        align_items: Some(AlignItems::Start),
        ..NodeInputOf::default()
    };
    let leaf = NodeInputOf {
        min_size: Size::new(MinSizeOf::ZERO, MinSizeOf::AUTO),
        ..NodeInputOf::default()
    };
    let root = PublicLayoutTreeOf::<f64>::new()
        .children(0, [1])
        .children(1, [])
        .style(0, child.clone())
        .style(1, leaf.clone())
        .measure_with(1, measure);
    let nested = PublicLayoutTreeOf::<f64>::new()
        .children(0, [1])
        .children(1, [2])
        .children(2, [])
        .style(
            0,
            NodeInputOf {
                display: Display::Flex,
                size: Size::new(PreferredSizeOf::px(100.0), PreferredSizeOf::AUTO),
                align_items: Some(AlignItems::Start),
                ..NodeInputOf::default()
            },
        )
        .style(1, child)
        .style(2, leaf)
        .measure_with(2, measure);
    let mut percentage_parent = nested.node_input(0).clone();
    percentage_parent.size.height = PreferredSizeOf::px(50.0);
    let mut percentage_child = nested.node_input(1).clone();
    percentage_child.size.height = PreferredSizeOf::percent(0.8);
    let nested_percentage = nested
        .clone()
        .style(0, percentage_parent)
        .style(1, percentage_child);
    for (tree, child_node) in [(root, 0), (nested, 1), (nested_percentage, 1)] {
        let batch = compute_layout(&tree, 0, request(flow, Some(300.0), None)).unwrap();
        for phase in [Phase::Unrounded, Phase::Final] {
            assert_eq!(
                readback::single_box_output(&batch, child_node, phase).size,
                Size::new(40.0, 20.0),
                "child at node {child_node}"
            );
        }
    }
}

#[test]
fn preferred_main_max_content_ratio_cross_keeps_natural_lane_packing_indefinite() {
    for alignment in [Some(AlignItems::Start), None] {
        assert_natural_lane_ratio_cross_is_indefinite(alignment);
    }
}

fn assert_natural_lane_ratio_cross_is_indefinite(alignment: Option<AlignItems>) {
    let flow = FlowAxes::new(WritingMode::HorizontalTb, Direction::Ltr);
    let tree = nested_ratio_flex::<f64>(PreferredSizeOf::px(100.0), PreferredSizeOf::AUTO, 2.0);
    let mut item = tree.node_input(1).clone();
    item.justify_self = Some(AlignItems::Start);
    item.align_self = alignment;
    item.min_size.height = MinSizeOf::ZERO;
    let mut leaf = tree.node_input(2).clone();
    leaf.size.height = PreferredSizeOf::percent(1.0);
    let tree = tree
        .children(0, [1, 3])
        .children(3, [])
        .style(
            0,
            NodeInputOf {
                display: Display::GridLanes,
                size: Size::new(PreferredSizeOf::px(100.0), PreferredSizeOf::AUTO),
                grid_lanes_flow: GridLanesFlow::new(GridAxisKind::Column, false, false),
                grid_template_columns: vec![TrackComponentOf::px(100.0)],
                ..NodeInputOf::default()
            },
        )
        .style(1, item)
        .style(2, leaf)
        .style(3, NodeInputOf::default())
        .measure(3, Size::new(10.0, 10.0));
    let batch = compute_layout(&tree, 0, request(flow, None, None)).unwrap();
    assert_eq!(
        readback::single_box_output(&batch, 1, Phase::Unrounded).size,
        Size::new(65.0, 32.5),
    );
    assert_eq!(
        readback::single_box_output(&batch, 3, Phase::Unrounded)
            .location
            .y,
        32.5,
    );
    assert_eq!(
        readback::single_box_output(&batch, 2, Phase::Unrounded)
            .size
            .height,
        10.0,
        "stacking alignment {alignment:?}",
    );
}

#[test]
fn preferred_main_max_content_sets_the_extent_used_for_lane_packing() {
    let flow = FlowAxes::new(WritingMode::HorizontalTb, Direction::Ltr);
    let tree = PublicLayoutTreeOf::<f64>::new()
        .children(0, [1, 3])
        .children(1, [2])
        .children(2, [])
        .children(3, [])
        .style(
            0,
            NodeInputOf {
                display: Display::GridLanes,
                size: Size::new(PreferredSizeOf::px(100.0), PreferredSizeOf::AUTO),
                grid_lanes_flow: GridLanesFlow::new(GridAxisKind::Column, false, false),
                grid_template_columns: vec![TrackComponentOf::px(100.0)],
                ..NodeInputOf::default()
            },
        )
        .style(
            1,
            NodeInputOf {
                display: Display::Flex,
                size: Size::new(PreferredSizeOf::MAX_CONTENT, PreferredSizeOf::AUTO),
                align_items: Some(AlignItems::Start),
                ..NodeInputOf::default()
            },
        )
        .style(
            2,
            NodeInputOf {
                size: Size::new(PreferredSizeOf::percent(1.0), PreferredSizeOf::AUTO),
                min_size: Size::new(MinSizeOf::ZERO, MinSizeOf::AUTO),
                ..NodeInputOf::default()
            },
        )
        .style(3, NodeInputOf::default())
        .measure_with(2, |input| {
            let inline = input.known_content_size().width.unwrap_or(40.0);
            Size::new(inline, if inline > 40.0 { 30.0 } else { 10.0 })
        })
        .measure(3, Size::new(10.0, 10.0));
    let batch = compute_layout(&tree, 0, request(flow, Some(300.0), None)).unwrap();
    for phase in [Phase::Unrounded, Phase::Final] {
        assert_eq!(
            readback::single_box_output(&batch, 1, phase).size,
            Size::new(40.0, 10.0)
        );
        assert_eq!(
            readback::single_box_output(&batch, 3, phase).location.y,
            10.0
        );
    }
}

#[test]
fn preferred_main_max_content_lane_packing_uses_bounded_cross_size() {
    let flow = FlowAxes::new(WritingMode::HorizontalTb, Direction::Ltr);
    let mut differences = Vec::new();
    for (height, max_height, ratio, expected) in [
        (PreferredSizeOf::px(80.0), MaxSizeOf::px(20.0), None, 20.0),
        (PreferredSizeOf::AUTO, MaxSizeOf::px(20.0), None, 20.0),
        (
            PreferredSizeOf::AUTO,
            MaxSizeOf::NONE,
            AspectRatioOf::new(2.0),
            100.0,
        ),
    ] {
        let tree = PublicLayoutTreeOf::<f64>::new()
            .children(0, [1, 3])
            .children(1, [2])
            .children(2, [])
            .children(3, [])
            .style(
                0,
                NodeInputOf {
                    display: Display::GridLanes,
                    size: Size::new(PreferredSizeOf::AUTO, PreferredSizeOf::px(100.0)),
                    grid_lanes_flow: GridLanesFlow::new(GridAxisKind::Row, false, false),
                    grid_template_rows: vec![TrackComponentOf::px(100.0)],
                    ..NodeInputOf::default()
                },
            )
            .style(
                1,
                NodeInputOf {
                    display: Display::Flex,
                    size: Size::new(PreferredSizeOf::MAX_CONTENT, height),
                    max_size: Size::new(MaxSizeOf::NONE, max_height),
                    align_self: Some(AlignItems::Stretch),
                    aspect_ratio: ratio,
                    align_items: Some(AlignItems::Start),
                    ..NodeInputOf::default()
                },
            )
            .style(
                2,
                NodeInputOf {
                    size: Size::new(PreferredSizeOf::AUTO, PreferredSizeOf::percent(1.0)),
                    min_size: Size::new(MinSizeOf::ZERO, MinSizeOf::ZERO),
                    ..NodeInputOf::default()
                },
            )
            .style(3, NodeInputOf::default())
            .measure_with(2, |input| {
                let block = input.known_content_size().height.unwrap_or(10.0);
                Size::new(block, block)
            })
            .measure(3, Size::new(10.0, 10.0));
        match compute_layout(&tree, 0, request(flow, Some(300.0), None)) {
            Ok(batch) => {
                for phase in [Phase::Unrounded, Phase::Final] {
                    let sibling_x = readback::single_box_output(&batch, 3, phase).location.x;
                    let flex_size = readback::single_box_output(&batch, 1, phase).size;
                    if sibling_x != expected || flex_size != Size::splat(expected) {
                        differences.push(format!(
                            "expected {expected}, got sibling x {sibling_x}, Flex {flex_size:?}"
                        ));
                    }
                }
            }
            Err(error) => differences.push(format!("expected {expected}, got {error:?}")),
        }
    }
    assert!(differences.is_empty(), "{differences:?}");
}

#[test]
fn preferred_main_max_content_accepts_independent_flex_cross_stretch() {
    let flow = FlowAxes::new(WritingMode::HorizontalTb, Direction::Ltr);
    let mut differences = Vec::new();
    for basis in [FlexBasisOf::px(40.0), FlexBasisOf::CONTENT] {
        let tree = PublicLayoutTreeOf::<f64>::new()
            .children(0, [1])
            .children(1, [2])
            .children(2, [])
            .style(
                0,
                NodeInputOf {
                    display: Display::Flex,
                    size: Size::new(PreferredSizeOf::px(300.0), PreferredSizeOf::px(100.0)),
                    ..NodeInputOf::default()
                },
            )
            .style(
                1,
                NodeInputOf {
                    display: Display::Flex,
                    size: Size::new(PreferredSizeOf::MAX_CONTENT, PreferredSizeOf::AUTO),
                    flex_basis: basis.clone(),
                    min_size: Size::new(MinSizeOf::ZERO, MinSizeOf::ZERO),
                    aspect_ratio: AspectRatioOf::new(2.0),
                    align_self: Some(AlignItems::Stretch),
                    ..NodeInputOf::default()
                },
            )
            .style(2, NodeInputOf::default())
            .measure(2, Size::new(40.0, 10.0));
        match compute_layout(&tree, 0, request(flow, Some(300.0), None)) {
            Ok(batch) => {
                for phase in [Phase::Unrounded, Phase::Final] {
                    let observed = readback::single_box_output(&batch, 1, phase).size;
                    if observed.height != 100.0 {
                        differences.push(format!("{basis:?} produced {observed:?}"));
                    }
                }
            }
            Err(error) => differences.push(format!("{basis:?} produced {error:?}")),
        }
    }
    assert!(differences.is_empty(), "{differences:?}");
}
