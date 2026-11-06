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
    let prior = compute_layout(&control, 2, query).expect("definite-width completed control");
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

fn inline_ratio_finishes_auto_main<S: LayoutScalar>() {
    for direction in [Direction::Ltr, Direction::Rtl] {
        for box_sizing in [BoxSizing::BorderBox, BoxSizing::ContentBox] {
            for percentage_gap in [false, true] {
                for imposed_block in [false, true] {
                    // The independently authored200-inline control precedes the
                    // auto-inline case. Only the accepted INLINE size is fixed;
                    // an auto main size must include the final ratio-based50.
                    for definite_inline in [true, false] {
                        let mut parent = native_box::<S>(
                            WritingMode::HorizontalTb,
                            direction,
                            box_sizing,
                            if definite_inline {
                                PreferredSizeOf::px(scalar(200.0))
                            } else {
                                PreferredSizeOf::AUTO
                            },
                            if imposed_block {
                                PreferredSizeOf::px(scalar(100.0))
                            } else {
                                PreferredSizeOf::AUTO
                            },
                        );
                        parent.flex_direction = FlexDirection::Column;
                        if percentage_gap {
                            parent.gap.height = LengthOf::percent(scalar(0.1));
                            assert_eq!(scalar::<S>(0.1) * scalar(100.0), scalar(10.0));
                        }
                        let mut ratio = native_box(
                            WritingMode::HorizontalTb,
                            direction,
                            box_sizing,
                            PreferredSizeOf::percent(scalar(0.5)),
                            PreferredSizeOf::AUTO,
                        );
                        ratio.aspect_ratio = AspectRatioOf::new(scalar(2.0));
                        let tree = PublicLayoutTreeOf::new()
                            .children(2, [3, 4])
                            .children(3, [])
                            .children(4, [])
                            .style(2, parent)
                            .style(
                                3,
                                native_box(
                                    WritingMode::HorizontalTb,
                                    direction,
                                    box_sizing,
                                    PreferredSizeOf::px(scalar(200.0)),
                                    PreferredSizeOf::px(scalar(10.0)),
                                ),
                            )
                            .style(4, ratio);
                        assert_eq!(Traverse::children(&tree, 2).collect::<Vec<_>>(), [3, 4]);
                        assert_eq!(
                            tree.node_input(4).size.width,
                            PreferredSizeOf::percent(scalar(0.5))
                        );
                        assert!(tree.node_input(4).size.height.is_auto());
                        assert_eq!(
                            tree.node_input(4).aspect_ratio,
                            AspectRatioOf::new(scalar(2.0))
                        );
                        for node in [2, 3, 4] {
                            assert!(!tree.has_leaf_measurement(node));
                        }
                        let batch = compute_layout(
                            &tree,
                            2,
                            request(WritingMode::HorizontalTb, SizingOperation::Normal),
                        )
                        .expect("real Normal percentage/ratio Flex fixture");
                        // Fixed sibling pins inline200. Resolved child inline100
                        // transfers through ratio2 to block50;10+50=60. The
                        // existing definite100 block allocation remains100.
                        // A cyclic auto-block percentage gap remains0; an
                        // initially definite100 basis resolves the same10% to10.
                        let gap = if percentage_gap && imposed_block {
                            10.0
                        } else {
                            0.0
                        };
                        for entries in [batch.unrounded_entries(), batch.final_entries()] {
                            assert_box(
                                entries,
                                2,
                                [0.0, 0.0, 200.0, if imposed_block { 100.0 } else { 60.0 }],
                                0,
                            );
                            assert_box(entries, 3, [0.0, 0.0, 200.0, 10.0], 0);
                            assert_box(
                                entries,
                                4,
                                [
                                    if direction == Direction::Ltr {
                                        0.0
                                    } else {
                                        100.0
                                    },
                                    10.0 + gap,
                                    100.0,
                                    50.0,
                                ],
                                1,
                            );
                        }
                    }
                }
            }
        }
    }
}

