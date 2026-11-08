use super::*;
use crate::test_support::readback::{self, Phase};

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
    let output = readback::single_box_output(batch, node, Phase::Unrounded);
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
                    let root = readback::single_box_output(&batch, 1, Phase::Unrounded);
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
                .node_layout(2)
                .map(|layout| match layout {
                    crate::NodeLayoutRefOf::SingleBox(value) => value.unrounded_entry(),
                    crate::NodeLayoutRefOf::NonBox(value) => value.unrounded_entry(),
                    crate::NodeLayoutRefOf::Fragments(_) => {
                        panic!("continuous fixture unexpectedly fragmented")
                    }
                })
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

fn assert_dense_holes_and_reverse_keep_static_rectangle<S: LayoutScalar>() {
    let px = S::from_f64;
    let mut differences = Vec::new();
    let leaf = |width, height| NodeInputOf {
        display: Display::Block,
        writing_mode: WritingMode::HorizontalTb,
        direction: Direction::Ltr,
        size: Size::new(px(width), px(height)).map(PreferredSizeOf::px),
        justify_self: Some(AlignItems::Start),
        align_self: Some(AlignItems::Start),
        margin: Edges::all(LengthAutoOf::ZERO),
        ..NodeInputOf::default()
    };
    for reversed in [false, true] {
        for (alignment, absolute_start) in [
            (AlignItems::Start, (15.0, 15.0)),
            (AlignItems::End, (75.0, 95.0)),
        ] {
            for insertion in [0, 1, 2, 3] {
                let children = match insertion {
                    0 => vec![3, 4, 5],
                    1 => vec![2, 3, 4, 5],
                    2 => vec![3, 4, 2, 5],
                    _ => vec![3, 4, 5, 2],
                };
                let mut tree = PublicLayoutTreeOf::<S>::new()
                    .children(1, children)
                    .children(3, [])
                    .children(4, [])
                    .children(5, [])
                    .style(
                        1,
                        NodeInputOf {
                            grid_template_columns: vec![TrackComponentOf::px(px(20.0)); 3],
                            grid_lanes_flow: GridLanesFlow::new(
                                GridAxisKind::Column,
                                reversed,
                                true,
                            ),
                            gap: Size::splat(LengthOf::px(px(5.0))),
                            ..positioned_static_root(
                                Display::GridLanes,
                                WritingMode::HorizontalTb,
                                Direction::Ltr,
                            )
                        },
                    )
                    .style(
                        3,
                        NodeInputOf {
                            grid_column: GridPlacement::try_line(1).unwrap(),
                            ..leaf(20.0, 30.0)
                        },
                    )
                    .style(
                        4,
                        NodeInputOf {
                            grid_column: GridPlacement::try_lines(1, 3).unwrap(),
                            ..leaf(45.0, 10.0)
                        },
                    )
                    .style(5, leaf(20.0, 20.0));
                if insertion != 0 {
                    tree = tree.children(2, []).style(
                        2,
                        positioned_static_child(
                            WritingMode::HorizontalTb,
                            Direction::Ltr,
                            alignment,
                            alignment,
                            false,
                        ),
                    );
                }
                let label = format!(
                    "dense hole, reversed {reversed}, {alignment:?}, insertion {insertion}"
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
                // Content width 70 is 20+5+20+5+20, with starts 15/40/65.
                // A's 30 plus gutter 5 places B at stacking offset 35 (y=50).
                // B leaves the second lane's 0..35 hole. C's 20+5 fits there.
                // Forward primary order visits that hole before the equally
                // high empty third lane, so C backfills lane 2. Reverse order
                // visits empty lane 3 first; the equal-offset hole is not an
                // earlier placement, so C stays in lane 3. Definite A/B do not
                // change. Abspos insertion cannot consume the hole or cursor.
                check_native_positioned_box(
                    &batch,
                    1,
                    (Point::ZERO, Size::new(px(100.0), px(120.0))),
                    true,
                    &label,
                    &mut differences,
                );
                for (node, x, y, width, height) in [
                    (3, 15.0, 15.0, 20.0, 30.0),
                    (4, 15.0, 50.0, 45.0, 10.0),
                    (5, if reversed { 65.0 } else { 40.0 }, 15.0, 20.0, 20.0),
                ] {
                    check_native_positioned_box(
                        &batch,
                        node,
                        (Point::new(px(x), px(y)), Size::new(px(width), px(height))),
                        false,
                        &label,
                        &mut differences,
                    );
                }
                if insertion != 0 {
                    // The fixed content-box static rectangle is independent of
                    // the actual skipped space and primary traversal direction.
                    check_native_positioned_box(
                        &batch,
                        2,
                        (
                            Point::new(px(absolute_start.0), px(absolute_start.1)),
                            Size::splat(px(10.0)),
                        ),
                        false,
                        &label,
                        &mut differences,
                    );
                }
            }
        }
    }
    assert!(differences.is_empty(), "{}", differences.join("\n"));
}

#[test]
fn dense_holes_and_reverse_keep_static_rectangle_f32() {
    assert_dense_holes_and_reverse_keep_static_rectangle::<f32>();
}

#[test]
fn dense_holes_and_reverse_keep_static_rectangle_f64() {
    assert_dense_holes_and_reverse_keep_static_rectangle::<f64>();
}

fn assert_orthogonal_child_uses_parent_static_rectangle<S: LayoutScalar>() {
    let px = S::from_f64;
    let mut differences = Vec::new();
    for (parent_mode, child_mode, start, end) in [
        (
            WritingMode::HorizontalTb,
            WritingMode::VerticalRl,
            (15.0, 15.0),
            (75.0, 85.0),
        ),
        (
            WritingMode::VerticalRl,
            WritingMode::HorizontalTb,
            (75.0, 15.0),
            (15.0, 85.0),
        ),
    ] {
        for (alignment, location) in [(AlignItems::Start, start), (AlignItems::End, end)] {
            let tree = PublicLayoutTreeOf::<S>::new()
                .children(1, [2])
                .children(2, [])
                .style(
                    1,
                    positioned_static_root(Display::GridLanes, parent_mode, Direction::Ltr),
                )
                .style(
                    2,
                    NodeInputOf {
                        size: Size::new(px(10.0), px(20.0)).map(PreferredSizeOf::px),
                        ..positioned_static_child(
                            child_mode,
                            Direction::Ltr,
                            alignment,
                            alignment,
                            false,
                        )
                    },
                );
            let label = format!("parent {parent_mode:?}, child {child_mode:?}, {alignment:?}");
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
            // The child has physical width 10 and height 20 despite its own
            // writing mode. Content bounds remain x=15..85, y=15..105. In an
            // Htb parent, End gives (85-10,105-20)=(75,85). In VerticalRl,
            // inline Start is top and block Start is right, giving (75,15);
            // End is bottom/left, giving (15,85). The child's axes do not swap
            // either physical dimensions or the parent's alignment rectangle.
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
                (
                    Point::new(px(location.0), px(location.1)),
                    Size::new(px(10.0), px(20.0)),
                ),
                false,
                &label,
                &mut differences,
            );
        }
    }
    assert!(differences.is_empty(), "{}", differences.join("\n"));
}

#[test]
fn orthogonal_child_uses_parent_static_rectangle_f32() {
    assert_orthogonal_child_uses_parent_static_rectangle::<f32>();
}

#[test]
fn orthogonal_child_uses_parent_static_rectangle_f64() {
    assert_orthogonal_child_uses_parent_static_rectangle::<f64>();
}
