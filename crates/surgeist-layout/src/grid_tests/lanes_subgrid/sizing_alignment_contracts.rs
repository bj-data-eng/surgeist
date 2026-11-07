use super::*;

fn sizing_point<S: LayoutScalar>(axis: GridAxisKind, grid: f64, stacking: f64) -> Point<S> {
    let px = S::from_f64;
    if axis == GridAxisKind::Column {
        Point::new(px(grid), px(stacking))
    } else {
        Point::new(px(stacking), px(grid))
    }
}

fn sizing_size<S: LayoutScalar>(axis: GridAxisKind, grid: f64, stacking: f64) -> Size<S> {
    let point = sizing_point(axis, grid, stacking);
    Size::new(point.x, point.y)
}

fn sizing_parent<S: LayoutScalar>(axis: GridAxisKind, lanes: usize, gap: f64) -> NodeInputOf<S> {
    let px = S::from_f64;
    let columns = axis == GridAxisKind::Column;
    NodeInputOf {
        display: Display::GridLanes,
        grid_lanes_flow: GridLanesFlow::new(axis, false, false),
        size: sizing_size(axis, 10.0 * lanes as f64, 100.0).map(PreferredSizeOf::px),
        grid_template_columns: if columns {
            vec![TrackComponentOf::px(px(10.0)); lanes]
        } else {
            Vec::new()
        },
        grid_template_rows: if columns {
            Vec::new()
        } else {
            vec![TrackComponentOf::px(px(10.0)); lanes]
        },
        gap: if columns {
            Size::new(LengthOf::ZERO, LengthOf::px(px(gap)))
        } else {
            Size::new(LengthOf::px(px(gap)), LengthOf::ZERO)
        },
        grid_flow_tolerance: GridFlowToleranceOf::Length(LengthOf::ZERO),
        justify_content: Some(AlignContent::Start),
        align_content: Some(AlignContent::Start),
        ..NodeInputOf::default()
    }
}

fn sizing_item<S: LayoutScalar>(
    axis: GridAxisKind,
    lane: isize,
    stacking: PreferredSizeOf<S>,
    alignment: AlignItems,
    margin_start: LengthAutoOf<S>,
    margin_end: LengthAutoOf<S>,
) -> NodeInputOf<S> {
    let px = S::from_f64;
    let columns = axis == GridAxisKind::Column;
    NodeInputOf {
        display: Display::Block,
        size: if columns {
            Size::new(PreferredSizeOf::px(px(10.0)), stacking)
        } else {
            Size::new(stacking, PreferredSizeOf::px(px(10.0)))
        },
        grid_column: if columns {
            GridPlacement::try_line(lane).unwrap()
        } else {
            GridPlacement::AUTO
        },
        grid_row: if columns {
            GridPlacement::AUTO
        } else {
            GridPlacement::try_line(lane).unwrap()
        },
        justify_self: Some(if columns {
            AlignItems::Start
        } else {
            alignment
        }),
        align_self: Some(if columns {
            alignment
        } else {
            AlignItems::Start
        }),
        margin: if columns {
            Edges::new(
                margin_start,
                LengthAutoOf::ZERO,
                margin_end,
                LengthAutoOf::ZERO,
            )
        } else {
            Edges::new(
                LengthAutoOf::ZERO,
                margin_end,
                LengthAutoOf::ZERO,
                margin_start,
            )
        },
        ..NodeInputOf::default()
    }
}

fn natural_child<S: LayoutScalar>() -> NodeInputOf<S> {
    NodeInputOf {
        display: Display::Block,
        size: Size::splat(S::from_f64(10.0)).map(PreferredSizeOf::px),
        ..NodeInputOf::default()
    }
}

fn check_sizing_geometry<S: LayoutScalar>(
    tree: &PublicLayoutTreeOf<S>,
    axis: GridAxisKind,
    lanes: usize,
    expected: &[(u32, f64, f64, f64)],
    label: &str,
    differences: &mut Vec<String>,
) -> Option<CompletedLayoutBatchOf<u32, S>> {
    let batch = match compute_layout(
        tree,
        1,
        super::super::fixtures::fri08_c01_placement_request(),
    ) {
        Ok(batch) => batch,
        Err(error) => {
            differences.push(format!("{axis:?}, {label}: {error:?}"));
            return None;
        }
    };
    let root = fri08_c01_placement_output(&batch, 1);
    let expected_root = (Point::ZERO, sizing_size(axis, lanes as f64 * 10.0, 100.0));
    if (root.location, root.size) != expected_root {
        differences.push(format!(
            "{axis:?}, {label}: root {:?} {:?}, expected {expected_root:?}",
            root.location, root.size
        ));
    }
    for &(node, grid_start, stacking_start, stacking_size) in expected {
        let output = fri08_c01_placement_output(&batch, node);
        let actual = (output.location, output.size);
        let expected = (
            sizing_point(axis, grid_start, stacking_start),
            sizing_size(axis, 10.0, stacking_size),
        );
        if actual != expected {
            differences.push(format!(
                "{axis:?}, {label}, node {node}: {actual:?}, expected {expected:?}"
            ));
        }
    }
    Some(batch)
}