fn captured_normal_wrapping_finishes_auto_main<S: LayoutScalar>() {
    use crate::test_support::text::{PreparedParagraph, ahem_style};
    use surgeist_text as text;

    let mut style = ahem_style(10.0, 10.0);
    style.wrap = text::Wrap::Word;
    let paragraph = PreparedParagraph::plain(7, text::Source::new("XXXXX XXXXX"), style);
    let origin = paragraph.origin(7);
    let origins = paragraph.prepared.origin_views().collect::<Vec<_>>();
    assert_eq!(origins.len(), 1);
    assert_eq!(origins[0].handle(), origin);
    assert_eq!(
        origins[0].source().text().code_units(),
        &[88, 88, 88, 88, 88, 32, 88, 88, 88, 88, 88]
    );
    assert_eq!(origins[0].local_range(), text::Range::new(0, 11));
    assert_eq!(origins[0].aggregate_range(), text::Range::new(0, 11));
    assert_eq!(
        paragraph
            .prepared
            .unit_views()
            .map(|unit| unit.advance())
            .sum::<f64>(),
        110.0
    );
    for unit in paragraph
        .prepared
        .unit_views()
        .filter(|unit| !unit.is_whitespace())
    {
        assert_eq!(unit.ascent(), 8.0);
        assert_eq!(unit.descent(), 2.0);
    }
    for definite_inline in [true, false] {
        let mut parent = native_box::<S>(
            WritingMode::HorizontalTb,
            Direction::Ltr,
            BoxSizing::BorderBox,
            if definite_inline {
                PreferredSizeOf::px(scalar(200.0))
            } else {
                PreferredSizeOf::AUTO
            },
            PreferredSizeOf::AUTO,
        );
        parent.flex_direction = FlexDirection::Column;
        // A Block owns this actual prepared paragraph. No Text source
        // is assigned to the fixed native sibling or the outer Flex formatter.
        let tree = PublicLayoutTreeOf::new()
            .children(2, [3, 4])
            .children(3, [])
            .children(4, [7])
            .children(7, [])
            .style(2, parent)
            .style(
                3,
                native_box(
                    WritingMode::HorizontalTb,
                    Direction::Ltr,
                    BoxSizing::BorderBox,
                    PreferredSizeOf::px(scalar(200.0)),
                    PreferredSizeOf::px(scalar(10.0)),
                ),
            )
            .style(
                4,
                NodeInputOf {
                    display: Display::Block,
                    size: Size::new(PreferredSizeOf::percent(scalar(0.5)), PreferredSizeOf::AUTO),
                    ..NodeInputOf::default()
                },
            )
            .input(7, paragraph.text_input(7));
        for node in [2, 3, 4, 7] {
            assert!(!tree.has_leaf_measurement(node));
        }
        let binding = paragraph.binding(4);
        assert_eq!(binding.container, 4);
        assert_eq!(binding.nodes, [7]);
        assert_eq!(binding.origins.len(), 1);
        assert_eq!(binding.origins[0].origin, origin);
        let capture = LayoutFlowInput::try_new(
            &tree,
            2,
            vec![binding],
            WrappingInput {
                native_inline_boxes: Vec::new(),
                gaps: Vec::new(),
                scopes: Vec::new(),
                child_order: Vec::new(),
            },
            Vec::new(),
        )
        .expect("actual complete paragraph belongs to Block4");
        let batch = compute_layout_with_flow(
            &capture,
            2,
            request(WritingMode::HorizontalTb, SizingOperation::Normal),
        )
        .expect("registered Ahem source in real Normal ColumnFlex layout");
        // Intrinsic one-line source width110 cannot enlarge the fixed200 inline
        // demand. Used50%=100 admits one50px word per line: two10px lines.
        // Therefore the auto main extent is fixed sibling10 + Text20 =30.
        for entries in [batch.unrounded_entries(), batch.final_entries()] {
            assert_box(entries, 2, [0.0, 0.0, 200.0, 30.0], 0);
            assert_box(entries, 3, [0.0, 0.0, 200.0, 10.0], 0);
            assert_box(entries, 4, [0.0, 10.0, 100.0, 20.0], 1);
            assert_eq!(output(entries, 7).source_index, SourceIndex::ZERO);
        }
        let placed = batch.placed_text();
        assert_eq!(placed.len(), 1);
        assert_eq!(placed[0].container(), 4);
        assert_eq!(placed[0].nodes(), &[7]);
        let layout = placed[0].layout();
        assert_eq!(
            layout.source().text().code_units(),
            &[88, 88, 88, 88, 88, 32, 88, 88, 88, 88, 88]
        );
        let lines = layout.lines();
        assert_eq!(lines.len(), 2);
        for (index, (range, top, baseline)) in [
            (text::Range::new(0, 6), 10.0, 18.0),
            (text::Range::new(6, 11), 20.0, 28.0),
        ]
        .into_iter()
        .enumerate()
        {
            assert_eq!(lines[index].range(), range);
            assert_eq!(lines[index].bounds().origin.y, top);
            assert_eq!(lines[index].bounds().size.height, 10.0);
            assert_eq!(lines[index].baseline(), baseline);
        }
        for entry in batch.final_inline_fragments() {
            let fragment = entry.fragment().finalized_fragment();
            assert_eq!(
                fragment.view().owner(),
                text::FinalizedFragmentOwner::Origin(origin.clone())
            );
            assert_eq!(fragment.finalized_line().feedback().line_height, 10.0);
            assert_eq!(
                fragment
                    .finalized_line()
                    .selected_line()
                    .request()
                    .context
                    .available_inline,
                Some(100.0)
            );
        }
        // Real original X cluster identities determine these rectangles, rather
        // than an emitted-run ordinal or another publication's measured output.
        let clusters = layout.clusters();
        let visible = clusters
            .iter()
            .filter(|cluster| cluster.range().start != 5)
            .collect::<Vec<_>>();
        assert_eq!(visible.len(), 10);
        for cluster in visible {
            let at = cluster.range().start;
            assert_eq!(cluster.range(), text::Range::new(at, at + 1));
            let (column, top) = if at < 5 { (at, 10.0) } else { (at - 6, 20.0) };
            assert_eq!(
                cluster.bounds(),
                text::Rect::new(column as f64 * 10.0, top, 10.0, 10.0)
            );
        }
    }
}

