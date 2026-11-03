//! Inline cyclic percentages resolve for contents after the Flex width is accepted.
//! Intrinsic contributions still treat those percentages as initial; computed
//! percentage cross sizes never acquire computed-Auto stretch eligibility.
use super::*;
use crate::test_support::requests::NativeRootFrameOf;

fn scalar<S: LayoutScalar>(value: f64) -> S {
    S::from_f64(value)
}

fn physical_size<T>(mode: WritingMode, inline: T, block: T) -> Size<T> {
    match mode {
        WritingMode::HorizontalTb => Size::new(inline, block),
        WritingMode::VerticalLr | WritingMode::VerticalRl => Size::new(block, inline),
        _ => panic!("this fixture declares only horizontal-tb and vertical-lr/rl"),
    }
}

fn request<S: LayoutScalar>(
    mode: WritingMode,
    operation: SizingOperation,
) -> LayoutRootRequestOf<S> {
    let viewport = Size::new(
        NonNegativeFiniteOf::new(scalar(600.0)).unwrap(),
        NonNegativeFiniteOf::new(scalar(400.0)).unwrap(),
    );
    // This native host explicitly supplies an indefinite percentage parent.
    // The finite ICB is fallback authority, not a percentage basis/allocation.
    let frame = NativeRootFrameOf::new(
        FlowAxes::new(mode, Direction::Ltr),
        Size::NONE,
        OrthogonalFallbackOf::new(
            viewport.map(Some),
            NearestScrollportOf::Absent,
            Size::splat(OrthogonalFallbackLimitOf::Unbounded),
        ),
    );
    // Finite physical space and an intrinsic operation are independent facts.
    // Normal uses no host allocation; its auto inline size is content-derived.
    let inline_space = if operation == SizingOperation::Normal {
        AvailableSpaceOf::Indefinite
    } else {
        AvailableSpaceOf::Definite(NonNegativeFiniteOf::new(scalar(600.0)).unwrap())
    };
    frame.request(physical_size(
        mode,
        AxisSizingRequestOf::new(inline_space, operation),
        AxisSizingRequestOf::new(AvailableSpaceOf::Indefinite, SizingOperation::Normal),
    ))
}

fn native_box<S: LayoutScalar>(
    mode: WritingMode,
    direction: Direction,
    box_sizing: BoxSizing,
    inline: PreferredSizeOf<S>,
    block: PreferredSizeOf<S>,
) -> NodeInputOf<S> {
    NodeInputOf {
        display: Display::Flex,
        writing_mode: mode,
        direction,
        box_sizing,
        size: physical_size(mode, inline, block),
        ..NodeInputOf::default()
    }
}

fn subtree<S: LayoutScalar>(
    mode: WritingMode,
    direction: Direction,
    box_sizing: BoxSizing,
    percentage: f64,
    definite: bool,
    maximum: bool,
) -> PublicLayoutTreeOf<S> {
    let mut parent = native_box(
        mode,
        direction,
        box_sizing,
        if definite {
            PreferredSizeOf::px(scalar(200.0))
        } else {
            PreferredSizeOf::AUTO
        },
        PreferredSizeOf::AUTO,
    );
    parent.flex_direction = FlexDirection::Column;
    if maximum {
        parent.max_size = physical_size(mode, MaxSizeOf::px(scalar(300.0)), MaxSizeOf::NONE);
    }
    PublicLayoutTreeOf::new()
        .children(2, [3, 4, 5])
        .children(3, [])
        .children(4, [])
        .children(5, [6])
        .children(6, [])
        .style(2, parent)
        .style(
            3,
            native_box(
                mode,
                direction,
                box_sizing,
                PreferredSizeOf::px(scalar(200.0)),
                PreferredSizeOf::px(scalar(10.0)),
            ),
        )
        .style(
            4,
            native_box(
                mode,
                direction,
                box_sizing,
                PreferredSizeOf::percent(scalar(percentage)),
                PreferredSizeOf::px(scalar(10.0)),
            ),
        )
        .style(
            5,
            native_box(
                mode,
                direction,
                box_sizing,
                PreferredSizeOf::percent(scalar(percentage)),
                PreferredSizeOf::px(scalar(10.0)),
            ),
        )
        .style(
            6,
            native_box(
                mode,
                direction,
                box_sizing,
                PreferredSizeOf::px(scalar(10.0)),
                PreferredSizeOf::px(scalar(10.0)),
            ),
        )
}

