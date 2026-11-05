//! Absolute children remain real boxes when either Grid track axis is empty.
//! Insets use the padding containing block; zero-padding static controls keep
//! the distinct content-edge static-position rule out of these expectations.

use super::*;
use crate::test_support::requests::NativeRootFrameOf;

#[derive(Clone, Copy, Debug)]
enum Tracks {
    Both,
    Neither,
    ColumnsOnly,
    RowsOnly,
}

const TOPOLOGIES: [Tracks; 4] = [
    Tracks::Both,
    Tracks::Neither,
    Tracks::ColumnsOnly,
    Tracks::RowsOnly,
];

fn root_request<S: LayoutScalar>(operation: SizingOperation) -> LayoutRootRequestOf<S> {
    let viewport = Size::new(
        NonNegativeFiniteOf::new(S::from_f64(600.0)).unwrap(),
        NonNegativeFiniteOf::new(S::from_f64(400.0)).unwrap(),
    );
    let fallback = OrthogonalFallbackOf::new(
        viewport.map(Some),
        NearestScrollportOf::Absent,
        Size::splat(OrthogonalFallbackLimitOf::Unbounded),
    );
    let space = if operation == SizingOperation::Normal {
        viewport.map(AvailableSpaceOf::Definite)
    } else {
        Size::splat(AvailableSpaceOf::Indefinite)
    };
    NativeRootFrameOf::new(
        FlowAxes::new(WritingMode::HorizontalTb, Direction::Ltr),
        viewport.map(Some),
        fallback,
    )
    .request(space.map(|space| AxisSizingRequestOf::new(space, operation)))
}

fn grid<S: LayoutScalar>(
    direction: Direction,
    box_sizing: BoxSizing,
    size: Size<f64>,
    tracks: Tracks,
) -> NodeInputOf<S> {
    NodeInputOf {
        display: Display::Grid,
        direction,
        box_sizing,
        size: size.map(|value| PreferredSizeOf::px(S::from_f64(value))),
        grid_template_columns: if matches!(tracks, Tracks::Both | Tracks::ColumnsOnly) {
            vec![TrackComponentOf::px(S::from_f64(32.0))]
        } else {
            Vec::new()
        },
        grid_template_rows: if matches!(tracks, Tracks::Both | Tracks::RowsOnly) {
            vec![TrackComponentOf::px(S::from_f64(24.0))]
        } else {
            Vec::new()
        },
        ..NodeInputOf::default()
    }
}

fn fixed_absolute<S: LayoutScalar>(direction: Direction, size: Size<f64>) -> NodeInputOf<S> {
    NodeInputOf {
        display: Display::Block,
        position: Position::Absolute,
        direction,
        size: size.map(|value| PreferredSizeOf::px(S::from_f64(value))),
        ..NodeInputOf::default()
    }
}

fn output<S: LayoutScalar>(entries: &[LayoutOutputEntryOf<u32, S>], node: u32) -> NodeOutputOf<S> {
    entries
        .iter()
        .find(|entry| entry.node() == node)
        .unwrap_or_else(|| panic!("missing actual output for node {node}"))
        .output()
}

#[track_caller]
fn assert_rect<S: LayoutScalar>(actual: NodeOutputOf<S>, expected: [S; 4]) {
    assert_eq!(actual.location, Point::new(expected[0], expected[1]));
    assert_eq!(actual.size, Size::new(expected[2], expected[3]));
}

#[track_caller]
fn assert_integer_rect<S: LayoutScalar>(
    batch: &CompletedLayoutBatchOf<u32, S>,
    node: u32,
    expected: [f64; 4],
) {
    let expected = expected.map(S::from_f64);
    for (phase, entries) in [
        ("unrounded", batch.unrounded_entries()),
        ("final", batch.final_entries()),
    ] {
        let actual = output(entries, node);
        assert_eq!(
            [
                actual.location.x,
                actual.location.y,
                actual.size.width,
                actual.size.height
            ],
            expected,
            "{phase} box for node {node}"
        );
    }
}

fn assert_nodes<S: LayoutScalar>(batch: &CompletedLayoutBatchOf<u32, S>, expected: &[u32]) {
    for entries in [batch.unrounded_entries(), batch.final_entries()] {
        let mut actual = entries
            .iter()
            .map(LayoutOutputEntryOf::node)
            .collect::<Vec<_>>();
        actual.sort_unstable();
        assert_eq!(
            actual, expected,
            "complete source-node publication without duplicates"
        );
    }
}

fn compute<S: LayoutScalar>(tree: &PublicLayoutTreeOf<S>) -> CompletedLayoutBatchOf<u32, S> {
    compute_layout(tree, 1, root_request::<S>(SizingOperation::Normal))
        .expect("authored native Grid boxes are admitted")
}

