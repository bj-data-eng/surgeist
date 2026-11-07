use super::*;

fn phase_frame() -> ReferenceFrame {
    ReferenceFrame {
        extent: 100.0,
        padded: false,
        gap: 0.0,
        lines: [0.0, 20.0, 50.0],
    }
}

fn phase_wide_input<S: LayoutScalar>(flow: ReferenceFlow, width: f64) -> NodeInputOf<S> {
    let px = S::from_f64;
    let columns = flow.axis() == GridAxisKind::Column;
    NodeInputOf {
        grid_column: if columns {
            GridPlacement::try_lines(1, 3).unwrap()
        } else {
            GridPlacement::try_line(1).unwrap()
        },
        grid_row: if columns {
            GridPlacement::try_line(1).unwrap()
        } else {
            GridPlacement::try_lines(1, 3).unwrap()
        },
        size: Size::new(px(width), px(10.0)).map(PreferredSizeOf::px),
        justify_self: Some(AlignItems::Start),
        align_self: Some(AlignItems::Start),
        ..NodeInputOf::default()
    }
}

fn check_phase_track_probes<S: LayoutScalar>(
    batch: &CompletedLayoutBatchOf<u32, S>,
    flow: ReferenceFlow,
    label: &str,
    differences: &mut Vec<String>,
) {
    let px = S::from_f64;
    for (node, logical_start, width) in [(3, 0.0, 20.0), (4, 20.0, 30.0)] {
        let x = if flow.reversed() {
            100.0 - logical_start - width
        } else {
            logical_start
        };
        let output = fri08_c01_placement_output(batch, node);
        let actual = (output.location.x, output.size.width);
        let expected = (px(x), px(width));
        if actual != expected {
            differences.push(format!(
                "{label}, track probe {node}: {actual:?}, expected {expected:?}"
            ));
        }
    }
}

fn assert_absolute_reference_phase_order<S: LayoutScalar>() {
    let px = S::from_f64;
    let mut differences = Vec::new();
    for display in [Display::Grid, Display::GridLanes] {
        // Independently vary the wide in-flow box and large abspos sibling
        // before/after target 2; the probes remain in-flow in every permutation.
        for order in [
            [5, 2, 6, 3, 4],
            [2, 5, 6, 3, 4],
            [6, 5, 2, 3, 4],
            [6, 2, 5, 3, 4],
        ] {
            let tree = reference_tree::<S>(
                display,
                ReferenceFlow::Columns,
                phase_frame(),
                GridPlacement::try_line(2).unwrap(),
            )
            .children(1, order)
            .children(5, [])
            .children(6, [])
            .style(5, phase_wide_input(ReferenceFlow::Columns, 160.0))
            .style(
                6,
                NodeInputOf {
                    position: Position::Absolute,
                    size: Size::new(px(500.0), px(5.0)).map(PreferredSizeOf::px),
                    inset: Edges::new(
                        LengthAutoOf::ZERO,
                        LengthAutoOf::AUTO,
                        LengthAutoOf::AUTO,
                        LengthAutoOf::ZERO,
                    ),
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
                    differences.push(format!("{display:?}, source order {order:?}: {error:?}"));
                    continue;
                }
            };
            let root = fri08_c01_placement_output(&batch, 1);
            let target = fri08_c01_placement_output(&batch, 2);
            let wide = fri08_c01_placement_output(&batch, 5);
            let large = fri08_c01_placement_output(&batch, 6);
            // Grid 2 abspos qualification is frozen after in-flow layout:
            // valid line 2 at 20 to qualified Auto end 160 gives 140/2 = 70.
            // The fixed abspos sibling contributes to final overflow, ending
            // at 500, but cannot change that frozen end to 500 (width 240).
            let actual = (
                root.size,
                root.scroll_geometry.map(|geometry| {
                    (
                        geometry.scrollable_overflow().origin(),
                        geometry.scrollable_overflow().size(),
                    )
                }),
                (target.location, target.size),
                (wide.location, wide.size),
                (large.location, large.size),
            );
            let expected = (
                Size::new(px(100.0), px(20.0)),
                Some((Point::ZERO, Size::new(px(500.0), px(20.0)))),
                (Point::new(px(20.0), S::ZERO), Size::new(px(70.0), px(5.0))),
                (Point::ZERO, Size::new(px(160.0), px(10.0))),
                (Point::ZERO, Size::new(px(500.0), px(5.0))),
            );
            if actual != expected {
                differences.push(format!(
                    "{display:?}, source order {order:?}: {actual:?}, expected {expected:?}"
                ));
            }
            check_phase_track_probes(
                &batch,
                ReferenceFlow::Columns,
                &format!("{display:?}, source order {order:?}"),
                &mut differences,
            );
        }
    }
    assert!(differences.is_empty(), "{}", differences.join("\n"));
}

#[test]
fn absolute_reference_phase_order_f32() {
    assert_absolute_reference_phase_order::<f32>();
}

#[test]
fn absolute_reference_phase_order_f64() {
    assert_absolute_reference_phase_order::<f64>();
}

