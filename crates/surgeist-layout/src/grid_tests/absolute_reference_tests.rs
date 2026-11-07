use super::fixtures::fri08_c01_placement_output;
use super::*;

#[derive(Clone, Copy, Debug)]
enum ReferenceFlow {
    Columns,
    RtlColumns,
    Rows,
    VerticalRlRows,
}

impl ReferenceFlow {
    fn axis(self) -> GridAxisKind {
        match self {
            Self::Columns | Self::RtlColumns => GridAxisKind::Column,
            Self::Rows | Self::VerticalRlRows => GridAxisKind::Row,
        }
    }

    fn horizontal(self) -> bool {
        !matches!(self, Self::Rows)
    }

    fn reversed(self) -> bool {
        matches!(self, Self::RtlColumns | Self::VerticalRlRows)
    }

    fn point<S: LayoutScalar>(self, selected: f64, other: f64) -> Point<S> {
        let px = S::from_f64;
        if self.horizontal() {
            Point::new(px(selected), px(other))
        } else {
            Point::new(px(other), px(selected))
        }
    }

    fn size<S: LayoutScalar>(self, selected: f64, other: f64) -> Size<S> {
        let point = self.point(selected, other);
        Size::new(point.x, point.y)
    }
}

#[derive(Clone, Copy, Debug)]
struct ReferenceFrame {
    extent: f64,
    padded: bool,
    gap: f64,
    // Independently authored logical coordinates; the tree never consumes them.
    lines: [f64; 3],
}

fn reference_tree<S: LayoutScalar>(
    display: Display,
    flow: ReferenceFlow,
    frame: ReferenceFrame,
    placement: GridPlacement,
) -> PublicLayoutTreeOf<S> {
    let px = S::from_f64;
    let columns = flow.axis() == GridAxisKind::Column;
    let mut padding = Edges::ZERO;
    if frame.padded {
        let (low, high) = if flow.reversed() {
            (20.0, 10.0)
        } else {
            (10.0, 20.0)
        };
        if flow.horizontal() {
            padding.left = px(low);
            padding.right = px(high);
        } else {
            padding.top = px(low);
            padding.bottom = px(high);
        }
    }
    let selected_alignment = if frame.padded {
        AlignContent::End
    } else {
        AlignContent::Start
    };
    let mut tree = PublicLayoutTreeOf::new()
        .children(1, [2, 3, 4])
        .children(2, [])
        .style(
            1,
            NodeInputOf {
                display,
                position: Position::Relative,
                writing_mode: if matches!(flow, ReferenceFlow::VerticalRlRows) {
                    WritingMode::VerticalRl
                } else {
                    WritingMode::HorizontalTb
                },
                direction: if matches!(flow, ReferenceFlow::RtlColumns) {
                    Direction::Rtl
                } else {
                    Direction::Ltr
                },
                box_sizing: BoxSizing::BorderBox,
                size: flow.size(frame.extent, 20.0).map(PreferredSizeOf::px),
                padding: padding.map(LengthOf::px),
                grid_lanes_flow: GridLanesFlow::new(flow.axis(), false, false),
                grid_template_columns: if columns {
                    [20.0, 30.0]
                        .map(|value| TrackComponentOf::px(px(value)))
                        .into()
                } else {
                    vec![TrackComponentOf::px(px(20.0))]
                },
                grid_template_rows: if columns {
                    vec![TrackComponentOf::px(px(20.0))]
                } else {
                    [20.0, 30.0]
                        .map(|value| TrackComponentOf::px(px(value)))
                        .into()
                },
                // Any accidentally created implicit tracks would alter alignment
                // and the independently checked in-flow track probes below.
                grid_auto_columns: vec![TrackComponentOf::px(px(11.0))],
                grid_auto_rows: vec![TrackComponentOf::px(px(11.0))],
                gap: if columns {
                    Size::new(LengthOf::px(px(frame.gap)), LengthOf::ZERO)
                } else {
                    Size::new(LengthOf::ZERO, LengthOf::px(px(frame.gap)))
                },
                justify_content: Some(if columns {
                    selected_alignment
                } else {
                    AlignContent::Start
                }),
                align_content: Some(if columns {
                    AlignContent::Start
                } else {
                    selected_alignment
                }),
                ..NodeInputOf::default()
            },
        )
        .style(
            2,
            NodeInputOf {
                position: Position::Absolute,
                grid_column: if columns {
                    placement
                } else {
                    GridPlacement::AUTO
                },
                grid_row: if columns {
                    GridPlacement::AUTO
                } else {
                    placement
                },
                size: if flow.horizontal() {
                    Size::new(
                        PreferredSizeOf::percent(px(0.5)),
                        PreferredSizeOf::px(px(5.0)),
                    )
                } else {
                    Size::new(
                        PreferredSizeOf::px(px(5.0)),
                        PreferredSizeOf::percent(px(0.5)),
                    )
                },
                // Physical left/top anchor the physical low edge in every mode.
                // Opposite auto insets avoid unrelated overconstraint precedence.
                inset: Edges::new(
                    LengthAutoOf::ZERO,
                    LengthAutoOf::AUTO,
                    LengthAutoOf::AUTO,
                    LengthAutoOf::ZERO,
                ),
                ..NodeInputOf::default()
            },
        );
    for (node, line) in [(3, 1), (4, 2)] {
        tree = tree.children(node, []).style(
            node,
            NodeInputOf {
                grid_column: GridPlacement::try_line(if columns { line } else { 1 }).unwrap(),
                grid_row: GridPlacement::try_line(if columns { 1 } else { line }).unwrap(),
                size: if flow.horizontal() {
                    Size::new(PreferredSizeOf::AUTO, PreferredSizeOf::px(px(1.0)))
                } else {
                    Size::new(PreferredSizeOf::px(px(1.0)), PreferredSizeOf::AUTO)
                },
                ..NodeInputOf::default()
            },
        );
    }
    tree
}

