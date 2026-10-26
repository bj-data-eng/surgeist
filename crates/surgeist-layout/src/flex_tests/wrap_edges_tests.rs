//! Real flex collection from declared edges. Literal geometry follows pinned
//! Text4 §5.3 and Flexbox §6/§9.3/§9.4, not the collector's current output.

use super::*;

fn fixed<S: LayoutScalar>(width: f64, height: f64) -> NodeInputOf<S> {
    NodeInputOf {
        display: Display::Block,
        size: Size::new(
            PreferredSizeOf::value(LengthPercentageOf::px(S::from_f64(width)).unwrap()),
            PreferredSizeOf::value(LengthPercentageOf::px(S::from_f64(height)).unwrap()),
        ),
        flex_grow: FlexGrowOf::ZERO,
        flex_shrink: FlexShrinkOf::try_new(S::ZERO).unwrap(),
        ..NodeInputOf::default()
    }
}

fn cells<S: LayoutScalar>() -> [NodeInputOf<S>; 3] {
    std::array::from_fn(|_| fixed(10.0, 10.0))
}

fn run<S: LayoutScalar>(
    items: [NodeInputOf<S>; 3],
    width: Option<f64>,
    available: AvailableOf<S>,
    reverse: bool,
    wrap: FlexWrap,
    main_gap: f64,
    row_gap: f64,
) -> CompletedLayoutBatchOf<u32, S> {
    let mut tree = PublicLayoutTreeOf::new().children(0, [1, 2, 3]).style(
        0,
        NodeInputOf {
            display: Display::Flex,
            flex_direction: if reverse {
                FlexDirection::RowReverse
            } else {
                FlexDirection::Row
            },
            flex_wrap: wrap,
            align_items: Some(AlignItems::FlexStart),
            align_content: Some(AlignContent::FlexStart),
            justify_content: Some(AlignContent::FlexStart),
            size: Size::new(
                width.map_or(PreferredSizeOf::AUTO, |width| {
                    PreferredSizeOf::value(LengthPercentageOf::px(S::from_f64(width)).unwrap())
                }),
                PreferredSizeOf::AUTO,
            ),
            gap: Size::new(
                LengthOf::px(S::from_f64(main_gap)),
                LengthOf::px(S::from_f64(row_gap)),
            ),
            ..NodeInputOf::default()
        },
    );
    for (node, item) in (1..=3).zip(items) {
        tree = tree.children(node, []).style(node, item);
    }
    compute_layout(
        &tree,
        0,
        LayoutRootRequestOf::viewport(Size::new(available, AvailableOf::MAX_CONTENT)).unwrap(),
    )
    .unwrap()
}

fn output<S: LayoutScalar>(batch: &CompletedLayoutBatchOf<u32, S>, node: u32) -> NodeOutputOf<S> {
    batch
        .final_entries()
        .iter()
        .find(|entry| entry.node() == node)
        .unwrap()
        .output()
}

fn assert_size<S: LayoutScalar>(batch: &CompletedLayoutBatchOf<u32, S>, width: f64, height: f64) {
    assert_eq!(
        output(batch, 0).size,
        Size::new(S::from_f64(width), S::from_f64(height))
    );
}

fn assert_item<S: LayoutScalar>(batch: &CompletedLayoutBatchOf<u32, S>, node: u32, rect: [f64; 4]) {
    let item = output(batch, node);
    assert_eq!(
        item.location,
        Point::new(S::from_f64(rect[0]), S::from_f64(rect[1]))
    );
    assert_eq!(
        item.size,
        Size::new(S::from_f64(rect[2]), S::from_f64(rect[3]))
    );
    assert_eq!(
        item.source_index,
        SourceIndex::new(usize::try_from(node - 1).unwrap())
    );
}

fn assert_cells<S: LayoutScalar>(batch: &CompletedLayoutBatchOf<u32, S>, positions: [[f64; 2]; 3]) {
    for (node, [x, y]) in (1..=3).zip(positions) {
        assert_item(batch, node, [x, y, 10.0, 10.0]);
    }
}

