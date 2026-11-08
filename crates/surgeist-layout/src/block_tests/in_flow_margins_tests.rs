use super::fixtures::{
    BlockTree, CalcBlockTree, PublicBlockTree, all_writing_mode_directions, computed_overflow,
    fri05_c03_block_union_content_size, fri06_atomic_participation, lp, public_final_output,
    scalar_value,
};
use super::*;

fn scalar_percentage<S: LayoutScalar>(
    absolute_px: f64,
    percent_fraction: f64,
) -> LengthPercentageOf<S> {
    LengthPercentageOf::from_coefficients(scalar_value(absolute_px), scalar_value(percent_fraction))
        .expect("test coefficients are finite")
}

#[test]
fn parent_context_gates_only_block_boundary_collapse_in_both_scalar_lanes() {
    fn assert_lane<S: LayoutScalar>()
    where
        crate::test_support::layout_tree::OracleTreeOf<S>: Traverse<Node = u32, Scalar = S>,
    {
        let flow_axes = FlowAxes::new(WritingMode::HorizontalTb, Direction::Ltr);
        for (parent_context, expected_collapse) in [
            (ParentFormattingContext::BlockFlow, true),
            (ParentFormattingContext::Flex, false),
            (ParentFormattingContext::Grid, false),
            (ParentFormattingContext::NoParent, false),
        ] {
            let mut child_output =
                ComputeOutputOf::from_outer_size(Size::new(S::from_f64(40.0), S::ZERO));
            child_output.block_margin_collapse = PhysicalBlockMarginCollapseOf::from_block_flow(
                flow_axes,
                CollapsibleMarginOf::from_margin(S::from_f64(3.0)),
                CollapsibleMarginOf::from_margin(S::from_f64(5.0)),
                true,
            );
            let mut tree = crate::test_support::layout_tree::OracleTreeOf::<S>::new()
                .children(0, [1])
                .children(1, [])
                .style(
                    0,
                    NodeInputOf {
                        display: Display::Block,
                        size: Size::new(
                            PreferredSizeOf::px(S::from_f64(40.0)),
                            PreferredSizeOf::AUTO,
                        ),
                        ..NodeInputOf::default()
                    },
                )
                .style(
                    1,
                    NodeInputOf {
                        display: Display::Block,
                        margin: Edges::new(
                            LengthAutoOf::px(S::from_f64(3.0)),
                            LengthAutoOf::ZERO,
                            LengthAutoOf::px(S::from_f64(5.0)),
                            LengthAutoOf::ZERO,
                        ),
                        ..NodeInputOf::default()
                    },
                )
                .measure(1, child_output);
            let output = crate::compute_block(
                &mut tree.borrow_compute(),
                0,
                ComputeInputOf::for_child(
                    RunMode::PerformLayout,
                    SizingMode::InherentSize,
                    RequestedAxis::Both,
                    Size::NONE,
                    Size::new(Some(S::from_f64(100.0)), Some(S::from_f64(100.0))),
                    ContainingLayoutContext::new(flow_axes, parent_context),
                    SizingRequestOf::new(
                        Size::new(
                            AxisSizingRequestOf::new(
                                AvailableSpaceOf::Definite(
                                    NonNegativeFiniteOf::new(S::from_f64(100.0)).unwrap(),
                                ),
                                SizingOperation::Normal,
                            ),
                            AxisSizingRequestOf::new(
                                AvailableSpaceOf::Indefinite,
                                SizingOperation::Normal,
                            ),
                        ),
                        OrthogonalFallbackOf::new(
                            Size::NONE,
                            NearestScrollportOf::Unknown,
                            Size::splat(OrthogonalFallbackLimitOf::Unknown),
                        ),
                    ),
                ),
            )
            .expect("block layout succeeds");

            let collapse = output.block_margin_collapse;
            assert_eq!(
                collapse.at(flow_axes.block_start()).resolve(),
                if expected_collapse {
                    S::from_f64(5.0)
                } else {
                    S::ZERO
                },
                "unexpected block-start collapse for {parent_context:?}"
            );
            assert_eq!(
                collapse.at(flow_axes.block_end()).resolve(),
                if expected_collapse {
                    S::from_f64(5.0)
                } else {
                    S::ZERO
                },
                "unexpected block-end collapse for {parent_context:?}"
            );
            assert_eq!(
                collapse.can_collapse_through(flow_axes),
                expected_collapse,
                "unexpected boundary collapse for {parent_context:?}"
            );
        }

        let mut root_tree = crate::test_support::layout_tree::OracleTreeOf::<S>::new()
            .children(0, [])
            .style(0, NodeInputOf::default());
        let root_output = crate::compute_block(
            &mut root_tree.borrow_compute(),
            0,
            ComputeInputOf::root_layout(
                Size::NONE,
                Size::new(Some(S::from_f64(100.0)), Some(S::from_f64(100.0))),
                ContainingLayoutContext::new(flow_axes, ParentFormattingContext::BlockFlow),
                SizingRequestOf::new(
                    Size::splat(AxisSizingRequestOf::new(
                        AvailableSpaceOf::Definite(
                            NonNegativeFiniteOf::new(S::from_f64(100.0)).unwrap(),
                        ),
                        SizingOperation::Normal,
                    )),
                    OrthogonalFallbackOf::new(
                        Size::NONE,
                        NearestScrollportOf::Unknown,
                        Size::splat(OrthogonalFallbackLimitOf::Unknown),
                    ),
                ),
            ),
        )
        .expect("root-mode block layout succeeds");
        assert!(
            !root_output
                .block_margin_collapse
                .can_collapse_through(flow_axes)
        );
    }

    assert_lane::<f32>();
    assert_lane::<f64>();
}

fn fri06_mr02_physical_edge_value<S: LayoutScalar>(edges: Edges<S>, side: PhysicalSide) -> S {
    match side {
        PhysicalSide::Top => edges.top,
        PhysicalSide::Right => edges.right,
        PhysicalSide::Bottom => edges.bottom,
        PhysicalSide::Left => edges.left,
    }
}

fn assert_fri06_mr02_physical_edge_block_margin_selection<S: LayoutScalar>()
where
    crate::test_support::layout_tree::OracleTreeOf<S>: Traverse<Node = u32, Scalar = S>,
{
    let sentinels = Edges::new(
        S::from_f64(11.0),
        S::from_f64(22.0),
        S::from_f64(33.0),
        S::from_f64(44.0),
    );

    for (writing_mode, direction) in [
        (WritingMode::HorizontalTb, Direction::Ltr),
        (WritingMode::VerticalRl, Direction::Ltr),
        (WritingMode::SidewaysRl, Direction::Rtl),
    ] {
        let flow_axes = FlowAxes::new(writing_mode, direction);
        let mut child_output = ComputeOutputOf::from_outer_size(Size::ZERO);
        child_output.block_margin_collapse = PhysicalBlockMarginCollapseOf::from_block_flow(
            flow_axes,
            CollapsibleMarginOf::ZERO,
            CollapsibleMarginOf::ZERO,
            true,
        );
        let mut tree = crate::test_support::layout_tree::OracleTreeOf::<S>::new()
            .children(0, [1])
            .children(1, [])
            .style(
                0,
                NodeInputOf {
                    display: Display::Block,
                    writing_mode,
                    direction,
                    size: flow_axes.physical_size(crate::geometry::LogicalSizeOf::new(
                        PreferredSizeOf::px(S::from_f64(100.0)),
                        PreferredSizeOf::AUTO,
                    )),
                    ..NodeInputOf::default()
                },
            )
            .style(
                1,
                NodeInputOf {
                    display: Display::Block,
                    writing_mode,
                    direction,
                    margin: sentinels.map(LengthAutoOf::px),
                    ..NodeInputOf::default()
                },
            )
            .measure(1, child_output);

        let output = crate::compute_block(
            &mut tree.borrow_compute(),
            0,
            ComputeInputOf::for_child(
                RunMode::PerformLayout,
                SizingMode::InherentSize,
                RequestedAxis::Both,
                Size::NONE,
                Size::splat(Some(S::from_f64(100.0))),
                ContainingLayoutContext::new(flow_axes, ParentFormattingContext::BlockFlow),
                SizingRequestOf::new(
                    Size::splat(AxisSizingRequestOf::new(
                        AvailableSpaceOf::Definite(
                            NonNegativeFiniteOf::new(S::from_f64(100.0)).unwrap(),
                        ),
                        SizingOperation::Normal,
                    )),
                    OrthogonalFallbackOf::new(
                        Size::NONE,
                        NearestScrollportOf::Unknown,
                        Size::splat(OrthogonalFallbackLimitOf::Unknown),
                    ),
                ),
            ),
        )
        .expect("block layout with physical margin sentinels succeeds");

        assert_eq!(
            output
                .block_margin_collapse
                .at(flow_axes.block_start())
                .resolve(),
            fri06_mr02_physical_edge_value(sentinels, flow_axes.block_start()).max(
                fri06_mr02_physical_edge_value(sentinels, flow_axes.block_end()),
            ),
            "block-start margin selection changed for {writing_mode:?} {direction:?}"
        );
        assert_eq!(
            output
                .block_margin_collapse
                .at(flow_axes.block_end())
                .resolve(),
            fri06_mr02_physical_edge_value(sentinels, flow_axes.block_end()),
            "block-end margin selection changed for {writing_mode:?} {direction:?}"
        );
    }
}

fn assert_fri06_mr02_physical_edge_leaf_collapse_selection<S: LayoutScalar>() {
    let sentinels = [
        (PhysicalSide::Top, S::from_f64(11.0)),
        (PhysicalSide::Right, S::from_f64(22.0)),
        (PhysicalSide::Bottom, S::from_f64(33.0)),
        (PhysicalSide::Left, S::from_f64(44.0)),
    ];

    for (writing_mode, direction) in [
        (WritingMode::HorizontalTb, Direction::Ltr),
        (WritingMode::VerticalRl, Direction::Ltr),
        (WritingMode::VerticalLr, Direction::Rtl),
        (WritingMode::SidewaysRl, Direction::Rtl),
        (WritingMode::SidewaysLr, Direction::Ltr),
    ] {
        let flow_axes = FlowAxes::new(writing_mode, direction);
        for (side, sentinel) in sentinels {
            for use_border in [false, true] {
                let mut edge = Edges::all(LengthOf::ZERO);
                match side {
                    PhysicalSide::Top => edge.top = LengthOf::px(sentinel),
                    PhysicalSide::Right => edge.right = LengthOf::px(sentinel),
                    PhysicalSide::Bottom => edge.bottom = LengthOf::px(sentinel),
                    PhysicalSide::Left => edge.left = LengthOf::px(sentinel),
                }
                let mut style = NodeInputOf {
                    display: Display::Block,
                    writing_mode,
                    direction,
                    ..NodeInputOf::default()
                };
                if use_border {
                    style.border = edge;
                } else {
                    style.padding = edge;
                }
                let input = ComputeInputOf::leaf_layout(
                    Size::NONE,
                    Size::splat(Some(S::from_f64(100.0))),
                    ContainingLayoutContext::new(flow_axes, ParentFormattingContext::BlockFlow),
                    SizingRequestOf::new(
                        Size::splat(AxisSizingRequestOf::new(
                            AvailableSpaceOf::Definite(
                                NonNegativeFiniteOf::new(S::from_f64(100.0)).unwrap(),
                            ),
                            SizingOperation::Normal,
                        )),
                        OrthogonalFallbackOf::new(
                            Size::NONE,
                            NearestScrollportOf::Unknown,
                            Size::splat(OrthogonalFallbackLimitOf::Unknown),
                        ),
                    ),
                )
                .expect("physical-edge leaf input is valid");
                let output = compute_leaf(input, &style, |_| Ok::<_, ()>(Size::ZERO))
                    .expect("physical-edge leaf layout succeeds");
                let selected_for_block_axis =
                    side == flow_axes.block_start() || side == flow_axes.block_end();

                assert_eq!(
                    output.block_margin_collapse.can_collapse_through(flow_axes),
                    !selected_for_block_axis,
                    "leaf {} selection changed for {side:?} in {writing_mode:?} {direction:?}",
                    if use_border { "border" } else { "padding" }
                );
            }
        }
    }
}

#[test]
fn fri06_mr02_physical_edge_block_and_leaf_callers_select_physical_sentinels_both_scalars() {
    assert_fri06_mr02_physical_edge_block_margin_selection::<f32>();
    assert_fri06_mr02_physical_edge_block_margin_selection::<f64>();
    assert_fri06_mr02_physical_edge_leaf_collapse_selection::<f32>();
    assert_fri06_mr02_physical_edge_leaf_collapse_selection::<f64>();
}

#[test]
fn replaced_block_child_keeps_measured_auto_inline_size_in_both_scalar_lanes() {
    fn assert_lane<S: LayoutScalar>() {
        let scalar = scalar_value::<S>;
        let tree = PublicBlockTree::default()
            .with_children(0, [1, 2])
            .with_children(1, [])
            .with_children(2, [])
            .with_style(
                0,
                NodeInputOf {
                    display: Display::Block,
                    size: Size::new(PreferredSizeOf::px(scalar(200.0)), PreferredSizeOf::AUTO),
                    ..NodeInputOf::default()
                },
            )
            .with_style(
                1,
                NodeInputOf {
                    display: Display::Block,
                    item_is_replaced: true,
                    ..NodeInputOf::default()
                },
            )
            .with_style(
                2,
                NodeInputOf {
                    display: Display::Block,
                    ..NodeInputOf::default()
                },
            )
            .with_measurement(1, Size::new(scalar(50.0), scalar(10.0)))
            .with_measurement(2, Size::new(scalar(50.0), scalar(10.0)));
        let batch = compute_layout(
            &tree,
            0,
            crate::test_support::requests::NativeRootFrameOf::new(
                FlowAxes::new(
                    tree.node_input(0).writing_mode,
                    tree.node_input(0).direction,
                ),
                Size::new(Some(NonNegativeFiniteOf::new(scalar(200.0)).unwrap()), None),
                OrthogonalFallbackOf::new(
                    Size::new(Some(NonNegativeFiniteOf::new(scalar(200.0)).unwrap()), None),
                    NearestScrollportOf::Absent,
                    Size::splat(OrthogonalFallbackLimitOf::Unbounded),
                ),
            )
            .request(Size::new(
                AxisSizingRequestOf::new(
                    AvailableSpaceOf::Definite(NonNegativeFiniteOf::new(scalar(200.0)).unwrap()),
                    SizingOperation::Normal,
                ),
                AxisSizingRequestOf::new(AvailableSpaceOf::Indefinite, SizingOperation::Normal),
            )),
        )
        .expect("measured block children lay out");

        assert_eq!(public_final_output(&batch, 1).size.width, scalar(50.0));
        assert_eq!(public_final_output(&batch, 2).size.width, scalar(200.0));
    }

    assert_lane::<f32>();
    assert_lane::<f64>();
}

fn assert_replaced_block_consumes_host_selected_content_sizes<S: LayoutScalar>() {
    let scalar = scalar_value::<S>;
    // These are concrete host measurements. Choosing whether natural facts may
    // be used, and which object-size fallback applies, belongs to that host.
    // In particular, a zero measurement is actual zero, not absent dimensions.
    let host_measurements = [
        ("natural content", Size::new(scalar(50.0), scalar(10.0))),
        (
            "host fallback without natural dimensions",
            Size::new(scalar(300.0), scalar(150.0)),
        ),
        ("zero-sized content", Size::ZERO),
    ];
    for (measurement_kind, measurement) in host_measurements {
        for replaced in [false, true] {
            for authored_size in [false, true] {
                let tree = PublicBlockTree::default()
                    .with_children(0, [1])
                    .with_children(1, [])
                    .with_style(
                        0,
                        NodeInputOf {
                            display: Display::Block,
                            size: Size::new(
                                PreferredSizeOf::px(scalar(400.0)),
                                PreferredSizeOf::AUTO,
                            ),
                            ..NodeInputOf::default()
                        },
                    )
                    .with_style(
                        1,
                        NodeInputOf {
                            display: Display::Block,
                            item_is_replaced: replaced,
                            size: if authored_size {
                                Size::new(
                                    PreferredSizeOf::px(scalar(80.0)),
                                    PreferredSizeOf::px(scalar(40.0)),
                                )
                            } else {
                                Size::new(PreferredSizeOf::AUTO, PreferredSizeOf::AUTO)
                            },
                            ..NodeInputOf::default()
                        },
                    )
                    .with_measurement(1, measurement);
                let batch = compute_layout(
                    &tree,
                    0,
                    crate::test_support::requests::NativeRootFrameOf::new(
                        FlowAxes::new(
                            tree.node_input(0).writing_mode,
                            tree.node_input(0).direction,
                        ),
                        Size::new(Some(NonNegativeFiniteOf::new(scalar(400.0)).unwrap()), None),
                        OrthogonalFallbackOf::new(
                            Size::new(Some(NonNegativeFiniteOf::new(scalar(400.0)).unwrap()), None),
                            NearestScrollportOf::Absent,
                            Size::splat(OrthogonalFallbackLimitOf::Unbounded),
                        ),
                    )
                    .request(Size::new(
                        AxisSizingRequestOf::new(
                            AvailableSpaceOf::Definite(
                                NonNegativeFiniteOf::new(scalar(400.0)).unwrap(),
                            ),
                            SizingOperation::Normal,
                        ),
                        AxisSizingRequestOf::new(
                            AvailableSpaceOf::Indefinite,
                            SizingOperation::Normal,
                        ),
                    )),
                )
                .expect("host-selected content measurement lays out");
                let expected = if authored_size {
                    Size::new(scalar(80.0), scalar(40.0))
                } else if replaced {
                    measurement
                } else {
                    Size::new(scalar(400.0), measurement.height)
                };
                assert_eq!(
                    public_final_output(&batch, 1).size,
                    expected,
                    "{measurement_kind}, replaced={replaced}, authored_size={authored_size}",
                );
            }
        }
    }
}

#[test]
fn replaced_block_consumes_host_selected_content_sizes_in_f32() {
    assert_replaced_block_consumes_host_selected_content_sizes::<f32>();
}

#[test]
fn replaced_block_consumes_host_selected_content_sizes_in_f64() {
    assert_replaced_block_consumes_host_selected_content_sizes::<f64>();
}

fn assert_replaced_zero_height_leaf_prevents_margin_collapse_through<S: LayoutScalar>() {
    let scalar = scalar_value::<S>;
    for replaced in [false, true] {
        let tree = PublicBlockTree::default()
            .with_children(0, [1, 2])
            .with_children(1, [])
            .with_children(2, [])
            .with_style(
                0,
                NodeInputOf {
                    display: Display::Block,
                    size: Size::new(PreferredSizeOf::px(scalar(200.0)), PreferredSizeOf::AUTO),
                    ..NodeInputOf::default()
                },
            )
            .with_style(
                1,
                NodeInputOf {
                    display: Display::Block,
                    item_is_replaced: replaced,
                    margin: Edges::new(
                        LengthAutoOf::px(scalar(10.0)),
                        LengthAutoOf::ZERO,
                        LengthAutoOf::px(scalar(20.0)),
                        LengthAutoOf::ZERO,
                    ),
                    ..NodeInputOf::default()
                },
            )
            .with_style(
                2,
                NodeInputOf {
                    display: Display::Block,
                    size: Size::new(
                        PreferredSizeOf::px(scalar(10.0)),
                        PreferredSizeOf::px(scalar(10.0)),
                    ),
                    margin: Edges::new(
                        LengthAutoOf::px(scalar(30.0)),
                        LengthAutoOf::ZERO,
                        LengthAutoOf::ZERO,
                        LengthAutoOf::ZERO,
                    ),
                    ..NodeInputOf::default()
                },
            )
            .with_measurement(1, Size::new(scalar(50.0), S::ZERO));
        let batch = compute_layout(
            &tree,
            0,
            crate::test_support::requests::NativeRootFrameOf::new(
                FlowAxes::new(
                    tree.node_input(0).writing_mode,
                    tree.node_input(0).direction,
                ),
                Size::new(Some(NonNegativeFiniteOf::new(scalar(200.0)).unwrap()), None),
                OrthogonalFallbackOf::new(
                    Size::new(Some(NonNegativeFiniteOf::new(scalar(200.0)).unwrap()), None),
                    NearestScrollportOf::Absent,
                    Size::splat(OrthogonalFallbackLimitOf::Unbounded),
                ),
            )
            .request(Size::new(
                AxisSizingRequestOf::new(
                    AvailableSpaceOf::Definite(NonNegativeFiniteOf::new(scalar(200.0)).unwrap()),
                    SizingOperation::Normal,
                ),
                AxisSizingRequestOf::new(AvailableSpaceOf::Indefinite, SizingOperation::Normal),
            )),
        )
        .expect("zero-height measured leaf and sibling lay out");
        let first = public_final_output(&batch, 1);
        let following = public_final_output(&batch, 2);
        let root = public_final_output(&batch, 0);

        assert_eq!(first.location.y, scalar(10.0));
        assert_eq!(first.size.height, S::ZERO);
        // CSS Display gives replacements an independent context even at
        // zero height. Their own top/bottom margins cannot collapse through
        // it, while the external adjacent sibling margins still collapse:
        // replaced: 10 + max(20, 30) = 40; ordinary: max(10, 20, 30) = 30.
        let expected_sibling_top = scalar(if replaced { 40.0 } else { 30.0 });
        assert_eq!(
            following.location.y, expected_sibling_top,
            "unexpected collapse-through for replaced={replaced}",
        );
        assert_eq!(root.size.height, expected_sibling_top + scalar(10.0));
    }
}

#[test]
fn replaced_zero_height_leaf_prevents_margin_collapse_through_in_f32() {
    assert_replaced_zero_height_leaf_prevents_margin_collapse_through::<f32>();
}

#[test]
fn replaced_zero_height_leaf_prevents_margin_collapse_through_in_f64() {
    assert_replaced_zero_height_leaf_prevents_margin_collapse_through::<f64>();
}

#[test]
fn block_layout_ignores_item_order_for_geometry() {
    let layout = |item_orders: [ItemOrder; 3]| {
        let tree = PublicBlockTree::default()
            .with_children(0, [1, 2, 3])
            .with_children(1, [])
            .with_children(2, [])
            .with_children(3, [])
            .with_style(
                0,
                NodeInput {
                    display: Display::Block,
                    size: Size::splat_clone(PreferredSize::px(100.0)),
                    ..NodeInput::default()
                },
            )
            .with_style(
                1,
                NodeInput {
                    display: Display::Block,
                    item_order: item_orders[0],
                    size: Size::new(PreferredSize::px(10.0), PreferredSize::px(10.0)),
                    ..NodeInput::default()
                },
            )
            .with_style(
                2,
                NodeInput {
                    display: Display::Block,
                    item_order: item_orders[1],
                    size: Size::new(PreferredSize::px(20.0), PreferredSize::px(20.0)),
                    ..NodeInput::default()
                },
            )
            .with_style(
                3,
                NodeInput {
                    display: Display::Block,
                    item_order: item_orders[2],
                    size: Size::new(PreferredSize::px(30.0), PreferredSize::px(30.0)),
                    ..NodeInput::default()
                },
            );
        let request = crate::test_support::requests::NativeRootFrameOf::new(
            FlowAxes::new(
                tree.node_input(0).writing_mode,
                tree.node_input(0).direction,
            ),
            Size::splat(Some(NonNegativeFiniteOf::new(100.0).unwrap())),
            OrthogonalFallbackOf::new(
                Size::splat(Some(NonNegativeFiniteOf::new(100.0).unwrap())),
                NearestScrollportOf::Absent,
                Size::splat(OrthogonalFallbackLimitOf::Unbounded),
            ),
        )
        .request(Size::splat(AxisSizingRequestOf::new(
            AvailableSpaceOf::Definite(NonNegativeFiniteOf::new(100.0).unwrap()),
            SizingOperation::Normal,
        )));
        let batch = compute_layout(&tree, 0, request).expect("ordinary block layout succeeds");

        [
            public_final_output(&batch, 1),
            public_final_output(&batch, 2),
            public_final_output(&batch, 3),
        ]
    };

    let source_order = layout([ItemOrder::ZERO; 3]);
    let non_default_order = layout([ItemOrder::new(7), ItemOrder::new(-3), ItemOrder::new(2)]);

    assert_eq!(non_default_order, source_order);
    assert_eq!(
        non_default_order.map(|output| (output.source_index, output.location)),
        [
            (SourceIndex::new(0), Point::new(0.0, 0.0)),
            (SourceIndex::new(1), Point::new(0.0, 10.0)),
            (SourceIndex::new(2), Point::new(0.0, 30.0)),
        ]
    );
}

fn assert_ordinary_block_flow<S: LayoutScalar>(
    writing_mode: WritingMode,
    direction: Direction,
    expected_first: Point<S>,
    expected_second: Point<S>,
) {
    let scalar = scalar_value::<S>;
    let child_style = NodeInputOf {
        display: Display::Block,
        writing_mode,
        direction,
        size: Size::new(
            PreferredSizeOf::px(scalar(20.0)),
            PreferredSizeOf::px(scalar(10.0)),
        ),
        ..NodeInputOf::default()
    };
    let tree = PublicBlockTree::default()
        .with_children(0, [1, 2])
        .with_children(1, [])
        .with_children(2, [])
        .with_style(
            0,
            NodeInputOf {
                display: Display::Block,
                writing_mode,
                direction,
                size: Size::new(
                    PreferredSizeOf::px(scalar(100.0)),
                    PreferredSizeOf::px(scalar(100.0)),
                ),
                ..NodeInputOf::default()
            },
        )
        .with_style(1, child_style.clone())
        .with_style(2, child_style);
    let request = crate::test_support::requests::NativeRootFrameOf::new(
        FlowAxes::new(
            tree.node_input(0).writing_mode,
            tree.node_input(0).direction,
        ),
        Size::splat(Some(NonNegativeFiniteOf::new(scalar(100.0)).unwrap())),
        OrthogonalFallbackOf::new(
            Size::splat(Some(NonNegativeFiniteOf::new(scalar(100.0)).unwrap())),
            NearestScrollportOf::Absent,
            Size::splat(OrthogonalFallbackLimitOf::Unbounded),
        ),
    )
    .request(Size::splat(AxisSizingRequestOf::new(
        AvailableSpaceOf::Definite(NonNegativeFiniteOf::new(scalar(100.0)).unwrap()),
        SizingOperation::Normal,
    )));

    let batch = compute_layout(&tree, 0, request).expect("ordinary block layout succeeds");

    assert_eq!(public_final_output(&batch, 1).location, expected_first);
    assert_eq!(public_final_output(&batch, 2).location, expected_second);
}

