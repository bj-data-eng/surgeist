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

fn assert_nested_stacking_request_preserves_pair<S: LayoutScalar>() {
    let px = S::from_f64;
    let tree = three_leaves(
        NodeInputOf {
            display: Display::Grid,
            grid_column: GridPlacement::try_lines(1, 3).unwrap(),
            grid_template_columns: tracks(),
            grid_template_rows: subgrid(),
            justify_content: Some(AlignContent::Start),
            align_content: Some(AlignContent::Start),
            ..NodeInputOf::default()
        },
        NodeInputOf {
            display: Display::GridLanes,
            grid_lanes_flow: GridLanesFlow::new(GridAxisKind::Row, false, false),
            grid_column: GridPlacement::try_lines(1, 3).unwrap(),
            grid_template_columns: subgrid(),
            grid_template_rows: subgrid(),
            grid_flow_tolerance: GridFlowToleranceOf::Length(LengthOf::ZERO),
            justify_content: Some(AlignContent::Start),
            align_content: Some(AlignContent::Start),
            ..NodeInputOf::default()
        },
    )
    // The original three-leaf fixture is now rooted at an ordinary Grid
    // wrapper. That wrapper first adopts the outer Lanes pair from rows only.
    .children(0, [1])
    .style(0, parent());
    let batch = compute_layout(
        &tree,
        0,
        super::super::fixtures::fri08_c01_placement_request(),
    )
    .unwrap();
    // The nested wrapper then shares the first wrapper's 20/30 columns.
    // Lane heights are 5+9 and 7, so each wrapper is max(14, 7) = 14 high.
    for node in [1, 2] {
        let output = fri08_c01_placement_output(&batch, node);
        assert_eq!(
            (output.location, output.size),
            (Point::ZERO, Size::new(px(50.0), px(14.0))),
            "wrapper {node}"
        );
    }
    for (node, x, y) in [(3, 0.0, 0.0), (4, 20.0, 0.0), (5, 0.0, 5.0)] {
        assert_eq!(
            fri08_c01_placement_output(&batch, node).location,
            Point::new(px(x), px(y)),
            "leaf {node}"
        );
    }
}

#[test]
fn nested_stacking_request_preserves_inherited_axis_pair_f32() {
    assert_nested_stacking_request_preserves_pair::<f32>();
}

#[test]
fn nested_stacking_request_preserves_inherited_axis_pair_f64() {
    assert_nested_stacking_request_preserves_pair::<f64>();
}

fn intrinsic_pair_request<S: LayoutScalar>(operation: SizingOperation) -> LayoutRootRequestOf<S> {
    LayoutRootRequestOf::standalone(
        SizingRequestOf::new(
            Size::splat(AxisSizingRequestOf::new(
                AvailableSpaceOf::Indefinite,
                operation,
            )),
            OrthogonalFallbackOf::new(
                Size::NONE,
                crate::NearestScrollportOf::Absent,
                Size::splat(crate::OrthogonalFallbackLimitOf::Unbounded),
            ),
        ),
        FlowAxes::new(WritingMode::HorizontalTb, Direction::Ltr),
        Size::NONE,
    )
}