fn qualify_inputs<S: LayoutScalar>(
    tree: &PublicLayoutTreeOf<S>,
    mode: WritingMode,
    direction: Direction,
    box_sizing: BoxSizing,
    percentage: f64,
    definite: bool,
) {
    assert_eq!(Traverse::children(tree, 2).collect::<Vec<_>>(), [3, 4, 5]);
    assert_eq!(Traverse::children(tree, 5).collect::<Vec<_>>(), [6]);
    for node in 2..=6 {
        let input = tree.node_input(node);
        assert_eq!(input.display, Display::Flex);
        assert_eq!(input.writing_mode, mode);
        assert_eq!(input.direction, direction);
        assert_eq!(input.box_sizing, box_sizing);
        assert_eq!(input.margin, Edges::all(LengthAutoOf::ZERO));
        assert_eq!(input.border, Edges::all(LengthOf::ZERO));
        assert_eq!(input.padding, Edges::all(LengthOf::ZERO));
        assert!(
            !tree.has_leaf_measurement(node),
            "real unmeasured native box {node}"
        );
    }
    assert_eq!(tree.node_input(2).flex_direction, FlexDirection::Column);
    let preferred_inline = |node| {
        let size = &tree.node_input(node).size;
        if mode == WritingMode::HorizontalTb {
            &size.width
        } else {
            &size.height
        }
    };
    assert_eq!(
        preferred_inline(2),
        &if definite {
            PreferredSizeOf::px(scalar(200.0))
        } else {
            PreferredSizeOf::AUTO
        }
    );
    assert_eq!(preferred_inline(3), &PreferredSizeOf::px(scalar(200.0)));
    assert_eq!(
        preferred_inline(4),
        &PreferredSizeOf::percent(scalar(percentage))
    );
    assert_eq!(
        preferred_inline(5),
        &PreferredSizeOf::percent(scalar(percentage))
    );
    assert!(!preferred_inline(4).is_auto());
    assert!(!preferred_inline(5).is_auto());
}

fn output<S: LayoutScalar>(entries: &[LayoutOutputEntryOf<u32, S>], node: u32) -> NodeOutputOf<S> {
    entries
        .iter()
        .find(|entry| entry.node() == node)
        .unwrap_or_else(|| panic!("missing native output {node}"))
        .output()
}

#[track_caller]
fn assert_box<S: LayoutScalar>(
    entries: &[LayoutOutputEntryOf<u32, S>],
    node: u32,
    rect: [f64; 4],
    source: usize,
) {
    let actual = output(entries, node);
    let [x, y, width, height] = rect.map(scalar::<S>);
    assert_eq!(
        actual.location,
        Point::new(x, y),
        "node{node} physical local origin"
    );
    assert_eq!(
        actual.size,
        Size::new(width, height),
        "node{node} border size"
    );
    assert_eq!(
        actual.content_box_size(),
        Size::new(width, height),
        "node{node} zero-edge content box"
    );
    assert_eq!(actual.source_index, SourceIndex::new(source));
    assert_eq!(
        (actual.margin, actual.border, actual.padding),
        (Edges::ZERO, Edges::ZERO, Edges::ZERO)
    );
}