#[test]
fn ordinary_block_flow_uses_logical_block_progression_for_f32() {
    assert_ordinary_block_flow::<f32>(
        WritingMode::HorizontalTb,
        Direction::Ltr,
        Point::new(0.0, 0.0),
        Point::new(0.0, 10.0),
    );
    assert_ordinary_block_flow::<f32>(
        WritingMode::HorizontalTb,
        Direction::Rtl,
        Point::new(80.0, 0.0),
        Point::new(80.0, 10.0),
    );
    assert_ordinary_block_flow::<f32>(
        WritingMode::VerticalRl,
        Direction::Ltr,
        Point::new(80.0, 0.0),
        Point::new(60.0, 0.0),
    );
    assert_ordinary_block_flow::<f32>(
        WritingMode::VerticalRl,
        Direction::Rtl,
        Point::new(80.0, 90.0),
        Point::new(60.0, 90.0),
    );
    assert_ordinary_block_flow::<f32>(
        WritingMode::VerticalLr,
        Direction::Ltr,
        Point::new(0.0, 0.0),
        Point::new(20.0, 0.0),
    );
    assert_ordinary_block_flow::<f32>(
        WritingMode::VerticalLr,
        Direction::Rtl,
        Point::new(0.0, 90.0),
        Point::new(20.0, 90.0),
    );
    assert_ordinary_block_flow::<f32>(
        WritingMode::SidewaysRl,
        Direction::Ltr,
        Point::new(80.0, 0.0),
        Point::new(60.0, 0.0),
    );
    assert_ordinary_block_flow::<f32>(
        WritingMode::SidewaysRl,
        Direction::Rtl,
        Point::new(80.0, 90.0),
        Point::new(60.0, 90.0),
    );
    assert_ordinary_block_flow::<f32>(
        WritingMode::SidewaysLr,
        Direction::Ltr,
        Point::new(0.0, 90.0),
        Point::new(20.0, 90.0),
    );
    assert_ordinary_block_flow::<f32>(
        WritingMode::SidewaysLr,
        Direction::Rtl,
        Point::new(0.0, 0.0),
        Point::new(20.0, 0.0),
    );
}

#[test]
fn ordinary_block_flow_uses_logical_block_progression_for_f64() {
    assert_ordinary_block_flow::<f64>(
        WritingMode::HorizontalTb,
        Direction::Ltr,
        Point::new(0.0, 0.0),
        Point::new(0.0, 10.0),
    );
    assert_ordinary_block_flow::<f64>(
        WritingMode::HorizontalTb,
        Direction::Rtl,
        Point::new(80.0, 0.0),
        Point::new(80.0, 10.0),
    );
    assert_ordinary_block_flow::<f64>(
        WritingMode::VerticalRl,
        Direction::Ltr,
        Point::new(80.0, 0.0),
        Point::new(60.0, 0.0),
    );
    assert_ordinary_block_flow::<f64>(
        WritingMode::VerticalRl,
        Direction::Rtl,
        Point::new(80.0, 90.0),
        Point::new(60.0, 90.0),
    );
    assert_ordinary_block_flow::<f64>(
        WritingMode::VerticalLr,
        Direction::Ltr,
        Point::new(0.0, 0.0),
        Point::new(20.0, 0.0),
    );
    assert_ordinary_block_flow::<f64>(
        WritingMode::VerticalLr,
        Direction::Rtl,
        Point::new(0.0, 90.0),
        Point::new(20.0, 90.0),
    );
    assert_ordinary_block_flow::<f64>(
        WritingMode::SidewaysRl,
        Direction::Ltr,
        Point::new(80.0, 0.0),
        Point::new(60.0, 0.0),
    );
    assert_ordinary_block_flow::<f64>(
        WritingMode::SidewaysRl,
        Direction::Rtl,
        Point::new(80.0, 90.0),
        Point::new(60.0, 90.0),
    );
    assert_ordinary_block_flow::<f64>(
        WritingMode::SidewaysLr,
        Direction::Ltr,
        Point::new(0.0, 90.0),
        Point::new(20.0, 90.0),
    );
    assert_ordinary_block_flow::<f64>(
        WritingMode::SidewaysLr,
        Direction::Rtl,
        Point::new(0.0, 0.0),
        Point::new(20.0, 0.0),
    );
}

fn assert_ordinary_block_boundaries<S: LayoutScalar>() {
    let scalar = scalar_value::<S>;
    let container_size = Size::new(scalar(100.0), scalar(100.0));
    let child_logical_size = crate::geometry::LogicalSizeOf::new(scalar(20.0), scalar(10.0));

    for (writing_mode, direction) in all_writing_mode_directions() {
        let flow_axes = crate::geometry::FlowAxes::new(writing_mode, direction);
        let child_size = flow_axes.physical_size(child_logical_size);
        let relative_inset = flow_axes.physical_edges(crate::geometry::LogicalEdgesOf::new(
            LengthAutoOf::px(scalar(3.0)),
            LengthAutoOf::AUTO,
            LengthAutoOf::px(scalar(5.0)),
            LengthAutoOf::AUTO,
        ));
        let relative_expected = flow_axes.physical_point(
            crate::geometry::LogicalPointOf::new(scalar(3.0), scalar(5.0)),
            child_logical_size,
            container_size,
        );
        let relative_tree = PublicBlockTree::default()
            .with_children(0, [1])
            .with_children(1, [])
            .with_style(
                0,
                NodeInputOf {
                    display: Display::Block,
                    writing_mode,
                    direction,
                    size: Size::new(
                        PreferredSizeOf::px(scalar(100.0)),
                        PreferredSizeOf::px(scalar(100.0)),
                    ),
                    ..NodeInputOf::default()
                },
            )
            .with_style(
                1,
                NodeInputOf {
                    display: Display::Block,
                    writing_mode,
                    direction,
                    position: Position::Relative,
                    size: child_size.map(PreferredSizeOf::px),
                    inset: relative_inset,
                    ..NodeInputOf::default()
                },
            );
        let request = crate::test_support::requests::NativeRootFrameOf::new(
            FlowAxes::new(WritingMode::HorizontalTb, Direction::Ltr),
            Size::splat(Some(NonNegativeFiniteOf::new(scalar(100.0)).unwrap())),
            OrthogonalFallbackOf::new(
                Size::splat(Some(NonNegativeFiniteOf::new(scalar(100.0)).unwrap())),
                NearestScrollportOf::Absent,
                Size::splat(OrthogonalFallbackLimitOf::Unbounded),
            ),
        )
        .request(Size::splat(AxisSizingRequestOf::new(
            AvailableSpaceOf::Definite(NonNegativeFiniteOf::new(scalar(100.0)).unwrap()),
            SizingOperation::Normal,
        )));
        let relative =
            compute_layout(&relative_tree, 0, request).expect("relative block layout succeeds");

        assert_eq!(
            public_final_output(&relative, 1).location,
            relative_expected
        );

        let inline_expected = flow_axes.physical_point(
            crate::geometry::LogicalPointOf::new(S::ZERO, scalar(10.0)),
            child_logical_size,
            container_size,
        );
        let inline_tree = PublicBlockTree::default()
            .with_children(0, [1, 2])
            .with_children(1, [])
            .with_children(2, [])
            .with_style(
                0,
                NodeInputOf {
                    display: Display::Block,
                    writing_mode,
                    direction,
                    size: Size::new(
                        PreferredSizeOf::px(scalar(100.0)),
                        PreferredSizeOf::px(scalar(100.0)),
                    ),
                    ..NodeInputOf::default()
                },
            )
            .with_style(
                1,
                NodeInputOf {
                    display: Display::Block,
                    writing_mode,
                    direction,
                    size: child_size.map(PreferredSizeOf::px),
                    ..NodeInputOf::default()
                },
            )
            .with_style(
                2,
                NodeInputOf {
                    display: Display::InlineBlock,
                    atomic_inline_participation: Some(fri06_atomic_participation()),
                    writing_mode,
                    direction,
                    size: child_size.map(PreferredSizeOf::px),
                    ..NodeInputOf::default()
                },
            );
        let request = crate::test_support::requests::NativeRootFrameOf::new(
            FlowAxes::new(WritingMode::HorizontalTb, Direction::Ltr),
            Size::splat(Some(NonNegativeFiniteOf::new(scalar(100.0)).unwrap())),
            OrthogonalFallbackOf::new(
                Size::splat(Some(NonNegativeFiniteOf::new(scalar(100.0)).unwrap())),
                NearestScrollportOf::Absent,
                Size::splat(OrthogonalFallbackLimitOf::Unbounded),
            ),
        )
        .request(Size::splat(AxisSizingRequestOf::new(
            AvailableSpaceOf::Definite(NonNegativeFiniteOf::new(scalar(100.0)).unwrap()),
            SizingOperation::Normal,
        )));
        let inline =
            compute_layout(&inline_tree, 0, request).expect("inline block layout succeeds");

        assert_eq!(public_final_output(&inline, 2).location, inline_expected);

        let static_expected = flow_axes.physical_point(
            crate::geometry::LogicalPointOf::new(S::ZERO, scalar(10.0)),
            child_logical_size,
            container_size,
        );
        let static_tree = PublicBlockTree::default()
            .with_children(0, [1, 2])
            .with_children(1, [])
            .with_children(2, [])
            .with_style(
                0,
                NodeInputOf {
                    display: Display::Block,
                    writing_mode,
                    direction,
                    size: Size::new(
                        PreferredSizeOf::px(scalar(100.0)),
                        PreferredSizeOf::px(scalar(100.0)),
                    ),
                    ..NodeInputOf::default()
                },
            )
            .with_style(
                1,
                NodeInputOf {
                    display: Display::Block,
                    writing_mode,
                    direction,
                    size: child_size.map(PreferredSizeOf::px),
                    ..NodeInputOf::default()
                },
            )
            .with_style(
                2,
                NodeInputOf {
                    display: Display::Block,
                    writing_mode,
                    direction,
                    position: Position::Absolute,
                    size: child_size.map(PreferredSizeOf::px),
                    ..NodeInputOf::default()
                },
            );
        let request = crate::test_support::requests::NativeRootFrameOf::new(
            FlowAxes::new(WritingMode::HorizontalTb, Direction::Ltr),
            Size::splat(Some(NonNegativeFiniteOf::new(scalar(100.0)).unwrap())),
            OrthogonalFallbackOf::new(
                Size::splat(Some(NonNegativeFiniteOf::new(scalar(100.0)).unwrap())),
                NearestScrollportOf::Absent,
                Size::splat(OrthogonalFallbackLimitOf::Unbounded),
            ),
        )
        .request(Size::splat(AxisSizingRequestOf::new(
            AvailableSpaceOf::Definite(NonNegativeFiniteOf::new(scalar(100.0)).unwrap()),
            SizingOperation::Normal,
        )));
        let static_position =
            compute_layout(&static_tree, 0, request).expect("static fallback layout succeeds");

        assert_eq!(
            public_final_output(&static_position, 2).location,
            static_expected
        );
    }
}

#[test]
fn ordinary_block_boundaries_project_through_containing_flow_for_f32() {
    assert_ordinary_block_boundaries::<f32>();
}

#[test]
fn ordinary_block_boundaries_project_through_containing_flow_for_f64() {
    assert_ordinary_block_boundaries::<f64>();
}

fn assert_ordinary_block_boundary_baselines<S: LayoutScalar>()
where
    crate::test_support::layout_tree::OracleTreeOf<S>: Traverse<Node = u32, Scalar = S>,
{
    let container_size = Size::new(S::from_f64(100.0), S::from_f64(100.0));
    let logical_size = crate::geometry::LogicalSizeOf::new(S::from_f64(20.0), S::from_f64(10.0));

    for (writing_mode, direction) in all_writing_mode_directions() {
        let flow_axes = crate::geometry::FlowAxes::new(writing_mode, direction);
        let child_size = flow_axes.physical_size(logical_size);
        let mut tree = crate::test_support::layout_tree::OracleTreeOf::<S>::new()
            .children(0, [1, 2])
            .style(
                0,
                NodeInputOf {
                    display: Display::Block,
                    writing_mode,
                    direction,
                    size: Size::new(
                        PreferredSizeOf::px(S::from_f64(100.0)),
                        PreferredSizeOf::px(S::from_f64(100.0)),
                    ),
                    ..NodeInputOf::default()
                },
            )
            .style(
                1,
                NodeInputOf {
                    display: Display::Block,
                    writing_mode,
                    direction,
                    size: child_size.map(PreferredSizeOf::px),
                    ..NodeInputOf::default()
                },
            )
            .style(
                2,
                NodeInputOf {
                    display: Display::InlineBlock,
                    atomic_inline_participation: Some(fri06_atomic_participation()),
                    writing_mode,
                    direction,
                    size: child_size.map(PreferredSizeOf::px),
                    ..NodeInputOf::default()
                },
            );
        let output = crate::compute_block(
            &mut tree.borrow_compute(),
            0,
            ComputeInputOf::for_child(
                RunMode::PerformLayout,
                SizingMode::InherentSize,
                RequestedAxis::Both,
                Size::NONE,
                container_size.map(Some),
                crate::ContainingLayoutContext::new(
                    flow_axes,
                    crate::ParentFormattingContext::NoParent,
                ),
                SizingRequestOf::new(
                    Size::splat(AxisSizingRequestOf::new(
                        AvailableSpaceOf::Definite(
                            NonNegativeFiniteOf::new(S::from_f64(100.0)).unwrap(),
                        ),
                        SizingOperation::Normal,
                    )),
                    OrthogonalFallbackOf::new(
                        Size::NONE,
                        NearestScrollportOf::Unknown,
                        Size::splat(OrthogonalFallbackLimitOf::Unknown),
                    ),
                ),
            ),
        )
        .expect("block layout succeeds");

        let (expected_first, expected_last) =
            if flow_axes.block_axis() == crate::PhysicalAxis::Horizontal {
                let location = flow_axes.physical_point(
                    crate::geometry::LogicalPointOf::new(S::ZERO, S::from_f64(20.0)),
                    crate::geometry::LogicalSizeOf::new(S::ZERO, S::ZERO),
                    container_size,
                );
                let baseline = Point::new(Some(location.x), None);
                (baseline, baseline)
            } else {
                let baseline = Some(S::from_f64(20.0));
                (Point::new(None, baseline), Point::new(None, baseline))
            };
        assert_eq!(output.first_baselines, expected_first);
        assert_eq!(output.last_baselines, expected_last);
    }
}

#[test]
fn ordinary_block_boundaries_project_inline_baselines_for_f32() {
    assert_ordinary_block_boundary_baselines::<f32>();
}

#[test]
fn ordinary_block_boundaries_project_inline_baselines_for_f64() {
    assert_ordinary_block_boundary_baselines::<f64>();
}

fn assert_ordinary_block_boundary_inline_report_overflow<S: LayoutScalar>() {
    let scalar = scalar_value::<S>;
    let root_size = Size::new(scalar(40.0), scalar(100.0));

    for (writing_mode, direction) in all_writing_mode_directions()
        .into_iter()
        .filter(|(writing_mode, _)| *writing_mode != WritingMode::HorizontalTb)
    {
        let expected_scrollable_overflow =
            ScrollRectOf::try_new(Point::ZERO, Size::new(scalar(40.0), scalar(100.0)))
                .expect("finite expected overflow rectangle");
        let tree = PublicBlockTree::default()
            .with_children(0, [1])
            .with_children(1, [])
            .with_style(
                0,
                NodeInputOf {
                    display: Display::Block,
                    writing_mode,
                    direction,
                    text_align: TextAlign::LegacyCenter,
                    overflow: computed_overflow(Overflow::Hidden, Overflow::Hidden),
                    size: root_size.map(PreferredSizeOf::px),
                    ..NodeInputOf::default()
                },
            )
            .with_style(
                1,
                NodeInputOf {
                    display: Display::InlineBlock,
                    atomic_inline_participation: Some(fri06_atomic_participation()),
                    writing_mode,
                    direction,
                    size: Size::splat_clone(PreferredSizeOf::px(scalar(20.0))),
                    ..NodeInputOf::default()
                },
            );
        let batch = compute_layout(
            &tree,
            0,
            crate::test_support::requests::NativeRootFrameOf::new(
                FlowAxes::new(
                    tree.node_input(0).writing_mode,
                    tree.node_input(0).direction,
                ),
                Size::splat(Some(NonNegativeFiniteOf::new(scalar(100.0)).unwrap())),
                OrthogonalFallbackOf::new(
                    Size::splat(Some(NonNegativeFiniteOf::new(scalar(100.0)).unwrap())),
                    NearestScrollportOf::Absent,
                    Size::splat(OrthogonalFallbackLimitOf::Unbounded),
                ),
            )
            .request(Size::splat(AxisSizingRequestOf::new(
                AvailableSpaceOf::Definite(NonNegativeFiniteOf::new(scalar(100.0)).unwrap()),
                SizingOperation::Normal,
            ))),
        )
        .expect("inline run layout succeeds");
        let root = public_final_output(&batch, 0);

        assert_eq!(root.content_size, fri05_c03_block_union_content_size(root));
        assert_eq!(
            root.scroll_geometry
                .expect("root always has scroll geometry")
                .scrollable_overflow(),
            expected_scrollable_overflow,
        );
    }
}

#[test]
fn ordinary_block_boundaries_project_vertical_and_sideways_inline_report_overflow_for_f32() {
    assert_ordinary_block_boundary_inline_report_overflow::<f32>();
}

#[test]
fn ordinary_block_boundaries_project_vertical_and_sideways_inline_report_overflow_for_f64() {
    assert_ordinary_block_boundary_inline_report_overflow::<f64>();
}

fn assert_ordinary_block_boundaries_keep_inline_content_coordinates<S: LayoutScalar>() {
    let scalar = scalar_value::<S>;
    let root_size = Size::new(scalar(50.0), scalar(50.0));
    let padding = Edges::new(
        LengthOf::px(scalar(2.0)),
        LengthOf::px(scalar(3.0)),
        LengthOf::px(scalar(5.0)),
        LengthOf::px(scalar(7.0)),
    );
    let border = Edges::new(
        LengthOf::px(scalar(1.0)),
        LengthOf::px(scalar(2.0)),
        LengthOf::px(scalar(3.0)),
        LengthOf::px(scalar(4.0)),
    );
    let expected_content_size = Size::new(scalar(40.0), scalar(45.0));

    for (writing_mode, direction) in all_writing_mode_directions() {
        let tree = PublicBlockTree::default()
            .with_children(0, [1])
            .with_children(1, [])
            .with_style(
                0,
                NodeInputOf {
                    display: Display::Block,
                    writing_mode,
                    direction,
                    overflow: computed_overflow(Overflow::Hidden, Overflow::Hidden),
                    size: root_size.map(PreferredSizeOf::px),
                    padding,
                    border,
                    ..NodeInputOf::default()
                },
            )
            .with_style(
                1,
                NodeInputOf {
                    display: Display::InlineBlock,
                    atomic_inline_participation: Some(fri06_atomic_participation()),
                    writing_mode,
                    direction,
                    size: expected_content_size.map(PreferredSizeOf::px),
                    ..NodeInputOf::default()
                },
            );
        let batch = compute_layout(
            &tree,
            0,
            crate::test_support::requests::NativeRootFrameOf::new(
                FlowAxes::new(
                    tree.node_input(0).writing_mode,
                    tree.node_input(0).direction,
                ),
                Size::splat(Some(NonNegativeFiniteOf::new(scalar(50.0)).unwrap())),
                OrthogonalFallbackOf::new(
                    Size::splat(Some(NonNegativeFiniteOf::new(scalar(50.0)).unwrap())),
                    NearestScrollportOf::Absent,
                    Size::splat(OrthogonalFallbackLimitOf::Unbounded),
                ),
            )
            .request(Size::splat(AxisSizingRequestOf::new(
                AvailableSpaceOf::Definite(NonNegativeFiniteOf::new(scalar(50.0)).unwrap()),
                SizingOperation::Normal,
            ))),
        )
        .expect("padded inline block layout succeeds");
        let root = public_final_output(&batch, 0);
        let expected_scrollable_overflow = match (writing_mode, direction) {
            (WritingMode::HorizontalTb, Direction::Ltr) => ScrollRectOf::try_new(
                Point::new(scalar(4.0), scalar(1.0)),
                Size::new(scalar(50.0), scalar(52.0)),
            ),
            (WritingMode::HorizontalTb, Direction::Rtl)
            | (WritingMode::VerticalRl, Direction::Ltr)
            | (WritingMode::SidewaysRl, Direction::Ltr) => ScrollRectOf::try_new(
                Point::new(scalar(-2.0), scalar(1.0)),
                Size::new(scalar(50.0), scalar(52.0)),
            ),
            (WritingMode::VerticalRl, Direction::Rtl)
            | (WritingMode::SidewaysRl, Direction::Rtl) => ScrollRectOf::try_new(
                Point::new(scalar(-2.0), scalar(-5.0)),
                Size::new(scalar(50.0), scalar(52.0)),
            ),
            (WritingMode::VerticalLr, Direction::Ltr)
            | (WritingMode::SidewaysLr, Direction::Rtl) => ScrollRectOf::try_new(
                Point::new(scalar(4.0), scalar(1.0)),
                Size::new(scalar(50.0), scalar(52.0)),
            ),
            (WritingMode::VerticalLr, Direction::Rtl)
            | (WritingMode::SidewaysLr, Direction::Ltr) => ScrollRectOf::try_new(
                Point::new(scalar(4.0), scalar(-5.0)),
                Size::new(scalar(50.0), scalar(52.0)),
            ),
        }
        .expect("finite expected scrollable overflow");

        assert_eq!(root.content_size, fri05_c03_block_union_content_size(root));
        assert_eq!(
            root.scroll_geometry
                .expect("root always has scroll geometry")
                .scrollable_overflow(),
            expected_scrollable_overflow,
        );
    }
}

#[test]
fn ordinary_block_boundaries_keep_padded_inline_content_coordinates_for_f32() {
    assert_ordinary_block_boundaries_keep_inline_content_coordinates::<f32>();
}

#[test]
fn ordinary_block_boundaries_keep_padded_inline_content_coordinates_for_f64() {
    assert_ordinary_block_boundaries_keep_inline_content_coordinates::<f64>();
}

fn assert_ordinary_block_boundaries_use_logical_float_bfc_cursor<S: LayoutScalar>() {
    let scalar = scalar_value::<S>;

    for writing_mode in [WritingMode::VerticalRl, WritingMode::VerticalLr] {
        let tree = PublicBlockTree::default()
            .with_children(0, [1, 2, 3])
            .with_children(1, [])
            .with_children(2, [])
            .with_children(3, [])
            .with_style(
                0,
                NodeInputOf {
                    display: Display::Block,
                    writing_mode,
                    size: Size::new(
                        PreferredSizeOf::px(scalar(100.0)),
                        PreferredSizeOf::px(scalar(100.0)),
                    ),
                    ..NodeInputOf::default()
                },
            )
            .with_style(
                1,
                NodeInputOf {
                    display: Display::Block,
                    writing_mode,
                    size: Size::new(
                        PreferredSizeOf::px(scalar(10.0)),
                        PreferredSizeOf::px(scalar(20.0)),
                    ),
                    ..NodeInputOf::default()
                },
            )
            .with_style(
                2,
                NodeInputOf {
                    display: Display::Block,
                    writing_mode,
                    float: Float::Left,
                    size: Size::new(
                        PreferredSizeOf::px(scalar(10.0)),
                        PreferredSizeOf::px(scalar(20.0)),
                    ),
                    ..NodeInputOf::default()
                },
            )
            .with_style(
                3,
                NodeInputOf {
                    display: Display::Block,
                    writing_mode,
                    clear: Clear::Left,
                    overflow: computed_overflow(Overflow::Hidden, Overflow::Hidden),
                    size: Size::new(
                        PreferredSizeOf::px(scalar(10.0)),
                        PreferredSizeOf::px(scalar(20.0)),
                    ),
                    ..NodeInputOf::default()
                },
            );
        let batch = compute_layout(
            &tree,
            0,
            crate::test_support::requests::NativeRootFrameOf::new(
                FlowAxes::new(
                    tree.node_input(0).writing_mode,
                    tree.node_input(0).direction,
                ),
                Size::splat(Some(NonNegativeFiniteOf::new(scalar(100.0)).unwrap())),
                OrthogonalFallbackOf::new(
                    Size::splat(Some(NonNegativeFiniteOf::new(scalar(100.0)).unwrap())),
                    NearestScrollportOf::Absent,
                    Size::splat(OrthogonalFallbackLimitOf::Unbounded),
                ),
            )
            .request(Size::splat(AxisSizingRequestOf::new(
                AvailableSpaceOf::Definite(NonNegativeFiniteOf::new(scalar(100.0)).unwrap()),
                SizingOperation::Normal,
            ))),
        )
        .expect("vertical float and BFC layout succeeds");

        let flow_axes = FlowAxes::new(writing_mode, Direction::Ltr);
        let containing_size = Size::splat(scalar(100.0));
        for (node, expected_block) in [(2, 10.0), (3, 20.0)] {
            let output = public_final_output(&batch, node);
            assert_eq!(
                flow_axes
                    .logical_point(output.location, output.size, containing_size)
                    .block,
                scalar(expected_block),
            );
        }
    }
}

#[test]
fn ordinary_block_boundaries_use_vertical_logical_float_bfc_cursor_for_f32() {
    assert_ordinary_block_boundaries_use_logical_float_bfc_cursor::<f32>();
}

#[test]
fn ordinary_block_boundaries_use_vertical_logical_float_bfc_cursor_for_f64() {
    assert_ordinary_block_boundaries_use_logical_float_bfc_cursor::<f64>();
}

fn assert_ordinary_block_logical_sizing<S: LayoutScalar>(writing_mode: WritingMode) {
    let scalar = scalar_value::<S>;
    let percentage_thirty = LengthOf::value(scalar_percentage::<S>(0.0, 0.3));
    let percentage_sixty = LengthOf::value(scalar_percentage::<S>(0.0, 0.6));
    let tree = PublicBlockTree::default()
        .with_children(0, [1])
        .with_children(1, [])
        .with_style(
            0,
            NodeInputOf {
                display: Display::Block,
                writing_mode,
                size: Size::new(PreferredSizeOf::AUTO, PreferredSizeOf::px(scalar(100.0))),
                ..NodeInputOf::default()
            },
        )
        .with_style(
            1,
            NodeInputOf {
                display: Display::Block,
                writing_mode,
                size: Size::new(PreferredSizeOf::px(scalar(20.0)), PreferredSizeOf::AUTO),
                padding: Edges::new(
                    percentage_thirty,
                    LengthOf::ZERO,
                    percentage_sixty,
                    LengthOf::ZERO,
                ),
                ..NodeInputOf::default()
            },
        );
    let request = crate::test_support::requests::NativeRootFrameOf::new(
        FlowAxes::new(
            tree.node_input(0).writing_mode,
            tree.node_input(0).direction,
        ),
        Size::splat(Some(NonNegativeFiniteOf::new(scalar(100.0)).unwrap())),
        OrthogonalFallbackOf::new(
            Size::splat(Some(NonNegativeFiniteOf::new(scalar(100.0)).unwrap())),
            NearestScrollportOf::Absent,
            Size::splat(OrthogonalFallbackLimitOf::Unbounded),
        ),
    )
    .request(Size::splat(AxisSizingRequestOf::new(
        AvailableSpaceOf::Definite(NonNegativeFiniteOf::new(scalar(100.0)).unwrap()),
        SizingOperation::Normal,
    )));

    let batch = compute_layout(&tree, 0, request).expect("ordinary block layout succeeds");
    let root = public_final_output(&batch, 0);
    let child = public_final_output(&batch, 1);

    assert_eq!(root.size, Size::new(scalar(20.0), scalar(100.0)));
    assert_eq!(child.size, Size::new(scalar(20.0), scalar(100.0)));
    assert_eq!(child.padding.top, scalar(30.0));
    assert_eq!(child.padding.bottom, scalar(60.0));
}