fn check_reference_case<S: LayoutScalar>(
    display: Display,
    flow: ReferenceFlow,
    frame: ReferenceFrame,
    placement: GridPlacement,
    logical_area: (f64, f64),
    differences: &mut Vec<String>,
) {
    let tree = reference_tree::<S>(display, flow, frame, placement);
    let batch = match compute_layout(&tree, 1, super::fixtures::fri08_c01_placement_request()) {
        Ok(batch) => batch,
        Err(error) => {
            differences.push(format!(
                "{display:?} {flow:?} {frame:?} {placement:?}: {error:?}"
            ));
            return;
        }
    };
    let (low, high) = logical_area;
    let physical_low = if flow.reversed() {
        frame.extent - high
    } else {
        low
    };
    let expected_child = (
        flow.point(physical_low, 0.0),
        flow.size((high - low) / 2.0, 5.0),
    );
    let child = fri08_c01_placement_output(&batch, 2);
    let actual_child = (child.location, child.size);
    if actual_child != expected_child {
        differences.push(format!("{display:?} {flow:?} {frame:?} {placement:?}: {actual_child:?}, expected {expected_child:?}"));
    }
    let root = fri08_c01_placement_output(&batch, 1);
    if root.size != flow.size(frame.extent, 20.0) {
        differences.push(format!(
            "{display:?} {flow:?} {placement:?}: root {:?}",
            root.size
        ));
    }
    for (node, start, width) in [(3, frame.lines[0], 20.0), (4, frame.lines[1], 30.0)] {
        let physical_start = if flow.reversed() {
            frame.extent - start - width
        } else {
            start
        };
        let expected = (flow.point(physical_start, 0.0), flow.size(width, 1.0));
        let output = fri08_c01_placement_output(&batch, node);
        let actual = (output.location, output.size);
        if actual != expected {
            differences.push(format!("{display:?} {flow:?} {placement:?}: in-flow track probe {node} {actual:?}, expected {expected:?}; abspos must not grow the grid"));
        }
    }
}

fn assert_absolute_reference_endpoints<S: LayoutScalar>() {
    let mut differences = Vec::new();
    // Grid 2 (pinned 2025-03-26), absolute positioning: each non-existent
    // reference becomes Auto independently, including a reference from a span.
    // Auto is the corresponding padding edge, not the first/last used line.
    for display in [Display::Grid, Display::GridLanes] {
        let base = ReferenceFrame {
            extent: 100.0,
            padded: false,
            gap: 0.0,
            lines: [0.0, 20.0, 50.0],
        };
        check_reference_case::<S>(
            display,
            ReferenceFlow::Columns,
            base,
            GridPlacement::try_line_span(2, 1).unwrap(),
            (20.0, 50.0),
            &mut differences,
        );
        check_reference_case::<S>(
            display,
            ReferenceFlow::Columns,
            base,
            GridPlacement::try_line_span(2, 4).unwrap(),
            (20.0, 100.0),
            &mut differences,
        );
        // Border box 130 minus padding 10+20 gives content 100. End-aligned
        // tracks occupy 50, so used lines are 10+50+[0,20,50] = [60,80,110].
        let padded = ReferenceFrame {
            extent: 130.0,
            padded: true,
            gap: 0.0,
            lines: [60.0, 80.0, 110.0],
        };
        for flow in [
            ReferenceFlow::Columns,
            ReferenceFlow::RtlColumns,
            ReferenceFlow::Rows,
            ReferenceFlow::VerticalRlRows,
        ] {
            for (placement, area) in [
                (GridPlacement::try_line_span(2, 1).unwrap(), (80.0, 110.0)),
                (GridPlacement::try_line_span(2, 4).unwrap(), (80.0, 130.0)),
                (GridPlacement::try_span_line(4, 2).unwrap(), (0.0, 80.0)),
                (GridPlacement::try_line_span(-2, 4).unwrap(), (80.0, 130.0)),
                (GridPlacement::try_span_line(4, -2).unwrap(), (0.0, 80.0)),
                // An invalid anchor leaves only a span, hence Auto/Auto.
                (GridPlacement::try_line_span(4, 1).unwrap(), (0.0, 130.0)),
                (GridPlacement::try_span_line(1, -4).unwrap(), (0.0, 130.0)),
            ] {
                check_reference_case::<S>(display, flow, padded, placement, area, &mut differences);
            }
        }
    }
    assert!(differences.is_empty(), "{}", differences.join("\n"));
}