fn explicit_empty_axes<S: LayoutScalar>(direction: Direction) {
    for tracks in TOPOLOGIES {
        for end_insets in [false, true] {
            let mut absolute = fixed_absolute::<S>(direction, Size::new(20.0, 10.0));
            absolute.inset = if end_insets {
                Edges {
                    right: LengthAutoOf::px(S::from_f64(9.0)),
                    bottom: LengthAutoOf::px(S::from_f64(13.0)),
                    ..Edges::all(LengthAutoOf::AUTO)
                }
            } else {
                Edges {
                    left: LengthAutoOf::px(S::from_f64(7.0)),
                    top: LengthAutoOf::px(S::from_f64(11.0)),
                    ..Edges::all(LengthAutoOf::AUTO)
                }
            };
            let tree = PublicLayoutTreeOf::new()
                .children(1, [2])
                .children(2, [])
                .style(
                    1,
                    grid::<S>(
                        direction,
                        BoxSizing::BorderBox,
                        Size::new(100.0, 80.0),
                        tracks,
                    ),
                )
                .style(2, absolute);
            assert!(!tree.has_leaf_measurement(1) && !tree.has_leaf_measurement(2));
            let batch = compute(&tree);
            assert_nodes(&batch, &[1, 2]);
            assert_integer_rect(&batch, 1, [0.0, 0.0, 100.0, 80.0]);
            // Physical insets retain their sides even when the Grid direction changes.
            assert_integer_rect(
                &batch,
                2,
                if end_insets {
                    [71.0, 57.0, 20.0, 10.0]
                } else {
                    [7.0, 11.0, 20.0, 10.0]
                },
            );
            assert_eq!(
                output(batch.final_entries(), 2).source_index,
                SourceIndex::ZERO
            );
        }
    }
}

#[test]
fn empty_axis_explicit_insets_f32_ltr() {
    explicit_empty_axes::<f32>(Direction::Ltr);
}
#[test]
fn empty_axis_explicit_insets_f32_rtl() {
    explicit_empty_axes::<f32>(Direction::Rtl);
}
#[test]
fn empty_axis_explicit_insets_f64_ltr() {
    explicit_empty_axes::<f64>(Direction::Ltr);
}
#[test]
fn empty_axis_explicit_insets_f64_rtl() {
    explicit_empty_axes::<f64>(Direction::Rtl);
}

fn explicit_padding_box<S: LayoutScalar>() {
    for direction in [Direction::Ltr, Direction::Rtl] {
        for (box_sizing, outer) in [
            (BoxSizing::BorderBox, 100.0),
            (BoxSizing::ContentBox, 140.0),
        ] {
            for tracks in TOPOLOGIES {
                let mut parent = grid::<S>(direction, box_sizing, Size::splat(100.0), tracks);
                parent.border = Edges::all(LengthOf::px(S::from_f64(10.0)));
                parent.padding = Edges::all(LengthOf::px(S::from_f64(10.0)));
                let mut tree = PublicLayoutTreeOf::new()
                    .children(1, [2, 3, 4, 5])
                    .style(1, parent);
                for (node, at_end, margin) in [
                    (2, false, 0.0),
                    (3, true, 0.0),
                    (4, false, 10.0),
                    (5, true, 10.0),
                ] {
                    let mut child = fixed_absolute::<S>(direction, Size::splat(50.0));
                    child.box_sizing = box_sizing;
                    child.margin = Edges::all(LengthAutoOf::px(S::from_f64(margin)));
                    child.inset = if at_end {
                        Edges {
                            right: LengthAutoOf::px(S::ZERO),
                            bottom: LengthAutoOf::px(S::ZERO),
                            ..Edges::all(LengthAutoOf::AUTO)
                        }
                    } else {
                        Edges {
                            left: LengthAutoOf::px(S::ZERO),
                            top: LengthAutoOf::px(S::ZERO),
                            ..Edges::all(LengthAutoOf::AUTO)
                        }
                    };
                    tree = tree.children(node, []).style(node, child);
                }
                let batch = compute(&tree);
                assert_nodes(&batch, &[1, 2, 3, 4, 5]);
                assert_integer_rect(&batch, 1, [0.0, 0.0, outer, outer]);
                // Insets start at border10; padding remains inside this containing block.
                for (ordinal, (node, coordinate)) in
                    [(2, 10.0), (3, outer - 60.0), (4, 20.0), (5, outer - 70.0)]
                        .into_iter()
                        .enumerate()
                {
                    assert_integer_rect(&batch, node, [coordinate, coordinate, 50.0, 50.0]);
                    assert_eq!(
                        output(batch.final_entries(), node).source_index,
                        SourceIndex::new(ordinal)
                    );
                }
            }
        }
    }
}
#[test]
fn empty_axes_explicit_padding_box_f32() {
    explicit_padding_box::<f32>();
}
#[test]
fn empty_axes_explicit_padding_box_f64() {
    explicit_padding_box::<f64>();
}

