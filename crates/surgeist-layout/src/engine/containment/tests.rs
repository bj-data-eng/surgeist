use crate::test_support::layout_tree::PublicLayoutTreeOf;
use crate::{
    AlignItems, AspectRatioOf, AvailableSpaceOf, AxisSizingRequestOf, Clear,
    CompletedLayoutBatchOf, ComputeInputOf, ContainingLayoutContext, Direction, Display, Edges,
    Float, FlowAxes, LayoutRootRequestOf, LayoutScalar, LengthOf, NearestScrollportOf, NodeInputOf,
    NodeOutputOf, OrthogonalFallbackLimitOf, OrthogonalFallbackOf, ParentFormattingContext, Point,
    PreferredSizeOf, Size, SizeContainment, SizingOperation, SizingRequestOf, WritingMode,
    compute_layout, compute_leaf,
};

fn sizing_request<S: LayoutScalar>() -> SizingRequestOf<S> {
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
    )
}

fn layout<S: LayoutScalar>(tree: &PublicLayoutTreeOf<S>) -> CompletedLayoutBatchOf<u32, S> {
    let request = LayoutRootRequestOf::standalone(
        sizing_request(),
        FlowAxes::new(WritingMode::HorizontalTb, Direction::Ltr),
        Size::NONE,
    );
    compute_layout(tree, 1, request).expect("containment fixture lays out")
}

fn output<S: LayoutScalar>(batch: &CompletedLayoutBatchOf<u32, S>, node: u32) -> NodeOutputOf<S> {
    batch
        .unrounded_entries()
        .iter()
        .find(|entry| entry.node() == node)
        .expect("fixture box publishes geometry")
        .output()
}

fn box_style<S: LayoutScalar>(width: f64, height: Option<f64>) -> NodeInputOf<S> {
    NodeInputOf {
        display: Display::Block,
        size: Size::new(
            PreferredSizeOf::px(S::from_f64(width)),
            height.map_or(PreferredSizeOf::AUTO, |height| {
                PreferredSizeOf::px(S::from_f64(height))
            }),
        ),
        ..NodeInputOf::default()
    }
}

fn assert_contained_block_percentages<S: LayoutScalar>() {
    for (height, parent_height, child_height) in [(None, 0.0, 24.0), (Some(40.0), 40.0, 20.0)] {
        let tree = PublicLayoutTreeOf::<S>::new()
            .children(1, [2])
            .children(2, [3])
            .style(
                1,
                NodeInputOf {
                    size_containment: SizeContainment::BothAxes,
                    ..box_style(100.0, height)
                },
            )
            .style(
                2,
                NodeInputOf {
                    display: Display::Block,
                    size: Size::new(
                        PreferredSizeOf::AUTO,
                        PreferredSizeOf::percent(S::from_f64(0.5)),
                    ),
                    ..NodeInputOf::default()
                },
            )
            .style(3, box_style(20.0, Some(24.0)));
        let batch = layout(&tree);
        assert_eq!(output(&batch, 1).size.height, S::from_f64(parent_height));
        assert_eq!(output(&batch, 2).size.height, S::from_f64(child_height));
        assert_eq!(output(&batch, 3).size.height, S::from_f64(24.0));
    }
}

#[test]
fn size_containment_preserves_block_percentage_provenance() {
    assert_contained_block_percentages::<f32>();
    assert_contained_block_percentages::<f64>();
}

fn assert_contained_block_shared_floats<S: LayoutScalar>() {
    let tree = PublicLayoutTreeOf::<S>::new()
        .children(1, [2, 3, 5])
        .children(3, [4])
        .style(1, box_style(100.0, None))
        .style(
            2,
            NodeInputOf {
                float: Float::Left,
                ..box_style(20.0, Some(30.0))
            },
        )
        .style(
            3,
            NodeInputOf {
                size_containment: SizeContainment::BothAxes,
                ..box_style(100.0, None)
            },
        )
        .style(
            4,
            NodeInputOf {
                float: Float::Left,
                ..box_style(10.0, Some(70.0))
            },
        )
        .style(
            5,
            NodeInputOf {
                clear: Clear::Both,
                ..box_style(100.0, Some(10.0))
            },
        );
    let batch = layout(&tree);
    assert_eq!(
        output(&batch, 3).size,
        Size::new(S::from_f64(100.0), S::ZERO)
    );
    assert_eq!(
        output(&batch, 4).location,
        Point::new(S::from_f64(20.0), S::ZERO)
    );
    assert_eq!(output(&batch, 5).location.y, S::from_f64(70.0));
    assert_eq!(output(&batch, 1).size.height, S::from_f64(80.0));
}

