use super::*;

#[derive(Clone, Copy, Debug)]
enum PositionedRange {
    Centered,
    Signed,
    Empty,
}

fn positioned_point<S: LayoutScalar>(axis: GridAxisKind, grid: f64, stack: f64) -> Point<S> {
    let px = S::from_f64;
    match axis {
        GridAxisKind::Column => Point::new(px(grid), px(stack)),
        GridAxisKind::Row => Point::new(px(stack), px(grid)),
    }
}

fn positioned_size<S: LayoutScalar>(axis: GridAxisKind, grid: f64, stack: f64) -> Size<S> {
    let point = positioned_point(axis, grid, stack);
    Size::new(point.x, point.y)
}

fn positioned_line_tree<S: LayoutScalar>(
    axis: GridAxisKind,
    range: PositionedRange,
    grid_reference: RawGridPlacement,
    stacking_reference: RawGridPlacement,
    percentage: bool,
    with_absolute: bool,
) -> PublicLayoutTreeOf<S> {
    let px = S::from_f64;
    let columns = axis == GridAxisKind::Column;
    let stacking_alignment = if matches!(range, PositionedRange::Centered) {
        AlignContent::Center
    } else {
        AlignContent::Start
    };
    let mut tree = PublicLayoutTreeOf::new().style(
        1,
        NodeInputOf {
            display: Display::GridLanes,
            position: Position::Relative,
            box_sizing: BoxSizing::ContentBox,
            size: positioned_size(axis, 70.0, 90.0).map(PreferredSizeOf::px),
            border: Edges::all(LengthOf::px(px(5.0))),
            padding: Edges::all(LengthOf::px(px(10.0))),
            grid_lanes_flow: GridLanesFlow::new(axis, false, false),
            grid_template_columns: if columns {
                vec![TrackComponentOf::px(px(70.0))]
            } else {
                Vec::new()
            },
            grid_template_rows: if columns {
                Vec::new()
            } else {
                vec![TrackComponentOf::px(px(70.0))]
            },
            gap: Size::splat(LengthOf::ZERO),
            grid_flow_tolerance: GridFlowToleranceOf::Length(LengthOf::ZERO),
            justify_content: Some(if columns {
                AlignContent::Start
            } else {
                stacking_alignment
            }),
            align_content: Some(if columns {
                stacking_alignment
            } else {
                AlignContent::Start
            }),
            ..NodeInputOf::default()
        },
    );
    let mut children = Vec::new();
    if with_absolute {
        // Raw references reach positioned resolution without a typed placement
        // overriding their alias identities or their original endpoint roles.
        let mut absolute = NodeInputOf {
            display: Display::Block,
            position: Position::Absolute,
            inset: Edges::all(LengthAutoOf::ZERO),
            justify_self: Some(AlignItems::Start),
            align_self: Some(AlignItems::Start),
            ..NodeInputOf::default()
        };
        // Explicit percentages make the containing block observable even with
        // self-alignment Start; Auto sizing is permitted to shrink to content.
        absolute.size =
            Size::splat(px(if percentage { 0.5 } else { 1.0 })).map(PreferredSizeOf::percent);
        if columns {
            absolute.raw_grid_column = grid_reference;
            absolute.raw_grid_row = stacking_reference;
        } else {
            absolute.raw_grid_row = grid_reference;
            absolute.raw_grid_column = stacking_reference;
        }
        tree = tree.children(2, []).style(2, absolute);
        // Put the absolute child first: it must not establish a stacking range,
        // demand tracks, advance the placement cursor or consume a packed gap.
        children.push(2);
    }
    if !matches!(range, PositionedRange::Empty) {
        let stack_size = if matches!(range, PositionedRange::Signed) {
            10.0
        } else {
            40.0
        };
        let mut item = NodeInputOf {
            display: Display::Block,
            size: positioned_size(axis, 70.0, stack_size).map(PreferredSizeOf::px),
            justify_self: Some(AlignItems::Start),
            align_self: Some(AlignItems::Start),
            ..NodeInputOf::default()
        };
        if columns {
            item.grid_column = GridPlacement::try_line(1).unwrap();
            if matches!(range, PositionedRange::Signed) {
                item.margin.top = LengthAutoOf::px(px(-20.0));
            }
        } else {
            item.grid_row = GridPlacement::try_line(1).unwrap();
            if matches!(range, PositionedRange::Signed) {
                item.margin.left = LengthAutoOf::px(px(-20.0));
            }
        }
        tree = tree.children(3, []).style(3, item);
        children.push(3);
    }
    tree.children(1, children)
}