fn assert_absolute_reference_descendant_exposure<S: LayoutScalar>() {
    let px = S::from_f64;
    let mut differences = Vec::new();
    for display in [Display::Grid, Display::GridLanes] {
        for exposure in [Overflow::Visible, Overflow::Hidden, Overflow::Clip] {
            let tree = reference_tree::<S>(
                display,
                ReferenceFlow::Columns,
                phase_frame(),
                GridPlacement::try_line(2).unwrap(),
            )
            .children(1, [2, 5, 3, 4])
            .children(5, [7])
            .children(7, [])
            .style(
                5,
                NodeInputOf {
                    display: Display::Block,
                    overflow: ComputedOverflow::try_new(exposure, exposure).unwrap(),
                    ..phase_wide_input(ReferenceFlow::Columns, 50.0)
                },
            )
            .style(
                7,
                NodeInputOf {
                    display: Display::Block,
                    size: Size::new(px(160.0), px(5.0)).map(PreferredSizeOf::px),
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
                    differences.push(format!("{display:?}, exposure {exposure:?}: {error:?}"));
                    continue;
                }
            };
            let root = fri08_c01_placement_output(&batch, 1);
            let target = fri08_c01_placement_output(&batch, 2);
            let wrapper = fri08_c01_placement_output(&batch, 5);
            let descendant = fri08_c01_placement_output(&batch, 7);
            // Overflow 3 permits transitive descendant overflow only through
            // Visible axes. The wrapper's own border ends at 50 in every case;
            // visible descendant end 160 qualifies Auto, whereas Hidden/Clip
            // trap it, leaving the parent's 100-unit padding-box seed.
            let visible = exposure == Overflow::Visible;
            let end = if visible { 160.0 } else { 100.0 };
            let target_width = if visible { 70.0 } else { 40.0 };
            let actual = (
                root.size,
                root.scroll_geometry.map(|geometry| {
                    (
                        geometry.scrollable_overflow().origin(),
                        geometry.scrollable_overflow().size(),
                    )
                }),
                (target.location, target.size),
                (wrapper.location, wrapper.size),
                (descendant.location, descendant.size),
            );
            let expected = (
                Size::new(px(100.0), px(20.0)),
                Some((Point::ZERO, Size::new(px(end), px(20.0)))),
                (
                    Point::new(px(20.0), S::ZERO),
                    Size::new(px(target_width), px(5.0)),
                ),
                (Point::ZERO, Size::new(px(50.0), px(10.0))),
                (Point::ZERO, Size::new(px(160.0), px(5.0))),
            );
            if actual != expected {
                differences.push(format!(
                    "{display:?}, exposure {exposure:?}: {actual:?}, expected {expected:?}"
                ));
            }
            check_phase_track_probes(
                &batch,
                ReferenceFlow::Columns,
                &format!("{display:?}, exposure {exposure:?}"),
                &mut differences,
            );
        }
    }
    assert!(differences.is_empty(), "{}", differences.join("\n"));
}

#[test]
fn absolute_reference_descendant_exposure_f32() {
    assert_absolute_reference_descendant_exposure::<f32>();
}

#[test]
fn absolute_reference_descendant_exposure_f64() {
    assert_absolute_reference_descendant_exposure::<f64>();
}

fn assert_absolute_reference_signed_auto_area<S: LayoutScalar>() {
    let px = S::from_f64;
    let mut differences = Vec::new();
    for display in [Display::Grid, Display::GridLanes] {
        for flow in [ReferenceFlow::RtlColumns, ReferenceFlow::VerticalRlRows] {
            let tree = reference_tree::<S>(
                display,
                flow,
                phase_frame(),
                GridPlacement::try_line(2).unwrap(),
            )
            .children(1, [2, 5, 3, 4])
            .children(5, [])
            .style(5, phase_wide_input(flow, 160.0));
            let batch = match compute_layout(
                &tree,
                1,
                super::super::fixtures::fri08_c01_placement_request(),
            ) {
                Ok(batch) => batch,
                Err(error) => {
                    differences.push(format!("{display:?} {flow:?}: {error:?}"));
                    continue;
                }
            };
            let root = fri08_c01_placement_output(&batch, 1);
            let target = fri08_c01_placement_output(&batch, 2);
            let wide = fri08_c01_placement_output(&batch, 5);
            // Both flows progress toward physical left. Start-aligning 160 in
            // a 100-unit box gives 100-160 = -60. The qualified logical Auto
            // area is 0..160; valid line 2 remains 20, so its 140-unit interval
            // projects to physical -60..80. Physical left:0 and 50% give
            // target origin -60 and width 70. Complete overflow retains this
            // signed origin, independently of reachable scroll-range policy.
            let actual = (
                root.size,
                root.scroll_geometry.map(|geometry| {
                    (
                        geometry.scrollable_overflow().origin(),
                        geometry.scrollable_overflow().size(),
                    )
                }),
                (wide.location, wide.size),
                (target.location, target.size),
            );
            let expected = (
                Size::new(px(100.0), px(20.0)),
                Some((
                    Point::new(px(-60.0), S::ZERO),
                    Size::new(px(160.0), px(20.0)),
                )),
                (
                    Point::new(px(-60.0), S::ZERO),
                    Size::new(px(160.0), px(10.0)),
                ),
                (Point::new(px(-60.0), S::ZERO), Size::new(px(70.0), px(5.0))),
            );
            if actual != expected {
                differences.push(format!(
                    "{display:?} {flow:?}: {actual:?}, expected {expected:?}"
                ));
            }
            check_phase_track_probes(
                &batch,
                flow,
                &format!("{display:?} {flow:?}"),
                &mut differences,
            );
        }
    }
    assert!(differences.is_empty(), "{}", differences.join("\n"));
}

#[test]
fn absolute_reference_signed_auto_area_f32() {
    assert_absolute_reference_signed_auto_area::<f32>();
}

#[test]
fn absolute_reference_signed_auto_area_f64() {
    assert_absolute_reference_signed_auto_area::<f64>();
}