#[test]
fn resolved_inline_ratio_finishes_auto_main_but_preserves_definite_main_f32() {
    inline_ratio_finishes_auto_main::<f32>();
}
#[test]
fn resolved_inline_ratio_finishes_auto_main_but_preserves_definite_main_f64() {
    inline_ratio_finishes_auto_main::<f64>();
}
#[test]
fn actual_normal_text_wrap_finishes_auto_main_after_inline_percentage_f32() {
    captured_normal_wrapping_finishes_auto_main::<f32>();
}
#[test]
fn actual_normal_text_wrap_finishes_auto_main_after_inline_percentage_f64() {
    captured_normal_wrapping_finishes_auto_main::<f64>();
}

fn cyclic_main_gap<S: LayoutScalar>(fixed: f64) {
    for direction in [Direction::Ltr, Direction::Rtl] {
        for imposed in [true, false] {
            for operation in [
                SizingOperation::Normal,
                SizingOperation::MinContent,
                SizingOperation::MaxContent,
            ] {
                let mut parent = native_box::<S>(
                    WritingMode::HorizontalTb,
                    direction,
                    BoxSizing::BorderBox,
                    PreferredSizeOf::px(scalar(100.0)),
                    if imposed {
                        PreferredSizeOf::px(scalar(80.0))
                    } else {
                        PreferredSizeOf::AUTO
                    },
                );
                parent.flex_direction = FlexDirection::Column;
                parent.gap.height = LengthOf::value(
                    LengthPercentageOf::from_coefficients(scalar(fixed), scalar(0.125)).unwrap(),
                );
                let tree = PublicLayoutTreeOf::new()
                    .children(2, [3, 4])
                    .children(3, [])
                    .children(4, [])
                    .style(2, parent)
                    .style(
                        3,
                        native_box(
                            WritingMode::HorizontalTb,
                            direction,
                            BoxSizing::BorderBox,
                            PreferredSizeOf::px(scalar(20.0)),
                            PreferredSizeOf::px(scalar(10.0)),
                        ),
                    )
                    .style(
                        4,
                        native_box(
                            WritingMode::HorizontalTb,
                            direction,
                            BoxSizing::BorderBox,
                            PreferredSizeOf::px(scalar(20.0)),
                            PreferredSizeOf::px(scalar(10.0)),
                        ),
                    );
                let normal = request::<S>(WritingMode::HorizontalTb, SizingOperation::Normal);
                let mut axes = normal.sizing_request().axes();
                axes.height = AxisSizingRequestOf::new(
                    if operation == SizingOperation::Normal {
                        AvailableSpaceOf::Indefinite
                    } else {
                        AvailableSpaceOf::Definite(NonNegativeFiniteOf::new(scalar(600.0)).unwrap())
                    },
                    operation,
                );
                let query = LayoutRootRequestOf::standalone(
                    normal.sizing_request().with_axes(axes),
                    FlowAxes::new(WritingMode::HorizontalTb, Direction::Ltr),
                    Size::NONE,
                );
                let batch = compute_layout(&tree, 2, query).expect("real Flex gap basis qualifier");
                // Align3 §8.3: cyclic Flex gap percentages resolve against ZERO
                // in every phase. A calc retains its fixed term. An authored
                // definite80 block size instead supplies80 * 1/8 =10.
                let gap = fixed + if imposed { 10.0 } else { 0.0 };
                let height = if imposed { 80.0 } else { 20.0 + fixed };
                let x = if direction == Direction::Ltr {
                    0.0
                } else {
                    80.0
                };
                for entries in [batch.unrounded_entries(), batch.final_entries()] {
                    assert_box(entries, 2, [0.0, 0.0, 100.0, height], 0);
                    assert_box(entries, 3, [x, 0.0, 20.0, 10.0], 0);
                    assert_box(entries, 4, [x, 10.0 + gap, 20.0, 10.0], 1);
                }
            }
        }
    }
}

