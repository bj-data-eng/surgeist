use super::*;

fn positioned_static_root<S: LayoutScalar>(
    display: Display,
    writing_mode: WritingMode,
    direction: Direction,
) -> NodeInputOf<S> {
    let px = S::from_f64;
    NodeInputOf {
        display,
        position: Position::Relative,
        writing_mode,
        direction,
        box_sizing: BoxSizing::BorderBox,
        size: Size::new(px(100.0), px(120.0)).map(PreferredSizeOf::px),
        padding: Edges::all(LengthOf::px(px(10.0))),
        border: Edges::all(LengthOf::px(px(5.0))),
        margin: Edges::all(LengthAutoOf::ZERO),
        grid_lanes_flow: GridLanesFlow::new(GridAxisKind::Column, false, false),
        grid_template_columns: [20.0, 30.0]
            .map(|value| TrackComponentOf::px(px(value)))
            .into(),
        grid_template_rows: vec![TrackComponentOf::px(px(20.0))],
        grid_flow_tolerance: GridFlowToleranceOf::Length(LengthOf::ZERO),
        gap: Size::splat(LengthOf::ZERO),
        justify_content: Some(AlignContent::Start),
        align_content: Some(AlignContent::Start),
        ..NodeInputOf::default()
    }
}

fn positioned_static_child<S: LayoutScalar>(
    writing_mode: WritingMode,
    direction: Direction,
    justify: AlignItems,
    align: AlignItems,
    physical_zero_insets: bool,
) -> NodeInputOf<S> {
    NodeInputOf {
        display: Display::Block,
        position: Position::Absolute,
        writing_mode,
        direction,
        box_sizing: BoxSizing::BorderBox,
        size: Size::splat(S::from_f64(10.0)).map(PreferredSizeOf::px),
        justify_self: Some(justify),
        align_self: Some(align),
        margin: Edges::all(LengthAutoOf::ZERO),
        inset: if physical_zero_insets {
            Edges::new(
                LengthAutoOf::ZERO,
                LengthAutoOf::AUTO,
                LengthAutoOf::AUTO,
                LengthAutoOf::ZERO,
            )
        } else {
            Edges::all(LengthAutoOf::AUTO)
        },
        ..NodeInputOf::default()
    }
}

fn check_native_positioned_box<S: LayoutScalar>(
    batch: &CompletedLayoutBatchOf<u32, S>,
    node: u32,
    expected_rect: (Point<S>, Size<S>),
    root: bool,
    label: &str,
    differences: &mut Vec<String>,
) {
    let px = S::from_f64;
    let output = batch
        .unrounded_entries()
        .iter()
        .find(|entry| entry.node() == node)
        .expect("native layout publishes the requested unrounded box")
        .output();
    let actual = (
        output.location,
        output.size,
        output.padding,
        output.border,
        output.margin,
    );
    let expected = (
        expected_rect.0,
        expected_rect.1,
        Edges::all(px(if root { 10.0 } else { 0.0 })),
        Edges::all(px(if root { 5.0 } else { 0.0 })),
        Edges::ZERO,
    );
    if actual != expected {
        differences.push(format!(
            "{label}, node {node}: {actual:?}, expected {expected:?}"
        ));
    }
}

