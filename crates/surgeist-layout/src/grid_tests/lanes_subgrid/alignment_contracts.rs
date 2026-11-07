use super::*;

fn alignment_point<S: LayoutScalar>(axis: GridAxisKind, grid: f64, stacking: f64) -> Point<S> {
    let px = S::from_f64;
    match axis {
        GridAxisKind::Column => Point::new(px(grid), px(stacking)),
        GridAxisKind::Row => Point::new(px(stacking), px(grid)),
    }
}

fn alignment_size<S: LayoutScalar>(axis: GridAxisKind, grid: f64, stacking: f64) -> Size<S> {
    let point = alignment_point(axis, grid, stacking);
    Size::new(point.x, point.y)
}

fn alignment_parent<S: LayoutScalar>(
    axis: GridAxisKind,
    lanes: usize,
    gap: f64,
    content_alignment: AlignContent,
) -> NodeInputOf<S> {
    let px = S::from_f64;
    let columns = axis == GridAxisKind::Column;
    NodeInputOf {
        display: Display::GridLanes,
        grid_lanes_flow: GridLanesFlow::new(axis, false, false),
        size: alignment_size(axis, 10.0 * lanes as f64, 100.0).map(PreferredSizeOf::px),
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
        justify_content: Some(if columns {
            AlignContent::Start
        } else {
            content_alignment
        }),
        align_content: Some(if columns {
            content_alignment
        } else {
            AlignContent::Start
        }),
        ..NodeInputOf::default()
    }
}

fn alignment_leaf<S: LayoutScalar>(
    axis: GridAxisKind,
    lane: isize,
    span: usize,
    stacking_size: f64,
    self_alignment: AlignItems,
    margin_start: f64,
    margin_end: f64,
) -> NodeInputOf<S> {
    let px = S::from_f64;
    let columns = axis == GridAxisKind::Column;
    NodeInputOf {
        display: Display::Block,
        size: alignment_size(axis, 10.0 * span as f64, stacking_size).map(PreferredSizeOf::px),
        grid_column: if columns {
            GridPlacement::try_line_span(lane, span).unwrap()
        } else {
            GridPlacement::AUTO
        },
        grid_row: if columns {
            GridPlacement::AUTO
        } else {
            GridPlacement::try_line_span(lane, span).unwrap()
        },
        justify_self: Some(if columns {
            AlignItems::Start
        } else {
            self_alignment
        }),
        align_self: Some(if columns {
            self_alignment
        } else {
            AlignItems::Start
        }),
        margin: if columns {
            Edges::new(px(margin_start), S::ZERO, px(margin_end), S::ZERO)
        } else {
            Edges::new(S::ZERO, px(margin_end), S::ZERO, px(margin_start))
        }
        .map(LengthAutoOf::px),
        ..NodeInputOf::default()
    }
}

fn check_alignment_geometry<S: LayoutScalar>(
    tree: &PublicLayoutTreeOf<S>,
    axis: GridAxisKind,
    lanes: usize,
    expected: &[(u32, f64, f64, f64, f64)],
    label: &str,
    differences: &mut Vec<String>,
) {
    let batch = match compute_layout(
        tree,
        1,
        super::super::fixtures::fri08_c01_placement_request(),
    ) {
        Ok(batch) => batch,
        Err(error) => {
            differences.push(format!("{axis:?}, {label}: {error:?}"));
            return;
        }
    };
    // These oracles describe continuous layout; publication rounds half-pixel
    // positions separately. Observe the corresponding public batch phase.
    let output_for = |node| {
        batch
            .unrounded_entries()
            .iter()
            .find(|entry| entry.node() == node)
            .expect("alignment output contains the requested node")
            .output()
    };
    let root = output_for(1);
    let expected_root = (
        Point::ZERO,
        alignment_size(axis, lanes as f64 * 10.0, 100.0),
    );
    if (root.location, root.size) != expected_root {
        differences.push(format!(
            "{axis:?}, {label}: root {:?} {:?}, expected {expected_root:?}",
            root.location, root.size
        ));
    }
    for &(node, grid_start, stacking_start, grid_size, stacking_size) in expected {
        let output = output_for(node);
        let actual = (output.location, output.size);
        let expected = (
            alignment_point(axis, grid_start, stacking_start),
            alignment_size(axis, grid_size, stacking_size),
        );
        if actual != expected {
            differences.push(format!(
                "{axis:?}, {label}, node {node}: {actual:?}, expected {expected:?}"
            ));
        }
    }
}

