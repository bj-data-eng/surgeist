use super::*;

fn check_conflict_tree<S: LayoutScalar>(
    tree: &PublicLayoutTreeOf<S>,
    flow: ReferenceFlow,
    frame: ReferenceFrame,
    logical_area: (f64, f64),
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
            differences.push(format!("{label}: {error:?}"));
            return;
        }
    };
    let (low, high) = logical_area;
    let physical_low = if flow.reversed() {
        frame.extent - high
    } else {
        low
    };
    let expected = (
        flow.point(physical_low, 0.0),
        flow.size((high - low) / 2.0, 5.0),
    );
    let child = fri08_c01_placement_output(&batch, 2);
    let actual = (child.location, child.size);
    if actual != expected {
        differences.push(format!("{label}: child {actual:?}, expected {expected:?}"));
    }
    let root = fri08_c01_placement_output(&batch, 1);
    if root.size != flow.size(frame.extent, 20.0) {
        differences.push(format!("{label}: root size {:?}", root.size));
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
            differences.push(format!(
                "{label}: in-flow probe {node} {actual:?}, expected {expected:?}"
            ));
        }
    }
}

fn with_raw_reference<S: LayoutScalar>(
    tree: PublicLayoutTreeOf<S>,
    flow: ReferenceFlow,
    placement: RawGridPlacement,
) -> PublicLayoutTreeOf<S> {
    let mut child = LayoutTree::node_input(&tree, 2).clone();
    match flow.axis() {
        GridAxisKind::Column => child.raw_grid_column = placement,
        GridAxisKind::Row => child.raw_grid_row = placement,
    }
    tree.style(2, child)
}

fn assert_equal_absolute_references_drop_end<S: LayoutScalar>() {
    let px = S::from_f64;
    let mut differences = Vec::new();
    let frame = ReferenceFrame {
        extent: 100.0,
        padded: false,
        gap: 0.0,
        lines: [0.0, 20.0, 50.0],
    };
    // Pinned Grid 2:3924-3934 removes the end when both references resolve to
    // one line. Absolute Auto end then uses padding end 100, not line 3 at 50.
    // The two-track explicit frame makes positive 2 and negative -2 identical.
    for display in [Display::Grid, Display::GridLanes] {
        for flow in [
            ReferenceFlow::Columns,
            ReferenceFlow::RtlColumns,
            ReferenceFlow::Rows,
            ReferenceFlow::VerticalRlRows,
        ] {
            for end in [2, -2] {
                check_reference_case::<S>(
                    display,
                    flow,
                    frame,
                    GridPlacement::try_lines(2, end).unwrap(),
                    (20.0, 100.0),
                    &mut differences,
                );
            }
            for (label, raw) in [
                ("positive", RawGridPlacement::lines(2, 2)),
                ("negative", RawGridPlacement::lines(2, -2)),
                (
                    "named",
                    RawGridPlacement::new(
                        RawGridLine::BareIdent("middle".into()),
                        RawGridLine::BareIdent("middle".into()),
                    ),
                ),
            ] {
                let tree = reference_tree::<S>(display, flow, frame, GridPlacement::AUTO);
                let mut parent = LayoutTree::node_input(&tree, 1).clone();
                let tracks = vec![
                    TrackComponentOf::px(px(20.0)),
                    TrackComponentOf::line_names(["middle"]),
                    TrackComponentOf::px(px(30.0)),
                ];
                match flow.axis() {
                    GridAxisKind::Column => parent.grid_template_columns = tracks,
                    GridAxisKind::Row => parent.grid_template_rows = tracks,
                }
                let tree = with_raw_reference(tree.style(1, parent), flow, raw);
                check_conflict_tree(
                    &tree,
                    flow,
                    frame,
                    (20.0, 100.0),
                    &format!("{display:?} {flow:?} equal {label}"),
                    &mut differences,
                );
            }
        }
    }
    assert!(differences.is_empty(), "{}", differences.join("\n"));
}

#[test]
fn equal_absolute_references_drop_end_f32() {
    assert_equal_absolute_references_drop_end::<f32>();
}

#[test]
fn equal_absolute_references_drop_end_f64() {
    assert_equal_absolute_references_drop_end::<f64>();
}