fn assert_static_auto_insets_use_content_rectangle<S: LayoutScalar>() {
    let px = S::from_f64;
    let mut differences = Vec::new();
    // #998 selects ordinary Grid's content-box static rectangle for Lanes.
    // Jan Grid 3 #abspos retains an editorial question; this is the adopted
    // policy, supported by the pinned WebKit content-start witness, not a claim
    // that January normatively resolves that question for every implementation.
    for display in [Display::Grid, Display::GridLanes] {
        for (writing_mode, direction, positions, tall_location, tall_size) in [
            (
                WritingMode::HorizontalTb,
                Direction::Ltr,
                [(15.0, 15.0), (75.0, 95.0), (75.0, 15.0), (15.0, 95.0)],
                (15.0, 15.0),
                (50.0, 200.0),
            ),
            (
                WritingMode::HorizontalTb,
                Direction::Rtl,
                [(75.0, 15.0), (15.0, 95.0), (15.0, 15.0), (75.0, 95.0)],
                (35.0, 15.0),
                (50.0, 200.0),
            ),
            (
                WritingMode::VerticalRl,
                Direction::Ltr,
                [(75.0, 15.0), (15.0, 95.0), (75.0, 95.0), (15.0, 15.0)],
                (-115.0, 15.0),
                (200.0, 50.0),
            ),
        ] {
            for ((justify, align), (x, y)) in [
                (AlignItems::Start, AlignItems::Start),
                (AlignItems::End, AlignItems::End),
                (AlignItems::End, AlignItems::Start),
                (AlignItems::Start, AlignItems::End),
            ]
            .into_iter()
            .zip(positions)
            {
                for source_order in [0, 1, 2] {
                    let mut tree = PublicLayoutTreeOf::<S>::new()
                        .children(
                            1,
                            match source_order {
                                0 => vec![2],
                                1 => vec![2, 3],
                                _ => vec![3, 2],
                            },
                        )
                        .children(2, [])
                        .style(1, positioned_static_root(display, writing_mode, direction))
                        .style(
                            2,
                            positioned_static_child(writing_mode, direction, justify, align, false),
                        );
                    if source_order != 0 {
                        tree = tree.children(3, []).style(
                            3,
                            NodeInputOf {
                                display: Display::Block,
                                writing_mode,
                                direction,
                                box_sizing: BoxSizing::BorderBox,
                                size: Size::new(px(tall_size.0), px(tall_size.1))
                                    .map(PreferredSizeOf::px),
                                grid_column: GridPlacement::try_lines(1, 3).unwrap(),
                                grid_row: GridPlacement::try_line(1).unwrap(),
                                justify_self: Some(AlignItems::Start),
                                align_self: Some(AlignItems::Start),
                                margin: Edges::all(LengthAutoOf::ZERO),
                                ..NodeInputOf::default()
                            },
                        );
                    }
                    let label = format!(
                        "{display:?} {writing_mode:?} {direction:?}, {justify:?}/{align:?}, source order {source_order}"
                    );
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
                    // Root border box 100x120 minus borders 5+5 and padding
                    // 10+10 yields content rectangle (15,15), 70x90. A 10x10
                    // static box therefore reaches low edges 15 and high edges
                    // 85-10=75 / 105-10=95. The table independently maps inline
                    // and block Start/End in each authored physical flow.
                    check_native_positioned_box(
                        &batch,
                        1,
                        (Point::ZERO, Size::new(px(100.0), px(120.0))),
                        true,
                        &label,
                        &mut differences,
                    );
                    check_native_positioned_box(
                        &batch,
                        2,
                        (Point::new(px(x), px(y)), Size::splat(px(10.0))),
                        false,
                        &label,
                        &mut differences,
                    );
                    let root = batch
                        .unrounded_entries()
                        .iter()
                        .find(|entry| entry.node() == 1)
                        .unwrap()
                        .output();
                    let actual_content = root.scroll_geometry.map(|geometry| {
                        (
                            geometry.content_box().origin(),
                            geometry.content_box().size(),
                        )
                    });
                    let expected_content = Some((
                        Point::new(px(15.0), px(15.0)),
                        Size::new(px(70.0), px(90.0)),
                    ));
                    if actual_content != expected_content {
                        differences.push(format!("{label}: content rectangle {actual_content:?}, expected {expected_content:?}"));
                    }
                    if source_order != 0 {
                        // The tall box spans the 20+30 grid tracks. Start is
                        // physical right for RTL inline / VerticalRl block, so
                        // its low edges are 85-50=35 or 85-200=-115. Abspos
                        // source order cannot alter this in-flow geometry.
                        check_native_positioned_box(
                            &batch,
                            3,
                            (
                                Point::new(px(tall_location.0), px(tall_location.1)),
                                Size::new(px(tall_size.0), px(tall_size.1)),
                            ),
                            false,
                            &label,
                            &mut differences,
                        );
                    }
                }
            }
        }
    }
    assert!(differences.is_empty(), "{}", differences.join("\n"));
}

#[test]
fn static_auto_insets_use_content_rectangle_f32() {
    assert_static_auto_insets_use_content_rectangle::<f32>();
}

#[test]
fn static_auto_insets_use_content_rectangle_f64() {
    assert_static_auto_insets_use_content_rectangle::<f64>();
}

fn assert_physical_zero_insets_keep_padding_containing_block<S: LayoutScalar>() {
    let px = S::from_f64;
    let mut differences = Vec::new();
    for display in [Display::Grid, Display::GridLanes] {
        for (writing_mode, direction) in [
            (WritingMode::HorizontalTb, Direction::Ltr),
            (WritingMode::HorizontalTb, Direction::Rtl),
            (WritingMode::VerticalRl, Direction::Ltr),
        ] {
            let tree = PublicLayoutTreeOf::<S>::new()
                .children(1, [2])
                .children(2, [])
                .style(1, positioned_static_root(display, writing_mode, direction))
                .style(
                    2,
                    positioned_static_child(
                        writing_mode,
                        direction,
                        AlignItems::Start,
                        AlignItems::Start,
                        true,
                    ),
                );
            let label = format!("{display:?} {writing_mode:?} {direction:?}, physical zero insets");
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
            // Explicit physical left/top 0 use the ordinary Auto padding-edge
            // containing block, beginning just inside the 5-unit border. This
            // separate control must not acquire the static rectangle's inset15.
            check_native_positioned_box(
                &batch,
                1,
                (Point::ZERO, Size::new(px(100.0), px(120.0))),
                true,
                &label,
                &mut differences,
            );
            check_native_positioned_box(
                &batch,
                2,
                (Point::new(px(5.0), px(5.0)), Size::splat(px(10.0))),
                false,
                &label,
                &mut differences,
            );
        }
    }
    assert!(differences.is_empty(), "{}", differences.join("\n"));
}

#[test]
fn physical_zero_insets_keep_padding_containing_block_f32() {
    assert_physical_zero_insets_keep_padding_containing_block::<f32>();
}