#[test]
fn ordinary_block_logical_sizing_uses_vertical_and_sideways_inline_bases_for_f32() {
    assert_ordinary_block_logical_sizing::<f32>(WritingMode::VerticalRl);
    assert_ordinary_block_logical_sizing::<f32>(WritingMode::VerticalLr);
    assert_ordinary_block_logical_sizing::<f32>(WritingMode::SidewaysRl);
    assert_ordinary_block_logical_sizing::<f32>(WritingMode::SidewaysLr);
}

#[test]
fn ordinary_block_logical_sizing_uses_vertical_and_sideways_inline_bases_for_f64() {
    assert_ordinary_block_logical_sizing::<f64>(WritingMode::VerticalRl);
    assert_ordinary_block_logical_sizing::<f64>(WritingMode::VerticalLr);
    assert_ordinary_block_logical_sizing::<f64>(WritingMode::SidewaysRl);
    assert_ordinary_block_logical_sizing::<f64>(WritingMode::SidewaysLr);
}

fn assert_ordinary_block_collapse_relationship<S: LayoutScalar>(
    child_writing_mode: WritingMode,
    child_direction: Direction,
    measured_leaf: bool,
    expected_second_block_offset: S,
) {
    let scalar = scalar_value::<S>;
    let child_size = if child_writing_mode == WritingMode::HorizontalTb {
        Size::new(
            PreferredSizeOf::px(scalar(10.0)),
            PreferredSizeOf::px(S::ZERO),
        )
    } else {
        Size::new(
            PreferredSizeOf::px(S::ZERO),
            PreferredSizeOf::px(scalar(10.0)),
        )
    };
    let mut tree = PublicBlockTree::default()
        .with_children(0, [1, 2])
        .with_children(1, [])
        .with_children(2, [])
        .with_style(
            0,
            NodeInputOf {
                display: Display::Block,
                size: Size::new(
                    PreferredSizeOf::px(scalar(100.0)),
                    PreferredSizeOf::px(scalar(100.0)),
                ),
                ..NodeInputOf::default()
            },
        )
        .with_style(
            1,
            NodeInputOf {
                display: Display::Block,
                writing_mode: child_writing_mode,
                direction: child_direction,
                size: child_size,
                margin: Edges::new(
                    LengthAutoOf::px(scalar(30.0)),
                    LengthAutoOf::ZERO,
                    LengthAutoOf::px(scalar(60.0)),
                    LengthAutoOf::ZERO,
                ),
                ..NodeInputOf::default()
            },
        )
        .with_style(
            2,
            NodeInputOf {
                display: Display::Block,
                size: Size::new(
                    PreferredSizeOf::px(scalar(10.0)),
                    PreferredSizeOf::px(scalar(10.0)),
                ),
                ..NodeInputOf::default()
            },
        );
    if measured_leaf {
        let measured = if child_writing_mode == WritingMode::HorizontalTb {
            Size::new(scalar(10.0), S::ZERO)
        } else {
            Size::new(S::ZERO, scalar(10.0))
        };
        tree = tree.with_measurement(1, measured);
    }
    let request = crate::test_support::requests::NativeRootFrameOf::new(
        FlowAxes::new(
            tree.node_input(0).writing_mode,
            tree.node_input(0).direction,
        ),
        Size::splat(Some(NonNegativeFiniteOf::new(scalar(100.0)).unwrap())),
        OrthogonalFallbackOf::new(
            Size::splat(Some(NonNegativeFiniteOf::new(scalar(100.0)).unwrap())),
            NearestScrollportOf::Absent,
            Size::splat(OrthogonalFallbackLimitOf::Unbounded),
        ),
    )
    .request(Size::splat(AxisSizingRequestOf::new(
        AvailableSpaceOf::Definite(NonNegativeFiniteOf::new(scalar(100.0)).unwrap()),
        SizingOperation::Normal,
    )));

    let batch = compute_layout(&tree, 0, request).expect("ordinary block layout succeeds");

    assert_eq!(
        public_final_output(&batch, 2).location,
        Point::new(S::ZERO, expected_second_block_offset)
    );
}

fn assert_ordinary_block_relationship_matrix<S: LayoutScalar>() {
    for measured_leaf in [false, true] {
        assert_ordinary_block_collapse_relationship::<S>(
            WritingMode::HorizontalTb,
            Direction::Ltr,
            measured_leaf,
            scalar_value(60.0),
        );
        assert_ordinary_block_collapse_relationship::<S>(
            WritingMode::HorizontalTb,
            Direction::Rtl,
            measured_leaf,
            scalar_value(60.0),
        );
        assert_ordinary_block_collapse_relationship::<S>(
            WritingMode::VerticalRl,
            Direction::Ltr,
            measured_leaf,
            scalar_value(100.0),
        );
    }

    for measured_leaf in [false, true] {
        let scalar = scalar_value::<S>;
        let mut tree = PublicBlockTree::default()
            .with_children(0, [1])
            .with_children(1, [])
            .with_style(
                0,
                NodeInputOf {
                    display: Display::Block,
                    writing_mode: WritingMode::VerticalLr,
                    size: Size::new(
                        PreferredSizeOf::px(scalar(100.0)),
                        PreferredSizeOf::px(scalar(200.0)),
                    ),
                    ..NodeInputOf::default()
                },
            )
            .with_style(
                1,
                NodeInputOf {
                    display: Display::Block,
                    writing_mode: WritingMode::HorizontalTb,
                    size: Size::new(PreferredSizeOf::AUTO, PreferredSizeOf::px(scalar(10.0))),
                    ..NodeInputOf::default()
                },
            );
        if measured_leaf {
            tree = tree.with_measurement(1, Size::new(scalar(5.0), scalar(10.0)));
        }
        let request = crate::test_support::requests::NativeRootFrameOf::new(
            FlowAxes::new(
                tree.node_input(0).writing_mode,
                tree.node_input(0).direction,
            ),
            Size::new(
                Some(NonNegativeFiniteOf::new(scalar(100.0)).unwrap()),
                Some(NonNegativeFiniteOf::new(scalar(200.0)).unwrap()),
            ),
            OrthogonalFallbackOf::new(
                Size::new(
                    Some(NonNegativeFiniteOf::new(scalar(100.0)).unwrap()),
                    Some(NonNegativeFiniteOf::new(scalar(200.0)).unwrap()),
                ),
                NearestScrollportOf::Absent,
                Size::splat(OrthogonalFallbackLimitOf::Unbounded),
            ),
        )
        .request(Size::new(
            AxisSizingRequestOf::new(
                AvailableSpaceOf::Definite(NonNegativeFiniteOf::new(scalar(100.0)).unwrap()),
                SizingOperation::Normal,
            ),
            AxisSizingRequestOf::new(
                AvailableSpaceOf::Definite(NonNegativeFiniteOf::new(scalar(200.0)).unwrap()),
                SizingOperation::Normal,
            ),
        ));

        let batch = compute_layout(&tree, 0, request).expect("orthogonal layout succeeds");
        assert_eq!(
            public_final_output(&batch, 1).size,
            Size::new(scalar(100.0), scalar(10.0))
        );
    }

    for child_direction in [Direction::Ltr, Direction::Rtl] {
        for measured_leaf in [false, true] {
            let scalar = scalar_value::<S>;
            let mut tree = PublicBlockTree::default()
                .with_children(0, [1])
                .with_children(1, [])
                .with_style(
                    0,
                    NodeInputOf {
                        display: Display::Block,
                        writing_mode: WritingMode::VerticalLr,
                        size: Size::new(
                            PreferredSizeOf::px(scalar(100.0)),
                            PreferredSizeOf::px(scalar(200.0)),
                        ),
                        ..NodeInputOf::default()
                    },
                )
                .with_style(
                    1,
                    NodeInputOf {
                        display: Display::Block,
                        writing_mode: WritingMode::VerticalLr,
                        direction: child_direction,
                        size: Size::new(PreferredSizeOf::px(scalar(10.0)), PreferredSizeOf::AUTO),
                        ..NodeInputOf::default()
                    },
                );
            if measured_leaf {
                tree = tree.with_measurement(1, Size::new(scalar(10.0), scalar(5.0)));
            }
            let request = crate::test_support::requests::NativeRootFrameOf::new(
                FlowAxes::new(
                    tree.node_input(0).writing_mode,
                    tree.node_input(0).direction,
                ),
                Size::new(
                    Some(NonNegativeFiniteOf::new(scalar(100.0)).unwrap()),
                    Some(NonNegativeFiniteOf::new(scalar(200.0)).unwrap()),
                ),
                OrthogonalFallbackOf::new(
                    Size::new(
                        Some(NonNegativeFiniteOf::new(scalar(100.0)).unwrap()),
                        Some(NonNegativeFiniteOf::new(scalar(200.0)).unwrap()),
                    ),
                    NearestScrollportOf::Absent,
                    Size::splat(OrthogonalFallbackLimitOf::Unbounded),
                ),
            )
            .request(Size::new(
                AxisSizingRequestOf::new(
                    AvailableSpaceOf::Definite(NonNegativeFiniteOf::new(scalar(100.0)).unwrap()),
                    SizingOperation::Normal,
                ),
                AxisSizingRequestOf::new(
                    AvailableSpaceOf::Definite(NonNegativeFiniteOf::new(scalar(200.0)).unwrap()),
                    SizingOperation::Normal,
                ),
            ));

            let batch = compute_layout(&tree, 0, request).expect("parallel layout succeeds");
            assert_eq!(
                public_final_output(&batch, 1).size,
                Size::new(scalar(10.0), scalar(200.0))
            );
        }
    }
}

fn assert_ordinary_block_opposing_flow_collapse<S: LayoutScalar>(measured_leaf: bool) {
    let scalar = scalar_value::<S>;
    let mut tree = PublicBlockTree::default()
        .with_children(0, [1, 2])
        .with_children(1, [])
        .with_children(2, [])
        .with_style(
            0,
            NodeInputOf {
                display: Display::Block,
                writing_mode: WritingMode::VerticalLr,
                size: Size::new(
                    PreferredSizeOf::px(scalar(100.0)),
                    PreferredSizeOf::px(scalar(100.0)),
                ),
                ..NodeInputOf::default()
            },
        )
        .with_style(
            1,
            NodeInputOf {
                display: Display::Block,
                writing_mode: WritingMode::VerticalRl,
                size: Size::new(
                    PreferredSizeOf::px(S::ZERO),
                    PreferredSizeOf::px(scalar(10.0)),
                ),
                margin: Edges::new(
                    LengthAutoOf::ZERO,
                    LengthAutoOf::px(scalar(60.0)),
                    LengthAutoOf::ZERO,
                    LengthAutoOf::px(scalar(30.0)),
                ),
                ..NodeInputOf::default()
            },
        )
        .with_style(
            2,
            NodeInputOf {
                display: Display::Block,
                writing_mode: WritingMode::VerticalLr,
                size: Size::new(
                    PreferredSizeOf::px(scalar(10.0)),
                    PreferredSizeOf::px(scalar(10.0)),
                ),
                ..NodeInputOf::default()
            },
        );
    if measured_leaf {
        tree = tree.with_measurement(1, Size::new(S::ZERO, scalar(10.0)));
    }
    let request = crate::test_support::requests::NativeRootFrameOf::new(
        FlowAxes::new(
            tree.node_input(0).writing_mode,
            tree.node_input(0).direction,
        ),
        Size::splat(Some(NonNegativeFiniteOf::new(scalar(100.0)).unwrap())),
        OrthogonalFallbackOf::new(
            Size::splat(Some(NonNegativeFiniteOf::new(scalar(100.0)).unwrap())),
            NearestScrollportOf::Absent,
            Size::splat(OrthogonalFallbackLimitOf::Unbounded),
        ),
    )
    .request(Size::splat(AxisSizingRequestOf::new(
        AvailableSpaceOf::Definite(NonNegativeFiniteOf::new(scalar(100.0)).unwrap()),
        SizingOperation::Normal,
    )));

    let batch = compute_layout(&tree, 0, request).expect("opposing block layout succeeds");

    assert_eq!(
        public_final_output(&batch, 1).location,
        Point::new(scalar(30.0), S::ZERO)
    );
    assert_eq!(
        public_final_output(&batch, 2).location,
        Point::new(scalar(60.0), S::ZERO)
    );
}

fn assert_ordinary_block_opposing_flow_collapse_for_scalar<S: LayoutScalar>() {
    for measured_leaf in [false, true] {
        assert_ordinary_block_opposing_flow_collapse::<S>(measured_leaf);
    }
}

fn assert_ordinary_block_orthogonal_inline_margin_subtraction<S: LayoutScalar>(
    measured_leaf: bool,
) {
    let scalar = scalar_value::<S>;
    let mut tree = PublicBlockTree::default()
        .with_children(0, [1])
        .with_children(1, [])
        .with_style(
            0,
            NodeInputOf {
                display: Display::Block,
                writing_mode: WritingMode::VerticalLr,
                size: Size::new(
                    PreferredSizeOf::px(scalar(100.0)),
                    PreferredSizeOf::px(scalar(200.0)),
                ),
                ..NodeInputOf::default()
            },
        )
        .with_style(
            1,
            NodeInputOf {
                display: Display::Block,
                writing_mode: WritingMode::HorizontalTb,
                size: Size::new(PreferredSizeOf::AUTO, PreferredSizeOf::px(scalar(10.0))),
                margin: Edges::new(
                    LengthAutoOf::ZERO,
                    LengthAutoOf::px(scalar(60.0)),
                    LengthAutoOf::ZERO,
                    LengthAutoOf::px(scalar(30.0)),
                ),
                ..NodeInputOf::default()
            },
        );
    if measured_leaf {
        tree = tree.with_measurement(1, Size::new(scalar(5.0), scalar(10.0)));
    }
    let request = crate::test_support::requests::NativeRootFrameOf::new(
        FlowAxes::new(
            tree.node_input(0).writing_mode,
            tree.node_input(0).direction,
        ),
        Size::new(
            Some(NonNegativeFiniteOf::new(scalar(100.0)).unwrap()),
            Some(NonNegativeFiniteOf::new(scalar(200.0)).unwrap()),
        ),
        OrthogonalFallbackOf::new(
            Size::new(
                Some(NonNegativeFiniteOf::new(scalar(100.0)).unwrap()),
                Some(NonNegativeFiniteOf::new(scalar(200.0)).unwrap()),
            ),
            NearestScrollportOf::Absent,
            Size::splat(OrthogonalFallbackLimitOf::Unbounded),
        ),
    )
    .request(Size::new(
        AxisSizingRequestOf::new(
            AvailableSpaceOf::Definite(NonNegativeFiniteOf::new(scalar(100.0)).unwrap()),
            SizingOperation::Normal,
        ),
        AxisSizingRequestOf::new(
            AvailableSpaceOf::Definite(NonNegativeFiniteOf::new(scalar(200.0)).unwrap()),
            SizingOperation::Normal,
        ),
    ));

    let batch = compute_layout(&tree, 0, request).expect("orthogonal layout succeeds");

    assert_eq!(
        public_final_output(&batch, 1).size,
        Size::new(scalar(10.0), scalar(10.0))
    );
}

#[test]
fn ordinary_block_orthogonal_preserves_parallel_opposing_and_measured_leaf_relationships_for_f32() {
    assert_ordinary_block_relationship_matrix::<f32>();
}

#[test]
fn ordinary_block_orthogonal_preserves_parallel_opposing_and_measured_leaf_relationships_for_f64() {
    assert_ordinary_block_relationship_matrix::<f64>();
}

#[test]
fn ordinary_block_opposing_flow_collapse_preserves_real_and_measured_leaves_for_f32() {
    assert_ordinary_block_opposing_flow_collapse_for_scalar::<f32>();
}

#[test]
fn ordinary_block_opposing_flow_collapse_preserves_real_and_measured_leaves_for_f64() {
    assert_ordinary_block_opposing_flow_collapse_for_scalar::<f64>();
}

#[test]
fn ordinary_block_orthogonal_subtracts_physical_child_inline_margins_for_f32() {
    for measured_leaf in [false, true] {
        assert_ordinary_block_orthogonal_inline_margin_subtraction::<f32>(measured_leaf);
    }
}

#[test]
fn ordinary_block_orthogonal_subtracts_physical_child_inline_margins_for_f64() {
    for measured_leaf in [false, true] {
        assert_ordinary_block_orthogonal_inline_margin_subtraction::<f64>(measured_leaf);
    }
}

#[test]
fn block_layout_stacks_in_flow_children_vertically() {
    let mut tree = BlockTree::default();
    tree.insert_children(1, vec![2, 3]);
    tree.insert_children(2, vec![]);
    tree.insert_children(3, vec![]);
    tree.insert_style(
        1,
        NodeInput {
            display: Display::Block,
            size: Size::new(PreferredSize::px(100.0), PreferredSize::AUTO),
            padding: Edges {
                top: Length::px(3.0),
                right: Length::px(5.0),
                bottom: Length::px(7.0),
                left: Length::px(11.0),
            },
            border: Edges::all(Length::px(1.0)),
            ..NodeInput::default()
        },
    );
    tree.insert_style(
        2,
        NodeInput {
            display: Display::Block,
            margin: Edges {
                top: LengthAuto::px(2.0),
                right: LengthAuto::ZERO,
                bottom: LengthAuto::px(4.0),
                left: LengthAuto::px(6.0),
            },
            ..NodeInput::default()
        },
    );
    tree.insert_style(
        3,
        NodeInput {
            display: Display::Block,
            margin: Edges {
                top: LengthAuto::px(5.0),
                right: LengthAuto::ZERO,
                bottom: LengthAuto::ZERO,
                left: LengthAuto::ZERO,
            },
            ..NodeInput::default()
        },
    );
    tree.insert_measure(
        2,
        ComputeOutput::from_sizes(Size::new(20.0, 10.0), Size::new(20.0, 10.0)),
    );
    tree.insert_measure(
        3,
        ComputeOutput::from_sizes(Size::new(30.0, 12.0), Size::new(30.0, 12.0)),
    );

    let output = crate::compute_block(
        &mut tree.borrow_compute(),
        1,
        ComputeInput::for_child(
            RunMode::PerformRootLayout,
            SizingMode::InherentSize,
            RequestedAxis::Both,
            Size::NONE,
            Size::new(Some(300.0), Some(200.0)),
            crate::ContainingLayoutContext::new(
                crate::geometry::FlowAxes::new(
                    crate::WritingMode::HorizontalTb,
                    crate::Direction::Ltr,
                ),
                crate::ParentFormattingContext::NoParent,
            ),
            SizingRequestOf::new(
                Size::new(
                    AxisSizingRequestOf::new(
                        AvailableSpaceOf::Definite(NonNegativeFiniteOf::new(300.0).unwrap()),
                        SizingOperation::Normal,
                    ),
                    AxisSizingRequestOf::new(AvailableSpaceOf::Indefinite, SizingOperation::Normal),
                ),
                OrthogonalFallbackOf::new(
                    Size::NONE,
                    NearestScrollportOf::Unknown,
                    Size::splat(OrthogonalFallbackLimitOf::Unknown),
                ),
            ),
        ),
    )
    .unwrap();

    assert_eq!(output.size, Size::new(100.0, 41.0));
    assert_eq!(output.content_size, Size::new(98.0, 39.0));
    assert_eq!(
        tree.layout(2).expect("child layout is staged").location,
        Point::new(18.0, 6.0)
    );
    assert_eq!(
        tree.layout(2).expect("child layout is staged").size,
        Size::new(20.0, 10.0)
    );
    assert_eq!(
        tree.layout(2).expect("child layout is staged").margin.left,
        6.0
    );
    assert_eq!(
        tree.layout(3).expect("child layout is staged").location,
        Point::new(12.0, 21.0)
    );
    assert_eq!(
        tree.layout(3).expect("child layout is staged").size,
        Size::new(30.0, 12.0)
    );
    assert_eq!(tree.inputs(2)[0].parent(), Size::new(Some(82.0), None));
    assert_eq!(tree.inputs(3)[0].parent(), Size::new(Some(82.0), None));
}

#[test]
fn block_in_flow_affine_margin_resolves_against_containing_block_width() {
    let mut tree = CalcBlockTree::default();
    let margin_left = lp(-4.0, 0.1);
    let width = lp(20.0, 0.5);
    tree.insert_children(1, vec![2]);
    tree.insert_children(2, vec![]);
    tree.insert_style(
        1,
        NodeInput {
            display: Display::Block,
            size: Size::new(PreferredSize::px(200.0), PreferredSize::AUTO),
            ..NodeInput::default()
        },
    );
    tree.insert_style(
        2,
        NodeInput {
            display: Display::Block,
            size: Size::new(PreferredSize::value(width), PreferredSize::AUTO),
            margin: Edges {
                left: LengthAuto::value(margin_left),
                right: LengthAuto::ZERO,
                top: LengthAuto::ZERO,
                bottom: LengthAuto::ZERO,
            },
            ..NodeInput::default()
        },
    );
    tree.insert_measure(2, ComputeOutput::from_outer_size(Size::new(120.0, 10.0)));

    crate::compute_block(
        &mut tree.borrow_compute(),
        1,
        ComputeInput::for_child(
            RunMode::PerformLayout,
            SizingMode::InherentSize,
            RequestedAxis::Both,
            Size::NONE,
            Size::new(Some(200.0), None),
            crate::ContainingLayoutContext::new(
                crate::geometry::FlowAxes::new(
                    crate::WritingMode::HorizontalTb,
                    crate::Direction::Ltr,
                ),
                crate::ParentFormattingContext::NoParent,
            ),
            SizingRequestOf::new(
                Size::new(
                    AxisSizingRequestOf::new(
                        AvailableSpaceOf::Definite(NonNegativeFiniteOf::new(200.0).unwrap()),
                        SizingOperation::Normal,
                    ),
                    AxisSizingRequestOf::new(AvailableSpaceOf::Indefinite, SizingOperation::Normal),
                ),
                OrthogonalFallbackOf::new(
                    Size::NONE,
                    NearestScrollportOf::Unknown,
                    Size::splat(OrthogonalFallbackLimitOf::Unknown),
                ),
            ),
        ),
    )
    .unwrap();

    assert_eq!(tree.inputs(2)[0].known(), Size::new(Some(120.0), None));
    assert_eq!(
        tree.layout(2).expect("child layout is staged").location,
        Point::new(16.0, 0.0)
    );
    assert_eq!(
        tree.layout(2).expect("child layout is staged").margin.left,
        16.0
    );
    assert_eq!(
        tree.layout(2).expect("child layout is staged").size,
        Size::new(120.0, 10.0)
    );
}

#[test]
fn block_container_affine_padding_uses_parent_basis() {
    let mut tree = CalcBlockTree::default();
    let padding = lp(2.0, 0.1);
    tree.insert_children(0, vec![1]);
    tree.insert_children(1, vec![]);
    tree.insert_style(
        0,
        NodeInput {
            padding: Edges::all(Length::value(padding)),
            ..NodeInput::default()
        },
    );
    tree.insert_style(1, NodeInput::default());

    let output = crate::compute_block(
        &mut tree.borrow_compute(),
        0,
        ComputeInput::for_child(
            RunMode::PerformLayout,
            SizingMode::InherentSize,
            RequestedAxis::Both,
            Size::new(Some(100.0), None),
            Size::new(Some(100.0), None),
            crate::ContainingLayoutContext::new(
                crate::geometry::FlowAxes::new(
                    crate::WritingMode::HorizontalTb,
                    crate::Direction::Ltr,
                ),
                crate::ParentFormattingContext::NoParent,
            ),
            SizingRequestOf::new(
                Size::new(
                    AxisSizingRequestOf::new(
                        AvailableSpaceOf::Definite(NonNegativeFiniteOf::new(100.0).unwrap()),
                        SizingOperation::Normal,
                    ),
                    AxisSizingRequestOf::new(AvailableSpaceOf::Indefinite, SizingOperation::Normal),
                ),
                OrthogonalFallbackOf::new(
                    Size::NONE,
                    NearestScrollportOf::Unknown,
                    Size::splat(OrthogonalFallbackLimitOf::Unknown),
                ),
            ),
        ),
    )
    .unwrap();

    assert_eq!(output.content_size.width, 100.0);
}

#[test]
fn block_auto_width_includes_in_flow_child_horizontal_margins() {
    let mut tree = BlockTree::default();
    tree.insert_children(1, vec![2]);
    tree.insert_children(2, vec![]);
    tree.insert_style(
        1,
        NodeInput {
            display: Display::Block,
            size: Size::new(PreferredSize::AUTO, PreferredSize::AUTO),
            ..NodeInput::default()
        },
    );
    tree.insert_style(
        2,
        NodeInput {
            display: Display::Block,
            item_is_table: true,
            margin: Edges {
                top: LengthAuto::ZERO,
                right: LengthAuto::px(9.0),
                bottom: LengthAuto::ZERO,
                left: LengthAuto::px(3.0),
            },
            ..NodeInput::default()
        },
    );
    tree.insert_measure(
        2,
        ComputeOutput::from_sizes(Size::new(20.0, 10.0), Size::new(20.0, 10.0)),
    );

    let output = crate::compute_block(
        &mut tree.borrow_compute(),
        1,
        ComputeInput::for_child(
            RunMode::PerformRootLayout,
            SizingMode::InherentSize,
            RequestedAxis::Both,
            Size::NONE,
            Size::new(Some(300.0), Some(200.0)),
            crate::ContainingLayoutContext::new(
                crate::geometry::FlowAxes::new(
                    crate::WritingMode::HorizontalTb,
                    crate::Direction::Ltr,
                ),
                crate::ParentFormattingContext::NoParent,
            ),
            SizingRequestOf::new(
                Size::new(
                    AxisSizingRequestOf::new(AvailableSpaceOf::Indefinite, SizingOperation::Normal),
                    AxisSizingRequestOf::new(AvailableSpaceOf::Indefinite, SizingOperation::Normal),
                ),
                OrthogonalFallbackOf::new(
                    Size::NONE,
                    NearestScrollportOf::Unknown,
                    Size::splat(OrthogonalFallbackLimitOf::Unknown),
                ),
            ),
        ),
    )
    .unwrap();

    assert_eq!(
        tree.layout(2).expect("child layout is staged").location,
        Point::new(3.0, 0.0)
    );
    assert_eq!(output.size, Size::new(32.0, 10.0));
    assert_eq!(output.content_size, Size::new(32.0, 10.0));
}