fn percentage_subtree<S: LayoutScalar>() {
    // Match the original fixture adapter's normalized Flex box leaves.
    for direction in [Direction::Ltr, Direction::Rtl] {
        for box_sizing in [BoxSizing::BorderBox, BoxSizing::ContentBox] {
            for tracks in [Tracks::Both, Tracks::Neither] {
                let tree = PublicLayoutTreeOf::new()
                    .children(1, [2])
                    .children(2, [3, 4, 5])
                    .children(3, [])
                    .children(4, [])
                    .children(5, [6])
                    .children(6, [])
                    .style(
                        1,
                        grid::<S>(direction, box_sizing, Size::new(300.0, 110.0), tracks),
                    )
                    .style(
                        2,
                        NodeInputOf {
                            display: Display::Flex,
                            position: Position::Absolute,
                            direction,
                            box_sizing,
                            flex_direction: FlexDirection::Column,
                            // Definite width isolates empty-axis traversal from
                            // the original auto-width Flex percentage question.
                            size: Size::new(
                                PreferredSizeOf::px(S::from_f64(200.0)),
                                PreferredSizeOf::AUTO,
                            ),
                            inset: Edges {
                                left: LengthAutoOf::px(S::from_f64(50.0)),
                                top: LengthAutoOf::px(S::from_f64(40.0)),
                                ..Edges::all(LengthAutoOf::AUTO)
                            },
                            ..NodeInputOf::default()
                        },
                    )
                    .style(
                        3,
                        NodeInputOf {
                            display: Display::Flex,
                            direction,
                            box_sizing,
                            size: Size::new(
                                PreferredSizeOf::px(S::from_f64(200.0)),
                                PreferredSizeOf::px(S::from_f64(10.0)),
                            ),
                            ..NodeInputOf::default()
                        },
                    )
                    .style(
                        4,
                        NodeInputOf {
                            display: Display::Flex,
                            direction,
                            box_sizing,
                            size: Size::new(
                                PreferredSizeOf::percent(S::ONE),
                                PreferredSizeOf::px(S::from_f64(10.0)),
                            ),
                            ..NodeInputOf::default()
                        },
                    )
                    .style(
                        5,
                        NodeInputOf {
                            display: Display::Flex,
                            direction,
                            box_sizing,
                            size: Size::new(
                                PreferredSizeOf::percent(S::ONE),
                                PreferredSizeOf::px(S::from_f64(10.0)),
                            ),
                            ..NodeInputOf::default()
                        },
                    )
                    .style(
                        6,
                        NodeInputOf {
                            display: Display::Flex,
                            direction,
                            box_sizing,
                            size: Size::splat(S::from_f64(10.0)).map(PreferredSizeOf::px),
                            ..NodeInputOf::default()
                        },
                    );
                for node in 1..=6 {
                    assert!(!tree.has_leaf_measurement(node));
                }
                let batch = compute(&tree);
                assert_nodes(&batch, &[1, 2, 3, 4, 5, 6]);
                assert_integer_rect(&batch, 1, [0.0, 0.0, 300.0, 110.0]);
                // All actual percentage descendants use the definite200px width.
                assert_integer_rect(&batch, 2, [50.0, 40.0, 200.0, 30.0]);
                for (ordinal, (node, y)) in [(3, 0.0), (4, 10.0), (5, 20.0)].into_iter().enumerate()
                {
                    assert_integer_rect(&batch, node, [0.0, y, 200.0, 10.0]);
                    assert_eq!(
                        output(batch.final_entries(), node).source_index,
                        SourceIndex::new(ordinal)
                    );
                }
                assert_integer_rect(
                    &batch,
                    6,
                    [
                        if direction == Direction::Ltr {
                            0.0
                        } else {
                            190.0
                        },
                        0.0,
                        10.0,
                        10.0,
                    ],
                );
                assert_eq!(
                    output(batch.final_entries(), 6).source_index,
                    SourceIndex::ZERO
                );
            }
        }
    }
}
#[test]
fn empty_axes_retain_real_percentage_subtree_f32() {
    percentage_subtree::<f32>();
}
#[test]
fn empty_axes_retain_real_percentage_subtree_f64() {
    percentage_subtree::<f64>();
}

fn percentage_padding_cb<S: LayoutScalar>() {
    for direction in [Direction::Ltr, Direction::Rtl] {
        for (box_sizing, outer, cb) in [
            (BoxSizing::BorderBox, 100.0, 80.0),
            (BoxSizing::ContentBox, 140.0, 120.0),
        ] {
            for tracks in TOPOLOGIES {
                let mut parent = grid::<S>(direction, box_sizing, Size::splat(100.0), tracks);
                parent.border = Edges::all(LengthOf::px(S::from_f64(10.0)));
                parent.padding = Edges::all(LengthOf::px(S::from_f64(10.0)));
                let mut fractional = fixed_absolute::<S>(direction, Size::ZERO);
                fractional.box_sizing = box_sizing;
                fractional.size = Size::splat(S::from_f64(0.5)).map(PreferredSizeOf::percent);
                fractional.inset = Edges {
                    left: LengthAutoOf::percent(S::from_f64(0.25)),
                    top: LengthAutoOf::percent(S::from_f64(0.25)),
                    ..Edges::all(LengthAutoOf::AUTO)
                };
                let mut far_start = fixed_absolute::<S>(direction, Size::ZERO);
                far_start.box_sizing = box_sizing;
                far_start.inset = Edges {
                    left: LengthAutoOf::percent(S::ONE),
                    top: LengthAutoOf::percent(S::ONE),
                    ..Edges::all(LengthAutoOf::AUTO)
                };
                let mut far_end = fixed_absolute::<S>(direction, Size::ZERO);
                far_end.box_sizing = box_sizing;
                far_end.inset = Edges {
                    right: LengthAutoOf::percent(S::ONE),
                    bottom: LengthAutoOf::percent(S::ONE),
                    ..Edges::all(LengthAutoOf::AUTO)
                };
                let mut fill = fixed_absolute::<S>(direction, Size::ZERO);
                fill.box_sizing = box_sizing;
                fill.size = Size::splat(S::ONE).map(PreferredSizeOf::percent);
                fill.inset = Edges {
                    left: LengthAutoOf::px(S::ZERO),
                    top: LengthAutoOf::px(S::ZERO),
                    ..Edges::all(LengthAutoOf::AUTO)
                };
                let mut offset = fixed_absolute::<S>(direction, Size::new(10.0, 12.0));
                offset.box_sizing = box_sizing;
                offset.inset = Edges {
                    left: LengthAutoOf::px(S::from_f64(30.0)),
                    top: LengthAutoOf::px(S::from_f64(30.0)),
                    ..Edges::all(LengthAutoOf::AUTO)
                };
                let tree = PublicLayoutTreeOf::new()
                    .children(1, [2, 3, 4, 5, 6])
                    .style(1, parent)
                    .children(2, [])
                    .style(2, fractional)
                    .children(3, [])
                    .style(3, far_start)
                    .children(4, [])
                    .style(4, far_end)
                    .children(5, [])
                    .style(5, fill)
                    .children(6, [])
                    .style(6, offset);
                let batch = compute(&tree);
                assert_nodes(&batch, &[1, 2, 3, 4, 5, 6]);
                assert_integer_rect(&batch, 1, [0.0, 0.0, outer, outer]);
                // Percent insets/sizes use the80/120 padding CB, not60/100 content.
                assert_integer_rect(
                    &batch,
                    2,
                    [10.0 + cb / 4.0, 10.0 + cb / 4.0, cb / 2.0, cb / 2.0],
                );
                assert_integer_rect(&batch, 3, [outer - 10.0, outer - 10.0, 0.0, 0.0]);
                assert_integer_rect(&batch, 4, [10.0, 10.0, 0.0, 0.0]);
                assert_integer_rect(&batch, 5, [10.0, 10.0, cb, cb]);
                assert_integer_rect(&batch, 6, [40.0, 40.0, 10.0, 12.0]);
            }
        }
    }
}
#[test]
fn empty_axes_percentage_insets_use_padding_cb_f32() {
    percentage_padding_cb::<f32>();
}
#[test]
fn empty_axes_percentage_insets_use_padding_cb_f64() {
    percentage_padding_cb::<f64>();
}