fn assert_subtree<S: LayoutScalar>(
    batch: &CompletedLayoutBatchOf<u32, S>,
    mode: WritingMode,
    direction: Direction,
    percentage: f64,
    absolute: bool,
) {
    // Fixed200 dominates both intrinsic contributions. No percentage iteration
    // changes that accepted inline extent. The three fixed block10 items sum30.
    let inline = 200.0 * percentage;
    let inline_at = if direction == Direction::Ltr {
        0.0
    } else {
        200.0 - inline
    };
    let nested_at = if direction == Direction::Ltr {
        0.0
    } else {
        inline - 10.0
    };
    for entries in [batch.unrounded_entries(), batch.final_entries()] {
        let mut nodes = entries
            .iter()
            .map(LayoutOutputEntryOf::node)
            .collect::<Vec<_>>();
        nodes.sort_unstable();
        assert_eq!(
            nodes,
            if absolute {
                vec![1, 2, 3, 4, 5, 6]
            } else {
                vec![2, 3, 4, 5, 6]
            }
        );
        if absolute {
            assert_box(entries, 1, [0.0, 0.0, 300.0, 110.0], 0);
            assert_box(entries, 2, [50.0, 40.0, 200.0, 30.0], 0);
        } else {
            assert_box(
                entries,
                2,
                if mode == WritingMode::HorizontalTb {
                    [0.0, 0.0, 200.0, 30.0]
                } else {
                    [0.0, 0.0, 30.0, 200.0]
                },
                0,
            );
        }
        // Explicit physical-side tables; no algorithm-produced coordinates form
        // the oracle. Vertical-rl block progression is physically decreasing.
        match mode {
            WritingMode::HorizontalTb => {
                assert_box(entries, 3, [0.0, 0.0, 200.0, 10.0], 0);
                assert_box(entries, 4, [inline_at, 10.0, inline, 10.0], 1);
                assert_box(entries, 5, [inline_at, 20.0, inline, 10.0], 2);
                assert_box(entries, 6, [nested_at, 0.0, 10.0, 10.0], 0);
            }
            WritingMode::VerticalLr => {
                assert_box(entries, 3, [0.0, 0.0, 10.0, 200.0], 0);
                assert_box(entries, 4, [10.0, inline_at, 10.0, inline], 1);
                assert_box(entries, 5, [20.0, inline_at, 10.0, inline], 2);
                assert_box(entries, 6, [0.0, nested_at, 10.0, 10.0], 0);
            }
            WritingMode::VerticalRl => {
                assert_box(entries, 3, [20.0, 0.0, 10.0, 200.0], 0);
                assert_box(entries, 4, [10.0, inline_at, 10.0, inline], 1);
                assert_box(entries, 5, [0.0, inline_at, 10.0, inline], 2);
                assert_box(entries, 6, [0.0, nested_at, 10.0, 10.0], 0);
            }
            _ => unreachable!(),
        }
    }
}

fn absolute_grid<S: LayoutScalar>(percentage: f64) {
    for direction in [Direction::Ltr, Direction::Rtl] {
        for box_sizing in [BoxSizing::BorderBox, BoxSizing::ContentBox] {
            // Populated controls isolate this from empty-track traversal. The
            // original empty Grid remains a distinct complete composition case.
            for populated in [true, false] {
                for definite in [true, false] {
                    let mut tree = subtree::<S>(
                        WritingMode::HorizontalTb,
                        direction,
                        box_sizing,
                        percentage,
                        definite,
                        false,
                    );
                    let mut parent = tree.node_input(2).clone();
                    parent.position = Position::Absolute;
                    parent.inset = Edges {
                        left: LengthAutoOf::px(scalar(50.0)),
                        top: LengthAutoOf::px(scalar(40.0)),
                        ..Edges::all(LengthAutoOf::AUTO)
                    };
                    tree = tree.style(2, parent).children(1, [2]).style(
                        1,
                        NodeInputOf {
                            display: Display::Grid,
                            direction,
                            box_sizing,
                            size: Size::new(
                                PreferredSizeOf::px(scalar(300.0)),
                                PreferredSizeOf::px(scalar(110.0)),
                            ),
                            grid_template_columns: if populated {
                                vec![TrackComponentOf::px(scalar(32.0))]
                            } else {
                                Vec::new()
                            },
                            grid_template_rows: if populated {
                                vec![TrackComponentOf::px(scalar(24.0))]
                            } else {
                                Vec::new()
                            },
                            ..NodeInputOf::default()
                        },
                    );
                    qualify_inputs(
                        &tree,
                        WritingMode::HorizontalTb,
                        direction,
                        box_sizing,
                        percentage,
                        definite,
                    );
                    assert_eq!(tree.node_input(2).position, Position::Absolute);
                    assert!(!tree.has_leaf_measurement(1));
                    let batch = compute_layout(
                        &tree,
                        1,
                        request(WritingMode::HorizontalTb, SizingOperation::Normal),
                    )
                    .expect("actual Grid absolute/Flex tree is admitted");
                    assert_subtree(
                        &batch,
                        WritingMode::HorizontalTb,
                        direction,
                        percentage,
                        true,
                    );
                }
            }
        }
    }
}

