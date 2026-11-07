use super::*;

fn baseline_size<S: LayoutScalar>(axis: GridAxisKind, grid: f64, stacking: f64) -> Size<S> {
    let px = S::from_f64;
    match axis {
        GridAxisKind::Column => Size::new(px(grid), px(stacking)),
        GridAxisKind::Row => Size::new(px(stacking), px(grid)),
    }
}

fn baseline_point<S: LayoutScalar>(axis: GridAxisKind, grid: f64, stacking: f64) -> Point<S> {
    let size = baseline_size(axis, grid, stacking);
    Point::new(size.width, size.height)
}

fn check_baseline_box<S: LayoutScalar>(
    batch: &CompletedLayoutBatchOf<u32, S>,
    node: u32,
    expected: (Point<S>, Size<S>),
    label: &str,
    differences: &mut Vec<String>,
) {
    let output = fri08_c01_placement_output(batch, node);
    let actual = (output.location, output.size);
    if actual != expected {
        differences.push(format!(
            "{label}, node {node}: {actual:?}, expected {expected:?}"
        ));
    }
}

fn assert_stacking_container_exports_extreme_occupied_baselines<S: LayoutScalar>() {
    let px = S::from_f64;
    let mut differences = Vec::new();
    // Selected Grid 3:1310-1322 and retained October:814-818 use the highest
    // first and lowest last baseline among each track's actual boundary items.
    // Each lane has one item at packed zero. Physical border-edge synthesis
    // gives 9 and 3, so this container exports First3 and Last9.
    for (alignment, peer_height, actor_y, peer_y) in [
        (AlignItems::Baseline, 9.0, 6.0, 0.0),
        (AlignItems::LastBaseline, 3.0, 11.0, 17.0),
    ] {
        let mut tree = PublicLayoutTreeOf::<S>::new()
            .children(0, [2, 5])
            .children(2, [3, 4])
            .children(3, [])
            .children(4, [])
            .children(5, [])
            .style(
                0,
                NodeInputOf {
                    display: Display::Grid,
                    size: Size::new(px(50.0), px(20.0)).map(PreferredSizeOf::px),
                    grid_template_columns: [40.0, 10.0]
                        .map(|value| TrackComponentOf::px(px(value)))
                        .into(),
                    grid_template_rows: vec![TrackComponentOf::px(px(20.0))],
                    justify_content: Some(AlignContent::Start),
                    align_content: Some(AlignContent::Start),
                    ..NodeInputOf::default()
                },
            )
            .style(
                2,
                NodeInputOf {
                    display: Display::GridLanes,
                    size: Size::new(px(40.0), px(9.0)).map(PreferredSizeOf::px),
                    grid_lanes_flow: GridLanesFlow::new(GridAxisKind::Column, false, false),
                    grid_column: GridPlacement::try_line(1).unwrap(),
                    grid_row: GridPlacement::try_line(1).unwrap(),
                    grid_template_columns: vec![TrackComponentOf::px(px(20.0)); 2],
                    align_self: Some(alignment),
                    justify_self: Some(AlignItems::Start),
                    justify_content: Some(AlignContent::Start),
                    align_content: Some(AlignContent::Start),
                    ..NodeInputOf::default()
                },
            );
        for (node, column, width, height, self_alignment) in [
            (3, 1, 20.0, 9.0, AlignItems::Start),
            (4, 2, 20.0, 3.0, AlignItems::Start),
            (5, 2, 10.0, peer_height, alignment),
        ] {
            let size = Size::new(px(width), px(height));
            tree = tree
                .style(
                    node,
                    NodeInputOf {
                        size: size.map(PreferredSizeOf::px),
                        grid_column: GridPlacement::try_line(column).unwrap(),
                        grid_row: GridPlacement::try_line(1).unwrap(),
                        align_self: Some(self_alignment),
                        justify_self: Some(AlignItems::Start),
                        ..NodeInputOf::default()
                    },
                )
                // Public host measurement supplies only size. The Layout
                // baseline owner synthesizes the missing physical border port.
                .measure(node, size);
        }
        let batch = match compute_layout(
            &tree,
            0,
            super::super::fixtures::fri08_c01_placement_request(),
        ) {
            Ok(batch) => batch,
            Err(error) => {
                differences.push(format!("{alignment:?}: {error:?}"));
                continue;
            }
        };
        // First: target max(First3, peer9)=9 moves the actor to 9-3=6.
        // Last: both under-distances are zero, hence the row-end target20;
        // actor and peer start at 20-9=11 and 20-3=17 respectively.
        for (node, x, y, width, height) in [
            (0, 0.0, 0.0, 50.0, 20.0),
            (2, 0.0, actor_y, 40.0, 9.0),
            (3, 0.0, 0.0, 20.0, 9.0),
            (4, 20.0, 0.0, 20.0, 3.0),
            (5, 40.0, peer_y, 10.0, peer_height),
        ] {
            check_baseline_box(
                &batch,
                node,
                (Point::new(px(x), px(y)), Size::new(px(width), px(height))),
                &format!("stacking export {alignment:?}"),
                &mut differences,
            );
        }
    }
    assert!(differences.is_empty(), "{}", differences.join("\n"));
}

