use super::*;

fn subgrid<S: LayoutScalar>() -> Vec<TrackComponentOf<S>> {
    vec![TrackComponentOf::Subgrid(SubgridTrack::new(Vec::new()))]
}

fn tracks<S: LayoutScalar>() -> Vec<TrackComponentOf<S>> {
    [20.0, 30.0]
        .map(|size| TrackComponentOf::px(S::from_f64(size)))
        .into()
}

fn parent<S: LayoutScalar>() -> NodeInputOf<S> {
    NodeInputOf {
        display: Display::GridLanes,
        grid_lanes_flow: GridLanesFlow::new(GridAxisKind::Column, false, false),
        size: Size::new(
            PreferredSizeOf::px(S::from_f64(50.0)),
            PreferredSizeOf::AUTO,
        ),
        grid_template_columns: tracks(),
        justify_content: Some(AlignContent::Start),
        align_content: Some(AlignContent::Start),
        ..NodeInputOf::default()
    }
}

fn three_leaves<S: LayoutScalar>(
    parent: NodeInputOf<S>,
    child: NodeInputOf<S>,
) -> PublicLayoutTreeOf<S> {
    let mut tree = PublicLayoutTreeOf::new()
        .children(1, [2])
        .children(2, [3, 4, 5])
        .style(1, parent)
        .style(2, child);
    for (node, height) in [(3, 5.0), (4, 7.0), (5, 9.0)] {
        tree = tree.children(node, []).style(
            node,
            NodeInputOf {
                size: Size::new(S::from_f64(20.0), S::from_f64(height)).map(PreferredSizeOf::px),
                ..NodeInputOf::default()
            },
        );
    }
    tree
}

fn assert_grid_child_adopts_pair_for_either_request<S: LayoutScalar>() {
    let px = S::from_f64;
    let mut differences = Vec::new();
    for (columns, rows) in [(false, false), (true, false), (false, true), (true, true)] {
        let child = NodeInputOf {
            display: Display::Grid,
            grid_column: GridPlacement::try_lines(1, 3).unwrap(),
            grid_template_columns: if columns { subgrid() } else { tracks() },
            grid_template_rows: if rows { subgrid() } else { Vec::new() },
            grid_flow_tolerance: GridFlowToleranceOf::Length(LengthOf::ZERO),
            justify_content: Some(AlignContent::Start),
            align_content: Some(AlignContent::Start),
            ..NodeInputOf::default()
        };
        let tree = three_leaves(parent(), child);
        let batch = compute_layout(
            &tree,
            1,
            super::super::fixtures::fri08_c01_placement_request(),
        )
        .unwrap();
        let output = |node| fri08_c01_placement_output(&batch, node);
        let actual = (
            output(3).location,
            output(4).location,
            output(5).location,
            output(2).size,
        );
        // With either request the third leaf uses the shorter, 5-high lane.
        // Without a request, ordinary Grid's first row is max(5, 7) = 7 high.
        let adopted = columns || rows;
        let expected = (
            Point::ZERO,
            Point::new(px(20.0), S::ZERO),
            Point::new(S::ZERO, px(if adopted { 5.0 } else { 7.0 })),
            Size::new(px(50.0), px(if adopted { 14.0 } else { 16.0 })),
        );
        if actual != expected {
            differences.push(format!(
                "columns {columns}, rows {rows}: {actual:?}, expected {expected:?}"
            ));
        }
    }
    assert!(differences.is_empty(), "{}", differences.join("\n"));
}

#[test]
fn grid_child_adopts_inherited_axis_pair_for_either_request_f32() {
    assert_grid_child_adopts_pair_for_either_request::<f32>();
}

#[test]
fn grid_child_adopts_inherited_axis_pair_for_either_request_f64() {
    assert_grid_child_adopts_pair_for_either_request::<f64>();
}