fn ratio_profiles<S: LayoutScalar>() {
    for direction in [Direction::Ltr, Direction::Rtl] {
        for box_sizing in [BoxSizing::BorderBox, BoxSizing::ContentBox] {
            for tracks in [Tracks::Both, Tracks::Neither] {
                let mut first = fixed_absolute::<S>(direction, Size::ZERO);
                first.box_sizing = box_sizing;
                first.size = Size::new(PreferredSizeOf::AUTO, PreferredSizeOf::AUTO);
                first.aspect_ratio = AspectRatioOf::new(S::from_f64(3.0));
                first.inset = Edges::all(LengthAutoOf::percent(S::from_f64(0.05)));
                let mut second = first.clone();
                second.inset.left = LengthAutoOf::percent(S::from_f64(0.1));
                second.inset.right = LengthAutoOf::percent(S::from_f64(0.1));
                second.inset.bottom = LengthAutoOf::AUTO;
                let mut third = second.clone();
                third.size.width = PreferredSizeOf::percent(S::from_f64(0.4));
                let tree = PublicLayoutTreeOf::new()
                    .children(1, [2, 3, 4])
                    .style(
                        1,
                        grid::<S>(direction, box_sizing, Size::new(400.0, 300.0), tracks),
                    )
                    .children(2, [])
                    .style(2, first)
                    .children(3, [])
                    .style(3, second)
                    .children(4, [])
                    .style(4, third);
                let batch = compute(&tree);
                assert_nodes(&batch, &[1, 2, 3, 4]);
                assert_integer_rect(&batch, 1, [0.0, 0.0, 400.0, 300.0]);
                // Full insets suggest270px height, but width360 and ratio3 yield120.
                assert_integer_rect(&batch, 2, [20.0, 15.0, 360.0, 120.0]);
                assert_rect(
                    output(batch.unrounded_entries(), 3),
                    [
                        S::from_f64(40.0),
                        S::from_f64(15.0),
                        S::from_f64(320.0),
                        S::from_f64(320.0) / S::from_f64(3.0),
                    ],
                );
                assert_rect(
                    output(batch.final_entries(), 3),
                    [
                        S::from_f64(40.0),
                        S::from_f64(15.0),
                        S::from_f64(320.0),
                        S::from_f64(107.0),
                    ],
                );
                let third_x = if direction == Direction::Ltr {
                    40.0
                } else {
                    200.0
                };
                assert_rect(
                    output(batch.unrounded_entries(), 4),
                    [
                        S::from_f64(third_x),
                        S::from_f64(15.0),
                        S::from_f64(160.0),
                        S::from_f64(160.0) / S::from_f64(3.0),
                    ],
                );
                assert_rect(
                    output(batch.final_entries(), 4),
                    [
                        S::from_f64(third_x),
                        S::from_f64(15.0),
                        S::from_f64(160.0),
                        S::from_f64(53.0),
                    ],
                );
            }
        }
    }
}
#[test]
fn empty_axes_ratio_precedence_retains_actual_boxes_f32() {
    ratio_profiles::<f32>();
}
#[test]
fn empty_axes_ratio_precedence_retains_actual_boxes_f64() {
    ratio_profiles::<f64>();
}