#[test]
fn block_layout_collapses_adjacent_in_flow_vertical_margins() {
    let mut tree = BlockTree::default();
    tree.insert_children(1, vec![2, 3]);
    tree.insert_children(2, vec![]);
    tree.insert_children(3, vec![]);
    tree.insert_style(
        1,
        NodeInput {
            display: Display::Block,
            size: Size::new(PreferredSize::px(100.0), PreferredSize::AUTO),
            ..NodeInput::default()
        },
    );
    tree.insert_style(
        2,
        NodeInput {
            display: Display::Block,
            margin: Edges {
                bottom: LengthAuto::px(10.0),
                ..Edges::all(LengthAuto::ZERO)
            },
            ..NodeInput::default()
        },
    );
    tree.insert_style(
        3,
        NodeInput {
            display: Display::Block,
            margin: Edges {
                top: LengthAuto::px(5.0),
                ..Edges::all(LengthAuto::ZERO)
            },
            ..NodeInput::default()
        },
    );
    tree.insert_measure(
        2,
        ComputeOutput::from_sizes(Size::new(100.0, 10.0), Size::new(100.0, 10.0)),
    );
    tree.insert_measure(
        3,
        ComputeOutput::from_sizes(Size::new(100.0, 10.0), Size::new(100.0, 10.0)),
    );

    let output = crate::compute_block(
        &mut tree.borrow_compute(),
        1,
        ComputeInput::for_child(
            RunMode::PerformLayout,
            SizingMode::InherentSize,
            RequestedAxis::Both,
            Size::NONE,
            Size::new(Some(300.0), Some(200.0)),
            crate::ContainingLayoutContext::new(
                crate::geometry::FlowAxes::new(
                    crate::WritingMode::HorizontalTb,
                    crate::Direction::Ltr,
                ),
                crate::ParentFormattingContext::NoParent,
            ),
            SizingRequestOf::new(
                Size::new(
                    AxisSizingRequestOf::new(
                        AvailableSpaceOf::Definite(NonNegativeFiniteOf::new(300.0).unwrap()),
                        SizingOperation::Normal,
                    ),
                    AxisSizingRequestOf::new(AvailableSpaceOf::Indefinite, SizingOperation::Normal),
                ),
                OrthogonalFallbackOf::new(
                    Size::NONE,
                    NearestScrollportOf::Unknown,
                    Size::splat(OrthogonalFallbackLimitOf::Unknown),
                ),
            ),
        ),
    )
    .unwrap();

    assert_eq!(
        tree.layout(2).expect("child layout is staged").location,
        Point::new(0.0, 0.0)
    );
    assert_eq!(
        tree.layout(3).expect("child layout is staged").location,
        Point::new(0.0, 20.0)
    );
    assert_eq!(output.size, Size::new(100.0, 30.0));
    assert_eq!(output.content_size, Size::new(100.0, 30.0));
}

#[test]
fn block_layout_collapses_first_child_top_margin_through_parent() {
    let mut tree = BlockTree::default();
    tree.insert_children(1, vec![2]);
    tree.insert_children(2, vec![]);
    tree.insert_style(
        1,
        NodeInput {
            display: Display::Block,
            size: Size::new(PreferredSize::px(100.0), PreferredSize::AUTO),
            ..NodeInput::default()
        },
    );
    tree.insert_style(
        2,
        NodeInput {
            display: Display::Block,
            margin: Edges {
                top: LengthAuto::px(10.0),
                ..Edges::all(LengthAuto::ZERO)
            },
            ..NodeInput::default()
        },
    );
    tree.insert_measure(
        2,
        ComputeOutput::from_sizes(Size::new(100.0, 5.0), Size::new(100.0, 5.0)),
    );

    let output = crate::compute_block(
        &mut tree.borrow_compute(),
        1,
        ComputeInput::for_child(
            RunMode::PerformLayout,
            SizingMode::InherentSize,
            RequestedAxis::Both,
            Size::NONE,
            Size::new(Some(300.0), Some(200.0)),
            crate::ContainingLayoutContext::new(
                crate::geometry::FlowAxes::new(
                    crate::WritingMode::HorizontalTb,
                    crate::Direction::Ltr,
                ),
                crate::ParentFormattingContext::BlockFlow,
            ),
            SizingRequestOf::new(
                Size::new(
                    AxisSizingRequestOf::new(
                        AvailableSpaceOf::Definite(NonNegativeFiniteOf::new(300.0).unwrap()),
                        SizingOperation::Normal,
                    ),
                    AxisSizingRequestOf::new(AvailableSpaceOf::Indefinite, SizingOperation::Normal),
                ),
                OrthogonalFallbackOf::new(
                    Size::NONE,
                    NearestScrollportOf::Unknown,
                    Size::splat(OrthogonalFallbackLimitOf::Unknown),
                ),
            ),
        ),
    )
    .unwrap();

    assert_eq!(
        tree.layout(2).expect("child layout is staged").location,
        Point::new(0.0, 0.0)
    );
    assert_eq!(output.size, Size::new(100.0, 5.0));
    assert_eq!(
        output.block_margin_collapse.at(PhysicalSide::Top).resolve(),
        10.0
    );
    assert_eq!(
        output
            .block_margin_collapse
            .at(PhysicalSide::Bottom)
            .resolve(),
        0.0
    );
}

#[test]
fn block_scroll_container_keeps_first_child_top_margin_inside() {
    let mut tree = BlockTree::default();
    tree.insert_children(1, vec![2]);
    tree.insert_children(2, vec![]);
    tree.insert_style(
        1,
        NodeInput {
            display: Display::Block,
            overflow: computed_overflow(Overflow::Auto, Overflow::Scroll),
            size: Size::new(PreferredSize::px(100.0), PreferredSize::AUTO),
            ..NodeInput::default()
        },
    );
    tree.insert_style(
        2,
        NodeInput {
            display: Display::Block,
            margin: Edges {
                top: LengthAuto::px(10.0),
                ..Edges::all(LengthAuto::ZERO)
            },
            ..NodeInput::default()
        },
    );
    tree.insert_measure(
        2,
        ComputeOutput::from_sizes(Size::new(100.0, 5.0), Size::new(100.0, 5.0)),
    );

    let output = crate::compute_block(
        &mut tree.borrow_compute(),
        1,
        ComputeInput::for_child(
            RunMode::PerformLayout,
            SizingMode::InherentSize,
            RequestedAxis::Both,
            Size::NONE,
            Size::new(Some(300.0), Some(200.0)),
            crate::ContainingLayoutContext::new(
                crate::geometry::FlowAxes::new(
                    crate::WritingMode::HorizontalTb,
                    crate::Direction::Ltr,
                ),
                crate::ParentFormattingContext::NoParent,
            ),
            SizingRequestOf::new(
                Size::new(
                    AxisSizingRequestOf::new(
                        AvailableSpaceOf::Definite(NonNegativeFiniteOf::new(300.0).unwrap()),
                        SizingOperation::Normal,
                    ),
                    AxisSizingRequestOf::new(AvailableSpaceOf::Indefinite, SizingOperation::Normal),
                ),
                OrthogonalFallbackOf::new(
                    Size::NONE,
                    NearestScrollportOf::Unknown,
                    Size::splat(OrthogonalFallbackLimitOf::Unknown),
                ),
            ),
        ),
    )
    .unwrap();

    assert_eq!(
        tree.layout(2).expect("child layout is staged").location,
        Point::new(0.0, 10.0)
    );
    assert_eq!(output.size, Size::new(100.0, 15.0));
    assert_eq!(output.content_size, Size::new(100.0, 15.0));
    assert_eq!(
        output.block_margin_collapse.at(PhysicalSide::Top).resolve(),
        0.0
    );
    assert_eq!(
        output
            .block_margin_collapse
            .at(PhysicalSide::Bottom)
            .resolve(),
        0.0
    );
    assert!(
        !output
            .block_margin_collapse
            .can_collapse_through(FlowAxes::new(WritingMode::HorizontalTb, Direction::Ltr))
    );
}

#[test]
fn block_rtl_scrollbar_gutter_uses_left_inset() {
    let mut tree = BlockTree::default();
    tree.insert_children(1, vec![2]);
    tree.insert_children(2, vec![]);
    tree.insert_style(
        1,
        NodeInput {
            display: Display::Block,
            direction: Direction::Rtl,
            overflow: computed_overflow(Overflow::Auto, Overflow::Scroll),
            scrollbar_width: crate::ScrollbarWidthOf::try_new(17.0).unwrap(),
            size: Size::new(PreferredSize::px(100.0), PreferredSize::AUTO),
            ..NodeInput::default()
        },
    );
    tree.insert_style(
        2,
        NodeInput {
            display: Display::Block,
            ..NodeInput::default()
        },
    );
    tree.insert_measure(2, ComputeOutput::from_outer_size(Size::new(83.0, 10.0)));

    crate::compute_block(
        &mut tree.borrow_compute(),
        1,
        ComputeInput::for_child(
            RunMode::PerformLayout,
            SizingMode::InherentSize,
            RequestedAxis::Both,
            Size::NONE,
            Size::new(Some(300.0), Some(200.0)),
            crate::ContainingLayoutContext::new(
                crate::geometry::FlowAxes::new(
                    crate::WritingMode::HorizontalTb,
                    crate::Direction::Ltr,
                ),
                crate::ParentFormattingContext::NoParent,
            ),
            SizingRequestOf::new(
                Size::new(
                    AxisSizingRequestOf::new(
                        AvailableSpaceOf::Definite(NonNegativeFiniteOf::new(300.0).unwrap()),
                        SizingOperation::Normal,
                    ),
                    AxisSizingRequestOf::new(AvailableSpaceOf::Indefinite, SizingOperation::Normal),
                ),
                OrthogonalFallbackOf::new(
                    Size::NONE,
                    NearestScrollportOf::Unknown,
                    Size::splat(OrthogonalFallbackLimitOf::Unknown),
                ),
            ),
        ),
    )
    .unwrap();

    assert_eq!(
        tree.layout(2).expect("child layout is staged").location,
        Point::new(17.0, 0.0)
    );
    assert_eq!(
        tree.layout(2).expect("child layout is staged").size,
        Size::new(83.0, 10.0)
    );
}

#[test]
fn block_layout_collapses_last_child_bottom_margin_through_parent() {
    let mut tree = BlockTree::default();
    tree.insert_children(1, vec![2]);
    tree.insert_children(2, vec![]);
    tree.insert_style(
        1,
        NodeInput {
            display: Display::Block,
            size: Size::new(PreferredSize::px(100.0), PreferredSize::AUTO),
            ..NodeInput::default()
        },
    );
    tree.insert_style(
        2,
        NodeInput {
            display: Display::Block,
            margin: Edges {
                bottom: LengthAuto::px(10.0),
                ..Edges::all(LengthAuto::ZERO)
            },
            ..NodeInput::default()
        },
    );
    tree.insert_measure(
        2,
        ComputeOutput::from_sizes(Size::new(100.0, 5.0), Size::new(100.0, 5.0)),
    );

    let output = crate::compute_block(
        &mut tree.borrow_compute(),
        1,
        ComputeInput::for_child(
            RunMode::PerformLayout,
            SizingMode::InherentSize,
            RequestedAxis::Both,
            Size::NONE,
            Size::new(Some(300.0), Some(200.0)),
            crate::ContainingLayoutContext::new(
                crate::geometry::FlowAxes::new(
                    crate::WritingMode::HorizontalTb,
                    crate::Direction::Ltr,
                ),
                crate::ParentFormattingContext::BlockFlow,
            ),
            SizingRequestOf::new(
                Size::new(
                    AxisSizingRequestOf::new(
                        AvailableSpaceOf::Definite(NonNegativeFiniteOf::new(300.0).unwrap()),
                        SizingOperation::Normal,
                    ),
                    AxisSizingRequestOf::new(AvailableSpaceOf::Indefinite, SizingOperation::Normal),
                ),
                OrthogonalFallbackOf::new(
                    Size::NONE,
                    NearestScrollportOf::Unknown,
                    Size::splat(OrthogonalFallbackLimitOf::Unknown),
                ),
            ),
        ),
    )
    .unwrap();

    assert_eq!(
        tree.layout(2).expect("child layout is staged").location,
        Point::new(0.0, 0.0)
    );
    assert_eq!(output.size, Size::new(100.0, 5.0));
    assert_eq!(
        output.block_margin_collapse.at(PhysicalSide::Top).resolve(),
        0.0
    );
    assert_eq!(
        output
            .block_margin_collapse
            .at(PhysicalSide::Bottom)
            .resolve(),
        10.0
    );
}

#[test]
fn block_layout_keeps_grid_child_margins_inside_parent_flow() {
    let mut tree = BlockTree::default();
    tree.insert_children(1, vec![2]);
    tree.insert_children(2, vec![]);
    tree.insert_style(
        1,
        NodeInput {
            display: Display::Block,
            size: Size::new(PreferredSize::px(50.0), PreferredSize::AUTO),
            ..NodeInput::default()
        },
    );
    tree.insert_style(
        2,
        NodeInput {
            display: Display::Grid,
            margin: Edges {
                top: LengthAuto::px(10.0),
                ..Edges::all(LengthAuto::ZERO)
            },
            ..NodeInput::default()
        },
    );
    tree.insert_measure(2, ComputeOutput::from_outer_size(Size::new(50.0, 20.0)));

    let output = crate::compute_block(
        &mut tree.borrow_compute(),
        1,
        ComputeInput::for_child(
            RunMode::PerformLayout,
            SizingMode::InherentSize,
            RequestedAxis::Both,
            Size::NONE,
            Size::NONE,
            crate::ContainingLayoutContext::new(
                crate::geometry::FlowAxes::new(
                    crate::WritingMode::HorizontalTb,
                    crate::Direction::Ltr,
                ),
                crate::ParentFormattingContext::NoParent,
            ),
            SizingRequestOf::new(
                Size::new(
                    AxisSizingRequestOf::new(AvailableSpaceOf::Indefinite, SizingOperation::Normal),
                    AxisSizingRequestOf::new(AvailableSpaceOf::Indefinite, SizingOperation::Normal),
                ),
                OrthogonalFallbackOf::new(
                    Size::NONE,
                    NearestScrollportOf::Unknown,
                    Size::splat(OrthogonalFallbackLimitOf::Unknown),
                ),
            ),
        ),
    )
    .unwrap();

    assert_eq!(output.size, Size::new(50.0, 30.0));
    assert_eq!(
        tree.layout(2).expect("child layout is staged").location,
        Point::new(0.0, 10.0)
    );
    assert_eq!(
        tree.layout(2).expect("child layout is staged").margin.top,
        10.0
    );
}

#[test]
fn block_layout_collapses_margins_through_empty_in_flow_child() {
    let mut tree = BlockTree::default();
    tree.insert_children(1, vec![2, 3]);
    tree.insert_children(2, vec![]);
    tree.insert_children(3, vec![]);
    tree.insert_style(
        1,
        NodeInput {
            display: Display::Block,
            size: Size::new(PreferredSize::px(100.0), PreferredSize::AUTO),
            border: Edges {
                top: Length::px(1.0),
                right: Length::ZERO,
                bottom: Length::px(1.0),
                left: Length::ZERO,
            },
            ..NodeInput::default()
        },
    );
    tree.insert_style(
        2,
        NodeInput {
            display: Display::Block,
            margin: Edges {
                top: LengthAuto::px(10.0),
                bottom: LengthAuto::px(5.0),
                ..Edges::all(LengthAuto::ZERO)
            },
            ..NodeInput::default()
        },
    );
    tree.insert_style(
        3,
        NodeInput {
            display: Display::Block,
            margin: Edges {
                top: LengthAuto::px(7.0),
                ..Edges::all(LengthAuto::ZERO)
            },
            ..NodeInput::default()
        },
    );
    let mut empty_output = ComputeOutput::from_sizes(Size::new(100.0, 0.0), Size::new(100.0, 0.0));
    empty_output.block_margin_collapse = PhysicalBlockMarginCollapse::from_block_flow(
        FlowAxes::new(WritingMode::HorizontalTb, Direction::Ltr),
        CollapsibleMargin::ZERO,
        CollapsibleMargin::ZERO,
        true,
    );
    tree.insert_measure(2, empty_output);
    tree.insert_measure(
        3,
        ComputeOutput::from_sizes(Size::new(100.0, 10.0), Size::new(100.0, 10.0)),
    );

    let output = crate::compute_block(
        &mut tree.borrow_compute(),
        1,
        ComputeInput::for_child(
            RunMode::PerformLayout,
            SizingMode::InherentSize,
            RequestedAxis::Both,
            Size::NONE,
            Size::new(Some(300.0), Some(200.0)),
            crate::ContainingLayoutContext::new(
                crate::geometry::FlowAxes::new(
                    crate::WritingMode::HorizontalTb,
                    crate::Direction::Ltr,
                ),
                crate::ParentFormattingContext::NoParent,
            ),
            SizingRequestOf::new(
                Size::new(
                    AxisSizingRequestOf::new(
                        AvailableSpaceOf::Definite(NonNegativeFiniteOf::new(300.0).unwrap()),
                        SizingOperation::Normal,
                    ),
                    AxisSizingRequestOf::new(AvailableSpaceOf::Indefinite, SizingOperation::Normal),
                ),
                OrthogonalFallbackOf::new(
                    Size::NONE,
                    NearestScrollportOf::Unknown,
                    Size::splat(OrthogonalFallbackLimitOf::Unknown),
                ),
            ),
        ),
    )
    .unwrap();

    assert_eq!(
        tree.layout(2).expect("child layout is staged").location,
        Point::new(0.0, 11.0)
    );
    assert_eq!(
        tree.layout(3).expect("child layout is staged").location,
        Point::new(0.0, 11.0)
    );
    assert_eq!(output.size, Size::new(100.0, 22.0));
    assert_eq!(output.content_size, Size::new(100.0, 20.0));
}

#[test]
fn block_ratio_dependent_height_cannot_collapse_through_with_known_size() {
    let flow = FlowAxes::new(WritingMode::HorizontalTb, Direction::Ltr);
    let ratio = AspectRatio::new(2.0);
    let mut differences = Vec::new();
    for (ratio, preferred_height, known, expected) in [
        (None, PreferredSize::AUTO, Size::new(50.0, 0.0), true),
        (ratio, PreferredSize::px(0.0), Size::new(0.0, 0.0), true),
        (ratio, PreferredSize::AUTO, Size::new(0.0, 0.0), false),
        (ratio, PreferredSize::AUTO, Size::new(50.0, 25.0), false),
    ] {
        let automatic = preferred_height.is_auto();
        let mut tree = OracleTree::new().children(1, []).style(
            1,
            NodeInput {
                display: Display::Block,
                size: Size::new(PreferredSize::AUTO, preferred_height),
                aspect_ratio: ratio,
                ..NodeInput::default()
            },
        );
        let output = crate::compute_block(
            &mut tree.borrow_compute(),
            1,
            ComputeInput::for_child(
                RunMode::PerformLayout,
                SizingMode::InherentSize,
                RequestedAxis::Both,
                known.map(Some),
                Size::new(Some(50.0), None),
                ContainingLayoutContext::new(flow, ParentFormattingContext::BlockFlow),
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
            ),
        )
        .unwrap();
        assert_eq!(output.size, known);
        let actual = output.block_margin_collapse.can_collapse_through(flow);
        if actual != expected {
            differences.push(format!(
                "ratio {ratio:?}, automatic height {automatic}, known {known:?}: \
                 expected {expected}, got {actual}"
            ));
        }
    }
    assert!(differences.is_empty(), "{}", differences.join("\n"));
}

#[test]
fn block_empty_auto_height_can_collapse_through() {
    #[derive(Default)]
    struct BlockTree {
        children: HashMap<u32, Vec<u32>>,
        styles: HashMap<u32, NodeInput>,
    }

    impl Traverse for BlockTree {
        type Node = u32;

        type Scalar = Scalar;
        type Children<'a>
            = std::iter::Copied<std::slice::Iter<'a, u32>>
        where
            Self: 'a;

        fn children(&self, node: Self::Node) -> Self::Children<'_> {
            self.children[&node].iter().copied()
        }

        fn child_count(&self, node: Self::Node) -> usize {
            self.children[&node].len()
        }

        fn child(&self, node: Self::Node, index: usize) -> Self::Node {
            self.children[&node][index]
        }
    }

    struct BlockTreeCompute<'input> {
        children: &'input HashMap<u32, Vec<u32>>,
        styles: &'input HashMap<u32, NodeInput>,
    }

    impl BlockTree {
        fn borrow_compute(&mut self) -> BlockTreeCompute<'_> {
            BlockTreeCompute {
                children: &self.children,
                styles: &self.styles,
            }
        }
    }

    impl<'input> Traverse for BlockTreeCompute<'input> {
        type Node = u32;

        type Scalar = Scalar;
        type Children<'a>
            = std::iter::Copied<std::slice::Iter<'a, u32>>
        where
            Self: 'a;

        fn children(&self, node: Self::Node) -> Self::Children<'_> {
            self.children[&node].iter().copied()
        }

        fn child_count(&self, node: Self::Node) -> usize {
            self.children[&node].len()
        }

        fn child(&self, node: Self::Node, index: usize) -> Self::Node {
            self.children[&node][index]
        }
    }

    impl<'input> Compute<'input> for BlockTreeCompute<'input> {
        fn observe_grid_overflow(
            &mut self,
            _node: Self::Node,
            _probe: crate::grid::GridOverflowProbe<'input, '_, Self::Node, Self::Scalar>,
        ) -> crate::LayoutResultOf<
            Self::Node,
            crate::grid::GridOverflowObservation<Self::Scalar>,
            Self::Scalar,
            core::convert::Infallible,
        > {
            panic!(
                "this focused computation fixture does not admit intrinsic Grid Auto overflow observation"
            );
        }

        fn node_input(&self, node: Self::Node) -> &'input NodeInput {
            &self.styles[&node]
        }

        fn layout_input(&self, node: Self::Node) -> crate::LayoutInputViewOf<'input, Self::Scalar> {
            crate::LayoutInputViewOf::Box(self.node_input(node))
        }

        fn staged_unrounded(
            &self,
            node: Self::Node,
        ) -> crate::engine::contracts::UnroundedEntryResult<
            Self::Node,
            Self::Scalar,
            core::convert::Infallible,
        > {
            Err(crate::test_support::layout_tree::missing_staged_unrounded(
                node,
            ))
        }

        fn set_unrounded(&mut self, _entry: crate::LayoutOutputEntryOf<Self::Node, Self::Scalar>) {}

        fn compute_child(
            &mut self,
            _node: Self::Node,
            _input: ComputeInput,
        ) -> crate::LayoutResultOf<Self::Node, crate::ComputeOutputOf<Self::Scalar>, Self::Scalar>
        {
            panic!("empty block should not measure children")
        }
    }

    let mut tree = BlockTree::default();
    tree.children.insert(1, vec![]);
    tree.styles.insert(
        1,
        NodeInput {
            display: Display::Block,
            size: Size::new(PreferredSize::px(100.0), PreferredSize::AUTO),
            ..NodeInput::default()
        },
    );

    let output = crate::compute_block(
        &mut tree.borrow_compute(),
        1,
        ComputeInput::for_child(
            RunMode::PerformLayout,
            SizingMode::InherentSize,
            RequestedAxis::Both,
            Size::NONE,
            Size::new(Some(300.0), Some(200.0)),
            crate::ContainingLayoutContext::new(
                crate::geometry::FlowAxes::new(
                    crate::WritingMode::HorizontalTb,
                    crate::Direction::Ltr,
                ),
                crate::ParentFormattingContext::BlockFlow,
            ),
            SizingRequestOf::new(
                Size::new(
                    AxisSizingRequestOf::new(
                        AvailableSpaceOf::Definite(NonNegativeFiniteOf::new(300.0).unwrap()),
                        SizingOperation::Normal,
                    ),
                    AxisSizingRequestOf::new(AvailableSpaceOf::Indefinite, SizingOperation::Normal),
                ),
                OrthogonalFallbackOf::new(
                    Size::NONE,
                    NearestScrollportOf::Unknown,
                    Size::splat(OrthogonalFallbackLimitOf::Unknown),
                ),
            ),
        ),
    )
    .unwrap();

    assert_eq!(output.size, Size::new(100.0, 0.0));
    assert!(
        output
            .block_margin_collapse
            .can_collapse_through(FlowAxes::new(WritingMode::HorizontalTb, Direction::Ltr))
    );
}

#[test]
fn block_with_padding_reports_own_margins_when_child_collapse_is_blocked() {
    #[derive(Default)]
    struct BlockTree {
        children: HashMap<u32, Vec<u32>>,
        styles: HashMap<u32, NodeInput>,
    }

    impl Traverse for BlockTree {
        type Node = u32;

        type Scalar = Scalar;
        type Children<'a>
            = std::iter::Copied<std::slice::Iter<'a, u32>>
        where
            Self: 'a;

        fn children(&self, node: Self::Node) -> Self::Children<'_> {
            self.children[&node].iter().copied()
        }

        fn child_count(&self, node: Self::Node) -> usize {
            self.children[&node].len()
        }

        fn child(&self, node: Self::Node, index: usize) -> Self::Node {
            self.children[&node][index]
        }
    }

    struct BlockTreeCompute<'input> {
        children: &'input HashMap<u32, Vec<u32>>,
        styles: &'input HashMap<u32, NodeInput>,
    }

    impl BlockTree {
        fn borrow_compute(&mut self) -> BlockTreeCompute<'_> {
            BlockTreeCompute {
                children: &self.children,
                styles: &self.styles,
            }
        }
    }

    impl<'input> Traverse for BlockTreeCompute<'input> {
        type Node = u32;

        type Scalar = Scalar;
        type Children<'a>
            = std::iter::Copied<std::slice::Iter<'a, u32>>
        where
            Self: 'a;

        fn children(&self, node: Self::Node) -> Self::Children<'_> {
            self.children[&node].iter().copied()
        }

        fn child_count(&self, node: Self::Node) -> usize {
            self.children[&node].len()
        }

        fn child(&self, node: Self::Node, index: usize) -> Self::Node {
            self.children[&node][index]
        }
    }

    impl<'input> Compute<'input> for BlockTreeCompute<'input> {
        fn observe_grid_overflow(
            &mut self,
            _node: Self::Node,
            _probe: crate::grid::GridOverflowProbe<'input, '_, Self::Node, Self::Scalar>,
        ) -> crate::LayoutResultOf<
            Self::Node,
            crate::grid::GridOverflowObservation<Self::Scalar>,
            Self::Scalar,
            core::convert::Infallible,
        > {
            panic!(
                "this focused computation fixture does not admit intrinsic Grid Auto overflow observation"
            );
        }

        fn node_input(&self, node: Self::Node) -> &'input NodeInput {
            &self.styles[&node]
        }

        fn layout_input(&self, node: Self::Node) -> crate::LayoutInputViewOf<'input, Self::Scalar> {
            crate::LayoutInputViewOf::Box(self.node_input(node))
        }

        fn staged_unrounded(
            &self,
            node: Self::Node,
        ) -> crate::engine::contracts::UnroundedEntryResult<
            Self::Node,
            Self::Scalar,
            core::convert::Infallible,
        > {
            Err(crate::test_support::layout_tree::missing_staged_unrounded(
                node,
            ))
        }

        fn set_unrounded(&mut self, _entry: crate::LayoutOutputEntryOf<Self::Node, Self::Scalar>) {}

        fn compute_child(
            &mut self,
            _node: Self::Node,
            _input: ComputeInput,
        ) -> crate::LayoutResultOf<Self::Node, crate::ComputeOutputOf<Self::Scalar>, Self::Scalar>
        {
            panic!("empty block should not measure children")
        }
    }

    let mut tree = BlockTree::default();
    tree.children.insert(1, vec![]);
    tree.styles.insert(
        1,
        NodeInput {
            display: Display::Block,
            size: Size::new(PreferredSize::px(100.0), PreferredSize::AUTO),
            margin: Edges {
                top: LengthAuto::px(8.0),
                bottom: LengthAuto::px(6.0),
                ..Edges::all(LengthAuto::ZERO)
            },
            padding: Edges {
                top: Length::px(1.0),
                bottom: Length::px(1.0),
                ..Edges::all(Length::ZERO)
            },
            ..NodeInput::default()
        },
    );

    let output = crate::compute_block(
        &mut tree.borrow_compute(),
        1,
        ComputeInput::for_child(
            RunMode::PerformLayout,
            SizingMode::InherentSize,
            RequestedAxis::Both,
            Size::NONE,
            Size::new(Some(300.0), Some(200.0)),
            crate::ContainingLayoutContext::new(
                crate::geometry::FlowAxes::new(
                    crate::WritingMode::HorizontalTb,
                    crate::Direction::Ltr,
                ),
                crate::ParentFormattingContext::NoParent,
            ),
            SizingRequestOf::new(
                Size::new(
                    AxisSizingRequestOf::new(
                        AvailableSpaceOf::Definite(NonNegativeFiniteOf::new(300.0).unwrap()),
                        SizingOperation::Normal,
                    ),
                    AxisSizingRequestOf::new(AvailableSpaceOf::Indefinite, SizingOperation::Normal),
                ),
                OrthogonalFallbackOf::new(
                    Size::NONE,
                    NearestScrollportOf::Unknown,
                    Size::splat(OrthogonalFallbackLimitOf::Unknown),
                ),
            ),
        ),
    )
    .unwrap();

    assert_eq!(output.size, Size::new(100.0, 2.0));
    assert_eq!(
        output.block_margin_collapse.at(PhysicalSide::Top).resolve(),
        8.0
    );
    assert_eq!(
        output
            .block_margin_collapse
            .at(PhysicalSide::Bottom)
            .resolve(),
        6.0
    );
    assert!(
        !output
            .block_margin_collapse
            .can_collapse_through(FlowAxes::new(WritingMode::HorizontalTb, Direction::Ltr))
    );
}