fn assert_raw_absolute_span_endpoints_qualify_independently<S: LayoutScalar>() {
    let mut differences = Vec::new();
    let frame = ReferenceFrame {
        extent: 130.0,
        padded: true,
        gap: 0.0,
        lines: [60.0, 80.0, 110.0],
    };
    // Grid 2 absolute positioning makes each non-existent reference Auto.
    // End-aligned tracks occupy 60..110 inside padding edges 0..130. Thus
    // span 4/end 2 retains end 80: the containing extent is 80, giving 40.
    for display in [Display::Grid, Display::GridLanes] {
        for flow in [
            ReferenceFlow::Columns,
            ReferenceFlow::RtlColumns,
            ReferenceFlow::Rows,
            ReferenceFlow::VerticalRlRows,
        ] {
            for (label, raw, area) in [
                (
                    "invalid span start, positive end",
                    RawGridPlacement::new(RawGridLine::Span(4), RawGridLine::Line(2)),
                    (0.0, 80.0),
                ),
                (
                    "invalid span start, negative end",
                    RawGridPlacement::new(RawGridLine::Span(4), RawGridLine::Line(-2)),
                    (0.0, 80.0),
                ),
                (
                    "valid positive start, invalid span end",
                    RawGridPlacement::new(RawGridLine::Line(2), RawGridLine::Span(4)),
                    (80.0, 130.0),
                ),
                (
                    "valid negative start, invalid span end",
                    RawGridPlacement::new(RawGridLine::Line(-2), RawGridLine::Span(4)),
                    (80.0, 130.0),
                ),
                (
                    "invalid positive start with span",
                    RawGridPlacement::new(RawGridLine::Line(4), RawGridLine::Span(1)),
                    (0.0, 130.0),
                ),
                (
                    "invalid negative end with span",
                    RawGridPlacement::new(RawGridLine::Span(1), RawGridLine::Line(-4)),
                    (0.0, 130.0),
                ),
            ] {
                let tree = with_raw_reference(
                    reference_tree::<S>(display, flow, frame, GridPlacement::AUTO),
                    flow,
                    raw,
                );
                check_conflict_tree(
                    &tree,
                    flow,
                    frame,
                    area,
                    &format!("{display:?} {flow:?} {label}"),
                    &mut differences,
                );
            }
        }
    }
    assert!(differences.is_empty(), "{}", differences.join("\n"));
}

#[test]
fn raw_absolute_span_endpoints_qualify_independently_f32() {
    assert_raw_absolute_span_endpoints_qualify_independently::<f32>();
}

#[test]
fn raw_absolute_span_endpoints_qualify_independently_f64() {
    assert_raw_absolute_span_endpoints_qualify_independently::<f64>();
}

fn assert_collapsed_absolute_reference_gutters_overlap<S: LayoutScalar>() {
    let px = S::from_f64;
    let mut differences = Vec::new();
    let frame = ReferenceFrame {
        extent: 90.0,
        padded: false,
        gap: 10.0,
        lines: [0.0, 30.0, 60.0],
    };
    // One auto-fit group initially occupies 20+20+30+2*10 = 90. Only its first
    // and third tracks have in-flow occupants, so the middle track collapses.
    // Grid 2:5023 makes both adjacent gutters coincide at 20..30. End line 3
    // uses near edge 20; start line 3 uses far edge 30. Absolute occupants do
    // not keep auto-fit tracks open, and the in-flow probes qualify 20/30 sizes.
    let display = Display::Grid;
    for flow in [
        ReferenceFlow::Columns,
        ReferenceFlow::RtlColumns,
        ReferenceFlow::Rows,
        ReferenceFlow::VerticalRlRows,
    ] {
        for (placement, area) in [
            (GridPlacement::try_lines(1, 3).unwrap(), (0.0, 20.0)),
            (GridPlacement::try_lines(3, 4).unwrap(), (30.0, 60.0)),
        ] {
            let tree = reference_tree::<S>(display, flow, frame, placement);
            let mut parent = LayoutTree::node_input(&tree, 1).clone();
            // Gap is a physical Size; the selected vertical-rl Row axis
            // is horizontal despite its logical Row name.
            parent.gap = if flow.horizontal() {
                Size::new(LengthOf::px(px(10.0)), LengthOf::ZERO)
            } else {
                Size::new(LengthOf::ZERO, LengthOf::px(px(10.0)))
            };
            let repeat = TrackComponentOf::Repeat(
                TrackRepetitionOf::auto_fit_components(
                    [20.0, 20.0, 30.0]
                        .map(|value| TrackComponentOf::px(px(value)))
                        .into(),
                )
                .unwrap(),
            );
            let mut last_probe = LayoutTree::node_input(&tree, 4).clone();
            match flow.axis() {
                GridAxisKind::Column => {
                    parent.grid_template_columns = vec![repeat];
                    last_probe.grid_column = GridPlacement::try_line(3).unwrap();
                }
                GridAxisKind::Row => {
                    parent.grid_template_rows = vec![repeat];
                    last_probe.grid_row = GridPlacement::try_line(3).unwrap();
                }
            }
            let tree = tree.style(1, parent).style(4, last_probe);
            check_conflict_tree(
                &tree,
                flow,
                frame,
                area,
                &format!("{display:?} {flow:?} collapsed {placement:?}"),
                &mut differences,
            );
        }
    }
    assert!(differences.is_empty(), "{}", differences.join("\n"));
}

#[test]
fn collapsed_absolute_reference_gutters_overlap_f32() {
    assert_collapsed_absolute_reference_gutters_overlap::<f32>();
}

#[test]
fn collapsed_absolute_reference_gutters_overlap_f64() {
    assert_collapsed_absolute_reference_gutters_overlap::<f64>();
}