#[test]
fn physical_zero_insets_keep_padding_containing_block_f64() {
    assert_physical_zero_insets_keep_padding_containing_block::<f64>();
}

fn assert_lanes_without_track_templates_keep_absolute_children<S: LayoutScalar>() {
    let px = S::from_f64;
    let mut differences = Vec::new();
    for axis in [GridAxisKind::Column, GridAxisKind::Row] {
        for physical_zero_insets in [false, true] {
            let tree = PublicLayoutTreeOf::<S>::new()
                .children(1, [2])
                .children(2, [])
                .style(
                    1,
                    NodeInputOf {
                        grid_lanes_flow: GridLanesFlow::new(axis, false, false),
                        grid_template_columns: Vec::new(),
                        grid_template_rows: Vec::new(),
                        ..positioned_static_root(
                            Display::GridLanes,
                            WritingMode::HorizontalTb,
                            Direction::Ltr,
                        )
                    },
                )
                .style(
                    2,
                    positioned_static_child(
                        WritingMode::HorizontalTb,
                        Direction::Ltr,
                        AlignItems::Start,
                        AlignItems::Start,
                        physical_zero_insets,
                    ),
                );
            let label =
                format!("zero {axis:?} tracks, physical zero insets {physical_zero_insets}");
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
            // A native absolute child needs no in-flow grid tracks. The static
            // content origin remains 5+10=15; definite physical zero insets
            // instead use padding edges at 5. Neither case requests grid lines.
            check_native_positioned_box(
                &batch,
                1,
                (Point::ZERO, Size::new(px(100.0), px(120.0))),
                true,
                &label,
                &mut differences,
            );
            let actual = batch
                .unrounded_entries()
                .iter()
                .find(|entry| entry.node() == 2)
                .map(|entry| {
                    let output = entry.output();
                    (
                        output.location,
                        output.size,
                        output.padding,
                        output.border,
                        output.margin,
                    )
                });
            let start = px(if physical_zero_insets { 5.0 } else { 15.0 });
            let expected = Some((
                Point::new(start, start),
                Size::splat(px(10.0)),
                Edges::ZERO,
                Edges::ZERO,
                Edges::ZERO,
            ));
            if actual != expected {
                differences.push(format!(
                    "{label}: unrounded absolute box {actual:?}, expected {expected:?}"
                ));
            }
        }
        for both_axes_intrinsic in [false, true] {
            for has_absolute_child in [false, true] {
                let expected_root_size = if both_axes_intrinsic {
                    Size::splat(px(30.0))
                } else if axis == GridAxisKind::Column {
                    Size::new(px(30.0), px(120.0))
                } else {
                    Size::new(px(100.0), px(30.0))
                };
                let mut tree = PublicLayoutTreeOf::<S>::new()
                    .children(
                        1,
                        if has_absolute_child {
                            vec![2]
                        } else {
                            Vec::new()
                        },
                    )
                    .style(
                        1,
                        NodeInputOf {
                            grid_lanes_flow: GridLanesFlow::new(axis, false, false),
                            grid_template_columns: Vec::new(),
                            grid_template_rows: Vec::new(),
                            size: if both_axes_intrinsic {
                                Size::new(
                                    PreferredSizeOf::MIN_CONTENT,
                                    PreferredSizeOf::MIN_CONTENT,
                                )
                            } else if axis == GridAxisKind::Column {
                                Size::new(
                                    PreferredSizeOf::MIN_CONTENT,
                                    PreferredSizeOf::px(px(120.0)),
                                )
                            } else {
                                Size::new(
                                    PreferredSizeOf::px(px(100.0)),
                                    PreferredSizeOf::MIN_CONTENT,
                                )
                            },
                            ..positioned_static_root(
                                Display::GridLanes,
                                WritingMode::HorizontalTb,
                                Direction::Ltr,
                            )
                        },
                    );
                if has_absolute_child {
                    tree = tree.children(2, []).style(
                        2,
                        positioned_static_child(
                            WritingMode::HorizontalTb,
                            Direction::Ltr,
                            AlignItems::Start,
                            AlignItems::Start,
                            false,
                        ),
                    );
                }
                let label = format!(
                    "zero {axis:?} intrinsic tracks, both axes intrinsic {both_axes_intrinsic}, absolute child {has_absolute_child}"
                );
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
                // Grid 2's out-of-flow rule excludes abspos children from track
                // sizing. With no in-flow content, grid-axis min-content is only
                // two borders and paddings: 2*(5+10)=30, with or without the child.
                // A fictitious 10-unit track created from that child would give 40.
                // Both intrinsic axes also exclude a synthetic stacking interval.
                check_native_positioned_box(
                    &batch,
                    1,
                    (Point::ZERO, expected_root_size),
                    true,
                    &label,
                    &mut differences,
                );
            }
        }
    }
    assert!(differences.is_empty(), "{}", differences.join("\n"));
}

#[test]
fn lanes_without_track_templates_keep_absolute_children_f32() {
    assert_lanes_without_track_templates_keep_absolute_children::<f32>();
}

#[test]
fn lanes_without_track_templates_keep_absolute_children_f64() {
    assert_lanes_without_track_templates_keep_absolute_children::<f64>();
}
