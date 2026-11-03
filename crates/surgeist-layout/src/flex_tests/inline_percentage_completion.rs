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

fn assert_basis_rect<S: LayoutScalar>(rect: ScrollRectOf<S>, expected: [f64; 4]) {
    let [x, y, width, height] = expected.map(scalar::<S>);
    assert_eq!(rect.origin(), Point::new(x, y));
    assert_eq!(rect.size(), Size::new(width, height));
}

fn fractional_content_basis<S: LayoutScalar>() {
    for direction in [Direction::Ltr, Direction::Rtl] {
        for (box_sizing, outer_width, content_width, half) in [
            (BoxSizing::BorderBox, 150.5, 125.5, 62.75),
            (BoxSizing::ContentBox, 160.5, 135.5, 67.75),
        ] {
            for definite in [true, false] {
                let mut tree = subtree::<S>(
                    WritingMode::HorizontalTb,
                    direction,
                    box_sizing,
                    0.5,
                    false,
                    false,
                );
                let mut parent = tree.node_input(2).clone();
                parent.size.width = if definite {
                    PreferredSizeOf::px(scalar(150.5))
                } else {
                    PreferredSizeOf::AUTO
                };
                parent.max_size.width = MaxSizeOf::px(scalar(150.5));
                parent.border = Edges::all(LengthOf::px(scalar(2.0)));
                parent.padding = Edges::all(LengthOf::px(scalar(3.0)));
                parent.overflow =
                    ComputedOverflow::try_new(Overflow::Scroll, Overflow::Scroll).unwrap();
                parent.scrollbar_width = ScrollbarWidthOf::try_new(scalar(15.0)).unwrap();
                tree = tree.style(2, parent);
                assert_eq!(Traverse::children(&tree, 2).collect::<Vec<_>>(), [3, 4, 5]);
                assert_eq!(Traverse::children(&tree, 5).collect::<Vec<_>>(), [6]);
                assert_eq!(
                    tree.node_input(2).max_size.width,
                    MaxSizeOf::px(scalar(150.5))
                );
                assert_eq!(
                    tree.node_input(2).border,
                    Edges::all(LengthOf::px(scalar(2.0)))
                );
                assert_eq!(
                    tree.node_input(2).padding,
                    Edges::all(LengthOf::px(scalar(3.0)))
                );
                assert_eq!(
                    tree.node_input(4).size.width,
                    PreferredSizeOf::percent(scalar(0.5))
                );
                assert_eq!(
                    tree.node_input(5).size.width,
                    PreferredSizeOf::percent(scalar(0.5))
                );
                for node in 2..=6 {
                    assert!(!tree.has_leaf_measurement(node));
                }
                // Native quantitative ContentBox bounds add padding+border10;
                // forced gutters are reserved inside that border box. Thus the
                // two accepted outer widths are150.5/160.5 and actual content
                // widths subtract25, yielding125.5/135.5. The fixed200 child
                // exceeds both maxima; it cannot become the final basis.
                // Block content remains10+10+10=30, so outer height is55.
                assert_eq!(outer_width - 25.0, content_width);
                assert_eq!(content_width * 0.5, half);
                let batch = compute_layout(
                    &tree,
                    2,
                    request(WritingMode::HorizontalTb, SizingOperation::Normal),
                )
                .expect("real bounded ColumnFlex with actual reserved scrollbars");
                let entries = batch.unrounded_entries();
                let actual = output(entries, 2);
                assert_eq!(actual.location, Point::ZERO);
                assert_eq!(actual.size, Size::new(scalar(outer_width), scalar(55.0)));
                assert_eq!(actual.source_index, SourceIndex::ZERO);
                assert_eq!(actual.border, Edges::all(scalar(2.0)));
                assert_eq!(actual.padding, Edges::all(scalar(3.0)));
                let scroll = actual
                    .scroll_geometry
                    .expect("actual accepted scroll/content geometry");
                assert_eq!(
                    scroll.flow_axes(),
                    FlowAxes::new(WritingMode::HorizontalTb, direction)
                );
                assert_eq!(scroll.used_overflow_x(), Overflow::Scroll);
                assert_eq!(scroll.used_overflow_y(), Overflow::Scroll);
                assert_eq!(scroll.scrollbar_size(), Size::splat(scalar(15.0)));
                let content_x = if direction == Direction::Ltr {
                    5.0
                } else {
                    20.0
                };
                let scrollport_x = if direction == Direction::Ltr {
                    2.0
                } else {
                    17.0
                };
                assert_basis_rect(scroll.border_box(), [0.0, 0.0, outer_width, 55.0]);
                assert_basis_rect(scroll.padding_box(), [2.0, 2.0, outer_width - 4.0, 51.0]);
                assert_basis_rect(
                    scroll.scrollport(),
                    [scrollport_x, 2.0, outer_width - 19.0, 36.0],
                );
                assert_basis_rect(scroll.content_box(), [content_x, 5.0, content_width, 30.0]);
                assert_eq!(
                    actual.content_box_size(),
                    Size::new(scalar(content_width), scalar(30.0))
                );
                assert_eq!(scroll.gutters().top(), None);
                let (vertical, absent, vertical_x) = if direction == Direction::Ltr {
                    (
                        scroll.gutters().right(),
                        scroll.gutters().left(),
                        outer_width - 17.0,
                    )
                } else {
                    (scroll.gutters().left(), scroll.gutters().right(), 2.0)
                };
                assert_eq!(absent, None);
                assert_basis_rect(
                    vertical.expect("real inline-end15px reservation"),
                    [vertical_x, 2.0, 15.0, 36.0],
                );
                assert_basis_rect(
                    scroll
                        .gutters()
                        .bottom()
                        .expect("real bottom15px reservation"),
                    [scrollport_x, 38.0, outer_width - 19.0, 15.0],
                );
                let fixed_x = if direction == Direction::Ltr {
                    content_x
                } else {
                    content_x + content_width - 200.0
                };
                let percentage_x = if direction == Direction::Ltr {
                    content_x
                } else {
                    content_x + half
                };
                assert_box(entries, 3, [fixed_x, 5.0, 200.0, 10.0], 0);
                assert_box(entries, 4, [percentage_x, 15.0, half, 10.0], 1);
                assert_box(entries, 5, [percentage_x, 25.0, half, 10.0], 2);
                assert_box(
                    entries,
                    6,
                    [
                        if direction == Direction::Ltr {
                            0.0
                        } else {
                            half - 10.0
                        },
                        0.0,
                        10.0,
                        10.0,
                    ],
                    0,
                );
                // Exact independent publication table under native (x+.5).floor
                // corner policy. Normal percentage resolution precedes this;
                // canonical content126/136 must not replace raw125.5/135.5.
                let (
                    published_outer,
                    published_half,
                    published_fixed_x,
                    published_pct_x,
                    published_nested_x,
                ) = match (box_sizing, direction) {
                    (BoxSizing::BorderBox, Direction::Ltr) => (151.0, 63.0, 5.0, 5.0, 0.0),
                    (BoxSizing::BorderBox, Direction::Rtl) => (151.0, 63.0, -54.0, 83.0, 53.0),
                    (BoxSizing::ContentBox, Direction::Ltr) => (161.0, 68.0, 5.0, 5.0, 0.0),
                    (BoxSizing::ContentBox, Direction::Rtl) => (161.0, 68.0, -44.0, 88.0, 58.0),
                };
                let published = batch.final_entries();
                assert_eq!(output(published, 2).location, Point::ZERO);
                assert_eq!(
                    output(published, 2).size,
                    Size::new(scalar(published_outer), scalar(55.0))
                );
                assert_box(published, 3, [published_fixed_x, 5.0, 200.0, 10.0], 0);
                assert_box(
                    published,
                    4,
                    [published_pct_x, 15.0, published_half, 10.0],
                    1,
                );
                assert_box(
                    published,
                    5,
                    [published_pct_x, 25.0, published_half, 10.0],
                    2,
                );
                assert_box(published, 6, [published_nested_x, 0.0, 10.0, 10.0], 0);
            }
        }
    }
}