#[test]
fn flex_edge_values_rank_real_fitting_groups_in_both_scalar_lanes_and_main_directions() {
    fn lane<S: LayoutScalar>() {
        for reverse in [false, true] {
            for before in [false, true] {
                for edge in [
                    WrapEdge::Auto,
                    WrapEdge::Avoid,
                    WrapEdge::AvoidLine,
                    WrapEdge::AvoidFlex,
                    WrapEdge::Line,
                    WrapEdge::Flex,
                ] {
                    let mut items = cells::<S>();
                    // Flexbox §4 blockifies these native layout-ready items.
                    // The `line` keyword does not apply to their flex boundary.
                    if before {
                        items[2].display = Display::Block;
                        items[2].wrap_edges.before = edge;
                    } else {
                        items[1].display = Display::Block;
                        items[1].wrap_edges.after = edge;
                    }
                    let batch = run(
                        items,
                        Some(25.0),
                        AvailableOf::definite(S::from_f64(25.0)),
                        reverse,
                        FlexWrap::Wrap,
                        0.0,
                        0.0,
                    );
                    assert_size(&batch, 25.0, 20.0);
                    let avoid = matches!(edge, WrapEdge::Avoid | WrapEdge::AvoidFlex);
                    let positions = match (reverse, avoid) {
                        (false, false) => [[0.0, 0.0], [10.0, 0.0], [0.0, 10.0]],
                        (false, true) => [[0.0, 0.0], [0.0, 10.0], [10.0, 10.0]],
                        (true, false) => [[15.0, 0.0], [5.0, 0.0], [15.0, 10.0]],
                        (true, true) => [[15.0, 0.0], [15.0, 10.0], [5.0, 10.0]],
                    };
                    assert_cells(&batch, positions);
                    if avoid {
                        let mut items = cells::<S>();
                        if before {
                            items[2].wrap_edges.before = edge;
                        } else {
                            items[1].wrap_edges.after = edge;
                        }
                        let necessary = run(
                            items,
                            Some(15.0),
                            AvailableOf::definite(S::from_f64(15.0)),
                            reverse,
                            FlexWrap::Wrap,
                            0.0,
                            0.0,
                        );
                        assert_size(&necessary, 15.0, 30.0);
                        assert_cells(
                            &necessary,
                            if reverse {
                                [[5.0, 0.0], [5.0, 10.0], [5.0, 20.0]]
                            } else {
                                [[0.0, 0.0], [0.0, 10.0], [0.0, 20.0]]
                            },
                        );
                    }
                }
            }
        }
    }
    lane::<f32>();
    lane::<f64>();
}

#[test]
fn flex_force_wins_at_shared_edges_but_nowrap_keeps_one_overflowing_group() {
    fn lane<S: LayoutScalar>() {
        for reverse in [false, true] {
            for force_before in [false, true] {
                let mut items = cells::<S>();
                items[0].wrap_edges.after = if force_before {
                    WrapEdge::Avoid
                } else {
                    WrapEdge::Flex
                };
                items[1].wrap_edges.before = if force_before {
                    WrapEdge::Flex
                } else {
                    WrapEdge::AvoidFlex
                };
                let batch = run(
                    items.clone(),
                    Some(100.0),
                    AvailableOf::definite(S::from_f64(100.0)),
                    reverse,
                    FlexWrap::Wrap,
                    0.0,
                    0.0,
                );
                assert_size(&batch, 100.0, 20.0);
                assert_cells(
                    &batch,
                    if reverse {
                        [[90.0, 0.0], [90.0, 10.0], [80.0, 10.0]]
                    } else {
                        [[0.0, 0.0], [0.0, 10.0], [10.0, 10.0]]
                    },
                );
                let nowrap = run(
                    items,
                    Some(25.0),
                    AvailableOf::definite(S::from_f64(25.0)),
                    reverse,
                    FlexWrap::NoWrap,
                    0.0,
                    0.0,
                );
                assert_size(&nowrap, 25.0, 10.0);
                assert_cells(
                    &nowrap,
                    if reverse {
                        [[15.0, 0.0], [5.0, 0.0], [-5.0, 0.0]]
                    } else {
                        [[0.0, 0.0], [10.0, 0.0], [20.0, 0.0]]
                    },
                );
            }
        }
    }
    lane::<f32>();
    lane::<f64>();
}

