use super::*;

fn inset_percent<S: LayoutScalar>(fraction: f64) -> LengthAutoOf<S> {
    LengthAutoOf::value(LengthPercentageOf::from_percent_fraction(px::<S>(fraction)).unwrap())
}

fn percentage_position<S: LayoutScalar>(
    batch: &CompletedLayoutBatchOf<u32, S>,
    node: u32,
    raw: Point<S>,
    rounded: (f64, f64),
    size: (f64, f64),
) {
    // Percentage coefficients and their resolved product are in the tree's S
    // lane. These expected values use those same documented scalar operations,
    // not decimal-literal bits or a relaxed geometry tolerance.
    assert_eq!(output(batch, node).location, raw, "unrounded node {node}");
    assert_eq!(
        output(batch, node).size,
        Size::new(px::<S>(size.0), px::<S>(size.1))
    );
    let final_node = batch
        .final_entries()
        .iter()
        .find(|entry| entry.node() == node)
        .unwrap()
        .output();
    assert_eq!(
        final_node.location,
        Point::new(px::<S>(rounded.0), px::<S>(rounded.1)),
        "final node {node}"
    );
    assert_eq!(final_node.size, Size::new(px::<S>(size.0), px::<S>(size.1)));
}

fn opposite(direction: Direction) -> Direction {
    match direction {
        Direction::Ltr => Direction::Rtl,
        Direction::Rtl => Direction::Ltr,
    }
}

fn percentage_horizontal_contexts<S: LayoutScalar>(definite_bases: &[bool]) {
    for &definite in definite_bases {
        for display in [Display::Flex, Display::Grid, Display::Block] {
            for direction in [Direction::Ltr, Direction::Rtl] {
                for box_sizing in [BoxSizing::BorderBox, BoxSizing::ContentBox] {
                    // Existing real border10 fixture, including its actual1px
                    // padding+border on both sides in either sizing mode.
                    let mut tree = three_boxes::<S>(display, direction, box_sizing, true);
                    let mut parent = root_style(direction);
                    if definite {
                        parent.size.height = PreferredSizeOf::px(px::<S>(100.0));
                    }
                    tree.insert_input(0, LayoutInputOf::box_input(parent));
                    for node in [1, 2, 3] {
                        let mut child = crate::LayoutTree::node_input(&tree, node).clone();
                        child.direction = opposite(direction);
                        tree.insert_input(node, LayoutInputOf::box_input(child));
                    }
                    let control = compute(&tree);
                    let height = if definite { 100.0 } else { 30.0 };
                    assert_box(&control, 0, (0.0, 0.0), (50.0, height));
                    for (node, y) in [(1, 0.0), (2, 10.0), (3, 20.0)] {
                        assert_box(&control, node, (0.0, y), (50.0, 10.0));
                    }
                    replace_inset(
                        &mut tree,
                        1,
                        Edges {
                            top: inset_percent(0.04),
                            left: inset_percent(0.04),
                            right: inset_percent(0.08),
                            ..Edges::all(LengthAutoOf::AUTO)
                        },
                    );
                    replace_inset(
                        &mut tree,
                        2,
                        Edges {
                            bottom: inset_percent(0.08),
                            ..Edges::all(LengthAutoOf::AUTO)
                        },
                    );
                    replace_inset(
                        &mut tree,
                        3,
                        Edges {
                            top: inset_percent(0.04),
                            bottom: inset_percent(0.08),
                            ..Edges::all(LengthAutoOf::AUTO)
                        },
                    );
                    let actual = compute(&tree);
                    // Position3: top/bottom use physical HEIGHT, not width50.
                    // Definite100 =>4,2,24. Selected native final-auto basis30
                    // =>1.2,7.6,21.2 mathematically, then final pixels1,8,21.
                    // The auto basis is native policy, not a CSSWG resolution.
                    let four = px::<S>(0.04) * px::<S>(height);
                    let eight = px::<S>(0.08) * px::<S>(height);
                    let x = if direction == Direction::Ltr {
                        px::<S>(0.04) * px::<S>(50.0)
                    } else {
                        -(px::<S>(0.08) * px::<S>(50.0))
                    };
                    let final_x = if direction == Direction::Ltr {
                        2.0
                    } else {
                        -4.0
                    };
                    percentage_position(
                        &actual,
                        1,
                        Point::new(x, four),
                        (final_x, if definite { 4.0 } else { 1.0 }),
                        (50.0, 10.0),
                    );
                    percentage_position(
                        &actual,
                        2,
                        Point::new(S::ZERO, px::<S>(10.0) - eight),
                        (0.0, if definite { 2.0 } else { 8.0 }),
                        (50.0, 10.0),
                    );
                    percentage_position(
                        &actual,
                        3,
                        Point::new(S::ZERO, px::<S>(20.0) + four),
                        (0.0, if definite { 24.0 } else { 21.0 }),
                        (50.0, 10.0),
                    );
                    // Root normal size and old immutable publication stay put.
                    assert_box(&actual, 0, (0.0, 0.0), (50.0, height));
                    assert_box(&control, 2, (0.0, 10.0), (50.0, 10.0));
                }
            }
        }
    }
}