fn assert_auto_margins_absorb_only_bounded_cell_slack<S: LayoutScalar>() {
    let px = S::from_f64;
    let mut differences = Vec::new();
    for axis in [GridAxisKind::Column, GridAxisKind::Row] {
        for (label, alignment, auto_start, auto_end, start_margin, end_margin) in [
            (
                "zero Start control",
                AlignItems::Start,
                false,
                false,
                0.0,
                0.0,
            ),
            (
                "start Auto before End",
                AlignItems::End,
                true,
                false,
                20.0,
                0.0,
            ),
            (
                "end Auto before End",
                AlignItems::End,
                false,
                true,
                0.0,
                20.0,
            ),
            (
                "both Auto before End",
                AlignItems::End,
                true,
                true,
                10.0,
                10.0,
            ),
        ] {
            let tree = PublicLayoutTreeOf::<S>::new()
                .children(1, [2, 3])
                .children(2, [])
                .children(3, [])
                .style(1, sizing_parent(axis, 2, 0.0))
                .style(
                    2,
                    sizing_item(
                        axis,
                        1,
                        PreferredSizeOf::px(px(10.0)),
                        alignment,
                        if auto_start {
                            LengthAutoOf::AUTO
                        } else {
                            LengthAutoOf::ZERO
                        },
                        if auto_end {
                            LengthAutoOf::AUTO
                        } else {
                            LengthAutoOf::ZERO
                        },
                    ),
                )
                .style(
                    3,
                    sizing_item(
                        axis,
                        2,
                        PreferredSizeOf::px(px(30.0)),
                        AlignItems::Start,
                        LengthAutoOf::ZERO,
                        LengthAutoOf::ZERO,
                    ),
                );
            // The terminal cap is 30, leaving 30-10 = 20 of positive cell
            // slack. Grid 2 auto margins absorb it before self alignment:
            // one Auto receives 20, two Autos receive 10 each, including End.
            if let Some(batch) = check_sizing_geometry(
                &tree,
                axis,
                2,
                &[(2, 0.0, start_margin, 10.0), (3, 10.0, 0.0, 30.0)],
                label,
                &mut differences,
            ) {
                let short = fri08_c01_placement_output(&batch, 2);
                let actual = if axis == GridAxisKind::Column {
                    (short.margin.top, short.margin.bottom)
                } else {
                    (short.margin.left, short.margin.right)
                };
                let expected = (px(start_margin), px(end_margin));
                if actual != expected {
                    differences.push(format!(
                        "{axis:?}, {label}: resolved margins {actual:?}, expected {expected:?}"
                    ));
                }
            }
        }
    }
    assert!(differences.is_empty(), "{}", differences.join("\n"));
}

#[test]
fn auto_margins_absorb_only_bounded_cell_slack_f32() {
    assert_auto_margins_absorb_only_bounded_cell_slack::<f32>();
}

#[test]
fn auto_margins_absorb_only_bounded_cell_slack_f64() {
    assert_auto_margins_absorb_only_bounded_cell_slack::<f64>();
}

fn assert_terminal_stretch_uses_frozen_cell<S: LayoutScalar>() {
    let px = S::from_f64;
    let mut differences = Vec::new();
    for axis in [GridAxisKind::Column, GridAxisKind::Row] {
        for (alignment, wrapper_size) in [(AlignItems::Start, 10.0), (AlignItems::Stretch, 30.0)] {
            let tree = PublicLayoutTreeOf::<S>::new()
                .children(1, [2, 3])
                .children(2, [4])
                .children(3, [])
                .children(4, [])
                .style(1, sizing_parent(axis, 2, 0.0))
                .style(
                    2,
                    sizing_item(
                        axis,
                        1,
                        PreferredSizeOf::AUTO,
                        alignment,
                        LengthAutoOf::ZERO,
                        LengthAutoOf::ZERO,
                    ),
                )
                .style(
                    3,
                    sizing_item(
                        axis,
                        2,
                        PreferredSizeOf::px(px(30.0)),
                        AlignItems::Start,
                        LengthAutoOf::ZERO,
                        LengthAutoOf::ZERO,
                    ),
                )
                .style(4, natural_child());
            // Native Block layout gives a natural stacking size of 10 from
            // its fixed child. The last-item cell ends at 30, so Stretch fills
            // 30, not the containing block's unused tail to 100. The child's
            // definite 10-unit extent remains unchanged by wrapper stretching.
            check_sizing_geometry(
                &tree,
                axis,
                2,
                &[
                    (2, 0.0, 0.0, wrapper_size),
                    (3, 10.0, 0.0, 30.0),
                    (4, 0.0, 0.0, 10.0),
                ],
                &format!("terminal {alignment:?}"),
                &mut differences,
            );
        }
    }
    assert!(differences.is_empty(), "{}", differences.join("\n"));
}