#[test]
fn flex_edges_follow_actual_item_order_and_exterior_forces_add_no_empty_gap() {
    fn lane<S: LayoutScalar>() {
        for reverse in [false, true] {
            let mut ordered = cells::<S>();
            for (item, order) in ordered.iter_mut().zip([2, -1, 0]) {
                item.item_order = ItemOrder::new(order);
            }
            ordered[1].wrap_edges.after = WrapEdge::Flex;
            let forced = run(
                ordered.clone(),
                Some(100.0),
                AvailableOf::definite(S::from_f64(100.0)),
                reverse,
                FlexWrap::Wrap,
                0.0,
                0.0,
            );
            assert_size(&forced, 100.0, 20.0);
            assert_cells(
                &forced,
                if reverse {
                    [[80.0, 10.0], [90.0, 0.0], [90.0, 10.0]]
                } else {
                    [[10.0, 10.0], [0.0, 0.0], [0.0, 10.0]]
                },
            );
            ordered[1].wrap_edges.after = WrapEdge::Auto;
            ordered[1].wrap_edges.before = WrapEdge::Flex; // first in order
            ordered[0].wrap_edges.after = WrapEdge::Flex; // last in order
            let exterior = run(
                ordered,
                Some(25.0),
                AvailableOf::definite(S::from_f64(25.0)),
                reverse,
                FlexWrap::Wrap,
                0.0,
                3.0,
            );
            // Flex6 requires an item in each line. Two10px lines + one3px gap.
            assert_size(&exterior, 25.0, 23.0);
            assert_cells(
                &exterior,
                if reverse {
                    [[15.0, 13.0], [15.0, 0.0], [5.0, 0.0]]
                } else {
                    [[0.0, 13.0], [0.0, 0.0], [10.0, 0.0]]
                },
            );
        }
    }
    lane::<f32>();
    lane::<f64>();
}

#[test]
fn flex_edge_fit_uses_signed_outer_margins_and_real_collection_gaps() {
    fn lane<S: LayoutScalar>() {
        for reverse in [false, true] {
            for avoid in [false, true] {
                let mut items = cells::<S>();
                for item in &mut items {
                    item.margin = if reverse {
                        Edges::new(
                            LengthAutoOf::ZERO,
                            LengthAutoOf::px(S::from_f64(5.0)),
                            LengthAutoOf::ZERO,
                            LengthAutoOf::px(S::from_f64(-5.0)),
                        )
                    } else {
                        Edges::new(
                            LengthAutoOf::ZERO,
                            LengthAutoOf::px(S::from_f64(-5.0)),
                            LengthAutoOf::ZERO,
                            LengthAutoOf::px(S::from_f64(5.0)),
                        )
                    };
                }
                items[1].wrap_edges.after = if avoid {
                    WrapEdge::AvoidFlex
                } else {
                    WrapEdge::Auto
                };
                let signed = run(
                    items,
                    Some(25.0),
                    AvailableOf::definite(S::from_f64(25.0)),
                    reverse,
                    FlexWrap::Wrap,
                    0.0,
                    0.0,
                );
                assert_size(&signed, 25.0, 20.0);
                assert_cells(
                    &signed,
                    match (reverse, avoid) {
                        (false, false) => [[5.0, 0.0], [15.0, 0.0], [5.0, 10.0]],
                        (false, true) => [[5.0, 0.0], [5.0, 10.0], [15.0, 10.0]],
                        (true, false) => [[10.0, 0.0], [0.0, 0.0], [10.0, 10.0]],
                        (true, true) => [[10.0, 0.0], [10.0, 10.0], [0.0, 10.0]],
                    },
                );
                let mut items = cells::<S>();
                items[1].wrap_edges.after = if avoid {
                    WrapEdge::Avoid
                } else {
                    WrapEdge::Auto
                };
                let gapped = run(
                    items,
                    Some(25.0),
                    AvailableOf::definite(S::from_f64(25.0)),
                    reverse,
                    FlexWrap::Wrap,
                    5.0,
                    0.0,
                );
                assert_size(&gapped, 25.0, 20.0);
                assert_cells(
                    &gapped,
                    match (reverse, avoid) {
                        (false, false) => [[0.0, 0.0], [15.0, 0.0], [0.0, 10.0]],
                        (false, true) => [[0.0, 0.0], [0.0, 10.0], [15.0, 10.0]],
                        (true, false) => [[15.0, 0.0], [0.0, 0.0], [15.0, 10.0]],
                        (true, true) => [[15.0, 0.0], [15.0, 10.0], [0.0, 10.0]],
                    },
                );
            }
        }
    }
    lane::<f32>();
    lane::<f64>();
}