fn assert_collapsible_percentage_margins_use_containing_inline_extent<S: LayoutScalar>(
    writing_mode: WritingMode,
) where
    crate::test_support::layout_tree::OracleTreeOf<S>: Traverse<Node = u32, Scalar = S>,
{
    let top_margin = LengthPercentageOf::<S>::from_coefficients(S::ZERO, S::from_f64(0.25))
        .expect("test coefficients are finite");
    let bottom_margin = LengthPercentageOf::<S>::from_coefficients(S::ZERO, S::from_f64(0.5))
        .expect("test coefficients are finite");
    let mut tree = crate::test_support::layout_tree::OracleTreeOf::<S>::new()
        .children(1, [])
        .style(
            1,
            NodeInputOf::<S> {
                display: Display::Block,
                size: Size::new(
                    PreferredSizeOf::px(S::from_f64(100.0)),
                    PreferredSizeOf::AUTO,
                ),
                margin: Edges {
                    top: LengthAutoOf::value(top_margin),
                    bottom: LengthAutoOf::value(bottom_margin),
                    ..Edges::all(LengthAutoOf::ZERO)
                },
                padding: Edges {
                    top: LengthOf::px(S::from_f64(1.0)),
                    bottom: LengthOf::px(S::from_f64(1.0)),
                    ..Edges::all(LengthOf::ZERO)
                },
                ..NodeInputOf::default()
            },
        );

    let output = crate::compute_block(
        &mut tree.borrow_compute(),
        1,
        ComputeInputOf::for_child(
            RunMode::PerformLayout,
            SizingMode::InherentSize,
            RequestedAxis::Both,
            Size::NONE,
            Size::new(Some(S::from_f64(40.0)), Some(S::from_f64(120.0))),
            crate::ContainingLayoutContext::new(
                crate::geometry::FlowAxes::new(writing_mode, Direction::Ltr),
                crate::ParentFormattingContext::NoParent,
            ),
            SizingRequestOf::new(
                Size::new(
                    AxisSizingRequestOf::new(
                        AvailableSpaceOf::Definite(
                            NonNegativeFiniteOf::new(S::from_f64(40.0)).unwrap(),
                        ),
                        SizingOperation::Normal,
                    ),
                    AxisSizingRequestOf::new(
                        AvailableSpaceOf::Definite(
                            NonNegativeFiniteOf::new(S::from_f64(120.0)).unwrap(),
                        ),
                        SizingOperation::Normal,
                    ),
                ),
                OrthogonalFallbackOf::new(
                    Size::NONE,
                    NearestScrollportOf::Unknown,
                    Size::splat(OrthogonalFallbackLimitOf::Unknown),
                ),
            ),
        ),
    )
    .expect("block layout succeeds");

    assert_eq!(
        output.block_margin_collapse.at(PhysicalSide::Top).resolve(),
        S::from_f64(30.0)
    );
    assert_eq!(
        output
            .block_margin_collapse
            .at(PhysicalSide::Bottom)
            .resolve(),
        S::from_f64(60.0)
    );
}

#[test]
fn collapsible_percentage_margins_use_non_horizontal_containing_inline_extent_for_f32() {
    assert_collapsible_percentage_margins_use_containing_inline_extent::<f32>(
        WritingMode::VerticalRl,
    );
    assert_collapsible_percentage_margins_use_containing_inline_extent::<f32>(
        WritingMode::SidewaysLr,
    );
}

#[test]
fn collapsible_percentage_margins_use_non_horizontal_containing_inline_extent_for_f64() {
    assert_collapsible_percentage_margins_use_containing_inline_extent::<f64>(
        WritingMode::VerticalRl,
    );
    assert_collapsible_percentage_margins_use_containing_inline_extent::<f64>(
        WritingMode::SidewaysLr,
    );
}

#[test]
fn block_in_flow_invalid_numeric_horizontal_margin_uses_zero_fallback() {
    let invalid_margin = LengthPercentageOf::from_coefficients(f32::MAX, f32::MAX)
        .expect("test coefficients are finite");
    let mut tree = crate::test_support::layout_tree::OracleTree::new()
        .children(1, [2])
        .children(2, [])
        .style(
            1,
            NodeInput {
                display: Display::Block,
                size: Size::new(PreferredSize::px(f32::MAX), PreferredSize::AUTO),
                ..NodeInput::default()
            },
        )
        .style(
            2,
            NodeInput {
                display: Display::Block,
                size: Size::new(PreferredSize::px(10.0), PreferredSize::AUTO),
                margin: Edges {
                    left: LengthAuto::value(invalid_margin),
                    ..Edges::all(LengthAuto::ZERO)
                },
                ..NodeInput::default()
            },
        );

    crate::compute_block(
        &mut tree.borrow_compute(),
        1,
        ComputeInput::for_child(
            RunMode::PerformLayout,
            SizingMode::InherentSize,
            RequestedAxis::Both,
            Size::NONE,
            Size::new(Some(f32::MAX), None),
            crate::ContainingLayoutContext::new(
                crate::geometry::FlowAxes::new(WritingMode::HorizontalTb, Direction::Ltr),
                crate::ParentFormattingContext::NoParent,
            ),
            SizingRequestOf::new(
                Size::new(
                    AxisSizingRequestOf::new(
                        AvailableSpaceOf::Definite(NonNegativeFiniteOf::new(f32::MAX).unwrap()),
                        SizingOperation::Normal,
                    ),
                    AxisSizingRequestOf::new(AvailableSpaceOf::Indefinite, SizingOperation::Normal),
                ),
                OrthogonalFallbackOf::new(
                    Size::NONE,
                    NearestScrollportOf::Unknown,
                    Size::splat(OrthogonalFallbackLimitOf::Unknown),
                ),
            ),
        ),
    )
    .expect("the in-flow invalid-numeric margin falls back to zero");

    assert_eq!(
        tree.output(2)
            .expect("child block receives an in-flow layout")
            .margin
            .left,
        0.0
    );
}

#[test]
fn block_layout_positions_in_flow_children_from_right_edge_in_rtl() {
    let mut tree = BlockTree::default();
    tree.insert_children(1, vec![2]);
    tree.insert_children(2, vec![]);
    tree.insert_style(
        1,
        NodeInput {
            display: Display::Block,
            direction: Direction::Rtl,
            size: Size::new(PreferredSize::px(100.0), PreferredSize::AUTO),
            padding: Edges {
                top: Length::ZERO,
                right: Length::px(5.0),
                bottom: Length::ZERO,
                left: Length::px(11.0),
            },
            border: Edges::all(Length::px(1.0)),
            ..NodeInput::default()
        },
    );
    tree.insert_style(
        2,
        NodeInput {
            display: Display::Block,
            margin: Edges {
                top: LengthAuto::ZERO,
                right: LengthAuto::px(7.0),
                bottom: LengthAuto::ZERO,
                left: LengthAuto::px(3.0),
            },
            ..NodeInput::default()
        },
    );
    tree.insert_measure(
        2,
        ComputeOutput::from_sizes(Size::new(20.0, 10.0), Size::new(20.0, 10.0)),
    );

    let output = crate::compute_block(
        &mut tree.borrow_compute(),
        1,
        ComputeInput::for_child(
            RunMode::PerformLayout,
            SizingMode::InherentSize,
            RequestedAxis::Both,
            Size::NONE,
            Size::new(Some(300.0), Some(200.0)),
            crate::ContainingLayoutContext::new(
                crate::geometry::FlowAxes::new(
                    crate::WritingMode::HorizontalTb,
                    crate::Direction::Ltr,
                ),
                crate::ParentFormattingContext::NoParent,
            ),
            SizingRequestOf::new(
                Size::new(
                    AxisSizingRequestOf::new(
                        AvailableSpaceOf::Definite(NonNegativeFiniteOf::new(300.0).unwrap()),
                        SizingOperation::Normal,
                    ),
                    AxisSizingRequestOf::new(AvailableSpaceOf::Indefinite, SizingOperation::Normal),
                ),
                OrthogonalFallbackOf::new(
                    Size::NONE,
                    NearestScrollportOf::Unknown,
                    Size::splat(OrthogonalFallbackLimitOf::Unknown),
                ),
            ),
        ),
    )
    .unwrap();

    assert_eq!(output.size, Size::new(100.0, 12.0));
    assert_eq!(
        tree.layout(2).expect("child layout is staged").location,
        Point::new(67.0, 1.0)
    );
    assert_eq!(
        tree.layout(2).expect("child layout is staged").size,
        Size::new(20.0, 10.0)
    );
    assert_eq!(
        tree.layout(2).expect("child layout is staged").margin.right,
        7.0
    );
}

#[test]
fn block_layout_expands_horizontal_auto_margins_for_in_flow_children() {
    let mut tree = BlockTree::default();
    tree.insert_children(1, vec![2]);
    tree.insert_children(2, vec![]);
    tree.insert_style(
        1,
        NodeInput {
            display: Display::Block,
            size: Size::new(PreferredSize::px(100.0), PreferredSize::AUTO),
            ..NodeInput::default()
        },
    );
    tree.insert_style(
        2,
        NodeInput {
            display: Display::Block,
            margin: Edges {
                top: LengthAuto::ZERO,
                right: LengthAuto::AUTO,
                bottom: LengthAuto::ZERO,
                left: LengthAuto::AUTO,
            },
            ..NodeInput::default()
        },
    );
    tree.insert_measure(
        2,
        ComputeOutput::from_sizes(Size::new(20.0, 10.0), Size::new(20.0, 10.0)),
    );

    let output = crate::compute_block(
        &mut tree.borrow_compute(),
        1,
        ComputeInput::for_child(
            RunMode::PerformLayout,
            SizingMode::InherentSize,
            RequestedAxis::Both,
            Size::NONE,
            Size::new(Some(300.0), Some(200.0)),
            crate::ContainingLayoutContext::new(
                crate::geometry::FlowAxes::new(
                    crate::WritingMode::HorizontalTb,
                    crate::Direction::Ltr,
                ),
                crate::ParentFormattingContext::NoParent,
            ),
            SizingRequestOf::new(
                Size::new(
                    AxisSizingRequestOf::new(
                        AvailableSpaceOf::Definite(NonNegativeFiniteOf::new(300.0).unwrap()),
                        SizingOperation::Normal,
                    ),
                    AxisSizingRequestOf::new(AvailableSpaceOf::Indefinite, SizingOperation::Normal),
                ),
                OrthogonalFallbackOf::new(
                    Size::NONE,
                    NearestScrollportOf::Unknown,
                    Size::splat(OrthogonalFallbackLimitOf::Unknown),
                ),
            ),
        ),
    )
    .unwrap();

    assert_eq!(output.size, Size::new(100.0, 10.0));
    assert_eq!(output.content_size, Size::new(100.0, 10.0));
    assert_eq!(
        tree.layout(2).expect("child layout is staged").location,
        Point::new(40.0, 0.0)
    );
    assert_eq!(
        tree.layout(2).expect("child layout is staged").size,
        Size::new(20.0, 10.0)
    );
    assert_eq!(
        tree.layout(2).expect("child layout is staged").margin.left,
        40.0
    );
    assert_eq!(
        tree.layout(2).expect("child layout is staged").margin.right,
        40.0
    );
}

#[test]
fn block_content_size_includes_visible_child_overflow_content() {
    let mut tree = BlockTree::default();
    tree.insert_children(1, vec![2]);
    tree.insert_children(2, vec![]);
    tree.insert_style(
        1,
        NodeInput {
            display: Display::Block,
            size: Size::new(PreferredSize::px(100.0), PreferredSize::AUTO),
            ..NodeInput::default()
        },
    );
    tree.insert_style(
        2,
        NodeInput {
            display: Display::Block,
            overflow: computed_overflow(Overflow::Visible, Overflow::Visible),
            ..NodeInput::default()
        },
    );
    tree.insert_measure(
        2,
        ComputeOutput::from_sizes(Size::new(40.0, 10.0), Size::new(120.0, 24.0)),
    );

    let output = crate::compute_block(
        &mut tree.borrow_compute(),
        1,
        ComputeInput::for_child(
            RunMode::PerformLayout,
            SizingMode::InherentSize,
            RequestedAxis::Both,
            Size::NONE,
            Size::new(Some(300.0), Some(200.0)),
            crate::ContainingLayoutContext::new(
                crate::geometry::FlowAxes::new(
                    crate::WritingMode::HorizontalTb,
                    crate::Direction::Ltr,
                ),
                crate::ParentFormattingContext::NoParent,
            ),
            SizingRequestOf::new(
                Size::new(
                    AxisSizingRequestOf::new(
                        AvailableSpaceOf::Definite(NonNegativeFiniteOf::new(300.0).unwrap()),
                        SizingOperation::Normal,
                    ),
                    AxisSizingRequestOf::new(AvailableSpaceOf::Indefinite, SizingOperation::Normal),
                ),
                OrthogonalFallbackOf::new(
                    Size::NONE,
                    NearestScrollportOf::Unknown,
                    Size::splat(OrthogonalFallbackLimitOf::Unknown),
                ),
            ),
        ),
    )
    .unwrap();

    assert_eq!(
        tree.layout(2).expect("child layout is staged").location,
        Point::new(0.0, 0.0)
    );
    assert_eq!(
        tree.layout(2).expect("child layout is staged").size,
        Size::new(40.0, 10.0)
    );
    assert_eq!(output.content_size, Size::new(120.0, 24.0));
}

#[test]
fn block_relative_child_inset_offsets_final_layout_location() {
    let mut tree = BlockTree::default();
    tree.insert_children(1, vec![2]);
    tree.insert_children(2, vec![]);
    tree.insert_style(
        1,
        NodeInput {
            display: Display::Block,
            size: Size::new(PreferredSize::px(100.0), PreferredSize::AUTO),
            ..NodeInput::default()
        },
    );
    tree.insert_style(
        2,
        NodeInput {
            display: Display::Block,
            inset: Edges {
                left: LengthAuto::px(7.0),
                top: LengthAuto::px(3.0),
                ..Edges::all(LengthAuto::AUTO)
            },
            margin: Edges {
                top: LengthAuto::px(2.0),
                right: LengthAuto::ZERO,
                bottom: LengthAuto::px(4.0),
                left: LengthAuto::px(6.0),
            },
            ..NodeInput::default()
        },
    );
    tree.insert_measure(
        2,
        ComputeOutput::from_sizes(Size::new(20.0, 10.0), Size::new(20.0, 10.0)),
    );

    let output = crate::compute_block(
        &mut tree.borrow_compute(),
        1,
        ComputeInput::for_child(
            RunMode::PerformLayout,
            SizingMode::InherentSize,
            RequestedAxis::Both,
            Size::NONE,
            Size::new(Some(300.0), Some(200.0)),
            crate::ContainingLayoutContext::new(
                crate::geometry::FlowAxes::new(
                    crate::WritingMode::HorizontalTb,
                    crate::Direction::Ltr,
                ),
                crate::ParentFormattingContext::BlockFlow,
            ),
            SizingRequestOf::new(
                Size::new(
                    AxisSizingRequestOf::new(
                        AvailableSpaceOf::Definite(NonNegativeFiniteOf::new(300.0).unwrap()),
                        SizingOperation::Normal,
                    ),
                    AxisSizingRequestOf::new(AvailableSpaceOf::Indefinite, SizingOperation::Normal),
                ),
                OrthogonalFallbackOf::new(
                    Size::NONE,
                    NearestScrollportOf::Unknown,
                    Size::splat(OrthogonalFallbackLimitOf::Unknown),
                ),
            ),
        ),
    )
    .unwrap();

    assert_eq!(output.size, Size::new(100.0, 10.0));
    assert_eq!(
        tree.layout(2).expect("child layout is staged").location,
        Point::new(13.0, 3.0)
    );
    assert_eq!(
        tree.layout(2).expect("child layout is staged").size,
        Size::new(20.0, 10.0)
    );
}

#[test]
fn block_layout_stretches_auto_width_in_flow_children() {
    let mut tree = BlockTree::default();
    tree.insert_children(1, vec![2]);
    tree.insert_children(2, vec![]);
    tree.insert_style(
        1,
        NodeInput {
            display: Display::Block,
            size: Size::new(PreferredSize::px(100.0), PreferredSize::AUTO),
            padding: Edges {
                top: Length::ZERO,
                left: Length::px(5.0),
                right: Length::px(7.0),
                bottom: Length::ZERO,
            },
            ..NodeInput::default()
        },
    );
    tree.insert_style(
        2,
        NodeInput {
            display: Display::Block,
            margin: Edges {
                left: LengthAuto::px(3.0),
                right: LengthAuto::px(9.0),
                ..Edges::all(LengthAuto::ZERO)
            },
            ..NodeInput::default()
        },
    );
    tree.insert_measure(2, ComputeOutput::from_outer_size(Size::new(76.0, 10.0)));

    let output = crate::compute_block(
        &mut tree.borrow_compute(),
        1,
        ComputeInput::for_child(
            RunMode::PerformLayout,
            SizingMode::InherentSize,
            RequestedAxis::Both,
            Size::NONE,
            Size::new(Some(300.0), Some(200.0)),
            crate::ContainingLayoutContext::new(
                crate::geometry::FlowAxes::new(
                    crate::WritingMode::HorizontalTb,
                    crate::Direction::Ltr,
                ),
                crate::ParentFormattingContext::NoParent,
            ),
            SizingRequestOf::new(
                Size::new(
                    AxisSizingRequestOf::new(
                        AvailableSpaceOf::Definite(NonNegativeFiniteOf::new(300.0).unwrap()),
                        SizingOperation::Normal,
                    ),
                    AxisSizingRequestOf::new(AvailableSpaceOf::Indefinite, SizingOperation::Normal),
                ),
                OrthogonalFallbackOf::new(
                    Size::NONE,
                    NearestScrollportOf::Unknown,
                    Size::splat(OrthogonalFallbackLimitOf::Unknown),
                ),
            ),
        ),
    )
    .unwrap();

    assert_eq!(tree.inputs(2)[0].known().width, Some(76.0));
    assert_eq!(
        tree.layout(2).expect("child layout is staged").size,
        Size::new(76.0, 10.0)
    );
    assert_eq!(
        tree.layout(2).expect("child layout is staged").location,
        Point::new(8.0, 0.0)
    );
    assert_eq!(output.content_size, Size::new(100.0, 10.0));
    assert_eq!(output.size, Size::new(100.0, 10.0));
}

#[test]
fn block_compute_size_uses_in_flow_children_for_auto_height() {
    let mut tree = BlockTree::default();
    tree.insert_children(1, vec![2]);
    tree.insert_children(2, vec![]);
    tree.insert_style(
        1,
        NodeInput {
            display: Display::Block,
            size: Size::new(PreferredSize::px(100.0), PreferredSize::AUTO),
            padding: Edges {
                top: Length::px(3.0),
                left: Length::px(5.0),
                right: Length::px(7.0),
                bottom: Length::px(7.0),
            },
            ..NodeInput::default()
        },
    );
    tree.insert_style(
        2,
        NodeInput {
            display: Display::Block,
            margin: Edges {
                top: LengthAuto::px(2.0),
                right: LengthAuto::px(9.0),
                bottom: LengthAuto::px(4.0),
                left: LengthAuto::px(3.0),
            },
            ..NodeInput::default()
        },
    );
    tree.insert_measure(2, ComputeOutput::from_outer_size(Size::new(76.0, 10.0)));

    let output = crate::compute_block(
        &mut tree.borrow_compute(),
        1,
        ComputeInput::for_child(
            RunMode::ComputeSize,
            SizingMode::InherentSize,
            RequestedAxis::Both,
            Size::NONE,
            Size::new(Some(300.0), Some(200.0)),
            crate::ContainingLayoutContext::new(
                crate::geometry::FlowAxes::new(
                    crate::WritingMode::HorizontalTb,
                    crate::Direction::Ltr,
                ),
                crate::ParentFormattingContext::NoParent,
            ),
            SizingRequestOf::new(
                Size::new(
                    AxisSizingRequestOf::new(
                        AvailableSpaceOf::Definite(NonNegativeFiniteOf::new(300.0).unwrap()),
                        SizingOperation::Normal,
                    ),
                    AxisSizingRequestOf::new(AvailableSpaceOf::Indefinite, SizingOperation::Normal),
                ),
                OrthogonalFallbackOf::new(
                    Size::NONE,
                    NearestScrollportOf::Unknown,
                    Size::splat(OrthogonalFallbackLimitOf::Unknown),
                ),
            ),
        ),
    )
    .unwrap();

    assert_eq!(tree.inputs(2)[0].run_mode(), RunMode::ComputeSize);
    assert_eq!(tree.inputs(2)[0].known().width, Some(76.0));
    assert_eq!(output.size, Size::new(100.0, 26.0));
    assert_eq!(output.content_size, Size::ZERO);
    assert!(tree.layout(2).is_none());
}

#[test]
fn block_compute_size_uses_definite_min_max_without_measuring_children() {
    #[derive(Default)]
    struct BlockTree {
        children: HashMap<u32, Vec<u32>>,
        styles: HashMap<u32, NodeInput>,
    }

    impl Traverse for BlockTree {
        type Node = u32;

        type Scalar = Scalar;
        type Children<'a>
            = std::iter::Copied<std::slice::Iter<'a, u32>>
        where
            Self: 'a;

        fn children(&self, node: Self::Node) -> Self::Children<'_> {
            self.children[&node].iter().copied()
        }

        fn child_count(&self, node: Self::Node) -> usize {
            self.children[&node].len()
        }

        fn child(&self, node: Self::Node, index: usize) -> Self::Node {
            self.children[&node][index]
        }
    }

    struct BlockTreeCompute<'input> {
        children: &'input HashMap<u32, Vec<u32>>,
        styles: &'input HashMap<u32, NodeInput>,
    }

    impl BlockTree {
        fn borrow_compute(&mut self) -> BlockTreeCompute<'_> {
            BlockTreeCompute {
                children: &self.children,
                styles: &self.styles,
            }
        }
    }

    impl<'input> Traverse for BlockTreeCompute<'input> {
        type Node = u32;

        type Scalar = Scalar;
        type Children<'a>
            = std::iter::Copied<std::slice::Iter<'a, u32>>
        where
            Self: 'a;

        fn children(&self, node: Self::Node) -> Self::Children<'_> {
            self.children[&node].iter().copied()
        }

        fn child_count(&self, node: Self::Node) -> usize {
            self.children[&node].len()
        }

        fn child(&self, node: Self::Node, index: usize) -> Self::Node {
            self.children[&node][index]
        }
    }

    impl<'input> Compute<'input> for BlockTreeCompute<'input> {
        fn observe_grid_overflow(
            &mut self,
            _node: Self::Node,
            _probe: crate::grid::GridOverflowProbe<'input, '_, Self::Node, Self::Scalar>,
        ) -> crate::LayoutResultOf<
            Self::Node,
            crate::grid::GridOverflowObservation<Self::Scalar>,
            Self::Scalar,
            core::convert::Infallible,
        > {
            panic!(
                "this focused computation fixture does not admit intrinsic Grid Auto overflow observation"
            );
        }

        fn node_input(&self, node: Self::Node) -> &'input NodeInput {
            &self.styles[&node]
        }

        fn layout_input(&self, node: Self::Node) -> crate::LayoutInputViewOf<'input, Self::Scalar> {
            crate::LayoutInputViewOf::Box(self.node_input(node))
        }

        fn staged_unrounded(
            &self,
            node: Self::Node,
        ) -> crate::engine::contracts::UnroundedEntryResult<
            Self::Node,
            Self::Scalar,
            core::convert::Infallible,
        > {
            Err(crate::test_support::layout_tree::missing_staged_unrounded(
                node,
            ))
        }

        fn set_unrounded(&mut self, _entry: crate::LayoutOutputEntryOf<Self::Node, Self::Scalar>) {}

        fn compute_child(
            &mut self,
            _node: Self::Node,
            _input: ComputeInput,
        ) -> crate::LayoutResultOf<Self::Node, crate::ComputeOutputOf<Self::Scalar>, Self::Scalar>
        {
            panic!("definite min/max compute-size should not measure children")
        }
    }

    let mut tree = BlockTree::default();
    tree.children.insert(1, vec![2]);
    tree.children.insert(2, vec![]);
    tree.styles.insert(
        1,
        NodeInput {
            display: Display::Block,
            min_size: Size::new(MinSize::px(100.0), MinSize::px(40.0)),
            max_size: Size::new(MaxSize::px(100.0), MaxSize::px(40.0)),
            ..NodeInput::default()
        },
    );
    tree.styles.insert(2, NodeInput::default());

    let output = crate::compute_block(
        &mut tree.borrow_compute(),
        1,
        ComputeInput::for_child(
            RunMode::ComputeSize,
            SizingMode::InherentSize,
            RequestedAxis::Both,
            Size::NONE,
            Size::new(Some(500.0), Some(400.0)),
            crate::ContainingLayoutContext::new(
                crate::geometry::FlowAxes::new(
                    crate::WritingMode::HorizontalTb,
                    crate::Direction::Ltr,
                ),
                crate::ParentFormattingContext::NoParent,
            ),
            SizingRequestOf::new(
                Size::new(
                    AxisSizingRequestOf::new(
                        AvailableSpaceOf::Definite(NonNegativeFiniteOf::new(500.0).unwrap()),
                        SizingOperation::Normal,
                    ),
                    AxisSizingRequestOf::new(AvailableSpaceOf::Indefinite, SizingOperation::Normal),
                ),
                OrthogonalFallbackOf::new(
                    Size::NONE,
                    NearestScrollportOf::Unknown,
                    Size::splat(OrthogonalFallbackLimitOf::Unknown),
                ),
            ),
        ),
    )
    .unwrap();

    assert_eq!(output.size, Size::new(100.0, 40.0));
    assert_eq!(output.content_size, Size::ZERO);
}