fn assert_intrinsic_request_preserves_pair<S: LayoutScalar>() {
    let px = S::from_f64;
    let mut differences = Vec::new();
    for operation in [SizingOperation::MinContent, SizingOperation::MaxContent] {
        let tree = three_leaves(
            NodeInputOf {
                display: Display::InlineGridLanes,
                size: Size::splat_clone(PreferredSizeOf::AUTO),
                ..parent()
            },
            NodeInputOf {
                display: Display::GridLanes,
                grid_lanes_flow: GridLanesFlow::new(GridAxisKind::Row, false, false),
                grid_column: GridPlacement::try_lines(1, 3).unwrap(),
                grid_template_columns: subgrid(),
                grid_template_rows: subgrid(),
                size: Size::new(
                    PreferredSizeOf::AUTO,
                    if operation == SizingOperation::MinContent {
                        PreferredSizeOf::MIN_CONTENT
                    } else {
                        PreferredSizeOf::MAX_CONTENT
                    },
                ),
                grid_flow_tolerance: GridFlowToleranceOf::Length(LengthOf::ZERO),
                justify_content: Some(AlignContent::Start),
                align_content: Some(AlignContent::Start),
                ..NodeInputOf::default()
            },
        );
        let batch = match compute_layout(&tree, 1, intrinsic_pair_request(operation)) {
            Ok(batch) => batch,
            Err(error) => {
                differences.push(format!("{operation:?}: {error:?}"));
                continue;
            }
        };
        // Fixed tracks sum to 50 under both intrinsic operations. Fixed leaf
        // heights make both stacking contributions max(5+9, 7) = 14.
        for node in [1, 2] {
            let actual = fri08_c01_placement_output(&batch, node).size;
            let expected = Size::new(px(50.0), px(14.0));
            if actual != expected {
                differences.push(format!(
                    "{operation:?}, node {node}: {actual:?}, expected {expected:?}"
                ));
            }
        }
        for (node, x, y) in [(3, 0.0, 0.0), (4, 20.0, 0.0), (5, 0.0, 5.0)] {
            let actual = fri08_c01_placement_output(&batch, node).location;
            let expected = Point::new(px(x), px(y));
            if actual != expected {
                differences.push(format!(
                    "{operation:?}, leaf {node}: {actual:?}, expected {expected:?}"
                ));
            }
        }
    }
    assert!(differences.is_empty(), "{}", differences.join("\n"));
}

#[test]
fn intrinsic_requests_preserve_inherited_axis_pair_f32() {
    assert_intrinsic_request_preserves_pair::<f32>();
}

#[test]
fn intrinsic_requests_preserve_inherited_axis_pair_f64() {
    assert_intrinsic_request_preserves_pair::<f64>();
}