#[test]
fn flex_zero_main_item_keeps_its_real_line_height_and_edge_identity() {
    fn lane<S: LayoutScalar>() {
        for reverse in [false, true] {
            for (edge, expected_zero_y, expected_last_y) in [
                (WrapEdge::Auto, 0.0, 25.0),
                (WrapEdge::AvoidFlex, 10.0, 10.0),
                (WrapEdge::Flex, 0.0, 25.0),
            ] {
                let mut items = [fixed::<S>(20.0, 10.0), fixed(0.0, 25.0), fixed(10.0, 10.0)];
                items[1].wrap_edges.after = edge;
                let batch = run(
                    items,
                    Some(20.0),
                    AvailableOf::definite(S::from_f64(20.0)),
                    reverse,
                    FlexWrap::Wrap,
                    0.0,
                    0.0,
                );
                assert_size(&batch, 20.0, 35.0);
                assert_item(&batch, 1, [0.0, 0.0, 20.0, 10.0]);
                let zero_x = match (reverse, edge == WrapEdge::AvoidFlex) {
                    (false, false) | (true, true) => 20.0,
                    (false, true) | (true, false) => 0.0,
                };
                assert_item(&batch, 2, [zero_x, expected_zero_y, 0.0, 25.0]);
                assert_item(
                    &batch,
                    3,
                    [
                        if reverse { 10.0 } else { 0.0 },
                        expected_last_y,
                        10.0,
                        10.0,
                    ],
                );
            }
        }
    }
    lane::<f32>();
    lane::<f64>();
}

#[test]
fn flex_intrinsic_groups_honor_force_and_yield_soft_avoidance() {
    fn lane<S: LayoutScalar>() {
        for reverse in [false, true] {
            for edge in [
                WrapEdge::Auto,
                WrapEdge::Avoid,
                WrapEdge::AvoidFlex,
                WrapEdge::Flex,
            ] {
                let mut items = cells::<S>();
                items[0].wrap_edges.after = edge;
                let min = run(
                    items.clone(),
                    None,
                    AvailableOf::MIN_CONTENT,
                    reverse,
                    FlexWrap::Wrap,
                    0.0,
                    0.0,
                );
                assert_size(&min, 10.0, 30.0);
                assert_cells(&min, [[0.0, 0.0], [0.0, 10.0], [0.0, 20.0]]);
                let max = run(
                    items,
                    None,
                    AvailableOf::MAX_CONTENT,
                    reverse,
                    FlexWrap::Wrap,
                    0.0,
                    0.0,
                );
                if edge == WrapEdge::Flex {
                    assert_size(&max, 20.0, 20.0);
                    assert_cells(
                        &max,
                        if reverse {
                            [[10.0, 0.0], [10.0, 10.0], [0.0, 10.0]]
                        } else {
                            [[0.0, 0.0], [0.0, 10.0], [10.0, 10.0]]
                        },
                    );
                } else {
                    assert_size(&max, 30.0, 10.0);
                    assert_cells(
                        &max,
                        if reverse {
                            [[20.0, 0.0], [10.0, 0.0], [0.0, 0.0]]
                        } else {
                            [[0.0, 0.0], [10.0, 0.0], [20.0, 0.0]]
                        },
                    );
                }
            }
            let mut items = cells::<S>();
            items[0].wrap_edges.after = WrapEdge::Flex;
            let nowrap_min = run(
                items,
                None,
                AvailableOf::MIN_CONTENT,
                reverse,
                FlexWrap::NoWrap,
                0.0,
                0.0,
            );
            assert_size(&nowrap_min, 30.0, 10.0);
            assert_cells(
                &nowrap_min,
                if reverse {
                    [[20.0, 0.0], [10.0, 0.0], [0.0, 0.0]]
                } else {
                    [[0.0, 0.0], [10.0, 0.0], [20.0, 0.0]]
                },
            );
        }
    }
    lane::<f32>();
    lane::<f64>();
}

