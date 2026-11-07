use super::*;

fn assert_threshold_placement<S: LayoutScalar>(
    grid_axis: GridAxisKind,
    tolerance: GridFlowToleranceOf<S>,
    expected_automatic_location: Point<S>,
) {
    let px = S::from_f64;
    let columns = grid_axis == GridAxisKind::Column;
    let physical_size = if columns {
        Size::new(px(80.0), px(100.0))
    } else {
        Size::new(px(100.0), px(80.0))
    };
    let leaf = |stacking_size| NodeInputOf {
        size: if columns {
            Size::new(
                PreferredSizeOf::AUTO,
                PreferredSizeOf::px(px(stacking_size)),
            )
        } else {
            Size::new(
                PreferredSizeOf::px(px(stacking_size)),
                PreferredSizeOf::AUTO,
            )
        },
        ..NodeInputOf::default()
    };
    let tree = PublicLayoutTreeOf::new()
        .children(1, [2, 3])
        .children(2, [])
        .children(3, [])
        .style(
            1,
            NodeInputOf {
                display: Display::GridLanes,
                grid_lanes_flow: GridLanesFlow::new(grid_axis, false, false),
                size: physical_size.map(PreferredSizeOf::px),
                grid_template_columns: if columns {
                    vec![TrackComponentOf::px(px(40.0)); 2]
                } else {
                    Vec::new()
                },
                grid_template_rows: if columns {
                    Vec::new()
                } else {
                    vec![TrackComponentOf::px(px(40.0)); 2]
                },
                grid_flow_tolerance: tolerance,
                justify_content: Some(AlignContent::Start),
                align_content: Some(AlignContent::Start),
                ..NodeInputOf::default()
            },
        )
        .style(
            2,
            NodeInputOf {
                grid_column: if columns {
                    GridPlacement::try_line(1).unwrap()
                } else {
                    GridPlacement::AUTO
                },
                grid_row: if columns {
                    GridPlacement::AUTO
                } else {
                    GridPlacement::try_line(1).unwrap()
                },
                ..leaf(22.0)
            },
        )
        .style(3, leaf(5.0));
    let batch = compute_layout(
        &tree,
        1,
        super::super::fixtures::fri08_c01_placement_request(),
    )
    .unwrap();
    assert_eq!(fri08_c01_placement_output(&batch, 1).size, physical_size);
    let first = fri08_c01_placement_output(&batch, 2);
    let automatic = fri08_c01_placement_output(&batch, 3);
    assert_eq!(
        (first.location, first.size),
        (
            Point::ZERO,
            if columns {
                Size::new(px(40.0), px(22.0))
            } else {
                Size::new(px(22.0), px(40.0))
            }
        )
    );
    assert_eq!(
        (automatic.location, automatic.size),
        (
            expected_automatic_location,
            if columns {
                Size::new(px(40.0), px(5.0))
            } else {
                Size::new(px(5.0), px(40.0))
            }
        ),
        "{grid_axis:?}, tolerance {tolerance:?}"
    );
}

fn assert_normal_tolerance_uses_supplied_font_size<S: LayoutScalar>() {
    let px = S::from_f64;
    // Pinned Grid 3 flow-tolerance normal is 1em. The definite first-column
    // item leaves running positions [22, 0] and does not advance the cursor.
    // A 20-unit threshold excludes 22; a 24-unit threshold includes it and
    // the cursor's first column wins. Neither case uses the default font size.
    for (font_size, expected) in [
        (20.0, Point::new(px(40.0), S::ZERO)),
        (24.0, Point::new(S::ZERO, px(22.0))),
    ] {
        assert_threshold_placement(
            GridAxisKind::Column,
            GridFlowToleranceOf::Normal {
                font_size: px(font_size),
            },
            expected,
        );
    }
}

#[test]
fn normal_tolerance_uses_supplied_font_size_f32() {
    assert_normal_tolerance_uses_supplied_font_size::<f32>();
}

#[test]
fn normal_tolerance_uses_supplied_font_size_f64() {
    assert_normal_tolerance_uses_supplied_font_size::<f64>();
}