#[test]
fn block_definite_compute_size_keeps_grid_children_on_fast_path_until_grid_baselines() {
    #[derive(Default)]
    struct BlockTree {
        children: HashMap<u32, Vec<u32>>,
        styles: HashMap<u32, NodeInput>,
    }

    impl Traverse for BlockTree {
        type Node = u32;

        type Scalar = Scalar;
        type Children<'a>
            = std::iter::Copied<std::slice::Iter<'a, u32>>
        where
            Self: 'a;

        fn children(&self, node: Self::Node) -> Self::Children<'_> {
            self.children[&node].iter().copied()
        }

        fn child_count(&self, node: Self::Node) -> usize {
            self.children[&node].len()
        }

        fn child(&self, node: Self::Node, index: usize) -> Self::Node {
            self.children[&node][index]
        }
    }

    struct BlockTreeCompute<'input> {
        children: &'input HashMap<u32, Vec<u32>>,
        styles: &'input HashMap<u32, NodeInput>,
    }

    impl BlockTree {
        fn borrow_compute(&mut self) -> BlockTreeCompute<'_> {
            BlockTreeCompute {
                children: &self.children,
                styles: &self.styles,
            }
        }
    }

    impl<'input> Traverse for BlockTreeCompute<'input> {
        type Node = u32;

        type Scalar = Scalar;
        type Children<'a>
            = std::iter::Copied<std::slice::Iter<'a, u32>>
        where
            Self: 'a;

        fn children(&self, node: Self::Node) -> Self::Children<'_> {
            self.children[&node].iter().copied()
        }

        fn child_count(&self, node: Self::Node) -> usize {
            self.children[&node].len()
        }

        fn child(&self, node: Self::Node, index: usize) -> Self::Node {
            self.children[&node][index]
        }
    }

    impl<'input> Compute<'input> for BlockTreeCompute<'input> {
        fn observe_grid_overflow(
            &mut self,
            _node: Self::Node,
            _probe: crate::grid::GridOverflowProbe<'input, '_, Self::Node, Self::Scalar>,
        ) -> crate::LayoutResultOf<
            Self::Node,
            crate::grid::GridOverflowObservation<Self::Scalar>,
            Self::Scalar,
            core::convert::Infallible,
        > {
            panic!(
                "this focused computation fixture does not admit intrinsic Grid Auto overflow observation"
            );
        }

        fn node_input(&self, node: Self::Node) -> &'input NodeInput {
            &self.styles[&node]
        }

        fn layout_input(&self, node: Self::Node) -> crate::LayoutInputViewOf<'input, Self::Scalar> {
            crate::LayoutInputViewOf::Box(self.node_input(node))
        }

        fn staged_unrounded(
            &self,
            node: Self::Node,
        ) -> crate::engine::contracts::UnroundedEntryResult<
            Self::Node,
            Self::Scalar,
            core::convert::Infallible,
        > {
            Err(crate::test_support::layout_tree::missing_staged_unrounded(
                node,
            ))
        }

        fn set_unrounded(&mut self, _entry: crate::LayoutOutputEntryOf<Self::Node, Self::Scalar>) {}

        fn compute_child(
            &mut self,
            _node: Self::Node,
            _input: ComputeInput,
        ) -> crate::LayoutResultOf<Self::Node, crate::ComputeOutputOf<Self::Scalar>, Self::Scalar>
        {
            panic!("definite grid compute-size should stay on the fast path")
        }
    }

    for display in [Display::Grid, Display::GridLanes] {
        let mut tree = BlockTree::default();
        tree.children.insert(1, vec![2]);
        tree.children.insert(2, vec![3]);
        tree.children.insert(3, vec![]);
        tree.styles.insert(
            1,
            NodeInput {
                display: Display::Block,
                min_size: Size::new(MinSize::px(100.0), MinSize::px(40.0)),
                max_size: Size::new(MaxSize::px(100.0), MaxSize::px(40.0)),
                ..NodeInput::default()
            },
        );
        tree.styles.insert(
            2,
            NodeInput {
                display,
                ..NodeInput::default()
            },
        );
        tree.styles.insert(3, NodeInput::default());

        let output = crate::compute_block(
            &mut tree.borrow_compute(),
            1,
            ComputeInput::for_child(
                RunMode::ComputeSize,
                SizingMode::InherentSize,
                RequestedAxis::Both,
                Size::NONE,
                Size::new(Some(500.0), Some(400.0)),
                crate::ContainingLayoutContext::new(
                    crate::geometry::FlowAxes::new(
                        crate::WritingMode::HorizontalTb,
                        crate::Direction::Ltr,
                    ),
                    crate::ParentFormattingContext::NoParent,
                ),
                SizingRequestOf::new(
                    Size::new(
                        AxisSizingRequestOf::new(
                            AvailableSpaceOf::Definite(NonNegativeFiniteOf::new(500.0).unwrap()),
                            SizingOperation::Normal,
                        ),
                        AxisSizingRequestOf::new(
                            AvailableSpaceOf::Indefinite,
                            SizingOperation::Normal,
                        ),
                    ),
                    OrthogonalFallbackOf::new(
                        Size::NONE,
                        NearestScrollportOf::Unknown,
                        Size::splat(OrthogonalFallbackLimitOf::Unknown),
                    ),
                ),
            ),
        )
        .unwrap();

        assert_eq!(output.size, Size::new(100.0, 40.0));
        assert_eq!(output.content_size, Size::ZERO);
    }
}

#[test]
fn block_auto_height_clamps_to_max_size() {
    let mut tree = BlockTree::default();
    tree.insert_children(1, vec![2]);
    tree.insert_children(2, vec![]);
    tree.insert_style(
        1,
        NodeInput {
            display: Display::Block,
            size: Size::new(PreferredSize::px(100.0), PreferredSize::AUTO),
            max_size: Size::new(MaxSize::NONE, MaxSize::px(12.0)),
            ..NodeInput::default()
        },
    );
    tree.insert_style(
        2,
        NodeInput {
            display: Display::Block,
            ..NodeInput::default()
        },
    );
    tree.insert_measure(
        2,
        ComputeOutput::from_sizes(Size::new(100.0, 20.0), Size::new(100.0, 20.0)),
    );

    let output = crate::compute_block(
        &mut tree.borrow_compute(),
        1,
        ComputeInput::for_child(
            RunMode::PerformLayout,
            SizingMode::InherentSize,
            RequestedAxis::Both,
            Size::NONE,
            Size::new(Some(300.0), Some(200.0)),
            crate::ContainingLayoutContext::new(
                crate::geometry::FlowAxes::new(
                    crate::WritingMode::HorizontalTb,
                    crate::Direction::Ltr,
                ),
                crate::ParentFormattingContext::NoParent,
            ),
            SizingRequestOf::new(
                Size::new(
                    AxisSizingRequestOf::new(
                        AvailableSpaceOf::Definite(NonNegativeFiniteOf::new(300.0).unwrap()),
                        SizingOperation::Normal,
                    ),
                    AxisSizingRequestOf::new(AvailableSpaceOf::Indefinite, SizingOperation::Normal),
                ),
                OrthogonalFallbackOf::new(
                    Size::NONE,
                    NearestScrollportOf::Unknown,
                    Size::splat(OrthogonalFallbackLimitOf::Unknown),
                ),
            ),
        ),
    )
    .unwrap();

    assert_eq!(output.size, Size::new(100.0, 12.0));
    assert_eq!(output.content_size, Size::new(100.0, 20.0));
    assert_eq!(
        tree.layout(2).expect("child layout is staged").size,
        Size::new(100.0, 20.0)
    );
}

#[test]
fn block_auto_size_applies_aspect_ratio_to_max_size() {
    let mut tree = BlockTree::default();
    tree.insert_children(1, vec![2]);
    tree.insert_children(2, vec![]);
    tree.insert_style(
        1,
        NodeInput {
            display: Display::Block,
            aspect_ratio: AspectRatio::new(2.0),
            max_size: Size::new(MaxSize::px(50.0), MaxSize::NONE),
            ..NodeInput::default()
        },
    );
    tree.insert_style(
        2,
        NodeInput {
            display: Display::Block,
            ..NodeInput::default()
        },
    );
    tree.insert_measure(
        2,
        ComputeOutput::from_sizes(Size::new(80.0, 40.0), Size::new(80.0, 40.0)),
    );

    let output = crate::compute_block(
        &mut tree.borrow_compute(),
        1,
        ComputeInput::for_child(
            RunMode::PerformLayout,
            SizingMode::InherentSize,
            RequestedAxis::Both,
            Size::NONE,
            Size::new(Some(500.0), Some(400.0)),
            crate::ContainingLayoutContext::new(
                crate::geometry::FlowAxes::new(
                    crate::WritingMode::HorizontalTb,
                    crate::Direction::Ltr,
                ),
                crate::ParentFormattingContext::NoParent,
            ),
            SizingRequestOf::new(
                Size::new(
                    AxisSizingRequestOf::new(AvailableSpaceOf::Indefinite, SizingOperation::Normal),
                    AxisSizingRequestOf::new(AvailableSpaceOf::Indefinite, SizingOperation::Normal),
                ),
                OrthogonalFallbackOf::new(
                    Size::NONE,
                    NearestScrollportOf::Unknown,
                    Size::splat(OrthogonalFallbackLimitOf::Unknown),
                ),
            ),
        ),
    )
    .unwrap();

    assert_eq!(output.size, Size::new(50.0, 25.0));
}

#[test]
fn block_legacy_text_align_offsets_table_child_in_free_inline_space() {
    fn run(text_align: TextAlign, direction: Direction) -> NodeOutput {
        let mut tree = BlockTree::default();
        tree.insert_children(1, vec![2]);
        tree.insert_children(2, vec![]);
        tree.insert_style(
            1,
            NodeInput {
                display: Display::Block,
                direction,
                text_align,
                size: Size::new(PreferredSize::px(200.0), PreferredSize::AUTO),
                ..NodeInput::default()
            },
        );
        tree.insert_style(
            2,
            NodeInput {
                display: Display::Block,
                item_is_table: true,
                ..NodeInput::default()
            },
        );
        tree.insert_measure(2, ComputeOutput::from_outer_size(Size::new(60.0, 10.0)));

        crate::compute_block(
            &mut tree.borrow_compute(),
            1,
            ComputeInput::for_child(
                RunMode::PerformLayout,
                SizingMode::InherentSize,
                RequestedAxis::Both,
                Size::NONE,
                Size::new(Some(300.0), Some(200.0)),
                crate::ContainingLayoutContext::new(
                    crate::geometry::FlowAxes::new(
                        crate::WritingMode::HorizontalTb,
                        crate::Direction::Ltr,
                    ),
                    crate::ParentFormattingContext::NoParent,
                ),
                SizingRequestOf::new(
                    Size::new(
                        AxisSizingRequestOf::new(
                            AvailableSpaceOf::Definite(NonNegativeFiniteOf::new(300.0).unwrap()),
                            SizingOperation::Normal,
                        ),
                        AxisSizingRequestOf::new(
                            AvailableSpaceOf::Indefinite,
                            SizingOperation::Normal,
                        ),
                    ),
                    OrthogonalFallbackOf::new(
                        Size::NONE,
                        NearestScrollportOf::Unknown,
                        Size::splat(OrthogonalFallbackLimitOf::Unknown),
                    ),
                ),
            ),
        )
        .unwrap();

        tree.layout(2).expect("child layout is staged")
    }

    assert_eq!(
        run(TextAlign::LegacyCenter, Direction::Ltr).location.x,
        70.0
    );
    assert_eq!(
        run(TextAlign::LegacyRight, Direction::Ltr).location.x,
        140.0
    );
    assert_eq!(
        run(TextAlign::LegacyCenter, Direction::Rtl).location.x,
        70.0
    );
    assert_eq!(run(TextAlign::LegacyLeft, Direction::Rtl).location.x, 0.0);
}

fn assert_leading_empty_margins_expose_whole_group<S: LayoutScalar>(external_float: bool) {
    let px = S::from_f64;
    let width = NonNegativeFiniteOf::new(px(100.0)).unwrap();
    let request = crate::test_support::requests::NativeRootFrameOf::new(
        FlowAxes::new(WritingMode::HorizontalTb, Direction::Ltr),
        Size::new(Some(width), None),
        OrthogonalFallbackOf::new(
            Size::NONE,
            NearestScrollportOf::Absent,
            Size::splat(OrthogonalFallbackLimitOf::Unbounded),
        ),
    )
    .request(Size::new(
        AxisSizingRequestOf::new(AvailableSpaceOf::Definite(width), SizingOperation::Normal),
        AxisSizingRequestOf::new(AvailableSpaceOf::Indefinite, SizingOperation::Normal),
    ));
    let ordinary = NodeInputOf {
        display: Display::Block,
        block_flow_context: BlockFlowContext::Normal,
        size: Size::new(PreferredSizeOf::px(px(100.0)), PreferredSizeOf::AUTO),
        min_size: Size::new(MinSizeOf::px(S::ZERO), MinSizeOf::px(S::ZERO)),
        ..NodeInputOf::default()
    };
    for leading_empty in [false, true] {
        let mut tree = PublicBlockTree::default()
            .with_children(0, if external_float { vec![1, 2] } else { vec![2] })
            .with_children(2, if leading_empty { vec![3, 4] } else { vec![4] })
            .with_children(4, [])
            .with_style(0, ordinary.clone())
            .with_style(2, ordinary.clone())
            .with_style(
                4,
                NodeInputOf {
                    size: Size::new(
                        PreferredSizeOf::px(px(100.0)),
                        PreferredSizeOf::px(px(10.0)),
                    ),
                    margin: Edges {
                        top: LengthAutoOf::px(px(20.0)),
                        ..Edges::all(LengthAutoOf::ZERO)
                    },
                    ..ordinary.clone()
                },
            );
        if external_float {
            tree = tree.with_children(1, []).with_style(
                1,
                NodeInputOf {
                    float: Float::Left,
                    size: Size::new(
                        PreferredSizeOf::px(px(100.0)),
                        PreferredSizeOf::px(px(30.0)),
                    ),
                    ..ordinary.clone()
                },
            );
        }
        if leading_empty {
            tree = tree.with_children(3, []).with_style(
                3,
                NodeInputOf {
                    margin: Edges {
                        top: LengthAutoOf::px(px(8.0)),
                        bottom: LengthAutoOf::px(px(12.0)),
                        ..Edges::all(LengthAutoOf::ZERO)
                    },
                    ..ordinary.clone()
                },
            );
        }
        let batch = compute_layout(&tree, 0, request).unwrap();
        if external_float {
            let float = public_final_output(&batch, 1);
            assert_eq!(
                (float.location, float.size),
                (Point::ZERO, Size::new(px(100.0), px(30.0)))
            );
        }
        if leading_empty {
            let empty = public_final_output(&batch, 3);
            assert_eq!(
                (empty.location, empty.size),
                (Point::ZERO, Size::new(px(100.0), S::ZERO))
            );
        }
        let root = public_final_output(&batch, 0);
        let parent = public_final_output(&batch, 2);
        let body = public_final_output(&batch, 4);
        // P->B is the nearest control. Adding empty A adjoins its8/12
        // margins to B's20, so the same maximum20 remains exposed by P.
        assert_eq!(
            (
                parent.location,
                parent.size,
                body.location,
                body.size,
                root.size
            ),
            (
                Point::new(S::ZERO, px(20.0)),
                Size::new(px(100.0), px(10.0)),
                Point::ZERO,
                Size::new(px(100.0), px(10.0)),
                Size::new(px(100.0), px(30.0)),
            ),
            "external float {external_float}, leading empty {leading_empty}: parent/body/root"
        );
    }
}

#[test]
fn leading_empty_margins_expose_whole_group_without_floats_f32() {
    assert_leading_empty_margins_expose_whole_group::<f32>(false);
}
#[test]
fn leading_empty_margins_expose_whole_group_without_floats_f64() {
    assert_leading_empty_margins_expose_whole_group::<f64>(false);
}
#[test]
fn leading_empty_margins_expose_whole_group_with_external_float_f32() {
    assert_leading_empty_margins_expose_whole_group::<f32>(true);
}
#[test]
fn leading_empty_margins_expose_whole_group_with_external_float_f64() {
    assert_leading_empty_margins_expose_whole_group::<f64>(true);
}

#[test]
fn leading_margin_components_keep_extrema_and_actual_barriers_in_all_flows() {
    fn lane<S: LayoutScalar>() {
        use crate::geometry::{LogicalEdgesOf, LogicalSizeOf};
        let px = S::from_f64;
        for (writing_mode, direction) in all_writing_mode_directions() {
            let axes = FlowAxes::new(writing_mode, direction);
            let margins = |top, bottom| {
                axes.physical_edges(LogicalEdgesOf::new(S::ZERO, S::ZERO, px(top), px(bottom)))
                    .map(LengthAutoOf::px)
            };
            let edge = axes
                .physical_edges(LogicalEdgesOf::new(S::ZERO, S::ZERO, px(1.0), S::ZERO))
                .map(LengthOf::px);
            let ordinary = NodeInputOf {
                display: Display::Block,
                writing_mode,
                direction,
                size: axes.physical_size(LogicalSizeOf::new(
                    PreferredSizeOf::px(px(100.0)),
                    PreferredSizeOf::AUTO,
                )),
                min_size: Size::new(MinSizeOf::px(S::ZERO), MinSizeOf::px(S::ZERO)),
                ..Default::default()
            };
            let request = crate::test_support::requests::NativeRootFrameOf::new(
                axes,
                axes.physical_size(LogicalSizeOf::new(
                    Some(NonNegativeFiniteOf::new(px(100.0)).unwrap()),
                    None,
                )),
                OrthogonalFallbackOf::new(
                    Size::NONE,
                    NearestScrollportOf::Absent,
                    Size::splat(OrthogonalFallbackLimitOf::Unbounded),
                ),
            )
            .request(axes.physical_size(LogicalSizeOf::new(
                AxisSizingRequestOf::new(
                    AvailableSpaceOf::Definite(NonNegativeFiniteOf::new(px(100.0)).unwrap()),
                    SizingOperation::Normal,
                ),
                AxisSizingRequestOf::new(AvailableSpaceOf::Indefinite, SizingOperation::Normal),
            )));
            for (case, y, height, empty_y, empty_height, body_y) in [
                ("baseline", 12.0, 10.0, 0.0, 0.0, 0.0),
                ("larger_body_margin", 20.0, 10.0, 0.0, 0.0, 0.0),
                ("multiple", 15.0, 10.0, 0.0, 0.0, 0.0),
                ("signed", 3.0, 10.0, 0.0, 0.0, 0.0),
                ("own_positive", 20.0, 10.0, 0.0, 0.0, 0.0),
                ("own_negative", 8.0, 10.0, 0.0, 0.0, 0.0),
                ("parent_border", 0.0, 23.0, 9.0, 0.0, 13.0),
                ("parent_padding", 0.0, 23.0, 9.0, 0.0, 13.0),
                ("parent_independent", 0.0, 22.0, 8.0, 0.0, 12.0),
                ("child_border", 8.0, 23.0, 0.0, 1.0, 13.0),
                ("child_padding", 8.0, 23.0, 0.0, 1.0, 13.0),
                ("child_minimum", 8.0, 23.0, 0.0, 1.0, 13.0),
                ("child_independent", 8.0, 22.0, 0.0, 0.0, 12.0),
                ("parent_minimum", 12.0, 10.0, 0.0, 0.0, 0.0),
                ("all_empty_minimum", 12.0, 5.0, 0.0, 0.0, 0.0),
                ("direct_root", 0.0, 22.0, 8.0, 0.0, 12.0),
            ] {
                let mut parent = ordinary.clone();
                let mut empty = NodeInputOf {
                    margin: margins(8.0, 12.0),
                    ..ordinary.clone()
                };
                let mut body = NodeInputOf {
                    size: axes.physical_size(LogicalSizeOf::new(
                        PreferredSizeOf::px(px(100.0)),
                        PreferredSizeOf::px(px(10.0)),
                    )),
                    margin: margins(7.0, 0.0),
                    ..ordinary.clone()
                };
                match case {
                    "larger_body_margin" => body.margin = margins(20.0, 0.0),
                    "signed" => empty.margin = margins(8.0, -12.0),
                    "own_positive" => parent.margin = margins(20.0, 0.0),
                    "own_negative" => parent.margin = margins(-4.0, 0.0),
                    "parent_border" => parent.border = edge,
                    "parent_padding" => parent.padding = edge,
                    "parent_independent" => {
                        parent.block_flow_context = BlockFlowContext::Independent
                    }
                    "child_border" => empty.border = edge,
                    "child_padding" => empty.padding = edge,
                    "child_minimum" => {
                        empty.min_size = axes.physical_size(LogicalSizeOf::new(
                            MinSizeOf::px(S::ZERO),
                            MinSizeOf::px(px(1.0)),
                        ))
                    }
                    "child_independent" => empty.block_flow_context = BlockFlowContext::Independent,
                    "parent_minimum" | "all_empty_minimum" => {
                        parent.min_size = axes.physical_size(LogicalSizeOf::new(
                            MinSizeOf::px(S::ZERO),
                            MinSizeOf::px(px(if case == "parent_minimum" { 1.0 } else { 5.0 })),
                        ))
                    }
                    _ => {}
                }
                let mut children = vec![3, 4];
                if case == "multiple" || case == "signed" {
                    children.insert(1, 5);
                }
                if case == "all_empty_minimum" {
                    children.pop();
                }
                let direct = case == "direct_root";
                let mut tree = PublicBlockTree::default()
                    .with_children(0, if direct { children.clone() } else { vec![2] })
                    .with_style(0, ordinary.clone())
                    .with_children(3, [])
                    .with_style(3, empty)
                    .with_children(4, [])
                    .with_style(4, body);
                if !direct {
                    tree = tree.with_children(2, children).with_style(2, parent);
                }
                if case == "multiple" || case == "signed" {
                    tree = tree.with_children(5, []).with_style(
                        5,
                        NodeInputOf {
                            margin: margins(15.0, if case == "signed" { -3.0 } else { 3.0 }),
                            ..ordinary.clone()
                        },
                    );
                }
                let batch = compute_layout(&tree, 0, request).unwrap();
                let root = public_final_output(&batch, 0);
                let parent = public_final_output(&batch, if direct { 0 } else { 2 });
                assert_eq!(
                    axes.logical_point(parent.location, parent.size, root.size)
                        .block,
                    px(y),
                    "{axes:?}, {case}: parent origin"
                );
                assert_eq!(
                    axes.logical_size(parent.size).block,
                    px(height),
                    "{axes:?}, {case}: parent height"
                );
                let empty = public_final_output(&batch, 3);
                assert_eq!(
                    axes.logical_point(empty.location, empty.size, parent.size)
                        .block,
                    px(empty_y),
                    "{axes:?}, {case}: empty origin"
                );
                assert_eq!(
                    axes.logical_size(empty.size).block,
                    px(empty_height),
                    "{axes:?}, {case}: empty height"
                );
                if case != "all_empty_minimum" {
                    let body = public_final_output(&batch, 4);
                    assert_eq!(
                        axes.logical_point(body.location, body.size, parent.size)
                            .block,
                        px(body_y),
                        "{axes:?}, {case}: body origin"
                    );
                    assert_eq!(axes.logical_size(body.size).block, px(10.0));
                }
            }
        }
    }
    lane::<f32>();
    lane::<f64>();
}

fn assert_leading_start_component_stays_outside_parent_end<S: LayoutScalar>() {
    let px = S::from_f64;
    let ordinary = NodeInputOf {
        display: Display::Block,
        block_flow_context: BlockFlowContext::Normal,
        size: Size::new(PreferredSizeOf::px(px(100.0)), PreferredSizeOf::AUTO),
        min_size: Size::new(MinSizeOf::px(S::ZERO), MinSizeOf::px(S::ZERO)),
        ..Default::default()
    };
    let width = NonNegativeFiniteOf::new(px(100.0)).unwrap();
    let request = crate::test_support::requests::NativeRootFrameOf::new(
        FlowAxes::new(WritingMode::HorizontalTb, Direction::Ltr),
        Size::new(Some(width), None),
        OrthogonalFallbackOf::new(
            Size::NONE,
            NearestScrollportOf::Absent,
            Size::splat(OrthogonalFallbackLimitOf::Unbounded),
        ),
    )
    .request(Size::new(
        AxisSizingRequestOf::new(AvailableSpaceOf::Definite(width), SizingOperation::Normal),
        AxisSizingRequestOf::new(AvailableSpaceOf::Indefinite, SizingOperation::Normal),
    ));
    for (case, minimum, top, bottom, parent_y, following_y, root_height) in [
        ("zero margins", 5.0, 0.0, 0.0, 0.0, 5.0, 15.0),
        ("through parent", 0.0, 8.0, 12.0, 12.0, 12.0, 22.0),
        ("separate parent end", 5.0, 8.0, 12.0, 12.0, 17.0, 27.0),
    ] {
        let tree = PublicBlockTree::default()
            .with_children(0, [2, 4])
            .with_children(2, [3])
            .with_children(3, [])
            .with_children(4, [])
            .with_style(0, ordinary.clone())
            .with_style(
                2,
                NodeInputOf {
                    min_size: Size::new(MinSizeOf::px(S::ZERO), MinSizeOf::px(px(minimum))),
                    ..ordinary.clone()
                },
            )
            .with_style(
                3,
                NodeInputOf {
                    margin: Edges {
                        top: LengthAutoOf::px(px(top)),
                        bottom: LengthAutoOf::px(px(bottom)),
                        ..Edges::all(LengthAutoOf::ZERO)
                    },
                    ..ordinary.clone()
                },
            )
            .with_style(
                4,
                NodeInputOf {
                    size: Size::new(
                        PreferredSizeOf::px(px(100.0)),
                        PreferredSizeOf::px(px(10.0)),
                    ),
                    ..ordinary.clone()
                },
            );
        let batch = compute_layout(&tree, 0, request).unwrap();
        let parent = public_final_output(&batch, 2);
        let empty = public_final_output(&batch, 3);
        let following = public_final_output(&batch, 4);
        let root = public_final_output(&batch, 0);
        assert_eq!(
            (parent.location, parent.size, parent.margin),
            (
                Point::new(S::ZERO, px(parent_y)),
                Size::new(px(100.0), px(minimum)),
                Edges::ZERO
            ),
            "{case}: parent"
        );
        assert_eq!(
            (empty.location, empty.size),
            (Point::ZERO, Size::new(px(100.0), S::ZERO)),
            "{case}: real empty child"
        );
        // The whole8/12 component adjoins P's top. A positive minimum
        // separates P's own boundaries, so that component is not also its end.
        assert_eq!(
            (following.location, following.size, root.size),
            (
                Point::new(S::ZERO, px(following_y)),
                Size::new(px(100.0), px(10.0)),
                Size::new(px(100.0), px(root_height))
            ),
            "{case}: following/root"
        );
    }
}