fn check_positioned_line_tree<S: LayoutScalar>(
    tree: &PublicLayoutTreeOf<S>,
    axis: GridAxisKind,
    range: PositionedRange,
    absolute: Option<(f64, f64, f64, f64)>,
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
            differences.push(format!("{axis:?}, {range:?}, {label}: {error:?}"));
            return;
        }
    };
    let mut expected = vec![(1, 0.0, 0.0, 100.0, 120.0)];
    if let Some((grid_start, stack_start, grid_size, stack_size)) = absolute {
        expected.push((2, grid_start, stack_start, grid_size, stack_size));
    }
    match range {
        PositionedRange::Centered => expected.push((3, 15.0, 40.0, 70.0, 40.0)),
        PositionedRange::Signed => expected.push((3, 15.0, -5.0, 70.0, 10.0)),
        PositionedRange::Empty => {}
    }
    for (phase, entries) in [
        ("unrounded", batch.unrounded_entries()),
        ("final", batch.final_entries()),
    ] {
        assert_eq!(
            entries.len(),
            expected.len(),
            "{label}, {phase}: real boxes"
        );
        for &(node, grid_start, stack_start, grid_size, stack_size) in &expected {
            let output = entries
                .iter()
                .find(|entry| entry.node() == node)
                .unwrap_or_else(|| panic!("{label}, {phase}: missing node {node}"))
                .output();
            let actual = (output.location, output.size);
            let expected = (
                positioned_point(axis, grid_start, stack_start),
                positioned_size(axis, grid_size, stack_size),
            );
            if actual != expected {
                differences.push(format!(
                    "{axis:?}, {range:?}, {label}, {phase}, node {node}: {actual:?}, expected {expected:?}"
                ));
            }
        }
    }
}

fn assert_positioned_stacking_lines_preserve_aliases_and_auto_edges<S: LayoutScalar>() {
    let mut differences = Vec::new();
    for axis in [GridAxisKind::Column, GridAxisKind::Row] {
        // Jan Grid 3:1402-1410 gives the two stacking line identities. Grid 2
        // conflict handling precedes geometry: reversed identities swap, equal
        // identities drop the end. Grid 2:4849/4873 retains independent auto
        // padding edges and invalidates only nonexistent reference endpoints.
        // Content70x90 plus border5/padding10 gives outer100x120. The 40-unit
        // stacking range is centered at 15+(90-40)/2 = 40, ending at80.
        // Grid-axis lines remain15/85; auto stacking edges remain5/115.
        for (label, reference, start, extent) in [
            ("positive", RawGridPlacement::lines(1, 2), 40.0, 40.0),
            (
                "negative aliases",
                RawGridPlacement::lines(-2, -1),
                40.0,
                40.0,
            ),
            ("mixed aliases", RawGridPlacement::lines(1, -1), 40.0, 40.0),
            (
                "reversed positive",
                RawGridPlacement::lines(2, 1),
                40.0,
                40.0,
            ),
            (
                "reversed negative",
                RawGridPlacement::lines(-1, -2),
                40.0,
                40.0,
            ),
            (
                "equal start aliases",
                RawGridPlacement::lines(1, -2),
                40.0,
                75.0,
            ),
            (
                "equal end aliases",
                RawGridPlacement::lines(2, -1),
                80.0,
                35.0,
            ),
            ("start and auto", RawGridPlacement::line(1), 40.0, 75.0),
            (
                "auto and end",
                RawGridPlacement::new(RawGridLine::Auto, RawGridLine::Line(2)),
                5.0,
                75.0,
            ),
            ("invalid end", RawGridPlacement::lines(1, 100), 40.0, 75.0),
            ("invalid start", RawGridPlacement::lines(-100, 2), 5.0, 75.0),
            (
                "both invalid",
                RawGridPlacement::lines(-100, 100),
                5.0,
                110.0,
            ),
            (
                "span exceeds end",
                RawGridPlacement::new(RawGridLine::Line(1), RawGridLine::Span(2)),
                40.0,
                75.0,
            ),
            (
                "span exceeds start",
                RawGridPlacement::new(RawGridLine::Span(2), RawGridLine::Line(2)),
                5.0,
                75.0,
            ),
            ("auto edges", RawGridPlacement::AUTO, 5.0, 110.0),
        ] {
            let tree = positioned_line_tree::<S>(
                axis,
                PositionedRange::Centered,
                RawGridPlacement::lines(1, 2),
                reference,
                false,
                true,
            );
            check_positioned_line_tree(
                &tree,
                axis,
                PositionedRange::Centered,
                Some((15.0, start, 70.0, extent)),
                label,
                &mut differences,
            );
        }
        let half = positioned_line_tree::<S>(
            axis,
            PositionedRange::Centered,
            RawGridPlacement::lines(1, 2),
            RawGridPlacement::lines(-2, -1),
            true,
            true,
        );
        // Authored 50% sizes use the actual explicit containing block70x40,
        // yielding35x20. No measurement fake can ignore a supplied known size.
        check_positioned_line_tree(
            &half,
            axis,
            PositionedRange::Centered,
            Some((15.0, 40.0, 35.0, 20.0)),
            "explicit range percentage basis",
            &mut differences,
        );
    }
    assert!(differences.is_empty(), "{}", differences.join("\n"));
}