#[test]
fn size_containment_keeps_block_float_handoff() {
    assert_contained_block_shared_floats::<f32>();
    assert_contained_block_shared_floats::<f64>();
}

fn assert_contained_flex<S: LayoutScalar>() {
    for (size_containment, expected_height) in [
        (SizeContainment::None, 24.0),
        (SizeContainment::BothAxes, 0.0),
    ] {
        let tree = PublicLayoutTreeOf::<S>::new()
            .children(1, [2])
            .style(
                1,
                NodeInputOf {
                    display: Display::Flex,
                    size_containment,
                    ..box_style(100.0, None)
                },
            )
            .style(2, box_style(20.0, Some(24.0)));
        let batch = layout(&tree);
        assert_eq!(output(&batch, 1).size.height, S::from_f64(expected_height));
        assert_eq!(
            output(&batch, 2).size,
            Size::new(S::from_f64(20.0), S::from_f64(24.0))
        );
    }
}

#[test]
fn size_containment_formats_flex_items_in_the_empty_box_size() {
    assert_contained_flex::<f32>();
    assert_contained_flex::<f64>();
}

fn assert_contained_flex_physical_baseline<S: LayoutScalar>() {
    let px = S::from_f64;
    let tree = PublicLayoutTreeOf::<S>::new()
        .children(1, [2, 4])
        .children(2, [3])
        .style(
            1,
            NodeInputOf {
                display: Display::Flex,
                align_items: Some(AlignItems::Baseline),
                ..box_style(100.0, None)
            },
        )
        .style(
            2,
            NodeInputOf {
                display: Display::Flex,
                align_items: Some(AlignItems::Start),
                size_containment: SizeContainment::BothAxes,
                ..box_style(30.0, None)
            },
        )
        .style(3, box_style(20.0, Some(24.0)))
        .style(4, box_style(20.0, Some(10.0)));
    let batch = layout(&tree);
    assert_eq!(output(&batch, 2).size.height, S::ZERO);
    assert_eq!(output(&batch, 2).location.y, S::ZERO);
    assert_eq!(output(&batch, 3).size.height, px(24.0));
    assert_eq!(output(&batch, 4).location.y, px(14.0));
}

#[test]
fn size_containment_preserves_flex_physical_baseline_alignment() {
    assert_contained_flex_physical_baseline::<f32>();
    assert_contained_flex_physical_baseline::<f64>();
}

fn assert_contained_atomic_baseline_preview<S: LayoutScalar>() {
    let px = S::from_f64;
    let line = crate::LineBreakInputOf::new().with_metrics(
        crate::InlineMetricsOf::from_ascent_descent(
            crate::InlineBaselineContext::Physical,
            px(10.0),
            px(2.0),
        )
        .unwrap(),
    );
    let tree = PublicLayoutTreeOf::<S>::new()
        .children(1, [2, 5])
        .children(2, [3])
        .children(3, [4])
        .style(
            1,
            NodeInputOf {
                display: Display::Flex,
                align_items: Some(AlignItems::Baseline),
                ..box_style(70.0, None)
            },
        )
        .style(
            2,
            NodeInputOf {
                min_size: Size::new(crate::MinSizeOf::px(S::ZERO), crate::MinSizeOf::AUTO),
                ..box_style(100.0, None)
            },
        )
        .style(
            3,
            NodeInputOf {
                display: Display::InlineBlock,
                atomic_inline_participation: Some(crate::AtomicInlineParticipation::new(
                    crate::BidiLevel::try_new(0).unwrap(),
                    crate::InlineBreakOpportunity::Allowed,
                )),
                size_containment: SizeContainment::BothAxes,
                ..box_style(30.0, Some(40.0))
            },
        )
        .input(4, crate::LayoutInputOf::line_break(line))
        .style(
            5,
            NodeInputOf {
                flex_shrink: crate::FlexShrinkOf::try_new(S::ZERO).unwrap(),
                ..box_style(10.0, Some(10.0))
            },
        );
    let batch = layout(&tree);
    assert_eq!(output(&batch, 2).size.width, px(60.0));
    assert_eq!(output(&batch, 3).size, Size::new(px(30.0), px(40.0)));
    assert_eq!(output(&batch, 5).location, Point::new(px(60.0), S::ZERO));
}