fn zero_padding_static<S: LayoutScalar>() {
    for direction in [Direction::Ltr, Direction::Rtl] {
        for alignment in [AlignItems::Start, AlignItems::End] {
            for tracks in TOPOLOGIES {
                let mut child = fixed_absolute::<S>(direction, Size::new(20.0, 10.0));
                child.align_self = Some(alignment);
                child.justify_self = Some(alignment);
                let tree = PublicLayoutTreeOf::new()
                    .children(1, [2])
                    .children(2, [])
                    .style(
                        1,
                        grid::<S>(
                            direction,
                            BoxSizing::BorderBox,
                            Size::new(100.0, 80.0),
                            tracks,
                        ),
                    )
                    .style(2, child);
                let batch = compute(&tree);
                let at_end = alignment == AlignItems::End;
                let x = if at_end == (direction == Direction::Ltr) {
                    80.0
                } else {
                    0.0
                };
                // Zero padding makes content and padding edges coincide. This
                // checks execution of auto insets, not the distinct padded rule.
                assert_integer_rect(&batch, 2, [x, if at_end { 70.0 } else { 0.0 }, 20.0, 10.0]);
                assert_integer_rect(&batch, 1, [0.0, 0.0, 100.0, 80.0]);
                assert_nodes(&batch, &[1, 2]);
            }
        }
    }
}
#[test]
fn empty_axes_zero_padding_auto_insets_f32() {
    zero_padding_static::<f32>();
}
#[test]
fn empty_axes_zero_padding_auto_insets_f64() {
    zero_padding_static::<f64>();
}

fn absolute_no_track_demand<S: LayoutScalar>() {
    for direction in [Direction::Ltr, Direction::Rtl] {
        for operation in [SizingOperation::MinContent, SizingOperation::MaxContent] {
            for placement in [
                GridPlacement::try_line(100).unwrap(),
                GridPlacement::try_span(4).unwrap(),
            ] {
                for in_flow in [false, true] {
                    for imposed_tracks in [false, true] {
                        let parent = NodeInputOf {
                            display: Display::Grid,
                            direction,
                            grid_template_columns: if imposed_tracks {
                                vec![TrackComponentOf::px(S::from_f64(31.0))]
                            } else {
                                Vec::new()
                            },
                            grid_template_rows: if imposed_tracks {
                                vec![TrackComponentOf::px(S::from_f64(37.0))]
                            } else {
                                Vec::new()
                            },
                            grid_auto_columns: vec![TrackComponentOf::px(S::from_f64(31.0))],
                            grid_auto_rows: vec![TrackComponentOf::px(S::from_f64(37.0))],
                            gap: Size::splat(LengthOf::px(S::from_f64(13.0))),
                            ..NodeInputOf::default()
                        };
                        let mut absolute = fixed_absolute::<S>(direction, Size::new(200.0, 30.0));
                        absolute.grid_column = placement;
                        absolute.grid_row = placement;
                        absolute.inset = Edges {
                            left: LengthAutoOf::px(S::from_f64(50.0)),
                            top: LengthAutoOf::px(S::from_f64(40.0)),
                            ..Edges::all(LengthAutoOf::AUTO)
                        };
                        let children = if in_flow { vec![2, 3, 4] } else { vec![2, 3] };
                        let mut tree = PublicLayoutTreeOf::new()
                            .children(1, children)
                            .style(1, parent)
                            .children(2, [])
                            .style(2, absolute)
                            .children(3, [])
                            .style(
                                3,
                                NodeInputOf {
                                    display: Display::None,
                                    direction,
                                    size: Size::splat(S::from_f64(999.0)).map(PreferredSizeOf::px),
                                    grid_column: GridPlacement::try_line(100).unwrap(),
                                    grid_row: GridPlacement::try_line(100).unwrap(),
                                    ..NodeInputOf::default()
                                },
                            );
                        if in_flow {
                            tree = tree.children(4, []).style(
                                4,
                                NodeInputOf {
                                    display: Display::Block,
                                    direction,
                                    size: Size::new(
                                        PreferredSizeOf::px(S::from_f64(7.0)),
                                        PreferredSizeOf::px(S::from_f64(5.0)),
                                    ),
                                    ..NodeInputOf::default()
                                },
                            );
                        }
                        let batch = compute_layout(&tree, 1, root_request::<S>(operation))
                            .expect("actual intrinsic native Grid request succeeds");
                        // Positive31/37 implicit patterns expose any invented demand.
                        assert_integer_rect(
                            &batch,
                            1,
                            if in_flow || imposed_tracks {
                                [0.0, 0.0, 31.0, 37.0]
                            } else {
                                [0.0, 0.0, 0.0, 0.0]
                            },
                        );
                        assert_integer_rect(&batch, 2, [50.0, 40.0, 200.0, 30.0]);
                        assert_integer_rect(&batch, 3, [0.0, 0.0, 0.0, 0.0]);
                        if in_flow {
                            assert_integer_rect(
                                &batch,
                                4,
                                [
                                    if direction == Direction::Ltr {
                                        0.0
                                    } else {
                                        24.0
                                    },
                                    0.0,
                                    7.0,
                                    5.0,
                                ],
                            );
                            assert_nodes(&batch, &[1, 2, 3, 4]);
                            assert_eq!(
                                output(batch.final_entries(), 4).source_index,
                                SourceIndex::new(2)
                            );
                        } else {
                            assert_nodes(&batch, &[1, 2, 3]);
                        }
                        assert_eq!(
                            output(batch.final_entries(), 2).source_index,
                            SourceIndex::ZERO
                        );
                        assert_eq!(
                            output(batch.final_entries(), 3).source_index,
                            SourceIndex::new(1)
                        );
                    }
                }
            }
        }
    }
}
#[test]
fn absolute_only_grids_create_no_tracks_or_intrinsic_demand_f32() {
    absolute_no_track_demand::<f32>();
}
#[test]
fn absolute_only_grids_create_no_tracks_or_intrinsic_demand_f64() {
    absolute_no_track_demand::<f64>();
}