#[test]
fn positioned_stacking_lines_preserve_aliases_and_auto_edges_f32() {
    assert_positioned_stacking_lines_preserve_aliases_and_auto_edges::<f32>();
}

#[test]
fn positioned_stacking_lines_preserve_aliases_and_auto_edges_f64() {
    assert_positioned_stacking_lines_preserve_aliases_and_auto_edges::<f64>();
}

fn assert_signed_stacking_lines_keep_their_start_anchor<S: LayoutScalar>() {
    let mut differences = Vec::new();
    for axis in [GridAxisKind::Column, GridAxisKind::Row] {
        for (label, reference, start, extent) in [
            (
                "positive identities",
                RawGridPlacement::lines(1, 2),
                15.0,
                0.0,
            ),
            (
                "negative identities",
                RawGridPlacement::lines(-2, -1),
                15.0,
                0.0,
            ),
            (
                "reversed identities",
                RawGridPlacement::lines(2, 1),
                15.0,
                0.0,
            ),
            ("end and auto", RawGridPlacement::line(2), 5.0, 110.0),
        ] {
            let tree = positioned_line_tree::<S>(
                axis,
                PositionedRange::Signed,
                RawGridPlacement::lines(1, 2),
                reference,
                false,
                true,
            );
            // Packed first/last outer edges are0/-10. Content Start adds15,
            // so line1=15 and line2=5. The ordinary positioned-axis contract
            // preserves the selected start anchor and floors only end-start:
            // explicit1/2 is zero at15, not a geometric5..15 interval.
            // Line2/Auto is5..115. The negative in-flow border at-5..5 can
            // extend auto-start; this test deliberately uses only auto-end.
            check_positioned_line_tree(
                &tree,
                axis,
                PositionedRange::Signed,
                Some((15.0, start, 70.0, extent)),
                label,
                &mut differences,
            );
        }
    }
    assert!(differences.is_empty(), "{}", differences.join("\n"));
}

#[test]
fn signed_stacking_lines_keep_their_start_anchor_f32() {
    assert_signed_stacking_lines_keep_their_start_anchor::<f32>();
}

#[test]
fn signed_stacking_lines_keep_their_start_anchor_f64() {
    assert_signed_stacking_lines_keep_their_start_anchor::<f64>();
}

fn assert_empty_stacking_range_keeps_distinct_line_identities<S: LayoutScalar>() {
    let mut differences = Vec::new();
    for axis in [GridAxisKind::Column, GridAxisKind::Row] {
        for (label, reference, extent) in [
            ("distinct positive", RawGridPlacement::lines(1, 2), 0.0),
            ("distinct negative", RawGridPlacement::lines(-2, -1), 0.0),
            ("equal start aliases", RawGridPlacement::lines(1, -2), 100.0),
        ] {
            let tree = positioned_line_tree::<S>(
                axis,
                PositionedRange::Empty,
                RawGridPlacement::lines(1, 2),
                reference,
                false,
                true,
            );
            // With Content Start, the empty range is anchored at content inset15.
            // Lines1/2 remain distinct even when both coordinates are15: size0.
            // Equal aliases1/-2 instead drop the end, exposing Auto end115.
            check_positioned_line_tree(
                &tree,
                axis,
                PositionedRange::Empty,
                Some((15.0, 15.0, 70.0, extent)),
                label,
                &mut differences,
            );
        }
    }
    assert!(differences.is_empty(), "{}", differences.join("\n"));
}