#[test]
fn cyclic_main_gap_uses_zero_percentage_basis_f32() {
    cyclic_main_gap::<f32>(0.0);
}
#[test]
fn cyclic_main_gap_uses_zero_percentage_basis_f64() {
    cyclic_main_gap::<f64>(0.0);
}
#[test]
fn cyclic_main_calc_gap_retains_fixed_term_f32() {
    cyclic_main_gap::<f32>(5.0);
}
#[test]
fn cyclic_main_calc_gap_retains_fixed_term_f64() {
    cyclic_main_gap::<f64>(5.0);
}

fn normal_compute_size_finishes_inline_ratio_main<S: LayoutScalar>() {
    for definite_inline in [true, false] {
        let mut parent = native_box::<S>(
            WritingMode::HorizontalTb,
            Direction::Ltr,
            BoxSizing::BorderBox,
            if definite_inline {
                PreferredSizeOf::px(scalar(200.0))
            } else {
                PreferredSizeOf::AUTO
            },
            PreferredSizeOf::AUTO,
        );
        parent.flex_direction = FlexDirection::Column;
        let fixed = native_box(
            WritingMode::HorizontalTb,
            Direction::Ltr,
            BoxSizing::BorderBox,
            PreferredSizeOf::px(scalar(200.0)),
            PreferredSizeOf::px(scalar(10.0)),
        );
        let mut ratio = native_box(
            WritingMode::HorizontalTb,
            Direction::Ltr,
            BoxSizing::BorderBox,
            PreferredSizeOf::percent(scalar(0.5)),
            PreferredSizeOf::AUTO,
        );
        ratio.aspect_ratio = AspectRatioOf::new(scalar(2.0));
        // No measurements or staged outputs are supplied to this harness. Its
        // actual child dispatcher invokes production compute_flex/compute_block.
        let mut tree = OracleTreeOf::<S>::new()
            .children(2, [3, 4])
            .children(3, [])
            .children(4, [])
            .style(2, parent)
            .style(3, fixed)
            .style(4, ratio);
        assert!(tree.node_input(2).size.height.is_auto());
        if !definite_inline {
            assert!(tree.node_input(2).size.width.is_auto());
        }
        assert_eq!(
            tree.node_input(4).size.width,
            PreferredSizeOf::percent(scalar(0.5))
        );
        assert!(tree.node_input(4).size.height.is_auto());
        assert_eq!(
            tree.node_input(4).aspect_ratio,
            AspectRatioOf::new(scalar(2.0))
        );
        let sizing_request = SizingRequestOf::new(
            Size::new(
                AxisSizingRequestOf::new(
                    AvailableSpaceOf::Definite(NonNegativeFiniteOf::new(scalar(300.0)).unwrap()),
                    SizingOperation::Normal,
                ),
                AxisSizingRequestOf::new(AvailableSpaceOf::Indefinite, SizingOperation::Normal),
            ),
            OrthogonalFallbackOf::new(
                Size::new(
                    Some(NonNegativeFiniteOf::new(scalar(600.0)).unwrap()),
                    Some(NonNegativeFiniteOf::new(scalar(400.0)).unwrap()),
                ),
                NearestScrollportOf::Absent,
                Size::splat(OrthogonalFallbackLimitOf::Unbounded),
            ),
        );
        let input = ComputeInputOf::for_child(
            RunMode::ComputeSize,
            SizingMode::InherentSize,
            RequestedAxis::Both,
            Size::NONE,
            Size::NONE,
            ContainingLayoutContext::new(
                FlowAxes::new(WritingMode::HorizontalTb, Direction::Ltr),
                ParentFormattingContext::NoParent,
            ),
            sizing_request,
        );
        assert_eq!(input.run_mode(), RunMode::ComputeSize);
        assert_eq!(input.known(), Size::NONE);
        assert_eq!(input.parent(), Size::NONE);
        assert_eq!(input.sizing_request().axes(), sizing_request.axes());
        assert_eq!(
            input.sizing_request().axes().width.operation(),
            SizingOperation::Normal
        );
        assert_eq!(
            input.sizing_request().axes().height.operation(),
            SizingOperation::Normal
        );
        let measured = {
            let mut owner = tree.borrow_compute();
            for node in [2, 3, 4] {
                assert!(!owner.has_leaf_measurement(node));
            }
            owner
                .compute_child(2, input)
                .expect("real Normal ComputeSize Flex dispatch")
        };
        // Finite300 space is neither a known size nor an allocation. The fixed
        // sibling determines inline200;50% then resolves100 and ratio2 transfers
        // to block50. Normal size demand therefore includes10+50=60 even though
        // it publishes no geometry. This is not an intrinsic MIN/MAX operation.
        assert_eq!(measured.size, Size::new(scalar(200.0), scalar(60.0)));
        assert_eq!(measured.scroll_geometry, None);
        assert_eq!(tree.inputs(2).first(), Some(&input));
        for node in [2, 3, 4] {
            assert_eq!(
                tree.layout(node),
                None,
                "ComputeSize must not stage node{node} geometry"
            );
            assert_eq!(
                tree.final_layout(node),
                None,
                "ComputeSize must not finalize node{node} geometry"
            );
        }
        // A separate real PerformLayout on the unchanged declared tree agrees
        // on Normal sizing; changing run mode does not select another operation.
        let performed_input = input.with_run_mode(RunMode::PerformLayout);
        assert_eq!(performed_input.sizing_request(), input.sizing_request());
        let performed = tree
            .borrow_compute()
            .compute_child(2, performed_input)
            .expect("same source production Normal PerformLayout");
        assert_eq!(performed.size, Size::new(scalar(200.0), scalar(60.0)));
        assert!(performed.scroll_geometry.is_some());
    }
}