fn assert_consecutive_items_do_not_align_into_unused_tail<S: LayoutScalar>() {
    let mut differences = Vec::new();
    for axis in [GridAxisKind::Column, GridAxisKind::Row] {
        for alignment in [AlignItems::Start, AlignItems::End] {
            let tree = PublicLayoutTreeOf::<S>::new()
                .children(1, [2, 3])
                .children(2, [])
                .children(3, [])
                .style(1, alignment_parent(axis, 1, 5.0, AlignContent::Start))
                .style(2, alignment_leaf(axis, 1, 1, 10.0, alignment, 0.0, 0.0))
                .style(3, alignment_leaf(axis, 1, 1, 10.0, alignment, 0.0, 0.0));
            // Oct Grid 3 stacking-self-alignment: the ordinary next item makes
            // item 2 ineligible. Item 3 is terminal, but its alignment container
            // ends at the packed last outer edge 10+5+10 = 25, with no tail gap.
            check_alignment_geometry(
                &tree,
                axis,
                1,
                &[(2, 0.0, 0.0, 10.0, 10.0), (3, 0.0, 15.0, 10.0, 10.0)],
                &format!("consecutive {alignment:?}"),
                &mut differences,
            );
        }
    }
    assert!(differences.is_empty(), "{}", differences.join("\n"));
}

#[test]
fn consecutive_items_do_not_align_into_unused_tail_f32() {
    assert_consecutive_items_do_not_align_into_unused_tail::<f32>();
}

#[test]
fn consecutive_items_do_not_align_into_unused_tail_f64() {
    assert_consecutive_items_do_not_align_into_unused_tail::<f64>();
}

fn assert_terminal_item_aligns_within_global_packed_end<S: LayoutScalar>() {
    let mut differences = Vec::new();
    for axis in [GridAxisKind::Column, GridAxisKind::Row] {
        for (alignment, first_start) in [
            (AlignItems::Start, 0.0),
            (AlignItems::End, 20.0),
            (AlignItems::Center, 10.0),
        ] {
            let tree = PublicLayoutTreeOf::<S>::new()
                .children(1, [2, 3])
                .children(2, [])
                .children(3, [])
                .style(1, alignment_parent(axis, 2, 0.0, AlignContent::Start))
                .style(2, alignment_leaf(axis, 1, 1, 20.0, alignment, 0.0, 0.0))
                .style(
                    3,
                    alignment_leaf(axis, 2, 1, 40.0, AlignItems::Start, 0.0, 0.0),
                );
            // The global terminal outer edge is 40. The short item's available
            // slack is 40-20 = 20, giving End 20 and Center 10. The 100-unit
            // container's remaining tail is outside both alignment containers.
            check_alignment_geometry(
                &tree,
                axis,
                2,
                &[
                    (2, 0.0, first_start, 10.0, 20.0),
                    (3, 10.0, 0.0, 10.0, 40.0),
                ],
                &format!("terminal {alignment:?}"),
                &mut differences,
            );
        }
    }
    assert!(differences.is_empty(), "{}", differences.join("\n"));
}

#[test]
fn terminal_item_aligns_within_global_packed_end_f32() {
    assert_terminal_item_aligns_within_global_packed_end::<f32>();
}

#[test]
fn terminal_item_aligns_within_global_packed_end_f64() {
    assert_terminal_item_aligns_within_global_packed_end::<f64>();
}

fn assert_pre_spanner_alignment_reserves_fixed_gap<S: LayoutScalar>() {
    let mut differences = Vec::new();
    for axis in [GridAxisKind::Column, GridAxisKind::Row] {
        for (alignment, first_start) in [(AlignItems::Start, 0.0), (AlignItems::End, 20.0)] {
            let tree = PublicLayoutTreeOf::<S>::new()
                .children(1, [2, 3, 4])
                .children(2, [])
                .children(3, [])
                .children(4, [])
                .style(1, alignment_parent(axis, 2, 5.0, AlignContent::Start))
                .style(2, alignment_leaf(axis, 1, 1, 10.0, alignment, 0.0, 0.0))
                .style(
                    3,
                    alignment_leaf(axis, 2, 1, 30.0, AlignItems::Start, 0.0, 0.0),
                )
                .style(
                    4,
                    alignment_leaf(axis, 1, 2, 10.0, AlignItems::Start, 0.0, 0.0),
                );
            // The tall item's end 30 plus the fixed gap 5 places the span at
            // 35. The first item's alignment container ends at 35-5 = 30;
            // End therefore puts its 10-unit box at 20. Alignment cannot move
            // the spanner or change the tall item's fixed alignment container.
            check_alignment_geometry(
                &tree,
                axis,
                2,
                &[
                    (2, 0.0, first_start, 10.0, 10.0),
                    (3, 10.0, 0.0, 10.0, 30.0),
                    (4, 0.0, 35.0, 20.0, 10.0),
                ],
                &format!("pre-spanner {alignment:?}"),
                &mut differences,
            );
        }
    }
    assert!(differences.is_empty(), "{}", differences.join("\n"));
}