#[test]
fn size_containment_preserves_atomic_inline_baseline_during_flex_refresh() {
    assert_contained_atomic_baseline_preview::<f32>();
    assert_contained_atomic_baseline_preview::<f64>();
}

fn assert_contained_grid_lanes<S: LayoutScalar>() {
    let px = S::from_f64;
    for (size_containment, expected_height) in [
        (SizeContainment::None, 24.0),
        (SizeContainment::BothAxes, 0.0),
    ] {
        let tree = PublicLayoutTreeOf::<S>::new()
            .children(1, [2])
            .style(
                1,
                NodeInputOf {
                    display: Display::GridLanes,
                    size_containment,
                    grid_template_columns: vec![crate::TrackComponentOf::px(px(100.0))],
                    ..box_style(100.0, None)
                },
            )
            .style(2, box_style(20.0, Some(24.0)));
        let batch = layout(&tree);
        assert_eq!(
            output(&batch, 1).size,
            Size::new(px(100.0), px(expected_height))
        );
        assert_eq!(output(&batch, 2).size, Size::new(px(20.0), px(24.0)));
    }
}

#[test]
fn size_containment_formats_grid_lanes_items_in_the_empty_box_size() {
    assert_contained_grid_lanes::<f32>();
    assert_contained_grid_lanes::<f64>();
}

fn assert_contained_leaf_measurement<S: LayoutScalar>() {
    let px = S::from_f64;
    let context = ContainingLayoutContext::new(
        FlowAxes::new(WritingMode::HorizontalTb, Direction::Ltr),
        ParentFormattingContext::NoParent,
    );
    let edge = |value| {
        let length = LengthOf::px(px(value));
        Edges::new(length, length, length, length)
    };
    let style = NodeInputOf {
        size_containment: SizeContainment::BothAxes,
        border: edge(2.0),
        padding: edge(3.0),
        ..box_style::<S>(60.0, None)
    };
    let size_query =
        ComputeInputOf::leaf_content_size(Size::NONE, Size::NONE, context, sizing_request())
            .unwrap();
    let empty = compute_leaf(size_query, &style, |_| -> Result<Size<S>, ()> {
        panic!("a contained intrinsic query must not measure leaf contents")
    })
    .unwrap();
    assert_eq!(empty.size, Size::splat(px(10.0)));

    let placed =
        ComputeInputOf::leaf_layout(Size::NONE, Size::NONE, context, sizing_request()).unwrap();
    let measured = compute_leaf(placed, &style, |input| {
        assert_eq!(input.known_content_size(), Size::NONE);
        assert_eq!(
            input
                .content_sizing_request()
                .axes()
                .map(|axis| axis.space().definite_size().map(|size| size.get())),
            Size::new(Some(px(50.0)), Some(S::ZERO))
        );
        Ok::<_, ()>(Size::new(px(80.0), px(24.0)))
    })
    .unwrap();
    assert_eq!(measured.size, Size::new(px(60.0), px(10.0)));
    assert!(measured.content_size.width >= px(80.0));
    assert!(measured.content_size.height >= px(24.0));

    let narrower = ComputeInputOf::leaf_layout(
        Size::new(Some(px(40.0)), None),
        Size::NONE,
        context,
        sizing_request(),
    )
    .unwrap();
    let measured = compute_leaf(narrower, &style, |input| {
        assert_eq!(
            input
                .content_sizing_request()
                .axes()
                .width
                .space()
                .definite_size()
                .unwrap()
                .get(),
            px(30.0)
        );
        Ok::<_, ()>(Size::new(px(80.0), px(24.0)))
    })
    .unwrap();
    assert_eq!(measured.size, Size::new(px(40.0), px(10.0)));
}

#[test]
fn size_containment_separates_leaf_measurement_and_formatting() {
    assert_contained_leaf_measurement::<f32>();
    assert_contained_leaf_measurement::<f64>();
}