fn assert_conflicting_lanes_child_adopts_pair_before_masking<S: LayoutScalar>() {
    let px = S::from_f64;
    let mut differences = Vec::new();
    for (columns, rows, correct_pair) in [
        (true, false, true),
        (true, false, false),
        (false, true, false),
        (true, true, false),
    ] {
        let mut tree = PublicLayoutTreeOf::new()
            .children(1, [2])
            .children(2, [3, 4])
            .style(1, parent())
            .style(
                2,
                NodeInputOf {
                    display: Display::GridLanes,
                    grid_lanes_flow: GridLanesFlow::new(
                        if correct_pair {
                            GridAxisKind::Column
                        } else {
                            GridAxisKind::Row
                        },
                        false,
                        false,
                    ),
                    grid_column: GridPlacement::try_lines(1, 3).unwrap(),
                    grid_template_columns: if columns { subgrid() } else { tracks() },
                    grid_template_rows: if rows {
                        subgrid()
                    } else {
                        vec![TrackComponentOf::px(px(80.0))]
                    },
                    justify_content: Some(AlignContent::Start),
                    align_content: Some(AlignContent::Start),
                    ..NodeInputOf::default()
                },
            );
        for (node, height) in [(3, 5.0), (4, 7.0)] {
            tree = tree.children(node, []).style(
                node,
                NodeInputOf {
                    grid_column: GridPlacement::try_line(2).unwrap(),
                    size: Size::new(PreferredSizeOf::AUTO, PreferredSizeOf::px(px(height))),
                    ..NodeInputOf::default()
                },
            );
        }
        let batch = match compute_layout(
            &tree,
            1,
            super::super::fixtures::fri08_c01_placement_request(),
        ) {
            Ok(batch) => batch,
            Err(error) => {
                differences.push(format!(
                    "columns {columns}, rows {rows}, correct pair {correct_pair}: {error:?}"
                ));
                continue;
            }
        };
        let first = fri08_c01_placement_output(&batch, 3);
        let second = fri08_c01_placement_output(&batch, 4);
        // Column 2 starts after the 20-unit first track and is 30 units wide.
        // Both explicit placements stack in that track, so the second starts at 5.
        let actual = (first.location, first.size, second.location, second.size);
        let expected = (
            Point::new(px(20.0), S::ZERO),
            Size::new(px(30.0), px(5.0)),
            Point::new(px(20.0), px(5.0)),
            Size::new(px(30.0), px(7.0)),
        );
        if actual != expected {
            differences.push(format!(
                "columns {columns}, rows {rows}, correct pair {correct_pair}: {actual:?}, expected {expected:?}"
            ));
        }
    }
    assert!(differences.is_empty(), "{}", differences.join("\n"));
}

#[test]
fn conflicting_lanes_child_adopts_inherited_axis_pair_before_masking_f32() {
    assert_conflicting_lanes_child_adopts_pair_before_masking::<f32>();
}

#[test]
fn conflicting_lanes_child_adopts_inherited_axis_pair_before_masking_f64() {
    assert_conflicting_lanes_child_adopts_pair_before_masking::<f64>();
}