fn percentage_vertical_contexts<S: LayoutScalar>(definite_bases: &[bool]) {
    for mode in [WritingMode::VerticalLr, WritingMode::VerticalRl] {
        for &definite in definite_bases {
            for direction in [Direction::Ltr, Direction::Rtl] {
                for display in [Display::Block, Display::Flex, Display::Grid] {
                    for box_sizing in [BoxSizing::BorderBox, BoxSizing::ContentBox] {
                        let width = if definite { 100.0 } else { 30.0 };
                        let mut tree = PublicLayoutTreeOf::<S>::new().children(0, [1, 2, 3]).style(
                            0,
                            NodeInputOf {
                                display: Display::Block,
                                writing_mode: mode,
                                direction,
                                size: Size::new(
                                    if definite {
                                        PreferredSizeOf::px(px::<S>(100.0))
                                    } else {
                                        PreferredSizeOf::AUTO
                                    },
                                    PreferredSizeOf::px(px::<S>(50.0)),
                                ),
                                ..NodeInputOf::default()
                            },
                        );
                        for node in [1, 2, 3] {
                            let mut child = ten_box(display, opposite(direction), box_sizing, true);
                            child.writing_mode = mode;
                            child.size = Size::new(
                                PreferredSizeOf::px(px::<S>(
                                    if box_sizing == BoxSizing::ContentBox {
                                        6.0
                                    } else {
                                        10.0
                                    },
                                )),
                                PreferredSizeOf::px(px::<S>(
                                    if box_sizing == BoxSizing::ContentBox {
                                        46.0
                                    } else {
                                        50.0
                                    },
                                )),
                            );
                            tree = tree.children(node, []).style(node, child);
                        }
                        let run = |tree: &PublicLayoutTreeOf<S>| {
                            compute_layout(
                                tree,
                                0,
                                LayoutRootRequestOf::viewport(Size::splat(
                                    AvailableOf::MAX_CONTENT,
                                ))
                                .unwrap(),
                            )
                            .unwrap()
                        };
                        let control = run(&tree);
                        assert_box(&control, 0, (0.0, 0.0), (width, 50.0));
                        let normals = if mode == WritingMode::VerticalLr {
                            [0.0, 10.0, 20.0]
                        } else if definite {
                            [90.0, 80.0, 70.0]
                        } else {
                            [20.0, 10.0, 0.0]
                        };
                        for (node, x) in [(1, normals[0]), (2, normals[1]), (3, normals[2])] {
                            assert_box(&control, node, (x, 0.0), (10.0, 50.0));
                        }
                        replace_inset(
                            &mut tree,
                            1,
                            Edges {
                                left: inset_percent(0.04),
                                top: inset_percent(0.04),
                                bottom: inset_percent(0.08),
                                ..Edges::all(LengthAutoOf::AUTO)
                            },
                        );
                        replace_inset(
                            &mut tree,
                            2,
                            Edges {
                                right: inset_percent(0.08),
                                ..Edges::all(LengthAutoOf::AUTO)
                            },
                        );
                        replace_inset(
                            &mut tree,
                            3,
                            Edges {
                                left: inset_percent(0.04),
                                right: inset_percent(0.08),
                                ..Edges::all(LengthAutoOf::AUTO)
                            },
                        );
                        let actual = run(&tree);
                        // Physical width is100 or final-auto30, not inline50.
                        // Parent block-start selects LEFT for VLR, RIGHT for
                        // VRL. Parent inline-start selects TOP for LTR, BOTTOM
                        // for RTL, despite each child's opposite direction.
                        let four = px::<S>(0.04) * px::<S>(width);
                        let eight = px::<S>(0.08) * px::<S>(width);
                        let first_y = if direction == Direction::Ltr {
                            px::<S>(0.04) * px::<S>(50.0)
                        } else {
                            -(px::<S>(0.08) * px::<S>(50.0))
                        };
                        let finals = match (mode, definite) {
                            (WritingMode::VerticalLr, true) => [4.0, 2.0, 24.0],
                            (WritingMode::VerticalLr, false) => [1.0, 8.0, 21.0],
                            (WritingMode::VerticalRl, true) => [94.0, 72.0, 62.0],
                            (WritingMode::VerticalRl, false) => [21.0, 8.0, -2.0],
                            _ => unreachable!(),
                        };
                        percentage_position(
                            &actual,
                            1,
                            Point::new(px::<S>(normals[0]) + four, first_y),
                            (
                                finals[0],
                                if direction == Direction::Ltr {
                                    2.0
                                } else {
                                    -4.0
                                },
                            ),
                            (10.0, 50.0),
                        );
                        percentage_position(
                            &actual,
                            2,
                            Point::new(px::<S>(normals[1]) - eight, S::ZERO),
                            (finals[1], 0.0),
                            (10.0, 50.0),
                        );
                        let third = if mode == WritingMode::VerticalLr {
                            px::<S>(normals[2]) + four
                        } else {
                            px::<S>(normals[2]) - eight
                        };
                        percentage_position(
                            &actual,
                            3,
                            Point::new(third, S::ZERO),
                            (finals[2], 0.0),
                            (10.0, 50.0),
                        );
                        assert_box(&actual, 0, (0.0, 0.0), (width, 50.0));
                    }
                }
            }
        }
    }
}