#[test]
fn normal_compute_size_with_unknown_dimensions_finishes_inline_ratio_main_f32() {
    normal_compute_size_finishes_inline_ratio_main::<f32>();
}
#[test]
fn normal_compute_size_with_unknown_dimensions_finishes_inline_ratio_main_f64() {
    normal_compute_size_finishes_inline_ratio_main::<f64>();
}

fn content_basis_uses_cross_margin_fit_content_space<S: LayoutScalar>() {
    use crate::test_support::text::{PreparedParagraph, ahem_style};
    use surgeist_text as text;

    let mut style = ahem_style(10.0, 10.0);
    style.wrap = text::Wrap::Word;
    let paragraph = PreparedParagraph::plain(4, text::Source::new("XXXX XXXX"), style);
    for (margin_right, expected_height, expected_lines) in [(0.0, 10.0, 1), (21.0, 20.0, 2)] {
        let tree = PublicLayoutTreeOf::new()
            .children(1, [2])
            .children(2, [4])
            .children(4, [])
            .style(
                1,
                NodeInputOf::<S> {
                    display: Display::Flex,
                    flex_direction: FlexDirection::Column,
                    size: Size::new(
                        PreferredSizeOf::px(scalar(100.0)),
                        PreferredSizeOf::px(scalar(100.0)),
                    ),
                    ..NodeInputOf::default()
                },
            )
            .style(
                2,
                NodeInputOf {
                    display: Display::Block,
                    align_self: Some(AlignItems::FlexStart),
                    flex_basis: FlexBasisOf::CONTENT,
                    flex_shrink: FlexShrinkOf::try_new(S::ZERO).unwrap(),
                    min_size: Size::new(MinSizeOf::ZERO, MinSizeOf::ZERO),
                    margin: Edges {
                        right: LengthAutoOf::px(scalar(margin_right)),
                        ..Edges::all(LengthAutoOf::ZERO)
                    },
                    ..NodeInputOf::default()
                },
            )
            .input(4, paragraph.text_input(4));
        let capture = LayoutFlowInput::try_new(
            &tree,
            1,
            vec![paragraph.binding(2)],
            WrappingInput {
                native_inline_boxes: Vec::new(),
                gaps: Vec::new(),
                scopes: Vec::new(),
                child_order: Vec::new(),
            },
            Vec::new(),
        )
        .unwrap();
        let batch = compute_layout_with_flow(
            &capture,
            1,
            request(WritingMode::HorizontalTb, SizingOperation::Normal),
        )
        .unwrap();

        // Four 10px glyphs, one 10px space, then four more glyphs fit in
        // 100px. A 21px margin leaves only 79px and splits the two words.
        assert_eq!(
            output(batch.unrounded_entries(), 2).size.height,
            scalar(expected_height)
        );
        assert_eq!(
            batch.placed_text()[0].layout().lines().len(),
            expected_lines
        );
    }
}