fn direct<S: LayoutScalar>(modes: &[WritingMode], operations: &[SizingOperation]) {
    for &mode in modes {
        for direction in [Direction::Ltr, Direction::Rtl] {
            for box_sizing in [BoxSizing::BorderBox, BoxSizing::ContentBox] {
                for percentage in [1.0, 0.5] {
                    for maximum in [false, true] {
                        let tree =
                            subtree::<S>(mode, direction, box_sizing, percentage, false, maximum);
                        qualify_inputs(&tree, mode, direction, box_sizing, percentage, false);
                        if maximum {
                            assert_eq!(
                                tree.node_input(2).max_size,
                                physical_size(mode, MaxSizeOf::px(scalar(300.0)), MaxSizeOf::NONE)
                            );
                        }
                        for &operation in operations {
                            let query = request(mode, operation);
                            let inline_query = if mode == WritingMode::HorizontalTb {
                                query.sizing_request().axes().width
                            } else {
                                query.sizing_request().axes().height
                            };
                            assert_eq!(inline_query.operation(), operation);
                            assert_eq!(
                                inline_query.space(),
                                if operation == SizingOperation::Normal {
                                    AvailableSpaceOf::Indefinite
                                } else {
                                    AvailableSpaceOf::Definite(
                                        NonNegativeFiniteOf::new(scalar(600.0)).unwrap(),
                                    )
                                }
                            );
                            assert_eq!(
                                query.context(),
                                LayoutRootContextOf::Standalone {
                                    containing_flow_axes: FlowAxes::new(mode, Direction::Ltr),
                                    percentage_parent: Size::NONE,
                                }
                            );
                            let batch = compute_layout(&tree, 2, query)
                                .expect("direct native Flex request");
                            if operation == SizingOperation::Normal {
                                assert_subtree(&batch, mode, direction, percentage, false);
                            } else {
                                // A formal intrinsic query remains intrinsic even
                                // when PerformLayout publishes its geometry. The
                                // fixed200 item supplies this contribution; the
                                // external600 bound is not its percentage basis.
                                // Descendant Normal geometry needs a separate
                                // Normal request and is checked above.
                                for entries in [batch.unrounded_entries(), batch.final_entries()] {
                                    assert_box(
                                        entries,
                                        2,
                                        if mode == WritingMode::HorizontalTb {
                                            [0.0, 0.0, 200.0, 30.0]
                                        } else {
                                            [0.0, 0.0, 30.0, 200.0]
                                        },
                                        0,
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

fn block_cycle<S: LayoutScalar>() {
    // Sizing3's block-axis exception remains separate from the inline case.
    // A fixed40 sibling sets the natural line height; a computed50% item with
    // a real10px descendant behaves as Auto, never Stretch-to40 or50%-of100.
    for direction in [Direction::Ltr, Direction::Rtl] {
        for box_sizing in [BoxSizing::BorderBox, BoxSizing::ContentBox] {
            for maximum in [false, true] {
                let mut parent = native_box::<S>(
                    WritingMode::HorizontalTb,
                    direction,
                    box_sizing,
                    PreferredSizeOf::px(scalar(200.0)),
                    PreferredSizeOf::AUTO,
                );
                if maximum {
                    parent.max_size.height = MaxSizeOf::px(scalar(100.0));
                }
                let tree = PublicLayoutTreeOf::new()
                    .children(2, [3, 4])
                    .children(3, [])
                    .children(4, [6])
                    .children(6, [])
                    .style(2, parent)
                    .style(
                        3,
                        native_box(
                            WritingMode::HorizontalTb,
                            direction,
                            box_sizing,
                            PreferredSizeOf::px(scalar(20.0)),
                            PreferredSizeOf::px(scalar(40.0)),
                        ),
                    )
                    .style(
                        4,
                        native_box(
                            WritingMode::HorizontalTb,
                            direction,
                            box_sizing,
                            PreferredSizeOf::px(scalar(20.0)),
                            PreferredSizeOf::percent(scalar(0.5)),
                        ),
                    )
                    .style(
                        6,
                        native_box(
                            WritingMode::HorizontalTb,
                            direction,
                            box_sizing,
                            PreferredSizeOf::px(scalar(10.0)),
                            PreferredSizeOf::px(scalar(10.0)),
                        ),
                    );
                assert!(!tree.node_input(4).size.height.is_auto());
                for node in [2, 3, 4, 6] {
                    assert!(!tree.has_leaf_measurement(node));
                }
                let batch = compute_layout(
                    &tree,
                    2,
                    request(WritingMode::HorizontalTb, SizingOperation::Normal),
                )
                .expect("native block-axis cyclic-percentage control");
                for entries in [batch.unrounded_entries(), batch.final_entries()] {
                    assert_box(entries, 2, [0.0, 0.0, 200.0, 40.0], 0);
                    assert_box(
                        entries,
                        3,
                        [
                            if direction == Direction::Ltr {
                                0.0
                            } else {
                                180.0
                            },
                            0.0,
                            20.0,
                            40.0,
                        ],
                        0,
                    );
                    assert_box(
                        entries,
                        4,
                        [
                            if direction == Direction::Ltr {
                                20.0
                            } else {
                                160.0
                            },
                            0.0,
                            20.0,
                            10.0,
                        ],
                        1,
                    );
                    assert_box(
                        entries,
                        6,
                        [
                            if direction == Direction::Ltr {
                                0.0
                            } else {
                                10.0
                            },
                            0.0,
                            10.0,
                            10.0,
                        ],
                        0,
                    );
                }
            }
        }
    }
}

#[test]
fn absolute_grid_auto_inline_percentage_100_f32() {
    absolute_grid::<f32>(1.0);
}
#[test]
fn absolute_grid_auto_inline_percentage_100_f64() {
    absolute_grid::<f64>(1.0);
}
#[test]
fn absolute_grid_auto_inline_percentage_50_f32() {
    absolute_grid::<f32>(0.5);
}
#[test]
fn absolute_grid_auto_inline_percentage_50_f64() {
    absolute_grid::<f64>(0.5);
}
#[test]
fn direct_auto_inline_normal_completion_f32() {
    direct::<f32>(&[WritingMode::HorizontalTb], &[SizingOperation::Normal]);
}
#[test]
fn direct_auto_inline_normal_completion_f64() {
    direct::<f64>(&[WritingMode::HorizontalTb], &[SizingOperation::Normal]);
}
#[test]
fn direct_intrinsic_inline_extent_does_not_borrow_available_bound_f32() {
    direct::<f32>(
        &[WritingMode::HorizontalTb],
        &[SizingOperation::MinContent, SizingOperation::MaxContent],
    );
}
#[test]
fn direct_intrinsic_inline_extent_does_not_borrow_available_bound_f64() {
    direct::<f64>(
        &[WritingMode::HorizontalTb],
        &[SizingOperation::MinContent, SizingOperation::MaxContent],
    );
}
#[test]
fn vertical_inline_percentage_is_not_block_percentage_f32() {
    direct::<f32>(
        &[WritingMode::VerticalLr, WritingMode::VerticalRl],
        &[SizingOperation::Normal],
    );
}
#[test]
fn vertical_inline_percentage_is_not_block_percentage_f64() {
    direct::<f64>(
        &[WritingMode::VerticalLr, WritingMode::VerticalRl],
        &[SizingOperation::Normal],
    );
}
#[test]
fn auto_and_max_only_block_cycle_stays_automatic_f32() {
    block_cycle::<f32>();
}
#[test]
fn auto_and_max_only_block_cycle_stays_automatic_f64() {
    block_cycle::<f64>();
}