fn assert_contained_replaced_ratio<S: LayoutScalar>() {
    let px = S::from_f64;
    for (aspect_ratio, height) in [(None, 0.0), (AspectRatioOf::new(S::ONE), 100.0)] {
        let tree = PublicLayoutTreeOf::<S>::new()
            .children(1, [2])
            .style(1, box_style(100.0, None))
            .style(
                2,
                NodeInputOf {
                    size_containment: SizeContainment::BothAxes,
                    item_is_replaced: true,
                    aspect_ratio,
                    ..box_style(100.0, None)
                },
            )
            .measure(2, Size::new(px(300.0), px(100.0)));
        assert_eq!(
            output(&layout(&tree), 2).size,
            Size::new(px(100.0), px(height))
        );
    }
}

#[test]
fn size_containment_replaces_natural_size_and_keeps_the_used_ratio() {
    assert_contained_replaced_ratio::<f32>();
    assert_contained_replaced_ratio::<f64>();
}

fn assert_inline_size_containment_keeps_live_block_content<S: LayoutScalar>() {
    use crate::geometry::LogicalSizeOf;
    let px = S::from_f64;
    for writing_mode in [WritingMode::HorizontalTb, WritingMode::VerticalLr] {
        let axes = FlowAxes::new(writing_mode, Direction::Ltr);
        for (containment, inline, block, expected_inline, expected_block, child_block) in [
            (
                SizeContainment::None,
                PreferredSizeOf::MAX_CONTENT,
                PreferredSizeOf::AUTO,
                60.0,
                24.0,
                24.0,
            ),
            (
                SizeContainment::BothAxes,
                PreferredSizeOf::MAX_CONTENT,
                PreferredSizeOf::AUTO,
                0.0,
                0.0,
                24.0,
            ),
            (
                SizeContainment::InlineAxis,
                PreferredSizeOf::MAX_CONTENT,
                PreferredSizeOf::AUTO,
                0.0,
                24.0,
                24.0,
            ),
            (
                SizeContainment::InlineAxis,
                PreferredSizeOf::px(px(40.0)),
                PreferredSizeOf::AUTO,
                40.0,
                24.0,
                24.0,
            ),
            (
                SizeContainment::InlineAxis,
                PreferredSizeOf::MAX_CONTENT,
                PreferredSizeOf::px(px(40.0)),
                0.0,
                40.0,
                20.0,
            ),
        ] {
            let tree = PublicLayoutTreeOf::<S>::new()
                .children(1, [2])
                .children(2, [3])
                .children(3, [])
                .style(
                    1,
                    NodeInputOf {
                        display: Display::Block,
                        writing_mode,
                        size_containment: containment,
                        size: axes.physical_size(LogicalSizeOf::new(inline, block)),
                        ..NodeInputOf::default()
                    },
                )
                .style(
                    2,
                    NodeInputOf {
                        display: Display::Block,
                        writing_mode,
                        size: axes.physical_size(LogicalSizeOf::new(
                            PreferredSizeOf::px(px(60.0)),
                            PreferredSizeOf::percent(px(0.5)),
                        )),
                        ..NodeInputOf::default()
                    },
                )
                .style(
                    3,
                    NodeInputOf {
                        display: Display::Block,
                        writing_mode,
                        size: axes.physical_size(LogicalSizeOf::new(
                            PreferredSizeOf::px(px(20.0)),
                            PreferredSizeOf::px(px(24.0)),
                        )),
                        ..NodeInputOf::default()
                    },
                );
            let request = LayoutRootRequestOf::standalone(sizing_request(), axes, Size::NONE);
            let batch = compute_layout(&tree, 1, request).unwrap();
            assert_eq!(
                axes.logical_size(output(&batch, 1).size),
                LogicalSizeOf::new(px(expected_inline), px(expected_block)),
                "{writing_mode:?}, {containment:?}",
            );
            assert_eq!(
                axes.logical_size(output(&batch, 2).size),
                LogicalSizeOf::new(px(60.0), px(child_block)),
                "{writing_mode:?}, {containment:?}",
            );
        }
    }
}

#[test]
fn inline_size_containment_keeps_live_block_content() {
    assert_inline_size_containment_keeps_live_block_content::<f32>();
    assert_inline_size_containment_keeps_live_block_content::<f64>();
}

