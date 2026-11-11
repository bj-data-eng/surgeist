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

fn own_ratio_tree<S: LayoutScalar>(block: PreferredSizeOf<S>) -> PublicLayoutTreeOf<S> {
    PublicLayoutTreeOf::<S>::new()
        .children(0, [1])
        .children(1, [])
        .style(
            0,
            NodeInputOf {
                display: Display::Flex,
                size: Size::new(PreferredSizeOf::MAX_CONTENT, block),
                aspect_ratio: AspectRatioOf::new(n(2.0)),
                ..NodeInputOf::default()
            },
        )
        .style(1, NodeInputOf::default())
        .measure(1, Size::new(n(65.0), n(10.0)))
}

fn nonautomatic_preferred_sizes<S: LayoutScalar>() {
    let tree = own_ratio_tree::<S>(PreferredSizeOf::px(n(10.0)));
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

#[test]
fn preferred_main_max_content_preserves_unresolved_own_ratio_cross_refusal() {
    let tree = own_ratio_tree::<f64>(PreferredSizeOf::AUTO);
    let flow = FlowAxes::new(WritingMode::HorizontalTb, Direction::Ltr);
    let error = compute_layout(&tree, 0, request(flow, Some(300.0), None))
        .expect_err("ratio-dependent automatic cross sizing requires its own settlement");
    assert!(
        matches!(error.kind(), LayoutErrorKindOf::UnsupportedCapability(
        LayoutUnsupportedCapability::SizingBehavior(behavior)
    ) if behavior.property() == SizingProperty::Preferred
        && behavior.behavior() == SizingBehavior::MaxContent
        && behavior.algorithm() == SizingAlgorithm::Flex
        && behavior.axis() == PhysicalAxis::Horizontal),
        "{error:?}"
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
