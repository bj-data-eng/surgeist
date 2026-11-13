use super::fixtures::{
    Fri04C04GridSizingValue, Fri08C02StretchTreeInput, Fri08C02TrackAxis, Fri08C06RInheritedAxes,
    SubgridChildParentContextInput, SubgridEligibilityInput,
    assert_fri08_c02_fit_content_flex_composes, baseline_measure, compute_oracle_grid,
    computed_overflow, default_grid_item_projection, empty_subgrid_track, entry_names,
    fri04_c03_grid_track_nested, fri04_c03_grid_track_percentage_nested,
    fri04_c03_grid_track_value, fri04_c04_grid_dispatch_assert_error,
    fri04_c04_grid_dispatch_style, fri05_c05_grid_sizing_input,
    fri06_c12_t08_inherited_baseline_gap_position, fri08_c01_placement_output,
    fri08_c01_topology_for_style, fri08_c02_auto_fit_output, fri08_c02_auto_fit_repeat,
    fri08_c02_flex_track, fri08_c02_stretch_track, fri08_c02_stretch_tree,
    fri08_c02_track_mix_tree, fri08_c02_track_sizes, fri08_c03_auto_fit_batch,
    fri08_c03_auto_fit_named_repeat, fri08_c04_standalone_intrinsic_minimum_width,
    fri08_c06r_assert_cold_warm, inherited_placement_member, inherited_placement_witness,
    invalid_numeric_lp, local_line_names, lp, subgrid_axis_report, subgrid_child_parent_context,
    subgrid_child_parent_context_with_geometry, subgrid_eligibility, subgrid_track,
    subgrid_track_of, traversal_leaf, vertical_baseline_measure,
    with_projected_subgrid_child_input,
};
use super::*;
use crate::test_support::readback::{self, Phase};

fn assert_explicit_lanes_flow_ignores_the_stacking_template<S: LayoutScalar>() {
    let px = S::from_f64;
    for columns in [true, false] {
        let tree = PublicLayoutTreeOf::new()
            .children(1, [2, 3])
            .style(
                1,
                NodeInputOf {
                    display: Display::GridLanes,
                    grid_lanes_flow: crate::GridLanesFlow::new(
                        if columns {
                            crate::GridAxisKind::Column
                        } else {
                            crate::GridAxisKind::Row
                        },
                        false,
                        false,
                    ),
                    grid_template_columns: if columns {
                        Vec::new()
                    } else {
                        vec![TrackComponentOf::px(px(90.0))]
                    },
                    grid_template_rows: if columns {
                        vec![TrackComponentOf::px(px(90.0))]
                    } else {
                        Vec::new()
                    },
                    grid_auto_columns: vec![TrackComponentOf::px(px(20.0))],
                    grid_auto_rows: vec![TrackComponentOf::px(px(20.0))],
                    justify_content: Some(AlignContent::Start),
                    align_content: Some(AlignContent::Start),
                    ..NodeInputOf::default()
                },
            )
            .style(
                2,
                NodeInputOf {
                    size: Size::splat(px(10.0)).map(PreferredSizeOf::px),
                    ..NodeInputOf::default()
                },
            )
            .style(
                3,
                NodeInputOf {
                    size: Size::splat(px(5.0)).map(PreferredSizeOf::px),
                    ..NodeInputOf::default()
                },
            );
        let batch =
            compute_layout(&tree, 1, super::fixtures::fri08_c01_placement_request()).unwrap();
        let first = fri08_c01_placement_output(&batch, 2);
        let second = fri08_c01_placement_output(&batch, 3);
        assert_eq!(first.location, Point::ZERO);
        assert_eq!(
            second.location,
            if columns {
                Point::new(S::ZERO, px(10.0))
            } else {
                Point::new(px(10.0), S::ZERO)
            },
            "columns {columns}"
        );
    }
}

#[test]
fn explicit_lanes_flow_ignores_the_stacking_template_f32() {
    assert_explicit_lanes_flow_ignores_the_stacking_template::<f32>();
}

#[test]
fn explicit_lanes_flow_ignores_the_stacking_template_f64() {
    assert_explicit_lanes_flow_ignores_the_stacking_template::<f64>();
}

fn child_subgrid_gap<S: LayoutScalar>(
    style: &NodeInputOf<S>,
    axis: GridAxisKind,
    area_size: Size<S>,
) -> Result<ResolvedSubgridGap<S>, crate::LengthResolutionStatus<S>> {
    super::child::child_subgrid_gap(
        &grid_container_projection!(style),
        axis,
        area_size.map(Some),
    )
}

fn needs_intrinsic_subgrid_context<Node: Copy, S: LayoutScalar>(
    style: &NodeInputOf<S>,
    item: SubgridItemReport<Node>,
    area: GridArea<S>,
) -> bool {
    super::tracks::needs_intrinsic_subgrid_context(
        &grid_container_projection!(style),
        &grid_item_projection!(style),
        item,
        area,
    )
}

fn subgrid_child_parent_context_from_ancestor_groups<
    'input,
    Node: Copy + PartialEq,
    S: LayoutScalar,
>(
    input: SubgridChildParentContextInput<'input, Node, S>,
    ancestor_baseline_groups: &FinalAncestorBaselineGroups<Node, S>,
    parent_grid: Node,
) -> Result<GridParentContext<S, Node>, SubgridChildContextError<S>> {
    with_projected_subgrid_child_input(input, |input| {
        super::child::subgrid_child_parent_context_from_ancestor_groups(
            input,
            ancestor_baseline_groups,
            parent_grid,
        )
    })
}

fn fri08_c06r_inherited_placement_leading_trailing_tree<S: LayoutScalar>(
    inherited_axes: Fri08C06RInheritedAxes,
) -> PublicLayoutTreeOf<S> {
    let scalar = S::from_f64;
    let inherited_columns = inherited_axes == Fri08C06RInheritedAxes::Columns;
    let root_columns = if inherited_columns {
        vec![TrackComponentOf::px(scalar(20.0)); 4]
    } else {
        vec![TrackComponentOf::px(scalar(100.0))]
    };
    let root_rows = if inherited_columns {
        vec![TrackComponentOf::px(scalar(100.0))]
    } else {
        vec![TrackComponentOf::px(scalar(20.0)); 4]
    };

    PublicLayoutTreeOf::new()
        .children(1, [2])
        .children(2, [3, 4])
        .style(
            1,
            NodeInputOf {
                display: Display::Grid,
                size: if inherited_columns {
                    Size::new(
                        PreferredSizeOf::px(scalar(80.0)),
                        PreferredSizeOf::px(scalar(100.0)),
                    )
                } else {
                    Size::new(
                        PreferredSizeOf::px(scalar(100.0)),
                        PreferredSizeOf::px(scalar(80.0)),
                    )
                },
                grid_template_columns: root_columns,
                grid_template_rows: root_rows,
                justify_content: Some(AlignContent::Start),
                align_content: Some(AlignContent::Start),
                ..NodeInputOf::default()
            },
        )
        .style(
            2,
            NodeInputOf {
                display: Display::Grid,
                grid_template_columns: if inherited_columns {
                    subgrid_track_of()
                } else {
                    vec![TrackComponentOf::px(scalar(40.0))]
                },
                grid_template_rows: if inherited_columns {
                    vec![TrackComponentOf::px(scalar(40.0))]
                } else {
                    subgrid_track_of()
                },
                grid_auto_columns: vec![
                    TrackComponentOf::px(scalar(10.0)),
                    TrackComponentOf::px(scalar(20.0)),
                ],
                grid_auto_rows: vec![
                    TrackComponentOf::px(scalar(10.0)),
                    TrackComponentOf::px(scalar(20.0)),
                ],
                grid_column: if inherited_columns {
                    GridPlacement::try_line_span(1, 4).expect("four inherited columns")
                } else {
                    GridPlacement::try_line(1).expect("standalone column")
                },
                grid_row: if inherited_columns {
                    GridPlacement::try_line(1).expect("standalone row")
                } else {
                    GridPlacement::try_line_span(1, 4).expect("four inherited rows")
                },
                grid_auto_flow: if inherited_columns {
                    GridAutoFlow::Row
                } else {
                    GridAutoFlow::Column
                },
                justify_content: Some(AlignContent::Start),
                align_content: Some(AlignContent::Start),
                ..NodeInputOf::default()
            },
        )
        .style(
            3,
            NodeInputOf {
                grid_column: if inherited_columns {
                    GridPlacement::try_line(1).expect("first inherited column")
                } else {
                    GridPlacement::try_line(-3).expect("leading implicit column")
                },
                grid_row: if inherited_columns {
                    GridPlacement::try_line(-3).expect("leading implicit row")
                } else {
                    GridPlacement::try_line(1).expect("first inherited row")
                },
                ..NodeInputOf::default()
            },
        )
        .style(
            4,
            NodeInputOf {
                grid_column: if inherited_columns {
                    GridPlacement::try_line(2).expect("second inherited column")
                } else {
                    GridPlacement::try_line(3).expect("trailing implicit column")
                },
                grid_row: if inherited_columns {
                    GridPlacement::try_line(3).expect("trailing implicit row")
                } else {
                    GridPlacement::try_line(2).expect("second inherited row")
                },
                ..NodeInputOf::default()
            },
        )
}

fn assert_fri08_c06r_inherited_placement_leading_trailing<S: LayoutScalar>() {
    let scalar = S::from_f64;
    for inherited_axes in [
        Fri08C06RInheritedAxes::Columns,
        Fri08C06RInheritedAxes::Rows,
    ] {
        let tree = fri08_c06r_inherited_placement_leading_trailing_tree(inherited_axes);
        fri08_c06r_assert_cold_warm(tree, &[1, 2, 3, 4], |batch| {
            let leading = fri08_c01_placement_output(batch, 3);
            let trailing = fri08_c01_placement_output(batch, 4);
            if inherited_axes == Fri08C06RInheritedAxes::Columns {
                assert_eq!(leading.location, Point::ZERO);
                assert_eq!(leading.size, Size::new(scalar(20.0), scalar(20.0)));
                assert_eq!(trailing.location, Point::new(scalar(20.0), scalar(70.0)));
                assert_eq!(trailing.size, Size::new(scalar(20.0), scalar(20.0)));
            } else {
                assert_eq!(leading.location, Point::ZERO);
                assert_eq!(leading.size, Size::new(scalar(20.0), scalar(20.0)));
                assert_eq!(trailing.location, Point::new(scalar(70.0), scalar(20.0)));
                assert_eq!(trailing.size, Size::new(scalar(20.0), scalar(20.0)));
            }
        });
    }
}

#[test]
fn fri08_c06r_inherited_placement_standalone_axes_preserve_leading_trailing_pattern_phase() {
    assert_fri08_c06r_inherited_placement_leading_trailing::<f32>();
    assert_fri08_c06r_inherited_placement_leading_trailing::<f64>();
}

#[test]
fn fri08_c03_auto_fit_ordinary_grid_and_lanes_auto_fill_remain_separate_controls() {
    for (display, repeat, expected_x) in [
        (Display::Grid, TrackRepeat::AutoFit, 40.0),
        (Display::GridLanes, TrackRepeat::AutoFill, 0.0),
    ] {
        let tree = PublicLayoutTreeOf::<f32>::new().children(1, [2, 3]).style(
            1,
            NodeInput {
                display,
                size: Size::new(PreferredSize::px(120.0), PreferredSize::px(20.0)),
                grid_template_columns: vec![fri08_c03_auto_fit_named_repeat(repeat)],
                grid_template_rows: vec![TrackComponent::px(20.0)],
                justify_content: Some(AlignContent::Center),
                ..NodeInput::DEFAULT
            },
        );
        let tree = [2, 3].into_iter().fold(tree, |tree, node| {
            tree.style(
                node,
                NodeInput {
                    grid_column: GridPlacement::try_line(1).expect("first repeated track"),
                    grid_row: GridPlacement::try_line(1).expect("single row"),
                    ..NodeInput::DEFAULT
                },
            )
        });
        let batch = fri08_c03_auto_fit_batch(&tree, Size::new(120.0, 20.0));
        let output = fri08_c01_placement_output(&batch, 2);
        assert_eq!((output.location.x, output.size.width), (expected_x, 40.0));
    }
}

#[test]
fn fri08_c03_auto_fit_lanes_flexible_tracks_and_stretched_items_ignore_collapsed_gutters_on_both_axes()
 {
    let mut track_geometries = Vec::new();
    for axis in [Fri08C02TrackAxis::Columns, Fri08C02TrackAxis::Rows] {
        let repeat = TrackComponent::Repeat(
            TrackRepetition::auto_fit_components(vec![TrackComponent::minmax(
                MinTrackSizing::px(40.0),
                MaxTrackSizing::flex(
                    TrackFlexFactor::try_new(1.0).expect("finite auto-fit flex factor"),
                ),
            )])
            .expect("valid flexible auto-fit repetition"),
        );
        let (columns, rows, auto_flow, size, gap, child_placement) = match axis {
            Fri08C02TrackAxis::Columns => (
                vec![repeat],
                vec![TrackComponent::px(20.0)],
                GridAutoFlow::Row,
                Size::new(140.0, 20.0),
                Size::new(Length::px(10.0), Length::ZERO),
                (
                    GridPlacement::try_line(1).expect("first repeated column"),
                    GridPlacement::try_line(1).expect("single row"),
                ),
            ),
            Fri08C02TrackAxis::Rows => (
                vec![TrackComponent::px(20.0)],
                vec![repeat],
                GridAutoFlow::Column,
                Size::new(20.0, 140.0),
                Size::new(Length::ZERO, Length::px(10.0)),
                (
                    GridPlacement::try_line(1).expect("single column"),
                    GridPlacement::try_line(1).expect("first repeated row"),
                ),
            ),
        };
        let tree = PublicLayoutTreeOf::<f32>::new()
            .children(1, [2])
            .style(
                1,
                NodeInput {
                    display: Display::GridLanes,
                    size: size.map(PreferredSize::px),
                    grid_template_columns: columns,
                    grid_template_rows: rows,
                    grid_lanes_flow: crate::GridLanesFlow::new(
                        if auto_flow.is_column() {
                            crate::GridAxisKind::Row
                        } else {
                            crate::GridAxisKind::Column
                        },
                        false,
                        auto_flow.is_dense(),
                    ),
                    gap,
                    justify_content: Some(AlignContent::Start),
                    align_content: Some(AlignContent::Start),
                    ..NodeInput::DEFAULT
                },
            )
            .style(
                2,
                NodeInput {
                    grid_column: child_placement.0,
                    grid_row: child_placement.1,
                    ..NodeInput::DEFAULT
                },
            );

        let batch = fri08_c03_auto_fit_batch(&tree, size);
        let child = fri08_c01_placement_output(&batch, 2);
        let track_geometry = match axis {
            Fri08C02TrackAxis::Columns => (child.location.x, child.size.width),
            Fri08C02TrackAxis::Rows => (child.location.y, child.size.height),
        };
        track_geometries.push(track_geometry);
    }
    assert_eq!(track_geometries, [(0.0, 140.0), (0.0, 140.0)]);
}

#[test]
fn fri08_c04_standalone_ordinary_one_axis_subgrid_keeps_automatic_minimum() {
    assert_eq!(
        fri08_c04_standalone_intrinsic_minimum_width(MinSize::AUTO),
        20.0
    );
}

#[test]
fn fri08_c02_stretch_lanes_policy_accepts_c03_auto_fit_control() {
    let tree = PublicLayoutTreeOf::<f32>::new()
        .children(1, [2, 3])
        .style(
            1,
            NodeInput {
                display: Display::GridLanes,
                size: Size::new(PreferredSize::px(120.0), PreferredSize::px(20.0)),
                grid_template_columns: vec![fri08_c02_auto_fit_repeat()],
                grid_template_rows: vec![TrackComponent::px(20.0)],
                justify_content: Some(AlignContent::Center),
                ..NodeInput::DEFAULT
            },
        )
        .style(
            2,
            NodeInput {
                grid_column: GridPlacement::try_line(1).expect("first lane track"),
                ..NodeInput::DEFAULT
            },
        )
        .style(
            3,
            NodeInput {
                grid_column: GridPlacement::try_line(1).expect("overlapping lane track"),
                ..NodeInput::DEFAULT
            },
        );
    let output = fri08_c02_auto_fit_output(&tree, Size::new(120.0, 20.0), 2);
    assert_eq!((output.location.x, output.size.width), (40.0, 40.0));
}

#[test]
fn fri08_c02_fit_content_vertical_sideways_and_scalar_lanes_project_the_same_sizes() {
    for writing_mode in [
        WritingMode::VerticalRl,
        WritingMode::VerticalLr,
        WritingMode::SidewaysRl,
        WritingMode::SidewaysLr,
    ] {
        assert_fri08_c02_fit_content_flex_composes::<f32>(Fri08C02TrackAxis::Columns, writing_mode);
        assert_fri08_c02_fit_content_flex_composes::<f32>(Fri08C02TrackAxis::Rows, writing_mode);
        assert_fri08_c02_fit_content_flex_composes::<f64>(Fri08C02TrackAxis::Columns, writing_mode);
        assert_fri08_c02_fit_content_flex_composes::<f64>(Fri08C02TrackAxis::Rows, writing_mode);
    }
}

fn assert_fri08_c02r_lanes_fit_content_flex<S: LayoutScalar>() {
    for writing_mode in [
        WritingMode::HorizontalTb,
        WritingMode::VerticalRl,
        WritingMode::SidewaysLr,
    ] {
        for axis in [Fri08C02TrackAxis::Columns, Fri08C02TrackAxis::Rows] {
            for (fit_limit, basis, intrinsic_bases, expected) in [
                (50.0, 200.0, [20.0, 0.0], [20.0, 180.0]),
                (20.0, 100.0, [30.0, 0.0], [30.0, 70.0]),
            ] {
                let (tree, flow_axes, viewport) = fri08_c02_track_mix_tree(
                    Display::GridLanes,
                    axis,
                    writing_mode,
                    (fit_limit, 0.0),
                    Some(basis),
                    vec![fri08_c02_flex_track::<S>(1.0)],
                    &intrinsic_bases,
                );
                let sizes = fri08_c02_track_sizes(&tree, flow_axes, viewport, axis, 2);
                assert_eq!(
                    sizes,
                    expected.map(S::from_f64),
                    "{writing_mode:?} {axis:?} fit-content and flex must execute one phase pipeline"
                );
            }
        }
    }
}

#[test]
fn fri08_c02r_lanes_track_phase_fit_content_and_flex_compose_across_axes_flows_and_scalars() {
    assert_fri08_c02r_lanes_fit_content_flex::<f32>();
    assert_fri08_c02r_lanes_fit_content_flex::<f64>();
}

fn assert_fri08_c02r_lanes_auto_max_stretch<S: LayoutScalar>() {
    for writing_mode in [
        WritingMode::HorizontalTb,
        WritingMode::VerticalRl,
        WritingMode::SidewaysLr,
    ] {
        for axis in [Fri08C02TrackAxis::Columns, Fri08C02TrackAxis::Rows] {
            for alignment in [None, Some(AlignContent::Stretch)] {
                let (tree, flow_axes, viewport) =
                    fri08_c02_stretch_tree(Fri08C02StretchTreeInput {
                        display: Display::GridLanes,
                        axis,
                        writing_mode,
                        definite_axis_size: Some(100.0),
                        viewport_axis_size: 100.0,
                        gap: 0.0,
                        alignment,
                        tracks: vec![fri08_c02_stretch_track(MinTrackSizingOf::px(S::ZERO))],
                        measurements: &[0.0],
                    });
                assert_eq!(
                    fri08_c02_track_sizes(&tree, flow_axes, viewport, axis, 1),
                    [S::from_f64(100.0)],
                    "{writing_mode:?} {axis:?} {alignment:?} minmax(0,auto) stretches"
                );
            }

            let (tree, flow_axes, viewport) = fri08_c02_stretch_tree(Fri08C02StretchTreeInput {
                display: Display::GridLanes,
                axis,
                writing_mode,
                definite_axis_size: Some(100.0),
                viewport_axis_size: 100.0,
                gap: 0.0,
                alignment: Some(AlignContent::Stretch),
                tracks: vec![
                    fri08_c02_stretch_track(MinTrackSizingOf::<S>::MIN_CONTENT),
                    fri08_c02_stretch_track(MinTrackSizingOf::<S>::MAX_CONTENT),
                ],
                measurements: &[20.0, 30.0],
            });
            assert_eq!(
                fri08_c02_track_sizes(&tree, flow_axes, viewport, axis, 2),
                [S::from_f64(45.0), S::from_f64(55.0)],
                "{writing_mode:?} {axis:?} intrinsic floors receive the same remainder"
            );
        }
    }
}

#[test]
fn fri08_c02r_lanes_track_phase_auto_max_stretch_preserves_floors_across_axes_flows_and_scalars() {
    assert_fri08_c02r_lanes_auto_max_stretch::<f32>();
    assert_fri08_c02r_lanes_auto_max_stretch::<f64>();
}

fn assert_fri08_c02r_lanes_stretch_exclusions<S: LayoutScalar>() {
    for writing_mode in [
        WritingMode::HorizontalTb,
        WritingMode::VerticalRl,
        WritingMode::SidewaysLr,
    ] {
        for axis in [Fri08C02TrackAxis::Columns, Fri08C02TrackAxis::Rows] {
            let resolve = |track, definite_axis_size, alignment, measurement| {
                let (tree, flow_axes, viewport) =
                    fri08_c02_stretch_tree(Fri08C02StretchTreeInput {
                        display: Display::GridLanes,
                        axis,
                        writing_mode,
                        definite_axis_size,
                        viewport_axis_size: 100.0,
                        gap: 0.0,
                        alignment,
                        tracks: vec![track],
                        measurements: &[measurement],
                    });
                fri08_c02_track_sizes(&tree, flow_axes, viewport, axis, 1)[0]
            };

            assert_eq!(
                resolve(
                    TrackComponentOf::minmax(
                        MinTrackSizingOf::px(S::ZERO),
                        MaxTrackSizingOf::MAX_CONTENT,
                    ),
                    Some(100.0),
                    Some(AlignContent::Stretch),
                    20.0,
                ),
                S::from_f64(20.0),
                "{writing_mode:?} {axis:?} a non-auto maximum is ineligible"
            );
            assert_eq!(
                resolve(
                    fri08_c02_stretch_track(MinTrackSizingOf::MIN_CONTENT),
                    Some(100.0),
                    Some(AlignContent::Start),
                    20.0,
                ),
                S::from_f64(20.0),
                "{writing_mode:?} {axis:?} start alignment does not stretch"
            );

            let (indefinite_tree, indefinite_flow_axes, _) =
                fri08_c02_stretch_tree(Fri08C02StretchTreeInput {
                    display: Display::GridLanes,
                    axis,
                    writing_mode,
                    definite_axis_size: None,
                    viewport_axis_size: 100.0,
                    gap: 0.0,
                    alignment: Some(AlignContent::Stretch),
                    tracks: vec![fri08_c02_stretch_track(MinTrackSizingOf::<S>::MIN_CONTENT)],
                    measurements: &[20.0],
                });
            let indefinite_batch = compute_layout(&indefinite_tree, 1, {
                let axes = Size::splat(AxisSizingRequestOf::new(
                    AvailableSpaceOf::Indefinite,
                    SizingOperation::MaxContent,
                ));
                let definite = axes.map(|axis| axis.space().definite_size());
                crate::LayoutRootRequestOf::standalone(
                    SizingRequestOf::new(
                        axes,
                        OrthogonalFallbackOf::new(
                            definite,
                            crate::NearestScrollportOf::Absent,
                            Size::splat(crate::OrthogonalFallbackLimitOf::Unbounded),
                        ),
                    ),
                    crate::geometry::FlowAxes::new(
                        crate::WritingMode::HorizontalTb,
                        crate::Direction::Ltr,
                    ),
                    definite,
                )
            })
            .expect("indefinite lanes track sizing completes");
            let indefinite_size = indefinite_flow_axes
                .logical_size(fri08_c01_placement_output(&indefinite_batch, 2).size);
            assert_eq!(
                match axis {
                    Fri08C02TrackAxis::Columns => indefinite_size.inline,
                    Fri08C02TrackAxis::Rows => indefinite_size.block,
                },
                S::from_f64(20.0),
                "{writing_mode:?} {axis:?} indefinite space does not stretch"
            );
            assert_eq!(
                resolve(
                    fri08_c02_stretch_track(MinTrackSizingOf::MIN_CONTENT),
                    Some(10.0),
                    Some(AlignContent::Stretch),
                    20.0,
                ),
                S::from_f64(20.0),
                "{writing_mode:?} {axis:?} a non-positive remainder preserves the floor"
            );

            let (collapsed_tree, collapsed_flow_axes, collapsed_viewport) =
                fri08_c02_stretch_tree(Fri08C02StretchTreeInput {
                    display: Display::GridLanes,
                    axis,
                    writing_mode,
                    definite_axis_size: Some(120.0),
                    viewport_axis_size: 120.0,
                    gap: 0.0,
                    alignment: Some(AlignContent::Stretch),
                    tracks: vec![fri08_c02_auto_fit_repeat::<S>()],
                    measurements: &[0.0],
                });
            assert_eq!(
                fri08_c02_track_sizes(
                    &collapsed_tree,
                    collapsed_flow_axes,
                    collapsed_viewport,
                    axis,
                    1,
                ),
                [S::from_f64(40.0)],
                "{writing_mode:?} {axis:?} collapsed auto-fit tracks stay zero under stretch"
            );
        }
    }
}

#[test]
fn fri08_c02r_lanes_track_phase_stretch_excludes_non_auto_collapsed_start_indefinite_and_non_positive()
 {
    assert_fri08_c02r_lanes_stretch_exclusions::<f32>();
    assert_fri08_c02r_lanes_stretch_exclusions::<f64>();
}

#[test]
fn fri08_c02_lanes_negative_auto_fit_accepts_c03_preplacement_collapse() {
    let tree = PublicLayoutTreeOf::<f32>::new()
        .children(1, [2, 3])
        .style(
            1,
            NodeInput {
                display: Display::GridLanes,
                size: Size::new(PreferredSize::px(120.0), PreferredSize::px(20.0)),
                grid_template_columns: vec![fri08_c02_auto_fit_repeat()],
                grid_template_rows: vec![TrackComponent::px(20.0)],
                justify_content: Some(AlignContent::Center),
                ..NodeInput::DEFAULT
            },
        )
        .style(
            2,
            NodeInput {
                grid_column: GridPlacement::try_line(1).expect("first lane track"),
                ..NodeInput::DEFAULT
            },
        )
        .style(
            3,
            NodeInput {
                grid_column: GridPlacement::try_line(1).expect("overlapping lane track"),
                ..NodeInput::DEFAULT
            },
        );
    let output = fri08_c02_auto_fit_output(&tree, Size::new(120.0, 20.0), 2);
    assert_eq!((output.location.x, output.size.width), (40.0, 40.0));
}

fn assert_fri08_c06_collapsed_gutter_grid_lanes_interior_collapse<S: LayoutScalar>() {
    let scalar = S::from_f64;
    let tree = PublicLayoutTreeOf::<S>::new()
        .children(1, [2, 3])
        .style(
            1,
            NodeInputOf::<S> {
                display: Display::GridLanes,
                size: Size::new(
                    PreferredSizeOf::<S>::px(scalar(190.0)),
                    PreferredSizeOf::<S>::px(scalar(20.0)),
                ),
                grid_template_columns: vec![fri08_c02_auto_fit_repeat()],
                grid_template_rows: vec![TrackComponentOf::<S>::px(scalar(20.0))],
                gap: Size::new(LengthOf::<S>::px(scalar(10.0)), LengthOf::<S>::ZERO),
                justify_content: Some(AlignContent::Start),
                ..NodeInputOf::<S>::default()
            },
        )
        .style(2, NodeInputOf::<S>::default())
        .style(
            3,
            NodeInputOf::<S> {
                grid_column: GridPlacement::try_line(4).expect("fourth lane track"),
                ..NodeInputOf::<S>::default()
            },
        );

    let automatic = fri08_c02_auto_fit_output(&tree, Size::new(scalar(190.0), scalar(20.0)), 2);
    let definite = fri08_c02_auto_fit_output(&tree, Size::new(scalar(190.0), scalar(20.0)), 3);
    assert_eq!(
        (automatic.location.x, automatic.size.width),
        (S::ZERO, scalar(40.0))
    );
    assert_eq!(
        (definite.location.x, definite.size.width),
        (scalar(40.0), scalar(40.0))
    );
    assert_eq!(
        definite.location.x - automatic.location.x - automatic.size.width,
        S::ZERO,
        "grid-lanes retains zero active-gap total when the two interior tracks collapse",
    );
}

#[test]
fn fri08_c06_collapsed_gutter_grid_lanes_interior_collapse_keeps_zero_active_gap_total() {
    assert_fri08_c06_collapsed_gutter_grid_lanes_interior_collapse::<f32>();
    assert_fri08_c06_collapsed_gutter_grid_lanes_interior_collapse::<f64>();
}

fn assert_fri08_c06_collapsed_gutter_inherited_grid_lanes_space_between<S: LayoutScalar>()
where
    OracleTreeOf<S>: Traverse<Node = u32, Scalar = S>,
{
    let scalar = S::from_f64;
    for reversed in [false, true] {
        let parent_style = NodeInputOf::<S> {
            display: Display::Grid,
            ..NodeInputOf::<S>::default()
        };
        let child_style = NodeInputOf::<S> {
            display: Display::GridLanes,
            direction: if reversed {
                Direction::Rtl
            } else {
                Direction::Ltr
            },
            size: Size::new(
                PreferredSizeOf::px(scalar(190.0)),
                PreferredSizeOf::px(scalar(20.0)),
            ),
            grid_template_columns: vec![TrackComponentOf::Subgrid(SubgridTrack {
                name_components: Vec::new(),
            })],
            grid_template_rows: vec![TrackComponentOf::px(scalar(20.0))],
            gap: Size::new(LengthOf::px(scalar(10.0)), LengthOf::ZERO),
            justify_content: Some(AlignContent::SpaceBetween),
            ..NodeInputOf::<S>::default()
        };
        let parent_gutters = OrdinaryGridAxisGuttersOf::new_zero_adjacent_to_collapsed_tracks(
            4,
            &[false, true, true, false],
            scalar(10.0),
        );
        let parent_geometry = UsedGridAxisGeometryOf::from_active_boundary_gutters(
            vec![scalar(40.0), S::ZERO, S::ZERO, scalar(40.0)],
            parent_gutters.collapsed().to_vec(),
            parent_gutters.active_boundary_after().to_vec(),
            parent_gutters.gutter_after().to_vec(),
        );
        let item = SubgridItemReport {
            node: 1_u32,
            column: subgrid_axis_report(&parent_style, &child_style, GridAxisKind::Column),
            row: subgrid_axis_report(&parent_style, &child_style, GridAxisKind::Row),
        };
        let mut parent_column_functions = Vec::new();
        let mut parent_row_functions = Vec::new();
        extend_auto_tracks(
            &mut parent_column_functions,
            &parent_style.grid_auto_columns,
            None,
            scalar(10.0),
            4,
        )
        .unwrap();
        extend_auto_tracks(
            &mut parent_row_functions,
            &parent_style.grid_auto_rows,
            None,
            S::ZERO,
            1,
        )
        .unwrap();
        let context = subgrid_child_parent_context_with_geometry(
            SubgridChildParentContextInput {
                minimum_functions: GridMinimumParentFunctions {
                    columns: Some(GridMinimumTrackFunctions::new(
                        &parent_column_functions,
                        None,
                    )),
                    rows: Some(GridMinimumTrackFunctions::new(&parent_row_functions, None)),
                },
                item,
                child_style: &child_style,
                area: GridArea {
                    column: 0,
                    row: 0,
                    column_end: 4,
                    row_end: 1,
                    size: LogicalSizeOf::new(scalar(190.0), scalar(20.0)),
                },
                content_box_size: Size::new(scalar(190.0), scalar(20.0)),
                columns: parent_geometry.sizes(),
                rows: &[scalar(20.0)],
                gap: LogicalSizeOf::new(scalar(10.0), S::ZERO),
                parent_named_columns: &NamedGridLines::new(GridAxisKind::Column, 4),
                parent_named_rows: &NamedGridLines::new(GridAxisKind::Row, 1),
                parent_area_facts: None,
                margin: Edges::all(Some(S::ZERO)),
                border: Edges::all(S::ZERO),
                padding: Edges::all(S::ZERO),
            },
            Some(&parent_geometry),
            None,
        )
        .expect("collapsed grid-lanes gutter policy remains inheritable");

        let mut tree = OracleTreeOf::<S>::new()
            .children(1, [2, 3])
            .style(1, child_style)
            .style(
                2,
                NodeInputOf::<S> {
                    size: Size::new(
                        PreferredSizeOf::px(scalar(40.0)),
                        PreferredSizeOf::px(scalar(20.0)),
                    ),
                    grid_column: GridPlacement::try_line(1).expect("first inherited track"),
                    ..NodeInputOf::<S>::default()
                },
            )
            .style(
                3,
                NodeInputOf::<S> {
                    size: Size::new(
                        PreferredSizeOf::px(scalar(40.0)),
                        PreferredSizeOf::px(scalar(20.0)),
                    ),
                    grid_column: GridPlacement::try_line(4).expect("last inherited track"),
                    ..NodeInputOf::<S>::default()
                },
            );
        compute_grid_with_context(
            &mut tree.borrow_compute(),
            1,
            ComputeInputOf::for_child(
                RunMode::PerformLayout,
                SizingMode::InherentSize,
                RequestedAxis::Both,
                Size::new(Some(scalar(190.0)), Some(scalar(20.0))),
                Size::new(Some(scalar(190.0)), Some(scalar(20.0))),
                ContainingLayoutContext::new(
                    FlowAxes::new(WritingMode::HorizontalTb, Direction::Ltr),
                    ParentFormattingContext::Grid,
                ),
                SizingRequestOf::new(
                    Size::new(
                        AxisSizingRequestOf::new(
                            AvailableSpaceOf::Definite(
                                crate::NonNegativeFiniteOf::new(scalar(190.0)).unwrap(),
                            ),
                            SizingOperation::Normal,
                        ),
                        AxisSizingRequestOf::new(
                            AvailableSpaceOf::Definite(
                                crate::NonNegativeFiniteOf::new(scalar(20.0)).unwrap(),
                            ),
                            SizingOperation::Normal,
                        ),
                    ),
                    OrthogonalFallbackOf::new(
                        Size::NONE,
                        crate::NearestScrollportOf::Absent,
                        Size::splat(crate::OrthogonalFallbackLimitOf::Unbounded),
                    ),
                ),
            ),
            context,
        )
        .expect("inherited grid-lanes layout");

        let first = tree.layout(2).expect("first inherited-track child");
        let last = tree.layout(3).expect("last inherited-track child");
        let recreated_gap = if reversed {
            first.location.x - last.location.x - last.size.width
        } else {
            last.location.x - first.location.x - first.size.width
        };
        assert_eq!(
            recreated_gap,
            S::ZERO,
            "{reversed:?} inherited grid-lanes SpaceBetween must not reactivate a boundary adjacent to collapsed tracks",
        );
    }
}

#[test]
fn fri08_c06_collapsed_gutter_inherited_grid_lanes_space_between_keeps_zero_gap() {
    assert_fri08_c06_collapsed_gutter_inherited_grid_lanes_space_between::<f32>();
    assert_fri08_c06_collapsed_gutter_inherited_grid_lanes_space_between::<f64>();
}

#[test]
fn fri08_c02_lanes_negative_stretch_retains_pre_c02_auto_track_geometry() {
    let tree = PublicLayoutTreeOf::<f32>::new()
        .children(1, [2])
        .style(
            1,
            NodeInput {
                display: Display::GridLanes,
                size: Size::new(PreferredSize::px(100.0), PreferredSize::px(20.0)),
                grid_template_columns: vec![TrackComponent::AUTO],
                grid_template_rows: vec![TrackComponent::px(20.0)],
                justify_content: Some(AlignContent::Stretch),
                ..NodeInput::DEFAULT
            },
        )
        .style(2, NodeInput::DEFAULT);
    let output = fri08_c02_auto_fit_output(&tree, Size::new(100.0, 20.0), 2);
    assert_eq!((output.location.x, output.size.width), (0.0, 100.0));
}

fn assert_fri08_c02_auto_fit_overlap_collapses_to_centered_track<S: LayoutScalar>(
    axis: Fri08C02TrackAxis,
) {
    let scalar = S::from_f64;
    let (size, columns, rows) = match axis {
        Fri08C02TrackAxis::Columns => (
            Size::new(
                PreferredSizeOf::px(scalar(120.0)),
                PreferredSizeOf::px(scalar(20.0)),
            ),
            vec![fri08_c02_auto_fit_repeat()],
            vec![TrackComponentOf::px(scalar(20.0))],
        ),
        Fri08C02TrackAxis::Rows => (
            Size::new(
                PreferredSizeOf::px(scalar(20.0)),
                PreferredSizeOf::px(scalar(120.0)),
            ),
            vec![TrackComponentOf::px(scalar(20.0))],
            vec![fri08_c02_auto_fit_repeat()],
        ),
    };
    let first_line = GridPlacement::try_line(1).expect("first repeated track");
    let mut tree = PublicLayoutTreeOf::new().children(1, [2, 3]).style(
        1,
        NodeInputOf {
            display: Display::Grid,
            size,
            grid_template_columns: columns,
            grid_template_rows: rows,
            justify_content: Some(AlignContent::Center),
            align_content: Some(AlignContent::Center),
            ..NodeInputOf::default()
        },
    );
    for node in [2, 3] {
        let (grid_column, grid_row) = match axis {
            Fri08C02TrackAxis::Columns => {
                (first_line, GridPlacement::try_line(1).expect("single row"))
            }
            Fri08C02TrackAxis::Rows => (
                GridPlacement::try_line(1).expect("single column"),
                first_line,
            ),
        };
        tree = tree.style(
            node,
            NodeInputOf {
                grid_column,
                grid_row,
                ..NodeInputOf::default()
            },
        );
    }
    let viewport = match axis {
        Fri08C02TrackAxis::Columns => Size::new(scalar(120.0), scalar(20.0)),
        Fri08C02TrackAxis::Rows => Size::new(scalar(20.0), scalar(120.0)),
    };
    let output = fri08_c02_auto_fit_output(&tree, viewport, 2);
    match axis {
        Fri08C02TrackAxis::Columns => {
            assert_eq!(output.location.x, scalar(40.0));
            assert_eq!(output.size.width, scalar(40.0));
        }
        Fri08C02TrackAxis::Rows => {
            assert_eq!(output.location.y, scalar(40.0));
            assert_eq!(output.size.height, scalar(40.0));
        }
    }
}

#[test]
fn fri08_c02_auto_fit_overlap_collapses_columns_and_rows_in_both_scalar_lanes() {
    assert_fri08_c02_auto_fit_overlap_collapses_to_centered_track::<f32>(
        Fri08C02TrackAxis::Columns,
    );
    assert_fri08_c02_auto_fit_overlap_collapses_to_centered_track::<f32>(Fri08C02TrackAxis::Rows);
    assert_fri08_c02_auto_fit_overlap_collapses_to_centered_track::<f64>(
        Fri08C02TrackAxis::Columns,
    );
    assert_fri08_c02_auto_fit_overlap_collapses_to_centered_track::<f64>(Fri08C02TrackAxis::Rows);
}

#[test]
fn fri08_c02_auto_fit_public_active_subgrid_preserves_space_between_boundary_geometry() {
    let repeat = TrackComponent::Repeat(
        TrackRepetition::auto_fit_components(vec![TrackComponent::px(100.0)])
            .expect("valid fixed auto-fit repetition"),
    );
    let tree = PublicLayoutTreeOf::<f32>::new()
        .children(1, [2])
        .children(2, [3, 4])
        .style(
            1,
            NodeInput {
                display: Display::Grid,
                size: Size::new(PreferredSize::px(500.0), PreferredSize::px(40.0)),
                grid_template_columns: vec![repeat],
                grid_template_rows: vec![TrackComponent::px(40.0)],
                justify_content: Some(AlignContent::SpaceBetween),
                ..NodeInput::DEFAULT
            },
        )
        .style(
            2,
            NodeInput {
                display: Display::Grid,
                grid_column: GridPlacement::try_lines(1, 4).expect("all active parent tracks"),
                grid_row: GridPlacement::try_line(1).expect("single parent row"),
                grid_template_columns: vec![empty_subgrid_track()],
                grid_template_rows: vec![TrackComponent::px(40.0)],
                gap: Size::new(Length::px(250.0), Length::ZERO),
                overflow: computed_overflow(Overflow::Auto, Overflow::Scroll),
                ..NodeInput::DEFAULT
            },
        )
        .style(
            3,
            NodeInput {
                size: Size::new(PreferredSize::px(20.0), PreferredSize::px(10.0)),
                grid_column: GridPlacement::try_line(2).expect("second inherited track"),
                grid_row: GridPlacement::try_line(1).expect("single inherited row"),
                justify_self: Some(AlignItems::Start),
                ..NodeInput::DEFAULT
            },
        )
        .style(
            4,
            NodeInput {
                size: Size::new(PreferredSize::px(100.0), PreferredSize::px(10.0)),
                grid_column: GridPlacement::try_line(3).expect("third inherited track"),
                grid_row: GridPlacement::try_line(1).expect("single inherited row"),
                justify_self: Some(AlignItems::Start),
                ..NodeInput::DEFAULT
            },
        );

    let batch = compute_layout(&tree, 1, {
        let axes = Size::new(
            AxisSizingRequestOf::new(
                AvailableSpaceOf::Definite(crate::NonNegativeFiniteOf::new(500.0).unwrap()),
                SizingOperation::Normal,
            ),
            AxisSizingRequestOf::new(
                AvailableSpaceOf::Definite(crate::NonNegativeFiniteOf::new(40.0).unwrap()),
                SizingOperation::Normal,
            ),
        );
        let definite = axes.map(|axis| axis.space().definite_size());
        crate::LayoutRootRequest::standalone(
            SizingRequestOf::new(
                axes,
                OrthogonalFallbackOf::new(
                    definite,
                    crate::NearestScrollportOf::Absent,
                    Size::splat(crate::OrthogonalFallbackLimitOf::Unbounded),
                ),
            ),
            crate::geometry::FlowAxes::new(crate::WritingMode::HorizontalTb, crate::Direction::Ltr),
            definite,
        )
    })
    .expect("active inherited-axis layout");
    let subgrid = fri08_c01_placement_output(&batch, 2);
    let second_track_child = fri08_c01_placement_output(&batch, 3);
    let overflowing_child = fri08_c01_placement_output(&batch, 4);
    let overflow = subgrid
        .scroll_geometry
        .expect("active inherited-axis scroll overflow")
        .physical_range()
        .x();

    assert_eq!((subgrid.location.x, subgrid.size.width), (0.0, 500.0));
    assert_eq!(second_track_child.location.x, 275.0);
    assert_eq!(overflowing_child.location.x, 475.0);
    assert_eq!((overflow.minimum(), overflow.maximum()), (0.0, 75.0));
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
struct Fri06C07SubgridRtlRow {
    source: &'static str,
    variant: &'static str,
    self_alignment: AlignItems,
    item_alignment: AlignItems,
    box_sizing: BoxSizing,
}

const FRI06_C07_SUBGRID_RTL_ROWS: [Fri06C07SubgridRtlRow; 32] = [
    Fri06C07SubgridRtlRow {
        source: "html/subgrid/subgrid_alignment_baseline_baseline_item.html",
        variant: "border_box_rtl",
        self_alignment: AlignItems::Baseline,
        item_alignment: AlignItems::Baseline,
        box_sizing: BoxSizing::BorderBox,
    },
    Fri06C07SubgridRtlRow {
        source: "html/subgrid/subgrid_alignment_baseline_baseline_item.html",
        variant: "content_box_rtl",
        self_alignment: AlignItems::Baseline,
        item_alignment: AlignItems::Baseline,
        box_sizing: BoxSizing::ContentBox,
    },
    Fri06C07SubgridRtlRow {
        source: "html/subgrid/subgrid_alignment_baseline_center_item.html",
        variant: "border_box_rtl",
        self_alignment: AlignItems::Baseline,
        item_alignment: AlignItems::Center,
        box_sizing: BoxSizing::BorderBox,
    },
    Fri06C07SubgridRtlRow {
        source: "html/subgrid/subgrid_alignment_baseline_center_item.html",
        variant: "content_box_rtl",
        self_alignment: AlignItems::Baseline,
        item_alignment: AlignItems::Center,
        box_sizing: BoxSizing::ContentBox,
    },
    Fri06C07SubgridRtlRow {
        source: "html/subgrid/subgrid_alignment_baseline_end_item.html",
        variant: "border_box_rtl",
        self_alignment: AlignItems::Baseline,
        item_alignment: AlignItems::End,
        box_sizing: BoxSizing::BorderBox,
    },
    Fri06C07SubgridRtlRow {
        source: "html/subgrid/subgrid_alignment_baseline_end_item.html",
        variant: "content_box_rtl",
        self_alignment: AlignItems::Baseline,
        item_alignment: AlignItems::End,
        box_sizing: BoxSizing::ContentBox,
    },
    Fri06C07SubgridRtlRow {
        source: "html/subgrid/subgrid_alignment_baseline_start_item.html",
        variant: "border_box_rtl",
        self_alignment: AlignItems::Baseline,
        item_alignment: AlignItems::Start,
        box_sizing: BoxSizing::BorderBox,
    },
    Fri06C07SubgridRtlRow {
        source: "html/subgrid/subgrid_alignment_baseline_start_item.html",
        variant: "content_box_rtl",
        self_alignment: AlignItems::Baseline,
        item_alignment: AlignItems::Start,
        box_sizing: BoxSizing::ContentBox,
    },
    Fri06C07SubgridRtlRow {
        source: "html/subgrid/subgrid_alignment_center_baseline_item.html",
        variant: "border_box_rtl",
        self_alignment: AlignItems::Center,
        item_alignment: AlignItems::Baseline,
        box_sizing: BoxSizing::BorderBox,
    },
    Fri06C07SubgridRtlRow {
        source: "html/subgrid/subgrid_alignment_center_baseline_item.html",
        variant: "content_box_rtl",
        self_alignment: AlignItems::Center,
        item_alignment: AlignItems::Baseline,
        box_sizing: BoxSizing::ContentBox,
    },
    Fri06C07SubgridRtlRow {
        source: "html/subgrid/subgrid_alignment_center_center_item.html",
        variant: "border_box_rtl",
        self_alignment: AlignItems::Center,
        item_alignment: AlignItems::Center,
        box_sizing: BoxSizing::BorderBox,
    },
    Fri06C07SubgridRtlRow {
        source: "html/subgrid/subgrid_alignment_center_center_item.html",
        variant: "content_box_rtl",
        self_alignment: AlignItems::Center,
        item_alignment: AlignItems::Center,
        box_sizing: BoxSizing::ContentBox,
    },
    Fri06C07SubgridRtlRow {
        source: "html/subgrid/subgrid_alignment_center_end_item.html",
        variant: "border_box_rtl",
        self_alignment: AlignItems::Center,
        item_alignment: AlignItems::End,
        box_sizing: BoxSizing::BorderBox,
    },
    Fri06C07SubgridRtlRow {
        source: "html/subgrid/subgrid_alignment_center_end_item.html",
        variant: "content_box_rtl",
        self_alignment: AlignItems::Center,
        item_alignment: AlignItems::End,
        box_sizing: BoxSizing::ContentBox,
    },
    Fri06C07SubgridRtlRow {
        source: "html/subgrid/subgrid_alignment_center_start_item.html",
        variant: "border_box_rtl",
        self_alignment: AlignItems::Center,
        item_alignment: AlignItems::Start,
        box_sizing: BoxSizing::BorderBox,
    },
    Fri06C07SubgridRtlRow {
        source: "html/subgrid/subgrid_alignment_center_start_item.html",
        variant: "content_box_rtl",
        self_alignment: AlignItems::Center,
        item_alignment: AlignItems::Start,
        box_sizing: BoxSizing::ContentBox,
    },
    Fri06C07SubgridRtlRow {
        source: "html/subgrid/subgrid_alignment_end_baseline_item.html",
        variant: "border_box_rtl",
        self_alignment: AlignItems::End,
        item_alignment: AlignItems::Baseline,
        box_sizing: BoxSizing::BorderBox,
    },
    Fri06C07SubgridRtlRow {
        source: "html/subgrid/subgrid_alignment_end_baseline_item.html",
        variant: "content_box_rtl",
        self_alignment: AlignItems::End,
        item_alignment: AlignItems::Baseline,
        box_sizing: BoxSizing::ContentBox,
    },
    Fri06C07SubgridRtlRow {
        source: "html/subgrid/subgrid_alignment_end_center_item.html",
        variant: "border_box_rtl",
        self_alignment: AlignItems::End,
        item_alignment: AlignItems::Center,
        box_sizing: BoxSizing::BorderBox,
    },
    Fri06C07SubgridRtlRow {
        source: "html/subgrid/subgrid_alignment_end_center_item.html",
        variant: "content_box_rtl",
        self_alignment: AlignItems::End,
        item_alignment: AlignItems::Center,
        box_sizing: BoxSizing::ContentBox,
    },
    Fri06C07SubgridRtlRow {
        source: "html/subgrid/subgrid_alignment_end_end_item.html",
        variant: "border_box_rtl",
        self_alignment: AlignItems::End,
        item_alignment: AlignItems::End,
        box_sizing: BoxSizing::BorderBox,
    },
    Fri06C07SubgridRtlRow {
        source: "html/subgrid/subgrid_alignment_end_end_item.html",
        variant: "content_box_rtl",
        self_alignment: AlignItems::End,
        item_alignment: AlignItems::End,
        box_sizing: BoxSizing::ContentBox,
    },
    Fri06C07SubgridRtlRow {
        source: "html/subgrid/subgrid_alignment_end_start_item.html",
        variant: "border_box_rtl",
        self_alignment: AlignItems::End,
        item_alignment: AlignItems::Start,
        box_sizing: BoxSizing::BorderBox,
    },
    Fri06C07SubgridRtlRow {
        source: "html/subgrid/subgrid_alignment_end_start_item.html",
        variant: "content_box_rtl",
        self_alignment: AlignItems::End,
        item_alignment: AlignItems::Start,
        box_sizing: BoxSizing::ContentBox,
    },
    Fri06C07SubgridRtlRow {
        source: "html/subgrid/subgrid_alignment_start_baseline_item.html",
        variant: "border_box_rtl",
        self_alignment: AlignItems::Start,
        item_alignment: AlignItems::Baseline,
        box_sizing: BoxSizing::BorderBox,
    },
    Fri06C07SubgridRtlRow {
        source: "html/subgrid/subgrid_alignment_start_baseline_item.html",
        variant: "content_box_rtl",
        self_alignment: AlignItems::Start,
        item_alignment: AlignItems::Baseline,
        box_sizing: BoxSizing::ContentBox,
    },
    Fri06C07SubgridRtlRow {
        source: "html/subgrid/subgrid_alignment_start_center_item.html",
        variant: "border_box_rtl",
        self_alignment: AlignItems::Start,
        item_alignment: AlignItems::Center,
        box_sizing: BoxSizing::BorderBox,
    },
    Fri06C07SubgridRtlRow {
        source: "html/subgrid/subgrid_alignment_start_center_item.html",
        variant: "content_box_rtl",
        self_alignment: AlignItems::Start,
        item_alignment: AlignItems::Center,
        box_sizing: BoxSizing::ContentBox,
    },
    Fri06C07SubgridRtlRow {
        source: "html/subgrid/subgrid_alignment_start_end_item.html",
        variant: "border_box_rtl",
        self_alignment: AlignItems::Start,
        item_alignment: AlignItems::End,
        box_sizing: BoxSizing::BorderBox,
    },
    Fri06C07SubgridRtlRow {
        source: "html/subgrid/subgrid_alignment_start_end_item.html",
        variant: "content_box_rtl",
        self_alignment: AlignItems::Start,
        item_alignment: AlignItems::End,
        box_sizing: BoxSizing::ContentBox,
    },
    Fri06C07SubgridRtlRow {
        source: "html/subgrid/subgrid_alignment_start_start_item.html",
        variant: "border_box_rtl",
        self_alignment: AlignItems::Start,
        item_alignment: AlignItems::Start,
        box_sizing: BoxSizing::BorderBox,
    },
    Fri06C07SubgridRtlRow {
        source: "html/subgrid/subgrid_alignment_start_start_item.html",
        variant: "content_box_rtl",
        self_alignment: AlignItems::Start,
        item_alignment: AlignItems::Start,
        box_sizing: BoxSizing::ContentBox,
    },
];

#[derive(Clone, Debug)]
struct Fri06C07SubgridRtlCase<S: LayoutScalar> {
    tree: PublicLayoutTreeOf<S>,
    target_inline_grid: u32,
    target_subgrid: u32,
    target_subject: u32,
}

fn fri06_c07_subgrid_rtl_alignment_index(alignment: AlignItems) -> usize {
    match alignment {
        AlignItems::Start => 0,
        AlignItems::End => 1,
        AlignItems::Center => 2,
        AlignItems::Baseline => 3,
        _ => unreachable!("the finite C07-T3 table contains only the four reviewed alignments"),
    }
}

fn fri06_c07_subgrid_rtl_expected_origin<S: LayoutScalar>(alignment: AlignItems) -> S {
    // Four actual level1 inline grids reverse under L2: their physical
    // origins are300/200/100/0, plus subgrid10 and subject20 local offsets.
    let value = match alignment {
        AlignItems::Start => 330.0,
        AlignItems::End => 230.0,
        AlignItems::Center => 130.0,
        AlignItems::Baseline => 30.0,
        _ => unreachable!("the finite C07-T3 table contains only the four reviewed alignments"),
    };
    S::from_f64(value)
}

fn fri06_c07_subgrid_rtl_tree<S: LayoutScalar>(
    row: Fri06C07SubgridRtlRow,
) -> Fri06C07SubgridRtlCase<S> {
    let scalar = S::from_f64;
    let root = NodeInputOf {
        display: Display::Block,
        box_sizing: row.box_sizing,
        direction: Direction::Rtl,
        size: Size::new(
            PreferredSizeOf::px(scalar(400.0)),
            PreferredSizeOf::px(scalar(400.0)),
        ),
        ..NodeInputOf::default()
    };
    let alignments = [
        AlignItems::Start,
        AlignItems::End,
        AlignItems::Center,
        AlignItems::Baseline,
    ];
    let mut children = HashMap::new();
    let mut inputs = HashMap::from([(0, root)]);
    let inline_grid_nodes = [1, 2, 3, 4];
    children.insert(0, inline_grid_nodes.to_vec());
    for (self_index, self_alignment) in alignments.into_iter().enumerate() {
        let inline_grid_node = inline_grid_nodes[self_index];
        let first_subgrid_node = 10 + u32::try_from(self_index * 4).expect("finite matrix index");
        let subgrid_nodes = (first_subgrid_node..first_subgrid_node + 4).collect::<Vec<_>>();
        let inline_grid = NodeInputOf {
            display: Display::InlineGrid,
            box_sizing: row.box_sizing,
            direction: Direction::Rtl,
            grid_template_columns: vec![TrackComponentOf::px(scalar(100.0))],
            grid_auto_rows: vec![TrackComponentOf::px(scalar(100.0))],
            atomic_inline_participation: Some(AtomicInlineParticipation::new(
                BidiLevel::try_new(1).expect("RTL bidi level is valid"),
                InlineBreakOpportunity::Prohibited,
            )),
            ..NodeInputOf::default()
        };
        inputs.insert(inline_grid_node, inline_grid);
        children.insert(inline_grid_node, subgrid_nodes.clone());

        for (item_index, item_alignment) in alignments.into_iter().enumerate() {
            let subgrid_node = subgrid_nodes[item_index];
            let subject_node =
                100 + u32::try_from(self_index * 4 + item_index).expect("finite matrix index");
            let subgrid = NodeInputOf {
                display: Display::Grid,
                box_sizing: row.box_sizing,
                direction: Direction::Rtl,
                grid_template_columns: vec![TrackComponentOf::Subgrid(SubgridTrack::new(vec![]))],
                grid_template_rows: vec![TrackComponentOf::Subgrid(SubgridTrack::new(vec![]))],
                align_self: Some(self_alignment),
                justify_self: Some(item_alignment),
                margin: Edges::all(LengthAutoOf::px(scalar(10.0))),
                border: Edges::all(LengthOf::px(scalar(10.0))),
                padding: Edges::all(LengthOf::px(scalar(10.0))),
                ..NodeInputOf::default()
            };
            let subject = NodeInputOf {
                display: Display::Block,
                box_sizing: row.box_sizing,
                direction: Direction::Rtl,
                size: Size::new(
                    PreferredSizeOf::px(scalar(40.0)),
                    PreferredSizeOf::px(scalar(40.0)),
                ),
                justify_self: Some(AlignItems::Start),
                align_self: Some(AlignItems::Start),
                ..NodeInputOf::default()
            };
            inputs.insert(subgrid_node, subgrid);
            inputs.insert(subject_node, subject);
            children.insert(subgrid_node, vec![subject_node]);
            children.insert(subject_node, vec![]);
        }
    }

    let target_self_index = fri06_c07_subgrid_rtl_alignment_index(row.self_alignment);
    let target_item_index = fri06_c07_subgrid_rtl_alignment_index(row.item_alignment);
    let target_subgrid =
        10 + u32::try_from(target_self_index * 4 + target_item_index).expect("finite matrix index");
    let target_subject = 100
        + u32::try_from(target_self_index * 4 + target_item_index).expect("finite matrix index");

    let mut tree = PublicLayoutTreeOf::new();
    for (node, node_children) in children {
        tree.insert_children(node, node_children);
    }
    for (node, input) in inputs {
        tree.insert_input(node, LayoutInputOf::box_input(input));
    }

    Fri06C07SubgridRtlCase {
        tree,
        target_inline_grid: inline_grid_nodes[target_self_index],
        target_subgrid,
        target_subject,
    }
}

fn fri06_c07_subgrid_rtl_mismatches<S: LayoutScalar>() -> Vec<String> {
    assert_eq!(FRI06_C07_SUBGRID_RTL_ROWS.len(), 32);
    let unique_rows = FRI06_C07_SUBGRID_RTL_ROWS
        .iter()
        .map(|row| (row.source, row.variant))
        .collect::<HashSet<_>>();
    assert_eq!(unique_rows.len(), 32);

    let mut physical_inline_mismatches = Vec::new();
    for row in FRI06_C07_SUBGRID_RTL_ROWS {
        let case = fri06_c07_subgrid_rtl_tree::<S>(row);
        let batch = compute_layout(&case.tree, 0, {
            let axes = Size::splat(AxisSizingRequestOf::new(
                AvailableSpaceOf::Definite(
                    crate::NonNegativeFiniteOf::new(S::from_f64(400.0)).unwrap(),
                ),
                SizingOperation::Normal,
            ));
            let definite = axes.map(|axis| axis.space().definite_size());
            crate::LayoutRootRequestOf::standalone(
                SizingRequestOf::new(
                    axes,
                    OrthogonalFallbackOf::new(
                        definite,
                        crate::NearestScrollportOf::Absent,
                        Size::splat(crate::OrthogonalFallbackLimitOf::Unbounded),
                    ),
                ),
                crate::geometry::FlowAxes::new(
                    crate::WritingMode::HorizontalTb,
                    crate::Direction::Ltr,
                ),
                definite,
            )
        })
        .expect("RTL subgrid alignment computes through the public front door");
        let inline_grid =
            readback::single_box_output(&batch, case.target_inline_grid, Phase::Unrounded);
        let subgrid = readback::single_box_output(&batch, case.target_subgrid, Phase::Unrounded);
        let subject = readback::single_box_output(&batch, case.target_subject, Phase::Unrounded);

        assert_eq!(subject.size, Size::splat(S::from_f64(40.0)), "{row:?}");
        let root_relative_origin = Point::new(
            inline_grid.location.x + subgrid.location.x + subject.location.x,
            inline_grid.location.y + subgrid.location.y + subject.location.y,
        );
        assert_eq!(
            root_relative_origin.y,
            S::from_f64(match row.item_alignment {
                AlignItems::Start => 30.0,
                AlignItems::End => 130.0,
                AlignItems::Center => 230.0,
                AlignItems::Baseline => 330.0,
                _ => unreachable!("finite table alignment"),
            }),
            "{row:?} must preserve the logical block origin"
        );
        let expected_x = fri06_c07_subgrid_rtl_expected_origin(row.self_alignment);
        if root_relative_origin.x != expected_x {
            physical_inline_mismatches.push(format!(
                "{row:?}: inline-grid x={:?}, subgrid x={:?}, subject x={:?}, root-relative x={:?}, expected x={expected_x:?}",
                inline_grid.location.x,
                subgrid.location.x,
                subject.location.x,
                root_relative_origin.x,
            ));
        }
    }
    physical_inline_mismatches
}

#[test]
fn fri06_c07_subgrid_rtl_exact_thirty_two_rows_mirror_only_physical_inline_origin() {
    let mut mismatches = fri06_c07_subgrid_rtl_mismatches::<f32>();
    mismatches.extend(fri06_c07_subgrid_rtl_mismatches::<f64>());
    assert!(
        mismatches.is_empty(),
        "every row in both scalar lanes must mirror the logical inline origin exactly once: {mismatches:#?}"
    );
}

#[test]
fn fri05_c05_grid_geometry_measurement_remains_absent_while_grid_lanes_publish() {
    let style = NodeInput {
        display: Display::Grid,
        size: Size::new(PreferredSize::px(40.0), PreferredSize::px(30.0)),
        overflow: computed_overflow(Overflow::Scroll, Overflow::Scroll),
        grid_template_columns: vec![TrackComponent::px(40.0)],
        grid_template_rows: vec![TrackComponent::px(30.0)],
        ..NodeInput::default()
    };
    let mut measurement = OracleTree::new().children(0, []).style(0, style.clone());
    let measurement_output = compute_grid(
        &mut measurement.borrow_compute(),
        0,
        ComputeInput::for_child(
            RunMode::ComputeSize,
            SizingMode::InherentSize,
            RequestedAxis::Both,
            Size::new(Some(40.0), Some(30.0)),
            Size::new(Some(40.0), Some(30.0)),
            ContainingLayoutContext::new(
                FlowAxes::new(WritingMode::HorizontalTb, Direction::Ltr),
                ParentFormattingContext::NoParent,
            ),
            SizingRequestOf::new(
                Size::new(
                    AxisSizingRequestOf::new(
                        AvailableSpaceOf::Definite(crate::NonNegativeFiniteOf::new(40.0).unwrap()),
                        SizingOperation::Normal,
                    ),
                    AxisSizingRequestOf::new(
                        AvailableSpaceOf::Definite(crate::NonNegativeFiniteOf::new(30.0).unwrap()),
                        SizingOperation::Normal,
                    ),
                ),
                OrthogonalFallbackOf::new(
                    Size::NONE,
                    crate::NearestScrollportOf::Absent,
                    Size::splat(crate::OrthogonalFallbackLimitOf::Unbounded),
                ),
            ),
        ),
    )
    .unwrap();
    assert!(measurement_output.scroll_geometry.is_none());

    let mut lanes = OracleTree::new().children(0, []).style(
        0,
        NodeInput {
            display: Display::GridLanes,
            ..style
        },
    );
    let lanes_output = compute_grid(
        &mut lanes.borrow_compute(),
        0,
        fri05_c05_grid_sizing_input(Size::new(Some(40.0), Some(30.0))),
    )
    .unwrap();
    assert!(lanes_output.scroll_geometry.is_some());
}

#[test]
fn fri05_c05_grid_lanes_geometry_publishes_reservation_range_and_target_in_all_flows() {
    for writing_mode in [
        WritingMode::HorizontalTb,
        WritingMode::VerticalRl,
        WritingMode::VerticalLr,
        WritingMode::SidewaysRl,
        WritingMode::SidewaysLr,
    ] {
        for direction in [Direction::Ltr, Direction::Rtl] {
            for (overflow, replaced, expected_used) in [
                (Overflow::Visible, false, Overflow::Visible),
                (Overflow::Clip, false, Overflow::Clip),
                (Overflow::Hidden, false, Overflow::Hidden),
                (Overflow::Scroll, false, Overflow::Scroll),
                (Overflow::Auto, false, Overflow::Auto),
                (Overflow::Hidden, true, Overflow::Clip),
            ] {
                // Actual item extent supplies stacking overflow; template rows
                // are nonoperational on this Columns-oriented container.
                let child_size = FlowAxes::new(writing_mode, direction)
                    .physical_size(LogicalSizeOf::new(160.0, 140.0));
                let mut tree = OracleTree::new()
                    .children(0, [1])
                    .children(1, [])
                    .style(
                        1,
                        NodeInput {
                            size: child_size.map(PreferredSize::px),
                            ..NodeInput::default()
                        },
                    )
                    .style(
                        0,
                        NodeInput {
                            display: Display::GridLanes,
                            writing_mode,
                            direction,
                            size: Size::new(PreferredSize::px(100.0), PreferredSize::px(80.0)),
                            overflow: computed_overflow(overflow, overflow),
                            item_is_replaced: replaced,
                            scrollbar_gutter: ScrollbarGutter::StableBothEdges,
                            scrollbar_width: ScrollbarWidth::try_new(10.0).unwrap(),
                            justify_content: Some(AlignContent::End),
                            align_content: Some(AlignContent::Center),
                            grid_template_columns: vec![TrackComponent::px(160.0)],
                            grid_template_rows: vec![TrackComponent::px(140.0)],
                            scroll_margin: ScrollMargin::try_new(1.0, 2.0, 3.0, 4.0).unwrap(),
                            scroll_snap_align: ScrollSnapAlign::new(
                                ScrollSnapAlignValue::End,
                                ScrollSnapAlignValue::Center,
                            ),
                            scroll_snap_stop: ScrollSnapStop::Always,
                            ..NodeInput::default()
                        },
                    );

                let output = compute_grid(
                    &mut tree.borrow_compute(),
                    0,
                    fri05_c05_grid_sizing_input(Size::new(Some(100.0), Some(80.0))),
                )
                .expect("grid-lanes geometry computes");
                let geometry = output
                    .scroll_geometry
                    .expect("performed grid-lanes publishes canonical geometry");
                assert_eq!(geometry.used_overflow_x(), expected_used);
                assert_eq!(geometry.used_overflow_y(), expected_used);
                assert_eq!(geometry.target().border_box(), geometry.border_box());
                assert_eq!(
                    geometry.target().scroll_margin(),
                    ScrollMargin::try_new(1.0, 2.0, 3.0, 4.0).unwrap()
                );
                assert_eq!(geometry.target().snap_stop(), ScrollSnapStop::Always);
                assert!(
                    output.content_size.width >= geometry.canonical_content_size().unwrap().width
                        && output.content_size.height
                            >= geometry.canonical_content_size().unwrap().height
                );

                if matches!(overflow, Overflow::Scroll) && !replaced {
                    let flow_range = geometry
                        .flow_axes()
                        .flow_relative_scroll_range(geometry.physical_range());
                    assert!(flow_range.inline().minimum() < 0.0);
                    assert_eq!(flow_range.inline().maximum(), 0.0);
                    assert!(flow_range.block().minimum() < 0.0);
                    assert!(flow_range.block().maximum() > 0.0);
                }
            }
        }
    }
}

#[test]
fn fri05_c05_grid_lanes_geometry_reserves_forced_stable_both_zero_tiny_and_auto() {
    for (overflow, gutter, width, size, expected_edges) in [
        (
            computed_overflow(Overflow::Scroll, Overflow::Scroll),
            ScrollbarGutter::Auto,
            10.0,
            Size::new(100.0, 80.0),
            (false, true, true),
        ),
        (
            computed_overflow(Overflow::Hidden, Overflow::Hidden),
            ScrollbarGutter::Stable,
            10.0,
            Size::new(100.0, 80.0),
            (false, true, false),
        ),
        (
            computed_overflow(Overflow::Hidden, Overflow::Hidden),
            ScrollbarGutter::StableBothEdges,
            10.0,
            Size::new(100.0, 80.0),
            (true, true, false),
        ),
        (
            computed_overflow(Overflow::Scroll, Overflow::Scroll),
            ScrollbarGutter::Auto,
            0.0,
            Size::new(100.0, 80.0),
            (false, false, false),
        ),
        (
            computed_overflow(Overflow::Hidden, Overflow::Hidden),
            ScrollbarGutter::StableBothEdges,
            8.0,
            Size::new(10.0, 6.0),
            (true, true, false),
        ),
    ] {
        let mut tree = OracleTree::new().children(0, []).style(
            0,
            NodeInput {
                display: Display::GridLanes,
                size: size.map(PreferredSize::px),
                overflow,
                scrollbar_gutter: gutter,
                scrollbar_width: ScrollbarWidth::try_new(width).unwrap(),
                grid_template_columns: vec![TrackComponent::px(1.0)],
                grid_template_rows: vec![TrackComponent::px(1.0)],
                ..NodeInput::default()
            },
        );
        let geometry = compute_grid(
            &mut tree.borrow_compute(),
            0,
            fri05_c05_grid_sizing_input(size.map(Some)),
        )
        .unwrap()
        .scroll_geometry
        .unwrap();
        assert_eq!(geometry.gutters().left().is_some(), expected_edges.0);
        assert_eq!(geometry.gutters().right().is_some(), expected_edges.1);
        assert_eq!(geometry.gutters().bottom().is_some(), expected_edges.2);
        if size == Size::new(10.0, 6.0) {
            assert_eq!(geometry.content_box().size(), Size::new(0.0, 6.0));
        }
    }

    let mut auto = OracleTree::new()
        .children(0, [1])
        .children(1, [])
        .style(
            0,
            NodeInput {
                display: Display::GridLanes,
                size: Size::new(PreferredSize::px(100.0), PreferredSize::px(100.0)),
                overflow: computed_overflow(Overflow::Auto, Overflow::Auto),
                scrollbar_width: ScrollbarWidth::try_new(10.0).unwrap(),
                grid_template_columns: vec![TrackComponent::px(95.0)],
                grid_template_rows: vec![TrackComponent::px(120.0)],
                ..NodeInput::default()
            },
        )
        .style(
            1,
            NodeInput {
                position: Position::Relative,
                size: Size::new(PreferredSize::px(20.0), PreferredSize::px(20.0)),
                inset: Edges::new(
                    LengthAuto::px(95.0),
                    LengthAuto::AUTO,
                    LengthAuto::AUTO,
                    LengthAuto::px(95.0),
                ),
                ..NodeInput::default()
            },
        );
    let geometry = compute_grid(
        &mut auto.borrow_compute(),
        0,
        fri05_c05_grid_sizing_input(Size::splat(Some(100.0))),
    )
    .unwrap()
    .scroll_geometry
    .unwrap();
    assert!(geometry.gutters().right().is_some());
    assert!(geometry.gutters().bottom().is_some());
    assert!(auto.inputs(1).iter().all(|input| {
        input.settled_auto_scrollbars() == crate::scroll::SettledAutoScrollbarState::INITIAL
    }));
}

#[test]
fn fri04_c04_grid_dispatch_grid_and_lanes_cover_all_direct_and_keyword_basis_payloads() {
    let sizing =
        || SizingCalculation::value(LengthPercentageOf::px(10.0).expect("finite calculation"));
    let calc = || CalcSizeCalculation::value(LengthPercentageOf::ZERO);

    for (display, algorithm) in [
        (Display::Grid, SizingAlgorithm::Grid),
        (Display::InlineGrid, SizingAlgorithm::Grid),
        (Display::GridLanes, SizingAlgorithm::GridLanes),
        (Display::InlineGridLanes, SizingAlgorithm::GridLanes),
    ] {
        for (value, behavior) in [
            (PreferredSize::STRETCH, SizingBehavior::Stretch),
            (PreferredSize::FIT_CONTENT, SizingBehavior::FitContent),
            (PreferredSize::CONTAIN, SizingBehavior::Contain),
            (
                PreferredSize::fit_content_function(sizing()),
                SizingBehavior::FitContentFunction,
            ),
        ] {
            fri04_c04_grid_dispatch_assert_error(
                display,
                fri04_c04_grid_dispatch_style(
                    Fri04C04GridSizingValue::Preferred(value),
                    PhysicalAxis::Horizontal,
                ),
                SizingProperty::Preferred,
                behavior,
                algorithm,
                PhysicalAxis::Horizontal,
                0,
            );
        }
        for (value, behavior) in [
            (MinSize::MIN_CONTENT, SizingBehavior::MinContent),
            (MinSize::MAX_CONTENT, SizingBehavior::MaxContent),
            (MinSize::STRETCH, SizingBehavior::Stretch),
            (MinSize::FIT_CONTENT, SizingBehavior::FitContent),
            (MinSize::CONTAIN, SizingBehavior::Contain),
            (
                MinSize::fit_content_function(sizing()),
                SizingBehavior::FitContentFunction,
            ),
        ] {
            fri04_c04_grid_dispatch_assert_error(
                display,
                fri04_c04_grid_dispatch_style(
                    Fri04C04GridSizingValue::Minimum(value),
                    PhysicalAxis::Vertical,
                ),
                SizingProperty::Minimum,
                behavior,
                algorithm,
                PhysicalAxis::Vertical,
                0,
            );
        }
        for (value, behavior) in [
            (MaxSize::MIN_CONTENT, SizingBehavior::MinContent),
            (MaxSize::MAX_CONTENT, SizingBehavior::MaxContent),
            (MaxSize::STRETCH, SizingBehavior::Stretch),
            (MaxSize::FIT_CONTENT, SizingBehavior::FitContent),
            (MaxSize::CONTAIN, SizingBehavior::Contain),
            (
                MaxSize::fit_content_function(sizing()),
                SizingBehavior::FitContentFunction,
            ),
        ] {
            fri04_c04_grid_dispatch_assert_error(
                display,
                fri04_c04_grid_dispatch_style(
                    Fri04C04GridSizingValue::Maximum(value),
                    PhysicalAxis::Horizontal,
                ),
                SizingProperty::Maximum,
                behavior,
                algorithm,
                PhysicalAxis::Horizontal,
                0,
            );
        }

        for (basis, behavior) in [
            (PreferredSizeCalcBasis::Auto, CalcSizeBehaviorBasis::Auto),
            (
                PreferredSizeCalcBasis::MinContent,
                CalcSizeBehaviorBasis::MinContent,
            ),
            (
                PreferredSizeCalcBasis::MaxContent,
                CalcSizeBehaviorBasis::MaxContent,
            ),
            (
                PreferredSizeCalcBasis::Stretch,
                CalcSizeBehaviorBasis::Stretch,
            ),
            (
                PreferredSizeCalcBasis::FitContent,
                CalcSizeBehaviorBasis::FitContent,
            ),
            (
                PreferredSizeCalcBasis::Contain,
                CalcSizeBehaviorBasis::Contain,
            ),
        ] {
            fri04_c04_grid_dispatch_assert_error(
                display,
                fri04_c04_grid_dispatch_style(
                    Fri04C04GridSizingValue::Preferred(
                        PreferredSize::calc_size(basis, calc()).expect("valid calc-size"),
                    ),
                    PhysicalAxis::Vertical,
                ),
                SizingProperty::Preferred,
                SizingBehavior::CalcSize(behavior),
                algorithm,
                PhysicalAxis::Vertical,
                0,
            );
        }
        for (basis, behavior) in [
            (MinSizeCalcBasis::Auto, CalcSizeBehaviorBasis::Auto),
            (
                MinSizeCalcBasis::MinContent,
                CalcSizeBehaviorBasis::MinContent,
            ),
            (
                MinSizeCalcBasis::MaxContent,
                CalcSizeBehaviorBasis::MaxContent,
            ),
            (MinSizeCalcBasis::Stretch, CalcSizeBehaviorBasis::Stretch),
            (
                MinSizeCalcBasis::FitContent,
                CalcSizeBehaviorBasis::FitContent,
            ),
            (MinSizeCalcBasis::Contain, CalcSizeBehaviorBasis::Contain),
        ] {
            fri04_c04_grid_dispatch_assert_error(
                display,
                fri04_c04_grid_dispatch_style(
                    Fri04C04GridSizingValue::Minimum(
                        MinSize::calc_size(basis, calc()).expect("valid calc-size"),
                    ),
                    PhysicalAxis::Horizontal,
                ),
                SizingProperty::Minimum,
                SizingBehavior::CalcSize(behavior),
                algorithm,
                PhysicalAxis::Horizontal,
                0,
            );
        }
        for (basis, behavior) in [
            (MaxSizeCalcBasis::None, CalcSizeBehaviorBasis::None),
            (
                MaxSizeCalcBasis::MinContent,
                CalcSizeBehaviorBasis::MinContent,
            ),
            (
                MaxSizeCalcBasis::MaxContent,
                CalcSizeBehaviorBasis::MaxContent,
            ),
            (MaxSizeCalcBasis::Stretch, CalcSizeBehaviorBasis::Stretch),
            (
                MaxSizeCalcBasis::FitContent,
                CalcSizeBehaviorBasis::FitContent,
            ),
            (MaxSizeCalcBasis::Contain, CalcSizeBehaviorBasis::Contain),
        ] {
            fri04_c04_grid_dispatch_assert_error(
                display,
                fri04_c04_grid_dispatch_style(
                    Fri04C04GridSizingValue::Maximum(
                        MaxSize::calc_size(basis, calc()).expect("valid calc-size"),
                    ),
                    PhysicalAxis::Vertical,
                ),
                SizingProperty::Maximum,
                SizingBehavior::CalcSize(behavior),
                algorithm,
                PhysicalAxis::Vertical,
                0,
            );
        }
    }
}

#[test]
fn grid_lanes_order_modified_sequence_drives_running_offsets_and_intrinsic_contributions_in_both_scalar_lanes()
 {
    assert_grid_lanes_order_modified_sequence::<f32>();
    assert_grid_lanes_order_modified_sequence::<f64>();
}

fn assert_grid_lanes_order_modified_sequence<S: LayoutScalar>()
where
    OracleTreeOf<S>: Traverse<Node = u32, Scalar = S>,
{
    let item_style = |order, width, height| NodeInputOf::<S> {
        item_order: ItemOrder::new(order),
        size: Size::new(
            PreferredSizeOf::px(S::from_f64(width)),
            PreferredSizeOf::px(S::from_f64(height)),
        ),
        grid_column: GridPlacement::try_line(1).expect("one is a valid grid line"),
        ..NodeInputOf::default()
    };
    let mut tree = OracleTreeOf::<S>::new()
        .children(0, [1, 2, 3, 4, 5, 6])
        .children(1, [])
        .children(2, [])
        .children(3, [])
        .children(4, [])
        .children(5, [])
        .children(6, [])
        .style(
            0,
            NodeInputOf {
                display: Display::GridLanes,
                grid_template_columns: vec![TrackComponentOf::AUTO, TrackComponentOf::AUTO],
                grid_template_rows: vec![TrackComponentOf::px(S::ZERO)],
                grid_flow_tolerance: GridFlowToleranceOf::Length(LengthOf::ZERO),
                ..NodeInputOf::default()
            },
        )
        .style(
            1,
            NodeInputOf {
                grid_column: GridPlacement::try_line_span(1, 2)
                    .expect("line one with span two is valid"),
                ..item_style(2, 100.0, 10.0)
            },
        )
        .style(
            2,
            NodeInputOf {
                display: Display::None,
                item_order: ItemOrder::new(i32::MIN),
                ..item_style(0, 500.0, 500.0)
            },
        )
        .style(3, item_style(-1, 80.0, 20.0))
        .style(
            4,
            NodeInputOf {
                position: Position::Absolute,
                item_order: ItemOrder::new(i32::MIN),
                ..item_style(0, 500.0, 500.0)
            },
        )
        .style(5, item_style(0, 0.0, 30.0))
        .style(6, item_style(2, 0.0, 40.0));

    let placement = |column, in_flow| ResolvedGridItemPlacement {
        column,
        row: GridPlacement::try_line(1).expect("one is a valid grid line"),
        absolute_column: column,
        absolute_row: GridPlacement::try_line(1).expect("one is a valid grid line"),
        in_flow,
    };
    let mut compute = tree.borrow_compute();
    let item_inputs = (1..=6)
        .map(|child| grid_child_input!(compute.node_input(child)))
        .collect();
    let placements = GridPlacementContext::new_with_child_inputs(
        vec![1, 2, 3, 4, 5, 6],
        vec![
            placement(
                GridPlacement::try_line_span(1, 2).expect("line-span placement is valid"),
                true,
            ),
            placement(GridPlacement::AUTO, false),
            placement(
                GridPlacement::try_line(1).expect("one is a valid grid line"),
                true,
            ),
            placement(GridPlacement::AUTO, false),
            placement(
                GridPlacement::try_line(1).expect("one is a valid grid line"),
                true,
            ),
            placement(
                GridPlacement::try_line(1).expect("one is a valid grid line"),
                true,
            ),
        ],
        item_inputs,
    )
    .with_order_modified_indexes(vec![
        SourceIndex::new(2),
        SourceIndex::new(4),
        SourceIndex::new(0),
        SourceIndex::new(5),
    ]);
    let constants = Constants {
        child_orthogonal_fallback: OrthogonalFallbackOf::new(
            Size::NONE,
            crate::NearestScrollportOf::Absent,
            Size::splat(crate::OrthogonalFallbackLimitOf::Unknown),
        ),
        flow_axes: FlowAxes::new(WritingMode::HorizontalTb, Direction::Ltr),
        explicit_definite_content_size: Size::NONE,
        indefinite_percentage_axis: None,
        node_outer_size: Size::NONE,
        node_inner_size: Size::NONE,
        node_min_size: Size::NONE,
        node_definite_min_size: Size::NONE,
        fallback_minimum_outer: Size::splat(FixedFallbackSource::Unqualified),
        fallback_maximum_outer: Size::splat(FixedFallbackSource::Unqualified),
        node_max_size: Size::NONE,
        available_inner_size: Size::NONE,
        content_box_inset: Edges::ZERO,
        scrollbar_gutter: Edges::ZERO,
        padding: Edges::ZERO,
        border: Edges::ZERO,
    };
    let tracks = [TrackSizingOf::AUTO, TrackSizingOf::AUTO];
    let container_input = compute.node_input(0);
    let container_style = grid_container_projection!(container_input);
    let subgrid_report = collect_subgrid_report(&container_style, &placements);
    let row_tracks =
        expand_track_components(&container_input.grid_template_rows, None, S::ZERO, None).unwrap();
    let lines = GridLines {
        column_explicit_start: 0,
        column_explicit_count: 2,
        row_explicit_start: 0,
        row_explicit_count: 1,
    };
    let named_columns = named::NamedGridLines::new(GridAxisKind::Column, tracks.len());
    let named_rows = named::NamedGridLines::new(GridAxisKind::Row, row_tracks.len());
    let grid = IntrinsicGrid {
        style: &container_style,
        constants: &constants,
        sizing_flow_axes: constants.flow_axes,
        track_basis: constants.flow_axes.logical_size(constants.node_inner_size),
        minimum_functions: GridMinimumParentFunctions {
            columns: Some(GridMinimumTrackFunctions::new(&tracks, None)),
            rows: Some(GridMinimumTrackFunctions::new(&row_tracks, None)),
        },
        column_tracks: &tracks,
        row_tracks: &row_tracks,
        gap: LogicalSizeOf::new(S::ZERO, S::ZERO),
        column_gutters: None,
        row_gutters: None,
        lines,
        named_columns: &named_columns,
        named_rows: &named_rows,
        area_facts: None,
        subgrid_report: &subgrid_report,
        placements: &placements,
        query: IntrinsicGridQuery::complete(GridContributionPurpose::Minimum),
    };
    let rows = lane_intrinsic_contributions(
        &mut compute,
        0,
        LaneIntrinsicTrackSizeInput {
            grid,
            frame: intrinsic_stage::IntrinsicStageFrame {
                axis: GridAxisKind::Column,
                stage: GridSizingStage {
                    first_axis: GridAxisKind::Column,
                    through: GridSizingPhase::InitialFirst,
                },
                columns: None,
                rows: None,
                observed_axis: None,
            },
            container_style: &container_style,
            constants: &constants,
            axis: GridAxisKind::Column,
            tracks: &tracks,
            gap: S::ZERO,
            gutters: None,
            lines,
            placements: &placements,
            subgrid_report: &subgrid_report,
        },
        None,
        false,
    )
    .expect("intrinsic collection succeeds")
    .expect("intrinsic placement is valid")
    .rows;
    assert_eq!(rows.len(), 2);
    for (span, value) in [
        (GridTrackSpan::new(1, 2), S::from_f64(80.0)),
        (GridTrackSpan::new(1, 3), S::from_f64(100.0)),
    ] {
        let row = rows
            .iter()
            .find(|row| row.span == span)
            .expect("actual Lane span is retained");
        assert_eq!(
            [
                row.contributions.minimum,
                row.contributions.min_content,
                row.contributions.max_content
            ],
            [value; 3],
            "fixed authored widths retain each actual purpose at its source span",
        );
    }
    let intrinsic_phases = resolve_grid_contribution_rows(
        SpanningTrackContext {
            tracks: &tracks,
            basis: None,
            available: GridTrackSizingConstraint {
                operation: SizingOperation::MaxContent,
                available_extent: None,
            },
            gap: S::ZERO,
            gutters: None,
            base_floors: &[],
        },
        &rows,
    )
    .expect("collected row phases are valid");
    assert_eq!(
        intrinsic_phases.base_sizes,
        vec![S::from_f64(80.0), S::from_f64(20.0)],
        "overlapping definite spans use the ordinary contribution leveling phase in order-modified sequence"
    );
    // Grid 1 §11.5 carries the existing first-track occupancy through the
    // spanning phases: a 100-wide span needs only the remaining 20 after 80.
    assert_eq!(
        intrinsic_phases.growth_limits,
        vec![Some(S::from_f64(80.0)), Some(S::from_f64(20.0))],
    );

    let output = compute_grid(
        &mut compute,
        0,
        ComputeInputOf::for_child(
            RunMode::PerformLayout,
            SizingMode::InherentSize,
            RequestedAxis::Both,
            Size::NONE,
            Size::NONE,
            ContainingLayoutContext::new(
                FlowAxes::new(WritingMode::HorizontalTb, Direction::Ltr),
                ParentFormattingContext::NoParent,
            ),
            SizingRequestOf::new(
                Size::new(
                    AxisSizingRequestOf::new(
                        AvailableSpaceOf::Indefinite,
                        SizingOperation::MaxContent,
                    ),
                    AxisSizingRequestOf::new(
                        AvailableSpaceOf::Indefinite,
                        SizingOperation::MaxContent,
                    ),
                ),
                OrthogonalFallbackOf::new(
                    Size::NONE,
                    crate::NearestScrollportOf::Absent,
                    Size::splat(crate::OrthogonalFallbackLimitOf::Unbounded),
                ),
            ),
        ),
    )
    .expect("order-modified grid-lanes layout succeeds");

    assert_eq!(output.size.width, S::from_f64(100.0));
    for (node, expected_y) in [(3, 0.0), (5, 20.0), (1, 50.0), (6, 60.0)] {
        let layout = tree.layout(node).expect("in-flow lane child is staged");
        assert_eq!(layout.source_index, SourceIndex::new((node - 1) as usize));
        assert_eq!(layout.location.y, S::from_f64(expected_y));
    }
    assert_eq!(
        Traverse::children(&tree, 0).collect::<Vec<_>>(),
        [1, 2, 3, 4, 5, 6],
        "final traversal stays in source order"
    );
    assert!(tree.inputs(2).iter().any(|input| {
        input.run_mode() == RunMode::PerformHiddenLayout
            && input.parent_formatting_context() == ParentFormattingContext::Grid
    }));
    assert_eq!(
        tree.layout(4)
            .expect("absolute lane child is staged")
            .source_index,
        SourceIndex::new(3)
    );
}

#[test]
fn grid_lanes_replaced_normal_preplacement_starts_while_explicit_stretch_remains_in_both_scalar_lanes()
 {
    assert_grid_lanes_replaced_normal_preplacement::<f32>();
    assert_grid_lanes_replaced_normal_preplacement::<f64>();
}

fn assert_grid_lanes_replaced_normal_preplacement<S: LayoutScalar>()
where
    OracleTreeOf<S>: Traverse<Node = u32, Scalar = S>,
{
    for grid_axis in [GridAxisKind::Column, GridAxisKind::Row] {
        for (label, replaced, item_alignment, container_alignment, stretches) in [
            ("replaced normal", true, None, None, false),
            ("non-replaced normal", false, None, None, true),
            (
                "explicit item stretch",
                true,
                Some(AlignItems::Stretch),
                None,
                true,
            ),
            (
                "explicit container stretch",
                true,
                None,
                Some(AlignItems::Stretch),
                true,
            ),
        ] {
            let grid_axis_size = match grid_axis {
                GridAxisKind::Column => S::from_f64(100.0),
                GridAxisKind::Row => S::from_f64(80.0),
            };
            let measured =
                ComputeOutputOf::from_outer_size(Size::new(S::from_f64(30.0), S::from_f64(20.0)));
            let container_style = NodeInputOf {
                justify_items: container_alignment,
                align_items: container_alignment,
                ..NodeInputOf::default()
            };
            let child_style = NodeInputOf {
                item_is_replaced: replaced,
                justify_self: item_alignment,
                align_self: item_alignment,
                ..NodeInputOf::default()
            };
            let mut tree = OracleTreeOf::<S>::new()
                .children(1, [])
                .style(1, child_style.clone())
                .measure(1, measured);
            let container_projection = grid_container_projection!(&container_style);
            let child_projection = grid_item_projection!(&child_style);
            let constants = Constants {
                child_orthogonal_fallback: OrthogonalFallbackOf::new(
                    Size::NONE,
                    crate::NearestScrollportOf::Absent,
                    Size::splat(crate::OrthogonalFallbackLimitOf::Unknown),
                ),
                flow_axes: FlowAxes::new(WritingMode::HorizontalTb, Direction::Ltr),
                explicit_definite_content_size: Size::NONE,
                indefinite_percentage_axis: None,
                node_outer_size: Size::NONE,
                node_inner_size: Size::NONE,
                node_min_size: Size::NONE,
                node_definite_min_size: Size::NONE,
                fallback_minimum_outer: Size::splat(FixedFallbackSource::Unqualified),
                fallback_maximum_outer: Size::splat(FixedFallbackSource::Unqualified),
                node_max_size: Size::NONE,
                available_inner_size: Size::NONE,
                content_box_inset: Edges::ZERO,
                scrollbar_gutter: Edges::ZERO,
                padding: Edges::ZERO,
                border: Edges::ZERO,
            };
            let measurement = prepare_lane_axis_margin_box_measurement(
                &tree.borrow_compute(),
                1,
                LaneAxisMarginBoxMeasureInput {
                    child_style: &child_projection,
                    container_style: &container_projection,
                    constants: &constants,
                    stacking_operation: SizingOperation::Normal,
                    inherits_grid_axis: false,
                    lane_axis: match grid_axis {
                        GridAxisKind::Column => GridAxisKind::Row,
                        GridAxisKind::Row => GridAxisKind::Column,
                    },
                    containing_block: GridLanesItemContainingBlockOf::new(
                        constants.flow_axes,
                        grid_axis,
                        grid_axis_size,
                        LogicalSizeOf::new(None, None),
                    ),
                },
            )
            .expect("grid-lanes pre-placement request is valid");
            tree.borrow_compute()
                .compute_child(1, measurement.input())
                .expect("grid-lanes pre-placement measurement succeeds");

            let input = tree
                .inputs(1)
                .last()
                .expect("measurement input is recorded");
            let expected = stretches.then_some(grid_axis_size);
            assert_eq!(
                match grid_axis {
                    GridAxisKind::Column => input.known().width,
                    GridAxisKind::Row => input.known().height,
                },
                expected,
                "{label} must resolve pre-placement {grid_axis:?} alignment"
            );
        }
    }
}

#[test]
fn ordinary_grid_order_modified_placement_precedes_mixed_phases_and_preserves_source_identity_in_both_scalar_lanes()
 {
    assert_ordinary_grid_order_modified_placement::<f32>();
    assert_ordinary_grid_order_modified_placement::<f64>();
}

fn assert_ordinary_grid_order_modified_placement<S: LayoutScalar>()
where
    OracleTreeOf<S>: Traverse<Node = u32, Scalar = S>,
{
    for auto_flow in [
        GridAutoFlow::Row,
        GridAutoFlow::RowDense,
        GridAutoFlow::Column,
        GridAutoFlow::ColumnDense,
    ] {
        let column_flow = auto_flow.is_column();
        let item_style = |order| NodeInputOf::<S> {
            item_order: ItemOrder::new(order),
            size: Size::splat_clone(PreferredSizeOf::px(S::from_f64(10.0))),
            ..NodeInputOf::default()
        };
        let definite_major = |order| NodeInputOf::<S> {
            grid_column: if column_flow {
                GridPlacement::try_line(1).expect("one is a valid grid line")
            } else {
                GridPlacement::AUTO
            },
            grid_row: if column_flow {
                GridPlacement::AUTO
            } else {
                GridPlacement::try_line(1).expect("one is a valid grid line")
            },
            ..item_style(order)
        };
        let mut tree = OracleTreeOf::<S>::new()
            .children(0, [1, 2, 3, 4, 5, 6, 7, 8])
            .children(1, [])
            .children(2, [])
            .children(3, [])
            .children(4, [])
            .children(5, [])
            .children(6, [])
            .children(7, [])
            .children(8, [])
            .style(
                0,
                NodeInputOf {
                    display: Display::Grid,
                    size: Size::splat_clone(PreferredSizeOf::px(S::from_f64(80.0))),
                    grid_template_columns: vec![TrackComponentOf::px(S::from_f64(20.0)); 4],
                    grid_template_rows: vec![TrackComponentOf::px(S::from_f64(20.0)); 4],
                    grid_auto_flow: auto_flow,
                    ..NodeInputOf::default()
                },
            )
            .style(1, item_style(2))
            .style(
                2,
                NodeInputOf {
                    display: Display::None,
                    item_order: ItemOrder::new(i32::MIN),
                    ..item_style(0)
                },
            )
            .style(
                3,
                NodeInputOf {
                    grid_column: GridPlacement::try_line(2).expect("two is a valid grid line"),
                    grid_row: GridPlacement::try_line(2).expect("two is a valid grid line"),
                    item_order: ItemOrder::new(i32::MAX),
                    ..item_style(0)
                },
            )
            .style(4, definite_major(2))
            .style(
                5,
                NodeInputOf {
                    position: Position::Absolute,
                    item_order: ItemOrder::new(i32::MIN),
                    ..item_style(0)
                },
            )
            .style(6, definite_major(-1))
            .style(7, item_style(0))
            .style(8, item_style(2));

        let computation = compute_grid_with_report(
            &mut tree.borrow_compute(),
            0,
            ComputeInputOf::for_child(
                RunMode::PerformLayout,
                SizingMode::InherentSize,
                RequestedAxis::Both,
                Size::NONE,
                Size::splat(Some(S::from_f64(80.0))),
                ContainingLayoutContext::new(
                    FlowAxes::new(WritingMode::HorizontalTb, Direction::Ltr),
                    ParentFormattingContext::NoParent,
                ),
                SizingRequestOf::new(
                    Size::splat(AxisSizingRequestOf::new(
                        AvailableSpaceOf::Definite(
                            crate::NonNegativeFiniteOf::new(S::from_f64(80.0)).unwrap(),
                        ),
                        SizingOperation::Normal,
                    )),
                    OrthogonalFallbackOf::new(
                        Size::NONE,
                        crate::NearestScrollportOf::Absent,
                        Size::splat(crate::OrthogonalFallbackLimitOf::Unbounded),
                    ),
                ),
            ),
        )
        .expect("order-modified ordinary-grid layout succeeds");
        assert!(computation.report().is_empty());

        let expected_cells = if column_flow {
            [
                (6, 0, 0),
                (3, 1, 1),
                (4, 0, 1),
                (7, 0, 2),
                (1, 0, 3),
                (8, 1, 0),
            ]
        } else {
            [
                (6, 0, 0),
                (3, 1, 1),
                (4, 1, 0),
                (7, 2, 0),
                (1, 3, 0),
                (8, 0, 1),
            ]
        };
        for (node, column, row) in expected_cells {
            let layout = tree.layout(node).expect("in-flow child layout is staged");
            assert_eq!(layout.source_index, SourceIndex::new((node - 1) as usize));
            assert_eq!(
                layout.location,
                Point::new(S::from_usize(column * 20), S::from_usize(row * 20)),
                "{auto_flow:?} node {node} must use order-modified placement"
            );
        }
        assert!(
            tree.inputs(2)
                .iter()
                .any(|input| input.run_mode() == RunMode::PerformHiddenLayout),
            "hidden children remain outside the ordinary-grid permutation"
        );
        assert_eq!(
            tree.layout(5)
                .expect("absolute child layout is staged")
                .source_index,
            SourceIndex::new(4)
        );
    }
}

#[test]
fn fri04_c03_grid_track_grid_and_lanes_consume_nested_properties_on_both_axes_and_clamp_results() {
    for display in [Display::Grid, Display::GridLanes] {
        let mut tree = OracleTree::new().children(1, []).style(
            1,
            NodeInput {
                display,
                size: Size::new(
                    PreferredSize::calculation(fri04_c03_grid_track_nested(20.0, 80.0, 120.0)),
                    PreferredSize::calculation(fri04_c03_grid_track_nested(20.0, 70.0, 120.0)),
                ),
                min_size: Size::new(
                    MinSize::calculation(fri04_c03_grid_track_nested(40.0, 90.0, 110.0)),
                    MinSize::calculation(fri04_c03_grid_track_nested(30.0, 60.0, 90.0)),
                ),
                max_size: Size::new(
                    MaxSize::calculation(fri04_c03_grid_track_nested(30.0, 85.0, 100.0)),
                    MaxSize::calculation(fri04_c03_grid_track_nested(30.0, 65.0, 100.0)),
                ),
                ..NodeInput::default()
            },
        );

        let output = compute_grid(
            &mut tree.borrow_compute(),
            1,
            ComputeInput::for_child(
                RunMode::PerformLayout,
                SizingMode::InherentSize,
                RequestedAxis::Both,
                Size::NONE,
                Size::splat(Some(200.0)),
                ContainingLayoutContext::new(
                    FlowAxes::new(WritingMode::HorizontalTb, Direction::Ltr),
                    ParentFormattingContext::NoParent,
                ),
                SizingRequestOf::new(
                    Size::splat(AxisSizingRequestOf::new(
                        AvailableSpaceOf::Definite(crate::NonNegativeFiniteOf::new(200.0).unwrap()),
                        SizingOperation::Normal,
                    )),
                    OrthogonalFallbackOf::new(
                        Size::NONE,
                        crate::NearestScrollportOf::Absent,
                        Size::splat(crate::OrthogonalFallbackLimitOf::Unbounded),
                    ),
                ),
            ),
        )
        .expect("grid property calculations resolve");

        // The resolved width minimum of 90 wins over the maximum of 85.
        // The height bounds are ordered, so the maximum still caps 70 at 65.
        assert_eq!(output.size, Size::new(90.0, 65.0), "{display:?}");

        let mut negative_tree = OracleTree::new().children(1, []).style(
            1,
            NodeInput {
                display,
                size: Size::new(
                    PreferredSize::calculation(fri04_c03_grid_track_nested(-40.0, -20.0, -10.0)),
                    PreferredSize::calculation(fri04_c03_grid_track_nested(-30.0, -15.0, -5.0)),
                ),
                ..NodeInput::default()
            },
        );
        let negative = compute_grid(
            &mut negative_tree.borrow_compute(),
            1,
            ComputeInput::for_child(
                RunMode::PerformLayout,
                SizingMode::InherentSize,
                RequestedAxis::Both,
                Size::NONE,
                Size::splat(Some(200.0)),
                ContainingLayoutContext::new(
                    FlowAxes::new(WritingMode::HorizontalTb, Direction::Ltr),
                    ParentFormattingContext::NoParent,
                ),
                SizingRequestOf::new(
                    Size::splat(AxisSizingRequestOf::new(
                        AvailableSpaceOf::Definite(crate::NonNegativeFiniteOf::new(200.0).unwrap()),
                        SizingOperation::Normal,
                    )),
                    OrthogonalFallbackOf::new(
                        Size::NONE,
                        crate::NearestScrollportOf::Absent,
                        Size::splat(crate::OrthogonalFallbackLimitOf::Unbounded),
                    ),
                ),
            ),
        )
        .expect("negative complete calculations clamp after evaluation");
        assert_eq!(negative.size, Size::ZERO, "{display:?}");
    }
}

#[test]
fn fri04_c03_grid_track_grid_and_lanes_preserve_missing_and_definite_property_bases() {
    for display in [Display::Grid, Display::GridLanes] {
        let style = NodeInput {
            display,
            size: Size::new(
                PreferredSize::calculation(fri04_c03_grid_track_percentage_nested(10.0, 0.5, 80.0)),
                PreferredSize::calculation(fri04_c03_grid_track_percentage_nested(
                    10.0, 0.25, 80.0,
                )),
            ),
            min_size: Size::new(
                MinSize::calculation(fri04_c03_grid_track_percentage_nested(10.0, 0.6, 80.0)),
                MinSize::calculation(fri04_c03_grid_track_percentage_nested(10.0, 0.3, 80.0)),
            ),
            max_size: Size::new(
                MaxSize::calculation(fri04_c03_grid_track_percentage_nested(10.0, 0.625, 80.0)),
                MaxSize::calculation(fri04_c03_grid_track_percentage_nested(10.0, 0.325, 80.0)),
            ),
            ..NodeInput::default()
        };
        let mut definite_tree = OracleTree::new().children(1, []).style(1, style.clone());
        let definite = compute_grid(
            &mut definite_tree.borrow_compute(),
            1,
            ComputeInput::for_child(
                RunMode::PerformLayout,
                SizingMode::InherentSize,
                RequestedAxis::Both,
                Size::NONE,
                Size::new(Some(120.0), Some(200.0)),
                ContainingLayoutContext::new(
                    FlowAxes::new(WritingMode::HorizontalTb, Direction::Ltr),
                    ParentFormattingContext::NoParent,
                ),
                SizingRequestOf::new(
                    Size::new(
                        AxisSizingRequestOf::new(
                            AvailableSpaceOf::Definite(
                                crate::NonNegativeFiniteOf::new(120.0).unwrap(),
                            ),
                            SizingOperation::Normal,
                        ),
                        AxisSizingRequestOf::new(
                            AvailableSpaceOf::Definite(
                                crate::NonNegativeFiniteOf::new(200.0).unwrap(),
                            ),
                            SizingOperation::Normal,
                        ),
                    ),
                    OrthogonalFallbackOf::new(
                        Size::NONE,
                        crate::NearestScrollportOf::Absent,
                        Size::splat(crate::OrthogonalFallbackLimitOf::Unbounded),
                    ),
                ),
            ),
        )
        .expect("definite property bases resolve");
        assert_eq!(definite.size.width, 72.0, "{display:?} width");
        assert!(
            (definite.size.height - 60.0).abs() <= 0.000_01,
            "{display:?} height: {}",
            definite.size.height
        );

        let mut missing_tree = OracleTree::new().children(1, []).style(1, style);
        let missing = compute_grid(
            &mut missing_tree.borrow_compute(),
            1,
            ComputeInput::for_child(
                RunMode::ComputeSize,
                SizingMode::InherentSize,
                RequestedAxis::Both,
                Size::NONE,
                Size::NONE,
                ContainingLayoutContext::new(
                    FlowAxes::new(WritingMode::HorizontalTb, Direction::Ltr),
                    ParentFormattingContext::NoParent,
                ),
                SizingRequestOf::new(
                    Size::splat(AxisSizingRequestOf::new(
                        AvailableSpaceOf::Indefinite,
                        SizingOperation::MaxContent,
                    )),
                    OrthogonalFallbackOf::new(
                        Size::NONE,
                        crate::NearestScrollportOf::Absent,
                        Size::splat(crate::OrthogonalFallbackLimitOf::Unbounded),
                    ),
                ),
            ),
        )
        .expect("missing property bases retain intrinsic sizing");
        assert_eq!(missing.size, Size::ZERO, "{display:?}");
    }
}

#[test]
fn fri04_c03_grid_track_grid_and_lanes_propagate_invalid_property_numeric() {
    let invalid = SizingCalculation::max(vec![
        fri04_c03_grid_track_value(10.0),
        SizingCalculation::value(invalid_numeric_lp()),
    ])
    .expect("nested maximum is nonempty");
    for display in [Display::Grid, Display::GridLanes] {
        for property in ["preferred", "minimum", "maximum"] {
            for axis in [PhysicalAxis::Horizontal, PhysicalAxis::Vertical] {
                let mut style = NodeInput {
                    display,
                    size: Size::new(PreferredSize::px(10.0), PreferredSize::px(10.0)),
                    ..NodeInput::default()
                };
                match (property, axis) {
                    ("preferred", PhysicalAxis::Horizontal) => {
                        style.size.width = PreferredSize::calculation(invalid.clone())
                    }
                    ("preferred", PhysicalAxis::Vertical) => {
                        style.size.height = PreferredSize::calculation(invalid.clone())
                    }
                    ("minimum", PhysicalAxis::Horizontal) => {
                        style.min_size.width = MinSize::calculation(invalid.clone())
                    }
                    ("minimum", PhysicalAxis::Vertical) => {
                        style.min_size.height = MinSize::calculation(invalid.clone())
                    }
                    ("maximum", PhysicalAxis::Horizontal) => {
                        style.max_size.width = MaxSize::calculation(invalid.clone())
                    }
                    ("maximum", PhysicalAxis::Vertical) => {
                        style.max_size.height = MaxSize::calculation(invalid.clone())
                    }
                    _ => unreachable!("property and physical axis matrix is exhaustive"),
                }
                let mut tree = OracleTree::new().children(1, []).style(1, style);
                let error = compute_grid(
                    &mut tree.borrow_compute(),
                    1,
                    ComputeInput::for_child(
                        RunMode::PerformLayout,
                        SizingMode::InherentSize,
                        RequestedAxis::Both,
                        Size::NONE,
                        Size::splat(Some(f32::MAX)),
                        ContainingLayoutContext::new(
                            FlowAxes::new(WritingMode::HorizontalTb, Direction::Ltr),
                            ParentFormattingContext::NoParent,
                        ),
                        SizingRequestOf::new(
                            Size::splat(AxisSizingRequestOf::new(
                                AvailableSpaceOf::Definite(
                                    crate::NonNegativeFiniteOf::new(f32::MAX).unwrap(),
                                ),
                                SizingOperation::Normal,
                            )),
                            OrthogonalFallbackOf::new(
                                Size::NONE,
                                crate::NearestScrollportOf::Absent,
                                Size::splat(crate::OrthogonalFallbackLimitOf::Unbounded),
                            ),
                        ),
                    ),
                )
                .expect_err("invalid nested grid property numeric must fail");
                assert!(
                    matches!(
                        error.kind(),
                        LayoutErrorKind::InvalidInput(LayoutInvalidInput::InvalidNumeric { value })
                            if value.is_infinite()
                    ),
                    "{display:?} {property} {axis:?}"
                );
            }
        }
    }
}

#[test]
fn fri04_c03_grid_track_lanes_resolve_nested_track_bases_and_propagate_invalid_numeric() {
    let report = lane_intrinsic_sizing(LaneIntrinsicSizingInput {
        axis: GridAxisKind::Column,
        available: None,
        gap: 0.0,
        tracks: vec![TrackSizing::new(
            MinTrackSizing::Calculation(fri04_c03_grid_track_nested(20.0, 40.0, 60.0)),
            MaxTrackSizing::Auto,
        )],
        content_sized_tracks: vec![0],
        items: Vec::new(),
    })
    .expect("nested lane track sizing resolves")
    .expect("nested lane placement is valid");
    assert_eq!(report.final_track_sizes, [40.0]);

    let invalid = SizingCalculation::max(vec![
        fri04_c03_grid_track_value(10.0),
        SizingCalculation::value(invalid_numeric_lp()),
    ])
    .expect("nested maximum is nonempty");
    let error = lane_intrinsic_sizing(LaneIntrinsicSizingInput {
        axis: GridAxisKind::Row,
        available: Some(f32::MAX),
        gap: 0.0,
        tracks: vec![TrackSizing::new(
            MinTrackSizing::Calculation(invalid),
            MaxTrackSizing::Auto,
        )],
        content_sized_tracks: vec![0],
        items: Vec::new(),
    })
    .expect_err("invalid nested lane track numeric must fail");
    assert!(matches!(
        error.kind(),
        LayoutErrorKind::InvalidInput(LayoutInvalidInput::InvalidNumeric { value })
            if value.is_infinite()
    ));
}

#[test]
fn lane_intrinsic_item_exposes_exactly_one_kind() {
    let contribution = LaneContributionFacts {
        min_content: 1.0,
        max_content: 2.0,
        min_size: 0.0,
        automatic_minimum_applies: true,
    };
    let span = LaneTrackSpanLength::new(2).expect("span should be nonzero");
    let item = LaneIntrinsicItem::indefinite("item", span, contribution);

    assert!(matches!(
        item.kind(),
        LaneIntrinsicItemKind::Indefinite { span } if span.get() == 2
    ));
}

#[test]
fn lane_intrinsic_item_rejects_malformed_definite_span_without_track_context() {
    let contribution = LaneContributionFacts {
        min_content: 1.0,
        max_content: 2.0,
        min_size: 0.0,
        automatic_minimum_applies: true,
    };
    let span = LaneTrackSpan::new(0, 1);
    let err = LaneIntrinsicItem::definite("item", span, contribution)
        .expect_err("malformed definite span should be rejected at construction");

    assert_eq!(err, LanePlacementError::InvalidDefiniteLaneSpan { span });
}

#[test]
fn lane_track_span_length_rejects_zero() {
    assert!(LaneTrackSpanLength::new(0).is_none());
}

#[test]
fn lane_errors_carry_context() {
    let err = place_lanes(LanePlacementInput {
        grid_axis_tracks: 2,
        grid_axis: crate::GridAxisKind::Column,
        reversed: false,
        lane_gap: 0.0,
        tolerance: GridFlowTolerance::Infinite,
        tolerance_basis: 0.0,
        items: vec![LaneItem {
            item: "item",
            grid_axis_span: 1,
            definite_grid_axis_start: Some(0),
            lane_axis_margin_box: 1.0,
        }],
    })
    .expect_err("zero grid axis start should be rejected with context");

    assert_eq!(err, LanePlacementError::InvalidGridAxisStart { start: 0 });
}

#[test]
fn lanes_reject_invalid_raw_tolerance_basis() {
    let err = place_lanes(LanePlacementInput::<&str> {
        grid_axis_tracks: 2,
        grid_axis: crate::GridAxisKind::Column,
        reversed: false,
        lane_gap: 0.0,
        tolerance: GridFlowTolerance::Percent(0.25),
        tolerance_basis: f32::NAN,
        items: Vec::new(),
    })
    .expect_err("invalid raw tolerance basis should return a typed error");

    assert_eq!(err, LanePlacementError::InvalidGridFlowToleranceBasis);
}

#[test]
fn grid_lanes_content_size_uses_measured_lane_margin_boxes() {
    #[derive(Default)]
    struct RecursiveTree {
        children: HashMap<u32, Vec<u32>>,
        styles: HashMap<u32, NodeInput>,
        layouts: HashMap<u32, crate::LayoutOutputEntryOf<u32, Scalar>>,
        outputs: HashMap<u32, ComputeOutput>,
    }

    impl<'input> RecursiveTreeCompute<'input> {
        fn compute_node(
            &mut self,
            node: u32,
            input: ComputeInput,
        ) -> LayoutResultOf<u32, ComputeOutput, Scalar> {
            let node_input = self.styles[&node].clone();
            if self.children[&node].is_empty() {
                return Ok(self.outputs[&node]);
            }

            match node_input.display.inner_display() {
                Display::Grid | Display::GridLanes => crate::compute_grid(self, node, input),
                Display::Block => crate::compute_block(self, node, input),
                Display::Flex => compute_flex(self, node, input),
                Display::None => Ok(ComputeOutput::HIDDEN),
                Display::InlineBlock | Display::InlineGrid | Display::InlineGridLanes => {
                    unreachable!("inner_display removes inline display variants")
                }
            }
        }
    }

    impl Traverse for RecursiveTree {
        type Node = u32;

        type Scalar = Scalar;
        type Children<'a>
            = std::iter::Copied<std::slice::Iter<'a, u32>>
        where
            Self: 'a;

        fn children(&self, node: Self::Node) -> Self::Children<'_> {
            self.children[&node].iter().copied()
        }

        fn child_count(&self, node: Self::Node) -> usize {
            self.children[&node].len()
        }

        fn child(&self, node: Self::Node, index: usize) -> Self::Node {
            self.children[&node][index]
        }
    }

    struct RecursiveTreeCompute<'input> {
        children: &'input HashMap<u32, Vec<u32>>,
        styles: &'input HashMap<u32, NodeInput>,
        layouts: &'input mut HashMap<u32, crate::LayoutOutputEntryOf<u32, Scalar>>,
        outputs: &'input mut HashMap<u32, ComputeOutput>,
    }

    impl RecursiveTree {
        fn borrow_compute(&mut self) -> RecursiveTreeCompute<'_> {
            RecursiveTreeCompute {
                children: &self.children,
                styles: &self.styles,
                layouts: &mut self.layouts,
                outputs: &mut self.outputs,
            }
        }
    }

    impl<'input> Traverse for RecursiveTreeCompute<'input> {
        type Node = u32;

        type Scalar = Scalar;
        type Children<'a>
            = std::iter::Copied<std::slice::Iter<'a, u32>>
        where
            Self: 'a;

        fn children(&self, node: Self::Node) -> Self::Children<'_> {
            self.children[&node].iter().copied()
        }

        fn child_count(&self, node: Self::Node) -> usize {
            self.children[&node].len()
        }

        fn child(&self, node: Self::Node, index: usize) -> Self::Node {
            self.children[&node][index]
        }
    }

    impl<'input> Compute<'input> for RecursiveTreeCompute<'input> {
        fn observe_grid_overflow(
            &mut self,
            _node: Self::Node,
            _probe: crate::grid::GridOverflowProbe<'input, '_, Self::Node, Self::Scalar>,
        ) -> crate::LayoutResultOf<
            Self::Node,
            crate::grid::GridOverflowObservation<Self::Scalar>,
            Self::Scalar,
            core::convert::Infallible,
        > {
            panic!(
                "this focused computation fixture does not admit intrinsic Grid Auto overflow observation"
            );
        }

        fn node_input(&self, node: Self::Node) -> &'input NodeInput {
            &self.styles[&node]
        }

        fn layout_input(&self, node: Self::Node) -> crate::LayoutInputViewOf<'input, Self::Scalar> {
            crate::LayoutInputViewOf::Box(self.node_input(node))
        }

        fn staged_unrounded(
            &self,
            node: Self::Node,
        ) -> crate::engine::contracts::UnroundedEntryResult<
            Self::Node,
            Self::Scalar,
            core::convert::Infallible,
        > {
            self.layouts
                .get(&node)
                .copied()
                .filter(|entry| entry.node() == node && entry.geometry_state().is_unrounded())
                .ok_or_else(|| crate::test_support::layout_tree::missing_staged_unrounded(node))
        }

        fn set_unrounded(&mut self, entry: crate::LayoutOutputEntryOf<Self::Node, Self::Scalar>) {
            self.layouts.insert(entry.node(), entry);
        }

        fn compute_child(
            &mut self,
            node: Self::Node,
            input: ComputeInput,
        ) -> crate::LayoutResultOf<Self::Node, crate::ComputeOutputOf<Self::Scalar>, Self::Scalar>
        {
            self.compute_node(node, input)
        }
    }

    let mut tree = RecursiveTree::default();
    tree.children.insert(1, vec![2, 3]);
    tree.children.insert(2, vec![]);
    tree.children.insert(3, vec![]);
    tree.styles.insert(
        1,
        NodeInput {
            display: Display::GridLanes,
            grid_template_columns: vec![TrackComponent::px(20.0)],
            grid_template_rows: vec![TrackComponent::px(0.0)],
            gap: Size::new(Length::ZERO, Length::px(5.0)),
            grid_flow_tolerance: GridFlowTolerance::Length(Length::ZERO),
            ..NodeInput::default()
        },
    );
    for child in [2, 3] {
        tree.styles.insert(
            child,
            NodeInput {
                grid_column: GridPlacement::try_line(1).expect("valid grid line"),
                grid_row: GridPlacement::try_line(1).expect("valid grid line"),
                ..NodeInput::default()
            },
        );
    }
    tree.outputs
        .insert(2, ComputeOutput::from_outer_size(Size::new(20.0, 10.0)));
    tree.outputs
        .insert(3, ComputeOutput::from_outer_size(Size::new(20.0, 15.0)));

    let output = tree
        .borrow_compute()
        .compute_node(
            1,
            ComputeInput::for_child(
                RunMode::PerformLayout,
                SizingMode::InherentSize,
                RequestedAxis::Both,
                Size::NONE,
                Size::NONE,
                crate::ContainingLayoutContext::new(
                    crate::geometry::FlowAxes::new(
                        crate::WritingMode::HorizontalTb,
                        crate::Direction::Ltr,
                    ),
                    crate::ParentFormattingContext::NoParent,
                ),
                SizingRequestOf::new(
                    Size::new(
                        AxisSizingRequestOf::new(
                            AvailableSpaceOf::Indefinite,
                            SizingOperation::MaxContent,
                        ),
                        AxisSizingRequestOf::new(
                            AvailableSpaceOf::Indefinite,
                            SizingOperation::MaxContent,
                        ),
                    ),
                    OrthogonalFallbackOf::new(
                        Size::NONE,
                        crate::NearestScrollportOf::Absent,
                        Size::splat(crate::OrthogonalFallbackLimitOf::Unbounded),
                    ),
                ),
            ),
        )
        .unwrap();

    assert_eq!(output.content_size, Size::new(20.0, 30.0));
    assert_eq!(tree.layouts[&2].output().size, Size::new(20.0, 10.0));
    assert_eq!(tree.layouts[&3].output().size, Size::new(20.0, 15.0));
}

#[test]
fn grid_lanes_content_size_uses_stacking_range_for_intrinsic_requests() {
    let mut tree = OracleTree::new()
        .children(1, [2])
        .style(
            1,
            NodeInput {
                display: Display::GridLanes,
                grid_template_columns: vec![TrackComponent::px(80.0)],
                grid_template_rows: vec![TrackComponent::px(20.0)],
                ..NodeInput::default()
            },
        )
        .style(2, NodeInput::default())
        .measure(2, ComputeOutput::from_outer_size(Size::new(20.0, 10.0)));

    for (operation, expected_height) in [
        (SizingOperation::Normal, 10.0),
        (SizingOperation::MaxContent, 10.0),
    ] {
        let output = crate::compute_grid(
            &mut tree.borrow_compute(),
            1,
            ComputeInput::for_child(
                RunMode::PerformLayout,
                SizingMode::InherentSize,
                RequestedAxis::Both,
                Size::NONE,
                Size::NONE,
                crate::ContainingLayoutContext::new(
                    crate::geometry::FlowAxes::new(
                        crate::WritingMode::HorizontalTb,
                        crate::Direction::Ltr,
                    ),
                    crate::ParentFormattingContext::NoParent,
                ),
                SizingRequestOf::new(
                    Size::new(
                        AxisSizingRequestOf::new(
                            AvailableSpaceOf::Indefinite,
                            SizingOperation::MaxContent,
                        ),
                        AxisSizingRequestOf::new(AvailableSpaceOf::Indefinite, operation),
                    ),
                    OrthogonalFallbackOf::new(
                        Size::NONE,
                        crate::NearestScrollportOf::Absent,
                        Size::splat(crate::OrthogonalFallbackLimitOf::Unbounded),
                    ),
                ),
            ),
        )
        .unwrap();

        assert_eq!(output.content_size, Size::new(80.0, expected_height));
    }
}

#[test]
fn named_grid_lanes_use_resolved_raw_grid_axis_placement() {
    let mut tree = OracleTree::new()
        .children(1, [2])
        .style(
            1,
            NodeInput {
                display: Display::GridLanes,
                size: Size::new(PreferredSize::px(120.0), PreferredSize::px(20.0)),
                grid_template_columns: vec![
                    TrackComponent::px(40.0),
                    TrackComponent::line_names(["lane"]),
                    TrackComponent::px(40.0),
                    TrackComponent::px(40.0),
                ],
                grid_template_rows: vec![TrackComponent::px(20.0)],
                grid_flow_tolerance: GridFlowTolerance::Length(Length::ZERO),
                ..NodeInput::DEFAULT
            },
        )
        .style(
            2,
            NodeInput {
                raw_grid_column: RawGridPlacement::new(
                    RawGridLine::NamedLine {
                        name: "lane".to_string(),
                        index: 1,
                    },
                    RawGridLine::Auto,
                ),
                ..NodeInput::DEFAULT
            },
        )
        .measure(2, ComputeOutput::from_outer_size(Size::new(40.0, 20.0)));

    compute_oracle_grid(&mut tree);
    let child = tree.final_layout(2).expect("lane child should be laid out");

    assert_eq!(child.location, Point::new(40.0, 0.0));
    assert_eq!(child.size, Size::new(40.0, 20.0));
}

#[test]
fn named_grid_lanes_intrinsic_sizing_uses_resolved_raw_grid_axis_placement() {
    let mut tree = OracleTree::new()
        .children(1, [2, 3])
        .style(
            1,
            NodeInput {
                display: Display::GridLanes,
                grid_template_columns: vec![
                    TrackComponent::AUTO,
                    TrackComponent::line_names(["lane"]),
                    TrackComponent::AUTO,
                ],
                grid_template_rows: vec![TrackComponent::px(20.0)],
                grid_flow_tolerance: GridFlowTolerance::Length(Length::ZERO),
                ..NodeInput::DEFAULT
            },
        )
        .style(
            2,
            NodeInput {
                raw_grid_column: RawGridPlacement::line(1),
                ..NodeInput::DEFAULT
            },
        )
        .style(
            3,
            NodeInput {
                raw_grid_column: RawGridPlacement::new(
                    RawGridLine::NamedLine {
                        name: "lane".to_string(),
                        index: 1,
                    },
                    RawGridLine::Auto,
                ),
                ..NodeInput::DEFAULT
            },
        )
        .measure(2, ComputeOutput::from_outer_size(Size::new(10.0, 20.0)))
        .measure(3, ComputeOutput::from_outer_size(Size::new(50.0, 20.0)));

    let output = crate::compute_grid(
        &mut tree.borrow_compute(),
        1,
        ComputeInput::for_child(
            RunMode::PerformLayout,
            SizingMode::InherentSize,
            RequestedAxis::Both,
            Size::NONE,
            Size::NONE,
            crate::ContainingLayoutContext::new(
                crate::geometry::FlowAxes::new(
                    crate::WritingMode::HorizontalTb,
                    crate::Direction::Ltr,
                ),
                crate::ParentFormattingContext::NoParent,
            ),
            SizingRequestOf::new(
                Size::new(
                    AxisSizingRequestOf::new(
                        AvailableSpaceOf::Indefinite,
                        SizingOperation::MaxContent,
                    ),
                    AxisSizingRequestOf::new(
                        AvailableSpaceOf::Indefinite,
                        SizingOperation::MaxContent,
                    ),
                ),
                OrthogonalFallbackOf::new(
                    Size::NONE,
                    crate::NearestScrollportOf::Absent,
                    Size::splat(crate::OrthogonalFallbackLimitOf::Unbounded),
                ),
            ),
        ),
    )
    .unwrap();
    let named = tree.layout(3).expect("named lane child should be laid out");

    assert_eq!(output.content_size.width, 60.0);
    assert_eq!(named.location.x, 10.0);
    assert_eq!(named.size.width, 50.0);
}

#[test]
fn named_grid_lanes_resolve_repeated_named_start_and_end_lines() {
    let mut tree = OracleTree::new()
        .children(1, [2])
        .style(
            1,
            NodeInput {
                display: Display::GridLanes,
                grid_template_columns: vec![
                    TrackComponent::line_names(["lane"]),
                    TrackComponent::px(20.0),
                    TrackComponent::line_names(["lane"]),
                    TrackComponent::px(30.0),
                    TrackComponent::line_names(["lane"]),
                    TrackComponent::px(40.0),
                    TrackComponent::line_names(["lane"]),
                ],
                ..NodeInput::DEFAULT
            },
        )
        .style(
            2,
            NodeInput {
                raw_grid_column: RawGridPlacement::new(
                    RawGridLine::NamedLine {
                        name: "lane".to_string(),
                        index: 2,
                    },
                    RawGridLine::NamedLine {
                        name: "lane".to_string(),
                        index: 4,
                    },
                ),
                ..NodeInput::DEFAULT
            },
        )
        .measure(2, ComputeOutput::from_outer_size(Size::new(70.0, 0.0)));

    let output = crate::compute_grid(
        &mut tree.borrow_compute(),
        1,
        ComputeInput::for_child(
            RunMode::PerformLayout,
            SizingMode::InherentSize,
            RequestedAxis::Both,
            Size::NONE,
            Size::NONE,
            crate::ContainingLayoutContext::new(
                crate::geometry::FlowAxes::new(
                    crate::WritingMode::HorizontalTb,
                    crate::Direction::Ltr,
                ),
                crate::ParentFormattingContext::NoParent,
            ),
            SizingRequestOf::new(
                Size::new(
                    AxisSizingRequestOf::new(
                        AvailableSpaceOf::Indefinite,
                        SizingOperation::MaxContent,
                    ),
                    AxisSizingRequestOf::new(
                        AvailableSpaceOf::Indefinite,
                        SizingOperation::MaxContent,
                    ),
                ),
                OrthogonalFallbackOf::new(
                    Size::NONE,
                    crate::NearestScrollportOf::Absent,
                    Size::splat(crate::OrthogonalFallbackLimitOf::Unbounded),
                ),
            ),
        ),
    )
    .unwrap();
    let child = tree.layout(2).expect("named lane child should be laid out");

    assert_eq!(output.size, Size::new(90.0, 0.0));
    assert_eq!(child.location.x, 20.0);
    assert_eq!(child.size.width, 70.0);
}

#[test]
fn grid_lanes_with_rows_template_uses_columns_as_lane_axis_for_intrinsic_width() {
    let mut tree = OracleTree::new()
        .children(1, [2, 3, 4, 5])
        .style(
            1,
            NodeInput {
                display: Display::GridLanes,
                grid_lanes_flow: crate::GridLanesFlow::new(crate::GridAxisKind::Row, false, false),
                grid_template_rows: vec![
                    TrackComponent::AUTO,
                    TrackComponent::AUTO,
                    TrackComponent::AUTO,
                ],
                ..NodeInput::default()
            },
        )
        .style(
            5,
            NodeInput {
                grid_row: GridPlacement::try_span(2).expect("valid grid span"),
                ..NodeInput::default()
            },
        )
        .style(2, NodeInput::default())
        .style(3, NodeInput::default())
        .style(4, NodeInput::default())
        .measure(2, ComputeOutput::from_outer_size(Size::new(72.0, 15.0)))
        .measure(3, ComputeOutput::from_outer_size(Size::new(72.0, 15.0)))
        .measure(4, ComputeOutput::from_outer_size(Size::new(72.0, 15.0)))
        .measure(5, ComputeOutput::from_outer_size(Size::new(73.0, 30.0)));

    let output = crate::compute_grid(
        &mut tree.borrow_compute(),
        1,
        ComputeInput::for_child(
            RunMode::PerformLayout,
            SizingMode::InherentSize,
            RequestedAxis::Both,
            Size::NONE,
            Size::NONE,
            crate::ContainingLayoutContext::new(
                crate::geometry::FlowAxes::new(
                    crate::WritingMode::HorizontalTb,
                    crate::Direction::Ltr,
                ),
                crate::ParentFormattingContext::NoParent,
            ),
            SizingRequestOf::new(
                Size::new(
                    AxisSizingRequestOf::new(
                        AvailableSpaceOf::Indefinite,
                        SizingOperation::MaxContent,
                    ),
                    AxisSizingRequestOf::new(
                        AvailableSpaceOf::Indefinite,
                        SizingOperation::MaxContent,
                    ),
                ),
                OrthogonalFallbackOf::new(
                    Size::NONE,
                    crate::NearestScrollportOf::Absent,
                    Size::splat(crate::OrthogonalFallbackLimitOf::Unbounded),
                ),
            ),
        ),
    )
    .unwrap();

    assert_eq!(output.size, Size::new(145.0, 45.0));
    assert_eq!(
        tree.layout(5)
            .expect("spanning item should be laid out")
            .location
            .x,
        72.0
    );
}

#[test]
fn grid_lanes_lane_measurement_honors_min_content_width() {
    let mut tree = OracleTree::new()
        .children(1, [2, 3])
        .style(
            1,
            NodeInput {
                display: Display::GridLanes,
                grid_lanes_flow: crate::GridLanesFlow::new(crate::GridAxisKind::Row, false, false),
                grid_template_rows: vec![TrackComponent::AUTO, TrackComponent::AUTO],
                ..NodeInput::default()
            },
        )
        .style(
            2,
            NodeInput {
                size: Size::new(PreferredSize::MIN_CONTENT, PreferredSize::AUTO),
                ..NodeInput::default()
            },
        )
        .style(
            3,
            NodeInput {
                size: Size::new(PreferredSize::MAX_CONTENT, PreferredSize::AUTO),
                ..NodeInput::default()
            },
        )
        // Block MIN/MAX under a resolved 72px inline area share the same
        // 15px natural block extent; preferred-inline MIN is queried separately.
        .measure_when(
            2,
            OracleMeasurement::new(ComputeOutput::from_outer_size(Size::new(72.0, 15.0)))
                .operations(
                    (Size::new(
                        AxisSizingRequestOf::new(
                            AvailableSpaceOf::Definite(
                                crate::NonNegativeFiniteOf::new(72.0).unwrap(),
                            ),
                            SizingOperation::Normal,
                        ),
                        AxisSizingRequestOf::new(
                            AvailableSpaceOf::Indefinite,
                            SizingOperation::MinContent,
                        ),
                    ))
                    .map(|axis| Some(axis.operation())),
                )
                .spaces(
                    (Size::new(
                        AxisSizingRequestOf::new(
                            AvailableSpaceOf::Definite(
                                crate::NonNegativeFiniteOf::new(72.0).unwrap(),
                            ),
                            SizingOperation::Normal,
                        ),
                        AxisSizingRequestOf::new(
                            AvailableSpaceOf::Indefinite,
                            SizingOperation::MinContent,
                        ),
                    ))
                    .map(|axis| {
                        (axis.operation() == SizingOperation::Normal).then_some(axis.space())
                    }),
                ),
        )
        .measure_when(
            2,
            OracleMeasurement::new(ComputeOutput::from_outer_size(Size::new(72.0, 15.0)))
                .operations(
                    (Size::new(
                        AxisSizingRequestOf::new(
                            AvailableSpaceOf::Definite(
                                crate::NonNegativeFiniteOf::new(72.0).unwrap(),
                            ),
                            SizingOperation::Normal,
                        ),
                        AxisSizingRequestOf::new(
                            AvailableSpaceOf::Indefinite,
                            SizingOperation::MaxContent,
                        ),
                    ))
                    .map(|axis| Some(axis.operation())),
                )
                .spaces(
                    (Size::new(
                        AxisSizingRequestOf::new(
                            AvailableSpaceOf::Definite(
                                crate::NonNegativeFiniteOf::new(72.0).unwrap(),
                            ),
                            SizingOperation::Normal,
                        ),
                        AxisSizingRequestOf::new(
                            AvailableSpaceOf::Indefinite,
                            SizingOperation::MaxContent,
                        ),
                    ))
                    .map(|axis| {
                        (axis.operation() == SizingOperation::Normal).then_some(axis.space())
                    }),
                ),
        )
        .measure_when(
            3,
            OracleMeasurement::new(ComputeOutput::from_outer_size(Size::new(72.0, 15.0)))
                .operations(
                    (Size::new(
                        AxisSizingRequestOf::new(
                            AvailableSpaceOf::Definite(
                                crate::NonNegativeFiniteOf::new(72.0).unwrap(),
                            ),
                            SizingOperation::Normal,
                        ),
                        AxisSizingRequestOf::new(
                            AvailableSpaceOf::Indefinite,
                            SizingOperation::MinContent,
                        ),
                    ))
                    .map(|axis| Some(axis.operation())),
                )
                .spaces(
                    (Size::new(
                        AxisSizingRequestOf::new(
                            AvailableSpaceOf::Definite(
                                crate::NonNegativeFiniteOf::new(72.0).unwrap(),
                            ),
                            SizingOperation::Normal,
                        ),
                        AxisSizingRequestOf::new(
                            AvailableSpaceOf::Indefinite,
                            SizingOperation::MinContent,
                        ),
                    ))
                    .map(|axis| {
                        (axis.operation() == SizingOperation::Normal).then_some(axis.space())
                    }),
                ),
        )
        .measure_when(
            3,
            OracleMeasurement::new(ComputeOutput::from_outer_size(Size::new(72.0, 15.0)))
                .operations(
                    (Size::new(
                        AxisSizingRequestOf::new(
                            AvailableSpaceOf::Definite(
                                crate::NonNegativeFiniteOf::new(72.0).unwrap(),
                            ),
                            SizingOperation::Normal,
                        ),
                        AxisSizingRequestOf::new(
                            AvailableSpaceOf::Indefinite,
                            SizingOperation::MaxContent,
                        ),
                    ))
                    .map(|axis| Some(axis.operation())),
                )
                .spaces(
                    (Size::new(
                        AxisSizingRequestOf::new(
                            AvailableSpaceOf::Definite(
                                crate::NonNegativeFiniteOf::new(72.0).unwrap(),
                            ),
                            SizingOperation::Normal,
                        ),
                        AxisSizingRequestOf::new(
                            AvailableSpaceOf::Indefinite,
                            SizingOperation::MaxContent,
                        ),
                    ))
                    .map(|axis| {
                        (axis.operation() == SizingOperation::Normal).then_some(axis.space())
                    }),
                ),
        )
        .measure_when(
            2,
            OracleMeasurement::new(ComputeOutput::from_outer_size(Size::new(72.0, 15.0)))
                .operations(Size::new(Some(SizingOperation::MaxContent), None))
                .spaces(Size::new(None, None)),
        )
        .measure_when(
            2,
            OracleMeasurement::new(ComputeOutput::from_outer_size(Size::new(54.0, 30.0)))
                .operations(Size::new(Some(SizingOperation::MinContent), None))
                .spaces(Size::new(None, None)),
        )
        .measure_when(
            2,
            OracleMeasurement::new(ComputeOutput::from_outer_size(Size::new(54.0, 30.0)))
                .operations(
                    (Size::new(
                        AxisSizingRequestOf::new(
                            AvailableSpaceOf::Definite(
                                crate::NonNegativeFiniteOf::new(54.0).unwrap(),
                            ),
                            SizingOperation::Normal,
                        ),
                        AxisSizingRequestOf::new(
                            AvailableSpaceOf::Definite(
                                crate::NonNegativeFiniteOf::new(15.0).unwrap(),
                            ),
                            SizingOperation::Normal,
                        ),
                    ))
                    .map(|axis| Some(axis.operation())),
                )
                .spaces(
                    (Size::new(
                        AxisSizingRequestOf::new(
                            AvailableSpaceOf::Definite(
                                crate::NonNegativeFiniteOf::new(54.0).unwrap(),
                            ),
                            SizingOperation::Normal,
                        ),
                        AxisSizingRequestOf::new(
                            AvailableSpaceOf::Definite(
                                crate::NonNegativeFiniteOf::new(15.0).unwrap(),
                            ),
                            SizingOperation::Normal,
                        ),
                    ))
                    .map(|axis| {
                        (axis.operation() == SizingOperation::Normal).then_some(axis.space())
                    }),
                ),
        )
        .measure_when(
            2,
            OracleMeasurement::new(ComputeOutput::from_outer_size(Size::new(72.0, 15.0)))
                .known(Size::new(Some(72.0), None))
                .parent(Size::new(Some(72.0), Some(0.0)))
                .operations(
                    (Size::new(
                        AxisSizingRequestOf::new(
                            AvailableSpaceOf::Definite(
                                crate::NonNegativeFiniteOf::new(72.0).unwrap(),
                            ),
                            SizingOperation::Normal,
                        ),
                        AxisSizingRequestOf::new(
                            AvailableSpaceOf::Indefinite,
                            SizingOperation::MaxContent,
                        ),
                    ))
                    .map(|axis| Some(axis.operation())),
                )
                .spaces(
                    (Size::new(
                        AxisSizingRequestOf::new(
                            AvailableSpaceOf::Definite(
                                crate::NonNegativeFiniteOf::new(72.0).unwrap(),
                            ),
                            SizingOperation::Normal,
                        ),
                        AxisSizingRequestOf::new(
                            AvailableSpaceOf::Indefinite,
                            SizingOperation::MaxContent,
                        ),
                    ))
                    .map(|axis| {
                        (axis.operation() == SizingOperation::Normal).then_some(axis.space())
                    }),
                ),
        )
        .measure_when(
            2,
            OracleMeasurement::new(ComputeOutput::from_outer_size(Size::new(72.0, 15.0)))
                .parent(Size::new(Some(72.0), Some(0.0)))
                .operations(
                    (Size::new(
                        AxisSizingRequestOf::new(
                            AvailableSpaceOf::Definite(
                                crate::NonNegativeFiniteOf::new(72.0).unwrap(),
                            ),
                            SizingOperation::Normal,
                        ),
                        AxisSizingRequestOf::new(
                            AvailableSpaceOf::Indefinite,
                            SizingOperation::MaxContent,
                        ),
                    ))
                    .map(|axis| Some(axis.operation())),
                )
                .spaces(
                    (Size::new(
                        AxisSizingRequestOf::new(
                            AvailableSpaceOf::Definite(
                                crate::NonNegativeFiniteOf::new(72.0).unwrap(),
                            ),
                            SizingOperation::Normal,
                        ),
                        AxisSizingRequestOf::new(
                            AvailableSpaceOf::Indefinite,
                            SizingOperation::MaxContent,
                        ),
                    ))
                    .map(|axis| {
                        (axis.operation() == SizingOperation::Normal).then_some(axis.space())
                    }),
                ),
        )
        .measure_when(
            2,
            OracleMeasurement::new(ComputeOutput::from_outer_size(Size::new(54.0, 30.0)))
                .known(Size::new(None, Some(30.0)))
                .parent(Size::new(None, Some(30.0)))
                .operations(
                    (Size::new(
                        AxisSizingRequestOf::new(
                            AvailableSpaceOf::Indefinite,
                            SizingOperation::MinContent,
                        ),
                        AxisSizingRequestOf::new(
                            AvailableSpaceOf::Definite(
                                crate::NonNegativeFiniteOf::new(30.0).unwrap(),
                            ),
                            SizingOperation::Normal,
                        ),
                    ))
                    .map(|axis| Some(axis.operation())),
                )
                .spaces(
                    (Size::new(
                        AxisSizingRequestOf::new(
                            AvailableSpaceOf::Indefinite,
                            SizingOperation::MinContent,
                        ),
                        AxisSizingRequestOf::new(
                            AvailableSpaceOf::Definite(
                                crate::NonNegativeFiniteOf::new(30.0).unwrap(),
                            ),
                            SizingOperation::Normal,
                        ),
                    ))
                    .map(|axis| {
                        (axis.operation() == SizingOperation::Normal).then_some(axis.space())
                    }),
                ),
        )
        .measure_when(
            2,
            OracleMeasurement::new(ComputeOutput::from_outer_size(Size::new(54.0, 30.0)))
                .known(Size::new(None, Some(30.0)))
                .parent(Size::new(Some(72.0), Some(30.0)))
                .operations(
                    (Size::new(
                        AxisSizingRequestOf::new(
                            AvailableSpaceOf::Indefinite,
                            SizingOperation::MinContent,
                        ),
                        AxisSizingRequestOf::new(
                            AvailableSpaceOf::Definite(
                                crate::NonNegativeFiniteOf::new(30.0).unwrap(),
                            ),
                            SizingOperation::Normal,
                        ),
                    ))
                    .map(|axis| Some(axis.operation())),
                )
                .spaces(
                    (Size::new(
                        AxisSizingRequestOf::new(
                            AvailableSpaceOf::Indefinite,
                            SizingOperation::MinContent,
                        ),
                        AxisSizingRequestOf::new(
                            AvailableSpaceOf::Definite(
                                crate::NonNegativeFiniteOf::new(30.0).unwrap(),
                            ),
                            SizingOperation::Normal,
                        ),
                    ))
                    .map(|axis| {
                        (axis.operation() == SizingOperation::Normal).then_some(axis.space())
                    }),
                ),
        )
        .measure_when(
            2,
            OracleMeasurement::new(ComputeOutput::from_outer_size(Size::new(54.0, 30.0)))
                .run_mode(RunMode::PerformLayout)
                .known(Size::new(None, Some(30.0)))
                .parent(Size::new(Some(72.0), Some(30.0)))
                .operations(
                    (Size::new(
                        AxisSizingRequestOf::new(
                            AvailableSpaceOf::Definite(
                                crate::NonNegativeFiniteOf::new(72.0).unwrap(),
                            ),
                            SizingOperation::Normal,
                        ),
                        AxisSizingRequestOf::new(
                            AvailableSpaceOf::Definite(
                                crate::NonNegativeFiniteOf::new(30.0).unwrap(),
                            ),
                            SizingOperation::Normal,
                        ),
                    ))
                    .map(|axis| Some(axis.operation())),
                )
                .spaces(
                    (Size::new(
                        AxisSizingRequestOf::new(
                            AvailableSpaceOf::Definite(
                                crate::NonNegativeFiniteOf::new(72.0).unwrap(),
                            ),
                            SizingOperation::Normal,
                        ),
                        AxisSizingRequestOf::new(
                            AvailableSpaceOf::Definite(
                                crate::NonNegativeFiniteOf::new(30.0).unwrap(),
                            ),
                            SizingOperation::Normal,
                        ),
                    ))
                    .map(|axis| {
                        (axis.operation() == SizingOperation::Normal).then_some(axis.space())
                    }),
                ),
        )
        .measure_when(
            2,
            OracleMeasurement::new(ComputeOutput::from_outer_size(Size::new(54.0, 30.0)))
                .run_mode(RunMode::PerformLayout)
                .known(Size::new(None, Some(30.0)))
                .parent(Size::new(Some(54.0), Some(30.0)))
                .operations(
                    (Size::new(
                        AxisSizingRequestOf::new(
                            AvailableSpaceOf::Definite(
                                crate::NonNegativeFiniteOf::new(54.0).unwrap(),
                            ),
                            SizingOperation::Normal,
                        ),
                        AxisSizingRequestOf::new(
                            AvailableSpaceOf::Definite(
                                crate::NonNegativeFiniteOf::new(30.0).unwrap(),
                            ),
                            SizingOperation::Normal,
                        ),
                    ))
                    .map(|axis| Some(axis.operation())),
                )
                .spaces(
                    (Size::new(
                        AxisSizingRequestOf::new(
                            AvailableSpaceOf::Definite(
                                crate::NonNegativeFiniteOf::new(54.0).unwrap(),
                            ),
                            SizingOperation::Normal,
                        ),
                        AxisSizingRequestOf::new(
                            AvailableSpaceOf::Definite(
                                crate::NonNegativeFiniteOf::new(30.0).unwrap(),
                            ),
                            SizingOperation::Normal,
                        ),
                    ))
                    .map(|axis| {
                        (axis.operation() == SizingOperation::Normal).then_some(axis.space())
                    }),
                ),
        )
        .measure_when(
            2,
            OracleMeasurement::new(ComputeOutput::from_outer_size(Size::new(54.0, 30.0)))
                .run_mode(RunMode::PerformLayout)
                .known(Size::new(Some(54.0), Some(30.0)))
                .parent(Size::new(Some(54.0), Some(30.0)))
                .operations(
                    (Size::new(
                        AxisSizingRequestOf::new(
                            AvailableSpaceOf::Definite(
                                crate::NonNegativeFiniteOf::new(54.0).unwrap(),
                            ),
                            SizingOperation::Normal,
                        ),
                        AxisSizingRequestOf::new(
                            AvailableSpaceOf::Definite(
                                crate::NonNegativeFiniteOf::new(30.0).unwrap(),
                            ),
                            SizingOperation::Normal,
                        ),
                    ))
                    .map(|axis| Some(axis.operation())),
                )
                .spaces(
                    (Size::new(
                        AxisSizingRequestOf::new(
                            AvailableSpaceOf::Definite(
                                crate::NonNegativeFiniteOf::new(54.0).unwrap(),
                            ),
                            SizingOperation::Normal,
                        ),
                        AxisSizingRequestOf::new(
                            AvailableSpaceOf::Definite(
                                crate::NonNegativeFiniteOf::new(30.0).unwrap(),
                            ),
                            SizingOperation::Normal,
                        ),
                    ))
                    .map(|axis| {
                        (axis.operation() == SizingOperation::Normal).then_some(axis.space())
                    }),
                ),
        )
        .measure_when(
            3,
            OracleMeasurement::new(ComputeOutput::from_outer_size(Size::new(72.0, 15.0)))
                .operations(Size::new(Some(SizingOperation::MinContent), None))
                .spaces(Size::new(None, None)),
        )
        .measure_when(
            3,
            OracleMeasurement::new(ComputeOutput::from_outer_size(Size::new(72.0, 15.0)))
                .operations(Size::new(Some(SizingOperation::MaxContent), None))
                .spaces(Size::new(None, None)),
        )
        .measure_when(
            3,
            OracleMeasurement::new(ComputeOutput::from_outer_size(Size::new(72.0, 15.0)))
                .operations(
                    (Size::new(
                        AxisSizingRequestOf::new(
                            AvailableSpaceOf::Definite(
                                crate::NonNegativeFiniteOf::new(72.0).unwrap(),
                            ),
                            SizingOperation::Normal,
                        ),
                        AxisSizingRequestOf::new(
                            AvailableSpaceOf::Definite(
                                crate::NonNegativeFiniteOf::new(15.0).unwrap(),
                            ),
                            SizingOperation::Normal,
                        ),
                    ))
                    .map(|axis| Some(axis.operation())),
                )
                .spaces(
                    (Size::new(
                        AxisSizingRequestOf::new(
                            AvailableSpaceOf::Definite(
                                crate::NonNegativeFiniteOf::new(72.0).unwrap(),
                            ),
                            SizingOperation::Normal,
                        ),
                        AxisSizingRequestOf::new(
                            AvailableSpaceOf::Definite(
                                crate::NonNegativeFiniteOf::new(15.0).unwrap(),
                            ),
                            SizingOperation::Normal,
                        ),
                    ))
                    .map(|axis| {
                        (axis.operation() == SizingOperation::Normal).then_some(axis.space())
                    }),
                ),
        )
        .measure_when(
            3,
            OracleMeasurement::new(ComputeOutput::from_outer_size(Size::new(72.0, 15.0)))
                .known(Size::new(Some(72.0), None))
                .parent(Size::new(Some(72.0), Some(0.0)))
                .operations(
                    (Size::new(
                        AxisSizingRequestOf::new(
                            AvailableSpaceOf::Definite(
                                crate::NonNegativeFiniteOf::new(72.0).unwrap(),
                            ),
                            SizingOperation::Normal,
                        ),
                        AxisSizingRequestOf::new(
                            AvailableSpaceOf::Indefinite,
                            SizingOperation::MaxContent,
                        ),
                    ))
                    .map(|axis| Some(axis.operation())),
                )
                .spaces(
                    (Size::new(
                        AxisSizingRequestOf::new(
                            AvailableSpaceOf::Definite(
                                crate::NonNegativeFiniteOf::new(72.0).unwrap(),
                            ),
                            SizingOperation::Normal,
                        ),
                        AxisSizingRequestOf::new(
                            AvailableSpaceOf::Indefinite,
                            SizingOperation::MaxContent,
                        ),
                    ))
                    .map(|axis| {
                        (axis.operation() == SizingOperation::Normal).then_some(axis.space())
                    }),
                ),
        )
        .measure_when(
            3,
            OracleMeasurement::new(ComputeOutput::from_outer_size(Size::new(72.0, 15.0)))
                .parent(Size::new(Some(72.0), Some(0.0)))
                .operations(
                    (Size::new(
                        AxisSizingRequestOf::new(
                            AvailableSpaceOf::Definite(
                                crate::NonNegativeFiniteOf::new(72.0).unwrap(),
                            ),
                            SizingOperation::Normal,
                        ),
                        AxisSizingRequestOf::new(
                            AvailableSpaceOf::Indefinite,
                            SizingOperation::MaxContent,
                        ),
                    ))
                    .map(|axis| Some(axis.operation())),
                )
                .spaces(
                    (Size::new(
                        AxisSizingRequestOf::new(
                            AvailableSpaceOf::Definite(
                                crate::NonNegativeFiniteOf::new(72.0).unwrap(),
                            ),
                            SizingOperation::Normal,
                        ),
                        AxisSizingRequestOf::new(
                            AvailableSpaceOf::Indefinite,
                            SizingOperation::MaxContent,
                        ),
                    ))
                    .map(|axis| {
                        (axis.operation() == SizingOperation::Normal).then_some(axis.space())
                    }),
                ),
        )
        .measure_when(
            3,
            OracleMeasurement::new(ComputeOutput::from_outer_size(Size::new(72.0, 15.0)))
                .known(Size::new(None, Some(30.0)))
                .parent(Size::new(None, Some(30.0)))
                .operations(
                    (Size::new(
                        AxisSizingRequestOf::new(
                            AvailableSpaceOf::Indefinite,
                            SizingOperation::MaxContent,
                        ),
                        AxisSizingRequestOf::new(
                            AvailableSpaceOf::Definite(
                                crate::NonNegativeFiniteOf::new(30.0).unwrap(),
                            ),
                            SizingOperation::Normal,
                        ),
                    ))
                    .map(|axis| Some(axis.operation())),
                )
                .spaces(
                    (Size::new(
                        AxisSizingRequestOf::new(
                            AvailableSpaceOf::Indefinite,
                            SizingOperation::MaxContent,
                        ),
                        AxisSizingRequestOf::new(
                            AvailableSpaceOf::Definite(
                                crate::NonNegativeFiniteOf::new(30.0).unwrap(),
                            ),
                            SizingOperation::Normal,
                        ),
                    ))
                    .map(|axis| {
                        (axis.operation() == SizingOperation::Normal).then_some(axis.space())
                    }),
                ),
        )
        .measure_when(
            3,
            OracleMeasurement::new(ComputeOutput::from_outer_size(Size::new(72.0, 15.0)))
                .known(Size::new(None, Some(30.0)))
                .parent(Size::new(Some(72.0), Some(30.0)))
                .operations(
                    (Size::new(
                        AxisSizingRequestOf::new(
                            AvailableSpaceOf::Indefinite,
                            SizingOperation::MaxContent,
                        ),
                        AxisSizingRequestOf::new(
                            AvailableSpaceOf::Definite(
                                crate::NonNegativeFiniteOf::new(30.0).unwrap(),
                            ),
                            SizingOperation::Normal,
                        ),
                    ))
                    .map(|axis| Some(axis.operation())),
                )
                .spaces(
                    (Size::new(
                        AxisSizingRequestOf::new(
                            AvailableSpaceOf::Indefinite,
                            SizingOperation::MaxContent,
                        ),
                        AxisSizingRequestOf::new(
                            AvailableSpaceOf::Definite(
                                crate::NonNegativeFiniteOf::new(30.0).unwrap(),
                            ),
                            SizingOperation::Normal,
                        ),
                    ))
                    .map(|axis| {
                        (axis.operation() == SizingOperation::Normal).then_some(axis.space())
                    }),
                ),
        )
        .measure_when(
            3,
            OracleMeasurement::new(ComputeOutput::from_outer_size(Size::new(72.0, 15.0)))
                .run_mode(RunMode::PerformLayout)
                .known(Size::new(None, Some(30.0)))
                .parent(Size::new(Some(72.0), Some(30.0)))
                .operations(
                    (Size::new(
                        AxisSizingRequestOf::new(
                            AvailableSpaceOf::Definite(
                                crate::NonNegativeFiniteOf::new(72.0).unwrap(),
                            ),
                            SizingOperation::Normal,
                        ),
                        AxisSizingRequestOf::new(
                            AvailableSpaceOf::Definite(
                                crate::NonNegativeFiniteOf::new(30.0).unwrap(),
                            ),
                            SizingOperation::Normal,
                        ),
                    ))
                    .map(|axis| Some(axis.operation())),
                )
                .spaces(
                    (Size::new(
                        AxisSizingRequestOf::new(
                            AvailableSpaceOf::Definite(
                                crate::NonNegativeFiniteOf::new(72.0).unwrap(),
                            ),
                            SizingOperation::Normal,
                        ),
                        AxisSizingRequestOf::new(
                            AvailableSpaceOf::Definite(
                                crate::NonNegativeFiniteOf::new(30.0).unwrap(),
                            ),
                            SizingOperation::Normal,
                        ),
                    ))
                    .map(|axis| {
                        (axis.operation() == SizingOperation::Normal).then_some(axis.space())
                    }),
                ),
        )
        .measure_when(
            3,
            OracleMeasurement::new(ComputeOutput::from_outer_size(Size::new(72.0, 15.0)))
                .run_mode(RunMode::PerformLayout)
                .known(Size::new(Some(72.0), Some(30.0)))
                .parent(Size::new(Some(72.0), Some(30.0)))
                .operations(
                    (Size::new(
                        AxisSizingRequestOf::new(
                            AvailableSpaceOf::Definite(
                                crate::NonNegativeFiniteOf::new(72.0).unwrap(),
                            ),
                            SizingOperation::Normal,
                        ),
                        AxisSizingRequestOf::new(
                            AvailableSpaceOf::Definite(
                                crate::NonNegativeFiniteOf::new(30.0).unwrap(),
                            ),
                            SizingOperation::Normal,
                        ),
                    ))
                    .map(|axis| Some(axis.operation())),
                )
                .spaces(
                    (Size::new(
                        AxisSizingRequestOf::new(
                            AvailableSpaceOf::Definite(
                                crate::NonNegativeFiniteOf::new(72.0).unwrap(),
                            ),
                            SizingOperation::Normal,
                        ),
                        AxisSizingRequestOf::new(
                            AvailableSpaceOf::Definite(
                                crate::NonNegativeFiniteOf::new(30.0).unwrap(),
                            ),
                            SizingOperation::Normal,
                        ),
                    ))
                    .map(|axis| {
                        (axis.operation() == SizingOperation::Normal).then_some(axis.space())
                    }),
                ),
        );

    let output = crate::compute_grid(
        &mut tree.borrow_compute(),
        1,
        ComputeInput::for_child(
            RunMode::PerformLayout,
            SizingMode::InherentSize,
            RequestedAxis::Both,
            Size::NONE,
            Size::NONE,
            crate::ContainingLayoutContext::new(
                crate::geometry::FlowAxes::new(
                    crate::WritingMode::HorizontalTb,
                    crate::Direction::Ltr,
                ),
                crate::ParentFormattingContext::NoParent,
            ),
            SizingRequestOf::new(
                Size::new(
                    AxisSizingRequestOf::new(
                        AvailableSpaceOf::Indefinite,
                        SizingOperation::MaxContent,
                    ),
                    AxisSizingRequestOf::new(
                        AvailableSpaceOf::Indefinite,
                        SizingOperation::MaxContent,
                    ),
                ),
                OrthogonalFallbackOf::new(
                    Size::NONE,
                    crate::NearestScrollportOf::Absent,
                    Size::splat(crate::OrthogonalFallbackLimitOf::Unbounded),
                ),
            ),
        ),
    )
    .unwrap();

    assert_eq!(output.size, Size::new(72.0, 60.0));
    assert!(tree.inputs(2).iter().any(|input| {
        input.run_mode() == RunMode::ComputeSize
            && input.sizing_request().axes()
                == Size::new(
                    AxisSizingRequestOf::new(
                        AvailableSpaceOf::Indefinite,
                        SizingOperation::MinContent,
                    ),
                    AxisSizingRequestOf::new(
                        AvailableSpaceOf::Indefinite,
                        SizingOperation::MaxContent,
                    ),
                )
    }));
}

#[test]
fn named_grid_lanes_place_item_between_named_ordinary_grid_lines() {
    let oracle_lines = lts::oracle::grid::NamedGridLines::new(
        lts::oracle::grid::GridAxis::Column,
        3,
        vec![
            Vec::<&str>::new(),
            vec!["slot-start"],
            vec![],
            vec!["slot-end"],
        ],
    )
    .unwrap();
    let expected = lts::oracle::grid::resolve_named_axis_placement(
        &oracle_lines,
        lts::oracle::grid::NamedAxisPlacement {
            start: lts::oracle::grid::NamedGridLine::Named {
                name: "slot-start".to_string(),
                occurrence: 1,
            },
            end: lts::oracle::grid::NamedGridLine::Named {
                name: "slot-end".to_string(),
                occurrence: 1,
            },
        },
        None,
    )
    .unwrap()
    .resolved;
    let mut tree = OracleTree::new()
        .children(1, [2])
        .style(
            1,
            NodeInput {
                display: Display::GridLanes,
                size: Size::new(PreferredSize::px(120.0), PreferredSize::px(20.0)),
                grid_template_columns: vec![
                    TrackComponent::px(40.0),
                    TrackComponent::line_names(["slot-start"]),
                    TrackComponent::px(40.0),
                    TrackComponent::px(40.0),
                    TrackComponent::line_names(["slot-end"]),
                ],
                grid_template_rows: vec![TrackComponent::px(20.0)],
                grid_flow_tolerance: GridFlowTolerance::Length(Length::ZERO),
                ..NodeInput::DEFAULT
            },
        )
        .style(
            2,
            NodeInput {
                raw_grid_column: RawGridPlacement::new(
                    RawGridLine::NamedLine {
                        name: "slot-start".to_string(),
                        index: 1,
                    },
                    RawGridLine::NamedLine {
                        name: "slot-end".to_string(),
                        index: 1,
                    },
                ),
                ..NodeInput::DEFAULT
            },
        )
        .measure(2, ComputeOutput::from_outer_size(Size::new(80.0, 20.0)));

    compute_oracle_grid(&mut tree);
    let child = tree.final_layout(2).expect("lane child should be laid out");

    assert_eq!(
        child.location.x,
        (expected.start_line as Scalar - 1.0) * 40.0
    );
    assert_eq!(child.size.width, expected.span as Scalar * 40.0);
}

#[test]
fn named_grid_lanes_span_named_implicit_fallback_line() {
    let oracle_lines = lts::oracle::grid::NamedGridLines::new(
        lts::oracle::grid::GridAxis::Column,
        1,
        vec![vec!["a"], vec!["a"]],
    )
    .unwrap();
    let expected = lts::oracle::grid::resolve_named_axis_placement(
        &oracle_lines,
        lts::oracle::grid::NamedAxisPlacement {
            start: lts::oracle::grid::NamedGridLine::Named {
                name: "a".to_string(),
                occurrence: 2,
            },
            end: lts::oracle::grid::NamedGridLine::Span {
                name: Some("a".to_string()),
                count: 2,
            },
        },
        None,
    )
    .unwrap()
    .resolved;
    let mut tree = OracleTree::new()
        .children(1, [2])
        .style(
            1,
            NodeInput {
                display: Display::GridLanes,
                size: Size::new(PreferredSize::px(120.0), PreferredSize::px(20.0)),
                grid_template_columns: vec![
                    TrackComponent::line_names(["a"]),
                    TrackComponent::px(40.0),
                    TrackComponent::line_names(["a"]),
                ],
                grid_template_rows: vec![TrackComponent::px(20.0)],
                grid_auto_columns: vec![TrackComponent::px(40.0)],
                grid_flow_tolerance: GridFlowTolerance::Length(Length::ZERO),
                ..NodeInput::DEFAULT
            },
        )
        .style(
            2,
            NodeInput {
                raw_grid_column: RawGridPlacement::new(
                    RawGridLine::NamedLine {
                        name: "a".to_string(),
                        index: 2,
                    },
                    RawGridLine::NamedSpan {
                        name: "a".to_string(),
                        index: 2,
                    },
                ),
                ..NodeInput::DEFAULT
            },
        )
        .measure(2, ComputeOutput::from_outer_size(Size::new(80.0, 20.0)));

    compute_oracle_grid(&mut tree);
    let child = tree.final_layout(2).expect("lane child should be laid out");

    assert_eq!(
        child.location.x,
        (expected.start_line as Scalar - 1.0) * 40.0
    );
    assert_eq!(child.size.width, expected.span as Scalar * 40.0);
}

#[test]
fn named_grid_lanes_subgrid_axis_uses_inherited_line_names() {
    let parent_lines = lts::oracle::grid::NamedGridLines::new(
        lts::oracle::grid::GridAxis::Column,
        4,
        vec![
            vec!["a"],
            vec!["b"],
            Vec::<&str>::new(),
            vec!["c"],
            vec!["d"],
        ],
    )
    .unwrap();
    let subgrid = lts::oracle::grid::inherit_named_subgrid_lines(
        &parent_lines,
        lts::oracle::grid::TrackSpan::new(2, 5),
        false,
        vec![Vec::<String>::new(), Vec::new(), Vec::new(), Vec::new()],
        None,
    )
    .unwrap();
    let expected = lts::oracle::grid::resolve_named_subgrid_axis_placement(
        &subgrid.lines,
        lts::oracle::grid::NamedAxisPlacement {
            start: lts::oracle::grid::NamedGridLine::Named {
                name: "b".to_string(),
                occurrence: 1,
            },
            end: lts::oracle::grid::NamedGridLine::Named {
                name: "c".to_string(),
                occurrence: 1,
            },
        },
        None,
    )
    .unwrap()
    .clamped
    .resolved;
    let mut tree = OracleTree::new()
        .children(1, [2])
        .children(2, [3])
        .style(
            1,
            NodeInput {
                display: Display::Grid,
                size: Size::new(PreferredSize::px(160.0), PreferredSize::px(20.0)),
                grid_template_columns: vec![
                    TrackComponent::line_names(["a"]),
                    TrackComponent::px(40.0),
                    TrackComponent::line_names(["b"]),
                    TrackComponent::px(40.0),
                    TrackComponent::px(40.0),
                    TrackComponent::line_names(["c"]),
                    TrackComponent::px(40.0),
                    TrackComponent::line_names(["d"]),
                ],
                grid_template_rows: vec![TrackComponent::px(20.0)],
                ..NodeInput::DEFAULT
            },
        )
        .style(
            2,
            NodeInput {
                display: Display::GridLanes,
                grid_column: GridPlacement::try_lines(2, 5).expect("valid grid lines"),
                grid_row: GridPlacement::try_line(1).expect("valid grid line"),
                grid_template_columns: vec![empty_subgrid_track()],
                grid_template_rows: vec![TrackComponent::px(20.0)],
                grid_flow_tolerance: GridFlowTolerance::Length(Length::ZERO),
                ..NodeInput::DEFAULT
            },
        )
        .style(
            3,
            NodeInput {
                raw_grid_column: RawGridPlacement::new(
                    RawGridLine::NamedLine {
                        name: "b".to_string(),
                        index: 1,
                    },
                    RawGridLine::NamedLine {
                        name: "c".to_string(),
                        index: 1,
                    },
                ),
                raw_grid_row: RawGridPlacement::line(1),
                ..NodeInput::DEFAULT
            },
        )
        .measure(3, ComputeOutput::from_outer_size(Size::new(80.0, 20.0)));

    compute_oracle_grid(&mut tree);
    let child = tree
        .final_layout(3)
        .expect("subgridded lane child should be laid out");

    assert_eq!(
        child.location.x,
        (expected.start_line as Scalar - 1.0) * 40.0
    );
    assert_eq!(child.size.width, expected.span as Scalar * 40.0);
}

fn assert_logical_inherited_grid_axis_contexts<S: LayoutScalar>()
where
    OracleTreeOf<S>: Traverse<Node = u32, Scalar = S>,
{
    let scalar = S::from_f64;
    let parent_columns = [scalar(30.0), scalar(40.0)];
    let parent_rows = [scalar(50.0), scalar(60.0)];
    let parent_gap = LogicalSizeOf::new(scalar(7.0), scalar(11.0));
    let parent_area_size = LogicalSizeOf::new(scalar(77.0), scalar(121.0));
    let parent_named_columns = named::NamedGridLines::new(GridAxisKind::Column, 2);
    let parent_named_rows = named::NamedGridLines::new(GridAxisKind::Row, 2);

    for writing_mode in [
        WritingMode::HorizontalTb,
        WritingMode::VerticalRl,
        WritingMode::VerticalLr,
        WritingMode::SidewaysRl,
        WritingMode::SidewaysLr,
    ] {
        for direction in [Direction::Ltr, Direction::Rtl] {
            let parent_direction = match direction {
                Direction::Ltr => Direction::Rtl,
                Direction::Rtl => Direction::Ltr,
            };
            let parent_style = NodeInputOf {
                display: Display::Grid,
                writing_mode: WritingMode::HorizontalTb,
                direction: parent_direction,
                ..NodeInputOf::default()
            };
            let child_style = NodeInputOf {
                display: Display::Grid,
                writing_mode,
                direction,
                grid_template_columns: subgrid_track_of(),
                grid_template_rows: subgrid_track_of(),
                ..NodeInputOf::default()
            };
            let parent_flow_axes =
                crate::geometry::FlowAxes::new(parent_style.writing_mode, parent_style.direction);
            let child_flow_axes =
                crate::geometry::FlowAxes::new(child_style.writing_mode, child_style.direction);
            let parent_physical_size = parent_flow_axes.physical_size(parent_area_size);
            let item = SubgridItemReport {
                node: 1,
                column: subgrid_axis_report(&parent_style, &child_style, GridAxisKind::Column),
                row: subgrid_axis_report(&parent_style, &child_style, GridAxisKind::Row),
            };
            let column_mapping = item.column.mapping;
            let row_mapping = item.row.mapping;
            assert_eq!(column_mapping.queried_axis, GridAxisKind::Column);
            assert_eq!(column_mapping.child_axis, GridAxisKind::Column);
            assert_eq!(row_mapping.queried_axis, GridAxisKind::Row);
            assert_eq!(row_mapping.child_axis, GridAxisKind::Row);

            let mut context = subgrid_child_parent_context(SubgridChildParentContextInput {
                minimum_functions: GridMinimumParentFunctions {
                    columns: None,
                    rows: None,
                },
                item,
                child_style: &child_style,
                area: GridArea {
                    row: 0,
                    column: 0,
                    row_end: 2,
                    column_end: 2,
                    size: parent_area_size,
                },
                content_box_size: parent_physical_size,
                columns: &parent_columns,
                rows: &parent_rows,
                gap: parent_gap,
                parent_named_columns: &parent_named_columns,
                parent_named_rows: &parent_named_rows,
                parent_area_facts: None,
                margin: Edges::all(Some(S::ZERO)),
                border: Edges::ZERO,
                padding: Edges::ZERO,
            })
            .expect("subgrid inherited context must resolve");
            let columns = context
                .columns
                .as_mut()
                .expect("columns subgrid axis must inherit");
            columns.offset = scalar(13.0);
            let rows = context
                .rows
                .as_mut()
                .expect("rows subgrid axis must inherit");
            rows.offset = scalar(17.0);

            let columns = context
                .columns
                .as_ref()
                .expect("columns subgrid axis must remain inherited");
            let rows = context
                .rows
                .as_ref()
                .expect("rows subgrid axis must remain inherited");
            let inherited_logical_size = LogicalSizeOf::new(
                track_sum(&columns.tracks, columns.gap),
                track_sum(&rows.tracks, rows.gap),
            );
            assert_eq!(
                child_flow_axes.physical_size(inherited_logical_size),
                parent_physical_size,
                "{writing_mode:?} {direction:?} must retain parent physical extent through child logical axes"
            );
            assert_eq!(
                grid_axis_logical_offsets(
                    &columns.tracks,
                    Some(columns.offset),
                    S::ZERO,
                    GridAlignment {
                        start: S::ZERO,
                        gap: columns.gap,
                    },
                )[0],
                scalar(13.0),
                "{writing_mode:?} {direction:?} must retain the inherited inline offset logically"
            );
            assert_eq!(
                grid_axis_logical_offsets(
                    &rows.tracks,
                    Some(rows.offset),
                    S::ZERO,
                    GridAlignment {
                        start: S::ZERO,
                        gap: rows.gap,
                    },
                )[0],
                scalar(17.0),
                "{writing_mode:?} {direction:?} must retain the inherited block offset logically"
            );

            let mut tree = OracleTreeOf::<S>::new()
                .children(1, [])
                .style(1, child_style);
            let output = compute_grid_with_context(
                &mut tree.borrow_compute(),
                1,
                ComputeInputOf::for_child(
                    RunMode::PerformLayout,
                    SizingMode::InherentSize,
                    RequestedAxis::Both,
                    Size::NONE,
                    parent_physical_size.map(Some),
                    crate::ContainingLayoutContext::new(
                        parent_flow_axes,
                        crate::ParentFormattingContext::NoParent,
                    ),
                    SizingRequestOf::new(
                        parent_physical_size.map(|value| {
                            AxisSizingRequestOf::new(
                                AvailableSpaceOf::Definite(
                                    crate::NonNegativeFiniteOf::new(value).unwrap(),
                                ),
                                SizingOperation::Normal,
                            )
                        }),
                        OrthogonalFallbackOf::new(
                            Size::NONE,
                            crate::NearestScrollportOf::Absent,
                            Size::splat(crate::OrthogonalFallbackLimitOf::Unbounded),
                        ),
                    ),
                ),
                context,
            )
            .expect("inherited grid sizing must complete");

            assert_eq!(
                output.size, parent_physical_size,
                "{writing_mode:?} {direction:?} must project inherited track totals to physical output"
            );
            assert_eq!(
                output.content_size,
                parent_physical_size
                    + child_flow_axes.physical_size(LogicalSizeOf::new(scalar(13.0), scalar(17.0))),
                "{writing_mode:?} {direction:?} must include the inherited track offsets in overflow"
            );
        }
    }
}

#[test]
fn logical_inherited_grid_axis_contexts_f32() {
    assert_logical_inherited_grid_axis_contexts::<f32>();
}

#[test]
fn logical_inherited_grid_axis_contexts_f64() {
    assert_logical_inherited_grid_axis_contexts::<f64>();
}

#[test]
fn subgrid_template_resolves_to_empty_explicit_tracks_and_grows_implicit_tracks() {
    let mut tree = OracleTree::new();
    tree.insert_children(1, vec![2, 3, 4, 5, 6]);
    for child in 2..=6 {
        tree.insert_children(child, vec![]);
    }
    tree.insert_style(
        1,
        NodeInput {
            display: Display::Grid,
            grid_template_columns: vec![
                TrackComponent::Subgrid(crate::SubgridTrack {
                    name_components: vec![crate::SubgridLineNameComponent::LineNames(vec![
                        "main".to_string(),
                    ])],
                }),
                TrackComponent::Repeat(
                    crate::TrackRepetition::auto_fit(vec![crate::TrackSizing::px(10.0)])
                        .expect("valid track repetition"),
                ),
            ],
            grid_template_rows: vec![TrackComponent::px(10.0)],
            grid_auto_columns: vec![TrackComponent::px(10.0)],
            grid_auto_rows: vec![TrackComponent::px(10.0)],
            ..NodeInput::default()
        },
    );
    for child in 2..=6 {
        tree.insert_style(child, NodeInput::default());
        tree.insert_measure(child, ComputeOutput::from_outer_size(Size::new(10.0, 10.0)));
    }

    let output = crate::compute_grid(
        &mut tree.borrow_compute(),
        1,
        ComputeInput::for_child(
            RunMode::PerformLayout,
            SizingMode::InherentSize,
            RequestedAxis::Both,
            Size::NONE,
            Size::NONE,
            crate::ContainingLayoutContext::new(
                crate::geometry::FlowAxes::new(
                    crate::WritingMode::HorizontalTb,
                    crate::Direction::Ltr,
                ),
                crate::ParentFormattingContext::NoParent,
            ),
            SizingRequestOf::new(
                Size::new(
                    AxisSizingRequestOf::new(
                        AvailableSpaceOf::Indefinite,
                        SizingOperation::MaxContent,
                    ),
                    AxisSizingRequestOf::new(
                        AvailableSpaceOf::Indefinite,
                        SizingOperation::MaxContent,
                    ),
                ),
                OrthogonalFallbackOf::new(
                    Size::NONE,
                    crate::NearestScrollportOf::Absent,
                    Size::splat(crate::OrthogonalFallbackLimitOf::Unbounded),
                ),
            ),
        ),
    )
    .unwrap();

    assert_eq!(output.content_size, Size::new(10.0, 50.0));
    assert_eq!(
        tree.layout(6).expect("node layout is staged").location.y,
        40.0
    );
}

#[test]
fn row_subgrid_intrinsic_width_uses_inherited_rows_for_column_auto_flow() {
    let mut tree = OracleTree::new()
        .children(1, [2])
        .children(2, [3, 4])
        .children(3, [])
        .children(4, [])
        .style(
            1,
            NodeInput {
                display: Display::InlineGrid,
                ..NodeInput::DEFAULT
            },
        )
        .style(
            2,
            NodeInput {
                display: Display::Grid,
                size: Size::new(PreferredSize::MIN_CONTENT, PreferredSize::AUTO),
                grid_auto_flow: GridAutoFlow::Column,
                grid_template_rows: vec![empty_subgrid_track()],
                grid_row: GridPlacement::try_span(2).expect("valid grid span"),
                ..NodeInput::DEFAULT
            },
        )
        .style(
            3,
            NodeInput {
                size: Size::new(PreferredSize::px(100.0), PreferredSize::px(50.0)),
                ..NodeInput::DEFAULT
            },
        )
        .style(
            4,
            NodeInput {
                size: Size::new(PreferredSize::px(100.0), PreferredSize::px(50.0)),
                ..NodeInput::DEFAULT
            },
        );

    let output = crate::compute_grid(
        &mut tree.borrow_compute(),
        1,
        ComputeInput::for_child(
            RunMode::PerformLayout,
            SizingMode::InherentSize,
            RequestedAxis::Both,
            Size::NONE,
            Size::NONE,
            crate::ContainingLayoutContext::new(
                crate::geometry::FlowAxes::new(
                    crate::WritingMode::HorizontalTb,
                    crate::Direction::Ltr,
                ),
                crate::ParentFormattingContext::NoParent,
            ),
            SizingRequestOf::new(
                Size::new(
                    AxisSizingRequestOf::new(
                        AvailableSpaceOf::Indefinite,
                        SizingOperation::MaxContent,
                    ),
                    AxisSizingRequestOf::new(
                        AvailableSpaceOf::Indefinite,
                        SizingOperation::MaxContent,
                    ),
                ),
                OrthogonalFallbackOf::new(
                    Size::NONE,
                    crate::NearestScrollportOf::Absent,
                    Size::splat(crate::OrthogonalFallbackLimitOf::Unbounded),
                ),
            ),
        ),
    )
    .unwrap();

    assert_eq!(output.content_size, Size::new(100.0, 100.0));
    assert_eq!(tree.layout(2).unwrap().size, Size::new(100.0, 100.0));
    assert_eq!(tree.layout(3).unwrap().location, Point::new(0.0, 0.0));
    assert_eq!(tree.layout(4).unwrap().location, Point::new(0.0, 50.0));
}

#[test]
fn row_subgrid_constrained_sizing_keeps_fixed_descendants_when_sibling_uses_percent() {
    let mut tree = OracleTree::new()
        .children(1, [2])
        .children(2, [3, 4])
        .children(3, [])
        .children(4, [])
        .style(
            1,
            NodeInput {
                display: Display::Grid,
                grid_template_columns: vec![TrackComponent::px(100.0)],
                grid_template_rows: vec![TrackComponent::AUTO],
                ..NodeInput::DEFAULT
            },
        )
        .style(
            2,
            NodeInput {
                display: Display::Grid,
                grid_template_columns: vec![TrackComponent::px(100.0)],
                grid_template_rows: vec![empty_subgrid_track()],
                grid_row: GridPlacement::try_lines(1, -1).expect("valid grid lines"),
                ..NodeInput::DEFAULT
            },
        )
        .style(
            3,
            NodeInput {
                size: Size::new(PreferredSize::px(100.0), PreferredSize::px(30.0)),
                grid_row: GridPlacement::try_line(1).expect("valid grid line"),
                ..NodeInput::DEFAULT
            },
        )
        .style(
            4,
            NodeInput {
                size: Size::new(PreferredSize::px(100.0), PreferredSize::percent(0.5)),
                grid_row: GridPlacement::try_line(1).expect("valid grid line"),
                ..NodeInput::DEFAULT
            },
        );

    let output = crate::compute_grid(
        &mut tree.borrow_compute(),
        1,
        ComputeInput::for_child(
            RunMode::PerformLayout,
            SizingMode::InherentSize,
            RequestedAxis::Both,
            Size::NONE,
            Size::NONE,
            crate::ContainingLayoutContext::new(
                crate::geometry::FlowAxes::new(
                    crate::WritingMode::HorizontalTb,
                    crate::Direction::Ltr,
                ),
                crate::ParentFormattingContext::NoParent,
            ),
            SizingRequestOf::new(
                Size::new(
                    AxisSizingRequestOf::new(
                        AvailableSpaceOf::Indefinite,
                        SizingOperation::MaxContent,
                    ),
                    AxisSizingRequestOf::new(
                        AvailableSpaceOf::Indefinite,
                        SizingOperation::MaxContent,
                    ),
                ),
                OrthogonalFallbackOf::new(
                    Size::NONE,
                    crate::NearestScrollportOf::Absent,
                    Size::splat(crate::OrthogonalFallbackLimitOf::Unbounded),
                ),
            ),
        ),
    )
    .unwrap();

    assert_eq!(output.size, Size::new(100.0, 30.0));
    assert_eq!(tree.layout(2).unwrap().size, Size::new(100.0, 30.0));
    assert_eq!(tree.layout(3).unwrap().size, Size::new(100.0, 30.0));
    let percent_item = tree.layout(4).unwrap();
    assert_eq!(percent_item.location.x, 100.0);
    assert_eq!(percent_item.size.width, 100.0);
    // The second item creates a standalone implicit column. Its used track
    // extends the overflow of the 100px-wide ancestor.
    assert_eq!(output.content_size, Size::new(200.0, 30.0));
}

fn row_subgrid_auto_track_sizing_tree(
    columns: Vec<TrackComponent>,
    subgrid_column: GridPlacement,
) -> OracleTree {
    OracleTree::new()
        .children(1, [2])
        .children(2, [3])
        .children(3, [])
        .style(
            1,
            NodeInput {
                display: Display::InlineGrid,
                grid_template_columns: columns,
                grid_template_rows: vec![TrackComponent::AUTO],
                ..NodeInput::DEFAULT
            },
        )
        .style(
            2,
            NodeInput {
                display: Display::Grid,
                grid_template_columns: vec![TrackComponent::AUTO],
                grid_template_rows: vec![empty_subgrid_track()],
                grid_column: subgrid_column,
                margin: Edges {
                    top: LengthAuto::px(7.0),
                    right: LengthAuto::px(11.0),
                    bottom: LengthAuto::px(3.0),
                    left: LengthAuto::px(5.0),
                },
                padding: Edges {
                    top: Length::px(3.0),
                    right: Length::px(5.0),
                    bottom: Length::px(7.0),
                    left: Length::px(11.0),
                },
                border: Edges {
                    top: Length::px(5.0),
                    right: Length::px(7.0),
                    bottom: Length::px(11.0),
                    left: Length::px(3.0),
                },
                ..NodeInput::DEFAULT
            },
        )
        .style(3, NodeInput::default())
        .measure_when(
            3,
            OracleMeasurement::new(ComputeOutput::from_outer_size(Size::new(58.0, 84.0)))
                .run_mode(RunMode::ComputeSize)
                .operations(Size::new(
                    Some(
                        (AxisSizingRequestOf::new(
                            AvailableSpaceOf::Definite(
                                crate::NonNegativeFiniteOf::new(58.0).unwrap(),
                            ),
                            SizingOperation::Normal,
                        ))
                        .operation(),
                    ),
                    None,
                ))
                .spaces(Size::new(
                    ((AxisSizingRequestOf::new(
                        AvailableSpaceOf::Definite(crate::NonNegativeFiniteOf::new(58.0).unwrap()),
                        SizingOperation::Normal,
                    ))
                    .operation()
                        == SizingOperation::Normal)
                        .then_some(
                            (AxisSizingRequestOf::new(
                                AvailableSpaceOf::Definite(
                                    crate::NonNegativeFiniteOf::new(58.0).unwrap(),
                                ),
                                SizingOperation::Normal,
                            ))
                            .space(),
                        ),
                    None,
                )),
        )
        .measure_when(
            3,
            OracleMeasurement::new(ComputeOutput::from_outer_size(Size::new(58.0, 84.0)))
                .run_mode(RunMode::PerformLayout)
                .operations(
                    (Size::new(
                        AxisSizingRequestOf::new(
                            AvailableSpaceOf::Definite(
                                crate::NonNegativeFiniteOf::new(58.0).unwrap(),
                            ),
                            SizingOperation::Normal,
                        ),
                        AxisSizingRequestOf::new(
                            AvailableSpaceOf::Definite(
                                crate::NonNegativeFiniteOf::new(84.0).unwrap(),
                            ),
                            SizingOperation::Normal,
                        ),
                    ))
                    .map(|axis| Some(axis.operation())),
                )
                .spaces(
                    (Size::new(
                        AxisSizingRequestOf::new(
                            AvailableSpaceOf::Definite(
                                crate::NonNegativeFiniteOf::new(58.0).unwrap(),
                            ),
                            SizingOperation::Normal,
                        ),
                        AxisSizingRequestOf::new(
                            AvailableSpaceOf::Definite(
                                crate::NonNegativeFiniteOf::new(84.0).unwrap(),
                            ),
                            SizingOperation::Normal,
                        ),
                    ))
                    .map(|axis| {
                        (axis.operation() == SizingOperation::Normal).then_some(axis.space())
                    }),
                ),
        )
        .measure(3, ComputeOutput::from_outer_size(Size::new(58.0, 116.0)))
}

#[test]
fn row_subgrid_auto_track_sizing_fixed_then_auto_uses_descendant_contribution_once() {
    let mut tree = row_subgrid_auto_track_sizing_tree(
        vec![TrackComponent::px(100.0), TrackComponent::AUTO],
        GridPlacement::try_line(1).expect("valid grid line"),
    );

    let output = crate::compute_grid(
        &mut tree.borrow_compute(),
        1,
        ComputeInput::for_child(
            RunMode::PerformLayout,
            SizingMode::InherentSize,
            RequestedAxis::Both,
            Size::NONE,
            Size::NONE,
            crate::ContainingLayoutContext::new(
                crate::geometry::FlowAxes::new(
                    crate::WritingMode::HorizontalTb,
                    crate::Direction::Ltr,
                ),
                crate::ParentFormattingContext::NoParent,
            ),
            SizingRequestOf::new(
                Size::new(
                    AxisSizingRequestOf::new(
                        AvailableSpaceOf::Indefinite,
                        SizingOperation::MaxContent,
                    ),
                    AxisSizingRequestOf::new(
                        AvailableSpaceOf::Indefinite,
                        SizingOperation::MaxContent,
                    ),
                ),
                OrthogonalFallbackOf::new(
                    Size::NONE,
                    crate::NearestScrollportOf::Absent,
                    Size::splat(crate::OrthogonalFallbackLimitOf::Unbounded),
                ),
            ),
        ),
    )
    .unwrap();

    assert_eq!(output.size.height, 120.0);
    assert_eq!(tree.layout(2).unwrap().size.height, 110.0);
}

#[test]
fn row_subgrid_auto_track_sizing_auto_then_fixed_uses_descendant_contribution_once() {
    let mut tree = row_subgrid_auto_track_sizing_tree(
        vec![TrackComponent::AUTO, TrackComponent::px(100.0)],
        GridPlacement::try_line(2).expect("valid grid line"),
    );

    let output = crate::compute_grid(
        &mut tree.borrow_compute(),
        1,
        ComputeInput::for_child(
            RunMode::PerformLayout,
            SizingMode::InherentSize,
            RequestedAxis::Both,
            Size::NONE,
            Size::NONE,
            crate::ContainingLayoutContext::new(
                crate::geometry::FlowAxes::new(
                    crate::WritingMode::HorizontalTb,
                    crate::Direction::Ltr,
                ),
                crate::ParentFormattingContext::NoParent,
            ),
            SizingRequestOf::new(
                Size::new(
                    AxisSizingRequestOf::new(
                        AvailableSpaceOf::Indefinite,
                        SizingOperation::MaxContent,
                    ),
                    AxisSizingRequestOf::new(
                        AvailableSpaceOf::Indefinite,
                        SizingOperation::MaxContent,
                    ),
                ),
                OrthogonalFallbackOf::new(
                    Size::NONE,
                    crate::NearestScrollportOf::Absent,
                    Size::splat(crate::OrthogonalFallbackLimitOf::Unbounded),
                ),
            ),
        ),
    )
    .unwrap();

    assert_eq!(output.size.height, 120.0);
    assert_eq!(tree.layout(2).unwrap().size.height, 110.0);
}

#[test]
fn row_subgrid_intrinsic_width_accumulates_standalone_percent_columns() {
    let mut tree = OracleTree::new()
        .children(1, [2])
        .children(2, [3, 4])
        .children(3, [5])
        .children(4, [6])
        .children(5, [])
        .children(6, [])
        .style(
            1,
            NodeInput {
                display: Display::Grid,
                grid_template_rows: vec![TrackComponent::px(100.0)],
                ..NodeInput::DEFAULT
            },
        )
        .style(
            2,
            NodeInput {
                display: Display::Grid,
                size: Size::new(PreferredSize::MIN_CONTENT, PreferredSize::AUTO),
                grid_template_columns: vec![
                    TrackComponent::percent(0.2),
                    TrackComponent::percent(0.3),
                ],
                grid_template_rows: vec![empty_subgrid_track()],
                ..NodeInput::DEFAULT
            },
        )
        .style(
            5,
            NodeInput {
                size: Size::new(PreferredSize::px(100.0), PreferredSize::AUTO),
                ..NodeInput::DEFAULT
            },
        )
        .style(
            6,
            NodeInput {
                size: Size::new(PreferredSize::px(100.0), PreferredSize::AUTO),
                ..NodeInput::DEFAULT
            },
        )
        .style(3, NodeInput::default())
        .style(4, NodeInput::default());

    let output = crate::compute_grid(
        &mut tree.borrow_compute(),
        1,
        ComputeInput::for_child(
            RunMode::PerformLayout,
            SizingMode::InherentSize,
            RequestedAxis::Both,
            Size::NONE,
            Size::NONE,
            crate::ContainingLayoutContext::new(
                crate::geometry::FlowAxes::new(
                    crate::WritingMode::HorizontalTb,
                    crate::Direction::Ltr,
                ),
                crate::ParentFormattingContext::NoParent,
            ),
            SizingRequestOf::new(
                Size::new(
                    AxisSizingRequestOf::new(
                        AvailableSpaceOf::Indefinite,
                        SizingOperation::MaxContent,
                    ),
                    AxisSizingRequestOf::new(
                        AvailableSpaceOf::Indefinite,
                        SizingOperation::MaxContent,
                    ),
                ),
                OrthogonalFallbackOf::new(
                    Size::NONE,
                    crate::NearestScrollportOf::Absent,
                    Size::splat(crate::OrthogonalFallbackLimitOf::Unbounded),
                ),
            ),
        ),
    )
    .unwrap();

    assert_eq!(output.content_size.width, 200.0);
    // The two standalone columns contribute 100 each before their percentages
    // resolve to 40 and 60 against the established 200px min-content width.
    assert_eq!(tree.layout(2).unwrap().size.width, 200.0);
}

#[test]
fn vertical_intrinsic_subgrid_final_sizing_keeps_definite_physical_height() {
    let mut tree = OracleTree::new()
        .children(1, [2, 3])
        .children(2, [])
        .children(3, [])
        .style(
            1,
            NodeInput {
                display: Display::Grid,
                writing_mode: WritingMode::VerticalRl,
                size: Size::new(PreferredSize::AUTO, PreferredSize::px(100.0)),
                grid_template_columns: vec![
                    TrackComponent::percent(0.2),
                    TrackComponent::percent(0.3),
                ],
                grid_template_rows: vec![empty_subgrid_track()],
                ..NodeInput::DEFAULT
            },
        )
        .style(2, NodeInput::default())
        .style(3, NodeInput::default())
        .measure(2, ComputeOutput::from_outer_size(Size::new(100.0, 10.0)))
        .measure(3, ComputeOutput::from_outer_size(Size::new(100.0, 10.0)));

    // The ancestor declares Auto independently of its allocated 100px row.
    let parent_row_functions = [TrackSizing::AUTO];
    let parent_context = GridParentContext {
        columns: None,
        rows: Some(InheritedGridAxis {
            baseline_reference: crate::GridBaselineReference::Physical,
            offset: 0.0,
            gap: 0.0,
            tracks: vec![100.0],
            minimum_functions: Some(
                GridMinimumTrackFunctions::new(&parent_row_functions, None)
                    .retain()
                    .unwrap(),
            ),
            geometry: UsedGridAxisGeometryOf::new(vec![100.0], vec![false], 0.0),
            named_lines: named::NamedGridLines::new(GridAxisKind::Row, 1),
            area_facts: None,
            template_area_expanded: false,
            owner_baseline_targets: None,
            parent_start: 0,
            parent_end: 1,
            reversed: false,
        }),
    };

    let output = compute_grid_with_context(
        &mut tree.borrow_compute(),
        1,
        ComputeInput::for_child(
            RunMode::PerformLayout,
            SizingMode::InherentSize,
            RequestedAxis::Both,
            Size::NONE,
            Size::new(None, Some(100.0)),
            crate::ContainingLayoutContext::new(
                crate::geometry::FlowAxes::new(
                    crate::WritingMode::HorizontalTb,
                    crate::Direction::Ltr,
                ),
                crate::ParentFormattingContext::NoParent,
            ),
            SizingRequestOf::new(
                Size::new(
                    AxisSizingRequestOf::new(
                        AvailableSpaceOf::Indefinite,
                        SizingOperation::MaxContent,
                    ),
                    AxisSizingRequestOf::new(
                        AvailableSpaceOf::Definite(crate::NonNegativeFiniteOf::new(100.0).unwrap()),
                        SizingOperation::Normal,
                    ),
                ),
                OrthogonalFallbackOf::new(
                    Size::NONE,
                    crate::NearestScrollportOf::Absent,
                    Size::splat(crate::OrthogonalFallbackLimitOf::Unbounded),
                ),
            ),
        ),
        parent_context,
    )
    .unwrap();

    assert_eq!(output.size, Size::new(100.0, 100.0));
}

#[test]
fn subgrid_line_names_merge_local_names_at_corresponding_lines() {
    let mut tree = OracleTree::new()
        .children(1, [2])
        .children(2, [3])
        .style(
            1,
            NodeInput {
                display: Display::Grid,
                size: Size::new(PreferredSize::px(120.0), PreferredSize::px(20.0)),
                grid_template_columns: vec![
                    TrackComponent::px(40.0),
                    TrackComponent::px(40.0),
                    TrackComponent::px(40.0),
                ],
                grid_template_rows: vec![TrackComponent::px(20.0)],
                ..NodeInput::DEFAULT
            },
        )
        .style(
            2,
            NodeInput {
                display: Display::Grid,
                grid_column: GridPlacement::try_lines(1, 4).expect("valid grid lines"),
                grid_row: GridPlacement::try_line(1).expect("valid grid line"),
                grid_template_columns: vec![TrackComponent::Subgrid(crate::SubgridTrack::new(
                    vec![
                        vec!["local-start".to_string()],
                        vec![],
                        vec!["middle".to_string()],
                        vec!["local-end".to_string()],
                    ],
                ))],
                grid_template_rows: vec![TrackComponent::px(20.0)],
                ..NodeInput::DEFAULT
            },
        )
        .style(
            3,
            NodeInput {
                raw_grid_column: RawGridPlacement::new(
                    RawGridLine::NamedLine {
                        name: "local-start".to_string(),
                        index: 1,
                    },
                    RawGridLine::NamedLine {
                        name: "middle".to_string(),
                        index: 1,
                    },
                ),
                raw_grid_row: RawGridPlacement::line(1),
                ..NodeInput::DEFAULT
            },
        );

    compute_oracle_grid(&mut tree);
    let child = tree
        .final_layout(3)
        .expect("local-name child should be laid out");

    assert_eq!(child.location.x, 0.0);
    assert_eq!(child.size.width, 80.0);
}

#[test]
fn subgrid_line_names_clip_parent_area_generated_names_to_span() {
    let mut tree = OracleTree::new()
        .children(1, [2])
        .children(2, [3])
        .style(
            1,
            NodeInput {
                display: Display::Grid,
                size: Size::new(PreferredSize::px(160.0), PreferredSize::px(20.0)),
                grid_template_columns: vec![
                    TrackComponent::px(40.0),
                    TrackComponent::px(40.0),
                    TrackComponent::px(40.0),
                    TrackComponent::px(40.0),
                ],
                grid_template_rows: vec![TrackComponent::px(20.0)],
                grid_template_areas: GridTemplateAreas {
                    rows: vec![GridTemplateAreaRow {
                        cells: vec![
                            None,
                            Some("main".to_string()),
                            Some("main".to_string()),
                            None,
                        ],
                    }],
                },
                ..NodeInput::DEFAULT
            },
        )
        .style(
            2,
            NodeInput {
                display: Display::Grid,
                grid_column: GridPlacement::try_lines(2, 4).expect("valid grid lines"),
                grid_row: GridPlacement::try_line(1).expect("valid grid line"),
                grid_template_columns: vec![empty_subgrid_track()],
                grid_template_rows: vec![TrackComponent::px(20.0)],
                ..NodeInput::DEFAULT
            },
        )
        .style(
            3,
            NodeInput {
                raw_grid_column: RawGridPlacement::new(
                    RawGridLine::BareIdent("main".to_string()),
                    RawGridLine::BareIdent("main".to_string()),
                ),
                raw_grid_row: RawGridPlacement::line(1),
                ..NodeInput::DEFAULT
            },
        );

    compute_oracle_grid(&mut tree);
    let child = tree
        .final_layout(3)
        .expect("area-name child should be laid out");

    assert_eq!(child.location.x, 0.0);
    assert_eq!(child.size.width, 80.0);
}

#[test]
fn subgrid_line_names_nested_subgrid_inherits_area_generated_names() {
    let mut tree = OracleTree::new()
        .children(1, [2])
        .children(2, [3])
        .children(3, [4, 5, 6, 7])
        .style(
            1,
            NodeInput {
                display: Display::Grid,
                size: Size::new(PreferredSize::px(160.0), PreferredSize::px(20.0)),
                grid_template_columns: vec![
                    TrackComponent::px(40.0),
                    TrackComponent::px(40.0),
                    TrackComponent::px(40.0),
                    TrackComponent::px(40.0),
                ],
                grid_template_rows: vec![TrackComponent::px(20.0)],
                grid_template_areas: GridTemplateAreas {
                    rows: vec![GridTemplateAreaRow {
                        cells: vec![
                            None,
                            Some("main".to_string()),
                            Some("main".to_string()),
                            None,
                        ],
                    }],
                },
                ..NodeInput::DEFAULT
            },
        )
        .style(
            2,
            NodeInput {
                display: Display::Grid,
                grid_column: GridPlacement::try_lines(2, 4).expect("valid grid lines"),
                grid_row: GridPlacement::try_line(1).expect("valid grid line"),
                grid_template_columns: vec![empty_subgrid_track()],
                grid_template_rows: vec![TrackComponent::px(20.0)],
                ..NodeInput::DEFAULT
            },
        )
        .style(
            3,
            NodeInput {
                display: Display::Grid,
                grid_column: GridPlacement::try_lines(1, 3).expect("valid grid lines"),
                grid_row: GridPlacement::try_line(1).expect("valid grid line"),
                grid_template_columns: vec![empty_subgrid_track()],
                grid_template_rows: vec![TrackComponent::px(20.0)],
                ..NodeInput::DEFAULT
            },
        )
        .style(
            4,
            NodeInput {
                raw_grid_column: RawGridPlacement::new(
                    RawGridLine::BareIdent("main".to_string()),
                    RawGridLine::BareIdent("main".to_string()),
                ),
                raw_grid_row: RawGridPlacement::line(1),
                ..NodeInput::DEFAULT
            },
        )
        .style(5, NodeInput::default())
        .style(6, NodeInput::default())
        .style(7, NodeInput::default());

    compute_oracle_grid(&mut tree);
    let child = tree
        .final_layout(4)
        .expect("nested area-name child should be laid out");

    assert_eq!(child.location.x, 0.0);
    assert_eq!(child.size.width, 80.0);
}

#[test]
fn subgrid_line_names_named_placement_beyond_span_clamps_to_edge_track() {
    let mut tree = OracleTree::new()
        .children(1, [2])
        .children(2, [3])
        .style(
            1,
            NodeInput {
                display: Display::Grid,
                size: Size::new(PreferredSize::px(40.0), PreferredSize::px(20.0)),
                grid_template_columns: vec![TrackComponent::px(40.0)],
                grid_template_rows: vec![TrackComponent::px(20.0)],
                ..NodeInput::DEFAULT
            },
        )
        .style(
            2,
            NodeInput {
                display: Display::Grid,
                grid_column: GridPlacement::try_lines(1, 2).expect("valid grid lines"),
                grid_row: GridPlacement::try_line(1).expect("valid grid line"),
                grid_template_columns: vec![empty_subgrid_track()],
                grid_template_rows: vec![TrackComponent::px(20.0)],
                ..NodeInput::DEFAULT
            },
        )
        .style(
            3,
            NodeInput {
                raw_grid_column: RawGridPlacement::new(RawGridLine::Line(2), RawGridLine::Span(3)),
                raw_grid_row: RawGridPlacement::line(1),
                ..NodeInput::DEFAULT
            },
        );

    compute_oracle_grid(&mut tree);
    let child = tree
        .final_layout(3)
        .expect("clamped child should be laid out");

    assert_eq!(child.location.x, 0.0);
    assert_eq!(child.size.width, 40.0);
}

fn assert_subgrid_clips_extreme_raw_placement<S: LayoutScalar + std::panic::RefUnwindSafe>() {
    let px = S::from_f64;
    let container = Size::new(px(60.0), px(20.0));
    let named_span = || RawGridLine::NamedSpan {
        name: "absent".into(),
        index: usize::MAX,
    };
    let named_line = |index| RawGridLine::NamedLine {
        name: "absent".into(),
        index,
    };
    let mut differences = Vec::new();
    for (placement, left, width) in [
        (RawGridPlacement::lines(2, 4), 10.0, 50.0),
        (
            RawGridPlacement::new(RawGridLine::Line(2), RawGridLine::Span(usize::MAX)),
            10.0,
            50.0,
        ),
        (
            RawGridPlacement::new(RawGridLine::Span(usize::MAX), RawGridLine::Line(3)),
            0.0,
            30.0,
        ),
        (
            RawGridPlacement::new(RawGridLine::Line(2), named_span()),
            10.0,
            50.0,
        ),
        (
            RawGridPlacement::new(named_span(), RawGridLine::Line(3)),
            0.0,
            30.0,
        ),
        (RawGridPlacement::line(isize::MAX), 30.0, 30.0),
        (RawGridPlacement::line(isize::MIN), 0.0, 10.0),
        (
            RawGridPlacement::new(named_line(isize::MAX), RawGridLine::Auto),
            30.0,
            30.0,
        ),
        (
            RawGridPlacement::new(RawGridLine::Auto, named_line(isize::MIN)),
            0.0,
            10.0,
        ),
    ] {
        let actual = std::panic::catch_unwind(|| {
            let tree = PublicLayoutTreeOf::new()
                .children(1, [2])
                .children(2, [3])
                .children(3, [])
                .style(
                    1,
                    NodeInputOf {
                        display: Display::Grid,
                        size: container.map(PreferredSizeOf::px),
                        grid_template_columns: [10.0, 20.0, 30.0]
                            .map(|width| TrackComponentOf::px(px(width)))
                            .into(),
                        grid_template_rows: vec![TrackComponentOf::px(px(20.0))],
                        ..NodeInputOf::default()
                    },
                )
                .style(
                    2,
                    NodeInputOf {
                        display: Display::Grid,
                        grid_column: GridPlacement::try_lines(1, 4).unwrap(),
                        grid_template_columns: vec![TrackComponentOf::Subgrid(
                            crate::SubgridTrack {
                                name_components: Vec::new(),
                            },
                        )],
                        grid_template_rows: vec![TrackComponentOf::px(px(20.0))],
                        ..NodeInputOf::default()
                    },
                )
                .style(
                    3,
                    NodeInputOf {
                        display: Display::Block,
                        raw_grid_column: placement.clone(),
                        ..NodeInputOf::default()
                    },
                );
            let batch = fri08_c03_auto_fit_batch(&tree, container);
            let child = fri08_c01_placement_output(&batch, 3);
            (child.location.x, child.size.width)
        })
        .ok();
        let expected = Some((px(left), px(width)));
        if actual != expected {
            differences.push(format!("{placement:?}: {actual:?}, expected {expected:?}"));
        }
    }
    assert!(differences.is_empty(), "{}", differences.join("\n"));
}

#[test]
fn subgrid_clips_extreme_raw_placement_f32() {
    assert_subgrid_clips_extreme_raw_placement::<f32>();
}

#[test]
fn subgrid_clips_extreme_raw_placement_f64() {
    assert_subgrid_clips_extreme_raw_placement::<f64>();
}

#[test]
fn grid_subgrid_declaration_without_parent_grid_keeps_ordinary_grid_fallback() {
    let mut tree = OracleTree::new();
    tree.insert_children(1, vec![2]);
    tree.insert_children(2, vec![]);
    tree.insert_style(
        1,
        NodeInput {
            display: Display::Grid,
            grid_template_columns: vec![empty_subgrid_track()],
            grid_template_rows: vec![TrackComponent::AUTO],
            grid_auto_columns: vec![TrackComponent::px(20.0)],
            ..NodeInput::default()
        },
    );
    tree.insert_style(2, NodeInput::default());
    tree.insert_measure(2, ComputeOutput::from_outer_size(Size::new(20.0, 10.0)));

    let output = crate::compute_grid(
        &mut tree.borrow_compute(),
        1,
        ComputeInput::for_child(
            RunMode::PerformLayout,
            SizingMode::InherentSize,
            RequestedAxis::Both,
            Size::NONE,
            Size::NONE,
            crate::ContainingLayoutContext::new(
                crate::geometry::FlowAxes::new(
                    crate::WritingMode::HorizontalTb,
                    crate::Direction::Ltr,
                ),
                crate::ParentFormattingContext::NoParent,
            ),
            SizingRequestOf::new(
                Size::new(
                    AxisSizingRequestOf::new(
                        AvailableSpaceOf::Indefinite,
                        SizingOperation::MaxContent,
                    ),
                    AxisSizingRequestOf::new(
                        AvailableSpaceOf::Indefinite,
                        SizingOperation::MaxContent,
                    ),
                ),
                OrthogonalFallbackOf::new(
                    Size::NONE,
                    crate::NearestScrollportOf::Absent,
                    Size::splat(crate::OrthogonalFallbackLimitOf::Unbounded),
                ),
            ),
        ),
    )
    .unwrap();

    assert_eq!(output.content_size, Size::new(20.0, 10.0));
    assert_eq!(
        tree.layout(2).expect("node layout is staged").size,
        Size::new(20.0, 10.0)
    );
}

#[test]
fn nested_inherited_grid_axis_preserves_owner_targets_and_physical_points_through_context_builder()
{
    let owner_flow = FlowAxes::new(WritingMode::HorizontalTb, Direction::Ltr);
    let progression = owner_flow.physical_axis_progression(PhysicalAxis::Vertical);
    let owner_member =
        inherited_placement_member(91, GridAxisKind::Row, AncestorBaselineRole::First, 1, 17.0);
    let owner_group = AncestorBaselineGroup::reduce(
        1_u32,
        GridAxisKind::Row,
        PhysicalAxis::Vertical,
        crate::GridBaselineReference::Physical,
        4,
        [owner_member],
    );
    let owner_direct_member =
        inherited_placement_member(92, GridAxisKind::Row, AncestorBaselineRole::First, 1, 10.0);
    // Four25px tracks with10px gutters place owner row1 at35..60. Its
    // immutable First distance17 selects physical52, so a port10 item starts42.
    let owner_start = 35.0;
    let owner_end = 60.0;
    let owner_point = 52.0;
    let owner_offset = owner_group
        .placement_offset(owner_direct_member, owner_start, owner_end)
        .expect("owner-direct item selects the declared owner track");
    assert_eq!(owner_offset, 42.0);
    assert_eq!(owner_offset - owner_start, 7.0);
    assert_eq!(owner_offset + 10.0, owner_point);
    let ancestor_groups = final_ancestor_baseline_groups_for_transport_test(
        owner_group.clone(),
        AncestorBaselineGroup::reduce(
            1_u32,
            GridAxisKind::Column,
            PhysicalAxis::Horizontal,
            crate::GridBaselineReference::Physical,
            1,
            Vec::<AncestorBaselineMember<u32>>::new(),
        ),
        owner_flow,
    );
    let parent_style = NodeInput {
        display: Display::Grid,
        ..NodeInput::default()
    };
    let child_style = NodeInput {
        display: Display::Grid,
        grid_template_rows: subgrid_track(),
        gap: Size::new(Length::ZERO, Length::px(26.0)),
        ..NodeInput::default()
    };
    let context = subgrid_child_parent_context_from_ancestor_groups(
        SubgridChildParentContextInput {
            minimum_functions: GridMinimumParentFunctions {
                columns: None,
                rows: None,
            },
            item: SubgridItemReport {
                node: 7_u32,
                column: subgrid_axis_report(&parent_style, &child_style, GridAxisKind::Column),
                row: subgrid_axis_report(&parent_style, &child_style, GridAxisKind::Row),
            },
            child_style: &child_style,
            area: GridArea {
                row: 0,
                column: 0,
                row_end: 4,
                column_end: 1,
                size: LogicalSizeOf::new(25.0, 130.0),
            },
            content_box_size: Size::new(25.0, 130.0),
            columns: &[25.0],
            rows: &[25.0; 4],
            gap: LogicalSizeOf::new(0.0, 10.0),
            parent_named_columns: &NamedGridLines::new(GridAxisKind::Column, 1),
            parent_named_rows: &NamedGridLines::new(GridAxisKind::Row, 4),
            parent_area_facts: None,
            margin: Edges::all(Some(0.0)),
            border: Edges::ZERO,
            padding: Edges::ZERO,
        },
        &ancestor_groups,
        1_u32,
    )
    .unwrap();

    let row = context.rows.as_ref().unwrap();
    // The actual26px gutters consume8px from each incident track side:
    // [17,9,9,17] + three26px gutters still occupy the authored130px span.
    assert_eq!(row.geometry.sizes(), &[17.0, 9.0, 9.0, 17.0]);
    assert_eq!(row.geometry.total_extent(), 130.0);
    assert_eq!(row.geometry.line_offsets()[1], 43.0);
    let transported = row.owner_baseline_targets.as_ref().unwrap();
    assert_eq!(transported.group.owner(), 1);
    assert_eq!(transported.mapping.owner(), 1);
    assert_eq!(transported.mapping.current_grid(), 7);
    assert_eq!(transported.mapping.boundary_count(), 1);
    assert_eq!(
        transported
            .group
            .target_record(AncestorBaselineRole::First, 1)
            .unwrap()
            .finite_owner_logical_target(),
        17.0,
        "production context transport preserves the original owner's target",
    );
    let placement = InheritedCurrentGridBaselinePlacement::try_derive(
        &transported.group,
        InheritedCurrentGridBaselinePlacementInput {
            reference: crate::GridBaselineReference::Physical,
            axis: GridAxisKind::Row,
            physical_axis: PhysicalAxis::Vertical,
            mapping: transported.mapping.clone(),
            direct_witness: inherited_placement_witness(
                GridAxisKind::Row,
                AncestorBaselineRole::First,
                1,
                progression,
                progression,
            ),
            current_grid: 7,
            item: 11,
        },
    )
    .unwrap();
    assert_eq!(placement.selected_local_track(), 1);
    assert_eq!(placement.immutable_owner_target(), 17.0);
    assert_eq!(placement.translated_target(), 9.0);
    assert_eq!(
        row.geometry.line_offsets()[1] + placement.translated_target(),
        owner_point
    );

    let current_rows = row.tracks.clone();
    let current_groups = final_ancestor_baseline_groups_with_parent_context_for_transport_test(
        final_ancestor_baseline_groups_for_transport_test(
            AncestorBaselineGroup::reduce(
                7_u32,
                GridAxisKind::Row,
                PhysicalAxis::Vertical,
                crate::GridBaselineReference::Physical,
                4,
                Vec::<AncestorBaselineMember<u32>>::new(),
            ),
            AncestorBaselineGroup::reduce(
                7_u32,
                GridAxisKind::Column,
                PhysicalAxis::Horizontal,
                crate::GridBaselineReference::Physical,
                1,
                Vec::<AncestorBaselineMember<u32>>::new(),
            ),
            owner_flow,
        ),
        &context,
    );
    let nested_style = NodeInput {
        display: Display::Grid,
        grid_template_rows: subgrid_track(),
        gap: Size::new(Length::ZERO, Length::px(34.0)),
        ..NodeInput::default()
    };
    let nested_context = subgrid_child_parent_context_from_ancestor_groups(
        SubgridChildParentContextInput {
            minimum_functions: GridMinimumParentFunctions {
                columns: None,
                rows: None,
            },
            item: SubgridItemReport {
                node: 11_u32,
                column: subgrid_axis_report(&child_style, &nested_style, GridAxisKind::Column),
                row: subgrid_axis_report(&child_style, &nested_style, GridAxisKind::Row),
            },
            child_style: &nested_style,
            area: GridArea {
                row: 0,
                column: 0,
                row_end: 4,
                column_end: 1,
                size: LogicalSizeOf::new(25.0, 130.0),
            },
            content_box_size: Size::new(25.0, 130.0),
            columns: &[25.0],
            rows: &current_rows,
            gap: LogicalSizeOf::new(0.0, 26.0),
            parent_named_columns: &NamedGridLines::new(GridAxisKind::Column, 1),
            parent_named_rows: &NamedGridLines::new(GridAxisKind::Row, 4),
            parent_area_facts: None,
            margin: Edges::all(Some(0.0)),
            border: Edges::ZERO,
            padding: Edges::ZERO,
        },
        &current_groups,
        7_u32,
    )
    .unwrap();
    let nested_row = nested_context.rows.as_ref().unwrap();
    // Increasing the gutter26→34 consumes another4px per side.
    // The selected start is now47, so the same physical52 needs distance5.
    assert_eq!(nested_row.geometry.sizes(), &[13.0, 1.0, 1.0, 13.0]);
    assert_eq!(nested_row.geometry.total_extent(), 130.0);
    assert_eq!(nested_row.geometry.line_offsets()[1], 47.0);
    let nested_transport = nested_row.owner_baseline_targets.as_ref().unwrap();
    assert_eq!(nested_transport.group.owner(), 1);
    assert_eq!(nested_transport.mapping.owner(), 1);
    assert_eq!(nested_transport.mapping.current_grid(), 11);
    assert_eq!(nested_transport.mapping.boundary_count(), 2);
    let nested_placement = InheritedCurrentGridBaselinePlacement::try_derive(
        &nested_transport.group,
        InheritedCurrentGridBaselinePlacementInput {
            reference: crate::GridBaselineReference::Physical,
            axis: GridAxisKind::Row,
            physical_axis: PhysicalAxis::Vertical,
            mapping: nested_transport.mapping.clone(),
            direct_witness: CurrentGridDirectWitness::new(
                11,
                12,
                GridAxisKind::Row,
                GridTrackSpan::new(1, 2),
                AncestorBaselineRole::First,
                progression,
                progression,
            ),
            current_grid: 11,
            item: 12,
        },
    )
    .unwrap();
    assert_eq!(nested_placement.selected_local_track(), 1);
    assert_eq!(nested_placement.immutable_owner_target(), 17.0);
    assert_eq!(nested_placement.translated_target(), 5.0);
    assert_eq!(
        nested_row.geometry.line_offsets()[1] + nested_placement.translated_target(),
        owner_point
    );
    assert_eq!(
        owner_group
            .target_record(AncestorBaselineRole::First, 1)
            .unwrap()
            .finite_owner_logical_target(),
        17.0,
    );
}

#[test]
fn fri06_c12_t08_refreshed_subgrid_offsets_remain_logical_until_projection() {
    let mut tree = OracleTree::new()
        .children(1, [2])
        .children(2, [3])
        .children(3, [])
        .style(
            1,
            NodeInput {
                display: Display::Grid,
                direction: Direction::Rtl,
                size: Size::new(PreferredSize::px(100.0), PreferredSize::px(100.0)),
                grid_template_columns: vec![TrackComponent::px(100.0)],
                grid_template_rows: vec![TrackComponent::px(100.0)],
                ..NodeInput::default()
            },
        )
        .style(
            2,
            NodeInput {
                display: Display::Grid,
                direction: Direction::Rtl,
                grid_template_columns: vec![empty_subgrid_track()],
                grid_template_rows: vec![empty_subgrid_track()],
                margin: Edges {
                    left: LengthAuto::px(24.0),
                    right: LengthAuto::px(10.0),
                    ..Edges::all(LengthAuto::ZERO)
                },
                ..NodeInput::default()
            },
        )
        .style(3, NodeInput::default())
        .measure(3, baseline_measure(20.0, 20.0, Some(14.0), None));

    compute_oracle_grid(&mut tree);

    assert_eq!(
        tree.layout(2).expect("subgrid is laid out").location.x,
        24.0
    );
}

#[test]
fn fri06_c12_t08_smaller_inherited_gap_applies_one_signed_track_transform() {
    assert_eq!(
        fri06_c12_t08_inherited_baseline_gap_position(20.0, 10.0),
        82.0
    );
}

fn fri06_c12_t08_nested_inherited_row_baseline_delta(
    parent_gap: f32,
    child_gap: f32,
    reversed: bool,
    alignment: AlignItems,
) -> f32 {
    let child_writing_mode = if reversed {
        WritingMode::VerticalLr
    } else {
        WritingMode::VerticalRl
    };
    let mut tree = OracleTree::new()
        .children(1, [2])
        .children(2, [3, 4])
        .children(3, [])
        .children(4, [5])
        .children(5, [])
        .style(
            1,
            NodeInput {
                display: Display::Grid,
                writing_mode: WritingMode::VerticalRl,
                size: Size::new(PreferredSize::px(140.0), PreferredSize::px(80.0)),
                grid_template_rows: vec![TrackComponent::px(60.0), TrackComponent::px(60.0)],
                grid_template_columns: vec![TrackComponent::px(40.0), TrackComponent::px(40.0)],
                gap: Size::new(Length::px(parent_gap), Length::ZERO),
                ..NodeInput::default()
            },
        )
        .style(
            2,
            NodeInput {
                display: Display::Grid,
                writing_mode: child_writing_mode,
                grid_row: GridPlacement::try_lines(1, 3).expect("valid inherited row span"),
                grid_column: GridPlacement::try_lines(1, 3).expect("valid column span"),
                grid_template_rows: vec![empty_subgrid_track()],
                grid_template_columns: vec![TrackComponent::px(40.0), TrackComponent::px(40.0)],
                gap: Size::new(Length::px(child_gap), Length::ZERO),
                ..NodeInput::default()
            },
        )
        .style(
            3,
            NodeInput {
                writing_mode: child_writing_mode,
                grid_row: GridPlacement::try_line(2).expect("valid direct row"),
                grid_column: GridPlacement::try_line(1).expect("valid direct column"),
                align_self: Some(alignment),
                ..NodeInput::default()
            },
        )
        .style(
            4,
            NodeInput {
                display: Display::Grid,
                writing_mode: child_writing_mode,
                grid_row: GridPlacement::try_lines(1, 3).expect("valid nested row span"),
                grid_column: GridPlacement::try_line(2).expect("valid nested column"),
                grid_template_rows: vec![empty_subgrid_track()],
                grid_template_columns: vec![TrackComponent::px(40.0)],
                gap: Size::new(Length::px(child_gap), Length::ZERO),
                ..NodeInput::default()
            },
        )
        .style(
            5,
            NodeInput {
                writing_mode: child_writing_mode,
                grid_row: GridPlacement::try_line(2).expect("valid flattened row"),
                align_self: Some(alignment),
                ..NodeInput::default()
            },
        );
    let (direct, nested) = match alignment {
        AlignItems::Baseline => (
            vertical_baseline_measure(30.0, 20.0, Some(20.0), None),
            vertical_baseline_measure(30.0, 20.0, Some(5.0), None),
        ),
        AlignItems::LastBaseline => (
            vertical_baseline_measure(30.0, 20.0, None, Some(5.0)),
            vertical_baseline_measure(30.0, 20.0, None, Some(20.0)),
        ),
        _ => unreachable!("the inherited-row control uses baseline alignment"),
    };
    tree = tree.measure(3, direct).measure(5, nested);

    {
        let axes = Size::new(
            AxisSizingRequestOf::new(
                AvailableSpaceOf::Definite(crate::NonNegativeFiniteOf::new(140.0).unwrap()),
                SizingOperation::Normal,
            ),
            AxisSizingRequestOf::new(
                AvailableSpaceOf::Definite(crate::NonNegativeFiniteOf::new(80.0).unwrap()),
                SizingOperation::Normal,
            ),
        );
        let definite = axes.map(|axis| axis.space().definite_size());
        let style = tree.node_input(1);
        let flow = FlowAxes::new(style.writing_mode, style.direction);
        let request = LayoutRootRequestOf::standalone(
            SizingRequestOf::new(
                axes,
                OrthogonalFallbackOf::new(
                    definite,
                    crate::NearestScrollportOf::Absent,
                    Size::splat(crate::OrthogonalFallbackLimitOf::Unbounded),
                ),
            ),
            flow,
            definite,
        );
        compute_root(&mut tree.borrow_compute(), 1, request)
    }
    .expect("nested inherited-row layout computes");
    round_layout(&mut tree.borrow_compute(), 1).expect("nested inherited-row layout rounds");

    let direct_layout = tree.final_layout(3).expect("direct member is laid out");
    let nested_grid = tree.final_layout(4).expect("nested grid is laid out");
    let nested_layout = tree.final_layout(5).expect("flattened member is laid out");
    let (direct_baseline, nested_baseline) = match alignment {
        AlignItems::Baseline => (20.0, 5.0),
        AlignItems::LastBaseline => (25.0, 10.0),
        _ => unreachable!("the inherited-row control uses baseline alignment"),
    };
    direct_layout.location.x + direct_baseline
        - nested_grid.location.x
        - nested_layout.location.x
        - nested_baseline
}

#[test]
fn fri06_c12_t08_inherited_row_gap_adjustment_stays_in_member_and_view_mapping() {
    let cases = [
        (10.0, 20.0, false, AlignItems::Baseline),
        (10.0, 20.0, false, AlignItems::LastBaseline),
        (20.0, 10.0, false, AlignItems::Baseline),
        (20.0, 10.0, false, AlignItems::LastBaseline),
        (10.0, 20.0, true, AlignItems::Baseline),
        (10.0, 20.0, true, AlignItems::LastBaseline),
        (20.0, 10.0, true, AlignItems::Baseline),
        (20.0, 10.0, true, AlignItems::LastBaseline),
    ];
    let deltas = cases.map(|(parent_gap, child_gap, reversed, alignment)| {
        fri06_c12_t08_nested_inherited_row_baseline_delta(
            parent_gap, child_gap, reversed, alignment,
        )
    });
    assert_eq!(deltas, [0.0; 8]);
}

#[test]
fn nested_subgrid_percent_columns_rerun_rows_after_inherited_width_and_margin() {
    let mut tree = OracleTree::new()
        .children(1, [2])
        .children(2, [3])
        .children(3, [4])
        .children(4, [])
        .style(
            1,
            NodeInput {
                display: Display::InlineGrid,
                grid_template_columns: vec![TrackComponent::px(100.0)],
                ..NodeInput::DEFAULT
            },
        )
        .style(
            2,
            NodeInput {
                display: Display::Grid,
                grid_template_columns: vec![empty_subgrid_track()],
                grid_template_rows: vec![empty_subgrid_track()],
                margin: Edges {
                    left: LengthAuto::px(10.0),
                    right: LengthAuto::px(5.0),
                    ..Edges::all(LengthAuto::ZERO)
                },
                ..NodeInput::DEFAULT
            },
        )
        .style(
            3,
            NodeInput {
                display: Display::Grid,
                grid_template_columns: vec![TrackComponent::percent(1.0)],
                grid_template_rows: vec![empty_subgrid_track()],
                margin: Edges {
                    right: LengthAuto::px(5.0),
                    ..Edges::all(LengthAuto::ZERO)
                },
                ..NodeInput::DEFAULT
            },
        )
        .style(4, NodeInput::default())
        .measure_when(
            4,
            OracleMeasurement::new(ComputeOutput::from_outer_size(Size::new(100.0, 64.0)))
                .run_mode(RunMode::ComputeSize)
                .known(Size::new(None, None))
                .operations(
                    (Size::new(
                        AxisSizingRequestOf::new(
                            AvailableSpaceOf::Definite(
                                crate::NonNegativeFiniteOf::new(100.0).unwrap(),
                            ),
                            SizingOperation::Normal,
                        ),
                        AxisSizingRequestOf::new(
                            AvailableSpaceOf::Indefinite,
                            SizingOperation::MaxContent,
                        ),
                    ))
                    .map(|axis| Some(axis.operation())),
                )
                .spaces(
                    (Size::new(
                        AxisSizingRequestOf::new(
                            AvailableSpaceOf::Definite(
                                crate::NonNegativeFiniteOf::new(100.0).unwrap(),
                            ),
                            SizingOperation::Normal,
                        ),
                        AxisSizingRequestOf::new(
                            AvailableSpaceOf::Indefinite,
                            SizingOperation::MaxContent,
                        ),
                    ))
                    .map(|axis| {
                        (axis.operation() == SizingOperation::Normal).then_some(axis.space())
                    }),
                ),
        )
        .measure_when(
            4,
            OracleMeasurement::new(ComputeOutput::from_outer_size(Size::new(80.0, 96.0)))
                .run_mode(RunMode::ComputeSize)
                .known(Size::new(Some(80.0), None))
                .operations(
                    (Size::new(
                        AxisSizingRequestOf::new(
                            AvailableSpaceOf::Definite(
                                crate::NonNegativeFiniteOf::new(80.0).unwrap(),
                            ),
                            SizingOperation::Normal,
                        ),
                        AxisSizingRequestOf::new(
                            AvailableSpaceOf::Indefinite,
                            SizingOperation::MaxContent,
                        ),
                    ))
                    .map(|axis| Some(axis.operation())),
                )
                .spaces(
                    (Size::new(
                        AxisSizingRequestOf::new(
                            AvailableSpaceOf::Definite(
                                crate::NonNegativeFiniteOf::new(80.0).unwrap(),
                            ),
                            SizingOperation::Normal,
                        ),
                        AxisSizingRequestOf::new(
                            AvailableSpaceOf::Indefinite,
                            SizingOperation::MaxContent,
                        ),
                    ))
                    .map(|axis| {
                        (axis.operation() == SizingOperation::Normal).then_some(axis.space())
                    }),
                ),
        )
        .measure_when(
            4,
            OracleMeasurement::new(ComputeOutput::from_outer_size(Size::new(80.0, 96.0)))
                .run_mode(RunMode::PerformLayout)
                .known(Size::new(None, None))
                .operations(
                    (Size::new(
                        AxisSizingRequestOf::new(
                            AvailableSpaceOf::Definite(
                                crate::NonNegativeFiniteOf::new(80.0).unwrap(),
                            ),
                            SizingOperation::Normal,
                        ),
                        AxisSizingRequestOf::new(
                            AvailableSpaceOf::Definite(
                                crate::NonNegativeFiniteOf::new(96.0).unwrap(),
                            ),
                            SizingOperation::Normal,
                        ),
                    ))
                    .map(|axis| Some(axis.operation())),
                )
                .spaces(
                    (Size::new(
                        AxisSizingRequestOf::new(
                            AvailableSpaceOf::Definite(
                                crate::NonNegativeFiniteOf::new(80.0).unwrap(),
                            ),
                            SizingOperation::Normal,
                        ),
                        AxisSizingRequestOf::new(
                            AvailableSpaceOf::Definite(
                                crate::NonNegativeFiniteOf::new(96.0).unwrap(),
                            ),
                            SizingOperation::Normal,
                        ),
                    ))
                    .map(|axis| {
                        (axis.operation() == SizingOperation::Normal).then_some(axis.space())
                    }),
                ),
        )
        .measure(4, ComputeOutput::from_outer_size(Size::new(100.0, 64.0)));

    let output = crate::compute_grid(
        &mut tree.borrow_compute(),
        1,
        ComputeInput::for_child(
            RunMode::PerformLayout,
            SizingMode::InherentSize,
            RequestedAxis::Both,
            Size::NONE,
            Size::NONE,
            crate::ContainingLayoutContext::new(
                crate::geometry::FlowAxes::new(
                    crate::WritingMode::HorizontalTb,
                    crate::Direction::Ltr,
                ),
                crate::ParentFormattingContext::NoParent,
            ),
            SizingRequestOf::new(
                Size::new(
                    AxisSizingRequestOf::new(
                        AvailableSpaceOf::Indefinite,
                        SizingOperation::MaxContent,
                    ),
                    AxisSizingRequestOf::new(
                        AvailableSpaceOf::Indefinite,
                        SizingOperation::MaxContent,
                    ),
                ),
                OrthogonalFallbackOf::new(
                    Size::NONE,
                    crate::NearestScrollportOf::Absent,
                    Size::splat(crate::OrthogonalFallbackLimitOf::Unbounded),
                ),
            ),
        ),
    )
    .unwrap();
    tree.borrow_compute()
        .set_unrounded(crate::test_support::layout_tree::ordinary_unrounded_box(
            1,
            NodeOutput {
                size: output.size,
                content_size: output.content_size,
                ..NodeOutput::new()
            },
        ));
    round_layout(&mut tree.borrow_compute(), 1).unwrap();

    assert_eq!(output.size.height, 96.0);
    assert_eq!(tree.final_layout(2).unwrap().size, Size::new(85.0, 96.0));
    assert_eq!(tree.final_layout(3).unwrap().size, Size::new(80.0, 96.0));
}

#[test]
fn row_subgrid_percent_column_leaf_uses_spanned_inline_size_for_row_contribution() {
    let mut tree = OracleTree::new()
        .children(1, [2])
        .children(2, [3])
        .children(3, [])
        .style(
            1,
            NodeInput {
                display: Display::InlineGrid,
                grid_template_columns: vec![TrackComponent::px(100.0)],
                ..NodeInput::DEFAULT
            },
        )
        .style(
            2,
            NodeInput {
                display: Display::Grid,
                grid_template_columns: vec![
                    TrackComponent::percent(0.5),
                    TrackComponent::percent(0.5),
                ],
                grid_template_rows: vec![empty_subgrid_track()],
                ..NodeInput::DEFAULT
            },
        )
        .style(
            3,
            NodeInput {
                grid_column: GridPlacement::try_line(1).expect("valid grid line"),
                ..NodeInput::DEFAULT
            },
        )
        .measure_when(
            3,
            OracleMeasurement::new(ComputeOutput::from_outer_size(Size::new(50.0, 90.0)))
                .run_mode(RunMode::ComputeSize)
                .known(Size::new(Some(50.0), None))
                .operations(
                    (Size::new(
                        AxisSizingRequestOf::new(
                            AvailableSpaceOf::Definite(
                                crate::NonNegativeFiniteOf::new(50.0).unwrap(),
                            ),
                            SizingOperation::Normal,
                        ),
                        AxisSizingRequestOf::new(
                            AvailableSpaceOf::Indefinite,
                            SizingOperation::MaxContent,
                        ),
                    ))
                    .map(|axis| Some(axis.operation())),
                )
                .spaces(
                    (Size::new(
                        AxisSizingRequestOf::new(
                            AvailableSpaceOf::Definite(
                                crate::NonNegativeFiniteOf::new(50.0).unwrap(),
                            ),
                            SizingOperation::Normal,
                        ),
                        AxisSizingRequestOf::new(
                            AvailableSpaceOf::Indefinite,
                            SizingOperation::MaxContent,
                        ),
                    ))
                    .map(|axis| {
                        (axis.operation() == SizingOperation::Normal).then_some(axis.space())
                    }),
                ),
        )
        .measure_when(
            3,
            OracleMeasurement::new(ComputeOutput::from_outer_size(Size::new(50.0, 90.0)))
                .run_mode(RunMode::PerformLayout)
                .known(Size::new(None, None))
                .operations(
                    (Size::new(
                        AxisSizingRequestOf::new(
                            AvailableSpaceOf::Definite(
                                crate::NonNegativeFiniteOf::new(50.0).unwrap(),
                            ),
                            SizingOperation::Normal,
                        ),
                        AxisSizingRequestOf::new(
                            AvailableSpaceOf::Definite(
                                crate::NonNegativeFiniteOf::new(90.0).unwrap(),
                            ),
                            SizingOperation::Normal,
                        ),
                    ))
                    .map(|axis| Some(axis.operation())),
                )
                .spaces(
                    (Size::new(
                        AxisSizingRequestOf::new(
                            AvailableSpaceOf::Definite(
                                crate::NonNegativeFiniteOf::new(50.0).unwrap(),
                            ),
                            SizingOperation::Normal,
                        ),
                        AxisSizingRequestOf::new(
                            AvailableSpaceOf::Definite(
                                crate::NonNegativeFiniteOf::new(90.0).unwrap(),
                            ),
                            SizingOperation::Normal,
                        ),
                    ))
                    .map(|axis| {
                        (axis.operation() == SizingOperation::Normal).then_some(axis.space())
                    }),
                ),
        )
        .measure(3, ComputeOutput::from_outer_size(Size::new(100.0, 40.0)));

    let output = crate::compute_grid(
        &mut tree.borrow_compute(),
        1,
        ComputeInput::for_child(
            RunMode::PerformLayout,
            SizingMode::InherentSize,
            RequestedAxis::Both,
            Size::NONE,
            Size::NONE,
            crate::ContainingLayoutContext::new(
                crate::geometry::FlowAxes::new(
                    crate::WritingMode::HorizontalTb,
                    crate::Direction::Ltr,
                ),
                crate::ParentFormattingContext::NoParent,
            ),
            SizingRequestOf::new(
                Size::new(
                    AxisSizingRequestOf::new(
                        AvailableSpaceOf::Indefinite,
                        SizingOperation::MaxContent,
                    ),
                    AxisSizingRequestOf::new(
                        AvailableSpaceOf::Indefinite,
                        SizingOperation::MaxContent,
                    ),
                ),
                OrthogonalFallbackOf::new(
                    Size::NONE,
                    crate::NearestScrollportOf::Absent,
                    Size::splat(crate::OrthogonalFallbackLimitOf::Unbounded),
                ),
            ),
        ),
    )
    .unwrap();
    tree.borrow_compute()
        .set_unrounded(crate::test_support::layout_tree::ordinary_unrounded_box(
            1,
            NodeOutput {
                size: output.size,
                content_size: output.content_size,
                ..NodeOutput::new()
            },
        ));
    round_layout(&mut tree.borrow_compute(), 1).unwrap();

    assert_eq!(output.size, Size::new(100.0, 90.0));
    assert_eq!(tree.final_layout(2).unwrap().size, Size::new(100.0, 90.0));
    assert!(
        tree.inputs(3)
            .iter()
            .any(|input| input.run_mode() == RunMode::ComputeSize
                && input.known().width == Some(50.0)
                && input.sizing_request().axes().width
                    == AxisSizingRequestOf::new(
                        AvailableSpaceOf::Definite(crate::NonNegativeFiniteOf::new(50.0).unwrap()),
                        SizingOperation::Normal
                    )),
        "row contribution should measure the leaf against its 50px column span"
    );
}

#[test]
fn orthogonal_nested_subgrid_width_includes_full_horizontal_leaf_contribution() {
    let mut tree = OracleTree::new()
        .children(1, [2])
        .children(2, [3, 6])
        .children(3, [4, 5])
        .children(4, [])
        .children(5, [])
        .children(6, [])
        .style(
            1,
            NodeInput {
                display: Display::InlineGrid,
                gap: Size::new(Length::px(20.0), Length::px(20.0)),
                border: Edges::all(Length::px(3.0)),
                grid_template_columns: vec![TrackComponent::px(100.0), TrackComponent::AUTO],
                grid_template_rows: vec![TrackComponent::px(100.0), TrackComponent::AUTO],
                ..NodeInput::DEFAULT
            },
        )
        .style(
            2,
            NodeInput {
                display: Display::Grid,
                writing_mode: WritingMode::VerticalRl,
                gap: Size::new(Length::px(100.0), Length::px(100.0)),
                grid_template_columns: vec![empty_subgrid_track()],
                grid_template_rows: vec![empty_subgrid_track()],
                grid_column: GridPlacement::try_span(2).expect("valid grid span"),
                grid_row: GridPlacement::try_span(2).expect("valid grid span"),
                ..NodeInput::DEFAULT
            },
        )
        .style(
            3,
            NodeInput {
                display: Display::Grid,
                writing_mode: WritingMode::HorizontalTb,
                gap: Size::new(Length::px(100.0), Length::px(100.0)),
                grid_template_columns: vec![TrackComponent::px(100.0)],
                grid_template_rows: vec![empty_subgrid_track()],
                grid_column: GridPlacement::try_span(2).expect("valid grid span"),
                ..NodeInput::DEFAULT
            },
        )
        .style(4, NodeInput::DEFAULT)
        .style(
            5,
            NodeInput {
                grid_row: GridPlacement::try_line(2).expect("valid grid line"),
                ..NodeInput::DEFAULT
            },
        )
        .style(
            6,
            NodeInput {
                grid_column: GridPlacement::try_line(2).expect("valid grid line"),
                grid_row: GridPlacement::try_line(1).expect("valid grid line"),
                ..NodeInput::DEFAULT
            },
        )
        .measure(4, ComputeOutput::from_outer_size(Size::new(24.0, 24.0)))
        .measure(5, ComputeOutput::from_outer_size(Size::new(24.0, 24.0)))
        .measure(6, ComputeOutput::from_outer_size(Size::new(72.0, 24.0)));

    let output = crate::compute_grid(
        &mut tree.borrow_compute(),
        1,
        ComputeInput::for_child(
            RunMode::PerformLayout,
            SizingMode::InherentSize,
            RequestedAxis::Both,
            Size::NONE,
            Size::NONE,
            crate::ContainingLayoutContext::new(
                crate::geometry::FlowAxes::new(
                    crate::WritingMode::HorizontalTb,
                    crate::Direction::Ltr,
                ),
                crate::ParentFormattingContext::NoParent,
            ),
            SizingRequestOf::new(
                Size::new(
                    AxisSizingRequestOf::new(
                        AvailableSpaceOf::Indefinite,
                        SizingOperation::MaxContent,
                    ),
                    AxisSizingRequestOf::new(
                        AvailableSpaceOf::Indefinite,
                        SizingOperation::MaxContent,
                    ),
                ),
                OrthogonalFallbackOf::new(
                    Size::NONE,
                    crate::NearestScrollportOf::Absent,
                    Size::splat(crate::OrthogonalFallbackLimitOf::Unbounded),
                ),
            ),
        ),
    )
    .unwrap();

    assert_eq!(output.size.width, 238.0);
}

#[test]
fn grid_auto_size_ignores_ineligible_row_subgrid_when_resolving_percent_columns() {
    let mut tree = OracleTree::new()
        .measure(2, ComputeOutput::from_outer_size(Size::new(100.0, 100.0)))
        .measure(3, ComputeOutput::from_outer_size(Size::new(100.0, 100.0)));
    tree.insert_children(1, vec![2, 3]);
    for node in 2..=3 {
        tree.insert_children(node, vec![]);
        tree.insert_style(node, NodeInput::default());
    }
    tree.insert_style(
        1,
        NodeInput {
            display: Display::Grid,
            grid_template_columns: vec![TrackComponent::percent(0.5), TrackComponent::percent(0.5)],
            grid_template_rows: vec![empty_subgrid_track()],
            ..NodeInput::default()
        },
    );
    tree.insert_style(
        2,
        NodeInput {
            grid_column: GridPlacement::try_line(1).expect("valid grid line"),
            grid_row: GridPlacement::try_line(1).expect("valid grid line"),
            ..NodeInput::default()
        },
    );
    tree.insert_style(
        3,
        NodeInput {
            grid_column: GridPlacement::try_line(2).expect("valid grid line"),
            grid_row: GridPlacement::try_line(1).expect("valid grid line"),
            ..NodeInput::default()
        },
    );

    let output = crate::compute_grid(
        &mut tree.borrow_compute(),
        1,
        ComputeInput::for_child(
            RunMode::PerformLayout,
            SizingMode::InherentSize,
            RequestedAxis::Both,
            Size::NONE,
            Size::NONE,
            crate::ContainingLayoutContext::new(
                crate::geometry::FlowAxes::new(
                    crate::WritingMode::HorizontalTb,
                    crate::Direction::Ltr,
                ),
                crate::ParentFormattingContext::NoParent,
            ),
            SizingRequestOf::new(
                Size::new(
                    AxisSizingRequestOf::new(
                        AvailableSpaceOf::Indefinite,
                        SizingOperation::MaxContent,
                    ),
                    AxisSizingRequestOf::new(
                        AvailableSpaceOf::Indefinite,
                        SizingOperation::MaxContent,
                    ),
                ),
                OrthogonalFallbackOf::new(
                    Size::NONE,
                    crate::NearestScrollportOf::Absent,
                    Size::splat(crate::OrthogonalFallbackLimitOf::Unbounded),
                ),
            ),
        ),
    )
    .unwrap();

    // An ineligible row declaration does not suppress the two column queries.
    // Cyclic 50% columns each contribute 100 as Auto, establishing width 200.
    assert_eq!(output.size.width, 200.0);
}

#[test]
fn subgrid_intrinsic_row_sizing_uses_subgrid_content_not_parent_height() {
    let mut tree = OracleTree::new()
        .children(1, [2])
        .children(2, [3])
        .children(3, [4, 5, 6, 7])
        .style(
            1,
            NodeInput {
                display: Display::Block,
                size: Size::new(PreferredSize::px(100.0), PreferredSize::px(200.0)),
                ..NodeInput::DEFAULT
            },
        )
        .style(
            2,
            NodeInput {
                display: Display::Grid,
                size: Size::new(PreferredSize::MIN_CONTENT, PreferredSize::AUTO),
                grid_template_columns: vec![TrackComponent::AUTO],
                grid_template_rows: vec![TrackComponent::AUTO; 4],
                ..NodeInput::DEFAULT
            },
        )
        .style(
            3,
            NodeInput {
                display: Display::Grid,
                grid_template_columns: vec![empty_subgrid_track()],
                grid_template_rows: vec![empty_subgrid_track()],
                grid_column: GridPlacement::try_lines(1, -1).expect("valid grid lines"),
                grid_row: GridPlacement::try_lines(1, -1).expect("valid grid lines"),
                ..NodeInput::DEFAULT
            },
        )
        .style(
            4,
            NodeInput {
                display: Display::InlineBlock,
                size: Size::new(PreferredSize::px(25.0), PreferredSize::px(25.0)),
                ..NodeInput::DEFAULT
            },
        )
        .style(
            5,
            NodeInput {
                display: Display::InlineBlock,
                size: Size::new(PreferredSize::px(100.0), PreferredSize::px(25.0)),
                ..NodeInput::DEFAULT
            },
        )
        .style(
            6,
            NodeInput {
                display: Display::InlineBlock,
                size: Size::new(PreferredSize::px(50.0), PreferredSize::px(25.0)),
                ..NodeInput::DEFAULT
            },
        )
        .style(
            7,
            NodeInput {
                display: Display::InlineBlock,
                size: Size::new(PreferredSize::px(75.0), PreferredSize::px(25.0)),
                ..NodeInput::DEFAULT
            },
        );

    {
        let axes = Size::new(
            AxisSizingRequestOf::new(AvailableSpaceOf::Indefinite, SizingOperation::MaxContent),
            AxisSizingRequestOf::new(AvailableSpaceOf::Indefinite, SizingOperation::MaxContent),
        );
        let definite = axes.map(|axis| axis.space().definite_size());
        let style = tree.node_input(1);
        let flow = FlowAxes::new(style.writing_mode, style.direction);
        let request = LayoutRootRequestOf::standalone(
            SizingRequestOf::new(
                axes,
                OrthogonalFallbackOf::new(
                    definite,
                    crate::NearestScrollportOf::Absent,
                    Size::splat(crate::OrthogonalFallbackLimitOf::Unbounded),
                ),
            ),
            flow,
            definite,
        );
        compute_root(&mut tree.borrow_compute(), 1, request)
    }
    .unwrap();
    round_layout(&mut tree.borrow_compute(), 1).unwrap();
    let child = tree.final_layout(2).expect("child grid should be laid out");
    let subgrid = tree.final_layout(3).expect("subgrid should be laid out");

    assert_eq!(child.size, Size::new(100.0, 100.0));
    assert_eq!(subgrid.size, Size::new(100.0, 100.0));
}

#[test]
fn lane_axis_margin_box_measurement_resolves_affine_margins_against_grid_axis() {
    let margin = lp(4.0, 0.10);
    let child_style = NodeInput {
        margin: Edges {
            left: LengthAuto::value(margin),
            right: LengthAuto::px(6.0),
            top: LengthAuto::ZERO,
            bottom: LengthAuto::ZERO,
        },
        ..NodeInput::default()
    };
    let container_style = NodeInput::default();
    let constants = Constants {
        child_orthogonal_fallback: OrthogonalFallbackOf::new(
            Size::NONE,
            crate::NearestScrollportOf::Absent,
            Size::splat(crate::OrthogonalFallbackLimitOf::Unknown),
        ),
        flow_axes: crate::geometry::FlowAxes::new(
            container_style.writing_mode,
            container_style.direction,
        ),
        explicit_definite_content_size: Size::new(Some(200.0), Some(80.0)),
        indefinite_percentage_axis: None,
        node_outer_size: Size::new(Some(200.0), Some(80.0)),
        node_inner_size: Size::new(Some(200.0), Some(80.0)),
        node_min_size: Size::NONE,
        node_definite_min_size: Size::NONE,
        fallback_minimum_outer: Size::splat(FixedFallbackSource::Unqualified),
        fallback_maximum_outer: Size::splat(FixedFallbackSource::Unqualified),
        node_max_size: Size::NONE,
        available_inner_size: Size::new(Some(200.0), Some(80.0)),
        content_box_inset: Edges::ZERO,
        scrollbar_gutter: Edges::ZERO,
        padding: Edges::ZERO,
        border: Edges::ZERO,
    };
    let mut tree = LaneMarginMeasureTree {
        child_style: child_style.clone(),
        child_output: ComputeOutput::from_sizes_and_baselines(
            Size::new(50.0, 12.0),
            Size::new(50.0, 12.0),
            Baselines::NONE,
        ),
        last_input: None,
    };
    let child_projection = grid_item_projection!(&child_style);
    let container_projection = grid_container_projection!(&container_style);

    let measurement = prepare_lane_axis_margin_box_measurement(
        &tree.borrow_compute(),
        LaneMarginMeasureTree::CHILD,
        LaneAxisMarginBoxMeasureInput {
            child_style: &child_projection,
            container_style: &container_projection,
            constants: &constants,
            lane_axis: GridAxisKind::Column,
            stacking_operation: SizingOperation::Normal,
            inherits_grid_axis: false,
            containing_block: GridLanesItemContainingBlockOf::new(
                constants.flow_axes,
                GridAxisKind::Column,
                200.0,
                LogicalSizeOf::new(Some(200.0), Some(80.0)),
            ),
        },
    )
    .expect("lane margin measurement request is valid");
    let output = tree
        .borrow_compute()
        .compute_child(LaneMarginMeasureTree::CHILD, measurement.input())
        .expect("lane margin measurement succeeds");

    assert_eq!(measurement.extent(output), 80.0);
    let input = tree
        .last_input
        .expect("measurement should compute the child");
    assert_eq!(input.known().width, Some(170.0));
    assert_eq!(input.parent().width, Some(200.0));
    assert_eq!(
        input.sizing_request().axes().width,
        AxisSizingRequestOf::new(
            AvailableSpaceOf::Definite(crate::NonNegativeFiniteOf::new(170.0).unwrap()),
            SizingOperation::Normal
        )
    );
}

struct LaneMarginMeasureTree {
    child_style: NodeInput,
    child_output: ComputeOutput,
    last_input: Option<ComputeInput>,
}

impl LaneMarginMeasureTree {
    const ROOT: usize = 0;
    const CHILD: usize = 1;
}

impl Traverse for LaneMarginMeasureTree {
    type Node = usize;
    type Scalar = Scalar;
    type Children<'a>
        = std::vec::IntoIter<Self::Node>
    where
        Self: 'a;

    fn children(&self, node: Self::Node) -> Self::Children<'_> {
        match node {
            Self::ROOT => vec![Self::CHILD].into_iter(),
            _ => Vec::new().into_iter(),
        }
    }

    fn child_count(&self, node: Self::Node) -> usize {
        usize::from(node == Self::ROOT)
    }

    fn child(&self, _node: Self::Node, index: usize) -> Self::Node {
        assert_eq!(index, 0);
        Self::CHILD
    }
}

struct LaneMarginMeasureTreeCompute<'input> {
    child_style: &'input NodeInput,
    child_output: ComputeOutput,
    last_input: &'input mut Option<ComputeInput>,
}

impl LaneMarginMeasureTree {
    fn borrow_compute(&mut self) -> LaneMarginMeasureTreeCompute<'_> {
        LaneMarginMeasureTreeCompute {
            child_style: &self.child_style,
            child_output: self.child_output,
            last_input: &mut self.last_input,
        }
    }
}

impl LaneMarginMeasureTreeCompute<'_> {
    const ROOT: usize = LaneMarginMeasureTree::ROOT;
    const CHILD: usize = LaneMarginMeasureTree::CHILD;
}

impl<'input> Traverse for LaneMarginMeasureTreeCompute<'input> {
    type Node = usize;
    type Scalar = Scalar;
    type Children<'a>
        = std::vec::IntoIter<Self::Node>
    where
        Self: 'a;

    fn children(&self, node: Self::Node) -> Self::Children<'_> {
        match node {
            Self::ROOT => vec![Self::CHILD].into_iter(),
            _ => Vec::new().into_iter(),
        }
    }

    fn child_count(&self, node: Self::Node) -> usize {
        usize::from(node == Self::ROOT)
    }

    fn child(&self, _node: Self::Node, index: usize) -> Self::Node {
        assert_eq!(index, 0);
        Self::CHILD
    }
}

impl<'input> Compute<'input> for LaneMarginMeasureTreeCompute<'input> {
    fn observe_grid_overflow(
        &mut self,
        _node: Self::Node,
        _probe: crate::grid::GridOverflowProbe<'input, '_, Self::Node, Self::Scalar>,
    ) -> crate::LayoutResultOf<
        Self::Node,
        crate::grid::GridOverflowObservation<Self::Scalar>,
        Self::Scalar,
        core::convert::Infallible,
    > {
        panic!(
            "this focused computation fixture does not admit intrinsic Grid Auto overflow observation"
        );
    }

    fn node_input(&self, node: Self::Node) -> &'input NodeInput {
        assert_eq!(node, Self::CHILD);
        self.child_style
    }

    fn layout_input(&self, node: Self::Node) -> crate::LayoutInputViewOf<'input, Self::Scalar> {
        crate::LayoutInputViewOf::Box(self.node_input(node))
    }

    fn staged_unrounded(
        &self,
        node: Self::Node,
    ) -> crate::engine::contracts::UnroundedEntryResult<
        Self::Node,
        Self::Scalar,
        core::convert::Infallible,
    > {
        Err(crate::test_support::layout_tree::missing_staged_unrounded(
            node,
        ))
    }

    fn set_unrounded(&mut self, _entry: crate::LayoutOutputEntryOf<Self::Node, Self::Scalar>) {
        unreachable!("lane margin measurement should not write layout output");
    }

    fn compute_child(
        &mut self,
        node: Self::Node,
        input: ComputeInput,
    ) -> crate::LayoutResultOf<Self::Node, crate::ComputeOutputOf<Self::Scalar>, Self::Scalar> {
        Ok({
            assert_eq!(node, Self::CHILD);
            *self.last_input = Some(input);
            self.child_output
        })
    }
}

#[test]
fn grid_lane_track_base_rejects_positive_invalid_affine_numeric_result() {
    let outcome = lane_intrinsic_sizing(LaneIntrinsicSizingInput {
        axis: GridAxisKind::Column,
        available: Some(2.0),
        gap: 0.0,
        tracks: vec![TrackSizing::new(
            MinTrackSizing::Calculation(SizingCalculation::value(invalid_numeric_lp())),
            MaxTrackSizing::Auto,
        )],
        content_sized_tracks: vec![0],
        items: Vec::new(),
    });

    let error = outcome.expect_err("invalid lane track sizing must not produce output");

    assert_eq!(error.site(), LayoutErrorSite::Standalone);
    assert_eq!(error.operation(), LayoutOperation::ValueResolution);
    crate::test_support::assert_error_kind(
        error.kind(),
        &LayoutErrorKind::InvalidInput(LayoutInvalidInput::InvalidNumeric {
            value: f32::INFINITY,
        }),
    );
}

#[test]
fn grid_lane_track_base_rejects_signed_invalid_affine_numeric_result() {
    let outcome = lane_intrinsic_sizing(LaneIntrinsicSizingInput {
        axis: GridAxisKind::Column,
        available: Some(f32::MAX),
        gap: 0.0,
        tracks: vec![TrackSizing::new(
            MinTrackSizing::Calculation(SizingCalculation::value(lp(-f32::MAX, -1.0))),
            MaxTrackSizing::Auto,
        )],
        content_sized_tracks: vec![0],
        items: Vec::new(),
    });

    let error = outcome.expect_err("invalid lane track sizing must not produce output");

    assert_eq!(error.site(), LayoutErrorSite::Standalone);
    assert_eq!(error.operation(), LayoutOperation::ValueResolution);
    crate::test_support::assert_error_kind(
        error.kind(),
        &LayoutErrorKind::InvalidInput(LayoutInvalidInput::InvalidNumeric {
            value: f32::NEG_INFINITY,
        }),
    );
}

#[test]
fn grid_lane_track_base_rejects_positive_and_signed_invalid_f64_affine_numeric_results() {
    for (label, absolute, percent, expected) in [
        ("positive", f64::MAX, 1.0_f64, f64::INFINITY),
        ("signed", -f64::MAX, -1.0_f64, f64::NEG_INFINITY),
    ] {
        let outcome = lane_intrinsic_sizing(LaneIntrinsicSizingInputOf::<f64> {
            axis: GridAxisKind::Column,
            available: Some(f64::MAX),
            gap: 0.0,
            tracks: vec![TrackSizingOf::new(
                MinTrackSizingOf::Calculation(SizingCalculationOf::value(
                    LengthPercentageOf::from_coefficients(absolute, percent)
                        .expect("test coefficients are finite"),
                )),
                MaxTrackSizingOf::Auto,
            )],
            content_sized_tracks: vec![0],
            items: Vec::new(),
        });

        let error = outcome.expect_err("invalid lane track sizing must not produce output");

        assert_eq!(error.site(), LayoutErrorSite::Standalone, "{label} site");
        assert_eq!(
            error.operation(),
            LayoutOperation::ValueResolution,
            "{label} operation"
        );
        crate::test_support::assert_error_kind(
            error.kind(),
            &LayoutErrorKindOf::InvalidInput(LayoutInvalidInputOf::InvalidNumeric {
                value: expected,
            }),
        );
    }
}

#[test]
fn lane_intrinsic_public_inputs_accept_non_default_scalar() {
    let facts = LaneContributionFactsOf::<f64> {
        min_content: 1.25_f64,
        max_content: 2.5_f64,
        min_size: 0.75_f64,
        automatic_minimum_applies: true,
    };
    let item = LaneIntrinsicItemOf::<f64>::indefinite(
        "wide",
        LaneTrackSpanLength::new(2).expect("span should be nonzero"),
        facts,
    );
    let input = LaneIntrinsicSizingInputOf::<f64> {
        axis: GridAxisKind::Column,
        available: Some(10.5_f64),
        gap: 1.5_f64,
        tracks: vec![TrackSizingOf::<f64>::AUTO],
        content_sized_tracks: vec![0],
        items: vec![item],
    };

    assert_eq!(input.gap, 1.5_f64);
    assert_eq!(input.items[0].contribution().max_content, 2.5_f64);

    let placement_input = LanePlacementInputOf::<_, f64> {
        grid_axis_tracks: 1,
        grid_axis: crate::GridAxisKind::Column,
        reversed: false,
        lane_gap: 1.5_f64,
        tolerance: GridFlowToleranceOf::Percent(0.25_f64),
        tolerance_basis: 10.5_f64,
        items: Vec::<LaneItemOf<&str, f64>>::new(),
    };

    assert_eq!(
        placement_input.tolerance,
        GridFlowToleranceOf::Percent(0.25_f64)
    );
}

#[test]
fn lane_public_helpers_compute_with_non_default_scalar() {
    let placement = place_lanes(LanePlacementInputOf::<_, f64> {
        grid_axis_tracks: 2,
        grid_axis: crate::GridAxisKind::Column,
        reversed: false,
        lane_gap: 0.5,
        tolerance: GridFlowToleranceOf::Normal { font_size: 0.0 },
        tolerance_basis: 0.0,
        items: vec![
            LaneItemOf {
                item: "a",
                grid_axis_span: 1,
                definite_grid_axis_start: None,
                lane_axis_margin_box: 10.25,
            },
            LaneItemOf {
                item: "b",
                grid_axis_span: 1,
                definite_grid_axis_start: None,
                lane_axis_margin_box: 12.5,
            },
        ],
    })
    .expect("f64 lane placement should compute");

    assert_eq!(placement.content_size, 12.5);
    assert_eq!(placement.item_offsets[1].offset, 0.0);

    let intrinsic = lane_intrinsic_sizing(LaneIntrinsicSizingInputOf::<f64> {
        axis: GridAxisKind::Column,
        available: Some(80.0),
        gap: 1.25,
        tracks: vec![TrackSizingOf::<f64>::AUTO],
        content_sized_tracks: vec![0],
        items: vec![
            LaneIntrinsicItemOf::<f64>::definite(
                "definite",
                LaneTrackSpan::new(1, 2),
                LaneContributionFactsOf {
                    min_content: 9.5,
                    max_content: 14.25,
                    min_size: 7.0,
                    automatic_minimum_applies: true,
                },
            )
            .expect("span is valid"),
        ],
    })
    .expect("f64 lane intrinsic sizing should not fail")
    .expect("f64 lane intrinsic sizing should produce a report");

    assert_eq!(intrinsic.final_track_sizes, vec![9.5]);
}

#[test]
fn grid_lanes_compute_result_accepts_non_default_scalar() {
    #[derive(Clone)]
    struct F64GridTree {
        styles: Vec<NodeInputOf<f64>>,
        children: Vec<Vec<usize>>,
        layouts: Vec<Option<crate::LayoutOutputEntryOf<usize, f64>>>,
    }

    impl Traverse for F64GridTree {
        type Node = usize;
        type Scalar = f64;
        type Children<'a>
            = std::iter::Copied<std::slice::Iter<'a, usize>>
        where
            Self: 'a;

        fn children(&self, node: Self::Node) -> Self::Children<'_> {
            self.children[node].iter().copied()
        }

        fn child_count(&self, node: Self::Node) -> usize {
            self.children[node].len()
        }

        fn child(&self, node: Self::Node, index: usize) -> Self::Node {
            self.children[node][index]
        }
    }

    struct F64GridTreeCompute<'input> {
        styles: &'input Vec<NodeInputOf<f64>>,
        children: &'input Vec<Vec<usize>>,
        layouts: &'input mut Vec<Option<crate::LayoutOutputEntryOf<usize, f64>>>,
    }

    impl F64GridTree {
        fn borrow_compute(&mut self) -> F64GridTreeCompute<'_> {
            F64GridTreeCompute {
                styles: &self.styles,
                children: &self.children,
                layouts: &mut self.layouts,
            }
        }
    }

    impl<'input> Traverse for F64GridTreeCompute<'input> {
        type Node = usize;
        type Scalar = f64;
        type Children<'a>
            = std::iter::Copied<std::slice::Iter<'a, usize>>
        where
            Self: 'a;

        fn children(&self, node: Self::Node) -> Self::Children<'_> {
            self.children[node].iter().copied()
        }

        fn child_count(&self, node: Self::Node) -> usize {
            self.children[node].len()
        }

        fn child(&self, node: Self::Node, index: usize) -> Self::Node {
            self.children[node][index]
        }
    }

    impl<'input> Compute<'input> for F64GridTreeCompute<'input> {
        fn observe_grid_overflow(
            &mut self,
            _node: Self::Node,
            _probe: crate::grid::GridOverflowProbe<'input, '_, Self::Node, Self::Scalar>,
        ) -> crate::LayoutResultOf<
            Self::Node,
            crate::grid::GridOverflowObservation<Self::Scalar>,
            Self::Scalar,
            core::convert::Infallible,
        > {
            panic!(
                "this focused computation fixture does not admit intrinsic Grid Auto overflow observation"
            );
        }

        fn node_input(&self, node: Self::Node) -> &'input NodeInputOf<f64> {
            &self.styles[node]
        }

        fn layout_input(&self, node: Self::Node) -> crate::LayoutInputViewOf<'input, Self::Scalar> {
            crate::LayoutInputViewOf::Box(self.node_input(node))
        }

        fn staged_unrounded(
            &self,
            node: Self::Node,
        ) -> crate::engine::contracts::UnroundedEntryResult<
            Self::Node,
            Self::Scalar,
            core::convert::Infallible,
        > {
            self.layouts
                .get(node)
                .copied()
                .flatten()
                .filter(|entry| entry.node() == node && entry.geometry_state().is_unrounded())
                .ok_or_else(|| crate::test_support::layout_tree::missing_staged_unrounded(node))
        }

        fn set_unrounded(&mut self, entry: crate::LayoutOutputEntryOf<Self::Node, Self::Scalar>) {
            self.layouts[entry.node()] = Some(entry);
        }

        fn compute_child(
            &mut self,
            node: Self::Node,
            input: ComputeInputOf<f64>,
        ) -> crate::LayoutResultOf<Self::Node, crate::ComputeOutputOf<Self::Scalar>, Self::Scalar>
        {
            Ok({
                let style = &self.styles[node];
                let size = input.known().unwrap_or(Size::new(
                    style
                        .size
                        .width
                        .resolve_simple_with_status(input.parent().width)
                        .expect("affine preferred width is supported")
                        .value
                        .or_else(|| {
                            input
                                .sizing_request()
                                .axes()
                                .width
                                .space()
                                .definite_size()
                                .map(crate::NonNegativeFiniteOf::get)
                        })
                        .unwrap_or(0.0),
                    style
                        .size
                        .height
                        .resolve_simple_with_status(input.parent().height)
                        .expect("affine preferred height is supported")
                        .value
                        .or_else(|| {
                            input
                                .sizing_request()
                                .axes()
                                .height
                                .space()
                                .definite_size()
                                .map(crate::NonNegativeFiniteOf::get)
                        })
                        .unwrap_or(0.0),
                ));
                ComputeOutputOf::from_sizes(size, size)
            })
        }
    }

    let root_style = NodeInputOf::<f64> {
        display: Display::GridLanes,
        size: Size::new(PreferredSizeOf::px(120.0), PreferredSizeOf::px(90.0)),
        grid_template_columns: vec![TrackSizingOf::px(60.0).into()],
        grid_auto_rows: vec![TrackSizingOf::px(40.0).into()],
        ..NodeInputOf::default()
    };
    let child_style = NodeInputOf::<f64> {
        size: Size::new(PreferredSizeOf::px(30.0), PreferredSizeOf::px(20.0)),
        ..NodeInputOf::default()
    };
    let mut tree = F64GridTree {
        styles: vec![root_style, child_style],
        children: vec![vec![1], Vec::new()],
        layouts: vec![None, None],
    };

    let computation = compute_grid_with_report(
        &mut tree.borrow_compute(),
        0,
        ComputeInputOf::for_child(
            RunMode::PerformLayout,
            SizingMode::InherentSize,
            RequestedAxis::Both,
            Size::NONE,
            Size::NONE,
            crate::ContainingLayoutContext::new(
                crate::geometry::FlowAxes::new(
                    crate::WritingMode::HorizontalTb,
                    crate::Direction::Ltr,
                ),
                crate::ParentFormattingContext::NoParent,
            ),
            SizingRequestOf::new(
                Size::splat(AxisSizingRequestOf::new(
                    AvailableSpaceOf::Indefinite,
                    SizingOperation::MaxContent,
                )),
                OrthogonalFallbackOf::new(
                    Size::NONE,
                    crate::NearestScrollportOf::Absent,
                    Size::splat(crate::OrthogonalFallbackLimitOf::Unbounded),
                ),
            ),
        ),
    )
    .unwrap();
    let (output, report) = computation.into_parts();

    assert!(report.is_empty());
    assert_eq!(output.size, Size::new(120.0, 90.0));
    assert_eq!(
        tree.layouts[1].unwrap().output().size,
        Size::new(30.0, 20.0)
    );
}

fn assert_fri08_c01_area_only_pattern_topology<S: LayoutScalar>() {
    let px = |value| TrackComponentOf::Track(TrackSizingOf::px(S::from_f64(value)));
    let style = NodeInputOf::<S> {
        grid_auto_columns: vec![px(40.0), px(20.0)],
        grid_auto_rows: vec![px(5.0), px(7.0)],
        grid_template_areas: GridTemplateAreas {
            rows: vec![
                GridTemplateAreaRow {
                    cells: vec![
                        Some("main".to_string()),
                        Some("main".to_string()),
                        Some("main".to_string()),
                    ],
                },
                GridTemplateAreaRow {
                    cells: vec![
                        Some("main".to_string()),
                        Some("main".to_string()),
                        Some("main".to_string()),
                    ],
                },
                GridTemplateAreaRow {
                    cells: vec![
                        Some("main".to_string()),
                        Some("main".to_string()),
                        Some("main".to_string()),
                    ],
                },
            ],
        },
        ..NodeInputOf::default()
    };

    let topology = fri08_c01_topology_for_style(&style, None, None);

    assert_eq!(topology.explicit_columns, 3);
    assert_eq!(topology.explicit_rows, 3);
    assert_eq!(
        topology.column_tracks,
        vec![
            TrackSizingOf::px(S::from_f64(40.0)),
            TrackSizingOf::px(S::from_f64(20.0)),
            TrackSizingOf::px(S::from_f64(40.0)),
        ]
    );
    assert_eq!(
        topology.row_tracks,
        vec![
            TrackSizingOf::px(S::from_f64(5.0)),
            TrackSizingOf::px(S::from_f64(7.0)),
            TrackSizingOf::px(S::from_f64(5.0)),
        ]
    );
    assert_eq!(
        topology
            .column_origins
            .iter()
            .map(|origin| origin.sizing)
            .collect::<Vec<_>>(),
        vec![
            topology::ExplicitTrackSizingOrigin::TemplateAreaAutoPattern { pattern_index: 0 },
            topology::ExplicitTrackSizingOrigin::TemplateAreaAutoPattern { pattern_index: 1 },
            topology::ExplicitTrackSizingOrigin::TemplateAreaAutoPattern { pattern_index: 0 },
        ]
    );
    assert_eq!(
        topology
            .row_origins
            .iter()
            .map(|origin| origin.sizing)
            .collect::<Vec<_>>(),
        vec![
            topology::ExplicitTrackSizingOrigin::TemplateAreaAutoPattern { pattern_index: 0 },
            topology::ExplicitTrackSizingOrigin::TemplateAreaAutoPattern { pattern_index: 1 },
            topology::ExplicitTrackSizingOrigin::TemplateAreaAutoPattern { pattern_index: 0 },
        ]
    );
    assert_eq!(
        topology.named_columns.named_occurrences("main-start"),
        vec![1]
    );
    assert_eq!(
        topology.named_columns.named_occurrences("main-end"),
        vec![4]
    );
    assert_eq!(topology.named_rows.named_occurrences("main-start"), vec![1]);
    assert_eq!(topology.named_rows.named_occurrences("main-end"), vec![4]);
    assert!(topology.has_complete_origin_evidence());
}

#[test]
fn fri08_c01_topology_area_only_pattern_phase_is_axis_symmetric_in_both_scalar_lanes() {
    assert_fri08_c01_area_only_pattern_topology::<f32>();
    assert_fri08_c01_area_only_pattern_topology::<f64>();
}

#[test]
fn fri08_c01_topology_inherited_and_local_names_collide_as_one_membership() {
    let parent = named_parent_lines(1, &[&["a"], &[]]);
    let local = local_subgrid_entries(&[&["a"], &[]]);

    let lines = named::inherit_subgrid_named_lines(&parent, 1, 2, false, &local, None)
        .expect("valid inherited named span");

    assert_eq!(lines.named_occurrences("a"), vec![1]);
    assert_eq!(
        lines
            .entries_on_line(1)
            .iter()
            .map(|entry| entry.origin)
            .collect::<Vec<_>>(),
        vec![
            named::LineNameOrigin::Inherited,
            named::LineNameOrigin::LocalSubgrid,
        ]
    );
}

#[test]
fn named_lines_return_empty_local_map_for_subgrid() {
    let lines =
        named::named_lines_from_track_components(GridAxisKind::Row, &subgrid_track(), 2).unwrap();

    assert_eq!(lines.axis, GridAxisKind::Row);
    assert_eq!(lines.explicit_track_count, 2);
    assert!(lines.named_occurrences("anything").is_empty());
}

#[test]
fn subgrid_axis_placement_reports_one_authored_fallback_once() {
    let lines = named::NamedGridLines::new(GridAxisKind::Column, 2);
    for in_flow in [true, false] {
        let (placement, absolute, report) = resolve_grid_item_axis_placements_with_report(
            &lines,
            &RawGridPlacement::new(RawGridLine::Line(0), RawGridLine::Auto),
            GridPlacement::AUTO,
            true,
            in_flow,
            None,
        )
        .unwrap();

        assert_eq!(placement, GridPlacement::AUTO);
        assert_eq!(absolute, GridPlacement::AUTO);
        assert_eq!(
            report
                .errors()
                .iter()
                .filter(|error| **error == NamedGridErrorReport::ZeroLine)
                .count(),
            1
        );
    }
}

fn assert_subgrid_inherits_the_resolved_lanes_grid_axis<S: LayoutScalar>() {
    let px = S::from_f64;
    let lines = |start, end| GridPlacement::try_lines(start, end).unwrap();
    let mut differences = Vec::new();
    for axis in [GridAxisKind::Row, GridAxisKind::Column] {
        for writing_mode in [WritingMode::HorizontalTb, WritingMode::VerticalRl] {
            let parent_columns = axis == GridAxisKind::Column;
            let child_columns = parent_columns == (writing_mode == WritingMode::HorizontalTb);
            let tracks = || {
                [20.0, 30.0, 40.0]
                    .map(|size| TrackComponentOf::px(px(size)))
                    .into()
            };
            let child_tracks = |is_subgrid| {
                if is_subgrid {
                    vec![TrackComponentOf::Subgrid(crate::SubgridTrack::new(
                        Vec::new(),
                    ))]
                } else {
                    vec![TrackComponentOf::px(px(10.0))]
                }
            };
            let root = NodeInputOf {
                display: Display::GridLanes,
                grid_lanes_flow: crate::GridLanesFlow::new(axis, false, false),
                grid_template_columns: if parent_columns { tracks() } else { Vec::new() },
                grid_template_rows: if parent_columns { Vec::new() } else { tracks() },
                ..NodeInputOf::default()
            };
            let subgrid = NodeInputOf {
                display: Display::Grid,
                writing_mode,
                grid_column: if parent_columns {
                    lines(1, 4)
                } else {
                    GridPlacement::AUTO
                },
                grid_row: if parent_columns {
                    GridPlacement::AUTO
                } else {
                    lines(1, 4)
                },
                grid_template_columns: child_tracks(child_columns),
                grid_template_rows: child_tracks(!child_columns),
                ..NodeInputOf::default()
            };
            let descendant = NodeInputOf {
                grid_column: if child_columns {
                    lines(2, 3)
                } else {
                    lines(1, 2)
                },
                grid_row: if child_columns {
                    lines(1, 2)
                } else {
                    lines(2, 3)
                },
                ..NodeInputOf::default()
            };
            let tree = PublicLayoutTreeOf::new()
                .children(1, [2])
                .children(2, [3])
                .children(3, [])
                .style(1, root)
                .style(2, subgrid)
                .style(3, descendant);
            let batch =
                compute_layout(&tree, 1, super::fixtures::fri08_c01_placement_request()).unwrap();
            let subgrid = fri08_c01_placement_output(&batch, 2);
            let descendant = fri08_c01_placement_output(&batch, 3);
            let actual = if parent_columns {
                (
                    subgrid.size.width,
                    descendant.location.x,
                    descendant.size.width,
                )
            } else {
                (
                    subgrid.size.height,
                    descendant.location.y,
                    descendant.size.height,
                )
            };
            let expected = (px(90.0), px(20.0), px(30.0));
            if actual != expected {
                differences.push(format!(
                    "{axis:?} {writing_mode:?}: {actual:?}, expected {expected:?}"
                ));
            }
        }
    }
    assert!(differences.is_empty(), "{}", differences.join("\n"));
}

#[test]
fn subgrid_inherits_the_resolved_lanes_grid_axis_f32() {
    assert_subgrid_inherits_the_resolved_lanes_grid_axis::<f32>();
}

#[test]
fn subgrid_inherits_the_resolved_lanes_grid_axis_f64() {
    assert_subgrid_inherits_the_resolved_lanes_grid_axis::<f64>();
}

fn grid_lanes_subgrid_intrinsic_request<S: LayoutScalar>() -> LayoutRootRequestOf<S> {
    let axes = Size::splat(AxisSizingRequestOf::new(
        AvailableSpaceOf::Indefinite,
        SizingOperation::MaxContent,
    ));
    LayoutRootRequestOf::standalone(
        SizingRequestOf::new(
            axes,
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

fn assert_grid_lanes_subgrid_descendant_intrinsic_incidence<S: LayoutScalar>() {
    let px = S::from_f64;
    let request = grid_lanes_subgrid_intrinsic_request();
    let mut differences = Vec::new();
    for (wrapper_placement, expected) in [
        (GridPlacement::try_span(2).unwrap(), 60.0),
        (GridPlacement::try_lines(1, 3).unwrap(), 30.0),
    ] {
        for (named, nested, start) in [
            (false, false, 2),
            (true, false, 2),
            (false, true, 2),
            (false, false, 4),
        ] {
            let descendant = if named {
                NodeInputOf {
                    raw_grid_column: RawGridPlacement::new(
                        RawGridLine::NamedLine {
                            name: "middle".into(),
                            index: 1,
                        },
                        RawGridLine::Line(3),
                    ),
                    ..NodeInputOf::default()
                }
            } else {
                NodeInputOf {
                    grid_column: GridPlacement::try_lines(start, start + 1).unwrap(),
                    ..NodeInputOf::default()
                }
            };
            let tree = PublicLayoutTreeOf::new()
                .children(1, [2])
                .children(2, [3])
                .style(
                    1,
                    NodeInputOf {
                        display: Display::InlineGridLanes,
                        grid_template_columns: vec![TrackComponentOf::AUTO; 2],
                        ..NodeInputOf::default()
                    },
                )
                .style(
                    2,
                    NodeInputOf {
                        display: Display::Grid,
                        grid_column: wrapper_placement,
                        grid_template_columns: vec![TrackComponentOf::Subgrid(
                            crate::SubgridTrack::new(vec![
                                Vec::new(),
                                vec!["middle".into()],
                                Vec::new(),
                            ]),
                        )],
                        ..NodeInputOf::default()
                    },
                );
            let leaf = NodeInputOf {
                size: Size::new(px(30.0), px(10.0)).map(PreferredSizeOf::px),
                ..descendant
            };
            let tree = if nested {
                tree.children(3, [4])
                    .children(4, [])
                    .style(
                        3,
                        NodeInputOf {
                            display: Display::Grid,
                            grid_column: GridPlacement::try_lines(1, 3).unwrap(),
                            grid_template_columns: vec![TrackComponentOf::Subgrid(
                                crate::SubgridTrack::new(Vec::new()),
                            )],
                            ..NodeInputOf::default()
                        },
                    )
                    .style(4, leaf)
            } else {
                tree.children(3, []).style(3, leaf)
            };
            let batch = compute_layout(&tree, 1, request).unwrap();
            let actual = fri08_c01_placement_output(&batch, 1).size.width;
            if actual != px(expected) {
                differences.push(format!(
                    "{wrapper_placement:?}, named {named}, nested {nested}, line {start}: {actual:?}, expected {expected}"
                ));
            }
        }
    }
    for (direction, start, expected) in [
        (Direction::Ltr, 1, 120.0),
        (Direction::Ltr, 2, 90.0),
        (Direction::Ltr, 3, 150.0),
        (Direction::Ltr, 5, 150.0),
        (Direction::Rtl, 1, 150.0),
        (Direction::Rtl, 2, 90.0),
        (Direction::Rtl, 3, 120.0),
    ] {
        let tree = PublicLayoutTreeOf::new()
            .children(1, [2])
            .children(2, [3])
            .children(3, [])
            .style(
                1,
                NodeInputOf {
                    display: Display::InlineGridLanes,
                    grid_template_columns: vec![TrackComponentOf::AUTO; 3],
                    ..NodeInputOf::default()
                },
            )
            .style(
                2,
                NodeInputOf {
                    display: Display::Grid,
                    direction,
                    grid_column: GridPlacement::try_span(3).unwrap(),
                    grid_template_columns: vec![TrackComponentOf::Subgrid(
                        crate::SubgridTrack::new(Vec::new()),
                    )],
                    padding: Edges::new(
                        LengthOf::ZERO,
                        LengthOf::px(px(20.0)),
                        LengthOf::ZERO,
                        LengthOf::px(px(10.0)),
                    ),
                    ..NodeInputOf::default()
                },
            )
            .style(
                3,
                NodeInputOf {
                    grid_column: GridPlacement::try_lines(start, start + 1).unwrap(),
                    size: Size::new(px(30.0), px(10.0)).map(PreferredSizeOf::px),
                    ..NodeInputOf::default()
                },
            );
        let batch = compute_layout(&tree, 1, request).unwrap();
        let actual = fri08_c01_placement_output(&batch, 1).size.width;
        if actual != px(expected) {
            differences.push(format!(
                "{direction:?} explicit descendant track {start} with wrapper padding: {actual:?}, expected {expected}"
            ));
        }
    }
    let named_nested = PublicLayoutTreeOf::new()
        .children(1, [2])
        .children(2, [3])
        .children(3, [4])
        .children(4, [])
        .style(
            1,
            NodeInputOf {
                display: Display::InlineGridLanes,
                grid_template_columns: vec![TrackComponentOf::AUTO; 3],
                ..NodeInputOf::default()
            },
        )
        .style(
            2,
            NodeInputOf {
                display: Display::Grid,
                grid_column: GridPlacement::try_span(3).unwrap(),
                grid_template_columns: vec![TrackComponentOf::Subgrid(crate::SubgridTrack::new(
                    vec![Vec::new(), vec!["middle".into()], Vec::new(), Vec::new()],
                ))],
                ..NodeInputOf::default()
            },
        )
        .style(
            3,
            NodeInputOf {
                display: Display::Grid,
                grid_column: GridPlacement::try_lines(1, 3).unwrap(),
                grid_template_columns: vec![TrackComponentOf::Subgrid(crate::SubgridTrack::new(
                    Vec::new(),
                ))],
                padding: Edges::new(
                    LengthOf::ZERO,
                    LengthOf::px(px(20.0)),
                    LengthOf::ZERO,
                    LengthOf::px(px(10.0)),
                ),
                ..NodeInputOf::default()
            },
        )
        .style(
            4,
            NodeInputOf {
                raw_grid_column: RawGridPlacement::new(
                    RawGridLine::NamedLine {
                        name: "middle".into(),
                        index: 1,
                    },
                    RawGridLine::Span(1),
                ),
                size: Size::new(px(30.0), px(10.0)).map(PreferredSizeOf::px),
                ..NodeInputOf::default()
            },
        );
    let batch = compute_layout(&named_nested, 1, request).unwrap();
    let actual = fri08_c01_placement_output(&batch, 1).size.width;
    if actual != px(150.0) {
        differences.push(format!(
            "nested named edge keeps its actual end: {actual:?}, expected 150"
        ));
    }
    assert!(differences.is_empty(), "{}", differences.join("\n"));
}

#[test]
fn grid_lanes_subgrid_descendant_intrinsic_incidence_f32() {
    assert_grid_lanes_subgrid_descendant_intrinsic_incidence::<f32>();
}

#[test]
fn grid_lanes_subgrid_descendant_intrinsic_incidence_f64() {
    assert_grid_lanes_subgrid_descendant_intrinsic_incidence::<f64>();
}

fn assert_grid_lanes_nested_descendant_edge_scope<S: LayoutScalar>() {
    let px = S::from_f64;
    let request = grid_lanes_subgrid_intrinsic_request();
    let mut differences = Vec::new();
    for (direction, start, expected) in [
        (Direction::Ltr, 1, 160.0),
        (Direction::Ltr, 2, 120.0),
        (Direction::Rtl, 1, 120.0),
        (Direction::Rtl, 3, 160.0),
    ] {
        let tree = PublicLayoutTreeOf::new()
            .children(1, [2])
            .children(2, [3])
            .children(3, [4])
            .children(4, [])
            .style(
                1,
                NodeInputOf {
                    display: Display::InlineGridLanes,
                    grid_template_columns: vec![TrackComponentOf::AUTO; 4],
                    ..NodeInputOf::default()
                },
            )
            .style(
                2,
                NodeInputOf {
                    display: Display::Grid,
                    grid_column: GridPlacement::try_span(4).unwrap(),
                    grid_template_columns: vec![TrackComponentOf::Subgrid(
                        crate::SubgridTrack::new(Vec::new()),
                    )],
                    padding: Edges::new(
                        LengthOf::ZERO,
                        LengthOf::ZERO,
                        LengthOf::ZERO,
                        LengthOf::px(px(10.0)),
                    ),
                    ..NodeInputOf::default()
                },
            )
            .style(
                3,
                NodeInputOf {
                    display: Display::Grid,
                    direction,
                    grid_column: GridPlacement::try_lines(1, 4).unwrap(),
                    grid_template_columns: vec![TrackComponentOf::Subgrid(
                        crate::SubgridTrack::new(Vec::new()),
                    )],
                    ..NodeInputOf::default()
                },
            )
            .style(
                4,
                NodeInputOf {
                    grid_column: GridPlacement::try_lines(start, start + 1).unwrap(),
                    size: Size::new(px(30.0), px(10.0)).map(PreferredSizeOf::px),
                    ..NodeInputOf::default()
                },
            );
        let batch = compute_layout(&tree, 1, request).unwrap();
        let actual = fri08_c01_placement_output(&batch, 1).size.width;
        if actual != px(expected) {
            differences.push(format!(
                "{direction:?} nested local line {start}: {actual:?}, expected {expected}"
            ));
        }
    }
    let named = PublicLayoutTreeOf::new()
        .children(1, [2])
        .children(2, [3, 4])
        .children(3, [])
        .children(4, [5])
        .children(5, [])
        .style(
            1,
            NodeInputOf {
                display: Display::InlineGridLanes,
                grid_template_columns: vec![TrackComponentOf::AUTO; 3],
                ..NodeInputOf::default()
            },
        )
        .style(
            2,
            NodeInputOf {
                display: Display::Grid,
                grid_auto_flow: GridAutoFlow::Column,
                grid_column: GridPlacement::try_span(3).unwrap(),
                grid_template_columns: vec![TrackComponentOf::Subgrid(crate::SubgridTrack::new(
                    Vec::new(),
                ))],
                grid_template_rows: vec![TrackComponentOf::px(px(10.0))],
                ..NodeInputOf::default()
            },
        )
        .style(
            3,
            NodeInputOf {
                grid_column: GridPlacement::try_lines(1, 3).unwrap(),
                grid_row: GridPlacement::try_line(1).unwrap(),
                size: Size::ZERO.map(PreferredSizeOf::px),
                ..NodeInputOf::default()
            },
        )
        .style(
            4,
            NodeInputOf {
                display: Display::Grid,
                grid_column: GridPlacement::try_span(2).unwrap(),
                grid_row: GridPlacement::try_line(1).unwrap(),
                grid_template_columns: vec![TrackComponentOf::Subgrid(crate::SubgridTrack::new(
                    vec![vec!["first".into()]],
                ))],
                padding: Edges::new(
                    LengthOf::ZERO,
                    LengthOf::px(px(20.0)),
                    LengthOf::ZERO,
                    LengthOf::px(px(10.0)),
                ),
                ..NodeInputOf::default()
            },
        )
        .style(
            5,
            NodeInputOf {
                raw_grid_column: RawGridPlacement::new(
                    RawGridLine::NamedLine {
                        name: "first".into(),
                        index: 1,
                    },
                    RawGridLine::Span(1),
                ),
                size: Size::new(px(30.0), px(10.0)).map(PreferredSizeOf::px),
                ..NodeInputOf::default()
            },
        );
    let typed = named.clone().style(
        5,
        NodeInputOf {
            grid_column: GridPlacement::try_lines(1, 2).unwrap(),
            size: Size::new(px(30.0), px(10.0)).map(PreferredSizeOf::px),
            ..NodeInputOf::default()
        },
    );
    let [named_width, typed_width] = [&named, &typed].map(|tree| {
        let batch = compute_layout(tree, 1, request).unwrap();
        fri08_c01_placement_output(&batch, 1).size.width
    });
    if named_width != typed_width {
        differences.push(format!(
            "automatic nested first line: named {named_width:?}, typed {typed_width:?}"
        ));
    }
    assert!(differences.is_empty(), "{}", differences.join("\n"));
}

#[test]
fn grid_lanes_subgrid_descendant_intrinsic_incidence_ancestor_edges_f32() {
    assert_grid_lanes_nested_descendant_edge_scope::<f32>();
}

#[test]
fn grid_lanes_subgrid_descendant_intrinsic_incidence_ancestor_edges_f64() {
    assert_grid_lanes_nested_descendant_edge_scope::<f64>();
}

fn assert_grid_lanes_nested_intrinsic_receipts() {
    const TRACKS: usize = 7;
    let outer_placement = GridPlacement::try_span(TRACKS).unwrap();
    let mut tree = OracleTree::new().children(0, [1]).style(
        0,
        NodeInput {
            display: Display::GridLanes,
            grid_template_columns: vec![TrackComponent::AUTO; TRACKS],
            ..NodeInput::default()
        },
    );
    for (node, span) in (1..).zip((1..=TRACKS).rev()) {
        let placement = if node == 1 {
            outer_placement
        } else {
            GridPlacement::try_lines(1, isize::try_from(span).unwrap() + 1).unwrap()
        };
        tree = tree.children(node, [node + 1]).style(
            node,
            NodeInput {
                display: Display::Grid,
                grid_column: placement,
                grid_template_columns: vec![empty_subgrid_track()],
                ..NodeInput::default()
            },
        );
    }
    let leaf = u32::try_from(TRACKS).unwrap() + 1;
    tree = tree.children(leaf, []).style(
        leaf,
        NodeInput {
            size: Size::new(30.0, 10.0).map(PreferredSize::px),
            ..NodeInput::default()
        },
    );
    let mut compute = tree.borrow_compute();
    let placements = GridPlacementContext::new_with_child_inputs(
        vec![1],
        vec![ResolvedGridItemPlacement {
            column: outer_placement,
            row: GridPlacement::AUTO,
            absolute_column: outer_placement,
            absolute_row: GridPlacement::AUTO,
            in_flow: true,
        }],
        vec![grid_child_input!(compute.node_input(1))],
    );
    let container = grid_container_projection!(compute.node_input(0));
    let constants = Constants {
        child_orthogonal_fallback: OrthogonalFallbackOf::new(
            Size::NONE,
            crate::NearestScrollportOf::Absent,
            Size::splat(crate::OrthogonalFallbackLimitOf::Unknown),
        ),
        flow_axes: FlowAxes::new(WritingMode::HorizontalTb, Direction::Ltr),
        explicit_definite_content_size: Size::NONE,
        indefinite_percentage_axis: None,
        node_outer_size: Size::NONE,
        node_inner_size: Size::NONE,
        node_min_size: Size::NONE,
        node_definite_min_size: Size::NONE,
        fallback_minimum_outer: Size::splat(FixedFallbackSource::Unqualified),
        fallback_maximum_outer: Size::splat(FixedFallbackSource::Unqualified),
        node_max_size: Size::NONE,
        available_inner_size: Size::NONE,
        content_box_inset: Edges::ZERO,
        scrollbar_gutter: Edges::ZERO,
        padding: Edges::ZERO,
        border: Edges::ZERO,
    };
    let tracks = [TrackSizing::AUTO; TRACKS];
    let lines = GridLines {
        column_explicit_start: 0,
        column_explicit_count: TRACKS,
        row_explicit_start: 0,
        row_explicit_count: 0,
    };
    let named_columns = named::NamedGridLines::new(GridAxisKind::Column, TRACKS);
    let named_rows = named::NamedGridLines::new(GridAxisKind::Row, 0);
    let subgrid_report = collect_subgrid_report(&container, &placements);
    let grid = IntrinsicGrid {
        style: &container,
        constants: &constants,
        sizing_flow_axes: constants.flow_axes,
        track_basis: LogicalSizeOf::new(None, None),
        minimum_functions: GridMinimumParentFunctions {
            columns: Some(GridMinimumTrackFunctions::new(&tracks, None)),
            rows: Some(GridMinimumTrackFunctions::new(&[], None)),
        },
        column_tracks: &tracks,
        row_tracks: &[],
        gap: LogicalSizeOf::new(0.0, 0.0),
        column_gutters: None,
        row_gutters: None,
        lines,
        named_columns: &named_columns,
        named_rows: &named_rows,
        area_facts: None,
        subgrid_report: &subgrid_report,
        placements: &placements,
        query: IntrinsicGridQuery::complete(GridContributionPurpose::Minimum),
    };
    let contributions = lane_intrinsic_contributions(
        &mut compute,
        0,
        LaneIntrinsicTrackSizeInput {
            grid,
            frame: intrinsic_stage::IntrinsicStageFrame {
                axis: GridAxisKind::Column,
                stage: GridSizingStage {
                    first_axis: GridAxisKind::Column,
                    through: GridSizingPhase::InitialFirst,
                },
                columns: None,
                rows: None,
                observed_axis: None,
            },
            container_style: &container,
            constants: &constants,
            axis: GridAxisKind::Column,
            tracks: &tracks,
            gap: 0.0,
            gutters: None,
            lines,
            placements: &placements,
            subgrid_report: &subgrid_report,
        },
        None,
        true,
    )
    .unwrap()
    .unwrap();
    assert_eq!(contributions.rows.len(), TRACKS);
    assert_eq!(
        contributions.min_content.rows.len(),
        TRACKS,
        "one contribution for each distinct leaf span",
    );
}

#[test]
fn grid_lanes_subgrid_descendant_intrinsic_incidence_coalesces_nested_visits() {
    std::thread::Builder::new()
        .stack_size(16 * 1024 * 1024)
        .spawn(assert_grid_lanes_nested_intrinsic_receipts)
        .unwrap()
        .join()
        .unwrap();
}

fn lane_stacking_probe_tree<S: LayoutScalar>(
    column_tracks: bool,
    inherited: bool,
    nested: NodeInputOf<S>,
) -> PublicLayoutTreeOf<S> {
    let px = S::from_f64;
    let fixed = || {
        vec![
            TrackComponentOf::px(px(20.0)),
            TrackComponentOf::px(px(30.0)),
        ]
    };
    let nested_tracks = if inherited {
        vec![TrackComponentOf::Subgrid(SubgridTrack::new(Vec::new()))]
    } else {
        fixed()
    };
    let mut tree = PublicLayoutTreeOf::new()
        .children(1, [2, 3])
        .children(2, [4, 5])
        .style(
            1,
            NodeInputOf {
                display: Display::InlineGridLanes,
                grid_lanes_flow: crate::GridLanesFlow::new(
                    if column_tracks {
                        crate::GridAxisKind::Column
                    } else {
                        crate::GridAxisKind::Row
                    },
                    false,
                    false,
                ),
                grid_template_columns: if column_tracks { fixed() } else { Vec::new() },
                grid_template_rows: if column_tracks { Vec::new() } else { fixed() },
                align_items: Some(AlignItems::Start),
                justify_items: Some(AlignItems::Start),
                ..NodeInputOf::default()
            },
        )
        .style(
            2,
            NodeInputOf {
                display: Display::GridLanes,
                grid_lanes_flow: crate::GridLanesFlow::new(
                    if column_tracks {
                        crate::GridAxisKind::Column
                    } else {
                        crate::GridAxisKind::Row
                    },
                    false,
                    false,
                ),
                grid_column: if column_tracks {
                    GridPlacement::try_span(2).unwrap()
                } else {
                    GridPlacement::AUTO
                },
                grid_row: if column_tracks {
                    GridPlacement::AUTO
                } else {
                    GridPlacement::try_span(2).unwrap()
                },
                grid_template_columns: if column_tracks {
                    nested_tracks.clone()
                } else {
                    Vec::new()
                },
                grid_template_rows: if column_tracks {
                    Vec::new()
                } else {
                    nested_tracks
                },
                ..nested
            },
        )
        .style(
            3,
            NodeInputOf {
                size: Size::new(px(5.0), px(5.0)).map(PreferredSizeOf::px),
                ..NodeInputOf::default()
            },
        );
    for child in [4, 5] {
        tree = tree.style(
            child,
            NodeInputOf {
                size: if column_tracks {
                    Size::new(px(20.0), px(10.0)).map(PreferredSizeOf::px)
                } else {
                    Size::new(px(10.0), px(20.0)).map(PreferredSizeOf::px)
                },
                ..NodeInputOf::default()
            },
        );
    }
    tree
}

fn lane_stacking_probe_geometry<S: LayoutScalar>(
    tree: &PublicLayoutTreeOf<S>,
    column_tracks: bool,
) -> (S, S, S) {
    let batch = compute_layout(tree, 1, grid_lanes_subgrid_intrinsic_request()).unwrap();
    let nested = fri08_c01_placement_output(&batch, 2);
    let follower = fri08_c01_placement_output(&batch, 3);
    let root = fri08_c01_placement_output(&batch, 1);
    let inherited_extent = if column_tracks {
        nested.size.width
    } else {
        nested.size.height
    };
    assert_eq!(
        inherited_extent,
        S::from_f64(50.0),
        "inherited border-box extent"
    );
    if column_tracks {
        (nested.size.height, follower.location.y, root.size.height)
    } else {
        (nested.size.width, follower.location.x, root.size.width)
    }
}

fn assert_grid_lanes_stacking_probe_retains_inherited_tracks<S: LayoutScalar>() {
    let px = S::from_f64;
    let mut differences = Vec::new();
    for column_tracks in [true, false] {
        for (inherited, authored_extent, has_ratio) in [
            (false, None, false),
            (true, None, false),
            (true, Some(1.0), false),
            (true, Some(100.0), false),
            (true, None, true),
            (true, Some(1.0), true),
            (true, Some(100.0), true),
        ] {
            for empty in [false, true] {
                let authored = authored_extent.map_or(PreferredSizeOf::AUTO, |value| {
                    PreferredSizeOf::px(px(value))
                });
                let mut tree = lane_stacking_probe_tree(
                    column_tracks,
                    inherited,
                    NodeInputOf {
                        size: if column_tracks {
                            Size::new(authored, PreferredSizeOf::AUTO)
                        } else {
                            Size::new(PreferredSizeOf::AUTO, authored)
                        },
                        aspect_ratio: has_ratio
                            .then(|| crate::AspectRatioOf::new(px(2.0)).unwrap()),
                        ..NodeInputOf::default()
                    },
                );
                if empty {
                    tree = tree.children(2, []);
                }
                let actual = lane_stacking_probe_geometry(&tree, column_tracks);
                let extent = if has_ratio {
                    if column_tracks { 25.0 } else { 100.0 }
                } else if empty {
                    0.0
                } else {
                    10.0
                };
                let expected = (px(extent), px(extent), px(extent + 5.0));
                if actual != expected {
                    differences.push(format!(
                        "columns {column_tracks}, inherited {inherited}, authored {authored_extent:?}, ratio {has_ratio}, empty {empty}: {actual:?}, expected {expected:?}",
                    ));
                }
            }
        }
    }
    assert!(differences.is_empty(), "{}", differences.join("\n"));
}

#[test]
fn grid_lanes_stacking_probe_retains_inherited_tracks_f32() {
    assert_grid_lanes_stacking_probe_retains_inherited_tracks::<f32>();
}

#[test]
fn grid_lanes_stacking_probe_retains_inherited_tracks_f64() {
    assert_grid_lanes_stacking_probe_retains_inherited_tracks::<f64>();
}

fn assert_grid_lanes_inherited_ratio_box_and_constraints<S: LayoutScalar>() {
    let px = S::from_f64;
    for column_tracks in [true, false] {
        for box_sizing in [BoxSizing::ContentBox, BoxSizing::BorderBox] {
            for insets in [false, true] {
                for ignored_bounds in [false, true] {
                    let inherited_size = PreferredSizeOf::px(px(1.0));
                    let mut nested = NodeInputOf {
                        size: if column_tracks {
                            Size::new(inherited_size, PreferredSizeOf::AUTO)
                        } else {
                            Size::new(PreferredSizeOf::AUTO, inherited_size)
                        },
                        aspect_ratio: crate::AspectRatioOf::new(px(2.0)),
                        box_sizing,
                        padding: if insets {
                            Edges::new(
                                LengthOf::px(px(2.0)),
                                LengthOf::px(px(5.0)),
                                LengthOf::px(px(2.0)),
                                LengthOf::px(px(5.0)),
                            )
                        } else {
                            Edges::ZERO.map(LengthOf::px)
                        },
                        ..NodeInputOf::default()
                    };
                    if ignored_bounds {
                        let minimum = MinSizeOf::px(px(200.0));
                        let maximum = crate::MaxSizeOf::px(px(1.0));
                        if column_tracks {
                            nested.min_size.width = minimum;
                            nested.max_size.width = maximum;
                        } else {
                            nested.min_size.height = minimum;
                            nested.max_size.height = maximum;
                        }
                    }
                    let tree = lane_stacking_probe_tree(column_tracks, true, nested);
                    let actual = lane_stacking_probe_geometry(&tree, column_tracks);
                    let extent = if box_sizing == BoxSizing::ContentBox && insets {
                        if column_tracks { 24.0 } else { 102.0 }
                    } else if column_tracks {
                        25.0
                    } else {
                        100.0
                    };
                    assert_eq!(
                        actual,
                        (px(extent), px(extent), px(extent + 5.0)),
                        "columns {column_tracks}, box {box_sizing:?}, insets {insets}, ignored bounds {ignored_bounds}"
                    );
                }
            }
        }
        for (minimum, maximum, preferred, expected) in if column_tracks {
            [
                (Some(40.0), None, None, 40.0),
                (None, Some(15.0), None, 15.0),
                (None, None, Some(30.0), 30.0),
            ]
        } else {
            [
                (Some(120.0), None, None, 120.0),
                (None, Some(80.0), None, 80.0),
                (None, None, Some(120.0), 120.0),
            ]
        } {
            let mut nested = NodeInputOf {
                aspect_ratio: crate::AspectRatioOf::new(px(2.0)),
                ..NodeInputOf::default()
            };
            let minimum = minimum.map_or(MinSizeOf::AUTO, |value| MinSizeOf::px(px(value)));
            let maximum = maximum.map_or(crate::MaxSizeOf::NONE, |value| {
                crate::MaxSizeOf::px(px(value))
            });
            let preferred = preferred.map_or(PreferredSizeOf::AUTO, |value| {
                PreferredSizeOf::px(px(value))
            });
            if column_tracks {
                nested.min_size.height = minimum;
                nested.max_size.height = maximum;
                nested.size.height = preferred;
            } else {
                nested.min_size.width = minimum;
                nested.max_size.width = maximum;
                nested.size.width = preferred;
            }
            let tree = lane_stacking_probe_tree(column_tracks, true, nested);
            assert_eq!(
                lane_stacking_probe_geometry(&tree, column_tracks),
                (px(expected), px(expected), px(expected + 5.0)),
                "columns {column_tracks}, standalone constraints"
            );
        }
    }
}

#[test]
fn grid_lanes_inherited_ratio_box_and_constraints_f32() {
    assert_grid_lanes_inherited_ratio_box_and_constraints::<f32>();
}

#[test]
fn grid_lanes_inherited_ratio_box_and_constraints_f64() {
    assert_grid_lanes_inherited_ratio_box_and_constraints::<f64>();
}

fn assert_grid_lanes_wrapper_intrinsic_candidates_in_grid<S: LayoutScalar>() {
    let px = S::from_f64;
    let auto = GridPlacement::AUTO;
    let line = |index| GridPlacement::try_line(index).unwrap();
    let mut differences = Vec::new();
    for (label, wrapper_column, leaf_column, start_padding, nested, expected) in [
        (
            "automatic wrapper and leaf",
            auto,
            auto,
            0.0,
            false,
            [25.0, 25.0],
        ),
        ("definite wrapper", line(2), auto, 0.0, false, [0.0, 25.0]),
        ("definite leaf", auto, line(1), 0.0, false, [25.0, 0.0]),
        (
            "padded automatic leaf",
            auto,
            auto,
            10.0,
            false,
            [35.0, 35.0],
        ),
        (
            "padded definite leaf",
            auto,
            line(1),
            10.0,
            false,
            [35.0, 0.0],
        ),
        (
            "nested automatic wrapper",
            auto,
            auto,
            0.0,
            true,
            [25.0, 25.0],
        ),
        (
            "nested definite leaf",
            auto,
            line(1),
            10.0,
            true,
            [35.0, 0.0],
        ),
    ] {
        let mut tree = PublicLayoutTreeOf::new()
            .children(1, [2, 4, 5])
            .children(2, [3])
            .style(
                1,
                NodeInputOf {
                    display: Display::InlineGrid,
                    grid_template_columns: vec![TrackComponentOf::AUTO; 2],
                    grid_template_rows: vec![TrackComponentOf::AUTO; 2],
                    ..NodeInputOf::default()
                },
            )
            .style(
                2,
                NodeInputOf {
                    display: Display::GridLanes,
                    grid_column: wrapper_column,
                    grid_row: line(1),
                    padding: Edges::new(
                        LengthOf::ZERO,
                        LengthOf::ZERO,
                        LengthOf::ZERO,
                        LengthOf::px(px(start_padding)),
                    ),
                    grid_template_columns: vec![TrackComponentOf::Subgrid(SubgridTrack::new(
                        Vec::new(),
                    ))],
                    ..NodeInputOf::default()
                },
            )
            .style(
                3,
                NodeInputOf {
                    grid_column: leaf_column,
                    size: Size::new(px(25.0), px(10.0)).map(PreferredSizeOf::px),
                    ..NodeInputOf::default()
                },
            );
        if nested {
            tree = tree.children(1, [6, 4, 5]).children(6, [2]).style(
                6,
                NodeInputOf {
                    display: Display::Grid,
                    grid_column: GridPlacement::try_lines(1, 3).unwrap(),
                    grid_row: line(1),
                    grid_template_columns: vec![TrackComponentOf::Subgrid(SubgridTrack::new(
                        Vec::new(),
                    ))],
                    ..NodeInputOf::default()
                },
            );
        }
        for (probe, column) in [(4, 1), (5, 2)] {
            tree = tree
                .style(
                    probe,
                    NodeInputOf {
                        grid_column: line(column),
                        grid_row: line(2),
                        min_size: Size::ZERO.map(MinSizeOf::px),
                        ..NodeInputOf::default()
                    },
                )
                .measure(probe, Size::ZERO);
        }
        match compute_layout(&tree, 1, grid_lanes_subgrid_intrinsic_request()) {
            Ok(batch) => {
                let tracks = [4, 5].map(|node| fri08_c01_placement_output(&batch, node).size.width);
                if tracks != expected.map(px) {
                    differences.push(format!("{label}: {tracks:?}, expected {expected:?}"));
                }
            }
            Err(error) => differences.push(format!("{label}: {error:?}")),
        }
    }
    assert!(differences.is_empty(), "{}", differences.join("\n"));
}

#[test]
fn grid_lanes_wrapper_intrinsic_candidates_in_grid_f32() {
    assert_grid_lanes_wrapper_intrinsic_candidates_in_grid::<f32>();
}

#[test]
fn grid_lanes_wrapper_intrinsic_candidates_in_grid_f64() {
    assert_grid_lanes_wrapper_intrinsic_candidates_in_grid::<f64>();
}

#[test]
fn grid_lanes_wrapper_candidates_retain_distinct_leaf_frames() {
    const TRACKS: usize = 64;
    for nested in [false, true] {
        for (start_padding, end_padding) in [(0.0, 0.0), (10.0, 20.0), (10.0, 10.0)] {
            let axis = GridAxisKind::Column;
            let mut tree = OracleTree::new()
                .children(1, [2])
                .children(2, [3])
                .children(3, [])
                .style(
                    1,
                    NodeInput {
                        display: Display::InlineGrid,
                        grid_template_columns: vec![TrackComponent::AUTO; TRACKS],
                        grid_template_rows: vec![TrackComponent::px(40.0)],
                        ..NodeInput::default()
                    },
                )
                .style(
                    2,
                    NodeInput {
                        display: Display::GridLanes,
                        grid_column: GridPlacement::try_span(TRACKS / 2).unwrap(),
                        grid_template_columns: subgrid_track(),
                        padding: Edges::new(
                            Length::ZERO,
                            Length::px(end_padding),
                            Length::ZERO,
                            Length::px(start_padding),
                        ),
                        ..NodeInput::default()
                    },
                )
                .style(
                    3,
                    NodeInput {
                        size: Size::new(PreferredSize::px(25.0), PreferredSize::px(10.0)),
                        ..NodeInput::default()
                    },
                );
            if nested {
                tree = tree.children(1, [4]).children(4, [2]).style(
                    4,
                    NodeInput {
                        display: Display::Grid,
                        grid_column: GridPlacement::try_lines(1, (TRACKS + 1) as isize).unwrap(),
                        grid_template_columns: subgrid_track(),
                        ..NodeInput::default()
                    },
                );
            }
            let mut compute = tree.borrow_compute();
            let style = grid_container_projection!(compute.node_input(1));
            let flow = FlowAxes::new(style.writing_mode, style.direction);
            let request = ComputeInput::for_child(
                RunMode::ComputeSize,
                SizingMode::InherentSize,
                RequestedAxis::Both,
                Size::NONE,
                Size::NONE,
                ContainingLayoutContext::new(flow, ParentFormattingContext::NoParent),
                SizingRequestOf::new(
                    Size::splat(AxisSizingRequestOf::new(
                        AvailableSpaceOf::Indefinite,
                        SizingOperation::MaxContent,
                    )),
                    OrthogonalFallbackOf::new(
                        Size::NONE,
                        crate::NearestScrollportOf::Absent,
                        Size::splat(crate::OrthogonalFallbackLimitOf::Unbounded),
                    ),
                ),
            );
            let constants =
                Constants::new::<_, core::convert::Infallible>(&compute, 1, &style, request)
                    .unwrap();
            let parent_context = GridParentContext::none();
            let initialized = initialize_grid_tracks::<_, core::convert::Infallible>(
                &mut compute,
                1,
                &style,
                &constants,
                &parent_context,
                request,
                None,
            )
            .unwrap();
            let context = &initialized.context;
            let columns = [0.0; TRACKS];
            let rows = [40.0];
            let children = compute.children(1).collect::<Vec<_>>();
            let placed_areas = resolve_grid_child_areas(ResolveGridChildAreasInput {
                children: &children,
                placements: &initialized.placements,
                style: &style,
                columns: &columns,
                rows: &rows,
                gap: context.gap,
                lines: context.lines,
            });
            let query = IntrinsicGridQuery {
                axis: Some(axis),
                ..IntrinsicGridQuery::complete(GridContributionPurpose::MaxContent)
            };
            let grid = IntrinsicGrid {
                style: &style,
                constants: &constants,
                sizing_flow_axes: flow,
                track_basis: context.percent_basis,
                minimum_functions: minimum_parent_functions(
                    &initialized.column_tracks,
                    &initialized.row_tracks,
                    context.percent_basis,
                    &parent_context,
                ),
                column_tracks: &initialized.column_tracks,
                row_tracks: &initialized.row_tracks,
                gap: context.gap,
                column_gutters: Some(&context.column_gutters),
                row_gutters: Some(&context.row_gutters),
                lines: context.lines,
                named_columns: &context.named_columns,
                named_rows: &context.named_rows,
                area_facts: context.area_facts.as_ref(),
                subgrid_report: &initialized.subgrid_report,
                placements: &initialized.placements,
                query,
            };
            let report = collect_grid_subgrid_intrinsic_traversal(
                &mut compute,
                GridSubgridIntrinsicTraversalInput {
                    grid,
                    axis,
                    containing_flow_axes: flow,
                    children: &children,
                    placed_areas: &placed_areas,
                    subgrid_report: &initialized.subgrid_report,
                    placements: &initialized.placements,
                    named_columns: &context.named_columns,
                    named_rows: &context.named_rows,
                    area_facts: context.area_facts.as_ref(),
                    parent_gap: Size::new(context.gap.inline, context.gap.block),
                    column_gutters: Some(&context.column_gutters),
                    row_gutters: Some(&context.row_gutters),
                    column_sizes: &columns,
                    row_sizes: &rows,
                    container_size: constants.node_inner_size,
                    intrinsic_min_track_facts: IntrinsicMinTrackFacts::Known(&[true; TRACKS]),
                    query,
                },
            )
            .unwrap()
            .unwrap();
            let mut adjustment_path = Vec::new();
            let mut observed = report
                .leaves
                .iter()
                .map(|leaf| {
                    assert_eq!(leaf.node, 3);
                    assert_eq!(leaf.ancestor_span.end, leaf.ancestor_span.start + 1);
                    let (scalar, baseline) = report
                        .adjustments
                        .for_leaf(leaf, &mut adjustment_path)
                        .unwrap();
                    (
                        leaf.ancestor_span.start,
                        scalar,
                        baseline.start,
                        baseline.end,
                    )
                })
                .collect::<Vec<_>>();
            let mut expected = Vec::new();
            for wrapper_start in 0..=TRACKS / 2 {
                for local in 0..TRACKS / 2 {
                    let start = if local == 0 { start_padding } else { 0.0 };
                    let end = if local + 1 == TRACKS / 2 {
                        end_padding
                    } else {
                        0.0
                    };
                    expected.push((wrapper_start + local + 1, start + end, start, end));
                }
            }
            let order = |left: &(usize, f32, f32, f32), right: &(usize, f32, f32, f32)| {
                left.0
                    .cmp(&right.0)
                    .then_with(|| left.1.total_cmp(&right.1))
                    .then_with(|| left.2.total_cmp(&right.2))
                    .then_with(|| left.3.total_cmp(&right.3))
            };
            observed.sort_unstable_by(order);
            expected.sort_unstable_by(order);
            expected.dedup();
            assert_eq!(
                observed, expected,
                "distinct edge and baseline frames, nested {nested}, padding {start_padding}/{end_padding}",
            );
            assert!(report.leaves.len() <= 3 * TRACKS);
        }
    }
}

fn assert_grid_lanes_empty_subgrid_edge_floors<S: LayoutScalar>() {
    let px = S::from_f64;
    let inherited = || vec![TrackComponentOf::Subgrid(SubgridTrack::new(Vec::new()))];
    for (padding, expected_width) in [(0.0, 0.0), (10.0, 30.0)] {
        let tree = PublicLayoutTreeOf::new()
            .children(1, [2])
            .children(2, [3])
            .children(3, [])
            .style(
                1,
                NodeInputOf {
                    display: Display::InlineGridLanes,
                    grid_template_columns: vec![TrackComponentOf::AUTO; 3],
                    ..NodeInputOf::default()
                },
            )
            .style(
                2,
                NodeInputOf {
                    display: Display::Grid,
                    grid_column: GridPlacement::try_span(3).unwrap(),
                    grid_template_columns: inherited(),
                    ..NodeInputOf::default()
                },
            )
            .style(
                3,
                NodeInputOf {
                    display: Display::Grid,
                    grid_column: GridPlacement::try_lines(1, 2).unwrap(),
                    grid_template_columns: inherited(),
                    padding: Edges::new(
                        LengthOf::ZERO,
                        LengthOf::ZERO,
                        LengthOf::ZERO,
                        LengthOf::px(px(padding)),
                    ),
                    ..NodeInputOf::default()
                },
            );
        let batch = compute_layout(&tree, 1, grid_lanes_subgrid_intrinsic_request()).unwrap();
        assert_eq!(
            fri08_c01_placement_output(&batch, 1).size.width,
            px(expected_width),
            "empty nested wrapper with {padding}px start padding",
        );
    }
}

#[test]
fn grid_lanes_empty_subgrid_edge_floors_f32() {
    assert_grid_lanes_empty_subgrid_edge_floors::<f32>();
}

#[test]
fn grid_lanes_empty_subgrid_edge_floors_f64() {
    assert_grid_lanes_empty_subgrid_edge_floors::<f64>();
}

fn assert_grid_lanes_empty_subgrid_edge_qualification<S: LayoutScalar>() {
    let px = S::from_f64;
    let inherited = || vec![TrackComponentOf::Subgrid(SubgridTrack::new(Vec::new()))];
    let padding = |value| {
        Edges::new(
            LengthOf::ZERO,
            LengthOf::ZERO,
            LengthOf::ZERO,
            LengthOf::px(px(value)),
        )
    };
    let wrapper = NodeInputOf {
        display: Display::Grid,
        grid_template_columns: inherited(),
        ..NodeInputOf::default()
    };
    let root = |tracks| NodeInputOf {
        display: Display::InlineGridLanes,
        grid_template_columns: tracks,
        ..NodeInputOf::default()
    };
    let containment = |size_containment| {
        PublicLayoutTreeOf::new()
            .children(1, [2])
            .children(2, [3])
            .style(1, root(vec![TrackComponentOf::AUTO; 3]))
            .style(
                2,
                NodeInputOf {
                    size_containment,
                    padding: padding(5.0),
                    ..wrapper.clone()
                },
            )
            .style(3, NodeInputOf::default())
            .measure(3, Size::new(px(60.0), px(0.0)))
    };
    let gutter = |width| {
        PublicLayoutTreeOf::new()
            .children(1, [2])
            .children(2, [3])
            .children(3, [])
            .style(1, root(vec![TrackComponentOf::AUTO; 2]))
            .style(
                2,
                NodeInputOf {
                    overflow: computed_overflow(Overflow::Hidden, Overflow::Hidden),
                    scrollbar_gutter: ScrollbarGutter::Stable,
                    scrollbar_width: ScrollbarWidthOf::try_new(px(width)).unwrap(),
                    ..wrapper.clone()
                },
            )
            .style(
                3,
                NodeInputOf {
                    padding: padding(7.0),
                    ..wrapper.clone()
                },
            )
    };
    let fixed_minimum = PublicLayoutTreeOf::new()
        .children(1, [2])
        .children(2, [])
        .style(
            1,
            root(vec![TrackComponentOf::minmax(
                MinTrackSizingOf::px(px(0.0)),
                MaxTrackSizingOf::MAX_CONTENT,
            )]),
        )
        .style(
            2,
            NodeInputOf {
                padding: padding(10.0),
                ..wrapper.clone()
            },
        );
    let cyclic_gap = |nested| {
        let mut tree = PublicLayoutTreeOf::new()
            .children(1, [2])
            .style(1, root(vec![TrackComponentOf::AUTO; 2]));
        let terminal = if nested {
            tree = tree.children(2, [3]).style(
                2,
                NodeInputOf {
                    grid_column: GridPlacement::try_span(2).unwrap(),
                    ..wrapper.clone()
                },
            );
            3
        } else {
            2
        };
        tree.children(terminal, []).style(
            terminal,
            NodeInputOf {
                grid_column: GridPlacement::try_span(2).unwrap(),
                size: Size::new(PreferredSizeOf::px(px(100.0)), PreferredSizeOf::AUTO),
                gap: Size::new(LengthOf::percent(px(0.1)), LengthOf::ZERO),
                ..wrapper.clone()
            },
        )
    };
    let mut differences = Vec::new();
    for (label, tree, expected) in [
        (
            "inline size containment",
            containment(crate::SizeContainment::InlineAxis),
            15.0,
        ),
        (
            "size containment",
            containment(crate::SizeContainment::BothAxes),
            15.0,
        ),
        ("zero ancestor gutter", gutter(0.0), 14.0),
        ("ancestor gutter", gutter(10.0), 34.0),
        ("fixed minimum track", fixed_minimum, 0.0),
        ("cyclic percentage gap", cyclic_gap(false), 0.0),
        ("nested cyclic percentage gap", cyclic_gap(true), 0.0),
    ] {
        match compute_layout(&tree, 1, grid_lanes_subgrid_intrinsic_request()) {
            Ok(batch) => {
                let width = fri08_c01_placement_output(&batch, 1).size.width;
                if width != px(expected) {
                    differences.push(format!("{label}: {width:?}, expected {expected}"));
                }
            }
            Err(error) => differences.push(format!("{label}: {error:?}")),
        }
    }
    assert!(differences.is_empty(), "{}", differences.join("\n"));
}

#[test]
fn grid_lanes_empty_subgrid_edge_qualification_f32() {
    assert_grid_lanes_empty_subgrid_edge_qualification::<f32>();
}

#[test]
fn grid_lanes_empty_subgrid_edge_qualification_f64() {
    assert_grid_lanes_empty_subgrid_edge_qualification::<f64>();
}

fn assert_grid_lanes_padded_empty_wrapper_chain<S: LayoutScalar>() {
    let px = S::from_f64;
    let wrapper = NodeInputOf {
        display: Display::Grid,
        grid_template_columns: vec![TrackComponentOf::Subgrid(SubgridTrack::new(Vec::new()))],
        grid_template_rows: vec![TrackComponentOf::AUTO],
        padding: Edges::new(
            LengthOf::px(px(11.0)),
            LengthOf::px(px(3.0)),
            LengthOf::px(px(9.0)),
            LengthOf::px(px(7.0)),
        ),
        ..NodeInputOf::default()
    };
    let mut differences = Vec::new();
    for definite_inner in [false, true] {
        let mut tree = PublicLayoutTreeOf::new()
            .children(1, [2, 15, 16, 17, 18])
            .children(2, [3, 4])
            .children(3, 5..10)
            .children(4, 10..15)
            .style(
                1,
                NodeInputOf {
                    display: Display::InlineGridLanes,
                    grid_template_columns: vec![TrackComponentOf::AUTO; 4],
                    ..NodeInputOf::default()
                },
            )
            .style(
                2,
                NodeInputOf {
                    grid_column: GridPlacement::try_span(4).unwrap(),
                    gap: Size::new(LengthOf::px(px(10.0)), LengthOf::px(px(20.0))),
                    ..wrapper.clone()
                },
            );
        for (node, line) in [(3, 1), (4, 3)] {
            let grid_column = if definite_inner {
                GridPlacement::try_line_span(line, 2).unwrap()
            } else {
                GridPlacement::try_span(2).unwrap()
            };
            tree = tree.style(
                node,
                NodeInputOf {
                    grid_column,
                    ..wrapper.clone()
                },
            );
        }
        for node in 5..15 {
            tree = tree.children(node, []).style(node, wrapper.clone());
        }
        for (node, line) in (15..19).zip(1..5) {
            tree = tree
                .style(
                    node,
                    NodeInputOf {
                        grid_column: GridPlacement::try_line(line).unwrap(),
                        min_size: Size::ZERO.map(MinSizeOf::px),
                        ..NodeInputOf::default()
                    },
                )
                .measure(node, Size::ZERO);
        }
        match compute_layout(&tree, 1, grid_lanes_subgrid_intrinsic_request()) {
            Ok(batch) => {
                let width = fri08_c01_placement_output(&batch, 1).size.width;
                let track_widths: Vec<_> = (15..19)
                    .map(|node| fri08_c01_placement_output(&batch, node).size.width)
                    .collect();
                if width != px(116.0) || track_widths != [px(29.0); 4] {
                    differences.push(format!(
                        "definite inner {definite_inner}: {width:?} with tracks {track_widths:?}, expected 116 with four 29px tracks"
                    ));
                }
            }
            Err(error) => differences.push(format!("definite inner {definite_inner}: {error:?}")),
        }
    }
    assert!(differences.is_empty(), "{}", differences.join("\n"));
}

#[test]
fn grid_lanes_padded_empty_wrapper_chain_f32() {
    assert_grid_lanes_padded_empty_wrapper_chain::<f32>();
}

#[test]
fn grid_lanes_padded_empty_wrapper_chain_f64() {
    assert_grid_lanes_padded_empty_wrapper_chain::<f64>();
}

fn assert_grid_lanes_subgrid_parent_names_follow_placement<S: LayoutScalar>() {
    let px = S::from_f64;
    let mut differences = Vec::new();
    for display in [Display::Grid, Display::GridLanes] {
        for (placement, parent_name_geometry) in [
            (GridPlacement::try_span(2).unwrap(), (20.0, 30.0)),
            (GridPlacement::try_line_span(1, 2).unwrap(), (0.0, 20.0)),
        ] {
            let mut tree = PublicLayoutTreeOf::new()
                .children(1, [2])
                .children(2, [3, 4, 5])
                .style(
                    1,
                    NodeInputOf {
                        display: Display::GridLanes,
                        grid_template_columns: vec![
                            TrackComponentOf::line_names(["parent-only"]),
                            TrackComponentOf::px(px(20.0)),
                        ],
                        grid_auto_columns: vec![TrackComponentOf::px(px(30.0))],
                        grid_template_areas: GridTemplateAreas {
                            rows: vec![GridTemplateAreaRow {
                                cells: vec![Some("parent-area".into())],
                            }],
                        },
                        justify_content: Some(AlignContent::Start),
                        ..NodeInputOf::default()
                    },
                )
                .style(
                    2,
                    NodeInputOf {
                        display,
                        grid_column: placement,
                        grid_template_columns: vec![TrackComponentOf::Subgrid(SubgridTrack::new(
                            vec![vec!["local".into()]],
                        ))],
                        grid_template_rows: vec![TrackComponentOf::px(px(10.0)); 3],
                        ..NodeInputOf::default()
                    },
                );
            for (row, name) in ["parent-only", "parent-area-start", "local"]
                .into_iter()
                .enumerate()
            {
                let node = u32::try_from(row).unwrap() + 3;
                tree = tree.children(node, []).style(
                    node,
                    NodeInputOf {
                        raw_grid_column: RawGridPlacement::new(
                            RawGridLine::NamedLine {
                                name: name.into(),
                                index: 1,
                            },
                            RawGridLine::Span(1),
                        ),
                        grid_row: GridPlacement::try_line(isize::try_from(row).unwrap() + 1)
                            .unwrap(),
                        ..NodeInputOf::default()
                    },
                );
            }
            let batch =
                compute_layout(&tree, 1, super::fixtures::fri08_c01_placement_request()).unwrap();
            let wrapper = fri08_c01_placement_output(&batch, 2);
            let actual = [3, 4, 5].map(|node| {
                let item = fri08_c01_placement_output(&batch, node);
                (item.location.x, item.size.width)
            });
            let expected = [parent_name_geometry, parent_name_geometry, (0.0, 20.0)]
                .map(|(x, width)| (px(x), px(width)));
            if wrapper.size.width != px(50.0) || actual != expected {
                differences.push(format!(
                    "{display:?} {placement:?}: wrapper {:?}, {actual:?}, expected 50 and {expected:?}",
                    wrapper.size.width
                ));
            }
        }
    }
    let intrinsic = [false, true].map(|declare_parent_name| {
        let mut columns = Vec::new();
        if declare_parent_name {
            columns.push(TrackComponentOf::line_names(["parent-only"]));
        }
        columns.extend([TrackComponentOf::AUTO, TrackComponentOf::AUTO]);
        let tree = PublicLayoutTreeOf::new()
            .children(1, [2])
            .children(2, [3])
            .children(3, [])
            .style(
                1,
                NodeInputOf {
                    display: Display::InlineGridLanes,
                    grid_template_columns: columns,
                    ..NodeInputOf::default()
                },
            )
            .style(
                2,
                NodeInputOf {
                    display: Display::Grid,
                    grid_column: GridPlacement::try_span(2).unwrap(),
                    grid_template_columns: vec![TrackComponentOf::Subgrid(SubgridTrack::new(
                        Vec::new(),
                    ))],
                    padding: Edges::new(
                        LengthOf::ZERO,
                        LengthOf::px(px(20.0)),
                        LengthOf::ZERO,
                        LengthOf::px(px(10.0)),
                    ),
                    ..NodeInputOf::default()
                },
            )
            .style(
                3,
                NodeInputOf {
                    raw_grid_column: RawGridPlacement::new(
                        RawGridLine::NamedLine {
                            name: "parent-only".into(),
                            index: 1,
                        },
                        RawGridLine::Span(1),
                    ),
                    size: Size::new(px(30.0), px(10.0)).map(PreferredSizeOf::px),
                    ..NodeInputOf::default()
                },
            );
        let batch = compute_layout(&tree, 1, grid_lanes_subgrid_intrinsic_request()).unwrap();
        fri08_c01_placement_output(&batch, 1).size.width
    });
    if intrinsic[0] != intrinsic[1] {
        differences.push(format!(
            "automatic intrinsic width changes with a parent-only name: {intrinsic:?}"
        ));
    }
    for typed in [false, true] {
        let placement = if typed {
            GridPlacement::try_line_span(2, 2).unwrap()
        } else {
            GridPlacement::AUTO
        };
        let raw_placement = if typed {
            RawGridPlacement::AUTO
        } else {
            RawGridPlacement::new(RawGridLine::Line(2), RawGridLine::Span(2))
        };
        let tree = PublicLayoutTreeOf::new()
            .children(1, [2])
            .children(2, [3])
            .children(3, [4])
            .children(4, [])
            .style(
                1,
                NodeInputOf {
                    display: Display::Grid,
                    grid_template_columns: vec![
                        TrackComponentOf::px(px(20.0)),
                        TrackComponentOf::px(px(30.0)),
                    ],
                    grid_template_rows: vec![TrackComponentOf::px(px(10.0))],
                    ..NodeInputOf::default()
                },
            )
            .style(
                2,
                NodeInputOf {
                    display: Display::GridLanes,
                    grid_column: GridPlacement::try_lines(1, 3).unwrap(),
                    grid_template_columns: vec![TrackComponentOf::Subgrid(SubgridTrack::new(
                        vec![vec!["lane-only".into()]],
                    ))],
                    ..NodeInputOf::default()
                },
            )
            .style(
                3,
                NodeInputOf {
                    display: Display::Grid,
                    grid_column: placement,
                    raw_grid_column: raw_placement,
                    grid_template_columns: vec![TrackComponentOf::Subgrid(SubgridTrack::new(
                        Vec::new(),
                    ))],
                    ..NodeInputOf::default()
                },
            )
            .style(
                4,
                NodeInputOf {
                    raw_grid_column: RawGridPlacement::new(
                        RawGridLine::NamedLine {
                            name: "lane-only".into(),
                            index: 1,
                        },
                        RawGridLine::Span(1),
                    ),
                    ..NodeInputOf::default()
                },
            );
        let batch = match compute_layout(&tree, 1, super::fixtures::fri08_c01_placement_request()) {
            Ok(batch) => batch,
            Err(error) => {
                differences.push(format!("nested inherited span, typed {typed}: {error:?}"));
                continue;
            }
        };
        let wrapper = fri08_c01_placement_output(&batch, 3);
        let child = fri08_c01_placement_output(&batch, 4);
        let actual = (wrapper.location.x, wrapper.size.width, child.size.width);
        let expected = (px(20.0), px(30.0), px(30.0));
        if actual != expected {
            differences.push(format!(
                "nested inherited span, typed {typed}: {actual:?}, expected {expected:?}"
            ));
        }
    }
    assert!(differences.is_empty(), "{}", differences.join("\n"));
}

#[test]
fn grid_lanes_subgrid_parent_names_follow_placement_f32() {
    assert_grid_lanes_subgrid_parent_names_follow_placement::<f32>();
}

#[test]
fn grid_lanes_subgrid_parent_names_follow_placement_f64() {
    assert_grid_lanes_subgrid_parent_names_follow_placement::<f64>();
}

fn assert_grid_lanes_subgrid_shares_inherited_track<S: LayoutScalar>() {
    let px = S::from_f64;
    let leaf = |name: &str, height| NodeInputOf {
        raw_grid_column: RawGridPlacement::new(
            RawGridLine::NamedLine {
                name: name.into(),
                index: 1,
            },
            RawGridLine::Span(1),
        ),
        size: Size::new(PreferredSizeOf::AUTO, PreferredSizeOf::px(px(height))),
        ..NodeInputOf::default()
    };
    let parent = NodeInputOf {
        display: Display::Grid,
        grid_template_columns: vec![
            TrackComponentOf::line_names(["parent"]),
            TrackComponentOf::px(px(20.0)),
        ],
        grid_template_rows: vec![TrackComponentOf::px(px(40.0))],
        ..NodeInputOf::default()
    };
    let wrapper = NodeInputOf {
        display: Display::GridLanes,
        grid_column: GridPlacement::try_span(1).unwrap(),
        grid_template_columns: vec![TrackComponentOf::Subgrid(SubgridTrack::new(vec![
            vec!["local".into()],
            Vec::new(),
            Vec::new(),
        ]))],
        ..NodeInputOf::default()
    };
    for descendants in [&[3][..], &[4][..], &[3, 4][..]] {
        let mut tree = PublicLayoutTreeOf::new()
            .children(1, [2])
            .children(2, descendants.iter().copied())
            .style(1, parent.clone())
            .style(2, wrapper.clone());
        for &node in descendants {
            let (name, height) = if node == 3 {
                ("local", 10.0)
            } else {
                ("parent", 15.0)
            };
            tree = tree.children(node, []).style(node, leaf(name, height));
        }
        let batch = compute_layout(&tree, 1, super::fixtures::fri08_c01_placement_request())
            .unwrap_or_else(|error| panic!("descendants {descendants:?}: {error:?}"));
        let mut expected_y = S::ZERO;
        for &node in descendants {
            let child = fri08_c01_placement_output(&batch, node);
            let height = if node == 3 { px(10.0) } else { px(15.0) };
            assert_eq!(
                (
                    child.location.x,
                    child.location.y,
                    child.size.width,
                    child.size.height
                ),
                (S::ZERO, expected_y, px(20.0), height),
                "descendants {descendants:?}, node {node}",
            );
            expected_y = expected_y + height;
        }
    }
}

#[test]
fn grid_lanes_subgrid_shares_inherited_track_f32() {
    assert_grid_lanes_subgrid_shares_inherited_track::<f32>();
}

#[test]
fn grid_lanes_subgrid_shares_inherited_track_f64() {
    assert_grid_lanes_subgrid_shares_inherited_track::<f64>();
}

fn assert_grid_lanes_subgrid_contributes_to_inherited_tracks<S: LayoutScalar>() {
    let px = S::from_f64;
    for (placement, expected_width) in [
        (GridPlacement::AUTO, 50.0),
        (GridPlacement::try_line(2).unwrap(), 25.0),
    ] {
        let tree = PublicLayoutTreeOf::new()
            .children(1, [2])
            .children(2, [3])
            .children(3, [])
            .style(
                1,
                NodeInputOf {
                    display: Display::InlineGrid,
                    grid_template_columns: vec![TrackComponentOf::AUTO; 2],
                    grid_template_rows: vec![TrackComponentOf::px(px(40.0))],
                    ..NodeInputOf::default()
                },
            )
            .style(
                2,
                NodeInputOf {
                    display: Display::GridLanes,
                    grid_column: GridPlacement::try_lines(1, 3).unwrap(),
                    grid_template_columns: vec![TrackComponentOf::Subgrid(SubgridTrack::new(
                        Vec::new(),
                    ))],
                    ..NodeInputOf::default()
                },
            )
            .style(
                3,
                NodeInputOf {
                    grid_column: placement,
                    size: Size::new(px(25.0), px(10.0)).map(PreferredSizeOf::px),
                    ..NodeInputOf::default()
                },
            );
        let batch = compute_layout(&tree, 1, grid_lanes_subgrid_intrinsic_request()).unwrap();
        assert_eq!(
            fri08_c01_placement_output(&batch, 1).size.width,
            px(expected_width),
            "descendant placement {placement:?}",
        );
    }
}

#[test]
fn grid_lanes_subgrid_contributes_to_inherited_tracks_f32() {
    assert_grid_lanes_subgrid_contributes_to_inherited_tracks::<f32>();
}

#[test]
fn grid_lanes_subgrid_contributes_to_inherited_tracks_f64() {
    assert_grid_lanes_subgrid_contributes_to_inherited_tracks::<f64>();
}

fn assert_grid_lanes_nested_candidate_incidence<S: LayoutScalar>() {
    let px = S::from_f64;
    let inherited = || vec![TrackComponentOf::Subgrid(SubgridTrack::new(Vec::new()))];
    let tree = PublicLayoutTreeOf::new()
        .children(1, [2])
        .children(2, [3])
        .children(3, [4])
        .children(4, [])
        .style(
            1,
            NodeInputOf {
                display: Display::InlineGrid,
                grid_template_columns: vec![TrackComponentOf::AUTO; 3],
                grid_template_rows: vec![TrackComponentOf::px(px(40.0))],
                ..NodeInputOf::default()
            },
        )
        .style(
            2,
            NodeInputOf {
                display: Display::GridLanes,
                grid_column: GridPlacement::try_lines(1, 4).unwrap(),
                grid_template_columns: inherited(),
                ..NodeInputOf::default()
            },
        )
        .style(
            3,
            NodeInputOf {
                display: Display::GridLanes,
                grid_column: GridPlacement::try_span(2).unwrap(),
                grid_template_columns: inherited(),
                ..NodeInputOf::default()
            },
        )
        .style(
            4,
            NodeInputOf {
                size: Size::new(px(25.0), px(10.0)).map(PreferredSizeOf::px),
                ..NodeInputOf::default()
            },
        );
    let batch = compute_layout(&tree, 1, grid_lanes_subgrid_intrinsic_request()).unwrap();
    assert_eq!(fri08_c01_placement_output(&batch, 1).size.width, px(75.0));
}

#[test]
fn grid_lanes_nested_candidate_incidence_f32() {
    assert_grid_lanes_nested_candidate_incidence::<f32>();
}

#[test]
fn grid_lanes_nested_candidate_incidence_f64() {
    assert_grid_lanes_nested_candidate_incidence::<f64>();
}

fn assert_grid_lanes_subgrid_span_from_name_slots<S: LayoutScalar>() {
    let px = S::from_f64;
    let named = |name: &str| {
        RawGridPlacement::new(
            RawGridLine::NamedLine {
                name: name.into(),
                index: 1,
            },
            RawGridLine::Span(1),
        )
    };
    let mut differences = Vec::new();
    for (parent_display, child_display, parent_rows) in [
        (Display::GridLanes, Display::Grid, false),
        (Display::GridLanes, Display::Grid, true),
        (Display::Grid, Display::GridLanes, false),
    ] {
        let stacking_template = if parent_display == Display::Grid {
            vec![TrackComponentOf::px(px(10.0))]
        } else {
            Vec::new()
        };
        let tracks = || {
            vec![
                TrackComponentOf::line_names(["parent"]),
                TrackComponentOf::px(px(20.0)),
            ]
        };
        let root = NodeInputOf {
            display: parent_display,
            grid_lanes_flow: crate::GridLanesFlow::new(
                if parent_rows {
                    GridAxisKind::Row
                } else {
                    GridAxisKind::Column
                },
                false,
                false,
            ),
            grid_template_columns: if parent_rows { Vec::new() } else { tracks() },
            grid_template_rows: if parent_rows {
                tracks()
            } else {
                stacking_template
            },
            grid_auto_columns: vec![TrackComponentOf::px(px(30.0))],
            grid_auto_rows: vec![TrackComponentOf::px(px(30.0))],
            ..NodeInputOf::default()
        };
        for (typed, raw, width, parent_named_extent) in [
            (GridPlacement::AUTO, RawGridPlacement::AUTO, 50.0, 30.0),
            (
                GridPlacement::try_line(1).unwrap(),
                RawGridPlacement::AUTO,
                50.0,
                20.0,
            ),
            (
                GridPlacement::AUTO,
                RawGridPlacement::new(RawGridLine::Line(1), RawGridLine::Auto),
                50.0,
                20.0,
            ),
            (
                GridPlacement::try_span(1).unwrap(),
                RawGridPlacement::AUTO,
                20.0,
                20.0,
            ),
        ] {
            let child =
                NodeInputOf {
                    display: child_display,
                    writing_mode: if parent_rows {
                        WritingMode::VerticalRl
                    } else {
                        WritingMode::HorizontalTb
                    },
                    grid_column: if parent_rows {
                        GridPlacement::AUTO
                    } else {
                        typed
                    },
                    grid_row: if parent_rows {
                        typed
                    } else {
                        GridPlacement::AUTO
                    },
                    raw_grid_column: if parent_rows {
                        RawGridPlacement::AUTO
                    } else {
                        raw.clone()
                    },
                    raw_grid_row: if parent_rows {
                        raw.clone()
                    } else {
                        RawGridPlacement::AUTO
                    },
                    grid_template_columns: vec![TrackComponentOf::Subgrid(SubgridTrack::new(
                        vec![vec!["local".into()], Vec::new(), Vec::new()],
                    ))],
                    ..NodeInputOf::default()
                };
            let two_descendants = child_display == Display::Grid;
            let mut tree = PublicLayoutTreeOf::new()
                .children(1, [2])
                .children(
                    2,
                    if two_descendants {
                        &[3, 4][..]
                    } else {
                        &[3][..]
                    }
                    .iter()
                    .copied(),
                )
                .children(3, [])
                .style(1, root.clone())
                .style(2, child)
                .style(
                    3,
                    NodeInputOf {
                        raw_grid_column: named(if two_descendants { "local" } else { "parent" }),
                        ..NodeInputOf::default()
                    },
                );
            if two_descendants {
                tree = tree.children(4, []).style(
                    4,
                    NodeInputOf {
                        raw_grid_column: named("parent"),
                        ..NodeInputOf::default()
                    },
                );
            }
            let request = super::fixtures::fri08_c01_placement_request();
            let batch = match compute_layout(&tree, 1, request) {
                Ok(batch) => batch,
                Err(error) => {
                    differences.push(format!("{parent_display:?} {child_display:?} rows={parent_rows}, {typed:?} {raw:?}: {error:?}"));
                    continue;
                }
            };
            let extent = |node| {
                let item = fri08_c01_placement_output(&batch, node);
                if parent_rows {
                    item.size.height
                } else {
                    item.size.width
                }
            };
            let actual = (extent(2), extent(3), two_descendants.then(|| extent(4)));
            let expected = (
                px(width),
                px(20.0),
                two_descendants.then(|| px(parent_named_extent)),
            );
            if actual != expected {
                differences.push(format!("{parent_display:?} {child_display:?} rows={parent_rows}, {typed:?} {raw:?}: {actual:?}, expected {expected:?}"));
            }
        }
    }
    assert!(differences.is_empty(), "{}", differences.join("\n"));
}

#[test]
fn grid_lanes_subgrid_span_from_name_slots_f32() {
    assert_grid_lanes_subgrid_span_from_name_slots::<f32>();
}

#[test]
fn grid_lanes_subgrid_span_from_name_slots_f64() {
    assert_grid_lanes_subgrid_span_from_name_slots::<f64>();
}

fn assert_automatic_subgrid_span_from_name_slots<S: LayoutScalar>() {
    let px = S::from_f64;
    let mut differences = Vec::new();
    for (placement, expected_subgrid, expected_sibling) in [
        (GridPlacement::AUTO, (0.0, 50.0), (50.0, 40.0)),
        (
            GridPlacement::try_line(2).unwrap(),
            (20.0, 70.0),
            (0.0, 20.0),
        ),
        (
            GridPlacement::try_end_line(3).unwrap(),
            (0.0, 50.0),
            (50.0, 40.0),
        ),
        (
            GridPlacement::try_span(1).unwrap(),
            (0.0, 20.0),
            (20.0, 30.0),
        ),
        (
            GridPlacement::try_lines(2, 3).unwrap(),
            (20.0, 30.0),
            (0.0, 20.0),
        ),
    ] {
        let tree = PublicLayoutTreeOf::<S>::new()
            .children(1, [2, 3])
            .children(2, [])
            .children(3, [])
            .style(
                1,
                NodeInputOf {
                    display: Display::Grid,
                    grid_template_columns: [20.0, 30.0, 40.0]
                        .map(|width| TrackComponentOf::px(px(width)))
                        .into(),
                    grid_template_rows: vec![TrackComponentOf::px(px(10.0))],
                    ..NodeInputOf::default()
                },
            )
            .style(
                2,
                NodeInputOf {
                    display: Display::Grid,
                    grid_column: placement,
                    grid_row: GridPlacement::try_line(1).unwrap(),
                    grid_template_columns: vec![TrackComponentOf::Subgrid(crate::SubgridTrack {
                        name_components: vec![SubgridLineNameComponent::Repeat {
                            count: SubgridLineNameRepeatCount::Count(3),
                            line_name_sets: vec![vec!["edge".into()]],
                        }],
                    })],
                    ..NodeInputOf::default()
                },
            )
            .style(
                3,
                NodeInputOf {
                    grid_row: GridPlacement::try_line(1).unwrap(),
                    ..NodeInputOf::default()
                },
            );
        let batch =
            compute_layout(&tree, 1, super::fixtures::fri08_c01_placement_request()).unwrap();
        let geometry = |node| {
            let child = super::fixtures::fri08_c01_placement_output(&batch, node);
            (child.location.x, child.size.width)
        };
        let actual = (geometry(2), geometry(3));
        let expected = (
            (px(expected_subgrid.0), px(expected_subgrid.1)),
            (px(expected_sibling.0), px(expected_sibling.1)),
        );
        if actual != expected {
            differences.push(format!("{placement:?}: {actual:?}, expected {expected:?}"));
        }
    }
    assert!(differences.is_empty(), "{}", differences.join("\n"));
}

#[test]
fn automatic_subgrid_span_from_name_slots_f32() {
    assert_automatic_subgrid_span_from_name_slots::<f32>();
}

#[test]
fn automatic_subgrid_span_from_name_slots_f64() {
    assert_automatic_subgrid_span_from_name_slots::<f64>();
}

fn assert_automatic_subgrid_span_across_implicit_tracks<S: LayoutScalar>() {
    let px = S::from_f64;
    let named = |name: &str, index| RawGridLine::NamedLine {
        name: name.into(),
        index,
    };
    let mut differences = Vec::new();
    for (columns, auto_width, placement, local, expected) in [
        (
            vec![TrackComponentOf::px(px(20.0))],
            30.0,
            GridPlacement::AUTO,
            RawGridPlacement::new(named("edge", 2), named("edge", 3)),
            (50.0, 20.0, 30.0),
        ),
        (
            vec![
                TrackComponentOf::line_names(["parent-start"]),
                TrackComponentOf::px(px(20.0)),
                TrackComponentOf::line_names(["parent-middle"]),
                TrackComponentOf::px(px(30.0)),
            ],
            10.0,
            GridPlacement::try_line(-4).unwrap(),
            RawGridPlacement::new(named("parent-start", 1), named("parent-middle", 1)),
            (30.0, 10.0, 20.0),
        ),
    ] {
        let tree = PublicLayoutTreeOf::<S>::new()
            .children(1, [2])
            .children(2, [3])
            .children(3, [])
            .style(
                1,
                NodeInputOf {
                    display: Display::Grid,
                    grid_template_columns: columns,
                    grid_auto_columns: vec![TrackComponentOf::px(px(auto_width))],
                    grid_template_rows: vec![TrackComponentOf::px(px(10.0))],
                    justify_content: Some(AlignContent::Start),
                    ..NodeInputOf::default()
                },
            )
            .style(
                2,
                NodeInputOf {
                    display: Display::Grid,
                    grid_column: placement,
                    grid_row: GridPlacement::try_line(1).unwrap(),
                    grid_template_columns: vec![TrackComponentOf::Subgrid(crate::SubgridTrack {
                        name_components: vec![SubgridLineNameComponent::Repeat {
                            count: SubgridLineNameRepeatCount::Count(3),
                            line_name_sets: vec![vec!["edge".into()]],
                        }],
                    })],
                    ..NodeInputOf::default()
                },
            )
            .style(
                3,
                NodeInputOf {
                    raw_grid_column: local,
                    grid_row: GridPlacement::try_line(1).unwrap(),
                    ..NodeInputOf::default()
                },
            );
        let batch =
            compute_layout(&tree, 1, super::fixtures::fri08_c01_placement_request()).unwrap();
        let subgrid = super::fixtures::fri08_c01_placement_output(&batch, 2);
        let child = super::fixtures::fri08_c01_placement_output(&batch, 3);
        let actual = (subgrid.size.width, child.location.x, child.size.width);
        let expected = (px(expected.0), px(expected.1), px(expected.2));
        if actual != expected {
            differences.push(format!("{placement:?}: {actual:?}, expected {expected:?}"));
        }
    }
    assert!(differences.is_empty(), "{}", differences.join("\n"));
}

#[test]
fn automatic_subgrid_span_across_implicit_tracks_f32() {
    assert_automatic_subgrid_span_across_implicit_tracks::<f32>();
}

#[test]
fn automatic_subgrid_span_across_implicit_tracks_f64() {
    assert_automatic_subgrid_span_across_implicit_tracks::<f64>();
}

fn assert_automatic_subgrid_span_clamps_to_inherited_tracks<S: LayoutScalar>() {
    let px = S::from_f64;
    let huge_subgrid = || {
        TrackComponentOf::Subgrid(crate::SubgridTrack {
            name_components: vec![SubgridLineNameComponent::Repeat {
                count: SubgridLineNameRepeatCount::Count(usize::MAX),
                line_name_sets: vec![vec!["slot".into()]],
            }],
        })
    };
    let mut differences = Vec::new();
    for (columns, anchor, raw_anchor) in [
        (
            true,
            GridPlacement::try_line(1).unwrap(),
            RawGridPlacement::AUTO,
        ),
        (true, GridPlacement::AUTO, RawGridPlacement::line(1)),
        (false, GridPlacement::AUTO, RawGridPlacement::AUTO),
    ] {
        let nested = if columns {
            NodeInputOf {
                display: Display::Grid,
                grid_column: anchor,
                raw_grid_column: raw_anchor,
                grid_row: GridPlacement::try_line(1).unwrap(),
                grid_template_columns: vec![huge_subgrid()],
                ..NodeInputOf::default()
            }
        } else {
            NodeInputOf {
                display: Display::Grid,
                grid_column: GridPlacement::try_line(1).unwrap(),
                grid_template_rows: vec![huge_subgrid()],
                ..NodeInputOf::default()
            }
        };
        let tree = PublicLayoutTreeOf::<S>::new()
            .children(1, [2])
            .children(2, [3])
            .children(3, [])
            .style(
                1,
                NodeInputOf {
                    display: Display::Grid,
                    grid_template_columns: [20.0, 30.0]
                        .map(|width| TrackComponentOf::px(px(width)))
                        .into(),
                    grid_template_rows: [10.0, 15.0]
                        .map(|height| TrackComponentOf::px(px(height)))
                        .into(),
                    ..NodeInputOf::default()
                },
            )
            .style(
                2,
                NodeInputOf {
                    display: Display::Grid,
                    grid_column: GridPlacement::try_lines(1, 3).unwrap(),
                    grid_row: GridPlacement::try_lines(1, 3).unwrap(),
                    grid_template_columns: subgrid_track_of(),
                    grid_template_rows: subgrid_track_of(),
                    ..NodeInputOf::default()
                },
            )
            .style(3, nested);
        let batch =
            compute_layout(&tree, 1, super::fixtures::fri08_c01_placement_request()).unwrap();
        let child = super::fixtures::fri08_c01_placement_output(&batch, 3);
        let expected = if columns {
            Size::new(px(50.0), px(10.0))
        } else {
            Size::new(px(20.0), px(25.0))
        };
        if child.size != expected {
            differences.push(format!(
                "{columns} {anchor:?}: {:?}, expected {expected:?}",
                child.size
            ));
        }
    }
    assert!(differences.is_empty(), "{}", differences.join("\n"));
}

#[test]
fn automatic_subgrid_span_clamps_to_inherited_tracks_f32() {
    assert_automatic_subgrid_span_clamps_to_inherited_tracks::<f32>();
}

#[test]
fn automatic_subgrid_span_clamps_to_inherited_tracks_f64() {
    assert_automatic_subgrid_span_clamps_to_inherited_tracks::<f64>();
}

fn assert_nested_subgrid_keeps_inherited_area_edge_orientation<S: LayoutScalar>() {
    let px = S::from_f64;
    let mut differences = Vec::new();
    for leading in [false, true] {
        let parent = NodeInputOf {
            display: Display::Grid,
            grid_template_columns: [20.0, 30.0]
                .map(|width| TrackComponentOf::px(px(width)))
                .into(),
            grid_auto_columns: vec![TrackComponentOf::px(px(10.0))],
            grid_template_rows: vec![TrackComponentOf::px(px(10.0))],
            grid_template_areas: GridTemplateAreas {
                rows: vec![GridTemplateAreaRow {
                    cells: vec![Some("zone".into()), Some("zone".into())],
                }],
            },
            ..NodeInputOf::default()
        };
        let wrapper = |grid_column| NodeInputOf {
            display: Display::Grid,
            direction: Direction::Rtl,
            grid_column,
            grid_row: GridPlacement::try_line(1).unwrap(),
            grid_template_columns: subgrid_track_of(),
            grid_template_rows: vec![TrackComponentOf::px(px(10.0))],
            ..NodeInputOf::default()
        };
        let tree = PublicLayoutTreeOf::<S>::new()
            .children(1, [2])
            .children(2, [3])
            .children(3, [4])
            .children(4, [])
            .style(1, parent)
            .style(
                2,
                wrapper(GridPlacement::try_lines(if leading { -4 } else { 1 }, 3).unwrap()),
            )
            .style(
                3,
                wrapper(GridPlacement::try_lines(1, if leading { 4 } else { 3 }).unwrap()),
            )
            .style(
                4,
                NodeInputOf {
                    raw_grid_column: RawGridPlacement::new(
                        RawGridLine::Auto,
                        RawGridLine::NamedLine {
                            name: "zone-start".into(),
                            index: 1,
                        },
                    ),
                    grid_row: GridPlacement::try_line(1).unwrap(),
                    ..NodeInputOf::default()
                },
            );
        let batch =
            compute_layout(&tree, 1, super::fixtures::fri08_c01_placement_request()).unwrap();
        let leaf = super::fixtures::fri08_c01_placement_output(&batch, 4);
        let actual = (leaf.location.x, leaf.size.width);
        let expected = (px(if leading { 10.0 } else { 0.0 }), px(20.0));
        if actual != expected {
            differences.push(format!(
                "leading={leading}: {actual:?}, expected {expected:?}"
            ));
        }
    }
    assert!(differences.is_empty(), "{}", differences.join("\n"));
}

#[test]
fn nested_subgrid_keeps_inherited_area_edge_orientation_f32() {
    assert_nested_subgrid_keeps_inherited_area_edge_orientation::<f32>();
}

#[test]
fn nested_subgrid_keeps_inherited_area_edge_orientation_f64() {
    assert_nested_subgrid_keeps_inherited_area_edge_orientation::<f64>();
}

fn assert_automatic_subgrid_span_preserves_leading_raw_anchors<S: LayoutScalar>() {
    let px = S::from_f64;
    let mut differences = Vec::new();
    for (name_slots, typed, raw, expected_width) in [
        (
            3,
            GridPlacement::try_end_line(1).unwrap(),
            RawGridPlacement::AUTO,
            22.0,
        ),
        (
            3,
            GridPlacement::AUTO,
            RawGridPlacement::new(RawGridLine::Auto, RawGridLine::Line(1)),
            22.0,
        ),
        (
            3,
            GridPlacement::AUTO,
            RawGridPlacement::new(
                RawGridLine::Auto,
                RawGridLine::NamedLine {
                    name: "first".into(),
                    index: 1,
                },
            ),
            22.0,
        ),
        (3, GridPlacement::AUTO, RawGridPlacement::line(-4), 31.0),
        (
            1,
            GridPlacement::AUTO,
            RawGridPlacement::new(RawGridLine::Auto, RawGridLine::Line(1)),
            11.0,
        ),
    ] {
        let tree = PublicLayoutTreeOf::<S>::new()
            .children(1, [2])
            .children(2, [])
            .style(
                1,
                NodeInputOf {
                    display: Display::Grid,
                    grid_template_columns: vec![
                        TrackComponentOf::line_names(["first"]),
                        TrackComponentOf::px(px(20.0)),
                        TrackComponentOf::px(px(30.0)),
                    ],
                    grid_auto_columns: vec![TrackComponentOf::px(px(11.0))],
                    grid_template_rows: vec![TrackComponentOf::px(px(10.0))],
                    justify_content: Some(AlignContent::Start),
                    ..NodeInputOf::default()
                },
            )
            .style(
                2,
                NodeInputOf {
                    display: Display::Grid,
                    grid_column: typed,
                    raw_grid_column: raw.clone(),
                    grid_row: GridPlacement::try_line(1).unwrap(),
                    grid_template_columns: vec![TrackComponentOf::Subgrid(crate::SubgridTrack {
                        name_components: vec![SubgridLineNameComponent::Repeat {
                            count: SubgridLineNameRepeatCount::Count(name_slots),
                            line_name_sets: vec![vec!["edge".into()]],
                        }],
                    })],
                    ..NodeInputOf::default()
                },
            );
        let batch =
            compute_layout(&tree, 1, super::fixtures::fri08_c01_placement_request()).unwrap();
        let child = super::fixtures::fri08_c01_placement_output(&batch, 2);
        let actual = (child.location.x, child.size.width);
        let expected = (S::ZERO, px(expected_width));
        if actual != expected {
            differences.push(format!(
                "{raw:?} slots={name_slots}: {actual:?}, expected {expected:?}"
            ));
        }
    }
    assert!(differences.is_empty(), "{}", differences.join("\n"));
}

#[test]
fn automatic_subgrid_span_preserves_leading_raw_anchors_f32() {
    assert_automatic_subgrid_span_preserves_leading_raw_anchors::<f32>();
}

#[test]
fn automatic_subgrid_span_preserves_leading_raw_anchors_f64() {
    assert_automatic_subgrid_span_preserves_leading_raw_anchors::<f64>();
}

fn subgrid_line_repeat_placement<S: LayoutScalar>(
    track_widths: &[f64],
    name_components: Vec<SubgridLineNameComponent>,
    start: (&str, isize),
    end: (&str, isize),
) -> (S, S) {
    let line = |(name, index): (&str, isize)| RawGridLine::NamedLine {
        name: name.into(),
        index,
    };
    let tree = PublicLayoutTreeOf::<S>::new()
        .children(1, [2])
        .children(2, [3])
        .children(3, [])
        .style(
            1,
            NodeInputOf {
                display: Display::Grid,
                grid_template_columns: track_widths
                    .iter()
                    .map(|width| TrackComponentOf::px(S::from_f64(*width)))
                    .collect(),
                grid_template_rows: vec![TrackComponentOf::px(S::from_f64(10.0))],
                justify_content: Some(AlignContent::Start),
                ..NodeInputOf::default()
            },
        )
        .style(
            2,
            NodeInputOf {
                display: Display::Grid,
                grid_column: GridPlacement::try_lines(
                    1,
                    isize::try_from(track_widths.len() + 1).unwrap(),
                )
                .unwrap(),
                grid_row: GridPlacement::try_line(1).unwrap(),
                grid_template_columns: vec![TrackComponentOf::Subgrid(crate::SubgridTrack {
                    name_components,
                })],
                grid_template_rows: vec![TrackComponentOf::px(S::from_f64(10.0))],
                ..NodeInputOf::default()
            },
        )
        .style(
            3,
            NodeInputOf {
                raw_grid_column: RawGridPlacement::new(line(start), line(end)),
                grid_row: GridPlacement::try_line(1).unwrap(),
                ..NodeInputOf::default()
            },
        );
    let batch = compute_layout(&tree, 1, super::fixtures::fri08_c01_placement_request()).unwrap();
    let child = super::fixtures::fri08_c01_placement_output(&batch, 3);
    (child.location.x, child.size.width)
}

fn assert_subgrid_counted_line_repeat_clips_to_used_span<S: LayoutScalar>() {
    for count in [2, usize::MAX] {
        let components = vec![
            SubgridLineNameComponent::Repeat {
                count: SubgridLineNameRepeatCount::AutoFill,
                line_name_sets: vec![vec!["fill".into()]],
            },
            SubgridLineNameComponent::Repeat {
                count: SubgridLineNameRepeatCount::Count(count),
                line_name_sets: vec![vec!["a".into()], vec!["b".into()]],
            },
        ];
        assert_eq!(
            subgrid_line_repeat_placement::<S>(
                &[20.0, 30.0],
                components.clone(),
                ("a", 1),
                ("b", 1)
            ),
            (S::ZERO, S::from_f64(20.0)),
        );
        assert_eq!(
            subgrid_line_repeat_placement::<S>(&[20.0, 30.0], components, ("b", 1), ("a", 2)),
            (S::from_f64(20.0), S::from_f64(30.0)),
        );
    }
}

#[test]
fn subgrid_counted_line_repeat_clips_to_used_span_f32() {
    assert_subgrid_counted_line_repeat_clips_to_used_span::<f32>();
}

#[test]
fn subgrid_counted_line_repeat_clips_to_used_span_f64() {
    assert_subgrid_counted_line_repeat_clips_to_used_span::<f64>();
}

fn assert_subgrid_auto_fill_line_repeat_uses_whole_groups<S: LayoutScalar>() {
    let components = vec![
        SubgridLineNameComponent::Repeat {
            count: SubgridLineNameRepeatCount::AutoFill,
            line_name_sets: vec![vec!["a".into()], vec!["b".into()]],
        },
        SubgridLineNameComponent::LineNames(vec!["z".into()]),
        SubgridLineNameComponent::LineNames(vec!["z".into()]),
    ];
    for (tracks, start, width) in [
        (&[20.0, 30.0][..], 0.0, 20.0),
        (&[20.0, 30.0, 40.0, 50.0][..], 50.0, 40.0),
    ] {
        assert_eq!(
            subgrid_line_repeat_placement::<S>(tracks, components.clone(), ("z", 1), ("z", 2)),
            (S::from_f64(start), S::from_f64(width)),
        );
    }
}

#[test]
fn subgrid_auto_fill_line_repeat_uses_whole_groups_f32() {
    assert_subgrid_auto_fill_line_repeat_uses_whole_groups::<f32>();
}

#[test]
fn subgrid_auto_fill_line_repeat_uses_whole_groups_f64() {
    assert_subgrid_auto_fill_line_repeat_uses_whole_groups::<f64>();
}

#[test]
fn subgrid_line_names_expand_auto_fill_and_fixed_slots() {
    let names = named::expand_subgrid_local_line_names(
        GridAxisKind::Column,
        4,
        &[
            SubgridLineNameComponent::LineNames(vec!["start".to_string()]),
            SubgridLineNameComponent::Repeat {
                count: SubgridLineNameRepeatCount::AutoFill,
                line_name_sets: vec![vec!["fill".to_string()]],
            },
            SubgridLineNameComponent::LineNames(vec!["end".to_string()]),
        ],
    )
    .unwrap();

    assert_eq!(names.len(), 5);
    assert_eq!(
        local_line_names(&names),
        vec![
            vec!["start"],
            vec!["fill"],
            vec!["fill"],
            vec!["fill"],
            vec!["end"],
        ]
    );
}

#[test]
fn subgrid_line_names_inherit_parent_explicit_and_local_names() {
    let parent = named_parent_lines(4, &[&["a"], &["b"], &[], &["c"], &["d"]]);
    let local = local_subgrid_entries(&[&["local-start"], &[], &["middle"], &["local-end"]]);

    let lines = named::inherit_subgrid_named_lines(&parent, 2, 5, false, &local, None).unwrap();

    assert_eq!(
        entry_names(lines.entries_on_line(1)),
        vec!["b", "local-start"]
    );
    assert_eq!(entry_names(lines.entries_on_line(3)), vec!["c", "middle"]);
    assert_eq!(
        entry_names(lines.entries_on_line(4)),
        vec!["d", "local-end"]
    );
    assert_eq!(
        lines.entries_on_line(1)[1].origin,
        named::LineNameOrigin::LocalSubgrid
    );
}

#[test]
fn subgrid_line_names_reinherit_local_parent_names() {
    let parent = named_parent_lines(2, &[&["outer"], &[], &["outer-end"]]);
    let outer_local = local_subgrid_entries(&[&["local-start"], &[], &["local-end"]]);
    let outer =
        named::inherit_subgrid_named_lines(&parent, 1, 3, false, &outer_local, None).unwrap();
    let nested_local = local_subgrid_entries(&[&[], &[], &[]]);

    let nested =
        named::inherit_subgrid_named_lines(&outer, 1, 3, false, &nested_local, None).unwrap();

    assert_eq!(
        entry_names(nested.entries_on_line(1)),
        vec!["outer", "local-start"]
    );
    assert_eq!(
        entry_names(nested.entries_on_line(3)),
        vec!["outer-end", "local-end"]
    );
}

#[test]
fn subgrid_line_names_reverse_parent_line_order() {
    let parent = named_parent_lines(4, &[&["a"], &["b"], &[], &["c"], &["d"]]);
    let local = local_subgrid_entries(&[&[], &[], &[], &[]]);

    let lines = named::inherit_subgrid_named_lines(&parent, 2, 5, true, &local, None).unwrap();

    assert_eq!(entry_names(lines.entries_on_line(1)), vec!["d"]);
    assert_eq!(entry_names(lines.entries_on_line(2)), vec!["c"]);
    assert_eq!(entry_names(lines.entries_on_line(4)), vec!["b"]);
}

#[test]
fn subgrid_intrinsic_parent_context_uses_actual_span_and_reversal() {
    let parent = named_parent_lines(4, &[&["a"], &["b"], &[], &["c"], &["d"]]);
    let report = SubgridAxisReport {
        inherit_parent_names: true,
        mapping: GridAxisMappingReport {
            queried_axis: GridAxisKind::Column,
            parent_axis: GridAxisKind::Column,
            child_axis: GridAxisKind::Column,
            reversed: true,
        },
        eligibility: SubgridEligibility::SharedTracks,
    };

    let axis: InheritedGridAxis<Scalar> = intrinsic_subgrid_axis_parent_context(
        report,
        GridArea {
            row: 0,
            column: 1,
            row_end: 1,
            column_end: 4,
            size: LogicalSizeOf::new(Scalar::ZERO, Scalar::ZERO),
        },
        Size::<Scalar>::ZERO,
        SubgridParentTrackSource {
            baseline_references: crate::GridBaselineReferences::default(),
            flow_axes: FlowAxes::new(WritingMode::HorizontalTb, Direction::Ltr),
            gap: Size::ZERO,
            columns: None,
            rows: None,
            minimum_functions: GridMinimumParentFunctions {
                columns: None,
                rows: None,
            },
            algorithm: None,
            lane_grid_axis: None,
            allocated: LogicalSizeOf::new(None, None),
        },
        &parent,
        &parent,
        None,
    )
    .unwrap()
    .expect("eligible inherited axis is represented");
    let local = local_subgrid_entries(&[&[], &[], &[], &[]]);
    let lines = named::inherit_subgrid_named_lines(
        &axis.named_lines,
        axis.parent_start + 1,
        axis.parent_end + 1,
        axis.reversed,
        &local,
        axis.area_facts.as_ref(),
    )
    .unwrap();

    assert_eq!(axis.parent_start, 1);
    assert_eq!(axis.parent_end, 4);
    assert!(axis.reversed);
    assert_eq!(entry_names(lines.entries_on_line(1)), vec!["d"]);
    assert_eq!(entry_names(lines.entries_on_line(4)), vec!["b"]);
}

#[test]
fn subgrid_line_names_recompute_area_generated_names_clipped_to_span() {
    let areas = crate::GridTemplateAreas {
        rows: vec![crate::GridTemplateAreaRow {
            cells: vec![
                Some("a".to_string()),
                Some("a".to_string()),
                Some("a".to_string()),
                Some("a".to_string()),
            ],
        }],
    };
    let parent = named::add_area_generated_lines(
        GridAxisKind::Column,
        named::named_lines_from_track_components::<Scalar>(GridAxisKind::Column, &[], 4).unwrap(),
        &areas,
    )
    .unwrap();
    let local = local_subgrid_entries(&[&[], &[], &[]]);

    let lines =
        named::inherit_subgrid_named_lines(&parent, 2, 4, false, &local, Some(&parent.area_facts))
            .unwrap();

    assert_eq!(entry_names(lines.entries_on_line(1)), vec!["a-start"]);
    assert_eq!(entry_names(lines.entries_on_line(3)), vec!["a-end"]);
}

#[test]
fn subgrid_area_facts_preserve_reversed_orientation_and_axis_validity() {
    let areas = crate::GridTemplateAreas {
        rows: vec![crate::GridTemplateAreaRow {
            cells: vec![
                None,
                Some("main".to_string()),
                Some("main".to_string()),
                None,
            ],
        }],
    };
    let parent_lines = named::add_area_generated_lines(
        GridAxisKind::Column,
        named::named_lines_from_track_components::<Scalar>(GridAxisKind::Column, &[], 4).unwrap(),
        &areas,
    )
    .unwrap();
    let parent_context = GridParentContext {
        columns: Some(test_inherited_axis(
            parent_lines.clone(),
            Some(parent_lines.area_facts.clone()),
            1,
            3,
            true,
        )),
        rows: None,
    };

    let context = named::build_grid_named_context(
        &grid_container_projection!(&NodeInput {
            grid_template_columns: subgrid_track(),
            ..NodeInput::DEFAULT
        }),
        2,
        1,
        &parent_context,
    )
    .unwrap();
    let facts = context.area_facts.as_ref().unwrap();
    let rectangle = &facts.area_rectangles[0];

    assert_eq!(context.columns.named_occurrences("main-start"), vec![3]);
    assert_eq!(context.columns.named_occurrences("main-end"), vec![1]);
    assert!(facts.columns_valid);
    assert!(!facts.rows_valid);
    assert_eq!(rectangle.column_start, 1);
    assert_eq!(rectangle.column_end, 3);
    assert_eq!(rectangle.column_start_name, 3);
    assert_eq!(rectangle.column_end_name, 1);
}

#[test]
fn subgrid_local_area_facts_clamp_to_inherited_span() {
    let parent_context = GridParentContext {
        columns: Some(test_inherited_axis(
            named::named_lines_from_track_components::<Scalar>(GridAxisKind::Column, &[], 4)
                .unwrap(),
            None,
            0,
            2,
            false,
        )),
        rows: None,
    };

    let context = named::build_grid_named_context(
        &grid_container_projection!(&NodeInput {
            grid_template_columns: subgrid_track(),
            grid_template_areas: crate::GridTemplateAreas {
                rows: vec![crate::GridTemplateAreaRow {
                    cells: vec![
                        Some("wide".to_string()),
                        Some("wide".to_string()),
                        Some("wide".to_string()),
                        Some("wide".to_string()),
                    ],
                }],
            },
            ..NodeInput::DEFAULT
        }),
        2,
        1,
        &parent_context,
    )
    .unwrap();
    let facts = context.area_facts.as_ref().unwrap();
    let rectangle = &facts.area_rectangles[0];

    assert_eq!(context.columns.explicit_track_count, 2);
    assert_eq!(context.columns.named_occurrences("wide-start"), vec![1]);
    assert_eq!(context.columns.named_occurrences("wide-end"), vec![3]);
    assert_eq!(facts.column_count, 2);
    assert_eq!(rectangle.column_start, 1);
    assert_eq!(rectangle.column_end, 3);
}

#[test]
fn subgrid_duplicate_area_facts_merge_with_parent_clipped_boundaries() {
    let parent_areas = crate::GridTemplateAreas {
        rows: vec![crate::GridTemplateAreaRow {
            cells: vec![Some("same".to_string()), None, None, None],
        }],
    };
    let parent_lines = named::add_area_generated_lines(
        GridAxisKind::Column,
        named::named_lines_from_track_components::<Scalar>(GridAxisKind::Column, &[], 4).unwrap(),
        &parent_areas,
    )
    .unwrap();
    let parent_context = GridParentContext {
        columns: Some(test_inherited_axis(
            parent_lines.clone(),
            Some(parent_lines.area_facts.clone()),
            0,
            3,
            false,
        )),
        rows: None,
    };

    let context = named::build_grid_named_context(
        &grid_container_projection!(&NodeInput {
            grid_template_columns: subgrid_track(),
            grid_template_areas: crate::GridTemplateAreas {
                rows: vec![crate::GridTemplateAreaRow {
                    cells: vec![None, Some("same".to_string()), None],
                }],
            },
            ..NodeInput::DEFAULT
        }),
        3,
        1,
        &parent_context,
    )
    .unwrap();
    let facts = context.area_facts.as_ref().unwrap();
    let rectangle = &facts.area_rectangles[0];

    assert_eq!(context.columns.named_occurrences("same-start"), vec![1]);
    assert_eq!(context.columns.named_occurrences("same-end"), vec![2]);
    assert_eq!(rectangle.column_start, 1);
    assert_eq!(rectangle.column_end, 2);
}

#[test]
fn subgrid_named_placement_clamps_beyond_explicit_span() {
    let lines = named_parent_lines(2, &[&["a"], &[], &["a"]]);

    let placement = named::resolve_subgrid_placement(
        &lines,
        &RawGridPlacement::new(
            RawGridLine::NamedLine {
                name: "a".to_string(),
                index: -3,
            },
            RawGridLine::NamedLine {
                name: "a".to_string(),
                index: 4,
            },
        ),
        None,
        None,
    )
    .unwrap();

    assert_eq!(
        placement,
        GridPlacement::try_lines(1, 3).expect("valid grid lines")
    );
}

#[test]
fn subgrid_named_placement_resolves_wpt_line_names_before_clamping_to_span() {
    let parent = named_parent_lines(6, &[&["a"], &[], &[], &[], &["b"], &[], &["a", "b"]]);
    let local = local_subgrid_entries(&[&["x"], &["b"], &[], &[], &["b"]]);
    let lines = named::inherit_subgrid_named_lines(&parent, 2, 6, false, &local, None).unwrap();

    assert_eq!(lines.named_occurrences("b"), vec![2, 4, 5]);

    let cases = [
        (
            RawGridPlacement::new(
                RawGridLine::NamedLine {
                    name: "b".to_string(),
                    index: 1,
                },
                RawGridLine::NamedLine {
                    name: "b".to_string(),
                    index: 2,
                },
            ),
            GridPlacement::try_lines(2, 4).expect("valid grid lines"),
        ),
        (
            RawGridPlacement::new(
                RawGridLine::NamedLine {
                    name: "b".to_string(),
                    index: 1,
                },
                RawGridLine::NamedLine {
                    name: "b".to_string(),
                    index: -2,
                },
            ),
            GridPlacement::try_lines(2, 4).expect("valid grid lines"),
        ),
        (
            RawGridPlacement::new(
                RawGridLine::NamedLine {
                    name: "b".to_string(),
                    index: -2,
                },
                RawGridLine::NamedLine {
                    name: "b".to_string(),
                    index: 1,
                },
            ),
            GridPlacement::try_lines(2, 4).expect("valid grid lines"),
        ),
        (
            RawGridPlacement::new(
                RawGridLine::NamedLine {
                    name: "b".to_string(),
                    index: -2,
                },
                RawGridLine::NamedSpan {
                    name: "b".to_string(),
                    index: 2,
                },
            ),
            GridPlacement::try_lines(4, 5).expect("valid grid lines"),
        ),
    ];

    for (raw, expected) in cases {
        let placement = named::resolve_subgrid_placement(&lines, &raw, None, None).unwrap();
        assert_eq!(placement, expected, "raw placement {raw:?}");
    }
}

#[test]
fn subgrid_named_placement_resolves_wpt_named_spans_before_clamping_to_span() {
    let parent = named_parent_lines(6, &[&["a"], &[], &[], &[], &["b"], &[], &["a", "b"]]);
    let local = local_subgrid_entries(&[&["x"], &["b"], &[], &[], &["b"]]);
    let lines = named::inherit_subgrid_named_lines(&parent, 2, 6, false, &local, None).unwrap();

    let cases = [
        (
            RawGridPlacement::new(
                RawGridLine::NamedLine {
                    name: "b".to_string(),
                    index: 1,
                },
                RawGridLine::NamedSpan {
                    name: "b".to_string(),
                    index: 1,
                },
            ),
            GridPlacement::try_lines(2, 4).expect("valid grid lines"),
        ),
        (
            RawGridPlacement::new(
                RawGridLine::NamedSpan {
                    name: "b".to_string(),
                    index: 1,
                },
                RawGridLine::NamedLine {
                    name: "b".to_string(),
                    index: 2,
                },
            ),
            GridPlacement::try_lines(2, 4).expect("valid grid lines"),
        ),
        (
            RawGridPlacement::new(
                RawGridLine::NamedSpan {
                    name: "b".to_string(),
                    index: 2,
                },
                RawGridLine::NamedLine {
                    name: "b".to_string(),
                    index: 2,
                },
            ),
            GridPlacement::try_lines(1, 4).expect("valid grid lines"),
        ),
        (
            RawGridPlacement::new(
                RawGridLine::NamedSpan {
                    name: "b".to_string(),
                    index: 1,
                },
                RawGridLine::NamedLine {
                    name: "b".to_string(),
                    index: -2,
                },
            ),
            GridPlacement::try_lines(2, 4).expect("valid grid lines"),
        ),
    ];

    for (raw, expected) in cases {
        let placement = named::resolve_subgrid_placement(&lines, &raw, None, None).unwrap();
        assert_eq!(placement, expected, "raw placement {raw:?}");
    }
}

#[test]
fn subgrid_named_placement_expands_collapsed_clamp_to_edge_track() {
    let lines = named::NamedGridLines::new(GridAxisKind::Column, 1);

    let placement = named::resolve_subgrid_placement(
        &lines,
        &RawGridPlacement::new(RawGridLine::Line(2), RawGridLine::Span(3)),
        None,
        None,
    )
    .unwrap();

    assert_eq!(
        placement,
        GridPlacement::try_lines(1, 2).expect("valid grid lines")
    );
}

#[test]
fn subgrid_named_span_counts_implicit_names_beyond_end_before_clamping() {
    let lines = named::NamedGridLines::new(GridAxisKind::Column, 10);

    let placement = named::resolve_subgrid_placement(
        &lines,
        &RawGridPlacement::new(
            RawGridLine::NamedSpan {
                name: "a".to_string(),
                index: 1,
            },
            RawGridLine::NamedLine {
                name: "a".to_string(),
                index: 8,
            },
        ),
        None,
        None,
    )
    .unwrap();

    assert_eq!(
        placement,
        GridPlacement::try_lines(10, 11).expect("valid grid lines")
    );
}

fn named_parent_lines(
    explicit_track_count: usize,
    line_names: &[&[&str]],
) -> named::NamedGridLines {
    let mut lines = named::NamedGridLines::new(GridAxisKind::Column, explicit_track_count);
    for (line_index, names) in line_names.iter().enumerate() {
        lines.line_names[line_index] = names
            .iter()
            .map(|name| named::LineNameEntry {
                name: (*name).to_string(),
                origin: named::LineNameOrigin::Explicit,
            })
            .collect();
    }
    lines
}

fn test_inherited_axis(
    named_lines: named::NamedGridLines,
    area_facts: Option<named::GridAreaNameFacts>,
    parent_start: usize,
    parent_end: usize,
    reversed: bool,
) -> InheritedGridAxis {
    let track_count = parent_end - parent_start;
    InheritedGridAxis {
        baseline_reference: crate::GridBaselineReference::Physical,
        offset: 0.0,
        gap: 0.0,
        tracks: vec![0.0; track_count],
        minimum_functions: None,
        geometry: UsedGridAxisGeometryOf::new(
            vec![0.0; track_count],
            vec![false; track_count],
            0.0,
        ),
        named_lines,
        area_facts,
        template_area_expanded: false,
        owner_baseline_targets: None,
        parent_start,
        parent_end,
        reversed,
    }
}

fn local_subgrid_entries(line_names: &[&[&str]]) -> Vec<Vec<named::LineNameEntry>> {
    line_names
        .iter()
        .map(|names| {
            names
                .iter()
                .map(|name| named::LineNameEntry {
                    name: (*name).to_string(),
                    origin: named::LineNameOrigin::LocalSubgrid,
                })
                .collect()
        })
        .collect()
}

#[test]
fn vertical_subgrid_percentage_gap_uses_flow_relative_axis_basis() {
    let style = NodeInput {
        writing_mode: WritingMode::VerticalLr,
        gap: Size::new(Length::percent(0.10), Length::percent(0.10)),
        ..NodeInput::default()
    };
    let area_size = Size::new(300.0, 500.0);

    assert_eq!(
        child_subgrid_gap(&style, GridAxisKind::Column, area_size),
        Ok(ResolvedSubgridGap::Length(50.0))
    );
    assert_eq!(
        child_subgrid_gap(&style, GridAxisKind::Row, area_size),
        Ok(ResolvedSubgridGap::Length(30.0))
    );
}

// Build the native parent and all borrowed track/placement authority through
// the same canonical owners used by real Grid. The source declares exactly
// the200/1 inline and100/1 block track profile; the extra1px track may overflow.
fn collect_percentage_edge_parent_traversal<'input>(
    tree: &'input mut OracleTree,
    axis: GridAxisKind,
) -> SubgridTraversalReport<'input, u32> {
    let mut compute = tree.borrow_compute();
    let parent_style = grid_container_projection!(compute.node_input(1));
    let flow = FlowAxes::new(parent_style.writing_mode, parent_style.direction);
    let available = Size::new(
        AxisSizingRequestOf::new(
            AvailableSpaceOf::Definite(crate::NonNegativeFiniteOf::new(100.0).unwrap()),
            SizingOperation::Normal,
        ),
        AxisSizingRequestOf::new(
            AvailableSpaceOf::Definite(crate::NonNegativeFiniteOf::new(200.0).unwrap()),
            SizingOperation::Normal,
        ),
    );
    let parent_input = ComputeInput::for_child(
        RunMode::ComputeSize,
        SizingMode::InherentSize,
        RequestedAxis::Both,
        Size::NONE,
        Size::NONE,
        ContainingLayoutContext::new(flow, ParentFormattingContext::NoParent),
        SizingRequestOf::new(
            available,
            OrthogonalFallbackOf::new(
                Size::NONE,
                crate::NearestScrollportOf::Absent,
                Size::splat(crate::OrthogonalFallbackLimitOf::Unbounded),
            ),
        ),
    );
    let constants =
        Constants::new::<_, core::convert::Infallible>(&compute, 1, &parent_style, parent_input)
            .expect("the registered native parent resolves its authored size");
    assert_eq!(
        constants.node_inner_size,
        Size::new(Some(100.0), Some(200.0))
    );
    let parent_context = GridParentContext::none();
    let initialized = initialize_grid_tracks::<_, core::convert::Infallible>(
        &mut compute,
        1,
        &parent_style,
        &constants,
        &parent_context,
        parent_input,
        None,
    )
    .expect("the real parent admits its actual child and original track functions");
    let children = compute.children(1).collect::<Vec<_>>();
    assert_eq!(children, [2]);
    let columns = [200.0, 1.0];
    let rows = [100.0, 1.0];
    let placed_areas = resolve_grid_child_areas(ResolveGridChildAreasInput {
        children: &children,
        placements: &initialized.placements,
        style: &parent_style,
        columns: &columns,
        rows: &rows,
        gap: initialized.context.gap,
        lines: initialized.context.lines,
    });
    let area = placed_areas[0].expect("the first-track child is placed");
    assert_eq!(area.size, LogicalSizeOf::new(200.0, 100.0));
    let grid = IntrinsicGrid {
        style: &parent_style,
        constants: &constants,
        sizing_flow_axes: flow,
        track_basis: initialized.context.percent_basis,
        minimum_functions: minimum_parent_functions(
            &initialized.column_tracks,
            &initialized.row_tracks,
            initialized.context.percent_basis,
            &parent_context,
        ),
        column_tracks: &initialized.column_tracks,
        row_tracks: &initialized.row_tracks,
        gap: initialized.context.gap,
        column_gutters: Some(&initialized.context.column_gutters),
        row_gutters: Some(&initialized.context.row_gutters),
        lines: initialized.context.lines,
        named_columns: &initialized.context.named_columns,
        named_rows: &initialized.context.named_rows,
        area_facts: initialized.context.area_facts.as_ref(),
        subgrid_report: &initialized.subgrid_report,
        placements: &initialized.placements,
        query: IntrinsicGridQuery::complete(GridContributionPurpose::MaxContent),
    };
    let grid = intrinsic_stage::intrinsic_role_grid(
        grid,
        intrinsic_stage::IntrinsicStageFrame {
            axis,
            stage: GridSizingStage::COMPLETE,
            columns: Some(&columns),
            rows: Some(&rows),
            observed_axis: None,
        },
        GridContributionPurpose::MaxContent,
    )
    .unwrap();
    // These authored profiles contain only fixed or min-content minima.
    // Preserve true intrinsic eligibility; fixed rows do not acquire it.
    let queried_tracks = match axis {
        GridAxisKind::Column => &initialized.column_tracks,
        GridAxisKind::Row => &initialized.row_tracks,
    };
    let intrinsic = queried_tracks
        .iter()
        .map(|track| match &track.min {
            MinTrackSizing::MinContent => true,
            MinTrackSizing::Calculation(_) => false,
            _ => unreachable!("the declared parent has only the qualified two minima"),
        })
        .collect::<Vec<_>>();
    collect_grid_subgrid_intrinsic_traversal::<_, core::convert::Infallible>(
        &mut compute,
        GridSubgridIntrinsicTraversalInput {
            grid,
            axis,
            containing_flow_axes: flow,
            children: &children,
            placed_areas: &placed_areas,
            placements: &initialized.placements,
            subgrid_report: &initialized.subgrid_report,
            named_columns: &initialized.context.named_columns,
            named_rows: &initialized.context.named_rows,
            area_facts: initialized.context.area_facts.as_ref(),
            parent_gap: Size::new(
                initialized.context.gap.inline,
                initialized.context.gap.block,
            ),
            column_gutters: Some(&initialized.context.column_gutters),
            row_gutters: Some(&initialized.context.row_gutters),
            column_sizes: &columns,
            row_sizes: &rows,
            container_size: constants.node_inner_size,
            intrinsic_min_track_facts: IntrinsicMinTrackFacts::Known(&intrinsic),
            query: grid.query,
        },
    )
    .expect("the actual prepared property frames resolve")
    .expect("the source-declared inherited axis is eligible")
}

#[test]
fn vertical_subgrid_percentage_edges_use_physical_area_basis() {
    let parent_style = NodeInput {
        display: Display::Grid,
        writing_mode: WritingMode::VerticalRl,
        size: Size::new(PreferredSize::px(100.0), PreferredSize::px(200.0)),
        grid_template_columns: vec![
            TrackComponent::minmax(MinTrackSizing::MIN_CONTENT, MaxTrackSizing::px(200.0)),
            TrackComponent::px(1.0),
        ],
        grid_template_rows: vec![TrackComponent::px(100.0), TrackComponent::px(1.0)],
        justify_content: Some(AlignContent::Start),
        align_content: Some(AlignContent::Start),
        ..NodeInput::default()
    };
    let child_style = NodeInput {
        display: Display::Grid,
        writing_mode: WritingMode::VerticalRl,
        grid_template_columns: subgrid_track(),
        padding: Edges::all(Length::percent(0.1)),
        // The real200-high border makes the min-content parent track200,
        // while10% padding retains its 200-high containing basis.
        box_sizing: BoxSizing::BorderBox,
        size: Size::new(PreferredSize::AUTO, PreferredSize::px(200.0)),
        ..NodeInput::default()
    };
    let mut tree = OracleTree::new()
        .children(1, [2])
        .children(2, [3])
        .children(3, [])
        .style(1, parent_style)
        .style(2, child_style)
        .style(3, NodeInput::default());
    let report = collect_percentage_edge_parent_traversal(&mut tree, GridAxisKind::Column);

    assert_eq!(report.edge_lower_bounds, vec![40.0, 0.0]);
    assert_eq!(report.edge_adjustments_for_leaf(0), vec![40.0, 0.0]);
}

#[test]
fn orthogonal_subgrid_percentage_edges_use_containing_physical_area_basis() {
    let parent_style = NodeInput {
        display: Display::Grid,
        writing_mode: WritingMode::VerticalRl,
        size: Size::new(PreferredSize::px(100.0), PreferredSize::px(200.0)),
        grid_template_columns: vec![
            TrackComponent::minmax(MinTrackSizing::MIN_CONTENT, MaxTrackSizing::px(200.0)),
            TrackComponent::px(1.0),
        ],
        grid_template_rows: vec![TrackComponent::px(100.0), TrackComponent::px(1.0)],
        justify_content: Some(AlignContent::Start),
        align_content: Some(AlignContent::Start),
        ..NodeInput::default()
    };
    let child_style = NodeInput {
        display: Display::Grid,
        grid_template_rows: subgrid_track(),
        padding: Edges::all(Length::percent(0.1)),
        // The real200-high border makes the min-content parent track200,
        // while10% padding retains its 200-high containing basis.
        box_sizing: BoxSizing::BorderBox,
        size: Size::new(PreferredSize::AUTO, PreferredSize::px(200.0)),
        ..NodeInput::default()
    };
    let mut tree = OracleTree::new()
        .children(1, [2])
        .children(2, [3])
        .children(3, [])
        .style(1, parent_style)
        .style(2, child_style)
        .style(3, NodeInput::default());
    let report = collect_percentage_edge_parent_traversal(&mut tree, GridAxisKind::Column);

    assert_eq!(report.edge_lower_bounds, vec![40.0, 0.0]);
    assert_eq!(report.edge_adjustments_for_leaf(0), vec![40.0, 0.0]);
}

#[test]
fn nested_subgrid_same_flow_projects_physical_edge_sums_before_local_track_sizing() {
    let parent_style = NodeInput {
        display: Display::Grid,
        writing_mode: WritingMode::VerticalRl,
        size: Size::new(PreferredSize::px(100.0), PreferredSize::px(200.0)),
        grid_template_columns: vec![TrackComponent::px(200.0), TrackComponent::px(1.0)],
        grid_template_rows: vec![TrackComponent::px(100.0), TrackComponent::px(1.0)],
        justify_content: Some(AlignContent::Start),
        align_content: Some(AlignContent::Start),
        ..NodeInput::default()
    };
    let outer_style = NodeInput {
        display: Display::Grid,
        writing_mode: WritingMode::VerticalRl,
        grid_template_rows: subgrid_track(),
        grid_template_columns: vec![TrackComponent::percent(1.0)],
        // Actual border height190 is the200-high area minus margins3+7.
        // It establishes a source-owned property frame, not an inline estimate.
        box_sizing: BoxSizing::BorderBox,
        size: Size::new(PreferredSize::AUTO, PreferredSize::px(190.0)),
        margin: Edges::new(
            LengthAuto::px(3.0),
            LengthAuto::px(5.0),
            LengthAuto::px(7.0),
            LengthAuto::px(11.0),
        ),
        border: Edges::new(
            Length::px(13.0),
            Length::px(17.0),
            Length::px(19.0),
            Length::px(23.0),
        ),
        padding: Edges::new(
            Length::px(29.0),
            Length::px(31.0),
            Length::px(37.0),
            Length::px(41.0),
        ),
        ..NodeInput::default()
    };
    let inner_style = NodeInput {
        display: Display::Grid,
        writing_mode: WritingMode::VerticalRl,
        grid_template_rows: subgrid_track(),
        grid_template_columns: vec![TrackComponent::percent(1.0)],
        // Outer190 minus top/bottom border+padding13+19+29+37 gives92.
        box_sizing: BoxSizing::BorderBox,
        size: Size::new(PreferredSize::AUTO, PreferredSize::px(92.0)),
        padding: Edges::new(
            Length::percent(0.1),
            Length::percent(0.2),
            Length::percent(0.3),
            Length::percent(0.4),
        ),
        ..NodeInput::default()
    };
    let mut tree = OracleTree::new()
        .children(1, [2])
        .children(2, [3])
        .children(3, [4])
        .children(4, [])
        .style(1, parent_style)
        .style(2, outer_style)
        .style(3, inner_style)
        .style(4, NodeInput::default());
    let report = collect_percentage_edge_parent_traversal(&mut tree, GridAxisKind::Row);

    assert_eq!(report.leaves.len(), 1);
    let leaf = &report.leaves[0];
    assert_eq!(leaf.node, 4);
    let frame = leaf
        .min_frame
        .expect("the immediate parent owns the query frame");
    let prepared = frame.prepared();
    let flow = FlowAxes::new(WritingMode::VerticalRl, Direction::Ltr);
    assert_eq!(frame.flow_axes, flow);
    assert_eq!(frame.axis, GridAxisKind::Row);
    assert_eq!(frame.algorithm, SizingAlgorithm::Grid);
    assert_eq!(prepared.containing_flow_axes(), flow);
    // This replaces the former detached Auto estimate with authored known
    // geometry. Preserve the percentage equation in the represented lane.
    let outer_border: Scalar = 200.0 - (3.0 + 7.0);
    let inner_border = outer_border - (13.0 + 19.0 + 29.0 + 37.0);
    let nearest_content = inner_border - (0.1 * inner_border + 0.3 * inner_border);
    assert_eq!(outer_border, 190.0);
    assert_eq!(inner_border, 92.0);
    assert_eq!(
        prepared.parent().height,
        Some(nearest_content),
        "the property basis comes from the nearest parent's real content box"
    );
    assert_eq!(
        prepared.known().height,
        Some(nearest_content),
        "only the opposite-axis stretch is prepared"
    );
    assert_eq!(
        prepared.sizing_request().axes().height,
        AxisSizingRequestOf::new(
            AvailableSpaceOf::Definite(crate::NonNegativeFiniteOf::new(nearest_content).unwrap()),
            SizingOperation::Normal
        )
    );
    assert_eq!(
        prepared.sizing_request().axes().width,
        AxisSizingRequestOf::new(AvailableSpaceOf::Indefinite, SizingOperation::MaxContent),
        "the queried physicalX axis retains its actual MaxContent role"
    );
}

#[test]
fn vertical_grid_axis_offsets_add_local_inset_to_inherited_offsets() {
    let style = NodeInput {
        writing_mode: WritingMode::VerticalLr,
        ..NodeInput::default()
    };
    let tracks = [20.0, 30.0];
    let alignment = GridAlignment {
        start: 7.0,
        gap: 5.0,
    };
    let content_box_inset = Edges {
        left: 11.0,
        right: 0.0,
        top: 13.0,
        bottom: 0.0,
    };
    let geometry = UsedGridAxisGeometryOf::new(tracks.to_vec(), vec![false; tracks.len()], 5.0);
    let style_projection = grid_container_projection!(&style);

    let column_offsets = grid_axis_offsets(GridAxisOffsetsInput {
        style: &style_projection,
        axis: GridAxisKind::Column,
        tracks: &tracks,
        geometry: &geometry,
        inherited_offset: Some(100.0),
        content_box_left: 0.0,
        content_box_size: Size::new(300.0, 400.0),
        content_box_inset,
        alignment,
    });
    let row_offsets = grid_axis_offsets(GridAxisOffsetsInput {
        style: &style_projection,
        axis: GridAxisKind::Row,
        tracks: &tracks,
        geometry: &geometry,
        inherited_offset: Some(200.0),
        content_box_left: 0.0,
        content_box_size: Size::new(300.0, 400.0),
        content_box_inset,
        alignment,
    });

    assert_eq!(column_offsets, vec![120.0, 145.0]);
    assert_eq!(row_offsets, vec![218.0, 243.0]);
}

#[test]
fn subgrid_eligibility_reports_first_blocking_reason() {
    let report = subgrid_eligibility(SubgridEligibilityInput {
        axis: GridAxisKind::Column,
        parent_style: &NodeInput {
            display: Display::Grid,
            ..NodeInput::default()
        },
        has_parent_grid: false,
        child_style: &NodeInput {
            display: Display::Block,
            position: Position::Absolute,
            grid_template_columns: subgrid_track(),
            ..NodeInput::default()
        },
    });

    assert_eq!(
        report,
        SubgridEligibility::Excluded(SubgridIneligibleReason::NoParentGrid)
    );
}

#[test]
fn subgrid_eligibility_rejects_non_grid_container_display() {
    let report = subgrid_eligibility(SubgridEligibilityInput {
        axis: GridAxisKind::Column,
        parent_style: &NodeInput {
            display: Display::Grid,
            ..NodeInput::default()
        },
        has_parent_grid: true,
        child_style: &NodeInput {
            display: Display::Block,
            grid_template_columns: subgrid_track(),
            ..NodeInput::default()
        },
    });

    assert_eq!(
        report,
        SubgridEligibility::Excluded(SubgridIneligibleReason::UnsupportedDisplay)
    );
}

#[test]
fn subgrid_axis_report_allows_supported_vertical_parent_mapping_to_inherit() {
    let report = subgrid_axis_report(
        &NodeInput {
            display: Display::Grid,
            writing_mode: WritingMode::VerticalRl,
            ..NodeInput::default()
        },
        &NodeInput {
            display: Display::Grid,
            grid_template_columns: subgrid_track(),
            ..NodeInput::default()
        },
        GridAxisKind::Column,
    );

    assert!(report.can_inherit());
    assert_eq!(
        report.mapping,
        GridAxisMappingReport {
            queried_axis: GridAxisKind::Column,
            parent_axis: GridAxisKind::Row,
            child_axis: GridAxisKind::Column,
            reversed: true,
        }
    );
    assert!(report.can_inherit());
}

fn subgrid_item_report(parent: &NodeInput, child: &NodeInput) -> SubgridItemReport<()> {
    SubgridItemReport {
        node: (),
        column: subgrid_axis_report(parent, child, GridAxisKind::Column),
        row: subgrid_axis_report(parent, child, GridAxisKind::Row),
    }
}

fn grid_area(column: usize, column_end: usize, row: usize, row_end: usize) -> GridArea {
    GridArea {
        column,
        column_end,
        row,
        row_end,
        size: LogicalSizeOf::new(0.0, 0.0),
    }
}

#[test]
fn intrinsic_subgrid_context_is_needed_for_both_axis_subgrids() {
    let parent = NodeInput {
        display: Display::Grid,
        ..NodeInput::default()
    };
    let child = NodeInput {
        display: Display::Grid,
        grid_auto_flow: GridAutoFlow::Row,
        grid_template_columns: subgrid_track(),
        grid_template_rows: subgrid_track(),
        ..NodeInput::default()
    };

    assert!(needs_intrinsic_subgrid_context(
        &child,
        subgrid_item_report(&parent, &child),
        grid_area(0, 3, 0, 2),
    ));
}

#[test]
fn intrinsic_subgrid_context_is_not_needed_for_single_column_both_axis_subgrid() {
    let parent = NodeInput {
        display: Display::Grid,
        ..NodeInput::default()
    };
    let child = NodeInput {
        display: Display::Grid,
        grid_auto_flow: GridAutoFlow::Row,
        grid_template_columns: subgrid_track(),
        grid_template_rows: subgrid_track(),
        ..NodeInput::default()
    };

    assert!(!needs_intrinsic_subgrid_context(
        &child,
        subgrid_item_report(&parent, &child),
        grid_area(0, 1, 0, 2),
    ));
}

#[test]
fn intrinsic_subgrid_context_is_needed_for_row_subgrid_with_percent_columns() {
    let parent = NodeInput {
        display: Display::Grid,
        ..NodeInput::default()
    };
    let child = NodeInput {
        display: Display::Grid,
        grid_auto_flow: GridAutoFlow::Row,
        grid_template_columns: vec![TrackComponent::percent(0.5)],
        grid_template_rows: subgrid_track(),
        ..NodeInput::default()
    };

    assert!(needs_intrinsic_subgrid_context(
        &child,
        subgrid_item_report(&parent, &child),
        grid_area(0, 1, 0, 2),
    ));
}

#[test]
fn intrinsic_subgrid_context_uses_mapped_parent_axis_for_orthogonal_subgrid() {
    let parent = NodeInput {
        display: Display::Grid,
        writing_mode: WritingMode::VerticalRl,
        ..NodeInput::default()
    };
    let child = NodeInput {
        display: Display::Grid,
        grid_auto_flow: GridAutoFlow::Column,
        grid_template_columns: subgrid_track(),
        ..NodeInput::default()
    };

    assert!(needs_intrinsic_subgrid_context(
        &child,
        subgrid_item_report(&parent, &child),
        grid_area(0, 1, 0, 2),
    ));
}

#[test]
fn subgrid_eligibility_classifies_grid_lanes_parent_stacking_axis() {
    let report = subgrid_eligibility(SubgridEligibilityInput {
        axis: GridAxisKind::Row,
        parent_style: &NodeInput {
            display: Display::GridLanes,
            ..NodeInput::default()
        },
        has_parent_grid: true,
        child_style: &NodeInput {
            display: Display::Grid,
            grid_template_rows: subgrid_track(),
            ..NodeInput::default()
        },
    });

    assert_eq!(report, SubgridEligibility::Stacking);
}

#[test]
fn subgrid_eligibility_allows_grid_lanes_parent_in_grid_axis() {
    let report = subgrid_eligibility(SubgridEligibilityInput {
        axis: GridAxisKind::Column,
        parent_style: &NodeInput {
            display: Display::GridLanes,
            ..NodeInput::default()
        },
        has_parent_grid: true,
        child_style: &NodeInput {
            display: Display::Grid,
            grid_template_columns: subgrid_track(),
            ..NodeInput::default()
        },
    });

    assert_eq!(report, SubgridEligibility::SharedTracks);
}

#[test]
fn subgrid_eligibility_treats_inline_grid_lanes_parent_as_lanes() {
    let stacking = subgrid_eligibility(SubgridEligibilityInput {
        axis: GridAxisKind::Row,
        parent_style: &NodeInput {
            display: Display::InlineGridLanes,
            ..NodeInput::default()
        },
        has_parent_grid: true,
        child_style: &NodeInput {
            display: Display::InlineGrid,
            grid_template_rows: subgrid_track(),
            ..NodeInput::default()
        },
    });

    assert_eq!(stacking, SubgridEligibility::Stacking);

    let allowed = subgrid_eligibility(SubgridEligibilityInput {
        axis: GridAxisKind::Column,
        parent_style: &NodeInput {
            display: Display::InlineGridLanes,
            ..NodeInput::default()
        },
        has_parent_grid: true,
        child_style: &NodeInput {
            display: Display::InlineGrid,
            grid_template_columns: subgrid_track(),
            ..NodeInput::default()
        },
    });

    assert_eq!(allowed, SubgridEligibility::SharedTracks);
}

#[test]
fn subgrid_eligibility_allows_ordinary_grid_parent_in_both_axes() {
    let parent = NodeInput {
        display: Display::Grid,
        ..NodeInput::default()
    };
    let child = NodeInput {
        display: Display::Grid,
        grid_template_columns: subgrid_track(),
        grid_template_rows: subgrid_track(),
        ..NodeInput::default()
    };

    for axis in [GridAxisKind::Column, GridAxisKind::Row] {
        let report = subgrid_eligibility(SubgridEligibilityInput {
            axis,
            parent_style: &parent,
            has_parent_grid: true,
            child_style: &child,
        });

        assert_eq!(
            report,
            SubgridEligibility::SharedTracks,
            "{axis:?} subgrid should be eligible"
        );
    }
}

#[test]
fn subgrid_track_inheritance_copies_parent_columns_for_span() {
    let report = inherit_subgrid_tracks(SubgridTrackInheritanceInput {
        parent_tracks: &[40.0, 60.0, 90.0],
        parent_span: GridTrackSpan::new(2, 4),
        reversed: false,
        start_mbp: 0.0,
        end_mbp: 0.0,
        parent_gap: 10.0,
        subgrid_gap: ResolvedSubgridGap::Length(10.0),
    })
    .unwrap();

    assert_eq!(report.copied_parent_tracks, vec![60.0, 90.0]);
    assert_eq!(report.final_tracks, vec![60.0, 90.0]);
}

#[test]
fn subgrid_track_inheritance_reverses_copied_columns_before_mbp_consumption() {
    let report = inherit_subgrid_tracks(SubgridTrackInheritanceInput {
        parent_tracks: &[40.0, 60.0, 90.0],
        parent_span: GridTrackSpan::new(1, 4),
        reversed: true,
        start_mbp: 10.0,
        end_mbp: 20.0,
        parent_gap: 10.0,
        subgrid_gap: ResolvedSubgridGap::Length(10.0),
    })
    .unwrap();

    assert_eq!(report.after_reversal, vec![90.0, 60.0, 40.0]);
    assert_eq!(report.final_tracks, vec![80.0, 60.0, 20.0]);
}

#[test]
fn subgrid_track_inheritance_consumes_start_and_end_mbp_across_tracks() {
    let report = inherit_subgrid_tracks(SubgridTrackInheritanceInput {
        parent_tracks: &[5.0, 20.0, 10.0],
        parent_span: GridTrackSpan::new(1, 4),
        reversed: false,
        start_mbp: 12.0,
        end_mbp: 25.0,
        parent_gap: 0.0,
        subgrid_gap: ResolvedSubgridGap::Length(0.0),
    })
    .unwrap();

    assert_eq!(report.start_mbp_removed, vec![0.0, 13.0, 10.0]);
    assert_eq!(report.end_mbp_removed, vec![0.0, 0.0, 0.0]);
    assert_eq!(report.final_tracks, vec![0.0, 0.0, 0.0]);
}

#[test]
fn subgrid_track_inheritance_resolves_normal_gap_to_parent_gap() {
    let report = inherit_subgrid_tracks(SubgridTrackInheritanceInput {
        parent_tracks: &[50.0, 50.0],
        parent_span: GridTrackSpan::new(1, 3),
        reversed: false,
        start_mbp: 0.0,
        end_mbp: 0.0,
        parent_gap: 20.0,
        subgrid_gap: ResolvedSubgridGap::Normal,
    })
    .unwrap();

    assert_eq!(report.resolved_subgrid_gap, 20.0);
    assert_eq!(report.gap_difference, 0.0);
    assert_eq!(report.final_tracks, vec![50.0, 50.0]);
}

#[test]
fn subgrid_track_inheritance_applies_explicit_gap_difference_to_internal_edges() {
    let report = inherit_subgrid_tracks(SubgridTrackInheritanceInput {
        parent_tracks: &[50.0, 50.0, 50.0],
        parent_span: GridTrackSpan::new(1, 4),
        reversed: false,
        start_mbp: 0.0,
        end_mbp: 0.0,
        parent_gap: 10.0,
        subgrid_gap: ResolvedSubgridGap::Length(20.0),
    })
    .unwrap();

    assert_eq!(report.gap_difference, 5.0);
    assert_eq!(report.final_tracks, vec![45.0, 40.0, 45.0]);
}

#[test]
fn column_subgrid_layout_tracks_expand_collapsed_tracks_into_shifted_lines() {
    let report = inherit_subgrid_tracks(SubgridTrackInheritanceInput {
        parent_tracks: &[100.0, 100.0, 100.0],
        parent_span: GridTrackSpan::new(1, 4),
        reversed: false,
        start_mbp: 0.0,
        end_mbp: 0.0,
        parent_gap: 0.0,
        subgrid_gap: ResolvedSubgridGap::Length(150.0),
    })
    .unwrap();

    let (tracks, gap) = inherited_subgrid_layout_tracks(GridAxisKind::Column, &report);

    assert_eq!(report.final_tracks, vec![25.0, 0.0, 25.0]);
    assert_eq!(tracks, vec![175.0, 100.0, 25.0]);
    assert_eq!(gap, 0.0);
}

#[test]
fn row_subgrid_layout_tracks_keep_collapsed_tracks_with_resolved_gap() {
    let report = inherit_subgrid_tracks(SubgridTrackInheritanceInput {
        parent_tracks: &[100.0, 100.0, 100.0],
        parent_span: GridTrackSpan::new(1, 4),
        reversed: false,
        start_mbp: 0.0,
        end_mbp: 0.0,
        parent_gap: 0.0,
        subgrid_gap: ResolvedSubgridGap::Length(150.0),
    })
    .unwrap();

    let (tracks, gap) = inherited_subgrid_layout_tracks(GridAxisKind::Row, &report);

    assert_eq!(report.final_tracks, vec![25.0, 0.0, 25.0]);
    assert_eq!(tracks, vec![25.0, 0.0, 25.0]);
    assert_eq!(gap, 150.0);
}

#[test]
fn subgrid_layout_tracks_keep_non_collapsed_gap_sizing() {
    let report = inherit_subgrid_tracks(SubgridTrackInheritanceInput {
        parent_tracks: &[100.0, 100.0],
        parent_span: GridTrackSpan::new(1, 3),
        reversed: false,
        start_mbp: 0.0,
        end_mbp: 0.0,
        parent_gap: 20.0,
        subgrid_gap: ResolvedSubgridGap::Length(100.0),
    })
    .unwrap();

    let (tracks, gap) = inherited_subgrid_layout_tracks(GridAxisKind::Column, &report);

    assert_eq!(report.final_tracks, vec![60.0, 60.0]);
    assert_eq!(tracks, vec![60.0, 60.0]);
    assert_eq!(gap, 100.0);
}

#[test]
fn subgrid_track_inheritance_expands_tracks_for_smaller_subgrid_gap() {
    let report = inherit_subgrid_tracks(SubgridTrackInheritanceInput {
        parent_tracks: &[40.0, 40.0],
        parent_span: GridTrackSpan::new(1, 3),
        reversed: false,
        start_mbp: 0.0,
        end_mbp: 0.0,
        parent_gap: 20.0,
        subgrid_gap: ResolvedSubgridGap::Length(10.0),
    })
    .unwrap();

    assert_eq!(report.gap_difference, -5.0);
    assert_eq!(report.final_tracks, vec![45.0, 45.0]);
}

#[test]
fn subgrid_track_inheritance_rejects_empty_parent_tracks() {
    let err = inherit_subgrid_tracks(SubgridTrackInheritanceInput {
        parent_tracks: &[],
        parent_span: GridTrackSpan::new(1, 2),
        reversed: false,
        start_mbp: 0.0,
        end_mbp: 0.0,
        parent_gap: 0.0,
        subgrid_gap: ResolvedSubgridGap::Normal,
    })
    .unwrap_err();

    assert_eq!(err, SubgridTrackInheritanceError::EmptyTrackList);
}

#[test]
fn subgrid_track_inheritance_rejects_invalid_parent_spans() {
    for span in [
        GridTrackSpan::new(0, 2),
        GridTrackSpan::new(2, 2),
        GridTrackSpan::new(3, 2),
        GridTrackSpan::new(1, 4),
    ] {
        let err = inherit_subgrid_tracks(SubgridTrackInheritanceInput {
            parent_tracks: &[10.0, 20.0],
            parent_span: span,
            reversed: false,
            start_mbp: 0.0,
            end_mbp: 0.0,
            parent_gap: 0.0,
            subgrid_gap: ResolvedSubgridGap::Normal,
        })
        .unwrap_err();

        assert_eq!(err, SubgridTrackInheritanceError::SpanOutOfRange);
    }
}

fn traversal_subgrid<'input>(
    default_input: &'input NodeInput,
    node: u32,
    start: usize,
    end: usize,
    children: Vec<SubgridTraversalChild<'input, u32>>,
) -> SubgridTraversalChild<'input, u32> {
    SubgridTraversalChild::Subgrid(SubgridTraversalNode {
        min_frame: None,
        node,
        style: default_grid_item_projection(default_input),
        axis: SubgridTraversalAxis::Inherited,
        reversed: false,
        span_in_parent: GridTrackSpan::new(start, end),
        align_self: AlignItems::Stretch,
        standalone_parent_context: None,
        margins: SubgridAxisEdges::default(),
        border: SubgridAxisEdges::default(),
        padding: SubgridAxisEdges::default(),
        parent_gap: 0.0,
        subgrid_gap: 0.0,
        children,
        scrollbar_gutter: SubgridAxisEdges::default(),
    })
}

#[test]
fn subgrid_traversal_keeps_edge_lower_bounds_off_non_intrinsic_tracks() {
    let default_input = crate::NodeInputOf::default();
    let report = traverse_subgrid_intrinsic(SubgridTraversalInput {
        ancestor_track_intrinsic_min_eligibility: IntrinsicMinTrackFacts::Known(&[false, false]),
        root_children: vec![SubgridTraversalChild::Subgrid(SubgridTraversalNode {
            min_frame: None,
            node: 1,
            style: default_grid_item_projection(&default_input),
            axis: SubgridTraversalAxis::Inherited,
            reversed: false,
            span_in_parent: GridTrackSpan::new(1, 3),
            align_self: AlignItems::Stretch,
            standalone_parent_context: None,
            margins: SubgridAxisEdges {
                start: 10.0,
                end: 12.0,
            },
            border: SubgridAxisEdges::default(),
            padding: SubgridAxisEdges::default(),
            parent_gap: 0.0,
            subgrid_gap: 0.0,
            children: Vec::new(),
            scrollbar_gutter: SubgridAxisEdges::default(),
        })],
    })
    .unwrap();

    assert_eq!(report.edge_lower_bounds, vec![0.0, 0.0]);
}

#[test]
fn subgrid_traversal_places_edge_lower_bounds_in_ancestor_track_space() {
    let default_input = crate::NodeInputOf::default();
    let report = traverse_subgrid_intrinsic(SubgridTraversalInput {
        ancestor_track_intrinsic_min_eligibility: IntrinsicMinTrackFacts::Known(&[
            true, true, true, true,
        ]),
        root_children: vec![SubgridTraversalChild::Subgrid(SubgridTraversalNode {
            min_frame: None,
            node: 1,
            style: default_grid_item_projection(&default_input),
            axis: SubgridTraversalAxis::Inherited,
            reversed: false,
            span_in_parent: GridTrackSpan::new(2, 5),
            align_self: AlignItems::Stretch,
            standalone_parent_context: None,
            margins: SubgridAxisEdges::default(),
            border: SubgridAxisEdges::default(),
            padding: SubgridAxisEdges {
                start: 20.0,
                end: 30.0,
            },
            parent_gap: 20.0,
            subgrid_gap: 10.0,
            children: vec![traversal_leaf(&default_input, 2, 1, 2)],
            scrollbar_gutter: SubgridAxisEdges::default(),
        })],
    })
    .unwrap();

    assert_eq!(report.edge_lower_bounds, vec![0.0, 20.0, 0.0, 30.0]);
}

#[test]
fn subgrid_traversal_reports_missing_intrinsic_min_facts() {
    let default_input = crate::NodeInputOf::default();
    let err = traverse_subgrid_intrinsic(SubgridTraversalInput {
        ancestor_track_intrinsic_min_eligibility: IntrinsicMinTrackFacts::Unknown,
        root_children: vec![traversal_subgrid(&default_input, 1, 1, 2, Vec::new())],
    })
    .unwrap_err();

    assert_eq!(err, SubgridTraversalError::MissingIntrinsicMinTrackFacts);
}

#[test]
fn subgrid_traversal_accumulates_edge_adjustment_in_nested_translated_span() {
    let default_input = crate::NodeInputOf::default();
    let report = traverse_subgrid_intrinsic(SubgridTraversalInput {
        ancestor_track_intrinsic_min_eligibility: IntrinsicMinTrackFacts::Known(&[
            true, true, true,
        ]),
        root_children: vec![SubgridTraversalChild::Subgrid(SubgridTraversalNode {
            min_frame: None,
            node: 1,
            style: default_grid_item_projection(&default_input),
            axis: SubgridTraversalAxis::Inherited,
            reversed: false,
            span_in_parent: GridTrackSpan::new(1, 4),
            align_self: AlignItems::Stretch,
            standalone_parent_context: None,
            margins: SubgridAxisEdges {
                start: 2.0,
                end: 4.0,
            },
            border: SubgridAxisEdges::default(),
            padding: SubgridAxisEdges::default(),
            parent_gap: 0.0,
            subgrid_gap: 0.0,
            children: vec![SubgridTraversalChild::Subgrid(SubgridTraversalNode {
                min_frame: None,
                node: 2,
                style: default_grid_item_projection(&default_input),
                axis: SubgridTraversalAxis::Inherited,
                reversed: false,
                span_in_parent: GridTrackSpan::new(2, 3),
                align_self: AlignItems::Stretch,
                standalone_parent_context: None,
                margins: SubgridAxisEdges {
                    start: 3.0,
                    end: 5.0,
                },
                border: SubgridAxisEdges::default(),
                padding: SubgridAxisEdges::default(),
                parent_gap: 0.0,
                subgrid_gap: 0.0,
                children: vec![traversal_leaf(&default_input, 3, 1, 2)],
                scrollbar_gutter: SubgridAxisEdges::default(),
            })],
            scrollbar_gutter: SubgridAxisEdges::default(),
        })],
    })
    .unwrap();

    assert_eq!(report.leaves[0].ancestor_span, GridTrackSpan::new(2, 3));
    assert_eq!(report.edge_adjustments_for_leaf(0), vec![2.0, 8.0, 4.0]);
}

#[test]
fn subgrid_traversal_accumulates_deep_track_adjustments() {
    let count = 4096;
    let input = NodeInputOf::<f64>::default();
    for padding in [0.0, 1.0] {
        let mut child = SubgridTraversalChild::Leaf(SubgridTraversalLeaf {
            node: 0,
            style: default_grid_item_projection(&input),
            span_in_parent: GridTrackSpan::new(1, 2),
            align_self: AlignItems::Stretch,
            min_frame: None,
        });
        for node in 1..=count {
            child = SubgridTraversalChild::Subgrid(SubgridTraversalNode {
                min_frame: None,
                node,
                style: default_grid_item_projection(&input),
                axis: SubgridTraversalAxis::Inherited,
                reversed: false,
                span_in_parent: GridTrackSpan::new(1, 2),
                align_self: AlignItems::Stretch,
                standalone_parent_context: None,
                margins: SubgridAxisEdges::default(),
                border: SubgridAxisEdges::default(),
                padding: SubgridAxisEdges {
                    start: padding,
                    end: 0.0,
                },
                scrollbar_gutter: SubgridAxisEdges::default(),
                parent_gap: 0.0,
                subgrid_gap: 0.0,
                children: vec![child],
            });
        }
        let report = traverse_subgrid_intrinsic(SubgridTraversalInput {
            ancestor_track_intrinsic_min_eligibility: IntrinsicMinTrackFacts::Known(&[true]),
            root_children: vec![child],
        })
        .unwrap();

        let expected = f64::from(count) * padding;
        assert_eq!(report.edge_lower_bounds, vec![expected]);
        assert_eq!(report.leaves.len(), 1);
        let (scalar, baseline) = report
            .adjustments
            .for_leaf(&report.leaves[0], &mut Vec::new())
            .unwrap();
        assert_eq!(
            (scalar, baseline.start, baseline.end),
            (expected, expected, 0.0)
        );
    }
}

#[test]
fn subgrid_traversal_places_many_padded_candidates() {
    let count = 4096;
    let input = NodeInputOf::<f64>::default();
    let candidate = |track| {
        SubgridTraversalChild::Subgrid(SubgridTraversalNode {
            min_frame: None,
            node: 2,
            style: default_grid_item_projection(&input),
            axis: SubgridTraversalAxis::Inherited,
            reversed: false,
            span_in_parent: GridTrackSpan::new(track, track + 1),
            align_self: AlignItems::Stretch,
            standalone_parent_context: None,
            margins: SubgridAxisEdges::default(),
            border: SubgridAxisEdges::default(),
            padding: SubgridAxisEdges {
                start: 1.0,
                end: 0.0,
            },
            scrollbar_gutter: SubgridAxisEdges::default(),
            parent_gap: 0.0,
            subgrid_gap: 0.0,
            children: vec![SubgridTraversalChild::Leaf(SubgridTraversalLeaf {
                node: 3,
                style: default_grid_item_projection(&input),
                span_in_parent: GridTrackSpan::new(1, 2),
                align_self: AlignItems::Stretch,
                min_frame: None,
            })],
        })
    };
    let report = traverse_subgrid_intrinsic(SubgridTraversalInput {
        ancestor_track_intrinsic_min_eligibility: IntrinsicMinTrackFacts::Known(&vec![true; count]),
        root_children: vec![SubgridTraversalChild::Subgrid(SubgridTraversalNode {
            min_frame: None,
            node: 1,
            style: default_grid_item_projection(&input),
            axis: SubgridTraversalAxis::Inherited,
            reversed: false,
            span_in_parent: GridTrackSpan::new(1, count + 1),
            align_self: AlignItems::Stretch,
            standalone_parent_context: None,
            margins: SubgridAxisEdges::default(),
            border: SubgridAxisEdges::default(),
            padding: SubgridAxisEdges::default(),
            scrollbar_gutter: SubgridAxisEdges::default(),
            parent_gap: 0.0,
            subgrid_gap: 0.0,
            children: (1..=count).map(candidate).collect(),
        })],
    })
    .unwrap();

    assert_eq!(report.edge_lower_bounds, vec![1.0; count]);
    assert_eq!(report.leaves.len(), count);
    let mut path = Vec::new();
    for (index, leaf) in report.leaves.iter().enumerate() {
        assert_eq!(leaf.ancestor_span, GridTrackSpan::new(index + 1, index + 2));
        let (scalar, baseline) = report.adjustments.for_leaf(leaf, &mut path).unwrap();
        assert_eq!((scalar, baseline.start, baseline.end), (1.0, 1.0, 0.0));
    }
}

#[test]
fn subgrid_traversal_accumulates_gap_adjustment_through_nested_subgrids() {
    let default_input = crate::NodeInputOf::default();
    let report = traverse_subgrid_intrinsic(SubgridTraversalInput {
        ancestor_track_intrinsic_min_eligibility: IntrinsicMinTrackFacts::Known(&[
            true, true, true,
        ]),
        root_children: vec![SubgridTraversalChild::Subgrid(SubgridTraversalNode {
            min_frame: None,
            node: 1,
            style: default_grid_item_projection(&default_input),
            axis: SubgridTraversalAxis::Inherited,
            reversed: false,
            span_in_parent: GridTrackSpan::new(1, 4),
            align_self: AlignItems::Stretch,
            standalone_parent_context: None,
            margins: SubgridAxisEdges::default(),
            border: SubgridAxisEdges::default(),
            padding: SubgridAxisEdges::default(),
            parent_gap: 10.0,
            subgrid_gap: 20.0,
            children: vec![SubgridTraversalChild::Subgrid(SubgridTraversalNode {
                min_frame: None,
                node: 2,
                style: default_grid_item_projection(&default_input),
                axis: SubgridTraversalAxis::Inherited,
                reversed: false,
                span_in_parent: GridTrackSpan::new(2, 3),
                align_self: AlignItems::Stretch,
                standalone_parent_context: None,
                margins: SubgridAxisEdges::default(),
                border: SubgridAxisEdges::default(),
                padding: SubgridAxisEdges::default(),
                parent_gap: 20.0,
                subgrid_gap: 28.0,
                children: vec![traversal_leaf(&default_input, 3, 1, 2)],
                scrollbar_gutter: SubgridAxisEdges::default(),
            })],
            scrollbar_gutter: SubgridAxisEdges::default(),
        })],
    })
    .unwrap();

    assert_eq!(report.gap_adjustments_for_leaf(0), vec![5.0, 10.0, 5.0]);
}

#[test]
fn subgrid_traversal_applies_gap_adjustment_to_internal_edges() {
    let default_input = crate::NodeInputOf::default();
    let report = traverse_subgrid_intrinsic(SubgridTraversalInput {
        ancestor_track_intrinsic_min_eligibility: IntrinsicMinTrackFacts::Known(&[
            true, true, true,
        ]),
        root_children: vec![SubgridTraversalChild::Subgrid(SubgridTraversalNode {
            min_frame: None,
            node: 1,
            style: default_grid_item_projection(&default_input),
            axis: SubgridTraversalAxis::Inherited,
            reversed: false,
            span_in_parent: GridTrackSpan::new(1, 4),
            align_self: AlignItems::Stretch,
            standalone_parent_context: None,
            margins: SubgridAxisEdges::default(),
            border: SubgridAxisEdges::default(),
            padding: SubgridAxisEdges::default(),
            parent_gap: 10.0,
            subgrid_gap: 20.0,
            children: vec![traversal_leaf(&default_input, 2, 2, 3)],
            scrollbar_gutter: SubgridAxisEdges::default(),
        })],
    })
    .unwrap();

    assert_eq!(report.leaves[0].ancestor_span, GridTrackSpan::new(2, 3));
    assert_eq!(report.gap_adjustments_for_leaf(0), vec![5.0, 10.0, 5.0]);
}

#[test]
fn subgrid_traversal_uses_positive_gap_adjustments_as_empty_track_lower_bounds() {
    let default_input = crate::NodeInputOf::default();
    let report = traverse_subgrid_intrinsic(SubgridTraversalInput {
        ancestor_track_intrinsic_min_eligibility: IntrinsicMinTrackFacts::Known(&[
            true, true, true, true,
        ]),
        root_children: vec![SubgridTraversalChild::Subgrid(SubgridTraversalNode {
            min_frame: None,
            node: 1,
            style: default_grid_item_projection(&default_input),
            axis: SubgridTraversalAxis::Inherited,
            reversed: false,
            span_in_parent: GridTrackSpan::new(1, 5),
            align_self: AlignItems::Stretch,
            standalone_parent_context: None,
            margins: SubgridAxisEdges::default(),
            border: SubgridAxisEdges::default(),
            padding: SubgridAxisEdges::default(),
            parent_gap: 0.0,
            subgrid_gap: 10.0,
            children: Vec::new(),
            scrollbar_gutter: SubgridAxisEdges::default(),
        })],
    })
    .unwrap();

    assert_eq!(report.edge_lower_bounds, vec![5.0, 10.0, 10.0, 5.0]);
}

#[test]
fn subgrid_traversal_combines_empty_edge_and_gap_lower_bounds() {
    let default_input = crate::NodeInputOf::default();
    let report = traverse_subgrid_intrinsic(SubgridTraversalInput {
        ancestor_track_intrinsic_min_eligibility: IntrinsicMinTrackFacts::Known(&[
            true, true, true, true,
        ]),
        root_children: vec![SubgridTraversalChild::Subgrid(SubgridTraversalNode {
            min_frame: None,
            node: 1,
            style: default_grid_item_projection(&default_input),
            axis: SubgridTraversalAxis::Inherited,
            reversed: false,
            span_in_parent: GridTrackSpan::new(1, 5),
            align_self: AlignItems::Stretch,
            standalone_parent_context: None,
            margins: SubgridAxisEdges::default(),
            border: SubgridAxisEdges::default(),
            padding: SubgridAxisEdges {
                start: 21.0,
                end: 9.0,
            },
            parent_gap: 10.0,
            subgrid_gap: 20.0,
            children: Vec::new(),
            scrollbar_gutter: SubgridAxisEdges::default(),
        })],
    })
    .unwrap();

    assert_eq!(report.edge_lower_bounds, vec![26.0, 10.0, 10.0, 14.0]);
}

#[test]
fn subgrid_traversal_ignores_gap_adjustment_for_single_track_subgrid() {
    let default_input = crate::NodeInputOf::default();
    let report = traverse_subgrid_intrinsic(SubgridTraversalInput {
        ancestor_track_intrinsic_min_eligibility: IntrinsicMinTrackFacts::Known(&[true]),
        root_children: vec![SubgridTraversalChild::Subgrid(SubgridTraversalNode {
            min_frame: None,
            node: 1,
            style: default_grid_item_projection(&default_input),
            axis: SubgridTraversalAxis::Inherited,
            reversed: false,
            span_in_parent: GridTrackSpan::new(1, 2),
            align_self: AlignItems::Stretch,
            standalone_parent_context: None,
            margins: SubgridAxisEdges::default(),
            border: SubgridAxisEdges::default(),
            padding: SubgridAxisEdges::default(),
            parent_gap: 10.0,
            subgrid_gap: 30.0,
            children: vec![traversal_leaf(&default_input, 2, 1, 2)],
            scrollbar_gutter: SubgridAxisEdges::default(),
        })],
    })
    .unwrap();

    assert_eq!(report.gap_adjustments_for_leaf(0), vec![0.0]);
}

#[test]
fn subgrid_traversal_keeps_standalone_as_one_boundary_leaf() {
    let default_input = crate::NodeInputOf::default();
    let report = traverse_subgrid_intrinsic(SubgridTraversalInput {
        ancestor_track_intrinsic_min_eligibility: IntrinsicMinTrackFacts::Known(&[true]),
        root_children: vec![SubgridTraversalChild::Subgrid(SubgridTraversalNode {
            min_frame: None,
            node: 1,
            style: default_grid_item_projection(&default_input),
            axis: SubgridTraversalAxis::Standalone,
            reversed: false,
            span_in_parent: GridTrackSpan::new(1, 2),
            align_self: AlignItems::Stretch,
            standalone_parent_context: None,
            margins: SubgridAxisEdges::default(),
            border: SubgridAxisEdges::default(),
            padding: SubgridAxisEdges::default(),
            parent_gap: 0.0,
            subgrid_gap: 0.0,
            children: vec![traversal_leaf(&default_input, 2, 1, 2)],
            scrollbar_gutter: SubgridAxisEdges::default(),
        })],
    })
    .unwrap();

    assert_eq!(report.leaves.len(), 1);
    assert_eq!(report.leaves[0].node, 1);
    assert_eq!(report.leaves[0].ancestor_span, GridTrackSpan::new(1, 2));
    assert!(report.leaves[0].standalone_parent_context.is_none());
}

#[path = "lanes_subgrid/lanes_minmax_authority.rs"]
mod lanes_minmax_authority;

#[path = "lanes_subgrid/inherited_axis_pair.rs"]
mod inherited_axis_pair;

#[path = "lanes_subgrid/placement_contracts.rs"]
mod placement_contracts;

#[path = "lanes_subgrid/alignment_contracts.rs"]
mod alignment_contracts;

#[path = "lanes_subgrid/baseline_contracts.rs"]
mod baseline_contracts;

#[path = "lanes_subgrid/membership_contracts.rs"]
mod membership_contracts;

#[path = "lanes_subgrid/sizing_alignment_contracts.rs"]
mod sizing_alignment_contracts;

#[path = "lanes_subgrid/positioned_static_contracts.rs"]
mod positioned_static_contracts;

#[path = "lanes_subgrid/positioned_line_contracts.rs"]
mod positioned_line_contracts;

fn assert_resolved_lanes_reverse_physical_geometry<S: LayoutScalar>() {
    let px = S::from_f64;
    for mode in [
        WritingMode::HorizontalTb,
        WritingMode::VerticalRl,
        WritingMode::VerticalLr,
        WritingMode::SidewaysRl,
        WritingMode::SidewaysLr,
    ] {
        for direction in [Direction::Ltr, Direction::Rtl] {
            for axis in [GridAxisKind::Column, GridAxisKind::Row] {
                let tree = PublicLayoutTreeOf::new()
                    .children(1, [2, 3])
                    .style(
                        1,
                        NodeInputOf {
                            display: Display::GridLanes,
                            writing_mode: mode,
                            direction,
                            size: Size::splat_clone(PreferredSizeOf::px(px(100.0))),
                            grid_lanes_flow: crate::GridLanesFlow::new(axis, true, false),
                            grid_template_columns: vec![TrackComponentOf::px(px(20.0)); 2],
                            grid_template_rows: vec![TrackComponentOf::px(px(20.0)); 2],
                            grid_flow_tolerance: GridFlowToleranceOf::Length(LengthOf::ZERO),
                            justify_content: Some(AlignContent::Start),
                            align_content: Some(AlignContent::Start),
                            ..NodeInputOf::default()
                        },
                    )
                    .style(
                        2,
                        NodeInputOf {
                            size: Size::new(
                                PreferredSizeOf::px(px(5.0)),
                                PreferredSizeOf::px(px(10.0)),
                            ),
                            ..NodeInputOf::default()
                        },
                    )
                    .style(
                        3,
                        NodeInputOf {
                            size: Size::new(
                                PreferredSizeOf::px(px(5.0)),
                                PreferredSizeOf::px(px(10.0)),
                            ),
                            ..NodeInputOf::default()
                        },
                    );
                let batch =
                    compute_layout(&tree, 1, super::fixtures::fri08_c01_placement_request())
                        .unwrap();
                for (node, track_start) in [(2, 20.0), (3, 0.0)] {
                    // Reversal visits the second 20-unit track first. Independent
                    // physical expectations use the 100-unit box minus the
                    // fixed child extent on each negative-progression axis.
                    let (inline, block) = if axis == GridAxisKind::Column {
                        (track_start, 0.0)
                    } else {
                        (0.0, track_start)
                    };
                    let rtl = direction == Direction::Rtl;
                    let (x, y) = match mode {
                        WritingMode::HorizontalTb => {
                            (if rtl { 95.0 - inline } else { inline }, block)
                        }
                        WritingMode::VerticalRl | WritingMode::SidewaysRl => {
                            (95.0 - block, if rtl { 90.0 - inline } else { inline })
                        }
                        WritingMode::VerticalLr => {
                            (block, if rtl { 90.0 - inline } else { inline })
                        }
                        WritingMode::SidewaysLr => {
                            (block, if rtl { inline } else { 90.0 - inline })
                        }
                    };
                    assert_eq!(
                        fri08_c01_placement_output(&batch, node).location,
                        Point::new(px(x), px(y)),
                        "{mode:?} {direction:?} {axis:?} node {node}"
                    );
                }
            }
        }
    }
}

#[test]
fn resolved_lanes_reverse_respects_physical_writing_modes() {
    assert_resolved_lanes_reverse_physical_geometry::<f32>();
    assert_resolved_lanes_reverse_physical_geometry::<f64>();
}

fn assert_resolved_lanes_reverse_dense_opening<S: LayoutScalar>() {
    let px = S::from_f64;
    for dense in [false, true] {
        let tree = PublicLayoutTreeOf::new()
            .children(1, [2, 3, 4])
            .style(
                1,
                NodeInputOf {
                    display: Display::GridLanes,
                    size: Size::new(
                        PreferredSizeOf::px(px(30.0)),
                        PreferredSizeOf::px(px(100.0)),
                    ),
                    grid_lanes_flow: crate::GridLanesFlow::new(GridAxisKind::Column, true, dense),
                    grid_template_columns: vec![TrackComponentOf::px(px(10.0)); 3],
                    grid_flow_tolerance: GridFlowToleranceOf::Length(LengthOf::ZERO),
                    justify_content: Some(AlignContent::Start),
                    align_content: Some(AlignContent::Start),
                    ..NodeInputOf::default()
                },
            )
            .style(
                2,
                NodeInputOf {
                    grid_column: GridPlacement::try_line(3).unwrap(),
                    size: Size::new(PreferredSizeOf::AUTO, PreferredSizeOf::px(px(40.0))),
                    ..NodeInputOf::default()
                },
            )
            .style(
                3,
                NodeInputOf {
                    grid_column: GridPlacement::try_lines(2, 4).unwrap(),
                    size: Size::new(PreferredSizeOf::AUTO, PreferredSizeOf::px(px(10.0))),
                    ..NodeInputOf::default()
                },
            )
            .style(
                4,
                NodeInputOf {
                    size: Size::new(PreferredSizeOf::AUTO, PreferredSizeOf::px(px(10.0))),
                    ..NodeInputOf::default()
                },
            );
        let batch =
            compute_layout(&tree, 1, super::fixtures::fri08_c01_placement_request()).unwrap();
        // The 40-high explicit item in track 3 lifts the bridge over tracks
        // 2 and 3, leaving a 40-high opening in track 2. At equal offset zero,
        // reverse dense order prefers this opening over the empty first track.
        assert_eq!(
            fri08_c01_placement_output(&batch, 2).location,
            Point::new(px(20.0), S::ZERO)
        );
        assert_eq!(
            fri08_c01_placement_output(&batch, 3).location,
            Point::new(px(10.0), px(40.0))
        );
        assert_eq!(
            fri08_c01_placement_output(&batch, 4).location,
            Point::new(if dense { px(10.0) } else { S::ZERO }, S::ZERO)
        );
    }
}

#[test]
fn resolved_lanes_reverse_dense_prefers_equal_offset_openings() {
    assert_resolved_lanes_reverse_dense_opening::<f32>();
    assert_resolved_lanes_reverse_dense_opening::<f64>();
}