#[test]
fn inline_size_containment_keeps_live_flex_lines() {
    fn lane<S: LayoutScalar>() {
        let px = S::from_f64;
        for (direction, containment, expected_inline, expected_block) in [
            (crate::FlexDirection::Row, SizeContainment::None, 40.0, 12.0),
            (
                crate::FlexDirection::Row,
                SizeContainment::InlineAxis,
                20.0,
                20.0,
            ),
            (
                crate::FlexDirection::Column,
                SizeContainment::None,
                30.0,
                20.0,
            ),
            (
                crate::FlexDirection::Column,
                SizeContainment::InlineAxis,
                20.0,
                20.0,
            ),
        ] {
            let item_inline = if direction == crate::FlexDirection::Row {
                20.0
            } else {
                30.0
            };
            let item = |height| NodeInputOf {
                flex_shrink: crate::FlexShrinkOf::try_new(S::ZERO).unwrap(),
                ..box_style::<S>(item_inline, Some(height))
            };
            let tree = PublicLayoutTreeOf::<S>::new()
                .children(1, [2, 3])
                .children(2, [])
                .children(3, [])
                .style(
                    1,
                    NodeInputOf {
                        display: Display::Flex,
                        flex_direction: direction,
                        flex_wrap: crate::FlexWrap::Wrap,
                        min_size: Size::new(crate::MinSizeOf::px(px(20.0)), crate::MinSizeOf::AUTO),
                        size_containment: containment,
                        ..NodeInputOf::default()
                    },
                )
                .style(2, item(8.0))
                .style(3, item(12.0));
            let batch = layout(&tree);
            assert_eq!(
                output(&batch, 1).size,
                Size::new(px(expected_inline), px(expected_block)),
                "{direction:?}, {containment:?}",
            );
        }
    }
    lane::<f32>();
    lane::<f64>();
}

#[test]
fn inline_size_containment_measures_live_leaf_block_at_accepted_inline_size() {
    fn lane<S: LayoutScalar>() {
        let px = S::from_f64;
        let context = ContainingLayoutContext::new(
            FlowAxes::new(WritingMode::HorizontalTb, Direction::Ltr),
            ParentFormattingContext::NoParent,
        );
        for (replaced, known_width, height) in [
            (false, None, 24.0),
            (false, Some(40.0), 16.0),
            (true, None, 24.0),
        ] {
            let style = NodeInputOf {
                display: Display::Block,
                size_containment: SizeContainment::InlineAxis,
                item_is_replaced: replaced,
                ..NodeInputOf::default()
            };
            let known = Size::new(known_width.map(px), None);
            let query =
                ComputeInputOf::leaf_content_size(known, Size::NONE, context, sizing_request())
                    .unwrap();
            let mut measured = false;
            let result = compute_leaf(query, &style, |input| {
                measured = true;
                assert_eq!(
                    input
                        .content_sizing_request()
                        .axes()
                        .width
                        .space()
                        .definite_size()
                        .map(|size| size.get()),
                    Some(px(known_width.unwrap_or(0.0))),
                );
                Ok::<_, ()>(Size::new(px(80.0), px(height)))
            })
            .unwrap();
            assert!(measured);
            assert_eq!(
                result.size,
                Size::new(px(known_width.unwrap_or(0.0)), px(height))
            );
        }
    }
    lane::<f32>();
    lane::<f64>();
}

fn width_sensitive_grid_measurement<S: LayoutScalar>(
    input: crate::LeafMeasureInputOf<S>,
) -> Size<S> {
    let px = S::from_f64;
    let flow = input.flow_axes();
    let inline = flow
        .logical_size(input.content_sizing_request().axes())
        .inline;
    let block = if inline.space().definite_size().map(|size| size.get()) == Some(px(20.0)) {
        px(90.0)
    } else {
        px(40.0)
    };
    flow.physical_size(crate::geometry::LogicalSizeOf::new(S::ZERO, block))
}

fn inline_containment_test_columns<S: LayoutScalar>() -> Vec<crate::TrackComponentOf<S>> {
    vec![
        crate::TrackComponentOf::px(S::from_f64(40.0)),
        crate::TrackComponentOf::percent(S::from_f64(0.5)),
    ]
}