#[test]
fn collapsed_item_causes_survive_recollection_but_hidden_and_absolute_items_do_not() {
    fn lane<S: LayoutScalar>() {
        for reverse in [false, true] {
            // Collapse keeps a real first collected item, even though item2
            // is first visible. Its first-round30px strut survives recollection.
            for (after_collapsed, before_visible, separated, exterior) in [
                (WrapEdge::Auto, WrapEdge::Auto, false, false),
                (WrapEdge::Auto, WrapEdge::Auto, false, true),
                (WrapEdge::Flex, WrapEdge::Auto, true, true),
                (WrapEdge::Auto, WrapEdge::Flex, true, true),
                (WrapEdge::Flex, WrapEdge::Flex, true, true),
                (WrapEdge::Flex, WrapEdge::Avoid, true, true),
            ] {
                let mut items = [fixed::<S>(10.0, 30.0), fixed(10.0, 10.0), fixed(10.0, 10.0)];
                items[0].flex_item_collapse = FlexItemCollapse::Collapsed;
                items[0].wrap_edges = WrapEdges {
                    before: if exterior {
                        WrapEdge::Flex
                    } else {
                        WrapEdge::Auto
                    },
                    after: after_collapsed,
                };
                items[1].wrap_edges.before = before_visible;
                items[2].wrap_edges.after = if exterior {
                    WrapEdge::Flex
                } else {
                    WrapEdge::Auto
                };
                let batch = run(
                    items,
                    Some(100.0),
                    AvailableOf::definite(S::from_f64(100.0)),
                    reverse,
                    FlexWrap::Wrap,
                    0.0,
                    3.0,
                );
                assert_size(&batch, 100.0, if separated { 43.0 } else { 30.0 });
                assert_eq!(
                    output(&batch, 1),
                    NodeOutputOf::with_source_index(SourceIndex::new(0))
                );
                let y = if separated { 33.0 } else { 0.0 };
                assert_item(&batch, 2, [if reverse { 90.0 } else { 0.0 }, y, 10.0, 10.0]);
                assert_item(
                    &batch,
                    3,
                    [if reverse { 80.0 } else { 10.0 }, y, 10.0, 10.0],
                );
            }
            for hidden in [false, true] {
                let mut items = [fixed::<S>(10.0, 30.0), fixed(10.0, 10.0), fixed(10.0, 10.0)];
                if hidden {
                    items[0].display = Display::None;
                } else {
                    items[0].position = Position::Absolute;
                }
                items[0].wrap_edges.after = WrapEdge::Flex;
                items[1].wrap_edges.before = WrapEdge::Flex;
                let batch = run(
                    items,
                    Some(100.0),
                    AvailableOf::definite(S::from_f64(100.0)),
                    reverse,
                    FlexWrap::Wrap,
                    0.0,
                    3.0,
                );
                assert_size(&batch, 100.0, 10.0);
                assert_item(
                    &batch,
                    2,
                    [if reverse { 90.0 } else { 0.0 }, 0.0, 10.0, 10.0],
                );
                assert_item(
                    &batch,
                    3,
                    [if reverse { 80.0 } else { 10.0 }, 0.0, 10.0, 10.0],
                );
                if hidden {
                    assert_eq!(
                        output(&batch, 1),
                        NodeOutputOf::with_source_index(SourceIndex::new(0))
                    );
                }
            }
        }
    }
    lane::<f32>();
    lane::<f64>();
}