fn late_inline_overflow<S: LayoutScalar>() {
    let control = subtree::<S>(
        WritingMode::HorizontalTb,
        Direction::Ltr,
        BoxSizing::BorderBox,
        0.5,
        true,
        false,
    );
    let query = request(WritingMode::HorizontalTb, SizingOperation::Normal);
    let prior =
        compute_layout(&control, 2, query).expect("genuine definite-width completed control");
    assert_subtree(
        &prior,
        WritingMode::HorizontalTb,
        Direction::Ltr,
        0.5,
        false,
    );
    let retained_unrounded = prior.unrounded_entries().to_vec();
    let retained_final = prior.final_entries().to_vec();
    let largest = fri06_mr02_geometry_error_largest_finite::<S>();
    assert!(largest.is_finite());
    let expected_invalid = scalar::<S>(200.0) * largest;
    assert!(!expected_invalid.is_finite() && expected_invalid > S::ZERO);
    let mut automatic = control.node_input(2).clone();
    automatic.size.width = PreferredSizeOf::AUTO;
    let mut overflowing = control.node_input(4).clone();
    overflowing.size.width = PreferredSizeOf::percent(largest);
    let invalid = control.clone().style(2, automatic).style(4, overflowing);
    assert!(invalid.node_input(2).size.width.is_auto());
    assert_eq!(
        invalid.node_input(4).size.width,
        PreferredSizeOf::percent(largest)
    );
    assert!(!invalid.node_input(4).size.width.is_auto());
    for node in 2..=6 {
        assert!(!invalid.has_leaf_measurement(node));
    }
    // Finite authored coefficient is valid. It has no basis during formal
    // intrinsic completion, which must remain intrinsic rather than resolving
    // against either a provisional200 or the independent external600 bound.
    for operation in [SizingOperation::MinContent, SizingOperation::MaxContent] {
        let intrinsic = compute_layout(&invalid, 2, request(WritingMode::HorizontalTb, operation))
            .expect("cyclic inline percentage is initial during intrinsic contribution sizing");
        for entries in [intrinsic.unrounded_entries(), intrinsic.final_entries()] {
            assert_box(entries, 2, [0.0, 0.0, 200.0, 30.0], 0);
        }
    }
    // Normal contents layout alone receives the actual accepted200 basis.
    // 200*MAX is positive infinity in either scalar lane, owned by node4's
    // preferred-width resolution, not a root/frame/provider failure.
    let error = match compute_layout(&invalid, 2, query) {
        Err(error) => error,
        Ok(_) => panic!("late Normal percentage overflow must refuse the complete batch"),
    };
    assert_eq!(error.site(), LayoutErrorSiteOf::Node(4));
    assert_eq!(error.operation(), LayoutOperation::ValueResolution);
    match error.kind() {
        LayoutErrorKindOf::InvalidInput(LayoutInvalidInputOf::InvalidNumeric { value }) => {
            assert_eq!(*value, expected_invalid);
        }
        other => panic!("expected actual percentage arithmetic InvalidNumeric, got {other:?}"),
    }
    // Public computation is transactional: no partially successful batch is
    // returned; a previously completed immutable publication retains identity.
    assert_eq!(prior.unrounded_entries(), retained_unrounded.as_slice());
    assert_eq!(prior.final_entries(), retained_final.as_slice());
    assert_subtree(
        &prior,
        WritingMode::HorizontalTb,
        Direction::Ltr,
        0.5,
        false,
    );
    let recovered = invalid.style(4, control.node_input(4).clone());
    let recovery =
        compute_layout(&recovered, 2, query).expect("restored finite coefficient recovers");
    assert_subtree(
        &recovery,
        WritingMode::HorizontalTb,
        Direction::Ltr,
        0.5,
        false,
    );
    let replay =
        compute_layout(&control, 2, query).expect("unchanged real control inputs remain reusable");
    assert_subtree(
        &replay,
        WritingMode::HorizontalTb,
        Direction::Ltr,
        0.5,
        false,
    );
    assert_eq!(prior.unrounded_entries(), retained_unrounded.as_slice());
    assert_eq!(prior.final_entries(), retained_final.as_slice());
}

#[test]
fn accepted_fractional_content_basis_excludes_edges_gutters_and_maximum_f32() {
    fractional_content_basis::<f32>();
}
#[test]
fn accepted_fractional_content_basis_excludes_edges_gutters_and_maximum_f64() {
    fractional_content_basis::<f64>();
}
#[test]
fn late_normal_inline_percentage_overflow_refuses_and_recovers_f32() {
    late_inline_overflow::<f32>();
}
#[test]
fn late_normal_inline_percentage_overflow_refuses_and_recovers_f64() {
    late_inline_overflow::<f64>();
}