fn assert_empty_grid_scroll_rect<S: LayoutScalar>(actual: ScrollRectOf<S>, expected: [f64; 4]) {
    assert_eq!(
        actual.origin(),
        Point::new(S::from_f64(expected[0]), S::from_f64(expected[1]))
    );
    assert_eq!(
        actual.size(),
        Size::new(S::from_f64(expected[2]), S::from_f64(expected[3]))
    );
}

fn explicit_scroll_padding_cb<S: LayoutScalar>() {
    for direction in [Direction::Ltr, Direction::Rtl] {
        for (box_sizing, outer, padding_extent, cb, content_extent) in [
            (BoxSizing::BorderBox, 200.0, 160.0, 145.0, 115.0),
            (BoxSizing::ContentBox, 270.0, 230.0, 215.0, 185.0),
        ] {
            for tracks in TOPOLOGIES {
                let mut parent = grid::<S>(direction, box_sizing, Size::splat(200.0), tracks);
                parent.border = Edges::all(LengthOf::px(S::from_f64(20.0)));
                parent.padding = Edges::all(LengthOf::px(S::from_f64(15.0)));
                parent.overflow =
                    ComputedOverflow::try_new(Overflow::Scroll, Overflow::Scroll).unwrap();
                parent.scrollbar_width = ScrollbarWidthOf::try_new(S::from_f64(15.0)).unwrap();

                let mut fixed = fixed_absolute::<S>(direction, Size::new(20.0, 10.0));
                fixed.box_sizing = box_sizing;
                fixed.inset = Edges {
                    left: LengthAutoOf::px(S::ZERO),
                    top: LengthAutoOf::px(S::ZERO),
                    ..Edges::all(LengthAutoOf::AUTO)
                };
                let mut fill = fixed_absolute::<S>(direction, Size::ZERO);
                fill.box_sizing = box_sizing;
                fill.size = Size::splat(S::ONE).map(PreferredSizeOf::percent);
                fill.inset = fixed.inset;
                let mut half = fixed_absolute::<S>(direction, Size::ZERO);
                half.box_sizing = box_sizing;
                half.size = Size::splat(S::from_f64(0.5)).map(PreferredSizeOf::percent);
                half.inset = Edges {
                    left: LengthAutoOf::percent(S::from_f64(0.25)),
                    top: LengthAutoOf::percent(S::from_f64(0.25)),
                    ..Edges::all(LengthAutoOf::AUTO)
                };
                let mut far_start = fixed_absolute::<S>(direction, Size::ZERO);
                far_start.box_sizing = box_sizing;
                far_start.inset = Edges {
                    left: LengthAutoOf::percent(S::ONE),
                    top: LengthAutoOf::percent(S::ONE),
                    ..Edges::all(LengthAutoOf::AUTO)
                };
                let mut far_end = fixed_absolute::<S>(direction, Size::ZERO);
                far_end.box_sizing = box_sizing;
                far_end.inset = Edges {
                    right: LengthAutoOf::percent(S::ONE),
                    bottom: LengthAutoOf::percent(S::ONE),
                    ..Edges::all(LengthAutoOf::AUTO)
                };
                let tree = PublicLayoutTreeOf::new()
                    .children(1, [2, 3, 4, 5, 6])
                    .style(1, parent)
                    .children(2, [])
                    .style(2, fixed)
                    .children(3, [])
                    .style(3, fill)
                    .children(4, [])
                    .style(4, half)
                    .children(5, [])
                    .style(5, far_start)
                    .children(6, [])
                    .style(6, far_end);
                for node in 1..=6 {
                    assert!(!tree.has_leaf_measurement(node));
                }
                let batch = compute(&tree);
                assert_nodes(&batch, &[1, 2, 3, 4, 5, 6]);
                assert_integer_rect(&batch, 1, [0.0, 0.0, outer, outer]);
                let cb_x = if direction == Direction::Ltr {
                    20.0
                } else {
                    35.0
                };
                let vertical_gutter_x = if direction == Direction::Ltr {
                    outer - 35.0
                } else {
                    20.0
                };
                for entries in [batch.unrounded_entries(), batch.final_entries()] {
                    let root_output = output(entries, 1);
                    let scroll = root_output
                        .scroll_geometry
                        .expect("actual Grid scroll geometry");
                    assert_eq!(
                        scroll.flow_axes(),
                        FlowAxes::new(WritingMode::HorizontalTb, direction)
                    );
                    assert_eq!(scroll.used_overflow_x(), Overflow::Scroll);
                    assert_eq!(scroll.used_overflow_y(), Overflow::Scroll);
                    assert_eq!(root_output.border, Edges::all(S::from_f64(20.0)));
                    assert_eq!(root_output.padding, Edges::all(S::from_f64(15.0)));
                    assert_eq!(root_output.scrollbar_size(), Size::splat(S::from_f64(15.0)));
                    assert_eq!(scroll.scrollbar_size(), Size::splat(S::from_f64(15.0)));
                    assert_empty_grid_scroll_rect(scroll.border_box(), [0.0, 0.0, outer, outer]);
                    // Public padding_box precedes scrollbar reservation; the actual
                    // padding containing block excludes its15px gutters.
                    assert_empty_grid_scroll_rect(
                        scroll.padding_box(),
                        [20.0, 20.0, padding_extent, padding_extent],
                    );
                    assert_empty_grid_scroll_rect(scroll.scrollport(), [cb_x, 20.0, cb, cb]);
                    assert_empty_grid_scroll_rect(
                        scroll.content_box(),
                        [cb_x + 15.0, 35.0, content_extent, content_extent],
                    );
                    assert_eq!(scroll.gutters().top(), None);
                    let (vertical, absent) = if direction == Direction::Ltr {
                        (scroll.gutters().right(), scroll.gutters().left())
                    } else {
                        (scroll.gutters().left(), scroll.gutters().right())
                    };
                    assert_eq!(absent, None);
                    assert_empty_grid_scroll_rect(
                        vertical.expect("actual inline-end scrollbar"),
                        [vertical_gutter_x, 20.0, 15.0, cb],
                    );
                    assert_empty_grid_scroll_rect(
                        scroll.gutters().bottom().expect("actual bottom scrollbar"),
                        [cb_x, outer - 35.0, cb, 15.0],
                    );
                    // All actual out-of-flow border boxes lie within the scrollport.
                    // They nominate no in-flow terminal-padding extension. The
                    // public visual seed still retains the full padding rectangle;
                    // reserved gutter alone must not manufacture a scroll range.
                    assert_empty_grid_scroll_rect(
                        scroll.scrollable_overflow(),
                        [20.0, 20.0, padding_extent, padding_extent],
                    );
                    assert_eq!(scroll.physical_range().x().minimum(), S::ZERO);
                    assert_eq!(scroll.physical_range().x().maximum(), S::ZERO);
                    assert_eq!(scroll.physical_range().y().minimum(), S::ZERO);
                    assert_eq!(scroll.physical_range().y().maximum(), S::ZERO);
                }
                assert_integer_rect(&batch, 2, [cb_x, 20.0, 20.0, 10.0]);
                assert_integer_rect(&batch, 3, [cb_x, 20.0, cb, cb]);
                // All fractions are exact binary quarters/halves in both lanes.
                // Quarter inset + half size gives independent rounded endpoints:
                // BB56.25..128.75 (or71.25..143.75), CB73.75..181.25
                // (or88.75..196.25). Widths are73 and107, not rounded sizes.
                assert_rect(
                    output(batch.unrounded_entries(), 4),
                    [
                        S::from_f64(cb_x + cb / 4.0),
                        S::from_f64(20.0 + cb / 4.0),
                        S::from_f64(cb / 2.0),
                        S::from_f64(cb / 2.0),
                    ],
                );
                let (rounded_x, rounded_y, rounded_extent) = match (box_sizing, direction) {
                    (BoxSizing::BorderBox, Direction::Ltr) => (56.0, 56.0, 73.0),
                    (BoxSizing::BorderBox, Direction::Rtl) => (71.0, 56.0, 73.0),
                    (BoxSizing::ContentBox, Direction::Ltr) => (74.0, 74.0, 107.0),
                    (BoxSizing::ContentBox, Direction::Rtl) => (89.0, 74.0, 107.0),
                };
                assert_rect(
                    output(batch.final_entries(), 4),
                    [
                        S::from_f64(rounded_x),
                        S::from_f64(rounded_y),
                        S::from_f64(rounded_extent),
                        S::from_f64(rounded_extent),
                    ],
                );
                assert_integer_rect(&batch, 5, [cb_x + cb, 20.0 + cb, 0.0, 0.0]);
                assert_integer_rect(&batch, 6, [cb_x, 20.0, 0.0, 0.0]);
                for (ordinal, node) in [2, 3, 4, 5, 6].into_iter().enumerate() {
                    assert_eq!(
                        output(batch.final_entries(), node).source_index,
                        SourceIndex::new(ordinal)
                    );
                }
            }
        }
    }
}