#[test]
fn flex_content_basis_uses_cross_margin_fit_content_space() {
    content_basis_uses_cross_margin_fit_content_space::<f32>();
    content_basis_uses_cross_margin_fit_content_space::<f64>();
}

fn assert_relative_main_percentage_uses_accepted_extent<S: LayoutScalar>() {
    let px = S::from_f64;
    let mode = WritingMode::HorizontalTb;
    let child = NodeInputOf {
        display: Display::Block,
        size: Size::new(PreferredSizeOf::px(px(20.0)), PreferredSizeOf::px(px(10.0))),
        min_size: Size::new(MinSizeOf::ZERO, MinSizeOf::ZERO),
        flex_shrink: FlexShrinkOf::try_new(S::ZERO).unwrap(),
        ..NodeInputOf::default()
    };
    let mut shifted = child.clone();
    shifted.inset.left = LengthAutoOf::percent(px(0.1));
    shifted.inset.top = LengthAutoOf::percent(px(0.25));
    shifted.inset.bottom = LengthAutoOf::px(px(3.0));
    for height in [PreferredSizeOf::px(px(20.0)), PreferredSizeOf::AUTO] {
        let mut root = native_box(
            mode,
            Direction::Ltr,
            BoxSizing::ContentBox,
            PreferredSizeOf::px(px(100.0)),
            height,
        );
        root.flex_direction = FlexDirection::Column;
        root.align_items = Some(AlignItems::FlexStart);
        root.padding = Edges::new(
            LengthOf::px(px(4.0)),
            LengthOf::px(px(10.0)),
            LengthOf::px(px(6.0)),
            LengthOf::px(px(10.0)),
        );
        let tree = PublicLayoutTreeOf::new()
            .children(0, [1, 2])
            .children(1, [])
            .children(2, [])
            .style(0, root)
            .style(1, shifted.clone())
            .style(2, child.clone());
        let batch = compute_layout(&tree, 0, request(mode, SizingOperation::Normal)).unwrap();
        let entry = |node| {
            batch
                .unrounded_entries()
                .iter()
                .find(|entry| entry.node() == node)
                .unwrap()
        };
        assert_eq!(entry(0).output().size, Size::new(px(120.0), px(30.0)));
        let crate::output::GeometryStateOf::OrdinaryUnrounded { measured_normal } =
            entry(1).geometry_state()
        else {
            panic!("the first item has ordinary unrounded geometry");
        };
        assert_eq!(measured_normal.location, Point::new(px(10.0), px(4.0)));
        assert_eq!(entry(1).output().location, Point::new(px(20.0), px(9.0)));
        assert_eq!(entry(2).output().location, Point::new(px(10.0), px(14.0)));
    }
}