#[test]
fn absolute_reference_endpoints_f32() {
    assert_absolute_reference_endpoints::<f32>();
}

#[test]
fn absolute_reference_endpoints_f64() {
    assert_absolute_reference_endpoints::<f64>();
}

fn assert_absolute_reference_gutter_edges<S: LayoutScalar>() {
    let mut differences = Vec::new();
    // Grid 2's thick-line rule makes track 1 occupy 0..20 and track 2 30..60.
    // End references use the near edge of the interior gutter; start references
    // use its far edge. No gutter exists between the final track and Auto.
    let frame = ReferenceFrame {
        extent: 100.0,
        padded: false,
        gap: 10.0,
        lines: [0.0, 30.0, 60.0],
    };
    for display in [Display::Grid, Display::GridLanes] {
        for flow in [ReferenceFlow::Columns, ReferenceFlow::Rows] {
            for (placement, area) in [
                (GridPlacement::try_lines(1, 2).unwrap(), (0.0, 20.0)),
                (GridPlacement::try_line_span(1, 1).unwrap(), (0.0, 20.0)),
                (GridPlacement::try_line_span(2, 4).unwrap(), (30.0, 100.0)),
            ] {
                check_reference_case::<S>(display, flow, frame, placement, area, &mut differences);
            }
        }
    }
    assert!(differences.is_empty(), "{}", differences.join("\n"));
}

#[test]
fn absolute_reference_gutter_edges_f32() {
    assert_absolute_reference_gutter_edges::<f32>();
}

#[test]
fn absolute_reference_gutter_edges_f64() {
    assert_absolute_reference_gutter_edges::<f64>();
}

fn assert_absolute_auto_edge_uses_in_flow_overflow<S: LayoutScalar>() {
    let px = S::from_f64;
    let mut differences = Vec::new();
    for display in [Display::Grid, Display::GridLanes] {
        let frame = ReferenceFrame {
            extent: 100.0,
            padded: false,
            gap: 0.0,
            lines: [0.0, 20.0, 50.0],
        };
        let tree = reference_tree::<S>(
            display,
            ReferenceFlow::Columns,
            frame,
            GridPlacement::try_line(2).unwrap(),
        )
        .children(1, [5, 2, 3, 4])
        .children(5, [])
        .style(
            5,
            NodeInputOf {
                grid_column: GridPlacement::try_lines(1, 3).unwrap(),
                grid_row: GridPlacement::try_line(1).unwrap(),
                size: Size::new(px(160.0), px(10.0)).map(PreferredSizeOf::px),
                justify_self: Some(AlignItems::Start),
                ..NodeInputOf::default()
            },
        );
        let batch = match compute_layout(&tree, 1, super::fixtures::fri08_c01_placement_request()) {
            Ok(batch) => batch,
            Err(error) => {
                differences.push(format!("{display:?}: {error:?}"));
                continue;
            }
        };
        let root = fri08_c01_placement_output(&batch, 1);
        let wide = fri08_c01_placement_output(&batch, 5);
        let child = fri08_c01_placement_output(&batch, 2);
        // Grid 2 absolute positioning uses the scrollable area's padding edge
        // when the grid overflows. In-flow content independently ends at 160;
        // the valid start is 20 and the end is explicitly Auto, independently
        // of invalid-span handling, so (160-20)*50% = 70. The abspos child's
        // right edge is only 90 and cannot establish this overflow qualification.
        let actual = (
            root.size,
            root.scroll_geometry
                .map(|geometry| geometry.scrollable_overflow().size().width),
            wide.location,
            wide.size,
            child.location,
            child.size,
        );
        let expected = (
            Size::new(px(100.0), px(20.0)),
            Some(px(160.0)),
            Point::ZERO,
            Size::new(px(160.0), px(10.0)),
            Point::new(px(20.0), S::ZERO),
            Size::new(px(70.0), px(5.0)),
        );
        if actual != expected {
            differences.push(format!("{display:?}: {actual:?}, expected {expected:?}"));
        }
        for (node, x, width) in [(3, 0.0, 20.0), (4, 20.0, 30.0)] {
            let probe = fri08_c01_placement_output(&batch, node);
            if (probe.location.x, probe.size.width) != (px(x), px(width)) {
                differences.push(format!(
                    "{display:?}: in-flow probe {node}: x {:?}, width {:?}; expected {x}, {width}",
                    probe.location.x, probe.size.width
                ));
            }
        }
    }
    assert!(differences.is_empty(), "{}", differences.join("\n"));
}

#[test]
fn absolute_reference_auto_edge_uses_in_flow_overflow_f32() {
    assert_absolute_auto_edge_uses_in_flow_overflow::<f32>();
}

#[test]
fn absolute_reference_auto_edge_uses_in_flow_overflow_f64() {
    assert_absolute_auto_edge_uses_in_flow_overflow::<f64>();
}