fn assert_ordinary_parent_does_not_share_child_stacking_tracks<S: LayoutScalar>() {
    let px = S::from_f64;
    let mut differences = Vec::new();
    for requested in [false, true] {
        let tree = three_leaves(
            NodeInputOf {
                display: Display::Grid,
                size: Size::new(px(50.0), px(100.0)).map(PreferredSizeOf::px),
                grid_template_columns: tracks(),
                grid_template_rows: [40.0, 60.0]
                    .map(|size| TrackComponentOf::px(px(size)))
                    .into(),
                ..NodeInputOf::default()
            },
            NodeInputOf {
                display: Display::GridLanes,
                grid_lanes_flow: GridLanesFlow::new(GridAxisKind::Column, false, false),
                grid_column: GridPlacement::try_lines(1, 3).unwrap(),
                grid_row: GridPlacement::try_lines(1, 3).unwrap(),
                grid_template_columns: tracks(),
                grid_template_rows: if requested { subgrid() } else { Vec::new() },
                grid_flow_tolerance: GridFlowToleranceOf::Length(LengthOf::ZERO),
                align_self: Some(AlignItems::Start),
                justify_content: Some(AlignContent::Start),
                align_content: Some(AlignContent::Start),
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
                differences.push(format!("stacking requested {requested}: {error:?}"));
                continue;
            }
        };
        // The parent's two rows supply a 100-unit area, but a stacking request
        // supplies no inherited row census. The lanes remain 5+9 and 7 high.
        let actual = fri08_c01_placement_output(&batch, 2).size;
        let expected = Size::new(px(50.0), px(14.0));
        if actual != expected {
            differences.push(format!(
                "stacking requested {requested}: size {actual:?}, expected {expected:?}"
            ));
        }
        for (node, x, y) in [(3, 0.0, 0.0), (4, 20.0, 0.0), (5, 0.0, 5.0)] {
            let actual = fri08_c01_placement_output(&batch, node).location;
            let expected = Point::new(px(x), px(y));
            if actual != expected {
                differences.push(format!(
                    "stacking requested {requested}, node {node}: {actual:?}, expected {expected:?}"
                ));
            }
        }
    }
    assert!(differences.is_empty(), "{}", differences.join("\n"));
}

#[test]
fn ordinary_parent_preserves_inherited_axis_pair_without_stacking_tracks_f32() {
    assert_ordinary_parent_does_not_share_child_stacking_tracks::<f32>();
}

#[test]
fn ordinary_parent_preserves_inherited_axis_pair_without_stacking_tracks_f64() {
    assert_ordinary_parent_does_not_share_child_stacking_tracks::<f64>();
}

fn assert_orthogonal_pair_maps_grid_tracks_and_reversal<S: LayoutScalar>() {
    let px = S::from_f64;
    let mut differences = Vec::new();
    for correct_pair in [true, false] {
        let mut tree = PublicLayoutTreeOf::new()
            .children(1, [2])
            .children(2, [3, 4])
            .style(1, parent())
            .style(
                2,
                NodeInputOf {
                    display: Display::GridLanes,
                    writing_mode: WritingMode::VerticalRl,
                    grid_lanes_flow: GridLanesFlow::new(
                        if correct_pair {
                            GridAxisKind::Row
                        } else {
                            GridAxisKind::Column
                        },
                        false,
                        false,
                    ),
                    grid_column: GridPlacement::try_lines(1, 3).unwrap(),
                    size: Size::new(PreferredSizeOf::AUTO, PreferredSizeOf::px(px(20.0))),
                    grid_template_columns: if correct_pair { Vec::new() } else { subgrid() },
                    grid_template_rows: subgrid(),
                    justify_content: Some(AlignContent::Start),
                    align_content: Some(AlignContent::Start),
                    ..NodeInputOf::default()
                },
            );
        for (node, row, height) in [(3, 1, 5.0), (4, 2, 7.0)] {
            tree = tree.children(node, []).style(
                node,
                NodeInputOf {
                    grid_row: GridPlacement::try_line(row).unwrap(),
                    size: Size::new(PreferredSizeOf::AUTO, PreferredSizeOf::px(px(height))),
                    ..NodeInputOf::default()
                },
            );
        }
        let batch = match compute_layout(
            &tree,
            1,
            super::super::fixtures::fri08_c01_placement_request(),
        ) {
            Ok(batch) => batch,
            Err(error) => {
                differences.push(format!("correct pair {correct_pair}: {error:?}"));
                continue;
            }
        };
        // The parent's horizontal grid dimension becomes the vertical-rl child's
        // Row axis. Its first row is the right-hand 30-unit track; the next is 20.
        for (node, x, width, height) in [(3, 20.0, 30.0, 5.0), (4, 0.0, 20.0, 7.0)] {
            let output = fri08_c01_placement_output(&batch, node);
            let actual = (output.location, output.size);
            let expected = (Point::new(px(x), S::ZERO), Size::new(px(width), px(height)));
            if actual != expected {
                differences.push(format!(
                    "correct pair {correct_pair}, node {node}: {actual:?}, expected {expected:?}"
                ));
            }
        }
    }
    assert!(differences.is_empty(), "{}", differences.join("\n"));
}