#[test]
fn pre_spanner_alignment_reserves_fixed_gap_f32() {
    assert_pre_spanner_alignment_reserves_fixed_gap::<f32>();
}

#[test]
fn pre_spanner_alignment_reserves_fixed_gap_f64() {
    assert_pre_spanner_alignment_reserves_fixed_gap::<f64>();
}

fn assert_content_center_uses_signed_terminal_range<S: LayoutScalar>() {
    let mut differences = Vec::new();
    for axis in [GridAxisKind::Column, GridAxisKind::Row] {
        for (alignment, first_start, terminal_start) in [
            (AlignContent::Start, 0.0, 5.0),
            (AlignContent::Center, 42.5, 47.5),
        ] {
            let tree = PublicLayoutTreeOf::<S>::new()
                .children(1, [2, 3])
                .children(2, [])
                .children(3, [])
                .style(1, alignment_parent(axis, 1, 5.0, alignment))
                .style(
                    2,
                    alignment_leaf(axis, 1, 1, 20.0, AlignItems::Start, 0.0, 0.0),
                )
                .style(
                    3,
                    alignment_leaf(axis, 1, 1, 10.0, AlignItems::Start, -20.0, 0.0),
                );
            // Jan Grid 3 defines the range by first/last outer edges. The last
            // item starts at 20+5 = 25, its signed outer size is 10-20 = -10,
            // and its terminal outer edge is 15. The range is 0..15 even though
            // the earlier item ends at 20. Center shift is (100-15)/2 = 42.5;
            // the terminal border starts at 25-20+42.5 = 47.5.
            check_alignment_geometry(
                &tree,
                axis,
                1,
                &[
                    (2, 0.0, first_start, 10.0, 20.0),
                    (3, 0.0, terminal_start, 10.0, 10.0),
                ],
                &format!("signed content {alignment:?}"),
                &mut differences,
            );
        }
    }
    assert!(differences.is_empty(), "{}", differences.join("\n"));
}

#[test]
fn content_center_uses_signed_terminal_range_f32() {
    assert_content_center_uses_signed_terminal_range::<f32>();
}

#[test]
fn content_center_uses_signed_terminal_range_f64() {
    assert_content_center_uses_signed_terminal_range::<f64>();
}

fn assert_positive_stacking_margins_do_not_collapse<S: LayoutScalar>() {
    let mut differences = Vec::new();
    for axis in [GridAxisKind::Column, GridAxisKind::Row] {
        let tree = PublicLayoutTreeOf::<S>::new()
            .children(1, [2, 3])
            .children(2, [])
            .children(3, [])
            .style(1, alignment_parent(axis, 1, 5.0, AlignContent::Start))
            .style(
                2,
                alignment_leaf(axis, 1, 1, 10.0, AlignItems::Start, 2.0, 3.0),
            )
            .style(
                3,
                alignment_leaf(axis, 1, 1, 10.0, AlignItems::Start, 2.0, 3.0),
            );
        // Each margin box is 2+10+3 = 15. The next starts at 15+5 = 20,
        // so the border starts are 2 and 20+2 = 22, with no margin collapse.
        check_alignment_geometry(
            &tree,
            axis,
            1,
            &[(2, 0.0, 2.0, 10.0, 10.0), (3, 0.0, 22.0, 10.0, 10.0)],
            "positive noncollapsing margins",
            &mut differences,
        );
    }
    assert!(differences.is_empty(), "{}", differences.join("\n"));
}

#[test]
fn positive_stacking_margins_do_not_collapse_f32() {
    assert_positive_stacking_margins_do_not_collapse::<f32>();
}

#[test]
fn positive_stacking_margins_do_not_collapse_f64() {
    assert_positive_stacking_margins_do_not_collapse::<f64>();
}