#[test]
fn terminal_stretch_uses_frozen_cell_f32() {
    assert_terminal_stretch_uses_frozen_cell::<f32>();
}

#[test]
fn terminal_stretch_uses_frozen_cell_f64() {
    assert_terminal_stretch_uses_frozen_cell::<f64>();
}

fn assert_sequential_auto_wrappers_do_not_stretch_into_tail<S: LayoutScalar>() {
    let mut differences = Vec::new();
    for axis in [GridAxisKind::Column, GridAxisKind::Row] {
        let tree = PublicLayoutTreeOf::<S>::new()
            .children(1, [2, 3])
            .children(2, [4])
            .children(3, [5])
            .children(4, [])
            .children(5, [])
            .style(1, sizing_parent(axis, 1, 5.0))
            .style(
                2,
                sizing_item(
                    axis,
                    1,
                    PreferredSizeOf::AUTO,
                    AlignItems::Stretch,
                    LengthAutoOf::ZERO,
                    LengthAutoOf::ZERO,
                ),
            )
            .style(
                3,
                sizing_item(
                    axis,
                    1,
                    PreferredSizeOf::AUTO,
                    AlignItems::Stretch,
                    LengthAutoOf::ZERO,
                    LengthAutoOf::ZERO,
                ),
            )
            .style(4, natural_child())
            .style(5, natural_child());
        // Natural 10-unit wrappers pack at 0 and 10+5 = 15. The first has an
        // ordinary successor and is ineligible for stacking self alignment.
        // The last has no slack before terminal cap 25, so both remain 10.
        check_sizing_geometry(
            &tree,
            axis,
            1,
            &[
                (2, 0.0, 0.0, 10.0),
                (3, 0.0, 15.0, 10.0),
                (4, 0.0, 0.0, 10.0),
                (5, 0.0, 0.0, 10.0),
            ],
            "sequential native auto wrappers",
            &mut differences,
        );
    }
    assert!(differences.is_empty(), "{}", differences.join("\n"));
}

#[test]
fn sequential_auto_wrappers_do_not_stretch_into_tail_f32() {
    assert_sequential_auto_wrappers_do_not_stretch_into_tail::<f32>();
}

#[test]
fn sequential_auto_wrappers_do_not_stretch_into_tail_f64() {
    assert_sequential_auto_wrappers_do_not_stretch_into_tail::<f64>();
}

fn assert_stretch_minmax_keeps_full_containing_block_basis<S: LayoutScalar>() {
    let px = S::from_f64;
    let mut differences = Vec::new();
    for axis in [GridAxisKind::Column, GridAxisKind::Row] {
        let minimum =
            MinSizeOf::value(LengthPercentageOf::from_percent_fraction(px(0.20)).unwrap());
        let maximum =
            MaxSizeOf::value(LengthPercentageOf::from_percent_fraction(px(0.25)).unwrap());
        let wrapper = NodeInputOf {
            min_size: if axis == GridAxisKind::Column {
                Size::new(MinSizeOf::AUTO, minimum)
            } else {
                Size::new(minimum, MinSizeOf::AUTO)
            },
            max_size: if axis == GridAxisKind::Column {
                Size::new(MaxSizeOf::NONE, maximum)
            } else {
                Size::new(maximum, MaxSizeOf::NONE)
            },
            ..sizing_item(
                axis,
                1,
                PreferredSizeOf::AUTO,
                AlignItems::Stretch,
                LengthAutoOf::ZERO,
                LengthAutoOf::ZERO,
            )
        };
        let tree = PublicLayoutTreeOf::<S>::new()
            .children(1, [2, 3])
            .children(2, [4])
            .children(3, [])
            .children(4, [])
            .style(1, sizing_parent(axis, 2, 0.0))
            .style(2, wrapper)
            .style(
                3,
                sizing_item(
                    axis,
                    2,
                    PreferredSizeOf::px(px(30.0)),
                    AlignItems::Start,
                    LengthAutoOf::ZERO,
                    LengthAutoOf::ZERO,
                ),
            )
            .style(4, natural_child());
        // Jan Grid 3's containing-block rule retains the full stacking content
        // box, 100, independently of the 30-unit alignment cell. Min/max are
        // 20%*100 = 20 and 25%*100 = 25. Stretch toward 30 therefore clamps to
        // 25. Resolving against the cell instead would invent a 7.5-unit cap.
        check_sizing_geometry(
            &tree,
            axis,
            2,
            &[
                (2, 0.0, 0.0, 25.0),
                (3, 10.0, 0.0, 30.0),
                (4, 0.0, 0.0, 10.0),
            ],
            "percentage minmax retains full containing block",
            &mut differences,
        );
    }
    assert!(differences.is_empty(), "{}", differences.join("\n"));
}

#[test]
fn stretch_minmax_keeps_full_containing_block_basis_f32() {
    assert_stretch_minmax_keeps_full_containing_block_basis::<f32>();
}

#[test]
fn stretch_minmax_keeps_full_containing_block_basis_f64() {
    assert_stretch_minmax_keeps_full_containing_block_basis::<f64>();
}