fn final_inset_basis_does_not_settle_size_percentages<S: LayoutScalar>() {
    let direction = Direction::Ltr;
    let mut tree = three_boxes::<S>(Display::Block, direction, BoxSizing::BorderBox, false);
    tree.insert_children(0, [1, 2, 3, 4]);
    tree.insert_children(4, []);
    tree.insert_input(
        4,
        LayoutInputOf::box_input(NodeInputOf {
            display: Display::Block,
            position: Position::Relative,
            size: Size::new(
                PreferredSizeOf::AUTO,
                PreferredSizeOf::value(
                    LengthPercentageOf::from_percent_fraction(px::<S>(0.5)).unwrap(),
                ),
            ),
            inset: Edges {
                top: inset_percent(0.04),
                ..Edges::all(LengthAutoOf::AUTO)
            },
            ..NodeInputOf::default()
        }),
    );
    let actual = compute(&tree);
    // Ordinary percentage height still has an indefinite incoming basis. The
    // empty fourth block therefore has height0, not50% of the final parent30.
    // Only its relative top uses the property's final-auto inset basis30.
    assert_box(&actual, 0, (0.0, 0.0), (50.0, 30.0));
    percentage_position(
        &actual,
        4,
        Point::new(S::ZERO, px::<S>(30.0) + px::<S>(0.04) * px::<S>(30.0)),
        (0.0, 31.0),
        (50.0, 0.0),
    );
    for (node, y) in [(1, 0.0), (2, 10.0), (3, 20.0)] {
        assert_box(&actual, node, (0.0, y), (50.0, 10.0));
    }
}