#[test]
fn leading_start_component_stays_outside_minimum_parent_end_f32() {
    assert_leading_start_component_stays_outside_parent_end::<f32>();
}
#[test]
fn leading_start_component_stays_outside_minimum_parent_end_f64() {
    assert_leading_start_component_stays_outside_parent_end::<f64>();
}

fn assert_leading_component_text_lines<S: LayoutScalar>(inside_empty: bool, supplied_strut: bool) {
    use crate::test_support::text::{ParagraphPart, PreparedParagraph, ahem_style};
    let px = S::from_f64;
    let ordinary = NodeInputOf {
        display: Display::Block,
        size: Size::new(PreferredSizeOf::px(px(100.0)), PreferredSizeOf::AUTO),
        min_size: Size::new(MinSizeOf::px(S::ZERO), MinSizeOf::px(S::ZERO)),
        ..Default::default()
    };
    let request = crate::test_support::requests::NativeRootFrameOf::new(
        FlowAxes::new(WritingMode::HorizontalTb, Direction::Ltr),
        Size::new(Some(NonNegativeFiniteOf::new(px(100.0)).unwrap()), None),
        OrthogonalFallbackOf::new(
            Size::NONE,
            NearestScrollportOf::Absent,
            Size::splat(OrthogonalFallbackLimitOf::Unbounded),
        ),
    )
    .request(Size::new(
        AxisSizingRequestOf::new(
            AvailableSpaceOf::Definite(NonNegativeFiniteOf::new(px(100.0)).unwrap()),
            SizingOperation::Normal,
        ),
        AxisSizingRequestOf::new(AvailableSpaceOf::Indefinite, SizingOperation::Normal),
    ));
    for visible in [true, false] {
        let style = ahem_style(4.0, 4.0);
        let paragraph = PreparedParagraph::with_strut_at_boundary(
            vec![ParagraphPart::text(
                5,
                surgeist_text::Source::new(if visible { "X" } else { "" }),
                style.clone(),
            )],
            style,
            surgeist_text::ParagraphBaseDirection::LeftToRight,
            surgeist_text::ParagraphStrut::DefaultFont,
            Vec::new(),
            if inside_empty {
                surgeist_text::ParagraphTerminalEdge::BlockEnd
            } else {
                surgeist_text::ParagraphTerminalEdge::NonBlockBoundary
            },
        );
        let tree = PublicBlockTree::default()
            .with_children(0, [2])
            .with_children(
                2,
                if inside_empty {
                    vec![3, 4]
                } else {
                    vec![3, 5, 4]
                },
            )
            .with_children(3, if inside_empty { vec![5] } else { Vec::new() })
            .with_children(4, [])
            .with_children(5, [])
            .with_style(0, ordinary.clone())
            .with_style(2, ordinary.clone())
            .with_style(
                3,
                NodeInputOf {
                    margin: Edges {
                        top: LengthAutoOf::px(px(8.0)),
                        bottom: LengthAutoOf::px(px(12.0)),
                        ..Edges::all(LengthAutoOf::ZERO)
                    },
                    ..ordinary.clone()
                },
            )
            .with_style(
                4,
                NodeInputOf {
                    size: Size::new(
                        PreferredSizeOf::px(px(100.0)),
                        PreferredSizeOf::px(px(10.0)),
                    ),
                    margin: Edges {
                        top: LengthAutoOf::px(px(7.0)),
                        ..Edges::all(LengthAutoOf::ZERO)
                    },
                    ..ordinary.clone()
                },
            )
            .with_style(5, NodeInputOf::non_box())
            .with_layout_input(5, paragraph.text_input(5));
        let mut binding = paragraph.binding(if inside_empty { 3 } else { 2 });
        if supplied_strut {
            binding.line_strut = Some(surgeist_text::LineStrut {
                baseline: 3.0,
                line_extent: 4.0,
            });
        }
        let capture = LayoutFlowInput::try_new(
            &tree,
            0,
            vec![binding],
            WrappingInput::default(),
            Vec::new(),
        )
        .unwrap();
        let batch = compute_layout_with_flow(&capture, 0, request).unwrap();
        let [placed] = batch.placed_text() else {
            panic!("one actual Text layout belongs to the paragraph");
        };
        assert_eq!(placed.layout().source().text().is_empty(), !visible);
        assert_eq!(placed.layout().lines().len(), usize::from(visible));
        if !visible {
            let completions = placed.layout().nonformatted_lines().collect::<Vec<_>>();
            assert_eq!(completions.len(), 1);
            let receipt = completions[0].line.finalized_line();
            assert!(!receipt.has_in_flow_content());
            // The captured font contributes 3.2/.8; the supplied canonical
            // strut contributes 3/1. Their union is 4.2, still nonformatted.
            assert_eq!(
                receipt.feedback().line_height,
                if supplied_strut { 4.2 } else { 4.0 }
            );
            assert_eq!(receipt.prepared_text(), paragraph.prepared);
            let allocation = receipt.selected_line().request().line_strut;
            assert_eq!(allocation.is_some(), supplied_strut);
            if let Some(allocation) = allocation {
                assert_eq!(allocation.line_extent, 4.0);
            }
        }
        let (parent_y, parent_height, empty_height, body_y) = if !visible {
            (12.0, 10.0, 0.0, 0.0)
        } else if inside_empty {
            (8.0, 26.0, 4.0, 16.0)
        } else {
            (12.0, 21.0, 0.0, 11.0)
        };
        let parent = public_final_output(&batch, 2);
        let empty = public_final_output(&batch, 3);
        let body = public_final_output(&batch, 4);
        assert_eq!(
            (
                parent.location.y,
                parent.size.height,
                empty.location.y,
                empty.size.height,
                body.location.y
            ),
            (
                px(parent_y),
                px(parent_height),
                S::ZERO,
                px(empty_height),
                px(body_y)
            ),
            "text visible {visible}, inside empty {inside_empty}"
        );
        assert_eq!(batch.unrounded_inline_fragments().is_empty(), !visible);
    }
}

#[test]
fn leading_component_text_inside_empty_box_f32() {
    assert_leading_component_text_lines::<f32>(true, false);
}
#[test]
fn leading_component_text_inside_empty_box_f64() {
    assert_leading_component_text_lines::<f64>(true, false);
}
#[test]
fn leading_component_text_between_empty_and_body_f32() {
    assert_leading_component_text_lines::<f32>(false, false);
}
#[test]
fn leading_component_text_between_empty_and_body_f64() {
    assert_leading_component_text_lines::<f64>(false, false);
}

fn assert_empty_origin_with_participating_text_control<S: LayoutScalar>() {
    use crate::test_support::text::{ParagraphPart, PreparedParagraph, ahem_style};
    let px = S::from_f64;
    for kind in 0..3 {
        let style = ahem_style(4.0, 4.0);
        let participant = match kind {
            0 => ParagraphPart::Control {
                node: 3,
                kind: surgeist_text::ParagraphControlKind::ForcedBreak,
            },
            1 => ParagraphPart::Atomic {
                node: 3,
                size: surgeist_text::Size::new(0.0, 0.0),
            },
            _ => ParagraphPart::Control {
                node: 3,
                kind: surgeist_text::ParagraphControlKind::InlineBoundaryStart,
            },
        };
        let paragraph = PreparedParagraph::with_strut_at_boundary(
            vec![
                ParagraphPart::text(2, surgeist_text::Source::new(""), style.clone()),
                participant,
            ],
            style,
            surgeist_text::ParagraphBaseDirection::LeftToRight,
            surgeist_text::ParagraphStrut::DefaultFont,
            Vec::new(),
            surgeist_text::ParagraphTerminalEdge::NonBlockBoundary,
        );
        let ordinary = NodeInputOf {
            display: Display::Block,
            size: Size::new(PreferredSizeOf::px(px(100.0)), PreferredSizeOf::AUTO),
            min_size: Size::new(MinSizeOf::px(S::ZERO), MinSizeOf::px(S::ZERO)),
            ..Default::default()
        };
        let mut tree = PublicBlockTree::default()
            .with_children(0, [1, 2, 3, 4])
            .with_children(1, [])
            .with_children(2, [])
            .with_children(3, [])
            .with_children(4, [])
            .with_style(0, ordinary.clone())
            .with_style(
                1,
                NodeInputOf {
                    size: Size::new(
                        PreferredSizeOf::px(px(100.0)),
                        PreferredSizeOf::px(px(10.0)),
                    ),
                    margin: Edges {
                        bottom: LengthAutoOf::px(px(12.0)),
                        ..Edges::all(LengthAutoOf::ZERO)
                    },
                    ..ordinary.clone()
                },
            )
            .with_style(2, NodeInputOf::non_box())
            .with_layout_input(2, paragraph.text_input(2))
            .with_style(3, NodeInputOf::non_box())
            .with_style(
                4,
                NodeInputOf {
                    size: Size::new(
                        PreferredSizeOf::px(px(100.0)),
                        PreferredSizeOf::px(px(10.0)),
                    ),
                    margin: Edges {
                        top: LengthAutoOf::px(px(7.0)),
                        ..Edges::all(LengthAutoOf::ZERO)
                    },
                    ..ordinary
                },
            );
        let metrics = InlineMetricsOf::from_ascent_descent(
            crate::InlineBaselineContext::Physical,
            px(3.0),
            px(1.0),
        )
        .unwrap();
        tree = match kind {
            0 => tree.with_layout_input(
                3,
                LayoutInputOf::line_break(LineBreakInputOf::new().with_metrics(metrics)),
            ),
            1 => tree.with_style(
                3,
                NodeInputOf {
                    display: Display::InlineBlock,
                    size: Size::new(PreferredSizeOf::px(S::ZERO), PreferredSizeOf::px(S::ZERO)),
                    min_size: Size::new(MinSizeOf::px(S::ZERO), MinSizeOf::px(S::ZERO)),
                    ..Default::default()
                },
            ),
            _ => tree.with_layout_input(
                3,
                LayoutInputOf::inline_boundary(
                    InlineBoundaryInputOf::contribution(
                        InlineBoundaryKind::Start,
                        metrics,
                        crate::InlineBoundaryEdgeOf::try_new(S::ZERO, px(1.0), S::ZERO).unwrap(),
                    )
                    .unwrap(),
                ),
            ),
        };
        let capture = LayoutFlowInput::try_new(
            &tree,
            0,
            vec![paragraph.binding(0)],
            WrappingInput::default(),
            Vec::new(),
        )
        .unwrap();
        let request = crate::test_support::requests::NativeRootFrameOf::new(
            FlowAxes::new(WritingMode::HorizontalTb, Direction::Ltr),
            Size::new(Some(NonNegativeFiniteOf::new(px(100.0)).unwrap()), None),
            OrthogonalFallbackOf::new(
                Size::NONE,
                NearestScrollportOf::Absent,
                Size::splat(OrthogonalFallbackLimitOf::Unbounded),
            ),
        )
        .request(Size::new(
            AxisSizingRequestOf::new(
                AvailableSpaceOf::Definite(NonNegativeFiniteOf::new(px(100.0)).unwrap()),
                SizingOperation::Normal,
            ),
            AxisSizingRequestOf::new(AvailableSpaceOf::Indefinite, SizingOperation::Normal),
        ));
        let batch = compute_layout_with_flow(&capture, 0, request).unwrap();
        // Empty source membership cannot erase a real forced, atomic, or
        // border-bearing line. The line consumes 12 before and 7 after it.
        assert_eq!(
            batch.placed_text()[0].layout().lines().len(),
            1,
            "kind {kind}"
        );
        assert_eq!(public_final_output(&batch, 1).location.y, S::ZERO);
        assert_eq!(
            public_final_output(&batch, 4).location.y,
            px(33.0),
            "kind {kind}"
        );
        assert_eq!(
            public_final_output(&batch, 0).size.height,
            px(43.0),
            "kind {kind}"
        );
    }
}

#[test]
fn empty_text_origin_preserves_participating_controls_f32() {
    assert_empty_origin_with_participating_text_control::<f32>();
}
#[test]
fn empty_text_origin_preserves_participating_controls_f64() {
    assert_empty_origin_with_participating_text_control::<f64>();
}

#[test]
fn leading_component_text_supplied_strut_inside_empty_box_f32() {
    assert_leading_component_text_lines::<f32>(true, true);
}
#[test]
fn leading_component_text_supplied_strut_inside_empty_box_f64() {
    assert_leading_component_text_lines::<f64>(true, true);
}
#[test]
fn leading_component_text_supplied_strut_between_empty_and_body_f32() {
    assert_leading_component_text_lines::<f32>(false, true);
}
#[test]
fn leading_component_text_supplied_strut_between_empty_and_body_f64() {
    assert_leading_component_text_lines::<f64>(false, true);
}

#[derive(Clone, Copy)]
enum ProcessedBlockEntry {
    LeadingEmpty,
    Positive,
    Zero,
    Negative,
}

fn assert_processed_whitespace_preserves_block_adjacency<S: LayoutScalar>(
    entry: ProcessedBlockEntry,
) {
    let prefix_margin = match entry {
        ProcessedBlockEntry::LeadingEmpty => None,
        ProcessedBlockEntry::Positive => Some(12.0),
        ProcessedBlockEntry::Zero => Some(0.0),
        ProcessedBlockEntry::Negative => Some(-12.0),
    };
    let px = S::from_f64;
    for visible in [true, false] {
        let value = if visible { "X" } else { " " };
        // This fixture performs contextual source processing before analysis
        // and prepares the retained source with registered Ahem10 metrics.
        let fixture = crate::test_support::inline_endpoint::Fixture::new(
            value,
            (0, 1),
            surgeist_text::WhiteSpace::Collapse,
            false,
            false,
        );
        assert!(fixture.binding.prepared.captured_owner().is_some());
        let ordinary = NodeInputOf {
            display: Display::Block,
            size: Size::new(PreferredSizeOf::px(px(100.0)), PreferredSizeOf::AUTO),
            min_size: Size::new(MinSizeOf::px(S::ZERO), MinSizeOf::px(S::ZERO)),
            ..Default::default()
        };
        let tree = fixture
            .tree::<S>(
                InlineBoundaryEdgeOf::ZERO,
                InlineBoundaryEdgeOf::ZERO,
                false,
                Some(100.0),
                false,
            )
            .children(100, [50])
            .style(100, ordinary.clone())
            .children(50, [0, 51])
            .style(50, ordinary.clone())
            .style(
                0,
                NodeInputOf {
                    margin: Edges {
                        top: LengthAutoOf::px(px(8.0)),
                        bottom: LengthAutoOf::px(px(12.0)),
                        ..Edges::all(LengthAutoOf::ZERO)
                    },
                    ..ordinary.clone()
                },
            )
            .children(51, [])
            .style(
                51,
                NodeInputOf {
                    size: Size::new(
                        PreferredSizeOf::px(px(100.0)),
                        PreferredSizeOf::px(px(10.0)),
                    ),
                    margin: Edges {
                        top: LengthAutoOf::px(px(7.0)),
                        ..Edges::all(LengthAutoOf::ZERO)
                    },
                    ..ordinary.clone()
                },
            );
        let tree = if let Some(margin) = prefix_margin {
            tree.children(
                0,
                std::iter::once(52).chain(fixture.binding.nodes.iter().copied()),
            )
            .style(0, ordinary.clone())
            .children(52, [])
            .style(
                52,
                NodeInputOf {
                    size: Size::new(
                        PreferredSizeOf::px(px(100.0)),
                        PreferredSizeOf::px(px(10.0)),
                    ),
                    margin: Edges {
                        bottom: LengthAutoOf::px(px(margin)),
                        ..Edges::all(LengthAutoOf::ZERO)
                    },
                    ..ordinary
                },
            )
        } else {
            tree
        };
        let capture = LayoutFlowInput::try_new(
            &tree,
            100,
            vec![fixture.binding.clone()],
            WrappingInput::default(),
            Vec::new(),
        )
        .unwrap();
        let request = crate::test_support::requests::NativeRootFrameOf::new(
            FlowAxes::new(WritingMode::HorizontalTb, Direction::Ltr),
            Size::new(Some(NonNegativeFiniteOf::new(px(100.0)).unwrap()), None),
            OrthogonalFallbackOf::new(
                Size::NONE,
                NearestScrollportOf::Absent,
                Size::splat(OrthogonalFallbackLimitOf::Unbounded),
            ),
        )
        .request(Size::new(
            AxisSizingRequestOf::new(
                AvailableSpaceOf::Definite(NonNegativeFiniteOf::new(px(100.0)).unwrap()),
                SizingOperation::Normal,
            ),
            AxisSizingRequestOf::new(AvailableSpaceOf::Indefinite, SizingOperation::Normal),
        ));
        let batch = compute_layout_with_flow(&capture, 100, request).unwrap();
        if prefix_margin.is_some() {
            let prefix = public_final_output(&batch, 52);
            assert_eq!(prefix.location.y, S::ZERO);
            assert_eq!(prefix.size.height, px(10.0));
        }
        let [placed] = batch.placed_text() else {
            panic!("one actual processed paragraph");
        };
        assert_eq!(
            placed.layout().source().text(),
            &surgeist_text::Text::from(value)
        );
        assert_eq!(placed.layout().lines().len(), usize::from(visible));
        if !visible {
            let completions = placed.layout().nonformatted_lines().collect::<Vec<_>>();
            assert_eq!(completions.len(), 1);
            let receipt = completions[0].line.finalized_line();
            assert_eq!(receipt.prepared_text(), fixture.binding.prepared);
            assert!(!receipt.has_in_flow_content());
            assert_eq!(receipt.feedback().line_height, 10.0);
            let selected = receipt.selected_line();
            assert_eq!(selected.request().controls.len(), 2);
            assert!(selected.request().controls.iter().all(|control| {
                control
                    .boundary
                    .as_ref()
                    .is_some_and(|edge| !edge.participates())
            }));
        }
        // A real10px line blocks E's own margins from adjoining through it.
        // Its12px bottom then separates B. A fully discarded line leaves
        // E8/12 and B7 in the one adjoining start group12.
        let (parent_y, height, empty_height, body_y, root_height) = match (entry, visible) {
            (ProcessedBlockEntry::LeadingEmpty, true) => (8.0, 32.0, 10.0, 22.0, 40.0),
            (ProcessedBlockEntry::LeadingEmpty, false) => (12.0, 10.0, 0.0, 0.0, 22.0),
            (ProcessedBlockEntry::Positive, true) => (0.0, 49.0, 32.0, 39.0, 49.0),
            (ProcessedBlockEntry::Positive, false) => (0.0, 32.0, 10.0, 22.0, 32.0),
            (ProcessedBlockEntry::Zero, true) => (0.0, 37.0, 20.0, 27.0, 37.0),
            (ProcessedBlockEntry::Zero, false) => (0.0, 27.0, 10.0, 17.0, 27.0),
            (ProcessedBlockEntry::Negative, true) => (0.0, 25.0, 8.0, 15.0, 25.0),
            (ProcessedBlockEntry::Negative, false) => (0.0, 15.0, 10.0, 5.0, 15.0),
        };
        let parent = public_final_output(&batch, 50);
        let empty = public_final_output(&batch, 0);
        let body = public_final_output(&batch, 51);
        let root = public_final_output(&batch, 100);
        assert_eq!(
            (
                parent.location.y,
                parent.size.height,
                empty.location.y,
                empty.size.height,
                body.location.y,
                root.size.height
            ),
            (
                px(parent_y),
                px(height),
                S::ZERO,
                px(empty_height),
                px(body_y),
                px(root_height)
            ),
            "processed text visible {visible}"
        );
    }
}

#[test]
fn processed_whitespace_preserves_block_adjacency_f32() {
    assert_processed_whitespace_preserves_block_adjacency::<f32>(ProcessedBlockEntry::LeadingEmpty);
}
#[test]
fn processed_whitespace_preserves_block_adjacency_f64() {
    assert_processed_whitespace_preserves_block_adjacency::<f64>(ProcessedBlockEntry::LeadingEmpty);
}

#[test]
fn processed_whitespace_preserves_positive_adjoining_displacement_f32() {
    assert_processed_whitespace_preserves_block_adjacency::<f32>(ProcessedBlockEntry::Positive);
}

#[test]
fn processed_whitespace_preserves_positive_adjoining_displacement_f64() {
    assert_processed_whitespace_preserves_block_adjacency::<f64>(ProcessedBlockEntry::Positive);
}

#[test]
fn processed_whitespace_preserves_zero_adjoining_displacement_f32() {
    assert_processed_whitespace_preserves_block_adjacency::<f32>(ProcessedBlockEntry::Zero);
}

#[test]
fn processed_whitespace_preserves_zero_adjoining_displacement_f64() {
    assert_processed_whitespace_preserves_block_adjacency::<f64>(ProcessedBlockEntry::Zero);
}

#[test]
fn processed_whitespace_preserves_negative_adjoining_displacement_f32() {
    assert_processed_whitespace_preserves_block_adjacency::<f32>(ProcessedBlockEntry::Negative);
}

#[test]
fn processed_whitespace_preserves_negative_adjoining_displacement_f64() {
    assert_processed_whitespace_preserves_block_adjacency::<f64>(ProcessedBlockEntry::Negative);
}

fn entry_progress_request<S: LayoutScalar>(width: S) -> LayoutRootRequestOf<S> {
    crate::test_support::requests::NativeRootFrameOf::new(
        FlowAxes::new(WritingMode::HorizontalTb, Direction::Ltr),
        Size::new(Some(NonNegativeFiniteOf::new(width).unwrap()), None),
        OrthogonalFallbackOf::new(
            Size::NONE,
            NearestScrollportOf::Absent,
            Size::splat(OrthogonalFallbackLimitOf::Unbounded),
        ),
    )
    .request(Size::new(
        AxisSizingRequestOf::new(
            AvailableSpaceOf::Definite(NonNegativeFiniteOf::new(width).unwrap()),
            SizingOperation::Normal,
        ),
        AxisSizingRequestOf::new(AvailableSpaceOf::Indefinite, SizingOperation::Normal),
    ))
}

fn assert_entry_displacement_under_line_selection<S: LayoutScalar>() {
    let px = S::from_f64;
    for policy in [
        TextWrapStyle::Auto,
        TextWrapStyle::Balance,
        TextWrapStyle::Pretty,
        TextWrapStyle::AvoidShortLastLine,
    ] {
        for (margin, float) in [(12.0, false), (-12.0, false), (12.0, true)] {
            let fixture = crate::test_support::inline_endpoint::Fixture::new(
                "A B C D",
                (0, 7),
                surgeist_text::WhiteSpace::Collapse,
                false,
                false,
            );
            let ordinary = NodeInputOf {
                display: Display::Block,
                size: Size::new(PreferredSizeOf::px(px(30.0)), PreferredSizeOf::AUTO),
                min_size: Size::new(MinSizeOf::px(S::ZERO), MinSizeOf::px(S::ZERO)),
                ..Default::default()
            };
            let mut binding = fixture.binding.clone();
            if float {
                // The float is the first native event in the actual captured
                // window, before any allocated inline control or Text origin.
                binding.nodes.insert(0, 2);
            }
            let mut children = vec![52];
            children.extend(binding.nodes.iter().copied());
            let tree = fixture
                .tree::<S>(
                    InlineBoundaryEdgeOf::ZERO,
                    InlineBoundaryEdgeOf::ZERO,
                    false,
                    Some(30.0),
                    float,
                )
                .children(100, [0])
                .style(100, ordinary.clone())
                .children(0, children)
                .style(
                    0,
                    NodeInputOf {
                        text_wrap_style: policy,
                        ..ordinary.clone()
                    },
                )
                .children(52, [])
                .style(
                    52,
                    NodeInputOf {
                        size: Size::new(
                            PreferredSizeOf::px(px(30.0)),
                            PreferredSizeOf::px(px(10.0)),
                        ),
                        margin: Edges {
                            bottom: LengthAutoOf::px(px(margin)),
                            ..Edges::all(LengthAutoOf::ZERO)
                        },
                        ..ordinary
                    },
                );
            // Float replay is qualified in its actual owning root BFC.
            // The ordinary nested case separately exercises shared entry.
            let root = if float { 0 } else { 100 };
            let capture = LayoutFlowInput::try_new(
                &tree,
                root,
                vec![binding],
                WrappingInput::default(),
                Vec::new(),
            )
            .unwrap();
            let batch =
                compute_layout_with_flow(&capture, root, entry_progress_request(px(30.0))).unwrap();
            let layout = batch.placed_text()[0].layout();
            let lines = layout.lines();
            assert_eq!(
                lines.len(),
                2,
                "policy {policy:?},margin {margin},float {float}"
            );
            // Each Ahem word is10px; 30px admits exactly two words per line.
            // The25px float leaves5px, so the first word waits its20px height.
            let first = 10.0 + margin + if float { 20.0 } else { 0.0 };
            assert_eq!(
                lines
                    .iter()
                    .map(|line| line.bounds().origin.y)
                    .collect::<Vec<_>>(),
                vec![first, first + 10.0]
            );
            assert_eq!(layout.metrics().size().height, 20.0);
            assert_eq!(layout.bounds().origin.y, first);
            assert_eq!(layout.bounds().size.height, 20.0);
            assert_eq!(public_final_output(&batch, 0).size.height, px(first + 20.0));
            assert_eq!(
                public_final_output(&batch, root).size.height,
                px(first + 20.0)
            );
            if float {
                assert_eq!(public_final_output(&batch, 2).location.y, px(22.0));
                assert_eq!(public_final_output(&batch, 2).size.height, px(20.0));
            }
        }
    }
}

#[test]
fn entry_displacement_survives_quality_multiline_and_float_replay_f32() {
    assert_entry_displacement_under_line_selection::<f32>();
}
#[test]
fn entry_displacement_survives_quality_multiline_and_float_replay_f64() {
    assert_entry_displacement_under_line_selection::<f64>();
}

#[test]
fn numeric_entry_progress_preserves_f32_addition_association() {
    let ordinary = NodeInputOf::<f32> {
        display: Display::Block,
        size: Size::new(PreferredSizeOf::px(100.0), PreferredSizeOf::AUTO),
        min_size: Size::new(MinSizeOf::px(0.0), MinSizeOf::px(0.0)),
        ..Default::default()
    };
    let tree = PublicBlockTree::default()
        .with_children(0, [1, 2])
        .with_children(1, [])
        .with_children(2, [])
        .with_style(0, ordinary.clone())
        .with_style(
            1,
            NodeInputOf {
                size: Size::new(
                    PreferredSizeOf::px(100.0),
                    PreferredSizeOf::px(33_554_432.0),
                ),
                margin: Edges {
                    bottom: LengthAutoOf::px(-33_554_432.0),
                    ..Edges::all(LengthAutoOf::ZERO)
                },
                ..ordinary.clone()
            },
        )
        .with_style(
            2,
            NodeInputOf {
                display: Display::InlineBlock,
                size: Size::new(PreferredSizeOf::px(1.0), PreferredSizeOf::px(1.0)),
                atomic_inline_participation: Some(fri06_atomic_participation()),
                ..ordinary
            },
        );
    let batch = compute_layout(&tree, 0, entry_progress_request(100.0)).unwrap();
    assert_eq!(public_final_output(&batch, 1).size.height, 33_554_432.0);
    assert_eq!(public_final_output(&batch, 2).location.y, 0.0);
    assert_eq!(public_final_output(&batch, 2).size.height, 1.0);
    assert_eq!(public_final_output(&batch, 0).size.height, 1.0);
}