fn assert_content_translation_preserves_local_alignment_bounds<S: LayoutScalar>() {
    let mut differences = Vec::new();
    for axis in [GridAxisKind::Column, GridAxisKind::Row] {
        for (alignment, short_start) in [(AlignItems::Start, 30.0), (AlignItems::End, 50.0)] {
            let tree = PublicLayoutTreeOf::<S>::new()
                .children(1, [2, 3])
                .children(2, [])
                .children(3, [])
                .style(1, alignment_parent(axis, 2, 0.0, AlignContent::Center))
                .style(2, alignment_leaf(axis, 1, 1, 20.0, alignment, 0.0, 0.0))
                .style(
                    3,
                    alignment_leaf(axis, 2, 1, 40.0, AlignItems::Start, 0.0, 0.0),
                );
            // The packed range is 0..40, so content Center translates both
            // items by (100-40)/2 = 30. The short item's fixed local container
            // remains 0..40, yielding local End offset 40-20 = 20. Composing
            // these independent offsets gives 50; the tall item stays at 30.
            // Oct Grid 3 fixes the cells; Jan Grid 3 translates the full range.
            check_alignment_geometry(
                &tree,
                axis,
                2,
                &[
                    (2, 0.0, short_start, 10.0, 20.0),
                    (3, 10.0, 30.0, 10.0, 40.0),
                ],
                &format!("content translation with local {alignment:?}"),
                &mut differences,
            );
        }
    }
    assert!(differences.is_empty(), "{}", differences.join("\n"));
}

#[test]
fn content_translation_preserves_local_alignment_bounds_f32() {
    assert_content_translation_preserves_local_alignment_bounds::<f32>();
}

#[test]
fn content_translation_preserves_local_alignment_bounds_f64() {
    assert_content_translation_preserves_local_alignment_bounds::<f64>();
}

fn assert_signed_terminal_end_alignment_has_no_extra_slack<S: LayoutScalar>() {
    let mut differences = Vec::new();
    for axis in [GridAxisKind::Column, GridAxisKind::Row] {
        for alignment in [AlignItems::Start, AlignItems::End] {
            let tree = PublicLayoutTreeOf::<S>::new()
                .children(1, [2, 3])
                .children(2, [])
                .children(3, [])
                .style(1, alignment_parent(axis, 1, 5.0, AlignContent::Center))
                .style(
                    2,
                    alignment_leaf(axis, 1, 1, 20.0, AlignItems::Start, 0.0, 0.0),
                )
                .style(3, alignment_leaf(axis, 1, 1, 10.0, alignment, -20.0, 0.0));
            // The terminal margin box starts at 25 and has signed size -10,
            // ending at the global terminal cap 15. Its fixed alignment
            // container has that same signed extent: there is no extra slack
            // for End to consume. Clamping the region to zero would invent
            // 10 units of slack. The first/last outer-edge range remains 0..15,
            // giving Center translation 42.5 and terminal border 25-20+42.5.
            check_alignment_geometry(
                &tree,
                axis,
                1,
                &[(2, 0.0, 42.5, 10.0, 20.0), (3, 0.0, 47.5, 10.0, 10.0)],
                &format!("signed terminal {alignment:?} has no slack"),
                &mut differences,
            );
        }
    }
    assert!(differences.is_empty(), "{}", differences.join("\n"));
}

#[test]
fn signed_terminal_end_alignment_has_no_extra_slack_f32() {
    assert_signed_terminal_end_alignment_has_no_extra_slack::<f32>();
}

#[test]
fn signed_terminal_end_alignment_has_no_extra_slack_f64() {
    assert_signed_terminal_end_alignment_has_no_extra_slack::<f64>();
}

fn assert_negative_stacking_range_keeps_finite_margin_geometry<S: LayoutScalar>() {
    let mut differences = Vec::new();
    for axis in [GridAxisKind::Column, GridAxisKind::Row] {
        for (margin_start, margin_end, border_start) in [(-20.0, 0.0, -20.0), (0.0, -20.0, 0.0)] {
            let tree = PublicLayoutTreeOf::<S>::new()
                .children(1, [2])
                .children(2, [])
                .style(1, alignment_parent(axis, 1, 0.0, AlignContent::Start))
                .style(
                    2,
                    alignment_leaf(
                        axis,
                        1,
                        1,
                        10.0,
                        AlignItems::Start,
                        margin_start,
                        margin_end,
                    ),
                );
            // The sole packed margin box has signed extent 10-20=-10.
            // Finite negative margins remain valid layout inputs. Start
            // alignment leaves its border at the resolved start margin.
            check_alignment_geometry(
                &tree,
                axis,
                1,
                &[(2, 0.0, border_start, 10.0, 10.0)],
                &format!("negative range, margins {margin_start}/{margin_end}"),
                &mut differences,
            );
        }
    }
    assert!(differences.is_empty(), "{}", differences.join("\n"));
}

#[test]
fn negative_stacking_range_keeps_finite_margin_geometry_f32() {
    assert_negative_stacking_range_keeps_finite_margin_geometry::<f32>();
}

#[test]
fn negative_stacking_range_keeps_finite_margin_geometry_f64() {
    assert_negative_stacking_range_keeps_finite_margin_geometry::<f64>();
}