fn assert_auto_scrollbar_retry_preserves_pair<S: LayoutScalar>() {
    let px = S::from_f64;
    // Only the authored pair differs: both cases keep both raw requests.
    // Column is the nearest already-correct control for adopted Row.
    for grid_axis in [GridAxisKind::Column, GridAxisKind::Row] {
        let tree = PublicLayoutTreeOf::new()
            .children(1, [2])
            .children(2, [3, 4])
            .children(3, [])
            .children(4, [])
            .style(
                1,
                NodeInputOf {
                    size: Size::splat_clone(PreferredSizeOf::px(px(100.0))),
                    grid_template_columns: vec![TrackComponentOf::px(px(100.0))],
                    ..parent()
                },
            )
            .style(
                2,
                NodeInputOf {
                    display: Display::GridLanes,
                    grid_lanes_flow: GridLanesFlow::new(grid_axis, false, false),
                    size: Size::splat_clone(PreferredSizeOf::px(px(100.0))),
                    grid_template_columns: subgrid(),
                    grid_template_rows: subgrid(),
                    overflow: computed_overflow(Overflow::Auto, Overflow::Auto),
                    scrollbar_width: ScrollbarWidthOf::try_new(px(10.0)).unwrap(),
                    justify_content: Some(AlignContent::Start),
                    align_content: Some(AlignContent::Start),
                    ..NodeInputOf::default()
                },
            )
            .style(
                3,
                NodeInputOf {
                    size: Size::new(px(95.0), px(120.0)).map(PreferredSizeOf::px),
                    ..NodeInputOf::default()
                },
            )
            .style(
                4,
                NodeInputOf {
                    size: Size::new(PreferredSizeOf::AUTO, PreferredSizeOf::px(px(20.0))),
                    ..NodeInputOf::default()
                },
            );
        let batch = compute_layout(
            &tree,
            1,
            super::super::fixtures::fri08_c01_placement_request(),
        )
        .unwrap();
        let child = fri08_c01_placement_output(&batch, 2);
        let first = fri08_c01_placement_output(&batch, 3);
        let second = fri08_c01_placement_output(&batch, 4);
        assert_eq!(
            (child.location, child.size),
            (Point::ZERO, Size::splat(px(100.0)))
        );
        assert_eq!(
            (first.location, first.size),
            (Point::ZERO, Size::new(px(95.0), px(120.0)))
        );
        // Vertical overflow first reserves 10 on the right. The resulting 90-wide
        // scrollport is narrower than 95, so Auto must also reserve 10 below.
        // The parent track stays 100, but the subgrid's edge gutter reduces its
        // inherited used track and cell availability to 100-10 = 90. The next
        // item stacks at 120 (Grid 2's pinned subgrid-margins rule).
        assert_eq!(
            (second.location, second.size),
            (
                Point::new(S::ZERO, px(120.0)),
                Size::new(px(90.0), px(20.0))
            )
        );
        let geometry = child
            .scroll_geometry
            .expect("performed adopted child publishes qualified scroll geometry");
        assert_eq!(
            (geometry.used_overflow_x(), geometry.used_overflow_y()),
            (Overflow::Auto, Overflow::Auto)
        );
        assert_eq!(geometry.border_box().size(), Size::splat(px(100.0)));
        assert_eq!(geometry.scrollbar_size(), Size::splat(px(10.0)));
        assert_eq!(geometry.content_box().size(), Size::splat(px(90.0)));
        assert_eq!(geometry.scrollport().size(), Size::splat(px(90.0)));
        assert!(geometry.gutters().right().is_some());
        assert!(geometry.gutters().bottom().is_some());
        assert_eq!(
            geometry.scrollable_overflow().size(),
            Size::new(px(100.0), px(140.0)),
            "authored grid axis {grid_axis:?}"
        );
        let range = geometry.physical_range();
        // Complete overflow includes the 100-wide padding-box seed. Scroll
        // range excludes the reserved gutter by using the 90-wide scrollport
        // seed instead: max(used grid 90, leaf 95) - 90 = 5. Vertically the
        // content ends at 120+20 = 140, giving 140-90 = 50. The published
        // overflow rectangle therefore does not itself determine scroll reach.
        assert_eq!(
            (range.x().minimum(), range.x().maximum()),
            (S::ZERO, px(5.0)),
            "authored grid axis {grid_axis:?}"
        );
        assert_eq!(
            (range.y().minimum(), range.y().maximum()),
            (S::ZERO, px(50.0))
        );
    }
}

#[test]
fn auto_scrollbar_retry_preserves_inherited_axis_pair_f32() {
    assert_auto_scrollbar_retry_preserves_pair::<f32>();
}

#[test]
fn auto_scrollbar_retry_preserves_inherited_axis_pair_f64() {
    assert_auto_scrollbar_retry_preserves_pair::<f64>();
}