#[test]
fn orthogonal_inherited_axis_pair_maps_tracks_and_writing_mode_reversal_f32() {
    assert_orthogonal_pair_maps_grid_tracks_and_reversal::<f32>();
}

#[test]
fn orthogonal_inherited_axis_pair_maps_tracks_and_writing_mode_reversal_f64() {
    assert_orthogonal_pair_maps_grid_tracks_and_reversal::<f64>();
}

fn assert_adopted_pair_keeps_child_reverse_dense<S: LayoutScalar>() {
    let px = S::from_f64;
    let mut differences = Vec::new();
    for correct_pair in [true, false] {
        for dense in [false, true] {
            let tree = PublicLayoutTreeOf::new()
                .children(1, [2])
                .children(2, [3, 4, 5])
                .style(
                    1,
                    NodeInputOf {
                        grid_template_columns: vec![TrackComponentOf::px(px(10.0)); 3],
                        size: Size::new(PreferredSizeOf::px(px(30.0)), PreferredSizeOf::AUTO),
                        ..parent()
                    },
                )
                .style(
                    2,
                    NodeInputOf {
                        display: Display::GridLanes,
                        grid_lanes_flow: GridLanesFlow::new(
                            if correct_pair {
                                GridAxisKind::Column
                            } else {
                                GridAxisKind::Row
                            },
                            true,
                            dense,
                        ),
                        grid_column: GridPlacement::try_lines(1, 4).unwrap(),
                        grid_template_columns: subgrid(),
                        grid_template_rows: if correct_pair { Vec::new() } else { subgrid() },
                        grid_flow_tolerance: GridFlowToleranceOf::Length(LengthOf::ZERO),
                        justify_content: Some(AlignContent::Start),
                        align_content: Some(AlignContent::Start),
                        ..NodeInputOf::default()
                    },
                )
                .style(
                    3,
                    NodeInputOf {
                        grid_column: GridPlacement::try_line(3).unwrap(),
                        size: Size::new(PreferredSizeOf::AUTO, PreferredSizeOf::px(px(40.0))),
                        ..NodeInputOf::default()
                    },
                )
                .style(
                    4,
                    NodeInputOf {
                        grid_column: GridPlacement::try_lines(2, 4).unwrap(),
                        size: Size::new(PreferredSizeOf::AUTO, PreferredSizeOf::px(px(10.0))),
                        ..NodeInputOf::default()
                    },
                )
                .style(
                    5,
                    NodeInputOf {
                        size: Size::new(PreferredSizeOf::AUTO, PreferredSizeOf::px(px(10.0))),
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
                    differences.push(format!(
                        "correct pair {correct_pair}, dense {dense}: {error:?}"
                    ));
                    continue;
                }
            };
            // Track 3's 40-high item lifts the bridge over tracks 2 and 3.
            // Child reverse+dense chooses the opening in track 2 before empty track 1;
            // sparse placement chooses the latter. Parent flags are both false.
            for (node, x, y) in [
                (3, 20.0, 0.0),
                (4, 10.0, 40.0),
                (5, if dense { 10.0 } else { 0.0 }, 0.0),
            ] {
                let actual = fri08_c01_placement_output(&batch, node).location;
                let expected = Point::new(px(x), px(y));
                if actual != expected {
                    differences.push(format!("correct pair {correct_pair}, dense {dense}, node {node}: {actual:?}, expected {expected:?}"));
                }
            }
        }
    }
    assert!(differences.is_empty(), "{}", differences.join("\n"));
}

#[test]
fn adopted_inherited_axis_pair_keeps_child_reverse_dense_f32() {
    assert_adopted_pair_keeps_child_reverse_dense::<f32>();
}

#[test]
fn adopted_inherited_axis_pair_keeps_child_reverse_dense_f64() {
    assert_adopted_pair_keeps_child_reverse_dense::<f64>();
}