#[test]
fn empty_axes_explicit_scroll_padding_cb_f32() {
    explicit_scroll_padding_cb::<f32>();
}

#[test]
fn empty_axes_explicit_scroll_padding_cb_f64() {
    explicit_scroll_padding_cb::<f64>();
}

fn sole_line_uses_axis_start<S: LayoutScalar>() {
    for direction in [Direction::Ltr, Direction::Rtl] {
        for tracks in TOPOLOGIES {
            let mut parent = grid::<S>(
                direction,
                BoxSizing::BorderBox,
                Size::new(100.0, 80.0),
                tracks,
            );
            parent.justify_content = Some(AlignContent::Center);
            parent.align_content = Some(AlignContent::End);
            let mut child = fixed_absolute::<S>(direction, Size::new(20.0, 10.0));
            child.grid_column = GridPlacement::try_line(1).unwrap();
            child.grid_row = GridPlacement::try_line(1).unwrap();
            child.inset = Edges {
                left: if direction == Direction::Ltr {
                    LengthAutoOf::px(S::from_f64(7.0))
                } else {
                    LengthAutoOf::AUTO
                },
                right: if direction == Direction::Rtl {
                    LengthAutoOf::px(S::from_f64(7.0))
                } else {
                    LengthAutoOf::AUTO
                },
                top: LengthAutoOf::px(S::from_f64(11.0)),
                bottom: LengthAutoOf::AUTO,
            };
            let tree = PublicLayoutTreeOf::new()
                .children(1, [2])
                .style(1, parent)
                .children(2, [])
                .style(2, child);
            let batch = compute(&tree);
            // Grid2 §11.3: an empty axis has one line at its start edge,
            // regardless of content alignment. Populated axes still align
            // their real32/24 tracks: (100-32)/2=34 and80-24=56.
            let inline_start = if matches!(tracks, Tracks::Both | Tracks::ColumnsOnly) {
                34.0
            } else {
                0.0
            };
            let block_start = if matches!(tracks, Tracks::Both | Tracks::RowsOnly) {
                56.0
            } else {
                0.0
            };
            let x = if direction == Direction::Ltr {
                inline_start + 7.0
            } else {
                100.0 - inline_start - 7.0 - 20.0
            };
            assert_integer_rect(&batch, 1, [0.0, 0.0, 100.0, 80.0]);
            assert_integer_rect(&batch, 2, [x, block_start + 11.0, 20.0, 10.0]);
            assert_nodes(&batch, &[1, 2]);
        }
    }
}