fn assert_zero_height_forced_line_consumes_entry<S: LayoutScalar>() {
    use crate::test_support::text::{ParagraphPart, PreparedParagraph, ahem_style};
    let px = S::from_f64;
    let style = ahem_style(10.0, 10.0);
    let paragraph = PreparedParagraph::with_strut_at_boundary(
        vec![ParagraphPart::Control {
            node: 20,
            kind: surgeist_text::ParagraphControlKind::ForcedBreak,
        }],
        style,
        surgeist_text::ParagraphBaseDirection::LeftToRight,
        surgeist_text::ParagraphStrut::AllocatedContributions,
        Vec::new(),
        surgeist_text::ParagraphTerminalEdge::BlockEnd,
    );
    let ordinary = NodeInputOf {
        display: Display::Block,
        size: Size::new(PreferredSizeOf::px(px(100.0)), PreferredSizeOf::AUTO),
        min_size: Size::new(MinSizeOf::px(S::ZERO), MinSizeOf::px(S::ZERO)),
        ..Default::default()
    };
    let tree = PublicBlockTree::default()
        .with_children(100, [0])
        .with_children(0, [52, 20])
        .with_children(52, [])
        .with_children(20, [])
        .with_style(100, ordinary.clone())
        .with_style(0, ordinary.clone())
        .with_style(
            52,
            NodeInputOf {
                size: Size::new(
                    PreferredSizeOf::px(px(100.0)),
                    PreferredSizeOf::px(px(10.0)),
                ),
                margin: Edges {
                    bottom: LengthAutoOf::px(px(12.0)),
                    ..Edges::all(LengthAutoOf::ZERO)
                },
                ..ordinary
            },
        )
        .with_style(20, NodeInputOf::non_box())
        .with_layout_input(
            20,
            LayoutInputOf::line_break(
                LineBreakInputOf::new().with_metrics(
                    InlineMetricsOf::from_ascent_descent(
                        crate::InlineBaselineContext::Physical,
                        S::ZERO,
                        S::ZERO,
                    )
                    .unwrap(),
                ),
            ),
        );
    let mut binding = paragraph.binding(0);
    binding.line_strut = Some(surgeist_text::LineStrut {
        baseline: 0.0,
        line_extent: 0.0,
    });
    let capture = LayoutFlowInput::try_new(
        &tree,
        100,
        vec![binding],
        WrappingInput::default(),
        Vec::new(),
    )
    .unwrap();
    let batch = compute_layout_with_flow(&capture, 100, entry_progress_request(px(100.0))).unwrap();
    let layout = batch.placed_text()[0].layout();
    assert_eq!(layout.lines().len(), 1);
    assert_eq!(layout.lines()[0].bounds().size.height, 0.0);
    assert_eq!(layout.lines()[0].baseline(), 22.0);
    assert_eq!(layout.nonformatted_lines().count(), 1);
    assert_eq!(public_final_output(&batch, 0).size.height, px(22.0));
    assert_eq!(public_final_output(&batch, 100).size.height, px(22.0));
}
#[test]
fn zero_height_forced_line_consumes_entry_before_phantom_receiver_f32() {
    assert_zero_height_forced_line_consumes_entry::<f32>();
}
#[test]
fn zero_height_forced_line_consumes_entry_before_phantom_receiver_f64() {
    assert_zero_height_forced_line_consumes_entry::<f64>();
}

fn assert_nonformatted_capture_retains_float_entry<S: LayoutScalar>() {
    let px = S::from_f64;
    for visible in [true, false] {
        let value = if visible { "A B C D" } else { " " };
        let fixture = crate::test_support::inline_endpoint::Fixture::new(
            value,
            (0, value.len()),
            surgeist_text::WhiteSpace::Collapse,
            false,
            false,
        );
        assert!(fixture.binding.prepared.captured_owner().is_some());
        let mut binding = fixture.binding.clone();
        binding.nodes.insert(0, 2);
        let ordinary = NodeInputOf {
            display: Display::Block,
            size: Size::new(PreferredSizeOf::px(px(30.0)), PreferredSizeOf::AUTO),
            min_size: Size::new(MinSizeOf::px(S::ZERO), MinSizeOf::px(S::ZERO)),
            ..Default::default()
        };
        let tree = fixture
            .tree::<S>(
                InlineBoundaryEdgeOf::ZERO,
                InlineBoundaryEdgeOf::ZERO,
                false,
                Some(30.0),
                true,
            )
            .children(0, std::iter::once(52).chain(binding.nodes.iter().copied()))
            .style(0, ordinary.clone())
            .children(52, [])
            .style(
                52,
                NodeInputOf {
                    size: Size::new(PreferredSizeOf::px(px(30.0)), PreferredSizeOf::px(px(10.0))),
                    margin: Edges {
                        bottom: LengthAutoOf::px(px(12.0)),
                        ..Edges::all(LengthAutoOf::ZERO)
                    },
                    ..ordinary
                },
            );
        let capture = LayoutFlowInput::try_new(
            &tree,
            0,
            vec![binding],
            WrappingInput::default(),
            Vec::new(),
        )
        .unwrap();
        let batch =
            compute_layout_with_flow(&capture, 0, entry_progress_request(px(30.0))).unwrap();
        assert_eq!(public_final_output(&batch, 52).location.y, S::ZERO);
        assert_eq!(public_final_output(&batch, 52).size.height, px(10.0));
        let [placed] = batch.placed_text() else {
            panic!("one actual processed capture");
        };
        let layout = placed.layout();
        assert_eq!(layout.source().text(), &surgeist_text::Text::from(value));
        if visible {
            assert_eq!(
                layout
                    .lines()
                    .iter()
                    .map(|line| line.bounds().origin.y)
                    .collect::<Vec<_>>(),
                vec![42.0, 52.0]
            );
        } else {
            assert!(layout.lines().is_empty());
            let completions = layout.nonformatted_lines().collect::<Vec<_>>();
            assert_eq!(completions.len(), 1);
            let receipt = completions[0].line.finalized_line();
            assert_eq!(receipt.prepared_text(), fixture.binding.prepared);
            assert!(!receipt.has_in_flow_content());
            assert_eq!(receipt.feedback().line_height, 10.0);
            let selected = receipt.selected_line();
            assert_eq!(selected.request().controls.len(), 2);
            assert!(selected.request().controls.iter().all(|control| {
                control
                    .boundary
                    .as_ref()
                    .is_some_and(|edge| !edge.participates())
            }));
        }
        let float = public_final_output(&batch, 2);
        assert_eq!(float.size, Size::new(px(25.0), px(20.0)));
        // The float's hypothetical empty parent follows A's10px border end
        // plus its12px adjoining margin, independently of line existence.
        assert_eq!(float.location.y, px(22.0), "visible {visible}");
        assert_eq!(
            public_final_output(&batch, 0).size.height,
            px(if visible { 62.0 } else { 42.0 })
        );
    }
}

#[test]
fn nonformatted_capture_retains_float_adjoining_entry_f32() {
    assert_nonformatted_capture_retains_float_entry::<f32>();
}
#[test]
fn nonformatted_capture_retains_float_adjoining_entry_f64() {
    assert_nonformatted_capture_retains_float_entry::<f64>();
}

fn adjoining_float_style<S: LayoutScalar>(
    axes: FlowAxes,
    inline: f64,
    block: Option<f64>,
    start: f64,
    end: f64,
) -> NodeInputOf<S> {
    use crate::geometry::{LogicalEdgesOf, LogicalSizeOf};
    let px = S::from_f64;
    NodeInputOf {
        display: Display::Block,
        writing_mode: axes.writing_mode(),
        direction: axes.direction(),
        size: axes.physical_size(LogicalSizeOf::new(
            PreferredSizeOf::px(px(inline)),
            block.map_or(PreferredSizeOf::AUTO, |value| {
                PreferredSizeOf::px(px(value))
            }),
        )),
        min_size: Size::new(MinSizeOf::px(S::ZERO), MinSizeOf::px(S::ZERO)),
        margin: axes
            .physical_edges(LogicalEdgesOf::new(S::ZERO, S::ZERO, px(start), px(end)))
            .map(LengthAutoOf::px),
        ..Default::default()
    }
}

fn adjoining_float_request<S: LayoutScalar>(axes: FlowAxes, inline: f64) -> LayoutRootRequestOf<S> {
    use crate::geometry::LogicalSizeOf;
    let inline = NonNegativeFiniteOf::new(S::from_f64(inline)).unwrap();
    crate::test_support::requests::NativeRootFrameOf::new(
        axes,
        axes.physical_size(LogicalSizeOf::new(Some(inline), None)),
        OrthogonalFallbackOf::new(
            Size::NONE,
            NearestScrollportOf::Absent,
            Size::splat(OrthogonalFallbackLimitOf::Unbounded),
        ),
    )
    .request(axes.physical_size(LogicalSizeOf::new(
        AxisSizingRequestOf::new(AvailableSpaceOf::Definite(inline), SizingOperation::Normal),
        AxisSizingRequestOf::new(AvailableSpaceOf::Indefinite, SizingOperation::Normal),
    )))
}

fn assert_ordinary_float_internal_margin<S: LayoutScalar>() {
    let px = S::from_f64;
    // The zero-margin control runs in every flow before any missing-margin case.
    for (margin, following, float_top, body_top, extent) in [
        (0.0, 0.0, 10.0, 10.0, 30.0),
        (12.0, 0.0, 22.0, 22.0, 42.0),
        (12.0, 20.0, 22.0, 30.0, 42.0),
    ] {
        for (writing_mode, direction) in all_writing_mode_directions() {
            let axes = FlowAxes::new(writing_mode, direction);
            let tree = PublicBlockTree::default()
                .with_children(0, [1, 2, 3])
                .with_children(1, [])
                .with_children(2, [])
                .with_children(3, [])
                .with_style(0, adjoining_float_style(axes, 100.0, None, 0.0, 0.0))
                .with_style(
                    1,
                    adjoining_float_style(axes, 100.0, Some(10.0), 0.0, margin),
                )
                .with_style(
                    2,
                    NodeInputOf {
                        float: Float::Left,
                        ..adjoining_float_style(axes, 25.0, Some(20.0), 0.0, 0.0)
                    },
                )
                .with_style(
                    3,
                    adjoining_float_style(axes, 100.0, Some(10.0), following, 0.0),
                );
            let batch = compute_layout(&tree, 0, adjoining_float_request(axes, 100.0)).unwrap();
            let root = public_final_output(&batch, 0);
            let before = public_final_output(&batch, 1);
            let float = public_final_output(&batch, 2);
            let body = public_final_output(&batch, 3);
            assert_eq!(axes.logical_size(before.size).block, px(10.0));
            assert_eq!(axes.logical_size(float.size).block, px(20.0));
            assert_eq!(axes.logical_size(body.size).block, px(10.0));
            assert_eq!(float.source_index, SourceIndex::new(1));
            assert_eq!(body.source_index, SourceIndex::new(2));
            // Internal anonymous-parent border uses preceding margin12 and its
            // own zero top, not the future sibling's larger20px margin.
            assert_eq!(
                (
                    axes.logical_point(float.location, float.size, root.size)
                        .block,
                    axes.logical_point(body.location, body.size, root.size)
                        .block,
                    axes.logical_size(root.size).block
                ),
                (px(float_top), px(body_top), px(extent)),
                "{writing_mode:?}/{direction:?}, margins {margin}/{following}",
            );
        }
    }
}

#[test]
fn ordinary_float_adjoining_internal_margin_f32() {
    assert_ordinary_float_internal_margin::<f32>();
}
#[test]
fn ordinary_float_adjoining_internal_margin_f64() {
    assert_ordinary_float_internal_margin::<f64>();
}

fn assert_ordinary_float_leading_boundaries<S: LayoutScalar>() {
    use crate::geometry::LogicalEdgesOf;
    let px = S::from_f64;
    // Exported start is a control: the anonymous parent shares P's border.
    for (boundary, parent_top, parent_height, empty_top, float_top, body_top, extent) in [
        (0, 20.0, 10.0, 0.0, 0.0, 0.0, 40.0),
        (1, 0.0, 31.0, 9.0, 13.0, 21.0, 33.0),
        (2, 0.0, 31.0, 9.0, 13.0, 21.0, 33.0),
        (3, 0.0, 32.0, 8.0, 12.0, 20.0, 32.0),
    ] {
        for (writing_mode, direction) in all_writing_mode_directions() {
            let axes = FlowAxes::new(writing_mode, direction);
            let mut parent = adjoining_float_style(axes, 100.0, None, 0.0, 0.0);
            let edge = axes
                .physical_edges(LogicalEdgesOf::new(S::ZERO, S::ZERO, px(1.0), S::ZERO))
                .map(LengthOf::px);
            match boundary {
                1 => parent.border = edge,
                2 => parent.padding = edge,
                3 => parent.block_flow_context = BlockFlowContext::Independent,
                _ => {}
            }
            let tree = PublicBlockTree::default()
                .with_children(0, [1])
                .with_children(1, [2, 3, 4])
                .with_children(2, [])
                .with_children(3, [])
                .with_children(4, [])
                .with_style(0, adjoining_float_style(axes, 100.0, None, 0.0, 0.0))
                .with_style(1, parent)
                .with_style(2, adjoining_float_style(axes, 100.0, None, 8.0, 12.0))
                .with_style(
                    3,
                    NodeInputOf {
                        float: Float::Left,
                        ..adjoining_float_style(axes, 25.0, Some(20.0), 0.0, 0.0)
                    },
                )
                .with_style(4, adjoining_float_style(axes, 100.0, Some(10.0), 20.0, 0.0));
            let batch = compute_layout(&tree, 0, adjoining_float_request(axes, 100.0)).unwrap();
            let root = public_final_output(&batch, 0);
            let parent = public_final_output(&batch, 1);
            let empty = public_final_output(&batch, 2);
            let float = public_final_output(&batch, 3);
            let body = public_final_output(&batch, 4);
            assert_eq!(axes.logical_size(empty.size).block, S::ZERO);
            assert_eq!(axes.logical_size(float.size).block, px(20.0));
            assert_eq!(
                (
                    axes.logical_point(parent.location, parent.size, root.size)
                        .block,
                    axes.logical_size(parent.size).block,
                    axes.logical_point(empty.location, empty.size, parent.size)
                        .block,
                    axes.logical_point(float.location, float.size, parent.size)
                        .block,
                    axes.logical_point(body.location, body.size, parent.size)
                        .block,
                    axes.logical_size(root.size).block
                ),
                (
                    px(parent_top),
                    px(parent_height),
                    px(empty_top),
                    px(float_top),
                    px(body_top),
                    px(extent)
                ),
                "{writing_mode:?}/{direction:?}, boundary {boundary}",
            );
        }
    }
}

#[test]
fn ordinary_float_adjoining_leading_boundaries_f32() {
    assert_ordinary_float_leading_boundaries::<f32>();
}
#[test]
fn ordinary_float_adjoining_leading_boundaries_f64() {
    assert_ordinary_float_leading_boundaries::<f64>();
}

fn assert_ordinary_float_constrained_signed_group<S: LayoutScalar>() {
    let px = S::from_f64;
    for bottom in [0.0, -4.0] {
        for (writing_mode, direction) in all_writing_mode_directions() {
            let axes = FlowAxes::new(writing_mode, direction);
            let tree = PublicBlockTree::default()
                .with_children(0, [1, 2, 3, 4, 5])
                .with_children(1, [])
                .with_children(2, [])
                .with_children(3, [])
                .with_children(4, [])
                .with_children(5, [])
                .with_style(0, adjoining_float_style(axes, 100.0, None, 0.0, 0.0))
                .with_style(1, adjoining_float_style(axes, 100.0, Some(30.0), 0.0, 0.0))
                .with_style(
                    2,
                    NodeInputOf {
                        float: Float::Left,
                        ..adjoining_float_style(axes, 100.0, Some(20.0), 0.0, 0.0)
                    },
                )
                .with_style(3, adjoining_float_style(axes, 100.0, None, 12.0, bottom))
                .with_style(
                    4,
                    NodeInputOf {
                        float: Float::Left,
                        ..adjoining_float_style(axes, 25.0, Some(20.0), 0.0, 0.0)
                    },
                )
                .with_style(5, adjoining_float_style(axes, 100.0, Some(10.0), 0.0, 0.0));
            let batch = compute_layout(&tree, 0, adjoining_float_request(axes, 100.0)).unwrap();
            let root = public_final_output(&batch, 0);
            let first = public_final_output(&batch, 2);
            let second = public_final_output(&batch, 4);
            let body = public_final_output(&batch, 5);
            // Full-width F1 occupies30..50 in both the signed and all-negative-
            // margins-zero cases. F2 stays50: this does not assert an undefined
            // negative-margin-raised float position. B reveals max12 + min-4.
            assert_eq!(
                (
                    axes.logical_point(first.location, first.size, root.size)
                        .block,
                    axes.logical_point(second.location, second.size, root.size)
                        .block,
                    axes.logical_point(body.location, body.size, root.size)
                        .block,
                    axes.logical_size(root.size).block
                ),
                (px(30.0), px(50.0), px(42.0 + bottom), px(70.0)),
                "{writing_mode:?}/{direction:?}, empty bottom {bottom}",
            );
        }
    }
}

#[test]
fn ordinary_float_adjoining_constrained_signed_extrema_f32() {
    assert_ordinary_float_constrained_signed_group::<f32>();
}
#[test]
fn ordinary_float_adjoining_constrained_signed_extrema_f64() {
    assert_ordinary_float_constrained_signed_group::<f64>();
}

fn assert_ordinary_float_outside_processed_window<S: LayoutScalar>() {
    let px = S::from_f64;
    let axes = FlowAxes::new(WritingMode::HorizontalTb, Direction::Ltr);
    for margin in [0.0, 12.0] {
        let fixture = crate::test_support::inline_endpoint::Fixture::new(
            "A B C D",
            (0, 7),
            surgeist_text::WhiteSpace::Collapse,
            false,
            false,
        );
        // F is outside the actual admitted Text window; no source role is
        // deleted or invented to select ordinary Block ownership.
        let tree = fixture
            .tree::<S>(
                InlineBoundaryEdgeOf::ZERO,
                InlineBoundaryEdgeOf::ZERO,
                false,
                Some(30.0),
                true,
            )
            .children(0, [52, 2, 20, 10, 21])
            .style(0, adjoining_float_style(axes, 30.0, None, 0.0, 0.0))
            .children(52, [])
            .style(
                52,
                adjoining_float_style(axes, 30.0, Some(10.0), 0.0, margin),
            );
        let capture = LayoutFlowInput::try_new(
            &tree,
            0,
            vec![fixture.binding.clone()],
            WrappingInput::default(),
            Vec::new(),
        )
        .unwrap();
        let batch =
            compute_layout_with_flow(&capture, 0, adjoining_float_request(axes, 30.0)).unwrap();
        let [placed] = batch.placed_text() else {
            panic!("one actual processed paragraph");
        };
        assert_eq!(
            placed.layout().source().text(),
            &surgeist_text::Text::from("A B C D")
        );
        assert_eq!(placed.layout().lines().len(), 2);
        let float = public_final_output(&batch, 2);
        assert_eq!(float.size, Size::new(px(25.0), px(20.0)));
        assert_eq!(
            (
                float.location.y,
                placed
                    .layout()
                    .lines()
                    .iter()
                    .map(|line| line.bounds().origin.y)
                    .collect::<Vec<_>>(),
                public_final_output(&batch, 0).size.height
            ),
            (
                px(10.0 + margin),
                vec![30.0 + margin, 40.0 + margin],
                px(50.0 + margin)
            ),
            "preceding bottom margin {margin}",
        );
    }
}

#[test]
fn ordinary_float_adjoining_outside_processed_window_f32() {
    assert_ordinary_float_outside_processed_window::<f32>();
}
#[test]
fn ordinary_float_adjoining_outside_processed_window_f64() {
    assert_ordinary_float_outside_processed_window::<f64>();
}

fn actual_clearance_auto_root_request<S: LayoutScalar>() -> LayoutRootRequestOf<S> {
    let width = NonNegativeFiniteOf::new(S::from_f64(100.0)).unwrap();
    crate::test_support::requests::NativeRootFrameOf::new(
        FlowAxes::new(WritingMode::HorizontalTb, Direction::Ltr),
        Size::new(Some(width), None),
        OrthogonalFallbackOf::new(
            Size::NONE,
            NearestScrollportOf::Absent,
            Size::splat(OrthogonalFallbackLimitOf::Unbounded),
        ),
    )
    .request(Size::new(
        AxisSizingRequestOf::new(AvailableSpaceOf::Definite(width), SizingOperation::Normal),
        AxisSizingRequestOf::new(AvailableSpaceOf::Indefinite, SizingOperation::Normal),
    ))
}

fn assert_actual_clearance_empty_block_without_floats<S: LayoutScalar>(clear: Clear) {
    let px = S::from_f64;
    for clear in [Clear::None, clear] {
        let ordinary = NodeInputOf {
            display: Display::Block,
            size: Size::new(PreferredSizeOf::px(px(100.0)), PreferredSizeOf::AUTO),
            min_size: Size::new(MinSizeOf::px(S::ZERO), MinSizeOf::px(S::ZERO)),
            ..NodeInputOf::default()
        };
        let tree = PublicBlockTree::default()
            .with_children(0, [1, 2])
            .with_children(1, [])
            .with_children(2, [])
            .with_style(
                0,
                NodeInputOf {
                    border: Edges {
                        top: LengthOf::px(px(1.0)),
                        bottom: LengthOf::px(px(1.0)),
                        ..Edges::all(LengthOf::ZERO)
                    },
                    ..ordinary.clone()
                },
            )
            .with_style(
                1,
                NodeInputOf {
                    clear,
                    margin: Edges {
                        top: LengthAutoOf::px(px(10.0)),
                        bottom: LengthAutoOf::px(px(5.0)),
                        ..Edges::all(LengthAutoOf::ZERO)
                    },
                    ..ordinary.clone()
                },
            )
            .with_style(
                2,
                NodeInputOf {
                    size: Size::new(
                        PreferredSizeOf::px(px(100.0)),
                        PreferredSizeOf::px(px(10.0)),
                    ),
                    margin: Edges {
                        top: LengthAutoOf::px(px(7.0)),
                        ..Edges::all(LengthAutoOf::ZERO)
                    },
                    ..ordinary
                },
            );
        let batch = compute_layout(&tree, 0, actual_clearance_auto_root_request()).unwrap();
        let root = public_final_output(&batch, 0);
        let empty = public_final_output(&batch, 1);
        let following = public_final_output(&batch, 2);
        // No float introduces clearance. The adjoining 10/5/7 margins form
        // one 10px margin, and the empty box does not advance the flow cursor.
        assert_eq!(
            (
                empty.location,
                empty.size,
                following.location,
                following.size,
                root.size
            ),
            (
                Point::new(S::ZERO, px(11.0)),
                Size::new(px(100.0), S::ZERO),
                Point::new(S::ZERO, px(11.0)),
                Size::new(px(100.0), px(10.0)),
                Size::new(px(100.0), px(22.0)),
            ),
            "clear {clear:?}: empty/following/root geometry"
        );
    }
}

#[test]
fn actual_clearance_empty_left_without_floats_f32() {
    assert_actual_clearance_empty_block_without_floats::<f32>(Clear::Left);
}
#[test]
fn actual_clearance_empty_left_without_floats_f64() {
    assert_actual_clearance_empty_block_without_floats::<f64>(Clear::Left);
}
#[test]
fn actual_clearance_empty_right_without_floats_f32() {
    assert_actual_clearance_empty_block_without_floats::<f32>(Clear::Right);
}
#[test]
fn actual_clearance_empty_right_without_floats_f64() {
    assert_actual_clearance_empty_block_without_floats::<f64>(Clear::Right);
}
#[test]
fn actual_clearance_empty_both_without_floats_f32() {
    assert_actual_clearance_empty_block_without_floats::<f32>(Clear::Both);
}
#[test]
fn actual_clearance_empty_both_without_floats_f64() {
    assert_actual_clearance_empty_block_without_floats::<f64>(Clear::Both);
}

fn assert_actual_clearance_parent_geometry<S: LayoutScalar>(float_height: f64) {
    let px = S::from_f64;
    for clear in [Clear::None, Clear::Left] {
        let ordinary = NodeInputOf {
            display: Display::Block,
            size: Size::new(PreferredSizeOf::px(px(100.0)), PreferredSizeOf::AUTO),
            min_size: Size::new(MinSizeOf::px(S::ZERO), MinSizeOf::px(S::ZERO)),
            ..NodeInputOf::default()
        };
        let tree = PublicBlockTree::default()
            .with_children(0, [1, 2])
            .with_children(1, [])
            .with_children(2, [3])
            .with_children(3, [])
            .with_style(0, ordinary.clone())
            .with_style(
                1,
                NodeInputOf {
                    float: Float::Left,
                    size: Size::new(
                        PreferredSizeOf::px(px(100.0)),
                        PreferredSizeOf::px(px(float_height)),
                    ),
                    ..ordinary.clone()
                },
            )
            .with_style(2, ordinary.clone())
            .with_style(
                3,
                NodeInputOf {
                    clear,
                    size: Size::new(
                        PreferredSizeOf::px(px(100.0)),
                        PreferredSizeOf::px(px(10.0)),
                    ),
                    margin: Edges {
                        top: LengthAutoOf::px(px(10.0)),
                        ..Edges::all(LengthAutoOf::ZERO)
                    },
                    ..ordinary
                },
            );
        let batch = compute_layout(&tree, 0, actual_clearance_auto_root_request()).unwrap();
        let float = public_final_output(&batch, 1);
        assert_eq!(float.location, Point::ZERO);
        assert_eq!(float.size, Size::new(px(100.0), px(float_height)));
        let parent = public_final_output(&batch, 2);
        let child = public_final_output(&batch, 3);
        let introduced = clear == Clear::Left && float_height == 20.0;
        // Hypothetical child border y10 already clears a 5px float. A 20px
        // float introduces clearance, separating the child's margin from P.
        let (parent_y, parent_height, child_y) = if introduced {
            (0.0, 30.0, 20.0)
        } else {
            (10.0, 10.0, 0.0)
        };
        assert_eq!(
            (parent.location, parent.size, child.location, child.size),
            (
                Point::new(S::ZERO, px(parent_y)),
                Size::new(px(100.0), px(parent_height)),
                Point::new(S::ZERO, px(child_y)),
                Size::new(px(100.0), px(10.0)),
            ),
            "float height {float_height}, clear {clear:?}: parent/child geometry"
        );
    }
}

#[test]
fn actual_clearance_short_float_preserves_parent_child_collapse_f32() {
    assert_actual_clearance_parent_geometry::<f32>(5.0);
}
#[test]
fn actual_clearance_short_float_preserves_parent_child_collapse_f64() {
    assert_actual_clearance_parent_geometry::<f64>(5.0);
}
#[test]
fn actual_clearance_tall_float_separates_parent_child_margin_f32() {
    assert_actual_clearance_parent_geometry::<f32>(20.0);
}
#[test]
fn actual_clearance_tall_float_separates_parent_child_margin_f64() {
    assert_actual_clearance_parent_geometry::<f64>(20.0);
}