fn assert_empty_and_inline_contained_preserve_pair<S: LayoutScalar>() {
    let px = S::from_f64;
    let mut differences = Vec::new();
    for contained in [false, true] {
        let mut tree = PublicLayoutTreeOf::new()
            .children(1, [2])
            .children(2, if contained { vec![3, 4] } else { Vec::new() })
            .style(1, parent())
            .style(
                2,
                NodeInputOf {
                    display: Display::GridLanes,
                    grid_lanes_flow: GridLanesFlow::new(GridAxisKind::Row, false, false),
                    grid_column: GridPlacement::try_lines(1, 3).unwrap(),
                    grid_template_columns: subgrid(),
                    grid_template_rows: subgrid(),
                    size_containment: if contained {
                        SizeContainment::InlineAxis
                    } else {
                        SizeContainment::None
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
                differences.push(format!("inline contained {contained}: {error:?}"));
                continue;
            }
        };
        // Empty stacking demand is zero. Inline containment suppresses no
        // block demand and does not replace the inherited fixed 20/30 tracks.
        let actual = fri08_c01_placement_output(&batch, 2).size;
        let expected = Size::new(px(50.0), px(if contained { 5.0 + 7.0 } else { 0.0 }));
        if actual != expected {
            differences.push(format!(
                "inline contained {contained}: {actual:?}, expected {expected:?}"
            ));
        }
        if contained {
            for (node, y, height) in [(3, 0.0, 5.0), (4, 5.0, 7.0)] {
                let output = fri08_c01_placement_output(&batch, node);
                let actual = (output.location, output.size);
                let expected = (Point::new(px(20.0), px(y)), Size::new(px(30.0), px(height)));
                if actual != expected {
                    differences.push(format!(
                        "contained leaf {node}: {actual:?}, expected {expected:?}"
                    ));
                }
            }
        }
    }
    assert!(differences.is_empty(), "{}", differences.join("\n"));
}

#[test]
fn empty_and_inline_contained_preserve_inherited_axis_pair_f32() {
    assert_empty_and_inline_contained_preserve_pair::<f32>();
}

#[test]
fn empty_and_inline_contained_preserve_inherited_axis_pair_f64() {
    assert_empty_and_inline_contained_preserve_pair::<f64>();
}

fn assert_stacking_request_preserves_host_leaf_measurement<S: LayoutScalar>() {
    let px = S::from_f64;
    let mut differences = Vec::new();
    for display in [Display::Grid, Display::GridLanes] {
        for (columns, rows) in [(false, false), (true, false), (false, true), (true, true)] {
            let tree = PublicLayoutTreeOf::new()
                .children(1, [2])
                .children(2, [])
                .style(
                    1,
                    NodeInputOf {
                        grid_template_columns: vec![TrackComponentOf::px(px(50.0))],
                        ..parent()
                    },
                )
                .style(
                    2,
                    NodeInputOf {
                        display,
                        grid_lanes_flow: GridLanesFlow::new(GridAxisKind::Row, false, false),
                        size: Size::new(px(20.0), px(10.0)).map(PreferredSizeOf::px),
                        justify_self: Some(AlignItems::Start),
                        grid_template_columns: if columns { subgrid() } else { Vec::new() },
                        grid_template_rows: if rows { subgrid() } else { Vec::new() },
                        ..NodeInputOf::default()
                    },
                )
                .measure(2, Size::new(px(40.0), px(30.0)));
            let batch = match compute_layout(
                &tree,
                1,
                super::super::fixtures::fri08_c01_placement_request(),
            ) {
                Ok(batch) => batch,
                Err(error) => {
                    differences.push(format!(
                        "{display:?}, columns {columns}, rows {rows}: {error:?}"
                    ));
                    continue;
                }
            };
            // Performed leaf layout retains the host's actual content and
            // scroll facts even when both outer dimensions are known
            // (docs/reference.md, the compute_leaf contract). With no edges,
            // the fixed box is 20x10 and measured content/overflow are 40x30.
            // Start alignment avoids stretching to the parent's 50-unit track;
            // the sole 10-high outer box supplies the parent's stacking demand.
            let child = fri08_c01_placement_output(&batch, 2);
            let actual = (
                child.location,
                child.size,
                child.content_size,
                child
                    .scroll_geometry
                    .map(|geometry| geometry.scrollable_overflow().size()),
                fri08_c01_placement_output(&batch, 1).size,
            );
            let expected = (
                Point::ZERO,
                Size::new(px(20.0), px(10.0)),
                Size::new(px(40.0), px(30.0)),
                Some(Size::new(px(40.0), px(30.0))),
                Size::new(px(50.0), px(10.0)),
            );
            if actual != expected {
                differences.push(format!(
                    "{display:?}, columns {columns}, rows {rows}: {actual:?}, expected {expected:?}"
                ));
            }
        }
    }
    assert!(differences.is_empty(), "{}", differences.join("\n"));
}

#[test]
fn stacking_request_preserves_host_leaf_measurement_f32() {
    assert_stacking_request_preserves_host_leaf_measurement::<f32>();
}

#[test]
fn stacking_request_preserves_host_leaf_measurement_f64() {
    assert_stacking_request_preserves_host_leaf_measurement::<f64>();
}