#[test]
fn stacking_container_exports_extreme_occupied_baselines_f32() {
    assert_stacking_container_exports_extreme_occupied_baselines::<f32>();
}

#[test]
fn stacking_container_exports_extreme_occupied_baselines_f64() {
    assert_stacking_container_exports_extreme_occupied_baselines::<f64>();
}

fn grid_axis_baseline_tree<S: LayoutScalar>(
    display: Display,
    axis: GridAxisKind,
    reference: GridBaselineReference,
    alignment: AlignItems,
) -> PublicLayoutTreeOf<S> {
    let px = S::from_f64;
    let columns = axis == GridAxisKind::Column;
    let grid_tracks = || vec![TrackComponentOf::px(px(30.0))];
    let stacking_tracks = || vec![TrackComponentOf::px(px(10.0)); 2];
    let mut tree = PublicLayoutTreeOf::new()
        .children(1, [2, 3])
        .children(2, [])
        .children(3, [])
        .style(
            1,
            NodeInputOf {
                display,
                size: baseline_size(axis, 30.0, 40.0).map(PreferredSizeOf::px),
                grid_lanes_flow: GridLanesFlow::new(axis, false, false),
                // Ordinary Grid controls use the same producer facts.
                // Lanes ignores the complementary stacking-axis template.
                grid_template_columns: if columns {
                    grid_tracks()
                } else {
                    stacking_tracks()
                },
                grid_template_rows: if columns {
                    stacking_tracks()
                } else {
                    grid_tracks()
                },
                grid_baseline_references: GridBaselineReferences {
                    columns: if columns {
                        reference
                    } else {
                        GridBaselineReference::Physical
                    },
                    rows: if columns {
                        GridBaselineReference::Physical
                    } else {
                        reference
                    },
                },
                justify_content: Some(AlignContent::Start),
                align_content: Some(AlignContent::Start),
                ..NodeInputOf::default()
            },
        );
    for (node, extent) in [(2, 3.0), (3, 9.0)] {
        tree = tree.style(
            node,
            NodeInputOf {
                // Empty native Flex proves named absence. Physical uses the
                // line-under border edge; SidewaysLr selects Right for Column
                // ports3/9. Named Central uses the midpoint independently.
                display: Display::Flex,
                writing_mode: if columns {
                    WritingMode::SidewaysLr
                } else {
                    WritingMode::HorizontalTb
                },
                size: baseline_size(axis, extent, 10.0).map(PreferredSizeOf::px),
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
                justify_self: Some(if columns {
                    alignment
                } else {
                    AlignItems::Start
                }),
                align_self: Some(if columns {
                    AlignItems::Start
                } else {
                    alignment
                }),
                ..NodeInputOf::default()
            },
        );
    }
    tree
}

fn assert_grid_axis_baselines_align_without_changing_packing<S: LayoutScalar>(named: bool) {
    let mut differences = Vec::new();
    let reference = if named {
        GridBaselineReference::Named(BaselineName::Central)
    } else {
        GridBaselineReference::Physical
    };
    // Grid 3 grid-axis alignment retains ordinary Grid baseline rules. Fixed
    // track30 and border extents3/9 give physical First target9 and Last target30.
    // Central midpoint ports1.5/4.5 give First target4.5 and Last target25.5.
    for display in [Display::Grid, Display::GridLanes] {
        for axis in [GridAxisKind::Column, GridAxisKind::Row] {
            for (alignment, short, tall) in if named {
                [
                    (AlignItems::Baseline, 3.0, 0.0),
                    (AlignItems::LastBaseline, 24.0, 21.0),
                ]
            } else {
                [
                    (AlignItems::Baseline, 6.0, 0.0),
                    (AlignItems::LastBaseline, 27.0, 21.0),
                ]
            } {
                let tree = grid_axis_baseline_tree::<S>(display, axis, reference, alignment);
                let label = format!("{display:?} {axis:?} {reference:?} {alignment:?}");
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
                for (node, grid_start, stacking_start, grid_size, stacking_size) in [
                    (1, 0.0, 0.0, 30.0, 40.0),
                    (2, short, 0.0, 3.0, 10.0),
                    (3, tall, 10.0, 9.0, 10.0),
                ] {
                    check_baseline_box(
                        &batch,
                        node,
                        (
                            baseline_point(axis, grid_start, stacking_start),
                            baseline_size(axis, grid_size, stacking_size),
                        ),
                        &label,
                        &mut differences,
                    );
                }
                for node in [2, 3] {
                    assert_eq!(
                        fri08_c01_placement_output(&batch, node).named_baselines,
                        NamedBaselinesOf::absent(),
                        "{label}, node {node}: native empty producer establishes absence"
                    );
                }
            }
        }
    }
    assert!(differences.is_empty(), "{}", differences.join("\n"));
}