#[test]
fn empty_stacking_range_keeps_distinct_line_identities_f32() {
    assert_empty_stacking_range_keeps_distinct_line_identities::<f32>();
}

#[test]
fn empty_stacking_range_keeps_distinct_line_identities_f64() {
    assert_empty_stacking_range_keeps_distinct_line_identities::<f64>();
}

fn assert_positioned_grid_axis_qualification_does_not_change_in_flow_range<S: LayoutScalar>() {
    let mut differences = Vec::new();
    for axis in [GridAxisKind::Column, GridAxisKind::Row] {
        for range in [
            PositionedRange::Centered,
            PositionedRange::Signed,
            PositionedRange::Empty,
        ] {
            let control = positioned_line_tree::<S>(
                axis,
                range,
                RawGridPlacement::AUTO,
                RawGridPlacement::AUTO,
                false,
                false,
            );
            check_positioned_line_tree(
                &control,
                axis,
                range,
                None,
                "without absolute child",
                &mut differences,
            );
        }
        for (label, grid_reference, grid_start, grid_size) in [
            ("explicit", RawGridPlacement::lines(1, 2), 15.0, 70.0),
            (
                "negative aliases",
                RawGridPlacement::lines(-2, -1),
                15.0,
                70.0,
            ),
            (
                "invalid grid end",
                RawGridPlacement::lines(1, 100),
                15.0,
                80.0,
            ),
            (
                "invalid grid start",
                RawGridPlacement::lines(-100, 2),
                5.0,
                80.0,
            ),
            (
                "both invalid grid lines",
                RawGridPlacement::lines(-100, 100),
                5.0,
                90.0,
            ),
        ] {
            let tree = positioned_line_tree::<S>(
                axis,
                PositionedRange::Centered,
                grid_reference,
                RawGridPlacement::lines(1, 2),
                false,
                true,
            );
            check_positioned_line_tree(
                &tree,
                axis,
                PositionedRange::Centered,
                Some((grid_start, 40.0, grid_size, 40.0)),
                label,
                &mut differences,
            );
        }
    }
    assert!(differences.is_empty(), "{}", differences.join("\n"));
}

#[test]
fn positioned_grid_axis_qualification_does_not_change_in_flow_range_f32() {
    assert_positioned_grid_axis_qualification_does_not_change_in_flow_range::<f32>();
}

#[test]
fn positioned_grid_axis_qualification_does_not_change_in_flow_range_f64() {
    assert_positioned_grid_axis_qualification_does_not_change_in_flow_range::<f64>();
}