#[test]
fn relative_main_percentage_uses_accepted_extent() {
    assert_relative_main_percentage_uses_accepted_extent::<f32>();
    assert_relative_main_percentage_uses_accepted_extent::<f64>();
}

fn assert_vertical_relative_main_percentage_uses_physical_width<S: LayoutScalar>() {
    let px = S::from_f64;
    let mode = WritingMode::VerticalRl;
    let mut root = native_box(
        mode,
        Direction::Ltr,
        BoxSizing::ContentBox,
        PreferredSizeOf::px(px(100.0)),
        PreferredSizeOf::AUTO,
    );
    root.flex_direction = FlexDirection::Column;
    root.align_items = Some(AlignItems::FlexStart);
    let child = NodeInputOf {
        display: Display::Block,
        writing_mode: mode,
        size: Size::new(PreferredSizeOf::px(px(10.0)), PreferredSizeOf::px(px(20.0))),
        min_size: Size::new(MinSizeOf::ZERO, MinSizeOf::ZERO),
        flex_shrink: FlexShrinkOf::try_new(S::ZERO).unwrap(),
        ..NodeInputOf::default()
    };
    let mut shifted = child.clone();
    shifted.inset.right = LengthAutoOf::percent(px(0.25));
    shifted.inset.left = LengthAutoOf::px(px(3.0));
    let tree = PublicLayoutTreeOf::new()
        .children(0, [1, 2])
        .children(1, [])
        .children(2, [])
        .style(0, root)
        .style(1, shifted)
        .style(2, child);
    let batch = compute_layout(&tree, 0, request(mode, SizingOperation::Normal)).unwrap();
    let entry = |node| {
        batch
            .unrounded_entries()
            .iter()
            .find(|entry| entry.node() == node)
            .unwrap()
    };
    assert_eq!(entry(0).output().size, Size::new(px(20.0), px(100.0)));
    let crate::output::GeometryStateOf::OrdinaryUnrounded { measured_normal } =
        entry(1).geometry_state()
    else {
        panic!("the first item has ordinary unrounded geometry");
    };
    assert_eq!(measured_normal.location, Point::new(px(10.0), S::ZERO));
    assert_eq!(entry(1).output().location, Point::new(px(5.0), S::ZERO));
    assert_eq!(entry(2).output().location, Point::ZERO);
}