fn assert_row_percentage_uses_grid_height<S: LayoutScalar>() {
    let px = S::from_f64;
    // The pinned percentage basis is grid-axis content size: Row means the
    // 80-unit height, not the 100-unit stacking width. Thus thresholds are
    // .25*80 = 20 and .30*80 = 24 against running positions [22, 0].
    for (factor, expected) in [
        (0.25, Point::new(S::ZERO, px(40.0))),
        (0.30, Point::new(px(22.0), S::ZERO)),
    ] {
        assert_threshold_placement(
            GridAxisKind::Row,
            GridFlowToleranceOf::Percent(px(factor)),
            expected,
        );
    }
}

#[test]
fn row_percentage_tolerance_uses_grid_height_f32() {
    assert_row_percentage_uses_grid_height::<f32>();
}

#[test]
fn row_percentage_tolerance_uses_grid_height_f64() {
    assert_row_percentage_uses_grid_height::<f64>();
}

fn assert_dense_negative_margin_preserves_skipped_space<S: LayoutScalar>() {
    let px = S::from_f64;
    let leaf = |height| NodeInputOf {
        size: Size::new(PreferredSizeOf::AUTO, PreferredSizeOf::px(px(height))),
        ..NodeInputOf::default()
    };
    for dense in [false, true] {
        let tree = PublicLayoutTreeOf::new()
            .children(1, [2, 3, 4, 5])
            .children(2, [])
            .children(3, [])
            .children(4, [])
            .children(5, [])
            .style(
                1,
                NodeInputOf {
                    display: Display::GridLanes,
                    grid_lanes_flow: GridLanesFlow::new(GridAxisKind::Column, false, dense),
                    size: Size::new(px(20.0), px(100.0)).map(PreferredSizeOf::px),
                    grid_template_columns: vec![TrackComponentOf::px(px(10.0)); 2],
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
                    grid_column: GridPlacement::try_line(2).unwrap(),
                    ..leaf(40.0)
                },
            )
            .style(
                3,
                NodeInputOf {
                    grid_column: GridPlacement::try_lines(1, 3).unwrap(),
                    margin: Edges::new(px(-20.0), S::ZERO, S::ZERO, S::ZERO).map(LengthAutoOf::px),
                    ..leaf(10.0)
                },
            )
            .style(4, leaf(25.0))
            .style(5, leaf(10.0));
        let batch = compute_layout(
            &tree,
            1,
            super::super::fixtures::fri08_c01_placement_request(),
        )
        .unwrap();
        assert_eq!(
            fri08_c01_placement_output(&batch, 1).size,
            Size::new(px(20.0), px(100.0))
        );
        // The bridge is placed at running position 40; its -20 top margin
        // moves its border to 20. Its negative outer size cannot lower either
        // running position, and visual intrusion cannot consume the 0..40 gap.
        // Dense backfill consumes 0..25 then 25..35 in column 1. Sparse mode
        // starts at 40 in column 1, then chooses the shorter column 2 at 40.
        // Authority: pinned Grid 3 placement algorithm and visual-intrusion note.
        for (node, x, y, width, height) in [
            (2, 10.0, 0.0, 10.0, 40.0),
            (3, 0.0, 20.0, 20.0, 10.0),
            (4, 0.0, if dense { 0.0 } else { 40.0 }, 10.0, 25.0),
            (
                5,
                if dense { 0.0 } else { 10.0 },
                if dense { 25.0 } else { 40.0 },
                10.0,
                10.0,
            ),
        ] {
            let output = fri08_c01_placement_output(&batch, node);
            assert_eq!(
                (output.location, output.size),
                (Point::new(px(x), px(y)), Size::new(px(width), px(height))),
                "dense {dense}, node {node}"
            );
        }
    }
}

#[test]
fn dense_negative_margin_preserves_skipped_space_f32() {
    assert_dense_negative_margin_preserves_skipped_space::<f32>();
}

#[test]
fn dense_negative_margin_preserves_skipped_space_f64() {
    assert_dense_negative_margin_preserves_skipped_space::<f64>();
}