fn assert_partial_grid_span_uses_logical_start_insets<S: LayoutScalar>() {
    let px = S::from_f64;
    let mut differences = Vec::new();
    for (mode, direction, child_rect, flow_rect) in [
        (
            WritingMode::HorizontalTb,
            Direction::Ltr,
            [60.0, 12.0, 20.0, 15.0],
            [12.0, 12.0, 40.0, 30.0],
        ),
        (
            WritingMode::HorizontalTb,
            Direction::Rtl,
            [40.0, 12.0, 20.0, 15.0],
            [68.0, 12.0, 40.0, 30.0],
        ),
        (
            WritingMode::VerticalRl,
            Direction::Ltr,
            [93.0, 60.0, 15.0, 20.0],
            [78.0, 12.0, 30.0, 40.0],
        ),
    ] {
        for reversed in [false, true] {
            let vertical = mode == WritingMode::VerticalRl;
            let mut inset = Edges::all(LengthAutoOf::AUTO);
            inset.top = LengthAutoOf::ZERO;
            if vertical || direction == Direction::Rtl {
                inset.right = LengthAutoOf::ZERO;
            } else {
                inset.left = LengthAutoOf::ZERO;
            }
            let tree = PublicLayoutTreeOf::<S>::new()
                .children(1, [2, 3])
                .children(2, [])
                .children(3, [])
                .style(
                    1,
                    NodeInputOf {
                        display: Display::GridLanes,
                        position: Position::Relative,
                        writing_mode: mode,
                        direction,
                        box_sizing: BoxSizing::BorderBox,
                        size: Size::splat(px(120.0)).map(PreferredSizeOf::px),
                        border: Edges::all(LengthOf::px(px(2.0))),
                        padding: Edges::all(LengthOf::px(px(10.0))),
                        grid_lanes_flow: GridLanesFlow::new(GridAxisKind::Column, reversed, false),
                        grid_template_columns: vec![TrackComponentOf::px(px(40.0)); 2],
                        gap: if vertical {
                            Size::new(LengthOf::ZERO, LengthOf::px(px(8.0)))
                        } else {
                            Size::new(LengthOf::px(px(8.0)), LengthOf::ZERO)
                        },
                        justify_content: Some(AlignContent::Start),
                        align_content: Some(AlignContent::Start),
                        ..NodeInputOf::default()
                    },
                )
                .style(
                    2,
                    NodeInputOf {
                        display: Display::Block,
                        position: Position::Absolute,
                        writing_mode: mode,
                        direction,
                        size: Size::splat(px(0.5)).map(PreferredSizeOf::percent),
                        raw_grid_column: RawGridPlacement::lines(2, 3),
                        raw_grid_row: RawGridPlacement::lines(1, 2),
                        inset,
                        justify_self: Some(AlignItems::Start),
                        align_self: Some(AlignItems::Start),
                        ..NodeInputOf::default()
                    },
                )
                .style(
                    3,
                    NodeInputOf {
                        display: Display::Block,
                        writing_mode: mode,
                        direction,
                        size: if vertical {
                            Size::new(px(30.0), px(40.0))
                        } else {
                            Size::new(px(40.0), px(30.0))
                        }
                        .map(PreferredSizeOf::px),
                        grid_column: GridPlacement::try_line(1).unwrap(),
                        justify_self: Some(AlignItems::Start),
                        align_self: Some(AlignItems::Start),
                        ..NodeInputOf::default()
                    },
                );
            let label = format!("{mode:?} {direction:?} reversed={reversed}");
            let batch = match compute_layout(
                &tree,
                1,
                super::super::fixtures::fri08_c01_placement_request(),
            ) {
                Ok(batch) => batch,
                Err(error) => {
                    differences.push(format!("{label}: {error:?}"));
                    continue;
                }
            };
            // Independent physical table: content inset is 2+10=12. Column 2
            // starts logically at 12+40+8=60 and has extent40. The stacking
            // range is 12..42, extent30. Authored 50% gives grid20/stack15.
            // LTR logical starts are physical left/top, yielding (60,12).
            // RTL starts at right/top: x=120-60-20=40, not physical-left x20.
            // VerticalRl starts at right/top: x=120-12-15=93 and y=60.
            // Reversal changes only automatic candidate traversal: these
            // explicit lines and the definite in-flow placement stay fixed.
            for (phase, entries) in [
                ("unrounded", batch.unrounded_entries()),
                ("final", batch.final_entries()),
            ] {
                assert_eq!(entries.len(), 3, "{label}, {phase}: real source boxes");
                for (node, rect) in [
                    (1, [0.0, 0.0, 120.0, 120.0]),
                    (2, child_rect),
                    (3, flow_rect),
                ] {
                    let output = entries
                        .iter()
                        .find(|entry| entry.node() == node)
                        .unwrap_or_else(|| panic!("{label}, {phase}: missing node {node}"))
                        .output();
                    let actual = (output.location, output.size);
                    let expected = (
                        Point::new(px(rect[0]), px(rect[1])),
                        Size::new(px(rect[2]), px(rect[3])),
                    );
                    if actual != expected {
                        differences.push(format!(
                            "{label}, {phase}, node {node}: {actual:?}, expected {expected:?}"
                        ));
                    }
                }
            }
        }
    }
    assert!(differences.is_empty(), "{}", differences.join("\n"));
}

#[test]
fn partial_grid_span_uses_logical_start_insets_f32() {
    assert_partial_grid_span_uses_logical_start_insets::<f32>();
}

#[test]
fn partial_grid_span_uses_logical_start_insets_f64() {
    assert_partial_grid_span_uses_logical_start_insets::<f64>();
}