#[test]
fn physical_grid_axis_baselines_preserve_packing_f32() {
    assert_grid_axis_baselines_align_without_changing_packing::<f32>(false);
}

#[test]
fn physical_grid_axis_baselines_preserve_packing_f64() {
    assert_grid_axis_baselines_align_without_changing_packing::<f64>(false);
}

#[test]
fn named_central_grid_axis_baselines_preserve_packing_f32() {
    assert_grid_axis_baselines_align_without_changing_packing::<f32>(true);
}

#[test]
fn named_central_grid_axis_baselines_preserve_packing_f64() {
    assert_grid_axis_baselines_align_without_changing_packing::<f64>(true);
}

fn assert_named_lane_baseline_controls<S: LayoutScalar>() {
    for axis in [GridAxisKind::Column, GridAxisKind::Row] {
        let named = GridBaselineReference::Named(BaselineName::Central);
        let active =
            grid_axis_baseline_tree::<S>(Display::GridLanes, axis, named, AlignItems::Baseline)
                // A host Size result supplies no named baseline fact. It must not
                // acquire the native empty Flex producer's affirmative absence.
                .measure(2, baseline_size(axis, 3.0, 10.0));
        let error = compute_layout(
            &active,
            1,
            super::super::fixtures::fri08_c01_placement_request(),
        )
        .unwrap_err();
        assert!(
            matches!(
                error.kind(),
                LayoutErrorKindOf::MissingContext(LayoutMissingContext::NamedGridBaseline)
            ),
            "{axis:?}: {error:?}"
        );
        let dormant =
            grid_axis_baseline_tree::<S>(Display::GridLanes, axis, named, AlignItems::Start)
                .measure(2, baseline_size(axis, 3.0, 10.0));
        let batch = compute_layout(
            &dormant,
            1,
            super::super::fixtures::fri08_c01_placement_request(),
        )
        .unwrap();
        for (node, extent, stacking) in [(2, 3.0, 0.0), (3, 9.0, 10.0)] {
            let output = fri08_c01_placement_output(&batch, node);
            assert_eq!(
                (output.location, output.size),
                (
                    baseline_point(axis, 0.0, stacking),
                    baseline_size(axis, extent, 10.0)
                )
            );
        }

        let mut stacking = grid_axis_baseline_tree::<S>(
            Display::GridLanes,
            axis,
            GridBaselineReference::Physical,
            AlignItems::Start,
        );
        let mut parent = LayoutTree::node_input(&stacking, 1).clone();
        let stacking_axis = match axis {
            GridAxisKind::Column => {
                parent.grid_baseline_references.rows = named;
                GridAxisKind::Row
            }
            GridAxisKind::Row => {
                parent.grid_baseline_references.columns = named;
                GridAxisKind::Column
            }
        };
        stacking = stacking.style(1, parent);
        for node in [2, 3] {
            let mut child = LayoutTree::node_input(&stacking, node).clone();
            match axis {
                GridAxisKind::Column => {
                    child.writing_mode = WritingMode::HorizontalTb;
                    child.align_self = Some(AlignItems::Baseline);
                }
                GridAxisKind::Row => {
                    child.writing_mode = WritingMode::VerticalLr;
                    child.justify_self = Some(AlignItems::Baseline);
                }
            }
            stacking = stacking.style(node, child);
        }
        let error = compute_layout(
            &stacking,
            1,
            super::super::fixtures::fri08_c01_placement_request(),
        )
        .unwrap_err();
        assert!(
            matches!(error.kind(), LayoutErrorKindOf::UnsupportedCapability(LayoutUnsupportedCapability::NamedGridLanesBaseline { axis }) if *axis == stacking_axis),
            "{axis:?}: {error:?}"
        );
    }
}

#[test]
fn named_lane_baseline_controls_f32() {
    assert_named_lane_baseline_controls::<f32>();
}

#[test]
fn named_lane_baseline_controls_f64() {
    assert_named_lane_baseline_controls::<f64>();
}