fn inline_containment_grid_leaf<S: LayoutScalar>(writing_mode: WritingMode) -> NodeInputOf<S> {
    NodeInputOf {
        display: Display::Block,
        writing_mode,
        grid_column: crate::GridPlacement::try_line(2).unwrap(),
        ..NodeInputOf::default()
    }
}

#[test]
fn inline_size_containment_splits_grid_track_and_container_extents() {
    fn lane<S: LayoutScalar>() {
        let px = S::from_f64;
        for display in [Display::Grid, Display::GridLanes] {
            for writing_mode in [WritingMode::HorizontalTb, WritingMode::VerticalLr] {
                let flow = FlowAxes::new(writing_mode, Direction::Ltr);
                let tree = PublicLayoutTreeOf::<S>::new()
                    .children(1, [2])
                    .style(
                        1,
                        NodeInputOf {
                            display,
                            writing_mode,
                            grid_template_columns: inline_containment_test_columns(),
                            justify_content: Some(crate::AlignContent::Start),
                            size_containment: SizeContainment::InlineAxis,
                            ..NodeInputOf::default()
                        },
                    )
                    .style(2, inline_containment_grid_leaf(writing_mode));
                let batch = layout(&tree.measure_with(2, width_sensitive_grid_measurement));
                assert_eq!(
                    flow.logical_size(output(&batch, 1).size),
                    crate::geometry::LogicalSizeOf::new(px(40.0), px(90.0)),
                    "{display:?}, {writing_mode:?}"
                );
                assert_eq!(
                    flow.logical_size(output(&batch, 2).size),
                    crate::geometry::LogicalSizeOf::new(px(20.0), px(90.0)),
                    "{display:?}, {writing_mode:?}"
                );
            }
        }
    }
    lane::<f32>();
    lane::<f64>();
}

#[test]
fn inline_size_containment_keeps_subgrid_block_contributions_live() {
    fn lane<S: LayoutScalar>() {
        let px = S::from_f64;
        for writing_mode in [WritingMode::HorizontalTb, WritingMode::VerticalLr] {
            let flow = FlowAxes::new(writing_mode, Direction::Ltr);
            let transposed = writing_mode == WritingMode::VerticalLr;
            let inline_track = vec![crate::TrackComponentOf::px(px(100.0))];
            let (columns, rows) = if transposed {
                (Vec::new(), inline_track)
            } else {
                (inline_track, Vec::new())
            };
            let tree = PublicLayoutTreeOf::<S>::new()
                .children(1, [2])
                .children(2, [3])
                .style(
                    1,
                    NodeInputOf {
                        display: Display::Grid,
                        grid_template_columns: columns,
                        grid_template_rows: rows,
                        ..NodeInputOf::default()
                    },
                )
                .style(
                    2,
                    NodeInputOf {
                        display: Display::Grid,
                        writing_mode,
                        justify_self: Some(AlignItems::Start),
                        align_self: Some(AlignItems::Start),
                        grid_template_columns: inline_containment_test_columns(),
                        grid_template_rows: vec![crate::TrackComponentOf::Subgrid(
                            crate::SubgridTrack {
                                name_components: Vec::new(),
                            },
                        )],
                        size: flow.physical_size(crate::geometry::LogicalSizeOf::new(
                            PreferredSizeOf::MAX_CONTENT,
                            PreferredSizeOf::AUTO,
                        )),
                        size_containment: SizeContainment::InlineAxis,
                        ..NodeInputOf::default()
                    },
                )
                .style(3, inline_containment_grid_leaf(writing_mode));
            let batch = layout(&tree.measure_with(3, width_sensitive_grid_measurement));
            for (node, inline) in [(1, 100.0), (2, 40.0), (3, 20.0)] {
                assert_eq!(
                    flow.logical_size(output(&batch, node).size),
                    crate::geometry::LogicalSizeOf::new(px(inline), px(90.0)),
                    "{writing_mode:?}, node {node}"
                );
            }
        }
    }
    lane::<f32>();
    lane::<f64>();
}