#[test]
fn vertical_relative_main_percentage_uses_physical_width() {
    assert_vertical_relative_main_percentage_uses_physical_width::<f32>();
    assert_vertical_relative_main_percentage_uses_physical_width::<f64>();
}

fn relative_cross_range_batch<S: LayoutScalar>(
    quarter: S,
    end_aligned: bool,
) -> LayoutResultOf<u32, CompletedLayoutBatchOf<u32, S>, S> {
    let px = S::from_f64;
    let mode = WritingMode::HorizontalTb;
    let mut root = native_box(
        mode,
        Direction::Ltr,
        BoxSizing::ContentBox,
        PreferredSizeOf::px(px(40.0)),
        PreferredSizeOf::AUTO,
    );
    root.flex_direction = FlexDirection::Row;
    root.flex_wrap = FlexWrap::WrapReverse;
    let child = NodeInputOf {
        display: Display::Block,
        size: Size::new(PreferredSizeOf::px(px(10.0)), PreferredSizeOf::px(S::ZERO)),
        ..NodeInputOf::default()
    };
    let mut tall = child.clone();
    tall.size.height = PreferredSizeOf::px(px(3.0) * quarter);
    let mut shifted = child;
    if end_aligned {
        shifted.align_self = Some(AlignItems::FlexEnd);
        shifted.inset.bottom = LengthAutoOf::percent(px(0.5));
    } else {
        shifted.align_self = Some(AlignItems::FlexStart);
        shifted.inset.top = LengthAutoOf::percent(px(0.5));
    }
    let tree = PublicLayoutTreeOf::new()
        .children(0, [1, 2])
        .children(1, [])
        .children(2, [])
        .style(0, root)
        .style(1, shifted)
        .style(2, tall);
    compute_layout(&tree, 0, request(mode, SizingOperation::Normal))
}

fn assert_relative_cross_projection_keeps_representable_range<S: LayoutScalar>(quarter: S) {
    let px = S::from_f64;
    let batch = relative_cross_range_batch(quarter, true)
        .expect("the reversed cross coordinate fits after projection");
    let entry = batch
        .unrounded_entries()
        .iter()
        .find(|entry| entry.node() == 1)
        .unwrap();
    let crate::output::GeometryStateOf::OrdinaryUnrounded { measured_normal } =
        entry.geometry_state()
    else {
        panic!("the item has ordinary unrounded geometry");
    };
    assert_eq!(measured_normal.location.y, S::ZERO);
    assert_eq!(entry.output().location.y, -px(1.5) * quarter);
    assert_eq!(entry.output().size.height, S::ZERO);
    let error = relative_cross_range_batch(quarter, false).unwrap_err();
    assert_eq!(error.site(), LayoutErrorSiteOf::Node(1));
    assert!(matches!(
        error.kind(),
        LayoutErrorKindOf::InvalidInput(LayoutInvalidInputOf::InvalidNumeric { value })
            if *value == S::INFINITY
    ));
}

#[test]
fn relative_cross_projection_keeps_representable_range() {
    assert_relative_cross_projection_keeps_representable_range(2.0_f32.powi(126));
    assert_relative_cross_projection_keeps_representable_range(2.0_f64.powi(1022));
}