#[test]
fn empty_axis_sole_line_stays_at_start_despite_content_alignment_f32() {
    sole_line_uses_axis_start::<f32>();
}

#[test]
fn empty_axis_sole_line_stays_at_start_despite_content_alignment_f64() {
    sole_line_uses_axis_start::<f64>();
}

fn grid_static_position_uses_content_edges<S: LayoutScalar>() {
    for (box_sizing, size) in [
        (BoxSizing::BorderBox, Size::new(100.0, 80.0)),
        (BoxSizing::ContentBox, Size::new(60.0, 40.0)),
    ] {
        for direction in [Direction::Ltr, Direction::Rtl] {
            for has_tracks in [false, true] {
                let mut parent = grid::<S>(direction, box_sizing, size, Tracks::Neither);
                parent.border = Edges::all(LengthOf::px(S::from_f64(10.0)));
                parent.padding = Edges::all(LengthOf::px(S::from_f64(10.0)));
                parent.justify_content = Some(AlignContent::Start);
                parent.align_content = Some(AlignContent::Start);
                let mut start = fixed_absolute::<S>(direction, Size::new(20.0, 10.0));
                if has_tracks {
                    parent.grid_template_columns = [24.0, 32.0]
                        .map(|extent| TrackComponentOf::px(S::from_f64(extent)))
                        .to_vec();
                    parent.grid_template_rows = [12.0, 20.0]
                        .map(|extent| TrackComponentOf::px(S::from_f64(extent)))
                        .to_vec();
                    start.grid_column = GridPlacement::try_lines(2, 3).unwrap();
                    start.grid_row = GridPlacement::try_lines(2, 3).unwrap();
                }
                let mut end = start.clone();
                end.justify_self = Some(AlignItems::End);
                end.align_self = Some(AlignItems::End);
                let mut explicit = start.clone();
                explicit.inset.left = LengthAutoOf::px(S::from_f64(3.0));
                explicit.inset.top = LengthAutoOf::px(S::from_f64(5.0));
                let mut mixed = end.clone();
                mixed.inset.left = explicit.inset.left;
                let tree = PublicLayoutTreeOf::new()
                    .children(1, [2, 3, 4, 5])
                    .style(1, parent)
                    .children(2, [])
                    .style(2, start)
                    .children(3, [])
                    .style(3, end)
                    .children(4, [])
                    .style(4, explicit)
                    .children(5, [])
                    .style(5, mixed);
                let batch = compute(&tree);

                let ltr = direction == Direction::Ltr;
                let (inset_x, inset_y) = match (has_tracks, ltr) {
                    (false, _) => (13.0, 15.0),
                    (true, true) => (47.0, 37.0),
                    (true, false) => (27.0, 37.0),
                };
                assert_nodes(&batch, &[1, 2, 3, 4, 5]);
                assert_integer_rect(&batch, 1, [0.0, 0.0, 100.0, 80.0]);
                assert_integer_rect(&batch, 4, [inset_x, inset_y, 20.0, 10.0]);
                // The 60×40 content rectangle starts at (20, 20) in both box modes.
                let (start_x, end_x) = if ltr { (20.0, 60.0) } else { (60.0, 20.0) };
                assert_integer_rect(&batch, 2, [start_x, 20.0, 20.0, 10.0]);
                assert_integer_rect(&batch, 3, [end_x, 50.0, 20.0, 10.0]);
                assert_integer_rect(&batch, 5, [inset_x, 50.0, 20.0, 10.0]);
            }
        }
    }
}

#[test]
fn grid_static_position_uses_content_edges_f32() {
    grid_static_position_uses_content_edges::<f32>();
}

#[test]
fn grid_static_position_uses_content_edges_f64() {
    grid_static_position_uses_content_edges::<f64>();
}

#[test]
fn grid_static_position_checks_safe_alignment_against_content_extent() {
    let mut parent = grid::<f32>(
        Direction::Ltr,
        BoxSizing::BorderBox,
        Size::new(100.0, 80.0),
        Tracks::Both,
    );
    parent.border = Edges::all(LengthOf::px(10.0));
    parent.padding = Edges::all(LengthOf::px(10.0));
    let mut child = fixed_absolute::<f32>(Direction::Ltr, Size::new(40.0, 30.0));
    child.grid_column = GridPlacement::try_lines(1, 2).unwrap();
    child.grid_row = GridPlacement::try_lines(1, 2).unwrap();
    child.justify_self = Some(AlignItems::SafeEnd);
    child.align_self = Some(AlignItems::SafeEnd);
    let tree = PublicLayoutTreeOf::new()
        .children(1, [2])
        .style(1, parent)
        .children(2, [])
        .style(2, child);
    // The child fits the 60×40 content box even though its 32×24 inset area is smaller.
    assert_integer_rect(&compute(&tree), 2, [40.0, 30.0, 40.0, 30.0]);
}