#[test]
fn inline_size_containment_keeps_live_rows_when_lanes_stack_inline() {
    fn lane<S: LayoutScalar>() {
        let px = S::from_f64;
        let tree = PublicLayoutTreeOf::<S>::new()
            .children(1, [2, 3])
            .style(
                1,
                NodeInputOf {
                    display: Display::GridLanes,
                    grid_lanes_flow: crate::GridLanesFlow::new(
                        crate::GridAxisKind::Row,
                        false,
                        false,
                    ),
                    size: Size::new(PreferredSizeOf::MAX_CONTENT, PreferredSizeOf::AUTO),
                    grid_template_rows: vec![crate::TrackComponentOf::AUTO; 2],
                    size_containment: SizeContainment::InlineAxis,
                    ..NodeInputOf::default()
                },
            )
            .style(2, box_style(40.0, Some(10.0)))
            .style(
                3,
                NodeInputOf {
                    grid_row: crate::GridPlacement::try_line(2).unwrap(),
                    ..box_style(50.0, Some(15.0))
                },
            );
        let batch = layout(&tree);
        assert_eq!(output(&batch, 1).size, Size::new(S::ZERO, px(25.0)));
        assert_eq!(output(&batch, 2).size, Size::new(px(40.0), px(10.0)));
        assert_eq!(output(&batch, 3).size, Size::new(px(50.0), px(15.0)));
        assert_eq!(output(&batch, 3).location, Point::new(S::ZERO, px(10.0)));
    }
    lane::<f32>();
    lane::<f64>();
}

#[test]
fn inline_size_containment_leaves_excluded_box_roles_unsized_by_containment() {
    let ordinary = NodeInputOf::<f64> {
        size_containment: SizeContainment::InlineAxis,
        ..NodeInputOf::default()
    };
    assert_eq!(
        super::size_containment(&ordinary),
        SizeContainment::InlineAxis
    );
    for excluded in [
        NodeInputOf {
            display: Display::None,
            ..ordinary.clone()
        },
        NodeInputOf {
            block_generation: crate::BlockGeneration::Anonymous,
            ..ordinary.clone()
        },
        NodeInputOf {
            item_is_table: true,
            ..ordinary
        },
    ] {
        assert_eq!(super::size_containment(&excluded), SizeContainment::None);
    }
}

#[test]
fn inline_size_containment_supplies_the_live_inline_basis_to_row_repeat() {
    fn lane<S: LayoutScalar>() {
        let px = S::from_f64;
        let repeated_rows = crate::TrackRepetitionOf::auto_fill_components(vec![
            crate::TrackComponentOf::MAX_CONTENT,
        ])
        .unwrap();
        let tree = PublicLayoutTreeOf::<S>::new()
            .children(1, [2])
            .style(
                1,
                NodeInputOf {
                    display: Display::GridLanes,
                    grid_lanes_flow: crate::GridLanesFlow::new(
                        crate::GridAxisKind::Row,
                        false,
                        false,
                    ),
                    size: Size::new(PreferredSizeOf::MAX_CONTENT, PreferredSizeOf::AUTO),
                    min_size: Size::new(crate::MinSizeOf::px(px(40.0)), crate::MinSizeOf::AUTO),
                    max_size: Size::new(crate::MaxSizeOf::NONE, crate::MaxSizeOf::px(px(100.0))),
                    gap: Size::new(LengthOf::ZERO, LengthOf::px(px(5.0))),
                    grid_template_rows: vec![crate::TrackComponentOf::Repeat(repeated_rows)],
                    align_content: Some(crate::AlignContent::Start),
                    size_containment: SizeContainment::InlineAxis,
                    ..NodeInputOf::default()
                },
            )
            .style(
                2,
                NodeInputOf {
                    size: Size::new(PreferredSizeOf::percent(px(0.5)), PreferredSizeOf::AUTO),
                    grid_row: crate::GridPlacement::try_line(1).unwrap(),
                    ..NodeInputOf::default()
                },
            );
        let batch = layout(&tree.measure_with(2, width_sensitive_grid_measurement));
        // The 20px leaf width yields a 90px repeat operand. Two groups plus
        // their gutter cannot fit the 100px maximum, so the used gap is zero.
        assert_eq!(output(&batch, 1).size, Size::new(px(40.0), px(90.0)));
        assert_eq!(output(&batch, 2).size, Size::new(px(20.0), px(90.0)));
    }
    lane::<f32>();
    lane::<f64>();
}