fn assert_stacking_baseline_fallback_uses_item_own_edge<S: LayoutScalar>() {
    let px = S::from_f64;
    let mut differences = Vec::new();
    for (writing_mode, first_x, last_x) in [
        (WritingMode::VerticalRl, 20.0, 0.0),
        (WritingMode::VerticalLr, 0.0, 20.0),
    ] {
        for (alignment, short_x) in [
            (AlignItems::Baseline, first_x),
            (AlignItems::LastBaseline, last_x),
        ] {
            let tree = PublicLayoutTreeOf::<S>::new()
                .children(1, [2, 3])
                .children(2, [])
                .children(3, [])
                .style(
                    1,
                    NodeInputOf {
                        display: Display::GridLanes,
                        writing_mode: WritingMode::HorizontalTb,
                        direction: Direction::Ltr,
                        grid_lanes_flow: GridLanesFlow::new(GridAxisKind::Row, false, false),
                        size: Size::new(px(100.0), px(20.0)).map(PreferredSizeOf::px),
                        grid_template_rows: vec![TrackComponentOf::px(px(10.0)); 2],
                        grid_flow_tolerance: GridFlowToleranceOf::Length(LengthOf::ZERO),
                        gap: Size::splat(LengthOf::ZERO),
                        justify_content: Some(AlignContent::Start),
                        align_content: Some(AlignContent::Start),
                        ..NodeInputOf::default()
                    },
                )
                .style(
                    2,
                    NodeInputOf {
                        display: Display::Block,
                        writing_mode,
                        direction: Direction::Ltr,
                        grid_row: GridPlacement::try_line(1).unwrap(),
                        size: Size::splat(px(10.0)).map(PreferredSizeOf::px),
                        justify_self: Some(alignment),
                        align_self: Some(AlignItems::Start),
                        margin: Edges::all(LengthAutoOf::ZERO),
                        ..NodeInputOf::default()
                    },
                )
                .style(
                    3,
                    NodeInputOf {
                        display: Display::Block,
                        writing_mode: WritingMode::HorizontalTb,
                        direction: Direction::Ltr,
                        grid_row: GridPlacement::try_line(2).unwrap(),
                        size: Size::new(px(30.0), px(10.0)).map(PreferredSizeOf::px),
                        justify_self: Some(AlignItems::Start),
                        align_self: Some(AlignItems::Start),
                        margin: Edges::all(LengthAutoOf::ZERO),
                        ..NodeInputOf::default()
                    },
                );
            let batch = match compute_layout(
                &tree,
                1,
                super::super::fixtures::fri08_c01_placement_request(),
            ) {
                Ok(batch) => batch,
                Err(error) => {
                    differences.push(format!("{writing_mode:?} {alignment:?}: {error:?}"));
                    continue;
                }
            };
            // Pinned Align 3:685/704 specifies safe self-start/self-end for
            // first/last baseline self-alignment fallback. The packed cell is
            // physical x=0..30, leaving 30-10 = 20 of positive slack. The
            // VerticalRl item's own block start/end are Right/Left, giving
            // first x=20 and last x=0. VerticalLr's Left/Right gives 0 and 20.
            // The peer's 30-unit packed extent and position are unaffected.
            for (node, location, size) in [
                (1, Point::ZERO, Size::new(px(100.0), px(20.0))),
                (2, Point::new(px(short_x), S::ZERO), Size::splat(px(10.0))),
                (
                    3,
                    Point::new(S::ZERO, px(10.0)),
                    Size::new(px(30.0), px(10.0)),
                ),
            ] {
                let output = batch
                    .unrounded_entries()
                    .iter()
                    .find(|entry| entry.node() == node)
                    .expect("native public layout publishes the requested unrounded box")
                    .output();
                let actual = (
                    output.location,
                    output.size,
                    output.margin,
                    output.padding,
                    output.border,
                );
                let expected = (location, size, Edges::ZERO, Edges::ZERO, Edges::ZERO);
                if actual != expected {
                    differences.push(format!("{writing_mode:?} {alignment:?}, unrounded node {node}: {actual:?}, expected {expected:?}"));
                }
            }
        }
    }
    assert!(differences.is_empty(), "{}", differences.join("\n"));
}

#[test]
fn stacking_baseline_fallback_uses_item_own_edge_f32() {
    assert_stacking_baseline_fallback_uses_item_own_edge::<f32>();
}

#[test]
fn stacking_baseline_fallback_uses_item_own_edge_f64() {
    assert_stacking_baseline_fallback_uses_item_own_edge::<f64>();
}