fn percentage_numeric_atomic<S: LayoutScalar>(definite_bases: &[bool]) {
    for &definite in definite_bases {
        let mut tree =
            three_boxes::<S>(Display::Block, Direction::Ltr, BoxSizing::BorderBox, false);
        let mut parent = root_style(Direction::Ltr);
        if definite {
            parent.size.height = PreferredSizeOf::px(px::<S>(100.0));
        }
        tree.insert_input(0, LayoutInputOf::box_input(parent));
        let mut atomic = ten_box(
            Display::InlineBlock,
            Direction::Ltr,
            BoxSizing::BorderBox,
            false,
        );
        atomic.size.width = PreferredSizeOf::px(px::<S>(10.0));
        atomic.atomic_inline_participation =
            Some(crate::block_tests::fixtures::fri06_atomic_participation());
        atomic.inset = Edges {
            top: inset_percent(0.04),
            ..Edges::all(LengthAutoOf::AUTO)
        };
        tree.insert_input(1, LayoutInputOf::box_input(atomic));
        let actual = compute(&tree);
        let height = if definite { 100.0 } else { 30.0 };
        // Genuine numeric host-supplied level0 atomic, no Text/capture/font.
        // Its line and the two following ordinary blocks retain normal height30.
        assert_box(&actual, 0, (0.0, 0.0), (50.0, height));
        percentage_position(
            &actual,
            1,
            Point::new(S::ZERO, px::<S>(0.04) * px::<S>(height)),
            (0.0, if definite { 4.0 } else { 1.0 }),
            (10.0, 10.0),
        );
        assert_box(&actual, 2, (0.0, 10.0), (50.0, 10.0));
        assert_box(&actual, 3, (0.0, 20.0), (50.0, 10.0));
    }
}

#[test]
fn percentage_insets_horizontal_containing_basis_f32() {
    percentage_horizontal_contexts::<f32>(&[true, false]);
}
#[test]
fn percentage_insets_horizontal_containing_basis_f64() {
    percentage_horizontal_contexts::<f64>(&[true, false]);
}
#[test]
fn percentage_insets_vertical_physical_basis_f32() {
    percentage_vertical_contexts::<f32>(&[true, false]);
}
#[test]
fn percentage_insets_vertical_physical_basis_f64() {
    percentage_vertical_contexts::<f64>(&[true, false]);
}
#[test]
fn percentage_inset_final_basis_keeps_size_indefinite_f32() {
    final_inset_basis_does_not_settle_size_percentages::<f32>();
}
#[test]
fn percentage_inset_final_basis_keeps_size_indefinite_f64() {
    final_inset_basis_does_not_settle_size_percentages::<f64>();
}
#[test]
fn percentage_insets_numeric_atomic_f32() {
    percentage_numeric_atomic::<f32>(&[true, false]);
}
#[test]
fn percentage_insets_numeric_atomic_f64() {
    percentage_numeric_atomic::<f64>(&[true, false]);
}

// Isolate definite physical-axis behavior from final-auto settlement.
#[test]
fn definite_relative_inset_axes_f32() {
    percentage_horizontal_contexts::<f32>(&[true]);
    percentage_vertical_contexts::<f32>(&[true]);
    percentage_numeric_atomic::<f32>(&[true]);
}
#[test]
fn definite_relative_inset_axes_f64() {
    percentage_horizontal_contexts::<f64>(&[true]);
    percentage_vertical_contexts::<f64>(&[true]);
    percentage_numeric_atomic::<f64>(&[true]);
}
